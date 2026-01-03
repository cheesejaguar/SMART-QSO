# SMART-QSO Operator Training Materials

**Document ID**: SQSO-OTM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document contains the detailed training materials for SMART-QSO operator training, providing content for all seven training modules defined in the Operator Training Plan.

## 2. Module 1: Mission Overview

### 2.1 Learning Objectives

After completing this module, operators will be able to:
- State the SMART-QSO mission objectives
- Describe the spacecraft architecture
- Explain the ground segment configuration
- List the mission phases and success criteria

### 2.2 Mission Objectives

**Primary Mission Objective**:
Demonstrate AI-assisted amateur radio operations from a CubeSat platform, providing contextual beacon messages and autonomous operations.

**Mission Success Criteria**:

| Level | Duration | Criterion |
|-------|----------|-----------|
| Level 1 (Minimum) | 30 days | Beacon operations verified |
| Level 2 (Full) | 1 year | AI-assisted beacons operational |
| Level 3 (Extended) | 2+ years | Continuous amateur radio service |

### 2.3 Spacecraft Overview

```
SMART-QSO SPACECRAFT ARCHITECTURE

┌─────────────────────────────────────────────────────────────┐
│                    1U CubeSat (10×10×11.35 cm)              │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐      │
│   │    EPS      │   │    OBC      │   │     RF      │      │
│   │ 7W solar    │◄─►│  Apollo4    │◄─►│  Si4463     │      │
│   │ 7Wh battery │   │  FreeRTOS   │   │  VHF/UHF    │      │
│   └─────────────┘   └──────┬──────┘   └─────────────┘      │
│                            │                                │
│                     ┌──────┴──────┐                         │
│                     │   Jetson    │                         │
│                     │  Orin Nano  │                         │
│                     │  (AI)       │                         │
│                     └─────────────┘                         │
│                                                             │
│   Mass: 1.2 kg        Power: 0.35W avg      Life: 2 years  │
└─────────────────────────────────────────────────────────────┘
```

**Key Subsystems**:

| Subsystem | Function | Key Parameters |
|-----------|----------|----------------|
| EPS | Power generation/storage | 7W solar, 7Wh battery |
| OBC | Command and control | Apollo4 MCU, FreeRTOS |
| RF | Communications | 145/435 MHz, 0.5-2W |
| AI Payload | Beacon generation | Jetson Orin Nano, TinyLM |
| ADCS | Attitude sensing | Magnetometer, sun sensors |

### 2.4 Ground Segment

```
GROUND SEGMENT ARCHITECTURE

┌─────────────────────────────────────────────────────────────┐
│                    GROUND STATION                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐   │
│   │ Antenna │──►│  TRX    │──►│  TNC    │──►│  MCS    │   │
│   │ VHF/UHF │   │         │   │         │   │ Software│   │
│   └─────────┘   └─────────┘   └─────────┘   └────┬────┘   │
│                                                   │         │
│                                              ┌────┴────┐    │
│                                              │Database │    │
│                                              └─────────┘    │
└─────────────────────────────────────────────────────────────┘
```

**Ground Station Components**:
- VHF/UHF antenna with elevation/azimuth rotator
- Dual-band transceiver (2m/70cm)
- TNC for packet decoding
- Mission Control Software (MCS)
- Telemetry database

### 2.5 Mission Phases

| Phase | Duration | Key Activities |
|-------|----------|----------------|
| LEOP | 24 hours | Acquisition, health check, deploy confirmation |
| Commissioning | 14 days | System checkout, calibration, AI enable |
| Primary Ops | 2 years | Autonomous beacon operations, amateur service |
| EOL | TBD | Passivation, deorbit |

### 2.6 Module 1 Assessment Questions

1. What is the primary mission objective of SMART-QSO?
2. What are the three levels of mission success?
3. What is the nominal spacecraft power consumption?
4. How long is the commissioning phase?
5. What happens during LEOP?

