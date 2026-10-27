# Project Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-PM-001
**Revision:** B
**Date:** 2026-10-27
**Reference:** NPR 7123.1D and NPR 7120.8A as reference frameworks, tailored (`docs/project/CDR_CRITERIA_MATRIX.md`
§1-§2); `docs/BASELINE_PARAMETERS.md` Rev D governs every value

---

## 1. Purpose and Status

This plan states how the SMART-QSO project is organised, scheduled, resourced, reviewed and controlled.

**Status on 2026-10-27: Phase B, pre-PDR** (BASELINE §1). No independent life-cycle review has been held; the SRR,
PDR and CDR records under `docs/reviews/` are self-assessments (findings PI-03, PI-04). No hardware has been bought,
built or tested. The flight software (Rust, Embassy executor) builds and is tested in CI against a simulated HAL and
builds for the STM32L4R5ZI; it has not run on hardware. Revision A of this plan listed the SRR and PDR as "Complete"
and the CDR as "Current"; that was wrong.

## 2. Mission

SMART-QSO is a 1U CubeSat carrying a single-channel U/V FM amateur transponder, duty-gated by available power, with
an AX.25 telemetry beacon and an onboard AI experiment that schedules the transponder and selects plain-language
beacon text. It falls back to a conventional transponder and template beacon when the AI is unavailable (BASELINE §2).
Objectives MO-1 to MO-5 and the mission requirements are in `docs/requirements/MISSION_REQUIREMENTS.md`; success
criteria (L1 30 days, L2 1 year, L3 2+ years) in `docs/mission/MISSION_SUCCESS_CRITERIA.md`.

## 3. Governing Framework

The project is privately funded and licensed under FCC Part 97 by an individual amateur licensee; no NASA directive
is mandatory. The binding requirements come from the FCC (47 CFR Part 97), the IARU coordination, CDS Rev 14.1 and the
integrator's ICD and launch-provider guide. NASA documents are used as a voluntary, tailored framework: NPR 7123.1D for
the systems-engineering processes and the review criteria, NPR 8705.4B for the Class D designation (a reference, since
the project is not a NASA payload), NPR 7150.2D Class C and NASA-STD-8739.8B for software, GSFC-STD-7000B for
environmental verification. If a NASA centre becomes a partner, NPR 7120.8A would be the likely management framework.
The applicability by path is in `docs/project/CDR_CRITERIA_MATRIX.md` §1.

## 4. Organisation

| Role | Holder | Notes |
|------|--------|-------|
| Principal investigator, project manager, systems engineer, space-station licensee | KK6GDB (one person, BASELINE §1, D1) | Single point of failure for the project (risk R14) |
| Control operators and telecommand stations | Not yet named; at least two geographically separated stations needed before the IARU request (D1) | Prospective partners: NASA Ames Research Center, Santa Clara University; nothing agreed |
| Electrical and RF, flight software, mechanical and thermal, integration and test | Open positions (labour model in `docs/funding_audit/LIFECYCLE_COST_ROM.md` §4: about 6.25 FTE-years) | Volunteer, university or contract |
| Independent reviewers for the PDR and CDR | To be invited by the PI; must not be developers | PDR 2027-01-12 |
| Host organisation | None (needed only for CSLI or institution-only funding) | BASELINE §1 |

Operators are not compensated for operations (Part 97 condition C2); paying people to build the spacecraft is
compatible with Part 97 under grant-type funding (ROM §4).

## 5. Work Breakdown Structure

The WBS follows the NASA standard project WBS used by the cost ROM (SQSO-AUD-005 §3), so cost and schedule share one
structure.

| WBS | Element | Content | Schedule tasks (`mission/SCHEDULE.md`) |
|-----|---------|---------|---------------------------------------|
| 01 | Project management | Plans, schedule, budget, risk, CM, reviews | RID, PDR, CDR, PSR |
| 02 | Systems engineering | Requirements, budgets, ICDs, ODAR and DAS, VCRM | RID, DAS |
| 03 | Safety and mission assurance | Hazard analysis, FMEA, battery screening, workmanship, inspections | BATT |
| 04 | Science / technology (AI experiment) | Selector model, gated generative experiment, D5 energy gate | EM_TEST (VI-02) |
| 05 | Payload (Jetson module and carrier) | Carrier design, thermal strap, power switch | SCH, LAYOUT, EM_FAB |
| 06 | Spacecraft bus | Structure (buy), EPS and battery, OBC, transponder, antennas (buy), ADCS, GNSS, harness | SCH to FM_INT |
| 07 | Mission operations | Control stations, SatNOGS, operations procedures, training | TCS |
| 08 | Launch services | Integrator contract, export counsel, logistics | RFQ, QUOTES, NEGOT, CONTRACT, EXPORT |
| 09 | Ground data system | Decoder, dashboards, orbit determination and OEM upload | — |
| 10 | Integration and test | Flatsat, EM tests, FM integration, protoflight campaign | EM_INT, EM_TEST, FM_INT, FM_FUNC, PF_ENV |
| 11 | Education and public outreach | Decoder releases, documentation | — |
| — | Regulatory (in 02 and 08) | IARU, FCC pre/in/post-space, ITU via FCC, NOAA, Space-Track | FCC_CONS, IARU_REQ, IARU, FCC_PRE |

## 6. Schedule

