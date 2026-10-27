# SMART-QSO Safety Review Package

> **Historical record (2026-01-02).** Superseded for current values by
> [`docs/BASELINE_PARAMETERS.md`](../BASELINE_PARAMETERS.md) and
> [`docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md`](CDR/CDR_2026_DESIGN_REVIEW.md). It is the safety review package
> generated on 2026-01-02; it closes no hazard. No independent safety review has been held: the project is in Phase B
> (pre-PDR), and the independent PDR (2027-01-12) and CDR (2027-09-28) are planned in `mission/SCHEDULE.md` (BASELINE
> §1). The record is kept as written; its values, statuses, approvals and the dated "superseded" notes in its body are
> those of their dates and are not to be cited. Still useful for: the first hazard list. The current safety analyses
> are `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md` (SQSO-SDP-002, HR-01 to HR-16, none closed),
> `docs/safety/SAFETY_DATA_PACKAGE.md` (SQSO-SDP-001), `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md` and
> `docs/environment/BATTERY_SAFETY_ASSESSMENT.md`. Index of the review documents: [README.md](README.md).

> **Funding audit notice (2026-10-14): internal self-assessment, not an independent review.**
> This record was generated on 2026-01-02 without named independent reviewers, signatures, or a
> RID/RFA log. Its PASS/APPROVED/COMPLETE statements are **not substantiated** and must not be cited
> to funders or regulators as review outcomes. See `docs/funding_audit/FUNDING_AUDIT_REPORT.md` and
> finding PI-03 in `docs/funding_audit/FINDINGS_REGISTER.md`.

**Document ID**: SQSO-SR-001
**Version**: 1.1
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the Safety Review package for SMART-QSO per NASA-STD-8719.13 and NASA-STD-8719.14. It demonstrates that all safety requirements are identified, analyzed, and mitigated.

## 2. Safety Approach

### 2.1 Safety Philosophy

```
SAFETY HIERARCHY

1. Design for minimum hazard
   └─ Eliminate hazards through design

2. Incorporate safety devices
   └─ Use protective mechanisms

3. Provide warning devices
   └─ Alert to hazardous conditions

4. Develop procedures and training
   └─ Operational safety measures
```

### 2.2 Applicable Standards

| Standard | Title | Applicability |
|----------|-------|---------------|
| NASA-STD-8719.13 | Software Safety | Flight software |
| NASA-STD-8719.14 | Orbital Debris | Mission design |
| NPR 8715.3 | Safety Requirements | Overall |
| Range Safety | Per Launch Provider | Integration |

### 2.3 Safety Classification

| Classification | Criteria | SMART-QSO |
|----------------|----------|-----------|
| Class I | Catastrophic potential | No |
| Class II | Critical potential | No |
| Class III | Marginal potential | Yes |

**Rationale**: CubeSat with no propulsion, pyrotechnics, or pressurized systems. Marginal hazards limited to RF emissions and battery.

## 3. Hazard Analysis Summary

### 3.1 Preliminary Hazard List (PHL)

| ID | Hazard | Cause | Effect | Severity |
|----|--------|-------|--------|----------|
| HZ-001 | Unintended RF emission | Software fault | EMI to launch vehicle | III |
| HZ-002 | Battery thermal runaway | Overcharge/damage | Fire | I |
| HZ-003 | Uncontrolled deployment | Mechanism failure | Collision | III |
| HZ-004 | Orbital debris creation | Collision/breakup | Debris | II |
| HZ-005 | RF interference | Wrong frequency | Regulatory violation | IV |
| HZ-006 | Commanded unsafe state | Command error | Mission impact | IV |

### 3.2 Hazard Severity Definitions

| Severity | Category | Definition |
|----------|----------|------------|
| I | Catastrophic | Loss of life or system |
| II | Critical | Severe injury or major damage |
| III | Marginal | Minor injury or damage |
| IV | Negligible | Less than minor |

### 3.3 Hazard Probability Definitions

| Probability | Category | Definition |
|-------------|----------|------------|
| A | Frequent | Likely to occur often |
| B | Probable | Will occur several times |
| C | Occasional | Likely to occur sometime |
| D | Remote | Unlikely but possible |
| E | Improbable | So unlikely, assume won't occur |

## 4. Hazard Controls

### 4.1 HZ-001: Unintended RF Emission

**Hazard**: RF transmitted during launch vehicle operations

**Controls**:
| Control | Type | Verification |
|---------|------|--------------|
| 30-minute RF inhibit timer | Design | Analysis, test |
| Deployment switch interlock | Design | Test |
| No RF until antenna deploy | Design | Test |
| Ground command required | Procedure | Test |

**Residual Risk**: Remote (D) × Marginal (III) = Acceptable

**Closure Status**: [x] Open [ ] Closed. *Reopened 2026-10-15 (PI-07): no verification test has been run (§8.1); the hazard log is `docs/safety/SAFETY_DATA_PACKAGE.md` §4.*

