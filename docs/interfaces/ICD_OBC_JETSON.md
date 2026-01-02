# Interface Control Document: OBC-Jetson

**Document Number**: SMART-QSO-ICD-002
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

---

## 1. Introduction

### 1.1 Purpose

This Interface Control Document (ICD) defines the hardware and software interface between the On-Board Computer (OBC) and the Jetson Orin Nano AI payload.

### 1.2 Scope

This document covers:
- Physical/electrical interface specifications
- Communication protocol
- Message formats
- Timing requirements
- Error handling

### 1.3 Interface Diagram

```
+----------------+                    +------------------+
|                |     UART (3.3V)    |                  |
|      OBC       |<------------------>|     Jetson       |
| (STM32L4/      |     TX/RX/GND      | Orin Nano Super  |
|  Apollo4)      |                    |                  |
|                |     Power Enable   |                  |
|                |------------------->|                  |
|                |     (GPIO)         |                  |
+----------------+                    +------------------+
        |                                    |
        v                                    v
   EPS Control                          Power Gating
   (3.3V, 5V rails)                     Circuit
```

---

## 2. Physical Interface

### 2.1 Connector Specification

| Parameter | OBC Side | Jetson Side |
|-----------|----------|-------------|
| Connector | JST-GH 4-pin | JST-GH 4-pin |
| Pin 1 | GND | GND |
| Pin 2 | UART TX (3.3V) | UART RX (3.3V) |
| Pin 3 | UART RX (3.3V) | UART TX (3.3V) |
| Pin 4 | Power Enable | Power Enable |

### 2.2 Electrical Characteristics

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| Logic High | 2.4 | 3.3 | 3.6 | V |
| Logic Low | 0 | 0 | 0.4 | V |
| Output Current | - | - | 8 | mA |
| Input Current | - | - | 1 | mA |

### 2.3 UART Configuration

| Parameter | Value |
|-----------|-------|
| Baud Rate | 115200 |
| Data Bits | 8 |
| Stop Bits | 1 |
| Parity | None |
| Flow Control | None |

---

## 3. Power Interface

### 3.1 Power Control

The OBC controls Jetson power via:
1. **Power Enable GPIO** - Active high signal to power gating circuit
2. **EPS Load Switch** - 5V rail to Jetson power supply

### 3.2 Power Sequencing

#### Power-On Sequence
```
T=0:      EPS enables 5V rail
T+100ms:  OBC asserts Power Enable
T+500ms:  Jetson begins boot
T+30s:    Jetson ready for communication
```

#### Power-Off Sequence
```
T=0:      OBC sends SHUTDOWN command
T+5s:     OBC de-asserts Power Enable
T+100ms:  EPS disables 5V rail
```

### 3.3 Power Budget

| State | Current (5V) | Power |
|-------|--------------|-------|
| Boot | 2.0 A | 10.0 W |
| Idle | 1.0 A | 5.0 W |
| Inference | 1.5 A | 7.5 W |
| Peak | 3.0 A | 15.0 W |

---

## 4. Communication Protocol

### 4.1 Protocol Overview

The OBC-Jetson protocol uses ASCII-based messages with the following structure:

```
<MSG_TYPE>,<FIELD1>,<FIELD2>,...<CRC16>\n
```

### 4.2 Message Types

| Type | Direction | Description |
|------|-----------|-------------|
| TELEMETRY | OBC → Jetson | Health/status data |
| BEACON_REQ | OBC → Jetson | Request beacon text |
| BEACON_RSP | Jetson → OBC | Beacon text response |
| STATUS | Jetson → OBC | Jetson status report |
| SHUTDOWN | OBC → Jetson | Shutdown command |
| ACK | Both | Acknowledgment |
| NACK | Both | Negative acknowledgment |

### 4.3 Message Formats

#### TELEMETRY (OBC → Jetson)
```
TELEMETRY,<timestamp_ms>,<sunlit>,<soc>,<power_mode>,<power_limit>,<sensors...>,<crc16>\n
```

Example:
```
TELEMETRY,1704153600000,SUNLIT,0.75,2,3.0,BV=8.1V,BT=22.5C,BUSV=5.0V,1A2B\n
```

| Field | Type | Description |
|-------|------|-------------|
| timestamp_ms | uint64 | Milliseconds since epoch |
| sunlit | string | "SUNLIT" or "ECLIPSE" |
| soc | float | State of charge (0.0-1.0) |
| power_mode | int | 0=SAFE, 1=IDLE, 2=ACTIVE |
| power_limit | float | Power limit in watts |
| sensors | key=value | Sensor readings |
| crc16 | hex | CRC-16 CCITT |

