# ADCS Sensor Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-ADCS-SEN-001
**Version:** 1.2
**Date:** 2026-10-27
**Status:** CDR Baseline

---

## 1. Overview

This document defines the Attitude Determination and Control System (ADCS) sensor design for the SMART-QSO 1U CubeSat.

### 1.1 Sensor Suite

| Sensor | Quantity | Purpose |
|--------|----------|---------|
| Magnetometer | 1 (3-axis) | B-field measurement |
| Sun Sensors | 6 | Sun vector determination |
| Gyroscope | 1 (3-axis) | Rate sensing (optional) |

### 1.2 Performance Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Attitude knowledge | ±10° (3σ) | Mission |
| Rate knowledge | ±1°/s | Detumble |
| Sun pointing accuracy | ±15° | Power |

---

## 2. Magnetometer Design

### 2.1 Selected Sensor

| Parameter | Value |
|-----------|-------|
| Type | 3-axis MEMS magnetometer |
| Part | LIS3MDL, BMM150, or equivalent |
| Range | ±16 gauss (±1.6 mT) |
| Resolution | 0.5 mGauss (50 nT) |
| Noise | < 3 mGauss RMS |
| Interface | I2C (400 kHz) |
| Power | < 1 mW |

### 2.2 Placement

| Parameter | Value |
|-----------|-------|
| Location | ADCS board, away from magnetorquers |
| Orientation | Aligned with spacecraft body axes |
| Distance from torquers | > 50 mm |
| Distance from battery | > 30 mm |

### 2.3 Calibration Requirements

| Calibration | Method |
|-------------|--------|
| Hard iron offset | Ground calibration + on-orbit |
| Soft iron scaling | Ground calibration |
| Axis alignment | Assembly measurement |
| Temperature compensation | Internal or lookup table |

### 2.4 Calibration Model

```
B_cal = S × (B_raw - H)

Where:
  B_cal = Calibrated field vector
  S = 3×3 soft iron matrix
  B_raw = Raw sensor reading
  H = Hard iron offset vector
```

---

## 3. Sun Sensor Design

### 3.1 Sensor Type Selection

| Type | Accuracy | FOV | Complexity | Selected |
|------|----------|-----|------------|----------|
| Photodiode (analog) | ±5° | 120° | Low | **Yes** |
| Slit sensor | ±1° | 60° | Medium | No |
| Digital sun sensor | ±0.1° | 50° | High | No |

### 3.2 Photodiode Sensor Specification

| Parameter | Value |
|-----------|-------|
| Type | Silicon photodiode |
| Part | VEMD1060X or equivalent |
| Spectral response | 400-1100 nm |
| Active area | 1-4 mm² |
| FOV | ±60° (120° total) |
| Current output | 10-100 µA in sun |

### 3.3 Configuration

| Face | Axis | Quantity |
|------|------|----------|
| +X | +X | 1 |
| -X | -X | 1 |
| +Y | +Y | 1 |
| -Y | -Y | 1 |
| +Z | +Z | 1 |
| -Z | -Z | 1 |

### 3.4 Sun Vector Algorithm

```
For each face i with photodiode current I_i:
  If I_i > threshold:
    sun_vector += I_i × face_normal_i

sun_vector = normalize(sun_vector)
```

### 3.5 Interface Circuit

| Component | Value | Purpose |
|-----------|-------|---------|
| Transimpedance amp | TLV2371 | Current to voltage |
| Gain resistor | 10kΩ | Sets sensitivity |
| ADC | OBC internal | Digitize signal |
| Sample rate | 10 Hz | Sufficient for control |

---

## 4. Gyroscope Design

The gyro is not optional. The magnetometer rate estimate cannot see the rate about the field, so the gyro is the only
check that the detumble is complete. It is also the rate in the spin law (`docs/test/SIL_MISSION_SIMULATION.md`
SIL-05; `docs/hardware/ADCS_DESIGN.md` §4.2.1). The selection is in `docs/hardware/BUILD_VS_BUY.md` §5.

### 4.1 Sensor Specification

| Parameter | Requirement | Primary: Epson M-G370PDT0 | Backup: Bosch BMI088 (on the OBC board) |
|-----------|-------------|---------------------------|------------------------------------------|
| Range | ≥ ±50°/s (tip-off up to 30°/s) | ±200°/s | ±125°/s range selected |
| Bias in use, per axis | **≤ 0.1°/s (1σ)** (the Monte Carlo: 0.2°/s degrades, 0.3°/s fails; BUILD_VS_BUY §6.4) | 0.10°/s (1σ, factory-compensated −40…+85 °C); turn-on repeatability 0.01°/s | ±1°/s as delivered: **ground thermal calibration required** (TCO ±0.015°/s/K) |
| Noise | ≪ 0.5°/s per sample | 0.0007°/s/√Hz | 0.014°/s/√Hz |
| Scale factor | ≤ 1 % | ±0.2 % | ±1 % |
| Interface | SPI (STM32L4R5, 3.3 V) | SPI / UART | SPI / I²C |
| Power | ≤ 60 mW (ADCS sensor allocation 0.12 W) | 53 mW | 16 mW (off unless the primary has failed) |
| Mass | — | 10 g | < 1 g |
| Radiation | ≥ 10 krad(Si) TID (BASELINE §6); latch-up current-limited and switchable | TID pass to 20 krad(Si); no SEL data | No data |
| Heritage | Flight-tested | Int-Ball2 inside the ISS (JAXA, since 2023) | OreSat0.5 (free flyer, 2024) |

