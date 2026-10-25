# Mutation testing: the SRS-C003 stack changes

Scope: the functions added or changed to bring the worst-case stack under 8 KB (2026-10-25): `sqso_fsw::fixed`,
`beacon::serialized_len` and `build_frame`, `checksum::crc16_x25_parts`, `ax25::fcs_parts`, `c::Line`,
`console_parts`, `os::print_parts`, `sensors::poll_job`, `print_reading` and `reading_line`, `beacon_transmit`,
`fault_mgmt::add`, and `Executive::start`, `step`, `send_telemetry_to_jetson` and `request_beacon_text`.

## Commands

```bash
# Rust unit tests (from software/rust, clean tree)
cargo mutants -p sqso-fsw -p sqso-protocol -p sqso-ffi --in-place --profile release --timeout 120 \
  -F 'fixed::|serialized_len|build_frame|crc16_x25_parts|fcs_parts|Line::|poll_job|print_reading|reading_line|console_parts|print_parts|beacon_transmit|fault_mgmt::add|Executive::(start|step|request_beacon_text|send_telemetry_to_jetson)' \
  -o OUT
# The missed ones in those functions against the C suites (from the repository root)
python3 software/rust/mutants/ctest_missed.py OUT/mutants.out BUILD
```

The `-F` pattern also matches some struct-field mutants of other functions; those are outside this scope and are
covered by `RESULTS-sqso-fsw.md` and the C suites.

## Numbers

| Run | Mutants | Caught | Missed | Unviable |
|-----|---------|--------|--------|----------|
| Rust unit tests | 139 | 59 | 76 | 4 |
| Missed ones in the scope, against the C suites | 54 | 15 | 36 | 3 |
| After the new tests (`ax25::fcs_parts`; `c::Line`, `reading_line` in sqso-ffi), Rust unit tests, `Line::` and `reading_line` | 36 | 20 | 14 (1 in scope) | 2 |

`sqso_fsw::fixed`, run by file (`-f sqso-fsw/src/fixed.rs`; the pattern above matches function names, not modules):
91 mutants, 87 caught, 3 missed, 1 unviable. The three are equivalent:

- `decompose`, `-1074` with its sign deleted: a subnormal is then taken as an integer shifted past the last limb and
  prints as 0. Every subnormal is below 10^-307 and prints as 0 at nine decimals or fewer, so the text is the same.
- `decompose`, `frac | (1 << 52)` with `^`: `frac` has no bit 52, so both are the same.
- `div_chunk`, `(rem << 32) | limb` with `^`: the operands have no bit in common.

## Survivors

- **`ax25::fcs_parts` replaced by 0 or 1**: caught only by sqso-fsw's beacon tests, which cargo-mutants does not run
  for a sqso-protocol mutant. Fixed: `fcs_over_parts_is_the_fcs_of_the_whole` in `sqso-protocol/src/ax25.rs`.
- **`c::Line::text` and `dec`, `sensors::reading_line`**: nothing checked the text of the fault console line or of the
  `[READ]` line. Fixed: unit tests in `sqso-ffi/src/c.rs` and `sqso-ffi/src/sensors.rs` (sqso-ffi now runs unit tests).
- **`c::Line::print`, `console_parts`, `os::print_parts` replaced by ()**: console output only. No test reads the
  console. Accepted: a console line has no flight function (the debug UART).
- **`Executive::step`, `send_telemetry_to_jetson`, `request_beacon_text`, `start` replaced by true** (24 mutants):
  the main loop runs only in the ctest `Simulation_Run`, which checks for a clean shutdown, and in the simulation the
  Jetson UART does not open (no serial device), so the telemetry path never runs. These are a test gap of the
  executive (line coverage of `sqso-ffi/src/executive.rs` is 59 %), not of the stack changes, which moved code without
  changing its logic. Closing it needs a simulation run with a pseudo-terminal as the Jetson port and assertions on
  the lines it sends (as `test_uart_comm` does for the UART driver). Done: see the next section.

## The executive with a simulated Jetson (2026-10-25)

The test gap in the last survivor above is closed by `software/flight/tests/integration/test_executive_jetson.c`
(ctest `Executive_Jetson_link` and `Executive_Jetson_guard`): the host simulation runs with a pseudo-terminal as the
Jetson port, the driver plays the Jetson and checks the lines sent, their timing and the console (see
`software/flight/tests/README.md`). Rerun on a clean worktree of that commit:

```bash
cargo mutants -p sqso-ffi --in-place --profile release --timeout 120 \
  -F 'Executive::(start|step|send_telemetry_to_jetson|request_beacon_text)' -o OUT   # in software/rust
python3 software/rust/mutants/ctest_missed.py OUT/mutants.out BUILD                    # from the repository root
```

40 mutants, all missed by the Rust unit tests (the executive has none). Against the C suites:

| Mutants | Caught | Missed |
|---------|--------|--------|
| In `Executive::start`, `step`, `send_telemetry_to_jetson`, `request_beacon_text` | 25 of 27 | 2 (equivalent) |
| Matched by `-F` elsewhere (struct-field deletions) | 6 of 13 | 7 |

Line coverage of `sqso-ffi/src/executive.rs` (`software/flight/scripts/rust_coverage.sh`): 59.3 % before, 87.8 % with
the two entries; all Rust 92.3 % to 93.4 %.

Survivors:

- **`step`, `(now % 10_000) < 100` with `<=`**: the health check also runs at 100 ms past the window start, a
  millisecond no 20 ms pass is guaranteed to hit, and the first pass in the window does the check either way.
  Equivalent.
- **`send_telemetry_to_jetson`, `!payload || !uart` with `&&`**: its only caller, `step`, calls it only when both are
  true, so the early return never differs. Equivalent while the guard in `step` stays.
- **`update_beacon_measurements`, `soc`, `sunlit` or `valid` deleted**: these feed the beacon, and no beacon is sent in
  a simulation run (each boot starts the 45-minute RF silence, the simulated flash being RAM, and the beacon interval
  is 60 s). Killing them needs a run whose RF silence is complete, for example a host-only hook in `hal_sim.rs` that
  preloads a completed RF-silence record, and a beacon check on the `[BEACON] TX:` line.
- **`sys_state_init` (`initialized`, `watchdog_ok`, `last_update_ms`) and `check_task_watchdogs` (`name`)**: outside
  the executive, matched by the pattern; not in this scope.
