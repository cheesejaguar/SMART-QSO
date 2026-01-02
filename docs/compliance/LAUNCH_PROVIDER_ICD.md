# Launch Provider Interface Control Document
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-ICD-004
**Revision:** A
**Date:** 2026-01-02
**Reference:** Cal Poly CDS Rev 14.1, GSFC-STD-7000A, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the interface requirements between the SMART-QSO CubeSat and the launch provider, including mechanical, electrical, environmental, and operational interfaces.

## 2. Scope

This document covers:
- Mechanical interfaces (CDS compliance)
- Electrical interfaces
- Environmental requirements
- RF emissions constraints
- Operational interfaces
- Documentation requirements

---

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| CDS Rev 14.1 | CubeSat Design Specification |
| LSP-REQ-317 | NASA Launch Services Program Requirements |
| GSFC-STD-7000A | GEVS Environmental Test Standard |
| NASA-STD-5001 | Structural Design Standard |
| MIL-STD-1540 | Test Requirements |

---

## 4. Mechanical Interface

### 4.1 Form Factor Compliance

| Parameter | CDS Requirement | SMART-QSO Value | Status |
|-----------|-----------------|-----------------|--------|
| Form Factor | 1U | 1U | Compliant |
| Dimensions (X) | 100.0 ± 0.1 mm | 99.9 mm | Compliant |
| Dimensions (Y) | 100.0 ± 0.1 mm | 99.9 mm | Compliant |
| Dimensions (Z) | 113.5 ± 0.1 mm | 113.4 mm | Compliant |
| Mass | ≤ 2.0 kg | 1.075 kg | Compliant |

### 4.2 Rail Requirements

| Parameter | Requirement | SMART-QSO Value | Status |
|-----------|-------------|-----------------|--------|
| Rail width | 8.5 mm +0.1/-0.0 | 8.5 mm | Compliant |
| Rail length | ≥ 75% of Z | 85 mm (75%) | Compliant |
| Rail material | Hard anodized Al | 7075-T6 anodized | Compliant |
| Surface roughness | ≤ 1.6 μm Ra | 0.8 μm Ra | Compliant |
| Edge radius | 1.0 mm min | 1.0 mm | Compliant |

### 4.3 Protrusion Limits

| Zone | Limit | SMART-QSO | Status |
|------|-------|-----------|--------|
| Rail standoff | 6.5 mm max | 5.0 mm | Compliant |
| +X/-X faces | 6.5 mm max | 3.0 mm | Compliant |
| +Y/-Y faces | 6.5 mm max | 4.0 mm | Compliant |
| +Z face (ejection) | 6.5 mm max | 2.0 mm | Compliant |
| -Z face (spring) | Per deployer | 0 mm | Compliant |

### 4.4 Center of Gravity

| Axis | Limit | SMART-QSO | Status |
|------|-------|-----------|--------|
| X | 50 ± 20 mm | 48 mm | Compliant |
| Y | 50 ± 20 mm | 51 mm | Compliant |
| Z | 56.75 ± 20 mm | 55 mm | Compliant |

### 4.5 Structural Requirements

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| Fundamental frequency | > 100 Hz | 185 Hz | Compliant |
| Random vibration | Per GEVS | Qualified | Compliant |
| Quasi-static load | 15 g all axes | Qualified | Compliant |
| Shock | 2000 g SRS | Qualified | Compliant |

---

## 5. Electrical Interface

### 5.1 Deployment Switches

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| Switch count | ≥ 2 | 2 | Compliant |
| Switch type | Normally open | Normally open | Compliant |
| Switch location | On rails | Both rails | Compliant |
| Rating | ≥ 1A @ 30V | 3A @ 30V | Compliant |

### 5.2 Inhibits

| Inhibit | Type | Status |
|---------|------|--------|
| Inhibit 1 | Deployment switch 1 | RF and deployment |
| Inhibit 2 | Deployment switch 2 | RF and deployment |
| Inhibit 3 | RBF pin | All power |

### 5.3 RBF Pin

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| Location | Accessible | -Z face | Compliant |
| Function | Interrupt power | Battery disconnect | Compliant |
| Pull force | < 5 N | 3 N | Compliant |
| Marking | "RBF" labeled | Labeled | Compliant |

### 5.4 Separation Signal (if applicable)

| Parameter | Requirement | SMART-QSO |
|-----------|-------------|-----------|
| Signal type | N/A (passive) | N/A |
| Notes | SMART-QSO uses deployment switches only | |

---

## 6. Environmental Requirements

### 6.1 Pre-Launch Environment

| Parameter | Requirement | Compliance |
|-----------|-------------|------------|
| Storage temperature | -20°C to +50°C | Compatible |
| Storage humidity | < 75% RH | Compatible |
| Cleanliness | Class 100,000 | Compatible |

### 6.2 Launch Environment

