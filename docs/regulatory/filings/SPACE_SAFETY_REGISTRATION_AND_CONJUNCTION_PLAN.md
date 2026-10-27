# Space Safety Registration and Conjunction Response Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-015
**Revision:** C
**Date:** 2026-10-27
**Status:** Plan. Registrations are not started.
**Supports:** 47 CFR 97.207(g)(1)(iv)(A)(5) (conjunction-warning certification) and (g)(1)(v) (trackability, identification, 18 SDS registration, ephemeris sharing)

---

## 1. Trackability (97.207(g)(1)(v))

| Item | SMART-QSO | Basis |
|------|-----------|-------|
| Smallest dimension, excluding deployables | 100 mm (1U, 10 × 10 × 11.35 cm) | CDS Rev 14.1 1U envelope |
| Presumption | **Presumed trackable**: "Space station(s) operating in low-Earth orbit will be presumed trackable if each individual space station is 10 cm or larger in its smallest dimension, exclusive of deployable components." | 97.207(g)(1)(v) |
| Tracking method | Passive radar skin track by the U.S. Space Surveillance Network, plus the onboard GNSS receiver (decision D6, BASELINE §6) feeding operator ephemeris (§3a) | — |

The 100 mm dimension is right at the presumption threshold. Do not reduce structure dimensions below the CDS 1U envelope.

## 2. Post-Deployment Identification (97.207(g)(1)(v)(A))

Deployments from multi-payload rideshares release many similar objects. SMART-QSO will be identified by:

1. **Deployment data**: dispenser ejection time, sequence and Δv from the launch provider, shared with 18 SDS through the provider.
2. **RF signature**: telemetry beacon on the IARU-coordinated downlink, with the call sign in every AX.25 frame. Each candidate TLE is tested against Doppler curves from SatNOGS and team stations, typically within 1–7 days.
3. **Association report**: the licensee emails the identified NORAD number to the provider and posts it on the project page and the AMSAT-BB mailing list.

## 3. Registrations

