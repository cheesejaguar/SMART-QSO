# SMART-QSO Hardware Development Plan

**Document ID**: SQSO-HDP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the Hardware Development Plan for SMART-QSO, establishing the approach, methods, and schedule for developing flight hardware per NASA Ames SmallSat heritage practices.

## 2. Hardware Development Approach

### 2.1 Development Philosophy

```
HARDWARE DEVELOPMENT PHILOSOPHY

For CubeSat-class missions:

1. Maximize COTS/Heritage
   └─ Use proven components where possible
   └─ Custom designs only where necessary

2. Prototype Early
   └─ Breadboard for concept validation
   └─ EDU for software development
   └─ EM for qualification

3. Test As You Fly
   └─ Flight-like test configurations
   └─ Realistic environmental conditions

4. Design for Manufacturability
   └─ Standard processes
   └─ Available materials
   └─ Repeatable assembly
```

### 2.2 Hardware Classification

| Classification | Definition | Examples |
|----------------|------------|----------|
| COTS | Commercial off-the-shelf | Jetson, connectors |
| Modified COTS | COTS with modifications | Radio module |
| Custom | Purpose-designed | Structure, carrier boards |
| Heritage | From prior missions | Design patterns |

### 2.3 Development Models

| Model | Purpose | Fidelity |
|-------|---------|----------|
| Breadboard | Concept validation | Low |
| EDU | Software development | Medium |
| EM | Qualification | High |
| Flight | Mission | Full |

## 3. Hardware Architecture

### 3.1 Subsystem Breakdown

```
HARDWARE ARCHITECTURE

SMART-QSO Spacecraft
├── Structure Subsystem
│   ├── Primary structure (rails, panels)
│   ├── Deployment mechanism
│   └── Mounting hardware
│
├── Electrical Power Subsystem (EPS)
│   ├── Solar arrays
│   ├── Battery pack
│   ├── Power conditioning
│   └── Power distribution
│
├── On-Board Computer (OBC)
│   ├── Processor board
│   ├── Memory
│   └── Interfaces
│
├── RF Subsystem
│   ├── Transceiver
│   ├── Power amplifier
│   ├── Antenna system
│   └── RF filtering
│
├── AI Payload
│   ├── Jetson Orin Nano
│   ├── Carrier board
│   └── Thermal management
│
└── ADCS Sensors
    ├── Magnetometer
    ├── Sun sensors
    └── Rate sensors
```

### 3.2 Interface Summary

| Interface | Type | Protocol |
|-----------|------|----------|
| OBC-EPS | I2C | Telemetry, commands |
| OBC-RF | SPI | Data, control |
| OBC-Jetson | UART | AI requests/responses |
| OBC-Sensors | I2C/SPI | Sensor data |

## 4. Development Schedule

### 4.1 Hardware Development Timeline

| Phase | Duration | Activities |
|-------|----------|------------|
| Breadboard | 2 months | Concept validation |
| EDU Build | 2 months | Flatsat development |
| EDU Test | 2 months | Software integration |
| EM Build | 2 months | Qualification unit |
| EM Test | 3 months | Environmental qualification |
| Flight Build | 2 months | Flight unit fabrication |
| Flight Test | 1 month | Acceptance testing |
| Integration | 1 month | Final integration |

### 4.2 Key Milestones

| Milestone | Target |
|-----------|--------|
| Breadboard complete | PDR |
| EDU/Flatsat complete | CDR |
| EM complete | TRR |
| EM qual complete | PER |
| Flight build complete | PSR |
| Flight acceptance complete | FRR |

### 4.3 Build Flow

```
HARDWARE BUILD FLOW

Breadboard ──→ EDU ──→ EM ──→ Flight
    │           │       │        │
    ▼           ▼       ▼        ▼
Concept     Software  Qual    Mission
Validation    Dev     Test    Ready
```

## 5. Subsystem Development

### 5.1 Structure Subsystem

