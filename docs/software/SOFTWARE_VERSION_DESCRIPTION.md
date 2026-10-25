# Software Version Description
## SMART-QSO Flight Software

**Document Number:** SQSO-SVD-001
**Revision:** C
**Date:** 2026-10-24
**Reference:** NPR 7150.2 §3.6
**Supersedes:** `docs/reviews/SOFTWARE_VERSION_DESCRIPTION.md`, which also carried SQSO-SVD-001 and is merged here (finding PI-17)

---

## 1. Purpose

This Software Version Description (SVD) identifies and describes a specific version of the SMART-QSO flight software, documenting its contents, build configuration, and verification status. It also identifies the payload (Jetson) and ground software, which the superseded SVD covered.

## 2. Version Identification

### 2.1 Release Information

| Field | Value |
|-------|-------|
| Software Name | SMART-QSO Flight Software |
| Version Number | 0.2.0 (`SMART_QSO_VERSION_STRING` is "0.2.0-dev" in `software/flight/include/smart_qso.h`) |
| Version Type | Development |
| Release Date | 2026-01-02 |
| Git Tag | None. The repository has no configuration tags yet; the first is created at PDR (finding PI-04) |
| Git Commit | [commit hash] |

### 2.2 Version History

| Version | Date | Type | Description |
|---------|------|------|-------------|
| 0.1.0 | 2025-xx-xx | Development | Initial skeleton |
| 0.2.0 | 2026-01-02 | Development | CDR candidate |
| 1.0.0 | TBD | Flight | Flight release |

### 2.3 Payload and Ground Software

| Field | Payload (Jetson) | Ground |
|-------|------------------|--------|
| Software Name | SMART-QSO Payload Software | SMART-QSO Ground Software |
| Version Number | 1.0.0 (`software/payload/pyproject.toml`) | Not assigned |
| Git Tag | None (see §2.1) | None (see §2.1) |
| Source | `software/payload/` | `software/ground/` |

## 3. Software Components

### 3.1 Source Files

