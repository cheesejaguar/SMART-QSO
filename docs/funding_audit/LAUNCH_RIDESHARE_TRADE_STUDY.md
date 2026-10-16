# Launch and Rideshare Trade Study
## SMART-QSO 1U CubeSat — launch window 2027–2028

**Document Number:** SQSO-AUD-004
**Revision:** B
**Date:** 2026-10-15
**Status:** Decision D7 taken 2026-10-15: rank 1 as primary, rank 2 as backup (`docs/BASELINE_PARAMETERS.md` §9). Updated from
[D7 research](research/D7_LAUNCH_INTEGRATORS.md). Prices are ROM unless a primary source is cited.

Evidence grades: **[P]** primary source read directly · **[S]** secondary (trade press) · **[A]** our own analysis ·
**[U]** unverified/ROM. Do not quote [U] items to a funder as commitments.

---

## 1. Decision Summary

| Rank | Option | Verdict |
|------|--------|---------|
| **1 (baseline)** | **SpaceX Transporter SSO rideshare through a 1U-capable integrator**, contracted for a deployment altitude **≤ 500 km** (hard limit 525 km) | Only route with posted 2027 flights. Viable if: (a) booked by about Q2 2027 because of reported post-2028 booking limits, (b) the deployment altitude is held ≤ 500 km so FCC 5-year disposal holds even if the solar wings fail to deploy (§3), (c) the GNSS-ephemeris duty is met, (d) export/ITAR integration-service controls are handled. |
| 2 (backup) | ISS deployment via Voyager (Nanoracks) NRCSD | Simplest debris case and a U.S. integrator, but a 0.3–1.1 year orbital life puts mission success Level 2 (1 year) at risk. It also carries the heaviest battery-safety burden (JSC 20793 Rev D), and ISS retirement is planned for 2030. |
| 3 (contingency only) | NASA CubeSat Launch Initiative (CSLI/ELaNa) | **Not available for a new 2027–28 launch**: NASA states the next solicitation is anticipated in **2028** [P]. Selection-to-launch has been 2–5 years [P], so launch would be about 2030 or later. Eligibility also needs a U.S. educational institution or nonprofit (e.g., a university or AMSAT partner). |
| Contingency (altitude control) | Rocket Lab Electron rideshare from New Zealand with a third-party 1U dispenser | The only route where a ≤ 500 km SSO is a design point rather than a menu choice. Price unpublished; New Zealand launch needs a spacecraft export licence (6+ months). One RFQ line. |
| Not recommended | Firefly Alpha, Isar Spectrum, PSLV, Vega-C SSMS, RFA, Gilmour; any Chinese provider | No 1U-capable rideshare evidenced for 2027–28, unproven or grounded vehicles, or a foreign launch site (export license + ITAR TAA). Chinese launchers are barred: 22 CFR 126.1(d)(1) policy of denial [P]. |

**The single most important funding fact in this study:** launch capacity for small CubeSats is tightening. SpaceX's
posted price is $7,000/kg (2026) rising to **$7,500/kg (2027)** with a 50 kg minimum [P]. Trade press reports that
SpaceX stopped accepting new Transporter/Bandwagon bookings beyond late 2028 (not confirmed by SpaceX) [S]. A
funding decision that slips past mid-2027 may leave **no commercial rideshare path before 2029**.

## 2. Mission Constraints That Drive the Trade

| Constraint | Value | Source |
|------------|-------|--------|
| Minimum useful orbital life | ≥ 1 yr (Level 2 full success); ≥ 2 yr (Level 3) | `docs/mission/MISSION_SUCCESS_CRITERIA.md` §2.2 |
| FCC disposal | Re-entry ≤ 5 years after mission completion, ≥ 0.9 probability | 47 CFR 97.207(g)(1)(vii)(D)(1) [P] |
| Debris casualty risk | < 1:10,000 (DAS) | 97.207(g)(1)(vii)(D)(2)(ii) [P] |
| Inhibits / timers | 3 RF inhibits, 3 deployable inhibits; deployables ≥ 30 min; RF ≥ 45 min after deployment | CDS Rev 14.1 §2.3.7, 2.3.8, 2.4.4, 2.4.5 [P] |
| Export | Spacecraft ECCN 9A515.a.5; launch-integration services are ITAR | [Export memo](../regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md) [P] |
| Propulsion | None, so altitude is fixed by the deployment orbit | Baseline |

## 3. Orbital Lifetime versus Deployment Altitude [A]

