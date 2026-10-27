# Interface Control Document: Space-to-Ground

**Document ID**: SQSO-ICD-005
**Version**: 2.0
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
         downlink 145.800-146.000 MHz      uplink 435.000-437.300 MHz
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
| Frequency | One IARU-coordinated frequency in 435.000–437.300 MHz; planning value 435.5 MHz | Clear of 3 × f_down (SQSO-LBA-001 §7.1) |
| Receiver | Always on after the RF silence; ±7.5 kHz IF; T_sys 1,000 K budgeted | RF-P006, RF-P010 |
| Telecommand modulation | AFSK Bell 202 on FM, ±3.0 kHz; AX.25 UI to KK6GDB-11; 10K4F2D | SQSO-ICD-013 §3 |
| Transponder access | FM voice with CTCSS 67.0 Hz at ±0.5–0.75 kHz | RF-P012 |
| Doppler correction | **Telecommand: pre-correct to ±1 kHz** (uncorrected 10 kHz does not decode). Users: 5 kHz steps across the pass | SQSO-LBA-001 §3.2, §4.4 |
| Polarization | Any; the spacecraft element is linear | |

### 4.3 Link summary (SQSO-LBA-001 v3.0; analysis)

| Link | Case | Margin or SINAD |
|------|------|-----------------|
| Beacon, 0.5 W, reference station (12 dBic, 500 K) | 5°, median / 10th-percentile spacecraft gain | +7.6 dB / +0.6 dB |
| Telecommand, 25 W, 14 dBic | 10°, 10th-percentile gain, +6 dB interference | +2.3 dB (+5.3 dB at 50 W) |
| Transponder, handheld Yagis both ends | 5° / 20° | 19 dB / 26 dB SINAD |
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
| ACTIVE | 60 s (recommended ±5 s jitter, SQSO-LBA-001 §5.3) | Filtered AI text once per request, otherwise a template |
| IDLE | 120 s | Template |
| SAFE | 180 s | Template |
| EMERGENCY | None (beacon shed) | |

Values not measured within 60 s are sent as `-`. Typical frame 110 bytes of information field, about 1.2 s on the air.

### 5.4 Telecommand

Defined in SQSO-ICD-013, which governs: frame `0xAA 0x55 | cmd_id | len | counter (4, BE) | args | tag (16) | sum16 (2,
BE)` in the information field of a UI frame to KK6GDB-11; HMAC-SHA-256 truncated to 16 bytes; strictly increasing
counters per key; command dictionary in SQSO-ICD-013 §7. `TX_DISABLE` (0x32) ceases all emissions in every mode.

### 5.5 Command response

**There is no command response on the downlink.** The executive calls `uplink_poll(NULL)`
(`software/rust/sqso-ffi/src/executive.rs`), so the `CmdResponse_t` that `cmd_process()` fills, including the 4 bytes
of `GET_STATUS` and the EPS and ADCS telemetry of `EPS_GET_TELEMETRY` and `ADCS_GET_ATTITUDE` (SQSO-ICD-013 §7), is
discarded. A station sees a command's effect only through the beacon's fields (mode, RF), and has no way to confirm
that a contact was counted by the loss-of-contact monitor. The Version 1.x response format is removed.

Proposed for the flight software (for the chair; finding GND-03): after each authenticated command, one UI frame
`ack KK6GDB-11: C=<counter> R=<result> N=<days since contact>` (plain ASCII, 97.113(a)(4)), within the downlink duty
limits; and the loss-of-contact count in every beacon. Without it, the contact cadence of SQSO-GND-001 §5 cannot be
confirmed from the ground.

## 6. Ground Station Obligations

| ID | Obligation | Applies to |
|----|------------|------------|
| GS-1 | Designated in writing by the licensee (97.211(a)); listed in the IARU request and the pre-space notification | Telecommand stations |
| GS-2 | Holds the operational key under SQSO-ICD-013 §9 (offline at rest; never on a networked machine); the recovery key with a separate custodian | Telecommand stations |
| GS-3 | Counters per SQSO-ICD-013 §5.2 (shared sequence or UTC-derived, §5.2.1) | Telecommand stations |
| GS-4 | Doppler pre-correction to ±1 kHz; tracking antennas | Telecommand stations |
| GS-5 | Power within 47 CFR 97.313 and 2.106 US270 (50 W PEP near Beale AFB and other listed sites unless 97.313(f)); RF exposure per 1.1310 | Telecommand stations |
| GS-6 | Able to send `TX_DISABLE` on the next pass after an FCC order (97.207(b)); 24-hour contact number | Telecommand stations |
| GS-7 | Forward decoded frames with time, station and Doppler-corrected frequency to the mission database | All receive stations |
| GS-8 | Transponder users identify per 97.119 and use CTCSS 67.0 Hz | Users |

## 7. Operational Interface

### 7.1 Telecommand pass

| Time | Action |
|------|--------|
| AOS − 10 min | Current orbit elements; clock checked against GNSS (±1 s); Doppler table loaded |
| AOS | Receive; wait for a beacon (it confirms the spacecraft is not silent and gives the frequency) |
| Above 15° | Send the planned commands, each with a new counter; up to four attempts a command, checking the effect in the next beacon or status |
| LOS | Log counters used, results, beacons heard; upload to the mission database |

### 7.2 Pass statistics (SQSO-GND-001 §4)

Santa Clara, 500 km SSO: 3.85 passes a day above 5°, 7.2 min average, 28 minutes a day; 2.5 a day above 15°. Longest
gap between passes 11.4 h.

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
