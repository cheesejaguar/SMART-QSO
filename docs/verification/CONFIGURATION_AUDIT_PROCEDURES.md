# Configuration Audit Procedures

**Document Number:** SQSO-CAP-001
**Version:** 1.1
**Date:** 2026-10-16
**Status:** CDR Baseline
**Reference:** NPR 7120.5 Section 3.4, NPR 7123.1 SE Process 13
**Supersedes:** `docs/reviews/CONFIGURATION_AUDIT_CHECKLIST.md` (SQSO-CA-001) and `docs/reviews/CONFIGURATION_AUDIT_CHECKLISTS.md` (SQSO-CAC-001), merged here (finding PI-17)

---

## 1. Purpose

This document defines the procedures for conducting Functional Configuration Audits (FCA) and Physical Configuration Audits (PCA) for the SMART-QSO CubeSat flight software, in accordance with NASA NPR 7120.5 configuration management requirements.

Version 1.1 merges the two checklist documents that duplicated this one (SQSO-CA-001 and SQSO-CAC-001). Their software items are added to §4.4 and §5.4 under new item numbers, their report and finding templates to §7, and their system-level (hardware) checklists are Appendix B. Where the three documents disagreed, the difference is noted where it occurs. The configuration item list in SQSO-CA-001 §4 is not carried; the software configuration index is SQSO-SW-006 (`docs/software/CONFIGURATION_INDEX.md`).

## 2. Scope

These procedures apply to:
- OBC Flight Software (SW-001)
- Jetson Payload Software (SW-002)
- Associated firmware and documentation
- All configuration items identified in the Configuration Management Plan
- Flight hardware, for the system-level checklists in Appendix B

## 3. References

| Document | Description |
|----------|-------------|
| NPR 7120.5 | NASA Program and Project Management Requirements |
| NPR 7123.1 | NASA Systems Engineering Processes and Requirements |
| NPR 7150.2 | NASA Software Engineering Requirements |
| SMART-QSO-CMP-001 | Configuration Management Plan |
| SMART-QSO-SRS-001 | Software Requirements Specification |
| SQSO-BL-001 | Baseline Parameters (`docs/BASELINE_PARAMETERS.md`) |

---

## 4. Functional Configuration Audit (FCA)

### 4.1 Purpose

The FCA verifies that the actual performance of the configuration item meets the requirements specified in its functional and allocated configuration documentation.

### 4.2 Timing

- Conducted prior to Test Readiness Review (TRR)
- Required before any baseline is established
- Repeated after major software changes
- Delta FCA before launch for any change made after the Flight Readiness Review (FRR)

SQSO-CAC-001 scheduled the full FCA before FRR rather than before TRR. This document's timing stands until the Configuration Management Plan sets it.

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
| FCA-015 | All executed tests passed, or failures dispositioned | | Test reports |
| FCA-016 | Stress tests executed | | Test report |

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

#### 4.4.6 Anomaly Resolution

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| FCA-050 | All anomalies documented | | Anomaly log |
| FCA-051 | Critical anomalies resolved | | Closure records |
| FCA-052 | High anomalies resolved or waived | | Closure records |
| FCA-053 | Medium anomalies dispositioned | | Status report |
| FCA-054 | Remaining open anomalies accepted | | Risk assessment |
| FCA-055 | Anomaly trends analyzed | | Trend report |

#### 4.4.7 Documentation Review

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| FCA-060 | SRS at the audited revision | | `docs/DOCUMENT_REGISTER.md` |
| FCA-061 | ICDs at the audited revision | | `docs/DOCUMENT_REGISTER.md` |
| FCA-062 | Test procedures reviewed | | Review records |
| FCA-063 | User documentation available | | Document set |
| FCA-064 | Operator manuals available | | Document set |
| FCA-065 | Maintenance procedures available | | Document set |

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
- Delta PCA before launch for any rework made after FRR

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
| PCA-006 | Compiler version documented | | Build log |
| PCA-007 | Build warnings resolved or recorded | | Build log |

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
| PCA-024 | CI relationships documented | | Dependency tree |
| PCA-025 | CI change history complete | | Change log |
| PCA-026 | CI access controlled | | Repository settings |

#### 5.4.4 Reproducibility

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-030 | Build reproducible on clean system | | Test build |
| PCA-031 | Binary reproducible | | Hash match |
| PCA-032 | Test results reproducible | | Test rerun |
| PCA-033 | Archive complete | | Archive checklist |

