# Radio Frequency (RF) Subsystem Design Document

**Document Number**: SMART-QSO-RF-001
**Version**: 2.6
**Date**: 2026-10-27
**Status**: CDR design description. The architecture, interfaces and requirements are defined here; the build-to board design with selected parts is [RF_BOARD_DESIGN.md](../../hardware/electrical/rf/RF_BOARD_DESIGN.md) (SQSO-RF-BRD-001 F), not yet built or measured (§9). Values follow [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) §7, which governs.

**Governing documents**:
- 47 CFR Part 97: 97.207 (space station), 97.211 (telecommand), 97.307(e) (spurious, 30–225 MHz), 97.313(f) and
  47 CFR 2.106 US270 (70 cm power near Beale AFB); eCFR as of 2026-10-01
- ITU Radio Regulations Nos. 1.56, 1.57, 22.1, 25.2A, 25.11
- IARU Amateur Satellite Frequency Coordination Request, form v40
- Cal Poly CubeSat Design Specification (CDS) Rev 14.1 §2.3.7 (RF inhibits), §2.4.5 (45-minute RF silence)
- NPR 7123.1D Appendix G (CDR entrance and success criteria, tailored for a Class D CubeSat); GSFC-STD-7000B §2.5
  (EMC; the self-compatibility tests of VI-RF-06 and VI-RF-07 stand in for its radiated-emissions and susceptibility
  tests at the receive bands)
- SpaceX Rideshare Payload User's Guide v11 (interim provider document): payload unpowered and RF silent through
  launch and deployment; the integrator ICD replaces it when signed
- Link budget: [LINK_BUDGET_ANALYSIS.md](../environment/LINK_BUDGET_ANALYSIS.md) (SQSO-LBA-001 v3.5)
- Interfaces: [ICD_OBC_RF.md](../interfaces/ICD_OBC_RF.md) (SQSO-ICD-003 v2.4),
  [TELECOMMAND_AUTHENTICATION.md](../interfaces/TELECOMMAND_AUTHENTICATION.md) (SQSO-ICD-013)

Version 1.x described a beacon-only transmitter with no receiver. That design could not be authorized (47 CFR
97.207(b), RR 22.1; findings RG-01, TB-04) and is replaced in full by this version.

---

## 1. Scope

The RF subsystem is the amateur station of the spacecraft:

1. a 435 MHz receiver, always on after the RF silence, carrying authenticated telecommand and the transponder uplink;
2. a 145.9 MHz FM transmitter shared, in time, by the AX.25 beacon (through a KISS TNC) and the FM repeat path;
3. the repeat path from receiver audio to transmitter audio, gated in hardware by `XPDR_EN`, CTCSS and a key-down timer
   (`docs/hardware/BUILD_VS_BUY.md` §3.2);
4. the TX-OFF and inhibit hardware (`RF_TX_EN`, `RADIO_EN`, the separation switches);
5. the antennas: ISIS AntS 1U–3U, separate 2 m and 70 cm tape elements (BASELINE §7; BUILD_VS_BUY §2.6).

Decision record: **build** the transponder radio from the receiver and the 2 m transmitter; **buy** the antenna
(BUILD_VS_BUY §1, §3). No flight-proven U/v FM transponder is for sale.

## 2. Requirements

### 2.1 Functional

| ID | Requirement | Source | Verification |
|----|-------------|--------|--------------|
| RF-F001 | Transmit AX.25 UI beacons, 1200 bps AFSK (Bell 202) on FM, from a KISS TNC | SYS-202, SYS-210; BEACON_SPEC | Test |
| RF-F002 | Receive authenticated telecommand, 1200 bps AFSK AX.25 UI, on the coordinated 435 MHz uplink, whenever the radio is powered | SYS-204, SYS-325; 97.207(b) | Test |
| RF-F003 | Repeat a CTCSS-valid (67.0 Hz) FM uplink on the 2 m downlink while `XPDR_EN` is valid, with the TNC's frames taking the transmitter first | MRD-031; BUILD_VS_BUY §3.2 | Test |
| RF-F004 | TX-OFF: `RF_TX_EN` low removes PA power through a hardware switch; pulled down so a reset or unpowered OBC leaves the PA off | SYS-204; ICD-013 §6 | Test (TP-RF-006) |
| RF-F005 | No RF signal generated or transmitted until 45 min after deployment (`RADIO_EN` and `RF_TX_EN` low) | SYS-224; CDS 2.4.5 | Test (TP-RF-006 §8.8) |
| RF-F006 | Power select 0.5 W / 1 W (`TX_PWR_SEL`), never above 1 W | SYS-201 | Test |
| RF-F007 | Forward, reflected power and PA temperature to the OBC ADC | ICD-003 §5 | Test |
| RF-F008 | Hardware key-down timer, 120 s, on the repeat path | BUILD_VS_BUY §3.2 | Test |

