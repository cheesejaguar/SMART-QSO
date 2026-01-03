# Test Procedure: Solar Array Characterization
## TP-EPS-004

**Document Number:** SQSO-TP-EPS-004
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Characterize solar array I-V curves and verify MPPT performance.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Solar simulator | AM0, 1 sun |
| I-V tracer | 0-10V, 0-2A |
| Electronic load | 0-10A programmable |

---

## 3. PROCEDURE

### 3.1 I-V Curve Measurement

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Illuminate @ AM0 | Stable illumination | |
| 2 | Measure Voc | Record open circuit V | |
| 3 | Measure Isc | Record short circuit I | |
| 4 | Sweep I-V curve | Record Pmax, Vmpp, Impp | |

### 3.2 MPPT Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Connect to MPPT | Tracking active | |
| 2 | Measure power extracted | Within 95% of Pmax | |
| 3 | Vary illumination | MPPT tracks | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Cell efficiency | Per spec |
| MPPT efficiency | > 95% |
| Array power | Per power budget |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
