# Integrated Master Schedule
## SMART-QSO 1U CubeSat, 2026-10-27 to a 2028 Launch

**Document ID**: SQSO-SCH-001
**Version**: B
**Date**: 2026-10-27
**Status**: Planning baseline from a critical-path model. No vendor quote, laboratory booking, integrator schedule or
funding commitment exists yet; each is an assumption below. Update at least every 30 days (finding PI-12).
**Model**: `software/simulation/cdr/pm/schedule_cpm.py` (run it to regenerate every date in this document)
**Integrator deliverables**: `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` (SQSO-LVR-001) §9, LVR-003 and LVR-007;
decision points DP-1 to DP-9 in `docs/ORBITAL_ACCESS.md` (SQSO-OAP-001)

This replaces the relative "9-month to flight delivery" schedule (Month −9 to Month 0), which assumed a heritage design
and an already-held PDR and CDR; neither exists (BASELINE §1).

---

## 1. Key Dates, Deadlines and Assumptions

| Item | Date | Basis |
|------|------|-------|
| Today | 2026-10-27 | — |
| Contracting legal entity in place | **by 2026-12-31** | DP-1 (`docs/ORBITAL_ACCESS.md`) |
| Integrator requests for quotes out | by 2026-12-31 | Decision D7 |
| EM funding available | **2027-01-15 (assumption)** | Needed for the EM long-lead buys after the PDR |
| Independent PDR (NPR 7123.1D Table G-6) | 2027-01-12 | `docs/project/CDR_CRITERIA_MATRIX.md` §2 |
| FM and launch funding committed; launch deposit | **2027-05-31 (assumption)** | Risk R11: funding decision by Q2 2027 |
| Launch contract signed (launch-vehicle determination) | 2027-05-31, deadline **2027-06-30** | Decision D7 |
| Independent CDR (Table G-7) with EM test data | 2027-08-03 | §3 |
| Planning launch | **2028-11-01**, the last Transporter-class rideshare of 2028 (was 2028-06-01, now only an opportunity) | BASELINE Rev E §3 (CR-2026-007), from finding 2 of §3; the SIL's deployment date |
| Mission-analysis inputs: CAD, dynamic model, mass properties ± 15 %, webforms | L − 8 months = **2028-03-01** (2027-10-01 for June) | LVR-003, LVR-005 (UG v11 Table G-3) |
| Hazardous-systems declaration, 91-710 letter | L − 6 months = 2028-05-01 | LVR-008 |
| Protoflight environmental campaign complete | **L − 4 months = 2028-07-01** (2028-02-01 for June) | LVR §9 (UG v11 Fig. 1-2) |
| Environmental verification report, final mass properties ± 5 % | **L − 3.5 months = 2028-07-17** | LVR-007 |
| Delivery to the integrator | L − 50 days = 2028-09-12 | LVR §9 (date set by the mission ICD) |
| Final charge and integration into the dispenser; no access afterwards | L − 47 days = 2028-09-15 | LVR §9 |

The binding deadline is the **end of the protoflight campaign at L − 4 months**, not the delivery date.

## 2. Schedule

Early dates (the same for any launch date) and total float to the binding deadlines, for the June 2028 date (the
planning launch until BASELINE Rev E) and for the November 2028 planning launch. Negative float (bold) means the task cannot finish in time on the June date.

