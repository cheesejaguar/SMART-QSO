# Integration Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-INT-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7123.1 SE Process 8 (Product Integration)

---

## 1. Purpose

This Integration Plan defines the approach, sequence, and procedures for integrating the SMART-QSO CubeSat subsystems into a complete flight system, following NASA Ames Small Satellite Division heritage.

## 2. Integration Philosophy

Following GeneSat/PharmaSat/O/OREOS/EcAMSat heritage:

| Principle | Implementation |
|-----------|----------------|
| **Flatsat First** | Develop and test all software on flatsat before flight hardware |
| **Bottom-Up Integration** | Integrate from component → subsystem → system |
| **Test at Each Step** | Functional test after each integration step |
| **Early Integration** | Start integration early to find issues early |
| **Protoflight Approach** | Single flight unit tested to protoflight levels |

## 3. Integration Sequence

### 3.1 Phase 1: Flatsat Development

```
Week 1-4: Flatsat Assembly
├── OBC development board
├── EPS evaluation board
├── RF development board
├── ADCS sensor breakout
├── Jetson development kit
└── GSE interfaces
```

### 3.2 Phase 2: Software Development on Flatsat

| Activity | Duration | Exit Criteria |
|----------|----------|---------------|
| OBC software integration | 4 weeks | All modules functional |
| Subsystem interface testing | 2 weeks | All interfaces verified |
| Fault injection testing | 2 weeks | All fault responses verified |
| Day-in-the-life testing | 1 week | 72+ hour stable operation |

### 3.3 Phase 3: Flight Unit Integration

```
Week 9-16: Flight Unit Assembly
├── Structure assembly
├── EPS integration
├── OBC integration
├── RF integration
├── ADCS integration
├── Jetson integration
├── Harness integration
└── Final mechanical assembly
```

### 3.4 Integration Sequence Diagram

```
                    ┌─────────────┐
                    │  Structure  │
                    └──────┬──────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
        ┌─────────┐  ┌─────────┐  ┌─────────┐
        │   EPS   │  │ Harness │  │ Antenna │
        └────┬────┘  └────┬────┘  └────┬────┘
             │            │            │
             └────────────┼────────────┘
                          ▼
                    ┌─────────┐
                    │   OBC   │
                    └────┬────┘
                         │
         ┌───────────────┼───────────────┐
         ▼               ▼               ▼
   ┌─────────┐     ┌─────────┐     ┌─────────┐
   │   RF    │     │  ADCS   │     │ Jetson  │
   └────┬────┘     └────┬────┘     └────┬────┘
        │               │               │
        └───────────────┴───────────────┘
                        ▼
                  ┌───────────┐
                  │   Solar   │
                  │   Panels  │
                  └─────┬─────┘
                        ▼
                  ┌───────────┐
                  │  Closeout │
                  └───────────┘
```

## 4. Integration Procedures

### 4.1 Pre-Integration Requirements

| Requirement | Verification |
|-------------|-------------|
| All components flight-qualified | Test reports reviewed |
| Workmanship inspection complete | Inspection records signed |
| Cleanliness requirements met | Particle counts within spec |
| ESD procedures in place | ESD audit passed |
| Integration procedures reviewed | Procedure walkthrough complete |

### 4.2 Structure Integration

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Inspect structure for damage | Visual inspection |
| 2 | Verify dimensions | CMM measurement |
| 3 | Install standoffs and fasteners | Torque verification |
| 4 | Install deployment mechanisms | Fit check |
| 5 | Install RBF pin | Function test |

### 4.3 EPS Integration (TP-INT-001)

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Mount EPS board to structure | Torque specification |
| 2 | Connect battery | Connector seated |
| 3 | Connect solar array harness | Continuity test |
| 4 | Connect power distribution | Continuity test |
| 5 | Power-on test | All rails verified |
| 6 | Load test | Currents within spec |

### 4.4 OBC Integration (TP-INT-002)

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Mount OBC to structure | Torque specification |
| 2 | Connect power from EPS | Voltage verified |
| 3 | Load flight software | Version verified |
| 4 | Boot test | Boot sequence complete |
| 5 | Interface test to EPS | Telemetry verified |

### 4.5 RF Integration (TP-INT-003)

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Mount RF board | Torque specification |
| 2 | Connect power from EPS | Voltage verified |
| 3 | Connect antenna feedline | VSWR < 2:1 |
| 4 | Connect to OBC | Interface verified |
| 5 | RF power test (into load) | Power within spec |
| 6 | Beacon test | Decoded correctly |

