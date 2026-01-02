# Interface Control Document: Beacon Format

**Document Number**: SMART-QSO-ICD-006
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

---

## 1. Introduction

### 1.1 Purpose

This Interface Control Document (ICD) defines the beacon frame format for the SMART-QSO CubeSat, including AX.25 protocol implementation and information field content.

### 1.2 Scope

This document covers:
- AX.25 frame structure
- Beacon information field format
- Telemetry encoding
- AI-generated text integration

### 1.3 Regulatory Compliance

All beacon transmissions comply with:
- ITU Radio Regulations
- FCC Part 97 (Amateur Radio)
- IARU coordination requirements

---

## 2. AX.25 Frame Structure

### 2.1 UI Frame Format

SMART-QSO uses AX.25 Unnumbered Information (UI) frames for beacon transmission:

```
+-------+------+------+------+------+---------+-----+
| Flag  | Dest | Src  | Digi | Ctrl | Info    | FCS |
| 0x7E  | Addr | Addr | Path |      | Field   |     |
+-------+------+------+------+------+---------+-----+
  1B      7B     7B    0-56B   1B    0-256B    2B
```

### 2.2 Flag Field

| Field | Value | Description |
|-------|-------|-------------|
| Flag | 0x7E | Frame delimiter |

### 2.3 Address Fields

#### Destination Address (7 bytes)

| Byte | Content | Description |
|------|---------|-------------|
| 0-5 | "CQ    " | Destination callsign (space-padded) |
| 6 | 0xE0 | SSID byte (SSID=0, C-bit=1, reserved=11) |

#### Source Address (7 bytes)

| Byte | Content | Description |
|------|---------|-------------|
| 0-5 | Callsign | SMART-QSO callsign (e.g., "KK6XXX") |
| 6 | 0x61 | SSID byte (SSID=0, C-bit=0, reserved=11, last=1) |

Note: Callsign must be obtained from FCC/IARU coordination

### 2.4 Control Field

| Field | Value | Description |
|-------|-------|-------------|
| Control | 0x03 | UI frame (Unnumbered Information) |

### 2.5 PID Field

| Field | Value | Description |
|-------|-------|-------------|
| PID | 0xF0 | No layer 3 protocol |

### 2.6 Frame Check Sequence

16-bit CRC-CCITT (polynomial 0x1021, initial 0xFFFF)

---

## 3. Information Field Format

### 3.1 Structure Overview

```
+--------+-------+--------+-----------+
| Header | Telem | Delim  | Text      |
| 16B    | 64B   | 1B     | 0-175B    |
+--------+-------+--------+-----------+
         Max 256 bytes total
```

### 3.2 Header Section (16 bytes)

| Offset | Length | Field | Format |
|--------|--------|-------|--------|
| 0 | 8 | Callsign | ASCII, space-padded |
| 8 | 4 | Seq Num | 4-digit decimal |
| 12 | 4 | Timestamp | HHMM UTC |

Example:
```
KK6XXX  00420735
```

### 3.3 Telemetry Section (64 bytes max)

| Field | Format | Example |
|-------|--------|---------|
| Battery Voltage | BV=X.XV | BV=7.8V |
| Battery Current | BI=XmA | BI=-500mA |
| Battery Temp | BT=X.XC | BT=22.5C |
| SOC | SOC=XX% | SOC=68% |
| Solar Voltage | SV=X.XV | SV=9.2V |
| Solar Current | SI=XmA | SI=450mA |
| Bus Voltage | BUSV=X.XV | BUSV=5.0V |
| Mode | M=X | M=2 |
| Uptime | UP=Xs | UP=86400s |
| Reset Count | RC=X | RC=3 |

Telemetry fields are comma-separated:
```
BV=7.8V,BT=22.5C,SOC=68%,SV=9.2V,M=2,UP=86400s,RC=3
```

### 3.4 Delimiter

