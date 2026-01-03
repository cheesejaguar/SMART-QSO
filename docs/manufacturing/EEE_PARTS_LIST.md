# EEE Parts List
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-MFG-002
**Revision:** A
**Date:** 2026-01-02
**Reference:** EEE-INST-002, MIL-STD-975, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document provides a comprehensive list of Electrical, Electronic, and Electromechanical (EEE) parts used in the SMART-QSO CubeSat, including part specifications, derating, and radiation tolerance.

## 2. Scope

This document covers:
- All EEE parts in flight configuration
- Part specifications and manufacturers
- Derating requirements
- Radiation tolerance data
- Approved alternatives

---

## 3. Parts Classification

### 3.1 Grade Levels

| Grade | Description | Application |
|-------|-------------|-------------|
| Level 1 | Space-qualified (Class S) | Critical functions |
| Level 2 | Hi-rel commercial | Primary functions |
| Level 3 | Industrial grade | Non-critical functions |
| COTS | Commercial off-the-shelf | Payload, non-flight-critical |

### 3.2 Part Categories

| Category | Count | Notes |
|----------|-------|-------|
| Microcontrollers | 2 | OBC, backup |
| Memory (Flash) | 2 | Code, data storage |
| Memory (SRAM) | 1 | Working memory |
| Power Regulators | 6 | Various rails |
| Battery | 1 | Li-ion pack |
| RF Components | 8 | Transceiver, PA, filters |
| Sensors | 12 | Magnetometer, sun, temp |
| Passives | ~150 | R, C, L |
| Connectors | 8 | PC104, debug |

---

## 4. Microcontrollers

### 4.1 Primary OBC

| Parameter | Value |
|-----------|-------|
| Part Number | STM32L4R5ZIT6 |
| Manufacturer | STMicroelectronics |
| Description | Arm Cortex-M4 MCU, 2MB Flash |
| Package | LQFP144 |
| Grade | Industrial (-40°C to +85°C) |
| Quantity | 1 |
| TID Tolerance | ~30 krad (tested) |
| SEL Threshold | >60 MeV-cm²/mg |
| Derating | 80% max clock |

### 4.2 Backup/Watchdog MCU

| Parameter | Value |
|-----------|-------|
| Part Number | MSP430FR5969 |
| Manufacturer | Texas Instruments |
| Description | Ultra-low-power MCU |
| Package | VQFN-48 |
| Grade | Industrial |
| Quantity | 1 |
| TID Tolerance | ~20 krad |
| Function | Watchdog, safe mode |

---

## 5. Memory Devices

### 5.1 Code Flash

| Parameter | Value |
|-----------|-------|
| Part Number | Internal to STM32L4R5 |
| Type | Embedded Flash |
| Capacity | 2 MB |
| TID Tolerance | ~30 krad |
| Notes | Triple-voted critical sections |

### 5.2 Data Storage Flash

| Parameter | Value |
|-----------|-------|
| Part Number | W25Q128JVSIQ |
| Manufacturer | Winbond |
| Description | 128Mbit SPI Flash |
| Package | SOIC-8 |
| Grade | Industrial |
| Quantity | 2 (redundant) |
| TID Tolerance | ~15 krad |
| Derating | EDAC enabled |

### 5.3 SRAM

| Parameter | Value |
|-----------|-------|
| Part Number | Internal to STM32L4R5 |
| Type | Embedded SRAM |
| Capacity | 640 KB |
| TID Tolerance | ~30 krad |
| Notes | ECC enabled on critical regions |

---

## 6. Power Management

### 6.1 Battery Management IC

| Parameter | Value |
|-----------|-------|
| Part Number | BQ25703A |
| Manufacturer | Texas Instruments |
| Description | Battery charger/manager |
| Package | QFN-32 |
| Grade | Industrial |
| Quantity | 1 |
| TID Tolerance | ~10 krad |

### 6.2 3.3V Regulator (Primary)

