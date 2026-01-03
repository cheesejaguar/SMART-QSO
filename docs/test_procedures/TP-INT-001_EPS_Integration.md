# Test Procedure: EPS Integration
## TP-INT-001

**Document Number:** SQSO-TP-INT-001
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

This procedure defines the integration test for the Electrical Power Subsystem (EPS) with the CubeSat structure and other subsystems.

---

## 2. SCOPE

### 2.1 Tests Included
- EPS mechanical integration
- Power bus verification
- Load testing with integrated subsystems
- Telemetry verification

### 2.2 Tests Excluded
- Standalone EPS functional tests (see TP-EPS-001)
- Battery characterization (see TP-EPS-003)

---

## 3. APPLICABLE DOCUMENTS

1. ICD_OBC_EPS.md - Interface Control Document
2. hardware/eps/PDU_DESIGN.md - PDU Design
3. hardware/eps/BATTERY_DESIGN.md - Battery Design
4. TP-EPS-001 - EPS Functional Test

---

## 4. TEST ARTICLE

| Item | Configuration |
|------|---------------|
| Unit Under Test | EPS Assembly |
| Structure | Flight structure or equivalent |
| Harness | Flight harness |
| Loads | Resistive loads or actual subsystems |

---

## 5. TEST EQUIPMENT

| Equipment | Specification | Cal Required |
|-----------|---------------|--------------|
| Power supply | 0-10V, 5A | Yes |
| DMM | 6.5 digit | Yes |
| Oscilloscope | 200 MHz, 4 channel | Yes |
| Electronic load | 0-4A | Yes |
| Current probe | 0-30A DC | Yes |
| Temperature logger | 0.1°C resolution | Yes |

---

## 6. PRECONDITIONS

- [ ] EPS functional test (TP-EPS-001) passed
- [ ] Structure clean and prepared
- [ ] Test equipment calibrated
- [ ] Battery charged to 50% SOC
- [ ] Integration work area ESD-compliant

---

## 7. SAFETY

| Hazard | Control |
|--------|---------|
| Battery short circuit | Fused connections, PPE |
| Pinch points | Care during assembly |
| ESD | Grounding, ESD equipment |

---

## 8. PROCEDURE

### 8.1 Mechanical Integration

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Inspect EPS board | No damage, clean | | |
| 2 | Install EPS in structure | Secure mounting | | |
| 3 | Verify fastener torque | Per spec | | |
| 4 | Connect battery harness | Secure connection | | |
| 5 | Connect solar array harness | Secure connection | | |
| 6 | Route harness per ICD | No pinch, proper radius | | |

### 8.2 Power Bus Verification

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Measure battery voltage | 7.2V ± 0.5V | | |
| 2 | Enable 3.3V rail | 3.3V ± 0.1V | | |
| 3 | Enable 5V rail | 5.0V ± 0.25V | | |
| 4 | Measure no-load current | < 50 mA | | |
| 5 | Verify all load switches OFF | No current draw | | |

### 8.3 Load Testing

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Enable 1A load on 5V | 5.0V ± 0.1V | | |
| 2 | Measure 5V ripple | < 50 mVpp | | |
| 3 | Enable 0.5A load on 3.3V | 3.3V ± 0.1V | | |
| 4 | Measure 3.3V ripple | < 20 mVpp | | |
| 5 | Enable full mission load | All rails stable | | |
| 6 | Monitor for 30 min | No anomalies | | |

### 8.4 Telemetry Verification

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Read battery voltage | Within 100 mV of DMM | | |
| 2 | Read battery current | Within 50 mA of probe | | |
| 3 | Read all rail voltages | Within spec | | |
| 4 | Read temperature | Within 2°C of logger | | |

---

## 9. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Mechanical fit | No interference, secure mount |
| Voltage accuracy | All rails ± 5% |
| Load regulation | < 3% at full load |
| Telemetry accuracy | Per Section 8.4 |
| Stability | 30 min operation, no anomalies |

---

## 10. DATA RECORDING

- [ ] All test values recorded above
- [ ] Photographs of installation
- [ ] Oscilloscope captures of ripple
- [ ] Telemetry log file

---

## 11. POST-TEST

| Step | Action |
|------|--------|
| 1 | Remove external loads |
| 2 | Verify quiescent current |
| 3 | Leave EPS installed for next integration |
| 4 | Complete test report |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release for CDR |
