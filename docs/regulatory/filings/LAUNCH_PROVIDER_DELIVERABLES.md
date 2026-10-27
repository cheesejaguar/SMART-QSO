# Launch Provider Deliverables Package
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REG-016
**Revision:** E
**Date:** 2026-10-27
**Status:** Checklist. Every item is **open**; no hardware exists yet.
**Applies to:** the primary path of the [orbital access plan](../../ORBITAL_ACCESS.md) (SpaceX Transporter-class rideshare through Exolaunch, EXOpod Nova) and, where noted, the NRCSD backup. The dated deliverable schedule with sources is [SQSO-LVR-001 §9](../../mission/LAUNCH_VEHICLE_REQUIREMENTS.md); the requirement flow-down with verification is the rest of that document. The integrator's mission ICD **supersedes** this list where it differs.

## 0. SpaceX Statement-of-Work deliverables (Rideshare Payload User's Guide v11, Appendix G, Table G-3)

Delivered to the integrator (Exolaunch), which forwards them to SpaceX, unless the row says otherwise.

| Milestone | Deliverable | SMART-QSO source | Status |
|-----------|-------------|------------------|--------|
| Kickoff | Technical points of contact; TAA questionnaire or Export Compliance Agreement | [Export memo](EXPORT_CLASSIFICATION_MEMO.md); counsel | Open (contracting entity: RID LV-09) |
| L-8 M | Configuration webform (Appendix H constituent details); CAD (Appendix C); dynamic model and worksheet (Appendix B); mass properties ± 15 %; separation verification analysis; transmitter verification webform; electrical pinout webform (none: no LV interface); environmental test approach and schedule | MASS_BUDGET v3.1; CAD and FE model (`hardware/mechanical/MECHANICAL_DESIGN.md`, `docs/environment/STRUCTURAL_ANALYSIS.md`; SQSO-LVR-001 LVR-004, LVR-005); `software/simulation/cdr/launch/leop_feasibility.py` (ejection velocity, turn-on delay) | Open (CAD and FE model exist; webform at L-8 M) |
| L-6 M | Spacecraft Declaration of Hazardous Systems; SSCMAN 91-710 compliance letter; FAA payload review template; FAA cross-waivers | `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md` (skeleton) | Open |
| L-5 M | Orbit selection webform (≤ 485 km; target 475 km) | BASELINE §3 | Open |
| L-3.5 M | Mass properties ± 5 %; environmental verification report (SpaceX template), including test-like-you-fly exceptions and any "advised" tests not done (shock) | `LAUNCH_VEHICLE_REQUIREMENTS.md` §8 | Open |
| L-2 M | TPL additional insureds; payload insurer details and waivers of subrogation; badging; Ground Operating Plan and hazardous procedures (if any at the site); licence copies; Appendix F licensing certification letter (before shipment); launch-site training; 18th SDS coordination; as-measured mass | §3 below | Open |
| Launch campaign | Encapsulation readiness and launch readiness certificates (Appendix D, scenario A: no charging; SOC 60 % at integration) | `docs/operations/LAUNCH_SITE_OPS.md` | Open |
| Post launch | Payload operations status | `docs/operations/MISSION_TIMELINE.md` §5 | Open |
| Separation to cataloging (to Space-Track and SpaceX directly) | Forward-predicted GNSS ephemeris with covariance within 24 h of first contact, after every pass and at least every 24 h; object claimed within 30 days | [Space safety plan](SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md) §3a | Open: onboard GNSS chain and ground orbit determination implemented (RID LV-04 closed; SQSO-LVR-001 LVR-089); demonstration VI-LV-13 by L-2 M |
| Mission analysis | Bounding hard-body radius for appendage deployment in the first 24 h: **0.6 m** (UG §3.3.4) | `leop_feasibility.py` §7 | Analysis done |

---

## 1. CubeSat Design Specification Rev 14.1 — items the provider will verify

Source: Cal Poly *CubeSat Design Specification (1U–12U) Rev 14.1*, CP-CDS-R14.1 (Feb 2022). Requirement numbers are
CDS numbers.

