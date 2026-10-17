# Test Procedure: ADCS Algorithm Test
## TP-ADCS-003

**Document Number:** SQSO-TP-ADCS-003
**Version:** 1.2
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** MRD-061, SYS-421, SYS-422

---

## 1. PURPOSE

Verify ADCS attitude determination and control algorithms.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Helmholtz cage | 3-axis, ±1 Gauss |
| Rate table | 0.1-10°/s |
| Sun simulator | Collimated light source |

---

## 3. PROCEDURE

### 3.1 Attitude Determination (TRIAD) [MRD-061]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Apply known B-field | Mag reading correct | |
| 2 | Illuminate known sun angle | Sun sensor correct | |
| 3 | Verify attitude solution | < 5° error | |

### 3.2 B-dot Detumble [SYS-421]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Simulate rotation | Rate detected | |
| 2 | Enable B-dot controller | Actuators respond | |
| 3 | Verify opposing torque | Correct direction | |

**Proposed (PI to ratify), TBR-TPADCS003-001:** Step 1: rotate the unit on the rate table at 0.5, 5 and 10°/s about each axis in turn; pass if the reported rate is within ±0.1°/s at 0.5°/s and within ±5 % at 5 and 10°/s. Steps 2 and 3: in the Helmholtz cage, apply a field that rotates at the same rates; pass if each magnetorquer's commanded dipole has the sign that opposes the change in the measured field and reaches at least 0.15 A·m² at saturation (`docs/BASELINE_PARAMETERS.md` §8). §4 "B-dot convergence": replace "Damping observed" by the mission detumble criterion once the PI ratifies it (proposed: below 0.5°/s within 3 orbits from 10°/s, shown by simulation and a hardware-in-the-loop run). The rate table covers 0.1–10°/s (§2) and 10°/s is the baseline's worst tip-off rate. ±0.1°/s is a fifth of the 0.5°/s detumble exit threshold, so measurement error cannot decide the exit.

### 3.3 Sun Pointing [SYS-422]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Set sun vector | Known direction | |
| 2 | Enable sun pointing | Tracks sun | |
| 3 | Verify pointing accuracy | < 10° error | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Attitude accuracy | < 5° |
| B-dot convergence | Damping observed |
| Sun pointing | < 10° |

---

## 5. TBD/TBR ITEMS

| ID | Item | Closure Plan | Target |
|----|------|--------------|--------|
| TBR-TPADCS003-001 | §3.2 and §4 give no pass values for detumble ("Rate detected", "Damping observed") | PI ratifies, amends or rejects the values proposed beneath §3.2, with the mission detumble requirement (finding VV-16) | TBD |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Proposed pass values for §3.2 and §4, for the PI to ratify, beneath §3.2; tracked as TBR-TPADCS003-001 in §5 (finding VV-16) |
| 1.2 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
