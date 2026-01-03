# Thermal Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-THM-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the thermal design for the SMART-QSO 1U CubeSat for LEO mission operations.

### 1.1 Thermal Design Requirements

| Component | Op Min (°C) | Op Max (°C) | Survival Min (°C) | Survival Max (°C) |
|-----------|-------------|-------------|-------------------|-------------------|
| OBC | -20 | +70 | -40 | +85 |
| Battery | -10 | +45 | -20 | +60 |
| RF electronics | -20 | +60 | -40 | +85 |
| Jetson module | 0 | +70 | -20 | +85 |
| Solar cells | -40 | +85 | -100 | +100 |
| Structure | -40 | +80 | -60 | +100 |

---

## 2. Orbital Environment

### 2.1 Orbital Parameters

| Parameter | Value |
|-----------|-------|
| Altitude | 400 km (ISS orbit) |
| Inclination | 51.6° |
| Beta angle range | -75° to +75° |
| Eclipse duration | 0-36 minutes |
| Orbit period | 92 minutes |
| Mission duration | 1 year |

### 2.2 Environmental Heat Loads

| Source | Value | Notes |
|--------|-------|-------|
| Solar flux | 1367 W/m² | AM0 |
| Earth albedo | 0.30 (avg) | Variable |
| Earth IR | 237 W/m² | Average |

---

## 3. Surface Properties

### 3.1 External Surfaces

| Surface | Finish | α (solar) | ε (IR) | α/ε |
|---------|--------|-----------|--------|-----|
| +Z face | Black anodize | 0.88 | 0.85 | 1.04 |
| -Z face | Black anodize | 0.88 | 0.85 | 1.04 |
| ±X faces | Solar cells | 0.92 | 0.85 | 1.08 |
| ±Y faces | Black anodize | 0.88 | 0.85 | 1.04 |

### 3.2 Internal Surfaces

| Surface | Finish | ε (IR) |
|---------|--------|--------|
| PCB | FR4 green | 0.90 |
| Components | Various | 0.85 |
| Structure | Bare Al | 0.05 |

---

## 4. Thermal Model

### 4.1 Node Definition

| Node | Component | Capacitance (J/K) |
|------|-----------|-------------------|
| 1 | OBC PCB | 15 |
| 2 | Battery | 120 |
| 3 | RF PCB | 10 |
| 4 | Jetson module | 80 |
| 5 | ADCS PCB | 8 |
| 6 | Structure (+X) | 100 |
| 7 | Structure (-X) | 100 |
| 8 | Structure (+Y) | 100 |
| 9 | Structure (-Y) | 100 |
| 10 | Structure (+Z) | 50 |
| 11 | Structure (-Z) | 50 |

### 4.2 Conductance Network

| From | To | Conductance (W/K) | Path |
|------|-----|-------------------|------|
| OBC | Structure | 0.5 | Standoffs |
| Battery | Structure | 1.0 | Thermal pad |
| RF | Structure | 0.3 | Standoffs |
| Jetson | Structure | 2.0 | Heatspreader |
| ADCS | Structure | 0.3 | Standoffs |

---

## 5. Thermal Analysis Cases

### 5.1 Hot Case

| Parameter | Value |
|-----------|-------|
| Beta angle | 0° (no eclipse) |
| Solar cells facing Sun | Maximum |
| Power dissipation | Maximum (Jetson burst) |
| Heaters | Off |

### 5.2 Cold Case

| Parameter | Value |
|-----------|-------|
| Beta angle | 75° |
| Maximum eclipse | 36 minutes |
| Power dissipation | Minimum (safe mode) |
| Heaters | As required |

### 5.3 Analysis Results

| Component | Hot Case (°C) | Cold Case (°C) | Op Limit | Margin |
|-----------|---------------|----------------|----------|--------|
| OBC | +52 | -8 | -20/+70 | OK |
| Battery | +38 | +5 | -10/+45 | OK |
| RF | +45 | -5 | -20/+60 | OK |
| Jetson (burst) | +68 | +15 | 0/+70 | Marginal |
| ADCS | +40 | -2 | -20/+70 | OK |
| Structure | +55 | -25 | -40/+80 | OK |

---

## 6. Thermal Control Hardware

### 6.1 Passive Thermal Control

| Element | Location | Purpose |
|---------|----------|---------|
| Black anodize | External | Radiator |
| Thermal pads | Battery, Jetson | Conduction |
| Heatspreader | Jetson | Heat spreading |
| MLI | Not used | N/A for 1U |

### 6.2 Active Thermal Control

| Element | Location | Power | Activation |
|---------|----------|-------|------------|
| Battery heater | Battery pack | 0.5W | < -5°C |
| (Jetson heater) | Heatspreader | (0.5W) | (Optional) |

### 6.3 Heater Sizing

```
Battery heater:
  Q_loss = k × A × ΔT / L
  For ΔT = 15°C, A = 0.002 m², k = 0.3 W/m·K:
  Q_loss ≈ 0.5W

  Heater power: 0.5W (matches loss)
  Thermostat setpoint: +0°C (on), +5°C (off)
```

---

## 7. Thermal Interfaces

### 7.1 Jetson Thermal Stack

| Layer | Material | Thickness | Conductivity |
|-------|----------|-----------|--------------|
| Jetson die | Silicon | - | - |
| TIM 1 | Fujipoly | 0.5 mm | 3 W/m·K |
| Heatspreader | Al 6061 | 3 mm | 167 W/m·K |
| TIM 2 | Gap pad | 1 mm | 2 W/m·K |
| Structure | Al 7075 | 3 mm | 130 W/m·K |

### 7.2 Battery Thermal Interface

| Layer | Material | Thickness | Conductivity |
|-------|----------|-----------|--------------|
| Cell case | Aluminum | 0.3 mm | 200 W/m·K |
| Thermal pad | Bergquist | 1 mm | 3 W/m·K |
| Bracket | Al 6061 | 2 mm | 167 W/m·K |

---

## 8. Thermal Straps (If Required)

### 8.1 Strap Specification

| Parameter | Value |
|-----------|-------|
| Material | Copper braid |
| Cross-section | 5 mm² |
| Conductance | 0.5 W/K per 100mm |
| Application | Jetson to radiator (optional) |

---

## 9. Thermal Testing Requirements

### 9.1 Thermal Balance Test

| Parameter | Requirement |
|-----------|-------------|
| Environment | Thermal vacuum chamber |
| Temperature range | -30°C to +60°C (shroud) |
| Vacuum level | < 10⁻⁵ Torr |
| Duration | Until steady state (2+ hours) |
| Correlation | ±5°C model accuracy |

### 9.2 Thermal Cycling Test

| Parameter | Requirement |
|-----------|-------------|
| Hot plateau | +50°C |
| Cold plateau | -20°C |
| Dwell time | 2 hours minimum |
| Cycles | 4 (protoflight) |
| Functional test | Each plateau |

---

## 10. Thermal Telemetry

### 10.1 Temperature Sensors

| Location | Sensor | Range | Accuracy |
|----------|--------|-------|----------|
| OBC board | NTC thermistor | -40 to +85°C | ±1°C |
| Battery 1 | NTC thermistor | -40 to +85°C | ±1°C |
| Battery 2 | NTC thermistor | -40 to +85°C | ±1°C |
| Jetson | Internal sensor | 0 to +100°C | ±2°C |
| Structure | NTC thermistor | -40 to +85°C | ±1°C |

### 10.2 Heater Telemetry

| Point | Type |
|-------|------|
| Heater status | On/Off |
| Heater current | mA |
| Thermostat state | Open/Closed |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
