# Test Procedure: Flight Configuration Verification
## TP-FAT-002

**Document Number:** SQSO-TP-FAT-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

This procedure verifies that the flight unit is configured correctly for launch and mission operations. This is the final configuration verification before delivery to the launch provider.

---

## 2. SCOPE

- Software version verification
- Configuration parameter verification
- Hardware configuration verification
- Inhibit state verification
- Documentation verification

---

## 3. APPLICABLE DOCUMENTS

1. Software Version Description (SVD)
2. Flight Configuration Baseline
3. As-Built Configuration List
4. CDS Rev 14.1

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Development PC | Ground station software |
| Serial interface | USB-UART adapter |
| Power supply | 4.0V battery simulation |
| RBF pin | Flight unit |
| Inspection tools | Torque wrench, magnifier |

---

## 5. PRECONDITIONS

- [ ] Final functional test complete (TP-FAT-001)
- [ ] All anomalies resolved
- [ ] Flight software loaded
- [ ] Flight configuration file loaded
- [ ] All waivers and deviations documented

---

## 6. PROCEDURE

### 6.1 Software Version Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on spacecraft | Boot complete | |
| 2 | Query OBC software version | Read version string | |
| 3 | Compare to SVD | Matches flight baseline | |
| 4 | Query OBC software CRC | Read CRC value | |
| 5 | Compare to SVD | Matches flight baseline | |
| 6 | Query Jetson software version | Read version | |
| 7 | Compare to SVD | Matches flight baseline | |
| 8 | Document all versions | Record in checklist | |

### 6.2 Configuration Parameter Verification

#### 6.2.1 Identification Parameters

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| Callsign | (Assigned callsign) | | |
| NORAD ID | (If assigned) | | |
| Mission ID | SMART-QSO | | |
| Serial Number | FM-001 | | |

#### 6.2.2 RF Configuration

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| TX Frequency | 145.825 MHz | | |
| TX Power (Low) | 0.5W | | |
| TX Power (High) | 2.0W | | |
| Beacon Interval (Normal) | 60 s | | |
| Beacon Interval (Low Power) | 120 s | | |
| Modulation | AFSK 1200 | | |

#### 6.2.3 Power Management Configuration

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| Battery Low Threshold | 3.3V | | |
| Battery Critical Threshold | 3.1V | | |
| Safe Mode Threshold | 10% SOC | | |
| Jetson Enable Threshold | 50% SOC | | |
| Load Shed Sequence | Per FDIR | | |

#### 6.2.4 ADCS Configuration

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| Detumble Threshold | 2°/s | | |
| B-dot Gain | (Calculated) | | |
| Control Mode | Sun-pointing | | |
| Magnetometer Calibration | Applied | | |

#### 6.2.5 Deployment Configuration

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| Deployment Timer | 30 minutes | | |
| Antenna Deploy First | Yes | | |
| Solar Deploy Second | Yes | | |
| Burn Wire Duration | 3 seconds | | |
| Retry Attempts | 3 | | |

### 6.3 Hardware Configuration Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify rail dimensions | 8.5×8.5 mm ±0.1 | |
| 2 | Verify overall dimensions | 100×100×113.5 mm | |
| 3 | Verify deployment switch location | Per drawing | |
| 4 | Verify RBF pin location | Per drawing | |
| 5 | Verify antenna stowage | Properly stowed | |
| 6 | Verify solar panel stowage | Properly stowed | |
| 7 | Verify all fasteners | Torqued and staked | |
| 8 | Verify all connectors | Properly mated | |
| 9 | Verify harness routing | No pinch points | |
| 10 | Verify markings | Visible and correct | |

### 6.4 Inhibit State Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify RBF pin installed | Pin in place | |
| 2 | Verify deployment inhibit | Status = inhibited | |
| 3 | Verify separation switch | Not actuated | |
| 4 | Remove RBF momentarily | Inhibit clears | |
| 5 | Re-install RBF pin | Inhibit active | |
| 6 | Verify RF inhibit (if applicable) | Per launch config | |

### 6.5 Pre-Ship Battery State

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure battery voltage | Record value | |
| 2 | Verify shipping SOC | 30-50% per guidelines | |
| 3 | Verify heater status | Off or as required | |
| 4 | Record battery temperature | Within safe range | |

### 6.6 Documentation Verification

| Document | Version | Verified | P/F |
|----------|---------|----------|-----|
| Software Version Description | | | |
| As-Built Configuration List | | | |
| Test Report Summary | | | |
| Waiver/Deviation List | | | |
| ICD Compliance Matrix | | | |
| Environmental Test Report | | | |
| Mass Properties Report | | | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Software version | Matches SVD baseline |
| Configuration parameters | Match flight baseline |
| Hardware configuration | Matches as-built list |
| Inhibit states | Correct for shipping |
| Battery state | 30-50% SOC |
| Documentation | Complete and approved |

---

## 8. DATA RECORDING

- [ ] Software version record
- [ ] Configuration parameter dump
- [ ] Hardware inspection checklist
- [ ] Inhibit state verification
- [ ] Battery state record
- [ ] Documentation checklist
- [ ] Configuration photos

---

## 9. SHIP READINESS CHECKLIST

| Item | Verified |
|------|----------|
| All tests complete | |
| All anomalies resolved | |
| Configuration verified | |
| RBF pin installed | |
| Battery at shipping SOC | |
| Documentation package complete | |
| Shipping container ready | |

---

## 10. SIGNATURES

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Test Engineer | | | |
| Quality Assurance | | | |
| Configuration Manager | | | |
| Project Manager | | | |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
