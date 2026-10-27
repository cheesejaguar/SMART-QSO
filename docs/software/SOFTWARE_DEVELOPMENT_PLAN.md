# Software Development Plan (SDP)

**Document Number**: SMART-QSO-SDP-001
**Version**: 1.3
**Date**: 2026-10-27
**Status**: Draft

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements
- NPR 7150.2D - NASA Software Engineering Requirements (Version 1.1 said Rev B)

**Process Heritage**: NASA Ames Research Center Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

---

## 1. Introduction

### 1.1 Purpose

This Software Development Plan (SDP) defines the technical and management approach for developing the SMART-QSO flight software. It establishes processes, standards, and controls to ensure software quality and mission success.

### 1.2 Scope

This plan covers:
- On-Board Computer (OBC) flight software
- Jetson AI payload software
- Ground support software
- Simulation and test software

### 1.3 Document Organization

| Section | Content |
|---------|---------|
| 2 | Project Overview |
| 3 | Software Classification |
| 4 | Lifecycle Model |
| 5 | Development Environment |
| 6 | Coding Standards |
| 7 | Configuration Management |
| 8 | Testing Strategy |
| 9 | Verification & Validation |
| 10 | Risk Management |

---

## 2. Project Overview

### 2.1 Mission Description

SMART-QSO is a 1U CubeSat experimenting with onboard agentic AI to manage an amateur radio transponder. The satellite uses TinyML/TinyLM models to learn power availability, predict regional demand, prioritize QSOs fairly, and compose dynamic beacons.

### 2.2 Software Architecture Overview

```
+------------------+     UART     +------------------+
|   OBC (MCU)      |<------------>|   Jetson Orin    |
|   Rust, Embassy  |              |   AI Payload     |
+------------------+              +------------------+
        |                                 |
        v                                 v
+------------------+              +------------------+
|   EPS Control    |              |   TinyLM Model   |
|   ADCS Control   |              |   Beacon Gen     |
|   RF Beacon TX   |              +------------------+
+------------------+
```

### 2.3 Software Components

| Component | Language | Target | Classification |
|-----------|----------|--------|----------------|
| OBC Flight Software | Rust (one C11 file, the flight-log shim) | STM32L4R5ZI | Class C |
| Jetson Payload SW | Python | Jetson Orin Nano | Class C |
| Ground Software, flight operations (telecommand, keys, beacon decoding, telemetry display, pass prediction) | Python | Linux/Windows | Class C |
| Ground Software, research and outreach (`software/ground/tools/fl_update_builder.py`) | Python | Linux/Windows | Class D |
| Simulation and test: the flight software's host simulation (`software/rust/sqso-sim`) and test suites | Rust, C | Linux | Class C |
| Simulation: analysis notebooks and the orbital-lifetime cross-check (`software/simulation`) | Python | Linux | Class D |

Version 1.1 (2026-10-24): the OBC flight software was ported from C to Rust (`software/rust/README.md`). The Jetson
payload and ground tools use the same wire-format code through the Python module `sqso_protocol`
(`software/rust/sqso-py`).

---

## 3. Software Classification

### 3.1 Classification per NPR 7150.2D

Determined 2026-10-25 against NPR 7150.2D Appendix D. SWE-020 asks for each system and subsystem containing software
to be classified at the highest class that applies. This replaces the "Class D" of Version 1.1, and the B, C/D and D
stated in other documents (findings PI-18 and VV-12 in `docs/funding_audit/FINDINGS_REGISTER.md`). SMART-QSO is not a
NASA project and applies NPR 7150.2D as a tailored reference, so this is the project's own determination; no NASA
technical authority has reviewed it.

