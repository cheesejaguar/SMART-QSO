# Test Procedure: OBC Timing Verification
## TP-OBC-003

**Document Number:** SQSO-TP-OBC-003
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-301 (steps 1–3), SYS-211 (step 4, the 60 s ACTIVE beacon period)

---

## 1. PURPOSE

Verify OBC real-time clock accuracy and task timing.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| GPS time reference | 1 PPS output |
| Oscilloscope | 4-ch, 200 MHz |
| Counter | 0.1 ppm accuracy |

---

## 3. PROCEDURE [SYS-211, SYS-301]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure 32.768 kHz crystal | ±20 ppm | |
| 2 | Measure 1 second tick | ±100 ppm | |
| 3 | Run for 24 hours | Drift < 10 seconds | |
| 4 | Measure task period (beacon) | 60s ±0.1s | |
| 5 | Measure task jitter | < 1 ms | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Clock accuracy | ±100 ppm |
| 24-hour drift | < 10 seconds |
| Task jitter | < 1 ms |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
