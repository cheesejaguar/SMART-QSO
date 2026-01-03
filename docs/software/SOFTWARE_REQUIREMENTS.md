# Software Requirements Specification (SRS)

**Document Number**: SMART-QSO-SRS-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

**Governing Standards**:
- NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements
- NPR 7150.2 Rev B - NASA Software Engineering Requirements

---

## 1. Introduction

### 1.1 Purpose

This Software Requirements Specification (SRS) defines the functional, performance, interface, and quality requirements for the SMART-QSO flight software.

### 1.2 Scope

This document covers requirements for:
- OBC Flight Software
- Jetson AI Payload Software
- Ground Support Software

### 1.3 Requirements Identification

Requirements are identified as:
- **SRS-Fxxx**: Functional requirements
- **SRS-Pxxx**: Performance requirements
- **SRS-Ixxx**: Interface requirements
- **SRS-Qxxx**: Quality requirements
- **SRS-Sxxx**: Safety requirements

---

## 2. System Overview

### 2.1 System Context

```
                    +------------------+
                    |   Ground Station |
                    +--------+---------+
                             |
                         RF Link
                             |
+--------------------------------------------------+
|                    SMART-QSO CubeSat             |
|  +------------+   UART   +------------------+    |
|  |   OBC      |<-------->| Jetson AI Payload|    |
|  | (FreeRTOS) |          | (Linux)          |    |
|  +-----+------+          +------------------+    |
|        |                                         |
|   +----+----+----+----+                          |
|   |    |    |    |    |                          |
|   v    v    v    v    v                          |
|  EPS  ADCS  RF  Sensors Deployment               |
+--------------------------------------------------+
```

### 2.2 Software Functions

1. **Electrical Power Subsystem (EPS) Control**
2. **Attitude Determination and Control (ADCS)**
3. **RF Beacon Transmission**
4. **AI-based Beacon Generation**
5. **Telemetry Collection and Transmission**
6. **Fault Detection, Isolation, and Recovery (FDIR)**

---

## 3. Functional Requirements

### 3.1 System Initialization

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F001 | The software shall initialize all subsystems within 30 seconds of power-on | T | High |
| SRS-F002 | The software shall load configuration from non-volatile memory on startup | T | High |
| SRS-F003 | The software shall perform self-test and report status via beacon | T | High |
| SRS-F004 | The software shall increment reset counter on each boot | T | Medium |
| SRS-F005 | The software shall log boot reason (power-on, watchdog, commanded) | T | Medium |

### 3.2 EPS Control

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F010 | The software shall monitor battery state of charge (SOC) | T | High |
| SRS-F011 | The software shall control power to subsystems via load switches | T | High |
| SRS-F012 | The software shall implement power modes: SAFE, IDLE, ACTIVE | T | High |
| SRS-F013 | The software shall transition to SAFE mode when SOC < 25% | T | High |
| SRS-F014 | The software shall transition to IDLE mode when SOC < 40% | T | High |
| SRS-F015 | The software shall enable payload only when SOC > 55% and sunlit | T | High |
| SRS-F016 | The software shall enforce power budget limits per mode | T | Medium |
| SRS-F017 | The software shall persist EPS state across resets | T | Medium |

### 3.3 Beacon Transmission

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F020 | The software shall transmit AX.25 beacon at configurable intervals | T | High |
| SRS-F021 | The software shall include callsign in all beacon transmissions | I | High |
| SRS-F022 | The software shall include mission status in beacon info field | T | High |
| SRS-F023 | The software shall transmit AI-generated text when payload is active | T | High |
| SRS-F024 | The software shall fallback to template text when payload is inactive | T | High |
| SRS-F025 | The software shall limit beacon info field to 256 bytes | T | Medium |

### 3.4 AI Payload Control

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F030 | The software shall send health telemetry to Jetson via UART | T | High |
| SRS-F031 | The software shall receive beacon text from Jetson via UART | T | High |
| SRS-F032 | The software shall enforce timeout on Jetson communication | T | High |
| SRS-F033 | The software shall disable payload on communication timeout | T | High |
| SRS-F034 | The software shall validate received beacon text (ASCII, length) | T | High |
| SRS-F035 | The software shall monitor Jetson power consumption | T | Medium |

### 3.5 Fault Management

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F040 | The software shall detect and log all fault conditions | T | High |
| SRS-F041 | The software shall maintain fault log in non-volatile memory | T | High |
| SRS-F042 | The software shall implement autonomous fault recovery | T | High |
| SRS-F043 | The software shall enter SAFE mode on critical faults | T | High |
| SRS-F044 | The software shall attempt recovery from SAFE mode when SOC improves | T | Medium |
| SRS-F045 | The software shall track fault statistics across resets | T | Medium |

### 3.6 Watchdog

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F050 | The software shall kick hardware watchdog at least every 30 seconds | T | High |
| SRS-F051 | The software shall save state before watchdog reset | T | High |
| SRS-F052 | The software shall detect watchdog reset and log occurrence | T | Medium |

### 3.7 Data Persistence

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F060 | The software shall persist mission data to non-volatile memory | T | High |
| SRS-F061 | The software shall protect persisted data with CRC32 | T | High |
| SRS-F062 | The software shall detect and handle corrupted data | T | High |
| SRS-F063 | The software shall maintain mission statistics across resets | T | Medium |

