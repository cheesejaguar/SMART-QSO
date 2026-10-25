# Hardware-in-the-Loop (HIL) Test Plan

**Document ID**: SMART-QSO-HIL-001
**Version**: 1.3
**Date**: 2026-10-25
**Classification**: Engineering

## 1. Introduction

### 1.1 Purpose

This document defines the Hardware-in-the-Loop (HIL) testing approach for SMART-QSO flight software verification. HIL testing bridges the gap between unit/integration testing and full system testing by executing flight software on target-representative hardware.

### 1.2 Scope

HIL testing covers:
- OBC flight software execution on STM32L4/Apollo4 target
- Simulated subsystem interfaces (EPS, ADCS, RF, Payload)
- Real-time execution with hardware timers and watchdogs
- Communication protocol validation

### 1.3 References

- NPR 7150.2 - NASA Software Engineering Requirements
- NASA-STD-8739.8 - Software Assurance Standard
- SMART-QSO-STP-001 - Software Test Plan
- SMART-QSO-SDD-001 - Software Design Document

### 1.4 Status and Target (Version 1.1, 2026-10-24; 1.3, 2026-10-25)

None of these tests has been run, and none of the software components in §2.3 exists yet: this directory holds only
this plan. Three probe scripts exist for HIL-004, HIL-040 and HIL-044 (§4.4). Changes since version 1.0:

- **Target.** The OBC is the STM32L4R5ZI (`docs/BASELINE_PARAMETERS.md` §6). The STM32L476 Nucleo-64 and the Apollo4
  in §2.2 and §7.1 are superseded. A development board with the STM32L4R5ZI is needed. Its pins must be mapped to the
  provisional assignment in `software/rust/firmware/src/gpio.rs` until the OBC schematic baselines one.
- **Image.** The flight software under test is the Rust firmware (`software/rust/firmware`). Build it with
  `cargo build --release` in that directory, and flash and run it with `cargo run --release` (the probe-rs runner in
  `software/rust/firmware/.cargo/config.toml`) instead of STM32CubeProgrammer. "Real HAL" in §2.1 is the firmware's
  Rust HAL (`software/rust/README.md`, section The firmware).
- **Interfaces.** The console is the debug UART (USART3, PD8/PD9). The Jetson link is USART1 (PA9/PA10). Telecommands
  arrive as KISS frames from the TNC on USART2 (PD5/PD6, 9600 baud), so HIL-031 needs a KISS sender.
- **Stack (SRS-C003).** The firmware arms an MPU guard just below the 8 KB stack budget and paints the budget at boot
  (`software/rust/firmware/src/stack.rs`, `software/flight/docs/MEMORY_POLICY.md` §6). HIL-004 overruns the budget;
  HIL-044 reads the high-water mark through the probe and compares it with the static worst case
  (`scripts/stack_usage.py`).
- **Watchdog.** HIL-043 expects a kick at least every 1 s; the firmware refreshes the IWDG on every 20 ms pass while no
  stage is stalled (`software/flight/docs/RTOS_ARCHITECTURE.md` §8), and a stalled stage stops the kicks on purpose.
- **Bring-up (1.3).** The first-day order of checks on the board, the NUCLEO-L4R5ZI's differences from the flight pin
  map (the ST-LINK virtual COM port is LPUART1 on PG7/PG8, not the USART3 console) and what to record are in
  `software/rust/firmware/BRINGUP.md`. The firmware times every main-loop pass for HIL-040 (`SQSO_LOOP_TIMING`,
  `software/rust/firmware/src/timing.rs`).

## 2. HIL Test Architecture

### 2.1 System Configuration

```
+------------------------------------------------------------------+
|                    HIL Test Configuration                         |
|                                                                    |
|  +------------------+         +------------------+                 |
|  |  HOST COMPUTER   |         |  OBC TARGET      |                 |
|  |                  |  UART   |  (STM32L4)       |                 |
|  |  Test Executive  |<------->|  Flight SW       |                 |
|  |  Subsystem Sims  |         |  Real HAL        |                 |
|  |  Data Logging    |         |  HW Watchdog     |                 |
|  +------------------+         +--------+---------+                 |
|                                        |                           |
|                               +--------+---------+                 |
|                               |  BREAKOUT BOARD  |                 |
|                               |  GPIO Monitoring |                 |
|                               |  I2C Simulation  |                 |
|                               |  SPI Simulation  |                 |
|                               +------------------+                 |
+------------------------------------------------------------------+
```

### 2.2 Hardware Components

| Component | Description | Purpose |
|-----------|-------------|---------|
| OBC Dev Board | STM32L476 Nucleo or Apollo4 EVK | Target execution |
| USB-UART Adapter | Debug/command interface | Host communication |
| Logic Analyzer | 8+ channel digital analyzer | Protocol verification |
| Breakout Board | GPIO/I2C/SPI breakout | Interface testing |
| Power Supply | Programmable 3.3V/5V supply | Power testing |

