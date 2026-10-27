# Loop Bounds Documentation

## SMART-QSO Flight Software

**Document Number:** SQSO-SW-007
**Revision:** C
**Date:** 2026-10-27
**Reference:** JPL Power of Ten Rule 2, NPR 7150.2

---

## 1. Overview

This document records the loop bounds of the SMART-QSO flight software per JPL Power of Ten Rule 2:

> "All loops must have a fixed upper-bound. It must be trivially possible for a checking tool to prove statically that a preset upper-bound on the number of iterations of a loop cannot be exceeded."

Since 2026-10-24 the flight software is Rust (`software/rust/README.md`). Revision A listed loops in C files that have
since been replaced. Many of the functions it named never existed in the code. This revision inventories the loops of
the Rust as it is. The one remaining C file, `software/flight/src/flight_log_fmt.c`, has no loops.

**Revision C (2026-10-27).** The inventory of §3 and the matrix of §8 are those of 2026-10-24. The code added since
(the CDR 2026 flight-software rounds: stored housekeeping, GNSS, transponder, acknowledgements, passivation, the second
FRAM, the stack paint) raises the non-test counts to about 139 `for`, 26 `while`, 11 `while let` and 9 `loop`
(2026-10-27, by pattern search outside `#[cfg(test)]` modules). The new non-`for` loops and their bounds:

| Where | Bound |
|-------|-------|
| `sqso-fsw/src/hk_store.rs` (`dump_frame`) | `DUMP_PER_FRAME` records and the records left in the dump |
| `sqso-fsw/src/transponder.rs` (`record`) | Elapsed time since the last pass divided by `BUCKET_MS`: bounded by the clock, not by a constant (a long gap between passes runs one iteration per bucket); for review |
| `sqso-fsw/src/ack.rs` (`front`, `while let`) | The queue length; each iteration pops one entry |
| `firmware/src/fram.rs` (two `const` checks) | The region table length (compile time) |
| `firmware/src/stack.rs` (`paint`) | The 8 KB stack budget below the current SP |

The per-crate counts of §8 are to be recounted at the next review of this document.

**How rule 2 is checked:** by review. No tool proves loop bounds for the Rust. Fuzzing (`software/rust/fuzz`, 30 s per
target in CI) runs the parser loops on arbitrary input, and the watchdog (§4.2) bounds the main loop in time.

---

## 2. Where the Bounds Come From

The Rust has no separate loop-bound constants. A loop's bound is one of the following:

| Bound | Example |
|-------|---------|
| The length of a fixed-size array (a constant in the module, mirrored from the C header) | Sensor table of `MAX_SENSORS` = 32 (`SMART_QSO_MAX_SENSORS`), fault log `MAX_ENTRIES` = 100, watchdog `MAX_TASKS` = 8, flight log `BUFFER_SIZE` |
| The length of an input slice, itself bounded by its buffer | A received frame (at most the receive buffer), a Jetson line (`LINE_MAX`), a beacon text (`TEXT_MAX` = 176) |
| A constant range | Command dictionary entries, the states and events of the mode manager (`State::ALL`, `Event::ALL`) |
| A deadline on the mission clock | `hal_uart_read` and `hal_uart_read_line` (`software/rust/firmware/src/uart.rs`): the caller's `timeout_ms` |
| A count limited by a buffer | A UART drain stops when the hold buffer is full (`RX_HOLD`) |

Lengths and indices are `usize`. Iterators and `get()` cannot run past a slice: the lint `clippy::indexing_slicing` is
denied (`docs/development/CODING_STANDARDS.md` §11.2).

---

## 3. Loop Inventory

Counted in the non-test code of `sqso-protocol`, `sqso-fsw`, `sqso-ffi`, `sqso-sim`, `sqso-clib` and the firmware
(2026-10-24): 72 `for`, 19 `while`, 10 `while let` and 9 `loop`.

### 3.1 `for` loops (72)

