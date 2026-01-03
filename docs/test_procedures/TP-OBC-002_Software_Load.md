# Test Procedure: OBC Software Load
## TP-OBC-002

**Document Number:** SQSO-TP-OBC-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify OBC software loading, version verification, and configuration.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| ST-Link debugger | V3 |
| Development PC | STM32CubeIDE |
| Serial terminal | 115200 baud |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect debugger | Target detected | |
| 2 | Erase flash | Erase complete | |
| 3 | Load flight software | Program success | |
| 4 | Reset and boot | Boot banner appears | |
| 5 | Query version | Matches SVD | |
| 6 | Verify CRC | Matches build | |
| 7 | Load configuration | Config accepted | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Flash program | Success |
| Version match | SVD baseline |
| CRC match | Build checksum |
| Boot time | < 30 seconds |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