#### 5.4.5 Source Control

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-040 | No uncommitted changes in the audited build | | `git status` |
| PCA-041 | Branch protection enforced on the release branch | | Repository settings |
| PCA-042 | Every change reviewed before merge | | Pull request records |

#### 5.4.6 Binary Verification

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-050 | Binary checksum recorded | | Checksum file |
| PCA-051 | Binary size within limits | | Size report |
| PCA-052 | Symbols stripped (if required) | | File analysis |
| PCA-053 | Debug symbols archived | | Archive |
| PCA-054 | Binary signed (if required) | | Signature |
| PCA-055 | Binary version string matches the release | | Version check |

#### 5.4.7 As-Built Documentation

| Item | Description | Status | Evidence |
|------|-------------|--------|----------|
| PCA-060 | Code matches design | | Design review |
| PCA-061 | Interfaces match ICDs | | ICD verification |
| PCA-062 | Memory map matches the linker output | | Linker map |
| PCA-063 | Timing analysis valid for the audited build | | Analysis report |
| PCA-064 | Resource usage documented | | Usage report |
| PCA-065 | Deviations documented | | Deviation log |

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
| Systems Engineer | Technical verification for the system-level checklists (Appendix B) |
| QA Representative | Verify process compliance |

### 6.2 Audit Process

```
1. Pre-Audit Planning
   ├── Define audit scope
   ├── Assemble audit team
   ├── Request documentation
   ├── Distribute the checklist 5 days before the audit
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
   ├── Verify corrections
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

### 7.3 Combined Audit Report Template

Used when the FCA and PCA are held together (from SQSO-CAC-001). The report gets its own number in `docs/DOCUMENT_REGISTER.md` when it is issued.

```
CONFIGURATION AUDIT REPORT

Report number: _______________ (assigned in the document register)
Date: _______________
Auditors: _______________

1. EXECUTIVE SUMMARY
   - Audit scope
   - Key findings
   - Recommendations

2. FCA RESULTS
   - Requirements status
   - Verification status
   - Discrepancies

3. PCA RESULTS
   - As-built status
   - Documentation status
   - Discrepancies

4. COMBINED FINDINGS
   - All discrepancies
   - Dispositions
   - Open items

5. CONCLUSIONS
   - Audit status (PASS/CONDITIONAL/FAIL)
   - Conditions for closure
   - Actions required

6. SIGNATURES
   - Audit team
   - Project manager
   - Customer (if applicable)
```

### 7.4 Finding Record Template

From SQSO-CA-001.

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

### 7.5 Finding Closure Record Template

From SQSO-CA-001.

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

### 8.3 Resolution Time and Closeout

From SQSO-CAC-001.

| Priority | Resolution Time | Authority |
|----------|-----------------|-----------|
| Critical | Before FRR | PM + Customer |
| Major | Before FRR | CCB |
| Minor | Before launch | Systems Engr |
| Editorial | Best effort | Doc owner |

| Action | Responsibility | Due |
|--------|----------------|-----|
| Discrepancies resolved | Owner | Pre-FRR |
| Documentation updated | CM | Pre-FRR |
| Final report issued | Audit Lead | FRR-7 days |
| Audit closed | PM | FRR |

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

## Appendix B: System-Level FCA and PCA Checklists

Merged from SQSO-CAC-001. Values marked (BL) follow `docs/BASELINE_PARAMETERS.md`, which replaced values in SQSO-CAC-001
that disagreed with it. Values marked TBR come from SQSO-CAC-001, are not in the baseline, and are to be confirmed against the
requirement specifications. Drawing and specification numbers (DWG-, SPEC-) are placeholders: no drawings are released.

### B.1 FCA Entry Criteria

| Criterion | Verification | Status |
|-----------|--------------|--------|
| All requirements baselined | RTM review | [ ] |
| All verification complete | VCRM review | [ ] |
| Test reports available | Document check | [ ] |
| Anomalies resolved/waived | NCR review | [ ] |
| Software version baselined | CM check | [ ] |

### B.2 FCA Checklist - System and Software Requirements

One row per requirement, taken from the specifications themselves. (SQSO-CAC-001 listed example rows whose requirement
numbers did not match the specifications, so they are not carried.)

| Req ID | Requirement | Test/Analysis | Pass/Fail | Evidence |
|--------|-------------|---------------|-----------|----------|
| *Each requirement in `docs/requirements/SYSTEM_REQUIREMENTS.md`* | | | [ ] | |
| *Each requirement in `docs/software/SOFTWARE_REQUIREMENTS.md`* | | | [ ] | |

### B.3 FCA Checklist - Interface Requirements

| Interface | ICD | Verified | Pass/Fail | Evidence |
|-----------|-----|----------|-----------|----------|
| OBC-EPS | SMART-QSO-ICD-001, audited revision | Test | [ ] | |
| OBC-RF | SQSO-ICD-003, audited revision | Test | [ ] | |
| OBC-Jetson | SMART-QSO-ICD-002, audited revision | Test | [ ] | |
| OBC-ADCS | SQSO-ICD-004, audited revision | Test | [ ] | |
| RF-Antenna | No ICD written | Test | [ ] | |
| Ground-Space | SQSO-ICD-005, audited revision | Test | [ ] | |
| Deployer | SQSO-ICD-011 (not written) | Inspection | [ ] | |

### B.4 FCA Checklist - Performance

| Parameter | Requirement | Measured | Pass/Fail |
|-----------|-------------|----------|-----------|
| Power (Safe) | ≤0.2 W (TBR) | | [ ] |
| Power (Idle) | ≤0.4 W (TBR) | | [ ] |
| Power (Active) | ≤0.8 W (TBR) | | [ ] |
| Power (AI Burst) | Jetson ≤ 8 W input including carrier, 7 W power mode (BL §6) | | [ ] |
| TX Power | ≤ 1 W maximum, 0.5 W nominal (BL §7) | | [ ] |
| Data Rate | 1200 bps AFSK telemetry (BL §2) | | [ ] |
| Beacon Period | 60 s ACTIVE, 120 s IDLE, 180 s SAFE (BL §8) | | [ ] |
| Boot Time | <30 s (TBR) | | [ ] |
| Mode Transition | <5 s (TBR) | | [ ] |
| Fault Response | <5 s (TBR) | | [ ] |

### B.5 FCA Discrepancy Log

| ID | Requirement | Discrepancy | Disposition | Status |
|----|-------------|-------------|-------------|--------|
| | | | | |
| | | | | |

### B.6 FCA Closure

```
FUNCTIONAL CONFIGURATION AUDIT CLOSURE

