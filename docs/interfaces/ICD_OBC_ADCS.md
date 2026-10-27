# Interface Control Document: OBC to ADCS Subsystem

**Document ID**: SQSO-ICD-004
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the interface between the On-Board Computer (OBC) and the Attitude Determination and Control Subsystem (ADCS) for the SMART-QSO CubeSat per NPR 7123.1 SE Process 5.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-SRS-001 | System Requirements Specification |
| SQSO-HW-ADCS-001 | ADCS Design (`docs/hardware/ADCS_DESIGN.md`) |
| SQSO-HW-OBC-001 | OBC Design |
| SQSO-OBC-BRD-001 | OBC board build-to design (`hardware/electrical/obc/OBC_BOARD_DESIGN.md`) §9–§10 |
| SQSO-ICX-001 | Spacecraft electrical interconnect, master pin allocation (`hardware/electrical/INTERCONNECT.md`) §3.4, §4 |

## 3. Interface Summary

There is no ADCS board. The gyro module plugs into socket X1 on the OBC board; the backup gyro, the magnetometer, the
sun-sensor amplifiers and their ADC, and the three magnetorquer H-bridges are parts of the OBC board (SQSO-ICX-001 §2).
What leaves the OBC board is the sensor supply from the EPS (W6B), the three rods (W6C) and the six panel photodiodes
(W6D). Pins are those of SQSO-ICX-001 §4, which governs.

```
INTERFACE BLOCK DIAGRAM

┌──────────────────────────────── OBC board ─────────────────────────────────┐
│  STM32L4R5ZI                                                                │
│   SPI3 ──── Epson M-G370PDT0 gyro (X1)        SPI2 ──── Bosch BMI088 backup │
│   I2C4 ──┬─ PNI RM3100 magnetometer 0x20                                    │
│          └─ ADS7138-Q1 ADC 0x10 ◄── 6 transimpedance amplifiers ◄── W6D     │
│   TIM1 CH1–CH3 PWM, TIM15 burst ──── 3 × DRV8876-Q1 H-bridges ──── W6C rods │
│   I2C1 ──── RV-3028-C7 mission clock 0x52     PG11 ──── TPS3850 supervisor  │
│   PF6/PF7/PF8 ──── three latch-up sensor-rail switches ◄── W6B 3V3_ADCS     │
└─────────────────────────────────────────────────────────────────────────────┘
     ▲ W6B: 3V3_ADCS from EPS SW4        ▲ W3: VBAT_MTQ from EPS SW11 (ADCS_EN PB6)
```

## 4. Physical Interface

### 4.1 Connector Definition

| Interface | OBC connector | Far end | Harness | Gauge |
|-----------|---------------|---------|---------|-------|
| Sensor supply 3V3_ADCS | J6, Pico-Clasp 2 | EPS J20 (SW4) | W6B | 26 AWG |
| Magnetorquer rods | J7, Pico-Clasp 6 | CubeSpace CR0002 rods X, Y, Z | W6C (twisted pairs) | 26 AWG |
| Coarse sun sensors | J8, Pico-Clasp 14 | six panel photodiodes | W6D (twisted pairs under one shield) | 28 AWG |
| Gyro module | X1, Samtec CLM-110-02-L-D socket | Epson M-G370PDT0 (FTMH-110 header) | none | — |
| Coil supply VBAT_MTQ | J1-36, J1-39 (W3 from EPS J3) | EPS SW11 | W3 | 28 AWG |

### 4.2 Pin Assignment

Harness pins (SQSO-ICX-001 §3.4):

