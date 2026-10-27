# SMART-QSO System Test Cases

**Document ID**: SQSO-STC-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft; not yet run (no CDR has been held; no hardware exists)

> **Test cases not yet run.** No hardware exists (`docs/BASELINE_PARAMETERS.md` §1); every result and signature field
> is blank. The "Requirement" cells cite IDs from an earlier numbering (SYS-001 to SYS-016, SW-, PL-, RF-, EPS-, SAF-,
> ICD-001 to -003) that do not carry those meanings in SQSO-REQ-002 or SMART-QSO-SRS-001; trace through
> `docs/verification/RTM.csv` and the VCRM. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended), which governs.

## 1. Purpose

This document defines the System Test Cases for SMART-QSO, providing detailed test specifications for verifying system-level requirements and validating operational capabilities per NPR 7123.1.

## 2. Test Case Organization

### 2.1 Test Categories

| Category | Prefix | Description |
|----------|--------|-------------|
| Functional | TC-FUN | Core functionality |
| Performance | TC-PER | Performance verification |
| Interface | TC-INT | Interface verification |
| Environmental | TC-ENV | Environmental tolerance |
| Safety | TC-SAF | Safety-critical functions |
| Stress | TC-STR | Boundary/stress conditions |

### 2.2 Test Priority

| Priority | Description |
|----------|-------------|
| P1 | Critical - Must pass for flight |
| P2 | High - Important for mission success |
| P3 | Medium - Operational capability |
| P4 | Low - Nice to have |

## 3. Functional Test Cases

### TC-FUN-001: System Boot

| Field | Value |
|-------|-------|
| **ID** | TC-FUN-001 |
| **Title** | System Boot and Initialization |
| **Priority** | P1 |
| **Requirement** | SYS-001, SW-001 |
| **Objective** | Verify system boots correctly from power-on |

**Preconditions**:
- System powered off
- All connections verified
- Test equipment ready

**Test Steps**:
| Step | Action | Expected Result | Pass |
|------|--------|-----------------|------|
| 1 | Apply power | Power rails stable | [ ] |
| 2 | Monitor boot | Boot messages on console | [ ] |
| 3 | Verify time | Boot < 30 seconds | [ ] |
| 4 | Check mode | MODE = DETUMBLE (then IDLE) | [ ] |
| 5 | Check telemetry | All fields valid | [ ] |
| 6 | Check watchdog | Kicking confirmed | [ ] |

**Pass Criteria**: Flight software running within 30 seconds (SYS-300)

---

### TC-FUN-002: State Machine Operation

| Field | Value |
|-------|-------|
| **ID** | TC-FUN-002 |
| **Title** | Operating Mode State Machine |
| **Priority** | P1 |
| **Requirement** | SYS-002, SW-002 |
| **Objective** | Verify all state transitions work correctly |

**Test Steps**:
| Step | Transition | Expected | Pass |
|------|------------|----------|------|
| 1 | SAFE → IDLE | Allowed | [ ] |
| 2 | IDLE → ACTIVE | Allowed | [ ] |
| 3 | ACTIVE → IDLE | Allowed | [ ] |
| 4 | IDLE → SAFE | Allowed | [ ] |
| 5 | SAFE → ACTIVE (direct) | Rejected | [ ] |
| 6 | Any → SAFE (fault) | Allowed | [ ] |

**Pass Criteria**: All valid transitions work, invalid rejected

---

### TC-FUN-003: Beacon Generation

| Field | Value |
|-------|-------|
| **ID** | TC-FUN-003 |
| **Title** | Beacon Generation and Transmission |
| **Priority** | P1 |
| **Requirement** | SYS-003, SYS-004 |
| **Objective** | Verify beacon generation and RF transmission |

**Test Steps**:
| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Enable beacon | Beacon started | [ ] |
| 2 | Receive on SDR | RF signal present | [ ] |
| 3 | Decode packet | AX.25 valid | [ ] |
| 4 | Check callsign | Correct ID | [ ] |
| 5 | Check content | Telemetry + message | [ ] |
| 6 | Time period | 60 / 120 / 180 s ± 5 s in ACTIVE / IDLE / SAFE (SYS-211) | [ ] |

**Pass Criteria**: Beacons transmitted and decoded correctly

---

### TC-FUN-004: AI Beacon Generation

| Field | Value |
|-------|-------|
| **ID** | TC-FUN-004 |
| **Title** | AI-Assisted Beacon Content |
| **Priority** | P2 |
| **Requirement** | SYS-005, PL-001 |
| **Objective** | Verify AI generates contextual beacon content |

