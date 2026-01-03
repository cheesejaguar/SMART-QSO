# SMART-QSO ConOps Validation Matrix

**Document ID**: SQSO-CVM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the ConOps Validation Matrix for SMART-QSO per NPR 7123.1 §4.6.2, ensuring the system validates against the Concept of Operations through defined scenarios, procedures, and success criteria.

## 2. ConOps Reference

The SMART-QSO ConOps (mission/CONOPS.md) defines five mission phases:
1. Launch & Early Orbit (LEOP)
2. Commissioning
3. Nominal Operations
4. Extended Operations
5. Deorbit & Passivation

## 3. Validation Approach

### 3.1 Validation Methods

| Method | Application |
|--------|-------------|
| Simulation | Pre-hardware validation |
| Flatsat | Hardware-in-the-loop |
| Day-in-the-Life | Operational simulation |
| Mission Rehearsal | Full procedure validation |

### 3.2 Validation Evidence

| Evidence Type | Description |
|---------------|-------------|
| Test Report | Formal test execution record |
| As-Run | Actual vs. planned procedure |
| Data Review | Telemetry and performance data |
| Operator Certification | Training completion records |

## 4. Phase 1: LEOP Validation

### 4.1 LEOP ConOps Elements

| ConOps Element | Validation Scenario | Status |
|----------------|---------------------|--------|
| Detumble using magnetorquers | VAL-LEOP-001 | [ ] |
| Establish coarse sun-pointing | VAL-LEOP-002 | [ ] |
| Enable basic beacon (120s) | VAL-LEOP-003 | [ ] |
| Template text only | VAL-LEOP-004 | [ ] |
| Verify reception | VAL-LEOP-005 | [ ] |
| Confirm power-positive | VAL-LEOP-006 | [ ] |

### 4.2 LEOP Validation Scenarios

#### VAL-LEOP-001: Detumble Verification

**Objective**: Verify magnetorquer detumble capability

**Configuration**: Flatsat with ADCS hardware or simulation

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Initialize with tumble rates | Rates > 5°/s | [ ] |
| 2 | Enable detumble mode | B-dot active | [ ] |
| 3 | Monitor rate reduction | Rates decreasing | [ ] |
| 4 | Wait for stabilization | Rates < 1°/s | [ ] |
| 5 | Verify power consumption | Within budget | [ ] |

**Success Criteria**: Tumble rates reduced to <1°/s within 2 orbits (simulated)

#### VAL-LEOP-003: Initial Beacon Verification

**Objective**: Verify basic beacon at LEOP cadence

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Boot to SAFE mode | MODE = SAFE | [ ] |
| 2 | Transition to LEOP mode | MODE = LEOP | [ ] |
| 3 | Enable beacon | Beacon active | [ ] |
| 4 | Time 5 consecutive beacons | Period = 120s ± 5s | [ ] |
| 5 | Decode beacon content | Template only | [ ] |
| 6 | Verify callsign | Correct ID | [ ] |

**Success Criteria**: Beacon period 120s ± 5s, template content only

#### VAL-LEOP-005: Ground Reception Verification

**Objective**: Verify partner station can receive beacons

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Configure ground station | Ready | [ ] |
| 2 | Transmit beacon | RF output verified | [ ] |
| 3 | Receive at ground | Signal detected | [ ] |
| 4 | Decode packet | AX.25 valid | [ ] |
| 5 | Parse telemetry | All fields valid | [ ] |
| 6 | Archive data | Stored in DB | [ ] |

**Success Criteria**: End-to-end beacon reception verified

## 5. Phase 2: Commissioning Validation

### 5.1 Commissioning ConOps Elements

| ConOps Element | Validation Scenario | Status |
|----------------|---------------------|--------|
| Characterize power/thermal | VAL-COM-001 | [ ] |
| Calibrate solar/battery telemetry | VAL-COM-002 | [ ] |
| Exercise DVFS profiles | VAL-COM-003 | [ ] |
| Validate thermal margins | VAL-COM-004 | [ ] |
| Beacon conformance check | VAL-COM-005 | [ ] |
| Set adaptive cadence schedule | VAL-COM-006 | [ ] |

### 5.2 Commissioning Validation Scenarios

