# SMART-QSO RF Board: Build-to Design

**Document ID**: SQSO-RF-BRD-001
**Version**: C
**Date**: 2026-10-27
**Status**: Build-to design for the engineering model (EM): schematic captured as code with selected, orderable parts,
netlist, BOM, ERC, calculations and layout rules. Not built, not measured. Values follow
[BASELINE_PARAMETERS.md](../../../docs/BASELINE_PARAMETERS.md) Rev E, which governs.

**Design files** (this directory): `design.py` (SKiDL 3.0 schematic as code), `rf_board.net` (netlist, KiCad
format), `rf_board_bom.csv` (BOM), `rf_board_erc.log` (electrical rule check), `rf_calcs.py` with its output
`rf_calcs.json` (every calculated value in this document). Block drawing: `hardware/diagrams/schematics/S4_rf_board.py`
(version 2); harnesses `hardware/diagrams/wiring/W4_rf_board.yml` (version 3) and `W10_rf_coax.yml` (version 3).

**Governing documents**: [RF_DESIGN.md](../../../docs/hardware/RF_DESIGN.md) (SMART-QSO-RF-001 2.1),
[TRANSMITTER_DESIGN.md](../../rf/TRANSMITTER_DESIGN.md) (SQSO-RF-TX-001 1.4),
[RF_ELECTRICAL_DESIGN.md](../../rf/RF_ELECTRICAL_DESIGN.md) (SQSO-RF-003 1.2),
[ANTENNA_DESIGN.md](../../rf/ANTENNA_DESIGN.md) (SQSO-RF-ANT-001 2.1),
[BUILD_VS_BUY.md](../../../docs/hardware/BUILD_VS_BUY.md) §3, [ICD_OBC_RF.md](../../../docs/interfaces/ICD_OBC_RF.md)
(SQSO-ICD-003 2.0), [LINK_BUDGET_ANALYSIS.md](../../../docs/environment/LINK_BUDGET_ANALYSIS.md) (SQSO-LBA-001 3.0),
[EPS_DESIGN.md](../../../docs/hardware/EPS_DESIGN.md) 2.0, [EMC_ANALYSIS.md](../../../docs/environment/EMC_ANALYSIS.md),
[THERMAL_ANALYSIS.md](../../../docs/environment/THERMAL_ANALYSIS.md) v2.0,
[EEE_PARTS_LIST.md](../../../docs/manufacturing/EEE_PARTS_LIST.md) Rev C. Derating: NASA EEE-INST-002 Level 3 as
adapted in EEE_PARTS_LIST Rev C §12 (resistors 50 % power, MLCC 50 % voltage, transistors 50 % power and current,
regulators 70 % current, IC junction ≤ 80 % of maximum; FET RF peak drain voltage ≤ 75 % of V(BR)DSS).

---

## 1. Summary

The board is the spacecraft's amateur station: an always-on 435 MHz receiver, a 145.9 MHz FM transmitter shared by
the AX.25 beacon and the FM repeat path, a KISS TNC on USART2, and the hardware gate behind `XPDR_EN`.

