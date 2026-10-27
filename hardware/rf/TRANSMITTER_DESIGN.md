# RF Transmitter Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-TX-001
**Version:** 1.7
**Date:** 2026-10-27
**Status:** CDR Baseline

---

## 1. Overview

This document defines the 2 m transmitter of the SMART-QSO 1U CubeSat. It carries the AX.25 beacon (through the KISS TNC) and the FM transponder downlink (through the repeat path), time-shared (`docs/hardware/RF_DESIGN.md` v2.5 §3). Values follow BASELINE §7; the link budget is SQSO-LBA-001 v3.0. Version 1.3 (CDR) removes the 2 W level, sets the deviation, stability and harmonic requirements, and records the filter analysis. Version 1.4 records the selected design: the build-to schematic, parts and calculations are in `hardware/electrical/rf/RF_BOARD_DESIGN.md` (SQSO-RF-BRD-001), which governs where the two differ.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Frequency | One IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz | BASELINE §7 |
| Modulation | FM: AFSK (Bell 202) beacon, ±3.0 kHz peak (10K4F2D); FM voice repeat, ±5.0 kHz peak with limiter (16K0F3E) | ITU_APPENDIX4_DATA §C |
| Data Rate | 1200 bps | AX.25 Standard |
| Output Power | 0.5 W nominal, 1 W maximum, at the antenna port | BASELINE §7 |
| Harmonics and spurious | ≤ 25 µW and ≤ −46 dBc at 1 W (47 CFR 97.307(e)); **3rd harmonic ≤ −70 dBc at the antenna port** (proposed; BASELINE target −60 dBc; SQSO-LBA-001 §7.3) | 97.307(e); desense |
| Frequency stability | ±1.0 ppm over temperature (verified −40 to +70 °C) and ≤ ±3.0 ppm with 2-year ageing, ground uplink correction (RF-P005 as amended, RFB-05) | SQSO-LBA-001 §4.4; RF_DESIGN 2.4 |
| Duty | Beacon ≈ 1.2 s per 60 s; transponder ≤ 25 % (software); continuous key-down ≤ 120 s (hardware timer) | BASELINE §5 |

---

## 2. Transmitter Architecture

### 2.1 Block Diagram

```
Audio (TNC AFSK or repeat) → PCM1808 ADC → I2S → LMX2571 (PLL + VCO, FSK word = deviation) → 3 dB pad
   → RD01MUS3 2-stage PA (5 V, APC on gate bias) → 2-section match → elliptic LPF → sense line → balun → 2 m pair
                                     ↑
                      TCXO 40 MHz ±0.5 ppm (shared with the receiver)
```

### 2.2 Architecture Selection

| Approach | Selected | Rationale |
|----------|----------|-----------|
| Single-chip transceiver or module (DRA818V/SA818-V class) | No (1.4) | Consumer modules; phase noise and harmonics not specified; no AX5043-class part remains in production |
| Synthesizer + audio ADC + discrete PA | **Yes (1.4)** | LMX2571 phase noise keeps full-duplex desense below 0.01 dB; deviation set digitally (exact, hard-limited); PA efficiency and harmonics under the design's control |
| SDR approach | No | Power consumption |

---

## 3. Exciter and PA Selection (1.4)

### 3.1 Selected parts

| Function | Part | Key data |
|----------|------|----------|
| Synthesizer | TI LMX2571NJKR | 10–1344 MHz, −145 dBc/Hz at 1 MHz at 480 MHz (≈ −152 at 146 MHz), FSK I2S mode for analog FM, 0 dBm output, 39–51 mA, −40…+85 °C; supply switched by the TX permit, CE = PTT_EXCITER |
| Audio ADC | TI PCM1808PWR | Master 512 fs from 40 MHz (fs 78.125 kHz), I2S; full scale 3.0 Vpp = ±6.3 kHz |
| PA | Mitsubishi RD01MUS3-T614 | 2-stage Si MOSFET; 175 MHz: 1.0 W / 60 % at 7.2 V, 0.60 W / 58 % at 3.6 V, 0 dBm drive; here 5.0 V, 7.5 Ω load line |
| Power detector | Analog Devices AD8361ARMZ (×2) | 7.5 V/Vrms; TI LMH2121 alternate |
| Reference | Epson TG2016SMN 40 MHz | ±0.5 ppm, −40…+85 °C |

### 3.2 Earlier candidates

The module options of Version 1.3 (Radiometrix TX2M, DRA818V) and the ADF7021/Si4463 synthesizers are not used: the
modules are consumer parts without phase-noise or harmonic data, and the desense analysis (SQSO-LBA-001 §7.3) needs a
synthesizer at least as quiet as its assumption; the LMX2571 is 17 dB quieter at 1 MHz.

