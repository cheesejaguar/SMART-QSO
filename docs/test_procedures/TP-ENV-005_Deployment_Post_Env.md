# Test Procedure: Deployment Test (Post-Environmental)
## TP-ENV-005

**Document Number:** SQSO-TP-ENV-005
**Version:** 1.0
**Date:** 2026-01-02
**Status:** CDR Baseline

---

## 1. PURPOSE

This procedure defines the deployment test to be performed after completion of environmental testing to verify that all deployables function correctly after exposure to qualification environments.

---

## 2. SCOPE

- Antenna deployment test (flight-like actuation)
- Solar panel deployment test (if applicable)
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
| Power supply | 0-12V, 5A |
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

### 7.3 Antenna Deployment Test

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Remove RBF pin | Deployment enabled | |
| 2 | Start video recording | Recording active | |
| 3 | Start timer | T=0 | |
| 4 | Command antenna deploy | Command acknowledged | |
| 5 | Monitor burn wire current | Current pulse detected | |
| 6 | Record burn time | Time to release | |
| 7 | Observe antenna deployment | Elements deploy | |
| 8 | Record deployment time | Total deployment time | |
| 9 | Verify full deployment | Elements fully extended | |
| 10 | Stop video recording | Recording saved | |

### 7.4 Solar Panel Deployment Test (If Applicable)

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Reset/restow or new test article | Deployables stowed | |
| 2 | Start video recording | Recording active | |
| 3 | Start timer | T=0 | |
| 4 | Command solar panel deploy | Command acknowledged | |
| 5 | Monitor burn wire current | Current pulse detected | |
| 6 | Record burn time | Time to release | |
| 7 | Observe panel deployment | Panels deploy | |
| 8 | Record deployment time | Total deployment time | |
| 9 | Verify full deployment | Panels locked at 90° | |
| 10 | Stop video recording | Recording saved | |

### 7.5 Deployment Sequence Timing Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Power on system | Boot complete | |
| 2 | Simulate separation switch | Timer starts | |
| 3 | Verify T+30 min timer | Timer counting | |
| 4 | Wait for deployment command | At T+30 min | |
| 5 | Verify sequence | Antenna first, panels second | |
| 6 | Record all timing | Per specification | |

### 7.6 Post-Deployment Verification

| Step | Action | Expected | P/F |
|------|--------|----------|-----|
| 1 | Visual inspection | No damage | |
| 2 | Photograph deployed config | Photos captured | |
| 3 | RF test (if antenna) | VSWR < 2:1 | |
| 4 | Solar panel I-V (if panels) | Characteristic correct | |
| 5 | Mechanical check | Locked position | |

---

## 8. PASS/FAIL CRITERIA

| Criterion | Requirement |
|-----------|-------------|
| RBF inhibit | Prevents deployment when installed |
| Burn wire time | < 5 seconds to release |
| Deployment time | < 30 seconds to full deploy |
| Full deployment | All elements fully deployed |
| No damage | No visible damage to mechanism |
| RF (antenna) | VSWR < 2:1 after deployment |
| Lock (panels) | Panels locked at deployment angle |

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