---

## 3. Module 2: Spacecraft Systems

### 3.1 Learning Objectives

After completing this module, operators will be able to:
- Explain each subsystem's function and operation
- Interpret subsystem telemetry
- Identify nominal and off-nominal conditions
- Describe power modes and transitions

### 3.2 Electrical Power System (EPS)

**Architecture**:
```
EPS BLOCK DIAGRAM

Solar Arrays (6x)      Battery Pack          Power Distribution
    │                      │                       │
    ▼                      ▼                       ▼
┌────────┐            ┌────────┐            ┌────────────┐
│  MPPT  │───────────►│  BMS   │───────────►│   PMIC     │
│        │            │        │            │            │
└────────┘            └────────┘            └────────────┘
    │                      │                       │
    ▼                      ▼                       ▼
 7W BOL               2x18650               3.3V, 5V rails
 5.6W EOL             7 Wh                  Load switches
```

**Key Telemetry Points**:

| Parameter | Nominal Range | Warning | Critical |
|-----------|---------------|---------|----------|
| Battery Voltage | 7.4-8.4V | <7.0V | <6.0V |
| Battery Current | ±500mA | ±1A | ±2A |
| SOC | 30-100% | <30% | <20% |
| Solar Current | 0-1A | N/A | N/A |
| 3.3V Rail | 3.2-3.4V | <3.1V | <3.0V |
| 5V Rail | 4.9-5.1V | <4.8V | <4.7V |

**Power Modes**:

| Mode | Description | Power | AI Active |
|------|-------------|-------|-----------|
| SAFE | Minimum operations | <0.2W | No |
| IDLE | Normal standby | <0.4W | No |
| ACTIVE | Full operations | <0.8W | Yes (duty cycled) |
| AI_BURST | Inference active | <7W | Yes |

### 3.3 On-Board Computer (OBC)

**Architecture**:
```
OBC ARCHITECTURE

┌─────────────────────────────────────────────────────────────┐
│                    Apollo4 Blue Plus                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │  Core   │  │  SRAM   │  │  Flash  │  │  FRAM   │       │
│  │  96MHz  │  │  2MB    │  │  2+32MB │  │  128KB  │       │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘       │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐ │
│  │                    FreeRTOS                           │ │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐       │ │
│  │  │Beacon│ │ TLM  │ │ CMD  │ │ PWR  │ │ AI   │       │ │
│  │  │ Task │ │ Task │ │ Task │ │ Task │ │ Task │       │ │
│  │  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘       │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  Watchdog: 5-second timeout                                │
└─────────────────────────────────────────────────────────────┘
```

**Key Telemetry**:

| Parameter | Description | Nominal |
|-----------|-------------|---------|
| MODE | Current operating mode | SAFE/IDLE/ACTIVE |
| UPTIME | Seconds since boot | Increasing |
| BOOT_COUNT | Boot counter | Stable |
| RESET_CAUSE | Last reset reason | POWER/WDT/CMD |
| CPU_UTIL | CPU utilization | <50% |
| TASK_STATUS | Task health flags | All OK |

**Fault Management**:

| Fault | Detection | Response |
|-------|-----------|----------|
| Watchdog timeout | 5s no kick | Reset, increment counter |
| Low battery | SOC <20% | Transition to SAFE |
| Over-temperature | Temp >50°C | Throttle, then SAFE |
| Task failure | Health check | Restart task |

### 3.4 AI Payload (Jetson)

**Operation**:
```
AI PAYLOAD OPERATION

              Power Gate                 UART Interface
                 │                            │
                 ▼                            ▼
┌──────────────────────────────────────────────────────────┐
│              Jetson Orin Nano                            │
│  ┌──────────────────────────────────────────────────┐   │
│  │           TinyLM Model (~1 MB INT8)              │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐ │   │
│  │  │  Context   │─►│  Inference │─►│   Output   │ │   │
│  │  │  (TLM)     │  │  (TensorRT)│  │  (Beacon)  │ │   │
│  │  └────────────┘  └────────────┘  └────────────┘ │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  Duty Cycle: ≤10%     Power: 5-7W     Startup: ~10s    │
└──────────────────────────────────────────────────────────┘
```

