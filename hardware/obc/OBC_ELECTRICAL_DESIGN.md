# OBC Electrical Design Specification
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-OBC-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document specifies the electrical design requirements for the SMART-QSO OBC (On-Board Computer) PCB, including schematic requirements, component selection, PCB layout guidelines, and manufacturing specifications.

---

## 2. Schematic Requirements

### 2.1 Microcontroller Section

| Block | Component | Notes |
|-------|-----------|-------|
| MCU | STM32L476RGT6 | 80 MHz, 1MB Flash, 128KB SRAM |
| Crystal (HSE) | 8 MHz | ±10 ppm |
| Crystal (LSE) | 32.768 kHz | ±20 ppm |
| Reset circuit | RC + diode | External reset |
| Decoupling | 100nF per VDD | Close to pins |

### 2.2 Memory Section

| Block | Component | Notes |
|-------|-----------|-------|
| FRAM | FM25V20A | 256KB, SPI |
| SD Card | microSD slot | SPI mode |
| Program flash | Internal | 1MB in MCU |

### 2.3 Interface Section

| Interface | Component | Notes |
|-----------|-----------|-------|
| UART ×3 | MCU internal | Debug, Jetson, EPS |
| I2C ×2 | MCU internal | Sensors, EPS |
| SPI ×2 | MCU internal | FRAM, SD card |
| GPIO | MCU + expander | Deployment, status |
| Debug | SWD header | ST-Link |

### 2.4 Power Section

| Block | Component | Notes |
|-------|-----------|-------|
| 3.3V input | Connector | From EPS |
| 5V input | Connector | From EPS |
| LDO (optional) | TLV70233 | If local regulation needed |
| Power monitoring | INA219 | 3.3V rail |

### 2.5 Watchdog Section

| Block | Component | Notes |
|-------|-----------|-------|
| Internal WDT | MCU IWDG | Primary |
| External WDT | TPL5010 | Backup |
| Heartbeat | GPIO toggle | Kick input |

---

## 3. Bill of Materials (BOM) Guidelines

### 3.1 Component Selection Criteria

| Criterion | Requirement |
|-----------|-------------|
| Temperature range | -40°C to +85°C minimum |
| Package | No BGA preferred |
| Availability | Dual-source |
| Derating | Per EEE-INST-002 |
| Flight heritage | COTS+ acceptable |

### 3.2 Key Components

| Component | Primary | Alternate | Qty |
|-----------|---------|-----------|-----|
| MCU | STM32L476RGT6 | STM32L4R5ZIT6 | 1 |
| FRAM | FM25V20A | MR25H256 | 1 |
| SD card slot | Molex 473521001 | Hirose DM3D | 1 |
| Ext. watchdog | TPL5010 | MAX6369 | 1 |
| I2C I/O expander | PCA9555 | MCP23017 | 1 |
| Current monitor | INA219 | INA226 | 1 |
| 8 MHz crystal | ABM8G | FA-238 | 1 |
| 32.768 kHz crystal | ABS07 | FC-135 | 1 |

### 3.3 Passive Components

| Component | Value | Package | Spec |
|-----------|-------|---------|------|
| Decoupling cap | 100nF | 0402 | X7R, 16V |
| Bulk cap | 10µF | 0603 | X5R, 10V |
| Crystal load cap | 12pF | 0402 | C0G |
| Pull-up resistor | 10kΩ | 0402 | 1% |
| ESD protection | TVS | SOT-23 | USB ESD |

---

## 4. PCB Layout Guidelines

### 4.1 Board Specifications

| Parameter | Value |
|-----------|-------|
| Dimensions | 90mm × 90mm (max) |
| Layers | 4-6 (signal-GND-PWR-GND-signal) |
| Thickness | 1.6mm |
| Material | FR4, Tg > 170°C |
| Copper weight | 1 oz (35µm) |
| Surface finish | ENIG |
| Solder mask | Green, matte |

### 4.2 Design Rules

| Parameter | Minimum | Preferred |
|-----------|---------|-----------|
| Trace width (signal) | 0.1mm | 0.15mm |
| Trace width (power) | 0.3mm | 0.5mm |
| Trace spacing | 0.1mm | 0.15mm |
| Via diameter | 0.3mm | 0.4mm |
| Via annular ring | 0.15mm | 0.2mm |
| Component clearance | 0.25mm | 0.4mm |

### 4.3 Stack-up (6-layer)

