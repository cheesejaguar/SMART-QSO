# SMART-QSO Anticipated Lessons (Lessons-Learned Register)

**Document ID**: SQSO-LL-001
**Version**: 1.1
**Date**: 2026-10-16
**Classification**: Unclassified
**Status**: Living document. **Anticipated lessons only:** no development phase has been completed

> **Funding audit (2026-10-16, PI-16).** Version 1.0 reported lessons from requirements, design, implementation and
> test phases as if they had happened, with figures that were not true (for example "478+ tests", "zero MISRA
> violations" and a flatsat campaign). The project is in Phase B, before PDR (`docs/BASELINE_PARAMETERS.md` §1); no
> flatsat, hardware or environmental test has been run. Every entry below is now an **anticipated lesson**: a risk we
> expect, why we expect it, and what we plan to do. When something actually happens, record it with the form in §5 and
> cite the evidence.

## 1. Purpose

This document lists the lessons we expect to learn while developing SMART-QSO, so that they can be acted on before they
become real, and gives the process for recording actual lessons later. It follows the intent of NPR 7120.6 (lessons
learned) without claiming compliance with it.

## 2. Lessons Learned Process

### 2.1 Collection Process

```
LESSONS LEARNED LIFECYCLE

1. IDENTIFY
   - Team member identifies a lesson (anticipated or actual)
   - Documents it in the submission form (§5)

2. REVIEW
   - Project management reviews
   - Confirms whether it is anticipated or actual; an actual lesson needs evidence
   - Assigns category and priority

3. DOCUMENT
   - Added to this document
   - Cross-referenced to phase
   - Root cause identified (actual lessons only)

4. SHARE
   - Distributed to the team
   - Published in this repository
   - Briefed at reviews

5. APPLY
   - Incorporated into processes and procedures
```

### 2.2 Categories

| Category | Description |
|----------|-------------|
| Technical | Hardware, software, design |
| Programmatic | Schedule, cost, management |
| Process | Procedures, reviews, testing |
| Team | Communication, training, resources |

### 2.3 Priority Levels

| Priority | Definition |
|----------|------------|
| High | Could significantly affect the mission or schedule |
| Medium | Could cause rework or inefficiency |
| Low | Minor improvement opportunity |

## 3. Anticipated Lessons by Phase

Each entry gives the expected risk, the possible impact, why we expect it, the recommendation, and the planned
mitigation with its current state. None of these has happened on this project.

### 3.1 Requirements Phase

#### LL-001: Early AI Feasibility Assessment
**Category**: Technical | **Priority**: High | **Type**: Anticipated

**Expected risk**: AI payload requirements that are too ambitious for the power available on a 1U CubeSat.

**Possible impact**: Design iterations late in the program to bring AI capability within the power budget.

**Why we expect it**: Energy per beacon is easy to underestimate. An early estimate in this repository counted tokens ×
joules per token and left out Jetson boot, model load and shutdown, which dominate a cold start
(`docs/payload/AI_MODEL_SELECTION.md`; decision D5).

**Recommendation**: Analyse the power budget, including boot and shutdown energy, before fixing AI requirements. Put
duty-cycle limits in the first trade studies.

**Planned mitigation**: Flight baseline is a selector on the OBC; Jetson generation is a gated experiment, enabled only if
the measured energy per accepted beacon is ≤ 0.1 Wh. Jetson duty ≤ 5 % tumbling and ≤ 10 % sun-pointed
(BASELINE §5, §6). The energy measurement on the engineering model has not been made.

---

#### LL-002: Amateur Radio Regulatory Clarity
**Category**: Process | **Priority**: Medium | **Type**: Anticipated

**Expected risk**: Part 97 constraints (open protocols, no encrypted content, telecommand only from designated stations,
cessation of emissions) found after requirements are fixed.

**Possible impact**: Late requirement changes and rework of the command and beacon design.

**Why we expect it**: Regulatory review often starts after the technical design.

**Recommendation**: Bring the regulatory constraints into the ConOps and requirements from the start.

**Planned mitigation**: Licensing pathway and baseline decisions D1–D3 are recorded
(`docs/regulatory/LICENSING_PATHWAY_DETERMINATION.md`; BASELINE §9). FCC consultation is still open (D3).

---

### 3.2 Design Phase

#### LL-003: Graceful Degradation Architecture
**Category**: Technical | **Priority**: High | **Type**: Anticipated

**Expected risk**: Fault handling becomes complex if primary functions depend on the experimental AI payload.

**Possible impact**: A payload fault that takes down the beacon or transponder.

**Why we expect it**: Coupling between an experimental payload and core functions is a common source of mission loss.

**Recommendation**: Design primary functions to work without optional capabilities. Never make mission success depend on
an experiment.

**Planned mitigation**: Flight software is designed to run without the AI payload, with template beacons as the fallback
(BASELINE §2). Exercised so far only in host simulation and unit tests.

---

#### LL-004: State Machine Simplicity
**Category**: Technical | **Priority**: Medium | **Type**: Anticipated

**Expected risk**: A state machine that grows states for elegance rather than need, which multiplies test cases.

**Possible impact**: More testing and harder verification.

**Why we expect it**: Extra states (commissioning, eclipse, science) are often added before they are needed.

**Recommendation**: Start with the fewest states; add one only when operations need it.

**Planned mitigation**: The design uses SAFE, IDLE and ACTIVE, plus an EMERGENCY state that sheds the beacon at very low
charge (BASELINE §5). Whether these cover every operational scenario will be known only after operations rehearsals.

---

#### LL-005: Hardware Abstraction Layer
**Category**: Technical | **Priority**: Medium | **Type**: Anticipated

**Expected risk**: Software development blocked waiting for target hardware.

**Possible impact**: Schedule slip; little testing before hardware arrives.

**Why we expect it**: Without a hardware abstraction layer, most flight code cannot run off-target.

**Recommendation**: Define HAL interfaces early and stub every hardware dependency, so unit tests run without hardware.

**Planned mitigation**: A HAL with a simulation back end exists; 31 CMocka suites (428 cases) run on the host without
hardware (`docs/reviews/FLIGHT_SOFTWARE_DELIVERY.md` §4.2). Several hardware drivers are still stubs
(`docs/reviews/KNOWN_ISSUES.md` KI-001).

---

### 3.3 Implementation Phase

#### LL-006: Test-Driven Development Value
**Category**: Process | **Priority**: High | **Type**: Anticipated

**Expected risk**: Design faults found late because tests are written after the code, or test figures that overstate
what is tested.

**Possible impact**: Integration problems; false confidence in software quality.

**Why we expect it**: Coverage tools can leave out source files no test runs, which inflates the figure
(`.github/workflows/ci.yml`, coverage-gate comment).

**Recommendation**: Unit tests for every safety-related function, run in CI; measure coverage over every compiled source.

**Planned mitigation**: CI runs the unit tests and fails below 65 % line coverage. Current line coverage is 68.2 %
(ci.yml). The long-term target of > 80 % has not been reached.

---

#### LL-007: MISRA C Compliance Effort
**Category**: Technical | **Priority**: Medium | **Type**: Anticipated

**Expected risk**: Coding-standard findings accumulate and are fixed late, in batches.

**Possible impact**: Rework close to delivery; subtle defects left in place.

**Why we expect it**: Retrofitting a coding standard is slower than applying it from the start.

**Recommendation**: Adopt the coding standard from day one; run static analysis in CI; fix findings as they appear.

**Planned mitigation**: CI blocks new findings. The current baselines are 1,189 MISRA C:2012 findings (cppcheck addon, not
a qualified MISRA tool) and 58 clang-tidy findings (`software/flight/quality/`). The code is not MISRA-compliant.

---

#### LL-008: Documentation as Code
**Category**: Process | **Priority**: Low | **Type**: Anticipated

**Expected risk**: Documents drift from the code and from each other.

**Possible impact**: Reviewers and funders read figures that are no longer true.

**Why we expect it**: Hand-typed counts and status claims go stale.

**Recommendation**: Keep documents in version control, review them with code changes, and generate figures from CI
instead of typing them.

**Planned mitigation**: Documents are in Git; CI checks document references, key parameters and traceability
(`scripts/check_doc_references.py`, `scripts/check_parameters.py`, `scripts/check_traceability.py`).

---

### 3.4 Testing Phase

#### LL-009: Flatsat Test Procedure Detail
**Category**: Process | **Priority**: Medium | **Type**: Anticipated

**Expected risk**: Test procedures that are ambiguous, so results cannot be repeated.

**Possible impact**: Repeated tests; disputes over pass/fail.

**Why we expect it**: Procedures written before anyone has run them often lack expected values and tolerances.

**Recommendation**: Write each procedure as if the operator has no prior knowledge. Include expected values, tolerances
and contingencies.

**Planned mitigation**: Draft procedures exist in `docs/test_procedures/`. None has been executed.

---

#### LL-010: Extended Duration Testing
**Category**: Technical | **Priority**: High | **Type**: Anticipated

**Expected risk**: Memory leaks, timing drift or counter overflows that appear only after days of operation.

**Possible impact**: An on-orbit fault that short tests did not reveal.

**Why we expect it**: Short tests cannot reveal slow resource exhaustion.

**Recommendation**: Include an extended-duration test (≥ 7 days) and monitor memory, timing and cumulative errors.

**Planned mitigation**: An extended-duration test is planned for the flatsat campaign. It has not been run.

---

## 4. Lessons from Other Missions (public sources)

These are events on other missions, reported publicly, that we intend to learn from. They are not SMART-QSO experience.
Earlier versions attributed lessons to specific NASA Ames missions without sources; those attributions are removed.

| Source | What was reported | What we will do |
|--------|-------------------|-----------------|
| AMSAT Fox-1A (AO-85), 1U, launched 2015 ([Wikipedia summary](https://en.wikipedia.org/wiki/Fox-1A)) | From December 2018 the battery voltage fell dangerously low in eclipse, and the transmitters were disabled except for periodic telemetry | Size the eclipse energy margin for an aged battery; keep the 0.1 Wh-per-beacon gate and the duty limits (BASELINE §5) |
| SwissCube, 1U, launched 2009 ("The SwissCube: Results and Lessons Learned After 4 Years of Operations in Space", SmallSat 2013, [abstract](https://digitalcommons.usu.edu/smallsat/2013/all2013/1)) | Results and lessons from a B-dot magnetorquer controller on a 1U (full text not read) | Read the full paper before the ADCS design review |
| ESA Φ-Sat-1, 6U, launched 2020 ([ESA Φ-lab](https://philab.esa.int/flagship-programmes/phi-sats-programme/); [Ubotica](https://www.ubotica.com/news/successful-launch-of-ground-breaking-ai-enabled-earth-observation-satellite)) | A commercial AI chip (Intel Myriad 2) needed about two years of radiation characterization before flight | Plan radiation characterization of the Jetson module; treat it as non-critical and expendable (BASELINE §6) |

## 5. Lessons Submission Form

```
LESSONS LEARNED SUBMISSION

Date: ___________
Submitter: ___________
Type: [ ] Anticipated  [ ] Actual (evidence required)
Phase: [ ] Requirements [ ] Design [ ] Implementation [ ] Test [ ] Ops

CATEGORY: [ ] Technical [ ] Programmatic [ ] Process [ ] Team

PRIORITY: [ ] High [ ] Medium [ ] Low

TITLE: _________________________

OBSERVATION (actual) or EXPECTED RISK (anticipated):
_________________________________
_________________________________

EVIDENCE (actual lessons: test log, commit, report):
_________________________________

IMPACT:
What was (or could be) the impact?
_________________________________

ROOT CAUSE:
Why did this happen (or why do we expect it)?
_________________________________

RECOMMENDATION:
What should be done differently?
_________________________________

ACTION TAKEN:
What action was taken on this project?
_________________________________

APPLICABLE TO:
[ ] This project only
[ ] Similar CubeSat projects
[ ] All spacecraft projects
```

## 6. Lessons Tracking

### 6.1 Lessons Summary

| Phase | High | Medium | Low | Total |
|-------|------|--------|-----|-------|
| Requirements | 1 | 1 | 0 | 2 |
| Design | 1 | 2 | 0 | 3 |
| Implementation | 1 | 1 | 1 | 3 |
| Testing | 1 | 1 | 0 | 2 |
| **Total** | **4** | **5** | **1** | **10** |

All ten are anticipated; no actual lessons have been recorded.

### 6.2 Lessons Status

| ID | Title | Status |
|----|-------|--------|
| LL-001 | Early AI Feasibility | Anticipated; mitigation planned |
| LL-002 | Regulatory Clarity | Anticipated; mitigation planned |
| LL-003 | Graceful Degradation | Anticipated; mitigation planned |
| LL-004 | State Machine Simplicity | Anticipated; mitigation planned |
| LL-005 | Hardware Abstraction | Anticipated; mitigation in progress |
| LL-006 | Test-Driven Development | Anticipated; mitigation in progress |
| LL-007 | MISRA Compliance | Anticipated; mitigation in progress |
| LL-008 | Documentation as Code | Anticipated; mitigation in progress |
| LL-009 | Test Procedure Detail | Anticipated; mitigation planned |
| LL-010 | Extended Duration Test | Anticipated; mitigation planned |

## 7. Sharing

Lessons are published in this repository. Actual lessons that would help other amateur and university CubeSat teams can
be offered to the community (for example AMSAT or the SmallSat conference). The project has no NASA relationship, so the
earlier plan to submit to the NASA Lessons Learned Information System is removed.

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | SMART-QSO Team | Funding audit PI-16: every lesson relabelled as anticipated; statements that phases or tests happened removed; false figures corrected (tests, coverage, MISRA, power margin); NASA Ames attributions and LLIS submission removed; lessons from other missions cited from public sources; summary corrected to 10 lessons |
