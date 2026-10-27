# Test Procedure: Comprehensive Functional Test
## TP-SYS-001

**Document Number:** TP-SYS-001
**Revision:** B
**Date:** 2026-10-27
**Classification:** System Integration Test

**Requirements exercised:** SYS-211, SYS-221, SYS-224, SYS-304, SYS-503, SYS-521, SYS-800, SYS-801; the comprehensive performance test of the protoflight campaign (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §4) and the reference limits of VI-12

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

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
| 12 | Verify mode after boot | DETUMBLE in TLM (`software/rust/sqso-fsw/src/mode.rs`) | | |
| 13 | Verify RF silence active (`[RF_SILENCE] state=COUNTING` at boot) | No RF signal for 45 min (CDS §2.4.5, SYS-224); radio unpowered | | |

### 6.3 Subsystem Health Check

| Step | Subsystem | Check | Expected | Observed | P/F |
|------|-----------|-------|----------|----------|-----|
| 14 | EPS | Battery voltage | 11.2–16.4 V (4S, charged to 4.10 V/cell; BASELINE §5) | | |
| 15 | EPS | 3.3V rail | 3.30V ±2% | | |
| 16 | EPS | 5.0V rail | 5.00V ±2% | | |
| 17 | EPS | Power draw | Record against the SAFE load, 0.79 W (BASELINE §5) | | |
| 18 | OBC | CPU temperature | 20-40°C | | |
| 19 | OBC | Memory usage | < 80% | | |
| 20 | OBC | Watchdog status | Active | | |
| 21 | ADCS | Magnetometer | Valid data | | |
| 22 | ADCS | Sun sensors | Valid data | | |
| 23 | RF | Subsystem status | Standby | | |
| 24 | Payload | Power status | Off | | |

### 6.4 First Beacon (After the 45-min RF Silence)

| Step | Action | Expected | Observed | P/F |
|------|--------|----------|----------|-----|
| 25 | Wait for the RF silence; the wings deploy at 30 min and the AntS after them | No RF before 45 min (SYS-224), nor before the AntS reports deployed (decision C-19) | | |
| 26 | Verify first beacon TX | RF output detected | | |
| 27 | Decode beacon | AX.25 frame decoded | | |
| 28 | Verify callsign | KK6GDB-11 | | |
| 29 | Verify telemetry in beacon | All fields present | | |
| 30 | Record beacon interval | 180 s in SAFE, 120 s in IDLE (SYS-211) | | |

### 6.5 Power Mode Transitions

| Step | Transition | Trigger | Expected | Observed | P/F |
|------|------------|---------|----------|----------|-----|
| 31 | SAFE → IDLE | SOC > 40% | Mode change | | |
| 32 | Verify IDLE beacon interval | 120 seconds | | | |
| 33 | Verify loads enabled | ADCS active | | | |
| 34 | IDLE → ACTIVE | SOC ≥ 55 %, sunlit | Mode change | | |
| 35 | Verify ACTIVE beacon interval | 60 seconds | | | |
| 36 | Verify Jetson power | Payload enabled for a burst only when sunlit, SOC ≥ 55 % and Jetson ≥ −15 °C (BASELINE §6) | | | |
| 37 | ACTIVE → IDLE | SOC < 40% | Mode change | | |
| 38 | IDLE → SAFE | SOC < 25%; EMERGENCY (beacon shed) below 15 % (BASELINE §5) | Mode change | | |
| 39 | Verify load shedding | Payload off | | | |

### 6.6 ADCS Operation

| Step | Test | Expected | Observed | P/F |
|------|------|----------|----------|-----|
| 40 | Magnetometer readout | 3-axis data | | |
| 41 | Sun sensor readout | Valid when illuminated | | |
| 42 | Detumble mode activation | Mode active | | |
| 43 | Magnetorquer operation | Current flowing | | |
| 44 | ADCS telemetry | All fields present | | |
| 45 | Sun pointing (spin about +Y at 8°/s) | Commanded dipole sign per the law (TP-ADCS-003 §3.3) | | |

### 6.7 AI Payload Operation

| Step | Test | Expected | Observed | P/F |
|------|------|----------|----------|-----|
| 46 | Jetson power-on | Boot sequence | | |
| 47 | OBC-Jetson communication | UART active | | |
| 48 | Health data to Jetson | JSON transmitted | | |
| 49 | Beacon text from the OBC selector | Pre-approved template chosen (flight baseline, D5) | | |
| 50 | Gated experiment only: Jetson lines in the beacon | Filtered on the OBC (BEACON_SPEC §6); source indicated | | |
| 51 | Jetson power consumption | ≤ 8 W at the carrier input (SYS-501) | | |
| 52 | Jetson thermal | Junction ≤ 95 °C (SYS-603) | | |
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
| 60 | Battery voltage | 11.2–16.4 V | | |
| 61 | Solar array current | 0-2A | | |
| 62 | Total power | 0-12 W (ACTIVE with a Jetson burst) | | |
| 63 | OBC temperature | 0-50°C | | |
| 64 | Battery temperature | 0 to +45 °C for charging (BASELINE §5) | | |
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

The charge disconnect is permanent and non-resettable (SYS-800; `docs/BASELINE_PARAMETERS.md` §5). On the flight unit it
is exercised only up to the arm step, or with the one-shot replaced by a dummy load on the EM; it is never fired on the
flight unit.

| Step | Test | Expected | Observed | P/F |
|------|------|----------|----------|-----|
| 72 | Passivation arm (operational key) | Accepted; armed state in telemetry (SQSO-ICD-013 §6.4) | | |
| 73 | Passivation execute (recovery key), EM or dummy load only | Charge-disconnect drive seen at the dummy load; final frame carries the disconnect and fuse state | | |
| 74 | RF disable | No transmission; TX-OFF latched | | |
| 75 | Disarm (flight unit) or restore the dummy load (EM) | Passivation not armed; nominal operation | | |

### 6.12 Extended Operation (Optional)

| Step | Duration | Test | Expected | P/F |
|------|----------|------|----------|-----|
| 76 | 1 hour | Continuous operation | Stable | |
| 77 | 4 hours | Extended operation | No anomalies | |
| 78 | 24 hours | Day-in-the-life (TP-SYS-002) | All nominal | |

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
| B | 2026-10-27 | Documentation audit (post-CDR 2026): SOC thresholds and beacon intervals from BASELINE §5/§8 (were 60/80/70 % and 120/60/30 s), first beacon after the 45-min silence (was 30 min), 4S voltages, Jetson ≤ 8 W, EOL passivation never fired on the FM; banner | Documentation audit |
