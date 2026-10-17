# Test Procedure: ADCS Algorithm Test
## TP-ADCS-003

**Document Number:** SQSO-TP-ADCS-003
**Version:** 1.1
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

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
