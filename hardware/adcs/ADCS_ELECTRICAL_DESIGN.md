# ADCS Electrical Design Specification
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-ADCS-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document specifies the electrical design requirements for the SMART-QSO ADCS (Attitude Determination and Control Subsystem) electronics, including sensor interfaces, actuator drivers, and PCB design guidelines.

---

## 2. ADCS Architecture

### 2.1 Block Diagram

```
                    ┌─────────────────────────────────────┐
                    │           OBC (I2C/SPI)             │
                    └──────────────────┬──────────────────┘
                                       │
           ┌───────────────────────────┼───────────────────────────┐
           │                           │                           │
    ┌──────▼──────┐            ┌───────▼───────┐           ┌───────▼───────┐
    │ Magnetometer│            │  Sun Sensors  │           │   Gyroscope   │
    │  (I2C)      │            │    (ADC)      │           │    (SPI)      │
    └─────────────┘            └───────────────┘           └───────────────┘

    ┌─────────────────────────────────────────────────────────────────────┐
    │                      ADCS Interface Board                           │
    │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐             │
    │  │ H-Bridge │  │ H-Bridge │  │ H-Bridge │  │  Current │             │
    │  │    X     │  │    Y     │  │    Z     │  │  Sense   │             │
    │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────────┘             │
    └───────┼─────────────┼─────────────┼─────────────────────────────────┘
            │             │             │
    ┌───────▼─────┐ ┌─────▼───────┐ ┌───▼─────────┐
    │ MTQ Coil X  │ │ MTQ Coil Y  │ │ MTQ Coil Z  │
    └─────────────┘ └─────────────┘ └─────────────┘
```

---

## 3. Sensor Interface Design

### 3.1 Magnetometer Interface

| Parameter | Specification |
|-----------|---------------|
| Sensor IC | LIS3MDL or equivalent |
| Interface | I2C (400 kHz) |
| Address | 0x1C (default) |
| Power | 3.3V, 10 mA max |
| Range | ±4 gauss default |
| Resolution | 16-bit |

**Schematic Requirements:**
| Component | Purpose |
|-----------|---------|
| 100nF decoupling | Close to VDD pin |
| 2.2µF bulk | Power stability |
| 4.7kΩ pull-ups | I2C lines (shared) |
| ESD protection | On I2C lines |

### 3.2 Sun Sensor Interface

| Parameter | Specification |
|-----------|---------------|
| Sensor type | Photodiode array (6 faces) |
| Interface | Analog to OBC ADC |
| Voltage range | 0 - 3.0V |
| Response | Linear with cos(θ) |

**Schematic Requirements (per channel):**
| Component | Purpose |
|-----------|---------|
| Photodiode | BPW34 or equivalent |
| Load resistor | 10kΩ (adjustable) |
| RC filter | 1kΩ + 100nF (16 Hz cutoff) |
| Buffer op-amp | Optional for long cables |

### 3.3 Gyroscope Interface

| Parameter | Specification |
|-----------|---------------|
| Sensor IC | LSM6DSO or equivalent |
| Interface | SPI (10 MHz max) |
| Power | 3.3V, 2 mA typical |
| Range | ±125°/s default |
| Resolution | 16-bit |
| Noise | 4 mdps/√Hz |

**Schematic Requirements:**
| Component | Purpose |
|-----------|---------|
| 100nF decoupling | Close to VDD pin |
| 4.7µF bulk | Power stability |
| SPI series resistors | 33Ω for EMI |

---

## 4. Actuator Driver Design

### 4.1 Magnetorquer Coil Specifications

| Parameter | X-Coil | Y-Coil | Z-Coil |
|-----------|--------|--------|--------|
| Magnetic moment | 0.2 A·m² | 0.2 A·m² | 0.2 A·m² |
| Coil resistance | 30Ω | 30Ω | 30Ω |
| Coil inductance | 5 mH | 5 mH | 5 mH |
| Operating current | 100 mA | 100 mA | 100 mA |
| Operating voltage | 3.3V | 3.3V | 3.3V |

### 4.2 H-Bridge Driver

| Parameter | Specification |
|-----------|---------------|
| Driver IC | DRV8833 or equivalent |
| Channels | 2 per IC (need 2 ICs) |
| Output current | 500 mA continuous |
| PWM frequency | 20 kHz |
| Protection | Overcurrent, thermal |

**Schematic Requirements (per H-bridge):**
| Component | Purpose |
|-----------|---------|
| 10µF input cap | Power decoupling |
| 100nF local cap | High-frequency filtering |
| Flyback diodes | In IC (Schottky internal) |
| Current sense resistor | 0.5Ω, 1% (optional) |

### 4.3 Current Sensing

| Parameter | Specification |
|-----------|---------------|
| Sense resistor | 0.5Ω per coil |
| Amplifier | INA199 or equivalent |
| Gain | 50 V/V |
| Output range | 0 - 3.0V |
| Interface | Analog to OBC ADC |

---

## 5. PCB Layout Guidelines

### 5.1 Board Specifications

