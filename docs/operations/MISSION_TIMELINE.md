# SMART-QSO Mission Timeline and Event Sequence

**Document ID**: SQSO-MTL-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the mission timeline, event sequences, and operational phases for the SMART-QSO mission following NASA Ames SmallSat heritage.

## 2. Mission Overview

### 2.1 Key Dates

| Milestone | Target Date | Status |
|-----------|-------------|--------|
| CDR Complete | 2026-Q1 | Baseline |
| Flatsat Testing | 2026-Q2 | Planned |
| Flight Build | 2026-Q3 | Planned |
| Environmental Test | 2026-Q3 | Planned |
| Launch Readiness | 2026-Q4 | Planned |
| Launch | TBD | Dependent on rideshare |
| End of Mission | L+2 years | Planned |

### 2.2 Mission Duration

| Phase | Duration |
|-------|----------|
| Commissioning | 14 days |
| Primary Operations | 18 months |
| Extended Operations | 6 months (optional) |
| Decommissioning | 30 days |
| **Total** | **~2 years** |

## 3. Pre-Launch Timeline

### 3.1 Integration and Test (I&T)

| Week | Activity |
|------|----------|
| I-16 | Flatsat assembly complete |
| I-15 | Software load and initial checkout |
| I-14 | TP-001 through TP-004 execution |
| I-13 | TP-005 through TP-006 execution |
| I-12 | TP-007 extended duration test start |
| I-11 | Extended duration test continue |
| I-10 | Flight hardware integration start |
| I-9 | Payload integration |
| I-8 | RF integration and calibration |
| I-7 | Functional test |
| I-6 | Pre-environmental review |
| I-5 | Vibration test |
| I-4 | Thermal vacuum test |
| I-3 | Post-environmental functional |
| I-2 | Flight readiness review |
| I-1 | Final configuration and pack |

### 3.2 Launch Campaign

| Day | Activity |
|-----|----------|
| L-30 | Ship to launch site |
| L-21 | Receive and inspect |
| L-14 | Integration with deployer |
| L-7 | Final functional check |
| L-5 | Deployer integration to vehicle |
| L-3 | Launch readiness review |
| L-1 | Final battery charge |
| L-0 | LAUNCH |

## 4. Launch Day Timeline

### 4.1 Launch Day Sequence

```
EVENT SEQUENCE - LAUNCH DAY

T-4:00:00  Ground station readiness check
T-3:00:00  Operations team on station
T-2:00:00  Ground software start
T-1:00:00  Tracking acquisition configured
T-0:30:00  Final go/no-go poll
T-0:00:00  LAUNCH

T+0:XX:XX  Stage separation (per launch vehicle)
T+0:XX:XX  CubeSat deployer activation
T+0:XX:XX  SMART-QSO deployment
```

### 4.2 Deployment Sequence

```
SPACECRAFT AUTONOMOUS SEQUENCE

T+0:00:00  Deployment from P-POD
           - Deployment switches actuate
           - Boot sequence initiated
           - Timer starts

T+0:00:30  Boot complete
           - STATE: SAFE
           - Watchdog active
           - Antenna deploy inhibit active

T+0:30:00  Antenna deployment timer expires
           - Antenna deploy command
           - UHF antenna released
           - VHF antenna released

T+0:31:00  RF activation
           - Transmitter enabled
           - First beacon queued

T+0:32:00  First beacon transmission
           - AX.25 beacon on 145.XXX MHz
           - State: SAFE mode
           - Minimal telemetry

T+0:35:00  Nominal safe mode operations
           - Beacon every 60 seconds
           - Battery charging monitored
           - Attitude stabilization passive
```

## 5. Commissioning Phase (L+0 to L+14 days)

### 5.1 Day 1: Initial Acquisition

| Time | Event | Success Criteria |
|------|-------|------------------|
| First pass | Signal acquisition | Beacon received |
| First pass | Telemetry decode | All fields valid |
| Pass 2-4 | Health assessment | Nominal values |
| Pass 5+ | Continued monitoring | Stable trends |

### 5.2 Days 2-3: System Checkout

| Day | Activity | Commands |
|-----|----------|----------|
| 2 | Battery characterization | Monitor only |
| 2 | Solar array verification | Monitor only |
| 2 | Thermal trending | Monitor only |
| 3 | RF link characterization | TX_POWER variations |
| 3 | Command uplink test | ECHO_TEST |

### 5.3 Days 4-7: Mode Testing

| Day | Activity | Commands |
|-----|----------|----------|
| 4 | SAFE mode verification | Monitor |
| 5 | Transition to IDLE | RELEASE_SAFE |
| 5 | IDLE mode verification | Monitor |
| 6 | Transition to ACTIVE | SET_MODE(ACTIVE) |
| 6 | ACTIVE mode verification | Monitor |
| 7 | Mode cycling test | Multiple transitions |

### 5.4 Days 8-10: Payload Commissioning

