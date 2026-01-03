# Test Procedure: Comprehensive Functional Test
## TP-SYS-001

**Document Number:** TP-SYS-001
**Revision:** A
**Date:** 2026-01-02
**Classification:** System Integration Test

---

## 1. Purpose

Verify the complete integrated functionality of the SMART-QSO spacecraft at the system level, including all subsystems operating together.

## 2. Scope

This comprehensive functional test verifies:
- All subsystem operation
- Inter-subsystem communication
- Power mode transitions
- Beacon generation and transmission
- ADCS operation
- AI payload integration
- Fault detection and response
- Telemetry generation

## 3. Test Article

| Item | Description |
|------|-------------|
| Unit Under Test | SMART-QSO Integrated Spacecraft |
| Serial Number | _____________ |
| Configuration | Flight or Proto-flight |
| Software Version | _____________ |

## 4. Test Equipment

| Equipment | Purpose |
|-----------|---------|
| GSE Power Supply | Simulate solar array/battery |
| RF Monitoring Station | Receive and decode beacons |
| Helmholtz Cage | ADCS testing (optional) |
| Thermal Chamber | Thermal testing (if applicable) |
| Data Acquisition System | Telemetry recording |
| Sun Simulator | ADCS sun sensor testing |

## 5. Preconditions

- [ ] Spacecraft fully integrated
- [ ] All subsystem-level tests passed
- [ ] Flight software loaded and verified
- [ ] GSE connected and verified
- [ ] Test environment prepared
- [ ] Test plan reviewed and approved

## 6. Procedure

### 6.1 Pre-Test Setup

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify spacecraft configuration | Per drawing | |
| 2 | Connect GSE power | Connections secure | |
| 3 | Connect RF monitoring | Receiver ready | |
| 4 | Initialize data acquisition | Recording active | |
| 5 | Verify ambient conditions | 20-25°C, <60% RH | |
| 6 | Photograph configuration | Photos archived | |

### 6.2 Power-On and Initialization

| Step | Action | Expected | Observed | P/F |
|------|--------|----------|----------|-----|
| 7 | Remove RBF pin (simulated) | Pin removed | | |
| 8 | Actuate deployment switches | Switches closed | | |
| 9 | Apply power via GSE | OBC boots | | |
| 10 | Verify boot sequence | LEDs indicate boot | | |
| 11 | Wait for initialization | 30 seconds | | |
| 12 | Verify safe mode entry | SAFE mode in TLM | | |
| 13 | Verify beacon inhibit active | No RF for 30 min | | |

### 6.3 Subsystem Health Check

| Step | Subsystem | Check | Expected | Observed | P/F |
|------|-----------|-------|----------|----------|-----|
| 14 | EPS | Battery voltage | 14.4-16.8V | | |
| 15 | EPS | 3.3V rail | 3.30V ±2% | | |
| 16 | EPS | 5.0V rail | 5.00V ±2% | | |
| 17 | EPS | Current draw | < 500mA (SAFE) | | |
| 18 | OBC | CPU temperature | 20-40°C | | |
| 19 | OBC | Memory usage | < 80% | | |
| 20 | OBC | Watchdog status | Active | | |
| 21 | ADCS | Magnetometer | Valid data | | |
| 22 | ADCS | Sun sensors | Valid data | | |
| 23 | RF | Subsystem status | Standby | | |
| 24 | Payload | Power status | Off (SAFE mode) | | |

### 6.4 First Beacon (After 30-min Inhibit)

| Step | Action | Expected | Observed | P/F |
|------|--------|----------|----------|-----|
| 25 | Wait for inhibit timer | 30 minutes | | |
| 26 | Verify first beacon TX | RF output detected | | |
| 27 | Decode beacon | AX.25 frame decoded | | |
| 28 | Verify callsign | [CALLSIGN] | | |
| 29 | Verify telemetry in beacon | All fields present | | |
| 30 | Record beacon interval | 120 sec (SAFE mode) | | |

### 6.5 Power Mode Transitions

| Step | Transition | Trigger | Expected | Observed | P/F |
|------|------------|---------|----------|----------|-----|
| 31 | SAFE → IDLE | SOC > 60% | Mode change | | |
| 32 | Verify IDLE beacon interval | 60 seconds | | | |
| 33 | Verify loads enabled | ADCS active | | | |
| 34 | IDLE → ACTIVE | SOC > 80% | Mode change | | |
| 35 | Verify ACTIVE beacon interval | 30 seconds | | | |
| 36 | Verify Jetson power | Payload enabled | | | |
| 37 | ACTIVE → IDLE | SOC < 70% | Mode change | | |
| 38 | IDLE → SAFE | SOC < 40% | Mode change | | |
| 39 | Verify load shedding | Payload off | | | |

### 6.6 ADCS Operation

