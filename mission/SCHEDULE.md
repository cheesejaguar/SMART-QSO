# Integrated Master Schedule
## SMART-QSO 1U CubeSat, 2026-10-27 to a 2028 Launch

**Document ID**: SQSO-SCH-001
**Version**: E
**Date**: 2026-10-27
**Status**: Planning baseline from a critical-path model. No vendor quote, laboratory booking, integrator schedule or
funding commitment exists yet; each is an assumption below. Update at least every 30 days (finding PI-12).
**Model**: `software/simulation/cdr/pm/schedule_cpm.py` (run it to regenerate every date in this document)
**Integrator deliverables**: `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001) §9, LVR-003 and LVR-007;
decision points DP-1 to DP-9 in `docs/ORBITAL_ACCESS.md` (SQSO-OAP-001); the dated actions of §5 below; every
verification item with its event and date in `docs/verification/VI_REGISTER.md` (SQSO-VIR-001)

This replaces the relative "9-month to flight delivery" schedule (Month −9 to Month 0), which assumed a heritage design
and an already-held PDR and CDR; neither exists (BASELINE §1).

---

## 1. Key Dates, Deadlines and Assumptions

| Item | Date | Basis |
|------|------|-------|
| Today | 2026-10-27 | — |
| NASA DAS software usage agreement requested; ISIS AntS RFQ (with its ICD) | **by 2026-11-06** | DA-06; tasks DAS_REQ, ANTS_RFQ |
| VI-RFB-12 (AntS and coax drawings, before the RF layout; RF lead) | **by 2026-11-13** | RID RFB-10; task RFB_PRE |
| FCC Space Bureau consultation requested with the agenda (`docs/regulatory/REGULATORY_ROADMAP.md` Rev D §6) | **by 2026-11-15** | DA-11; task FCC_REQ |
| PDR board named by the PI | **by 2026-12-01** | DA-03 (RID P3P-05) |
| Layout designers contracted (two, in parallel) | **by 2026-12-15** | DA-04 (RIDs P3P-05, R14) |
| Contracting legal entity in place | **by 2026-12-31** | DP-1 (`docs/ORBITAL_ACCESS.md`); DA-05 |
| Integrator requests for quotes out | by 2026-12-31 | Decision D7; DA-07 lists what each RFQ asks for |
| NASA DAS: agreement in hand; DAS run reproducing ODAR Draft D; ODAR Draft E | 2026-12-15; 2027-01-08; 2027-01-12 | DA-06 (RIDs SE-14, ODA-02): all before the early FCC filing |
| FCC consultation meeting; outcome recorded | by 2027-01-05; 2027-01-08 | DA-11; tasks FCC_CONS, FCC_OUT |
| Independent PDR (NPR 7123.1D Table G-6) | 2027-01-12 | `docs/project/CDR_CRITERIA_MATRIX.md` §2 |
| EM funding available | **2027-01-15 (assumption)** | Needed for the EM long-lead buys after the PDR |
| Procurement data reviews before the EM order (V-HAR-3, V-HAR-11, EPSB-09, JETB-07) | by 2027-01-15 | DA-18; task PRE_ORDER |
| Early FCC pre-space notification with the API data and ODAR Draft E | **2027-01-19** | `docs/regulatory/REGULATORY_ROADMAP.md`; task FCC_EARLY |
| IARU coordination request | **2027-01-26** | After the PDR and a week after the early FCC filing (`docs/regulatory/IARU_COORDINATION.md`) |
| NVIDIA design guides and Molex drawings (JETB-06, JETB-12), before the carrier layout | by 2027-02-01 (carrier layout starts 2027-03-16) | DA-19; task JET_DG |
| IARU coordination letter | **by 2027-05-18** (90th percentile; median 2027-03-25) | IARU status pages, 39 coordinations (`docs/regulatory/IARU_COORDINATION.md`); DA-09 |
| FM and launch funding committed; launch deposit | **2027-05-31 (assumption)** | Risk R11: funding decision by Q2 2027 |
| Launch contract signed (launch-vehicle determination); integrator ICD, shock MPE and dispenser thermal analysis | 2027-05-31, deadline **2027-06-30** | Decision D7; DA-08, DA-12 |
| Independent CDR (Table G-7) with EM test data | **2027-09-28** (was 2027-08-03 before the layout re-estimate and the reliability programme) | §3 |
| Planning launch | **2028-11-01**, the last Transporter-class rideshare of 2028 (2028-06-01 is an opportunity only) | BASELINE Rev E §3 (CR-2026-007) |
| Mission-analysis inputs: CAD, dynamic model, mass properties ± 15 %, webforms | L − 8 months = **2028-03-01** | LVR-003, LVR-005 (UG v11 Table G-3) |
| Hazardous-systems declaration, 91-710 letter | L − 6 months = 2028-05-01 | LVR-008 |
| Protoflight environmental campaign complete | **L − 4 months = 2028-07-01** (2028-02-01 for June) | LVR §9 (UG v11 Fig. 1-2) |
| Environmental verification report, final mass properties ± 5 % | **L − 3.5 months = 2028-07-17** | LVR-007 |
| Delivery to the integrator | L − 50 days = 2028-09-12 | LVR §9 (date set by the mission ICD) |
| Final charge and integration into the dispenser; no access afterwards | L − 47 days = 2028-09-15 | LVR §9 |

The binding deadline is the **end of the protoflight campaign at L − 4 months**, not the delivery date.

## 2. Schedule

Early dates (the same for any launch date) and total float to the binding deadlines, for the June 2028 opportunity and
the November 2028 planning launch. Negative float (bold) means the task cannot finish in time for that launch. Two
layout designers, layout after the PDR, the reliability programme in (the model's defaults).

| ID | Task | Duration | Early start | Early finish | Float, L = 2028-06-01 | Float, L = 2028-11-01 |
|----|------|----------|-------------|--------------|-----------------------|-----------------------|
| ANTS_RFQ | ISIS AntS RFQ, with the ICD (VI-RFB-13, AV-V7, DA-17) | 10 d | 2026-10-27 | 2026-11-06 | 0 d | 0 d |
| DAS_REQ | NASA DAS software usage agreement requested (DA-06) | 10 d | 2026-10-27 | 2026-11-06 | 0 d | 0 d |
| ENTITY | Contracting legal entity in place (ORBITAL_ACCESS DP-1) | 8 wk | 2026-10-27 | 2026-12-22 | 9 d | 9 d |
| EPS_DES | EPS controller firmware design (register map, keys, SOC, MPPT; language ratified, DA-22), into the PDR package (flight software lead, PI until named) | 5 wk | 2026-10-27 | 2026-12-01 | **-52 d** | 99 d |
| FCC_REQ | FCC Space Bureau consultation requested with the agenda (REGULATORY_ROADMAP D §6; DA-11) | 19 d | 2026-10-27 | 2026-11-15 | 0 d | 0 d |
| FW_REQ | Board firmware requirements: SQSO-SRS-TNC-001 (67) and SQSO-SRS-EPSC-001 (62) (complete at this CDR; flight software lead, PI until named) | done | 2026-10-27 | 2026-10-27 | **-59 d** | 92 d |
| JET_DG | JETB-06/V-JET-2, JETB-12: NVIDIA DG-10931-001 and TDG-11127-001, Molex drawings; conformance review before the carrier layout | 6 wk | 2026-10-27 | 2026-12-08 | 18 d | 55 d |
| MECH | Mechanical CAD, FE and dynamic model (complete: SQSO-MECH-CAD-001, STRUCTURAL_ANALYSIS) | done | 2026-10-27 | 2026-10-27 | **-10 d** | 141 d |
| PDR_BOARD | PI names the independent PDR board and its chair (DA-03, by 2026-12-01) | 5 wk | 2026-10-27 | 2026-12-01 | **-52 d** | 0 d |
| PRE_ORDER | Procurement data reviews before the EM order: V-HAR-3, V-HAR-11, EPSB-09, JETB-07 | 6 wk | 2026-10-27 | 2026-12-08 | 11 d | 38 d |
| RFB_PRE | VI-RFB-12 (RF lead): AntS and coax drawings, plating, attenuation, inner conductor (before RF layout) | 17 d | 2026-10-27 | 2026-11-13 | 0 d | 0 d |
| RFQ | Integrator requests for quotes (D7: by 2026-12-31) | 9 wk | 2026-10-27 | 2026-12-29 | 2 d | 2 d |
| RID | Close this review's RIDs (rounds 1-3); requirements rebaseline (CR-2026-007 to -015) | 6 wk | 2026-10-27 | 2026-12-08 | **-59 d** | 92 d |
| SCH | EM schematic capture (complete: schematic as code, ERC, BOM; SQSO-EPS/OBC/RF/JET-BRD-001) | done | 2026-10-27 | 2026-10-27 | **-17 d** | 134 d |
| TCS | Name >= 2 telecommand stations (D1) | 10 wk | 2026-10-27 | 2027-01-05 | 7 d | 7 d |
| TNC_DES | TNC firmware design (modem, KISS, scrub, supervision; LMX2571 write in power-down checked), into the PDR package (flight software lead, PI until named) | 6 wk | 2026-10-27 | 2026-12-08 | **-59 d** | 92 d |
| DAS_SW | NASA DAS agreement in hand (DA-06) | 39 d | 2026-11-06 | 2026-12-15 | 0 d | 0 d |
| FCC_CONS | FCC Space Bureau consultation (D3): meeting | 51 d | 2026-11-15 | 2027-01-05 | 0 d | 0 d |
| PDR_PKG | PDR data package to the board, two weeks before the review | 1 wk | 2026-12-08 | 2026-12-15 | **-59 d** | 92 d |
| DAS | NASA DAS run reproducing ODAR Draft D (DA-06) | 24 d | 2026-12-15 | 2027-01-08 | 0 d | 0 d |
| QUOTES | Quotes, evaluation, selection | 8 wk | 2026-12-29 | 2027-02-23 | 43 d | 43 d |
| FCC_OUT | FCC consultation outcome recorded (DA-11) | 3 d | 2027-01-05 | 2027-01-08 | 0 d | 0 d |
| PDR | Independent PDR (NPR 7123.1D Table G-6), held as a peer review | 1 wk | 2027-01-05 | 2027-01-12 | **-80 d** | 71 d |
| ODAR_E | ODAR Draft E with the DAS output, for the early filing (DA-06) | 4 d | 2027-01-08 | 2027-01-12 | 0 d | 0 d |
| EPS_CODE | EPS controller firmware code after the PDR (flight software lead, PI until named) | 10 wk | 2027-01-12 | 2027-03-23 | **-17 d** | 134 d |
| FCC_EARLY | Early FCC pre-space notification with the API data and ODAR Draft E | 1 wk | 2027-01-12 | 2027-01-19 | 0 d | 0 d |
| L_EPS | EM layout: EPS board, SQSO-EPS-BRD-001 (PC/104, 6 layers) (9 wk, basis LAYOUT_BOARDS) | 9 wk | 2027-01-12 | 2027-03-16 | **-80 d** | 71 d |
| L_OBC | EM layout: OBC board, SQSO-OBC-BRD-001 (PC/104, 6 layers) (5 wk, basis LAYOUT_BOARDS) | 5 wk | 2027-01-12 | 2027-02-16 | **-73 d** | 78 d |
| TNC_CODE | TNC firmware code after the PDR (flight software lead, PI until named) | 12 wk | 2027-01-12 | 2027-04-06 | **-45 d** | 106 d |
| EM_LL | EM COTS buys: frame, AntS, magnetometer, gyro, GNSS, Jetson (12 wk, BVB §2) | 12 wk | 2027-01-15 | 2027-04-09 | **-27 d** | 124 d |
| FUND_EM | EM funding available (assumption) | — | 2027-01-15 | 2027-01-15 | **-27 d** | 124 d |
| PANELS | Solar panels, EM and FM sets ordered together (3-7 months, BVB §2; 7 used) | 30 wk | 2027-01-15 | 2027-08-13 | 50 d | 201 d |
| IARU_REQ | IARU coordination request submitted (form v40), after the PDR and the early FCC filing | 1 wk | 2027-01-19 | 2027-01-26 | 302 d | 455 d |
| IARU | IARU panel iteration to coordination letter (90th percentile 112 d; median 57 d) | 16 wk | 2027-01-26 | 2027-05-18 | 302 d | 455 d |
| L_RF | EM layout: RF board, SQSO-RF-BRD-001 (6 layers, 300 fitted) (6 wk, basis LAYOUT_BOARDS) | 6 wk | 2027-02-16 | 2027-03-30 | **-73 d** | 78 d |
| NEGOT | Contract negotiation, altitude clause, export terms | 12 wk | 2027-02-23 | 2027-05-18 | 43 d | 43 d |
| L_CAR | EM layout: Jetson carrier, SQSO-JET-BRD-001 (6 layers, 2.0 mm) (5 wk, basis LAYOUT_BOARDS) | 5 wk | 2027-03-16 | 2027-04-20 | **-80 d** | 71 d |
| EPS_UT | EPS controller firmware unit tests in CI and development-board tests (pack and array simulators) (flight software lead, PI until named) | 5 wk | 2027-03-23 | 2027-04-27 | **-17 d** | 134 d |
| L_SMALL | EM layout: Magnetometer board A2 and access-port board A3 (2 layers) (2 wk, basis LAYOUT_BOARDS) | 2 wk | 2027-03-30 | 2027-04-13 | **-73 d** | 78 d |
| TNC_UT | TNC firmware unit tests in CI and development-board tests (CC1200 EVM; demodulator threshold, CPU load) (flight software lead, PI until named) | 6 wk | 2027-04-06 | 2027-05-18 | **-45 d** | 106 d |
| LAYOUT | EM layout cross-board check (stack, harness, VI-MCAD-01 fit) and fabrication release | 1 wk | 2027-04-20 | 2027-04-27 | **-80 d** | 71 d |
| EM_FAB | EM PCB fabrication and assembly | 5 wk | 2027-04-27 | 2027-06-01 | **-80 d** | 71 d |
| CONTRACT | Contract signature and deposit (D7: by 2027-06-30); launch-vehicle determination | — | 2027-05-31 | 2027-05-31 | 30 d | 30 d |
| EXPORT | Export counsel; TAA if the integrator is foreign (3-6 months, R15) | 20 wk | 2027-05-31 | 2027-10-18 | 177 d | 330 d |
| FCC_PRE | FCC 97.207(g)(1) replacement notification (<= 30 d after LV determination) | 4 wk | 2027-05-31 | 2027-06-28 | 289 d | 442 d |
| FUND_FM | FM and launch funding committed (R11: decision by Q2 2027) | — | 2027-05-31 | 2027-05-31 | 30 d | 30 d |
| INT_DATA | Integrator data at the contract: mission ICD, attenuated shock MPE, dispenser thermal analysis, coupled-loads approach (SE-05, VV-07, THM-06/07) | — | 2027-05-31 | 2027-05-31 | 30 d | 30 d |
| MA_IN | Mission-analysis inputs to the integrator (L-8 M: CAD, dynamic model, webforms) | 2 wk | 2027-05-31 | 2027-06-14 | 109 d | 261 d |
| BATT_ABUSE | Custom-battery abuse tests on sacrificial EM-design packs (BSA D §3.1, V-BSA-9; Falcon 9 set, 3-4 weeks; ISS backup set 8-12 weeks) | 4 wk | 2027-06-01 | 2027-06-29 | 4 d | 155 d |
| EM_INT | EM integration, flatsat, flight software on target (TP-FLATSAT-001) | 8 wk | 2027-06-01 | 2027-07-27 | **-80 d** | 71 d |
| EPS_EM | EPS controller firmware on the EPS board EM: V-EPS-1/-6 to -10/-12 to -17, IFT-EPS-001 to -006 (flight software lead with the EPS designer, PI until named) | 4 wk | 2027-06-01 | 2027-06-29 | **-52 d** | 99 d |
| TNC_EM | TNC firmware on the RF board EM: VI-RFB-01/-05/-07/-08/-10, flatsat with the OBC (flight software lead with the RF lead, PI until named) | 5 wk | 2027-06-01 | 2027-07-06 | **-59 d** | 92 d |
| EM_TEST | EM development tests: Jetson heat path and burst energy, EM TVAC, pathfinder vibe, shock if VI-07 requires it, RF/EMC pre-scan, GNSS noise, antenna pattern | 8 wk | 2027-07-27 | 2027-09-21 | **-80 d** | 71 d |
| FLATSAT_500H | Reliability programme: >= 500 h powered on the EM boards with flight software (REL §6; ETP §6, VI-11) | 4 wk | 2027-07-27 | 2027-08-24 | **-52 d** | 99 d |
| CDR | Independent CDR (Table G-7) with EM data | 1 wk | 2027-09-21 | 2027-09-28 | **-80 d** | 71 d |
| BATT | Flight-lot cell screening, pack build, LEO cycling at 4.10 V (D8) | 10 wk | 2027-09-28 | 2027-12-07 | **-66 d** | 85 d |
| CDR_RID | CDR RID closure | 3 wk | 2027-09-28 | 2027-10-19 | 176 d | 329 d |
| FM_LAYOUT | FM layout update from EM findings | 4 wk | 2027-09-28 | 2027-10-26 | **-66 d** | 85 d |
| FM_LL | FM COTS buys | 12 wk | 2027-09-28 | 2027-12-21 | **-80 d** | 71 d |
| SDHS | Hazardous-systems declaration and 91-710 letter (L-6 M) | 4 wk | 2027-09-28 | 2027-10-26 | 36 d | 188 d |
| FM_FAB | FM PCB fabrication, assembly, inspection | 6 wk | 2027-10-26 | 2027-12-07 | **-66 d** | 85 d |
| EPS_QUAL | EPS controller firmware qualification of the flight build on the FM EPS board (flight software lead, PI until named) | 3 wk | 2027-12-07 | 2027-12-28 | **-45 d** | 106 d |
| TNC_QUAL | TNC firmware qualification of the flight build on the FM RF board (flight software lead, PI until named) | 3 wk | 2027-12-07 | 2027-12-28 | **-45 d** | 106 d |
| FM_INT | FM integration (TP-INT-001 to -006) | 6 wk | 2027-12-21 | 2028-02-01 | **-80 d** | 71 d |
| FM_FUNC | FM comprehensive functional and day-in-the-life (TP-SYS-001, -002) | 3 wk | 2028-02-01 | 2028-02-22 | **-80 d** | 71 d |
| FM_BURNIN | Reliability programme: FM burn-in >= 200 h powered, last 100 h failure-free, 4 ambient thermal cycles (REL §6; ETP §3.11, VI-12) | 10 d | 2028-02-22 | 2028-03-03 | **-80 d** | 71 d |
| PF_ENV | Protoflight campaign: mass properties, vibe, TVAC + bake-out, deployment hot/cold, EMC/self-compatibility, magnetic dipole, post-env CPT, transport (complete by L-4 M) | 7 wk | 2028-03-03 | 2028-04-21 | **-80 d** | 71 d |
| ENV_REPORT | Environmental verification report on the SpaceX template (L-3.5 M) | 1 wk | 2028-04-21 | 2028-04-28 | **-73 d** | 80 d |
| PSR | Pre-ship review and integrator data package | 3 wk | 2028-04-28 | 2028-05-19 | **-37 d** | 116 d |
| DELIVER | Delivery to the integrator | — | 2028-05-19 | 2028-05-19 | **-37 d** | 116 d |

**Basis of the EM layout estimate (RID P3P-05).** The round-2 build-to designs give the board sizes: EPS 700
placements (PC/104, 6 layers), OBC 351 (PC/104, 6 layers), RF 300 fitted (6 layers, impedance-controlled), Jetson
carrier 137 (6 layers, 2.0 mm, PCIe and USB striplines, 260-pin SO-DIMM), magnetometer board A2 15 and access-port
board A3 8 (2 layers). Designer-hours = placements × 0.25 h × complexity (1.0 power and digital, 1.5 RF, 2.0 carrier)
+ 40 h per board (stack-up and constraints, netlist import from the schematic-as-code, fabrication and assembly
outputs, design-rule and peer review; 24 h for A2 and A3 together) + 20 % for review changes, at 32 productive hours a
week: EPS 9 weeks, OBC 5, RF 6, carrier 5, A2/A3 2; 27 designer-weeks in all, against the 8 calendar weeks of Version C.
With two designers (A: EPS then carrier; B: OBC, RF, then A2/A3) the layout takes 14 weeks plus a one-week cross-board
check and fabrication release. The rates are a planning assumption for a contract KiCad designer, not a quote; no
layout designer is engaged (risk R14, DA-04).

**Reliability programme (RIDs REL-01, P3P-03, P3E-07).** RELIABILITY_ANALYSIS §6 credits two activities with halving
the early-failure probabilities (L1/L2/L3 0.86 / 0.71 / 0.59 instead of 0.75 / 0.61 / 0.51): ≥ 500 h powered on the
EM boards with the flight software on the target (task FLATSAT_500H, in parallel with the EM tests, before the CDR)
and an FM burn-in of ≥ 200 h powered, the last 100 h failure-free, with four ambient thermal cycles (task FM_BURNIN,
10 days on the critical path). Their pass criteria are verification items VI-11 and VI-12 of
`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §8. Dropping both returns 10 days of float and the allocation-case
estimates (DA-02).

