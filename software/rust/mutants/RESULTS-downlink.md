# Mutation testing: the downlink transmitter (CR-2026-005)

Scope: the code added for the downlink on 2026-10-25: `sqso_fsw::downlink`, `sqso-ffi/src/downlink.rs`, the KISS
parameter commands in `sqso_protocol::kiss`, and the functions the downlink changed or added elsewhere
(`ax25::append_fcs`, `beacon::serialize_body`, `beacon_transmit`, the host HAL's `hal_radio_transmit` and
`kiss_stream`, the simulation's `serve_rf_silence`).

## Commands

```bash
# in software/rust, on a clean worktree of 5306165 (the downlink commit)
cargo mutants -p sqso-fsw -p sqso-ffi -p sqso-protocol --in-place --profile release --timeout 120 \
  -f sqso-fsw/src/downlink.rs -f sqso-ffi/src/downlink.rs -f sqso-protocol/src/kiss.rs -o OUT1
cargo mutants -p sqso-fsw -p sqso-ffi -p sqso-protocol --in-place --profile release --timeout 120 \
  -F 'append_fcs|serialize_body|beacon_transmit|hal_radio_transmit|kiss_stream|serve_rf_silence' -o OUT2
# the missed ones against the C suites, from the repository root of a worktree of ba361ca
python3 software/rust/mutants/ctest_missed.py OUT1/mutants.out BUILD
python3 software/rust/mutants/ctest_missed.py OUT2/mutants.out BUILD
```

`-f` and `-F` together select the mutants that match both, so the files and the functions are two runs. The first
C-suite run was on a worktree of 5306165, whose Release build did not compile (`test_downlink_sim.c`,
`-Wmaybe-uninitialized`, fixed in b0caed7): every mutant read "unviable". The numbers below are from the rerun on
ba361ca.

## Numbers

| Run | Mutants | Caught by the Rust unit tests | Timeouts (caught) | Unviable | Missed | Of those, caught by the C suites |
|-----|---------|-------------------------------|-------------------|----------|--------|----------------------------------|
| The three files | 148 | 127 | 7 | 10 | 4 | 1 |
| The functions | 79 | 55 | 0 | 0 | 24 | 15 |

## Survivors

The three files:

- **`downlink_poll` replaced by ()**: the Rust unit tests do not call it; `test_downlink` (VSWR, no output, thermal)
  catches it.
- **`Downlink::decide`, `cost > self.credit` with `>=`**: nothing sent a frame costing exactly the remaining credit.
  Fixed: `a_frame_may_spend_the_last_of_the_credit` (ba361ca); applied by hand, the mutant fails it.
- **`Downlink::thermal`, `temp_c < TEMP_RESUME_C` with `<=`**: nothing read exactly 60 °C. Fixed:
  `at_the_resume_point_exactly_it_stays_held` (ba361ca); applied by hand, the mutant fails it.
- **`kiss::encode_command`, `(port << 4) | command` with `^`**: equivalent. The port is in the high nibble and the
  command in the low one, so the two have no bit in common.

The functions:

- **`kiss_stream` (7 mutants), `serve_rf_silence`, the six `JetsonMsg` fields of `jetson_link::convert`, and the
  beacon measurements' `valid` field**: caught by the C suites (`Downlink_TNC_served` for the first two and the
  last; the Jetson link suites for `convert`).
- **`update_beacon_measurements`, `soc` or `sunlit` deleted**: the end-to-end check required a digit after `SOC=` and
  `SUN=`, and a deleted field reads 0. Fixed: `Downlink_TNC_served` now requires the first beacon to show the power
  model's start, about 75 % and sunlit; applied by hand, each mutant fails it. These, with `valid`, were the
  survivors no simulation run could reach before the downlink (`RESULTS-stack-changes.md`): every boot started the
  45-minute RF silence.
- **`sys_state_init` (`initialized`, `watchdog_ok`, `last_update_ms`), `SystemState::defaults`
  (`persistence_dirty`, `external_temp_c`), `SystemState::compute_crc` (`crc32`) and `check_task_watchdogs`
  (`name`)**: struct-field deletions in other modules that the `-F` pattern matched; outside this scope, as in
  `RESULTS-stack-changes.md`.

After the fixes, no mutant in the scope survives except the equivalent one.
