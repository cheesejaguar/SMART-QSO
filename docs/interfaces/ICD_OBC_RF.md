# Interface Control Document: OBC to RF Subsystem

**Document ID**: SQSO-ICD-003
**Version**: 2.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR. Both sides are build-to designs: the OBC board SQSO-OBC-BRD-001 (`hardware/electrical/obc/OBC_BOARD_DESIGN.md` §13) with the master pin allocation SQSO-ICX-001 (`hardware/electrical/INTERCONNECT.md`), and the RF board SQSO-RF-BRD-001 (`hardware/electrical/rf/RF_BOARD_DESIGN.md` §4). Not built or measured. The firmware's pins are unchanged for this interface except the three readback inputs and (version 2.2) the receiver status inputs RX_CD and RX_FAULT.

## 1. Purpose

This document defines the interface between the On-Board Computer (STM32L4R5ZI) and the RF subsystem: the KISS TNC
serial link that carries every uplink and downlink frame, the discrete control lines (radio power, PA power, power
select, transponder heartbeat) and the analog monitors.

Version 1.x described an SPI interface to a Si4463 transceiver at 9600 bps GFSK on 145.825 MHz. None of that is in
the baseline: the flight software was built to a KISS TNC on a UART (change request CR-2026-005), the downlink is
1200 bps AFSK, and 145.825 MHz is excluded (BASELINE §7). Version 2.0 replaces it.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-BL-001 Rev D | Baseline parameters (§7 governs RF values) |
| SMART-QSO-RF-001 v2.0 | RF subsystem design (`docs/hardware/RF_DESIGN.md`) |
| SQSO-RF-BRD-001 A | RF board build-to design (`hardware/electrical/rf/RF_BOARD_DESIGN.md`) |
| SQSO-OBC-BRD-001 A | OBC board build-to design (`hardware/electrical/obc/OBC_BOARD_DESIGN.md`) |
| SQSO-ICX-001 A | Spacecraft electrical interconnect and master pin allocation (`hardware/electrical/INTERCONNECT.md`) |
| SQSO-RF-TX-001 | RF transmitter design (`hardware/rf/TRANSMITTER_DESIGN.md`) |
| SQSO-ICD-013 | Telecommand authentication and TX-OFF (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`) |
| SMART-QSO-ICD-006 | Beacon format (`docs/BEACON_SPEC.md`) |
| SQSO-LBA-001 v3.0 | Link budget |
| `software/flight/include/hal/hal_radio.h`, `hal_gpio.h`, `hal_adc.h`, `downlink.h`, `uplink.h`, `rf_control.h` | The OBC's interface as built |

## 3. Interface Summary

```
 OBC (STM32L4R5ZI)                                      RF board
 ---------------------------------------------------------------------------------------------
 USART2 TX  PD5  ---- KISS frames, 9600 8N1 ------>  TNC  -> AFSK -> exciter -> PA -> LPF -> 2 m
 USART2 RX  PD6  <--- KISS frames (FCS checked) ---  TNC  <- AFSK <- receiver <- RX filter <- 70 cm
 RADIO_EN   PB5  ---- radio supply load switch ---->  receiver, TNC, exciter, repeat path
 RF_TX_EN   PA3  ---- PA supply load switch ------->  PA (TX-OFF, inhibits)
 TX_PWR_SEL PE3  ---- low 0.5 W / high 1 W -------->  PA bias / drive
 XPDR_EN    PE4  ---- 100 Hz heartbeat ------------>  monostable -> repeat-path key gate
 TX_TEMP    PA0  <--- PA temperature (analog) -----  NTC / TMP36-class sensor
 TX_FWD     PC0  <--- forward power (analog) ------  -20 dB coupler + RMS detector
 TX_REV     PC1  <--- reflected power (analog) ----  coupler reverse port
 TONE_OK    PC9  <--- CTCSS 67.0 Hz decoded -------  TNC MCU (CMX138A tone status)
 XPDR_KEY   PG12 <--- repeat path keyed ----------  EN_OK . TONE_OK . NOT_TOT
 PTT_EXC    PC7  <--- transmitter keyed (TIM8_CH2)-  TX_PERMIT_HW . (TNC_PTT + XPDR_KEY)
```

The OBC does not modulate, demodulate, run HDLC or the CTCSS decoder. The TNC and the repeat path are hardware on the
RF board, so no flight software is in the audio path (BUILD_VS_BUY §3.2).

## 4. Physical and Electrical Interface

### 4.1 Connector

Harness W4A version 3: Samtec FTSH-108-01-L-DV-K (16 positions, 1.27 mm) at both ends, OBC J2 to RF J1, 28 AWG,
0.06 m, three signal grounds interleaved and an analog return on pin 14; pins 15–16 (RX_CD, RX_FAULT) are new in
version 3 (version 2: FTSH-107, 14 positions; pins 1–14 unchanged). RADIO_EN and RF_TX_EN switch the RF rails on
the EPS (W3 pins 10 and 14); the rails reach the RF board on W4B (EPS J19 to RF J2, Molex Pico-Clasp 6, 24 AWG:
RADIO_3V3 on pin 1, PA_5V on pins 3–4). Pin numbers below are J1/J2 positions.

### 4.2 Signals

| J pin | Signal | Dir. | MCU pin | Type, levels | Default and fail-safe | OBC-side parts | RF-side parts |
|------:|--------|------|---------|--------------|-----------------------|----------------|---------------|
| 1, 4, 7 | GND | — | — | Signal return | — | — | — |
| 2 | TNC_RXD (OBC → TNC) | OBC→RF | PD5 (USART2 TX) | 3.3 V CMOS UART, 9600 baud, 8N1 | Idle high; high-Z at OBC reset | 100 Ω series, ESD | 1 kΩ series into the TNC MCU (5 V tolerant) |
| 3 | TNC_TXD (TNC → OBC) | RF→OBC | PD6 (USART2 RX) | 3.1 V CMOS UART | — | 10 kΩ series (back-power limit), 100 kΩ pull-up | 100 Ω series |
| 5 | TX_PWR_SEL | OBC→RF | PE3 | 3.3 V, low = 0.5 W, high = 1 W | **Low at reset** (RF 100 kΩ pull-down) | 100 Ω | 1 kΩ + 100 kΩ |
| 6 | XPDR_EN | OBC→RF | PE4 | 100 Hz square wave (`HAL_GPIO_HEARTBEAT_HZ`, TIM7 interrupt); the RF board's retriggerable monostable (U401, 46.8–58.0 ms) gives EN_OK | Low at reset; **a level held high is "disabled"** | 100 Ω | 1 kΩ + 100 kΩ pull-down |
| 8 | TONE_OK | RF→OBC | **PC9** (EXTI9) | 3.1 V CMOS push-pull through 1 kΩ; high = CTCSS 67.0 Hz decoded | low with the TNC in reset or the radio off | 10 kΩ series, **100 kΩ pull-down** | 1 kΩ, 100 kΩ pull-down |
| 9 | XPDR_KEY | RF→OBC | **PG12** (EXTI12) | as TONE_OK; high = repeat path keyed (EN_OK · TONE_OK · NOT_TOT) | low with the radio off | 10 kΩ, 100 kΩ pull-down | 1 kΩ |
| 10 | PTT_EXCITER | RF→OBC | **PC7** (TIM8_CH2 input capture) | as TONE_OK; high = transmitter keyed (beacon or repeat, TX permitted) | low with the radio off | 10 kΩ, 100 kΩ pull-down | 1 kΩ |
| 11 | TX_TEMP | RF→OBC | PA0 (ADC1 IN5) | 0.3–1.7 V, 10 mV/°C, 500 mV at 0 °C | — | 10 kΩ series, 10 nF | 1 kΩ / 100 nF |
| 12 | TX_FWD | RF→OBC | PC0 (ADC1 IN1) | 0–2.5 V, P = 0.16 V² W (2.5 V at 1 W) | — | 10 kΩ, 10 nF | 1 kΩ / 100 nF |
| 13 | TX_REV | RF→OBC | PC1 (ADC1 IN2) | as TX_FWD; 1.25 V at 3:1 and 1 W | — | 10 kΩ, 10 nF | 1 kΩ / 100 nF |
| 14 | AGND | — | — | monitor return | — | — | — |
| 15 | RX_CD | RF→OBC | **PG0** (EXTI0) | as TONE_OK; **high = receiver carrier detect** (RF carrier above the receiver's carrier-sense threshold, with or without CTCSS; TONE_OK is the tone decode, not this) | low with the radio off | 10 kΩ series, **100 kΩ pull-down** | 1 kΩ (push-pull from V_DIG), 100 kΩ pull-down |
| 16 | RX_FAULT | RF→OBC | **PF13** (EXTI13) | as TONE_OK; **high = receiver fault** (conditions per SQSO-RF-BRD-001 B §4.1) | **low = healthy when the line is unwired, open or the radio unpowered** | 10 kΩ series, **100 kΩ pull-down** | 1 kΩ (push-pull from V_DIG), 100 kΩ pull-down (no pull-up on either side) |
| (W3-10) | RADIO_EN | OBC→EPS | PB5 | 3.3 V, high = SW2 (radio rail) on | **Low at reset** (EPS 100 kΩ) | 100 Ω | — |
| (W3-14) | RF_TX_EN | OBC→EPS | PA3 | 3.3 V, high = SW3 (PA rail) on | **Low at reset**; EPS pull-down mandatory, so a reset or unpowered OBC leaves the PA off | 100 Ω | — (the RF board derives TX_PERMIT_HW from PA_5V) |

Readback levels: V_DIG 3.10 V ± 1.5 %, VOH ≥ 2.9 V at the 1 kΩ, against the STM32's VIH 2.31 V (0.6 V margin). The
OBC-side 10 kΩ series resistors limit the current into an unpowered OBC (the radio stays on while the OBC is power
cycled): together with the EPS lines, ≤ 2.7 mA (SQSO-OBC-BRD-001 §16). The ADC scalings are those of
`software/rust/firmware/src/adc.rs` and now match the RF board's detector and sensor (RF board §7.6).

### 4.3 Readbacks (wired, RIDs RF-21 and AV-05)

TONE_OK, XPDR_KEY and PTT_EXCITER are wired to the OBC (§4.2, J pins 8–10). The firmware reads TONE_OK and XPDR_KEY
into the transponder statistics (EXTI on each edge, or a poll each pass) and measures PTT_EXCITER's keyed time by
input capture on TIM8_CH2, replacing the frame-length estimate of §5.3 and giving the 25 % duty and 5 s key-down
checks a measurement. Reading them is flight-software work (SQSO-ICX-001 §6 row 18).

RX_CD and RX_FAULT (version 2.2, flight-software request) report the receiver: RX_CD on EXTI0 for channel-busy
detection and receive statistics, RX_FAULT on EXTI13 for receiver health. Both read low with the radio off, so the
firmware qualifies them with RADIO_EN and ignores RX_FAULT for 1 s after RADIO_EN switches on (receiver start-up); RX_FAULT therefore reads "healthy" with the radio off or the line open, as
requested, and a broken wire is not detected by the line itself (the KISS link and the receive statistics show a dead
receiver). SQSO-ICX-001 §6 row 28.

### 4.4 Power

| Rail | Source | Switched by | Load |
|------|--------|-------------|------|
| RADIO_3V3 (receiver, TNC, exciter, repeat path) | EPS 3V3_AO through SW2 (TPS2553-Q1), W4B pin 1 | RADIO_EN (PB5) | 31 mA typical, 38 mA worst receiving; + 7–9 mA repeat path; + 44–51 mA keyed |
| PA_5V | EPS 5V_PA buck through SW3 (TPS2553-Q1, trip ≈ 1 A), W4B pins 3–4 | RF_TX_EN (PA3) AND the EPS load shed | 0.352 A at 0.5 W RF, 0.469 A at 1 W |

Power-up order: RADIO_EN high, receiver ready ≤ 50 ms later (RF board §15 item 12), then the OBC sends the KISS parameter commands (§5.2) before the first frame
(the firmware sends them before every frame, because the radio may have been off).

## 5. Logical Interface

### 5.1 KISS framing

`sqso_protocol::kiss` (KISS, Chepponis and Karn, 1987): FEND 0xC0, FESC 0xDB, TFEND 0xDC, TFESC 0xDD; command byte
0x00 = data frame on port 0.

| Direction | Content |
|-----------|---------|
| OBC → TNC | Parameter commands (§5.2), then one data frame: AX.25 address through information field, **no FCS** (the TNC adds flags, FCS, bit stuffing, NRZI, AFSK). Largest frame 328 bytes (`DOWNLINK_MAX_FRAME_LEN`) |
| TNC → OBC | Each received AX.25 frame whose FCS the TNC has checked, **without the FCS**; the firmware appends the FCS again so that `uplink.rs` checks it a second time (defence in depth, ICD-013 §3.1) |

### 5.2 TNC parameters (sent before each frame)

| KISS command | Value | Why |
|--------------|-------|-----|
| TXDELAY (0x01) | 30 (300 ms) | Exciter and PA settle; the monitors are read after it (`DOWNLINK_TXDELAY_MS`) |
| P (0x02) | 255 | Full duplex: never defer |
| SLOTTIME (0x03) | 0 | as above |
| TXTAIL (0x04) | 1 (10 ms) | |
| FULLDUPLEX (0x05) | 1 | The downlink does not share the uplink channel, so the TNC must not wait for the receiver's carrier detect |

### 5.3 Transmission rules (OBC side, as built)

`downlink_transmit()` refuses a frame unless transmission is permitted at that moment (TX-OFF, RF silence, loss of
contact), sends one frame at a time, keeps each frame under 5 s on the air (`DOWNLINK_MAX_KEYDOWN_MS`) and the keyed
time under 25 % with a 60 s credit (`DOWNLINK_DUTY_*`). The time a frame is on the air is estimated from its length,
the UART rate and the TNC parameters; the PTT readback (§4.3) would make it measured.

### 5.4 Reception

`uplink_poll()` takes each frame from the HAL, checks the FCS again, accepts only UI frames with PID 0xF0 addressed to
KK6GDB-11, and passes the information field to `cmd_process()` (ICD-013 §8). The receiver and TNC are powered whenever
RADIO_EN is high; TX-OFF and the loss-of-contact inhibit do not remove receiver power.

### 5.5 Transponder

The OBC renews the XPDR_EN heartbeat lease only while `sqso_fsw::transponder` permits (transmission permitted, radio
on, ACTIVE, sunlit, SOC ≥ 55 %, PA < 70 °C with resume at 60 °C, keyed < 25 % of 100 min). Keying is decided on the RF
board: `XPDR_KEY = EN_OK · TONE_OK · NOT_TOT`, and the PA can radiate only with RF_TX_EN high. The TNC's frames take the
transmitter first (`PTT_EXCITER = TX_PERMIT_HW · (TNC_PTT + XPDR_KEY · ¬TNC_PTT)`).

## 6. Timing

| Parameter | Value | Source |
|-----------|-------|--------|
| UART time of the largest frame (328 B, every byte escaped, plus parameters) | ≤ 0.75 s at 9600 baud | `radio.rs` `STREAM_MAX` |
| Time on air of a 110-byte beacon | ≈ 1.2 s (TXDELAY 0.3 s + 1,060 bits at 1200 bps) | SIL: 1.22 s average |
| RF_TX_EN low to RF off at the antenna | ≤ 1 ms (load switch) | VI-RF-12 |
| XPDR_EN edges stop to repeat path unkeyed | ≤ 100 ms (monostable 50 ms + gate) | VI-RF-11 |
| RADIO_EN high to receiver ready | ≤ 50 ms (TNC MCU boot ≈ 5 ms, CC1200 set-up ≈ 10 ms, TCXO ≤ 2 ms) | SQSO-RF-BRD-001 §15 item 12 |
| XPDR_EN last edge to EN_OK low | 46.8–58.0 ms (stalled main loop: the ≤ 40 ms lease + 46.8–58.0 ms = 87–98 ms, 90 ms nominal; SRS-F081, `transponder.rs` `HEARTBEAT_LEASE_MS` 40, `STALL_DROPOUT_MS` 90), inside VI-RF-11's 100 ms | SQSO-RF-BRD-001 §8.1 |
| Readback valid after RADIO_EN | with the receiver ready (≤ 50 ms) | §4.2 |

## 7. Verification

| Test | Method | Pass criterion | When |
|------|--------|----------------|------|
| KISS link | Loop through the EM TNC; frames of 1–328 bytes, all escape cases | Byte-exact; FCS appended on receive | EM |
| Inhibit lines | Drive RADIO_EN and RF_TX_EN low, high, high-Z, OBC in reset | RF board state as §4.2 in every case | EM (VI-RF-12, VI-RF-13) |
| Heartbeat | Stop, stall, halt the OBC with XPDR_EN running | Repeat path unkeys ≤ 100 ms (VI-RF-11) | EM |
| Monitors | Calibrate TX_FWD, TX_REV against a power meter; TX_TEMP against a thermocouple | ±0.5 dB, ±2 °C | EM |
| Power select | TX_PWR_SEL low/high | 27.0 / 30.0 ± 0.5 dBm (VI-RF-02) | EM |
| Receiver status | Unmodulated carrier on and off at the receive sensitivity + 3 dB; RX_FAULT forced on the RF EM; W4A pin 16 open; radio off | RX_CD follows the carrier; RX_FAULT high only when forced; low (healthy) with the pin open or the radio off | EM |

## 8. Open Items

| Item | Closure |
|------|---------|
| Connector part and pin-out | **Closed**: W4A v3, FTSH-108 at both ends (§4.1, §4.2; SQSO-ICX-001 §3.2) |
| RX_CD and RX_FAULT on the RF board | OBC side allocated (PG0, PF13; W4A pins 15, 16); **closed**: SQSO-RF-BRD-001 B §4.1 drives J1-15/16 (FTSH-108) |
| Readback pins for TONE_OK, XPDR_KEY, PTT_EXCITER | **Closed in hardware**: PC9, PG12, PC7 (§4.3); the firmware reading them is open (flight-software change, SQSO-ICX-001 §6) |
| ADC scaling of TX_TEMP, TX_FWD, TX_REV | **Closed**: the RF board's detector and sensor match `adc.rs` (RF board §7.6) |
| TNC part | **Closed**: TNC MCU on the RF board (SQSO-RF-BRD-001 §13) |
| VI-RF-11 "unkeys within 100 ms" vs the stalled-loop case | **Closed**: the lease is 40 ms (SRS-F081, flight software round 2; was 60 ms), so a stalled loop drops the transponder within 90 ms nominal and 98 ms with the monostable at its 58.0 ms worst, inside VI-RF-11's 100 ms (RF board §15 item 13) |

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline (Si4463 SPI, 9600 GFSK) |
| 1.1 | 2026-10-16 | SMART-QSO Team | Beacon Format ICD cited by its own number, SMART-QSO-ICD-006 (finding PI-17) |
| 1.1, note | 2026-10-24 | SMART-QSO Team | RF_TX_EN row names the Rust file that replaced `rf_control.c` |
| 2.0 | 2026-10-27 | CDR review, communications panel | Rewritten to the interface as built: KISS TNC on USART2 (CR-2026-005), RADIO_EN, RF_TX_EN, TX_PWR_SEL, the XPDR_EN heartbeat, the transmitter monitors; KISS parameters; timing; readbacks allocated; verification. The Si4463 SPI command set, 9600 bps GFSK and 145.825 MHz are removed |
| 2.1 | 2026-10-27 | CDR design completion (OBC board, SQSO-OBC-BRD-001) | §3 readbacks drawn; §4.1 connector W4A v2 (FTSH-107, 14 positions) and W4B; §4.2 pin-by-pin with the OBC and RF parts, readback levels, OBC series resistors; §4.3 readbacks wired to PC9, PG12, PC7 (RIDs RF-21, AV-05); §4.4 rails from the EPS board; §6 receiver ready ≤ 50 ms, EN_OK timing; §8 open items closed or carried; from SQSO-RF-BRD-001 §4 |
| 2.2 | 2026-10-27 | CDR design completion, flight-software round 2 | §4.1 W4A version 3 (FTSH-108, 16 positions); §4.2 RX_CD (pin 15, PG0 EXTI0) and RX_FAULT (pin 16, PF13 EXTI13, low = healthy), back-power total 2.7 mA; §4.3 their use; §7 receiver-status test; §8 |
| 2.3 | 2026-10-27 | CDR round 3, launch and operations (finding P3A-07) | §6 stalled-loop timing with the 40 ms heartbeat lease (87–98 ms, 90 ms nominal; SRS-F081; was "60 ms lease, 118 ms"); §8 the VI-RF-11 item closed (inside 100 ms) |