| Software | Class | Basis |
|----------|-------|-------|
| OBC flight software: `software/rust` and, in `software/flight`, the C headers, the flight-log shim and the build | **C** | CubeSat flight software on a proposed Class D payload (below) |
| Jetson payload flight software (`software/payload`) | **C** | As for the OBC |
| Ground software used in flight operations: telecommand builder, key provisioning, command dictionary (`software/ground/tools`); beacon decoding and telemetry display (`software/ground`) when used in operations; pass prediction for planning contacts | **C** | Software for space flight operations not covered by Class A or B |
| Software that tests or verifies the flight software: the C test suites, the Rust tests, the host simulation `sqso-sim`, the CI analysis scripts in `scripts/` | **C** | Treated as the software it verifies (project choice; to confirm in the Appendix C mapping) |
| Research and analysis only: `software/ground/tools/fl_update_builder.py` (ground-side research; community model updates are not in the flight baseline), the notebooks in `software/simulation`, the orbital-lifetime cross-check (NASA DAS is the tool of record), displays for public outreach | **D** | Secondary data analysis and research; no use in flight or in operations |

Rationale, from the Appendix D definitions:
- SMART-QSO is a proposed Class D payload under NPR 8705.4B (`docs/BASELINE_PARAMETERS.md` §1). Appendix D lists in
  Class C "CubeSat mission software; SmallSat mission software" and "all software on NASA Class D payloads, as defined
  in NPR 8705.4".
- Class B explicitly excludes software for NASA Class D payloads, so Class B does not apply.
- Class D explicitly excludes flight software, so the OBC and Jetson software cannot be Class D, as Version 1.1 had it.
- Ground software used to operate the spacecraft (build and authenticate telecommands, load keys, decode beacons during
  passes) falls under Class C as "software for space flight operations not covered by Class A or B".
- Tools used only for secondary analysis or research do none of these things, so they are Class D.
- The fail-graceful design, the absence of crew and the small budget do not change the class. They are reasons for
  tailoring, which the Appendix C mapping records.

The mission's risk class is unchanged: a proposed Class D payload (NPR 8705.4B). The software class does not decide
whether software is safety-critical (§3.2).

The NPR 7150.2D Appendix C requirements mapping and compliance matrix for Class C is
`docs/software/NPR7150_COMPLIANCE_MATRIX.md` (SQSO-FSW-012, 2026-10-27): each SWE requirement with its status,
evidence and tailoring, and open items C-1 to C-6. §3.3 below keeps the summary of the tailored items.

### 3.2 Safety Criticality (NASA-STD-8739.8)

Safety criticality is a separate determination from the class (SWE-205). By NASA-STD-8739.8B §4.2, software is
safety-critical if, traceable to a hazard analysis, it causes or contributes to a system hazard, controls functions
identified in one, provides mitigation for one, mitigates damage, or detects, reports and takes corrective action when
the system reaches a potentially hazardous state. Version 1.1 called the software "not safety-critical" and cited
NASA-STD-8719.13 (cancelled in 2020); that statement did not meet these criteria.

**Determination (2026-10-25).** The Software Hazard Analysis (SQSO-SHA-001) lists software controls for four hazards
of Critical (2) severity, and the system hazard list includes RF interference (SH-008), whose control is the software
transmit inhibit. The components that implement those controls are safety-critical; the rest of the flight software is
mission-critical, not safety-critical.