| Item | Design | Key number |
|------|--------|------------|
| Receiver | Balun, FL1 7th-order high-pass with three 146 MHz traps, BFP640F LNA, TA2209A SAW, **TI CC1200** used receive-only in its analog-FM (custom frequency modulation, CFM) mode | System noise 548–598 K with a 290 K source (requirement 650 K, budget 1,000 K); 12 dB SINAD −121.4 dBm, telecommand −115.8 dBm (worst) |
| Antenna ports (C) | Molex 73415-1473 MMCX jacks (non-magnetic) on the top side at the −X edge; 1.37 mm cables with right-angle MMCX plugs down the −X wall gap to the AntS's MMCX ports (assumed) | Port to antenna, worst: 0.44 dB at 146 MHz, 0.62 dB at 436 MHz; return loss ≥ 13.6 dB worst, 19.6 dB typical |
| Receive filter | FL1 before the LNA, SAW after it | 144–146 MHz ≥ 60 dB specified (133 dB computed, worst of 3,000); 250–330 MHz ≥ 49.7 dB; 560–620 MHz ≥ 45.2 dB; loss ahead of the LNA ≤ 1.0 dB |
| TNC | STM32L431 running an AFSK 1200 modem, HDLC and KISS on the CC1200's demodulator samples; USART2 9600 baud, full duplex | Separate firmware item (§13) |
| Exciter | **TI LMX2571** synthesizer, deviation from a **PCM1808** audio ADC over I2S | Phase noise −152 dBc/Hz at 1 MHz; deviation full scale ±6.3 kHz exact; beacon ±3.0 kHz, repeat limit ±4.7 kHz |
| PA | **Mitsubishi RD01MUS3** 2-stage Si MOSFET at 5.0 V, closed-loop power control on the gate bias | 0.50 W at 1.76 W DC; 0.96 W (29.8 dBm) at 2.34 W DC; Tj ≤ 88 °C at a 60 °C board |
| Harmonic filter | 2-section match + 7th-order elliptic low-pass (traps at 2f, 3f) | LPF alone, worst of 3,000: 2f ≥ 65.5 dB, 3f ≥ 88.2 dB; H3 at the port −90 dBc with a −90 dBc board-leakage floor (requirement −70 dBc) |
| Desense | LMX2571 phase noise | ≤ 0.01 dB at 2.2 MHz with H3 at −70 dBc (0.46 dB with the LBA's assumed oscillator) |
| Repeat path, `XPDR_EN` | LVC1G123 monostable (49.7 ms), CMX138A CTCSS decode and audio filter, HC4060 120 s timer, LVC1G11 AND, TMUX1101 mute, TMUX1219 select, TX permit in the PTT gate | Unkeys ≤ 59 ms after the last heartbeat edge; key-down timer 120 ± 4.6 s |
| Power | Receive section on RADIO_3V3 (SW2) | 0.105 W typical, 0.131 W worst (allocation 0.15 W); repeat path +24 mW (allocation 30 mW) |
| Receiver supervision (B) | RX_CD (CC1200 carrier sense) and RX_FAULT (hardware monostable on the TNC's health pulses) to the OBC on J1-15/16 | RX_FAULT asserts 0.36–0.62 s after the last health pulse; unpowered board reads not fault |
| Magnetic cleanliness (B, amended: OBC item C16) | Magnetometer board A2 above (+24, +27): no part within 20 mm, no ferrite or ferromagnetic lid within 40 mm, nickel-silver shield with soldered lids (no clips), PA supply and return paired on L4/L5 and kept ≥ 60 mm away; J1 in the (+X, −Y) corner, J2 by the PA (§10.1) | PA current field at the sensor 0.4 nT at 1 W (22 nT if unpaired or routed past the sensor); RM3100 noise 15 nT |
| Parts | 330 placed, 300 fitted and orderable, 194 nets; 4 mechanical items (shield frame and three lids) | ERC: 0 errors, 0 warnings, 7 waivers |

Three findings change the earlier documents and are reported for the chair (§15):

1. **The AX5043 and every analog FM-IF receiver IC are no longer available.** onsemi's AX5043 data sheet (Rev 4,
   May 2026) is watermarked "DISCONTINUED"; NXP lists the SA605/SA636 family as no longer manufactured; CML's
   CMX994EF draws 39–58 mA. The only in-production receiver found within the 0.15 W allocation that demodulates FM is a
   digital-IF transceiver; the CC1200's CFM output is read by the TNC MCU. The repeat audio therefore passes through
   the TNC firmware (not the OBC flight software); the decisions that make it safe stay in hardware (§8).
2. **Transmitter DC power at 0.5 W RF is 1.76 W, not 1.5 W** (overall 28 %). With a fixed 5.0 V PA rail one load line
   cannot give both 1 W at the port and 1.5 W DC at 0.5 W (trade in §7.3). Orbit-average cost at the 25 % transponder
   duty: +0.065 W.
3. **The 1 W level reaches 29.5–30.1 dBm** across the ±3 % PA rail (29.8 dBm nominal): inside RF-P003's −0.5 dB only
   just, at the low rail.

## 2. Requirements and Compliance

| Requirement (source) | Design value | Margin / status |
|----------------------|--------------|-----------------|
| Receive 435.000–437.300 MHz (BASELINE §7) | CC1200 410–475 MHz band; SAW passband 433–437 MHz guaranteed (8 MHz typical −3 dB) | 437.0–437.3 MHz only on the SAW's typical bandwidth; centre moves −15.7 kHz/K (+861/−548 kHz over −30…+60 °C): **recommend the IARU request ask for 435.050–436.500 MHz** (§6.4) |
| System noise ≤ 650 K with a 290 K source; budget 1,000 K (RF-P006, LBA §2.4) | 548 K typical, 598 K worst at the antenna terminals (cables, connectors and worst-case mismatch included, W10 v3) | +0.7 / +0.4 dB to 650 K; 898–948 K with the 350 K interference allowance: +0.5 / +0.2 dB to 1,000 K |
| Sensitivity: telecommand 90 % frames at 53.0 dB-Hz; voice 12 dB SINAD at 47.4 dB-Hz (RF-P011) | −116.2 / −115.8 dBm telecommand, −121.8 / −121.4 dBm voice (typ / worst, 2 dB implementation loss) | The LBA used 1,000 K; this receiver is 0.2–0.5 dB better |
| RX filter: ≥ 60 dB at 144–146, ≥ 40 dB at 250–330 and 560–620 MHz, ≤ 1.5 dB in band (RF-P007) | FL1 + SAW (§6.2) | 144–146: ≥ 60 dB specified (69 dB without the traps, 133 dB computed with them); 250–330: ≥ 49.7 dB; 560–620: ≥ 45.2 dB; loss before the LNA 1.0 dB worst. The SAW (2.1–4.2 dB) sits after the LNA; RF-P007 is restated in RF_DESIGN 2.1 accordingly |
| Receiver DC ≤ 0.15 W always on (BASELINE §5) | 0.105 W typ, 0.131 W worst | +13 % worst |
| Repeat path ≤ 30 mW while enabled (RF-P015) | 24.4 mW typ, 31.3 mW worst | **−4 % at the worst corner** (CMX138A 8.5 mA figure used as worst) |
| Reference ±1.0 ppm over −40 to +85 °C (RF-P005) | TG2016SMN ±0.5 ppm (−40…+85 °C), calibrated at test to ±0.1 ppm | 0.6 ppm at BOL; **2.1 ppm worst after 2 years with datasheet-class aging** (§6.5) |
| Downlink 145.800–146.000 MHz; 0.5 W nominal, 1 W maximum (BASELINE §7) | LMX2571 4668.8 MHz / 32; APC at 27.0 / 30.0 dBm | 0.5 W: 0.50 W; 1 W: 29.53–30.14 dBm over the 5 V ± 3 % rail |
| Deviation ±3.0 kHz beacon (10K4F2D), ±5.0 kHz voice with limiter (RF-P004) | Digital: ±6.3 kHz = ADC full scale; AFSK 2.86–3.14 kHz; repeat limit 4.47–4.90 kHz | Inside ±0.5 kHz; Carson bandwidth 10.7 kHz at the high AFSK corner (filed 10.4 kHz): measure the 99 % bandwidth (VI-RF-15) |
| Spurious ≤ 25 µW and ≥ 46 dBc at 1 W (97.307(e)); 3rd harmonic ≥ 70 dBc at the antenna port (BASELINE §7) | §7.4 | 2nd −89 dBc, 3rd −90 dBc, 4th −90 dBc with the board-leakage floor; +20 dB to 70 dBc |
| Desense ≤ 0.5 dB at 1 W (RF-P009) | §7.5 | ≤ 0.01 dB at 2.2 MHz; 0.44 dB even 100 kHz from 3f at 70 dBc |
| PA efficiency ≈ 1.5 W DC at 0.5 W RF (BASELINE §5) | 1.76 W | **+0.26 W (17 %) over; finding 2** |
| TX_FWD, TX_REV, TX_TEMP to the OBC ADC (ICD-003 §4.2) | 2.5 V at 1 W (P = 0.16 V² W, `adc.rs`); 10 mV/°C, 0.5 V at 0 °C | Scalings of the firmware unchanged |
| KISS TNC on USART2 9600, AFSK 1200, HDLC, full duplex (ICD-003 §5) | STM32L431 firmware | §5.3; firmware is a new item |
| `XPDR_EN` heartbeat, monostable ≈ 50 ms, CTCSS, 120 s timer, AND, 2:1 select, mute, readbacks (BUILD_VS_BUY §3.2) | §8 | All in hardware except the TONE_OK relay (§8.3) |
| Hardware TX permit gates exciter and PA (brief) | §8.5 | Two independent hardware paths |
| ACS ≥ 60 dB at ±25 kHz (RF-P010) | CC1200 data sheet: 54–60 dB at ±12.5 kHz in 12.5 kHz channels | Not demonstrated at ±25 kHz; verification item VI-RFB-03 |
| CTCSS key within 250 ms (RF-P012) | CMX138A 190 ms typ, 250 ms max, + ≤ 2 ms relay | 2 ms over at the specification limit; RF-P012 restated as ≤ 200 ms typical, ≤ 260 ms maximum |

## 3. Architecture and Part Selection

### 3.1 Signal flow

```
 AntS 70 cm +Y/-Y ─ J5/J6 ─ lattice balun ─ FL1 HPF+traps ─ BFP640F ─ TA2209A SAW ─ balun ─ CC1200 (RX, CFM)
                                                                                              │ SPI 8 MHz, 12.5 kS/s
 TCXO 40 MHz ±0.5 ppm ── CC1200 EXT_XOSC ;  ── squarer (PA domain) ── LMX2571 OSCin, PCM1808 SCKI
                                                                                              ▼
 OBC USART2 ⇄ J1 ⇄ STM32L431 TNC: AFSK demod → HDLC/FCS → KISS ;  KISS → HDLC → AFSK (DAC1) → TNC_AF
                                  repeat audio (DAC2) → 4 kHz LPF → CMX138A (CTCSS, 300–3000 Hz, limit) → AUDIO
 XPDR_EN (J1-6) → LVC1G123 → EN_OK ┐
 CMX138A tone status → TONE_OK ────┼─ LVC1G11 → XPDR_KEY ─ OR TNC_PTT ─ AND TX_PERMIT_HW → PTT_EXCITER
 HC4060 120 s (reset by ¬TONE_OK) ─┘          │                                               │
 AUDIO ─ TMUX1101 mute (XPDR_KEY) ─ TMUX1219 (SEL = TNC_PTT) ─ PCM1808 ─ I2S ─ LMX2571 (CE) ──┘
 LMX2571 RFoutTx ─ 3 dB pad ─ RD01MUS3 (5 V, APC) ─ match ─ elliptic LPF ─ sense line ─ lattice balun ─ J3/J4 ─ AntS 2 m ±X
                                                           AD8361 ×2 → TX_FWD, TX_REV ; TMP235 → TX_TEMP
```

### 3.2 Why these parts

| Function | Selected | Rejected (reason) |
|----------|----------|-------------------|
| Receiver | **TI CC1200RHBR** (Active, NiPdAu, −40…+85 °C; CFM "analog FM" soft output, 8-bit two's complement at the programmed symbol rate; 19 mA in low-power receive) | onsemi AX5043 (discontinued, Rev 4 data sheet May 2026); NXP SA605/SA636 FM IF (no longer manufactured); Motorola MC3372 (aftermarket only, IF to 100 MHz, needs a second mixer and quadrature coils); CML CMX994EF (39–58 mA, I/Q only); NiceRF SA818S module (60 mA, −30…+70 °C, consumer) |
| LNA | Infineon BFP640F at 6 mA (NFmin 0.55 dB at 450 MHz) | MMIC LNAs at 435 MHz draw 20–90 mA (SPF5189Z, PSA4-5043+) |
| SAW | Tai-Saw TA2209A (433–437 MHz, ≥ 45 dB at 10–385 and 485–1000 MHz) | Akoustis SF2446E (23 MHz bandwidth, supplier risk) |
| TNC | STM32L431CCT6 (same family and toolchain as the OBC: the `sqso-protocol` KISS/AX.25 code is `no_std`) | Hardware Bell-202 modem (CMX867A class) plus an MCU: two parts, and software correlator demodulators match the LBA §3 thresholds better |
| CTCSS / audio | CML CMX138AE1 (Active, −40…+85 °C), configured by the TNC MCU | Configured by the OBC: five more harness wires and an OBC driver for no safety gain, since the CMX138A reports tones only through its status register and IRQN (data sheet §7.5.1), so a processor relays TONE_OK either way; the TNC MCU sits next to it and scrubs its registers. Goertzel on the OBC (rejected in BUILD_VS_BUY) |
| Exciter | TI LMX2571NJKR (Active; −145 dBc/Hz at 1 MHz at 480 MHz; FSK I2S mode for analog FM) with TI PCM1808 (Active, pin-configured, −40…+85 °C) | AX5043 as exciter (discontinued; −115 dBc/Hz at 1 MHz would cost 3 dB desense at 80 dBc); discrete VCO and loop (more parts, deviation drifts with Kv) |
| PA | Mitsubishi RD01MUS3-T614 (data sheet Jan 2026; 175 MHz: 1.0 W/60 % at 7.2 V, 0.60 W/58 % at 3.6 V, 0 dBm drive) | NXP AFT05MS003N (discontinued); Qorvo RF5110G (discontinued); QPA9510 (220 mA quiescent, 3.6 V) |
| Detectors | Analog Devices AD8361 (MSOP-8); alternate TI LMH2121 (Active, DSBGA) | Diode detectors (temperature drift > ±0.5 dB) |
| Reference | Epson TG2016SMN 40 MHz, ±0.5 ppm, −40…+85 °C, 1.5 mA | 26 MHz (PCM1808 bit clock harmonic on 435.5 MHz); 38.4 MHz (436.8 MHz); MEMS TCXOs (20–30 mA) |

## 4. Interfaces

### 4.1 J1, OBC signals (W4 version 3, Samtec FTSH-108-01-L-DV-K, 16 positions)

Placement (OBC item C16): top side, in the board's (+X, −Y) corner at the +X edge, pin field along Y centred at about
(+41.0, −33.5) mm (y −39.3 … −27.7), clear of the Ø 7 mm keep-out of the (+40, −43) column, so the W4A ribbon meets
the OBC's J2 through the +X wall gap at the same corner (OBC_BOARD_DESIGN §21). The shield frame is notched at that
corner (x > +37.0, y < −26.0) to make room (§10). J1 is 63 mm from the magnetometer.

| J1 pin | Signal | Direction | Level, RF-board circuit | OBC side |
|--------|--------|-----------|-------------------------|----------|
| 1, 4, 7 | GND | — | — | — |
| 2 | TNC_RXD (OBC USART2 TX) | OBC → RF | 3.3 V CMOS, 1 kΩ series into the MCU (5 V tolerant pin) | PD5 |
| 3 | TNC_TXD (to OBC USART2 RX) | RF → OBC | 3.1 V CMOS push-pull, 100 Ω series | PD6 |
| 5 | TX_PWR_SEL | OBC → RF | 3.3 V CMOS; 1 kΩ series, 100 kΩ pull-down (0.5 W at OBC reset) | PE3 |
| 6 | XPDR_EN | OBC → RF | 100 Hz square wave; 1 kΩ series, 100 kΩ pull-down; edges trigger U401 | PE4 (TIM7 heartbeat) |
| 8 | TONE_OK | RF → OBC | **3.1 V CMOS push-pull through 1 kΩ, high = CTCSS 67.0 Hz decoded**; low with the TNC in reset (100 kΩ pull-down) | OBC input, pin to be allocated by the OBC designer |
| 9 | XPDR_KEY | RF → OBC | **3.1 V CMOS push-pull through 1 kΩ, high = repeat path keyed** (EN_OK · TONE_OK · NOT_TOT) | OBC input, pin to be allocated |
| 10 | PTT_EXCITER | RF → OBC | **3.1 V CMOS push-pull through 1 kΩ, high = transmitter keyed** (beacon or repeat, TX permitted) | OBC input, pin to be allocated; a timer input-capture pin lets the firmware measure keyed time |
| 11 | TX_TEMP | RF → OBC | 0.3–1.7 V (10 mV/°C, 0.5 V at 0 °C), 1 kΩ / 100 nF | PA0 |
| 12 | TX_FWD | RF → OBC | 0–2.5 V, 2.5 V at 1 W, P = 0.16 V² W; 1 kΩ / 100 nF | PC0 |
| 13 | TX_REV | RF → OBC | as TX_FWD; 1.25 V at 3:1 and 1 W | PC1 |
| 14 | AGND | — | Monitor return | — |
| 15 | **RX_CD** | RF → OBC | **3.1 V CMOS push-pull through 1 kΩ, high = carrier above the receiver's carrier-sense threshold** (CC1200 GPIO2 = CARRIER_SENSE, RSSI threshold AGC_CS_THR set 6 dB above the measured noise floor, buffered by U480); 100 kΩ pull-down. Not the CTCSS decision: any carrier, including terrestrial interference | OBC input, pin to be allocated by the OBC designer (RID RFB-07) |
| 16 | **RX_FAULT** | RF → OBC | **3.1 V CMOS push-pull through 1 kΩ, high = receiver fault** (§8.6); 100 kΩ pull-down: an unpowered board or an unwired line reads not fault | OBC input, pin to be allocated by the OBC designer (RID RFB-07) |

The readback outputs are V_DIG logic (3.10 V ± 1.5 %): VOH ≥ 2.9 V at the 1 kΩ, so an STM32 input (VIH 0.7 × VDD =
2.31 V at 3.3 V) reads them with 0.6 V margin. They are valid whenever RADIO_3V3 is on; with the radio unpowered they
read low through the board's 100 kΩ pull-downs (J1-15/16) and the OBC's own pull-downs. The OBC inputs must use pull-downs, not pull-ups: a pull-up would turn an unwired RX_FAULT into a false fault and would push current into the unpowered board. The LVC outputs (Ioff) do not conduct when unpowered, so nothing back-powers the OBC. The firmware's 1 kΩ series makes an OBC input
configured by mistake as an output safe (3.3 mA).

### 4.2 J2, power (W4B, Molex Pico-Clasp 6)

Placement (C16): top side on the −Y edge strip outside the shield, at about (−26, −44) mm, next to the PA in the
(−X, −Y) corner, so the PA's 0.47 A runs from J2 to Q601 in about 16 mm and stays 80 mm or more from the magnetometer
(§10.1). PA_5V (pins 3, 4) and its returns (pins 2, 5) are adjacent contacts; the W4B harness twists each PA_5V wire
with a GND wire.

| J2 pin | Rail | Source (EPS_DESIGN 2.0 §4) | Load on this board |
|--------|------|----------------------------|--------------------|
| 1 | RADIO_3V3 | 3V3_AO through SW2 (RADIO_EN) | 31 mA typ / 38 mA worst receiving; +7–9 mA repeat path; +44–51 mA while keyed (LMX2571) |
| 3, 4 | PA_5V | 5V_PA buck through SW3 (RF_TX_EN, hardware TX-OFF) | 1.3 mA idle while permitted; 0.352 A at 0.5 W; 0.469 A at 1 W |
| 2, 5, 6 | GND | — | — |

EPS's table gives SW2 0.1 A peak and SW3 0.54 A peak: both cover this board. The 0.25 A trip of SW2 is above any
single-event-latch-up current a part on this board would draw from its normal 38 mA, so it does not detect latch-up:
recommend the EPS designer set the SW2 trip at 0.12 A or the OBC watch the RADIO rail current (§15).

### 4.3 RF connectors and antenna cables (C; W10 version 3)

J3/J4: 2 m, balun outputs A/B to the AntS elements along +X/−X. J5/J6: 70 cm, elements along +Y/−Y. 100 kΩ bleed
on each to ground (deployed tapes charge in the plasma). If ISIS supplies the phasing on the AntS, fit R103/R608
(0 Ω), omit the baluns, and use J3 and J5 only.

**Why not SMA (Version B).** The cables can leave the stack only through the 3.3 mm ±X wall gaps (x ±45.4 … ±48.4;
the ±Y gaps are 0.55 mm), and an SMA end-launch jack with a mated plug and an RG-316 bend is about 20 mm long. Free
space in the gaps ([MECHANICAL_DESIGN.md](../../mechanical/MECHANICAL_DESIGN.md) §4.1, §6.9 and the CAD of
`model.py`):

| Gap band | Occupied by | Free for coax |
|----------|-------------|---------------|
| \|y\| < 20, both walls | heat-spreader flanges (z +29.8 … +32.8) | no |
| \|y\| = 20 … 24, +X wall | Y rod (x +36.75 … +47.25, \|y\| ≤ 23.5, z −38.2 … −27.7) | no |
| **\|y\| = 20 … 24, −X wall** | nothing (Z rod and its clip at \|y\| < 7; isolators, clamp ears and burn boards at \|y\| ≥ 26) | **yes: 4.0 × 3.3 mm, full height** |
| \|y\| = 24 … 34 | the four harness bundles | no (harness design) |
| \|y\| = 34 … 44 | W4A ribbon (+X, −Y); hinge doublers, burn boards, hinge flex | no |

**Selection.** The two −X bands take two cables each, so the cable must be ≤ 1.8 mm. With RG-178 (1.8 mm) two fill
3.6 mm of 4.0 mm, too tight for the gap tolerances. Selected:

| Item | Part | Why |
|------|------|-----|
| Board jacks J3–J6 | **Molex 73415-1473**, MMCX straight jack, through-hole, brass body and BeCu contact, gold, listed **non-magnetic** (distributor listing, Active, read 2026-10-27) | Vertical on the top side: the plug turns the cable toward the gap. MMCX snap-on with 360° rotation, 500 cycles, DC–6 GHz. Legs trimmed to ≤ 1.0 mm below the board (the spreader is 2.0 mm below) |
| Cable | **1.37 mm 50 Ω micro-coax**: silver-plated copper inner conductor (no copper-clad steel: ANT-8), FEP or PFA jacket, outgassing TML ≤ 1 %, CVCM ≤ 0.1 % | Two per band: 2.74 mm of the 4.0 mm band. The MIL-DTL-17 RG-178 and RG-316 have silver-plated copper-covered steel inner conductors (ferromagnetic, ANT-8). Bend radius 7 mm (≥ 5 × OD) |
| Cable plugs, both ends | **Molex 73414-0351**, MMCX right-angle plug, crimp, for 1.37 mm cable, brass, gold | Nickel underplate likely (not stated): ≥ 58 mm from A2 at the board end, ≈ 90 mm at the AntS end (below 1 nT per µA·m², §10.1) |
| AntS ports | **MMCX jacks assumed**: the ISIS antenna brochure lists MMCX female RF ports. The 1U/3U ICD and connector positions are not in the repository | **VI-RFB-13** against the ISIS ICD; if ISIS fits another type (SSMCX or U.FL), only the AntS-end plug changes |

**Positions and envelopes** (stack frame of MECHANICAL_DESIGN §4.1; board top surface z +36.4; for the CAD model):

| Item | Position (x, y) mm | Envelope above the board | Notes |
|------|--------------------|--------------------------|-------|
| J3, J4 (2 m) | (−34.5, −19.0), (−34.5, −25.0) | box x −45.1 … −31.5, y −27.5 … −16.5, z +36.4 … +42.4 (jack and mated plug, **6.0 mm allocation**) | In a notch of the shield frame (x < −30.0, y −28.0 … −16.0) at the transmit cavity's −X wall |
| J5, J6 (70 cm) | (−34.5, +19.0), (−34.5, +25.0) | box x −45.1 … −31.5, y +16.5 … +27.5, z +36.4 … +42.4 | Outside the frame (its +Y wall is at y +14) |
| Cable exit and top bend | plug exit x −39.9, cable axis z +39.4 | R 7 bend to vertical at x −46.9, z +32.4; clears the board edge (x −45.1) by 0.7 mm | Pair side by side, centres y ±21.3 and ±22.7 |
| Wall-gap run | x −45.4 … −48.4, y +20.0 … +24.0 (70 cm pair) and −20.0 … −24.0 (2 m pair) | z +32.4 … −39.2 | Tied to the shear-panel side every 30 mm (harness design) |
| Bottom bend and AntS run | from x −46.9, z −39.2, R 7 inboard to z −46.2 | under the battery end caps (bottom z −44.9, 0.6 mm clear) and over the −Z end frame (inner face −46.9), into the 70 × 70 mm cut-out to the AntS ports (z −48.5) | Last 5–75 mm depends on the AntS port positions (VI-RFB-13) |
| J1 (W4A) | §4.1 | **10.0 mm allocation** for the FTSH-108 header, the FFSD socket and the ribbon bend; 12.0 mm is free to the +Z end frame | Mated height from the Samtec drawing: VI-RFB-12 |
| J2 (W4B) | §4.2 | **8.0 mm allocation** for the Pico-Clasp header, housing and wire bend | Mated height from the Molex drawing: VI-RFB-12 |

**Cable length.** The route needs 109–179 mm, depending on where the AntS ports are. The loss budget carries
0.22 m, and each pair is cut to the measured route, matched within 2 mm.

**Loss and match** (`rf_calcs.py` §16; 1.37 mm attenuation assumed at 2.6 dB/m at 2.4 GHz, scaled by √f, +20 %
at +60 °C, to confirm with the cable data sheet):

| Band | Cable | Two MMCX pairs | Mismatch (VSWR 1.12 per connector, 1.10 per cable end) | Port to antenna | Return loss of the path |
|------|-------|----------------|---------------------------------------------------------|-----------------|-------------------------|
| 146 MHz | 0.17 dB | 0.08 dB | 0.19 dB in phase (0.05 dB RSS) | **0.44 dB** | 13.6 dB worst, 19.6 dB RSS |
| 436 MHz | 0.29 dB | 0.13 dB | 0.19 dB (0.05 dB) | **0.62 dB** | 13.6 dB worst, 19.6 dB RSS |

The link-budget terms:
- **Downlink.** SQSO-LBA-001 §2.4 carries 1.0 dB of filter, cable and connector loss after the transmitter power.
  This board sets its power at its own port, after the LPF, with the APC loop. The LBA term therefore covers only
  port to antenna: 0.44 dB, which leaves 0.56 dB of margin.
- **1 W setting.** It gives 29.1–29.7 dBm at the antenna terminals, against 29.5–30.1 dBm at the port.
- **Uplink.** The receive chain carries 0.62 dB (previously 0.25 dB): T_sys rises from 503–550 K to 548–598 K, still
  inside the 650 K requirement (§6.1).
- **Antenna match.** The path's return loss of ≥ 13.6 dB sits in front of the AntS's own match, which governs (ANT-3).
- **Desense and harmonics.** The 2 m and 70 cm jacks are 38 mm apart and both outside the shield; their cables run
  in different bands. An MMCX's screening (≥ 60 dB class) is far below the 25 dB antenna coupling already in the
  desense budget (§7.5). The H3 at the port is unchanged.

## 5. Power and Domains

### 5.1 Rails

| Rail | Made by | Loads | Why |
|------|---------|-------|-----|
| +3V3_RAD | RADIO_3V3 through FB801 | LDOs, LMX2571 switch | Entry filter |
| +3V0_RX | TPS7A2030 (3.0 V ± 1.5 %, 6.5 µA Iq) | CC1200, LNA, TCXO | EMC rule E4: own LDO, 0.19 V minimum headroom, PSRR ≥ 50 dB at the 20 kHz magnetorquer PWM |
| +3V1_DIG | TPS73101 at 3.106 V (31.6 k/20.0 k) | MCU, CMX138A DVDD, key logic, TMP235 | CMX138A needs 3.0–3.6 V, so the 3.0 V LDO cannot feed it; 3.1 V keeps CC1200 inputs within VDD + 0.3 V |
| +3V1_AUD | +3V1_DIG through FB802, 10 µF | CMX138A AVDD | Audio thresholds track AVDD |
| +5V_PA | PA_5V through FB803 (2.2 A) | PA drains, APC, reference | Unpowered unless SW3 is on |
| +3V3_PA | TPS7A2033 from +5V_PA | PCM1808 VDD, squarer, detectors, PA-domain logic | Unpowered unless SW3 is on |
| +5V_PA_A | +5V_PA through FB804 | PCM1808 VCC | Analog supply; sets the deviation scale (§7.1) |
| +3V3_LMX | TPS22918 from +3V3_RAD, ON from a PA_5V divider | LMX2571, MICROWIRE buffer | LMX2571 needs 3.15–3.45 V (the rail is 3.19–3.40 V at the board); powered only when the PA rail is present |

The minimum RADIO_3V3 at the board is 3.189 V (3V3_AO −3 %, SW2, harness and ferrite drops at 80 mA): 39 mV above
the LMX2571's 3.15 V minimum. That margin depends on the EPS 3V3_AO tolerance (±3 %); a ±2 % set point would double it.

### 5.2 DC power (from `rf_calcs.json`)

| State | RADIO_3V3 | PA_5V | Total |
|-------|-----------|-------|-------|
| RF silence (SW2, SW3 off) | 0 | 0 | 0 |
| Receive, transponder off | 0.105 W typ / 0.131 W worst | 6.3 mW idle (if permitted) | 0.11–0.14 W |
| Receive + repeat path enabled (not keyed) | +24 / +31 mW | — | 0.13–0.16 W |
| Keyed 0.5 W (beacon or transponder) | +0.152 W (LMX2571) | 1.61 W (PA 1.51, PCM1808, detectors, logic) | 1.76 W transmitter + receive |
| Keyed 1 W | +0.152 W | 2.19 W | 2.34 W transmitter + receive |

Breakdown of the receive section (typ / worst mA): CC1200 19 / 21, LNA 6 / 7, MCU 3 / 5, TCXO 1.5 / 2, LMX2571 in
power-down 0.9 / 1.2, LVC1G123 0.28 / 0.4, receiver supervision 0.30 / 0.45, op-amps 0.24 / 0.3, LDO ground currents and pull-ups 0.45 / 0.9, others
0.1 / 0.2. Beacon (ACTIVE, ≈ 1.2 s per 60 s at 1.76 W): 0.035 W orbit average (allocation 0.10 W).

## 6. Receiver

### 6.1 Noise figure and sensitivity (`rf_calcs.py` §1)

| Stage | Loss or NF | Gain |
|-------|------------|------|
| Cable 0.22 m 1.37 mm, two MMCX pairs, worst-case mismatch (W10 v3, `rf_calcs.py` §16) | 0.62 dB | −0.62 dB |
| 70 cm lattice balun | 0.30 dB | −0.30 dB |
| FL1 (worst of the Monte Carlo) | 0.70 dB | −0.70 dB |
| LNA BFP640F (NFmin 0.55 dB + 0.25 dB match) | 0.80 dB | +18 dB |
| SAW TA2209A | 2.1 dB typ / 4.2 dB max | −2.1 / −4.2 dB |
| Balun and match to CC1200 | 0.5 dB | −0.5 dB |
| CC1200 (low-power receive; estimate: TI gives sensitivity, not NF) | 6 / 7 dB | — |

Cascade: NF 2.14 / 2.53 dB at the board connector, 2.76 / 3.15 dB at the antenna terminals; T_e 258 / 308 K;
T_sys 548 / 598 K with Earth (290 K) in view. Sensitivity with 2 dB implementation loss for the CFM demodulator in
the TNC (`impl_loss_db`): 12 dB SINAD voice −121.8 / −121.4 dBm; telecommand 90 % frames −116.2 / −115.8 dBm.
Version C carries the antenna cable at its worst case (in-phase mismatch, +60 °C); with the typical (RSS) mismatch
the cable costs 0.47 dB, and with the measured route of 0.11–0.18 m less still.

### 6.2 Receive filter FL1 (`rf_calcs.py` §2)

7th-order 0.1 dB Chebyshev high-pass, fc 350 MHz, shunt inductor first (DC path for the antenna's static charge), the
first three shunt arms made series-resonant at 146.5, 145.0 and 143.5 MHz with their inductors raised so that each arm
keeps its prototype reactance at 436 MHz:

| Element | Value | Part |
|---------|-------|------|
| L103 + C103 (arm 1, trap 146.5 MHz) | 22 nH + 56 pF | Coilcraft 0603HP ±2 %, Murata GRM18 C0G 100 V |
| C104 (series) | 6.4 pF | ±0.1 pF |
| L104 + C105 (arm 3, trap 145 MHz) | 12 nH + 100 pF | |
| C106 (series) | 5.8 pF | |
| L105 + C107 (arm 5, trap 143.5 MHz) | 12 nH + 100 pF | |
| C108 (series) | 6.4 pF | |
| L106 (shunt) | 20 nH | |

Monte Carlo, 3,000 trials: L ±2 %, C ±2 % (≥ 10 pF) or ±0.1 pF, inductor Q 45–75 at 436 MHz, inductor SRF
2.5–4 GHz, capacitor ESL 0.4 nH. Results: loss 0.52 dB nominal, **0.70 dB worst** at 435.0–437.3 MHz; return loss
≥ 22.1 dB; 144–146 MHz ≥ 133 dB computed (the Chebyshev response alone gives 69.4 dB). A printed filter will not show
133 dB: coupling around it sets the floor, so the specification is **≥ 60 dB at 144–146 MHz, measured**, and the
layout rules of §10 (FL1 in its own shield cavity, input and output on opposite sides of a wall) are what hold it.
250–330 MHz: FL1 ≥ 4.7 dB; 560–620 MHz: FL1 ≥ 0.2 dB. The SAW (≥ 45 dB at 10–385 and 485–1000 MHz) brings the totals
to **≥ 49.7 dB and ≥ 45.2 dB**. Placing the SAW after the LNA is deliberate: its 2.1–4.2 dB ahead of the LNA would
add 140–300 K. The rejection at 250–330 and 560–620 MHz exists to stop second-order products (LBA §7.5); in the LNA two
terrestrial signals at −90 dBm make an IM2 product near −200 dBm, so the post-LNA SAW is where those products are
generated and filtered.

### 6.3 Linearity and isolation (`rf_calcs.py` §12)

The 2 m fundamental at 1 W reaches the 70 cm port at +5 dBm through 25 dB of antenna isolation (ANT-5). After FL1 it
is ≤ −55 dBm at the LNA input by specification (−128 dBm computed), and ≤ −82 dBm at the CC1200 after LNA gain and the
SAW, against the CC1200's +10 dBm saturation and −14 dBm IIP3. Board requirement: **≥ 85 dB isolation at 146 MHz
from the PA output to the LNA input node** (layout §10).

### 6.4 Frequency plan of the board's clocks (`rf_calcs.py` §13)

| Clock | Harmonics in 435.000–437.300 MHz |
|-------|----------------------------------|
| 40 MHz reference (CC1200, LMX2571 PFD, PCM1808 SCKI) | none (400, 440 MHz) |
| PCM1808 bit clock 5.000 MHz (512 fs) | 435.000 MHz (band edge) |
| PCM1808 internal 10 MHz | none |
| CMX138A crystal 3.6864 MHz | none (434.995, 438.682 MHz) |
| MCU 16 MHz (HSI16, core, buses, ADC) | none (432, 448 MHz) |
| SPI1 8 MHz (CC1200, LMX2571) | none (432, 440 MHz) |
| SPI1 1 MHz for the CMX138A (≤ 1 ms per 10 s) | every MHz (436 MHz in band) |
| Rejected: CMX138A 6.144 MHz; 38.4 MHz reference (bit clock 4.8 MHz); 26 MHz reference (bit clock 3.25 MHz) | 436.224; 436.8; 435.5 MHz |

Recommendation for the IARU request (with the SAW's temperature drift, §2): **uplink 435.050–436.500 MHz**, clear of
the 435.000 MHz bit-clock and crystal harmonics and centred in the SAW.

### 6.5 Frequency reference

TG2016SMN: ±0.5 ppm over −40…+85 °C (meets the ±1.0 ppm over temperature with 0.5 ppm margin). Initial offset after
reflow is calibrated at acceptance: the TNC writes the CC1200 FREQOFF and the LMX2571 numerator from stored trims
(residual ±0.1 ppm): 0.6 ppm (261 Hz at 435.5 MHz, 88 Hz at 145.9 MHz) at the beginning of life. Aging is not on the
Epson listing reached; with a typical TCXO class of ≤ 1 ppm in year 1 and ≤ 0.5 ppm in year 2, the worst case after 2
years is **2.1 ppm (915 Hz at 435.5 MHz)**, RSS 1.2 ppm. Because one reference drives both bands, a station that
measures the beacon's offset scales it by 435.5/145.9 and pre-corrects its uplink exactly; and the CC1200 tracks
the residual (programmable AFC). The ±1.0 ppm over two years is therefore met only with a re-trim: option for the TNC
firmware (KISS SetHardware command from the OBC with a ground-measured offset, bounded to ±3 ppm), or ground-side
correction (no flight change). Measured in VI-RF-14.

## 7. Transmitter

### 7.1 Exciter and deviation (`rf_calcs.py` §9)

LMX2571: fPD 40 MHz (MULT 1), prescaler 2, fVCO 4668.8 MHz, channel divider 32 → 145.9 MHz; N = 58.36; loop
bandwidth 40 kHz (Icp 1.25 mA, Kvco 56 MHz/V: R504 430 Ω, C522 39 nF, C521 3.9 nF; ≈ 55° phase margin; to be
confirmed in TI PLLatinum Sim with the exact register set). The loop bandwidth is 13 times the highest audio frequency,
so the FSK-word modulation passes flat. Lock in ≈ 0.1 ms after CE; the PA is enabled only by PTT_EXCITER AND lock
detect (U733), so no emission occurs unlocked; the APC ramps the PA in ≈ 1 ms (no key clicks). TXDELAY 300 ms covers it.

Deviation: PCM1808 master, 512 fs from 40 MHz (fs 78.125 kHz, bit clock 5.000 MHz), I2S into the LMX2571's FSK I2S
port (16 MSBs used). DEN = 13,002,778 gives 0.1923 Hz per LSB, so ADC full scale (±32,767, 0.6 × VCC = 3.0 Vpp at
5.0 V) is ±6.3 kHz. The ADC's clipping is a hard deviation limit no audio can exceed.

| Source | Level | Deviation (nominal; ±4.7 % worst: PA_5V ±3 %, V_DIG ±1.5 %, resistors) |
|--------|-------|-------------------------------------------|
| TNC AFSK (DAC1, 12-bit sine at 50 kS/s) | 1.43 Vpp | ±3.0 kHz (2.86–3.14) |
| Repeat audio limit (CMX138A maximum undistorted output 2.23 Vpp at AVDD 3.1 V) | — | ±4.68 kHz (4.47–4.90); a ±3 kHz uplink repeats at ±3 kHz |

Phase noise at 145.9 MHz (from the data sheet's 480 MHz values, divided, floored): −125 dBc/Hz at 10 kHz, −140 at
100 kHz, −152 at 1 MHz, −157 at 10 MHz. The 2nd harmonic of the LMX2571 output (−25 dBc) and of the PA are removed by
the output filter.

### 7.2 PA

RD01MUS3 at 5.0 V, input from the LMX2571 (0 dBm maximum; FET1 maximum input 2 mW, so 50 % derated) through a 3 dB π
pad and the Mitsubishi 135–175 MHz EVB input network (L601 36 nH, C602 16 pF, C603 51 pF), interstage L603 39 nH /
C607 30 pF, drain feeds 68 nH and 56 nH (Coilcraft 1812SMS air core), gate bias through 4.7 kΩ / 270 Ω and 4.7 kΩ /
560 Ω (EVB values). Output load line 7.5 Ω: two low-pass L sections 7.5 → 19.4 → 50 Ω (Q 1.26 each: La 10 nH, Ca
68 pF, Lb 27 nH, Cb 27 pF), DC block 1 nF. All matching values are starting values to be tuned by load-pull on the EM.

Power control: APC loop U735 integrates (V_ref − TX_FWD) into the common gate bias; V_ref = 2.500 V (REF35250,
±0.05 %) for 1 W, 1.767 V for 0.5 W (R730/R731, switched by U731 from TX_PWR_SEL); U732 grounds V_ref unless PA_EN.
Power error: reference and dividers 0.04 dB, detector temperature ±0.25 dB.

### 7.3 Power, efficiency and the load-line trade (`rf_calcs.py` §6)

Losses from drain to the antenna port: match + LPF 0.74 dB worst, sense line 0.03 dB, balun 0.25 dB, trace and
connector 0.05 dB: 1.06 dB. P_sat = (V_DD − V_knee)²/(2 R_L), V_knee 0.7 V; saturated efficiency 59 % (interpolated
from the data sheet's 58 % at 3.6 V and 60 % at 7.2 V); below saturation efficiency falls as √(P/P_sat) (gate-bias
control, class-B-like).

| Level | Drain power | PA efficiency | PA DC | Exciter DC | Total DC | 5 V current | Overall | Board dissipation |
|-------|-------------|---------------|-------|------------|----------|-------------|---------|-------------------|
| 0.5 W at the port | 0.64 W | 42.6 % | 1.51 W | 0.255 W | **1.76 W** | 0.35 A | 28.4 % | 1.26 W |
| 1 W setting | 1.23 W (saturated) | 59 % | 2.09 W | 0.255 W | **2.34 W** | 0.47 A | 41.1 % | 1.38 W |

1 W setting at the port: 29.84 dBm at 5.00 V, 29.53 dBm at 4.85 V, 30.14 dBm at 5.15 V.

| Load line | P_sat | Maximum at the port | DC at 0.5 W |
|-----------|-------|---------------------|-------------|
| 7.0 Ω | 1.32 W | 30.14 dBm | 1.82 W |
| **7.5 Ω (design)** | 1.23 W | 29.84 dBm | 1.76 W |
| 8.5 Ω | 1.09 W | 29.29 dBm | 1.67 W |
| 10 Ω | 0.92 W | 28.59 dBm | 1.56 W |
| 12 Ω | 0.77 W | 27.80 dBm | 1.45 W |

Only a 12 Ω load line meets 1.5 W at 0.5 W, and it gives 0.6 W maximum. The design keeps the 1 W function. Options for
the chair (§15): accept 1.76 W (orbit average +0.065 W at 25 % transponder duty; ACTIVE tumbling margin about +31 %
→ +28 %); or a drain-switched 0.5 W mode (a 4.0 V buck on the board would give ≈ 1.38 W, at the cost of a switching
converter beside a full-duplex receiver, rejected here for EMC); or a 6 V PA rail (EPS change; 1 W with 1.5 dB margin,
no gain at 0.5 W).

Thermal: FET2 dissipates 0.78 W at 0.5 W (0.9 × 0.87 W). Tj2 = 60 °C (board, the per-unit hot limit) + 0.78 × (15 +
4) °C/W (Rth j-c maximum + case to board) + 0.87 × 15 °C/W (local spreading) = **88 °C** against the vendor's 120 °C
(80 % of 150 °C): margin 32 °C. FET1: 75 °C. LMX2571: 65 °C at a 60 °C board.

### 7.4 Output filter and harmonics (`rf_calcs.py` §3–§4)

Elliptic low-pass C1–(L2‖C2t)–C3–(L4‖C4t)–C5–(L6‖C6t)–C7, 50 Ω, from a 7th-order 0.1 dB Chebyshev at 175 MHz with
the series arms made parallel traps at 437.7 MHz (arms 1, 3) and 291.8 MHz (arm 2), snapped to standard values and
the traps re-tuned to the snapped inductors:

| C614 | L607 ‖ C615 | C616 | L608 ‖ C617 | C618 | L609 ‖ C619 | C620 |
|------|-------------|------|-------------|------|-------------|------|
| 20 pF | 56 nH ‖ 2.4 pF | 27 pF | 27 nH ‖ 11 pF | 27 pF | 56 nH ‖ 2.4 pF | 24 pF |

Inductors Coilcraft 1812SMS air core ±2 % (Q > 100 at 150 MHz, Irms ≥ 2.5 A); capacitors Murata GRM18 C0G 100 V
(±2 %, or ±0.1 pF below 10 pF). Monte Carlo, 3,000 trials: L ±2 %, C ±2 % or ±0.1 pF, inductor Q 80–150 at 146 MHz
(rising as √f), inductor SRF 1.0–2.0 GHz, capacitor Q 800 and ESL 0.6 nH (0603 plus via; 0.3 nH for the trap
capacitors). Cut-off (−3 dB) 171 MHz.

| | Loss at 145.8–146.0 | Return loss | 2f (291.6–292.0) | 3f (437.4–438.0) | Uplink 435.0–437.3 |
|-|---------------------|-------------|------------------|------------------|--------------------|
| LPF alone, nominal | 0.36 dB | 18.3 dB | | | |
| LPF alone, worst of 3,000 | 0.60 dB | | **65.5 dB** | **88.2 dB** | 88.4 dB |
| Match + LPF, worst of 3,000 | 0.74 dB | 13.3 dB (from 7.5 Ω) | 78.5 dB | 117 dB | 117 dB |

Harmonic budget at the port, 1 W: PA harmonics assumed −20 dBc (2nd) and −15 dBc (3rd) (saturated single-ended stage,
worst case: VI-RF-04 measures). After the network the computed levels are below −98 dBc; coupling around the filter
is not in an ABCD model, so a −90 dBc leakage floor is added, which the shield rules of §10 must hold: **2nd −89 dBc,
3rd −90 dBc, 4th −90 dBc**. 97.307(e) at 1 W needs ≤ −46 dBc and ≤ 25 µW: met by 43 dB. 3rd harmonic: −70 dBc
required, **20 dB margin**. Passive intermodulation after the filter (antenna hinges, connector plating) is outside
this board and is what VI-RF-04 with the antennas deployed measures.

### 7.5 Full-duplex desense (`rf_calcs.py` §5)

Desense = 10 log(1 + N_skirt/N₀), N₀ = −168.6 dBm/Hz (1,000 K); skirt = P_TX + H3 − 15 dB (antenna isolation at
437 MHz) + L(f) + 9.5 dB.

| Offset from 3f | LBA model and LBA oscillator, H3 −70 dBc | LBA model, LMX2571, H3 −70 dBc | LMX2571, H3 −90 dBc (this design) |
|----------------|------------------------------------------|--------------------------------|-----------------------------------|
| 0.1 MHz | 13.9 dB | 0.44 dB | 0.005 dB |
| 2.2 MHz (435.5 vs 437.7) | 0.46 dB | 0.008 dB | 0.000 dB |
| 5.0 MHz | 0.17 dB | 0.004 dB | 0.000 dB |

(The LBA model evaluates the phase noise at a third of the offset, which is conservative: a harmonic's noise is the
fundamental's +9.5 dB at the same offset; with that, 2.2 MHz gives 0.004 dB.) RF-P009 (≤ 0.5 dB) is met with more than
20 dB to spare; the LPF's 88 dB at the uplink frequency also removes the fundamental's far-out noise.

### 7.6 Monitors (`rf_calcs.py` §10)

Sense line CPL601 after the LPF (so harmonics do not reach the detectors and no ferrite is in the power path): 45 mm
edge-coupled line, 0.37 mm wide, 0.15 mm gap, on L1 over the 0.2 mm prepreg; coupling −25.3 dB (estimate,
k = 0.22), directivity ≥ 15 dB (to measure). At 1 W, +4.7 dBm at each coupled port; a select-at-test π pad (0–4 dB,
nominal 1.2 dB) sets +3.5 dBm (0.333 Vrms) at the AD8361 for 2.5 V at 1 W (7.5 V/Vrms): **TX_FWD 2.50 V at 1 W,
1.77 V at 0.5 W; TX_REV 1.25 V at 3:1** (P = 0.16 V², the firmware's scale). The detectors are powered down unless
PTT_EXCITER (U734). TX_TEMP: TMP235A2 on the PA ground pad (10 mV/°C, 0.5 V at 0 °C, ±2 °C), always powered, so the
software's 70 °C hold works with the PA off.

## 8. FM Repeat Path and the `XPDR_EN` Gate

### 8.1 Heartbeat and EN_OK

Flight software as implemented (`software/rust/firmware/src/heartbeat.rs`): TIM7 interrupt every 5 ms toggles PE4,
a 100 Hz square wave, while a lease (60 ms, renewed by the main loop each pass while `sqso_fsw::transponder` permits)
lasts; the pin is driven low when the lease runs out. On the board: 1 kΩ series, 100 kΩ pull-down, U401
SN74LVC1G123 triggered on rising edges (B input, A low), tw = Rext × Cext = 226 kΩ × 0.22 µF (C0G) = 49.7 ms
(46.8–58.0 ms with K 1.00–1.10, C ±5 %, R ±1 %): retriggered every 10 ms, so EN_OK stays high with 5× margin. A level
held high or low, a high-impedance pin (pull-down), an OBC in reset or browned out: no edges, EN_OK drops ≤ 58 ms after
the last edge. A stalled main loop: ≤ 60 ms lease + ≤ 58 ms = **≤ 118 ms**. CLR is a 10 ms power-on RC, so EN_OK starts
low. Note: a rising CLR with B already high fires one pulse (≤ 58 ms of EN_OK at power-up); TONE_OK is low then (TNC
starting), so no key results.

### 8.2 CTCSS, audio, key-down timer, gate

| Element | Part | Function |
|---------|------|----------|
| Repeat audio | MCU DAC2 → TLV9002 Sallen-Key 4 kHz → CMX138A DISC (gain 1) | The CC1200's demodulated samples, DC-removed, 12.5 kS/s |
| CTCSS decoder, audio conditioning | CMX138AE1 in Rx mode, 3.6864 MHz crystal (CLKSEL low) | 67.0 Hz decode (response 190 ms typ, 250 ms max; de-response 240 ms typ); 300–3000 Hz band filter strips the tone; de-emphasis off (the uplink's pre-emphasis is repeated as received); the output's maximum level is the repeat deviation limit |
| TONE_OK | MCU PA7, from the CMX138A tone status ($C6 b11, $CC) on IRQN | 100 kΩ pull-down; the MCU's independent watchdog (LSI, 250 ms) returns it to reset (low) |
| Key-down timer | CD74HC4060 RC oscillator 68.27 Hz (Ct 100 nF C0G, Rt 56.2 k + select-at-test, Rs 120 k; f = 1/(2.3 Rt Ct)); Q14 high after 8,192 periods = 120 s; D401 from Q14 stops the oscillator (latched) | MR = NOT TONE_OK (U402): the count restarts whenever the tone drops; NOT_TOT = NOT Q14. Trimmed to 120 ± 1 s at 25 °C, drift ±3 % → 120 ± 4.6 s |
| Key gate | SN74LVC1G11 | XPDR_KEY = EN_OK · TONE_OK · NOT_TOT |
| Mute | TMUX1101 (SEL = XPDR_KEY) | Repeat audio reaches the select only while keyed |
| 2:1 select | TMUX1219 (SEL = TNC_PTT) | TNC audio (S2) when the TNC keys, repeat audio (S1) otherwise; both sources biased at 1.55 V so the switch makes no step |
| PTT | SN74LVC1G32 + SN74LVC1G08 | PTT_EXCITER = TX_PERMIT_HW · (TNC_PTT + XPDR_KEY) |

### 8.3 What software does and does not do

The OBC decides when the heartbeat runs. The TNC firmware demodulates the uplink (AFSK and the repeat audio) and relays
the CMX138A's tone status to TONE_OK. It cannot key the transmitter alone: keying needs EN_OK (the OBC heartbeat), the
hardware timer and, for any emission, the hardware TX permit and the PA rail (RF_TX_EN). A TNC hung with TONE_OK high
keys at most an unmodulated carrier until its watchdog resets it (≤ 0.25 s), the OBC stops the heartbeat, or the 120 s
timer latches. A TNC that outputs garbage audio cannot make the CMX138A decode 67.0 Hz, and whatever it outputs is
band-limited to 3 kHz and level-limited by the CMX138A, then hard-limited by the ADC: the occupied bandwidth of the
repeat path is bounded in hardware.

### 8.4 Truth table

| EN_OK | TONE_OK | NOT_TOT | TNC_PTT | TX_PERMIT_HW | XPDR_KEY | PTT_EXCITER | Audio to the exciter |
|-------|---------|---------|---------|--------------|----------|-------------|----------------------|
| 0 | x | x | 0 | x | 0 | 0 | — |
| 1 | 1 | 1 | 0 | 1 | 1 | 1 | Repeat |
| 1 | 1 | 0 (≥ 120 s) | 0 | 1 | 0 | 0 | — (until the tone drops) |
| x | x | x | 1 | 1 | x | 1 | TNC (frames first) |
| x | x | x | x | 0 | x | 0 | — (no exciter power, no CE, no PA) |

### 8.5 Hardware TX permit

TX-OFF (RF_TX_EN low, `rf_control`), the loss-of-contact and RF-silence inhibits, the three deployment switches and
the EPS low-voltage shed all act by removing PA_5V (SW3; EPS_DESIGN 2.0 §5, §8). The board turns the presence of
PA_5V into two independent hardware enables: (1) divider R803/R804 → U804 Schmitt buffer → TX_PERMIT_HW, an input of
the PTT gate (so the LMX2571 CE, the PCM1808 clock and the PA enable all need it); (2) divider R805/R806 → the ON pin
of U805, which powers the LMX2571 itself. With TX-OFF the PA has no supply, the exciter has no supply and its CE is
low: a single failure (a stuck gate or a shorted switch) leaves at least one of the three removed. No state of the
repeat-path logic can power the PA.

### 8.6 Receiver supervision lines (B, RID RFB-07)

The flight software (SRS-F115 to F117) power-cycles RADIO_EN after 24 h of receiver silence and keeps the beacon when
the receiver is known to have failed. The board gives it two lines:

| Line | Source | Meaning |
|------|--------|---------|
| RX_CD (J1-15) | CC1200 GPIO2 configured CARRIER_SENSE (IOCFG2 = 17), buffered by U480 SN74LVC1G17 | High while the in-channel RSSI exceeds the carrier-sense threshold (AGC_CS_THR, set at test 6 dB above the measured noise floor, about −116 dBm). Independent of the TNC firmware once configured; the TNC re-writes IOCFG2 and the threshold in its scrub. The OBC can count carriers per pass to tell a deaf receiver from an empty channel |
| RX_FAULT (J1-16) | U481 SN74LVC1G123 retriggered by the TNC's RX_ALIVE pulses (PB7, every 100 ms), inverted by U482 SN74LVC1G04 | Low (healthy) while pulses arrive; high 0.36–0.62 s after the last pulse (470 kΩ × 1.0 µF; K 1.00–1.10, X7R −15/+10 %). A hung, resetting or dead TNC produces no pulses, so it reads as a fault without firmware help |

The TNC emits RX_ALIVE only while all of the following hold; any failure stops the pulses (RX_FAULT high):

| # | Criterion | Check |
|---|-----------|-------|
| 1 | CC1200 responds | PARTNUMBER register reads 0x20 and a scratch-register write/read-back matches, every 1 s; 3 consecutive failures = fault |
| 2 | CC1200 is receiving | MARCSTATE = RX (after one automatic re-entry attempt); not RX for > 2 s = fault |
| 3 | Synthesizer locked, reference running | Lock status in RX and XOSC stable; CLKEN_CFM sample interrupts at 12.5 kHz ± 5 % (none for > 100 ms in RX = fault) |
| 4 | Configuration intact | The 10 s scrub re-writes any register mismatch (logged); a mismatch that persists after 3 re-writes = fault |
| 5 | TNC itself healthy | RX_ALIVE is toggled from the main loop, not an interrupt; an independent-watchdog reset stops the pulses for the reset and re-initialisation (about 50 ms) |

RX_FAULT is high for about 0.5 s after RADIO_EN rises (no pulses yet): the OBC ignores it for 1 s after switching
RADIO_EN on. A fault clears when the pulses resume (after the TNC re-initialises the CC1200, or after the OBC's
RADIO_EN power cycle). RX_FAULT says nothing about the antenna, LNA or filter (a deaf front end with a healthy CC1200
reads not fault): RX_CD carrier counts and the 24 h silence rule of SRS-F115 cover those.

## 9. Antenna Interface

ISIS AntS 1U–3U on the −Z face (MASS_BUDGET layout B): two 2 m tape monopoles along +X and −X, two 70 cm monopoles along
+Y and −Y. Each pair is fed in antiphase from a lumped lattice balun on this board, so each pair radiates as a
half-wave dipole across the body (the body carries little net current, so the wings disturb it less than a single
monopole driven against the body):

| Band | Balun (50 Ω unbalanced → 72 Ω balanced: two ≈ 36 Ω monopoles) | Loss (Q 100 / 60) |
|------|---------------------------------------------------|------------------|
| 2 m, 145.9 MHz | L610, L611 68 nH (√(50·72)/ω = 65.5 nH); C621, C622 18 pF (18.2 pF) | 0.25 dB |
| 70 cm, 436 MHz | L101, L102 22 nH (21.9 nH); C101, C102 6.0 pF (6.08 pF) | 0.30 dB |

Coax: four 1.37 mm cables with MMCX plugs (W10 v3, §4.3), up to 0.22 m, length-matched within 2 mm in each pair (0.5° at 146 MHz, 1.5° at 436 MHz). Isolation the
desense analysis assumed (ANT-5): ≥ 25 dB at 146 MHz and ≥ 15 dB at 437 MHz between the 2 m and 70 cm ports; the
orthogonal dipoles (X and Y) help, and the board needs no more than that (§6.3, §7.5). Verified with the AntS on a
flight-like body (VI-RF-17). If ISIS supplies the phasing on the AntS, the board baluns are depopulated (§4.3).

## 10. Layout, Shielding and Mechanical Rules (for the layout designer)

| Item | Rule |
|------|------|
| Outline and mounting | PC/104 form 90.17 × 95.89 mm, 1.6 mm, four stack columns Ø 5.8 mm at (±40.0, ±43.0) mm from the board centre, at z = +34.8 mm in layout B (values of the CDR mechanical model, to be confirmed against its drawing); board centre support per [STRUCTURAL_ANALYSIS.md](../../../docs/environment/STRUCTURAL_ANALYSIS.md) (PCB fatigue). Component height ≤ 7 mm under the 84 × 54 × 7 mm shield frame at x −42 … +42, y −40 … +14 mm that the mechanical model carries ([MECHANICAL_DESIGN.md](../../mechanical/MECHANICAL_DESIGN.md) §11 item 4), notched at the (+X, −Y) corner for J1 (x > +37.0, y < −26.0) and at the −X edge for J3/J4 (x < −30.0, y −28.0 … −16.0); ≤ 2 mm outside it except J1–J6 (envelopes in §4.3). Nothing at x < −42 in y ±(16 … 28), where the cables pass over the board edge |
| Stack-up | 6 layers, 1.6 mm, high-Tg FR-4 (IS410 or 370HR class): L1 RF and components (35 µm), 0.20 mm prepreg, L2 solid ground, L3 signals, L4 power planes split by domain (+3V0_RX, +3V1_DIG, +5V_PA, +3V3_PA), L5 solid ground (L4–L5 dielectric ≤ 0.30 mm: the PA supply on L4 and its return on L5 form the pair of §10.1), L6 components (digital). 50 Ω microstrip on L1: 0.37 mm (εr 4.2, h 0.20 mm; confirm with the fabricator's field solver); coplanar ground with via fence |
| Placement (B, amended for C16) | Three shielded cavities on L1 under one frame with internal walls: (a) receive, at the frame's (−X, +Y) end (x −42 … −5, y −8 … +14): balun (to J5/J6 just outside the +Y wall), FL1, LNA, SAW, CC1200, TCXO, with the SAW and TCXO at x ≤ −15 (≥ 40 mm from the magnetometer); (b) transmit, at the (−X, −Y) end (x −42 … −5, y −40 … −8): LMX2571, PCM1808, PA, match, LPF, sense line, VHF balun (to J3/J4 in the −X notch); PA and LPF at the (−X, −Y) corner, close to the (−40, −43) column (heat path); (c) digital/audio, the +X half (x −5 … +42) less the J1 notch: MCU, CMX138A, logic, with the 3.6864 MHz crystal Y451 and FB802 at y ≤ −15. Nothing in zone M1 (§10.1), which takes the frame's strip y > +7 between x +9 and +39. FL1 input and output on opposite sides of a cavity wall; ≥ 25 mm between the PA output and the LNA input (LPF and sense line toward −Y, LNA input at y ≥ +2). Connectors: J1 (+X, −Y) corner (§4.1), J2 at (−26, −44) (§4.2), J3–J6 at x −34.5 (§4.3) |
| Isolation | ≥ 85 dB at 146 MHz from the PA output to the LNA input (§6.3); harmonic leakage around the LPF ≤ −90 dBc (§7.4). Shield walls soldered every ≤ 5 mm (λ/20 at 3 GHz is 5 mm), via fence every 2 mm along RF traces and walls |
| Shields (BOM SH1–SH4) | **Nickel-silver CuNi18Zn20 (EN CW409J, UNS C75200), 0.20 mm, unplated**, frame and all three lids. Not tin-plated steel (ferromagnetic, and tin is excluded by EEE_PARTS_LIST §2A.3 item 5), and no nickel plating (electroplated nickel is ferromagnetic). Lids soldered to the frame: **no shield clips** and no steel spring fingers. On the EM only, lids may be held by the frame's own nickel-silver dimples during FL1 and match tuning. 7 mm high |
| Magnetics | Zones M1 and M2 and the current rules of §10.1 (magnetometer board A2 above (+24, +27)); air-core or ceramic-core inductors only (Coilcraft 0603HP and 1812SMS) |
| RF rules | No ground splits under RF; no RF trace over a plane edge; inductors of the LPF at right angles to their neighbours, ≥ 3 mm apart; trap capacitors at the inductor pads; TCXO trace ≤ 10 mm with ground guard; the 5.000 MHz I2S lines and SPI inside the digital cavity, away from the receive cavity |
| Thermal | Q601 exposed pad on ≥ 25 × 25 mm of copper on L1, ≥ 16 vias (0.3 mm) to L2 and L5, 2 oz on L2/L5 under the PA; TMP235 within 5 mm of Q601 on the same pour. Total keyed dissipation 1.26–1.38 W (§11) |
| Creepage, voltage | Highest voltage: 24 V peak RF at the LPF nodes at 1 W into 3:1; 5.15 V DC. IPC-2221 B4 (coated) ≤ 30 V: 0.13 mm; use ≥ 0.2 mm. No corona: well below the Paschen minimum (≈ 300 V) |
| Test points | TP1–TP22 (rails, EN_OK, TONE_OK, NOT_TOT, XPDR_KEY, PTT_EXC, TX_PERMIT_HW, repeat audio, TX audio, lock detect, Vgg, APC reference, debug UART, timer oscillator); SWD on a Tag-Connect TC2030 footprint; U.FL test pads (DNP) at the LPF input and FL1 output for EM characterization |
| Conformal coat | After test: coat everything except connectors and the trap capacitors' tuning area (or coat after final tuning); it is also the tin-whisker mitigation for the QFN and DSBGA parts that cannot be re-tinned |

### 10.1 Magnetic cleanliness (B, amended: OBC item C16)

The magnetometer (PNI RM3100) is on board A2 at (+24, +27) mm in the +Z end-frame slot, z +40.4 … +48.4 (sensor
centre taken at z +44, OBC_BOARD_DESIGN §9.4). That is 4.0 mm above this board's top surface and 2.0 mm above its
2 mm parts outside the shield. Numbers from `rf_calcs.py` §15:

| Zone or source | Rule | Basis |
|----------------|------|-------|
| **M1**: 20 mm in plan round (+24, +27), both sides | **No part of any kind.** Allowed: copper, vias, solder mask, the shield wall and lid | 20 mm in plan keeps every part ≥ 20 mm (3D) from A2's underside (19.6 mm suffices at the board surface, 19.9 mm at 2 mm parts). M1 cuts across the frame's +Y wall, between x +9 and +39, for y > +7 |
| **M2**: 20–40 mm in plan | No ferromagnetic part: ferrite beads, ferrite or iron-powder cores, metal-lidded crystals, TCXOs and SAWs (Kovar or alloy-42 lids), steel cans, nickel-plated or magnetic-stainless hardware or connector bodies. Ordinary chip passives and copper-lead-frame ICs are allowed | A remanent moment of 1 µA·m² gives 25 nT on axis at 20 mm, 7.4 nT at 30 mm, 3.1 nT at 40 mm and 0.9 nT at 60 mm (1/r³) |
| Ferromagnetic parts on this board | FB801–FB804 (ferrite), Y301 TCXO, U201 SAW and Y451 crystal (metal lids or can): all placed ≥ 40 mm away (§10 placement). The BOM notes each one | as above |
| Shield (SH1–SH4) | Nickel-silver, unplated, soldered lids, no clips (§10). The nearest wall and lid edge are 13.0 mm from the sensor, inside M1's radius, so only a non-magnetic shield is acceptable | At 13 mm, 1 µA·m² would give 91 nT |
| Inductors | All RF and drain inductors are Coilcraft 0603HP (ceramic core) or 1812SMS (air core): no magnetic core on the board | — |
| Connectors | J1 (Samtec, phosphor bronze, gold over a nickel underplate) at 63 mm; J2 (Molex) at 87 mm; MMCX jacks J3–J6 (Molex 73415-1473, listed non-magnetic) at 58–78 mm; cable plugs (gold over a probable nickel underplate) ≥ 58 mm | Underplate nickel is microns thick. At these distances it is below 1 nT per µA·m² |
| PA current, 0.469 A at 1 W | Supply and return as a pair: +5V_PA on L4 over its return on L5 (≤ 0.30 mm apart) from J2 to the drain feeds, and the drain loop on L1 over the L2 plane. The whole path, J2 → FB803 → L602/L604 → Q601, stays in the (−X, −Y) corner, ≥ 60 mm from (+24, +27). No PA current in any other layer pair, and no split in L5 under the run | At the sensor, paired on this route: **0.39 nT** (L4/L5) + 0.12 nT (drain loop). Unpaired with the return detoured round a 20 × 14 mm loop: 22 nT. Paired but routed 20 mm from the sensor: 22 nT. A single unpaired wire at 20 mm: 4.7 µT |
| Other rails | RADIO_3V3 (≤ 0.09 A) and the LDO outputs over their own ground plane (L2/L5 solid); none routed through M1 | Pairing as above; currents 5× smaller than the PA's |

The field that matters most is the step between transmitter off and keyed, because calibration does not remove a
field that switches with the transmitter. The target is below the RM3100's 15 nT noise; the computed step is 0.5 nT.
Static fields from remanence are a constant bias, removed by the on-orbit calibration. They are kept small by M1
and M2 and are measured (VI-RFB-11).

## 11. Thermal Model Inputs

| State | Board dissipation | Main sources |
|-------|-------------------|--------------|
| Receive only | 0.10–0.13 W | CC1200, LNA, MCU |
| Receive + repeat enabled | 0.13–0.16 W | + CMX138A |
| Keyed 0.5 W | 1.26 W + receive | Q601 0.87 W, LMX2571 0.15 W, PCM1808 0.07 W, network losses 0.09 W |
| Keyed 1 W | 1.38 W + receive | Q601 0.86 W, network losses 0.17 W |

THERMAL_ANALYSIS v2.0 used 1.7 W dissipated while keyed at 1 W: conservative against these values. Hot case: PA
junction 88 °C at the 60 °C per-unit limit (§7.3); cold case: all parts rated −40 °C (the board's −11.9 °C and
−18.6 °C contingency nodes are inside every part's range; the TCXO and CC1200 start at −40 °C).

## 12. Parts, Derating, Finishes, Radiation

Stresses (worst case, `rf_calcs.json` "stresses" and the BOM column "Derated stress"):

| Part | Stress | Rating | Ratio | Rule |
|------|--------|--------|-------|------|
| RD01MUS3 V_DS (RF peak, 2 × 5.15 V × 1.5 at 3:1) | 15.5 V | 25 V | 0.62 | ≤ 0.75 (DC 0.21) |
| RD01MUS3 I_D2 | 0.42 A | 0.9 A | 0.46 | ≤ 0.50 |
| RD01MUS3 P_D2 | 0.77 W | 6.0 W at Tc 60 °C | 0.13 | ≤ 0.50 |
| RD01MUS3 Tj | 88 °C | 150 °C | 0.58 | ≤ 0.80 |
| RD01MUS3 input (FET1) | 1 mW | 2 mW | 0.50 | ≤ 0.50 |
| LPF and match capacitors | 24 V / 15.5 V peak | 100 V C0G | 0.24 / 0.16 | ≤ 0.50 |
| LPF inductors, drain chokes | 0.35 A peak, 0.42 A DC | 2.5 A | 0.10–0.17 | ≤ 0.50 |
| BFP640F V_CE | 2.0 V | 4.1 V | 0.49 | ≤ 0.50 |
| LDOs (TPS7A2030, TPS73101, TPS7A2033), TPS22918 | 30, 14, 17, 51 mA | 300, 150, 300, 2,000 mA | ≤ 0.10 | ≤ 0.70 |
| Bulk X7R capacitors | 5.15 V / 3.40 V | 25 V, 16 V / 10 V | ≤ 0.34 | ≤ 0.50 |
| Resistors | ≤ 33 mW (R605), ≤ 11 mW (I2S series, fault case) | 100 mW | ≤ 0.33 | ≤ 0.50 |
| J2 PA contacts | 0.47 A over two | 1.0 A each | 0.23 | ≤ 0.50 |
| LMX2571 Tj | 65 °C | 125 °C | 0.52 | ≤ 0.80 |

Every resistor, capacitor and inductor line in the BOM carries its own computed stress.

Verification of the MPNs (2026-10-27): TI parts (CC1200, LMX2571, PCM1808, TPS7A2030/2033, TPS73101, TPS22918,
REF35250Q, TLV9001/9002, TMUX1101/1219, TMP235, LVC logic, CD74HC4060): orderable addenda read, all **Active**, with
their lead finishes. Mitsubishi RD01MUS3 (data sheet January 2026), Infineon BFP640F (data sheet 2024), CML CMX138A
(data sheet 2014; distributor: Active), Tai-Saw TA2209A (approval sheet 2017), Epson TG2016SMN family (distributors):
documents read, production status of TA2209A and the exact 40 MHz Epson code not confirmed. Not checked: STM32L431CCT6
(st.com unreachable; LQFP-48 pin numbers taken from DS11453 as remembered, to check at schematic review), AD8361ARMZ
(analog.com unreachable; strapping and pinout to check; the TI LMH2121 is a verified-Active alternate), Samtec, Molex,
Amphenol connectors, Abracon crystal, Vishay BAS16, passive part numbers (series checked, each value's code built from
the ordering scheme, Murata X7R suffixes to confirm in SimSurfing).

Finishes (SYS-915; EEE_PARTS_LIST §2A.3): NiPdAu on most TI parts; SN (matte tin) on TPS73101, LMX2571 (QFN),
TLV9001, some LVC logic and TMP235 options, chip passives, connector tails: re-tin leaded and chip parts with Sn63Pb37;
conformal-coat the QFN/DSBGA parts (LMX2571, CC1200 is NiPdAu). Coilcraft inductors with the tin-lead termination
option. Shields and connector contacts not tin-plated.

Radiation (BASELINE §6: 10 krad design): no part has published TID data except as noted; CMOS RF ICs of this class
(CC1200, LMX2571) typically survive tens of krad, but that is an expectation, not data: lot sample test RAD-V3 at
≥ 10 krad for CC1200, LMX2571, STM32L431, CMX138A, PCM1808 (EEE_PARTS_LIST §2A.4). Latch-up: the RADIO rail's
always-on parts have no current limiter on this board (§4.2, §15). Single-event upsets in the CC1200, LMX2571 and
CMX138A registers: the TNC re-writes and verifies their configuration every 10 s and at every key-up.

## 13. TNC Firmware (new software item)

Not flight software of the OBC, but software on the spacecraft (Class C per the SDP): KISS on USART2 at 9600 baud
(parameters of ICD-003 §5.2, FULLDUPLEX 1); HDLC with FCS-16, NRZI, bit stuffing; AFSK 1200 demodulator (correlator,
as LBA §3) on the CC1200 CFM samples at 12.5 kS/s; AFSK modulator (DAC1 sine table at 50 kS/s) with TXDELAY/TXTAIL;
repeat audio to DAC2 (DC removal, 1 sample latency); (B) RX_ALIVE pulses every 100 ms from the main loop only while
the five receiver-health criteria of §8.6 hold, and CC1200 GPIO2 configured as carrier sense with the calibrated
threshold; CC1200, LMX2571 and CMX138A configuration with read-back
scrubbing; TONE_OK relay from CMX138A IRQN; frequency trims from flash; independent watchdog. It can reuse the
`sqso-protocol` KISS and AX.25 code (`no_std`, Rust). Its requirements and tests belong in
`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` (for the chair).

## 14. Verification Items Added by This Design

| ID | Test and pass criterion | Verifies | When |
|----|-------------------------|----------|------|
| VI-RFB-01 | CC1200 CFM demodulator through the TNC: 12 dB SINAD at ≤ −120 dBm and telecommand 90 % frames at ≤ −114 dBm (2 dB implementation loss assumed here), 0 and ±2.5 kHz offset | §6.1, RF-P011 | EM |
| VI-RFB-02 | FL1 and SAW on the board: ≥ 60 dB at 144–146 MHz, ≥ 40 dB at 250–330 and 560–620 MHz, ≤ 1.0 dB before the LNA; and board isolation PA → LNA ≥ 85 dB at 146 MHz | RF-P007, §6.3 | EM |
| VI-RFB-03 | Adjacent-channel selectivity at ±25 kHz (25 kHz channel filter) | RF-P010 | EM |
| VI-RFB-04 | PA load-pull at 5.0 V: P_sat ≥ 1.15 W at 145.9 MHz with 0 dBm drive; DC at 0.5 W ≤ 1.8 W | §7.3 | EM |
| VI-RFB-05 | PCM1808 → LMX2571 I2S alignment (16 MSBs) and deviation: ±3.0 ± 0.15 kHz AFSK, repeat limit ≤ 5.0 kHz at PA_5V 5.15 V | RF-P004 | EM |
| VI-RFB-06 | LPF on the board: ≥ 55 dB at 2f and ≥ 75 dB at 3f measured; leakage floor ≤ −90 dBc | §7.4, RF-07 | EM |
| VI-RFB-07 | Key-down timer 120 ± 5 s over −20 to +60 °C after trim | RF-F008 | EM, thermal |
| VI-RFB-08 | TX permit: with PA_5V removed, no emission > −60 dBm at J3/J4 with TNC_PTT and XPDR_KEY forced high; with U406 output forced high and PA_5V removed, the same | §8.5, VI-RF-12 | EM |
| VI-RFB-10 | Receiver supervision: RX_CD follows a carrier 3 dB above the threshold and stays low on noise (< 1 false alarm per hour); RX_FAULT goes high 0.36–0.62 s after each injected fault of §8.6 (CC1200 held in reset, SPI line open, TCXO removed, TNC halted) and low after recovery; with RADIO_3V3 off both lines read low at the OBC and no current flows into the board | RID RFB-07, SRS-F115–F117 | EM |
| VI-RFB-11 | Magnetic cleanliness (C16): with the RF board in the stack and A2 fitted (or a reference fluxgate at the A2 sensor position), the field step between transmitter off and keyed at 1 W is < 15 nT on each axis, and the RF board's static field at the sensor (board in and out, as received and after a 5 mT exposure on each axis) is recorded for the ADCS calibration; first-article inspection of the shield for permeability (no attraction to a 0.1 T test magnet) | OBC item C16, OBC_BOARD_DESIGN §9.4 | EM |
| VI-RFB-12 | Mated heights and footprints from the manufacturers' drawings, **due 2026-11-13 (before layout starts)**: J1 Samtec FTSH-108-01-L-DV-K with its FFSD-08 cable socket (allocation 10.0 mm), J2 Molex 501331-0607 with its housing (8.0 mm), J3–J6 Molex 73415-1473 (SD-73415-147-001) with the 73414-0351 plug (6.0 mm; cable axis z +39.4); confirm 73415-1473 is non-magnetic with no nickel underplate and the plug's plating; the 1.37 mm cable's attenuation (≤ 1.33 dB/m at 436 MHz hot), inner conductor (silver-plated copper) and outgassing | §4.3, §10, OBC item C16 | Analysis (drawings) |
| VI-RFB-13 | AntS RF interface against the ISIS ICD: connector type (MMCX jack assumed), number of ports (four, or two if ISIS phases the pairs), their positions on the AntS's inner face, and the clearance of the mated right-angle plugs under the battery (z −48.5 … −42.2 under the cells, −48.5 … −44.9 under the clamp end caps); then the cable cut lengths. Also: fit check of the four cables in the −X wall-gap bands on the EM structure; return loss port to antenna ≥ 13.6 dB with the AntS replaced by 50 Ω loads | ANT-3, ANT-8; §4.3 | Analysis (ICD), EM |
| VI-RFB-09 | Sense-line coupling and directivity (≥ 15 dB); detector calibration ±0.5 dB −20 to +60 °C | §7.6 | EM |

VI-RF-01 to VI-RF-17 of SQSO-LBA-001 §8, RF_DESIGN §6 and ANTENNA_DESIGN §7 still apply.

## 15. For the Chair: Interface and Baseline Changes

| # | Change | Owner | Recommended text or action |
|---|--------|-------|----------------------------|
| 1 | OBC inputs for the three readbacks (RID RF-21 / AV-05): TONE_OK on W4 J1-8, XPDR_KEY on J1-9, PTT_EXCITER on J1-10; 3.1 V CMOS push-pull through 1 kΩ, active high, valid while RADIO_3V3 is on; three GPIO inputs with pull-down, PTT_EXCITER preferably on a timer input-capture pin | OBC designer (pins), AV (firmware) | Allocate three STM32 pins; firmware reads them into the transponder statistics and checks keyed time against TX_FWD |
| 2 | W4 signal connector 10 → 16 positions (FTSH-108): XPDR_EN (missing from W4), readbacks J1-8/9/10, and (B) RX_CD J1-15, RX_FAULT J1-16 | OBC designer | OBC J-RF pinout per §4.1; two more OBC inputs with pull-downs for RX_CD and RX_FAULT (RID RFB-07) |
| 3 | OBC SPI2 (PB13–PB15, PD0, "RF transceiver" in `spi.rs`) is no longer used by the radio | OBC designer, AV | Free the pins |
| 4 | SW2 trip current 0.25 A does not detect latch-up of a 38 mA load | EPS designer | SW2 current limit ≈ 0.12 A, or OBC monitoring of the RADIO rail current with a power cycle of RADIO_EN |
| 5 | 3V3_AO tolerance ±3 % leaves 39 mV for the LMX2571's 3.15 V minimum | EPS designer | ±2 % set point, or confirm ±3 % includes load and temperature |
| 6 | Transponder DC 1.76 W at 0.5 W RF (BASELINE §5 says ≈ 1.5 W) | Chair (BASELINE §5), PBA | "FM transponder (0.5 W RF ≈ 1.8 W DC), duty ≤ 25 %: 0.44 W"; or a 6 V PA rail / 12 Ω load line (§7.3) |
| 7 | 1 W setting 29.5–30.1 dBm | Chair | Keep "1 W maximum" for the filing; RF-P003 1 W level 30.0 +0.3/−0.6 dB |
| 8 | Uplink request 435.050–436.500 MHz (SAW temperature drift, 435.000 MHz clock harmonics) | Chair (IARU request), REG | Add to the IARU request alongside the 437.3 MHz upper bound |
| 9 | Receiver changed from "AX5043/SA818 class" to CC1200; the repeat audio passes through the TNC firmware | Chair (BUILD_VS_BUY §3.2 text) | "…the receiver's demodulated audio, produced by the TNC MCU from the CC1200's frequency samples (no analog FM receiver IC is in production within the power budget); tone decode, band-limiting, level limit, key gate, timer and TX permit in hardware" |
| 10 | TNC firmware is a new software item | Chair (SDP) | Add to the software inventory, Class C, with tests VI-RFB-01, -05 |
| 11 | ±1.0 ppm over 2 years needs a re-trim | Chair, AV | Optional KISS SetHardware frequency-trim command, or ground-side correction |
| 12 | ICD-003 §4.2/§4.3/§6/§8: readbacks wired, TONE_OK etc. levels, RADIO_EN-to-ready ≤ 50 ms (MCU boot ≈ 5 ms, CC1200 set-up ≈ 10 ms, TCXO ≤ 2 ms), connector part, ADC scalings confirmed | Chair (ICD owner) | Update from §4 |
| 13 | VI-RF-11 "unkeys within 100 ms in every case": a stalled main loop takes up to 118 ms (60 ms lease + 58 ms) | AV / chair | Measure from the last edge (≤ 59 ms), or shorten the lease to ≤ 40 ms |
| 14 | (B, amended) C16 layout constraints: J1 top side at (+41.0, −33.5) with the pin field along Y; J2 top side at (−26, −44); shield frame notched at x > +37.0, y < −26.0; top-side connectors exceed the 2 mm outside-shield envelope (mated heights open, Samtec and Molex drawings); zone M1 on the RF board | Mechanical designer, harness designer (W4A, W4B) | Carry the notch and the two connector envelopes in the CAD model; W4A ribbon from J1 into the +X wall gap at y −28 … −39 (it shares the bundle band y −24 … −34 with the harness: fold 45° or move the −Y bundle, as in OBC_BOARD_DESIGN §21); W4B twisted PA_5V/GND pairs |
| 15 | (C) RF connectors and antenna cables, **settled**: MMCX jacks J3–J6 on the top side at x −34.5 (J3/J4 in a −X notch of the shield frame, x < −30.0, y −28.0 … −16.0), 1.37 mm cables with right-angle MMCX plugs, down the −X wall gap in the bands y ±(20.0 … 24.0), to the AntS's MMCX ports (assumed). Envelopes, the cable path and the bends in §4.3 | Mechanical designer (CAD envelopes, shield notch); harness designer (W10 v3, ties in the gap; HARNESS_DESIGN §4.7 H7 still says RG-316 and SMA); chair (TRANSMITTER_DESIGN §11 and RF_ELECTRICAL_DESIGN connector rows still say SMA end launch) | Put the §4.3 envelopes in the CAD model in one pass; keep the −X bands y ±(20 … 24) free of harness; update H7 to W10 v3. The CAD model's AntS has two elements (option A: 2 m +X, 70 cm −X); this board and ANTENNA_DESIGN 2.1 use four (2 m ±X, 70 cm ±Y): the model needs the four-element case for the port count |

## 16. Not Verified

- No hardware: every RF number is analysis (filters by ABCD with tolerances, PA from data-sheet points and a
  load-line model, receiver NF partly estimated, coupler coupling estimated).
- CC1200 analog-FM (CFM) quality for voice and the AFSK threshold through it (VI-RFB-01); its 25 kHz-channel
  selectivity (VI-RFB-03).
- LMX2571 FSK I2S alignment with the PCM1808 (VI-RFB-05); phase noise at 146 MHz extrapolated from 480 MHz.
- RD01MUS3 at 5 V (only 3.6 V and 7.2 V characterized) and its efficiency at back-off (VI-RFB-04).
- Pinouts not read from a data sheet in this session: STM32L431CCT6 (LQFP-48), AD8361, BFP640F (TSFP-4), Epson
  TG2016SMN pads, BAS16; status of TA2209A, AD8361, connectors and the Abracon crystal (§12).
- CC1200 receive balun and loop-capacitor values: starting values; copy TI's 420–470 MHz reference design.
- TCXO aging (not on the listings reached).
- AntS interface (connector type, whether ISIS supplies the phasing): RFQ.
- Connector drawings not read (Molex and Samtec sites unreachable or without the data): mated heights in §4.3 are
  allocations, and the 1.37 mm cable attenuation is a class value (VI-RFB-12). AntS RF connector type and positions
  assumed from the ISIS brochure (VI-RFB-13).
- Magnetic moments of the parts (lids, cans, ferrites, MLCC nickel) are not known: §10.1 uses distance rules and a
  dipole scale, and the PA current field is a Biot–Savart model of the planned route, not a layout extraction
  (VI-RFB-11).

---

## Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR design completion, RF board | Initial issue: build-to design of the RF board (receiver, TNC, exciter, PA, filters, repeat path and `XPDR_EN` gate, monitors, antenna interface), schematic as code with parts, netlist, BOM, ERC, calculations, layout rules |
| B | 2026-10-27 | CDR design completion, RF board (RID RFB-07) | Receiver supervision lines to the OBC: RX_CD (CC1200 carrier sense, J1-15) and RX_FAULT (hardware health monostable on the TNC's RX_ALIVE pulses, J1-16), fault criteria (§8.6), TNC firmware duties (§13), VI-RFB-10; J1/W4 16 positions (W4 v3); power, part counts and ERC re-run; BASELINE cited at Rev E |
| B (amended) | 2026-10-27 | CDR design completion, RF board (OBC item C16) | Magnetometer moved to board A2 above (+24, +27): magnetic-cleanliness rules (§10.1: zone M1 with no parts, zone M2 with no ferromagnetic parts, PA supply and return paired on L4/L5 and kept in the (−X, −Y) corner, field computed in `rf_calcs.py` §15); shield specified as unplated nickel-silver with soldered lids and no clips (BOM SH1–SH4); cavities re-placed (receive cavity to the frame's (−X, +Y) end); J1 in the (+X, −Y) corner with a shield notch, J2 by the PA; VI-RFB-11; §15 items 14 and 15 (connector envelopes, coax connectors versus the wall gaps) |
| C | 2026-10-27 | CDR design completion, RF board (§15 item 15) | Antenna ports settled (§4.3): SMA end launch replaced by Molex 73415-1473 MMCX jacks (non-magnetic) at x −34.5, J3/J4 in a −X shield notch; 1.37 mm cables with Molex 73414-0351 right-angle MMCX plugs down the −X wall gap in the free bands y ±(20 … 24) to the AntS (MMCX assumed); positions, envelopes and cable path for the CAD; J1/J2/J3–J6 mated-height allocations; loss and return loss computed (`rf_calcs.py` §16): 0.44 dB at 146 MHz (0.56 dB inside the LBA's 1.0 dB), 0.62 dB at 436 MHz, T_sys 548–598 K (requirement 650 K); W10 v3; VI-RFB-12 (drawings, due 2026-11-13), VI-RFB-13 (AntS ICD) |