### 4.2 Usage

| Mode | Gyroscope Use |
|------|---------------|
| Detumble | Rate measurement for B-dot |
| Pointing | Rate feedback for control |
| Eclipse | Propagate attitude |

### 4.3 Bias Estimation

- Ground: the BMI088 backup is calibrated over −20…+50 °C before integration, against a rate table, to ≤ 0.1°/s
  residual. The M-G370PDT0 is checked at three temperatures against its factory compensation.
- On orbit (not designed): the bias could be estimated against the magnetometer during the steady spin, when the
  field turns in the body at the known spin rate. Until that is designed, the bias requirement above has to be met
  without it.

---

## 5. Sensor Use (no onboard attitude determination)

**Decisions C-22 and C-23 (CDR 2026, round 3 closure).** The flight software has no three-axis attitude estimate: the
spin-stabilised sun-pointing law uses the sun vector from the six face photodiodes, the magnetometer field and the
gyro rate directly (`docs/hardware/ADCS_DESIGN.md` §4.2.1, §4.3). The TRIAD and extended Kalman filter of
Version 1.1 are out of the flight baseline; TRIAD stays a ground tool for reconstructing a pass from the downlinked
sun vector and field.

| Quantity | Sensor | Used for | Requirement |
|----------|--------|----------|-------------|
| Sun direction (body) | Six face photodiodes, opposite faces subtracted (ADS7138) | Momentum and spin law | None on its accuracy (C-23); the pointing is judged by its power, ADCS-P002 |
| Body rate | Gyro (M-G370PDT0, BMI088 backup) | Nutation damping, capture | ADCS-P007: ≤ 0.2°/s per axis in use |
| Field | Magnetometer (RM3100) | B-dot, dipole command | Sampled in the coils-off window, not while gated (SRS-F090, SRS-F143) |

The ADCS Monte Carlo models the photodiodes with their gain spread, grazing loss, the wings' shadow and the Earth's
albedo (ADCS_DESIGN §11.6): albedo dominates the sun-vector error (about 10° mean), and the pointing requirement is
set on generation for that reason. If the EM measurement of TP-ADCS-003 §3.1 is worse than the model, the recorded
option is an albedo-immune sun sensor on +Y (§3.1: slit or digital types).

---

## 6. Sensor Interface

### 6.1 I2C Bus Configuration

| Parameter | Value |
|-----------|-------|
| Bus | I2C2 on OBC |
| Speed | 400 kHz |
| Pull-ups | 4.7kΩ to 3.3V |
| Cable length | < 100 mm |

### 6.2 I2C Addresses

| Device | Address |
|--------|---------|
| Magnetometer | 0x1E |
| Gyroscope | 0x6A |

### 6.3 ADC Channels (Sun Sensors)

| Channel | Sensor |
|---------|--------|
| ADC_IN8 | Sun +X |
| ADC_IN9 | Sun -X |
| ADC_IN10 | Sun +Y |
| ADC_IN11 | Sun -Y |
| ADC_IN12 | Sun +Z |
| ADC_IN13 | Sun -Z |

---

## 7. Environmental Considerations

### 7.1 Magnetic Cleanliness

| Source | Mitigation |
|--------|------------|
| Magnetorquers | Distance, timing |
| Battery | Distance |
| DC-DC converters | Shielding |
| Solar cells | Negligible |

### 7.2 Radiation

| Sensor | TID Tolerance |
|--------|---------------|
| Magnetometer MEMS | > 30 krad |
| Photodiodes | > 100 krad |
| Gyroscope MEMS | > 30 krad |

### 7.3 Temperature Compensation

| Sensor | Temp Coefficient | Compensation |
|--------|------------------|--------------|
| Magnetometer | 0.3%/°C | Internal/LUT |
| Photodiodes | -0.2%/°C | Ratio method |
| Gyroscope | 0.01°/s/°C | Bias tracking |

---

## 8. Testing Requirements

### 8.1 Magnetometer Tests

| Test | Requirement |
|------|-------------|
| Calibration | Hard/soft iron determined |
| Accuracy | ±1° in Earth field |
| Noise | < 5 mGauss RMS |
| Temperature | Stable -20°C to +50°C |

### 8.2 Sun Sensor Tests

| Test | Requirement |
|------|-------------|
| FOV verification | ±60° response |
| Sun vector accuracy | ±5° |
| Threshold determination | Correct albedo rejection |

### 8.3 System-Level Tests

| Test | Requirement |
|------|-------------|
| Attitude determination | ±10° accuracy |
| Sensor fusion | Consistent output |
| Fault detection | Identify bad sensor |

---

## 9. Telemetry

### 9.1 Sensor Telemetry Points

| Point | Type | Rate |
|-------|------|------|
| Mag X, Y, Z | Float | 1 Hz |
| Sun sensor (6) | Uint16 | 1 Hz |
| Gyro X, Y, Z | Float | 10 Hz |
| Sensor status | Bitmap | 1 Hz |
| Temperature | Float | 0.1 Hz |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-26 | §4: gyro required; Epson M-G370PDT0 primary, BMI088 backup; bias ≤ 0.1°/s from the ADCS Monte Carlo (BUILD_VS_BUY §5–§6) |
| 1.2 | 2026-10-27 | §5: TRIAD and EKF out of the flight baseline (decisions C-22, C-23); the sensors' use and the requirements on them; the albedo-immune +Y sun sensor recorded as the option (finding P3F-G-07) |
