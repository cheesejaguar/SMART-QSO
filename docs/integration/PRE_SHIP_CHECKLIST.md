# Pre-Ship Review Checklist
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-INT-003
**Revision:** B
**Date:** 2026-10-27

> **Template for a future review.** No hardware exists and no pre-ship review has been held; the review is planned
> for 2028-05-19 (`docs/verification/VI_REGISTER.md`; `mission/SCHEDULE.md`). Every check box, date, initial and
> signature is blank until then. Values follow `docs/BASELINE_PARAMETERS.md` Rev E (amended).

---

## 1. Purpose

This checklist ensures the SMART-QSO CubeSat is ready for shipment to the integrator (Exolaunch on the baseline path, BASELINE §3), verifying all hardware, software, documentation, and test requirements are complete.

## 2. Checklist Summary

| Category | Items | Complete |
|----------|-------|----------|
| Hardware | 25 | ___/25 |
| Software | 12 | ___/12 |
| Testing | 18 | ___/18 |
| Documentation | 20 | ___/20 |
| Shipping | 10 | ___/10 |
| **Total** | **85** | **___/85** |

---

## 3. Hardware Verification

### 3.1 Structure and Mechanical

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 1 | All fasteners installed and torqued | ☐ | | |
| 2 | Witness marks applied to all fasteners | ☐ | | |
| 3 | No loose hardware or FOD | ☐ | | |
| 4 | Mass measured and within spec (≤ 2.0 kg, SYS-002; EXOpod Nova 1U limit 2.5 kg) | ☐ | | |
| 5 | CG measured and within spec (±2 cm) | ☐ | | |
| 6 | Fit check in the integrator's test pod passed (TP-MECH-001) | ☐ | | |
| 6a | Moments of inertia measured, ballast trimmed (VI-MCAD-03; Iyy ≥ 1.03 × max) | ☐ | | |
| 7 | Rail surfaces clean and smooth | ☐ | | |

### 3.2 Deployment Systems

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 8 | Antennas (ISIS AntS) and both solar wings stowed and restrained | ☐ | | |
| 9 | Burn-resistor continuity (four resistors) and stowed-state telemetry recorded | ☐ | | |
| 10 | All three deployment switches (DS1–DS3) actuate properly | ☐ | | |
| 11 | RBF pins installed and accessible | ☐ | | |
| 12 | Separation springs installed (if applicable) | ☐ | | |

### 3.3 Electrical

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 13 | All connectors mated and secured | ☐ | | |
| 14 | Harness routing verified | ☐ | | |
| 15 | No exposed conductors | ☐ | | |
| 16 | Battery at the shipping SOC agreed with the integrator (the final 60 % ± 5 % charge is made at the integrator about 47 days before launch, BASELINE §4) | ☐ | | |
| 17 | Battery voltage recorded: ______V | ☐ | | |
| 18 | Solar cells inspected (no damage) | ☐ | | |

### 3.4 RF System

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 19 | Antenna connections verified | ☐ | | |
| 20 | RF cables properly routed | ☐ | | |
| 21 | Transmitter disabled (RBF) | ☐ | | |

### 3.5 Thermal/Contamination

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 22 | Thermal interfaces properly installed | ☐ | | |
| 23 | No contamination visible | ☐ | | |
| 24 | Conformal coating complete | ☐ | | |
| 25 | Outgassing complete (if applicable) | ☐ | | |

---

## 4. Software Verification

### 4.1 Flight Software

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 1 | Flight software version: ________ | ☐ | | |
| 2 | Software CRC verified | ☐ | | |
| 3 | Backup image loaded | ☐ | | |
| 4 | Boot sequence verified | ☐ | | |
| 5 | Configuration parameters set | ☐ | | |

### 4.2 Mission Configuration

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 6 | Callsign configured: ________ | ☐ | | |
| 7 | Beacon interval set: ______ sec | ☐ | | |
| 8 | ADCS mode: Detumble on deploy | ☐ | | |
| 9 | Safe mode thresholds set | ☐ | | |
| 10 | Ground station parameters loaded | ☐ | | |

### 4.3 Inhibits

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 11 | Hardware deployment inhibits (RBF, SYS-225) installed: software alone does not inhibit burns on a powered bench unit | ☐ | | |
| 12 | `ARM_FLIGHT` sent through the umbilical as the last powered action (RF silence and deployment ARMED; TP-FAT-002 §6.7); no power-up of 25 min or more since | ☐ | | |
| 12a | Deployment switches released read released (`software/rust/sqso-ffi/src/separation_switch.rs` polarity, TBR) | ☐ | | |

---

## 5. Testing Verification

### 5.1 Unit/Subsystem Tests

| # | Test | Passed | Date | Report # |
|---|------|--------|------|----------|
| 1 | TP-EPS-001 EPS Functional | ☐ | | |
| 2 | TP-RF-001 RF Functional | ☐ | | |
| 3 | TP-OBC-001 OBC Functional | ☐ | | |
| 4 | TP-ADCS-001 ADCS Sensor Test | ☐ | | |

