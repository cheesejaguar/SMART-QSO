# Test Procedure: Pre-Environmental Baseline
## SMART-QSO CubeSat Mission

**Document Number:** TP-ENV-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** GSFC-STD-7000 (GEVS), NASA Ames SmallSat Heritage

---

## 1. Purpose

This procedure establishes a pre-environmental test baseline for the SMART-QSO CubeSat. All measurements and functional tests in this procedure shall be repeated after environmental testing to verify no degradation.

## 2. Scope

- Mass and mass properties measurement
- Dimensional verification
- Complete functional test
- RF characterization baseline
- Visual inspection and documentation

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| GSFC-STD-7000 | General Environmental Verification Standard (GEVS) |
| Cal Poly CDS Rev 14.1 | CubeSat Design Specification |
| SQSO-REQ-001 | System Requirements |
| TP-SYS-001 | Comprehensive Functional Test |

## 4. Test Article

| Item | Description |
|------|-------------|
| Unit Under Test | SMART-QSO Flight Unit |
| Configuration | Flight configuration (antenna stowed) |
| S/N | |
| Software Version | |
| RBF Status | Installed |

## 5. Test Equipment

| Equipment | Model | Cal Due |
|-----------|-------|---------|
| Precision scale | 0.1g resolution | |
| CG measurement fixture | Project-specific | |
| Calipers | 0.01mm resolution | |
| Go/No-go gauge | 1U deployer mock | |
| Spectrum analyzer | TinySA Ultra | |
| Power meter | RF power meter | |
| Multimeter | Keysight 34465A | |
| Camera | High resolution | |

## 6. Preconditions

- [ ] All integration complete
- [ ] All subsystem functional tests passed
- [ ] Battery charged to nominal SOC
- [ ] All inhibits in place (RBF installed)
- [ ] Clean room environment (if required)

## 7. Safety

- Handle with ESD precautions
- Do not drop or impact unit
- Wear gloves to prevent contamination
- Secure antennas to prevent deployment

---

## 8. Procedure

### 8.1 Visual Inspection

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.1.1 | Inspect all external surfaces | No damage, debris, contamination | |
| 8.1.2 | Inspect rails | Smooth, correct finish | |
| 8.1.3 | Inspect deployment switches | Properly positioned | |
| 8.1.4 | Inspect RBF pin | Properly installed | |
| 8.1.5 | Inspect antenna stowage | Secure, no protrusions | |
| 8.1.6 | Inspect solar panels | No cracks, proper bonding | |
| 8.1.7 | Photograph all 6 faces | Document | |
| 8.1.8 | Photograph any anomalies | Document | |

### 8.2 Mass Measurement

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.2.1 | Zero scale | Reading = 0.0g | |
| 8.2.2 | Place unit on scale | Stable reading | |
| 8.2.3 | Record mass | Mass = ____g | |
| 8.2.4 | Verify mass < 2000g | Per CDS requirement | |
| 8.2.5 | Remove unit from scale | | |

**Mass Measurement Record:**

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| Total mass | < 2000g (1U) | g | |
| Mass margin | > 10% | % | |

### 8.3 Dimensional Verification

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.3.1 | Measure X dimension | 100mm 0.1mm | |
| 8.3.2 | Measure Y dimension | 100mm 0.1mm | |
| 8.3.3 | Measure Z dimension | 113.5mm +0/-0.1mm | |
| 8.3.4 | Verify rail dimensions | Per CDS spec | |
| 8.3.5 | Check protrusion limits | Max 6.5mm from rails | |
| 8.3.6 | Perform go/no-go gauge test | Smooth insertion | |

**Dimensional Record:**

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| X dimension | 100 0.1mm | mm | |
| Y dimension | 100 0.1mm | mm | |
| Z dimension | 113.5 +0/-0.1mm | mm | |
| Rail straightness | Per CDS | | |
| Surface finish | Per CDS | | |

### 8.4 Center of Gravity Measurement

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.4.1 | Set up CG measurement fixture | Level, calibrated | |
| 8.4.2 | Place unit in fixture (X orientation) | Stable | |
| 8.4.3 | Measure X CG offset | < 20mm from center | |
| 8.4.4 | Rotate for Y measurement | | |
| 8.4.5 | Measure Y CG offset | < 20mm from center | |
| 8.4.6 | Rotate for Z measurement | | |
| 8.4.7 | Measure Z CG offset | < 20mm from center | |

**CG Measurement Record:**

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| CG X offset | < 20mm | mm | |
| CG Y offset | < 20mm | mm | |
| CG Z offset | < 20mm | mm | |

