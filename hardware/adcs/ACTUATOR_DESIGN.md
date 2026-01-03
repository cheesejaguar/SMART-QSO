# ADCS Actuator Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-ADCS-ACT-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the magnetorquer actuator design for the SMART-QSO 1U CubeSat ADCS.

### 1.1 Actuator Configuration

| Actuator | Quantity | Axis |
|----------|----------|------|
| Magnetorquer X | 1 | Body X-axis |
| Magnetorquer Y | 1 | Body Y-axis |
| Magnetorquer Z | 1 | Body Z-axis |

### 1.2 Performance Requirements

| Parameter | Requirement | Source |
|-----------|-------------|--------|
| Dipole moment (each) | > 0.1 Am² | Detumble analysis |
| Detumble time | < 24 hours | Mission |
| Power consumption | < 0.5 W total | EPS budget |

---

## 2. Magnetorquer Design

### 2.1 Coil Type Selection

| Type | Pros | Cons | Selected |
|------|------|------|----------|
| Air core | Simple, linear | Lower dipole | **Yes** |
| Permalloy core | Higher dipole | Nonlinear, heavier | No |
| PCB trace | Integrated | Low dipole | No |

### 2.2 Air Core Coil Specification

| Parameter | Value | Notes |
|-----------|-------|-------|
| Core type | Air core (wound on frame) | Linear response |
| Wire gauge | 30 AWG (0.254 mm) | Balance: resistance vs. turns |
| Coil form | Rectangular, on structure | Uses available space |

### 2.3 Dipole Moment Calculation

```
Dipole moment: M = N × I × A

Where:
  N = Number of turns
  I = Current (A)
  A = Coil area (m²)

For M = 0.15 Am²:
  A = 0.08 × 0.08 = 0.0064 m² (80mm × 80mm coil)
  I = 0.1 A (practical limit)
  N = M / (I × A) = 0.15 / (0.1 × 0.0064) = 234 turns

Select: N = 250 turns for margin
```

### 2.4 Coil Parameters (Each Axis)

| Parameter | Value |
|-----------|-------|
| Turns | 250 |
| Coil dimensions | 80mm × 80mm |
| Wire diameter | 0.254 mm (30 AWG) |
| Wire length | 250 × 0.32m = 80 m |
| Resistance | 80 × 0.339 Ω/m = 27Ω |
| Inductance | ~5 mH |
| Current (max) | 120 mA @ 3.3V |
| Dipole moment | 0.19 Am² @ 120 mA |
| Power | 0.4W @ max current |

---

## 3. Driver Circuit Design

### 3.1 H-Bridge Driver

| Parameter | Value |
|-----------|-------|
| Topology | Full H-bridge |
| IC | DRV8833, TB6612, or discrete |
| Supply | 3.3V or 5V |
| Current rating | 1A (margin) |
| PWM frequency | 20 kHz |

### 3.2 H-Bridge Operation

| State | Q1 | Q2 | Q3 | Q4 | Coil Current |
|-------|----|----|----|----|--------------|
| Off | Off | Off | Off | Off | 0 |
| +M | On | Off | Off | On | +I |
| -M | Off | On | On | Off | -I |
| Brake | On | On | Off | Off | 0 (decay) |

### 3.3 PWM Control

| Parameter | Value |
|-----------|-------|
| PWM resolution | 8-bit (256 levels) |
| PWM frequency | 20 kHz |
| Polarity control | Direction pin |
| Current range | 0 to 120 mA |

### 3.4 Current Sensing

| Method | Implementation |
|--------|----------------|
| Sense resistor | 1Ω in series |
| Amplifier | OPA333 (low power) |
| ADC | OBC internal |
| Resolution | 1 mA |

---

## 4. Mechanical Integration

### 4.1 Coil Placement

| Coil | Location | Orientation |
|------|----------|-------------|
| X-torquer | -Y face inner | Normal to +X |
| Y-torquer | -X face inner | Normal to +Y |
| Z-torquer | -Z face inner | Normal to +Z |

### 4.2 Coil Construction

| Parameter | Value |
|-----------|-------|
| Bobbin material | Delrin or PEEK |
| Winding method | Layer wound |
| Potting | Epoxy (low outgassing) |
| Mass (each) | ~20g |

### 4.3 Mounting

- Adhesive bonding to structure
- Strain relief on leads
- Conformal coating on terminations

