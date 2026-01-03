# ADCS Sensor Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-ADCS-SEN-001
**Version:** 1.0
**Date:** 2026-01-02
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

## 4. Gyroscope Design (Optional)

### 4.1 Sensor Specification

| Parameter | Value |
|-----------|-------|
| Type | 3-axis MEMS gyroscope |
| Part | BMI088, LSM6DSO, or equivalent |
| Range | ±2000°/s |
| Resolution | 0.01°/s |
| Noise | < 0.01°/s/√Hz |
| Bias stability | < 10°/hr |
| Interface | SPI or I2C |
| Power | < 5 mW |

### 4.2 Usage

| Mode | Gyroscope Use |
|------|---------------|
| Detumble | Rate measurement for B-dot |
| Pointing | Rate feedback for control |
| Eclipse | Propagate attitude |

### 4.3 Bias Estimation

- Gyro bias estimated on-orbit using magnetometer
- Kalman filter for bias tracking
- Re-calibrate during low-rate periods

---

## 5. Sensor Fusion

### 5.1 Attitude Determination Method

| Method | Sensors Used | Accuracy |
|--------|--------------|----------|
| TRIAD | Magnetometer + Sun | ±5° |
| Extended Kalman Filter | All sensors | ±3° |
| B-dot (rates only) | Magnetometer | Rates only |

### 5.2 TRIAD Algorithm

```
Given: Sun vector (s) and B-field vector (b) in body frame
       Sun vector (S) and B-field vector (B) in inertial frame

Body frame reference vectors:
  v1_body = s
  v2_body = (s × b) / |s × b|
  v3_body = v1_body × v2_body

Inertial frame reference vectors:
  v1_inertial = S
  v2_inertial = (S × B) / |S × B|
  v3_inertial = v1_inertial × v2_inertial

Rotation matrix:
  R = [v1_body v2_body v3_body] × [v1_inertial v2_inertial v3_inertial]ᵀ
```

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
