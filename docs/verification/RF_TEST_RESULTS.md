# RF Test Results Document
## SMART-QSO CubeSat

**Document Number:** SQSO-VER-RF-001
**Version:** 1.3
**Date:** 2026-10-27
**Status:** Template - To be completed post-test

> **Template; no test has been run.** No hardware exists (`docs/BASELINE_PARAMETERS.md` §1). Every "TBD" in a
> result, serial-number, calibration or pass/fail cell is a blank field to be filled when the test is run, not an open
> design value. The procedures are TP-RF-001 to TP-RF-006; the criteria are VI-RF-02 to VI-RF-04 and VI-RF-12
> (`docs/verification/VI_REGISTER.md`).

---

## 1. Overview

This document records the RF characterization and verification test results for the SMART-QSO CubeSat communication subsystem.

---

## 2. Test Configuration

### 2.1 Unit Under Test

| Item | Value |
|------|-------|
| Unit | SMART-QSO Flight Unit |
| Serial Number | TBD |
| Software Version | TBD |
| Configuration | Flight |

### 2.2 Test Equipment

| Equipment | Model | Serial | Cal Date |
|-----------|-------|--------|----------|
| Spectrum Analyzer | TBD | TBD | TBD |
| Power Meter | TBD | TBD | TBD |
| Frequency Counter | TBD | TBD | TBD |
| Dummy Load (50Ω) | TBD | TBD | N/A |
| Attenuator | TBD | TBD | TBD |
| RF Cables | TBD | TBD | TBD |

---

## 3. Transmitter Power Output Verification

### 3.1 Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Nominal Power | 0.5 W nominal, never above 1 W | SYS-201; BASELINE §7 |
| Power Tolerance | ± 0.5 dB at the antenna port, −20 to +60 °C | VI-RF-02 (TP-RF-002) |
| Measurement Uncertainty | < 0.5 dB | Calibration |

### 3.2 Test Results

| Power Setting | Expected (dBm) | Measured (dBm) | Pass/Fail |
|---------------|----------------|----------------|-----------|
| Nominal (0.5W) | 27.0 ± 0.5 | TBD | TBD |
| Maximum (1.0W) | 29.5–30.0 (never above 30.0, SYS-201) | TBD | TBD |

### 3.3 Temperature Variation

| Temp (°C) | Power (dBm) | Delta from 25°C |
|-----------|-------------|-----------------|
| -20 | TBD | TBD |
| +25 | TBD | Reference |
| +60 | TBD | TBD |

---

## 4. Frequency Accuracy and Stability

### 4.1 Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Center Frequency (f<sub>c</sub>) | One IARU-coordinated frequency in 145.800–146.000 MHz, **not 145.825 MHz**; TBD until coordination | `docs/BASELINE_PARAMETERS.md` §7 |
| Frequency Accuracy | ± 5 kHz from f<sub>c</sub> under all test conditions, including temperature drift (≈ ± 34 ppm at 145.9 MHz) | Design allocation (TBC) |
| Frequency Stability | Drift ≤ ± 2 kHz from the +25 °C value over the whole −20 °C to +50 °C test range (total, not per °C) | Design allocation (TBC) |

The stability line is a total over the temperature range, so it uses part of the ± 5 kHz accuracy budget: the setting
error at +25 °C may be at most ± 3 kHz for the total to stay within ± 5 kHz. (A per-degree figure of ± 2 kHz/°C, as
earlier revisions stated, would allow 140 kHz of drift over a 70 °C range and could never meet ± 5 kHz.)

### 4.2 Test Results - Accuracy

| Condition | Expected (MHz) | Measured (MHz) | Error (kHz) | Pass/Fail |
|-----------|----------------|----------------|-------------|-----------|
| 25°C | f<sub>c</sub> (TBD) | TBD | TBD | TBD |
| After warm-up | f<sub>c</sub> (TBD) | TBD | TBD | TBD |

