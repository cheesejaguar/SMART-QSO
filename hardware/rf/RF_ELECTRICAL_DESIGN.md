# RF Electrical Design Specification
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document specifies the electrical design requirements for the SMART-QSO RF subsystem PCB, including schematic requirements, component selection, PCB layout guidelines for RF, and manufacturing specifications.

---

## 2. RF Specifications

### 2.1 Transmitter Specifications

| Parameter | Value | Notes |
|-----------|-------|-------|
| Frequency | 145.825 MHz | 2m amateur band |
| Modulation | AFSK | 1200 baud |
| Output power | 0.5W / 2W | Selectable |
| Spurious | < -50 dBc | All spurious |
| Harmonics | 2nd < -30 dBc, 3rd < -40 dBc | |
| Frequency stability | ±2 kHz | Over temperature |

### 2.2 Receiver Specifications (If Applicable)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Frequency | 435-438 MHz | 70cm amateur band |
| Sensitivity | -115 dBm | 12 dB SINAD |
| Selectivity | -60 dB @ ±25 kHz | Adjacent channel |

---

## 3. Schematic Requirements

### 3.1 Frequency Synthesizer

| Block | Component | Notes |
|-------|-----------|-------|
| Reference oscillator | TCXO 16.368 MHz | ±1 ppm |
| PLL synthesizer | ADF4351 or equiv | Fractional-N |
| Loop filter | 2nd order | Per calculation |
| VCO | Internal to IC | |

### 3.2 Modulator

| Block | Component | Notes |
|-------|-----------|-------|
| Audio input | Op-amp buffer | From MCU DAC |
| Deviation control | Resistor divider | ±3 kHz deviation |
| FM modulator | VCO modulation input | |
| Pre-emphasis | RC network | Optional |

### 3.3 Power Amplifier

| Block | Component | Notes |
|-------|-----------|-------|
| Driver stage | RF transistor | +10 dBm |
| Final stage | RF transistor | +33 dBm (2W) |
| Bias network | Temperature compensated | |
| Matching network | L-match | 50Ω |

### 3.4 Output Filter

| Block | Component | Notes |
|-------|-----------|-------|
| Low-pass filter | 7-element Chebyshev | Fc = 175 MHz |
| Harmonic attenuation | > 40 dB @ 2nd harmonic | |
| Insertion loss | < 0.5 dB | |
| Return loss | > 15 dB | |

### 3.5 Antenna Switch (If RX)

| Block | Component | Notes |
|-------|-----------|-------|
| TX/RX switch | RF switch IC | SP2T |
| Isolation | > 30 dB | |
| Insertion loss | < 0.5 dB | |

### 3.6 Control Interface

| Block | Component | Notes |
|-------|-----------|-------|
| SPI interface | Level shifter | To OBC |
| Power control | Load switch | TX power gate |
| Temperature sensor | NTC thermistor | PA monitoring |
| Power detector | AD8318 | Forward power |

---

## 4. Bill of Materials (BOM) Guidelines

### 4.1 Component Selection Criteria

| Criterion | Requirement |
|-----------|-------------|
| Temperature range | -40°C to +85°C |
| RF-grade components | Required for RF path |
| Package | Low inductance |
| Q factor | High Q inductors |
| Tolerance | 1-2% for matching |

### 4.2 Key Components

| Component | Primary | Alternate | Qty |
|-----------|---------|-----------|-----|
| Synthesizer | ADF4351 | Si5351 | 1 |
| TCXO | ASTX-13 | ASVTX-13 | 1 |
| PA transistor | RD01MUS1 | MRF101AN | 2 |
| RF switch | PE42520 | SKY13286 | 1 |
| LNA | BGA725 | SPF5189Z | 1 |
| Power detector | AD8318 | LTC5507 | 1 |

### 4.3 RF Passives

| Component | Value | Package | Notes |
|-----------|-------|---------|-------|
| RF capacitor | Various | 0402 | NP0/C0G |
| RF inductor | Various | 0402 | Wire-wound |
| Chip inductor | Various | 0603 | Air-core |
| Matching resistor | Various | 0402 | Thin film |

---

## 5. PCB Layout Guidelines

### 5.1 Board Specifications

| Parameter | Value |
|-----------|-------|
| Dimensions | 70mm × 50mm (max) |
| Layers | 4 (signal-GND-GND-signal) |
| Thickness | 1.0mm |
| Material | Rogers RO4003C or FR4 |
| Dielectric constant | 3.55 (Rogers) |
| Copper weight | 1 oz (35µm) |
| Surface finish | ENIG |

### 5.2 RF Design Rules

| Parameter | Value | Notes |
|-----------|-------|-------|
| 50Ω microstrip (Rogers) | 0.55mm | On 1.0mm substrate |
| 50Ω microstrip (FR4) | 0.8mm | On 1.0mm substrate |
| Ground via spacing | < λ/20 | Along RF traces |
| Component spacing | > 2mm | Between RF stages |
| Shielding gap | 0.5mm | For RF cans |

