# SMART-QSO Flight Readiness Review Package

**Document ID**: SQSO-FRR-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Template (Pre-FRR). No FRR has been held

> **Template for a future review: not held.** No flight readiness review has been held, no flight unit exists, and
> nothing in this document records a result: every count, version, status and signature field is to be filled in at
> the review. For a rideshare through an integrator, the project's own flight readiness check is the **pre-ship review
> before delivery to the integrator** (`mission/SCHEDULE.md` §4: earliest 2028-05-19, delivery required by 2028-09-12,
> L − 50 days, for the 2028-11-01 planning launch); the launch-vehicle FRR is SpaceX's and the integrator's. Values to
> check against are in [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md); the current review record is
> [CDR_2026_DESIGN_REVIEW.md](CDR/CDR_2026_DESIGN_REVIEW.md) (an engineering assessment, not an independent board).
> Index of the review documents: [README.md](README.md).

## 1. Purpose

This document provides the Flight Readiness Review (FRR) package for the SMART-QSO mission per NPR 7120.5 requirements. FRR confirms readiness to proceed with launch operations.

## 2. Review Objectives

### 2.1 FRR Success Criteria

| Criterion | Requirement |
|-----------|-------------|
| Hardware ready | Flight unit tested and configured |
| Software ready | Flight software baselined |
| Ground ready | Ground systems operational |
| Team ready | Operations team trained |
| Documentation complete | All deliverables approved |
| Risks acceptable | No unacceptable open risks |

### 2.2 Entrance Criteria

- [ ] Independent CDR held (planned 2027-09-28) and its RIDs closed
- [ ] Every verification item of `docs/verification/VI_REGISTER.md` due before delivery passed
- [ ] Environmental testing complete
- [ ] Flight software delivered
- [ ] Operations team certified
- [ ] Ground station validated

## 3. System Readiness

### 3.1 Flight Hardware Status

| Subsystem | Status | S/N | Notes |
|-----------|--------|-----|-------|
| Structure | [ ] Ready | | |
| EPS | [ ] Ready | | |
| Battery | [ ] Ready | | |
| OBC | [ ] Ready | | |
| RF System | [ ] Ready | | |
| Antenna | [ ] Ready | | |
| AI Payload | [ ] Ready | | |
| ADCS Sensors | [ ] Ready | | |
| Harness | [ ] Ready | | |

### 3.2 Flight Software Status

| Item | Version | Status |
|------|---------|--------|
| Flight Software (`software/rust`, Embassy firmware) | | [ ] Baselined |
| Payload Software (`software/payload`) | | [ ] Baselined |
| Configuration (keys, NVM parameters) | | [ ] Baselined |

### 3.3 Ground System Status

| System | Status | Notes |
|--------|--------|-------|
| Primary Ground Station | [ ] Ready | |
| Backup Ground Station | [ ] Ready | |
| Mission Operations | [ ] Ready | |
| Tracking (TLE) | [ ] Ready | |
| GNSS ephemeris with covariance to the integrator within 24 h of first contact (SpaceX UG v11 §9.5.7.2) | [ ] Ready | |
| Second telecommand station, ≥ 1,000 km from the first (BASELINE §1) | [ ] Ready | |

## 4. Verification Summary

### 4.1 Requirements Verification

Totals from the generated traceability matrix (`docs/verification/RTM.csv`) at the time of the review.

| Category | Total | Verified | Open | Status |
|----------|-------|----------|------|--------|
| Functional | | | | [ ] |
| Performance | | | | [ ] |
| Interface | | | | [ ] |
| Environmental | | | | [ ] |
| Safety | | | | [ ] |
| **Total** | | | | |

### 4.2 Test Summary

| Test Campaign | Tests | Pass | Fail | Waived |
|---------------|-------|------|------|--------|
| Unit Tests | | | | |
| Flatsat Tests | | | | |
| Integration Tests | | | | |
| Environmental (protoflight campaign) | | | | |
| End-to-End | | | | |

### 4.3 Open Test Failures

| Test | Failure | Disposition |
|------|---------|-------------|
| | | |

## 5. Configuration Status

### 5.1 Configuration Audit Results

| Audit | Date | Result | Findings |
|-------|------|--------|----------|
| FCA | | [ ] Pass | |
| PCA | | [ ] Pass | |

### 5.2 As-Built Configuration

| Item | Drawing/Spec | As-Built | Match |
|------|--------------|----------|-------|
| Structure | SQSO-MECH-CAD-001, SQSO-STR-001 | | [ ] |
| EPS | SQSO-EPS-BRD-001 | | [ ] |
| OBC | SQSO-OBC-BRD-001 | | [ ] |
| RF | SQSO-RF-BRD-001 | | [ ] |
| Jetson carrier | SQSO-JET-BRD-001 | | [ ] |
| Interconnect | SQSO-ICX-001 | | [ ] |
| Software | SQSO-SVD-001 | | [ ] |

## 6. Safety Status

### 6.1 Hazard Closure

System hazard reports HR-01 to HR-16 (`docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md`, SQSO-SDP-002) and software
hazards HZ-001 to HZ-006 (`docs/safety/SAFETY_DATA_PACKAGE.md`, SQSO-SDP-001); one row each at the review.

| Hazard | Status | Verification |
|--------|--------|--------------|
| HR-01 … HR-16 | [ ] Closed | |
| HZ-001 Unintended RF Emission | [ ] Closed | |
| HZ-002 Battery Over-discharge | [ ] Closed | |
| HZ-003 Thermal Damage | [ ] Closed | |
| HZ-004 Software Hang | [ ] Closed | |
| HZ-005 Memory Corruption | [ ] Closed | |
| HZ-006 Command Injection | [ ] Closed | |

