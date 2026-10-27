# SMART-QSO Telemetry Dictionary

**Document ID**: SQSO-TLM-001
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines all telemetry parameters for the SMART-QSO mission. It serves as the authoritative reference for telemetry interpretation.

## 2. Telemetry Frame Structure

### 2.1 Frame Format

```
┌──────────────────────────────────────────────────────────────┐
│                    TELEMETRY FRAME (64 bytes)                │
├──────┬──────┬──────┬───────────────────────────┬─────┬──────┤
│SYNC  │ SEQ  │ TIME │         PAYLOAD           │FLAGS│ CRC  │
│2 byte│2 byte│4 byte│        52 bytes           │2byte│2 byte│
└──────┴──────┴──────┴───────────────────────────┴─────┴──────┘
```

### 2.2 Header Fields

| Field | Offset | Size | Format | Description |
|-------|--------|------|--------|-------------|
| SYNC | 0 | 2 | 0xAA55 | Frame sync word |
| SEQ | 2 | 2 | uint16 | Sequence counter (0-65535) |
| TIME | 4 | 4 | uint32 | Seconds since boot |

### 2.3 Trailer Fields

| Field | Offset | Size | Format | Description |
|-------|--------|------|--------|-------------|
| FLAGS | 58 | 2 | uint16 | Status flags |
| CRC | 60 | 2 | uint16 | CRC-16-CCITT |

## 3. Telemetry Parameters

### 3.1 Power System (EPS)

| ID | Name | Offset | Size | Type | Units | Range | Resolution |
|----|------|--------|------|------|-------|-------|------------|
| TLM-001 | BATT_V | 8 | 2 | uint16 | mV | 2500-4200 | 1 mV |
| TLM-002 | BATT_I | 10 | 2 | int16 | mA | -2000 to +2000 | 1 mA |
| TLM-003 | BATT_SOC | 12 | 1 | uint8 | % | 0-100 | 1% |
| TLM-004 | SOLAR_V | 13 | 2 | uint16 | mV | 0-6000 | 1 mV |
| TLM-005 | SOLAR_I | 15 | 2 | uint16 | mA | 0-1000 | 1 mA |
| TLM-006 | BUS_V | 17 | 2 | uint16 | mV | 3000-3600 | 1 mV |
| TLM-007 | LOAD_I | 19 | 2 | uint16 | mA | 0-2000 | 1 mA |

**Conversion Formulas**:
```
BATT_V (V) = raw_value / 1000.0
BATT_I (A) = raw_value / 1000.0  (signed)
BATT_SOC (%) = raw_value
```

### 3.2 Thermal System

| ID | Name | Offset | Size | Type | Units | Range | Resolution |
|----|------|--------|------|------|-------|-------|------------|
| TLM-010 | OBC_TEMP | 21 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-011 | EPS_TEMP | 23 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-012 | BATT_TEMP | 25 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-013 | RF_TEMP | 27 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-014 | JETSON_TEMP | 29 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-015 | STRUCT_TEMP | 31 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |

**Conversion Formula**:
```
TEMP (°C) = raw_value / 10.0
```

### 3.3 State and Mode

| ID | Name | Offset | Size | Type | Units | Values |
|----|------|--------|------|------|-------|--------|
| TLM-020 | STATE | 33 | 1 | enum | - | See 3.3.1 |
| TLM-021 | MODE_TIME | 34 | 4 | uint32 | sec | Time in current mode |
| TLM-022 | BOOT_COUNT | 38 | 2 | uint16 | - | Total boot count |
| TLM-023 | BOOT_REASON | 40 | 1 | enum | - | See 3.3.2 |
| TLM-024 | UPTIME | 41 | 4 | uint32 | sec | Seconds since boot |

#### 3.3.1 STATE Values

| Value | Name | Description |
|-------|------|-------------|
| 0 | INIT | Initializing |
| 1 | SAFE | Safe mode |
| 2 | IDLE | Idle mode |
| 3 | ACTIVE | Active mode |
| 255 | FAULT | Fault state |

#### 3.3.2 BOOT_REASON Values

| Value | Name | Description |
|-------|------|-------------|
| 0 | POWER_ON | Normal power-on |
| 1 | WDT_RESET | Watchdog reset |
| 2 | SW_RESET | Software reset |
| 3 | CMD_RESET | Command reset |
| 4 | FAULT_RESET | Fault recovery |
| 5 | BANK_SWITCH | Software bank switch |

### 3.4 RF System

| ID | Name | Offset | Size | Type | Units | Range |
|----|------|--------|------|------|-------|-------|
| TLM-030 | TX_ENABLED | 45 | 1 | bool | - | 0/1 |
| TLM-031 | TX_POWER | 46 | 1 | enum | - | 0=0.5W, 1=1W, 2=2W |
| TLM-032 | TX_COUNT | 47 | 4 | uint32 | - | Beacons transmitted |
| TLM-033 | RX_COUNT | 51 | 4 | uint32 | - | Commands received |
| TLM-034 | LAST_RSSI | 55 | 1 | int8 | dBm | -128 to 0 |

### 3.5 ADCS System

| ID | Name | Offset | Size | Type | Units | Range |
|----|------|--------|------|------|-------|-------|
| TLM-040 | SUN_DETECTED | 56 | 1 | bool | - | 0/1 |
| TLM-041 | TUMBLE | 57 | 1 | bool | - | 0=stable, 1=tumbling |

### 3.6 Payload (Jetson)

| ID | Name | Bits | Type | Description |
|----|------|------|------|-------------|
| TLM-050 | JETSON_STATE | FLAGS[0:1] | enum | 0=OFF, 1=BOOT, 2=READY, 3=BUSY |
| TLM-051 | AI_AVAILABLE | FLAGS[2] | bool | AI model loaded |
| TLM-052 | FALLBACK_MODE | FLAGS[3] | bool | Using fallback beacons |

