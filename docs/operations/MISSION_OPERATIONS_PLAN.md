# Mission Operations Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-MOP-001
**Revision:** C
**Date:** 2026-10-27
**Reference:** NPR 7120.5, NPR 7123.1

---

## 1. Purpose

This Mission Operations Plan defines the operational procedures, team organization, and resources required to conduct the SMART-QSO CubeSat mission from launch through end-of-life.

## 2. Mission Overview

### 2.1 Mission Objectives

From `docs/mission/MISSION_SUCCESS_CRITERIA.md` (SQSO-MSC-001) §2 and BASELINE §2:

1. **MO-1 (primary):** a single-channel U/V FM transponder for worldwide amateur contacts, duty-gated by available power
2. **MO-2:** a 1200 bps AX.25 telemetry beacon in published formats
3. **MO-3:** an onboard AI experiment: the OBC selector schedules the transponder within the power budget and selects
   beacon templates, with hard limits outside the model and fallback to a conventional transponder (D5)
4. **MO-4:** operation within Part 97 and the debris rules, including TX-OFF and end-of-mission passivation
5. **E-1 (experiment, not a success gate):** Jetson-generated beacon text, enabled only if the measured energy per
   accepted beacon line is ≤ 0.1 Wh (decision C-20)

### 2.2 Mission Phases

| Phase | Duration | Activities |
|-------|----------|------------|
| Launch & Early Ops (LEOP) | Separation to about S+1 d | 45-minute RF silence, deployment, first beacon, first authenticated command, TX-OFF proof (`docs/operations/MISSION_TIMELINE.md` §5) |
| Commissioning | Days 1–14 | Steps C-1 to C-11 of `docs/operations/COMMISSIONING_PLAN.md`; transponder and Jetson only after `COMMISSIONING_RELEASE` |
| Primary Mission | Day 14 to 2 years (design life; minimum 12 months) | Duty-gated transponder, beacon, selector; Jetson experiment if its gate passes |
| End-of-Life (EOL) | At the end of the mission | Two-key passivation (`docs/operations/END_OF_LIFE_PLAN.md`); re-entry within 5 years of the mission end |

### 2.3 Success Criteria

Defined in SQSO-MSC-001 (L1 at 30 days, L2 at 12 months, L3 at 24 months; BASELINE §2). The Jetson experiment is not
a gate for any level.

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

### 4.2 Telecommand Stations

Telecommands go up on the coordinated 70 cm uplink (435–438 MHz) from designated stations only (47 CFR 97.211;
decision D1 in `docs/BASELINE_PARAMETERS.md` §9: at least two stations to be named before the IARU request). Each
command carries an HMAC-SHA-256 tag and a per-key counter (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`); TX-OFF is
accepted without a key when no key is usable. Station equipment: a 70 cm transmitter and TNC, the command builder
(`software/ground/tools/telecommand_builder.py`) and the station's key, held offline and never in the repository.

### 4.3 SatNOGS Network

SMART-QSO will utilize the SatNOGS global ground station network for:
- Beacon reception
- Telemetry collection
- Pass scheduling
- Data aggregation

### 4.4 Mission Control Center

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
| SAFE | 180 seconds | ~480 |
| IDLE | 120 seconds | ~720 |
| ACTIVE | 60 seconds | ~1440 |

Intervals are `BEACON_INTERVAL_*_MS` in `software/flight/include/beacon.h`. ACTIVE is the baseline rate (one beacon per
60 s, `docs/BASELINE_PARAMETERS.md`); no mode beacons faster. No beacon is sent while TX-OFF, loss of contact or the
post-deployment RF silence inhibits the transmitter.

### 5.5 Telecommand and Transponder Operations

- **Contact.** Each authenticated command resets the loss-of-contact timer: after N days without one (30 at launch,
  14 after commissioning) the spacecraft stops transmitting and keeps listening (`docs/BASELINE_PARAMETERS.md` §7).
  Send at least one authenticated command well inside N; a station that cannot reach the spacecraft for that long
  hands off to another designated station.
- **Cessation of emissions.** TX-OFF is the first command of any interference report; it latches in NVM and is
  cleared only by an authenticated TX-ON (`docs/BASELINE_PARAMETERS.md` §7, 97.207(b)).
- **Transponder.** The FM transponder is duty-gated by power and thermal limits (decision D4): sunlit, ACTIVE, SOC ≥ 55 %,
  PA below 70 °C, keyed ≤ 25 % of 100 min (`sqso_fsw::transponder`; SQSO-ICD-003 §5.5); `XPDR_ENABLE` (0x34) and
  `XPDR_DISABLE` (0x35) allow or stop it, and it runs only after `COMMISSIONING_RELEASE`.
- **Counters and keys.** Each station uses its own key and never reuses a counter; key rotation uses the recovery key
  (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`).

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
4. Below 40 % SOC expect ACTIVE → IDLE, below 25 % SAFE, below 15 % EMERGENCY (beacon shed) (BASELINE §5)
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

### 7.1 LEOP and Commissioning

The LEOP sequence minute by minute, with energy and ground actions, is `docs/operations/MISSION_TIMELINE.md`
(SQSO-MTL-001) §5; commissioning, days 1–14, step by step with entry and exit criteria, is
`docs/operations/COMMISSIONING_PLAN.md` (SQSO-COMM-001). They govern this section.

### 7.2 Primary Mission

**Duration:** Day 14 to the end of the 2-year design life

| Activity | Frequency |
|----------|-----------|
| Beacon monitoring | Continuous |
| Authenticated contact | At least once in every 14-day loss-of-contact period |
| `DUMP_TLM` of the stored housekeeping | Daily (as in commissioning) or as passes allow |
| Ephemeris to Space-Track | After every pass until cataloged, then per SQSO-REG-015 |
| Trend analysis | Weekly |
| Status reports | Monthly |
| Success declarations | L1 at 30 days, L2 at 12 months, L3 at 24 months (SQSO-MSC-001 §9) |

### 7.3 Mission Extension

An extension beyond 2 years is decided on the conditions of `docs/operations/END_OF_LIFE_PLAN.md` §2.2.

### 7.4 End-of-Life

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
| B | 2026-10-25 | §4.2 telecommand stations and §5.5 telecommand and transponder operations, from the baseline (the plan described a receive-only ground segment) | SMART-QSO Team |
| C | 2026-10-27 | Documentation audit (post-CDR 2026): §2 objectives, phases and success criteria from SQSO-MSC-001 and BASELINE §2 (were AI-beacon objectives, a 12-month full success and 30-day commissioning); §5.5 the transponder as implemented (was "not yet implemented"); §6.2.3 the SOC thresholds of BASELINE §5; §7 LEOP and commissioning point to SQSO-MTL-001 and SQSO-COMM-001, primary mission to 2 years | Documentation audit |
