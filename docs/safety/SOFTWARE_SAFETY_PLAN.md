# Software Safety Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-SSP-001
**Revision:** F
**Date:** 2026-10-27
**Classification:** Class D payload (NPR 8705.4B, proposed); software Class C (NPR 7150.2D)
**Governing Standards:** NASA-STD-8739.8B (replaces NASA-STD-8719.13, cancelled 2020), NPR 7150.2D

---

## 1. Purpose

This Software Safety Plan establishes the software safety program for the SMART-QSO CubeSat mission. It defines the organization, responsibilities, processes, and activities required to ensure that software does not contribute to system hazards or cause unacceptable risk to the mission, ground personnel, or other space assets.

## 2. Scope

This plan applies to all software components of the SMART-QSO mission:
- OBC Flight Software (Rust on the Embassy executor, no RTOS: the baseline since 2026-10-25, BASELINE §6, CR-2026-002.
  Revision B said C and FreeRTOS; FreeRTOS was never integrated. See `software/rust/README.md`)
- AI Payload Software (Python, Jetson)
- Ground Support Software
- Test and Simulation Software

## 3. Applicable Documents

### 3.1 Governing Standards
| Document | Title |
|----------|-------|
| NPR 7120.5F | NASA Space Flight Program and Project Management Requirements (reference only; not governing for this privately funded mission, CM plan 1.1) |
| NPR 7123.1D | NASA Systems Engineering Processes and Requirements |
| NPR 7150.2D | NASA Software Engineering Requirements |
| NASA-STD-8739.8B | Software Assurance and Software Safety Standard (replaced NASA-STD-8719.13, cancelled 2020) |

### 3.2 Project Documents
| Document | Title |
|----------|-------|
| SMART-QSO-SEMP-001 | Systems Engineering Management Plan |
| SMART-QSO-SDP-001 | Software Development Plan |
| SMART-QSO-SRS-001 | Software Requirements Specification |
| SQSO-SHA-001 | Software Hazard Analysis |

## 4. Software Safety Organization

### 4.1 Roles and Responsibilities

| Role | Responsibility |
|------|---------------|
| Project Manager | Overall safety program oversight, safety waivers |
| Systems Engineer | System-level hazard analysis, safety requirements flow-down |
| Software Lead | Software safety implementation, code reviews |
| Safety Engineer | Hazard tracking, verification oversight |
| Quality Assurance | Process compliance, safety audits |

### 4.2 Independence Requirements

Per NASA-STD-8739.8B (software safety and independence tailored for Class C software on a Class D payload):
- Software safety analysis may be performed by development team
- Independent review required for safety-critical software changes
- Safety-critical code requires peer review by qualified reviewer

## 5. Software Criticality Classification

### 5.1 Software Classification Criteria

Per NPR 7150.2D Appendix D and NASA-STD-8739.8B:

| Class | Description | SMART-QSO Application |
|-------|-------------|----------------------|
| A | Loss of life or vehicle | Not applicable (unmanned, no propulsion) |
| B | Loss of mission | EPS control, Safe mode entry |
| C | Major mission degradation | ADCS, Beacon generation |
| D | Minor mission degradation | AI payload, Telemetry |
| E | No safety impact | Ground software, Simulation |

Note (2026-10-25). The letters A to E in §5.1, §5.2 and §8.2 are this plan's criticality scale. They are not NPR 7150.2D
software classes: the flight software is Class C (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1). The safety-critical
components under NASA-STD-8739.8B are determined in SDP §3.2 (§8.2 here).

### 5.2 SMART-QSO Software Criticality

| Module | Class | Rationale |
|--------|-------|-----------|
| EPS Control | B | Controls power to all subsystems, battery protection |
| Fault Management | B | Safe mode entry, fault recovery |
| Watchdog | B | System reset, fault recovery |
| ADCS Control | C | Attitude control for communication |
| Beacon Generation | C | Primary mission data product |
| UART Communication | C | OBC-Jetson interface |
| Sensor Management | D | Telemetry collection |
| AI Payload | D | Non-critical enhancement |
| Mission Data | D | Data persistence |

## 6. Software Safety Requirements

### 6.1 General Safety Requirements

| ID | Requirement | Verification |
|----|-------------|-------------|
| SSR-001 | Software shall detect and respond to all identified fault conditions within 1 second | Test |
| SSR-002 | Software shall transition to safe mode upon detection of critical faults | Test |
| SSR-003 | Software shall not command power levels exceeding hardware limits | Analysis, Test |
| SSR-004 | Software shall implement watchdog timeout with maximum 30 second period | Test |
| SSR-005 | Software shall preserve critical data across resets using CRC protection | Test |
| SSR-006 | Software shall limit RF transmission power to FCC/IARU approved levels | Analysis, Test |
| SSR-007 | Software shall implement battery protection (over/under voltage, temperature) | Test |
| SSR-008 | Software shall disable AI payload when power is insufficient | Test |

### 6.2 Fail-Safe Design Requirements

