# D6 Research Report: GNSS Receiver for SMART-QSO (SpaceX UG v11 §9.5.7 compliance)

> Research input to decision D6, prepared 2026-10-15 and imported unchanged into the repository. Where it mentions working files (raw downloads, extracted texts, scripts) or says no repository files were modified, it refers to the research session; those files were not kept. How the decision was taken and applied is recorded in `docs/BASELINE_PARAMETERS.md` section 9.


Prepared 2026-10-15. All web sources were accessed 2026-10-15 unless a document date is given.
No repository files were modified. Extracted source texts are in `./src/` next to this file (untrusted data, not code).

Evidence tags (same convention as the repo's launch trade study):
**[P]** primary source read in full or in the relevant section this session (datasheet, regulation, user's guide);
**[U]** unverified or secondary (distributor listing, search snippet, vendor web page not backed by a datasheet);
**[A]** analyst estimate or judgment, derived by me, not sourced.

---

## 1. Executive Summary

**The requirement is a launch-provider contract term, not an FCC rule, and it is not strictly "GNSS or nothing".**
SpaceX Rideshare Payload User's Guide **Version 11, August 2026**, §9.5.7.2 requires the customer to "publish forward predicted
GPS/GNSS-based satellite ephemerides with covariance" to Space-Track and to SpaceX Space Traffic Coordination, first within
24 h of first contact and then at least every 24 h until the 18th SDS has cataloged the object. A customer "not capable of
publishing ephemerides due to lack of GPS or predictive capability must coordinate with a third party ... prior to
integration". The integrator enforces it, and failure "may result in mandatory re-booking to a later launch" **[P]**.
So D6 is a genuine two-way choice, but the GNSS route is the path the guide and SpaceX's own safety docs assume.

**No FCC rule that applies to SMART-QSO today requires GNSS or owner/operator ephemeris.** The August 2024 FCC order
(89 FR, doc. 2024-17093) only added the 5-year LEO disposal definition to 47 CFR 5.64, 25.114(d)(14)(vii)(D)(1), 25.283(e) and
97.207(g)(1)(vii)(D)(1); it does not mention ephemeris **[P]**. §97.207(g)(1)(v) requires a trackability statement, 18th SDS
registration statement and a statement of the extent of ephemeris sharing, not a capability. The new mandatory ephemeris rule
(§100.201(c), FCC 26-47, adopted 2026-07-22) sits in the new Part 100, and §100.5(f) leaves Amateur Satellite Service stations
under Part 97, so it does not reach SMART-QSO today **[P]**. Watch the FNPRM (Part 5 experimental consolidation and ephemeris
sharing are open questions).

**Recommendation: choose "add a GNSS receiver" for D6.** Ranked shortlist:

1. **SkyFox Labs piNAV-NG/FM** (datasheet Rev. A/2025): 24 g, 125 mW typical, UART/NMEA, L1 C/A, LEO-rated cold start 80 s.
   Best fit for the power budget, STM32 interface and 1U volume. Main frictions: Czech dual-use export licence, no stated TID,
   antenna accommodation.
2. **Spacemanic Celeste + ZENITH V3a antenna** (datasheets Feb 2026): 25 g + 20 g, 250 mW + 24 mW, multi-constellation, many
   buses. Best alternative; longer lead time (4-5 months receiver, 12 weeks antenna).
3. **Pumpkin GPSRM 1 (NovAtel OEM719)**: US-supplied, PC/104 footprint, dual-frequency option, but 1.2-1.8 W, 106 g, about
   $11.5k, radiation untested. Use only if a US-only supply chain or dual-frequency is wanted.

**Duty cycling is easy: energy is not the problem, attitude is.** At 3-6 fixes/day the orbit-average power is about 1-7 mW
(piNAV-NG / Celeste) or 4-24 mW (OEM719 class) **[A]**, versus a tumbling-case headroom of only 0.24 W before the 20 %
generation margin is lost. A flat **0.05 W** allocation line covers any shortlist option with retries. The real risks are
(a) getting the patch antenna within about 45 degrees of zenith for 100-120 s on a tumbling or sun-pointed 1U, (b) EMC with the
Jetson and the 2 m PA, (c) where a patch fits on a 1U that already uses the faces for cells and wings, and (d) building the
ground orbit-determination (OD) and OEM upload pipeline, which is half of the deliverable.

---

## 2. Technical Overview: What Is Actually Required

### 2.1 SpaceX Rideshare Payload User's Guide, Version 11, August 2026 [P]
Source: `https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf` (111 pages; cover "Version 11 -
August 2026"). §9.5.7 "Coordination with Space Situational Awareness Agencies", pp. 57-58, plus §1.2 and §3.3.

| Clause | Requirement (paraphrased; quotes are short) |
|--------|-----------------------------------------------|
| §1.2 | Customer registers all deployed objects with the 18th SDS and SpaceX Space Traffic Coordination "no later than L-2 months". All payloads are expected to meet the FCC 5-year disposal rule |
| §9.5.7.1 | Register with the 18th SDS via space-track.org (SPOC.SPACE.CustomerService@spaceforce.mil). Operating-team contact data must be provided and SpaceX requires opt-in to **public listing of that contact for the mission lifetime**. If any orbit crosses ISS altitude, also contact NASA (jsc-dl-topo-iwg@mail.nasa.gov). Recommends the NASA Conjunction Assessment best-practices handbook |
| **§9.5.7.2** | "Customer is required to publish forward predicted GPS/GNSS-based satellite ephemerides with covariance" to (1) Space-Track and (2) SpaceX Space Traffic Coordination (onboarding: space-safety-onboarding@spacex.com). Publish **within 24 h of first contact**; then at regular intervals (e.g., after every ground-station pass) and **at least every 24 h**, from initial signal acquisition after separation until **cataloging is complete** (permanent NORAD ID assigned, positional data on space-track.org, operator contact public) |
| §9.5.7.2 (fallback) | "Customers not capable of publishing ephemerides due to lack of GPS or predictive capability must coordinate with a third party to meet these requirements prior to integration to the Launch Vehicle. SpaceX can provide third party support recommendations." |
| §9.5.7.2 (enforcement) | "Integrators are responsible for enforcing these requirements with their constituents"; failure to demonstrate ability "may result in mandatory re-booking". Claim the object with the 18th SDS **no later than 30 days after deployment**, with public contact info |
| §9.5.7.3 | Maneuvering payloads: publish 24 h before a planned maneuver. Not applicable (no propulsion) |
| §9.5.7.5 (best practice) | Upload within **3 h of deployment**; publish **at least three times per day**. SpaceX also files its own post-launch ephemeris from Stage 2 state, but "customer-provided orbit determination ... (e.g., from GPS or GNSS) is more accurate" |
| §4.1.7.3 | Transmitters wait a minimum time after separation (Table 4-10; 1 W EIRP at 0.3-5 m/s separation: 19 s to 2 s). Does not restrict a receive-only GNSS |

Reading notes **[A]**:
- The hard requirement is **time-limited** (until cataloging completes, days to a few weeks). Beyond that the guide's text is a
  best practice, but conjunction response under §97.207(g)(1)(iv)(A)(5) and the repo's conjunction plan still benefit from
  accurate ephemeris for the whole mission.
- "GPS/GNSS-based" is stated as the content of the ephemeris. A TLE-propagated ephemeris is not what the text describes, which is
  why the third-party fallback exists. Whether a third party's radar-derived OD satisfies SpaceX is not defined in the guide;
  get acceptance in writing from SpaceX and the integrator.
- Ground-contact count is not specified. A beacon that carries the latest fix lets any SatNOGS-class station supply data (§4.3).

### 2.2 SpaceX / Starlink Space Safety upload format [P-web, undated]
`https://docs.space-safety.starlink.com/docs/tutorial-basics/trajectories/` and `.../best-practices/best_practices`:
- Accepts OEM/OCM (CCSDS **KVN only, XML not supported**), Modified ITC, UTC, NASA, JSpOC formats.
- Frames: ITRF preferred, EME2000 accepted. **All timestamps UTC; GPS time not supported.**
- Covariance **recommended, not required**; without it no collision probability is computed. RTN, ITRF or EME2000 covariance frames.
- At least 6 future state vectors; span at least 42 s and under 21 days; velocities at most 70 km/s. No cadence mandated.
- Best-practices page: "Equip satellites with GNSS receivers" to produce high-accuracy state estimates without relying on third-party observations; publish often, with every ground contact, with covariance **[U: via search summary]**.
- Space-Track operator upload file-naming rules (satellite name and NORAD ID in the `.oem` file name) appear in the Space-Track
  "Spaceflight Safety Handbook for Operators", which I did **not** retrieve. **To check.**

### 2.3 FCC and Part 97 / Part 5 expectations

| Rule | What it says about ephemeris, GNSS, position knowledge | Applies to SMART-QSO? |
|------|---------------------------------------------------------|-----------------------|
| 47 CFR 97.207(g)(1)(v) [P, Cornell LII text] | Trackability: LEO station presumed trackable if each space station "is 10 cm or larger"; (A) how it will be identified after deployment; (B) whether registered with the 18th Space Control Squadron or successor before deployment; (C) the extent to which the operator "plans to share information regarding initial deployment, ephemeris, and/or planned maneuvers" with 18th SCS, other SSA entities and other operators. **A statement, not a capability.** No mention of GNSS or GPS | Yes (Part 97 pre-space notification). A 1U (100 mm smallest dimension) sits exactly at the 10 cm threshold |
| 97.207(g)(1)(iv)(A)(5) [P] | Certify review of collision risk on conjunction warnings; suggested steps include "sharing ephemeris data" | Yes |
| 47 CFR 25.114(d)(14)(v)(B)(C) [P, search + eCFR summary] | Same items for Part 25 | No (Part 25) |
| 47 CFR 25.283(e) [P] | LEO uncontrolled re-entry as soon as practicable, **no later than 5 years** after end of mission. Nothing on ephemeris or position knowledge | No, but the 5-year rule is mirrored in 97.207(g)(1)(vii)(D)(1) |
| FCC Report and Order, 89 FR (2024-08-09), doc. 2024-17093 [P] | Amends 5.64(b)(7)(iv)(A), 25.114(d)(14)(vii)(D)(1), 25.283(b),(d),(e), 97.207(g)(1)(vii)(D)(1): 5-year disposal. Effective 2024-09-09. **Does not mention ephemeris, the 18th SCS/SDS or trackability** | Disposal only |
| FCC 26-47, Space Modernization for the 21st Century (adopted 2026-07-22, released 2026-07-23), **§100.201(c)** [P] | "Space station licensees and market access recipients must submit accurate and timely ephemeris data" to the 18th SDS (or successor) and to FCC-identified SSA systems. ¶287: initially *encourages* every 8 h for LEO, registering with an SSA provider before launch, submitting within 8 h of deployment; FCC declined to name Space-Track.org as mandatory. FNPRM ¶¶372-374 seeks comment on cadence and certified private ephemeris-tracking systems | **Not today.** §100.5(f): Amateur Satellite Service stations are licensed under Part 97; §100.5(b): Experimental Radio Service "may be subject to licensing under part 5". Effective date to be set by Space Bureau public notice (DA 26-1080, 2026-10-07 [U, not fetched]) |

**Premise correction:** the user brief suggested that 25.114(d)(14) and 25.283 in the 2024 FCC rule carry owner/operator ephemeris
expectations. They do not: ephemeris appears only as a disclosure item in 25.114(d)(14)(v)(C) / 97.207(g)(1)(v)(C), and the 2024
change was disposal-only. The 18th SDS is named "18th Space Control Squadron" in the legacy CFR text and "18th Space Defense
Squadron" in the 2026 FCC order and the SpaceX guide.

**ISS / NRCSD route:** the §9.5.7 obligation is a SpaceX contract term, so it does not apply to an ISS deployment. In the
sources I reviewed (NanoRacks lessons-learned briefing, search summaries) I found no GNSS requirement for NRCSD deployments,
only the recommendation to coordinate with the 18th SCS before deployment **[U]**. I did **not** retrieve the current
Voyager NRCSD IDD text, so verify it.

---

## 3. Implementation Considerations

### 3.1 COCOM limits and export classification
- **COCOM limit** (commercial GNSS receivers stop or invalidate output above ~18 km altitude and ~515 m/s / 1,000 kt). Some
  vendors implement it as AND, some as OR; an OR implementation stops a LEO receiver **[U: u-blox community, search summaries]**.
  u-blox documentation for the MAX-M10S says output becomes invalid above 500 m/s **[U]**. Space-capable receivers are therefore
  special factory variants: piNAV-NG/FM "up to 9 km/s" while the EM is software-limited to 500 m/s **[P]**; Pumpkin and
  GomSpace state "COCOM limits removed" **[P]**.
- **US EAR view** (Office of Space Commerce deck, 2022-03-18) **[P]**: "Spaceborne GNSS receivers are controlled by 9A515.x"
  and "Eligible for License Exception STA"; typical commercial GPS (no decryption, no adaptive antenna) is 7A994; 7A105.b.1 covers
  non-military receivers "designed or modified for airborne applications" capable of navigation above 600 m/s; USML Cat XV(e)
  lists no GNSS items; military receivers are USML XII(d)(2). 7A105 items are MT-controlled and lose most license exceptions.
- **Vendors disagree with each other:** Pumpkin self-classifies the GPSRM 1 as **ECCN 7A105.b.1** (data 7E101) **[P]**; I found no
  published ECCN for the other candidates. **Action:** obtain a written ECCN (or CCATS) from whichever vendor is chosen.
- **Why it matters for D6 [A]:** the CubeSat as a whole is likely a 9A515 spacecraft regardless of the receiver, so the receiver
  ECCN mostly affects handling, plus one trap: MT-controlled items such as 7A105 generally disallow license exceptions (OSC deck),
  so STA would likely be unavailable for a 7A105 receiver, which matters if the integrator is foreign (Exolaunch DE, ISISPACE NL)
  and the spacecraft is exported. Confirm with export counsel.
- **Foreign supply:** SkyFox: dual-use goods, HS 8526.91.20.20 **[U satsearch]**; the datasheet states the FM/EM "export is
  controlled and needs special Export License" from the Czech Ministry of Industry and Trade, applied for by the manufacturer
  with end-user and application details **[P, Rev. A/2025]**. The 2018 datasheet quoted **30-60 days** processing **[P, Rev. F/2018]**;
  Rev. A does not repeat it. Spacemanic: "End user declaration required" **[U satsearch]**.
- **End-use disclaimer (SkyFox Rev. A/2025) [P]:** the unit "must not be used ... on launchers, missiles ... or governmental end use
  or end user". A Falcon 9 rideshare payload is not "on the launcher" in the intended sense, but if the mission is facilitated by
  NASA (CSLI/ISS) or has a government end user, **ask SkyFox for written confirmation** before ordering.

### 3.2 Radiation
The baseline designs to 10 krad. TID statements: piNAV-NG none ("no Radiation Hardened integrated circuit are used"; 45 nm
correlator, 90 nm PVT) **[P]**; Celeste none **[P]**; NSS NGPS-03-422 "10 krad (component level)" **[P]**; AAC GPS-701 ">10 kRads" **[P]**;
Pumpkin GPSRM 1 "Not yet tested" **[P]**. **Treat the receiver as non-critical:** OBC-controlled load switch with a latch-up
current trip, power-cycle on watchdog, and a flight-software fallback (no fix, no ephemeris update, third-party path available).

### 3.3 Interfaces
- STM32L4R5 side: one UART plus a switched 3.3 V rail covers piNAV-NG (9600-8-N-1, LVCMOS 3.3 V, TXD only; RXD "not used").
  Celeste offers UART (direct to the GNSS module), I2C, RS485, CAN and CSP. OEM719 modules use LVCMOS serial / USB.
- The piNAV sentences `$PSLSP` (ECEF X,Y,Z in WGS-84 [m], GPS time of week [s], GPS week, #satellites, PDOP) and `$PSLSV`
  (velocity vector [m/s]) give state vectors directly; `$PSLSS` gives housekeeping (bus and core voltages, input current, antenna
  feed current, core temperature). `$PSLSP` example from the datasheet:
  `$PSLSP,3061.1001020,801,-6602427.515,97187.285,-1721790.821,7,2.6*7F`.
- **Timing:** VPP (valid position pulse) is a 1 PPS-class output, specified at 2σ ≤ 1 µs (Rev. A) **[P]**. The Rev. F table said
  100 ns while its text said 1 µs, so budget 1 µs. Celeste, GPSRM 1 and the OEM719 provide a PPS output as well.
- **Time scale trap:** piNAV sentences carry **GPS time** but SpaceX uploads require **UTC**. Apply the leap-second offset in
  ground software (18 s at last check; verify against current IERS bulletin).

### 3.4 EMC and RF
- 11th harmonic of the 145.8-146.0 MHz downlink = **1603.8-1606.0 MHz [A, arithmetic]**: about 30 MHz above GPS L1
  (1575.42 MHz) but **inside the GLONASS G1 band**. An L1-only receiver (piNAV-NG) has more rejection than a multi-constellation
  one (Celeste, OEM719 with GLONASS enabled). The baseline already keeps the 70 cm uplink away from 3 x the downlink (3 x 146 MHz
  = 438 MHz); no low-order harmonic of the 435-438 MHz receiver band lands in L1 (4 x = 1740-1752 MHz).
- SkyFox states the piNAV-NG was tested on a 1U CubeSat with a **1000 mW EIRP 435 MHz FM** transmitter and a 7-pole Pi LC
  harmonic filter with no degradation **[P, Rev. A/2025]** (500 mW without the filter statement in Rev. F). Not the same band as
  our 2 m downlink, but a useful precedent.
- The Jetson Orin Nano carrier (DRAM, switchers) and magnetorquer PWM are credible L1 noise sources **[A]**. SkyFox's own advice:
  observe C/N0 in GSV sentences while toggling subsystems, and test on a flight-representative model. **Plan an EM-level L1
  C/N0 test with the Jetson, PA and magnetorquers on and off.** Scheduling GNSS fixes when the Jetson and the 1 W PA are off is
  free mitigation (see §4.3).

### 3.5 Antenna
| Antenna | Size / mass | Gain / LNA | Supply | Connector | Price, lead [U] | Notes |
|---------|-------------|-----------|--------|-----------|-----------------|-------|
| SkyFox piPATCH-L1E1 | 98 x 98 mm, 50 g (M2.5 holes, "CubeSat Z+/- side") | not stated | 2.7-5.5 V, ~20 mA at 3.3 V (about 66 mW) | MCX-F (power over coax) | EUR 1,790-2,490; 2-4 weeks | **Occupies a whole 1U end face.** L1/E1 |
| piNAV-NG datasheet reference antenna | 35 x 35 mm patch + LNA (gain 13 dB-class, NF about 1 dB) on a 100 x 100 mm ground-plane PCB, copper, matching the Z-axis footprint | - | bias up to 3.3 V / 50 mA from receiver | MCX | - | "Best results" configuration, Rev. A/2025 [P] |
| Spacemanic ZENITH V3a (SM-ZEN-DS-0001 issue 2.1, 2026-02-12) [P] | 50 x 50 x 10 mm, about 20 g | peak gain about 4 dBi, efficiency about 70 %, RHCP, **LNA 30 dB**, NF about 2 dB | 3.3-5 V, 24 mW | MMCX female or MMCX male on RG-178 pigtail | EUR 800; 12 weeks (spacemanic.com) | L1/E1/B1/G1. Most plausible fit on a 1U face; MMCX vs piNAV-NG MCX needs an adapter or pigtail |
| NSS NANT-PTCL1 (v10a) [P] | 54 x 54 x 14.1 mm, <80 g | active gain >= 16 dBiC at zenith, NF <2 dB, -3 dB beamwidth >= 100 degrees | 5 V, <80 mW | SMA female | price on request | Operating -25 to +55 C |
| Tallysman/Calian TW1322 (GomSpace kit) | about 35 mm dia x 7.7 mm; kit total 54-57 g with module (datasheet) vs "antenna 54 g" on the shop page, which conflicts | single-band, pre-filtered | 2.5-16 V, <10 mA | cable 15-45 cm | with kit | GomSpace DS 1015610 rev 2.6 (2025-10-27) |

**Antenna pointing constraint (piNAV-NG Rev. A/2025) [P]:** keep the patch normal within a **+/-45 degree cone about zenith** to
fix within the stated TTFF; each satellite must be tracked "at least 30 seconds with C/N0 > 38 dBc-Hz" to decode ephemeris; once
a fix exists the receiver keeps tracking if the antenna moves toward nadir as long as four satellites stay above 30 dBc-Hz.
For a randomly oriented patch the fraction of time its normal lies in that cone is (1 - cos 45 deg)/2 = **14.6 %** per patch, 29 %
for two opposed patches **[A]**.

**Mechanical conflict with the baseline [A]:** the baseline puts two cells on each long face, deployable wings on the 1U faces,
and two deployable monopoles. A 50 x 50 mm ZENITH removes about 25 % of one 100 x 100 mm face's cell area; piPATCH-L1E1 removes a
whole face. This is the main open mechanical item. Keep the patch away from the VHF/UHF monopole bases (SkyFox: "as far from the
transmitting antennas as possible").

---

## 4. Comparative Analysis

### 4.1 Receiver comparison (specs are datasheet values unless tagged)

| | **SkyFox piNAV-NG/FM** | **Spacemanic Celeste** | **Pumpkin GPSRM 1 (OEM719)** | **NSS NGPS-03-422** | **GomSpace NanoSense GPS Kit (OEM719)** | **AAC GNSS-701 (ex-SpaceQuest GPS-701)** |
|--|--|--|--|--|--|--|
| Document, date | Rev. A/2025 (and Rev. F/2018) [P] | SM-CEL-DS-0002 issue 20.2, 2026-02-19 [P] | Rev. D, Aug 2022 [P] | v10a, 2020 [P] | DS 1015610 rev 2.6, 2025-10-27 [P] | GPS-701-ver-20180810 [P] |
| Constellations | GPS L1 C/A, 15 ch | GPS, GLONASS, Galileo, BeiDou | OEM719: L1 std; L1/L2 option; multi-constellation option | GPS L1, 12 ch | OEM719, L1/L2 capable, "dual-frequency" | A: L1/E1/B1; B,C more |
| Mass | 24 g | 25 g | 106 g [U, 2023 review] | <130 g | 31 g module; 54-57 g kit with antenna | 160 g |
| Size (mm) | 71.1 x 45.7 x 11 | 67 x 42 x 7.5 | PC/104 size, 104-pin CSK header; 96 x 90 x 12 [U review] | 96 x 91 x 18, PC104 footprint (4 x M3) | 46 x 72 x 11; **mounts only on GomSpace NanoDock** | 94 x 56 x 26 |
| Power | **125 mW typ** (passive ant.), **324 mW max** with active ant.; 540 mW inrush <1 s | **250 mW avg** (peak blank) (+24 mW antenna) | 3.3 V, 0-1.8 W, **typ 1.2 W**; supervisor <50 mW | **1 W** (excl. antenna); antenna <80 mW | <400 mA at 3.3 V (about 1.3 W max); [U review 1.8 W] | A 0.9 W, B 1.3 W, C 1.8 W (incl. active ant.) |
| Cold start (LEO?) | **80 s typ, stated "in LEO"** (Rev. F: 90 s) | startup hot/warm/cold 1/28/29 s; TTFF 29 s (conditions not stated) | <40 s cold, <19 s warm (typ, conditions not stated) | not published | not published | 90 s cold / 45 s warm / 30 s hot |
| Warm/hot restart after power-off | **No.** Nav data not stored in NVM; "warm 50 s" is only DROP recovery of a dropout while powered (<20 min) | hot 1 s / warm 28 s suggests retained data (retention across power-off not stated) | not stated in the Pumpkin datasheet; ask Pumpkin/NovAtel about backup power for warm starts | not stated | not stated | not stated |
| Accuracy | 10 m (2σ H); vel 10 cm/s (Rev. F) vs "0.1+1 per mille x v" (Rev. A, unclear, ask vendor) | 2 m CEP [U] | 1.5 m RMS L1 | <10 m (1σ), <25 cm/s | 1.5 m, 0.03 m/s | <5 m RMS, <0.1 m/s |
| Dynamics limit | 3600 km, 9 km/s (FM) | 10 km/s | "none" (COCOM removed) | not stated | "COCOM limit removed" | not stated |
| Interface | UART 9600 8N1 LVCMOS, NMEA 0183 + piNAV ($PSLSP/$PSLSV/$PSLSS), 1 Hz, VPP, PF | I2C, RS485, CAN, 2 x UART, PPS, CSP | LVCMOS serial x2, USB, PPS, VARF, I2C to supervisor; Vinti and SGP4 propagators on supervisor MCU | RS-422 or TTL UART, Nano-D | via NanoDock | 2 serial (LVCMOS/RS-422), USB, PPS |
| TID | none stated | none stated | not tested | 10 krad (component) | not stated | >10 krad |
| Heritage | line delivered from 2015, first flight 2017, >50 units, TRL 9 [U satsearch] | "10+ nanosatellite missions" (LASARsat, CroCube, GRBBeta, Planetum-1) [vendor] | "4x CubeSat-class LEO missions" [vendor] | >30 units delivered [vendor] | "tested and proven in flight" [vendor] | 42 delivered, 18 launched [vendor] |
| Operating temp | -40 to +85 C | -40 to +85 C | GEVS-qualified | **acceptance -10 to +50 C** | not stated | -40 to +85 C |
| Price (ROM) | EUR 3,290-6,190 [U satsearch] | EUR 5,200 [U] | **USD 11,485** (up to 20,026 with options) [U Pumpkin store; page shows "Unavailable"] | on request | on request | on request |
| Lead time | 2-4 weeks [U] + Czech export licence (30-60 d, 2018 figure) | **4-5 months** [U] | not stated | not stated | not stated | not stated |
| Export | Czech dual-use licence; HS 8526.91.20.20 [U]; end-use disclaimer | end-user declaration [U] | **ECCN 7A105.b.1** self-class; DDTC-registered seller; US persons unrestricted [P] | South Africa (not researched) | "export restrictions may apply to certain markets" | US (ECCN not found) |
| EMC note | Tested on 1U with 1 W EIRP 435 MHz FM + 7-pole LC filter | GLONASS G1 overlaps 11th harmonic of 2 m TX [A] | same as Celeste if GLONASS enabled [A] | L1 only | same as OEM719 | - |

### 4.2 Other options considered
- **Hyperion GNSS200:** only a SatCatalog entry (3 g, 0.157 W); the 3 g mass is implausible and I found **no datasheet** **[U]**. Not evaluated.
- **SSTL SGR-05U / SGR-Ligo:** a 2023 review lists 0.8 W / 45 g / >11 krad and <0.5 W / 90 g / >5 krad **[U]**; vendor pages returned 403.
- **Accord ACC-GPS-NANO (India):** 0.5-1.35 W, 45 g, 20 krad **[U review]**; not pursued.
- **Septentrio:** the space page says receivers can have COCOM limits removed **[U]** (fetch 403); no CubeSat-class space product found.
- **u-blox (COTS):** no sanctioned unrestricted variant; the u-blox forum says governmental constraints apply to all devices. Only
  flown example is a **u-blox-modified** receiver (new firmware for LEO dynamics) on an Astrocast test CubeSat (launched Dec 2018,
  Galileo/GPS OD below 5 m) **[U GPS World]**. Unflown candidates in the 2023 review are all marked "CoCom". **Not recommended.**
- **"Antelope" and "Celestia"** (named in the brief): no matching GNSS product found. "Celeste" (Spacemanic) is the likely intent.
- **Third-party ephemeris (the other half of D6):** LeoLabs reported a standard 12-month LeoTrack subscription at **USD 2,500 per
  month per satellite** (press release, about 2019; may be stale) **[U]**, roughly USD 60k over 24 months **[A]**. Kayhan (via Satcat) offers
  Space-Track ephemeris delivery; pricing not found. A third-party radar OD gives about 100s of m on a 1U **[U, USU SmallSat 2019]**
  and the guide does not define acceptance. Cost over the mission is above the GNSS BOM (EUR 4-9k plus ground NRE) **[A]**, and it
  leaves the guide's "GPS/GNSS-based" wording unmet unless SpaceX accepts it in writing.

---

## 5. Practical Recommendations

### 5.1 Ranked shortlist (judgment [A], scored on power, mass/volume, STM32 interface, evidence of LEO behavior, heritage, export friction, price, schedule)

| Rank | Choice | Why | Principal risks |
|------|--------|-----|-----------------|
| **1** | **SkyFox piNAV-NG/FM + patch (ZENITH V3a via MMCX-MCX pigtail, or SkyFox piPATCH variant with a smaller footprint if SkyFox offers one)** | Lowest power and mass; plug-and-play NMEA/UART; the only candidate whose datasheet states a TTFF **in LEO** and gives pointing/tracking rules; tested next to a 1 W 435 MHz FM transmitter; cheapest; 2-4 week lead | Cold start every power-up (no NVM): 80 s typ; export licence time and end-use clause; no TID; L1-only; velocity-accuracy line unclear; patch placement |
| **2** | **Spacemanic Celeste + ZENITH V3a** | Multi-constellation raises the chance of 4+ satellites in view on a tumbling or sun-pointed body; fast claimed TTFF (29 s) and 1 s hot start reduce session length; flexible buses (UART, I2C, CAN, CSP); 7.5 mm thick; single vendor for receiver and antenna | TTFF not stated for LEO; 4-5 month + 12 week lead; end-user declaration; GLONASS G1 vs 2 m 11th harmonic; vendor-only heritage list |
| **3** | **Pumpkin GPSRM 1 (OEM719)** | US supplier (no foreign export licence to import), COCOM removed, L1/L2 and multi-constellation options, on-board SGP4/Vinti propagators that run with the GPS unpowered | 1.2-1.8 W (needs duty cycling and a 5 V/3.3 V power path), 106 g, USD 11.5k+, "Unavailable" listing, 104-pin CSK interface on an STM32 OBC, TID untested, 7A105.b.1 |
| 4 | NSS NGPS-03-422 + NANT-PTCL1 | 10 krad component TID, 30+ delivered, PC104 footprint | 1 W + 130 g + 80 g antenna, -10 to +50 C acceptance window, TTFF unpublished, price on request, 2020 datasheet |
| 5 | AAC GNSS-701 | Space-qualified, >10 krad | 160 g, 0.9-1.8 W, 26 mm tall |
| - | GomSpace NanoSense GPS Kit | Good module (OEM719) | Only mounts on GomSpace NanoDock ADCS boards; exclude unless using GomSpace ADCS |

**Recommended D6 decision:** add a GNSS receiver (rank 1, with rank 2 as the qualified alternate, quoting both EM units now). Keep the
third-party path as a documented contingency (fallback also if the receiver fails on orbit).

### 5.2 Actions and dates (all relative to the baseline's "before integrator contract" need-by)
1. Email SpaceX space-safety-onboarding@spacex.com and the shortlisted integrators: (a) confirm GNSS-based ephemeris from a duty-cycled
   receiver (3-6 fixes/day plus ground OD) is acceptable; (b) ask what accuracy/covariance they require; (c) request a staging
   upload test. Ask SpaceX for its "third party support recommendations" as the contingency.
2. Request written quotes, ECCN/CCATS, export-licence timing and end-user wording from SkyFox and Spacemanic. Ask SkyFox to confirm the
   "governmental end use" clause is not triggered by a university/amateur mission. **Also ask SkyFox to clarify the Rev. A velocity
   accuracy line ("0.1+1 per mille x v" m/s):** read literally at 7.6 km/s it is about 7.7 m/s (2σ), versus 10 cm/s (2σ) in Rev. F.
   If it is really about 8 m/s, the OD must be position-only (the plan in §6.2 already does not depend on GNSS velocity).
3. Mechanical: decide the patch location (a 50 x 50 mm patch on a long face versus an end face) and the ground-plane size with the
   structure/solar-cell layout. Update the cell count in the power closure if a cell is lost.
4. EMC: schedule an EM test of L1 C/N0 with Jetson, 2 m PA, magnetorquers and DC-DC converters each toggled.
5. Build and test the ground OD/OEM chain before FM delivery (see §6.2), and register the object with the 18th SDS by L-2 months.
6. Update the repo's conjunction plan, which currently relies on a "GPS-free orbit estimate"
   (`docs/regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md`, step 3) once D6 is closed.

### 5.3 Resource estimates [A]
- Hardware: receiver EUR 3-6k plus antenna EUR 0.8-2.5k; mass 45-75 g including cable; flight-software effort about 1-2 person-weeks
  (NMEA/piNAV parser, load switch, scheduler, telemetry); ground OD + OEM generation + upload about 2-4 person-weeks using an open
  astrodynamics library; onboarding with Space-Track and SpaceX about 1 week of elapsed time.
- Schedule: allow about 3 months for a Czech export licence plus delivery; 6-7 months for Celeste plus antenna.

---

## 6. Technical Deep Dive

### 6.1 Duty-cycle and orbit-average power
Orbit: SSO about 500 km, period 94.6 min, **15.2 orbits/day** (computed; 450 km: 15.4, 525 km: 15.1) **[A]**.

Baseline context (`docs/BASELINE_PARAMETERS.md` §5): tumbling load 1.75 W against 2.39 W generation (margin 36.6 %); sun-pointed
2.15 W against 3.58 W (66.5 %); requirement 20 %. Headroom before the margin is violated: **0.242 W tumbling, 0.833 W sun-pointed**
**[A, derived]**. Therefore *continuous* GNSS at 0.324 W (piNAV-NG with active antenna, datasheet maximum) would fail the tumbling-case
margin; 0.125 W passes. Duty cycling removes the question.

Session model **[A]**: P_on x (cold TTFF x 1.5 + about 30 s ephemeris-decode/hold). Nominal session 90-180 s, conservative
180-300 s. Because piNAV-NG has no NVM, **every session is a cold start**.

| Case | P_on (W) | Wh per session (nom / cons) | Orbit-avg mW at 3 / 6 / 15 sessions/day, **nominal** | Same, **conservative** |
|------|----------|-----------------------------|-------------------------------|---------------------------|
| piNAV-NG + passive patch | 0.125 | 0.0063 / 0.0104 | 0.8 / 1.6 / 3.9 | 1.3 / 2.6 / 6.5 |
| piNAV-NG + active patch (about 0.19 typ) | 0.19 | 0.0095 / 0.0158 | 1.2 / 2.4 / 5.9 | 2.0 / 4.0 / 9.9 |
| piNAV-NG datasheet worst case | 0.324 | 0.0162 / 0.0270 | 2.0 / 4.0 / 10.1 | 3.4 / 6.8 / 16.9 |
| Celeste + ZENITH | 0.274 | 0.0069 / 0.0137 | 0.9 / 1.7 / 4.3 | 1.7 / 3.4 / 8.6 |
| NSS NGPS-03-422 + antenna | 1.08 | 0.0450 / 0.0900 | 5.6 / 11.2 / 28.1 | 11.2 / 22.5 / 56.2 |
| OEM719 (GPSRM 1 typ 1.2 W + antenna) | 1.3 | 0.0325 / 0.0650 | 4.1 / 8.1 / 20.3 | 8.1 / 16.2 / 40.6 |
| OEM719 at 1.8 W max + antenna | 1.9 | 0.0475 / 0.0950 | 5.9 / 11.9 / 29.7 | 11.9 / 23.8 / 59.4 |

(Nominal sessions: piNAV-NG 180 s, Celeste 90 s, NSS 150 s, OEM719 90 s. Conservative: 300 s, 180 s, 300 s, 180 s. Script:
`./duty.py`.) **A flat 0.05 W allocation line in the power table covers every row except the NSS (56 mW) and OEM719-at-1.8-W
(59 mW) cases at 15 sessions/day with conservative session lengths**, which are not needed in steady state. Budget 0.05 W even
though the expected steady-state draw is about 1-4 mW, because retries on a tumbling body are the uncertainty.

**Recommended operating modes [A]:**
- *Mode A, LEOP and cataloging (deployment to about T+7-14 days):* fixes every orbit (15/day) or near-continuous operation while the
  Jetson and transponder are off (commissioning loads are lower, so 0.19-0.32 W continuous is affordable for the piNAV-NG).
  First fix as early as possible after the 45-minute RF hold; the guide's best practice is an upload within 3 h of deployment.
  Required: first publication within 24 h of first contact and at least every 24 h.
- *Mode B, steady state:* 3-4 scheduled sessions/day spaced about 6 h apart (guide best practice is at least three ephemeris
  publications per day), plus one session before each planned own-station pass, plus an on-demand session after a conjunction warning.
  Expected 1-4 mW orbit average.
- *Session gating:* only when (i) the Jetson and 2 m PA are not active (eclipse or idle windows), (ii) battery SOC above the
  contingency floor, (iii) predicted patch-normal zenith angle under about 45 degrees (needs sun/magnetometer attitude estimate or a
  low body rate), (iv) temperature in range. Abort after 240-300 s without a 3D fix; retry next orbit; cap retries per day.
- *Tumbling case:* if body rates are high, the 14.6 % in-cone probability per patch means many sessions fail; use two opposed patches
  with a switch or a combiner (adds mass and a mechanical/RF item), or schedule fixes only after detumble to sun-pointed
  attitude. The datasheet says a tracked satellite can keep tracking while the antenna rotates toward nadir, so a lock achieved in
  cone survives short excursions.

### 6.2 Ground pipeline: forward-predicted ephemeris with covariance
The requirement is a *forward-predicted* ephemeris, so a single fix is not the product; an OD fit over several fixes is **[A]**.
1. **Data path:** put the latest fix (ECEF X, Y, Z, V, GPS time of week and week, #sats, PDOP, fix age; about 28-40 bytes) in every
   60 s beacon, so any SatNOGS-class station contributes to the data set; keep recent fixes in the 7-day housekeeping store for the
   own-station passes.
2. **OD:** batch least squares or EKF over the last 24-48 h of fixes with gravity (8 x 8), atmospheric drag with an estimated
   ballistic-coefficient scale, solar radiation pressure and third bodies. Output OEM KVN, UTC, ITRF (preferred) or EME2000, state
   vectors every 60 s over 3-7 days with a covariance block per record, plus quality metrics. Convert GPS time to UTC explicitly.
3. **Publish:** upload to Space-Track and to the SpaceX portal (OEM naming per the Space-Track Spaceflight Safety Handbook, not yet
   read) after each pass, at least every 24 h, within 3 h of deployment if possible.
4. **Uncertainty size, why drag dominates [A]:** a velocity error dv alone grows along-track as about 3 dv t: 0.1 m/s gives about 26 km after
   24 h (instantaneous GNSS velocity), which is why single-fix propagation is not used. A fit over a day of 10 m-class fixes reduces the
   effective dv by orders of magnitude. What remains is drag: at 500 km with density 5e-13 kg/m^3 [A, moderate activity], ballistic
   coefficient 26-44 kg/m^2 (baseline: 1.6 kg, Cd 2.2, area 0.028 m^2 wings deployed, 0.0164 m^2 wings failed), acceleration 3-6e-7 m/s^2; a
   **20 % drag error** (assumption) gives **about 240-415 m along-track after 24 h**; solar-maximum density can double this.
   That, not receiver accuracy (10 m, 2σ), sets the forward covariance and the publication cadence.
5. **Attitude-dependent area:** a tumbling 1U with wings has a drag area that varies; estimate a scale factor rather than assume Cd.
   The spin/sun-pointed state in the baseline changes it; carry the attitude mode in the OD.

### 6.3 piNAV-NG notes useful for integration [P, Rev. A/2025]
- Power: 2.7-3.6 V; 38 mA typ (about 125 mW) passive; absolute inrush 100 mA <1 s (540 mW incl. active antenna).
- `/RESET` low forces a cold start; without a fix for about 20 min the firmware restarts into Fast Acquisition. A forced power
  cycle is the supported recovery, so a switched rail is compatible.
- DROP propagator is gravity-only and degrades by about 6 km per 15 min, so it is *not* a source for the ephemeris; use real fixes
  only. The datasheet marks DROP-estimated output with `$GPGGA` mode 6 and `E` status in GLL/RMC/VTG, but the `$PSLSP`/`$PSLSV`
  examples carry no such flag: **cross-check the GGA/RMC flag before accepting an LSP/LSV record, reject DROP records in the OD,
  and ask SkyFox whether DROP can be disabled or flagged in LSP/LSV** **[A]**.
- Eval kit is PC/104+ form factor with a USB-serial cable, useful for FlatSat work.
- Operating limits (3600 km, 9 km/s) are for the FM only; the EM is limited to 500 m/s and will not track in orbit.

### 6.4 Open points and gaps in this research
- Not found: "Antelope" and "Celestia" GNSS products; a datasheet for Hyperion GNSS200.
- Not retrievable (HTTP 403): Septentrio space page, SSTL SGR-Ligo page, ScienceDirect Astrocast paper.
- NSS Gemini-SR5/SR28 datasheets not located; NSS TTFF not published.
- Space-Track Spaceflight Safety Handbook for Operators and the Voyager NRCSD IDD (current rev.) not read.
- TTFF values for Celeste (29 s) and Pumpkin (<40 s) read like ground specs; only SkyFox states a LEO cold start.
- Prices are list prices from distributor pages (satsearch, Pumpkin store, Spacemanic site) and have not been quoted to us.
- FCC DA 26-1080 (Part 100 effective-date notice) was seen only in search results.

---

## 7. References (all accessed 2026-10-15 unless dated)

### Launch provider and space-safety operators
- SpaceX, *Rideshare Payload User's Guide*, Version 11, August 2026, §1.2, §4.1.7.3, §9.5.7.1-9.5.7.5.
  https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf [P]
- Starlink Space Safety docs, trajectories and best practices (undated).
  https://docs.space-safety.starlink.com/docs/tutorial-basics/trajectories/ ; https://docs.space-safety.starlink.com/docs/best-practices/best_practices [P/U]
- Satcat docs on Space-Track linking via Kayhan; LeoLabs LeoTrack announcement; USU SmallSat 2019 LeoLabs accuracy paper [U].
  https://docs.satcat.com/operations/organization-settings/space-track-linking/ ;
  https://www.leolabs.space/news/leolabs-unveils-commercial-satellite-tracking-service-for-small-satellites/ ;
  https://digitalcommons.usu.edu/smallsat/2019/all2019/113
- NanoRacks lessons-learned briefing (SmallSat 2017) and NRCSD IDD (listed, not read). https://swfound.org:443/media/205941/nanoracks-secure-world-presentation-smallsat-2017-08072017.pdf [U]

### Regulation
- FCC 26-47, *Space Modernization for the 21st Century*, Report and Order and FNPRM, SB Docket 25-306, adopted 2026-07-22,
  released 2026-07-23: ¶¶286-287, ¶¶372-374, Appendix A §§100.5, 100.201(c). https://docs.fcc.gov/public/attachments/FCC-26-47A1.pdf [P]
- FCC Space Bureau public notice DA 26-1080 (2026-10-07). https://docs.fcc.gov/public/attachments/DA-26-1080A1.pdf [U]
- Federal Register 2024-08-09, doc. 2024-17093 (5-year disposal). https://www.govinfo.gov/content/pkg/FR-2024-08-09/html/2024-17093.htm [P]
- 47 CFR 97.207(g) (Cornell LII). https://www.law.cornell.edu/cfr/text/47/97.207 [P]; 47 CFR 25.114(d)(14) and 25.283 (eCFR API
  versioner, date 2026-10-01) [P]. The eCFR HTML page redirected to an unrelated host (unblock.federalregister.gov); I did not follow it.
- FCC ODM Plan Checklist, Streamlined Small Space Stations (shows (d)(14)(v)(C) item). https://www.fcc.gov/sites/default/files/ODM%20Plan%20Checklist%20-%20Streamlined%20Small%20Space%20Stations.pdf [U]

### Export control
- Office of Space Commerce, *U.S. Export Controls on GPS/GNSS Equipment*, 2022-03-18. https://www.space.commerce.gov/wp-content/uploads/2022-03-US-export-controls-GPS-GNSS-equipment.pdf [P]
- BIS 2014 final rule moving spacecraft items to 9A515 (not re-read). https://www.federalregister.gov/documents/2014/05/13/2014-10807 [U]

### Receiver and antenna sources
- SkyFox Labs, piNAV-NG Product Datasheet **Rev. A/2025** (bundled with Eval Kit Quick Start Rev. E/2024):
  https://www.skyfoxlabs.com/pdf/piNAV-NG_Datasheet_with_Eval_Kit.pdf [P]; **Rev. F/2018**:
  https://www.skyfoxlabs.com/pdf/piNAV-NG_Datasheet_rev_F.pdf [P]; product page https://www.skyfoxlabs.com/product/30-pinav-ng [P]
- satsearch listings: piNAV-NG FM (price EUR 3,290-6,190, 2-4 weeks, HS code) https://satsearch.co/products/skyfox-labs-pi-nav-ng-fm ;
  piPATCH-L1E1 https://satsearch.co/products/skyfox-labs-pipatch-l1e1-active-gps-galileo-patch-antenna [U]
- Spacemanic, *Celeste Datasheet* SM-CEL-DS-0002 issue 20.2 (2026-02-19)
  https://spacemanic.com/wp-content/uploads/2026/03/SM-CEL-DS-0002_20.2_Data-Sheet.pdf [P]; *ZENITH V3a* SM-ZEN-DS-0001 issue 2.1
  (2026-02-12) https://spacemanic.com/wp-content/uploads/2026/03/SM-ZEN-DS-0001_2.1_Data-Sheet.pdf [P]; price/lead:
  https://satsearch.co/products/spacemanic-celeste-the-gnss-receiver and https://spacemanic.com/products/zenith/ [U]
- Pumpkin, *CubeSat Kit GPSRM 1 GNSS Receiver Module* Rev. D (Aug 2022) https://www.pumpkininc.com/space/datasheet/710-00908-D_DS_GPSRM_1.pdf [P];
  store page (price) https://www.pumpkinspace.com/store/p58/GNSS_Receiver_Module_(GPSRM_1)_Kit.html [U]
- GomSpace, *NanoSense GPS Kit* datasheet DS 1015610 rev 2.6 (2025-10-27)
  https://gomspace.com/wp-content/uploads/2025/09/gs-ds-nanosense-gpskit-26.pdf [P]; shop page https://gomspace.com/?p=1034 [U]
- NewSpace Systems, *GPS Receiver* v10a (NGPS-01-422, NGPS-03-422, NANT-PTCL1)
  https://www.cubesatshop.com/wp-content/uploads/2020/11/NewSpace-GPS-Receiver_2020-10a.pdf [P]; https://www.cubesatshop.com/product/nss-gps-receiver/ [U]
- AAC Clyde Space / SpaceQuest GPS-701 datasheet (2018-08-10)
  https://www.aac-clyde.space/wp-content/uploads/2021/11/GPS_701_Datasheet_20180810_17a1fe01a8.pdf [P]
- Hyperion GNSS200 catalog entry https://www.satcatalog.com/component/gnss200/ [U]
- Independent review: "Overview of Space-Capable Global Navigation Satellite Systems Receivers: Heritage, Status and the Trend
  towards Miniaturization" (2023) https://pmc.ncbi.nlm.nih.gov/articles/PMC10490762/ [U: secondary, some values dated, e.g. Celeste 0.1 W]
- GPS World, "CubeSat finds its way in space with Galileo receiver" (modified u-blox on Astrocast test CubeSat)
  https://www.gpsworld.com/cubesat-finds-its-way-in-space-with-galileo-receiver/ [U]
- Septentrio space page https://www.septentrio.com/en/applications/assured-pnt-space/space [U, blocked]

### Repository context read (read-only)
- `docs/BASELINE_PARAMETERS.md` §3-§9; `docs/funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md` line 73;
  `docs/regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md`.