## 4. Frequency Synthesizer

### 4.1 Reference Oscillator

| Parameter | Value |
|-----------|-------|
| Type | TCXO, clipped sine (Epson TG2016SMN) |
| Frequency | **40 MHz** (1.4: the CC1200 receiver needs 38.4–40 MHz; 40 MHz puts no PCM1808 clock harmonic inside 435.050–437.300 MHz) |
| Stability | **±1.0 ppm** over temperature (verified −40 to +70 °C; ≤ ±3.0 ppm with ageing, RF-P005 as amended), shared by the 2 m exciter and the 70 cm receiver (RF-P005; ±2.5 ppm would give ±1.1 kHz at 435 MHz alone) |
| Phase Noise | < -110 dBc/Hz @ 10kHz |
| Aging | ±1 ppm/year |

### 4.2 PLL Configuration

| Parameter | Value |
|-----------|-------|
| Loop Bandwidth | 40 kHz (1.4: the FSK word is applied at the N divider, so the loop must pass the audio flat) |
| Phase Detector Freq | 40 MHz (fractional-N, DEN 13,002,778) |
| Lock Time | ≈ 0.1 ms after CE; the PA is enabled only with lock detect |
| Frequency Resolution | 0.19 Hz |

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

### 5.3 Modulator Implementation (1.4)

- The PCM1808's samples are written into the LMX2571's FSK deviation word over I2S: 0.1923 Hz per LSB, ±6.3 kHz at
  full scale. The ADC's clipping is a hard deviation limit.
- Beacon: the TNC's DAC sine (1.43 Vpp) gives ±3.0 kHz (2.86–3.14 kHz over supply and resistor tolerances).
- Repeat: the CMX138A's maximum output gives ±4.68 kHz (4.47–4.90 kHz); no pre-emphasis is added (the uplink's is
  repeated). Audio selection by the 2:1 switch, SEL = TNC_PTT.

---

## 6. Power Amplifier

### 6.1 PA Specifications

| Parameter | Value |
|-----------|-------|
| Output Power | 0.5 W nominal (27 dBm) |
| Power Range | 0.5 W / 1 W selectable; no 2 W level (BASELINE §7) |
| Gain | 15-20 dB |
| Efficiency | Allocation ≤ 1.85 W DC at 0.5 W RF including the exciter (BASELINE §5; was ≤ 1.5 W, ≥ 33 %). **Design: 27 % (1.84 W DC, Version E of the board: LMX2571 on its own LDO)**, about 40 % at the 1 W setting (2.42 W) (SQSO-RF-BRD-001 §1, §7.3) |
| Supply | Through the RF_TX_EN load switch (SQSO-ICD-003 §4.4) |
| Current | Set by the selected PA |

### 6.2 Power Control

| Setting | Power | Current |
|---------|-------|---------|
| Low (TX_PWR_SEL low) | 0.5 W (27 dBm) | 1.84 W DC (0.37 A at 5 V) |
| High (TX_PWR_SEL high) | 1.0 W (30 dBm); 29.5–30.1 dBm over the 5 V ± 3 % rail | 2.42 W DC (0.48 A) |

Power is held by a closed loop: an integrator compares TX_FWD with 2.500 V (1 W) or 1.767 V (0.5 W) from a ±0.05 %
reference and drives the common gate bias; it ramps the PA in ≈ 1 ms.

### 6.3 PA Protection

- Junction 88 °C at a 60 °C board (limit 120 °C); the software holds the downlink above 70 °C on TX_TEMP
- VSWR protection: software fold-back (3:1) on TX_REV; the APC regulates forward power
- PA enable = PTT_EXCITER · synthesizer lock; PA supply only through SW3 (RF_TX_EN)

---

## 7. Output Matching and Filtering

### 7.1 Output Match

| Parameter | Value |
|-----------|-------|
| Impedance | 50Ω |
| Return Loss | > 15 dB |
| Bandwidth | 144-148 MHz |

### 7.2 Low-Pass Filter Design (1.4)

Version 1.3's 7-element ladder (L 47, 82, 82, 47 nH; C 33, 47, 33 pF) is replaced by an elliptic filter with traps at
the harmonics (SQSO-RF-BRD-001 §7.4; `hardware/electrical/rf/rf_calcs.py`):

