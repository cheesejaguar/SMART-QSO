# D7 Research Report: Integrator and Launch Options for SMART-QSO (1U, ~1.3-1.6 kg, 2028, deployment <= 500 km)

> Research input to decision D7, prepared 2026-10-15 and imported unchanged into the repository. Where it mentions working files (raw downloads, extracted texts, scripts) or says no repository files were modified, it refers to the research session; those files were not kept. How the decision was taken and applied is recorded in `docs/BASELINE_PARAMETERS.md` section 9.


**Prepared:** 2026-10-15/16 (all web sources fetched on those dates unless a different date is shown)
**Scope:** Decision D7 (integrator and flight at <= 500 km). Repository files were read but NOT modified.
**Evidence grades (same convention as `docs/funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md`):**
**[P]** primary source read directly this session; **[S]** secondary (trade press, aggregators, search-engine summaries of a page I did not read in full); **[A]** my own analysis or arithmetic; **[U]** unverified / ROM / from memory. Do not quote [U] or [S] items to a funder as commitments.

---

## 1. Executive Summary

**No 1U integrator publishes a price.** I checked Exolaunch, ISISPACE/ISILaunch, D-Orbit, Maverick, TriSept, SEOPS and Voyager. Every one routes pricing through a quote form [P]. The only 1U-level numbers on the open web are undated or weakly sourced (third-party guides at $40-80k, a July 2026 community post at $86k, a legacy Nanoracks figure of about $85-90k) [S/U]. The ROM ranges below are therefore planning numbers, not quotes. The existing study's band of $90-150k for a 1U slot is not contradicted by anything I found. I would move its low end to about $85k, because the one dated 2026 anchor is $86k.

**The raw SpaceX price is irrelevant to a 1U; the slot price is mostly integration.** SpaceX's own API (queried 2026-10-16) posts $7,500/kg for 2027 with a 50 kg minimum, i.e. $375,000 [P]. At 1.6 kg the mass-equivalent is about $12,000 [A]. A $85-150k integrator slot is therefore about 7-12x the raw per-kg price. The balance is dispenser share, mission management, aggregation risk and margin. Per-kg budgeting for a 1U is wrong; only written quotes matter.

**The real D7 constraint is capacity and altitude, not list price.**
1. *No 2028 Transporter is bookable on SpaceX's public portal.* The booking API lists exactly two flights: NET 2027-06-01 and NET 2027-10-01, both "SSO+/-0.1, 500-600 km" [P]. A 2028 flight can only come from an integrator's pre-bought allocation (Exolaunch says it has SpaceX contracts through 2028 [P]; Exolaunch Exo-1 NET late 2027 and Exo-2 2028, and SEOPS Waymaker-1 in 2028, are new dedicated Falcon 9 flights [P/S]). Trade press says SpaceX stopped taking Transporter/Bandwagon bookings past late 2028, with "minimal excess mass margin" left on existing flights; this is unconfirmed by SpaceX and rests on unnamed sources [S]. Treat it as plausible, because it is consistent with what the API shows.
2. *The deployment altitude is a menu, not a negotiable term.* SpaceX UG v11 Appendix G: the customer picks the orbit at L-5 months from a list "pre-populated ... selected by SpaceX" for that mission [P]. An integrator cannot contractually guarantee <= 500 km unless SpaceX offers it on that flight. SpaceX posts 500-600 km for both open 2027 flights [P]. Observed Transporter-class drop-offs: about 510 km (two D-Orbit ION carriers, Nov 2025 [P: NASA SOA 2026]) and about 590 km (D-Orbit ION on Transporter-17, 2026-07-07 [S]). **This is the largest risk to the baseline** and should be handled by contract language (section 6), not by hope.

**Recommended path (ROM):**
- **Primary:** Transporter-class SSO rideshare via a 1U-capable integrator (lead RFQ Exolaunch; price-check ISISPACE; US-owned alternative SEOPS), contract signed by 2027-06-30. ROM **$115-250k all-in** ($85-150k integrator slot + $30-100k program-side tests, export counsel, logistics, ephemeris service). Central case about $150-170k.
- **Backup:** ISS deployment via Voyager NRCSD (US company, lowest export burden, near-certain <= 420 km). ROM **$120-250k all-in**, but 0.3-1.1 yr orbital life (repo analysis [A]) puts Level 2 at risk, and the JSC-20793 battery program is the cost driver.
- **Contingency only:** NASA CSLI. Zero launch cost to the program, but the next solicitation is "anticipated in 2028" [P], the 2024 round produced no awards and non-manifested missions were cut in July 2025 [S], so launch is about 2030 or later.
- **Not viable for 2027-28:** PSLV, Firefly Alpha rideshare, Gilmour Eris, Vega-C SSMS, Bandwagon (45 deg, 550-600 km).

---

## 2. Technical Overview

### 2.1 How a 1U actually reaches orbit

SpaceX sells mass on rideshare plates (50 kg minimum "quarter plate"), and a CubeSat must fly inside an "Integrated CubeSat Dispenser Assembly" that has a single mechanical interface to the plate (UG v11 s1.2, s5.7) [P]. A 1U therefore always buys a slot from an integrator who owns plate capacity and a dispenser. Integrators (a) sell the slot, (b) supply and fill the dispenser, (c) own the SpaceX ICD and mission analysis, (d) are contractually required to enforce SpaceX's space-safety rules on their customers (UG s9.5.7.2: "Integrators are responsible for enforcing these requirements ... may result in mandatory re-booking") [P].

NASA's 2026 State-of-the-Art chapter (Ch. 10, May 2026) lists 1U-capable dispensers from Cal Poly (tab), Dhruva Space, Exolaunch (EXOtube/EXOpod Nova), ISISPACE (DuoPack/QuadPack), NPC Spacemind (SMPOD), UARX (RAMI), Voyager (NRCSD) and Xterra (XCD). Maverick's Mercury series and Rocket Lab's CSD/Maxwell start at 3U [P: Table 10-2].

### 2.2 Orbit classes available to a 1U and what they mean for disposal

Lifetime figures below are from the repo's own tool and trade study (`software/simulation/orbit_lifetime`, NRLMSIS 2.1; 1.33 kg, Cd 2.2) and were **not re-derived here** [A, carried forward]. They agree with common rules of thumb: a roughly 45 kg/m2 ballistic coefficient lasts about 1 year at 400-420 km and several years at 500 km.

