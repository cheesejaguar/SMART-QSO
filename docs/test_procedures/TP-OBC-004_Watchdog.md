# Test Procedure: Watchdog Verification
## TP-OBC-004

**Document Number:** SQSO-TP-OBC-004
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SRS-F050, SRS-F051

---

## 1. PURPOSE

Verify internal and external watchdog timer operation and recovery.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Oscilloscope | 4-ch, 200 MHz |
| Serial terminal | 115200 baud |
| Timer | 0.1s resolution |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Boot and observe WDT kick | Regular pulse | |
| 2 | Inject software hang | WDT timeout | |
| 3 | Verify reset occurs | System resets | |
| 4 | Measure timeout | 25.6 s (25600 ms; 24.1–27.8 s, IWDG reload 3200 / 256; before the TPS3850 supervisor's 29.3 s minimum) | |
| 5 | Verify boot reason logged | WDT reset flag | |
| 6 | Test external WDT (if present) | Backup reset | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| WDT kick | Regular, < 10s period |
| Timeout | 30s ±5s |
| Reset recovery | < 30s boot |
| Boot reason | Correctly logged |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
