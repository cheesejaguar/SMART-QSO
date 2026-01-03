# SMART-QSO Acceptance Test Procedure

**Document ID**: SQSO-ATP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the Acceptance Test Procedure (ATP) for SMART-QSO flight unit qualification per NASA-STD-7009 and NASA Ames SmallSat heritage practices. Successful completion of this procedure is required for Flight Readiness Review (FRR).

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| NPR 7120.5 | Program and Project Management |
| NASA-STD-7009 | Standard for Models and Simulations |
| SQSO-SRS-001 | System Requirements Specification |
| SQSO-VVM-001 | Verification & Validation Matrix |
| SQSO-ATP-ENV | Environmental Test Procedure |

## 3. Test Configuration

### 3.1 Unit Under Test

| Parameter | Value |
|-----------|-------|
| Unit Type | Flight |
| Serial Number | _______________ |
| Build Date | _______________ |
| Software Version | _______________ |
| Test Article Config | Per As-Built List |

### 3.2 Test Equipment

| Equipment | Cal ID | Cal Due | Status |
|-----------|--------|---------|--------|
| DC Power Supply | | | [ ] Ready |
| Digital Multimeter | | | [ ] Ready |
| Oscilloscope | | | [ ] Ready |
| RF Power Meter | | | [ ] Ready |
| Spectrum Analyzer | | | [ ] Ready |
| Thermal Chamber | | | [ ] Ready |
| EGSE Computer | | | [ ] Ready |
| SDR Receiver | | | [ ] Ready |

### 3.3 Test Environment

| Parameter | Requirement | Verified |
|-----------|-------------|----------|
| Temperature | 20-25°C | [ ] |
| Humidity | 30-60% RH | [ ] |
| Cleanliness | Class 100,000 | [ ] |
| ESD Control | <100V | [ ] |

## 4. Pre-Test Requirements

### 4.1 Documentation Review

| Document | Reviewed | Initials |
|----------|----------|----------|
| As-built configuration list | [ ] | |
| Assembly traveler complete | [ ] | |
| NCRs closed or waived | [ ] | |
| Previous test reports | [ ] | |
| Software release notes | [ ] | |

### 4.2 Visual Inspection

| Check | Pass | Fail | N/A |
|-------|------|------|-----|
| No visible damage | [ ] | [ ] | [ ] |
| All fasteners installed | [ ] | [ ] | [ ] |
| Connectors mated properly | [ ] | [ ] | [ ] |
| Harness routing correct | [ ] | [ ] | [ ] |
| Thermal interfaces installed | [ ] | [ ] | [ ] |
| Solar panels undamaged | [ ] | [ ] | [ ] |
| Antenna elements stowed | [ ] | [ ] | [ ] |
| RBF pin installed | [ ] | [ ] | [ ] |

**Visual Inspection**: [ ] PASS  [ ] FAIL

Inspector: _______________ Date: _______________

---

## 5. Test Procedures

### 5.1 Test 1: Electrical Power System

**Test ID**: ATP-001
**Duration**: 60 minutes
**Requirement**: EPS-001 through EPS-010

#### 5.1.1 Bus Voltage Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 1.1 | Connect EGSE power | Voltage stable | | [ ] |
| 1.2 | Measure 3.3V rail | 3.3V ± 3% | V | [ ] |
| 1.3 | Measure 5V rail | 5.0V ± 5% | V | [ ] |
| 1.4 | Measure battery voltage | 7.2-8.4V | V | [ ] |
| 1.5 | Verify no anomalies | No shorts/opens | | [ ] |

#### 5.1.2 Power Consumption Verification

| Mode | Budget | Measured | Margin | Pass |
|------|--------|----------|--------|------|
| SAFE | <0.2W | W | | [ ] |
| IDLE | <0.4W | W | | [ ] |
| ACTIVE | <0.8W | W | | [ ] |
| AI Burst | <7W | W | | [ ] |

