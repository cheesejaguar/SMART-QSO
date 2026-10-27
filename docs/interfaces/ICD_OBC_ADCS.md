# Interface Control Document: OBC to ADCS Subsystem

**Document ID**: SQSO-ICD-004
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the interface between the On-Board Computer (OBC) and the Attitude Determination and Control Subsystem (ADCS) for the SMART-QSO CubeSat per NPR 7123.1 SE Process 5.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-SRS-001 | System Requirements Specification |
| SQSO-HW-ADCS-001 | ADCS Design |
| SQSO-HW-OBC-001 | OBC Design |

## 3. Interface Summary

```
INTERFACE BLOCK DIAGRAM

┌─────────────────────────────────────────────────────────────┐
│                          OBC                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  ADCS       │    │   B-dot     │    │   Sun       │     │
│  │  Manager    │    │  Controller │    │  Pointing   │     │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘     │
│         │                  │                   │            │
└─────────┼──────────────────┼───────────────────┼────────────┘
          │                  │                   │
          │ I2C              │                   │
          │                  │                   │
┌─────────┼──────────────────┼───────────────────┼────────────┐
│         ▼                  ▼                   ▼            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │Magnetometer │    │Magnetorquers│    │ Sun Sensors │     │
│  │  (LIS3MDL)  │    │   (3-axis)  │    │  (Analog)   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                         ADCS Board                          │
└─────────────────────────────────────────────────────────────┘
```

## 4. Physical Interface

### 4.1 Connector Definition

| Interface | OBC Connector | ADCS Connector | Cable |
|-----------|---------------|----------------|-------|
| I2C/PWM | J4 (Hirose DF12) | J1 (Hirose DF12) | HAR-004 |

### 4.2 Pin Assignment

| Pin | Signal | Direction | Description |
|-----|--------|-----------|-------------|
| 1 | GND | - | Signal ground |
| 2 | I2C_SDA | Bidirectional | I2C data |
| 3 | I2C_SCL | OBC→ADCS | I2C clock |
| 4 | MTQ_X | OBC→ADCS | Magnetorquer X PWM |
| 5 | MTQ_Y | OBC→ADCS | Magnetorquer Y PWM |
| 6 | MTQ_Z | OBC→ADCS | Magnetorquer Z PWM |
| 7 | SUN_1 | ADCS→OBC | Sun sensor 1 analog |
| 8 | SUN_2 | ADCS→OBC | Sun sensor 2 analog |
| 9 | GYRO_INT | ADCS→OBC | Gyro interrupt |
| 10 | NC | - | Reserved |

### 4.3 Cable Requirements

| Parameter | Requirement |
|-----------|-------------|
| Length | ≤100 mm |
| Type | Twisted pair, shielded |
| AWG | 26 AWG minimum |
| I2C pull-ups | 4.7kΩ on ADCS board |

## 5. Electrical Interface

### 5.1 I2C Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Clock frequency | - | 100 | 400 | kHz |
| Data rise time | - | - | 1000 | ns |
| Data fall time | - | - | 300 | ns |
| Pull-up resistance | 4.7 | - | 10 | kΩ |
| Bus capacitance | - | - | 400 | pF |

### 5.2 I2C Addresses

| Device | Address | R/W |
|--------|---------|-----|
| Magnetometer (LIS3MDL) | 0x1E | R/W |
| Gyroscope (LSM6DS3) | 0x6A | R/W |
| Temperature sensor | 0x48 | R |
| ADCS controller (optional) | 0x60 | R/W |

### 5.3 PWM Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| PWM frequency | 1 | 10 | 20 | kHz |
| Duty cycle range | 0 | - | 100 | % |
| Rise time | - | - | 100 | ns |
| Logic high | 2.4 | 3.3 | - | V |
| Logic low | - | 0 | 0.4 | V |

### 5.4 Analog Interface

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Sun sensor output | 0 | - | 3.3 | V |
| ADC resolution | 12 | - | - | bits |
| Sample rate | 10 | - | 100 | Hz |

## 6. Logical Interface

### 6.1 Magnetometer Interface (LIS3MDL)

#### 6.1.1 Register Map

| Register | Address | Description |
|----------|---------|-------------|
| WHO_AM_I | 0x0F | Device ID (0x3D) |
| CTRL_REG1 | 0x20 | Control register 1 |
| CTRL_REG2 | 0x21 | Control register 2 |
| CTRL_REG3 | 0x22 | Control register 3 |
| STATUS_REG | 0x27 | Status register |
| OUT_X_L | 0x28 | X-axis LSB |
| OUT_X_H | 0x29 | X-axis MSB |
| OUT_Y_L | 0x2A | Y-axis LSB |
| OUT_Y_H | 0x2B | Y-axis MSB |
| OUT_Z_L | 0x2C | Z-axis LSB |
| OUT_Z_H | 0x2D | Z-axis MSB |
| TEMP_OUT_L | 0x2E | Temperature LSB |
| TEMP_OUT_H | 0x2F | Temperature MSB |

