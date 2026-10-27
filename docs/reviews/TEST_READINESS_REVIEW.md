# SMART-QSO Test Readiness Review Package

**Document ID**: SQSO-TRR-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Template (Pre-TRR). No TRR has been held

> **Template for a future review: not held.** No test readiness review has been held, no flatsat or engineering model
> exists, and nothing in this document records a result; every status, version and signature field is to be filled in
> at the review. It belongs to the **EM integration and flatsat** of `mission/SCHEDULE.md` (EM_INT, 2027-06-01 to
> 2027-07-27, procedure TP-FLATSAT-001), after the independent PDR (2027-01-12) and before the independent CDR
> (2027-09-28); the same form serves later test campaigns (EM tests, FM integration). Design values are in
> [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md); the current review record is
> [CDR_2026_DESIGN_REVIEW.md](CDR/CDR_2026_DESIGN_REVIEW.md). Index of the review documents: [README.md](README.md).

## 1. Purpose

This document provides the Test Readiness Review (TRR) package for the SMART-QSO mission per NPR 7123.1 requirements. TRR confirms readiness to proceed with system-level testing.

## 2. Review Objectives

### 2.1 TRR Success Criteria

| Criterion | Requirement |
|-----------|-------------|
| Test articles ready | Flatsat/flight unit complete |
| Test procedures approved | All TPs reviewed and signed |
| Test facilities ready | Lab/chamber available |
| Test team trained | Operators certified |
| Support equipment ready | GSE calibrated and ready |
| Software testable | Test version loaded |

### 2.2 Entrance Criteria

- [ ] Independent PDR held (planned 2027-01-12) and its RIDs dispositioned
- [ ] Verification items due before the campaign passed (`docs/verification/VI_REGISTER.md`)
- [ ] Test articles assembled
- [ ] Test procedures approved
- [ ] Facilities scheduled
- [ ] Team assigned

## 3. Test Article Status

### 3.1 Flatsat Configuration

| Component | P/N | S/N | Status |
|-----------|-----|-----|--------|
| OBC board EM (STM32L4R5ZI) | SQSO-OBC-BRD-001 | | [ ] Ready |
| Jetson Orin Nano 8 GB module 900-13767-0030-000 on the carrier EM | SQSO-JET-BRD-001 | | [ ] Ready |
| EPS board EM, or battery and array simulators | SQSO-EPS-BRD-001 | | [ ] Ready |
| RF board EM (TNC, receiver, transmitter) | SQSO-RF-BRD-001 | | [ ] Ready |
| Debug Interface | J-Link Plus | | [ ] Ready |
| Power Supply | E36312A | | [ ] Ready |

### 3.2 Software Configuration

| Software | Version | Loaded | Verified |
|----------|---------|--------|----------|
| Flight Software (Git commit) | | [ ] | [ ] |
| Payload Software | | [ ] | [ ] |
| Ground Software | | [ ] | [ ] |
| Test Scripts | | [ ] | [ ] |

### 3.3 Configuration Control

| Item | Baseline | Deviations |
|------|----------|------------|
| Hardware | EM layout release | |
| Software | | |
| Procedures | | |

## 4. Test Procedure Status

### 4.1 Procedure Approval

TP-001 to TP-007 are this template's placeholder numbers. The written procedures for the flatsat are
`docs/test_procedures/TP-FLATSAT-001_Software_Development.md` and the TP-INT, TP-OBC, TP-RF and TP-EPS procedures in
`docs/test_procedures/`; list those at the review.

| Procedure | Version | Author | Reviewed | Approved |
|-----------|---------|--------|----------|----------|
| TP-001 Boot Test | 1.0 | | [ ] | [ ] |
| TP-002 Mode Transition | 1.0 | | [ ] | [ ] |
| TP-003 Telemetry | 1.0 | | [ ] | [ ] |
| TP-004 Beacon | 1.0 | | [ ] | [ ] |
| TP-005 Fault Response | 1.0 | | [ ] | [ ] |
| TP-006 Power Mode | 1.0 | | [ ] | [ ] |
| TP-007 Extended Duration | 1.0 | | [ ] | [ ] |

### 4.2 Procedure Dry Runs

| Procedure | Dry Run Date | Issues Found | Resolved |
|-----------|--------------|--------------|----------|
| TP-001 | | | [ ] |
| TP-002 | | | [ ] |
| TP-003 | | | [ ] |
| TP-004 | | | [ ] |
| TP-005 | | | [ ] |
| TP-006 | | | [ ] |
| TP-007 | | | [ ] |

## 5. Test Facilities

### 5.1 Facility Availability

| Facility | Required Dates | Reserved | Status |
|----------|----------------|----------|--------|
| Integration Lab | Week 1-2 | [ ] | [ ] Ready |
| Thermal Chamber | Week 3 | [ ] | [ ] Ready |
| EMC Chamber | Week 4 | [ ] | [ ] Ready |
| Clean Room | Week 5 | [ ] | [ ] Ready |

### 5.2 Facility Requirements

| Facility | Requirement | Verified |
|----------|-------------|----------|
| Lab | ESD controlled | [ ] |
| Lab | Network access | [ ] |
| Thermal | -20 to +60°C | [ ] |
| EMC | 145-438 MHz quiet | [ ] |