### 5.3 Stack-up

```
Layer 1 (Top):     35µm Cu - RF, components
Prepreg:           0.2mm (Rogers or FR4)
Layer 2 (GND):     35µm Cu - Solid ground
Core:              0.4mm
Layer 3 (GND):     35µm Cu - Solid ground
Prepreg:           0.2mm
Layer 4 (Bot):     35µm Cu - Control, power
                   ──────────
Total:             ~1.0mm
```

### 5.4 RF Layout Guidelines

| Guideline | Implementation |
|-----------|----------------|
| Ground continuity | No splits under RF |
| Via fence | Every 2mm along RF |
| Component orientation | Minimize loop area |
| Power amplifier | Thermal vias to ground |
| Synthesizer | Isolate from PA |
| Crystal | Ground guard ring |

### 5.5 Grounding

| Zone | Requirement |
|------|-------------|
| PA section | Solid ground, thermal vias |
| Synthesizer | Isolated but connected |
| Digital | Separate from RF |
| Star ground | At main connector |

---

## 6. Connector Selection

### 6.1 RF Connectors

| Connector | Type | Purpose |
|-----------|------|---------|
| J1 | SMA female | Antenna output |
| J2 | U.FL | Internal test |

### 6.2 Interface Connectors

| Connector | Type | Purpose |
|-----------|------|---------|
| J3 | Molex Pico-Clasp 4-pin | Power input |
| J4 | JST-SH 4-pin | SPI control |

---

## 7. Shielding

### 7.1 Shielding Requirements

| Zone | Shielding |
|------|-----------|
| PA section | RF can, mandatory |
| Synthesizer | Optional |
| LNA (if present) | RF can, recommended |

### 7.2 Shield Can Specifications

| Parameter | Value |
|-----------|-------|
| Material | Tin-plated steel or mu-metal |
| Height | 5mm |
| Attachment | Solder to ground pads |
| Ventilation | Not required (vacuum) |

---

## 8. EMC Considerations

### 8.1 Harmonics Suppression

| Harmonic | Attenuation |
|----------|-------------|
| 2nd (291 MHz) | > 30 dB |
| 3rd (437 MHz) | > 40 dB |
| 4th (583 MHz) | > 50 dB |

### 8.2 Spurious Control

| Source | Mitigation |
|--------|------------|
| PLL reference | Shielded trace |
| PLL loop | Low noise design |
| PA intermod | Class C operation |
| Digital noise | Filtering |

### 8.3 EMI from Digital

| Mitigation | Implementation |
|------------|----------------|
| Digital power | LC filter |
| SPI lines | Series resistors, ferrite |
| Ground | Separate digital/RF |

---

## 9. Thermal Design

### 9.1 Power Dissipation

| Component | Max Power | Notes |
|-----------|-----------|-------|
| PA (2W mode) | 4W | 50% efficiency |
| Synthesizer | 0.3W | |
| LNA | 0.1W | |
| Total | 4.5W | |

### 9.2 Thermal Management

| Component | Thermal Design |
|-----------|----------------|
| PA transistor | Large copper pour + vias |
| Board mounting | Thermally coupled to chassis |
| Temperature sensor | Near PA |

---

## 10. Test Points

| Test Point | Signal | Purpose |
|------------|--------|---------|
| TP1 | VCC_PA | PA power |
| TP2 | GND | RF ground |
| TP3 | VCO_OUT | Synthesizer output |
| TP4 | PA_OUT | Pre-filter |
| TP5 | TEMP | Temperature monitor |
| J2 | ANT_OUT | U.FL for test |

---

## 11. Fabrication Notes

### 11.1 Special Requirements

| Requirement | Specification |
|-------------|---------------|
| Material | Rogers RO4003C preferred |
| Impedance | 50Ω ±5% |
| Via plating | Good plating for RF vias |
| Surface finish | ENIG (low loss) |

### 11.2 Gerber Files

| Layer | Filename |
|-------|----------|
| Top copper | RF_top.gtl |
| Inner ground 1 | RF_gnd1.g2 |
| Inner ground 2 | RF_gnd2.g3 |
| Bottom copper | RF_bot.gbl |
| Solder masks | RF_tsm.gts, RF_bsm.gbs |
| Silkscreen | RF_tss.gto, RF_bss.gbo |
| Drill | RF_drill.drl |
| Outline | RF_outline.gko |

---

## 12. Deliverables Checklist

| Deliverable | Format | Status |
|-------------|--------|--------|
| Schematic PDF | PDF | Required |
| Schematic source | KiCad/OrCAD | Required |
| BOM | Excel/CSV | Required |
| PCB layout | Gerber | Required |
| Impedance report | PDF | Required |
| Assembly drawing | PDF | Required |
| 3D model | STEP | Recommended |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
