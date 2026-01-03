# Test Procedure: EMC/EMI Test
## TP-ENV-004

**Document Number:** SQSO-TP-ENV-004
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

This procedure defines the electromagnetic compatibility (EMC) and electromagnetic interference (EMI) testing for the SMART-QSO CubeSat per MIL-STD-461G tailored for CubeSat applications.

---

## 2. SCOPE

### 2.1 Tests Included

| Test | MIL-STD-461G | Applicability |
|------|--------------|---------------|
| Conducted Emissions | CE102 | Power leads |
| Radiated Emissions | RE102 | 2 MHz - 18 GHz |
| Conducted Susceptibility | CS101/CS114 | Power leads |
| Radiated Susceptibility | RS103 | If required |
| Self-Compatibility | N/A | All systems |

### 2.2 Tests Excluded

- CS115, CS116: Not applicable to battery-powered spacecraft
- RE101: Magnetic field emissions (optional)

---

## 3. APPLICABLE DOCUMENTS

1. MIL-STD-461G - EMI Control Requirements
2. Launch Provider EMC Requirements
3. IARU Coordination Agreement
4. SMART-QSO ICD_BEACON_FORMAT.md

---

## 4. TEST ARTICLE

### 4.1 Configuration

| Item | Configuration |
|------|---------------|
| Unit Under Test | SMART-QSO Flight Unit |
| Power Source | Flight batteries or equivalent |
| Antennas | Flight antennas deployed |
| Software | Flight software |
| Mode | All operational modes |

### 4.2 Test Modes

| Mode | Description |
|------|-------------|
| Mode 1 | Safe Mode (beacon only) |
| Mode 2 | Idle Mode (OBC + beacon) |
| Mode 3 | Active Mode (all systems) |
| Mode 4 | Burst Mode (Jetson active) |

---

## 5. TEST EQUIPMENT

| Equipment | Specification | Cal Required |
|-----------|---------------|--------------|
| EMI Receiver | 9 kHz - 18 GHz | Yes |
| Spectrum Analyzer | 9 kHz - 26 GHz | Yes |
| LISN | 50µH, 50Ω | Yes |
| Anechoic Chamber | > 10 dB absorption | N/A |
| Current Probe | 20 Hz - 100 MHz | Yes |
| E-Field Probe | 10 kHz - 18 GHz | Yes |
| Signal Generator | 10 kHz - 18 GHz | Yes |
| Power Amplifier | 10 W minimum | N/A |
| Test Antennas | Biconical, Log-periodic | Yes |

---

## 6. PRECONDITIONS

- [ ] Functional test passed
- [ ] Test equipment calibrated within 12 months
- [ ] Anechoic chamber RF quiet verified
- [ ] Test software loaded and verified
- [ ] Batteries fully charged

---

## 7. SAFETY

| Hazard | Control |
|--------|---------|
| RF Exposure | Chamber interlock, power limits |
| High Voltage | Equipment grounding |
| Lithium Battery | Fire extinguisher nearby |

---

## 8. PROCEDURE

### 8.1 Pre-Test Setup

| Step | Action | Expected Result |
|------|--------|-----------------|
| 1 | Position UUT on test table | Centered, grounded |
| 2 | Connect LISN to power input | Proper impedance |
| 3 | Deploy antennas | Full deployment |
| 4 | Power on UUT | Boot successful |
| 5 | Verify beacon transmission | RF signal present |
| 6 | Record ambient noise floor | < -100 dBm |

### 8.2 Conducted Emissions (CE102)

| Step | Action | Pass Criteria |
|------|--------|---------------|
| 1 | Configure UUT in Mode 1 | Stable operation |
| 2 | Scan 10 kHz - 10 MHz | Record emissions |
| 3 | Configure UUT in Mode 3 | Stable operation |
| 4 | Scan 10 kHz - 10 MHz | Record emissions |
| 5 | Compare to limit | < CE102 limit - 6 dB |

**CE102 Limit (tailored):**
- 10 kHz - 500 kHz: 94 dBµV
- 500 kHz - 2 MHz: 94 - 20log(f/0.5) dBµV
- 2 MHz - 10 MHz: 60 dBµV

### 8.3 Radiated Emissions (RE102)

| Step | Action | Pass Criteria |
|------|--------|---------------|
| 1 | Configure UUT in Mode 1 | Stable operation |
| 2 | Position antenna 1m from UUT | Correct orientation |
| 3 | Scan 2 MHz - 18 GHz | Record emissions |
| 4 | Note intentional transmissions | 145.825 MHz beacon |
| 5 | Configure UUT in Mode 4 | Jetson active |
| 6 | Scan 2 MHz - 18 GHz | Record emissions |
| 7 | Compare to limit | < RE102 limit - 6 dB |

**RE102 Limit (tailored):**
- 2 MHz - 30 MHz: 34 dBµV/m
- 30 MHz - 1 GHz: 34 - 20log(f/30) dBµV/m
- > 1 GHz: Launch provider specific

**Exceptions:**
- Intentional transmission at 145.825 MHz (coordinated)
- Harmonics within IARU spurious limits (-43 dBc)

### 8.4 Self-Compatibility Test

| Step | Action | Pass Criteria |
|------|--------|---------------|
| 1 | Configure all systems active | Mode 4 |
| 2 | Monitor OBC operation | No resets |
| 3 | Monitor ADCS sensors | Nominal readings |
| 4 | Monitor beacon generation | Correct format |
| 5 | Monitor Jetson inference | Correct output |
| 6 | Duration: 30 minutes | No anomalies |

### 8.5 Conducted Susceptibility (CS101)

| Step | Action | Pass Criteria |
|------|--------|---------------|
| 1 | Inject 1 Vrms, 30 Hz - 50 kHz | Via LISN |
| 2 | Monitor UUT operation | No disruption |
| 3 | Inject 0.5 Vrms, 50 kHz - 150 kHz | Via LISN |
| 4 | Monitor UUT operation | No disruption |

### 8.6 Radiated Susceptibility (RS103) - If Required

| Step | Action | Pass Criteria |
|------|--------|---------------|
| 1 | Illuminate with 1 V/m, 2 MHz - 1 GHz | Via antenna |
| 2 | Monitor UUT operation | No disruption |
| 3 | Illuminate with 1 V/m, 1 GHz - 18 GHz | Via antenna |
| 4 | Monitor UUT operation | No disruption |

---

## 9. PASS/FAIL CRITERIA

| Test | Criterion |
|------|-----------|
| CE102 | Emissions < limit - 6 dB margin |
| RE102 | Emissions < limit - 6 dB margin |
| Self-Compat | No operational anomalies |
| CS101 | No operational disruption |
| RS103 | No operational disruption |

---

## 10. DATA RECORDING

### 10.1 Required Data

- [ ] Emission plots (frequency vs. amplitude)
- [ ] Limit line overlay
- [ ] Margin analysis
- [ ] Ambient noise floor
- [ ] UUT configuration photographs
- [ ] Test equipment calibration certificates

### 10.2 Report Template

```
EMC Test Report
Date: ___________
UUT S/N: ___________
Test Engineer: ___________

CE102: PASS / FAIL
RE102: PASS / FAIL
Self-Compat: PASS / FAIL
CS101: PASS / FAIL
RS103: PASS / FAIL / N/A

Overall: PASS / FAIL

Anomalies: ___________
```

---

## 11. POST-TEST

| Step | Action |
|------|--------|
| 1 | Power off UUT |
| 2 | Stow antennas (if applicable) |
| 3 | Perform abbreviated functional test |
| 4 | Complete test data package |
| 5 | Review results with test lead |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release for CDR |
