# SMART-QSO Telemetry Dictionary

**Document ID**: SQSO-TLM-001
**Version**: 1.8
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft. §4 is the telemetry the flight software sends (the 2026-01 binary frame of earlier versions is removed, §2). No independent CDR has been held (BASELINE Rev E §1)

## 1. Purpose

This document defines all telemetry parameters for the SMART-QSO mission. It serves as the authoritative reference for telemetry interpretation.

## 2. Telemetry Frame Structure

The downlink frames are the beacon (§4.1–§4.3), the housekeeping frames HK1 and HK2 (§4.4, §4.4.1), the GNSS state
frame `GNS` (§4.4.2), the command acknowledgement (§4.5) and the stored housekeeping records (§4.6), all AX.25 UI
frames with plain-text information fields (SQSO-ICD-005 §5). There is no binary telemetry frame.

*Removed in Version 1.8 (documentation audit, 2026-10-27):* the 64-byte 0xAA55 frame of the 2026-01 design (sync,
sequence, time, 52-byte payload, flags, CRC-16) and its parameter table TLM-001 to TLM-061 (a single-cell battery of
2.5–4.2 V, a 2 W power level, a 3.3 V bus). The flight software never sent that frame; Version 1.7 and earlier keep it in
the repository history.

## 3. Telemetry Parameters

The parameters are the fields of the frames of §4, defined there with their units (beacon tail §4.2, HK1 §4.4, HK2
§4.4.1, `GNS` §4.4.2).

## 4. Beacon Content

### 4.1 AX.25 Beacon Frame

```
┌───────────────────────────────────────────────────────────────┐
│                      AX.25 UI FRAME                           │
├──────┬─────────────┬─────────────┬────┬────┬─────────┬───────┤
│ FLAG │ DESTINATION │   SOURCE    │CTRL│PID │  INFO   │  FCS  │
│ 0x7E │   7 bytes   │   7 bytes   │0x03│0xF0│≤256 byte│2 bytes│
└──────┴─────────────┴─────────────┴────┴────┴─────────┴───────┘
```

### 4.2 Beacon Info Field

The INFO field contains human-readable status and AI-generated content:

```
de KK6GDB-11: <AI or template text> | T=<s since boot> M=<SAFE|IDLE|ACTIVE> SOC=<%> BV=<V.d> SUN=<0|1> RF=<0|1> QSO=<count>
```

This is the format `beacon_format_payload()` produces (`software/rust/sqso-fsw/src/beacon.rs`). The text is cut to the space left in
the 256-byte information field after the telemetry fields (at most 176 characters with this call sign); the telemetry
fields are never cut.

Every telemetry value is a measurement. A field with no measurement, or one older than 60 s (`BEACON_MEAS_MAX_AGE_MS`), is
sent as `-`, for example `SOC=- BV=-`. Decoders must treat `-` as "not measured", never as zero. `QSO` is `-` until a QSO
detector exists. `RF` is the radio power switch state as the EPS reports it.

`T=` is seconds since the last OBC boot, not UTC (on the host simulation, host wall-clock seconds, as in the example);
UTC is HK2 `RT` (§4.4.1). Example (host simulation): `de KK6GDB-11: Hello from orbit! | T=1792115511 M=ACTIVE SOC=83 BV=15.8 SUN=1 RF=1 QSO=-`

The text before ` | ` passes the content filter (`BEACON_SPEC.md` section 6). It never contains `|` or digits other than
`73`, `1U` and the call sign, so the telemetry fields cannot be spoofed by the text.

### 4.3 Beacon Timing

| Parameter | Value |
|-----------|-------|
| Interval | 60 s ACTIVE, 120 s IDLE, 180 s SAFE (`BEACON_SPEC.md` section 7), each interval jittered by up to ±5 s (uniform, a new draw each beacon; the mean per mode unchanged; SRS-F127) |
| Duration | ~2 seconds |
| Frequency | 145.XXX MHz |

