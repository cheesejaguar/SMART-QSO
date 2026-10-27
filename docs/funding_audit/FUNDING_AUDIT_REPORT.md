# SMART-QSO Funding Audit Report

> **Historical record (2026-10-14, Rev B 2026-10-15).** Superseded for current values by `docs/BASELINE_PARAMETERS.md` and
> `docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md`. Since then the launch plan moved to a 475 km target with a 485 km cap and about 2028-11 (BASELINE Rev E §3), decisions D8 and D9 were taken, and the CDR 2026 design review (an engineering assessment by AI reviewers, not an independent board) closed with no review action open. Still good for: the verdict and the staged funding recommendation as they stood on 2026-10-15, and the integrity findings behind them.

**Document Number:** SQSO-AUD-001
**Revision:** B
**Date:** 2026-10-15
**Audience:** Prospective funders of design, construction, test and launch (e.g., NASA, NSF, amateur-radio foundations, university sponsors)
**Basis:** NASA practice for a proposed Class D CubeSat (NPR 8705.4B), the Cal Poly CubeSat Design Specification Rev 14.1, and current U.S. regulations (47 CFR Part 97, 15 CFR 774/960, 14 CFR 450)
**Baseline audited:** repository commit `0578833` (pre-audit). Corrections made by this audit are in later commits on the same branch.

---

## 1. Verdict

**SMART-QSO is not ready for a build-and-launch award. It is ready for a scoped formulation (Phase 1) award with
explicit go/no-go criteria.**

The repository is an extensive, design-phase documentation set (≈ 235 Markdown documents, C flight modules, Python
payload and ground software). It contains useful engineering intent. It also contains:

- **no hardware**
- **no executed hardware tests**
- **no independent design review.** The SRR, PDR and CDR records were generated in one day without named reviewers.
- **no licensee or named team**
- **no budget or dated schedule**
- **three mutually incompatible mission definitions**: an FM transponder with uplink, a beacon-only spacecraft with no
  receiver, and a spacecraft commanded on Federal frequencies
- **at least four incompatible power-system configurations**

Several documents presented status that did not exist. A NASA center was named as operator. NASA, DoD and NOAA spectrum
filings were described as "Submitted" or "Complete". Hardware was described as "Qualified" and "Tested". Coverage was
claimed at 80–90 % when the measured figure was 15 %. **A funder relying on those documents would have been misled.**
This audit has withdrawn or relabeled those claims.

The concept is technically plausible as a duty-gated transponder with a power-gated AI experiment in a 1U, but only
under conditions this audit defines: deployable wings, ≤ 10 % AI duty, a ≤ 500 km orbit, and a demonstrated Jetson heat
path. A credible path to a 2028 launch exists if funding and a launch reservation are secured by mid-2027 (§7).

**Findings:** 103 total. 19 Critical, 46 High, 32 Medium, 6 Low. 10 are closed and 59 partially closed by this audit's
changes; 33 remain open. Full register with file:line evidence: [FINDINGS_REGISTER.md](FINDINGS_REGISTER.md).

## 2. Scope and Method

| Area | Method |
|------|--------|
| Program integrity | Inventory of 104 status claims across all documents, each checked for evidence (6 substantiated, 10 partial, 30 unsubstantiated, 58 contradicted). Git history and review records examined |
| Technical budgets | Cross-document parameter table (28 parameters); independent power, link, thermal, mass, battery, lifetime, radiation and data calculations |
| Verification & safety | 29 requirements traced end to end; test levels checked against GEVS-7000B, CDS Rev 14.1 and SpaceX User's Guide v11; hazard analyses; standards currency |
| Software | Built and ran all tests (Apple clang and GCC 16); measured coverage; command-path probe harness; cppcheck; CI gate inventory |
| Regulatory | Primary-source review of 47 CFR 97.207/97.211/97.113/97.119/97.307 and Part 5 (eCFR, current 2026-10-13), FCC DA 13-445, IARU form v40 and instructions, 15 CFR 960 and 774 (ECCN 9A515), 14 CFR 450.43, NASA-STD-8719.14C and the NASA SSTP ODAR template |
| Launch | Primary sources dated 2026-10-14: NASA CSLI pages, SpaceX rideshare pricing/flight API and User's Guide v11, NASA State-of-the-Art Small Spacecraft Technology (May 2026), Voyager NRCSD IDD Rev A, LSP-REQ-317.01 Rev B, CDS Rev 14.1 |

