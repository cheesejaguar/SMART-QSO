# RF Board TNC Firmware: Software Requirements Specification

**Document Number**: SQSO-SRS-TNC-001
**Version**: A
**Date**: 2026-10-27
**Status**: Draft (CDR). Requirements of a Class C software item that is not yet written (design starts with the
schedule task TNC_DES, `mission/SCHEDULE.md`). Values follow `docs/BASELINE_PARAMETERS.md` Rev E, which governs.
**Software item**: RF board TNC firmware, on the STM32L431CCT6 (U310) of the RF board SQSO-RF-BRD-001
**Classification**: NPR 7150.2D Class C (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1, §3.4); safety-critical
requirements flagged per NASA-STD-8739.8B §4.2 from `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md` (SQSO-SHA-001 G,
SW-HAZ-014 and SW-HAZ-015)
**Closes**: CDR finding P3A-03 (TNC part) and NPR 7150.2D SWE-050 for this item (compliance matrix open item C-7)

---

## 1. Purpose and Scope

This specification gives the requirements of the firmware of the RF board's TNC microcontroller. The firmware is
software on the spacecraft but not OBC flight software: it runs on its own MCU on the RF board, talks to the OBC only
over the KISS link on USART2 and a few discrete lines, and configures the board's three radio ICs.

Its functions (SQSO-RF-BRD-001 §13):
- the KISS host interface to the OBC, with the TNC's own STATUS query;
- HDLC framing with FCS-16, NRZI and bit stuffing;
- the AFSK 1200 demodulator on the CC1200's analog-FM (CFM) samples and the AFSK modulator on DAC1;
- the repeat audio of the FM transponder (CC1200 samples to DAC2) and the TONE_OK relay from the CMX138A;
- configuration and read-back scrubbing of the CC1200, CMX138A and LMX2571, with the frequency trims from flash;
- the receiver supervision behind RX_FAULT (the RX_ALIVE pulses and their six criteria) and the carrier sense on RX_CD;
- the independent watchdog.

Not in scope: the OBC's side of the interface (SMART-QSO-SRS-001: SRS-F081 heartbeat, SRS-F085 downlink gate,
SRS-F115, SRS-F116 and SRS-F152 receiver evidence, SRS-P002 key-down and duty), the board hardware, and the ground
software that builds telecommands.

## 2. Applicable Documents