| Parameter | Value |
|-----------|-------|
| Part Number | TPS62840 |
| Manufacturer | Texas Instruments |
| Description | 750mA DC-DC converter |
| Package | VSON-6 |
| Grade | Industrial |
| Quantity | 2 (redundant) |
| Efficiency | >90% |
| Derating | 70% max load |

### 6.3 5V Regulator

| Parameter | Value |
|-----------|-------|
| Part Number | TPS63070 |
| Manufacturer | Texas Instruments |
| Description | 2A buck-boost converter |
| Package | VQFN-14 |
| Grade | Industrial |
| Quantity | 1 |
| Derating | 60% max load |

### 6.4 Battery Pack

| Parameter | Value |
|-----------|-------|
| Part Number | Custom 2S2P |
| Cell Type | Panasonic NCR18650GA |
| Capacity | 5200 mAh |
| Voltage | 7.2V nominal |
| Quantity | 1 pack (4 cells) |
| Protection | OV, UV, OC, OT |

---

## 7. RF Components

### 7.1 Transceiver IC

| Parameter | Value |
|-----------|-------|
| Part Number | CC1125 |
| Manufacturer | Texas Instruments |
| Description | Sub-1GHz RF transceiver |
| Package | QFN-32 |
| Grade | Industrial |
| Quantity | 1 |
| Frequency | 145/435 MHz |
| TID Tolerance | ~15 krad |

### 7.2 Power Amplifier

| Parameter | Value |
|-----------|-------|
| Part Number | RF5110G |
| Manufacturer | Qorvo |
| Description | 3W linear PA |
| Package | QFN-16 |
| Grade | Industrial |
| Quantity | 1 |
| Output Power | 0.5-2W (configurable) |

### 7.3 LNA

| Parameter | Value |
|-----------|-------|
| Part Number | SKY67151-396LF |
| Manufacturer | Skyworks |
| Description | Low-noise amplifier |
| Package | QFN-6 |
| Grade | Industrial |
| Quantity | 1 |
| NF | 0.6 dB |

### 7.4 SAW Filters

| Parameter | Value |
|-----------|-------|
| Part Number | SF2049E (145 MHz), SF2098E (435 MHz) |
| Manufacturer | Murata |
| Description | SAW bandpass filters |
| Package | SMD |
| Quantity | 2 each |

---

## 8. Sensors

### 8.1 Magnetometer

| Parameter | Value |
|-----------|-------|
| Part Number | LIS3MDL |
| Manufacturer | STMicroelectronics |
| Description | 3-axis magnetometer |
| Package | LGA-12 |
| Grade | Industrial |
| Quantity | 2 (redundant) |
| Range | ±16 gauss |
| Interface | I2C |

### 8.2 Sun Sensors (Photodiodes)

| Parameter | Value |
|-----------|-------|
| Part Number | TEMD6200FX01 |
| Manufacturer | Vishay |
| Description | Silicon photodiode |
| Package | 0805 SMD |
| Grade | Industrial |
| Quantity | 6 (one per face) |
| Spectral Response | 440-1050 nm |

### 8.3 Temperature Sensors

| Parameter | Value |
|-----------|-------|
| Part Number | TMP117 |
| Manufacturer | Texas Instruments |
| Description | ±0.1°C digital temp sensor |
| Package | WSON-6 |
| Grade | Industrial |
| Quantity | 6 |
| Interface | I2C |
| Range | -55°C to +150°C |

### 8.4 Current Sensors

| Parameter | Value |
|-----------|-------|
| Part Number | INA226 |
| Manufacturer | Texas Instruments |
| Description | Current/power monitor |
| Package | VSSOP-10 |
| Grade | Industrial |
| Quantity | 4 |
| Interface | I2C |

---

## 9. Actuators

### 9.1 Magnetorquer Driver

