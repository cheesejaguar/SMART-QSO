# Test Procedure: Watchdog Verification
## TP-OBC-004

**Document Number:** SQSO-TP-OBC-004
**Version:** 1.1
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SRS-F050, SRS-F051

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

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
| 6 | Test the external supervisor (TPS3850) with the IWDG kick withheld and the IWDG reset suppressed on the bench | Backup reset by the supervisor, not before its 29.3 s minimum (SQSO-ICX-001 D §4.3) | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| WDT kick | Regular, < 10s period |
| Timeout | 24.1–27.8 s (IWDG, 25.6 s nominal; `software/flight/include/watchdog_mgr.h` `WDT_HW_TIMEOUT_MS`), before the supervisor's 29.3 s minimum (was 30 s ± 5 s, which disagreed with step 4) |
| Reset recovery | Flight software running within 30 s (SYS-300) |
| Boot reason | Correctly logged |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-27 | Documentation audit (post-CDR 2026): pass criterion 24.1–27.8 s (was 30 s ± 5 s, against step 4 and WDT_HW_TIMEOUT_MS); external check is the TPS3850 supervisor (≥ 29.3 s); status and banner |