Computed with `software/simulation/orbit_lifetime/orbit_lifetime.py` (NRLMSIS 2.1, orbit-averaged; 1.33 kg; Cd 2.2;
launch 2028-01-01). **Nominal** uses an approximate solar-cycle profile (cycle 26 amplitude assumed); **bounding** holds
F10.7 at 70 throughout. An independent implementation by the research agent (constant F10.7 = 80–190, Ap = 12) agrees
within about 20 %. NASA DAS must be run before any filing.

| Deployment altitude | Tumbling body only (A = 0.0164 m²) nominal / bounding | Wings deployed (A ≈ 0.028 m²) nominal / bounding | Meets ≥ 1 yr life? | Meets FCC (mission 2 yr + 5 yr = 7 yr)? |
|--------------------|------------------------------|------------------------------|---------------------|-----------------------------------------|
| 400 km (ISS low) | 0.6 / 1.4 yr | 0.3 / 0.8 yr | **No** | Yes |
| 420 km (ISS high) | 1.1 / 2.3 yr | 0.5 / 1.4 yr | Marginal | Yes |
| 450 km | 2.7 / 4.5 yr | 1.2 / 2.6 yr | Yes | Yes |
| 475 km | 4.7 / 7.7 yr | 2.6 / 4.5 yr | Yes | Yes |
| 500 km | 6.0 / 13.0 yr | 4.6 / 7.6 yr | Yes | Yes (nominal); bounding fails without wings |
| 525 km | 7.1 / 21.4 yr | 5.8 / 12.5 yr | Yes | Marginal (nominal); bounding fails |
| 550 km | 8.5 / 34.4 yr | 6.8 / 20.1 yr | Yes | **No** under bounding; marginal nominal |

**Reading:** the sweet spot is a deployment altitude of **475–500 km**. With the baseline mass (1.6 kg, deployment mid-2028) the wings-failed case reaches 7.4 years at 525 km, beyond a 2-year mission plus 5 years, so 525 km is acceptable only with confirmed wing deployment ([BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md) §3). SpaceX posts Transporter SSO flights at
500–600 km [P], so the altitude must be negotiated with the integrator (orbit selection happens around L-5 months
[P]). If only 550–600 km is available, SMART-QSO needs a drag-augmentation device with demonstrated ≥ 0.9 deployment
reliability. That device is itself a deployable (3 inhibits, 30-min rule, SpaceX 7-day rule for secondary deployments
[P]) and a new design and test burden.

## 4. Option Assessments

### 4.1 SpaceX Transporter (SSO) via integrator — BASELINE

| Factor | Assessment |
|--------|-----------|
| Availability | SpaceX posted SSO rideshare flights NET 2027-06-01 and 2027-10-01 (500–600 km) [P]. 2026 flew Transporter-16 (Mar 30), -17 (Jul 7) and -18 (Oct 1) [P/S]. Post-2028 booking limits reported [S]. |
| Cost | SpaceX base $7,500/kg (2027 price year), 50 kg minimum = $375k [P], so a 1U is bought through an integrator; at 1.6 kg the mass itself is worth only ≈ $12k, so the slot price is mostly integration and margin [A]. **None of the seven integrators checked publishes a 1U price** (D7 research) [P]. Budget ROM: **$85k–$150k per 1U including integration** [U]; anchors $86k (2026 community post) and ≈ $85–90k (legacy Nanoracks) [U]. **All-in $115k–$250k** with program-side tests, export counsel, logistics and an ephemeris service; central case ≈ $150–170k. Obtain at least two written quotes before committing funds. |
| 1U-capable integrators | Exolaunch EXOpod Nova (Germany; SpaceX contracts through 2028; dedicated Exo-1 NET late 2027 and Exo-2 2028; EQT acquisition signed 2026-06-18, closing unverified), ISISPACE QuadPack/DuoPack (Netherlands; ISILaunch folded into isispace.nl), Voyager NRCSD (U.S., ISS only) per NASA SOA 2026 Table 10-2 [P]. Maverick (U.S.) and Rocket Lab dispensers list 3U+ only [P]. SEOPS (U.S.; Waymaker-1 dedicated Falcon 9 bookable from 2028) 1U support unverified [U]. D-Orbit ION placed customers at ≈ 510 km and ≈ 590 km; 1U acceptance unverified [P/S]. |
| Altitude | SpaceX's posted 2027 flights offer 500–600 km. The orbit is chosen at L-5 months from a list SpaceX sets for that flight (UG v11 App. G) [P], so an integrator cannot promise ≤ 500 km on its own. **Contract language:** 500 km target, 525 km hard cap, and no-penalty rebooking to the next Transporter, Exo or Waymaker flight if the assigned flight offers nothing at or below the cap; ask for the drop-off altitudes of the last 4–6 Transporters. |
| Export | Foreign integrator, so integration support is an ITAR defense service; SpaceX UG §9.2 requires a TAA or Export Compliance Agreement where foreign persons are involved [P]. Budget 3–6 months and counsel fees. |
| Environments | SpaceX User's Guide v11 (Jul/Aug 2026): random vibration required at no less than the 5.57 Grms MPE; **power-inhibit function verified during random vibe**; first mode > 40 Hz [P]. Integrator levels may be higher [U]. |
| Space safety | UG v11 §9.5.7.2: forward-predicted **GNSS-based ephemerides with covariance** to Space-Track and SpaceX within 24 h of first contact, then at least every 24 h until cataloged. Spacecraft without GPS must arrange a third party before integration [P]. **Decision D6: add a GNSS receiver** (piNAV-NG/FM, Celeste alternate; BASELINE §6), with third-party ephemeris as contingency. Register with 18 SDS by L-2 months [P]. |
| Deliverables | SpaceX Table G-3 via the integrator: export questionnaire/TAA, configuration webform, STEP CAD, mass properties, environmental test report, hazardous-systems declaration and SSCMAN 91-710 letter (L-6), FAA payload-review template and cross-waivers (L-5), battery-charge certificates, licensing letter with copies of all licenses (L-2) [P]. See [deliverables checklist](../regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md). |
| Mission fit | SSO gives global coverage, and at 475–500 km meets Level 3 (≥ 2 years) and FCC disposal (nominal, including the wings-failed case). |