| ID | Task | Duration | Early start | Early finish | Float, L = 2028-06-01 | Float, L = 2028-11-01 |
|----|------|----------|-------------|--------------|-----------------------|-----------------------|
| ENTITY | Contracting legal entity in place (ORBITAL_ACCESS DP-1) | 8 wk | 2026-10-27 | 2026-12-22 | 9 d | 9 d |
| FCC_CONS | FCC Space Bureau consultation (D3) | 10 wk | 2026-10-27 | 2027-01-05 | 267 d | 420 d |
| RFQ | Integrator requests for quotes (D7: by 2026-12-31) | 9 wk | 2026-10-27 | 2026-12-29 | 2 d | 2 d |
| RID | Close this review's RIDs; requirements rebaseline (CR-2026-007) | 6 wk | 2026-10-27 | 2026-12-08 | 7 d | 158 d |
| SCH | EM schematic capture: OBC, EPS, transponder, Jetson carrier, battery mount | 10 wk | 2026-10-27 | 2027-01-05 | **-21 d** | 130 d |
| TCS | Name >= 2 telecommand stations (D1) | 10 wk | 2026-10-27 | 2027-01-05 | 267 d | 420 d |
| QUOTES | Quotes, evaluation, selection | 8 wk | 2026-12-29 | 2027-02-23 | 43 d | 43 d |
| PDR | Independent PDR (NPR 7123.1D Table G-6), held as a peer review | 1 wk | 2027-01-05 | 2027-01-12 | **-21 d** | 130 d |
| DAS | NASA DAS run and ODAR final (after MEL re-run) | 4 wk | 2027-01-12 | 2027-02-09 | 400 d | 553 d |
| IARU_REQ | IARU coordination request submitted (form v40) | 2 wk | 2027-01-12 | 2027-01-26 | 260 d | 413 d |
| LAYOUT | EM PCB layout and design-rule review | 8 wk | 2027-01-12 | 2027-03-09 | **-21 d** | 130 d |
| MECH | Mechanical CAD and structural model (dynamic model, mass properties ±15 %) | 10 wk | 2027-01-12 | 2027-03-23 | 178 d | 330 d |
| EM_LL | EM COTS buys: frame, AntS, magnetometer, gyro, GNSS, Jetson (12 wk, BVB §2) | 12 wk | 2027-01-15 | 2027-04-09 | **-17 d** | 134 d |
| FUND_EM | EM funding available (assumption) | — | 2027-01-15 | 2027-01-15 | **-17 d** | 134 d |
| PANELS | Solar panels, EM and FM sets ordered together (3-7 months, BVB §2; 7 used) | 30 wk | 2027-01-15 | 2027-08-13 | 60 d | 211 d |
| IARU | IARU panel iteration to coordination letter (about 6 months) | 26 wk | 2027-01-26 | 2027-07-27 | 260 d | 413 d |
| NEGOT | Contract negotiation, altitude clause, export terms | 12 wk | 2027-02-23 | 2027-05-18 | 43 d | 43 d |
| EM_FAB | EM PCB fabrication and assembly | 5 wk | 2027-03-09 | 2027-04-13 | **-21 d** | 130 d |
| EM_INT | EM integration, flatsat, flight software on target (TP-FLATSAT-001) | 8 wk | 2027-04-13 | 2027-06-08 | **-21 d** | 130 d |
| CONTRACT | Contract signature and deposit (D7: by 2027-06-30); launch-vehicle determination | — | 2027-05-31 | 2027-05-31 | 30 d | 30 d |
| EXPORT | Export counsel; TAA if the integrator is foreign (3-6 months, R15) | 20 wk | 2027-05-31 | 2027-10-18 | 177 d | 330 d |
| FCC_PRE | FCC 97.207(g)(1) pre-space notification (<= 30 d after LV determination) | 4 wk | 2027-05-31 | 2027-06-28 | 289 d | 442 d |
| FUND_FM | FM and launch funding committed (R11: decision by Q2 2027) | — | 2027-05-31 | 2027-05-31 | 30 d | 30 d |
| MA_IN | Mission-analysis inputs to the integrator (L-8 M: CAD, dynamic model, webforms) | 2 wk | 2027-05-31 | 2027-06-14 | 109 d | 261 d |
| EM_TEST | EM development tests: Jetson heat path and burst energy, EM TVAC, pathfinder vibe, RF/EMC pre-scan, GNSS noise, antenna pattern | 8 wk | 2027-06-08 | 2027-08-03 | **-21 d** | 130 d |
| CDR | Independent CDR (Table G-7) with EM data | 1 wk | 2027-08-03 | 2027-08-10 | **-21 d** | 130 d |
| BATT | Flight-lot cell screening, pack build, LEO cycling at 4.10 V (D8) | 10 wk | 2027-08-10 | 2027-10-19 | **-7 d** | 144 d |
| CDR_RID | CDR RID closure | 3 wk | 2027-08-10 | 2027-08-31 | 225 d | 378 d |
| FM_LAYOUT | FM layout update from EM findings | 4 wk | 2027-08-10 | 2027-09-07 | **-7 d** | 144 d |
| FM_LL | FM COTS buys | 12 wk | 2027-08-10 | 2027-11-02 | **-21 d** | 130 d |
| SDHS | Hazardous-systems declaration and 91-710 letter (L-6 M) | 4 wk | 2027-08-10 | 2027-09-07 | 85 d | 237 d |
| FM_FAB | FM PCB fabrication, assembly, inspection | 6 wk | 2027-09-07 | 2027-10-19 | **-7 d** | 144 d |
| FM_INT | FM integration (TP-INT-001 to -006) | 6 wk | 2027-11-02 | 2027-12-14 | **-21 d** | 130 d |
| FM_FUNC | FM comprehensive functional and day-in-the-life (TP-SYS-001, -002) | 3 wk | 2027-12-14 | 2028-01-04 | **-21 d** | 130 d |
| PF_ENV | Protoflight campaign: mass properties, vibe, TVAC + bake-out, deployment hot/cold, EMC/self-compatibility, magnetic dipole, post-env CPT, transport (complete by L-4 M) | 7 wk | 2028-01-04 | 2028-02-22 | **-21 d** | 130 d |
| ENV_REPORT | Environmental verification report on the SpaceX template (L-3.5 M) | 1 wk | 2028-02-22 | 2028-02-29 | **-14 d** | 139 d |
| PSR | Pre-ship review and integrator data package | 3 wk | 2028-02-29 | 2028-03-21 | 22 d | 175 d |
| DELIVER | Delivery to the integrator | — | 2028-03-21 | 2028-03-21 | 22 d | 175 d |

