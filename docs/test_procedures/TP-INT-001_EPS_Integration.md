# Test Procedure: EPS Integration
## TP-INT-001

**Document Number:** SQSO-TP-INT-001
**Version:** 1.2
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-120, SYS-121, SYS-122, SYS-124, SYS-320

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

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

1. `docs/interfaces/ICD_OBC_EPS.md` (SMART-QSO-ICD-001) - Interface Control Document
2. `docs/hardware/EPS_DESIGN.md` (SMART-QSO-EPS-001) and `hardware/electrical/eps/EPS_BOARD_DESIGN.md` (SQSO-EPS-BRD-001) - EPS design; `hardware/eps/PDU_DESIGN.md` is earlier background
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
| Power supply | 0-20 V, 5 A (4S pack simulator) | Yes |
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

### 8.2 Power Bus Verification [SYS-120, SYS-121, SYS-122]

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Measure battery voltage | Within 11.2–16.4 V (4S pack, 14.4 V nominal, charged to 16.4 V; `docs/BASELINE_PARAMETERS.md` §5); record | | |
| 2 | Enable 3V3_AO | 3.3 V ± 2 % (`docs/hardware/EPS_DESIGN.md` §4) | | |
| 3 | Enable 5V_PA | 5.0 V ± 2 % (EPS_DESIGN §4) | | |
| 4 | Measure no-load current | < 50 mA | | |
| 5 | Verify all load switches OFF | No current draw | | |

### 8.3 Load Testing [SYS-120, SYS-121]

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Enable 0.55 A load on 5V_PA (the PA at 1 W RF, EPS_DESIGN §4) | 5.0 V ± 2 % | | |
| 2 | Measure 5V ripple | < 50 mVpp | | |
| 3 | Enable 0.26 A load on 3V3_AO (its maximum steady load, EPS_DESIGN §4) | 3.3 V ± 2 % | | |
| 4 | Measure 3.3V ripple | < 20 mVpp | | |
| 5 | Enable full mission load | All rails stable; bus ripple ≤ 200 mV p-p, 1 MHz bandwidth (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` §3.6) | | |
| 6 | Monitor for 30 min | No anomalies | | |

### 8.4 Telemetry Verification [SYS-124, SYS-320]

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
| Voltage accuracy | 3V3_AO and 5V_PA ± 2 % (EPS_DESIGN §4) |
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
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): 4S bus 11.2–16.4 V (was 7.2 V), rails 3V3_AO and 5V_PA ± 2 % from EPS_DESIGN; status and banner |
