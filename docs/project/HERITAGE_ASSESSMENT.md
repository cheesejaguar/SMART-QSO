# SMART-QSO Heritage Assessment

**Document ID**: SQSO-HA-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the Heritage Assessment for SMART-QSO per NPR 7120.5 §2.2 and NASA-STD-7009 (Models and Simulations), tracing design elements to NASA Ames SmallSat heritage missions and quantifying mission risk reduction through heritage application.

## 2. Heritage Mission Summary

### 2.1 Reference Missions

| Mission | Launch | Duration | Bus | Payload | Status |
|---------|--------|----------|-----|---------|--------|
| GeneSat-1 | 2006 | 4 months | 3U | Life science | Success |
| PharmaSat | 2009 | 5 months | 3U | Microfluidics | Success |
| O/OREOS | 2010 | 3+ years | 3U | Astrobiology | Success |
| EcAMSat | 2017 | 4 months | 6U | Life science | Success |

### 2.2 Heritage Architecture

```
NASA AMES SMALLSAT HERITAGE ARCHITECTURE

┌─────────────────────────────────────────────────────────────┐
│                    SMART-QSO (2026)                         │
│                                                             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │   EPS   │  │   OBC   │  │   RF    │  │Payload  │       │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘       │
│       │            │            │            │              │
└───────┼────────────┼────────────┼────────────┼──────────────┘
        │            │            │            │
        ▼            ▼            ▼            ▼
┌───────────────────────────────────────────────────────────┐
│                    HERITAGE SOURCES                        │
│                                                            │
│  GeneSat (2006)   PharmaSat (2009)   O/OREOS (2010)       │
│  - Basic EPS      - Power mgmt       - Fault tolerance    │
│  - AX.25 beacon   - Thermal ctrl     - Extended ops       │
│  - Simple ADCS    - Autonomous ops   - Telemetry          │
│                                                            │
│                    EcAMSat (2017)                          │
│                    - Software arch                         │
│                    - Mission assurance                     │
│                    - Quality standards                     │
└───────────────────────────────────────────────────────────┘
```

## 3. Heritage Assessment Matrix

### 3.1 Subsystem Heritage Mapping

| Subsystem | TRL | Heritage Source | Heritage % | Notes |
|-----------|-----|-----------------|------------|-------|
| Structure | 8 | CubeSat standard | 90% | Standard 1U rails |
| EPS | 7 | EcAMSat/PharmaSat | 70% | Similar architecture |
| OBC | 6 | EcAMSat | 60% | New MCU, similar SW |
| RF | 7 | GeneSat/O/OREOS | 75% | AX.25 heritage |
| ADCS | 6 | GeneSat | 65% | B-dot heritage |
| Payload | 4 | Novel | 10% | New AI capability |
| Software | 6 | All missions | 50% | Architecture heritage |
| Ground | 7 | All missions | 70% | Station heritage |

### 3.2 Technology Readiness Levels

| TRL | Definition | SMART-QSO Application |
|-----|------------|----------------------|
| 9 | Flight proven | CubeSat rails |
| 8 | Qualified | Solar cells, batteries |
| 7 | Demonstrated | Radio transceiver, EPS |
| 6 | Validated | OBC, ADCS, software |
| 5 | Breadboard | Jetson integration |
| 4 | Lab validated | AI inference chain |

### 3.3 Component Heritage

| Component | Heritage | Source Mission | TRL |
|-----------|----------|----------------|-----|
| 18650 Li-ion cells | High | PharmaSat | 8 |
| Si4463 transceiver | Medium | Similar to O/OREOS | 7 |
| AX.25 protocol | High | All missions | 9 |
| Magnetorquers | High | GeneSat | 8 |
| Magnetometer | High | All missions | 8 |
| Sun sensors | High | O/OREOS | 8 |
| MCU (Apollo4) | Low | New selection | 5 |
| Jetson Orin | Low | Novel | 4 |
| FRAM | Medium | EcAMSat | 7 |

## 4. Detailed Heritage Analysis

### 4.1 Electrical Power System

#### 4.1.1 Heritage Elements

