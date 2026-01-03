# SMART-QSO End-to-End Test Plan

**Document ID**: SQSO-E2E-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the End-to-End (E2E) Test Plan for SMART-QSO, covering mission-level testing that validates the complete system from ground command through spacecraft response and data return per NASA Ames SmallSat heritage practices.

## 2. End-to-End Test Philosophy

### 2.1 Test Objectives

| Objective | Description |
|-----------|-------------|
| System Integration | Verify all subsystems work together |
| Operations Validation | Verify procedures and tools |
| Performance Verification | Measure system-level performance |
| Fault Response | Verify fault handling across system |
| Mission Readiness | Confirm flight-like operations |

### 2.2 Test Approach

```
END-TO-END TEST ARCHITECTURE

Ground System                     Spacecraft
┌──────────────────┐             ┌──────────────────┐
│  Mission Ops     │             │    Flight SW     │
│  Workstation     │             │                  │
│  ┌────────────┐  │   RF Link   │  ┌────────────┐  │
│  │ Ground SW  │──┼─────────────┼──│    OBC     │  │
│  └────────────┘  │  (or EGSE)  │  └────────────┘  │
│        │         │             │        │         │
│  ┌────────────┐  │             │  ┌────────────┐  │
│  │  Decoder   │  │             │  │  Payload   │  │
│  └────────────┘  │             │  └────────────┘  │
└──────────────────┘             └──────────────────┘

Test Configurations:
1. RF Link (Flight-like)
2. EGSE/Umbilical (Debug)
3. Simulator (Pre-hardware)
```

### 2.3 Test Levels

| Level | Configuration | Purpose |
|-------|---------------|---------|
| E2E-SIM | Simulator | Software validation |
| E2E-FLAT | Flatsat | Hardware-in-loop |
| E2E-EM | Engineering Model | Near-flight testing |
| E2E-FM | Flight Model | Flight acceptance |

## 3. Test Configuration

### 3.1 Ground Segment

| Component | Description | Status |
|-----------|-------------|--------|
| Ground Station | TNC, radio, antenna | [ ] Ready |
| Mission Ops SW | Command/telemetry tools | [ ] Ready |
| Tracking | TLE management | [ ] Ready |
| Network | Connectivity | [ ] Ready |

### 3.2 Space Segment

| Component | Description | Status |
|-----------|-------------|--------|
| OBC | Flight computer | [ ] Ready |
| EPS | Power system | [ ] Ready |
| RF | Radio subsystem | [ ] Ready |
| Payload | Jetson AI | [ ] Ready |

### 3.3 EGSE Configuration

| Equipment | Purpose | Interface |
|-----------|---------|-----------|
| Power Supply | Simulate solar/battery | J1 Power |
| Serial Adapter | Debug access | J2 UART |
| Logic Analyzer | Signal monitoring | Test points |
| RF Attenuator | Link simulation | SMA |

## 4. End-to-End Test Cases

### 4.1 E2E-001: Nominal Pass Operations

**Objective**: Verify complete pass from AOS to LOS

| Step | Action | Expected Result | Pass |
|------|--------|-----------------|------|
| 1 | Power on spacecraft | Boot to SAFE | [ ] |
| 2 | Wait for beacon | Beacon received <2 min | [ ] |
| 3 | Decode telemetry | All fields valid | [ ] |
| 4 | Send REQUEST_TLM | ACK received | [ ] |
| 5 | Receive telemetry | Complete packet | [ ] |
| 6 | Send SET_MODE(ACTIVE) | State change confirmed | [ ] |
| 7 | Monitor 10 beacons | All received correctly | [ ] |
| 8 | Send SAFE_MODE | Return to SAFE | [ ] |

**Pass Criteria**: All steps complete without error

### 4.2 E2E-002: Command Chain

**Objective**: Verify sequential commanding

| Step | Command Sequence | Expected |
|------|------------------|----------|
| 1 | RELEASE_SAFE | → IDLE |
| 2 | SET_MODE(ACTIVE) | → ACTIVE |
| 3 | SET_TX_POWER(2) | Power level 2 |
| 4 | REQUEST_TLM | Full telemetry |
| 5 | AI_ENABLE(1) | AI content in beacon |
| 6 | AI_ENABLE(0) | Template beacon |
| 7 | SAFE_MODE | → SAFE |