Every finding cites file:line at commit `0578833`. Numbers computed by the audit are labeled as such. Prices without a
primary source are labeled ROM.

## 3. Readiness Scorecard

| Area | Rating | Basis |
|------|--------|-------|
| Program management & integrity | **Red** | No PI, licensee, team, budget or dated schedule; self-generated reviews; false affiliation and status claims (PI-01…PI-14) |
| Mission definition & technical baseline | **Red → Amber** | Three mission definitions and four power families (PI-13, TB-01). The audit proposes a single baseline ([BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md)) pending PI approval |
| Power / thermal feasibility of the AI payload | **Amber** | Closes on allocation with wings and ≤ 10 % duty (TB-02). The thermal path is unproven (≈ 0.44 W/K documented vs ≥ 1 W/K needed, TB-07) |
| Flight software | **Red** | Host simulation only; no RTOS or target port (SW-01). The 45-min RF silence is now implemented in simulation (SW-05). Authenticated telecommand, key management and a latched TX-OFF now work end to end in simulation from the receiver HAL; the target receiver driver and hardware verification are open. After the audit: 24 suites, 359 tests passing; overall line coverage was 44.5 % before the telecommand work (not re-measured on the same basis) |
| Payload & ground software | **Amber** | 143 payload and 98 ground tests now pass in CI. The OBC–Jetson protocol is implemented three incompatible ways (SW-24); AI text needs content controls (SW-12) |
| Verification & test program | **Red** | Hardware results claimed with no hardware; requirements traceability broken (2 of 29 trace cleanly); test levels inconsistent with GEVS/CDS/provider (VV-01, VV-06, VV-10) |
| Safety & mission assurance | **Red** | Catastrophic battery hazard closed without test; inhibits do not meet CDS (3+3); no battery safety program (VV-03…VV-05) |
| Regulatory / licensing | **Amber** | Part 97 path determined, with conditions on funding structure. All filings drafted on the correct forms. Nothing filed; licensee not named (RG-03) |
| Launch access | **Amber** | Path identified (commercial SSO ≤ 500 km); no quotes; reported capacity limits after 2028 (LA-01…LA-03) |
| Cost & schedule | **Red → Amber** | ROM produced ($0.28M–$2.02M cash depending on labor model); a dated plan exists (§7) but is not yet an owned schedule |

## 4. Critical Findings (all 19)

| Theme | Findings | Summary |
|-------|----------|---------|
| Integrity of claims | PI-01, PI-02, PI-03, RG-02 | NASA named as operator, owner and copyright holder; Federal spectrum filings "Submitted/Complete"; reviews self-generated. *Corrected or relabeled by this audit; the real team and reviews remain open* |
| Licensability | RG-01, RG-03, SW-02, SW-05 | The beacon-only design has no receiver, so it cannot cease emissions by telecommand (97.207(b), RR 22.1). The audit implemented authenticated TX-OFF and its receive path in flight software (simulation). Exercising it needs a selected receiver, its driver and the TP-RF-006 hardware test. The 45-min RF silence is implemented in simulation; hardware verification is pending. No licensee |
| Flight software maturity | SW-01 | Host-only simulation; safety-critical modules never called |
| Technical baseline | TB-01, TB-02, TB-03, TB-04 | Impossible battery figure; payload energy not closed; lifetime at 400 km shorter than design life; transponder undefined |
| Launch-safety compliance | VV-01, VV-02, VV-03, VV-04, VV-05 | "Qualified/Tested" with no hardware; 30-min RF silence vs the required 45; inhibits below CDS; no battery safety program; catastrophic hazard closed without test |

## 5. Decisions from the PI

