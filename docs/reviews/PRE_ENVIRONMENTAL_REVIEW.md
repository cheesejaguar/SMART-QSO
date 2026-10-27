# SMART-QSO Pre-Environmental Review Package

**Document ID**: SQSO-PER-001
**Version**: 1.2
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Template (Pre-PER). No PER has been held

> **Template for a future review: not held.** No pre-environmental review has been held, no test article exists and
> nothing in this document records a result; every status, serial number and signature field is to be filled in at
> the review. It belongs to the start of the **protoflight campaign** of `mission/SCHEDULE.md` (PF_ENV, 2028-03-03 to
> 2028-04-21, after the FM burn-in; it must end by L − 4 months, 2028-07-01); the EM development tests (EM_TEST,
> 2027-07-27 to 2027-09-21) can use the same form. Test flow, levels and pass criteria are those of
> `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` (SQSO-ETP-001 Rev E), which governs where this template differs; design
> values are in [BASELINE_PARAMETERS.md](../BASELINE_PARAMETERS.md). The current review record is
> [CDR_2026_DESIGN_REVIEW.md](CDR/CDR_2026_DESIGN_REVIEW.md). Index of the review documents: [README.md](README.md).

## 1. Purpose

This document provides the Pre-Environmental Review (PER) package for SMART-QSO per NASA Ames SmallSat heritage practices. PER confirms readiness to proceed with environmental qualification testing.

## 2. Review Objectives

### 2.1 PER Success Criteria

| Criterion | Requirement |
|-----------|-------------|
| Test article ready | Hardware assembled and functional |
| Test procedures approved | All environmental TPs reviewed |
| Facilities ready | Chambers reserved and calibrated |
| GSE ready | All test equipment calibrated |
| Team trained | Environmental test operators qualified |
| Pre-test baseline | Functional performance verified |

### 2.2 Entrance Criteria

- [ ] Independent CDR held (planned 2027-09-28) and its RIDs closed
- [ ] Flatsat testing complete
- [ ] FM integration, functional, day-in-the-life and burn-in complete (FM_INT, FM_FUNC, FM_BURNIN)
- [ ] Hardware configuration frozen
- [ ] Test procedures written and approved
- [ ] Facilities and GSE scheduled

## 3. Test Article Status

### 3.1 Hardware Configuration

| Subsystem | P/N | S/N | Status |
|-----------|-----|-----|--------|
| Structure | SQSO-MECH-CAD-001 | | [ ] Integrated |
| EPS board | SQSO-EPS-BRD-001 | | [ ] Integrated |
| Battery | 4S1P 18650 pack (BASELINE §5) | | [ ] Installed |
| OBC board (STM32L4R5ZI) | SQSO-OBC-BRD-001 | | [ ] Integrated |
| RF board | SQSO-RF-BRD-001 | | [ ] Integrated |
| Antenna | ISIS AntS 1U–3U | | [ ] Stowed |
| AI Payload: Jetson Orin Nano 8 GB module 900-13767-0030-000 on its carrier | SQSO-JET-BRD-001 | | [ ] Integrated |
| Harness | SQSO-ICX-001 | | [ ] Installed |

### 3.2 Software Configuration

| Software | Version | Status |
|----------|---------|--------|
| Flight Software | | [ ] Loaded |
| Payload Software | | [ ] Loaded |
| Configuration | | [ ] Applied |

### 3.3 Configuration Control

| Item | As-Designed | As-Built | Delta |
|------|-------------|----------|-------|
| Mass | CBE 1.52 kg, MEV 1.75 kg, limit 2.00 kg (BASELINE §4; `docs/environment/MASS_BUDGET.md`) | | |
| CG Location | Within 5 mm of the geometric centre (limit ±20 mm); CAD (+0.1, −0.3, −1.4) mm CBE (BASELINE §4) | | |
| Power Draw | SAFE 0.79 W, IDLE 1.18 W (BASELINE §5) | | |
| Dimensions | 100×100×113.5mm | | |

## 4. Pre-Environmental Functional Test

### 4.1 Baseline Performance

| Test | Requirement | Pre-Env Result |
|------|-------------|----------------|
| Boot time | <5 seconds | |
| Mode transitions | All pass | |
| Beacon generation | 60 s ACTIVE, 120 s IDLE, 180 s SAFE (BASELINE §8) | |
| Telemetry valid | All channels | |
| Command response | <1 second | |
| Power consumption | Per budget | |

### 4.2 Functional Test Results

```
PRE-ENVIRONMENTAL FUNCTIONAL TEST

Date: ___________
Test Conductor: ___________

TEST CASE                RESULT    NOTES
─────────────────────────────────────────
Boot sequence            [ ]
Safe mode operation      [ ]
Mode transitions         [ ]
Beacon transmission      [ ]
Command reception        [ ]
Telemetry generation     [ ]
AI payload function      [ ]
Fault handling           [ ]

OVERALL: [ ] PASS  [ ] FAIL

Signature: _________ Date: _____
```