| ID | Requirement | Verification |
|----|-------------|-------------|
| FSR-001 | *Withdrawn (documentation audit, 2026-10-27): the design has no beacon without the OBC; the TNC sends only frames the OBC gives it (TNC-F020).* Upon loss of the OBC, the supervisor reset and the EPS controller's OBC power cycle shall restore it (SRS-F146, SRS-F162) | Test |
| FSR-002 | Upon loss of Jetson, OBC shall continue with template beacons | Test |
| FSR-003 | Upon low battery, system shall shed loads in priority order | Test |
| FSR-004 | Upon thermal limit, Jetson shall throttle or shutdown | Test |
| FSR-005 | Upon communication timeout, system shall enter safe mode | Test |

## 7. Safety Analysis Process

### 7.1 Hazard Analysis Approach

The software safety analysis follows a top-down approach:

1. **System Hazard Analysis (SHA)**: Identify system-level hazards
2. **Software Hazard Analysis**: Identify software contributions to system hazards
3. **Software FMEA**: Analyze failure modes of each software module
4. **Fault Tree Analysis (FTA)**: Trace paths from hazards to root causes

### 7.2 Hazard Tracking

All software hazards are tracked in the Software Hazard Analysis document (SQSO-SHA-001) with:
- Unique hazard ID
- Hazard description
- Severity and likelihood assessment
- Software contribution
- Controls and mitigations
- Verification status
- Closure status

### 7.3 Hazard Severity Classification

| Category | Description | Example |
|----------|-------------|---------|
| Catastrophic | Loss of human life or permanent disability | N/A (unmanned) |
| Critical | Loss of mission or major system damage | Battery thermal runaway |
| Major | Significant mission degradation | Loss of attitude control |
| Minor | Mission degradation recoverable by ground | Temporary beacon loss |
| Negligible | No mission impact | Telemetry glitch |

### 7.4 Hazard Likelihood Classification

| Category | Description |
|----------|-------------|
| Frequent | > 10^-1 per mission |
| Probable | 10^-2 to 10^-1 per mission |
| Occasional | 10^-4 to 10^-2 per mission |
| Remote | 10^-6 to 10^-4 per mission |
| Improbable | < 10^-6 per mission |

## 8. Safety Verification

### 8.1 Verification Methods

| Method | Application |
|--------|-------------|
| Analysis | Algorithm correctness, timing analysis, code review |
| Inspection | Code inspection, design review |
| Demonstration | Functional demonstration of safety features |
| Test | Automated tests, fault injection, integration tests |

### 8.2 Safety-Critical Code Testing

The safety-critical components are determined in `docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.2 (NASA-STD-8739.8B
§4.2 criteria, from the hazard analysis). NPR 7150.2D SWE-219 requires for them 100 % code coverage by the MC/DC
criterion. rustc cannot measure MC/DC; change request CR-2026-003 (approved by the PI, 2026-10-25,
`docs/project/CHANGE_REQUEST_LOG.md`) replaces it with 100 % line and 100 % condition coverage. Status (2026-10-25):
CI gates line coverage (Rust ≥ 90 %, 94.8 %) and branch and condition coverage (Rust ≥ 80 %, 81.1 %) over the whole
flight software, and each safety-critical file at 100 % of lines and 100 % of conditions
(`software/flight/scripts/safety_coverage.py`, CI job "Build and Test"). All of them meet it; 12 lines that no test
can reach (a defensive arm, an unreachable error path) are marked `// COVERAGE-EXEMPT:` with the reason, each
reviewed (SDP §3.2).

Status (2026-10-27, CDR): Rust lines 94.2 % and branches and conditions 80.9 % over the flight crates; the 23
safety-critical files pass the per-file gate with 0 problems after the CDR changes to `rf_silence.rs`, `deployment.rs`
and `state_machine.rs`. The CDR added four hazards (`docs/safety/SOFTWARE_HAZARD_ANALYSIS.md` Rev B §9: transponder
emission, RF inhibits, commissioning hold, OBC selector). Their controls in `transponder.rs` and `ops_state.rs` meet
the NASA-STD-8739.8B §4.2 criteria and joined the safety-critical list on 2026-10-27 (RID FSW-05, SRS-Q013).

Status (2026-10-27, CDR flight software round 2): Rust lines 94.3 % and branches and conditions 81.4 % over the flight
crates; the 26 safety-critical files (`sqso-fsw/src/transponder.rs`, `sqso-fsw/src/ops_state.rs` and
`sqso-ffi/src/ops_state.rs` added) pass the per-file gate with 0 problems and 15 reviewed `COVERAGE-EXEMPT` lines. NPR 7150.2D SWE-220 (cyclomatic complexity
≤ 15 for safety-critical components) is not yet measured for the Rust (RID FSW-06,
`docs/software/NPR7150_COMPLIANCE_MATRIX.md`). Also required:
- Fault injection testing for all identified fault conditions
- Boundary value testing for all safety parameters
- Stress testing under worst-case conditions

### 8.3 Safety Review Gates

| Review | Safety Activities |
|--------|------------------|
| PDR | Preliminary hazard analysis, safety requirements |
| CDR | Complete hazard analysis, safety design review |
| TRR | Safety test results, hazard closure status |
| FRR | All hazards closed or accepted, safety data package |

