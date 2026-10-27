# Test Procedure: ADCS Actuator Test
## TP-ADCS-002

**Document Number:** SQSO-TP-ADCS-002
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-324, SYS-410, SYS-411, SYS-412

---

## 1. PURPOSE

Verify magnetorquer operation, polarity, and dipole moment. The baseline actuators are three CubeSpace CR0002 rods
(> 0.2 A·m²), with the 250-turn air-core coil (≥ 0.15 A·m²) as the alternate, driven by DRV8876-Q1 H-bridges with a
hardware off-timer (`docs/BASELINE_PARAMETERS.md` §8; `hardware/electrical/obc/OBC_BOARD_DESIGN.md`).

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
| 8 | Command one 900 ms burst on each axis | Coil current zero within 900 ms + 2 ms of the command, ended by the driver's own off-timer (AV-V3) | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Coil current | 100 mA ±10% |
| Polarity | Correct per drawing |
| PWM control | Linear with duty |
| Dipole moment | > 0.2 A·m² per axis with the CR0002 rods, ≥ 0.15 A·m² with the air-core alternate (BASELINE §8); SYS-411: at least 0.1 A·m² per axis |
| Burst end | Current zero within 902 ms of the command (AV-V3) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
