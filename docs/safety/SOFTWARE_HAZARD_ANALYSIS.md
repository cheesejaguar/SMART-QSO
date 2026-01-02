# Software Hazard Analysis
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-SHA-001
**Revision:** A
**Date:** 2026-01-02
**Classification:** Class D Mission
**Reference:** NASA-STD-8719.13

---

## 1. Purpose

This document identifies, analyzes, and tracks software hazards for the SMART-QSO CubeSat mission. It establishes the linkage between system hazards and software contributions, defines controls, and tracks verification status.

## 2. Scope

This analysis covers all flight software components:
- OBC Flight Software
- AI Payload Software
- Software interfaces to hardware subsystems

## 3. Hazard Analysis Methodology

### 3.1 Process Overview

1. Review System Hazard Analysis for software contributions
2. Analyze software architecture for additional hazards
3. Perform FMEA on each software module
4. Develop fault trees for critical hazards
5. Define controls and mitigations
6. Verify controls through test and analysis
7. Track hazards to closure

### 3.2 Severity/Likelihood Matrix

|                | Catastrophic (1) | Critical (2) | Major (3) | Minor (4) | Negligible (5) |
|----------------|-----------------|--------------|-----------|-----------|----------------|
| Frequent (A)   | 1A              | 2A           | 3A        | 4A        | 5A             |
| Probable (B)   | 1B              | 2B           | 3B        | 4B        | 5B             |
| Occasional (C) | 1C              | 2C           | 3C        | 4C        | 5C             |
| Remote (D)     | 1D              | 2D           | 3D        | 4D        | 5D             |
| Improbable (E) | 1E              | 2E           | 3E        | 4E        | 5E             |

**Risk Acceptance:**
- High Risk (Red): 1A-1C, 2A-2B - Unacceptable, require elimination or control
- Medium Risk (Yellow): 1D-1E, 2C-2D, 3A-3C - Require controls and monitoring
- Low Risk (Green): All others - Acceptable with standard controls

## 4. System Hazard Summary

### 4.1 System Hazards with Software Contribution

| Hazard ID | System Hazard | Software Contribution |
|-----------|---------------|----------------------|
| SH-001 | Battery thermal runaway | SW commands excessive charging, fails to monitor temp |
| SH-002 | Loss of attitude control | ADCS algorithm failure, sensor data corruption |
| SH-003 | Communication blackout | Beacon generation failure, RF control failure |
| SH-004 | Uncontrolled power consumption | Load control failure, power budgeting error |
| SH-005 | Premature battery depletion | Power mode logic failure, AI payload runaway |
| SH-006 | Deployment mechanism failure | Deployment timer error, switch detection failure |
| SH-007 | Orbital debris contribution | EOL passivation failure |
| SH-008 | RF interference | Transmit power exceeds limits, spurious emissions |

## 5. Software Hazard Analysis

### 5.1 SW-HAZ-001: EPS Control Failure

**Hazard Description:** EPS control software fails to manage power distribution correctly.

**Severity:** Critical (2)
**Likelihood:** Remote (D)
**Risk Level:** 2D (Medium)

**Causes:**
1. Power mode state machine deadlock
2. Load switch control logic error
3. Battery protection threshold error
4. Sensor reading corruption

**Effects:**
- Uncontrolled power drain
- Battery damage
- Loss of mission

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-001-1 | Hardware voltage/current limits on EPS | Analysis |
| C-001-2 | Watchdog forces safe mode on SW hang | Test |
| C-001-3 | State machine formal verification | Analysis |
| C-001-4 | Sensor value range validation | Test |
| C-001-5 | Default to SAFE mode on invalid state | Test |

**Verification Status:** Partial - Tests in progress
**Closure Status:** Open

---

### 5.2 SW-HAZ-002: ADCS Algorithm Failure

**Hazard Description:** ADCS control algorithm produces incorrect actuator commands.

