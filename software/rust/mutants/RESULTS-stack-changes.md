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

`sqso_fsw::fixed`: every mutant caught (its tests compare with core's formatting for every tie, power of two and
400,000 random values; the fuzz target `fixed_format` runs in CI).

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
  the lines it sends (as `test_uart_comm` does for the UART driver).
