# SMART-QSO Contingency Procedures

**Document ID**: SQSO-CP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines contingency procedures for responding to off-nominal spacecraft conditions. These procedures ensure rapid, coordinated response to anomalies while maintaining spacecraft safety.

## 2. Applicable Documents

| Document ID | Title |
|-------------|-------|
| SQSO-MOP-001 | Mission Operations Plan |
| SQSO-MRC-001 | Mission Rules and Constraints |
| SQSO-SHA-001 | Software Hazard Analysis |

## 3. Contingency Response Framework

### 3.1 Response Levels

| Level | Condition | Response Time | Authority |
|-------|-----------|---------------|-----------|
| RED | Loss of spacecraft | Immediate | Mission Director |
| ORANGE | Critical system failure | < 1 hour | Mission Director |
| YELLOW | Degraded operations | < 4 hours | Flight Controller |
| GREEN | Minor anomaly | < 24 hours | On-console operator |

### 3.2 Response Team Structure

```
                    ┌─────────────────┐
                    │ Mission Director│
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────┴───────┐   ┌───────┴───────┐   ┌───────┴───────┐
│Flight Director│   │Systems Lead   │   │Safety Officer │
└───────┬───────┘   └───────────────┘   └───────────────┘
        │
┌───────┴───────┐
│Flight Controller│
└───────────────┘
```

## 4. Contingency Procedures

---

### CP-001: Loss of Signal (LOS)

**Response Level**: YELLOW → ORANGE (if prolonged)

**Symptoms**:
- No telemetry received during expected pass
- No beacon detected
- Weak or no carrier signal

**Immediate Actions** (within pass):
```
1. VERIFY ground station status
   [ ] Antenna tracking correct
   [ ] Receiver tuned correctly
   [ ] TNC receiving data
   [ ] No local interference

2. IF ground station nominal:
   [ ] Increase receiver bandwidth
   [ ] Attempt wide-band search ±10 kHz
   [ ] Check TLE currency (propagate from last known)

3. DOCUMENT:
   [ ] Time of last known contact
   [ ] Pass geometry (AOS/LOS, max elevation)
   [ ] Ground station configuration
```

**Follow-up Actions** (within 24 hours):
```
4. COORDINATE with backup ground stations
   [ ] Request coverage of next passes
   [ ] Share TLE and configuration

5. ANALYZE possible causes:
   [ ] Orbit decay (check TLE age)
   [ ] Tumbling (doppler signature)
   [ ] Safe mode (reduced TX power)
   [ ] Hardware failure

6. IF no contact after 3 passes:
   [ ] Escalate to ORANGE
   [ ] Notify Mission Director
   [ ] Convene anomaly team
```

**Recovery Criteria**:
- Telemetry received from spacecraft
- Beacon decoded successfully
- Two-way communication established

---

### CP-002: Unexpected Safe Mode Entry

**Response Level**: YELLOW

**Symptoms**:
- Telemetry shows STATE = SAFE
- Reduced beacon content
- Lower TX power
- No AI-generated content

**Immediate Actions**:
```
1. DO NOT COMMAND safe mode release
   (Allow investigation first)

2. COLLECT diagnostic data:
   [ ] Boot reason code
   [ ] Fault counters
   [ ] Last known state before safe mode
   [ ] Time of transition

3. DOCUMENT in anomaly log:
   [ ] Timeline of events
   [ ] Telemetry at transition
   [ ] Any correlated events
```

**Analysis Actions**:
```
4. REVIEW fault indicators:
   [ ] Reset count increased?
   [ ] Fault flags set?
   [ ] Battery SOC at transition?
   [ ] Temperature anomalies?

5. DETERMINE cause:
   [ ] Ground commanded?
   [ ] Low SOC trigger?
   [ ] Watchdog reset?
   [ ] Thermal limit?
   [ ] Software fault?

6. DEVELOP recovery plan:
   [ ] Document root cause
   [ ] Define safe mode exit criteria
   [ ] Get Mission Director approval
```

**Recovery Procedure**:
```
7. VERIFY exit conditions:
   [ ] Root cause understood
   [ ] SOC > 40%
   [ ] Temperatures nominal
   [ ] No active faults
   [ ] MD approval obtained

8. COMMAND safe mode release:
   [ ] Two operators on console
   [ ] Send RELEASE_SAFE command
   [ ] Monitor transition
   [ ] Verify IDLE state achieved

9. MONITOR for 24 hours:
   [ ] No repeat safe mode entry
   [ ] Telemetry nominal
   [ ] Document recovery complete
```

---

