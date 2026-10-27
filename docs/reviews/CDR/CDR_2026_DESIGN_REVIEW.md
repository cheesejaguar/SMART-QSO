# SMART-QSO Design Review against the NPR 7123.1D CDR Criteria (2026-10-27)

**Document ID**: SQSO-REV-CDR-002
**Version**: A (draft: round 2 in progress)
**Date**: 2026-10-27
**Status**: Draft. Engineering assessment by AI reviewers against the NASA CDR criteria; **not an independent review
board** (§1.2)

---

## 1. Purpose and nature of the review

### 1.1 Purpose

The principal investigator asked for an exhaustive design review in the style of a NASA Critical Design Review: every
analysis the design needs, the factors of safety and margins, the governing NASA and launch-provider standards for a
Class D spacecraft, the full launch story (vehicle, integrator, dispenser, requirements on the spacecraft), a
minute-by-minute concept of operations, nothing missing from the design, and iteration until no action item is left
open.

### 1.2 What this review is and is not

- It is an engineering assessment against the CDR entrance and success criteria of NPR 7123.1D Appendix G (Table G-7),
  tailored for a Class D CubeSat, carried out by AI reviewers (Claude) working on the repository: seven discipline
  reviewers, five designers and a chair. Every finding is a Review Item Discrepancy (RID) with a recorded disposition
  and evidence.
- It is **not an independent review board**. There are no named human reviewers and no signatures. The funding audit
  flagged the earlier review records for exactly this (FINDINGS_REGISTER PI-03, PI-04); this record does not repair
  that. The project's independent PDR and CDR are scheduled (`mission/SCHEDULE.md`); this review is their input.
- The programmatics reviewer found that the project is at pre-PDR maturity by the NPR 7123.1D criteria (no
  independent SRR or PDR, no engineering model; `docs/project/CDR_CRITERIA_MATRIX.md`). The review therefore also did
  the design work a CDR presumes: schematic capture of the boards with selected parts, a mechanical CAD model and a
  finite-element model (§5).
- An action that needs hardware cannot close by analysis. Each one is a **verification item** with a test, a pass
  criterion and a date. An action that needs an outside party (a contract, an agreement, a filing) is a **decision
  point** with an owner and a date. Neither is counted as an open review action; both are tracked in the schedule and
  the verification matrix.

### 1.3 Governing framework (programmatics review, `docs/project/CDR_CRITERIA_MATRIX.md`)

No NASA directive is mandatory for this privately funded Part 97 mission. NPR 8705.4B (effective 2024-11-01) applies
to NASA payloads; "Class D" is the project's self-designation, used as the tailoring basis. If a NASA centre partners,
NPR 7120.8A would apply; on the NASA CSLI path, LSP-REQ-317.01 Rev B. Binding on the project: CDS Rev 14.1, the SpaceX
Rideshare Payload User's Guide v11 (August 2026, confirmed current), the integrator's ICD, 47 CFR Part 97, and the
FCC's five-year post-mission disposal rule. Used as standards of practice: NPR 7123.1D, NPR 7150.2D (software Class C),
NASA-STD-8739.8B, GSFC-STD-7000B, NASA-STD-5001B, -5017B, -5019A, -5020B, NASA-STD-8719.14C, NASA-STD-6016, JSC 20793
Rev D, NASA EEE-INST-002.

## 2. Panel and conduct

| Role | Scope | RID prefixes |
|------|-------|--------------|
| Chair | Baseline, ConOps, register, dispositions, this report | CH |
| Launch, mission integration and ConOps | Launch chain, requirements flow-down, minute-by-minute timeline, commissioning | LV |
| Structures, mechanisms and mass properties | MEL, CG, inertia, loads, factors of safety, deployables | STR |
| Power, thermal and battery safety | Power closure, thermal cases, battery hazards, EMC | EPS, THM, EMC |
| Communications, ground and regulatory | Link budgets, RF design, ground stations, Part 97, IARU, ITU | RF, GND, REG, DB |
| Avionics, flight software, radiation and GN&C | OBC, software readiness, radiation, ADCS | AV, FSW, RAD, GNC |
| Safety, debris and reliability | Hazards, ODAR, FMEA, reliability, parts and materials | SMA, ODA, REL |
| Programmatics, SE and V&V | Framework, criteria, requirements, schedule, cost, test programme | PM, SE, VV |
| Designers (round 2) | EPS board, RF board, OBC board and interconnect, mechanical CAD and FE, flight software | MCAD and closures |

Agenda: (1) discipline reviews and analyses; (2) chair dispositions and cross-discipline changes; (3) design
completion and implementation of the open RIDs; (4) closure verification by a fresh panel; (5) iteration until no
review action is open.

## 3. The mission as reviewed

### 3.1 How SMART-QSO gets to orbit

| Step | Who | What (source) |
|------|-----|---------------|
| Launch vehicle | SpaceX | Falcon 9, Transporter-class dedicated SSO rideshare from Vandenberg SLC-4E |
| Integrator | Exolaunch (lead RFQ; ISISPACE, SEOPS price checks; D-Orbit, Maverick one question each) | SpaceX's customer. SMART-QSO contracts with the integrator, not with SpaceX (`docs/ORBITAL_ACCESS.md`) |
| Dispenser | Exolaunch EXOpod Nova | A 1U in a 3U slot with a non-deployable adapter, or shared with other 1U/2U spacecraft; 1U limits 2.5 kg, CG ±20 mm |
| Contracting party | A legal entity, named by 2026-12-31 | Launch services agreement, FAA cross-waivers, insurance, export agreements |
| Contract | Signed by 2027-06-30, with the altitude clause | 475 km target, 485 km hard cap, no-penalty rebooking |
| Planning launch | The last Transporter of 2028 (about 2028-11-01) | June 2028 misses the integrator's protoflight milestone (§4) |
| Flight-model delivery | About L-50 d | To the integrator; final charge to 60 % ± 5 %, key load, ARM_FLIGHT, RBF out, integration into the Nova at L-47 d; no access afterwards |
| Orbit | 475 km circular SSO, LTAN 22:30 | FCC five-year rule with P ≥ 0.98 (ODAR Draft B) |
| Backup | ISS via Voyager NRCSD, 400–420 km | Contract needed 12 months before launch, so no earlier than about 2028-09 if triggered on 2027-09-30; design deltas in SQSO-LVR-001 §7 |
| Contingency | Rocket Lab Electron with a third-party dispenser | Export licence lead time 6+ months |

