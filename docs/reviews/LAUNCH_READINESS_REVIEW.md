# SMART-QSO Launch Readiness Review Package

**Document ID**: SQSO-LRR-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Template (Pre-LRR). No LRR has been held

> **Template for a future review: not held.** No launch readiness review has been held and nothing in this document
> records a result; every status, version and signature field is to be filled in at the review. It belongs to the
> **launch** milestone of `mission/SCHEDULE.md` (planning launch 2028-11-01, the last Transporter-class rideshare of
> 2028, Falcon 9 from Vandenberg SLC-4E through Exolaunch in an EXOpod Nova; BASELINE §3). The spacecraft is sealed in
> the dispenser from about L − 47 days with no access or charging (BASELINE §4), so the hardware rows of §4 record the
> integration at the integrator, not checks made at the review. Values to check against are in
> [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md); the current review record is
> [CDR_2026_DESIGN_REVIEW.md](CDR/CDR_2026_DESIGN_REVIEW.md); the planned event sequence is
> `docs/operations/MISSION_TIMELINE.md`. Index of the review documents: [README.md](README.md).

## 1. Purpose

This document provides the Launch Readiness Review (LRR) package for SMART-QSO per NASA Ames SmallSat heritage practices. LRR is the final review confirming readiness for launch day operations.

## 2. Review Timing

### 2.1 LRR Schedule

| Event | Timing |
|-------|--------|
| LRR | L-7 days |
| LRR Actions Close | L-3 days |
| Launch Commit | L-1 day |
| Launch | L+0 |

### 2.2 LRR vs FRR Distinction

| Review | Focus | Timing |
|--------|-------|--------|
| FRR | System ready for launch campaign | Pre-ship |
| LRR | Ready for this specific launch | L-7 days |

## 3. LRR Success Criteria

### 3.1 Criteria Summary

| Criterion | Requirement |
|-----------|-------------|
| FRR complete | All FRR actions closed |
| Spacecraft integrated | In the EXOpod Nova since ≈ L − 47 d |
| Ground systems operational | Primary and backup ready |
| Team in position | All stations staffed |
| No open waivers | All approved or N/A |
| Weather acceptable | Per launch provider |

### 3.2 Entrance Criteria

- [ ] FRR complete and closed
- [ ] Spacecraft delivered to the integrator and received (≈ L − 50 d)
- [ ] Dispenser integration complete (≈ L − 47 d)
- [ ] Final functional test passed before integration
- [ ] Ground stations verified
- [ ] Ops team on station

## 4. Spacecraft Status (recorded at the integrator, ≈ L − 50 d to L − 47 d)

### 4.1 Pre-Integration Status

| Check | Date | Status |
|-------|------|--------|
| Receiving inspection | | [ ] Complete |
| Visual inspection | | [ ] Pass |
| Continuity check | | [ ] Pass |
| Battery SOC after the final charge | | [ ] 60 % ± 5 % (BASELINE §4) |

### 4.2 Deployer Integration

| Activity | Date | Status |
|----------|------|--------|
| Fit check | | [ ] Complete |
| Integration | | [ ] Complete |
| Deployment switch check | | [ ] Verified |
| Final mass properties | | [ ] Recorded |

### 4.3 Final Configuration

| Item | Configuration | Verified |
|------|---------------|----------|
| Flight software | (version) | [ ] |
| Keys loaded; `ARM_FLIGHT` the last umbilical action | Done | [ ] |
| State | Off: deployment switches open the battery path | [ ] |
| Antennas and wings | Stowed | [ ] |
| RBF pin | Removed at integration | [ ] |
| Deployment switches | Actuated (held by the dispenser) | [ ] |
| Access panels | Closed | [ ] |

## 5. Ground System Readiness

### 5.1 Primary Ground Station

| System | Status | Verified |
|--------|--------|----------|
| Antenna | Operational | [ ] |
| Radio | Configured | [ ] |
| TNC | Tested | [ ] |
| Decoder | Running | [ ] |
| Network | Connected | [ ] |
| Backup power | Ready | [ ] |

### 5.2 Mission Operations Center