| Orbit offered | Typical altitude / inclination | Source | Repo lifetime (tumbling / wings deployed, nominal) | FCC 5-yr disposal (2-yr mission + 5 yr = 7 yr) |
|---|---|---|---|---|
| ISS deploy (NRCSD, J-SSOD) | 400-420 km, 51.6 deg | NASA SOA 2026 s10.6.1 [P] | 400 km: 0.6 / 0.3 yr; 420 km: 1.1 / 0.5 yr | Passes easily; **fails 1-yr Level 2** |
| Transporter SSO | 500-600 km, SSO+/-0.1 (about 97.4-97.8 deg) | SpaceX flights API [P] | 500 km: 6.0 / 4.6 yr; 525 km: 7.1 / 5.8; 550 km: 8.5 / 6.8 | 500 km passes nominal (bounding fails without wings); 525 km marginal; 550+ fails under bounding |
| Bandwagon (mid-inclination) | 45 deg, 550-600 km | Wikipedia Bandwagon-3 entry [S] | similar to 550 km row | **Fails <= 500 km**; also a different orbit from the SSO power design |
| Electron SSO | 500 km SSO reference, LC-1 inclination 39-120 deg | Rocket Lab PUG 8.0 (Sept 2025) [P] | 500 km row | Passes nominal; customer-selectable altitude is the main attraction |
| Electron from Wallops (LC-2) | 38-60 deg inclination only | PUG 8.0 [P] | altitude selectable | Cannot reach SSO |
| D-Orbit ION | about 510 km (Nov 2025), about 590 km (Jul 2026) | SOA [P]; startupbusiness.it/Satnews [S] | varies | Altitude depends on the customer slot that is sold; ask whether <= 500 km is offered |
| Cygnus "above ISS" (NICL, SEOPS SlingShot) | 465-500 km | NASA SOA s10.6.2 [P] | n/a | **Both on hold** (Russia/Ukraine-related ISS policy); do not plan on it |

### 2.3 Mass and envelope fit (SMART-QSO CBE about 1.44 kg, MEV 1.7 kg, CDS limit 2.0 kg)

| Dispenser / route | 1U mass limit | Note |
|---|---|---|
| CDS Rev 14.1 | 2.0 kg | Repo baseline |
| NRCSD (ISS) | 2.4 kg | Repo trade study, from the NRCSD IDD (not re-read) [P-carried] |
| EXOpod Nova | "30% extra mass" over the CDS norm | Exolaunch brochure Q1 2026 [P]; confirm number in the ICD |
| Rocket Lab Maxwell (3U/6U/12U) | 2 kg per U | PUG 8.0 [P]; 3U minimum, so a 1U needs a third-party dispenser |
| JAXA J-SSOD (Kibo) | 1.33 kg per 1U | JAXA spec sheet via search summary [S]; **below MEV, so not viable without a mass cut** |

Also confirm each integrator's 1U slot accepts the 113.5 mm length and the stowed-wing keep-out volume.

---

## 3. Provider-by-Provider Findings

Prices are per 1U unless noted. "Not published" means I looked at the provider's own pages and found only a contact or inquiry route.

### 3.1 SpaceX Transporter / Bandwagon (direct)

- **Posted price [P]:** `https://api-rideshare.spacex.com/api/v2/price` (the data feed behind rideshare.spacex.com), queried 2026-10-16. Values are in cents: 2023 $5,500/kg; 2024 $6,000; 2025 $6,500; 2026 $7,000; **2027 $7,500/kg**. Minimums: quarter plate 50 kg, half 100 kg, full 200 kg, XL 300 kg. So the minimum purchase is $375,000 (2027) or $350,000 (2026). No 2028 price is posted. Extrapolating +$500/kg/yr gives $8,000/kg and a $400,000 minimum for 2028 [A/U]. Optional add-ons include an 8-inch separation system (2027: $370,500), a fuel option and an "Insurance" line with value 5 in every year (units undocumented; presumably a percentage) [P/U].
- **Flights bookable on the public portal [P]:** NET 2027-06-01 and NET 2027-10-01, both SSO, 500-600 km. Nothing for 2028.
- **Cadence [P/S]:** Transporter-12 2025-01-14; -13 2025-03-15; -14 2025-06-23; -15 2025-11-28; -16 2026-03-30 (119 payloads); -17 2026-07-07 (81 payloads); -18 2026-10-01 (130 payloads). About 3-4 SSO flights per year, plus about 2 Bandwagons per year (Exolaunch heritage list: Bandwagon-1..4 in Apr 2024, Dec 2024, Apr 2025, Nov 2025).
- **Lead-time shape (UG v11 Fig. 1-2) [P]:** mission analysis inputs L-8 months; range-safety submissions L-6, L-4, L-2; orbit selection L-5; final analysis inputs L-3.5; registrations with 18th SDS L-2.
- **Rules that touch SMART-QSO [P]:** random vibration MPE 5.57 Grms with power-inhibit verification during vibe (UG s4.1.6, Table 6-2; containerized-CubeSat unit tests may be done at unit level with mass models standing in); no elastic modes below 40 Hz; no passive appendage deployment during CubeSat ejection (s5.7); forward-predicted **GPS/GNSS ephemerides with covariance** to Space-Track and SpaceX within 24 h of first contact, or a third party arranged before integration (s9.5.7.2); US export law applies to "any foreign person" including the customer, and SpaceX may rebook or terminate with a refund if a license is unobtainable (s9.2).
- **Verdict:** Buy through an integrator. Direct purchase makes sense only for a consortium of 50 kg or more.
- *Unreliable claim noted:* Satnews (2026-07-20) quotes a "sub-$300,000 baseline price point for standard CubeSat slots." That conflicts with every other source and is almost certainly the 50 kg minimum misdescribed. Disregard it.

### 3.2 Exolaunch (Berlin; offices in US, France, Japan)

