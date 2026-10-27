# Test Procedure: Jetson Thermal Test
## TP-JETSON-002

**Document Number:** SQSO-TP-JETSON-002
**Version:** 1.1
**Date:** 2026-01-02
**Status:** CDR Baseline

**Requirements verified:** SYS-504, SYS-603
**Verification items:** VI-01, V-THM-3, V-JET-7 (`docs/verification/VI_REGISTER.md`)

---

## 1. PURPOSE

Verify Jetson module thermal performance under load conditions, and measure the heat-path values that the thermal
model needs for SYS-603: spreader-to-frame conductance and junction-to-spreader resistance. The steps below are a bench
screen; the VI-01 measurement is made in vacuum on the EM (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md`).

---

## 2. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Thermocouple logger | 4+ channels |
| Thermal camera | ±2°C accuracy |
| Power analyzer | DC, 0-30V, 0-5A |

---

## 3. PROCEDURE [SYS-504, SYS-603]

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Boot Jetson, record idle temp | < 50°C | |
| 2 | Run one flight burst: power-on of at most 10 min including boot, 7 W mode, ≤ 8 W at the carrier input (BASELINE §6) | Temp rising | |
| 3 | Record peak junction temp | ≤ 95 °C (SYS-603) | |
| 4 | Record heatspreader temp | Record; with the measured conductance the model gives ≤ +40 °C for a 600 s batch in the hot case (SYS-603) | |
| 5 | Stuck-on case: hold the module at 8 W until steady | Record; model with the measured values gives the spreader ≤ +70 °C (SYS-603) | |
| 6 | Record thermal throttling | If occurs | |
| 7 | Power off, record cool-down | < 50°C in 5 min | |

---

## 4. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| Idle temperature | < 50°C |
| Junction | ≤ 95 °C (SYS-603) |
| Heat path | Junction-to-spreader ≤ 5.7 K/W (design 1.09–1.16 K/W); conductance measured (design 1.10 W/K); with them the correlated model meets SYS-603 (V-THM-3, VI-01) |
| Cool-down | < 5 minutes |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; steps whose values conflict with a requirement are listed, not cited (finding VV-06) |
