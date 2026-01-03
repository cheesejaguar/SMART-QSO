# SMART-QSO Safety Data Package

**Document ID**: SQSO-SDP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document consolidates all safety-related information for the SMART-QSO mission per NASA-STD-8719.13 requirements. It provides the complete safety case for flight software release authorization.

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
| Mission Category | Category III | Unmanned, low-cost |
| Software Class | Class B | Mission critical |
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

| ID | Hazard | Severity | Likelihood | Risk | Status |
|----|--------|----------|------------|------|--------|
| HZ-001 | Unintended RF Emission | III | C | Medium | Controlled |
| HZ-002 | Battery Over-discharge | III | C | Medium | Controlled |
| HZ-003 | Thermal Damage | III | D | Low | Controlled |
| HZ-004 | Software Hang | III | C | Medium | Controlled |
| HZ-005 | Memory Corruption | III | D | Low | Controlled |
| HZ-006 | Command Injection | IV | E | Low | Controlled |

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
| Low SOC Alarm | Software | Threshold at 30% |
| Auto-SAFE Mode | Software | Trigger at 20% |
| Absolute Cutoff | Hardware | EPS protection at 15% |

**Verification**:
- [ ] SOC monitoring accurate (TP-003)
- [ ] Auto-SAFE at 20% SOC (TP-002)
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
| AI Duty Cycle Limit | Software | Max 10% duty |

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
| Watchdog Timer | Hardware | 5 second timeout |
| Stack Checking | Software | FreeRTOS stack checks |
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
| Defensive Coding | Software | MISRA C compliance |
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
| Callsign Filter | Software | Source validation |
| Sequence Counter | Software | Replay detection |
| Command Validation | Software | Range/format checks |
| Open Protocol | Design | No encryption per amateur rules |

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
| OBC | Watchdog reset | 5 seconds | Auto |
| Battery | Low SOC | Immediate | Auto-SAFE |
| RF TX | Over-temp | Immediate | Throttle |
| Jetson | Timeout | 30 seconds | Fallback |

## 8. Safety Analysis Summary

### 8.1 Software FMEA Results

Total failure modes analyzed: 45
- Critical: 0
- High: 3 (all mitigated to Medium)
- Medium: 12
- Low: 30

### 8.2 Fault Tree Analysis Results

Top events analyzed: 5
All top events show acceptably low probability with controls in place.

### 8.3 Software Hazard Analysis Results

Hazards identified: 6
Hazards controlled: 6
Hazards accepted: 0
Open hazards: 0

## 9. Safety Verification Status

### 9.1 Pre-Flight Safety Verification

| Item | Status | Evidence |
|------|--------|----------|
| Software Hazard Analysis complete | [x] | SQSO-SHA-001 |
| FMEA complete | [x] | SQSO-FMEA-001 |
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
| Software Safety Plan | 1.0 | Complete |
| Software Hazard Analysis | 1.0 | Complete |
| Software FMEA | 1.0 | Complete |
| Safety Data Package | 1.0 | Complete |
| Safety Test Procedures | 1.0 | Complete |
| Safety Test Reports | - | Pending execution |

### 10.2 Archive Location

All safety documentation archived in:
- Repository: docs/safety/
- Backup: Project document server

## 11. Safety Approval

### 11.1 Safety Review Board Recommendation

```
SAFETY REVIEW BOARD FINDINGS

Review Date: ___________
Configuration: SQSO-FSW-1.0.0

FINDINGS:
1. All identified hazards have appropriate controls
2. All controls have verification methods
3. Residual risk is acceptable per mission class

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