### 4.2 ISS deployment (Voyager/Nanoracks NRCSD) — BACKUP

| Factor | Assessment |
|--------|-----------|
| Availability | NRCSD IDD Rev A (2024-01-19) current [P]; the D7 research (2026-10-15) could locate only the 2018 initial release, so obtain the current IDD from Voyager. ISS deployments continue; Cygnus above-ISS deployment is "on hold" [P]. NASA plans ISS retirement in 2030 (extension bills pending, not enacted) [S/U]. |
| Orbit / life | 51.6°, 400–420 km [P], so 0.3–1.1 years nominal for SMART-QSO [A]. **Level 2 (1 year) at risk; Level 3 (2 years) not achievable.** |
| Cost | Historical ~$85k per unit, undated [U]. All-in ROM $120k–$250k: slot plus JSC 20793 battery acceptance and cell screening, ground-leg inhibit design and safety data package [U]. Quote required. **Trigger:** if no integrator commits in writing to ≤ 525 km by 2027-06-30, or the primary exceeds $250k all-in, request the NRCSD quote and decide by 2027-09. |
| Safety burden | IDD: **≥ 3 independent inhibit switches including a ground-leg inhibit**, RBF/ABF, ≤ 6 in of ≥ 26 AWG wire from battery to first inhibit; batteries per **JSC 20793 Rev D**; Li-ion screening per NR-SRD-139; packs > 80 Wh need extra assessment [P]. Phase 0/I/II/III safety reviews from L-10 to L-3.5 months [P]. Mass limit 2.40 kg for 1U [P]. |
| Export | U.S. integrator and NASA handover, so the lowest export burden. |

### 4.3 NASA CSLI — CONTINGENCY ONLY

| Factor | Assessment |
|--------|-----------|
| Availability | "The agency anticipates releasing the next CSLI solicitation in 2028" (NASA apply page, updated 2026-10-14) [P]. A July 2025 NASA letter to AMSAT cited FY2026 budget-driven reductions and discontinued work with non-manifested missions [S]. |
| Cost | Launch and integration provided by NASA (2023 FAQ: integration up to $300k per mission including the deployer) [P, dated]. **No development funding** [P]. |
| Eligibility | U.S. educational institutions, nonprofits (with an education/outreach component) and NASA centers [P]. SMART-QSO would need an eligible lead organization. A non-negotiable CRADA is required [P]. |
| Requirements | LSP-REQ-317.01 Rev B: no propulsion or explosive devices, powered off from delivery to deployment, no RF until 45 min after deployment, ≥ 3 RF inhibits, NPR 8715.6, NASA-STD-6016 materials; vibe qualification MPE+6 dB/3 min, protoflight MPE+3 dB/2 min, acceptance MPE/1 min per axis [P]. Final report 9 months after launch [P]. CSLI will not select missions deploying parts smaller than 1U [P], which affects any tether/drag-device concept. |