| Parameter | Value |
|-----------|-------|
| Type | 7th-order elliptic: C – (L ∥ C trap at 3f) – C – (L ∥ C trap at 2f) – C – (L ∥ C trap at 3f) – C |
| Cutoff (−3 dB) | 171 MHz |
| Insertion loss at 145.8–146.0 MHz | 0.36 dB nominal, 0.60 dB worst |
| 2nd harmonic rejection | ≥ 65.5 dB (worst of 3,000 Monte Carlo trials) |
| 3rd harmonic rejection | ≥ 88.2 dB (worst of 3,000); with the PA match ≥ 117 dB computed |
| Realization | A −90 dBc coupling floor around the filter is held by shield and layout rules; measured at the port with the antennas deployed (VI-RF-04, VI-RFB-06) |

### 7.3 Filter Component Values

| Element | Value | Type |
|---------|-------|------|
| C614 | 20 pF | C0G 100 V, ±2 % |
| L607 ∥ C615 | 56 nH ∥ 2.4 pF | Coilcraft 1812SMS air core ±2 % ∥ C0G ±0.1 pF |
| C616 | 27 pF | C0G |
| L608 ∥ C617 | 27 nH ∥ 11 pF | air core ∥ C0G |
| C618 | 27 pF | C0G |
| L609 ∥ C619 | 56 nH ∥ 2.4 pF | air core ∥ C0G |
| C620 | 24 pF | C0G |

Monte Carlo model: L ±2 %, C ±2 % or ±0.1 pF, inductor Q 80–150 at 146 MHz, SRF 1.0–2.0 GHz, capacitor ESL 0.6 nH.

## 8. RF Power Detection

### 8.1 Forward Power Monitor

| Parameter | Value |
|-----------|-------|
| Method | Printed sense line after the LPF (45 mm, 0.15 mm gap) + RMS detector (1.4) |
| Coupling | −25 dB (estimate), select-at-test pad 0–4 dB |
| Detector | AD8361 (TI LMH2121 alternate) |
| Output | 2.5 V at 1 W, P = 0.16 V² W (firmware `adc.rs`) |
| Accuracy | ±0.5 dB after calibration (VI-RFB-09) |

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
| TX_TEMP | ADC | Temperature monitor (`ADC_CHANNEL_TX_TEMP`): TMP235 on the PA pad, 10 mV/°C, 0.5 V at 0 °C |
| TONE_OK, XPDR_KEY, PTT_EXCITER | Readbacks to OBC inputs | W4 J1-8/9/10, 3.1 V CMOS through 1 kΩ, active high; STM32 pins allocated by the OBC designer (RID RF-21) |
| TX_FWD | ADC | Forward power (`ADC_CHANNEL_TX_FWD`) |
| TX_REV | ADC | Reflected power (`ADC_CHANNEL_TX_REV`) |
| XPDR_EN | GPIO (timer-driven) | FM transponder enable: a 100 Hz heartbeat (not a level) on PE4; while the edges run, the repeat path keys the PA on a CTCSS-valid uplink carrier (`GPIO_PIN_XPDR_EN`; the TNC's frames take the transmitter first). Driven by `sqso_fsw::transponder` through `hal_gpio_heartbeat()` |

As built (2026-10-25, CR-2026-005): the OBC software is `software/flight/include/downlink.h`. It hands a frame to the
TNC only while transmission is permitted, one frame at a time, at most 5 s on the air and 25 % of the time; selects
TX_PWR; and acts on the monitors as §8.2 and §6.3 ask: VSWR above 3:1 reduces the power to 0.5 W, then stops the
downlink for 10 minutes; above 70 °C it holds the downlink until 60 °C. The transmitter and the TNC are now selected
in the RF board design SQSO-RF-BRD-001 F (LMX2571 and PCM1808 exciter, RD01MUS3 PA, STM32L431 TNC with its own
firmware item, §3.1); the values stay under the existing review item TBR-SRS-003 of `docs/TBD_REGISTER.md` until
they are confirmed on the RF board EM (VI-RFB-01, VI-RFB-04).

XPDR_EN (2026-10-26, from the mission simulation, `docs/test/SIL_MISSION_SIMULATION.md`): the interface had no
way for the OBC to gate the transponder, so the power closure's transponder line (BASELINE §5: sunlit, SOC ≥ 55 %,
duty ≤ 25 %) could not be enforced. The OBC now runs the heartbeat only while transmission is permitted, the mode is
ACTIVE, the array is sunlit, the SOC is at least 55 %, the PA is below 70 °C (resuming at 60 °C) and the keyed time,
measured on TX_FWD, is under 25 % of the last 100 minutes.

