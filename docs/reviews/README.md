# Review Documents (`docs/reviews/`)

Index of every file in this directory: what kind of document it is, its status, and the milestone of the integrated
master schedule ([`mission/SCHEDULE.md`](../../mission/SCHEDULE.md), SQSO-SCH-001) it belongs to. Revisions of the
controlled documents are in [`docs/DOCUMENT_REGISTER.md`](../DOCUMENT_REGISTER.md).

## What governs

- **Design values:** [`docs/BASELINE_PARAMETERS.md`](../BASELINE_PARAMETERS.md) (SQSO-BL-001). Where any document here
  disagrees with it, the baseline governs.
- **Current review record:** [`CDR/CDR_2026_DESIGN_REVIEW.md`](CDR/CDR_2026_DESIGN_REVIEW.md) (SQSO-REV-CDR-002 Rev B,
  2026-10-27), a design review against the NPR 7123.1D CDR criteria by AI reviewers. It is an engineering
  assessment, **not an independent review board**; its chair decisions C-1 to C-24 await the PI's ratification
  (DA-01, by 2027-01-12).
- **Project facts:** the project is in Phase B (pre-PDR). No independent SRR, PDR or CDR has been held, and no
  hardware, engineering model or PCB layout exists (BASELINE §1). The independent PDR is planned for 2027-01-12 and
  the independent CDR for 2027-09-28; the planning launch is 2028-11-01 (`mission/SCHEDULE.md` §4).

## Kinds of document

| Kind | Meaning | Rule |
|------|---------|------|
| Current record | The review record that is in force | Cite it; do not edit outside its own review |
| Historical | A record of something that did happen in this repository: the review packages and minutes generated on 2026-01-02, which the funding audit found to be self-assessments (findings PI-03, PI-04 in `docs/funding_audit/FINDINGS_REGISTER.md`) | Kept as written under a banner; not to be cited for any value, status or approval |
| Template | The form for a future review or event that has not happened | Banner at the top; no result, signature or status filled in until the event |
| Live | Kept current as the project changes | Updated in place, with a revision row |
| Pointer | A superseded file kept so that old links still resolve | Read the document it points to |

## Index