**Test Steps**:
| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Enable ACTIVE mode | Mode confirmed | [ ] |
| 2 | Power on Jetson | Power enabled | [ ] |
| 3 | Wait for ready | JETSON_READY | [ ] |
| 4 | Send context | Data transmitted | [ ] |
| 5 | Receive response | AI text returned | [ ] |
| 6 | Transmit beacon | Jetson lines included after the OBC filter (gated experiment, D5) | [ ] |
| 7 | Verify power | ≤ 8 W; duty ≤ 5 % tumbling, ≤ 10 % sun-pointed (BASELINE §5, §6) | [ ] |

**Pass Criteria**: AI-generated content in beacon, duty cycle met

---

### TC-FUN-005: Telemetry Collection

| Field | Value |
|-------|-------|
| **ID** | TC-FUN-005 |
| **Title** | Housekeeping Telemetry Collection |
| **Priority** | P1 |
| **Requirement** | SYS-006, SW-003 |
| **Objective** | Verify all telemetry points are collected |

**Test Steps**:
| Step | Parameter | Verify | Pass |
|------|-----------|--------|------|
| 1 | Battery voltage | Valid range | [ ] |
| 2 | Battery current | Valid range | [ ] |
| 3 | SOC | 0-100% | [ ] |
| 4 | Temperatures | Valid range | [ ] |
| 5 | Mode | Correct state | [ ] |
| 6 | Uptime | Increasing | [ ] |
| 7 | Boot count | Stable | [ ] |

**Pass Criteria**: All telemetry parameters valid and updating

---

### TC-FUN-006: Command Processing

| Field | Value |
|-------|-------|
| **ID** | TC-FUN-006 |
| **Title** | Command Reception and Execution |
| **Priority** | P1 |
| **Requirement** | SYS-007, SW-004 |
| **Objective** | Verify commands are received and executed |

**Test Steps**:
| Step | Command | Expected | Pass |
|------|---------|----------|------|
| 1 | GET_TLM | Telemetry returned | [ ] |
| 2 | SET_MODE(IDLE) | Mode changed | [ ] |
| 3 | SET_BEACON(90) | Period changed (accepted range 10–120 s, SYS-211 note) | [ ] |
| 4 | GET_LOG | Log entries returned | [ ] |
| 5 | Invalid command | NAK returned | [ ] |

**Pass Criteria**: Valid commands executed, invalid rejected

---

### TC-FUN-007: Fault Detection and Response

| Field | Value |
|-------|-------|
| **ID** | TC-FUN-007 |
| **Title** | Fault Detection and Safe Mode |
| **Priority** | P1 |
| **Requirement** | SYS-008, SW-005 |
| **Objective** | Verify faults trigger safe mode |

**Test Steps**:
| Step | Fault | Expected Response | Pass |
|------|-------|-------------------|------|
| 1 | Low voltage | → SAFE | [ ] |
| 2 | Over temperature | Throttle/SAFE | [ ] |
| 3 | Watchdog timeout | Reset | [ ] |
| 4 | I2C failure | Fault logged | [ ] |
| 5 | AI timeout | Fallback mode | [ ] |

**Pass Criteria**: All faults detected and handled correctly

---

## 4. Performance Test Cases

### TC-PER-001: Power Budget Verification

| Field | Value |
|-------|-------|
| **ID** | TC-PER-001 |
| **Title** | Power Budget Verification |
| **Priority** | P1 |
| **Requirement** | SYS-010, EPS-001 |
| **Objective** | Verify power consumption meets budget |

**Test Steps**:
| Mode | Budget | Measured | Margin | Pass |
|------|--------|----------|--------|------|
| SAFE | 0.79 W | | | [ ] |
| IDLE | 1.18 W | | | [ ] |
| ACTIVE | 2.50 W sun-pointed, 1.82 W tumbling (duty limits) | | | [ ] |
| Jetson burst | ≤ 8 W at the carrier input | | | [ ] |
| Generation, EOL | 2.48 W tumbling, 3.84 W sun-pointed (margin ≥ 20 %) | | | [ ] |

**Pass Criteria**: All modes within power budget with margin

---

### TC-PER-002: Timing Requirements

| Field | Value |
|-------|-------|
| **ID** | TC-PER-002 |
| **Title** | System Timing Verification |
| **Priority** | P2 |
| **Requirement** | SW-010, SW-011 |
| **Objective** | Verify timing requirements met |

