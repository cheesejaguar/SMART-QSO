# SMART-QSO Acceptance Test Procedure

**Document ID**: SQSO-ATP-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status:** Draft; not yet run (no hardware exists)

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

## 1. Purpose

This document defines the Acceptance Test Procedure (ATP) for the SMART-QSO flight unit. The flight unit is tested to
protoflight levels; there is no separate qualification unit (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §2, §4).
Successful completion of this procedure is required for the pre-ship review (`docs/integration/PRE_SHIP_CHECKLIST.md`).

> **Overlap.** The protoflight flow of `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §4 names TP-SYS-001, TP-FAT-001
> and TP-FAT-002 for the final functional and configuration checks, not this document. Where the two differ, those
> procedures govern. The requirement IDs below (EPS-001, SW-001, RF-001, ADCS-001, PL-001, SAF-003, MECH-005,
> THERM-001) are not defined in the requirements documents; trace through `docs/verification/VERIFICATION_MATRIX.md`.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| NPR 7120.5 | Program and Project Management |
| SQSO-REQ-002 | System Requirements Specification |
| SQSO-VCRM-001 | Verification Cross-Reference Matrix (`docs/verification/VERIFICATION_MATRIX.md`) |
| SQSO-ETP-001 | Environmental Test Plan (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md`) |

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
| 1.2 | Measure 3V3_AO | 3.3V ± 2% | V | [ ] |
| 1.3 | Measure 5V_PA | 5.0V ± 2% | V | [ ] |
| 1.4 | Measure battery voltage | 11.2–16.4 V (4S; BASELINE §5) | V | [ ] |
| 1.5 | Verify no anomalies | No shorts/opens | | [ ] |

#### 5.1.2 Power Consumption Verification

| Mode | Budget | Measured | Margin | Pass |
|------|--------|----------|--------|------|
| SAFE | 0.79 W (BASELINE §5) | W | | [ ] |
| IDLE | 1.18 W | W | | [ ] |
| ACTIVE | 2.60 W sun-pointed, 1.91 W tumbling (bottom-up) | W | | [ ] |
| Jetson burst | ≤ 8 W at the carrier input (SYS-501) | W | | [ ] |

#### 5.1.3 Solar Array Verification

| Panel | Isc (mA) | Voc (V) | Pass |
|-------|----------|---------|------|
| +X | | | [ ] |
| -X | | | [ ] |
| +Y | | | [ ] |
| -Y | | | [ ] |
| Wing 1 (both sides) | | | [ ] |
| Wing 2 (both sides) | | | [ ] |

#### 5.1.4 Battery Protection Test

| Protection | Threshold | Verified | Pass |
|------------|-----------|----------|------|
| Overvoltage | 4.25 V/cell (charge set point 4.10 V/cell, D8) | | [ ] |
| Undervoltage | 11.2 V pack BMS cut (EPS_DESIGN §4) | | [ ] |
| Overcurrent | BMS 4 A, fuse 5 A (EPS_DESIGN §5.1) | | [ ] |
| Charge temperature | Charging only 0 °C to +45 °C | | [ ] |

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
| 2.4 | Verify mode | DETUMBLE after boot | | [ ] |
| 2.5 | Check watchdog | Active | | [ ] |

#### 5.2.2 Memory Verification

| Memory | Test | Expected | Actual | Pass |
|--------|------|----------|--------|------|
| Flash | ECC status | No errors | | [ ] |
| SRAM2 | Parity status | No errors | | [ ] |
| FRAM (both copies) | Persistence | Data retained | | [ ] |

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
| MODE | 0-5 (BOOT, SAFE, DETUMBLE, IDLE, ACTIVE, EMERGENCY) | | [ ] |
| UPTIME | >0 | s | [ ] |
| BOOT_COUNT | ≥1 | | [ ] |
| BATT_V | 11.2–16.4 V | V | [ ] |
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
| 3.8 | Measure power | 0.5 W nominal, never above 1 W (SYS-201) | W | [ ] |
| 3.9 | Measure frequency | IARU-coordinated downlink frequency ± 5 kHz (SYS-200, SYS-206) | Hz | [ ] |
| 3.10 | Measure spurious | ≤ −46 dBc and ≤ 25 µW; third harmonic ≤ −70 dBc (SYS-203) | dBc | [ ] |

#### 5.3.3 Beacon Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 3.11 | Enable beacon mode | Beacon starts | | [ ] |
| 3.12 | Receive on SDR | Signal detected | | [ ] |
| 3.13 | Decode AX.25 | Valid frame | | [ ] |
| 3.14 | Verify callsign | KK6GDB-11 | | [ ] |
| 3.15 | Verify content | Telemetry valid | | [ ] |
| 3.16 | Time 3 beacons | 60 s ACTIVE, 120 s IDLE, 180 s SAFE (SYS-211) | s | [ ] |

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
| Sun-pointing | Light source | Commanded dipole sign per the spin law (TP-ADCS-003 §3.3) | | [ ] |

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
| 5.2 | Wait for boot | First STATUS line ≤ 45 s (V-JET-3) | s | [ ] |
| 5.3 | Verify UART link | Messages received | | [ ] |
| 5.4 | Check status | STATUS line received | | [ ] |