**Board firmware (finding P3A-03, Version E).** The RF board's TNC firmware and the EPS controller firmware are Class C
software items (SMART-QSO-SDP-001 §3.4) with no task until Version D. Their requirements are complete at this CDR
(task FW_REQ: SQSO-SRS-TNC-001, 67 requirements; SQSO-SRS-EPSC-001, 62). Each item then has a design that goes into
the PDR package (TNC_DES, EPS_DES; PDR_PKG now waits for both), code after the PDR (TNC_CODE, EPS_CODE), host unit
tests and development-board tests (TNC_UT, EPS_UT), integration on its EM board with the flatsat before the EM tests
and the 500 h (TNC_EM, EPS_EM; EM_TEST and FLATSAT_500H wait for both) and qualification of the flight build on its FM
board before the FM functional test (TNC_QUAL, EPS_QUAL; FM_FUNC waits for both). Owner of every task: the flight
software lead (PI until named), with the RF lead and the EPS designer on the EM tasks. Durations are planning
estimates for one developer per item. None is on the critical path: the least float among them is 92 days (TNC_DES,
FW_REQ, TNC_EM) for the November launch, and the critical path and its +71 days are unchanged; for June 2028 their
float is −59 to −17 days, inside the −80 days of the path.

## 3. Critical Path and Margin