Audit Date: _______________
Audit Lead: _______________

Summary:
- Requirements Audited: ___ / ___
- Requirements Verified: ___ / ___
- Open Discrepancies: ___
- Waivers Required: ___

Findings:
[ ] All requirements verified
[ ] Discrepancies documented and dispositioned
[ ] Waivers approved by CCB
[ ] Ready for FRR

Signatures:
Audit Lead: _____________ Date: _______
Systems Engineer: _____________ Date: _______
QA Lead: _____________ Date: _______
Project Manager: _____________ Date: _______
```

### B.7 PCA Entry Criteria

| Criterion | Verification | Status |
|-----------|--------------|--------|
| Flight unit assembled | Inspection | [ ] |
| As-built list available | CM check | [ ] |
| Drawings released | CM check | [ ] |
| Parts list current | CM check | [ ] |
| Serial numbers recorded | Traveler check | [ ] |

### B.8 PCA Checklist - Structure

| Item | Drawing | As-Built | Match | Notes |
|------|---------|----------|-------|-------|
| Primary structure | DWG-STR-001 | S/N: | [ ] | |
| +X Panel | DWG-STR-002 | S/N: | [ ] | |
| -X Panel | DWG-STR-003 | S/N: | [ ] | |
| +Y Panel | DWG-STR-004 | S/N: | [ ] | |
| -Y Panel | DWG-STR-005 | S/N: | [ ] | |
| +Z Panel | DWG-STR-006 | S/N: | [ ] | |
| -Z Panel | DWG-STR-007 | S/N: | [ ] | |
| Antenna mount | DWG-STR-010 | S/N: | [ ] | |
| Deployment mechanism | DWG-STR-011 | S/N: | [ ] | |

### B.9 PCA Checklist - EPS

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| EPS Board | DWG-EPS-001 | S/N: | [ ] | |
| Solar cells, 16 (BL §5) | SPEC-EPS-010 | Lot, per cell: | [ ] | One row per cell |
| Battery cells, 4 (4S1P 18650, BL §5) | SPEC-EPS-020 | S/N, per cell: | [ ] | One row per cell |

### B.10 PCA Checklist - OBC

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| OBC Board | DWG-OBC-001 | S/N: | [ ] | |
| OBC MCU, STM32L4R5ZI (BL §6) | SPEC-OBC-010 | Lot: | [ ] | |
| FRAM | SPEC-OBC-020 | Lot: | [ ] | |
| External Flash | SPEC-OBC-030 | Lot: | [ ] | |
| Watchdog IC | SPEC-OBC-040 | Lot: | [ ] | |

### B.11 PCA Checklist - RF

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| RF Board | DWG-RF-001 | S/N: | [ ] | |
| Si4463 | SPEC-RF-010 | Lot: | [ ] | |
| VHF PA | SPEC-RF-020 | S/N: | [ ] | |
| UHF LNA | SPEC-RF-030 | Lot: | [ ] | |
| VHF Antenna | DWG-RF-050 | S/N: | [ ] | |
| UHF Antenna | DWG-RF-051 | S/N: | [ ] | |

### B.12 PCA Checklist - AI Payload

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| Jetson Orin Nano 8 GB module (BL §6) | SPEC-PL-001 | S/N: | [ ] | |
| Carrier Board | DWG-PL-001 | S/N: | [ ] | |
| Thermal Pad | SPEC-PL-010 | Lot: | [ ] | |
| Heatsink | DWG-PL-020 | P/N: | [ ] | |

### B.13 PCA Checklist - Harness

| Harness | Drawing | Length | Connectors | Match |
|---------|---------|--------|------------|-------|
| EPS-OBC | DWG-HAR-001 | | | [ ] |
| OBC-RF | DWG-HAR-002 | | | [ ] |
| OBC-Jetson | DWG-HAR-003 | | | [ ] |
| OBC-ADCS | DWG-HAR-004 | | | [ ] |
| RF-Antenna | DWG-HAR-005 | | | [ ] |
| Solar-EPS | DWG-HAR-006 | | | [ ] |

### B.14 PCA Checklist - Software

Versions are those in the Software Version Description (SQSO-SVD-001) for the audited release.

| Software | Version | Loaded | Verified | Match |
|----------|---------|--------|----------|-------|
| Flight Software | Per SVD | [ ] | CRC: | [ ] |
| Golden Image | Per SVD | [ ] | CRC: | [ ] |
| Payload Software | Per SVD | [ ] | CRC: | [ ] |
| AI model files (BL §6) | Per SVD | [ ] | CRC: | [ ] |
| Configuration | Per SVD | [ ] | CRC: | [ ] |

### B.15 PCA Checklist - Mass Properties

| Property | Requirement | Measured | Pass/Fail |
|----------|-------------|----------|-----------|
| Total Mass | ≤ 2.00 kg, or the integrator's dispenser limit if lower (BL §4) | | [ ] |
| CG-X | ±2 cm | | [ ] |
| CG-Y | ±2 cm | | [ ] |
| CG-Z | ±2 cm | | [ ] |

### B.16 PCA Checklist - Documentation

Revisions are those listed in `docs/DOCUMENT_REGISTER.md` at the time of the audit.

| Document | Revision | Current | Match |
|----------|----------|---------|-------|
| System Requirements | Per register | [ ] | [ ] |
| Software Requirements | Per register | [ ] | [ ] |
| Software Design | Per register | [ ] | [ ] |
| All ICDs | Per register | [ ] | [ ] |
| Test Reports | Per register | [ ] | [ ] |
| Parts List | Per register | [ ] | [ ] |
| As-Built List | Per register | [ ] | [ ] |

### B.17 PCA Discrepancy Log

| ID | Item | Discrepancy | Disposition | Status |
|----|------|-------------|-------------|--------|
| | | | | |
| | | | | |

### B.18 PCA Closure

```
PHYSICAL CONFIGURATION AUDIT CLOSURE

Audit Date: _______________
Audit Lead: _______________

Summary:
- Items Audited: ___ / ___
- Items Match: ___ / ___
- Open Discrepancies: ___
- Deviations Documented: ___

Findings:
[ ] All items match documentation
[ ] Discrepancies documented and dispositioned
[ ] As-built list complete and accurate
[ ] Ready for FRR

Signatures:
Audit Lead: _____________ Date: _______
Integration Lead: _____________ Date: _______
QA Lead: _____________ Date: _______
Project Manager: _____________ Date: _______
```

---

## Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | SMART-QSO Team | Merged SQSO-CA-001 and SQSO-CAC-001, which duplicated this document (finding PI-17): new FCA items 015–016, 050–065; new PCA items 006–007, 024–026, 040–065; combined report and finding templates (§7.3–7.5); resolution times (§8.3); system-level checklists (Appendix B), with values that disagreed with the baseline replaced by baseline values and the rest marked TBR. CMP and SRS cited by their own numbers |
