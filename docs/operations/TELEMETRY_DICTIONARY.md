# SMART-QSO Telemetry Dictionary

**Document ID**: SQSO-TLM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines all telemetry parameters for the SMART-QSO mission. It serves as the authoritative reference for telemetry interpretation.

## 2. Telemetry Frame Structure

### 2.1 Frame Format

```
┌──────────────────────────────────────────────────────────────┐
│                    TELEMETRY FRAME (64 bytes)                │
├──────┬──────┬──────┬───────────────────────────┬─────┬──────┤
│SYNC  │ SEQ  │ TIME │         PAYLOAD           │FLAGS│ CRC  │
│2 byte│2 byte│4 byte│        52 bytes           │2byte│2 byte│
└──────┴──────┴──────┴───────────────────────────┴─────┴──────┘
```

### 2.2 Header Fields

| Field | Offset | Size | Format | Description |
|-------|--------|------|--------|-------------|
| SYNC | 0 | 2 | 0xAA55 | Frame sync word |
| SEQ | 2 | 2 | uint16 | Sequence counter (0-65535) |
| TIME | 4 | 4 | uint32 | Seconds since boot |

### 2.3 Trailer Fields

| Field | Offset | Size | Format | Description |
|-------|--------|------|--------|-------------|
| FLAGS | 58 | 2 | uint16 | Status flags |
| CRC | 60 | 2 | uint16 | CRC-16-CCITT |

## 3. Telemetry Parameters

### 3.1 Power System (EPS)

| ID | Name | Offset | Size | Type | Units | Range | Resolution |
|----|------|--------|------|------|-------|-------|------------|
| TLM-001 | BATT_V | 8 | 2 | uint16 | mV | 2500-4200 | 1 mV |
| TLM-002 | BATT_I | 10 | 2 | int16 | mA | -2000 to +2000 | 1 mA |
| TLM-003 | BATT_SOC | 12 | 1 | uint8 | % | 0-100 | 1% |
| TLM-004 | SOLAR_V | 13 | 2 | uint16 | mV | 0-6000 | 1 mV |
| TLM-005 | SOLAR_I | 15 | 2 | uint16 | mA | 0-1000 | 1 mA |
| TLM-006 | BUS_V | 17 | 2 | uint16 | mV | 3000-3600 | 1 mV |
| TLM-007 | LOAD_I | 19 | 2 | uint16 | mA | 0-2000 | 1 mA |

**Conversion Formulas**:
```
BATT_V (V) = raw_value / 1000.0
BATT_I (A) = raw_value / 1000.0  (signed)
BATT_SOC (%) = raw_value
```

### 3.2 Thermal System

| ID | Name | Offset | Size | Type | Units | Range | Resolution |
|----|------|--------|------|------|-------|-------|------------|
| TLM-010 | OBC_TEMP | 21 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-011 | EPS_TEMP | 23 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-012 | BATT_TEMP | 25 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-013 | RF_TEMP | 27 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-014 | JETSON_TEMP | 29 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |
| TLM-015 | STRUCT_TEMP | 31 | 2 | int16 | 0.1°C | -400 to +850 | 0.1°C |

**Conversion Formula**:
```
TEMP (°C) = raw_value / 10.0
```

### 3.3 State and Mode

| ID | Name | Offset | Size | Type | Units | Values |
|----|------|--------|------|------|-------|--------|
| TLM-020 | STATE | 33 | 1 | enum | - | See 3.3.1 |
| TLM-021 | MODE_TIME | 34 | 4 | uint32 | sec | Time in current mode |
| TLM-022 | BOOT_COUNT | 38 | 2 | uint16 | - | Total boot count |
| TLM-023 | BOOT_REASON | 40 | 1 | enum | - | See 3.3.2 |
| TLM-024 | UPTIME | 41 | 4 | uint32 | sec | Seconds since boot |

#### 3.3.1 STATE Values

| Value | Name | Description |
|-------|------|-------------|
| 0 | INIT | Initializing |
| 1 | SAFE | Safe mode |
| 2 | IDLE | Idle mode |
| 3 | ACTIVE | Active mode |
| 255 | FAULT | Fault state |

#### 3.3.2 BOOT_REASON Values

| Value | Name | Description |
|-------|------|-------------|
| 0 | POWER_ON | Normal power-on |
| 1 | WDT_RESET | Watchdog reset |
| 2 | SW_RESET | Software reset |
| 3 | CMD_RESET | Command reset |
| 4 | FAULT_RESET | Fault recovery |
| 5 | BANK_SWITCH | Software bank switch |

### 3.4 RF System

| ID | Name | Offset | Size | Type | Units | Range |
|----|------|--------|------|------|-------|-------|
| TLM-030 | TX_ENABLED | 45 | 1 | bool | - | 0/1 |
| TLM-031 | TX_POWER | 46 | 1 | enum | - | 0=0.5W, 1=1W, 2=2W |
| TLM-032 | TX_COUNT | 47 | 4 | uint32 | - | Beacons transmitted |
| TLM-033 | RX_COUNT | 51 | 4 | uint32 | - | Commands received |
| TLM-034 | LAST_RSSI | 55 | 1 | int8 | dBm | -128 to 0 |