### 3.8 Telemetry

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-F070 | The software shall collect telemetry from all sensors | T | High |
| SRS-F071 | The software shall sample sensors at configurable rates | T | Medium |
| SRS-F072 | The software shall validate sensor readings against limits | T | High |
| SRS-F073 | The software shall include telemetry in beacon transmissions | T | High |

---

## 4. Performance Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-P001 | The main control loop shall execute within 20 ms | T | High |
| SRS-P002 | Beacon transmission shall complete within 5 seconds | T | High |
| SRS-P003 | Fault detection shall occur within 100 ms of condition | T | High |
| SRS-P004 | Mode transition shall complete within 1 second | T | Medium |
| SRS-P005 | UART communication timeout shall be configurable (default 5s) | T | Medium |
| SRS-P006 | Sensor polling shall meet configured period within 10% | T | Medium |
| SRS-P007 | Data persistence shall complete within 500 ms | T | Medium |
| SRS-P008 | System initialization shall complete within 30 seconds | T | High |

---

## 5. Interface Requirements

### 5.1 UART Interface (OBC-Jetson)

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-I001 | The UART interface shall operate at 115200 baud, 8N1 | T | High |
| SRS-I002 | The UART shall use LVTTL voltage levels (3.3V) | I | High |
| SRS-I003 | Telemetry messages shall be formatted as CSV text | T | High |
| SRS-I004 | Beacon text responses shall be ASCII with newline terminator | T | High |
| SRS-I005 | Message format shall include CRC for integrity checking | T | Medium |

### 5.2 Sensor Interfaces

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-I010 | The software shall interface with EPS via I2C | T | High |
| SRS-I011 | The software shall read battery voltage, current, and temperature | T | High |
| SRS-I012 | The software shall read solar panel voltage and current | T | High |

### 5.3 RF Interface

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-I020 | The software shall control beacon transmitter via GPIO | T | High |
| SRS-I021 | The software shall format AX.25 UI frames | T | High |
| SRS-I022 | The software shall use AFSK 1200 bps modulation | T | High |

---

## 6. Quality Requirements

### 6.1 Reliability

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-Q001 | The software shall operate continuously for minimum 1 year | A | High |
| SRS-Q002 | The software shall recover from single-bit memory errors | T | Medium |
| SRS-Q003 | MTBF for software-induced failures shall exceed 30 days | A | Medium |

### 6.2 Maintainability

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-Q010 | The software shall be modular with defined interfaces | I | High |
| SRS-Q011 | Code coverage shall exceed 80% for unit tests | A | High |
| SRS-Q012 | All public functions shall have Doxygen documentation | I | Medium |

### 6.3 Portability

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-Q020 | The software shall compile on ARM and x86 platforms | T | High |
| SRS-Q021 | Hardware dependencies shall be isolated in HAL layer | I | High |
| SRS-Q022 | The software shall support both simulation and flight builds | T | High |

---

## 7. Safety Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| SRS-S001 | The software shall not enable RF transmission until after separation | T | High |
| SRS-S002 | The software shall disable all loads in SAFE mode except beacon | T | High |
| SRS-S003 | The software shall implement hardware watchdog | T | High |
| SRS-S004 | The software shall validate all commands before execution | T | High |
| SRS-S005 | The software shall limit power consumption to prevent thermal damage | T | High |

---

## 8. Constraints

### 8.1 Design Constraints

| ID | Constraint |
|----|------------|
| SRS-C001 | OBC code shall be written in C conforming to C11 standard |
| SRS-C002 | No dynamic memory allocation after initialization |
| SRS-C003 | Stack size shall not exceed 8 KB |
| SRS-C004 | Total code size shall not exceed 256 KB |

### 8.2 External Constraints

| ID | Constraint |
|----|------------|
| SRS-C010 | Beacon transmissions shall comply with amateur radio regulations |
| SRS-C011 | RF transmissions shall not occur until deployment timer expires |
| SRS-C012 | Callsign shall be included in all RF transmissions |

---

## 9. Traceability

### 9.1 Parent Requirements

| SRS ID | Parent (SYS) | Description |
|--------|--------------|-------------|
| SRS-F012 | SYS-020 | Power mode implementation |
| SRS-F020 | SYS-030 | Beacon transmission |
| SRS-F030 | SYS-040 | AI payload interface |

### 9.2 Child Traceability

See Requirements Traceability Matrix (RTM) document for complete traceability to design and code.

---

## Appendix A: TBDs/TBRs

| ID | Item | Resolution Date |
|----|------|-----------------|
| TBD-001 | Exact beacon interval range | PDR |
| TBD-002 | Uplink command set definition | CDR |
| TBR-001 | Watchdog timeout value (currently 30s) | PDR |

---

## Appendix B: Verification Methods

| Code | Method | Description |
|------|--------|-------------|
| A | Analysis | Mathematical or logical analysis |
| I | Inspection | Visual examination of documentation or hardware |
| D | Demonstration | Operational demonstration of function |
| T | Test | Execution with specific inputs and measured outputs |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