**Severity:** Major (3)
**Likelihood:** Occasional (C)
**Risk Level:** 3C (Medium)

**Causes:**
1. B-dot gain too high causing oscillation
2. Sun sensor misread causing wrong pointing
3. Magnetometer calibration error
4. Numerical overflow in calculations

**Effects:**
- Loss of attitude stability
- Reduced communication link margin
- Increased power consumption (solar panel misalignment)

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-002-1 | Actuator output limiting (saturation) | Test |
| C-002-2 | Angular rate limiting | Test |
| C-002-3 | Fixed-point arithmetic with overflow checks | Analysis |
| C-002-4 | Sensor validation before use | Test |
| C-002-5 | Fallback to passive magnetic stabilization | Test |

**Verification Status:** Partial
**Closure Status:** Open

---

### 5.3 SW-HAZ-003: Beacon Generation Failure

**Hazard Description:** Beacon generation software fails to produce valid AX.25 frames.

**Severity:** Major (3)
**Likelihood:** Remote (D)
**Risk Level:** 3D (Low)

**Causes:**
1. AX.25 frame formatting error
2. CRC calculation error
3. Buffer overflow in text generation
4. Timing error (beacon too frequent/infrequent)

**Effects:**
- Loss of telemetry downlink
- Ground station cannot decode beacons
- Violation of amateur radio regulations

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-003-1 | Template fallback if AI text invalid | Test |
| C-003-2 | AX.25 frame validation before TX | Test |
| C-003-3 | Fixed beacon interval bounds | Test |
| C-003-4 | CRC verification test suite | Test |
| C-003-5 | Text length and ASCII validation | Test |

**Verification Status:** Complete
**Closure Status:** Closed

---

### 5.4 SW-HAZ-004: Watchdog Failure

**Hazard Description:** Watchdog timer fails to reset system on software hang.

**Severity:** Critical (2)
**Likelihood:** Remote (D)
**Risk Level:** 2D (Medium)

**Causes:**
1. Watchdog kick in interrupt that doesn't detect main loop hang
2. Watchdog disabled during debugging and not re-enabled
3. Hardware watchdog malfunction

**Effects:**
- Software hung indefinitely
- No beacon transmission
- Loss of mission

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-004-1 | Main loop must kick watchdog (not ISR) | Inspection |
| C-004-2 | Watchdog enable verified in startup | Test |
| C-004-3 | Watchdog test mode in ground testing | Test |
| C-004-4 | Multi-task watchdog checking | Test |

**Verification Status:** Partial
**Closure Status:** Open

---

### 5.5 SW-HAZ-005: Data Persistence Corruption

**Hazard Description:** Mission data corruption causes loss of state across resets.

**Severity:** Minor (4)
**Likelihood:** Occasional (C)
**Risk Level:** 4C (Low)

**Causes:**
1. Write interrupted by reset
2. Flash/FRAM bit flip (radiation)
3. CRC calculation error
4. Buffer overrun during serialization

**Effects:**
- Loss of mission statistics
- Reset counter errors
- Configuration data loss

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-005-1 | CRC32 protection on all persisted data | Test |
| C-005-2 | Triple redundant storage with voting | Test |
| C-005-3 | Default values on CRC failure | Test |
| C-005-4 | Atomic write operations | Analysis |

**Verification Status:** Complete
**Closure Status:** Closed

---

### 5.6 SW-HAZ-006: Jetson Power Management Failure

**Hazard Description:** Jetson payload consumes excessive power or fails to shutdown.

**Severity:** Major (3)
**Likelihood:** Occasional (C)
**Risk Level:** 3C (Medium)

**Causes:**
1. Power gating signal failure
2. Jetson software hang consuming max power
3. OBC-Jetson communication failure
4. Thermal throttling not working