### 2.3 Software Components

| Component | Description | Location |
|-----------|-------------|----------|
| Test Executive | Python test framework | `tests/hil/hil_executive.py` |
| EPS Simulator | Power subsystem simulation | `tests/hil/sim_eps.py` |
| ADCS Simulator | Attitude simulation | `tests/hil/sim_adcs.py` |
| RF Simulator | Beacon/telemetry simulation | `tests/hil/sim_rf.py` |
| Jetson Simulator | Payload response simulation | `tests/hil/sim_jetson.py` |

## 3. HIL Test Cases

### 3.1 Startup Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-001 | Cold Start | Power-on initialization | System reaches SAFE mode within 5s |
| HIL-002 | Warm Start | Reset after fault | State restored from NVM |
| HIL-003 | Watchdog Timeout | No SW activity | Hardware reset occurs |
| HIL-004 | Stack Overflow | Halt with the probe, set SP to 0x2009E010 (16 bytes above the guard at 0x2009DFE0), resume (§4.4, `guard_test.py`) | MPU guard faults; IWDG resets the processor; the next boot reports a watchdog reset (`hal_reset_cause`) |

### 3.2 EPS Interface Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-010 | I2C Read | Read EPS telemetry | Correct values within 100ms |
| HIL-011 | I2C Write | Set power mode | Mode confirmed within 50ms |
| HIL-012 | I2C Timeout | No EPS response | Timeout fault logged |
| HIL-013 | Battery Monitor | Low SOC simulation | Power mode transitions |

### 3.3 ADCS Interface Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-020 | Magnetometer Read | Read mag sensor | Data within valid range |
| HIL-021 | Gyro Read | Read rate gyro | Data within valid range |
| HIL-022 | Actuator Command | Set MTQ polarity | GPIO state correct |
| HIL-023 | Detumble Algorithm | Rate reduction | Rates decrease over time |

### 3.4 Communication Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-030 | UART TX | Beacon transmission | Frame correct, timing met |
| HIL-031 | UART RX | Command reception | Command executed correctly |
| HIL-032 | Jetson Protocol | Payload request/response | Response within timeout |
| HIL-033 | Fallback Mode | Jetson timeout | Fallback beacon used |

### 3.5 Timing Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-040 | Main Loop Timing | Time of each `Executive::step()` pass, from the DWT cycle counter (`SQSO_LOOP_TIMING`; §4.4, `loop_timing.py`) | ≤ 20 ms per pass, no overrun counted (SRS-P001) |
| HIL-041 | Beacon Interval | Periodic beacon | Within ±1s of target |
| HIL-042 | Telemetry Rate | Telemetry generation | 1Hz ± 10% |
| HIL-043 | Watchdog Kick | WDT service interval | < 1s between kicks |
| HIL-044 | Stack High Water | Run every timing and fault-injection test, then read the painted budget (0x2009E000-0x2009FFFF) with the probe; used bytes = 8192 - 4 x (leading words still 0x57ACC0DE) (§4.4, `stack_high_water.py`) | Used ≤ the worst case in `stack_usage.txt` of the image under test, and ≤ 8 KB (SRS-C003) |

### 3.6 Fault Injection Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-050 | Sensor Fault | Invalid sensor data | Fault logged, safe values used |
| HIL-051 | Comm Fault | UART disconnect | Timeout handled, no crash |
| HIL-052 | Power Fault | Low voltage | Emergency mode entered |
| HIL-053 | Memory Fault | CRC mismatch | Data rejected, default used |

## 4. Test Procedures

### 4.1 Test Environment Setup

1. Connect OBC target to host via USB-UART
2. Connect breakout board for GPIO/I2C monitoring
3. Configure logic analyzer for protocol capture
4. Start test executive software
5. Verify communication with target

### 4.2 Test Execution

```bash
# Run all HIL tests
cd software/flight/tests/hil
python hil_executive.py --port /dev/ttyUSB0 --all

# Run specific test suite
python hil_executive.py --port /dev/ttyUSB0 --suite startup

# Run single test
python hil_executive.py --port /dev/ttyUSB0 --test HIL-001
```

### 4.3 Data Collection

| Data Type | Collection Method | Storage |
|-----------|-------------------|---------|
| Test results | JSON/XML report | `results/hil_results.json` |
| Protocol captures | Logic analyzer export | `results/captures/` |
| Console logs | Serial log capture | `results/logs/` |
| Timing data | Timestamp analysis | `results/timing/` |

### 4.4 Probe Procedures (HIL-004, HIL-040, HIL-044)

