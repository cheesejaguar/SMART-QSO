# Power Distribution Unit (PDU) Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-PDU-001
**Version:** 1.6
**Date:** 2026-10-27
**Status:** CDR Baseline — parts selected in the EPS board design `hardware/electrical/eps/EPS_BOARD_DESIGN.md`
(SQSO-EPS-BRD-001 G), which governs part-level values; `docs/hardware/EPS_DESIGN.md` v2.5 §4–§6 governs the architecture

---

## 1. Overview

This document defines the Power Distribution Unit (PDU) design for the SMART-QSO 1U CubeSat, following NASA Ames SmallSat heritage and EEE-INST-002 derating guidelines.

---

## 2. Voltage Rail Definitions

Version 1.0 defined a 2S bus (6.0–8.4 V), a 3.3 V LDO from 5 V and an optional 12 V boost. The pack is 4S
(BASELINE §5): the bus is unregulated VBAT and every converter takes it directly (finding F6 of SQSO-HW-DIAG-001).

### 2.1 Primary Rails

| Rail | Nominal | Range | Source | Load |
|------|---------|-------|--------|------|
| VBAT | 14.4 V | 11.2 V (BMS cut) – 16.4 V (CV 4.10 V/cell); 16.8 V abs max | Battery and trackers | Heater, burn wires, Jetson carrier (through the eFuse), the bucks |
| 3V3_AO | 3.3 V | 3.23–3.37 V (± 2 %, RF board LMX2571 ≥ 3.15 V) | Buck U15 from VBAT, always on | OBC (SW10), EPS controller (SW9), receiver/TNC/exciter (SW2), ADCS, GNSS, AntS |
| 5V_PA | 5.0 V | 4.90–5.10 V | Buck U16 from VBAT | PA only (EMC rule E3); 1.76 W DC at 0.5 W RF |
| V_BURN | 7.04 V | 6.83–7.25 V | Buck U17 from VBAT_DEP (SW6) | Four burn drivers, only with DEPLOY_EN |

### 2.2 Rail Specifications

| Rail | Max load | Converter rating | Regulation | Efficiency |
|------|----------|------------------|------------|------------|
| 3V3_AO | 0.30 A | ≥ 0.6 A, input ≥ 25 V | ±3 % | ≥ 85 % at 0.1 A |
| 5V_PA | 0.54 A | ≥ 1.2 A, input ≥ 25 V | ±3 % | ≥ 90 % |

## 3. Power Regulators

| Parameter | 3V3_AO buck | 5V_PA buck |
|-----------|-------------|------------|
| Topology | Synchronous buck | Synchronous buck |
| Input | starts 10.5 V, stops 7.5 V (EN divider; fixed EN hysteresis, BRD-001 §2.2) – 16.8 V; 42 V abs max | starts 12.1 V, stops 8.7 V |
| Switching | 1.000 MHz from the EPS's own 4 MHz crystal oscillator ÷ 4; no spread spectrum (SQSO-EMC-001 rule E1) | same |
| Protection | OCP, OTP, UVLO at 10.8 V falling | same |
| Part | TI LMR43620MC3RPERQ1 (2 A, 42 V abs, sync 0.2–2.5 MHz, no spread spectrum) | TI LMR43620MC5RPERQ1 (same; also the 7.0 V burn buck U17) |

## 4. Load Switch Design

### 4.1 Load Switch Configuration

