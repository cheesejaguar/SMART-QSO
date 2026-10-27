# CDR Governing Framework, Tailoring and Criteria Compliance Matrix
## SMART-QSO 1U CubeSat

**Document ID**: SQSO-PM-002
**Version**: C
**Date**: 2026-10-27
**Status**: Review record of the CDR panel's programmatics, systems engineering and V&V reviewer. Assessment only;
it approves nothing.
**Governing values**: `docs/BASELINE_PARAMETERS.md` Rev E (amended)

---

## 0. Verdict

> **Documentation audit note (2026-10-27).** The panel is AI reviewers (Claude), not an independent board
> (`docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md` §1.2); "independent" in the recommendation below means independent of
> the design work only, and the PDR and CDR it recommends are the independent reviews still to be held. Where the Rev A
> text below gives other dates or floats (CDR 2027-08-03; June 2028 late by 21 days; 18.6 weeks of float), the Rev B
> status box above and `mission/SCHEDULE.md` (SQSO-SCH-001 E) govern.

> **Status at Rev B (CDR round 3, 2026-10-27; RID P3P-06).** Rev A (below) recorded the state before the round-2 design
> completion. Since then every custom board has a schematic captured as code with parts, BOM and ERC, the mechanical
> CAD and FE model exist, and the FMEA, safety data package, security plan and technology readiness assessment exist.
> Rows 6.3, 6.20, 6.24, 6.26, 6.27, 6.30 and 6.32 of §3 and rows 1 and 13 of §4 are restated (their Rev A text kept in
> italics). Counts at Rev B: entrance items 3 met, 21 met with action, 8 not met, 5 N/A; success criteria 0 met, 7 met
> with action, 11 not met, 2 N/A. The verdict stands: no layout, no hardware, no EM test, so the gate is the
> independent PDR on 2027-01-12 (board named by 2026-12-01, `mission/SCHEDULE.md` DA-03) and the CDR on **2027-09-28**
> with EM data (SQSO-SCH-001 D: the per-board layout estimate and the reliability programme moved it from 2027-08-03).
> Against the integrator's dates the November 2028 plan has 71 days of float (10.1 weeks against the 9.6 weeks GOLD
> Rule 1.06 wants); the June 2028 opportunity is 80 days late.

**The project does not meet the NPR 7123.1D Table G-7 entrance criteria for a CDR, and a combined PDR/CDR is not
acceptable at this maturity.** Of the 37 entrance items assessed (5 top-level criteria and the 32 work products of
criterion 6), 2 are met, 15 met with action, 15 not met and 5 not applicable; of the 20 success criteria, none is met,
6 are met with action, 12 not met and 2 not applicable (§3, §4). The detailed design does not exist as a build-to product: there
are no schematics in a CAD tool, no PCB layouts, no mechanical CAD or drawing package, no finite-element model, no
hardware FMEA and no EEE parts selection with procurement status. Nothing has been built or tested, so success
criterion 12 (engineering units tested per plan) cannot be met. The baseline itself records the project as Phase B,
pre-PDR, with no independent review held (BASELINE §1).

Recommendation (§2.3): treat this panel as an independent **pre-PDR technical assessment**; hold an independent
**PDR** against Table G-6 on 2027-01-12 once this review's RIDs are closed and EM schematic capture is done; hold the
**CDR** against Table G-7 on 2027-08-03 with the engineering-model test results, before flight-model fabrication. Those
dates are the early dates of the critical-path schedule (`mission/SCHEDULE.md`). Against the integrator's deliverable
dates (`docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` §9), **a 2028-06-01 launch is not achievable**: the protoflight
campaign ends 21 days after the L − 4 month milestone (2028-02-01), 9.6 weeks short of the GSFC margin. A launch around
2028-11 has 18.6 weeks of float against 10.2 wanted; the reviewer recommends planning on it.

## 1. Governing NASA Framework

### 1.1 What applies, and on which path

SMART-QSO is privately funded, built and licensed by an individual amateur (licensee KK6GDB, BASELINE §1), has no host
organisation, and is not a NASA program or project. **No NASA directive is mandatory for it.** NASA documents apply in
three ways: (a) as a reference framework the project adopts voluntarily and tailors (this review, the classification);
(b) indirectly, when a document the project must meet invokes them (CDS Rev 14.1 §2.4.3 invokes NPR 8715.6; the FCC debris
rules reference NASA-STD-8719.14 practice and NASA DAS); (c) directly, only if NASA becomes a sponsor or partner.

Revisions were checked on 2026-10-27 at the sources cited.