### 5.2 System Tests

| # | Test | Passed | Date | Report # |
|---|------|--------|------|----------|
| 5 | TP-SYS-001 Comprehensive Functional | ☐ | | |
| 6 | TP-SYS-002 Day-in-the-Life | ☐ | | |

### 5.3 Environmental Tests

| # | Test | Passed | Date | Report # |
|---|------|--------|------|----------|
| 7 | TP-ENV-002 Vibration | ☐ | | |
| 8 | TP-ENV-003 Thermal Vacuum | ☐ | | |
| 9 | EMC/EMI Test | ☐ | | |
| 10 | Deployment test, run before the protoflight vibration (TP-ENV-005), and post-environment continuity checks (CDS §3.5.2) | ☐ | | |

### 5.4 Final Verification

| # | Test | Passed | Date | Report # |
|---|------|--------|------|----------|
| 11 | Post-environmental functional | ☐ | | |
| 12 | Final mass measurement | ☐ | | |
| 13 | Final CG measurement | ☐ | | |
| 14 | Final fit check | ☐ | | |
| 15 | RF functional (beacon received) | ☐ | | |
| 16 | End-to-end comm test | ☐ | | |
| 17 | Battery capacity verified | ☐ | | |
| 18 | All anomalies closed | ☐ | | |

---

## 6. Documentation Verification

### 6.1 Design Documentation

| # | Document | Complete | Rev |
|---|----------|----------|-----|
| 1 | System Design Document | ☐ | |
| 2 | Software Design Document | ☐ | |
| 3 | Interface Control Documents | ☐ | |
| 4 | Verification Matrix | ☐ | |

### 6.2 Test Documentation

| # | Document | Complete | Rev |
|---|----------|----------|-----|
| 5 | Test Procedures | ☐ | |
| 6 | Test Reports | ☐ | |
| 7 | Anomaly Reports (all closed) | ☐ | |
| 8 | Non-conformance Reports (dispositioned) | ☐ | |

### 6.3 Safety Documentation

| # | Document | Complete | Rev |
|---|----------|----------|-----|
| 9 | Safety Plan | ☐ | |
| 10 | Hazard Analysis | ☐ | |
| 11 | FMEA | ☐ | |

### 6.4 Operations Documentation

| # | Document | Complete | Rev |
|---|----------|----------|-----|
| 12 | Mission Operations Plan | ☐ | |
| 13 | Command/Telemetry Database | ☐ | |
| 14 | Ground Station Procedures | ☐ | |
| 15 | Launch Site Operations Plan | ☐ | |

### 6.5 Compliance Documentation

| # | Document | Complete | Rev |
|---|----------|----------|-----|
| 16 | CDS Compliance Matrix | ☐ | |
| 17 | Orbital Debris Assessment | ☐ | |
| 18 | Frequency Coordination | ☐ | |
| 19 | Export Control Assessment | ☐ | |
| 20 | As-built Configuration | ☐ | |

---

## 7. Shipping Preparation

### 7.1 Spacecraft Configuration

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 1 | Spacecraft powered off | ☐ | | |
| 2 | Battery at the agreed shipping SOC (BASELINE §4) | ☐ | | |
| 3 | RBF pins installed | ☐ | | |
| 4 | Antennas stowed | ☐ | | |
| 5 | Deployment restraints in place | ☐ | | |

### 7.2 Packaging

| # | Item | Verified | Date | Initials |
|---|------|----------|------|----------|
| 6 | ESD bag | ☐ | | |
| 7 | Foam cushioning | ☐ | | |
| 8 | Desiccant packs | ☐ | | |
| 9 | Shock/tilt indicators | ☐ | | |
| 10 | Shipping container sealed | ☐ | | |

---

## 8. Approvals

### 8.1 Technical Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Systems Engineer | | | |
| Software Lead | | | |
| Integration Lead | | | |
| Quality Assurance | | | |

### 8.2 Management Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Project Manager | | | |
| Mission Manager | | | |

---

## 9. Ship Authorization

**Spacecraft Serial Number:** ________________

**Ship To:** ________________________________

**Carrier:** ________________________________

**Tracking Number:** ________________________

**Ship Date:** ______________________________

**Authorized By:** __________________________

**Signature:** ______________________________

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| A, note | 2026-10-24 | Item 12a names the Rust file that replaced `separation_switch.c` | SMART-QSO Team |
| B | 2026-10-27 | Documentation audit (post-CDR 2026): template banner (review planned 2028-05-19, none held); shipping SOC per BASELINE §4 (was 30 %; the 60 % ± 5 % charge is at the integrator); deployment test before vibration with continuity checks after (was post-environment); three switches, AntS, wings, inertia; Nova limit | Documentation audit |
