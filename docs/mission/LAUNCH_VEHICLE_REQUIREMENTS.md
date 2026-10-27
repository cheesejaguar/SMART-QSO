# Launch Vehicle and Dispenser Requirements Flow-Down

**Document ID**: SQSO-LVR-001
**Version**: A
**Date**: 2026-10-27
**Status**: Draft for CDR. No launch contract exists: every requirement below is flowed from the published provider
documents, not from an agreed mission ICD. The mission ICD will supersede it where it differs.
**Owner**: Launch, mission integration and operations
**Governs with**: `docs/BASELINE_PARAMETERS.md` (Rev D) §3–§4 (orbit, launch, physical); `docs/ORBITAL_ACCESS.md` (the
launch path, contract chain and decision points); `docs/compliance/LAUNCH_PROVIDER_ICD.md` (SQSO-ICD-010, the
interface values); `docs/compliance/CDS_COMPLIANCE_MATRIX.md` (SQSO-CDS-001, the CDS clauses one by one)

---

## 1. Purpose and sources

This document lists every requirement the launch vehicle, the rideshare integrator and the dispenser place on the
SMART-QSO spacecraft, with its source paragraph, the design feature that meets it, the verification method and the
status. It is the requirement half of the launch interface; the launch ICD (SQSO-ICD-010) holds the values.

| Ref | Document | Revision read | How obtained |
|-----|----------|---------------|--------------|
| UG | SpaceX *Rideshare Payload User's Guide* | **Version 11, August 2026** (the current revision on 2026-10-27; the cover reads "Version 11 - August 2026") | Downloaded 2026-10-27 from `https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf` (111 pages) |
| CDS | Cal Poly *CubeSat Design Specification* (1U–12U) | Rev 14.1, 2022-02-09 | Downloaded 2026-10-27 from `https://www.cubesat.org/s/CDS-REV14_1-2022-02-09.pdf` |
| NOVA | Exolaunch *EXOpod Nova User Manual* | Revision 1.2, June 2024 | `https://www.exolaunch.com/documents/EXOpod_Nova_User_Manual_June_2024.pdf` |
| EXO | Exolaunch *EXOpod User Manual* | Revision 6.0, 2022-09-06 | `https://www.exolaunch.com/documents/EXOpod_User_Manual_September_2022.pdf` (the 1U adapter and spring data Nova does not publish) |
| AIR | Exolaunch *EXOpod AIR 12UXL User Manual* | March 2026 | `https://www.exolaunch.com/documents/EXOpod_AIR_12UXL_User_Manual_March_2026.pdf` (read for the current product line only) |
| IDD | Nanoracks *CubeSat Deployer (NRCSD) Interface Definition Document*, NR-NRCSD-S0003 | Rev "-" (initial release, 2018-06-04) | NASA SSRI knowledge base. **The newer revision the funding audit cites ("Rev A, 2024-01-19") could not be found on the web; Voyager must supply the current one.** Backup path only |
| 91-710 | Space Systems Command Manual 91-710, *Range Safety User Requirements* | Current edition, as tailored by SpaceX for the mission | Not read; SpaceX supplies the tailored version and the templates (UG §11.1) |
| JSC 20793 | *Crewed Space Vehicle Battery Safety Requirements* | Rev D (as cited by the repository) | Not read; backup path only |

Verification methods: **A** analysis, **T** test, **I** inspection, **D** demonstration, **R** review of design or
documents. Status words: *Met (design)*: the design as documented meets it, verification still to run;
*Met (analysis)*: shown by an analysis named in the row; *Open*: the design does not yet contain what is needed;
*Not met*: the design conflicts with it; *VI-LV-nn*: a defined verification item (§8).

The primary path is a SpaceX Transporter-class rideshare from Vandenberg in an Exolaunch EXOpod Nova (decision D7;
`docs/ORBITAL_ACCESS.md` §2). Where the integrator's mission ICD is not yet available, the Exolaunch manuals stand in.
Requirements of the ISS backup (NRCSD) that differ are in §7.

## 2. Contract chain, schedule and documentation

