# Test Procedure: Jetson Functional Test
## TP-JETSON-003

**Document Number:** SQSO-TP-JETSON-003
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-510, SYS-512, SYS-513, SYS-514

---

## 1. PURPOSE

Verify Jetson module AI inference and communication functions. Jetson-generated beacon text is the gated experiment
of decision D5; the flight-baseline beacon text comes from the OBC selector's pre-approved templates
(`docs/BASELINE_PARAMETERS.md` §6). The go/no-go energy measurement is VI-02 (≥ 30 EM bursts).

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| USB-UART adapter | 115200 baud |
| Power supply | Carrier input from VBAT, 11.2–16.4 V (`docs/hardware/EPS_DESIGN.md` §4), ≥ 1.5 A |
| Development PC | SSH access |

---

## 3. PROCEDURE [SYS-510, SYS-512, SYS-513, SYS-514]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on Jetson | First STATUS line within 45 s (V-JET-3) | |
| 2 | Verify UART communication | Response received | |
| 3 | Send health request | JSON received | |
| 4 | Request inference | Processing starts | |
| 5 | Verify the burst | One power-on ≤ 10 min including boot and ≤ 3.6 kJ at the carrier input, ending early on the energy gate; a batch of ≥ 10 lines (SYS-512, decision C-20) | |
| 6 | Verify output format | ASCII only, within the length limits (SYS-513) | |
| 7 | Verify content filtering | Every line passes the content filter of `docs/BEACON_SPEC.md` §6 on the payload and the OBC: no URLs, commercial words, profanity, encoded-looking words, other call signs or numbers (SYS-513); the shared test vectors give the same verdict on both | |
| 8 | Command shutdown | Graceful < 10s | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Boot time | First STATUS line within 45 s (V-JET-3) |
| UART communication | Reliable |
| Burst | ≤ 10 min and ≤ 3.6 kJ per power-on (SYS-512) |
| Content | Passes the BEACON_SPEC §6 filter on both processors (SYS-513) |
| Energy per accepted line | Recorded; the go/no-go is ≤ 0.1 Wh (360 J) over ≥ 30 EM bursts (VI-02) |
| Shutdown | < 10 seconds |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