**Pass Criteria**: All commands acknowledged and executed

### 4.3 E2E-003: AI Payload Integration

**Objective**: Verify end-to-end AI beacon generation

| Step | Action | Expected |
|------|--------|----------|
| 1 | Power on system | Boot complete |
| 2 | Transition to ACTIVE | STATE = ACTIVE |
| 3 | Enable AI | JETSON_POWER = ON |
| 4 | Wait for AI ready | JETSON_STATE = READY |
| 5 | Request AI beacon | AI content generated |
| 6 | Transmit beacon | AI beacon received |
| 7 | Decode content | Contextual text valid |
| 8 | Verify power | Within budget |

**Pass Criteria**: AI-generated beacon received and decoded

### 4.4 E2E-004: Fault Injection

**Objective**: Verify fault handling across system

| Fault | Injection | Expected Response |
|-------|-----------|-------------------|
| Power glitch | Brief power dip | Watchdog recovery |
| Bad command | Invalid opcode | NAK, no state change |
| Comm loss | RF path blocked | Autonomous beacons |
| Jetson hang | No AI response | Fallback to templates |
| Low battery | Reduce input voltage | Auto-SAFE transition |

**Pass Criteria**: All faults handled gracefully

### 4.5 E2E-005: Extended Duration

**Objective**: 24-hour autonomous operations

| Hour | Check Point | Criteria |
|------|-------------|----------|
| 0 | Start test | Initial state recorded |
| 1 | Health check | All nominal |
| 4 | Health check | All nominal |
| 8 | Health check | All nominal |
| 12 | Midpoint | Beacon count verified |
| 16 | Health check | All nominal |
| 20 | Health check | All nominal |
| 24 | End test | Final state recorded |

**Pass Criteria**:
- 1,440 beacons transmitted (±5%)
- No resets or anomalies
- Power positive each orbit

### 4.6 E2E-006: Day-in-the-Life (DITL)

**Objective**: Full operational day simulation

```
DITL TIMELINE (24 HOURS)

00:00 - Shift 1 Start
        └─ Handoff briefing
        └─ System status review

02:00 - Pass 1 (Simulated)
        └─ Execute nominal pass procedure
        └─ Command health request

06:00 - Pass 2
        └─ Mode change exercise
        └─ AI beacon collection

10:00 - Shift 2 Start
        └─ Handoff exercise
        └─ Review pass data

12:00 - Anomaly Injection
        └─ Simulate comm timeout
        └─ Execute contingency

14:00 - Pass 3
        └─ Recovery verification
        └─ Nominal operations

18:00 - Shift 3 Start
        └─ Night ops handoff
        └─ Reduced manning

22:00 - Pass 4
        └─ Standard monitoring
        └─ Prepare for day 2

24:00 - DITL Complete
        └─ Final handoff
        └─ Test report
```

**Pass Criteria**: All procedures executed successfully

## 5. Test Environment

### 5.1 RF Link Configuration

| Parameter | Test Value | Flight Value |
|-----------|------------|--------------|
| TX Frequency | 145.XXX MHz | Same |
| RX Frequency | 435.XXX MHz | Same |
| TX Power | 0.5W (attenuated) | 0.5-2W |
| Path Loss | 60 dB (simulated) | 145 dB |
| Data Rate | 1200/9600 bps | Same |

### 5.2 Orbit Simulation

| Parameter | Simulation |
|-----------|------------|
| Pass duration | 10 min simulated |
| Eclipse | Simulated via power |
| Doppler | Not simulated (static) |
| Range | Fixed |

### 5.3 Environment Conditions

| Condition | E2E-FLAT | E2E-FM |
|-----------|----------|--------|
| Temperature | Ambient | Ambient |
| Pressure | 1 atm | 1 atm |
| Vibration | None | None |
| RF | Lab conditions | Lab conditions |

## 6. Test Procedures

### 6.1 Test Setup

```
E2E TEST SETUP PROCEDURE

1. Ground System Setup
   [ ] Power on ground station
   [ ] Start mission ops software
   [ ] Load TLE (simulated)
   [ ] Verify decoder running
   [ ] Configure logging

2. Spacecraft Setup
   [ ] Install in test fixture
   [ ] Connect EGSE (if used)
   [ ] Verify power supply settings
   [ ] Connect RF (attenuated)
   [ ] Verify test equipment ready

3. Configuration Verification
   [ ] Ground SW version: _______
   [ ] Flight SW version: _______
   [ ] RF attenuation: _______ dB
   [ ] Power supply: _______ V

4. Go/No-Go
   [ ] Ground: GO
   [ ] Spacecraft: GO
   [ ] Test Director: GO FOR TEST
```

