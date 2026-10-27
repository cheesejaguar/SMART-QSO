# Test Procedure: EMC/EMI Test
## TP-ENV-004

**Document Number:** SQSO-TP-ENV-004
**Version:** 1.3
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** MRD-080 (launch provider EMC), SYS-200, SYS-203, SYS-713

> **Governing tests.** `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` (SQSO-ETP-001) §3.6 sets the EMC tests and pass
> criteria; they are in §8.7 below and govern where this procedure differs. MIL-STD-461G RE102 and CE102 at spacecraft
> level are not required (the spacecraft is unpowered in the dispenser; GEVS-B Appendix B §2.5), so §8.2, §8.3, §8.5 and
> §8.6 are optional engineering-model pre-scans. On the flight model the antennas are **not** deployed for this test:
> it runs after the protoflight vibration and thermal vacuum (ETP §4 step 8), and a deployment after protoflight testing
> needs the integrator's written approval (CDS Rev 14.1 §3.5.2; ETP §3.8). Radiated checks with deployed antennas are
> made on the engineering model.

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

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
| Antennas | EM: deployed. FM: stowed, transmitters into loads through the antenna ports (see the governing-tests note) |
| Software | Flight software |
| Mode | All operational modes |

### 4.2 Test Modes

| Mode | Description |
|------|-------------|
| Mode 1 | Safe Mode (beacon only) |
| Mode 2 | Idle Mode (OBC + beacon) |
| Mode 3 | Active Mode (all systems) |
| Mode 4 | Active Mode with a Jetson burst and magnetorquer actuation (there is no separate burst mode) |

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
| 3 | EM: deploy antennas. FM: connect the antenna ports to loads | Configuration recorded |
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

### 8.3 Radiated Emissions (RE102) [SYS-200, SYS-203]

| Step | Action | Pass Criteria |
|------|--------|---------------|
| 1 | Configure UUT in Mode 1 | Stable operation |
| 2 | Position antenna 1m from UUT | Correct orientation |
| 3 | Scan 2 MHz - 18 GHz | Record emissions |
| 4 | Note intentional transmissions | Beacon at the IARU-coordinated downlink frequency (TBD until coordination, TBD-SYS-001; SYS-200) |
| 5 | Configure UUT in Mode 4 | Jetson active |
| 6 | Scan 2 MHz - 18 GHz | Record emissions |
| 7 | Compare to limit | < RE102 limit - 6 dB |

**RE102 Limit (tailored):**
- 2 MHz - 30 MHz: 34 dBµV/m
- 30 MHz - 1 GHz: 34 - 20log(f/30) dBµV/m
- > 1 GHz: Launch provider specific

**Exceptions:**
- Intentional transmission at the IARU-coordinated downlink frequency in 145.800–146.000 MHz, not 145.825 MHz (SYS-200; TBD-SYS-001)
- Harmonics at least 46 dB below carrier and ≤ 25 µW at 1 W (SYS-203; 47 CFR 97.307(e)); third harmonic at least 70 dB below carrier (SYS-203 Rev G)

### 8.4 Self-Compatibility Test

| Step | Action | Pass Criteria |
|------|--------|---------------|
| 1 | Configure all systems active | Mode 4 |
| 2 | Monitor OBC operation | No resets |
| 3 | Monitor ADCS sensors | Nominal readings |
| 4 | Monitor beacon generation | Correct format |
| 5 | Monitor Jetson inference | Correct output |
| 6 | Duration: 30 minutes | No anomalies |

### 8.7 ETP §3.6 Tests (govern) [SYS-203, SYS-713]

| Step | Test | Pass Criteria |
|------|------|---------------|
| 1 | Bus ripple at each power card and at the distribution point, all loads on | ≤ 200 mV peak-to-peak on the 14.4 V bus, 1 MHz bandwidth |
| 2 | Emissions at the 435 MHz receiver input in every mode, including a Jetson burst and magnetorquer PWM | ≥ 6 dB below the receiver's minimum usable signal (SYS-713) |
| 3 | RF self-compatibility, each transmitter at full power | No reset, no false telecommand, no sensor fault (SYS-713) |
| 4 | Spurious and harmonics, conducted (TP-RF-003) | ≥ 46 dBc and ≤ 25 µW at 1 W; third harmonic ≥ 70 dBc at the antenna port (SYS-203) |
| 5 | EM only: GNSS L1 noise with the Jetson, PA and magnetorquers on | Receiver keeps a fix with C/N0 loss ≤ 3 dB |
| 6 | EM only: magnetometer with the Jetson and magnetorquer currents | Bias shift within the ADCS calibration budget |

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
| ETP §3.6 (§8.7) | Every criterion of §8.7 met (governs) |
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
| 2 | EM only: stow antennas per the vendor procedure |
| 3 | Perform abbreviated functional test |
| 4 | Complete test data package |
| 5 | Review results with test lead |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release for CDR |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-16 | §8.3 step 4 and exceptions: beacon at the IARU-coordinated frequency (SYS-200, TBD-SYS-001; was 145.825 MHz); harmonic exception restated as SYS-203 plus the 25 µW of 47 CFR 97.307(e) (was -43 dBc, "IARU") (finding VV-06) |
| 1.3 | 2026-10-27 | Documentation audit (post-CDR 2026): ETP §3.6 tests added as governing (SYS-713); MIL-STD-461 scans optional EM pre-scans; no antenna deployment on the FM after protoflight tests (CDS §3.5.2); harmonics per SYS-203 Rev G; status and banner |
