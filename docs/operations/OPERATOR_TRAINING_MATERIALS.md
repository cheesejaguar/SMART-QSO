# SMART-QSO Operator Training Materials

**Document ID**: SQSO-OTM-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft training material; no training has been held. `docs/BASELINE_PARAMETERS.md` governs every value

> **Funding audit notice (2026-10-16, finding PI-14), updated by the documentation audit (2026-10-27).** The spacecraft
> figures, telemetry tables and orbit of Modules 1–5 were corrected to the baseline in Version 1.1: 4S1P ≈ 50 Wh
> battery (14.4 V nominal), 16-cell array, STM32L4R5ZI OBC, 0.5 W nominal and 1 W maximum, 475 km SSO. The block diagrams
> are schematic. For telemetry use `docs/operations/TELEMETRY_DICTIONARY.md` §4 (beacon tail, HK1, HK2), for commands
> `docs/operations/COMMAND_DICTIONARY.md` §1.1, for procedures `docs/operations/COMMISSIONING_PLAN.md` and
> `docs/operations/CONTINGENCY_PROCEDURES.md`.

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
│   │ 7W solar    │◄─►│  STM32L4R5  │◄─►│  Si4463     │      │
│   │ 7Wh battery │   │ Rust/Embassy│   │  VHF/UHF    │      │
│   └─────────────┘   └──────┬──────┘   └─────────────┘      │
│                            │                                │
│                     ┌──────┴──────┐                         │
│                     │   Jetson    │                         │
│                     │  Orin Nano  │                         │
│                     │  (AI)       │                         │
│                     └─────────────┘                         │
│                                                             │
│   Mass: CBE 1.52 kg   Load: 1.86-2.26 W     Life: 2 years  │
└─────────────────────────────────────────────────────────────┘
```

**Key Subsystems**:

| Subsystem | Function | Key Parameters |
|-----------|----------|----------------|
| EPS | Power generation/storage | 16 cells, 2.48 W tumbling / 3.84 W sun-pointed EOL orbit average; 4S1P ≈ 50 Wh battery |
| OBC | Command and control | STM32L4R5ZI MCU; Rust flight software on the Embassy executor, no RTOS (BASELINE §6, 2026-10-25) |
| RF | Communications | FM transponder and 1200 bps beacon, 145.8–146.0 MHz down, 435 MHz up; 0.5 W nominal, 1 W maximum |
| AI | Template selector on the OBC (flight baseline); Jetson text (gated experiment) | Jetson Orin Nano 8 GB, 7 W mode; SmolLM2-360M or Qwen3-0.6B (D5) |
| ADCS | Detumble and spin-stabilised sun-pointing (+Y at 8°/s) | Magnetometer, sun sensors, M-G370 gyro, CR0002 rods |

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
 16 cells             4S1P 18650            3.3V, 5V_PA rails
 2.48-3.84 W EOL      ≈ 50 Wh               Load switches
```

**Key Telemetry Points**:

