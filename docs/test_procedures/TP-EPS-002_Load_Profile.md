# Test Procedure: EPS Load Profile Test
## SMART-QSO CubeSat Mission

**Document Number:** TP-EPS-002
**Revision:** D
**Date:** 2026-10-27
**Reference:** GSFC-STD-7000, NASA Ames SmallSat Heritage

**Requirements verified:** MRD-050, SYS-111, SYS-112, SYS-114, SYS-121, SYS-211, SYS-304, SRS-F010

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. Purpose

This procedure verifies the Electrical Power Subsystem (EPS) performance under mission-representative load profiles, including transient loads, mode transitions, and eclipse/sunlight cycling.

## 2. Scope

- Mission power profile simulation
- Power budget margin verification
- Load transient response
- Battery SOC tracking accuracy
- Power mode transition testing

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SMART-QSO-EPS-001 | EPS Design Document (`docs/hardware/EPS_DESIGN.md`) |
| SQSO-REQ-002 | System Requirements Specification |
| SMART-QSO-SDD-001 | Software Design Document |
| SQSO-PBA-001 | Power Budget Analysis (`docs/environment/POWER_BUDGET_ANALYSIS.md`) |
| EEE-INST-002 | EEE Parts Derating |

## 4. Test Article

| Item | Description |
|------|-------------|
| Unit Under Test | EPS Assembly (EM or Flight) |
| Configuration | Integrated with OBC and loads |
| S/N | |
| Software Version | |

## 5. Test Equipment

| Equipment | Model | Cal Due |
|-----------|-------|---------|
| Programmable Power Supply | Keithley 2230G | |
| Electronic Load | BK Precision 8600 | |
| Data Acquisition | NI DAQ-6008 | |
| Oscilloscope | Keysight DSOX2024A | |
| Multimeter | Keysight 34465A | |
| Temperature Chamber | (if thermal cycling) | |

## 6. Preconditions

- [ ] EPS functional test (TP-EPS-001) completed and passed
- [ ] Battery fully charged to the flight set point, 4.10 V/cell (16.4 V; decision D8)
- [ ] All test equipment calibrated
- [ ] OBC software loaded and verified
- [ ] Load simulators configured

## 7. Safety

- Battery cells may become hot during discharge testing
- Monitor cell temperatures continuously
- Abort if any cell exceeds 45C
- Ensure adequate ventilation
- Have fire extinguisher accessible

---

## 8. Procedure

### 8.1 Test Setup

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.1.1 | Configure solar simulator for 1.5W output | Stable current | |
| 8.1.2 | Connect EPS to OBC | Communication established | |
| 8.1.3 | Connect electronic loads to all rails | Loads controllable | |
| 8.1.4 | Enable telemetry logging | Data recording active | |
| 8.1.5 | Verify initial battery SOC | >95% | |

### 8.2 Safe Mode Load Profile [SYS-211]

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.2.1 | Set total load to the SAFE load, 0.79 W (`docs/BASELINE_PARAMETERS.md` §5, heater included) | Stable voltage on all rails | |
| 8.2.2 | Maintain for 10 minutes | No anomalies | |
| 8.2.3 | Record power consumption | Record against 0.79 W | |
| 8.2.4 | Verify beacon transmission | Every 180 seconds (SAFE; SYS-211) | |
| 8.2.5 | Record battery voltage trend | Stable or rising | |

### 8.3 Idle Mode Load Profile [SYS-211]

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.3.1 | Set total load to the IDLE load, 1.18 W (BASELINE §5) | Stable voltage on all rails | |
| 8.3.2 | Maintain for 10 minutes | No anomalies | |
| 8.3.3 | Record power consumption | Record against 1.18 W | |
| 8.3.4 | Verify beacon timing | Every 120 seconds (IDLE; SYS-211) | |
| 8.3.5 | Record battery voltage trend | Stable | |

### 8.4 Active Mode Load Profile

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.4.1 | Set total load to the ACTIVE load at the duty limits, 2.60 W sun-pointed bottom-up (BASELINE §5) | Stable voltage on all rails | |
| 8.4.2 | Maintain for 10 minutes | No anomalies | |
| 8.4.3 | Record power consumption | Record against 2.60 W | |
| 8.4.4 | Verify all subsystems active | Telemetry nominal | |
| 8.4.5 | Record battery voltage trend | Slight decline acceptable | |

