# SMART-QSO Power Budget Analysis

**Document ID**: SQSO-PBA-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the power budget analysis for the SMART-QSO mission, demonstrating positive energy balance across all operational scenarios.

## 2. Power System Overview

### 2.1 Solar Array

| Parameter | Value |
|-----------|-------|
| Cell Type | Triple-junction GaAs |
| Configuration | Body-mounted + deployable wings |
| Total Area | 0.06 m² (6U equivalent) |
| Cell Efficiency | 30% (BOL) |
| Packing Factor | 85% |
| Pointing Loss | 30% average (tumble/spin) |
| EOL Degradation | 15% (2 years) |

### 2.2 Battery

| Parameter | Value |
|-----------|-------|
| Chemistry | Li-ion (18650 cells) |
| Configuration | 2S2P |
| Capacity | 5.2 Ah @ 7.4V nominal |
| Energy | 38.5 Wh |
| Usable DOD | 70% (15% to 85% SOC) |
| Usable Energy | 27.0 Wh |
| Cycle Life | >500 cycles @ 70% DOD |

### 2.3 Power Conditioning

| Parameter | Value |
|-----------|-------|
| Bus Voltage | 3.3V regulated |
| Conversion Efficiency | 90% |
| Battery Charging | CC/CV, max 1A |

## 3. Power Generation

### 3.1 Solar Input Calculation

```
SOLAR ARRAY POWER GENERATION

Solar Constant:           1361 W/m²
Array Area:               0.06 m²
Raw Incident Power:       81.7 W

Cell Efficiency:          30%
Packing Factor:           85%
Pointing Efficiency:      70% (average)
EOL Degradation:          85%
─────────────────────────────────
Generated Power (EOL):    12.3 W peak

Average over orbit:       ~6.2 W (50% sun, cosine losses)
```

### 3.2 Orbital Illumination

| Orbit Phase | Duration | Illumination |
|-------------|----------|--------------|
| Sunlight | 57 min | 62% |
| Eclipse | 35 min | 38% |
| **Total** | **92 min** | |

### 3.3 Average Power Generation

| Scenario | Avg Power | Notes |
|----------|-----------|-------|
| Best case (sun-pointing) | 8.0 W | Optimal orientation |
| Nominal (tumbling) | 6.2 W | Random orientation |
| Worst case (edge-on) | 3.5 W | Poor orientation |
| Eclipse | 0 W | Battery only |

## 4. Power Consumption

### 4.1 Subsystem Power

| Subsystem | SAFE | IDLE | ACTIVE | Peak |
|-----------|------|------|--------|------|
| OBC | 50 mW | 80 mW | 100 mW | 150 mW |
| EPS | 30 mW | 30 mW | 30 mW | 30 mW |
| RF (RX) | 20 mW | 50 mW | 100 mW | 100 mW |
| RF (TX) | 0 | 0 | 500-2000 mW | 2000 mW |
| Sensors | 10 mW | 20 mW | 30 mW | 50 mW |
| Jetson | 0 | 0 | 0-10000 mW | 15000 mW |
| Heater | 0-500 mW | 0-500 mW | 0-500 mW | 500 mW |
| **Total** | **110-610 mW** | **180-680 mW** | **760-12,760 mW** | **17,830 mW** |

### 4.2 Mode Power Summary

| Mode | Typical | Maximum | Duration |
|------|---------|---------|----------|
| SAFE | 100 mW | 600 mW | As needed |
| IDLE | 200 mW | 700 mW | ~80% of time |
| ACTIVE (no AI) | 500 mW | 2,500 mW | ~18% |
| ACTIVE (AI burst) | 10,000 mW | 15,000 mW | <2% (60s/10min) |
| TX (beacon) | +500-2000 mW | 2,000 mW | ~2s/60s |

### 4.3 Beacon Power Profile

```
BEACON TRANSMISSION POWER PROFILE

Time (s)    Power (mW)    Activity
────────────────────────────────────
0           200           IDLE baseline
0.1         200           TX enable
0.2         700           PA warm-up
0.5         2,200         Full TX power
2.0         2,200         TX complete
2.1         700           PA cool-down
2.5         200           Return to IDLE
```

### 4.4 AI Inference Power Profile

```
AI INFERENCE POWER PROFILE

Time (s)    Power (mW)    Activity
────────────────────────────────────
0           500           ACTIVE baseline
1           1,000         Jetson boot
10          3,000         Model loading
20          10,000        Inference running
50          10,000        Inference complete
55          1,000         Jetson idle
60          500           Jetson power-off
```

## 5. Energy Balance Analysis

### 5.1 Orbit-Average Power

