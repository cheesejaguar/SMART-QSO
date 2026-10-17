# SMART-QSO CDR Technical Baseline

> **Funding audit notice (2026-10-14): internal self-assessment, not an independent review.**
> This record was generated on 2026-01-02 without named independent reviewers, signatures, or a
> RID/RFA log. Its PASS/APPROVED/COMPLETE statements are **not substantiated** and must not be cited
> to funders or regulators as review outcomes. See `docs/funding_audit/FUNDING_AUDIT_REPORT.md` and
> finding F-PI-03 in `docs/funding_audit/FINDINGS_REGISTER.md`.

**Document ID**: SQSO-CDR-TB-001
**Version**: 1.1
**Date**: 2026-01-02

> **Superseded values (funding audit, 2026-10-16, finding PI-14).** This record is kept as written on 2026-01-02, but it
> did not describe one configuration. `docs/BASELINE_PARAMETERS.md` governs every value in §3 and §4; the values below
> replace those in the record:

| Parameter (where in this record) | Value in this record | Governing value | Source |
|----------------------------------|----------------------|-----------------|--------|
| Battery (§3.1, §4.2) | 2× 18650 Li-ion, 7 Wh | 4S1P 18650, 14.4 V, ≈ 50 Wh (49.7 Wh) | BASELINE §5; `hardware/eps/BATTERY_DESIGN.md` v2.0 |
| Solar array (§4.2) | 6 cells, 7 W BOL | 16 cells; EOL orbit average 2.39 W tumbling (2.14 W with self-shadowing), 3.58 W sun-pointed | BASELINE §5; `hardware/eps/SOLAR_ARRAY_DESIGN.md` v2.0 |
| Mass (§3.2, §4.1) | 1.2 kg, "33 % to 2 kg limit"; structure 350 g | Limit 2.00 kg. BASELINE §4: CBE ≈ 1.51 kg, MEV ≤ 1.75 kg. Bottom-up MEL: CBE 1.69 kg, predicted 1.92–2.00 kg | BASELINE §4; `docs/environment/MASS_BUDGET.md` v2.0 |
| Average power (§3.2) | 0.35 W, "128 % positive" | Load 1.80 W tumbling, 2.20 W sun-pointed; margin +33 % / +63 % (+19 % tumbling with self-shadowing) | BASELINE §5; SOLAR_ARRAY_DESIGN §9.4 |
| Orbit (§3.2) | 400 km, 51.6° | Circular SSO, ≈ 97.4°, ≤ 500 km target, 525 km cap | BASELINE §3 |
| OBC processor (§3.1, §4.3) | Ambiq Apollo4 Blue Plus | STM32L4R5ZI, FreeRTOS | BASELINE §6 |
| TX power (§4.4) | 0.5–2 W | 0.5 W nominal, 1 W maximum | BASELINE §7 |
| Link margin (§4.4) | VHF downlink +1.6 dB at 30°, 20° minimum elevation | 1200 bps beacon at 0.5 W: +7.7 dB at 5° elevation (+4.7 dB conservative pattern); telecommand +4.7 dB at 5° in 92 % of orientations | `docs/environment/LINK_BUDGET_ANALYSIS.md` v2.0 |
| ADCS actuators (§4.6) | None (passive) | 3-axis magnetorquers | BASELINE §8 |

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
| MO-1 | Demonstrate AI-managed amateur radio operations from LEO | AI-generated beacons transmitted |
| MO-2 | Provide amateur radio operators with transponder access | Beacons received by community |
| MO-3 | Demonstrate graceful degradation to conventional operation | Transponder and template beacon continue with the AI payload off (SYS-521, SYS-522) |
| MO-4 | Advance TinyML/TinyLM technology for space applications | Not stated in SQSO-REQ-001 |
| MO-5 | Engage amateur radio community in space communications | Not stated in SQSO-REQ-001 |

Objectives corrected 2026-10-16 to SQSO-REQ-001 §4.2. This baseline listed "operate autonomously for 2 years" and
"comply with amateur regulations" as MO-3 and MO-4; those are mission requirements (MRD-001, MRD-002; MRD-024, MRD-090
to MRD-092).

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
| 1.0a | 2026-10-16 | Funding audit | Superseded-values table added under the notice (PI-14); body unchanged |
| 1.1 | 2026-10-16 | SMART-QSO Team | §2.2 objectives corrected to SQSO-REQ-001 §4.2; MO-5 added (finding VV-06) |