#### VAL-COM-001: Power Characterization

**Objective**: Verify power system performance matches design

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Measure SAFE power | <0.2W | [ ] |
| 2 | Measure IDLE power | <0.4W | [ ] |
| 3 | Measure ACTIVE power | <0.8W | [ ] |
| 4 | Measure AI burst power | <7W | [ ] |
| 5 | Verify power budget | Positive margin | [ ] |

**Success Criteria**: All power measurements within budget

#### VAL-COM-003: DVFS Profile Verification

**Objective**: Verify Jetson DVFS operation

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Enable Jetson | Power on | [ ] |
| 2 | Set DVFS profile LOW | Clocks reduced | [ ] |
| 3 | Measure power | <3W | [ ] |
| 4 | Set DVFS profile MEDIUM | Clocks medium | [ ] |
| 5 | Measure power | <5W | [ ] |
| 6 | Verify thermal | Within limits | [ ] |

**Success Criteria**: DVFS profiles control power as designed

#### VAL-COM-005: Beacon Conformance

**Objective**: Verify beacon meets RF specifications

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Transmit beacon | RF output | [ ] |
| 2 | Measure frequency | 145.XXX ± 1kHz | [ ] |
| 3 | Measure deviation | 3.5 kHz ± 10% | [ ] |
| 4 | Measure bandwidth | <16 kHz | [ ] |
| 5 | Measure cadence | 60-120s | [ ] |
| 6 | Verify callsign | Per regulations | [ ] |

**Success Criteria**: All RF parameters within IARU limits

## 6. Phase 3: Nominal Operations Validation

### 6.1 Nominal Ops ConOps Elements

| ConOps Element | Validation Scenario | Status |
|----------------|---------------------|--------|
| Smart Beacon Mode (adaptive) | VAL-NOM-001 | [ ] |
| Sunlit + SOC threshold | VAL-NOM-002 | [ ] |
| Jetson bursts for text | VAL-NOM-003 | [ ] |
| Eclipse/low-SOC behavior | VAL-NOM-004 | [ ] |
| Watchdog operation | VAL-NOM-005 | [ ] |
| Duty-cycle and thermal gates | VAL-NOM-006 | [ ] |

### 6.2 Nominal Operations Validation Scenarios

#### VAL-NOM-001: Smart Beacon Mode

**Objective**: Verify adaptive beacon behavior

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Set SOC = 80% | Above threshold | [ ] |
| 2 | Set sunlit condition | Sun flag set | [ ] |
| 3 | Request beacon | AI content | [ ] |
| 4 | Reduce SOC to 25% | Below threshold | [ ] |
| 5 | Request beacon | Template content | [ ] |
| 6 | Verify cadence change | Period increased | [ ] |

**Success Criteria**: Beacon adapts to power state

#### VAL-NOM-003: AI Beacon Generation

**Objective**: Verify end-to-end AI beacon

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Enable AI mode | Jetson powered | [ ] |
| 2 | Wait for ready | JETSON = READY | [ ] |
| 3 | Collect telemetry context | Data assembled | [ ] |
| 4 | Request AI generation | Text generated | [ ] |
| 5 | Format beacon | AX.25 frame | [ ] |
| 6 | Transmit and decode | Content valid | [ ] |
| 7 | Verify duty cycle | <10% | [ ] |

**Success Criteria**: AI-generated beacon received and decoded

#### VAL-NOM-004: Eclipse Behavior

**Objective**: Verify eclipse power management

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Simulate eclipse entry | Sun flag = 0 | [ ] |
| 2 | Monitor Jetson | Powers off | [ ] |
| 3 | Monitor beacon | Template mode | [ ] |
| 4 | Monitor cadence | Increased | [ ] |
| 5 | Simulate eclipse exit | Sun flag = 1 | [ ] |
| 6 | Verify recovery | Normal ops | [ ] |

**Success Criteria**: Correct power management during eclipse

## 7. Phase 4: Mode Transitions Validation

### 7.1 Mode Transition ConOps Elements

