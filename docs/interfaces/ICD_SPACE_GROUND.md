# Interface Control Document: Space-to-Ground

**Document ID**: SQSO-ICD-005
**Version**: 2.4
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR. RF values follow BASELINE §7; frequencies are planning values until the IARU coordination letter.

## 1. Purpose

This document defines the interface between the SMART-QSO spacecraft and the ground segment: the RF links, the frame
formats, the ground stations' obligations and the operations interface. Version 1.x described a 9600 bps GFSK link on
145.825 MHz with circular polarization, 2 W, connected-mode AX.25 and the call sign "SQSO"; none of that is in the
baseline, and Version 2.0 replaces it.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-BL-001 Rev D | Baseline parameters |
| SQSO-LBA-001 v3.0 | Link budget (`docs/environment/LINK_BUDGET_ANALYSIS.md`) |
| SMART-QSO-ICD-006 | Beacon format (`docs/BEACON_SPEC.md`) |
| SQSO-ICD-013 | Telecommand authentication and TX-OFF (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`): **governs telecommand** |
| SQSO-GND-001 Rev C | Ground station design (`docs/ground/GROUND_STATION_DESIGN.md`) |
| SQSO-GSOM-001 | Ground station operations manual |
| `docs/operations/TELEMETRY_DICTIONARY.md` | Telemetry fields |

## 3. Interface Overview

```
                       SMART-QSO (KK6GDB-11)
         downlink 145.800-146.000 MHz      uplink 435.222-435.778 MHz (else 436.222-436.500)
         beacon (AX.25 UI, 1200 AFSK)      telecommand (AX.25 UI, 1200 AFSK, HMAC)
         FM transponder output             FM transponder input (CTCSS 67.0 Hz)
                 |                                     ^
   +-------------+--------------+                      |
   v             v              v                      |
 Telecommand   SatNOGS and     Amateur users  ---------+  (voice, through the transponder)
 stations (>=2) other receivers (handhelds,
 RX + TX       (RX only)       mobiles)
   |             |
   +------+------+---> mission database and public dashboard (SQSO-GND-001 §6)
```

## 4. RF Interface

### 4.1 Downlink (space to ground)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Frequency | One IARU-coordinated frequency in 145.800–146.000 MHz; planning value 145.9 MHz | Not 145.825 MHz |
| Frequency accuracy | ±1.0 ppm (±146 Hz) spacecraft; Doppler ±3.47 kHz, up to 53 Hz/s | RF-P005; SQSO-LBA-001 §4.4 |
| Beacon modulation | AFSK Bell 202 (1200/2200 Hz), NRZI, HDLC, on FM, ±3.0 kHz peak; 10K4F2D | |
| Transponder | FM voice, ±5.0 kHz peak; 16K0F3E | |
| Power | 0.5 W nominal, 1 W maximum (EIRP ≤ −1 dBW) | BASELINE §7 |
| Polarization | Linear (spinning spacecraft; polarization angle rotates with the spin and with Faraday rotation) | Circular ground antennas see a constant 3 dB loss; linear ones see fades |
| Silence | Nothing for 45 min after deployment; nothing while TX-OFF or the loss-of-contact inhibit is in force | SQSO-ICD-013 §6 |

### 4.2 Uplink (ground to space)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Frequency | One IARU-coordinated frequency, requested in 435.222–435.778 MHz, or else 436.222–436.500 MHz (BASELINE §7); planning value 435.5 MHz | Clear of 3 × f_down (SQSO-LBA-001 §7.1); inside the receiver's 435.050–436.500 MHz and clear of the EPS converter harmonics (SQSO-RF-BRD-001, SQSO-EPS-BRD-001) |
| Receiver | Always on after the RF silence; ±7.5 kHz IF; T_sys 1,000 K budgeted | RF-P006, RF-P010 |
| Telecommand modulation | AFSK Bell 202 on FM, ±3.0 kHz; AX.25 UI to KK6GDB-11; 10K4F2D | SQSO-ICD-013 §3 |
| Transponder access | FM voice with CTCSS 67.0 Hz at ±0.5–0.75 kHz | RF-P012 |
| Doppler correction | **Telecommand: pre-correct to ±1 kHz** (uncorrected 10 kHz does not decode). Users: 5 kHz steps across the pass | SQSO-LBA-001 §3.2, §4.4 |
| Polarization | Any; the spacecraft element is linear | |

### 4.3 Link summary (SQSO-LBA-001 v3.2, 475 km; analysis)

| Link | Case | Margin or SINAD |
|------|------|-----------------|
| Beacon, 0.5 W, reference station (12 dBic, 500 K) | 5°, median / 10th-percentile spacecraft gain | +7.9 dB / +0.9 dB |
| Telecommand, 64 dBm EIRP toward the spacecraft (decision C-14; either station variant), receiver as designed | 5° / 10°, 10th-percentile gain, +6 dB interference | +6.5 / +8.3 dB, less 0.3 dB pointing |
| Transponder, handheld Yagis both ends | 5° / 20° | 20 dB / 27 dB SINAD |
| Transponder, handheld whips both ends | 20° / zenith | 14 dB / 26 dB SINAD |

## 5. Protocol Interface

### 5.1 AX.25

AX.25 v2.2 **UI frames only** (control 0x03, PID 0xF0). The spacecraft does not support connected mode (SABM, UA, DISC
and DM in Version 1.x are removed). FCS CRC-16/X.25.

### 5.2 Addresses

| Station | Address | Notes |
|---------|---------|-------|
| Spacecraft | `KK6GDB-11` | Source of every beacon; destination of every telecommand (`station_id.h`) |
| Beacon destination | `CQ-0` | BEACON_SPEC §2 |
| Telecommand station | Its own call sign (source of its telecommands) | SQSO-ICD-013 §3.1 |

### 5.3 Beacon

The beacon's information field is plain ASCII (BEACON_SPEC §3):

```
de KK6GDB-11: <text> | T=<unix s> M=<SAFE|IDLE|ACTIVE> SOC=<%> BV=<V.d> SUN=<0|1> RF=<0|1> QSO=<count>
```

| Mode | Interval | Text |
|------|----------|------|
| ACTIVE | 60 s | Filtered AI text once per request, otherwise a template |
| IDLE | 120 s | Template |
| SAFE | 180 s | Template |
| EMERGENCY | None (beacon shed) | |

Every interval is jittered by a uniform draw of up to ±5 s, new for each beacon, so the mean interval of each mode is the
one in the table (SQSO-LBA-001 §5.3, RID RF-04, SRS-F127): two spacecraft on the same schedule do not collide pass after
pass. A station should not expect a beacon at an exact time.

Values not measured within 60 s are sent as `-`. Typical frame 110 bytes of information field, about 1.2 s on the air.
Every fifth beacon is followed by the housekeeping frame `HK1 ...` (SQSO-TLM-001, `docs/operations/TELEMETRY_DICTIONARY.md`),
which carries the loss-of-contact state (`CT=` timeout in days, `CS=` hours since contact) and the commands counted as
contact since boot (`CN=`).

### 5.4 Telecommand

Defined in SQSO-ICD-013, which governs: frame `0xAA 0x55 | cmd_id | len | counter (4, BE) | args | tag (16) | sum16 (2,
BE)` in the information field of a UI frame to KK6GDB-11; HMAC-SHA-256 truncated to 16 bytes; strictly increasing
counters per key; command dictionary in SQSO-ICD-013 §7. `TX_DISABLE` (0x32) ceases all emissions in every mode.

### 5.5 Command acknowledgement

After each authenticated command that counts as contact (SQSO-ICD-013 §6.2: its counter was committed, whatever its
result), the spacecraft downlinks one UI frame from KK6GDB-11 whose information field is plain ASCII (97.113(a)(4);
SRS-F125, RID GND-03):

```
ack KK6GDB-11: C=<counter> R=<result> N=<days since contact>
```

| Field | Meaning |
|-------|---------|
| `C` | The command's counter (decimal), which identifies the command at the station that sent it |
| `R` | Its result, `CmdResult_t` (SQSO-ICD-013 §8): 0 success, 1 invalid command, 2 invalid parameter, 4 not allowed, 5 execution failed, 6 busy |
| `N` | The whole days the loss-of-contact monitor counts after the command: 0 confirms the command was counted as contact |

| Property | Behaviour |
|----------|-----------|
| No acknowledgement | Forged or replayed frames, frames rejected before their counter is committed (SQSO-ICD-013 §8 steps 1 to 5), and the keyless `TX_DISABLE`: a frame that anyone can make must not key the transmitter. After `TX_DISABLE`, nothing is radiated, so no acknowledgement either |
| Path | Through the downlink and its inhibits and duty limit (TX-OFF, RF silence, loss of contact, the 25 % duty) like every frame; at most four wait, and one not sent within 30 s is dropped |
| Ground use | The station matches `C` to the command it sent; a missing acknowledgement means the command was not received, not counted, or the downlink was not heard. Repeat it with a new counter (Section 7.1) |

The `CmdResponse_t` that `cmd_process()` fills (the 4 bytes of `GET_STATUS`, the telemetry of `EPS_GET_TELEMETRY` and
`ADCS_GET_ATTITUDE`) is still not downlinked; `GET_HOUSEKEEPING` (0x09) gives the same values in an HK1 frame.

### 5.6 Stored housekeeping (DUMP_TLM)

`DUMP_TLM` (0x0A, SQSO-ICD-013 §7) asks for stored housekeeping records (SRS-F120 to SRS-F122, RID DB-01): store 0 holds
a record every 5 minutes for the last 7.1 days (2,048 records), store 1 a record every 60 s for the last 6.8 h (409
records) plus the 6 h around each of the last three faults or mode changes (three windows of 360 records). The frames
are UI frames from KK6GDB-11, at most one every 12 s, after the beacon and housekeeping frames and through the same
inhibits and duty limit; a count of 0 stops a dump, a count above 600 is refused, and records no longer stored are
skipped. The information field is binary, in the published format below (no encryption or obscured meaning,
97.113(a)(4)):

```
"HKD" (3) | store (1: 0 five-minute, 1 sixty-second) | records in this frame (1, at most 4) | records (44 each)
```

Record, 44 bytes, little-endian:

| Offset | Bytes | Field |
|--------|-------|-------|
| 0 | 1 | Store id: `S` (five-minute) or `D` (sixty-second) |
| 1 | 1 | Record version (1) |
| 2 | 4 | Sequence number (continues across resets; the first record of a store is 0) |
| 6 | 4 | Uptime, s |
| 10 | 2 | Boot count |
| 12 | 1 | Operating mode (`SystemMode_t`) |
| 13 | 1 | Flags: bit 0 commissioning hold, 1 transponder enabled, 2 TX permitted, 3 heater on, 4 payload on, 5 event (the record that started an event window), 6 loss-of-contact inhibit, 7 passivated |
| 14 | 2 | SOC, ‰ |
| 16 | 2 | Battery voltage, mV |
| 18 | 2 | Battery current, mA (signed, positive charging) |
| 20 | 2 | Array power, mW |
| 22 | 8 | Battery, OBC, PA and Jetson temperatures, 0.1 °C each (signed) |
| 30 | 2 | Hours since contact |
| 32 | 2 | Fault-log entries |
| 34 | 1 | Newest fault: type (`FaultType_t`) |
| 35 | 1 | Newest fault: severity |
| 36 | 2 | Body rate magnitude, 0.01 °/s |
| 38 | 1 | ADCS mode |
| 39 | 1 | Deployment state |
| 40 | 4 | CRC-32 (IEEE 802.3) over bytes 0 to 39 |

A value not measured is 0xFFFF (0x8000 for a signed field). The HK1 field `ST=<a>/<b>` gives the next sequence number
of each store, so a station knows the range to ask for. A day's five-minute records (288) take 72 frames, about 15
minutes of dump at the maximum rate; the mission simulation's plan asks for the newest 48 each day.

## 6. Ground Station Obligations

| ID | Obligation | Applies to |
|----|------------|------------|
| GS-1 | Designated in writing by the licensee (97.211(a)); listed in the IARU request and the pre-space notification | Telecommand stations |
| GS-2 | Holds the operational key under SQSO-ICD-013 §9 (offline at rest; never on a networked machine); the recovery key with a separate custodian | Telecommand stations |
| GS-3 | Counters per SQSO-ICD-013 §5.2 (shared sequence or UTC-derived, §5.2.1) | Telecommand stations |
| GS-4 | Doppler pre-correction to ±1 kHz; tracking antennas | Telecommand stations |
| GS-5 | ≥ 64 dBm EIRP (2.5 kW) toward the spacecraft above 10° elevation (decision C-14): variant A, outside the 47 CFR 2.106 US270 areas, ≥ 100 W PEP at the feed into ≥ 14 dBic; variant B, inside them (the Bay Area site near Beale AFB, Fairbanks near Clear SFS), **≤ 50 W PEP** at the feed into ≥ 17.5 dBic (a stacked pair of cross-Yagis, ≤ 0.5 dB feed); power within 47 CFR 97.313 and 2.106 US270 (≤ 50 W PEP at US270 sites); RF exposure per 1.1310 (SQSO-GND-001 Rev F §3.1a) | Telecommand stations |
| GS-6 | Able to send `TX_DISABLE` on the next pass after an FCC order (97.207(b)); 24-hour contact number | Telecommand stations |
| GS-7 | Forward decoded frames with time, station and Doppler-corrected frequency to the mission database | All receive stations |
| GS-8 | Transponder users identify per 97.119 and use CTCSS 67.0 Hz | Users |

## 7. Operational Interface

### 7.1 Telecommand pass

| Time | Action |
|------|--------|
| AOS − 10 min | Current orbit elements; clock checked against GNSS (±1 s); Doppler table loaded |
| AOS | Receive; wait for a beacon (it confirms the spacecraft is not silent and gives the frequency) |
| Above 15° | Send the planned commands, each with a new counter; up to four attempts a command, each checked by its acknowledgement (§5.5) |
| LOS | Log counters used, results, beacons heard; upload to the mission database |

### 7.2 Pass statistics (SQSO-GND-001 §4)

Santa Clara, 475 km SSO (the baseline target; SQSO-GND-001 Rev D §4): 3.77 passes a day above 5°, 6.9 min average, 26 minutes a day; 2.4 a day above 15°. Longest
gap between passes 12.9 h.

## 8. Security and Amateur-Radio Compliance

| Requirement | Implementation |
|-------------|----------------|
| No encryption | All downlink in the clear, formats published (97.113(a)(4)); telecommand authenticated, not encrypted (97.211(b)) |
| Station identification | KK6GDB-11 in every frame |
| Open access | Any amateur may receive and use the transponder |
| Pass predictions | Published (an amateur station; no reason to withhold them) |
| Rate limiting | None needed: rejected frames do not consume counters and cost the spacecraft only a CRC and an HMAC |

## 9. Timing

| Parameter | Value |
|-----------|-------|
| Spacecraft time | UTC seconds in the beacon `T=` field; set from GNSS when available (D6) |
| Propagation | 1.7–7 ms (500–2,078 km) |
| Command to effect | < 1 s onboard; seen in the next beacon (≤ 60 s in ACTIVE) |

## 10. Verification

| Test | Criterion | When |
|------|-----------|------|
| End-to-end RF compatibility: a telecommand station kit against the EM over an attenuated cable link at the budget's levels, with Doppler applied | Beacons decoded at the §4.3 levels; commands accepted at 53 dB-Hz; TX-OFF end to end | EM (with VI-RF-03, VI-RF-12) |
| SatNOGS decoder | The SatNOGS demodulator and the published decoder decode recorded EM beacons | Before launch |
| Beacon format | `software/ground/beacon_decoder.py` decodes every BEACON_SPEC field | Software test |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-15 | Funding audit | §6.3, §6.4, §9.2 and §11.2 aligned with the flight software; command framing moved to SQSO-ICD-013 |
| 1.2 | 2026-10-16 | SMART-QSO Team | Beacon Format ICD cited by its own number, SMART-QSO-ICD-006 (finding PI-17) |
| 1.3 | 2026-10-16 | Funding audit (TB-05) | §4.3 cites the re-issued link budget (SQSO-LBA-001 v2.0) |
| 2.0 | 2026-10-27 | CDR review, communications panel | Rewritten to the baseline: 1200 bps AFSK UI frames only, frequency ranges, linear polarization, power, Doppler rules, KK6GDB-11, beacon schedule per BASELINE §8, the binary beacon and response formats and connected mode removed, ground-station obligations, telecommand pass, link summary from SQSO-LBA-001 v3.0 |
| 2.1 | 2026-10-27 | CDR flight software, round 2 | §5.3 the ±5 s beacon jitter as implemented (RF-04) and the HK1 contact fields; §5.5 the command acknowledgement as implemented (GND-03; was a proposal); §5.6 new: stored housekeeping, DUMP_TLM frames and the 44-byte record (DB-01); §7.1 commands checked by their acknowledgement |
| 2.2 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: uplink request windows from BASELINE Rev E §7 (§2, §4); pass-analysis altitude noted |
| 2.3 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | §4.3 link summary from SQSO-LBA-001 v3.2 and §7.2 pass statistics from SQSO-GND-001 Rev D, both at 475 km (were 500 km) |
| 2.4 | 2026-10-27 | CDR S&MA analysis (decision C-14) | §4.3 telecommand margin at 64 dBm EIRP; GS-5 the EIRP requirement and the two station variants, ≤ 50 W PEP at US270 sites |
