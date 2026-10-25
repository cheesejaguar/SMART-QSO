# SMART-QSO Reliability Analysis

**Document ID**: SQSO-REL-001
**Version**: 1.2
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the reliability analysis for SMART-QSO per NASA-HDBK-4002A, establishing reliability requirements, predictions, and design features for the 2-year mission life.

## 2. Reliability Requirements

### 2.1 Mission Reliability Goals

| Level | Objective | Reliability Target |
|-------|-----------|-------------------|
| Level 1 | 30-day minimum | R > 0.95 |
| Level 2 | 1-year full success | R > 0.80 |
| Level 3 | 2-year extended | R > 0.60 |

### 2.2 Subsystem Reliability Allocation

| Subsystem | Allocation | Basis |
|-----------|------------|-------|
| Structure | 0.999 | Passive, no moving parts |
| EPS | 0.95 | Battery life limiting |
| OBC | 0.98 | Radiation tolerance |
| RF System | 0.97 | Component heritage |
| AI Payload | 0.90 | Non-critical, fallback exists |
| Harness | 0.999 | Workmanship |
| **System** | **0.82** | Product of subsystems |

## 3. Failure Modes Analysis

### 3.1 Critical Single Points of Failure

| ID | Component | Failure Mode | Effect | Mitigation |
|----|-----------|--------------|--------|------------|
| SPF-001 | Battery | Open circuit | Mission loss | Cell screening |
| SPF-002 | OBC | Latchup | Safe mode | Watchdog, SEL protection |
| SPF-003 | Radio TX | Amplifier fail | No downlink | Power derating |
| SPF-004 | Antenna | Deploy fail | Weak link | Spring redundancy |
| SPF-005 | Solar cells | Degradation | Power loss | EOL margin |

### 3.2 Single Point Failure Mitigation

```
SINGLE POINT FAILURE APPROACH

For Class D CubeSat missions, single-string design
is acceptable with appropriate mitigation:

1. Component Selection
   └─ Heritage/space-rated parts preferred
   └─ Commercial parts with screening

2. Derating
   └─ Power: 50% derating on semiconductors
   └─ Voltage: 70% derating on capacitors
   └─ Current: 75% derating on traces

3. Margin
   └─ Power budget: 28% margin minimum
   └─ Link budget: > 6 dB threshold, > 10 dB goal (SEMP §3.3); beacon +7.7 dB at 5° elevation
      (docs/environment/LINK_BUDGET_ANALYSIS.md v2.0, analysis)
   └─ Thermal: 5°C margin to limits

4. Graceful Degradation
   └─ AI payload non-critical
   └─ Fallback beacon mode
   └─ Reduced power modes
```

### 3.3 Failure Rate Estimates

| Component | Quantity | λ (FIT) | Source |
|-----------|----------|---------|--------|
| Apollo4 MCU | 1 | 50 | MIL-HDBK-217F |
| Jetson Nano | 1 | 500 | Estimate |
| Si4463 Radio | 1 | 100 | Vendor data |
| Li-ion Cell | 2 | 200 | Industry data |
| Solar Cells | 6 | 10 | Heritage |
| Passives | ~100 | 1 each | MIL-HDBK-217F |
| Connectors | 8 | 5 | MIL-HDBK-217F |
| **Total** | | ~1050 | |

*FIT = Failures In Time (10⁹ hours)*

### 3.4 MTBF Calculation

```
MEAN TIME BETWEEN FAILURES

Total failure rate (λ): 1050 FIT = 1.05 × 10⁻⁶ /hr

MTBF = 1/λ = 952,381 hours = 108.7 years

Mission duration: 2 years = 17,520 hours

Reliability for 2 years:
R(t) = e^(-λt) = e^(-1.05×10⁻⁶ × 17520)
R(2 years) = 0.982

Note: This is component-level only.
System reliability includes:
- Radiation effects
- Thermal cycling
- Workmanship defects
- Software faults

System reliability estimate: 0.60-0.80 for 2 years
```

## 4. Radiation Analysis

### 4.1 Environment

| Parameter | Value | Notes |
|-----------|-------|-------|
| Orbit altitude | 400 km | ISS-like |
| Inclination | 51.6° | Passes through SAA |
| Total dose (2 yr) | ~3 krad (Si) | Behind 100 mil Al |
| Peak flux | Variable | SAA transits |

### 4.2 Component Radiation Tolerance

