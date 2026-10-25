# RTOS Architecture
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-011
**Revision:** B
**Date:** 2026-10-24
**Reference:** NPR 7150.2, Embassy (embassy-executor 0.10, embassy-stm32 0.6, embassy-time 0.5), NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the run-time architecture of the SMART-QSO flight software on the OBC (STM32L4R5ZI): how the
work is scheduled, which interrupts run, how the watchdog supervises the main loop, and the timing. There is no RTOS.
The image runs on the Embassy async executor with one task. Revision A described a FreeRTOS design that was never
implemented. §13 keeps a short history note on it.

## 2. Scope

This document covers:
- The executor and why there is no RTOS (§3)
- The task and the stages of the main loop (§4)
- Interrupts and the data they hand to the loop (§5)
- Timing and scheduling (§6)
- Memory (§7)
- Watchdog supervision (§8)
- Power management, error handling and debugging (§9 to §11)

Sources: `software/rust/firmware/src/main.rs` (the image), `software/rust/sqso-ffi/src/executive.rs` (the main loop,
shared with the host simulation), `software/rust/sqso-ffi/src/watchdog_mgr.rs` and
`software/rust/sqso-fsw/src/watchdog.rs` (supervision). Nothing here has been measured on hardware: no hardware has run
the image.

---

## 3. RTOS Selection

### 3.1 No RTOS: the Embassy executor

FreeRTOS was never integrated and is not planned. Since 2026-10-25 the baseline is the Embassy executor with no RTOS
(`docs/BASELINE_PARAMETERS.md` §6 OBC row; requirement OBC-F001 in `hardware/obc/OBC_DESIGN.md`; change request
CR-2026-002). The flight image (`software/rust/firmware`) uses:

| Component | Version | Role |
|-----------|---------|------|
| embassy-executor | 0.10 (`platform-cortex-m`, `executor-thread`) | Runs the one async task; the core sleeps (WFE) when the task is waiting |
| embassy-stm32 | 0.6 (`stm32l4r5zi`, `dual-bank`, `time-driver-any`, `memory-x`, `rt`, `exti`) | Chip bring-up, clocks, peripheral drivers, the time driver |
| embassy-time | 0.5 (`tick-hz-32_768`) | `Timer::after_millis`; the HAL time base |
| cortex-m-rt | 0.7 | Vector table, `Reset`, the linker script `link.x` |

Reasons, from the design as built:
- One loop already does all the work in a fixed order (§4.2), so preemptive tasks would add shared-state locking and no
  function.
- The flight software runs the same passes on the host simulation (`sqso-sim`) and on the target. Only the wait between
  passes differs (§6.1), so the C test suites and the simulation exercise the target's control flow.
- The executor and the drivers allocate nothing. The image links no heap and no C library (CI checks it).

### 3.2 Target

| Item | Value |
|------|-------|
| MCU | STM32L4R5ZI (Cortex-M4F) |
| Core clock | 80 MHz: 4 MHz MSI through the PLL, M = 1, N = 40, R = 2; AHB and APB at the core clock (`clocks()` in `software/rust/firmware/src/main.rs`) |
| Rust target | `thumbv7em-none-eabihf` |
| Image (release) | text 115,876 B, data 19,764 B, bss 16,496 B; vector table at 0x08000000; initial SP 0x200A0000; entry `Reset` |

---

## 4. Task Architecture

### 4.1 One task

The image has one async task, `main` (`#[embassy_executor::main]`). It runs these steps in order:

1. Start the IWDG at its longest timeout (about 32 s, `wdt::start_boot_watchdog`) before anything else, so a PLL that
   never locks resets the OBC.
2. `embassy_stm32::init(clocks())`: clocks, the time driver, the peripherals.
3. Install the HAL drivers (GPIO first, with enable and burn lines low; FRAM; SPI; I2C; the three UARTs; the ADC).
4. `Executive::start()`: bring the flight software up in the order of `init_system` (time base, GPIO and flash,
   deployment, fault management, RF safety and telecommand, mission data, EPS, sensors, beacon, payload guard, system
   state and mode manager, watchdog manager, Jetson UART). If it returns `None`, the processor resets.
5. Loop: `flight.step()`, then `Timer::after_millis(20).await`.

The task never ends. There is no other task, no queue, no mutex and no semaphore.

### 4.2 The stages of one pass

`Executive::step()` (`software/rust/sqso-ffi/src/executive.rs`) runs these stages in this order. A stage checks in with
the watchdog manager when it completes:

| # | Stage | Work | Watchdog check-in |
|---|-------|------|-------------------|
| 1 | Mission state | Power model step, SOC to the EPS, power event to the mode manager, uptime and payload energy | — |
| 2 | Health (every 10 s) | Temperatures, bus voltage, SOC, Jetson link | `health` (period 20 s) |
| 3 | Main | — | `main` (period 1 s) |
| 4 | Jetson telemetry | When the payload is on and the telemetry interval (60 s default) has passed: TELEMETRY line and beacon request; UART health check; sensor polling | — |
| 5 | Payload | Jetson replies; the OBC-enforced payload limits (`payload_guard`) | `payload` (period 1 s) |
| 6 | Beacon | Beacon measurements; a beacon when one is due (silent while TX-OFF, loss-of-contact or RF-silence inhibits are set) | `beacon` (period 1 s) |
| 7 | Uplink | Telecommands from the receiver (`uplink_poll`); GSE key provisioning (umbilical only) | `uplink` (period 1 s) |
| 8 | Timers | Elapsed time to the contact monitor, RF silence and deployment | — |
| 9 | Persistence (every 100 passes) | Mission data and EPS state saved | — |
| 10 | Watchdog | `wdt_mgr_kick()`: task check once a second; hardware refresh unless a task is stalled (§8) | — |

---

## 5. Interrupts and Inter-Task Communication

There is one task, so there is no inter-task communication. Interrupts are bound in `software/rust/firmware/src/main.rs`
(`bind_interrupts!`) or by the Embassy time driver:

| Interrupt | Handler | Hands to the loop |
|-----------|---------|-------------------|
| USART1 (Jetson link, 115,200 baud) | embassy-stm32 `BufferedInterruptHandler` | Receive buffer 512 B, transmit buffer 256 B |
| USART2 (TNC, KISS, 9600 baud) | `BufferedInterruptHandler` | Receive buffer 1,024 B; the loop decodes KISS frames (`sqso_protocol::kiss`) in `hal_radio_*` |
| USART3 (debug console, 115,200 baud) | `BufferedInterruptHandler` | Receive buffer 64 B, transmit buffer 512 B |
| Time driver timer | embassy-stm32 `time-driver-any` | Wakes the executor when `Timer::after_millis` expires; 64-bit tick count at 32,768 Hz |

The interrupt handlers only move bytes between the peripheral and the buffers. The flight software reads the buffers
from the loop, with no blocking wait. No flight-software code runs in interrupt context. The HAL's callback timers and
the watchdog early-warning callback are not provided, because a callback from interrupt context would bypass the
module-state rules (`software/rust/firmware/src/timer.rs`). Module state is held in a busy-flag cell, so a call that
re-entered a module would get "busy" rather than alias its state (`docs/development/CODING_STANDARDS.md` §11.8).

---

## 6. Timing and Scheduling

### 6.1 Pass period

| Build | Wait between passes | Source |
|-------|---------------------|--------|
| Firmware | `Timer::after_millis(20).await`; the core sleeps in the executor | `software/rust/firmware/src/main.rs` (`TICK_MS`) |
| Host simulation | `sleep_us(20,000)`; 1000 passes (about 20 s), or until stopped with feature `flight` | `software/rust/sqso-ffi/src/executive.rs` (`run_host`) |

The 20 ms wait comes after the pass. The pass period is therefore 20 ms plus the time of the pass. It is not a fixed
rate, and a long pass delays the next one instead of being skipped. SRS-P001 (main loop within 20 ms) is not verified
on hardware: no execution time has been measured on the target (`software/flight/docs/TIMING_ANALYSIS.md`).

### 6.2 Slower activities

Activities slower than one pass are timed from the mission clock (milliseconds since boot, 64-bit) inside the stages:
the health check every 10 s, Jetson telemetry at the telemetry interval, the watchdog task check every 1 s, the beacon
interval by power mode (`beacon.h`), and persistence every 100 passes. Elapsed-time arithmetic is 64-bit or saturating
(`software/rust/README.md`, behaviour change 11).

### 6.3 Time-triggered scheduler

`scheduler.h` (`software/rust/sqso-fsw/src/scheduler.rs`) is a time-triggered task table with deadline monitoring. The
C test suites exercise it. The executive does not use it: the main loop runs its stages directly in the order of §4.2.

---

## 7. Memory Management