| Field | Value | Description |
|-------|-------|-------------|
| Delimiter | `|` (0x7C) | Separates telemetry from text |

### 3.5 Text Section (up to 175 bytes)

Contains either:
1. AI-generated beacon text (when payload active)
2. Fallback template text (when payload inactive)

#### AI-Generated Text Requirements

| Requirement | Value |
|-------------|-------|
| Character set | ASCII printable (0x20-0x7E) |
| Max length | 175 bytes |
| Line endings | None (single line) |
| Content | Mission status, greetings, technical info |

#### Fallback Templates

| Template ID | Content |
|-------------|---------|
| 0 | "SMART-QSO CubeSat - AI Beacon Demo. Payload inactive. 73!" |
| 1 | "Hello from SMART-QSO! Autonomous AI beacon experiment. 73!" |
| 2 | "SMART-QSO: Experimenting with onboard AI for ham radio. 73!" |
| 3 | "CQ CQ from SMART-QSO satellite. AI payload in standby. 73!" |

---

## 4. Complete Frame Examples

### 4.1 Example: AI Beacon (Active Mode)

```
Frame:
7E                              ; Flag
43 51 20 20 20 20 E0            ; Dest: CQ
4B 4B 36 58 58 58 61            ; Src: KK6XXX (example)
03                              ; Control: UI
F0                              ; PID: No L3

; Information Field (186 bytes):
KK6XXX  00420735                ; Header (16B)
BV=7.8V,BT=22.5C,SOC=68%,SV=9.2V,SI=450mA,M=2,UP=86400s,RC=3
|                               ; Delimiter
Greetings from SMART-QSO! Currently over the Pacific at 420km altitude.
Solar panels generating 4.2W. AI inference active with TinyLM model.
Send reports to [email]. 73 de KK6XXX!

XX XX                           ; FCS (CRC-16)
7E                              ; Flag
```

### 4.2 Example: Fallback Beacon (Safe Mode)

```
Frame:
7E                              ; Flag
43 51 20 20 20 20 E0            ; Dest: CQ
4B 4B 36 58 58 58 61            ; Src: KK6XXX
03                              ; Control: UI
F0                              ; PID: No L3

; Information Field (112 bytes):
KK6XXX  00430623                ; Header (16B)
BV=6.8V,BT=15.0C,SOC=22%,SV=0.0V,SI=0mA,M=0,UP=3600s,RC=5
|                               ; Delimiter
SMART-QSO CubeSat - AI Beacon Demo. Payload inactive. 73!

XX XX                           ; FCS (CRC-16)
7E                              ; Flag
```

---

## 5. Encoding

### 5.1 AFSK Modulation

| Parameter | Value |
|-----------|-------|
| Modulation | AFSK |
| Bit rate | 1200 bps |
| Mark frequency | 1200 Hz |
| Space frequency | 2200 Hz |
| Pre-amble | 50 flags (0x7E) |
| Post-amble | 10 flags (0x7E) |

### 5.2 NRZI Encoding

AX.25 uses NRZI (Non-Return-to-Zero Inverted):
- 0 bit: Transition
- 1 bit: No transition

### 5.3 Bit Stuffing

After five consecutive 1-bits, insert one 0-bit (except in flag bytes).

---

## 6. Timing

### 6.1 Beacon Interval

| Mode | Interval | Requirement |
|------|----------|-------------|
| SAFE | 60 s | SRS-F020 |
| IDLE | 30 s | SRS-F020 |
| ACTIVE | 30 s | SRS-F020 |

### 6.2 Transmission Timing

| Parameter | Value |
|-----------|-------|
| TX preamble | 500 ms |
| Frame duration | ~1.5 s (typical) |
| TX tail | 100 ms |
| Total TX time | < 5 s |

---

## 7. Software Interface

### 7.1 Beacon Generation Function

