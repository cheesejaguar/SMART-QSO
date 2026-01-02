# Launch Site Operations Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-OPS-004
**Revision:** A
**Date:** 2026-01-02
**Reference:** NASA Ames SmallSat Launch Heritage

---

## 1. Purpose

This document defines the launch site operations for the SMART-QSO CubeSat, covering activities from arrival at the launch site through integration with the launch vehicle.

## 2. Launch Campaign Overview

### 2.1 Campaign Timeline

| Phase | Duration | Activities |
|-------|----------|------------|
| Arrival/Setup | Day 1-2 | Unpack, GSE setup, initial inspection |
| Pre-integration | Day 3-5 | Functional test, battery charge, final config |
| Integration | Day 6-7 | Dispenser integration, fit check |
| Closeout | Day 8 | Final inspection, RBF installation |
| Standby | Day 9+ | Launch readiness, contingency |

### 2.2 Team Composition

| Role | Count | Responsibilities |
|------|-------|------------------|
| Mission Manager | 1 | Overall campaign authority |
| Systems Engineer | 1 | Technical lead |
| Integration Engineer | 1 | Mechanical integration |
| Flight Software Engineer | 1 | SW configuration |
| Quality Assurance | 1 | Inspection, documentation |
| Safety Officer | 1 | Safety oversight |

## 3. Shipping and Receiving

### 3.1 Shipping Configuration

| Item | Configuration |
|------|---------------|
| Spacecraft | In protective container |
| Battery | Charged to 30% SOC |
| RBF pins | Installed |
| Deployment switches | Constrained |
| Antennas | Stowed |
| Documentation | Shipped separately |

### 3.2 Shipping Container

| Requirement | Specification |
|-------------|---------------|
| Type | Pelican 1600 or equivalent |
| Cushioning | Closed-cell foam, custom cut |
| Humidity | Desiccant packs included |
| Shock indicators | ShockWatch or equivalent |
| Tilt indicators | TiltWatch or equivalent |

### 3.3 Receiving Inspection

| Check | Criteria | Result |
|-------|----------|--------|
| Container condition | No visible damage | ______ |
| Shock indicator | Not triggered | ______ |
| Tilt indicator | Not triggered | ______ |
| Seal intact | Tamper-evident seal OK | ______ |
| Documentation | All docs present | ______ |

## 4. Facility Requirements

### 4.1 Cleanroom

| Parameter | Requirement |
|-----------|-------------|
| Class | ISO 8 (Class 100,000) minimum |
| Temperature | 20-25°C |
| Humidity | 30-60% RH |
| ESD | ANSI/ESD S20.20 compliant |
| Access | Controlled, logged |

### 4.2 Equipment

| Equipment | Purpose |
|-----------|---------|
| ESD workstation | Spacecraft handling |
| Power supply | Battery charging, functional test |
| Test equipment | Multimeter, oscilloscope |
| RF receiver | Beacon verification |
| Laptop | Flight SW configuration |
| Camera | Documentation |

### 4.3 Consumables

| Item | Quantity |
|------|----------|
| IPA wipes | 1 container |
| Lint-free wipes | 1 box |
| Gloves (nitrile) | 2 boxes |
| ESD bags | 10 |
| Kapton tape | 1 roll |

## 5. Pre-Integration Activities

### 5.1 Initial Inspection

| Step | Action | Result |
|------|--------|--------|
| 1 | Remove from shipping container | ______ |
| 2 | Visual inspection - all faces | ______ |
| 3 | Photograph all faces | ______ |
| 4 | Check RBF pins installed | ______ |
| 5 | Check deployment switches | ______ |
| 6 | Check antenna stowage | ______ |
| 7 | Document any anomalies | ______ |

### 5.2 Functional Test

| Test | Procedure | Result |
|------|-----------|--------|
| Power on | Apply external power | ______ |
| Boot verification | Check debug console | ______ |
| Telemetry | Verify all channels | ______ |
| Beacon test | Receive and decode | ______ |
| Command test | Send test commands | ______ |
| ADCS test | Verify sensor readings | ______ |
| Power off | Verify clean shutdown | ______ |

### 5.3 Battery Operations

