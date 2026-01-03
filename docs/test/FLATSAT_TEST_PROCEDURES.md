# SMART-QSO Flatsat Test Procedures

**Document ID**: SQSO-TP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the test procedures for flatsat (engineering development unit) testing of the SMART-QSO flight software. These procedures verify hardware/software integration prior to flight hardware testing.

## 2. Applicable Documents

| Document ID | Title |
|-------------|-------|
| NPR 7150.2 | NASA Software Engineering Requirements |
| NPR 7123.1 | NASA Systems Engineering Processes and Requirements |
| SQSO-VVM-001 | Verification and Validation Matrix |
| SQSO-SRS-001 | Software Requirements Specification |

## 3. Test Environment

### 3.1 Hardware Configuration

| Component | Description | P/N |
|-----------|-------------|-----|
| OBC EDU | Ambiq Apollo4 Blue Plus EVB | AMA4B2KK-KBR |
| Jetson EDU | Jetson Orin Nano Super Dev Kit | 945-13767-0005-000 |
| EPS Simulator | Custom EPS emulator board | SQSO-EPS-SIM-001 |
| RF Interface | SMA breakout with attenuators | SQSO-RF-SIM-001 |
| Debug Interface | Segger J-Link Plus | 8.08.28 |
| Power Supply | Keysight E36312A | - |

### 3.2 Software Configuration

| Component | Version | Build ID |
|-----------|---------|----------|
| Flight Software | 1.0.0 | TBD |
| Payload Software | 1.0.0 | TBD |
| Ground Software | 1.0.0 | TBD |
| Test Framework | Unity 2.5.2 | - |

### 3.3 Test Equipment

- Digital oscilloscope (≥100 MHz, 4 channel)
- Logic analyzer (≥16 channel)
- Serial terminal (115200 baud capable)
- RF spectrum analyzer (145-438 MHz range)
- Thermal chamber (-20°C to +60°C)
- DC power supply (0-12V, 0-5A)

## 4. Test Procedures

---

### 4.1 TP-001: Boot and Initialization Test

**Objective**: Verify flight software initializes correctly from power-on reset

**Prerequisites**:
- [ ] Flatsat assembled per SQSO-FLATSAT-ASSY-001
- [ ] Debug interface connected
- [ ] Power supply set to 3.3V
- [ ] Serial terminal connected at 115200 baud

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Apply power to OBC | Current draw < 50mA | |
| 2 | Observe boot message on serial | "SMART-QSO FSW v1.0.0" within 2s | |
| 3 | Verify clock initialization | "CLK: HSE locked" message | |
| 4 | Verify memory test | "MEM: SRAM OK, Flash OK" message | |
| 5 | Verify peripheral init | All peripherals report OK | |
| 6 | Verify scheduler start | "SCHED: Started" message | |
| 7 | Verify initial state | STATE: INIT -> SAFE transition | |
| 8 | Measure boot time | Total boot < 5 seconds | |
| 9 | Verify watchdog | WDT kick within 1 second | |
| 10 | Power cycle 5 times | Consistent boot each time | |

**Success Criteria**:
- All steps pass
- Boot time < 5 seconds
- No error messages during initialization
- Consistent behavior across power cycles

**Test Data**:
- Boot time: _______ ms
- Peak current: _______ mA
- Tested by: _______________ Date: ___________
- Witnessed by: ____________ Date: ___________

---

### 4.2 TP-002: Mode Transition Test

**Objective**: Verify all state machine transitions operate correctly

**Prerequisites**:
- [ ] TP-001 passed
- [ ] EPS simulator connected
- [ ] Telemetry logging enabled

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Power on, verify SAFE mode | STATE: SAFE | |
| 2 | Set EPS SOC = 50%, Sun = 1 | No immediate transition | |
| 3 | Wait 60 seconds | Transition to IDLE | |
| 4 | Set EPS SOC = 80%, Sun = 1 | Transition to ACTIVE within 30s | |
| 5 | Set Sun = 0 (eclipse) | Remain in ACTIVE | |
| 6 | Set SOC = 25% | Transition to IDLE | |
| 7 | Set SOC = 15% | Transition to SAFE | |
| 8 | Set SOC = 10% | Remain in SAFE | |
| 9 | Set SOC = 30%, Sun = 1 | Transition to IDLE | |
| 10 | Send SAFE command | Immediate transition to SAFE | |
| 11 | Release SAFE command | Return to IDLE/ACTIVE per SOC | |

**State Transition Matrix Verification**:

| From State | To State | Trigger | Verified |
|------------|----------|---------|----------|
| INIT | SAFE | Boot complete | [ ] |
| SAFE | IDLE | SOC > 20% AND stable 60s | [ ] |
| IDLE | ACTIVE | SOC > 70% AND power budget positive | [ ] |
| ACTIVE | IDLE | SOC < 30% OR power budget negative | [ ] |
| IDLE | SAFE | SOC < 20% OR ground command | [ ] |
| ANY | SAFE | Critical fault OR ground command | [ ] |