| Document | Current revision (source) | As built today: private amateur, no NASA funding | NASA CSLI (ELaNa) launch | NASA centre partner (e.g., Ames under a Space Act Agreement, or NASA funding) |
|----------|---------------------------|---------------------------------------------------|--------------------------|----------------------------------------------------------|
| NPR 8705.4 Risk Classification for NASA Payloads | **8705.4B**, effective 2024-11-01, expires 2032-10-31; it replaced 8705.4A (NODIS; NASA OSMA release note 2024-12-02) | Not applicable: P.2b applies it to "NASA robotic programs and projects" managed under NPR 7120.5. "Class D" is a self-designation used as a reference (BASELINE §1 "proposed") | CSLI CubeSats are partner payloads, not NASA-classified payloads; LSP's interest is "do no harm" through LSP-REQ-317.01 | Applies if the work becomes a NASA project under NPR 7120.5 (the Mission Directorate sets the class; Class D expected). P.2d excludes projects under NPR 7120.8, which may adopt the Appendix D objectives |
| NPR 7120.5 Space Flight Program and Project Management | **7120.5F** with Change 4, effective 2021-08-03; NODIS lists an end date of 2026-08-03, which has passed; no approved 7120.5G was found (status unverified) | Not applicable | Not applicable | Applies only if the project is a NASA space-flight project: 7120.8A P.2b sends R&T projects to 7120.5 when directly funded by a flight program, when a flight program depends on them, above $365M LCC, with significant radioactive material, or by AA decision |
| NPR 7120.8 NASA Research and Technology Program and Project Management | **7120.8A**, effective 2018-09-14, expires 2028-09-14 (NODIS) | Not applicable; the closest NASA model for a PI-led technology demonstration | Not applicable | **The likely governing NPR** for a centre-sponsored technology-demonstration CubeSat (P.2c: R&T managed or funded by NASA). Its life cycle uses authority to proceed, project approval, periodic reviews, continuation assessments and closeout (Tables 4-2 to 4-6); it does not mandate a PDR and CDR, which the centre would add by tailoring |
| NPR 7123.1 Systems Engineering Processes and Requirements | **7123.1D**, effective 2023-07-05, expires 2028-07-05 (NODIS); no Rev E exists (the repository cited "Rev E", finding VV-11) | Reference framework, tailored (§2). Appendix G Table G-7 is the yardstick of this review | Reference only; LSP holds a Mission Readiness Review, not a CDR | Applies (both 7120.5 and 7120.8 projects use it), tailored. G.1.2: "Tailoring and customizing are expected" |
| NPR 7150.2 NASA Software Engineering Requirements | **7150.2D**, effective 2022-03-08 | Reference; flight software self-classified Class C (Appendix D; BASELINE §1) | Reference | Applies, including the Appendix C requirements mapping for Class C (still to do: SQSO-AUD-003 §1) |
| NASA-STD-8739.8 Software Assurance and Software Safety | **8739.8B**, 2022-09-08; replaced NASA-STD-8719.13 (cancelled 2020) | Reference | Reference | Applies with 7150.2D |
| GSFC-STD-7000 General Environmental Verification Standard | **7000B**, approved 2021-04-28 (standards.nasa.gov copy shows revalidation 2027-04-28); **Appendix B, Environmental Verification for CubeSats** (Everett and Hartz, 2018) | Engineering guidance where the provider is silent; this plan adopts its protoflight definitions (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C) | LSP-REQ-317.01 cites it | Applies at a GSFC-like centre; Ames would use its own centre standards, usually GEVS-derived |
| NASA CubeSat 101 (CSLI, 2017) | Single edition | Good-practice guidance | The CSLI process guide: MRR before delivery, deliverables to the mission integrator ≥ 1 month before | Guidance |
| LSP-REQ-317.01 Program Level Dispenser and CubeSat Requirements | **Rev B** (2014); no later revision found (unverified) | Not applicable | **Mandatory**: e.g., protoflight random MPE + 3 dB for 2 min/axis; bake-out ≥ 70 °C for 3 h (or 60 °C for 6 h) at ≤ 1 × 10⁻⁴ Torr (Table 1) | Mandatory only on a NASA LSP launch |
| NPR 8715.6 / NASA-STD-8719.14 Orbital debris | **8719.14C** (2021-11-05); DAS 3.2.6 | Mandatory **through CDS Rev 14.1 §2.4.3**; FCC disclosure under 47 CFR 97.207(g)(1)(vii) | Mandatory (ODAR to NASA) | Mandatory |
| Cal Poly CubeSat Design Specification | **Rev 14.1** (February 2022) | **Mandatory through the integrator** (dispenser compatibility) | Mandatory | Mandatory |
| SpaceX Rideshare Payload User's Guide | **v11**, August 2026 | **Mandatory through the integrator** on a Transporter flight (interim provider document, D7) | Not applicable unless the CSLI flight is a SpaceX rideshare | Same as today |
| 47 CFR Part 97 (97.113, 97.207, 97.211, 97.307) | eCFR current | **Mandatory** (the licence) | Mandatory | Mandatory, unless the partner requires a Part 5 station (D2) |

### 1.2 Consequences

1. The risk classification and the life-cycle reviews are **voluntary**. The project gains from them only if they are
   held independently and recorded honestly (findings PI-03, PI-04). The binding requirements come from the
   integrator (CDS Rev 14.1, the user guide and the ICD), the FCC and the IARU.
