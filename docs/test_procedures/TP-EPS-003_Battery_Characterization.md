# Test Procedure: Battery Characterization
## TP-EPS-003

**Document Number:** SQSO-TP-EPS-003
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Characterize the battery pack capacity, internal resistance, and charge/discharge performance.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Battery cycler | 0-5V, 0-5A per channel |
| Thermocouple logger | 4+ channels |
| Thermal chamber | -20°C to +60°C |

---

## 3. PROCEDURE

### 3.1 Capacity Test (25°C)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Charge to 4.2V @ C/5 | Full charge | |
| 2 | Rest 30 min | Voltage stable | |
| 3 | Discharge @ C/5 to 3.0V | Record capacity | |
| 4 | Record capacity | > 3.0 Ah | |

### 3.2 Internal Resistance

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Charge to 50% SOC | | |
| 2 | Apply 1A pulse, 10s | Record V drop | |
| 3 | Calculate IR | < 100 mΩ | |

### 3.3 Temperature Performance

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Repeat capacity @ -10°C | > 80% of 25°C | |
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