**Critical path**: PDR → EM layout (EPS, then the Jetson carrier) → cross-board check → EM fabrication → EM integration
→ EM development tests → CDR → FM COTS buys → FM integration → FM functional and DITL → FM burn-in → protoflight
campaign (must end by L − 4 months). EM schematic capture and the mechanical CAD are complete (the round-2 build-to
designs), so neither is on the path any more.

**For the 2028-11-01 planning launch the float is +71 days (10.1 weeks)**: the protoflight campaign ends on 2028-04-21
against the 2028-07-01 milestone. GSFC-STD-1000 (GOLD Rules) Rule 1.06, as commonly applied (the text was not retrieved
in this review), wants 1 month per year to the start of I&T and 2 months per year from the start of I&T to the
deadline: 9.6 weeks here. **The plan meets the rule by 0.5 weeks.** **For the June 2028 opportunity the float is −80
days**: the campaign ends 11.4 weeks after the 2028-02-01 milestone, before any margin.

| Scenario (`schedule_cpm.py` options) | Protoflight campaign ends | Float to the binding deadline | Against the GOLD margin |
|--------------------------------------|---------------------------|-------------------------------|-------------------------|
| **Baseline: launch 2028-11-01** (no options; two layout designers, layout after the PDR, reliability programme in) | 2028-04-21 | **+71 d** | **meets** (10.1 against 9.6 weeks) |
| One layout designer (`--layout-designers 1`) | 2028-07-21 | **−20 d** | −11.4 weeks |
| Layout started at risk on the PDR package (`--layout-at-risk`) | 2028-03-24 | +99 d | meets (14.1 against 9.9 weeks) |
| FM long leads ordered at risk at the PDR (`--fm-at-risk`) | 2028-04-07 | +85 d | meets (12.1 against 9.8 weeks) |
| EM funds 2027-03-15 (`--em-funds 2027-03-15`) | 2028-04-27 | +65 d | −0.2 weeks |
| FM and launch funds 2027-09-30 (`--fm-funds 2027-09-30`) | 2028-04-23 | +69 d | meets, but the contract misses the D7 deadline |
| Reliability programme dropped (`--no-reliability`; DA-02) | 2028-04-11 | +81 d | meets (11.6 weeks) |
| Launch 2028-09-01 (`--launch 2028-09-01`) | 2028-04-21 | +10 d | −6.7 weeks |
| Launch 2028-08-01 | 2028-04-21 | −20 d | −10.3 weeks |
| Launch 2028-07-01 | 2028-04-21 | −51 d | −14.0 weeks |
| **June 2028 opportunity** (`--launch 2028-06-01`) | 2028-04-21 | **−80 d** | −17.4 weeks |
| June 2028, FM long leads and layout both at risk (`--launch 2028-06-01 --fm-at-risk --layout-at-risk`) | 2028-03-10 | −38 d | −11.9 weeks |