#### 5.5.2 AI Inference Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 5.5 | Send context data | ACK received | | [ ] |
| 5.6 | Wait for response | Within the burst: one power-on ≤ 10 min and ≤ 3.6 kJ (SYS-512) | s | [ ] |
| 5.7 | Verify text format | Valid ASCII; passes the BEACON_SPEC §6 filter (SYS-513) | | [ ] |
| 5.8 | Check length | ≤200 chars | chars | [ ] |

#### 5.5.3 Power Management Verification

| Step | Action | Expected | Actual | Pass |
|------|--------|----------|--------|------|
| 5.9 | Measure active power | ≤ 8 W at the carrier input (SYS-501) | W | [ ] |
| 5.10 | Command shutdown | Power off | | [ ] |
| 5.11 | Verify standby | <0.1W | W | [ ] |

#### 5.5.4 Thermal Verification

| Condition | Temperature | Limit | Pass |
|-----------|-------------|-------|------|
| Idle | °C | <85°C | [ ] |
| Active (one burst) | °C | Junction ≤ 95 °C (SYS-603) | [ ] |

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
| RBF pin | Pin installed | Deploy blocked (the RBF does not count as an inhibit, CDS 2.3.8) | | [ ] |
| Deployment switches DS1, DS2, DS3 | Each held alone | Deploy blocked by each (three physical inhibits, SYS-225) | | [ ] |
| Software | Timer not expired (30 min) | Deploy blocked | | [ ] |

#### 5.6.2 Deployment Readiness

| Check | Verified | Pass |
|-------|----------|------|
| Antenna (AntS) stowed, status read over I2C3 | | [ ] |
| Burn resistor resistance, four resistors (4–6.9 Ω) | Ω | [ ] |
| Deployment harness continuity | | [ ] |
| Switch continuity | | [ ] |

**Note**: Actual deployment test performed separately per TP-ENV-005 and TP-MECH-002, before the protoflight vibration

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
| Jetson | T_JETSON | °C | −15 °C (power-on gate) to +50 °C at the spreader (SYS-600) | [ ] |

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
| 8.3 | Wait 24.1–27.8 s (IWDG, TP-OBC-004) | Reset occurs | | [ ] |
| 8.4 | Verify boot | Boot count +1 | | [ ] |
| 8.5 | Verify mode | DETUMBLE after boot; reset cause logged | | [ ] |

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
| Power cycle | Off/On | Boot completes, mode per the state machine | [ ] |
| Software reset | CMD_RESET | Boot completes, mode per the state machine | [ ] |
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
| 9.1 | Power on | Boot completes | | [ ] |
| 9.2 | Transition to IDLE | Mode = IDLE | | [ ] |
| 9.3 | Enable beacon | Beacons start | | [ ] |
| 9.4 | Receive 3 beacons | All decoded | | [ ] |
| 9.5 | Send command | Response received | | [ ] |
| 9.6 | Transition to ACTIVE | Mode = ACTIVE | | [ ] |
| 9.7 | Enable AI | Jetson powers on | | [ ] |
| 9.8 | Beacon text from the OBC selector; Jetson lines only if the gated experiment is enabled | Content valid, filtered (BEACON_SPEC §6) | | [ ] |
| 9.9 | Disable AI | Jetson off | | [ ] |
| 9.10 | Return to IDLE | Mode = IDLE | | [ ] |
| 9.11 | Force SAFE mode | Mode = SAFE | | [ ] |
| 9.12 | Verify SAFE power | Record against 0.79 W | | [ ] |

**End-to-End Test Result**: [ ] PASS  [ ] FAIL

---

## 6. Mass Properties

### 6.1 Mass Measurement

| Property | Requirement | Measured | Pass |
|----------|-------------|----------|------|
| Total mass | ≤ 2.0 kg (SYS-002) and the dispenser limit; CBE 1,520 g, MEV 1,755 g (BASELINE §4) | kg | [ ] |
| Mass margin | Record against the 2.00 kg design limit | % | [ ] |

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
| Magnetometer board A2 S/N | | | [ ] |
| AntS S/N | | | [ ] |
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

Unit Status: [ ] READY FOR PRE-SHIP REVIEW  [ ] NOT READY

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
| 1.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): values brought to BASELINE Rev E (4S bus, mode powers, ≤ 1 W RF, beacon intervals, Jetson ≤ 8 W, three switches, OV 4.25 V/cell); overlap with TP-SYS-001/TP-FAT-001/-002 noted; nonexistent document numbers replaced; status and banner |