### 4.4 Housekeeping Frame (HK1)

Every fifth beacon is followed, 5 s later, by a UI frame whose information field starts `HK1 ` (and one is sent on
`GET_HOUSEKEEPING`, 0x09). Plain ASCII `KEY=value` pairs, integers only, `-` for a value not measured, at most 250
bytes a frame (`software/rust/sqso-fsw/src/housekeeping.rs`, SRS-F085, SRS-F126). **A text longer than a frame is sent
whole in as many frames as it needs (Version 1.5, SIL finding SIL-40):** cut at the spaces between fields, never inside
one, 5 s apart, each frame after the first starting with the first's tag and sequence (`HK1 N=7 `), so a decoder that
reads `KEY=value` pairs merges the frames of one `N`. HK1 with every field at its widest is 341 bytes (two frames);
in flight it passes 250 bytes once its counters grow (in the mission simulation after about 140 days, when Version 1.4's
single frame went out empty and the `ST=` the ground plans dumps with was lost). HK2 at its widest is 474 bytes (two
frames). The order on the air: the HK1 frames, then the HK2 frames, then the GNSS frame when one is due.

| Key | Value | Unit |
|-----|-------|------|
| `N` | Frame sequence (wraps at 65536) | |
| `UP` | Seconds since boot | s |
| `BOOT` | Resets since the mission data began | |
| `RST` | Last reset cause (`HalResetCause_t`) | |
| `MODE` | Operating state (`state_machine.h`) | |
| `HOLD` | Commissioning hold, 1/0 | |
| `XP` | Transponder enabled now, 1/0 | |
| `TX` | Transmission permitted, 1/0 | |
| `SOC` | State of charge | ‰ |
| `VB` | Battery voltage | mV |
| `IB` | Battery current, positive charging | mA |
| `PA` | Array power (battery voltage × array current, the current measured at the bus) | mW |
| `TB`, `TO`, `TP`, `TJ` | Battery, OBC, PA, Jetson heat-spreader temperature | 0.1 °C |
| `DEP` | Deployment state / AntS elements deployed (bit n−1 for element n, 15 all; `-` no answer) / wing sense | |
| `RFS` | RF-silence state / seconds counted | s |
| `CT` | Loss-of-contact timeout | days |
| `CS` | Time since the last authenticated command (the loss-of-contact count) | h |
| `FC` | Fault-log entries | |
| `FL` | Newest fault: type / severity (`fault_mgmt.h`) | |
| `W` | Body rate x / y / z | 0.01 °/s |
| `AM` | ADCS mode | |
| `MR` | Magnetometer samples rejected (coils on) | |
| `GF` | GNSS: age of the latest fix / satellites used | s |
| `GS` | GNSS sessions run / with a fix | |
| `PS` | Passivation: phase 0 idle, 1 armed, 2 firing, 3 passivated, 4 final report (`passivation.h`) / the EPS `EOL_STATE` register (bit 0 armed, bit 1 fired: charge path open, bit 2 fuse F1 intact; `-` no answer) / fire attempts (RID REG-12, finding EOL-1) / the fire's reasons, bits: 0 waiting for sunlight, 1 waiting for the EPS board to reach 0 °C (F1's rating; at most 6 h of each attempt), 2 fired with the board still cold after that bound, 3 the board's temperature unreadable, not waited for (RID SMA-B20, SRS-F111; 0: fired with every condition met; kept in the final report) | |
| `RX` | Command-receiver recovery cycles since boot / known failed 1/0 (`receiver.h`) | |
| `CN` | Authenticated commands counted as contact since boot | |
| `ST` | Stored housekeeping: next five-minute / sixty-second record number (`DUMP_TLM`) | |
| `FR` | FRAM: 0 working, 1 declared failed (safe defaults kept; `FRAM_BYPASS` accepted), 2 the degraded mode (RAM copies under CRC) / failed one-minute checks since boot (RID SMA-B10, SRS-F160) | |
| `AD` | AntS temperature gate (decision C-18, SRS-F165): antenna attempts deferred for the AntS temperature (persisted; deployed anyway after 30) / the AntS's last temperature read before an attempt (`-` none valid) | — / °C |

