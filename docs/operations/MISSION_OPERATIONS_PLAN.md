# Mission Operations Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-MOP-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7120.5, NPR 7123.1

---

## 1. Purpose

This Mission Operations Plan defines the operational procedures, team organization, and resources required to conduct the SMART-QSO CubeSat mission from launch through end-of-life.

## 2. Mission Overview

### 2.1 Mission Objectives

1. Demonstrate autonomous AI-generated amateur radio beacons
2. Validate TinyLM/TinyML operation in LEO environment
3. Provide educational amateur radio contact opportunities
4. Collect spacecraft health telemetry for model validation

### 2.2 Mission Phases

| Phase | Duration | Activities |
|-------|----------|------------|
| Launch & Early Ops (LEOP) | 0-7 days | Deployment, checkout, commissioning |
| Commissioning | 7-30 days | Full system checkout, calibration |
| Primary Mission | 30-365 days | Nominal operations, AI beacon generation |
| Extended Mission | 365-730 days | Continued operations if healthy |
| End-of-Life (EOL) | Final 30 days | Passivation, deorbit preparation |

### 2.3 Success Criteria

| Level | Criteria |
|-------|----------|
| Full Success | All objectives met, 12-month operation |
| Partial Success | Primary beacon operations for 6 months |
| Minimum Success | Initial checkout complete, 10+ beacons received |

## 3. Operations Organization

### 3.1 Team Structure

```
Mission Director
├── Flight Director
│   ├── Spacecraft Controller
│   ├── Ground Network Operator
│   └── Data Manager
├── Systems Engineer
│   ├── EPS Lead
│   ├── ADCS Lead
│   └── Payload Lead
└── Mission Planner
    └── Pass Scheduler
```

### 3.2 Roles and Responsibilities

| Role | Primary Responsibilities |
|------|-------------------------|
| Mission Director | Overall mission authority, anomaly decisions |
| Flight Director | Real-time operations, pass execution |
| Spacecraft Controller | Commanding, telemetry monitoring |
| Ground Network Operator | Station scheduling, link management |
| Data Manager | Telemetry archiving, distribution |
| Systems Engineer | Technical analysis, anomaly investigation |
| EPS Lead | Power system monitoring, analysis |
| ADCS Lead | Attitude monitoring, analysis |
| Payload Lead | AI payload operations, performance |
| Mission Planner | Schedule development, resource allocation |

### 3.3 Shift Coverage

| Phase | Coverage | Staffing |
|-------|----------|----------|
| LEOP | 24/7 | 3 shifts, 2 operators per shift |
| Commissioning | Extended hours | 16 hours/day, 1-2 operators |
| Primary Ops | Business hours | 8 hours/day, 1 operator + on-call |
| Extended Ops | Reduced | 4 hours/day, 1 operator + on-call |
| EOL | Extended | As needed for passivation |

## 4. Ground Segment

### 4.1 Primary Ground Station

| Parameter | Specification |
|-----------|---------------|
| Location | TBD (university or amateur station) |
| Antenna | Yagi or dish, 10+ dB gain |
| Frequency | 145.8-146.0 MHz (downlink) |
| Receiver | SDR-based, 1200 bps AFSK capability |
| TNC | Dire Wolf or hardware TNC |
| Tracking | GPREDICT or similar |

### 4.2 SatNOGS Network

SMART-QSO will utilize the SatNOGS global ground station network for:
- Beacon reception
- Telemetry collection
- Pass scheduling
- Data aggregation

### 4.3 Mission Control Center

| System | Description |
|--------|-------------|
| Telemetry Display | Real-time beacon decoding, health display |
| Archive | PostgreSQL database for all received data |
| Analysis Tools | Python-based analysis notebooks |
| Alerting | Email/SMS for anomaly conditions |
| Web Dashboard | Public beacon display (delayed) |

## 5. Nominal Operations

### 5.1 Daily Activities

| Time | Activity | Duration |
|------|----------|----------|
| Morning | Review overnight beacon data | 30 min |
| Morning | Update TLE, pass predictions | 15 min |
| Morning | Schedule day's passes | 15 min |
| Pass time | Execute scheduled passes | Variable |
| Evening | Daily status summary | 30 min |

### 5.2 Weekly Activities

| Activity | Duration |
|----------|----------|
| Trend analysis (power, thermal) | 2 hours |
| SatNOGS data review | 1 hour |
| Weekly status report | 1 hour |
| Ground system maintenance | 2 hours |

### 5.3 Pass Execution Procedure

1. **T-30 min:** Verify ground station ready, TLE updated
2. **T-5 min:** Start tracking, verify antenna movement
3. **AOS:** Begin recording, enable decoder
4. **Pass:** Monitor beacon reception, log data
5. **LOS:** Stop recording, verify data captured
6. **Post-pass:** Upload to database, verify decode

### 5.4 Beacon Cadence

| Power Mode | Beacon Interval | Expected per Day |
|------------|-----------------|------------------|
| SAFE | 120 seconds | ~720 |
| IDLE | 60 seconds | ~1440 |
| ACTIVE | 30 seconds | ~2880 |

## 6. Contingency Operations

### 6.1 Anomaly Classification

| Class | Definition | Response Time |
|-------|------------|---------------|
| Critical | Mission-threatening | Immediate |
| Major | Significant degradation | < 4 hours |
| Minor | Limited impact | < 24 hours |
| Observation | Trending concern | Next planning cycle |