- **Price:** Not published; contact form only [P: exolaunch.com]. No dated public 1U figure. Third-party guide: $40-80k for a 1U via a broker, undated and unsourced [S/U].
- **What is included [P: Q1 2026 brochure]:** launch planning, mission management, environmental testing at its Berlin facility (optional), logistics and customs clearance, integration, deployment. EXOpod Nova/Air accept 0.25U to 16U in subdivided slots, with windows for RBF removal; the brochure states the hardware is "ITAR-free ... not subject to export restrictions of any kind." (Hardware being ITAR-free does **not** remove ITAR/EAR obligations on the US spacecraft or on integration services; see section 4.4.)
- **Capacity [P]:** more than 160 contracts with SpaceX since 2020; new multi-year contracts extend capacity "through 2028" (SSO, mid-inclination, near-polar, dawn-dusk) (exolaunch.com/news_127, dated 2025-05-20 on the page; the BusinessWire URL carries 2025-03-17, so the exact date is unresolved). Two dedicated Falcon 9 flights: **Exo-1 NET late 2027, Exo-2 2028**, EXOtube stacks, orbit and altitude not disclosed (Via Satellite 2026-05-26; Exolaunch news_153) [P/S].
- **Heritage [P]:** 47 missions and 790+ satellites (EQT press release 2026-06-18); flown on every Transporter and Bandwagon; also Electron (Rocket Lab), Ariane 6 (July 2024, 4 smallsats), PSLV.
- **Corporate:** EQT X signed a definitive agreement to acquire Exolaunch on 2026-06-18, closing expected Q4 2026, terms undisclosed (PR Newswire) [P]. One search-result headline says "EQT Completes Acquisition"; **closing status is unverified.**
- **2025-26 manifest examples:** Transporter-16: 57 Exolaunch payloads (26 microsats, 31 CubeSats, 25+ customers) [S]. Transporter-17: 49 customer satellites including MARMOTSat (3U, Canadian CUBICS), KOSTKA (Brno University, IARU-coordinated 436.870 MHz; **size disputed: Wikipedia and a URL slug say 1U, nanosats.eu says 3U in an EXOpod Nova 12U/16U**), Balkan-3 Cuby-1..5 (EnduroSat) [P: exolaunch.com/mission_48; libre.space thread] [S]. I found **no verified 1.5U or 2U Transporter example**; sizes are rarely published.
- **Free-slot program:** UNOOSA/Exolaunch "Access to Space for All" gives selected entities one EXOpod slot and launch for a 1U-3U CubeSat at no charge (2024 call) [S]. I could not retrieve the call (404), so eligibility is unknown; it is aimed at UN-member developing-country entities and probably does not apply to a US team [U].

### 3.3 ISISPACE / ISILaunch (Delft, NL)

- **Naming:** isilaunch.com now 301-redirects to isispace.nl/launch-equipment-services [P, observed]. ISILaunch is the launch-services arm of ISISPACE.
- **Price:** Not published. The inquiry flow is: submit size, mass, orbit and schedule; ISISPACE then defines opportunities, scope and price [P]. Third-party guides list ISILaunch among brokers pricing by U and orbit [S].
- **Hardware / services [P]:** DuoPack (heritage since 2014) and QuadPack (since 2016) for 1U to 16U; launch management; integration; launch-campaign execution. Claims 826+ CubeSat and microsat launches since 2009.
- **Manifest:** ISL56 listed for 2026-07-07 on Transporter-17. The page lists no 2027 mission [P].
- **Notes:** Netherlands-based, so the same ITAR integration-services question as Exolaunch. The deployer's export status is not stated on the page.

### 3.4 D-Orbit (ION Satellite Carrier, Italy)

- **Price:** Not published [P]. A 1U is a very small customer for an OTV; 1U acceptance is **not evidenced** [U].
- **Relevance:** the one rideshare route with propulsion on board, so the customer may be placed at a slot different from the plate's drop-off. Demonstrated: two ION carriers delivered eight missions to SSO at about 510 km in Nov 2025 [P: SOA 2026]; ION on Transporter-17 placed at about 590 km SSO [S]. Whether ION can offer <= 500 km is an RFQ question [U].
- **Status [P: dorbit.space, 2026-10-16]:** 26 orbital transportation missions, 89 hosted payloads, 168 satellites deployed; next launch February 2027; SOA counts 21 uses through Dec 2025. Independent reports: mission 22 (Wayfinder) on Transporter-16, mission 23 on Transporter-17 [S].

### 3.5 Voyager Technologies (ex-Nanoracks) NRCSD

- **Names:** Nanoracks -> Voyager Space -> Voyager Technologies (current site voyagertechnologies.com; the NASA SOA and ESA-hosted portfolio still say "Voyager Space") [P]. (The 2021 acquisition of Nanoracks is from memory [U].)
- **Service [P: Voyager NRCSD datasheet; NASA SOA 2026 s10.6.1]:** self-contained deployer launched pressurized on ISS resupply vehicles, moved out through the Kibo airlock and released by the JEM arm. 12U total per unit (1U up to 6x1U; 2x3U; 2x6U), up to 48U per cycle, 0.5-2.0 m/s, tip-off up to 5 deg/s/axis, independent inhibits, no electrical interface, real-time deployment support. Deploys **1-3 months after berthing** at **51.6 deg, 400-420 km**.
- **Price:** Not published. Legacy anchor "about $85k per unit" (undated, already in the repo study) and "about $90,000 per 1U" (an older guide) [U]. No 2026 figure found.
- **Safety burden (carried from the repo study; not re-read):** JSC 20793 Rev D battery requirements, NR-SRD-139 Li-ion screening, 3 independent inhibits including a ground-leg inhibit, RBF/ABF, NASA Phase 0-III safety reviews from L-10 to L-3.5 months, extra assessment above 80 Wh (SMART-QSO is about 50 Wh), 1U mass limit 2.4 kg. **I could only find the 2018 initial release of the NRCSD IDD on NASA's SSRI knowledge base; I could not locate the "Rev A, 2024-01-19" the repo study cites.** Get the current IDD from Voyager.
- **"Free-flyer" options:** None for CubeSats. Bishop airlock (attached Dec 2020) and Kaber (24U / 82 kg microsat deployer) are ISS-based; Starlab is a planned commercial station, not a satellite-deployment service [P/S].
- **Above-ISS options on hold [P: SOA s10.6.2]:** Nanoracks NICL (465-500 km via Cygnus) and SEOPS SlingShot (about 500 km after Cygnus departs).
- **2025-26 NRCSD 1U example:** none found. This is a gap. Kibo J-SSOD (JAXA) deployments in 2025-26 are documented (see 3.5.1).
- **Orbital life:** 0.3-1.1 yr nominal for SMART-QSO [A, repo]. ISS retirement is planned for 2030 (extension bills pending) [S/U, carried forward].

#### 3.5.1 Kibo J-SSOD via Space BD / Mitsui Bussan Aerospace (alternative ISS route)
JAXA-appointed service providers since May 2018. Supports 1U-6U, but the spec sheet limit is **1.33 kg per U** [S], below SMART-QSO's MEV. 2025-26 1U/2U examples: RSP-03 (1U), Atsushi Space Challenge (1U), GHS-01 (2U), deployed 2025-09-19; six CubeSats (J-SSOD #35) Feb 2026 [S: JAXA news via search]. No published price. The spacecraft would be exported to Japan (EAR 9A515 license or STA to an A:5 country per repo memo) [A]. Not recommended.

