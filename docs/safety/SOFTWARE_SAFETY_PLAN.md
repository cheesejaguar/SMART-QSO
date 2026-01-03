# Software Safety Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-SSP-001
**Revision:** A
**Date:** 2026-01-02
**Classification:** Class D Mission (per NPR 7120.5)
**Governing Standards:** NASA-STD-8719.13, NPR 7150.2

---

## 1. Purpose

This Software Safety Plan establishes the software safety program for the SMART-QSO CubeSat mission. It defines the organization, responsibilities, processes, and activities required to ensure that software does not contribute to system hazards or cause unacceptable risk to the mission, ground personnel, or other space assets.

## 2. Scope

This plan applies to all software components of the SMART-QSO mission:
- OBC Flight Software (C, FreeRTOS)
- AI Payload Software (Python, Jetson)
- Ground Support Software
- Test and Simulation Software

## 3. Applicable Documents

### 3.1 Governing Standards
| Document | Title |
|----------|-------|
| NPR 7120.5 | NASA Space Flight Program and Project Management Requirements |
| NPR 7123.1 | NASA Systems Engineering Processes and Requirements |
| NPR 7150.2 | NASA Software Engineering Requirements |
| NASA-STD-8719.13 | Software Safety Standard |
| NASA-STD-8739.8 | Software Assurance Standard |

### 3.2 Project Documents
| Document | Title |
|----------|-------|
| SQSO-SEMP-001 | Systems Engineering Management Plan |
| SQSO-SDP-001 | Software Development Plan |
| SQSO-SRS-001 | Software Requirements Specification |
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

Per NASA-STD-8719.13 for Class D missions:
- Software safety analysis may be performed by development team
- Independent review required for safety-critical software changes
- Safety-critical code requires peer review by qualified reviewer

## 5. Software Criticality Classification

### 5.1 Software Classification Criteria

Per NPR 7150.2 and NASA-STD-8719.13:

| Class | Description | SMART-QSO Application |
|-------|-------------|----------------------|
| A | Loss of life or vehicle | Not applicable (unmanned, no propulsion) |
| B | Loss of mission | EPS control, Safe mode entry |
| C | Major mission degradation | ADCS, Beacon generation |
| D | Minor mission degradation | AI payload, Telemetry |
| E | No safety impact | Ground software, Simulation |

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
| FSR-001 | Upon loss of OBC, beacon shall continue in hardware-timer mode | Test |
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

Safety-critical software (Class B and C) requires:
- 100% statement coverage
- 100% branch coverage
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
| W-001 | Use of COTS Jetson without radiation testing | Class D mission, graceful degradation to OBC | Approved |
| W-002 | Single-string OBC (no redundancy) | CubeSat mass/volume constraints, watchdog recovery | Approved |

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

---

*Prepared in accordance with NASA-STD-8719.13 and NPR 7150.2*