**Test Steps**:
| Timing | Requirement | Measured | Pass |
|--------|-------------|----------|------|
| Boot time | <30s | | [ ] |
| Mode transition | < 1 s (SRS-P004) | | [ ] |
| Beacon period | 60 / 120 / 180 s ± 5 s (SYS-211) | | [ ] |
| Command response | <1s | | [ ] |
| Watchdog timeout | 24.1–27.8 s (IWDG) | | [ ] |
| Jetson burst | ≤ 10 min including boot (SYS-502, SYS-512) | | [ ] |

**Pass Criteria**: All timing requirements met

---

### TC-PER-003: Data Rate Verification

| Field | Value |
|-------|-------|
| **ID** | TC-PER-003 |
| **Title** | Telemetry Data Rate |
| **Priority** | P2 |
| **Requirement** | RF-003 |
| **Objective** | Verify data rates achieved |

**Test Steps**:
| Configuration | Expected | Measured | Pass |
|---------------|----------|----------|------|
| AFSK 1200 | 1200 bps | | [ ] |
| Beacon size | <256 bytes | | [ ] |
| Daily volume | <252 KB | | [ ] |

**Pass Criteria**: Data rates meet specifications

---

## 5. Interface Test Cases

### TC-INT-001: OBC-EPS Interface

| Field | Value |
|-------|-------|
| **ID** | TC-INT-001 |
| **Title** | OBC to EPS Interface |
| **Priority** | P1 |
| **Requirement** | ICD-001 |
| **Objective** | Verify I2C interface to EPS |

**Test Steps**:
| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Scan I2C2 | EPS controller at 0x48 (OBC_BOARD_DESIGN) | [ ] |
| 2 | Read voltage | Valid value | [ ] |
| 3 | Read current | Valid value | [ ] |
| 4 | Read SOC | 0-100% | [ ] |
| 5 | Test polling rate | 10 Hz | [ ] |

**Pass Criteria**: All EPS telemetry accessible via I2C

---

### TC-INT-002: OBC-RF Interface

| Field | Value |
|-------|-------|
| **ID** | TC-INT-002 |
| **Title** | OBC to RF Interface |
| **Priority** | P1 |
| **Requirement** | ICD-002 |
| **Objective** | Verify the OBC to RF board interface (USART2 to the TNC, enables and readbacks; SQSO-ICX-001) |

**Test Steps**:
| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Query the TNC (STM32L431) | Identity and status returned | [ ] |
| 2 | Configure TX (LMX2571 exciter) | Success | [ ] |
| 3 | Send a frame to the TNC | Accepted | [ ] |
| 4 | Trigger TX | RF output | [ ] |
| 5 | Read status | TX complete | [ ] |

**Pass Criteria**: Full RF control over the RF board interface

---

### TC-INT-003: OBC-Jetson Interface

| Field | Value |
|-------|-------|
| **ID** | TC-INT-003 |
| **Title** | OBC to Jetson Interface |
| **Priority** | P1 |
| **Requirement** | ICD-003 |
| **Objective** | Verify UART interface to Jetson |

**Test Steps**:
| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Power enable | Jetson boots | [ ] |
| 2 | Wait ready | Status OK | [ ] |
| 3 | Send context | ACK received | [ ] |
| 4 | Wait response | Text returned | [ ] |
| 5 | Verify format | Protocol valid | [ ] |
| 6 | Power disable | Jetson off | [ ] |

**Pass Criteria**: Complete AI request/response cycle

---

## 6. Safety Test Cases

### TC-SAF-001: Watchdog Function

| Field | Value |
|-------|-------|
| **ID** | TC-SAF-001 |
| **Title** | Hardware Watchdog Function |
| **Priority** | P1 |
| **Requirement** | SAF-001, SW-020 |
| **Objective** | Verify watchdog resets on hang |

**Test Steps**:
| Step | Action | Expected | Pass |
|------|--------|----------|------|
| 1 | Normal operation | WDT kicked | [ ] |
| 2 | Halt OBC (debugger) | No kicks | [ ] |
| 3 | Wait 28 seconds | Reset occurs (IWDG 24.1–27.8 s) | [ ] |
| 4 | Verify boot | Boot counter +1 | [ ] |
| 5 | Verify mode | Reset cause logged; mode per the state machine | [ ] |

**Pass Criteria**: Watchdog resets the system within 27.8 seconds, before the TPS3850 supervisor's 29.3 s

---

### TC-SAF-002: Battery Protection

