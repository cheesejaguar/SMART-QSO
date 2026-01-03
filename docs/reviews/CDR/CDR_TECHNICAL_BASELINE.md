# SMART-QSO CDR Technical Baseline

**Document ID**: SQSO-CDR-TB-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document establishes the technical baseline for SMART-QSO at Critical Design Review (CDR), capturing the complete system design that will proceed to fabrication and test.

## 2. Mission Summary

### 2.1 Mission Statement

SMART-QSO is a 1U CubeSat mission demonstrating AI-assisted amateur radio operations, providing autonomous beacon generation and transponder services to the amateur radio community.

### 2.2 Mission Objectives

| ID | Objective | Success Measure |
|----|-----------|-----------------|
| MO-1 | Demonstrate AI-assisted amateur radio | AI-generated beacons transmitted |
| MO-2 | Provide amateur radio service | Beacons received by community |
| MO-3 | Operate autonomously for 2 years | Continuous operation |
| MO-4 | Comply with amateur regulations | No violations |

### 2.3 Mission Success Criteria

| Level | Duration | Key Criterion |
|-------|----------|---------------|
| Level 1 (Minimum) | 30 days | Beacon operations |
| Level 2 (Full) | 1 year | AI-assisted beacons |
| Level 3 (Extended) | 2+ years | Continuous service |

## 3. System Design Baseline

### 3.1 System Architecture

```
SMART-QSO SYSTEM ARCHITECTURE

┌─────────────────────────────────────────────────────────┐
│                    SMART-QSO 1U CubeSat                 │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │    EPS      │  │    OBC      │  │     RF      │     │
│  │  Solar/Bat  │◄─┤  Apollo4    │─►│  Si4463     │     │
│  │  7Wh        │  │  FreeRTOS   │  │  VHF/UHF    │     │
│  └─────────────┘  └──────┬──────┘  └─────────────┘     │
│                          │                              │
│                   ┌──────┴──────┐                       │
│                   │   Jetson    │                       │
│                   │  TinyLM AI  │                       │
│                   │  (Payload)  │                       │
│                   └─────────────┘                       │
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │    ADCS     │  │  Structure  │  │   Antenna   │     │
│  │ Mag+Sun+Gyr │  │  Al 7075    │  │ Tape Spring │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Key Design Parameters

| Parameter | Value | Margin |
|-----------|-------|--------|
| Mass | 1.2 kg | 33% to 2 kg limit |
| Power (avg) | 0.35W | 128% positive |
| Data rate | 1200/9600 bps | N/A |
| Design life | 2 years | Orbital life 2-5 yr |
| Orbit | 400 km, 51.6° | ISS-like |

## 4. Subsystem Baselines

### 4.1 Structure Subsystem

| Parameter | Baseline |
|-----------|----------|
| Form factor | 1U (100×100×113.5mm) |
| Material | Al 7075-T6 (rails), Al 6061-T6 (panels) |
| Mass | 350g |
| Deployment | Tape spring antenna |
| CDS compliance | Rev 14.1 |

### 4.2 Electrical Power Subsystem

| Parameter | Baseline |
|-----------|----------|
| Solar cells | 6× triple-junction, 7W BOL |
| Battery | 2× 18650 Li-ion, 7 Wh |
| Regulation | MPPT, 3.3V/5V rails |
| Protection | HW OVP/UVP, SW limits |

**Power Budget Summary**:
| Mode | Power | Duration | Margin |
|------|-------|----------|--------|
| SAFE | 0.1W | Continuous | +600% |
| IDLE | 0.2W | 90% | +350% |
| ACTIVE | 0.5W | 10% | +140% |
| AI Burst | 5W | <1% | +40% |

### 4.3 On-Board Computer

| Parameter | Baseline |
|-----------|----------|
| Processor | Ambiq Apollo4 Blue Plus |
| Clock | 96 MHz |
| RAM | 2 MB |
| Flash | 2 MB + 32 MB external |
| FRAM | 128 KB (state storage) |
| RTOS | FreeRTOS |

### 4.4 RF Subsystem

| Parameter | Baseline |
|-----------|----------|
| Transceiver | Silicon Labs Si4463 |
| Downlink | 145.XXX MHz VHF |
| Uplink | 435.XXX MHz UHF |
| TX Power | 0.5-2W selectable |
| Modulation | AFSK (1200), GFSK (9600) |
| Protocol | AX.25 |

**Link Budget Summary**:
| Link | Margin @ 30° | Minimum El |
|------|--------------|------------|
| VHF Downlink | +1.6 dB | 20° |
| UHF Uplink | +6.0 dB | 15° |

### 4.5 AI Payload

| Parameter | Baseline |
|-----------|----------|
| Compute | NVIDIA Jetson Orin Nano |
| Memory | 8 GB |
| Model | TinyLM (~1 MB INT8) |
| Duty cycle | ≤10% |
| Fallback | Template beacons |

### 4.6 ADCS

| Parameter | Baseline |
|-----------|----------|
| Sensors | Magnetometer, sun sensors, gyro |
| Actuators | None (passive) |
| Pointing | Tumbling acceptable |
| Requirement | Antenna coverage |

## 5. Software Baseline

### 5.1 Flight Software

| Parameter | Baseline |
|-----------|----------|
| Version | 1.0.0 |
| Language | C11 (MISRA C:2012) |
| RTOS | FreeRTOS 10.x |
| Size | ~64 KB code |
| Tasks | 6 (beacon, telemetry, command, power, fault, AI) |

### 5.2 State Machine

```
STATE MACHINE BASELINE

         ┌─────────┐
    ┌────│  INIT   │────┐
    │    └────┬────┘    │
    │         │         │
    │    ┌────▼────┐    │
    │    │  SAFE   │◄───┤ (fault)
    │    └────┬────┘    │
    │         │         │
    │    ┌────▼────┐    │
    │    │  IDLE   │────┤
    │    └────┬────┘    │
    │         │         │
    │    ┌────▼────┐    │
    └────│ ACTIVE  │────┘
         └─────────┘
