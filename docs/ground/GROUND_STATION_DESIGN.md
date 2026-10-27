# Ground Station Design
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-GND-001
**Revision:** E
**Date:** 2026-10-27
**Status:** CDR design. **No telecommand station is agreed** (BASELINE §1, decision D1); the sites below are candidates
for analysis, not partners. Link values: SQSO-LBA-001 v3.2. Interface: SQSO-ICD-005 v2.0. Telecommand: SQSO-ICD-013.

---

## 1. Overview

The ground segment has four parts:

1. **Designated telecommand stations** (≥ 2, geographically separated; 97.211(a), RR 25.11): receive the beacon,
   send authenticated telecommand, and can stop every emission on the next pass (97.207(b)).
2. **Receive network**: SatNOGS and other amateur stations, receive only.
3. **Mission database and public dashboard**: ingest, decode, archive, alert, publish.
4. **Operations team**: the licensee (KK6GDB), control operators, key custodians.

Revision B had only a receive chain (no 70 cm transmitter, no keys, no telecommand station at all) and a superseded
link budget. Revision C replaces it.

## 2. Requirements

| ID | Requirement | Source |
|----|-------------|--------|
| GND-R1 | At least two telecommand stations, designated in writing by the licensee before the IARU request, ≥ 1,000 km apart (independent weather, fire, seismic and grid risk), each able to command alone | BASELINE §1, D1; MRD-095; RR 25.11 |
| GND-R2 | Each telecommand station: G/T and EIRP of §3.1 (beacon margin ≥ 6 dB at 5°, telecommand ≥ 6 dB at 10° per SQSO-LBA-001 LB-1, LB-3): **≥ 100 W PEP at the 70 cm antenna feed into ≥ 14 dBic (EIRP ≥ 2.5 kW), and commands sent only above 10° elevation** (chair decision C-14) | SQSO-LBA-001 v3.3 §6.1 |
| GND-R3 | 70 cm power within 47 CFR 97.313 and 2.106 US270: **50 W PEP within 240 km of Beale AFB, within 160 km of Clear SFS** and in the other listed areas, unless authorized case by case (FCC field office with the military area frequency coordinator), or under 97.313(f) (≤ 611 W ERP, 1 kW EIRP, with the antenna's lower half-power point above 10°). At such a site GND-R2's EIRP comes from the authorization or from 50 W PEP into a ≈ 17.5 dBic antenna with a ≤ 0.5 dB feed (§3.1) | 47 CFR (eCFR 2026-10-01) |
| GND-R4 | RF exposure evaluation per 47 CFR 1.1310 for each telecommand antenna at its maximum power (§3.1a: 13.3 m in the main beam for the general population at 2.5 kW EIRP and continuous carrier) | 47 CFR 1.1307(b), 97.13(c) |
| GND-R5 | Doppler pre-correction of the uplink to ±1 kHz; tracking of both antennas to ±5° | SQSO-LBA-001 §4.4 |
| GND-R6 | Operational key held offline at rest; recovery key with a second custodian at a different site; no key on a networked machine | SQSO-ICD-013 §9 |
| GND-R7 | Counters per SQSO-ICD-013 §5.2; with two stations, a shared sequence or UTC-derived counters (§5.2.1) | SQSO-ICD-013 |
| GND-R8 | An authenticated contact at least once a day in routine operations, and an alarm if no contact has been confirmed for 5 days (the inhibit is at 14) | §5 |
| GND-R9 | A `TX_DISABLE` sent within one pass of an FCC order; 24-hour contact for the licensee | 97.207(b) |
| GND-R10 | Every decoded frame archived with time, station and frequency; data public | 97.113; IARU form 5.5 |
| GND-R11 | Quiet 2 m site: system temperature ≤ 500 K at 5–20° elevation in the commanding azimuths | SQSO-LBA-001 §4.2 |

## 3. Telecommand Station

### 3.1 Design

| Item | Specification | Example (not selected) | ROM |
|------|---------------|------------------------|-----|
| 2 m antenna | Cross-Yagi, ≥ 12 dBic, RHCP/LHCP switchable | M2 2MCP14 class | $400 |
| 70 cm antenna | Cross-Yagi, ≥ 14 dBic | M2 436CP30 class (≈ 15.5 dBic) | $400 |
| Rotator | Az-el, ±5°, computer control | Yaesu G-5500DC with a USB interface | $900 |
| 2 m preamplifier | Mast-mounted, NF ≤ 0.8 dB, sequenced (full duplex: no TX on 2 m, so no sequencer needed for the beacon; needed only if the 2 m antenna is ever used to transmit) | SSB SP-2000 class | $350 |
| Transceiver | Satellite (full-duplex) mode, CAT Doppler control, 2 m receive, 70 cm exciter set to the amplifier's drive (≈ 20–25 W) | Icom IC-9700 class | $1,900 |
| 70 cm power amplifier (C-14) | Solid-state **class AB linear**, ≥ 150 W FM output at 430–440 MHz, continuous carrier ≥ 10 min, hardware-keyed (RF-sense keying disabled), reflected-power trip; set for **100 W at the antenna feed** (126 W at its output with the 1.0 dB feed) | 70 cm 150–200 W solid-state amplifier class | $1,200 |
| Sequencer and PTT | Hardware sequencer: PTT in → amplifier to transmit → 30 ms → exciter keyed; release in reverse order; 60 s key-down timer; keying refused below 10° elevation (16° in the degraded mode) from the rotator controller's elevation | DEMI/LunaLink sequencer class | $150 |
| Amplifier supply | 13.8 V, ≥ 40 A, with the UPS covering the receive chain only | | $250 |
| TNC | Software: Dire Wolf 1.7 (KISS over TCP), AFSK 1200, ±3.0 kHz deviation set at the transceiver (measured, VI-GND-01) | Dire Wolf | — |
| Tracking and Doppler | Gpredict or equivalent with Hamlib | | — |
| Telecommand software | `software/ground/tools/telecommand_builder.py` (frames byte-identical to the flight vectors) | | — |
| Key handling | Operational key on an offline laptop or a hardware token, mode 600, never on the station computer; frames built offline and carried over (or the builder runs on the offline machine with the station computer as a dumb KISS relay) | | $300 |
| Time | GNSS-disciplined clock or NTP checked against GNSS, ±1 s (UTC-derived counters, telecommand timing) | | $150 |
| Coax | 70 cm: LMR-600 class, ≤ 20 m, **≤ 1.0 dB** at 435 MHz with connectors and the sequencer relay; 2 m: LMR-400 class | | $450 |
| Power | UPS ≥ 30 min | | $200 |
| **Total per station** | | | **≈ $6,650** (≈ $4,900 before C-14) |

**Budget at these values (SQSO-LBA-001 v3.3, 475 km, receiver as designed, worst case):** beacon +7.9 dB at 5° (median
gain); telecommand at 100 W PEP at the feed (C-14), 10th-percentile spacecraft gain and +6 dB interference: **+6.5 dB at
5°, +8.3 dB at 10°**. Degraded mode (amplifier out of service, or a station at 50 W PEP with the 14 dBic antenna): +3.8 dB
at 10°, 6 dB from about 16°, so commands are then sent only above 16°.

### 3.1a Transmit chain, sequencing and RF exposure (decision C-14)

```
station computer ─ KISS ─ transceiver (70 cm exciter, ≈ 20–25 W) ─┐
rotator controller (elevation) ─ PTT gate (≥ 10°) ─ sequencer ─────┼─ amplifier (class AB, ≥ 150 W FM) ─ coupler/wattmeter
                                                                   └──────────────────────────────── LMR-600 ≤ 1.0 dB ─ arrestor ─ 14 dBic cross-Yagi
2 m cross-Yagi ─ 70 cm rejection filter (≥ 40 dB at 435 MHz) ─ mast preamplifier ─ LMR-400 ─ transceiver 2 m receive
```

- **Sequencing.** Hardware PTT only: the sequencer puts the amplifier into transmit, then keys the exciter 30 ms later,
  and on release unkeys the exciter before returning the amplifier to bypass, so no relay switches RF. The PTT gate
  refuses to key when the rotator reports less than 10° elevation (16° in the degraded mode). A 60 s key-down timer
  backs the software (a telecommand frame is ≤ 1 s; four attempts a pass). The amplifier trips on reflected power
  above 10 % (VSWR 1.9).
- **Full duplex.** The 2 m receive chain stays on while 100 W at 435 MHz radiates from the adjacent antenna: a 70 cm
  rejection filter (≥ 40 dB) ahead of the 2 m mast preamplifier keeps it out of compression; the antennas are separated
  on the boom or mast as the rotator allows (checked with VI-GND-01 by transmitting into the antenna while measuring the
  2 m noise floor).
- **RF exposure** (47 CFR 1.1307(b), 1.1310 Table 1, 97.13(c)(1); OET Bulletin 65 far-field method with the 2.56 ground
  reflection factor). EIRP 2.5 kW (100 W at the feed × 14 dBic) at 436 MHz. Limits: general population f/1500 =
  0.29 mW/cm² (30-minute average), controlled f/300 = 1.45 mW/cm² (6-minute average).

  | Exposure | Duty assumed | Distance in the main beam to meet the limit |
  |----------|--------------|---------------------------------------------|
  | General population | 100 % (continuous carrier, bounding) | **13.3 m** |
  | General population | 10 % (≤ 3 minutes keyed in 30 minutes; a pass of commands keys ≤ 1 minute) | 4.2 m |
  | Controlled (operator, aware) | 100 % | 5.9 m |
  | Controlled | 17 % (≤ 1 minute keyed in 6 minutes) | 2.4 m |
  | Degraded mode, 50 W PEP (0.89 kW EIRP), general population, 100 % | | 7.9 m |

  The beam points at least 10° above the horizon, so with the antenna on a mast at least 3 m above any accessible area,
  ground-level areas are below the main beam. Site rule: no occupied area (roof, balcony, neighbouring window) within
  13.3 m of the antenna in any direction it can point above 10°, or the duty limited and documented; the evaluation is
  recorded for each site before its designation (VI-GND-05).

### 3.2 Site criteria

| Criterion | Value |
|-----------|-------|
| Horizon | ≤ 5° over the azimuths of the passes (north–south arcs for the SSO) |
| 2 m noise | System temperature ≤ 500 K at 5–20° (GND-R11); P.372 residential man-made noise would take a 5° margin from +7.6 to −3.1 dB |
| Power, network | Mains with UPS; internet for the database upload |
| Security | Locked room for the key medium; the key never leaves the custodian's control |
| Licensed operator | A control operator with 70 cm privileges (any U.S. class) for each pass |

### 3.3 Candidate sites (analysis only; nothing agreed)

| Site | Status | Distance from Santa Clara | Passes/day ≥ 5° | Minutes/day | Commanding passes/day (≥ 15°) | Notes |
|------|--------|---------------------------|-----------------|-------------|-------------------------------|-------|
| Santa Clara, CA (licensee area; Santa Clara University prospective) | Prospective | — | 3.77 | 26.1 | 2.4 | Within 240 km of Beale AFB (≈ 204 km): 50 W PEP (US270); C-14 by authorization or 50 W PEP into ≈ 17.5 dBic (GND-R3) |
| Blacksburg, VA (example of an East Coast university site) | Example | 3,640 km | 3.78 | 26.0 | 2.4 | Passes 3 h earlier in UTC; union with Santa Clara 51.9 min/day, longest gap 9.7 h |
| Fairbanks, AK | Example | 3,500 km | 8.7 | 56.3 | 4.9 | High-latitude: most passes a day; auroral scintillation at VHF/UHF (SQSO-LBA-001 §2.3); union 81.6 min/day, gap 9.6 h; within 160 km of Clear SFS (≈ 90 km): 50 W PEP (US270), as for Santa Clara |
| Honolulu, HI | Example | 3,880 km | 3.18 | 22.0 | 2.1 | Independent of every mainland hazard; union gap 9.8 h |
| Boulder, CO | Example | 1,470 km | 3.95 | 27.3 | | 51 % of its visibility overlaps Santa Clara's; gap not reduced (11.3 h) |
| San Diego, CA | Example | 660 km | 3.55 | 24.7 | | Too close: 80 % overlap; same seismic region; south of 37°10′ N, so 50 W PEP (US270) |

**Recommendation:** the second station at least 1,500 km from the Bay Area and outside California (East Coast or
Midwest university club, or Alaska). NASA Ames, the other prospective partner, is 13 km from Santa Clara and would not
meet GND-R1 as the second station. A third, receive-and-command-capable backup kit (antennas and radio of §3.1 held in
storage, keys not loaded) is cheap insurance against the common-mode outages of §5.

## 4. Coverage

`software/simulation/cdr/rf/pass_geometry_mc.py`, 60 days from 2028-11-01, 475 km, 97.4°, LTAN 22:30 (BASELINE Rev E §3):

| Santa Clara | ≥ 0° | ≥ 5° | ≥ 10° | ≥ 20° | ≥ 25° |
|-------------|------|------|-------|-------|-------|
| Passes per day | 4.82 | 3.77 | 2.98 | 1.98 | 1.68 |
| Mean duration | 8.7 min | 6.9 min | 5.6 min | 3.7 min | 3.0 min |
| Minutes per day | 41.9 | 26.1 | 16.6 | 7.3 | 5.1 |
| Median maximum elevation | 15.2° | 21.5° | 27.9° | 41.1° | 46.1° |

Passes come in two groups a day, near 10:30 and 22:30 local time (the descending and ascending nodes), so the longest
gap at one mid-latitude station is 11.4–12.9 h (Blacksburg, Santa Clara); two stations three time zones apart cut it to
9.7 h. The orbit gives every
point on the Earth at least two passes a day above 5° (proposed MRD-012).

## 5. Contact Cadence and the Loss-of-Contact Timers

`software/simulation/cdr/rf/contact_cadence.py` (2,000 two-year missions). The inhibit trips after N days with no
authenticated contact (SQSO-ICD-013 §6.2: N = 30 days at launch, 14 days after commissioning). Model: commanding passes
above 15°, four attempts a pass at 90 % each; each station unavailable on 10 % of days at random and suffering one
outage a year of 3–21 days; common-mode outages of both stations (keys or counters lost, licensee unavailable) 0.25 a
year lasting 1–30 days.

| Stations | Routine contact | P(14-day inhibit trips at least once in 2 years), independent outages only | Same, with common-mode outages | P(30-day trips) | Longest wait for a commanding pass (TX-OFF after an FCC order) |
|----------|-----------------|------|------|------|------|
| Santa Clara only | daily | 56 % | 67 % | 1.2–1.8 % | 13.0 h |
| Santa Clara + Blacksburg | daily | **0.6 %** | 25 % | 0–1.0 % | 9.9 h |
| Santa Clara + Blacksburg | every 7 days | 2.5 % | 29 % | 0–6.2 % | 9.9 h |
| Santa Clara + Fairbanks | daily | 0.5 % | 25 % | 0 % | 11.3 h |
| Santa Clara + Honolulu | daily | 0.8 % | 26 % | 0–0.4 % | 11.5 h |

With the commanding rule of C-14 (a pass counts when it reaches 10°: 2.97 commanding passes a day at Santa Clara
instead of 2.43) the results do not change beyond the Monte Carlo scatter: Santa Clara alone 57 % / 67 %, Santa Clara
and Blacksburg 0.65 % / 26 % (`contact_cadence.py --min-el 10`); the inhibit risk is set by ground availability.

Findings:

1. **One station cannot hold N = 14 days**: a single 3–21-day equipment outage trips the inhibit in more than half of
   missions. Two independent stations bring it below 1 %. This is the quantitative case for GND-R1.
2. **Common-mode ground failure dominates with two stations** (25 %): the shared counter file, a single operational
   key holder, one licensee. Mitigations: UTC-derived counters (SQSO-ICD-013 §5.2.1), a copy of the operational key
   with each station's custodian, a backup control operator for the licensee, the spare kit of §3.3. A trip is not a
   loss: the receiver stays on and the next authenticated command lifts the inhibit, but the transponder and beacon
   are silent until then.
3. **The 30-day launch value is safe for first contact**: object identification after a Transporter deployment
   (dozens of objects, TLEs after 1–3 days) leaves more than 2 weeks of margin.
4. **Routine contact daily, alarm at 5 days** (GND-R8). The cadence is cheap; a weekly cadence adds risk.
5. The analysis assumes a contact can be confirmed from the ground. Today it cannot: no command response reaches the
   downlink and the beacon does not carry the contact count (SQSO-ICD-005 v2.0 §5.5; finding GND-03).

## 6. Receive Network

### 6.1 SatNOGS

SatNOGS network, 2026-10-27 (`network.satnogs.org/api/stations`): 504 stations seen in the last day; **253 with a 2 m
antenna seen in the last 7 days** (134 Europe and Africa, 68 North America, 37 Asia and Oceania, 7 South America);
80 % of them omni (turnstile, QFH, eggbeater, vertical, discone, dipole). For an omni station (0 dBic, 1,000 K) the 0.5 W
beacon has −7.4 dB margin at 5° and +5.5 dB at zenith (SQSO-LBA-001 §4.2): **31 % of frames and 74 % of passes** are
decoded in the pass simulation (§5.3 there). SatNOGS adds many passes a day worldwide, but weakly; it cannot replace
the telecommand stations for telemetry.

| Action | When |
|--------|------|
| SatNOGS DB transmitter entry (downlink frequency, AFSK 1200, AX.25) once the IARU letter arrives | 2027-05 |
| Decoder for the ASCII beacon (Kaitai struct or the SatNOGS AX.25 text decoder) from BEACON_SPEC §3 | Before launch |
| Ask the SatNOGS community for priority scheduling during LEOP (first 14 days) | 2028-05 |

### 6.2 Other receivers

Amateur reception reports (decoded frames pasted or uploaded) through the dashboard. The transponder's users give the
best indication of its health.

## 7. Mission Data Flow

```
telecommand stations --(KISS frames, JSON, after each pass)--+
SatNOGS (db.satnogs.org telemetry API, polled hourly) ------+--> ingest --> database --> decoder --> dashboard (public)
amateur reports (web form) ---------------------------------+                  |               --> alerts (ops)
                                                                               +--> archive (raw frames, permanent)
orbit: TLE (Space-Track/CelesTrak) and GNSS fixes (D6) --> orbit determination --> CCSDS OEM upload (SpaceX §9.5.7.2)
```

| Element | Design |
|---------|--------|
| Ingest | Raw AX.25 frame (hex), UTC time, station, Doppler-corrected frequency, source. Duplicates (the same frame from several stations) kept as one frame with several receptions |
| Database | PostgreSQL: frames, receptions, decoded telemetry (BEACON_SPEC §4 fields with "not measured" kept as null), commands sent (counter, key ID, result as seen) |
| Dashboard | Last beacon, telemetry plots, pass predictions (published), beacon text (AI or template) |
| Alerts | No beacon heard by any station for 6 h after commissioning; mode SAFE or EMERGENCY; no confirmed contact for 5 days (GND-R8) |
| Archive | Raw frames permanent; SatNOGS audio is held by SatNOGS |
| Orbit | GNSS-derived ephemeris pipeline is a D6 action (BASELINE §6); TLEs as contingency |

Data volumes are in `docs/environment/DATA_BUDGET.md` (SQSO-DB-001 v2.0).

## 8. Operations Roles

| Role | Who | Notes |
|------|-----|-------|
| Licensee | KK6GDB | Signs filings; designates stations; 24-hour contact (GND-R9) |
| Control operators | ≥ 2 per station, licensed | Named before the IARU request |
| Operational-key custodians | One per telecommand station | GND-R6 |
| Recovery-key custodian | A person other than the operational custodians, at another site | SQSO-ICD-013 §9 |
| Database and dashboard maintainer | One person | |

Procedures: `docs/ground/GROUND_STATION_OPERATIONS_MANUAL.md` (SQSO-GSOM-001 v1.1).

## 9. Verification

| ID | Test | Pass criterion | When |
|----|------|----------------|------|
| VI-GND-01 | Station transmit chain into a dummy load: power, deviation, Doppler control, sequencing | 100 W ± 0.5 dB at the antenna feed (≤ 50 W PEP at the transmitter at a US270 site without an authorization); ±3.0 ± 0.3 kHz deviation; frequency within ±200 Hz of command; exciter keyed ≥ 30 ms after the amplifier, no keying below 10° elevation; 2 m noise floor rise ≤ 0.5 dB while transmitting | Each station, before LEOP |
| VI-GND-02 | Site noise survey: 2 m noise at 5–20° elevation in 8 azimuths | T_sys ≤ 500 K (GND-R11) | Before a site is designated |
| VI-GND-03 | End-to-end with the EM over an attenuated cable link with Doppler applied | Beacons decoded, commands (including `TX_DISABLE`) accepted at the SQSO-LBA-001 levels | EM |
| VI-GND-04 | Two-station counter exercise (shared or UTC-derived): 50 commands alternating stations against the EM | All accepted; no counter reused | EM |
| VI-GND-05 | RF exposure evaluation per 47 CFR 1.1310 at the station's maximum EIRP (§3.1a) | Documented distances met: no occupied area within 13.3 m in the main beam at 2.5 kW EIRP, or a documented duty limit | Each station |
| VI-GND-06 | Pass rehearsal (LEOP): 5 simulated passes with the SIL or the EM | Commands sent and verified within the pass timeline of SQSO-ICD-005 §7.1 | Before launch |

## 10. Open Items

| Item | Closure |
|------|---------|
| No telecommand station agreed (GND-R1) | Licensee: written agreements by 2027-01-05 (REGULATORY_ROADMAP F13) |
| Command response and contact count on the downlink (finding GND-03) | Flight-software change request |
| UTC-derived counters in the ground tool | Ground-software change |
| Key custody plan | Licensee, with the agreements |
| SatNOGS decoder | Before launch |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| A.1 | 2026-10-16 | §3.3 and §6 point to the re-issued link budget (TB-05) | Funding audit |
| B | 2026-10-16 | §4.2 SatNOGS frequency: the IARU-coordinated frequency (SYS-200) | SMART-QSO Team |
| C | 2026-10-27 | Rewritten at the CDR: telecommand station design with the 70 cm transmit chain, keys, time and power limits (US270, 97.313(f)); site criteria and candidate sites with coverage; contact-cadence analysis against the 14- and 30-day timers; SatNOGS network as measured; mission data flow; roles; verification items VI-GND-01 to -06. The superseded link budget and the receive-only design are removed | CDR review, communications panel |
| D | 2026-10-27 | Coverage (§3.3, §4) and contact cadence (§5) re-run at the Rev E baseline orbit, 475 km from 2028-11-01 (was 500 km from 2028-06-01): Santa Clara 3.77 passes and 26.1 min a day (was 3.85, 27.7); one station trips the 14-day inhibit in 56–67 % of missions, two independent stations in 0.6 % (was 0.8 %); longest wait for TX-OFF 13.0 h with one station, 9.9 h with two (RID SE-16) | CDR S&MA analysis |
| E | 2026-10-27 | Decision C-14: telecommand at ≥ 100 W PEP at the antenna feed and commands only above 10° (GND-R2); station transmit chain with a class AB ≥ 150 W amplifier, sequencer and PTT gate, 1.0 dB feed, 70 cm rejection on the 2 m preamplifier (§3.1, §3.1a); US270 areas (Beale for the Bay Area, Clear SFS for Fairbanks) and the 50 W / 17.5 dBic alternative (GND-R3); RF exposure evaluation at 2.5 kW EIRP (GND-R4, §3.1a); station cost ≈ $6,650; cadence at the 10° rule (§5) | CDR S&MA analysis |