**Findings.**

1. **The June 2028 opportunity is not achievable on this plan**, even with every at-risk start: the protoflight
   campaign ends at best 38 days after the integrator's L − 4 month milestone. The RFQs ask for the last 2028
   Transporter flight (BASELINE §3).
2. **The November 2028 plan meets the GOLD margin by half a week.** The margin depends on two layout designers
   (DA-04): with one, the plan is 20 days late before any margin. A two-month slip in EM funding now costs 0.2 weeks of
   the rule. The PI's levers, in order of cost: start the EM layout at risk on the frozen PDR package (+28 days), order
   the FM long leads at the PDR (+14 days), drop the reliability programme (+10 days, at the reliability cost in DA-02).
3. The administrative deadlines have almost no float: the legal entity (DP-1) by 2026-12-31 has 9 days, the RFQs 2
   days, the PDR board 0 days to 2026-12-01, VI-RFB-12 0 days to 2026-11-13, the DAS and FCC-consultation chain 0 days
   to the early filing (DAS agreement requested by 2026-11-06, in hand 2026-12-15, run by 2027-01-08, ODAR Draft E by
   2027-01-12; consultation requested by 2026-11-15, met by 2027-01-05, outcome recorded 2027-01-08), and the contract
   30 days to the D7 deadline. They do not move with the launch date.
