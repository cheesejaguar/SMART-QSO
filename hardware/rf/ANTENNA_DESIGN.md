# Antenna Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-RF-ANT-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the antenna design for the SMART-QSO 1U CubeSat VHF beacon downlink system.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Frequency | 145.825 MHz | IARU Coordination |
| Polarization | Circular or Linear | Link Budget |
| Gain | > 0 dBi omnidirectional | Link Budget |
| Impedance | 50Ω | Standard |
| VSWR | < 2:1 | Design |
| Deployment | Autonomous post-separation | CDS |

---

## 2. Antenna Type Selection

### 2.1 Trade Study

| Type | Gain | Pattern | Complexity | Selected |
|------|------|---------|------------|----------|
| Quarter-wave monopole | 2.1 dBi | Omnidirectional | Low | **Yes** |
| Dipole | 2.1 dBi | Omnidirectional | Medium | No |
| Turnstile | 0 dBi | Hemispherical CP | High | No |
| Patch | 6 dBi | Directional | Medium | No |

### 2.2 Selected Configuration

**Quarter-Wave Monopole with CubeSat Ground Plane**

| Parameter | Value |
|-----------|-------|
| Type | λ/4 monopole |
| Element length | ~49 cm (145.825 MHz) |
| Ground plane | CubeSat body (-Z face) |
| Gain | +2.1 dBi (ideal) |
| Pattern | Toroidal (omnidirectional in XY) |

---

## 3. Antenna Element Design

### 3.1 Element Material Options

| Material | Form | Advantages | Disadvantages |
|----------|------|------------|---------------|
| Stainless steel tape | Tape spring | Self-deploying, robust | Heavier |
| Nitinol (SMA) | Wire | Shape memory, compact | Expensive |
| Beryllium copper | Tape | Good conductivity | Brittle |
| Steel music wire | Wire | Cheap, springy | Corrosion |

### 3.2 Selected Material

| Parameter | Value |
|-----------|-------|
| Material | 301 Stainless Steel |
| Form | Tape spring (measuring tape style) |
| Width | 6 mm |
| Thickness | 0.1 mm |
| Length | 490 mm (λ/4 at 145.825 MHz) |

### 3.3 Electrical Length Calculation

```
λ = c / f = 3×10⁸ / 145.825×10⁶ = 2.057 m
λ/4 = 514 mm (free space)
Velocity factor ≈ 0.95 for tape element
Physical length = 514 × 0.95 = 488 mm ≈ 490 mm
```

---

## 4. Antenna Base Design

### 4.1 Base Assembly

| Component | Material | Purpose |
|-----------|----------|---------|
| Base plate | Aluminum | Ground/mounting |
| Insulator | PEEK | RF isolation |
| Feed pin | Brass, gold plated | RF connection |
| Spring contact | BeCu | Element connection |

### 4.2 RF Connection

| Parameter | Value |
|-----------|-------|
| Connector | SMA female |
| Feed impedance | 50Ω |
| Matching | L-network if needed |

---

## 5. Deployment Mechanism

### 5.1 Stowed Configuration

| Parameter | Value |
|-----------|-------|
| Coiling method | Flat coil around base |
| Coil diameter | < 80 mm |
| Retention | Nylon monofilament |
| Burn wire | Nichrome, 30 AWG |

### 5.2 Deployment Actuation

| Parameter | Value |
|-----------|-------|
| Method | Resistive burn wire |
| Wire material | Nichrome |
| Wire resistance | ~5Ω |
| Burn current | 0.5-1.0 A |
| Burn time | 2-5 seconds |
| Redundancy | Dual burn wires |

### 5.3 Deployment Sequence

1. OBC receives deployment command
2. Wait T+30 min from separation (per CDS)
3. Activate burn wire circuit
4. Current flows, wire heats, nylon melts
5. Tape spring self-deploys
6. Deployment switch confirms deployment
7. If no confirmation, retry with backup wire

### 5.4 Deployment Switch

| Parameter | Value |
|-----------|-------|
| Type | Microswitch |
| Actuation | Deployed element presses |
| Output | GPIO to OBC |
| Redundancy | Optional second switch |

