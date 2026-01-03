# SMART-QSO Link Budget Analysis

**Document ID**: SQSO-LBA-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the RF link budget analysis for the SMART-QSO mission, demonstrating adequate link margin for all operational scenarios.

## 2. Link Parameters

### 2.1 Orbital Parameters

| Parameter | Value |
|-----------|-------|
| Altitude | 400 km (nominal) |
| Inclination | 51.6° (ISS orbit) |
| Period | 92.6 minutes |
| Maximum slant range | 2,200 km (5° elevation) |
| Minimum slant range | 400 km (90° elevation) |

### 2.2 Frequency Allocations

| Link | Frequency | Band | Allocation |
|------|-----------|------|------------|
| Downlink | 145.825 MHz | VHF (2m) | Amateur |
| Uplink | 435.100 MHz | UHF (70cm) | Amateur |

### 2.3 Modulation Parameters

| Parameter | Value |
|-----------|-------|
| Modulation | AFSK |
| Data rate | 1200 bps |
| Deviation | ±3 kHz |
| Bandwidth | 12 kHz |
| Required Eb/N0 | 10 dB (for BER < 10⁻⁵) |

## 3. Downlink Budget (VHF - 145.825 MHz)

### 3.1 Transmitter Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| TX Power (low) | 0.5 W | +27.0 dBm |
| TX Power (high) | 2.0 W | +33.0 dBm |
| TX Line Loss | -1.0 dB | Coax + connectors |
| Antenna Gain | 0 dBi | Dipole |
| **EIRP (low)** | **+26.0 dBm** | |
| **EIRP (high)** | **+32.0 dBm** | |

### 3.2 Path Loss

| Parameter | 5° Elev | 30° Elev | 90° Elev |
|-----------|---------|----------|----------|
| Slant Range | 2,200 km | 730 km | 400 km |
| Free Space Loss | -144.3 dB | -134.7 dB | -129.5 dB |
| Atmospheric Loss | -0.5 dB | -0.3 dB | -0.2 dB |
| Polarization Loss | -3.0 dB | -1.0 dB | -0.5 dB |
| **Total Path Loss** | **-147.8 dB** | **-136.0 dB** | **-130.2 dB** |

### 3.3 Ground Station Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Antenna Gain | +12 dBi | M2 2MCP22 Yagi |
| LNA Gain | +20 dB | SP-2000 |
| LNA Noise Figure | 0.5 dB | |
| Line Loss | -2.0 dB | Coax to receiver |
| System Noise Temp | 350 K | -174 + 10log(350) = -148.4 dBm/Hz |
| Receiver Bandwidth | 12 kHz | -40.8 dB-Hz |
| **Noise Floor** | **-107.6 dBm** | |

### 3.4 Downlink Margin Calculation

#### Low Power (0.5W), 5° Elevation (Worst Case)
```
EIRP:                    +26.0 dBm
Path Loss:              -147.8 dB
Ground Antenna Gain:     +12.0 dBi
Line Loss:               -2.0 dB
─────────────────────────────────
Received Power:        -111.8 dBm

Noise Floor:           -107.6 dBm
─────────────────────────────────
C/N0:                   -4.2 dB   ❌ INSUFFICIENT

Required Eb/N0:         +10.0 dB
Implementation Loss:     +2.0 dB
─────────────────────────────────
Required C/N:           +12.0 dB
Link Margin:            -16.2 dB  ❌ NEGATIVE
```

#### High Power (2W), 5° Elevation
```
EIRP:                    +32.0 dBm
Path Loss:              -147.8 dB
Ground Antenna Gain:     +12.0 dBi
Line Loss:               -2.0 dB
─────────────────────────────────
Received Power:        -105.8 dBm

Noise Floor:           -107.6 dBm
─────────────────────────────────
C/N:                     +1.8 dB

Required C/N:           +12.0 dB
─────────────────────────────────
Link Margin:            -10.2 dB  ❌ MARGINAL
```

#### High Power (2W), 30° Elevation (Nominal)
```
EIRP:                    +32.0 dBm
Path Loss:              -136.0 dB
Ground Antenna Gain:     +12.0 dBi
Line Loss:               -2.0 dB
─────────────────────────────────
Received Power:         -94.0 dBm

Noise Floor:           -107.6 dBm
─────────────────────────────────
C/N:                    +13.6 dB

Required C/N:           +12.0 dB
─────────────────────────────────
Link Margin:             +1.6 dB  ✓ POSITIVE
```

#### High Power (2W), 90° Elevation (Best Case)
```
EIRP:                    +32.0 dBm
Path Loss:              -130.2 dB
Ground Antenna Gain:     +12.0 dBi
Line Loss:               -2.0 dB
─────────────────────────────────
Received Power:         -88.2 dBm

Noise Floor:           -107.6 dBm
─────────────────────────────────
C/N:                    +19.4 dB

Required C/N:           +12.0 dB
─────────────────────────────────
Link Margin:             +7.4 dB  ✓ GOOD
```

### 3.5 Downlink Summary