| Item | Approach | Status |
|------|----------|--------|
| Primary structure | Custom machined Al | Design complete |
| Panels | Custom machined Al | Design complete |
| Antenna mount | Custom | Design complete |
| Deployment mech | Custom spring | Design complete |

**Key Design Features**:
- 100mm × 100mm × 113.5mm envelope
- Al 7075-T6 rails for strength
- Al 6061-T6 panels for mass efficiency
- Tape spring antenna deployment

### 5.2 EPS Subsystem

| Item | Approach | Status |
|------|----------|--------|
| Solar cells | COTS (Spectrolab) | Spec complete |
| Battery | COTS (LG MJ1) | Spec complete |
| MPPT | COTS IC (SPV1050) | Design complete |
| Distribution | Custom PCB | Design complete |

**Key Design Features**:
- 6× triple-junction solar cells
- 2× 18650 Li-ion cells (2P)
- Hardware battery protection
- Software power management

### 5.3 OBC Subsystem

| Item | Approach | Status |
|------|----------|--------|
| Processor | COTS (Ambiq Apollo4) | Selected |
| Memory | COTS (FRAM, Flash) | Selected |
| Carrier board | Custom PCB | Design complete |
| Watchdog | COTS IC + custom | Design complete |

**Key Design Features**:
- Ultra-low power MCU (~200μA active)
- Radiation-tolerant by design
- Hardware watchdog with safe mode trigger
- Multiple debug interfaces

### 5.4 RF Subsystem

| Item | Approach | Status |
|------|----------|--------|
| Transceiver | COTS (Si4463) | Selected |
| VHF PA | COTS (RD00HVS1) | Selected |
| Antenna | Custom tape spring | Design complete |
| Filters | COTS SAW | Selected |

**Key Design Features**:
- VHF downlink (145 MHz)
- UHF uplink (435 MHz)
- 0.5-2W selectable TX power
- AX.25 compatible

### 5.5 AI Payload

| Item | Approach | Status |
|------|----------|--------|
| Compute | COTS (Jetson Orin Nano) | Selected |
| Carrier | Custom minimal | Design complete |
| Thermal | Passive + duty cycle | Design complete |
| Power | Switched, power-gated | Design complete |

**Key Design Features**:
- TinyLM inference capability
- Power-gated for efficiency
- Duty cycle limited (≤10%)
- Graceful fallback to templates

## 6. Fabrication

### 6.1 Fabrication Methods

| Component | Method | Vendor |
|-----------|--------|--------|
| Structure | CNC machining | TBD |
| PCBs | Standard FR-4 | TBD |
| Harness | Hand assembly | In-house |
| Integration | In-house | - |

### 6.2 Fabrication Standards

| Standard | Application |
|----------|-------------|
| IPC-A-600 | PCB fabrication |
| IPC-A-610 | PCB assembly |
| NASA-STD-8739.3 | Soldering |
| NASA-STD-8739.4 | Crimping |

### 6.3 Fabrication Flow

```
FABRICATION FLOW

1. Procurement
   └─ Parts, materials, services

2. Receiving Inspection
   └─ Verify compliance

3. Board Fabrication
   └─ External vendor

4. Board Assembly
   └─ In-house or vendor

5. Board Test
   └─ Functional verification

6. Integration
   └─ Mechanical and electrical

7. System Test
   └─ End-to-end verification
```

## 7. Assembly and Integration

### 7.1 Assembly Sequence

| Step | Activity | Prerequisites |
|------|----------|---------------|
| 1 | Structure assembly | Parts received |
| 2 | Harness installation | Structure complete |
| 3 | EPS installation | Harness installed |
| 4 | OBC installation | EPS installed |
| 5 | RF installation | OBC installed |
| 6 | Payload installation | RF installed |
| 7 | Antenna installation | All boards installed |
| 8 | Close-out | All installation complete |

### 7.2 Integration Environment

