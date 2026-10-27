# SMART-QSO Flight Software Delivery Package

> **Funding audit (2026-10-16, SW-18): no flight software has been released or delivered.** This revision replaces the
> earlier status claims (a v1.0.0 tag, "no known issues", "> 80 % line coverage", "MISRA C:2012 compliant") with facts
> that can be checked in the repository. Sections 3, 5.3, 7 and 9 describe what a future delivery must contain; nothing
> in them exists yet.
>
> **Update 2026-10-24 (version 1.2):** all flight software is Rust (`software/rust/README.md`). The only C left is the
> flight-log printf shim. §2.1, §2.3, §4.1, §5 and §6.2 now describe that state. §4.3 gives the figures of 2026-10-24.
> §4.2 keeps the figures of 2026-10-16 as the record of that date. Still no release and no delivery.
>
> **Update 2026-10-27 (version 1.3):** §4.4 gives the figures recorded by the CDR 2026 design review
> ([CDR_2026_DESIGN_REVIEW.md](CDR/CDR_2026_DESIGN_REVIEW.md) §7.1); §6 matches `KNOWN_ISSUES.md` Version 1.4. Still no
> release and no delivery. Index of the review documents: [README.md](README.md).

**Document ID**: SQSO-FSW-DEL-001
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Template and status record. No release, no delivery (project phase: Phase B, pre-PDR; `docs/BASELINE_PARAMETERS.md` §1)

## 1. Purpose

This document will be the flight software delivery package for the SMART-QSO mission (NPR 7150.2 version description
and delivery record). Until a release is made, it records the current state of the flight software and what a delivery
must contain. The flight software is NPR 7150.2D Class C (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1, 2026-10-25).

## 2. Software Version Description

### 2.1 Flight Software (OBC)

| Item | Value |
|------|-------|
| Name | SMART-QSO Flight Software |
| Version | **None released.** The software is identified only by Git commit |
| Git tag | **None.** The repository has no tags and no GitHub releases (checked 2026-10-16 with `git tag` and the GitHub API) |
| Git SHA | The commit being described; no release commit exists |
| Host build | `smart_qso_flight`, a host simulation executable (Linux/macOS; crate `sqso-sim`), using the simulated HAL `software/rust/sqso-ffi/src/hal_sim.rs` |
| Target build | The Embassy firmware `software/rust/firmware` (ELF `sqso-firmware`), linked for the STM32L4R5ZI and built and checked in CI. It has not run on hardware. (Until 2026-10-24 this was a C static library with a register-level HAL; it was removed) |
| Binary size / checksum | Release image (2026-10-27): text 207,284 B, data 26,400 B, bss 17,052 B (`software/flight/docs/MEMORY_POLICY.md` Rev H §3.2; 2026-10-25: text 115,876 B, data 19,764 B, bss 16,496 B). No checksum is recorded because nothing is released. CI prints the section sizes on each run |

### 2.2 Payload Software (Jetson)

| Item | Value |
|------|-------|
| Name | SMART-QSO Payload Software (`software/payload`, Python package `smart-qso-payload`) |
| Version | None released. `pyproject.toml` carries the placeholder version 1.0.0, which is not a release |
| Git tag | None |
| Tests | `python -m pytest -q software/payload/tests` in the "Program Checks" workflow (Python 3.12 in CI; the package declares Python ≥ 3.9) |

### 2.3 Build Environment

| Component | Version / source |
|-----------|------------------|
| Host compiler | The runner's default GCC on `ubuntu-24.04` in CI (images pinned in the workflows); Apple clang works for local builds |
| Compiler (OBC target and host flight software) | Rust 1.97.0, pinned by `software/rust/rust-toolchain.toml`, target `thumbv7em-none-eabihf`. Minimum supported 1.88 (checked in CI). No C cross compiler is used (2026-10-24; the former `gcc-arm-none-eabi` toolchain file was removed). `binutils-arm-none-eabi` is installed in CI for image inspection only |
| CMake | ≥ 3.16 (`cmake_minimum_required` in `software/flight/CMakeLists.txt`) |
| Unit-test framework | CMocka (`libcmocka-dev` in CI) |
| Static analysis | cppcheck 2.13.x and clang-tidy 18.x, pinned by the CI version checks |
| Coverage | gcovr (Ubuntu 24.04 package; 7.0 when the figure below was recorded) |
| RTOS | **None.** The image runs on the Embassy executor (embassy-executor 0.10) with one task. FreeRTOS was never integrated; no RTOS is the baseline since 2026-10-25 (BASELINE §6, CR-2026-002; `software/flight/docs/RTOS_ARCHITECTURE.md`) |
| Vendor HAL / CMSIS | **None.** embassy-stm32 0.6 (with its generated register access); the flight HAL is `software/rust/firmware/src` |
| Payload | Python ≥ 3.9 |