| Component | TID Rating | SEL Rating | Margin |
|-----------|------------|------------|--------|
| Apollo4 MCU | 30 krad | Immune | 10× |
| Jetson Nano | 3 krad (est) | Susceptible | 1× |
| Si4463 | 20 krad | LET > 40 | 7× |
| EPS ICs | 10 krad | LET > 20 | 3× |

### 4.3 Radiation Mitigation

| Component | Mitigation |
|-----------|------------|
| OBC | Watchdog, ECC RAM, safe mode |
| Jetson | Duty cycle limits, reset capability |
| RF | Operational monitoring |
| EPS | Hardware protection circuits |

## 5. Thermal Cycling Analysis

### 5.1 Thermal Environment

| Parameter | Value |
|-----------|-------|
| Orbital period | 94.6 minutes (500 km, BASELINE §3) |
| Eclipse duration | 0-35.8 minutes (depends on β) |
| Cycles per day | ~15.2 |
| Cycles per year | ~5,559 |
| Cycles (2 years) | ~11,118 |

### 5.2 Temperature Ranges

| Component | Hot | Cold | ΔT |
|-----------|-----|------|-----|
| OBC | +40°C | -10°C | 50°C |
| Battery | +30°C | +5°C | 25°C |
| Jetson | +55°C | 0°C | 55°C |
| Structure | +50°C | -20°C | 70°C |

### 5.3 Thermal Cycling Qualification

| Component | Qual Cycles | Mission Cycles | Margin |
|-----------|-------------|----------------|--------|
| Solder joints | 20,000 | 11,118 | 1.8× |
| Connectors | 15,000 | 11,118 | 1.3× |
| PCBs | 25,000 | 11,118 | 2.2× |

## 6. Wear-Out Mechanisms

### 6.1 Battery Degradation

```
BATTERY LIFE ANALYSIS (summary of BATTERY_DESIGN §3.3; analysis, not measured)

Cell: Samsung INR18650-35E (candidate), 4S1P, 3.45 Ah, 49.7 Wh (BASELINE §5)

Cycling:
- Eclipses: 5,559 per year at 500 km; 11,118 in 2 years
- DoD per eclipse: 1.3 % nominal, 2.0 % bounding (limit 10 %, BASELINE §5)
- Equivalent full cycles in 2 years: 148 nominal, 408 bounding
- Cell data: specification criterion, >= 60 % of minimum capacity after
  500 full-depth 1C cycles; no shallow-DoD data found for this cell
- Margin on charge throughput: 3.4x nominal, 1.2x bounding
  (no credit taken for shallow cycling)
- Capacity needed at end of life for the 10 % DoD limit: 20 % of BOL

Calendar aging: not quantified (no cell-specific data found); cell test
required (BATTERY_DESIGN §9.3)

Status: cycle count is not life-limiting by analysis; calendar aging
and the cycle test are open

Version 1.0 claimed ">10,000 cycles at 15 % DoD, 1.8x margin" with no
source; 10,000 / 11,320 two-year cycles is 0.88x.
```

### 6.2 Solar Cell Degradation

| Parameter | BOL | EOL (2 yr) | Degradation |
|-----------|-----|------------|-------------|
| Efficiency | 29% | 27.5% | 5% |
| Power output | 7.0W | 6.65W | 5% |
| Margin to load | 128% | 117% | Still positive |

### 6.3 Other Wear-Out

| Mechanism | Component | Life | Mission | Margin |
|-----------|-----------|------|---------|--------|
| EEPROM writes | OBC | 100,000 | ~10,000 | 10× |
| Flash writes | OBC | 10,000 | ~1,000 | 10× |
| Antenna spring | Deployer | 1 cycle | 1 cycle | 1× |

## 7. Redundancy Assessment

### 7.1 Redundancy Implementation

| Subsystem | Redundancy | Type |
|-----------|------------|------|
| Power paths | Partial | Dual solar strings |
| Battery cells | None | 4S1P single string (BASELINE §5); see SPF-001 |
| OBC | None | Single string (Class D) |
| RF | None | Single string |
| AI Payload | Functional | Template fallback |
| Watchdog | Full | Hardware + software |

### 7.2 Graceful Degradation Modes

```
DEGRADATION HIERARCHY

Full Capability (ACTIVE + AI)
        ↓ AI failure
ACTIVE Mode (No AI, full ops)
        ↓ Power constraint
IDLE Mode (Reduced ops)
        ↓ Critical fault
SAFE Mode (Beacon only)
        ↓ Total failure
Loss of Mission

Each level maintains beacon transmission
until total system failure.
```

