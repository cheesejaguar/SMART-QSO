# Test Procedure: Thermal Vacuum Test
## TP-ENV-003

**Document Number:** TP-ENV-003
**Revision:** A
**Date:** 2026-01-02
**Reference:** GSFC-STD-7000A (GEVS)

---

## 1. Purpose

Verify functional and survival performance of the SMART-QSO CubeSat across operational and non-operational temperature extremes in vacuum conditions.

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Thermal vacuum chamber | < 10⁻⁵ Torr, -40°C to +80°C |
| Temperature controllers | Shroud and platen |
| Thermocouples | Type T, 30+ channels |
| Pressure gauge | Ion gauge, convection gauge |
| Power supply | External, isolated |
| Data acquisition | 100+ channel capacity |
| GSE | Complete test harness |

## 3. Test Levels

### 3.1 Temperature Limits

| Parameter | Hot (°C) | Cold (°C) |
|-----------|----------|-----------|
| Operational | +50 | -20 |
| Survival | +60 | -30 |
| Protoflight margin | +5 | -5 |

### 3.2 Test Profile

| Phase | Temperature | Duration | Pressure |
|-------|-------------|----------|----------|
| Pumpdown | Ambient | Until < 10⁻⁵ Torr | Decreasing |
| Hot operational | +50°C | 4 hours minimum | < 10⁻⁵ Torr |
| Hot survival | +60°C | 1 hour minimum | < 10⁻⁵ Torr |
| Transition | +60°C to -20°C | Natural | < 10⁻⁵ Torr |
| Cold operational | -20°C | 4 hours minimum | < 10⁻⁵ Torr |
| Cold survival | -30°C | 1 hour minimum | < 10⁻⁵ Torr |
| **Cycles** | **4 complete** | | |

## 4. Procedure

### 4.1 Pre-Test Baseline

| Step | Action | Expected Result | Result | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Visual inspection | No damage | ______ | |
| 2 | Functional test | TP-SYS-001 pass | ______ | |
| 3 | Install thermocouples | All channels reading | ______ | |
| 4 | Record ambient temps | Baseline documented | ______ | |
| 5 | Photograph spacecraft | Documentation | ______ | |

### 4.2 Chamber Setup

| Step | Action | Verification |
|------|--------|--------------|
| 1 | Mount spacecraft | Isolated from shroud |
| 2 | Route harness | Strain relief verified |
| 3 | Install MLI | Per thermal design |
| 4 | Connect GSE | All channels verified |
| 5 | Verify data acquisition | All TCs reading |

### 4.3 Pumpdown

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Close chamber | Sealed | |
| 2 | Start rough pump | Pressure dropping | |
| 3 | Monitor outgassing | No anomalies | |
| 4 | Switch to high vac | < 10⁻³ Torr | |
| 5 | Achieve test pressure | < 10⁻⁵ Torr | |
| 6 | Record pressure | ______ Torr | |

### 4.4 Thermal Cycle 1

#### Hot Phase

| Step | Action | Expected | Measured | P/F |
|------|--------|----------|----------|-----|
| 1 | Ramp to +50°C | 1-2°C/min rate | ______ °C/min | |
| 2 | Stabilize at +50°C | All TCs ±2°C | ______ | |
| 3 | Hot soak (4 hrs) | Maintain +50°C | ______ hrs | |
| 4 | Functional test | TP-SYS-001 pass | ______ | |
| 5 | Power off | Spacecraft safe | ______ | |
| 6 | Ramp to +60°C | Survival temp | ______ °C | |
| 7 | Survival soak (1 hr) | Maintain +60°C | ______ hrs | |

#### Cold Phase

| Step | Action | Expected | Measured | P/F |
|------|--------|----------|----------|-----|
| 1 | Ramp to -20°C | 1-2°C/min rate | ______ °C/min | |
| 2 | Stabilize at -20°C | All TCs ±2°C | ______ | |
| 3 | Power on | Successful boot | ______ | |
| 4 | Cold soak (4 hrs) | Maintain -20°C | ______ hrs | |
| 5 | Functional test | TP-SYS-001 pass | ______ | |
| 6 | Power off | Spacecraft safe | ______ | |
| 7 | Ramp to -30°C | Survival temp | ______ °C | |
| 8 | Survival soak (1 hr) | Maintain -30°C | ______ hrs | |

### 4.5 Thermal Cycles 2-4

| Cycle | Hot Op | Hot Surv | Cold Op | Cold Surv | Functional | P/F |
|-------|--------|----------|---------|-----------|------------|-----|
| 2 | ______ | ______ | ______ | ______ | ______ | |
| 3 | ______ | ______ | ______ | ______ | ______ | |
| 4 | ______ | ______ | ______ | ______ | ______ | |

### 4.6 Return to Ambient

| Step | Action | Result | P/F |
|------|--------|--------|-----|
| 1 | Ramp to ambient | Controlled rate | |
| 2 | Backfill with N2 | Slow backfill | |
| 3 | Return to atmosphere | Pressure equalized | |
| 4 | Open chamber | No condensation | |
| 5 | Remove spacecraft | Careful handling | |

### 4.7 Post-Test Verification

| Step | Action | Expected Result | Result | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Visual inspection | No damage | ______ | |
| 2 | Functional test | TP-SYS-001 pass | ______ | |
| 3 | Deployment test | Mechanisms operate | ______ | |
| 4 | Remove thermocouples | Document locations | ______ | |

## 5. Temperature Monitoring Points

| Location | TC# | Hot Limit | Cold Limit |
|----------|-----|-----------|------------|
| OBC | TC01 | +70°C | -40°C |
| Battery | TC02 | +45°C | 0°C |
| Jetson | TC03 | +70°C | -20°C |
| RF PA | TC04 | +65°C | -30°C |
| Structure +X | TC05 | +80°C | -40°C |
| Structure -X | TC06 | +80°C | -40°C |
| Solar panel | TC07 | +100°C | -60°C |
| Antenna | TC08 | +85°C | -50°C |

## 6. Pass/Fail Criteria

- Complete 4 thermal cycles
- All functional tests pass at hot/cold operational
- No damage after survival temperatures
- Temperature rate < 2°C/min
- Pressure maintained < 10⁻⁵ Torr
- Post-test functional complete

## 7. Abort Criteria

| Condition | Action |
|-----------|--------|
| Component exceeds limit | Abort cycle, safe spacecraft |
| Pressure rises > 10⁻³ Torr | Pause test, troubleshoot |
| GSE failure | Safe spacecraft |
| Chamber malfunction | Emergency backfill |

## 8. Test Data Summary

| Parameter | Requirement | Measured | P/F |
|-----------|-------------|----------|-----|
| Hot operational | +50°C ± 2°C | ______ | |
| Hot survival | +60°C ± 2°C | ______ | |
| Cold operational | -20°C ± 2°C | ______ | |
| Cold survival | -30°C ± 2°C | ______ | |
| Cycles complete | 4 | ______ | |
| Test pressure | < 10⁻⁵ Torr | ______ | |
| Functional at hot | Pass | ______ | |
| Functional at cold | Pass | ______ | |
| Post-test functional | Pass | ______ | |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