| Field | Value |
|-------|-------|
| **ID** | TC-SAF-002 |
| **Title** | Battery Protection Circuits |
| **Priority** | P1 |
| **Requirement** | SAF-002, EPS-010 |
| **Objective** | Verify battery protection active |

**Test Steps**:
| Step | Condition | Expected | Pass |
|------|-----------|----------|------|
| 1 | Overvoltage (> 4.25 V/cell; charge set point 4.10 V/cell) | Charge stops | [ ] |
| 2 | Undervoltage (bus < 13.6 V; pack < 11.2 V) | Jetson rail shed; BMS cut | [ ] |
| 3 | Overcurrent (> 4 A BMS; 5 A fuse) | Limit active | [ ] |
| 4 | Temperature outside 0 to +45 °C | Charge stops | [ ] |
| 5 | Software SOC limit (25 %) | SAFE mode | [ ] |

**Pass Criteria**: All battery protections function

---

### TC-SAF-003: RF Inhibit

| Field | Value |
|-------|-------|
| **ID** | TC-SAF-003 |
| **Title** | RF Transmission Inhibit |
| **Priority** | P1 |
| **Requirement** | SAF-003 |
| **Objective** | Verify RF inhibit functions |

**Test Steps**:
| Step | Condition | Expected | Pass |
|------|-----------|----------|------|
| 1 | Each of DS1, DS2, DS3 pressed alone | TX inhibited (three physical inhibits, SYS-225) | [ ] |
| 2 | RBF pin inserted | TX inhibited (not counted as an inhibit) | [ ] |
| 3 | TX_DISABLE latched | TX off in every mode, across resets (SYS-204) | [ ] |
| 4 | 45-min RF silence | TX allowed only after 45 min and after the AntS reports deployed (SYS-224; C-19) | [ ] |

**Pass Criteria**: All RF inhibits function correctly

---

## 7. Stress Test Cases

### TC-STR-001: Extended Operation

| Field | Value |
|-------|-------|
| **ID** | TC-STR-001 |
| **Title** | Extended Duration Operation |
| **Priority** | P2 |
| **Requirement** | SYS-015 |
| **Objective** | Verify stable operation over extended period |

**Test Configuration**: 168 hours (1 week) continuous operation

**Daily Checkpoints**:
| Day | Beacons | Resets | Anomalies | Pass |
|-----|---------|--------|-----------|------|
| 1 | | | | [ ] |
| 2 | | | | [ ] |
| 3 | | | | [ ] |
| 4 | | | | [ ] |
| 5 | | | | [ ] |
| 6 | | | | [ ] |
| 7 | | | | [ ] |

**Pass Criteria**: No anomalies, no unexpected resets

---

### TC-STR-002: Power Cycling

| Field | Value |
|-------|-------|
| **ID** | TC-STR-002 |
| **Title** | Power Cycle Endurance |
| **Priority** | P2 |
| **Requirement** | SYS-016 |
| **Objective** | Verify system survives power cycles |

**Test Steps**:
| Cycle | Power Off | Power On | Boot OK | Pass |
|-------|-----------|----------|---------|------|
| 1 | 5s | Boot | | [ ] |
| 10 | 5s | Boot | | [ ] |
| 50 | 5s | Boot | | [ ] |
| 100 | 5s | Boot | | [ ] |

**Pass Criteria**: 100 power cycles without failure

---

## 8. Test Execution Summary

### 8.1 Test Status Matrix

| Category | Total | Passed | Failed | Pending |
|----------|-------|--------|--------|---------|
| Functional | 7 | | | |
| Performance | 3 | | | |
| Interface | 3 | | | |
| Safety | 3 | | | |
| Stress | 2 | | | |
| **Total** | **18** | **0** | **0** | **18** |

### 8.2 Test Closure

```
SYSTEM TEST CLOSURE

All tests executed: [ ] YES  [ ] NO
All tests passed: [ ] YES  [ ] NO

Failed tests: ___
Waivers required: ___

Test Complete Date: ___

Test Lead: ___________ Date: ___
Systems Engineer: ___________ Date: ___
QA: ___________ Date: ___
```

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): status "CDR Baseline" replaced, not-yet-run banner (the requirement cells use an earlier numbering); values from BASELINE Rev E: mode powers (were 0.2/0.4/0.8/7 W), beacon 60/120/180 s, watchdog 24.1–27.8 s (was 5 s), battery protection 4.25 V/cell and 0–45 °C, three deployment switches and the 45-min silence (was 30 min); EPS controller at 0x48; RF board interface (was an SPI Si4463, not in the design); 9600 bps GFSK removed |