#### 5.1.3 Solar Array Verification

| Panel | Isc (mA) | Voc (V) | Pass |
|-------|----------|---------|------|
| +X | | | [ ] |
| -X | | | [ ] |
| +Y | | | [ ] |
| -Y | | | [ ] |
| +Z | | | [ ] |
| -Z (if present) | | | [ ] |

#### 5.1.4 Battery Protection Test

| Protection | Threshold | Verified | Pass |
|------------|-----------|----------|------|
| Overvoltage | 4.2V/cell | | [ ] |
| Undervoltage | 2.5V/cell | | [ ] |
| Overcurrent | 2A | | [ ] |
| Over-temp | 45°C | | [ ] |

**EPS Test Result**: [ ] PASS  [ ] FAIL

---

### 5.2 Test 2: On-Board Computer

**Test ID**: ATP-002
**Duration**: 45 minutes
**Requirement**: SW-001 through SW-020

#### 5.2.1 Boot Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 2.1 | Power on OBC | Console output | | [ ] |
| 2.2 | Verify boot time | <30 seconds | s | [ ] |
| 2.3 | Check software version | Match manifest | | [ ] |
| 2.4 | Verify mode | SAFE | | [ ] |
| 2.5 | Check watchdog | Active | | [ ] |

#### 5.2.2 Memory Verification

| Memory | Test | Expected | Actual | Pass |
|--------|------|----------|--------|------|
| Flash | Read/Write | No errors | | [ ] |
| SRAM | Pattern test | No errors | | [ ] |
| FRAM | Persistence | Data retained | | [ ] |

#### 5.2.3 State Machine Verification

| Transition | Command | Expected | Verified | Pass |
|------------|---------|----------|----------|------|
| SAFE → IDLE | SET_MODE(IDLE) | Mode = IDLE | | [ ] |
| IDLE → ACTIVE | SET_MODE(ACTIVE) | Mode = ACTIVE | | [ ] |
| ACTIVE → IDLE | SET_MODE(IDLE) | Mode = IDLE | | [ ] |
| IDLE → SAFE | SET_MODE(SAFE) | Mode = SAFE | | [ ] |
| Invalid transition | SET_MODE(X) | Rejected | | [ ] |

#### 5.2.4 Telemetry Verification

| Parameter | Valid Range | Measured | Pass |
|-----------|-------------|----------|------|
| MODE | 0-4 | | [ ] |
| UPTIME | >0 | s | [ ] |
| BOOT_COUNT | ≥1 | | [ ] |
| BATT_V | 7.0-8.4V | V | [ ] |
| SOC | 0-100% | % | [ ] |
| TEMP_OBC | -40 to +85°C | °C | [ ] |

**OBC Test Result**: [ ] PASS  [ ] FAIL

---

### 5.3 Test 3: RF Subsystem

**Test ID**: ATP-003
**Duration**: 60 minutes
**Requirement**: RF-001 through RF-010

**WARNING**: Use dummy load for all TX tests. Ensure RF safety.

#### 5.3.1 Receiver Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 3.1 | Configure RX frequency | 435.XXX MHz | | [ ] |
| 3.2 | Inject test signal | -110 dBm | | [ ] |
| 3.3 | Verify AGC | Signal detected | | [ ] |
| 3.4 | Decode test packet | AX.25 valid | | [ ] |
| 3.5 | Measure sensitivity | <-115 dBm | dBm | [ ] |

#### 5.3.2 Transmitter Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 3.6 | Connect dummy load | Load verified | | [ ] |
| 3.7 | Enable TX | PTT active | | [ ] |
| 3.8 | Measure power | 0.5-2W | W | [ ] |
| 3.9 | Measure frequency | 145.XXX MHz ± 1kHz | Hz | [ ] |
| 3.10 | Measure spurious | <-40 dBc | dBc | [ ] |

