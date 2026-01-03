# Test Procedure: Modulation Quality
## TP-RF-004

**Document Number:** SQSO-TP-RF-004
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify AFSK modulation parameters and AX.25 frame encoding.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| SDR receiver | RTL-SDR or better |
| Modulation analyzer | FM deviation |
| AX.25 decoder | Direwolf or equivalent |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Transmit 1200 Hz tone | Deviation ±3 kHz | |
| 2 | Transmit 2200 Hz tone | Deviation ±3 kHz | |
| 3 | Transmit AX.25 packet | Decode successful | |
| 4 | Verify callsign | Correct | |
| 5 | Verify SSID | Correct | |
| 6 | Verify data content | Matches source | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| FM deviation | ±3 kHz ±0.5 kHz |
| AX.25 decode | 100% success rate |
| Callsign | Correct encoding |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
