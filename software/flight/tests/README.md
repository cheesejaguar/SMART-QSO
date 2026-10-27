# SMART-QSO Flight Software Tests

The CMocka suites in this directory are the specification of the flight software. The flight software is Rust
(`software/rust`; overview in `software/rust/README.md`). These C suites run unchanged against it through the C headers
in `software/flight/include`, which are the C ABI of the Rust modules. They run on the host, on the simulated HAL
(`software/rust/sqso-ffi/src/hal_sim.rs`). They use the [CMocka](https://cmocka.org) framework.

There are 56 ctest entries: 52 CMocka suites with 758 `cmocka_unit_test` cases (two of them run as two
entries each: `integration/test_executive_jetson.c` as `Executive_Jetson_link` and `Executive_Jetson_guard`, and
`integration/test_downlink_sim.c` as `Downlink_TNC_served` and `Downlink_TNC_silent`), the
`no_dynamic_memory` check, and `Simulation_Run`, the host simulation, which must reach a clean shutdown (counted
2026-10-27 with `ctest -N`).

## Running them

From the repository root:

```bash
cmake -S software/flight -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j
ctest --test-dir build --output-on-failure
```

The build runs cargo (`software/flight/cmake/rust.cmake`) to build the Rust library `libsqso_flight.a` (crate
`sqso-clib`) that every suite links, and the host simulation `smart_qso_flight`. Rust is needed. The toolchain is pinned by
`software/rust/rust-toolchain.toml` and rustup installs it on first use. CMocka must be installed (`brew install cmocka`
or `apt install libcmocka-dev`); without it the top-level `CMakeLists.txt` skips the tests directory.

Useful configurations (all run in CI):

| Option | What it adds |
|--------|--------------|
| `-DENABLE_WERROR=ON` (default) | Warnings are errors (the C suites and the flight-log shim) |
| `-DENABLE_SANITIZERS=ON` | AddressSanitizer and UndefinedBehaviorSanitizer on the C; any finding fails the test |
| `-DENABLE_COVERAGE=ON` | gcov instrumentation of the C and LLVM source-based coverage of the Rust library, for the coverage gates |
| `-DFLIGHT_BUILD=ON` | Flight configuration: the simulation is built with the cargo feature `flight` (runs until stopped) |

The Rust also has its own unit tests (`cd software/rust && cargo test --workspace`, 624 `#[test]` functions on 2026-10-27) and fuzz targets
(`software/rust/fuzz`).

## How tests are organised

- One suite per module, `test_<module>.c`, listed in `MODULE_TESTS` in `CMakeLists.txt`. Each links against
  `flight_test_lib`, a static library of the flight-log shim (`software/flight/src/flight_log_fmt.c`) linked with the
  Rust library. A few suites (`test_crc32`, `test_eps_control`, `test_fault_mgmt`, `test_sensors`,
  `test_mission_data`, `test_uart_comm`) have their own blocks at the top of `CMakeLists.txt`.
- `test_rust_contract.c` and `test_rust_contract_adcs.c` compare the size and alignment of every struct the Rust mirrors,
  and every constant it keeps, with the C headers (`rust_contract.h`). A header change that the Rust does not follow
  fails them.
- `integration/` and `regression/` hold cross-module suites.
- `integration/test_executive_jetson.c` tests the main loop (`software/rust/sqso-ffi/src/executive.rs`) end to end. It
  runs the host simulation `smart_qso_flight` (about 20 s) with a pseudo-terminal as the Jetson port (`UART_DEVICE`) and
  a second one as the console, plays the Jetson, and checks what the OBC sends and prints against
  `docs/interfaces/ICD_OBC_JETSON.md` §4. It links no flight code: it checks the CRC with its own CRC-16. Two entries:
  - `Executive_Jetson_link` (the flight `sensors.yaml`): every line sealed with a correct CRC and within 512 bytes;
    each TELEMETRY line has the ICD's fixed fields, the sensor fields in table order and the EPS, fault, UART and
    watchdog fields, and is followed by `BEACON_REQ,<seq>,176` with the sequence counting up from 1. Telemetry comes
    every `TELEMETRY_INTERVAL_MS` (1 s here) while the payload is on, none while the power model has it off, and the
    first one without waiting another interval when the payload comes back on. The Jetson's STATUS reply is taken; of its BEACON_RSP
    answers only those to the latest request with a good CRC are kept.
  - `Executive_Jetson_guard` (a generated sensor table too long for the line): the status fields do not fit and the
    comma after the last sensor field is trimmed. A STATUS with the junction at 99 °C trips the payload guard and no
    telemetry follows. The driver then closes its end of the line (a hang-up) away from the periodic UART health
    check, which must notice it once, in its next window (wall clock within 100 ms of a multiple of 10 s). Timing
    checks allow 250 ms for scheduling on a loaded host (`ARRIVAL_SLACK_MS`), and half an interval for the first
    telemetry after the payload returns (`RESUME_SLACK_MS`), whose pass also writes files and console lines; a
    telemetry that waited for the interval timer is a whole interval late.

  The beacon is not checked here: these boots start the 45-minute RF silence (`integration/test_downlink_sim.c` checks
  it). The driver sets VMIN 0 and VTIME 0 on the Jetson line first, because the OBC's host port keeps the terminal's VMIN 1 and
  its poll of the line would otherwise block until the Jetson sends a byte. Each run writes its transcript
  (`executive_jetson_link.log`, `executive_jetson_guard.log`) to `build/tests`.
- `test_downlink.c` specifies the downlink (`downlink.h`): the gate before the TNC (TX-OFF and each inhibit), one frame
  at a time, the duty credit, the power select line, the HAL refusing a frame, the VSWR, no-output and thermal
  protections (the monitors set with `hal_sim_adc_set()`), failed readings, and the beacon handing its frame over. Its
  first case checks `downlink.h` against the library (`rust_contract.h` ids 300 to 349).
- `integration/test_downlink_sim.c` checks the downlink end to end. It runs `smart_qso_flight` with `TNC_DEVICE` set to
  a file, where the simulated radio HAL writes what the TNC would receive, decodes the KISS stream and the AX.25 frames
  with its own code (it links no flight code), and reads the console. Two entries:
  - `Downlink_TNC_served` (`SIM_RF_SILENCE_SERVED`: a software-reset boot with the silence served and the deployment finished, decision C-19): the five TNC
    parameter commands before every data frame; each frame a UI frame from KK6GDB-11 to CQ; the frames exactly the
    console's `[BEACON] TX:` beacons, at least one; the beacon carrying the power model's SOC, battery voltage and sun
    flag.
  - `Downlink_TNC_silent` (a first boot): the silence is counting, and not one byte reaches the TNC.
- `vectors/` holds data shared with other code. `beacon_filter_vectors.txt` is run by both the OBC filter tests here and
  the Jetson filter tests in `software/payload/tests`.
- `no_dynamic_memory` is a ctest check, not a CMocka suite. It fails if the test library or the Rust library references
  `malloc`, `calloc`, `realloc` or `free` (`software/flight/scripts/check_no_dynamic_memory.sh`).

The simulated fault log, EPS configuration and mission data persist to files. `CMakeLists.txt` gives every test its own
files (`SMART_QSO_FAULT_LOG`, `SMART_QSO_EPS_CONFIG`, `SMART_QSO_MISSION_DATA`), so parallel runs cannot interfere.

## Writing a test

Start from an existing suite, for example `test_beacon_format.c`. It has a file comment that says what is tested and
why, `static void test_x(void **cm_state)` functions, and a `CMUnitTest` array run by `cmocka_run_group_tests`. Assert
what the specification says (the requirement or design document), not what the code happens to do. Tag a test with
`@requirement <ID>` only when the ID exists in `docs/requirements/` or `docs/software/SOFTWARE_REQUIREMENTS.md`. CI fails
if undefined IDs increase (`scripts/check_traceability.py`).

A change to existing behaviour belongs in the Rust and, when deliberate, in the numbered list of behaviour changes in
`software/rust/README.md`.

## Coverage

CI (job Build and Test) runs these gates:

- **Rust**: line coverage of the flight crates (`sqso-protocol`, `sqso-fsw`, `sqso-ffi`), from these suites and the
  Rust unit tests together, must not fall below `RUST_COVERAGE_MIN_LINE` (90 %). It was 94.8 % on 2026-10-25.
- **Flight software**: the C in `src/` (gcovr) and the Rust together must not fall below `COVERAGE_MIN_LINE` (80 %). It
  was 94.8 % on 2026-10-25.
- **Branches and conditions** (a second build on the pinned nightly `BRANCH_COVERAGE_TOOLCHAIN`, with
  `-Zcoverage-options=condition`): Rust branch and condition coverage must not fall below `RUST_BRANCH_COVERAGE_MIN`
  (80 %); it was 81.1 %. Each safety-critical file (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.2) must reach
  100 % of lines and conditions, except lines marked `// COVERAGE-EXEMPT: <reason>` (`../scripts/safety_coverage.py`,
  CR-2026-003).

To reproduce:

```bash
cmake -S software/flight -B build-cov -DCMAKE_BUILD_TYPE=Debug -DENABLE_COVERAGE=ON
cmake --build build-cov -j
LLVM_PROFILE_FILE=$PWD/build-cov/profraw/%p-%m.profraw ctest --test-dir build-cov
software/flight/scripts/rust_coverage.sh build-cov 90
(cd software/flight && gcovr -r . --filter 'src/' ../../build-cov --print-summary)
```

Use the `gcov` that matches the compiler (`--gcov-executable gcov-16` with Homebrew GCC 16).

## Hardware tests

These are host tests only. Hardware-in-the-loop testing is planned in `hil/HIL_TEST_PLAN.md` and has not been run. No
hardware has run the firmware (`software/rust/firmware`).
