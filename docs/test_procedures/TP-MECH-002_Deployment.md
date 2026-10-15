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

### 3.3 Deployment timer and burn safety across resets (SYS-221, SW-10)

Use burn-wire dummy loads (or flight burn wires with spare restraints), the hardware inhibits removed for this test only, and a scope on DEPLOY_EN, DEPLOY_1 and DEPLOY_2.

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Send `arm-flight` through the umbilical; power off | Reply `SQGR 00` | |
| 2 | Power on with the switches released; start a timer | No burn line high | |
| 3 | At 20 min, force a watchdog reset | Count resumes (boot log); no burn before 30:00 from step 2 | |
| 4 | Repeat from step 1; at 25 min remove and restore power (switch toggle) | Count restarts; no burn before 30:00 from the power restore | |
| 5 | At 30 min | DEPLOY_EN and DEPLOY_1 high for 3 s; DEPLOY_2 stays low | |
| 6 | With the antenna sense held low, let attempts run | Exactly 3 antenna burns, then FAILED; no further burns across 3 more power cycles | |
| 7 | Remove power during a burn | On restore: burn lines low from power-on; the attempt counted; no immediate re-burn | |
| 8 | Hold the antenna sense high before 30 min | No antenna burn; solar burn proceeds | |

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