The integrated master schedule is `mission/SCHEDULE.md` (SQSO-SCH-001), generated by
`software/simulation/cdr/pm/schedule_cpm.py`. Key dates: PDR 2027-01-12; launch contract by 2027-06-30 (D7); CDR
2027-08-03; protoflight campaign 2028-01 to 2028-02; delivery to the integrator by 2028-04-02 for a 2028-06-01 launch.
**The critical path has 2.7 weeks of float against about 8 weeks of margin wanted (GSFC-STD-1000 Rule 1.06)**; the
schedule recommends planning on the late-2028 rideshare and treating June 2028 as an opportunity. The schedule is
updated at least every 30 days.

### 6.1 Decision gates (tailored key decision points)

There is no NASA decision authority; the PI decides at four gates, each after an independent review or a dated
record.

| Gate | Decision | Date | Entry |
|------|----------|------|-------|
| G1 | Proceed to EM build; release EM and both panel sets | After the PDR (2027-01-12) | PDR RIDs dispositioned; EM funding in hand |
| G2 | Sign the launch contract; file the FCC pre-space notification | By 2027-06-30 (D7) | Quotes; FM and launch funding committed; altitude clause |
| G3 | Proceed to FM fabrication | After the CDR (2027-08-03) | EM test results (VI-01 to VI-09) |
| G4 | Ship to the integrator | Pre-ship review (2028-03-14) | VCRM closed for every requirement due before delivery; filings in hand |

### 6.2 Reviews

| Review | Date | Criteria | Independence |
|--------|------|----------|--------------|
| PDR | 2027-01-12 | NPR 7123.1D Table G-6, tailored (`CDR_CRITERIA_MATRIX.md` §2.3, §5) | Named reviewers outside the team; RIDs and dispositions recorded in `docs/reviews/` |
| CDR | 2027-08-03 | Table G-7, tailored | As the PDR |
| Test readiness | 2028-01 | Procedures, facility, levels, FM functional baseline | PI and test lead, one independent reviewer |
| Pre-ship review | 2028-03-14 | VCRM closure, data package, filings | As the PDR |
| Post-launch assessment | L + about 2 weeks | Commissioning complete; FCC in-space notification filed | PI |

## 7. Resources and Budget

Cost: `docs/funding_audit/LIFECYCLE_COST_ROM.md` (ROM, no quotes): non-labour $0.28M-$0.72M including 30 % reserve;
labour from $0 (volunteer) to $0.9M-$1.3M (professional). Funding profile: about 45 % in FY2027 (EM, launch deposit,
export counsel) and 50 % in FY2028. The schedule needs **EM money by 2027-02-07** at the latest and **FM and launch
money by the contract** (assumed 2027-05-31).

| Phase | Period | Effort |
|-------|--------|--------|
| B to PDR | 2026-11 to 2027-01 | PI plus at least one electrical designer for the schematic capture |
| C (EM build and test) | 2027-01 to 2027-08 | Electrical/RF 1.0, flight software 1.0, mechanical/thermal 0.5, PI 0.75 FTE |
| D (FM build, test, delivery) | 2027-08 to 2028-04 | As C, plus integration and test 0.5 FTE |
| E (operations) | 2028 to 2030 | At least two trained control operators, uncompensated |

## 8. Risk Management

Process: one 5 × 5 register (`mission/RISK_REGISTER.md`), each risk with an owner, a likelihood and consequence score,
a trend and dated mitigation steps; reviewed monthly and at each gate. Risks rated high are reported in the
schedule update. Top risks today: funding timing (R11), key-person dependency (R14), launch capacity (R13), Jetson
feasibility in 1U (R16), regulatory timing (R19), and the schedule margin shortfall in §6 (to be added to the register:
RID PM-07).

## 9. Configuration Management

`docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` (SMART-QSO-CMP-001). Baselines: functional (requirements and the
baseline parameters) at the PDR, allocated (design and ICDs) at the CDR, product (as-built) at the pre-ship review,
each with a Git tag. Class I changes need a change request in `docs/project/CHANGE_REQUEST_LOG.md` approved by the PI.

## 10. Communication and Documentation

All documents are Markdown in the Git repository, controlled through pull requests and the document register
(`docs/DOCUMENT_REGISTER.md`). Status is reported in the 30-day schedule update and at each gate. No document is cited
externally unless its status is evidenced (risk R20).

## 11. Quality and Metrics

| Metric | Target | Today |
|--------|--------|-------|
| Requirements with a method and a planned event | 100 % | 100 % (VCRM Rev C §3) |
| Requirements verified with evidence | 100 % of those due before delivery | 0 %; 57 of 277 partly covered by CI tests |
| Open TBD/TBR items (CI-counted) | Downward each month; each with a dated closure | 265 / 34 before this review (`docs/TBD_REGISTER.md`) |
| Flight-software line coverage | ≥ 90 % Rust crates (CI gate) | 94.8 % (SQSO-AUD-003 §1.1) |
| Review RID closure | All severity 1 and 2 before the next gate | This review's RIDs: see `CDR_CRITERIA_MATRIX.md` §6 |
| Schedule float on the critical path | ≥ GOLD Rule 1.06 margin | 2.7 weeks against 8.0 |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | CDR review (PM/SE/V&V). Rewritten to the real status (Phase B, pre-PDR, no hardware; Rev A claimed the SRR and PDR complete); governing framework; organisation as it is (one person, open roles); WBS on the NASA standard structure of the cost ROM; dated schedule and decision gates G1-G4; tailored reviews; funding dates the schedule needs; risk and CM process; metrics with today's values | CDR review, PM/SE/V&V |
