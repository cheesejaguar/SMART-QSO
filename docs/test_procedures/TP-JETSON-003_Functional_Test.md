# Test Procedure: Jetson Functional Test
## TP-JETSON-003

**Document Number:** SQSO-TP-JETSON-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify Jetson module AI inference and communication functions.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| USB-UART adapter | 115200 baud |
| Power supply | 5V, 5A |
| Development PC | SSH access |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on Jetson | Boot < 60s | |
| 2 | Verify UART communication | Response received | |
| 3 | Send health request | JSON received | |
| 4 | Request inference | Processing starts | |
| 5 | Verify inference time | < 10 seconds | |
| 6 | Verify output format | Valid text | |
| 7 | Verify content filtering | Appropriate content | |
| 8 | Command shutdown | Graceful < 10s | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Boot time | < 60 seconds |
| UART communication | Reliable |
| Inference time | < 10 seconds |
| Content | Valid, appropriate |
| Shutdown | < 10 seconds |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