| Pin | Signal | Level / current | OBC part | Far end |
|-----|--------|-----------------|----------|---------|
| J6-1 | 3V3_ADCS | 3.3 V, 0.049 A worst normal; EPS SW4 trip 0.265–0.339 A | U5, U6, U7 TPS2553-Q1 (50–100 mA latch-up trips) | EPS SW4 (ADCS_EN PB6) |
| J6-2 | GND | — | — | — |
| J7-1/2 | MTQ_X_P / MTQ_X_N | ± 0.087 A at 0.2 A·m², hardware limit 0.104–0.126 A | U22 DRV8876-Q1 OUT1/OUT2 | rod X |
| J7-3/4 | MTQ_Y_P / MTQ_Y_N | as X | U23 | rod Y |
| J7-5/6 | MTQ_Z_P / MTQ_Z_N | as X | U24 | rod Z |
| J8-1/2 … 11/12 | CSS_PX_K/A, MX, PY, MY, PZ, MZ | photocurrent ≤ 4.0 mA | U17–U19 TLV9002-Q1 (619 Ω) → U16 ADS7138-Q1 AIN0–AIN5 | panel photodiodes |
| J8-13/14 | shield drains | — | GND at J8 only | — |

MCU pins (STM32L4R5ZIT6, SQSO-ICX-001 §4.1, §4.3):

| Function | Pins | Peripheral |
|----------|------|------------|
| Primary gyro M-G370PDT0 | SCK PC10, MISO PC11, MOSI PC12, CS PD1; DRDY PD10 (EXTI10); /RST PD14 | SPI3 (`hal_spi` bus 1), mode 3, ≤ 1 MHz burst; through U9 TXU0304-Q1 and U10 TXU0204-Q1 |
| Backup gyro BMI088 | SCK PB13, MISO PB14, MOSI PB15; gyro CS PD0, accelerometer CS PD4; INT3 PD15 (EXTI15) | SPI2 (`hal_spi` bus 0), ≤ 10 MHz; through U12 TXU0304-Q1 and U13 TXU0204-Q1 |
| Magnetometer RM3100 (0x20) and sun-sensor ADC ADS7138 (0x10) | SCL PD12, SDA PD13; MAG_DRDY PD11 (EXTI11) | I2C4 (`hal_i2c` bus 1), through U14 TCA9517-Q1 |
| Magnetorquer drive | PWM (EN) PE9, PE11, PE13; direction (PH) PE8, PE10, PE12; MTQ_ARM PF9; MTQ_FAULT_N PE14 (EXTI14); MTQ_EXPIRED PE15 | TIM1 CH1–CH3 (40 kHz); TIM15 CH1 one-pulse 900 ms |
| Coil current telemetry | MTQ_X_I PA4, MTQ_Y_I PA5, MTQ_Z_I PA6 | ADC1 IN9–IN11 (10 mV/mA) |
| Sensor-rail switches and trip readbacks | GYRO_EN PF6, IMU2_EN PF7, SNS_EN PF8; GYRO_RB PF10, IMU2_RB PF11, SNS_RB PF12 | GPIO |
| Mission clock RV-3028-C7 (0x52) | SCL PG14, SDA PG13; 1 Hz CLKOUT to PC8 | I2C1 (new `hal_i2c` bus 2); TIM3 CH3 capture |
| External supervisor TPS3850 | WDI PG11 (falling edge every 10 s) | GPIO |
| ADCS power | ADCS_EN PB6 (J1-11): EPS SW4 (3V3_ADCS) and SW11 (VBAT_MTQ) | GPIO |

### 4.3 Cable Requirements

| Parameter | Requirement |
|-----------|-------------|
| Length | ≤100 mm |
| Type | Twisted pairs; W6D shielded, drains grounded at the OBC end only |
| AWG | 26 AWG (W6B, W6C), 28 AWG (W6D) |
| I2C pull-ups | 2.2 kΩ on the OBC (I2C4), 4.7 kΩ (I2C1) |

## 5. Electrical Interface

### 5.1 I2C Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Clock frequency | - | 100 | 400 | kHz |
| Data rise time | - | - | 1000 | ns |
| Data fall time | - | - | 300 | ns |
| Pull-up resistance | 2.2 | - | 4.7 | kΩ |
| Bus capacitance | - | - | 400 | pF |

### 5.2 I2C and SPI Devices