### 4.3 Baseline Measurements

| Parameter | Pre-Env Value | Units |
|-----------|---------------|-------|
| Battery voltage | | V |
| Boot current peak | | mA |
| SAFE mode current | | mA |
| IDLE mode current | | mA |
| ACTIVE mode current | | mA |
| TX power output | | dBm |
| Beacon interval | | seconds |

## 5. Environmental Test Plan Summary

### 5.1 Test Sequence

The protoflight flow of SQSO-ETP-001 Rev E §4 (22.5 working days with transport):

| Order | Test | Duration | Procedure |
|-------|------|----------|-----------|
| 1 | Comprehensive performance test (baseline), RF characterisation | 3 days | TP-SYS-001, TP-RF-001 to -004, TP-ENV-001 |
| 2 | Mass properties; residual dipole | 2 days | TP-ENV-001; ETP §3.9 |
| 3 | Deployment tests ambient, hot and cold; re-stow; inspection | 3 days | TP-ENV-005, TP-MECH-002, TP-MECH-003 |
| 4 | Fit check in the integrator's test pod | 0.5 day | TP-MECH-001 |
| 5 | Random vibration, three axes, sine signatures, inhibit monitoring | 2 days | TP-ENV-002 |
| 6 | Post-vibration limited functional, inspection | 1 day | TP-SYS-001 (limited) |
| 7 | TVAC: pump-down, bake-out, hot start, 8 cycles, thermal balance, cold start | 6 days | TP-ENV-003 |
| 8 | EMC and RF self-compatibility, spurious | 2 days | TP-ENV-004, TP-RF-003 |
| 9 | Final comprehensive performance test, burn-wire continuity, mass, photographs | 3 days | TP-SYS-001, TP-FAT-001, TP-FAT-002 |

### 5.2 Test Levels

From SQSO-ETP-001 Rev E §3 and BASELINE §8; the integrator's ICD makes them final.

| Test | Level | Duration |
|------|-------|----------|
| Random Vibe | Protoflight: envelope of MPE + 3 dB and the GSFC-STD-7000B workmanship spectrum, **8.18 Grms** on the interim levels (UG v11 MPE 5.57 Grms) | 1 min/axis |
| Sine signature | 0.1 g minimum (0.25 g if the fixture allows), 5–2000 Hz, ≤ 3 oct/min, before and after random | — |
| TVAC plateaus | Allowable flight temperatures ± 10 °C, hot plateaus capped at +60 °C by the battery (`hardware/structure/THERMAL_DESIGN.md` §1.1) | ≥ 3 h operational plateaus |
| TVAC Cycles | 8 in vacuum (plus 4 board-level ambient cycles) | |
| Bake-out | +60 °C, ≥ 6 h after stabilisation, ≤ 1 × 10⁻⁴ Torr | |
| Vacuum | ≤ 1 × 10⁻⁵ Torr | |

### 5.3 Test Procedures

| Procedure | Version | Approved |
|-----------|---------|----------|
| TP-ENV-001 Pre-Environmental (functional baseline, mass properties) | | [ ] |
| TP-ENV-002 Vibration | | [ ] |
| TP-ENV-003 Thermal Vacuum | | [ ] |
| TP-ENV-004 EMC | | [ ] |
| TP-ENV-005 Deployment, post-environmental | | [ ] |
| TP-SYS-001 Comprehensive Functional | | [ ] |

## 6. Test Facilities

### 6.1 Vibration Facility

| Item | Requirement | Status |
|------|-------------|--------|
| Shaker capacity | >50 lbs | [ ] Verified |
| Frequency range | 20-2000 Hz | [ ] Verified |
| Control system | Calibrated | [ ] Current |
| Fixture | Flight-like | [ ] Available |
| Schedule | Reserved | [ ] Confirmed |

### 6.2 Thermal Vacuum Facility

| Item | Requirement | Status |
|------|-------------|--------|
| Temperature range | -40 to +80°C | [ ] Verified |
| Vacuum level | <1×10⁻⁵ Torr | [ ] Verified |
| Feedthroughs | Power, data, RF | [ ] Available |
| Shroud control | ±2°C | [ ] Calibrated |
| Schedule | Reserved | [ ] Confirmed |

### 6.3 EMC Facility

| Item | Requirement | Status |
|------|-------------|--------|
| Shielding | >80 dB | [ ] Verified |
| Frequency range | 10 kHz - 18 GHz | [ ] Verified |
| Antennas | VHF/UHF | [ ] Available |
| Receivers | Calibrated | [ ] Current |
| Schedule | Reserved | [ ] Confirmed |

## 7. Ground Support Equipment

### 7.1 GSE List