| Document | Title | Used for |
|----------|-------|----------|
| SQSO-BL-001 Rev E | Baseline parameters (`docs/BASELINE_PARAMETERS.md`) | §7 RF values; governs |
| SQSO-RF-BRD-001 F | RF board build-to design (`hardware/electrical/rf/RF_BOARD_DESIGN.md`) | §4.1 J1, §6.4 clock plan, §6.5 trims, §7.1 deviation, §8 repeat path and gate, §8.5 TX permit, §8.6 supervision, §12 scrubbing, §13 firmware duties, §14 verification items |
| `hardware/electrical/rf/design.py` | RF board schematic as code | MCU pin allocation (U310), CC1200 PA pin (L206, C208 not fitted), LMX2571 MICROWIRE buffer U503 |
| SMART-QSO-RF-001 2.4 | RF subsystem design (`docs/hardware/RF_DESIGN.md`) | RF-F001 to RF-F008, RF-P001 to RF-P015, RF-R001 to RF-R005 |
| SQSO-ICD-003 2.3 | OBC to RF subsystem ICD (`docs/interfaces/ICD_OBC_RF.md`) | §4.2 signals, §5 KISS, §6 timing, §7 tests |
| SQSO-ICD-013 0.13 | Telecommand authentication and TX-OFF (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`) | §2 receiver contract, §3 frame, §6 TX-OFF |
| SQSO-ICD-005 2.5 | Space-to-ground ICD (`docs/interfaces/ICD_SPACE_GROUND.md`) | §4 RF interface, §5 AX.25 |
| SQSO-LBA-001 3.5 | Link budget (`docs/environment/LINK_BUDGET_ANALYSIS.md`) | §3 demodulator thresholds |
| SMART-QSO-SRS-001 1.10 | Flight software requirements (`docs/software/SOFTWARE_REQUIREMENTS.md`) | The OBC side: SRS-F081, SRS-F085, SRS-F115, SRS-F116, SRS-F152, SRS-P002 |
| `software/flight/include/downlink.h` | The OBC's downlink limits as built | DOWNLINK_MAX_FRAME_LEN 328, DOWNLINK_MAX_KEYDOWN_MS 5000, DOWNLINK_DUTY_PERCENT 25 with a 60 s window |
| `software/rust/sqso-protocol/src/kiss.rs` | KISS codec shared by the flight, payload and ground software | Command codes, SetHardware STATUS (0x06/0x01, answer 0x81, `TncStatus`) |
| SQSO-SHA-001 G, SQSO-FMEA-003 B | Software hazard analysis, board FMEA | SW-HAZ-014, SW-HAZ-015; BF-RF-07, -08, -25, -26, -32; SMA-B11 |
| SMART-QSO-SDP-001 1.6 | Software development plan | §3.4: the item, its standards and verification |

## 3. Context

### 3.1 Where the firmware sits

```
 OBC USART2 TX (PD5) ──J1-2──> PA3 USART2 RX ┐                     ┌─ PA8 TNC_PTT ──┐
 OBC USART2 RX (PD6) <──J1-3── PA2 USART2 TX ┤   TNC firmware      │                 ├─ PTT_EXCITER = TX_PERMIT_HW · (TNC_PTT + XPDR_KEY)
 CC1200: SPI1 8 MHz (PB0 CSn), GPIO0 CLKEN_CFM ─> PB10, GPIO2 CARRIER_SENSE ─> PB11 (and RX_CD J1-15)
 CMX138A: SPI1 1 MHz (PB1 CSn), IRQN ─> PA6                 PA7 TONE_OK ──> key gate (EN_OK · TONE_OK · NOT_TOT), J1-8
 LMX2571: MICROWIRE write-only through U503 (PB2 LE), MUXout = LMX_LD ─> PB14 (and the hardware PA enable)
 DAC1 PA4 ─> TNC audio (AFSK)    DAC2 PA5 ─> repeat audio ─> 4 kHz LPF ─> CMX138A ─> mute ─> 2:1 select ─> PCM1808 ─> LMX2571
 Inputs: PA1 TX_PERMIT_HW, PB13 EN_OK, PB15 XPDR_KEY, PB6 PTT_EXC      Output: PB7 RX_ALIVE ─> U481/U482 ─> RX_FAULT J1-16
```

Pins as allocated in `hardware/electrical/rf/design.py` (U310; LQFP-48 numbers to confirm at the schematic review,
SQSO-RF-BRD-001 §16). The MCU runs from HSI16 (no crystal); its debug port is a Tag-Connect SWD footprint and a debug
UART on USART1 (test points).

### 3.2 The telecommand path: the TNC only demodulates and passes frames

The TNC is a modem. It demodulates the uplink, checks the AX.25 FCS and hands every good frame to the OBC; the OBC
checks the FCS again, filters by address, UI and PID, and authenticates (HMAC-SHA-256 with the replay counter) before
any command acts (SQSO-ICD-003 §5.4, SQSO-ICD-013 §3 and §8). The TNC holds no key, decides nothing from a frame's
content and has no command decoder of its own (SMART-QSO-RF-001 §3.5: "There is no second command decoder in the
radio"). TX-OFF is the OBC driving RF_TX_EN low, which removes PA_5V on the EPS board; the TNC is in that path only as
the receiver of the TX_DISABLE frame. So the TNC's duties toward TX-OFF are to pass every frame (TNC-F012) and to stay
alive (TNC-F004).

### 3.3 Functions enforced in hardware, which this firmware must not be relied on for

The firmware cannot defeat any of these, and no requirement below may be read as making them depend on it:

| Function | Hardware | Source |
|----------|----------|--------|
| TX permit: no PA supply, no exciter supply and no CE without PA_5V; two independent paths (TX_PERMIT_HW into the PTT gate, PA_5V divider to U805's enable) | SW3 on the EPS (RF_TX_EN, TX-OFF, RF silence, loss of contact, deployment switches), U804, U805 | SQSO-RF-BRD-001 §8.5 |
| Transponder key gate: XPDR_KEY = EN_OK · TONE_OK · NOT_TOT | LVC1G123 monostable on the OBC heartbeat (≤ 58 ms), LVC1G11 AND | §8.1, §8.2 |
| Repeat key-down timer 120 ± 4.6 s, reset by the tone dropping | CD74HC4060 with its latch | §8.2 |
| Hard deviation limit ±6.3 kHz (ADC full scale); repeat band-limit 300–3000 Hz and level limit | PCM1808 clipping; CMX138A filter and limiter | §7.1, §8.3 |
| PA enabled only with PTT_EXCITER AND the synthesizer's lock detect | U733 | §7.1 |
| Power level 0.5 / 1 W and the APC loop | TX_PWR_SEL from the OBC, U731, U735 | §7.2 |
| RX_FAULT high when the RX_ALIVE pulses stop, whatever the firmware does (a hung or dead TNC reads as a fault) | U481 monostable, U482 | §8.6 |
| TNC_PTT and TONE_OK low with the MCU in reset | R408 and the TONE_OK pull-down | §8.2, `design.py` |
| VSWR, PA temperature and 25 % duty protection | **Not on this board and not in this firmware**: the OBC reads TX_FWD, TX_REV and TX_TEMP (the TNC has none of them) | SQSO-ICD-003 §4.2, §5.3; `downlink.h` |

What the hardware does **not** cover, and the firmware therefore carries (SQSO-SHA-001 G §9.8.1, SW-HAZ-014):
- the beacon path TNC_PTT bypasses EN_OK and the 120 s timer (truth table, §8.4): only RF_TX_EN bounds it, so the
  key-down and duty backstops (TNC-F024, TNC-F025) are firmware;
- the CC1200 is a transceiver on RADIO_3V3, which TX-OFF does not remove: it must never be put in transmit (TNC-F041);
- the transmit frequency is the LMX2571 register image the firmware writes, and the board has no read-back path for it
  (TNC-F042).

## 4. Conventions

**Identifiers.** TNC-Fnnn functional, TNC-Innn interface, TNC-Pnnn performance, TNC-Cnnn constraints. Every row is a
requirement ("shall"), defined once. The "Parent" column names the requirement it flows from where one exists
(SMART-QSO-RF-001 RF-…, SMART-QSO-SRS-001 SRS-…); "derived" rows have no parent and give their reason.

**Verification methods.** T test, A analysis, I inspection, D demonstration.

**Verification activities** (the schedule tasks of `mission/SCHEDULE.md` in brackets):
- **UT**: host unit tests in CI, the firmware's logic crate compiled for the host, with the `sqso-protocol` tests and
  fuzz targets (task TNC_UT);
- **DB**: bench test on an STM32L431 development board with a CC1200 evaluation board (task TNC_UT);
- **EM**: RF board engineering model, with the board verification items of SQSO-RF-BRD-001 §14 (task TNC_EM);
- **FS**: EM integration with the OBC flatsat (TP-FLATSAT-001; task TNC_EM, before EM_TEST);
- **FQ**: qualification of the flight build on the FM RF board, re-running the EM, FS and VI-RFB-15 temperature
  cases on the flight hardware (task TNC_QUAL);
- **IN**: inspection of the code, the build and the release image.

**Safety-critical (SC).** "Yes" with the hazard where the requirement implements a hazard control of SQSO-SHA-001 G.
The modules that implement them take the safety-critical coverage rule (TNC-C005).

## 5. Requirements

### 5.1 Start-up, clocks and watchdog

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F001 | From the release of any reset (power-up of RADIO_3V3, watchdog or other), the firmware shall have the CC1200 receiving on the configured uplink frequency and USART2 accepting KISS frames within 50 ms | RF-F002 | Receiver ready ≤ 50 ms after RADIO_EN (MCU boot ≈ 5 ms, CC1200 set-up ≈ 10 ms, TCXO ≤ 2 ms): SQSO-ICD-003 §6, SQSO-RF-BRD-001 §15 item 12 | T | EM: RADIO_EN to the first CFM sample interrupt and the first accepted byte, 10 starts at −40, +25 and +70 °C | — |
| TNC-F002 | Until its initialisation is complete, the firmware shall hold TNC_PTT (PA8), TONE_OK (PA7) and RX_ALIVE (PB7) low, hold DAC1 and DAC2 at mid-scale (1.55 V), and configure every pin not used by §3.1 as an analog input | RF-F003 | No key, no tone report and no health pulse before the radio ICs are configured; both audio sources are biased at 1.55 V so the TMUX1219 select makes no step (SQSO-RF-BRD-001 §8.2); unused MCU pins configured analog in firmware (`design.py` ERC waivers) | T, I | UT (start-up sequence); EM: logic analyser on PA7, PA8, PB7 through 100 power cycles | Yes (SW-HAZ-014) |
| TNC-F003 | The firmware shall emit no RX_ALIVE pulse earlier than 100 ms after any reset | SRS-F116 | TNC-FW-POR: the OBC's power-on self-test of RX_FAULT reads the line 10–90 ms after RADIO_EN and expects it high (SQSO-RF-BRD-001 §8.6, §13) | T | EM: VI-RFB-10 (no RX_ALIVE before 100 ms); FS with SRS-F116 | Yes (SW-HAZ-015) |
| TNC-F004 | The independent watchdog (IWDG, on the LSI) shall be started in hardware mode by the option bytes at every reset, with a timeout of 250 ms nominal and at most 0.35 s with the LSI's tolerance, and shall be refreshed only from the main loop after that pass's health checks, never from an interrupt | — (derived) | A hung TNC with TONE_OK or TNC_PTT high keys at most until its watchdog resets it, "≤ 0.25 s" (SQSO-RF-BRD-001 §8.3; BF-RF-08); hardware mode means no firmware path can stop it | T, I | IN: option bytes of the release image; EM: main loop halted by the test build, reset within 0.35 s with TNC_PTT and TONE_OK low | Yes (SW-HAZ-014, SW-HAZ-015) |
| TNC-F005 | After any reset the firmware shall start as from power-up (unkeyed, TONE_OK low, KISS parameters at their defaults, queued frames discarded), record the reset cause, and count the watchdog resets since power-up for the STATUS answer | — (derived) | A reset must never resume a transmission or a tone report; the count tells the ground about hangs (SQSO-FMEA-003 BF-RF-08) | T | UT; EM: watchdog reset during a transmission | Yes (SW-HAZ-014) |
| TNC-F006 | The firmware shall run the core and buses from HSI16 at 16 MHz, SPI1 at 8 MHz for the CC1200 and the LMX2571 and at 1 MHz for the CMX138A, with no clock output (MCO off) and no other clock frequency in the flight build | — (derived) | The board's clock plan puts no harmonic of these clocks in 435.000–437.300 MHz except the CMX138A's 1 MHz SPI (SQSO-RF-BRD-001 §6.4) | I, T | IN: clock configuration; EM: receiver noise floor with the firmware running and halted (no line in the uplink window) | — |
| TNC-F007 | The firmware shall keep SPI1 at 1 MHz (CMX138A accesses) to at most 1 ms in any 10 s | — (derived) | 1 MHz SPI harmonics fall in the uplink band (436 MHz); the clock plan allows ≤ 1 ms per 10 s (SQSO-RF-BRD-001 §6.4) | A, T | UT (scheduler budget); EM: logic-analyser capture over 1 h | — |

### 5.2 KISS host interface (USART2)

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-I001 | The firmware shall run USART2 (PA2 TX, PA3 RX) at 9600 baud, 8 data bits, no parity, 1 stop bit, full duplex, without flow control | — | SQSO-ICD-003 §4.2, §5 | T | UT; EM: loop test through the EM TNC (SQSO-ICD-003 §7 "KISS link") | — |
| TNC-I002 | The firmware shall frame and unframe KISS (FEND 0xC0, FESC 0xDB, TFEND 0xDC, TFESC 0xDD) in both directions, on port 0 only, and ignore frames for any other port | — | SQSO-ICD-003 §5.1; the codec of `software/rust/sqso-protocol/src/kiss.rs` | T | UT and the fuzz target `kiss_stream` | — |
| TNC-I003 | For each KISS data frame (command 0x00) of 1 to 328 bytes from the OBC, the firmware shall transmit that frame once, in order of arrival, adding the flags, FCS, bit stuffing, NRZI and AFSK; a data frame longer than 328 bytes after unescaping shall be discarded whole and counted | RF-F001 | The OBC sends the AX.25 frame from the first address byte through the information field, without the FCS; largest 328 bytes (SQSO-ICD-003 §5.1; DOWNLINK_MAX_FRAME_LEN) | T | UT; EM: frames of 1 to 328 bytes with every escape case, byte-exact at a reference AX.25 decoder (SQSO-ICD-003 §7) | — |
| TNC-I004 | The firmware shall apply the KISS parameter commands TXDELAY (0x01), P (0x02), SLOTTIME (0x03), TXTAIL (0x04) and FULLDUPLEX (0x05) to the next transmission, and set them at every reset to TXDELAY 30, P 255, SLOTTIME 0, TXTAIL 1, FULLDUPLEX 1 | — | SQSO-ICD-003 §5.2 (the OBC sends them before each frame, because the radio may have been off) | T | UT | — |
| TNC-I005 | The firmware shall apply a TXDELAY above 50 (500 ms) as 50, a TXTAIL above 5 (50 ms) as 5, and FULLDUPLEX 0 as 1 | — (derived) | A corrupted parameter byte must not lengthen a key-down; the downlink must never wait for the uplink carrier (SQSO-ICD-003 §5.2, FULLDUPLEX row); with these bounds the longest legitimate transmission is 3.2 s (TNC-F024) | T | UT | Yes (SW-HAZ-014) |
| TNC-I006 | On a SetHardware frame (command 0x06) with sub-command STATUS (0x01), the firmware shall answer within 100 ms with a SetHardware frame, sub-command 0x81, carrying in order: firmware version (1 byte), the six supervision criteria (bit n set = criterion n+1 met), the noise floor and its calibrated value (dBm, signed byte each), the carrier-sense count (16 bits), the frames decoded (16 bits) and the uptime (s, 32 bits), multi-byte fields big-endian, followed by the extension of TNC-I007 | SRS-F116 | TNC-FW-STAT, the OBC–TNC loop query every 10 min that tests both USART directions (SQSO-RF-BRD-001 §8.6, §13); layout of `TncStatus` in `software/rust/sqso-protocol/src/kiss.rs` | T | UT (encoded answer decoded by `TncStatus::decode`); EM: VI-RFB-10 (answered within 100 ms); FS with SRS-F116 | Yes (SW-HAZ-015) |
| TNC-I007 | The STATUS answer shall continue after the 12 bytes of TNC-I006 with, in order: the key-down limit trips, the duty backstop trips, the frames from the OBC discarded, the frames not keyed for want of TX permit or lock, the configuration scrub corrections and the watchdog resets since power-up (1 byte each, saturating), the trim and calibration record flags (1 byte: bit 0 trim record invalid, bit 1 noise-floor table invalid), and the image CRC-16 (2 bytes, big-endian); the whole answer after the command byte shall be at most 24 bytes | — (derived) | Ground diagnosis of the firmware's own limits; the OBC's decoder ignores trailing bytes and keeps at most 24 (`HW_MAX`) | T | UT | — |
| TNC-I008 | The firmware shall ignore every SetHardware sub-command other than 0x01, the KISS return command (0xFF) and every unknown command byte, with no effect on the radio; no command shall change a frequency, a trim, the calibration record or the flash | RF-P005 | The frequency is corrected on the ground from the measured downlink offset, and the re-trim command is not required (chair decision RFB-05; SQSO-RF-BRD-001 §6.5; SDP §3.4: no telecommand re-trims the frequency) | T | UT and fuzz (every command byte and sub-command) | Yes (SW-HAZ-014) |
| TNC-I009 | A USART framing, noise or overrun error inside a KISS frame shall discard that frame; the decoder shall resynchronise at the next FEND and count the error | — | SDP §3.4 (behaviour on USART framing errors) | T | UT; EM with injected line errors | — |
| TNC-I010 | The firmware shall hold at least 2 data frames from the OBC waiting for transmission; a frame arriving with the queue full shall be discarded and counted | — (derived) | The OBC sends one frame at a time (SQSO-ICD-003 §5.3); the queue covers the parameter commands sent before each frame | T | UT | — |
| TNC-I011 | For each frame accepted by TNC-F011, the firmware shall send the OBC a KISS data frame on port 0 (command 0x00) with the frame from the first address byte through the information field, without the FCS, in order of reception, starting within 100 ms of the closing flag; it shall buffer at least 2 such frames and count any discarded for a full buffer | RF-F002 | SQSO-ICD-003 §5.1 (the OBC appends the FCS again and checks it a second time); every uplink frame, TX_DISABLE included, reaches the OBC (SQSO-ICD-013 §2, §6) | T | UT; EM: telecommand frames received and delivered byte-exact; FS: TX_DISABLE over the air to the OBC | Yes (SW-HAZ-015) |
| TNC-I012 | The firmware shall send a STATUS answer only between whole KISS frames, never inside one | — | The OBC decodes data and SetHardware frames from the same byte stream (`HardwareDecoder`, `Decoder` in `kiss.rs`) | T | UT | — |
| TNC-I013 | In the flight build USART1 shall be disabled (its pins analog) and no input other than those of §3.1 shall change the firmware's behaviour | — (derived) | The debug UART is a ground test point; no unauthenticated input path in flight (NPR 7150.2D SWE-157) | I | IN: pin configuration of the release image | — |

### 5.3 HDLC and AX.25 framing

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F010 | For each transmission the firmware shall send HDLC flags (0x7E) for TXDELAY, then the frame and its FCS (CRC-16/X.25, low byte first) with a 0 stuffed after five consecutive 1s, NRZI-coded (a 0 is a change of tone), then flags for TXTAIL and at least one closing flag | RF-F001 | SQSO-RF-BRD-001 §13; SQSO-ICD-005 §4.1, §5.1 (AFSK Bell 202, NRZI, HDLC; FCS CRC-16/X.25) | T | UT against the `sqso-protocol` CRC and reference vectors; EM: decoded by a reference AX.25 software decoder | — |
| TNC-F011 | The firmware shall NRZI-decode the demodulated bits, find the flags, remove the stuffed bits, check the FCS, and accept only frames of 17 to 330 bytes (FCS included) with a whole number of octets and a valid FCS; it shall discard any other frame and any frame ended by an abort (seven 1s), and count them | RF-F002 | SQSO-ICD-013 §2 (frames with flags and stuffing removed, FCS checked); AX.25 maximum 330 bytes (`MAX_FRAME_LEN` in `ax25.rs`), minimum two addresses, control and FCS | T | UT and fuzz (random bit streams, every length) | — |
| TNC-F012 | The firmware shall pass every frame accepted by TNC-F011 to the OBC whatever its addresses, control, PID or information field; it shall not interpret, authenticate, alter, reorder or originate frames, and shall hold no key material | RF-F002, RF-R003 | The OBC filters by address, UI and PID and authenticates (SQSO-ICD-003 §5.4; SQSO-ICD-013 §3, §8); the TNC is not a command decoder (§3.2) | T, I | UT (frames of every address, control and PID passed unchanged); IN: no key material or HMAC code in the image | Yes (SW-HAZ-015) |
| TNC-F013 | Bytes received over RF shall never reach the KISS command parser: only USART2 input shall be parsed as KISS commands | — (derived) | An uplink frame containing FEND and a command byte must not change the TNC's parameters or answer as a command (SW-HAZ-014) | T, I | UT (a received frame carrying FEND 0x06 0x01 and FEND 0x01 0xFF is delivered as an escaped data frame and changes nothing); IN | Yes (SW-HAZ-014) |
| TNC-F014 | The firmware shall keep receiving and passing frames while it transmits | RF-F002 | Full duplex: the downlink does not share the uplink channel (SQSO-ICD-003 §5.2); desense ≤ 0.5 dB (RF-P009) | T | EM: telecommand frames at −114 dBm while keyed at 1 W, with the VI-RFB-01 set-up | — |

### 5.4 Demodulator

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F015 | The firmware shall read every CC1200 CFM sample on its CLKEN_CFM interrupt (GPIO0 to PB10) at 12.5 kS/s while receiving, and count missed samples | RF-F002 | The demodulator and the repeat audio both run on these samples (SQSO-RF-BRD-001 §3.1, §13) | T | UT (sample pipeline); DB; EM (missed-sample count zero over 24 h with every function running) | — |
| TNC-P001 | The AFSK 1200 (Bell 202) demodulator shall decode at least 90 % of 60-byte frames at an input of −114 dBm at the board's 70 cm port, ±3.0 kHz deviation, at carrier offsets of 0 and ±2.5 kHz | RF-P011 | Telecommand: 90 % of 60-byte frames at C/N₀ ≤ 53.0 dB-Hz with the 2 dB implementation loss the design allows for the correlator (SQSO-LBA-001 §3; SQSO-RF-BRD-001 §6.1, VI-RFB-01) | T | EM: VI-RFB-01 at +25 °C and at −40 and +70 °C (VI-RFB-15); FQ | — |
| TNC-P002 | TNC-P001 shall hold with the 2200 Hz tone from 3 dB below to 6 dB above the 1200 Hz tone | — (derived) | Ground transmitters send AFSK with or without pre-emphasis; SQSO-ICD-005 §4.2 does not fix it | T | UT (synthesised CFM samples with twist); EM with VI-RFB-01 | — |
| TNC-P003 | On synthesised CFM sample files with white noise at C/N₀ 53.0 dB-Hz, the demodulator shall decode at least 90 % of 1,000 frames of 60 bytes in the host test | RF-P011 | The CI regression of TNC-P001's threshold (SQSO-LBA-001 §3) | T | UT on every change | — |

### 5.5 Modulator and keying

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F020 | The firmware shall assert TNC_PTT only to transmit a data frame received from the OBC (TNC-I003), and shall generate no frame, beacon, identification, tone or carrier of its own | RF-F001 | Keying only for a frame from the OBC (SDP §3.4); every emission decision is the OBC's (SQSO-ICD-003 §5.3) | T, I | UT; IN: TNC_PTT set at one place in the code | Yes (SW-HAZ-014) |
| TNC-F021 | The firmware shall not assert TNC_PTT while TX_PERMIT_HW (PA1) reads low, and shall discard and count a frame due then; if TX_PERMIT_HW falls during a transmission it shall drop TNC_PTT within 1 ms and discard the rest of the frame | RF-F004 | The hardware already removes the PA and exciter without PA_5V (SQSO-RF-BRD-001 §8.5); the firmware does not leave a key asserted to resume when the permit returns | T | UT; EM with VI-RFB-08 | Yes (SW-HAZ-014) |
| TNC-F022 | After asserting TNC_PTT the firmware shall start the frame's bits only at the end of TXDELAY and only with LMX_LD (PB14) high; if LMX_LD is low then, or falls during the frame, it shall drop TNC_PTT within 1 ms, discard the frame and count it | — (derived) | Lock in ≈ 0.1 ms after CE, PA ramp ≈ 1 ms, TXDELAY 300 ms (SQSO-RF-BRD-001 §7.1); the PA enable also needs the lock detect (U733), so an unlocked key radiates nothing but blocks the channel | T | UT; EM: synthesizer reference removed (test fixture), no frame keyed | Yes (SW-HAZ-014) |
| TNC-F023 | The firmware shall release TNC_PTT no later than 5.0 s after asserting it, whatever the frame, the parameters or the state of the modulator, by a hardware timer compare that drives PA8 low independently of the frame logic, and count each such release | SRS-P002 | The beacon path TNC_PTT bypasses EN_OK and the 120 s timer (truth table, SQSO-RF-BRD-001 §8.4); the OBC limits each frame to < 5 s (DOWNLINK_MAX_KEYDOWN_MS 5000), and the longest legitimate transmission with TNC-I005's bounds is 3.2 s (0.5 s TXDELAY, 330 bytes with worst-case stuffing at 1200 bit/s 2.64 s, flags and TXTAIL) | T | UT; EM: a test build that holds the modulator unkeys at 5.0 s ± 0.05 s | Yes (SW-HAZ-014) |
| TNC-F024 | The firmware shall keep TNC_PTT high for at most 60 s in any 120 s, and shall not start a transmission that would exceed it (the frame discarded and counted) | SRS-P002 | Backstop to the OBC's 25 % duty with a credit of at most 15 s (DOWNLINK_DUTY_PERCENT 25, DOWNLINK_DUTY_WINDOW_MS 60000): a correct OBC keys at most 45 s in 120 s, so the backstop acts only on a fault | T | UT (keying histories at the OBC's limit and above it) | Yes (SW-HAZ-014) |
| TNC-F025 | The modulator shall generate phase-continuous Bell 202 AFSK, mark 1200 Hz and space 2200 Hz, from a 12-bit sine table at 50 kS/s on DAC1 (PA4) by DMA, at 1200 bit/s, with each tone and the bit rate within ±0.5 % from −40 to +85 °C | RF-F001 | SQSO-RF-BRD-001 §7.1, §13; SQSO-ICD-005 §4.1. The tolerance is derived (SQSO-ICD-005 gives none; it leaves margin to the ground decoders' clock recovery); the HSI16's accuracy over temperature was not read (SQSO-RF-BRD-001 §12, §16), so the design may trim it against the TCXO-derived 12.5 kHz CLKEN_CFM | T | UT (sample stream); EM: frequency counter on TNC_AF at −40, +25 and +70 °C; FQ | — |
| TNC-F026 | Both tones shall have the same amplitude within 0.5 dB, without pre-emphasis, at the level set by the calibration record, so that the beacon's peak deviation is ±3.0 ± 0.15 kHz | RF-P004 | 1.43 Vpp on DAC1 gives ±3.0 kHz (SQSO-RF-BRD-001 §7.1); VI-RFB-05 criterion | T | EM: VI-RFB-05; FQ | — |
| TNC-F027 | With TNC_PTT low the firmware shall hold DAC1 at mid-scale (1.55 V) | — | Both select inputs biased at 1.55 V so the TMUX1219 makes no step (SQSO-RF-BRD-001 §8.2) | T | UT; EM | — |
| TNC-F028 | The firmware shall count a keying-path fault when, for more than 10 ms, PTT_EXC (PB6) reads high with TNC_PTT and XPDR_KEY (PB15) low, or low with TNC_PTT and TX_PERMIT_HW high, and report the count in the STATUS extension | — (derived) | Detects a stuck PTT gate or select (SQSO-FMEA-003 BF-RF-26); report only, the OBC measures PTT_EXCITER itself (SQSO-ICD-003 §4.3) | T | UT; EM with the gate output forced | — |

### 5.6 Repeat audio and TONE_OK

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F030 | The firmware shall write each CFM sample to DAC2 (PA5) within one sample period (80 µs) of its CLKEN_CFM interrupt, after removing DC with a high-pass filter whose corner is at most 10 Hz | RF-F003 | "DC removal, 1 sample latency" (SQSO-RF-BRD-001 §13); the CMX138A decodes the 67.0 Hz tone from this audio, so the filter must pass it (−0.1 dB at 67 Hz with a 10 Hz first-order corner) | T | UT (filter response, latency budget); EM: scope latency, CTCSS decode through the path | — |
| TNC-F031 | With the scale of the calibration record, an uplink at ±3.0 kHz peak deviation (1 kHz tone) shall repeat at ±3.0 ± 0.3 kHz; samples beyond the DAC range shall clip, not wrap | RF-P004 | A ±3 kHz uplink repeats at ±3 kHz (SQSO-RF-BRD-001 §7.1 table); the CMX138A limiter and the ADC clip are the hard limits | T | EM: VI-RFB-05 (repeat limit ≤ 5.0 kHz at PA_5V 5.15 V) | — |
| TNC-F032 | While the CC1200 is not receiving or its samples have stopped, the firmware shall hold DAC2 at mid-scale (1.55 V) | — (derived) | No step or stale sample on the repeat audio (SQSO-RF-BRD-001 §8.2) | T | UT; EM | — |
| TNC-P004 | Through the firmware, the repeat path shall give 12 dB SINAD at an input of −120 dBm at the 70 cm port (1 kHz tone, ±3 kHz deviation), at carrier offsets of 0 and ±2.5 kHz | RF-P011 | Voice: 12 dB SINAD at C/N₀ ≤ 47.5 dB-Hz (SQSO-LBA-001 §3.2; VI-RFB-01) | T | EM: VI-RFB-01; FQ | — |
| TNC-F033 | The firmware shall configure the CMX138A for receive, 67.0 Hz CTCSS decode, the 300–3000 Hz band filter that strips the tone, de-emphasis off, its 3.6864 MHz crystal (CLKSEL low) and IRQN on a change of tone status | RF-F003, RF-P012 | SQSO-RF-BRD-001 §8.2 (the uplink's pre-emphasis is repeated as received) | T | EM: tone decode and audio response; FQ | — |
| TNC-F034 | The firmware shall drive TONE_OK (PA7) to follow the CMX138A's 67.0 Hz decode status (read on IRQN) within 2 ms of IRQN, high only while the CMX138A reports the tone decoded and its configuration was verified at the last scrub, and low while the CMX138A does not answer, while its configuration mismatches (until rewritten and verified) and through any reset | RF-P012 | The relay's 2 ms is inside RF-P012's 260 ms maximum (SQSO-RF-BRD-001 §2, §8.2); a CMX138A with a corrupted configuration must not open the key gate | T | UT; EM: IRQN-to-TONE_OK latency; register corruption injected by the test build | — |
| TNC-P005 | With the firmware's relay, the transponder shall key within 200 ms typical and 260 ms maximum of a valid 67.0 Hz tone at 6 dB SINAD and drop within 260 ms of its loss | RF-P012 | SMART-QSO-RF-001 RF-P012 as restated in 2.1 | T | EM (with VI-RFB-07's set-up); FQ | — |

### 5.7 Radio IC configuration, scrubbing and frequency

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F040 | The firmware shall configure the CC1200 for receive only in its CFM mode on the uplink frequency of the configuration record, with the 25 kHz channel filter (±7.5 kHz IF), the CFM output at 12.5 kS/s with CLKEN_CFM on GPIO0, GPIO2 as CARRIER_SENSE (IOCFG2 = 17) with the AGC_CS_THR threshold of the calibration record, GPIO1 high-impedance with CSn high (IOCFG1), and FREQOFF from the trim record | RF-F002, RF-P010 | SQSO-RF-BRD-001 §6.1, §6.5, §8.6; `design.py` U202 notes; the uplink frequency is the coordinated one inside 435.050–436.500 MHz (BASELINE §7), held as configuration data | T | EM; FQ | — |
| TNC-F041 | The firmware shall never issue the CC1200's STX or SFSTXON strobes and shall hold its PA power setting at the minimum | RF-R002 | The CC1200 is a transceiver used receive-only (PA pin to VDD through L206, C208 not fitted); RADIO_3V3 stays on under TX-OFF, so a CC1200 transmission is not removed by the hardware TX permit (SW-HAZ-014) | T, I | UT (the strobe type admits no transmit strobe); IN; EM: a test build forcing STX at maximum power measures the leakage at J5/J6 and J3/J4 (recorded; the flight build cannot do it) | Yes (SW-HAZ-014) |
| TNC-F042 | Whenever TX_PERMIT_HW is high, the LMX2571 shall hold the register image of the configuration record (the coordinated downlink frequency, FSK I2S mode, the numerator trim): the firmware shall write it within 10 ms of TX_PERMIT_HW rising, every 10 s while it stays high, and before each TNC key-up, after checking the image's CRC-32; with a bad CRC it shall write nothing, not key (TNC_PTT) and report it | RF-P001, RF-P005 | The synthesizer is powered only with PA_5V (U805) and loses its registers without it; the repeat path keys it through XPDR_KEY without the TNC, so it must already be programmed (SQSO-RF-BRD-001 §5.1, §8.5, §12). The board has no read-back path (MICROWIRE through U503 is write-only; MUXout is the lock detect), so frequency integrity rests on the fresh write and the lock (TNC-F022) | T | UT (CRC path); EM: frequency at 1,000 key-ups by beacon and by the repeat path; spectrum across 144–148 MHz; FQ | Yes (SW-HAZ-014) |
| TNC-F043 | Every 10 s, and at each TNC key-up, the firmware shall read back every configuration register of the CC1200 and the CMX138A, compare it with the configuration record, rewrite and re-verify any mismatch and count the correction; a mismatch that persists after 3 rewrites shall fail criterion 4 (TNC-F050) | — | SEU scrubbing of the radio ICs (SQSO-RF-BRD-001 §8.6 criterion 4, §12) | T | UT; EM: register corruption injected by the test build, corrected within 10 s | Yes (SW-HAZ-014, SW-HAZ-015) |
| TNC-F044 | The firmware shall take the CC1200 FREQOFF and the LMX2571 numerator trims only from the trim record in flash, each bounded to ±3 ppm; a trim outside ±3 ppm, or a record with a bad CRC, shall be replaced by a zero trim and flagged in the STATUS extension | RF-P005 | Trims set at acceptance to ±0.1 ppm residual; ageing is corrected on the ground (RFB-05; SQSO-RF-BRD-001 §6.5) | T | UT; EM: VI-RF-14 with the calibrated record | Yes (SW-HAZ-014) |

### 5.8 Receiver supervision (RX_ALIVE, RX_CD)

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F050 | The firmware shall toggle RX_ALIVE (PB7) every 100 ± 10 ms from the main loop, never from an interrupt, while all six criteria of TNC-F043 and TNC-F051 to TNC-F054 hold, and stop toggling while any fails (criterion 5: a hung or resetting TNC produces no pulse) | SRS-F116 | RX_FAULT goes high 0.36–0.62 s after the last pulse (SQSO-RF-BRD-001 §8.6, §13) | T | UT; EM: VI-RFB-10 (each injected fault gives RX_FAULT high within 0.36–0.62 s, low after recovery); FS | Yes (SW-HAZ-015) |
| TNC-F051 | Criterion 1, CC1200 responds: every 1 s the firmware shall read PARTNUMBER (0x20 expected) and write and read back a scratch register; three consecutive failures shall fail the criterion | SRS-F116 | SQSO-RF-BRD-001 §8.6 | T | UT; EM: CC1200 held in reset, SPI line open (VI-RFB-10) | Yes (SW-HAZ-015) |
| TNC-F052 | Criterion 2, CC1200 receiving: MARCSTATE shall read RX; when it does not, the firmware shall make one re-entry attempt, and MARCSTATE not RX for more than 2 s shall fail the criterion | SRS-F116 | SQSO-RF-BRD-001 §8.6 | T | UT; EM | Yes (SW-HAZ-015) |
| TNC-F053 | Criterion 3, synthesizer and reference: the CC1200 shall report lock in RX and a stable crystal oscillator, and CLKEN_CFM interrupts shall arrive at 12.5 kHz ± 5 %; no interrupt for more than 100 ms in RX shall fail the criterion | SRS-F116 | SQSO-RF-BRD-001 §8.6 | T | UT; EM: TCXO removed (VI-RFB-10) | Yes (SW-HAZ-015) |
| TNC-F054 | Criterion 6, front end not deaf (TNC-FW-NF): every 60 s the firmware shall take the 10th percentile of the in-channel RSSI of that window, excluding samples with carrier sense, as the noise floor, and compare it with the calibrated floor interpolated from the record's three-temperature table at the MCU's internal temperature sensor; a floor more than 5 dB below the calibrated value in 3 consecutive windows shall fail the criterion, and a rise shall never fail it; with a table whose CRC fails, the criterion shall not be evaluated (it does not stop RX_ALIVE) and the calibrated value shall be reported as −128 dBm | SRS-F116 | A dead LNA or an open SAW lowers the floor by 7.8–9.6 dB; interference raises it (SQSO-RF-BRD-001 §8.6 criterion 6, §13; `rf_calcs.py` §18; SQSO-FMEA-003 SMA-B11). Not evaluating with a bad table avoids a false RX_FAULT, which would keep the beacon under REL-04 when it should be silenced (BF-RF-32) | T | UT (percentile, interpolation, windows); EM: VI-RFB-10 (LNA bias removed and SAW output shorted give RX_FAULT within 4 min; no false fault over −40 … +70 °C and with a +6 dB interference floor) | Yes (SW-HAZ-015) |
| TNC-F055 | The firmware shall count the rising edges of CARRIER_SENSE (GPIO2 on PB11) and report the count and the last window's noise floor in the STATUS answer | SRS-F115 | The ground tells interference (raised floor) from a stuck line (normal floor, frozen count) (SQSO-RF-BRD-001 §8.6, RX_CD row) | T | UT; EM: carrier 3 dB above the threshold counted, noise not (< 1 false count per hour, VI-RFB-10) | — |
| TNC-F056 | When criterion 1, 2 or 3 fails, the firmware shall re-initialise the CC1200 (reset through CC_RESETN, full configuration) at most once per 10 s, and resume RX_ALIVE when every criterion holds again | SRS-F116 | A fault clears when the pulses resume after the TNC re-initialises the CC1200, or after the OBC's RADIO_EN power cycle (SQSO-RF-BRD-001 §8.6); the rate limit keeps a hard failure from blocking the main loop | T | UT; EM: recovery after each injected fault | Yes (SW-HAZ-015) |

### 5.9 Calibration and configuration records

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-F060 | The configuration and calibration records (uplink and downlink frequencies, LMX2571 image, CC1200 and CMX138A images, frequency trims, AGC_CS_THR, the noise-floor table at three temperatures, the AFSK and repeat-audio scales) shall be written only by a ground calibration build through SWD, never by the flight build | — | TNC-FW-CAL: the floor table at acceptance and in TVAC with the antenna port terminated in 50 Ω at the chamber temperature; the threshold 6 dB above the measured floor (SQSO-RF-BRD-001 §8.6, §13) | I, T | IN: no flash-programming code in the flight build; EM: calibration in TVAC (VI-RFB-10 records the table) | — |
| TNC-F061 | Each record shall carry a version and a CRC-32; at start-up the firmware shall check them, use a compiled-in default for an invalid record (zero trims; criterion 6 not evaluated) and report it in the STATUS extension | — (derived) | A corrupted record must not stop the radio or fake a fault | T | UT | — |
| TNC-F062 | The flight build shall contain no bootloader or flash-programming path reachable from USART2, USART1, RF or any GPIO, and its option bytes shall boot from main flash whatever the BOOT0 pin, with readout protection level 1 and write protection on the image | — (derived) | No in-flight image change (as for the OBC: SMART-QSO-SDP-001 §3.3, NPR 7150.2D SWE-075 row); debug-port protection was an open item of SWE-157 | I | IN: option bytes and the link map of the release image | — |
| TNC-F063 | At start-up the firmware shall compute the CRC-16 of its image and report it, with its version, in the STATUS answer | — | Configuration status accounting of the flight image in orbit (SWE-083) | T | UT; EM | — |

### 5.10 Constraints

| ID | Requirement | Parent | Rationale and source | Verification | Activity | SC |
|----|-------------|--------|----------------------|--------------|----------|----|
| TNC-C001 | The firmware shall be written in Rust (2024 edition, `no_std`) for `thumbv7em-none-eabihf` with the toolchain pinned for the OBC (`software/rust/rust-toolchain.toml`), and shall take its KISS, AX.25 and CRC code from `sqso-protocol` | — | §7; the KISS and AX.25 code is already `no_std` and fuzzed (SQSO-RF-BRD-001 §3.2) | I | IN: build configuration and CI toolchain pin | — |
| TNC-C002 | The logic that implements the SC requirements shall be in a crate with `unsafe` forbidden, built and tested on the host; `unsafe` shall appear only in the board-support layer, each block with its safety argument | — | As for `sqso-fsw` (SMART-QSO-SDP-001 §2.3; `docs/development/CODING_STANDARDS.md`) | I | IN | — |
| TNC-C003 | The firmware shall allocate no dynamic memory, use no recursion and bound every loop | — | `docs/development/CODING_STANDARDS.md` (JPL Power of Ten rules 1–3) | I, A | IN; A: symbol check of the image | — |
| TNC-C004 | Flash use shall be at most 75 % of 256 KB, RAM (static plus the worst-case stack by analysis) at most 75 % of 64 KB, and the worst-case CPU load at most 70 % at 16 MHz with the demodulator, modulator, repeat audio and scrub running together | — (derived) | 25 % margin at CDR for growth, as for the OBC's stack (SRS-C003) | A, T | A: map file, stack analysis; DB and EM: CPU load measured by a timer probe | — |
| TNC-C005 | The modules that implement the SC requirements shall have 100 % line and 100 % condition coverage in the host tests, every uncovered line marked and justified; the rest at least 90 % of lines | — | SWE-219 as tailored by CR-2026-003 (SMART-QSO-SDP-001 §3.2) | A | UT: the coverage gate in CI | — |
| TNC-C006 | The firmware shall be its own configuration item with a version number, built and tested in CI on every change, and released under the release procedure | — | SMART-QSO-SDP-001 §3.4 and §7; SWE-187 | I | IN: `docs/software/CONFIGURATION_INDEX.md` entry when the item is created | — |

## 6. Requirement Count and Safety-Critical Requirements

| Group | Requirements | Safety-critical |
|-------|-------------:|----------------:|
| §5.1 Start-up, clocks, watchdog | 7 | 4 |
| §5.2 KISS host interface | 13 | 4 |
| §5.3 HDLC and AX.25 | 5 | 2 |
| §5.4 Demodulator | 4 | 0 |
| §5.5 Modulator and keying | 9 | 5 |
| §5.6 Repeat audio and TONE_OK | 7 | 0 |
| §5.7 Radio IC configuration and frequency | 5 | 4 |
| §5.8 Receiver supervision | 7 | 6 |
| §5.9 Records and image | 4 | 0 |
| §5.10 Constraints | 6 | 0 |
| **Total** | **67** | **25** |

Methods: T on 60 requirements (alone or with another method), I on 14, A on 4, D on none.

The safety-critical requirements implement the controls of SQSO-SHA-001 G: SW-HAZ-014 (emission outside the TNC's
limits: TNC-F002, F004, F005, I005, I008, F013, F020 to F024, F041 to F044) and SW-HAZ-015 (the telecommand path and
the receiver evidence: TNC-F003, F004, I006, I011, F012, F043, F050 to F054, F056).

## 7. Language and Toolchain

**Decision proposed: Rust, `no_std`, with the OBC's pinned toolchain** (for the PI's ratification, dated action DA-22
in `mission/SCHEDULE.md`).

- The STM32L431CCT6 is a Cortex-M4F like the OBC's STM32L4R5: the target `thumbv7em-none-eabihf` is already in the
  pinned toolchain (Rust 1.97.0 stable, `software/rust/rust-toolchain.toml`), so no new compiler enters the project.
- The KISS codec, the STATUS layout and the AX.25 CRC exist in `sqso-protocol` (`no_std`, `unsafe` forbidden,
  fuzzed), and the OBC, the payload and the ground use the same code: the two ends of the KISS link cannot drift.
- The Embassy HAL used for the OBC firmware (`software/rust/firmware`) supports the STM32L4 family; the sample paths
  (12.5 kS/s CFM interrupt, 50 kS/s DAC DMA) are interrupt-driven at the highest priorities, with the supervision and
  scrubbing in the main loop. No RTOS.
- The project's coverage gate, clippy, stack analysis and coding standard (`docs/development/CODING_STANDARDS.md`,
  Rust rules) apply unchanged.
- Risks, honestly: the DSP code (correlator, filters) needs its timing measured on the target early (task TNC_UT,
  CPU load TNC-C004); the toolchain is not qualified (CR-2026-004 applies as to the OBC).

## 8. Traceability

The requirements of this specification are rows of the generated RTM (`docs/verification/RTM.csv`,
`scripts/gen_rtm.py`, specification `docs/software/TNC_FIRMWARE_REQUIREMENTS.md`) and of the VCRM's generated block
(`docs/verification/VERIFICATION_MATRIX.md`); `scripts/check_traceability.py` checks them with the other
specifications. Code and tests will tag them with `@requirement TNC-…` like the flight software.

Upward, by parent:

| Parent | Requirements |
|--------|--------------|
| RF-F001 (AX.25 beacon, AFSK, from a KISS TNC) | TNC-I003, F010, F020, F025 |
| RF-F002 (receive telecommand) | TNC-F001, I011, F011, F012, F014, F015, F040 |
| RF-F003 (repeat with CTCSS) | TNC-F002, F030, F033 |
| RF-F004 (TX-OFF), RF-R002 (cessation) | TNC-F021, F041 |
| RF-P001, RF-P005 (frequency, stability) | TNC-I008, F042, F044 |
| RF-P004 (deviation) | TNC-F026, F031 |
| RF-P010, RF-P011, RF-P012 (receiver, demodulators, CTCSS) | TNC-F040, P001, P003, P004, F033, F034, P005 |
| RF-R003 (only telecommand authenticated) | TNC-F012 |
| SRS-F115, SRS-F116 (receiver evidence, OBC side) | TNC-F003, I006, F050 to F056 |
| SRS-P002 (key-down and duty, OBC side) | TNC-F023, F024 |
| Derived (reason in each row) | TNC-F004 to F007, I005, I007, I010, I013, F013, F022, F028, F032, P002, F061, F062, C004 |

The requirements of SQSO-RF-BRD-001 §13 by name: TNC-FW-NF is TNC-F054; TNC-FW-STAT is TNC-I006 and I007; TNC-FW-POR
is TNC-F003; TNC-FW-CAL is TNC-F060 and F061.

## 9. Findings for the Chair and the Owners

1. **No read-back of the LMX2571.** SQSO-RF-BRD-001 §12 says the TNC "re-writes and verifies" the LMX2571's
   configuration, but the MICROWIRE path through U503 is write-only and MUXout is the lock detect. TNC-F042 therefore
   rewrites the image from a CRC-checked record before every key-up and every 10 s and relies on the lock detect;
   a register upset between a write and a key keys at most one transmission on a wrong frequency. Owner: RF designer
   (a MUXout read-back mode or an accepted residual in SQSO-SHA-001 G SW-HAZ-014).
2. **Writes with CE low.** TNC-F042 programs the LMX2571 while CE is low (the PTT gate raises CE only when keyed).
   That the LMX2571 accepts and keeps MICROWIRE writes in power-down is to be confirmed from the TI data sheet before
   the TNC design review (task TNC_DES); if it does not, the image must be written after CE rises, inside TXDELAY,
   and the repeat path needs a way to do the same. Owner: RF designer with the flight software lead.
3. **The TNC_PTT path has no hardware key-down timer.** The 120 s timer covers only the repeat path. TNC-F023 and
   TNC-F024 are the only bounds besides RF_TX_EN. Recommendation for the SRS owner: the OBC watches PTT_EXCITER (it
   already captures it on TIM8_CH2) and drives RF_TX_EN low when it stays high for more than 6 s with no frame
   outstanding (SQSO-SHA-001 G, C-014-9). Not in SMART-QSO-SRS-001 1.10.
4. **CC1200 transmit leakage** has not been measured: TNC-F041's EM measurement records it.
5. **HSI16 accuracy** over temperature was not read (ST site not reached, SQSO-RF-BRD-001 §16); TNC-F025 sets the
   AFSK tolerance either way.

---

## Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | SMART-QSO Team (CDR round 3, software systems; finding P3A-03) | Initial issue: 67 requirements of the RF board TNC firmware, 25 safety-critical (SW-HAZ-014, SW-HAZ-015), derived from SQSO-RF-BRD-001 F, SQSO-ICD-003 2.3, SMART-QSO-RF-001 2.4, SQSO-ICD-013 and the OBC's limits; hardware functions it must not be relied on (§3.3); language Rust `no_std` proposed (§7); traceability through the generated RTM (§8); findings for the chair (§9) |