#### 5.3.3 Beacon Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 3.11 | Enable beacon mode | Beacon starts | | [ ] |
| 3.12 | Receive on SDR | Signal detected | | [ ] |
| 3.13 | Decode AX.25 | Valid frame | | [ ] |
| 3.14 | Verify callsign | Correct ID | | [ ] |
| 3.15 | Verify content | Telemetry valid | | [ ] |
| 3.16 | Time 3 beacons | Period 60-120s | s | [ ] |

**RF Test Result**: [ ] PASS  [ ] FAIL

---

### 5.4 Test 4: ADCS Subsystem

**Test ID**: ATP-004
**Duration**: 45 minutes
**Requirement**: ADCS-001 through ADCS-010

#### 5.4.1 Sensor Verification

| Sensor | Reading | Valid Range | Pass |
|--------|---------|-------------|------|
| Magnetometer X | nT | ±60000 nT | [ ] |
| Magnetometer Y | nT | ±60000 nT | [ ] |
| Magnetometer Z | nT | ±60000 nT | [ ] |
| Sun sensor 1 | | 0-100% | [ ] |
| Sun sensor 2 | | 0-100% | [ ] |
| Gyro X | °/s | ±300°/s | [ ] |
| Gyro Y | °/s | ±300°/s | [ ] |
| Gyro Z | °/s | ±300°/s | [ ] |

#### 5.4.2 Actuator Verification

| Actuator | Command | Response | Pass |
|----------|---------|----------|------|
| Magnetorquer X+ | +100 | Current draws | [ ] |
| Magnetorquer X- | -100 | Current draws | [ ] |
| Magnetorquer Y+ | +100 | Current draws | [ ] |
| Magnetorquer Y- | -100 | Current draws | [ ] |
| Magnetorquer Z+ | +100 | Current draws | [ ] |
| Magnetorquer Z- | -100 | Current draws | [ ] |

#### 5.4.3 Control Mode Verification

| Mode | Test | Expected | Verified | Pass |
|------|------|----------|----------|------|
| B-dot | Apply field | Detumble response | | [ ] |
| Sun-pointing | Light source | Tracking | | [ ] |

**ADCS Test Result**: [ ] PASS  [ ] FAIL

---

### 5.5 Test 5: AI Payload

**Test ID**: ATP-005
**Duration**: 60 minutes
**Requirement**: PL-001 through PL-010

#### 5.5.1 Power-On Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 5.1 | Enable Jetson power | Power on | | [ ] |
| 5.2 | Wait for boot | <60 seconds | s | [ ] |
| 5.3 | Verify UART link | Messages received | | [ ] |
| 5.4 | Check status | READY | | [ ] |

#### 5.5.2 AI Inference Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 5.5 | Send context data | ACK received | | [ ] |
| 5.6 | Wait for response | <30 seconds | s | [ ] |
| 5.7 | Verify text format | Valid ASCII | | [ ] |
| 5.8 | Check length | ≤200 chars | chars | [ ] |

#### 5.5.3 Power Management Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 5.9 | Measure active power | <7W | W | [ ] |
| 5.10 | Command shutdown | Power off | | [ ] |
| 5.11 | Verify standby | <0.1W | W | [ ] |

#### 5.5.4 Thermal Verification

| Condition | Temperature | Limit | Pass |
|-----------|-------------|-------|------|
| Idle | °C | <85°C | [ ] |
| Active (30s) | °C | <95°C | [ ] |

**Payload Test Result**: [ ] PASS  [ ] FAIL

---

### 5.6 Test 6: Deployment Systems

**Test ID**: ATP-006
**Duration**: 30 minutes
**Requirement**: SAF-003, MECH-005

**CAUTION**: Wear safety glasses. Antenna may deploy unexpectedly.

#### 5.6.1 Inhibit Verification

| Inhibit | Test | Expected | Verified | Pass |
|---------|------|----------|----------|------|
| RBF pin | Pin installed | Deploy blocked | | [ ] |
| Deploy switch | Switch pressed | Deploy blocked | | [ ] |
| Software | Timer not expired | Deploy blocked | | [ ] |

