# Software Failure Mode and Effects Analysis (FMEA)
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-FMEA-001
**Revision:** C
**Date:** 2026-10-27
**Reference:** NASA-STD-8739.8B (software safety analysis), MIL-STD-1629A

---

## 1. Purpose

This document provides a systematic analysis of potential failure modes in SMART-QSO flight software, their causes, effects, and detection/mitigation methods.

## 2. FMEA Methodology

### 2.1 Severity Rating

| Rating | Category | Definition |
|--------|----------|------------|
| 1 | Catastrophic | Loss of vehicle or loss of life |
| 2 | Critical | Loss of mission |
| 3 | Major | Significant mission degradation |
| 4 | Minor | Minor mission degradation |
| 5 | None | No effect |

### 2.2 Occurrence Rating

| Rating | Probability | Definition |
|--------|-------------|------------|
| 1 | Very High | > 0.1 per mission |
| 2 | High | 0.01 - 0.1 per mission |
| 3 | Moderate | 0.001 - 0.01 per mission |
| 4 | Low | 0.0001 - 0.001 per mission |
| 5 | Very Low | < 0.0001 per mission |

### 2.3 Detection Rating

| Rating | Detectability | Definition |
|--------|---------------|------------|
| 1 | Very Low | Failure not detectable |
| 2 | Low | Failure difficult to detect |
| 3 | Moderate | Failure detectable with effort |
| 4 | High | Failure easily detected |
| 5 | Very High | Failure automatically detected |

### 2.4 Risk Priority Number (RPN)

RPN = Severity x Occurrence x Detection

| RPN Range | Action Required |
|-----------|-----------------|
| 1-20 | No action required |
| 21-50 | Monitor, optional action |
| 51-100 | Action required |
| > 100 | Immediate action required |

---

## 3. EPS Control Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| EPS-FM-001 | Power mode stuck in SAFE | State machine logic error | Payload never enabled | Mission degraded | 3 | 4 | 4 | 48 | Telemetry monitoring | Ground command override |
| EPS-FM-002 | Power mode stuck in ACTIVE | State machine logic error | Battery depleted | Loss of mission | 2 | 4 | 4 | 32 | Battery voltage telemetry | Hardware low voltage cutoff |
| EPS-FM-003 | Load switch fails ON | GPIO control failure | Payload stuck on | Battery drain | 3 | 4 | 4 | 48 | Current telemetry | Hardware fuse protection |
| EPS-FM-004 | Load switch fails OFF | GPIO control failure | Subsystem offline | Mission degraded | 3 | 4 | 4 | 48 | Subsystem telemetry | Redundant control path |
| EPS-FM-005 | Incorrect SOC calculation | Algorithm error | Wrong power decisions | Battery damage | 3 | 4 | 3 | 36 | Ground analysis | Conservative margins |
| EPS-FM-006 | Battery temp ignored | Sensor read failure | Thermal damage | Loss of mission | 2 | 4 | 4 | 32 | Temperature telemetry | Hardware thermal cutoff |

---

## 4. Fault Management Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| FM-FM-001 | Fault not logged | Log buffer full | History lost | Debugging impaired | 4 | 3 | 3 | 36 | Log count telemetry | Circular buffer |
| FM-FM-002 | False fault trigger | Threshold too sensitive | Unnecessary safe mode | Mission interruption | 4 | 3 | 4 | 48 | Fault rate analysis | Debounce/hysteresis |
| FM-FM-003 | Fault missed | Threshold too loose | Damage not prevented | Hardware damage | 2 | 4 | 2 | 16 | Post-anomaly analysis | Conservative thresholds |
| FM-FM-004 | Safe mode not entered | Logic error | Continued unsafe operation | Loss of mission | 2 | 4 | 3 | 24 | Safe mode flag telemetry | Watchdog backup |
| FM-FM-005 | Recovery loop | Fault clears then recurs | System oscillating | Power wasted | 4 | 3 | 4 | 48 | Mode transition count | Recovery lockout |

---