### 8.5 Jetson Burst Load Profile (in ACTIVE) [SYS-121]

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.5.1 | Add 8.0 W on VBAT_JET (the Jetson at its carrier-input limit, BASELINE §6) to the ACTIVE load | Stable voltage on all rails | |
| 8.5.2 | Maintain for 10 minutes (one burst, BASELINE §6) | No voltage sag below limits; bus above the 13.6 V hardware shed | |
| 8.5.3 | Record peak current draw | Record; VBAT_JET ≤ 0.62 A at 8 W, below the 1.20 A eFuse (`docs/hardware/EPS_DESIGN.md` §4) | |
| 8.5.4 | Verify 5V_PA and 3V3_AO stability | ± 2 % (EPS_DESIGN §4) | |
| 8.5.5 | Return to Active mode | Smooth transition | |

### 8.6 Load Transient Response

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.6.1 | Start in Idle mode | Stable | |
| 8.6.2 | Step load from IDLE (1.18 W) to IDLE plus an 8 W Jetson burst | Voltage dip <10%; bus above 12.0 V, no OBC reset (V-EMC-6) | |
| 8.6.3 | Measure recovery time | <100ms to steady state | |
| 8.6.4 | Step the Jetson burst off | Voltage spike <10% | |
| 8.6.5 | Measure settling time | <100ms | |

### 8.7 Eclipse Simulation [SYS-111, SYS-112]

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.7.1 | Start with solar input on | Battery charging | |
| 8.7.2 | Record initial battery voltage | |V| = ____V | |
| 8.7.3 | Disable solar input (eclipse entry) | Battery discharging | |
| 8.7.4 | Maintain eclipse for 36 minutes (β = 0, BASELINE §8) in IDLE | Voltage within limits | |
| 8.7.5 | Record battery voltage at end | |V| = ____V | |
| 8.7.6 | Re-enable solar input (eclipse exit) | Charging resumes | |
| 8.7.7 | Monitor recovery | SOC increasing | |
| 8.7.8 | Calculate DOD during eclipse | ≤ 10 % nominal (SYS-111, SYS-112); never above 30 % | |

### 8.8 Multi-Orbit Simulation [MRD-050, SYS-304]

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.8.1 | Configure the 475 km orbit profile, 94.1 min (SYS-012; TP-SYS-002 §3.1) | 36 min eclipse, 58 min sun | |
| 8.8.2 | Execute 4 complete orbits | ~6 hours | |
| 8.8.3 | Record battery voltage profile | Document | |
| 8.8.4 | Verify power mode transitions | Automatic per power state | |
| 8.8.5 | Verify end-of-test SOC | >80% (positive margin) | |

### 8.9 Battery SOC Tracking [SYS-114, SRS-F010]

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.9.1 | Compare OBC SOC reading to actual | Within 5% | |
| 8.9.2 | Verify coulomb counter accuracy | Within 3% | |
| 8.9.3 | Verify voltage-based estimation | Within 5% | |
| 8.9.4 | Test SOC at various temperatures | Correction applied | |

---

## 9. Pass/Fail Criteria

| Criterion | Requirement | Result |
|-----------|-------------|--------|
| Voltage regulation | All rails within spec | |
| Transient response | <10% deviation, <100ms recovery | |
| Eclipse survival | ≤ 10 % DOD per eclipse nominal, ≤ 30 % contingency (SYS-112) | |
| Multi-orbit margin | Positive power margin | |
| SOC tracking | Within 5% of actual | |
| Mode transitions | Smooth, no glitches | |

## 10. Data Recording

### 10.1 Required Data

- [ ] Continuous voltage logging (all rails)
- [ ] Continuous current logging (all rails)
- [ ] Battery cell voltages and temperatures
- [ ] OBC telemetry log
- [ ] Mode transition timestamps
- [ ] Anomaly observations

### 10.2 Data Files

| File | Description |
|------|-------------|
| EPS-002_voltage.csv | Voltage time series |
| EPS-002_current.csv | Current time series |
| EPS-002_battery.csv | Battery parameters |
| EPS-002_telemetry.log | OBC telemetry |

---

## 11. Post-Test

- [ ] Return to safe mode
- [ ] Recharge battery to >95% SOC
- [ ] Review all recorded data
- [ ] Complete test report
- [ ] Note any anomalies

---

## 12. Test Log

| Date | Time | Operator | Notes |
|------|------|----------|-------|
| | | | |
| | | | |
| | | | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; it had cited only standards (finding VV-06) | SMART-QSO Team |
| C | 2026-10-16 | Steps 8.2.4 and 8.3.4 corrected to the SYS-211 beacon intervals (180 s in SAFE, 120 s in IDLE; were 60 s and 30 s) and cited (finding VV-06) | SMART-QSO Team |
| D | 2026-10-27 | Documentation audit (post-CDR 2026): mode loads from BASELINE §5 (0.79/1.18/2.60 W; were 0.5/1.5/3.0 W), Jetson ≤ 8 W for one ≤ 10 min burst, 94.1 min orbit with 36 min eclipse, DoD ≤ 10 %; document numbers corrected; banner | Documentation audit |