4. **Regulatory timing**: the early FCC pre-space notification goes in on **2027-01-19** with the API data and the
   ODAR Draft E (with the DAS result, DA-06); the IARU request on 2027-01-26; the IARU letter by 2027-05-18 (90th
   percentile). The replacement notification follows within 30 days of the launch-vehicle determination (2027-06-28 if
   the contract is signed on 2027-05-31), with the IARU frequencies and the final ODAR. The IARU request goes in 19.6
   months before integration; the roadmap target is 18 months and the floor 12.
5. **Mission-analysis inputs at L − 8 months** (2028-03-01) need the CAD and the structural dynamic model, which exist
   (SQSO-MECH-CAD-001; `docs/environment/STRUCTURAL_ANALYSIS.md`); 261 days of float.
6. **The ISS (NRCSD) backup** needs its contract about 12 months before launch, so the D7 backup decision on
   2027-09-30 gives an NRCSD launch no earlier than about 2028-09 (`docs/ORBITAL_ACCESS.md`). Its battery abuse-test
   set takes 8–12 weeks instead of the Falcon 9 set's 3–4 (task BATT_ABUSE, 155 days of float either way).
7. Export: a foreign integrator (Exolaunch, ISISPACE) may need a TAA for the integration services (3–6 months, risk
   R15); it fits with 330 days of float if export counsel is engaged by 2026-12-15 (DA-13).

