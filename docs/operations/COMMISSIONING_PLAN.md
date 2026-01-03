# SMART-QSO Commissioning Plan

**Document ID**: SQSO-COMM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the commissioning plan for the SMART-QSO mission, covering activities from deployment through transition to primary operations per NASA Ames SmallSat heritage.

## 2. Commissioning Overview

### 2.1 Commissioning Objectives

| Objective | Success Criteria |
|-----------|------------------|
| Establish communication | Two-way link verified |
| Verify spacecraft health | All telemetry nominal |
| Characterize power system | Power budget validated |
| Verify thermal behavior | Temps within limits |
| Commission AI payload | AI beacons generated |
| Validate operations | Procedures verified |

### 2.2 Commissioning Duration

| Phase | Duration | Days from L+0 |
|-------|----------|---------------|
| Initial Acquisition | 0-24 hours | L+0 to L+1 |
| Health Assessment | 1-3 days | L+1 to L+3 |
| System Checkout | 3-7 days | L+3 to L+7 |
| Payload Commissioning | 7-10 days | L+7 to L+10 |
| Operations Validation | 10-14 days | L+10 to L+14 |
| **Total** | **14 days** | |

## 3. Pre-Commissioning

### 3.1 Ground Readiness

| Item | Responsible | Status |
|------|-------------|--------|
| Primary GS operational | Ground Lead | [ ] |
| Backup GS available | Ground Lead | [ ] |
| TLE loaded | Ops Lead | [ ] |
| Tracking configured | Ground Lead | [ ] |
| Team on station | Mission Director | [ ] |
| Comm plan distributed | Ops Lead | [ ] |

### 3.2 Pre-Deployment State

| Parameter | Expected Value |
|-----------|----------------|
| Battery SOC | >95% |
| Software | v1.0.0 flight |
| Mode | Will boot to SAFE |
| Antenna | Stowed |
| RF | Inhibited (30 min timer) |

## 4. Phase 1: Initial Acquisition (L+0 to L+1)

### 4.1 Deployment Timeline

```
DEPLOYMENT SEQUENCE

L+0:00:00   Separation from deployer
            └─ Deployment switches actuate
            └─ Power applied to OBC
            └─ Boot sequence begins

L+0:00:30   Boot complete
            └─ STATE: SAFE
            └─ Watchdog active
            └─ RF inhibit timer running

L+0:30:00   Antenna deployment
            └─ Timer expires
            └─ Antenna deploy command
            └─ RF inhibit released

L+0:31:00   First beacon possible
            └─ Beacon scheduled
            └─ TX enabled

L+0:32:00   FIRST BEACON TRANSMITTED
```

### 4.2 First Pass Activities

| Time | Activity | Success Criteria |
|------|----------|------------------|
| AOS-5 min | Ground station ready | All systems green |
| AOS | Begin search | Carrier detected |
| AOS+1 min | Signal acquisition | Beacon decoded |
| AOS+2 min | Health check | Telemetry valid |
| Pass | Monitor | Beacons received |
| LOS | End pass | Data logged |

### 4.3 First Pass Checklist

```
FIRST PASS CHECKLIST

Pre-Pass:
[ ] Antenna pointed to AOS position
[ ] Radio configured for 145.XXX MHz
[ ] Decoder running
[ ] Logging enabled
[ ] Team on headset

During Pass:
[ ] Signal detected at AOS+___
[ ] First beacon decoded at AOS+___
[ ] Callsign verified: SQSO-1
[ ] STATE = SAFE confirmed
[ ] Battery SOC = ___%
[ ] Temperatures nominal
[ ] No fault flags

Post-Pass:
[ ] Data archived
[ ] First pass report filed
[ ] Team notified
[ ] Next pass scheduled
```

### 4.4 Acquisition Success Criteria

| Criterion | Requirement | Achieved |
|-----------|-------------|----------|
| Signal acquired | Within 24 hours | [ ] |
| Beacon decoded | Valid AX.25 | [ ] |
| Telemetry received | All fields valid | [ ] |
| No anomalies | No faults | [ ] |

## 5. Phase 2: Health Assessment (L+1 to L+3)

### 5.1 Health Check Points

| System | Check | Nominal Range | Actual |
|--------|-------|---------------|--------|
| Battery V | Voltage | 3.6-4.1V | |
| Battery I | Current | -0.5 to +1.0A | |
| Battery SOC | State of charge | 50-100% | |
| OBC Temp | Temperature | -20 to +50°C | |
| Battery Temp | Temperature | 0 to +35°C | |
| RF Status | TX enabled | Yes | |
| State | Operating mode | SAFE | |
| Boot Count | Resets | 1 (initial) | |

### 5.2 Power System Characterization

| Orbit | Eclipse Entry SOC | Eclipse Exit SOC | Net Change |
|-------|-------------------|------------------|------------|
| 1 | | | |
| 2 | | | |
| 3 | | | |
| Average | | | |

**Expected**: Net positive charge per orbit

### 5.3 Thermal Characterization

| Component | Hot (Sun) | Cold (Eclipse) | Range OK |
|-----------|-----------|----------------|----------|
| OBC | | | [ ] |
| Battery | | | [ ] |
| Jetson | | | [ ] |

### 5.4 Health Assessment Criteria

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Battery healthy | SOC stable/increasing | [ ] |
| Thermal stable | All temps in limits | [ ] |
| No resets | Boot count = 1 | [ ] |
| Power positive | Net charge > 0 | [ ] |

## 6. Phase 3: System Checkout (L+3 to L+7)

### 6.1 Mode Transition Test

