# RF Electrical Design Specification
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-003
**Version:** 1.1
**Date:** 2026-10-27
**Status:** Layout and fabrication guidelines for the RF board. Electrical requirements are in `docs/hardware/RF_DESIGN.md` v2.0 §2, which governs where this document differs (corrected in Version 1.1). The schematic and parts are not yet drawn or selected (RF_DESIGN §9).

---

## 1. Overview

This document specifies the electrical design requirements for the SMART-QSO RF subsystem PCB, including schematic requirements, component selection, PCB layout guidelines for RF, and manufacturing specifications.

---

## 2. RF Specifications

### 2.1 Transmitter Specifications

| Parameter | Value | Notes |
|-----------|-------|-------|
| Frequency | One IARU-coordinated frequency in 145.800–146.000 MHz (not 145.825 MHz) | RF-P001 |
| Modulation | FM: AFSK 1200 bps ±3.0 kHz; voice repeat ±5.0 kHz | RF-P004 |
| Output power | 0.5 W / 1 W (TX_PWR_SEL) | RF-P003; no 2 W level |
| Spurious | ≤ 25 µW and ≤ −46 dBc at 1 W | 47 CFR 97.307(e) |
| Harmonics | 2nd ≤ −46 dBc (−55 dBc design); **3rd ≤ −70 dBc** at the antenna port | RF-P008. Version 1.0's "2nd < −30 dBc, 3rd < −40 dBc" did not meet 97.307(e) |
| Frequency stability | ±1.0 ppm over temperature (±146 Hz) | RF-P005 |

### 2.2 Receiver Specifications (required: the receiver is always on)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Frequency | One IARU-coordinated frequency in 435.000–437.300 MHz | RF-P002 |
| Sensitivity | 12 dB SINAD at ≤ −122 dBm (C/N₀ 47.4 dB-Hz with T_sys 650 K) | RF-P006, RF-P011 |
| Selectivity | ≥ 60 dB at ±25 kHz | RF-P010 |
| RX filter | ≥ 60 dB at 144–146 MHz; ≥ 40 dB at 250–330 and 560–620 MHz; ≤ 1.5 dB in band | RF-P007 |
| Full-duplex desense | ≤ 0.5 dB with the transmitter keyed at 1 W | RF-P009 |

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
| Final stage | RF transistor | +30 dBm (1 W) maximum |
| Bias network | Temperature compensated | |
| Matching network | L-match | 50Ω |

### 3.4 Output Filter

| Block | Component | Notes |
|-------|-----------|-------|
| Low-pass filter | 7-element ladder of `TRANSMITTER_DESIGN.md` §7.3 | −3 dB at 164 MHz (analysed) |
| Harmonic attenuation | > 40 dB @ 2nd harmonic | |
| Insertion loss | < 0.5 dB | |
| Return loss | > 15 dB | |

### 3.5 Antenna Ports (no T/R switch)

The transponder is full duplex on separate antennas (BASELINE §7), so there is **no TX/RX switch**: the 2 m transmitter
feeds the 2 m element through the LPF, and the 70 cm element feeds the receiver through the RX filter. Isolation comes
from the antennas (ANT-5) and the RX filter (RF-P007). Keep the two RF paths on opposite sides of the board under
separate shield cans.

### 3.6 Control Interface

| Block | Component | Notes |
|-------|-----------|-------|
| UART to the TNC, discrete lines | 3.3 V CMOS | SQSO-ICD-003 §4.2 |
| Power control | Load switch | TX power gate |
| Temperature sensor | NTC thermistor | PA monitoring |
| Power detector | RMS detector | Forward and reflected power (TX_FWD, TX_REV; SQSO-ICD-003 §4.2) |

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
| Synthesizer | ADF4351-class fractional-N (check DC power against RF-P015) | Si5351 | 1 |
| TCXO | ASTX-13 | ASVTX-13 | 1 |
| PA transistor | RD01MUS1 | MRF101AN | 2 |
| RF switch | Not used (§3.5) | — | 0 |
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

| Harmonic | Requirement at the antenna port (1 W) |
|----------|---------------------------------------|
| 2nd (291.6–292.0 MHz) | ≤ −46 dBc (−55 dBc design) |
| 3rd (437.4–438.0 MHz) | ≤ −70 dBc (desense; SQSO-LBA-001 §7.3) |
| 4th (583.2–584.0 MHz) | ≤ −46 dBc |

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
| PA (1 W mode) | ≈ 1.7 W | ≈ 37 % efficiency |
| Synthesizer | 0.3W | |
| LNA | 0.1W | |
| Total | ≈ 2.1 W keyed at 1 W | |

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
| 1.1 | 2026-10-27 | CDR review (communications panel): transmitter and receiver specifications to RF_DESIGN v2.0 (no 2 W level, 97.307(e) spurious, 3rd harmonic ≤ −70 dBc, ±1.0 ppm, receiver sensitivity, selectivity and RX filter); no T/R switch (full duplex on separate antennas); thermal at 1 W; interface lines per SQSO-ICD-003 v2.0 |