## 5. ADCS Control Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| ADCS-FM-001 | Detumble timeout | High initial rates | Extended detumble | Delayed operations | 4 | 4 | 5 | 80 | Mode duration telemetry | Extended timeout allowance |
| ADCS-FM-002 | Wrong detumble direction | Magnetometer polarity error | Increased rotation | Extended detumble | 4 | 4 | 3 | 48 | Angular rate telemetry | Auto-detect and correct |
| ADCS-FM-003 | Sun pointing failure | Sun sensor failure | No sun pointing | Power reduced | 3 | 4 | 4 | 48 | Solar array current | Passive stabilization |
| ADCS-FM-004 | Actuator saturation | Excessive disturbance | No control authority | Attitude drift | 3 | 3 | 4 | 36 | Command telemetry | Wait for favorable conditions |
| ADCS-FM-005 | Control instability | Gain error | Oscillation | Increased power use | 4 | 4 | 5 | 80 | Angular rate telemetry | Automatic gain reduction |

---

## 6. Beacon Generation Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| BCN-FM-001 | No beacon transmitted | RF failure | No downlink | Loss of communication | 3 | 4 | 3 | 36 | Beacon count telemetry | Hardware beacon timer |
| BCN-FM-002 | Invalid AX.25 frame | Formatting error | Frame not decoded | No useful data | 4 | 4 | 4 | 64 | Ground decode rate | Format validation |
| BCN-FM-003 | Beacon too frequent | Timer error | FCC/IARU violation | Regulatory issue | 4 | 5 | 4 | 80 | Beacon interval telemetry | Hardware timer limit |
| BCN-FM-004 | Beacon too infrequent | Timer error | Poor data rate | Mission degraded | 4 | 4 | 4 | 64 | Beacon interval telemetry | Minimum interval check |
| BCN-FM-005 | Invalid AI text | Content filter failure | Inappropriate content | Regulatory/reputation | 4 | 4 | 3 | 48 | Ground review | Character whitelist |

---

## 7. UART Communication Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| UART-FM-001 | TX buffer overflow | Data rate too high | Data loss | Incomplete telemetry | 4 | 3 | 4 | 48 | Overflow counter | Flow control |
| UART-FM-002 | RX buffer overflow | Data not processed | Commands lost | Jetson data missed | 4 | 3 | 4 | 48 | Overflow counter | DMA/larger buffer |
| UART-FM-003 | Framing error | Noise/clock mismatch | Packet corruption | Retry required | 4 | 3 | 5 | 60 | Error counter | CRC detection |
| UART-FM-004 | Timeout | Jetson not responding | No AI text | Use template | 4 | 3 | 5 | 60 | Timeout counter | Automatic fallback |
| UART-FM-005 | Port initialization failure | Hardware error | No communication | Manual intervention | 3 | 5 | 5 | 75 | Init status check | Retry with delay |

---

## 8. Mission Data Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| MD-FM-001 | CRC mismatch on load | Data corruption | Default values used | Statistics lost | 4 | 3 | 5 | 60 | CRC check | Redundant storage |
| MD-FM-002 | Write failure | Flash wear | Data not persisted | Statistics lost | 4 | 4 | 5 | 80 | Write verification | Wear leveling |
| MD-FM-003 | Read timeout | Storage busy | Delayed startup | Increased boot time | 4 | 4 | 5 | 80 | Timeout detection | Default values |
| MD-FM-004 | Structure version mismatch | SW update | Data misinterpreted | Incorrect behavior | 3 | 4 | 4 | 48 | Version field | Migration logic |

---

## 9. Watchdog Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| WDT-FM-001 | Watchdog not started | Init failure | No protection | Potential hang | 2 | 5 | 4 | 40 | Status register | Startup verification |
| WDT-FM-002 | Watchdog kicked too frequently | Loop too fast | No reset on hang | Potential hang | 2 | 4 | 3 | 24 | Timing analysis | Minimum kick interval |
| WDT-FM-003 | Spurious reset | Noise/EMI | Unplanned reset | Data loss | 4 | 4 | 5 | 80 | Reset counter | Filtering, grounding |
| WDT-FM-004 | Reset not recovery | Hardware failure | System bricked | Loss of mission | 2 | 5 | 2 | 20 | Boot success flag | Golden image |

---

## 10. Sensor Management Module FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| SEN-FM-001 | Sensor offline | Hardware failure | No data | Degraded decisions | 4 | 3 | 5 | 60 | Sensor status flag | Default/backup values |
| SEN-FM-002 | Invalid reading | Out of range | Bad data used | Wrong decisions | 3 | 3 | 4 | 36 | Range check | Value rejection |
| SEN-FM-003 | Stale data | Read failure | Old data used | Wrong decisions | 4 | 3 | 4 | 48 | Timestamp check | Staleness flag |
| SEN-FM-004 | Conversion error | Formula wrong | Incorrect engineering units | Wrong display | 4 | 4 | 3 | 48 | Ground cross-check | Unit tests |