### 2.2 Performance

| ID | Parameter | Requirement | Basis |
|----|-----------|-------------|-------|
| RF-P001 | Downlink frequency | One IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz | BASELINE §7, SYS-200 |
| RF-P002 | Uplink frequency | One IARU-coordinated frequency, requested in **435.222–435.778 MHz, or else 436.222–436.500 MHz** (BASELINE §7): inside the RF board's receive band 435.050–436.500 MHz (SAW drift and the 435.000 MHz clock harmonics, SQSO-RF-BRD-001), in the windows clear of the EPS converters' 1.000 MHz harmonics (SQSO-EPS-BRD-001, SQSO-EMC-001), and ≥ 0.9 MHz below 3 × f_down. (2.5: the 435.000–437.300 MHz segment of 2.2 is narrowed to the board's band.) With the LMX2571 exciter at −70 dBc H3 the desense is ≤ 0.44 dB even 100 kHz from 3f (SQSO-RF-BRD-001 §7.5), so the earlier 2 MHz separation is not needed | SQSO-LBA-001 §7.1, §7.3 |
| RF-P003 | Output power at the antenna port | 27.0 ± 0.5 dBm and 29.5–30.0 dBm (never above 1 W, SYS-201), −40 to +70 °C (2.4: was −20 to +60 °C; board allowables plus test margin, first-orbit cold start at −40 °C) | BASELINE §7; SQSO-MFG-003 |
| RF-P004 | Deviation | Beacon ±3.0 kHz peak (10K4F2D); transponder ±5.0 kHz peak with a limiter (16K0F3E) | ITU_APPENDIX4_DATA §C |
| RF-P005 | Frequency stability, both bands (amended, RID RFB-05) | **±1.0 ppm over temperature** (−40 … +85 °C, about the calibrated offset), and **≤ ±3.0 ppm total offset over 2 years including ageing**. The ±1.0 ppm over 2 years of 2.2 is not met: the selected TG2016SMN reaches 2.1 ppm worst after 2 years (0.6 ppm at BOL plus class ageing; SQSO-RF-BRD-001 §6.5). **The ground corrects the uplink from the measured downlink offset.** One 40 MHz reference drives both bands, so a station that measures the beacon's offset scales it by 435.5/145.9 and pre-corrects its uplink. That keeps the residual at the spacecraft within ±1 kHz, and the CC1200's AFC tracks it. An optional TNC re-trim (KISS SetHardware, bounded to ±3 ppm) is not required | SQSO-LBA-001 §4.4 (±1 kHz pre-correction); chair decision RFB-05 (2026-10-27) |
| RF-P006 | Receiver system noise temperature at the antenna port, RX filter included | ≤ 650 K with a 290 K source (NF ≤ 4.7 dB); the budget carries 1,000 K | SQSO-LBA-001 §2.4 |
| RF-P007 | RX filter | Ahead of the LNA: ≥ 60 dB rejection at 144–146 MHz, ≤ 1.5 dB loss at 435.0–437.3 MHz. Ahead of the receiver IC (filter after the LNA allowed): ≥ 40 dB at 250–330 MHz and 560–620 MHz (2.1: the second-order products these bands guard against form after the LNA) | SQSO-LBA-001 §7.4–7.5; SQSO-RF-BRD-001 §6.2 |
| RF-P008 | Harmonics and spurious at the antenna port | 2nd, 4th and all spurious ≤ 25 µW and ≤ −46 dBc at 1 W (97.307(e)); **3rd ≤ −70 dBc** measured at the antenna port with the antennas deployed (BASELINE §7) | 97.307(e); SQSO-LBA-001 §7.3 |
| RF-P009 | Full-duplex desense | ≤ 0.5 dB loss of receiver sensitivity with the transmitter keyed at 1 W | SQSO-LBA-001 §7.3 |
| RF-P010 | Receiver IF | ±7.5 kHz (25 kHz-channel filter); adjacent-channel rejection ≥ 60 dB at ±25 kHz (selected CC1200: 54–60 dB data-sheet value at ±12.5 kHz; to be measured, VI-RFB-03) | SQSO-LBA-001 §3, §4.4 |
| RF-P011 | Demodulators | Telecommand AFSK: 90 % of 60-byte frames at C/N₀ ≤ 53.0 dB-Hz. Repeat audio: 12 dB SINAD at C/N₀ ≤ 47.5 dB-Hz | SQSO-LBA-001 §3.2 |
| RF-P012 | CTCSS | 67.0 Hz at ±0.5 kHz decoded at 6 dB SINAD, within ±2.5 kHz carrier offset; key within 200 ms typical and 260 ms maximum (2.1: the CMX138A specifies 250 ms maximum, plus a 2 ms relay); drop within 260 ms of tone loss | SQSO-LBA-001 §4.4; SQSO-RF-BRD-001 §8 |
| RF-P013 | Antenna gain (proposed SYS-222 replacement) | Median ≥ −3 dBi and 10th percentile ≥ −10 dBi toward the ground over the pass-weighted directions, each band | SQSO-LBA-001 §5.2 |
| RF-P014 | Antenna VSWR | < 2:1 at the coordinated frequencies, deployed, on the flight body | SYS-223 |
| RF-P015 | Power (DC) | Receiver chain ≤ 0.15 W always on; transmitter ≤ 1.85 W DC at 0.5 W RF, exciter included (BASELINE §5 allocation, raised from 1.5 W to the RF board's designed draw); repeat path ≤ 30 mW while enabled. Design (SQSO-RF-BRD-001 §1, §5): receiver 0.102 W typical, 0.127 W worst; transmitter 1.84 W at 0.5 W RF (Version E: LMX2571 on its own LDO), 2.42 W at 1 W; repeat path 24 mW typical, 31 mW worst | BASELINE §5 |

### 2.3 Regulatory

| ID | Requirement | Authority |
|----|-------------|-----------|
| RF-R001 | IARU coordination before frequencies are frozen | IARU form v40 |
| RF-R002 | Part 97 space station: cessation of emissions by telecommand (RF-F004) | 47 CFR 97.207(b); RR 22.1 |
| RF-R003 | No encoded downlink; only telecommand authenticated | 97.113(a)(4), 97.211(b) |
| RF-R004 | Station identification KK6GDB-11 in every beacon (good practice; space stations are exempt from 97.119(a)) | BEACON_SPEC §2 |
| RF-R005 | Emission designators as filed: 16K0F3E, 10K4F2D; occupied bandwidth not above them | ITU_APPENDIX4_DATA §C |

### 2.4 Environment

| Parameter | Operating | Non-operating |
|-----------|-----------|---------------|
| Temperature | Verified −40 °C to +70 °C (2.4: was −20 … +60 °C; the board allowables of THERMAL_DESIGN §1.1 plus test margin, with the first-orbit cold start at −40 °C; PA held below 70 °C by the downlink software). Q601 (RD01MUS3) has no published minimum operating temperature: Mitsubishi statement and EM characterisation at −40 °C (SQSO-RF-BRD-001 F, VI-RFB-14) | −40 °C to +85 °C |
| Vibration | Protoflight random vibration 8.18 Grms, 1 min per axis (BASELINE §8; `docs/test_procedures/TP-ENV-002_Vibration.md`; 2.1: was 5.57 Grms MPE) | |
| Vacuum | 10⁻⁶ Torr; no corona at 1 W (a few volts RMS) | |
| Radiation | 10 krad TID design (BASELINE §6) | |

## 3. Architecture

```
            70 cm tape (AntS)                                   2 m tape(s) (AntS)
                  |                                                    ^
           RX filter (RF-P007)                                         |
                  |                                               LPF (§5.3)
            LNA + 435 MHz FM receiver  <---- TCXO ±1 ppm ---->   PA (load switch: RF_TX_EN)
            (always on after silence;                                  ^
             RADIO_EN powers the radio)                           FM exciter 145.9 MHz
                  |  discriminator audio                               ^ audio
        +---------+-----------------------+                     [2:1 audio select]
        |                                 |                      ^             ^
  AFSK demod (KISS TNC) --UART--> OBC     CTCSS + audio          | TNC audio   | repeat audio
        ^                          |      conditioning           |             |
        +---- KISS frames <--------+      (CMX138A class)        |             |
                                   |        TONE_OK              |             |
                     XPDR_EN (heartbeat) -> monostable EN_OK -> AND(EN_OK, TONE_OK, NOT_TOT) = XPDR_KEY
                     RF_TX_EN ---------------------------------------------------------> PA supply switch
                     TX_PWR_SEL, ADC: TX_FWD, TX_REV, TX_TEMP <-------------------------- coupler, NTC
```

`PTT_EXCITER = TX_PERMIT_HW · (TNC_PTT + XPDR_KEY · ¬TNC_PTT)`; the audio select follows TNC_PTT, so beacon frames
take the transmitter with no software arbitration and receiver noise never rides on a beacon (BUILD_VS_BUY §3.2).
No state of the repeat-path logic can power the PA: only `RF_TX_EN` can.

**As designed (2.1, SQSO-RF-BRD-001):** the discriminator audio of this diagram is produced by the TNC MCU from the
CC1200's analog-FM frequency samples: no analog FM receiver IC is in production within the 0.15 W allocation (the
AX5043 is discontinued, the SA6xx and MC33xx FM IF families are no longer made). The tone decode, band-limiting, level
limit, key gate, timer and TX permit remain hardware; the TX permit is the presence of the PA rail, and it also powers
and enables the exciter (SQSO-RF-BRD-001 §8).

### 3.1 Receiver

| Item | Design |
|------|--------|
| Function | Single-conversion or SDR-on-chip FM receiver, 435.050–436.500 MHz (the board's band, RF-P002), ±7.5 kHz IF, discriminator audio out (flat, before de-emphasis) for the TNC and the repeat path |
| Front end | RX filter (RF-P007) then an LNA, NF ≤ 1 dB, IIP3 ≥ 0 dBm |
| Always on | Powered whenever `RADIO_EN` is high, which is always after the 45-minute silence; TX-OFF, loss of contact and EPS `radio_enabled` do not switch it (ICD-013 §6) |
| Selected (2.1) | 70 cm lattice balun, FL1 7th-order high-pass with three 146 MHz traps, Infineon BFP640F LNA (6 mA), Tai-Saw TA2209A SAW, TI CC1200 receive-only in its custom-frequency-modulation (analog FM) mode, TCXO Epson TG2016SMN 40 MHz ±0.5 ppm: T_sys 503–550 K with a 290 K source, 0.104–0.129 W (SQSO-RF-BRD-001 §6). Rejected: NiceRF SA818 (60 mA, −30…+70 °C), AX5043 (discontinued), CMX994EF (39–58 mA) |

### 3.2 TNC

| Item | Design |
|------|--------|
| Function | KISS TNC: HDLC framing, FCS, AFSK 1200 modulation and demodulation, PTT, on USART2 at 9600 baud (as built in firmware, CR-2026-005) |
| Parameters | TXDELAY 300 ms (`DOWNLINK_TXDELAY_MS`), persistence 255, slot time 0, full duplex (the downlink does not wait for a clear uplink) |
| Selected (2.1) | STM32L431CCT6 running an AFSK 1200 correlator modem, HDLC and KISS on the CC1200's samples; it also configures and scrubs the CC1200, LMX2571 and CMX138A and relays TONE_OK. New software item (SQSO-RF-BRD-001 §13). The OBC does not run the modem (SQSO-ICD-003 §6) |

### 3.3 Transmitter

| Item | Design |
|------|--------|
| Exciter | PLL synthesizer locked to the shared TCXO, direct FM, ±3.0 / ±5.0 kHz deviation set by fixed attenuators per audio source (RF-P004) |
| PA | 0.5 / 1 W, ≥ 33 % overall efficiency at 0.5 W, supply through the `RF_TX_EN` load switch from the battery-derived rail; forward/reverse coupler and NTC (RF-F007) |
| Filter | 7-element low-pass as `hardware/rf/TRANSMITTER_DESIGN.md` §7.3: −0.66 dB at 145.9 MHz, −50.5 dB at 2f, −77.5 dB at 3f (analysis with Q 80, SQSO-LBA-001 §7.2). Realized stopband to be measured (VI-RF-04) |
| Selected (2.1) | TI LMX2571 synthesizer (phase noise −152 dBc/Hz at 1 MHz at 146 MHz), deviation from a TI PCM1808 audio ADC over I2S (ADC full scale = ±6.3 kHz, the hard deviation limit); Mitsubishi RD01MUS3 PA at 5 V with closed-loop power control on the gate bias; 2-section match and 7th-order elliptic LPF (worst of 3,000 Monte Carlo trials: 2f ≥ 65.5 dB, 3f ≥ 88.2 dB); printed sense line and two AD8361 detectors; TMP235 on the PA (SQSO-RF-BRD-001 §7). 0.50 W at 1.84 W DC (SQSO-RF-BRD-001 E: the LMX2571 on its own regulator; 1.76 W before); 1 W setting 29.5–30.1 dBm saturated over the 5 V ± 3 % rail, held by the APC loop at 30.0 dBm. Replaces the filter of TRANSMITTER_DESIGN 1.3 §7.3 |
| Software protections (as built) | Frame-level duty (25 %), key-down ≤ 5 s, VSWR 3:1 fold-back to 0.5 W then a 10-minute stop, 70 °C hold (`software/flight/include/downlink.h`) |

### 3.4 Repeat path and `XPDR_EN`

As BUILD_VS_BUY §3.2: discriminator audio → 300 Hz high-pass (strips the tone), 3 kHz low-pass, de-emphasis then
pre-emphasis, limiter → mute → audio select. `XPDR_EN` from the OBC is a ≈ 100 Hz heartbeat into a retriggerable
monostable (≈ 50 ms), so an OBC that hangs, resets or browns out drops the transponder within ≈ 50 ms. The key gate is a
hardware AND of the heartbeat, CTCSS valid, and the 120 s key-down timer. The OBC's gates (ACTIVE, sunlit, SOC ≥ 55 %,
PA < 70 °C, keyed < 25 % of 100 min) decide when the heartbeat runs (`sqso_fsw::transponder`; TRANSMITTER_DESIGN §10.1).

**Firmware (2026-10-27):** the heartbeat is `hal_gpio_heartbeat()` (`software/flight/include/hal/hal_gpio.h`;
`software/rust/firmware/src/heartbeat.rs`): the TIM7 interrupt toggles PE4 to make a 100 Hz square wave for a 40 ms
lease (SRS-F081; 60 ms until flight software round 2) that the main loop renews each pass, so a stalled loop, a hard fault or a reset stops the edges. A level held high
(no edges) is treated by the radio board as disabled. Not run on hardware (VI-RF-11).

**Board (2.1, SQSO-RF-BRD-001 §8):** SN74LVC1G123 retriggerable monostable, 226 kΩ × 0.22 µF = 49.7 ms (46.8–58.0 ms),
100 kΩ pull-down on the input; CMX138A CTCSS decoder and 300–3000 Hz filter (configured by the TNC MCU, which relays its
tone status as TONE_OK); CD74HC4060 120 s key-down timer reset by the tone dropping; SN74LVC1G11 AND; TMUX1101 mute;
TMUX1219 2:1 select; readbacks TONE_OK, XPDR_KEY, PTT_EXCITER on W4 J1-8/9/10 to OBC inputs. Unkeyed ≤ 59 ms after
the last edge; ≤ 98 ms after a stalled main loop (40 ms lease + 58 ms; 90 ms nominal), inside VI-RF-11's 100 ms.

### 3.5 TX-OFF and inhibits

| Inhibit | Hardware | Driven by | Counts as a CDS 2.3.7 physical RF inhibit? |
|---------|----------|-----------|---------------------------------------------|
| Separation (deployment) switches | Switches in the battery-to-bus path | The dispenser | Yes |
| Remove-before-flight pin | In the same path | Ground crew | Yes |
| `RADIO_EN` | Load switch on the radio supply | OBC (RF silence; pulled down) | Only if its supply cannot be enabled before deployment by any software path; the power/safety reviewer to confirm against the inhibit schematic (finding VV-03) |
| `RF_TX_EN` | Load switch on the PA supply | OBC (TX-OFF, loss of contact, RF silence; pulled down) | Same condition |

**TX-OFF honestly described.** TX-OFF depends on the OBC: the radio's TNC delivers the frame, the OBC authenticates it
and drives `RF_TX_EN` low, then latches it. There is no second command decoder in the radio. A hung OBC is reset by its
independent watchdog (≈ 32 s), and during a reset the pull-down holds the PA off. The IARU draft's item 7.8 claimed "a
second, independent path"; that text is corrected (SQSO-REG-010 Rev A.2).

### 3.6 Antennas

**Baseline (BASELINE §4, §7, Rev E):** 2 m elements along ±X; the board feeds the 2 m pair (±X) and the 70 cm pair (±Y) in antiphase through lumped baluns (SQSO-RF-BRD-001 §9). The options below are the review's analysis.

ISIS AntS 1U–3U on one ±Z face, tapes deploying in the plane of the face (each along ±X or ±Y), redundant thermal-knife
release, deployment switches, I²C. Configuration choice (finding RF-03): with the spin about +Y (BASELINE §8),

| Option | Result (SQSO-LBA-001 §5) | Recommendation |
|--------|--------------------------|----------------|
| One 2 m tape across the spin axis (±X) | At 8°/s, 8.3 fades below −10 dBi per pass, 5 s each; 96.6 % of beacon frames at station A (98.2 % with the beacon jitter as built) | Not preferred |
| One 2 m tape along the spin axis (+Y) | No spin modulation; 0.23 long fades per pass; 100 % of frames at station A | **Baseline if one 2 m tape** |
| Two 2 m tapes, X and Y, quadrature hybrid | Never below −6 dBi; 100 % at A, 72 % (from 49 %) at a handheld Yagi | **Preferred if the AntS can carry two 2 m and one or two 70 cm tapes with phasing** (RFQ question) |

The 70 cm element follows the same rule (along +Y, or crossed). The wings (normal +Y) sit near the elements and will
distort the pattern; the EM pattern measurement with wings deployed is VI-RF-01.

## 4. Power

| State | DC power (allocation → design, SQSO-RF-BRD-001 §5) | Basis |
|-------|----------|-------|
| RF silence | 0 (radio unpowered) | RF-F005 |
| Receive (always) | ≤ 0.15 W → 0.102 W typical, 0.127 W worst | BASELINE §5 |
| Repeat path enabled, not keyed | ≤ 30 mW → 24 mW typical, 31 mW worst | RF-P015 |
| Beacon at 0.5 W (≈ 1.2 s per 60 s) | 1.85 W → 1.84 W while keyed; 0.035 W average | SIL: 1,431 frames/day, 1,756 s keyed/day |
| Transponder at 0.5 W | 1.85 W → 1.84 W while keyed; ≤ 25 % duty (SIL: 57–80 min keyed a day) | BASELINE §5 |
| 1 W | ≈ 2.7 W → 2.42 W while keyed (29.5–30.1 dBm) | PA saturated at 5 V |

## 5. Findings Against the Baseline (for the chair)

*Status (2.5): the 3rd-harmonic, uplink and spurious rows are adopted in BASELINE Rev E §7 (≥ 70 dB at the antenna port; uplink windows ≥ 0.9 MHz below 3 × f_down; 97.307(e) text). The table is kept as the record.*

| Item | Baseline text | Finding | Recommended change |
|------|---------------|---------|--------------------|
| 3rd-harmonic target | "Target ≥ 60 dB 3rd-harmonic suppression" (BASELINE §7) | Costs 3.2 dB of uplink sensitivity at 1 W with the planning frequencies (SQSO-LBA-001 §7.3) | "≥ 70 dB, measured at the antenna port, with the uplink ≥ 2 MHz below 3 × f_down" |
| Uplink choice | "chosen away from 3 × f_downlink" | 3 × (145.800–146.000) = 437.400–438.000 MHz | "in 435.000–437.300 MHz" |
| Spurious requirement | SYS-203 "at least 40 dB below carrier" | 97.307(e): ≤ 25 µW and ≥ 40 dB, i.e. ≥ 46 dBc at 1 W | Amend SYS-203 to the rule text |
| Antenna requirement | Proposed SYS-222 "−6 dBi over 90 % of the sphere" | A single monopole reaches −6 dBi over 81 % only | RF-P013 wording |

## 6. Verification

The verification items VI-RF-01 to VI-RF-10 are defined in SQSO-LBA-001 §8. Items specific to this design:

| ID | Test and pass criterion | Verifies | When |
|----|-------------------------|----------|------|
| VI-RF-11 | `XPDR_EN` fail-safe: stop the heartbeat high, low, high-Z and with the OBC in reset; repeat path unkeys within 100 ms in every case; key-down timer cuts a stuck carrier at 120 ± 5 s | RF-F003, RF-F008 | EM |
| VI-RF-12 | TX-OFF at the antenna port: `TX_DISABLE` from a telecommand station simulator; RF at the port < −60 dBm within 1 s; PA supply < 0.1 V; persists across resets (TP-RF-006) | RF-F004, SYS-204 | EM, FM |
| VI-RF-13 | RF silence: spectrum analyzer across 0–50 min after a simulated deployment with a reset at 20 min; no emission > −60 dBm before 45 min, and the GNSS receiver unpowered (GNSS_EN low) before 45:00 (SYS-224 Rev G covers every emitter, the GNSS receiver included) (TP-RF-006 §8.8) | RF-F005, SYS-224 | EM, FM |
| VI-RF-14 | Frequency stability, ageing and offset estimation, both bands:<br>(a) ±1.0 ppm about the calibrated offset from −40 to +70 °C (2.4: was −20 to +60 °C);<br>(b) ageing of the flight TCXO lot measured (≥ 30 days at +60 °C plus the vendor's ageing data), extrapolated total offset ≤ ±3.0 ppm at 2 years;<br>(c) ground offset estimation: the station measures the 2 m beacon offset (Doppler removed) within ±20 Hz, scales it by 435.5/145.9, and the pre-corrected uplink arrives within ±1 kHz with the reference set at ±3 ppm (TCXO pulled, or a signal generator substituted);<br>(d) the same in orbit on the first passes (commissioning) | RF-P005 | EM; commissioning |
| VI-RF-15 | Occupied bandwidth: 99 % bandwidth ≤ 10.4 kHz (beacon) and ≤ 16 kHz (transponder, 1 kHz at full deviation) | RF-R005 | EM, FM |
| VI-RF-16 | Antenna deployment with the AntS on the flight face, thermal vacuum cold case; VSWR after deployment | RF-P014, SYS-220 | EM, FM (TP-RF-005) |

## 7. Risks

| ID | Risk | L | C | Mitigation |
|----|------|---|---|------------|
| RF-K1 | No flight-proven U/v FM repeater; the built radio slips the schedule | M | H | EM of the flight-design RF board with the TNC firmware from development boards (`mission/SCHEDULE.md` TNC_UT, TNC_EM); RFQs to ISIS (TRXUV) and LETSCOM (BUILD_VS_BUY §3.1) as fall-backs |
| RF-K2 | Desense from the 3rd harmonic or PIM at the AntS hinges | M | M | Frequency plan (RF-P002), 70 dBc, VI-RF-04, VI-RF-06 |
| RF-K3 | Pattern nulls and wing blockage worse than modelled | M | M | Element along +Y or crossed (§3.6), VI-RF-01 |
| RF-K4 | Terrestrial interference at 435 MHz above +6 dB | M | M | 50 W telecommand option; command at higher elevations; VI-RF-09 |
| RF-K5 | IARU assigns frequencies close in harmonic | L | M | State RF-P002 in the request |
| RF-K6 | Module parts (SA818 class) fail radiation or thermal tests | — | — | Retired (2.5): no module parts; the EM uses the flight-design board (ETP Rev C §2) |

## 8. Ground Segment

See [GROUND_STATION_DESIGN.md](../ground/GROUND_STATION_DESIGN.md) (SQSO-GND-001 Rev H) and
[ICD_SPACE_GROUND.md](../interfaces/ICD_SPACE_GROUND.md) (SQSO-ICD-005 v2.7). Beacon format:
[BEACON_SPEC.md](../BEACON_SPEC.md).

## 9. Selected and Designed; What Remains Open (closure path)

Version 2.1: the board design is SQSO-RF-BRD-001 (`hardware/electrical/rf/RF_BOARD_DESIGN.md`, with `design.py`,
the netlist, the BOM and the ERC log). What it leaves open:

| Item | State | Closure |
|------|-------|---------|
| 435 MHz receiver and RX filter | Designed: FL1 + BFP640F + TA2209A + CC1200 (CFM) | EM: VI-RFB-01 to -03, VI-RF-08 |
| KISS TNC hardware | Designed: STM32L431; firmware is a new software item | Firmware development under the SDP; VI-RF-03 |
| 2 m exciter and PA | Designed: LMX2571 + PCM1808, RD01MUS3; DC at 0.5 W 1.84 W, inside the 1.85 W allocation of BASELINE §5 | EM load-pull VI-RFB-04 |
| Repeat path, CTCSS decoder, heartbeat monostable, key-down timer, audio select | Designed (SQSO-RF-BRD-001 §8) | VI-RF-11, VI-RFB-07, VI-RFB-08 |
| TCXO | Selected: TG2016SMN 40 MHz ±0.5 ppm; 2.1 ppm worst after 2 years with ageing, against RF-P005 as amended (≤ ±3.0 ppm); ground-side uplink correction from the measured downlink offset (RFB-05) | VI-RF-14 |
| AntS configuration (elements, faces, phasing) | Baseline: 2 m pair ±X, 70 cm pair ±Y, antiphase from on-board lattice baluns (or ISIS phasing) | RFQ to ISIS; VI-RF-01, VI-RF-17 |
| RF board layout, shielding, thermal path | Rules defined (SQSO-RF-BRD-001 §10) | EM layout and review |
| Inhibit schematic (which switches are physical RF inhibits) | EPS_DESIGN 2.6 §8 (DS1–DS3) | EPS design |
| `XPDR_EN` heartbeat | Firmware implemented (40 ms lease, SRS-F081); board monostable designed | VI-RF-11 |
| Readbacks TONE_OK, XPDR_KEY, PTT_EXCITER | On W4 J1-8/9/10; OBC pins not yet allocated | OBC design (RID RF-21) |

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release (beacon-only) |
| 1.1 | 2026-10-16 | SMART-QSO Team | §9.2 test IDs aligned with the procedure files (finding VV-18) |
| 1.2 | 2026-10-16 | SMART-QSO Team | §2.4 vibration: interim SpaceX UG v11 level (finding LA-05) |
| 1.2, note | 2026-10-24 | SMART-QSO Team | §3.3.2 names the Rust file that replaced `rf_silence.c` |
| 2.0 | 2026-10-27 | CDR review, communications panel | Rewritten for the baseline U/v transponder: receiver, TNC, transmitter, repeat path and `XPDR_EN`, TX-OFF and inhibits, AntS configuration options from the spin analysis; requirements RF-F001–F008, RF-P001–P015; findings for the baseline; verification items VI-RF-11 to VI-RF-16; what is not yet selected. The beacon-only design, its link budget and the AX.25 appendices are removed (BEACON_SPEC and SQSO-LBA-001 govern) |
| 2.1 | 2026-10-27 | CDR design completion, RF board | Parts selected and the board designed (SQSO-RF-BRD-001): §3 "as designed" notes (receiver CC1200 with the audio from the TNC MCU, LMX2571/PCM1808 exciter, RD01MUS3 PA, filters), §3.4 heartbeat lease and board gate, §4 power from the design, §9 replaced by what remains open; RF-P007 split into before-LNA and before-receiver-IC parts, RF-P010 and RF-P015 annotated with the design values, RF-P012 timing restated (200 ms typical, 260 ms maximum) |
| 2.2 | 2026-10-27 | CDR 2026 consistency | RF-P002: uplink ≥ 0.9 MHz below 3 × f_down (was 2 MHz), from the RF board's desense result; the requested windows |
| 2.3 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | Element-option rows at 8°/s and 475 km (SQSO-LBA-001 v3.2 §5) |
| 2.3 (amended) | 2026-10-27 | CDR round 3 (P3A-05, RID RFB-05) | RF-P005: ±1.0 ppm over temperature and ≤ ±3.0 ppm with 2-year ageing (TG2016SMN worst 2.1 ppm); the ground corrects the uplink from the measured downlink offset. VI-RF-14 adds ageing and offset estimation |
| 2.3 (amended) | 2026-10-27 | CDR round 3 (P3A-07) | §3.4 and §9: the XPDR_EN heartbeat lease is 40 ms (SRS-F081; was 60 ms), a stalled loop unkeys within 98 ms (90 ms nominal), inside VI-RF-11's 100 ms; VI-RF-13 also checks the GNSS receiver unpowered (GNSS_EN low) before 45:00 (SYS-224 Rev G) |
| 2.4 | 2026-10-27 | CDR parts temperature audit (SQSO-MFG-003) | Verification temperature range −40 … +70 °C (RF-P003, §2.4, VI-RF-14), was −20 … +60 °C; Q601 minimum operating temperature path (SQSO-RF-BRD-001 F VI-RFB-14) |
| 2.5 | 2026-10-27 | Documentation audit (post-CDR 2026) | RF-P002 uplink to the BASELINE Rev E windows inside the board's 435.050–436.500 MHz band (was 435.000–437.300 MHz); RF-P008 3rd harmonic per BASELINE §7; RF-P015 and §4 transmitter allocation 1.85 W and design 1.84 W / 2.42 W (were 1.5 W and 1.76 / 2.34 W), receiver 0.102 / 0.127 W; §5 findings marked adopted; RF-K1 and RF-K6 without the SA818 engineering model; references SQSO-RF-BRD-001 F, LBA 3.5, ICD-003 2.4, GND-001 Rev H, ICD-005 2.7, EPS_DESIGN 2.6, BASELINE Rev E |
| 2.6 | 2026-10-27 | Documentation audit (post-CDR 2026), follow-up | RF-P003 1 W level 29.5–30.0 dBm, never above 1 W (SYS-201); selected-design row: 0.50 W at 1.84 W DC (SQSO-RF-BRD-001 E; was 1.76 W) and the APC holding 30.0 dBm |
