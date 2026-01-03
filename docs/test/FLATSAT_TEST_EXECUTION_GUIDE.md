# SMART-QSO Flatsat Test Execution Guide

**Document ID**: SQSO-FTEG-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This guide provides step-by-step instructions for executing the flatsat test campaign, covering setup, test execution, troubleshooting, and data collection per NASA Ames SmallSat heritage practices.

## 2. Flatsat Configuration

### 2.1 Hardware Setup

```
FLATSAT CONFIGURATION

┌───────────────────────────────────────────────────────────┐
│                    FLATSAT BENCH                          │
├───────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐              │
│  │   EPS   │◄──►│   OBC   │◄──►│   RF    │              │
│  │  Board  │    │  Board  │    │  Board  │              │
│  └────┬────┘    └────┬────┘    └────┬────┘              │
│       │              │              │                    │
│       │         ┌────┴────┐         │                    │
│       │         │ Jetson  │         │                    │
│       │         │  Orin   │         │                    │
│       │         └────┬────┘         │                    │
│       │              │              │                    │
│  ┌────┴────┐    ┌────┴────┐    ┌────┴────┐              │
│  │ Power   │    │ Debug   │    │ RF Test │              │
│  │ Supply  │    │ Console │    │ Setup   │              │
│  └─────────┘    └─────────┘    └─────────┘              │
│                                                           │
│  ┌─────────────────────────────────────────────────┐     │
│  │               GROUND STATION GSE                 │     │
│  │  TNC + SDR + Mission Control Software           │     │
│  └─────────────────────────────────────────────────┘     │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

### 2.2 Equipment List

| Equipment | Purpose | Settings |
|-----------|---------|----------|
| Power supply | EPS power | 0-20V, 5A |
| USB hub | Debug interfaces | USB 3.0 |
| J-Link debugger | OBC debug | SWD |
| USB-UART adapter | Console | 115200 8N1 |
| SDR (RTL-SDR/USRP) | RF monitoring | VHF/UHF |
| RF attenuator | RF testing | 40-60 dB |
| Dummy load | TX testing | 50Ω |
| TNC | Packet decoding | KISS mode |
| Oscilloscope | Signal debug | Optional |
| Logic analyzer | Bus debug | Optional |

### 2.3 Software Requirements

| Software | Purpose | Version |
|----------|---------|---------|
| J-Link Commander | Debug/programming | Latest |
| PuTTY/minicom | Serial console | Any |
| GQRX/SDR# | SDR receiver | Latest |
| Direwolf | Packet decoder | 1.6+ |
| Mission Control SW | Operations | 1.0.0 |
| Test Automation | Scripted tests | 1.0.0 |

### 2.4 Connection Diagram

| Connection | From | To | Cable |
|------------|------|-----|-------|
| Power | Bench supply | EPS power input | 2-wire |
| OBC Debug | J-Link | OBC SWD header | 10-pin |
| OBC Console | USB-UART | OBC debug UART | 3-wire |
| Jetson Debug | USB | Jetson micro-USB | USB |
| RF TX | RF board | Attenuator→SDR | SMA coax |
| RF RX | Signal gen | RF board UHF | SMA coax |

## 3. Pre-Test Procedures

### 3.1 Daily Startup Checklist

```
DAILY STARTUP CHECKLIST

Date: ___________
Operator: ___________

WORKSPACE:
[ ] ESD workspace verified (wrist strap, mat)
[ ] Bench clean and organized
[ ] Fire extinguisher accessible
[ ] Emergency contacts posted

EQUIPMENT:
[ ] Power supply verified (voltage limits set)
[ ] Connections verified (no loose wires)
[ ] Debug tools connected
[ ] Ground station software running
[ ] SDR receiver configured

SOFTWARE:
[ ] Latest firmware loaded (check version)
[ ] Test scripts updated
[ ] Log files configured (new date folder)
[ ] Telemetry database running

DOCUMENTATION:
[ ] Test procedure available
[ ] As-run log template ready
[ ] Anomaly forms available

Ready to test: [ ] YES  [ ] NO

Signature: ___________ Date/Time: ___________
```

### 3.2 Power-On Sequence

```
POWER-ON SEQUENCE

1. Verify all connections
   └─ Visual inspection
   └─ Continuity check if uncertain

