# EPS Electrical Design Specification
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document specifies the electrical design requirements for the SMART-QSO EPS (Electrical Power Subsystem) PCB, including schematic requirements, component selection, PCB layout guidelines, and manufacturing specifications.

---

## 2. Schematic Requirements

### 2.1 Power Input Section

| Block | Components | Notes |
|-------|------------|-------|
| Solar array input | Schottky diodes, TVS | 6 channels, OR'd |
| MPPT controller | BQ25570 or equiv | Per channel |
| Battery interface | Connector, fuse | 2S1P Li-ion |
| Charge controller | BQ25883 or equiv | 2-cell |

### 2.2 Power Distribution Section

| Block | Components | Notes |
|-------|------------|-------|
| 3.3V regulator | TPS62840 or equiv | High efficiency |
| 5V regulator | TPS62082 or equiv | For Jetson |
| Load switches | TPS22918 or equiv | Per load |
| Current sensing | INA219 or equiv | Per rail |

### 2.3 Control and Telemetry Section

| Block | Components | Notes |
|-------|------------|-------|
| I2C interface | Level shifter | 3.3V logic |
| GPIO expander | PCA9555 | Load switch control |
| Temperature sensors | TMP117 | Battery, board |
| Voltage dividers | Precision resistors | Analog monitoring |

### 2.4 Protection Section

| Block | Components | Notes |
|-------|------------|-------|
| Overcurrent protection | eFuse or PTC | Each output |
| Overvoltage protection | TVS diodes | All inputs/outputs |
| Reverse polarity | P-FET or diode | Solar inputs |
| EMI filtering | Common-mode chokes | I/O interfaces |

---

## 3. Bill of Materials (BOM) Guidelines

### 3.1 Component Selection Criteria

| Criterion | Requirement |
|-----------|-------------|
| Temperature range | -40°C to +85°C minimum |
| Radiation tolerance | TID > 10 krad (Si) preferred |
| Package | No BGA for ease of rework |
| Availability | Dual-source or COTS+ |
| Derating | Per EEE-INST-002 |

### 3.2 Key Components

| Component | Primary | Alternate | Qty |
|-----------|---------|-----------|-----|
| MPPT IC | BQ25570 | SPV1040 | 6 |
| Battery charger | BQ25883 | LTC4162 | 1 |
| 3.3V regulator | TPS62840 | LT3042 | 1 |
| 5V regulator | TPS62082 | LM2775 | 1 |
| Load switch | TPS22918 | SIP32431 | 6 |
| Current monitor | INA219 | INA226 | 4 |
| Temp sensor | TMP117 | ADT7422 | 2 |
| GPIO expander | PCA9555 | MCP23017 | 1 |
| Schottky diode | PMEG3010CEH | STPS2L40U | 12 |
| TVS diode | SMBJ6.0A | 1SMB5.0AT3G | 8 |

### 3.3 Passive Components

| Component | Value | Package | Spec |
|-----------|-------|---------|------|
| Input capacitor | 10µF/10V | 0805 | X7R |
| Output capacitor | 100µF/6.3V | 1206 | X5R |
| Inductor (MPPT) | 22µH | 1210 | Shielded |
| Inductor (buck) | 10µH | 1008 | Shielded |
| Sense resistor | 10mΩ | 0805 | 1%, 0.5W |

---

## 4. PCB Layout Guidelines

### 4.1 Board Specifications

| Parameter | Value |
|-----------|-------|
| Dimensions | 90mm × 90mm (max) |
| Layers | 4 (signal-GND-PWR-signal) |
| Thickness | 1.6mm |
| Material | FR4, Tg > 170°C |
| Copper weight | 1 oz (35µm) outer, 1 oz inner |
| Surface finish | ENIG |
| Solder mask | Green, matte |

### 4.2 Design Rules

| Parameter | Minimum | Preferred |
|-----------|---------|-----------|
| Trace width (signal) | 0.15mm | 0.2mm |
| Trace width (power) | 0.5mm | 1.0mm |
| Trace spacing | 0.15mm | 0.2mm |
| Via diameter | 0.3mm | 0.4mm |
| Via annular ring | 0.15mm | 0.2mm |
| Component clearance | 0.3mm | 0.5mm |

### 4.3 Power Plane Design

| Layer | Usage | Notes |
|-------|-------|-------|
| Layer 1 | Signal + components | High-frequency signals on top |
| Layer 2 | Ground plane | Solid, no splits |
| Layer 3 | Power distribution | 3.3V, 5V, VBAT islands |
| Layer 4 | Signal + components | Low-frequency signals |

### 4.4 Thermal Considerations

| Component | Thermal Design |
|-----------|----------------|
| MPPT ICs | Thermal vias to ground plane |
| Load switches | Copper pour + vias |
| Regulators | Exposed pad to ground |
| Power resistors | Copper pour cooling |

