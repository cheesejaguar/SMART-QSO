# Memory Allocation Policy
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-006
**Revision:** I
**Date:** 2026-10-27
**Reference:** JPL D-60411 (Power of Ten), NPR 7150.2

---

## 1. Purpose

This document defines the memory allocation policy for SMART-QSO flight software, ensuring deterministic behavior and compliance with JPL Power of Ten Rule 3 (no dynamic memory allocation after initialization).

**Revision B (2026-10-24).** The flight software is Rust (`software/rust/README.md`). This revision states the policy as
the code implements it. Revision A also described triple modular redundancy, buffer pools, stack canaries, an MPU stack
guard and SRAM scrubbing. None of these existed in the code. §7 and §10 say so, and they keep those items as design
intent only. Revision C (2026-10-25) adds the MPU stack guard, a painted stack budget and the static worst case (§6).

## 2. Scope

This policy applies to all flight software executing on the OBC (STM32L4R5ZI): the Rust crates `sqso-protocol`,
`sqso-fsw` and `sqso-ffi`, the firmware (`software/rust/firmware`), and the remaining C file
`software/flight/src/flight_log_fmt.c`. The host simulation's simulated HAL (`software/rust/sqso-ffi/src/hal_sim.rs`) is
not flight code, but it allocates nothing either.

---

## 3. Memory Architecture

### 3.1 Memory Map

The linker uses the firmware's own `memory.x` (`software/rust/firmware/memory.x`, since Revision E; it was the one
embassy-stm32 generates, feature `memory-x`), with the cortex-m-rt `link.x`:

| Region | Start | Size | Usage |
|--------|-------|------|-------|
| Flash | 0x08000000 | 2 MB (dual bank) | Vector table, code, constants, `.data` initial values |
| SRAM1 | 0x20000000 | 192 KB | Not linked. The NMI's cause record (12 B at 0x2002FFF0, §10.3), which survives a reset |
| SRAM2 | 0x20030000 | 64 KB, parity | `.data`, `.bss` and `.uninit` at its start: every static, the executor's task and the flight software's state (44,244 B on 2026-10-27, after round 3, fifth part); the 8 KB stack at its top, growing down from 0x20040000 (since Revision F, RID AV-13). The link fails if the statics, the 32-byte guard and the stack outgrow SRAM2 (`ASSERT` in `memory.x`) |
| SRAM3 | 0x20040000 | 384 KB | Not used |
| FRAM (FM25V20A, SPI1) | external | 256 KB | Non-volatile records (§8), on U3 (CS# PG5) and mirrored on U36 (CS# PD7; SQSO-ICX-001 D §6 row 32, RID SMA-B10) |

The linked RAM region is SRAM2 alone (0x20030000, 64 KB), so the initial SP is 0x20040000 (it was 0x200A0000 with
SRAM2 and SRAM3 linked; the CI image check pins the new value). Revision A split
SRAM into regions for "critical data (TMR)" and "buffers, logs", and listed a backup SRAM; nothing uses backup SRAM.

### 3.2 Memory Budget

Measured from the release image (2026-10-27, after flight software round 3, fifth part: a local build of the tree with
the pinned toolchain, `cargo build --release --locked`, `arm-none-eabi-size`, in the documentation audit; CI job STM32L4
Cross-Compile (library) prints the same):

| Category | Used | Available | Margin |
|----------|------|-----------|--------|
| Code and constants (text) | 211,604 B | 2 MB flash (SRS-C004: 256 KB) | 90 % (19 % of SRS-C004) |
| Initialized data (data) | 27,192 B | (in SRAM2; initial values in flash) | — |
| Zeroed data (bss) | 17,052 B | (in SRAM2) | — |
| Static RAM (data + bss) | 44,244 B | 64 KB SRAM2, less the 8 KB stack and its 32-byte guard (57,312 B) | 23 % (13,068 B free between the statics and the guard) |
| Stack | Worst case 5,812 B (static analysis, §6.2) | 8 KB budget below 0x20040000, in SRAM2 (SRS-C003) | 29 % |
| Heap | None | — | — |

---

## 4. Allocation Policy

### 4.1 Permitted Allocations

| Phase | Heap | Static | Stack |
|-------|:----:|:------:|:-----:|
| Initialization | **None** | Permitted | Permitted |
| Runtime | **None** | Permitted | Permitted |
| Shutdown | **None** | Permitted | Permitted |

The flight software allocates no dynamic memory at any time, initialization included. That is stricter than Rule 3.

### 4.2 Rule: No Dynamic Allocation

- No crate uses `alloc`: no `Box`, `Vec` or `String`. The crates are `no_std`, and there is no global allocator.
- The C shim formats into a stack buffer (`vsnprintf`) and allocates nothing.
- Interrupt handlers (embassy-stm32 UART handlers, the time driver) only move bytes into static buffers.

### 4.3 Enforcement

| Check | What it checks | Where |
|-------|----------------|-------|
| ctest `no_dynamic_memory` | No object of the test library, and nothing in the Rust library `libsqso_flight.a`, references `malloc`, `calloc`, `realloc` or `free` | `software/flight/scripts/check_no_dynamic_memory.sh`, every build |
| Image check | The firmware image has no `malloc`, `free`, `_sbrk` or `printf` (no heap and no C library linked) | CI job STM32L4 Cross-Compile (library) |
| Compiler | A `no_std` crate without `extern crate alloc` cannot name `Box` or `Vec`. The firmware has no `#[global_allocator]`, so code that allocates would not link | rustc |

---

## 5. Buffer Management

### 5.1 Static Buffers

All buffers have sizes fixed at compile time. There is no buffer pool and no allocate or free API.

| Buffer | Size | Where |
|--------|------|-------|
| Jetson UART receive / transmit | 512 B / 256 B | `software/rust/firmware/src/main.rs` (`ConstStaticCell`) |
| TNC UART receive / transmit | 1,024 B / 1,024 B (`RADIO_TX_LEN`, at least the longest KISS stream) | same |
| Debug UART receive / transmit | 64 B / 512 B | same |
| GNSS UART (LPUART1) receive / transmit | 256 B / 64 B | same |
| Module state (fault log 100 entries, flight log, sensor table of 32, key table, scheduler, watchdog table, ...) | Fixed arrays | One `cell::Global` per module in `software/rust/sqso-ffi/src` |
| Frame and line buffers | `LINE_MAX`, `TEXT_MAX`, receive frame sizes | Stack arrays in the function that uses them |

### 5.2 Buffer Allocation API

None. Revision A's `buffer_pool_alloc()`, `buffer_pool_free()` and `buffer_pool_available()` were never implemented.

### 5.3 Buffer Sizes

The sizes are the constants of the C headers, mirrored in Rust and checked by
`software/flight/tests/test_rust_contract.c`. For example, `SMART_QSO_MAX_SENSORS` = 32 and the fault log's 100 entries.
§5.1 lists the UART buffers.

---

## 6. Stack Management

SRS-C003: the stack shall not exceed 8 KB.

### 6.1 Stack Allocation

There is one stack (cortex-m-rt), from the initial SP 0x20040000 (the top of SRAM2) downward. The executor runs one
task, and the interrupt handlers use the same stack. The Embassy task's state (the async main function's locals held
across an `await`, including the executive's state) is static, in `.bss`, not on the stack. The budget is the 8 KB
below the initial SP, 0x2003E000 to 0x2003FFFF; the static data end at `__ebss` (0x2003ACD4 on 2026-10-27, after round 3, fifth part), 13,068 B
below the guard.

### 6.2 Worst Case (static analysis)

`scripts/stack_usage.py` computes the worst case from the image's machine code, and CI fails above 8 KB (job STM32L4
Cross-Compile (library), step Worst-case stack). It sums each function's frame along the deepest call path from the
reset handler, and adds one interrupt handler per priority level with a 108-byte exception frame (FPU state). The frames
are cross-checked against LLVM's own `.stack_sizes` for every function LLVM reports. Indirect calls (trait objects,
`core::fmt`, the Embassy task and waker, C callbacks) are resolved by a reviewed table in the script; a new function
pointer fails the check until it is reviewed. The panic handler may nest once (a panic while one is reported) and the
analysis counts it.

