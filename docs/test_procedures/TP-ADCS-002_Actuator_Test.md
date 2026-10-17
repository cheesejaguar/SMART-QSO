# Test Procedure: ADCS Actuator Test
## TP-ADCS-002

**Document Number:** SQSO-TP-ADCS-002
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-324, SYS-410, SYS-411, SYS-412

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

## 3. PROCEDURE [SYS-324, SYS-410, SYS-412]

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
| Dipole moment | 0.2 A·m² ±20% (SYS-411: at least 0.1 A·m² per axis) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