**AI Request/Response Flow**:
1. OBC assembles context (telemetry, orbit data)
2. OBC powers on Jetson
3. Jetson boots, loads model
4. OBC sends context via UART
5. Jetson generates beacon text
6. OBC receives response, powers off Jetson
7. OBC formats beacon for transmission

**Fallback Hierarchy**:
```
AI Available?
    │
    ├── Yes → Use AI-generated beacon
    │
    └── No → Use template beacon
            │
            ├── Template A: Telemetry summary
            ├── Template B: Orbit info
            └── Template C: Minimal beacon
```

### 3.5 RF Subsystem

**Configuration**:

| Parameter | Value |
|-----------|-------|
| Downlink Frequency | 145.XXX MHz (VHF) |
| Uplink Frequency | 435.XXX MHz (UHF) |
| TX Power | 0.5-2W (selectable) |
| Modulation | AFSK 1200 / GFSK 9600 |
| Protocol | AX.25 |
| Beacon Period | 60-120 seconds |

**Beacon Structure**:
```
AX.25 BEACON FRAME

┌─────────┬─────────┬─────────┬─────────┬─────────────────┐
│ Flag    │ Address │ Control │ PID     │ Information     │
│ 0x7E    │ Callsign│ 0x03    │ 0xF0    │ Beacon content  │
└─────────┴─────────┴─────────┴─────────┴─────────────────┘

Information Field:
>CALLSIGN: [Telemetry] [AI Message or Template]
```

### 3.6 Module 2 Assessment Questions

1. What is the battery SOC threshold for entering SAFE mode?
2. How long is the watchdog timeout?
3. What is the AI payload duty cycle limit?
4. What happens if the Jetson times out during inference?
5. What is the beacon transmission period range?

---

## 4. Module 3: Ground Station Operations

### 4.1 Learning Objectives

After completing this module, operators will be able to:
- Operate ground station hardware
- Configure ground software
- Plan and execute passes
- Decode beacons and telemetry

### 4.2 Ground Station Hardware

**Antenna System**:
- VHF Yagi (9-element, 13 dBd)
- UHF Yagi (18-element, 15 dBd)
- Az/El rotator with computer control
- Low-noise preamplifiers

**Transceiver**:
- Dual-band (144/430 MHz)
- USB CAT control
- Audio interface to TNC

**TNC Configuration**:
```
TNC SETTINGS (KISS Mode)

Port: COM3 (or /dev/ttyUSB0)
Baud: 9600
Mode: KISS
TXD: 300ms
Persist: 63
SlotTime: 10ms
```

### 4.3 Pass Planning

**Orbit Parameters**:
- Altitude: ~400 km
- Inclination: 51.6°
- Period: ~92 minutes
- Passes per day: 4-6 visible

**Pass Planning Procedure**:
1. Load current TLE from Space-Track
2. Predict passes for next 24 hours
3. Identify passes with elevation >20°
4. Schedule priority operations for best passes
5. Prepare command scripts if needed
6. Brief team on pass plan

**Pass Timeline Example**:
```
PASS TIMELINE

T-5:00  Ground station power up
T-3:00  Antenna to initial position
T-2:00  Receiver configured, recording started
T-1:00  Ready for AOS

T+0:00  AOS (Acquisition of Signal)
        - Acquire beacon
        - Begin telemetry collection

T+X:XX  TCA (Time of Closest Approach)
        - Maximum elevation
        - Best link conditions

T+Y:YY  LOS (Loss of Signal)
        - End recording
        - Initial data review

T+5:00  Antenna stowed
        - Full data review
        - Update shift log
```

### 4.4 Doppler Compensation

