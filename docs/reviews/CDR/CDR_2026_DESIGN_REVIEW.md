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
| Orbit | 475 km circular SSO, LTAN 22:30 | FCC five-year rule with P ≥ 0.997 under the planned-end reading (ODAR Draft D); 460 km if an early failure ends the mission |
| Backup | ISS via Voyager NRCSD, 400–420 km | Contract needed 12 months before launch, so no earlier than about 2028-09 if triggered on 2027-09-30; design deltas in SQSO-LVR-001 §7 |
| Contingency | Rocket Lab Electron with a third-party dispenser | Export licence lead time 6+ months |

The requirements the launch puts on the spacecraft (86 requirement rows, LVR-001 to LVR-114, each with its source
paragraph, design feature, verification and status, and 18 verification items) are in
`docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001), with the dated
deliverables (L-8 M CAD and mass properties, L-6 M safety letters, L-5 M orbit selection, L-4 M protoflight complete,
L-2 M licences and ephemeris demonstration).

### 3.2 Minute by minute (summary; full timeline `docs/operations/MISSION_TIMELINE.md`)

| Time | Event | Evidence |
|------|-------|----------|
| L-47 d to L | Unpowered in the Nova, three deployment switches held open; drain 11.9 µA worst with the tap ladder and supervision (limit 20 µA); SOC ≥ 54.6 % at separation from the 55 % minimum final charge | LVR-071; EPS_BOARD_DESIGN E §6.6 |
| S (separation) | Switches close; the OBC boots; EXOpod ejection 1.10–1.48 m/s, tip-off < 10°/s per axis | LV analysis |
| S + 0–120 s | No actuation (UG v11 §3.3.4); timers running | SRS-F086; SIL first coil command at 121 s |
| S + 120 s | B-dot detumble starts (magnetometer sampled 50 ms after a 900 ms coil burst) | ADCS_DESIGN 1.5 |
| S + 30 min | Wings deployed (one resistor at a time on four independent 0.8 A lines, 10 s each (C-17); the second resistor of a wing only if the first fails), then the AntS over I²C; 33 % of cases burn in eclipse | SIL: wings 30.2 min, antennas 30.6 min |
| S + 45 min | RF silence ends and is latched; receiver on; first beacon in IDLE (template, 120 s) | SIL 45.0 min |
| S + ≈ 1 h | Detumbled; spin-stabilized sun pointing of +Y at 8°/s | SIL 0.2 h, first Sun acquisition 22–36 min; Monte Carlo ≤ 2.1 h |
| S + 0.7–13.7 h | First pass over the Bay Area (median 5.3 h, at most 12.1 h); first commanding pass median 6.4 h (at most 13.7 h) | MISSION_TIMELINE 2.3 |
| ≤ 24 h after first contact | First operator ephemeris from the onboard GNSS fixes | UG v11 §9.5.7.2 |
| Day 1–14 | Commissioning: telecommand checkout from each station, TX-OFF proven, housekeeping dump, power and thermal characterised, then COMMISSIONING_RELEASE; transponder and Jetson only after the release | `docs/operations/COMMISSIONING_PLAN.md`; SIL release at day 1.5 |
| ≤ 30 d | Object claimed with the 18th SDS | REG-015 |
| 2 years | Operations: transponder duty-gated (sunlit, SOC ≥ 55 %, ≤ 25 % of an orbit) | SIL full mission |
| End of mission | Two-step passivation command early in a sunlit pass: arm (operational key), execute (recovery key); the one-shot charge disconnect fires while the transmitter still reports, a final frame carries the disconnect and fuse state, then TX-OFF is latched | ICD-013 §6.4; END_OF_LIFE_PLAN |
| ≤ 5 years after | Re-entry by natural decay with the wings deployed | ODAR Draft D §6 (4.8–5.7 years after deployment at 475 km, nominal and conservative); the SIL's simpler decay gives 4.0 years (SE-20) |

### 3.3 Margins at this review

| Quantity | Requirement | Predicted | Margin | Source |
|----------|-------------|-----------|--------|--------|
| Mass | ≤ 2.00 kg (design), 2.5 kg Nova | MEV 1.754 kg (ballast, harness distribution, cell retention) | +14.0 % (+42.5 % to the Nova) | MASS_BUDGET 3.1 |
| CG, stowed | ±20 mm | (+0.1, −0.4, −1.4) mm CBE (CAD) | > 18 mm | MECHANICAL_DESIGN |
| Deployed inertia | Iyy ≥ 1.03 × max, axis within 5° of +Y | 1.056 / 1.047 (CBE / MEV, four AntS elements, C-13 ballast, harness), 1.2° / 0.7°; MEV 5th percentile 1.037 at 2 mm placement scatter | Met | MECHANICAL_DESIGN §5 |
| First mode, stowed | > 40 Hz | 170 Hz CBE / 162 Hz MEV (FE, 2.0 mm EPS, rod on the −X wall), ≈ 140 Hz (analytic) | × 3.5–4.2 | STRUCTURAL_ANALYSIS B |
| Strength | MS ≥ 0 (NASA-STD-5001B protoflight factors at MPE + 3 dB) | Lowest +0.96 (+0.85 at MEV masses) | Positive | STRUCTURAL_ANALYSIS B |
| PCB fatigue (Steinberg) | MS ≥ 0 | +0.42 to +0.71 at CBE, +0.14 to +0.50 at MEV (2.0 mm EPS and carrier) | Positive | STRUCTURAL_ANALYSIS B |
| Wing deployment torque | NASA-STD-5017B torque margin ≥ 0 at end of travel, at −55 °C | +0.08 (one spring out +0.07); release-energy ratio 1.90 with the 10 s burn (C-17) | Positive; resistive torque below −40 °C verified at −55 °C (VI-MECH-01..03) | MECHANISMS_ANALYSIS E |
| Power, EOL, sun-pointed | ≥ 20 % | 3.92 W against the bottom-up ACTIVE load 2.60 W with the heater and RF board E's transponder (2.26 W allocation) | +48 % (+73 % on the allocation) | POWER_BUDGET_ANALYSIS 2.4; BASELINE §5 |
| Power, EOL, tumbling | ≥ 20 % | 2.48 W against the bottom-up ACTIVE load 1.91 W with the heater and RF board E's transponder (1.86 W allocation) | +27 % (+33 % on the allocation) | POWER_BUDGET_ANALYSIS 2.4; BASELINE §5 |
| Depth of discharge | ≤ 10 % per eclipse | ≤ 3.7 % as flown (SIL D, 475 km); 2.8 % worst contingency (PBA 2.2) | × 2.7 | SIL_MISSION_SIMULATION D §7; POWER_BUDGET_ANALYSIS 2.2 |
| Battery temperature | 0 to +45 °C charging | ≥ +4.4 °C at 14.0 V; +3.7 °C at the 12.0 V floor; +1.2 °C with a 0.10 W/K mount; −1.1 °C with the wings stowed (charging then inhibited in hardware) | Met in operation; stowed-wing case THM-03 | THERMAL_ANALYSIS |
| Beacon link, 5° | ≥ 6 dB (median gain) | +7.9 dB | +1.9 dB | LINK_BUDGET 3.2 |
| Telecommand link, 10° | 6 dB with interference allowance | ≥ +8.4 dB at both variants' minimums (variant A +9.5 dB), receiver as designed, 10th-percentile gain, pointing included (≥ +6.6 dB at 5°) | ≥ +2.4 dB | LINK_BUDGET 3.5 (C-14) |
| Sun pointing | ±15° | mean 1.8°, p95 3.5°, 99.98 % within 15° (SIL); Monte Carlo 100 % of sunlit time | Met | SIL_MISSION_SIMULATION D §7; BUILD_VS_BUY §6 |
| Detumble | ≤ 3 orbits | ≤ 2.1 h (≤ 1.4 orbits) | × 2.1 | BUILD_VS_BUY §6 |
| Total dose, 2 mm Al | design 10 krad, margin ≥ 2 | 5.25 krad at 500 km | × 1.90 → ≥ 2.1 mm required | RADIATION_ANALYSIS C |
| Orbital lifetime | ≤ 5 yr after mission, P ≥ 0.9 | P ≥ 0.997 at 475–485 km (planned-end reading); wings-failed probability 0.29 % | Met; fallback 460 km | ODAR Draft D |
| Casualty risk | ≤ 1:10,000 | No fragment > 15 J (nominal); 1:47,000 worst sensitivity | Met | ODAR Draft D |
| Reliability | (goals 0.95 / 0.80 / 0.60) | 0.75 / 0.61 / 0.51 (L1 / L2 / L3); 0.86 / 0.71 / 0.59 with the burn-in and flatsat programme | PI acceptance (REL-01) | RELIABILITY 2.1 |
| Schedule float | ≥ 9.6 weeks (67 d, GOLD 1.06) to the integrator milestone for 2028-11 | +71 days (10.1 weeks; layout re-estimated, reliability programme included; −20 d with one layout designer) | Met by 0.5 weeks | SCHEDULE Rev D; `schedule_cpm.py` |

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
| C-13 | 2 × 10 g non-ferromagnetic ballast on the Z axis at the end frames, so the deployed inertia ratio keeps margin with all four AntS elements | MCAD-09, RFB-11 |
| C-14 | Telecommand stations radiate ≥ 64 dBm EIRP above 10° elevation: ≥ 100 W PEP into ≥ 15.5 dBic outside the 47 CFR 2.106 US270 areas (variant A), ≤ 50 W PEP at the transmitter output into a ≥ 17.8 dBic stacked pair inside them (variant B); commands only above 10° | RF-23 |
| C-15 | Harness wire under the EPS tied to the deck straps and shear panels; only connector strain-relief ties on the EPS board (EPS Steinberg +0.23, −0.52 otherwise) | MCAD-11 |
| C-16 | Battery-lead shorts: the two positive-leg fuses are diverse by design (F0A MIL-PRF-23419/12 from AEM, F0B MIL-PRF-23419/8 from Littelfuse), with F0N in the negative leg the third, so BSA H3b is 2-fault tolerant without counting two same-part fuses as independent (an earlier wording did; revised after the board FMEA). NRCSD safety-engineer concurrence for the ISS backup path | EPSB-10, P3E-02 |
| C-17 | Burn 10 s per resistor (was 8 s): at the bounding −55 °C the 8 s burn gives a release-energy ratio of only 1.52 against 1.5; 10 s gives 1.90 inside the 12 s hardware one-shot | P3S-01 |
| C-18 | The AntS deploys only when its own reported temperature is ≥ −17 °C (the −20 °C qualified minimum plus 3 °C sensor tolerance), deferring to the next 50-min slot without spending an attempt; after 30 deferrals (about 25 h, which covers the wings-stowed case where it never warms) or on ground command it deploys regardless. An ungated release after an eclipse separation would be at −44 °C; the gate opens within 177 min in every nominal case, before the first commanding pass (median 6.4 h). The wings still go first at S + 30 min | P3T-09 |
| C-19 | The transmitters stay silent while the AntS deployment is pending (deferred by C-18, or not yet attempted); the downlink starts when the AntS reports deployed or its sequence ends, never before the 45-min RF silence. Keying into stowed elements trips the VSWR protection (SIL-37: two 10-min stops, 1,312 s beacon gap) and stresses the PA, and nothing is lost: the first commanding pass (median 6.4 h) is later than the gate's nominal worst case (177 min); a failed AntS sequence still gets its stowed-antenna beacon | SIL-37 (P3T-09) |
| C-20 | One Smart Beacon burst definition, the one the design implements: a Jetson power-on of at most 10 min and at most 3.6 kJ at the carrier input, ended early by the energy gate. The go/no-go metric is energy per accepted beacon line ≤ 0.1 Wh (360 J), counted on accepted lines (not per burst), measured on the EM over ≥ 30 bursts (VI-02) and confirmed in commissioning over ≥ 10 bursts (MSC E-02). SYS-512, VI-02 and the D5 960 J figure are revised to match (CR) | P3F-G-03 |
| C-21 | With one wing deployed and the ground unable to say which, the spacecraft uses preset 0 (both wings) rather than guess between presets 1 and 2: the wrong one-wing preset is the case not analysed, preset 0 costs 2–3 % of power. Presets 1 and 2 are used only after the commissioning identification procedure (array current against the sun-sensor angle over a spin) names the wing | P3F-G-06 |
| C-22 | The attitude-knowledge requirement is restated as what the control law uses: sun-vector knowledge within ±5° in sunlight from the sun sensors, and rate knowledge from the gyros. A spin-stabilised sun-pointer needs no onboard three-axis estimator, so the EKF/TRIAD of ADCS_DESIGN §4.3 is removed from the flight baseline; the Monte Carlo measures the sun-vector error to verify it | P3F-G-07 |

## 5. Design completion (round 2)

Round 1 found no build-to design (SE-01). Round 2 produced one, to schematic level. Each board is captured as code
(SKiDL) with selected, orderable parts, a netlist, a BOM with derating, an electrical rules check and its own
calculations; the mechanical design is a CAD model with interference checks and a shell finite-element model. Every
design was iterated against the others until the interfaces closed.

| Item | Document | What exists | Key results |
|------|----------|-------------|-------------|
| EPS and safety board | SQSO-EPS-BRD-001 C | 700 placements, ERC 0; trackers, battery management, current-limited rails, four independent burn lines, DS1–DS3 inhibits, one-shot end-of-life disconnect, recharge lockout | Burn 0.67 A nominal one resistor at a time; storage drain 7.7 µA worst; pack and tap fuses MIL-PRF-23419/12 (1000 A interrupt) |
| OBC board | SQSO-OBC-BRD-001 C | 351 placements, ERC 0; STM32L4R5, window-watchdog supervisor, RTC, FRAM, gyros, sun-sensor ADC, magnetorquer drivers; magnetometer board A2; access-port board A3 | Every pin one function (SQSO-ICX-001); PAYLOAD_EN off the JTAG pin; magnetometer moved to the +Z end (rod coupling ×1.44 lower) |
| RF board | SQSO-RF-BRD-001 D | 300 fitted parts, ERC 0; CC1200 receiver, TNC MCU, LMX2571 exciter, PA, filters, nickel-silver shield, MMCX ports | Desense ≤ 0.44 dB; receive noise 548–598 K; carrier-detect and receiver-health lines; magnetically clean near A2 |
| Jetson carrier | SQSO-JET-BRD-001 E | 137 placements, ERC 0; 5 V buck and eFuse, sequencing, translated UART, NVMe | Module capped at 6.5 W (+6.5–10 % to 8 W); gyro window; 2.0 mm, 6 layers |
| Interconnect | SQSO-ICX-001 C | Master pin allocation and harness list | 29-row firmware pin change list |
| Harness | SQSO-HAR-001 2.0 | 206 conductors, routes in the CAD frame, calculations script | Derating ≤ 60 % normal; drops ≤ 44 mV; corner fill ≤ 51 %; 84.3 g |
| Mechanical | SQSO-MECH-CAD-001 B | CAD (STEP), 58 harness zones, interference checks; hinge model; shell FE (eight cases) | First mode 169 Hz; lowest strength margin +1.00; board fatigue +0.08 to +0.37; inertia ratio 1.055 / 1.046 with the C-13 ballast; mass 1.748 kg MEV (+14.4 %) |
| Flight software | SRS 1.7 and later; `software/rust` | Passivation, stored housekeeping, acknowledgements, receiver supervision, memory-upset handling (round 2); pin map and drivers (round 3) | Full mission simulation 0 anomalies in 42 checks (SQSO-SIL-001 D §7) |

What the design does not yet have: PCB layouts (SE-24, scheduled after the independent PDR), an engineering model,
and the vendor-drawing confirmations listed as verification items (connector mated heights, gold part numbers, fuse
ratings, outgassing data).

## 6. RID log

<!-- RID-SUMMARY-BEGIN -->
408 RIDs: severity 1: 32, severity 2: 210, severity 3: 166.

| Disposition | Round 1 | Final |
|-------------|---------|-------|
| Closed by analysis | 25 | 28 |
| Closed by design change | 101 | 262 |
| Closed by chair decision | 42 | 35 |
| Verification item | 46 | 32 |
| Decision point (dated) | 15 | 34 |
| Implementation | 179 | 17 |

Round 3 (independent closure verification) reopened 45 RIDs and raised 102 new findings; 45 of the reopened and 82 of the new are since closed.

Open implementation RIDs: SMA-15, P3S-13, SMA-B02, P3F-G-01, P3F-G-02, P3F-G-03, P3F-G-04, P3F-G-05, P3F-G-06, P3F-G-07, P3F-G-08, P3F-A-01, P3F-A-02, P3F-A-03, P3F-A-04, P3F-A-05, P3F-A-06.
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
| LV-04 | 2 | No GNSS ephemeris chain (gnss.rs never called; no schedule, telemetry or ground OD) | IMP | Closed by design change | GNSS state frame after each fix (SRS-F158) decoded by ephemeris_tool.py: batch least squares with drag, CCSDS OEM 3.0 with covariance and a TLE (SQSO-GND-002 A): 0.6 km at +24 h (fix every orbit), 2.0-2.6 km in the deadline case, covariance realistic, 32 tests in CI; 24 h procedure §7, owner ground segment lead; VI-LV-13 demonstration |
| LV-05 | 1 | No inhibit schematic: third RF and deployable inhibit not designed | CD | Closed by design change | EPS_DESIGN 2.0 §8: DS1-DS3 on three rails (with SMA-01, EPS-04, RF-14) |
| LV-06 | 2 | Integration SOC stated five ways; no storage limit or drain specification | CH | Closed by chair decision | BASELINE §4 Integration and storage row: 60 % ± 5 %, ≥ 40 % at separation, ≤ 20 µA |
| LV-07 | 2 | A failed deployable cannot be retried from the ground; AntS is I2C-commanded, not a GPIO burn | IMP | Closed by design change | DEPLOY_RETRY 0x50, retries 50 min apart; AntS over I2C (ICD_OBC_ADCS §12.3) |
| LV-08 | 2 | Santa Clara and NASA Ames are 12 km apart: not geographically separated | CH | Closed by chair decision | BASELINE §1: two stations ≥ 1,000 km apart (with GND-01) |
| LV-09 | 2 | No contracting entity for the launch services agreement, cross-waivers, insurance, export | DP | Decision point (dated) | DA-05 (PI and licensee, 2026-12-31): contracting legal entity |
| LV-10 | 2 | Mission-analysis inputs due at L-8 M with no hardware | CH | Closed by chair decision | Planning launch 2028-11 (BASELINE §3); L-8 M = 2028-03-01 (SQSO-LVR-001 §9) |
| LV-11 | 3 | Launch ICD carried unmeasured values, wrong deployers and dates | CD | Closed by design change | SQSO-ICD-010 Rev G aligned to BASELINE Rev E (amended): mass, CG, tensors, DS1-DS3 for pack and array, 8.18 Grms, FE results |
| LV-12 | 2 | CDS matrix misstated CDS 14.1 | CD | Closed by design change | SQSO-CDS-001 Rev B, clause by clause |
| LV-13 | 2 | Mission timeline and commissioning plan described another spacecraft | CD | Closed by design change | SQSO-COMM-001 2.1 against the flight-software command dictionary: HK1/HK2 and acks as proof of each step, COMMISSIONING_RELEASE before the transponder, retry, GNSS, geometry and gyro-alignment commands, the 10°/12° rule |
| LV-14 | 2 | Commands for commissioning do not exist (transponder on/off, GNSS fix, deployment retry) | IMP | Closed by design change | XPDR_ENABLE 0x34, XPDR_DISABLE 0x35, GNSS_REQUEST 0x60, DEPLOY_RETRY 0x50 |
| LV-15 | 2 | RF silence restarts whenever the sense line reads stowed, even after completion | IMP | Closed by design change | RF silence latched once complete (SRS-F087; test_rf_silence.c) |
| LV-16 | 2 | NRCSD backup changes the design and cannot meet mid-2028 | CH | Closed by chair decision | BASELINE §3 Backup row; SQSO-OAP-001 §3; SQSO-LVR-001 §7 |
| LV-17 | 2 | Mass limit: CDS 2.00 kg is a note; the dispenser limit governs | CH | Closed by chair decision | BASELINE §4: design limit 2.00 kg (+15.9 %), Nova 2.5 kg, NRCSD 2.40 kg |
| LV-18 | 3 | Ejection velocity, tip-off and transmitter turn-on rule not analysed | CA | Closed by analysis | 1.10-1.48 m/s; Table 4-10 7.5-8.6 s; tip-off < 10°/s |
| LV-19 | 3 | Venting not analysed | VI | Decision point (dated) | DA-15 (mechanical lead, 2026-12-15): venting of every volume from the CAD, V/A ≤ 50.8 m (RF cans done) |
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
| PM-03 | 2 | Wrong standards cited across the project documents | CD | Closed by design change | Citations corrected to NASA-STD-8739.8B, NPR 7123.1D, NPR 7150.2D (TEST_PROCEDURES, REQUIREMENTS_TRACEABILITY, SAFETY_DATA_PACKAGE, SOFTWARE_FMEA, SOFTWARE_SAFETY_PLAN, README, SDP/SRS/STP/SDD/SHA headers) |
| PM-04 | 2 | No configuration baselines, tags or controlled change process in use | CD | Closed by design change | gen_vcrm, gen_rid_log and gen_vi_register --check in program-checks.yml (CMP 1.2 §4.5); CR-2026-009 to -015; PI ratification of CR-007/-008 as DA-01 (PI, by 2027-01-12) |
| PM-05 | 1 | June 2028 misses the integrator's protoflight milestone by 21 days | CH | Closed by chair decision | BASELINE §3 Launch date: planning 2028-11 (+71 d float against 67 wanted, SCHEDULE Rev D; June −80 d) |
| PM-06 | 2 | Cost ROM inconsistent with the schedule and the test plan | CH | Closed by chair decision | LIFECYCLE_COST_ROM update (I&T $35-101k, legal entity and insurance, CAD/FEM, FY shift) |
| PM-07 | 2 | Risk register lacked scores, owners, schedule and funding risks | CD | Closed by design change | SQSO-RISK-001 Rev B, R21-R24 |
| PM-08 | 2 | No procurement or supply-chain status | CD | Closed by design change | SQSO-HRT-001 2.0 with dated actions |
| PM-09 | 3 | Project plan, SEMP and tracker claimed reviews that were not held | CD | Closed by design change | PM-001 B; SEMP 2.0; HRT-001 2.0 |

#### Systems engineering (SE)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| SE-01 | 1 | No detailed (build-to) design: schematics with parts, layouts, mechanical CAD, structural model | IMP | Closed by design change | Build-to design at schematic level: EPS, OBC, RF, Jetson carrier, magnetometer A2 and access-port A3 boards (SKiDL netlists, BOMs, ERC 0), interconnect SQSO-ICX-001, harness SQSO-HAR-001 2.0, CAD with 94+ parts and interference checks, shell FE model; PCB layouts per SE-24 |
| SE-02 | 2 | Flow-down missing from CDS, Part 97, NASA-STD-8719.14C and the provider | CD | Closed by design change | MRD-084..099; SYS-080..089, 703..709, 713, 900..919; LVR traced |
| SE-03 | 2 | Requirement values contradicted the baseline | CD | Closed by design change | SYS Rev E/F/G in CR-2026-009; SYS-100 2.23 W (1.86 × 1.2, transponder allocation 1.85 W DC with RF board E) met by 2.48 W, +11 % |
| SE-04 | 2 | Seven requirements need the PI's ratified wording | DP | Decision point (dated) | Ratify TBR-MRD-002..005 and TBR-SYS-002..004 by the PDR (2027-01-12) |
| SE-05 | 1 | No integrator ICD and no IARU letter | DP | Decision point (dated) | DA-07 RFQs (PI, 2026-12-31), DA-08 mission ICD at the contract (PI, 2027-06-30), DA-09 IARU letter by 2027-05-18 (licensee) |
| SE-06 | 3 | TBD/TBR targets were 'TBD' | CD | Closed by design change | Dates set |
| SE-07 | 1 | Margins below requirement: power, mass, Jetson heat path | CA | Closed by design change | SYS-603 Rev G temperature limits (CR-2026-010): spreader ≤ +40 °C in a 600 s batch (39.0 °C predicted), junction ≤ 95 °C, ≤ +70 °C stuck on; verified by VI-01 and V-THM-3 |
| SE-08 | 2 | No hardware FMEA, single-point-failure list, system hazard analysis | CD | Closed by design change | SQSO-FMEA-002; SQSO-SDP-002 1.0 |
| SE-09 | 2 | No technology readiness assessment for the Jetson in 1U and the custom transponder | VI | Closed by analysis | SQSO-TRA-001 A: 12 critical elements against NPR 7123.1D App. E, TRL now with evidence, TRL 6 plan to the CDR 2027-09-28; the OBC selector and Jetson radiation have stated fallbacks |
| SE-10 | 2 | NPR 7150.2D mapping missing; SRS-F081 cited but undefined | IMP | Closed by design change | SRS 1.6 requirements defined; SQSO-FSW-012 Appendix C mapping; traceability 0 undefined |
| SE-11 | 3 | No system security plan | IMP | Closed by design change | SQSO-SEC-001 Space System Protection and Security Plan (NASA-STD-1006A as practice) |
| SE-12 | 2 | No structural model; materials list incomplete | CA | Closed by analysis | Shell FE model (56,258 DOF): first mode 170 Hz, lowest margin +0.72 (STRUCTURAL_ANALYSIS B); materials list SQSO-MAT-001 Rev B |
| SE-13 | 3 | SYS-112 stated without the nominal 10 % | CD | Closed by design change | SYS-112 |
| SE-14 | 2 | NASA DAS not run | VI | Decision point (dated) | DA-06: DAS requested by 2026-11-06, run by 2027-01-08, ODAR Draft E 2027-01-12, before the 2027-01-19 filing; criterion ≤ 5 yr after mission end at 485 km and 1.85 kg with P ≥ 0.9, casualty ≤ 1:10,000; owner systems engineer |
| SE-15 | 2 | End-of-life plan describes a drag device and the 25-year rule | IMP | Closed by design change | END_OF_LIFE_PLAN C rewritten: end criteria, 0x70/0x71 passivation step by step with go/no-go from HK1 and the ack, cases where it cannot be commanded, post-space notification, ODAR Draft D lifetimes |
| SE-16 | 2 | Analyses computed at superseded values: ODAR wings-failed case, link and data frame statistics at 6 °/s, power year runs at 500 km / 2028-06, burn transients with two resistors, the mission timeline LEOP table | IMP | Closed by analysis | Re-run at Rev E: ODAR Draft D (wings-failed 0.29 %, P 0.997), EMC C, LBA 3.2, DATA_BUDGET 2.1, GND D, PBA 2.2 (tumbling 2.48 W, +34 %), MISSION_TIMELINE 2.2 LEOP from the SIL, RELIABILITY 2.1 (0.75 / 0.61 / 0.51); scripts updated |
| SE-17 | 3 | Harness design H1-H6 predates the interconnect | CD | Closed by design change | HARNESS_DESIGN 1.2 marked superseded by SQSO-ICX-001 and W1-W10 |
| SE-18 | 3 | Baseline §7 claimed the uplink windows are ≥ 2 MHz below 3 × f_down; 436.222-436.500 MHz is 0.9 MHz below | CD | Closed by design change | BASELINE §7 and RF-P002 restated: ≥ 0.9 MHz, desense ≤ 0.44 dB even 100 kHz from 3f (SQSO-RF-BRD-001 §7.5) |
| SE-19 | 2 | No current harness design: the old harness document predates the interconnect, and the deck-to-EPS band is over-full by about 8 mm² of wire | IMP | Closed by design change | Harness design SQSO-HAR-001 2.0: 206 conductors, routes against the CAD (58 zones, no interference), EEE-INST-002 derating (worst 60 % normal), drops ≤ 44 mV, corner-bundle fill ≤ 51 % (band B resolved by routing the EPS-only harnesses under the EPS), mass 84.5 g, single-point ground; V-HAR-1 to V-HAR-13 |
| SE-20 | 3 | SIL decay re-enters 4.0 years after deployment; the ODAR estimator gives 4.8-5.7 years | CH | Closed by chair decision | The ODAR (NRLMSIS, NASA-STD-8719.14C cross-sections) governs lifetime compliance; the SIL decay is indicative; aligned if cheap (flight software round 3) |
| SE-21 | 3 | SIL deploys both wings from one resistor and models 0.04 W of coil power in detumble (board: 1.16-1.3 W) | IMP | Closed by design change | SIL: each wing its own restraint on four lines (sqso-sil hw/deploy.rs, SIL-24); coil I²R from VBAT through the bridges (SIL-25): 1.19 W with three rods saturated against the board's 1.16–1.3 W; burn-lines check 1; MISSION_TIMELINE 2.6 §5; confirmed by the round-3 panel (P3F-G) |
| SE-22 | 2 | Connector contacts specified in tin, which the EEE parts list prohibits; PVC-insulated ribbons; 24/26 AWG on Pico-Clasp crimps that take 28-32 AWG | IMP | Closed by design change | All HAR-001 2.0 §15 items done including the round-3 follow-ups (W12B five wires, W3 pins 28/40 I2C3, W8 RST_REQ, zones under the 2.0 mm EPS); gold finishes; checks clean |
| SE-23 | 3 | No published E595 outgassing values found for the harness wire, PTFE assemblies, sleeving, lacing tape or connector housings; Pico-Clasp gold suffix not found | VI | Decision point (dated) | With EPSB-09 (DA-18); V-HAR-11 before the EM order |
| SE-24 | 2 | No PCB layouts: the boards are at schematic, BOM, placement-constraint and stack-up level | CH | Closed by chair decision | Layout is the scheduled next design step (SCHEDULE Rev D: EM PCB layout and design-rule review from 2027-01-12, 27 designer-weeks, 15 calendar weeks with two designers (DA-04), after the independent PDR), held to the placement, keep-out, stack-up and impedance constraints recorded in the board designs; the independent CDR (2027-09-28) reviews the layouts |

#### Verification and validation (VV)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| VV-01 | 1 | Protoflight defined backwards; vibration below the workmanship floor; TVAC ±5 °C 4 cycles; no bake-out | CD | Closed by design change | ETP Rev C; TP-ENV-002 E; TP-ENV-003 D; BASELINE §8 |
| VV-02 | 1 | Verification matrix had undefined IDs and unevidenced 'Complete' | CD | Closed by design change | VCRM Rev C (gen_vcrm.py) |
| VV-03 | 2 | No test facilities identified or costed | CA | Closed by analysis | ETP §7; test_cost.py |
| VV-04 | 3 | Two V&V plans | CD | Closed by design change | V&V plan 1.3; mission/VNV_PLAN.md pointer |
| VV-05 | 1 | No EM development tests before the CDR | VI | Verification item | VI-01..06, VI-09 on the EM, 2027-06 to 2027-08 |
| VV-06 | 2 | Deploying after protoflight vibration conflicts with CDS §3.5.2 | CD | Closed by design change | VI-LV-17: cold, minimum-voltage deployment on the EM and on the FM before vibration (ETP D §3.8); continuity checks only after the environmental tests |
| VV-07 | 2 | Shock waiver cannot be assumed | VI | Verification item | LVR-046 and VI-LV-16 follow ETP D §3.3: attenuated Nova shock MPE in the RFQ and ICD; VI-07 decision by 2027-07-14 (DA-21); EM shock test costed $3k-12k |
| VV-08 | 2 | TVAC hot plateaus capped at +60 °C by the battery | VI | Closed by design change | ETP §9 deviations 1-3 and 7 to the PI in CR-2026-013 (by 2027-01-12) |
| VV-09 | 3 | Integration plan undated | CD | Closed by design change | SQSO-INT-001 Rev B |

#### RF link and hardware (RF)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| RF-01 | 2 | Link budget not re-issued (TB-05) | CA | Closed by analysis | SQSO-LBA-001 v3.0 |
| RF-02 | 2 | Telecommand at 25 W short of 6 dB | CD | Closed by design change | Stations at 50 W PEP: +5.3 dB at 10° (SQSO-GND-001 Rev C) |
| RF-03 | 2 | Antenna element orientation against the spin unspecified | CH | Closed by chair decision | BASELINE §7: 2 m elements along ±X (with STR-01) |
| RF-04 | 3 | 60 s beacon interval equals the spin period: losses repeat | IMP | Closed by design change | Beacon interval jittered ±5 s, mean unchanged (SRS-F127; test_beacon) |
| RF-05 | 3 | SYS-222 gain requirement unachievable with a monopole | CH | Closed by design change | SYS-222 restated to the RF-P013 pattern (CR-2026-009); TBR-SYS-003 for the PI (DA-20) |
| RF-06 | 2 | Downlink 3rd harmonic inside the uplink segment | CH | Closed by chair decision | BASELINE §7 Uplink 435.000-437.300 MHz |
| RF-07 | 2 | 60 dB 3rd-harmonic suppression costs 3.2 dB | CH | Closed by design change | SYS-203 ≥ 70 dB at the antenna port (CR-2026-009) |
| RF-08 | 2 | No receive-filter requirement | CD | Closed by design change | RF-P007; VI-RF-06 |
| RF-09 | 3 | Spurious limits inconsistent | CH | Closed by chair decision | SYS-203 to 47 CFR 97.307(e) |
| RF-10 | 3 | Deviation ±3.5 kHz against the filed emission | CD | Closed by design change | TRANSMITTER_DESIGN 1.3 §5.2; VI-RF-15 |
| RF-11 | 2 | TCXO ±2.5 ppm too wide for telecommand | CD | Closed by design change | RF-P005 ±1.0 ppm; VI-RF-14 |
| RF-12 | 1 | RF hardware not selected or designed | IMP | Closed by design change | RF board build-to design: SQSO-RF-BRD-001 (316 parts, ERC 0 errors, BOM with stresses); EM tests VI-RFB-01..09 |
| RF-13 | 2 | TX-OFF independence over-claimed in filings | CD | Closed by design change | IARU request A.6 item 7.8: 45-minute silence, 24 h receiver power cycle, the TNC in the command path; pre-space A.3 matches |
| RF-14 | 2 | RF inhibits not shown to be physical | CD | Closed by design change | EPS_DESIGN 2.0 §8.1 |
| RF-15 | 2 | Stale RF documents | CD | Closed by design change | RF_DESIGN 2.0; ICD-003 2.0; ICD-005 2.0; ANTENNA 2.0 |
| RF-16 | 3 | Output low-pass filter never analysed | CA | Closed by analysis | LBA §7; VI-RF-04 |
| RF-17 | 2 | Uncorrected uplink Doppler: nothing decodes | CD | Closed by design change | Ground pre-correction to ±1 kHz (GND-R5) |
| RF-18 | 3 | CTCSS under Doppler not analysed | CA | Closed by analysis | LBA §4.4; VI-RF-05 |
| RF-19 | 2 | Budget inputs need hardware | VI | Verification item | VI-RF-01, 03, 08, 09, 10 |
| RF-20 | 3 | Self-generated EMI into the receivers | VI | Verification item | VI-RF-07 |
| RF-21 | 3 | Repeat-path readbacks not wired to the OBC | IMP | Closed by design change | Readbacks wired on the RF board (W4 J1-8/9/10) and the OBC (PC9, PG12, PC7) |
| RF-22 | 3 | SIL link model simplified | IMP | Closed by design change | SIL link model to LBA 3.0 |
| RF-23 | 2 | Telecommand link with the receiver as designed: +3.8 dB at 10° at 50 W PEP against the 6 dB requirement (LB-3) | IMP | Closed by design change | C-14: ≥ 64 dBm EIRP above 10°; variant A ≥ 100 W into ≥ 15.5 dBic (+7.7 / +9.5 dB at 5° / 10°), variant B ≤ 50 W into a stacked 18.2 dBic pair at the US270 sites (+7.0 / +8.8 dB), 0.3 dB pointing (LBA 3.4; GND F; RF exposure 13-16 m, VI-GND-05) |

#### Ground segment (GND)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| GND-01 | 1 | No telecommand station agreed | DP | Decision point (dated) | DA-10 (licensee, 2027-01-05): station agreements |
| GND-02 | 2 | Ground design incomplete (70 cm TX, keys, time, power) | CD | Closed by design change | SQSO-GND-001 Rev C |
| GND-03 | 2 | No command response or contact count in the downlink | IMP | Closed by design change | Acknowledgement after each counted command; HK1 adds CN, PS, RX, ST (SRS-F125, F126); SIL 1,370 acks |
| GND-04 | 2 | Shared counter file is a common-mode failure | CD | Closed by design change | ICD-013 §5.2.1 adopted; telecommand_builder.py --counter-from-utc with station index, per-station counter files, clock guard ≤ 0.5 s, monotonic guard, reserve-first; compatible with keys.rs; test_utc_counters.py (23 tests); key custody GND-R6; VI-GND-04 on the EM |
| GND-05 | 2 | Bay Area within 240 km of Beale AFB: 50 W PEP limit | CD | Closed by design change | GND-R3; FCC consultation item |
| GND-06 | 3 | Operations manual did not match the dictionary | CD | Closed by design change | SQSO-GSOM-001 1.1 |
| GND-07 | 3 | SatNOGS capability assumed | CA | Closed by analysis | LBA; SatNOGS station census |
| GND-08 | 3 | Residential noise at a site | VI | Verification item | VI-GND-02 site survey |

#### Regulatory (REG)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| REG-01 | 1 | No passivation telecommand exists | IMP | Closed by design change | PASSIVATE_ARM 0x70 (operational key, 600 s window) and PASSIVATE_EXECUTE 0x71 (recovery key): TX-OFF latched, then EOL_ARM and EOL_FIRE 60-120 s in sunlight, up to 3 attempts, confirmed from EOL_STATE (SRS-F110-F112; test_passivation.c); SIL passivated by telecommand on day 730.1; EM test V-EPS-8 |
| REG-02 | 2 | No dated regulatory plan | CD | Closed by design change | REGULATORY_ROADMAP Rev B |
| REG-03 | 3 | FCC rule changes after 2024 unchecked | CA | Closed by analysis | eCFR 2026-10-01 |
| REG-04 | 3 | IARU lead time unknown | CA | Closed by analysis | Median 57 d; schedule updated |
| REG-05 | 3 | ITU API timing and path | DP | Decision point (dated) | DA-11 (licensee): FCC consultation request 2026-11-15, meeting 2027-01-05, outcome 2027-01-08 |
| REG-06 | 3 | Export: authentication vs encryption; foreign integrators | DP | Decision point (dated) | DA-13 (PI, 2026-12-15): export counsel |
| REG-07 | 3 | Stale status in spectrum and IARU documents | CD | Closed by design change | SPECTRUM_LICENSE Rev C; IARU_COORDINATION Rev D |
| REG-08 | 3 | NOAA consultation not sent | DP | Decision point (dated) | PI by 2026-11-30 |
| REG-09 | 2 | 30-day pre-space rule against the dates | CD | Closed by design change | File early 2027-01-19, replace after contract (SQSO-LVR-001 §9) |
| REG-10 | 3 | FCC consultation agenda lacks RF questions | CH | Closed by design change | FCC consultation agenda Q1-Q10 (REGULATORY_ROADMAP D §6): licensee; request 2026-11-15, meeting by 2027-01-05 |
| REG-11 | 2 | Beacon kept on with a known-failed receiver: a transmitter that telecommand cannot turn off | DP | Decision point (dated) | Declared in IARU A.6 item 7.9 and pre-space A.3; FCC question Q4 by 2027-01-05 (licensee); if rejected, one condition in contact_monitor.rs removes it |
| REG-12 | 2 | Passivation cannot be confirmed from the ground: TX-OFF is latched before the charge disconnect fires, and HK1 has no EOL_STATE or fuse field | IMP | Closed by design change | Fire-first passivation implemented (ICD-013, passivation.rs) and described in END_OF_LIFE_PLAN D (execute early in a pass sunlit ≥ 6 min after it), CONOPS, ODAR D.1 §4, pre-space A.3; EM test V-EPS-8 |

#### Data budget (DB)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| DB-01 | 2 | Stored housekeeping not implemented, no downlink path | IMP | Closed by design change | Stored housekeeping (SRS-F120-F122): 5-minute records for 7.1 days, 60 s records for 6.8 h, three fault windows; DUMP_TLM 0x0A, 4 records per frame every 12 s; SIL dumps check |
| DB-02 | 3 | Data budget compared the wrong quantities | CA | Closed by analysis | SQSO-DB-001 v2.0 |

#### Orbital debris (ODA)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| ODA-01 | 1 | 525 km cap and 500 km target fail the FCC 5-year rule | CH | Closed by chair decision | BASELINE §3: 475 km target, 485 km cap; fallback 460 km; MRD-013 |
| ODA-02 | 2 | NASA DAS not run | VI | Decision point (dated) | With SE-14 (DA-06) |
| ODA-03 | 2 | ODAR inputs and claims wrong | CA | Closed by analysis | ODAR Draft B §2, §6 |
| ODA-04 | 2 | Baseline lifetime table used 0.028 m² and 1.6 kg only | CH | Closed by chair decision | BASELINE §3 table from ODAR Draft B §6.2 |
| ODA-05 | 2 | Whether an early failure ends the mission | DP | Decision point (dated) | DA-11 (with REG-05); fallback 460-465 km by CR |
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
| SMA-09 | 3 | SSCMAN 91-710 and ISS safety documents not obtained | DP | Decision point (dated) | DA-07 / DA-08 (PI) |
| SMA-10 | 3 | Integration SOC inconsistent | CD | Closed by design change | 60 % ± 5 % everywhere (EPS-20) |
| SMA-11 | 2 | Pure tin and other prohibited materials | CD | Closed by design change | EEE_PARTS_LIST Rev C §2A.3 |
| SMA-12 | 2 | EEE list: unsourced radiation claims, stale parts | CD | Closed by design change | SQSO-MFG-002 Rev C |
| SMA-13 | 2 | Passivation command does not exist | IMP | Closed by design change | With REG-01 |
| SMA-14 | 2 | Success criteria stale | CD | Closed by design change | SQSO-MSC-001 2.0 |
| SMA-15 | 3 | Telemetry needed by the success criteria missing | IMP | Implementation | With LV-27 (MSC v2.0 §7 field list) |
| SMA-16 | 2 | Risk register not 5×5 | CD | Closed by design change | SQSO-RISK-001 Rev B |
| SMA-B01 | 3 | DS2 gate drive only from BAT_F: an open in the pack path leaves the spacecraft dark in sunlight | IMP | Closed by design change | DS2 gate drive from BAT_F or PACK_P through a dual diode (EPS_BOARD_DESIGN E); V-EPS-15 |
| SMA-B02 | 1 | Flight software reads the separation sense with the wrong polarity: in flight the RF silence and the deployment wait restart for ever | IMP | Implementation | Correct polarity and pull; test every sense input against the netlists (flight software, interconnect) |
| SMA-B03 | 2 | Battery monitor and backup over-voltage protector share the tap wires: one open tap can blow F3 or latch the lockout | IMP | Closed by design change | 1 MΩ ladder across the cells holds an open tap between its neighbours (4.2 µA) |
| SMA-B04 | 2 | A failed NTC1 stops charging for good or isolates the pack | IMP | Closed by design change | NTC1 plausibility comparators; charging follows NTC2 on an NTC1 fault (CHG_INHIBIT bit 6); V-EPS-14 |
| SMA-B05 | 2 | A spurious permanent load shed holds the transmitter off with no override | IMP | Closed by design change | PA out of the hardware load shed (CR-2026-016) |
| SMA-B06 | 2 | Antenna deployment depends on one DEPLOY_EN line and the shared EPS I2C bus | IMP | Closed by design change | AntS supply DEPLOY_EN OR the EPS controller's ANTS_PWR (0x8D); AntS on its own OBC I2C3 with both controllers (0x31, 0x32) (EPS E, OBC D) |
| SMA-B07 | 3 | Single pull-up resistors on the OBC's I2C buses | IMP | Closed by design change | I2C2 and I2C3 with two 4.7 kΩ pull-ups in parallel per line, each able to work alone; 100 kHz (OBC_BOARD_DESIGN D §23.1) |
| SMA-B08 | 2 | A failed watchdog timing capacitor can hold the OBC in a reset loop | IMP | Closed by design change | TPS3850 timing capacitor as two parallel strings of two 470 nF C0G in series: one short or open keeps the 10 s kick inside the window; V-OBC-1 tests both |
| SMA-B09 | 2 | An OBC fault below the 130 mA trip is detected by nothing | IMP | Closed by design change | EPS controller supervises an OBC heartbeat (OBC_HEARTBEAT 0x8C): 600 s timeout power-cycles SW10 through a hardware one-shot, ≤ 3 per 24 h (ICD-001 1.6); V-EPS-16; flight software side in round 3 |
| SMA-B10 | 2 | A dead FRAM forces TX-OFF every 60 s: permanent silence | IMP | Closed by design change | FRAM mirror and degraded mode (fram_health.rs, SRS-F160; tests three_failed_checks_declare_and_one_good_resets_the_run, bypass_only_when_failed_and_with_the_code, the_degraded_mode_ends_when_the_fram_reads_back); VI: flatsat with U3 removed, mirror used, no TX-OFF loop, 2027-06; confirmed by P3F-A |
| SMA-B11 | 2 | Receiver health line misses a deaf front end and an open TNC wire; RX_CD stuck high blocks recovery | IMP | Closed by design change | Stuck carrier, STATUS loop query, RX_FAULT self-test (receiver.rs, SRS-F115, F116, F152, tests); SIL rx-recovery 7 of 7 (longest 24.9 h); STATUS layout fixed by SQSO-SRS-TNC-001; confirmed by P3F-A |
| SMA-B12 | 2 | A shorted lockout coil-driver FET locks out charging permanently | IMP | Closed by design change | Series-redundant FET drivers on each lockout coil: one shorted FET leaves the coil off; V-EPS-18 |
| SMA-B13 | 3 | DS2's second-pole wire disables all four trackers if it chafes to the frame | IMP | Closed by design change | Active-high tracker clamps: a chafed switch wire removes a clamp rather than disabling the trackers; TRK_EN never leaves the board |
| SMA-B14 | 2 | A short on the access port's reset line holds the OBC in reset | IMP | Closed by design change | NRST no longer leaves the OBC: J9/A3 pin 6 is RST_REQ (active high) through a 2N7002BK; a short cannot hold reset (OBC_BOARD_DESIGN D) |
| SMA-B15 | 3 | Ceramic capacitors on the unfused battery lines can short | IMP | Closed by design change | Pack-line capacitors ahead of the monitor FETs as 50 V series pairs |
| SMA-B16 | 3 | Synthesizer shares the PA rail regulator | IMP | Closed by design change | LMX2571 on its own TPS7A2033 3.3 V regulator from the PA rail (3.25 V min, 100 mV margin); TX-OFF still removes it; keyed DC 1.84 W at 0.5 W RF, allocation raised to 1.85 W DC (BASELINE §5, SYS-100 2.23 W) (RF_BOARD_DESIGN E §5) |
| SMA-B17 | 3 | A shorted tracker output diode cannot be isolated | IMP | Closed by design change | Per-tracker disable by the controller (TRK_DISABLE 0x8E); V-EPS-17 |
| SMA-B18 | 3 | Magnetometer shares a switch and bus segment | IMP | Closed by design change | Magnetometer on its own switch U35 (MAG_EN PF14, MAG_RB PF15) and I2C4 repeater U34 |
| SMA-B19 | 2 | One failed PA-temperature reading silences the downlink | IMP | Closed by design change | TX_TEMP plausibility with the board + 15 °C fallback (pa_temp.rs, SRS-F161; tests the_open_line_falls_back_to_the_board, a_jump_and_a_disagreement_are_faults_that_clear); confirmed by P3F-A |
| SMA-B20 | 2 | The F1 fire's EPS-board ≥ 0 °C condition (ICD-001 1.6 §4.6) can block passivation: the board sits near −11.5 °C in the cold orbit case and the temperature is unreadable with the EPS I2C bus down; not in SRS-F111 or passivation.rs | IMP | Closed by design change | Fire's EPS board ≥ 0 °C waited at most 6 h per attempt, then in sunlight; unreadable does not block (passivation.rs, SRS-F111; Rust and C test_passivation); confirmed by P3F-A |
| SMA-B21 | 2 | F3 now blows (15–27 W for up to 1 min, about 1.6 kJ) at the tab of the cell it protects, during that cell's overcharge: not analysed | IMP | Closed by design change | F3 on a thermal island ≥ 25 mm from every tab and can, 25 × 2 mm 2-oz neck (≈ 390 K/W), no pour toward the cells: ≤ 0.6 W reaches the cells, nearest can rise ≤ 1 K (one-cell lumped bound without the island: +32 K, fails the +60 °C limit); V-BSA-2 measures island, cell-4 tab and nearest can during the EM F3 trip, pass can ≤ 60 °C and rise ≤ 5 K (BATTERY_DESIGN 2.10, BSA G) |
| SMA-B22 | 2 | F3 is now in every pack-short loop (up to 211 A) beside the 5 A pack fuses; coordination not shown | IMP | Verification item | F3 never interrupts the short: F0A/F0B/F0N in series break up to 211 A (1000 A / 300 A ratings) against F3's 50 A; F3 survives 60 A × 5 ms 10,000 times (melt I²t well above 18 A²s) against an estimated ≈ 10 A²s let-through of a 5 A fast fuse at 40×; AEM and Littelfuse I²t curves a data item before the EM order 2027-01-15 (EPS designer); V-BSA-9 hard-short abuse test with F3 fitted, pass F3 intact or open without rupture |
| SMA-B23 | 3 | EPS_BOARD_DESIGN F §3, §6.1, §17 still place F3 on the EPS board | IMP | Closed by design change | EPS_BOARD_DESIGN G §3, §6.1, §17 and the shunt departure row no longer place F3 or the BQ771806 on the EPS board |
| SMA-B24 | 3 | U2's own taps have no open-tap ladder: one open resistor can blow F3 or blind a channel | IMP | Closed by design change | 4 × 2.2 MΩ 1 % tap-to-tap ladder at the BQ771806 pins after the 1 kΩ filters, 1.9 µA; cells 1 and 4 shift ≤ 1.9 mV, trip 4.350 V ± 12 mV; storage drain 7.8 µA typical, 13.8 µA worst against 20 µA, ≥ 54.5 % SOC after 47 days against ≥ 40 % (EPS_DESIGN 2.5) |

#### Reliability (REL)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| REL-01 | 2 | Reliability goals unsupported and not met | CH | Decision point (dated) | DA-02 (PI, 2026-12-15): 0.75 / 0.61 / 0.51, or 0.86 / 0.71 / 0.59 with the programme; VI-11 ≥ 500 h EM flatsat and VI-12 ≥ 200 h FM burn-in in ETP D §3.11, the schedule and the VCRM |
| REL-02 | 2 | Reliability document errors | CD | Closed by design change | SQSO-REL-001 2.0 |
| REL-03 | 2 | One burn driver for both wings | CD | Closed by design change | EPS_DESIGN 2.0 §5.1: one driver per resistor, two resistors per wing |
| REL-04 | 2 | Failed receiver plus the loss-of-contact inhibit silences the beacon | IMP | Closed by design change | Beacon continues when the receiver is known failed (recovery tried, health line in fault, contact timeout expired); transponder off; SIL contingency 26.5 days, longest gap 66 s (SRS-F116, F117); regulatory acceptance REG-11; health line RFB-07 |
| REL-05 | 3 | No hardware FMEA or critical items list | CD | Closed by design change | SQSO-FMEA-002 |

#### Structures and mechanisms (STR)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| STR-01 | 1 | Deployed inertia: +Y not a robust major axis; antenna along ±Y defeats sun pointing | CH | Closed by design change | BASELINE §4 mass properties; CAD layout: Iyy/max 1.056 CBE, 1.047 MEV, axis 0.6° / 1.1° from +Y (SQSO-MECH-CAD-001); full tensor in the flight law |
| STR-02 | 2 | MEL rebuilt; baseline obsolete; coils trip D8 | CH | Closed by chair decision | BASELINE §4 Mass: CR0002 rods, CBE 1.49 kg, MEV 1.73 kg, +15.9 % |
| STR-03 | 3 | CG never computed | CA | Closed by analysis | MASS_BUDGET v3.0 §5 |
| STR-04 | 2 | Bifilar MOI ±5 % cannot verify the Iyy/Izz difference | CH | Closed by design change | One accuracy in VI-LV-02, VI-STR-01, VI-MCAD-03 and ETP §3.7: spin-balance, MOI ±0.5 %, products ±1×10⁻⁵ kg·m², CG ±0.1 mm; pass deployed ratio − 0.010 ≥ 1.03, measured tilt ≤ 2.4° |
| STR-05 | 2 | PCB fatigue negative on four corner supports | CD | Closed by design change | Two centre supports at (0, ±21) mm: Steinberg margins +0.09 to +0.36 (STRUCTURAL_ANALYSIS B); VI-STR-02 |
| STR-06 | 2 | Stack 45 Hz if cantilevered | CD | Closed by design change | Tied to both end frames: 289 Hz (SA §5) |
| STR-07 | 2 | Jetson module held only by SODIMM latches | CD | Closed by design change | Clamp and staking (SA §8); VI-STR-02 |
| STR-08 | 2 | Battery isolator preload loss; cell retention | CD | Closed by design change | Mechanical cell retention, no adhesive or potting: G-10CR locator plates, disc-spring stack per cell at 140 N, vents open; MEL row 37 5.0 g; MS_sep +0.19 at 142 g (MECHANICAL_DESIGN §4.6); VI-MCAD-10 |
| STR-09 | 2 | Vent area fails CDS §2.1.9 | CD | Closed by design change | ≥ 22.3 mm² (SA §11); VI-STR-09 |
| STR-10 | 2 | Stowed wing 6.1 mm against 6.5 mm | VI | Closed by design change | Stowed protrusion 6.10 mm nominal, 6.38 mm worst case with the 0.5 mm panel recess (MECHANICAL_DESIGN B); VI-STR-08 |
| STR-11 | 2 | AntS and GNSS patch against the ±Z standoffs | CH | Closed by design change | −Z face recessed 1.5 mm: AntS 1.25 mm inside the rail ends (MECHANICAL_DESIGN); frame RFQ |
| STR-12 | 2 | Two switches against three inhibits; springs undefined | CD | Closed by design change | Three switches (STRUCTURAL_DESIGN §5.2-§5.3) |
| STR-13 | 2 | No wing spring, sensor or jumper specification; no torque margin | CA | Closed by design change | STRUCTURAL_DESIGN 1.5 §6.2 Version B hinge; cold point −55 °C (MECHANISMS_ANALYSIS E) |
| STR-14 | 2 | End-of-travel shock on the cells | CD | Closed by design change | Soft stop ≤ 0.3 N·m (MECH §5); VI-STR-05 |
| STR-15 | 2 | Wing burn energy marginal at −40 °C; AntS not modelled | IMP | Closed by design change | 8 s burn at 0.8 A, two resistors per wing on drivers A (PB10) and B (PE6), alternate attempts; SIL deploy.rs fixed and AntS modelled; VI-STR-06, VI-STR-13 |
| STR-16 | 2 | AntS tapes pass 4-6 mm below the stowed wings | IMP | Closed by design change | Wings first, then the AntS (SRS-F089); 37.8 mm nominal clearance (SQSO-MECH-CAD-001); stuck-wing case VI with the ISIS ICD |
| STR-17 | 3 | Hold-down line creep | VI | Verification item | VI-STR-07 |
| STR-18 | 2 | Thread-locker outgassing; locking rules | CD | Closed by design change | STRUCTURAL_DESIGN §8; VI-STR-12 |
| STR-19 | 3 | No fracture-control classification | CA | Closed by analysis | SA §14 |
| STR-20 | 2 | No frame vendor data; allowables not re-read | VI | Verification item | VI-STR-11 due 2027-01-15, before the frame order |
| STR-21 | 3 | Factors of safety from SpaceX only | CD | Closed by design change | NASA-STD-5001B protoflight factors at MPE + 3 dB (SA) |
| STR-22 | 3 | Dispenser-level loads used for components | CA | Closed by analysis | Max of QSL and 3σ Miles; Nova transfer function requested (RFQ) |
| STR-23 | 3 | STRUCTURE.md wrong | CD | Closed by design change | STRUCTURE.md B |
| STR-24 | 3 | README listed the removed drag device | CD | Closed by design change | README B |
| STR-25 | 2 | Battery leads too short; no hinge-crossing rules | CD | Closed by design change | HARNESS_DESIGN 1.1 |
| STR-26 | 2 | Structural design mass properties stale | CD | Closed by design change | STRUCTURAL_DESIGN 1.5 aligned to MECHANICAL_DESIGN, STRUCTURAL_ANALYSIS and MEL 3.1 |
| STR-27 | 2 | One wing deployed: major axis 12.5° off +Y | IMP | Closed by design change | ADCS presets WING_1_ONLY, WING_2_ONLY, NO_WINGS to MASS_BUDGET 3.2 §6.5 and the CAD results.json, spin axes the tensors' major axes (≤ 0.003°); Monte Carlo W1 8.4°/99.9 %, W2 9.3°/99.9 %, none 3.9°/100 % (ADCS_DESIGN 1.8 §11.4); identification of the deployed wing: P3F-G-06; confirmed by P3F-G |
| STR-28 | 3 | Flexible modes against spin and ADCS bandwidth | CA | Closed by analysis | MECH §9 |

#### Mechanical CAD and FE (MCAD)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| MCAD-01 | 2 | Wing torsion springs do not fit the hinge envelope; flex bend radius 1.25 mm | IMP | Closed by design change | Two coaxial 17-7PH torsion springs, clock-spring flex, damped stop: end-of-travel torque margin +0.14 cold (+0.13 one spring out) (MECHANISMS_ANALYSIS B §4A); VI-STR-04, VI-MECH-01, VI-MECH-02 |
| MCAD-02 | 3 | Torquer deck 23 g against 6 g in the MEL | IMP | Closed by design change | Deck 1.0 mm skeleton 9 g; MEL row 5 re-split, total unchanged (MASS_BUDGET 3.1) |
| MCAD-03 | 3 | Sweep clearance 0.35 mm needs the body cells offset 1 mm | DP | Decision point (dated) | DA-14 (mechanical lead, 2026-12-31): 1 mm cell offset in the panel RFQ; vendor confirms ≥ 0.35 mm clearance before the 2027-01-15 order |
| MCAD-04 | 2 | Gyro module 14 mm tall cannot fit under the carrier; raising the carrier or a bracket do not fit layout B | CD | Closed by design change | Gyro on the OBC through a 27 × 27 mm carrier window with a 2.2 g frame: 1.5 mm clearance, carrier Steinberg +0.09, inertia ratio 1.056 / 1.047, axis 0.4° / 1.4° (MECHANICAL_DESIGN §4.3, D-11) |
| MCAD-05 | 2 | Board-mounted gyro is 0.6-1.0° off the body axes: 0.7° uses the whole 0.1 °/s bias budget at 8 °/s | VI | Verification item | VI-MCAD-05: measure the gyro axes to ≤ 0.1° on the flight stack; correction matrix to the flight software |
| MCAD-06 | 3 | RM3100 43 mm from the Y rod end and 10 mm below carrier parts | IMP | Closed by design change | RM3100 on its own board A2 in the +Z end-frame slot (+24, +27): rod coupling 830 nT per mA·m² (was 1194), nearest rod end 54.7 mm, 63 mm from the Jetson input; 20 mm non-ferromagnetic zone; AV-V4 with per-polarity calibration (SQSO-OBC-BRD-001 B) |
| MCAD-07 | 3 | Magnetometer board A2 needs a 20 mm non-ferromagnetic zone (shield lid, column stud, fasteners, trim mass, W1 hinge pins, cell interconnects) | CD | Closed by design change | Materials specified (MECHANICAL_DESIGN §4.3.3, D-9); hinge spring, MP3 and carrier MP1 accepted as fixed biases; VI-MCAD-07 measures the zone and the bias |
| MCAD-08 | 3 | Window-frame screws: 0.45 mm walls at the proposed positions; frame adhesive weakens above 60 °C | IMP | Closed by design change | Bonded frame withdrawn (thermal shear at −40 °C near adhesive strength; no adhesive strong hot within the 1.0 % TML limit): carrier 2.0 mm with the window, fatigue +0.07, carrier mode 479 Hz; FE re-run lowest strength +0.74; materials list C: no structural adhesive |
| MCAD-09 | 2 | With all four AntS elements the deployed inertia ratio is 1.045 / 1.037 (CBE / MEV): 0.007 above the 1.03 requirement, gone at 1.5-2 mm placement scatter | CH | Closed by chair decision | Chair C-13: 2 × 10 g non-ferromagnetic ballast on the Z axis at the end frames in the baseline (ratio about 1.057 / 1.048); trimmed at integration (VI-MCAD-03) |
| MCAD-10 | 3 | A286 hinge pins are softer than 440C and gall on titanium if the MoS₂ film wears through (friction 0.26 zeroes the torque margin) | VI | Verification item | VI-MECH-03: pin friction and galling with A286 over the deployment and test cycles |
| MCAD-11 | 2 | EPS fatigue margin depends on where the ~25 g of harness under the EPS is tied: −0.52 if tied to the EPS | CH | Closed by chair decision | Chair C-15: wire tied to the deck straps and shear panels, only connector strain relief on the EPS (+0.23); harness design §10 and workmanship |
| MCAD-12 | 3 | MZ sun sensor beside the AntS: the −X 2 m tape exit may overlap the diode's lowest 0.5 mm; deck displacement 0.46 mm (3σ) against the GNSS receiver's random qualification | VI | Decision point (dated) | With MCAD-13: GNSS mount response extracted (36 Grms, 0.21 mm); qualification spectrum by 2027-01-15 (GNC lead) |
| MCAD-13 | 2 | Deck responds at up to 36 Grms (3σ 109 g) at the GNSS receiver mounts; SkyFox publishes no random qualification level; the CR0002 rod clip response 204 g | DP | Decision point (dated) | GNC lead obtains the piNAV-NG and CR0002 qualification spectra by 2027-01-15; criterion: vendor spectrum envelopes the FE response +3 dB; fallback defined: stiffen the deck (≈ +10 g) or isolate the receiver |
| MCAD-14 | 3 | DS2 as a DPDT switch fits the Ø4.7 mm bore only with a body ≤ Ø4.5 mm, not confirmed to exist | DP | Decision point (dated) | Switch RFQ (mechanical lead, 2026-12-31) asks for both a ≤ Ø4.5 mm DPDT and the fallback (a second SPDT in a +Z standoff, EPS designer) |
| MCAD-15 | 3 | Worst-case 246 N axial load on each cell rim needs the cell maker's limit | VI | Verification item | VI-MCAD-10: preload 140 ± 10 N per cell; OCV and impedance unchanged, no can damage after protoflight random; cell-maker rim limit as a data item before the EM order |
| MCAD-16 | 2 | At −81 °C (line free span, with margin) a stiff hold-down line can go slack (0.008-0.020 mm at EA 10 kN after creep): the wing could lift off its post and rattle before the burn | CD | Closed by design change | Series 17-7PH compression spring (≈ 40 N/mm) at the line anchor: tension 19.0-19.5 N at −81 °C after creep for either line material; VI-STR-07 adds a −81 °C thermal cycle of line, spring and anchor, pass ≥ 10 N (MECHANISMS_ANALYSIS E); spring placed in the CAD on each burn board (1.0 mm to the harness strip, 3.95 mm to the wing sweep, ≥ 14.5 N at −81 °C with guide friction, 0.6 g per wing inside the burn-board allocation); line EA and creep data DI-MECH-01 by 2027-01-15 |

#### Electrical power (EPS)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| EPS-01 | 1 | No power closure at CDR | CA | Closed by analysis | SQSO-PBA-001 v2.0 |
| EPS-02 | 2 | Baseline tumbling margin '+19 %' open | CH | Closed by design change | Generation 2.48 / 3.92 W EOL; ACTIVE +34 % / +54 % against the bottom-up load with the heater (PBA 2.3) |
| EPS-03 | 2 | EPS documents described a 2S module EPS | CD | Closed by design change | EPS_DESIGN 2.0; PDU 1.1; EPS_ELECTRICAL 1.1; ICD-001 1.1 |
| EPS-04 | 1 | Third inhibit not designed (array path) | CD | Closed by design change | Third array inhibit: DS2 now two-pole, pole 2 holds the tracker enables low independently of DS3; array broken by DS1, DS2, DS3; boost-off pass-through ≤ 6.16 V below every converter start (≥ 9.9 V); V-EPS-3 with each pair jumpered (EPS_BOARD_DESIGN D §6.2) |
| EPS-05 | 2 | Deployment-switch rating unspecified | CD | Closed by design change | §8.4 RC-filtered FET gates |
| EPS-06 | 2 | End-of-life charge disconnect has no hardware design, command or pins | IMP | Closed by design change | One-shot disconnect F1 with EOL_FIRE and EOL_ARM latch (SQSO-EPS-BRD-001 §7), PF4 and PG1 (SQSO-ICX-001), commands with REG-01; V-EPS-8 |
| EPS-07 | 2 | Drawings drive the antennas from the OBC though AntS is bought | IMP | Closed by design change | S5 and W7 redrawn for the AntS: supply after DS1-DS3, switched by DEPLOY_EN, I2C through a TCA9517 (SQSO-EPS-BRD-001) |
| EPS-08 | 2 | BMS low-temperature discharge cut-off would keep the bus dark | CD | Closed by design change | BATTERY_DESIGN 2.2 §4.1 |
| EPS-09 | 2 | Stuck-on Jetson drains the pack | CD | Closed by design change | SHED_N also gates the Jetson eFuse U19 enable: covers either series switch failed closed, a stuck enable, or software that never turns the Jetson off; residual both power stages failed closed (EPS_BOARD_DESIGN D §18, PBA 2.3 §8.4) |
| EPS-10 | 3 | ACTIVE with wings stowed closes only through the SOC gate | IMP | Closed by design change | IDLE held while the wings are spent and not deployed (SRS-F128; test_ops_state) |
| EPS-11 | 2 | Overcharge had two firmware-set controls | CD | Closed by design change | Secondary 4.35 V protector (SQSO-BSA-001 H1); V-BSA-2 |
| EPS-12 | 2 | No latched recharge lockout | CD | Closed by design change | EPS_CLEAR_LOCKOUT 0x14: I2C unlock then LOCK_CLR 150 ms (SRS-F129); V-EPS-9 |
| EPS-13 | 3 | String TVS below cold Voc; wire derating | CD | Closed by design change | SMBJ7.0A; 24 AWG |
| EPS-14 | 3 | GNSS_EN proposed on PE4 (XPDR_EN) | IMP | Closed by design change | GNSS_EN PE5, XPDR_EN PE4 (SQSO-ICX-001) |
| EPS-15 | 2 | Jetson and PA share one 5 V buck | CD | Closed by design change | Separate PA buck (EPS_DESIGN); drawing S2 IMP |
| EPS-16 | 2 | Switch voltage ratings and trip levels wrong | CD | Closed by design change | EPS_DESIGN §5; PDU 1.1 |
| EPS-17 | 2 | Charger had no power path | CD | Closed by design change | EPS-F004 |
| EPS-18 | 1 | Direct-drive burn 5.8 A; one driver; no on-time limit | CD | Closed by design change | EPS_DESIGN §5.1; second wing burn pin IMP |
| EPS-19 | 3 | One tracker for strings at different temperatures | CD | Closed by design change | Four trackers (mppt_mismatch.py) |
| EPS-20 | 3 | Integration SOC inconsistent | CD | Closed by design change | 60 % ± 5 % |
| EPS-21 | 3 | SIL document states a 5 A BMS trip | CH | Closed by chair decision | SQSO-SIL-001 §3: 4 A |
| EPS-22 | 2 | UN 38.3 and JSC 20793 screening not planned | VI | Verification item | V-BSA-1, V-BSA-5 |
| EPS-23 | 3 | Battery failure tolerance per SSCMAN 91-710 Vol. 3 not read | DP | Decision point (dated) | DA-07 (RFQ, 2026-12-31) and DA-08 (integrator safety requirements at the contract 2027-06-30, assessed by 2027-07-14); owner PI; claims restated in BSA D |

#### Thermal (THM)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| THM-01 | 1 | Thermal analysis used the wrong orbit and had no model | CA | Closed by analysis | SQSO-THA-001 v2.0 |
| THM-02 | 2 | Heater set points leave too little margin to 0 °C | IMP | Closed by design change | Heater on below +5 °C, off above +9 °C (SRS-F130) |
| THM-03 | 3 | Wings stowed at β = 0 saturates the heater | VI | Closed by chair decision | Accepted as a contingency: with the wings stowed the heater holds the pack at +2.6 °C at 1.47 W generation (+0.3 °C at the bound), charging refused in hardware below +1.1 °C (fail-safe, SYS-602); flight-software rule: SAFE rather than IDLE while the battery's orbit minimum is below +3 °C (THERMAL_ANALYSIS 2.2 §6.8) |
| THM-04 | 2 | SYS-600/601 cannot be met by the boards with margin | CH | Closed by design change | Per-unit allowables from the BOMs: −30 °C cold for every board, extended to −40 °C for the first orbit after separation, shown by a −40 °C cold start (V-THM-8; GEVS deviation in the ETP); worst new case L4 −27.6 / −28.6 / −28.9 °C (THERMAL_ANALYSIS 2.2 §6.6); SYS-600/601 by CR-2026-017; Y451 in its −40 °C grade |
| THM-05 | 2 | Wing cell qualification range unknown | VI | Verification item | V-THM-4 due 2027-01-15, before the panel order |
| THM-06 | 2 | Cold dispenser soak at the −10 °C discharge limit | VI | Decision point (dated) | With THM-07 (DA-12) |
| THM-07 | 1 | Hot dispenser soak > +60 °C after 2.4 h | VI | Decision point (dated) | DA-12: dispenser thermal analysis requested in the RFQ (2026-12-31), delivered at the contract (2027-06-30), owner PI with the integrator; pass pack ≤ +50 °C and ≥ −15 °C; fallback deployment-time limit, finish, pad conditioning, then the SOC/NRCSD decision |
| THM-08 | 3 | No minimum Jetson power-on temperature | IMP | Closed by design change | Jetson not powered below −15 °C (SRS-F131) |
| THM-09 | 2 | Battery node had no radiative coupling | CD | Closed by design change | sqso-sil env/thermal.rs |
| THM-10 | 2 | Jetson junction-to-spreader resistance unknown | VI | Verification item | V-THM-3 |
| THM-11 | 3 | Baseline hot case unreachable for this orbit | CH | Closed by chair decision | BASELINE §8 Thermal cases |
| THM-12 | 3 | SIL batches 10 min against the baseline's 120 s bursts | CH | Closed by chair decision | BASELINE §6 AI burst: ≤ 10 min per power-on |
| THM-13 | 3 | SpaceX protoflight thermal range exceeds the battery limits | VI | Verification item | V-THM-6 |
| THM-14 | 3 | THERMAL_DESIGN stale | CD | Closed by design change | THERMAL_DESIGN 1.3 |
| THM-15 | 3 | SIL heater fixed 0.5 W | IMP | Closed by design change | SIL heater V²/R with 207 Ω |

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
| AV-01 | 2 | No OBC schematic; firmware pin map provisional and in conflict with OBC_DESIGN | IMP | Closed by design change | OBC board build-to design SQSO-OBC-BRD-001 (356 parts, ERC 0) and the master pin allocation SQSO-ICX-001: every pin one function |
| AV-02 | 3 | No external supervisor; orphan watchdog-kick line | IMP | Closed by design change | TPS3850H33-Q1 window-watchdog supervisor (5.6 s to 33-45 s window), backup to the 30 s internal watchdog; kick in flight software |
| AV-03 | 1 | No driver or pins for the gyro, magnetometer, sun sensors, magnetorquers; no sensor-rail latch-up switch | IMP | Verification item | Drivers mg370.rs, bmi088.rs, rm3100.rs, ads7138.rs, mtq.rs, adcs_sensors.rs (SRS-F140, F141, F164) on the ICX D pins (pins.rs, C test_hal_pins), unit tests per driver; VI: EM OBC bring-up (TP-FLATSAT-001): each driver reads its part's ID and a self-test passes, coil current within 10 % of the commanded duty, a forced rail trip latched and restarted, 2027-06 to 2027-07; confirmed by the round-3 panel (P3F-A) |
| AV-04 | 2 | GNSS chain missing | CD | Closed by design change | LPUART1 PG7/PG8, GNSS_EN PE5, gnss_schedule.rs; receiver trip VI AV-V8 |
| AV-05 | 2 | No TONE_OK or XPDR_KEY readbacks | IMP | Closed by design change | TONE_OK PC9, XPDR_KEY PG12, PTT_EXCITER PC7 (timer capture) with pull-downs (SQSO-ICX-001) |
| AV-06 | 2 | One software state bit drives both Jetson switches, no readback | IMP | Closed by design change | PAYLOAD_EN and JETSON_PWR on separate lines with JET_PG readback; JET_3V3 high only when the whole chain is up (SQSO-JET-BRD-001, SQSO-ICX-001) |
| AV-07 | 2 | No SRAM parity handler, TMR or scrubbing | IMP | Closed by design change | Statics in SRAM2 with parity, NMI handler for parity, flash ECC and clock security, 60 s FRAM cross-check of the TX-OFF, operations and passivation records (SRS-F132, F133; MEMORY_POLICY §10); RAD-V6; option byte AV-12, stack AV-13 |
| AV-08 | 2 | OBC and radio rail latch-up trips not specified | IMP | Closed by design change | Current-limited auto-retry rails: OBC 0.15 A, radio 0.12 A, sensors 0.30 A, GNSS 0.20 A, EPS controller 75 mA, AntS 1.37 A, PA 0.99 A (SQSO-EPS-BRD-001) |
| AV-09 | 2 | Idle USART1 TX can back-power the unpowered Jetson | IMP | Closed by design change | UART through SN74LVC1T45-Q1 buffers powered from the carrier's 3.3 V: high impedance when the carrier is off |
| AV-10 | 3 | No mission clock across resets | IMP | Closed by design change | RV-3028-C7 RTC on the always-on rail, its own I2C bus; time-stamps every reset |
| AV-11 | 1 | XPDR_EN was a level | CD | Closed by design change | 100 Hz heartbeat, 60 ms lease, TIM7 on PE4; test_xpdr_heartbeat.c; SIL radio |
| AV-12 | 2 | SRAM2 parity is enabled only by an option byte that the flight software never writes | IMP | Closed by design change | RELEASE_PROCEDURE §5.3.1 step 7 writes and verifies SRAM2_PE; boot reports parity (firmware parity.rs, SRS-F154, HK2 PE); checked at the first FM load; confirmed by P3F-A |
| AV-13 | 3 | Stack in SRAM3, which has no parity | IMP | Closed by design change | Stack in SRAM2 (memory.x from 0x20030000, statics + 8 KB stack ≤ 0x20040000 asserted), CI image check SP 0x20040000; worst-case stack 5,812 of 8,192 B from the machine-code analysis; confirmed by P3F-A |

#### Flight software (FSW)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| FSW-01 | 1 | On the target, a silent EPS controller skipped the detumble and reported it complete | CD | Closed by design change | executive.rs run_adcs: host-only stand-in |
| FSW-04 | 2 | SRS lacked transponder, commissioning and ADCS requirements; SRS-F081 undefined | CD | Closed by design change | SRS 1.6: SRS-F080..F094, F100..F107 |
| FSW-05 | 2 | transponder.rs and ops_state.rs meet the safety-critical criteria but are not listed | IMP | Closed by design change | transponder.rs and ops_state.rs (fsw and ffi) on the safety-critical list (SRS-Q013): 26 files, 0 problems, 3 reviewed exemptions |
| FSW-06 | 2 | SWE-220 cyclomatic complexity not measured for the Rust | VI | Decision point (dated) | DA-16 (flight software lead, 2026-12-15): cyclomatic complexity ≤ 15 on the safety-critical files, gated, or a CR |
| FSW-07 | 2 | No end-of-life passivation telecommand | IMP | Closed by design change | With REG-01 |
| FSW-08 | 2 | Software FMEA scales inverted | CA | Closed by analysis | SOFTWARE_FMEA B §14 |
| FSW-09 | 2 | No recovery from a receiver SEFI | IMP | Closed by design change | RADIO_EN power cycle after 24 h of silence, never in a pass (SRS-F115); SIL 7 of 7 hangs recovered; input lines RFB-07 |
| FSW-10 | 2 | NPR 7150.2D Appendix C mapping missing | CD | Closed by design change | SQSO-FSW-012 |
| FSW-11 | 3 | 3·10⁻⁴ nutation gain fails without a gyro | CD | Closed by design change | NUTATION_GAIN_NO_GYRO = 1·10⁻⁴ |
| FSW-12 | 3 | Three documents name different magnetometers and gyros; L293D cannot run at 3.3 V | CD | Closed by design change | ADCS_DESIGN 1.5; ICD_OBC_ADCS §12 |
| FSW-13 | 2 | OBC selector and transponder scheduler had no design | CD | Closed by design change | SQSO-PAY-SEL-001 (design complete; implementation post-CDR) |
| FSW-14 | 3 | Repeated health-check faults log every 10 s: a low-SOC episode fills the 100-entry fault log in 17 minutes | IMP | Closed by design change | condition.rs (SRS-F153): onset and clearance with a count; tests onset_repeats_and_clearance, a_long_episode_leaves_older_entries_intact; SIL F full run logs each low-SOC condition once; confirmed by P3F-A |

#### Radiation (RAD)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| RAD-01 | 2 | Margin on 10 krad is 1.90×, below 2× | CH | Closed by chair decision | BASELINE §6: ≥ 2.1 mm Al-equivalent by sector analysis |
| RAD-02 | 3 | SIL dose model low by 0.55-0.64 | IMP | Closed by design change | SIL dose model to RADIATION_ANALYSIS §4; full run 4.9 krad |
| RAD-03 | 3 | Upset rate and mitigation claims unsupported | CA | Closed by analysis | RADIATION_ANALYSIS C §5 |

#### Guidance, navigation and control (GNC)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| GNC-01 | 1 | Magnetometer sampled with the coils on | CD | Closed by design change | 900 ms on / 50 ms settle window; coupling in the Monte Carlo |
| GNC-02 | 1 | Computed inertia: Iyy/Izz 1.03-1.04 and products tilt the axis | CD | Closed by design change | Full tensor in the law and the SIL; 8°/s, k_n 3·10⁻⁴ |
| GNC-03 | 2 | Flexible 2 m antenna tapes not modelled | VI | Decision point (dated) | DA-17 (ADCS lead): flexible-tape Monte Carlo by 2027-02-26, EM modal check before the CDR; ≥ 90 % of sunlit time within 15°, measured mode within ±20 % |
| GNC-04 | 2 | Magnetometer not selected, no driver | IMP | Closed by chair decision | Magnetometer: PNI RM3100 on the OBC board, bought-module footprint as fallback (chair accepts OBC_BOARD_DESIGN §24 C1); driver in the flight software |
| GNC-05 | 2 | Spin axis and inertia selectable for one wing | CD | Closed by design change | ADCS_SET_GEOMETRY 0x24; SRS-F094 |
| GNC-06 | 3 | Rod remanence not modelled | VI | Verification item | AV-V4 at the flatsat: bias repeatability ≤ 300 nT per axis, rod remanence ≤ 0.18 mA·m² or an 8-state table, Monte Carlo re-run before the CDR |
| GNC-07 | 3 | ADCS Monte Carlo plant rows still the MASS_BUDGET 3.0 values; at Iyy/Izz = 1.000 only 73 % of sunlit time is within 15° | IMP | Closed by analysis | adcs_mc_cdr.sh plant rows to MASS_BUDGET 3.2 (CBE 2.853/4.202/3.979, MEV 3.276/4.760/4.546 × 10⁻³ kg·m² with products); re-run CBE 1.8° mean, p95 7.0°, 100 % within 15°; MEV 1.7°, 6.8°, 100 % (ADCS_DESIGN 1.8 §11.4, BUILD_VS_BUY D §6.4); the gain sweep's stale row: P3F-G-04; confirmed by P3F-G |
| GNC-08 | 2 | Gyro correction matrix needed (VI-MCAD-05) and the magnetometer unusable for actuation during Jetson bursts (JETB-09) | IMP | Closed by design change | Gyro alignment matrix by ADCS_SET_GYRO_ALIGN 0x25 with CRC, data only (SRS-F142); magnetometer gated on JET_PG or PTT_EXC (SRS-F143); per-polarity bias table 0x26 (SRS-F144); tests calibration_commands_persist_and_validate, the_magnetometer_is_gated; VI-MCAD-05 measures the matrix on the flight unit; confirmed by P3F-G |

#### RF board (round 2) (RFB)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| RFB-07 | 2 | The RF board gives the OBC no carrier-detect or receiver-health line, so receiver supervision (FSW-09, REL-04) has no inputs | IMP | Closed by design change | RX_CD (CC1200 carrier sense) and RX_FAULT (TNC heartbeat monostable, fault 0.36-0.62 s after the last pulse) on J1-15/16 (SQSO-RF-BRD-001 B); OBC PG0 and PF13 with pull-downs (SQSO-ICX-001 B); firmware pins in round 3; VI-RFB-10 |
| RFB-01 | 2 | Transponder draws 1.76 W DC at 0.5 W RF, not the baseline's ≈ 1.5 W | CH | Closed by chair decision | BASELINE §5 allocation 1.8 W DC / 0.44 W orbit average; margins +32 %/+75 % (allocation), +53 %/+29 % (bottom-up) |
| RFB-02 | 3 | Receive SAW drift and clock harmonics at 435.000 MHz constrain the uplink | CH | Closed by chair decision | BASELINE §7 Uplink: request 435.050-436.500 MHz |
| RFB-03 | 2 | Analog FM receivers discontinued: repeat audio passes through the TNC firmware | IMP | Closed by design change | TNC firmware a Class C item (SDP 1.4 §3.4, NPR 7150.2D matrix §2.4) |
| RFB-04 | 3 | Unkey ≤ 118 ms after a stalled main loop against VI-RF-11's 100 ms | IMP | Closed by design change | Heartbeat lease 40 ms, drop within 90 ms (SRS-F081); HIL-040 measures the pass time |
| RFB-05 | 3 | TCXO two-year ageing up to 2.1 ppm | CH | Closed by chair decision | Ground corrects the uplink from the measured downlink offset (same reference); no new command |
| RFB-06 | 3 | Radio rail trip 0.25 A cannot see a 38 mA latch-up; LMX2571 margin at ±3 % rail | IMP | Closed by design change | Radio trip 0.12 A with current telemetry; 3V3_AO ±2 % (≥ 3.22 V at the RF connector) (SQSO-EPS-BRD-001) |
| RFB-08 | 3 | Magnetometer A2 is 4 mm above the RF board: shield, lids, ferrites, crystals and the PA loop near it | CD | Closed by design change | No part within 20 mm of A2, no ferromagnetic part within 40 mm; nickel-silver shield, soldered lids; PA supply paired on layers 4/5 and routed 80-87 mm away (0.5 nT keyed step) (SQSO-RF-BRD-001 B §10.1); VI-RFB-11 |
| RFB-09 | 2 | SMA end-launch connectors J3-J6 with mated plugs and cable bends do not fit the 3.3 mm gap to the side walls | IMP | Closed by design change | MMCX jacks (Molex 73415-1473, brass/BeCu, gold) at the −X edge; 1.37 mm coax with silver-plated copper inner conductor down the −X wall gap at \|y\| 20-24 mm; port-to-antenna 0.44 dB (146 MHz) and 0.62 dB (436 MHz); uplink system noise 548-598 K inside 650 K (SQSO-RF-BRD-001 C §4.3, §16) |
| RFB-10 | 3 | AntS RF port type and the connector mated heights not confirmed from the ISIS ICD and the Molex/Samtec drawings | VI | Decision point (dated) | Owner RF lead: VI-RFB-12 by 2026-11-13 before layout; VI-RFB-13 via the ISIS AntS RFQ (out by 2026-11-06), closed 10 working days after the response and before the EM buys 2027-01-15 (RF_BOARD_DESIGN D §15) |
| RFB-11 | 2 | CAD models two AntS elements; the AntS and the RF board use four (2 m along ±X, 70 cm along ±Y): the deployed inertia ratio must be re-checked | IMP | Closed by design change | Four AntS elements in the CAD; with the C-13 ballast the deployed ratio is 1.056 / 1.047, ≥ 1.03 with growth to 3.2 mm placement scatter (MECHANICAL_DESIGN B §5) |
| RFB-12 | 3 | W10 coax top bend 7 mm is 5.1 × OD, below the 6 × OD workmanship rule | CH | Closed by chair decision | Accepted by analysis: formed once and tied, 1.75 × the Taoglas PRT.001221 minimum static radius (4 mm), no RF effect at 436 MHz; conditions: procured cable minimum static radius ≤ 5 mm (VI-RFB-12), bend inspection (V-HAR-10), return loss after forming and vibration (VI-RFB-13) |
| RFB-14 | 2 | TX-OFF removes PA_5V only: the CC1200 stays powered and an accidental full-power transmission could leak −35 to −45 dBm to the antenna ports through the receive chain, above the −60 dBm TX-OFF limit; the LMX2571 "verify" after rewrite had no stated readback path (found by the TNC firmware SRS) | IMP | Verification item | LMX2571 readback over the existing MUXout line (R39 SDO_LD_SEL) with PTT low; key-up sequence CE high, ≥ 100 µs, write all, R0 FCAL_EN, read back and compare (second mismatch aborts), lock detect, PTT (TNC-FW-LMX); frequency and FSK registers rewritten every 5 s while keyed; CC1200 never sent STX, PA at minimum (−16 dBm: leakage below −60 dBm), no automatic transition into TX, 1 s MARCSTATE check (RF_BOARD_DESIGN F amended §7.7, §13); VI-RFB-16 on the EM measures leakage at full and minimum PA power with TX-OFF, pass ≤ −60 dBm at J5/J6, fallback remove L206 after TI confirms the receive-only use |

#### EPS board (round 2) (EPSB)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| EPSB-01 | 2 | Firmware fires driver A on both wings at once: 1.25 A at the pack | IMP | Closed by design change | Four lines, one resistor at a time, 10 s (deployment.rs, SRS-F089; tests full_sequence_one_element_at_a_time, C test_full_sequence, test_wing_retries_alternate_drivers); SIL burn-lines 1 |
| EPSB-02 | 3 | Uplink must avoid the synchronised converters' harmonics | CH | Closed by chair decision | BASELINE §7 Uplink: 435.222-435.778 MHz, else 436.222-436.500 MHz |
| EPSB-03 | 2 | EPS controller (MSP430FR2355) firmware is a new configuration item | IMP | Closed by design change | EPS controller firmware a Class C item (SDP 1.4 §3.4) |
| EPSB-04 | 3 | Tracker efficiency 90.9-92.3 % against EPS-F002's 92 % | VI | Decision point (dated) | With P3E-09 (CR-2026-014) |
| EPSB-05 | 3 | ITV4030 fuses rated from −10 °C, below the board's −30 °C allowable | VI | Verification item | V-EPS-8 qualification |
| EPSB-06 | 3 | Cross-document consistency after the EPS board (B2, diagram README, radiation trip table, BSA H8a, ICD-001, EMC E6/E8, IARU uplink windows) | IMP | Closed by design change | Consistency pass: B1-B4 redrawn with DS3, diagram findings F1-F17 dispositioned, RADIATION D §5.3 trips, BSA B H8a, ICD-001 1.2, EMC B §5.1 (E6, E8), IARU request A.3 windows; SVGs rendered |
| EPSB-07 | 3 | No magnetorquer current monitor on the EPS board | CH | Closed by chair decision | Coil current is the sum of the OBC's MTQ_X/Y/Z_I channels; SW11 (TPS1H000A, 0.361 A, auto-retry) protects VBAT_MTQ (SQSO-EPS-BRD-001 B) |
| EPSB-08 | 2 | Pack leads unfused (80-314 A short-circuit current); the Nano2 fuses' breaking capacity below the worst-case current | CD | Closed by design change | F0 added at the cell-4 tab, F0 and F2 MIL-PRF-23419/12 FM12A72V5.0A (1000 A interrupt, 3.2× the 314 A bound), tap fuses FM12A72V3/4A (BATTERY_DESIGN 2.6, EPS_BOARD_DESIGN C §6.1, BSA C H3/H3a/H3b) |
| EPSB-09 | 3 | FM12 interrupt rating, 60 °C rerating and termination code, and the SMF24A leakage, not read from the vendor data sheets | VI | Decision point (dated) | DA-18 by 2027-01-15: FM12 interrupting rating ≥ 209 A, 60 °C rerating, SMF24A drain, E595 per part number from the manufacturer data sheets |
| EPSB-10 | 3 | Shorts in the pack leads and cell taps (BSA H3b) are controlled by one fuse plus construction, not two-fault tolerant | CH | Closed by chair decision | C-16: F0A and F0B from different lots, each able to clear the fault alone, count as two independent controls with F0N the third: H3b 2-fault tolerant; NRCSD safety-engineer concurrence for the ISS backup path at the NRCSD contract (BSA D) |

#### OBC board (round 2) (OBCB)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| OBCB-01 | 1 | PAYLOAD_EN on PB4: the JTAG pull-up at reset switches the Jetson rail on at every OBC reset | IMP | Closed by design change | PAYLOAD_EN on PB2, driven low at reset (firmware gpio.rs, pins.rs, C test_hal_pins); confirmed by P3F-A |
| OBCB-02 | 2 | Twenty-five firmware pin changes and new drivers (gyros, magnetometer, sun-sensor ADC, coils on TIM1/TIM15, RTC, supervisor kick) | IMP | Closed by design change | Every SQSO-ICX-001 D §6 row in the firmware and the HAL (SRS-F140 to F152); firmware clippy, release build, image and stack checks pass; VI: TP-FLATSAT-001 pin continuity and each driver's bring-up, 2027-06; confirmed by P3F-A |
| OBCB-03 | 2 | Gyro module needs 14 mm above the OBC; the stack gives 11.4 mm | IMP | Closed by design change | Gyro on the OBC through a carrier window (MCAD-04, MECHANICAL_DESIGN D-11) |
| OBCB-04 | 2 | No carrier board design for the Jetson (its 3.3 V I/O rail, VBAT input) | IMP | Closed by design change | Jetson carrier build-to design SQSO-JET-BRD-001 (137 placements, ERC 0): 5.016 V buck and eFuse to VDD_IN, switched JET_3V3, translated UART |
| OBCB-05 | 3 | Magnetorquers draw 1.16-1.3 W in detumble; not in the power analysis | IMP | Closed by design change | POWER_BUDGET_ANALYSIS 2.1 §8.6 LEOP case with the magnetorquers at 1.16-1.3 W: first orbit −1.30 Wh, three orbits −3.44 Wh (SOC 40 % at separation falls to 33 %) |
| OBCB-06 | 3 | OBC-rail fault that does not trip SW10 goes undetected | CH | Closed by chair decision | Accepted: the external supervisor and the internal watchdog cover a hung OBC; an EPS-side heartbeat check is a fabrication-phase option |
| OBCB-07 | 3 | Gyro supply margin 3.195 V against 3.15 V minimum | VI | Verification item | V-OBC-12: gyro VCC ≥ 3.15 V at maximum current with 3V3_AO at −2 %, at −20 and +60 °C |
| OBCB-08 | 2 | Gyro module has no published random-vibration rating; it sees 22-25 Grms at the module | VI | Verification item | V-OBC-16: random vibration ≥ 22.2 / 25.0 / 21.7 Grms at the module and launch shock, bias change ≤ 0.01 °/s, scale factor ≤ 0.2 %; Epson qualification data requested (OBC item C12); fallback grommets and a flexible jumper |
| OBCB-09 | 3 | Two more lines from the powered radio raise the current into an unpowered OBC to 2.7 mA | VI | Verification item | V-ICX-3 / V-OBC-15 limit ≤ 3.0 mA |

#### Jetson carrier board (round 2) (JETB)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| JETB-11 | 2 | SIL Jetson model spends 0.036 Wh (130 J) per AI beacon; the carrier design gives 7.2-7.5 W at the input and about 424 J per line for a full burst | IMP | Closed by design change | SIL Jetson at the carrier input 7.19–7.48 W (SIL-26); energy gate ends a burst (SRS-F151); SIL F full run 8,145 Wh over 126,292 accepted beacons, 0.064 Wh each against 0.1 Wh (mean under assumed inputs; go/no-go on the EM, P3F-G-03); confirmed by P3F-G |
| JETB-01 | 2 | Jetson burst draws 7.7-8.0 W at the carrier input in the 7 W mode: no margin to the 8 W allocation | IMP | Closed by design change | sqso-platform.service.j2 (curr1_max 1300 mA, nvpmodel ID 1), tegra234-p3767-sqso-pcie-c4.dts (PCIe Gen2 x1), no carrier EEPROM; VI V-JET-4 on the EM carrier: VDD_IN 2 s average ≤ 1.3 A capped by the limit, carrier input ≤ 8.0 W, 2027-07; confirmed by P3F-G |
| JETB-02 | 2 | Heat-spreader temperature has no sensor path with the carrier off (guard 70 °C, power-on gate −15 °C) | IMP | Closed by design change | Carrier passes the spreader thermistor through unpowered on J3 pins 5-6; OBC PA2 (ADC1_IN7) with a 10.0 kΩ 0.1 % bias from VDDA, ±0.24 K at −15 °C (SQSO-OBC-BRD-001 B §11.3, SQSO-ICX-001 B E10) |
| JETB-03 | 2 | Module needs 8N2 from the OBC, a commanded SHUTDOWN before power removal (hold-up 0.11 ms), JPC from IMON_JET | IMP | Closed by design change | USART1 8N2 in the firmware; SHUTDOWN, JET_PG fall ≤ 5 s, JETSON_PWR then PAYLOAD_EN 10 ms apart, unexpected fall a fault (jetson_power.rs, SRS-F148/F149, tests); JPC from IMON_JET; VI V-JET-1 on the EM carrier, 2027-07; confirmed by P3F-G |
| JETB-04 | 3 | A full 10-minute burst costs about 424 J per beacon line against the 360 J gate | CH | Closed by chair decision | The energy gate, not the 10-minute limit, ends a burst; the generated-text experiment stays gated (decision D5); templates are the baseline |
| JETB-05 | 3 | Carrier mechanics: spreader pedestal, 0.05 mm flatness, module standoffs, flush fastener at (0, −21) under the module | IMP | Closed by design change | Pedestal 22 × 22 × 0.75 mm, flatness 0.05 mm, shims at integration (VI-MCAD-06), standoffs MP2/MP3, low-head fastener at (0, −21), clamp 38 N of 80 N (MECHANICAL_DESIGN §4.5); carrier D: window keep-outs by side, frame screws H10-H13 on tapped pads, MP2 assembly notes (SQSO-JET-BRD-001 D §18.1) |
| JETB-06 | 3 | NVIDIA Product and Thermal Design Guides not obtained: POWER_EN to SYS_RESET* timing, PCIe AC-coupling, FORCE_RECOVERY*, routing, lid θjc | DP | Decision point (dated) | DA-19 (with JETB-12): NVIDIA design guides by 2027-02-01, before carrier layout 2027-03-16 |
| JETB-07 | 3 | Some carrier part numbers confirmed only from distributor listings | VI | Decision point (dated) | With EPSB-09 (DA-18) |
| JETB-08 | 3 | NVIDIA screens the module at 2 Grms only; flight level is 8.18 Grms | VI | Verification item | V-JET-8: random vibration of the populated carrier at protoflight level, functional test before and after |
| JETB-09 | 2 | If the module's VDD_IN pins are at the SO-DIMM +X end, the 1.4 A burst current passes 12 mm from the RM3100 (order 10 µT) | IMP | Closed by design change | Magnetometer not used for actuation while JET_PG is high (adcs_control.rs, SRS-F143, test the_magnetometer_is_gated); the SIL drives JET_PG and PTT and adds the burst field at the magnetometer; confirmed by P3F-G |
| JETB-10 | 3 | SO-DIMM latch material unknown (TE); carrier magnetic signature unmeasured | VI | Verification item | V-JET-9: ≤ 0.5 mA·m² unpowered; ≤ 15 nT change at A2 during a burst |
| JETB-12 | 3 | Mated heights of the carrier's J1 and J2 not yet read from the Molex drawings (limits 13.2 mm and 7.7 mm) | VI | Decision point (dated) | DA-19: Molex drawings (J1 ≤ 13.2 mm, J2 ≤ 7.7 mm) and the NVIDIA guides by 2027-02-01 |
| JETB-13 | 3 | Spreader thermistor curve from a third-party Steinhart-Hart fit of the 10K3A material, not TE's table | VI | Verification item | V-JET-7: EM thermistor calibrated ±1 K at −15 and +70 °C; flight thermistor checked at FM carrier acceptance |

#### Round 3: programmatics, SE, V&V (P3P)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| P3P-01 | 2 | No single register of verification items; dated pre-layout items not in the schedule | IMP | Closed by design change | SQSO-VIR-001 generated by gen_vi_register.py (205 items from 20 documents, criterion and dated event checked), in CI and tests; linked from VCRM §5 |
| P3P-02 | 2 | Class I changes made without change requests; CR-007/-008 lack PI approval | IMP | Closed by design change | CR-2026-009 to -015 cover SYS E/F/G, ICD-013/-001/-005/-010, C-10, ETP deviations, MPPT and cost; PI decision by 2027-01-12 |
| P3P-03 | 2 | Reliability test programme (flatsat, FM burn-in) not in the ETP, schedule or VCRM | IMP | Closed by design change | ETP D §3.11, VI-11 and VI-12; CPM tasks FLATSAT_500H and FM_BURNIN |
| P3P-04 | 2 | Hardware FMEA not updated for the round-2 boards | IMP | Closed by design change | Board-level FMEA SQSO-FMEA-003 A: 176 failure modes, 59 mission SPFs (36 new, CIL-14 to CIL-28), 19 findings SMA-B01 to B19 dispositioned separately; SQSO-FMEA-002 D; V-BFM-01 to -08 |
| P3P-05 | 2 | Independent PDR has no board or owner; layout duration predates the design | IMP | Decision point (dated) | DA-03 PDR board (PI, 2026-12-01); DA-04 two layout designers (PI, 2026-12-15); layout re-estimated at 27 designer-weeks; CPM float +71 d at 2028-11-01 |
| P3P-06 | 3 | Criteria matrix, VCRM §6, hardware readiness, risk register and CPM not updated after design completion | IMP | Closed by design change | Criteria matrix B, VCRM D §6, hardware readiness tracker 2.1, risk register E (R16, R23, R24, R29), CPM SCH and MECH complete |
| P3P-08 | 3 | Cost inconsistencies (launch cost row, custom boards, PCB fabrication, reviews, shock and EMC tests) | IMP | Closed by design change | ROM F: custom boards, layout, reviews, abuse tests; I&T $42k-130k from test_cost.py; BASELINE §3 launch cost in CR-2026-015 |
| P3P-09 | 3 | Life-cycle cost estimate outside document control | IMP | Closed by design change | SQSO-AUD-001 to -005 registered; the ID checker excludes only research/, with a test |
| P3P-10 | 3 | Date conflicts: early FCC filing, IARU letter, JETB-06 vs layout | IMP | Closed by design change | Early FCC filing 2027-01-19 everywhere with a CPM task; IARU letter by 2027-05-18; carrier layout from 2027-03-16, after JETB-06 (2027-02-01) |
| P3P-11 | 3 | CR-2026-008 mass not propagated to LVR and orbital access plan | IMP | Closed by design change | LVR-100 and LVR §9 at 1.75 kg; OAP §4.1.2 by the launch fixer |
| P3P-13 | 3 | Stale TBR-CDS-005 (switch contact form now defined) | IMP | Closed by design change | TBR-CDS-005 closed (changeover contacts, DS2 two-pole, DEPLOY_SENSE PE2); TBR baseline 32 |

#### Round 3: power, safety, EMC (P3E)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| P3E-01 | 2 | Solar array path has only two independent inhibits | IMP | Closed by design change | With EPS-04 |
| P3E-02 | 2 | External-short fault tolerance overstated (RT1 cannot break 314 A; F0 and F2 are one part type) | IMP | Closed by design change | F0A, F0B (two lots) in BAT_TOP and F0N in BAT_BOT, FM12A72V5.0A; worst short ≤ 209 A against 1000 A; taps limited to ≤ 82 mA by 3 × 68.1 Ω plus a fuse; H3 2-fault tolerant without RT1 (BSA D, BATTERY_DESIGN 2.7); H3b per chair decision C-16 |
| P3E-03 | 2 | Battery case incomplete against JSC 20793 Rev D (Catastrophic class; abuse tests for a custom pack) | IMP | Closed by design change | BSA D: custom 4S Li-ion classed Catastrophic (JSC 20793 Rev D §4.1.3, to be confirmed with the NRCSD safety engineer); abuse-test programme §3.1 and V-BSA-9: Falcon 9 path (overcharge, over-discharge, pack hard/soft short with limiters bypassed, vibration; vent/burst by analysis) about $10-15k, ISS backup full set about $50-75k |
| P3E-04 | 2 | Launch compliance documents contradict the closed inhibit design | IMP | Closed by design change | With P3A-02: LVR-067/069/101/102 and CDS 2.3.x describe DS1-DS3 on pack and array |
| P3E-05 | 2 | JETSON_PWR pin inconsistent across documents (PA2 vs PA8) | IMP | Closed by design change | JETSON_PWR PA8 in EPS_BOARD_DESIGN D, EPS_DESIGN 2.4, ICD-001 1.5, CARRIER_BOARD_DESIGN E, ICD-002 1.4, PAYLOAD_DESIGN 1.2 |
| P3E-06 | 3 | Load-shed coverage mis-stated | IMP | Closed by design change | With EPS-09 |
| P3E-07 | 2 | Reliability programme absent from the test plan (with P3P-03) | IMP | Closed by design change | With P3P-03 |
| P3E-08 | 3 | Power margin bookkeeping mixes formulas; SYS-100 and EPS-F001 at 1.80 W | IMP | Closed by design change | SYS-100 2.23 W in CR-2026-009; report §3.3 margins against the bottom-up load |
| P3E-09 | 3 | MPPT efficiency requirement and pass criteria disagree | IMP | Decision point (dated) | CR-2026-014: ≥ 90 % at every point ≥ 1 W per tracker, ≥ 91 % power-weighted, PBA MPPT factor 0.91; PI decision by 2027-01-12 |
| P3E-10 | 3 | Stale harness gauges and fuse values in EPS and harness documents | IMP | Closed by design change | Harness and EPS documents at HAR-001 2.0 gauges (pack 22 AWG, strings and taps 28 AWG) and the FM12 fuses; tap shorts limited to 82 mA; zones under the 2.0 mm EPS lowered; harness_calcs --cad --wireviz clean |
| P3E-11 | 3 | End-of-life arm and fire both from one processor | IMP | Closed by design change | Arm latch clocked only by OBC EOL_ARM AND the EPS controller's keyed EOL_KEY (U64; register 0x7E, ICD-001 1.5); fire from OBC EOL_FIRE; no single hardware or OBC pin fault fires F1 (EPS_BOARD_DESIGN D §12); flight software sequence in round 3 |
| P3E-12 | 3 | Shipping state of charge inconsistent (65 % vs 30 %) | IMP | Closed by design change | ≤ 30 % SOC by air (IATA, strongly recommended for PI 967), 60 % integration state by road, final charge 60 ± 5 % at the integrator: BSA D §6, MISSION_TIMELINE 2.4, LAUNCH_PROVIDER_DELIVERABLES D |
| P3E-13 | 3 | Record inconsistencies (BMS part, stuck-on Jetson hours, cold-burn bus voltage, SIL F1 heater, third harmonic) | IMP | Closed by design change | BQ76952 throughout, stuck-on Jetson 66.6 h, cold-burn bus 14.1-14.4 V at 0.75 A (13.8-14.1 V at 1.0 A), EMC D third harmonic ≥ 70 dB, SIL F1 heater 15.6 Ω nominal (11.2-20.0 Ω) to the flight software |

#### Round 3: structures, mechanisms, thermal (P3S)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| P3S-01 | 2 | Mechanism cold design and test point (−40 °C) warmer than the predicted stowed-wing temperature | IMP | Closed by design change | Burn release at −41 °C (resistor after a 3 h soak): 8 s ratio 1.66, 10 s 2.07; thermal analyst gives the burn-time temperatures (THERMAL_ANALYSIS 2.2 §4.3): every margined node inside −55 °C by ≥ 3.3 °C (resistor −39.4 °C); mechanisms at −55 °C: torque margin +0.08 (one spring out +0.07), springs +0.12/+0.04 yield; burn 10 s (C-17) gives release-energy ratio 1.90; VI-STR-04/-05/-06, VI-MECH-01..03 at −55 °C; hinge.py T_COLD for the thermal analyst's hinge temperature |
| P3S-02 | 2 | Board fatigue computed at CBE mass only | IMP | Closed by design change | Steinberg at MEV masses: EPS board 2.0 mm (D-14): +0.14 to +0.50 at MEV, +0.42 to +0.71 at CBE; strength +0.96 (MEV +0.85); first mode 170 / 162 Hz |
| P3S-03 | 2 | EPS board layout guidance lacks the centre supports and the Z-rod notch | IMP | Closed by design change | EPS board outline: centre supports at (0, ±21) mm with Ø7 mm copper-free rings, Z-rod notch x < −35.5, \|y\| < 6.5 with 1 mm keep-out (EPS_BOARD_DESIGN D) |
| P3S-04 | 2 | No humidity control or ground cycle budget for the MoS2 hinges | IMP | Closed by design change | MoS2 hinge controls: ≤ 40 % RH or dry N2, cycle budget ≤ 30 / ≤ 10, film inspection (MECHANISMS_ANALYSIS E §5A; ETP D §3.8) |
| P3S-05 | 3 | Report §3.3 strength and battery-temperature rows optimistic | IMP | Closed by design change | Report §3.3: strength +1.00 design / +0.84 bounding; battery temperature rows at 12.0 V, 0.10 W/K and wings stowed |
| P3S-06 | 3 | Launch mass deliverables stale | IMP | Closed by design change | LVR-004/005/025/026 at the CAD and MEL 3.1 (1.75 ± 0.26 kg); ICD-010 G |
| P3S-07 | 3 | Stale mechanical text (mechanisms, MEL rows, FSW inertia) | IMP | Closed by design change | MECHANISMS §1, §6-§8, MASS_BUDGET §6.2, mass_properties.py FSW and SIL constants updated |
| P3S-08 | 3 | VI-STR-10 has two pass criteria | IMP | Closed by design change | VI-STR-10 one criterion: flight order ≥ 10 mm from both deployed wings; contingency ≥ 5 mm from a stowed wing (CAD 6.5 mm) |
| P3S-09 | 3 | Placement Monte Carlo and the C-15 bounding case not reproducible from scripts | IMP | Closed by design change | placement_mc.py reproduces §5.4; FE case design_harness_on_EPS (+0.38 with the 2.0 mm EPS) |
| P3S-10 | 3 | Thermal-vacuum cycle count and margin conflict between documents | IMP | Closed by design change | ETP D §3.4 one thermal-vacuum definition; SYS-707 Rev G; V-THM-6 restated in the VI register |
| P3S-11 | 3 | Trim masses not specified non-magnetic | IMP | Closed by design change | Trim masses W-Ni-Cu (ASTM B777, non-magnetic binder) or pure W (D-9, MATERIALS_LIST) |
| P3S-12 | 3 | Z rod clipped to two structures at the deck's peak response; clip loads not computed | IMP | Closed by design change | Z rod tied to the −X shear panel only (two clips, 19 N at 3σ, MS > +30); deck 247 Hz; deck and clip accelerometers in VI-STR-02, VI-MCAD-04 |
| P3S-13 | 3 | RID evidence numbers superseded | IMP | Implementation | Regenerate |

#### Round 3: launch, RF, ground, regulatory (P3A)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| P3A-01 | 2 | GNSS receiver powered inside the 45-minute RF silence | IMP | Closed by design change | First GNSS session at 46 min with a compile-time check (gnss_schedule.rs FIRST_SESSION_DELAY_MS, SRS-F084, test first_session_after_the_delay_then_a_fix_ends_it) plus the executive's !rf_silence_active() gate; confirmed by P3F-G |
| P3A-02 | 2 | Launch-compliance documents show pre-round-2 status | IMP | Closed by design change | LVR E (rows 004-109 to the design), CDS C, OAP D, superseded notice on CDR_TECHNICAL_BASELINE |
| P3A-03 | 2 | RF-board TNC firmware has no task, owner, requirements or hazard entry | IMP | Closed by design change | Requirements written for both board firmware items: SQSO-SRS-TNC-001 (67, 25 safety-critical) and SQSO-SRS-EPSC-001 (62, 20), each with a hardware-enforced table; hazards SW-HAZ-014 to -017 (SHA G §9.8); eleven schedule tasks (SCHEDULE E, CPM float unchanged +71 d, least new-task float 92 d), owner flight software lead (PI until named); SDP 1.6 §3.4, NPR 7150.2 matrix C (SWE-050/052/184/205 met); RTM and VCRM E; toolchains Rust no_std (TNC) and C11 MISRA (EPS controller) for PI ratification DA-22 by 2026-12-01 |
| P3A-04 | 3 | Telecommand variant B at its minimum is 0.2 dB under 64 dBm; station gain and pointing unverified | IMP | Closed by design change | Variant B ≤ 50 W PEP at the transmitter output into ≥ 17.8 dBic (64.1 dBm, +6.6 / +8.4 dB); VI-GND-01 ≥ 100 W at the feed; VI-GND-07 station EIRP, VI-GND-08 ±2° pointing (GND-001 G, LBA 3.5) |
| P3A-05 | 3 | TCXO ageing decision not flowed down | IMP | Closed by design change | RF-P005: ±1.0 ppm over temperature, ≤ ±3.0 ppm total over 2 years with ageing; ground scales the measured beacon offset by 435.5/145.9; VI-RF-14 (a)-(d) incl. offset estimation within ±20 Hz (RF_DESIGN 2.3) |
| P3A-06 | 3 | RF shield cans not vented | IMP | Closed by design change | One Ø1.5 mm vent per shield lid (SH2-SH4), V/A 3.2-9.4 m against 50.8 m; hole leakage ≥ 37 dB at 2.4 GHz, PA-to-LNA via vents ≈ 142 dB against 85 dB (RF_BOARD_DESIGN D §10) |
| P3A-07 | 3 | Operations texts contradict the design | IMP | Closed by design change | MISSION_TIMELINE 2.5, CONOPS, GND-001 G §5, ICD_OBC_RF 2.3 (40 ms lease, drop 87-98 ms) corrected |
| P3A-08 | 3 | Report §3.1-§3.3 numbers do not follow from their sources | IMP | Closed by design change | Report §3.1-§3.3 corrected: ODAR Draft D P 0.997, 86 LVR rows, storage SOC from the 7.7 µA drain, burn sequence, pass statistics (MISSION_TIMELINE 2.3), fire-first passivation, ODAR lifetimes |
| P3A-09 | 3 | 485 km cap not defined against injection dispersion; no 460 km quote | IMP | Closed by design change | Cap on the 3σ-high mean altitude or apogee, injection dispersion included; every RFQ asks for a 460 km quote (OAP D §2.2, LVR-087, ICD-010 G §9.1, BASELINE §3) |
| P3A-10 | 3 | SIL link model at the pre-C-14 station and wrong 70 cm tape axis | IMP | Closed by design change | SIL link at the C-14 station (64.0 dBm EIRP), 70 cm tapes along ±Y (link.rs, SIL-35); passes heard 95.7 %, commands 1,449 of 1,463 (SIL F); confirmed by P3F-G |

#### Round 3: avionics, flight software, GN&C (P3F)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| P3F-G-01 | 2 | RAD-V1 to V6 not in the VI register (the generator's pattern skips RAD-V): no owner or date; RAD-01 rests on RAD-V2 and the receiver's survival on RAD-V3 | IMP | Implementation | Generator pattern, owners, dates |
| P3F-G-02 | 2 | A missing AntS temperature counts as cold, so a temperature-path fault silences every transmitter for about 25 h (C-19); at the 30-deferral cap the release can fall at any orbital phase (−36 °C, −46 °C with margin); neither path runs in the SIL | IMP | Implementation | Cap release in sunlight; schedule fallback after repeated unknown readings; SIL cases |
| P3F-G-03 | 2 | Smart Beacon go/no-go criteria disagree with the design: SYS-512 and VI-02 bursts ≤ 120 s, D5 ≤ 960 J per burst, against BASELINE §6 and the code's ≤ 10 min and 3.6 kJ; MSC E-02 ≥ 10 bursts vs VI-02 ≥ 30; the flight gate caps energy per burst, not per accepted beacon | IMP | Implementation | One burst definition; gate on energy per accepted line |
| P3F-G-04 | 3 | ADCS performance record stale: §11.4 gain sweep not re-run (93 % vs 76.2 % at Iyy/Izz 1.000, C-4's rationale); BASELINE §8 quotes ADCS_DESIGN 1.5 figures; §11.3 preset axes not the code's | IMP | Implementation | Re-run the sweep; CR to BASELINE §8 |
| P3F-G-05 | 3 | Monte Carlo gaps: no magnetometer gating (coils off about 10 % of the time); wings-stowed case deploys the AntS at 30 min, not after the C-18 deferral; stowed-tensor products differ from MASS_BUDGET; AntS-pending interval and deployment transient not modelled; one-wing cases only with plant = flight model | IMP | Implementation | Add the cases |
| P3F-G-06 | 3 | The ground cannot tell which wing deployed (SOLAR_SENSE in series, only total array power), so it cannot choose between presets 1 and 2; a wrong preset is not analysed | IMP | Implementation | Identification procedure or preset-0 rule |
| P3F-G-07 | 3 | The ±5° attitude-knowledge requirement (ADCS-F003/P003, EKF/TRIAD) is not implemented; MRD-061 'verified' by a Monte Carlo that does not measure knowledge | IMP | Implementation | Restate or implement and verify |
| P3F-G-08 | 3 | RADIATION_ANALYSIS E does not match the selected parts (MM200 vs RM3100, load switches), omits the implemented mitigations, old dose model, Jetson upset probability 10× low, no latch-up statement, no 475/485 km table | IMP | Implementation | Revise against the parts list and the code |
| P3F-G-09 | 3 | V-JET-9's ≤ 15 nT burst limit is not traceable to a need (carrier bound 0.35 µT, magnetometer unused during bursts) | IMP | Closed by design change | V-JET-9 limit applies to the static field (carrier unpowered, when the magnetometer is used); the burst-induced field is recorded only with its nT/A coefficient, since the magnetometer is gated while JET_PG is high (SRS-F143); expected ≤ 0.35 µT (CARRIER_BOARD_DESIGN E §18.2) |
| P3F-A-01 | 2 | AntS deployed state read while its controller is unpowered (burn_off drops ANTS_PWR before process_verify): on hardware a deployed AntS is never confirmed, two more burns, FAILED, the C-19 hold longer; the SIL answers regardless of SW8, so ants_cold is not representative | IMP | Implementation | Power before every status read; SIL AntS answers only with SW8 on; re-run |
| P3F-A-02 | 2 | The PTT_EXCITER watch (SRS-F168) accepts any XPDR_KEY high without limit: a stuck key-gate or XPDR_KEY keys the exciter past the 120 s timer and blinds the watch to a stuck TNC_PTT; frequent overs reset the count | IMP | Implementation | Repeat path an explanation only with the heartbeat and ≤ 120 s plus margin |
| P3F-A-03 | 3 | The supervisor can reset the OBC up to 8.5 s before the IWDG after a stall (IWDG refreshed every pass, WDI edge every 10 s); obc_calcs ignores it, V-OBC-2 would fail; stale reload 3500 and 30000 values; SRS-F001 allows 30 s init beyond the boot IWDG's 24.1 s | IMP | Implementation | Refresh the IWDG with the WDI edge or ≤ 19 s |
| P3F-A-04 | 3 | C-18 deferral slot and force flag not persisted: each reset in ANTENNA_PENDING spends a deferral, so a reset loop exhausts the cap and deploys cold | IMP | Implementation | Persist the slot and flag |
| P3F-A-05 | 3 | ack and GNS formatters keep the 'too long gives empty' path; send_ui transmits an empty frame and marks it sent; GNSS PDOP unbounded | IMP | Implementation | Same fix as SIL-40; bound PDOP |
| P3F-A-06 | 3 | separation.rs and fram_health.rs (SH-008 controls) not on the safety-critical list or in safety_coverage.py | IMP | Implementation | Add them |
| P3F-A-07 | 3 | OBC heartbeat timeout and cycle limit inconsistent: EPS_DESIGN 120 s, ICD-001 600 s and 3 cycles in 24 h, INTERCONNECT E14 one 10 s pulse per 10 min | IMP | Closed by design change | Aligned to ICD-001 1.7: heartbeat at least every 60 s, 600 s timeout, SW10 off 1.95 ± 0.1 s, no cycle within 600 s of a start or the last cycle, at most 3 in 24 h (EPS_DESIGN SW10 row, INTERCONNECT E14, EPS_BOARD_DESIGN G §8.3 and V-EPS-16) |

#### Round 3: thermal and parts (P3T)

| RID | Sev | Finding | Round 1 | Final disposition | Evidence |
|-----|-----|---------|---------|-------------------|----------|
| P3T-01 | 2 | BOM temperature grades not verified part by part: the RF crystal Y451 was a −20…+70 °C option against the −40 °C first-orbit case | IMP | Closed by analysis | Parts temperature-grade audit of all 443 flight BOM lines against −40…+70 °C (SQSO-MFG-003 B, with CSV): 162 verified, 249 decoded, 20 not confirmed and fixed, 5 non-compliant, each dispositioned in P3T-02 to P3T-09; Y451 ABLS-L4-T; EEE_PARTS_LIST F §2A.9 lists every part below −40/+70 °C with its coverage |
| P3T-02 | 1 | EPS end-of-life fuses F1/F3 ITV4030L1412NR rated −10…+65 °C against the −40…+70 °C flight range (SQSO-MFG-003 §4) | IMP | Closed by design change | F3 and the BQ771806 moved to the battery sense board (fires only with charge current, refused below +1.1 °C at the pack, so always inside −10…+65 °C; S&MA: H1 still two-fault tolerant, a cold F3 overcharge needs three faults); F1 fired only with the EPS board ≥ 0 °C and in sunlight, waiting at most 6 h of each attempt (SMA-B20); current carrying below −10 °C: V-EPS-8 flight-lot test, 6 samples, 2 A for 1 h at −40 °C and +70 °C plus 5 cycles at 1.5 A, pass no opening and resistance change ≤ 10 %, trigger at 10.5/16.8 V at 0/+25/+65 °C in < 1 min, before FM integration (EPS_BOARD_DESIGN G, BATTERY_DESIGN 2.10) |
| P3T-03 | 2 | Bought items below the test range: ISIS AntS qualified −20…+60 °C, CubeSpace CR0002 rods −20…+80 °C (cold start −35 °C); NCR18650GA alternative cell and panel vendor ranges unconfirmed | IMP | Decision point (dated) | Vendor actions SQSO-MFG-003 B §12 with owner, date and fallback: item 1: ISIS AntS statement to −57 °C unpowered and its temperature telemetry (mechanical lead, 2026-12-15; fallback another antenna, chair decision before the EM order); item 2: CR0002 −40 °C rating (GNC lead, 2026-12-15; fallback flight-lot rod qualified at −40…+70 °C, ordered by 2027-01-15); item 3: NCR18650GA ranges (EPS designer/PI, 2026-11-30; fallback delete the alternative); item 4: panel vendor −56…+93 °C (EPS designer, 2027-01-15; fallback EM panel and wing qualification); item 6: piNAV-NG ranges (GNC lead, 2026-12-15; fallback receiver powered only above −30 °C) |
| P3T-04 | 2 | RF PA Q601 RD01MUS3 has no rated minimum operating temperature (storage −40 °C) | IMP | Verification item | VI-RFB-14: Mitsubishi statement of the minimum operating case temperature (RF lead, before the EM order 2027-01-15) and EM PA characterisation at −40 °C and +70 °C: output power within RF-P003 under the loop, DC power at 0.5 W ≤ 1.1 × room, no oscillation or spurious above −60 dBc 1–1000 MHz into 3:1 at all phases, bias settled ≤ 100 ms, 3f ≤ −70 dBc, 10 cold starts after a 4 h −40 °C soak; fallbacks keying only above −30 °C, then a CMX901-class PA with a board revision (RF_BOARD_DESIGN F §7.2) |
| P3T-05 | 2 | Fourteen BOM part numbers do not exist or are incomplete (OBC C23–C26 C0G 470 nF, A3 J1, RF L106/L605, carrier J1–J3/R1, EPS RS11 and others) and ordering codes are wrong (Samtec TFM "-S", tin Pico-Clasp 501334/501331, ERA-3 1 MΩ) | IMP | Closed by design change | Every flagged line orderable: EPS 1N4148WQ-7-F, ERJ-3EKF10R0V, ERA-6AEB105V/3653V on 0805, WSL2010R5000FEA, 3-digit ERA E24 codes, TFM-120-02-L-D-A; OBC C23–C26 → 4 × (2 × 220 nF C0G C1210C224J3GACAUTO) with the TPS3850 window re-derived (29.3 s minimum, IWDG reload 3200), D4, A3 J1 M83513/25-C01NN, gold Pico-Clasp 203556, TFM -L; RF L106 0603HP-18NXGLU re-tuned (FL1 0.73 dB, noise 552–603 K inside 650 K), L605 0807SQ-10NGLC, J1 TFM-108-01-L-D-RA; carrier J1 0430450402, J2/J3 2035560607/0207, R1 ERJ-6RQF1R0V; harness 501193-9000, 43030-0003, 73415-2261, MINI-1.13 (splice: P3T-10); ERC clean on every board; Samtec plating confirmations in VI-RFB-12 and V-HAR-3 |
| P3T-06 | 3 | BFP640F LNA has a last-time-buy of 2027-05-15 | IMP | Decision point (dated) | BFP640F lifetime buy of 60 pieces from one date code (EM 2, FM 2, spares 6, radiation and DPA 20, process and tuning 30) ordered by 2027-03-31, before the 2027-05-15 last-time-buy, dry-pack storage; owner RF lead with procurement (RF_BOARD_DESIGN F) |
| P3T-07 | 3 | X7R capacitance change at −40 °C (up to −15 %) not in the timing worst cases (EPS integrators, OBC C_RETRY) | IMP | Closed by design change | X7R −15 % at −40 °C in every timing worst case: EPS deploy-switch gate filters 47 → 68 nF (12.8 ms worst against 10 ms), radio/GNSS retry R_FAULT 150 kΩ (≥ 1.26 s against 1 s), load-shed delay 3.7–5.5 s (V-EPS-7 over temperature), integrators −25/+10 % measured by V-EPS-1 extended to −40 °C; OBC C_RETRY with R_FAULT 220 kΩ (0.18–0.33 s against 0.1 s; sensors used ≥ 0.6 s after an enable), V-OBC-5 at −40 °C |
| P3T-08 | 3 | 156 parts are rated exactly −40 °C, with zero margin to the first-orbit cold case | IMP | Closed by analysis | Parts rated −40 °C have +1.4 to +2.4 °C to the first-orbit prediction after the 10 °C margin (OBC/EPS −27.6 °C, RF −28.6 °C, THERMAL_ANALYSIS 2.4 §6) and their full rating elsewhere; the first-orbit −40 °C allowable is GEVS deviation 8 (CR-2026-017, THM-04); V-THM-8 and the −40 °C qualification cycles test them at their rating |
| P3T-09 | 1 | AntS released at S+31 min after an eclipse separation is at −44 °C against its −20 °C qualified minimum; unpowered first-orbit minimum −57 °C with margin against the −50 °C storage limit (THERMAL_ANALYSIS 2.4 §4.4) | IMP | Closed by design change | Gate C-18 (SRS-F165: AntS temperature ≥ −17 °C, 50-min deferral without spending an attempt, 30-deferral cap, ground force via DEPLOY_RETRY byte 1) and hold C-19 (SRS-F167: RF_INHIBIT_ANTS_PENDING, tests in Rust and C); SIL ants_cold: deferred once, deployed 80.7 min, first RF 81.2 min, 0 VSWR stops, gap 126 s; unpowered survival to −57 °C by the ISIS statement (P3T-03 item 1, 2026-12-15); cap and unknown-temperature paths: P3F-G-02; confirmed by P3F-G |
| P3T-10 | 2 | The harness splice M81824/1-1 (TE D-436-36) has a tin-plated barrel against the EEE no-tin rule; the 28 AWG conductors at W6C and W7C are below the smallest splice's range; the sealing sleeve has no outgassing data | IMP | Closed by design change | Splice TE D-200-82 (nickel barrel per AMS-QQ-N-290, −55…+200 °C, M81824/1-1 size; deviation HAR-DEV-1, not a QPL number) on the four 24 AWG deployed-sense leads only; the 28 AWG W6C/W7C leads (Pico-Clasp, 28–32 AWG only) end on bonded polyimide solder tabs TB-RX/RY/RZ and TB-ANT per NASA-STD-8739.3; harness CBE 85.5 g; harness_calcs flags any spliced wire outside 26–20 AWG; sleeve outgassing: TE E595 certificate on a shrunk splice, harness designer, 2027-01-15, fallback a project E595 test of ten splices then a crimp splice under FEP shrink (V-HAR-11; MATERIALS_LIST E §7) |
<!-- RID-LOG-END -->

## 7. Verdict

<!-- Written after the closure verification. -->

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| A | 2026-10-27 | Draft during round 2 |