The requirements the launch puts on the spacecraft (114 rows, each with its source paragraph, design feature,
verification and status) are in `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001 Rev B), with the dated
deliverables (L-8 M CAD and mass properties, L-6 M safety letters, L-5 M orbit selection, L-4 M protoflight complete,
L-2 M licences and ephemeris demonstration).

### 3.2 Minute by minute (summary; full timeline `docs/operations/MISSION_TIMELINE.md`)

| Time | Event | Evidence |
|------|-------|----------|
| L-47 d to L | Unpowered in the Nova, three deployment switches held open, ≤ 20 µA drain; SOC 60 % → ≥ 58 % | LVR-071; storage model |
| S (separation) | Switches close; the OBC boots; EXOpod ejection 1.10–1.48 m/s, tip-off < 10°/s per axis | LV analysis |
| S + 0–120 s | No actuation (UG v11 §3.3.4); timers running | SRS-F086; SIL first coil command at 121 s |
| S + 120 s | B-dot detumble starts (magnetometer sampled 50 ms after a 900 ms coil burst) | ADCS_DESIGN 1.5 |
| S + 30 min | Wings deployed (two resistors per wing, 0.8 A, 8 s), then the AntS over I²C; 33 % of cases burn in eclipse | SIL: wings 30.1 min, antennas 30.4 min |
| S + 45 min | RF silence ends and is latched; receiver on; first beacon in IDLE (template, 120 s) | SIL 45.0 min |
| S + ≈ 1 h | Detumbled; spin-stabilized sun pointing of +Y at 8°/s | SIL 0.8 h; Monte Carlo ≤ 2.1 h |
| S + 0.7–12.2 h | First pass over the Bay Area (median 5.2 h); first commanding pass median 6.3 h | Pass analysis |
| ≤ 24 h after first contact | First operator ephemeris from the onboard GNSS fixes | UG v11 §9.5.7.2 |
| Day 1–14 | Commissioning: telecommand checkout from each station, TX-OFF proven, housekeeping dump, power and thermal characterised, then COMMISSIONING_RELEASE; transponder and Jetson only after the release | `docs/operations/COMMISSIONING_PLAN.md`; SIL release at day 1.5 |
| ≤ 30 d | Object claimed with the 18th SDS | REG-015 |
| 2 years | Operations: transponder duty-gated (sunlit, SOC ≥ 55 %, ≤ 25 % of an orbit) | SIL full mission |
| End of mission | Two-step passivation command: TX-OFF, then the one-shot charge disconnect | ICD-013 §6.4 |
| ≤ 5 years after | Re-entry by natural decay with the wings deployed | ODAR Draft B §6 |

### 3.3 Margins at this review

| Quantity | Requirement | Predicted | Margin | Source |
|----------|-------------|-----------|--------|--------|
| Mass | ≤ 2.00 kg (design), 2.5 kg Nova | MEV 1.73 kg | +15.9 % | MASS_BUDGET 3.1 |
| CG, stowed | ±20 mm | (+0.4, −0.3, −1.7) mm (CAD) | > 18 mm | MECHANICAL_DESIGN |
| Deployed inertia | Iyy ≥ 1.03 × max, axis within 5° of +Y | 1.047–1.057, 0.3–1.3° | Met | MECHANICAL_DESIGN |
| First mode, stowed | > 40 Hz | 170 Hz (FE), ≈ 140 Hz (analytic) | × 3.5–4.2 | STRUCTURAL_ANALYSIS B |
| Strength | MS ≥ 0 (NASA-STD-5001B protoflight factors at MPE + 3 dB) | Lowest +0.72 (local peak) | Positive | STRUCTURAL_ANALYSIS B |
| PCB fatigue (Steinberg) | MS ≥ 0 | +0.09 to +0.32 with two supports | Positive | STRUCTURAL_ANALYSIS B |
| Wing deployment torque | NASA-STD-5017B | see MCAD-01 (round 2) | — | MECHANISMS_ANALYSIS |
| Power, EOL, sun-pointed | ≥ 20 % | 3.93 W against 2.20 W | +79 % | POWER_BUDGET_ANALYSIS 2.0 |
| Power, EOL, tumbling | ≥ 20 % | 2.43 W against 1.80 W | +35 % | POWER_BUDGET_ANALYSIS 2.0 |
| Depth of discharge | ≤ 10 % per eclipse | ≤ 2.1 % as flown; 4.5 % worst contingency | × 2.2 | SIL |
| Battery temperature | 0 to +45 °C charging | ≥ +4.4 °C with +5/+9 °C set points | — | THERMAL_ANALYSIS 2.0 |
| Beacon link, 5° | ≥ 6 dB (median gain) | +7.6 dB | +1.6 dB | LINK_BUDGET 3.0 |
| Telecommand link, 10° | 6 dB with interference allowance | +5.3 dB at 50 W PEP (6 dB from 12°) | −0.7 dB at 10° | LINK_BUDGET 3.0 |
| Sun pointing | ±15° | mean 4.0°, 99.6 % within 15° | Met | BUILD_VS_BUY §6 |
| Detumble | ≤ 3 orbits | ≤ 2.1 h (≤ 1.4 orbits) | × 2.1 | BUILD_VS_BUY §6 |
| Total dose, 2 mm Al | design 10 krad, margin ≥ 2 | 5.25 krad at 500 km | × 1.90 → ≥ 2.1 mm required | RADIATION_ANALYSIS C |
| Orbital lifetime | ≤ 5 yr after mission, P ≥ 0.9 | P ≥ 0.98 at 475 km (planned-end reading) | Met; fallback 460 km | ODAR Draft B |
| Casualty risk | ≤ 1:10,000 | No fragment > 15 J (nominal); 1:47,000 worst sensitivity | Met | ODAR Draft B |
| Reliability | (goals 0.95 / 0.80 / 0.60) | 0.75 / 0.60 / 0.50 (L1 / L2 / L3) | Accepted Class D risk (REL-01) | RELIABILITY 2.0 |
| Schedule float | ≥ 6.6 weeks to the integrator milestone | +130 days for 2028-11 | Met | SCHEDULE |

## 4. Chair decisions

| # | Decision | RIDs |
|---|----------|------|
| C-1 | Planning launch the last Transporter of 2028 (about 2028-11); June 2028 an opportunity | PM-05, LV-10 |
| C-2 | Deployment altitude 475 km target, 465–485 km band, 485 km cap; 460 km if the FCC counts an early failure as the end of the mission | ODA-01, ODA-05 |
| C-3 | Layout B; AntS 2 m elements along ±X; CR0002 rods; deployed-inertia requirement | STR-01, STR-02, RF-03 |
| C-4 | Spin 8°/s, nutation damping 3·10⁻⁴ (1·10⁻⁴ without gyro) | GNC-02 |
| C-5 | Two resistors per wing on their own 0.8 A drivers, 8 s burns, wings before antennas | SMA-05, EPS-18, REL-03, STR-15, STR-16 |
| C-6 | Three deployment switches DS1–DS3 on three rails | LV-05, SMA-01, EPS-04, RF-14 |
| C-7 | Integration SOC 60 % ± 5 %, separation ≥ 40 %, drain ≤ 20 µA | LV-06, SMA-10, EPS-20 |
| C-8 | Telecommand stations ≥ 1,000 km apart, each able to command alone; agreements by 2027-01-05 | LV-08, GND-01 |
| C-9 | Reliability 0.75 / 0.60 / 0.50 accepted as Class D risk, with the burn-in and flatsat test programme | REL-01 |
| C-10 | `transponder.rs` and `ops_state.rs` added to the safety-critical list | FSW-05 |
| C-11 | Heater set points +5/+9 °C; ≈ 207 Ω heater | THM-02, BUILD_VS_BUY §4.5 |
| C-12 | Two centre supports on the EPS, OBC and carrier boards | STR-05, MECHANICAL_DESIGN |

## 5. Design completion (round 2)

<!-- Filled in as the round-2 designs land. -->

## 6. RID log

<!-- RID-SUMMARY-BEGIN -->
241 RIDs: severity 1: 28, severity 2: 136, severity 3: 77.

| Disposition | Round 1 | Final |
|-------------|---------|-------|
| Closed by analysis | 25 | 25 |
| Closed by design change | 94 | 109 |
| Closed by chair decision | 32 | 31 |
| Verification item | 29 | 29 |
| Decision point (dated) | 11 | 11 |
| Implementation | 50 | 36 |

Open implementation RIDs: SE-01, RF-04, RF-21, RF-22, GND-03, REG-01, DB-01, SMA-13, SMA-15, REL-04, EPS-06, EPS-07, EPS-10, EPS-14, THM-02, THM-08, THM-15, AV-01, AV-02, AV-03, AV-05, AV-06, AV-07, AV-08, AV-09, AV-10, FSW-05, FSW-07, FSW-09, RAD-02, GNC-04, MCAD-01, MCAD-02, RFB-03, RFB-04, RFB-06.
<!-- RID-SUMMARY-END -->

Severity 1 = mission or safety critical; 2 = must close before fabrication; 3 = should fix. "Round 1" is the
disposition when the RID was raised; "Final" is its disposition after round 2 and the closure verification.

<!-- RID-LOG-BEGIN -->
#### Launch, mission integration and ConOps (LV)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| LV-01 | 1 | No commissioning hold: ACTIVE, Jetson and transponder on at S+52.8 min before any ground contact | IMP | Closed by design change | Commissioning hold: FRAM region 11, COMMISSIONING_RELEASE 0x08; test_ops_state.c; SIL release at 132,811 s |
| LV-02 | 2 | Magnetorquers driven 1 s after separation; UG v11 §3.3.4 allows none before 120 s | IMP | Closed by design change | No magnetorquer output before 120 s (SRS-F086); SIL first coil command at 121 s |
| LV-03 | 3 | Hard-body radius not reported | CA | Closed by analysis | 0.55 m to the VHF tip, report 0.6 m (leop_feasibility.py §7; LVR-065) |
| LV-04 | 2 | No GNSS ephemeris chain (gnss.rs never called; no schedule, telemetry or ground OD) | IMP | Closed by design change | gnss_schedule.rs: sessions at boot, every 95 min in LEOP, then 6 h; fix in HK1; GNSS_REQUEST 0x60; ground OD VI-LV-13 |
| LV-05 | 1 | No inhibit schematic: third RF and deployable inhibit not designed | CD | Closed by design change | EPS_DESIGN 2.0 §8: DS1-DS3 on three rails (with SMA-01, EPS-04, RF-14) |
| LV-06 | 2 | Integration SOC stated five ways; no storage limit or drain specification | CH | Closed by chair decision | BASELINE §4 Integration and storage row: 60 % ± 5 %, ≥ 40 % at separation, ≤ 20 µA |
| LV-07 | 2 | A failed deployable cannot be retried from the ground; AntS is I2C-commanded, not a GPIO burn | IMP | Closed by design change | DEPLOY_RETRY 0x50, retries 50 min apart; AntS over I2C (ICD_OBC_ADCS §12.3) |
| LV-08 | 2 | Santa Clara and NASA Ames are 12 km apart: not geographically separated | CH | Closed by chair decision | BASELINE §1: two stations ≥ 1,000 km apart (with GND-01) |
| LV-09 | 2 | No contracting entity for the launch services agreement, cross-waivers, insurance, export | DP | Decision point (dated) | Legal entity named by 2026-12-31 (BASELINE §1; SQSO-OAP-001 DP-1) |
| LV-10 | 2 | Mission-analysis inputs due at L-8 M with no hardware | CH | Closed by chair decision | Planning launch 2028-11 (BASELINE §3); L-8 M = 2028-03-01 (SQSO-LVR-001 §9) |
| LV-11 | 3 | Launch ICD carried unmeasured values, wrong deployers and dates | CD | Closed by design change | SQSO-ICD-010 Rev E |
| LV-12 | 2 | CDS matrix misstated CDS 14.1 | CD | Closed by design change | SQSO-CDS-001 Rev B, clause by clause |
| LV-13 | 2 | Mission timeline and commissioning plan described another spacecraft | CD | Closed by design change | SQSO-MTL-001 2.0; SQSO-COMM-001 2.0 |
| LV-14 | 2 | Commands for commissioning do not exist (transponder on/off, GNSS fix, deployment retry) | IMP | Closed by design change | XPDR_ENABLE 0x34, XPDR_DISABLE 0x35, GNSS_REQUEST 0x60, DEPLOY_RETRY 0x50 |
| LV-15 | 2 | RF silence restarts whenever the sense line reads stowed, even after completion | IMP | Closed by design change | RF silence latched once complete (SRS-F087; test_rf_silence.c) |
| LV-16 | 2 | NRCSD backup changes the design and cannot meet mid-2028 | CH | Closed by chair decision | BASELINE §3 Backup row; SQSO-OAP-001 §3; SQSO-LVR-001 §7 |
| LV-17 | 2 | Mass limit: CDS 2.00 kg is a note; the dispenser limit governs | CH | Closed by chair decision | BASELINE §4: design limit 2.00 kg (+15.9 %), Nova 2.5 kg, NRCSD 2.40 kg |
| LV-18 | 3 | Ejection velocity, tip-off and transmitter turn-on rule not analysed | CA | Closed by analysis | 1.10-1.48 m/s; Table 4-10 7.5-8.6 s; tip-off < 10°/s |
| LV-19 | 3 | Venting not analysed | VI | Verification item | VI-LV-12 (≥ 22.3 mm², with STR-09) |
| LV-20 | 2 | Space-safety plan missed three UG rules | CD | Closed by design change | SQSO-REG-015 Rev C, R5-R8 |
| LV-21 | 2 | Campaign documents assumed team-led integration at the launch site | CD | Closed by design change | SQSO-OPS-004 Rev B; SQSO-OPS-005 Rev B |
| LV-22 | 3 | Release at the cold hold-down temperature not verified | VI | Verification item | VI-LV-17 |
| LV-23 | 3 | Battery as a sealed container; hazardous materials declaration; UN 38.3 | VI | Verification item | VI-LV-11; LVR-072, LVR-073 |
| LV-24 | 3 | Electron contingency lacked the export-licence lead time | CD | Closed by design change | SQSO-OAP-001 §4 |
| LV-25 | 3 | RBF, umbilical and switch positions not checked against the Nova access windows | VI | Verification item | VI-LV-01 |
| LV-26 | 2 | UG prohibited materials (pure tin, cadmium, zinc) not applied to parts | CD | Closed by design change | SQSO-MFG-002 Rev C §2A.3; SQSO-MAT-001 Rev B §2.2 (SMA-11); VI-SAF-07 |
| LV-27 | 2 | Downlink cannot prove commissioning: no housekeeping frame | IMP | Closed by design change | HK1 housekeeping frame every fifth beacon and on GET_HOUSEKEEPING 0x09 |
| LV-28 | 2 | CONOPS LEOP text wrong (SAFE beacons, early ACTIVE) | CH | Closed by chair decision | mission/CONOPS.md §2-§3 rewritten |
| LV-29 | 3 | No RTC: onboard time starts at separation | CD | Closed by design change | ICD-010 §9.3; MISSION_TIMELINE §8.7 |

#### Programmatics (PM)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| PM-01 | 1 | No independent SRR or PDR: this panel cannot be a CDR board | CH | Closed by chair decision | Review report §1: an engineering assessment against the CDR criteria; independent PDR and CDR scheduled (mission/SCHEDULE.md) |
| PM-02 | 3 | Review conduct: agenda, reviewers, dispositions | CH | Closed by chair decision | This report: panel, agenda, RID log, dispositions |
| PM-03 | 2 | Wrong standards cited across the project documents | CD | Closed by design change | PM-owned documents corrected |
| PM-04 | 2 | No configuration baselines, tags or controlled change process in use | CD | Closed by design change | CM plan 1.1; CR-2026-007; gen_vcrm.py --check |
| PM-05 | 1 | June 2028 misses the integrator's protoflight milestone by 21 days | CH | Closed by chair decision | BASELINE §3 Launch date: planning 2028-11 (+130 d float) |
| PM-06 | 2 | Cost ROM inconsistent with the schedule and the test plan | CH | Closed by chair decision | LIFECYCLE_COST_ROM update (I&T $35-101k, legal entity and insurance, CAD/FEM, FY shift) |
| PM-07 | 2 | Risk register lacked scores, owners, schedule and funding risks | CD | Closed by design change | SQSO-RISK-001 Rev B, R21-R24 |
| PM-08 | 2 | No procurement or supply-chain status | CD | Closed by design change | SQSO-HRT-001 2.0 with dated actions |
| PM-09 | 3 | Project plan, SEMP and tracker claimed reviews that were not held | CD | Closed by design change | PM-001 B; SEMP 2.0; HRT-001 2.0 |

#### Systems engineering (SE)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| SE-01 | 1 | No detailed (build-to) design: schematics with parts, layouts, mechanical CAD, structural model | IMP | Implementation | Round 2 design completion: schematic capture with selected parts, mechanical model |
| SE-02 | 2 | Flow-down missing from CDS, Part 97, NASA-STD-8719.14C and the provider | CD | Closed by design change | MRD-084..099; SYS-080..089, 703..709, 713, 900..919; LVR traced |
| SE-03 | 2 | Requirement values contradicted the baseline | CD | Closed by design change | MRD and 19 SYS requirements changed; CR-2026-007 |
| SE-04 | 2 | Seven requirements need the PI's ratified wording | DP | Decision point (dated) | Ratify TBR-MRD-002..005 and TBR-SYS-002..004 by the PDR (2027-01-12) |
| SE-05 | 1 | No integrator ICD and no IARU letter | DP | Decision point (dated) | RFQs by 2026-12-31; IARU request 2027-01-26 (SQSO-LVR-001 §9) |
| SE-06 | 3 | TBD/TBR targets were 'TBD' | CD | Closed by design change | Dates set |
| SE-07 | 1 | Margins below requirement: power, mass, Jetson heat path | CA | Closed by analysis | Power +35 %/+79 % (PBA v2.0); mass +15.9 % (MB v3.0); Jetson ≤ 39 °C at 0.44 W/K (THA v2.0) |
| SE-08 | 2 | No hardware FMEA, single-point-failure list, system hazard analysis | CD | Closed by design change | SQSO-FMEA-002; SQSO-SDP-002 1.0 |
| SE-09 | 2 | No technology readiness assessment for the Jetson in 1U and the custom transponder | VI | Verification item | VI-01, VI-02 exit criteria (EM) |
| SE-10 | 2 | NPR 7150.2D mapping missing; SRS-F081 cited but undefined | IMP | Closed by design change | SRS 1.6 requirements defined; SQSO-FSW-012 Appendix C mapping; traceability 0 undefined |
| SE-11 | 3 | No system security plan | IMP | Closed by design change | SQSO-SEC-001 Space System Protection and Security Plan (NASA-STD-1006A as practice) |
| SE-12 | 2 | No structural model; materials list incomplete | CA | Closed by analysis | SQSO-SA-001; SQSO-MAT-001 Rev B |
| SE-13 | 3 | SYS-112 stated without the nominal 10 % | CD | Closed by design change | SYS-112 |
| SE-14 | 2 | NASA DAS not run | VI | Verification item | ODA-02: DAS 3.2.6 run 2027-02 |

#### Verification and validation (VV)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| VV-01 | 1 | Protoflight defined backwards; vibration below the workmanship floor; TVAC ±5 °C 4 cycles; no bake-out | CD | Closed by design change | ETP Rev C; TP-ENV-002 E; TP-ENV-003 D; BASELINE §8 |
| VV-02 | 1 | Verification matrix had undefined IDs and unevidenced 'Complete' | CD | Closed by design change | VCRM Rev C (gen_vcrm.py) |
| VV-03 | 2 | No test facilities identified or costed | CA | Closed by analysis | ETP §7; test_cost.py |
| VV-04 | 3 | Two V&V plans | CD | Closed by design change | V&V plan 1.3; mission/VNV_PLAN.md pointer |
| VV-05 | 1 | No EM development tests before the CDR | VI | Verification item | VI-01..06, VI-09 on the EM, 2027-06 to 2027-08 |
| VV-06 | 2 | Deploying after protoflight vibration conflicts with CDS §3.5.2 | CD | Closed by design change | ETP §3.8; TP-ENV-005 1.1 |
| VV-07 | 2 | Shock waiver cannot be assumed | VI | Verification item | VI-07 |
| VV-08 | 2 | TVAC hot plateaus capped at +60 °C by the battery | VI | Verification item | VI-03 and a deviation (ETP §9) |
| VV-09 | 3 | Integration plan undated | CD | Closed by design change | SQSO-INT-001 Rev B |

#### RF link and hardware (RF)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| RF-01 | 2 | Link budget not re-issued (TB-05) | CA | Closed by analysis | SQSO-LBA-001 v3.0 |
| RF-02 | 2 | Telecommand at 25 W short of 6 dB | CD | Closed by design change | Stations at 50 W PEP: +5.3 dB at 10° (SQSO-GND-001 Rev C) |
| RF-03 | 2 | Antenna element orientation against the spin unspecified | CH | Closed by chair decision | BASELINE §7: 2 m elements along ±X (with STR-01) |
| RF-04 | 3 | 60 s beacon interval equals the spin period: losses repeat | IMP | Implementation | ±5 s beacon interval jitter |
| RF-05 | 3 | SYS-222 gain requirement unachievable with a monopole | CH | Closed by chair decision | SYS-222 replaced by the pass-weighted gain requirement (RF-P013) |
| RF-06 | 2 | Downlink 3rd harmonic inside the uplink segment | CH | Closed by chair decision | BASELINE §7 Uplink 435.000-437.300 MHz |
| RF-07 | 2 | 60 dB 3rd-harmonic suppression costs 3.2 dB | CH | Closed by chair decision | BASELINE §7: ≥ 70 dB at the antenna port; VI-RF-04, VI-RF-06 |
| RF-08 | 2 | No receive-filter requirement | CD | Closed by design change | RF-P007; VI-RF-06 |
| RF-09 | 3 | Spurious limits inconsistent | CH | Closed by chair decision | SYS-203 to 47 CFR 97.307(e) |
| RF-10 | 3 | Deviation ±3.5 kHz against the filed emission | CD | Closed by design change | TRANSMITTER_DESIGN 1.3 §5.2; VI-RF-15 |
| RF-11 | 2 | TCXO ±2.5 ppm too wide for telecommand | CD | Closed by design change | RF-P005 ±1.0 ppm; VI-RF-14 |
| RF-12 | 1 | RF hardware not selected or designed | IMP | Closed by design change | RF board build-to design: SQSO-RF-BRD-001 (316 parts, ERC 0 errors, BOM with stresses); EM tests VI-RFB-01..09 |
| RF-13 | 2 | TX-OFF independence over-claimed in filings | CD | Closed by design change | IARU v40 A.2; FCC pre-space A.1; VI-RF-12 |
| RF-14 | 2 | RF inhibits not shown to be physical | CD | Closed by design change | EPS_DESIGN 2.0 §8.1 |
| RF-15 | 2 | Stale RF documents | CD | Closed by design change | RF_DESIGN 2.0; ICD-003 2.0; ICD-005 2.0; ANTENNA 2.0 |
| RF-16 | 3 | Output low-pass filter never analysed | CA | Closed by analysis | LBA §7; VI-RF-04 |
| RF-17 | 2 | Uncorrected uplink Doppler: nothing decodes | CD | Closed by design change | Ground pre-correction to ±1 kHz (GND-R5) |
| RF-18 | 3 | CTCSS under Doppler not analysed | CA | Closed by analysis | LBA §4.4; VI-RF-05 |
| RF-19 | 2 | Budget inputs need hardware | VI | Verification item | VI-RF-01, 03, 08, 09, 10 |
| RF-20 | 3 | Self-generated EMI into the receivers | VI | Verification item | VI-RF-07 |
| RF-21 | 3 | Repeat-path readbacks not wired to the OBC | IMP | Implementation | OBC pins and telemetry for TONE_OK, XPDR_KEY, PTT |
| RF-22 | 3 | SIL link model simplified | IMP | Implementation | sqso-sil env/link.rs to LBA v3.0 |

#### Ground segment (GND)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| GND-01 | 1 | No telecommand station agreed | DP | Decision point (dated) | Written agreements for ≥ 2 stations ≥ 1,000 km apart by 2027-01-05 (BASELINE §1) |
| GND-02 | 2 | Ground design incomplete (70 cm TX, keys, time, power) | CD | Closed by design change | SQSO-GND-001 Rev C |
| GND-03 | 2 | No command response or contact count in the downlink | IMP | Implementation | Command acknowledgement frame; contact count in the beacon |
| GND-04 | 2 | Shared counter file is a common-mode failure | CD | Closed by design change | UTC-derived counters (ICD-013 v0.7 §5.2.1) |
| GND-05 | 2 | Bay Area within 240 km of Beale AFB: 50 W PEP limit | CD | Closed by design change | GND-R3; FCC consultation item |
| GND-06 | 3 | Operations manual did not match the dictionary | CD | Closed by design change | SQSO-GSOM-001 1.1 |
| GND-07 | 3 | SatNOGS capability assumed | CA | Closed by analysis | LBA; SatNOGS station census |
| GND-08 | 3 | Residential noise at a site | VI | Verification item | VI-GND-02 site survey |

#### Regulatory (REG)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| REG-01 | 1 | No passivation telecommand exists | IMP | Implementation | PASSIVATE_ARM / PASSIVATE_EXECUTE (ICD-013 v0.7 §6.4) with the EPS one-shot disconnect (EPS-06) |
| REG-02 | 2 | No dated regulatory plan | CD | Closed by design change | REGULATORY_ROADMAP Rev B |
| REG-03 | 3 | FCC rule changes after 2024 unchecked | CA | Closed by analysis | eCFR 2026-10-01 |
| REG-04 | 3 | IARU lead time unknown | CA | Closed by analysis | Median 57 d; schedule updated |
| REG-05 | 3 | ITU API timing and path | DP | Decision point (dated) | FCC consultation (D3) |
| REG-06 | 3 | Export: authentication vs encryption; foreign integrators | DP | Decision point (dated) | Export memo Rev B; counsel by 2026-12-15 |
| REG-07 | 3 | Stale status in spectrum and IARU documents | CD | Closed by design change | SPECTRUM_LICENSE Rev C; IARU_COORDINATION Rev D |
| REG-08 | 3 | NOAA consultation not sent | DP | Decision point (dated) | PI by 2026-11-30 |
| REG-09 | 2 | 30-day pre-space rule against the dates | CD | Closed by design change | File early 2027-01-19, replace after contract (SQSO-LVR-001 §9) |
| REG-10 | 3 | FCC consultation agenda lacks RF questions | CH | Closed by chair decision | Consultation agenda in REGULATORY_ROADMAP |

#### Data budget (DB)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| DB-01 | 2 | Stored housekeeping not implemented, no downlink path | IMP | Implementation | 5-min summaries ≥ 7 days, fault windows, DUMP_TLM (BASELINE §8) |
| DB-02 | 3 | Data budget compared the wrong quantities | CA | Closed by analysis | SQSO-DB-001 v2.0 |

#### Orbital debris (ODA)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| ODA-01 | 1 | 525 km cap and 500 km target fail the FCC 5-year rule | CH | Closed by chair decision | BASELINE §3: 475 km target, 485 km cap; fallback 460 km; MRD-013 |
| ODA-02 | 2 | NASA DAS not run | VI | Verification item | DAS 3.2.6, 2027-02 |
| ODA-03 | 2 | ODAR inputs and claims wrong | CA | Closed by analysis | ODAR Draft B §2, §6 |
| ODA-04 | 2 | Baseline lifetime table used 0.028 m² and 1.6 kg only | CH | Closed by chair decision | BASELINE §3 table from ODAR Draft B §6.2 |
| ODA-05 | 2 | Whether an early failure ends the mission | DP | Decision point (dated) | FCC consultation before 2027-01-19; fallback band in BASELINE §3 |
| ODA-06 | 2 | No explosion, collision or casualty analysis | CA | Closed by analysis | ODAR §4, §5, §7 |

#### Safety and mission assurance (SMA)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| SMA-01 | 1 | Third inhibit not designed | CD | Closed by design change | EPS_DESIGN 2.0 §8 (DS3) |
| SMA-02 | 1 | No system hazard analysis | CA | Closed by analysis | SQSO-SDP-002 1.0 |
| SMA-03 | 2 | Wrong or cancelled standards in S&MA plans | CD | Closed by design change | MAP 2.0; WKS 1.1; WP Rev B; CCP 2.0 |
| SMA-04 | 2 | Materials list unsourced; non-compliant items | CD | Closed by design change | SQSO-MAT-001 Rev B; VI-SAF-07 |
| SMA-05 | 1 | Wing burn ≈ 5.8 A exceeds the 5 A fuse; no hardware on-time limit | CD | Closed by design change | EPS_DESIGN 2.0 §5.1 |
| SMA-06 | 2 | No recharge lockout after deep over-discharge | CD | Closed by design change | EPS_DESIGN §3.3; clear command IMP |
| SMA-07 | 2 | Cell screening below JSC 20793 practice | VI | Verification item | V-BSA-1, V-BSA-3 |
| SMA-08 | 2 | Pack UN 38.3 and hazardous-materials declaration | VI | Verification item | VI-SAF-09 / V-BSA-5 |
| SMA-09 | 3 | SSCMAN 91-710 and ISS safety documents not obtained | DP | Decision point (dated) | Through the integrator at contract (2027-06-30) |
| SMA-10 | 3 | Integration SOC inconsistent | CD | Closed by design change | 60 % ± 5 % everywhere (EPS-20) |
| SMA-11 | 2 | Pure tin and other prohibited materials | CD | Closed by design change | EEE_PARTS_LIST Rev C §2A.3 |
| SMA-12 | 2 | EEE list: unsourced radiation claims, stale parts | CD | Closed by design change | SQSO-MFG-002 Rev C |
| SMA-13 | 2 | Passivation command does not exist | IMP | Implementation | With REG-01 |
| SMA-14 | 2 | Success criteria stale | CD | Closed by design change | SQSO-MSC-001 2.0 |
| SMA-15 | 3 | Telemetry needed by the success criteria missing | IMP | Implementation | With LV-27 (MSC v2.0 §7 field list) |
| SMA-16 | 2 | Risk register not 5×5 | CD | Closed by design change | SQSO-RISK-001 Rev B |

#### Reliability (REL)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| REL-01 | 2 | Reliability goals unsupported and not met | CH | Closed by chair decision | Accept 0.75/0.60/0.50 as Class D risk; test programme (≥ 500 h flatsat, ≥ 200 h burn-in) to reach 0.86/0.70/0.58 |
| REL-02 | 2 | Reliability document errors | CD | Closed by design change | SQSO-REL-001 2.0 |
| REL-03 | 2 | One burn driver for both wings | CD | Closed by design change | EPS_DESIGN 2.0 §5.1: one driver per resistor, two resistors per wing |
| REL-04 | 2 | Failed receiver plus the loss-of-contact inhibit silences the beacon | IMP | Implementation | Keep the beacon when the receiver is known failed (D3 logic) |
| REL-05 | 3 | No hardware FMEA or critical items list | CD | Closed by design change | SQSO-FMEA-002 |

#### Structures and mechanisms (STR)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| STR-01 | 1 | Deployed inertia: +Y not a robust major axis; antenna along ±Y defeats sun pointing | CH | Closed by design change | BASELINE §4 mass properties; CAD layout: Iyy/max 1.057 CBE, 1.047 MEV, axis within 1.3° (SQSO-MECH-CAD-001); full tensor in the flight law |
| STR-02 | 2 | MEL rebuilt; baseline obsolete; coils trip D8 | CH | Closed by chair decision | BASELINE §4 Mass: CR0002 rods, CBE 1.49 kg, MEV 1.73 kg, +15.9 % |
| STR-03 | 3 | CG never computed | CA | Closed by analysis | MASS_BUDGET v3.0 §5 |
| STR-04 | 2 | Bifilar MOI ±5 % cannot verify the Iyy/Izz difference | CH | Closed by chair decision | VI-LV-02 trifilar ±2 % (SQSO-LVR-001 Rev B); VI-STR-01 |
| STR-05 | 2 | PCB fatigue negative on four corner supports | CD | Closed by design change | Centre support (SA §8); VI-STR-02 |
| STR-06 | 2 | Stack 45 Hz if cantilevered | CD | Closed by design change | Tied to both end frames: 289 Hz (SA §5) |
| STR-07 | 2 | Jetson module held only by SODIMM latches | CD | Closed by design change | Clamp and staking (SA §8); VI-STR-02 |
| STR-08 | 2 | Battery isolator preload loss; cell retention | CD | Closed by design change | Disc spring etc. (SA §9); BATTERY_DESIGN §7.1 cell retention change by chair |
| STR-09 | 2 | Vent area fails CDS §2.1.9 | CD | Closed by design change | ≥ 22.3 mm² (SA §11); VI-STR-09 |
| STR-10 | 2 | Stowed wing 6.1 mm against 6.5 mm | VI | Verification item | VI-STR-08 with VI-LV-01 |
| STR-11 | 2 | AntS and GNSS patch against the ±Z standoffs | CH | Closed by chair decision | Recessed −Z face in the frame RFQ (BUILD_VS_BUY §2.2); VI-STR-11 |
| STR-12 | 2 | Two switches against three inhibits; springs undefined | CD | Closed by design change | Three switches (STRUCTURAL_DESIGN §5.2-§5.3) |
| STR-13 | 2 | No wing spring, sensor or jumper specification; no torque margin | CA | Closed by analysis | MECH §3-§4; VI-STR-04 |
| STR-14 | 2 | End-of-travel shock on the cells | CD | Closed by design change | Soft stop ≤ 0.3 N·m (MECH §5); VI-STR-05 |
| STR-15 | 2 | Wing burn energy marginal at −40 °C; AntS not modelled | IMP | Closed by design change | 8 s burn at 0.8 A, two resistors per wing on drivers A (PB10) and B (PE6), alternate attempts; SIL deploy.rs fixed and AntS modelled; VI-STR-06, VI-STR-13 |
| STR-16 | 2 | AntS tapes pass 4-6 mm below the stowed wings | IMP | Closed by design change | Wings first, then the AntS (SRS-F089); 37.8 mm nominal clearance (SQSO-MECH-CAD-001); stuck-wing case VI with the ISIS ICD |
| STR-17 | 3 | Hold-down line creep | VI | Verification item | VI-STR-07 |
| STR-18 | 2 | Thread-locker outgassing; locking rules | CD | Closed by design change | STRUCTURAL_DESIGN §8; VI-STR-12 |
| STR-19 | 3 | No fracture-control classification | CA | Closed by analysis | SA §14 |
| STR-20 | 2 | No frame vendor data; allowables not re-read | VI | Verification item | VI-STR-11 |
| STR-21 | 3 | Factors of safety from SpaceX only | CD | Closed by design change | NASA-STD-5001B protoflight factors at MPE + 3 dB (SA) |
| STR-22 | 3 | Dispenser-level loads used for components | CA | Closed by analysis | Max of QSL and 3σ Miles; Nova transfer function requested (RFQ) |
| STR-23 | 3 | STRUCTURE.md wrong | CD | Closed by design change | STRUCTURE.md B |
| STR-24 | 3 | README listed the removed drag device | CD | Closed by design change | README B |
| STR-25 | 2 | Battery leads too short; no hinge-crossing rules | CD | Closed by design change | HARNESS_DESIGN 1.1 |
| STR-26 | 2 | Structural design mass properties stale | CD | Closed by design change | STRUCTURAL_DESIGN 1.3 |
| STR-27 | 2 | One wing deployed: major axis 12.5° off +Y | IMP | Closed by design change | ADCS_SET_GEOMETRY 0x24, presets 0-3, persisted; one-wing Monte Carlo (BUILD_VS_BUY §6.4) |
| STR-28 | 3 | Flexible modes against spin and ADCS bandwidth | CA | Closed by analysis | MECH §9 |

#### Mechanical CAD and FE (MCAD)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| MCAD-01 | 2 | Wing torsion springs do not fit the hinge envelope; flex bend radius 1.25 mm | IMP | Implementation | Hinge redesign (round 2, mechanical designer) |
| MCAD-02 | 3 | Torquer deck 23 g against 6 g in the MEL | IMP | Implementation | Lighten or grow the MEL row (round 2) |
| MCAD-03 | 3 | Sweep clearance 0.35 mm needs the body cells offset 1 mm | DP | Decision point (dated) | Panel vendor RFQ |

#### Electrical power (EPS)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| EPS-01 | 1 | No power closure at CDR | CA | Closed by analysis | SQSO-PBA-001 v2.0 |
| EPS-02 | 2 | Baseline tumbling margin '+19 %' open | CH | Closed by chair decision | BASELINE §5: 2.43/3.93 W, +35 %/+79 % |
| EPS-03 | 2 | EPS documents described a 2S module EPS | CD | Closed by design change | EPS_DESIGN 2.0; PDU 1.1; EPS_ELECTRICAL 1.1; ICD-001 1.1 |
| EPS-04 | 1 | Third inhibit not designed (array path) | CD | Closed by design change | EPS_DESIGN §8 DS3; drawings S1/B2/W7 IMP |
| EPS-05 | 2 | Deployment-switch rating unspecified | CD | Closed by design change | §8.4 RC-filtered FET gates |
| EPS-06 | 2 | End-of-life charge disconnect has no hardware design, command or pins | IMP | Implementation | EPS_DESIGN §7 design; commands and pins with REG-01 |
| EPS-07 | 2 | Drawings drive the antennas from the OBC though AntS is bought | IMP | Implementation | Redraw S5/W7 for the AntS |
| EPS-08 | 2 | BMS low-temperature discharge cut-off would keep the bus dark | CD | Closed by design change | BATTERY_DESIGN 2.2 §4.1 |
| EPS-09 | 2 | Stuck-on Jetson drains the pack | CD | Closed by design change | Hardware load shed below 13.6 V (EPS_DESIGN §6) |
| EPS-10 | 3 | ACTIVE with wings stowed closes only through the SOC gate | IMP | Implementation | Hold IDLE when the wings read not deployed |
| EPS-11 | 2 | Overcharge had two firmware-set controls | CD | Closed by design change | Secondary 4.35 V protector (SQSO-BSA-001 H1); V-BSA-2 |
| EPS-12 | 2 | No latched recharge lockout | CD | Closed by design change | EPS_DESIGN §3.3; clear command IMP |
| EPS-13 | 3 | String TVS below cold Voc; wire derating | CD | Closed by design change | SMBJ7.0A; 24 AWG |
| EPS-14 | 3 | GNSS_EN proposed on PE4 (XPDR_EN) | IMP | Implementation | Pin assignment |
| EPS-15 | 2 | Jetson and PA share one 5 V buck | CD | Closed by design change | Separate PA buck (EPS_DESIGN); drawing S2 IMP |
| EPS-16 | 2 | Switch voltage ratings and trip levels wrong | CD | Closed by design change | EPS_DESIGN §5; PDU 1.1 |
| EPS-17 | 2 | Charger had no power path | CD | Closed by design change | EPS-F004 |
| EPS-18 | 1 | Direct-drive burn 5.8 A; one driver; no on-time limit | CD | Closed by design change | EPS_DESIGN §5.1; second wing burn pin IMP |
| EPS-19 | 3 | One tracker for strings at different temperatures | CD | Closed by design change | Four trackers (mppt_mismatch.py) |
| EPS-20 | 3 | Integration SOC inconsistent | CD | Closed by design change | 60 % ± 5 % |
| EPS-21 | 3 | SIL document states a 5 A BMS trip | CH | Closed by chair decision | SQSO-SIL-001 §3: 4 A |
| EPS-22 | 2 | UN 38.3 and JSC 20793 screening not planned | VI | Verification item | V-BSA-1, V-BSA-5 |
| EPS-23 | 3 | Battery failure tolerance per SSCMAN 91-710 Vol. 3 not read | DP | Decision point (dated) | Integrator at contract; design already two-fault tolerant for overcharge and short |

#### Thermal (THM)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| THM-01 | 1 | Thermal analysis used the wrong orbit and had no model | CA | Closed by analysis | SQSO-THA-001 v2.0 |
| THM-02 | 2 | Heater set points leave too little margin to 0 °C | IMP | Implementation | sqso_fsw::heater +5/+9 °C (BASELINE §8) |
| THM-03 | 3 | Wings stowed at β = 0 saturates the heater | VI | Verification item | V-THM-1 |
| THM-04 | 2 | SYS-600/601 cannot be met by the boards with margin | CH | Closed by chair decision | Per-unit allowable flight temperatures (THERMAL_DESIGN 1.3 §1.1; BASELINE §8) |
| THM-05 | 2 | Wing cell qualification range unknown | VI | Verification item | V-THM-4 (−56 to +93 °C) |
| THM-06 | 2 | Cold dispenser soak at the −10 °C discharge limit | VI | Verification item | V-THM-5 |
| THM-07 | 1 | Hot dispenser soak > +60 °C after 2.4 h | VI | Verification item | V-THM-5 with the integrator's dispenser analysis |
| THM-08 | 3 | No minimum Jetson power-on temperature | IMP | Implementation | Inhibit below −15 °C (BASELINE §6) |
| THM-09 | 2 | Battery node had no radiative coupling | CD | Closed by design change | sqso-sil env/thermal.rs |
| THM-10 | 2 | Jetson junction-to-spreader resistance unknown | VI | Verification item | V-THM-3 |
| THM-11 | 3 | Baseline hot case unreachable for this orbit | CH | Closed by chair decision | BASELINE §8 Thermal cases |
| THM-12 | 3 | SIL batches 10 min against the baseline's 120 s bursts | CH | Closed by chair decision | BASELINE §6 AI burst: ≤ 10 min per power-on |
| THM-13 | 3 | SpaceX protoflight thermal range exceeds the battery limits | VI | Verification item | V-THM-6 |
| THM-14 | 3 | THERMAL_DESIGN stale | CD | Closed by design change | THERMAL_DESIGN 1.3 |
| THM-15 | 3 | SIL heater fixed 0.5 W | IMP | Implementation | V²/R heater in sqso-sil mission.rs |

#### EMC (EMC)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| EMC-01 | 2 | No EMC analysis | CA | Closed by analysis | SQSO-EMC-001 |
| EMC-02 | 2 | Jetson and PA share one buck | CD | Closed by design change | With EPS-15 |
| EMC-03 | 2 | Converter harmonics near 435 MHz unplanned | CD | Closed by design change | Synchronised converter frequency plan (PDU 1.1) |
| EMC-04 | 3 | Jetson clock harmonic in the GNSS L1 band | VI | Verification item | V-EMC-1, V-EMC-2 |
| EMC-05 | 3 | Test plan lacks unit-level EMC tests | IMP | Closed by design change | ETP Rev C.1 §3.6: V-EMC-1, V-EMC-3, V-EMC-6 |

#### Avionics (AV)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| AV-01 | 2 | No OBC schematic; firmware pin map provisional and in conflict with OBC_DESIGN | IMP | Implementation | OBC board schematic and master pin allocation (round 2 design) |
| AV-02 | 3 | No external supervisor; orphan watchdog-kick line | IMP | Implementation | External supervisor in the OBC board design |
| AV-03 | 1 | No driver or pins for the gyro, magnetometer, sun sensors, magnetorquers; no sensor-rail latch-up switch | IMP | Implementation | Pins and hardware in the OBC board design; drivers in the flight software |
| AV-04 | 2 | GNSS chain missing | CD | Closed by design change | LPUART1 PG7/PG8, GNSS_EN PE5, gnss_schedule.rs; receiver trip VI AV-V8 |
| AV-05 | 2 | No TONE_OK or XPDR_KEY readbacks | IMP | Implementation | With RF-21 |
| AV-06 | 2 | One software state bit drives both Jetson switches, no readback | IMP | Implementation | Independent lines with readback (OBC board design); PAYLOAD_DESIGN 1.1 §11.1 |
| AV-07 | 2 | No SRAM parity handler, TMR or scrubbing | IMP | Implementation | SRAM2 parity with NMI handler; CRC of critical state (flight software round 2) |
| AV-08 | 2 | OBC and radio rail latch-up trips not specified | IMP | Implementation | EPS board design |
| AV-09 | 2 | Idle USART1 TX can back-power the unpowered Jetson | IMP | Implementation | Back-power protection in the OBC board design |
| AV-10 | 3 | No mission clock across resets | IMP | Implementation | RTC on the always-on rail (OBC board design) |
| AV-11 | 1 | XPDR_EN was a level | CD | Closed by design change | 100 Hz heartbeat, 60 ms lease, TIM7 on PE4; test_xpdr_heartbeat.c; SIL radio |

#### Flight software (FSW)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| FSW-01 | 1 | On the target, a silent EPS controller skipped the detumble and reported it complete | CD | Closed by design change | executive.rs run_adcs: host-only stand-in |
| FSW-04 | 2 | SRS lacked transponder, commissioning and ADCS requirements; SRS-F081 undefined | CD | Closed by design change | SRS 1.6: SRS-F080..F094, F100..F107 |
| FSW-05 | 2 | transponder.rs and ops_state.rs meet the safety-critical criteria but are not listed | IMP | Implementation | Chair: add to the list, 100 % line and condition coverage (round 2) |
| FSW-06 | 2 | SWE-220 cyclomatic complexity not measured for the Rust | VI | Verification item | Compliance matrix open item C-4 |
| FSW-07 | 2 | No end-of-life passivation telecommand | IMP | Implementation | With REG-01 |
| FSW-08 | 2 | Software FMEA scales inverted | CA | Closed by analysis | SOFTWARE_FMEA B §14 |
| FSW-09 | 2 | No recovery from a receiver SEFI | IMP | Implementation | RADIO_EN power cycle after a defined silence (round 2) |
| FSW-10 | 2 | NPR 7150.2D Appendix C mapping missing | CD | Closed by design change | SQSO-FSW-012 |
| FSW-11 | 3 | 3·10⁻⁴ nutation gain fails without a gyro | CD | Closed by design change | NUTATION_GAIN_NO_GYRO = 1·10⁻⁴ |
| FSW-12 | 3 | Three documents name different magnetometers and gyros; L293D cannot run at 3.3 V | CD | Closed by design change | ADCS_DESIGN 1.5; ICD_OBC_ADCS §12 |
| FSW-13 | 2 | OBC selector and transponder scheduler had no design | CD | Closed by design change | SQSO-PAY-SEL-001 (design complete; implementation post-CDR) |

#### Radiation (RAD)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| RAD-01 | 2 | Margin on 10 krad is 1.90×, below 2× | CH | Closed by chair decision | BASELINE §6: ≥ 2.1 mm Al-equivalent by sector analysis |
| RAD-02 | 3 | SIL dose model low by 0.55-0.64 | IMP | Implementation | sqso-sil env/radiation.rs (round 2) |
| RAD-03 | 3 | Upset rate and mitigation claims unsupported | CA | Closed by analysis | RADIATION_ANALYSIS C §5 |

#### Guidance, navigation and control (GNC)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| GNC-01 | 1 | Magnetometer sampled with the coils on | CD | Closed by design change | 900 ms on / 50 ms settle window; coupling in the Monte Carlo |
| GNC-02 | 1 | Computed inertia: Iyy/Izz 1.03-1.04 and products tilt the axis | CD | Closed by design change | Full tensor in the law and the SIL; 8°/s, k_n 3·10⁻⁴ |
| GNC-03 | 2 | Flexible 2 m antenna tapes not modelled | VI | Verification item | ADCS_DESIGN §11.5 |
| GNC-04 | 2 | Magnetometer not selected, no driver | IMP | Implementation | Selection in the OBC board design; driver in the flight software |
| GNC-05 | 2 | Spin axis and inertia selectable for one wing | CD | Closed by design change | ADCS_SET_GEOMETRY 0x24; SRS-F094 |
| GNC-06 | 3 | Rod remanence not modelled | VI | Verification item | AV-V4 |
<!-- RID-LOG-END -->

## 7. Verdict

<!-- Written after the closure verification. -->

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-10-27 | Draft during round 2 |
