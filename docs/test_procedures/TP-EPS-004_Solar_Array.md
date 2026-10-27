# Test Procedure: Solar Array Characterization
## TP-EPS-004

**Document Number:** SQSO-TP-EPS-004
**Version:** 1.2
**Date:** 2026-10-27
**Status:** Draft; not yet run (no hardware exists)

**Requirements verified:** SYS-102; SYS-100 and SYS-103 by analysis from the §3.1 I-V data

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

Characterize solar array I-V curves and verify MPPT performance. The array is 16 triple-junction cells: 2 on each of
the 4 long body faces and 2 deployable wings with 2 cells per side, on four trackers (`docs/BASELINE_PARAMETERS.md` §5;
`docs/hardware/EPS_DESIGN.md`).

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Solar simulator | AM0, 1 sun |
| I-V tracer | 0-10V, 0-2A |
| Electronic load | 0-10A programmable |

---

## 3. PROCEDURE

### 3.1 I-V Curve Measurement [SYS-100, SYS-103]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Illuminate @ AM0 | Stable illumination | |
| 2 | Measure Voc | Record open circuit V | |
| 3 | Measure Isc | Record short circuit I | |
| 4 | Sweep I-V curve | Record Pmax, Vmpp, Impp | |

### 3.2 MPPT Verification [SYS-102]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect to MPPT | Tracking active | |
| 2 | Measure power extracted | Within 95% of Pmax | |
| 2a | Tracker efficiency over 3.5–6.3 V, 0.05–1.1 A at −20, +25 and +60 °C, synchronised | ≥ 92 % above 2 W, ≥ 90 % above 1 W (V-EPS-1) | |
| 3 | Vary illumination | MPPT tracks | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Cell efficiency | Per spec |
| MPPT efficiency | > 95% |
| Tracker efficiency | ≥ 92 % above 2 W, ≥ 90 % above 1 W (V-EPS-1) |
| Array power | Per power budget: the measured I-V curves entered in the power analysis, which must still deliver ≥ 2.23 W orbit average at end of life tumbling (SYS-100; simulated 2.48 W, `docs/environment/POWER_BUDGET_ANALYSIS.md`) |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): 16-cell array described; V-EPS-1 tracker efficiency and SYS-100 2.23 W; status and banner |
