# SMART-QSO Software Safety Data Package

**Document ID**: SQSO-SDP-001
**Version**: 1.1
**Date**: 2026-10-16
**Classification**: Unclassified
**Status**: Draft. Software hazards only; every hazard is open

> **Scope (funding audit VV-20, 2026-10-16).** This package covers **flight software** only. It is not the mission
> safety case and it is not a launch-provider deliverable on its own. The launch provider reviews a **system** safety
> data package covering the whole spacecraft, its batteries, inhibits, structure, materials, ground equipment and
> launch-site operations; its outline is [SYSTEM_SAFETY_DATA_PACKAGE.md](SYSTEM_SAFETY_DATA_PACKAGE.md) (skeleton,
> incomplete). This document feeds section 13 of it.

## 1. Purpose

This document consolidates the software safety information for SMART-QSO, following NASA-STD-8719.13 (software safety).
It is one input to the system safety data package. It does not authorize release of flight software or flight.

## 2. Applicable Documents

| Document ID | Title |
|-------------|-------|
| NASA-STD-8719.13 | NASA Software Safety Standard |
| NPR 8715.3 | NASA General Safety Program Requirements |
| NPR 8705.4 | Risk Classification for NASA Payloads |
| SQSO-SHA-001 | Software Hazard Analysis |
| SQSO-SSP-001 | Software Safety Plan |
| SQSO-FMEA-001 | Software FMEA |

## 3. Safety Classification

### 3.1 Mission Classification

| Item | Classification | Rationale |
|------|----------------|-----------|
| Payload risk classification | Class D (NPR 8705.4B), proposed | BASELINE §1 |
| Software Class | **Pending**: NPR 7150.2D Appendix D classification not yet determined | BASELINE §1 |
| Safety-Critical Functions | Yes | RF, battery management |

### 3.2 Safety-Critical Software Identification

| Function | Safety Criticality | Hazard Link |
|----------|-------------------|-------------|
| Watchdog Timer | Critical | HZ-004 |
| Battery Protection | Critical | HZ-002 |
| RF TX Enable | Critical | HZ-001 |
| Thermal Protection | Critical | HZ-003 |
| Safe Mode Logic | Critical | All hazards |

## 4. Hazard Summary

### 4.1 Hazard List

This table is the **hazard log** (FINDINGS_REGISTER PI-07). Software hazards (SW-HAZ-*) are analysed in
`SOFTWARE_HAZARD_ANALYSIS.md` and roll up here. A hazard closes only when every control has cited verification
evidence: test log, analysis or inspection. Software unit tests already cover some controls. HZ-001: TX-OFF,
RF-silence and loss-of-contact tests. HZ-004: the supervised watchdog test. HZ-006: telecommand-authentication
tests. No hardware verification has been run, so every hazard is open.

| ID | Hazard | Severity | Likelihood | Risk | Status |
|----|--------|----------|------------|------|--------|
| HZ-001 | Unintended RF Emission | III | C | Medium | Open: controls designed, not verified |
| HZ-002 | Battery Over-discharge | III | C | Medium | Open: controls designed, not verified |
| HZ-003 | Thermal Damage | III | D | Low | Open: controls designed, not verified |
| HZ-004 | Software Hang | III | C | Medium | Open: controls designed, not verified |
| HZ-005 | Memory Corruption | III | D | Low | Open: controls designed, not verified |
| HZ-006 | Command Injection | IV | E | Low | Open: controls designed, not verified |

### 4.2 Severity Definitions

| Level | Definition | Example |
|-------|------------|---------|
| I | Catastrophic | Loss of life |
| II | Critical | Major injury |
| III | Marginal | Minor injury, mission failure |
| IV | Negligible | Nuisance, no injury |

### 4.3 Likelihood Definitions

| Level | Definition | Probability |
|-------|------------|-------------|
| A | Frequent | P > 0.1 |
| B | Probable | 0.1 > P > 0.01 |
| C | Occasional | 0.01 > P > 0.001 |
| D | Remote | 0.001 > P > 0.000001 |
| E | Improbable | P < 0.000001 |

## 5. Hazard Controls

### 5.1 HZ-001: Unintended RF Emission

**Hazard Description**: Spacecraft transmits RF energy when not authorized, potentially causing interference or regulatory violation.

**Causes**:
- Software bug enabling transmitter
- Cosmic ray bit flip
- Command processing error

**Controls**:
| Control | Type | Implementation |
|---------|------|----------------|
| TX Enable Interlock | Hardware | GPIO + enable circuit |
| TX State Machine | Software | Explicit state required |
| Power-on State | Design | TX disabled at boot |
| Command Verification | Procedural | Two-operator rule |

**Verification**:
- [ ] TX disabled at boot (TP-001)
- [ ] TX only enabled in ACTIVE mode (TP-002)
- [ ] TX disabled on fault (TP-005)
- [ ] Code review of TX control logic

**Residual Risk**: Low - Multiple independent controls