### 8.5 Functional Test

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.5.1 | Remove RBF pin | Unit powers on | |
| 8.5.2 | Verify boot sequence | Complete in <30s | |
| 8.5.3 | Verify deployment inhibit | No deployment (switches held) | |
| 8.5.4 | Execute TP-SYS-001 abbreviated | All tests pass | |
| 8.5.5 | Verify beacon transmission | Correct format and timing | |
| 8.5.6 | Verify all telemetry channels | Within limits | |
| 8.5.7 | Test all operational modes | Transitions correct | |
| 8.5.8 | Reinstall RBF pin | Unit powers off | |

### 8.6 RF Characterization

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.6.1 | Connect RF test cable to TX port | Secure connection | |
| 8.6.2 | Command beacon transmission | TX active | |
| 8.6.3 | Measure center frequency | 145.825 MHz 5kHz | |
| 8.6.4 | Measure output power | 0.5W to 2W (as configured) | |
| 8.6.5 | Measure spurious emissions | >40dB below carrier | |
| 8.6.6 | Measure modulation deviation | 3.5kHz 0.5kHz | |
| 8.6.7 | Verify AX.25 decode | Clean decode | |
| 8.6.8 | Document spectrum plot | Screenshot | |

**RF Characterization Record:**

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| Center frequency | 145.825 MHz 5kHz | MHz | |
| Output power | 0.5-2W | W | |
| Spurious emissions | >40dB suppression | dB | |
| FM deviation | 3.5 0.5kHz | kHz | |
| Occupied bandwidth | <16 kHz | kHz | |

### 8.7 Power Characterization

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.7.1 | Measure battery voltage | 7.2-8.4V | |
| 8.7.2 | Measure safe mode current | <150mA | |
| 8.7.3 | Measure idle mode current | <500mA | |
| 8.7.4 | Measure active mode current | <1A | |
| 8.7.5 | Measure peak current (Jetson burst) | <2.5A | |

**Power Characterization Record:**

| Mode | Expected | Measured | P/F |
|------|----------|----------|-----|
| Safe | <0.6W | W | |
| Idle | <1.8W | W | |
| Active | <3.5W | W | |
| Burst | <10W | W | |

### 8.8 Documentation

| Step | Action | Expected Result | P/F |
|------|--------|-----------------|-----|
| 8.8.1 | Complete all data sheets | All fields filled | |
| 8.8.2 | Archive photographs | Labeled and stored | |
| 8.8.3 | Archive spectrum plots | Labeled and stored | |
| 8.8.4 | Generate test report | Signed | |

---

## 9. Pass/Fail Criteria

| Criterion | Requirement | Result |
|-----------|-------------|--------|
| Visual inspection | No damage or contamination | |
| Mass | < 2000g | |
| Dimensions | Per CDS Rev 14.1 | |
| CG location | < 20mm from geometric center | |
| Functional test | All tests pass | |
| RF characterization | All parameters within spec | |
| Power consumption | All modes within budget | |

## 10. Data Recording

### 10.1 Required Data

- [ ] Mass measurement (3 readings averaged)
- [ ] Dimensional measurements (all axes)
- [ ] CG measurements (all axes)
- [ ] Functional test results (per TP-SYS-001)
- [ ] RF spectrum plots
- [ ] Power consumption readings
- [ ] Photographs (all faces plus details)

### 10.2 Data Package

| Item | Filename |
|------|----------|
| Mass data | ENV-001_mass.csv |
| Dimensional data | ENV-001_dimensions.csv |
| CG data | ENV-001_cg.csv |
| Functional results | ENV-001_functional.pdf |
| RF plots | ENV-001_rf_spectrum.png |
| Power data | ENV-001_power.csv |
| Photos | ENV-001_photos/ |

---

## 11. Post-Test

- [ ] Reinstall RBF pin
- [ ] Return unit to storage configuration
- [ ] Complete and sign test report
- [ ] Archive all data
- [ ] Review results before proceeding to environmental test

---

## 12. Comparison Matrix (Post-Environmental)

This matrix shall be completed after environmental testing by comparing pre- and post-environmental results.

| Parameter | Pre-Env | Post-Env | Delta | Limit | P/F |
|-----------|---------|----------|-------|-------|-----|
| Mass | g | g | g | <5g | |
| X dimension | mm | mm | mm | <0.1mm | |
| Y dimension | mm | mm | mm | <0.1mm | |
| Z dimension | mm | mm | mm | <0.1mm | |
| CG X | mm | mm | mm | <2mm | |
| CG Y | mm | mm | mm | <2mm | |
| CG Z | mm | mm | mm | <2mm | |
| TX frequency | MHz | MHz | kHz | <2kHz | |
| TX power | W | W | dB | <1dB | |
| Safe mode power | W | W | % | <10% | |

---

## 13. Test Log

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
