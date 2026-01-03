# SMART-QSO Operator Training Plan

**Document ID**: SQSO-OTP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the operator training program for SMART-QSO mission operations per NPR 7123.1 validation requirements. Training ensures operators can safely and effectively conduct mission operations.

## 2. Applicable Documents

| Document ID | Title |
|-------------|-------|
| NPR 7123.1 | NASA Systems Engineering Processes and Requirements |
| NPR 8705.6 | Safety and Mission Assurance Audits, Reviews, and Assessments |
| SQSO-MOP-001 | Mission Operations Plan |
| SQSO-CMD-001 | Command and Telemetry Database |

## 3. Training Philosophy

Following NASA Ames SmallSat heritage (GeneSat, PharmaSat, O/OREOS, EcAMSat), the training program emphasizes:

1. **Hands-on experience** with flight-like systems
2. **Scenario-based training** using realistic mission situations
3. **Graduated complexity** from basic to anomaly response
4. **Certification requirements** before operational duty

## 4. Training Organization

### 4.1 Training Roles

| Role | Responsibilities |
|------|------------------|
| Training Lead | Develop curriculum, schedule training, maintain records |
| Mission Director | Certify operators, approve training completion |
| Subject Matter Experts | Deliver specialized training modules |
| Operators | Complete training, maintain proficiency |

### 4.2 Operator Positions

| Position | Code | Description |
|----------|------|-------------|
| Mission Director | MD | Overall mission authority |
| Flight Controller | FC | Real-time spacecraft operations |
| Ground Controller | GC | Ground station operations |
| Analyst | AN | Telemetry analysis and trending |

## 5. Training Curriculum

### 5.1 Module 1: Mission Overview (2 hours)

**Objective**: Understand SMART-QSO mission objectives and architecture

**Topics**:
- [ ] Mission objectives and success criteria
- [ ] Spacecraft bus overview
- [ ] Payload description (AI-assisted amateur radio)
- [ ] Orbital parameters and coverage
- [ ] Ground segment architecture
- [ ] Mission phases and timeline

**Assessment**: Written quiz (80% passing)

### 5.2 Module 2: Spacecraft Systems (4 hours)

**Objective**: Understand spacecraft subsystems and operations

**Topics**:
- [ ] Electrical Power System (EPS)
  - Battery management
  - Solar array operation
  - Power modes (SAFE/IDLE/ACTIVE)
- [ ] On-Board Computer (OBC)
  - FreeRTOS task architecture
  - Watchdog operation
  - Fault management
- [ ] AI Payload (Jetson)
  - Power-gated operation
  - Inference scheduling
  - Fallback modes
- [ ] RF Subsystem
  - Transponder operation
  - Beacon generation
  - AX.25 protocol

**Assessment**: Written exam (80% passing)

### 5.3 Module 3: Ground Station Operations (4 hours)

**Objective**: Operate ground station equipment proficiently

**Topics**:
- [ ] Ground station hardware
  - Antenna systems
  - Transceiver operation
  - TNC configuration
- [ ] Ground software
  - Beacon decoder operation
  - Telemetry display usage
  - Command generation
- [ ] Pass planning
  - Orbit prediction
  - Visibility windows
  - Doppler compensation
- [ ] Link budget and margins

**Assessment**: Practical demonstration

### 5.4 Module 4: Telemetry Analysis (4 hours)

**Objective**: Interpret spacecraft telemetry and identify anomalies

**Topics**:
- [ ] Telemetry frame structure
- [ ] Health channel interpretation
  - Battery voltage/current/SOC
  - Temperatures
  - Mode status
- [ ] Trend analysis techniques
- [ ] Limit checking and alarms
- [ ] Anomaly identification
- [ ] Reporting procedures

**Assessment**: Scenario-based practical

### 5.5 Module 5: Commanding Procedures (4 hours)

**Objective**: Execute commanding operations safely and correctly

**Topics**:
- [ ] Command authority and constraints
- [ ] Command verification (two-person rule)
- [ ] Real-time commanding
- [ ] Stored command sequences
- [ ] Command response verification
- [ ] Emergency commanding

**Assessment**: Supervised commanding exercise

### 5.6 Module 6: Anomaly Response (4 hours)

**Objective**: Respond to spacecraft anomalies effectively

**Topics**:
- [ ] Anomaly classification
- [ ] Initial response procedures
- [ ] Escalation protocols
- [ ] Anomaly investigation
- [ ] Corrective action development
- [ ] Anomaly closure process
- [ ] Lessons learned

**Assessment**: Tabletop anomaly exercise

### 5.7 Module 7: Emergency Procedures (2 hours)

**Objective**: Execute emergency procedures under pressure

**Topics**:
- [ ] Loss of communication response
- [ ] Safe mode recovery
- [ ] Battery emergency procedures
- [ ] Thermal emergency procedures
- [ ] Collision avoidance
- [ ] End-of-life procedures

**Assessment**: Emergency drill (timed)

## 6. Practical Training

### 6.1 Simulation Sessions

| Session | Duration | Description |
|---------|----------|-------------|
| SIM-1 | 4 hrs | Nominal pass operations |
| SIM-2 | 4 hrs | Multiple pass sequence |
| SIM-3 | 4 hrs | Anomaly injection (minor) |
| SIM-4 | 4 hrs | Anomaly injection (major) |
| SIM-5 | 8 hrs | Full shift simulation |
| SIM-6 | 4 hrs | Emergency procedures |