Three scripts in `software/rust/firmware/tools/` run these tests through a debug probe with the `probe-rs` command line
(`read` and `write`; written against probe-rs 0.32.0) and take symbol addresses from the image under test with `nm`.
Each prints PASS or FAIL and exits 0 (pass), 1 (fail) or 2 (error). `ELF` is the image flashed, from
`scripts/release_package.sh`, whose `stack_usage.txt` belongs to the same build. The order of these and the other first
checks is in `software/rust/firmware/BRINGUP.md` §5.

| Test | Command | What it does |
|------|---------|--------------|
| HIL-040 | `loop_timing.py --elf ELF --samples 5 --interval 10` | Reads `SQSO_LOOP_TIMING` (last and longest pass in 80 MHz cycles, passes over 20 ms, passes recorded) without halting the core; fails if the longest pass exceeds 20 ms, any overrun was counted, or the pass count does not grow between samples |
| HIL-044 | `stack_high_water.py --stack-usage stack_usage.txt --elf ELF --save stack_dump.txt` | Reads the 2,048 words of the budget without halting the core; fails if the used depth exceeds the static worst case in `stack_usage.txt`, or no painted word is left |
| HIL-004 | `guard_test.py --elf ELF` | Freezes the IWDG while the core is halted (DBGMCU_APB1FZR1.DBG_IWDG_STOP), halts, sets SP to 0x2009E010 through DCRDR/DCRSR, clears the vector catches, resumes; polls CFSR, HFSR, MMFAR until the processor resets; reads the reset cause the firmware cached at boot (`RESET_CAUSE`, `software/rust/firmware/src/wdt.rs`); restores DBGMCU_APB1FZR1. Passes on a MemManage stacking error or a data access violation inside 0x2009DFE0-0x2009DFFF, then a reset, then reset cause WATCHDOG |

HIL-044 is run after the other timing and fault-injection tests and before any reset, which repaints the budget.
HIL-004 resets the processor, so it is run last. The firmware clears the RCC_CSR reset flags at boot, so HIL-004 checks
the reset cause the firmware recorded, not IWDGRSTF. Without a board, `--dump` (HIL-040, HIL-044) evaluates saved
`probe-rs read` output and `guard_test.py --dry-run` runs against a simulated board; the unit tests are
`python3 -m pytest -q software/rust/firmware/tools/tests`.

## 5. Pass/Fail Criteria

### 5.1 Individual Test

- Test passes if all assertions satisfied
- Test passes if no unexpected faults occur
- Test passes if timing requirements met

### 5.2 Test Suite

- Suite passes if all tests pass
- Suite passes with notes if <5% tests require rerun

### 5.3 HIL Campaign

- Campaign passes if all suites pass
- All safety-critical tests must pass
- Known issues must be documented

## 6. Test Schedule

| Phase | Tests | Duration |
|-------|-------|----------|
| Setup Verification | HIL-001 to HIL-004 | 1 day |
| Interface Tests | HIL-010 to HIL-033 | 2 days |
| Timing Tests | HIL-040 to HIL-044 | 1 day |
| Fault Injection | HIL-050 to HIL-053 | 2 days |
| Regression | All tests | 1 day |

## 7. Resources Required

### 7.1 Hardware

- 1x STM32L476 Nucleo-64 board
- 1x FTDI USB-UART adapter
- 1x Saleae Logic 8 analyzer
- 1x Custom breakout board
- 1x Programmable power supply

### 7.2 Software

- Python 3.8+ with pyserial, pytest
- STM32CubeProgrammer
- Logic 2 analyzer software
- Serial terminal (minicom/screen)

### 7.3 Personnel

- 1x Software test engineer
- 1x Embedded software engineer (support)

## 8. Risk Management

| Risk | Mitigation |
|------|------------|
| Hardware unavailable | Use simulation mode fallback |
| Protocol timing issues | Add configurable delays |
| Flaky tests | Implement retry logic with limits |
| Environment differences | Document and control test environment |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-24 | SMART-QSO Team | §1.4: the target is the STM32L4R5ZI and the image is the Rust firmware (cargo, probe-rs); interfaces of the firmware; criteria to revisit (HIL-004, HIL-040, HIL-043); nothing run or written yet |
| 1.2 | 2026-10-25 | SMART-QSO Team | HIL-040 pass criterion 20 ms, as SRS-P001 requires (was 100 ms); HIL-004 against the MPU stack guard; HIL-044 stack high-water mark added (SRS-C003) |
| 1.3 | 2026-10-25 | SMART-QSO Team | §4.4 probe procedures for HIL-004, HIL-040 and HIL-044 (scripts in `software/rust/firmware/tools/`); HIL-040 measured per `Executive::step()` pass with the DWT cycle counter, ≤ 20 ms and no overrun; §1.4 points to the bring-up checklist (`software/rust/firmware/BRINGUP.md`) |

---

*This document follows NASA Ames SmallSat HIL testing heritage patterns.*