| ID | Requirement | Source | SMART-QSO design or plan | Method | Status |
|----|-------------|--------|--------------------------|--------|--------|
| LVR-001 | A CubeSat flies only inside an Integrated CubeSat Dispenser Assembly with one mechanical interface to the plate; the integrator owns that assembly and its ICD with SpaceX | UG §1.2, §2.6.3 | Buy a slot from a 1U-capable integrator (Exolaunch lead) | R | Open: RFQs due 2026-12-31 (D7) |
| LVR-002 | The integrator's customer signs a launch services agreement and the mission ICD; SpaceX's ICD is signed by the integrator | UG §1.2, §9.4.1 | Contracting party: **not yet named** (BASELINE §1 has no host organization); see RID LV-09 | R | Open |
| LVR-003 | Customer deliverables and due dates per the SOW; typical: mission analysis inputs L-8 M, range safety submissions L-6/L-4/L-2 M, orbit selection L-5 M, final mission analysis inputs L-3.5 M, launch campaign readiness L-2 M | UG Fig. 1-2, Appendix G Tables G-1 to G-3 | Deliverable schedule §6 | R | Open (schedule exists in §6) |
| LVR-004 | Payload configuration webform, CAD model (Appendix C), dynamic model and worksheet (Appendix B) at kickoff / L-8 M | UG Table G-3 | CAD and dynamic model do not exist (no FEM; TB-15) | A, R | Open |
| LVR-005 | Mass properties (mass, CG, moments of inertia) within ±15 % at L-8 M, within ±5 % (flyaway < 10 kg) at L-3.5 M, as-measured mass at L-2 M; updates stay inside the earlier bounds | UG Table G-3 | MEL `docs/environment/MASS_BUDGET.md` v2.0: CBE 1.69 kg, predicted 1.92–2.00 kg; CG and MOI not computed (§5–§6 there). Report 1.85 kg ± 15 % (1.57–2.13 kg) at L-8 M so that later values stay inside | A, T | Open; VI-LV-02 |
| LVR-006 | Payload transmitter verification webform: turn-on times compatible with LV restrictions | UG §4.1.7.3, Table G-3 | First RF 45 min after separation (`rf_silence.rs`) against a minimum of 7.5–8.6 s (§4, LVR-061) | A | Met (analysis) |
| LVR-007 | Environmental test approach and schedule, with any deviation, approved by SpaceX before testing; verification report at L-3.5 M on the SpaceX template | UG §6.4, Table G-3 | Protoflight test campaign §8; TP-ENV-002 levels from UG v11 | R | Open |
| LVR-008 | Spacecraft Declaration of Hazardous Systems and SSCMAN 91-710 compliance letter at L-6 M; Ground Operating Plan and hazardous procedures at L-2 M (91-710 Vol. 6 Att. 2) | UG §9.5.3, §11, Table G-3 | Hazardous systems: Li-ion battery (≈ 50 Wh), burn-wire release circuits, RF transmitter (inhibited). System safety data package is a skeleton (`docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md`) | R | Open |
| LVR-009 | FAA payload review information template and signed FAA cross-waivers (required by federal law; late submission may mean rebooking) | UG Table G-3; 14 CFR 450.43, 440.17 | Contracting party signs (LV-09) | R | Open |
| LVR-010 | Payload licensing certification letter (Appendix F) before shipment; copies of all on-orbit licences at L-2 M | UG §9.5.6, Appendix F, Table G-3 | FCC Part 97 pre-space notification (due ≥ 90 days before integration into the LV, `docs/regulatory/filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md`); IARU coordination letter (CDS §2.4.1.1); NOAA determination | R | Open |
| LVR-011 | Third-party-liability additional insureds; payload insurer details and waivers of subrogation | UG Appendix G Tables G-1, G-3 | No payload insurance planned (self-insured); the waivers and the declaration are still required. Confirm with the integrator | R | Open |
| LVR-012 | Export: TAA questionnaire or Export Compliance Agreement at kickoff; customer complies with ITAR/EAR and customs; SpaceX may rebook or terminate if a licence is unobtainable | UG §9.2, Table G-3 | Presumptive ECCN 9A515.a.5 (`docs/regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md`); a foreign integrator (Exolaunch, Germany) needs counsel's view on defense services, 3–6 months | R | Open |
| LVR-013 | Integration readiness letters: encapsulation readiness and launch readiness certificates (Appendix D), stating the battery charge date and that the payload stays flight-ready for 45 days after encapsulation (scenario A, no charging) | UG Appendix D | Scenario A: no charging after integration. The state of charge at integration is 60 % ± 5 %, not "full" (LVR-051); the integrator must accept that wording | R | Open |
| LVR-014 | Daily processing schedule at the launch site; 72 h notice for hazardous operations; customer PPE, cleaning of customer equipment | UG §9.5.2 | No hazardous operation planned at the launch site (no charging there); the integrator handles the dispenser | R | Met (design) |
| LVR-015 | Post-launch payload operations status | UG Table G-3 | First-contact report (MISSION_TIMELINE §6) | D | Met (design) |

## 3. Mechanical: envelope, rails, mass, CG, MOI