---

## 11. AI Payload Interface FMEA

| ID | Failure Mode | Cause | Local Effect | System Effect | S | O | D | RPN | Detection Method | Mitigation |
|----|--------------|-------|--------------|---------------|---|---|---|-----|------------------|------------|
| AI-FM-001 | Jetson boot failure | Hardware/SW error | No AI capability | Template beacons | 4 | 3 | 5 | 60 | Boot acknowledgment | Retry, fallback |
| AI-FM-002 | Inference timeout | Model too slow | No AI text | Template beacons | 4 | 3 | 5 | 60 | Timeout counter | Fixed deadline |
| AI-FM-003 | Power limit exceeded | Workload spike | Thermal throttle | Delayed response | 4 | 3 | 4 | 48 | Current monitor | DVFS limiting |
| AI-FM-004 | Communication corruption | UART error | Invalid text | Use template | 4 | 3 | 5 | 60 | CRC check | Retry/fallback |
| AI-FM-005 | Jetson hang | Software bug | No response | Power cycle | 4 | 3 | 4 | 48 | Heartbeat timeout | Power gate reset |

---

## 12. Summary Statistics

### 12.1 Failure Mode Count by Module

| Module | Total FMs | RPN > 50 | RPN > 100 |
|--------|-----------|----------|-----------|
| EPS Control | 6 | 0 | 0 |
| Fault Management | 5 | 0 | 0 |
| ADCS Control | 5 | 2 | 0 |
| Beacon Generation | 5 | 3 | 0 |
| UART Communication | 5 | 2 | 0 |
| Mission Data | 4 | 3 | 0 |
| Watchdog | 4 | 1 | 0 |
| Sensor Management | 4 | 1 | 0 |
| AI Payload | 5 | 2 | 0 |
| **Total** | **43** | **14** | **0** |

### 12.2 High RPN Items Requiring Action

| ID | Module | Failure Mode | RPN | Action |
|----|--------|--------------|-----|--------|
| ADCS-FM-001 | ADCS | Detumble timeout | 80 | Extended timeout verification |
| ADCS-FM-005 | ADCS | Control instability | 80 | Gain stability analysis |
| BCN-FM-003 | Beacon | Too frequent | 80 | Hardware timer verification |
| MD-FM-002 | Mission Data | Write failure | 80 | Flash wear analysis |
| MD-FM-003 | Mission Data | Read timeout | 80 | Timeout tuning |
| WDT-FM-003 | Watchdog | Spurious reset | 80 | EMC testing |

---

## 13. Conclusions and Recommendations

1. **No failure modes exceed RPN of 100** - No immediate critical actions required
2. **14 failure modes exceed RPN of 50** - Require monitoring and planned mitigation
3. **Primary risk areas:** ADCS stability, beacon timing, storage reliability
4. **Key mitigations:** Watchdog protection, graceful degradation, telemetry monitoring

---

## 14. CDR Update (2026-10-27)

### 14.1 The Rating Scales Are Inverted (RID FSW-08)

In §2 all three scales give the **low** number to the **bad** case: severity 1 is catastrophic, occurrence 1 is very
likely, detection 1 is undetectable. The product of §2.4 is then largest for the most benign modes, and §2.4's action
thresholds (act above 50) select exactly the wrong ones. For example, EPS-FM-002 (power mode stuck in ACTIVE, loss of
mission) scores 32, "no action", while ADCS-FM-001 (detumble timeout, delayed operations, detected automatically)
scores 80, "action required". §12 and §13 inherit the error.

Correction used from this revision on: RPN′ = (6 − S)(6 − O)(6 − D), so that every factor grows with the risk; the
§2.4 thresholds then apply to RPN′. The ratings themselves are unchanged (they were not re-assessed here). On the
corrected scale no mode reaches 50; the highest are:

| ID | Failure mode | System effect | RPN (Rev A) | RPN′ |
|----|--------------|---------------|-------------|------|
| FM-FM-003 | Fault missed | Hardware damage | 16 | 32 |
| FM-FM-004 | Safe mode not entered | Loss of mission | 24 | 24 |
| WDT-FM-002 | Watchdog kicked too frequently | Potential hang | 24 | 24 |
| ADCS-FM-004 | Actuator saturation | Attitude drift | 36 | 18 |
| BCN-FM-001 | No beacon transmitted | Loss of communication | 36 | 18 |
| EPS-FM-005 | Incorrect SOC calculation | Battery damage | 36 | 18 |
| FM-FM-001 | Fault not logged | Debugging impaired | 36 | 18 |
| SEN-FM-002 | Invalid reading | Wrong decisions | 36 | 18 |
| EPS-FM-002 | Power mode stuck in ACTIVE | Loss of mission | 32 | 16 |
| EPS-FM-006 | Battery temp ignored | Loss of mission | 32 | 16 |

