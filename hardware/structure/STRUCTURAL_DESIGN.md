# Structural Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-STR-001
**Version:** 1.1
**Date:** 2026-10-16
**Status:** Draft. No finite-element model (FEM), structural analysis or structural test exists

> **Funding audit (2026-10-16, TB-15).** Launch environments in this document now come from one source: **SpaceX
> Rideshare Payload User's Guide v11, August 2026** ("UG v11",
> <https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf>) as the **interim provider,
> pending the launch contract and PI confirmation** (BASELINE §3, decision D7). The integrator's ICD will supersede it.
> Earlier values with no source (first mode > 90 Hz, ±10 g / ±6 g quasi-static, a 6.8 Grms profile, mode frequencies and
> margins of safety) are removed or marked **analysis pending FEM**. The same environment is used by
> `docs/test_procedures/TP-ENV-002_Vibration.md` and `docs/compliance/LAUNCH_PROVIDER_ICD.md` §4.5 and §6.2.

---

## 1. Overview

This document defines the structural design for the SMART-QSO 1U CubeSat per CubeSat Design Specification (CDS) Rev 14.1.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Form factor | 1U (100×100×113.5 mm) | CDS Rev 14.1 |
| Mass | ≤ 2.00 kg | CDS Rev 14.1 (BASELINE §4; MEV ≤ 1.75 kg) |
| CG location | ≤ 2 cm from geometric center | CDS Rev 14.1 |
| Rail dimensions | 8.5×8.5 mm | CDS Rev 14.1 |
| Rail surface finish | Ra ≤ 1.6 µm | CDS Rev 14.1 |
| First natural frequency | > 40 Hz (no elastic mode below 40 Hz) | SpaceX Rideshare Payload User's Guide v11 §4.1.1 (interim); BASELINE §8 |
| Random vibration | ≥ 5.57 Grms MPE (§7.3) | SpaceX Rideshare Payload User's Guide v11 §4.1.6, Table 4-6 |
| Quasi-static load | 10 g axial, 17 g lateral (RSS) (§7.2) | SpaceX Rideshare Payload User's Guide v11 Table 4-1 (interim; see §7.2) |
| Factors of safety | 1.10 yield; 1.25 ultimate (flight); 1.40 ultimate (ground operations) | SpaceX Rideshare Payload User's Guide v11 Table 5-1 |

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
| Edge radius | Rounded to a radius of **at least 1 mm** (CDS Rev 14.1 §2.2.7, a "should" requirement) | ≥ 1.0 mm (to be shown on the drawings; the earlier 0.5 mm value did not meet the CDS) |
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

Minimum factors of safety from UG v11 Table 5-1 (interim provider):

| Load Case | Description | Factor of Safety |
|-----------|-------------|------------------|
| Launch | Quasi-static (§7.2) and random vibration (§7.3) | 1.10 yield; 1.25 ultimate |
| Joints | Gapping and slip at interfaces | 1.10 |
| Ground operations | Handling and transport | 1.10 yield; 1.40 ultimate |
| Thermal | Thermal stress over the predicted temperature range | TBD (range and factor to be set with the thermal analysis) |

### 7.2 Quasi-Static Loads

| Axis | Load (g) | Notes |
|------|----------|-------|
| Axial | 10 | UG v11 Table 4-1 |
| Lateral | 17 (RSS of the two lateral axes) | UG v11 Table 4-1 |

These are the maximum predicted design load factors for an **integrated CubeSat dispenser assembly** with a combined mass
of at least 20 kg (UG v11 §4.1.2.1), not CubeSat-level loads. They are used here as the interim design envelope at the
CubeSat centre of gravity until the integrator supplies CubeSat-level loads. A quasi-static load **test** is not required
for a fully containerized CubeSat (UG v11 Table 6-2). The earlier ±10 g / ±6 g values (this document) and "15 g all
axes" (launch ICD) had no source and are withdrawn. **Open for the PI and the integrator:** confirm the design loads.

### 7.3 Random Vibration

Design and test level: the UG v11 maximum predicted environment (Table 4-6), all axes. Protoflight test: MPE, 1 minute
per axis (UG v11 Table 6-2; procedure `docs/test_procedures/TP-ENV-002_Vibration.md`).

| Frequency | ASD (g²/Hz) | Notes |
|-----------|-------------|-------|
| 20 Hz | 0.01 | Start |
| 50 Hz | 0.015 | |
| 700 Hz | 0.015 | |
| 800 Hz | 0.03 | |
| 925 Hz | 0.03 | |
| 2000 Hz | 0.00644 | |
| Overall | 5.57 Grms | Duration: 1 min/axis (protoflight) |

**Correction of the earlier profile.** Version 1.0 gave 20 Hz 0.01, 50 Hz 0.04, 800 Hz 0.04, 2000 Hz 0.01 g²/Hz and
called it 6.8 grms. Integrating that profile with straight lines between breakpoints on log-log axes gives segment areas
of 0.716 g² (20–50 Hz), 30.000 g² (50–800 Hz) and 23.394 g² (800–2000 Hz), a total of 54.11 g², so the overall level
was √54.11 = **7.36 Grms**, not 6.8. That is 2.4 dB above the UG v11 MPE (20 log₁₀(7.36/5.57)). The test procedure's
revision A used yet another level, 14.1 Grms. Both are withdrawn in favour of the single provider level above.

### 7.4 FEA Results Summary

**No finite-element model exists**, so there are no analysed mode frequencies. The values given in version 1.0
(150, 155 and 210 Hz) had no model behind them and are withdrawn.

| Mode | Frequency | Description |
|------|-----------|-------------|
| 1st mode | TBD (analysis pending FEM); requirement > 40 Hz | — |
| 2nd mode | TBD (analysis pending FEM) | — |
| 3rd mode | TBD (analysis pending FEM) | — |

The first mode will be measured by the low-level sine sweep before and after random vibration (UG v11 §6.7.1,
Table 6-3).

### 7.5 Margin Summary

Margin of safety will be computed as MS = allowable / (FoS × applied stress) − 1, with the factors of safety in §7.1.
**Every margin is analysis pending FEM.** The stresses and margins in version 1.0 (+2.3, +1.75, +3.5) had no model
behind them and are withdrawn.

| Location | Stress (MPa) | Allowable (MPa) | MS |
|----------|--------------|-----------------|-----|
| Rail root | TBD | TBD | Analysis pending FEM |
| Bracket joint | TBD | TBD | Analysis pending FEM |
| PCB mount | TBD | TBD | Analysis pending FEM |

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
| 1.1 | 2026-10-16 | Funding audit TB-15: one launch environment from SpaceX UG v11 (interim provider): first mode > 40 Hz, random vibration 5.57 Grms MPE, quasi-static 10 g / 17 g, factors of safety per Table 5-1. The old profile is shown to integrate to 7.36 Grms, not 6.8. Mode frequencies and margins withdrawn (analysis pending FEM). Rail edge radius ≥ 1 mm per CDS Rev 14.1 §2.2.7. Mass line (§1.1) and §4 mass properties not changed here |