```c
/**
 * @brief Build complete beacon frame
 * @param frame Output buffer for AX.25 frame
 * @param frame_len Output: actual frame length
 * @param max_len Maximum buffer size
 * @param telemetry Current telemetry string
 * @param text Beacon text (AI or fallback)
 * @return SMART_QSO_OK on success
 */
SmartQsoResult_t beacon_build_frame(
    uint8_t *frame,
    size_t *frame_len,
    size_t max_len,
    const char *telemetry,
    const char *text
);
```

### 7.2 Telemetry Format Function

```c
/**
 * @brief Format telemetry string for beacon
 * @param buffer Output buffer
 * @param buffer_len Buffer size
 * @param state Current system state
 * @return Bytes written
 */
size_t beacon_format_telemetry(
    char *buffer,
    size_t buffer_len,
    const SystemState_t *state
);
```

### 7.3 CRC-16 Calculation

```c
/**
 * @brief Calculate CRC-16 CCITT for AX.25
 * @param data Input data
 * @param len Data length
 * @return CRC-16 value
 */
uint16_t ax25_crc16(const uint8_t *data, size_t len);
```

---

## 8. Validation

### 8.1 Frame Validation

| Check | Requirement |
|-------|-------------|
| Flag bytes | 0x7E at start and end |
| Address valid | Proper callsign format |
| Control byte | 0x03 (UI) |
| PID byte | 0xF0 |
| Info length | ≤ 256 bytes |
| FCS valid | CRC-16 matches |

### 8.2 Content Validation

| Check | Requirement |
|-------|-------------|
| Header format | Callsign + seq + time |
| Telemetry format | Key=value pairs |
| Text ASCII | Printable characters only |
| Total length | ≤ 256 bytes info field |

---

## 9. Test Requirements

### 9.1 Frame Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| BCN-001 | Frame structure | Valid AX.25 UI frame |
| BCN-002 | CRC calculation | CRC matches reference |
| BCN-003 | Bit stuffing | Correct after 5 ones |
| BCN-004 | NRZI encoding | Correct transitions |

### 9.2 Content Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| BCN-010 | Telemetry format | All fields present |
| BCN-011 | AI text integration | Text included correctly |
| BCN-012 | Fallback selection | Correct template used |
| BCN-013 | Length limits | ≤ 256 bytes |

### 9.3 RF Tests

| Test | Description | Pass Criteria |
|------|-------------|---------------|
| BCN-020 | AFSK modulation | Decode successful |
| BCN-021 | Frequency accuracy | ±50 Hz |
| BCN-022 | Power output | Within spec |
| BCN-023 | Timing | Interval correct |

---

## 10. Ground Station Decoding

### 10.1 Recommended Software

- Direwolf (Linux/Windows)
- MultiPSK (Windows)
- APRS decoding software

### 10.2 Decoding Parameters

```
Modem: AFSK 1200
Mark: 1200 Hz
Space: 2200 Hz
Protocol: AX.25 UI
```

### 10.3 Data Extraction

After decoding AX.25 frame:
1. Parse header (bytes 0-15)
2. Parse telemetry until `|` delimiter
3. Extract text after delimiter
4. Report to ground station network

---

## Appendix A: ASCII Character Set

Information field uses printable ASCII (0x20-0x7E):
- Space through tilde
- No control characters
- No extended ASCII

---

## Appendix B: Telemetry Field Definitions

| Field | Unit | Range | Resolution |
|-------|------|-------|------------|
| BV | Volts | 6.0-8.4 | 0.1 V |
| BI | mA | -2000 to +2000 | 1 mA |
| BT | °C | -20 to +60 | 0.1 °C |
| SOC | % | 0-100 | 1% |
| SV | Volts | 0-12 | 0.1 V |
| SI | mA | 0-1000 | 1 mA |
| BUSV | Volts | 4.5-5.5 | 0.01 V |
| M | - | 0-2 | 1 |
| UP | seconds | 0-2^32 | 1 s |
| RC | count | 0-65535 | 1 |

---

## Appendix C: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