| Step | Action | Criteria | Result |
|------|--------|----------|--------|
| 1 | Measure initial SOC | Record | ______ % |
| 2 | Connect charger | CC-CV mode | ______ |
| 3 | Charge to flight SOC | 50-70% | ______ % |
| 4 | Verify cell balance | < 50 mV | ______ mV |
| 5 | Disconnect charger | Verify | ______ |
| 6 | Record final SOC | Document | ______ % |

### 5.4 Flight Configuration

| Item | Configuration | Verified |
|------|---------------|----------|
| Flight software | v1.0.0 loaded | ______ |
| Beacon callsign | [CALLSIGN] set | ______ |
| Beacon interval | 60 seconds | ______ |
| ADCS mode | Detumble on deploy | ______ |
| Safe mode thresholds | Default | ______ |
| Time | UTC synchronized | ______ |

## 6. Dispenser Integration

### 6.1 Pre-Integration Checklist

| Item | Status |
|------|--------|
| Spacecraft functional test complete | ______ |
| Battery charged to flight level | ______ |
| RBF pins verified | ______ |
| Deployment switches verified | ______ |
| Antennas stowed and secured | ______ |
| Mass properties verified | ______ |
| Photos taken | ______ |

### 6.2 Integration Procedure

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Verify dispenser is clean | Visual |
| 2 | Position spacecraft at dispenser | Alignment |
| 3 | Slide spacecraft into rails | Smooth motion |
| 4 | Verify deployment switches | Actuated |
| 5 | Verify door clearance | No interference |
| 6 | Secure access panel | Fasteners |
| 7 | Install RBF access | If required |

### 6.3 Post-Integration Verification

| Check | Criteria | Result |
|-------|----------|--------|
| Deployment switch continuity | Closed | ______ |
| RBF access | Accessible | ______ |
| Door clearance | > 1 mm | ______ |
| Rail engagement | Full length | ______ |
| Photographs | All sides | ______ |

## 7. Final Closeout

### 7.1 Closeout Inspection

| Item | Verification | Result |
|------|--------------|--------|
| All fasteners torqued | Witness marks | ______ |
| No FOD present | Visual | ______ |
| RBF pins installed | Verify | ______ |
| Access panels secured | Verify | ______ |
| Documentation complete | Review | ______ |

### 7.2 RBF Pin Removal

| Step | Action | Time |
|------|--------|------|
| 1 | Verify launch authorization | L-TBD |
| 2 | Access spacecraft | Coordinate with LV |
| 3 | Remove RBF pin(s) | Document |
| 4 | Verify RF silence | Confirm |
| 5 | Secure access | Close out |
| 6 | Report complete | Mission Manager |

## 8. Launch Readiness

### 8.1 Go/No-Go Criteria

| Criterion | Status Required |
|-----------|-----------------|
| Functional test passed | GO |
| Battery SOC > 50% | GO |
| RBF removed | GO |
| Dispenser integration complete | GO |
| Documentation complete | GO |
| Range safety approval | GO |
| Ground station ready | GO |

### 8.2 Contingency Procedures

| Situation | Response |
|-----------|----------|
| Functional test fail | Troubleshoot, retest |
| Low battery | Charge, verify |
| Physical damage | Assess, document, disposition |
| Integration issue | Coordinate with LV provider |
| Launch delay | Safe configuration, monitor |

## 9. Post-Launch Activities

### 9.1 Deployment Verification

| Event | Expected Time | Verification |
|-------|---------------|--------------|
| Separation | T+0 min | LV confirmation |
| Antenna deploy | T+30 min | TLE tracking |
| First beacon | T+32 min | Ground station |
| Detumble complete | T+90 min | Beacon telemetry |

### 9.2 Pack-Out

| Item | Disposition |
|------|-------------|
| GSE | Return ship |
| Documentation | Archive |
| Unused consumables | Dispose/return |
| Shipping containers | Return ship |

## 10. Documentation

### 10.1 Required Records

| Document | Purpose |
|----------|---------|
| Shipping log | Chain of custody |
| Inspection photos | As-shipped condition |
| Functional test data | Pre-integration verification |
| Integration photos | Configuration documentation |
| Closeout checklist | Final configuration |
| RBF removal record | Launch configuration |

### 10.2 Data Retention

| Record | Retention |
|--------|-----------|
| Test data | Mission lifetime + 3 years |
| Photos | Mission lifetime + 3 years |
| Checklists | Mission lifetime + 3 years |
| Anomaly reports | Permanent |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