| Parameter | Value |
|-----------|-------|
| Part Number | DRV8837 |
| Manufacturer | Texas Instruments |
| Description | H-bridge motor driver |
| Package | WSON-8 |
| Grade | Industrial |
| Quantity | 3 (X, Y, Z axes) |
| Current | 1.8A peak |

### 9.2 Antenna Deployment

| Parameter | Value |
|-----------|-------|
| Part Number | Nichrome wire |
| Type | Burn wire |
| Resistance | 2 ohm |
| Quantity | 2 (redundant) |
| Activation | 1A for 5 seconds |

---

## 10. Connectors

### 10.1 PC/104 Stack Connector

| Parameter | Value |
|-----------|-------|
| Part Number | ESQ-126-39-G-D |
| Manufacturer | Samtec |
| Description | 104-pin stack connector |
| Quantity | 2 per board |

### 10.2 Debug Connector

| Parameter | Value |
|-----------|-------|
| Part Number | FTSH-105-01-L-DV-K |
| Manufacturer | Samtec |
| Description | 10-pin SWD header |
| Quantity | 1 |

### 10.3 RF Connector

| Parameter | Value |
|-----------|-------|
| Part Number | U.FL-R-SMT |
| Manufacturer | Hirose |
| Description | Micro coax connector |
| Quantity | 2 |

---

## 11. Passive Components

### 11.1 Resistors

| Type | Value Range | Package | Tolerance | Quantity |
|------|-------------|---------|-----------|----------|
| Thick film | 10Ω - 1MΩ | 0402, 0603 | 1% | ~80 |
| Thin film | 1kΩ - 100kΩ | 0603 | 0.1% | ~10 |

### 11.2 Capacitors

| Type | Value Range | Package | Voltage | Quantity |
|------|-------------|---------|---------|----------|
| MLCC (C0G) | 10pF - 100nF | 0402, 0603 | 25V | ~40 |
| MLCC (X7R) | 100nF - 10µF | 0603, 0805 | 16V | ~30 |
| Tantalum | 10µF - 100µF | 1206, 1210 | 10V | ~10 |

### 11.3 Inductors

| Type | Value Range | Package | DCR | Quantity |
|------|-------------|---------|-----|----------|
| Chip inductor | 1µH - 100µH | 0603, 0805 | <1Ω | ~15 |
| Power inductor | 2.2µH - 22µH | Shielded | <50mΩ | ~5 |

---

## 12. Derating Requirements

### 12.1 Derating Table

| Component | Parameter | Derating |
|-----------|-----------|----------|
| Resistors | Power | 50% |
| Capacitors (MLCC) | Voltage | 50% |
| Capacitors (Tantalum) | Voltage | 60% |
| Transistors | Power | 50% |
| Voltage regulators | Current | 70% |
| ICs | Junction temp | 80% of max |

### 12.2 Thermal Derating

| Condition | Additional Derating |
|-----------|---------------------|
| Hot case (+60°C) | +10% |
| Vacuum operation | Power components +20% |
| Radiation environment | Per part analysis |

---

## 13. Radiation Considerations

### 13.1 Mission Environment

| Parameter | Value |
|-----------|-------|
| Orbit altitude | 400-500 km |
| Inclination | TBD (depends on launch) |
| Mission duration | 12-18 months |
| Total Ionizing Dose | ~5 krad (with 2mm Al) |
| SEE flux | Low (LEO) |

### 13.2 Mitigation Strategy

| Component | Mitigation |
|-----------|------------|
| MCU | Triple voting, watchdog |
| Memory | EDAC, scrubbing |
| Analog | Redundancy, filtering |
| Power | Current limiting, OVP |

---

## 14. Approved Alternates

### 14.1 Alternate Parts

| Primary | Alternate | Notes |
|---------|-----------|-------|
| STM32L4R5 | STM32L4S9 | Pin compatible |
| TPS62840 | TPS62842 | Same family |
| LIS3MDL | LSM303AGR | Different pinout |
| CC1125 | CC1120 | Lower data rate |

---

## 15. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