| Date | Worst case | Of 8 KB | Deepest path |
|------|------------|---------|--------------|
| 2026-10-25 | 4,900 B (thread 3,376 B, handlers 1,524 B) | 59.8 % | `Executive::step` → beacon → `fmt` → a panic in the fault path, nested once |
| 2026-10-25, downlink (CR-2026-005) | 5,004 B (thread 3,480 B, handlers 1,524 B) | 61.1 % | The same path; `beacon_transmit` also holds the frame's copy for the TNC (696 B frame) |
| 2026-10-27, CDR flight-software changes | 5,444 B (thread 3,920 B, handlers 1,524 B) | 66.5 % | `Executive::step` → `run_housekeeping` (1,144 B: the HK1 text, its AX.25 frame and the housekeeping record) → a sensor read → a panic in the fault path, nested once |
| 2026-10-27, CDR flight software round 2 | 6,004 B (thread 4,448 B, handlers 1,556 B) | 73.3 % | The same path (`run_housekeeping` 968 B, `housekeeping` 688 B, `sensors_get_by_id` 656 B, then the fault path); the handlers add the NMI (148 B, §10.3) at its own priority level |
| 2026-10-27, CDR flight software round 3 | 6,236 B | 76.1 % | The same path (`housekeeping` 712 B with the HK2 counters); the stack now in SRAM2 |
| 2026-10-27, CDR flight software round 3, third part | 6,332 B (thread 4,776 B, handlers 1,556 B) | 77.3 % | The same path (`run_housekeeping` 1,032 B, `housekeeping` 744 B with the GNSS, FRAM, AntS-gate and passivation-reason fields, `sensors_get_by_id` 656 B, then the fault path) |
| 2026-10-27, after round 3, fifth part (documentation audit, local build) | 5,812 B (thread 4,256 B, handlers 1,556 B) | 70.9 % | `scripts/stack_usage.py` on the current tree; the CI run of the same step is the record |

The first run gave 12,820 B. The code changed to bring it down: core's exact float formatting (a 2 KB frame) replaced
by `sqso_fsw::fixed`; the fault path's console line built without `core::fmt`; large locals moved out of the async main
task and out of `sensors_poll`, `send_telemetry_to_jetson` and the beacon path; and the panic handler made to report
only the first panic.

### 6.3 Stack Protection

At boot (`software/rust/firmware/src/stack.rs`) an MPU region with no access covers the 32 bytes below the budget
(0x2003DFE0). A stack that outgrows 8 KB faults there: MemManage escalates to HardFault, whose handler spins until the
independent watchdog resets the processor. Without it the stack would run on toward the static data.

### 6.4 Stack Monitoring

At boot the unused part of the budget is painted with 0x57ACC0DE. Nothing in the flight software reads it back; a debug
probe does: the used depth is 8192 bytes minus 4 per word still painted from the bottom of the budget up. The hardware
test HIL-044 (`software/flight/tests/hil/HIL_TEST_PLAN.md`) compares it with the static worst case. Not yet measured
(no hardware has run the image).

---

## 7. Critical Data Storage

### 7.1 Triple Modular Redundancy (TMR)

Not implemented. There is no `TmrValue_t`, `tmr_read()` or `tmr_write()`. Critical records are protected instead by a
CRC-32, and the ones that must survive a power failure during a write by two slots (§8).

### 7.2 TMR Data Items

None. Revision A's list (power mode, beacon interval, boot count, mission time, configuration CRC) is design intent only.

---

## 8. Persistent Storage

### 8.1 Non-Volatile Storage Map

On the target the non-volatile memory is the FM25V20A FRAM. The regions (`HalFlashRegion_t`, `hal_flash.h`) are at
the addresses of `hardware/obc/STORAGE_DESIGN.md` §3.3 (`software/rust/firmware/src/fram.rs`):

