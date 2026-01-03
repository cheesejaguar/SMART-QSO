# End-of-Life Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-EOL-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NASA-STD-8719.14A, FCC 47 CFR 25.114

---

## 1. Purpose

This document defines the procedures for safe and compliant end-of-life (EOL) operations for the SMART-QSO CubeSat, including passivation and deorbit provisions.

## 2. EOL Triggers

### 2.1 Planned EOL

| Trigger | Condition |
|---------|-----------|
| Mission complete | All objectives achieved, no extension approved |
| Mission duration | 24 months elapsed (extended mission limit) |
| Resource depletion | Battery capacity degraded below useful threshold |

### 2.2 Unplanned EOL

| Trigger | Condition |
|---------|-----------|
| Critical anomaly | Non-recoverable spacecraft failure |
| Regulatory requirement | License expiration or revocation |
| Safety concern | Collision risk or other safety issue |

### 2.3 Autonomous EOL

| Trigger | Condition |
|---------|-----------|
| Contact loss | No valid ground contact for 30+ days |
| Critical battery | SOC < 15% for extended period |
| Thermal emergency | Temperature outside survival limits |

## 3. Passivation Requirements

### 3.1 Passivation Objectives

Per NASA-STD-8719.14A:
1. Eliminate all sources of stored energy that could cause break-up
2. Prevent unintentional RF emissions
3. Prepare spacecraft for atmospheric reentry

### 3.2 Energy Sources to Passivate

| Source | Initial State | Passivated State | Method |
|--------|---------------|------------------|--------|
| Li-ion battery | 50-100% SOC | < 30% SOC | Controlled discharge |
| Solar array | Generating power | Disconnected | Disable charging |
| RF transmitter | Active | Disabled | Software command |
| Momentum | Variable | Passive | Disable actuators |

## 4. Passivation Procedures

### 4.1 Commanded Passivation Sequence

**Pre-Passivation (T-7 days):**

| Step | Action | Verification |
|------|--------|-------------|
| 1 | Announce EOL to amateur community | Confirmed on AMSAT-BB |
| 2 | Dump all stored telemetry | Data archived |
| 3 | Verify ground station ready | Checklist complete |
| 4 | Final configuration review | Approved by MD |

**Passivation Day (T-0):**

| Step | Time | Action | Verification |
|------|------|--------|-------------|
| 1 | T-4h | Final status assessment | All systems reviewed |
| 2 | T-2h | Transmit EOL announcement beacon | Received by community |
| 3 | T-1h | Transmit final telemetry | Data archived |
| 4 | T-30m | Transmit "goodbye" beacon | Received |
| 5 | T-15m | Disable battery charging | Confirmed via telemetry |
| 6 | T-10m | Enable maximum loads (drain battery) | Current increased |
| 7 | T-5m | Disable RF transmitter | No further beacons |
| 8 | T-0 | Deploy drag device (if equipped) | Visual (if possible) |

**Post-Passivation:**

| Step | Action |
|------|--------|
| 1 | Monitor for any unexpected transmissions |
| 2 | Archive all final data |
| 3 | Update Space Track with EOL status |
| 4 | Notify regulatory authorities |
| 5 | Final mission report |

### 4.2 Autonomous Passivation Sequence

Triggered automatically after 30 days without valid ground contact:

| Step | Time | Action |
|------|------|--------|
| 1 | T+30d | Enter passivation mode |
| 2 | T+30d+1h | Disable battery charging |
| 3 | T+30d+2h | Enable maximum loads |
| 4 | T+30d+4h | Transmit final beacon (if power available) |
| 5 | T+30d+6h | Disable RF transmitter |
| 6 | T+30d+8h | Deploy drag device |

### 4.3 Emergency Passivation

For critical anomaly requiring immediate passivation:

| Step | Action |
|------|--------|
| 1 | Assess situation, confirm passivation required |
| 2 | Execute abbreviated passivation sequence |
| 3 | Disable RF immediately |
| 4 | Deploy drag device if possible |
| 5 | Document actions taken |

## 5. Deorbit Provisions

### 5.1 Passive Deorbit

SMART-QSO relies on atmospheric drag for deorbit:

| Parameter | Value |
|-----------|-------|
| Initial altitude | 400-500 km |
| Ballistic coefficient (body) | 33 kg/m² |
| Ballistic coefficient (w/drag) | 8.3 kg/m² |
| Deorbit time (no drag device) | 15 years from 500 km |
| Deorbit time (with drag device) | 4.5 years from 500 km |

