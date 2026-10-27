# Software Hazard Analysis
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-SHA-001
**Revision:** G
**Date:** 2026-10-27
**Classification:** Class D Mission
**Reference:** NASA-STD-8739.8B (replaced NASA-STD-8719.13, cancelled 2020)

---

## 1. Purpose

This document identifies, analyzes, and tracks software hazards for the SMART-QSO CubeSat mission. It establishes the linkage between system hazards and software contributions, defines controls, and tracks verification status.

## 2. Scope

This analysis covers all flight software components:
- OBC Flight Software
- AI Payload Software
- Software interfaces to hardware subsystems
- The firmware of the RF board's TNC (STM32L431) and of the EPS controller (MSP430FR2355), both Class C software items
  written by the project (SOFTWARE_DEVELOPMENT_PLAN §3.4), since Revision G (§9.8)

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
| SW-HAZ-003 | Beacon Generation Failure | Major | Remote | Low | Open (reopened: closure needs cited verification per control) |
| SW-HAZ-004 | Watchdog Failure | Critical | Remote | Medium | Open |
| SW-HAZ-005 | Data Persistence Corruption | Minor | Occasional | Low | Open (reopened: closure needs cited verification per control) |
| SW-HAZ-006 | Jetson Power Management | Major | Occasional | Medium | Open |
| SW-HAZ-007 | Deployment Timer Error | Critical | Improbable | Low | Open |
| SW-HAZ-008 | Safe Mode Entry Failure | Critical | Remote | Medium | Open (reopened: closure needs cited verification per control) |

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

## 9. CDR Additions (2026-10-27)

Added by the CDR avionics and flight-software review. Severity and likelihood use §3.2. These close the open action of
SOFTWARE_DEVELOPMENT_PLAN §3.2 ("RF inhibit has no SW-HAZ entry of its own") and cover the functions added at CDR.

### 9.1 SW-HAZ-009: Unpermitted RF Emission by the Transponder

**Hazard:** the FM transponder radiates while transmission is not permitted (within the 45-minute post-deployment
silence, after TX-OFF, under the loss-of-contact inhibit, during the commissioning hold, or with the ground's
XPDR_DISABLE in force). Interference with the launch vehicle or other users; a breach of 47 CFR 97.207 and CDS 2.4.5.
**Severity:** Critical (2). **Likelihood:** Improbable (E) with the controls.

**Causes:** a software fault leaves the enable asserted; the main loop stalls with the enable high; a reset leaves the
pin high; a gate is evaluated on stale data.

| Control | Description | Verification |
|---------|-------------|--------------|
| C-009-1 | `XPDR_EN` is a heartbeat, never a level, with a lease of 40 ms; the radio board's 50 ms monostable drops the transponder within 90 ms of the last main-loop pass; a stalled loop, a halted MCU and a reset all stop it (SRS-F081, `firmware/src/heartbeat.rs`) | T: `test_xpdr_heartbeat` (6 tests); SIL radio model treats a stopped heartbeat as off within 50 ms; EM: scope test of a halted MCU (open) |
| C-009-2 | Every gate in `transponder.rs` (permitted, powered, allowed, ACTIVE, sunlit, SOC, PA temperature, duty) must hold on every pass; any failing one stops the heartbeat (SRS-F080) | T: Rust unit tests; SIL checks `xpdr-forbidden` (0 s) and `xpdr-gates` (0.01 %) |
| C-009-3 | Hardware: `RF_TX_EN` and the RF inhibit switches are in series with the transponder's PA supply (BASELINE §4) | I, T at the EM (RF reviewer) |

**Status:** open until C-009-1's EM test and C-009-3. `transponder.rs` is on the safety-critical list since
2026-10-27 (RID FSW-05, SDP §3.2, SRS-Q013) and passes the per-file gate: 98.8 % lines and 94.4 % conditions measured,
the rest three reviewed `COVERAGE-EXEMPT` lines (a window slot that always exists). Since Revision C the transponder's
`permitted` input also requires the loss-of-contact inhibit to be clear, so the REL-04 exception below never lets the
transponder radiate.

### 9.2 SW-HAZ-010: RF Inhibit Failure (Beacon and Telemetry Downlink)

