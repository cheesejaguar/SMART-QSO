# RF Transmitter Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-TX-001
**Version:** 1.2
**Date:** 2026-10-26
**Status:** CDR Baseline

---

## 1. Overview

This document defines the RF transmitter design for the SMART-QSO 1U CubeSat beacon system, following IARU coordination requirements and amateur radio regulations.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Frequency | 145.825 MHz (superseded: one IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz, BASELINE §7) | IARU Coordination |
| Modulation | AFSK (Bell 202) | AX.25 Standard |
| Data Rate | 1200 bps | AX.25 Standard |
| Output Power | 0.5 - 2.0 W (superseded: maximum 1 W, nominal 0.5 W, BASELINE §7) | Mission/Link Budget |
| Harmonics | < -43 dBc | IARU |
| Spurious | < -60 dBc | ITU |

---

## 2. Transmitter Architecture

### 2.1 Block Diagram

```
Audio In → Modulator → VCO/PLL → Driver → PA → LPF → Antenna
                         ↑
                    Reference
                    Oscillator
```

### 2.2 Architecture Selection

| Approach | Selected | Rationale |
|----------|----------|-----------|
| Single-chip transceiver | Yes | Integration, reliability |
| Discrete stages | No | Complexity, space |
| SDR approach | No | Power consumption |

---

## 3. Transceiver Module Selection

### 3.1 Selected Module

| Parameter | Value |
|-----------|-------|
| Type | VHF FM Transceiver Module |
| Examples | Radiometrix TX2M, DRA818V, or equivalent |
| Frequency Range | 134-174 MHz |
| Channel Spacing | 12.5/25 kHz |
| Power Output | 0.5W - 1W |
| Supply Voltage | 3.3V - 5.0V |
| Current (TX) | 450 mA @ 1W |

### 3.2 Alternate: Discrete Design

If higher power required:

| Parameter | Value |
|-----------|-------|
| VCO/Synthesizer | ADF7021 or Si4463 |
| PA Stage | RFMD/Qorvo 2W PA module |
| LPF | Custom 7-pole Chebyshev |

---

## 4. Frequency Synthesizer

### 4.1 Reference Oscillator

| Parameter | Value |
|-----------|-------|
| Type | TCXO |
| Frequency | 12.8 MHz or 26 MHz |
| Stability | ±2.5 ppm (-40 to +85°C) |
| Phase Noise | < -110 dBc/Hz @ 10kHz |
| Aging | ±1 ppm/year |

### 4.2 PLL Configuration

| Parameter | Value |
|-----------|-------|
| Loop Bandwidth | 5 kHz |
| Phase Detector Freq | 12.5 kHz |
| Lock Time | < 1 ms |
| Frequency Resolution | 12.5 kHz steps |

### 4.3 Frequency Accuracy

| Condition | Accuracy |
|-----------|----------|
| At 25°C | ±500 Hz |
| Over temperature | ±2 kHz |
| End of life | ±3 kHz |
| **Requirement** | **±5 kHz** |

---

## 5. Modulator Design

### 5.1 AFSK Parameters (Bell 202)

| Parameter | Value |
|-----------|-------|
| Mark Frequency | 1200 Hz |
| Space Frequency | 2200 Hz |
| Bit Rate | 1200 bps |
| Modulation Type | FSK (FM) |

### 5.2 FM Deviation

| Parameter | Value |
|-----------|-------|
| Nominal Deviation | ±3.5 kHz |
| Deviation Tolerance | ±0.5 kHz |
| Modulation Index | ~1.0 (narrowband FM) |

### 5.3 Modulator Implementation

- Direct modulation of VCO varactor
- Pre-emphasis filter (optional, +6 dB/octave)
- Level control via potentiometer or DAC
- Audio input: 0-3.3V from MCU DAC or PWM

---

## 6. Power Amplifier

### 6.1 PA Specifications

| Parameter | Value |
|-----------|-------|
| Output Power | 1W nominal (30 dBm) |
| Power Range | 0.5W - 2W selectable |
| Gain | 15-20 dB |
| Efficiency | > 50% |
| Supply | 5V |
| Current | 400-800 mA |

### 6.2 Power Control

| Setting | Power | Current |
|---------|-------|---------|
| Low | 0.5W (27 dBm) | 250 mA |
| Medium | 1.0W (30 dBm) | 450 mA |
| High | 2.0W (33 dBm) | 800 mA |

### 6.3 PA Protection

- Thermal shutdown at 85°C junction
- VSWR protection (foldback)
- Supply voltage monitoring
- Enable control from OBC

---

## 7. Output Matching and Filtering

### 7.1 Output Match

| Parameter | Value |
|-----------|-------|
| Impedance | 50Ω |
| Return Loss | > 15 dB |
| Bandwidth | 144-148 MHz |

### 7.2 Low-Pass Filter Design

| Parameter | Value |
|-----------|-------|
| Type | 7-pole Chebyshev |
| Cutoff | 160 MHz |
| Passband Ripple | 0.5 dB |
| 2nd Harmonic Rej | > 50 dB |
| 3rd Harmonic Rej | > 60 dB |

### 7.3 Filter Component Values

| Element | Value | Type |
|---------|-------|------|
| L1 | 47 nH | Air core |
| C1 | 33 pF | NP0/C0G |
| L2 | 82 nH | Air core |
| C2 | 47 pF | NP0/C0G |
| L3 | 82 nH | Air core |
| C3 | 33 pF | NP0/C0G |
| L4 | 47 nH | Air core |

---

## 8. RF Power Detection

### 8.1 Forward Power Monitor

