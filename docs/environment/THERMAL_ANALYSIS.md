# SMART-QSO Thermal Analysis

**Document ID**: SQSO-THA-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the thermal analysis for the SMART-QSO mission, demonstrating all components remain within operational temperature limits throughout the mission.

## 2. Thermal Environment

### 2.1 Orbital Parameters

| Parameter | Value |
|-----------|-------|
| Altitude | 400 km |
| Inclination | 51.6° |
| Beta Angle Range | -75° to +75° |
| Eclipse Duration | 0-35 minutes |
| Orbit Period | 92.6 minutes |

### 2.2 Environmental Heat Loads

| Source | Hot Case | Cold Case | Units |
|--------|----------|-----------|-------|
| Solar Flux | 1414 | 1322 | W/m² |
| Albedo | 0.35 | 0.25 | - |
| Earth IR | 258 | 218 | W/m² |

### 2.3 Thermal Cases

| Case | Description | Conditions |
|------|-------------|------------|
| Hot | Maximum temperature | β=0°, max solar, max internal |
| Cold | Minimum temperature | β=75°, eclipse, min internal |
| Nominal | Typical operations | Average conditions |

## 3. Component Temperature Limits

### 3.1 Operational Limits

| Component | Min Op (°C) | Max Op (°C) | Min Surv (°C) | Max Surv (°C) |
|-----------|-------------|-------------|---------------|---------------|
| OBC (Apollo4) | -40 | +85 | -55 | +125 |
| Battery (Li-ion) | -10 | +45 | -20 | +60 |
| Jetson Orin | -25 | +70 | -40 | +85 |
| RF Transceiver | -30 | +60 | -40 | +85 |
| Solar Cells | -100 | +100 | -150 | +125 |
| Structure | -40 | +80 | -100 | +120 |
| Antenna | -40 | +80 | -55 | +100 |

### 3.2 Critical Components

| Component | Concern | Mitigation |
|-----------|---------|------------|
| Battery | Narrow operating range | Heater + insulation |
| Jetson | High heat generation | Thermal throttling |
| RF PA | Heat during TX | Duty cycle limit |

## 4. Thermal Model

### 4.1 Model Description

```
THERMAL NODE MODEL (10 nodes)

         ┌─────────────────────────────────┐
         │         SOLAR PANEL             │
         │           (Node 1)              │
         └──────────────┬──────────────────┘
                        │ Conduction
         ┌──────────────┴──────────────────┐
         │           STRUCTURE             │
         │           (Node 2)              │
         └─┬─────┬─────┬─────┬─────┬─────┬─┘
           │     │     │     │     │     │
         ┌─┴─┐ ┌─┴─┐ ┌─┴─┐ ┌─┴─┐ ┌─┴─┐ ┌─┴─┐
         │OBC│ │EPS│ │BAT│ │ RF│ │JET│ │ANT│
         │ 3 │ │ 4 │ │ 5 │ │ 6 │ │ 7 │ │ 8 │
         └───┘ └───┘ └───┘ └───┘ └───┘ └───┘
                   ↓
         ┌─────────────────────────────────┐
         │        SPACE (Sink)             │
         │         (Node 10)               │
         └─────────────────────────────────┘
```

### 4.2 Thermal Properties

| Node | Mass (g) | Cp (J/kg·K) | Area (m²) | ε | α |
|------|----------|-------------|-----------|---|---|
| Solar | 50 | 700 | 0.06 | 0.85 | 0.92 |
| Structure | 400 | 900 | 0.06 | 0.85 | 0.3 |
| OBC | 30 | 900 | 0.004 | 0.8 | - |
| EPS | 50 | 900 | 0.005 | 0.8 | - |
| Battery | 200 | 1000 | 0.008 | 0.8 | - |
| RF | 40 | 900 | 0.004 | 0.8 | - |
| Jetson | 100 | 900 | 0.01 | 0.8 | - |
| Antenna | 20 | 900 | 0.003 | 0.8 | - |

### 4.3 Internal Heat Dissipation

| Component | SAFE (W) | IDLE (W) | ACTIVE (W) | Peak (W) |
|-----------|----------|----------|------------|----------|
| OBC | 0.05 | 0.08 | 0.10 | 0.15 |
| EPS | 0.03 | 0.03 | 0.03 | 0.03 |
| Battery | 0.02 | 0.02 | 0.05 | 0.10 |
| RF | 0.02 | 0.05 | 0.50 | 2.00 |
| Jetson | 0.00 | 0.00 | 5.00 | 15.00 |
| Heater | 0-0.5 | 0-0.5 | 0-0.5 | 0.50 |
| **Total** | **0.12-0.62** | **0.18-0.68** | **5.68-6.18** | **17.78** |

## 5. Thermal Analysis Results

### 5.1 Hot Case Results

**Conditions**: β=0°, max solar, ACTIVE mode with AI burst

| Component | Limit (°C) | Predicted (°C) | Margin (°C) | Status |
|-----------|------------|----------------|-------------|--------|
| OBC | +85 | +45 | +40 | ✓ OK |
| EPS | +85 | +42 | +43 | ✓ OK |
| Battery | +45 | +38 | +7 | ✓ OK |
| RF (TX) | +60 | +52 | +8 | ✓ OK |
| Jetson | +70 | +65 | +5 | ⚠ Marginal |
| Structure | +80 | +55 | +25 | ✓ OK |

