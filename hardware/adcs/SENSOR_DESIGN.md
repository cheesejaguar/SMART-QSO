# ADCS Sensor Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-ADCS-SEN-001
**Version:** 1.3
**Date:** 2026-10-27
**Status:** CDR Baseline

---

## 1. Overview

This document defines the Attitude Determination and Control System (ADCS) sensor design for the SMART-QSO 1U CubeSat.
The sensor circuits, pins and parts of record are in the OBC build-to design `hardware/electrical/obc/OBC_BOARD_DESIGN.md`
(SQSO-OBC-BRD-001) §9 and `docs/interfaces/ICD_OBC_ADCS.md` (SQSO-ICD-004), which govern where this document differs.
There is no ADCS board. Nothing has been built or measured.

### 1.1 Sensor Suite

| Sensor | Quantity | Purpose |
|--------|----------|---------|
| Magnetometer | 1 (3-axis), PNI RM3100 on board A2 | B-field measurement |
| Sun Sensors | 6 panel photodiodes | Sun direction for the spin law |
| Gyroscope | 1 (3-axis) primary plus 1 cold spare | Rate sensing (required, §4) |

### 1.2 Performance Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Attitude knowledge | None: no onboard attitude estimate (decision C-22); sun-vector knowledge is not a requirement (C-23) | `docs/hardware/ADCS_DESIGN.md` §2 |
| Rate knowledge | ≤ 0.2°/s per axis in use (ADCS-P007) | ADCS_DESIGN §2.2 |
| Sun pointing (ADCS-P002) | ≤ 5 % of sunlit generation lost to pointing and the Sun within 30° of +Y for ≥ 95 % of sunlit time, in every Monte Carlo trial (C-23) | Power; BASELINE §8 |

---

## 2. Magnetometer Design

### 2.1 Selected Sensor

| Parameter | Value |
|-----------|-------|
| Type | 3-axis magneto-inductive magnetometer |
| Part | **PNI RM3100** (SQSO-OBC-BRD-001 §9.4; LIS3MDL and BMM150 withdrawn) |
| Range | ± 800 µT, linear ± 200 µT |
| Resolution | 13 nT |
| Noise | 15 nT at 200 cycle counts |
| Interface | I2C4 at 0x20, single-shot on command with DRDY |
| Power | 260 µA at 8 Hz three-axis |

### 2.2 Placement

| Parameter | Value |
|-----------|-------|
| Location | Board A2 (24 × 24 mm) at the +Z end frame, slot (+24, +27) mm, harness W6E to OBC J10 |
| Orientation | Aligned with spacecraft body axes |
| Distance from torquers | Set by the rod-remanence coupling at the slot (SQSO-OBC-BRD-001 §9.4) |
| Magnetic keep-out | No ferromagnetic part within 20 mm of A2 (SQSO-OBC-BRD-001 §24 C13) |

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
| Type | Silicon photodiode, bought with the panels (BUILD_VS_BUY §2.3) |
| Part | Named by the panel vendor (VEMD1060X withdrawn) |
| FOV | Cosine response |
| Current output | Short-circuit current ≤ 4.0 mA at AM0 normal incidence (amplifier clips at 5.25 mA) |

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
| Transimpedance amp | TLV9002-Q1 (three duals), zero bias | Current to voltage |
| Gain resistor | 619 Ω 0.1 %, Cf 10 nF C0G | Sets sensitivity (up to 10 kΩ if the panel diode is smaller) |
| ADC | ADS7138-Q1, 12 bits, I2C4 0x10 | Digitize signal; 1.30 µA per LSB |
| Sample rate | 1 Hz (ADCS_DESIGN §4.2.1) | Sufficient for the spin law |

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
| Detumble | Capture criterion (rate across +Y and spin about +Y) |
| Pointing | Rate feedback for nutation damping and the spin hold |
| Eclipse | Rate feedback for the spin hold (no attitude is propagated: C-22) |

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
Pins and buffers: `docs/interfaces/ICD_OBC_ADCS.md` and `hardware/electrical/INTERCONNECT.md` §4, which govern.

| Parameter | Value |
|-----------|-------|
| Magnetometer and sun-sensor ADC bus | I2C4 (SCL PD12, SDA PD13) through a TCA9517-Q1; MAG_DRDY PD11 |
| Primary gyro bus | SPI3 (PC10–PC12, CS PD1), DRDY PD10 |
| Magnetometer harness | W6E, ≈ 0.09 m |

### 6.2 Bus Addresses

| Device | Address |
|--------|---------|
| Magnetometer RM3100 | I2C4 0x20 |
| Sun-sensor ADC ADS7138-Q1 | I2C4 0x10 |
| Gyroscope M-G370PDT0 | SPI3 (no address) |

### 6.3 ADC Channels (Sun Sensors)

The six photodiode amplifiers occupy six inputs of the ADS7138-Q1; AIN6 is a TMP235-Q1 beside the gyros and AIN7 is
VBAT_MTQ / 6 (SQSO-OBC-BRD-001 §9.5). The channel-to-face assignment is in the board's netlist.

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
| Magnetometer RM3100 | No published data: own latch-up switch and the RAD-V3 lot test (SQSO-OBC-BRD-001 §9.4) |
| Photodiodes | Panel vendor's data |
| Gyroscope M-G370PDT0 | TID pass to 20 krad(Si); no SEL data (§4.1) |
| Gyroscope BMI088 | No data (§4.1) |

Design level 10 krad(Si) with the shielding rule of BASELINE §6.

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
| Noise | ≤ 50 nT (the Monte Carlo assumption) |
| Temperature | Stable -20°C to +50°C |

### 8.2 Sun Sensor Tests

| Test | Requirement |
|------|-------------|
| FOV verification | ±60° response |
| Response | Against the photodiode-and-albedo model (no sun-vector accuracy requirement, C-23) |
| Threshold determination | Correct albedo rejection |

### 8.3 System-Level Tests

| Test | Requirement |
|------|-------------|
| Sun direction and pointing | Photodiode response against the model (TP-ADCS-003); ADCS-P002 by the Monte Carlo (ADCS_DESIGN §11.6) |
| Rate | ADCS-P007 (≤ 0.2°/s per axis in use) |
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
| 1.3 | 2026-10-27 | Documentation audit (post-CDR 2026): brought into line with SQSO-OBC-BRD-001 §9 and SQSO-ICD-004: §1 gyro required, requirements per C-22/C-23 (ADCS-P002, ADCS-P007); §2 RM3100 on board A2 (LIS3MDL/BMM150 and the ADCS board withdrawn); §3 panel photodiodes, TLV9002-Q1 and ADS7138-Q1; §4.2 no attitude propagation; §6 buses I2C4 and SPI3; §7.2 radiation data per part; §8 tests per C-23 |
