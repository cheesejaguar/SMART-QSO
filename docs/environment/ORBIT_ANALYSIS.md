# SMART-QSO Orbit Analysis

**Document ID**: SQSO-OA-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the orbit analysis for the SMART-QSO mission, including ground contact analysis, eclipse predictions, and coverage calculations.

## 2. Orbital Elements

### 2.1 Reference Orbit (ISS-like)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Semi-major axis (a) | 6,778 km | 400 km altitude |
| Eccentricity (e) | 0.0001 | Near-circular |
| Inclination (i) | 51.6° | ISS inclination |
| RAAN (Ω) | Variable | Precesses ~5°/day |
| Arg. of perigee (ω) | N/A | Circular orbit |
| Period | 92.6 min | ~15.5 orbits/day |

### 2.2 Orbital Perturbations

| Effect | Magnitude | Impact |
|--------|-----------|--------|
| J2 (oblateness) | RAAN drift ~5°/day | Ground track shift |
| Atmospheric drag | ~1-5 km/year decay | Lifetime limit |
| Solar radiation | Negligible | - |
| Third body | Negligible | - |

### 2.3 Orbital Lifetime

```
ORBITAL LIFETIME ESTIMATE

Initial altitude:     400 km
Solar activity:       Moderate (F10.7 = 150)
Ballistic coefficient: 0.02 m²/kg
Drag area:            0.01 m² (1U)
Mass:                 1.2 kg

Estimated lifetime:   2-5 years (solar cycle dependent)
Compliance:           < 25 years (NASA-STD-8719.14) ✓
```

## 3. Ground Station Coverage

### 3.1 Primary Ground Station

| Parameter | Value |
|-----------|-------|
| Location | TBD (Example: 37.4°N, 122.0°W) |
| Minimum elevation | 10° |
| Antenna type | Yagi (tracking) |

### 3.2 Visibility Analysis

```
DAILY PASS STATISTICS (37.4°N latitude)

Passes per day:        4-6 visible
Average pass duration: 8 minutes
Maximum duration:      12 minutes (overhead)
Minimum duration:      4 minutes (low elevation)

Total contact/day:     30-50 minutes
```

### 3.3 Pass Geometry

| Pass Type | Elevation | Duration | Quality |
|-----------|-----------|----------|---------|
| Overhead | 80-90° | 10-12 min | Excellent |
| High | 45-80° | 8-10 min | Good |
| Medium | 20-45° | 6-8 min | Acceptable |
| Low | 10-20° | 4-6 min | Marginal |

### 3.4 Ground Track

```
GROUND TRACK PATTERN (51.6° inclination)

     90°N ─────────────────────────────────
          │     ╱╲     ╱╲     ╱╲     ╱╲
     51.6°│────╱──╲───╱──╲───╱──╲───╱──╲───
          │   ╱    ╲ ╱    ╲ ╱    ╲ ╱    ╲
       0° │──╱──────╳──────╳──────╳──────╲─
          │ ╱      ╱ ╲    ╱ ╲    ╱ ╲      ╲
    -51.6°│╱──────╱───╲──╱───╲──╱───╲──────╲
          │     ╱      ╲╱     ╲╱     ╲
     90°S ─────────────────────────────────
          0°    90°   180°   270°   360°
                    Longitude

Ground track repeats approximately every 3 days.
```

## 4. Eclipse Analysis

### 4.1 Eclipse Geometry

| Parameter | Value |
|-----------|-------|
| Maximum eclipse | 35.5 minutes |
| Minimum eclipse | 0 minutes (β > 71°) |
| Average eclipse | ~32 minutes |
| Sunlit fraction | 62% average |

### 4.2 Beta Angle Effects

| Beta Angle | Eclipse Duration | Sunlit % |
|------------|------------------|----------|
| 0° | 35.5 min | 62% |
| 30° | 33 min | 64% |
| 51.6° | 27 min | 71% |
| 71° | 0 min | 100% |

### 4.3 Annual Beta Angle Variation

```
BETA ANGLE ANNUAL VARIATION

Beta (°)
    75 ─┐        ╭────╮        ╭────╮
       │       ╱      ╲      ╱      ╲
    50 ─┤      ╱        ╲    ╱        ╲
       │     ╱          ╲  ╱          ╲
    25 ─┤    ╱            ╲╱            ╲
       │   ╱              │              ╲
     0 ─┴──╱───────────────┴───────────────╲─
       Jan    Apr    Jul    Oct    Jan

Sun-synchronous variation with 51.6° orbit
```

