# EPS Electrical Design Specification
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-003
**Version:** 1.4
**Date:** 2026-10-27
**Status:** CDR Baseline — parts selected: the build-to schematic, BOM and layout constraints are
`hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001 D), which governs where this document differs;
`docs/hardware/EPS_DESIGN.md` v2.4 governs the architecture

> **CDR review (2026-10-27).** The funding-audit notice of 2026-10-16 (2S battery, 2-cell charger, six solar channels
> superseded) is acted on here: §2.1, §2.2 and §3.2 now carry the 4S configuration of `docs/hardware/EPS_DESIGN.md`
> v2.0 (eight strings on four trackers, a 4S CC/CV charger with power path at 4.10 V/cell, VBAT-rated switches, a
> secondary over-voltage protector). Every part is a candidate class, not a selection; the thermal (§7) and EMC (§8)
> sections are superseded by SQSO-THA-001 v2.0 and SQSO-EMC-001.

---

## 1. Overview

This document specifies the electrical design requirements for the SMART-QSO EPS (Electrical Power Subsystem) PCB, including schematic requirements, component selection, PCB layout guidelines, and manufacturing specifications.

---

## 2. Schematic Requirements

### 2.1 Power Input Section

| Block | Components | Notes |
|-------|------------|-------|
| Solar array input | Blocking diode (or ideal diode) per string, TVS per tracker with standoff ≥ 7 V | 8 strings, 3.5–6.4 V, ≤ 0.55 A each; string Voc reaches 6.31 V at −45 °C |
| MPPT / charge regulator | 4S buck-boost charger with MPPT, TS input and power path (BQ25798 class) or boost stages into one 4S CC/CV regulator | Four trackers (EPS_DESIGN §3.2); CC ≤ 0.40 A, CV 16.40 V ± 0.5 %; charge only 0 … +45 °C in hardware |
| Battery interface | Micro-Fit power, Pico-Clasp cell taps, pack fuse F2 5 A, PTC | 4S1P Li-ion |
| BMS | 4S AFE BQ76952 (OTP thresholds) + secondary OVP BQ771806 with a three-terminal fuse | SQSO-BSA-001 §3 |
| EOL disconnect | One-shot thermal cut-off with heater, two series fire switches | EPS_DESIGN §7 |

### 2.2 Power Distribution Section

| Block | Components | Notes |
|-------|------------|-------|
| 3V3_AO regulator | Synchronous buck, input ≥ 25 V | Always on |
| 5V_PA regulator | Synchronous buck, input ≥ 25 V | PA only |
| Jetson supply | eFuse (TPS25947 class) from VBAT; converter on the carrier | EMC rule E3 |
| Load switches | TPS22918 on 3.3/5 V; ≥ 25 V high-side switches on VBAT (TPS1H100 class) | EPS_DESIGN §5 |
| Low-voltage load shed | Comparator with hysteresis, 13.6/14.4 V, 5 s filter, gating SW1 and SW3 | EPS_DESIGN §6 |
| Current sensing | INA226 or equivalent | Battery, array per tracker, Jetson, PA |

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

| Component | Selected (SQSO-EPS-BRD-001 BOM) | Replaces (candidate class of 1.1) | Qty |
|-----------|---------------------------------|------------------------------------|-----|
| MPPT tracker | TI TPS55340QRTERQ1 boost, 1.000 MHz sync, hardware CV/CC/MPPT loops | BQ25798 (no sync input; four NVDC chargers cannot share a pack) | 4 |
| BMS AFE | TI BQ76952PFBR (OTP thresholds, SUV permanent fail, high-side FETs) | BQ76920 | 1 |
| Secondary OVP | TI BQ771806DPJR (4.350 V, 3 s) + Littelfuse ITV4030L1412NR three-terminal fuse | BQ7718 class | 1 + 1 |
| EOL disconnect | ITV4030L1412NR, two series fire FETs, arm latch SN74LVC2G74-Q1 | thermal cut-off | 1 |
| Recharge lockout | Panasonic TQ2SA-L2-3V-Z 2-coil latching relay, TPL5111 pulsers | — | 1 |
| 3V3_AO buck | TI LMR43620MC3RPERQ1 | 25 V synchronous buck | 1 |
| 5V_PA buck, 7.0 V burn buck | TI LMR43620MC5RPERQ1 | 25 V synchronous buck | 2 |
| Load switch, 3.3/5 V | TI TPS2553QDBVRQ1 (current limit, latch-up retry) | TPS22918 (no current limit) | 7 |
| Load switch, VBAT | TI TPS1H100BQPWPRQ1 (SW1, SW6), TPS1H000AQDGNRQ1 (SW5, SW11) | TPS1H100 class | 4 |
| eFuse (Jetson) | TI TPS16630RGER (60 V, latch-off) | TPS25947 class (23 V) | 1 |
| Current/power monitors | TI INA228AQDGSRQ1 (2), INA3221AQRGVRQ1 (4), INA186A2/A3 (3) | INA226 | 9 |
| ADC, temperatures | TI ADS7138QRTERQ1; 10 kΩ NTCs: two at the cells, six on the panels, one at the AFE FETs | TMP117 | 1 |
| EPS controller | TI MSP430FR2355TPTR (register block 0x48) | PCA9555 expander | 1 |
| I2C isolator, AntS buffer | TI ISO1540QDRQ1, TCA9517DGKRQ1 | level shifter | 1 + 1 |
| Blocking diode | TI LM74700QDDFRQ1 ideal-diode controller + Nexperia BUK7M12-40EX | PMEG3010CEH (≈ 6 % loss) | 8 |
| TVS, string input | Littelfuse SMBJ7.0A, one per string | 4 per tracker | 8 |
| Clock | Epson SG-8018CA 4 MHz + SN74LVC2G74-Q1 ÷ 4 | — | 1 |

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
| Dimensions | PC/104, 90.17 mm × 95.89 mm (SQSO-EPS-BRD-001 §17) |
| Layers | 6 (BRD-001 §17; the 4-layer stack of 1.1 is superseded) |
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
| 1.0a | 2026-10-16 | Funding audit notice: battery and array interfaces superseded by BASELINE §5 (PI-14) |
| 1.1 | 2026-10-27 | CDR review (EPS): §2.1, §2.2, §3.2 corrected to the 4S design of EPS_DESIGN v2.0 (trackers, 4S charger, secondary OVP, VBAT switches, load shed, EOL disconnect; string TVS 7.0 V); §7–§8 superseded by SQSO-THA-001 v2.0 and SQSO-EMC-001 |
| 1.2 | 2026-10-27 | CDR design (EPS board) | §3.2 parts selected (SQSO-EPS-BRD-001 A); board outline and layer count point to BRD-001 §17 |
| 1.3 | 2026-10-27 | CDR design (EPS board) | §3.2 VBAT load switches: SW11 magnetorquer supply added (SQSO-EPS-BRD-001 B, SQSO-ICX-001 E3) |
| 1.4 | 2026-10-27 | CDR design (EPS board) | §2 BMS row: BQ76952 and BQ771806 as selected (was "BQ76920 class") |
