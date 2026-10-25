# Launch Provider Interface Control Document
## SMART-QSO CubeSat Mission

> **Funding audit notice (2026-10-14): not an agreed ICD; statuses are design intent.**
> No launch provider, integrator or dispenser has been selected, so no interface has been agreed. Entries marked
> "Qualified", "Tested", "Complete" or given measured values (mass 1.075 kg, 185 Hz, 99.9 mm, shock 2500 g SRS, etc.)
> are **not** supported by any test or inspection (finding VV-01). The 30-minute RF timer here is superseded by the
> 45-minute rule (SYS-224), now implemented in flight software (`software/rust/sqso-fsw/src/rf_silence.rs`; hardware test pending). The provider deliverables checklist is
> `docs/regulatory/filings/LAUNCH_PROVIDER_DELIVERABLES.md`; the launch path is in
> `docs/funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md`.

**Document Number:** SQSO-ICD-010
**Revision:** D (draft)
**Date:** 2026-10-16
**Reference:** Cal Poly CDS Rev 14.1; SpaceX Rideshare Payload User's Guide v11, Aug 2026 (interim provider, pending the launch contract and PI confirmation); GSFC-STD-7000B as fallback

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
| SpaceX UG v11 | SpaceX Rideshare Payload User's Guide, Version 11, Aug 2026 (<https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf>); interim provider |
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
| Mass | ≤ 2.0 kg | Not weighed. Predicted 1.92–2.00 kg, CBE 1.69 kg (`docs/environment/MASS_BUDGET.md` v2.0, analysis); BASELINE §4 still carries CBE ≈ 1.51 kg pending the PI | Open |

### 4.2 Rail Requirements

| Parameter | Requirement | SMART-QSO Value | Status |
|-----------|-------------|-----------------|--------|
| Rail width | 8.5 mm +0.1/-0.0 | 8.5 mm | Compliant |
| Rail length | ≥ 75% of Z | 85 mm (75%) | Compliant |
| Rail material | Hard anodized Al | 7075-T6 anodized | Compliant |
| Surface roughness | ≤ 1.6 μm Ra | 0.8 μm Ra | Compliant |
| Edge radius | ≥ 1 mm (CDS Rev 14.1 §2.2.7, "should") | ≥ 1.0 mm by design (`hardware/structure/STRUCTURAL_DESIGN.md` §5.1); not yet drawn or inspected | Open |

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

One environment specification, from SpaceX UG v11 (interim provider). No analysis or test has been done.

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| Fundamental frequency | > 40 Hz, no elastic mode below (UG v11 §4.1.1) | TBD: analysis pending FEM; low-level sine sweep at test | Open |
| Random vibration | ≥ MPE 5.57 Grms, protoflight 1 min/axis, inhibits verified during the test (UG v11 Table 4-6, Table 6-2, §6.7.7) | Not tested (`docs/test_procedures/TP-ENV-002_Vibration.md`) | Open |
| Quasi-static load | 10 g axial, 17 g lateral RSS for the integrated dispenser assembly (UG v11 Table 4-1); CubeSat-level loads from the integrator. No test required for a containerized CubeSat (Table 6-2) | Analysis pending FEM | Open |
| Shock | MPE per UG v11 Table 4-5 (30 g at 100 Hz rising to 1000 g at 1000–10000 Hz, Q = 10); test advised, not required (Table 6-2) | Not tested | Open |

---

## 5. Electrical Interface

### 5.1 Deployment Switches

| Parameter | Requirement | SMART-QSO | Status |
|-----------|-------------|-----------|--------|
| Switch count | ≥ 2 | 2 | Compliant |
| Switch type | Per CDS Rev 14.1 and the dispenser ICD | Contact form and sense polarity open: TBR-CDS-005 (`docs/compliance/CDS_COMPLIANCE_MATRIX.md` §9) | Open |
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
| Notes | No electrical interface to the dispenser or launch vehicle once integrated; separation is sensed by the deployment switches only | The umbilical (charge, diagnostic and key-load) connector is used before integration only; see §9.2 |

---

## 6. Environmental Requirements

### 6.1 Pre-Launch Environment

| Parameter | Requirement | Compliance |
|-----------|-------------|------------|
| Storage temperature | -20°C to +50°C | Compatible |
| Storage humidity | < 75% RH | Compatible |
| Cleanliness | Class 100,000 | Compatible |

### 6.2 Launch Environment

Same environment as §4.5 (SpaceX UG v11, interim). Nothing has been tested.

