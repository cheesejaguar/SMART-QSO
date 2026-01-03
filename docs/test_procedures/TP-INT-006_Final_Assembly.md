# Test Procedure: Final Assembly Integration
## TP-INT-006

**Document Number:** SQSO-TP-INT-006
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

This procedure defines the final mechanical assembly and integration verification for the SMART-QSO CubeSat, ensuring all subsystems are properly installed and the spacecraft meets CubeSat Design Specification requirements.

---

## 2. SCOPE

### 2.1 Tests Included
- Final mechanical assembly
- Harness installation verification
- Antenna stowage verification
- Solar panel installation (if deployable)
- Mass properties measurement
- Fit check with deployer mockup

### 2.2 Prerequisites
All preceding integration tests must be complete:
- TP-INT-001 (EPS Integration) PASS
- TP-INT-002 (OBC Integration) PASS
- TP-INT-003 (RF Integration) PASS
- TP-INT-004 (ADCS Integration) PASS
- TP-INT-005 (Jetson Integration) PASS

---

## 3. APPLICABLE DOCUMENTS

1. CubeSat Design Specification (CDS) Rev 14.1
2. Launch Provider ICD
3. SMART-QSO Mechanical Drawings
4. SMART-QSO Harness Drawings
5. docs/compliance/CDS_COMPLIANCE_MATRIX.md

---

## 4. TEST ARTICLE

| Item | Configuration |
|------|---------------|
| Unit Under Test | SMART-QSO Flight Unit |
| Configuration | Full flight configuration |
| Software | Flight software loaded |
| Battery | Charged to 50% SOC |

---

## 5. TEST EQUIPMENT

| Equipment | Specification | Cal Required |
|-----------|---------------|--------------|
| Precision scale | 0.1g resolution, 5kg range | Yes |
| CG measurement fixture | Custom or commercial | Yes |
| MOI measurement fixture | Torsional pendulum or equivalent | Optional |
| Deployer mockup | Representative of flight deployer | N/A |
| Torque wrench | 0.1-5 N·m range | Yes |
| Inspection microscope | 10x minimum | No |
| Calipers | 0.01mm resolution | Yes |
| Height gauge | 0.01mm resolution | Yes |
| Go/No-Go gauges | CDS rail dimensions | No |

---

## 6. PRECONDITIONS

- [ ] All subsystem integration tests passed
- [ ] Final software loaded and verified
- [ ] All fasteners available and inspected
- [ ] Workbench clean and ESD-protected
- [ ] Inspection forms ready
- [ ] Camera for documentation

---

## 7. SAFETY

| Hazard | Control |
|--------|---------|
| Battery | Fire extinguisher nearby |
| Pinch points | Careful handling |
| Deployment mechanisms | Safed with RBF pin |
| Dropping | Two-person handling |

---

## 8. PROCEDURE

### 8.1 Pre-Assembly Inspection

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Inspect all fasteners | No damage, correct type | | |
| 2 | Inspect all boards | No damage, clean | | |
| 3 | Inspect structure | No damage, clean | | |
| 4 | Verify RBF pin present | Installed | | |
| 5 | Photograph current state | Documentation | | |

### 8.2 Final Harness Installation

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Route remaining harnesses | Per drawing | | |
| 2 | Verify connector mating | All fully seated | | |
| 3 | Secure with tie-downs | No loose wires | | |
| 4 | Verify strain relief | At all connectors | | |
| 5 | Check clearances | Min 1mm to structure | | |

### 8.3 Antenna Stowage

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Coil/fold antenna elements | Per procedure | | |
| 2 | Secure with restraint | Nylon line/mechanism | | |
| 3 | Install burn wire | Proper routing | | |
| 4 | Verify stowed dimensions | Within envelope | | |
| 5 | Verify deployment path clear | No obstructions | | |

### 8.4 Solar Panel Installation (If Deployable)

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Install hinge hardware | Per torque spec | | |
| 2 | Install panel | Proper alignment | | |
| 3 | Install restraint mechanism | Properly tensioned | | |
| 4 | Verify stowed dimensions | Within envelope | | |
| 5 | Connect solar array harness | Proper routing | | |

### 8.5 Final Closeout

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Install access panels | All fasteners torqued | | |
| 2 | Install end plates | Per spec | | |
| 3 | Verify kill switch access | Accessible from exterior | | |
| 4 | Verify RBF pin access | Accessible | | |
| 5 | Apply thread locker | Per drawing notes | | |
| 6 | Final torque all fasteners | Per spec, witness marks | | |

### 8.6 Mass Properties Measurement

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Measure total mass | < 1.33 kg (1U limit) | | |
| 2 | Measure CG X position | Within 2cm of center | | |
| 3 | Measure CG Y position | Within 2cm of center | | |
| 4 | Measure CG Z position | Within 2cm of center | | |
| 5 | Record results | Document | | |

**Mass Properties Table:**

| Parameter | Requirement | Measured | Margin |
|-----------|-------------|----------|--------|
| Total Mass | < 1.33 kg | _____ kg | _____ |
| CG offset X | < ±2 cm | _____ cm | _____ |
| CG offset Y | < ±2 cm | _____ cm | _____ |
| CG offset Z | < ±2 cm | _____ cm | _____ |

### 8.7 Dimensional Verification

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Measure overall X dimension | 100mm ± 0.1mm | | |
| 2 | Measure overall Y dimension | 100mm ± 0.1mm | | |
| 3 | Measure overall Z dimension | 113.5mm ± 0.5mm | | |
| 4 | Verify rail straightness | < 0.1mm bow | | |
| 5 | Verify rail surface finish | < 1.6 µm Ra | | |
| 6 | Check for protrusions | None beyond envelope | | |

### 8.8 Deployer Fit Check

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Obtain deployer mockup | Representative dimensions | | |
| 2 | Insert CubeSat into mockup | Smooth insertion | | |
| 3 | Verify rail engagement | Full contact | | |
| 4 | Verify deployment switch actuation | Proper depression | | |
| 5 | Verify extraction | Smooth removal | | |
| 6 | Repeat insertion 3 times | Consistent results | | |

---

## 9. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Mass | < 1.33 kg |
| CG location | Within ±2 cm of geometric center |
| Dimensions | Per CDS Rev 14.1 |
| Rail finish | < 1.6 µm Ra |
| Protrusions | None beyond CDS envelope |
| Deployer fit | Smooth insertion/extraction |
| All fasteners | Torqued to spec, witness marked |
| Harness routing | Per drawing, no interference |

---

## 10. DATA RECORDING

### 10.1 Required Documentation

- [ ] Completed procedure with all values
- [ ] Mass properties data sheet
- [ ] Dimensional inspection report
- [ ] Photographs (minimum 10 views)
- [ ] Deployer fit check video
- [ ] Witness/inspector signatures

### 10.2 Configuration Record

| Item | Value |
|------|-------|
| Serial Number | ____________ |
| Assembly Date | ____________ |
| Flight Software Version | ____________ |
| Configuration Baseline | ____________ |
| Test Engineer | ____________ |
| QA Inspector | ____________ |

---

## 11. POST-TEST

| Step | Action |
|------|--------|
| 1 | Power functional check |
| 2 | RF functional check (into load) |
| 3 | Secure in protective storage |
| 4 | Complete final assembly report |
| 5 | Update configuration log |
| 6 | Schedule environmental testing |

---

## 12. NON-CONFORMANCE HANDLING

Any non-conformance shall be:
1. Documented on NCR form
2. Evaluated for rework or waiver
3. Approved by Project Engineer
4. Recorded in project database

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release for CDR |
