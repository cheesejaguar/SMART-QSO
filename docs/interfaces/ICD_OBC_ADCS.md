# Interface Control Document: OBC to ADCS Subsystem

**Document ID**: SQSO-ICD-004
**Version**: 1.0
**Date**: 2026-01-02
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

## 12. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