#### 6.1.2 Configuration

| Setting | Value | Description |
|---------|-------|-------------|
| Full scale | ±4 gauss | Earth field range |
| ODR | 10 Hz | Output data rate |
| Mode | Continuous | Operating mode |

### 6.2 Gyroscope Interface (LSM6DS3)

#### 6.2.1 Register Map

| Register | Address | Description |
|----------|---------|-------------|
| WHO_AM_I | 0x0F | Device ID (0x69) |
| CTRL1_XL | 0x10 | Accelerometer control |
| CTRL2_G | 0x11 | Gyroscope control |
| STATUS_REG | 0x1E | Status register |
| OUTX_L_G | 0x22 | Gyro X LSB |
| OUTX_H_G | 0x23 | Gyro X MSB |
| OUTY_L_G | 0x24 | Gyro Y LSB |
| OUTY_H_G | 0x25 | Gyro Y MSB |
| OUTZ_L_G | 0x26 | Gyro Z LSB |
| OUTZ_H_G | 0x27 | Gyro Z MSB |

#### 6.2.2 Configuration

| Setting | Value | Description |
|---------|-------|-------------|
| Full scale | ±250 °/s | Detumble range |
| ODR | 26 Hz | Output data rate |
| Mode | Active | Operating mode |

### 6.3 Magnetorquer Control

#### 6.3.1 PWM Command Format

| Axis | PWM Value | Current | Dipole Moment |
|------|-----------|---------|---------------|
| X | 0-255 | 0-100 mA | 0-0.2 Am² |
| Y | 0-255 | 0-100 mA | 0-0.2 Am² |
| Z | 0-255 | 0-100 mA | 0-0.2 Am² |

#### 6.3.2 Polarity Control

| PWM Range | Direction | Effect |
|-----------|-----------|--------|
| 0-127 | Negative | -M dipole |
| 128 | Off | 0 dipole |
| 129-255 | Positive | +M dipole |

### 6.4 Sun Sensor Interface

#### 6.4.1 Analog Reading

| Sensor | ADC Channel | Orientation |
|--------|-------------|-------------|
| Sun 1 | ADC0 | +X face |
| Sun 2 | ADC1 | -X face |
| Sun 3 | ADC2 | +Y face |
| Sun 4 | ADC3 | -Y face |
| Sun 5 | ADC4 | +Z face |

#### 6.4.2 Conversion

```
Sun Vector Calculation:

1. Read all 5 sun sensor ADC values
2. Normalize to 0-1 range: V_norm = ADC / 4095
3. Compute sun vector:
   S_x = (Sun_+X - Sun_-X) / max(Sun_+X, Sun_-X)
   S_y = (Sun_+Y - Sun_-Y) / max(Sun_+Y, Sun_-Y)
   S_z = Sun_+Z / max(all)
4. Normalize: S = S / |S|
```

## 7. Control Interface

### 7.1 ADCS Modes

| Mode | Description | Actuator State |
|------|-------------|----------------|
| OFF | ADCS disabled | All MTQ off |
| DETUMBLE | B-dot control | Active |
| SUNPOINT | Sun pointing | Active |
| IDLE | Sensors only | MTQ off |

### 7.2 B-dot Controller

```
B-DOT ALGORITHM

1. Read magnetometer (B vector)
2. Compute B_dot = (B_current - B_previous) / dt
3. Compute control: M = -K * B_dot
4. Command magnetorquers:
   PWM_x = scale(M_x)
   PWM_y = scale(M_y)
   PWM_z = scale(M_z)
5. Wait for sample period
6. Repeat
```

| Parameter | Value | Unit |
|-----------|-------|------|
| Control gain K | 1e6 | Am²/T/s |
| Sample period | 1 | s |
| Rate threshold | 1 | °/s |

### 7.3 Sun Pointing Controller

```
SUN POINTING ALGORITHM

1. Read sun sensors → sun vector S
2. Read magnetometer → B vector
3. Compute torque: T = S × B_ref
4. Compute dipole: M = T × B
5. Command magnetorquers
6. Repeat
```

## 8. Data Interface

### 8.1 Telemetry Parameters

| Parameter | Size | Units | Rate |
|-----------|------|-------|------|
| MAG_X | int16 | nT | 10 Hz |
| MAG_Y | int16 | nT | 10 Hz |
| MAG_Z | int16 | nT | 10 Hz |
| GYRO_X | int16 | m°/s | 26 Hz |
| GYRO_Y | int16 | m°/s | 26 Hz |
| GYRO_Z | int16 | m°/s | 26 Hz |
| SUN_VEC | 3×float | - | 1 Hz |
| ADCS_MODE | uint8 | - | 1 Hz |
| MTQ_CMD | 3×int8 | - | 1 Hz |