### 4.5 EMC Guidelines

| Guideline | Implementation |
|-----------|----------------|
| Ground plane | Continuous on Layer 2 |
| Decoupling | 100nF at each IC, close |
| High-frequency | Ground guard rings |
| Input filtering | LC filter at connectors |
| Shielding | Ground via fence on edges |

---

## 5. Fabrication Notes

### 5.1 Gerber Files Required

| Layer | Filename | Extension |
|-------|----------|-----------|
| Top copper | EPS_top | .gtl |
| Top solder mask | EPS_tsm | .gts |
| Top silkscreen | EPS_tss | .gto |
| Inner layer 2 | EPS_in2 | .g2 |
| Inner layer 3 | EPS_in3 | .g3 |
| Bottom copper | EPS_bot | .gbl |
| Bottom solder mask | EPS_bsm | .gbs |
| Bottom silkscreen | EPS_bss | .gbo |
| Drill file | EPS_drill | .drl |
| Board outline | EPS_outline | .gko |

### 5.2 Stack-up Definition

```
Layer 1 (Top):     35µm Cu
Prepreg:           0.2mm FR4
Layer 2 (GND):     35µm Cu
Core:              0.8mm FR4
Layer 3 (PWR):     35µm Cu
Prepreg:           0.2mm FR4
Layer 4 (Bot):     35µm Cu
                   ──────────
Total:             ~1.6mm
```

### 5.3 Manufacturing Requirements

| Parameter | Specification |
|-----------|---------------|
| IPC Class | Class 2 (high reliability) |
| Impedance control | 50Ω ±10% (if required) |
| Controlled depth drilling | Not required |
| Via fill | Plugged, optional |
| Testing | Flying probe or bed-of-nails |

---

## 6. Assembly Drawing Requirements

### 6.1 Drawing Content

| Element | Requirement |
|---------|-------------|
| Component placement | All components shown |
| Reference designators | All visible |
| Polarity markers | Diodes, capacitors, ICs |
| Pin 1 markers | All ICs |
| Mounting holes | Dimensioned |
| Connector orientation | Keying shown |

### 6.2 Assembly Notes

| Note | Content |
|------|---------|
| Workmanship | Per IPC-A-610 Class 2 |
| Soldering | Per J-STD-001 |
| ESD handling | Per ANSI/ESD S20.20 |
| Cleaning | Aqueous, no residue |
| Conformal coating | Per NASA-STD-8739.1 (optional) |

---

## 7. Thermal Analysis

### 7.1 Power Dissipation

| Component | Typical (mW) | Maximum (mW) |
|-----------|--------------|--------------|
| MPPT ICs (×6) | 50 each | 100 each |
| Battery charger | 100 | 500 |
| 3.3V regulator | 50 | 150 |
| 5V regulator | 200 | 800 |
| Load switches | 10 each | 50 each |
| Total | 700 | 2500 |

### 7.2 Thermal Management

| Condition | Max Junction Temp | Margin |
|-----------|-------------------|--------|
| Regulators @ 50°C ambient | 85°C | > 40°C |
| MPPT @ 50°C ambient | 95°C | > 30°C |
| Load switches @ 50°C ambient | 75°C | > 50°C |

---

## 8. EMC Considerations

### 8.1 Conducted Emissions

| Limit | Frequency | Level |
|-------|-----------|-------|
| CE102 | 10 kHz - 10 MHz | Per MIL-STD-461G tailored |

### 8.2 Radiated Emissions

| Limit | Frequency | Level |
|-------|-----------|-------|
| RE102 | 2 MHz - 18 GHz | Per MIL-STD-461G tailored |

### 8.3 EMC Design Features

| Feature | Implementation |
|---------|----------------|
| Input filtering | LC filter, CM choke |
| Switching frequency | > 500 kHz (above AM band) |
| Layout | Tight loops, ground pour |
| Shielding | Optional RF shield can |

---

## 9. Test Points

### 9.1 Required Test Points

| Test Point | Signal | Purpose |
|------------|--------|---------|
| TP1 | VBAT | Battery voltage |
| TP2 | V3V3 | 3.3V rail |
| TP3 | V5V0 | 5V rail |
| TP4 | GND | Ground reference |
| TP5 | I2C_SDA | Debug |
| TP6 | I2C_SCL | Debug |
| TP7 | MPPT1_OUT | MPPT debug |
| TP8 | ICHG | Charge current |

---

## 10. Deliverables Checklist

| Deliverable | Format | Status |
|-------------|--------|--------|
| Schematic PDF | PDF | Required |
| Schematic source | OrCAD/KiCad/Altium | Required |
| BOM | Excel/CSV | Required |
| PCB layout | Gerber | Required |
| Assembly drawing | PDF | Required |
| 3D model | STEP | Recommended |
| Thermal analysis | Report | Required |
| DRC report | PDF | Required |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
