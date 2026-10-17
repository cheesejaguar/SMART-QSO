# SMART-QSO Flight Software Tests

Unit and integration tests for the flight software in `software/flight/src/`, run on the host
against the simulated HAL (`src/hal/hal_sim.c`). They use the [CMocka](https://cmocka.org) framework.

## Running them

From `software/flight`:

```bash
cmake -S . -B build -DCMAKE_BUILD_TYPE=Debug
cmake --build build -j
ctest --test-dir build --output-on-failure
```

CMocka must be installed (`brew install cmocka` or `apt install libcmocka-dev`); without it the
top-level `CMakeLists.txt` skips the tests directory.

Useful configurations (all run in CI):

| Option | What it adds |
|--------|--------------|
| `-DENABLE_WERROR=ON` (default) | Warnings are errors |
| `-DENABLE_SANITIZERS=ON` | AddressSanitizer and UndefinedBehaviorSanitizer; any finding fails the test |
| `-DENABLE_COVERAGE=ON` | gcov instrumentation for the coverage gate |
| `-DFLIGHT_BUILD=ON` | Flight configuration (no simulation code; builds, no tests) |

## How tests are organised

- One suite per module, `test_<module>.c`, listed in `MODULE_TESTS` in `CMakeLists.txt`. Each links
  against `flight_test_lib`, a static library of every flight source except `main.c`.
- `integration/` and `regression/` hold cross-module suites.
- `vectors/` holds data shared with other code: `beacon_filter_vectors.txt` is run by both the
  OBC filter tests here and the Jetson filter tests in `software/payload/tests`.
- `no_dynamic_memory` is a ctest check, not a CMocka suite: it fails if any flight object references
  `malloc`, `calloc`, `realloc` or `free` (`software/flight/scripts/check_no_dynamic_memory.sh`).
- Suites not yet registered are listed, with the reason, in the "Not registered" block at the end
  of `CMakeLists.txt`.

The simulated fault log, EPS configuration and mission data persist to files. `CMakeLists.txt` gives
every test its own files (`SMART_QSO_FAULT_LOG`, `SMART_QSO_EPS_CONFIG`, `SMART_QSO_MISSION_DATA`),
so parallel runs cannot interfere.

## Writing a test

Start from an existing suite, e.g. `test_beacon_format.c`: a file comment that says what is tested
and why, `static void test_x(void **cm_state)` functions, and a `CMUnitTest` array run by
`cmocka_run_group_tests`. Assert what the specification says (the requirement or design document),
not what the code happens to do. Tag a test with `@requirement <ID>` only when the ID exists in
`docs/requirements/` or `docs/software/SOFTWARE_REQUIREMENTS.md`; CI fails if undefined IDs increase
(`scripts/check_traceability.py`).

## Coverage

CI measures line coverage with gcovr over every compiled source in `src/`, so a source no test runs
counts as 0 %, and fails below `COVERAGE_MIN_LINE` in `.github/workflows/ci.yml`. To reproduce:

```bash
cmake -S . -B build-cov -DCMAKE_BUILD_TYPE=Debug -DENABLE_COVERAGE=ON
cmake --build build-cov -j && ctest --test-dir build-cov
gcovr -r . --filter 'src/' build-cov --print-summary
```

Use the `gcov` that matches the compiler (`--gcov-executable gcov-16` with Homebrew GCC 16).

## Hardware tests

These are host tests only. Hardware-in-the-loop testing is planned in `hil/HIL_TEST_PLAN.md` and
has not been run.
