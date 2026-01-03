# Configuration Audit Procedures

**Document Number:** SQSO-CAP-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline
**Reference:** NPR 7120.5 Section 3.4, NPR 7123.1 SE Process 13

---

## 1. Purpose

This document defines the procedures for conducting Functional Configuration Audits (FCA) and Physical Configuration Audits (PCA) for the SMART-QSO CubeSat flight software, in accordance with NASA NPR 7120.5 configuration management requirements.

## 2. Scope

These procedures apply to:
- OBC Flight Software (SW-001)
- Jetson Payload Software (SW-002)
- Associated firmware and documentation
- All configuration items identified in the Configuration Management Plan

## 3. References

| Document | Description |
|----------|-------------|
| NPR 7120.5 | NASA Program and Project Management Requirements |
| NPR 7123.1 | NASA Systems Engineering Processes and Requirements |
| NPR 7150.2 | NASA Software Engineering Requirements |
| SQSO-CMP-001 | Configuration Management Plan |
| SQSO-SRS-001 | Software Requirements Specification |

---

## 4. Functional Configuration Audit (FCA)

### 4.1 Purpose

The FCA verifies that the actual performance of the configuration item meets the requirements specified in its functional and allocated configuration documentation.

### 4.2 Timing

- Conducted prior to Test Readiness Review (TRR)
- Required before any baseline is established
- Repeated after major software changes

### 4.3 Entry Criteria

- [ ] All software requirements documented in SRS
- [ ] Requirements Traceability Matrix complete
- [ ] All planned tests executed
- [ ] Test reports available for review
- [ ] Known defects documented with dispositions

### 4.4 FCA Checklist

#### 4.4.1 Requirements Verification

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| FCA-001 | All requirements have unique identifiers | | RTM |
| FCA-002 | All requirements are traceable to tests | | RTM |
| FCA-003 | All requirements are verifiable | | VCRM |
| FCA-004 | No orphan requirements exist | | RTM analysis |
| FCA-005 | All requirements have verification status | | VCRM |

#### 4.4.2 Test Coverage

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| FCA-010 | Unit test coverage ≥ 80% | | Coverage report |
| FCA-011 | All unit tests pass | | CI test results |
| FCA-012 | Integration tests executed | | Test reports |
| FCA-013 | System tests executed | | Test reports |
| FCA-014 | Regression tests pass | | CI test results |

#### 4.4.3 Performance Verification

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| FCA-020 | Timing requirements verified | | Timing analysis |
| FCA-021 | Memory usage within limits | | Memory analysis |
| FCA-022 | CPU utilization acceptable | | Performance tests |
| FCA-023 | Stack usage verified | | Stack analysis |

#### 4.4.4 Safety Requirements

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| FCA-030 | All safety requirements verified | | Safety V&V |
| FCA-031 | Hazard mitigations implemented | | FMEA/FTA |
| FCA-032 | Safe mode tested | | Test report |
| FCA-033 | Fault detection functional | | Test report |

#### 4.4.5 Defect Status

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| FCA-040 | No open critical defects | | Defect log |
| FCA-041 | No open major defects (or waived) | | Defect log |
| FCA-042 | Known issues documented | | Known issues |
| FCA-043 | Waivers approved by CCB | | CCB minutes |

### 4.5 FCA Exit Criteria

- All FCA checklist items complete or waived
- FCA report signed by Configuration Manager
- FCA findings closed or documented
- Authorization to proceed to PCA

---

## 5. Physical Configuration Audit (PCA)

### 5.1 Purpose

The PCA verifies that the as-built configuration matches the design documentation and that the product can be accurately reproduced from the configuration documentation.

### 5.2 Timing

- Conducted prior to Flight Readiness Review (FRR)
- Required before flight software delivery
- Performed after final code freeze

### 5.3 Entry Criteria

- [ ] FCA complete and approved
- [ ] Software build frozen (no changes)
- [ ] All documentation updated
- [ ] Build from source verified
- [ ] Flight binary available

### 5.4 PCA Checklist

#### 5.4.1 Build Verification

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-001 | Source code tagged with version | | Git tag |
| PCA-002 | Build from tag succeeds | | Build log |
| PCA-003 | Binary matches flight binary | | Hash comparison |
| PCA-004 | Build environment documented | | Build instructions |
| PCA-005 | Dependencies documented | | Dependency list |

#### 5.4.2 Documentation Verification

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-010 | SRS reflects as-built | | Document review |
| PCA-011 | SDD reflects as-built | | Document review |
| PCA-012 | ICDs reflect as-built | | Document review |
| PCA-013 | User manual complete | | Document review |
| PCA-014 | Version description complete | | SVD document |

#### 5.4.3 Configuration Item Verification

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-020 | All CIs identified | | CI index |
| PCA-021 | All CIs at correct version | | Version check |
| PCA-022 | CI index complete | | CI index |
| PCA-023 | Baseline established | | Git tag |