| Parameter | Requirement | Test level | Status |
|-----------|-------------|------------|--------|
| Thermal (operational) | -40°C to +60°C (source TBD) | Thermal vacuum and cycling advised (UG v11 Table 6-2) | Open |
| Random vibration | ≥ 5.57 Grms MPE (UG v11 Table 4-6) | Protoflight: MPE, 1 min/axis | Open |
| Acoustic | — | Not required for a containerized CubeSat (UG v11 Table 6-2) | Not applicable |
| Shock | MPE per UG v11 Table 4-5 | Advised only | Open |
| Depressurization | Fairing pressure decay ≤ 2.8 kPa/s, ≤ 4.5 kPa/s for ≤ 5 s (UG v11 §4.1.8) | Venting analysis advised (UG v11 §6.7.9) | Open |

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
| Timer delay after deployment | No RF before 45 min (SYS-224; CDS Rev 14.1 §2.4.5; `software/rust/sqso-fsw/src/rf_silence.rs`) |
| RF power at inhibit | 0 W |

### 7.2 Post-Deployment RF

| Parameter | Value |
|-----------|-------|
| Downlink frequency | One IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz (SYS-200; TBD-SYS-001) |
| Maximum TX power | 1 W; 0.5 W nominal (SYS-201) |
| Modulation | AFSK 1200 bps |
| Spurious emissions | At least 40 dB below carrier (SYS-203) and ≤ 25 µW (47 CFR 97.307(e)) |

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
| RBF removal | Before dispenser integration, unless the dispenser has an access port at the RBF location | CDS Rev 14.1 §2.3.5.1; integrator to confirm |
| GO/NO-GO | L-1 day | Final readiness poll |

### 9.2 Access Requirements

| Phase | Access Required |
|-------|-----------------|
| Pre-integration | Full access for checkout, battery charging and key loading through the umbilical connector |
| Post-integration | **None**, unless the integrator's dispenser has access ports at the umbilical and RBF locations (CDS Rev 14.1 §2.3.4, §2.3.5.1). SpaceX provides no Payload connectivity after mate except optional services (Rideshare Payload User's Guide v11 Table 7-4). Plan for no access |
| RBF removal | At dispenser integration (or through an access port, if one exists) |
| Post-RBF | No access |

### 9.3 State at Delivery

| Parameter | Required State |
|-----------|----------------|
| Power | Inhibited: RBF in until dispenser integration; deployment switches held actuated once integrated |
| Software | Flight version loaded |
| Battery SOC | 50% ± 10% |
| Configuration | Flight settings |
| Time | Within 1 second of UTC |

---

## 10. Documentation Requirements

### 10.1 Required Deliverables

Due dates are placeholders until the integrator's schedule is known (SpaceX UG v11 Fig. 1-2 and Appendix G give the
SpaceX-side milestones).

| Document | Due Date | Status |
|----------|----------|--------|
| ICD (this document) | L-90 days | Draft; not agreed with any provider |
| Mass properties report | L-60 days | Not started: provisional mass list only (BASELINE §4) |
| Environmental test report | L-60 days | Pending test |
| System safety data package | Per integrator (range-safety submissions, UG v11 Appendix G) | **Skeleton only**: `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md`. The software safety data package (`docs/safety/SAFETY_DATA_PACKAGE.md`) is a draft input to it |
| Flight software description | L-60 days | No release: `docs/reviews/FLIGHT_SOFTWARE_DELIVERY.md` records status only |
| RF test report | L-60 days | Pending test |
| Final as-built drawings | L-30 days | Not started: no hardware |

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
| B | 2026-10-16 | Document number changed from SQSO-ICD-004, which is the OBC-to-ADCS ICD (`docs/interfaces/ICD_OBC_ADCS.md`), to SQSO-ICD-010, the number `INTERFACE_CONTROL_MASTER.md` reserves for the launch vehicle interface (finding PI-17). No content change | SMART-QSO Team |
| C (draft) | 2026-10-16 | Funding audit VV-21: umbilical access made consistent (no access after integration unless the dispenser has ports; RBF out at integration). VV-20: deliverable statuses corrected (no "Complete"); safety data package is a system-level skeleton. TB-15: one environment specification from SpaceX UG v11 (interim) in §4.5 and §6.2, replacing unsourced "Qualified" and "Compliant" entries; rail edge radius per CDS §2.2.7 | SMART-QSO Team |
| D (draft) | 2026-10-16 | §4.1 mass from MASS_BUDGET v2.0 (was 1.075 kg, never weighed; TB-06); §5.1 switch type open as TBR-CDS-005 (was "normally open", against "normally closed" in the structural design); §7.1 timer 45 min (SYS-224; was 30 min); §7.2 frequency, power and spurious limits from SYS-200, SYS-201 and SYS-203 (were 145.825 MHz, 2 W, "Per FCC Part 97") | SMART-QSO Team |
| D, note | 2026-10-24 | RF-silence references name the Rust file that replaced `rf_silence.c` | SMART-QSO Team |
