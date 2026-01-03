# Software Failure Mode and Effects Analysis (FMEA)
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-FMEA-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NASA-STD-8719.13, MIL-STD-1629A

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

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