| Step | Test | Expected | Observed | P/F |
|------|------|----------|----------|-----|
| 40 | Magnetometer readout | 3-axis data | | |
| 41 | Sun sensor readout | Valid when illuminated | | |
| 42 | Detumble mode activation | Mode active | | |
| 43 | Magnetorquer operation | Current flowing | | |
| 44 | ADCS telemetry | All fields present | | |
| 45 | Sun pointing (if applicable) | Attitude change | | |

### 6.7 AI Payload Operation

| Step | Test | Expected | Observed | P/F |
|------|------|----------|----------|-----|
| 46 | Jetson power-on | Boot sequence | | |
| 47 | OBC-Jetson communication | UART active | | |
| 48 | Health data to Jetson | JSON transmitted | | |
| 49 | AI-generated beacon text | Text received | | |
| 50 | Beacon with AI text | AI source indicated | | |
| 51 | Jetson power consumption | < 10W peak | | |
| 52 | Jetson thermal | < 70°C | | |
| 53 | Fallback to template | On Jetson failure | | |

### 6.8 Fault Injection Testing

| Step | Fault | Expected Response | Observed | P/F |
|------|-------|-------------------|----------|-----|
| 54 | Simulate low battery | SAFE mode entry | | |
| 55 | Simulate over-temperature | Load shedding | | |
| 56 | Simulate UART timeout | Template beacon | | |
| 57 | Simulate sensor failure | Default values used | | |
| 58 | Simulate watchdog timeout | Reset and recovery | | |
| 59 | Clear fault conditions | Normal operation | | |

### 6.9 Telemetry Verification

| Step | Parameter | Expected Range | Observed | P/F |
|------|-----------|----------------|----------|-----|
| 60 | Battery voltage | 14.4-16.8V | | |
| 61 | Solar array current | 0-2A | | |
| 62 | Total power | 0-15W | | |
| 63 | OBC temperature | 0-50°C | | |
| 64 | Battery temperature | 0-40°C | | |
| 65 | Reset counter | Incrementing | | |
| 66 | Beacon counter | Incrementing | | |
| 67 | Uptime counter | Incrementing | | |

### 6.10 Deployment Sequence (Inhibited)

| Step | Test | Expected | Observed | P/F |
|------|------|----------|----------|-----|
| 68 | Verify deployment timer | Active | | |
| 69 | Timer countdown | Correct | | |
| 70 | Deployment inhibit status | Armed | | |
| 71 | Verify no deployment | Inhibited | | |

### 6.11 End-of-Life Functions (Simulated)

| Step | Test | Expected | Observed | P/F |
|------|------|----------|----------|-----|
| 72 | EOL beacon message | Transmitted | | |
| 73 | Charge disable | Confirmed | | |
| 74 | RF disable | No transmission | | |
| 75 | Recovery from EOL | Nominal operation | | |

### 6.12 Extended Operation (Optional)

| Step | Duration | Test | Expected | P/F |
|------|----------|------|----------|-----|
| 76 | 1 hour | Continuous operation | Stable | |
| 77 | 4 hours | Extended operation | No anomalies | |
| 78 | 24 hours | Day-in-the-life | All nominal | |

### 6.13 Test Completion

| Step | Action | Result |
|------|--------|--------|
| 79 | Command safe mode | Complete |
| 80 | Disable RF | Complete |
| 81 | Download all test data | Complete |
| 82 | Power off spacecraft | Complete |
| 83 | Disconnect GSE | Complete |
| 84 | Final inspection | Complete |

## 7. Pass/Fail Criteria

**Pass:** All steps completed successfully, all parameters within specification.

**Fail:** Any critical function non-operational or any parameter outside limits.

## 8. Test Data

All data shall be archived including:
- Complete telemetry log
- Decoded beacons
- Anomaly observations
- Photographs
- Test operator notes

## 9. Anomaly Handling

Any anomaly during testing:
1. Document immediately
2. Assess severity (continue/stop)
3. Photograph if applicable
4. Report to test lead
5. Document in anomaly log

---

## Test Summary

| Section | Steps | Passed | Failed | N/A |
|---------|-------|--------|--------|-----|
| Power-On | 7 | | | |
| Health Check | 11 | | | |
| First Beacon | 6 | | | |
| Mode Transitions | 9 | | | |
| ADCS | 6 | | | |
| AI Payload | 8 | | | |
| Fault Injection | 6 | | | |
| Telemetry | 8 | | | |
| Deployment | 4 | | | |
| EOL | 4 | | | |
| Extended | 3 | | | |
| **Total** | **72** | | | |

---

## Test Execution Record

| Field | Entry |
|-------|-------|
| Date | _____________ |
| Start Time | _____________ |
| End Time | _____________ |
| Operator | _____________ |
| Witness | _____________ |
| Overall Result | PASS / FAIL |
| Signature | _____________ |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