### 6.2 Test Execution

```
E2E TEST EXECUTION LOG

Test ID: E2E-___
Date: ___________
Test Conductor: ___________

START TIME: ___:___:___

Step | Time | Action | Result | Notes
─────┼──────┼────────┼────────┼───────
  1  |      |        |        |
  2  |      |        |        |
  3  |      |        |        |
...

END TIME: ___:___:___

RESULT: [ ] PASS  [ ] FAIL  [ ] INCOMPLETE

Anomalies: _______________________
Actions: _________________________

Test Conductor: _________ Date: _____
QA Witness: ___________ Date: _____
```

### 6.3 Test Teardown

```
E2E TEST TEARDOWN

1. Spacecraft Safing
   [ ] Command SAFE_MODE
   [ ] Verify SAFE state
   [ ] Power down spacecraft
   [ ] Disconnect EGSE

2. Data Collection
   [ ] Export telemetry logs
   [ ] Export command logs
   [ ] Export RF captures
   [ ] Archive to test data folder

3. Equipment Shutdown
   [ ] Ground station standby
   [ ] Power supplies off
   [ ] Test equipment secured
   [ ] Lab secured
```

## 7. Pass/Fail Criteria

### 7.1 Test Case Criteria

| Test | Pass Criteria |
|------|---------------|
| E2E-001 | All steps complete, no errors |
| E2E-002 | All commands ACK'd and executed |
| E2E-003 | AI beacon received and valid |
| E2E-004 | All faults handled correctly |
| E2E-005 | 24 hours, no anomalies |
| E2E-006 | All procedures executed |

### 7.2 Overall E2E Criteria

| Criterion | Requirement |
|-----------|-------------|
| All tests executed | 100% |
| All tests passed | 100% (or waived) |
| No critical anomalies | 0 |
| All procedures validated | Yes |

## 8. Test Schedule

### 8.1 E2E Test Campaign

| Week | Activity | Tests |
|------|----------|-------|
| 1 | Setup and checkout | - |
| 2 | E2E-001, E2E-002 | Nominal ops |
| 3 | E2E-003, E2E-004 | AI, faults |
| 4 | E2E-005 | Extended |
| 5 | E2E-006 | DITL |
| 6 | Retest and report | - |

### 8.2 Prerequisites

| Prerequisite | Status |
|--------------|--------|
| Flatsat tests complete | [ ] |
| Ground system validated | [ ] |
| Procedures approved | [ ] |
| Team trained | [ ] |

## 9. Resources

### 9.1 Personnel

| Role | Name | Availability |
|------|------|--------------|
| Test Director | | [ ] |
| Test Conductor | | [ ] |
| Ground Operator | | [ ] |
| Spacecraft Engineer | | [ ] |
| QA | | [ ] |

### 9.2 Facilities

| Facility | Dates | Reserved |
|----------|-------|----------|
| Integration Lab | Week 1-5 | [ ] |
| Ground Station | Week 2-5 | [ ] |

## 10. Data Products

### 10.1 Test Outputs

| Product | Description |
|---------|-------------|
| Test logs | Step-by-step execution |
| Telemetry files | All received telemetry |
| Command logs | All transmitted commands |
| Anomaly reports | Any issues found |
| Test report | Summary and results |

### 10.2 Success Metrics

| Metric | Target | Actual |
|--------|--------|--------|
| Command success rate | >99% | |
| Beacon decode rate | >99% | |
| Fault recovery rate | 100% | |
| Procedure compliance | 100% | |

## 11. Risk and Contingency

### 11.1 Test Risks

| Risk | Mitigation |
|------|------------|
| Equipment failure | Spares available |
| Personnel unavailable | Cross-training |
| Schedule slip | Buffer time included |
| Test environment issues | Pre-test checkout |

### 11.2 Anomaly Handling

| Severity | Response |
|----------|----------|
| Critical | Stop test, safe spacecraft, report |
| Major | Pause, assess, decide continue/stop |
| Minor | Log, continue, review post-test |

## 12. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR baseline |