| Element | Heritage Source | Similarity | Adaptations |
|---------|-----------------|------------|-------------|
| Battery pack | PharmaSat | High | 2S config vs 2S2P |
| Charge controller | EcAMSat | High | Similar MPPT |
| Power distribution | All missions | Medium | Added Jetson rail |
| Protection circuits | All missions | High | Same philosophy |
| Telemetry | All missions | High | Standard parameters |

#### 4.1.2 Risk Reduction

| Risk | Heritage Mitigation | Residual Risk |
|------|---------------------|---------------|
| Battery failure | Proven cell selection | Low |
| Overcharge | Proven protection | Low |
| Power deficit | Heritage margin philosophy | Medium |
| Thermal damage | Proven thermal design | Low |

#### 4.1.3 Delta from Heritage

- Added high-power Jetson rail (5V, 10W)
- Added power gating for AI payload
- Lower total power (2W vs 5W typical)

### 4.2 Command and Data Handling

#### 4.2.1 Heritage Elements

| Element | Heritage Source | Similarity | Adaptations |
|---------|-----------------|------------|-------------|
| State machine | All missions | High | 3 states vs 5 |
| Watchdog | All missions | High | External WDT |
| Boot sequence | EcAMSat | High | Similar logic |
| Telemetry format | GeneSat | Medium | Extended params |
| Command format | O/OREOS | Medium | Simplified |

#### 4.2.2 Risk Reduction

| Risk | Heritage Mitigation | Residual Risk |
|------|---------------------|---------------|
| Processor hang | Proven watchdog | Low |
| Memory corruption | FRAM + ECC heritage | Low |
| Boot failure | Dual boot heritage | Low |
| Mode confusion | Simplified state machine | Low |

#### 4.2.3 Delta from Heritage

- New processor (Apollo4 vs ARM7/Cortex-M3)
- Added AI interface (UART to Jetson)
- Reduced RAM footprint

### 4.3 RF Communications

#### 4.3.1 Heritage Elements

| Element | Heritage Source | Similarity | Adaptations |
|---------|-----------------|------------|-------------|
| AX.25 protocol | All missions | High | Standard |
| Beacon format | GeneSat | High | Added AI field |
| Frequency band | All missions | High | 2m/70cm |
| Power level | O/OREOS | High | 0.5-2W |
| Antenna | GeneSat | Medium | Tape dipole |

#### 4.3.2 Risk Reduction

| Risk | Heritage Mitigation | Residual Risk |
|------|---------------------|---------------|
| RF interference | Heritage frequency coord | Low |
| Link closure | Proven link margin | Low |
| Protocol errors | Standard AX.25 | Low |
| Antenna failure | Simple design | Low |

#### 4.3.3 Delta from Heritage

- Dual-band (VHF down, UHF up) vs single
- Dynamic beacon content
- Shorter beacon (200 chars vs 512)

### 4.4 Attitude Determination and Control

#### 4.4.1 Heritage Elements

| Element | Heritage Source | Similarity | Adaptations |
|---------|-----------------|------------|-------------|
| B-dot detumble | GeneSat | High | Same algorithm |
| Magnetorquers | All missions | High | Standard |
| Magnetometer | All missions | High | I2C interface |
| Sun sensors | O/OREOS | High | Coarse pointing |

#### 4.4.2 Risk Reduction

| Risk | Heritage Mitigation | Residual Risk |
|------|---------------------|---------------|
| Tumble recovery | Proven B-dot | Low |
| Sensor failure | Dual sensors | Low |
| Pointing loss | Coarse req only | Low |

#### 4.4.3 Delta from Heritage

- No reaction wheels (not needed)
- Reduced pointing requirement (30° vs 5°)
- Integrated sun sensor/magnetometer

### 4.5 Software

#### 4.5.1 Heritage Elements

| Element | Heritage Source | Similarity | Adaptations |
|---------|-----------------|------------|-------------|
| RTOS | All missions | Medium | FreeRTOS |
| Task architecture | EcAMSat | High | Similar partitioning |
| Fault handling | All missions | High | Same philosophy |
| Telemetry framework | All missions | Medium | Adapted format |
| Command handler | O/OREOS | High | Similar logic |

#### 4.5.2 Risk Reduction

