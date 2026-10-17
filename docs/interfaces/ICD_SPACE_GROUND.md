# Interface Control Document: Space-to-Ground

**Document ID**: SQSO-ICD-005
**Version**: 1.2
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the interface between the SMART-QSO spacecraft and the Ground Segment per NPR 7123.1 SE Process 5, including RF characteristics, data formats, and operational protocols.

## 2. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-SRS-001 | System Requirements Specification |
| SMART-QSO-ICD-006 | Beacon Format ICD |
| SQSO-GSD-001 | Ground Station Design |
| SQSO-MOP-001 | Mission Operations Plan |
| ITU Radio Regulations | Spectrum allocation |
| IARU Coordination | Amateur satellite frequency coordination |

## 3. Interface Overview

```
SPACE-GROUND INTERFACE

                    ┌─────────────────┐
                    │   SMART-QSO     │
                    │   Spacecraft    │
                    └────────┬────────┘
                             │
            ┌────────────────┼────────────────┐
            │ Downlink       │       Uplink   │
            │ 145.825 MHz    │   435.XXX MHz  │
            │ VHF            │       UHF      │
            ▼                │                ▼
   ┌─────────────────────────┴─────────────────────────┐
   │                    RF Link                         │
   │  - Amateur Radio Band (Part 97)                   │
   │  - AX.25 Protocol                                 │
   │  - Open/Unencrypted                               │
   └─────────────────────────┬─────────────────────────┘
                             │
            ┌────────────────┼────────────────┐
            ▼                │                ▼
    ┌──────────────┐         │        ┌──────────────┐
    │  Primary     │         │        │  Partner     │
    │  Ground Stn  │         │        │  Stations    │
    └──────────────┘         │        └──────────────┘
                             │
                    ┌────────┴────────┐
                    │  Mission Control │
                    │     Center       │
                    └─────────────────┘
```

## 4. RF Interface

### 4.1 Downlink (Space to Ground)

| Parameter | Value | Tolerance | Notes |
|-----------|-------|-----------|-------|
| Frequency | 145.825 MHz | ±1 kHz | VHF band |
| Modulation | GFSK | - | Gaussian filtered |
| Data rate | 9600 bps | - | Primary |
| Alt data rate | 1200 bps | - | Fallback AFSK |
| Deviation | 3.5 kHz | ±10% | |
| Bandwidth | 16 kHz | - | Occupied |
| Power | 0.5-2.0 W | ±0.5 dB | Selectable |
| Polarization | Circular (RHCP) | - | |

### 4.2 Uplink (Ground to Space)

| Parameter | Value | Tolerance | Notes |
|-----------|-------|-----------|-------|
| Frequency | 435.XXX MHz | ±1 kHz | UHF band |
| Modulation | GFSK | - | |
| Data rate | 9600 bps | - | |
| Deviation | 3.5 kHz | ±10% | |
| Min EIRP | 10 dBW | - | Ground station |
| Polarization | Circular (RHCP) | - | |

### 4.3 Link Budget Summary

| Parameter | Downlink | Uplink | Unit |
|-----------|----------|--------|------|
| TX Power | 30 | 40 | dBm |
| TX Antenna | 0 | 12 | dBi |
| Path Loss | -145 | -150 | dB |
| RX Antenna | 12 | 0 | dBi |
| System Noise | 290 | 500 | K |
| Data Rate | 9600 | 9600 | bps |
| Eb/N0 | 15 | 12 | dB |
| Required Eb/N0 | 10 | 10 | dB |
| **Margin** | **5** | **2** | **dB** |

## 5. Protocol Interface

### 5.1 AX.25 Frame Format

```
AX.25 FRAME STRUCTURE

┌────────┬────────────┬────────────┬─────────┬──────────┬─────────┬────────┐
│  Flag  │   Dest     │   Source   │ Control │   PID    │  Info   │  FCS   │
│  0x7E  │  Callsign  │  Callsign  │  0x03   │   0xF0   │ Payload │ CRC-16 │
│ 1 byte │  7 bytes   │  7 bytes   │ 1 byte  │  1 byte  │ N bytes │ 2 bytes│
└────────┴────────────┴────────────┴─────────┴──────────┴─────────┴────────┘
```

### 5.2 Callsign Assignment

