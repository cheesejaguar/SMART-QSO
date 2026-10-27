# RF Electrical Design Specification
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-003
**Version:** 1.2
**Date:** 2026-10-27
**Status:** Layout and fabrication guidelines for the RF board. Electrical requirements are in `docs/hardware/RF_DESIGN.md` v2.0 §2, which governs where this document differs (corrected in Version 1.1). Version 1.2: the schematic, parts and layout rules of record are `hardware/electrical/rf/RF_BOARD_DESIGN.md` (SQSO-RF-BRD-001), which supersedes §3–§10 of this document where they differ; the tables below are corrected to it.

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
| Reference oscillator | TCXO 40 MHz (Epson TG2016SMN) | ±0.5 ppm, −40…+85 °C, shared with the receiver |
| PLL synthesizer | TI LMX2571 | Fractional-N, FSK I2S mode for analog FM |
| Loop filter | 2nd order, 40 kHz bandwidth | 3.9 nF, 430 Ω + 39 nF |
| VCO | Internal to IC (4668.8 MHz / 32) | |

### 3.2 Modulator

| Block | Component | Notes |
|-------|-----------|-------|
| Audio input | 2:1 switch (TMUX1219): TNC AFSK or repeat audio | |
| Deviation control | PCM1808 ADC full scale = ±6.3 kHz (digital) | ±3.0 kHz beacon, ±4.7 kHz repeat limit |
| FM modulator | LMX2571 FSK deviation word over I2S | |
| Pre-emphasis | None added (the uplink's is repeated) | |

### 3.3 Power Amplifier

| Block | Component | Notes |
|-------|-----------|-------|
| Driver and final | Mitsubishi RD01MUS3 (2-stage) at 5.0 V | 0 dBm in, +30 dBm (1 W) maximum |
| Bias network | Closed-loop APC on the common gate bias | 27.0 / 30.0 dBm |
| Matching network | Two low-pass L sections, 7.5 → 19.4 → 50 Ω | Tuned by load-pull on the EM |

### 3.4 Output Filter

| Block | Component | Notes |
|-------|-----------|-------|
| Low-pass filter | 7th-order elliptic with traps at 2f and 3f (`TRANSMITTER_DESIGN.md` 1.4 §7.3) | −3 dB at 171 MHz |
| Harmonic attenuation | ≥ 65.5 dB at 2f, ≥ 88.2 dB at 3f (worst of 3,000) | |
| Insertion loss | 0.36 dB nominal, 0.60 dB worst | |
| Return loss | 18 dB nominal | |

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
| Receiver | TI CC1200 (receive only, analog-FM mode) | — | 1 |
| Synthesizer | TI LMX2571 | — | 1 |
| TCXO | Epson TG2016SMN 40 MHz | — | 1 |
| PA transistor | Mitsubishi RD01MUS3 (2-stage) | — | 1 |
| RF switch | Not used (§3.5) | — | 0 |
| LNA | Infineon BFP640F (6 mA) | — | 1 |
| Power detector | AD8361 | TI LMH2121 | 2 |

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
| Dimensions | PC/104 90.17 × 95.89 mm (stack board; 1.2: was 70 × 50 mm) |
| Layers | 6 (RF – GND – signal – power – GND – components) |
| Thickness | 1.6 mm |
| Material | High-Tg FR-4 (IS410 or 370HR class); RF losses at 146/436 MHz do not need Rogers |
| Dielectric constant | ≈ 4.2 |
| Copper weight | 1 oz (35µm) |
| Surface finish | ENIG |

### 5.2 RF Design Rules

| Parameter | Value | Notes |
|-----------|-------|-------|
| 50Ω microstrip (FR4) | 0.37 mm | L1 over L2, 0.20 mm prepreg (confirm with the fabricator's solver) |
| Ground via spacing | < λ/20 | Along RF traces |
| Component spacing | > 2mm | Between RF stages |
| Shielding gap | 0.5mm | For RF cans |

### 5.3 Stack-up

```
L1 (Top):   35 µm Cu - RF, components (three shield cavities)
Prepreg:    0.20 mm
L2:         35-70 µm Cu - solid ground
Core:       ~0.5 mm
L3:         35 µm Cu - signals
Prepreg:    ~0.2 mm
L4:         35 µm Cu - power planes by domain (+3V0_RX, +3V1_DIG, +5V_PA, +3V3_PA)
Core:       ~0.5 mm
L5:         35-70 µm Cu - solid ground
Prepreg:    0.20 mm
L6 (Bot):   35 µm Cu - digital components
            ─────────
Total:      1.6 mm
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
| J3, J4 | SMA female end launch, gold | 2 m balun outputs (+X, −X elements) |
| J5, J6 | SMA female end launch, gold | 70 cm balun inputs (+Y, −Y elements) |
| — | U.FL pads (DNP) | EM test at the LPF input and FL1 output |

### 6.2 Interface Connectors

| Connector | Type | Purpose |
|-----------|------|---------|
| J2 | Molex Pico-Clasp 6-pin | RADIO_3V3 and PA_5V (W4B) |
| J1 | Samtec FTSH-107 (14 pos) | KISS UART, TX_PWR_SEL, XPDR_EN, readbacks, monitors (W4A v2) |

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
| Material | Nickel-silver or copper alloy, not tin-plated (EEE_PARTS_LIST Rev C §2A.3 item 5; 1.2: tin-plated steel removed) |
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
| PA (1 W setting) | 0.86 W | 59 % PA efficiency, saturated |
| PA (0.5 W) | 0.87 W | 42.6 % |
| Synthesizer + audio ADC | 0.22 W | while keyed |
| Receiver (always) | 0.10–0.13 W | |
| Total | 1.26–1.38 W keyed + receiver | SQSO-RF-BRD-001 §11 |

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
| 1.2 | 2026-10-27 | CDR design completion, RF board | Tables corrected to the board design SQSO-RF-BRD-001: synthesizer, modulator, PA, filter, key components, board size (PC/104), 6-layer FR-4 stack-up, connectors, shield material (no tin-plated steel), dissipation |