2. Set power supply limits
   └─ Voltage: 8.4V (2S battery equivalent)
   └─ Current: 2A limit

3. Enable power supply output
   └─ Monitor current (should be <100mA at boot)

4. Verify OBC boot
   └─ Check console output
   └─ Verify version string
   └─ Confirm SAFE mode entry

5. Verify subsystem initialization
   └─ EPS responding (I2C)
   └─ RF configured (SPI)
   └─ Jetson powered down

6. Record power-on data
   └─ Time stamp
   └─ Boot message
   └─ Initial telemetry

Expected boot time: <30 seconds
```

### 3.3 System Health Check

| Check | Expected | Actual | Pass |
|-------|----------|--------|------|
| OBC boot complete | "Boot OK" | | [ ] |
| Mode | SAFE | | [ ] |
| EPS telemetry | All valid | | [ ] |
| Voltage | 8.2-8.4V | | [ ] |
| Current | <200mA | | [ ] |
| Temperature | 20-25°C | | [ ] |
| RF disabled | TX off | | [ ] |

## 4. Test Execution

### 4.1 Test 1: Boot and Initialization

**Test ID**: FLATSAT-001
**Duration**: 30 minutes
**Purpose**: Verify boot sequence and initialization

```
TEST PROCEDURE: BOOT AND INITIALIZATION

SETUP:
[ ] Power supply at 8.4V, 2A limit
[ ] Console connected
[ ] Log capture enabled

EXECUTION:

Step 1: Power cycle
  Action: Toggle power supply off then on
  Wait: 30 seconds
  Expected: Boot complete message
  Actual: _____________________
  Pass: [ ]

Step 2: Verify boot messages
  Action: Review console output
  Expected: All initialization OK
  Actual: _____________________
  Pass: [ ]

Step 3: Check mode
  Action: Send CMD_GET_MODE
  Expected: MODE_SAFE
  Actual: _____________________
  Pass: [ ]

Step 4: Check telemetry
  Action: Request full telemetry
  Expected: All values in range
  Actual: _____________________
  Pass: [ ]

Step 5: Check boot counter
  Action: Note boot counter value
  Expected: Incremented by 1
  Actual: _____________________
  Pass: [ ]

CLEANUP:
[ ] Log saved
[ ] Results recorded

OVERALL: [ ] PASS  [ ] FAIL  [ ] PARTIAL
```

### 4.2 Test 2: Mode Transitions

**Test ID**: FLATSAT-002
**Duration**: 45 minutes
**Purpose**: Verify state machine operation

```
TEST PROCEDURE: MODE TRANSITIONS

EXECUTION:

Step 1: SAFE → IDLE transition
  Action: Send CMD_SET_MODE IDLE
  Wait: 5 seconds
  Expected: MODE_IDLE confirmed
  Actual: _____________________
  Pass: [ ]

Step 2: IDLE → ACTIVE transition
  Action: Send CMD_SET_MODE ACTIVE
  Wait: 5 seconds
  Expected: MODE_ACTIVE confirmed
  Actual: _____________________
  Pass: [ ]

Step 3: ACTIVE → IDLE transition
  Action: Send CMD_SET_MODE IDLE
  Wait: 5 seconds
  Expected: MODE_IDLE confirmed
  Actual: _____________________
  Pass: [ ]

Step 4: IDLE → SAFE transition
  Action: Send CMD_SET_MODE SAFE
  Wait: 5 seconds
  Expected: MODE_SAFE confirmed
  Actual: _____________________
  Pass: [ ]

Step 5: Invalid transition rejection
  Action: From SAFE, send CMD_SET_MODE ACTIVE
  Expected: Rejected (must go through IDLE)
  Actual: _____________________
  Pass: [ ]

Step 6: Fault triggers SAFE
  Action: Simulate fault condition
  Expected: Auto transition to SAFE
  Actual: _____________________
  Pass: [ ]

OVERALL: [ ] PASS  [ ] FAIL  [ ] PARTIAL
```

### 4.3 Test 3: Telemetry Generation

**Test ID**: FLATSAT-003
**Duration**: 30 minutes
**Purpose**: Verify telemetry collection and formatting

```
TEST PROCEDURE: TELEMETRY GENERATION