## 3. Critical Path and Margin

**Critical path**: EM schematic capture → PDR → EM layout → EM fabrication → EM integration → EM development tests →
CDR → FM COTS buys → FM integration → FM functional and DITL → protoflight campaign (must end by L − 4 months).

**For a 2028-06-01 launch the float is −21 days**: the protoflight campaign ends on 2028-02-22, three weeks after the
integrator's 2028-02-01 milestone, and the environmental verification report (due 2028-02-15) is two weeks late.
GSFC-STD-1000 (GOLD Rules) Rule 1.06, as commonly applied (the text was not retrieved in this review), wants 1 month
per year to the start of I&T and 2 months per year from the start of I&T to the deadline: 6.6 weeks here. **Shortfall:
9.6 weeks.**

| Scenario (`schedule_cpm.py` options) | Protoflight campaign ends | Float to the binding deadline | Against the GOLD margin |
|--------------------------------------|---------------------------|-------------------------------|-------------------------|
| Baseline: launch 2028-06-01, EM funds 2027-01-15, FM long leads at the CDR | 2028-02-22 | **−21 d** | −9.6 weeks |
| Same, FM long leads ordered at risk at the PDR (`--fm-at-risk`) | 2028-02-08 | −7 d | −7.8 weeks |
| Same, EM funds 2027-02-15 | 2028-03-20 | −48 d | −13.1 weeks |
| Launch 2028-07-01 | 2028-02-22 | +8 d | −6.1 weeks |
| Launch 2028-08-01 | 2028-02-22 | +39 d | −2.4 weeks |
| Launch 2028-09-01 | 2028-02-22 | +69 d | **meets** (9.9 against 8.7 weeks) |
| **Launch 2028-11-01** | 2028-02-22 | **+130 d** | **meets** (18.6 against 10.2 weeks) |
| Launch 2028-11-01, EM funds 2027-03-15 | 2028-03-20 | +75 d | meets (10.7 against 9.5 weeks) |
| Launch 2028-11-01, FM and launch funds 2027-09-30 | 2028-04-13 | +79 d | meets, but the contract misses the D7 deadline |

**Findings.**

1. **A June 2028 launch is not achievable on this plan**: with no hardware, the EM-then-FM sequence ends the
   protoflight campaign three weeks after the integrator's L − 4 month milestone, even before any margin. Ordering the
   FM long leads at risk at the PDR recovers two weeks, not three. Only removing the EM development tests (VI-01 to
   VI-09) would recover the June date, and they are the evidence the CDR needs (the Jetson heat path and the D5 energy
   gate in particular).