### 4.3 Test Results - Stability

| Temperature (°C) | Frequency (MHz) | Drift from 25°C (kHz) | Pass/Fail |
|------------------|-----------------|------------------------|-----------|
| -20 | TBD | TBD | TBD |
| 0 | TBD | TBD | TBD |
| +25 | TBD | Reference | TBD |
| +50 | TBD | TBD | TBD |

---

## 5. Modulation Characteristics

### 5.1 Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Modulation Type | AFSK | Design |
| Baud Rate | 1200 bps | AX.25 |
| Mark Frequency | 1200 Hz | Bell 202 |
| Space Frequency | 2200 Hz | Bell 202 |
| FM Deviation | 3.0 kHz ± 0.5 kHz | 10K4F2D emission, BASELINE §7; TP-RF-004 |

### 5.2 Test Results

| Parameter | Expected | Measured | Pass/Fail |
|-----------|----------|----------|-----------|
| Mark Freq | 1200 Hz | TBD | TBD |
| Space Freq | 2200 Hz | TBD | TBD |
| Peak Deviation | 3.0 kHz | TBD | TBD |
| Modulation Index | 1.0 - 1.5 | TBD | TBD |

### 5.3 Eye Diagram / BER

| Parameter | Requirement | Measured | Pass/Fail |
|-----------|-------------|----------|-----------|
| Eye Opening | > 80% | TBD | TBD |
| BER (sensitivity) | < 1E-5 | TBD | TBD |

---

## 6. Spurious Emissions Compliance

### 6.1 Requirements

| Frequency Range | Limit | Source |
|-----------------|-------|--------|
| Harmonics and spurious | At least 46 dB below carrier and ≤ 25 µW at 1 W | SYS-203; 47 CFR 97.307(e); BASELINE §7 |
| 3rd harmonic (437.4–438.0 MHz, next to the uplink band) | ≤ −70 dBc at the antenna port, antennas deployed | SYS-203 Rev G; BASELINE §7; VI-RF-04 |
| Out-of-band | < -60 dBc | IARU |
| Spurious (general) | < -60 dBc | ITU |

### 6.2 Harmonic Measurements

| Harmonic | Frequency (MHz) | Level (dBc) | Limit (dBc) | Pass/Fail |
|----------|-----------------|-------------|-------------|-----------|
| Fundamental | f_c: the IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz (SYS-200; TBD-SYS-001) | 0 (ref) | - | - |
| 2nd | 2 f_c (291.6–292.0) | TBD | -46 | TBD |
| 3rd | 3 f_c (437.4–438.0) | TBD | -70 | TBD |
| 4th | 4 f_c (583.2–584.0) | TBD | -46 | TBD |
| 5th | 5 f_c (729.0–730.0) | TBD | -46 | TBD |

Limits are for the 1 W maximum (SYS-201): ≥ 46 dB below carrier and ≤ 25 µW (SYS-203; 47 CFR 97.307(e)), and the third
harmonic ≥ 70 dB below carrier (SYS-203 Rev G; `docs/BASELINE_PARAMETERS.md` §7).

### 6.3 Spurious Emissions Scan

| Frequency Range | Maximum Level (dBc) | Pass/Fail |
|-----------------|---------------------|-----------|
| 10 kHz - 145 MHz | TBD | TBD |
| 146 - 292 MHz | TBD | TBD |
| 292 - 1000 MHz | TBD | TBD |
| 1 - 2 GHz | TBD | TBD |

---

## 7. Occupied Bandwidth

### 7.1 Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Occupied BW (99%) | ≤ 10.4 kHz telemetry (10K4F2D); ≤ 16 kHz transponder (16K0F3E) | BASELINE §7 |
| Channel Spacing | 25 kHz | Amateur |

### 7.2 Test Results

| Parameter | Expected | Measured | Pass/Fail |
|-----------|----------|----------|-----------|
| -20 dB BW | < 12 kHz | TBD | TBD |
| 99% BW | ≤ 10.4 kHz (telemetry) | TBD | TBD |