| Device | Bus | Address / select | R/W |
|--------|-----|------------------|-----|
| Magnetometer PNI RM3100 | I2C4 | 0x20 | R/W |
| Sun-sensor ADC TI ADS7138-Q1 (AIN0–5 sun sensors, AIN6 TMP235, AIN7 VBAT_MTQ / 6) | I2C4 | 0x10 | R/W |
| Mission clock RV-3028-C7 | I2C1 | 0x52 | R/W |
| Gyroscope Epson M-G370PDT0 | SPI3 | CS PD1 | R/W |
| Backup gyroscope Bosch BMI088 (gyro / accelerometer) | SPI2 | CS PD0 / PD4 | R/W |

### 5.3 PWM Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| PWM frequency | - | 40 | 100 | kHz |
| Duty cycle range | 0 | - | 100 | % |
| Rise time | - | - | 100 | ns |
| Logic high | 2.4 | 3.3 | - | V |
| Logic low | - | 0 | 0.4 | V |

The bridges run in PH/EN mode from VBAT_MTQ (11.2–16.4 V): the duty for 0.2 A·m² (87 mA in a 51 Ω rod) is 0.41 at
11.2 V and 0.28 at 16.4 V; each bridge limits its rod to 104–126 mA in hardware (SQSO-OBC-BRD-001 §10).

### 5.4 Analog Interface

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Sun sensor output (transimpedance, 619 Ω) | 0 | - | 2.48 at 4.0 mA | V |
| ADC resolution (ADS7138) | 12 | - | - | bits |
| Sample rate | 1 | - | 100 | Hz |

## 6. Logical Interface

The LIS3MDL and LSM6DS3 register maps of the first issue are withdrawn: neither part is in the design. The parts are
the PNI RM3100 (single-shot measurement on command in the coils-off window, end marked by DRDY; SQSO-OBC-BRD-001 §9.4)
and the Epson M-G370PDT0 (SPI burst read of the 16-bit rate words; §9.2) with the BMI088 as cold spare (§9.3). Their
drivers and register use are to be written with the flight-software change of SQSO-ICX-001 §6; the maps below are kept
only as the record of the first issue.

### 6.1 Magnetometer Interface (LIS3MDL, withdrawn)

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

### 6.2 Gyroscope Interface (LSM6DS3, withdrawn)

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
| X | 0-255 | 0-87 mA | 0-0.2 Am² |
| Y | 0-255 | 0-87 mA | 0-0.2 Am² |
| Z | 0-255 | 0-87 mA | 0-0.2 Am² |

#### 6.3.2 Polarity Control

| PWM Range | Direction | Effect |
|-----------|-----------|--------|
| 0-127 | Negative | -M dipole |
| 128 | Off | 0 dipole |
| 129-255 | Positive | +M dipole |

### 6.4 Sun Sensor Interface

#### 6.4.1 Analog Reading

| Sensor | ADC Channel (ADS7138) | Orientation |
|--------|-----------------------|-------------|
| Sun 1 | AIN0 | +X face |
| Sun 2 | AIN1 | -X face |
| Sun 3 | AIN2 | +Y face |
| Sun 4 | AIN3 | -Y face |
| Sun 5 | AIN4 | +Z face |
| Sun 6 | AIN5 | -Z face |

#### 6.4.2 Conversion

```
Sun Vector Calculation:

1. Read all 6 sun sensor ADC values
2. Normalize to 0-1 range: V_norm = ADC / 4095
3. Compute sun vector:
   S_x = (Sun_+X - Sun_-X) / max(Sun_+X, Sun_-X)
   S_y = (Sun_+Y - Sun_-Y) / max(Sun_+Y, Sun_-Y)
   S_z = (Sun_+Z - Sun_-Z) / max(Sun_+Z, Sun_-Z)
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
| Control gain K | 1e4 | A·m²·s/T |
| Sample period | 1 | s |
| Rate threshold | 0.5 (rate across +Y, held 5 min) | °/s |

### 7.3 Sun Pointing Controller

```
SUN POINTING ALGORITHM (spin-stabilized, BASELINE §8)