2. **If NASA Ames or another centre partners**, expect NPR 7120.8A with NPR 7123.1D, 7150.2D and 8739.8B tailored by the
   centre; a centre-run PDR and CDR; centre environmental-test standards; and NPR 8705.4B Appendix D objectives
   adopted by agreement. A partnership that directly funds the work may also bring federal-procurement terms, which
   decision D2 and the Part 97 pathway conditions must screen before signature.
3. **If CSLI** becomes the route (long shot, solicitation 2028, launch about 2030: BASELINE §3), LSP-REQ-317.01 Rev B
   adds its test table, a Mission Readiness Review and LSP's waiver process; CSLI also needs an eligible institution
   (BASELINE §1).
4. The NPR 7120.5F expiry date has passed with no replacement found. Nothing in this project depends on it.

## 2. Tailoring

### 2.1 Life-cycle reviews

| Review | NPR 7123.1D table | Tailored content | Planned date (`mission/SCHEDULE.md`) |
|--------|-------------------|------------------|--------------------------------------|
| SRR / MDR | G-3, G-5 | **Not held separately.** The funding audit (2026-10-14) and the baseline Rev D (2026-10-26) serve as the requirements and architecture baseline; the PDR assesses the Table G-5 criteria that remain open | — |
| This panel | — | Independent pre-PDR technical assessment: RIDs against the CDR criteria to show the gap | 2026-10-27 |
| **PDR** | **G-6** | Independent, with named reviewers outside the development team; RIDs recorded and dispositioned; one day | 2027-01-12 |
| **CDR** | **G-7** | Independent; held after the EM development tests; authorises FM fabrication | 2027-09-28 (2027-08-03 in Rev A) |
| Test readiness review | G-11 (SIR/TRR elements) | Before the FM protoflight campaign: procedures, facility, levels, FM functional baseline | 2028-01 (before PF_ENV) |
| Pre-ship review | G-13/G-14 (SAR/ORR elements, tailored) | Verification closure (VCRM), integrator data package, regulatory filings in hand, operations readiness | 2028-03-14 |
| Integrator acceptance (and LSP MRR on the CSLI path) | External | The integrator's checklist | At delivery |
| Post-launch assessment | G-17 (PLAR, tailored) | Commissioning complete, FCC in-space notification filed | L + about 2 weeks |
| Decommissioning | G-18/G-19 (DR, tailored) | Passivation, FCC post-space notification | End of mission |

### 2.2 Is a combined PDR/CDR acceptable?

NPR 7123.1D Appendix G neither forbids nor describes combining reviews; G.1.2 expects tailoring, and G.1 applies the
entrance criteria whether a review is held "in a one-step or two-step process". Combined design reviews are common for
Class D CubeSats. **A combined review is acceptable only if it meets both Table G-6 and Table G-7**: a combined review
held now cannot, because the Table G-7 detailed-design criteria (success 1, 4, 12, 13, 14, 16) are not met; one held at
CDR maturity (after the EM tests, August 2027) could. A combined August 2027 review is **not recommended**, because
three things before it need an independent design gate: the EM long-lead orders and the solar panels for both models
(January 2027, a 3-7 month lead), the IARU coordination request (the radio design, frequencies, power and emissions it
declares), and the integrator RFQ responses that the contract (by 2027-06-30) commits to. **Recommendation: a separate
PDR in January 2027 and a CDR in August 2027.**

### 2.3 Products at each review

Tailored from Table G-7 entrance criterion 6 (and the matching Table G-6 items). "Required" means the product must be
available to the reviewers before the review.

| # | G-7 entrance product (6.n) | PDR (2027-01) | CDR (2027-08) | Tailoring rationale |
|---|----------------------------|---------------|---------------|--------------------|
| 1 | Build-to specifications and trades | Trades complete | Required | — |
| 2 | Fabrication, assembly, integration and test plans and procedures | Plans | Required, procedures near baseline | — |
| 3 | Technical data package (schematics, ICDs, analyses) | Schematics captured | Required, with layouts | — |
| 4 | TPM and margin status | Required | Required | — |
| 5 | Operational limits and constraints | Draft | Required | — |
| 6 | Resource estimates and margins (mass, power, data, link, thermal) | Required | Required, with EM measurements | — |
| 7 | Acceptance plans | — | Required | Protoflight: the protoflight plan is the acceptance plan |
| 8 | Command and telemetry list | Required | Required | — |
| 9 | V&V plan | Required (baseline) | Required (updated) | — |
| 10 | Integration plans | Required | Required | — |
| 11 | Launch site operations plan | — | Preliminary | Integrator does the integration |
| 12 | Checkout and activation plan | — | Preliminary | Commissioning plan |
| 13 | Disposal plan | Required (ODAR) | Required | FCC 5-year rule |
| 14, 15 | Technology readiness assessment; technology development plan | Required (Jetson, transponder) | Required | `docs/project/TECHNOLOGY_READINESS_ASSESSMENT.md` (SQSO-TRA-001, all critical elements; RID SE-09) |
| 16 | Risk assessment | Required | Required | — |
| 17 | SEMP | Required | Required | — |
| 18, 19 | HSI approach; human-rating certification package | N/A | N/A | Uncrewed; no ISS path unless the backup triggers (then the JSC 20793 battery package, SYS-910) |
| 20 | Reliability analyses | Preliminary | Required | — |
| 21 | Life-cycle cost and integrated master schedule | Required | Required | — |
| 22 | Integrated logistics support plan | N/A | N/A | One flight unit; ground-station spares only |
| 23 | Project protection plan | Required (telecommand authentication and key management) | Required | Replaced by SQSO-ICD-013 and the key-handling rules (BASELINE §7) |
| 24 | Subsystem and operations safety analyses | Preliminary | Required | — |
| 25 | Certification plans | — | Required (integrator acceptance, FCC and IARU) | — |
| 26 | System safety analysis with verifications | Preliminary | Required | — |
| 27 | Software products per NASA-HDBK-2203 | Required | Required | Class C |
| 28, 29 | NTIA Stage 2 certification; Stage 4 parameter update | N/A | N/A | Amateur service: replaced by the IARU coordination and the FCC Part 97 filings |
| 30 | System security plan | Required (as item 23) | Required | — |
| 31 | Procurement status and supply-chain risk | Required | Required | — |
| 32 | Single-point-failure list with rationale | Preliminary | Required | — |