### 3.6 Maverick Space Systems (San Luis Obispo, CA; US)

- Mercury-3/6/12 (tab and rail), NLAS 6U; SOA Table 10-2 lists 3U, 6U, 6T, 12U, 12T, 16U: **no 1U product** [P]. One of five vendors on NASA's CubeSat deployer supply IDIQ [S]. Contributed customer satellites to Transporter-18 (Via Satellite 2026-10-01) [P]. No price [P]. No acquisition or merger found in 2025-26 [S]. A 1U would need a 3U slot plus adapter or a partner dispenser [U].

### 3.7 TriSept (US)

- Government-focused integrator (founded 1994; CEO Rob Spicer per its site). Current news: Army SMDC cubesat on Electron, Gunsmoke-J dispenser integration, RocketStar student-payload launch, DRAG RACER; **no 1U product, no price, no ownership change found** [P: trisept.com, 2026-10-16]. A search for an acquisition returned only unrelated "TriSep" membrane companies (noise). Low prior for a 1U amateur mission.

### 3.8 SEOPS (Space Exploration Operations and Services; Giddings, TX / Nashville, TN)

Not on your required list but material because it is **US-based**:
- Equalizer dispenser: reported to support 1U-12U tab or rail, up to 96U [S]; the SEOPS site fetched this session gives no sizes, so **1U support is unverified**.
- **Waymaker:** new dedicated LEO rideshare program, first mission Waymaker-1 on a purchased dedicated Falcon 9, **bookable starting 2028**, RIDE! has booked 1,000 kg (Via Satellite 2026-05-26) [P/S]. Darkstar GTO rideshare 2028+. Five more dedicated Isar Spectrum launches for 2028-2030 from Andoya and Nova Scotia (announced 2026-09-15; six total) [S]. **LaunchLock** advertises a "firm, fixed price" with launch windows reserved up to 3 years ahead [P].
- Transporter-16: 19 SEOPS payloads; Transporter-18: 18 Planet SuperDoves [S/P]. President Evan Hoyt: there is a "sunset period" on the horizon for Transporter [S].
- Why it matters: a US integrator avoids the foreign-person integration-services question (repo export memo, case 1).

### 3.9 Rocket Lab Electron (rocketlabcorp.com; formerly rocketlabusa.com)

- **Price:** Not published. Aggregators quote about $7.5M for a dedicated Electron and a $15-25k/kg band for Electron/Alpha (SpaceQ 2026-08-06) [S/U]. No rideshare per-U price found.
- **Facts [P: PUG v8.0, Sept 2025; NASA SOA]:** 200 kg to a 500 km SSO; Launch Complex 1 (Mahia, NZ) inclinations 39-120 deg; LC-2 (Wallops) 38-60 deg only, so **SSO requires NZ** (foreign launch site: BIS license or STA plus ITAR integration authorization per the repo memo). Kick Stage can deliver rideshare payloads to different precise orbits. Rocket Lab's own dispensers (Maxwell, CSD) are 3U/6U/12U; Electron supports third-party separation systems, and rideshare CubeSats are integrated into dispensers at Long Beach about 40 days before launch; payloads clear customs at Auckland. 10 rideshare missions flew in 2025; an April 2026 JAXA rideshare put eight CubeSats in SSO [P/S].
- **Verdict:** the only route in this list where a <= 500 km SSO is a design point rather than a menu option, but it requires a third-party 1U dispenser (for example EXOpod, which has flown on Electron), a foreign launch site, and an unquoted price. Worth one RFQ line.

### 3.10 Firefly Aerospace (Alpha) and Spaceflight Inc.

- **Spaceflight Inc.:** acquired by Firefly (announced 2023-06-08); Spaceflight's rideshare aggregation was wound down after existing contracts; Sherpa OTVs retired; SpaceX had already cut ties in March 2022 [S: SpaceNews/Via Satellite/GeekWire; corroborated by NASA SOA s10.5.1: "retired SHERPA vehicle by Firefly (previously Spaceflight Inc.)" [P]]. **Exited.**
- **Alpha:** returned to flight 2026-03-12 (Flight 7, Block I, Lockheed Martin demonstrator); Flight 8 (Block II) undated; the last CubeSat rideshare was ELaNa 43 in July 2024 [S]. Elytra OTV is offered, with no schedule [P: SOA]. Not viable for 2027-28.

### 3.11 ISRO / NSIL PSLV

- PSLV-C61 (May 2025) and PSLV-C62 (2026-01-12; third-stage chamber-pressure drop; lost EOS-N1 and 15 smaller payloads) both failed; no return-to-flight date found [S]. NSIL prices are not published. Foreign site. **Not viable** until return to flight is demonstrated.

### 3.12 Other new small launchers

| Vehicle | Status (as of 2026-10) | Offers 1U rideshare? |
|---|---|---|
| Gilmour Eris (Australia) | Flight 1 failed July 2025 (14 s); TestFlight 2 "end of 2026", TBD [S] | No evidence |
| Isar Spectrum (Norway) | First fully successful flight 2026-09-05 (second flight overall) [S]; SEOPS holds 2028-30 launches; Isar's 2028 manifest "closed" [S] | Via SEOPS only; foreign site |
| Vega-C / SSMS | Avio took over Vega-C marketing; no SSMS rideshare on 2026 manifest [S] | No 2027-28 manifest found |
| Ariane 6 rideshare | 4 smallsats flew in July 2024 via Exolaunch [P brochure]; no 2027-28 offer found | No evidence |
| RFA One, Orbex, Skyrora | No successful orbital flight found | No |
| Chinese providers | Barred: 22 CFR 126.1 (repo) | No |

### 3.13 Companies that renamed, merged or exited

| Entity | Change | Evidence |
|---|---|---|
| Spaceflight Inc. | Acquired by Firefly 2023-06; rideshare brokerage ended | [S]+[P SOA] |
| Nanoracks | Now Voyager (Space -> Technologies); NRCSD retained | [P] |
| ISILaunch | Folded into isispace.nl (301 redirect) | [P] |
| Exolaunch | EQT X deal signed 2026-06-18; close expected Q4 2026 (closing unverified) | [P] |
| Arianespace -> Avio | Avio markets Vega-C; last Arianespace-managed Vega-C 2025-12-01 | [S] |
| Rocket Lab USA | Domain moved to rocketlabcorp.com; Planetary Systems' CSD now listed under Rocket Lab | [P] |
| Virgin Orbit, ABL | Left the launch business (2023, 2024) | [U, memory] |
| Maverick, TriSept | No ownership change found | [S] |