### CP-003: Battery Undervoltage

**Response Level**: YELLOW → ORANGE (if critical)

**Symptoms**:
- Battery voltage < 3.3V
- SOC < 30%
- Possible safe mode entry

**Immediate Actions**:
```
1. VERIFY telemetry accuracy:
   [ ] Multiple readings consistent?
   [ ] Sensor health nominal?

2. IF confirmed undervoltage:
   [ ] Stop all non-essential commanding
   [ ] Spacecraft should auto-enter SAFE mode at SOC < 20%

3. CALCULATE power budget:
   [ ] Current SOC
   [ ] Solar array status (sun/eclipse)
   [ ] Current load estimate
   [ ] Time to next sun
```

**Analysis Actions**:
```
4. DETERMINE cause:
   [ ] Excessive load (AI running when shouldn't)?
   [ ] Solar array degradation?
   [ ] Battery capacity loss?
   [ ] Unplanned power drain?

5. PROJECT recovery:
   [ ] Orbit position (time to sunlight)
   [ ] Expected charge rate
   [ ] Time to SOC > 40%
```

**Recovery Procedure**:
```
6. WAIT for passive charging:
   [ ] Monitor SOC trend
   [ ] Verify charge current positive in sun

7. WHEN SOC > 40%:
   [ ] Resume monitoring
   [ ] Consider reduced operations tempo

8. LONG-TERM:
   [ ] Update power budget model
   [ ] Adjust operations profile if needed
   [ ] Document in lessons learned
```

---

### CP-004: Thermal Limit Exceeded

**Response Level**: YELLOW

**Symptoms**:
- Component temperature above limit
- Telemetry shows thermal alarm
- Possible throttling or shutdown

**Immediate Actions**:
```
1. IDENTIFY affected component:
   [ ] OBC (limit: +85°C)
   [ ] Jetson (limit: +70°C)
   [ ] Battery (limit: +45°C)
   [ ] RF PA (limit: +60°C)

2. VERIFY spacecraft response:
   [ ] Auto-throttling activated?
   [ ] Component shutdown?
   [ ] Safe mode entry?

3. DO NOT command operations that add thermal load
```

**Analysis Actions**:
```
4. CORRELATE with conditions:
   [ ] Orbit position (sun angle)
   [ ] Recent operations (AI burst?)
   [ ] Eclipse exit timing

5. PROJECT thermal trajectory:
   [ ] Time to eclipse (cooling)
   [ ] Expected peak temperature
```

**Recovery Procedure**:
```
6. IF Jetson over-temp:
   [ ] Confirm Jetson powered off
   [ ] Wait for natural cooling
   [ ] Resume only when < 60°C

7. IF battery over-temp:
   [ ] Reduce operations
   [ ] Monitor closely
   [ ] Consider adjusted duty cycle

8. DOCUMENT and update thermal model
```

---

### CP-005: Watchdog Reset

**Response Level**: GREEN → YELLOW (if repeated)

**Symptoms**:
- Boot reason = WDT_RESET
- Reset counter incremented
- Fresh boot telemetry

**Immediate Actions**:
```
1. VERIFY reset occurred:
   [ ] Check boot reason code
   [ ] Confirm reset counter increased
   [ ] Note current state (should be SAFE)

2. COLLECT data:
   [ ] Last telemetry before reset
   [ ] Time of reset
   [ ] Operations at time of reset
```

**Analysis Actions**:
```
3. ANALYZE for patterns:
   [ ] Single event or repeated?
   [ ] Correlated with specific operations?
   [ ] Memory/load at time of reset?

4. IF repeated resets:
   [ ] Escalate to YELLOW
   [ ] Reduce operations
   [ ] Collect extended diagnostics
```

**Recovery Procedure**:
```
5. IF single reset:
   [ ] Document in log
   [ ] Resume normal operations
   [ ] Monitor for recurrence

6. IF repeated resets (>3 in 24 hours):
   [ ] Maintain SAFE mode
   [ ] Request additional telemetry
   [ ] Consider software issue
   [ ] Prepare for possible software upload
```

---

### CP-006: AI Payload Failure

**Response Level**: GREEN

**Symptoms**:
- No AI-generated beacon content
- Jetson timeout in telemetry
- Fallback beacon templates in use

**Immediate Actions**:
```
1. VERIFY failure:
   [ ] Beacon shows template (no AI content)
   [ ] Jetson status = OFFLINE
   [ ] Fallback mode active

2. CONFIRM graceful degradation:
   [ ] OBC operating normally
   [ ] Basic beacons transmitting
   [ ] Telemetry collection continues
```