| Safety-critical component | Hazard | Rust (`software/rust`) | Line / condition coverage, measured (2026-10-25); exempt lines |
|---------------------------|--------|------------------------|-----------------------------------------|
| EPS control, battery protection | SW-HAZ-001 | `sqso-fsw/src/eps.rs`, `sqso-ffi/src/eps_control.rs` | 100 % / 100 %; 99.5 % / 96.7 %, 2 exempt |
| Watchdog supervision | SW-HAZ-004 | `sqso-fsw/src/watchdog.rs`, `sqso-ffi/src/watchdog_mgr.rs`, `firmware/src/wdt.rs` | 100 % / 100 %; 99.3 % / 96.9 %, 2 exempt; target only |
| Deployment timer and sequence | SW-HAZ-007 | `sqso-fsw/src/deployment.rs`, `sqso-ffi/src/deployment.rs` | 99.7 % / 100 %, 1 exempt; 100 % / 100 % |
| Safe-mode entry, fault response | SW-HAZ-008 | `sqso-fsw/src/mode.rs`, `sqso-ffi/src/state_machine.rs`, `sqso-fsw/src/fault_log.rs`, `sqso-ffi/src/fault_mgmt.rs`, `sqso-fsw/src/assert_handler.rs`, `sqso-ffi/src/assert_handler.rs` | 100 % / 100 %; 100 % / 100 %; 98.8 % / 78.6 %, 4 exempt; 100 % / 100 %; 99.1 % / 95.0 %, 2 exempt; 100 % / 93.8 %, 1 exempt |
| Payload thermal and over-current protection | Safety Data Package (SQSO-SDP-001) §3.2, thermal protection | `sqso-fsw/src/payload_guard.rs`, `sqso-ffi/src/payload_guard.rs` | 100 % / 100 %; 100 % / no conditions |
| RF transmit inhibit (TX-OFF latch, post-deployment silence, loss of contact, separation switch) | SH-008 | `sqso-fsw/src/rf_control.rs`, `sqso-fsw/src/rf_silence.rs`, `sqso-fsw/src/contact_monitor.rs` and their `sqso-ffi` wrappers, `sqso-ffi/src/separation_switch.rs` | 100 % / 100 % each; wrappers 100 % / 100 % (`contact_monitor` and `separation_switch` have no conditions) |
| Downlink gate (the same inhibits checked again when a frame is handed to the TNC; key-down and duty limits; transmitter protection), added 2026-10-25 | SH-008 | `sqso-fsw/src/downlink.rs`, `sqso-ffi/src/downlink.rs` | 100 % / 100 %; 100 % / 100 % |

Coverage is from the C suites and the Rust unit tests together, measured with `-Zcoverage-options=condition` on the
pinned nightly (`software/flight/scripts/rust_coverage.sh`); "condition" counts each condition of a decision taken both
ways. An exempt line (`// COVERAGE-EXEMPT: <reason>`, 12 in all, each a defensive arm or an error path no test can
reach) counts as covered, with its conditions, so every file meets the 100 % / 100 % of CR-2026-003 and the gate
(`software/flight/scripts/safety_coverage.py`) passes with no problem. The PI concurred with this list on 2026-10-25. RF inhibit has no SW-HAZ entry of its own yet; adding one (with its controls and their verification) is an open
action of the hazard analysis. The list agrees with the Safety Data Package (SQSO-SDP-001) §3.2. The determination
was made with the PI, as SWE-205 asks; the safety lead's review is still to come when one is named.

**What it requires.** SWE-219 (NPR 7150.2D §3.7.4) asks for 100 % code coverage by the MC/DC criterion for every
safety-critical component, at every class. rustc cannot measure MC/DC (its MC/DC instrumentation was removed; the
nearest measure it offers is condition coverage). By change request CR-2026-003 (`docs/project/CHANGE_REQUEST_LOG.md`,
approved by the PI on 2026-10-25), the requirement for the components above is 100 % line and 100 % condition
coverage, gated per file in CI (`software/flight/scripts/safety_coverage.py`). A line no test can reach (defensive code)
counts only if it is marked `// COVERAGE-EXEMPT: <reason>`, which review checks.

### 3.3 Tailoring Rationale

Per NPR 7150.2D Appendix C, for Class C (§3.1); the mapping of every Class C requirement is still to be written
(`docs/funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md`, row NPR 7150.2; now written: `docs/software/NPR7150_COMPLIANCE_MATRIX.md`):

| Requirement | As applied | Rationale |
|-------------|------------|-----------|
| Peer reviews and inspections | Reviewed pull requests; CI gates | Small team |
| Code coverage | Gated in CI: Rust lines ≥ 90 %, C and Rust lines ≥ 80 %, Rust branches ≥ 80 % (81.1 % on 2026-10-25) | Measured on every change (`.github/workflows/ci.yml`) |
| MC/DC for safety-critical components (SWE-219) | 100 % line and condition coverage instead, gated per file (CR-2026-003, approved) | rustc cannot measure MC/DC (§3.2) |
| Independent V&V | Team V&V | No IV&V designation (not a NASA project) |
| Tool qualification | Pinned, documented toolchain (Rust 1.97.0), not qualified | PI decision 2026-10-25 (CR-2026-004): no qualified compiler; the tests, image checks, stack analysis and hardware tests check the compiler's output |