| System | Status | Verified |
|--------|--------|----------|
| Ops workstations | Ready | [ ] |
| Displays | Configured | [ ] |
| Voice loops | Tested | [ ] |
| Logging | Enabled | [ ] |
| Backup systems | Ready | [ ] |

### 5.3 Tracking Readiness

| Item | Status |
|------|--------|
| TLE source identified | [ ] Space-Track |
| Update process defined | [ ] Yes |
| Search plan ready | [ ] Yes |
| Pass predictions available | [ ] After TLE |
| GNSS ephemeris with covariance within 24 h of first contact (SpaceX UG v11 §9.5.7.2) | [ ] Pipeline ready |

## 6. Team Readiness

### 6.1 Launch Day Stations

| Position | Primary | Backup | Status |
|----------|---------|--------|--------|
| Mission Director | | | [ ] Ready |
| Flight Controller | | | [ ] Ready |
| Ground Controller | | | [ ] Ready |
| Systems Engineer | | | [ ] Ready |

### 6.2 Contact Information

| Position | Phone | Email | Verified |
|----------|-------|-------|----------|
| Mission Director | | | [ ] |
| Flight Controller | | | [ ] |
| Ground Controller | | | [ ] |
| Project Manager | | | [ ] |
| Launch Provider Contact | | | [ ] |

### 6.3 Team Location

| Phase | Location |
|-------|----------|
| Launch | MOC / Remote |
| First pass | MOC |
| Commissioning | MOC |

## 7. Launch Day Timeline

### 7.1 Pre-Launch Activities

```
LAUNCH DAY TIMELINE

L-8:00  Team arrives at MOC
        [ ] Comm check
        [ ] System power on
        [ ] Status review

L-6:00  Launch provider coordination
        [ ] Contact with range
        [ ] Weather update
        [ ] Timeline review

L-4:00  Go/No-Go poll (internal)
        [ ] Ground: GO / NO-GO
        [ ] Team: GO / NO-GO
        [ ] Mission Director: GO / NO-GO

L-2:00  Final preparations
        [ ] All systems verified
        [ ] Procedures staged
        [ ] Voice loops hot

L-1:00  Launch provider poll
        [ ] Await launch GO

L-0:00  LAUNCH
        [ ] Confirm launch
        [ ] Start T+ clock

L+1:00 to L+3:00  Separation (S) from the dispenser, by the
        integrator's deploy order (MISSION_TIMELINE §2, §4)
        [ ] Await separation confirmation and state vector
        [ ] Note separation time

S+0:45  First beacon possible: RF silence ends at 45 min
        (CDS 2.4.5), only once the AntS is deployed (C-19)
        [ ] Begin listening (if in view)
```

### 7.2 First Pass Operations

```
FIRST PASS OPERATIONS

T+??:?? AOS (First visible pass)
        [ ] Ground station tracking
        [ ] Begin beacon search

T+??:?? Signal acquired (expected)
        [ ] Carrier detected
        [ ] Doppler confirmed
        [ ] Beacon decoded

T+??:?? Health assessment
        [ ] Mode as expected (IDLE, commissioning hold)
        [ ] Battery SOC nominal
        [ ] Temps nominal
        [ ] No fault flags

T+??:?? LOS
        [ ] Pass summary logged
        [ ] Team notified
        [ ] Next pass planned
```

## 8. Contingency Preparations

### 8.1 Launch Contingencies

| Contingency | Response |
|-------------|----------|
| Launch scrub | Recycle per provider |
| Launch delay | Update predictions |
| Deployment failure | Extended search |
| No signal | Widen search, wait |

### 8.2 First Contact Contingencies

| Scenario | Response |
|----------|----------|
| No signal at AOS | Widen frequency search |
| Weak signal | Adjust antenna, wait for higher pass |
| Unexpected state | Monitor, assess |
| No beacon | Listen for carrier only |

### 8.3 Ground Contingencies

| Scenario | Response |
|----------|----------|
| Primary GS down | Switch to backup |
| Network outage | Local operations |
| Weather | Backup site |
| Personnel | Backup operators |

## 9. Regulatory Status

### 9.1 License Status

