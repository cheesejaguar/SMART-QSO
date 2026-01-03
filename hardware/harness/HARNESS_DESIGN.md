# Harness and Interconnect Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-HAR-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the electrical harness and interconnect design for the SMART-QSO 1U CubeSat per NASA-STD-8739.4 and IPC/WHMA-A-620 Class 3 (Space).

---

## 2. Harness Block Diagram

```
                    ┌─────────────┐
                    │   Battery   │
                    │    Pack     │
                    └──────┬──────┘
                           │ H1 (Power)
                    ┌──────▼──────┐
                    │     EPS     │
                    │     PDU     │
                    └──────┬──────┘
           ┌───────────────┼───────────────┐
           │               │               │
    H2 (Power)      H3 (Power)      H4 (Power)
           │               │               │
    ┌──────▼──────┐ ┌──────▼──────┐ ┌──────▼──────┐
    │     OBC     │ │   Jetson    │ │     RF      │
    │             │ │   Module    │ │   Board     │
    └──────┬──────┘ └──────┬──────┘ └──────┬──────┘
           │               │               │
    H5 (I2C)        H6 (UART)       H7 (RF)
           │               │               │
    ┌──────▼──────┐        │        ┌──────▼──────┐
    │    ADCS     │        │        │   Antenna   │
    │   Sensors   │        │        │             │
    └─────────────┘        │        └─────────────┘
                           │
                    ┌──────▼──────┐
                    │   Solar     │
                    │   Arrays    │
                    └─────────────┘
```

---

## 3. Connector Selection

### 3.1 Connector Types

| Application | Connector Family | Mating Cycles |
|-------------|------------------|---------------|
| Power (high current) | Molex Micro-Fit 3.0 | 25 |
| Power (low current) | Molex Pico-Clasp | 25 |
| Signal (board-board) | Samtec FTSH | 100 |
| Signal (debug) | 2.54mm header | 50 |
| RF | SMA/U.FL | 500/30 |

### 3.2 Connector Selection Criteria

- Flight-proven heritage
- Operating temperature: -55°C to +125°C
- Low outgassing (TML < 1%, CVCM < 0.1%)
- Adequate current rating with 50% derating
- Positive locking mechanism

---

## 4. Harness Definitions

### 4.1 H1 - Battery to EPS

| Pin | Signal | Wire | Color |
|-----|--------|------|-------|
| 1 | VBAT+ | 20 AWG | Red |
| 2 | VBAT+ | 20 AWG | Red |
| 3 | VBAT- | 20 AWG | Black |
| 4 | VBAT- | 20 AWG | Black |
| 5 | TEMP1 | 26 AWG | White |
| 6 | TEMP2 | 26 AWG | Yellow |

| Parameter | Value |
|-----------|-------|
| Connector | Molex Micro-Fit 6-pin |
| Length | 80 mm ± 5 mm |
| Current | 3A max |

### 4.2 H2 - EPS to OBC

| Pin | Signal | Wire | Color |
|-----|--------|------|-------|
| 1 | +3.3V | 22 AWG | Red |
| 2 | +5V | 22 AWG | Orange |
| 3 | GND | 22 AWG | Black |
| 4 | GND | 22 AWG | Black |
| 5 | I2C_SDA | 26 AWG | Blue |
| 6 | I2C_SCL | 26 AWG | Yellow |
| 7 | OBC_EN | 26 AWG | Green |
| 8 | EPS_INT | 26 AWG | White |

| Parameter | Value |
|-----------|-------|
| Connector | Molex Micro-Fit 8-pin |
| Length | 60 mm ± 5 mm |
| Current | 1A max per power pin |

### 4.3 H3 - EPS to Jetson

| Pin | Signal | Wire | Color |
|-----|--------|------|-------|
| 1 | +5V | 20 AWG | Red |
| 2 | +5V | 20 AWG | Red |
| 3 | GND | 20 AWG | Black |
| 4 | GND | 20 AWG | Black |

| Parameter | Value |
|-----------|-------|
| Connector | Molex Micro-Fit 4-pin |
| Length | 100 mm ± 5 mm |
| Current | 3A max |

### 4.4 H4 - EPS to RF

| Pin | Signal | Wire | Color |
|-----|--------|------|-------|
| 1 | +5V | 22 AWG | Red |
| 2 | GND | 22 AWG | Black |
| 3 | RF_EN | 26 AWG | Green |
| 4 | RF_TEMP | 26 AWG | White |

| Parameter | Value |
|-----------|-------|
| Connector | Molex Pico-Clasp 4-pin |
| Length | 70 mm ± 5 mm |
| Current | 1A max |

### 4.5 H5 - OBC to ADCS (I2C)

| Pin | Signal | Wire | Color |
|-----|--------|------|-------|
| 1 | I2C_SDA | 26 AWG | Blue |
| 2 | I2C_SCL | 26 AWG | Yellow |
| 3 | GND | 26 AWG | Black |
| 4 | +3.3V | 26 AWG | Red |

| Parameter | Value |
|-----------|-------|
| Connector | JST-SH 4-pin |
| Length | 50 mm ± 5 mm |
| Shielding | Twisted pair |

### 4.6 H6 - OBC to Jetson (UART)

| Pin | Signal | Wire | Color |
|-----|--------|------|-------|
| 1 | UART_TX | 26 AWG | Green |
| 2 | UART_RX | 26 AWG | Yellow |
| 3 | GND | 26 AWG | Black |

