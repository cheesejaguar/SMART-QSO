# SMART-QSO Mission Operations Review Package

**Document ID**: SQSO-MOR-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Template (Pre-MOR). No MOR has been held

> **Template for a future review: not held.** No mission operations review has been held, no operator has been
> trained or certified, and nothing in this document records a result; every status, version and signature field is
> to be filled in at the review. `mission/SCHEDULE.md` has no MOR task: the review belongs before the **pre-ship
> review** (earliest 2028-05-19), after the FM day-in-the-life (FM_FUNC, 2028-02-01 to 2028-02-22). Operations
> facts to check against: two designated telecommand stations at least 1,000 km apart, each able to command alone
> (BASELINE §1), commissioning of about 14 days (`docs/operations/COMMISSIONING_PLAN.md`), mission rules in
> `docs/operations/MISSION_RULES.md` and contingency procedures in `docs/operations/CONTINGENCY_PROCEDURES.md`. The
> current review record is [CDR_2026_DESIGN_REVIEW.md](CDR/CDR_2026_DESIGN_REVIEW.md). Index of the review documents:
> [README.md](README.md).

## 1. Purpose

This document provides the Mission Operations Review (MOR) package for SMART-QSO per NASA Ames SmallSat heritage practices. MOR confirms the operations team and systems are ready to support the mission.

## 2. Review Objectives

### 2.1 MOR Success Criteria

| Criterion | Requirement |
|-----------|-------------|
| Ops team trained | All operators certified |
| Procedures validated | All procedures tested |
| Ground systems ready | Primary and backup operational |
| Simulations complete | DITL and contingency runs done |
| Documentation complete | All ops docs approved |
| Lessons incorporated | Previous mission lessons applied |

### 2.2 Entrance Criteria

- [ ] Independent CDR held (planned 2027-09-28)
- [ ] Ground system development complete
- [ ] Procedures written
- [ ] Team identified
- [ ] Training program defined

## 3. Operations Team Certification

### 3.1 Position Certifications

| Position | Name | Written | Practical | Sim | Certified |
|----------|------|---------|-----------|-----|-----------|
| Mission Director | | [ ] | [ ] | [ ] | [ ] |
| Flight Controller 1 | | [ ] | [ ] | [ ] | [ ] |
| Flight Controller 2 | | [ ] | [ ] | [ ] | [ ] |
| Ground Controller 1 | | [ ] | [ ] | [ ] | [ ] |
| Ground Controller 2 | | [ ] | [ ] | [ ] | [ ] |
| Systems Analyst | | [ ] | [ ] | [ ] | [ ] |

### 3.2 Training Completion

| Training Module | Required For | Complete |
|-----------------|--------------|----------|
| Mission Overview | All | [ ] |
| Ground Systems | Ground, FC | [ ] |
| Commanding | FC, MD | [ ] |
| Telemetry Analysis | FC, Analyst | [ ] |
| Contingency Response | All | [ ] |
| Shift Operations | All | [ ] |
| Safety | All | [ ] |

### 3.3 Proficiency Requirements

| Skill | Demonstrated By |
|-------|-----------------|
| Command execution | Sim exercise |
| Anomaly recognition | Fault injection |
| Procedure compliance | Observed operations |
| Handoff execution | DITL |
| Communication | All exercises |

## 4. Procedure Validation

### 4.1 Nominal Procedures

| Procedure | Version | Tested | Issues | Status |
|-----------|---------|--------|--------|--------|
| Pre-pass preparation | | [ ] | | [ ] Approved |
| Pass operations | | [ ] | | [ ] Approved |
| Commanding | | [ ] | | [ ] Approved |
| Telemetry monitoring | | [ ] | | [ ] Approved |
| Post-pass activities | | [ ] | | [ ] Approved |
| Shift handoff | | [ ] | | [ ] Approved |
| Daily status | | [ ] | | [ ] Approved |

### 4.2 Contingency Procedures

Procedure names as in `docs/operations/CONTINGENCY_PROCEDURES.md`.