| License | Agency | Status |
|---------|--------|--------|
| Pre-space notification, 47 CFR 97.207(g) | FCC | [ ] Filed |
| Frequency coordination | IARU | [ ] Letter received |
| Launch License | Provider | [ ] Approved |
| Orbital debris assessment (ODAR) | FCC | [ ] Filed |

### 9.2 Notifications

| Notification | Recipient | Status |
|--------------|-----------|--------|
| Launch notice | IARU | [ ] Sent |
| Frequency activation | FCC | [ ] Ready |
| Tracking request | 18 SDS | [ ] Submitted |

## 10. Risk Status

### 10.1 Open Risks

From `mission/RISK_REGISTER.md` at the time of the review.

| Risk | Mitigation | Status |
|------|------------|--------|
| | | |

### 10.2 Accepted Risks

| Risk | Rationale |
|------|-----------|
| No propulsion | Passive disposal acceptable |
| Single-string | Acceptable for Class D |

## 11. Documentation Status

### 11.1 Required Documents

| Document | Location | Status |
|----------|----------|--------|
| Launch procedures | Ops binder | [ ] Ready |
| First pass procedures | Ops binder | [ ] Ready |
| Contingency procedures | Ops binder | [ ] Ready |
| Contact list | Posted | [ ] Current |
| Pass schedule | Posted | [ ] Ready |

### 11.2 Waivers and Deviations

| ID | Item | Approval | Status |
|----|------|----------|--------|
| | | | |

## 12. LRR Checklist

### 12.1 Hardware Status

- [ ] Spacecraft integrated with the dispenser
- [ ] Final functional test passed before integration
- [ ] Battery at 60 % ± 5 % SOC at integration
- [ ] Configuration verified
- [ ] All access panels closed

### 12.2 Ground Status

- [ ] Primary ground station operational
- [ ] Backup ground station ready
- [ ] Mission ops center ready
- [ ] Communications verified

### 12.3 Team Status

- [ ] All positions staffed
- [ ] Backup personnel identified
- [ ] Contact list verified
- [ ] Team on station by L-8 hours

### 12.4 Documentation Status

- [ ] All procedures at stations
- [ ] Log sheets ready
- [ ] Contact list posted
- [ ] Schedule posted

### 12.5 Regulatory Status

- [ ] All licenses active
- [ ] Notifications sent
- [ ] Tracking requested

## 13. LRR Board Decision

```
LAUNCH READINESS REVIEW DECISION

Date: ___________
Launch Window: ___________

The Launch Readiness Review Board has reviewed
all preparations and determines:

[ ] GO FOR LAUNCH
    - All readiness criteria met
    - Team ready for launch operations
    - Proceed to launch

[ ] CONDITIONAL GO
    - Conditions: ___________
    - Resolution required by: L-___

[ ] NO-GO
    - Rationale: ___________
    - Required actions: ___________

Signatures:

Mission Director: _________ Date: _____
Project Manager: __________ Date: _____
Ground Lead: ______________ Date: _____
Systems Engineer: _________ Date: _____
```

## 14. Post-LRR Actions

### 14.1 Actions to Launch

| Action | Owner | Due | Status |
|--------|-------|-----|--------|
| Final team brief | MD | L-4 hrs | [ ] |
| Ground validation | Ground | L-4 hrs | [ ] |
| Go/No-Go poll | MD | L-1 hr | [ ] |

### 14.2 Launch Commit Criteria

| Criterion | Status |
|-----------|--------|
| LRR complete | [ ] |
| No open actions | [ ] |
| Weather acceptable | [ ] |
| Launch provider GO | [ ] |
| Team GO | [ ] |
| **LAUNCH COMMIT** | [ ] |

## 15. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
| 1.1 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): banner (template, not held; milestone: launch, 2028-11-01); battery 60 % ± 5 % at integration (was > 95 % / fully charged; BASELINE §4); RBF removed at integration (was installed); hardware status recorded at the integrator, not at the review; timeline: separation at L + 1 to 3 h, first beacon not before S + 45 min (was L + 0:32); first-pass mode IDLE (was SAFE); FCC pre-space notification and IARU letter in place of "approved" licences; GNSS ephemeris row; pre-filled "None" risks and waivers removed |