| Switch | Load | Rail | Nom current | Peak current | Limit / trip (min–max) | Off after a trip | Default |
|--------|------|------|-------------|--------------|------------------------|------------------|---------|
| SW1 + eFuse U19 | Jetson carrier | VBAT | 0.62 A | 0.93 A | SW1 2.0 A; eFuse 1.20 A (1.10–1.30) latch-off | OBC re-enables | OFF |
| SW2 | Receiver, synthesizer, TNC, exciter | 3V3_AO | 0.038 A | 0.05 A | 0.12 A (0.10–0.14) | 1.3 s, auto | OFF at reset; on at boot |
| SW3 | PA (hardware TX-OFF) | 5V_PA | 0.37 A (1.84 W DC at 0.5 W RF, SQSO-RF-BRD-001) | 0.55 A (EPS board design bound; 0.49 A at 1 W RF) | 0.99 A (0.92–1.07) | 0.28 s, auto | OFF |
| SW4 | ADCS sensors (on the OBC, W6B) | 3V3_AO | 0.04 A | 0.049 A | 0.30 A (0.27–0.34) | 0.28 s, auto | OFF at reset |
| SW5 | Battery heater (207 Ω) | VBAT | 0.07 A | 0.08 A | 0.148 A, plus F4 0.25 A and a +40 °C cutout in series | auto-retry | OFF |
| SW6 | Burn buck → four drivers | VBAT | 0 | 0.5 A per burn at the pack | 2.0 A; 0.8 A per resistor; 12 s one-shot | thermal | OFF |
| SW7 | GNSS receiver | 3V3_AO | 0.04 A | 0.1 A | 0.20 A (0.18–0.23) | 1.3 s, auto | OFF |
| SW8 | ISIS AntS | 3V3_AO | 0 | 0.61 A | 1.37 A | 0.28 s, auto | OFF (DEPLOY_EN OR the EPS controller's ANTS_PWR) |
| SW9 | EPS controller | 3V3_AO | 0.012 A | — | 75 mA | 0.28 s, auto | ON |
| SW10 | OBC | 3V3_AO | 0.04 A | — | 0.15 A (0.13–0.18) | 0.28 s, auto | ON (RID AV-08) |
| SW11 | OBC magnetorquer H-bridges (VBAT_MTQ) | VBAT | 0.083 A | 0.23 A (bridges' limit) | 0.36 A (0.32–0.40) | auto-retry 40 ms / 1 s | OFF (ADCS_EN) |

### 4.2 Load Switch IC Selection

| Parameter | 3.3/5 V switches (SW2–SW4, SW7–SW10) | VBAT switches (SW1, SW5, SW6, SW11) |
|-----------|---------------------------------------|-------------------------------|
| IC | TI TPS2553-Q1 (2.5–6.5 V, adjustable 75 mA–1.3 A, FAULT tied to EN for auto-retry) | TPS1H100B-Q1 (SW1, SW6; 40 V, 100 mΩ, current sense), TPS1H000-Q1 (SW5, SW11; 40 V, 1 Ω, 0.05–1 A); eFuse TPS16630 (60 V) |
| Latch-up behaviour | current limited in 2 µs, rail removed after 8 ms, back after R_FAULT × C_RETRY | current limit, thermal cycling; eFuse latch-off |
| Fault output | to the EPS controller (register LOAD_SHED/fault bits) | same |
| Control | OBC GPIO with 100 kΩ pull-down; SW1 and the Jetson eFuse also gated by the hardware load shed (the PA's SW3 is not shed from SQSO-EPS-BRD-001 E) | same |

### 4.3 Load Switch Features

- Controlled slew rate for inrush limiting
- Reverse current blocking
- Thermal shutdown protection
- Fault flag output to OBC
- Every enable low at reset (pull-down)

## 5. Current Sensing

### 5.1 Current Sense Points (SQSO-EPS-BRD-001 §11)

| Point | Device | Shunt | Interface |
|-------|--------|-------|-----------|
| Battery (fuel gauge) | INA228-Q1 U28, high side | 10 mΩ RS6 | OBC I2C2 0x44 |
| Battery (analog IBATT_SNS) | INA186A2-Q1, 0.5 V/A | RS6 | OBC ADC PC3 |
| Array output | INA228-Q1 U29 and INA186A2-Q1 (ISOLAR_SNS 1 V/A) | 20 mΩ RS5 | I2C2 0x40, OBC ADC PC5 |
| Tracker inputs ×4, bus load, 3V3_AO, 5V_PA, radio, ADCS, GNSS, AntS, heater | INA3221-Q1 ×4 | 10–500 mΩ | EPS controller local bus |
| Burn currents ×4 | 0.25 Ω sense resistors | RS20–RS23 | EPS controller ADC |
| Jetson | eFuse IMON (0.84 V/A) | internal | EPS controller ADC |

---

## 6. Overcurrent Protection

### 6.1 Protection Levels

| Point | Warning (software) | Trip (hardware) | Action |
|-------|--------------------|-----------------|--------|
| Battery | 2.5 A | BMS over-current 4 A for 0.5 s (below the 5 A fuse); short-circuit trip ≤ 200 µs | Discharge FET opens |
| Jetson (VBAT) | 8 W for 2 s (`payload_guard.rs`) | eFuse 1.2 A; guard 12 W | Rail off, latched |
| 5V_PA | — | SW3 1.0 A | Rail off |
| Low-voltage load shed | — | VBAT < 13.6 V for 5 s | SW1 and the Jetson eFuse off; the PA is not shed (EPS_DESIGN §6) |

### 6.2 Protection Implementation

- Electronic current limiting in the regulators and switches
- Hardware comparator backups for the battery (SQSO-BSA-001 §3)
- Software monitoring with hysteresis
- Fuse backup

### 6.3 Fuse Sizing

| Location | Type | Rating |
|----------|------|--------|
| Battery output | Non-resettable fuse F2 FM12A72V5.0A (MIL-PRF-23419/12); F0, the same part, on the battery sense board | 5 A, 72 VDC, interrupt 1000 A (prospective ≤ 314 A, SQSO-EPS-BRD-001 §6.1) |
| Battery output | PTC RT1 Littelfuse 2920L260/33DR | Hold 2.6 A, trip 5.0 A, 33 V |
| Pack line | Secondary over-voltage protector's three-terminal fuse F3 ITV4030L1412NR | 12 A, heater-blown by the BQ771806 (pack isolated on a trip) |
| Array output | EOL fuse F1 ITV4030L1412NR | 12 A, heater-blown by two series switches |
| Heater | Fuse F4 0466.250NR | 0.25 A |
| Each load switch | Internal | §4.1 |

## 7. Inrush Current Limiting

### 7.1 Inrush Requirements

| Load | Expected Inrush | Limit Method |
|------|-----------------|--------------|
| Jetson | 5.0 A peak | Soft-start |
| RF TX | 2.0 A peak | Soft-start |
| Capacitive loads | 1.5× nominal | Slew control |

### 7.2 Implementation

- Programmable soft-start on all load switches
- Output capacitor pre-charge
- Staggered power-up sequencing
- dV/dt limited to < 1 V/ms

---

## 8. Power Sequencing

### 8.1 Power-Up Sequence

| Step | Action | Delay | Condition |
|------|--------|-------|-----------|
| 1 | VBAT enable | 0 ms | Deployment switch |
| 2 | +3.3V enable | 10 ms | VBAT stable |
| 3 | +5V enable | 50 ms | +3.3V stable |
| 4 | OBC boot | 100 ms | +5V stable |
| 5 | ADCS enable | 500 ms | OBC command |
| 6 | RF enable | 1000 ms | OBC command |
| 7 | Jetson enable | OBC cmd | Power mode check |

### 8.2 Power-Down Sequence

| Step | Action | Delay | Condition |
|------|--------|-------|-----------|
| 1 | Jetson shutdown | 0 ms | OBC command |
| 2 | RF disable | 100 ms | Jetson off |
| 3 | Non-essential loads | 200 ms | Sequential |
| 4 | Enter safe mode | 500 ms | All loads off |

### 8.3 Sequencing Controller

- Hardware sequencer IC or OBC-controlled
- Watchdog-protected sequencing
- Manual override for test
- State machine in OBC firmware

---

## 9. Connector and Harness Interface

### 9.1 Connector Selection

| Interface | Connector | Pins | Mating Cycles |
|-----------|-----------|------|---------------|
| Battery | Molex Micro-Fit (power), Pico-Clasp (5 cell taps, finding F12) | 6 + 5 | 25 |
| Solar array | Molex Pico-Clasp | 6 | 25 |
| OBC interface | Samtec FTSH | 20 | 100 |
| Load outputs | Molex Micro-Fit | 10 | 25 |

### 9.2 Harness Requirements

| Parameter | Specification |
|-----------|---------------|
| Power wires | 22-20 AWG (per current) |
| Signal wires | 26-28 AWG |
| Insulation | PTFE or Kapton rated |
| Strain relief | At all connections |
| Length tolerance | ±5 mm |

---

## 10. Telemetry Points

### 10.1 Voltage Telemetry

| Point | Range | Resolution | ADC Channel |
|-------|-------|------------|-------------|
| VBATT_SNS (PACK_P) | 0-19.8 V (1:6 divider, 1.00 M / 200 k, finding F1) | 5 mV | OBC PC2 |
| +5V | 0-6V | 10 mV | ADC1 |
| +3.3V | 0-4V | 5 mV | ADC2 |
| Solar V | 0-10V | 10 mV | ADC4 |

### 10.2 Current Telemetry

| Point | Range | Resolution | Interface |
|-------|-------|------------|-----------|
| Battery I | ±3.3 A | 0.01 mA (INA228) | I2C2 0x44 (INA228-Q1) and analog IBATT_SNS |
| Solar I | 0-3 A | 0.01 mA | I2C2 0x40 (INA228-Q1) and analog ISOLAR_SNS |
| 5V_PA I | 0-1 A | 0.4 mA | INA3221-Q1 via the EPS controller |

### 10.3 Status Telemetry

| Point | Type | Interface |
|-------|------|-----------|
| SW1-SW5 status | Boolean | GPIO |
| OCP fault flags | Boolean | GPIO |
| Regulator PG | Boolean | GPIO |
| Temperature | Analog | ADC |

---

## 11. EMC Considerations

### 11.1 Input Filtering

| Filter | Specification |
|--------|---------------|
| Common mode choke | 10 µH, 3A rated |
| Differential capacitor | 10 µF ceramic |
| Damping resistor | 1-10 Ω |

### 11.2 Switching Noise Mitigation

- Fixed-frequency switchers synchronised to a crystal-derived clock, with a frequency plan away from the uplink channel; no spread-spectrum modulation, which would raise the 70 cm receiver's noise floor (SQSO-EMC-001 rule E1)
- Snubber circuits on switching nodes
- Shielded inductors
- Careful PCB layout with ground planes
- LC filters on sensitive rails

### 11.3 Grounding

- Single-point star ground for analog/digital
- Separate PGND and SGND
- Ground plane under switchers
- Via stitching around switching area

---

## 12. PCB Layout Guidelines

### 12.1 Stack-Up

| Layer | Function |
|-------|----------|
| L1 | Components, signal |
| L2 | Ground plane |
| L3 | Power planes |
| L4 | Signal, components |

### 12.2 Layout Rules

| Parameter | Specification |
|-----------|---------------|
| Power trace width | 40 mil/A minimum |
| Thermal vias | 0.3mm, 4+ per pad |
| Keep-out zones | 5mm from switchers |
| Copper pour | For heat spreading |
| Via stitching | Every 5mm on GND |

---

## 13. Testing Requirements

### 13.1 Functional Tests

| Test | Requirement |
|------|-------------|
| Voltage accuracy | All rails ± spec |
| Load regulation | < 2% at full load |
| Line regulation | < 1% over Vin range |
| Efficiency | > target per rail |
| Protection trip | All OCP/OVP functional |

### 13.2 Environmental Tests

| Test | Condition |
|------|-----------|
| Temperature | -20°C to +60°C |
| Thermal cycling | 10 cycles |
| Vibration | Per launch provider |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-27 | CDR review (EPS): rails, regulators, switches, protection and fuses corrected to the 4S bus (§2–§4, §6); no spread spectrum (§11.2); VBAT telemetry range; battery connector with cell taps. EPS_DESIGN v2.0 governs |
| 1.2 | 2026-10-27 | CDR design (EPS board) | Regulators, switches, sensing and fuses selected and sized in SQSO-EPS-BRD-001 A (§2–§6, §10): LMR43620-Q1 bucks incl. the 7.0 V burn buck, TPS2553/TPS1H100/TPS1H000/TPS16630 switches with latch-up trips (radio 0.12 A, OBC rail SW10), INA228/INA3221/INA186 sensing; 3V3_AO ± 2 % |
| 1.3 | 2026-10-27 | CDR design (EPS board) | SQSO-ICX-001 interconnect (SQSO-EPS-BRD-001 B): SW11 magnetorquer supply (TPS1H000-Q1, 0.36 A), SW4 sensors only (§4.1, §4.2) |
| 1.4 | 2026-10-27 | CDR design (EPS board): battery-output fuse F2 changed to FM12A72V5.0A for breaking capacity (§6) |
| 1.5 | 2026-10-27 | CDR design (EPS board): load shed on the Jetson's two switches only, the PA not shed; AntS supply also by the EPS controller (SQSO-EPS-BRD-001 E) |
| 1.6 | 2026-10-27 | Documentation audit (post-CDR 2026): status cites SQSO-EPS-BRD-001 G (was E); SW3 nominal current from the RF board's 1.84 W DC at 0.5 W RF (was 0.35 A) |
