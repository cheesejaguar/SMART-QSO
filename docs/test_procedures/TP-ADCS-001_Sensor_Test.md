# Test Procedure: ADCS Sensor Test
## TP-ADCS-001

**Document Number:** TP-ADCS-001
**Revision:** A
**Date:** 2026-01-02

---

## 1. Purpose

Verify calibration and accuracy of ADCS sensors (magnetometer and sun sensors).

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Helmholtz cage | 3-axis, ±100 µT |
| Gauss meter | Reference magnetometer |
| Sun simulator | Collimated light source |
| Rotary table | 0.1° resolution |
| Data acquisition | 1 kHz sample rate |

## 3. Procedure

### 3.1 Magnetometer Calibration

| Step | Field (µT) | Axis | Expected | Measured | Error | P/F |
|------|------------|------|----------|----------|-------|-----|
| 1 | 0 | All | 0 ± 1 | ______ | <1 µT | |
| 2 | +50 | X | +50 ± 1 | ______ | <2% | |
| 3 | -50 | X | -50 ± 1 | ______ | <2% | |
| 4 | +50 | Y | +50 ± 1 | ______ | <2% | |
| 5 | -50 | Y | -50 ± 1 | ______ | <2% | |
| 6 | +50 | Z | +50 ± 1 | ______ | <2% | |
| 7 | -50 | Z | -50 ± 1 | ______ | <2% | |

### 3.2 Sun Sensor Test

| Step | Face | Illumination | Expected | Measured | P/F |
|------|------|--------------|----------|----------|-----|
| 1 | +X | Normal | Max output | ______ | |
| 2 | +X | 45° | 70% output | ______ | |
| 3 | -X | Normal | Max output | ______ | |
| 4 | +Y | Normal | Max output | ______ | |
| 5 | -Y | Normal | Max output | ______ | |
| 6 | +Z | Normal | Max output | ______ | |
| 7 | -Z | Normal | Max output | ______ | |

## 4. Pass/Fail Criteria

- Magnetometer accuracy: < 2% full scale
- Sun sensor detection: All faces functional
- Cross-axis coupling: < 2%

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
