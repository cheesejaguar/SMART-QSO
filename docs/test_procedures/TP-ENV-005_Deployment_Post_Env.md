# Test Procedure: Deployment Test (Post-Environmental)
## TP-ENV-005

**Document Number:** SQSO-TP-ENV-005
**Version:** 1.2
**Date:** 2026-10-27
**Status:** Draft; no test article exists

**Requirements verified:** SYS-220, SYS-221, SYS-709

> **Order changed at the CDR review (2026-10-27).** CDS Rev 14.1 §3.5.2 forbids modifying the flight unit after
> protoflight testing, and §3.5.2.1 notes that providers may treat a deployment as a configuration change that
> invalidates the vibration test. On the flight model this procedure therefore runs **before** the protoflight
> vibration (`docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev E §3.8 and §4 step 3): ambient, then hot (+60 °C) and
> cold (−55 °C, the bounding stowed-wing temperature at the burn with margin, RID P3S-01; was −30 °C) in a thermal chamber, at the minimum battery voltage of the end-of-life state of charge, followed by a
> re-stow per the vendor procedure and an inspection. After vibration and TVAC, only burn-wire continuity and
> stowed-state telemetry are checked unless the integrator approves a deployment in writing. On the engineering
> model the procedure runs at least five times per mechanism, including after the EM vibration. Where §5 below says
> "Environmental testing complete", read "Pre-environmental baseline complete" for the flight model.

> **Procedure for a test not yet run.** No hardware has been built and this procedure has never been executed
> (`docs/BASELINE_PARAMETERS.md` §1; `docs/project/HARDWARE_READINESS_TRACKER.md`). Every P/F, measured-value, date and
> signature field is blank until the test is run. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended) and the
> verification items of `docs/verification/VI_REGISTER.md`; where a step disagrees with them, they govern.

---

## 1. PURPOSE

This procedure defines the deployment test to be performed after completion of environmental testing to verify that all deployables function correctly after exposure to qualification environments.

---

## 2. SCOPE

- Solar wing deployment test (two wings, two burn resistors each, 10 s per resistor, decision C-17)
- Antenna deployment test (ISIS AntS, by its own controller over I2C, after the wings; `docs/BASELINE_PARAMETERS.md` §4)
- Deployment sequence timing verification
- RBF pin function verification

---

## 3. APPLICABLE DOCUMENTS

1. hardware/rf/ANTENNA_DESIGN.md
2. TP-ENV-003_Thermal_Vacuum.md (prerequisite)
3. TP-ENV-002_Vibration.md (prerequisite)
4. Deployment mechanism drawings

---

## 4. TEST EQUIPMENT

| Equipment | Specification |
|-----------|---------------|
| Power supply | Flight battery at the minimum voltage of the end-of-life state of charge, or a pack simulator 11.2–16.4 V, 5 A (SYS-709) |
| Timer/stopwatch | 0.1s resolution |
| Video camera | HD, 60 fps |
| Current monitor | 0-5A, mA resolution |
| Deployment fixture | Allows free deployment |
| RBF pin | Flight-equivalent |
| Safety glasses | Required for all personnel |

---

## 5. PRECONDITIONS

- [ ] Environmental testing complete (Vibration, TVAC)
- [ ] Post-environmental functional test complete
- [ ] Deployables stowed in flight configuration
- [ ] Burn wires installed (flight-equivalent)
- [ ] RBF pin installed
- [ ] Video recording ready
- [ ] All personnel briefed on safety

---

## 6. SAFETY

### 6.1 Hazards

| Hazard | Mitigation |
|--------|------------|
| Spring-loaded elements | Safety glasses, keep clear of path |
| Hot burn wire | No touching during actuation |
| Electrical | Power limited, current monitored |

### 6.2 Personal Protective Equipment

- Safety glasses (mandatory)
- ESD wrist strap
- Clean room gloves

---

## 7. PROCEDURE

### 7.1 Pre-Deployment Configuration

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Visual inspection | Deployables stowed correctly | |
| 2 | Photograph stowed config | Photo captured | |
| 3 | Install deployment fixture | Secure mounting | |
| 4 | Connect power supply | Voltage verified | |
| 5 | Connect current monitor | Inline with deploy circuit | |
| 6 | Set up video camera | Focused on deployables | |
| 7 | Verify RBF pin installed | Pin in place | |
| 8 | Clear deployment zone | All personnel at safe distance | |

### 7.2 RBF Pin Test

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on with RBF in | No deployment | |
| 2 | Command deployment | Inhibited by RBF | |
| 3 | Verify inhibit telemetry | Inhibit status correct | |
| 4 | Remove RBF pin | Pin removed cleanly | |
| 5 | Verify inhibit cleared | Status shows enabled | |
| 6 | Re-install RBF pin | For next test phase | |

### 7.3 Antenna Deployment Test (after the wings, §7.4)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Remove RBF pin | Deployment enabled | |
| 2 | Start video recording | Recording active | |
| 3 | Start timer | T=0 | |
| 4 | Command antenna deploy (AntS arm and automatic deployment over I2C3) | Command acknowledged; AntS temperature read ≥ −17 °C, or the gate's deferral recorded (decision C-18) | |
| 5 | Monitor the AntS supply current (3V3_ANTS) | ≤ 0.2 A at the pack (`docs/hardware/EPS_DESIGN.md` §5.1) | |
| 6 | Record each element's time to release from the status word | Time to release | |
| 7 | Observe antenna deployment | Elements deploy | |
| 8 | Record deployment time | Total deployment time | |
| 9 | Verify full deployment | Elements fully extended | |
| 10 | Stop video recording | Recording saved | |

### 7.4 Solar Wing Deployment Test (first)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Reset/restow or new test article | Deployables stowed | |
| 2 | Start video recording | Recording active | |
| 3 | Start timer | T=0 | |
| 4 | Command wing deployment (resistor A of each wing, one at a time) | Command acknowledged | |
| 5 | Monitor burn driver current | 0.800 A ± 10 % (ICD_OBC_ADCS §12.4) | |
| 6 | Record burn time | 10 s per resistor; second resistor only if the wing has not released; hardware limit 12.0 ± 0.6 s | |
| 7 | Observe wing deployment | Wings deploy | |
| 8 | Record deployment time | Within 5 s of release; rebound settles within 2 s (VI-STR-05) | |
| 9 | Verify full deployment | Wings at their deployed stops; deployment sense agrees (SYS-709) | |
| 10 | Stop video recording | Recording saved | |

### 7.5 Deployment Sequence Timing Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on system | Boot complete | |
| 2 | Simulate separation switch | Timer starts | |
| 3 | Verify T+30 min timer | Timer counting | |
| 4 | Wait for deployment command | At T+30 min | |
| 5 | Verify sequence | Wings first (one resistor at a time), antennas second (`docs/BASELINE_PARAMETERS.md` §4; SRS-F089); no magnetorquer actuation or deployment before 120 s | |
| 6 | Record all timing | Per specification | |

### 7.6 Post-Deployment Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Visual inspection | No damage | |
| 2 | Photograph deployed config | Photos captured | |
| 3 | RF test (if antenna) | VSWR < 2:1 | |
| 4 | Solar wing I-V | Characteristic correct | |
| 5 | Mechanical check | Locked position | |

---

## 8. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| RBF inhibit | Prevents deployment when installed |
| Burn time | Release on the first resistor's 10 s burn at the cold temperature and the minimum bus voltage (VI-LV-17), with at least 2× margin at cold (ETP §3.8) |
| Deployment time | Wings within 5 s of release (VI-STR-05); AntS elements within the AntS's own time limit |
| Full deployment | All elements fully deployed |
| No damage | No visible damage to mechanism |
| RF (antenna) | VSWR < 2:1 after deployment |
| Wings | At their deployed stops |

---

## 9. DATA RECORDING

- [ ] Pre-deployment photos
- [ ] Post-deployment photos
- [ ] Video recordings of all deployments
- [ ] Current measurements
- [ ] Timing data
- [ ] RF test results (if antenna)
- [ ] Anomaly notes (if any)

---

## 10. ANOMALY PROCEDURES

### 10.1 Partial Deployment

1. Do not cycle power
2. Document position and configuration
3. Photograph anomaly
4. Contact cognizant engineer
5. Assess root cause before retry

### 10.2 No Deployment

1. Verify power and commands
2. Check burn wire resistance
3. Check constraint mechanism
4. Document all observations
5. Assess for retest or investigation

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial release |
| 1.1 | 2026-10-27 | CDR review (PM/SE/V&V): run before the protoflight vibration on the flight model, hot and cold at the minimum battery voltage (SYS-709); post-environment deployment only with the integrator's written approval (CDS Rev 14.1 §3.5.2) |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): wings first then the AntS (was antenna first); cold −55 °C (ETP Rev E §3.8; was −30 °C); 10 s burns, C-18 gate, VI-STR-05 and VI-LV-17 criteria; banner |