---

## 4. Lifecycle Model

### 4.1 Development Lifecycle

SMART-QSO follows an incremental development model with the following phases:

```
Phase A ──> Phase B ──> Phase C ──> Phase D ──> Phase E
(Concept)   (Prelim)   (Final     (System    (Operations)
            (Design)    Design)    Assembly)

           SRR       PDR        CDR       TRR/FRR
            │         │          │          │
            ▼         ▼          ▼          ▼
```

### 4.2 Phase Activities

#### Phase B (Preliminary Design)
- Define software requirements
- Develop preliminary architecture
- Prototype critical algorithms
- Establish development environment

#### Phase C (Final Design & Implementation)
- Complete detailed design
- Implement all modules
- Unit testing
- Integration testing

#### Phase D (System Assembly & Test)
- System integration
- Environmental testing
- Flight software verification
- Configuration audit

### 4.3 Milestones and Reviews

| Review | Entry Criteria | Exit Criteria |
|--------|----------------|---------------|
| SRR | ConOps complete | Requirements approved |
| PDR | Requirements baselined | Design approach approved |
| CDR | Detailed design complete | Build/test authorized |
| TRR | Testing complete | Test readiness confirmed |
| FRR | Verification complete | Flight readiness confirmed |

---

## 5. Development Environment

### 5.1 Hardware Platforms

| Platform | Purpose |
|----------|---------|
| Development PC | Code development, simulation |
| STM32L4R5ZI development board | MCU firmware development (no hardware has run the firmware yet) |
| Jetson Orin Nano Dev Kit | AI payload development |
| Flatsat | System integration testing |

### 5.2 Software Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust toolchain (rustc, cargo, clippy, rustfmt, llvm-tools) | 1.97.0, pinned by `software/rust/rust-toolchain.toml`; minimum 1.88 | Flight software compilation (host and `thumbv7em-none-eabihf`), lints, formatting, coverage |
| Embassy (embassy-stm32, embassy-executor, embassy-time) | 0.6, 0.10, 0.5 | Target bring-up, drivers, executor (`software/rust/firmware/Cargo.toml`) |
| Rust nightly, cargo-fuzz | latest at CI run | Fuzzing (`software/rust/fuzz`) |
| cargo-audit | latest at CI run | Known vulnerabilities in the Rust dependencies |
| probe-rs | - | Flashing and running the image on a board |
| GNU Arm binutils | Ubuntu 24.04 package | Image inspection in CI (no C cross-compiler is used) |
| Host C compiler (GCC or Clang) | Ubuntu 24.04 | The C test suites and the flight-log shim |
| CMake | 3.16+ | Build system (drives cargo) |
| CMocka | 1.1+ | Unit testing framework (the C suites) |
| clang-tidy | 18.x (pinned in CI) | Static analysis of the remaining C |
| cppcheck | 2.13.x (pinned in CI) | Static analysis and the MISRA addon, remaining C |
| gcovr | Ubuntu 24.04 | C coverage (the Rust uses llvm-cov) |
| Git | 2.x | Version control |
| GitHub Actions | - | CI/CD pipeline |

### 5.3 Target Platforms

| Target | Processor | OS | Description |
|--------|-----------|----|----|
| OBC Flight | STM32L4R5ZI | None (Embassy executor, one task) | Primary flight computer |
| AI Payload | Jetson Orin Nano | Linux | AI inference engine |
| Ground | x86/ARM | Linux | Ground station software |

---

## 6. Coding Standards

### 6.0 Rust (OBC Flight Software)

The Rust coding standard is `docs/development/CODING_STANDARDS.md` §11: workspace lints with warnings as errors, no
panics on input, overflow checks in flight builds, `unsafe` only in `sqso-ffi` and the firmware with SAFETY comments,
no allocation, and the C ABI rules. The JPL Power of Ten rules apply as mapped in
`software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md` §2.1.

### 6.1 C Language (the remaining C: flight-log shim and test suites)

The following coding standards apply:

1. **JPL Institutional Coding Standard** (Power of Ten Rules)
2. **MISRA C:2012** Guidelines for critical sections (the shim; `software/flight/docs/MISRA_DEVIATIONS.md`)
3. **CERT C** Secure coding guidelines

### 6.2 Key Rules

Enforcement as implemented (Version 1.1; Revision 1.0 named a linker wrap and clang-tidy checks that did not exist).
Details are in `software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md` §2.1.

| Rule | Description | Enforcement (Rust) | Enforcement (remaining C) |
|------|-------------|--------------------|---------------------------|
| JPL-1 | Simple control flow | Compiler (no `goto`); CI call-graph cycle check on the image (`scripts/stack_usage.py`) | Review |
| JPL-2 | Fixed loop bounds | Review (`software/flight/docs/LOOP_BOUNDS.md`); fuzzing | Review |
| JPL-3 | No dynamic memory after init | ctest `no_dynamic_memory`; CI image check | ctest `no_dynamic_memory` |
| JPL-4 | Function length < 60 lines | clippy `too_many_lines` (two documented exceptions) | `function_length.py` ratchet |
| JPL-5 | Minimum 2 assertions per function | Not measured; bounds and overflow checks | Not measured |
| JPL-6 | Smallest scope for data | Compiler | Review |
| JPL-7 | Check all return values | Lint for `Result`/`Option`; review for C-ABI codes | cppcheck, clang-tidy |
| JPL-8 | Limited preprocessor use | Review | MISRA addon; review |
| JPL-9 | Restrict pointer use | Compiler (`#![forbid(unsafe_code)]`); review (SAFETY comments) | Review |
| JPL-10 | Compile with all warnings | clippy `-D warnings` | `-Werror` (CMake) |

### 6.3 Naming Conventions

C conventions (the C headers, the shim and the tests). Rust naming is in
`software/flight/docs/NAMING_CONVENTION.md` §12.3.

| Element | Convention | Example |
|---------|------------|---------|
| Functions | module_verb_object | `eps_control_payload()` |
| Types | PascalCase + _t | `EpsControlState_t` |
| Constants | UPPER_SNAKE_CASE | `MAX_SENSORS` |
| Global vars | g_ prefix | `g_system_state` |
| Static vars | s_ prefix | `s_initialized` |
| Local vars | lower_snake_case | `current_soc` |

### 6.4 Python (Payload Software)

- PEP 8 style guide
- Type hints required (mypy strict)
- Docstrings for all public functions

---

## 7. Configuration Management

### 7.1 Version Control

- Git repository hosted on GitHub
- Branch protection on main branch
- Signed commits required for releases

### 7.2 Branching Strategy

```
main ─────────────────────────────────────────>
       \           \           \
        feature/xyz develop     release/v1.0
```