| Region | FRAM address | Size | Content and protection |
|--------|--------------|------|------------------------|
| EPS_CONFIG | 0x00000 | 256 B | EPS state; 4-byte length; record checked on load |
| STATE | 0x00100 | 512 B | System state; 4-byte length; CRC-32 |
| RF_CONTROL | 0x00300 | 256 B | TX-OFF latch; two-slot record, CRC-32 (`nvm_record.h`) |
| MISSION_DATA | 0x00400 | 512 B | Mission data; 4-byte length; CRC-32 |
| AUTH | 0x0C400 | 768 B | Key table and replay counters; two-slot records, CRC-32 |
| CONTACT | 0x0C700 | 256 B | Loss-of-contact counter; two-slot record |
| RF_SILENCE | 0x0C800 | 128 B | RF-silence state; two-slot record |
| DEPLOYMENT | 0x0C880 | 128 B | Deployment state; two-slot record |
| FAULT_LOG | 0x0C900 | 9,728 B | Fault log, 100 entries of 96 B, each with its CRC-32; 4-byte length |
| HK_DETAIL | 0x10000 | 64 KB | Sixty-second housekeeping records: ring of 409 and three event windows of 360, 44 B each with its CRC-32 (`hk_store.h`; the telemetry buffer of STORAGE_DESIGN §3.3) |
| SENSOR_CONFIG | 0x20000 | 512 B | Reserved |
| BACKUP | 0x20200 | 1 KB | Reserved |
| OPS | 0x20600 | 128 B | Commissioning hold, transponder enable, ADCS geometry; two-slot record |
| HK_SUMMARY | 0x20700 | 90,112 B | Five-minute housekeeping records: ring of 2,048 (7.1 days), 44 B each with its CRC-32 |
| HK_INDEX | 0x36700 | 256 B | The event-window table; two-slot record |
| EOL | 0x36800 | 128 B | The passivation record; two-slot record (`passivation.h`) |
| ADCS_CAL | 0x36880 | 256 B | The gyro alignment matrix (offset 0, magic `GYAL`) and the magnetometer bias per polarity state (offset 128, magic `MGBS`); two-slot records, CRC-32 (SRS-F142, SRS-F144) |

The last six are in the extended storage of STORAGE_DESIGN §3.3 (0x20000 to 0x3FFFF) and its telemetry buffer;
`software/rust/firmware/src/fram.rs` checks at compile time that no two regions overlap and that each fits the part.
STORAGE_DESIGN §3.3 does not list them yet (for its owner).

The blobs (fault log, EPS state, mission data, system state) are written through `software/rust/sqso-ffi/src/store.rs`
with a 4-byte length (`software/rust/README.md`, behaviour change 22). On the host, the regions are RAM in the simulated
HAL, and the blobs are files.

### 8.2 Write Policy

- FRAM needs no erase and has no practical wear limit at these rates. A region "erase" writes 0xFF over it.
- Two-slot records write the slot that does not hold the newest valid copy and read it back
  (`software/rust/sqso-fsw/src/nvm.rs`), so an interrupted write leaves the previous copy intact.
- Mission data and the EPS state are saved every 100 main-loop passes and before a watchdog reset.
- Write failures return an error to the caller.
- Every write goes to both FRAMs, U3 and its mirror U36 (`fram.rs`, `hal_flash_write`: success when one took it).
  Reads come from U3; the records' loader (`nvm::load`) turns to the mirror (`hal_flash_read_mirror`) when U3 fails
  or gives no copy that passes its CRC, and a save picks its slot from the mirror when U3 cannot be read. With both
  unreadable, the one-minute check declares the FRAM failed after three checks and keeps the safe defaults until the
  ground's FRAM_BYPASS puts the critical state in RAM under CRC (`fram_health.rs`, SRS-F160).

---

## 9. Memory Safety

### 9.1 Bounds Checking

In Rust every slice access is bounds-checked. The lint `clippy::indexing_slicing` is denied, so code uses `get()`,
iterators and slice patterns, which cannot panic (`docs/development/CODING_STANDARDS.md` §11.2, §11.4). The C helpers
of revision A (`SAFE_ARRAY_ACCESS`, `safe_strncpy`) are not needed in Rust. `safe_string.h` remains for C callers, and
it is implemented in Rust (`software/rust/sqso-ffi/src/safe_string.rs`).

### 9.2 Null Pointer Checks

Pointers exist only at the C ABI. `software/rust/sqso-ffi/src/ptr.rs` turns each C pointer into a reference or slice
after a NULL check, and a NULL argument returns `SMART_QSO_ERROR_NULL_PTR`.

### 9.3 Integer Overflow Protection

The release profile keeps `overflow-checks = true`, so an arithmetic overflow is a panic, which is a fatal assertion.
Counters and tick arithmetic use 64-bit, `checked_*` or `saturating_*` operations where a value could wrap.