#### 5.4.4 Reproducibility

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-030 | Build reproducible on clean system | | Test build |
| PCA-031 | Binary reproducible | | Hash match |
| PCA-032 | Test results reproducible | | Test rerun |
| PCA-033 | Archive complete | | Archive checklist |

### 5.5 PCA Exit Criteria

- All PCA checklist items complete or waived
- PCA report signed by Configuration Manager
- PCA findings closed or documented
- Authorization for flight software delivery

---

## 6. Audit Procedures

### 6.1 Audit Team

| Role | Responsibilities |
|------|------------------|
| Audit Lead | Plan and conduct audit, prepare report |
| Configuration Manager | Provide documentation, support audit |
| Software Lead | Answer technical questions |
| QA Representative | Verify process compliance |

### 6.2 Audit Process

```
1. Pre-Audit Planning
   ├── Define audit scope
   ├── Assemble audit team
   ├── Request documentation
   └── Schedule audit sessions

2. Audit Execution
   ├── Opening meeting
   ├── Document review
   ├── Evidence collection
   ├── Findings documentation
   └── Exit briefing

3. Post-Audit Activities
   ├── Prepare audit report
   ├── Review findings with team
   ├── Track corrective actions
   └── Close audit
```

### 6.3 Finding Categories

| Category | Description | Required Action |
|----------|-------------|-----------------|
| Critical | Prevents mission success | Must resolve before proceeding |
| Major | Significant issue | Resolve or obtain waiver |
| Minor | Process improvement | Track for resolution |
| Observation | Suggestion for improvement | Optional action |

---

## 7. Audit Reports

### 7.1 FCA Report Template

```
FUNCTIONAL CONFIGURATION AUDIT REPORT

Project: SMART-QSO
CI: SW-001 OBC Flight Software
Version: vX.Y.Z
Date: YYYY-MM-DD

1. EXECUTIVE SUMMARY
   [Brief summary of audit results]

2. AUDIT SCOPE
   [What was audited]

3. AUDIT TEAM
   [Names and roles]

4. FINDINGS
   | ID | Category | Description | Status |
   |----|----------|-------------|--------|

5. CHECKLIST RESULTS
   [Completed checklist with status]

6. CONCLUSIONS
   [Pass/Fail determination]

7. RECOMMENDATIONS
   [Any recommendations]

8. APPROVAL
   Configuration Manager: _____________ Date: _____
   Software Lead: _____________ Date: _____
   QA: _____________ Date: _____
```

### 7.2 PCA Report Template

```
PHYSICAL CONFIGURATION AUDIT REPORT

Project: SMART-QSO
CI: SW-001 OBC Flight Software
Version: vX.Y.Z
Date: YYYY-MM-DD

1. EXECUTIVE SUMMARY
   [Brief summary of audit results]

2. CONFIGURATION ITEM STATUS
   | CI ID | Name | Version | Location | Verified |
   |-------|------|---------|----------|----------|

3. BUILD VERIFICATION
   Source Tag: [tag]
   Build Date: [date]
   Binary Hash: [SHA-256]

4. DOCUMENTATION STATUS
   | Document | Version | As-Built Match |
   |----------|---------|----------------|

5. FINDINGS
   | ID | Category | Description | Status |
   |----|----------|-------------|--------|

6. ARCHIVE CONTENTS
   [List of archived items]

7. CONCLUSIONS
   [Pass/Fail determination]

8. APPROVAL
   Configuration Manager: _____________ Date: _____
   Project Manager: _____________ Date: _____
   QA: _____________ Date: _____
```

---

## 8. Corrective Action Process

### 8.1 Finding Resolution

```
Finding Identified
       │
       v
   Assigned to Owner
       │
       v
   Root Cause Analysis
       │
       v
   Corrective Action Plan
       │
       v
   Implementation
       │
       v
   Verification
       │
       v
   Finding Closed
```

### 8.2 Tracking

All findings tracked in project issue tracking system with:
- Unique finding ID (FCA-xxx or PCA-xxx)
- Category and description
- Assigned owner
- Due date
- Resolution status

---

## 9. Records Retention

All audit records retained per project records retention policy:
- Audit reports: Permanent
- Checklists: Permanent
- Supporting evidence: 5 years post-mission
- Corrective action records: Permanent

---

## Appendix A: Quick Reference Checklist

### Pre-FCA Checklist
- [ ] Requirements Traceability Matrix complete
- [ ] All tests executed and results available
- [ ] Defect log current
- [ ] Documentation up to date

### Pre-PCA Checklist
- [ ] FCA complete and approved
- [ ] Code freeze in effect
- [ ] Version tag created
- [ ] Build verification complete
- [ ] Documentation matches as-built

---

## Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