### 8.2 Data Formats

| Field | Format | Range |
|-------|--------|-------|
| Magnetometer | 16-bit signed | ±4 gauss |
| Gyroscope | 16-bit signed | ±250 °/s |
| Sun sensor | 12-bit unsigned | 0-4095 |
| MTQ command | 8-bit signed | -127 to +127 |

## 9. Timing Requirements

### 9.1 Sensor Timing

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Mag read time | - | 1 | 2 | ms |
| Gyro read time | - | 1 | 2 | ms |
| Sun sensor sample | - | 10 | 20 | µs |
| I2C transaction | - | 200 | 500 | µs |

### 9.2 Control Timing

| Parameter | Value | Unit |
|-----------|-------|------|
| Control loop period | 1000 | ms |
| MTQ update rate | 1 | Hz |
| Sensor polling rate | 10 | Hz |

## 10. Error Handling

### 10.1 I2C Errors

| Error | Detection | Recovery |
|-------|-----------|----------|
| NACK | No ACK received | Retry 3x, then fail |
| Timeout | No response | Reset I2C, reinit |
| Bus stuck | SDA low | Clock recovery |
| Invalid data | Range check | Use last valid |

### 10.2 Sensor Faults

| Fault | Detection | Action |
|-------|-----------|--------|
| Mag offline | WHO_AM_I fail | Disable ADCS |
| Gyro offline | WHO_AM_I fail | Use mag only |
| Sun sensor fail | All zero | Switch to mag-only |
| Saturation | Max reading | Flag in telemetry |

## 11. Verification Requirements

### 11.1 Interface Verification

| Test | Method | Criteria |
|------|--------|----------|
| I2C connectivity | Read WHO_AM_I | Correct ID |
| Sensor data | Compare to reference | ±10% |
| MTQ response | Measure current | Linear with PWM |
| Sun sensor | Known illumination | Monotonic response |

### 11.2 Functional Verification

| Test | Criteria |
|------|----------|
| Detumble | Rates <1°/s within 2 orbits |
| Sun pointing | Error <30° |
| Sensor noise | <0.5°/s gyro, <100 nT mag |

## 12. CDR Update (2026-10-27): Interfaces as Designed and as Built

Sections 4 to 11 are the 2026-01 interface. They name a LIS3MDL magnetometer and an LSM6DS3 gyro, while
`docs/hardware/ADCS_DESIGN.md` names an HMC5883L and the baseline (BASELINE §8, decision D9) the Epson M-G370PDT0 with a
Bosch BMI088 backup. Where they disagree, this section and the baseline govern.

### 12.1 Interface Status

Pins are the firmware's provisional map (`software/rust/firmware/src/`), to be fixed by the OBC schematic.

| Interface | Bus / pins | Part | Driver (firmware) | Flight software | Status |
|-----------|-----------|------|-------------------|-----------------|--------|
| Gyro, primary | SPI3 (PC10 SCK / PC11 MISO / PC12 MOSI, CS PD1; `hal_spi` bus 1), DRDY to an EXTI pin (open), RESET (open) | Epson M-G370PDT0, 3.3 V, SPI mode 3 ≤ 1 MHz, burst read of the 16-bit rate words | none (generic `hal_spi` only) | `adcs_control` takes the rate as `Option<Vec3>`; the simulated HAL supplies it | **Driver and pins open** (RID AV-03) |
| Gyro, backup | SPI3 second chip select, or I2C bus 1 (address 0x68/0x69 gyro, 0x18/0x19 accelerometer) | Bosch BMI088 | none | as above; the calibrated-bias assumption (≤ 0.1°/s) is in the Monte Carlo | **Open** (AV-03) |
| Magnetometer | I2C4 (PD12 SCL / PD13 SDA; `hal_i2c` bus 1) | not selected (ADCS_DESIGN §3.2.1) | none | `adcs_control` takes the field; sampled in the coils-off window (§12.2) | **Part, driver and address open** (AV-03, GNC-04) |
| Coarse sun sensors (6) | ADC1, six channels | photodiodes | no channels assigned in `adc.rs` (the nine assigned channels are EPS and transmitter monitors) | `adcs_control` takes the sun vector | **Channels open**: ADC1 has free inputs, or an external I2C ADC (AV-03) |
| Magnetorquers (3) | three PWM outputs and three direction lines, on a timer that ends each burst by itself (§12.2) | CR0002 rods (STR-01), DRV8833 class driver | none | `MtqCommand` (dipole, PWM, enable), the burst length | **Driver and pins open** (AV-03) |
| ADCS rail | `ADCS_EN` (PB6) | load switch | GPIO | `eps_control` | built; shares sensors and coils (no separate sensor latch-up trip, RADIATION_ANALYSIS §5.4) |
| Antenna (AntS) | I2C2 (PF1 SCL / PF0 SDA; `hal_i2c` bus 0), address 0x31 | ISIS AntS | generic `hal_i2c` | `sqso_fsw::ants`, `sqso_ffi::ants` (§12.3) | built against the published protocol; **confirm against the delivered ICD** (AV-V7) |
| GNSS receiver | LPUART1 (PG7 TX / PG8 RX, 9600), `GNSS_EN` (PE5) | not selected | UART and GPIO | `gnss.rs` (NMEA GGA), `gnss_schedule.rs` | built; receiver and its rate open (OBC_DESIGN §13) |

