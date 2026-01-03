# SMART-QSO Data Budget

**Document ID**: SQSO-DB-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the data budget for the SMART-QSO mission, demonstrating adequate data storage and downlink capacity for all mission data.

## 2. Data Generation

### 2.1 Telemetry Data

| Data Type | Size | Rate | Daily Volume |
|-----------|------|------|--------------|
| Beacon frame | 64 bytes | 1/minute | 92 KB |
| Housekeeping | 128 bytes | 1/10 sec | 1.1 MB |
| Event log | 32 bytes/event | ~100/day | 3.2 KB |
| Fault log | 64 bytes/fault | ~10/day | 0.6 KB |
| **Total** | | | **~1.2 MB/day** |

### 2.2 AI Payload Data

| Data Type | Size | Rate | Daily Volume |
|-----------|------|------|--------------|
| AI beacon text | 200 bytes | ~20/day | 4 KB |
| Inference metrics | 64 bytes | per inference | 1.3 KB |
| Model status | 32 bytes | 1/hour | 0.8 KB |
| **Total** | | | **~6 KB/day** |

### 2.3 Summary by Category

| Category | Daily Volume | Priority |
|----------|--------------|----------|
| Health telemetry | 1.1 MB | High |
| Beacon data | 92 KB | High |
| Event/fault logs | 4 KB | Medium |
| AI data | 6 KB | Low |
| **Total** | **~1.2 MB/day** | |

## 3. Data Storage

### 3.1 On-Board Storage

| Storage | Capacity | Purpose |
|---------|----------|---------|
| OBC RAM buffer | 64 KB | Real-time telemetry |
| OBC Flash | 512 KB | Configuration, logs |
| SD Card (optional) | 4 GB | Extended storage |

### 3.2 Storage Allocation

| Partition | Size | Contents | Retention |
|-----------|------|----------|-----------|
| Boot | 32 KB | Bootloader | Permanent |
| App A | 240 KB | Flight software A | Permanent |
| App B | 240 KB | Flight software B | Permanent |
| Config | 64 KB | Configuration | Permanent |
| Telemetry | 32 KB | Ring buffer | 10 min |
| Event log | 16 KB | Event history | 1000 events |
| Fault log | 8 KB | Fault history | 100 faults |

### 3.3 Storage Duration

```
ON-BOARD DATA RETENTION

Telemetry buffer:    32 KB / 128 bytes × 10 Hz = 25 seconds
Extended (SD):       4 GB / 1.2 MB/day = 3,333 days (~9 years)

Note: Real-time telemetry relies on ground station capture.
      Extended storage optional for science data.
```

## 4. Downlink Capacity

### 4.1 Link Parameters

| Parameter | Value |
|-----------|-------|
| Data rate | 1200 bps |
| Protocol overhead | 30% (AX.25) |
| Effective data rate | 840 bps |
| Contact time per pass | 8 min average |
| Passes per day | 5 average |

### 4.2 Downlink Capacity Calculation

```
DOWNLINK CAPACITY

Effective rate:       840 bps = 105 bytes/sec
Pass duration:        8 minutes = 480 seconds
Data per pass:        480 × 105 = 50.4 KB

Passes per day:       5
Daily capacity:       5 × 50.4 KB = 252 KB/day
```

### 4.3 Data Downlink Budget

| Data Type | Volume/day | Priority | Allocation |
|-----------|------------|----------|------------|
| Beacon (auto) | 92 KB | High | 92 KB (37%) |
| Health summary | 10 KB | High | 10 KB (4%) |
| Event log | 4 KB | Medium | 4 KB (2%) |
| Reserved | - | - | 146 KB (58%) |
| **Total** | **~106 KB** | | **252 KB capacity** |

### 4.4 Margin Analysis

```
DATA DOWNLINK MARGIN

Required:             106 KB/day
Capacity:             252 KB/day
Margin:               146 KB/day (58%)

Status:               ✓ POSITIVE MARGIN
```

## 5. Beacon Data Content

### 5.1 Beacon Frame Structure

