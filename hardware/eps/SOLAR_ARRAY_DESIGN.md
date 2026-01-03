# Solar Array Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-SA-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. Overview

This document defines the solar array design for the SMART-QSO 1U CubeSat mission, following NASA Ames SmallSat heritage practices.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| EOL Power Generation | > 2.0 W avg | SYS-PWR-001 |
| Voltage Range | 6-8.4V (2S Li-ion) | EPS Design |
| Operating Temperature | -40°C to +85°C | GEVS |
| Deployment | Body-mounted + deployable | Mission |
| Design Life | 1 year on-orbit | Mission |

---

## 2. Solar Cell Selection

### 2.1 Cell Specification

| Parameter | Value |
|-----------|-------|
| Manufacturer | Spectrolab / Azur Space |
| Cell Type | UTJ (Ultra Triple Junction) GaAs |
| Efficiency | 29.5% (BOL, AM0) |
| Dimensions | 40mm x 80mm (standard CubeSat cell) |
| Vmp | 2.41V |
| Imp | 0.504A |
| Voc | 2.69V |
| Isc | 0.520A |
| Radiation Tolerance | > 1E15 1-MeV e/cm² |

### 2.2 Procurement Specification

- Qualified per MIL-PRF-38534 Class K or equivalent
- Lot acceptance testing required
- ESD-sensitive handling per ANSI/ESD S20.20
- Inspection per IPC-A-610 Class 3 Space Addendum

---

## 3. String Configuration

### 3.1 Body-Mounted Arrays (4 faces)

Each body face (4 deployable-facing):
- 2 cells per string (series)
- 1 string per face
- Vmp: 4.82V per string
- Imp: 0.504A per string

### 3.2 Deployable Arrays (2 wings)

Each deployable wing:
- 2 cells per string (series)
- 2 strings per wing (parallel)
- Vmp: 4.82V per wing
- Imp: 1.008A per wing

### 3.3 Total Array Summary

| Configuration | Cells | Vmp | Imp | Pmp (BOL) |
|---------------|-------|-----|-----|-----------|
| Body (4 faces) | 8 | 4.82V | 2.02A | 9.7W |
| Deployable (2 wings) | 8 | 4.82V | 2.02A | 9.7W |
| **Total** | **16** | 4.82V | 4.04A | **19.4W** |

Note: Actual power depends on illumination and angle.

---

## 4. Interconnect Design

### 4.1 Cell Interconnects

| Parameter | Specification |
|-----------|---------------|
| Material | Kovar or Invar stress-relief tabs |
| Method | Welded (preferred) or high-rel solder |
| Redundancy | Dual-tab interconnect |
| Workmanship | Per NASA-STD-8739.3 |

### 4.2 String Wiring

- 26 AWG radiation-resistant wire (PTFE/Kapton)
- Strain relief at all connection points
- Harness routing per ICD

---

## 5. Coverglass Design

### 5.1 Coverglass Specification

| Parameter | Value |
|-----------|-------|
| Material | Ceria-doped borosilicate (CMG) |
| Thickness | 100-150 µm |
| AR Coating | MgF₂ dual-layer |
| UV Cut-off | 350nm |
| Bonding | DC93-500 or equivalent |

### 5.2 Protection Provided

- Proton/electron shielding
- UV degradation protection
- Micrometeoroid protection (limited)
- Electrostatic discharge path

---

## 6. Bypass Diodes

### 6.1 Configuration

- One bypass diode per 2-cell string
- Schottky type for low forward drop
- Reverse voltage rating: > 10V
- Current rating: > 1A

### 6.2 Placement

- Mounted on solar panel PCB substrate
- Protected under coverglass or conformal coating
- Thermal path to substrate

---

## 7. Thermal Design

### 7.1 Operating Range

| Condition | Temperature | Duration |
|-----------|-------------|----------|
| Sunlit peak | +85°C max | Continuous |
| Eclipse | -40°C min | 35 min typical |
| Thermal cycling | 5500 cycles/yr | 1 year |

### 7.2 Thermal Expansion

- CTE matching between cell, coverglass, and substrate
- Stress-relief interconnects accommodate expansion
- Substrate: FR4 or aluminum-core PCB

---

## 8. Deployment Mechanism Interface

### 8.1 Hinge Design

| Parameter | Specification |
|-----------|---------------|
| Type | Spring-loaded hinge |
| Material | Stainless steel / aluminum |
| Deployment Angle | 90° ± 2° |
| Deployment Time | < 5 seconds |
| Actuation | Burn wire / thermal knife |

### 8.2 Stowed Configuration

- Panels secured with nylon restraint
- Burn wire redundancy (2 circuits)
- Deployment switches for status telemetry

---

## 9. Shadow Analysis

### 9.1 Deployed Configuration

- Antenna deployment verified to not shadow arrays
- Body-mounted cells may be partially shadowed by deployables
- MPPT handles partial shading conditions

### 9.2 Power Reduction

| Shadow Condition | Power Impact |
|------------------|--------------|
| No shadow | 100% |
| Antenna shadow | 95% |
| Partial eclipse | Linear reduction |

---

## 10. Testing Requirements

### 10.1 Cell-Level Testing

- Flash test (I-V curve)
- Visual inspection
- ESD sensitivity verification

### 10.2 Panel-Level Testing

| Test | Requirement |
|------|-------------|
| I-V Characterization | Per AIAA S-111 |
| Thermal Cycling | -40°C to +85°C, 10 cycles |
| Deployment Test | 10 cycles minimum |
| Illumination Test | AM0 equivalent |

---

## 11. Traceability

| Requirement | Design Element | Verification |
|-------------|----------------|--------------|
| SYS-PWR-001 | 16 cells, 19.4W BOL | Analysis, Test |
| SYS-PWR-002 | Deployable panels | Inspection |
| ENV-THERMAL-001 | Qualified cells | Test |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