## 3. Table G-7 Entrance Criteria: Compliance at This Review

Status: **Met**; **Met with action** (the product exists and an action closes the gap before the next gate); **Not
met**; **N/A** (tailored out, §2.3). The RID column gives this review's RIDs (prefixes PM, SE and VV, listed in §6);
funding-audit findings are cited as "finding" followed by their ID, because the audit also used the prefix VV.

| # | Criterion | Status | Evidence | RID |
|---|-----------|--------|----------|-----|
| 1 | Prior life-cycle reviews held; RFAs and RIDs closed or planned | Not met | No independent SRR or PDR (BASELINE §1; `docs/funding_audit/FINDINGS_REGISTER.md` PI-03) | PM-01 |
| 2 | Agenda, success criteria and reviewer instructions agreed | Met with action | The chair's reviewer brief (not in the repository) gives the success criteria and instructions; the reviewers are not named people and no agenda is published | PM-02 |
| 3 | Lower-level reviews and peer reviews complete | Not met | No subsystem design reviews; flight-software code review is by CI and pull request only | PM-01 |
| 4 | Programmatic products at the governing NPR's maturity | Met with action | `docs/project/PROJECT_PLAN.md` Rev B, `mission/SCHEDULE.md` (SQSO-SCH-001 A), `docs/funding_audit/LIFECYCLE_COST_ROM.md` (ROM, no quotes) | PM-05, PM-06 |
| 5 | Baselined detailed design meeting all requirements or with waivers | Not met | No build-to design; predicted non-compliances SYS-100, SYS-002, SYS-603 (`docs/verification/VERIFICATION_MATRIX.md` §6) | SE-01, SE-07 |
| 6.1 | Build-to specifications and trades | Not met | Trades exist (`docs/hardware/BUILD_VS_BUY.md`); no build-to specifications | SE-01 |
| 6.2 | Fabrication, assembly, integration, test plans and procedures | Met with action | `docs/integration/INTEGRATION_PLAN.md`, `docs/integration/ASSEMBLY_PROCEDURE.md`, 42 procedures in `docs/test_procedures/`; levels corrected in this review | VV-01 |
| 6.3 | Technical data package (schematics, ICDs, analyses) | Met with action | **Rev B:** every custom board's schematic captured as code with selected parts, KiCad netlist, BOM with stresses and ERC (SQSO-EPS-BRD-001, -OBC-BRD-001, -RF-BRD-001, -JET-BRD-001; boards A2 and A3), master pin allocation SQSO-ICX-001, harness SQSO-HAR-001, CAD SQSO-MECH-CAD-001; ICDs exist (`docs/interfaces/`). Action: PCB layouts (2027-01-12 to 2027-04-27, DA-04); the integrator ICD at the contract (DA-08). *(Rev A: block-level schematic drawings only)* | SE-01, SE-05 |
| 6.4 | TPM and margin status | Met with action | `docs/project/SEMP.md` v2.0 §3.3; mass margin 0-4 %, tumbling power margin +19 % (< 20 %) | SE-07 |
| 6.5 | Operational limits and constraints | Met with action | `docs/BASELINE_PARAMETERS.md` §5-§7; `docs/operations/MISSION_RULES.md` | — |
| 6.6 | Resource utilisation and margins | Met with action | `docs/environment/MASS_BUDGET.md` v2.0, `docs/environment/LINK_BUDGET_ANALYSIS.md` v2.0; power budget partly superseded; thermal not computed (TB-07) | SE-07 |
| 6.7 | Acceptance plans | Met with action | Protoflight campaign, `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §4 | VV-01 |
| 6.8 | Command and telemetry list | Met | `docs/operations/COMMAND_DICTIONARY.md` generated from `software/rust/sqso-protocol` (CI check); `docs/operations/TELEMETRY_DICTIONARY.md` | — |
| 6.9 | Updated V&V plan | Met with action | `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C; VCRM Rev C with every requirement's event; `docs/verification/VERIFICATION_VALIDATION_PLAN.md` still the January text | VV-04 |
| 6.10 | Updated integration plans | Met with action | `docs/integration/INTEGRATION_PLAN.md` Rev B (dated in this review; no hardware yet) | VV-09 |
| 6.11 | Preliminary launch site operations plan | Met with action | `docs/operations/LAUNCH_SITE_OPS.md` Rev A; the integrator's process is unknown until contract | — |
| 6.12 | Preliminary checkout and activation plan | Met | `docs/operations/COMMISSIONING_PLAN.md` v2.0 | — |
| 6.13 | Preliminary disposal plan | Met with action | `docs/operations/END_OF_LIFE_PLAN.md`; ODAR draft, DAS run pending (`docs/regulatory/filings/ODAR_SMART-QSO.md`) | SE-14 |
| 6.14 | Updated technology readiness assessment | Not met | No TRL assessment of the Jetson in 1U, the custom transponder or the isolated battery mount | SE-09 |
| 6.15 | Technology development plan | Not met | The D5 energy gate and the EM thermal test are decisions, not a plan with dates and exit criteria (now verification items VI-01, VI-02) | SE-09 |
| 6.16 | Updated risk assessment | Met with action | `mission/RISK_REGISTER.md`: L/C scores but no 5×5 score, owners, trend or dated mitigations | PM-07 |
| 6.17 | Updated SEMP | Met with action | `docs/project/SEMP.md` v2.0 (this review) | — |
| 6.18 | HSI approach | N/A | §2.3 | — |
| 6.19 | Human-rating certification package | N/A | §2.3 | — |
| 6.20 | Updated reliability analyses | Met with action | **Rev B:** `docs/environment/RELIABILITY_ANALYSIS.md` 2.1 (RBD, allocation, Monte Carlo: 0.75 / 0.61 / 0.51) and the system FMEA and critical-items list SQSO-FMEA-002 C. Action: board-level FMEA of the round-2 boards (RID P3P-04); the PI's acceptance of the estimates (DA-02, 2026-12-15). *(Rev A: REL 1.2, no hardware FMEA)* | SE-08 |
| 6.21 | Updated life-cycle cost and integrated master schedule | Met with action | ROM SQSO-AUD-005 Rev B; `mission/SCHEDULE.md` with critical path; cost differences in PM-06 | PM-05, PM-06 |
| 6.22 | Integrated logistics support plan | N/A | §2.3 | — |
| 6.23 | Project protection plan | Met with action | `docs/interfaces/TELECOMMAND_AUTHENTICATION.md` (SQSO-ICD-013), draft, host-tested only | — |
| 6.24 | Subsystem and operations safety analyses | Met with action | **Rev B:** `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md` 1.1 (16 hazard reports with verifications VI-SAF-01 to -09), the battery safety assessment SQSO-BSA-001 D and the software hazard analysis. Action: the integrator's 91-710 requirements at the contract (DA-08). *(Rev A: skeleton)* | SE-08 |
| 6.25 | Certification plans and requirements | Not met | No integrator ICD; regulatory roadmap exists (`docs/regulatory/REGULATORY_ROADMAP.md`); no filing made | SE-05 |
| 6.26 | System safety analysis with verifications | Met with action | As 6.24: hazard reports carry their verifications, now in the generated register `docs/verification/VI_REGISTER.md` | SE-08 |
| 6.27 | Software products per NASA-HDBK-2203 | Met with action | **Rev B:** SDP, SRS, design and test plans and the NPR 7150.2D Class C mapping (`docs/software/NPR7150_COMPLIANCE_MATRIX.md` B). Action: the flight software on the target (HIL, development board by 2026-11-30); SWE-220 complexity (DA-16). *(Rev A: no Appendix C mapping)* | SE-10 |
| 6.28 | NTIA Stage 2 certification | N/A | Amateur service (§2.3) | — |
| 6.29 | Stage 4 spectrum parameters | N/A | Replaced by IARU coordination and the FCC filing | — |
| 6.30 | Baselined system security plan | Met | **Rev B:** Space System Protection and Security Plan SQSO-SEC-001 A (key custody and recovery, ground station; RID SE-11 closed). *(Rev A: no plan)* | SE-11 |
| 6.31 | Procurement status and supply-chain risk | Not met | No orders, no quotes; the 1U COTS market is shrinking (`docs/hardware/BUILD_VS_BUY.md` §2.1); `docs/project/HARDWARE_READINESS_TRACKER.md` v2.0 | PM-08 |
| 6.32 | Single-point failures with rationale | Met with action | **Rev B:** single points of failure with retention rationale in SQSO-FMEA-002 §3–§4 (CIL-05 to CIL-13). Action: SYS-020 wording (DA-20). *(Rev A: no list)* | SE-08 |