| ID | Requirement | Source | SMART-QSO design | Method | Status |
|----|-------------|--------|------------------|--------|--------|
| LVR-020 | 1U envelope and rails per CDS Appendix B: 100.0 × 100.0 mm rails (±0.1), 113.5 mm rail length | CDS §2.2.1.1; NOVA Table 1 (Z ±0.5 mm, X/Y ±0.1 mm) | Bought 1U frame (ISIS, Pumpkin or Spacemanic, BUILD_VS_BUY §1) with CDS rails | I | Met (design); VI-LV-01 |
| LVR-021 | No protrusion beyond 6.5 mm from the rail plane on the shaded faces | CDS §2.2.3 (EXOpod Nova allows up to 25.0 mm from the rail, NOVA Table 1) | Stowed AntS (7 mm module on a ±Z face), stowed wings, RBF ≤ 6.5 mm; the design keeps the CDS value so that a P-POD-class or NRCSD dispenser stays possible | I | Met (design); VI-LV-01 |
| LVR-022 | Deployables constrained by the CubeSat, not by the dispenser; no passive deployment of appendages during ejection | CDS §2.2.4; UG §5.7; IDD §4.1.5 | Burn-wire hold-downs on the wings and the AntS release; nothing is spring-released by the dispenser door | I, T | Met (design) |
| LVR-023 | Rails ≥ 8.5 mm wide to the first protrusion (EXOpod: ≥ 6.4 mm); Ra ≤ 1.6 µm; edges rounded ≥ 1 mm (should); ±Z rail ends ≥ 6.5 × 6.5 mm contact; ≥ 75 % of the rail in contact | CDS §2.2.5–§2.2.9; EXO §2.3 | Bought frame | I | Met (design); VI-LV-01 |
| LVR-024 | Rails and contact surfaces hard anodized (Type III); any other finish approved in writing by Exolaunch, also for engineering models used in a Nova or TestPod | CDS §2.2.13; NOVA §2.3 | Bought frame, hard-anodized rails; the EM must also have them if it goes in a TestPod | I | Met (design) |
| LVR-025 | Mass ≤ the dispenser's 1U limit: CDS typical 2.00 kg (a note, not a requirement); EXOpod Nova 1U **2.5 kg** (qualified for Falcon 9) | CDS §2.2.10, Table 1; NOVA Table 1 | Predicted 1.92–2.00 kg (MASS_BUDGET v2.0). Margin: 0–4 % on 2.00 kg, **20–23 %** on the Nova's 2.5 kg | A, T | Met (analysis) against the Nova; at the limit against the CDS note. Proposal to the chair: RID LV-17 |
| LVR-026 | CG within ±2 cm of the geometric centre on each axis (Nova: ±20 mm, a guideline) | CDS §2.2.11, Table 2; NOVA Table 2 | Not computed (no layout; MASS_BUDGET §5). Battery (249 g) and Jetson with spreader (245 g) must be placed for it | A, T | Open; VI-LV-02 |
| LVR-027 | Moments and products of inertia reported (webform) | UG Table G-3 | Box estimate only: Ixx = Iyy ≈ 0.0032, Izz ≈ 0.0028 kg·m² stowed (MASS_BUDGET §6) | A, T | Open; VI-LV-02 |
| LVR-028 | Separation mechanism (springs) if the CubeSat shares a slot; max force < 6.7 N, stroke > 2.5 mm (guideline); not beyond the standoff when stowed | CDS §2.2.14; EXO §4 (1U and 2U CubeSats always share a 3U slot, with other CubeSats or a non-deployable adapter) | Four spring plungers in the -Z standoffs (CDS matrix §3.3); force from the frame vendor | I, T | Met (design); force open (TBR-CDS-001) |
| LVR-029 | Physical fit check with the dispenser or a TestPod; rail measurement on the assembled satellite; virtual fit check | CDS §2.4.7; NOVA §2.3.1 | Rail measurement before environmental test; TestPod fit during the vibration test | I | VI-LV-01 |
| LVR-030 | Tip-off: CubeSats tolerate up to 10°/s (EXOpod Nova expected below 10°/s; half-cone ±7.5°) | NOVA §2.5 | B-dot detumble from 10°/s (BASELINE §8); SIL detumbles from 10°/s in 0.8 h; Monte Carlo 40/40 trials (SIL-13) | A | Met (analysis) |
| LVR-031 | Deployment velocity: containerized CubeSats may exceed 1.0 m/s | UG §3.3.2; EXO Eq. 1, Tables 3–4 | 1.10–1.48 m/s for 1.51–2.00 kg in a 1U slot with its adapter, spring energy 2.19 J ± 20 % (`leop_feasibility.py` §3). The Nova's own value is given in the mission ICD | A | Met (analysis) |

## 4. Environments and loads