**Success Criteria**:
- All transitions occur per specification
- No invalid state transitions
- Telemetry reflects correct state
- Transition times within specification

**Test Data**:
- Transition times logged: [ ] Yes [ ] No
- Tested by: _______________ Date: ___________

---

### 4.3 TP-003: Telemetry Generation Test

**Objective**: Verify telemetry collection, formatting, and transmission

**Prerequisites**:
- [ ] TP-001 passed
- [ ] All sensors connected or simulated
- [ ] Telemetry receiver ready

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Verify telemetry task running | Telemetry generated every 10s | |
| 2 | Capture 10 telemetry packets | All packets complete | |
| 3 | Verify packet structure | Header, payload, CRC correct | |
| 4 | Verify timestamp monotonic | Each timestamp > previous | |
| 5 | Inject known sensor values | Values appear in telemetry | |
| 6 | Verify all channels present | All 32 channels in packet | |
| 7 | Verify CRC calculation | CRC matches calculated value | |
| 8 | Test at minimum rate (1/min) | Packets at correct interval | |
| 9 | Test at maximum rate (1/s) | Packets at correct interval | |
| 10 | Verify packet counter rollover | Counter wraps at 65535 | |

**Telemetry Channels Verification**:

| Channel | Description | Range | Verified |
|---------|-------------|-------|----------|
| BATT_V | Battery voltage | 3.0-4.2V | [ ] |
| BATT_I | Battery current | -2.0 to +2.0A | [ ] |
| BATT_SOC | State of charge | 0-100% | [ ] |
| OBC_TEMP | OBC temperature | -40 to +85°C | [ ] |
| EPS_TEMP | EPS temperature | -40 to +85°C | [ ] |
| JETSON_TEMP | Payload temperature | -20 to +70°C | [ ] |
| SUN_X | Sun sensor X | 0-3.3V | [ ] |
| SUN_Y | Sun sensor Y | 0-3.3V | [ ] |
| MAG_X | Magnetometer X | ±100 µT | [ ] |
| MAG_Y | Magnetometer Y | ±100 µT | [ ] |
| MAG_Z | Magnetometer Z | ±100 µT | [ ] |

**Success Criteria**:
- All telemetry channels present and valid
- CRC verification passes on all packets
- Timestamps monotonically increasing
- Packet rate within ±1% of specification

**Test Data**:
- Packets captured: _______
- CRC errors: _______
- Tested by: _______________ Date: ___________

---

### 4.4 TP-004: Beacon Generation Test

**Objective**: Verify AX.25 beacon frame generation and transmission

**Prerequisites**:
- [ ] TP-001 and TP-003 passed
- [ ] RF interface connected
- [ ] Beacon decoder ready

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Enable beacon generation | Beacon every 60 seconds | |
| 2 | Capture 5 beacon frames | All frames complete | |
| 3 | Verify AX.25 structure | Flag, addresses, control, PID correct | |
| 4 | Verify source callsign | "SQSO-1" in source field | |
| 5 | Verify destination | "CQ" in destination field | |
| 6 | Verify FCS | CRC-16 matches | |
| 7 | Verify info field | Telemetry summary present | |
| 8 | Test AI beacon mode | AI-generated text when available | |
| 9 | Test fallback mode | Template text when AI unavailable | |
| 10 | Verify beacon interval | 60±1 seconds between beacons | |

**Beacon Content Verification**:

| Field | Expected | Actual | Pass/Fail |
|-------|----------|--------|-----------|
| Flag (start) | 0x7E | | |
| Destination | "CQ    " (shifted) | | |
| Source | "SQSO-1" (shifted) | | |
| Control | 0x03 (UI) | | |
| PID | 0xF0 (no L3) | | |
| Info contains voltage | Yes | | |
| Info contains SOC | Yes | | |
| Info contains temp | Yes | | |
| FCS valid | Yes | | |
| Flag (end) | 0x7E | | |

**Success Criteria**:
- Valid AX.25 frames generated
- Correct callsign and addressing
- FCS valid on all frames
- Beacon interval within specification
- AI/fallback modes function correctly

**Test Data**:
- Beacons captured: _______
- FCS errors: _______
- Tested by: _______________ Date: ___________

---

### 4.5 TP-005: Fault Response Test

**Objective**: Verify fault detection, isolation, and recovery

**Prerequisites**:
- [ ] TP-001 through TP-004 passed
- [ ] Fault injection capability available

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Inject watchdog timeout | System resets within 5s | |
| 2 | Verify boot reason | Boot reason = WDT_RESET | |
| 3 | Inject memory error | Error logged, operation continues | |
| 4 | Inject sensor timeout | Sensor marked stale, fallback used | |
| 5 | Inject UART error | Error counted, retry initiated | |
| 6 | Simulate Jetson failure | Graceful degradation to fallback | |
| 7 | Inject power brownout | Transition to SAFE mode | |
| 8 | Simulate over-temperature | Thermal throttling activated | |
| 9 | Inject 10 rapid faults | No crash, all faults logged | |
| 10 | Verify fault counters | All faults counted correctly | |

**Fault Response Matrix**:

| Fault Type | Detection Time | Response | Recovery | Verified |
|------------|----------------|----------|----------|----------|
| Watchdog timeout | 5s | Reset | Auto | [ ] |
| Stack overflow | Immediate | Reset | Auto | [ ] |
| Memory corruption | Next access | Log + continue | N/A | [ ] |
| Sensor timeout | 3x poll interval | Use last good | Auto | [ ] |
| UART error | Immediate | Retry x3 | Auto | [ ] |
| Jetson unresponsive | 30s | Fallback mode | Manual | [ ] |
| Over-temperature | Immediate | Throttle | Auto | [ ] |
| Under-voltage | Immediate | SAFE mode | Auto | [ ] |

**Success Criteria**:
- All faults detected within specification
- Correct response to each fault type
- System recovers when fault clears
- No crashes or hangs during fault injection
- All faults logged correctly

**Test Data**:
- Faults injected: _______
- Unexpected behaviors: _______
- Tested by: _______________ Date: ___________

---

### 4.6 TP-006: Power Mode Test

**Objective**: Verify power consumption in each operational mode

**Prerequisites**:
- [ ] TP-001 through TP-005 passed
- [ ] Precision ammeter connected
- [ ] EPS simulator configured

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Measure SAFE mode power | < 100 mW average | |
| 2 | Measure IDLE mode power | < 200 mW average | |
| 3 | Measure ACTIVE mode (no AI) | < 500 mW average | |
| 4 | Measure ACTIVE mode (AI burst) | < 15 W peak | |
| 5 | Verify AI duty cycle | < 10% duty cycle | |
| 6 | Measure sleep current | < 10 µA | |
| 7 | Measure beacon transmit power | 0.5 W at low power | |
| 8 | Measure beacon transmit power | 2.0 W at high power | |
| 9 | Log power over 1 orbit (90 min) | Average within budget | |
| 10 | Verify power transitions | Smooth, no glitches | |

**Power Budget Verification**:

| Mode | Specification | Measured | Margin | Pass/Fail |
|------|---------------|----------|--------|-----------|
| SAFE | < 100 mW | | | |
| IDLE | < 200 mW | | | |
| ACTIVE (no AI) | < 500 mW | | | |
| AI Inference | < 15 W | | | |
| TX Low | 0.5 W | | | |
| TX High | 2.0 W | | | |
| Sleep | < 10 µA | | | |

**Success Criteria**:
- All modes within power budget
- Positive power margin in all modes
- AI duty cycle controlled correctly
- No power glitches during transitions

**Test Data**:
- Power log file: _____________
- Tested by: _______________ Date: ___________

---

### 4.7 TP-007: Extended Duration Test

**Objective**: Verify system stability over extended operation

**Prerequisites**:
- [ ] TP-001 through TP-006 passed
- [ ] Data logging configured
- [ ] Orbit simulation available

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Configure 7-day test | Logging enabled | |
| 2 | Simulate orbit cycles | 112 orbits (7 days) | |
| 3 | Include eclipse periods | 35 min eclipse per orbit | |
| 4 | Exercise all modes | Per ConOps timeline | |
| 5 | Log all telemetry | Complete record | |
| 6 | Monitor for anomalies | No unexpected resets | |
| 7 | Verify memory usage | No leaks, stable usage | |
| 8 | Verify timing accuracy | < 1 ppm drift | |
| 9 | Verify beacon generation | All beacons transmitted | |
| 10 | Post-test analysis | All requirements met | |

**Extended Test Checklist**:

| Day | Orbits | Resets | Errors | Memory | Notes |
|-----|--------|--------|--------|--------|-------|
| 1 | 16 | | | | |
| 2 | 16 | | | | |
| 3 | 16 | | | | |
| 4 | 16 | | | | |
| 5 | 16 | | | | |
| 6 | 16 | | | | |
| 7 | 16 | | | | |

**Success Criteria**:
- No unexpected resets during 7-day test
- Memory usage stable (< 5% variation)
- All scheduled operations completed
- Error rate < 0.01%
- Timing drift < 1 ppm

**Test Data**:
- Test start: _____________
- Test end: _______________
- Total errors: ___________
- Tested by: _______________ Date: ___________

---

## 5. Test Reports

### 5.1 Test Summary Report Template

```
SMART-QSO FLATSAT TEST SUMMARY REPORT

Test Campaign ID: _______________
Test Dates: _______________
Configuration: _______________

TEST RESULTS SUMMARY:
| Test ID | Test Name | Result | Issues |
|---------|-----------|--------|--------|
| TP-001 | Boot and Init | | |
| TP-002 | Mode Transition | | |
| TP-003 | Telemetry Gen | | |
| TP-004 | Beacon Gen | | |
| TP-005 | Fault Response | | |
| TP-006 | Power Mode | | |
| TP-007 | Extended Duration | | |

ANOMALIES:
1. _______________
2. _______________

RECOMMENDATIONS:
1. _______________
2. _______________

Prepared by: _______________ Date: ___________
Reviewed by: _______________ Date: ___________
Approved by: _______________ Date: ___________
```

## 6. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