#### BEACON_REQ (OBC → Jetson)
```
BEACON_REQ,<seq>,<max_len>,<crc16>\n
```

| Field | Type | Description |
|-------|------|-------------|
| seq | uint16 | Sequence number |
| max_len | uint16 | Maximum beacon length |
| crc16 | hex | CRC-16 CCITT |

#### BEACON_RSP (Jetson → OBC)
```
BEACON_RSP,<seq>,<text>,<crc16>\n
```

| Field | Type | Description |
|-------|------|-------------|
| seq | uint16 | Matching sequence number |
| text | string | ASCII beacon text |
| crc16 | hex | CRC-16 CCITT |

#### STATUS (Jetson → OBC)
```
STATUS,<cpu_temp>,<gpu_temp>,<mem_used>,<uptime>,<crc16>\n
```

| Field | Type | Description |
|-------|------|-------------|
| cpu_temp | float | CPU temperature (°C) |
| gpu_temp | float | GPU temperature (°C) |
| mem_used | uint32 | Memory used (MB) |
| uptime | uint32 | Uptime (seconds) |
| crc16 | hex | CRC-16 CCITT |

### 4.4 CRC Calculation

CRC-16 CCITT with:
- Polynomial: 0x1021
- Initial value: 0xFFFF
- Input/output reflected: No

---

## 5. Timing Requirements

### 5.1 Timing Diagram

```
OBC                                      Jetson
 |                                          |
 |-- TELEMETRY --------------------------->|
 |                                          | (processing)
 |<-- STATUS -----------------------------|
 |                                          |
 |-- BEACON_REQ -------------------------->|
 |                                          | (inference)
 |                                          | (T < 10s)
 |<-- BEACON_RSP --------------------------|
 |                                          |
```

### 5.2 Timing Specifications

| Parameter | Min | Typ | Max | Unit |
|-----------|-----|-----|-----|------|
| TELEMETRY interval | 30 | 60 | 300 | s |
| BEACON_RSP timeout | - | - | 10 | s |
| ACK timeout | - | - | 1 | s |
| Boot time | - | 20 | 30 | s |
| Shutdown time | - | 3 | 5 | s |

### 5.3 Latency Requirements

| Message | Max Latency |
|---------|-------------|
| TELEMETRY → STATUS | 100 ms |
| BEACON_REQ → BEACON_RSP | 10 s |
| SHUTDOWN → power off | 5 s |

---

## 6. Error Handling

### 6.1 Communication Errors

| Error | Detection | Recovery |
|-------|-----------|----------|
| Timeout | Timer expiry | Retry (3x), then fallback |
| CRC mismatch | CRC check | Request retransmission |
| Invalid format | Parse error | Send NACK |
| Buffer overflow | Length check | Truncate and NACK |

### 6.2 Retry Policy

| Condition | Retries | Action on Failure |
|-----------|---------|-------------------|
| No ACK | 3 | Log fault, continue |
| CRC error | 3 | Log fault, use fallback |
| Timeout | 3 | Disable payload |

### 6.3 Fallback Behavior

When Jetson communication fails:
1. OBC uses template beacon text
2. OBC logs communication fault
3. OBC attempts reconnection after 60s
4. After 3 failed reconnections, power cycle Jetson

---

## 7. Operational Modes

### 7.1 Mode Definitions

| Mode | OBC State | Jetson State |
|------|-----------|--------------|
| DORMANT | Running | Powered off |
| BOOT | Running | Booting |
| ACTIVE | Running | Inference active |
| SHUTDOWN | Running | Shutting down |

### 7.2 Mode Transitions

```
         power_on
DORMANT ─────────> BOOT
   ^                  |
   |                  | ready
   |                  v
   |              ACTIVE
   |                  |
   | timeout          | shutdown_cmd
   |                  v
   └──────────── SHUTDOWN
```

---

## 8. Test Requirements

### 8.1 Interface Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| IFT-001 | UART loopback | All bytes received correctly |
| IFT-002 | CRC verification | All CRCs calculated correctly |
| IFT-003 | Timeout handling | Fallback within timeout period |
| IFT-004 | Power sequencing | Correct timing verified |
| IFT-005 | Message parsing | All message types parsed |

### 8.2 Integration Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| INT-001 | End-to-end beacon | Beacon generated and transmitted |
| INT-002 | Power cycling | Jetson recovers after power cycle |
| INT-003 | Fault injection | Fallback behavior correct |

---

## Appendix A: Reference Documents

1. Jetson Orin Nano Developer Kit Carrier Board Specification
2. STM32L4 Reference Manual (RM0351)
3. SMART-QSO System Requirements Specification
4. AX.25 Link Access Protocol for Amateur Packet Radio

---

## Appendix B: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