### 5.2 Drag Device

| Parameter | Specification |
|-----------|---------------|
| Type | Deployable tape tether or drag sail |
| Deployed area | ~0.04 m² effective |
| Deployment trigger | Ground command or autonomous |
| Backup | Autonomous deployment on contact loss |

### 5.3 Deorbit Verification

| Activity | Method |
|----------|--------|
| Track orbital decay | TLE monitoring via Space Track |
| Predict reentry | GMAT/STK propagation |
| Announce reentry | AMSAT-BB, social media |
| Confirm reentry | No radar returns |

## 6. Final Transmissions

### 6.1 EOL Beacon Content

Final beacon transmitted before passivation:

```
de [CALLSIGN]: SMART-QSO EOL - Mission Complete
Thank you to all who listened! 73 de SQSO-1
Final stats: [days] days, [beacons] beacons, [QSOs] QSOs
T=[timestamp] EOL
```

### 6.2 Goodbye Beacon

Last beacon before RF disable:

```
de [CALLSIGN]: Goodbye from SMART-QSO
[AI-generated farewell message]
73 and thanks for all the QSOs!
```

## 7. Regulatory Compliance

### 7.1 Notifications

| Recipient | Timing | Content |
|-----------|--------|---------|
| FCC | 30 days pre-EOL | EOL notification |
| IARU | 30 days pre-EOL | Frequency release |
| Space Track | At EOL | Status update |
| AMSAT | 7 days pre-EOL | Community announcement |

### 7.2 License Termination

| Step | Action |
|------|--------|
| 1 | Confirm no RF emissions post-passivation |
| 2 | File final mission report with FCC |
| 3 | Request license termination |
| 4 | Archive all regulatory correspondence |

## 8. Data Archival

### 8.1 Final Data Products

| Product | Format | Archive Location |
|---------|--------|------------------|
| All telemetry | JSON | Project archive + NASA PDS |
| Beacon audio | WAV | Project archive |
| Beacon text | Text | Project archive + public |
| Analysis reports | PDF | Project archive |
| Lessons learned | Document | Public |

### 8.2 Archive Duration

| Category | Retention |
|----------|-----------|
| Engineering data | 10 years minimum |
| Science data | Permanent (public archive) |
| Regulatory docs | Per FCC requirements |

## 9. Post-EOL Activities

### 9.1 Mission Closeout

| Activity | Owner | Timeline |
|----------|-------|----------|
| Final mission report | PM | EOL + 30 days |
| Lessons learned | Systems Engineer | EOL + 60 days |
| Data archive verification | Data Manager | EOL + 30 days |
| Regulatory closeout | PM | EOL + 90 days |
| Publication of results | Science Lead | EOL + 6 months |

### 9.2 Reentry Tracking

| Phase | Activity |
|-------|----------|
| Pre-reentry | Monitor TLE decay |
| Reentry prediction | Announce predicted date |
| Reentry day | Track final orbits |
| Post-reentry | Confirm reentry, close tracking |

## 10. Contingencies

### 10.1 Drag Device Failure

If drag device fails to deploy:
1. Attempt command retry if communication available
2. Accept longer deorbit time (still < 25 years)
3. Document in final report

### 10.2 Communication Lost Before Passivation

Autonomous passivation will execute:
1. Monitor for any beacons indicating autonomous sequence
2. Document estimated passivation completion
3. Continue tracking for reentry prediction

### 10.3 Partial Passivation

If passivation incomplete:
1. Document remaining stored energy
2. Assess break-up risk (low for CubeSat)
3. Report to appropriate authorities if required

## 11. Success Criteria

| Criterion | Metric |
|-----------|--------|
| Passivation complete | All stored energy sources addressed |
| RF disabled | No emissions detected post-passivation |
| Drag device deployed | Visual confirmation or expected decay rate |
| Data archived | All products in permanent archive |
| Regulatory complete | All notifications filed |

---

## Appendix A: EOL Checklist

- [ ] Announce EOL to community
- [ ] Dump all stored data
- [ ] Transmit EOL announcement beacon
- [ ] Transmit final telemetry
- [ ] Transmit goodbye beacon
- [ ] Disable battery charging
- [ ] Enable maximum loads
- [ ] Confirm battery discharge
- [ ] Disable RF transmitter
- [ ] Deploy drag device
- [ ] Verify no RF emissions
- [ ] Update Space Track status
- [ ] File regulatory notifications
- [ ] Archive all data
- [ ] Complete final report
- [ ] Publish lessons learned

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