---

## 4. Implementation Considerations

### 4.1 NASA CSLI (the "free launch" path) and the Santa Clara question

**What NASA says (page "Announcement of Partnership Opportunity for CubeSat Launch Initiative", updated 2026-10-14) [P]:**
- Eligible developers: **U.S. educational institutions, nonprofit organizations, and NASA centers**. Individuals are not mentioned. The program page adds that informal educational institutions (museums, science centers) count as nonprofits.
- "The agency anticipates releasing the next CSLI solicitation in **2028**." As of Sept 2026, 228 missions had been selected and 183 launched.
- NASA does not fund development. External participants need a **non-reimbursable Cooperative Research and Development Agreement (CRADA)** with standard terms for all CSLI collaborators; participation depends on NASA finding a launch opportunity.
- Historical cadence: solicitation typically released each August, proposals due November [P]; the last awards I could document were the 15th round (call 2023-08-07; selections about March 2024; flights planned 2025-2028) [S]. Selection does not guarantee a launch (2012 release language) [P].

**Recent contraction [S]:** AIAA's Aerospace America reports that LSP told participants in July 2025 that the **2024 Partnership Opportunity would produce no awards** and nothing would be released in 2025, and that already-selected but unmanifested missions would no longer be manifested. AMSAT's ANS-222 bulletin describes NASA letters dated **2025-07-12** citing the FY2026 budget request and personnel cuts: cooperative work ends on missions that have no scheduled launch, and NASA "cannot provide launch opportunities" for AMSAT's GOLF-TEE (3U) and GOLF-1, and does not expect that to change; the CSLI panel was to reconvene in spring 2026. AMSAT's response was to finish GOLF-TEE and use "private integrators and launch services," noting commercial rides can be secured "in months rather than years." As of May 2026, GOLF-TEE's completion was targeted for early 2027 and I found no booked provider [S]. **This is the closest precedent for SMART-QSO: a CSLI-selected amateur CubeSat that never got its NASA ride.**

**How an individual amateur licensee with a university partner could qualify [A]:**
1. The eligibility attaches to the **developing organization**, not the radio licensee. Santa Clara University (an accredited U.S. university) could be the CSLI applicant and sign the CRADA; alternatively a U.S. nonprofit (AMSAT-NA, or a new 501(c)(3)) could be the applicant, with the university as a partner.
2. The FCC Part 97 licensee and control operators (decision D1) are a separate matter. In principle the licensee can be an individual call sign or a club station, but NASA-side requirements (education/outreach component, who signs, liability) must be checked against the 2028 announcement. **Unverified; NASA says details will be published closer to release.**
3. Expect launch about 2030 or later (selection-to-launch has been 2-5 years per the repo study [P]) and a build to LSP-REQ-317.01 Rev B (no RF until 45 min after deployment, 3 RF inhibits, powered off until deployment; vibe MPE+6 dB/3 min qualification, MPE+3 dB/2 min protoflight) [P-carried from the repo].
4. The NASA 2023 FAQ figure of integration "up to $300k per mission including the deployer" [P-carried, dated] is what NASA absorbs; the program still pays for build, test and travel (ROM $30-70k [U]).

**Other free or subsidized routes [P/S/U]:** ESA Fly Your Satellite and JAXA/UNOOSA KiboCUBE target ESA-member and developing-country teams (not US); NASA USTP funds up to two years of university SmallSat development but does not buy a launch (SOA Ch. 10) [P].

### 4.2 What the integrator typically provides vs what the program supplies

| Item | Integrator quote usually includes | Program supplies |
|---|---|---|
| Dispenser slot, door/pusher hardware | Yes (EXOpod, QuadPack, NRCSD) | CubeSat-to-dispenser ICD inputs |
| Mission management, SpaceX ICD, mission analysis | Yes | Mass properties, STEP model, inhibit/RF data |
| Launch-site transport, integration | Yes | Shipping to the integration site; travel |
| Environmental testing | Optional (Exolaunch Berlin facility) | Test at provider levels (random vibe >= 5.57 Grms MPE with power-inhibit check; TVAC; bakeout) |
| Export (ITAR/EAR) | Integrator ships; customer responsible for license | Classification (9A515.a.5 presumptive), TAA/license, counsel |
| Space-safety ephemeris | Integrator enforces | GNSS receiver or a third-party service (D6) |
| FCC / IARU / Space-Track | No | Customer licensing; FCC letter by L-2; 18 SDS registration by L-2 |
| Late battery charge, N2 purge | Generally not on rideshare (SOA s10.3.1) | Plan for no late access |

### 4.3 Documentation by path (carried from the repo study unless marked)

- **Transporter via integrator:** integrator services agreement; payload questionnaire; STEP model; mass properties; hazardous-systems declaration and SSCMAN 91-710 letter (L-6); FAA payload review package and cross-waivers (L-5); battery charge certificates; licensing letter with FCC/IARU copies (L-2); environmental test report; orbit selection webform (L-5, [P: UG App. G]); transmitter turn-on verification (UG s4.1.7.2) [P]; harness verification worksheet [P].
- **ISS NRCSD:** NRCSD IDD; JSC 20793 Rev D battery program; independent inhibits plus ground-leg inhibit; NASA Phase 0-III safety reviews (L-10 to L-3.5 months); NR-SRD-139 cell screening; materials/outgassing data.
- **CSLI:** CRADA; LSP-REQ-317.01 Rev B; NPR 8715.6 orbital debris; NASA-STD-6016 materials; final report 9 months post-launch.

### 4.4 Export and ITAR: both statements are true

Exolaunch advertises its **hardware** as ITAR-free [P]. The repo export memo (SQSO-REG-012, presumptive ECCN 9A515.a.5) says **launch-integration services provided to foreign persons** are ITAR defense services, which may require a TAA even at a US launch site [A, repo]. Practical consequences:
- US-owned integrators (Voyager, SEOPS, Maverick, TriSept) avoid the services question. Foreign integrators (Exolaunch, ISISPACE, D-Orbit) require counsel's view on a TAA (budget 3-6 months).
- Electron SSO from NZ and Spectrum from Norway involve spacecraft export (BIS license or STA to A:5 countries) plus ITAR services (budget 6+ months).
- SpaceX can rebook or terminate with refund if a license is unobtainable (UG s9.2) [P].