1. Read sun sensors → sun vector S; read magnetometer → B vector; gyro rate ω
2. Momentum target: h_s = J ω_s S, spin ω_s = 8 °/s about +Y
3. Torque: momentum error + nutation damping (no rate across +Y) + spin hold
4. Compute dipole: M = B × T / |B|²
5. Command magnetorquers (900 ms burst, saturated at 0.2 A·m²)
6. Repeat
```

Gains and the Monte Carlo are in `docs/hardware/ADCS_DESIGN.md` (`sqso_fsw::adcs::Gains::FLIGHT`).

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
| Detumble | From 10°/s, rate across +Y < 0.5°/s within 3 orbits |
| Sun pointing | +Y within 15° of the Sun for ≥ 90 % of sunlit time (spin 8°/s) |
| Sensor noise | <0.5°/s gyro, <100 nT mag |

## 12. CDR Update (2026-10-27): Interfaces as Designed and as Built

Sections 4 and 5 give the interface as designed (Version 1.2, from SQSO-OBC-BRD-001 and SQSO-ICX-001). Sections 6 to 11
keep parts of the 2026-01 interface, which named a LIS3MDL magnetometer and an LSM6DS3 gyro; the baseline (BASELINE §8,
decision D9) has the Epson M-G370PDT0 with a Bosch BMI088 backup, and the OBC board the PNI RM3100. Where they
disagree, this section, the board design and the baseline govern.

### 12.1 Interface Status

Pins are the master allocation of SQSO-ICX-001 §4 (from the OBC schematic). Since Version 1.3 the firmware
(`software/rust/firmware/src/`) follows it: every row of SQSO-ICX-001 §6 is implemented (flight software round 3), and
the drivers are built against the data sheets and tested with register-level mocks (`software/rust/sqso-fsw/src/`:
`mg370.rs`, `bmi088.rs`, `rm3100.rs`, `ads7138.rs`, `mtq.rs`, `rv3028.rs`; the sequencing in `adcs_sensors.rs`). No
driver has run on hardware: AV-V1 to AV-V5 verify them on the EM.

| Interface | Bus / pins | Part | Driver (firmware) | Flight software | Status |
|-----------|-----------|------|-------------------|-----------------|--------|
| Gyro, primary | SPI3 (PC10 SCK / PC11 MISO / PC12 MOSI, CS PD1; `hal_spi` bus 1), DRDY PD10, /RST PD14, rail GYRO_EN PF6 (readback PF10) | Epson M-G370PDT0, 3.3 V, SPI mode 3 ≤ 1 MHz, burst read of the 16-bit rate words | `sqso_fsw::mg370`: /RST, ID check (0x5345), self-test at power-on, burst read on DRDY | the rate through the gyro alignment matrix (SRS-F142); three failures in a row cycle the rail, then the BMI088 (SRS-F140); the simulated HAL supplies the rate on the host | Built, not run on hardware (AV-03, AV-V1) |
| Gyro, backup | SPI2 (PB13 SCK / PB14 MISO / PB15 MOSI; `hal_spi` bus 0), gyro CS PD0, accelerometer CS PD4, INT3 PD15, rail IMU2_EN PF7 (readback PF11) | Bosch BMI088, cold spare | `sqso_fsw::bmi088`: chip IDs, the gyro and the accelerometer on their own chip selects (each `hal_spi` bus keeps its own mode) | as above; the calibrated-bias assumption (≤ 0.1°/s) is in the Monte Carlo | Built, not run on hardware (AV-03) |
| Magnetometer | I2C4 (PD12 SCL / PD13 SDA; `hal_i2c` bus 1) through U14, address 0x20, MAG_DRDY PD11, rail SNS_EN PF8 (readback PF12) | PNI RM3100 on the OBC board (SQSO-OBC-BRD-001 §9.4); a bought module fits J10 if AV-V4 fails | `sqso_fsw::rm3100`: REVID 0x22, cycle count 200 (75 counts/µT), single measurement triggered in the coils-off window, read on MAG_DRDY | the field less the bias of the rods' last polarity state (SRS-F144); not used while JET_PG or PTT_EXCITER is high (SRS-F143) | Built, not run on hardware (AV-03, AV-V2) |
| Coarse sun sensors (6) | six TLV9002-Q1 transimpedance amplifiers into the ADS7138-Q1 (I2C4, 0x10, AIN0–AIN5) | panel photodiodes (≤ 4.0 mA at AM0) | `sqso_fsw::ads7138`: AIN0–AIN5 through the 619 Ω transimpedance (4.0 mA full sun), AIN6 TMP235, AIN7 VBAT_MTQ/6 | `adcs_control` takes the sun vector | Built, not run on hardware (AV-03, AV-V5) |
| Magnetorquers (3) | PWM TIM1 CH1–CH3 (PE9, PE11, PE13), direction PE8, PE10, PE12, MTQ_ARM TIM15 CH1 one-pulse 900 ms (PF9), MTQ_FAULT_N PE14, MTQ_EXPIRED PE15, coil currents PA4–PA6; hardware off-timer 0.92–0.98 s (§12.2) | CR0002 rods (STR-01), three DRV8876-Q1 bridges on VBAT_MTQ | `firmware/src/mtq.rs` (TIM1 PWM 40 kHz, TIM15 one-pulse; `hal_mtq.h`), `sqso_fsw::mtq` (direction and duty from the dipole, rod 2.3 A·m²/A, 51 Ω) | the direction lines set before the burst, the burst timed by TIM15 (SRS-F141); MTQ_FAULT_N and MTQ_EXPIRED logged at onset and clearance; the coil currents (PA4–PA6) checked at the magnetometer sample | Built, not run on hardware (AV-03, AV-V3) |
| ADCS rails | `ADCS_EN` (PB6): EPS SW4 3V3_ADCS (sensors, 0.30 A) and SW11 VBAT_MTQ (coils, 0.361 A); the OBC's own sensor switches U5–U7 (50–100 mA) | load switches | GPIO | `eps_control` | Designed: sensors and coils on separate switches, each sensor with its own latch-up trip (RADIATION_ANALYSIS §5.3) |
| Antenna (AntS) | I2C2 (PF1 SCL / PF0 SDA; `hal_i2c` bus 0), address 0x31 | ISIS AntS | generic `hal_i2c` | `sqso_fsw::ants`, `sqso_ffi::ants` (§12.3) | built against the published protocol; **confirm against the delivered ICD** (AV-V7) |
| GNSS receiver | LPUART1 (PG7 TX / PG8 RX, 9600) through U30/U31, `GNSS_EN` (PE5), 1PPS PC6 (TIM3_CH1 capture), GNSS_PG PG15 | SkyFox piNAV-NG/FM (D6) | UART and GPIO | `gnss.rs` (NMEA GGA), `gnss_schedule.rs` | built; GNSS_PG checked 1 s after GNSS_EN, the 1PPS period captured on TIM3 CH1 (`hal_capture.h`) |
| Mission clock | I2C1 (`hal_i2c` bus 2), address 0x52; CLKOUT 1 Hz to PC8 (TIM3 CH3 capture) | Micro Crystal RV-3028-C7 | `sqso_fsw::rv3028` | the reset time stamp at every boot, the MCU clock checked against the 1 Hz every 60 s (SRS-F145) | Built, not run on hardware |
| Supervisor | WDI PG11 | TI TPS3850 | GPIO | a falling edge every 10 s while every task checks in (SRS-F146) | Built |

### 12.2 Coil and Magnetometer Timing

| Item | Value | Owner |
|------|-------|-------|
| Control period | 1000 ms | flight software |
| Coil burst | 900 ms from the command; **TIM15 in one-pulse mode switches the coils off** at its end, whatever the software does next, and a hardware off-timer stops them at 0.92–0.98 s if it does not | OBC board (SQSO-OBC-BRD-001 §10.3); firmware `mtq.rs` (TIM15 one-pulse PWM mode 2) |
| Settling | 50 ms after the burst; the magnetometer sample is taken at 950–1000 ms | flight software (`CoilTiming::FLIGHT`) |
| Sample rejected | taken inside the burst or the settling: not used, counted, first one logged | flight software |
| Dipole command | law × 1000/900, saturated at 0.2 A·m² per axis | flight software |

A magnetometer with its own data-ready must be configured for single-shot conversions triggered in the window, not
continuous mode, so that no conversion straddles the burst's end.

**Calibration by telecommand (Version 1.3).** ADCS_SET_GYRO_ALIGN (0x25) sets the gyro alignment matrix (nine
elements, signed 32-bit big-endian in units of 1e-6; every element within 1.1 and the determinant within 0.05 of 1;
VI-MCAD-05) and ADCS_SET_MAG_BIAS (0x26) the magnetometer bias of one of the eight polarity states of the rods (bit n
set: rod n last driven positive; X, Y, Z signed 32-bit big-endian nT, each within 50 µT; AV-V4). Both are stored under
CRC in FLASH_REGION_ADCS_CAL (FRAM 0x36880, 256 B) and loaded at every boot; identity and zeros when no record is
stored (SQSO-ICD-013 §7).

### 12.3 AntS Deployment over I2C (STR-16, LV-07)

**Decision:** the antennas are deployed by the AntS's own controller on command over I2C; the OBC's former
antenna-burn GPIO (`GPIO_PIN_ANTENNA_BURN`, PB9) is reassigned to the wing 2 resistor A burn line BURN_W2A
(SQSO-ICX-001 §3.1, §6 row 4). Reasons: the AntS is bought with its
controller and redundant release (BUILD_VS_BUY §2.6); its per-element switches give four deployment readbacks instead
of one sense line; and the inhibits are then on the AntS supply, in hardware.

| Item | Value |
|------|-------|
| Address | 0x31 (side A; side B 0x32 not used) |
| Sequence | after both wings (SRS-F089): ARM (0xAD), automatic deployment (0xA5, 10 s per element), then STATUS (0xC3) polled; on completion or the attempt limit: CANCEL (0xA9), DISARM (0xAC) |
| Status word | two bytes, low first; element n (1–4): not deployed at bit 15 − 4(n − 1), time limit reached one below, burning two below; bit 0 armed |
| Deployed | all four "not deployed" bits clear; housekeeping `DEP` carries the four-bit deployed mask |
| Retry | DEPLOY_RETRY 0 (antenna) after the 30-minute wait; attempts 50 minutes apart (SRS-F088) |
| Inhibits | the AntS supply 3V3_ANTS (EPS SW8 on DEPLOY_EN PF3) is downstream of the three deployment switches DS1–DS3 (three physical inhibits, CDS 2.3.8; the RBF does not count); the arm command is a fourth, software one |
| No answer | the attempt counts as failed, and the next attempt is 50 minutes later; housekeeping shows `-` for the mask |

### 12.4 Verification Items

| ID | Item | Method | Pass |
|----|------|--------|------|
| AV-V1 | Gyro SPI read on the EM | Test | rate words match a rate table within the datasheet error |
| AV-V2 | Magnetometer in the window | Test, scope on coil current and conversion trigger | every conversion starts ≥ 50 ms after the coil current is below 1 % |
| AV-V3 | Coil driver ends its burst by itself | Test: halt the MCU mid-burst | coil current zero within 900 ms + 2 ms of the command (bridge sleep up to 1 ms, coil decay up to 0.9 ms) |
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
| 1.2 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: §2–§5 interface as designed (no ADCS board; harness W6B/W6C/W6D and the STM32 pins of SQSO-ICX-001: M-G370 on SPI3, BMI088 on SPI2, RM3100 and ADS7138 on I2C4, magnetorquers on TIM1/TIM15, RV-3028 on I2C1, supervisor kick PG11); LIS3MDL/LSM6DS3 maps withdrawn (§6); six sun sensors; B-dot gain 10⁴, detumble and 8 °/s spin criteria from BASELINE §8 (§7, §11); §12.1 pins allocated; AV-V3 criterion 2 ms (SQSO-OBC-BRD-001 §24 C4, C5) |
| 1.3 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3) | §12.1 the software side as built: the firmware on the master pin allocation (SQSO-ICX-001 §6), the drivers of the gyros, magnetometer, sun-sensor ADC, magnetorquers and mission clock (built against the data sheets, not run on hardware), the magnetometer gating and bias, the supervisor; the calibration telecommands ADCS_SET_GYRO_ALIGN and ADCS_SET_MAG_BIAS |
