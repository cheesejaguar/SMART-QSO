# SMART-QSO Rust modules

The flight software's untrusted-input surface and its mode, scheduling and fault logic are written in Rust. The radio
uplink (AX.25, the command frame, telecommand authentication, key table and replay counters), the OBC–Jetson link, the
GSE umbilical and the beacon content filter all parse bytes from outside the spacecraft. In C, a parser bug there can
crash the spacecraft or give an attacker control of it. In Rust the same mistake is a bounds-checked error, or at worst
a controlled reset (see [Rules](#rules)).

The C flight software is unchanged around these modules. CMake builds this workspace as a static library and links it
into the host simulation, the C test suites and the STM32L4 image, in place of the C files it replaced. The C headers
stay the interface.

## Crates

| Crate | What it is | `unsafe` |
|-------|------------|----------|
| `sqso-protocol` | Wire formats: the command dictionary (generated), uplink frame, telecommand authentication (RustCrypto HMAC-SHA-256), AX.25, the OBC–Jetson line protocol in both directions, the beacon content filter, the GSE frame, and CRCs from the `crc` crate's catalogue. `no_std`, no allocation | forbidden |
| `sqso-fsw` | Flight logic: two-slot NVM records, key table and counters, telecommand admission, the mode manager, the scheduler and the fault log. I/O sits behind a trait or is returned to the caller. `no_std`, no allocation | forbidden |
| `sqso-ffi` | The C ABI: implements the C headers listed below on top of the two crates above. The only crate with `unsafe` | pointer conversion, C calls, module state |
| `sqso-py` | Python module `sqso_protocol` (PyO3, maturin) for the Jetson payload and the ground tools | none |
| `fuzz` | cargo-fuzz targets for every parser (a separate workspace) | none |

### What is Rust and what is still C

| C header (unchanged interface) | Was | Now |
|--------------------------------|-----|-----|
| `ax25.h` | `ax25.c` | `sqso-protocol/src/ax25.rs` |
| `input_validation.h` (command framing) | `input_validation.c` | `sqso-protocol/src/frame.rs`, `sqso-ffi/src/input_validation.rs` |
| `cmd_dictionary.h` | `cmd_dictionary.c` | `sqso-protocol/src/dictionary.rs` (generated) |
| `cmd_auth.h`, `sha256.h` | `cmd_auth.c`, `sha256.c` | `sqso-fsw/src/keys.rs`, `sqso-protocol/src/auth.rs`, RustCrypto `sha2` and `hmac` |
| `nvm_record.h` | `nvm_record.c` | `sqso-fsw/src/nvm.rs` |
| `cmd_handler.h` | `cmd_handler.c` | `sqso-fsw/src/telecommand.rs`, `sqso-ffi/src/cmd_handler.rs` |
| `uplink.h` | `uplink.c` | `sqso-ffi/src/uplink.rs` |
| `gse_provision.h` | `gse_provision.c` | `sqso-protocol/src/gse.rs`, `sqso-ffi/src/gse_provision.rs` |
| `jetson_link.h` | `jetson_link.c` | `sqso-protocol/src/jetson.rs` |
| `beacon_filter.h` | `beacon_filter.c` | `sqso-protocol/src/filter.rs` |
| `state_machine.h` | `state_machine.c` | `sqso-fsw/src/mode.rs`, `sqso-ffi/src/state_machine.rs` |
| `scheduler.h` | `scheduler.c` | `sqso-fsw/src/scheduler.rs`, `sqso-ffi/src/scheduler.rs` |
| `fault_mgmt.h` | `fault_mgmt.c` | `sqso-fsw/src/fault_log.rs`, `sqso-ffi/src/fault_mgmt.rs`; storage stays in C (`fault_store.c`) |
| `smart_qso.h` (CRC-32) | `crc32.c` | `sqso-protocol/src/checksum.rs` |

About 4,700 lines of C were replaced. Still C: the executive (`main.c`), EPS, sensors, beacon formatting, telemetry,
ADCS, deployment, RF control and silence, contact monitor, payload guard, GNSS, mission data, flight log, watchdog
manager, assertion handler, safe strings, the UART driver and the HAL (simulated and STM32L4). Two small C files are
new: `rust_panic.c` (where a Rust panic ends) and `fault_store.c` (fault-log storage and console output).

The Python payload and ground tools use `sqso_protocol` for the same formats: `obc_link.py`, `content_filter.py` and
`crc16.py` (payload), and `telecommand_builder.py` and `key_provision.py` (ground). The OBC, the Jetson and the ground
station now run one implementation of each format (findings SW-04, SW-24).

## Build and test

The C build runs cargo itself (`software/flight/cmake/rust.cmake`), so the commands in the top-level README are
unchanged. The toolchain is pinned in `rust-toolchain.toml`, and rustup installs it on first use.

```bash
# Rust unit tests, lints and the Cortex-M4F build
cd software/rust
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release -p sqso-ffi --target thumbv7em-none-eabihf

# Fuzzing (nightly and cargo-fuzz: cargo install cargo-fuzz)
cd fuzz
cargo +nightly fuzz list
cargo +nightly fuzz run telecommand -- -max_total_time=300

# Python module for the payload and ground tools
pip install software/rust/sqso-py

# Rust unit-test coverage (cargo install cargo-llvm-cov)
cargo llvm-cov --workspace --exclude sqso-py --exclude sqso-ffi --summary-only
```

CI (`.github/workflows/ci.yml`, jobs *Rust Modules* and *Rust Fuzz Smoke*) runs formatting, clippy with warnings as
errors, the unit tests, the target build, the minimum-Rust-version check, a check that `unsafe` stays in `sqso-ffi`,
and 30 s of every fuzz target. The C jobs build and test the linked library as before.

### Adding a command

Add it to `software/flight/command_dictionary.json` and run `python3 scripts/gen_command_dictionary.py`. That
regenerates the Rust `Command` enum, the C ID enum and the ground table. The flight software then stops compiling at
the two places that must decide about the new command: the dispatcher in `sqso-ffi/src/cmd_handler.rs` (what it
does) and `availability()` in `sqso-fsw/src/telecommand.rs` (the modes it may run in). Both `match` statements are
exhaustive, with no wildcard arm.

## Rules

- **No panics on input.** The workspace lints deny indexing that can go out of bounds, `unwrap`, `expect`, `panic!`
  and `unreachable!`, and CI treats every warning as an error. Parsers use `get()` and slice patterns, and each one is
  fuzzed with properties checked, not only "does not crash" (see `fuzz/fuzz_targets`).
- **Checks stay on in flight builds (SW-08).** The release profile keeps integer overflow checks. A Rust panic is a
  fatal assertion: it goes to the C assertion handler (`rust_panic.c`), then the target waits for the watchdog to
  reset it and the host aborts, so the failing test fails.
- **`unsafe` only in `sqso-ffi`.** `sqso-protocol` and `sqso-fsw` declare `#![forbid(unsafe_code)]`. In `sqso-ffi`,
  `unsafe` does three things only: it turns C pointers into references after a NULL check (`ptr.rs`), calls C
  functions, and holds each module's state in `cell::Global`. A nested call into the same module, from a C callback
  or an interrupt, gets "busy" from `Global` instead of a second, aliasing reference.
- **No allocation.** No crate uses `alloc`. The C test `no_dynamic_memory` now checks the Rust archive as well.
- **Key material is zeroized** when dropped (`zeroize`), including the temporary copies. A plain `memset` before
  free can be optimized away; `zeroize` cannot.
- **One ABI.** C enums cross the boundary as 32-bit integers, and the STM32L4 C build uses `-fno-short-enums` so
  they are 32 bits there too. Struct layouts are mirrored with `#[repr(C)]`. The C test `test_rust_contract.c`
  compares the size and alignment of every mirrored struct, and every constant Rust keeps, with the C headers, so a
  header change that Rust does not follow fails the suite.

## Evidence that the port behaves like the C

- **The C suites as the specification.** All 41 CMocka suites (602 test cases) run unchanged against the Rust
  modules. They pass on the host, under AddressSanitizer and UBSan, and in the full simulation. No existing test case
  changed: the edits to existing suites are comments naming the Rust files, `test_crc32` linking the shared test
  library instead of compiling `crc32.c`, and four new cases in `test_uart_comm.c`.
- **Differential fuzzing against the original C.** A one-off harness, not kept in the tree, linked the C files this
  port replaced. For 8.3 million fuzzed inputs (934 coverage edges), the Rust and C implementations returned the same
  results, and the same output fields, for AX.25 parsing, command framing, Jetson line parsing and sealing, the
  content filter, CRC-32, the AX.25 FCS, CRC-16 and HMAC-SHA-256.
- **Same simulation output.** The host simulation prints the same 256 lines as the C build, apart from timestamps
  and the order of one sensor read.
- **Fuzzing.** Each of the 8 targets ran for 2–3 minutes during the port (about 140 million executions in all), with
  no failures. The telecommand target builds genuine frames, then corrupts them, and checks two properties: nothing
  is accepted without the key's tag or with a spent counter, and a genuine fresh command is always accepted.
- **Unit tests** in `sqso-protocol` and `sqso-fsw`: 60 tests, 88 % line coverage of the two crates (cargo-llvm-cov).
- **The Python suites** (payload 151 tests, ground 133) pass on `sqso_protocol`. Their golden lines and frames are
  the ones the C suites check.

### Behaviour changes from the C

The port keeps the C behaviour, apart from the following deliberate changes:

1. `scheduler_delay_ms()` runs scheduler ticks until the delay has passed. The C version waited for something else to
   advance the tick count; nothing did, so any delay above 0 never returned.
2. A scheduler task that suspends or disables itself while it runs keeps that state. The C version set it back to
   READY after the run.
3. A module entered again while it is busy (an interrupt, or a C callback into the same module) returns
   `SMART_QSO_ERROR_BUSY` instead of racing on its state.
4. `EPS_GET_TELEMETRY` saturates an out-of-range voltage or SOC when packing it into a byte. In C that conversion was
   undefined behaviour.
5. `fault_log_add()` refuses a fault type above 255 instead of truncating it into the one-byte field.
6. `Sha256Ctx_t` is opaque storage for the Rust hasher. `sha256_update()` or `sha256_final()` on a context that was
   never initialized returns `SMART_QSO_ERROR_INVALID`.
7. `cmd_get_name()` returns the dictionary names, for example `EPS_GET_TELEMETRY` (was `EPS_GET_TLM`) and
   `SET_BEACON` (was `COMM_SET_BEACON`).
8. On the Jetson side, the line builders apply the OBC's own rules. `BEACON_REQ` needs `max_len` 1–176. `ACK`, `NACK`
   and `SHUTDOWN` take one field without commas, which the OBC would otherwise refuse. TELEMETRY refuses NaN and
   infinite values, which Python's `float()` accepted.
9. A loaded fault-log entry whose `recovered` flag is not 0 or 1 is discarded as corrupt (C would have read an
   invalid `bool`).

## Next steps

- **Bring-up on the STM32L4R5 with Embassy.** The C target port is compile-only: register-level HAL, no RTOS, no
  startup checks. That bring-up has to happen anyway, and Embassy supports this chip (`embassy-stm32`, feature
  `stm32l4r5zi`). The crates here are already `no_std` and cross-compile for `thumbv7em-none-eabihf`, so an Embassy
  executable can use `sqso-protocol` and `sqso-fsw` directly, with no FFI. Doing it before flight hardware exists is
  the cheapest point.
- **Port more modules** in order of exposure: the payload guard and EPS policy (power and thermal limits), then the
  watchdog manager, deployment and RF silence. Their C tests can serve as the specification again.
- **Software assurance paperwork.** MISRA C does not apply to Rust code. The NPR 7150.2 classification, the coding
  standard (`docs/development/CODING_STANDARDS.md`) and the JPL Power of Ten audit need a Rust section (see
  `software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md` §1.1). The Ferrocene toolchain is qualified (ISO 26262, IEC 61508)
  if a qualified compiler is needed.
- **Jetson deployment** now builds `sqso_protocol` on the Jetson, with Rust installed by
  `software/payload/ansible/deploy_payload.yml`. Shipping a prebuilt aarch64 wheel would remove that step.