2. **The earliest launch that meets the GOLD margin is about 2028-09; a launch around 2028-11 has 18.6 weeks of float
   and absorbs a two-month slip in EM funding.** Recommendation to the PI and the chair: **baseline the last 2028
   rideshare (about 2028-11) for planning and funding**, write the June date out of the RFQs, and use the D7 contract's
   rebooking clause (BASELINE §3) only for a further slip. *Adopted by the chair: BASELINE Rev E §3 (CR-2026-007).*
3. The administrative deadlines have almost no float: the legal entity (DP-1) by 2026-12-31 has 9 days, the RFQs 2
   days, and the contract 30 days to the D7 deadline. They do not move with the launch date.
4. **Regulatory timing**: the FCC pre-space notification is due within **30 days of launch-vehicle determination**
   (2027-06-30 if the contract is signed on 2027-05-31), while the IARU letter arrives about **2027-07-27**. File the FCC
   notification early, with the IARU request in January 2027 (the FCC accepts notifications before launch-vehicle
   determination when the orbital range is definite, DA 13-445 fn. 6, `docs/regulatory/REGULATORY_ROADMAP.md` §2), and
   file the replacement after the contract. The IARU request goes in 14.7 months before integration for a June launch
   (19.6 months for November); the roadmap target is 18 months and the floor 12.
5. **Mission-analysis inputs at L − 8 months** (2027-10-01 for June) need the mechanical CAD and a structural dynamic
   model; neither exists. Task MECH (10 weeks after the PDR) gives 109 days of float, but only if a mechanical designer
   starts in January 2027.
6. **The ISS (NRCSD) backup** needs its contract about 12 months before launch, so the D7 backup decision on
   2027-09-30 cannot support a mid-2028 launch; the earliest NRCSD launch from that decision is about 2028-09
   (launch reviewer, `docs/ORBITAL_ACCESS.md`). This is one more reason to plan on late 2028.
7. Export: a foreign integrator (Exolaunch, ISISPACE) may need a TAA for the integration services (3-6 months, risk
   R15); it fits with 177 days of float only if export counsel is engaged at the contract.

## 4. Milestone Summary

| Milestone | Date |
|-----------|------|
| Contracting legal entity (DP-1); integrator RFQs out | 2026-12-22; 2026-12-29 |
| **PDR** | **2027-01-12** |
| IARU request and early FCC pre-space notification | 2027-01-26 |
| EM boards assembled | 2027-04-13 |
| **Launch contract** (deadline 2027-06-30) | **2027-05-31** |
| EM integrated, flight software running on the target | 2027-06-08 |
| IARU coordination letter (expected) | 2027-07-27 |
| **CDR** | **2027-08-03** |
| Mission-analysis inputs to the integrator | by 2027-10-01 (June launch) |
| FM integration start | 2027-11-02 |
| Protoflight campaign | 2028-01-04 to 2028-02-22 |
| Environmental verification report | 2028-02-29 (due 2028-02-15 for June; 2028-07-17 for November) |
| Pre-ship review; delivery | 2028-03-21 (earliest) |
| **Launch** | **2028-06-01 not achievable on this plan; plan on about 2028-11** |

---

## Revision History

| Version | Date | Description | Author |
|---------|------|-------------|--------|
| — | before 2026-10-27 | Relative "Month −9 to Month 0" schedule (no ID) | SMART-QSO Team |
| A | 2026-10-27 | Replaced by a dated critical-path schedule from `software/simulation/cdr/pm/schedule_cpm.py`, with the integrator deliverable dates of SQSO-LVR-001 §9 as deadlines, float, scenarios and the regulatory timing (CDR review, PM/SE/V&V; finding PI-12) | CDR review, PM/SE/V&V |
| B | 2026-10-27 | CDR 2026 consistency pass: planning launch 2028-11-01 (BASELINE Rev E §3, CR-2026-007) in the §1 key dates (L-relative dates from `schedule_cpm.py --launch 2028-11-01`); finding 2 marked adopted. Floats unchanged | CDR documentation review |
