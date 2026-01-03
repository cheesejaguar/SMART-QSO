# Test Procedure: EPS Load Profile Test
## SMART-QSO CubeSat Mission

**Document Number:** TP-EPS-002
**Revision:** A
**Date:** 2026-01-02
**Reference:** GSFC-STD-7000, NASA Ames SmallSat Heritage

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
| SQSO-HW-001 | EPS Design Document |
| SQSO-REQ-001 | System Requirements |
| SQSO-SW-001 | Software Design Document |
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
- [ ] Battery fully charged (>95% SOC)
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

### 8.2 Safe Mode Load Profile

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.2.1 | Set total load to 0.5W | Stable voltage on all rails | |
| 8.2.2 | Maintain for 10 minutes | No anomalies | |
| 8.2.3 | Record power consumption | <0.6W total | |
| 8.2.4 | Verify beacon transmission | Every 60 seconds | |
| 8.2.5 | Record battery voltage trend | Stable or rising | |

### 8.3 Idle Mode Load Profile

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.3.1 | Set total load to 1.5W | Stable voltage on all rails | |
| 8.3.2 | Maintain for 10 minutes | No anomalies | |
| 8.3.3 | Record power consumption | <1.8W total | |
| 8.3.4 | Verify beacon timing | Every 30 seconds | |
| 8.3.5 | Record battery voltage trend | Stable | |

### 8.4 Active Mode Load Profile

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.4.1 | Set total load to 3.0W | Stable voltage on all rails | |
| 8.4.2 | Maintain for 10 minutes | No anomalies | |
| 8.4.3 | Record power consumption | <3.5W total | |
| 8.4.4 | Verify all subsystems active | Telemetry nominal | |
| 8.4.5 | Record battery voltage trend | Slight decline acceptable | |

### 8.5 Burst Mode Load Profile (Jetson Active)

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.5.1 | Set total load to 8.0W | Stable voltage on all rails | |
| 8.5.2 | Maintain for 60 seconds | No voltage sag below limits | |
| 8.5.3 | Record peak current draw | <2.5A from battery | |
| 8.5.4 | Verify 5V rail stability | 5.0V 5% | |
| 8.5.5 | Return to Active mode | Smooth transition | |

### 8.6 Load Transient Response

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.6.1 | Start in Idle mode | Stable | |
| 8.6.2 | Step load from 1.5W to 8W | Voltage dip <10% | |
| 8.6.3 | Measure recovery time | <100ms to steady state | |
| 8.6.4 | Step load from 8W to 1.5W | Voltage spike <10% | |
| 8.6.5 | Measure settling time | <100ms | |

### 8.7 Eclipse Simulation

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.7.1 | Start with solar input on | Battery charging | |
| 8.7.2 | Record initial battery voltage | |V| = ____V | |
| 8.7.3 | Disable solar input (eclipse entry) | Battery discharging | |
| 8.7.4 | Maintain eclipse for 35 minutes | Voltage within limits | |
| 8.7.5 | Record battery voltage at end | |V| = ____V | |
| 8.7.6 | Re-enable solar input (eclipse exit) | Charging resumes | |
| 8.7.7 | Monitor recovery | SOC increasing | |
| 8.7.8 | Calculate DOD during eclipse | <30% DOD | |

### 8.8 Multi-Orbit Simulation

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.8.1 | Configure 90-minute orbit profile | 35min eclipse, 55min sun | |
| 8.8.2 | Execute 4 complete orbits | ~6 hours | |
| 8.8.3 | Record battery voltage profile | Document | |
| 8.8.4 | Verify power mode transitions | Automatic per power state | |
| 8.8.5 | Verify end-of-test SOC | >80% (positive margin) | |

### 8.9 Battery SOC Tracking

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
| Eclipse survival | <30% DOD per orbit | |
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
