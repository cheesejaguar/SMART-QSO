# Integrated Master Schedule
## SMART-QSO 1U CubeSat, 2026-10-27 to a 2028 Launch

**Document ID**: SQSO-SCH-001
**Version**: A
**Date**: 2026-10-27
**Status**: Planning baseline from a critical-path model. No vendor quote, laboratory booking, integrator schedule or
funding commitment exists yet; each is an assumption below. Update at least every 30 days (finding PI-12).
**Model**: `software/simulation/cdr/pm/schedule_cpm.py` (run it to regenerate every date in this document)

This replaces the relative "9-month to flight delivery" schedule (Month −9 to Month 0), which assumed a heritage design
and an already-held PDR and CDR; neither exists (BASELINE §1).

---

## 1. Key Dates and Assumptions

| Item | Date | Basis |
|------|------|-------|
| Today | 2026-10-27 | — |
| Integrator requests for quotes out | by 2026-12-31 | Decision D7 |
| EM funding available | **2027-01-15 (assumption)** | Needed for the EM long-lead buys after the PDR |
| Independent PDR (NPR 7123.1D Table G-6) | 2027-01-12 | `docs/project/CDR_CRITERIA_MATRIX.md` §2 |
| FM and launch funding committed; launch deposit | **2027-05-31 (assumption)** | Risk R11: funding decision by Q2 2027 |
| Launch contract signed (launch-vehicle determination) | 2027-05-31, deadline **2027-06-30** | Decision D7 |
| Independent CDR (Table G-7) with EM test data | 2027-08-03 | §3 |
| Target launch | **2028-06-01** (Transporter-class rideshare) | BASELINE §3; the orbital-lifetime analysis uses this deployment date |
| Delivery to the integrator | latest 2028-04-02 (L − 60 days) | Typical rideshare integrator practice; to be replaced by the contract date |
| Dispenser integration | about 2028-04-17 (L − 45 days) | Sets the FCC 90-day limit |

## 2. Schedule (early dates; float to the 2028-04-02 delivery)

| ID | Task | Duration | Early start | Early finish | Total float |
|----|------|----------|-------------|--------------|-------------|
| RID | Close this review's RIDs; requirements rebaseline (CR-2026-007) | 6 wk | 2026-10-27 | 2026-12-08 | 47 d |
| SCH | EM schematic capture: OBC, EPS, transponder, Jetson carrier, battery mount | 10 wk | 2026-10-27 | 2027-01-05 | **19 d** |
| RFQ | Integrator requests for quotes | 9 wk | 2026-10-27 | 2026-12-29 | 180 d |
| TCS | Name ≥ 2 telecommand stations (D1) | 10 wk | 2026-10-27 | 2027-01-05 | 257 d |
| FCC_CONS | FCC Space Bureau consultation (D3) | 10 wk | 2026-10-27 | 2027-01-05 | 257 d |
| PDR | Independent PDR | 1 wk | 2027-01-05 | 2027-01-12 | **19 d** |
| IARU_REQ | IARU coordination request (form v40) | 2 wk | 2027-01-12 | 2027-01-26 | 250 d |
| DAS | NASA DAS run and ODAR final | 4 wk | 2027-01-12 | 2027-02-09 | 390 d |
| LAYOUT | EM PCB layout and design-rule review | 8 wk | 2027-01-12 | 2027-03-09 | **19 d** |
| EM_LL | EM COTS buys (frame, AntS, magnetometer, gyro, GNSS, Jetson) | 12 wk | 2027-01-15 | 2027-04-09 | 23 d |
| PANELS | Solar panels, EM and FM sets ordered together | 30 wk | 2027-01-15 | 2027-08-13 | 100 d |
| QUOTES | Quotes, evaluation, selection | 8 wk | 2026-12-29 | 2027-02-23 | 180 d |
| NEGOT | Contract negotiation (altitude clause, export terms) | 12 wk | 2027-02-23 | 2027-05-18 | 180 d |
| EM_FAB | EM PCB fabrication and assembly | 5 wk | 2027-03-09 | 2027-04-13 | **19 d** |
| EM_INT | EM integration, flatsat, flight software on the target | 8 wk | 2027-04-13 | 2027-06-08 | **19 d** |
| CONTRACT | Contract signature and deposit | — | 2027-05-31 | 2027-05-31 | 30 d to the D7 deadline |
| FCC_PRE | FCC pre-space notification (≤ 30 days after the contract) | 4 wk | 2027-05-31 | 2027-06-28 | 2 d to the 30-day rule |
| EXPORT | Export counsel; TAA if the integrator is foreign | 20 wk | 2027-05-31 | 2027-10-18 | 167 d |
| IARU | IARU panel iteration to the coordination letter | 26 wk | 2027-01-26 | 2027-07-27 | 250 d |
| EM_TEST | EM development tests (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §6) | 8 wk | 2027-06-08 | 2027-08-03 | **19 d** |
| CDR | Independent CDR | 1 wk | 2027-08-03 | 2027-08-10 | **19 d** |
| FM_LL | FM COTS buys, ordered at the CDR | 12 wk | 2027-08-10 | 2027-11-02 | **19 d** |
| FM_LAYOUT | FM layout update from the EM findings | 4 wk | 2027-08-10 | 2027-09-07 | 33 d |
| FM_FAB | FM PCB fabrication, assembly, inspection | 6 wk | 2027-09-07 | 2027-10-19 | 33 d |
| BATT | Flight-lot cell screening, pack build, LEO cycling at 4.10 V (D8) | 10 wk | 2027-08-10 | 2027-10-19 | 33 d |
| FM_INT | FM integration (TP-INT-001 to -006) | 6 wk | 2027-11-02 | 2027-12-14 | **19 d** |
| FM_FUNC | FM comprehensive functional and day-in-the-life | 3 wk | 2027-12-14 | 2028-01-04 | **19 d** |
| PF_ENV | Protoflight environmental campaign | 7 wk | 2028-01-04 | 2028-02-22 | **19 d** |
| PSR | Pre-ship review and integrator data package | 3 wk | 2028-02-22 | 2028-03-14 | **19 d** |
| DELIVER | Delivery to the integrator | — | 2028-03-14 | 2028-03-14 | **19 d** |