**Hazard:** a beacon or housekeeping frame is keyed inside the post-deployment silence, after TX-OFF or under the
loss-of-contact inhibit. **Severity:** Critical (2). **Likelihood:** Improbable (E).

| Control | Description | Verification |
|---------|-------------|--------------|
| C-010-1 | TX-OFF latch persisted before it acts, honoured across resets (`rf_control`) | T: `test_rf_control`, `test_cmd_handler` |
| C-010-2 | 45-minute silence after separation, latched once served so that a stowed sense reading later cannot restart or skip it (SRS-F087, `rf_silence`) | T: `test_rf_silence`; SIL `rf-silence` 45.0 min |
| C-010-3 | The downlink gate re-checks every inhibit when a frame is handed to the TNC (`downlink`); the housekeeping frame goes through the same gate (SRS-F085) | T: `test_downlink`; SIL `forbidden-tx` 0 |
| C-010-4 | Loss-of-contact inhibit after N days (`contact_monitor`). **Exception (REL-04, Revision C):** with the command receiver known failed (a recovery cycle made, its health line in fault, the N days expired; SRS-F116), the inhibit no longer silences the beacon and housekeeping frames; the transponder stays off; TX-OFF, the RF silence and passivation still silence everything. Rationale and the regulatory question: SRS §3.12 (to be declared in the IARU request, item 7.9, and put to the FCC consultation, D3) | T: `test_contact_monitor`, `test_receiver`; SIL `contact`, `rx-failed-beacon` (contingency `rx_failed_day=20`) |

All four components are on the safety-critical list with 100 % line and condition coverage. **Status:** controls
verified on the host; open for the EM.

### 9.3 SW-HAZ-011: Commissioning Hold Lost or Bypassed

**Hazard:** after ARM_FLIGHT and before the authenticated COMMISSIONING_RELEASE, the payload or the transponder
switches on, or the mode reaches ACTIVE (LV-01). Possible effects: an RF emission the operators have not cleared,
a Jetson load on an uncharacterised power system. **Severity:** Major (3). **Likelihood:** Remote (D).

| Control | Description | Verification |
|---------|-------------|--------------|
| C-011-1 | The hold is persisted (two-slot record under CRC); a corrupt record holds; only the release clears it (SRS-F082) | T: `test_ops_state` (6 tests), Rust unit tests |
| C-011-2 | While held: the mode manager's guard refuses ACTIVE from every event; `PLD_ENABLE` is refused; the transponder's `allowed` input is false | T: `test_ops_state`; SIL commissioning: transponder and payload first on after the release (day 1.5) |
| C-011-3 | No magnetorquer output until 120 s after separation (SRS-F086) | SIL: first coil command at 121 s |

**Status:** verified on the host. `ops_state.rs` (both crates) is on the safety-critical list since 2026-10-27 (FSW-05) at 100 % lines and conditions. Since Revision C the operations record is also checked against its RAM copy every 60 s and merged toward held (SRS-F133), and the mode is held at IDLE when the wings are not deployed after their attempts (EPS-10, SRS-F128).

### 9.4 SW-HAZ-012: OBC Selector Extends a Gate

**Hazard:** the flight AI (the OBC selector of `docs/payload/OBC_SELECTOR_DESIGN.md`, decision D5) enables an
emission, a load or a mode that the fixed rules forbid, or composes a beacon with content that is not pre-approved.
**Severity:** Major (3). **Likelihood:** Improbable (E) by design.

| Control | Description | Verification |
|---------|-------------|--------------|
| C-012-1 | The selector's outputs only restrict the transponder gates and the mode manager; the combination is `rules AND selector` (SRS-F102, SRS-F104) | T: property test over all inputs and any valid model table (to be written with the selector) |
| C-012-2 | Templates only from the pre-approved set; every value from measurements; the content filter of BEACON_SPEC §6 on every beacon (SRS-F101) | T, I |
| C-012-3 | Abstention to the fixed rules on stale, missing or out-of-range features, or low confidence (SRS-F105) | T |

**Status:** design complete; not implemented (fabrication phase). Not safety-critical if C-012-1 is shown by the
property test, because no selector output can then do anything the rules forbid.