EXECUTION:

Step 1: Request housekeeping telemetry
  Action: Send CMD_GET_HK
  Expected: HK packet received
  Actual: _____________________
  Pass: [ ]

Step 2: Verify EPS telemetry
  Voltages: _____________________
  Currents: _____________________
  SOC: _____________________
  Pass: [ ]

Step 3: Verify OBC telemetry
  Mode: _____________________
  Uptime: _____________________
  Boot count: _____________________
  Pass: [ ]

Step 4: Verify sensor data
  Temperature: _____________________
  Magnetometer: _____________________
  Pass: [ ]

Step 5: Verify packet format
  Action: Parse with ground software
  Expected: All fields decode correctly
  Actual: _____________________
  Pass: [ ]

OVERALL: [ ] PASS  [ ] FAIL  [ ] PARTIAL
```

### 4.4 Test 4: Beacon Generation

**Test ID**: FLATSAT-004
**Duration**: 1 hour
**Purpose**: Verify beacon generation and transmission

```
TEST PROCEDURE: BEACON GENERATION

SETUP:
[ ] RF attenuator connected
[ ] SDR receiver configured (145.XXX MHz)
[ ] Packet decoder running

EXECUTION:

Step 1: Enable beacon in IDLE mode
  Action: Send CMD_BEACON_ENABLE
  Expected: Beacon started
  Actual: _____________________
  Pass: [ ]

Step 2: Verify RF transmission
  Action: Monitor SDR waterfall
  Expected: Beacon visible every 60s
  Actual: _____________________
  Pass: [ ]

Step 3: Decode beacon packet
  Action: Check packet decoder
  Expected: Valid AX.25 packet
  Actual: _____________________
  Pass: [ ]

Step 4: Verify beacon content
  Callsign: _____________________
  Telemetry: _____________________
  Message: _____________________
  Pass: [ ]

Step 5: Enable AI beacon (ACTIVE mode)
  Action: Transition to ACTIVE
  Wait: AI generates content
  Expected: AI-generated message
  Actual: _____________________
  Pass: [ ]

Step 6: Verify fallback on AI timeout
  Action: Disable Jetson
  Expected: Template beacon
  Actual: _____________________
  Pass: [ ]

OVERALL: [ ] PASS  [ ] FAIL  [ ] PARTIAL
```

### 4.5 Test 5: Fault Response

**Test ID**: FLATSAT-005
**Duration**: 2 hours
**Purpose**: Verify fault detection and response

```
TEST PROCEDURE: FAULT RESPONSE

EXECUTION:

Step 1: Watchdog timeout
  Action: Halt OBC (debugger)
  Wait: 5 seconds
  Expected: Watchdog reset
  Actual: _____________________
  Pass: [ ]

Step 2: Low voltage response
  Action: Reduce supply to 6.0V
  Expected: SAFE mode entry
  Actual: _____________________
  Pass: [ ]

Step 3: Over-temperature response
  Action: Inject temp fault via debug
  Expected: Throttling/SAFE mode
  Actual: _____________________
  Pass: [ ]

Step 4: AI timeout
  Action: Delay AI response >30s
  Expected: Fallback to template
  Actual: _____________________
  Pass: [ ]

Step 5: I2C bus error
  Action: Disconnect EPS temporarily
  Expected: Fault flagged, SAFE mode
  Actual: _____________________
  Pass: [ ]

Step 6: Recovery from fault
  Action: Restore nominal conditions
  Expected: Normal operation resumes
  Actual: _____________________
  Pass: [ ]

OVERALL: [ ] PASS  [ ] FAIL  [ ] PARTIAL
```

### 4.6 Test 6: Power Mode Test

**Test ID**: FLATSAT-006
**Duration**: 4 hours
**Purpose**: Verify power management

```
TEST PROCEDURE: POWER MODE

EXECUTION:

Step 1: Measure SAFE mode power
  Action: Record power in SAFE
  Expected: <200 mW
  Actual: _____________________
  Pass: [ ]

Step 2: Measure IDLE mode power
  Action: Record power in IDLE
  Expected: <400 mW
  Actual: _____________________
  Pass: [ ]

Step 3: Measure ACTIVE mode power
  Action: Record power in ACTIVE (no AI)
  Expected: <800 mW
  Actual: _____________________
  Pass: [ ]