## 3. Delivery Contents (planned)

Nothing in this section exists yet. A delivery will contain the following once there is a linked target image.

### 3.1 Flight Software Package

```
smart-qso-fsw-<version>/
├── bin/
│   ├── smart_qso_flight.elf     # Flight binary
│   ├── smart_qso_flight.bin     # Raw binary for flash
│   ├── smart_qso_flight.hex     # Intel HEX format
│   └── checksums.sha256         # Binary checksums
├── symbols/
│   ├── smart_qso_flight.map     # Linker map file
│   └── smart_qso_flight.sym     # Symbol table
├── source/
│   └── smart-qso-fsw-<version>.tar.gz  # Source archive
├── docs/
│   ├── RELEASE_NOTES.md
│   ├── KNOWN_ISSUES.md
│   └── BUILD_INSTRUCTIONS.md
└── config/
    └── sensors.yaml             # Default configuration
```

### 3.2 Payload Software Package

```
smart-qso-payload-<version>/
├── software/
│   └── payload/                 # Python package
├── models/
│   └── README.md               # Model placement instructions
├── config/
│   └── payload.yaml            # Configuration
└── docs/
    ├── RELEASE_NOTES.md
    └── DEPLOYMENT.md
```

## 4. Current Status (no release)

### 4.1 What exists

- Flight modules for the power-aware state machine (SAFE, IDLE, ACTIVE), telemetry and AX.25 beacon formatting, the
  beacon content filter, the OBC side of the Jetson link, the supervised watchdog, fault logging, telecommand
  authentication (HMAC-SHA-256), TX-OFF, post-deployment RF silence, the loss-of-contact timer and deployment
  sequencing. They run in the host simulation.
- A register-level STM32L4 HAL that compiles into the target library. It has not run on hardware. (2026-10-16. Since
  2026-10-24, the flight software is Rust and the target is the linked Embassy firmware with Rust drivers for GPIO,
  timer, IWDG, UARTs, I2C, SPI, ADC, FRAM and the receiver. It has not run on hardware either.)
- Graceful degradation without the AI payload is designed in (template beacons); it has been exercised only in the
  host simulation and unit tests.

### 4.2 Test and analysis summary

Every figure below comes from a CI configuration or a file in the repository, or was reproduced on 2026-10-16.

| Item | Value | Evidence |
|------|-------|----------|
| Unit tests | **31 CMocka suites, 428 test cases**, all passing, plus the `no_dynamic_memory` check (JPL rule 3): 32 CTest entries | `software/flight/tests/CMakeLists.txt`; reproduced locally on 2026-10-16 with `cmake -S software/flight -B <dir> && cmake --build <dir> && ctest --test-dir <dir>` (macOS, Apple clang, CMocka) |
| Tests not run | Six Unity suites (`test_adcs`, `test_beacon`, `test_scheduler`, `test_system_init`, `test_telemetry`, `test_telemetry_flow`), the integration suites and the regression suite are not built (finding SW-09) | Comment block in `software/flight/tests/CMakeLists.txt` |
| Line coverage | **68.2 %** over every compiled source in `software/flight/src` (gcovr 7.0, 2026-10-16). CI fails below **65 %** (`COVERAGE_MIN_LINE`). Branch and MC/DC coverage are not measured | `.github/workflows/ci.yml`, job "Build and Test", step "Coverage gate" |
| Static analysis | Ratchets that block new findings; they do not show compliance. clang-tidy: **58** findings; MISRA C:2012 (cppcheck addon, not a qualified MISRA tool): **1,189** findings; functions over 60 lines: **0** | `software/flight/quality/clang-tidy.baseline`, `misra.baseline`, `function-length.baseline`; `.github/workflows/ci.yml` jobs "Static Analysis" and "MISRA C:2012 Check" |
| Sanitizers | Address and undefined-behaviour sanitizers on the unit tests and on about 20 s of simulated operation | `.github/workflows/flight-software-ci.yml`, job "Sanitizer Checks" |
| Target build | The flight library cross-compiles for Cortex-M4 with `-Werror`; CI checks that key symbols are present | `.github/workflows/flight-software-ci.yml`, job "STM32L4 Cross-Compile (library)" |
| Integration / hardware tests | **None run.** No hardware exists | — |

### 4.3 Test and analysis summary, 2026-10-24