The PI took D1–D7 on 2026-10-15. The decisions and their remaining actions are in `docs/BASELINE_PARAMETERS.md` §9; the
research behind them is in [research/](research/).

| ID | Decision (2026-10-15) | Why it gated funding |
|----|----------|----------------------|
| D1 | Licensee KK6GDB, station KK6GDB-11; telecommand stations TBD (NASA Ames and Santa Clara University prospective partners only) | No filing can be signed without it |
| D2 | Paid contracts allowed, but only on a separate Part 5 radio held by a separate licensee (HuskySat-1 pattern); the amateur station takes no compensation and carries no paid content. Grants and donations until a contract is signed | Otherwise Part 97 is lost ([pathway §7](../regulatory/LICENSING_PATHWAY_DETERMINATION.md)) |
| D3 | Loss-of-contact N = 30 days at launch, 14 after commissioning; M = 180 days proposed, not implemented; both to the FCC consultation | Removes the largest licensing uncertainty |
| D4 | Keep the duty-gated transponder | Scope and cost |
| D5 | OBC selector as the flight baseline; Jetson generation gated on measured ≤ 0.1 Wh per accepted beacon ([selection](../payload/AI_MODEL_SELECTION.md)) | Power and thermal closure |
| D6 | Add a GNSS receiver (piNAV-NG/FM; Celeste alternate) | Launch-provider acceptance (SpaceX User's Guide v11 §9.5.7.2) |
| D7 | Transporter-class SSO via a 1U integrator, contract by 2027-06-30 with a 525 km altitude cap; ROM $115k–250k all-in; NRCSD backup | Launch-vehicle determination starts the 30-day FCC clock |

## 6. Launch and Rideshare (summary)

Full analysis: [LAUNCH_RIDESHARE_TRADE_STUDY.md](LAUNCH_RIDESHARE_TRADE_STUDY.md).

- **Baseline:** SpaceX Transporter SSO via a 1U-capable integrator, deployment ≤ 500 km. SpaceX posts $7,500/kg for
  2027 with a 50 kg minimum, so a 1U buys through an integrator; budget ROM $90k–$150k. Most 1U-capable dispensers are
  foreign-owned, which brings ITAR integration-service controls.
- **Backup:** ISS NRCSD. Simplest debris case, but a 0.7–1.9 yr life and the heaviest battery-safety burden.
- **Not available:** NASA CSLI for a 2027–28 launch (next solicitation anticipated 2028).
- **Time-critical:** trade press reports SpaceX closing new bookings beyond late 2028. Reserve by mid-2027.

## 7. Dated Plan to a 2028 Launch (critical path in bold)

| Window | Milestone | Regulatory / launch actions |
|--------|-----------|-----------------------------|
| 2026-10 to 2026-12 | **D1–D3 decisions**; team named; funding structure agreed | AMSAT-NA/ARRL consultation; **FCC informal consult**; NOAA informal consultation; **integrator quote requests (by 2026-12-31)**; export counsel engaged |
| 2027-01 to 2027-03 | **Phase 1 award**; rebuild power, thermal and link analyses at baseline; flatsat build; FSW target port begins (TX-OFF, telecommand authentication, RF silence first) | Preliminary DAS run; D6 decision; TAA scoping |
| 2027-04 to 2027-06 | **Independent PDR** (≥ 3 named external reviewers; RID log) | **IARU request submitted** (instructions: "right after" PDR; ≥ 12 months before integration); **launch reservation signed by 2027-06-30 (launch-vehicle determination)** |
| 2027-07 | — | **FCC pre-space notification filed within 30 days of LV determination** (with Appendix 4 data and ODAR) |
| 2027-07 to 2027-12 | EM build; Jetson thermal and power test (D4, D5); independent CDR; flight-part procurement | IARU coordination letter expected; TAA if needed; Space-Track account |
| 2028-01 to 2028-04 | PFM integration and functional test; FSW qualification on target hardware | Replacement pre-space notification if anything changed (≥ 90 days before integration) |
| 2028-04 to 2028-06 | Environmental test campaign at provider levels (random vibe ≥ 5.57 Grms MPE with inhibit verification; TVAC; bakeout); pre-ship review | Provider deliverables (hazard declaration L-6, FAA payload-review info L-5, licensing letter L-2); 18 SDS registration by L-2 |
| 2028-07 to 2028-09 | Delivery to integrator (L-2 to L-1 months) | — |
| 2028-Q3/Q4 | **Launch** (NET; per the integrator's manifest) | 45-min RF silence; **FCC in-space notification ≤ 7 days after first transmission** |

The repository's "9-month kickoff-to-delivery" schedule (`mission/SCHEDULE.md`) cannot accommodate IARU coordination,
the FCC 90-day rule or export lead times, and is superseded by this plan for funding purposes.

## 8. Cost (summary)

Full ROM with basis: [LIFECYCLE_COST_ROM.md](LIFECYCLE_COST_ROM.md). Non-labor including a 30 % reserve is
**$0.28M–$0.72M**. Labor is the dominant uncertainty, at ≈ 6.25 FTE-years to launch:

| Labor model | Total cash |
|-------------|-----------|
| Volunteer | $0.28M–$0.72M |
| University | $0.58M–$1.32M |
| Professional | $1.18M–$2.02M |

## 9. Recommended Funding Structure

| Stage | Scope | Indicative non-labor | Exit (go/no-go) criteria |
|-------|-------|----------------------|--------------------------|
| **Phase 1: Formulation** (≈ 6–9 months) | Close the integrity findings; name the team and licensee; rebuild budgets; flatsat; FSW regulatory functions on target hardware; independent PDR; IARU request; integrator quotes | ≈ $40k–$130k (flatsat/EM parts, Jetson dev hardware, export counsel) | (1) Licensee named and IARU request submitted. (2) Independent PDR passed with RIDs dispositioned. (3) Power simulation closes at baseline duty with ≥ 20 % margin. (4) TX-OFF, authenticated telecommand and 45-min RF silence demonstrated on target hardware. (5) Written integrator quote at ≤ 500 km with an export path. (6) Preliminary DAS run complies |
| **Phase 2: Build & test** | EM thermal/power test; CDR; PFM; environmental campaign | Balance of hardware and test ROM | EM Jetson heat path ≥ 1 W/K; CDR passed; environmental tests passed; FCC pre-space filed |
| **Phase 3: Launch & operations** | Launch services; 2 years of operations (uncompensated, Part 97) | Launch ROM $90k–$150k + ground station | Delivery accepted by integrator |

Funding agreements must not compensate anyone for operating the station, nor take data rights. That is required to keep
Part 97 valid ([pathway §5](../regulatory/LICENSING_PATHWAY_DETERMINATION.md)).

## 10. Regulatory and Delivery Paperwork Produced

| Document | Purpose | State |
|----------|---------|-------|
| [Licensing pathway determination](../regulatory/LICENSING_PATHWAY_DETERMINATION.md) | Part 97 vs Part 5 vs NTIA; conditions; funding guidance | Determination (needs D1–D3) |
| [Regulatory roadmap](../regulatory/REGULATORY_ROADMAP.md) | All filings, deadlines, sequence | Plan |
| [IARU coordination request (v40)](../regulatory/filings/IARU_COORDINATION_REQUEST_v40.md) | Frequency coordination | Draft, not submitted |
| [FCC 97.207(g)(1) pre-space notification](../regulatory/filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md) | FCC filing incl. debris disclosures | Draft, not filed |
| [FCC in-space / post-space notifications](../regulatory/filings/FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md) | 97.207(g)(2)/(3) | Templates |
| [ITU Appendix 4 data](../regulatory/filings/ITU_APPENDIX4_DATA.md) | API/notification via FCC | Draft |
| [ODAR](../regulatory/filings/ODAR_SMART-QSO.md) | NASA-STD-8719.14C / FCC debris | Draft; DAS pending |
| [NOAA remote-sensing determination](../regulatory/filings/NOAA_REMOTE_SENSING_DETERMINATION.md) | 15 CFR 960 | Determination (no license) |
| [Export classification memo](../regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md) | EAR/ITAR | Self-classification; counsel review pending |
| [Space safety registration & conjunction plan](../regulatory/filings/SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md) | 18 SDS / TraCSS; 97.207(g)(1)(iv)–(v) | Plan |
| [Launch provider deliverables](../regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md) | CDS 14.1 items, battery package, FAA payload info | Checklist |
| [Baseline parameters](../BASELINE_PARAMETERS.md) | Single source of truth | Proposed |
| [Standards compliance matrix](NASA_STANDARDS_COMPLIANCE_MATRIX.md) | NASA standards and regulations | Assessment |

## 11. What This Audit Changed vs. Flagged

**Changed (on this branch):**

- **Integrity corrections:**
  - false NASA copyright/approval attributions removed from 17 files and 3 documents
  - regulatory status documents rewritten (SPECTRUM_LICENSE Rev B, IARU_COORDINATION Rev B, REGULATORY, RF_PLAN)
  - audit notices on 11 review/status records and 7 superseded technical/compliance documents
  - RALPH task items pointing at non-existent files corrected
  - README and Executive Summary rewritten with evidenced status
- **Requirements:**
  - MRD-003 and SYS-803 changed to the FCC 5-year rule
  - MRD-013 added (deployment altitude)
  - SYS-224 added (45-min RF silence)
  - SYS-225 added (three physical inhibits)
  - SYS-204 added (cease emissions by authenticated telecommand, latched across resets)
  - SYS-205 added (loss-of-contact transmit inhibit)
  - SYS-325 added (authenticated, replay-protected telecommand)
  - SYS-326 added (keys loaded only through the umbilical; on-orbit rotation to a spare)
  - SYS-800 (permanent passivation) updated; SYS-802 drag device deleted
  - CDS matrix timer and inhibit rows corrected
- **Software:**
  - payload CRC defect fixed with an ICD-conformant CRC-16 (check value 0x29B1) and tests
  - payload dependencies declared; a payload test corrected to fail safe
  - 8 existing flight test suites registered (6 → 14 suites; 76 → 243 tests; coverage 15.3 % → 44.5 %), including fixes to 2 incorrect test expectations
  - flight tests link via pkg-config's cmocka target
  - authenticated telecommand and latched TX-OFF implemented in simulation (SW-03 and SW-04 closed; SW-02 and SW-14
    partially closed; interface in [TELECOMMAND_AUTHENTICATION.md](../interfaces/TELECOMMAND_AUTHENTICATION.md)):
    - HMAC-SHA-256 tags with persisted per-key replay counters; `0xDEADBEEF` and the second command-ID table removed
    - TX-OFF latch that drops the TX-enable line before persisting, survives reset and fails safe; accepted without a
      key when no key is usable (decision 2026-10-15)
    - receive path from the 435 MHz receiver HAL through AX.25 to the command handler, tested end to end
    - key table with spares and an offline recovery key, `KEY_ROTATE` with no key material on RF, and key loading only
      through the GSE umbilical
    - command dictionary generated from one JSON source for flight and ground, checked in CI
    - `SET_MODE` through the state-machine transition table; `SET_POWER` command
    - CI secret scan; ground telecommand and key-provisioning tools whose bytes match the flight vectors
    - loss-of-contact transmit inhibit: after N days without an authenticated command the transmitters stop and the
      receiver stays on; N is set by telecommand (1–90 days; 30 days at launch, 14 after commissioning, D3); the
      M-day autonomous passivation is not implemented
    - TX-OFF hardware test procedure TP-RF-006 (not yet run)
    - 45-minute post-deployment RF silence: whole radio unpowered; a power-on reset or switch toggle restarts it; a
      watchdog reset never shortens it; no telecommand can shorten it; the GSE re-arms it as the last action before
      shipping
    - deployment sequence (SW-10): persisted and reset-safe, separate burn and sense lines per element, attempts
      persisted before each burn and bounded across resets, the 30-minute wait restarted by a power-on reset or switch
      toggle; one GSE `ARM_FLIGHT` action re-arms the RF silence and the deployment
    - 10 new suites (14 → 24 suites; 243 → 359 tests)
- **Analysis tools:** NRLMSIS 2.1 orbital lifetime estimator with tests (`software/simulation/orbit_lifetime/`).
- **CI:** new "Program Checks" workflow runs payload, ground and simulation tests, plus a documentation-reference
  integrity check, on every pull request. Previously docs/payload/ground PRs ran no checks. The duplicate workflow name
  is resolved.
- **New documents:** this report, the findings register, the trade study, the cost ROM, the standards matrix, the
  baseline and all regulatory drafts listed in §10.

**Flagged, not changed:**

- Flight-software functional gaps (SW-01, SW-08, SW-13…SW-16; the open parts of SW-02, SW-05, SW-10 and SW-14, chiefly
  the target receiver driver, which waits on the transceiver selection, and running TP-RF-006 on hardware)
- The OBC–Jetson protocol mismatch (SW-24)
- Subsystem design documents that still carry superseded values (TB findings)
- Traceability (VV-06); test procedures and test levels (VV-09, VV-10); the hazard log (VV-05)
- Document-ID collisions (PI-17)

Fixing these needs design decisions or hardware, not documentation edits.

### 11.1 Follow-on Closure Work (2026-10-15)

After the PI's decisions D1–D7 (§5), a second pass closed or advanced most open items. The register status line
gives the current counts.

- **Flight software:**
  - Beacon content filter on the OBC and the Jetson, with shared vectors (SW-12).
  - Beacon telemetry from measurements with validity flags, and a beacon cycle in main (SW-13).
  - A guarded mode manager as the single source of the power mode (SW-14).
  - A supervised watchdog that stops refreshing when a task stalls (SW-15).
  - OBC-enforced Jetson power and temperature limits (SW-16).
  - Assertions active in flight builds (SW-08).
  - Function-length and dynamic-memory gates (SW-07).
  - A GNSS NMEA driver (D6).
  - A compile-only STM32L4 port with a register-level HAL, cross-compiled in CI (SW-01).
- **CI gates now block:**
  - pinned tools, a line-coverage floor (80 %; 82.7 % measured in CI over every compiled source, 2026-10-16), and sanitizers
    with undefined behaviour fatal (this found a real bug). The 80 % floor and 84 % first reported counted only the
    sources the tests ran, leaving out four with about 930 untested lines;
  - no-regression gates for clang-tidy, the MISRA checker and function length;
  - parameter consistency between code and documents;
  - requirements traceability (SW-17, VV-08, VV-06).
- **Decisions recorded:**
  - BASELINE Rev B, with research reports in [research/](research/);
  - loss-of-contact timeout 30 days at launch (D3);
  - AI model selection (D5);
  - GNSS receiver (D6);
  - integrator plan and cost (D7);
  - paid-contract terms (D2).
- **Integrity:**
  - verification status consolidated in the VCRM (PI-06);
  - unverified hazard closures reopened (PI-07);
  - CDR criteria re-assessed, and the project stated as Phase B (PI-04);
  - JPL and MISRA compliance claims withdrawn (SW-07).

## 12. Actions Only the Team Can Take

These cannot be done from the repository:

1. Designate the control operators and at least two telecommand stations (licensee KK6GDB designated, D1).
2. Consult the FCC Space Bureau.
3. Submit the IARU request (satcoord@iaru.org).
4. File the FCC pre-space notification.
5. Obtain NASA DAS 3.2.6 (Software Usage Agreement) and run it.
6. Obtain at least two integrator quotes.
7. Retain export counsel.
8. Request NOAA's written confirmation.
9. Register on Space-Track.
10. Recruit independent reviewers.
11. Execute funding agreements that meet the Part 97 conditions (and, for a paid contract, the Part 5 terms in the
    licensing pathway §7.1).
12. ~~Turn on branch protection so the CI gates are required before merge.~~ Done 2026-10-16: a repository ruleset on
    `main` requires the CI checks (SW-17).

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial funding audit report |
| B | 2026-10-15 | §5 decisions D1–D7; §8 cost totals; §11.1 follow-on closure work; §12 updated |
