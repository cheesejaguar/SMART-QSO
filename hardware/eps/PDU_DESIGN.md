# Power Distribution Unit (PDU) Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-PDU-001
**Version:** 1.1
**Date:** 2026-10-27
**Status:** CDR Baseline — rails, switches and protection corrected to the 4S bus (CDR review); `docs/hardware/EPS_DESIGN.md` v2.0 §4–§6 governs

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
| 3V3_AO | 3.3 V | 3.20–3.40 V | Buck from VBAT, always on | OBC, FRAM, receiver/TNC/exciter (own LDO), ADCS, GNSS |
| 5V_PA | 5.0 V | 4.85–5.15 V | Buck from VBAT | PA only (EMC rule E3) |

### 2.2 Rail Specifications

| Rail | Max load | Converter rating | Regulation | Efficiency |
|------|----------|------------------|------------|------------|
| 3V3_AO | 0.30 A | ≥ 0.6 A, input ≥ 25 V | ±3 % | ≥ 85 % at 0.1 A |
| 5V_PA | 0.54 A | ≥ 1.2 A, input ≥ 25 V | ±3 % | ≥ 90 % |

## 3. Power Regulators

| Parameter | 3V3_AO buck | 5V_PA buck |
|-----------|-------------|------------|
| Topology | Synchronous buck | Synchronous buck |
| Input | 10.8 (UVLO, falling) – 16.8 V, rated ≥ 25 V | same |
| Switching | Fixed frequency, synchronised to the OBC-derived clock; no spread spectrum (SQSO-EMC-001 rule E1) | same |
| Protection | OCP, OTP, UVLO at 10.8 V falling | same |
| Part | Not yet selected (TPS62180 and TPS7A8300 of version 1.0 cannot take 16.8 V) | Not yet selected |

## 4. Load Switch Design

### 4.1 Load Switch Configuration

| Switch | Load | Rail | Nom current | Peak current | Limit / trip | Default |
|--------|------|------|-------------|--------------|--------------|---------|
| SW1 + eFuse | Jetson carrier | VBAT | 0.62 A | 0.93 A | eFuse 1.2 A, latch-off ≤ 1 ms | OFF |
| SW2 | Receiver, TNC, exciter | 3V3_AO | 0.05 A | 0.1 A | 0.25 A | ON |
| SW3 | PA (hardware TX-OFF) | 5V_PA | 0.30 A | 0.54 A | 1.0 A | OFF |
| SW4 | ADCS | 3V3_AO | 0.04 A | 0.13 A | 0.3 A | ON (by software) |
| SW5 | Battery heater (≈ 200 Ω) | VBAT | 0.07 A | 0.08 A | 0.15 A, plus 0.25 A fuse and +40 °C cutout | OFF |
| SW6 | Burn-wire drivers | VBAT | 0 | 1.6 A | 2.0 A; 0.8 A per resistor | OFF |
| SW7 | GNSS receiver | 3V3_AO | 0.04 A | 0.1 A | 0.15 A, latch-up trip | OFF |

### 4.2 Load Switch IC Selection

| Parameter | 3.3/5 V switches (SW2, SW3, SW4, SW7) | VBAT switches (SW1, SW5, SW6) |
|-----------|---------------------------------------|-------------------------------|
| IC class | TPS22918 (5.5 V max) | High-side switch rated ≥ 25 V with adjustable current limit (TPS1H100 class); eFuse TPS25947 class for the Jetson |
| Rise time | 1–5 ms | ≥ 1 ms soft start |
| Fault output | To an OBC input (finding F16) | same |
| Control | GPIO from the OBC, pull-down; SW1 and SW3 also gated by the hardware low-voltage load shed (EPS_DESIGN §6) | same |

### 4.3 Load Switch Features

- Controlled slew rate for inrush limiting
- Reverse current blocking
- Thermal shutdown protection
- Fault flag output to OBC
- Every enable low at reset (pull-down)

## 5. Current Sensing

### 5.1 Current Sense Points

| Point | Range | Resolution | Method |
|-------|-------|------------|--------|
| Battery current | ±3.0 A | 10 mA | High-side sense |
| Solar input | 0-2.0 A | 5 mA | High-side sense |
| 5V rail | 0-4.0 A | 10 mA | High-side sense |
| Jetson load | 0-3.5 A | 10 mA | Low-side sense |

### 5.2 Current Sense IC

| Parameter | Specification |
|-----------|---------------|
| IC | INA226 or equivalent |
| Shunt Resistor | 10-50 mΩ (per rail) |
| ADC Resolution | 16-bit |
| Interface | I2C |
| Sample Rate | 140 µs min |
| Accuracy | ±0.1% FSR |

---

## 6. Overcurrent Protection

### 6.1 Protection Levels

| Point | Warning (software) | Trip (hardware) | Action |
|-------|--------------------|-----------------|--------|
| Battery | 2.5 A | BMS over-current 4 A for 0.5 s (below the 5 A fuse); short-circuit trip ≤ 200 µs | Discharge FET opens |
| Jetson (VBAT) | 8 W for 2 s (`payload_guard.rs`) | eFuse 1.2 A; guard 12 W | Rail off, latched |
| 5V_PA | — | SW3 1.0 A | Rail off |
| Low-voltage load shed | — | VBAT < 13.6 V for 5 s | SW1 and SW3 off (EPS_DESIGN §6) |

### 6.2 Protection Implementation

- Electronic current limiting in the regulators and switches
- Hardware comparator backups for the battery (SQSO-BSA-001 §3)
- Software monitoring with hysteresis
- Fuse backup

### 6.3 Fuse Sizing

| Location | Type | Rating |
|----------|------|--------|
| Battery output | Non-resettable fuse F2 | 5 A, breaking capacity ≥ 100 A at 32 VDC |
| Battery output | PTC RT1 | Hold 2.5 A, trip ≤ 5 A (was 4 A hold, 8 A trip: equal to the cell's 8 A continuous limit) |
| Charge path | Secondary over-voltage protector's three-terminal fuse | Sized for the 0.40 A charge current |
| Heater | Fuse | 0.25 A |
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
| VBAT | 0-20 V (1:6 divider, finding F1) | 10 mV | ADC0 |
| +5V | 0-6V | 10 mV | ADC1 |
| +3.3V | 0-4V | 5 mV | ADC2 |
| Solar V | 0-10V | 10 mV | ADC4 |

### 10.2 Current Telemetry

| Point | Range | Resolution | Interface |
|-------|-------|------------|-----------|
| Battery I | ±3A | 10 mA | I2C (INA226) |
| Solar I | 0-2A | 5 mA | I2C (INA226) |
| 5V rail I | 0-4A | 10 mA | I2C (INA226) |

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
