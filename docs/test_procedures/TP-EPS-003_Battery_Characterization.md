# Test Procedure: Battery Characterization
## TP-EPS-003

**Document Number:** SQSO-TP-EPS-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-110, SYS-111, SYS-114

---

## 1. PURPOSE

Characterize the battery pack capacity, internal resistance, and charge/discharge performance. The pack is 4S1P Li-ion
18650, ≈ 3.5 Ah, 14.4 V nominal, ≈ 50 Wh, charged to **4.10 V/cell (16.4 V)**, not 4.20 V (decision D8); cell
over-voltage protection ≤ 4.25 V/cell; charging only between 0 °C and +45 °C (`docs/BASELINE_PARAMETERS.md` §5). Cell
screening and acceptance are V-BSA-1 (`docs/environment/BATTERY_SAFETY_ASSESSMENT.md`).

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Battery cycler | 0-20 V pack (0-5 V per cell channel), 0-5 A |
| Thermocouple logger | 4+ channels |
| Thermal chamber | -20°C to +60°C |

---

## 3. PROCEDURE

### 3.1 Capacity Test (25°C)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Charge to 4.10 V/cell (16.4 V) @ C/5 (D8) | Full charge at the flight set point | |
| 2 | Rest 30 min | Voltage stable | |
| 3 | Discharge @ C/5 to 3.0 V/cell | Record capacity | |
| 4 | Record capacity | > 3.0 Ah (record; the ≈ 3.5 Ah rating is at 4.20 V) | |

### 3.2 Internal Resistance

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Charge to 50% SOC | | |
| 2 | Apply 1A pulse, 10s | Record V drop | |
| 3 | Calculate IR | < 100 mΩ | |

### 3.3 Temperature Performance

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Repeat capacity @ -10°C (discharge only; charge at ≥ 0 °C, SYS-602) | > 80% of 25°C | |
| 2 | Repeat capacity @ +45°C | > 90% of 25°C | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Capacity @ 25°C | > 3.0 Ah |
| Internal resistance | < 100 mΩ |
| Capacity retention | Per spec |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