| Day | Activity | Commands |
|-----|----------|----------|
| 8 | Jetson power-on | JETSON_POWER(ON) |
| 8 | Jetson boot verification | Monitor |
| 9 | AI inference test | AI_ENABLE(1) |
| 9 | AI beacon verification | Monitor beacon content |
| 10 | Fallback mode test | FORCE_FALLBACK |

### 5.5 Days 11-14: Nominal Operations Verification

| Day | Activity |
|-----|----------|
| 11 | 24-hour nominal ops demo |
| 12 | Continued monitoring |
| 13 | Commissioning review |
| 14 | Transition to Primary Ops |

### 5.6 Commissioning Success Criteria

| Criterion | Requirement |
|-----------|-------------|
| Signal acquisition | < 24 hours from deployment |
| All modes tested | SAFE, IDLE, ACTIVE |
| Payload functional | AI beacons generated |
| Power positive | Average charge > discharge |
| Thermal stable | All temps in limits |
| Command response | > 90% success rate |

## 6. Primary Operations Phase (L+14 days to L+18 months)

### 6.1 Nominal Orbit Operations

```
TYPICAL ORBIT TIMELINE (93 min period)

T+0:00  Eclipse entry
        - SOC decreasing
        - Jetson powered off (if running)
        - Thermal decreasing

T+0:35  Eclipse exit
        - Solar charging begins
        - SOC increasing
        - Thermal increasing

T+0:45  Ground pass (if visible)
        - Beacon transmission
        - Telemetry download
        - Commands (if scheduled)

T+0:55  AI window (if ACTIVE, SOC > 70%)
        - Jetson power-on
        - Inference execution
        - AI beacon generation

T+1:00  Continue operations
        - Monitoring
        - Beacon transmission

T+1:33  Next orbit begins
```

### 6.2 Weekly Operations Schedule

| Day | Primary Activity |
|-----|------------------|
| Monday | Health review, trending analysis |
| Tuesday | Routine passes |
| Wednesday | AI beacon analysis |
| Thursday | Routine passes |
| Friday | Weekly report |
| Weekend | Automated monitoring |

### 6.3 Monthly Milestones

| Month | Focus |
|-------|-------|
| 1 | Establish operational baseline |
| 2 | Optimize AI duty cycle |
| 3 | First monthly report |
| 6 | Mid-mission review |
| 12 | Annual review |
| 18 | Primary mission complete |

## 7. Extended Operations Phase (Optional)

### 7.1 Entry Criteria

- Primary mission objectives met
- Spacecraft health supports continuation
- Resources available for operations
- Science/education value justified

### 7.2 Extended Operations Focus

| Focus | Activities |
|-------|------------|
| Degradation trending | Monitor aging effects |
| Experimental modes | New AI configurations |
| Education outreach | Student operations |
| Data collection | Long-term statistics |

## 8. End of Life Phase

### 8.1 Decommissioning Criteria

| Criterion | Threshold |
|-----------|-----------|
| Battery capacity | < 50% of initial |
| Solar degradation | < 70% of initial |
| Communication reliability | < 80% pass success |
| Altitude decay | < 350 km |

### 8.2 Decommissioning Sequence

```
DECOMMISSIONING TIMELINE

EOL-30  Decommissioning decision
        - Review spacecraft health
        - Assess remaining capability
        - Plan final operations

EOL-21  Final science operations
        - Last AI experiments
        - Final data collection

EOL-14  Public notification
        - Announce decommissioning
        - Final QSO opportunities

EOL-7   System passivation prep
        - Download final telemetry
        - Configuration backup

EOL-1   Final transmission
        - Farewell beacon
        - RF disable command

EOL-0   PASSIVATION
        - Transmitter disabled
        - Battery discharge mode
        - Spacecraft silent
```

### 8.3 Disposal Compliance

Per NASA-STD-8719.14:
- Maximum orbital lifetime after EOL: 25 years
- Expected decay: < 5 years (low LEO)
- No debris generation
- Battery passivated

## 9. Event Response Timelines

### 9.1 Anomaly Response

| Severity | Initial Response | Resolution Target |
|----------|------------------|-------------------|
| Critical | < 1 hour | 24 hours |
| High | < 4 hours | 72 hours |
| Medium | < 24 hours | 1 week |
| Low | < 1 week | 1 month |

### 9.2 Contingency Event Timelines

| Event | Detection | Response | Recovery |
|-------|-----------|----------|----------|
| Safe mode entry | Next pass | Same pass | 24-72 hours |
| Loss of signal | 3 passes | Immediate | Varies |
| Battery low | Real-time | Same pass | 1-3 orbits |
| Thermal excursion | Real-time | Same pass | 1 orbit |

## 10. Contact Schedule

### 10.1 Ground Station Coverage

| Station | Location | Coverage |
|---------|----------|----------|
| Primary | TBD | All visible passes |
| Backup | TBD | Critical passes |
| Amateur network | Worldwide | Best effort |

### 10.2 Pass Statistics (estimated for 400 km, 51.6° orbit)

| Metric | Value |
|--------|-------|
| Passes per day | 4-6 visible from primary |
| Max elevation | Varies 10° to 85° |
| Pass duration | 5-12 minutes |
| Contact time per day | ~30-60 minutes |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
