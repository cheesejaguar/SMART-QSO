# SMART-QSO Mass Budget

**Document ID**: SQSO-MB-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the mass budget for the SMART-QSO 1U CubeSat, demonstrating compliance with the CubeSat Design Specification (CDS) Rev 14.1 mass limits.

## 2. Mass Requirements

### 2.1 CDS Requirements

| Parameter | Requirement | Allocation |
|-----------|-------------|------------|
| Maximum Mass | 2.00 kg | 1.80 kg (with margin) |
| CG Location | Within 2 cm of geometric center | Verified |
| Mass Properties | Measured and documented | Required |

### 2.2 Design Margins

| Phase | Margin Requirement | Status |
|-------|-------------------|--------|
| PDR | 30% | Met |
| CDR | 15% | Met |
| FRR | 5% | Expected |

## 3. Mass Budget Summary

### 3.1 Subsystem Mass Breakdown

| Subsystem | CBE (g) | Margin (%) | MEV (g) |
|-----------|---------|------------|---------|
| Structure | 280 | 5% | 294 |
| EPS | 180 | 10% | 198 |
| Battery | 200 | 3% | 206 |
| OBC | 45 | 5% | 47 |
| RF System | 120 | 10% | 132 |
| Antenna | 35 | 10% | 39 |
| AI Payload (Jetson) | 180 | 5% | 189 |
| ADCS Sensors | 25 | 10% | 28 |
| Harness | 80 | 15% | 92 |
| Thermal | 30 | 10% | 33 |
| Fasteners | 25 | 10% | 28 |
| **Total** | **1,200** | **11%** | **1,286** |

**CBE** = Current Best Estimate
**MEV** = Maximum Expected Value (CBE + Margin)

### 3.2 Mass Margin

```
MASS BUDGET SUMMARY

CDS Limit:              2,000 g
─────────────────────────────────
CBE Total:              1,200 g
Contingency (11%):        130 g
MEV Total:              1,330 g
─────────────────────────────────
Available Margin:         670 g (33%)
─────────────────────────────────
Status:                 ✓ COMPLIANT
```

## 4. Detailed Mass Breakdown

### 4.1 Structure (280g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| Primary frame (Al 6061) | 1 | 180 | 180 |
| Rails | 4 | 15 | 60 |
| Deployment switches | 4 | 5 | 20 |
| End panels | 2 | 10 | 20 |
| **Subtotal** | | | **280** |

### 4.2 Electrical Power System (180g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| EPS board | 1 | 60 | 60 |
| Solar cells | 6 | 10 | 60 |
| Solar cell coverglass | 6 | 5 | 30 |
| Power connectors | 4 | 5 | 20 |
| Protection circuits | 1 | 10 | 10 |
| **Subtotal** | | | **180** |

### 4.3 Battery (200g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| 18650 cells | 4 | 45 | 180 |
| Battery board | 1 | 15 | 15 |
| Cell holder | 1 | 5 | 5 |
| **Subtotal** | | | **200** |

### 4.4 On-Board Computer (45g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| OBC board (Apollo4) | 1 | 30 | 30 |
| SD card module | 1 | 5 | 5 |
| RTC crystal | 1 | 2 | 2 |
| Connectors | 4 | 2 | 8 |
| **Subtotal** | | | **45** |

### 4.5 RF System (120g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| Transceiver module | 1 | 50 | 50 |
| PA module | 1 | 30 | 30 |
| LNA | 1 | 10 | 10 |
| RF switches | 2 | 5 | 10 |
| Filters | 2 | 5 | 10 |
| Connectors | 4 | 2.5 | 10 |
| **Subtotal** | | | **120** |

### 4.6 Antenna System (35g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| UHF antenna element | 1 | 10 | 10 |
| VHF antenna element | 1 | 15 | 15 |
| Deployment mechanism | 1 | 8 | 8 |
| Ground plane | 1 | 2 | 2 |
| **Subtotal** | | | **35** |