### 3.5 ADCS System

| ID | Name | Offset | Size | Type | Units | Range |
|----|------|--------|------|------|-------|-------|
| TLM-040 | SUN_DETECTED | 56 | 1 | bool | - | 0/1 |
| TLM-041 | TUMBLE | 57 | 1 | bool | - | 0=stable, 1=tumbling |

### 3.6 Payload (Jetson)

| ID | Name | Bits | Type | Description |
|----|------|------|------|-------------|
| TLM-050 | JETSON_STATE | FLAGS[0:1] | enum | 0=OFF, 1=BOOT, 2=READY, 3=BUSY |
| TLM-051 | AI_AVAILABLE | FLAGS[2] | bool | AI model loaded |
| TLM-052 | FALLBACK_MODE | FLAGS[3] | bool | Using fallback beacons |

### 3.7 Fault Counters

| ID | Name | Bits | Type | Description |
|----|------|------|------|-------------|
| TLM-060 | WDT_RESETS | FLAGS[4:7] | uint4 | Watchdog reset count (0-15) |
| TLM-061 | FAULT_FLAGS | FLAGS[8:15] | uint8 | Active fault indicators |

#### 3.7.1 FAULT_FLAGS Bit Definitions

| Bit | Name | Meaning |
|-----|------|---------|
| 0 | BATT_LOW | Battery below 20% |
| 1 | BATT_CRIT | Battery below 15% |
| 2 | TEMP_HIGH | Over-temperature |
| 3 | TEMP_LOW | Under-temperature |
| 4 | COMM_ERR | Communication error |
| 5 | SENSOR_ERR | Sensor error |
| 6 | SW_ERR | Software error |
| 7 | RESERVED | Reserved |

## 4. Beacon Content

### 4.1 AX.25 Beacon Frame

```
┌───────────────────────────────────────────────────────────────┐
│                      AX.25 UI FRAME                           │
├──────┬─────────────┬─────────────┬────┬────┬─────────┬───────┤
│ FLAG │ DESTINATION │   SOURCE    │CTRL│PID │  INFO   │  FCS  │
│ 0x7E │   7 bytes   │   7 bytes   │0x03│0xF0│≤256 byte│2 bytes│
└──────┴─────────────┴─────────────┴────┴────┴─────────┴───────┘
```

### 4.2 Beacon Info Field

The INFO field contains human-readable status and AI-generated content:

```
SQSO-1 DE SMARTQ-1
STATE:ACTIVE SOC:78% V:3.85
SUN:Y RF:ON QSO:42
[AI or template message up to 150 chars]
```

### 4.3 Beacon Timing

| Parameter | Value |
|-----------|-------|
| Interval | 60 seconds |
| Duration | ~2 seconds |
| Frequency | 145.XXX MHz |

## 5. Telemetry Limits

### 5.1 Operational Limits

| Parameter | RED_LOW | YEL_LOW | YEL_HIGH | RED_HIGH |
|-----------|---------|---------|----------|----------|
| BATT_V | 3000 | 3200 | 4100 | 4200 |
| BATT_SOC | 15 | 25 | - | - |
| OBC_TEMP | -350 | -200 | 700 | 800 |
| BATT_TEMP | -50 | 0 | 400 | 450 |
| JETSON_TEMP | -150 | -100 | 600 | 700 |

### 5.2 Limit Actions

| Limit | Color | Action |
|-------|-------|--------|
| RED_LOW | Red | Immediate notification |
| YEL_LOW | Yellow | Monitor closely |
| NOMINAL | Green | Normal operations |
| YEL_HIGH | Yellow | Monitor closely |
| RED_HIGH | Red | Immediate notification |

## 6. Telemetry Processing

### 6.1 Decoder Output Format (JSON)

```json
{
  "timestamp": "2026-01-02T12:34:56Z",
  "sequence": 12345,
  "uptime": 86400,
  "state": "ACTIVE",
  "battery": {
    "voltage": 3.85,
    "current": 0.3,
    "soc": 78
  },
  "thermal": {
    "obc": 25.0,
    "battery": 22.0,
    "jetson": 45.0
  },
  "rf": {
    "tx_enabled": true,
    "tx_power": 1,
    "tx_count": 1440
  },
  "faults": []
}
```

### 6.2 CSV Export Format

```csv
timestamp,seq,state,batt_v,batt_i,batt_soc,obc_temp,batt_temp,jetson_temp
2026-01-02T12:34:56Z,12345,ACTIVE,3.85,0.3,78,25.0,22.0,45.0
```

## 7. Telemetry Archival

### 7.1 Archive Requirements

| Data Type | Retention | Format |
|-----------|-----------|--------|
| Raw frames | Mission + 5 years | Binary |
| Decoded TLM | Mission + 5 years | JSON |
| Summary | Mission + 10 years | CSV |

### 7.2 Archive Naming

```
SQSO_TLM_YYYYMMDD_HHMMSS.json
SQSO_RAW_YYYYMMDD_HHMMSS.bin
SQSO_SUMMARY_YYYYMM.csv
```

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