## 4. Milestone Summary

| Milestone | Date |
|-----------|------|
| VI-RFB-12 complete (RF layout input) | 2026-11-13 |
| PDR board named (DA-03) | 2026-12-01 |
| DAS agreement requested; ISIS AntS RFQ | 2026-11-06 |
| FCC consultation requested | 2026-11-15 |
| NASA DAS run; ODAR Draft E (DA-06) | 2027-01-08; 2027-01-12 |
| Contracting legal entity (DP-1); integrator RFQs out | 2026-12-22; 2026-12-29 |
| Board firmware requirements complete (SQSO-SRS-TNC-001, SQSO-SRS-EPSC-001) | 2026-10-27 |
| Board firmware designs in the PDR package | 2026-12-08 |
| **PDR** | **2027-01-12** |
| Early FCC pre-space notification | **2027-01-19** |
| IARU request | 2027-01-26 |
| EM layout released | 2027-04-27 |
| IARU coordination letter (90th percentile) | by 2027-05-18 |
| **Launch contract** (deadline 2027-06-30); integrator ICD | **2027-05-31** |
| EM boards assembled | 2027-06-01 |
| Board firmware integrated on the EM boards (TNC, EPS controller) | 2027-07-06 |
| EM integrated, flight software running on the target | 2027-07-27 |
| 500 h on the EM boards (reliability programme) | 2027-08-24 |
| **CDR** | **2027-09-28** |
| FM integration start | 2027-12-21 |
| Board firmware flight builds qualified on the FM boards | 2027-12-28 |
| FM burn-in (≥ 200 h) | 2028-02-22 to 2028-03-03 |
| Protoflight campaign | 2028-03-03 to 2028-04-21 |
| Environmental verification report | 2028-04-28 (due 2028-07-17) |
| Pre-ship review; delivery | 2028-05-19 (earliest; required 2028-09-12) |
| **Launch** | **2028-11-01 planning date; June 2028 not achievable on this plan** |

## 5. Decision Points and Dated Actions

Each row names an owner, a due date and the criterion or outcome that closes it. Rows owned by the PI are decisions only
the PI can take (CM plan §2.2; the review chair cannot approve Class I changes). Roles other than the PI are open
positions (PROJECT_PLAN §4); the PI holds each until it is named. `scripts/gen_vi_register.py` copies this table into
the verification-item register and fails CI if a row lacks an owner, a date or a criterion.