---

## 10. Memory Scrubbing and Upsets (RID AV-07)

### 10.1 SRAM Scrubbing

Not implemented: there is no scrub task. SRAM2 parity (§10.3) detects an upset when the byte is read; the persisted
state is checked against FRAM every minute (§10.4).

### 10.2 Scrub Implementation

None. A scrub that only reads SRAM2 would turn a latent parity error into an earlier reset; it is not needed while every
static is read at least once a minute by the main loop, which is not proven for every byte (§10.5).

### 10.3 SRAM2 Parity and the NMI

`software/rust/firmware/src/parity.rs` (SRS-F132):

- **Placement.** Every static and the stack (§3.1) are in SRAM2, the STM32L4R5's only RAM with parity (one bit per byte, checked on
  every read).
- **Erase.** `__pre_init`, before cortex-m-rt initializes RAM, erases SRAM2 by hardware (SYSCFG_SCSR.SRAM2ER after the
  SYSCFG_SKR key), so no byte is read with the random parity of power-on. It uses registers only (no stack, no static).
- **Enable.** Parity is on when the option bit `SRAM2_PE` is 0. **The flight software never writes option bytes**
  (an interrupted option-byte write can leave the part unbootable): `SRAM2_PE` = 0 is set when the board is programmed,
  with the other option bytes, by the board programming procedure (`docs/development/RELEASE_PROCEDURE.md` §5.3.1,
  STM32CubeProgrammer `-ob SRAM2_PE=0`); the boot console and HK2 `PE` report the state (SRS-F154). Every boot that finds it off logs "SRAM2 parity off (option bit SRAM2_PE)".
- **NMI.** An SRAM2 parity error (SYSCFG_CFGR2.SPF), a flash double-bit ECC error (FLASH_ECCR.ECCD, with its address)
  or a clock-security failure raises the NMI. The handler records the cause in SRAM1 (tag, cause, complement, address:
  outside the linked RAM, so neither erased nor zeroed at boot) and resets at once (`SCB::sys_reset`), without the 30 s
  wait for the watchdog. Every output to the EPS and the radio is low through the reset (the EPS boards pull them down).
- **Next boot.** Once the fault log is loaded, the record is read and cleared, and "Reset by NMI: SRAM2 parity error"
  (or flash ECC, clock security) is logged at CRITICAL; the latched error flags are cleared. The state is then what
  FRAM holds, each record under its CRC (§8).

### 10.4 Periodic Check of the Persisted State

Every 60 s the executive (`run_safety_checks`) re-reads the records whose RAM copy decides a safety action and settles
a disagreement toward the safe side (SRS-F133):

| Record | RAM copy disagrees with the record | Action |
|--------|-------------------------------------|--------|
| RF_CONTROL (TX-OFF latch) | The record says TX-OFF (or is unreadable) and RAM says permitted | TX-OFF applied, logged |
| | RAM says TX-OFF and the record does not | The record re-written to TX-OFF, logged |
| OPS | Hold, transponder enable or geometry differ | Merged to held and transponder stopped, the stored geometry taken, logged |
| EOL (passivation) | The record (or the EPS EOL_STATE) says passivated and RAM does not | Passivated: the inhibit set for good |

### 10.5 Not Mitigated

- **The stack (closed in Revision F, RID AV-13).** Until Revision E the 8 KB stack was in SRAM3, without parity. It is
  now at the top of SRAM2 (initial SP 0x20040000, pinned by `.github/workflows/flight-software-ci.yml`): every live
  byte of RAM has parity once `SRAM2_PE` is set (`docs/development/RELEASE_PROCEDURE.md` §5.3.1).
- **Detection only.** Parity detects a single flipped bit per byte, not two; it corrects nothing. The response is a
  reset, which loses the RAM state that is not in FRAM (the beacon schedule, the downlink queue, the counters since
  boot). A byte is checked only when it is read: an upset in a static read rarely is found late.
- **Between two checks.** A RAM copy of a persisted record that is wrong but has valid parity (a software fault, or an
  upset during a copy) lasts up to 60 s before §10.4 finds it.
- **Registers.** The core's registers, the peripherals' registers (GPIO, timers, DMA) and the FPU state have no
  protection; the watchdog and the safe default levels at reset are the mitigation.