**XPDR_EN as implemented (1.4).** In the flight software XPDR_EN is not a level: it is a **100 Hz square wave on PE4**,
toggled by the **TIM7 interrupt** while a **60 ms software lease** lasts; the main loop renews the lease on each pass
while `sqso_fsw::transponder` permits, and the pin is driven low when the lease runs out
(`software/rust/firmware/src/heartbeat.rs`). On the radio board the line has a **100 kΩ pull-down** and drives a
**retriggerable monostable of about 50 ms** (SN74LVC1G123, 226 kΩ × 0.22 µF = 49.7 ms, 46.8–58.0 ms) triggered on rising
edges. **A level held high, like a level held low or an open pin, has no edges and counts as disabled.** The
transponder unkeys within 58 ms of the last edge, and within 118 ms (lease plus monostable) of a stalled main loop. The
monostable output EN_OK is one input of the hardware key gate XPDR_KEY = EN_OK · TONE_OK · NOT_TOT (SQSO-RF-BRD-001 §8).

### 10.2 Timing

| Parameter | Value |
|-----------|-------|
| TX enable to RF | < 10 ms |
| RF to TX disable | < 1 ms |
| Power settle time | < 5 ms |

---

## 11. EMC Design

### 11.1 Shielding

- Three soldered shield cavities: receive, transmit, digital/audio (SQSO-RF-BRD-001 §10)
- Nickel-silver or copper-alloy shields (no tin plating, no steel near the magnetometer)
- Board isolation ≥ 85 dB at 146 MHz from the PA output to the LNA input

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

- Match + elliptic LPF: 2nd −89 dBc, 3rd −90 dBc at the port with the −90 dBc board-leakage floor (SQSO-RF-BRD-001 §7.4)
- Shielded enclosure
- Careful power supply filtering
- Proper grounding

---

## 13. Antenna Interface

### 13.1 Connector

| Parameter | Value |
|-----------|-------|
| Type | MMCX female jack, Molex 73415-1473, straight through-hole on the RF board's top side, brass/BeCu gold, non-magnetic; two (J3, J4: balun outputs A/B, at (−34.5, −19.0) and (−34.5, −25.0) mm; SQSO-RF-BRD-001 C §4.3) |
| Impedance | 50Ω |
| Frequency Range | DC - 6 GHz |
| Power Rating | 1 W at 146 MHz used (0.5 W per jack); 170 Vrms |

### 13.2 Cable

| Parameter | Value |
|-----------|-------|
| Type | 1.37 mm micro-coax (silver-plated copper inner, FEP/PFA), Molex 73414-0351 right-angle MMCX plugs at both ends; two cables (antiphase pair through the board's lattice balun), matched within 2 mm, down the −X wall gap at y −20 … −24 mm (W10 v3) |
| Length | ≤ 0.22 m budgeted, 109–179 mm needed (layout B: RF board at +Z, AntS at −Z) |
| Loss | 0.44 dB worst at 146 MHz, port to antenna (cable 0.17, plugs 0.08, in-phase mismatch 0.19 dB) |

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
| 1.4 | 2026-10-27 | CDR design completion, RF board | Selected design (SQSO-RF-BRD-001): LMX2571 + PCM1808 exciter, RD01MUS3 PA with APC, 40 MHz reference, elliptic LPF with harmonic traps (replaces the §7.3 ladder), sense-line detectors; §10.1 XPDR_EN as implemented (100 Hz square wave on PE4 from TIM7 with a 60 ms lease; board monostable ≈ 50 ms with pull-down; a held level is disabled); DC power 1.76 W at 0.5 W RF recorded against the 1.5 W allocation |
| 1.5 | 2026-10-27 | CDR design completion, RF board | §13 antenna interface to SQSO-RF-BRD-001 C §4.3 and W10 v3: MMCX jacks (Molex 73415-1473) and 1.37 mm cables with MMCX plugs replace SMA and RG-316 (the 3.3 mm wall gaps); loss port to antenna 0.44 dB worst |
| 1.6 | 2026-10-27 | CDR parts temperature audit (SQSO-MFG-003) | Frequency-stability rows to RF-P005 as amended (RFB-05) and the −40 … +70 °C verification range |
| 1.7 | 2026-10-27 | Documentation audit (post-CDR 2026) | §10.1: transmitter and TNC selected in SQSO-RF-BRD-001 F (was "until … selected"); RF_DESIGN cited at 2.5; §6.1–§6.2 DC power to SQSO-RF-BRD-001 Version E (1.84 W at 0.5 W RF, 2.42 W at 1 W; were 1.76 / 2.34 W) and the 1.85 W allocation of BASELINE §5 |
