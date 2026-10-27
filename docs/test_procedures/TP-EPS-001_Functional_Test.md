# Test Procedure: EPS Functional Test
## TP-EPS-001

**Document Number:** TP-EPS-001
**Revision:** C
**Date:** 2026-01-02
**Classification:** Development/Qualification Test

**Requirements verified:** SYS-102, SYS-113, SYS-120, SYS-121, SYS-122, SYS-123, SYS-124, SRS-F011, SRS-I011, SRS-I012

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
| SMART-QSO-EPS-001 | EPS Design Document (`docs/hardware/EPS_DESIGN.md`) |
| SQSO-EPS-BRD-001 | EPS Board: Build-to Design (`hardware/electrical/eps/EPS_BOARD_DESIGN.md`) |
| SMART-QSO-ICD-001 | OBC-EPS Interface Control Document |
| SMART-QSO-SRS-001 | Software Requirements Specification |

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

- **Electrical Hazard:** Voltages up to 16.4 V (4S battery charged to 4.10 V/cell, decision D8; 16.8 V absolute maximum)
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
| 4 | Connect electronic loads to 3V3_AO, 5V_PA, VBAT_JET and V_BURN (`docs/hardware/EPS_DESIGN.md` §4; there is no 12 V rail) | Loads connected | |
| 5 | Connect data acquisition to telemetry | DAQ recording | |
| 6 | Power on solar array simulator within the trackers' range (3.5–6.3 V, up to 1.1 A per string; V-EPS-1) | Power applied | |

### 8.2 Voltage Rail Verification [SYS-120, SYS-121]

| Step | Rail | Load | Expected V | Measured V | Tolerance | P/F |
|------|------|------|------------|------------|-----------|-----|
| 7 | 3V3_AO | 0mA | 3.30V | _______ | ±2% | |
| 8 | 3V3_AO | 100mA | 3.30V | _______ | ±2% | |
| 9 | 3V3_AO | 260mA (maximum steady) | 3.30V | _______ | ±2% | |
| 10 | 5V_PA | 0mA | 5.00V | _______ | ±2% | |
| 11 | 5V_PA | 350mA (0.5 W RF) | 5.00V | _______ | ±2% | |
| 12 | 5V_PA | 550mA (1 W RF) | 5.00V | _______ | ±2% | |
| 13 | V_BURN | 0mA | 7.04V | _______ | record | |
| 14 | V_BURN | 880mA (one resistor) | 7.04V | _______ | record | |

### 8.3 Load Switch Operation [SYS-122, SRS-F011]

| Step | Switch | Command | Expected | Measured | P/F |
|------|--------|---------|----------|----------|-----|
| 15 | Payload (SW1 + U19, PAYLOAD_EN and JETSON_PWR) | OFF | 0V output | _______ | |
| 16 | Payload | ON | VBAT output | _______ | |
| 17 | Payload | OFF | 0V output | _______ | |
| 18 | Radio (SW2, RADIO_EN) | OFF | 0V output | _______ | |
| 19 | Radio | ON | 3.3V output | _______ | |
| 20 | PA (SW3, RF_TX_EN) | ON, then OFF | 5V, then 0V | _______ | |
| 21 | ADCS (SW4, ADCS_EN) | OFF | 0V output | _______ | |
| 22 | ADCS | ON | 3.3V output | _______ | |
| 23 | Every switch at reset | Off (pull-down) (EPS-F007) | 0V output | _______ | |

### 8.4 Current Sensing Accuracy [SYS-124, SRS-I011]

| Step | Rail | Set Current | Measured (DMM) | Reported (TLM) | Error | P/F |
|------|------|-------------|----------------|----------------|-------|-----|
| 24 | 3V3_AO | 100mA | _______ | _______ | <5% | |
| 25 | 3V3_AO | 260mA | _______ | _______ | <5% | |
| 26 | 5V_PA | 350mA | _______ | _______ | <5% | |
| 27 | VBAT_JET | 620mA (8 W) | _______ | _______ | <5% | |
| 28 | Battery | 500mA | _______ | _______ | <5% | |

### 8.5 Battery Charging [SYS-113]

| Step | Action | Expected | Measured | P/F |
|------|--------|----------|----------|-----|
| 29 | Set SA sim to 2A max | SA providing power | _______ | |
| 30 | Verify charge current (CC phase) | I_chg ≤ 1A | _______ | |
| 31 | Verify charge voltage limit | V_chg ≤ 16.4 V (4.10 V/cell, decision D8) | _______ | |
| 32 | Verify charge taper (CV phase) | Current decreasing | _______ | |
| 33 | Verify charge termination | I_chg < 50mA | _______ | |

### 8.6 Battery Protection [SYS-113, SYS-123]

| Step | Action | Expected | Result | P/F |
|------|--------|----------|--------|-----|
| 34 | Simulate cell over-voltage above 4.25 V/cell (17.0 V pack) | Charge disabled (BASELINE §5) | _______ | |
| 35 | Simulate under-voltage: bus below 13.6 V, then below the 11.2 V BMS cut | Jetson rail shed in hardware below 13.6 V, PA not shed (EPS-F012); BMS cut at 11.2 V (EPS_DESIGN §4, §6) | _______ | |
| 36 | Simulate over-current (3A) | Current limit | _______ | |
| 37 | Simulate pack temperature above +45 °C, then below 0 °C | Charge disabled outside 0 °C to +45 °C (BASELINE §5; SYS-602) | _______ | |
| 38 | Clear fault conditions | Normal operation | _______ | |

### 8.7 MPPT Operation [SYS-102]

| Step | SA Condition | Expected Pmax | Measured | Efficiency | P/F |
|------|--------------|---------------|----------|------------|-----|
| 39 | One string at 6.3 V, 1.1 A | Record | _______ | >95% tracking; ≥ 92 % conversion above 2 W (V-EPS-1) | |
| 40 | One string at 5.0 V, 0.5 A | Record | _______ | >95% tracking; ≥ 90 % conversion above 1 W (V-EPS-1) | |
| 41 | One string at 3.5 V, 0.05 A | Record | _______ | >95% tracking | |

### 8.8 Telemetry Verification [SYS-124, SRS-I011, SRS-I012]

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
| B | 2026-10-16 | OBC-EPS ICD cited by its own number, SMART-QSO-ICD-001 (finding PI-17) | SMART-QSO Team |
| C | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) | SMART-QSO Team |