### 4.6 ADCS Integration (TP-INT-004)

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Mount ADCS board | Torque specification |
| 2 | Mount magnetometers | Alignment verified |
| 3 | Mount sun sensors | FOV verified |
| 4 | Connect magnetorquers | Polarity verified |
| 5 | Connect to OBC | Telemetry verified |
| 6 | Functional test | Sensor/actuator response |

### 4.7 Jetson Integration (TP-INT-005)

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Mount Jetson module | Thermal interface verified |
| 2 | Connect power from EPS | Power gating verified |
| 3 | Connect UART to OBC | Communication verified |
| 4 | Boot test | OS boots correctly |
| 5 | Inference test | AI text generated |
| 6 | Power consumption | Within budget |

### 4.8 Antenna Integration

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Install antenna base | Torque specification |
| 2 | Stow antenna elements | Constraint verified |
| 3 | Install burn wire | Resistance measured |
| 4 | Deployment test (if safe) | Deployment verified |
| 5 | RF test | VSWR < 2:1 |

### 4.9 Final Assembly (TP-INT-006)

| Step | Activity | Verification |
|------|----------|--------------|
| 1 | Final harness routing | No pinch points |
| 2 | Tie-down all cables | Workmanship inspection |
| 3 | Install solar panels | Connections verified |
| 4 | Final mechanical closeout | Fasteners verified |
| 5 | Mass measurement | Within budget |
| 6 | CG measurement | Within spec |
| 7 | Final photography | Configuration documented |

## 5. Integration Test Matrix

| Test | Post-EPS | Post-OBC | Post-RF | Post-ADCS | Post-Jetson | Final |
|------|----------|----------|---------|-----------|-------------|-------|
| Power rails | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Boot sequence | - | ✓ | ✓ | ✓ | ✓ | ✓ |
| Beacon TX | - | - | ✓ | ✓ | ✓ | ✓ |
| ADCS sensors | - | - | - | ✓ | ✓ | ✓ |
| ADCS actuators | - | - | - | ✓ | ✓ | ✓ |
| Jetson boot | - | - | - | - | ✓ | ✓ |
| AI generation | - | - | - | - | ✓ | ✓ |
| Mode transitions | - | ✓ | ✓ | ✓ | ✓ | ✓ |
| Full functional | - | - | - | - | - | ✓ |

## 6. Integration Environment

### 6.1 Cleanroom Requirements

| Parameter | Specification |
|-----------|---------------|
| Class | 100,000 (ISO 8) minimum |
| Temperature | 20-25°C |
| Humidity | 30-60% RH |
| ESD | Controlled environment |

### 6.2 Required GSE

| Equipment | Purpose |
|-----------|---------|
| Power supply | EPS simulation |
| RF test equipment | Beacon verification |
| Multimeters | Electrical verification |
| Torque tools | Fastener installation |
| Inspection tools | Workmanship verification |
| Data acquisition | Telemetry recording |

### 6.3 Required Documentation

| Document | Purpose |
|----------|---------|
| Integration procedures | Step-by-step instructions |
| Test procedures | Verification tests |
| Anomaly forms | Issue documentation |
| Inspection checklists | Quality verification |
| As-built drawings | Configuration control |

## 7. Anomaly Handling

### 7.1 Anomaly Classification

| Category | Response |
|----------|----------|
| Critical | Stop, investigate, resolve before continuing |
| Major | Document, assess impact, may continue with caution |
| Minor | Document, continue, resolve before closeout |
| Observation | Document for future reference |

### 7.2 Anomaly Resolution Process

1. **Detect**: Identify anomaly during integration
2. **Document**: Record in anomaly log with photos
3. **Assess**: Determine severity and impact
4. **Investigate**: Root cause analysis
5. **Resolve**: Implement corrective action
6. **Verify**: Confirm fix is effective
7. **Close**: Document closure in anomaly log

## 8. Schedule

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| Flatsat assembly | 4 weeks | T+0 | T+4 |
| Software development | 8 weeks | T+4 | T+12 |
| Flight unit fabrication | 8 weeks | T+8 | T+16 |
| Flight unit integration | 8 weeks | T+16 | T+24 |
| System functional test | 2 weeks | T+24 | T+26 |
| Environmental testing | 6 weeks | T+26 | T+32 |
| Final acceptance | 2 weeks | T+32 | T+34 |

## 9. Success Criteria

Integration is complete when:
- [ ] All subsystems integrated and functional
- [ ] All integration tests passed
- [ ] All anomalies resolved or waived
- [ ] Mass and CG within specification
- [ ] Configuration documented
- [ ] Ready for environmental testing

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
