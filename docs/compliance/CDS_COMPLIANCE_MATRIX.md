# CubeSat Design Specification Compliance Matrix
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-CDS-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** Cal Poly CubeSat Design Specification Rev 14.1

---

## 1. Purpose

This document provides a compliance matrix against the Cal Poly CubeSat Design Specification (CDS) Rev 14.1 for the SMART-QSO 1U CubeSat.

## 2. General Requirements

### 2.1 Dimensions

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Width (X) | 100.0 mm +0.1/-0.0 | 100.0 mm | Compliant |
| Depth (Y) | 100.0 mm +0.1/-0.0 | 100.0 mm | Compliant |
| Height (Z) | 113.5 mm +0.1/-0.0 | 113.5 mm | Compliant |
| Rail width | 8.5 mm minimum | 8.5 mm | Compliant |
| Rail contact length | Minimum 75% of Z | 85.1 mm (75%) | Compliant |
| Corner radii | 1.0 mm maximum | 0.5 mm | Compliant |

### 2.2 Mass

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Maximum mass (1U) | 2.0 kg | 1.33 kg (est.) | Compliant |
| Mass margin | 10% recommended | 34% margin | Compliant |

### 2.3 Center of Gravity

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| CG location (X) | Within 2 cm of geometric center | 0.5 cm offset | Compliant |
| CG location (Y) | Within 2 cm of geometric center | 0.5 cm offset | Compliant |
| CG location (Z) | Within 2 cm of geometric center | 1.0 cm offset | Compliant |

## 3. Mechanical Requirements

### 3.1 Rails

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Rail material | Hard anodized Al 7075 or 6061-T6 | Al 7075-T6 | Compliant |
| Surface finish (Ra) | 1.6 um maximum | 1.2 um | Compliant |
| Anodize thickness | 10 um minimum | 15 um | Compliant |
| Edges | No sharp edges | Deburred | Compliant |

### 3.2 Protrusions

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Rail protrusions | None beyond rail surface | None | Compliant |
| Side protrusions | 6.5 mm max from surface | 5.0 mm (stowed) | Compliant |
| Antenna (stowed) | Within envelope | Tape antenna stowed | Compliant |
| Solar panels (stowed) | Within envelope | Body-mounted | Compliant |

### 3.3 Separation Springs

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Spring plungers | 4 corners required | 4 corners | Compliant |
| Spring force | Per deployer spec | TBD | TBR |
| Plunger material | Non-marring | Delrin | Compliant |

## 4. Electrical Requirements

### 4.1 Deployment Switches

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Switch quantity | Minimum 2 | 2 switches | Compliant |
| Switch type | Mechanical, normally open | Mechanical NO | Compliant |
| Switch location | -Z rails | -Z rail pair | Compliant |
| Circuit break | All power circuits | All circuits disabled | Compliant |

### 4.2 RBF (Remove Before Flight) Pin

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| RBF pin | Required | Installed | Compliant |
| RBF function | Additional inhibit | Inhibits deployment | Compliant |
| RBF access | Accessible after integration | Side access | Compliant |
| RBF marking | "REMOVE BEFORE FLIGHT" | Tagged | Compliant |

### 4.3 Power Inhibits

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Deployment inhibits | Minimum 2 | 3 (RBF + 2 switches) | Compliant |
| RF inhibit | Until 30 min after deploy | Timer-based | Compliant |
| Inhibit independence | Independent mechanisms | Independent | Compliant |

## 5. Operational Requirements

### 5.1 Post-Deployment Timing

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| RF silence | 30 minutes minimum | 30 minutes (configurable) | Compliant |
| Deployment timing | After RF silence | 45 minutes (configurable) | Compliant |
| Timer implementation | Hardware or software | Software with HW backup | Compliant |

### 5.2 RF Emissions

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Pre-deployment emissions | None | Inhibited | Compliant |
| Post-deployment power | Per license | 0.5W nominal, 2W max | Compliant |
| Frequency coordination | IARU required | In progress | TBR |

## 6. Environmental Requirements

### 6.1 Vibration

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Random vibration | Per launch provider | Protoflight planned | TBR |
| Sine vibration | Per launch provider | Protoflight planned | TBR |
| Workmanship screening | Recommended | Planned | TBR |

### 6.2 Thermal Vacuum

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| Temperature range | Per mission | -20C to +50C operational | TBR |
| Thermal cycles | Minimum 4 protoflight | 4 cycles planned | TBR |
| Vacuum level | < 10^-5 Torr | TBD | TBR |

### 6.3 Outgassing

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| TML | < 1.0% | All materials qualified | Compliant |
| CVCM | < 0.1% | All materials qualified | Compliant |
| ASTM E595 | Compliance required | Materials list maintained | Compliant |

## 7. Documentation Requirements

### 7.1 Deliverables to Launch Provider

| Document | Status | Notes |
|----------|--------|-------|
| Compliance matrix | This document | In progress |
| Mass properties report | TBD | Post-fabrication |
| ICD verification | TBD | Post-integration |
| Test reports | TBD | Post-testing |
| Safety data package | TBD | Pre-ship |

### 7.2 Configuration Control

| Requirement | CDS Spec | SMART-QSO Design | Status |
|-------------|----------|------------------|--------|
| As-built documentation | Required | CM Plan in place | Compliant |
| Change control | Required | CCB process defined | Compliant |
| Final configuration | Frozen at ship | Process defined | Compliant |

## 8. Compliance Summary

| Category | Total Items | Compliant | TBR | Non-Compliant |
|----------|-------------|-----------|-----|---------------|
| Dimensions | 6 | 6 | 0 | 0 |
| Mass | 2 | 2 | 0 | 0 |
| Center of Gravity | 3 | 3 | 0 | 0 |
| Rails | 4 | 4 | 0 | 0 |
| Protrusions | 4 | 4 | 0 | 0 |
| Separation Springs | 3 | 2 | 1 | 0 |
| Deployment Switches | 4 | 4 | 0 | 0 |
| RBF Pin | 4 | 4 | 0 | 0 |
| Power Inhibits | 3 | 3 | 0 | 0 |
| Post-Deployment | 3 | 3 | 0 | 0 |
| RF Emissions | 3 | 2 | 1 | 0 |
| Vibration | 3 | 0 | 3 | 0 |
| Thermal Vacuum | 3 | 0 | 3 | 0 |
| Outgassing | 3 | 3 | 0 | 0 |
| **Total** | **48** | **40** | **8** | **0** |

**Compliance Rate:** 83% verified, 17% pending (TBR)

## 9. TBR/TBD Closure Plan

| Item | Description | Closure Activity | Target Date |
|------|-------------|------------------|-------------|
| TBR-001 | Separation spring force | Measure during integration | TBD |
| TBR-002 | IARU frequency coordination | Complete coordination | TBD |
| TBR-003 | Vibration test | Complete protoflight test | TBD |
| TBR-004 | Thermal vacuum test | Complete protoflight test | TBD |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
