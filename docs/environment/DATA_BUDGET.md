# SMART-QSO Data Budget

**Document ID**: SQSO-DB-001
**Version**: 2.4
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR analysis (finding TB-12). Values from BASELINE Rev E §8, BEACON_SPEC, the flight software as built (round 2: stored housekeeping and `DUMP_TLM`), the mission simulation (two-year `full` run at the Rev E baseline, 2026-10-27) and SQSO-LBA-001 v3.2 / SQSO-GND-001 Rev D (475 km, 8°/s). Not measured.

## 1. Purpose and Summary

Version 1.0 counted broadcast beacons against one station's pass capacity, generated 1.2 MB/day of housekeeping against
252 KB/day of capacity, and allocated 632 KB of flash partitions on a 512 KB part (TB-12). Version 2.0 separated the
budgets and found stored housekeeping missing; Version 2.1 puts in the flight software's store and dump as built.

| Budget | Requirement | Result | Verdict |
|--------|-------------|--------|---------|
| Broadcast telemetry (the beacon) | Beacon every 60 s ACTIVE / 120 s IDLE / 180 s SAFE (BASELINE §8) | ≈ 1,430 beacons and ≈ 285 HK1 frames a day; 1,725 frames/day radiated in all, 2.7 % transmitter duty | Met |
| Telemetry received by the mission | Health visible every day | ≈ 26 beacons and 5 HK1 frames a day at one telecommand station (≈ 4 kB), ≈ 51 and 10 at two, up to ≈ 490 beacons by SatNOGS | Met for health monitoring |
| Stored housekeeping (BASELINE §8: summarized every 5 minutes, ≥ 7 days onboard; 60 s records around faults and mode changes) | 7 days of five-minute records, retrievable by telecommand | FRAM store of 2,048 five-minute records (**7.1 days**) and 1,489 sixty-second records; `DUMP_TLM` (0x0A) | Met by design (flight software, SIL); verify on the EM |
| Daily dump | The ground can follow the five-minute record stream | The whole stream (288 records/day) needs 72 dump frames, 14.4 min of commanding passes: **62 % of one station's** with the 10° commanding rule (72 % at 15°), 31 % of two stations'. The SIL plan dumps the newest 48 records a day (17 % of the stream) | Met with two stations; one station only with most of its commanding time |
| Uplink | Commands per pass | ≤ 6 verified commands per pass at the beacon rate; capacity far above need | Met |

## 2. Broadcast Telemetry

### 2.1 Beacon frames

| Item | Value | Source |
|------|-------|--------|
| Information field | `de KK6GDB-11: <text ≤ 176> \| T=.. M=.. SOC=.. BV=.. SUN=.. RF=.. QSO=..`; typical 110 bytes, at most 256 | BEACON_SPEC §3 |
| Housekeeping frame | Every fifth beacon is followed, 5 s later, by an HK1 frame: `KEY=value` pairs, at most 250 bytes a frame; a longer text goes in a second frame (HK1 at its widest 341 bytes; SQSO-TLM-001 1.5, SIL-40), then HK2 | `docs/operations/TELEMETRY_DICTIONARY.md` §4.4 |
| Frame on air | 16-byte header + information + 2-byte FCS + flags, ≈ 3 % bit stuffing, 300 ms TXDELAY | SQSO-ICD-003 §5.2 |
| Beacon interval | 60 / 120 / 180 s with a uniform ±5 s jitter on each interval (RID RF-04) | SQSO-ICD-005 v2.1 §5.3 |
| Frames per day | **1,725** radiated over the two-year run (1,259,310 in 730 days): ≈ 1,430 beacons, ≈ 285 HK1 frames, 1.9 acknowledgements and 11.9 dump frames | SIL `full` run |
| Keyed time | 2,372 s/day (**2.7 %**; 1.38 s a frame on average) | SIL `full` run |
| Broadcast volume | ≈ 1,430 × 110 B + 285 × ≤ 250 B ≈ **157 kB + ≤ 71 kB a day** | |

The beacon is broadcast: every station under the footprint receives the same frames, so the broadcast volume is not a
capacity budget. What the mission sees is set by the receive network (§3).

### 2.2 AI text

At most one Jetson text per request, carried in the beacon's text field (BEACON_SPEC §5); in the two-year simulation
173,082 of the frames (≈ 14 % of the beacons) carried AI text. It adds nothing to the volume: the text field is there
either way.

## 3. Telemetry Received

Passes and frame success at 475 km, LTAN 22:30, spin 8°/s (SQSO-LBA-001 v3.2 §5; SQSO-GND-001 Rev D §4); 98.2 % of
frames decoded at a class A station with the beacon jitter as built.

