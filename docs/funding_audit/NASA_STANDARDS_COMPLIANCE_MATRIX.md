# NASA Standards and Regulatory Compliance Matrix
## SMART-QSO 1U CubeSat (proposed Class D, NPR 8705.4B)

**Document Number:** SQSO-AUD-003
**Revision:** C
**Date:** 2026-10-25

---

## Status Definitions

No standard is rated "Compliant". Compliance requires a signed compliance matrix or verification evidence, and none
exists yet.

| Status | Meaning |
|--------|---------|
| **Gap** | Applicable; not addressed, or addressed incorrectly |
| **Partial** | Applicable; some artifacts exist but evidence is incomplete or contradicted |
| **Tailored-plan** | Applicable; the audit has defined a tailored approach, but there is no verification evidence yet |
| **Conditional** | Applies only on a particular launch path or funding path |
| **N/A** | Not applicable (rationale given) |

Current revisions were verified on 2026-10-14 from the sources cited. The README previously claimed "Compliant" for
NPR 7150.2, NPR 7123.1 and NASA-STD-8719.13 with no supporting matrices (finding VV-11).

## 1. NASA Program, Engineering and Assurance Directives

| Standard | Current revision (source) | Applicability | Repo citation (pre-audit) | Status | Required action |
|----------|---------------------------|---------------|---------------------------|--------|-----------------|
| NPR 8705.4 Risk Classification for NASA Payloads | **8705.4B**, effective 2024-11-01 ([NASA SMA](https://sma.nasa.gov/news/articles/newsitem/2024/12/02/npr-8705-4b-risk-classification-for-nasa-payloads-formally-released-agencywide)) | Source of the Class D designation (if NASA-sponsored; otherwise used as a reference framework) | Class D attributed to "NPR 7120.5 Appendix G" (wrong source) | Partial | Cite 8705.4B for the Class D designation |
| NPR 7120.5 Space Flight Program and Project Management | 7120.5F, effective 2021-08-03 (listed expiry 2026-08-03; extension status unverified) | Large NASA flight projects; heavy for a non-NASA 1U | Called "governing" (54 cites) | Partial | Use as reference only; adopt tailored lifecycle reviews |
| NPR 7120.8 NASA Research and Technology Program and Project Management | 7120.8A | Common management framework for PI-led small satellites and technology demonstrations | Never cited | Gap | Consider as the management baseline if NASA-sponsored |
| NPR 7123.1 Systems Engineering Processes and Requirements | **7123.1D** (2023–2028); no "Rev E" found | Applicable, tailored | Cited as "Rev E"; "Compliant" | Partial | Correct the revision; tailoring matrix |
| NPR 7150.2 Software Engineering Requirements | **7150.2D**, effective 2022-03-08 | Applicable | Cited as "Rev B"; "Compliant"; class stated as B, C, C/D and D in different places | Partial | Classification done (2026-10-25): Class C for flight software and ground software used in operations, Class D for research and analysis tools (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1). Still to do: the Appendix C requirements mapping and compliance matrix for Class C |
| NASA-STD-8739.8 Software Assurance and Software Safety | **8739.8B**, 2022-09-08 | Applicable (it replaced 8719.13) | Listed, not implemented | Gap | Software assurance plan; safety-criticality assessment |
| NASA-STD-8719.13 Software Safety | **Cancelled 2020-06-10** (folded into 8739.8) | N/A | 27 citations; "Compliant" | Gap (stale citation) | Replace every citation with 8739.8B |
| NPR 8715.3 NASA General Safety Program | 8715.3E | Reference | Listed | Partial | — |
| NPR 8715.6 Orbital Debris | Required by CDS Rev 14.1 §2.4.3 | Applicable | Never cited | Tailored-plan | ODAR per 8719.14C ([ODAR](../regulatory/filings/ODAR_SMART-QSO.md)) |
| NASA-STD-8719.14 Process for Limiting Orbital Debris | **8719.14C**, 2021-11-05; DAS 3.2.6 (2024-02-05) | Applicable | Cited as "A"; 25-year rule; "Compliant" | Tailored-plan | Run DAS 3.2.6; finalize ODAR |

### 1.1 NPR 7150.2: Software Implementation Items (update 2026-10-24)

The NPR 7150.2 row above was **Gap** until 2026-10-25, when the software was classified against Appendix D (Class C;
`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1). It is now **Partial**: the Appendix C requirements mapping and
compliance matrix for Class C does not exist yet. This table records the state of the software items that a tailored matrix would cover, after the flight software was ported
from C to Rust on 2026-10-24 (`software/rust/README.md`). Each status uses the definitions above.

| Item | State on 2026-10-24 | Status |
|------|---------------------|--------|
| Implementation language | Rust (2024 edition) for all flight code. The only C is the flight-log printf shim (`software/flight/src/flight_log_fmt.c`) and the test suites. SRS-C001 still says C11, so the implementation does not meet it as written; amending it or recording a waiver is a PI decision (`docs/software/SOFTWARE_REQUIREMENTS.md` §8.1) | Partial |
| Coding standard | `docs/development/CODING_STANDARDS.md` Rev B: §11 for Rust (lints denied in CI, `unsafe` policy, no allocation, no panics on input, C ABI rules) and §3–§10 for the remaining C. The JPL Power of Ten mapping, with what is enforced by a tool and what by review, is `software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md` §2.1. MISRA C applies to the shim only (42-finding baseline, `software/flight/docs/MISRA_DEVIATIONS.md`) | Partial |
| Static analysis | In CI on every change: clippy with warnings as errors (host and Cortex-M4F), the `#![forbid(unsafe_code)]` check, CodeQL for Rust and C/C++, `cargo audit`; cppcheck, clang-tidy (baseline 0) and the MISRA addon (baseline 42) on the remaining C. None of the tools is qualified | Partial |
| Code coverage | Line coverage gated in CI: the Rust flight crates ≥ 90 % (92.2 % measured) and the flight software C and Rust together ≥ 80 % (92.2 %). Branch and MC/DC coverage are not measured | Partial |
| Tools and toolchain | Rust 1.97.0 pinned (`software/rust/rust-toolchain.toml`), CI tool versions pinned by the runner image; no tool qualification. PI decision 2026-10-25 (CR-2026-004): no qualified compiler for this Class C software; the tests, image checks, static stack analysis and hardware tests check the compiler's output | Partial |
| Target verification | The STM32L4R5ZI image (`software/rust/firmware`) is built and checked in CI. No hardware has run it | Gap |

## 2. Workmanship, Parts, Materials, Structures, Environments

| Standard | Current revision | Applicability | Repo citation | Status | Required action |
|----------|------------------|---------------|---------------|--------|-----------------|
| GSFC-STD-7000 GEVS | **7000B**, 2021-04-28 | Fallback test levels until provider levels are known (CDS §3) | Cited as "A"; protoflight defined backwards; workmanship level mislabeled as protoflight | Gap | Re-issue the environmental test plan per provider levels, GEVS-B envelope |
| SpaceX Rideshare Payload User's Guide | **v11** (Jul/Aug 2026) | Governs if on Transporter: random vibe ≥ 5.57 Grms MPE, inhibit verification during vibe, GNSS ephemeris duty | Not cited | Gap | Adopt when the integrator is selected |
| IPC J-STD-001 (Space Addendum) / IPC-A-610 | Current per IPC | Applicable (workmanship) | Listed | Partial | Name revision and class in the workmanship plan |
| NASA-STD-8739.3 Soldering | **Cancelled 2011-10-17** (replaced by J-STD-001ES and 8739.6) | N/A | 14 citations as governing | Gap (stale citation) | Replace citations |
| NASA-STD-8739.1 Polymerics / 8739.4 Crimping & Harness | 8739.1B Ch 2; 8739.4A Ch 3 | Applicable | Listed | Partial | — |
| NASA-STD-5001 Structural Design | Current per NASA | Applicable (factors of safety) | Listed once | Gap | FEM report with factors and margins |
| ASTM E595 outgassing (CDS §2.1.7) | — | Applicable | "Compliant" without data | Gap | Materials list with E595 sources; bakeout |
| NASA-STD-6016 Materials | — | Required by LSP-REQ-317.01 (CSLI) and ISS | Not cited | Conditional | If CSLI/ISS |
| MIL-STD-1540 "E" / SMC-S-016 | "1540E" is unofficial; SMC-S-016 (2014) → SSC-S-016 | Informative | Cited as "MIL-STD-1540E" | Partial | Cite SSC-S-016 or remove |
| MIL-STD-461 EMC | G (2015) | Tailored use acceptable | Listed | Partial | Tailor to provider EMC requirements |
| EEE-INST-002 | — | Acceptable for Class D parts selection | Cited | Partial | Parts list with screening level |

## 3. CubeSat, Launch-Path and Safety Documents

| Document | Current revision | Applicability | Repo citation | Status | Required action |
|----------|------------------|---------------|---------------|--------|-----------------|
| Cal Poly CubeSat Design Specification | **Rev 14.1** (Feb 2022) | Applicable | Rev 14.1 cited, but misstated (30/45-min swap, 2 inhibits, edge radius, 1.33 kg limit from Rev 13) | Partial | Clause-by-clause rebuild (VV-14); timers and inhibits now in SYS-224/225 |
| LSP-REQ-317.01 Program-Level Dispenser and CubeSat Requirements | **Rev B** (2014) | Only on the NASA CSLI/ELaNa path | Cited as "LSP-REQ-317" | Conditional | Adopt if CSLI |
| Voyager (Nanoracks) NRCSD IDD | **Rev A** (2024-01-19) | Only on the ISS backup path | Narrative citing an older IDD | Conditional | Re-map if ISS |
| JSC-20793 Crewed Space Vehicle Battery Safety | **Rev D** | ISS path (and good practice) | Never cited | Conditional (Gap if ISS) | Battery test plan |
| Nanoracks NR-SRD-139 Li-ion Battery Test Requirements | Current revision unverified | ISS/NRCSD path | Never cited | Conditional | — |
| AFSPCMAN 91-710 / SSCMAN 91-710 Range Safety | Per range | Hazardous materials and pyro (CDS §2.1.2, §2.1.6); SpaceX safety letter | Not addressed | Gap | Hazard declaration via integrator |

## 4. U.S. Regulatory Requirements

| Regulation | Requirement | Status | Artifact |
|------------|-------------|--------|----------|
| 47 CFR 97.207(b) | Cease transmissions by telecommand | **Gap** (not implemented: SW-02) | BASELINE §7 |
| 47 CFR 97.207(g)(1) | Pre-space notification ≤ 30 days after LV determination, ≥ 90 days before integration; Appendix 4 / Res. 642; debris disclosures | Tailored-plan (drafted, not filed) | [Pre-space draft](../regulatory/filings/FCC_97-207_PRE_SPACE_NOTIFICATION.md) |
| 47 CFR 97.207(g)(1)(vii)(D) | Re-entry ≤ 5 yr after mission; P ≥ 0.9; casualty < 1:10,000 | Tailored-plan (DAS pending) | [ODAR](../regulatory/filings/ODAR_SMART-QSO.md) |
| 47 CFR 97.207(g)(2)/(3) | In-space ≤ 7 days; post-space ≤ 3 months | Tailored-plan | [Templates](../regulatory/filings/FCC_97-207_IN_SPACE_AND_POST_SPACE_NOTIFICATIONS.md) |
| 47 CFR 97.211 | Telecommand by designated stations; codes may obscure telecommand | **Gap** (static key; SW-03) | BASELINE §7 |
| 47 CFR 97.113 | No pecuniary interest; no obscured messages; no obscene content | Partial (funding conditions defined; AI text filter missing: SW-12) | [Pathway](../regulatory/LICENSING_PATHWAY_DETERMINATION.md) |
| 47 CFR 97.307(e) | Spurious ≤ 25 µW and ≥ 40 dB below the fundamental (30–225 MHz, ≤ 25 W) | **Gap** (TP-RF-003 pass criteria too weak: VV-09) | BASELINE §7 |
| IARU coordination (form v40) | Mandatory for amateur frequencies (CDS §2.4.1.1) | Tailored-plan (drafted, not submitted) | [IARU draft](../regulatory/filings/IARU_COORDINATION_REQUEST_v40.md) |
| 15 CFR 960 (NOAA) | Remote-sensing license if imaging | N/A (no imager); confirmation recommended | [NOAA determination](../regulatory/filings/NOAA_REMOTE_SENSING_DETERMINATION.md) |
| 15 CFR 774 (EAR) / ITAR | Spacecraft 9A515.a.5; integration services ITAR | Partial (self-classified) | [Export memo](../regulatory/filings/EXPORT_CLASSIFICATION_MEMO.md) |
| 14 CFR 450.43 (FAA payload review) | Payload has all required licenses; information via launch operator | Tailored-plan | [Deliverables](../regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-14 | Initial matrix (funding audit) |
| B | 2026-10-24 | §1.1: NPR 7150.2 implementation items (language, coding standard, static analysis, coverage, tools, target verification) after the port of the flight software to Rust. No status in §1 changed |
| C | 2026-10-25 | NPR 7150.2 row: Gap to Partial. The software is classified (Class C; SDP §3.1); the Appendix C compliance matrix for Class C is still to do. §1.1 note and tools row to match |