| Module | Files | Lines | Description |
|--------|-------|-------|-------------|
| main | main.c | ~500 | Main application |
| sensors | sensors.c/h | ~600 | Sensor management |
| eps_control | eps_control.c/h | ~400 | Power system control |
| adcs_control | adcs_control.c/h | ~700 | Attitude control |
| beacon | beacon.c/h | ~600 | Beacon generation |
| fault_mgmt | fault_mgmt.h, Rust `software/rust/sqso-ffi/src/fault_mgmt.rs` | ~400 | Fault management |
| uart_comm | uart_comm.c/h | ~400 | UART communication |
| mission_data | mission_data.c/h | ~300 | Data persistence |
| crc32 | smart_qso.h, Rust `software/rust/sqso-protocol/src/checksum.rs` | ~150 | CRC calculation |
| time_utils | time_utils.c/h | ~100 | Time utilities |
| hal | hal/*.c/h | ~500 | Hardware abstraction |
| **Total** | | **~4650** | |

This table and §10.1 describe the 0.2.0 skeleton (history). Since 2026-10-24 the flight software is Rust. §3.5 lists
the current source, and `software/rust/README.md` maps every former C file to its Rust files.

### 3.2 Test Files

| Test Module | Files | Tests | Coverage |
|-------------|-------|-------|----------|
| test_main | test_main.c | 15 | Core functions |
| test_sensors | test_sensors.c | 10 | Sensor logic |
| test_eps | test_eps.c | 12 | EPS control |
| test_crc32 | test_crc32.c | 5 | CRC validation |

The table above is the 0.2.0 skeleton (history; `test_main.c` was removed on 2026-10-16). On 2026-10-24 there are 42
CMocka suites with 631 cases, and 44 ctest entries with `no_dynamic_memory` and `Simulation_Run`, in `software/flight/tests`. They run
against the Rust. There are also 299 Rust unit tests (`sqso-protocol` 74, `sqso-fsw` 225) and 12 fuzz targets.

### 3.3 Configuration Files

| File | Purpose |
|------|---------|
| CMakeLists.txt | Build configuration |
| .clang-tidy | Static analysis rules |
| .clang-format | Code formatting |
| .cppcheck | Cppcheck configuration |

### 3.5 Flight Software Source (Rust, 2026-10-24)

| Component | Location | Lines (2026-10-24, including tests and comments) | Description |
|-----------|----------|------|-------------|
| `sqso-protocol` | `software/rust/sqso-protocol/src` | 2,291 | Wire formats (`no_std`, no `unsafe`) |
| `sqso-fsw` | `software/rust/sqso-fsw/src` | 11,313 | Flight logic of every module (`no_std`, no `unsafe`) |
| `sqso-ffi` | `software/rust/sqso-ffi/src` | 10,139 | The modules behind the C headers, the executive, the simulated HAL, storage |
| `sqso-clib`, `sqso-sim` | `software/rust/sqso-clib`, `software/rust/sqso-sim` | 33, 24 | Static library for the C suites; host simulation `smart_qso_flight` |
| firmware | `software/rust/firmware/src` | 1,553 | STM32L4R5ZI image: Embassy main task and the HAL |
| `sqso-py` | `software/rust/sqso-py/src` | 346 | Python module `sqso_protocol` (payload, ground) |
| Fuzz targets | `software/rust/fuzz/fuzz_targets` | 416 | 12 targets |
| C interface | `software/flight/include` | — | The C headers: the C ABI of the Rust modules |
| Remaining C | `software/flight/src/flight_log_fmt.c` | 67 | printf-style flight-log shim |

### 3.4 Payload and Ground Source

| Component | Files |
|-----------|-------|
| Payload | `agent.py`, `content_filter.py`, `crc16.py`, `formatter.py`, `health_interface.py`, `llama_runner.py`, `obc_link.py`, `supervisor.py`; tests in `software/payload/tests/` |
| Ground | `beacon_decoder.py`, `telemetry_display.py`, `tools/`; tests in `software/ground/tests/` |

## 4. Build Configuration

### 4.1 Compiler and Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust toolchain | 1.97.0 (`software/rust/rust-toolchain.toml`); minimum 1.88 | Flight software compiler (host and `thumbv7em-none-eabihf`), clippy, rustfmt, llvm-tools |
| GCC | 12.x | C compiler (host only: the C test suites and the flight-log shim) |
| CMake | 3.20+ | Build system (drives cargo through `software/flight/cmake/rust.cmake`) |
| clang-tidy | 18.x (CI: 18.1.3) | Static analysis (remaining C) |
| cppcheck | 2.13 (CI: 2.13.0) | Static analysis and MISRA addon (remaining C) |

### 4.2 Compiler Flags

```
-std=c11
-Wall -Wextra -Wpedantic -Werror
-Wconversion -Wshadow -Wcast-qual
-ffunction-sections -fdata-sections
-O2 (Release) / -O0 -g (Debug)
```

These flags apply to the remaining C (the shim and the test suites). The Rust release profile
(`software/rust/Cargo.toml`, `software/rust/firmware/Cargo.toml`): `opt-level = "s"`, `lto = true`,
`codegen-units = 1`, `overflow-checks = true`, `panic = "abort"`, debug assertions off.

### 4.3 Build Variants

| Variant | Purpose | Debug | Assertions |
|---------|---------|-------|------------|
| Debug | Development | Yes | Yes |
| Release | Test builds | No | Yes |
| Flight | Flight software | No | Limited |

Since SW-08, assertions are active in every build, flight included, so "Limited" no longer applies. In Rust, bounds
and overflow checks stay on in the release profile. The Flight variant (`-DFLIGHT_BUILD=ON`) builds the host simulation
with the cargo feature `flight` (run until stopped). The flight image is the firmware, built with
`cargo build --release` in `software/rust/firmware`.

### 4.4 Payload and Ground Build Environment

| Tool | Version | Source |
|------|---------|--------|
| Python | ≥ 3.9 (`requires-python` in `software/payload/pyproject.toml`); CI runs 3.12 | `.github/workflows/program-checks.yml` |
| pytest | As installed by CI | `.github/workflows/program-checks.yml` |
| llama-cpp-python | Unpinned optional extra `llm` (gated generative-text experiment, BASELINE §6 [D5]) | `software/payload/pyproject.toml` |

## 5. Dependencies

### 5.1 External Libraries

| Library | Version | License | Purpose |
|---------|---------|---------|---------|
| cmocka | 1.1.5 | Apache 2.0 | Unit testing |
| crc | 3.4.0 | MIT or Apache 2.0 | CRC catalogue (runtime) |
| hmac, sha2 | 0.13.0, 0.11.1 | MIT or Apache 2.0 | HMAC-SHA-256 (runtime) |
| zeroize | 1.9.1 | MIT or Apache 2.0 | Key material erasure (runtime) |
| ctutils | 0.4.5 | MIT or Apache 2.0 | Constant-time comparison under the HMAC check (updated from 0.4.2 for RUSTSEC-2026-0385) |
| embassy-stm32, embassy-executor, embassy-time | 0.6.0, 0.10.0, 0.5.1 | MIT or Apache 2.0 | Target bring-up, drivers, executor (firmware only) |
| cortex-m, cortex-m-rt | 0.7.10, 0.7.7 | MIT or Apache 2.0 | Cortex-M support, vector table and `Reset` (firmware only) |
| libc | 0.2.191 | MIT or Apache 2.0 | Host simulation only |

Versions are those of `software/rust/Cargo.lock` and `software/rust/firmware/Cargo.lock` (2026-10-24). CI runs
`cargo audit` on both. Licenses are as published by each crate; this table does not replace a license review.

### 5.2 Hardware Dependencies

| Component | Driver | Version |
|-----------|--------|---------|
| STM32L4xx | HAL | Mfr provided |
| SPI Flash | Custom | 1.0 |
| I2C Sensors | Custom | 1.0 |

As built (2026-10-24): the STM32L4R5ZI peripherals are driven through embassy-stm32 0.6. The non-volatile memory is an
FM25V20A FRAM on SPI1 (`software/rust/firmware/src/fram.rs`), and the telecommand receiver is a TNC on USART2 (KISS).
The ADCS sensors and actuators have no drivers yet (`docs/reviews/KNOWN_ISSUES.md` KI-001).

## 6. Verification Status

### 6.1 Test Results

| Test Suite | Tests | Pass | Fail | Skip |
|------------|-------|------|------|------|
| Flight unit tests (`ctest`) | Run by CI on every pull request (`.github/workflows/flight-software-ci.yml`); no test report is archived in the repository | - | - | - |
| Payload and ground tests (pytest) | Run by CI on every pull request (`.github/workflows/program-checks.yml`); no test report is archived | - | - | - |
| Integration | TBD | - | - | - |
| System | TBD | - | - | - |

The two SVDs merged here gave different counts (42 unit tests; "22 suites, 300+ tests"). Neither count is backed by a report in the repository, so neither is kept.

### 6.2 Static Analysis

| Tool | Findings | Resolved | Open |
|------|----------|----------|------|
| clang-tidy | 58 | - | 58 |
| cppcheck | 0 (with the CI suppression list) | - | 0 |
| cppcheck MISRA addon | 1,189 | - | 1,189 |

The open counts are CI's no-regression baselines (`software/flight/quality/`); CI allows no increase.

On 2026-10-24, after the port to Rust, the baselines are clang-tidy 0, function length 0 and MISRA addon 42 (all in
the remaining C and the headers it includes; `software/flight/docs/MISRA_DEVIATIONS.md` §3). The Rust has no clippy
warnings: CI runs clippy with `-D warnings`. `cargo audit` found RUSTSEC-2026-0385 (ctutils 0.4.2), which was fixed by
updating to 0.4.5.

### 6.3 Code Coverage

| Metric | Target | Achieved |
|--------|--------|----------|
| Statement | 80% | TBD |
| Branch | 70% | TBD |
| Function | 100% | TBD |

Measured 2026-10-24 (CI job Build and Test): line coverage of the Rust flight crates is 92.2 % (gate 90 %), and of the
flight software C and Rust together 92.2 % (gate 80 %). Line coverage is not statement coverage. Branch and function
coverage are not measured, so the table above is left as it was.

## 7. Known Issues

### 7.1 Open Issues

| ID | Severity | Description | Workaround |
|----|----------|-------------|------------|
| — | — | Open issues are listed in SQSO-KI-001 (`docs/reviews/KNOWN_ISSUES.md`) and in `docs/funding_audit/FINDINGS_REGISTER.md` | — |

### 7.2 Limitations

| Limitation | Impact | Mitigation |
|------------|--------|------------|
| Simulation only | Not flight-ready | Hardware integration TBD |
| No RTOS (FreeRTOS was never integrated and is not planned) | One Embassy task; interrupts only for the UARTs and the time driver | By design; the baseline since 2026-10-25 (BASELINE §6, CR-2026-002; `software/flight/docs/RTOS_ARCHITECTURE.md`) |
| Firmware not run on hardware | Pin map provisional; timing and power not measured; stack bounded by analysis only (4,892 B of 8 KB) | Hardware bring-up (`software/rust/README.md`, Next steps) |
| Beacon interval | 60 s ACTIVE, 120 s IDLE, 180 s SAFE (BASELINE §8) | Operating constraint |
| Beacon text | At most 176 characters (`docs/BEACON_SPEC.md`) | Operating constraint |
| Hardware watchdog | 30 s timeout (`WDT_HW_TIMEOUT_MS`) | Operating constraint |
| Jetson duty | ≤ 5 % tumbling, ≤ 10 % sun-pointed; sunlit and SOC ≥ 55 % only (BASELINE §5, §6) | Power constraint |

### 7.3 Content Not Carried from the Superseded SVD

| Item in `docs/reviews/SOFTWARE_VERSION_DESCRIPTION.md` | Reason |
|---------------------------------------------------------|--------|
| Version 1.0.0 and Git tag `v1.0.0-cdr` | No such tag exists; the code says 0.2.0-dev |
| Flash and RAM memory map (1 MB / 256 KB) | Does not match the baseline OBC, STM32L4R5ZI (BASELINE §6); see SQSO-FSW-006 (`software/flight/docs/MEMORY_POLICY.md`) |
| "5-second hardware watchdog", "200-byte beacon limit", "fixed 60-second beacon" | Conflict with the 30 s watchdog, the 176-character text limit and the per-mode beacon intervals above |
| CDR memory and CPU estimates; 78 of 90 requirements verified; unit-test totals | No analysis or test report in the repository supports them |
| FreeRTOS 10.5.1 in the build environment | No RTOS is integrated (§7.2) |
| "No deviations" | MISRA deviations are logged in SQSO-FSW-010 (`software/flight/docs/MISRA_DEVIATIONS.md`) |

## 8. Change Summary

### 8.1 Changes from Previous Version

| Category | Changes |
|----------|---------|
| New Features | ADCS B-dot control, CRC32 protection |
| Bug Fixes | None (new code) |
| Enhancements | Modular architecture |
| Documentation | Complete design docs |

2026-10-24 (still 0.2.0-dev): the flight software was ported from C to Rust. The behaviour is the same, apart from the
deliberate changes numbered in `software/rust/README.md`. The STM32L4R5ZI image is now the Embassy firmware.

### 8.2 Requirements Implemented

| Requirement | Status | Test |
|-------------|--------|------|
| SRS-001 | Complete | test_eps_control |
| SRS-002 | Complete | test_fault_mgmt |
| SRS-003 | Complete | test_sensors |
| SRS-004 | Complete | test_beacon |
| SRS-005 | Complete | test_adcs |

## 9. Installation

### 9.1 Build Instructions

```bash
# Clone repository
git clone [repo-url]
cd SMART-QSO

# Build flight software
cmake -S software/flight -B build
cmake --build build

# Run tests
ctest --test-dir build

# Payload and ground tests
python -m pytest -q software/payload/tests
python -m pytest -q software/ground/tests
```

### 9.2 Deployment

| Target | Method |
|--------|--------|
| Simulation | Direct execution |
| Hardware | JTAG/SWD flash |

The simulation is `build/smart_qso_flight`. On hardware, `cargo run --release` in `software/rust/firmware` flashes the
image over SWD with probe-rs.

## 10. Configuration Items

### 10.1 Files in This Release

The tree below is the 0.2.0 skeleton (history). The current flight software source is in §3.5.

```
software/flight/
├── CMakeLists.txt
├── include/
│   ├── smart_qso.h
│   ├── sensors.h
│   ├── eps_control.h
│   ├── adcs_control.h
│   ├── beacon.h
│   ├── fault_mgmt.h
│   ├── uart_comm.h
│   ├── mission_data.h
│   ├── crc32.h
│   ├── time_utils.h
│   └── hal/
├── src/
│   ├── main.c
│   ├── sensors.c
│   ├── eps_control.c
│   ├── adcs_control.c
│   ├── beacon.c
│   ├── fault_mgmt.rs (Rust)
│   ├── uart_comm.c
│   ├── mission_data.c
│   ├── checksum.rs (Rust)
│   ├── time_utils.c
│   └── hal/
├── tests/
│   └── test_main.c
└── docs/
    ├── RADIATION_MITIGATION.md
    ├── BOOTLOADER_REQUIREMENTS.md
    ├── AUTONOMY_DESIGN.md
    ├── FDIR_DESIGN.md
    └── TIME_MANAGEMENT.md
```

## 11. Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Software Lead | | | |
| Systems Engineer | | | |
| Quality Assurance | | | |
| Project Manager | | | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | Merged with `docs/reviews/SOFTWARE_VERSION_DESCRIPTION.md`, which carried the same number (finding PI-17): payload and ground identification, build environment and limitations added; Git tag corrected to none; unsupported test counts and open-issue claim removed; §7.3 lists what was not carried and why | SMART-QSO Team |
| C | 2026-10-24 | The flight software is Rust: §3.5 current source by crate; tests, tools, Rust release profile, runtime dependencies with versions, hardware drivers, static-analysis baselines, measured line coverage, limitations, change summary and deployment updated. Skeleton tables kept as history | SMART-QSO Team |
| C, note | 2026-10-25 | §7.2: no RTOS (Embassy executor) is the baseline (BASELINE §6, CR-2026-002) | SMART-QSO Team |