- **Flash.** A double-bit ECC error in the code that the reset path itself executes resets again: a boot loop that only
  a reload of the image on the ground can break. Single-bit errors are corrected by hardware and
  not counted.
- **Rate.** The radiation analysis (RADIATION_ANALYSIS §5.2) gives 0.006 to 0.06 upsets a day in live state: with
  parity each one read becomes a logged reset: one every 17 to 170 days.

## 11. Verification

### 11.1 Static Analysis

- clippy with `-D warnings` (CI): denies indexing that can panic, `unwrap` and `panic!`.
- The `no_dynamic_memory` ctest and the image check (§4.3).
- cppcheck and clang-tidy on the remaining C.

### 11.2 Runtime Checks

- Bounds and overflow checks in every Rust build, flight included.
- The C test suites run against the Rust under AddressSanitizer and UBSan (CI job Sanitizer Checks).

### 11.3 Testing

- 12 fuzz targets: every parser, and the fixed-point formatter (30 s each in CI).
- Worst-case stack by static analysis of the image, in CI (§6.2): 5,812 B of 8 KB (2026-10-27).
- SRAM2 parity and the NMI: fault injection on the EM (RAD-V6 in RADIATION_ANALYSIS: an SRAM2 bit flipped through the debugger; flash ECC test mode), not done.
- Stack overflow (HIL-004), high-water mark (HIL-044) and memory stress tests on hardware: not done (no hardware has run the image).

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-24 | The flight software is Rust. Policy as implemented: no allocation at any time; enforcement by the `no_dynamic_memory` ctest, the image check and the compiler. Memory map and measured image budget; static buffers; FRAM regions. TMR, buffer pools, stack canary and guard, and scrubbing marked not implemented (design intent only) | SMART-QSO Team |
| C | 2026-10-25 | §6 for SRS-C003: the worst case by static analysis (4,892 B of 8 KB, a CI gate), the MPU stack guard and the painted budget, now implemented | SMART-QSO Team |
| D | 2026-10-27 | Worst-case stack 5,444 B of 8 KB (66.5 %) after the CDR flight-software changes; the housekeeping stage is the deepest frame | SMART-QSO Team (CDR avionics review) |
| E | 2026-10-27 | §3.1 the firmware's own `memory.x`: statics in SRAM2 (parity), the stack in SRAM3, SRAM1 unlinked; §3.2 budget of the CDR round-2 image; §6.2 worst case 6,004 B; §8.1 regions HK_DETAIL, OPS, HK_SUMMARY, HK_INDEX and EOL; §10 SRAM2 parity and the NMI handler, the periodic check of the persisted state and what is not mitigated (RID AV-07) | SMART-QSO Team (CDR flight software, round 2) |
| F | 2026-10-27 | §3.1 the stack moved into SRAM2 (initial SP 0x20040000; RID AV-13), SRAM3 unused; §3.2 budget of the round-3 image; §6 worst case 6,236 B, the budget and guard addresses; §8.1 region ADCS_CAL; §10.3 the programming procedure sets `SRAM2_PE`; §10.5 the stack no longer unprotected | SMART-QSO Team (CDR flight software, round 3) |
| G | 2026-10-27 | §3.1 the second FRAM U36 (RID SMA-B10); §8.2 every record mirrored, read from the mirror when the first FRAM fails, the degraded mode without either (SRS-F160); §3.2 and §6 the image and worst-case stack of round 3, second part | SMART-QSO Team (CDR flight software, round 3, second part) |
| H | 2026-10-27 | §3.2 and §6 the image and worst-case stack of round 3, third part (text 207,284 B, data 26,400 B, bss 17,052 B; 6,332 B of 8 KB), the statics' end 0x2003A9BC (Revision G named these without changing the figures) | SMART-QSO Team (CDR flight software, round 3, third part) |
| I | 2026-10-27 | Documentation audit (post-CDR 2026): §5.1 TNC transmit buffer 1,024 B (was 16 B) and the GNSS UART buffers, as in `firmware/src/main.rs`; §3.2, §6 and §11.3 the image after round 3, fifth part (text 211,604 B, data 27,192 B, bss 17,052 B; stack 5,812 B; `__ebss` 0x2003ACD4), from a local build | SMART-QSO Team |