| Receiver | Passes/day ≥ 5° | Contact | Beacons in view/day | Frames decoded | Beacons + HK1/day received | Information/day |
|----------|-----------------|---------|---------------------|----------------|-----------------------------|-----------------|
| One telecommand station (Santa Clara, class A; a candidate site, nothing agreed: SQSO-GND-001 §3.3) | 3.77 | 26.1 min | 26.1 | 98.2 % | **25.6 + 5.1** | 2.8 + ≤ 1.3 kB |
| Two telecommand stations (Santa Clara + Blacksburg, 3,640 km apart; Blacksburg an example East Coast site, SQSO-GND-001 §3.3) | 7.55 | 51.9 min (union) | 51.9 | ≈ 98 % | **≈ 51 + 10** | 5.6 + ≤ 2.6 kB |
| SatNOGS: 253 2 m stations active in the last 7 days, 49 directional (frame success 0.6 assumed) and 204 omni (0.35, SQSO-LBA-001 §5.3) | | Visible to ≥ 1 station 50 % of the time | 717 | 34 % of all beacons decoded by at least one station if every station observes | **≤ 488** beacons (upper bound; SatNOGS observations must be scheduled) | ≤ 54 kB |

Network figures: `software/simulation/cdr/rf/pass_geometry_mc.py` (orbit, station geometry) against the station list
of `network.satnogs.org/api/stations` (retrieved 2026-10-27), 3 days from 2028-11-01, 10 s steps, 5° mask (Version 2.0,
at 500 km: 52 %, 745 in view, ≤ 494).

Each beacon carries the whole health picture (mode, SOC, battery voltage, sunlit, RF state). Received beacons at one
station every ≈ 13 h (the longest pass gap, 12.9 h) are enough to follow the spacecraft's state; SatNOGS fills the
gaps.

## 4. Stored Housekeeping

### 4.1 The store as built (flight software round 2; SRS-F120 to SRS-F122)

| Item | Value |
|------|-------|
| Record | 44 bytes: store id, version, sequence, uptime, boot count, mode, flags, SOC, battery voltage and current, array power, four temperatures, hours since contact, fault count and newest fault, body rate, ADCS mode, deployment state, CRC-32 (SQSO-ICD-005 v2.1 §5.6; `software/rust/sqso-fsw/src/hk_store.rs`) |
| Five-minute store | 2,048 records × 44 B = **90,112 B**: the last **7.1 days** (BASELINE §8 asks ≥ 7 days) |
| Sixty-second store | 1,489 records = **65,516 B**: the last 409 records (6.8 h) plus the 6 h around each of the last three faults or mode changes (three windows of 360 records, 3 h either side) |
| Index | 256 B |
| FRAM used | 155,884 B of the 192 KB telemetry areas (`hardware/obc/STORAGE_DESIGN.md` §3.3): 79 % |
| Generated | Five-minute: 288 records/day = 12.7 kB/day. Sixty-second: 1,440 records/day = 63.4 kB/day, of which only the rolling 6.8 h and the event windows are kept |
| Retrieval | `DUMP_TLM` (0x0A, operational key): store, first record, count (≤ 600; 0 stops a dump). Four records a frame, **one frame every 12 s at most**, after the beacon and HK1 frames and through the same inhibits and duty limit (SQSO-ICD-013 §7) |
| Dump frame | `HKD` header (3 B), store, record count, 4 × 44 B: **181 B** information field, **≈ 1.6 s on air** at 1200 bps |
| Acknowledgement | Each authenticated command counted as contact is acknowledged: `ack KK6GDB-11: C=… R=… N=…`, **< 48 B** (SQSO-ICD-005 v2.1 §5.5) |

### 4.2 Daily downlink budget

A dump is started by a command, so it runs on commanding passes. Under decision C-14 commands are sent above 10°, so
a commanding pass is one that reaches 10°: at Santa Clara **2.98 a day with 23.2 minutes above 5°** on them
(`software/simulation/cdr/rf/pass_geometry_mc.py`, 60 days at 475 km; 2.43 a day and 20.1 minutes with the earlier
15° mask, SQSO-GND-001 Rev D §3.3). At one frame every 12 s that is **116 dump frames, 464 records a day** at most from
one station (100 and 400 at 15°; about 40 frames, 160 records, in a typical 8-minute pass); a single `DUMP_TLM` of 600
records (150 frames, 30 minutes) spans several passes.

| Dump plan | Records/day | Frames/day | Pass time | Keyed | Information | Transmitter energy (1.84 W DC at 0.5 W RF, BASELINE §5) | Share of one station's commanding time |
|-----------|-------------|------------|-----------|-------|-------------|-------------------------------|----------------------------------------|
| SIL plan: newest 48 records a day (from the HK1 `ST=` field) | 48 (4 h of five-minute records) | 12 | 2.4 min | 19 s | 2.2 kB | 0.010 Wh | 10 % |
| The whole five-minute stream | 288 | 72 | 14.4 min | 115 s | 13.0 kB | 0.059 Wh | **62 %** (31 % with two stations; 72 % with the 15° mask) |
| One event window from the sixty-second store, on demand | 360 | 90 | 18 min | 144 s | 16.3 kB | 0.074 Wh | 78 % for a day |
| Whole sixty-second store (rare) | 1,489 | 373 | 75 min | 597 s | 67.5 kB | 0.31 Wh | ≈ 3.2 days of one station |