### 12.2 Coil and Magnetometer Timing

| Item | Value | Owner |
|------|-------|-------|
| Control period | 1000 ms | flight software |
| Coil burst | 900 ms from the command; **the driver's timer switches the coils off** at its end, whatever the software does next | firmware driver (open) |
| Settling | 50 ms after the burst; the magnetometer sample is taken at 950–1000 ms | flight software (`CoilTiming::FLIGHT`) |
| Sample rejected | taken inside the burst or the settling: not used, counted, first one logged | flight software |
| Dipole command | law × 1000/900, saturated at 0.2 A·m² per axis | flight software |

A magnetometer with its own data-ready must be configured for single-shot conversions triggered in the window, not
continuous mode, so that no conversion straddles the burst's end.

### 12.3 AntS Deployment over I2C (STR-16, LV-07)

**Decision:** the antennas are deployed by the AntS's own controller on command over I2C; the OBC's former
antenna-burn GPIO (`GPIO_PIN_ANTENNA_BURN`, PB9) is unused and held low. Reasons: the AntS is bought with its
controller and redundant release (BUILD_VS_BUY §2.6); its per-element switches give four deployment readbacks instead
of one sense line; and the inhibits are then on the AntS supply, in hardware.

| Item | Value |
|------|-------|
| Address | 0x31 (side A; side B 0x32 not used) |
| Sequence | after both wings (SRS-F089): ARM (0xAD), automatic deployment (0xA5, 10 s per element), then STATUS (0xC3) polled; on completion or the attempt limit: CANCEL (0xA9), DISARM (0xAC) |
| Status word | two bytes, low first; element n (1–4): not deployed at bit 15 − 4(n − 1), time limit reached one below, burning two below; bit 0 armed |
| Deployed | all four "not deployed" bits clear; housekeeping `DEP` carries the four-bit deployed mask |
| Retry | DEPLOY_RETRY 0 (antenna) after the 30-minute wait; attempts 50 minutes apart (SRS-F088) |
| Inhibits | the AntS supply passes the separation switches and the RBF (three inhibits, CDS 2.3.8); the arm command is a fourth, software one |
| No answer | the attempt counts as failed, and the next attempt is 50 minutes later; housekeeping shows `-` for the mask |

### 12.4 Verification Items

| ID | Item | Method | Pass |
|----|------|--------|------|
| AV-V1 | Gyro SPI read on the EM | Test | rate words match a rate table within the datasheet error |
| AV-V2 | Magnetometer in the window | Test, scope on coil current and conversion trigger | every conversion starts ≥ 50 ms after the coil current is below 1 % |
| AV-V3 | Coil driver ends its burst by itself | Test: halt the MCU mid-burst | coil current zero within 900 ms + 1 ms of the command |
| AV-V4 | Coil-to-magnetometer coupling | Test, Helmholtz cage | measured coupling entered in the Monte Carlo; the 50 ms settle holds with the rods' recoil |
| AV-V5 | Sun-sensor channels | Test, known illumination | sun vector within 10° (the Monte Carlo's degraded case) |
| AV-V6 | Spin law on the flatsat | Analysis plus HIL | the Monte Carlo cases of ADCS_DESIGN §11.4 with the flight sensors |
| AV-V7 | AntS command codes and status bits | Inspection of the delivered ICD, then test on the EM with the AntS test harness | codes and bit positions as §12.3, or `sqso_fsw::ants` changed to match before the EM software release |
| AV-V8 | GNSS receiver NMEA, cold start and power gating | Test | fix within 300 s from cold, current zero with `GNSS_EN` low |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-27 | SMART-QSO Team (CDR avionics review) | §12 new (revision history now §13): interface status against the baseline parts (gyro, magnetometer, sun sensors, coils, AntS, GNSS), coil and magnetometer timing, the AntS-over-I2C decision and protocol, verification items AV-V1 to AV-V8 |