## 6. Ground Support Equipment

### 6.1 GSE List

| Equipment | P/N | Calibration Due | Status |
|-----------|-----|-----------------|--------|
| Power Supply | E36312A | | [ ] Current |
| Oscilloscope | | | [ ] Current |
| Logic Analyzer | | | [ ] Current |
| Spectrum Analyzer | | | [ ] Current |
| Multimeter | | | [ ] Current |

### 6.2 GSE Calibration Status

| Equipment | Last Cal | Next Cal | In Tolerance |
|-----------|----------|----------|--------------|
| Power Supply | | | [ ] |
| Oscilloscope | | | [ ] |
| Multimeter | | | [ ] |

## 7. Test Team

### 7.1 Team Assignment

| Role | Name | Trained | Available |
|------|------|---------|-----------|
| Test Director | | [ ] | [ ] |
| Test Conductor | | [ ] | [ ] |
| Software Engineer | | [ ] | [ ] |
| Hardware Engineer | | [ ] | [ ] |
| QA Representative | | [ ] | [ ] |

### 7.2 Training Status

| Training | Required For | Completed |
|----------|--------------|-----------|
| ESD Awareness | All | [ ] |
| Test Procedures | Test Conductor | [ ] |
| Software Debug | SW Engineer | [ ] |
| Safety Briefing | All | [ ] |

## 8. Risk Assessment

### 8.1 Test Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Hardware damage | Low | High | ESD controls, procedures |
| Schedule slip | Medium | Medium | Margin in schedule |
| Test failure | Medium | Low | Retest capability |

### 8.2 Constraints

| Constraint | Impact | Mitigation |
|------------|--------|------------|
| Facility availability | Schedule | Book early |
| Staff availability | Coverage | Cross-training |
| Equipment failures | Delay | Spares available |

## 9. Test Schedule

### 9.1 Test Campaign Timeline

| Week | Activity | Duration |
|------|----------|----------|
| 1 | Flatsat assembly | 3 days |
| 1-2 | TP-001 to TP-004 | 5 days |
| 2 | TP-005, TP-006 | 3 days |
| 2-3 | TP-007 start | 7 days |
| 3 | TP-007 complete | - |
| 4 | Data review | 2 days |
| 4 | Test report | 3 days |

### 9.2 Key Milestones

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| TRR Complete | | [ ] |
| Flatsat Ready | | [ ] |
| Testing Start | | [ ] |
| Testing Complete | | [ ] |
| Test Report | | [ ] |

## 10. Success Criteria

### 10.1 Test Success Criteria

| Test | Success Criteria |
|------|------------------|
| TP-001 | Boot < 5s, all init OK |
| TP-002 | All transitions per spec |
| TP-003 | All channels valid |
| TP-004 | AX.25 valid; 60 s ACTIVE, 120 s IDLE, 180 s SAFE (BASELINE §8) |
| TP-005 | All faults handled |
| TP-006 | All modes within power budget |
| TP-007 | 7 days stable, <0.01% errors |

### 10.2 Campaign Success Criteria

| Criterion | Requirement |
|-----------|-------------|
| All tests executed | 100% |
| All tests passed | 100% (or waived) |
| No critical anomalies | 0 open |
| Test report complete | Approved |

## 11. TRR Checklist

### 11.1 Hardware Readiness

- [ ] Test articles complete
- [ ] Configuration documented
- [ ] GSE calibrated
- [ ] Facilities reserved
- [ ] Spares available

### 11.2 Software Readiness

- [ ] Test version released
- [ ] Test scripts verified
- [ ] Debug tools ready
- [ ] Data collection ready

### 11.3 Procedure Readiness

- [ ] All procedures approved
- [ ] Dry runs complete
- [ ] Issues resolved
- [ ] Forms prepared

### 11.4 Team Readiness

- [ ] Roles assigned
- [ ] Training complete
- [ ] Schedule confirmed
- [ ] Contingency plans

## 12. TRR Board

### 12.1 Board Composition

| Role | Name |
|------|------|
| Chair | |
| Project Manager | |
| Test Lead | |
| Software Lead | |
| Quality Assurance | |

### 12.2 Board Decision

```
TEST READINESS REVIEW DECISION

Date: ___________
Test Campaign: Flatsat Testing

The Test Readiness Review Board has reviewed the
test readiness and determines:

[ ] GO FOR TEST
    - All readiness criteria met
    - Proceed with test campaign

[ ] GO WITH CONDITIONS
    - Conditions: ___________
    - Must resolve before: ___________

[ ] NO-GO
    - Rationale: ___________
    - Required actions: ___________

Chair: _______________ Date: _______
PM: __________________ Date: _______
Test Lead: ____________ Date: _______
QA: __________________ Date: _______
```

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
| 1.1 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): banner (template, not held; milestone: EM integration and flatsat, between the independent PDR and CDR); entrance criteria after the PDR (was "CDR complete"); test articles are the EM boards with the STM32L4R5ZI and the Jetson module 900-13767-0030-000 (were the Apollo4 AMA4B2KK-KBR and a Jetson developer kit); pre-filled versions and "None" deviations removed; beacon intervals per mode; §4.1 notes that TP-001 to TP-007 are placeholders |