| Equipment | Purpose | Cal Due | Status |
|-----------|---------|---------|--------|
| Power Supply | Simulate EPS | | [ ] Current |
| Accelerometers | Vibe response | | [ ] Current |
| Thermocouples | Temp measurement | | [ ] Current |
| Spectrum Analyzer | RF verification | | [ ] Current |
| Data Acquisition | Telemetry capture | | [ ] Ready |

### 7.2 Test Cables

| Cable | Description | Status |
|-------|-------------|--------|
| CAB-001 | Power umbilical | [ ] Verified |
| CAB-002 | Data umbilical | [ ] Verified |
| CAB-003 | RF test cable | [ ] Verified |
| CAB-004 | Thermocouple harness | [ ] Verified |

## 8. Personnel

### 8.1 Test Team

| Role | Name | Qualified | Available |
|------|------|-----------|-----------|
| Test Director | | [ ] | [ ] |
| Test Conductor | | [ ] | [ ] |
| Vibe Operator | | [ ] | [ ] |
| TVAC Operator | | [ ] | [ ] |
| EMC Operator | | [ ] | [ ] |
| QA | | [ ] | [ ] |

### 8.2 Training Status

| Training | Required For | Complete |
|----------|--------------|----------|
| ESD Awareness | All | [ ] |
| Facility Safety | All | [ ] |
| Vibe Operations | Vibe team | [ ] |
| TVAC Operations | TVAC team | [ ] |
| EMC Operations | EMC team | [ ] |

## 9. Risk Assessment

### 9.1 Environmental Test Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Hardware damage | Low | High | Test levels verified |
| Schedule slip | Medium | Medium | Buffer in schedule |
| Facility unavailable | Low | High | Backup facility ID'd |
| Test failure | Medium | Medium | Retest plan ready |

### 9.2 Test Constraints

| Constraint | Impact | Mitigation |
|------------|--------|------------|
| Battery state | TVAC temps | SOC monitoring |
| RF interference | EMC | Shielded cables |
| Handling | All | ESD procedures |

## 10. Success Criteria

### 10.1 Environmental Test Success

| Test | Success Criteria |
|------|------------------|
| Vibration | Survive levels, no damage; first-mode shift < 10 %, amplitude shift < 30 % (ETP §3.2) |
| TVAC | Function at temps, no failures |
| EMC | ETP §3.6 criteria (bus ripple, receiver-input emissions, self-compatibility, spurious ≥ 46 dBc and third harmonic ≥ 70 dBc) |
| Post-Functional | Match pre-test baseline |

### 10.2 Comparison Criteria

| Parameter | Acceptance |
|-----------|------------|
| Functional tests | All pass |
| Power consumption | Within 10% of baseline |
| RF performance | Within 1 dB of baseline |
| No physical damage | Visual inspection pass |

## 11. PER Checklist

### 11.1 Hardware Readiness

- [ ] Test article assembled
- [ ] Configuration documented
- [ ] Pre-test functional complete
- [ ] Baseline measurements recorded
- [ ] Handling procedures defined

### 11.2 Test Readiness

- [ ] Procedures approved
- [ ] Facilities reserved
- [ ] GSE calibrated
- [ ] Team trained
- [ ] Schedule confirmed

### 11.3 Documentation

- [ ] Test plan approved
- [ ] Procedures signed
- [ ] Risk assessment complete
- [ ] Anomaly forms ready
- [ ] Data sheets prepared

## 12. PER Board Decision

```
PRE-ENVIRONMENTAL REVIEW DECISION

Date: ___________
Test Article: SMART-QSO Flight Unit

The Pre-Environmental Review Board has reviewed
readiness for environmental qualification testing:

[ ] GO FOR ENVIRONMENTAL TEST
    - All readiness criteria met
    - Proceed with test campaign

[ ] GO WITH CONDITIONS
    - Conditions: ___________
    - Resolve before: ___________

[ ] NO-GO
    - Rationale: ___________
    - Required actions: ___________

Signatures:

Test Director: __________ Date: _____
Project Manager: ________ Date: _____
QA: ___________________ Date: _____
```

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
| 1.1 | 2026-10-16 | SMART-QSO Team | §3.3 as-designed mass from MASS_BUDGET v2.0 (was 1.2 kg; TB-06); §5.2 random vibration at the interim TP-ENV-002 level (was 14.1 Grms; LA-05) |
| 1.2 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): banner (template, not held; milestone: protoflight campaign PF_ENV); test article parts to the baseline (STM32L4R5ZI OBC board, Jetson module 900-13767-0030-000, AntS; were the Apollo4 AMA4B2KK-KBR and a Jetson developer kit) and the build-to design numbers; as-designed mass CBE 1.52 / MEV 1.75 kg, CG and SAFE power 0.79 W (BASELINE §4, §5); beacon intervals per mode; test sequence, levels and procedures from ETP Rev E (8.18 Grms protoflight, 8 TVAC cycles at AFT ± 10 °C capped +60 °C, bake-out; were 5.57 Grms, 4 cycles +50/−20 °C); pre-filled versions removed |