**Analysis Actions**:
```
3. DETERMINE cause:
   [ ] Power-gated off (normal during eclipse)?
   [ ] SOC too low for AI?
   [ ] Jetson hardware failure?
   [ ] Software crash?

4. IF hardware failure suspected:
   [ ] Continue in fallback mode
   [ ] Document for mission analysis
```

**Recovery Procedure**:
```
5. IF power/thermal cause:
   [ ] Wait for favorable conditions
   [ ] Jetson should auto-restart

6. IF persistent failure:
   [ ] Accept fallback operations
   [ ] Mission success not dependent on AI
   [ ] Update ConOps for fallback mode
```

---

### CP-007: Command Rejection

**Response Level**: GREEN

**Symptoms**:
- Command sent but no response
- NACK received
- State unchanged

**Immediate Actions**:
```
1. VERIFY command sent correctly:
   [ ] Command syntax correct
   [ ] Parameters in range
   [ ] Checksum valid

2. CHECK uplink conditions:
   [ ] Elevation adequate (>20°)?
   [ ] No interference?
   [ ] TX power sufficient?

3. RETRY once if conditions good
```

**Analysis Actions**:
```
4. ANALYZE rejection:
   [ ] Link budget marginal?
   [ ] Command during wrong mode?
   [ ] Sequence number issue?

5. IF persistent rejection:
   [ ] Wait for better pass geometry
   [ ] Check spacecraft mode
```

**Recovery Procedure**:
```
6. RETRY on subsequent pass:
   [ ] Verify higher elevation available
   [ ] Increase TX power if available
   [ ] Confirm command syntax

7. DOCUMENT all attempts
```

---

### CP-008: Collision Avoidance Maneuver

**Response Level**: ORANGE

**Symptoms**:
- Conjunction warning received from SSA
- Probability of collision above threshold

**Immediate Actions**:
```
1. RECEIVE warning data:
   [ ] Time of closest approach (TCA)
   [ ] Probability of collision (Pc)
   [ ] Miss distance estimate

2. IF Pc > 1e-4:
   [ ] Notify Mission Director immediately
   [ ] Convene operations team
```

**Analysis Actions**:
```
3. ASSESS options:
   [ ] SMART-QSO has no propulsion
   [ ] Options: Accept risk or coordinate

4. COORDINATE with 18 SDS / Space-Track:
   [ ] Confirm conjunction data
   [ ] Request updated TLE
   [ ] Monitor for updates
```

**Response**:
```
5. IF collision risk accepted:
   [ ] Document decision rationale
   [ ] Prepare for possible impact
   [ ] Alert backup tracking

6. IF collision occurs:
   [ ] Execute LOS procedures
   [ ] Report to authorities
```

---

## 5. Escalation Matrix

| Condition | Initial | 1 Hour | 4 Hours | 24 Hours |
|-----------|---------|--------|---------|----------|
| LOS single pass | GREEN | GREEN | YELLOW | ORANGE |
| Safe mode entry | YELLOW | YELLOW | YELLOW | GREEN* |
| Battery < 20% | YELLOW | YELLOW | ORANGE | RED |
| Repeated resets | GREEN | YELLOW | YELLOW | ORANGE |
| Thermal over-limit | YELLOW | YELLOW | GREEN* | GREEN* |

*If condition resolved

## 6. Emergency Contacts

| Role | Primary | Backup |
|------|---------|--------|
| Mission Director | TBD | TBD |
| Flight Director | TBD | TBD |
| Ground Station | TBD | TBD |
| Safety Officer | TBD | TBD |

## 7. Quick Reference Cards

### 7.1 LOS Quick Reference
```
┌─────────────────────────────────────────────┐
│ LOSS OF SIGNAL - QUICK REFERENCE            │
├─────────────────────────────────────────────┤
│ 1. Check ground station (antenna, receiver) │
│ 2. Widen search bandwidth ±10 kHz           │
│ 3. Verify TLE current                       │
│ 4. Contact backup stations                  │
│ 5. If 3 passes LOS → Escalate               │
└─────────────────────────────────────────────┘
```

### 7.2 Safe Mode Quick Reference
```
┌─────────────────────────────────────────────┐
│ SAFE MODE - QUICK REFERENCE                 │
├─────────────────────────────────────────────┤
│ 1. DO NOT command release immediately       │
│ 2. Collect boot reason, faults, SOC         │
│ 3. Determine root cause                     │
│ 4. Get MD approval for release              │
│ 5. Two operators for release command        │
└─────────────────────────────────────────────┘
```

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