and the six items §12.2 asked action on rank lowest:

| ID | Failure mode | System effect | RPN (Rev A) | RPN′ |
|----|--------------|---------------|-------------|------|
| ADCS-FM-001 | Detumble timeout | Delayed operations | 80 | 4 |
| ADCS-FM-005 | Control instability | Increased power use | 80 | 4 |
| BCN-FM-003 | Beacon too frequent | Regulatory issue | 80 | 4 |
| MD-FM-002 | Write failure | Statistics lost | 80 | 4 |
| MD-FM-003 | Read timeout | Increased boot time | 80 | 4 |
| WDT-FM-003 | Spurious reset | Data loss | 80 | 4 |

The ratings date from January 2026, before the Rust flight software; a re-assessment of S, O and D per mode against
the as-built FDIR below is an open item for the fabrication phase.

### 14.2 FDIR Coverage of the As-built Flight Software

Each Rev A failure-mode group and each mode the CDR added, with what detects it, what responds, and the test.

| Failure mode | Detection (as built) | Response | Verified by |
|--------------|----------------------|----------|-------------|
| EPS-FM-001/002 mode stuck | mode manager table, SOC thresholds with hysteresis (`state_machine`, `eps`) | SAFE below 25 %, EMERGENCY; ground SET_MODE through the same table | `test_state_machine`, `test_eps_control`; SIL `safe-entries`, `mode-chatter` |
| EPS-FM-003 load switch stuck on (Jetson) | `payload_guard` power and spreader samples | both lines low, 10 min off; two series switches (`docs/hardware/PAYLOAD_DESIGN.md` §11.1) | `test_payload_guard`; SIL latch-up injection (`seu-recovery`); switch readback open (AV-06) |
| EPS-FM-005/006 SOC or battery temperature wrong | plausibility of the EPS controller block; stand-in model when it does not answer | conservative SOC; hardware charge window 0–45 °C | `test_eps_control`; SIL `gauge` 0.5 % |
| EPS controller silent (new) | I2C read fails | stand-ins for power; **on the target the ADCS now runs regardless** (RID FSW-01, fixed in `executive.rs`): before, a silent controller skipped the detumble and reported it complete | inspection; host behaviour unchanged (C suites) |
| FM-FM-001 to 005 fault handling | fault log (100 entries, persisted), counters in housekeeping (`FC`, `FL`) | logged; SAFE on critical faults | `test_fault_mgmt`, `test_state_machine` |
| ADCS-FM-001 detumble timeout | 285 min timer (`adcs.rs`) | fault, ADCS IDLE, re-commanded to DETUMBLE | Rust unit tests, `test_adcs`; Monte Carlo (ADCS_DESIGN §11.4) |
| ADCS-FM-002/005 polarity, instability | rate telemetry (`W` in housekeeping) | none on board beyond the timeout; ground | Monte Carlo; HIL open |
| Magnetometer sampled with the coils on (new) | sample time against the coil burst (`CoilTiming`) | sample rejected and counted (`MR`), first one logged; detumble without gyro not completed on a stale rate | Rust unit tests; Monte Carlo `no_window_detected` (every sample rejected, detumble times out, no false completion) |
| Gyro absent or failed (new) | `Option` rate | magnetometer rate, nutation gain 1·10⁻⁴ | Monte Carlo `no_gyro` (17° mean) |
| BCN-FM beacon not sent or too frequent | downlink duty and key-down limits (`downlink`) | frame refused; template on Jetson failure | `test_downlink`, `test_beacon`; SIL `beacon-gap`, `duty`, `keydown` |
| Transponder enabled when forbidden (new) | gates every pass (`transponder.rs`, safety-critical since Rev C) | heartbeat stops (lease 40 ms); the radio board drops the enable in ≤ 90 ms; a stalled loop or a reset stops it too | `test_xpdr_heartbeat`; SIL `xpdr-forbidden` 0 s; coverage gate 100 % lines and conditions |
| UART-FM Jetson link | line CRC and parser (`jetson_link`) | NACK, template beacon | `test_jetson_link`, `test_uart_comm` |
| MD-FM storage write/read | two-slot records with CRC (`nvm`) | older slot used; corrupt record reported | `test_nvm_record`, `test_mission_data` |
| Operations record corrupt (new) | CRC of both slots | **hold** (a lost record must not release a held unit) | `test_ops_state` |
| WDT-FM watchdog | IWDG 30 s, per-task check-in | reset, cause in the boot record and housekeeping (`RST`) | `test_watchdog_mgr`; target test open |
| SEN-FM sensor invalid or stale | range and age checks (`sensors`) | value not used (`-` in housekeeping) | `test_sensors` |
| AI-FM Jetson over-power, over-temperature, hang | `payload_guard`, reply timeout | rail off, cooldown; template beacon | `test_payload_guard`, `test_executive_jetson`; SIL `tjetson`, `ai-energy` |
| Wing burn driver or resistor failed (new) | wing sense after each attempt | next attempt on the B resistors (four lines, one resistor at a time, SRS-F089); with every attempt spent and the wings not sensed, the mode is held at IDLE (EPS-10, SRS-F128) until a DEPLOY_RETRY | `test_deployment`, `test_ops_state`; SIL `burn-lines` ≤ 1 |
| AntS not answering or an element not deployed (new) | AntS status word per element | attempt counted failed, retry 50 min later, DEPLOY_RETRY by the ground | `test_deployment`; SIL smoke (AntS model) |
| GNSS no fix or latched (new) | no accepted fix in a 300 s session | power off; three in a row logged as a fault; the next session power-cycles it | Rust unit tests (`gnss_schedule`); SIL 73 sessions, all with a fix |
| Receiver SEFI (telecommand lost) | 24 h without a valid frame or a carrier (`receiver.rs`) | radio power-cycled for 2 s, never in a pass, logged, at most once a day (FSW-09, SRS-F115) | `test_receiver`, Rust unit tests; SIL `rx-recovery` (SEFIs modelled, each cleared within 26 h) |
| Receiver failed for good (new) | a recovery cycle made, the health line in fault, the loss-of-contact timeout expired (SRS-F116) | the loss-of-contact inhibit no longer silences the beacon; the transponder stays off; TX-OFF and passivation still work (REL-04, SRS-F117) | `test_receiver`; SIL contingency `rx_failed_day=20` (`rx-failed-beacon`) |
| SRAM upset in live state | SRAM2 parity on read (every static); flash double-bit ECC; the TX-OFF, operations and passivation records checked against RAM every 60 s | NMI: cause recorded, immediate reset, cause logged at the next boot; a disagreeing RAM copy settled toward the safe side (AV-07, SRS-F132, SRS-F133). The stack (SRAM3) is not covered | `test_rf_control`, `test_ops_state`, `test_passivation` (the checks); NMI path: fault injection on the EM (RAD-V6) open |
| Passivation commanded by mistake or by an attacker (new) | two commands, two keys (operational, then recovery within 600 s) | the arm lapses; the execute alone is refused (SRS-F110) | `test_passivation` |
| Passivation fire interrupted (eclipse, reset) (new) | sunlight and the fire time; the record and the EPS EOL_STATE at boot | both lines dropped, the fire repeated at the next sunlight, at most 3 attempts; transmitters off for good from the execute (SRS-F111, SRS-F112) | `test_passivation`; SIL end-of-life plan |
| EPS recharge lockout latched (new) | EPS status | cleared only by EPS_CLEAR_LOCKOUT (`CHG!`): the I2C unlock and a LOCK_CLR pulse ≥ 100 ms (EPS-12, SRS-F129) | `test_receiver` (lockout clear) |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-27 | §14 new: the three rating scales are inverted so the RPN ranks benign modes highest (RID FSW-08), corrected RPN′ with the ten highest; FDIR coverage of the as-built flight software for every Rev A group and the modes added at CDR. Ratings unchanged | SMART-QSO Team (CDR avionics review) |
| C | 2026-10-27 | §14.2: the receiver recovery and failed-receiver rows (FSW-09, REL-04), SRAM upsets (AV-07), the wings-failed IDLE hold (EPS-10), passivation and the recharge-lockout clear (REG-01, EPS-06, EPS-12), as implemented in the flight software round 2; the transponder row with the 40 ms lease. Ratings unchanged | SMART-QSO Team (CDR flight software, round 2) |