Every `for` loop runs over a fixed-size array, a slice or a constant range (`for x in table.iter()`,
`for (dst, src) in a.iter_mut().zip(b)`, `for i in 0..N`). Its bound is that length (§2). None runs over an open-ended
iterator.

### 3.2 `while` loops (19)

| Kind | Where | Bound |
|------|-------|-------|
| Index loops in `const` items and `const fn`s (`get()` is not available there) | `sqso-ffi`: `beacon.rs`, `beacon_filter.rs`, `cmd_dictionary.rs`, `contract.rs`, `hal_sim.rs`, `state_machine.rs` (4), `uart_comm.rs`; `sqso-fsw/src/mode.rs` (`Event::from_raw`); `firmware/src/fram.rs` (2, region overlap check) | The array length in the loop condition. Most are evaluated at compile time |
| FRAM erase | `firmware/src/fram.rs` (`hal_flash_erase`) | The region size, 64 bytes per write |
| C string scan | `sqso-ffi/src/safe_string.rs` | The documented maximum `max` |
| UART drain | `firmware/src/uart.rs` | `RX_HOLD` |
| Radio drain | `firmware/src/radio.rs` (`hal_radio_receive`) | The bytes in the 1,024 B receive buffer. It ends early at a complete KISS frame. Bytes arriving at 9600 baud during the drain extend it, but they arrive far more slowly than the loop reads them |
| Scheduler run | `sqso-ffi/src/scheduler.rs` (`scheduler_start`) | Until a task calls `scheduler_stop()`. Not used by the executive; the C test suites exercise it |
| Host simulation | `sqso-ffi/src/executive.rs` (`run_host`) | 1000 passes, or until SIGINT/SIGTERM with feature `flight` (§4.1) |

### 3.3 `while let` loops (10)

Nine consume a slice pattern (`while let [c @ b'0'..=b'9', tail @ ..] = rest`) or search a slice from a position that
only grows (`sqso-protocol/src/filter.rs`). Each iteration removes at least one byte, so the bound is the slice length.
They are in `sqso-fsw/src/gnss.rs` (2), `sqso-fsw/src/sensors.rs` (5) and `sqso-fsw/src/uart_comm.rs`. The tenth,
`sensors_poll` in `sqso-ffi/src/sensors.rs`, walks the sensor table by index up to the sensor count (at most 32).

### 3.4 `loop` (9)

| Where | Ends when | Bound |
|-------|-----------|-------|
| `firmware/src/main.rs`, main task | Never (the flight main loop) | The watchdog (§4.2) |
| `firmware/src/main.rs`, panic handler; `sqso-clib/src/lib.rs`, panic handler; `sqso-ffi/src/watchdog_mgr.rs` (`wdt_mgr_force_reset`) | Never: they wait for the IWDG to reset the OBC | The IWDG timeout |
| `firmware/src/uart.rs` (`hal_uart_read`, `hal_uart_read_line`) | Data read, or the deadline | `timeout_ms`; the flight software calls with 0 |
| `sqso-ffi/src/flight_log.rs` (output of every entry) | The last entry | The log's `BUFFER_SIZE` entries. The count is read again each time because a callback may log |
| `sqso-ffi/src/sensors.rs` (configuration file read, host only) | A short `fread` | The file size (there is no file system on the target) |
| `sqso-fsw/src/safe_string.rs` (`format_u32`) | The value is 0, or 10 digits | 10 |

---

## 4. Flight vs. Simulation Loops

### 4.1 Simulation

The host simulation `smart_qso_flight` (`software/rust/sqso-sim`) runs `SIMULATION_TICKS` = 1000 passes, 20 ms apart
(about 20 s), and stops early on SIGINT or SIGTERM with state saved. With the cargo feature `flight` (CMake
`FLIGHT_BUILD`) it runs until stopped, as in flight. CI runs the simulation to completion (job Sanitizer Checks).

### 4.2 Flight

The firmware's main task loops on `Executive::step()` and `Timer::after_millis(20).await` forever
(`software/flight/docs/RTOS_ARCHITECTURE.md` §4). The loop is bounded in time, not in count:

