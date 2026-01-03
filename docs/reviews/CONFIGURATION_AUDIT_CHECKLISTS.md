# SMART-QSO Configuration Audit Checklists

**Document ID**: SQSO-CAC-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the Functional Configuration Audit (FCA) and Physical Configuration Audit (PCA) checklists for SMART-QSO per NPR 7120.5 §3.4 and NASA-HDBK-4008.

## 2. Audit Schedule

### 2.1 Audit Timing

| Audit | Timing | Prerequisites |
|-------|--------|---------------|
| FCA | Pre-FRR | All tests complete |
| PCA | Pre-FRR | Flight unit built |
| Delta FCA | Pre-Launch | Any changes post-FRR |
| Delta PCA | Pre-Launch | Any rework post-FRR |

### 2.2 Audit Team

| Role | Responsibility |
|------|----------------|
| Audit Lead | Schedule, conduct, report |
| Systems Engineer | Technical verification |
| Software Lead | Software verification |
| QA Lead | Process compliance |
| CM Lead | Documentation status |

## 3. Functional Configuration Audit (FCA)

### 3.1 FCA Purpose

The FCA verifies that the system's actual performance matches the performance specified in requirements documentation.

### 3.2 FCA Entry Criteria

| Criterion | Verification | Status |
|-----------|--------------|--------|
| All requirements baselined | RTM review | [ ] |
| All verification complete | VCRM review | [ ] |
| Test reports available | Document check | [ ] |
| Anomalies resolved/waived | NCR review | [ ] |
| Software version baselined | CM check | [ ] |

### 3.3 FCA Checklist - System Requirements

| Req ID | Requirement | Test/Analysis | Pass/Fail | Evidence |
|--------|-------------|---------------|-----------|----------|
| SYS-001 | 1U form factor | Analysis | [ ] | |
| SYS-002 | Mass ≤2.0 kg | Test | [ ] | |
| SYS-003 | 2-year design life | Analysis | [ ] | |
| SYS-004 | Amateur band operation | Test | [ ] | |
| SYS-005 | AI beacon generation | Test | [ ] | |
| SYS-006 | Autonomous operation | Test | [ ] | |
| SYS-007 | Safe mode capability | Test | [ ] | |
| SYS-008 | Ground commanding | Test | [ ] | |
| SYS-009 | Telemetry downlink | Test | [ ] | |
| SYS-010 | Power positive | Analysis | [ ] | |
| | *Continue for all system requirements* | | | |

### 3.4 FCA Checklist - Software Requirements

| Req ID | Requirement | Verification | Pass/Fail | Evidence |
|--------|-------------|--------------|-----------|----------|
| SW-001 | Boot in <30s | Test | [ ] | |
| SW-002 | State machine | Test | [ ] | |
| SW-003 | Beacon generation | Test | [ ] | |
| SW-004 | Command processing | Test | [ ] | |
| SW-005 | Telemetry collection | Test | [ ] | |
| SW-006 | Power management | Test | [ ] | |
| SW-007 | Fault detection | Test | [ ] | |
| SW-008 | Watchdog operation | Test | [ ] | |
| SW-009 | AI integration | Test | [ ] | |
| SW-010 | Fallback operation | Test | [ ] | |
| | *Continue for all software requirements* | | | |

### 3.5 FCA Checklist - Interface Requirements

| Interface | ICD Version | Verified | Pass/Fail | Evidence |
|-----------|-------------|----------|-----------|----------|
| OBC-EPS | ICD-001 v1.0 | Test | [ ] | |
| OBC-RF | ICD-002 v1.0 | Test | [ ] | |
| OBC-Jetson | ICD-003 v1.0 | Test | [ ] | |
| OBC-ADCS | ICD-004 v1.0 | Test | [ ] | |
| RF-Antenna | ICD-005 v1.0 | Test | [ ] | |
| Ground-Space | ICD-006 v1.0 | Test | [ ] | |
| Deployer | ICD-007 v1.0 | Inspection | [ ] | |

### 3.6 FCA Checklist - Performance

| Parameter | Requirement | Measured | Pass/Fail |
|-----------|-------------|----------|-----------|
| Power (Safe) | ≤0.2 W | | [ ] |
| Power (Idle) | ≤0.4 W | | [ ] |
| Power (Active) | ≤0.8 W | | [ ] |
| Power (AI Burst) | ≤7 W | | [ ] |
| TX Power | 0.5-2 W | | [ ] |
| Data Rate | 1200/9600 bps | | [ ] |
| Beacon Period | 60-120 s | | [ ] |
| Boot Time | <30 s | | [ ] |
| Mode Transition | <5 s | | [ ] |
| Fault Response | <5 s | | [ ] |

### 3.7 FCA Discrepancy Log

| ID | Requirement | Discrepancy | Disposition | Status |
|----|-------------|-------------|-------------|--------|
| | | | | |
| | | | | |

### 3.8 FCA Closure

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

## 4. Physical Configuration Audit (PCA)

### 4.1 PCA Purpose

The PCA verifies that the as-built configuration matches the design documentation.

### 4.2 PCA Entry Criteria

| Criterion | Verification | Status |
|-----------|--------------|--------|
| Flight unit assembled | Inspection | [ ] |
| As-built list available | CM check | [ ] |
| Drawings released | CM check | [ ] |
| Parts list current | CM check | [ ] |
| Serial numbers recorded | Traveler check | [ ] |

