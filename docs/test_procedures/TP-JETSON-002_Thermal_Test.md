# Test Procedure: Jetson Thermal Test
## TP-JETSON-002

**Document Number:** SQSO-TP-JETSON-002
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

Verify Jetson module thermal performance under load conditions.

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Thermocouple logger | 4+ channels |
| Thermal camera | ±2°C accuracy |
| Power analyzer | DC, 0-30V, 0-5A |

---

## 3. PROCEDURE

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Boot Jetson, record idle temp | < 50°C | |
| 2 | Run inference load 5 min | Temp rising | |
| 3 | Record peak module temp | < 75°C | |
| 4 | Record heatspreader temp | < 60°C | |
| 5 | Continue load 10 min | Stable or throttling | |
| 6 | Record thermal throttling | If occurs | |
| 7 | Power off, record cool-down | < 50°C in 5 min | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Idle temperature | < 50°C |
| Peak temperature | < 75°C |
| Heatspreader | < 60°C |
| Cool-down | < 5 minutes |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