| File | Kind | Status (one line) | Milestone (`mission/SCHEDULE.md`) |
|------|------|-------------------|-----------------------------------|
| [`CDR/CDR_2026_DESIGN_REVIEW.md`](CDR/CDR_2026_DESIGN_REVIEW.md) | Current record | SQSO-REV-CDR-002 Rev B: complete, no open review action; engineering assessment by AI reviewers, not an independent board; design at strong-PDR maturity with CDR-depth analysis (§7.2) | Input to the independent PDR (2027-01-12) and CDR (2027-09-28) |
| [`CDR/CDR_REVIEW_MINUTES.md`](CDR/CDR_REVIEW_MINUTES.md) | Historical | SQSO-REV-CDR-001: self-assessed CDR minutes of 2026-01-02; "APPROVED" not substantiated; criteria re-assessed by the funding audit | Superseded by SQSO-REV-CDR-002; the CDR itself is 2027-09-28 |
| [`CDR/CDR_TECHNICAL_BASELINE.md`](CDR/CDR_TECHNICAL_BASELINE.md) | Historical | SQSO-CDR-TB-001 1.3: technical baseline of 2026-01-02; every value superseded by BASELINE Rev E (its own banners) | Superseded; none |
| [`CDR_PACKAGE.md`](CDR_PACKAGE.md) | Historical | SMART-QSO-CDR-001 1.6: CDR package of 2026-01-02; self-assessment, its PDR-completed claim untrue | Superseded; the CDR data package is prepared for 2027-09-28 |
| [`CDR_STATUS_SUMMARY.md`](CDR_STATUS_SUMMARY.md) | Historical | SQSO-REV-003 D: status summary of 2026-01-02; its "COMPLETE" and 100 % claims were never true | Superseded; none |
| [`PDR/PDR_REVIEW_MINUTES.md`](PDR/PDR_REVIEW_MINUTES.md) | Historical | SQSO-REV-PDR-001: self-assessed PDR minutes of 2026-01-02; "Approved" not substantiated | Superseded; the independent PDR is 2027-01-12 |
| [`PDR_PACKAGE.md`](PDR_PACKAGE.md) | Historical | SMART-QSO-PDR-001 1.4: PDR package of 2026-01-02; self-assessment | Superseded by the PDR data package (task PDR_PKG, to the board by 2026-12-15) |
| [`SYSTEM_REQUIREMENTS_REVIEW.md`](SYSTEM_REQUIREMENTS_REVIEW.md) | Historical | SQSO-SRR-001 1.2: SRR package of 2026-01-02; no SRR has been held | None: the SRR is not held separately; the PDR (2027-01-12) assesses the open Table G-5 criteria (`docs/project/CDR_CRITERIA_MATRIX.md` §2) |
| [`SAFETY_REVIEW.md`](SAFETY_REVIEW.md) | Historical | SQSO-SR-001 1.1: safety review of 2026-01-02; closes no hazard. Current: SQSO-SDP-002 (HR-01 to HR-16, none closed) and SQSO-SDP-001 | Superseded; hazard reports go to the integrator with the L − 6 month declaration (task SDHS, `mission/SCHEDULE.md`) |
| [`VERIFICATION_VALIDATION_PACKAGE.md`](VERIFICATION_VALIDATION_PACKAGE.md) | Historical | SQSO-REV-004 1.3: V&V package of 2026-01-02; nothing verified on hardware. Current: V&V plan SQSO-VVP-001, `RTM.csv`, `VI_REGISTER.md` | Superseded; none |
| [`TEST_READINESS_REVIEW.md`](TEST_READINESS_REVIEW.md) | Template | SQSO-TRR-001 1.1: not held | EM integration and flatsat (EM_INT, 2027-06-01 to 2027-07-27), between the PDR and the CDR |
| [`PRE_ENVIRONMENTAL_REVIEW.md`](PRE_ENVIRONMENTAL_REVIEW.md) | Template | SQSO-PER-001 1.2: not held; levels from the environmental test plan Rev E | Start of the protoflight campaign (PF_ENV, 2028-03-03) |
| [`POST_ENVIRONMENTAL_REVIEW.md`](POST_ENVIRONMENTAL_REVIEW.md) | Template | SQSO-PSR-001 1.2: not held. "PSR" there is the post-environmental review, not the schedule's pre-ship review | End of the protoflight campaign (2028-04-21) and the environmental verification report (due 2028-07-17) |
| [`MISSION_OPERATIONS_REVIEW.md`](MISSION_OPERATIONS_REVIEW.md) | Template | SQSO-MOR-001 1.1: not held; no operator trained | No task in the schedule; before the pre-ship review, after the FM day-in-the-life (2028-02-22) |
| [`FLIGHT_READINESS_REVIEW.md`](FLIGHT_READINESS_REVIEW.md) | Template | SQSO-FRR-001 1.1: not held | Pre-ship review and delivery to the integrator (earliest 2028-05-19; required by 2028-09-12, L − 50 d) |
| [`LAUNCH_READINESS_REVIEW.md`](LAUNCH_READINESS_REVIEW.md) | Template | SQSO-LRR-001 1.1: not held | Launch (planning date 2028-11-01) |
| [`FORMAL_INSPECTION_REPORT.md`](FORMAL_INSPECTION_REPORT.md) | Template | SQSO-FIR-001 1.0: blank NPR 7150.2 inspection form; no inspection recorded with it | Any time (software and design inspections) |
| [`KNOWN_ISSUES.md`](KNOWN_ISSUES.md) | Live | SQSO-KI-001 1.4: five flight, payload and ground software issues (two open, three accepted; no acceptance signed) | Reviewed at each milestone (its §6.3) |
| [`FLIGHT_SOFTWARE_DELIVERY.md`](FLIGHT_SOFTWARE_DELIVERY.md) | Live (template and status record) | SQSO-FSW-DEL-001 1.3: no release and no delivery; current build, test and memory figures | No task in the schedule; the flight build goes onto the FM at integration (FM_INT, 2027-12-21) and is frozen at the configuration freeze after the protoflight campaign (ETP §4 step 10) |
| [`SOFTWARE_VERSION_DESCRIPTION.md`](SOFTWARE_VERSION_DESCRIPTION.md) | Pointer | No number; merged into `docs/software/SOFTWARE_VERSION_DESCRIPTION.md` (SQSO-SVD-001) on 2026-10-16 | — |
| [`CONFIGURATION_AUDIT_CHECKLIST.md`](CONFIGURATION_AUDIT_CHECKLIST.md) | Pointer | No number (was SQSO-CA-001); merged into `docs/verification/CONFIGURATION_AUDIT_PROCEDURES.md` (SQSO-CAP-001) §4.4, §5.4, §7.4, §7.5 | Configuration audits (FCA/PCA) before delivery |
| [`CONFIGURATION_AUDIT_CHECKLISTS.md`](CONFIGURATION_AUDIT_CHECKLISTS.md) | Pointer | No number (was SQSO-CAC-001); merged into the same SQSO-CAP-001, Appendix B and §7.3 | Configuration audits (FCA/PCA) before delivery |
| `README.md` | Index | This file | — |

The two configuration-audit pointers are not duplicates of each other: each points to the parts of SQSO-CAP-001 that
came from its own former content. Older PDR material outside this directory is in `docs/pdr/`.
