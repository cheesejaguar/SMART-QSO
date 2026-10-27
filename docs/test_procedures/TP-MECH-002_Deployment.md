# Test Procedure: Deployable Mechanism Test
## TP-MECH-002

**Document Number:** SQSO-TP-MECH-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify all deployable mechanisms: the two solar wings (two burn resistors per wing, each on its own 0.8 A
current-limited driver, burned one at a time for 10 s, decision C-17, with a 12 s hardware on-time limit per driver)
and then the ISIS AntS, deployed by its own controller over I2C (`docs/BASELINE_PARAMETERS.md` §4;
`docs/hardware/EPS_DESIGN.md` §5.1; `docs/interfaces/ICD_OBC_ADCS.md` §12.3; SRS-F089). Wings first, antennas second.
Environmental deployment tests (hot, cold, minimum voltage) are TP-ENV-005; antenna RF checks are TP-RF-005.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Power supply | Pack simulator 11.2–16.4 V, 5 A (`docs/hardware/EPS_DESIGN.md` §4) |
| Timer | 0.1s resolution |
| Video camera | HD, 60 fps |

---

## 3. PROCEDURE

### 3.1 Solar Wing Deployment

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify stowed | Constraint intact | |
| 2 | Command deploy (wing 1 resistor A) | BURN_W1A high; driver current 0.800 A ± 10 % (ICD_OBC_ADCS §12.4) | |
| 3 | Time the burn | 10 s (C-17); the hardware one-shot ends it by 12.0 ± 0.6 s; the second resistor of a wing fires only if the wing has not released | |
| 4 | Verify full deployment | Wing at its deployed stop; deployment sense agrees (SYS-709) | |
| 5 | Repeat for wing 2 (BURN_W2A) | As steps 2–4; never two resistors at once | |

### 3.2 Antenna Deployment (ISIS AntS)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Verify stowed | Constraint intact | |
| 2 | After both wings: AntS temperature read (0xC0), ARM, automatic deployment, STATUS polled | Gate ≥ −17 °C (C-18); each element 10 s (ICD_OBC_ADCS §12.3) | |
| 3 | Verify full deployment | All four elements extended; status bits and housekeeping `DEP` agree | |
| 4 | Verify the transmitters stayed silent until the AntS reported deployed | No RF before then (decision C-19, SRS-F167) | |

### 3.3 Deployment timer and burn safety across resets (SYS-221, SW-10)

Use burn-resistor dummy loads (or flight resistors with spare restraints) and an AntS simulator on I2C3, the hardware inhibits removed for this test only, and a scope on DEPLOY_EN and the four burn lines BURN_W1A, BURN_W1B, BURN_W2A, BURN_W2B.

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Send `arm-flight` through the umbilical; power off | Reply `SQGR 00` | |
| 2 | Power on with the switches released; start a timer | No burn line high | |
| 3 | At 20 min, force a watchdog reset | Count resumes (boot log); no burn before 30:00 from step 2 | |
| 4 | Repeat from step 1; at 25 min remove and restore power (switch toggle) | Count restarts; no burn before 30:00 from the power restore | |
| 5 | At 30 min | DEPLOY_EN and BURN_W1A high for 10 s, then BURN_W2A for 10 s; the B lines only for a wing sensed not released; never two lines at once (SRS-F089) | |
| 6 | With the AntS simulator reporting not deployed, let attempts run | Attempts 50 min apart (SRS-F088), bounded and persisted across resets (SYS-221); no further attempts across 3 more power cycles once spent | |
| 7 | Remove power during a burn | On restore: burn lines low from power-on; the attempt counted; no immediate re-burn | |
| 8 | Hold a wing's deployment sense at "deployed" before 30 min | That wing's resistors are not fired; the other wing and the AntS proceed (SYS-221) | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Deployment success | 100% |
| Burn time | 10 s per resistor, ended by 12.0 ± 0.6 s in hardware (C-17) |
| Full extension | All elements |
| No damage | Visual inspection |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
