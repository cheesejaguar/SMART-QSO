# Mutation testing: sqso-protocol

Tool: cargo-mutants 27.1.0, toolchain 1.97.0 (pinned). Base commit 55c6f11.

## Commands

```bash
cd software/rust
cargo mutants -p sqso-protocol --in-place -o <OUT>   # before (unchanged tree) and after (with the new tests)
```

`--in-place` because the crate's tests read `software/flight/tests/vectors` and
`software/flight/command_dictionary.json`, which a copied tree (software/rust only) does not have.
No `.cargo/mutants.toml` was added: nothing is excluded.

## Numbers

| Run | Mutants | Caught | Missed | Unviable | Timeout |
|-----|---------|--------|--------|----------|---------|
| Before (55c6f11) | 498 | 359 | 83 | 53 | 3 |
| After (this change) | 498 | 433 | 9 | 53 | 3 |

Before, 12 of the 83 were caught by the C suites (`mutants/ctest_missed.py`, run on the before set when this work
was assigned; not re-run here) and 71 survived both. The missed set of my before run is identical to that one. After,
the 9 that cargo-mutants still reports are all argued equivalent below; in that C-suite run all 9 also survived the
C suites.

The 3 timeouts are the same in both runs: `filter.rs:281:25` (`+` to `-` and to `*` in `pos + offset`) and
`filter.rs:290:21` (`+` to `*` in `start + len`) in `check`. They stop the word loop from advancing, so the
existing filter tests hang instead of passing; a timeout counts as detected.

Unit tests in sqso-protocol: 37 before, 70 after (33 added, none changed or removed). No production code changed.

## Tests added

| Module | What the new tests check (source of the expected values) |
|--------|-----------------------------------------------------------|
| `auth` | `Key::from_slice` takes exactly 32 bytes; `build_frame` returns the length of a frame that `frame::parse`, `split` and `verify` accept, and `Space` for a buffer one byte short |
| `ax25` | 18-byte frame with an empty information field; digipeater path with an empty information field; an address field that runs into the FCS is `Format`; SSID 15 parses; `encode_ui` accepts exactly `MAX_FRAME_LEN` bytes, with and without the FCS |
| `command` | `accepts_len` is inclusive at both ends; every command's `name`, `c_name` and `summary` equal `command_dictionary.json` |
| `filter` | `Verdict::from_raw` for every C value and out-of-range values; texts of exactly `MAX_LEN` and words of exactly `MAX_WORD` pass, one more fails; two lower-to-upper case changes pass, three are ENCODED; call-sign shape needs one to four final letters (`docs/BEACON_SPEC.md` §6, module docs) |
| `frame` | a frame of exactly `MAX_LEN` (256) bytes is accepted; `FrameError::cmd_id` for every variant |
| `gse` | `ACTIVATE` and `ARM_FLIGHT` decode; `Debug` prints `KeyBytes(..)`, never the key |
| `jetson` | all eight golden lines of ICD_OBC_JETSON.md §4.3, byte for byte; the 512-byte line limit (seal and check); empty and out-of-buffer seals; a 1-byte body; non-printable lines are `Format`; empty numeric fields; ACK/NACK/SHUTDOWN fields of 1 to 176 characters; BEACON_RSP text of 1 to 176 characters (builder and parser); BEACON_REQ max_len 1 to 176; STATUS field ranges at both ends (ICD §4.2); `STATUS_REQ` with fields is `Format`, not `Type`; TELEMETRY in ECLIPSE; `LineRx` keeps a 511-byte line plus LF and drops a 512-byte one; `LineRx::reset` forgets a partial or overlong line |
| `kiss` | the decoder's port is the high nibble of the command byte (ports 1 and 15) |

## Equivalent survivors (9)

| Mutant | Why no behaviour differs |
|--------|--------------------------|
| `auth.rs:45:9` replace `<impl Drop for Key>::drop` with `()` | Zeroizes memory being dropped; no safe code can read it afterwards |
| `gse.rs:65:9` replace `<impl Drop for KeyBytes>::drop` with `()` | Same: zeroize-on-drop of memory that is never readable again |
| `auth.rs:177:38` replace `+` with `*` in `build_frame` | Only the size of the scratch array changes (`OVERHEAD * MAX_ARGS` = 1280 instead of 84 bytes); `build_payload` writes at most 84 and only `..len` is used. It changes stack use, not any output |
| `ax25.rs:36:26` replace `\|` with `^` in `encode_address` | `0x60` (bits 5-6) and `(ssid & 0x0F) << 1` (bits 1-4) share no bit, so OR equals XOR |
| `ax25.rs:36:49` replace `\|` with `^` in `encode_address` | The end-of-address bit (bit 0) shares no bit with bits 1-6, so OR equals XOR |
| `ax25.rs:81:45` replace `+` with `*` in `parse_ui` | `2 * ADDR_LEN + 2 + FCS_LEN` becomes `2 * ADDR_LEN + 2 * FCS_LEN`; both are 18 because `FCS_LEN` is 2 |
| `ax25.rs:95:35` replace `+` with `*` in `parse_ui` | `(i + 1) * ADDR_LEN + 2 + FCS_LEN` becomes `... + 2 * FCS_LEN`; both add 4 because `FCS_LEN` is 2 |
| `jetson.rs:108:36` replace `\|` with `^` in `check` | `received << 4` has its low four bits clear and the digit value is 0-15, so OR equals XOR |
| `jetson.rs:244:41` delete `-` in `parse_from_jetson` | `parse_i32` uses `min` only as `min.unsigned_abs()`, which is 128 for both `-128` and `128` |

## Validation

`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
(software/rust), and `cmake -S software/flight -B build && cmake --build build -j8 && ctest --test-dir build -j8`
(43 of 43 passed) were all run on the final tree.