| Parameter | Value |
|-----------|-------|
| Connector | JST-SH 3-pin |
| Length | 80 mm ± 5 mm |

### 4.7 H7 - RF to Antenna

| Parameter | Value |
|-----------|-------|
| Cable | RG-316 coaxial |
| Impedance | 50Ω |
| Length | 120 mm ± 10 mm |
| Connector (board) | SMA female |
| Connector (antenna) | Solder/crimp |
| Loss | < 0.5 dB |

### 4.8 H8 - Solar Array Harness

| Pin | Signal | Wire | Color |
|-----|--------|------|-------|
| 1 | SA+ | 24 AWG | Red |
| 2 | SA- | 24 AWG | Black |
| 3 | TEMP | 28 AWG | White |

| Parameter | Value |
|-----------|-------|
| Connector | Molex Pico-Clasp 3-pin |
| Length | Per panel location |
| Panels | 4× body + 2× deployable |

---

## 5. Wire Specifications

### 5.1 Wire Types

| AWG | Insulation | Temp Range | Application |
|-----|------------|------------|-------------|
| 20 | PTFE | -65 to +200°C | High current power |
| 22 | PTFE | -65 to +200°C | Power distribution |
| 24 | PTFE | -65 to +200°C | Solar array |
| 26 | PTFE | -65 to +200°C | Signal, low current |
| 28 | PTFE | -65 to +200°C | Sensor signals |

### 5.2 Wire Selection

| Current (A) | Minimum AWG | With 50% Derating |
|-------------|-------------|-------------------|
| 0.5 | 28 | 26 |
| 1.0 | 26 | 24 |
| 2.0 | 24 | 22 |
| 3.0 | 22 | 20 |
| 5.0 | 20 | 18 |

---

## 6. Wire Routing

### 6.1 Routing Guidelines

| Guideline | Requirement |
|-----------|-------------|
| Bend radius | ≥ 5× wire diameter |
| Clearance to structure | ≥ 1 mm |
| Tie-down spacing | ≤ 50 mm |
| Service loop | 10% extra length |
| Crossing power/signal | 90° angle |

### 6.2 Routing Zones

| Zone | Location | Contents |
|------|----------|----------|
| A | +X channel | Solar array harness |
| B | -X channel | Solar array harness |
| C | +Y channel | OBC-Jetson, debug |
| D | -Y channel | Power distribution |
| E | Center stack | Inter-board connections |

---

## 7. Shielding and Grounding

### 7.1 Shielding Requirements

| Harness | Shielding | Reason |
|---------|-----------|--------|
| H5 (I2C) | Twisted pair | EMI rejection |
| H6 (UART) | Optional twist | Low speed OK |
| H7 (RF) | Coaxial | RF containment |
| Power | None | Acceptable EMI |

### 7.2 Grounding Scheme

- Single-point ground at EPS
- Shield drain to connector shell
- No ground loops between boards
- Star ground topology

---

## 8. Strain Relief

### 8.1 Strain Relief Methods

| Location | Method |
|----------|--------|
| Board exit | Adhesive tie-down within 15mm |
| Connector | Backshell or potting |
| Flex points | Service loop |
| Hinge crossing | Strain relief bracket |

### 8.2 Tie-Down Materials

| Material | Application |
|----------|-------------|
| Lacing tape | Primary retention |
| Cable ties (Tefzel) | Secondary |
| RTV adhesive | Connector strain relief |
| P-clamps | Structure mounting |

---

## 9. Harness Mass Budget

| Harness | Mass (g) |
|---------|----------|
| H1 - Battery | 8 |
| H2 - OBC power | 5 |
| H3 - Jetson power | 10 |
| H4 - RF power | 3 |
| H5 - ADCS I2C | 2 |
| H6 - Jetson UART | 2 |
| H7 - RF coax | 8 |
| H8 - Solar arrays | 12 |
| Tie-downs, misc | 5 |
| **Total** | **55 g** |

---

## 10. Harness Drawings

### 10.1 Drawing Requirements

| Drawing | Content |
|---------|---------|
| Block diagram | System interconnect |
| Pin assignments | Each connector |
| Routing diagram | Physical path |
| Assembly drawing | Installation |
| Detail drawings | Each harness |

### 10.2 Drawing Notes

- All dimensions in millimeters
- Wire colors per table
- Connector P/N callouts
- Torque specifications for terminals
- Workmanship standard reference

---

## 11. Testing Requirements

### 11.1 Harness Tests

| Test | Requirement |
|------|-------------|
| Continuity | All conductors |
| Isolation | > 100 MΩ @ 100V |
| Hi-pot | 500V for 1 minute |
| Pull test | Per NASA-STD-8739.4 |

### 11.2 Installed Tests

| Test | Requirement |
|------|-------------|
| Functional | All signals verified |
| Impedance (RF) | 50Ω ± 5Ω |
| Voltage drop | < 100 mV at full load |

---

## 12. Workmanship Standards

### 12.1 Applicable Standards

| Standard | Application |
|----------|-------------|
| NASA-STD-8739.4 | Crimping, cables |
| IPC/WHMA-A-620 | Cable assemblies |
| J-STD-001 Space | Soldering |

### 12.2 Inspection Criteria

- Visual inspection per A-620 Class 3
- Crimp cross-section (sample)
- Solder joint quality
- Strain relief verification
- Continuity verification

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