### 6.2 Contingency Procedures

#### 6.2.1 No Beacon Received

1. Verify ground station functionality with other satellites
2. Check TLE accuracy, update if needed
3. Check for known RF interference
4. Contact SatNOGS network for independent verification
5. If no beacons for 48 hours, declare anomaly
6. Analyze last known telemetry for root cause

#### 6.2.2 Beacon Received but Not Decoded

1. Record raw audio/IQ for offline analysis
2. Verify TNC configuration (1200 bps AFSK)
3. Check for frequency drift
4. Compare with known good decoders
5. Report to community for assistance

#### 6.2.3 Low Power Indication

1. Monitor SOC trend over multiple passes
2. Analyze eclipse/sun pattern
3. Check ADCS attitude data
4. If SOC < 30%, expect transition to SAFE mode
5. Monitor for automatic recovery

#### 6.2.4 Thermal Anomaly

1. Monitor temperature trends
2. Check Jetson operation status
3. Verify heater operation (if applicable)
4. Compare with thermal model predictions
5. If outside limits, expect automatic safe mode

### 6.3 Anomaly Investigation Process

1. **Detection:** Automated or manual anomaly identification
2. **Notification:** Alert operations team
3. **Assessment:** Evaluate severity and impact
4. **Investigation:** Root cause analysis
5. **Resolution:** Develop and implement fix
6. **Documentation:** Update anomaly log
7. **Lessons Learned:** Update procedures if needed

## 7. Mission Phases

### 7.1 LEOP (Launch and Early Operations)

**Duration:** Days 0-7

| Day | Activities |
|-----|------------|
| 0 | Launch, separation, initial acquisition |
| 0-1 | Antenna deployment verification |
| 1-2 | Power system checkout |
| 2-3 | ADCS checkout, detumble verification |
| 3-5 | Communication system verification |
| 5-7 | Initial AI payload activation |

**Success Criteria:**
- Beacons received and decoded
- Power positive
- Attitude stabilizing
- Telemetry within limits

### 7.2 Commissioning

**Duration:** Days 7-30

| Week | Activities |
|------|------------|
| 2 | Full sensor checkout, calibration |
| 3 | Extended AI payload operations |
| 4 | Performance characterization |

**Success Criteria:**
- All subsystems operational
- AI-generated beacons validated
- Telemetry trending nominal

### 7.3 Primary Mission

**Duration:** Days 30-365

| Activity | Frequency |
|----------|-----------|
| Beacon monitoring | Continuous |
| Trend analysis | Weekly |
| Status reports | Monthly |
| Performance reviews | Quarterly |

### 7.4 Extended Mission

**Duration:** Days 365-730 (if approved)

| Criterion | Threshold |
|-----------|-----------|
| Power system | Positive energy balance |
| Communication | Regular beacon reception |
| Thermal | Within operational limits |
| ADCS | Stable attitude |

### 7.5 End-of-Life

See END_OF_LIFE_PLAN.md for detailed procedures.

## 8. Data Management

### 8.1 Data Products

| Product | Format | Retention |
|---------|--------|-----------|
| Raw beacon audio | WAV | 90 days |
| Decoded telemetry | JSON | Permanent |
| Beacon text | Text | Permanent |
| Analysis reports | PDF | Permanent |
| Orbit data (TLE) | TLE | Permanent |

### 8.2 Data Distribution

| Audience | Data | Latency |
|----------|------|---------|
| Operations team | All telemetry | Real-time |
| Science team | Processed data | Daily |
| Amateur community | Beacon text | Real-time |
| Public | Summary | Weekly |

### 8.3 Data Archive

| System | Description |
|--------|-------------|
| Primary | PostgreSQL database |
| Backup | Daily backup to cloud |
| Long-term | Annual archive to permanent storage |

## 9. Communications

### 9.1 Internal Communications

| Channel | Use |
|---------|-----|
| Slack/Discord | Real-time operations |
| Email | Formal communications |
| Phone | Emergency contact |
| Video conf | Team meetings |

### 9.2 External Communications

| Audience | Channel | Frequency |
|----------|---------|-----------|
| Amateur community | QRZ.com, AMSAT-BB | As needed |
| General public | Website, social media | Weekly |
| Scientific community | Publications | As results available |
| Regulatory | As required | As needed |

## 10. Training

### 10.1 Required Training

| Role | Training |
|------|----------|
| All operators | Ground station operation |
| All operators | Telemetry interpretation |
| All operators | Anomaly procedures |
| Flight directors | Pass execution |
| Systems engineers | Subsystem specifics |

### 10.2 Training Schedule

| Activity | Timing |
|----------|--------|
| Initial training | L-60 days |
| Simulation exercises | L-30 days |
| Dress rehearsal | L-7 days |
| Refresher training | Every 6 months |

## 11. Documentation

### 11.1 Operational Documents

| Document | Purpose |
|----------|---------|
| Mission Operations Plan | This document |
| Operations Handbook | Detailed procedures |
| Contingency Procedures | Anomaly response |
| Ground Station Manual | Station operation |
| Telemetry Handbook | Data interpretation |

### 11.2 Logs and Reports

| Document | Frequency |
|----------|-----------|
| Operations log | Continuous |
| Pass reports | Per pass |
| Daily summary | Daily |
| Weekly report | Weekly |
| Monthly report | Monthly |
| Anomaly reports | As needed |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