```
Layer 1 (Top):     35µm Cu - Components, signals
Prepreg:           0.1mm FR4
Layer 2 (GND):     35µm Cu - Ground plane
Core:              0.3mm FR4
Layer 3 (SIG):     35µm Cu - Inner signals
Prepreg:           0.2mm FR4
Layer 4 (PWR):     35µm Cu - Power distribution
Core:              0.3mm FR4
Layer 5 (GND):     35µm Cu - Ground plane
Prepreg:           0.1mm FR4
Layer 6 (Bot):     35µm Cu - Components, signals
                   ──────────
Total:             ~1.6mm
```

### 4.4 Impedance Control

| Signal Type | Impedance | Trace Width |
|-------------|-----------|-------------|
| Single-ended | 50Ω | 0.15mm |
| Differential | 100Ω | 0.1mm/0.1mm gap |
| SPI/I2C | Uncontrolled | Standard |

### 4.5 MCU Layout Guidelines

| Guideline | Implementation |
|-----------|----------------|
| Power pins | 100nF within 2mm of each pin |
| Crystal (HSE) | Ground guard, short traces |
| Crystal (LSE) | Keep away from digital |
| Reset | RC filter close to pin |
| SWD | Keep accessible, ESD protection |

---

## 5. Connector Selection

### 5.1 Interface Connectors

| Connector | Type | Purpose |
|-----------|------|---------|
| J1 | Molex Micro-Fit 8-pin | Power from EPS |
| J2 | JST-SH 4-pin | I2C to sensors |
| J3 | JST-SH 3-pin | UART to Jetson |
| J4 | JST-SH 3-pin | UART to EPS |
| J5 | 1.27mm header | SWD debug |
| J6 | microSD slot | Storage |

### 5.2 Test Connectors

| Connector | Type | Purpose |
|-----------|------|---------|
| J10 | 2.54mm header | Development debug |
| TP1-TP8 | Test points | Probing |

---

## 6. EMC Guidelines

### 6.1 Grounding

| Guideline | Implementation |
|-----------|----------------|
| Ground plane | Solid on Layer 2 and 5 |
| Plane stitching | Vias every 3mm |
| Connector ground | Multiple ground pins |
| Crystal ground | Local ground island |

### 6.2 Filtering

| Interface | Filter |
|-----------|--------|
| Power input | 10µF + 100nF |
| I2C lines | 100Ω series + 100pF |
| UART lines | ESD only |
| SWD | ESD + filter |

### 6.3 EMI Reduction

| Technique | Implementation |
|-----------|----------------|
| Clock frequency | 80 MHz (spread spectrum optional) |
| Rise time control | Series resistors on clocks |
| Loop area | Minimize signal loops |
| Via fencing | Around board edge |

---

## 7. Fabrication Notes

### 7.1 Gerber Files Required

| Layer | Filename |
|-------|----------|
| Top copper | OBC_top.gtl |
| Top solder mask | OBC_tsm.gts |
| Top silkscreen | OBC_tss.gto |
| Inner layers | OBC_inX.gX |
| Bottom copper | OBC_bot.gbl |
| Bottom solder mask | OBC_bsm.gbs |
| Bottom silkscreen | OBC_bss.gbo |
| Drill | OBC_drill.drl |
| Outline | OBC_outline.gko |

### 7.2 Manufacturing Requirements

| Parameter | Specification |
|-----------|---------------|
| IPC Class | Class 2 |
| Impedance control | Required for diff pairs |
| Via fill | Not required |
| Testing | Flying probe |
| Panelization | 2×2 if applicable |

---

## 8. Test Points

| Test Point | Signal | Purpose |
|------------|--------|---------|
| TP1 | VDD_3V3 | Power rail |
| TP2 | GND | Ground reference |
| TP3 | NRST | Reset |
| TP4 | UART_TX | Debug UART |
| TP5 | UART_RX | Debug UART |
| TP6 | I2C_SDA | I2C debug |
| TP7 | I2C_SCL | I2C debug |
| TP8 | FRAM_CS | SPI debug |

---

## 9. Deliverables Checklist

| Deliverable | Format | Status |
|-------------|--------|--------|
| Schematic PDF | PDF | Required |
| Schematic source | KiCad/OrCAD | Required |
| BOM | Excel/CSV | Required |
| PCB layout | Gerber | Required |
| Assembly drawing | PDF | Required |
| 3D model | STEP | Recommended |
| DRC report | PDF | Required |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
