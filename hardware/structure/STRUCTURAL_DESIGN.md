# Structural Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-STR-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the structural design for the SMART-QSO 1U CubeSat per CubeSat Design Specification (CDS) Rev 14.1.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Form factor | 1U (100×100×113.5 mm) | CDS Rev 14.1 |
| Mass | ≤ 1.33 kg | CDS Rev 14.1 |
| CG location | ≤ 2 cm from geometric center | CDS Rev 14.1 |
| Rail dimensions | 8.5×8.5 mm | CDS Rev 14.1 |
| Rail surface finish | Ra ≤ 1.6 µm | CDS Rev 14.1 |
| First natural frequency | > 90 Hz | Launch provider |

---

## 2. Structural Configuration

### 2.1 Primary Structure

| Component | Material | Mass |
|-----------|----------|------|
| Main frame | Al 7075-T6 | 150g |
| Rails (4×) | Al 7075-T6 | 40g |
| End plates (2×) | Al 6061-T6 | 30g |
| Internal brackets | Al 6061-T6 | 20g |
| **Total structure** | | **240g** |

### 2.2 Frame Configuration

- Monocoque-style frame with integrated rails
- Internal PCB card guides
- Access panels on ±Y faces
- Through-holes for harness routing

---

## 3. Material Selection

### 3.1 Primary Materials

| Material | Application | Properties |
|----------|-------------|------------|
| Al 7075-T6 | Main frame, rails | High strength, σy=503 MPa |
| Al 6061-T6 | Brackets, panels | Good machinability |
| A286 SS | Fasteners | High strength, non-magnetic |
| Ti-6Al-4V | Shims (if needed) | Low CTE, high strength |

### 3.2 Material Properties

| Property | Al 7075-T6 | Al 6061-T6 |
|----------|------------|------------|
| Density | 2810 kg/m³ | 2700 kg/m³ |
| Yield strength | 503 MPa | 276 MPa |
| Ultimate strength | 572 MPa | 310 MPa |
| Young's modulus | 71.7 GPa | 68.9 GPa |
| CTE | 23.6 µm/m·K | 23.6 µm/m·K |
| Thermal conductivity | 130 W/m·K | 167 W/m·K |

---

## 4. Mass Properties

### 4.1 Mass Budget

| Subsystem | Mass (g) | % Total |
|-----------|----------|---------|
| Structure | 240 | 22% |
| EPS (battery, solar) | 200 | 18% |
| OBC | 50 | 5% |
| RF | 80 | 7% |
| ADCS | 60 | 5% |
| Jetson payload | 250 | 23% |
| Harness | 50 | 5% |
| Thermal hardware | 30 | 3% |
| Margin | 130 | 12% |
| **Total** | **1090** | **100%** |

### 4.2 Center of Gravity

| Axis | Location | Offset from Center |
|------|----------|-------------------|
| X | 50 mm | < 5 mm |
| Y | 50 mm | < 5 mm |
| Z | 56.75 mm | < 5 mm |

### 4.3 Moments of Inertia

| Axis | Value (kg·m²) |
|------|---------------|
| Ixx | 0.0020 |
| Iyy | 0.0020 |
| Izz | 0.0015 |

---

## 5. Mechanical Interfaces

### 5.1 Rail Specifications (CDS Rev 14.1)

| Parameter | Requirement | Design |
|-----------|-------------|--------|
| Rail width | 8.5 +0/-0.1 mm | 8.5 mm |
| Rail height | 8.5 +0/-0.1 mm | 8.5 mm |
| Rail length | 113.5 ±0.1 mm | 113.5 mm |
| Surface finish | Ra ≤ 1.6 µm | Ra 0.8 µm |
| Edge radius | 1.0 mm max | 0.5 mm |
| Material | 7075 Al anodized | Hard anodize |

### 5.2 Deployment Switch Interface

