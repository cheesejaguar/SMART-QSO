# Test Procedure: Deployable Mechanism Test
## TP-MECH-002

**Document Number:** SQSO-TP-MECH-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify all deployable mechanisms (antenna, solar panels).

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Power supply | 0-12V, 5A |
| Timer | 0.1s resolution |
| Video camera | HD, 60 fps |

---

## 3. PROCEDURE

### 3.1 Antenna Deployment

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify stowed | Constraint intact | |
| 2 | Command deploy | Burn wire fires | |
| 3 | Time burn duration | < 5 seconds | |
| 4 | Verify full deployment | Elements extended | |

### 3.2 Solar Panel Deployment (if applicable)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify stowed | Constraint intact | |
| 2 | Command deploy | Release mechanism | |
| 3 | Verify full deployment | Panels at 90° | |
| 4 | Verify latch | Locked position | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Deployment success | 100% |
| Burn time | < 5 seconds |
| Full extension | All elements |
| No damage | Visual inspection |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