**Count (37 items): Met 2** (6.8, 6.12); **Met with action 15** (2, 4, 6.2, 6.4, 6.5, 6.6, 6.7, 6.9, 6.10, 6.11,
6.13, 6.16, 6.17, 6.21, 6.23); **Not met 15** (1, 3, 5, 6.1, 6.3, 6.14, 6.15, 6.20, 6.24, 6.25, 6.26, 6.27, 6.30, 6.31,
6.32); **N/A 5** (6.18, 6.19, 6.22, 6.28, 6.29).

## 4. Table G-7 Success Criteria: Compliance at This Review

| # | Criterion (NPR 7123.1D Table G-7) | Status | Evidence | RID |
|---|-----------------------------------|--------|----------|-----|
| 1 | Detailed design expected to meet the requirements with adequate margins | Not met | **Rev B:** the design is complete to schematic and analysis level with adequate margins (mass MEV 1.75 kg, +14.0 %; power 2.48 W against SYS-100's 2.21 W, +12 %; Jetson temperatures inside SYS-603 Rev G), but no layout or hardware exists and eight critical elements are at TRL 3 (`docs/project/TECHNOLOGY_READINESS_ASSESSMENT.md`). *(Rev A: no detailed design; mass 0-4 %, power −0.9 %, 0.44 against 1 W/K)* | SE-01, SE-07 |
| 2 | ICDs mature enough to proceed to fabrication | Not met | Internal ICDs exist; the launch ICD is not signed (contract by 2027-06-30); pin map provisional (`software/rust/firmware/src/gpio.rs`) | SE-05 |
| 3 | Cost and schedule credible and within constraints | Met with action | Dated critical path (`mission/SCHEDULE.md`); ROM without quotes; float −21 days to the integrator's L − 4 month milestone for a June 2028 launch, +130 days for November 2028; funding not committed (R11) | PM-05, PM-06 |
| 4 | High confidence in the product baseline; documentation ready for fabrication | Not met | No product baseline; CM plan had no tags or baselines (`docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` v1.1 fixes the process) | PM-04 |
| 5 | Verification and validation requirements and plans complete | Met with action | Every requirement has a method and a planned event (VCRM Rev C §3); protoflight plan with levels (ETP Rev C); 7 requirements unverifiable as written until the PI ratifies the proposed wording | VV-02, SE-04 |
| 6 | Testing approach comprehensive; AIT, launch-site and operations planning support the next phase | Met with action | ETP Rev C §4-§5, END_TO_END_TEST_PLAN, COMMISSIONING_PLAN; facilities not booked | VV-03 |
| 7 | Technical and programmatic margins and resources adequate | Not met | Schedule margin −9.6 weeks against GOLD Rule 1.06 for June 2028; mass and power margins below requirement; one contributor (R14) | PM-05, SE-07 |
| 8 | Safety and mission-success risks understood and manageable | Not met | Risk register without scores, owners or dated mitigations; system safety data package a skeleton | PM-07, SE-08 |
| 9 | SMA products show acceptable residual risk | Not met | No hardware FMEA, no system hazard analysis, no reliability re-issue | SE-08 |
| 10 | Compliance with applicable requirements and standards demonstrated | Not met | CDS compliance matrix partly rebuilt (finding VV-14); NPR 7150.2D Appendix C mapping missing | SE-02, SE-10 |
| 11 | TBD/TBR items identified with disposition plans | Met with action | `docs/TBD_REGISTER.md` with CI gate; requirements items now dated (MRD Rev D §8, SYS Rev E §15); many open items elsewhere | SE-06 |
| 12 | Engineering test units and models developed and tested per plan | Not met | No EM, flatsat or target hardware; the SIL and ADCS Monte Carlo exist (`software/rust/sqso-sil`) | VV-05 |
| 13 | Material property tests and analyses (loads, stress, contamination) complete | Met with action | **Rev B:** shell FE model (first mode 169 Hz, positive margins with NASA-STD-5001B factors; `docs/environment/STRUCTURAL_ANALYSIS.md` B); materials list SQSO-MAT-001 D with E595 data. Action: E595 per part number for the harness and connector items before the EM order (V-HAR-11, DA-18). *(Rev A: no FEM)* | SE-12 |
| 14 | EEE parts selected; testing and delivery support the build | Not met | `docs/manufacturing/EEE_PARTS_LIST.md` Rev B lists candidates, no screening plan, no orders | PM-08 |
| 15 | Operational concept at CDR detail and considered in test planning | Met with action | `mission/CONOPS.md`, `docs/verification/CONOPS_VALIDATION_MATRIX.md`; DITL and E2E in the ETP | — |
| 16 | Manufacturability included in design | Not met | No layouts or drawings to assess | SE-01 |
| 17 | Software components meet NASA-HDBK-2203 criteria | Met with action | Rust flight software built, tested in CI (coverage gated), Embassy firmware builds for the STM32L4R5ZI; no target execution; Appendix C mapping missing | SE-10 |
| 18 | HSI aspects sufficient | N/A | Uncrewed | — |
| 19 | Centre spectrum manager concurrence | N/A | Amateur service; the equivalent is the IARU coordination letter, not yet requested | SE-05 |
| 20 | Procurement and supply-chain risk management complementary with the schedule | Not met | No orders; panels 3-7 months lead set by cell supply; 1U vendors delisting | PM-08 |

**Count (20 criteria): Met 0; Met with action 6 (3, 5, 6, 11, 15, 17); Not met 12 (1, 2, 4, 7, 8, 9, 10, 12, 13, 14, 16,
20); N/A 2 (18, 19).**

## 5. Table G-6 (PDR) Success Criteria: Quick Look for the Recommended PDR

| # | PDR success criterion (summary) | Status now | To be ready by 2027-01-12 |
|---|---------------------------------|------------|---------------------------|
| 1 | Top-level requirements, success criteria and TPMs final and consistent | Met with action | PI ratifies the seven proposed wordings; CR-2026-007 logged |
| 2 | Flow-down complete, verifiable, traced | Met with action | Provider requirements traced (SYS-900 to SYS-919 to LVR-001 to LVR-114); re-check against the signed ICD; SRS methods for SRS-C* |
| 3 | Cost and schedule credible | Met with action | Quotes for the top cost lines; funding decision dated |
| 4 | Preliminary design meets requirements at acceptable risk | Met with action | Close the power and mass decisions (BASELINE §4-§5 notes) |
| 5 | Interface definitions match maturity | Met with action | Pin map from the schematic capture |
| 6 | New technology mature or backed up | Met with action | Fallback to the conventional transponder exists (MRD-034); Jetson TRL assessment |
| 7 | Risks understood, with plans and resources | Not met | 5×5 register with owners and dates |
| 8 | SMA addressed in the preliminary design | Not met | Preliminary hazard analysis and FMEA |
| 9 | Margins and resources adequate | Not met | Schedule margin; staffing |
| 10 | Operations concept sound | Met | `mission/CONOPS.md`, commissioning plan |
| 11 | Trades mostly complete | Met | `docs/hardware/BUILD_VS_BUY.md`, D1-D9 research |
| 12 | Compliance with requirements and standards | Met with action | CDS clause-by-clause matrix (finding VV-14) |
| 13 | TBD/TBR items planned | Met with action | Dated closure (MRD §8, SYS §15) |
| 14 | Preliminary subsystem analyses complete | Met with action | Thermal model computed (TB-07); power multi-orbit SIL |
| 15 | Modelling results considered | Met | SIL and ADCS Monte Carlo fed D8 and D9 |
| 16 | Heritage assessed | Met | `docs/project/HERITAGE_ASSESSMENT.md` (design-precedent survey) |
| 17 | Manufacturability considered | Met with action | COTS buys; custom boards to be captured |
| 18 | Software meets NASA-HDBK-2203 PDR criteria | Met with action | Appendix C mapping |
| 19 | HSI | N/A | — |
| 20 | Spectrum concurrence | N/A (IARU request after the PDR) | — |
| 21 | Procurement and supply chain aligned | Not met | RFQs for the COTS buys and the integrator (D7, by 2026-12-31) |

A PDR is reachable by 2027-01-12 if the four "not met" items are worked in the next ten weeks; the CDR criteria are
not reachable before the EM is built and tested.

## 6. RIDs Raised by the PM/SE/V&V Reviewer

Severity: 1 mission or safety critical; 2 must close before fabrication; 3 should fix. "Chair" means a decision or a
change outside the reviewer's documents. The review report gives the evidence and the exact changes requested.

| RID | Sev | Title | Status |
|-----|-----|-------|--------|
| PM-01 | 1 | No independent SRR or PDR; this panel cannot be a CDR | Chair: designate it a pre-PDR assessment (§2) |
| PM-02 | 3 | Review conduct: no agenda, no named reviewers, no RID dispositions or signatures | Chair |
| PM-03 | 2 | Wrong governing-standard citations (NPR 7123.1 "Rev E", NPR 7120.5 as governing, NPR 7150.2 "Rev B", GEVS "A") | Closed by design change in the PM/SE/V&V documents; others for the chair |
| PM-04 | 2 | No configuration baselines or tags; CM plan describes a branch model and CCB not in use | Closed by design change (`CONFIGURATION_MANAGEMENT_PLAN.md` v1.1); chair to tag at the PDR and log CR-2026-007 |
| PM-05 | 1 | A June 2028 launch misses the integrator's L − 4 month protoflight milestone by 21 days; earliest launch meeting the GOLD margin about 2028-09 | Closed by analysis (`mission/SCHEDULE.md`); chair: PI decision on the planning launch date and at-risk orders |
| PM-06 | 2 | Cost ROM differs from the bottom-up test cost and the schedule | Chair (ROM owner) |
| PM-07 | 2 | Risk register lacks scores, owners, dates and the schedule-margin and funding-timing risks | Chair (register owner) |
| PM-08 | 2 | No procurement or supply-chain status; panels lead time 3-7 months | Closed by design change (`HARDWARE_READINESS_TRACKER.md` v2.0); orders for the chair |
| PM-09 | 3 | Project plan, SEMP and tracker claimed reviews and a CDR that never happened | Closed by design change |
| SE-01 | 1 | No detailed (build-to) design | Chair: project action, EM schematic capture by 2027-01-05 |
| SE-02 | 2 | Requirements flow-down from CDS Rev 14.1, Part 97, NASA-STD-8719.14C and the provider missing | Closed by design change (MRD Rev D, SYS Rev E; LVR-001 to LVR-114 traced in SYS §13a) |
| SE-03 | 2 | Requirement values contradict the baseline | Closed by design change; chair logs CR-2026-007 |
| SE-04 | 2 | Seven requirements unverifiable until the PI ratifies the proposed wording | Chair (PI by the PDR) |
| SE-05 | 1 | External interfaces undefined: no integrator ICD, no IARU letter | Chair (RFQs by 2026-12-31; IARU request after the PDR) |
| SE-06 | 3 | TBD/TBR items in the specifications had no target dates | Closed by design change |
| SE-07 | 1 | Margins below requirement: SYS-100 −0.9 %, mass 0-4 %, Jetson heat path 0.44 against 1 W/K | Chair (PI decisions); VI-01 |
| SE-08 | 2 | No hardware FMEA, single-point-failure list, system hazard analysis or reliability re-issue | Chair (SMA discipline) |
| SE-09 | 2 | No technology readiness assessment or development plan for the Jetson in 1U and the custom transponder | Chair; VI-01, VI-02 define the exit criteria |
| SE-10 | 2 | NPR 7150.2D Appendix C mapping missing; SRS-C requirements have no method; SRS-F081 cited in code but undefined | Chair (software owner) |
| SE-11 | 3 | No system security plan for key custody, ground station and recovery | Chair |
| SE-12 | 2 | No structural model; materials list incomplete | Chair (structures) |
| SE-13 | 3 | SYS-112 depth of discharge stated without the nominal 10 % | Closed by design change (SYS Rev E) |
| SE-14 | 2 | NASA DAS not run; the ODAR and the FCC filing depend on it | Chair (regulatory) |
| VV-01 | 1 | Protoflight defined backwards; vibration below the GEVS workmanship floor; TVAC margin ± 5 °C, 4 cycles; no bake-out specification | Closed by design change (ETP Rev C, TP-ENV-002 E, TP-ENV-003 D); levels final at the ICD (VI-10) |
| VV-02 | 1 | VCRM used undefined IDs and claimed "Complete" without evidence; no planned events | Closed by design change (VCRM Rev C, `gen_vcrm.py`) |
| VV-03 | 2 | No test facilities identified or costed | Closed by analysis (ETP §7, `test_cost.py`); booking for the chair |
| VV-04 | 3 | Two V&V plans, one stale | Closed by design change (`mission/VNV_PLAN.md` superseded; VVP v1.3) |
| VV-05 | 1 | No engineering-model development tests before the CDR | Verification items VI-01 to VI-09 |
| VV-06 | 2 | Deployment tests after protoflight vibration conflict with CDS §3.5.2 | Closed by design change (ETP §3.8, TP-ENV-005 1.1) |
| VV-07 | 2 | Shock waiver cannot be assumed: vibe-equivalent SRS only 60-85 g at 1-2 kHz | Verification item VI-07 |
| VV-08 | 2 | TVAC hot plateaus limited by the battery (+60 °C against +70 °C GEVS and +66 °C SpaceX advised) | Verification item VI-03; deviation for PI approval (ETP §9) |
| VV-09 | 3 | Integration plan undated and without hardware status | Closed by design change (`docs/integration/INTEGRATION_PLAN.md` Rev B) |

---

## Revision History

| Version | Date | Description | Author |
|---------|------|-------------|--------|
| A | 2026-10-27 | Initial issue: governing framework and applicability by path (§1), tailoring of reviews and products (§2), Table G-7 entrance and success criteria assessed with evidence (§3, §4), Table G-6 quick look (§5) | CDR review, PM/SE/V&V |
| B | 2026-10-27 | CDR round 3, systems and programmatics (RID P3P-06): status note in §0; §3 rows 6.3, 6.20, 6.24, 6.26, 6.27, 6.30, 6.32 and §4 rows 1, 13 restated after the round-2 design completion (schematics as code, CAD and FE, FMEA, safety data package, security plan, TRA), Rev A text kept; §2.3 rows 14–15 point to SQSO-TRA-001; CDR date 2027-09-28 | CDR round 3 |
| C | 2026-10-27 | Documentation audit (post-CDR 2026): note in §0 on the panel's nature (AI reviewers, not an independent board) and on which dates govern | Documentation audit |