### 4.2 HZ-002: Battery Thermal Runaway

**Hazard**: Battery failure due to thermal or electrical abuse

**Controls**:
| Control | Type | Verification |
|---------|------|--------------|
| Cell-level protection | Design | Test |
| Software charge control | Design | Analysis |
| Thermal monitoring | Design | Test |
| Undervoltage cutoff | Design | Test |
| Overvoltage protection | Design | Test |

**Residual Risk**: Improbable (E) × Critical (II) = Acceptable

**Closure Status**: [x] Open [ ] Closed. *Reopened 2026-10-15 (PI-07): no verification test has been run (§8.1); the hazard log is `docs/safety/SAFETY_DATA_PACKAGE.md` §4.*

### 4.3 HZ-003: Uncontrolled Deployment

**Hazard**: Antenna deployment during launch vehicle ops

**Controls**:
| Control | Type | Verification |
|---------|------|--------------|
| Deployment switch inhibit | Design | Test |
| 30-minute timer delay | Design | Test |
| Mechanical restraint | Design | Inspection |
| RBF pin backup | Design | Inspection |

**Residual Risk**: Remote (D) × Marginal (III) = Acceptable

**Closure Status**: [x] Open [ ] Closed. *Reopened 2026-10-15 (PI-07): no verification test has been run (§8.1); the hazard log is `docs/safety/SAFETY_DATA_PACKAGE.md` §4.*

### 4.4 HZ-004: Orbital Debris Creation

**Hazard**: Spacecraft breakup creates debris

**Controls**:
| Control | Type | Verification |
|---------|------|--------------|
| Passivation at EOL | Design | Analysis |
| No stored energy at EOL | Design | Analysis |
| 25-year deorbit compliance | Design | Analysis |
| Debris mitigation design | Design | Analysis |

**Residual Risk**: Improbable (E) × Critical (II) = Acceptable

**Closure Status**: [x] Open [ ] Closed. *Reopened 2026-10-15 (PI-07): no verification test has been run (§8.1); the hazard log is `docs/safety/SAFETY_DATA_PACKAGE.md` §4.*

### 4.5 HZ-005: RF Interference

**Hazard**: Transmission on unauthorized frequency

**Controls**:
| Control | Type | Verification |
|---------|------|--------------|
| Frequency hardcoded | Design | Inspection |
| IARU coordination | Procedure | Documentation |
| FCC license | Procedure | Documentation |
| Power limits enforced | Design | Test |

**Residual Risk**: Improbable (E) × Negligible (IV) = Acceptable

**Closure Status**: [x] Open [ ] Closed. *Reopened 2026-10-15 (PI-07): no verification test has been run (§8.1); the hazard log is `docs/safety/SAFETY_DATA_PACKAGE.md` §4.*

### 4.6 HZ-006: Commanded Unsafe State

**Hazard**: Erroneous command causes system damage

**Controls**:
| Control | Type | Verification |
|---------|------|--------------|
| Command validation | Design | Test |
| Command authentication | Design | Test |
| Safe mode auto-trigger | Design | Test |
| Command logging | Design | Test |

**Residual Risk**: Remote (D) × Negligible (IV) = Acceptable

**Closure Status**: [x] Open [ ] Closed. *Reopened 2026-10-15 (PI-07): no verification test has been run (§8.1); the hazard log is `docs/safety/SAFETY_DATA_PACKAGE.md` §4.*

## 5. Software Safety

### 5.1 Software Hazard Analysis

| SW Function | Hazard | Criticality | Controls |
|-------------|--------|-------------|----------|
| RF control | Unintended TX | III | Inhibit timer, interlocks |
| Power management | Overcharge | II | Hardware protection |
| Mode control | Unsafe state | IV | Validation, watchdog |
| Command processing | Bad command | IV | Validation, limits |

### 5.2 Software Safety Requirements

| ID | Requirement | Verification |
|----|-------------|--------------|
| SSR-001 | RF inhibit timer shall be hardware-backed | Test |
| SSR-002 | Watchdog shall trigger safe mode | Test |
| SSR-003 | Commands shall be validated | Test |
| SSR-004 | State transitions shall be logged | Inspection |
| SSR-005 | Battery limits shall be enforced | Test |

### 5.3 Software Safety Verification

| SSR | Method | Status |
|-----|--------|--------|
| SSR-001 | Functional test | [ ] Complete |
| SSR-002 | Unit test | [x] Complete |
| SSR-003 | Unit test | [x] Complete |
| SSR-004 | Code inspection | [x] Complete |
| SSR-005 | Functional test | [ ] Complete |

## 6. Orbital Debris Compliance

### 6.1 NASA-STD-8719.14 Compliance