**Doppler Shift Calculation**:
- Maximum Doppler: ±3.4 kHz (VHF), ±10.2 kHz (UHF)
- Compensation: Automatic via CAT control
- Backup: Manual tuning knob

### 4.5 Module 3 Practical Exercise

**Exercise: Beacon Decode**
1. Configure SDR for VHF receive
2. Start packet decoder
3. Play recorded beacon file
4. Verify successful decode
5. Extract telemetry values

---

## 5. Module 4: Telemetry Analysis

### 5.1 Learning Objectives

After completing this module, operators will be able to:
- Parse beacon telemetry
- Identify nominal vs off-nominal values
- Perform trend analysis
- Create telemetry reports

### 5.2 Telemetry Frame Structure

```
TELEMETRY FRAME STRUCTURE

Byte 0-1:  Frame ID (0x544D = "TM")
Byte 2:    Frame type (0x01 = HK, 0x02 = EXT)
Byte 3:    Sequence counter
Byte 4-7:  Timestamp (seconds since epoch)
Byte 8-N:  Payload (type-dependent)
Byte N+1:  Checksum (XOR)
```

### 5.3 Housekeeping Telemetry

| Field | Bytes | Scale | Units |
|-------|-------|-------|-------|
| MODE | 1 | 0=SAFE,1=IDLE,2=ACTIVE | - |
| BATT_V | 2 | × 0.001 | V |
| BATT_I | 2 | × 0.001 | A |
| SOC | 1 | direct | % |
| SOLAR_I | 2 | × 0.001 | A |
| TEMP_OBC | 2 | × 0.1 - 40 | °C |
| TEMP_BATT | 2 | × 0.1 - 40 | °C |
| TEMP_PA | 2 | × 0.1 - 40 | °C |
| UPTIME | 4 | direct | s |
| BOOT_CNT | 2 | direct | - |
| AI_STATUS | 1 | bitmask | - |

### 5.4 Limit Checking

| Parameter | Green | Yellow | Red |
|-----------|-------|--------|-----|
| BATT_V | >7.4V | 7.0-7.4V | <7.0V |
| SOC | >30% | 20-30% | <20% |
| TEMP_OBC | 0-45°C | -10-0°C or 45-55°C | <-10°C or >55°C |
| TEMP_BATT | 0-40°C | -5-0°C or 40-45°C | <-5°C or >45°C |

### 5.5 Trend Analysis

**Key Trending Parameters**:
1. Battery SOC over multiple orbits
2. Temperature cycling (day/night)
3. Beacon success rate
4. AI utilization percentage
5. Boot counter (should be stable)

### 5.6 Module 4 Assessment

**Scenario**: Given telemetry data, identify:
1. Current operating mode
2. Battery health status
3. Any out-of-limit conditions
4. Recommended actions

---

## 6. Module 5: Commanding Procedures

### 6.1 Learning Objectives

After completing this module, operators will be able to:
- Understand command authority
- Execute real-time commands
- Verify command execution
- Handle command failures

### 6.2 Command Authority

| Command Class | Authority Required | Example |
|---------------|-------------------|---------|
| Routine | Flight Controller | Request telemetry |
| Mode Change | Flight Controller + MD concurrence | SAFE→IDLE |
| Critical | Mission Director approval | SW update |
| Emergency | Any authorized operator | SAFE mode |

### 6.3 Command Execution Procedure

```
COMMAND EXECUTION PROCEDURE

1. IDENTIFY command to be executed
   - Reference command dictionary
   - Verify parameters

2. VERIFY command is appropriate
   - Check spacecraft mode
   - Check constraints
   - Two-person verification

3. PREPARE command
   - Build command packet
   - Verify checksum
   - Stage for transmission

4. TRANSMIT command
   - Wait for uplink window
   - Send command
   - Note timestamp

5. VERIFY execution
   - Wait for acknowledgment
   - Check telemetry for effect
   - Log result

6. DOCUMENT
   - Log command sent
   - Log response received
   - Note any anomalies
```