| # | System | Action | When | Owner | Status |
|---|--------|--------|------|-------|--------|
| R1 | **Space-Track.org** (U.S. Space Force 18th Space Defense Squadron, successor to the "18th Space Control Squadron" named in 97.207(g)(1)(v)(B)) | Create an organization account; request satellite-operator status; submit an Orbital Data Request if deployment-specific screening is needed. Confirm current onboarding steps on Space-Track.org. | ≥ 3 months before launch | Licensee | Not started |
| R2 | **TraCSS** (Commerce Office of Space Commerce, https://www.space.commerce.gov/traffic-coordination-system-for-space-tracss/) | Join the TraCSS waitlist/registration (open to organizations owning/operating spacecraft in orbit; registration opened early 2026). Funding beyond FY2026 is uncertain, so keep Space-Track as primary. | At launch contract; activate at deployment | Licensee | Not started |
| R3 | Launch provider | Supply the provider's operator-contact form (24 h phone, email, call sign, beacon frequency) for its 18 SDS pre-launch submission | Per provider ICD (typ. L-1 month) | PI | Not started |
| R4 | Public | Publish beacon frequency, format and decoder before launch (IARU form §6, RR 25.2A) | Before launch | Ops lead | Not started |
| R5 | NASA ISS trajectory operations (jsc-dl-topo-iwg@mail.nasa.gov) | Coordinate for ISS conjunction deconfliction: the disposal decay crosses the ISS altitude (SpaceX UG v11 §9.5.7.1) | With R1; again before the decay reaches ≈ 450 km | Licensee | Not started |
| R6 | 18th SDS registration of the deployed object, with the operator contact **opted into public listing for the mission lifetime** (UG v11 §9.5.7.1) | Required by L-2 M (UG §1.2) | L-2 M | Licensee | Not started |
| R7 | Claim the object with the 18th SDS | No later than 30 days after deployment (UG §9.5.7.2) | S+30 d | Licensee | Not started |
| R8 | Bounding hard-body radius for the appendage deployment at S+30 min, to SpaceX through the integrator: **0.6 m** (UG §3.3.4; `software/simulation/cdr/launch/leop_feasibility.py` §7) | Mission analysis inputs | L-8 M | Systems | Analysis done |

## 3a. Operator Ephemeris (SpaceX User's Guide v11 §9.5.7.2; 97.207(g)(1)(v)(C))

The launch contract requires forward-predicted GNSS-based ephemerides with covariance, published to Space-Track and to
SpaceX Space Traffic Coordination:

- the first within 24 h of first contact (best practice: within 3 h of deployment). For the baseline orbit the first
  pass over the Bay Area stations above 5° comes 0.7–12.2 h after separation (median 5.2 h), so the first ephemeris is
  due 24.7–36.2 h after separation; the 3 h best practice is not reachable from the Bay Area alone
  (`software/simulation/cdr/launch/leop_passes.py`, 576 separation cases);
- then after every pass and at least every 24 h until the 18 SDS has cataloged the object;
- best practice afterwards is three times a day.

A spacecraft without GNSS must arrange a third party before integration; failure can mean rebooking. Decision D6 adds a
GNSS receiver:

1. **Onboard:** the receiver is duty-cycled. During launch and early operations it takes a fix every orbit; afterwards
   it runs 3–6 sessions a day, with the Jetson and PA off. The latest fix (ECEF position, GPS time, satellite count,
   PDOP, fix age) is stored for downlink. Fixes marked as dead-reckoned (GGA quality 6) are discarded.
2. **Ground:** orbit determination over the last 24–48 h of fixes, estimating a drag scale factor. Drag error, about
   240–415 m along track after 24 h, dominates the covariance; receiver error does not.
3. **Output:** CCSDS OEM in KVN format, UTC (convert from GPS time), ITRF or EME2000. State every 60 s over 3–7 days,
   with covariance.
4. **Contingency:** if the receiver fails, a third-party orbit-determination service, accepted in writing by SpaceX and
   the integrator.

Open: Space-Track operator file-naming rules (Spaceflight Safety Handbook for Operators), and SpaceX acceptance of a
duty-cycled receiver.

**Status at CDR (2026-10-27): the onboard part of this chain does not exist.** The NMEA parser
(`software/rust/sqso-fsw/src/gnss.rs`) is not called by the executive; there is no receiver power schedule, no fix in
the beacon or in any command response, no telecommand to start a session, and no ground orbit-determination code.
The UG says that failing to demonstrate the capability may mean mandatory rebooking (§9.5.7.2). Until the chain is
built and demonstrated end to end before integration (verification item VI-LV-13 in
`docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`), plan on the third-party contingency, contracted before integration
(RID LV-04).

## 4. Conjunction Response (97.207(g)(1)(iv)(A)(5))

The FCC rule requires certification that "upon receipt of a space situational awareness conjunction warning, the
operator will review and take all possible steps to assess the collision risk, and will mitigate the collision risk if
necessary."

SMART-QSO has **no propulsion**, so it cannot maneuver. Mitigation options are limited to coordination and operations:

| Step | Action | Timeline |
|------|--------|----------|
| 1 | Receive a Conjunction Data Message (CDM) from Space-Track/TraCSS at the 24 h operator contact | Continuous |
| 2 | Review miss distance, probability of collision (Pc) and covariance quality; log in the anomaly/ops log | ≤ 4 h of receipt |
| 3 | If the other object is an **active, maneuverable** spacecraft: contact its operator (contact data via Space-Track), share SMART-QSO ephemeris (latest operator OEM from GNSS fixes, §3a; TLE if no recent fix) and confirm SMART-QSO cannot maneuver | ≤ 12 h, or before time of closest approach (TCA) |
| 4 | If Pc ≥ 1e-4 and the counterpart is crewed (ISS/CSS) or maneuverable: request that the counterpart maneuver; provide any data requested | Before TCA |
| 5 | Operational mitigation: none available. (Attitude changes that alter drag are negligible on conjunction timescales.) | — |
| 6 | Record the outcome and update the in-space notification if any disclosed parameter changed | After TCA |

**Certification text for the pre-space notification:** "The space station operator certifies that upon receipt of a
space situational awareness conjunction warning, the operator will review and take all possible steps to assess the
collision risk, and will mitigate the collision risk if necessary, including contacting the operator of any active
spacecraft involved, sharing ephemeris data and other appropriate operational information, and modifying space station
attitude and/or operations where this would reduce risk. The space station has no propulsion and cannot perform
avoidance maneuvers."

## 5. Habitable-Spacecraft Transit (97.207(g)(1)(iv)(A)(3))

If deployed above the ISS (~415–420 km), SMART-QSO will pass through the ISS altitude band during decay, and through the
Tiangong band (~380–390 km). Design and operational strategies to disclose:

- No propulsion and no planned proximity operations; uncontrolled decay through these altitudes is brief compared with total lifetime.
- Ephemeris and identification shared openly (Space-Track and public TLEs) so the operators of crewed vehicles can screen.
- If deployed from the ISS (NRCSD/J-SSOD option), the deployment geometry and conjunction screening follow the ISS program's CubeSat deployment process; the 30-minute and 45-minute deployable/RF inhibits (CDS Rev 14.1) apply.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial plan (funding audit) |
| B | 2026-10-15 | §3a operator ephemeris from the GNSS receiver (decision D6) |
| C | 2026-10-27 | CDR launch review: registrations R5–R8 (ISS coordination, public operator contact, claim within 30 days, hard-body radius) from SpaceX UG v11 §3.3.4 and §9.5.7; first-ephemeris deadline from the pass analysis; status of the GNSS chain (not implemented) |
