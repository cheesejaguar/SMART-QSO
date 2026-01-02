# Test Procedure: EPS Functional Test
## TP-EPS-001

**Document Number:** TP-EPS-001
**Revision:** A
**Date:** 2026-01-02
**Classification:** Development/Qualification Test

---

## 1. Purpose

Verify the functional operation of the Electrical Power Subsystem (EPS) including voltage rails, load switches, current sensing, and battery management.

## 2. Scope

This procedure tests the EPS board in a standalone configuration with simulated loads and power sources. It verifies:
- All voltage rails
- Load switch operation
- Current sensing accuracy
- Battery charging and protection
- MPPT operation
- Telemetry accuracy

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-EPS-001 | EPS Design Document |
| SQSO-ICD-002 | OBC-EPS Interface Control Document |
| SQSO-SRS-001 | Software Requirements Specification |

## 4. Test Article

| Item | Description |
|------|-------------|
| Unit Under Test | EPS Board (EM or FM) |
| Serial Number | _____________ |
| Firmware Version | _____________ |
| Configuration | Standalone with test harness |

## 5. Test Equipment

| Equipment | Specification | Cal Due |
|-----------|---------------|---------|
| DC Power Supply | 0-30V, 5A | _______ |
| Solar Array Simulator | IV curve capable | _______ |
| Electronic Load | 0-5A, CC/CR modes | _______ |
| Digital Multimeter | 6.5 digit | _______ |
| Current Probe | DC-1MHz, 10mA resolution | _______ |
| Oscilloscope | 100MHz, 4 channel | _______ |
| Temperature Chamber | -40C to +85C | _______ |
| Data Acquisition | 16 channels, 1kHz | _______ |

## 6. Preconditions

- [ ] EPS board assembled and inspected
- [ ] Test harness connected and verified
- [ ] All test equipment calibrated
- [ ] Battery pack at 50% SOC
- [ ] Ambient temperature: 20-25°C
- [ ] Test data forms ready

## 7. Safety

- **Electrical Hazard:** Voltages up to 16.8V (4S battery)
- **Battery Hazard:** Li-ion cells, fire risk if shorted
- **Thermal Hazard:** Components may be hot during operation

**Required PPE:** Safety glasses, ESD wrist strap

**Emergency Procedures:**
- If smoke or fire: Disconnect power immediately, use fire extinguisher
- If battery swelling: Evacuate area, follow Li-ion emergency procedures

## 8. Procedure

### 8.1 Test Setup

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 1 | Connect EPS to test harness | All connections secure | |
| 2 | Connect solar array simulator to SA input | Simulator ready | |
| 3 | Connect battery pack to battery port | Battery connected | |
| 4 | Connect electronic loads to 3.3V, 5V, 12V rails | Loads connected | |
| 5 | Connect data acquisition to telemetry | DAQ recording | |
| 6 | Power on solar array simulator (Voc=8V, Isc=1A) | Power applied | |

### 8.2 Voltage Rail Verification

| Step | Rail | Load | Expected V | Measured V | Tolerance | P/F |
|------|------|------|------------|------------|-----------|-----|
| 7 | 3.3V | 0mA | 3.30V | _______ | ±2% | |
| 8 | 3.3V | 100mA | 3.30V | _______ | ±2% | |
| 9 | 3.3V | 500mA | 3.30V | _______ | ±3% | |
| 10 | 5.0V | 0mA | 5.00V | _______ | ±2% | |
| 11 | 5.0V | 200mA | 5.00V | _______ | ±2% | |
| 12 | 5.0V | 1000mA | 5.00V | _______ | ±3% | |
| 13 | 12V | 0mA | 12.0V | _______ | ±3% | |
| 14 | 12V | 100mA | 12.0V | _______ | ±3% | |

### 8.3 Load Switch Operation