1. **Stage supervision**: each stage of a pass checks in with the watchdog manager. A stage that misses its period 3
   times in a row stops the hardware refresh (SW-15).
2. **Hardware watchdog**: the IWDG resets the OBC 30 s after the last refresh, so a pass that never returns ends in a
   reset.
3. **Ground command**: safe-mode transition capability.

---

## 5. Loop Monitoring

### 5.1 Runtime Checks

No loop has a run-time iteration counter. The supervision of the main loop is the watchdog (§4.2). Integer overflow
checks are on in the release profile, so an index or count that would wrap stops the software instead
(`docs/development/CODING_STANDARDS.md` §11.5).

### 5.2 Static Analysis and Testing

1. **clippy** (CI, `-D warnings`): denies indexing that can panic. It does not prove loop bounds.
2. **Fuzzing**: every parser loop runs on arbitrary input (12 targets, `software/rust/fuzz/fuzz_targets`).
3. **Review**: the inventory in §3.

---

## 6. Prohibited Patterns

The following loop patterns are **PROHIBITED** in flight software:

### 6.1 Unbounded Loops

```rust
// PROHIBITED: no guaranteed termination
while condition() {
    process();
}
```

`loop` is used only for the main loop and for the waits for a watchdog reset (§3.4).

### 6.2 Data-Dependent Bounds

```rust
// PROHIBITED: an external count used as the bound
for i in 0..header.count {
    process(i);
}
```

### 6.3 Recursion

Direct or indirect recursion is prohibited (JPL rule 1, `software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md` §2.1).

---

## 7. Approved Patterns

### 7.1 Iteration over a fixed-size table

```rust
for entry in log.entries.iter().take(log.count.min(MAX_ENTRIES)) {
    process(entry);
}
```

### 7.2 Consuming a slice pattern

```rust
while let [c @ b'0'..=b'9', tail @ ..] = rest {
    value = value.checked_mul(10)?.checked_add(u32::from(c - b'0'))?;
    rest = tail;
}
```

### 7.3 Deadline-bounded wait

```rust
let deadline = Instant::now() + Duration::from_millis(u64::from(timeout_ms));
loop {
    if let Some(n) = try_read(buf) { return Ok(n); }
    if Instant::now() >= deadline { return Err(Timeout); }
}
```

---

## 8. Verification Matrix

| Crate | `for` | `while` | `while let` | `loop` | Bounded | Check |
|-------|-------|---------|-------------|--------|---------|-------|
| `sqso-protocol` | 6 | 0 | 1 | 0 | Yes (§3) | Review; fuzzing |
| `sqso-fsw` | 39 | 1 | 8 | 1 | Yes (§3) | Review; fuzzing; unit tests |
| `sqso-ffi` | 26 | 13 | 1 | 3 | Yes (§3); `wdt_mgr_force_reset` waits for the reset | Review; C suites |
| `firmware` | 1 | 5 | 0 | 4 | Main loop and panic wait: watchdog; UART reads: deadline | Review; not run on hardware |
| `sqso-clib` | 0 | 0 | 0 | 1 | Panic wait: watchdog | Review |
| `flight_log_fmt.c` (C) | 0 | 0 | 0 | 0 | N/A | Review |

---

## 9. Change Log

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| A | 2026-01-02 | SMART-QSO Team | Initial release |
| B | 2026-10-24 | SMART-QSO Team | The flight software is Rust: inventory of the Rust loops by form and bound (§2, §3, §8); flight and simulation loops of the Rust executive and firmware (§4); Rust patterns (§6, §7). Revision A's C inventory removed (its files are gone, and several functions it named did not exist). Rule 2 is checked by review |
| C | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): §1 status of the inventory, current counts and the loops added since Revision B |

---

## 10. References

1. JPL Institutional Coding Standard for C (D-60411)
2. NASA NPR 7150.2 - Software Engineering Requirements
3. MISRA C:2012 - Rule 15.4 (Loop termination)
4. Gerard J. Holzmann, "The Power of Ten: Rules for Developing Safety-Critical Code"