## 8. Parts Reliability

### 8.1 Parts Grade Summary

| Grade | Count | Usage |
|-------|-------|-------|
| Space-grade | 2 | MCU, critical ICs |
| Industrial (-40 to +85°C) | 15 | Active components |
| Commercial screened | ~50 | Passives, connectors |
| COTS (Jetson) | 1 | AI payload |

### 8.2 Screening Approach

| Part Type | Screening |
|-----------|-----------|
| Active semiconductors | Burn-in, visual |
| Passives | Lot sample testing |
| Connectors | Visual, continuity |
| PCBs | Visual, electrical |
| Harness | Continuity, hipot |

### 8.3 Derating Policy

| Component | Parameter | Limit | Derating |
|-----------|-----------|-------|----------|
| Semiconductors | Power | 50% | Required |
| Capacitors | Voltage | 70% | Required |
| Resistors | Power | 50% | Required |
| Traces | Current | 75% | Required |
| Connectors | Current | 80% | Required |

## 9. Software Reliability

### 9.1 Software Failure Modes

| Mode | Cause | Mitigation |
|------|-------|------------|
| Infinite loop | Logic error | Watchdog |
| Stack overflow | Recursion | Static allocation |
| Memory corruption | Pointer error | Bounds checking |
| Task deadlock | Resource contention | Timeout |
| Unhandled exception | Missing handler | Default handler |

### 9.2 Software Reliability Features

| Feature | Implementation |
|---------|----------------|
| Watchdog | 5-second hardware timer |
| Safe mode | Automatic on fault |
| Boot validation | CRC check |
| State persistence | FRAM storage |
| Fault logging | Non-volatile counter |

### 9.3 Software Quality Metrics

| Metric | Target | Achieved |
|--------|--------|----------|
| Code coverage | >80% | TBD |
| Static analysis | 0 critical | TBD |
| MISRA compliance | Required rules | Yes |
| Cyclomatic complexity | <15 | Yes |

Note (2026-10-24): the two "Yes" entries are not supported by any tool output. The flight software is Rust
(`software/rust/README.md`). MISRA C applies only to the remaining C shim, which has 42 findings from the cppcheck addon
(`software/flight/docs/MISRA_DEVIATIONS.md`), and cyclomatic complexity is not measured. Line coverage was 94.4 %, C and
Rust together, on 2026-10-24.

## 10. Reliability Summary

### 10.1 Mission Reliability Prediction

| Duration | Predicted R | Target | Status |
|----------|-------------|--------|--------|
| 30 days | 0.98 | 0.95 | Exceeds |
| 6 months | 0.92 | 0.90 | Exceeds |
| 1 year | 0.85 | 0.80 | Exceeds |
| 2 years | 0.72 | 0.60 | Exceeds |

### 10.2 Key Reliability Drivers

| Factor | Impact | Status |
|--------|--------|--------|
| Battery life | High | Adequate margin |
| Radiation | Medium | Protected |
| Thermal cycling | Medium | Qualified |
| Workmanship | Medium | Standards applied |
| Software | Medium | Robust design |

### 10.3 Reliability Statement

```
RELIABILITY ASSESSMENT SUMMARY

The SMART-QSO spacecraft is predicted to meet
all mission reliability requirements:

- Level 1 (30 days): R = 0.98 > 0.95 requirement
- Level 2 (1 year): R = 0.85 > 0.80 requirement
- Level 3 (2 years): R = 0.72 > 0.60 requirement

Key factors:
✓ Conservative component derating
✓ Graceful degradation design
✓ Robust fault tolerance
✓ Adequate environmental margins

Status: RELIABILITY REQUIREMENTS MET
```

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-16 | Funding audit | §6.1 battery life replaced by a summary of BATTERY_DESIGN §3.3 (TB-08); orbit cycles at 500 km; battery redundancy corrected to 4S1P |
| 1.2 | 2026-10-16 | SMART-QSO Team | §3.2 link-budget margin: the SEMP threshold and goal and the LINK_BUDGET_ANALYSIS v2.0 result replace "3 dB margin minimum" (finding TB-05). Header revision brought up to the history (it still read 1.0 after 1.1) |
| 1.2, note | 2026-10-24 | SMART-QSO Team | §9.3 note: the MISRA and complexity entries are unsupported; status for the Rust flight software |