### 5.2 HZ-002: Battery Over-discharge

**Hazard Description**: Battery depleted below safe threshold, causing permanent capacity loss or failure.

**Causes**:
- Excessive load operation
- Solar array failure
- Power management failure

**Controls**:
| Control | Type | Implementation |
|---------|------|----------------|
| SOC Monitoring | Software | Continuous monitoring |
| ACTIVE → IDLE | Software | SOC below 40 % (BASELINE §5) |
| Auto-SAFE Mode | Software | SOC below 25 % |
| EMERGENCY (beacon shed) | Software | SOC below 15 % |
| Cell protection | Hardware | EPS cell over-voltage ≤ 4.25 V/cell; under-voltage threshold TBD |

**Verification**:
- [ ] SOC monitoring accurate (TP-003)
- [ ] Auto-SAFE below 25 % SOC (TP-002)
- [ ] Power budget positive (TP-006)
- [ ] Extended operation test (TP-007)

**Residual Risk**: Low - Multiple layers of protection

### 5.3 HZ-003: Thermal Damage

**Hazard Description**: Component overheating causing permanent damage or reduced life.

**Causes**:
- Excessive processing load
- Inadequate thermal design
- Solar panel sun-pointing

**Controls**:
| Control | Type | Implementation |
|---------|------|----------------|
| Temperature Monitoring | Software | Multiple sensors |
| Thermal Alarms | Software | Per-component limits |
| Thermal Throttling | Software | Reduce operations |
| AI Duty Cycle Limit | Software | ≤ 5 % tumbling, ≤ 10 % sun-pointed (BASELINE §5) |

**Verification**:
- [ ] Temperature sensors read correctly (TP-003)
- [ ] Throttling activated on over-temp (TP-005)
- [ ] AI duty cycle enforced (TP-006)

**Residual Risk**: Low - Active thermal management

### 5.4 HZ-004: Software Hang

**Hazard Description**: Software enters infinite loop or deadlock, preventing spacecraft operation.

**Causes**:
- Software bug
- Stack overflow
- Interrupt storm
- Resource deadlock

**Controls**:
| Control | Type | Implementation |
|---------|------|----------------|
| Watchdog Timer | Hardware | 30 second timeout, refreshed only while every main-loop stage checks in |
| Stack Checking | Software | **Not implemented**: planned with the RTOS, which is not yet integrated |
| Task Monitoring | Software | Health monitoring task |
| Boot Counter | Software | Tracks resets |

**Verification**:
- [ ] Watchdog tested (TP-001, TP-005)
- [ ] Stack overflow handled (TP-005)
- [ ] Boot counter increments (TP-005)

**Residual Risk**: Low - Hardware watchdog ensures recovery

### 5.5 HZ-005: Memory Corruption

**Hazard Description**: Critical data corrupted by radiation or software error, causing incorrect operation.

**Causes**:
- Single Event Upset (SEU)
- Buffer overflow
- Uninitialized variables

**Controls**:
| Control | Type | Implementation |
|---------|------|----------------|
| ECC Memory | Hardware | OBC SRAM ECC |
| CRC on Config | Software | Configuration checksums |
| Defensive Coding | Software | MISRA C:2012 checking in CI (not compliant: 1,189 findings, `software/flight/quality/misra.baseline`). Since 2026-10-24 the flight software is Rust: MISRA applies to the remaining C shim only (42 findings), and the Rust lints are enforced in CI (`docs/development/CODING_STANDARDS.md` §11) |
| Bounds Checking | Software | Array bounds checked |

**Verification**:
- [ ] MISRA compliance (static analysis)
- [ ] CRC verification (TP-001)
- [ ] Memory test at boot (TP-001)

**Residual Risk**: Low - ECC and defensive coding

### 5.6 HZ-006: Command Injection

**Hazard Description**: Unauthorized command execution from malicious or erroneous uplink.

**Causes**:
- Incorrect callsign match
- Replay attack
- Software vulnerability

**Controls**:
| Control | Type | Implementation |
|---------|------|----------------|
| Command Authentication | Software | HMAC-SHA-256 (truncated tag) with a persisted monotonic counter per key, from designated stations only (BASELINE §7; 47 CFR 97.211) |
| Replay Protection | Software | Monotonic counter |
| Command Validation | Software | Command dictionary and argument-length checks |
| Open Content | Design | Telecommand authenticated, not encrypted; downlink in the clear |

**Verification**:
- [ ] Invalid callsign rejected (test)
- [ ] Invalid commands rejected (test)
- [ ] Code review of command parser

**Residual Risk**: Medium - Amateur radio requires open access, mitigated by community monitoring

## 6. Safety Verification Matrix

### 6.1 Safety Requirements Verification

