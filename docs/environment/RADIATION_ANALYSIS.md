# Radiation Environment Analysis
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-ENV-002
**Revision:** A
**Date:** 2026-01-02
**Reference:** NASA-HDBK-4002A

---

## 1. Purpose

This document analyzes the radiation environment for the SMART-QSO CubeSat mission and defines the radiation design requirements and mitigation strategies.

## 2. Mission Orbit Parameters

| Parameter | Value |
|-----------|-------|
| Altitude | 500 km |
| Inclination | 51.6° (ISS orbit) |
| Orbit type | LEO, circular |
| Mission duration | 12 months (design life) |
| Launch date | TBD |
| Solar cycle | Approaching solar maximum |

## 3. Radiation Environment

### 3.1 Trapped Radiation (Van Allen Belts)

| Component | Annual Fluence | Notes |
|-----------|----------------|-------|
| Trapped protons (>10 MeV) | 2×10⁸ p/cm² | Inner belt |
| Trapped electrons (>1 MeV) | 5×10⁹ e/cm² | Inner belt |
| Peak proton energy | 100 MeV | |
| Peak electron energy | 5 MeV | |

### 3.2 Galactic Cosmic Rays (GCR)

| Parameter | Value |
|-----------|-------|
| GCR flux | ~4 particles/cm²/s |
| Composition | 87% protons, 12% helium, 1% heavy ions |
| Energy range | 10 MeV to >10 GeV |
| Annual dose | ~0.1 krad (Si) |

### 3.3 Solar Particle Events (SPE)

| Parameter | Typical | Worst-case |
|-----------|---------|------------|
| Proton fluence (>10 MeV) | 10⁹ p/cm² | 10¹¹ p/cm² |
| Event duration | 1-3 days | 1 week |
| Annual frequency | 5-10 events | - |
| Peak flux | 10⁴ p/cm²/s | 10⁶ p/cm²/s |

### 3.4 South Atlantic Anomaly (SAA)

| Parameter | Value |
|-----------|-------|
| Geographic extent | -50° to 0° lat, -90° to 40° lon |
| Passes per day | 6-8 |
| Time in SAA | ~15 min/pass |
| Dose rate increase | 100-1000× background |

## 4. Total Ionizing Dose (TID) Analysis

### 4.1 Annual Dose Estimates

| Source | Dose Behind 1mm Al | Dose Behind 2mm Al |
|--------|--------------------|--------------------|
| Trapped protons | 5 krad | 2 krad |
| Trapped electrons | 1 krad | 0.3 krad |
| Solar protons | 0.5 krad | 0.2 krad |
| GCR | 0.1 krad | 0.1 krad |
| **Total** | **6.6 krad** | **2.6 krad** |

### 4.2 Component Dose Requirements

| Component | Location | Shielding | Annual Dose | Margin |
|-----------|----------|-----------|-------------|--------|
| OBC MCU | Center | 2mm Al equiv | 2.6 krad | 10× |
| Jetson | Center | 2mm Al | 2.6 krad | 4× |
| RF PA | Edge | 1.5mm Al | 4 krad | 25× |
| EEPROM | Center | 2mm Al | 2.6 krad | 4× |
| Solar cells | External | None | 10 krad | 100× |

### 4.3 Dose-Depth Curve

```
Dose Rate vs Shielding Depth (500 km, 51.6°)

Dose (krad/yr)
100 ┤
    │█
 50 ┤██
    │███
 20 ┤████
    │█████
 10 ┤██████
    │███████
  5 ┤████████
    │█████████
  2 ┤██████████
    │███████████
  1 ┤████████████
    └──────────────────────────────
    0  0.5  1  1.5  2  2.5  3  3.5 mm Al
```

## 5. Single Event Effects (SEE)

### 5.1 SEE Types and Rates

| Effect | Description | Rate Estimate |
|--------|-------------|---------------|
| SEU | Bit flip in memory | 10⁻⁷ errors/bit/day |
| MBU | Multiple bit upset | 10⁻⁹ errors/bit/day |
| SET | Transient on output | 10⁻⁶ events/device/day |
| SEL | Latchup | 10⁻¹⁰ events/device/day |
| SEFI | Functional interrupt | 10⁻⁸ events/device/day |
| SEB | Burnout | < 10⁻¹² events/device/day |

### 5.2 Component SEE Susceptibility

| Component | SEU Rate | SEL Threshold | Notes |
|-----------|----------|---------------|-------|
| STM32L4 | Low | >60 MeV-cm²/mg | Rad-tolerant process |
| EEPROM | Moderate | N/A | No latchup mode |
| Flash | Moderate | >80 MeV-cm²/mg | Monitor and scrub |
| SRAM | Higher | >40 MeV-cm²/mg | TMR critical data |
| Jetson | Higher | Unknown | Monitor power, reset |

