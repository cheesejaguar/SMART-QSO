# Test Procedure: OBC Timing Verification
## TP-OBC-003

**Document Number:** SQSO-TP-OBC-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

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

## 3. PROCEDURE

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