---

## 5. Comparative Analysis

Legend: Price column is ROM for a 1U slot unless "quote" is stated. Grades after each.

| # | Provider / route | 1U? | 1U price (source, date) | Included | Orbit / altitude | Lead time and cadence | Docs / battery | 2025-26 example | Verdict for D7 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | SpaceX Transporter (direct) | No (50 kg min) | $375,000 min (2027, $7,500/kg) [P, API 2026-10-16]; est. $400k for 2028 [A] | Plate seat; separation systems extra | SSO, 500-600 km | 3-4 flights/yr; only 2027-06-01 and 2027-10-01 open to book [P] | UG v11 [P] | T-18, 2026-10-01, 130 payloads [P] | Use via integrator only |
| 2 | Exolaunch (EXOpod Nova/Air; EQT) | Yes (0.25U-16U) [P] | Not published; ROM $85-150k [U] | Mission mgmt, test (Berlin), logistics, integration, deployment [P] | Transporter SSO 500-600 km; Exo-1/-2 dedicated (orbit TBD); Electron; Ariane 6 | SpaceX contracts through 2028; Exo-1 NET late 2027 [P] | SpaceX UG + integrator ICD; ITAR services via counsel | T-17: KOSTKA (1U/3U disputed), MARMOTSat 3U [S] | **Lead RFQ** |
| 3 | ISISPACE / ISILaunch | Yes (QuadPack/DuoPack, 1U-16U) [P] | Not published; ROM $85-150k [U] | Launch mgmt, integration, campaign [P] | Transporter SSO; formerly PSLV, Vega | Inquiry-based; ISL56 on T-17 [P] | Same | ISL56 (T-17) [P] | **Price check** |
| 4 | SEOPS (US) | Unverified (Equalizer reported 1U-12U [S]) | Not published; LaunchLock fixed price [P] | Integration, mgmt | Transporter SSO; Waymaker-1 2028 (dedicated F9, orbit TBD) | Waymaker bookable from 2028; windows reserved up to 3 yr [P] | Same | T-16: 19 payloads; T-18: 18 SuperDoves [S/P] | **US-owned alternative RFQ**; confirm 1U |
| 5 | D-Orbit ION | Unverified | Not published | OTV carrier, in-orbit slots | SSO about 510 km (Nov 2025) / about 590 km (Jul 2026) | Next ION Feb 2027 [P] | Same | T-17 ION [S] | Ask about <= 500 km and 1U; low prior |
| 6 | Voyager NRCSD (ISS) | Yes (up to 6x1U) [P] | Not published; legacy about $85-90k [U] | Deployer, ISS transport, NASA safety coordination | 400-420 km, 51.6 deg | Deploy 1-3 months after berthing; ISS resupply cadence | IDD, JSC 20793 Rev D, Phase 0-III reviews | none found (gap) | **Backup**; life 0.3-1.1 yr [A] |
| 7 | Space BD / Mitsui (Kibo J-SSOD) | Yes, 1.33 kg/U cap [S] | Not published | JAXA service | 400-420 km | Several/yr | JAXA spec | RSP-03, Atsushi (1U), 2025-09 [S] | Mass cap too low; export to Japan |
| 8 | Maverick Space Systems | No (3U+) [P] | Not published | Integration, mgmt | Transporter SSO | n/a | Same | T-18 customer sats [P] | Not for 1U |
| 9 | TriSept | Not evidenced | Not published | Gov/DoD integration | Various | n/a | n/a | none | Not for this mission |
| 10 | Rocket Lab Electron | 1U needs third-party dispenser [P] | Not published; dedicated about $7.5M [U] | Rideshare plate, Kick Stage | 500 km SSO (LC-1, NZ); 38-60 deg from Wallops | 10 rideshares in 2025 [P] | PUG 8.0; export license for NZ | JAXA rideshare Apr 2026 (8 CubeSats) [S] | **Contingency RFQ** for altitude control |
| 11 | NASA CSLI | Yes | $0 launch to program; program-side about $30-70k [U] | NASA integration; CRADA | Varies | Next solicitation 2028 [P]; launch about 2030+ | LSP-REQ-317.01 | none for 2027-28 | **Contingency only** |
| 12 | Firefly Alpha | No CubeSat rideshare since 2024 [S] | n/a | n/a | n/a | Flight 8 undated | n/a | none | Not viable |
| 13 | ISRO/NSIL PSLV | n/a | n/a | n/a | n/a | Two failures, no RTF [S] | n/a | n/a | Not viable |
| 14 | Gilmour, Isar, RFA, Vega-C | No | n/a | n/a | n/a | Unproven or no manifest | n/a | n/a | Not viable |
| 15 | Spaceflight Inc. | Exited 2023 | n/a | n/a | n/a | n/a | n/a | n/a | Exited |

**Unpublished-price anchors for a 1U (all [S/U], none from a provider):**

| Anchor | Value | Date / quality |
|---|---|---|
| Demetrius Aerospace community post (libre.space, 2026-07-22) | $86,000 (1U; $30k deposit, $35k milestone, $21k final); 2U $172k; 3U $258k; windows 2027 Q3, Q4, 2028 Q1; vehicle unnamed | A single vendor/reseller post, not a provider; verify before paying anyone |
| Third-party guides (SpaceNexus, GreenLaunch) | $40-80k per 1U via a broker; 3U $100-250k | Undated, unsourced |
| Nanoracks legacy | about $85-90k per unit | Undated |
| Alba Orbital (PocketQube vendor) | "$100k+ for a 1U CubeSat" vs EUR 25k PocketQube | Competitor's comparison |
| Satnews 2026-07-20 | "sub-$300,000 baseline for standard CubeSat slots" | Almost certainly the SpaceX minimum misdescribed; ignore |

---

## 6. Practical Recommendations

### 6.1 Primary path (ROM total launch + integration)

**Transporter-class SSO rideshare through a 1U-capable integrator, deployment <= 500 km (hard limit 525 km), contract signed by 2027-06-30.**