### 4.3 PCA Checklist - Structure

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

### 4.4 PCA Checklist - EPS

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| EPS Board | DWG-EPS-001 | S/N: | [ ] | |
| Solar Cell 1 | SPEC-EPS-010 | Lot: | [ ] | |
| Solar Cell 2 | SPEC-EPS-010 | Lot: | [ ] | |
| Solar Cell 3 | SPEC-EPS-010 | Lot: | [ ] | |
| Solar Cell 4 | SPEC-EPS-010 | Lot: | [ ] | |
| Solar Cell 5 | SPEC-EPS-010 | Lot: | [ ] | |
| Solar Cell 6 | SPEC-EPS-010 | Lot: | [ ] | |
| Battery Cell 1 | SPEC-EPS-020 | S/N: | [ ] | |
| Battery Cell 2 | SPEC-EPS-020 | S/N: | [ ] | |

### 4.5 PCA Checklist - OBC

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| OBC Board | DWG-OBC-001 | S/N: | [ ] | |
| Apollo4 MCU | SPEC-OBC-010 | Lot: | [ ] | |
| FRAM | SPEC-OBC-020 | Lot: | [ ] | |
| External Flash | SPEC-OBC-030 | Lot: | [ ] | |
| Watchdog IC | SPEC-OBC-040 | Lot: | [ ] | |

### 4.6 PCA Checklist - RF

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| RF Board | DWG-RF-001 | S/N: | [ ] | |
| Si4463 | SPEC-RF-010 | Lot: | [ ] | |
| VHF PA | SPEC-RF-020 | S/N: | [ ] | |
| UHF LNA | SPEC-RF-030 | Lot: | [ ] | |
| VHF Antenna | DWG-RF-050 | S/N: | [ ] | |
| UHF Antenna | DWG-RF-051 | S/N: | [ ] | |

### 4.7 PCA Checklist - AI Payload

| Item | Drawing/Spec | As-Built | Match | Notes |
|------|--------------|----------|-------|-------|
| Jetson Orin Nano | SPEC-PL-001 | S/N: | [ ] | |
| Carrier Board | DWG-PL-001 | S/N: | [ ] | |
| Thermal Pad | SPEC-PL-010 | Lot: | [ ] | |
| Heatsink | DWG-PL-020 | P/N: | [ ] | |

### 4.8 PCA Checklist - Harness

| Harness | Drawing | Length | Connectors | Match |
|---------|---------|--------|------------|-------|
| EPS-OBC | DWG-HAR-001 | | | [ ] |
| OBC-RF | DWG-HAR-002 | | | [ ] |
| OBC-Jetson | DWG-HAR-003 | | | [ ] |
| OBC-ADCS | DWG-HAR-004 | | | [ ] |
| RF-Antenna | DWG-HAR-005 | | | [ ] |
| Solar-EPS | DWG-HAR-006 | | | [ ] |

### 4.9 PCA Checklist - Software

| Software | Version | Loaded | Verified | Match |
|----------|---------|--------|----------|-------|
| Flight Software | v1.0.0 | [ ] | CRC: | [ ] |
| Golden Image | v1.0.0 | [ ] | CRC: | [ ] |
| Payload Software | v1.0.0 | [ ] | CRC: | [ ] |
| TinyLM Model | v1.0 | [ ] | CRC: | [ ] |
| Configuration | v1.0 | [ ] | CRC: | [ ] |

### 4.10 PCA Checklist - Mass Properties

| Property | Requirement | Measured | Pass/Fail |
|----------|-------------|----------|-----------|
| Total Mass | ≤2.0 kg | | [ ] |
| CG-X | ±2 cm | | [ ] |
| CG-Y | ±2 cm | | [ ] |
| CG-Z | ±2 cm | | [ ] |

### 4.11 PCA Checklist - Documentation

| Document | Version | Current | Match |
|----------|---------|---------|-------|
| System Requirements | 1.0 | [ ] | [ ] |
| Software Requirements | 1.0 | [ ] | [ ] |
| Software Design | 1.0 | [ ] | [ ] |
| All ICDs | 1.0 | [ ] | [ ] |
| Test Reports | 1.0 | [ ] | [ ] |
| Parts List | 1.0 | [ ] | [ ] |
| As-Built List | 1.0 | [ ] | [ ] |

### 4.12 PCA Discrepancy Log

| ID | Item | Discrepancy | Disposition | Status |
|----|------|-------------|-------------|--------|
| | | | | |
| | | | | |

### 4.13 PCA Closure

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

## 5. Combined Audit Report

### 5.1 Report Template

```
CONFIGURATION AUDIT REPORT

Document ID: SQSO-CAR-001
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

## 6. Post-Audit Actions

### 6.1 Discrepancy Resolution

| Priority | Resolution Time | Authority |
|----------|-----------------|-----------|
| Critical | Before FRR | PM + Customer |
| Major | Before FRR | CCB |
| Minor | Before launch | Systems Engr |
| Editorial | Best effort | Doc owner |

### 6.2 Audit Closeout

| Action | Responsibility | Due |
|--------|----------------|-----|
| Discrepancies resolved | Owner | Pre-FRR |
| Documentation updated | CM | Pre-FRR |
| Final report issued | Audit Lead | FRR-7 days |
| Audit closed | PM | FRR |

## 7. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