Example: `HK1 N=7 UP=3600 BOOT=3 RST=2 MODE=3 HOLD=1 XP=0 TX=1 SOC=873 VB=15812 IB=-120 PA=2410 TB=123 TO=215 TP=301
TJ=- DEP=8/15/1 RFS=3/2700 CT=30 CS=2 FC=4 FL=17/2 W=12/-5/600 AM=2 MR=0 GF=1200/7 GS=5/4 PS=1/4/0/3 RX=2/0 CN=3 ST=12/60 FR=0/0 AD=2/-19`
(one line on the air).

### 4.4.1 Second Housekeeping Frame (HK2)

Each HK1 is followed, 5 s later, by a frame starting `HK2 ` in the same format: the counters the mission success
criteria need (SQSO-MSC-001 §7, RID SMA-15) and the OBC board's supervision (`housekeeping.rs`, SRS-F156):

| Key | Value | Unit |
|-----|-------|------|
| `N` | The HK1 sequence it follows | |
| `CA` | Telecommands executed / refused / failed authentication since boot | |
| `WD` | Resets by the watchdog (kept across resets) | |
| `XE` | Transponder enables since boot / repeater keyed (XPDR_KEY) / exciter keyed (PTT_EXCITER) | — / s / s |
| `IA` | Array current (INA228 at 0x40, else the EPS controller's) | mA |
| `HL` | Hard-limit violations since boot (Jetson on outside ACTIVE, sunlight or SOC ≥ 55 %, or past its burst limit) | |
| `JB` | Jetson bursts since boot / last burst's energy at the carrier input / its lines accepted by the content filter / bursts ended by the energy gate (decision C-20: a burst ends when its energy exceeds 360 J for each accepted line plus the one in progress, or at 3.6 kJ or 10 min; SRS-F151) | — / J / — / — |
| `JP` | Jetson switch-offs clean / cut after the 5 s wait / power-good faults / readback faults | |
| `EOL` | EPS `EOL_STATE` (as in `PS`; `-` no answer) / fire attempts | |
| `RT` | Mission clock (RV-3028) UNIX time / the last reset's time stamp | s / s |
| `CK` | The MCU clock against the mission clock's 1 Hz | ppm |
| `TR` | Latch-up trips of the gyro, BMI088, sun-sensor and magnetometer rails since boot (four numbers) | |
| `GY` | Gyro in use (0 M-G370, 1 BMI088, 2 none) / magnetometer samples gated (Jetson, exciter) | |
| `MF` | Magnetorquer bridge faults / off-timer expiries seen | |
| `PE` | SRAM2 parity on 1/0 (`-` not known) | |
| `SEP` | Separation latched on this boot 1/0 (DEPLOY_SENSE released for 10 s) / stowed readings since the latch (an open line; RID SMA-B02, SRS-F159) | |
| `PT` | TX_TEMP sensor faulted now 1/0 (the downlink then uses the board temperature + 15 °C) / faults since boot (RID SMA-B19, SRS-F161) | |
| `SV` | The EPS controller's `EPS_SUPV` (bits 0–3 trackers disabled, bit 4 AntS supply from the controller, bits 5–6 OBC power cycles in 24 h, bit 7 heartbeat supervision stopped; `-` no answer) / heartbeats the controller refused since boot (SRS-F162) | |
| `CI` | The EPS controller's `CHG_INHIBIT` (0x7A): bit 0 cold, 1 hot, 2 cell over-voltage, 3 BMS fault, 4 secondary protector blown, 5 recharge lockout latched, 6 NTC1 implausible (its window handed to the AFE on NTC2, SQSO-EPS-BRD-001 E §5.3); `-` no answer (SQSO-ICD-001 1.6 §4.6) | |
| `TS` | The TNC's last STATUS answer: version / criteria / floor / calibrated floor / carrier count / frames / uptime (`-` none yet) | |
| `RL` | RX_FAULT passed its power-on self-test 1/0 / the STATUS loop query unanswered for a day 1/0 (RID SMA-B11) | |
| `SL` | The OBC selector (SQSO-PAY-SEL-001 §6.4, SRS-F170, RID SMA-15): table in use 1/0, then decisions / abstentions of H1 (beacon template), H2 (transponder bucket) and H3 (power-mode preference) since boot: seven numbers. 0 everywhere until the selector is implemented (fabrication phase) | |
| `GE` | ADCS geometry preset in use (ADCS_SET_GEOMETRY 0x24: 0 both wings, 1 W1 only, 2 W2 only, 3 no wing; preset 0 unless the ground has selected another, decision C-21, SRS-F171) | |
| `SN` | Sun vector in the body frame x / y / z, from the sun sensors (`-` not visible) | 0.001 |

During the final passivation report (SRS-F155) HK1 and HK2 are sent three times, 60 s apart, before TX-OFF.

### 4.4.2 GNSS State Frame (GNS)

After each GNSS session that ended with a fix, and after HK2 on `GET_HOUSEKEEPING` (the latest fix again), one UI frame
whose information field starts `GNS ` (`software/rust/sqso-fsw/src/gnss_frame.rs`, SRS-F158, RID LV-04), through the
downlink's inhibits and duty limit (a frame the downlink refuses is not queued; `GET_HOUSEKEEPING` sends it again).
Plain ASCII, keys in this order, integers only, `-` for a value not available, at most 160 bytes:

```
GNS N=<session> WK=<week> TOW=<ms> X=<m> Y=<m> Z=<m> VX=<mm/s> VY=<mm/s> VZ=<mm/s> VDT=<s> Q=<quality> SV=<sats> PD=<PDOP x 10> AGE=<s>
```

| Key | Value | Unit |
|-----|-------|------|
| `N` | The GNSS session that took the fix (HK1 `GS`, first number); the repeat on `GET_HOUSEKEEPING` keeps it, so the ground drops duplicates by `N` | |
| `WK` | GPS week as the receiver reports it: the piNAV-NG's `$PSLSP` week, **10 bits** (0–1023, rolled over 1999-08-22 and 2019-04-07). The full week is the one of the form `WK` + 1024 k nearest the ground's clock (in 2028–2033: `WK` + 2048) | week |
| `TOW` | GPS time of week of the fix (the `$PSLSP` record's epoch), GPS time (not UTC: GPS − UTC = 18 s since 2017) | ms |
| `X`, `Y`, `Z` | Receiver antenna position at `TOW`, ECEF, WGS-84 | m |
| `VX`, `VY`, `VZ` | ECEF velocity at the same epoch (in the Earth-fixed, rotating frame). Derived on board: the receiver reports no velocity. The mean velocity over the chord from the session's first accepted fix to this one (`VDT` seconds) carried to the fix epoch with the Earth-fixed acceleration (point mass, J2, Coriolis, centrifugal) by Simpson's rule; method error below 0.01 m/s for chords up to 120 s; with position noise σ the velocity noise is about √2 σ / `VDT` (0.5 m/s for 10 m over 30 s). `-` when the session had no chord of 10 to 120 s | mm/s |
| `VDT` | The chord the velocity comes from (the session runs 30 s past its first fix, so normally 20–30 s) | s |
| `Q` | GGA fix quality with the fix: 1 GPS, 2 differential (the only ones the driver accepts) | |
| `SV` | Satellites used | |
| `PD` | PDOP × 10 (at most 999: a fix with PDOP above 99.9 is not plausible and not used, RID P3F-A-05) | |
| `AGE` | Seconds from the fix to this frame: 0 after the session, larger on the repeat | s |

Example: `GNS N=17 WK=452 TOW=345600000 X=-4349115 Y=4803225 Z=2239564 VX=-4581234 VY=-1912078 VZ=5870431 VDT=30 Q=1
SV=8 PD=18 AGE=0` (one line on the air). Decoder: `software/ground/tools/gnss_frame.py` (SQSO-GND-002); unknown keys are
ignored. The stored housekeeping records (§4.6) do not carry the fix (§4.6); HK1 `GF` carries its age and satellites.

### 4.5 Command Acknowledgement

After each authenticated command counted as contact: `ack KK6GDB-11: C=<counter> R=<result> N=<days since contact>`
(SQSO-ICD-005 §5.5, SRS-F125). `R` is the `CmdResult_t` code; `N` = 0 confirms the command was counted as contact.

### 4.6 Stored Housekeeping Records

`DUMP_TLM` (0x0A) downlinks the stored records in binary frames starting `HKD`: 44-byte records every 5 minutes for 7.1
days and every 60 s for 6.8 h and the 6 h around the last three events. Frame and record layout: SQSO-ICD-005 §5.6
(SRS-F120 to SRS-F122).

**No GNSS position in the stored records (decided at CDR round 3, fourth part; RID SMA-15).** The success criteria's
telemetry list (SQSO-MSC-001 §7) has no position item, and no criterion needs an onboard position history: L2-04 (fair
scheduling across regions) takes "transponder on/off telemetry and orbit" — the record's flags (bit 1 transponder
enabled) and its time (uptime and boot count, tied to UTC by HK2 `RT`) give the on/off history, and the orbit at each
record's time comes from the ground's ephemeris (the 18th SDS elements, refined with the `GNS` state frames of every
fix session, SRS-F158; `docs/ground/ORBIT_DETERMINATION_AND_EPHEMERIS.md`), which is more accurate between fixes than
any position the record could hold (the receiver is on for about 2 minutes an orbit). L1-04 and L2-07 need the
deployment, rate and array fields, which the record has. Adding the last fix (its age and a compact position, 8 bytes)
would make the record 52 bytes: the five-minute region (2,048 × 44 = 90,112 B) would become 106,496 B and the
sixty-second region (1,489 × 44 = 65,516 B, at its 64 KB bound) would no longer fit: 64 KB would hold 1,260 records,
which leaves the ring 180 beside the three 360-record windows, fewer than the 188 a window's copy needs, for no
criterion. STORAGE_DESIGN §3.4.2 keeps the map unchanged.

## 5. Telemetry Limits

### 5.1 Operational Limits

| Parameter | RED_LOW | YEL_LOW | YEL_HIGH | RED_HIGH |
|-----------|---------|---------|----------|----------|
| HK1 `VB` (pack, mV; 4S, BASELINE §5) | 11200 (BMS cut-off, 2.8 V/cell) | 13600 (hardware load shed) | 16400 (end of charge, 4.10 V/cell) | 17000 (cell over-voltage protection, 4.25 V/cell) |
| BATT_SOC | 15 | 25 | - | - |
| OBC_TEMP | -350 | -200 | 700 | 800 |
| BATT_TEMP | -50 | 0 | 400 | 450 |
| JETSON_TEMP (HK1 `TJ`, heat spreader) | -150 (power-on gate −15 °C) | -100 | 400 (+40 °C in a batch) | 700 (+70 °C stuck-on limit; SYS-603 Rev G) |

### 5.2 Limit Actions

| Limit | Color | Action |
|-------|-------|--------|
| RED_LOW | Red | Immediate notification |
| YEL_LOW | Yellow | Monitor closely |
| NOMINAL | Green | Normal operations |
| YEL_HIGH | Yellow | Monitor closely |
| RED_HIGH | Red | Immediate notification |

## 6. Telemetry Processing

### 6.1 Decoder Output Format (JSON)

```json
{
  "timestamp": "2026-01-02T12:34:56Z",
  "sequence": 12345,
  "uptime": 86400,
  "state": "ACTIVE",
  "battery": {
    "voltage": 15.6,
    "current": 0.3,
    "soc": 78
  },
  "thermal": {
    "obc": 25.0,
    "battery": 22.0,
    "jetson": 45.0
  },
  "rf": {
    "tx_enabled": true,
    "tx_power": 1,
    "tx_count": 1440
  },
  "faults": []
}
```

### 6.2 CSV Export Format

```csv
timestamp,seq,state,batt_v,batt_i,batt_soc,obc_temp,batt_temp,jetson_temp
2026-01-02T12:34:56Z,12345,ACTIVE,15.6,0.3,78,25.0,22.0,45.0
```

## 7. Telemetry Archival

### 7.1 Archive Requirements

| Data Type | Retention | Format |
|-----------|-----------|--------|
| Raw frames | Mission + 5 years | Binary |
| Decoded TLM | Mission + 5 years | JSON |
| Summary | Mission + 10 years | CSV |

### 7.2 Archive Naming

```
SQSO_TLM_YYYYMMDD_HHMMSS.json
SQSO_RAW_YYYYMMDD_HHMMSS.bin
SQSO_SUMMARY_YYYYMM.csv
```

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 2) | §4.3 beacon interval jitter; §4.4 the HK1 housekeeping frame with its fields (new: `PS`, `RX`, `CN`, `ST`; `PA` from the bus-side array current); §4.5 the command acknowledgement; §4.6 the stored housekeeping records |
| 1.2 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3) | §4.4 `PS` carries the EPS `EOL_STATE` (fuse F1 intact, bit 2) and the fire attempts (RID REG-12, finding EOL-1); §4.4.1 new: the second housekeeping frame HK2 (RID SMA-15) |
| 1.3 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, second part) | §4.4.2 new: the GNSS state frame `GNS` (RID LV-04, SRS-F158); HK1 `FR` (SMA-B10); HK2 `TR` with the magnetometer rail, `SEP`, `PT`, `SV` (SMA-B02, B19, B09) |
| 1.4 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, third part) | §4.4 HK1 `PS` with the fire's reasons (the EPS board ≥ 0 °C condition bounded at 6 h, RID SMA-B20); HK1 `AD`, the AntS temperature gate (decision C-18); §4.4.1 HK2 `CI`, the EPS controller's CHG_INHIBIT with bit 6 NTC1 implausible |
| 1.5 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, fourth part) | §4.4 a housekeeping text longer than a frame is sent in frames of whole fields with the tag and sequence repeated (SIL finding SIL-40: HK1 had gone out empty past 250 bytes); §4.6 why the stored records carry no GNSS position (RID SMA-15) |
| 1.6 | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, fifth part) | §4.4.1 HK2 `SL` selector decisions and abstentions per head (RID SMA-15), `GE` geometry preset and `SN` body sun vector (decision C-21), `TS` and `RL` listed; `JB` lines are accepted lines (decision C-20); HK2 text up to 573 bytes, cut into frames of whole fields; §4.4.2 every GNS and acknowledgement field has a fixed width, no frame is empty, PDOP ≤ 99.9 (RID P3F-A-05) |
| 1.7 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): status; §2–§3 marked superseded by §4 (the 2026-01 frame is not sent); §4.3 example BV=15.8 (was 8.1, a 2S value); §5.1 battery-voltage limits for the 4S pack in HK1 `VB` mV and Jetson spreader limits from BASELINE §6 (were single-cell 3.0–4.2 V and 60/70 °C); §6 decoder examples 15.6 V |
| 1.8 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026), follow-up: §2–§3 the superseded 2026-01 binary frame and its TLM-001 to TLM-061 table removed (pointers to the frames of §4); §4.2 the beacon `T=` field is seconds since the last OBC boot (`smart_qso_now_ms()`), not Unix/UTC seconds; UTC is HK2 `RT` |