### 4.4 Eclipse Power Implications

| Beta Range | Eclipse | Battery Drain | Status |
|------------|---------|---------------|--------|
| 0-30° | 33-35 min | 0.12 Wh | OK |
| 30-50° | 27-33 min | 0.10 Wh | OK |
| 50-71° | 0-27 min | 0.08 Wh | OK |
| >71° | 0 min | 0 Wh | Best |

## 5. Lighting Conditions

### 5.1 Solar Illumination

| Condition | Occurrence | Impact |
|-----------|------------|--------|
| Full sun | 57 min/orbit avg | Charging |
| Eclipse | 35 min/orbit max | Battery drain |
| Terminator | 2 min × 2/orbit | Thermal transition |

### 5.2 Earth Albedo

| Surface | Albedo | Location |
|---------|--------|----------|
| Ocean | 0.06 | Tropics |
| Land | 0.15-0.25 | Temperate |
| Cloud | 0.50-0.80 | Variable |
| Ice/Snow | 0.80-0.90 | Polar |

Average albedo used: 0.30

## 6. Communication Windows

### 6.1 Ground Station Access

```
TYPICAL DAY CONTACT SCHEDULE

Time (UTC)    Elev    Duration    Quality
────────────────────────────────────────────
02:15         45°     8 min       Good
03:52         22°     6 min       Acceptable
14:30         78°     11 min      Excellent
16:08         35°     7 min       Good
17:45         18°     5 min       Marginal
────────────────────────────────────────────
Total:                37 min      5 passes
```

### 6.2 Global Station Network (Example)

| Station | Location | Passes/day |
|---------|----------|------------|
| Primary | California | 5-6 |
| Backup | Europe | 4-5 |
| Partner | Japan | 4-5 |
| Amateur | Worldwide | Variable |

### 6.3 Contact Gaps

| Scenario | Maximum Gap |
|----------|-------------|
| Single station | 16 hours |
| Two stations | 8 hours |
| Global network | 4 hours |

## 7. Orbital Events

### 7.1 Deployment Sequence

```
DEPLOYMENT TIMELINE

T+0:00:00   Separation from deployer
            - Separation springs fire
            - Deployment switches release

T+0:00:01   Tumbling begins
            - Initial rate: ~2°/s

T+0:30:00   Timer expires
            - Antenna deployment
            - RF activation

T+0:32:00   First beacon
            - Acquisition begins
```

### 7.2 Deorbit Timeline

| Altitude | Remaining Life |
|----------|----------------|
| 400 km | 2-5 years |
| 350 km | 6-12 months |
| 300 km | 2-4 months |
| 250 km | 2-4 weeks |
| 200 km | 2-4 days |

## 8. Conjunction Assessment

### 8.1 Debris Environment

| Size | Population | Collision Risk |
|------|------------|----------------|
| >10 cm | ~23,000 | Tracked |
| 1-10 cm | ~500,000 | Untracked |
| <1 cm | Millions | Shielding |

### 8.2 Conjunction Screening

| Parameter | Value |
|-----------|-------|
| Screening provider | 18 SDS / Space-Track |
| Warning threshold | Pc > 1e-5 |
| Alert threshold | Pc > 1e-4 |
| Action threshold | Pc > 1e-3 |

### 8.3 Collision Avoidance

SMART-QSO has no propulsion, therefore:
- Cannot perform avoidance maneuvers
- Must accept residual risk
- Will coordinate with tracking for awareness

## 9. Orbit Determination

### 9.1 Tracking Sources

| Source | Accuracy | Latency |
|--------|----------|---------|
| Space-Track TLE | 1 km | Hours |
| SatNOGS | 5 km | Real-time |
| Own observations | Variable | Real-time |

### 9.2 TLE Update Frequency

| Phase | Update Rate |
|-------|-------------|
| Post-deployment | Daily |
| Commissioning | Every 2-3 days |
| Normal ops | Weekly |
| Close approach | As needed |

## 10. Summary Tables

### 10.1 Orbit Parameters Summary

| Parameter | Value |
|-----------|-------|
| Altitude | 400 km |
| Inclination | 51.6° |
| Period | 92.6 min |
| Orbits/day | 15.5 |
| Eclipse (max) | 35.5 min |
| Lifetime | 2-5 years |

### 10.2 Contact Summary

| Parameter | Value |
|-----------|-------|
| Passes/day (single GS) | 4-6 |
| Contact time/day | 30-50 min |
| Max gap (single GS) | 16 hours |
| Data capacity/day | 252 KB |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