| CDS ref | Requirement (abridged) | SMART-QSO evidence to deliver | Status |
|---------|------------------------|-------------------------------|--------|
| 2.1.1 | All parts remain attached during launch, ejection and operation | Design description; post-vibe inspection report | Open |
| 2.1.2 / 2.1.6 | Pyrotechnics and hazardous materials conform to AFSPCMAN 91-710 Vol. 3 | Statement: no pyrotechnics. Hazardous materials list (Li-ion cells) | Open |
| 2.1.5 (note) | Consider FAA airline battery guidance (≤ 100 Wh per battery) | Battery Wh at the BASELINE value (well under 100 Wh) | Open |
| 2.1.7 | Materials TML ≤ 1.0 %, CVCM ≤ 0.1 % (ASTM E595) | Materials list with outgassing data (`docs/compliance/MATERIALS_LIST.md`, to be updated with sources) | Open |
| 2.1.8 | Passive magnets ≤ 0.5 G above Earth's field outside the static envelope | Statement / measurement (magnetorquers are not passive magnets; check any magnetic latches) | Open |
| 2.1.9 | Ascent venting | Vent-path description | Open |
| 2.2.x | Envelope, rails (≥ 75 % rail contact, 2.2.9), mass (1U typical max **2.00 kg**, Table 1; **the dispenser's own limit governs**) | CubeSat Acceptance Checklist (CAC) measurements; mass-properties report (mass, CG, MOI) | Open |
| 2.3.4 / 2.3.5 | RBF pin cuts all power; ≤ 6.5 mm protrusion; within access port if one exists | Electrical schematic; inspection | Open |
| 2.3.6 | Battery charge/discharge protection; extra documentation for modified or non-UL cells | Battery data package (§2) | Open |
| 2.3.7 | **≥ 3 independent RF inhibits** (a timer is not an inhibit) | Inhibit diagram showing three physical inhibits between battery and RF transmitters | Open: designed, three deployment switches DS1–DS3 on three rails, DS2 the ground leg (BASELINE Rev E §4; `hardware/electrical/eps/EPS_BOARD_DESIGN.md` §6.2; RID LV-05); the inhibit-by-inhibit test VI-LV-06 is to run |
| 2.3.8 | **≥ 3 independent inhibits** to deployables (antenna, solar panels) | Same diagram for burn-wire circuits | Open |
| 2.4.1 | Documentation of radio licenses; for amateur frequencies, **proof of IARU coordination** (2.4.1.1) | IARU coordination letter + FCC pre-space notification filing record | Open |
| 2.4.3 | Comply with NPR 8715.6; any component re-enters with < 15 J (2.4.3.1) | ODAR with DAS output | Open |
| 2.4.4 | Deployables wait **≥ 30 min** after deployment-switch activation | Flight software parameter + test record | Open: `deployment.rs` waits 30 min (SIL first burn at 30.0 min, SQSO-LVR-001 LVR-064); test record VI-LV-07 |
| 2.4.5 | **No signal generated or transmitted earlier than 45 min** after on-orbit deployment | Flight software parameter + test record | Open |
| 2.4.6 (note) | NOAA remote sensing license if applicable | [NOAA determination](NOAA_REMOTE_SENSING_DETERMINATION.md) | Open |
| 2.4.7 | Fit check(s) with the dispenser or an engineering dispenser | Fit-check report (EXOpod Nova TestPod; Exolaunch rail measurement guide) | Open |
| UG §3.3.4 | No attitude maneuver and no appendage deployment before 120 s after separation | Flight software parameter and test record | Open: no magnetorquer output before 120 s (SRS-F086; RID LV-02 closed; SIL first coil command at 121 s); test record VI-LV-07 |
| UG Table 5-5 | Prohibited materials, including pure tin and tin electroplate, apply to containerized CubeSats | Parts finish review of every board | Open (RID LV-26) |

## 2. Battery Data Package

Providers, and for ISS deployment NASA JSC-20793, require more than CDS 2.3.6. Prepare:

1. Cell manufacturer, part number, chemistry, capacity and a UL/IEC listing certificate
2. Pack configuration (series/parallel), nominal Wh, protection circuitry schematic (over-charge, over-discharge, over-current, short-circuit, thermal)
3. Cell screening/lot acceptance records (OCV, capacity, internal resistance, visual inspection)
4. Pack acceptance test records (charge/discharge cycles, vibration, thermal, vacuum leak/mass loss where required)
5. Charging procedure and state-of-charge limits for storage, integration and launch
6. For ISS deployment only: JSC-20793 compliance evidence. This is a substantial, schedule-driving activity; see the launch trade

## 3. Regulatory Evidence (provider and FAA payload review)

| Item | Why | Source in repo |
|------|-----|----------------|
| IARU coordination letter | CDS 2.4.1.1 | (after F1 in the roadmap) |
| FCC pre-space notification as filed, plus FCC acknowledgment if issued | FAA favorable payload determination needs "all required licenses, authorizations, and permits" (14 CFR 450.43(a)(1)) | [FCC pre-space draft](FCC_97-207_PRE_SPACE_NOTIFICATION.md) |
| ODAR / debris disclosures | CDS 2.4.3; FCC 97.207(g)(1) | [ODAR](ODAR_SMART-QSO.md) |
| NOAA determination | CDS 2.4.6 | [NOAA determination](NOAA_REMOTE_SENSING_DETERMINATION.md) |
| Export classification statement (and license/TAA if a foreign integrator or site is used) | Provider contract terms; 15 CFR 774 9A515 related controls | [Export memo](EXPORT_CLASSIFICATION_MEMO.md) |
| Operator contact form (24 h), beacon frequency, call sign | 18 SDS pre-launch data; identification | [Space safety plan](SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md) |

## 4. Test and Analysis Reports

| Report | Typical requirement | Repo procedure |
|--------|---------------------|----------------|
| Random vibration (protoflight or acceptance per provider levels) with pre/post functional tests and sine signature | Provider ICD / user guide levels (GEVS GSFC-STD-7000B as fallback) | `docs/test_procedures/TP-ENV-002_Vibration.md` (levels to be re-baselined to the chosen provider) |
| Thermal vacuum / bakeout (if required by provider) | Provider ICD | `docs/test_procedures/TP-ENV-003_Thermal_Vacuum.md` |
| Shock (analysis or test) | Provider ICD | — |
| Deployment-switch, RBF and inhibit verification | CDS 2.3.5–2.3.8 | `docs/test_procedures/TP-MECH-003_Deployment_Switches.md` |
| 30-min deployable / 45-min RF timer verification | CDS 2.4.4–2.4.5 | `docs/test_procedures/TP-ENV-005_Deployment_Post_Env.md` |
| Mass properties | CDS Table 1 / dispenser limit | — (create) |
| Fit check | CDS 2.4.7 | `docs/test_procedures/TP-MECH-001_Fit_Check.md` |
| Structural analysis summary (margins, first mode vs. provider requirement) | Provider ICD | `docs/environment/` analyses (re-baseline to provider loads) |

## 5. Delivery Logistics

- Pack-out procedure with ESD protection and the battery state of charge for transport (by air ≤ 30 % SoC: UN3481 contained in equipment, PI 967, where the IATA DGR 2026 guidance recommends it and operator variations may require it; by road the 60 % integration state; then the final charge to 60 ± 5 % at the integrator; SQSO-BSA-001 §6); shipping container; humidity indicators; UN 38.3 test summary
- Pre-ship review sign-off (`docs/integration/PRE_SHIP_CHECKLIST.md`)
- Charging and access plan matched to the dispenser's access-port availability (CDS 2.3.4.1)

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial checklist (funding audit) |
| B | 2026-10-27 | CDR launch review: §0 SpaceX Statement-of-Work deliverables from UG v11 Appendix G with sources and status; the 120 s rule, prohibited materials, hard-body radius and ephemeris rows; inhibit row restated (no schematic); shipping state of charge; the dated schedule is SQSO-LVR-001 §9 |
| C | 2026-10-27 | CDR 2026 consistency pass: orbit selection ≤ 485 km (BASELINE Rev E §3) |
| D | 2026-10-27 | CDR design (EPS board): pack-out state of charge aligned with SQSO-BSA-001 D §6 (≤ 30 % by air, PI 967) |
| E | 2026-10-27 | Documentation audit (post-CDR 2026): §0 L-8 M sources (MASS_BUDGET v3.1, CAD and FE model exist; was "no CAD, no FEM") and the ephemeris row (LV-04 closed); §1 the three-switch inhibits (2.3.7), the 30-minute wait (2.4.4) and the 120 s magnetorquer hold (LV-02 closed; was "start at 1 s") |