| Cost element | ROM low | ROM high | Basis |
|---|---|---|---|
| Integrator 1U slot incl. standard integration | $85k | $150k | Anchors above; 2026-27 price escalation (+7%/yr on SpaceX list) and scarce 2028 capacity push toward the upper half [U] |
| Environmental test at provider levels (vibe with inhibit check, TVAC, bakeout) | $15k | $40k | Third-party lab or Exolaunch Berlin [U, no source] |
| Export counsel, classification, TAA if foreign integrator | $10k | $25k | [U] |
| Shipping, travel, customs, integration-campaign support | $5k | $15k | [U] |
| Third-party ephemeris / space-traffic service if no GNSS (D6) | $0 | $20k | [U] |
| **All-in** | **$115k** | **$250k** | Central case about $150-170k |

For the funder: **$115-250k all-in** (program-side $30-100k), of which $85-150k is the integrator's quote. Add a 15% contingency on the integrator line for rebooking or storage if the flight slips.

**Whom to ask (by 2026-12-31):** Exolaunch (lead; the most 2028 capacity), ISISPACE (price check), SEOPS (US-owned; confirm 1U and the Waymaker/LaunchLock terms), and one line each to D-Orbit and Maverick (1U and <= 500 km questions). Ask each for: written 1U price and payment schedule; the deployer type and 1U envelope (113.5 mm length, stowed wings); which flight, and the drop-off altitude list on that flight; the integrator's vibe levels and whether unit-level testing is accepted; late-access limits; ITAR path and US-person integration capability; ephemeris-service partner; demanifest and rebook terms.

**Contract language to protect the <= 500 km requirement [A]:**
- State the maximum acceptable drop-off altitude (500 km target, 525 km hard cap) and make it a condition of the flight assignment.
- If SpaceX's orbit-selection list for the assigned flight (due L-5 months) has no option at or below the cap, the customer may move to the next Transporter or Exo/Waymaker flight with no penalty and a credit.
- Obtain historical drop-off altitudes for the last 4-6 Transporter flights to calibrate the risk.

### 6.2 Backup path

**ISS deployment via Voyager NRCSD.** ROM **$120-250k**: $85-150k slot [U, legacy anchor] plus $35-100k program-side (JSC 20793 battery acceptance/cell screening, ground-leg inhibit design, safety data package, vibe, counsel). Pros: US integrator, lowest export burden, always <= 420 km so FCC disposal is trivial. Cons: 0.3-1.1 yr life [A], heaviest battery safety program, and ISS retirement planned for 2030. **Decision gate:** if no integrator gives a written <= 525 km commitment by 2027-06-30, or if costs exceed $250k all-in, trigger the NRCSD RFQ in parallel and decide by 2027-09.

**Second backup (altitude-controlled):** Electron rideshare via an integrator that can fly a 1U. Cost unknown (ROM $120-300k all-in [U, no data]); NZ launch adds a spacecraft export license and 6+ months. Only pursue if the primary path cannot give a <= 525 km commitment.

**Contingency (no cost to the program, long delay):** Prepare a CSLI-style package (eligible lead organization, build to LSP-REQ-317.01) and apply when the 2028 solicitation opens. Do not base the funding plan on it.

### 6.3 Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| No <= 525 km drop-off offered on the assigned flight | Medium (SpaceX posts 500-600 km) | High (FCC 5-yr case fails) | Altitude clause; multiple integrators; Electron fallback; DAS run early |
| 2028 capacity unavailable (reported SpaceX freeze beyond late 2028; Isar 2028 closed) | Medium [S] | High | Sign by 2027-06-30; use LaunchLock-style reservation; keep ISS option |
| Integrator quote above $150k | Medium | Medium | Competing RFQs; consider a ride with a lower-priced second-tier integrator; stage funding |
| Export/TAA delay | Medium | Medium | US-owned integrator, or start counsel in Q4 2026 |
| No GNSS vs SpaceX ephemeris rule (UG s9.5.7.2) | High | Medium | Decide D6; budget a third-party service |
| Integrator corporate change (Exolaunch/EQT) | Low-Medium | Low | Contract terms that survive change of control |
| Fraud or reseller risk with unvetted brokers (e.g. community-post offers) | Low-Medium | High | Deal only with providers on NASA's dispenser lists or that appear on SpaceX manifests; escrow/milestones |

### 6.4 Open items (could not close)
1. No provider publishes a 1U price; every number above is ROM until RFQ replies arrive.
2. SEOPS Equalizer 1U support, D-Orbit ION 1U acceptance and <= 500 km capability: unverified.
3. NRCSD IDD "Rev A (2024-01-19)" not located; only the 2018 initial release.
4. SpaceX booking freeze: unconfirmed by SpaceX; the sources are unnamed.
5. No verified 1.5U/2U Transporter or 2025-26 NRCSD 1U manifest example.
6. KOSTKA size (1U vs 3U) conflicting.
7. Exolaunch/EQT closing status; UNOOSA/Exolaunch free-slot eligibility.
8. Electron rideshare 1U route and price.

---

## 7. Technical Deep Dive: Evidence Excerpts and Arithmetic

### 7.1 SpaceX API values used (cents converted to dollars), queried 2026-10-16
```
GET https://api-rideshare.spacex.com/api/v2/price
pricingMenus:  2023 $5,500/kg | 2024 $6,000 | 2025 $6,500 | 2026 $7,000 | 2027 $7,500
min spend:     quarter plate 50 kg | half 100 | full 200 | XL 300
Separation system (8"): 2025 $306,150 | 2026 $336,800 | 2027 $370,500
Insurance add-on: value 5 in all years (units not documented)

GET https://api-rideshare.spacex.com/api/v1/flights
2027-06-01  flightType 1  SSO+/-0.1  perigee/apogee 500-600 km
2027-10-01  flightType 1  SSO+/-0.1  perigee/apogee 500-600 km
```

### 7.2 Arithmetic [A]
- 50 kg x $7,500 = $375,000 (2027). Extrapolated 2028: 50 kg x $8,000 = $400,000.
- 1.6 kg x $7,500 = $12,000 mass-equivalent; slot $85-150k is 7.1-12.5x that, i.e. an effective $53-94k/kg.
- A full 12U at about 24 kg of raw mass is about $180k at $7,500/kg, so a 12U slot priced near $250-600k (third-party range) carries a similar premium.

### 7.3 Orbit selection and CubeSat rules (UG v11, Aug 2026) [P]
- Appendix G: "Orbit Selection webform ... choose the orbital parameters ... from a list of available orbit parameters for the mission selected by SpaceX"; due L-5 months.
- s5.7: any door order; remain within the dispenser volume; no passive appendage deployment during ejection.
- s6.6/Table 6-2: fully containerized CubeSats may be unit-tested; random vibration required at no less than the MPE (5.57 Grms, Table 4-6); power-inhibit verified during vibe; protoflight preferred.
- s9.5.7.2: GNSS ephemerides with covariance within 24 h; integrators enforce; claim the object with 18 SDS within 30 days.
- s9.2: export law applies; SpaceX may rebook or terminate with refund if a license is unobtainable.