#### 5.6.2 Deployment Readiness

| Check | Verified | Pass |
|-------|----------|------|
| Antenna stowed correctly | | [ ] |
| Burn resistor resistance | Ω | [ ] |
| Deployment harness continuity | | [ ] |
| Switch continuity | | [ ] |

**Note**: Actual deployment test performed separately per SQSO-TP-DEPLOY

**Deployment Test Result**: [ ] PASS  [ ] FAIL

---

### 5.7 Test 7: Thermal Control

**Test ID**: ATP-007
**Duration**: Variable (thermal soak)
**Requirement**: THERM-001 through THERM-005

#### 5.7.1 Ambient Temperature Verification

| Subsystem | Sensor | Temperature | Limit | Pass |
|-----------|--------|-------------|-------|------|
| OBC | T_OBC | °C | -40 to +85°C | [ ] |
| EPS | T_BATT | °C | 0 to +45°C | [ ] |
| RF PA | T_RF | °C | -40 to +70°C | [ ] |
| Jetson | T_JETSON | °C | -25 to +80°C | [ ] |

#### 5.7.2 Thermal Interface Verification

| Interface | Method | Verified | Pass |
|-----------|--------|----------|------|
| OBC to structure | Gap check | | [ ] |
| EPS to structure | Gap check | | [ ] |
| Jetson to radiator | Gap check | | [ ] |

**Thermal Test Result**: [ ] PASS  [ ] FAIL

---

### 5.8 Test 8: Fault Response

**Test ID**: ATP-008
**Duration**: 45 minutes
**Requirement**: SAF-001, SW-005

#### 5.8.1 Watchdog Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 8.1 | Normal operation | WDT kicked | | [ ] |
| 8.2 | Halt processor | WDT stops | | [ ] |
| 8.3 | Wait 5 seconds | Reset occurs | | [ ] |
| 8.4 | Verify boot | Boot count +1 | | [ ] |
| 8.5 | Verify mode | SAFE mode | | [ ] |

#### 5.8.2 Fault Injection Tests

| Fault | Response | Verified | Pass |
|-------|----------|----------|------|
| Low voltage (simulate) | → SAFE | | [ ] |
| High temperature (simulate) | Throttle | | [ ] |
| I2C timeout | Fault logged | | [ ] |
| UART timeout | Fallback mode | | [ ] |

#### 5.8.3 Recovery Verification

| Scenario | Action | Expected | Pass |
|----------|--------|----------|------|
| Power cycle | Off/On | Boot to SAFE | [ ] |
| Software reset | CMD_RESET | Boot to SAFE | [ ] |
| Watchdog reset | Halt | Auto-recovery | [ ] |

**Fault Test Result**: [ ] PASS  [ ] FAIL

---

### 5.9 Test 9: End-to-End Functional

**Test ID**: ATP-009
**Duration**: 120 minutes
**Requirement**: SYS-001 through SYS-015

#### 5.9.1 Operational Scenario

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 9.1 | Power on | Boot to SAFE | | [ ] |
| 9.2 | Transition to IDLE | Mode = IDLE | | [ ] |
| 9.3 | Enable beacon | Beacons start | | [ ] |
| 9.4 | Receive 3 beacons | All decoded | | [ ] |
| 9.5 | Send command | Response received | | [ ] |
| 9.6 | Transition to ACTIVE | Mode = ACTIVE | | [ ] |
| 9.7 | Enable AI | Jetson powers on | | [ ] |
| 9.8 | Generate AI beacon | Content valid | | [ ] |
| 9.9 | Disable AI | Jetson off | | [ ] |
| 9.10 | Return to IDLE | Mode = IDLE | | [ ] |
| 9.11 | Force SAFE mode | Mode = SAFE | | [ ] |
| 9.12 | Verify minimal power | <0.2W | | [ ] |