| Parameter | Requirement | Qualification | Status |
|-----------|-------------|---------------|--------|
| Thermal (operational) | -40°C to +60°C | -45°C to +65°C | Compliant |
| Random vibration | Per LV ICD | +3 dB margin | Compliant |
| Acoustic | Per LV ICD | GEVS levels | Compliant |
| Shock | 2000 g SRS | 2500 g SRS | Compliant |
| Depressurization | 7 kPa/s | Tested | Compliant |

### 6.3 On-Orbit Environment

| Parameter | Operational Range |
|-----------|------------------|
| Thermal | -40°C to +85°C |
| Vacuum | < 10^-5 Torr |
| Radiation | See RADIATION_ANALYSIS.md |

---

## 7. RF Emissions

### 7.1 Pre-Deployment RF Silence

| Requirement | SMART-QSO Implementation |
|-------------|--------------------------|
| No RF emissions prior to deployment | Deployment switches inhibit RF |
| Timer delay after deployment | 30 min minimum |
| RF power at inhibit | 0 W |

### 7.2 Post-Deployment RF

| Parameter | Value |
|-----------|-------|
| Downlink frequency | 145.825 MHz |
| Maximum TX power | 2 W |
| Modulation | AFSK 1200 bps |
| Spurious emissions | Per FCC Part 97 |

### 7.3 RF Test Data Required

| Test | Document |
|------|----------|
| Conducted power | RF_TEST_RESULTS.md |
| Spurious emissions | RF_TEST_RESULTS.md |
| Frequency accuracy | RF_TEST_RESULTS.md |

---

## 8. Materials and Outgassing

### 8.1 Outgassing Requirements

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| TML | < 1.0% | All materials compliant | Compliant |
| CVCM | < 0.1% | All materials compliant | Compliant |
| Database | NASA MAPTIS | Verified | Compliant |

### 8.2 Prohibited Materials

| Material | Prohibited | SMART-QSO Status |
|----------|------------|------------------|
| Mercury | Yes | Not present |
| Cadmium | Yes | Not present |
| Zinc (uncoated) | Yes | Not present |
| PVC | Yes | Not present |
| Silicones (non-approved) | Yes | Only approved types |

### 8.3 Reference

See `docs/compliance/MATERIALS_LIST.md` for complete materials list.

---

## 9. Operational Interface

### 9.1 Delivery Schedule

| Milestone | Timing | Notes |
|-----------|--------|-------|
| Documentation delivery | L-60 days | All ICDs and test reports |
| Spacecraft delivery | L-14 days | Flight-ready configuration |
| RBF removal | L-2 days | Per launch site procedures |
| GO/NO-GO | L-1 day | Final readiness poll |

### 9.2 Access Requirements

| Phase | Access Required |
|-------|-----------------|
| Pre-integration | Full access for checkout |
| Post-integration | Umbilical access only |
| RBF removal | RBF pin access |
| Post-RBF | No access |

### 9.3 State at Delivery

| Parameter | Required State |
|-----------|----------------|
| Power | Inhibited (RBF in) |
| Software | Flight version loaded |
| Battery SOC | 50% ± 10% |
| Configuration | Flight settings |
| Time | Within 1 second of UTC |

---

## 10. Documentation Requirements

### 10.1 Required Deliverables

| Document | Due Date | Status |
|----------|----------|--------|
| ICD (this document) | L-90 days | Complete |
| Mass properties report | L-60 days | Complete |
| Environmental test report | L-60 days | Pending test |
| Safety data package | L-60 days | Complete |
| Flight software description | L-60 days | Complete |
| RF test report | L-60 days | Pending test |
| Final as-built drawings | L-30 days | Complete |

### 10.2 Required Certifications

| Certification | Document |
|---------------|----------|
| CDS compliance | This document, Section 4 |
| Outgassing compliance | MATERIALS_LIST.md |
| Inhibit verification | Test report |
| Frequency coordination | IARU_COORDINATION.md |
| Export control | EXPORT_CONTROL.md |

---

## 11. Deployer Compatibility

### 11.1 Compatible Deployers

| Deployer | Compatibility | Notes |
|----------|---------------|-------|
| P-POD Mk III | Compatible | Standard 1U |
| ISIPOD | Compatible | Standard 1U |
| NRCSD | Compatible | ISS deployment |
| EXOpod | Compatible | Verify fit |

### 11.2 Deployer Interface

| Parameter | P-POD Mk III | SMART-QSO |
|-----------|--------------|-----------|
| Rail gap | 8.5 mm | 8.5 mm |
| Door spring | 5 N | Compatible |
| Ejection velocity | 1.5 m/s | Compatible |

---

## 12. Waivers and Deviations

### 12.1 Requested Waivers

| Item | Requirement | Request | Status |
|------|-------------|---------|--------|
| None | - | - | - |

### 12.2 Deviations

| Item | Requirement | Deviation | Rationale |
|------|-------------|-----------|-----------|
| None | - | - | - |

---

## 13. Points of Contact

| Role | Organization | Contact |
|------|--------------|---------|
| Mission Manager | SMART-QSO | TBD |
| Systems Engineer | SMART-QSO | TBD |
| Launch Integration | TBD | TBD |
| Safety | TBD | TBD |

---

## 14. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