## 9. Software Safety Controls

### 9.1 Design Controls

| Control | Implementation |
|---------|---------------|
| Defensive programming | Assertions, input validation, range checks |
| Watchdog timer | Hardware watchdog with 30s timeout |
| Safe mode | Autonomous entry on critical faults |
| Data integrity | CRC32 on persisted data |
| Power protection | Hardware and software current/voltage limits |
| Thermal protection | Temperature monitoring and throttling |

### 9.2 Process Controls

| Control | Implementation |
|---------|---------------|
| Coding standards | JPL Power of Ten, MISRA C:2012 |
| Static analysis | cppcheck, clang-tidy |
| Code review | Peer review for all changes |
| Unit testing | cmocka framework, >80% coverage |

As implemented (2026-10-24), for the Rust flight software: the coding standard is `docs/development/CODING_STANDARDS.md`
§11 (MISRA C applies only to the remaining C shim). The JPL rules are mapped to the Rust in
`software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md` §2.1. Static analysis is clippy with warnings as errors, the
`#![forbid(unsafe_code)]` check, CodeQL and `cargo audit`, plus cppcheck and clang-tidy on the remaining C. Every parser
is fuzzed in CI. The CMocka suites run unchanged against the Rust, together with 358 Rust unit tests. Safety-relevant
modules fail safe when re-entered: TX-OFF still drives the transmitter enable low, the RF silence reads as active, and
the payload guard trips (`software/rust/README.md`, behaviour change 10).
| Configuration management | Git, tagged releases |

## 10. Safety Waivers and Deviations

### 10.1 Waiver Process

Safety waivers require:
1. Technical justification for deviation
2. Risk assessment with deviation
3. Compensating controls identified
4. Approval by Project Manager and Safety Engineer

### 10.2 Current Waivers

| Waiver ID | Description | Rationale | Status |
|-----------|-------------|-----------|--------|
| W-001 | Use of COTS Jetson without radiation testing | Class D mission, graceful degradation to OBC | Proposed; no approval recorded |
| W-002 | Single-string OBC (no redundancy) | CubeSat mass/volume constraints, watchdog recovery | Proposed; no approval recorded |

## 11. Safety Metrics

### 11.1 Tracked Metrics

| Metric | Target | Frequency |
|--------|--------|-----------|
| Open safety hazards | 0 at FRR | Weekly |
| Safety requirements verified | 100% at FRR | Weekly |
| Safety-critical test coverage | 100% | Per build |
| Safety-related anomalies | Trending down | Weekly |
| Safety review actions closed | 100% before next gate | Weekly |

## 12. Documentation Requirements

### 12.1 Required Safety Documents

| Document | Description | Phase |
|----------|-------------|-------|
| Software Safety Plan | This document | PDR |
| Software Hazard Analysis | Hazard identification and tracking | PDR, CDR |
| Software FMEA | Failure mode analysis | CDR |
| Safety Test Results | Verification of safety requirements | TRR |
| Safety Data Package | Complete safety closure | FRR |

## 13. Training Requirements

All personnel working on safety-critical software shall complete:
- Software safety awareness training
- Project coding standards training
- Configuration management procedures
- Emergency procedures (if applicable)

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | §3.2 cites the numbers the documents carry: the Software Development Plan is SMART-QSO-SDP-001 (SQSO-SDP-001 is the Safety Data Package); SEMP and SRS likewise (finding PI-17) | SMART-QSO Team |
| C | 2026-10-24 | The OBC flight software is Rust: §2 scope; §8.2 coverage status (line coverage gated, branch not measured); §9.2 process controls as implemented for the Rust | SMART-QSO Team |
| C, note | 2026-10-25 | §2 scope: no RTOS is the baseline (CR-2026-002). §5: the criticality letters are not NPR 7150.2D classes (software Class C, SDP §3.1). Header: payload Class D (NPR 8705.4B) and NASA-STD-8739.8B. §8.2: the safety-critical components and SWE-219 (MC/DC; CR-2026-003) with measured coverage | SMART-QSO Team |
| D | 2026-10-27 | §8.2 CDR status: coverage after the CDR changes, the four hazards added to the hazard analysis, `transponder.rs` and `ops_state.rs` recommended as safety-critical (RID FSW-05), SWE-220 not measured (RID FSW-06) | SMART-QSO Team (CDR avionics review) |
| E | 2026-10-27 | §8.2: `transponder.rs` and `ops_state.rs` (both crates) on the safety-critical list and through the gate (FSW-05); coverage of the round-2 flight software | SMART-QSO Team (CDR flight software, round 2) |
| F | 2026-10-27 | Documentation audit (post-CDR 2026): §5.1 note, the safety-critical determination is made (SDP §3.2); FSR-001 hardware-timer beacon withdrawn (no beacon without the OBC, TNC-F020), restated as OBC recovery; §10.2 waivers W-001 and W-002 shown as proposed (no approval is recorded) | Documentation audit |

---

*Prepared in accordance with NASA-STD-8739.8B and NPR 7150.2D*