| Station | Callsign | SSID | Notes |
|---------|----------|------|-------|
| Spacecraft | SQSO | 0 | Primary |
| Ground Station | SQSO-GS | 0 | Primary ground |
| Broadcast | CQ | 0 | Beacon destination |

### 5.3 Frame Types

| Type | Control | Use |
|------|---------|-----|
| UI Frame | 0x03 | Beacon, telemetry |
| SABM | 0x2F | Connection request |
| UA | 0x63 | Acknowledgment |
| DISC | 0x43 | Disconnect |
| DM | 0x0F | Disconnect mode |

## 6. Data Interface

### 6.1 Beacon Format

```
BEACON PAYLOAD FORMAT (≤200 bytes)

Byte 0:     Frame type (0x01 = telemetry)
Bytes 1-2:  Sequence number (uint16, big-endian)
Bytes 3-6:  Timestamp (Unix epoch, uint32)
Bytes 7-N:  Telemetry data
Bytes N-M:  AI-generated text (if ACTIVE mode)
```

### 6.2 Telemetry Format

| Offset | Field | Type | Units | Description |
|--------|-------|------|-------|-------------|
| 0 | MODE | uint8 | - | Operating mode |
| 1 | BATT_V | uint8 | 0.04V/bit | Battery voltage |
| 2 | BATT_I | int8 | 10mA/bit | Battery current |
| 3 | SOC | uint8 | 1%/bit | State of charge |
| 4 | TEMP_OBC | int8 | 1°C/bit | OBC temperature |
| 5 | TEMP_BAT | int8 | 1°C/bit | Battery temperature |
| 6 | UPTIME | uint16 | 1min/bit | System uptime |
| 8 | BOOT_CNT | uint16 | - | Boot counter |
| 10 | MAG_X | int16 | 1nT/bit | Magnetometer X |
| 12 | MAG_Y | int16 | 1nT/bit | Magnetometer Y |
| 14 | MAG_Z | int16 | 1nT/bit | Magnetometer Z |
| 16 | RATE_X | int8 | 0.1°/s/bit | Rate X |
| 17 | RATE_Y | int8 | 0.1°/s/bit | Rate Y |
| 18 | RATE_Z | int8 | 0.1°/s/bit | Rate Z |
| 19 | FLAGS | uint8 | - | Status flags |

### 6.3 Command Format

*(Rev 1.1: superseded.)* The command frame, authentication, replay counter and TX-OFF command are defined in SQSO-ICD-013, [`TELECOMMAND_AUTHENTICATION.md`](TELECOMMAND_AUTHENTICATION.md), which governs. In summary, each command is the information field of an AX.25 UI frame (PID 0xF0) addressed to the spacecraft:

```
0xAA 0x55 | cmd_id | len | counter (4, BE) | args (N) | tag (16) | sum16 (2, BE)
```

### 6.4 Command Set

*(Rev 1.1: superseded.)* The command dictionary is SQSO-ICD-013 Section 7. It includes `TX_DISABLE` (0x32) and `TX_ENABLE` (0x31), which cease and restore emissions per 47 CFR 97.207(b). The Rev 1.0 table that stood here did not match the flight software and has been removed.

### 6.5 Response Format

```
RESPONSE FRAME FORMAT

Byte 0:     Response type (command echo + 0x80)
Byte 1:     Status (0=OK, else error code)
Bytes 2-N:  Response data
Bytes N-2:  CRC-16
```

## 7. Operational Interface

### 7.1 Pass Timeline

```
TYPICAL PASS TIMELINE

T-5 min:  Ground station preparation
          - Antenna positioned
          - Receiver configured
          - Recording started

AOS:      Acquisition of Signal
          - Lock on beacon
          - Begin telemetry collection

AOS+2min: Command window (if uplink planned)
          - Send queued commands
          - Wait for responses

TCA:      Time of Closest Approach
          - Maximum elevation
          - Best link quality

LOS-2min: Final commands
          - Send any remaining commands
          - Verify responses

LOS:      Loss of Signal
          - End recording
          - Process data
```

### 7.2 Pass Types

| Type | Duration | Activities |
|------|----------|------------|
| Listen-only | Any | Beacon reception, TLM collection |
| Command | >5 min | Commands + TLM |
| Extended | >10 min | Full operations |

### 7.3 Beacon Schedule