| Item | Value | Evidence |
|------|-------|----------|
| Unit tests | **45 CMocka suites, 695 test cases**, plus `no_dynamic_memory`, the `Simulation_Run` host simulation the two `Executive_Jetson` runs of the main loop against a simulated Jetson and the two `Downlink_TNC` runs of the downlink end to end: 49 ctest entries, run unchanged against the Rust. In addition, 358 Rust unit tests | `software/flight/tests/CMakeLists.txt`; `software/flight/tests/README.md` |
| Line coverage | Rust flight crates **94.8 %** (CI fails below 90 %); C and Rust together **94.8 %** (CI fails below 80 %). Branch and condition coverage **81.1 %** (CI fails below 80 %); the safety-critical files 100 % of lines and conditions, gated per file (CR-2026-003, in place of MC/DC) | `.github/workflows/ci.yml`, job "Build and Test" |
| Static analysis | clippy with warnings as errors (host and Cortex-M4F); clang-tidy **0**, MISRA addon **42**, functions over 60 lines **0** in the remaining C; CodeQL (Rust, C/C++); `cargo audit` (RUSTSEC-2026-0385 fixed by ctutils 0.4.5) | `software/flight/quality/`; jobs "Static Analysis", "MISRA C:2012 Check", "Security Scan" |
| Fuzzing | 12 targets, 30 s each per CI run | Job "Sanitizer Checks" |
| Equivalence with the former C | Simulation output identical (256 lines); differential tests on 8.3 million protocol inputs and on sensors, GNSS, safe strings, flight log and assertion handler | `software/rust/README.md`, section Evidence |
| Target build | Image linked; CI checks the entry point, vector table, initial SP, flight symbols, and no heap or C library | Job "STM32L4 Cross-Compile (library)" |
| Integration / hardware tests | **None run.** No hardware exists | — |

### 4.4 Test and analysis summary, 2026-10-27 (CDR 2026 design review)

| Item | Value | Evidence |
|------|-------|----------|
| Tests | 622 Rust tests and 56 C suites pass; the safety-critical coverage gate (32 files, 100 % of lines and conditions) and the image and stack checks pass | `docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md` §7.1 |
| Mission simulation | SQSO-SIL-001 G: 730 days, 0 anomalies in 50 checks | Same; `docs/test/SIL_MISSION_SIMULATION.md` |
| Integration / hardware tests | **None run.** No hardware exists; the firmware has not run on hardware and its pin map is provisional | Same, §7.2 |

## 5. Build Instructions

### 5.1 Prerequisites

```bash
# Host build and tests (Ubuntu 24.04); Rust from rustup (https://rustup.rs);
# the pinned toolchain installs on first use (software/rust/rust-toolchain.toml)
sudo apt install cmake libcmocka-dev

git clone https://github.com/cheesejaguar/SMART-QSO.git
cd SMART-QSO
# No release tag exists; build the commit you intend to describe
```

### 5.2 Build Procedure

```bash
# Host simulation and unit tests
cmake -S software/flight -B build
cmake --build build
ctest --test-dir build

# STM32L4R5ZI image (Embassy firmware)
cd software/rust/firmware
cargo build --release --locked
# target/thumbv7em-none-eabihf/release/sqso-firmware
```

### 5.3 Reproducible Build Verification (planned)

Applies once a linked image exists: build twice from a clean tree and compare the SHA-256 of the image.

## 6. Known Issues

### 6.1 Open Issues

This list matches `docs/reviews/KNOWN_ISSUES.md` Version 1.4 (5 issues: three accepted, KI-002 resolved in code and
open until verified on hardware, KI-003 open; no acceptance has been signed).

| ID | Severity | Status | Description | Workaround (from KNOWN_ISSUES) |
|----|----------|--------|-------------|--------------------------------|
| KI-001 | Medium | Accepted | Firmware not run on hardware; the sensor table is simulated on the target | None needed before hardware |
| KI-002 | Medium | Resolved in code | Fault log persisted in FRAM; not yet verified on hardware | Fault counts in each beacon; boot reason preserved |
| KI-003 | Low | Open | SET_BEACON is accepted but has no effect; intervals fixed per mode (60 / 120 / 180 s) | Baseline intervals per mode |
| KI-004 | Low | Accepted | AI model loaded when the payload agent starts; no hot-swap | None needed: the Jetson boots at every burst |
| KI-005 | Low | Accepted | Beacon decoder tested only on synthetic data, with no TNC | KISS standard |

### 6.2 Other open limitations (not yet in KNOWN_ISSUES)