```

### 5.3 Payload Software

| Parameter | Baseline |
|-----------|----------|
| Version | 1.0.0 |
| Language | Python 3.x |
| Framework | TensorRT |
| Interface | UART @ 115200 |

## 6. Interface Baseline

### 6.1 Internal Interfaces

| Interface | Type | Protocol |
|-----------|------|----------|
| OBC-EPS | I2C | 400 kHz, addr 0x50 |
| OBC-RF | SPI | 10 MHz, Mode 0 |
| OBC-Jetson | UART | 115200, 8N1 |
| OBC-Sensors | I2C/SPI | Various |

### 6.2 External Interfaces

| Interface | Type | Specification |
|-----------|------|---------------|
| Deployer | Mechanical | CDS Rev 14.1 |
| Ground RF | Wireless | AX.25, VHF/UHF |
| EGSE | Electrical | USB, SWD |

## 7. Environmental Baseline

### 7.1 Design Environments

| Environment | Specification |
|-------------|---------------|
| Temperature (op) | -20°C to +50°C |
| Temperature (surv) | -40°C to +85°C |
| Vacuum | <10⁻⁵ Torr |
| Radiation | 3 krad TID (2 yr) |
| Vibration | 14.1 Grms proto-flight |

### 7.2 Thermal Baseline

| Component | Hot Limit | Cold Limit | Margin |
|-----------|-----------|------------|--------|
| OBC | +85°C | -40°C | +35°C |
| Battery | +45°C | 0°C | +15°C |
| Jetson | +75°C | -25°C | +20°C |

## 8. Operations Baseline

### 8.1 Mission Phases

| Phase | Duration | Activities |
|-------|----------|------------|
| LEOP | 24 hours | Acquisition, health check |
| Commissioning | 14 days | System checkout, AI enable |
| Primary Ops | 2 years | Autonomous operations |
| EOL | TBD | Passivation, deorbit |

### 8.2 Ground Operations

| Parameter | Baseline |
|-----------|----------|
| Ground station | Single primary |
| Passes per day | 4-6 |
| Contact time | 30-50 min/day |
| Data volume | 252 KB/day |

## 9. Verification Baseline

### 9.1 Verification Approach

| Level | Environment | Method |
|-------|-------------|--------|
| Component | Lab | Unit test |
| Subsystem | Flatsat | Integration test |
| System | HITL | Functional test |
| Qualification | Environmental | Proto-flight |

### 9.2 Requirements Status

| Category | Total | Verified | Pending |
|----------|-------|----------|---------|
| System | 30 | 8 | 22 |
| Software | 30 | 8 | 22 |
| **Total** | **60** | **16** | **44** |

*Note: Pending verifications require flatsat/flight hardware*

## 10. Risk Baseline

### 10.1 Top Risks

| Risk | L×C | Status |
|------|-----|--------|
| AI power consumption | M×M | Mitigated (duty cycle) |
| Thermal management | M×M | Mitigated (analysis) |
| Battery life | L×H | Monitored |
| Radiation effects | L×M | Mitigated (design) |

### 10.2 Risk Summary

| Level | Count |
|-------|-------|
| High | 0 |
| Medium | 2 |
| Low | 3 |

## 11. Schedule Baseline

### 11.1 Key Milestones

| Milestone | Status |
|-----------|--------|
| SRR | Complete |
| PDR | Complete |
| CDR | Current |
| TRR | Planned |
| FRR | Planned |
| Launch | TBD |

### 11.2 Post-CDR Activities

| Activity | Duration |
|----------|----------|
| Flatsat testing | 4 weeks |
| EM build/test | 8 weeks |
| Environmental | 4 weeks |
| Flight build | 4 weeks |
| Acceptance | 2 weeks |

## 12. Configuration Baseline

### 12.1 Document Baseline

| Document | Version | Status |
|----------|---------|--------|
| System Requirements | 1.0 | Baselined |
| Software Requirements | 1.0 | Baselined |
| Software Design | 1.0 | Baselined |
| ICDs | 1.0 | Baselined |
| Test Plan | 1.0 | Baselined |

### 12.2 Software Baseline

| Software | Version | Status |
|----------|---------|--------|
| Flight Software | 1.0.0 | CDR baseline |
| Payload Software | 1.0.0 | CDR baseline |
| Ground Software | 1.0.0 | CDR baseline |

## 13. Approval

```
CDR TECHNICAL BASELINE APPROVAL

This document establishes the technical baseline
for SMART-QSO at Critical Design Review.

Project Manager: _____________ Date: _______

Systems Engineer: ____________ Date: _______

Software Lead: _______________ Date: _______

QA: _________________________ Date: _______
```

## 14. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