---

## 5. Control Interface

### 5.1 OBC Interface

| Signal | Type | Function |
|--------|------|----------|
| MTQ_X_PWM | PWM | X-axis magnitude |
| MTQ_X_DIR | GPIO | X-axis polarity |
| MTQ_Y_PWM | PWM | Y-axis magnitude |
| MTQ_Y_DIR | GPIO | Y-axis polarity |
| MTQ_Z_PWM | PWM | PWM | Z-axis magnitude |
| MTQ_Z_DIR | GPIO | Z-axis polarity |
| MTQ_EN | GPIO | Enable all torquers |

### 5.2 Command Format

```c
typedef struct {
    int8_t x_command;  // -100 to +100 %
    int8_t y_command;  // -100 to +100 %
    int8_t z_command;  // -100 to +100 %
} MtqCommand_t;
```

---

## 6. B-Dot Control Algorithm

### 6.1 Algorithm

```
For each control axis i:
  B_dot[i] = (B[i] - B_prev[i]) / dt
  M_cmd[i] = -K * B_dot[i]

Where:
  K = control gain (Am²/(T/s))
  dt = sample period (typically 1s)
```

### 6.2 Control Parameters

| Parameter | Value |
|-----------|-------|
| Sample rate | 1 Hz |
| Gain K | 1×10⁶ Am²/(T/s) |
| Command limit | ±0.15 Am² |
| Deadband | 1 mGauss/s |

### 6.3 Duty Cycling

To avoid magnetometer interference:

| Phase | Duration | Activity |
|-------|----------|----------|
| Sense | 100 ms | Magnetorquers OFF, read mag |
| Act | 900 ms | Magnetorquers ON |

---

## 7. Detumble Performance

### 7.1 Detumble Analysis

| Parameter | Value |
|-----------|-------|
| Initial tip-off rate | < 10°/s (typical) |
| Earth field strength | 25-60 µT (LEO) |
| Maximum torque | M × B = 0.15 × 50×10⁻⁶ = 7.5×10⁻⁶ Nm |
| Moment of inertia | ~0.002 kg·m² (1U) |
| Angular deceleration | 7.5×10⁻⁶ / 0.002 = 0.004 rad/s² |

### 7.2 Detumble Time Estimate

```
From 10°/s to 0.1°/s:
Δω = 9.9°/s = 0.17 rad/s
Time = Δω / α = 0.17 / 0.004 = 43 s (ideal)

With duty cycling and control inefficiency:
Actual time: 2-12 hours typical
```

---

## 8. Protection and Safety

### 8.1 Overcurrent Protection

| Level | Action |
|-------|--------|
| 150 mA | Software limit |
| 200 mA | Hardware current limit |
| 500 mA | Fuse (if discrete) |

### 8.2 Thermal Protection

| Condition | Action |
|-----------|--------|
| Driver > 80°C | Reduce duty cycle |
| Driver > 100°C | Disable torquers |
| Coil temperature | Not directly monitored |

### 8.3 Demagnetization

During RF transmission or magnetometer calibration:
- Disable magnetorquers
- Allow 100 ms settling time

---

## 9. Testing Requirements

### 9.1 Coil Tests

| Test | Requirement |
|------|-------------|
| Resistance | 27Ω ±10% |
| Inductance | 5 mH ±20% |
| Dipole moment | > 0.1 Am² @ 100 mA |
| Polarity | Correct sign |

### 9.2 Driver Tests

| Test | Requirement |
|------|-------------|
| PWM linearity | < 5% error |
| Direction control | Correct polarity |
| Current accuracy | ±10% |
| Enable function | Verified |

### 9.3 System Tests

| Test | Requirement |
|------|-------------|
| Detumble (HITL) | < 24 hours simulated |
| Magnetometer interference | < 1 mGauss when off |
| Thermal stability | < 50°C at max duty |

---

## 10. Telemetry

### 10.1 Actuator Telemetry

| Point | Type | Rate |
|-------|------|------|
| MTQ X command | Int8 | 1 Hz |
| MTQ Y command | Int8 | 1 Hz |
| MTQ Z command | Int8 | 1 Hz |
| MTQ X current | Uint8 | 1 Hz |
| MTQ Y current | Uint8 | 1 Hz |
| MTQ Z current | Uint8 | 1 Hz |
| Driver temperature | Int8 | 0.1 Hz |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