Step 4: Measure AI burst power
  Action: Trigger AI inference
  Expected: <7 W peak
  Actual: _____________________
  Pass: [ ]

Step 5: AI duty cycle
  Action: Run for 1 hour
  Expected: AI <10% duty
  Actual: _____________________
  Pass: [ ]

Step 6: SOC tracking
  Action: Discharge from 100% to 50%
  Expected: SOC tracks accurately (±5%)
  Actual: _____________________
  Pass: [ ]

OVERALL: [ ] PASS  [ ] FAIL  [ ] PARTIAL
```

### 4.7 Test 7: Extended Duration

**Test ID**: FLATSAT-007
**Duration**: 168 hours (1 week)
**Purpose**: Verify long-term stability

```
TEST PROCEDURE: EXTENDED DURATION

SETUP:
[ ] Automatic logging configured
[ ] Watchdog alert email set
[ ] Daily check schedule established

DAY 1:
[ ] System stable
[ ] No anomalies
[ ] Power nominal
Notes: _____________________

DAY 2:
[ ] System stable
[ ] No anomalies
[ ] Power nominal
Notes: _____________________

DAY 3:
[ ] System stable
[ ] No anomalies
[ ] Power nominal
Notes: _____________________

DAY 4:
[ ] System stable
[ ] No anomalies
[ ] Power nominal
Notes: _____________________

DAY 5:
[ ] System stable
[ ] No anomalies
[ ] Power nominal
Notes: _____________________

DAY 6:
[ ] System stable
[ ] No anomalies
[ ] Power nominal
Notes: _____________________

DAY 7:
[ ] System stable
[ ] No anomalies
[ ] Power nominal
Notes: _____________________

METRICS:
- Total uptime: _____ hours
- Reset count: _____
- Anomaly count: _____
- Power stability: _____

OVERALL: [ ] PASS  [ ] FAIL  [ ] PARTIAL
```

## 5. Troubleshooting

### 5.1 Common Issues

| Symptom | Possible Cause | Resolution |
|---------|----------------|------------|
| No boot | Power connection | Check cables |
| No console | UART settings | Verify 115200 8N1 |
| EPS not responding | I2C issue | Check address, pull-ups |
| RF not working | SPI issue | Check mode, clock |
| Jetson no response | Power enable | Check GPIO |
| Watchdog resets | Task hang | Check stack, debug |
| No beacon | TX not enabled | Verify mode, command |

### 5.2 Debug Commands

| Command | Purpose |
|---------|---------|
| `ver` | Show version |
| `stat` | System status |
| `tlm` | Dump telemetry |
| `i2c scan` | Find I2C devices |
| `spi test` | Test SPI bus |
| `gpio dump` | Show GPIO states |
| `log` | Show recent log |
| `reset` | Soft reset |

### 5.3 Escalation

| Issue | Action |
|-------|--------|
| Cannot resolve | Document in anomaly report |
| Hardware suspect | Swap with spare if available |
| Software bug | Create issue in tracker |
| Design issue | Escalate to systems engineer |

## 6. Data Collection

### 6.1 Required Data

| Data | Format | Frequency |
|------|--------|-----------|
| Console log | Text file | Continuous |
| Telemetry | CSV/binary | 1 Hz |
| Power data | CSV | 10 Hz |
| RF captures | IQ recording | Per beacon |
| Anomaly notes | Form | As needed |

### 6.2 Log Naming Convention

```
YYYYMMDD_HHMMSS_TESTID_description.log
Example: 20260102_143000_FLATSAT001_boot_test.log
```

### 6.3 Data Archive

| Archive | Location | Retention |
|---------|----------|-----------|
| Raw data | Project server | 5 years |
| Processed | Database | Mission life |
| Reports | Document repo | Permanent |

## 7. Safety

### 7.1 Safety Reminders

- Always use ESD protection
- Never exceed voltage limits
- Never transmit without attenuator
- Report any anomalies
- Know emergency procedures

### 7.2 Emergency Procedures

| Emergency | Action |
|-----------|--------|
| Fire | Power off, extinguisher, evacuate |
| Smoke/smell | Power off, ventilate |
| Injury | First aid, call emergency |
| Equipment damage | Power off, document |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