| Requirement | Compliance | Evidence |
|-------------|------------|----------|
| Limit debris during ops | Compliant | No separating parts |
| Limit debris during EOL | Compliant | Passive deorbit |
| 25-year rule | Compliant | 2-5 year lifetime |
| Passivation | Compliant | Battery discharge |
| Collision avoidance | N/A | No propulsion |

### 6.2 Orbital Debris Assessment

```
ORBITAL DEBRIS ASSESSMENT SUMMARY

Mission Duration: 2 years (design), 5 years (max)
Deorbit Time: 2-5 years from 400 km
25-Year Compliance: YES

Debris Generation:
- Operational debris: None
- Collision debris: Acceptable risk
- Breakup potential: Low (passive)

Passivation:
- Battery: Discharge at EOL
- RF: Disable at EOL
- Stored energy: None

Status: COMPLIANT
```

## 7. Range Safety

### 7.1 Launch Provider Requirements

| Requirement | Status |
|-------------|--------|
| RF inhibit during launch | [ ] Verified |
| Battery certification | [ ] Complete |
| No pyrotechnics | [x] N/A |
| No pressurized systems | [x] N/A |
| ICD compliance | [ ] Verified |

### 7.2 Integration Safety

| Phase | Safety Requirement | Status |
|-------|-------------------|--------|
| Transport | Handling procedures | [ ] Ready |
| Integration | ESD controls | [ ] Ready |
| Testing | Facility safety | [ ] Ready |
| Launch ops | RF inhibit verified | [ ] Ready |

## 8. Safety Verification Matrix

### 8.1 Verification Status

| Hazard | Analysis | Test | Inspection | Demo |
|--------|----------|------|------------|------|
| HZ-001 | [x] | [ ] | [x] | [ ] |
| HZ-002 | [x] | [ ] | [x] | [ ] |
| HZ-003 | [x] | [ ] | [x] | [ ] |
| HZ-004 | [x] | N/A | N/A | N/A |
| HZ-005 | [x] | [ ] | [x] | [ ] |
| HZ-006 | [x] | [ ] | [x] | [ ] |

### 8.2 Verification Evidence

| Hazard | Evidence Document |
|--------|-------------------|
| HZ-001 | RF test report, timer verification |
| HZ-002 | Battery test report, protection test |
| HZ-003 | Deployment test report |
| HZ-004 | Orbital debris assessment |
| HZ-005 | Frequency coordination docs |
| HZ-006 | Command validation test report |

## 9. Safety Risk Summary

### 9.1 Risk Matrix

```
SAFETY RISK MATRIX

           Probability
Severity  A    B    C    D    E
─────────────────────────────────
   I     ██   ██   ██   ░░   ░░    HZ-002
  II     ██   ██   ░░   ░░   ░░    HZ-004
 III     ██   ░░   ░░   ░░   ░░    HZ-001, HZ-003
  IV     ░░   ░░   ░░   ░░   ░░    HZ-005, HZ-006

██ = Unacceptable   ░░ = Acceptable
```

### 9.2 Residual Risk Summary

| Hazard | Initial Risk | Controls | Residual Risk | Acceptable |
|--------|--------------|----------|---------------|------------|
| HZ-001 | C-III | 4 controls | D-III | Yes |
| HZ-002 | C-I | 5 controls | E-II | Yes |
| HZ-003 | C-III | 4 controls | D-III | Yes |
| HZ-004 | D-II | 4 controls | E-II | Yes |
| HZ-005 | C-IV | 4 controls | E-IV | Yes |
| HZ-006 | C-IV | 4 controls | D-IV | Yes |

## 10. Safety Review Checklist

### 10.1 Hazard Analysis

- [x] Preliminary hazard list complete
- [x] System hazard analysis complete
- [x] Software hazard analysis complete
- [x] All hazards have controls
- [x] Residual risks acceptable

### 10.2 Verification

- [ ] All safety requirements verified
- [ ] Safety tests executed
- [x] Safety inspections complete
- [x] Safety analysis complete

### 10.3 Documentation

- [x] Safety data package complete
- [x] Hazard tracking log current
- [x] Waiver requests (if any) submitted
- [x] Safety review package complete

## 11. Safety Board Decision

```
SAFETY REVIEW DECISION

Date: ___________

The Safety Review Board has reviewed the
SMART-QSO safety package and determines:

[ ] SAFETY ACCEPTABLE
    - All hazards identified and controlled
    - Residual risks acceptable
    - Proceed to next phase

[ ] CONDITIONAL ACCEPTANCE
    - Conditions: ___________
    - Resolution required by: ___________

[ ] NOT ACCEPTABLE
    - Rationale: ___________
    - Required actions: ___________

Signatures:

Safety Engineer: __________ Date: _____
Project Manager: __________ Date: _____
Systems Engineer: _________ Date: _____
```

## 12. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): historical-record banner (superseded by BASELINE, SQSO-REV-CDR-002 and the current safety data packages); notice cites finding PI-03 (was F-PI-03); body unchanged |
