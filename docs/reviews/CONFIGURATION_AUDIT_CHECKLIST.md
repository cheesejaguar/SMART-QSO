# SMART-QSO Configuration Audit Checklist

**Document ID**: SQSO-CA-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides checklists for conducting Functional Configuration Audit (FCA) and Physical Configuration Audit (PCA) per NPR 7120.5 §3.4 requirements.

## 2. Functional Configuration Audit (FCA)

The FCA verifies that the software performs as specified in its requirements documentation.

### 2.1 Requirements Verification

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| FCA-001 | All requirements have unique identifiers | SRS document | [ ] | |
| FCA-002 | All requirements are testable | VVM traceability | [ ] | |
| FCA-003 | All requirements traced to tests | VVM complete | [ ] | |
| FCA-004 | All test procedures executed | Test reports | [ ] | |
| FCA-005 | All tests passed | Test results | [ ] | |
| FCA-006 | All waivers documented | Waiver log | [ ] | |

### 2.2 Test Coverage

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| FCA-007 | Unit test coverage > 80% | Coverage report | [ ] | |
| FCA-008 | Integration tests complete | Test report | [ ] | |
| FCA-009 | System tests complete | Test report | [ ] | |
| FCA-010 | Regression tests passed | Test report | [ ] | |
| FCA-011 | Performance tests complete | Test report | [ ] | |
| FCA-012 | Stress tests complete | Test report | [ ] | |

### 2.3 Anomaly Resolution

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| FCA-013 | All anomalies documented | Anomaly log | [ ] | |
| FCA-014 | Critical anomalies resolved | Closure records | [ ] | |
| FCA-015 | High anomalies resolved/waived | Closure records | [ ] | |
| FCA-016 | Medium anomalies dispositioned | Status report | [ ] | |
| FCA-017 | Open anomalies acceptable | Risk assessment | [ ] | |
| FCA-018 | Anomaly trends analyzed | Trend report | [ ] | |

### 2.4 Documentation Review

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| FCA-019 | SRS at correct version | Version control | [ ] | |
| FCA-020 | ICD at correct version | Version control | [ ] | |
| FCA-021 | Test procedures reviewed | Review records | [ ] | |
| FCA-022 | User documentation complete | Doc set | [ ] | |
| FCA-023 | Operator manuals complete | Doc set | [ ] | |
| FCA-024 | Maintenance procedures complete | Doc set | [ ] | |

## 3. Physical Configuration Audit (PCA)

The PCA verifies that the as-built software matches the design documentation.

### 3.1 Source Code Verification

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| PCA-001 | Source code under version control | Git repository | [ ] | |
| PCA-002 | Release tag created | Git tag | [ ] | |
| PCA-003 | All files at correct version | File listing | [ ] | |
| PCA-004 | No uncommitted changes | Git status | [ ] | |
| PCA-005 | Branch protection enforced | Repo settings | [ ] | |
| PCA-006 | Code review complete | PR records | [ ] | |

### 3.2 Build Verification

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| PCA-007 | Build environment documented | Build guide | [ ] | |
| PCA-008 | Build is reproducible | Repeat build | [ ] | |
| PCA-009 | Build produces identical binary | Hash comparison | [ ] | |
| PCA-010 | Dependencies at correct versions | Lock file | [ ] | |
| PCA-011 | Compiler version documented | Build log | [ ] | |
| PCA-012 | Build warnings resolved | Build log | [ ] | |

### 3.3 Binary Verification

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| PCA-013 | Binary checksum recorded | Checksum file | [ ] | |
| PCA-014 | Binary size within limits | Size report | [ ] | |
| PCA-015 | Symbols stripped (if required) | File analysis | [ ] | |
| PCA-016 | Debug symbols archived | Archive | [ ] | |
| PCA-017 | Binary signed (if required) | Signature | [ ] | |
| PCA-018 | Binary versioned correctly | Version check | [ ] | |

### 3.4 Configuration Items

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| PCA-019 | All CIs identified | CI list | [ ] | |
| PCA-020 | CI versions documented | Version list | [ ] | |
| PCA-021 | CI relationships documented | Dependency tree | [ ] | |
| PCA-022 | CI change history complete | Change log | [ ] | |
| PCA-023 | CI baselines established | Baseline record | [ ] | |
| PCA-024 | CI access controlled | Access log | [ ] | |

### 3.5 As-Built Documentation