## 3. Critical Path and Margin

**Critical path**: EM schematic capture → PDR → EM layout → EM fabrication → EM integration → EM development tests →
CDR → FM COTS buys → FM integration → FM functional and DITL → protoflight campaign → pre-ship review → delivery.

**Float on the critical path: 19 days (2.7 weeks).** GSFC-STD-1000 (GOLD Rules) Rule 1.06, as commonly applied (the
text was not retrieved in this review), wants 1 month per year to the start of I&T and 2 months per year from the
start of I&T to shipment: **8.0 weeks** here. **Shortfall: 5.3 weeks.** The plan has no funded schedule reserve.

| Scenario (`schedule_cpm.py` options) | Earliest delivery | Float | Against the GOLD margin |
|--------------------------------------|-------------------|-------|-------------------------|
| Baseline (EM funds 2027-01-15, FM long leads at the CDR, launch 2028-06-01) | 2028-03-14 | +19 d | −5.3 weeks |
| EM funds 2027-02-15 | 2028-04-10 | −8 d | misses delivery |
| EM funds 2027-03-15 | 2028-05-08 | −36 d | misses delivery |
| FM and launch funds 2027-09-30 (contract slips past 2027-06-30) | 2028-05-04 | −32 d | misses delivery and the D7 deadline |
| FM long leads ordered at risk at the PDR | 2028-02-29 | +33 d | −3.5 weeks |
| Launch on a later 2028 flight (2028-11-01), FM long leads at the PDR | 2028-02-29 | +186 d | meets the rule |

**Findings.**

1. A 2028-06-01 launch is feasible only if EM money is available by **2027-02-07** (the EM-funding float is 23 days)
   and nothing on the critical path slips more than 19 days. Any EM test failure that needs a board re-spin (6 to 11
   weeks) moves delivery past the deadline.
2. The cheapest margin is **ordering the FM COTS buys at risk at the PDR** (+14 days of float), and the
   most robust is **planning on the last 2028 rideshare** (about 2028-11) while keeping the June flight as the
   stretch target. The D7 contract's no-penalty rebooking clause (BASELINE §3) is what makes the second option cheap.
   **Recommendation to the PI: baseline the late-2028 flight for planning and funding, and treat June 2028 as an
   opportunity.**
3. The regulatory chain has float but one hard timing conflict: the FCC pre-space notification is due within **30 days
   of launch-vehicle determination** (2027-06-30 if the contract is signed on 2027-05-31), while the IARU letter arrives
   about **2027-07-27**. File the FCC notification early, together with the IARU request in January 2027 (the FCC
   accepts notifications before launch-vehicle determination when the orbital range is definite, DA 13-445 fn. 6,
   `docs/regulatory/REGULATORY_ROADMAP.md` §2), and file the replacement notification after the contract.
4. The IARU request goes in **14.7 months** before integration; the roadmap target is 18 months and the floor 12. The
   PDR date is what holds it back (the request declares the radio design). Prepare the request before the PDR.
5. Export: a foreign integrator (Exolaunch, ISISPACE) may need a TAA for the integration services (3-6 months, risk
   R15); it fits with 167 days of float only if export counsel is engaged at the contract.

## 4. Milestone Summary

| Milestone | Date |
|-----------|------|
| RIDs of this review closed | 2026-12-08 |
| Integrator RFQs out | 2026-12-29 |
| **PDR** | **2027-01-12** |
| IARU request and early FCC pre-space notification | 2027-01-26 |
| EM boards assembled | 2027-04-13 |
| **Launch contract** (deadline 2027-06-30) | **2027-05-31** |
| EM integrated, flight software running on the target | 2027-06-08 |
| IARU coordination letter (expected) | 2027-07-27 |
| **CDR** | **2027-08-03** |
| FM integration start | 2027-11-02 |
| Protoflight campaign | 2028-01-04 to 2028-02-22 |
| Pre-ship review | 2028-03-14 |
| **Delivery to the integrator** (latest) | **2028-04-02** |
| **Launch** | **2028-06-01** (planning alternative 2028-11) |

---

## Revision History

| Version | Date | Description | Author |
|---------|------|-------------|--------|
| — | before 2026-10-27 | Relative "Month −9 to Month 0" schedule (no ID) | SMART-QSO Team |
| A | 2026-10-27 | Replaced by a dated critical-path schedule from `software/simulation/cdr/pm/schedule_cpm.py`, with float, scenarios and the regulatory timing (CDR review, PM/SE/V&V; finding PI-12) | CDR review, PM/SE/V&V |