**End-to-End Test Result**: [ ] PASS  [ ] FAIL

---

## 6. Mass Properties

### 6.1 Mass Measurement

| Property | Requirement | Measured | Pass |
|----------|-------------|----------|------|
| Total mass | ≤2.0 kg | kg | [ ] |
| Mass margin | ≥5% | % | [ ] |

### 6.2 Center of Gravity

| Axis | Requirement | Measured | Pass |
|------|-------------|----------|------|
| CG-X | ±20 mm from geometric center | mm | [ ] |
| CG-Y | ±20 mm from geometric center | mm | [ ] |
| CG-Z | ±20 mm from geometric center | mm | [ ] |

**Mass Properties Result**: [ ] PASS  [ ] FAIL

---

## 7. Final Inspection

### 7.1 Workmanship

| Check | Pass | Fail | N/A |
|-------|------|------|-----|
| Solder joints per NASA-STD-8739.3 | [ ] | [ ] | [ ] |
| Wire routing per NASA-STD-8739.4 | [ ] | [ ] | [ ] |
| Mechanical assembly per drawings | [ ] | [ ] | [ ] |
| No FOD present | [ ] | [ ] | [ ] |
| All labels in place | [ ] | [ ] | [ ] |
| Access panels secure | [ ] | [ ] | [ ] |

### 7.2 Configuration Verification

| Item | As-Built | Matches Manifest | Pass |
|------|----------|------------------|------|
| OBC S/N | | | [ ] |
| EPS S/N | | | [ ] |
| RF S/N | | | [ ] |
| ADCS S/N | | | [ ] |
| Jetson S/N | | | [ ] |
| Battery S/N | | | [ ] |
| Structure S/N | | | [ ] |
| SW Version | | | [ ] |

**Final Inspection Result**: [ ] PASS  [ ] FAIL

---

## 8. Test Summary

### 8.1 Test Results Matrix

| Test | ID | Result | Anomalies |
|------|----|--------|-----------|
| EPS | ATP-001 | [ ] P [ ] F | |
| OBC | ATP-002 | [ ] P [ ] F | |
| RF | ATP-003 | [ ] P [ ] F | |
| ADCS | ATP-004 | [ ] P [ ] F | |
| Payload | ATP-005 | [ ] P [ ] F | |
| Deployment | ATP-006 | [ ] P [ ] F | |
| Thermal | ATP-007 | [ ] P [ ] F | |
| Fault | ATP-008 | [ ] P [ ] F | |
| End-to-End | ATP-009 | [ ] P [ ] F | |
| Mass Props | - | [ ] P [ ] F | |
| Final Insp | - | [ ] P [ ] F | |

### 8.2 Anomaly Summary

| Anomaly # | Description | Disposition |
|-----------|-------------|-------------|
| | | |
| | | |
| | | |

### 8.3 Test Closure

```
ACCEPTANCE TEST CLOSURE

Unit Serial Number: _______________

All tests completed: [ ] YES  [ ] NO
All tests passed: [ ] YES  [ ] NO

If NO, list failed tests: _______________

Waivers required: [ ] YES  [ ] NO
If YES, waiver numbers: _______________

Unit Status: [ ] FLIGHT READY  [ ] NOT READY

Reason if not ready: _______________

Test Conductor: _____________ Date: _______
QA Inspector: _____________ Date: _______
Systems Engineer: _____________ Date: _______
Project Manager: _____________ Date: _______
```

---

## 9. Attachments

### 9.1 Required Attachments

| Attachment | Included |
|------------|----------|
| Telemetry logs | [ ] |
| RF spectrum plots | [ ] |
| Thermal data | [ ] |
| Photos (pre/post) | [ ] |
| Equipment cal records | [ ] |

### 9.2 Reference Documents

| Document | Attached |
|----------|----------|
| As-built list | [ ] |
| NCR status | [ ] |
| Previous test results | [ ] |

---

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