| ID | Item | Evidence Required | Verified | Notes |
|----|------|-------------------|----------|-------|
| PCA-025 | Code matches design | Design review | [ ] | |
| PCA-026 | Interfaces match ICD | ICD verification | [ ] | |
| PCA-027 | Memory map accurate | Linker output | [ ] | |
| PCA-028 | Timing analysis valid | Analysis report | [ ] | |
| PCA-029 | Resource usage documented | Usage report | [ ] | |
| PCA-030 | Deviations documented | Deviation log | [ ] | |

## 4. Configuration Item List

### 4.1 Software Configuration Items

| CI ID | Name | Version | Baseline Date | Owner |
|-------|------|---------|---------------|-------|
| SQSO-FSW | Flight Software | 1.0.0 | TBD | Software Lead |
| SQSO-PSW | Payload Software | 1.0.0 | TBD | Payload Lead |
| SQSO-GSW | Ground Software | 1.0.0 | TBD | Ground Lead |
| SQSO-BSC | Build Scripts | 1.0.0 | TBD | Software Lead |
| SQSO-CFG | Configuration Files | 1.0.0 | TBD | Systems Engineer |

### 4.2 Documentation Configuration Items

| CI ID | Name | Version | Baseline Date | Owner |
|-------|------|---------|---------------|-------|
| SQSO-SRS | Software Requirements | 1.0 | TBD | Systems Engineer |
| SQSO-SDD | Software Design | 1.0 | TBD | Software Lead |
| SQSO-ICD | Interface Control | 1.0 | TBD | Systems Engineer |
| SQSO-VVM | Verification Matrix | 1.0 | TBD | V&V Lead |
| SQSO-UMN | User Manual | 1.0 | TBD | Documentation |

## 5. Audit Procedure

### 5.1 Pre-Audit Preparation

1. [ ] Schedule audit with all stakeholders
2. [ ] Distribute audit checklist 5 days prior
3. [ ] Collect required evidence
4. [ ] Prepare audit room/tools
5. [ ] Brief audit team

### 5.2 Audit Execution

1. [ ] Opening briefing
2. [ ] Review FCA items (Section 2)
3. [ ] Review PCA items (Section 3)
4. [ ] Document findings
5. [ ] Closing briefing

### 5.3 Post-Audit Activities

1. [ ] Generate audit report
2. [ ] Distribute findings
3. [ ] Track corrective actions
4. [ ] Verify corrections
5. [ ] Close audit

## 6. Audit Findings Template

### 6.1 Finding Record

```
CONFIGURATION AUDIT FINDING

Finding ID: CAF-XXX
Date: ___________
Auditor: ___________

FINDING TYPE:
[ ] Non-conformance
[ ] Observation
[ ] Improvement opportunity

CHECKLIST ITEM: ___________

DESCRIPTION:
_____________________________________________
_____________________________________________

EVIDENCE:
_____________________________________________

RECOMMENDED ACTION:
_____________________________________________

PRIORITY:
[ ] Critical (blocks release)
[ ] Major (requires resolution)
[ ] Minor (track for closure)

ASSIGNED TO: ___________
DUE DATE: ___________
```

### 6.2 Finding Closure Record

```
FINDING CLOSURE

Finding ID: CAF-XXX
Closure Date: ___________

CORRECTIVE ACTION TAKEN:
_____________________________________________

EVIDENCE OF CORRECTION:
_____________________________________________

VERIFIED BY: ___________
DATE: ___________

CLOSURE APPROVED BY: ___________
DATE: ___________
```

## 7. Audit Report Template

```
SMART-QSO CONFIGURATION AUDIT REPORT

Audit Date: ___________
Audit Type: [ ] FCA [ ] PCA [ ] Combined
Configuration: ___________

PARTICIPANTS:
- Audit Lead: ___________
- Auditors: ___________
- Auditees: ___________

SCOPE:
_____________________________________________

SUMMARY:
- Total items reviewed: ___
- Items passed: ___
- Findings raised: ___
- Critical findings: ___
- Major findings: ___
- Minor findings: ___

FINDINGS LIST:
| ID | Type | Priority | Description | Status |
|----|------|----------|-------------|--------|
| | | | | |

CONCLUSIONS:
_____________________________________________

RECOMMENDATIONS:
_____________________________________________

AUDIT RESULT:
[ ] Passed - no findings
[ ] Passed with conditions
[ ] Failed - critical findings

Audit Lead: ___________ Date: ___________
Project Manager: ___________ Date: ___________
```

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