All levels are the SpaceX maximum predicted environments at the plate interface. The integrator may give
dispenser-derived levels; until then the SpaceX levels apply directly (NOVA §3.5: CubeSats are commonly designed and
tested to the launcher's levels).

| ID | Requirement | Source | SMART-QSO design | Method | Status |
|----|-------------|--------|------------------|--------|--------|
| LVR-040 | No elastic natural frequency below 40 Hz; Q between 10 and 50. Verified by low-level sine sweep (≥ 0.1 g, ≤ 3 oct/min, three axes) before and after full level: all modes > 40 Hz, < 10 % shift of the first mode, < 30 % shift of its amplitude; investigate > 5 % | UG §4.1.1, §6.7.1, Table 6-3 | Bought frame; no FEM yet (TB-15). A stiff 1U is typically far above 40 Hz; the wing hold-down and AntS are the soft items | A, T | Open (no FEM); VI-LV-03 |
| LVR-041 | Quasi-static: 10 g axial, 17 g lateral (RSS) for the dispenser assembly ≥ 20 kg; no CubeSat test required when containerized | UG §4.1.2.1, Table 4-1, Table 6-2 | Design the structure, battery mount and Jetson spreader to 10 g / 17 g with factors 1.10 yield, 1.25 ultimate | A | Open (no FEM) |
| LVR-042 | Factors of safety: yield 1.10, joint gap and slip 1.10, ultimate 1.25 flight, 1.40 ground | UG §5.1, Table 5-1 | Structural analysis to use them | A | Open |
| LVR-043 | Sine: 1.4 g axial, 1.5 g lateral, 5–100 Hz (MPE); not required for containerized CubeSats | UG §4.1.3, Table 4-2, Table 6-2 | Not tested (allowed) | — | Met (not required) |
| LVR-044 | Random vibration, **required**: protoflight at MPE (5.57 Grms, 20–2000 Hz), 1 min per axis, 3 axes; qualification unit MPE + 3 dB (7.87 Grms), 2 min; notching only to the quasi-static load, primary mode only, < 10 dB | UG §4.1.6, Table 4-6, §6.7.6, Table 6-2 | Protoflight on the flight unit in a TestPod (`docs/test_procedures/TP-ENV-002_Vibration.md`, interim UG v11 levels). Grms recomputed from the breakpoints: 5.57 and 7.87 (`leop_feasibility.py` §5) | T | VI-LV-04 |
| LVR-045 | Power-inhibit verification **during** random vibration: battery in its flight charge state, RBF and GSE inhibits removed, switches held as in flight; evidence (boot log, oscilloscope) of no inadvertent power-up and isolated systems staying isolated; the inhibition must come from the mechanical separation signal, not a software delay | UG §6.1.2, §6.7.7, Table 6-2 note 2 | Test with the flight battery at 60 % SOC, RBF out, switches actuated by the TestPod; monitor the bus, burn lines and PA rail | T | VI-LV-04 |
| LVR-046 | Shock: MPE 30 g at 100 Hz to 1000 g at 1000–10,000 Hz (Q = 10); advised only (protoflight MPE + 3 dB, 2 actuations per axis) | UG §4.1.5, Table 4-5, Table 6-2 | Not tested; the risk acknowledgement goes in the summary report (UG §6.4). The Nova damps shock at high frequency (NOVA §3.5) | A | Met (acknowledged risk) |
| LVR-047 | Acoustic: OASPL 139.3 dB; not required for containerized CubeSats | UG §4.1.4, Table 6-2 | Not tested | — | Met (not required) |
| LVR-048 | Ascent venting: volume / vent area ≤ 50.8 m (CDS, the stricter) and ≤ 400 m (UG); fairing decay ≤ 2.8 kPa/s (4.5 kPa/s for ≤ 5 s) | CDS §2.1.9; UG §4.1.8, §6.7.9 | Open frame; the 1U gross volume (1.135 L) needs ≥ 22.3 mm² of vent area for the CDS ratio, and every enclosed sub-volume (battery box, Jetson enclosure) must meet it alone (`leop_feasibility.py` §4) | A | VI-LV-12 |
| LVR-049 | Thermal, flight: fairing radiative temperature profile (ε ≈ 0.9); free-molecular heating < 3,500 W/m² at fairing separation; conductive boundary −20 to +69 °C up to 7200 s | UG §4.1.9–§4.1.11, Table 4-11 | The spacecraft is unpowered; components must survive the boundary; TVAC advised | A, T | VI-LV-08 |
| LVR-050 | Thermal vacuum and thermal cycling, advised: protoflight acceptance ± 5 °C, 20 cycles (≥ 4 in vacuum); acceptance minimum range −24 to +61 °C | UG §6.7.10, Table 6-2 | `docs/test_procedures/TP-ENV-003_Thermal_Vacuum.md` (re-baseline the levels) | T | VI-LV-08 |
| LVR-051 | Ground environment: PPF 15–25 °C, 30–65 % RH, ISO Class 8, ≤ 45 days; encapsulated in hangar ≤ 40 days; pad 15–30 °C, 0–65 % RH, ≤ 5 days, GN₂ purge in countdown; about 90 days in all; conditioned air off ≤ 60 min | UG §4.3, Table 4-12 | Battery stored at 60 % ± 5 % SOC; storage drain analysed for 15–30 °C (§5, LVR-071) | A | Met (analysis) |
| LVR-052 | Transportation environments are enveloped by flight | UG §4.2 | — | — | Met |
| LVR-053 | EMI, susceptibility: launch vehicle 120 dBµV/m (1–18 GHz, 140 dBµV/m in the S-band telemetry bands); launch site 132–150 dBµV/m with 26 dB shielding assumed in the PPF | UG §4.1.7.1, Tables 4-7, 4-8 | The spacecraft is unpowered with mechanical inhibits in the power path, so RF cannot activate it (CDS §2.3.1) | A, T | Met (design); VI-LV-06 |
| LVR-054 | EMI, emissions while integrated (Transporter): ≤ 108 dBµV/m 30–1000 MHz by test (102 by analysis), 33 dBµV/m in 1127–1327 and 1475–1675 MHz (GNSS bands) | UG §4.1.7.2, Table 4-9 | Unpowered while integrated: no emission source. EMC test advised only | A | Met (design) |

## 5. Electrical: inhibits, RBF, timers, battery, RF

| ID | Requirement | Source | SMART-QSO design | Method | Status |
|----|-------------|--------|------------------|--------|--------|
| LVR-060 | Power off from delivery to the LV through on-orbit deployment; powered battery protection allowed; all payload systems including batteries off during launch except the circuits that sense inhibits | CDS §2.3.1, §2.3.1.2; UG §7.6 | Separation switches in the battery path; the BMS stays powered on the battery side | T | Met (design); VI-LV-06 |
| LVR-061 | No transmitter use while integrated; minimum turn-on delay after separation per Table 4-10 (≈ 6 s at 1 W EIRP and 1.0 m/s) | UG §4.1.7.3, Table 4-10, §9.5.2 | 45-minute RF silence with the whole radio unpowered. Maximum EIRP ≈ 1.6 W; interpolated minimum 7.5–8.6 s at 1.0–1.2 m/s, 16 s at 0.5 m/s (`leop_feasibility.py` §3). Margin > 2,600 s | A, T | Met (analysis); VI-LV-07 |
| LVR-062 | No open-air RF checkout during integration; RF only after separation | UG §9.5.2 | Conducted RF tests only, into a load, before integration | D | Met (procedure: `docs/operations/LAUNCH_SITE_OPS.md`) |
| LVR-063 | No signal generated or transmitted earlier than 45 min after deployment | CDS §2.4.5 | `software/rust/sqso-fsw/src/rf_silence.rs`; SIL: first RF at 45.0 min | T | VI-LV-07 |
| LVR-064 | Deployables wait ≥ 30 min after the deployment switches are activated | CDS §2.4.4 | `software/rust/sqso-fsw/src/deployment.rs`; SIL: first burn at 30.0 min | T | VI-LV-07 |
| LVR-065 | **Delay attitude maneuvers and appendage deployment ≥ 120 s after separation**; report a bounding hard-body radius for any appendage deployment in the first 24 h | UG §3.3.4 | Appendages at 30 min: compliant. **Magnetorquers are driven from 1 s after separation** (SIL `console.log`: "[ADCS] Starting detumble sequence" at 1.0 s; `sqso-ffi/src/executive.rs` `run_adcs`): not compliant if the integrator counts magnetic torquing as an attitude maneuver. Hard-body radius: report **0.6 m** (VHF tip 0.55 m from the centre, `leop_feasibility.py` §7) | A, T | **Not met** (RID LV-02); hard-body radius met (analysis) |
| LVR-066 | Deployment switch toggle resets the satellite to the pre-launch state, including the transmission and deployable timers | CDS §2.3.2.4 | Power-on reset restarts both counts; a stowed reading while powered restarts both (`rf_silence.rs` tick; `deployment.rs`) | T | Met (design); VI-LV-07 |
| LVR-067 | At least one deployment switch, actuated at all times while integrated, disconnecting the power system from the powered functions, at or below the interfacing surface | CDS §2.3.2–§2.3.2.3 | Two rail-end switches in the -Z standoffs; contact form open (TBR-CDS-005) | I, T | Open (schematic); VI-LV-05 |
| LVR-068 | RBF pin cuts all power; ≤ 6.5 mm protrusion; within an access port if the dispenser has one, else removed before integration | CDS §2.3.4, §2.3.5; NOVA §4.2.5 (access windows on three sides; positions in NOVA Fig. 37) | RBF on a side face; its position against the Nova windows is open. Plan: RBF out at integration, before the door closes | I | Open (position); VI-LV-01 |
| LVR-069 | ≥ 3 independent RF inhibits and ≥ 3 independent inhibits for each deployable; a timer is not an inhibit | CDS §2.3.7, §2.3.8 | **No inhibit schematic** (VV-03). Two switches plus RBF documented; RBF and timers do not count (BASELINE §4) | T | **Open** (RID LV-05); VI-LV-06 |
| LVR-070 | Battery protection for charge and discharge (balance); extra documentation for non-UL cells | CDS §2.3.6 | 4S BMS (BQ76920 class), INR18650-35E cells (UL-listed); `hardware/eps/BATTERY_DESIGN.md` | R, T | Met (design); VI-LV-11 |
| LVR-071 | Battery flight-ready ≥ 45 days after encapsulation with no charging (Appendix D scenario A); the spacecraft must also survive the integrator's storage and launch slips | UG Appendix D, §7.6 | Integration SOC 60 % ± 5 %; separation SOC floor 40 %. Self-discharge model of the SIL (1.5 %/month at 25 °C, doubling per 10 °C) plus the battery-side drain: at 20 °C, 18.9 months to 40 % with no drain, 15.7 months at 10 µA, 9.4 months at 50 µA, 6.3 months at 100 µA (`leop_feasibility.py` §1). Requirement set here: **battery-side drain ≤ 20 µA with all inhibits open** | A, T | Met (analysis) once the drain is measured: VI-LV-14 |
| LVR-072 | Batteries are a hazardous system to the range (91-710); hazardous materials declared against 49 CFR 172.101 | UG §11.2, Appendix F item 2 | Li-ion 4S1P ≈ 50 Wh; UN 38.3 test summary for transport | R | Open |
| LVR-073 | Sealed containers that may pressurize on orbit (batteries) meet the design and test factors of Table 5-6, or a vacuum test | UG §5.5.4 | 18650 cells are sealed steel cans with a vent; battery TVAC exposure in the cell screening (BATTERY_DESIGN §9) | T | Open (confirm with the integrator) |
| LVR-074 | No electrical interface to the dispenser or LV after integration (the Nova has no standard umbilical; access only through the windows) | NOVA §5.4; UG §7.6, Table 7-4 | No umbilical after integration; key load and ARM_FLIGHT are the last umbilical operations (SIL integration phase) | D | Met (design) |
| LVR-075 | Downlink frequency and power licensed by the customer; SpaceX licenses only its own emissions | UG §9.6.8 | FCC Part 97 (KK6GDB-11), IARU coordination | R | Open |

## 6. Materials, contamination, space safety, debris

| ID | Requirement | Source | SMART-QSO design | Method | Status |
|----|-------------|--------|------------------|--------|--------|
| LVR-080 | Non-metallic materials: TML ≤ 1.0 % and CVCM ≤ 0.1 % (CDS); UG uses RML ≤ 1.0 % with rationale codes. Fully containerized CubeSats are exempt from the UG visual-cleanliness, non-metallic, metallic and silicone rows, **not from the prohibited-materials row** | CDS §2.1.7; UG §5.3, Table 5-5, §6.7.12, Table 6-7 | `docs/compliance/MATERIALS_LIST.md` (needs sources); no markers (Sharpie prohibited) | I, R | VI-LV-09 |
| LVR-081 | Prohibited: cadmium and cadmium plating, zinc plating, mercury, **pure tin or tin electroplate** (unless alloyed with lead, antimony or bismuth) | UG Table 5-5 | Not yet applied to the EEE parts lists. Many COTS parts (STM32, the Jetson module, connectors) have matte-tin finishes | R | **Open** (RID LV-26) |
| LVR-082 | Contamination report with the complete vacuum-exposed non-metallic materials list and quantities | UG §6.7.12 | To be prepared from the materials list | R | Open |
| LVR-083 | No particulate generation during ascent; no uncontained pyrotechnics | UG Table 5-5; CDS §2.1.2 | Burn wires (thermal knife), no pyrotechnics; the release is on orbit at 30 min | R | Met (design) |
| LVR-084 | Thermal vacuum bakeout as specified by the launch provider | CDS §3.2 | Bakeout in the TVAC campaign | T | VI-LV-08 |
| LVR-085 | Passive magnets ≤ 0.5 G above Earth's field outside the static envelope | CDS §2.1.8 | No passive magnets planned; the magnetorquer rods are actuators (unpowered at launch); check rod remanence | T | VI-LV-18 |
| LVR-086 | All parts remain attached during launch, ejection and operation; no detachable parts | CDS §2.1.1; IDD §4.4.6 | Burn-wire line remnants retained in the hold-down | I | Met (design) |
| LVR-087 | Disposal no later than 5 years after the mission ends (FCC); comply with NPR 8715.6; components re-enter with < 15 J | UG §1.2; CDS §2.4.3, §2.4.3.1 | Passive decay with wings deployed: 3.1 yr post-mission at 500 km nominal (BASELINE §3); 525 km cap; ODAR with DAS open | A | Met (analysis, in-house tool); DAS run open |
| LVR-088 | Register all deployed objects with the 18th SDS no later than L-2 M, with the operator contact made public for the mission lifetime; if any orbit crosses the ISS altitude, coordinate with NASA (jsc-dl-topo-iwg@mail.nasa.gov) | UG §1.2, §9.5.7.1 | `docs/regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md` R1, R5 (the decay crosses the ISS altitude) | R | Open |
| LVR-089 | **Forward-predicted GPS/GNSS ephemeris with covariance to Space-Track and SpaceX Space Traffic Coordination within 24 h of first contact, after every pass and at least every 24 h until cataloged**; a spacecraft without GNSS arranges a third party before integration; integrators enforce, and failure to demonstrate may mean rebooking | UG §9.5.7.2 | GNSS receiver (D6). **The flight software does not run the GNSS**: the NMEA parser `sqso-fsw/src/gnss.rs` is not called by the executive, there is no receiver power schedule, no fix in telemetry and no ground orbit-determination pipeline. First ephemeris due 24.7–36.2 h after separation (first contact 0.7–12.2 h, `leop_passes.py`) | D | **Open** (RID LV-04); VI-LV-13 |
| LVR-090 | Claim the object with the 18th SDS within 30 days of deployment | UG §9.5.7.2 | Identification by beacon Doppler (SQSO-REG-015 §2) | D | Open |
| LVR-091 | Best practice: first ephemeris within 3 h of deployment; three a day | UG §9.5.7.5 | Not achievable with Bay Area stations only (median first pass 5.2 h); possible with a high-latitude second station or a SatNOGS-relayed fix | — | Not adopted |
| LVR-092 | No propulsive maneuvers or secondary deployments within 7 days | UG §3.3.4 | No propulsion | — | Met |

## 7. Backup path: what changes on the ISS route (NRCSD)

Read against the 2018 IDD (the only revision found). Rows marked **Δ** are design changes from the primary path.

| ID | Requirement | Source | Primary-path design | Delta |
|----|-------------|--------|---------------------|-------|
| LVR-100 | Mass ≤ 2.40 kg (1U); CG ±2 cm | IDD §4.1.2, Table 4.1.2-1 | Predicted 1.92–2.00 kg: 16.7–20 % margin | None |
| LVR-101 | **≥ 3 deployment switches**, each an independent inhibit on the main power system, ≤ 3 N each, ≤ 9 N total, captive | IDD §4.1.4 | Two switches | **Δ** third switch. Recommended for the primary as well (LV-05) |
| LVR-102 | **Ground-leg inhibit** between the battery negative and ground | IDD §4.2.1(3) | Not in the design | **Δ** |
| LVR-103 | **No system operating and the main power system not energized for ≥ 30 min** after deployment (timer with fault tolerance) | IDD §4.2.1(2) | The OBC boots at separation (CDS §2.4.5.1 allows it) | **Δ** a hardware delay before the bus is energized, or an exception in the ICA |
| LVR-104 | Switch cycling resets the pre-launch state within the first 30 min after the switches close | IDD §4.1.4(5) | Resets at any time (LVR-066) | None (stricter) |
| LVR-105 | Rail edge radius **0.5 ± 0.1 mm**; rail width ≥ 6 mm; ±Z rail ends bare 6 × 6 mm; rails extend ≥ 2 mm beyond the ±Z faces; rails the only contact; hardness ≥ hard anodize | IDD §4.1.1 | CDS §2.2.7 "should" ≥ 1 mm | **Δ** conflict on the edge radius: choose a frame with NRCSD heritage if the backup is live |
| LVR-106 | Integrable forwards and backwards (either Z face first) | IDD §4.1.5(2) | -Z first (CDS §2.2.2) | Check the AntS face and the switches |
| LVR-107 | Deployment velocity 0.5–2.0 m/s; tip-off ≤ 5°/s/axis (target < 2) | IDD §4.1.6 | Designed for 10°/s | None |
| LVR-108 | Quasi-static 7 g / 4 g / 4 g with 13.5 rad/s²; 1200 N across the rail ends in Z | IDD §4.3.1, §4.3.5 | 10 g / 17 g design | Check the 1200 N rail-end load |
| LVR-109 | Random vibration: soft-stow MEFL + 3 dB, 5.76 Grms, 60 s/axis, or hard-mount 9.47 Grms, 60 s/axis | IDD §4.3.2.1 | 5.57 Grms protoflight | **Δ** retest at the NRCSD level (Grms recomputed, `leop_feasibility.py` §5) |
| LVR-110 | Thermal 10–46 °C (Cygnus) to 0–50 °C (HTV), −10 to +45 °C on the arm; humidity 25–75 % RH; airlock 0–104.8 kPa at 1.0 kPa/s | IDD §4.3.6–§4.3.8 | — | Analysis |
| LVR-111 | Venting MEVR ≤ 5080 cm | IDD §4.4.2 | Same as CDS §2.1.9 | None |
| LVR-112 | Batteries per JSC 20793 (cell screening, acceptance testing, cell-level protection, circuit interrupters); > 80 Wh needs extra assessment | IDD §4.4.7 | ≈ 50 Wh, below 80 Wh | **Δ** JSC 20793 program: the backup's cost driver (BASELINE §3) |
| LVR-113 | Hazard controls to SSP 51700 (catastrophic: two-fault tolerant); secondary locking of external fasteners; materials per JSC 27472 and SSP 30233 | IDD §4.4 | 91-710 tailoring | **Δ** safety data package Phases 0–III (L-10 to L-3.5 M) |
| LVR-114 | Orbit 400–420 km, 51.6° | IDD §3.4.3; BASELINE §3 | 500 km SSO | **Δ** life 0.7–1.9 yr (Level 3 lost); β to ±75° changes the power and thermal cases; pass geometry changes |

## 8. Verification items

Each item verifies the requirements named; "when" is relative to the planning launch date L.

| ID | Test, inspection or demonstration | Pass criterion | Verifies | When |
|----|-----------------------------------|----------------|----------|------|
| VI-LV-01 | Rail measurement (Exolaunch guide or CIFP), protrusion check, RBF and switch positions against the Nova windows; fit check in a Nova TestPod with the 1U adapter | All CDS Appendix B dimensions in tolerance; slides freely; RBF reachable or removed by procedure | LVR-020–024, 029, 068 | Before environmental test (≈ L-5 M) and at integration |
| VI-LV-02 | Mass, CG (fixture, ±2 mm) and MOI (bifilar pendulum, ±5 %) of the flight unit, wings stowed | Mass ≤ 2.5 kg (Nova) and within the L-8 M bounds; CG within ±20 mm | LVR-005, 025–027 | L-4 M; as-measured mass at L-2 M |
| VI-LV-03 | Low-level sine sweep, three axes, before and after random | All modes > 40 Hz; first-mode shift < 10 % (investigate > 5 %); amplitude shift < 30 % | LVR-040 | With VI-LV-04 |
| VI-LV-04 | Protoflight random vibration at 5.57 Grms, 1 min/axis, in a TestPod; battery at the flight SOC, RBF out, switches actuated; the bus, burn lines and PA rail logged | No power-up of any load, no inhibit chatter (logged), no damage; functional test passes after | LVR-044, 045 | ≤ L-4 M (report due L-3.5 M) |
| VI-LV-05 | Separation-switch functional test before and after dynamics; switch toggle at t = 10, 29 and 44 min | Each toggle restarts both counts; the switches read released after the test | LVR-066, 067 | Before and after VI-LV-04 |
| VI-LV-06 | Inhibit-by-inhibit test: each of the three RF and three deployable inhibits held open alone, with the others closed | No current to the PA, the radio or any burn line through any single inhibit | LVR-053, 060, 069 | EM, then flight unit before VI-LV-04 |
| VI-LV-07 | Timer test on the flight unit (TP-RF-006 §8.8): release, resets mid-count (power-on, watchdog), switch bounce; spectrum analyser on the antenna ports, current probe on the burn lines | No RF before 45:00 min after the last release; no burn before 30:00 min | LVR-061, 063, 064, 066 | After VI-LV-04 |
| VI-LV-08 | Thermal vacuum and cycling with bakeout (protoflight acceptance ± 5 °C, 20 cycles, ≥ 4 in vacuum) | Functional at the extremes; bakeout criteria per the integrator | LVR-049, 050, 084 | ≤ L-4 M |
| VI-LV-09 | Materials list with ASTM E595 data or rationale codes A–E; contamination report | Every vacuum-exposed non-metallic item TML ≤ 1.0 %, CVCM ≤ 0.1 % or coded | LVR-080, 082 | L-4 M |
| VI-LV-10 | RF turn-on verification webform | Accepted by the integrator | LVR-006, 061 | L-8 M |
| VI-LV-11 | Cell screening and pack acceptance (OCV, capacity, internal resistance, vibration, vacuum) | Per BATTERY_DESIGN §9 | LVR-070, 073 | Before integration of the pack |
| VI-LV-12 | Venting analysis of every enclosed volume | Each volume / vent area ≤ 50.8 m | LVR-048 | L-6 M |
| VI-LV-13 | End-to-end ephemeris demonstration: receiver fix on the flatsat, downlink in telemetry, ground orbit determination, CCSDS OEM with covariance, test upload to the SpaceX and Space-Track test interfaces | Accepted in writing by the integrator and SpaceX before integration | LVR-089 | ≤ L-2 M |
| VI-LV-14 | Battery-side drain with all inhibits open (stowed configuration), at 20 °C and 30 °C | ≤ 20 µA | LVR-071 | EM, then flight unit |
| VI-LV-15 | Charging through the umbilical with the RBF inserted | Charges; no load (OBC, radio, burn lines) energized | LVR-060, IDD §4.2.1(4) | EM |
| VI-LV-16 | Shock (advised): not planned; risk acknowledged in the summary report | Acknowledgement accepted | LVR-046 | L-3.5 M |
| VI-LV-17 | Deployment test of the antennas and wings after environmental test, at the predicted wing hold-down temperature 30 min after separation in eclipse (the thermal model gives the value) | Both elements release on the first attempt at the cold temperature and at the minimum bus voltage | LVR-022, 064 | After VI-LV-08 |
| VI-LV-18 | Magnetic field of the stowed spacecraft (unpowered) | ≤ 0.5 G above Earth's field outside the envelope | LVR-085 | With the residual-dipole test (TP-ADCS) |

## 9. Deliverable schedule (planning launch L = 2028-06-01)

L = 2028-06-01 is the planning date of the mission simulation and the lifetime analysis (BASELINE §3), not a
contracted date. Shift every row with the contract.

| L-minus | Date | Deliverable or event | Source |
|---------|------|----------------------|--------|
| — | 2026-12-31 | Requests for quotes sent (D7); contracting entity decided (LV-09) | BASELINE §3, §9 |
| — | 2027-04-15 | IARU coordination request submitted (≥ 12 months before integration) | `docs/regulatory/IARU_COORDINATION.md` |
| — | 2027-06-30 | Launch contract signed with the altitude clause; else the NRCSD and Electron requests start | D7 |
| — | 2027-09-30 | Backup decision, if triggered | D7 |
| L-8 M | 2027-10-01 | Mission analysis inputs: configuration webform, CAD, dynamic model, mass properties ± 15 %, separation analysis, transmitter webform, environmental test approach, export compliance documents | UG Table G-3 |
| L-6 M | 2027-12-01 | SDHS and 91-710 compliance letter; FAA payload review template; cross-waivers | UG Table G-3 |
| L-5 M | 2028-01-01 | **Orbit selection** from SpaceX's list: ≤ 525 km must be on it, or the rebooking clause applies | UG Appendix G; BASELINE §3 |
| L-4.5 M | 2028-01-15 | FCC pre-space notification filed (≥ 90 days before integration, here 2028-04-15) | `FCC_97-207_PRE_SPACE_NOTIFICATION.md` |
| L-4 M | 2028-02-01 | Protoflight environmental campaign complete (VI-LV-03, 04, 05, 07, 08) | UG Fig. 1-2 |
| L-3.5 M | 2028-02-15 | Final mission analysis inputs: mass properties ± 5 %, environmental verification report | UG Table G-3 |
| L-2 M | 2028-04-01 | 18th SDS registration; licence copies; Appendix F letter; GOP and hazardous procedures; launch-site training; ephemeris capability demonstrated (VI-LV-13); as-measured mass | UG §1.2, Table G-3 |
| L-50 d | 2028-04-12 | Flight model delivered to the integrator (date set by the mission ICD) | — |
| L-47 d | 2028-04-15 | Final charge to 60 %, key load, ARM_FLIGHT, umbilical removed, RBF out, integration into the EXOpod Nova (the SIL's integration date) | UG §9.5.1 (dispenser at the site ≤ 8 weeks before launch) |
| ≈ L-5 d | 2028-05-27 | Encapsulated assembly mated to the LV | UG §10.1 |
| L+1 d | 2028-06-02 | Payload operations status; first operator ephemeris (≤ 24 h after first contact) | UG §9.5.7.2, Table G-3 |
| L+30 d | 2028-07-01 | Object claimed with the 18th SDS | UG §9.5.7.2 |

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-10-27 | Initial flow-down for the CDR from SpaceX UG v11 (Aug 2026), CDS Rev 14.1, EXOpod Nova UM Rev 1.2, EXOpod UM Rev 6.0 and the NRCSD IDD (2018); verification items VI-LV-01 to VI-LV-18; analyses in `software/simulation/cdr/launch/` |
