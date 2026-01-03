# Test Procedure: ADCS Integration
## TP-INT-004

**Document Number:** SQSO-TP-INT-004
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

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

1. hardware/adcs/SENSOR_DESIGN.md
2. hardware/adcs/ACTUATOR_DESIGN.md
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
| 1 | Install ADCS board | Secure mounting | |
| 2 | Connect I2C harness | To OBC | |
| 3 | Connect magnetorquer harness | All 3 axes | |
| 4 | Connect sun sensor harness | All 6 faces | |

### 6.2 Sensor Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Read magnetometer | Valid field | |
| 2 | Compare to reference | ±5% accuracy | |
| 3 | Illuminate sun sensor +X | +X highest | |
| 4 | Illuminate sun sensor -X | -X highest | |
| 5 | Repeat for Y and Z | Correct response | |

### 6.3 Actuator Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Command +X torquer +100% | Current flow | |
| 2 | Verify polarity | Correct sign | |
| 3 | Command -X torquer -100% | Reversed | |
| 4 | Repeat for Y and Z | All correct | |

### 6.4 Control Loop Test

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Enable B-dot mode | Control active | |
| 2 | Apply field variation | Torquer response | |
| 3 | Verify control sign | Correct direction | |

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
