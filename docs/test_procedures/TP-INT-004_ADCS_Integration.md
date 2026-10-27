# Test Procedure: ADCS Integration
## TP-INT-004

**Document Number:** SQSO-TP-INT-004
**Version:** 1.2
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-323, SYS-324, SYS-400, SYS-402, SYS-410, SYS-412, SYS-421

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

This procedure defines the integration test for the Attitude Determination and Control System.

---

## 2. SCOPE

- ADCS board integration
- Sensor verification (magnetometer, sun sensors)
- Actuator verification (magnetorquers)
- Control loop verification

---

## 3. APPLICABLE DOCUMENTS

1. `docs/hardware/ADCS_DESIGN.md` (SMART-QSO-ADCS-001) and `docs/interfaces/ICD_OBC_ADCS.md` §12; `hardware/adcs/SENSOR_DESIGN.md` and `ACTUATOR_DESIGN.md` are earlier background
2. `hardware/electrical/obc/OBC_BOARD_DESIGN.md`: the gyros, sun-sensor amplifiers and rod drivers are on the OBC board, the RM3100 magnetometer on board A2 at the +Z end frame
3. TP-ADCS-001 (prerequisite)
4. TP-INT-002 (prerequisite)

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Helmholtz cage | 3-axis, ±100 µT |
| Sun simulator | Collimated light |
| Magnetometer (ref) | Calibrated |
| DMM | For coil resistance |

---

## 5. PRECONDITIONS

- [ ] OBC integration complete (TP-INT-002)
- [ ] ADCS sensors calibrated
- [ ] Magnetorquers tested standalone
- [ ] Test environment magnetically quiet

---

## 6. PROCEDURE

### 6.1 ADCS Board Installation

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Install magnetometer board A2 at the +Z end frame | Secure mounting | |
| 2 | Connect harness W6E | A2 to OBC J10 | |
| 3 | Connect magnetorquer harness | All 3 axes | |
| 4 | Connect sun sensor harness | All 6 faces | |

### 6.2 Sensor Verification [SYS-323, SYS-400, SYS-402]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Read magnetometer | Valid field | |
| 2 | Compare to reference, coils off | ±5% accuracy (integration check; calibration is TP-ADCS-001, SYS-401) | |
| 2a | Read the gyro (M-G370, SPI3) on the rate table | Within 0.2°/s of the table per axis (SYS-404) | |
| 3 | Illuminate sun sensor +X | +X highest | |
| 4 | Illuminate sun sensor -X | -X highest | |
| 5 | Repeat for Y and Z | Correct response | |

### 6.3 Actuator Verification [SYS-324, SYS-410, SYS-412]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Command +X torquer +100% | Current flow | |
| 2 | Verify polarity | Correct sign | |
| 3 | Command -X torquer -100% | Reversed | |
| 4 | Repeat for Y and Z | All correct | |

### 6.4 Control Loop Test [SYS-421]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Enable B-dot mode | Control active | |
| 2 | Apply field variation | Torquer response | |
| 3 | Verify control sign | Correct direction | |
| 4 | Verify the magnetometer is sampled 50 ms after each 900 ms coil burst | Every conversion starts ≥ 50 ms after the coil current is below 1 % (AV-V2) | |
| 5 | Verify no actuation before 120 s after a simulated separation | First coil command after 120 s (SRS-F086; BASELINE §4) | |

---

## 7. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Magnetometer accuracy | ±5% of reference |
| Sun sensor response | Correct face ID |
| Magnetorquer polarity | All correct |
| Control loop | Stable response |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): magnetometer board A2 (no separate ADCS board), gyro, AV-V2 window, 120 s actuation delay; status and banner |