---

## 8. EIRP Calculation and Verification

### 8.1 Link Budget Parameters

| Parameter | Value | Source |
|-----------|-------|--------|
| TX Power | 27 dBm (0.5 W nominal; 30 dBm at the 1 W maximum) | Measured |
| Cable Loss | -1.5 dB | Measured |
| Antenna Gain | Median ≥ −3 dBi, 10th percentile ≥ −10 dBi over the pass-weighted directions (SYS-222; `docs/environment/LINK_BUDGET_ANALYSIS.md` §5.2) | Calculated, then measured (VI-05, VI-RF-01) |
| **EIRP** | **From the measured values** | Calculated |

### 8.2 Antenna Pattern Verification

| Direction | Expected Gain (dBi) | Measured Gain (dBi) | Pass/Fail |
|-----------|---------------------|---------------------|-----------|
| Median over the pass-weighted directions | ≥ −3 (SYS-222) | TBD | TBD |
| 10th percentile | ≥ −10 (SYS-222) | TBD | TBD |
| Model agreement, two orthogonal planes | within 2 dB (VI-RF-01) | TBD | TBD |

---

## 9. Beacon Decode Verification

### 9.1 Test Configuration

- Reference receiver: TBD
- TNC: Direwolf / hardware TNC
- Distance: 3m (anechoic) or direct cable

### 9.2 Results

| Parameter | Requirement | Result | Pass/Fail |
|-----------|-------------|--------|-----------|
| Callsign Decode | Correct | TBD | TBD |
| Packet CRC | Valid | TBD | TBD |
| Content Verify | Per ICD | TBD | TBD |
| Decode Rate | > 95% | TBD | TBD |

---

## 10. Summary

### 10.1 Compliance Summary

| Test | Result |
|------|--------|
| Power Output | TBD |
| Frequency Accuracy | TBD |
| Frequency Stability | TBD |
| Modulation | TBD |
| Spurious Emissions | TBD |
| Occupied Bandwidth | TBD |
| EIRP | TBD |
| Beacon Decode | TBD |
| **Overall** | **TBD** |

### 10.2 Non-Conformances

| NCR # | Description | Disposition |
|-------|-------------|-------------|
| - | (none recorded; no test run) | - |

---

## 11. Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Test Engineer | ________ | ________ | ________ |
| RF Lead | ________ | ________ | ________ |
| QA | ________ | ________ | ________ |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial template for CDR |
| 1.1 | 2026-10-16 | Funding audit VV-21: frequency stability restated as total drift over the test range (± 2 kHz), within the ± 5 kHz accuracy; centre frequency per BASELINE §7 (not 145.825 MHz) in §4. Other 145.825 MHz values (§6.2) and the −43 dBc harmonic limit (BASELINE §7: ≥ 46 dBc at 1 W, target ≥ 60 dB) are not yet updated |
| 1.2 | 2026-10-16 | §6.1 and §6.2: harmonic limits from SYS-203 and 47 CFR 97.307(e) (−46 dBc at 1 W; 3rd-harmonic target −60 dBc, BASELINE §7), replacing −43 dBc "IARU"; fundamental the IARU-coordinated frequency (SYS-200, TBD-SYS-001), harmonics as multiples of it, replacing 145.825 MHz (finding VV-21) |
| 1.3 | 2026-10-27 | Documentation audit (post-CDR 2026): template banner (no test run; placeholder cells are blank fields); power 0.5 W nominal, ≤ 1 W, ± 0.5 dB over −20 to +60 °C (VI-RF-02; the 2.0 W row removed); deviation 3.0 kHz (10K4F2D); third harmonic ≤ −70 dBc (SYS-203 Rev G); occupied bandwidth 10.4 kHz; antenna gain and pattern rows per SYS-222 Rev G (was +2.1 dBi at zenith) |