### 9.5 SW-HAZ-013: End-of-Life Passivation Commanded Wrongly or Not Completed (Revision C; order changed in D)

**Hazard:** the irreversible charge disconnect (SQSO-EPS-BRD-001 §7) fires when not intended (by a software fault, a
replayed or forged command, or one key compromised), ending the mission; or a commanded passivation leaves a
transmitter able to radiate or the charge path closed (the pre-space notification's end-of-life statement,
97.207(g)(1)(iii)). **Severity:** Critical (2) for the mission (no hazard to people: the fuse is on the pack's own
charge path). **Likelihood:** Improbable (E) with the controls.

| Control | Description | Verification |
|---------|-------------|--------------|
| C-013-1 | Two commands under two keys: PASSIVATE_ARM (operational key, `EOL!`) then PASSIVATE_EXECUTE (recovery key) within 600 s; the execute alone, a lapsed arm or a wrong code does nothing (SRS-F110) | T: `test_passivation`, Rust unit tests |
| C-013-2 | Two processors and two separate OBC outputs (Revision E, RID P3E-11): PASSIVATE_ARM checks EOL_STATE (not armed, fuse intact), writes the EPS controller's key (0x3C then 0xC3 to EOL_KEY within 1 s), raises EOL_ARM within the key's 10 s and confirms the latch; only then is PASSIVATE_EXECUTE accepted, and EOL_FIRE (a level held ≥ 60 s in sunlight) fires. Neither the OBC alone (no key) nor the controller alone (no EOL_ARM edge, no EOL_FIRE) can fire it; both lines low at reset and pulled down on the EPS board: no single stuck line, glitch, reset or software fault fires it (SRS-F157, SRS-F110, SRS-F111) | T: `test_passivation` (the key sequence, a refused key, an unconfirmed latch), Rust unit tests, the SIL's EPS-controller model; V-EPS-8 on the EM |
| C-013-3 | The passivation record persisted before the fire; the fire while still transmitting, then the final report (three HK1/HK2 pairs over 3 min with EOL_STATE, the fuse state and the attempts), then TX-OFF and the passivation inhibit, **whether the fire was confirmed or not** (Revision D, RID REG-12: TX-OFF used to come first, so the ground could not see the result); the inhibit is never lifted; after a reset the record or the EPS EOL_STATE restores it, a fire resumes within its 3 attempts and an unfinished report is sent again before TX-OFF (SRS-F111, SRS-F112, SRS-F155) | T: `test_passivation` (the report frames and TX-OFF at 180 s, the unconfirmed fire), the Rust unit tests, both crates' `passivation.rs` at 100 % line and condition coverage; SIL end-of-life plan |
| C-013-4 | The emissions cease within 180 s of the end of the fire, whatever the fire, the EPS or the record does: a line that cannot be driven, a record that cannot be written or an EPS that never confirms all lead to the report and then TX-OFF (Revision D) | T: `test_passivation`, `sqso-ffi` unit test `hal_failures_are_reported` |

**Status:** verified on the host; the pins are allocated (EOL_ARM PF4, EOL_FIRE PG1, SQSO-ICX-001 B §6) and driven by
the image; EM test open. Since Revision D the passivation (`sqso-fsw/src/passivation.rs`, `sqso-ffi/src/passivation.rs`)
is on the safety-critical coverage list (`software/flight/scripts/safety_coverage.py`): with TX-OFF after the final
report, the sequence is itself an RF transmit inhibit (SH-008), the same class as `rf_control`. For the PI's
concurrence with the SDP §3.2 list.

### 9.6 Changes from the Board FMEA (Revision E, SQSO-FMEA-003)

