# Test Procedure: ADCS Actuator Test
## TP-ADCS-002

**Document Number:** SQSO-TP-ADCS-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify magnetorquer coil operation, polarity, and dipole moment.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Gauss meter | 0-50 Gauss |
| Current meter | 0-1A, mA resolution |
| Helmholtz cage | Optional |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Command MTQ X positive | Current flows | |
| 2 | Measure X current | 100 mA ±10% | |
| 3 | Measure X field | Correct polarity | |
| 4 | Command MTQ X negative | Reversed polarity | |
| 5 | Repeat for Y coil | Pass all checks | |
| 6 | Repeat for Z coil | Pass all checks | |
| 7 | PWM test (50% duty) | Half current | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Coil current | 100 mA ±10% |
| Polarity | Correct per drawing |
| PWM control | Linear with duty |
| Dipole moment | 0.2 A·m² ±20% |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