| Step | Switch | Command | Expected | Measured | P/F |
|------|--------|---------|----------|----------|-----|
| 15 | Payload | OFF | 0V output | _______ | |
| 16 | Payload | ON | 5V output | _______ | |
| 17 | Payload | OFF | 0V output | _______ | |
| 18 | Radio | OFF | 0V output | _______ | |
| 19 | Radio | ON | 5V output | _______ | |
| 20 | Radio | OFF | 0V output | _______ | |
| 21 | ADCS | OFF | 0V output | _______ | |
| 22 | ADCS | ON | 3.3V output | _______ | |
| 23 | ADCS | OFF | 0V output | _______ | |

### 8.4 Current Sensing Accuracy

| Step | Rail | Set Current | Measured (DMM) | Reported (TLM) | Error | P/F |
|------|------|-------------|----------------|----------------|-------|-----|
| 24 | 3.3V | 100mA | _______ | _______ | <5% | |
| 25 | 3.3V | 500mA | _______ | _______ | <5% | |
| 26 | 5V | 200mA | _______ | _______ | <5% | |
| 27 | 5V | 1000mA | _______ | _______ | <5% | |
| 28 | Battery | 500mA | _______ | _______ | <5% | |

### 8.5 Battery Charging

| Step | Action | Expected | Measured | P/F |
|------|--------|----------|----------|-----|
| 29 | Set SA sim to 2A max | SA providing power | _______ | |
| 30 | Verify charge current (CC phase) | I_chg ≤ 1A | _______ | |
| 31 | Verify charge voltage limit | V_chg ≤ 16.8V | _______ | |
| 32 | Verify charge taper (CV phase) | Current decreasing | _______ | |
| 33 | Verify charge termination | I_chg < 50mA | _______ | |

### 8.6 Battery Protection

| Step | Action | Expected | Result | P/F |
|------|--------|----------|--------|-----|
| 34 | Simulate over-voltage (17.5V) | Charge disabled | _______ | |
| 35 | Simulate under-voltage (11.0V) | Load shed | _______ | |
| 36 | Simulate over-current (3A) | Current limit | _______ | |
| 37 | Simulate over-temperature (60°C) | Charge disabled | _______ | |
| 38 | Clear fault conditions | Normal operation | _______ | |

### 8.7 MPPT Operation

| Step | SA Condition | Expected Pmax | Measured | Efficiency | P/F |
|------|--------------|---------------|----------|------------|-----|
| 39 | Voc=8V, Isc=1A | ~6W | _______ | >95% | |
| 40 | Voc=6V, Isc=0.5A | ~2.25W | _______ | >95% | |
| 41 | Voc=10V, Isc=2A | ~15W | _______ | >95% | |

### 8.8 Telemetry Verification

| Step | Parameter | Reference | Reported | Error | P/F |
|------|-----------|-----------|----------|-------|-----|
| 42 | Battery voltage | DMM | TLM | <1% | |
| 43 | SA voltage | DMM | TLM | <1% | |
| 44 | SA current | Current probe | TLM | <5% | |
| 45 | 3.3V rail | DMM | TLM | <1% | |
| 46 | Temperature | Thermocouple | TLM | <2°C | |

### 8.9 Test Completion

| Step | Action | Result |
|------|--------|--------|
| 47 | Disable all loads | Complete |
| 48 | Disable solar array simulator | Complete |
| 49 | Disconnect battery | Complete |
| 50 | Download all test data | Complete |
| 51 | Power off all equipment | Complete |

## 9. Pass/Fail Criteria

**Pass:** All measured values within specified tolerances, all protection features functional.

**Fail:** Any measurement outside tolerance or protection feature non-functional.

## 10. Data Recording

All test data shall be recorded in the EPS Test Data Log with:
- Date and time
- Test article serial number
- Operator name
- All measured values
- Any anomalies observed
- Pass/fail determination

## 11. Post-Test

- [ ] All data recorded and verified
- [ ] Test equipment returned to storage
- [ ] Test article inspected for damage
- [ ] Test report completed
- [ ] Anomalies documented in anomaly log

---

## Test Execution Record

| Field | Entry |
|-------|-------|
| Date | _____________ |
| Operator | _____________ |
| Witness | _____________ |
| Overall Result | PASS / FAIL |
| Anomalies | _____________ |
| Comments | _____________ |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