### 6.4 Command Dictionary Reference

| Command | Code | Parameters | Response |
|---------|------|------------|----------|
| GET_TLM | 0x10 | None | TLM packet |
| SET_MODE | 0x20 | mode_id | ACK/NAK |
| SET_BEACON | 0x30 | period_s | ACK |
| SET_TXPWR | 0x40 | power_level | ACK |
| RESET | 0x50 | type | ACK then reset |
| GET_LOG | 0x60 | count | Log entries |

---

## 7. Module 6: Anomaly Response

### 7.1 Learning Objectives

After completing this module, operators will be able to:
- Classify anomalies by severity
- Execute initial response procedures
- Escalate appropriately
- Document and close anomalies

### 7.2 Anomaly Classification

| Severity | Description | Response Time |
|----------|-------------|---------------|
| Critical | Mission-threatening | Immediate |
| Major | Affects primary mission | <4 hours |
| Minor | Degraded but operable | <24 hours |
| Observation | Worth noting | Next review |

### 7.3 Common Anomaly Responses

**Loss of Communication**:
1. Verify ground station function
2. Check orbit prediction
3. Wait for next pass
4. Check for beacon on backup frequency
5. Escalate if no contact in 24 hours

**Unexpected Safe Mode Entry**:
1. Collect all available telemetry
2. Review for cause (low battery, fault, etc.)
3. Determine if spacecraft-initiated or commanded
4. Plan recovery sequence
5. Brief Mission Director before recovery

### 7.4 Anomaly Report Template

```
SMART-QSO ANOMALY REPORT

Anomaly ID: ANO-XXXX
Date/Time: YYYY-MM-DD HH:MM UTC
Reported By: [Name]
Severity: [Critical/Major/Minor/Observation]

DESCRIPTION:
[What was observed]

IMMEDIATE ACTIONS TAKEN:
[List actions]

ROOT CAUSE (if known):
[Analysis]

CORRECTIVE ACTIONS:
[Plan to resolve]

STATUS: [Open/Investigating/Resolved/Closed]
```

---

## 8. Module 7: Emergency Procedures

### 8.1 Emergency Procedure Index

| Code | Emergency | Reference |
|------|-----------|-----------|
| E-001 | Loss of communication | Section 8.2 |
| E-002 | Safe mode recovery | Section 8.3 |
| E-003 | Battery emergency | Section 8.4 |
| E-004 | Thermal emergency | Section 8.5 |

### 8.2 E-001: Loss of Communication

**Symptoms**: No beacon received for 3+ consecutive passes

**Procedure**:
1. [ ] Verify ground station functional (loopback test)
2. [ ] Verify antenna pointing and tracking
3. [ ] Check for beacon on all known frequencies
4. [ ] Review last known telemetry for clues
5. [ ] Coordinate with other ground stations
6. [ ] Escalate to Mission Director
7. [ ] Prepare recovery commanding plan

### 8.3 E-002: Safe Mode Recovery

**Symptoms**: Spacecraft in SAFE mode unexpectedly

**Procedure**:
1. [ ] Collect and review all telemetry
2. [ ] Identify trigger cause from telemetry
3. [ ] Verify battery SOC >30%
4. [ ] If ready, command IDLE mode
5. [ ] Monitor for 2+ orbits
6. [ ] If stable, command ACTIVE mode
7. [ ] Document root cause

### 8.4 E-003: Battery Emergency

**Symptoms**: SOC <20% or voltage <7.0V

**Procedure**:
1. [ ] Confirm spacecraft in SAFE mode
2. [ ] DO NOT command any power-intensive ops
3. [ ] Monitor battery recovery (should be positive power)
4. [ ] Wait for SOC >40% before any commanding
5. [ ] Review what caused the depletion
6. [ ] Adjust operations to prevent recurrence

---

## 9. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