| Risk | Heritage Mitigation | Residual Risk |
|------|---------------------|---------------|
| Software crash | Proven architecture | Low |
| Task starvation | Priority inheritance | Low |
| Deadlock | Proven patterns | Low |
| Memory leak | Static allocation | Low |

#### 4.5.3 Delta from Heritage

- New RTOS (FreeRTOS vs proprietary)
- AI integration layer
- Python ground software vs C

### 4.6 Payload (AI)

#### 4.6.1 Heritage Elements

| Element | Heritage Source | Similarity | Notes |
|---------|-----------------|------------|-------|
| Payload interface | EcAMSat | Medium | UART heritage |
| Power sequencing | PharmaSat | Medium | Adapted |
| Fallback mode | O/OREOS | High | Same concept |

#### 4.6.2 Novel Elements

| Element | TRL | Mitigation |
|---------|-----|------------|
| Jetson integration | 4 | Extensive testing |
| TinyLM inference | 4 | Fallback mode |
| Dynamic content | 4 | Template fallback |
| DVFS control | 5 | Conservative limits |

#### 4.6.3 Risk Reduction Strategy

- Mission success not dependent on AI
- Template beacons as fallback
- Power-gated when not in use
- Thermal limits enforced in hardware

## 5. Heritage Metrics

### 5.1 Overall Heritage Score

| Category | Weight | Score | Weighted |
|----------|--------|-------|----------|
| EPS | 20% | 0.70 | 0.14 |
| OBC | 20% | 0.60 | 0.12 |
| RF | 15% | 0.75 | 0.11 |
| ADCS | 10% | 0.65 | 0.07 |
| Structure | 10% | 0.90 | 0.09 |
| Software | 15% | 0.50 | 0.08 |
| Ground | 10% | 0.70 | 0.07 |
| **Total** | **100%** | | **0.68** |

**Overall Heritage Score: 68%**

### 5.2 Interpretation

| Score | Interpretation |
|-------|----------------|
| >80% | High heritage, low risk |
| 60-80% | Moderate heritage, manageable risk |
| 40-60% | Low heritage, higher risk |
| <40% | Novel, significant risk |

**SMART-QSO Assessment**: Moderate heritage with manageable risk. Novel AI payload balanced by high-heritage bus design.

### 5.3 Heritage by Phase

| Phase | Heritage Elements | Novel Elements |
|-------|-------------------|----------------|
| LEOP | All operations | None |
| Commissioning | Most checkout | AI characterization |
| Nominal Ops | Basic beacon | AI beacon generation |
| Contingency | All recovery | AI fallback |

## 6. Risk Reduction Summary

### 6.1 Heritage-Based Risk Reduction

| Risk Area | Without Heritage | With Heritage | Delta |
|-----------|------------------|---------------|-------|
| EPS failure | Medium | Low | -1 level |
| Comm failure | Medium | Low | -1 level |
| SW failure | High | Medium | -1 level |
| ADCS failure | Medium | Low | -1 level |
| Payload failure | High | High | 0 (novel) |

### 6.2 Remaining Risks

| Risk | Heritage N/A | Mitigation |
|------|--------------|------------|
| AI inference failure | Novel system | Fallback mode |
| Jetson thermal | Novel config | Conservative limits |
| Power cycling stress | Novel duty cycle | Qualification test |

## 7. Heritage Verification

### 7.1 Verification Approach

| Element | Verification Method |
|---------|---------------------|
| Design heritage | Design review |
| Component heritage | Similarity analysis |
| Process heritage | Process audit |
| Operations heritage | Rehearsal |

### 7.2 Heritage Validation Tests

| Test | Heritage Validation |
|------|---------------------|
| Flatsat | Bus heritage verification |
| Environmental | Heritage qualification levels |
| Extended duration | Heritage stability |
| Day-in-the-life | Operations heritage |

## 8. Conclusion

SMART-QSO leverages significant heritage from NASA Ames SmallSat missions, particularly in bus design and operations. The novel AI payload represents the primary risk area, mitigated by:

1. Fail-graceful design (template fallback)
2. Mission success independent of AI
3. Conservative power/thermal management
4. Extensive ground testing

The 68% heritage score indicates a balanced design combining proven approaches with measured innovation.

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