| Parameter | Value |
|-----------|-------|
| Method | Directional coupler + detector |
| Coupling | -20 dB |
| Detector | AD8361 or equivalent |
| Output | 0-3V analog |
| Accuracy | ±0.5 dB |

### 8.2 Reflected Power Monitor

| Parameter | Value |
|-----------|-------|
| Method | Same coupler, reverse port |
| VSWR Threshold | 3:1 for alarm |
| Action | Reduce power / disable |

---

## 9. Temperature Compensation

### 9.1 Power vs. Temperature

| Temperature | Power Variation |
|-------------|-----------------|
| -20°C | +0.5 dB |
| +25°C | Reference |
| +50°C | -0.5 dB |

### 9.2 Compensation Method

- Temperature sensor on PA
- Lookup table in firmware
- Bias adjustment for flat power

---

## 10. Control Interface

### 10.1 OBC Interface

| Signal | Type | Function |
|--------|------|----------|
| TX_EN | GPIO | Enable transmitter (PA power; `GPIO_PIN_RF_TX_EN`, driven by TX-OFF and the inhibits, `rf_control.h`) |
| TX_DATA | UART | AX.25 packet data: KISS frames to a TNC on USART2, which adds the FCS, modulates and keys (CR-2026-005) |
| TX_PWR | GPIO×1 | Power level select: low 0.5 W, high 1 W (`GPIO_PIN_TX_PWR_SEL`); the 2 W level of §6.2 is not used (BASELINE §7) |
| TX_TEMP | ADC | Temperature monitor (`ADC_CHANNEL_TX_TEMP`) |
| TX_FWD | ADC | Forward power (`ADC_CHANNEL_TX_FWD`) |
| TX_REV | ADC | Reflected power (`ADC_CHANNEL_TX_REV`) |
| XPDR_EN | GPIO | FM transponder enable: while high, the repeat path keys the PA on a CTCSS-valid uplink carrier (`GPIO_PIN_XPDR_EN`; the TNC's frames take the transmitter first). Driven by `sqso_fsw::transponder` |

As built (2026-10-25, CR-2026-005): the OBC software is `software/flight/include/downlink.h`. It hands a frame to the
TNC only while transmission is permitted, one frame at a time, at most 5 s on the air and 25 % of the time; selects
TX_PWR; and acts on the monitors as §8.2 and §6.3 ask: VSWR above 3:1 reduces the power to 0.5 W, then stops the
downlink for 10 minutes; above 70 °C it holds the downlink until 60 °C. The values are TBR-SRS-003 until the
transmitter and TNC are selected.

XPDR_EN (2026-10-26, from the mission simulation, `docs/test/SIL_MISSION_SIMULATION.md`): the interface had no
way for the OBC to gate the transponder, so the power closure's transponder line (BASELINE §5: sunlit, SOC ≥ 55 %,
duty ≤ 25 %) could not be enforced. The OBC now holds XPDR_EN high only while transmission is permitted, the mode is
ACTIVE, the array is sunlit, the SOC is at least 55 %, the PA is below 70 °C (resuming at 60 °C) and the keyed time,
measured on TX_FWD, is under 25 % of the last 100 minutes. The line is a placeholder until the transponder's
repeat-path hardware is selected.

### 10.2 Timing

| Parameter | Value |
|-----------|-------|
| TX enable to RF | < 10 ms |
| RF to TX disable | < 1 ms |
| Power settle time | < 5 ms |

---

## 11. EMC Design

### 11.1 Shielding

- RF section in shielded enclosure
- Feedthrough capacitors on DC lines
- EMI gasket on cover

### 11.2 Grounding

- Single-point RF ground
- Star ground at PA
- Ground plane under all RF traces

### 11.3 PCB Layout

| Requirement | Implementation |
|-------------|----------------|
| RF traces | 50Ω microstrip |
| Via fencing | Around RF section |
| Component spacing | >3× component size |
| Ground vias | Every λ/20 |

---

## 12. Spurious Emissions Compliance

### 12.1 IARU Limits

| Emission | Limit | Design Target |
|----------|-------|---------------|
| 2nd Harmonic | < -43 dBc | < -50 dBc |
| 3rd Harmonic | < -43 dBc | < -55 dBc |
| Spurious (general) | < -60 dBc | < -65 dBc |

### 12.2 Mitigation

- 7-pole LPF provides > 50 dB rejection
- Shielded enclosure
- Careful power supply filtering
- Proper grounding

---

## 13. Antenna Interface

### 13.1 Connector

| Parameter | Value |
|-----------|-------|
| Type | SMA female (board) |
| Impedance | 50Ω |
| Frequency Range | DC - 6 GHz |
| Power Rating | 10W |

### 13.2 Cable

| Parameter | Value |
|-----------|-------|
| Type | RG-316 or equivalent |
| Length | < 150 mm |
| Loss | < 0.5 dB |

---

## 14. Testing Requirements

### 14.1 Module Tests

| Test | Requirement |
|------|-------------|
| Output power | 30 ±1 dBm |
| Frequency accuracy | ±5 kHz |
| Harmonics | < -43 dBc |
| Spurious | < -60 dBc |
| Current consumption | Per specification |

### 14.2 Environmental Tests

| Test | Condition |
|------|-----------|
| Temperature | -20°C to +50°C |
| Vibration | Launch levels |
| TVAC | Operational |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-25 | §10.1 as built (CR-2026-005): KISS TNC on USART2, one power-select line, the monitors' channels and the software's actions; §1.1 frequency and power marked superseded by BASELINE §7 |
| 1.2 | 2026-10-26 | §10.1: XPDR_EN, the transponder enable, and the gates the OBC applies to it (found by the mission simulation) |