| Branch | Purpose |
|--------|---------|
| main | Flight-ready code |
| develop | Integration branch |
| feature/* | Feature development |
| release/* | Release preparation |
| hotfix/* | Critical fixes |

### 7.3 Version Numbering

Semantic Versioning: MAJOR.MINOR.PATCH

| Version | Meaning |
|---------|---------|
| 0.x.x | Development (pre-flight) |
| 1.0.0 | Initial flight release |
| 1.x.0 | Feature additions |
| 1.x.x | Bug fixes |

### 7.4 Configuration Items

| Item | Baseline |
|------|----------|
| Flight software source | CDR |
| Test procedures | CDR |
| Configuration files | TRR |
| Flight software binary | FRR |

---

## 8. Testing Strategy

### 8.1 Test Levels

| Level | Description | Coverage Target |
|-------|-------------|-----------------|
| Unit | Individual functions | 80% statement |
| Integration | Module interactions | 75% branch |
| System | End-to-end functionality | 100% requirements |
| Acceptance | Mission scenarios | All use cases |

### 8.2 Test Types

| Type | Purpose | When |
|------|---------|------|
| Functional | Verify behavior | All phases |
| Regression | Prevent regressions | Each build |
| Performance | Timing verification | Phase C/D |
| Stress | Boundary conditions | Phase D |
| Environmental | TVAC, vibration | Phase D |

### 8.3 Test Automation

- Unit tests run on every commit (CI)
- Integration tests run nightly
- Coverage reports generated automatically
- Static analysis on every PR

---

## 9. Verification & Validation

### 9.1 Verification Methods

| Method | Code | Description |
|--------|------|-------------|
| Analysis | A | Mathematical/analytical verification |
| Inspection | I | Visual or manual examination |
| Demonstration | D | Operational demonstration |
| Test | T | Execution with specific inputs |

### 9.2 Requirements Traceability

All requirements shall be:
1. Uniquely identified (SRS-xxx)
2. Traced to design elements
3. Traced to code (file:line)
4. Traced to test cases
5. Verified and recorded in VCRM

### 9.3 Verification Matrix

See `docs/verification/VERIFICATION_MATRIX.md` for complete VCRM.

---

## 10. Risk Management

### 10.1 Software Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Jetson power consumption | M | H | DVFS limiting, power gating |
| UART communication loss | L | M | Fallback to template beacons |
| Memory corruption | L | H | CRC protection, TMR |
| Watchdog failure | L | H | Hardware watchdog |

### 10.2 Risk Monitoring

- Risks reviewed at each milestone
- New risks added to register as identified
- Closed risks archived with rationale

---

## 11. Documentation Requirements

### 11.1 Required Documents

| Document | Phase | Owner |
|----------|-------|-------|
| Software Development Plan | B | SW Lead |
| Software Requirements Spec | B | SE Lead |
| Software Design Description | C | SW Lead |
| Interface Control Documents | C | SE Lead |
| Software Test Plan | C | Test Lead |
| Software Version Description | D | SW Lead |

### 11.2 Code Documentation

- Doxygen comments for all public functions
- Module-level documentation in headers
- Algorithm descriptions in design doc

---

## 12. Training Requirements

### 12.1 Developer Training

| Topic | Required For |
|-------|--------------|
| Coding standards | All developers |
| Tool usage | All developers |
| Safety-critical design | OBC developers |
| Embassy executor and async Rust on the OBC (no RTOS) | OBC developers |

---

## Appendix A: References

1. NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
2. NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements
3. NPR 7150.2D - NASA Software Engineering Requirements
4. NASA-STD-8739.8B - Software Assurance and Software Safety Standard (replaces NASA-STD-8719.13, cancelled 2020)
5. JPL D-60411 - JPL Institutional Coding Standard

---

## Appendix B: Acronyms

| Acronym | Definition |
|---------|------------|
| CDR | Critical Design Review |
| FRR | Flight Readiness Review |
| MCU | Microcontroller Unit |
| OBC | On-Board Computer |
| PDR | Preliminary Design Review |
| SDP | Software Development Plan |
| SRR | System Requirements Review |
| TRR | Test Readiness Review |
| VCRM | Verification Cross-Reference Matrix |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-24 | SMART-QSO Team | The OBC flight software is Rust on the STM32L4R5ZI with the Embassy executor (no FreeRTOS): §2.2, §2.3, §5 (platforms, tools, targets) and §6 (Rust standard; JPL enforcement as implemented). §3.3 notes: line coverage is gated, branch coverage is not; the toolchain is not qualified |
| 1.2 | 2026-10-25 | SMART-QSO Team | §3.2 safety-critical components determined by the NASA-STD-8739.8B §4.2 criteria from the hazard analysis, with their coverage; SWE-219 not met, tailoring request CR-2026-003. §3.3 tailoring for Class C, branch coverage now gated. §3.1 software classification determined against NPR 7150.2D Appendix D: Class C for flight software, ground software used in operations and verification software, Class D for research and analysis tools (findings PI-18, VV-12); §2.3 table to match; §3.2 cites NASA-STD-8739.8 and records that the safety-criticality statement disagrees with the hazard analysis and safety documents (open). Header cites NPR 7150.2D. §12.1: Embassy executor in place of RTOS concepts (BASELINE §6, CR-2026-002) |
| 1.3 | 2026-10-27 | SMART-QSO Team (CDR avionics review) | §3.1 and §3.3 point to the Appendix C compliance matrix (SQSO-FSW-012), which replaces the "still to do" |