| Field | Size | Description |
|-------|------|-------------|
| AX.25 header | 17 bytes | Addresses, control |
| Timestamp | 8 bytes | UTC time |
| State | 1 byte | SAFE/IDLE/ACTIVE |
| Battery | 4 bytes | V, I, SOC |
| Thermal | 6 bytes | 3 temps |
| RF status | 2 bytes | TX count, power |
| AI text | ≤150 bytes | Beacon message |
| Padding | Variable | Fill to frame |
| CRC | 2 bytes | Frame check |
| **Total** | **~200 bytes** | |

### 5.2 Beacon Content Priority

| Priority | Content | Always Present |
|----------|---------|----------------|
| 1 | State, SOC | Yes |
| 2 | Battery voltage | Yes |
| 3 | Temperatures | Yes |
| 4 | RF status | Yes |
| 5 | AI message | If available |

## 6. Command Uplink

### 6.1 Command Capacity

| Parameter | Value |
|-----------|-------|
| Data rate | 1200 bps |
| Command size | 32 bytes |
| Commands per second | 3.75 max |
| Pass duration | 8 minutes |
| **Commands per pass** | **~1,800 max** |

### 6.2 Typical Command Usage

| Operation | Commands | Frequency |
|-----------|----------|-----------|
| Health query | 1 | Each pass |
| Mode change | 1-2 | Weekly |
| Configuration | 5-10 | Monthly |
| Software upload | ~1000 | Rare |

### 6.3 Command Budget

```
COMMAND UPLINK BUDGET

Typical pass:         5-10 commands
Software upload:      ~1000 commands (1 pass)
Capacity:             1800 commands/pass

Status:               ✓ ADEQUATE
```

## 7. Data Flow Diagram

```
DATA FLOW ARCHITECTURE

┌─────────────────────────────────────────────────────────────┐
│                    SPACECRAFT                               │
│                                                              │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐                  │
│  │ Sensors │───▶│   OBC   │───▶│Telemetry│                  │
│  └─────────┘    │         │    │ Buffer  │                  │
│                 │         │    └────┬────┘                  │
│  ┌─────────┐    │         │         │                       │
│  │ Jetson  │───▶│         │         ▼                       │
│  └─────────┘    │         │    ┌─────────┐    ┌─────────┐  │
│                 │         │───▶│ Beacon  │───▶│   RF    │──┼──▶
│  ┌─────────┐    │         │    │Generator│    │   TX    │  │
│  │Commands │◀───│         │    └─────────┘    └─────────┘  │
│  └─────────┘    └─────────┘                                 │
│       ▲                                                      │
└───────┼──────────────────────────────────────────────────────┘
        │
        │ 1200 bps
        ▼
┌───────────────────────────────────────────────────────────────┐
│                    GROUND STATION                             │
│                                                                │
│  ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐   │
│  │   RF    │───▶│Decoder  │───▶│Telemetry│───▶│ Archive │   │
│  │   RX    │    │         │    │ Display │    │         │   │
│  └─────────┘    └─────────┘    └─────────┘    └─────────┘   │
│                                                                │
└───────────────────────────────────────────────────────────────┘
```

## 8. Data Latency

### 8.1 Latency Requirements

| Data Type | Max Latency | Actual |
|-----------|-------------|--------|
| Health critical | 1 orbit | <1 min |
| Telemetry | 2 orbits | <60 min |
| Event data | 1 day | <12 hours |

### 8.2 Latency Analysis

| Path | Latency |
|------|---------|
| Sensor → OBC | <10 ms |
| OBC → Beacon | <100 ms |
| Beacon → Ground | <5 ms (RF) |
| Ground → Archive | <1 min |
| **End-to-end** | **<2 min** |

## 9. Data Budget Summary

| Parameter | Requirement | Capability | Margin |
|-----------|-------------|------------|--------|
| Daily data generated | 1.2 MB | - | - |
| Downlink capacity | - | 252 KB | - |
| Data downloaded/day | 106 KB | 252 KB | 58% |
| Command capacity/pass | 10 | 1800 | 99% |
| Storage duration | 1 day | 25 sec* | N/A |

*Real-time data; extended storage optional

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
