# Test Procedure: Mechanical Fit Check
## TP-MECH-001

**Document Number:** SQSO-TP-MECH-001
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-001, SYS-004

---

## 1. PURPOSE

Verify CubeSat mechanical fit to dispenser interface per CDS Rev 14.1 and the integrator's dispenser, the Exolaunch
EXOpod Nova on the baseline launch path (`docs/BASELINE_PARAMETERS.md` §3; SYS-900).

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Integrator's test pod (EXOpod Nova) or equivalent | Per the integrator's ICD (SYS-900) |
| Go/no-go gauges | Rail dimensions |
| Calipers | 0.01mm resolution |

---

## 3. PROCEDURE [SYS-001, SYS-004]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Measure rail width | ≥ 8.5 mm (SYS-080; CDS §2.2.5) | |
| 2 | Measure rail length | 113.5 ±0.1 mm | |
| 3 | Measure overall width | 100 ±0.1 mm | |
| 4 | Verify rail edge radii | ≥ 1 mm (SYS-080; CDS §2.2.7); 0.5 ± 0.1 mm only if the NRCSD backup is used (BASELINE §3) | |
| 5 | Insert into Test-POD | Smooth insertion | |
| 6 | Verify deployment switch contact | All three switches (DS1–DS3) actuated (BASELINE §4) | |
| 7 | Verify no interference | Clear of structure | |
| 8 | Extract from Test-POD | Smooth extraction | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Rail dimensions | Per CDS Rev 14.1 |
| Fit in Test-POD | Smooth, no binding |
| Switch contact | Proper actuation |
| No interference | Verified |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