```
ORBIT ENERGY BALANCE - NOMINAL OPERATIONS

GENERATION:
Sunlight duration:        57 min × 6.2 W = 353.4 Wh-min
Eclipse duration:         35 min × 0 W   =   0.0 Wh-min
Total generated:          353.4 Wh-min = 5.89 Wh/orbit

CONSUMPTION (IDLE mode):
Sunlight:                 57 min × 0.20 W = 11.4 Wh-min
Eclipse:                  35 min × 0.20 W =  7.0 Wh-min
Total consumed:           18.4 Wh-min = 0.31 Wh/orbit

NET ENERGY:               +5.58 Wh/orbit ✓ POSITIVE
```

### 5.2 Active Operations Scenario

```
ORBIT ENERGY BALANCE - ACTIVE WITH AI

GENERATION:               5.89 Wh/orbit

CONSUMPTION:
IDLE (80 min):            80 min × 0.20 W = 0.27 Wh
AI burst (1 × 60s):       1 min × 10.0 W  = 0.17 Wh
Beacon TX (2 × 2s):       0.07 min × 2.0 W = 0.002 Wh
Total consumed:           0.44 Wh/orbit

NET ENERGY:               +5.45 Wh/orbit ✓ POSITIVE
```

### 5.3 Worst Case Scenario

```
ORBIT ENERGY BALANCE - WORST CASE

GENERATION (poor pointing): 3.5 W average
Sunlight:                 57 min × 3.5 W = 199.5 Wh-min = 3.33 Wh

CONSUMPTION (maximum ops):
ACTIVE baseline:          92 min × 0.5 W = 0.77 Wh
AI bursts (2 × 60s):      2 min × 10 W   = 0.33 Wh
Beacon TX (max):          2 min × 2 W    = 0.07 Wh
Heater (eclipse):         35 min × 0.5 W = 0.29 Wh
Total consumed:           1.46 Wh/orbit

NET ENERGY:               +1.87 Wh/orbit ✓ POSITIVE (marginal)
```

### 5.4 Eclipse Survival

```
ECLIPSE SURVIVAL ANALYSIS

Battery capacity:         27.0 Wh (usable)
Eclipse duration:         35 minutes
SAFE mode power:          0.1 W

Energy required:          35 min × 0.1 W = 0.058 Wh
Battery capacity:         27.0 Wh

Eclipse cycles possible:  465 eclipses (at full DOD)
Margin:                   99.8% ✓ EXCELLENT
```

## 6. Power Budget Summary

### 6.1 Margin Analysis

| Scenario | Generated | Consumed | Margin |
|----------|-----------|----------|--------|
| Nominal IDLE | 5.89 Wh | 0.31 Wh | +1800% |
| Active + AI | 5.89 Wh | 0.44 Wh | +1239% |
| Worst case | 3.33 Wh | 1.46 Wh | +128% |
| **Requirement** | - | - | **>20%** |

### 6.2 AI Duty Cycle Limit

```
AI DUTY CYCLE CALCULATION

Available power margin:    5.89 - 0.31 = 5.58 Wh/orbit
AI power consumption:      10 W
Maximum AI time:           5.58 Wh / 10 W = 0.558 h = 33.5 min/orbit

Orbit duration:            92 min
Maximum duty cycle:        33.5 / 92 = 36%

Conservative limit:        10% (for margin and battery life)
AI time allowed:           9.2 min/orbit
```

### 6.3 Operating Modes vs SOC

| SOC Range | Allowed Modes | Rationale |
|-----------|---------------|-----------|
| 85-100% | ACTIVE + AI | Full capability |
| 70-85% | ACTIVE (no AI) | Preserve margin |
| 30-70% | IDLE | Charge battery |
| 20-30% | IDLE (reduced) | Low power warning |
| 15-20% | SAFE | Critical threshold |
| <15% | SAFE (emergency) | Minimum ops |

## 7. Power Constraints

### 7.1 Operational Constraints

| Constraint | Value | Rationale |
|------------|-------|-----------|
| Min SOC for AI | 70% | 30% margin to IDLE threshold |
| AI duty cycle | ≤10% | Battery life preservation |
| Max TX duty | 5% | Thermal + power |
| Heater priority | Below 0°C | Battery protection |

### 7.2 Fault Responses

| Condition | Response |
|-----------|----------|
| SOC < 30% | Disable AI, reduce beacon rate |
| SOC < 20% | Enter SAFE mode |
| SOC < 15% | Emergency minimum operations |
| Charge fault | SAFE mode, disable charging |

## 8. Power Verification

### 8.1 Ground Test

| Test | Method | Status |
|------|--------|--------|
| Subsystem power | Current measurement | [ ] |
| Mode transitions | Power profiling | [ ] |
| Solar array output | Solar simulator | [ ] |
| Battery capacity | Charge/discharge cycle | [ ] |

### 8.2 Flight Verification

| Test | Method | Status |
|------|--------|--------|
| Power generation | SOC trending | [ ] |
| Eclipse survival | Monitor through eclipse | [ ] |
| Energy balance | Multi-orbit analysis | [ ] |

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