| Limitation | Reference |
|------------|-----------|
| No RTOS, startup code or linker script; no linked flight image (2026-10-16; resolved 2026-10-24: linked Embassy image, no RTOS by design) | `software/rust/firmware` |
| Unity, integration and regression suites not built (2026-10-16; all are built and run on 2026-10-24) | SW-09 |
| Static-analysis findings not yet worked down (58 clang-tidy, 1,189 MISRA on 2026-10-16; on 2026-10-24, 0 and 42 in the remaining C) | `software/flight/quality/` |
| Firmware not run on hardware; pin map provisional (SQSO-ICX-001 allocation); sensor table simulated (ADCS drivers written since the CDR 2026 review) | `docs/reviews/KNOWN_ISSUES.md` KI-001 |
| No hardware-in-the-loop or flatsat testing | `software/flight/tests/hil/HIL_TEST_PLAN.md` |

### 6.3 Accepted Deviations

| ID | Requirement | Deviation | Rationale | Approval |
|----|-------------|-----------|-----------|----------|
| - | - | None requested | - | - |

## 7. Memory Map

Not defined. There is no linker script, so no flash or RAM layout exists. The flight software allocates no dynamic
memory (`no_dynamic_memory` check), so the layout will need no heap. The layout will be defined with the linker script
and recorded here.

Update 2026-10-24: the firmware links with cortex-m-rt's `link.x` and the `memory.x` of embassy-stm32: 2 MB flash from
0x08000000 (vector table first) and 640 KB RAM from 0x20000000, with the stack growing down from 0x200A0000, and no
heap (`software/flight/docs/MEMORY_POLICY.md` §3).

Update 2026-10-27: the firmware's own `memory.x` links SRAM2 alone (64 KB from 0x20030000, parity): statics at its
start, the 8 KB stack at its top growing down from 0x20040000, no heap; SRAM1 and SRAM3 are not linked
(`software/flight/docs/MEMORY_POLICY.md` Rev H §3.1).

## 8. Resource Utilization

### 8.1 Memory Usage

From `software/flight/docs/MEMORY_POLICY.md` Rev H §3.2 (release image, 2026-10-27):

| Resource | Allocated | Used | Margin |
|----------|-----------|------|--------|
| Flash (code and constants) | 256 KB (SRS-C004; 2 MB device) | 207,284 B | 21 % of SRS-C004 |
| RAM (static) | 57,312 B (64 KB SRAM2 less the stack and its guard) | 43,452 B (data 26,400 B, bss 17,052 B) | 24 % |
| RAM (stack) | 8 KB (SRS-C003) | 6,332 B worst case, static analysis (MEMORY_POLICY §6.2) | 23 % |

CI prints the section sizes on every run.

### 8.2 CPU Utilization

Not measured: the software has not run on the target. Planned task periods are in
`software/flight/docs/TIMING_ANALYSIS.md` (estimates).

## 9. Upload Procedure

Not designed. There is no bootloader or software-update mechanism yet; requirements are in
`software/flight/docs/BOOTLOADER_REQUIREMENTS.md`. Any upload would be adopted only by authenticated telecommand
(47 CFR 97.211).

## 10. Delivery Verification

### 10.1 Delivery Checklist

| Item | Done | Date | By |
|------|------|------|-----|
| Source code tagged | [ ] | | |
| Binary built | [ ] | | |
| Checksum recorded | [ ] | | |
| Symbols archived | [ ] | | |
| Documentation finished | [ ] | | |
| Known issues documented | [ ] | | |
| Release notes written | [ ] | | |
| Build reproducible | [ ] | | |

### 10.2 Acceptance Signatures

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Software Lead | | | |
| Quality Assurance | | | |
| Systems Engineer | | | |
| Project Manager | | | |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | SMART-QSO Team | Funding audit SW-18: status sections rewritten from repository facts (no tag, no linked image, 31 suites / 428 cases, 68.2 % line coverage, static-analysis baselines, KI-001 to KI-005); invented memory map, FreeRTOS/CMSIS versions and upload procedure removed |
| 1.2 | 2026-10-24 | SMART-QSO Team | All flight software Rust: host and target builds, build environment (Rust toolchain, Embassy, no RTOS), §4.3 figures of 2026-10-24, build instructions for the firmware, limitations updated. The 2026-10-16 figures kept in §4.2. Still no release |
| 1.2, note | 2026-10-25 | SMART-QSO Team | §1: software classification NPR 7150.2D Class C (SDP §3.1); §2 RTOS row: no RTOS is the baseline (BASELINE §6, CR-2026-002) |
| 1.3 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): §4.4 figures of the CDR 2026 design review (622 Rust tests, 56 C suites, SIL G); §6.1 known issues to KNOWN_ISSUES 1.4 with their statuses (was "all Accepted" and the stale descriptions, and a note that KNOWN_ISSUES needed an update, which it has had); §6.2 ADCS drivers now written; §2.1, §7 and §8.1 memory figures and map from MEMORY_POLICY Rev H (the table was empty; SRAM2-only link) |