| Requirement | Specification |
|-------------|---------------|
| Cleanliness | Class 100,000 minimum |
| ESD control | Per ANSI/ESD S20.20 |
| Temperature | 20-25°C |
| Humidity | 30-60% RH |

### 7.3 Integration Documentation

| Document | Purpose |
|----------|---------|
| Assembly drawings | Build instructions |
| Procedures | Step-by-step guide |
| Travelers | As-built record |
| Photos | Visual record |

## 8. Testing

### 8.1 Test Program

| Level | Tests | Purpose |
|-------|-------|---------|
| Board | Functional | Verify board operation |
| Subsystem | Integration | Verify interfaces |
| System | Functional | Verify system operation |
| Environmental | Qualification | Verify robustness |
| Acceptance | Workmanship | Flight readiness |

### 8.2 Environmental Tests

| Test | Level | Duration |
|------|-------|----------|
| Vibration | Proto-flight | 1 min/axis |
| Thermal vacuum | Proto-flight | 4 cycles |
| EMC/EMI | Tailored | Per spec |

### 8.3 Test Flow

```
TEST FLOW

Board Test
    │
    ▼
Subsystem Test
    │
    ▼
Flatsat Test ◄─── SW Development
    │
    ▼
System Integration Test
    │
    ▼
Pre-Environmental Functional
    │
    ▼
Environmental Test ──► Vibration
    │                  Thermal Vacuum
    │                  EMC
    ▼
Post-Environmental Functional
    │
    ▼
Acceptance Test
    │
    ▼
FLIGHT READY
```

## 9. Quality Assurance

### 9.1 QA Activities

| Activity | When |
|----------|------|
| Design review | PDR, CDR |
| Receiving inspection | Parts arrival |
| In-process inspection | During assembly |
| Final inspection | Before close-out |
| Test witness | Key tests |

### 9.2 Critical Items

| Item | Controls |
|------|----------|
| Battery | Special handling, testing |
| Solar cells | ESD, handling |
| Antenna mechanism | Functional test |
| RF connectors | Torque verification |

### 9.3 Workmanship

| Standard | Application |
|----------|-------------|
| NASA-STD-8739.3 | Soldering |
| NASA-STD-8739.4 | Crimping |
| IPC-A-610 Class 3 | Inspection |

## 10. Configuration Control

### 10.1 Hardware Baseline

| Baseline | When | Content |
|----------|------|---------|
| Design | CDR | Drawings, specs |
| As-built | Integration | Serial numbers |
| Flight | FRR | Final configuration |

### 10.2 Change Control

| Change Type | Authority |
|-------------|-----------|
| Drawing (minor) | Lead engineer |
| Drawing (major) | CCB |
| Parts substitution | CCB |
| Design change | CCB |

### 10.3 Traceability

| Item | Tracking |
|------|----------|
| Parts | Lot/date codes |
| Assemblies | Serial numbers |
| Configuration | As-built list |
| Changes | ECO numbers |

## 11. Risk Management

### 11.1 Hardware Risks

| Risk | Mitigation |
|------|------------|
| Parts obsolescence | Early procurement |
| Vendor delays | Schedule margin |
| Test failures | Retest/repair capability |
| Workmanship | Training, inspection |

### 11.2 Technical Risks

| Risk | Mitigation |
|------|------------|
| Thermal | Analysis, testing |
| Power budget | Margin, testing |
| Mass budget | Tracking, design |
| RF performance | Analysis, testing |

## 12. Deliverables

### 12.1 Hardware Deliverables

| Deliverable | Description |
|-------------|-------------|
| Flight unit | Integrated spacecraft |
| GSE | Ground support equipment |
| Spares | Critical spares |
| Documentation | As-built records |

### 12.2 Documentation Deliverables

| Document | Purpose |
|----------|---------|
| As-built drawings | Final configuration |
| Test reports | Verification evidence |
| Travelers | Assembly records |
| Calibration records | Equipment status |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
