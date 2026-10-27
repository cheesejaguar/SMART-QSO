# Test Procedure: Thermal Vacuum Test
## TP-ENV-003

**Document Number:** TP-ENV-003
**Revision:** F
**Date:** 2026-10-27
**Reference:** GSFC-STD-7000B (GEVS) §2.6.3 and Appendix B §2.6; `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev D (amended) §3.4-§3.5
and §9 deviation 8 (levels); `hardware/structure/THERMAL_DESIGN.md` 1.5 §1.1 (allowable flight temperatures, AFT); SpaceX Rideshare Payload User's Guide v11 Table 6-2; LSP-REQ-317.01 Rev B Table 1 (bake-out)

**Requirements verified:** SYS-600, SYS-601 (cold survival limit open, TBR-TPENV003-001), SYS-602, SYS-702, SYS-706, SYS-707

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. Purpose

Verify functional and survival performance of the SMART-QSO CubeSat across operational and non-operational temperature extremes in vacuum conditions.

## 2. Test Equipment

| Equipment | Specification |
|-----------|---------------|
| Thermal vacuum chamber | < 10⁻⁵ Torr; platen and shroud -50 °C to +80 °C (the cold survival plateau needs -40 °C at the structure with margin for control) |
| Temperature controllers | Shroud and platen |
| Thermocouples | Type T, 30+ channels |
| Pressure gauge | Ion gauge, convection gauge |
| Power supply | External, isolated |
| Data acquisition | 100+ channel capacity |
| GSE | Complete test harness |

## 3. Test Levels

### 3.1 Temperature Limits [SYS-600, SYS-601]

| Parameter | Hot (°C) | Cold (°C) |
|-----------|----------|-----------|
| Operational AFT (SYS-600 as proposed in CR-2026-017; THERMAL_DESIGN §1.1) | +50 | -30 (boards); -40 for the first orbit after separation |
| Survival requirement (SYS-601) | +60 | -40 (cold survival limit open: TBR-TPENV003-001) |
| **Operational test plateau** (AFT ± 10 °C, GSFC-STD-7000B §2.6.3.2.1.1) | **+60** | **-40** (also the first-orbit extension at its AFT, without the 10 °C margin: ETP §9 deviation 8) |
| **Non-operating test plateau** (capped by the battery's +60 °C limit, ETP §3.4 and §9 deviation 1) | **+60** | **-40 at the structure, battery heater enabled, battery above -20** |
| Bake-out (SYS-706) | +60 (+0/-2) for ≥ 6 h after stabilisation at ≤ 1 × 10⁻⁴ Torr | — |

Rev C used a ± 5 °C protoflight margin; GEVS-B requires at least 10 °C beyond the allowable flight temperatures. When
the correlated thermal model exists, each plateau becomes max(margined prediction + 10 °C, AFT ± 10 °C), re-checked
against the battery cap.

### 3.2 Test Profile

| Phase | Temperature | Duration | Pressure |
|-------|-------------|----------|----------|
| Pumpdown | Ambient | Until < 10⁻⁵ Torr (spacecraft unpowered: launch configuration) | Decreasing |
| Bake-out (outgassing phase) | +60 °C | ≥ 6 hours after stabilisation | ≤ 1 × 10⁻⁴ Torr |
| Hot start | +60 °C | Power on after bake-out; boot within 30 s (SYS-300) | < 10⁻⁵ Torr |
| Hot operational | +60 °C | ≥ 3 hours (stabilisation + performance test); one Jetson burst (one power-on of at most 10 min including boot, BASELINE §6, SYS-512) | < 10⁻⁵ Torr |
| Hot non-operating | +60 °C | ≥ 2 hours | < 10⁻⁵ Torr |
| Transition | ≤ 1 °C/min | | < 10⁻⁵ Torr |
| Cold operational | -40 °C | ≥ 3 hours; charging must be inhibited below 0 °C (SYS-602); battery heater enabled, battery above -20 °C; Jetson off below its -15 °C gate | < 10⁻⁵ Torr |
| Cold non-operating | -40 °C at the structure (TBR-TPENV003-001) | ≥ 2 hours | < 10⁻⁵ Torr |
| Cold start | -40 °C | Power on after the cold non-operating plateau: the first-orbit extension of the AFT to -40 °C, tested at its AFT without the 10 °C margin (ETP §9 deviation 8); preceded on the EM by V-THM-8 (`docs/environment/THERMAL_ANALYSIS.md`) | < 10⁻⁵ Torr |
| Thermal balance | Two steady-state points (hot with burst, cold safe mode) | Until < 0.1 °C/h | < 10⁻⁵ Torr |
| **Cycles** | **8 complete** (plus 4 board-level air cycles before integration: 12 total, GEVS-B Table 2.6-1) | | |

Stabilisation: 98 % of the temperature sensors within 2 °C of the plateau, or changing < 0.1 °C/h (GEVS-B
§2.6.3.2.3). A comprehensive performance test (TP-SYS-001) runs at the first and last hot and cold operational
plateaus; a limited functional test at the others.

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

### 4.3 Pumpdown [SYS-702]

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
| 1 | Ramp to +60°C (hot operational plateau) | ≤ 1°C/min | ______ °C/min | |
| 2 | Stabilize at +60°C | 98 % of TCs within ±2°C, or < 0.1 °C/h | ______ | |
| 3 | Hot soak (≥ 3 hrs) with one Jetson burst (≤ 10 min including boot); battery charging inhibited above +45 °C | Maintain +60°C; burst ends on the OBC limit | ______ hrs | |
| 4 | Functional test | TP-SYS-001 pass | ______ | |
| 5 | Power off | Spacecraft safe | ______ | |
| 6 | Hold +60°C unpowered (hot non-operating) | Survival temp | ______ °C | |
| 7 | Survival soak (≥ 2 hrs) | Maintain +60°C | ______ hrs | |

#### Cold Phase

| Step | Action | Expected | Measured | P/F |
|------|--------|----------|----------|-----|
| 1 | Ramp to -40°C | ≤ 1°C/min | ______ °C/min | |
| 2 | Stabilize at -40°C | 98 % of TCs within ±2°C, or < 0.1 °C/h | ______ | |
| 3 | Power on (cold start at -40 °C, ETP §9 deviation 8); Jetson stays off below -15 °C | Successful boot | ______ | |
| 4 | Cold soak (≥ 3 hrs); charging inhibited below 0 °C | Maintain -40°C | ______ hrs | |
| 5 | Functional test | TP-SYS-001 pass | ______ | |
| 6 | Power off | Spacecraft safe | ______ | |
| 7 | Ramp to the cold survival limit, -40°C or -30°C (TBR-TPENV003-001) | Survival temp | ______ °C | |
| 8 | Survival soak (≥ 2 hrs) | Maintain the cold survival limit (TBR-TPENV003-001) | ______ hrs | |

### 4.5 Thermal Cycles 2-8

| Cycle | Hot Op | Hot Surv | Cold Op | Cold Surv | Functional | P/F |
|-------|--------|----------|---------|-----------|------------|-----|
| 2 | ______ | ______ | ______ | ______ | ______ | |
| 3 | ______ | ______ | ______ | ______ | ______ | |
| 4 | ______ | ______ | ______ | ______ | ______ | |
| 5 | ______ | ______ | ______ | ______ | ______ | |
| 6 | ______ | ______ | ______ | ______ | ______ | |
| 7 | ______ | ______ | ______ | ______ | ______ | |
| 8 | ______ | ______ | ______ | ______ | ______ | |

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
| 3 | Burn-wire continuity and stowed-state telemetry (deployments run before vibration: ETP §3.8) | Continuity within the pre-test value ± 10 %; stowed | ______ | |
| 4 | Remove thermocouples | Document locations | ______ | |

## 5. Temperature Monitoring Points

| Location | TC# | Hot Limit | Cold Limit |
|----------|-----|-----------|------------|
| OBC | TC01 | +70°C | -40°C |
| Battery | TC02 | +60°C (non-operating; charging inhibited above +45 °C) | -20°C (charging inhibited below 0 °C) |
| Jetson | TC03 | +70°C | -40°C unpowered (V-THM-8); not powered below -15°C |
| RF PA | TC04 | +65°C | -40°C |
| Structure +X | TC05 | +80°C | -40°C |
| Structure -X | TC06 | +80°C | -40°C |
| Solar panel | TC07 | +100°C | -60°C |
| Antenna | TC08 | +85°C | -50°C |

## 6. Pass/Fail Criteria

- Complete the bake-out and 8 thermal cycles
- All functional tests pass at hot/cold operational
- No damage after survival temperatures
- Temperature rate ≤ 1°C/min
- Charging inhibited above +45 °C and below 0 °C, shown by the EPS telemetry
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
| Bake-out | +60°C, ≥ 6 h, ≤ 1 × 10⁻⁴ Torr | ______ | |
| Hot operational | +60°C ± 2°C | ______ | |
| Hot survival | +60°C ± 2°C | ______ | |
| Cold operational | -40°C ± 2°C | ______ | |
| Cold survival | -40°C ± 2°C at the structure (TBR-TPENV003-001) | ______ | |
| Cycles complete | 8 | ______ | |
| Test pressure | < 10⁻⁵ Torr | ______ | |
| Functional at hot | Pass | ______ | |
| Functional at cold | Pass | ______ | |
| Post-test functional | Pass | ______ | |

## 9. TBD/TBR Items

| ID | Item | Owner | Closure Plan | Needed by |
|----|------|-------|--------------|-----------|
| TBR-TPENV003-001 | Cold survival limit: SYS-601 requires survival from -40 °C; Rev B of this procedure set its cold survival limit at -30 °C | PI | The PI sets the survival range: test to -40 °C (SYS-601) or amend SYS-601. Rev D tests -40 °C at the structure with the battery heater on and the battery above -20 °C (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §3.4), which is SYS-601 without the GEVS 10 °C margin; the RF PA cold limit in §5 is -30 °C, so the PA's own survival rating must be confirmed. The EM thermal-vacuum test and the correlated model (ETP VI-03) give the evidence. SYS-601 governs until it is amended | EM TVAC, 2027-07 |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | Requirement IDs added to the header and to the sections that verify them; it had cited only standards (finding VV-06) | SMART-QSO Team |
| C | 2026-10-16 | Cold survival limit marked open between -40 °C (SYS-601) and -30 °C (Rev B), tracked as TBR-TPENV003-001 in the new §9 for the PI; §3.1 cites SYS-601 (findings VV-06, VV-17) | SMART-QSO Team |
| D | 2026-10-27 | CDR review (PM/SE/V&V): levels from `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C: operational plateaus +60/-30 °C (10 °C margin, GSFC-STD-7000B), non-operating +60 °C (battery cap) and -40 °C at the structure, bake-out +60 °C for 6 h at ≤ 1 × 10⁻⁴ Torr, 8 cycles, hot and cold starts, thermal balance, ≤ 1 °C/min; battery monitoring limits corrected (the +45 °C limit is a charge limit); post-test deployment replaced by continuity; SYS-602, SYS-706, SYS-707 added. Was GSFC-STD-7000A, ± 5 °C, 4 cycles | CDR review, PM/SE/V&V |
| E | 2026-10-27 | CDR round 3, thermal (RID THM-04, CR-2026-017): levels from ETP Rev D (amended) §3.4: cold operational plateau and cold start at -40 °C (AFT -30 °C - 10 °C; the cold start tests the first-orbit extension of the AFT to -40 °C at its AFT without the 10 °C margin, ETP §9 deviation 8), V-THM-8 cited; operational AFT row from THERMAL_DESIGN 1.5 §1.1; Jetson and RF PA cold monitoring limits -40 °C | CDR thermal analyst |
| F | 2026-10-27 | Documentation audit (post-CDR 2026): Jetson burst is one power-on ≤ 10 min (C-20; was 120 s); not-yet-run banner | Documentation audit |