| Req ID | Requirement | Method | Status | Evidence |
|--------|-------------|--------|--------|----------|
| SR-001 | Safe mode on critical fault | Test | [ ] | TP-005 |
| SR-002 | Watchdog protection | Test | [ ] | TP-005.1 |
| SR-003 | No unintended TX | Analysis + Test | [ ] | Code review + TP-004 |
| SR-004 | Power budget positive | Analysis + Test | [ ] | Analysis + TP-006 |
| SR-005 | Thermal protection | Test | [ ] | TP-005.8 |

### 6.2 Safety Test Summary

| Test | Hazards Verified | Status |
|------|------------------|--------|
| TP-001 Boot Test | HZ-001, HZ-004, HZ-005 | [ ] |
| TP-002 Mode Transition | HZ-001, HZ-002 | [ ] |
| TP-005 Fault Response | All hazards | [ ] |
| TP-006 Power Mode | HZ-002, HZ-003 | [ ] |

## 7. Failure Mode Summary

### 7.1 Critical Failure Modes

| Failure Mode | Effect | Detection | Response |
|--------------|--------|-----------|----------|
| OBC hang | Loss of control | Watchdog | Auto-reset |
| Battery depletion | Loss of mission | Telemetry | SAFE mode |
| RF PA failure | Loss of downlink | No beacon | Graceful deg. |
| Jetson failure | Loss of AI | Telemetry | Fallback mode |

### 7.2 Failure Response Matrix

| Component | Failure | Response Time | Recovery |
|-----------|---------|---------------|----------|
| OBC | Watchdog reset | 30 seconds | Auto |
| Battery | Low SOC | Immediate | Auto-SAFE |
| RF TX | Over-temp | Immediate | Throttle |
| Jetson | Timeout | 30 seconds | Fallback |

## 8. Safety Analysis Summary

### 8.1 Software FMEA Results

From `SOFTWARE_FMEA.md` §12: 43 failure modes analysed; 14 with RPN above 50; none above 100. (Version 1.0 of this
package gave 45, a figure the FMEA does not contain.)

### 8.2 Fault Tree Analysis Results

No fault tree analysis exists yet. The software safety plan and hazard analysis call for one.

### 8.3 Software Hazard Analysis Results

Hazards identified: 6
Hazards with designed controls: 6
Hazards with verified controls: 0
Open hazards: 6 (see §4.1)

## 9. Safety Verification Status

### 9.1 Pre-Flight Safety Verification

| Item | Status | Evidence |
|------|--------|----------|
| Software Hazard Analysis drafted (v1.0; not independently reviewed) | [x] | SQSO-SHA-001 |
| Software FMEA drafted (v1.0; not independently reviewed) | [x] | SQSO-FMEA-001 |
| Safety requirements verified | [ ] | Test reports |
| Safety controls verified | [ ] | Test reports |
| Residual risks accepted | [ ] | Risk register |

### 9.2 Safety Closure Checklist

| Item | Verified | Date | By |
|------|----------|------|-----|
| All hazards have controls | [ ] | | |
| All controls verified | [ ] | | |
| No unacceptable risks | [ ] | | |
| Safety documentation complete | [ ] | | |
| Safety review board approval | [ ] | | |

## 10. Safety Data Package Contents

### 10.1 Document List

| Document | Version | Status |
|----------|---------|--------|
| Software Safety Plan | 1.0 | Draft |
| Software Hazard Analysis | 1.0 | Draft |
| Software FMEA | 1.0 | Draft |
| Software Safety Data Package (this document) | 1.1 | Draft |
| Safety Test Procedures | 1.0 | Draft; not executed |
| Safety Test Reports | - | None; no tests run |
| System Safety Data Package | 0.1 | Skeleton only (SQSO-SDP-002) |

### 10.2 Archive Location

All safety documentation archived in:
- Repository: docs/safety/
- Backup: Project document server

## 11. Safety Approval

### 11.1 Safety Review Board Recommendation

```
SAFETY REVIEW BOARD FINDINGS

Review Date: ___________
Configuration: repository commit ________ (no flight software release exists)

FINDINGS (to be written by the board):
1. _________________________________
2. _________________________________
3. _________________________________

OPEN ITEMS:
1. Complete flatsat safety testing (TP-005)
2. Verify hardware controls on flight unit

RECOMMENDATION:
[ ] Proceed to flight
[ ] Proceed with conditions (list below)
[ ] Do not proceed (list concerns)

CONDITIONS:
_________________________________

Board Chair: ____________ Date: _____
Member: _________________ Date: _____
Member: _________________ Date: _____
```

### 11.2 Safety Approval Signatures

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Project Manager | | | |
| Safety Engineer | | | |
| Mission Assurance | | | |

## 12. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | SMART-QSO Team | Funding audit VV-20: retitled Software Safety Data Package and scoped to software; system package split out as a skeleton (SQSO-SDP-002); "Complete" statuses, pre-written board findings and the zero-open-hazard summary removed; values aligned with BASELINE (SOC thresholds, watchdog 30 s, AI duty, software class pending); FMEA count corrected to 43 |
| 1.1, note | 2026-10-24 | SMART-QSO Team | Defensive-coding row: status after the port of the flight software to Rust |