| Parameter | Nominal Range | Warning | Critical |
|-----------|---------------|---------|----------|
| Battery Voltage (pack) | 13.6-16.4 V | <13.6 V (load shed) | <11.2 V (BMS cut-off) |
| Battery Current | ±500mA | ±1A | ±2A |
| SOC | 40-100% | <40% (ACTIVE → IDLE) | <25% (SAFE) |
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
│                    STM32L4R5ZI (Cortex-M4F)                 │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐         │
│  │  Core   │  │  SRAM   │  │  Flash  │  │  FRAM   │         │
│  │  80MHz  │  │  640KB  │  │  2MB    │  │  256KB  │         │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘         │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Embassy executor (Rust), no RTOS: one task           │  │
│  │  Main loop every 20 ms, stages in order:              │  │
│  │  mission state, health, Jetson TLM, payload,          │  │
│  │  beacon, uplink, timers, persistence, watchdog        │  │
│  │  UARTs interrupt-driven (Jetson, TNC, debug)          │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  Watchdog: IWDG, 30-second timeout                          │
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
| Watchdog timeout | Main loop hung: no IWDG refresh for 30 s | Reset, increment counter |
| Low battery | SOC <25% | Transition to SAFE (EMERGENCY below 15 %) |
| Over-temperature | Temp >50°C | Throttle, then SAFE |
| Stage stalled | A stage misses 3 check-ins in a row (checked every 1 s) | Stall logged, state saved; IWDG not refreshed, so the OBC resets within 30 s |

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
│  │  SmolLM2-360M or Qwen3-0.6B, INT4 (experiment)   │   │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐ │   │
│  │  │  Context   │─►│  Inference │─►│   Output   │ │   │
│  │  │  (TLM)     │  │  (TensorRT)│  │  (Beacon)  │ │   │
│  │  └────────────┘  └────────────┘  └────────────┘ │   │
│  └──────────────────────────────────────────────────┘   │
│                                                          │
│  Burst ≤10 min, ≤3.6 kJ; ≥100 min off; ≤8 W input      │
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
| Downlink Frequency | One IARU-coordinated frequency in 145.800–146.000 MHz (not 145.825) |
| Uplink Frequency | One IARU-coordinated frequency requested in 435.222–435.778 MHz, else 436.222–436.500 MHz |
| TX Power | 0.5 W nominal, 1 W maximum (`SET_POWER`) |
| Modulation | FM transponder (16K0F3E); 1200 bps AFSK (10K4F2D) |
| Protocol | AX.25 UI |
| Beacon Period | 60 s ACTIVE, 120 s IDLE, 180 s SAFE (±5 s jitter) |

**Beacon Structure**:
```
AX.25 BEACON FRAME

┌─────────┬─────────┬─────────┬─────────┬─────────────────┐
│ Flag    │ Address │ Control │ PID     │ Information     │
│ 0x7E    │ Callsign│ 0x03    │ 0xF0    │ Beacon content  │
└─────────┴─────────┴─────────┴─────────┴─────────────────┘

Information Field:
de KK6GDB-11: <text> | T=… M=… SOC=… BV=… SUN=… RF=… QSO=…   (BEACON_SPEC §3)
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
- Altitude: 475 km target, 465–485 km (BASELINE §3)
- Inclination: ≈ 97.3° (sun-synchronous, LTAN 22:30)
- Period: 94.1 minutes
- Passes per day: about 3.8 above 5° from the Bay Area, 3.0 reaching 10° (SQSO-MTL-001 §10)

**Pass Planning Procedure**:
1. Load current TLE from Space-Track
2. Predict passes for next 24 hours
3. Identify passes reaching 10° (commanding; 12° in the degraded station mode, decision C-14)
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
| VB (pack) | >13.6 V | 11.2-13.6 V | <11.2 V |
| SOC | >40% | 25-40% | <25% |
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
3. [ ] Verify battery SOC >40%
4. [ ] If ready, command IDLE mode (`SET_MODE` 1)
5. [ ] Monitor for 2+ orbits
6. [ ] If stable, command ACTIVE mode
7. [ ] Document root cause

### 8.4 E-003: Battery Emergency

**Symptoms**: SOC <25% (SAFE) or <15% (EMERGENCY, no beacon), or pack voltage <13.6 V (load shed)

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
| 1.0a | 2026-10-16 | Funding audit | Notice: Module 1 spacecraft figures superseded by BASELINE_PARAMETERS (PI-14) |
| 1.0, note | 2026-10-24 | SMART-QSO Team | OBC row updated: STM32L4R5ZI, Rust on the Embassy executor. The diagrams in §2 and §4 still show the superseded Apollo4 and FreeRTOS tasks; `docs/BASELINE_PARAMETERS.md` and `software/flight/docs/RTOS_ARCHITECTURE.md` govern |
| 1.0, note | 2026-10-25 | SMART-QSO Team | Embassy executor, no RTOS, is the baseline (BASELINE §6, CR-2026-002): OBC in the §2.3 and §3.3 diagrams (part, memory, one task, 30 s IWDG) and the watchdog and stage-stall faults in §3.3 |
| 1.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): Module 1–5 values to BASELINE Rev E (4S ≈ 50 Wh battery and pack-voltage limits, 16-cell array, mass, 0.5/1 W, frequencies, beacon intervals, 475 km SSO, SOC thresholds 40/25/15 %, the D5 models and burst rules); commanding from 10°; notice updated; status (no training held) |