| Daily downlink at one station (SIL plan) | Frames | Information |
|------------------------------------------|--------|-------------|
| Beacons received | 25.6 | 2.8 kB |
| HK1 frames received | 5.1 | ≤ 1.3 kB |
| Dump frames | 12 | 2.2 kB |
| Acknowledgements (≈ 2 commands a day: the routine contact and the dump) | ≈ 2 | < 0.1 kB |
| **Total** | **≈ 45** | **≈ 6.4 kB** |

Over the two-year simulation the plan produced 8,664 dump frames (34,656 records, 11.9 frames a day) and 1,370
acknowledgements. Following every five-minute record needs 72 frames a day, which one station can receive only by
spending most of its commanding time on dumps (62 %); with the two stations of BASELINE §1 it takes about a third of
their time. Recommended plan (for the operations owner): with two stations, dump the whole five-minute stream daily; with
one, dump the newest 48–96 records a day and fetch the event windows after each fault.

### 4.3 Conflicts and limits

| Item | Value |
|------|-------|
| Duty | A dump keys 1.6 s in each 12 s (13 %); with the beacon and HK1 frames it stays inside the 25 % duty limit (`DOWNLINK_DUTY_PERCENT`) |
| Transponder | Dump frames and the transponder share the transmitter; a dump displaces about 13 % of the transponder time while it runs (115 s a day for the whole five-minute stream) |
| Storage wear | FRAM, rated 10¹⁴ writes: no wear limit |

## 5. Uplink

| Item | Value |
|------|-------|
| Telecommand frame | 16-byte AX.25 header + 4 + 4 (counter) + N arguments (≤ 64) + 16 (tag) + 2 (sum16) + 2 (FCS): 44–108 bytes, 0.6–1.0 s on air with the ground's TXDELAY |
| Commands per pass | Not limited by the link: by the verification loop (one beacon interval each, SQSO-GSOM-001 §7.2), ≈ 6 verified commands in a 7-minute pass |
| Routine need | One authenticated contact a day (SQSO-GND-001 GND-R8) |
| Software or key upload | **None**: no flight-software upload path exists, and no key material crosses RF (SQSO-ICD-013 §9.3). Version 1.0's "software upload, ~1,000 commands" is withdrawn |

## 6. Latency

| Path | Latency |
|------|---------|
| Measurement to beacon | ≤ 60 s (values older than 60 s are sent as `-`) |
| Beacon to a station | Next pass: ≤ 12.9 h at one station, ≤ 9.7 h at two |
| Station to the mission database | After each pass (≤ 1 h); SatNOGS polled hourly |
| Command to effect | < 1 s onboard; confirmed in the next beacon (≤ 60 s) |

## 7. Verification

| Item | Method | When |
|------|--------|------|
| Beacon cadence and size | SIL (done: 1,725 frames/day in the two-year run, 1.38 s average); EM on the target | EM |
| Received beacons per day | Commissioning: count frames in the database by source | Commissioning |
| Stored housekeeping and dump | Flight software (`hk_store.rs`, `DUMP_TLM`) and the SIL's daily dump; EM test of a 600-record dump over several passes | EM |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR review, communications panel | Rewritten (TB-12): broadcast, received and stored budgets separated; SIL frame counts; SatNOGS network estimate from the live station list; stored housekeeping found not implemented and without a downlink path, and its sizing against FRAM, internal flash and dump capacity; uplink and latency |
| 2.1 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | Received telemetry at 475 km and 8°/s (3.77 passes, 26.1 min a day; 98.2 % of frames with the beacon jitter as built; SatNOGS ≤ 488 a day); broadcast counts from the two-year SIL run (1,725 frames a day, 2.7 % duty); §4 rewritten to the flight software's store (2,048 five-minute records, 7.1 days; 1,489 sixty-second records; 256 B index) and `DUMP_TLM` (≤ 600 records, 4 a frame, one frame per 12 s, 181 B); daily downlink budget recomputed: the whole five-minute stream needs 72 % of one station's commanding time; the stored-housekeeping verdict is now met by design |
| 2.2 | 2026-10-27 | CDR S&MA analysis (decision C-14) | Commanding passes at the 10° rule: 2.98 a day, 23.2 min above 5° (was 2.43, 20.1 min at 15°); dump capacity 116 frames, 464 records a day; the whole five-minute stream takes 62 % of one station's commanding time (was 72 %) |
| 2.3 | 2026-10-27 | CDR flight software, round 3 (fourth part) | §2.1 the housekeeping text in one or more frames of at most 250 bytes (SIL finding SIL-40) |
| 2.4 | 2026-10-27 | Documentation audit (post-CDR 2026) | §4.2 dump energies at the RF board's designed 1.84 W DC at 0.5 W RF (BASELINE §5; was 1.76 W of an earlier RF board revision): 0.010 / 0.059 / 0.074 / 0.31 Wh; §3 the Santa Clara and Blacksburg stations labelled as candidate and example sites (nothing agreed, BASELINE §1) |
