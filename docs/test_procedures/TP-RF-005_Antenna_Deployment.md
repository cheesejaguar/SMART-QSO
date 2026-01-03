# Test Procedure: Antenna Deployment
## TP-RF-005

**Document Number:** SQSO-TP-RF-005
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify antenna deployment mechanism and post-deployment RF performance.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Current monitor | 0-5A |
| Timer | 0.1s resolution |
| Video camera | HD, 60 fps |
| VNA | 50 MHz - 500 MHz |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify antenna stowed | Secure, constrained | |
| 2 | Command deployment | Burn wire activates | |
| 3 | Measure burn current | 0.5-1.0 A | |
| 4 | Measure burn time | < 5 seconds | |
| 5 | Verify full deployment | Elements extended | |
| 6 | Measure VSWR | < 2:1 @ 145.825 MHz | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Deployment success | 100% |
| Burn time | < 5 seconds |
| VSWR | < 2:1 |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
