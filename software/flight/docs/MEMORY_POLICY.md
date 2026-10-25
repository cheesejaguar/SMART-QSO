# Memory Allocation Policy
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-006
**Revision:** C
**Date:** 2026-10-25
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

The linker uses the `memory.x` that embassy-stm32 generates for the STM32L4R5ZI (feature `memory-x`), with the
cortex-m-rt `link.x`:

| Region | Start | Size | Usage |
|--------|-------|------|-------|
| Flash | 0x08000000 | 2 MB (dual bank) | Vector table, code, constants, `.data` initial values |
| SRAM (SRAM1 to SRAM3, contiguous) | 0x20000000 | 640 KB | `.data`, `.bss`, then the stack, which grows down from 0x200A0000 |
| FRAM (FM25V20A, SPI1) | external | 256 KB | Non-volatile records (§8) |

Revision A split SRAM into regions for "critical data (TMR)" and "buffers, logs", and listed a backup SRAM. The linker
script makes no such split, and nothing uses backup SRAM.

### 3.2 Memory Budget

Measured from the release image (2026-10-25, `arm-none-eabi-size`; CI job STM32L4 Cross-Compile (library) prints the same):

| Category | Used | Available | Margin |
|----------|------|-----------|--------|
| Code and constants (text) | 110,388 B | 2 MB flash | 95 % |
| Initialized data (data) | 19,648 B | (in SRAM; initial values in flash) | — |
| Zeroed data (bss) | 15,488 B | (in SRAM) | — |
| Static RAM (data + bss) | 35,136 B | 640 KB | 95 % |
| Stack | Worst case 4,900 B (static analysis, §6.2) | 8 KB budget below 0x200A0000 (SRS-C003) | 40 % |
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
| TNC UART receive / transmit | 1,024 B / 16 B | same |
| Debug UART receive / transmit | 64 B / 512 B | same |
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

There is one stack (cortex-m-rt), from the initial SP 0x200A0000 downward. The executor runs one task, and the
interrupt handlers use the same stack. The Embassy task's state (the async main function's locals held across an
`await`, including the executive's state) is static, in `.bss`, not on the stack. The budget is the 8 KB below the
initial SP, 0x2009E000 to 0x2009FFFF; the static data end at `_stack_end` (0x20008940 on 2026-10-25), about 600 KB
lower.

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

The first run gave 12,820 B. The code changed to bring it down: core's exact float formatting (a 2 KB frame) replaced
by `sqso_fsw::fixed`; the fault path's console line built without `core::fmt`; large locals moved out of the async main
task and out of `sensors_poll`, `send_telemetry_to_jetson` and the beacon path; and the panic handler made to report
only the first panic.

### 6.3 Stack Protection

At boot (`software/rust/firmware/src/stack.rs`) an MPU region with no access covers the 32 bytes below the budget
(0x2009DFE0). A stack that outgrows 8 KB faults there: MemManage escalates to HardFault, whose handler spins until the
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
| SENSOR_CONFIG | 0x20000 | 512 B | Reserved |
| BACKUP | 0x20200 | 1 KB | Reserved |

The blobs (fault log, EPS state, mission data, system state) are written through `software/rust/sqso-ffi/src/store.rs`
with a 4-byte length (`software/rust/README.md`, behaviour change 22). On the host, the regions are RAM in the simulated
HAL, and the blobs are files.

### 8.2 Write Policy

- FRAM needs no erase and has no practical wear limit at these rates. A region "erase" writes 0xFF over it.
- Two-slot records write the slot that does not hold the newest valid copy and read it back
  (`software/rust/sqso-fsw/src/nvm.rs`), so an interrupted write leaves the previous copy intact.
- Mission data and the EPS state are saved every 100 main-loop passes and before a watchdog reset.
- Write failures return an error to the caller.

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

## 10. Memory Scrubbing

### 10.1 SRAM Scrubbing

Not implemented. There is no scrub task.

### 10.2 Scrub Implementation

None. Whether scrubbing is needed is a radiation-mitigation decision (`software/flight/docs/RADIATION_MITIGATION.md`).
Revision A's example scrubbed with a raw pointer loop, which the Rust rules would allow only in the firmware, as
`unsafe` register-style access.

---

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
- Worst-case stack by static analysis of the image, in CI (§6.2): 4,900 B of 8 KB.
- Stack overflow (HIL-004), high-water mark (HIL-044) and memory stress tests on hardware: not done (no hardware has run the image).

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-24 | The flight software is Rust. Policy as implemented: no allocation at any time; enforcement by the `no_dynamic_memory` ctest, the image check and the compiler. Memory map and measured image budget; static buffers; FRAM regions. TMR, buffer pools, stack canary and guard, and scrubbing marked not implemented (design intent only) | SMART-QSO Team |
| C | 2026-10-25 | §6 for SRS-C003: the worst case by static analysis (4,892 B of 8 KB, a CI gate), the MPU stack guard and the painted budget, now implemented | SMART-QSO Team |