| ID | Decision or action | Owner | Due | Pass criterion or outcome | Parent RID | Status |
|----|--------------------|-------|-----|---------------------------|------------|--------|
| DA-01 | Ratify or reject CR-2026-007 and CR-2026-008 (approved by the review chair only), decide CR-2026-009 to CR-2026-018 (`docs/project/CHANGE_REQUEST_LOG.md`), and ratify or reject the review chair's decisions C-1 to C-24 (SQSO-REV-CDR-002 §4) | PI | 2027-01-12 | Each CR's CCB decision reads Approved, Approved with conditions or Rejected, dated and signed by the PI; a rejected change is reverted in the documents it touched | PM-04, P3P-02, SE-03 | Open |
| DA-02 | Accept or reject the Class D reliability estimates (L1/L2/L3 0.75 / 0.61 / 0.51; 0.86 / 0.71 / 0.59 with the reliability programme) and keep or drop the programme (≥ 500 h EM flatsat, ≥ 200 h FM burn-in) | PI | 2026-12-15 | Decision recorded in RELIABILITY_ANALYSIS §5 and MISSION_SUCCESS_CRITERIA §8; if the programme is dropped, FLATSAT_500H and FM_BURNIN leave the schedule and the estimates revert to the allocation case | REL-01, P3P-03, P3E-07 | Open |
| DA-03 | Name the independent PDR board: a chair and at least two reviewers outside the team covering systems, electrical and RF, and flight software; invitations with the 2027-01-12 date | PI | 2026-12-01 | Names, affiliations and conflict-of-interest statements in PROJECT_PLAN §6.2; data package to the board by 2026-12-29 | P3P-05, PM-01 | Open |
| DA-04 | Engage two PCB layout designers (KiCad, IPC-6012 Class 3) for a 2027-01-12 start; decide whether the layout starts at risk on the PDR package | PI | 2026-12-15 | Agreements in place for both designers; with one designer the plan has −20 d float (§3) | P3P-05 | Open |
| DA-05 | Contracting legal entity named (DP-1) | PI | 2026-12-31 | An entity able to sign the launch services agreement, cross-waivers, insurance declarations and export agreements | LV-09 | Open |
| DA-06 | NASA DAS: software usage agreement requested by 2026-11-06 and in hand by 2026-12-15; DAS run on the ODAR Draft D cases by 2027-01-08; ODAR Draft E with the DAS output by 2027-01-12, for the early FCC filing (2027-01-19) | Systems engineer | 2027-01-12 | DAS reproduces ODAR Draft D: orbital lifetime ≤ 5 years after the end of the 2-year mission at 485 km and 1.85 kg with P ≥ 0.9 under the conservative solar projection, and re-entry casualty risk ≤ 1:10,000 (NASA-STD-8719.14C). If DAS disagrees, the cap moves by CR to the highest altitude DAS passes, before the contract negotiation starts (2027-02-23) | SE-14, ODA-02 | Open |
| DA-07 | Integrator RFQs to Exolaunch, ISISPACE, SEOPS, D-Orbit and Maverick, each asking for: the mission ICD at the contract; the attenuated EXOpod Nova shock MPE at the CubeSat interface; the dispenser thermal analysis for the assigned flight and slot with the deployment time; the SSCMAN 91-710 and battery failure-tolerance requirements; the coupled-loads approach | PI | 2026-12-31 | Five RFQs sent with these items; replies logged in `docs/ORBITAL_ACCESS.md` | SE-05, VV-07, THM-07, SMA-09, EPS-23 | Open |
| DA-08 | Integrator data with the signed contract: mission ICD, levels, attenuated shock MPE, 91-710 and failure-tolerance requirements | PI (contract signatory) | 2027-06-30 | ICD received; LVR and ETP levels replaced by CR; VI-07 (shock) and EPS-23 (failure tolerance) assessed by 2027-07-14 | SE-05, VV-07, SMA-09, EPS-23 | Open |
| DA-09 | IARU coordination: request on 2027-01-26 after the early FCC filing; letter | Licensee (KK6GDB) | 2027-05-18 | Coordinated downlink and uplink frequencies frozen in BASELINE §7 by CR; with no letter by 2027-05-18 the licensee asks the IARU Satellite Advisor for status and the contract proceeds on the requested windows | SE-05, P3P-10 | Open |
| DA-10 | Telecommand stations: written agreements with at least two stations at least 1,000 km apart, each able to command alone | Licensee (KK6GDB) | 2027-01-05 | Agreements signed; stations named in the early FCC filing and the IARU request; VI-GND-02 site survey of each | GND-01 | Open |
| DA-11 | FCC Space Bureau consultation (D3): requested by 2026-11-15 with the agenda (`docs/regulatory/REGULATORY_ROADMAP.md` Rev D §6): the ITU API path and timing (REG-05), whether an early failure ends the mission (ODA-05), the contact timeouts N and M, and the beacon kept with a failed receiver (REG-11); meeting by 2027-01-05 | Licensee (KK6GDB) | 2027-01-08 | Outcome recorded by 2027-01-08, before the early filing (2027-01-19). ODA-05 fallback: if an early failure counts as the end of the mission, the 460–465 km band of BASELINE §3 is adopted by CR before the contract negotiation | REG-05, ODA-05, REG-11 | Open |
| DA-12 | Dispenser soak (THM-06, THM-07): the integrator's dispenser thermal analysis for the assigned flight and slot, with the deployment time, requested in the RFQ (DA-07) and delivered at the contract | PI (licensee) with the integrator | 2027-06-30 | Pack ≤ +50 °C and ≥ −15 °C at deployment, boards within their non-operating limits (V-THM-5). Fallback if the hot result exceeds +50 °C, in order: a deployment-time limit in the contract from the integrator's transient; a low-absorptance finish on the pack-facing surfaces (THERMAL_ANALYSIS §6.5); integrator pad conditioning (purge or air conditioning until fairing close); if none brings the pack to ≤ +50 °C, the PI decides between a lower storage SOC and the NRCSD backup | THM-06, THM-07 | Open |
| DA-13 | Export counsel engaged; export memo Rev B (authentication versus encryption; foreign integrators) | PI | 2026-12-15 | Counsel's written opinion; TAA need decided before the contract negotiation (2027-02-23) | REG-06 | Open |
| DA-14 | Panel vendor RFQ with the body-panel cells offset 1 mm from the hinge edge (wing sweep, MECHANICAL_DESIGN §6.6) | Mechanical lead | 2026-12-31 | RFQ states the offset; the vendor confirms before the panel order (2027-01-15) that the 45° sweep clearance is ≥ 0.35 mm with its tolerances | MCAD-03 | Open |
| DA-15 | Venting analysis of every enclosed volume from the CAD (VI-LV-12): the gross 1U volume, the battery clamp cavity, the pocket behind the heat spreader; the RF shield cans are done (V/A 3.2–9.4 m, SQSO-RF-BRD-001 D) | Mechanical lead | 2026-12-15 | Each volume / vent area ≤ 50.8 m (CDS §2.1.9); required holes on the drawings in the PDR package | LV-19 | Open |
| DA-16 | SWE-220: measure the cyclomatic complexity of the safety-critical Rust files with a tool and gate it in CI, or tailor by CR | Flight software lead | 2026-12-15 | Every function in the safety-critical components (SDP §3.2) ≤ 15, or a CR listing each exception with its justification; result in `docs/software/NPR7150_COMPLIANCE_MATRIX.md` item C-4 | FSW-06 | Open |
| DA-17 | Flexible 2 m AntS tapes in the ADCS Monte Carlo (ADCS_DESIGN §11.5): element mass, stiffness and damping from the ISIS ICD (with AV-V7); flexible-appendage case; EM measurement of the deployed tape's first mode and damping in the EM tests | ADCS and flight software lead | 2027-02-26 (analysis); EM measurement before the CDR (2027-09-28) | With the tape's first mode and a damping ratio no higher than the measured or 0.5 %, the spin about +Y converges and +Y stays within 15° of the Sun for ≥ 90 % of sunlit time; the measured first mode within ± 20 % of the model's, or the Monte Carlo re-run with it | GNC-03 | Open |
| DA-18 | Procurement data reviews before the EM order: V-HAR-3 (vendor drawings, gold part numbers), V-HAR-11 (ASTM E595 per part number), EPSB-09 (FM12 interrupting rating against the 209 A prospective short, 60 °C rerating, termination code; SMF24A leakage), JETB-07 (every carrier BOM line on the manufacturer's data sheet) | EPS designer, harness designer, payload lead | 2027-01-15 | Each data sheet or drawing cited in the BOM "MPN check" column; FM12 interrupting rating ≥ 209 A at ≥ 16.8 VDC and ≥ 5 A at +60 °C; storage drain with the SMF24A's +60 °C leakage ≤ 20 µA; every item without E595 data coded A–E or tested; a failing part changed before the order | EPSB-09, JETB-07, SE-23 | Open |
| DA-19 | Drawings before the layout: VI-RFB-12 (AntS RF port, coax plating, attenuation, inner conductor; by 2026-11-13, RF lead; ISIS AntS RFQ with the ICD by 2026-11-06); JETB-06 (NVIDIA DG-10931-001 and TDG-11127-001 through a developer account); JETB-12 (Molex drawings) | RF designer; payload lead | 2027-02-01 | VI-RFB-12 pass before the RF layout (2027-02-16); V-JET-2 passes before the carrier layout (2027-03-16); J1 mated height ≤ 13.2 mm and J2 ≤ 7.7 mm from the drawings, else the right-angle J1 fallback | RFB-10, JETB-06, JETB-12 | Open |
| DA-20 | Ratify the proposed wordings TBR-MRD-002 to -005 and TBR-SYS-002 to -004 | PI | 2027-01-12 | Each TBR closed in its document (ratified, amended or rejected) | SE-04 | Open |
| DA-21 | Shock (VV-07): decide VI-07 from the integrator's attenuated shock MPE | Test lead | 2027-07-14 | Analysis passes only if the vibration-equivalent SRS (24 / 53 / 85 / 60 g at 100 / 500 / 1000 / 2000 Hz) envelops the MPE below 2000 Hz; otherwise an EM shock test at MPE + 3 dB, 2 actuations per axis, in the EM tests (costed in `software/simulation/cdr/pm/test_cost.py`) | VV-07 | Open |
| DA-22 | Ratify the board firmware languages: RF board TNC in Rust `no_std` with the OBC's pinned toolchain (proposed, SQSO-SRS-TNC-001 §7); EPS controller in C11 to MISRA C:2012 with TI MSP430-GCC (recommended, SQSO-SRS-EPSC-001 §7) | PI | 2026-12-01 | Decision recorded in SMART-QSO-SDP-001 §3.4 and each specification's §7 before the firmware designs go into the PDR package; if Rust is chosen for the EPS controller, a CR with the nightly toolchain and its qualification plan (CR-2026-004 covers only the pinned stable toolchain) | P3A-03 | Open |

---

## Revision History

| Version | Date | Description | Author |
|---------|------|-------------|--------|
| — | before 2026-10-27 | Relative "Month −9 to Month 0" schedule (no ID) | SMART-QSO Team |
| A | 2026-10-27 | Replaced by a dated critical-path schedule from `software/simulation/cdr/pm/schedule_cpm.py`, with the integrator deliverable dates of SQSO-LVR-001 §9 as deadlines, float, scenarios and the regulatory timing (CDR review, PM/SE/V&V; finding PI-12) | CDR review, PM/SE/V&V |
| B | 2026-10-27 | CDR 2026 consistency pass: planning launch 2028-11-01 (BASELINE Rev E §3, CR-2026-007) in the §1 key dates (L-relative dates from `schedule_cpm.py --launch 2028-11-01`); finding 2 marked adopted. Floats unchanged | CDR documentation review |
| C | 2026-10-27 | `schedule_cpm.py` defaults to the 2028-11-01 planning launch (was 2028-06-01); the §3 scenario table labels the June rows with their options (RID SE-16). Dates and floats unchanged | CDR S&MA analysis |
| D | 2026-10-27 | CDR round 3 (P3P-05, P3P-10, P3P-01, REL-01, SE-05, SE-14). Schematic capture and the mechanical CAD complete; EM layout re-estimated per board from the round-2 designs (27 designer-weeks; two designers, 15 weeks); PDR board, pre-layout and pre-order verification items, NASA DAS (agreement 2026-11-06 to 2026-12-15, run 2027-01-08, ODAR Draft E 2027-01-12) and the FCC consultation (request 2026-11-15, meeting 2027-01-05, outcome 2027-01-08) before the early FCC filing (2027-01-19), IARU letter by 2027-05-18 (one date), integrator data at the contract, battery abuse tests, and the reliability programme (≥ 500 h EM flatsat, ≥ 200 h FM burn-in) added as tasks; CDR 2027-09-28; float +71 d for 2028-11 (meets GOLD 1.06 by 0.5 weeks), −80 d for June 2028; §5 decision points and dated actions DA-01 to DA-21 with owners, dates and criteria | CDR round 3, systems and programmatics |
| E | 2026-10-27 | Board firmware tasks (finding P3A-03): FW_REQ (requirements complete), TNC_DES/EPS_DES into the PDR package, TNC_CODE/EPS_CODE, TNC_UT/EPS_UT, TNC_EM/EPS_EM before the EM tests and the 500 h, TNC_QUAL/EPS_QUAL before the FM functional test; owner flight software lead (PI until named); DA-22 (the firmware languages); milestones. Critical path and floats unchanged (+71 d for 2028-11, −80 d for June 2028); least float of the new tasks 92 d | CDR round 3, software systems |
