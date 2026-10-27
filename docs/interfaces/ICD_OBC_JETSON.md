# Interface Control Document: OBC-Jetson

**Document Number**: SMART-QSO-ICD-002
**Version**: 1.2
**Date**: 2026-10-27
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

*Rev 1.1 (2026-10-15): this section is now implemented on both sides, and both test suites check the same golden lines. The OBC side is `software/rust/sqso-protocol/src/jetson.rs` (`test_jetson_link.c`). The Jetson side is `software/payload/obc_link.py`, used by `health_interface.py`, `agent.py` and `supervisor.py` (`test_obc_link.py`). Before this revision the OBC sent CSV with no CRC, the health interface expected binary 0x7E/0x7F frames carrying JSON, and the agent expected JSON lines (finding SW-24). Since 2026-10-24 both sides run one implementation: `obc_link.py` calls the same Rust code through the `sqso_protocol` Python module (`software/rust/sqso-py/src/lib.rs`).*

### 4.1 Line Format

Every message is one line:

```
<TYPE>,<field>,...,<field>,<CRC>\n
```

| Rule | Value |
|------|-------|
| Characters | Printable ASCII 0x20–0x7E. The terminator is a single LF (0x0A) |
| Length | At most 512 bytes, including the CRC field and the LF |
| CRC | Four upper-case hex digits of CRC-16/CCITT-FALSE (§4.4) over every byte before the last comma |
| Fields | Separated by commas, and a field never contains a comma. The one exception is the BEACON_RSP text (§4.3), which runs from after the sequence number to the last comma |
| Receiver | Bad CRC: count it and drop the line (the Jetson answers `NACK,CRC`). Malformed line: count it and drop it (`NACK,FORMAT`). An overlong line is discarded up to its LF |

### 4.2 Message Types

| Type | Direction | Fields |
|------|-----------|--------|
| TELEMETRY | OBC → Jetson | time_ms, SUNLIT/ECLIPSE, soc (0.000–1.000), power_mode (0 SAFE, 1 IDLE, 2 ACTIVE), power_limit_w, then any number of `KEY=VALUE` fields. Also serves as the heartbeat |
| BEACON_REQ | OBC → Jetson | seq (0–65535), max_len (1–176) |
| STATUS_REQ | OBC → Jetson | none |
| SHUTDOWN | OBC → Jetson | reason |
| BEACON_RSP | Jetson → OBC | seq (matching the request), text (1–max_len printable ASCII characters, without a call sign; the OBC adds `de <CALL>:` and the telemetry) |
| STATUS | Jetson → OBC | health (0 OK, 1 WARNING, 2 ERROR, 3 CRITICAL), temp_c (−128–200), power_mw (0–100000), mem_pct (0–100), uptime_s |
| ACK | Jetson → OBC | type acknowledged |
| NACK | Jetson → OBC | reason (`CRC`, `FORMAT`) |

The OBC uses a beacon text only if its sequence number matches the latest request. The OBC's content checks (finding SW-12) apply to every text before it is transmitted.

### 4.3 Golden Lines

Both test suites check these exact bytes:

```
TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON,9C86
BEACON_REQ,42,176,5593
BEACON_RSP,42,Hello, world from orbit,0784
STATUS,0,45,6800,37,1234,D775
STATUS_REQ,5D03
SHUTDOWN,POWER,05D4
ACK,BEACON_RSP,275C
NACK,CRC,49EE
```

### 4.4 CRC Calculation

CRC-16/CCITT-FALSE (also catalogued as CRC-16/IBM-3740):

- Polynomial: 0x1021
- Initial value: 0xFFFF
- Input/output reflected: No
- Final XOR: none
- Check value for `123456789`: 0x29B1

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


## 9. CDR Update (2026-10-27): Interface as Built

Sections 2, 3 and 6.3 are the 2026-01 preliminary interface. Where they disagree with this section, this section, §4
(exact since 1.1) and `docs/hardware/PAYLOAD_DESIGN.md` §11 govern.

| Item | As built (flight software and provisional firmware pin map) | Was |
|------|--------------------------------------------------------------|-----|
| Link | USART1, PA9 TX / PA10 RX, 115200 8N1, 3.3 V CMOS, interrupt-driven receive buffer (`firmware/src/uart.rs`) | same |
| Power | VBAT through SW1 (`PAYLOAD_EN`, PB4) and the eFuse U3 (`JETSON_PWR`, PA2), then the carrier's own converter (`docs/hardware/EPS_DESIGN.md` v2.0) | 5 V EPS rail and a Power Enable pin on the connector |
| Power limit | 8 W carrier input in operation; software trips at 12 W (one sample) and > 8 W (two samples); eFuse 1.2 A | 15 W peak |
| Connector pin 4 | not a power enable: the switches are on the EPS side. Use it as a second ground or the carrier's power-good to the OBC (open) | Power Enable |
| Switch-on | gates of BASELINE §5 and the commissioning hold, then both switches together; the OBC waits for STATUS before BEACON_REQ | EPS 5 V, then enable after 100 ms |
| Switch-off | both lines low together (recommended order and readback: PAYLOAD_DESIGN §11.1, RID AV-06); no SHUTDOWN message is sent, the Jetson must tolerate a power cut (read-only root, writable data partition: `hardware/payload-jetson/README.md`) | SHUTDOWN, 5 s, enable low |
| Fallback | a template beacon whenever no valid BEACON_RSP arrives in time or the text fails the filter; the burst schedule cycles the rail on every burst (165 power-ons in the 14-day SIL run), so no reconnection counter is kept | reconnect after 60 s, power-cycle after 3 failures |

**Back-powering (RID AV-09).** USART1 is enabled for the whole mission, so the OBC's TX line idles high (3.3 V) into a
carrier that is unpowered most of the time. Through the module's input protection diodes that can partly power the
module's I/O rail, hold it in an undefined state at switch-on, and add a few milliamperes of load. Fix at the EM, one
of: an isolating buffer on the carrier powered from its own 3.3 V (a part with I_off, e.g. 74LVC1T45 class) in each
direction; or the firmware switching PA9 to an analog input while `JETSON_PWR` is low and back to the USART function
after power-good. Until then the interface is not verified for power-off isolation (test: TX idle with the carrier off,
measure the carrier's 3.3 V rail; pass below 0.3 V).

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
| 1.1 | 2026-10-15 | Funding audit | §4 made exact (line rules, 512-byte limit, STATUS_REQ, STATUS fields, BEACON_RSP text, NACK reasons, golden lines) and implemented on both sides (SW-24) |
| 1.2 | 2026-10-27 | SMART-QSO Team (CDR avionics review) | §9 new: interface as built (power through SW1 and U3 from VBAT, 8 W limit, connector pin 4, switch-on/off, fallback); back-powering through the idle TX line (RID AV-09) |
