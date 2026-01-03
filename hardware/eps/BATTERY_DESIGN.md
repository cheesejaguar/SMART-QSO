# Battery Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-BAT-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the battery pack design for the SMART-QSO 1U CubeSat, following NASA Ames SmallSat heritage and NASA-STD-8719.14 safety requirements.

---

## 2. Cell Selection

### 2.1 Cell Specification

| Parameter | Value |
|-----------|-------|
| Manufacturer | Molicel / Samsung / Panasonic |
| Chemistry | Lithium-ion (NMC/NCA) |
| Form Factor | 18650 cylindrical |
| Nominal Voltage | 3.6V |
| Capacity | 3000-3500 mAh |
| Energy Density | > 240 Wh/kg |
| Cycle Life | > 500 cycles to 80% |
| Operating Temp | -20°C to +60°C charge, -40°C to +70°C discharge |

### 2.2 Selected Cell

| Parameter | Value |
|-----------|-------|
| Model | Molicel INR18650-P28A or equivalent |
| Nominal Capacity | 2800 mAh |
| Max Continuous Discharge | 35A |
| Internal Resistance | < 20 mΩ |
| Mass | 48g per cell |

---

## 3. Pack Configuration

### 3.1 Configuration

| Parameter | Value |
|-----------|-------|
| Configuration | 2S1P (2 series, 1 parallel) |
| Nominal Voltage | 7.2V |
| Voltage Range | 6.0V (empty) to 8.4V (full) |
| Capacity | 2800 mAh (20.2 Wh) |
| Total Mass | ~120g (cells + BMS + packaging) |

### 3.2 Energy Budget

| Mode | Power | Duration | Energy |
|------|-------|----------|--------|
| Safe Mode | 0.5W | 35 min eclipse | 0.29 Wh |
| Active Mode | 3.0W | 35 min eclipse | 1.75 Wh |
| Burst Mode | 8.0W | 5 min burst | 0.67 Wh |
| **Eclipse Margin** | - | - | **>5x** |

---

## 4. Battery Management System (BMS)

### 4.1 BMS Features

| Feature | Specification |
|---------|---------------|
| Cell Balancing | Passive, resistor-based |
| Balance Current | 50-100 mA |
| Balance Threshold | 50 mV |
| Overcharge Protection | 4.25V per cell |
| Overdischarge Protection | 2.8V per cell |
| Overcurrent Protection | 10A |
| Temperature Protection | -10°C charge inhibit, +65°C cutoff |

### 4.2 BMS IC Selection

- Primary: BQ76920 or equivalent
- Features: I2C telemetry, programmable thresholds
- Backup: Hardware comparator cutoffs

### 4.3 Charge Control

| Mode | Current | Condition |
|------|---------|-----------|
| Precharge | 100 mA | Vcell < 3.0V |
| CC Charge | 500-1000 mA | Vcell < 4.2V |
| CV Charge | Tapering | Vcell = 4.2V |
| Termination | 50 mA | End of charge |

---

## 5. Safety Features

### 5.1 Hardware Protections

| Protection | Implementation |
|------------|----------------|
| Fuse | Resettable PTC + backup fuse |
| Overcharge | BMS + hardware comparator |
| Overdischarge | BMS + hardware comparator |
| Overcurrent | PTC + electronic switch |
| Short Circuit | Electronic switch, < 10 µs response |
| Temperature | NTC sensors, charge inhibit |

### 5.2 Launch Safety (Charge Inhibit)

Per NASA-STD-8719.14 and launch provider requirements:

| Inhibit | Implementation |
|---------|----------------|
| Remove Before Flight (RBF) | Mechanical pin disconnects charger |
| Deployment Switch | Inhibits charging until deployed |
| Software Inhibit | Timer-based charge enable |

### 5.3 Passivation

End-of-life battery passivation per NASA-STD-8719.14:
- Controlled discharge to 50% SOC
- Open charge path
- Heater activation to accelerate discharge (optional)

---

## 6. Thermal Design

### 6.1 Temperature Limits

| Condition | Limit |
|-----------|-------|
| Charge Enable | > -10°C |
| Optimal Charge | +10°C to +30°C |
| Discharge Enable | > -40°C |
| High Temp Cutoff | +65°C |

### 6.2 Thermal Interface

- Thermal pad to spacecraft structure
- Heat spreading via aluminum cell holders
- Optional heater circuit for cold start

### 6.3 Heater Sizing

| Parameter | Value |
|-----------|-------|
| Heater Power | 0.5W |
| Activation Temp | < -10°C |
| Thermostat Type | NTC-based control |

---

## 7. Mechanical Packaging

### 7.1 Cell Holder Design

| Parameter | Specification |
|-----------|---------------|
| Material | Aluminum 6061-T6 |
| Cell Retention | Spring-loaded contacts |
| Mounting | M2 fasteners to structure |
| Vibration Survival | 14 grms random |

### 7.2 Wiring

- 20 AWG for power paths
- Fused at cell level
- Strain relief at all connections
- Conformal coating on BMS PCB

---

## 8. Telemetry

### 8.1 Telemetry Points

| Parameter | Sensor | Resolution |
|-----------|--------|------------|
| Pack Voltage | ADC | 10 mV |
| Cell 1 Voltage | BMS | 10 mV |
| Cell 2 Voltage | BMS | 10 mV |
| Pack Current | Hall sensor | 10 mA |
| Temperature 1 | NTC | 0.5°C |
| Temperature 2 | NTC | 0.5°C |
| SOC | Calculated | 1% |
| Charge Status | BMS | Boolean |

---

## 9. Testing Requirements

### 9.1 Cell Acceptance

| Test | Requirement |
|------|-------------|
| Capacity | > 95% rated |
| IR | < 25 mΩ |
| OCV | 3.6V ± 0.1V |

### 9.2 Pack-Level Testing

| Test | Specification |
|------|---------------|
| Capacity Verification | C/5 discharge |
| Charge/Discharge Cycling | 10 cycles |
| Protection Trip Test | All protection circuits |
| Thermal Cycling | -20°C to +50°C, 5 cycles |
| Vibration | Protoflight levels |

---

## 10. Compliance

### 10.1 Standards

| Standard | Compliance |
|----------|------------|
| UN 38.3 | Transport certification |
| NASA-STD-8719.14 | Orbital debris/passivation |
| Launch Provider | Battery safety requirements |
| IEC 62133 | Safety requirements |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