| Parameter | Value |
|-----------|-------|
| Switch quantity | 2 (redundant) |
| Location | -Z rail corners |
| Actuation force | 0.5-2.0 N |
| Travel | 1-2 mm |
| Contact | Normally closed |

### 5.3 Separation Springs

| Parameter | Value |
|-----------|-------|
| Spring quantity | 4 (one per rail) |
| Separation velocity | 0.5-2.0 m/s |
| Angular tip-off | < 10°/s |

---

## 6. Deployment Mechanisms

### 6.1 Antenna Deployment

| Parameter | Value |
|-----------|-------|
| Type | Burn-wire release |
| Location | -Z face |
| Stowed volume | 80mm dia × 10mm |
| Deployment direction | -Z axis |

### 6.2 Solar Panel Deployment (If Applicable)

| Parameter | Value |
|-----------|-------|
| Type | Spring-loaded hinge |
| Panels | 2× (±X faces) |
| Deployment angle | 90° |
| Retention | Nylon line + burn wire |

---

## 7. Structural Analysis

### 7.1 Load Cases

| Load Case | Description | Factor of Safety |
|-----------|-------------|------------------|
| Launch | Quasi-static + random vibe | 1.25 (yield) |
| Handling | Drop, shock | 2.0 (yield) |
| Thermal | -40°C to +70°C | 1.0 |

### 7.2 Quasi-Static Loads

| Axis | Load (g) | Notes |
|------|----------|-------|
| Axial | ±10 g | Launch axis |
| Lateral | ±6 g | Perpendicular |
| Combined | RSS | Worst case |

### 7.3 Random Vibration

| Frequency | ASD (g²/Hz) | Notes |
|-----------|-------------|-------|
| 20 Hz | 0.01 | Start |
| 50 Hz | 0.04 | Rise |
| 800 Hz | 0.04 | Plateau |
| 2000 Hz | 0.01 | Roll-off |
| Overall | 6.8 grms | Duration: 2 min/axis |

### 7.4 FEA Results Summary

| Mode | Frequency | Description |
|------|-----------|-------------|
| 1st mode | 150 Hz | X-axis bending |
| 2nd mode | 155 Hz | Y-axis bending |
| 3rd mode | 210 Hz | Torsion |

### 7.5 Margin Summary

| Location | Stress (MPa) | Allowable (MPa) | MS |
|----------|--------------|-----------------|-----|
| Rail root | 120 | 400 | +2.3 |
| Bracket joint | 80 | 220 | +1.75 |
| PCB mount | 40 | 180 | +3.5 |

---

## 8. Fastener Specifications

### 8.1 Fastener Types

| Application | Fastener | Torque |
|-------------|----------|--------|
| Structure assembly | M2.5×6 A286 | 0.35 N·m |
| PCB mounting | M2×4 A286 | 0.15 N·m |
| Access panels | M2×6 A286 | 0.15 N·m |
| Helicoil inserts | M2.5×1D | As specified |

### 8.2 Thread Locking

| Application | Method |
|-------------|--------|
| Primary structure | Loctite 242 |
| PCB standoffs | Loctite 222 |
| Access panels | None (frequent access) |

---

## 9. Venting

### 9.1 Venting Requirements

| Parameter | Requirement |
|-----------|-------------|
| Vent rate | 1 atm in 300 s |
| Vent area | > 4 mm² |
| Location | Internal cavities |

### 9.2 Vent Implementation

- Machined vent holes in structure
- Gap around PCB edges
- No hermetically sealed volumes

---

## 10. CAD Model

### 10.1 CAD Deliverables

| Format | Purpose |
|--------|---------|
| Native CAD | Design master |
| STEP AP214 | Exchange format |
| 2D drawings | Manufacturing |
| STL | 3D printing (prototypes) |

### 10.2 Drawing Package

- Assembly drawing with BOM
- Detail drawings for all machined parts
- Fastener callouts with torque specs
- Surface finish callouts
- GD&T per ASME Y14.5

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
