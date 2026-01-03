# Power Distribution Unit (PDU) Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-PDU-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the Power Distribution Unit (PDU) design for the SMART-QSO 1U CubeSat, following NASA Ames SmallSat heritage and EEE-INST-002 derating guidelines.

---

## 2. Voltage Rail Definitions

### 2.1 Primary Rails

| Rail | Nominal | Range | Source | Load |
|------|---------|-------|--------|------|
| VBAT | 7.2V | 6.0-8.4V | Battery | MPPT, 5V reg |
| +5V | 5.0V | 4.75-5.25V | Buck regulator | Jetson, OBC |
| +3.3V | 3.3V | 3.14-3.47V | LDO from 5V | OBC core, sensors |
| +12V | 12V | 11.4-12.6V | Boost from VBAT | RF PA (optional) |

### 2.2 Rail Specifications

| Rail | Max Current | Regulation | Efficiency |
|------|-------------|------------|------------|
| +5V | 4.0 A | ±5% | >90% |
| +3.3V | 1.0 A | ±5% | >85% |
| +12V | 0.5 A | ±5% | >85% |

---

## 3. Power Regulators

### 3.1 5V Buck Regulator

| Parameter | Value |
|-----------|-------|
| Topology | Synchronous buck |
| IC | TPS62180 or equivalent |
| Vin Range | 6.0V - 8.4V |
| Vout | 5.0V ± 2% |
| Iout Max | 4.0 A |
| Switching Freq | 2.25 MHz |
| Efficiency | >92% @ 1A load |
| Protection | OCP, OTP, UVLO |

### 3.2 3.3V LDO Regulator

| Parameter | Value |
|-----------|-------|
| IC | TPS7A8300 or equivalent |
| Vin | 5.0V rail |
| Vout | 3.3V ± 1% |
| Iout Max | 1.0 A |
| Dropout | < 200 mV |
| PSRR | > 60 dB @ 1 kHz |
| Noise | < 10 µVrms |

### 3.3 12V Boost Regulator (Optional)

| Parameter | Value |
|-----------|-------|
| Topology | Boost converter |
| IC | TPS61088 or equivalent |
| Vin Range | 6.0V - 8.4V |
| Vout | 12.0V ± 3% |
| Iout Max | 0.5 A |
| Protection | OCP, OVP |

---

## 4. Load Switch Design

### 4.1 Load Switch Configuration

| Switch | Load | Nom Current | Peak Current | Default |
|--------|------|-------------|--------------|---------|
| SW1_JETSON | Jetson module | 2.0 A | 3.5 A | OFF |
| SW2_RF_TX | RF transmitter | 0.5 A | 1.0 A | OFF |
| SW3_ADCS | ADCS board | 0.2 A | 0.5 A | ON |
| SW4_DEPLOY | Deployment mech | 0.1 A | 2.0 A | OFF |
| SW5_HEATER | Battery heater | 0.3 A | 0.5 A | OFF |

### 4.2 Load Switch IC Selection

| Parameter | Specification |
|-----------|---------------|
| IC | TPS22918 or equivalent |
| Rds(on) | < 15 mΩ |
| Rise Time | 1-5 ms (configurable) |
| Fault Output | Active-low flag |
| Control | GPIO from OBC |

### 4.3 Load Switch Features

- Controlled slew rate for inrush limiting
- Reverse current blocking
- Thermal shutdown protection
- Fault flag output to OBC
- Manual override capability (for test)

---

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

| Rail | Warning | Trip | Action |
|------|---------|------|--------|
| Battery | 2.5 A | 3.5 A | Load shed |
| +5V | 3.5 A | 4.5 A | Rail shutdown |
| +3.3V | 0.8 A | 1.2 A | Rail shutdown |
| +12V | 0.4 A | 0.6 A | Rail shutdown |

### 6.2 Protection Implementation

- Electronic current limiting in regulators
- Load switch current limit
- Hardware comparator backup trip
- Software monitoring with hysteresis
- Fuse backup (resettable PTC)

### 6.3 Fuse Sizing

| Location | Type | Rating |
|----------|------|--------|
| Battery output | PTC | 4.0 A hold, 8.0 A trip |
| Solar input | PTC | 2.5 A hold, 5.0 A trip |
| Each load switch | Internal | Per switch current limit |

---

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
| Battery | Molex Micro-Fit | 4 | 25 |
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
| VBAT | 0-10V | 10 mV | ADC0 |
| +5V | 0-6V | 10 mV | ADC1 |
| +3.3V | 0-4V | 5 mV | ADC2 |
| +12V | 0-15V | 20 mV | ADC3 |
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

- Spread-spectrum modulation on switchers
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
