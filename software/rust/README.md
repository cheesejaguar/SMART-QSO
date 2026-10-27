# SMART-QSO flight software (Rust)

Since 2026-10-24 the SMART-QSO flight software is Rust. Every flight module, the executive (the main loop that replaced
`main.c`), the simulated HAL of the host build and the STM32L4R5ZI firmware are in this directory. One C file is left in
the flight code: `software/flight/src/flight_log_fmt.c`, about 70 lines that implement the printf-style `log_write()`
and `log_vwrite()`, because stable Rust cannot define a C variadic function (see [What C remains](#what-c-remains)).

The C headers in `software/flight/include` did not change. They are the C ABI of the Rust modules. The C test suites in
`software/flight/tests` (CMocka) are the specification: they run unchanged against the Rust through that ABI.

Parsers on the untrusted-input surface (the AX.25 uplink, the command frame, telecommand authentication and the key
table, the OBC–Jetson link, the GSE umbilical, the beacon content filter, the sensor configuration file and the GNSS
NMEA stream) are where a C bug could crash the spacecraft or give an attacker control of it. In Rust the same mistake is
a bounds-checked error, or at worst a controlled reset (see [Rules](#rules)).

## Crates

| Crate | What it is | `unsafe` |
|-------|------------|----------|
| `sqso-protocol` | Wire formats: the command dictionary (generated), uplink frame, telecommand authentication (RustCrypto HMAC-SHA-256), AX.25, KISS, the OBC–Jetson line protocol in both directions, the beacon content filter, the GSE frame, and CRCs from the `crc` crate's catalogue. `no_std`, no allocation | forbidden (`#![forbid(unsafe_code)]`) |
| `sqso-fsw` | The flight logic of every module (list in `sqso-fsw/src/lib.rs`), with its I/O behind a trait or returned to the caller so it can be unit-tested and fuzzed on the host. `no_std`, no allocation | forbidden (`#![forbid(unsafe_code)]`) |
| `sqso-ffi` | The flight software behind the C headers: the C ABI glue, each module's state, the executive (`executive.rs`), the simulated HAL of the host build (`hal_sim.rs`), host operating-system access through libc (`os.rs`, host only) and persistent storage (`store.rs`) | pointer conversion after NULL checks, C and HAL calls, module state |
| `sqso-clib` | `sqso-ffi` as the static library `libsqso_flight.a` that the C test suites link; owns the panic handler there | none |
| `sqso-sim` | The host simulation, the binary `smart_qso_flight`. Feature `flight` (the CMake `FLIGHT_BUILD`): run until stopped instead of 1000 passes | none |
| `sqso-sil` | The mission simulation (software in the loop): the flight software, unchanged, on the simulated HAL and a virtual clock, against models of the orbit, attitude, power, thermal, RF, radiation, deployment hardware, the Jetson, the ground segment and the transponder's users, from integration to re-entry, with the checks that judge a run (`docs/test/SIL_MISSION_SIMULATION.md`). Build profile `sil` | calls into the C ABI of `sqso-ffi` (the HAL hooks, `adcs_control.h`) |
| `sqso-py` | Python module `sqso_protocol` (PyO3, maturin) for the Jetson payload and the ground tools | none (CI checks) |
| `fuzz` | 12 cargo-fuzz targets: one per parser, and `fixed_format` (`fuzz/fuzz_targets`), including `sensors_yaml`, `gnss_nmea` and `kiss_stream`. A separate workspace | none |
| `firmware` | The STM32L4R5ZI image (Embassy). A separate workspace, built only for `thumbv7em-none-eabihf` | register access, HAL exports, driver state, the two calibration reads in `firmware/src/adc.rs` |

### The firmware

`firmware/` is the flight image. It uses embassy-stm32 0.6 (features `stm32l4r5zi`, `dual-bank`, `time-driver-tim2`: TIM1, TIM3, TIM8 and TIM15 belong to the magnetorquers and the captures),
embassy-executor 0.10, embassy-time 0.5 and cortex-m-rt. There is one async main task (`firmware/src/main.rs`). It
starts the independent watchdog (IWDG) at its longest timeout (about 32 s) before the clock set-up, so a PLL that never
locks resets the OBC. It then brings the chip up (80 MHz from the 4 MHz MSI through the PLL, M = 1, N = 40, R = 2),
installs the HAL drivers and calls `Executive::start()`. After that it loops on `step()` and
`Timer::after_millis(20).await`, and the core sleeps in the executor between passes. The watchdog manager sets the
operating timeout (30 s, `WDT_HW_TIMEOUT_MS`).

The HAL (`hal/*.h`) is implemented in Rust as `#[no_mangle]` `hal_*` functions:

| HAL | File | What |
|-----|------|------|
| `hal_gpio.h` | `gpio.rs` | The master pin allocation, SQSO-ICX-001 B §4 with every row of its §6 change list (flight software round 3): PAYLOAD_EN PB2, DEPLOY_EN PF3, JETSON_PWR PA8, EOL_ARM PF4, EOL_FIRE PG1, RX_CD PG0, RX_FAULT PF13, BURN_W2B PE7, LOCK_CLR PF5, and the logical pins 25 to 45 (the sensor rails and their readbacks, GYRO_RST_N, the data-ready lines, the magnetorquer direction and fault lines, TONE_OK, XPDR_KEY, PTT_EXC, JET_PG, GNSS_PG, WDI). Every output is driven low before its driver is enabled, so no enable, burn or passivation line is high at reset. `ANTENNA_SENSE` (13) is not wired (the AntS reports over I2C); PTT_EXC (42) is read from the TIM8 capture's pin |
| `hal_mtq.h` | `mtq.rs` | The magnetorquer bridges: TIM1 CH1–CH3 PWM at 40 kHz (PE9/PE11/PE13), MTQ_ARM a TIM15 one-pulse of at most 900 ms (PF9), so the coils stop at the end of a burst whatever the software does |
| `hal_capture.h` | `capture.rs` | TIM3 captures of the GNSS 1PPS (PC6) and the RV-3028's 1 Hz (PC8), TIM8 both edges of PTT_EXCITER (PC7): the clock check and the exciter's keyed time |
| `hal_timer.h` | `timer.rs` | Time base from embassy-time (milliseconds since boot) |
| `hal_wdt.h`, `hal_reset.h` | `wdt.rs` | IWDG; reset cause from the RCC CSR flags |
| `hal_uart.h` | `uart.rs` | Buffered UARTs: Jetson USART1 (PA9/PA10, 8N2: the module's receiver needs two stop bits), debug USART3 (PD8/PD9) |
| `hal_i2c.h` | `i2c.rs` | I2C2 (EPS controller 0x48, the INA228s 0x44 and 0x40, AntS 0x31; PF1/PF0), I2C4 (ADS7138 0x10, RM3100 0x20; PD12/PD13), I2C1 (RV-3028-C7 0x52) |
| `hal_spi.h` | `spi.rs` | Logical bus 0: SPI2 with the BMI088 gyro's chip select PD0; bus 1: SPI3 with the M-G370PDT0's PD1 (mode 3); bus 2: SPI2 with the BMI088 accelerometer's PD4. Each bus keeps its own configuration, applied when its chip select is asserted. The sensor drivers are `sqso_fsw::mg370`, `bmi088`, `rm3100`, `ads7138`, `rv3028`, `ina228` (built against the data sheets and tested with register-level mocks; not run on hardware) |
| `hal_adc.h` | `adc.rs` | ADC1, VDDA measured against VREFINT, MCU temperature from TS_CAL; the downlink monitors TX_TEMP (PA0), TX_FWD (PC0) and TX_REV (PC1), the power ones square-law (an RMS detector); front-end scaling provisional (TBR-SRS-003) |
| `hal_flash.h` | `fram.rs` | FM25V20A FRAM on SPI1 (PG2/PG3/PG4, CS PG5, WP PE0, HOLD PE1), 17 regions at the `hardware/obc/STORAGE_DESIGN.md` §3.3 addresses; the fault log uses 9,728 B of the event log, the stored housekeeping the 64 KB telemetry buffer and 90 KB of the extended storage (`software/flight/docs/MEMORY_POLICY.md` §8.1) |
| `hal_radio.h` | `radio.rs` | The KISS TNC on USART2 (PD5/PD6, 9600 baud) for both directions (CR-2026-005): received frames decoded by `sqso_protocol::kiss`, with the FCS the TNC removed appended again; downlink frames written after the TNC parameters, from a 1 KB buffer that holds the longest stream, so the write never waits |

Not provided: the HAL's callback timers and the watchdog early-warning callback. Both are refused, as they were in the
former C port. The release image is 190,532 B of text, 24,776 B of data and 17,012 B of bss (2026-10-27, round 3); the data, the bss and
the 8 KB stack are in SRAM2 (parity). The vector table is at 0x08000000, the initial SP is 0x20040000 and the entry point
is `Reset`. No hardware has run it. CI builds the image and checks those properties.

For the first run on a board, `firmware/BRINGUP.md` gives the equipment, the NUCLEO-L4R5ZI's differences from the
flight pin map (its ST-LINK virtual COM port is LPUART1, not the USART3 console), power, flashing, the order of the first
checks with their HIL test IDs, and what to record. The firmware times every `step()` pass with the DWT cycle counter
(`firmware/src/timing.rs`) and keeps the last and longest pass, the passes over 20 ms and the passes recorded in the
`#[no_mangle]` static `SQSO_LOOP_TIMING`, which a probe reads by name (HIL-040); nothing on board reads it. The scripts in
`firmware/tools/` read it, the painted stack budget (HIL-044) and run the stack-guard test (HIL-004) through the
`probe-rs` command line; their unit tests (`python3 -m pytest -q firmware/tools/tests`) need no board.

## Porting table

Every C source file of the flight software before the port (and `fault_store.c` and `rust_panic.c`, which existed only
during it), and where it is now. The C headers kept their names and content. "fsw" is `sqso-fsw/src`, "ffi" is `sqso-ffi/src`, "protocol" is `sqso-protocol/src`.

| Former C file | Rust |
|---------------|------|
| `adcs_control.c` | fsw `adcs.rs`, ffi `adcs_control.rs` |
| `assert_handler.c` | fsw `assert_handler.rs`, ffi `assert_handler.rs` |
| `ax25.c` | protocol `ax25.rs`, ffi `ax25.rs` |
| `beacon.c` | fsw `beacon.rs`, ffi `beacon.rs` |
| `beacon_filter.c` | protocol `filter.rs`, ffi `beacon_filter.rs` |
| `cmd_auth.c` | fsw `keys.rs`, protocol `auth.rs`, ffi `cmd_auth.rs` |
| `cmd_dictionary.c` | protocol `dictionary.rs` (generated) and `command.rs`, ffi `cmd_dictionary.rs` |
| `cmd_handler.c` | fsw `telecommand.rs`, ffi `cmd_handler.rs` |
| `contact_monitor.c` | fsw `contact_monitor.rs`, ffi `contact_monitor.rs` |
| `crc32.c` | protocol `checksum.rs`, ffi `crc.rs` |
| `deployment.c` | fsw `deployment.rs`, ffi `deployment.rs` |
| `eps_control.c` | fsw `eps.rs`, ffi `eps_control.rs` |
| `fault_mgmt.c` | fsw `fault_log.rs`, ffi `fault_mgmt.rs` |
| `fault_store.c` (and `fault_store.h`) | ffi `store.rs` (storage), ffi `fault_mgmt.rs` (console output) |
| `flight_log.c` | fsw `flight_log.rs`, ffi `flight_log.rs`; the printf entry points stay C (`software/flight/src/flight_log_fmt.c`) |
| `gnss.c` | fsw `gnss.rs`, ffi `gnss.rs` |
| `gse_provision.c` | protocol `gse.rs`, ffi `gse_provision.rs` |
| `input_validation.c` | protocol `frame.rs`, ffi `input_validation.rs` |
| `jetson_link.c` | protocol `jetson.rs`, ffi `jetson_link.rs` |
| `main.c` | ffi `executive.rs` (bring-up, main loop, shutdown), fsw `executive.rs` (power model, field rules), `sqso-sim/src/main.rs` (host program), `firmware/src/main.rs` (target) |
| `mission_data.c` | fsw `mission_data.rs`, ffi `mission_data.rs` |
| `nvm_record.c` | fsw `nvm.rs`, ffi `nvm_record.rs` |
| `payload_guard.c` | fsw `payload_guard.rs`, ffi `payload_guard.rs` |
| `rf_control.c` | fsw `rf_control.rs`, ffi `rf_control.rs` |
| `rf_silence.c` | fsw `rf_silence.rs`, ffi `rf_silence.rs` |
| — (`downlink.h`, new 2026-10-25, no C original) | fsw `downlink.rs`, ffi `downlink.rs`, protocol `kiss.rs` (the TNC parameters) |
| `rust_panic.c` | ffi `lib.rs` (`report_panic`); panic handlers in `sqso-clib/src/lib.rs` and `firmware/src/main.rs`, panic hook in `sqso-sim/src/main.rs` |
| `safe_string.c` | fsw `safe_string.rs`, ffi `safe_string.rs` |
| `scheduler.c` | fsw `scheduler.rs`, ffi `scheduler.rs` |
| `sensors.c` | fsw `sensors.rs` (configuration parser, readings, telemetry text), ffi `sensors.rs` |
| `separation_switch.c` | ffi `separation_switch.rs` |
| `sha256.c` | ffi `sha256.rs` on RustCrypto `sha2` and `hmac` |
| `state_machine.c` | fsw `mode.rs`, ffi `state_machine.rs` |
| `system_state.c` | fsw `system_state.rs`, ffi `system_state.rs` |
| `telemetry.c` | fsw `telemetry.rs`, ffi `telemetry.rs` |
| `time_utils.c` | ffi `time.rs` (mission clock); host clock in ffi `os.rs` |
| `uart_comm.c` | fsw `uart_comm.rs` (settings), ffi `uart_comm.rs` (termios on the host, the HAL on the target) |
| `uplink.c` | ffi `uplink.rs` |
| `watchdog_mgr.c` | fsw `watchdog.rs`, ffi `watchdog_mgr.rs` |
| `hal/hal_sim.c` | ffi `hal_sim.rs` (host only) |
| `hal/hal_stm32l4.c` | `firmware/src/`: `gpio.rs`, `timer.rs`, `wdt.rs`, `uart.rs`, `i2c.rs`, `spi.rs`, `adc.rs`, `fram.rs`, `radio.rs` |
| `hal/stm32l4_clock.c` (and `hal/stm32l4_clock.h`) | `clocks()` in `firmware/src/main.rs` (embassy-stm32 RCC configuration) |
| `hal/stm32l4_startup.c` | cortex-m-rt (vector table, `Reset`) |
| `hal/stm32l4_syscalls.c` | not needed: the image links no C library |
| `cmake/arm-none-eabi-stm32l4.cmake`, `cmake/stm32l4r5zi.ld` | `firmware/.cargo/config.toml`, cortex-m-rt `link.x` with the firmware's own `memory.x` (statics in SRAM2, the parity RAM; `firmware/src/parity.rs`, RID AV-07) |

Two files that were in the tree until 2026-10-16, `system_init.c` and `yaml_parser.c`, were removed then as dead code and
were not ported. The ports are commits 36c7b7b, 000fe0a, 4c2f0f7, 636b871, 9e7c95a, af06dbb, df3a9c2 and
c56e5f4 (`git log --stat --diff-filter=D -- software/flight/src`).

### What C remains

- `software/flight/src/flight_log_fmt.c`: `log_write()` and `log_vwrite()` take a printf format and C variadic
  arguments, which stable Rust can neither define nor read. The shim asks `flight_log_admit()` whether the level passes,
  formats the message with `vsnprintf` and hands the text to `flight_log_record()`. It is compiled into the C test
  library on the host. The firmware does not link it: the image has no C library and no `printf`.
- The C headers in `software/flight/include` (the interface) and the C test suites in `software/flight/tests`.

The Python payload and ground tools use `sqso_protocol` for the same formats: `obc_link.py`, `content_filter.py` and
`crc16.py` (payload), and `telecommand_builder.py` and `key_provision.py` (ground). The OBC, the Jetson and the ground
station run one implementation of each format (findings SW-04, SW-24).

## Build and test

```bash
# Everything the C side needs: cargo builds the library (sqso-clib) and the
# simulation (sqso-sim) through software/flight/cmake/rust.cmake; ctest runs
# the C suites against the Rust
cmake -S software/flight -B build && cmake --build build && ctest --test-dir build

# Rust unit tests and lints (host)
cd software/rust
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# The flight software library for the Cortex-M4F
cargo build --release -p sqso-ffi --target thumbv7em-none-eabihf

# The firmware image (its own workspace)
cd firmware && cargo build --release && cargo clippy --release -- -D warnings
# target/thumbv7em-none-eabihf/release/sqso-firmware; on a board: cargo run --release (probe-rs)

# Fuzzing (nightly and cargo-fuzz: cargo install cargo-fuzz)
cd software/rust/fuzz
cargo +nightly fuzz list
cargo +nightly fuzz run telecommand -- -max_total_time=300

# Coverage of the Rust from the C suites and the Rust unit tests together
cmake -S software/flight -B build -DENABLE_COVERAGE=ON && cmake --build build
LLVM_PROFILE_FILE=$PWD/build/profraw/%p-%m.profraw ctest --test-dir build
software/flight/scripts/rust_coverage.sh build 90

# Python module for the payload and ground tools
pip install software/rust/sqso-py
```

The toolchain is pinned in `rust-toolchain.toml` (1.97.0, with the `thumbv7em-none-eabihf` target), and rustup installs
it on first use. The minimum supported Rust version is 1.88 (`rust-version` in `Cargo.toml`). Fuzzing uses nightly.
There is no RTOS. FreeRTOS was never integrated and is not planned. The Embassy executor is the scheduler of the image
(`software/flight/docs/RTOS_ARCHITECTURE.md`).

### CI

The Rust runs in the required jobs (`.github/workflows/ci.yml`, `.github/workflows/flight-software-ci.yml`). The
separate Rust jobs were folded into them.

| Job | Rust it runs |
|-----|--------------|
| Build and Test | ctest with the Rust library instrumented; `cargo test --workspace`; Rust line coverage ≥ 90 % (C suites and unit tests); flight-software line coverage, C and Rust together, ≥ 80 %; Rust branch and condition coverage ≥ 80 % and each safety-critical file at 100 % of lines and conditions (a second instrumented build on a pinned nightly, the only toolchain that counts branches; `software/flight/scripts/safety_coverage.py`); `cargo check` with Rust 1.88 |
| Static Analysis | rustfmt; clippy with `-D warnings` for the host and for `thumbv7em-none-eabihf`; a check that `sqso-protocol` and `sqso-fsw` keep `#![forbid(unsafe_code)]` and `sqso-py` has no `unsafe`. cppcheck, clang-tidy and the function-length ratchet run on the remaining C |
| MISRA C:2012 Check | The cppcheck MISRA addon on the remaining C (the shim): 42 findings, baseline `software/flight/quality/misra.baseline` |
| Security Scan | CodeQL for c-cpp and rust; `cargo audit` for both workspaces |
| Sanitizer Checks | The C suites against the Rust under ASan/UBSan; 30 s of every fuzz target; the simulation run to completion |
| STM32L4 Cross-Compile (library) | `sqso-ffi` and `sqso-clib` for `thumbv7em-none-eabihf`; firmware rustfmt, clippy and release build; checks of the linked image (entry point, vector table, initial SP, flight symbols present, no `malloc`/`free`/`_sbrk`/`printf`); the worst-case stack ≤ 8 KB (`scripts/stack_usage.py`, frames cross-checked against LLVM's); the release package as a development build (`scripts/release_package.sh --dev`) |
| Flight Configuration Build, Flight Build Validation | The `FLIGHT_BUILD` configuration (simulation with feature `flight`) |
| Unit Tests | ctest in a Debug build with a coverage report (the gate is in Build and Test) |
| Documentation Check | The required software documents exist |

The program checks (`.github/workflows/program-checks.yml`: Payload, Ground and Simulation Tests; Documentation Reference
Integrity; Secret Scan) run the Python suites on `sqso_protocol` and the repository's document checks.

### Adding a command

Add it to `software/flight/command_dictionary.json` and run `python3 scripts/gen_command_dictionary.py`. That
regenerates the Rust `Command` enum, the C ID enum and the ground table. The flight software then stops compiling at
the two places that must decide about the new command: the dispatcher in `sqso-ffi/src/cmd_handler.rs` (what it does)
and `availability()` in `sqso-fsw/src/telecommand.rs` (the modes it may run in). Both `match` statements are
exhaustive, with no wildcard arm.

## Rules

The full standard is the Rust section of `docs/development/CODING_STANDARDS.md`. The JPL Power of Ten mapping is
`software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md`.

- **No panics on input.** The workspace lints (`Cargo.toml`, the same in `firmware/Cargo.toml`) deny
  `indexing_slicing`, `unwrap_used`, `expect_used`, `panic`, `unreachable`, `todo`, `unimplemented` and
  `too_many_lines`, and warn on `cast_possible_truncation` and `missing_docs`. CI runs clippy with `-D warnings`, so
  every warning is an error. Parsers use `get()` and slice patterns, and each one is fuzzed with its properties checked,
  not only "does not crash". The 10 remaining `#[allow(clippy::indexing_slicing)]` are in `const` items and
  `const fn`s, where `get()` is not available. Each index is bounded by its loop condition, and in a `const` item an
  out-of-bounds index is a compile error.
- **Short functions (JPL rule 4).** `too_many_lines` is denied at 60 lines (`clippy.toml`). There are two documented
  exceptions, each one exhaustive `match`: `dispatch` in `sqso-ffi/src/cmd_handler.rs` and `transition` in
  `sqso-fsw/src/mode.rs`.
- **Checks stay on in flight builds (SW-08).** The release profile keeps `overflow-checks = true`, with
  `panic = "abort"` and debug assertions off. A panic is a fatal assertion. `report_panic` in `sqso-ffi/src/lib.rs`
  reports it to the assertion handler. Then the host aborts, so the failing test fails. The firmware waits for the IWDG
  to reset it, or resets itself if the IWDG was never started. The firmware reports only the first panic: a panic while
  one is reported goes straight to the reset.
- **Stack within 8 KB (SRS-C003).** `scripts/stack_usage.py` (CI) computes the image's worst case from its machine code
  and fails above 8 KB, on a call-graph cycle, and on any new function pointer until its targets are reviewed (the
  tables `INDIRECT` and `CLASSES` in the script). A large buffer in a function that calls deep paths counts in full:
  keep big locals in a function of their own (`#[inline(never)]`) or in a static. Floats are formatted with
  `sqso_fsw::fixed`, not `{:.N}` (core's exact float formatting takes 2 KB of stack). The fault path builds its lines
  without `core::fmt` (`c::Line`). At boot the firmware paints the 8 KB and arms an MPU guard below it
  (`firmware/src/stack.rs`).
- **`unsafe` only in `sqso-ffi` and the firmware.** `sqso-protocol` and `sqso-fsw` declare `#![forbid(unsafe_code)]`.
  In `sqso-ffi`, `unsafe` turns C pointers into references after a NULL check (`ptr.rs`), calls C and HAL functions, and
  holds each module's state in `cell::Global`. In the firmware it is register access, the exported HAL functions,
  driver state and the two factory-calibration reads in `adc.rs`. Each `unsafe` block is covered by a `// SAFETY:` comment.
  In `dispatch` (`cmd_handler.rs`), one comment at the top covers all of the function's calls. Review checks this; no
  lint enforces it. A scan on 2026-10-24 found all 415 `unsafe` blocks and impls covered.
- **Module state in a busy-flag cell.** `cell::Global` hands out one `&mut` at a time. A nested call into the same
  module, from a C callback or an interrupt, gets "busy" instead of a second, aliasing reference (behaviour changes 3
  and 10).
- **No allocation.** No crate uses `alloc`. The ctest `no_dynamic_memory` checks the library for
  `malloc`/`calloc`/`realloc`/`free`. The firmware links no heap and no C library, and CI checks that the image has no
  `malloc`, `free`, `_sbrk` or `printf`.
- **Key material is zeroized** when dropped (`zeroize`), including the temporary copies. A plain `memset` before free can
  be optimized away; `zeroize` cannot.
- **One ABI.** C enums cross the boundary as `u32`, or `i32` where they have negative values. Struct layouts are mirrored
  with `#[repr(C)]`. The C test `software/flight/tests/test_rust_contract.c` (with `test_rust_contract_adcs.c`) compares
  the size and alignment of every mirrored struct, and every constant Rust keeps, with the C headers. A header change
  that Rust does not follow therefore fails the suite. No C is cross-compiled any more, so the `-fno-short-enums` flag
  of the former STM32L4 C build is no longer needed.

## Evidence that the Rust behaves like the C

Measured 2026-10-24.

- **The C suites as the specification.** 49 ctest entries: 45 CMocka suites with 695 `cmocka_unit_test` cases
  (`test_executive_jetson`, which runs the host simulation with a pseudo-terminal as the Jetson port, and
  `test_downlink_sim`, which decodes the simulation's beacons as the TNC gets them, are two entries each), the `no_dynamic_memory` check and `Simulation_Run` (the host simulation, checked for a clean shutdown). They run
  unchanged against the Rust on the host and under AddressSanitizer and UBSan.
- **Coverage.** Rust line coverage is 94.8 %, from the C suites plus the 358 Rust unit tests (gate 90 %). Flight-software
  line coverage, C and Rust together, is 94.8 % (gate 80 %). Branch and condition coverage is 81.1 % (gate 80 %). The
  26 safety-critical files (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.2; the transponder and the operations
  record added on 2026-10-27) are at 100 % of lines and conditions, gated file by file, with 15 reviewed
  `// COVERAGE-EXEMPT:` lines (CR-2026-003, in place of MC/DC). On 2026-10-27: Rust lines 94.3 %, branches and
  conditions 81.4 %.
- **Stack.** The image's worst case is 6,004 B of the 8 KB budget on 2026-10-27 (5,004 B on 2026-10-25, the figures below; SRS-C003; `scripts/stack_usage.py`):
  3,368 B in thread mode plus one handler per priority level. Every frame the script reads from the image agrees with
  LLVM's own `.stack_sizes` (303 functions; the other 34, compiler-builtins and assembly, are read from the image only).
  The call graph has no cycle except the panic handler's single nesting. Not yet measured on hardware (HIL-044).
- **Same simulation output.** The host simulation prints the same 256 lines as the C build, apart from timestamps and
  the order of one sensor read.
- **Differential tests against the former C.** One-off harnesses, not kept in the tree, linked the C files each port
  replaced and compared results and output fields. They covered 8.3 million fuzzed inputs for the protocol layer
  (AX.25, command framing, Jetson lines, the content filter, CRC-32, the AX.25 FCS, CRC-16, HMAC-SHA-256), 13,000
  inputs each for the sensor configuration parser and the GNSS driver, 2 million `safe_string` calls, and 300,000
  operations on the flight log and the assertion handler. All results were identical, except for the deliberate
  changes listed below.
- **Fuzzing.** 12 targets. CI runs each for 30 s on every change. The telecommand target builds genuine frames, then
  corrupts them. It checks two properties: nothing is accepted without the key's tag or with a spent counter, and a
  genuine fresh command is always accepted.
- **Dependencies.** `cargo audit` found RUSTSEC-2026-0385 (`ctutils` 0.4.2, non-constant-time comparisons, used under
  the HMAC verification). `ctutils` was updated to 0.4.5.
- **The Python suites** (payload and ground) pass on `sqso_protocol`. Their golden lines and frames are the ones the C
  suites check.
- **Mutation testing.** The C mutation checks were repeated on the Rust code, with cargo-mutants 27.1 (Rust unit
  tests, then each surviving mutant against the C suites with `mutants/ctest_missed.py`):

  | Scope | Mutants | Caught | Missed | Unviable | Timeout |
  |---|---|---|---|---|---|
  | `sqso-protocol` | 498 | 433 | 9 | 53 | 3 |
  | `sqso-fsw` | 2180 | 1991 | 35 | 153 | 1 |
  | Targeted (the mutations the C commits and audit rows record) | 106 | 106 | 0 | 0 | 0 |

  Every remaining survivor is argued equivalent, one by one, in `mutants/RESULTS-*.md`. The functions changed for the
  stack budget (2026-10-25) were run again (`mutants/RESULTS-stack-changes.md`): the survivors led to three new tests;
  what remains is console output and the executive's main loop, which no test exercises with a Jetson port. None is in the key table,
  telecommand authentication, mode manager, RF control, RF silence, contact monitor, payload guard, EPS, watchdog or
  fault log.

### Behaviour changes from the C

The Rust keeps the C behaviour, apart from the following deliberate changes:

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
10. Re-entering any module returns `SMART_QSO_ERROR_BUSY` (or the getter's default value). The safety-relevant ones fail
    safe: `rf_control_tx_off()` still drives RF_TX_EN low, `rf_control_set_inhibit()` drops TX_EN (and RADIO_EN for the
    silence), `payload_guard_step()` returns `PG_TRIP_NO_TELEMETRY`, `payload_guard_may_enable()` returns false and
    `rf_silence_active()` returns true.
11. Tick arithmetic (contact monitor, RF silence, deployment, the executive's elapsed time) is 64-bit or saturating
    where the C wrapped near 2^32.
12. The payload-guard cool-down end saturates.
13. Deployment's invalid-state branch is gone: the state is an enum, and stored records are checked.
14. `safe_strcpy()`, `safe_strncpy()` and `safe_strcat()` refuse a source that overlaps the destination
    (`RESULT_ERROR_PARAM`).
15. A log callback that logs gets its own entry. An assertion raised during assertion handling is counted.
16. The host prints the `[ASSERT] file:line: msg` line in every build. Host builds always use termios and print log
    lines to stderr.
17. On the target, console lines (beacon TX, ADCS, UART configuration) go to the debug UART. Newlib discarded them.
18. `sensors_load_defaults()` stops at 32 sensors. The C wrote past its table when called more than 3 times.
19. A stored EPS record whose bool bytes are not 0 or 1 is refused. In a loaded system state such a byte is read as
    true. C read an invalid `bool` in both cases. Padding is zeroed.
20. Float-to-integer conversions saturate (NaN gives 0) instead of being C undefined behaviour.
21. `beacon_serialize_ax25()` returns 0 if `info_len` > 256. The C read past the array.
22. On the target, the fault log, EPS configuration, mission data and system state are stored in their FRAM regions,
    each with a 4-byte length. The fault log keeps the newest entries that fit. The C on the target lost them.
23. The simulated HAL refuses an I2C register read past register 0xFF instead of reading past its array.
24. On the target, the simulated sensor noise comes from a xorshift generator, because there is no C library.
25. `Sensor_t.read` with a NULL pointer returns false instead of crashing. `[READ]` lines are cut at 255 bytes.

## Next steps

Still open:

- **Hardware bring-up.** Run the firmware on an STM32L4R5ZI board (the `probe-rs` runner is configured in
  `firmware/.cargo/config.toml`). Measure the loop timing, the stack high-water mark (HIL-044, against the static
  worst case) and the power, and check every HAL driver against the part (TP-FLATSAT-001).
- **Downlink hardware.** The downlink is written (`downlink.h`, CR-2026-005) but the TNC and transmitter are not
  selected (SQSO-ICD-005): their parameters, the thresholds and the monitor scaling are TBR-SRS-003, checked by
  HIL-060 to HIL-064 (`docs/reviews/KNOWN_ISSUES.md` KI-001).
- **ADCS drivers and real sensor readings.** The magnetometer, sun sensor and magnetorquer drivers, and ADC-based
  sensor-table readings with the real front-end scaling, wait for parts and the EPS sense circuits
  (`docs/reviews/KNOWN_ISSUES.md` KI-001).
- **Pin map.** `firmware/src/gpio.rs` is provisional until the OBC schematic baselines a pinout.
  `hardware/obc/OBC_DESIGN.md` and `hardware/obc/STORAGE_DESIGN.md` disagree on PA4–PA7 and PC0–PC1 (FRAM WP#/HOLD#,
  now on PE0/PE1).
- **Jetson deployment** builds `sqso_protocol` on the Jetson, with Rust installed by
  `software/payload/ansible/deploy_payload.yml`. Shipping a prebuilt aarch64 wheel would remove that step.