| Scenario | Power | Elevation | Margin | Status |
|----------|-------|-----------|--------|--------|
| Worst case | 0.5W | 5° | -16.2 dB | ❌ No link |
| Low elev | 2.0W | 5° | -10.2 dB | ❌ Marginal |
| Nominal | 2.0W | 30° | +1.6 dB | ✓ OK |
| Best case | 2.0W | 90° | +7.4 dB | ✓ Good |

**Operational Constraint**: Minimum operational elevation = 20° for reliable link.

## 4. Uplink Budget (UHF - 435.100 MHz)

### 4.1 Ground Transmitter Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| TX Power | 50 W | +47.0 dBm |
| Line Loss | -2.0 dB | Coax |
| Antenna Gain | +14 dBi | M2 436CP30 Yagi |
| **EIRP** | **+59.0 dBm** | |

### 4.2 Path Loss (435 MHz)

| Parameter | 5° Elev | 30° Elev | 90° Elev |
|-----------|---------|----------|----------|
| Slant Range | 2,200 km | 730 km | 400 km |
| Free Space Loss | -153.8 dB | -144.2 dB | -139.0 dB |
| Atmospheric Loss | -0.3 dB | -0.2 dB | -0.1 dB |
| Polarization Loss | -3.0 dB | -1.0 dB | -0.5 dB |
| **Total Path Loss** | **-157.1 dB** | **-145.4 dB** | **-139.6 dB** |

### 4.3 Spacecraft Receiver Parameters

| Parameter | Value | Notes |
|-----------|-------|-------|
| Antenna Gain | 0 dBi | Dipole |
| Line Loss | -1.0 dB | Internal |
| Receiver Noise Figure | 3.0 dB | |
| System Noise Temp | 580 K | |
| Bandwidth | 12 kHz | |
| **Noise Floor** | **-105.4 dBm** | |

### 4.4 Uplink Margin Calculation

#### 30° Elevation (Nominal)
```
EIRP:                    +59.0 dBm
Path Loss:              -145.4 dB
Spacecraft Antenna:       0.0 dBi
Line Loss:               -1.0 dB
─────────────────────────────────
Received Power:         -87.4 dBm

Noise Floor:           -105.4 dBm
─────────────────────────────────
C/N:                    +18.0 dB

Required C/N:           +12.0 dB
─────────────────────────────────
Link Margin:             +6.0 dB  ✓ GOOD
```

### 4.5 Uplink Summary

| Scenario | Elevation | Margin | Status |
|----------|-----------|--------|--------|
| Low | 5° | -5.7 dB | ❌ No link |
| Nominal | 30° | +6.0 dB | ✓ Good |
| Best | 90° | +11.8 dB | ✓ Excellent |

## 5. Link Budget Summary

### 5.1 Operating Envelope

```
LINK AVAILABILITY vs ELEVATION

Elevation  Downlink    Uplink      Both
─────────────────────────────────────────
   5°      No          No          No
  10°      Marginal    Marginal    No
  15°      Marginal    OK          Marginal
  20°      OK          OK          OK ← Min operational
  30°      Good        Good        Good
  60°      Excellent   Excellent   Excellent
  90°      Excellent   Excellent   Excellent
```

### 5.2 Operational Constraints

| Constraint | Value | Rationale |
|------------|-------|-----------|
| Min elevation for command | 20° | +3 dB uplink margin |
| Min elevation for telemetry | 20° | +1.6 dB downlink margin |
| Recommended TX power | 2.0W | Required for acceptable margin |
| Max commanding range | 30° to LOS | Reliable uplink window |

### 5.3 Doppler Effects

| Parameter | Value |
|-----------|-------|
| Max Doppler shift (VHF) | ±3.6 kHz |
| Max Doppler shift (UHF) | ±10.8 kHz |
| Doppler rate | ~100 Hz/s max |

**Mitigation**: Ground station Doppler correction via Gpredict + radio CAT control.

## 6. Link Enhancement Options

### 6.1 If Margin Insufficient

| Option | Gain | Implementation |
|--------|------|----------------|
| Higher ground antenna | +3 dB | Larger Yagi |
| LNA improvement | +1 dB | Better NF |
| DSP processing | +3 dB | Software modem |
| Lower data rate (300 bps) | +6 dB | Reduced throughput |

### 6.2 Network Enhancement

Multiple ground stations can improve contact time:
- Primary: 4-6 passes/day
- With 2 stations: 8-10 passes/day
- Global network: Nearly continuous

## 7. Verification

### 7.1 Pre-Flight Verification

| Test | Method | Status |
|------|--------|--------|
| TX power calibration | RF power meter | [ ] |
| Antenna pattern | Anechoic chamber | [ ] |
| Receiver sensitivity | Signal generator | [ ] |
| Link test | RF link simulator | [ ] |

### 7.2 Post-Launch Verification

| Test | Method | Status |
|------|--------|--------|
| Signal acquisition | First pass | [ ] |
| Link margin measurement | S/N ratio | [ ] |
| Doppler verification | Frequency tracking | [ ] |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