### 5.3 Expected SEU Rates

| Memory | Size | SEU Rate | Events/Day |
|--------|------|----------|------------|
| OBC SRAM | 640 KB | 10⁻⁷/bit/day | ~0.5 |
| OBC Flash | 2 MB | 10⁻⁸/bit/day | ~0.2 |
| Jetson RAM | 8 GB | 10⁻⁶/bit/day | ~700 |

## 6. Radiation Design Requirements

### 6.1 TID Requirements

| Requirement | Value | Basis |
|-------------|-------|-------|
| TID capability | >10 krad | 4× margin on 2.5 krad annual |
| Component screening | Commercial | COTS with heritage |
| Lot testing | Sample only | CubeSat approach |

### 6.2 SEE Requirements

| Requirement | Value |
|-------------|-------|
| SEU tolerance | Detect and correct all SEU |
| SEL tolerance | Survive with power cycle |
| SET tolerance | Filtered at critical inputs |
| SEFI recovery | Watchdog reset |

## 7. Mitigation Strategies

### 7.1 TID Mitigation

| Strategy | Implementation |
|----------|----------------|
| Shielding | 1-2 mm Al equivalent enclosure |
| Component selection | Use rad-tolerant parts where available |
| Derating | Operate below max ratings |
| Redundancy | Critical function backup |

### 7.2 SEU Mitigation

| Strategy | Application |
|----------|-------------|
| ECC memory | Hardware ECC on SRAM |
| TMR | Critical state variables |
| CRC | Stored data integrity |
| Scrubbing | Periodic memory read/write |
| Voting | Redundant sensors |

### 7.3 SEL Mitigation

| Strategy | Implementation |
|----------|----------------|
| Current limiting | All power rails limited |
| Overcurrent detection | Fast trip < 100 ms |
| Power cycling | Automatic recovery |
| Event logging | Trend analysis |

### 7.4 Software Mitigation

| Strategy | Implementation |
|----------|----------------|
| Watchdog | 30-second timeout |
| State recovery | Periodic checkpointing |
| Safe mode | Minimal operation on fault |
| Memory protection | Stack guards, heap checks |

## 8. Component Radiation Data

### 8.1 Key Components

| Part | TID Tolerance | SEL LET | SEU Cross-section |
|------|---------------|---------|-------------------|
| STM32L4R5 | 30 krad | >60 MeV-cm²/mg | <10⁻¹⁴ cm²/bit |
| W25Q128 Flash | 20 krad | N/A | 10⁻¹⁴ cm²/bit |
| LIS3MDL | 15 krad | >37 MeV-cm²/mg | 10⁻¹² cm²/bit |
| TPS62840 | 50 krad | >80 MeV-cm²/mg | N/A |

### 8.2 Jetson Orin Nano

| Parameter | Value | Notes |
|-----------|-------|-------|
| TID tolerance | ~10 krad | Estimated, not tested |
| SEU susceptibility | High | Consumer silicon |
| Mitigation | Power monitoring, watchdog | Reset on anomaly |
| Operating time | Duty cycled | Reduce accumulated dose |

## 9. SAA Operations

### 9.1 SAA Detection

| Method | Implementation |
|--------|----------------|
| Geographic | Latitude/longitude bounds |
| Rate monitoring | Increased SEU detection |
| Orbit prediction | Pre-computed SAA times |

### 9.2 SAA Mitigation Actions

| Action | Purpose |
|--------|---------|
| Increase scrub rate | Faster SEU correction |
| Defer writes | Reduce corruption risk |
| Disable Jetson | Protect AI payload |
| Log events | Trend analysis |

## 10. Verification

### 10.1 Analysis

| Item | Status |
|------|--------|
| Environment model | AE9/AP9 |
| Shielding analysis | SHIELDOSE-2 |
| SEE rate prediction | CRÈME96 |

### 10.2 Test

| Test | Status |
|------|--------|
| Component TID | Sample lot (if available) |
| SEE test | Heritage data |
| System radiation | Not planned (CubeSat) |

## 11. Risk Assessment

| Risk | Likelihood | Consequence | Mitigation |
|------|------------|-------------|------------|
| TID degradation | Low | Moderate | Component selection, shielding |
| Memory SEU | High | Low | ECC, TMR, scrubbing |
| Processor SEFI | Moderate | Moderate | Watchdog reset |
| Latchup | Low | High | Current limit, power cycle |
| SPE damage | Low | High | Safe mode during event |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