**Critical Finding**: Jetson approaches limit during AI inference. Thermal throttling required above 60°C.

### 5.2 Cold Case Results

**Conditions**: β=75°, extended eclipse, SAFE mode

| Component | Limit (°C) | Predicted (°C) | Margin (°C) | Status |
|-----------|------------|----------------|-------------|--------|
| OBC | -40 | -15 | +25 | ✓ OK |
| EPS | -40 | -12 | +28 | ✓ OK |
| Battery | -10 | +2 | +12 | ✓ OK* |
| RF | -30 | -10 | +20 | ✓ OK |
| Jetson | -25 | -5 | +20 | ✓ OK |
| Structure | -40 | -25 | +15 | ✓ OK |

*Battery heater activates below +5°C to maintain temperature.

### 5.3 Nominal Case Results

**Conditions**: Average β, typical IDLE/ACTIVE cycling

| Component | Predicted Range (°C) | Status |
|-----------|----------------------|--------|
| OBC | +15 to +35 | ✓ OK |
| EPS | +12 to +32 | ✓ OK |
| Battery | +10 to +30 | ✓ OK |
| RF | +5 to +45 | ✓ OK |
| Jetson | +0 to +55 | ✓ OK |
| Structure | +5 to +40 | ✓ OK |

### 5.4 Transient Analysis

```
THERMAL TRANSIENT - ECLIPSE ENTRY/EXIT

Temperature (°C)
     50 ┤                                    ╭───
        │                                 ╭──╯
     40 ┤                              ╭──╯
        │                           ╭──╯
     30 ┤    ╭─────────────────────╯
        │ ╭──╯
     20 ┤─╯
        │   ← Eclipse Entry    Eclipse Exit →
     10 ┤
        └─────────────────────────────────────
        0    10    20    30    40    50    60
                     Time (minutes)

Component: Battery
Eclipse Duration: 35 minutes
Min Temp: +10°C (with heater)
Recovery Time: 15 minutes after eclipse exit
```

## 6. Thermal Control System

### 6.1 Passive Control

| Element | Implementation | Purpose |
|---------|----------------|---------|
| Surface Coatings | α/ε selection | Balance absorption/emission |
| MLI | Battery wrap | Reduce radiative losses |
| Thermal Straps | Cu straps to structure | Conduct heat to radiator |
| Radiator | Structure exterior | Reject excess heat |

### 6.2 Active Control

| Element | Implementation | Purpose |
|---------|----------------|---------|
| Battery Heater | 0.5W Kapton heater | Maintain >0°C in eclipse |
| Jetson Throttling | Software control | Limit power at high temp |
| Mode Control | SAFE mode entry | Reduce heat generation |

### 6.3 Heater Control Logic

```
BATTERY HEATER CONTROL

IF battery_temp < 5°C AND heater_power_available:
    heater_on = True
    target_temp = 10°C
ELIF battery_temp > 12°C:
    heater_on = False
ENDIF

Hysteresis: 7°C (5°C on, 12°C off)
Max heater power: 0.5W
Priority: Battery survival > operations
```

### 6.4 Jetson Thermal Throttling

| Threshold | Action |
|-----------|--------|
| T < 50°C | Full performance |
| 50°C ≤ T < 60°C | 75% clock |
| 60°C ≤ T < 65°C | 50% clock |
| T ≥ 65°C | Shutdown |

## 7. Thermal Margin Summary

### 7.1 Margin Analysis

| Component | Hot Margin | Cold Margin | Status |
|-----------|------------|-------------|--------|
| OBC | +40°C | +25°C | ✓ Excellent |
| EPS | +43°C | +28°C | ✓ Excellent |
| Battery | +7°C | +12°C | ⚠ Adequate |
| RF | +8°C | +20°C | ⚠ Adequate |
| Jetson | +5°C | +20°C | ⚠ Marginal |
| Structure | +25°C | +15°C | ✓ Good |

### 7.2 Margin Requirements

| Phase | Requirement | Status |
|-------|-------------|--------|
| CDR | ≥5°C margin | ✓ Met |
| Flight | ≥3°C margin | Expected |

## 8. Thermal Verification

### 8.1 Analysis Verification

| Method | Status |
|--------|--------|
| Hand calculations | Complete |
| Spreadsheet model | Complete |
| Thermal Desktop (if available) | TBD |

### 8.2 Test Verification

| Test | Purpose | Status |
|------|---------|--------|
| Component thermal | Verify limits | [ ] |
| Thermal balance | Validate model | [ ] |
| Thermal vacuum | Flight qualification | [ ] |

### 8.3 Flight Verification

| Verification | Method |
|--------------|--------|
| Temperature trending | Telemetry analysis |
| Heater operation | Power consumption |
| Throttling events | Mode telemetry |

## 9. Recommendations

### 9.1 Design Recommendations

1. **Battery**: Maintain MLI wrap, verify heater sizing
2. **Jetson**: Implement thermal throttling in software
3. **RF PA**: Limit TX duty cycle to 5% maximum
4. **General**: Add thermal telemetry for all critical components

### 9.2 Operational Recommendations

1. Limit AI inference when Jetson temp > 60°C
2. Avoid high-power TX during hot orbital phases
3. Monitor battery heater power consumption
4. Trend temperatures for degradation detection

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
