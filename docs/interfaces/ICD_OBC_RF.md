# Interface Control Document: OBC to RF Subsystem

**Document ID**: SQSO-ICD-003
**Version**: 2.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR. The OBC side is as built in the flight software and firmware (not run on hardware); the RF side is the design of `docs/hardware/RF_DESIGN.md` v2.0, whose parts are not yet selected. Pin numbers are provisional until the OBC schematic (`software/rust/firmware/src/gpio.rs`).

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
 (TONE_OK, XPDR_KEY, PTT_EXCITER readbacks: allocated on the RF board, OBC inputs not yet assigned; §4.3)
```

The OBC does not modulate, demodulate, run HDLC or the CTCSS decoder. The TNC and the repeat path are hardware on the
RF board, so no flight software is in the audio path (BUILD_VS_BUY §3.2).

## 4. Physical and Electrical Interface

### 4.1 Connector

One board-to-board or harness connector, OBC J-RF to RF J1, ≤ 150 mm, with signal grounds interleaved; shield to
chassis at the OBC end. Part and pin-out: open until the OBC and RF schematics (both not yet drawn). The table below is
the signal list the connector must carry.

### 4.2 Signals

| Signal | Dir. | MCU pin (provisional) | Type, levels | Default and fail-safe | Driven by |
|--------|------|-----------------------|--------------|-----------------------|-----------|
| GND (×3) | — | — | Signal return | — | — |
| TNC_TXD (OBC → TNC) | OBC→RF | PD5 (USART2 TX) | 3.3 V CMOS UART, 9600 baud, 8N1 | Idle high | `hal_radio_transmit()`, `software/rust/firmware/src/radio.rs` |
| TNC_RXD (TNC → OBC) | RF→OBC | PD6 (USART2 RX) | 3.3 V CMOS UART | — | TNC |
| RADIO_EN | OBC→RF | PB5 | 3.3 V, high = radio powered | **Low at reset** (push-pull low from boot; RF-side pull-down) | `rf_silence` (low for 45 min after deployment), then always high (ICD-013 §6.3) |
| RF_TX_EN | OBC→RF | PA3 | 3.3 V, high = PA supply on | **Low at reset**; RF-side pull-down, so a reset or unpowered OBC leaves the PA off | `rf_control`: high only when TX-OFF, loss-of-contact and RF-silence inhibits are all clear (ICD-013 §6) |
| TX_PWR_SEL | OBC→RF | PE3 | 3.3 V, low = 0.5 W, high = 1 W | Low at reset (0.5 W) | `downlink.h`: the commanded level (`SET_POWER`), capped at 0.5 W after a VSWR event |
| XPDR_EN | OBC→RF | PE4 | 3.3 V square wave, 100 Hz (`HAL_GPIO_HEARTBEAT_HZ`); the RF board's retriggerable monostable (≈ 50 ms) gives EN_OK | Low at reset (high-Z then low); **a level held high is "disabled"** | `hal_gpio_heartbeat()` with a lease renewed each main-loop pass (`software/rust/firmware/src/heartbeat.rs`) while `sqso_fsw::transponder` permits |
| TX_TEMP | RF→OBC | PA0 (ADC1 IN5) | 0–2.5 V, 10 mV/°C, 500 mV at 0 °C (TMP36-class) | — | `downlink_poll()`: hold above 70 °C, resume below 60 °C |
| TX_FWD | RF→OBC | PC0 (ADC1 IN1) | 0–2.5 V, square law, 2.5 V at 1 W | — | `downlink_poll()` |
| TX_REV | RF→OBC | PC1 (ADC1 IN2) | as TX_FWD | — | VSWR 3:1 (reflected > ¼ forward) on 3 samples: cap 0.5 W; at 0.5 W, stop 10 min |

The ADC scalings are those in `software/rust/firmware/src/adc.rs` and stay open (TBR in the code) until the detector
and sensor parts are selected.

### 4.3 Readbacks (allocated, not yet wired to the OBC)

BUILD_VS_BUY §3.2 gives the RF board three status outputs: TONE_OK (CTCSS valid), XPDR_KEY (repeat path keyed) and
PTT_EXCITER. The flight software measures keyed time on TX_FWD today. These three should be wired to spare OBC inputs
so that the transponder statistics and the key-down timer can be verified in flight (closure: the OBC schematic
assigns pins; the firmware reads them).

### 4.4 Power

| Rail | Source | Switched by | Load |
|------|--------|-------------|------|
| Radio (receiver, TNC, exciter, repeat path) | EPS 3.3 V / 5 V regulated | RADIO_EN load switch (on the RF board) | ≤ 0.15 W receiving; ≤ 30 mW repeat path |
| PA | Battery-derived rail (14.4 V nominal through a regulator, or a dedicated PA regulator) | RF_TX_EN load switch | ≈ 1.5 W keyed at 0.5 W RF, ≈ 2.7 W at 1 W |

Power-up order: RADIO_EN high, ≥ 10 ms, then the OBC sends the KISS parameter commands (§5.2) before the first frame
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
| RADIO_EN high to receiver ready | ≤ 100 ms | Receiver selection |

## 7. Verification

| Test | Method | Pass criterion | When |
|------|--------|----------------|------|
| KISS link | Loop through the EM TNC; frames of 1–328 bytes, all escape cases | Byte-exact; FCS appended on receive | EM |
| Inhibit lines | Drive RADIO_EN and RF_TX_EN low, high, high-Z, OBC in reset | RF board state as §4.2 in every case | EM (VI-RF-12, VI-RF-13) |
| Heartbeat | Stop, stall, halt the OBC with XPDR_EN running | Repeat path unkeys ≤ 100 ms (VI-RF-11) | EM |
| Monitors | Calibrate TX_FWD, TX_REV against a power meter; TX_TEMP against a thermocouple | ±0.5 dB, ±2 °C | EM |
| Power select | TX_PWR_SEL low/high | 27.0 / 30.0 ± 0.5 dBm (VI-RF-02) | EM |

## 8. Open Items

| Item | Closure |
|------|---------|
| Connector part and pin-out | OBC and RF schematics |
| Readback pins for TONE_OK, XPDR_KEY, PTT_EXCITER | OBC schematic, firmware |
| ADC scaling of TX_TEMP, TX_FWD, TX_REV | Part selection (RF_DESIGN §9) |
| TNC part | RF_DESIGN §9 |

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline (Si4463 SPI, 9600 GFSK) |
| 1.1 | 2026-10-16 | SMART-QSO Team | Beacon Format ICD cited by its own number, SMART-QSO-ICD-006 (finding PI-17) |
| 1.1, note | 2026-10-24 | SMART-QSO Team | RF_TX_EN row names the Rust file that replaced `rf_control.c` |
| 2.0 | 2026-10-27 | CDR review, communications panel | Rewritten to the interface as built: KISS TNC on USART2 (CR-2026-005), RADIO_EN, RF_TX_EN, TX_PWR_SEL, the XPDR_EN heartbeat, the transmitter monitors; KISS parameters; timing; readbacks allocated; verification. The Si4463 SPI command set, 9600 bps GFSK and 145.825 MHz are removed |
