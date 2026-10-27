# SMART-QSO Data Budget

**Document ID**: SQSO-DB-001
**Version**: 2.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR analysis (finding TB-12). Values from BASELINE §8, BEACON_SPEC, the flight software as built, the mission simulation (14-day commissioning run, 2026-10-27) and SQSO-LBA-001 v3.0 / SQSO-GND-001 Rev C. Not measured.

## 1. Purpose and Summary

Version 1.0 counted broadcast beacons against one station's pass capacity, generated 1.2 MB/day of housekeeping against
252 KB/day of capacity, and allocated 632 KB of flash partitions on a 512 KB part (TB-12). This version separates the
three budgets that matter:

| Budget | Requirement | Result | Verdict |
|--------|-------------|--------|---------|
| Broadcast telemetry (the beacon) | Beacon every 60 s ACTIVE / 120 s IDLE / 180 s SAFE (BASELINE §8) | 1,431 frames/day, 157 kB/day broadcast, 2.0 % transmitter duty | Met |
| Telemetry received by the mission | Health visible every day | ≈ 27 beacons/day at one telecommand station, ≈ 53 at two, up to ≈ 490 by SatNOGS | Met for health monitoring |
| Stored housekeeping (BASELINE §8: "housekeeping stored every 60 s; ≥ 7 days onboard storage") | 7 days of 60 s records | **Not implemented**: no onboard storage of housekeeping and **no telecommand to downlink stored data**. As specified, 7 days need 444 kB; the FRAM telemetry areas hold 192 KB (3.0 days); a one-station dump capacity of ≈ 44 kB/day is below the 63 kB/day generated | **Not met** (finding DB-01) |
| Uplink | Commands per pass | ≤ 6 verified commands per pass at the beacon rate; capacity far above need | Met |

## 2. Broadcast Telemetry

### 2.1 Beacon frames

| Item | Value | Source |
|------|-------|--------|
| Information field | `de KK6GDB-11: <text ≤ 176> \| T=.. M=.. SOC=.. BV=.. SUN=.. RF=.. QSO=..`; typical 110 bytes, at most 256 | BEACON_SPEC §3 |
| Frame on air | 16-byte header + information + 2-byte FCS + flags, ≈ 3 % bit stuffing, 300 ms TXDELAY | SQSO-ICD-003 §5.2 |
| Time on air, typical | 1.2 s (1.22 s average in the simulation) | SIL commissioning run: 20,036 frames, 24,585 s keyed in 14 days |
| Frames per day | **1,431** (ACTIVE 99 % of the time) | SIL |
| Keyed time | 1,756 s/day (2.0 %) | SIL |
| Broadcast volume | 1,431 × 110 B = **157 kB/day** | |

The beacon is broadcast: every station under the footprint receives the same frames, so the broadcast volume is not a
capacity budget. What the mission sees is set by the receive network (§3).

### 2.2 AI text

At most one Jetson text per request, carried in the beacon's text field (BEACON_SPEC §5); in the simulation 3,342 of
20,036 beacons (17 %) carried AI text. It adds nothing to the volume: the text field is there either way.

## 3. Telemetry Received

| Receiver | Passes/day ≥ 5° | Contact | Beacons in view/day | Frames decoded | Beacons/day received | Information/day |
|----------|-----------------|---------|---------------------|----------------|----------------------|-----------------|
| One telecommand station (Santa Clara, class A) | 3.85 | 27.7 min | 27.7 | 95.6 % (6°/s, 60 s) | **26.5** | 2.9 kB |
| Two telecommand stations (Santa Clara + a second site ≥ 1,500 km away) | 7.7 | 55 min | 55 | ≈ 96 % | **≈ 53** | 5.8 kB |
| SatNOGS: 253 2 m stations active in the last 7 days, 53 directional (frame success 0.6 assumed) and 200 omni (0.31, SQSO-LBA-001 §5.3) | | Visible to ≥ 1 station 52 % of the time | 745 | 35 % of all beacons decoded by at least one station if every station observes | **≤ 494** (upper bound; SatNOGS observations must be scheduled) | ≤ 54 kB |