| Item | Hazard affected | Change | Verification |
|------|-----------------|--------|--------------|
| SMA-B02 | SW-HAZ-007 (deployment timer), SW-HAZ-010 (RF silence) | DEPLOY_SENSE read as the hardware drives it (high = released, no internal pull; the software had a pull-up and high = stowed, which would have restarted the RF silence and the deployment wait for ever); separation latched after 10 s released, after which the line restarts neither timer (an open line cannot hold them); SOLAR_SENSE likewise (low = both wings deployed, no pull: a pull-down would have read deployed with the wings stowed); no internal pull on any sense input (SRS-F159) | T: `test_rf_silence`, `test_deployment`, `test_hal_pins` (the simulated HAL lets a configured pull win over the weak sources, as on the board); V-BFM-07 on the flatsat |
| SMA-B10 | SW-HAZ-005 (persistence), SW-HAZ-010 | A dead FRAM: every record mirrored to the second FRAM (U36) and read from it when the first fails; with both unreadable the safe defaults stay (TX-OFF, hold), the FRAM is declared failed after three checks, and only an authenticated FRAM_BYPASS makes the CRC-protected RAM copies authoritative (an upset of a RAM copy still settles toward TX-OFF; TX-OFF by command latches at once; a reset returns the defaults) (SRS-F160) | T: `test_fram_health`, the Rust unit tests of `rf_control`, `ops_state`, `fram_health`, `nvm` |
| SMA-B11 | SW-HAZ-010 (REL-04 exception) | A carrier detect stuck for more than 1 h is not evidence of a working receiver; RX_FAULT's power-on self-test; the TNC STATUS loop query, unanswered for 24 h with the radio on, counts as the receiver's fault (SRS-F115, SRS-F116) | T: `test_receiver`, Rust unit tests |
| SMA-B19 | SW-HAZ-010 (downlink held for temperature) | A failed TX_TEMP no longer holds the downlink for ever, nor hides a hot PA: an implausible reading (open line, out of range, too fast a rise, far from the board) is a sensor fault and the hold uses the board temperature plus 15 °C (SRS-F161) | T: `test_downlink`, Rust unit tests |
| SMA-B09 | SW-HAZ-004 (watchdog) | The OBC writes the EPS controller's heartbeat every 30 s; the controller power-cycles the OBC after 600 s without one (at most 3 times a day); the IWDG timeout is 25.6 s (24.1–27.8 s), below the external supervisor's 29.3 s minimum (SRS-F162) | T: `sqso-ffi` unit test `heartbeat_ants_power_and_trackers`; flatsat with the OBC halted |
| THM-03 | SW-HAZ-008 (safe mode) | With the wings stowed and the battery's orbit minimum below +3 °C the mode is held at SAFE so the heater gets the whole surplus (SRS-F163) | T: Rust unit tests of `cold_hold` |
| SMA-B20 (Revision F) | SW-HAZ-013 (passivation not completed) | The fire waits for the EPS board at 0 °C (F1's rating), but the condition cannot block the passivation: a cold board is waited for at most 6 h of each 24 h attempt, an unreadable temperature not at all, then the fire goes ahead in sunlight; the reason is reported in HK1 `PS` and the final report (SRS-F111) | T: `test_passivation` (cold 6 h then fires, warm at once, unreadable fires), Rust unit tests of `passivation.rs` (safety-critical coverage) |
| C-18 (Revision F) | SW-HAZ-007 (deployment) | The AntS is deployed only at ≥ −17 °C by its own temperature; colder or unknown defers by 50-minute slots without spending an attempt, and the deferral cannot block the antenna for good: after 30 deferrals (about 25 h) it deploys anyway, and DEPLOY_RETRY's force flag deploys at once (SRS-F165); the 30-minute wait and the wings-first order unchanged | T: `test_deployment` (`test_ants_temperature_gate`), Rust unit tests of `deployment.rs` (safety-critical coverage), the mission simulation's `ants-gate` check |
| R_FAULT 220 kΩ (Revision F) | SW-HAZ-002 (ADCS) | A sensor rail's turn-on takes up to 0.55 s: a readback low within 0.6 s of the enable is not a trip and no sensor is used before 0.6 s (SRS-F164) | T: Rust unit test `a_rail_is_used_only_after_its_turn_on` |

### 9.8 Board Firmware: the RF Board TNC and the EPS Controller (Revision G)

Added by the CDR round 3 software-systems work (finding P3A-03). Both items are Class C and not yet written; their
requirements are SQSO-SRS-TNC-001 (`docs/software/TNC_FIRMWARE_REQUIREMENTS.md`) and SQSO-SRS-EPSC-001
(`docs/software/EPS_CONTROLLER_FIRMWARE_REQUIREMENTS.md`), where each control below is a requirement flagged
safety-critical. **Determination (SWE-205):** both items contain safety-critical software, the requirements so
flagged (25 of the TNC's 67, 20 of the EPS controller's 62); the rest of each item is mission-critical. The hardware
controls listed first in each table do not depend on the firmware (the two specifications' §3 list them).

#### 9.8.1 SW-HAZ-014: RF Board TNC Firmware, Emission Outside Its Limits

**Hazard:** the TNC firmware causes an emission the OBC did not ask for or outside the transmitter's limits: a
continuous carrier on the beacon path, a transmission off the coordinated downlink frequency, or a 435 MHz emission
from the receiver IC. System hazard SH-008 (RF interference; 47 CFR 97.307, 97.207). **Severity:** Critical (2).
**Likelihood:** Improbable (E) with the controls.

**Causes the firmware can contribute to:**
1. TNC_PTT held high (a stuck modulator, a queue loop) while the main loop still refreshes the watchdog. The beacon
   path TNC_PTT bypasses the OBC heartbeat (EN_OK) and the 120 s key-down timer (SQSO-RF-BRD-001 §8.4): only RF_TX_EN
   bounds it in hardware.
2. A wrong or upset LMX2571 register image: the transmitter locks on another frequency. The board has no read-back of
   the synthesizer (MICROWIRE through U503 is write-only; MUXout is the lock detect).
3. The CC1200 put into transmit by a wrong strobe. It is on RADIO_3V3, which TX-OFF does not remove; its PA pin is
   unused (C208 not fitted), so the level would be leakage, not yet measured.
4. A corrupted KISS parameter (TXDELAY, TXTAIL) or an uplink frame taken as a KISS command, lengthening a key-down.
5. A hang with TNC_PTT or TONE_OK high.

| Control | Description | Verification |
|---------|-------------|--------------|
| C-014-1 | Hardware: no PA supply, no exciter supply and no synthesizer enable without PA_5V, which only RF_TX_EN (TX-OFF, RF silence, loss of contact) and the deployment switches give; two independent paths (SQSO-RF-BRD-001 §8.5); the PA enabled only with the lock detect (U733); deviation hard-limited by the PCM1808's full scale | EM: VI-RFB-08 (no emission with PA_5V removed, TNC_PTT forced high) |
| C-014-2 | Key-down limit: TNC_PTT released ≤ 5.0 s after it rises, by a hardware timer compare independent of the frame logic (TNC-F023); the legitimate maximum is 3.2 s | UT; EM: test build holding the modulator unkeys at 5.0 s |
| C-014-3 | Duty backstop: TNC_PTT ≤ 60 s in any 120 s (TNC-F024), above the OBC's maximum of 45 s, so it acts only on a fault | UT |
| C-014-4 | Keying only for an OBC frame, only with TX_PERMIT_HW high, frame bits only with the lock detect high (TNC-F020, TNC-F021, TNC-F022) | UT; EM |
| C-014-5 | The synthesizer image from a CRC-checked record, rewritten within 10 ms of the permit, every 10 s and before each key-up; no key with a bad CRC (TNC-F042); trims bounded to ±3 ppm and never changed in flight (TNC-F044, TNC-I008). **Residual:** an upset between a write and a key keys one transmission (≤ 5 s) off frequency | UT; EM: frequency at 1,000 key-ups; spectrum 144–148 MHz |
| C-014-6 | The CC1200 never given a transmit strobe; its PA at minimum (TNC-F041) | UT; IN; EM: leakage measured with a test build (recorded) |
| C-014-7 | Parameters clamped (TNC-I005); RF data never parsed as KISS commands (TNC-F013); unknown commands ignored (TNC-I008) | UT, fuzz |
| C-014-8 | Independent watchdog in hardware mode, 250 ms (≤ 0.35 s), refreshed from the main loop only; outputs low at and after a reset (TNC-F002, TNC-F004, TNC-F005); TNC_PTT and TONE_OK pulled down in reset (R408) | IN (option bytes); EM: halted loop |
| C-014-9 | **Recommended, not yet a requirement** (for the SRS owner): the OBC, which captures PTT_EXCITER on TIM8_CH2, drives RF_TX_EN low when PTT_EXCITER stays high for more than 6 s with no frame outstanding, so a TNC that holds TNC_PTT with its timer failed is cut by the OBC | — |

**Status:** requirements written (SQSO-SRS-TNC-001 A); firmware not written. Open until the EM tests and C-014-9.

#### 9.8.2 SW-HAZ-015: RF Board TNC Firmware, Telecommand Path Blocked or Receiver Evidence Falsified

**Hazard:** the TNC firmware keeps a frame from the OBC (TX_DISABLE included: the 97.207(b) cessation of emissions
by telecommand), or reports the receiver healthy when it is deaf, or failed when it works. A false fault lets the
REL-04 exception keep the beacon after N days of silence, which decision D3 meant to stop (C-010-4; SQSO-FMEA-003
BF-RF-32); a false health hides a deaf receiver from the recovery cycle. **Severity:** Critical (2).
**Likelihood:** Improbable (E).

**Causes:** frames filtered or dropped by content; a USART path dead while RX_ALIVE still pulses; a supervision
criterion wrongly computed (noise-floor table, thresholds); a hang.

| Control | Description | Verification |
|---------|-------------|--------------|
| C-015-1 | Hardware: RX_FAULT is a monostable on the RX_ALIVE pulses, so a hung, resetting or dead TNC reads as a fault without firmware help; RX_CD is the CC1200's carrier sense, independent of the firmware once configured (SQSO-RF-BRD-001 §8.6) | EM: VI-RFB-10 |
| C-015-2 | Every frame with a valid FCS passed to the OBC whatever its content; no key, no authentication, no command decoder in the TNC; the OBC re-checks the FCS and authenticates (TNC-F011, TNC-F012, TNC-I011; SQSO-ICD-013 §3, §8). The TNC cannot make a valid command | UT; IN; FS: TX_DISABLE over the air |
| C-015-3 | The STATUS loop query every 10 min, unanswered for 24 h with the radio on, counts as a receiver-path fault (TNC-I006; SRS-F116) | UT; EM: VI-RFB-10; FS |
| C-015-4 | The six criteria as specified, RX_ALIVE from the main loop only, none before 100 ms after a reset (TNC-F003, TNC-F043, TNC-F050 to TNC-F054, TNC-F056); a bad calibration table makes criterion 6 not evaluated rather than failed | UT; EM: VI-RFB-10 (injected faults, −40 … +70 °C, +6 dB interference) |
| C-015-5 | The OBC side: RX_FAULT power-on self-test, a carrier stuck for more than 1 h not taken as evidence, the 24 h recovery cycle, a valid frame clears known-failed (SRS-F115, SRS-F116, SRS-F152) | T: `test_receiver`; flatsat V-BFM-05 |

**Status:** requirements written; firmware not written.

#### 9.8.3 SW-HAZ-016: EPS Controller Firmware Weakens a Battery Hazard Control or Misreports the Pack

**Hazard:** the controller firmware removes one of the two signals the recharge-lockout clear needs, fails to set
the lockout when the AFE's own path has failed, sets it spuriously, changes the AFE's protection settings, or
reports a pack state that makes the OBC's software controls late: SOC too high (SAFE at 25 %, BSA H2 control 1),
the pack temperature wrong (the heater thermostat, H5 control 3 and H6 control 1), the Jetson current too low (the
payload guard). **Severity:** Critical (2) on orbit (SQSO-BSA-001; at launch the controller is unpowered behind
DS1–DS3). **Likelihood:** Improbable (E).

**Causes:** UNLOCK left high after a clear (ICD-001 1.6 lets it stay high until another value is written); LOCK_SET
missed, or raised on an implausible reading; a write command or key to the AFE; coulomb-count drift; NTC1 failed
and TBATT not handed over; an uncalibrated IMON channel.

| Control | Description | Verification |
|---------|-------------|--------------|
| C-016-1 | Hardware first: AFE OTP protections (COV, CUV, OCD, SCD, UTC/OTC/OTD), the BQ771806 + F3, the CV/CC loops, the NTC1 window with its hand-over, the 60 °C BOTHOFF, the lockout latch K1 set by the AFE's permanent fail without the firmware, the load shed, the 1.20 A eFuse (SQSO-EPS-BRD-001 §5, §6, §8) | EM: V-EPS-2, -7, -9, -12, -14 |
| C-016-2 | UNLOCK raised for 1.0 s per 0xC5 write, then lowered (EPSC-F062) | UT; EM: V-EPS-9 (LOCK_CLR alone after an unlock does nothing) |
| C-016-3 | LOCK_SET backup on the AFE's SUV permanent fail or 5 s of plausible readings below 2.00 V, once per event, and for no other reason (EPSC-F060, EPSC-F061) | UT; EM: V-EPS-9 |
| C-016-4 | The AFE accessed read-only; no unseal key in the image; its protection settings compared with the acceptance record every 10 min and a difference reported, never corrected by a write (EPSC-F063) | UT; IN; EM |
| C-016-5 | SOC within ±5 % after the first full charge, bounded by the cell voltage below 3.30 V, conservative after a lost state (EPSC-F040 to EPSC-F043); the OBC keeps its own INA228 gauge and analog VBATT/IBATT | UT; EM: pack cycling against a reference |
| C-016-6 | TBATT and TBATT2 within ±1.0 °C, TBATT from NTC2 while NTC1_BAD (EPSC-F011, EPSC-P002); IMON_JET within ±3 % ±20 mA (EPSC-F013) | EM: chamber; Jetson load simulator |
| C-016-7 | Watchdog ≤ 1.5 s, outputs at their pull-downs at and after a reset (EPSC-F001, EPSC-F003) | IN; EM |

**Status:** requirements written (SQSO-SRS-EPSC-001 A); firmware not written.

#### 9.8.4 SW-HAZ-017: EPS Controller Firmware, End-of-Life Key, OBC Bus or Supervisor Outputs Misused

**Hazard:** the controller firmware supplies the end-of-life key without the keyed sequence or holds it past 10 s
(the arm then needs only one OBC edge: SQSO-FMEA-003 fix (c)); misreports EOL_STATE, on which PASSIVATE_ARM and the
final report depend; holds I2C2, blocking the gauge, the lockout clear and EOL_KEY (BF-EPS-50, Category 3: no
passivation); or power-cycles the OBC spuriously or without limit. Related to SW-HAZ-013 (passivation) and SW-HAZ-004
(an OBC hang). **Severity:** Critical (2) for the mission. **Likelihood:** Improbable (E).

| Control | Description | Verification |
|---------|-------------|--------------|
| C-017-1 | Hardware: the arm latch needs the OBC's EOL_ARM edge AND EOL_KEY and is cleared at every power-up; only the OBC's EOL_FIRE (≥ 60 s) fires F1; each OBC power cycle lasts 1.95 s per rising edge of OBC_CYCLE (U82), whatever the line does (SQSO-EPS-BRD-001 §7, §12) | EM: V-EPS-8, V-EPS-16 |
| C-017-2 | EOL_KEY only after 0x3C then 0xC3 within 1 s with EOL_ARMED low and F1 intact; lowered at 10 s, on any other write, once the latch is set, and at any reset; ≤ 11.5 s even with a hang (EPSC-F070 to EPSC-F072) | UT; EM: 1,000 sequences, hang case |
| C-017-3 | EOL_STATE from samples ≤ 100 ms old; an ambiguous F1 reading is neither intact nor fired (EPSC-F015, EPSC-F073) | UT; EM |
| C-017-4 | Never a master on I2C2; SCL never held > 1 ms; the target reset within 25 ms of a stall, without changing an output (EPSC-I001, EPSC-I006) | UT; FS: IFT-EPS-006 |
| C-017-5 | OBC cycles only after 600–660 s without a heartbeat, not within 600 s of a start or a cycle, at most 3 in 24 h with the history in FRAM (EPSC-F081, EPSC-F082) | UT; EM: V-EPS-16 |

**Status:** requirements written; firmware not written. The AntS supply (EPSC-F090) and the tracker disables
(EPSC-F091, EPSC-F092) were reviewed and are not safety-critical: the AntS deploys only on the OBC's own commands
behind DS1–DS3, and a disabled tracker costs power, not safety.

### 9.7 Summary of the Additions

| Hazard ID | Description | Severity | Likelihood | Risk | Status |
|-----------|-------------|----------|------------|------|--------|
| SW-HAZ-009 | Unpermitted RF emission by the transponder | Critical | Improbable | Low | Open (EM tests) |
| SW-HAZ-010 | RF inhibit failure, beacon and downlink | Critical | Improbable | Low | Open (EM tests) |
| SW-HAZ-011 | Commissioning hold lost or bypassed | Major | Remote | Low | Verified on host |
| SW-HAZ-012 | OBC selector extends a gate | Major | Improbable | Low | Design only |
| SW-HAZ-013 | End-of-life passivation commanded wrongly or not completed | Critical (mission) | Improbable | Low | Open (EM test V-EPS-8) |
| SW-HAZ-014 | RF board TNC firmware: emission outside its limits | Critical | Improbable | Low | Open (requirements only; EM tests) |
| SW-HAZ-015 | RF board TNC firmware: telecommand path blocked or receiver evidence falsified | Critical | Improbable | Low | Open (requirements only; EM tests) |
| SW-HAZ-016 | EPS controller firmware weakens a battery hazard control or misreports the pack | Critical | Improbable | Low | Open (requirements only; EM tests) |
| SW-HAZ-017 | EPS controller firmware: end-of-life key, OBC bus or supervisor outputs misused | Critical (mission) | Improbable | Low | Open (requirements only; EM tests) |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | §9 new: SW-HAZ-009 (transponder emission, XPDR_EN heartbeat), SW-HAZ-010 (RF inhibits, the SDP's open action), SW-HAZ-011 (commissioning hold), SW-HAZ-012 (OBC selector); recommendation to add `transponder.rs` and `ops_state.rs` to the safety-critical list (RID FSW-05) | SMART-QSO Team (CDR avionics review) |
| C | 2026-10-27 | §9: C-009-1 lease 40 ms and drop within 90 ms; `transponder.rs` and `ops_state.rs` on the safety-critical list (FSW-05); C-010-4 the REL-04 exception (beacon kept with the receiver known failed); §9.5 new: SW-HAZ-013 end-of-life passivation (REG-01, EPS-06); the summary is §9.6 | SMART-QSO Team (CDR flight software, round 2) |
| D | 2026-10-27 | §9.5 SW-HAZ-013: the passivation order of RID REG-12 (the fire while transmitting, the final report, then TX-OFF, also after an unconfirmed fire), control C-013-4 (the emissions cease whatever fails), the pins allocated, `passivation.rs` (both crates) on the safety-critical list | SMART-QSO Team (CDR flight software, round 3) |
| E | 2026-10-27 | §9.5 C-013-2 the two-processor arm (RID P3E-11, SRS-F157); §9.6 new: the changes from the board FMEA (SMA-B02 sense polarity and separation latch, B10 FRAM mirror and degraded mode, B11 receiver evidence, B19 TX_TEMP plausibility, B09 EPS heartbeat and IWDG timeout) and THM-03 (the wings-stowed cold hold); the summary is §9.7 | SMART-QSO Team (CDR flight software, round 3, second part) |
| F | 2026-10-27 | §9.6: SMA-B20 the passivation's EPS-board condition bounded (6 h, unreadable not blocking); C-18 the AntS temperature gate with its cap and force flag; the sensor-rail turn-on (R_FAULT 220 kΩ); SMA-B09 row: the IWDG at 25.6 s before the supervisor's 29.3 s |
| G | 2026-10-27 | §2 scope and new §9.8: the board firmware items (finding P3A-03). SW-HAZ-014 (TNC emission outside its limits) and SW-HAZ-015 (TNC telecommand path and receiver evidence), SW-HAZ-016 (EPS controller and the battery hazard controls) and SW-HAZ-017 (EPS controller end-of-life key, bus and supervisor), each with causes, controls and verification from SQSO-SRS-TNC-001 A and SQSO-SRS-EPSC-001 A; the SWE-205 determination for both items; recommended control C-014-9 (the OBC watches PTT_EXCITER); §9.7 summary | SMART-QSO Team (CDR round 3, software systems) |

---

*Prepared in accordance with NASA-STD-8739.8B*