**Effects:**
- Battery depletion
- Thermal damage
- Loss of OBC due to power starvation

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-006-1 | Hardware power switch controlled by OBC | Test |
| C-006-2 | Jetson current monitoring by EPS | Test |
| C-006-3 | Automatic shutdown on over-current | Test |
| C-006-4 | Timeout on Jetson operations | Test |
| C-006-5 | OBC continues operation without Jetson | Test |

**Verification Status:** Partial
**Closure Status:** Open

---

### 5.7 SW-HAZ-007: Deployment Timer Error

**Hazard Description:** Antenna/solar panel deployment occurs at wrong time.

**Severity:** Critical (2)
**Likelihood:** Improbable (E)
**Risk Level:** 2E (Low)

**Causes:**
1. Timer calculation error
2. RTC failure
3. Premature deployment before separation
4. Deployment failure after timeout

**Effects:**
- Violation of launch provider requirements
- Damage to deployer or other satellites
- Antenna/solar panel stuck

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-007-1 | Hardware deployment inhibit (RBF pin) | Inspection |
| C-007-2 | Deployment switch required for arming | Test |
| C-007-3 | 30-minute minimum timer (configurable) | Test |
| C-007-4 | Multiple deployment attempts with backoff | Test |
| C-007-5 | Deployment status telemetry | Test |

**Verification Status:** Partial
**Closure Status:** Open

---

### 5.8 SW-HAZ-008: Safe Mode Entry Failure

**Hazard Description:** System fails to enter safe mode when required.

**Severity:** Critical (2)
**Likelihood:** Remote (D)
**Risk Level:** 2D (Medium)

**Causes:**
1. Safe mode transition blocked by deadlock
2. Safe mode configuration invalid
3. Fault detection not triggering
4. Safe mode state not preserved across reset

**Effects:**
- Continued operation in unsafe state
- Battery depletion
- Thermal damage

**Controls:**
| Control ID | Control Description | Verification |
|------------|---------------------|-------------|
| C-008-1 | Watchdog forces safe mode on timeout | Test |
| C-008-2 | Safe mode is default power-on state | Test |
| C-008-3 | Hardware-level safe mode (beacon only) | Test |
| C-008-4 | Multiple fault triggers for safe mode | Test |

**Verification Status:** Complete
**Closure Status:** Closed

---

## 6. Hazard Summary Table

| Hazard ID | Description | Severity | Likelihood | Risk | Status |
|-----------|-------------|----------|------------|------|--------|
| SW-HAZ-001 | EPS Control Failure | Critical | Remote | Medium | Open |
| SW-HAZ-002 | ADCS Algorithm Failure | Major | Occasional | Medium | Open |
| SW-HAZ-003 | Beacon Generation Failure | Major | Remote | Low | Closed |
| SW-HAZ-004 | Watchdog Failure | Critical | Remote | Medium | Open |
| SW-HAZ-005 | Data Persistence Corruption | Minor | Occasional | Low | Closed |
| SW-HAZ-006 | Jetson Power Management | Major | Occasional | Medium | Open |
| SW-HAZ-007 | Deployment Timer Error | Critical | Improbable | Low | Open |
| SW-HAZ-008 | Safe Mode Entry Failure | Critical | Remote | Medium | Closed |

## 7. Hazard Closure Criteria

A hazard is considered closed when:
1. All controls are implemented
2. All controls are verified by test or analysis
3. Residual risk is acceptable per risk matrix
4. Safety review has approved closure
5. Documentation is complete

## 8. Open Actions

| Action ID | Description | Owner | Due Date | Status |
|-----------|-------------|-------|----------|--------|
| A-001 | Complete EPS control fault injection tests | SW Lead | TBD | Open |
| A-002 | Complete ADCS algorithm verification | SW Lead | TBD | Open |
| A-003 | Verify watchdog operation on flight hardware | SW Lead | TBD | Open |
| A-004 | Complete Jetson power management tests | Payload Lead | TBD | Open |
| A-005 | Verify deployment timer on flight hardware | SW Lead | TBD | Open |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |

---

*Prepared in accordance with NASA-STD-8719.13*