### 7.4 Dispenser capability summary (NASA SOA 2026 Table 10-2) [P]
Cal Poly 1U/3U tab; Dhruva DSOD 1U-16U; Exolaunch 0.25U-16U; ISISPACE 1U-16U; Maverick 3U-16U; NPC Spacemind SMPOD 1U-16U; Rocket Lab CSD/ASD 3U-12U; UARX RAMI 1U-12U; Voyager NRCSD 1U-6U; Xterra XCD 1U-16U.

---

## 8. References

**Primary (read this session)**
1. SpaceX rideshare booking API: https://api-rideshare.spacex.com/api/v2/price and /api/v1/flights (queried 2026-10-16). Front end: https://rideshare.spacex.com/
2. SpaceX Rideshare Payload User's Guide v11, Aug 2026: https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf (s1.2, 1.4, 4.1.6, 5.7, 6.6, 9.2, 9.5.7, App. G)
3. NASA State-of-the-Art Small Spacecraft Technology, Ch. 10, May 2026: https://www.nasa.gov/wp-content/uploads/2026/05/10-soa-launch-2026-final.pdf (Tables 10-1, 10-2; s10.5.1 ION, s10.6)
4. NASA CSLI apply page (updated 2026-10-14): https://www.nasa.gov/announcement-of-partnership-opportunity-for-cubesat-launch-initiative/ ; CSLI program page: https://www.nasa.gov/cubesat-launch-initiative/
5. Rocket Lab Electron Payload User Guide v8.0, Sept 2025: https://rocketlabcorp.com/assets/Rocket-Lab-Electron-Payload-User-Guide-8.0.pdf
6. Exolaunch brochure Q1 2026: https://exolaunch.com/media_assets/company_overview/Exolaunch_Brochure_Q1_2026.pdf ; mission page https://exolaunch.com/mission_48 ; news_127 https://exolaunch.com/news_127.html ; https://www.exolaunch.com/
7. EQT to acquire Exolaunch (2026-06-18): https://www.prnewswire.com/news-releases/eqt-to-acquire-exolaunch-a-germany-based-satellite-deployment-technology-and-launch-mission-management-firm-powering-global-access-to-space-302804185.html
8. ISISPACE launch services: https://isispace.nl/launch-equipment-services/ (isilaunch.com redirects here)
9. D-Orbit: https://dorbit.space/ (next launch Feb 2027; 26 missions)
10. Voyager NRCSD datasheet (ESA-hosted): https://bsgn.esa.int/wp-content/uploads/2025/06/Voyager-Portfolio-6.pdf ; Voyager Technologies: https://voyagertechnologies.com/
11. SEOPS: https://seops.space/ ; TriSept: https://www.trisept.com/ ; Maverick: https://www.maverickspace.com/
12. Libre Space Transporter-17 thread: https://community.libre.space/t/transporter-17-rideshare-vsfb-slc-4e-2026-07-07-07-10-utc/14950 ; Demetrius Aerospace post: https://community.libre.space/t/launch-rideshare-services-for-cubesat-pocketqube-teams/15074 (2026-07-22)
13. AMSAT ANS-222 (NASA letters of 2025-07-12): https://www.amsat.org/ans-222-amsat-news-service-bulletins/
14. Via Satellite, Transporter-18 (2026-10-01): https://www.satellitetoday.com/launch/2026/10/01/the-first-mission-milestones-onboard-spacexs-transporter-18-rideshare-launch/ ; SEOPS and Exolaunch dedicated Falcon 9 (2026-05-26): https://www.satellitetoday.com/launch/2026/05/26/rideshare-providers-seops-and-exolaunch-buy-dedicated-falcon-9-missions/
15. Repo: `docs/funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md` (SQSO-AUD-004), `docs/regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md` (SQSO-REG-012), `docs/BASELINE_PARAMETERS.md` (read only)

**Secondary (trade press, aggregators; read in full or via search summaries)**
16. SpaceQ, "SpaceX's rideshare freeze" (2026-08-06/07): https://spaceq.ca/spacexs-rideshare-freeze-what-it-means-for-the-smallsat-market-and-canada/
17. Satnews (2026-07-20): https://satnews.com/2026/07/20/spacex-limits-future-commercial-rideshare-bookings-past-2028-amid-severe-manifest-saturation/
18. Payload / Via Satellite on SEOPS-Isar (2026-09-15/16): https://payloadspace.com/seops-books-five-launches-on-isars-spectrum-rocket/ ; https://www.satellitetoday.com/launch/2026/09/16/seops-buys-five-launches-from-isar-aerospace/
19. AIAA Aerospace America year in review (CSLI 2024 APO no awards): https://aerospaceamerica.aiaa.org/year-in-review/small-satellite-missions-continue-to-grow-in-number-and-impact/
20. Spaceflight Inc. / Firefly: https://spacenews.com/firefly-aerospace-acquires-spaceflight-inc/ ; https://www.satellitetoday.com/finance/2023/06/12/firefly-aerospace-to-acquire-rideshare-provider-spaceflight/
21. PSLV-C62: https://en.wikipedia.org/wiki/PSLV-C62 ; Firefly Alpha RTF: https://aiaa.org/2026/03/12/firefly-alpha-returns-to-flight-with-successful-orbital-launch/ ; Gilmour: https://orbitcodex.com/operations/flight-two-eris-1
22. KOSTKA: https://en.wikipedia.org/wiki/KOSTKA ; https://www.fekt.vut.cz/en/faculty/news_new/345947
23. JAXA J-SSOD / Space BD: https://humans-in-space.jaxa.jp/kibouser/library/item/jaxa_j-ssod_01en.pdf.pdf
24. Third-party price guides (low reliability, undated): https://spacenexus.us/guide/cost-to-launch/cubesat ; https://greenlaunch.space/feeds/blog/cheapest-cubesat-launch-cost ; Alba Orbital https://www.albaorbital.com/launch

**Not verified / from memory:** Nanoracks-to-Voyager corporate chain (2021 acquisition), Virgin Orbit and ABL exits, ISS 2030 retirement status, NRCSD IDD Rev A (2024), JSC 20793 Rev D text, LSP-REQ-317.01 Rev B text (all carried from the repo study or memory).