### 6.2 Range Safety

| Item | Status |
|------|--------|
| Range safety approval | [ ] Obtained |
| ICD compliance | [ ] Verified |
| Battery certification | [ ] Complete |

## 7. Risk Status

### 7.1 Open Risks

From `mission/RISK_REGISTER.md` at the time of the review.

| Risk ID | Description | Likelihood | Consequence | Mitigation | Status |
|---------|-------------|------------|-------------|------------|--------|
| | | | | | |

### 7.2 Risk Acceptance

| Risk Level | Accepted | Not Accepted |
|------------|----------|--------------|
| High | | |
| Medium | | |
| Low | | |

## 8. Operations Readiness

### 8.1 Team Certification

| Position | Name | Certified | Date |
|----------|------|-----------|------|
| Mission Director | | [ ] | |
| Flight Controller 1 | | [ ] | |
| Flight Controller 2 | | [ ] | |
| Ground Controller | | [ ] | |
| Analyst | | [ ] | |

### 8.2 Procedure Validation

| Procedure | Validated | Date |
|-----------|-----------|------|
| Pass Operations | [ ] | |
| Commanding | [ ] | |
| Contingency | [ ] | |
| Emergency | [ ] | |

### 8.3 Simulation Results

| Simulation | Date | Result |
|------------|------|--------|
| Nominal Ops | | [ ] Pass |
| Anomaly Response | | [ ] Pass |
| End-to-End | | [ ] Pass |

## 9. Documentation Status

### 9.1 Required Documents

| Document | Version | Status |
|----------|---------|--------|
| Mission Operations Plan | | [ ] Approved |
| Contingency Procedures | | [ ] Approved |
| Command Dictionary | | [ ] Approved |
| Telemetry Dictionary | | [ ] Approved |
| Training Records | | [ ] Complete |
| Software Version Description | | [ ] Approved |
| Known Issues | | [ ] Approved |

### 9.2 Waivers and Deviations

| ID | Description | Approval | Status |
|----|-------------|----------|--------|
| | | | |

## 10. Launch Campaign Readiness

### 10.1 Integrator Activities

Planned sequence: `docs/operations/MISSION_TIMELINE.md` §3 and `docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md` §9.

| Activity | Date | Status |
|----------|------|--------|
| Delivery to the integrator (≈ L − 50 d) | | [ ] Complete |
| Receiving inspection | | [ ] Complete |
| Final functional | | [ ] Complete |
| Final charge to 60 % ± 5 % SOC (≈ L − 47 d) | | [ ] Complete |
| Key load, `ARM_FLIGHT`, RBF removed, integration into the EXOpod Nova (≈ L − 47 d) | | [ ] Complete |

### 10.2 Launch Constraints

| Constraint | Requirement | Status |
|------------|-------------|--------|
| Battery SOC at integration | 60 % ± 5 % (BASELINE §4; separation SOC ≥ 40 %) | [ ] Met |
| Storage drain, all inhibits open | ≤ 20 µA (SQSO-LVR-001 LVR-071) | [ ] Met |
| Configuration | Flight | [ ] Verified |
| Remove-before-flight | Removed at integration | [ ] Verified |

## 11. FRR Checklist

### 11.1 Technical Readiness

- [ ] All requirements verified
- [ ] All tests complete and passed
- [ ] Configuration audits complete
- [ ] All hazards closed
- [ ] Software baselined and delivered
- [ ] Ground systems validated

### 11.2 Programmatic Readiness

- [ ] Schedule supports launch date
- [ ] Team trained and certified
- [ ] Documentation complete
- [ ] Risks acceptable
- [ ] Stakeholder concurrence

### 11.3 Mission Readiness

- [ ] Operations procedures validated
- [ ] Contingency procedures tested
- [ ] Communication links verified
- [ ] Tracking arrangements confirmed
- [ ] Regulatory compliance verified

## 12. FRR Board

### 12.1 Board Composition

| Role | Name | Organization |
|------|------|--------------|
| Chair | | |
| Project Manager | | |
| Systems Engineer | | |
| Software Lead | | |
| Safety Engineer | | |
| Quality Assurance | | |
| Mission Operations | | |

### 12.2 Board Decision

```
FLIGHT READINESS REVIEW DECISION

Date: ___________
Configuration: ___________

The Flight Readiness Review Board has reviewed the
SMART-QSO mission readiness and determines:

[ ] GO FOR LAUNCH
    - All success criteria met
    - No open critical items
    - Risks acceptable

[ ] GO WITH CONDITIONS
    - Conditions: ___________
    - Resolution required by: ___________

[ ] NO-GO
    - Rationale: ___________
    - Required actions: ___________

Signatures:

Chair: _______________ Date: _______
PM: __________________ Date: _______
SE: __________________ Date: _______
Safety: ______________ Date: _______
```

## 13. Post-FRR Actions

### 13.1 Required Actions

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| | | | |

### 13.2 Launch Authorization

Upon successful FRR closure:

| Authorization | Authority | Date |
|---------------|-----------|------|
| Launch Proceed | Project Manager | |
| Range Approval | Range Safety | |
| Regulatory | FCC/IARU | |

## 14. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
| 1.1 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): banner (template, not held; milestone: pre-ship review, `mission/SCHEDULE.md`); pre-filled results removed (counts, versions 1.0.0, "None" failures, risks and waivers); battery SOC 60 % ± 5 % at integration (was > 95 %; BASELINE §4); integrator activities per MISSION_TIMELINE; as-built rows cite the build-to designs (SQSO-EPS-001, -OBC-001 and -RF-001 do not exist); hazards HR-01 to HR-16 and HZ-001 to HZ-006; GNSS ephemeris and second telecommand station rows |