### 4.7 AI Payload - Jetson (180g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| Jetson Orin Nano module | 1 | 120 | 120 |
| Carrier board | 1 | 40 | 40 |
| Thermal interface | 1 | 10 | 10 |
| Mounting hardware | 1 | 10 | 10 |
| **Subtotal** | | | **180** |

### 4.8 ADCS Sensors (25g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| Magnetometer | 1 | 5 | 5 |
| Sun sensors | 6 | 2 | 12 |
| Gyroscope | 1 | 5 | 5 |
| Mounting brackets | 1 | 3 | 3 |
| **Subtotal** | | | **25** |

### 4.9 Harness (80g CBE)

| Item | Description | Mass (g) |
|------|-------------|----------|
| Power harness | EPS to all subsystems | 25 |
| Data harness | OBC to Jetson, RF | 20 |
| Sensor harness | ADCS sensors | 10 |
| RF cables | Coax to antenna | 15 |
| Misc connectors | Various | 10 |
| **Subtotal** | | **80** |

### 4.10 Thermal Control (30g CBE)

| Item | Qty | Unit (g) | Total (g) |
|------|-----|----------|-----------|
| MLI blanket (battery) | 1 | 10 | 10 |
| Kapton heater | 1 | 5 | 5 |
| Thermal straps | 2 | 5 | 10 |
| Thermal interface material | - | 5 | 5 |
| **Subtotal** | | | **30** |

### 4.11 Fasteners (25g CBE)

| Item | Qty | Total (g) |
|------|-----|-----------|
| M3 screws | 40 | 12 |
| M2 screws | 20 | 4 |
| Standoffs | 16 | 6 |
| Nuts/washers | misc | 3 |
| **Subtotal** | | **25** |

## 5. Center of Gravity

### 5.1 CG Calculation

| Axis | Location (mm) | Tolerance (mm) | CDS Limit (mm) |
|------|---------------|----------------|----------------|
| X | 50.0 | ±5 | 50 ± 20 |
| Y | 50.0 | ±5 | 50 ± 20 |
| Z | 55.0 | ±10 | 50 ± 20 |

**Status**: ✓ Within CDS limits

### 5.2 CG Verification

```
1U CubeSat CG Location (100mm cube)

        100mm
       ┌──────┐
       │      │
       │  CG  │ ← Z = 55mm
       │  ●   │
       │      │
       └──────┘
         ↑
      X,Y = 50mm (centered)
```

## 6. Moments of Inertia

### 6.1 Principal Moments

| Axis | Ixx (kg·m²) | Iyy (kg·m²) | Izz (kg·m²) |
|------|-------------|-------------|-------------|
| Value | 0.0020 | 0.0020 | 0.0018 |
| Tolerance | ±10% | ±10% | ±10% |

### 6.2 Products of Inertia

| Product | Value (kg·m²) |
|---------|---------------|
| Ixy | < 0.0001 |
| Ixz | < 0.0001 |
| Iyz | < 0.0001 |

## 7. Mass Growth Tracking

### 7.1 Historical Mass Growth

| Phase | Date | CBE (g) | Change |
|-------|------|---------|--------|
| Concept | 2025-Q1 | 900 | - |
| PDR | 2025-Q3 | 1,050 | +150g |
| CDR | 2026-Q1 | 1,200 | +150g |
| FRR | TBD | TBD | - |

### 7.2 Mass Contingency Allocation

| Category | Contingency |
|----------|-------------|
| Known items | 5% |
| Estimated items | 10% |
| Unknown items | 20% |
| System level | 5% |

## 8. Mass Verification

### 8.1 Verification Method

| Phase | Method |
|-------|--------|
| Design | Analysis (spreadsheet) |
| Build | Measurement (scale) |
| Integration | Measurement after each step |
| Final | Precision measurement + CG measurement |

### 8.2 Required Measurements

| Measurement | Equipment | Accuracy |
|-------------|-----------|----------|
| Total mass | Digital scale | ±1g |
| CG location | CG fixture | ±2mm |
| MOI | Bifilar pendulum | ±5% |

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR release |