Network figures: `software/simulation/cdr/rf/pass_geometry_mc.py` functions against the station list of
`network.satnogs.org/api/stations` (retrieved 2026-10-27), 3 days, 10 s steps.

Each beacon carries the whole health picture (mode, SOC, battery voltage, sunlit, RF state). Received beacons at one
station every ≈ 12 h (the longest pass gap, 11.4 h) are enough to follow the spacecraft's state; SatNOGS fills the
gaps.

## 4. Stored Housekeeping

### 4.1 What the baseline asks and what exists

| Item | State |
|------|-------|
| Requirement | Housekeeping stored every 60 s; ≥ 7 days onboard storage (BASELINE §8) |
| Record | `tlm_generate_housekeeping()`: 14-byte header + 26-byte payload + 4-byte CRC = **44 bytes** (`software/rust/sqso-fsw/src/telemetry.rs`) |
| Storage | **None.** The executive does not store housekeeping records (`software/rust/sqso-ffi/src/executive.rs`) |
| Retrieval | **None.** The command dictionary (SQSO-ICD-013 §7) has no "dump stored telemetry" command; `PLD_REQUEST_DATA` (0x43) has no handler, and command responses are not downlinked (SQSO-ICD-005 v2.0 §5.5) |

### 4.2 Sizing if it is implemented

| Quantity | Value |
|----------|-------|
| Generated | 1,440 records/day × 44 B = **63.4 kB/day** |
| 7 days | **444 kB** |
| FRAM areas (`hardware/obc/STORAGE_DESIGN.md` §3.3: telemetry buffer 64 KB + extended 128 KB) | 192 KB = **3.0 days** at 44 B; 7 days would need records ≤ 19 B |
| STM32L4R5 internal flash | 2 MB, of which 116 kB is code (`software/flight/docs/MEMORY_POLICY.md`): a 512 KB ring holds 8.3 days; 46 MB written over 2 years is ≈ 90 erase cycles a sector (rated 10,000) |
| Dump rate | 200-byte frames at 1.8 s each (1.5 s data + 0.3 s TXDELAY), within the 25 % duty limit (`DOWNLINK_DUTY_PERCENT`): 108 s keyed in a 7.2 min pass → 60 frames → **≈ 11.5 kB per pass** after the beacons |
| Dump capacity, one station | 3.85 passes × 11.5 kB ≈ **44 kB/day = 70 % of what is generated** |
| Dump capacity, two stations | ≈ 88 kB/day = 139 % (margin +39 %) |
| Power | 0.5 W RF ≈ 1.5 W DC for 108 s a pass: 0.045 Wh a pass |
| Conflict | Dump frames and the transponder share the transmitter; a dump displaces about 108 s of transponder time a pass |

### 4.3 Recommendation (for the chair: BASELINE §8 and flight software)

Either (a) amend BASELINE §8 to "housekeeping summarized every 5 minutes, ≥ 7 days onboard (88 kB); 60 s records
for the 6 hours around any fault or mode change", which one station can dump in under a day; or (b) keep 60 s and
allocate a 512 KB internal-flash ring, with two telecommand stations dumping. In both cases add a `DUMP_TLM` telecommand
(start record, count), plain-text or published binary frames (97.113(a)(4)), within the downlink duty limit.

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
| Beacon to a station | Next pass: ≤ 11.4 h at one station, ≤ 9.8 h at two |
| Station to the mission database | After each pass (≤ 1 h); SatNOGS polled hourly |
| Command to effect | < 1 s onboard; confirmed in the next beacon (≤ 60 s) |

## 7. Verification

| Item | Method | When |
|------|--------|------|
| Beacon cadence and size | SIL (done: 1,431 frames/day, 1.22 s average); EM on the target | EM |
| Received beacons per day | Commissioning: count frames in the database by source | Commissioning |
| Stored housekeeping and dump | Not implemented: test after the change request | After the change |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR review, communications panel | Rewritten (TB-12): broadcast, received and stored budgets separated; SIL frame counts; SatNOGS network estimate from the live station list; stored housekeeping found not implemented and without a downlink path, and its sizing against FRAM, internal flash and dump capacity; uplink and latency |