| Step | Command | Expected | Actual | Pass |
|------|---------|----------|--------|------|
| 1 | Verify SAFE | STATE=SAFE | | [ ] |
| 2 | RELEASE_SAFE | STATE=IDLE | | [ ] |
| 3 | Monitor 1 orbit | Stable IDLE | | [ ] |
| 4 | SET_MODE(ACTIVE) | STATE=ACTIVE | | [ ] |
| 5 | Monitor 1 orbit | Stable ACTIVE | | [ ] |
| 6 | SAFE_MODE | STATE=SAFE | | [ ] |

### 6.2 Command Verification

| Command | Sent | ACK | Effect Verified |
|---------|------|-----|-----------------|
| REQUEST_TLM | [ ] | [ ] | [ ] |
| SET_TX_POWER(1) | [ ] | [ ] | [ ] |
| ECHO_TEST | [ ] | [ ] | [ ] |
| RELEASE_SAFE | [ ] | [ ] | [ ] |
| SAFE_MODE | [ ] | [ ] | [ ] |

### 6.3 RF Characterization

| Pass | Elevation | Signal Strength | Link Quality |
|------|-----------|-----------------|--------------|
| 1 | | | |
| 2 | | | |
| 3 | | | |

### 6.4 System Checkout Criteria

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| All modes tested | SAFE, IDLE, ACTIVE | [ ] |
| Commands verified | All critical commands | [ ] |
| RF characterized | Link margins known | [ ] |

## 7. Phase 4: Payload Commissioning (L+7 to L+10)

### 7.1 Jetson Power-On

| Step | Action | Expected | Actual |
|------|--------|----------|--------|
| 1 | Verify ACTIVE mode | STATE=ACTIVE | |
| 2 | Verify SOC > 70% | SOC=___% | |
| 3 | JETSON_POWER(ON) | ACK received | |
| 4 | Wait 60 seconds | Boot complete | |
| 5 | Verify Jetson status | JETSON_STATE=READY | |

### 7.2 AI Inference Test

| Step | Action | Expected | Actual |
|------|--------|----------|--------|
| 1 | AI_ENABLE(1) | ACK received | |
| 2 | Wait for beacon | AI content present | |
| 3 | Verify content | Contextual text | |
| 4 | Monitor power | Within budget | |
| 5 | Monitor thermal | Jetson < 65°C | |

### 7.3 Fallback Mode Test

| Step | Action | Expected |
|------|--------|----------|
| 1 | FORCE_FALLBACK | ACK |
| 2 | Wait for beacon | Template text |
| 3 | Verify fallback | Standard beacon |
| 4 | AI_ENABLE(1) | Return to AI |

### 7.4 Payload Commissioning Criteria

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| Jetson boots | Status = READY | [ ] |
| AI generates content | Contextual beacons | [ ] |
| Power within budget | <15W peak | [ ] |
| Fallback works | Templates available | [ ] |

## 8. Phase 5: Operations Validation (L+10 to L+14)

### 8.1 Nominal Operations Demo

| Day | Activity | Duration | Result |
|-----|----------|----------|--------|
| 10 | 24-hour autonomous | 16 orbits | [ ] Pass |
| 11 | Multi-pass commanding | 5 passes | [ ] Pass |
| 12 | Contingency drill | 1 pass | [ ] Pass |
| 13 | Handoff exercise | 2 shifts | [ ] Pass |
| 14 | Commissioning review | - | [ ] Complete |

### 8.2 Procedure Validation

| Procedure | Executed | Issues | Resolution |
|-----------|----------|--------|------------|
| Pass operations | [ ] | | |
| Commanding | [ ] | | |
| Anomaly response | [ ] | | |
| Shift handoff | [ ] | | |

### 8.3 Operations Validation Criteria

| Criterion | Requirement | Status |
|-----------|-------------|--------|
| 24-hour demo | No anomalies | [ ] |
| Procedures verified | All executed | [ ] |
| Team proficient | Certified | [ ] |

## 9. Commissioning Completion

### 9.1 Commissioning Review Checklist

| Item | Verified | Date |
|------|----------|------|
| All acquisition criteria met | [ ] | |
| All health criteria met | [ ] | |
| All checkout criteria met | [ ] | |
| Payload commissioned | [ ] | |
| Operations validated | [ ] | |
| No open anomalies | [ ] | |
| Team recommends transition | [ ] | |

### 9.2 Transition to Primary Operations

```
COMMISSIONING COMPLETION

Date: ___________
Days since launch: ___

SUMMARY:
- Acquisition: [ ] Complete
- Health: [ ] Nominal
- Systems: [ ] Checked out
- Payload: [ ] Commissioned
- Operations: [ ] Validated

RECOMMENDATION:
[ ] Transition to Primary Operations
[ ] Extended commissioning needed: ___________

Mission Director: ___________ Date: ___
Systems Engineer: __________ Date: ___
Operations Lead: ___________ Date: ___
```

## 10. Commissioning Anomaly Handling

### 10.1 Anomaly Categories

| Category | Response | Authority |
|----------|----------|-----------|
| Critical | Stop, safe, report | Mission Director |
| Major | Investigate, continue | Ops Lead |
| Minor | Log, continue | Operator |

### 10.2 Common Anomalies

| Anomaly | Likely Cause | Response |
|---------|--------------|----------|
| No signal at AOS | TLE error, tumbling | Widen search |
| Weak signal | Low power mode | Monitor |
| Unexpected SAFE | Fault triggered | Investigate |
| High temps | Orientation | Monitor |
| Low SOC | Eclipse timing | Wait for sun |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
