# NPR 7150.2D Requirements Mapping and Compliance Matrix (Class C)

**Document ID**: SQSO-FSW-012
**Version**: B
**Date**: 2026-10-27
**Status**: Draft (CDR)
**Applies to**: the Class C software of `docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1 (OBC flight software, Jetson
payload software, ground software used in operations, the host simulation and test suites, and since Version B the
firmware of the RF board TNC and of the EPS controller, SDP §3.4)
**Standards**: NPR 7150.2D (effective 2022-03-08), Appendix C; NASA-STD-8739.8B

---

## 1. Purpose and Basis

SOFTWARE_DEVELOPMENT_PLAN §3.3 and the funding-audit standards matrix list the NPR 7150.2D Appendix C mapping for
Class C as missing (programmatics finding SE-10). This document is that mapping: one row per SWE requirement, its
status for SMART-QSO, the evidence, and the tailoring where the project does not meet it as written.

Basis and limits:

- SMART-QSO is not a NASA project and applies NPR 7150.2D as a tailored reference (SDP §3.1). Requirements that bind a
  NASA Center, the Office of the Chief Engineer or the NASA IV&V facility are listed as "not applicable (Center/OCE)".
- The SWE numbers and short titles below are paraphrased from NPR 7150.2D. The Class C applicability column of
  Appendix C was read from the NASA Software Engineering Handbook's web rendering, which does not show every cell
  unambiguously. **Open item C-1:** the software assurance lead checks each row's applicability against the signed
  PDF of Appendix C before the TRR. No row's status depends on a cell whose reading is doubtful without saying so.
- Status values: **Met** (evidence in the repository), **Partial** (part met, the rest has a named closure),
  **Not met** (a RID or open item), **Tailored** (met in a different form, with the approving record), **N/A**.

Evidence paths are relative to the repository root.

## 2. Matrix

### 2.1 Software Management (NPR 7150.2D Chapter 3)

| SWE | Requirement (short) | Status | Evidence / tailoring |
|-----|---------------------|--------|----------------------|
| 013 | Software plans developed, maintained, executed | Met | `docs/software/SOFTWARE_DEVELOPMENT_PLAN.md`, `docs/software/SOFTWARE_TEST_PLAN.md`, `docs/software/SOFTWARE_MAINTENANCE_PLAN.md`, `docs/safety/SOFTWARE_SAFETY_PLAN.md`, `docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` |
| 015 | Software cost estimate | Partial | Project cost ROM (chair-owned CDR documents); no separate software estimate. Closure: software line in the fabrication-phase plan |
| 151 | Cost estimate parameters | Partial | As SWE-015 |
| 016 | Software schedule | Partial | `mission/SCHEDULE.md` carries software milestones; no software-level schedule |
| 018 | Regular reviews of software activities with stakeholders | Met | CDR reviews (`docs/reviews/`), pull-request reviews |
| 017 | Training plan for software personnel | Not met | Small volunteer team; open item C-2: record the training in Rust, the flight-software standards and the tools in the SDP |
| 020 | Classify each system and subsystem containing software | Met | SDP §3.1 (2026-10-25) |
| 176 | Maintain records of software classification | Met | SDP §3.1, revision history |
| 121 | Tailoring of requirements recorded | Met | This matrix, SDP §3.3, `docs/project/CHANGE_REQUEST_LOG.md` (CR-2026-003, CR-2026-004) |
| 125 | Compliance matrix maintained | Met | This document |
| 139 | Comply with the requirements of NPR 7150.2 applicable to the class | Partial | This matrix: the Not met rows are its open items |
| 024 | Track actual results against plans | Partial | CI history and the CDR reviews; no planned/actual tracking |
| 034 | Acceptance criteria for the software | Partial | SRS verification methods (`docs/software/SOFTWARE_REQUIREMENTS.md`, every row), the SIL pass criteria (`docs/test/SIL_MISSION_SIMULATION.md`); formal acceptance review at TRR |
| 036 | Software processes established | Met | SDP §4 to §8, CI (`.github/workflows/ci.yml`) |
| 037 | Software milestones defined | Met | SDP §4.1 |
| 039 | Acquirer insight into supplier software | N/A | No software supplier; the bought units' firmware (AntS, GNSS receiver) is treated as hardware with an interface, verified at the interface (`docs/interfaces/ICD_OBC_ADCS.md` §12.4). The EPS controller firmware is not bought: the project writes it (SDP §3.4, §2.4 below) |
| 040 | Access to software products | Met | Public repository |
| 042 | Electronic access to source code | Met | Public repository |
| 022 | Software assurance per NASA-STD-8739.8 | Partial | `docs/safety/SOFTWARE_SAFETY_PLAN.md`, `docs/project/MISSION_ASSURANCE_PLAN.md`; no named software assurance lead (open item C-3) |
| 141 | IV&V for projects meeting the criteria | N/A | Not a NASA project; no IV&V designation (SDP §3.3) |
| 131 | IV&V project execution plan | N/A | As SWE-141 |
| 205 | Determine the safety-critical software | Met | SDP §3.2 (2026-10-25). CDR finding FSW-05 closed 2026-10-27: the transponder gates (`sqso-fsw/src/transponder.rs`) and the operations record (`sqso-fsw/src/ops_state.rs`, `sqso-ffi/src/ops_state.rs`) are on the list and in the coverage gate (SRS-Q013; PI concurrence to record). The board firmware items are still to be determined (open item C-8) |
| 023 | Safety-critical software requirements implemented | Met | SRS §3 safety rows; `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md` controls |
| 134 | Safety-critical software: initialisation, inhibits, safe state, prerequisite checks, command validation | Met | Inhibits: `rf_control`, `rf_silence`, `downlink`, `deployment`; command authentication `cmd_auth`; safe-mode entry `state_machine`; tested by the C suites |
| 219 | 100 % MC/DC coverage of safety-critical components | Tailored | 100 % line and condition coverage per file, CI gate `software/flight/scripts/safety_coverage.py` (CR-2026-003, approved 2026-10-25). Measured 2026-10-27: 23 files, 0 problems |
| 220 | Cyclomatic complexity ≤ 15 for safety-critical components | Not met | No complexity measurement of the Rust code. The 60-line function limit (`software/rust/clippy.toml`) bounds it indirectly. Open item C-4 (RID FSW-06): measure with a tool on the safety-critical files and gate it, or tailor by CR |
| 146 | Auto-generated code: define the approach | Met | The command dictionary is generated (`scripts/gen_command_dictionary.py`, CI `--check`); the generator is under CM |
| 206 | Auto-generated code: verification | Met | Generated files are compiled and tested like the rest; CI fails if they differ from the generator's output |
| 032 | Process capability (CMMI) | N/A for Class C as read (C-1) | Not appraised |
| 147 | Reusable software: specify reuse requirements | Met | `sqso-protocol` is shared by the flight, payload and ground software (`software/rust/README.md`) |
| 148 | Contribute reusable software to the NASA catalogue | N/A | Not a NASA project |
| 156 | Cybersecurity: identify risks | Partial | Telecommand authentication (HMAC, replay counter: `software/flight/include/cmd_auth.h`), 47 CFR 97.211; no software cybersecurity risk assessment document |
| 154 | Cybersecurity: identify mitigations in requirements | Partial | SRS authentication rows; dependency audit (`cargo audit` in CI) |
| 157 | Cybersecurity: protect against unauthorised access | Partial | Authenticated telecommand only; no other uplink path. The debug port's protection at flight (STM32 readout protection level) is not specified: open item with the release procedure |
| 159 | Cybersecurity: test the mitigations | Met | C suites `test_cmd_auth`, `test_cmd_handler` (forged, replayed and relabelled frames); fuzz targets (`software/rust/fuzz`) |
| 207 | Secure coding practices | Met | Rust with `unsafe` forbidden in `sqso-protocol` and `sqso-fsw`, workspace lints deny `unwrap` and panicking indexing; `docs/development/CODING_STANDARDS.md` |
| 185 | Static analysis for security weaknesses | Partial | clippy with `-D warnings`, `cargo audit`; no CWE-oriented analyser |
| 210 | Detect and report security incidents | Partial | Authentication failures counted and logged (fault log, housekeeping); no ground procedure |
| 052 | Bidirectional traceability | Met | `scripts/check_traceability.py --check` (CI), `docs/verification/RTM.csv` (generated, `scripts/gen_rtm.py --check`) |

### 2.2 Software Engineering Life Cycle (Chapter 4)

| SWE | Requirement (short) | Status | Evidence / tailoring |
|-----|---------------------|--------|----------------------|
| 050 | Software requirements from system requirements and operations | Met | `docs/software/SOFTWARE_REQUIREMENTS.md` v1.6 (flow-down in the RTM) |
| 051 | Requirements analysis | Met | SRS; CDR reviews |
| 184 | Safety-related requirements in the software requirements | Met | SRS §3.9 (SRS-F080 to SRS-F089), inhibit rows |
| 053 | Manage requirement changes | Met | Revision history of the SRS, `docs/project/CHANGE_REQUEST_LOG.md` |
| 054 | Correct inconsistencies between requirements and products | Partial | Traceability check; the open items of SRS Appendix A |
| 057 | Software architecture | Met | `docs/software/SOFTWARE_DESIGN.md`, `software/rust/README.md` |
| 143 | Architecture review | Met | CDR |
| 058 | Detailed design | Partial | Rust module documentation (rustdoc) and the C headers are the detailed design; the OBC selector is designed (`docs/payload/OBC_SELECTOR_DESIGN.md`) but not implemented; the sensor and actuator drivers are not designed (RID AV-03) |
| 060 | Implement the design | Partial | As SWE-058 |
| 061 | Coding standards | Met | `docs/development/CODING_STANDARDS.md`, rustfmt and clippy gates |
| 135 | Static analysis | Met | clippy `-D warnings` on host and target, the stack-usage analysis of the image (`scripts/stack_usage.py`), the no-dynamic-memory check |
| 062 | Unit test | Met | Rust unit tests (`cargo test --workspace`: 483 at 2026-10-27), C suites (51) |
| 186 | Unit tests repeatable | Met | CI on every change |
| 063 | Software version description | Met | `docs/software/SOFTWARE_VERSION_DESCRIPTION.md` |
| 136 | Software tools: validate and accredit | Tailored | Pinned toolchain, not qualified (CR-2026-004) |
| 065 | Test plans, procedures, reports | Partial | `docs/software/SOFTWARE_TEST_PLAN.md`, `software/flight/tests/hil/HIL_TEST_PLAN.md`; reports are CI logs; no hardware test has run |
| 066 | Perform testing | Partial | Host tests and SIL; target tests not yet (the firmware has not run on hardware) |
| 187 | Place software items under CM before test | Met | Git; CI tests the committed tree |
| 068 | Evaluate test results | Met | CI gates; SIL anomaly checks |
| 070 | Verify and validate models and simulations used | Partial | SIL models documented with sources (`docs/test/SIL_MISSION_SIMULATION.md`); validation against hardware open (EM) |
| 071 | Update test plans with changes | Met | Test plan revisions; tests change with the code |
| 073 | Test on the target or a high-fidelity platform | Not met | Planned HIL (`software/flight/tests/hil/HIL_TEST_PLAN.md`); no target hardware yet |
| 189 | Code coverage measured | Met | CI: Rust lines ≥ 90 % (94.2 %), branches ≥ 80 % (80.9 %), C + Rust lines ≥ 80 % |
| 190 | Verify code coverage, explain uncovered code | Partial | Safety-critical files: every uncovered line marked and justified; other files: not explained line by line |
| 191 | Regression testing | Met | Every change runs the full suites |
| 192 | Software hazard controls verified | Partial | Controls tested in the C suites and the SIL; hardware inhibit tests open |
| 193 | Acceptance testing of delivered software | Not met | At TRR |
| 211 | Test levels defined | Met | Unit, integration (C suites), SIL, HIL (planned) in the test plan |
| 075 | Operations, maintenance and retirement plans | Partial | `docs/software/SOFTWARE_MAINTENANCE_PLAN.md`, `docs/software/ROLLBACK_RECOVERY_PROCEDURES.md`; no in-flight software upload (no bootloader update path) |
| 077 | Deliver software products | Partial | At release (`docs/development/RELEASE_PROCEDURE.md`) |
| 194 | Verify delivery requirements | Not met | At release |
| 195 | Maintain software after delivery | Partial | Maintenance plan; no upload capability in flight |
| 196 | Retire software | Partial | End-of-life passivation is a hardware latch fired by authenticated commands (`hardware/eps/BATTERY_DESIGN.md`); the flight software has no passivation command yet, and the contact-timeout inhibit stops transmissions (`software/flight/include/contact_monitor.h`) |

### 2.3 Supporting Software Life Cycle (Chapter 5)

| SWE | Requirement (short) | Status | Evidence / tailoring |
|-----|---------------------|--------|----------------------|
| 079 | Configuration management plan | Met | `docs/project/CONFIGURATION_MANAGEMENT_PLAN.md` |
| 080 | Track and evaluate changes | Met | Git history, pull requests, change requests |
| 081 | Identify configuration items | Met | `docs/software/CONFIGURATION_INDEX.md` |
| 082 | Authorise changes | Partial | Pull-request review; no formal board for software |
| 083 | Status accounting | Met | Git, `docs/software/SOFTWARE_VERSION_DESCRIPTION.md` |
| 084 | Configuration audits | Partial | `docs/verification/CONFIGURATION_AUDIT_PROCEDURES.md`; not yet performed |
| 085 | Release management | Met | `docs/development/RELEASE_PROCEDURE.md` |
| 045 | Participate in audits | Partial | CDR |
| 086 | Software risk management | Partial | `mission/RISK_REGISTER.md`; software risks recorded there, no separate list |
| 087 | Peer reviews and inspections of requirements, plans, design, code, tests | Tailored | Reviewed pull requests and CI gates (SDP §3.3); `docs/development/CODE_REVIEW_CHECKLIST.md` |
| 088 | Peer review: checklists, entry and exit criteria | Partial | Checklist exists; no recorded entry and exit criteria |
| 089 | Peer review results recorded | Partial | Pull-request history; findings in `docs/reviews/` |
| 090 | Software measurement objectives | Partial | Coverage, stack, image size, test counts are measured and gated; no measurement plan |
| 093 | Analyse measurement data | Partial | CI ratchets (`software/flight/scripts/ratchet.sh`) |
| 094 | Report measurement data | Partial | CI artifacts, CDR reports |
| 199 | Performance measures for software assurance | Not met | No software assurance lead (C-3) |
| 200 | Software requirements volatility measured | Not met | Open item C-5: count SRS changes per release from the revision history |
| 201 | Track software non-conformances | Met | Findings registers, issues, RIDs |
| 202 | Software severity levels for non-conformances | Partial | RID severities 1–3 at reviews; no defect classification in CI |
| 203 | Mandatory assessment of non-conformances | Partial | At reviews |
| 204 | Process assessments | Not met | Open item with C-3 |

### 2.4 Board Firmware Items (Version B)

The RF board TNC firmware (STM32L431, SQSO-RF-BRD-001 §13) and the EPS controller firmware (MSP430FR2355,
SQSO-EPS-BRD-001 §12), SDP §3.4. Neither is written. The rows of §2.1 to §2.3 apply to them as to the OBC software; the
ones whose status differs:

| SWE | Requirement (short) | RF board TNC firmware | EPS controller firmware |
|-----|---------------------|-----------------------|-------------------------|
| 020 | Classification | Met: Class C (SDP §3.1) | Met: Class C (SDP §3.1) |
| 050 | Software requirements | Not met: the list to write is in SDP §3.4 (open item C-7) | Not met: as for the TNC (C-7) |
| 052 | Bidirectional traceability | Not met: with the requirements (C-7) | Not met (C-7) |
| 057 | Software architecture | Partial: the functions and what stays in hardware, SQSO-RF-BRD-001 §8 and §13 | Partial: SQSO-EPS-BRD-001 §12 and the ICD register block |
| 060 | Implement the design | Not met (fabrication phase) | Not met (fabrication phase) |
| 061 | Coding standards | Partial: `docs/development/CODING_STANDARDS.md` applies if it is Rust (proposed, reusing `sqso-protocol`) | Not met: language not chosen; the standard for it to be named |
| 066 | Software testing | Not met: host unit tests, VI-RFB-01 and VI-RFB-05, an end-to-end EM test with the OBC (SDP §3.4) | Not met: host unit tests, V-EPS-6, V-EPS-8, V-EPS-9, an interface test with the OBC (SDP §3.4) |
| 187 | Configuration management of the software items | Not met: an item of its own in the configuration index when created | Not met: as for the TNC |
| 205 | Safety-critical determination | Not met: from the hazard analysis (C-8) | Not met: from the hazard analysis (C-8) |

## 3. SRS-C Requirements: Verification Methods

The design constraints of SRS §8.1 have verification methods since SRS v1.6 (T test, A analysis, I inspection):

| Verification | Requirement | Status 2026-10-27 |
|--------------|-------------|-------------------|
| I: build configuration, CI toolchain pin | SRS-C001 | Met |
| T: ctest `no_dynamic_memory`, CI symbol check | SRS-C002 | Met |
| A: `scripts/stack_usage.py` on the image (CI gate); T: HIL-044 | SRS-C003 | Met by analysis: 6,004 B of 8,192 B (73.3 %, the CDR round-2 image with the NMI handler); hardware pending |
| I: image size at each release | SRS-C004 | Met at the last measurement (115,876 B, 2026-10-25); not a CI gate |
| as in the SRS table | SRS-C005 to SRS-C009 | see SRS §8.1 |
| I and T: beacon content filter against 47 CFR 97 | SRS-C010 | Met |
| T: `test_rf_silence`, `test_downlink`, SIL `rf-silence` | SRS-C011 | Met (45.0 min in the SIL commissioning run) |
| T: `test_beacon_format`, `test_downlink`; I | SRS-C012 | Met |

## 4. Open Items

| ID | Item | Closure |
|----|------|---------|
| C-1 | Confirm every row's Class C applicability against the signed Appendix C | Software assurance lead, before TRR |
| C-2 | Training record (SWE-017) | SDP update, fabrication phase |
| C-3 | Name a software assurance lead; SA performance measures and process assessment (SWE-022, -199, -204) | PI |
| C-4 | Cyclomatic complexity of the safety-critical Rust files (SWE-220), RID FSW-06 | Measure and gate, or a CR to tailor |
| C-5 | Requirements volatility (SWE-200) | From the SRS revision history at each release |
| C-6 | Target-platform testing (SWE-073, -066) | EM and HIL plan |
| C-7 | Requirements, traceability and tests of the RF board TNC firmware and the EPS controller firmware (SWE-050, -052, -066) | Software lead, before the EM integration |
| C-8 | Safety-critical determination of the two board firmware items (SWE-205) | Hazard analysis update |

## Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | SMART-QSO Team (CDR avionics and flight-software review) | Initial: Appendix C mapping for Class C, SRS-C verification status, open items C-1 to C-6 (programmatics finding SE-10) |
| B | 2026-10-27 | SMART-QSO Team (CDR flight software, round 2) | The RF board TNC firmware and the EPS controller firmware added as Class C items (§2.4, open items C-7 and C-8); SWE-039 and SWE-205 rows updated (FSW-05 closed: the transponder and the operations record are safety-critical); SRS-C003 at 6,004 B |