### 4.4 Other vehicles — not recommended for 2027–28

| Vehicle | Reason |
|---------|--------|
| Rocket Lab Electron | Active (10 rideshare missions in 2025) [P], but Rocket Lab dispensers list 3U+ [P]; 1U needs an integrator adapter [U]; no public price [U]. Launch site/export treatment not verified [U]. |
| Firefly Alpha | Returned to flight March 2026; no CubeSat rideshare since July 2024 [S/P]. |
| Isar Spectrum | One successful flight (2026-09-05) [P]; foreign site (Norway), so export license plus TAA. |
| ISRO PSLV / Vega-C SSMS | PSLV failures in May 2025 and Jan 2026 with no confirmed return to flight [S]; no 2027 SSMS manifest found [U]; foreign site. |
| RFA One, Gilmour Eris | Not yet flown successfully [S]. |
| Chinese providers | Prohibited (22 CFR 126.1) [P]. |

## 5. Scoring

Weights reflect funder priorities: the ability to launch in the funded period, mission success and regulatory risk.
Scores run 1 (poor) to 5 (good).

| Criterion (weight) | Transporter via integrator | ISS NRCSD | CSLI |
|--------------------|---------------------------|-----------|------|
| Launch available 2027–28 (30 %) | 4 | 3 | 1 |
| Mission-success fit, orbital life (25 %) | 5 | 2 | 3 (orbit unknown) |
| Regulatory/debris risk (15 %) | 3 (altitude-dependent) | 5 | 4 |
| Export/ITAR burden (10 %) | 2 (foreign integrator) | 5 | 5 |
| Safety documentation burden (10 %) | 4 | 2 | 3 |
| Cost to program (10 %) | 2 | 3 | 5 |
| **Weighted score** | **3.70** | **3.15** | **2.95** |

## 6. Recommended Actions (time-critical)

| # | Action | By |
|---|--------|----|
| 1 | Request written quotes and 1U availability: Exolaunch (lead), ISISPACE (price check), SEOPS (US-owned; confirm 1U and LaunchLock terms), Voyager, and one question each to D-Orbit and Maverick. Ask for the 1U price and payment schedule, deployer and 1U envelope (113.5 mm, stowed wings), flight and its drop-off altitude list, the altitude clause above, vibe levels and unit-level testing, late access, ITAR path, ephemeris-service partner, and demanifest/rebook terms. Deal only with providers on NASA's dispenser list or SpaceX manifests | 2026-12-31 |
| 2 | Retain export counsel; decide whether a TAA is needed for the chosen integrator | 2027-01-31 |
| 3 | GNSS decided (D6: add a receiver). Quote piNAV-NG and Celeste EM units; confirm in writing with SpaceX and the integrator that a duty-cycled receiver plus ground orbit determination meets §9.5.7.2 | 2027-01-31 |
| 4 | Sign a launch reservation for a 2028 Transporter slot. This is the **launch-vehicle determination** that starts the 30-day FCC pre-space clock | ≤ 2027-06-30 |
| 5 | Keep the ISS NRCSD backup open: request the Voyager NRCSD schedule and JSC 20793 battery test plan cost | 2027-03-31 |
| 6 | Track the 2028 CSLI solicitation; identify an eligible lead organization if CSLI becomes the path | 2028 |

## 7. Sources

Full source register with access dates: research notes compiled 2026-10-14. Key primary sources: NASA CSLI apply page
(https://www.nasa.gov/announcement-of-partnership-opportunity-for-cubesat-launch-initiative/); SpaceX rideshare
pricing and flight API (https://rideshare.spacex.com/); SpaceX Rideshare Payload User's Guide v11
(https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf); NASA State-of-the-Art Small
Spacecraft Technology Ch. 10 (May 2026, https://www.nasa.gov/wp-content/uploads/2026/05/10-soa-launch-2026-final.pdf);
Voyager NRCSD IDD Rev A; LSP-REQ-317.01 Rev B; CDS Rev 14.1; 47 CFR 97.207; 14 CFR 450.43; 22 CFR 126.1. Trade press:
SatNews 2026-07-20, AIAA 2026-07-21, SpaceQ 2026-08-06 (booking limits); Via Satellite 2026-05-26 and 2026-10-01.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial trade (funding audit) |
| B | 2026-10-15 | Decision D7 recorded; integrator, altitude-clause, cost and backup-trigger updates from the D7 research; Electron as altitude-control contingency; GNSS decided (D6) |