### 6.2 Simulation Scenarios

#### SIM-1: Nominal Operations
```
SCENARIO: SIM-1-NOM
DURATION: 4 hours (simulated 24 hours)
COMPLEXITY: Basic

OBJECTIVES:
1. Execute nominal pass operations
2. Collect and analyze telemetry
3. Verify beacon reception
4. Practice handoff procedures

INJECT SEQUENCE:
T+0:00  Ground station activation
T+0:30  First pass - telemetry collection
T+1:00  Data analysis period
T+1:30  Second pass - commanding
T+2:30  Third pass - routine ops
T+3:30  Shift handoff
T+4:00  End simulation

SUCCESS CRITERIA:
- All passes executed per procedure
- Telemetry logged correctly
- No procedural errors
```

#### SIM-3: Minor Anomaly
```
SCENARIO: SIM-3-ANO
DURATION: 4 hours
COMPLEXITY: Medium

OBJECTIVES:
1. Detect anomaly from telemetry
2. Execute anomaly response
3. Implement workaround
4. Document and report

INJECT SEQUENCE:
T+0:00  Normal operations
T+1:00  INJECT: Battery heater stuck ON
        - Battery temperature rising
        - SOC decreasing faster than expected
T+1:30  Expected detection
T+2:00  Expected response initiation
T+3:00  Expected workaround implemented
T+4:00  End simulation

SUCCESS CRITERIA:
- Anomaly detected within 30 minutes
- Correct procedure executed
- Anomaly reported per protocol
```

## 7. Certification Requirements

### 7.1 Initial Certification

| Position | Requirements |
|----------|--------------|
| Flight Controller | Modules 1-7, SIM-1 through SIM-5, 80% all assessments |
| Ground Controller | Modules 1-3, SIM-1, SIM-2, 80% all assessments |
| Analyst | Modules 1-4, SIM-1, SIM-3, 80% all assessments |
| Mission Director | All modules, all simulations, 90% all assessments |

### 7.2 Certification Process

1. Complete all required training modules
2. Pass all written assessments (≥80%, MD ≥90%)
3. Complete all required simulation sessions
4. Demonstrate proficiency to evaluator
5. Training Lead recommends certification
6. Mission Director approves certification

### 7.3 Certification Record

```
SMART-QSO OPERATOR CERTIFICATION

Name: ___________________________
Position: _______________________
Date: __________________________

MODULES COMPLETED:
[ ] Module 1: Mission Overview      Date: ___ Score: ___%
[ ] Module 2: Spacecraft Systems    Date: ___ Score: ___%
[ ] Module 3: Ground Station Ops    Date: ___ Score: ___%
[ ] Module 4: Telemetry Analysis    Date: ___ Score: ___%
[ ] Module 5: Commanding            Date: ___ Score: ___%
[ ] Module 6: Anomaly Response      Date: ___ Score: ___%
[ ] Module 7: Emergency Procedures  Date: ___ Score: ___%

SIMULATIONS COMPLETED:
[ ] SIM-1  Date: ___ Evaluator: ___ Result: ___
[ ] SIM-2  Date: ___ Evaluator: ___ Result: ___
[ ] SIM-3  Date: ___ Evaluator: ___ Result: ___
[ ] SIM-4  Date: ___ Evaluator: ___ Result: ___
[ ] SIM-5  Date: ___ Evaluator: ___ Result: ___
[ ] SIM-6  Date: ___ Evaluator: ___ Result: ___

CERTIFICATION:
Training Lead: _____________ Date: _______
Mission Director: __________ Date: _______

Certification Valid Until: _______________
```

## 8. Proficiency Maintenance

### 8.1 Currency Requirements

| Position | Requirement |
|----------|-------------|
| All positions | 1 simulation per quarter |
| Flight Controller | 4 live passes per month |
| Mission Director | Annual recertification |

### 8.2 Recertification

- Required annually for all positions
- Abbreviated module review (2 hours)
- One simulation session
- Emergency procedure review

## 9. Training Records

### 9.1 Record Retention

| Record Type | Retention Period |
|-------------|------------------|
| Certification records | Mission duration + 5 years |
| Training materials | Mission duration + 2 years |
| Assessment results | Mission duration + 5 years |
| Simulation logs | Mission duration + 2 years |

### 9.2 Training Database

All training records maintained in:
- Primary: Project document repository
- Backup: Training Lead personal archive

## 10. Training Schedule

### 10.1 Pre-Launch Training Timeline

| Phase | Duration | Activities |
|-------|----------|------------|
| L-6 months | 2 weeks | Module 1-2 training |
| L-5 months | 2 weeks | Module 3-4 training |
| L-4 months | 2 weeks | Module 5-7 training |
| L-3 months | 4 weeks | Simulation sessions |
| L-2 months | 2 weeks | Certification exams |
| L-1 month | 2 weeks | Dress rehearsals |
| L-1 week | 1 week | Final readiness review |

### 10.2 Training Resources

| Resource | Availability |
|----------|--------------|
| Ground station simulator | As scheduled |
| Spacecraft simulator | As scheduled |
| Training room | Reserved for modules |
| Subject matter experts | By appointment |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