| Mode | Period | Content |
|------|--------|---------|
| SAFE | 120 s | Template only |
| IDLE | 90 s | Template only |
| ACTIVE (sunlit) | 60 s | AI content |
| ACTIVE (eclipse) | 120 s | Template only |

## 8. Ground Station Requirements

### 8.1 Minimum Requirements

| Parameter | Requirement |
|-----------|-------------|
| Antenna gain | ≥10 dBi (VHF), ≥12 dBi (UHF) |
| Receiver sensitivity | <-120 dBm |
| TX EIRP | ≥10 dBW |
| Tracking | ±2° accuracy |
| TNC | AX.25, KISS mode |
| Software | Orbit prediction, TLM decode |

### 8.2 Ground Station Network

| Station | Location | Capability |
|---------|----------|------------|
| Primary | TBD | Full duplex |
| Partner 1 | TBD | RX only |
| Partner 2 | TBD | RX only |
| SatNOGS | Global | RX crowdsource |

## 9. Security Interface

### 9.1 Amateur Radio Compliance

| Requirement | Implementation |
|-------------|----------------|
| No encryption | All data in clear |
| Station ID | Callsign in every frame |
| Open access | Any licensed amateur can receive |
| Power limits | Within Part 97 limits |

### 9.2 Command Authentication

| Measure | Implementation |
|---------|----------------|
| Authentication | Every command carries a 128-bit HMAC-SHA-256 tag (SQSO-ICD-013 Section 4) |
| Replay protection | Strictly increasing 32-bit counter, persisted before execution (SQSO-ICD-013 Section 5) |
| Integrity of framing | 16-bit additive checksum (error detection only) |
| Key | 32 bytes per spacecraft, provisioned at integration, never in the repository |
| Rate limiting | Not implemented |

### 9.3 Operational Security

| Measure | Implementation |
|---------|----------------|
| Pass prediction | Not published in advance |
| Command scripts | Pre-reviewed |
| Access control | Licensed operators only |

## 10. Timing Interface

### 10.1 Time Reference

| Parameter | Value |
|-----------|-------|
| Spacecraft time | UTC (GPS synced when available) |
| Timestamp format | Unix epoch (32-bit) |
| Time accuracy | ±1 second |
| Time source | RTC, synced at ground contact |

### 10.2 Latency

| Parameter | Value |
|-----------|-------|
| Propagation delay | 2-8 ms |
| Processing delay | <100 ms |
| Command execution | <1 s |
| Response latency | <2 s |

## 11. Error Handling

### 11.1 Link Errors

| Error | Detection | Recovery |
|-------|-----------|----------|
| Lost packet | Sequence gap | Request retransmit |
| CRC error | FCS check | Discard, retry |
| Timeout | No response | Retry 3x |
| Link loss | No beacon | Wait for next pass |

### 11.2 Command Errors

Result codes are `CmdResult_t` in `software/flight/include/cmd_handler.h` (SQSO-ICD-013 Section 8):

| Error | Code | Action |
|-------|------|--------|
| Invalid command (framing, unknown ID) | 0x01 | Reject; counted |
| Invalid parameter (argument length) | 0x02 | Reject; counted |
| Authentication failure (tag, replay, no key) | 0x03 | Reject; counted |
| Not allowed in current state | 0x04 | Reject; counted |
| Execution failure | 0x05 | Counted |
| Busy | 0x06 | Retry |

Only the first rejection after an accepted command is written to the fault log.

## 12. Verification Requirements

### 12.1 RF Verification

| Test | Criteria |
|------|----------|
| Frequency accuracy | ±1 kHz |
| Power output | 0.5-2.0W ±0.5 dB |
| Modulation quality | Eye diagram |
| Spurious emissions | <-43 dBc |

### 12.2 Protocol Verification

| Test | Criteria |
|------|----------|
| AX.25 decode | 100% success |
| Command execution | All commands work |
| Error handling | Correct responses |
| End-to-end | Complete pass simulation |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-15 | Funding audit | §6.3, §6.4, §9.2 and §11.2 aligned with the flight software; command framing moved to SQSO-ICD-013 |
| 1.2 | 2026-10-16 | SMART-QSO Team | Beacon Format ICD cited by its own number, SMART-QSO-ICD-006 (finding PI-17) |