| Procedure | Version | Tested | Status |
|-----------|---------|--------|--------|
| CP-001 Loss of Signal (LOS) | | [ ] | [ ] Approved |
| CP-002 Unexpected Safe Mode Entry | | [ ] | [ ] Approved |
| CP-003 Battery Undervoltage | | [ ] | [ ] Approved |
| CP-004 Thermal Limit Exceeded | | [ ] | [ ] Approved |
| CP-005 Watchdog Reset | | [ ] | [ ] Approved |
| CP-006 AI Payload Failure | | [ ] | [ ] Approved |
| CP-007 Command Rejection | | [ ] | [ ] Approved |
| CP-008 Collision Avoidance Maneuver | | [ ] | [ ] Approved |

### 4.3 Procedure Validation Summary

```
PROCEDURE VALIDATION SUMMARY

Nominal Procedures:
- Total: 7
- Tested: ___
- Approved: ___
- Open issues: ___

Contingency Procedures:
- Total: 8
- Tested: ___
- Approved: ___
- Open issues: ___

Validation Status: [ ] COMPLETE  [ ] IN PROGRESS
```

## 5. Ground System Validation

### 5.1 Telecommand Stations (each of the ≥ 2 designated stations)

| Test | Date | Result |
|------|------|--------|
| RF link test | | [ ] Pass |
| Command uplink | | [ ] Pass |
| Telemetry downlink | | [ ] Pass |
| Beacon decode | | [ ] Pass |
| Tracking accuracy | | [ ] Pass |
| Backup power | | [ ] Pass |

### 5.2 Mission Operations Center

| System | Tested | Result |
|--------|--------|--------|
| Command system | [ ] | [ ] Pass |
| Telemetry display | [ ] | [ ] Pass |
| Archive system | [ ] | [ ] Pass |
| Voice loops | [ ] | [ ] Pass |
| Status displays | [ ] | [ ] Pass |
| Network connectivity | [ ] | [ ] Pass |

### 5.3 Backup Systems

| Backup | Tested | Switchover Time |
|--------|--------|-----------------|
| Backup ground station | [ ] | |
| Backup workstation | [ ] | |
| Backup network | [ ] | |
| Backup power | [ ] | |

## 6. Simulation Results

### 6.1 Day-in-the-Life (DITL)

| Run | Date | Duration | Result |
|-----|------|----------|--------|
| DITL-1 | | 24 hrs | [ ] Pass |
| DITL-2 (if needed) | | | [ ] N/A |

**DITL Summary**:
- Passes executed: ___
- Anomalies injected: ___
- All procedures executed: [ ] Yes
- Issues found: ___
- Issues resolved: ___

### 6.2 Contingency Simulations

| Scenario | Tested | Response Time | Result |
|----------|--------|---------------|--------|
| Safe mode recovery | [ ] | | [ ] Pass |
| Comm loss | [ ] | | [ ] Pass |
| Low power | [ ] | | [ ] Pass |
| Ground failure | [ ] | | [ ] Pass |

### 6.3 End-to-End Tests

| Test | Date | Result |
|------|------|--------|
| E2E-001 Nominal pass | | [ ] Pass |
| E2E-002 Command chain | | [ ] Pass |
| E2E-003 AI integration | | [ ] Pass |
| E2E-004 Fault injection | | [ ] Pass |
| E2E-005 Extended duration | | [ ] Pass |
| E2E-006 DITL | | [ ] Pass |

## 7. Mission Rules Validation

### 7.1 Rules Reviewed

| Category | Rules | Reviewed |
|----------|-------|----------|
| Mandatory Rules | MR-001 to MR-005 | [ ] |
| Critical Flight Constraints | FC-001 to FC-005 | [ ] |
| Standard Mission Rules | MR-101 to MR-107 | [ ] |
| Ground Rules | GR-001 to GR-004 | [ ] |

### 7.2 Rule Scenarios

| Rule | Scenario Tested | Result |
|------|-----------------|--------|
| MR-001 | RF transmission authorization | [ ] Pass |
| MR-005 | Emergency stop authority (TX-OFF) | [ ] Pass |
| FC-001 | Battery state-of-charge limits | [ ] Pass |
| FC-002 | Temperature limits | [ ] Pass |

## 8. Lessons Learned Application

### 8.1 Heritage Mission Lessons

