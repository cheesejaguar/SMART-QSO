# Test Procedure: Flight Configuration Verification
## TP-FAT-002

**Document Number:** SQSO-TP-FAT-002
**Version:** 1.2
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-001, SYS-004, SYS-005, SYS-200, SYS-201, SYS-211, SYS-212, SYS-221, SYS-224, SYS-225, SRS-F013, SRS-F014, SRS-F015

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

1. Software Version Description (SVD), SQSO-SVD-001 (`docs/software/SOFTWARE_VERSION_DESCRIPTION.md`)
2. Flight Configuration Baseline (`docs/BASELINE_PARAMETERS.md`)
3. As-Built Configuration List
4. CDS Rev 14.1

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Development PC | Ground station software |
| Serial interface | USB-UART adapter |
| Power supply | 4S pack simulator, 11.2–16.4 V |
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

#### 6.2.1 Identification Parameters [SYS-212]

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| Callsign | KK6GDB-11 (`docs/BASELINE_PARAMETERS.md` §1) | | |
| NORAD ID | (If assigned) | | |
| Mission ID | SMART-QSO | | |
| Serial Number | FM-001 | | |

#### 6.2.2 RF Configuration [SYS-200, SYS-201, SYS-211]

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| TX Frequency | The IARU-coordinated downlink frequency (TBD until coordination, TBD-SYS-001; SYS-200) | | |
| TX Power (Nominal) | 0.5W | | |
| TX Power (Maximum) | ≤ 1.0 W (SYS-201) | | |
| Beacon Interval (ACTIVE) | 60 s | | |
| Beacon Interval (IDLE) | 120 s | | |
| Beacon Interval (SAFE) | 180 s | | |
| Modulation | AFSK 1200 | | |
| Loss-of-contact timeout N | 30 days at launch (set to 14 days after commissioning; range 1–90; `docs/BASELINE_PARAMETERS.md` §7, D3) | | |
| TX-OFF latch | Not latched (transmitters enabled for the post-silence beacon) | | |
| Telecommand keys | 4 operational keys (1 active, 3 spares) and the recovery key loaded through the umbilical (SYS-326; SQSO-ICD-013); counters at their provisioned values | | |

#### 6.2.3 Power Management Configuration [SRS-F013, SRS-F014, SRS-F015]

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| BMS under-voltage cut | 11.2 V pack (`docs/hardware/EPS_DESIGN.md` §4) | | |
| Jetson hardware load shed | Below 13.6 V (BASELINE §5) | | |
| Safe Mode Threshold | 25% SOC (SRS-F013) | | |
| Idle Mode Threshold | 40% SOC (SRS-F014) | | |
| Emergency Threshold (beacon shed) | 15% SOC (`docs/BASELINE_PARAMETERS.md` §5) | | |
| Jetson Enable Threshold | 55% SOC and sunlit (SRS-F015) | | |
| Load Shed Sequence | Per FDIR | | |

#### 6.2.4 ADCS Configuration

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| Detumble Threshold | 0.5°/s across +Y (`DETUMBLE_RATE_THRESHOLD`) | | |
| Spin Rate (SUNPOINT) | 8°/s about +Y (`SPIN_RATE` 0.1396 rad/s, `software/rust/sqso-fsw/src/adcs.rs`; BASELINE §8, decision D9 as changed by the design review) | | |
| B-dot Gain | 1.0e4 A·m²·s/T (`ADCS_BDOT_GAIN`) | | |
| Control Mode | Sun-pointing | | |
| Magnetometer Calibration | Applied | | |

#### 6.2.5 Deployment Configuration [SYS-221]

| Parameter | Expected Value | Actual | P/F |
|-----------|----------------|--------|-----|
| Deployment Timer | 30 minutes | | |
| Solar Wings Deploy First | Yes, one resistor at a time (SRS-F089) | | |
| Antennas (AntS) Second | Yes, over I2C3, temperature gate ≥ −17 °C (C-18) | | |
| Burn Duration | 10 s per resistor (decision C-17); hardware one-shot 12 s | | |
| Retry Attempts | 50 minutes apart, bounded (SRS-F088) | | |

### 6.3 Hardware Configuration Verification [SYS-001, SYS-004, SYS-005]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify rail dimensions | ≥ 8.5 mm wide, rail ends ≥ 6.5 × 6.5 mm (SYS-080) | |
| 2 | Verify overall dimensions | 100×100×113.5 mm | |
| 3 | Verify deployment switch location | Per drawing | |
| 4 | Verify RBF pin location | Per drawing | |
| 5 | Verify antenna stowage | Properly stowed | |
| 6 | Verify solar wing stowage | Properly stowed | |
| 6a | Verify the inertia ballast | 2 × 10 g W-Ni-Cu on the +Z end frame, as trimmed (BASELINE §4, decision C-13) | |
| 7 | Verify all fasteners | Torqued and staked | |
| 8 | Verify all connectors | Properly mated | |
| 9 | Verify harness routing | No pinch points | |
| 10 | Verify markings | Visible and correct | |

### 6.4 Inhibit State Verification [SYS-005, SYS-225]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify RBF pin installed | Pin in place | |
| 2 | Verify deployment inhibit | Status = inhibited | |
| 3 | Verify separation switch | Not actuated | |
| 4 | Remove RBF momentarily (under 5 min; a powered unit fires burn wires 30 min after any boot unless the hardware inhibits hold deployment power off) | Inhibit clears | |
| 5 | Re-install RBF pin | Inhibit active | |
| 6 | Verify deployment switches released read released (RF-silence count advances) | Polarity confirmed (SQSO-ICD-013 §6.3, TBR) | |

### 6.5 Pre-Ship Battery State

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure battery voltage | Record value | |
| 2 | Verify shipping SOC | Storage band agreed with the integrator; the final 60 % ± 5 % charge is made at the integrator (BASELINE §4; SQSO-LVR-001 LVR-071) | |
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

### 6.7 Flight Arming of the RF Silence and Deployment (last powered action) [SYS-221, SYS-224]

Bench power-ups can complete the 45-minute silence and the deployment sequence, and both persist. Arm them last (SQSO-ICD-013 §6.3, `deployment.h`).

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | With the umbilical connected, send the `key_provision.py arm-flight` frame (arms RF silence and deployment) | Reply `SQGR 00` | |
| 2 | Power off | Done; time recorded | |
| 3 | Any later power-up of 25 minutes or more | Repeat steps 1–2 before integration | |

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Software version | Matches SVD baseline |
| Configuration parameters | Match flight baseline |
| Hardware configuration | Matches as-built list |
| Inhibit states | Correct for shipping |
| Battery state | Shipping SOC as agreed with the integrator (BASELINE §4) |
| Documentation | Complete; approvals recorded when given |

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
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-16 | §6.2.1 to §6.2.3 corrected and cited: callsign KK6GDB-11; TX frequency the IARU-coordinated frequency (SYS-200, TBD-SYS-001; was 145.825 MHz); 0.5 W nominal and ≤ 1 W maximum (SYS-201; was a 2.0 W high mode); beacon 60/120/180 s in ACTIVE/IDLE/SAFE (SYS-211; SAFE was missing); SOC thresholds 25 % SAFE, 40 % IDLE, 15 % emergency, 55 % Jetson enable (SRS-F013 to SRS-F015, BASELINE §5; were 10 % and 50 %) (finding VV-06) |