### 3.7 Fault Counters

| ID | Name | Bits | Type | Description |
|----|------|------|------|-------------|
| TLM-060 | WDT_RESETS | FLAGS[4:7] | uint4 | Watchdog reset count (0-15) |
| TLM-061 | FAULT_FLAGS | FLAGS[8:15] | uint8 | Active fault indicators |

#### 3.7.1 FAULT_FLAGS Bit Definitions

| Bit | Name | Meaning |
|-----|------|---------|
| 0 | BATT_LOW | Battery below 20% |
| 1 | BATT_CRIT | Battery below 15% |
| 2 | TEMP_HIGH | Over-temperature |
| 3 | TEMP_LOW | Under-temperature |
| 4 | COMM_ERR | Communication error |
| 5 | SENSOR_ERR | Sensor error |
| 6 | SW_ERR | Software error |
| 7 | RESERVED | Reserved |

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
de KK6GDB-11: <AI or template text> | T=<unix s> M=<SAFE|IDLE|ACTIVE> SOC=<%> BV=<V.d> SUN=<0|1> RF=<0|1> QSO=<count>
```

This is the format `beacon_format_payload()` produces (`software/rust/sqso-fsw/src/beacon.rs`). The text is cut to the space left in
the 256-byte information field after the telemetry fields (at most 176 characters with this call sign); the telemetry
fields are never cut.

Every telemetry value is a measurement. A field with no measurement, or one older than 60 s (`BEACON_MEAS_MAX_AGE_MS`), is
sent as `-`, for example `SOC=- BV=-`. Decoders must treat `-` as "not measured", never as zero. `QSO` is `-` until a QSO
detector exists. `RF` is the radio power switch state as the EPS reports it.

Example: `de KK6GDB-11: Hello from orbit! | T=1792115511 M=ACTIVE SOC=83 BV=8.1 SUN=1 RF=1 QSO=-`

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
bytes (`software/rust/sqso-fsw/src/housekeeping.rs`, SRS-F085, SRS-F126):

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
| `PS` | Passivation: phase 0 idle, 1 armed, 2 firing, 3 passivated, 4 final report (`passivation.h`) / the EPS `EOL_STATE` register (bit 0 armed, bit 1 fired: charge path open, bit 2 fuse F1 intact; `-` no answer) / fire attempts (RID REG-12, finding EOL-1) | |
| `RX` | Command-receiver recovery cycles since boot / known failed 1/0 (`receiver.h`) | |
| `CN` | Authenticated commands counted as contact since boot | |
| `ST` | Stored housekeeping: next five-minute / sixty-second record number (`DUMP_TLM`) | |
| `FR` | FRAM: 0 working, 1 declared failed (safe defaults kept; `FRAM_BYPASS` accepted), 2 the degraded mode (RAM copies under CRC) / failed one-minute checks since boot (RID SMA-B10, SRS-F160) | |

Example: `HK1 N=7 UP=3600 BOOT=3 RST=2 MODE=3 HOLD=1 XP=0 TX=1 SOC=873 VB=15812 IB=-120 PA=2410 TB=123 TO=215 TP=301
TJ=- DEP=8/15/1 RFS=3/2700 CT=30 CS=2 FC=4 FL=17/2 W=12/-5/600 AM=2 MR=0 GF=1200/7 GS=5/4 PS=1/4/0 RX=2/0 CN=3 ST=12/60 FR=0/0`
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
| `JB` | Jetson bursts since boot / last burst's energy / its lines / bursts ended by the energy gate | — / J / — / — |
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
| `PD` | PDOP × 10 | |
| `AGE` | Seconds from the fix to this frame: 0 after the session, larger on the repeat | s |

Example: `GNS N=17 WK=452 TOW=345600000 X=-4349115 Y=4803225 Z=2239564 VX=-4581234 VY=-1912078 VZ=5870431 VDT=30 Q=1
SV=8 PD=18 AGE=0` (one line on the air). Decoder: `software/ground/tools/gnss_frame.py` (SQSO-GND-002); unknown keys are
ignored. The stored housekeeping records (§4.6) do not carry the fix: their 44-byte layout and FRAM regions are full; HK1
`GF` carries its age and satellites.

### 4.5 Command Acknowledgement

After each authenticated command counted as contact: `ack KK6GDB-11: C=<counter> R=<result> N=<days since contact>`
(SQSO-ICD-005 §5.5, SRS-F125). `R` is the `CmdResult_t` code; `N` = 0 confirms the command was counted as contact.

### 4.6 Stored Housekeeping Records

`DUMP_TLM` (0x0A) downlinks the stored records in binary frames starting `HKD`: 44-byte records every 5 minutes for 7.1
days and every 60 s for 6.8 h and the 6 h around the last three events. Frame and record layout: SQSO-ICD-005 §5.6
(SRS-F120 to SRS-F122).

## 5. Telemetry Limits

### 5.1 Operational Limits

| Parameter | RED_LOW | YEL_LOW | YEL_HIGH | RED_HIGH |
|-----------|---------|---------|----------|----------|
| BATT_V | 3000 | 3200 | 4100 | 4200 |
| BATT_SOC | 15 | 25 | - | - |
| OBC_TEMP | -350 | -200 | 700 | 800 |
| BATT_TEMP | -50 | 0 | 400 | 450 |
| JETSON_TEMP | -150 | -100 | 600 | 700 |

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
    "voltage": 3.85,
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
2026-01-02T12:34:56Z,12345,ACTIVE,3.85,0.3,78,25.0,22.0,45.0
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