| Parameter | Value |
|-----------|-------|
| Dimensions | 50mm × 50mm (interface board) |
| Layers | 4 |
| Thickness | 1.6mm |
| Material | FR4 |
| Copper weight | 1 oz |
| Surface finish | ENIG |

### 5.2 Design Rules

| Parameter | Minimum | Preferred |
|-----------|---------|-----------|
| Trace width (signal) | 0.15mm | 0.2mm |
| Trace width (power) | 0.5mm | 1.0mm |
| Trace spacing | 0.15mm | 0.2mm |
| Via diameter | 0.3mm | 0.4mm |

### 5.3 Layout Guidelines

| Guideline | Implementation |
|-----------|----------------|
| Magnetometer placement | Away from H-bridges |
| H-bridge ground | Dedicated return path |
| Current sense | Close to H-bridge |
| Power planes | Separate digital/power |
| Thermal relief | On power components |

### 5.4 EMC Considerations

| Issue | Mitigation |
|-------|------------|
| H-bridge switching noise | LC filter on power |
| I2C noise pickup | Twisted pair cables |
| Gyroscope vibration sensitivity | Solid mounting |
| Magnetometer interference | Physical separation |

---

## 6. Bill of Materials

### 6.1 Key Components

| Component | Primary | Alternate | Qty |
|-----------|---------|-----------|-----|
| Magnetometer | LIS3MDL | HMC5883L | 1 |
| Gyroscope | LSM6DSO | BMI160 | 1 |
| H-bridge driver | DRV8833 | TB6612FNG | 2 |
| Current sense amp | INA199 | INA181 | 3 |
| Photodiode | BPW34 | SFH213 | 6 |

### 6.2 Passive Components

| Component | Value | Package | Qty |
|-----------|-------|---------|-----|
| Decoupling cap | 100nF | 0402 | 10 |
| Bulk cap | 10µF | 0805 | 4 |
| Pull-up resistor | 4.7kΩ | 0402 | 4 |
| Sense resistor | 0.5Ω, 1% | 1206 | 3 |
| Load resistor (sun) | 10kΩ | 0603 | 6 |

---

## 7. Interface to OBC

### 7.1 Signal Connections

| Signal | Pin | Direction | Interface |
|--------|-----|-----------|-----------|
| MAG_SDA | PA10 | Bidir | I2C |
| MAG_SCL | PA9 | Out | I2C |
| GYRO_MOSI | PB5 | Out | SPI |
| GYRO_MISO | PB4 | In | SPI |
| GYRO_SCK | PB3 | Out | SPI |
| GYRO_CS | PB6 | Out | GPIO |
| MTQ_X_FWD | PC0 | Out | PWM |
| MTQ_X_REV | PC1 | Out | PWM |
| MTQ_Y_FWD | PC2 | Out | PWM |
| MTQ_Y_REV | PC3 | Out | PWM |
| MTQ_Z_FWD | PC4 | Out | PWM |
| MTQ_Z_REV | PC5 | Out | PWM |
| SUN_1 | PA0 | In | ADC |
| SUN_2 | PA1 | In | ADC |
| SUN_3 | PA2 | In | ADC |
| SUN_4 | PA3 | In | ADC |
| SUN_5 | PA4 | In | ADC |
| SUN_6 | PA5 | In | ADC |
| MTQ_I_X | PA6 | In | ADC |
| MTQ_I_Y | PA7 | In | ADC |
| MTQ_I_Z | PB0 | In | ADC |

### 7.2 Connector

| Connector | Type | Pins |
|-----------|------|------|
| J1 | Samtec FTSH | 20-pin |

---

## 8. Test Points

| Test Point | Signal | Purpose |
|------------|--------|---------|
| TP1 | 3.3V | Power |
| TP2 | GND | Ground |
| TP3 | MAG_SDA | I2C debug |
| TP4 | MAG_SCL | I2C debug |
| TP5 | MTQ_X_OUT+ | Coil current |
| TP6 | MTQ_X_OUT- | Coil current |
| TP7 | SUN_1 | Sun sensor debug |
| TP8 | MTQ_I_X | Current sense |

---

## 9. Coil Winding Specifications

### 9.1 Air-Core Magnetorquer

| Parameter | Specification |
|-----------|---------------|
| Core | Air (no core) |
| Wire | 32 AWG magnet wire |
| Turns | 400 turns |
| Form | Rectangular, 80mm × 80mm |
| Resistance | 30Ω ±10% |
| Inductance | 5 mH typical |

### 9.2 Winding Notes

| Item | Requirement |
|------|-------------|
| Termination | Solder to PCB pads |
| Potting | Optional RTV |
| Marking | Polarity indicated |
| Testing | Resistance and inductance |

---

## 10. Deliverables Checklist

| Deliverable | Format | Status |
|-------------|--------|--------|
| Schematic PDF | PDF | Required |
| Schematic source | KiCad/OrCAD | Required |
| BOM | Excel/CSV | Required |
| PCB layout | Gerber | Required |
| Assembly drawing | PDF | Required |
| Coil winding spec | PDF | Required |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