| Mission | Lesson | Applied To |
|---------|--------|------------|
| GeneSat-1 | Early comm challenges | Extended search procedures |
| PharmaSat | Power management | Conservative margins |
| O/OREOS | Thermal variations | Enhanced monitoring |
| EcAMSat | Ops efficiency | Streamlined procedures |

### 8.2 SMART-QSO Development Lessons

To be filled from the flatsat, end-to-end and day-in-the-life campaigns once they have run (none has).

| Phase | Lesson | Implementation |
|-------|--------|----------------|
| | | |

## 9. Staffing Plan

### 9.1 Phase Staffing

| Phase | Duration | Shifts | Personnel/Shift |
|-------|----------|--------|-----------------|
| Launch | 1 day | 3 | Full team |
| LEOP | 3 days | 3 | Full team |
| Commissioning | 14 days | 2 | Core team |
| Normal ops | Ongoing | 1 | Minimal |

### 9.2 On-Call Support

| Position | On-Call | Response Time |
|----------|---------|---------------|
| Mission Director | Yes | 30 min |
| Systems Engineer | Yes | 1 hour |
| Software Engineer | Yes | 1 hour |

### 9.3 Schedule Coverage

| Time Period | Coverage |
|-------------|----------|
| Weekday passes | All covered |
| Weekend passes | On-call |
| Holidays | Reduced |
| Emergencies | Full response |

## 10. Interface Verification

### 10.1 External Interfaces

| Interface | Partner | Verified |
|-----------|---------|----------|
| Space-Track | 18 SDS | [ ] |
| SatNOGS | Community | [ ] |
| AMSAT | Organization | [ ] |
| Launch integrator | Exolaunch (planned; contract by 2027-06-30, BASELINE §3) | [ ] |

### 10.2 Internal Interfaces

| Interface | Systems | Verified |
|-----------|---------|----------|
| Ground-MOC | Ground to ops | [ ] |
| MOC-Archive | Ops to storage | [ ] |
| Alert system | Ops to team | [ ] |

## 11. MOR Checklist

### 11.1 Team Readiness

- [ ] All positions certified
- [ ] Training complete
- [ ] Simulations passed
- [ ] Backup personnel identified

### 11.2 Procedure Readiness

- [ ] All procedures validated
- [ ] Issues resolved
- [ ] Procedures approved
- [ ] Procedures at stations

### 11.3 System Readiness

- [ ] Ground systems validated
- [ ] Backup systems tested
- [ ] Tools functional
- [ ] Data flow verified

### 11.4 Documentation

- [ ] Ops plan approved
- [ ] Procedures distributed
- [ ] Contact lists current
- [ ] Mission rules posted

## 12. MOR Board Decision

```
MISSION OPERATIONS REVIEW DECISION

Date: ___________

The Mission Operations Review Board has reviewed
operations readiness and determines:

[ ] OPERATIONS READY
    - All readiness criteria met
    - Team certified for mission support
    - Proceed to FRR

[ ] CONDITIONAL READY
    - Conditions: ___________
    - Resolution required by: ___________

[ ] NOT READY
    - Rationale: ___________
    - Required actions: ___________

Signatures:

Mission Director: _________ Date: _____
Ops Lead: ________________ Date: _____
Training Lead: ____________ Date: _____
QA: _____________________ Date: _____
```

## 13. Post-MOR Activities

### 13.1 Actions to FRR

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| Complete any training gaps | Training | FRR-7d | [ ] |
| Finalize ops schedule | Ops Lead | FRR-5d | [ ] |
| Staff on-call roster | MD | FRR-3d | [ ] |

### 13.2 Handoff to Flight Operations

| Item | Status |
|------|--------|
| Ops plan delivered | [ ] |
| Team assigned | [ ] |
| Schedule finalized | [ ] |
| Ready for FRR | [ ] |

## 14. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
| 1.1 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): banner (template, not held; milestone: before the pre-ship review); development "lessons" from flatsat, E2E and DITL campaigns that have not run removed; contingency procedure names and mission-rule ranges as in CONTINGENCY_PROCEDURES and MISSION_RULES (CT and GC rules do not exist); telecommand stations (≥ 2, BASELINE §1); launch integrator named (was an open placeholder); pre-filled versions 1.0 removed |