| Transition | Trigger | Validation Scenario |
|------------|---------|---------------------|
| Launch-Safe → Detumble | Deploy switch | VAL-MODE-001 |
| Detumble → Commissioning | Rates stable | VAL-MODE-002 |
| Commissioning → Nominal-Idle | Checkout complete | VAL-MODE-003 |
| Nominal-Idle ↔ Nominal-Active | Power available | VAL-MODE-004 |
| Any → Safe-Mode | Fault detected | VAL-MODE-005 |

### 7.2 Mode Transition Validation Scenarios

#### VAL-MODE-001: Deployment Transition

**Objective**: Verify deployment trigger

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Boot with switch pressed | Launch-Safe mode | [ ] |
| 2 | Release switch | 30-min timer starts | [ ] |
| 3 | Wait 30 minutes | Transition to Detumble | [ ] |
| 4 | Verify beacon | Begins after delay | [ ] |

**Success Criteria**: Correct deployment sequence timing

#### VAL-MODE-005: Fault Trigger Safe Mode

**Objective**: Verify fault causes safe mode entry

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Normal operation | ACTIVE mode | [ ] |
| 2 | Inject low voltage | V < threshold | [ ] |
| 3 | Monitor mode | → SAFE | [ ] |
| 4 | Verify beacon | Template, slow cadence | [ ] |
| 5 | Verify power | Minimum load | [ ] |

**Success Criteria**: Automatic safe mode on fault

## 8. Contingency Validation

### 8.1 Contingency ConOps Elements

| Contingency | Validation Scenario | Status |
|-------------|---------------------|--------|
| Loss of attitude control | VAL-CON-001 | [ ] |
| Power shortfall | VAL-CON-002 | [ ] |
| Thermal over-limit | VAL-CON-003 | [ ] |
| RF anomaly | VAL-CON-004 | [ ] |

### 8.2 Contingency Validation Scenarios

#### VAL-CON-001: Loss of Attitude Control

**Objective**: Verify safe operation without attitude control

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Disable ADCS actuators | No control | [ ] |
| 2 | Verify mode | → SAFE | [ ] |
| 3 | Monitor beacon | Continues | [ ] |
| 4 | Verify power | Still positive | [ ] |

**Success Criteria**: Beacon continues despite attitude loss

#### VAL-CON-002: Power Shortfall

**Objective**: Verify power shortfall response

| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Reduce solar input | Low power | [ ] |
| 2 | Monitor SOC | Decreasing | [ ] |
| 3 | Cross threshold | → SAFE | [ ] |
| 4 | Verify Jetson | Disabled | [ ] |
| 5 | Verify cadence | Extended | [ ] |
| 6 | Restore power | Recovery | [ ] |

**Success Criteria**: Graceful degradation and recovery

## 9. Validation Summary Matrix

### 9.1 Complete Validation Matrix

| Phase | Scenarios | Validated | Pending |
|-------|-----------|-----------|---------|
| LEOP | 6 | [ ] | [ ] |
| Commissioning | 6 | [ ] | [ ] |
| Nominal Ops | 6 | [ ] | [ ] |
| Mode Transitions | 5 | [ ] | [ ] |
| Contingencies | 4 | [ ] | [ ] |
| **Total** | **27** | **0** | **27** |

### 9.2 Validation Closure

```
CONOPS VALIDATION CLOSURE

All ConOps elements validated: [ ] YES  [ ] NO

Open items: ___

Deviations from ConOps: ___

Validation Complete Date: ___

Systems Engineer: ___________ Date: ___
Mission Director: ___________ Date: ___
QA: ___________ Date: ___
```

## 10. Traceability

### 10.1 Traceability to ConOps

| ConOps Section | Validation Scenarios |
|----------------|---------------------|
| §2 Operational Phases | VAL-LEOP-*, VAL-COM-*, VAL-NOM-* |
| §3 Modes of Operation | VAL-MODE-* |
| §4 Communications | VAL-COM-005, VAL-NOM-003 |
| §5 Autonomy/Power | VAL-NOM-001 through VAL-NOM-006 |
| §7 Contingencies | VAL-CON-* |

### 10.2 Traceability to Test Procedures

| Validation | Test Procedure |
|------------|----------------|
| VAL-LEOP-003 | TP-SYS-001 |
| VAL-NOM-003 | E2E-003 |
| VAL-MODE-005 | TP-FLATSAT-005 |
| VAL-CON-002 | TP-EPS-002 |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
