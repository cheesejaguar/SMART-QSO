# RF Test Results Document
## SMART-QSO CubeSat

**Document Number:** SQSO-VER-RF-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** Template - To be completed post-test

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
| Nominal Power | 0.5 - 2.0 W | RF_DESIGN.md |
| Power Tolerance | ± 1 dB | Design |
| Measurement Uncertainty | < 0.5 dB | Calibration |

### 3.2 Test Results

| Power Setting | Expected (dBm) | Measured (dBm) | Pass/Fail |
|---------------|----------------|----------------|-----------|
| Low (0.5W) | 27.0 ± 1.0 | TBD | TBD |
| Medium (1.0W) | 30.0 ± 1.0 | TBD | TBD |
| High (2.0W) | 33.0 ± 1.0 | TBD | TBD |

### 3.3 Temperature Variation

| Temp (°C) | Power (dBm) | Delta from 25°C |
|-----------|-------------|-----------------|
| -20 | TBD | TBD |
| +25 | TBD | Reference |
| +50 | TBD | TBD |

---

## 4. Frequency Accuracy and Stability

### 4.1 Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Center Frequency | 145.825 MHz | IARU Coordination |
| Frequency Accuracy | ± 5 kHz | Design |
| Frequency Stability | ± 2 kHz/°C | Design |

### 4.2 Test Results - Accuracy

| Condition | Expected (MHz) | Measured (MHz) | Error (kHz) | Pass/Fail |
|-----------|----------------|----------------|-------------|-----------|
| 25°C | 145.825 | TBD | TBD | TBD |
| After warm-up | 145.825 | TBD | TBD | TBD |

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
| FM Deviation | 3.5 kHz ± 0.5 kHz | Design |

### 5.2 Test Results

| Parameter | Expected | Measured | Pass/Fail |
|-----------|----------|----------|-----------|
| Mark Freq | 1200 Hz | TBD | TBD |
| Space Freq | 2200 Hz | TBD | TBD |
| Peak Deviation | 3.5 kHz | TBD | TBD |
| Modulation Index | 1.0 - 1.5 | TBD | TBD |

### 5.3 Eye Diagram / BER

| Parameter | Requirement | Measured | Pass/Fail |
|-----------|-------------|----------|-----------|
| Eye Opening | > 80% | TBD | TBD |
| BER (sensitivity) | < 1E-5 | TBD | TBD |

---

## 6. Spurious Emissions Compliance

### 6.1 Requirements (IARU)

| Frequency Range | Limit | Source |
|-----------------|-------|--------|
| Harmonics | < -43 dBc | IARU |
| Out-of-band | < -60 dBc | IARU |
| Spurious (general) | < -60 dBc | ITU |

### 6.2 Harmonic Measurements

| Harmonic | Frequency (MHz) | Level (dBc) | Limit (dBc) | Pass/Fail |
|----------|-----------------|-------------|-------------|-----------|
| Fundamental | 145.825 | 0 (ref) | - | - |
| 2nd | 291.65 | TBD | -43 | TBD |
| 3rd | 437.475 | TBD | -43 | TBD |
| 4th | 583.3 | TBD | -43 | TBD |
| 5th | 729.125 | TBD | -43 | TBD |

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
| Occupied BW (99%) | < 15 kHz | Design |
| Channel Spacing | 25 kHz | Amateur |

### 7.2 Test Results

| Parameter | Expected | Measured | Pass/Fail |
|-----------|----------|----------|-----------|
| -20 dB BW | < 12 kHz | TBD | TBD |
| 99% BW | < 15 kHz | TBD | TBD |

---

## 8. EIRP Calculation and Verification

### 8.1 Link Budget Parameters

| Parameter | Value | Source |
|-----------|-------|--------|
| TX Power | 30 dBm (1W nominal) | Measured |
| Cable Loss | -1.5 dB | Measured |
| Antenna Gain | +2.1 dBi (λ/4 monopole) | Calculated |
| **EIRP** | **30.6 dBm** | Calculated |

### 8.2 Antenna Pattern Verification

| Direction | Expected Gain (dBi) | Measured Gain (dBi) | Pass/Fail |
|-----------|---------------------|---------------------|-----------|
| Zenith | +2.1 | TBD | TBD |
| Horizon | -3.0 | TBD | TBD |
| -Z axis | -10 | TBD | TBD |

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
| - | None identified | - |

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