---

## 6. RF Performance

### 6.1 Expected Performance

| Parameter | Value | Notes |
|-----------|-------|-------|
| Gain (zenith) | +2.1 dBi | Ideal monopole |
| Gain (horizon) | -3 dBi | Pattern rolloff |
| Gain (-Z) | -10 dBi | Below ground plane |
| Bandwidth | > 5 MHz | 3 dB points |
| VSWR | < 1.5:1 | At center frequency |

### 6.2 Radiation Pattern

- Omnidirectional in XY plane (azimuth)
- Maximum gain at 20-30° above horizon
- Null at zenith (directly above)
- Reduced gain below ground plane

### 6.3 Polarization

| Parameter | Value |
|-----------|-------|
| Polarization | Linear (vertical) |
| Axial ratio | N/A (linear) |
| Cross-pol | > 20 dB |

---

## 7. Matching Network

### 7.1 Impedance Considerations

| Condition | Impedance |
|-----------|-----------|
| Ideal λ/4 monopole | 36.5Ω |
| With finite ground plane | 40-50Ω |
| Target | 50Ω |

### 7.2 Matching Approach

If impedance < 50Ω, use series inductance:
- L = (Z₀ × tan(β)) / (2πf)
- Typical: 10-30 nH series inductor

If VSWR acceptable (< 2:1), no matching needed.

---

## 8. Ground Plane Considerations

### 8.1 CubeSat Body as Ground Plane

| Parameter | Value |
|-----------|-------|
| Size | 100mm × 100mm minimum |
| Material | Aluminum structure |
| Configuration | -Z face of CubeSat |
| Grounding | Bonded to RF ground |

### 8.2 Effect on Pattern

- Finite ground plane causes pattern distortion
- Some gain reduction vs. infinite ground
- Acceptable for omnidirectional coverage

---

## 9. Environmental Considerations

### 9.1 Thermal

| Condition | Effect | Mitigation |
|-----------|--------|------------|
| Hot (+80°C) | Slight length increase | Design margin |
| Cold (-40°C) | Slight length decrease | Design margin |
| Cycling | Material fatigue | SS tape is robust |

### 9.2 Atomic Oxygen (LEO)

| Material | AO Effect | Mitigation |
|----------|-----------|------------|
| Stainless steel | Minimal | None needed |
| PEEK insulator | Slight erosion | Acceptable for 1 year |

### 9.3 Outgassing

All materials are low-outgassing (see MATERIALS_LIST.md).

---

## 10. Mechanical Interface

### 10.1 Mounting

| Parameter | Value |
|-----------|-------|
| Location | -Z face corner |
| Footprint | 20mm × 20mm |
| Fasteners | M2 × 4 |
| Keep-out zone | 30mm radius for stowed element |

### 10.2 Stowed Envelope

| Parameter | Value |
|-----------|-------|
| Diameter | 80 mm |
| Height | 10 mm |
| Protrusion | None beyond CubeSat envelope |

### 10.3 Deployed Configuration

| Parameter | Value |
|-----------|-------|
| Length | 490 mm |
| Direction | -Z axis |
| Clearance | No interference with solar panels |

---

## 11. Testing Requirements

### 11.1 Pre-Integration Tests

| Test | Requirement |
|------|-------------|
| Impedance | 50Ω ±10Ω |
| VSWR | < 2:1 |
| Resonant frequency | 145.825 MHz ±1 MHz |

### 11.2 Deployment Tests

| Test | Requirement |
|------|-------------|
| Deployment time | < 5 seconds |
| Deployment success | 10/10 cycles |
| Temperature range | -20°C to +50°C |

### 11.3 RF Pattern Measurement

| Test | Requirement |
|------|-------------|
| Gain measurement | > 0 dBi average |
| Pattern shape | Toroidal, omnidirectional |
| Efficiency | > 80% |

---

## 12. Verification

### 12.1 Analysis

- RF simulation (HFSS, CST, or 4NEC2)
- Pattern prediction
- Impedance matching

### 12.2 Test

- VNA measurement of S11
- Anechoic chamber pattern measurement
- Deployment functional test

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
