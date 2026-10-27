# RF Transmitter Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-TX-001
**Version:** 1.3
**Date:** 2026-10-27
**Status:** CDR Baseline

---

## 1. Overview

This document defines the 2 m transmitter of the SMART-QSO 1U CubeSat. It carries the AX.25 beacon (through the KISS TNC) and the FM transponder downlink (through the repeat path), time-shared (`docs/hardware/RF_DESIGN.md` v2.0 §3). Values follow BASELINE §7; the link budget is SQSO-LBA-001 v3.0. Version 1.3 (CDR) removes the 2 W level, sets the deviation, stability and harmonic requirements, and records the filter analysis.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Frequency | One IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz | BASELINE §7 |
| Modulation | FM: AFSK (Bell 202) beacon, ±3.0 kHz peak (10K4F2D); FM voice repeat, ±5.0 kHz peak with limiter (16K0F3E) | ITU_APPENDIX4_DATA §C |
| Data Rate | 1200 bps | AX.25 Standard |
| Output Power | 0.5 W nominal, 1 W maximum, at the antenna port | BASELINE §7 |
| Harmonics and spurious | ≤ 25 µW and ≤ −46 dBc at 1 W (47 CFR 97.307(e)); **3rd harmonic ≤ −70 dBc at the antenna port** (proposed; BASELINE target −60 dBc; SQSO-LBA-001 §7.3) | 97.307(e); desense |
| Frequency stability | ±1.0 ppm over −20 to +60 °C and 2 years (RF-P005) | SQSO-LBA-001 §4.4 |
| Duty | Beacon ≈ 1.2 s per 60 s; transponder ≤ 25 % (software); continuous key-down ≤ 120 s (hardware timer) | BASELINE §5 |

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
| Power Output | 0.5 W / 1 W (TX_PWR_SEL) |
| Supply Voltage | 3.3V - 5.0V |
| Current (TX) | 450 mA @ 1W |

### 3.2 Alternate: Discrete Design

Flight option (preferred over modules for stability, harmonics and radiation; selection against RF_DESIGN RF-P003–P005, P008, P015):

| Parameter | Value |
|-----------|-------|
| VCO/Synthesizer | ADF7021 or Si4463 |
| PA Stage | A 1–2 W-class VHF PA device run at ≤ 1 W (derated) |
| LPF | Custom 7-pole Chebyshev |

---

## 4. Frequency Synthesizer

### 4.1 Reference Oscillator

| Parameter | Value |
|-----------|-------|
| Type | TCXO |
| Frequency | 12.8 MHz or 26 MHz |
| Stability | **±1.0 ppm** (−20 to +60 °C), shared by the 2 m exciter and the 70 cm receiver (RF-P005; ±2.5 ppm would give ±1.1 kHz at 435 MHz alone) |
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
| Over temperature | ±146 Hz (1.0 ppm) |
| End of life | ±200 Hz |
| **Requirement** | **±1.0 ppm** (SYS-206's ±5 kHz is met with margin) |

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
| Nominal deviation, beacon | **±3.0 kHz** peak (the filed 10K4F2D: 2 × 2.2 + 2 × 3.0 = 10.4 kHz; ±3.5 kHz would be 11K4F2D) |
| Nominal deviation, transponder | ±5.0 kHz peak, hard-limited (16K0F3E) |
| Deviation tolerance | ±0.5 kHz |

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
| Output Power | 0.5 W nominal (27 dBm) |
| Power Range | 0.5 W / 1 W selectable; no 2 W level (BASELINE §7) |
| Gain | 15-20 dB |
| Efficiency | ≥ 33 % overall at 0.5 W RF including the exciter (≤ 1.5 W DC, BASELINE §5) |
| Supply | Through the RF_TX_EN load switch (SQSO-ICD-003 §4.4) |
| Current | Set by the selected PA |

### 6.2 Power Control

| Setting | Power | Current |
|---------|-------|---------|
| Low (TX_PWR_SEL low) | 0.5 W (27 dBm) | ≈ 1.5 W DC |
| High (TX_PWR_SEL high) | 1.0 W (30 dBm) | ≈ 2.7 W DC |

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
| Cutoff (−3 dB, analysed) | 164 MHz |
| Insertion loss at 145.9 MHz | 0.66 dB with inductor Q 80 (0.06 dB ideal); return loss 18 dB |
| 2nd harmonic rejection (analysed) | 50.5 dB |
| 3rd harmonic rejection (analysed) | 77.5 dB ideal; **expect 50–60 dB realized** on a PCB (coupling around the filter, inductor self-resonance): the requirement is on the measured antenna-port level (VI-RF-04) |

### 7.3 Filter Component Values

Analysed as written by `software/simulation/cdr/rf/desense_emc.py` (ABCD matrices; SQSO-LBA-001 §7.2). The values are
not a textbook 0.5 dB Chebyshev at 160 MHz but give the response in §7.2; a 0.1 dB Chebyshev at 165 MHz gives a similar
result (−48.9 dB at 2f, −76.8 dB at 3f) and is an alternative. Air-core inductors self-resonate above 1 GHz at these
values; check the capacitors' series resonance against 437 MHz.

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

The applicable rule is 47 CFR 97.307(e) (30–225 MHz): for a transmitter of 25 W or less, the mean power of any spurious
emission supplied to the antenna line must not exceed 25 µW and must be at least 40 dB below the fundamental. At 1 W
(+30 dBm) 25 µW (−16 dBm) is the binding limit: **≥ 46 dBc**. (The "−43 dBc" figure is the 97.307(d) value for
transmitters below 30 MHz and does not apply.)

| Emission | Limit (97.307(e) at 1 W) | Design target |
|----------|--------------------------|---------------|
| 2nd harmonic (291.6–292.0 MHz) | ≤ −46 dBc | ≤ −55 dBc |
| 3rd harmonic (437.4–438.0 MHz, in the uplink segment) | ≤ −46 dBc | **≤ −70 dBc** (desense; SQSO-LBA-001 §7.3) |
| Other spurious | ≤ −46 dBc | ≤ −60 dBc |
| Phase noise of the fundamental | — | ≤ −118 dBc/Hz at 100 kHz, ≤ −135 dBc/Hz at 1 MHz (the desense analysis assumes these) |

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
| Output power | 27.0 ± 0.5 dBm and 30.0 ± 0.5 dBm at the antenna port (VI-RF-02) |
| Frequency accuracy | ±1.0 ppm over temperature (VI-RF-14) |
| Harmonics and spurious | §12.1 (VI-RF-04) |
| Occupied bandwidth | ≤ 10.4 kHz beacon, ≤ 16 kHz transponder (VI-RF-15) |
| Full-duplex desense | ≤ 0.5 dB at the uplink frequency, 1 W keyed (VI-RF-06) |
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
| 1.3 | 2026-10-27 | CDR review (communications panel): requirements to BASELINE §7 (no 2 W level), deviation ±3.0 kHz (the filed 10K4F2D), stability ±1.0 ppm, spurious per 97.307(e) (≥ 46 dBc at 1 W, not −43 dBc), 3rd harmonic ≤ −70 dBc and phase-noise targets from the desense analysis, LPF analysed as written; tests tied to the verification items of SQSO-LBA-001 §8 and RF_DESIGN §6 |