| Item | Value |
|------|-------|
| Heap | None. No crate uses `alloc`; the image links no heap (CI checks for `malloc`, `free`, `_sbrk`) |
| Task storage | Static, in the executor (`embassy-executor`) |
| UART buffers | Static `ConstStaticCell` arrays (§5) |
| Stack | One stack (cortex-m-rt), from the initial SP 0x200A0000 downward, 8 KB budget with an MPU guard below it. Worst case 5,004 B by static analysis (`software/flight/docs/MEMORY_POLICY.md` §6); not measured on hardware |

`software/flight/docs/MEMORY_POLICY.md` has the policy.

---

## 8. Watchdog Integration

### 8.1 Stage supervision (SW-15)

The watchdog manager (`software/rust/sqso-ffi/src/watchdog_mgr.rs` on `software/rust/sqso-fsw/src/watchdog.rs`) keeps a
table of the stages in §4.2. The executive registers them in `init_watchdog`: `main`, `uplink`, `payload` and `beacon`
with a period of 1 s, and `health` with 20 s. Then it starts the hardware watchdog with a 30 s timeout
(`WDT_HW_TIMEOUT_MS`).

- Each pass ends with `wdt_mgr_kick()`. Once a second it checks every registered stage. A stage that has not checked in
  within its period counts a miss. After 3 misses in a row (`MISS_THRESHOLD`) it is stalled. The stall is logged, and
  the pre-reset callback saves mission data and the EPS state.
- The hardware watchdog is refreshed on every pass while no stage is stalled. A stalled stage stops the refresh, and
  the IWDG resets the OBC within its 30 s timeout.
- If a pass never returns (a hang inside a stage), nothing refreshes the IWDG, and it resets the OBC after 30 s.
- Before the flight software starts, the boot watchdog (§4.1) covers the clock set-up and the driver installation.

### 8.2 Hardware

The IWDG runs on the 32 kHz LSI (`software/rust/firmware/src/wdt.rs`). It cannot be stopped once started, and it only
resets (no interrupt mode). The reset cause is read from the RCC CSR flags (`hal_reset.h`).

---

## 9. Power Management

Between passes the task waits on `Timer::after_millis(20)`, and the executor puts the core to sleep until an interrupt
(the time driver or a UART). No stop or standby mode is used. Power has not been measured on hardware.

---

## 10. Error Handling

| Event | Response |
|-------|----------|
| Rust panic (overflow check, broken invariant) | Fatal assertion: `report_panic` (`software/rust/sqso-ffi/src/lib.rs`) logs it and saves state through the assertion handler. Then the processor waits (WFI) for the IWDG, or resets itself through `SCB::sys_reset` if the IWDG was never started (`software/rust/firmware/src/main.rs`) |
| Driver set-up fails (UART) or `Executive::start()` returns `None` | `SCB::sys_reset` |
| Stage stalled or loop hung | IWDG reset (§8) |
| Stack overflow | Not detected. There is no stack guard (the stack grows down from the top of RAM) |

---

## 11. Debugging Support

- Console lines (`[SYSTEM]`, beacon TX, faults) go to the debug UART, USART3 (`software/rust/README.md`, behaviour
  change 17).
- `cargo run --release` in `software/rust/firmware` flashes and runs the image with probe-rs (runner in
  `software/rust/firmware/.cargo/config.toml`). The release profile keeps debug information (`debug = 2`).

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-24 | Rewritten for the design as built: no RTOS; the Embassy executor with one task, interrupt-driven UARTs and the time driver; the stages of a pass and their watchdog supervision; timing. The FreeRTOS design of Revision A, never implemented, kept as a history note (§13) | SMART-QSO Team |
| B, note | 2026-10-25 | §3.1: the Embassy executor with no RTOS is the baseline (BASELINE §6, OBC-F001, CR-2026-002) | SMART-QSO Team |

---

## 13. History Note: the FreeRTOS Design of Revision A (2026-01-02)

Revision A specified FreeRTOS 10.5.1 with CMSIS-RTOS2. It had nine tasks at priorities 0 to 4 (watchdog, sensor, ADCS,
main loop, beacon, command, telemetry, housekeeping, idle), queues between them, a telemetry mutex, a 1 kHz tick and a
32 KB `heap_4` heap. None of it was implemented: no FreeRTOS source, configuration or task code was ever in the
repository, and the C flight software ran one main loop until it was replaced by the Rust (2026-10-24). The heap in that
design would have conflicted with the no-dynamic-memory rule (JPL rule 3). The Revision A text is in the repository
history.
