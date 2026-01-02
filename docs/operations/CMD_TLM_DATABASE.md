# Command and Telemetry Database
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-OPS-003
**Revision:** A
**Date:** 2026-01-02

---

## 1. Purpose

This document defines the complete command and telemetry database for the SMART-QSO CubeSat, including all command definitions, telemetry points, and data formats.

## 2. Command Summary

### 2.1 Command Categories

| Category | ID Range | Count | Description |
|----------|----------|-------|-------------|
| System | 0x00-0x0F | 16 | System-level commands |
| EPS | 0x10-0x1F | 16 | Power system commands |
| ADCS | 0x20-0x2F | 16 | Attitude control commands |
| RF | 0x30-0x3F | 16 | Communication commands |
| Payload | 0x40-0x4F | 16 | Jetson payload commands |
| Data | 0x50-0x5F | 16 | Data management commands |
| Debug | 0xF0-0xFF | 16 | Debug/test commands |

### 2.2 Command Structure

```
┌────────────────────────────────────────────────────┐
│                Command Packet Format                │
├──────┬──────┬──────┬──────────────┬───────┬───────┤
│ SYNC │ LEN  │ CMD  │    DATA      │ SEQ   │  CRC  │
│ 2B   │ 1B   │ 1B   │   0-250B     │  2B   │  2B   │
└──────┴──────┴──────┴──────────────┴───────┴───────┘

SYNC:  0xEB90 (magic bytes)
LEN:   Total packet length
CMD:   Command ID
DATA:  Command-specific parameters
SEQ:   Sequence number (for acknowledgment)
CRC:   CRC-16-CCITT
```

## 3. System Commands (0x00-0x0F)

| ID | Mnemonic | Parameters | Description |
|----|----------|------------|-------------|
| 0x00 | SYS_NOP | None | No operation (connection test) |
| 0x01 | SYS_RESET | type: u8 | System reset (0=soft, 1=hard) |
| 0x02 | SYS_SET_TIME | epoch: u32 | Set system time (Unix epoch) |
| 0x03 | SYS_GET_STATUS | None | Request system status |
| 0x04 | SYS_SET_MODE | mode: u8 | Set mission mode |
| 0x05 | SYS_SAFE_MODE | None | Enter safe mode |
| 0x06 | SYS_NOMINAL | None | Exit safe mode |
| 0x07 | SYS_GET_CONFIG | None | Request configuration |
| 0x08 | SYS_SET_CONFIG | config: var | Set configuration |
| 0x09 | SYS_CLEAR_FAULTS | None | Clear fault counters |
| 0x0A | SYS_GET_FAULTS | None | Request fault log |
| 0x0B | SYS_DISABLE_AUTO | mask: u8 | Disable autonomy |
| 0x0C | SYS_ENABLE_AUTO | mask: u8 | Enable autonomy |
| 0x0D | SYS_GET_VERSION | None | Request software version |
| 0x0E | SYS_ECHO | data: var | Echo test |
| 0x0F | SYS_RESERVED | - | Reserved |

## 4. EPS Commands (0x10-0x1F)

| ID | Mnemonic | Parameters | Description |
|----|----------|------------|-------------|
| 0x10 | EPS_GET_STATUS | None | Request EPS telemetry |
| 0x11 | EPS_SET_MODE | mode: u8 | Set power mode (SAFE/IDLE/ACTIVE) |
| 0x12 | EPS_ENABLE_RAIL | rail: u8 | Enable power rail |
| 0x13 | EPS_DISABLE_RAIL | rail: u8 | Disable power rail |
| 0x14 | EPS_SET_LIMIT | rail: u8, limit: u16 | Set current limit |
| 0x15 | EPS_GET_HISTORY | None | Request power history |
| 0x16 | EPS_HEATER_ON | heater: u8 | Enable heater |
| 0x17 | EPS_HEATER_OFF | heater: u8 | Disable heater |
| 0x18 | EPS_SET_SOC_THRESH | low: u8, crit: u8 | Set SOC thresholds |
| 0x19 | EPS_BATTERY_TEST | None | Initiate battery test |
| 0x1A | EPS_MPPT_MODE | mode: u8 | Set MPPT mode |
| 0x1B-0x1F | - | - | Reserved |

## 5. ADCS Commands (0x20-0x2F)

| ID | Mnemonic | Parameters | Description |
|----|----------|------------|-------------|
| 0x20 | ADCS_GET_STATUS | None | Request ADCS telemetry |
| 0x21 | ADCS_SET_MODE | mode: u8 | Set ADCS mode |
| 0x22 | ADCS_DETUMBLE | None | Start detumble |
| 0x23 | ADCS_SUN_POINT | None | Start sun-pointing |
| 0x24 | ADCS_IDLE | None | ADCS idle (no control) |
| 0x25 | ADCS_SET_GAIN | axis: u8, gain: f32 | Set control gain |
| 0x26 | ADCS_CALIBRATE | sensor: u8 | Calibrate sensor |
| 0x27 | ADCS_GET_ATTITUDE | None | Request attitude data |
| 0x28 | ADCS_MTQ_TEST | axis: u8, level: i8 | Magnetorquer test |
| 0x29 | ADCS_SET_BDOT | kp: f32 | Set B-dot gain |
| 0x2A-0x2F | - | - | Reserved |

## 6. RF Commands (0x30-0x3F)

| ID | Mnemonic | Parameters | Description |
|----|----------|------------|-------------|
| 0x30 | RF_GET_STATUS | None | Request RF telemetry |
| 0x31 | RF_TX_ENABLE | None | Enable transmitter |
| 0x32 | RF_TX_DISABLE | None | Disable transmitter |
| 0x33 | RF_SET_POWER | power: u8 | Set TX power (0-3) |
| 0x34 | RF_SET_FREQ | freq: u32 | Set frequency (Hz) |
| 0x35 | RF_SET_BEACON_INT | interval: u16 | Set beacon interval (s) |
| 0x36 | RF_SEND_BEACON | None | Transmit immediate beacon |
| 0x37 | RF_SET_CALLSIGN | call: char[8] | Set callsign |
| 0x38 | RF_ANTENNA_DEPLOY | None | Deploy antenna |
| 0x39 | RF_GET_RSSI | None | Request RSSI |
| 0x3A-0x3F | - | - | Reserved |

## 7. Payload Commands (0x40-0x4F)

| ID | Mnemonic | Parameters | Description |
|----|----------|------------|-------------|
| 0x40 | PAY_GET_STATUS | None | Request payload status |
| 0x41 | PAY_ENABLE | None | Power on Jetson |
| 0x42 | PAY_DISABLE | None | Power off Jetson |
| 0x43 | PAY_GENERATE | None | Request AI generation |
| 0x44 | PAY_SET_MODE | mode: u8 | Set AI mode |
| 0x45 | PAY_GET_HEALTH | None | Request Jetson health |
| 0x46 | PAY_SET_DUTY | duty: u8 | Set duty cycle (%) |
| 0x47 | PAY_CLEAR_QUEUE | None | Clear generation queue |
| 0x48 | PAY_SET_TEMPLATE | idx: u8 | Set fallback template |
| 0x49 | PAY_FORCE_TEMPLATE | None | Force template mode |
| 0x4A-0x4F | - | - | Reserved |

## 8. Data Management Commands (0x50-0x5F)

| ID | Mnemonic | Parameters | Description |
|----|----------|------------|-------------|
| 0x50 | DATA_GET_CATALOG | None | Request data catalog |
| 0x51 | DATA_DOWNLOAD | id: u16, offset: u32 | Download data block |
| 0x52 | DATA_DELETE | id: u16 | Delete data item |
| 0x53 | DATA_CLEAR_ALL | confirm: u32 | Clear all stored data |
| 0x54 | DATA_GET_STATS | None | Request storage stats |
| 0x55 | DATA_SET_PRIORITY | type: u8, pri: u8 | Set logging priority |
| 0x56 | DATA_DUMP_TLM | start: u32, end: u32 | Dump telemetry range |
| 0x57-0x5F | - | - | Reserved |

## 9. Telemetry Summary

### 9.1 Telemetry Categories

| Category | Packet ID | Update Rate | Size |
|----------|-----------|-------------|------|
| Housekeeping | 0x01 | 1 Hz | 64 bytes |
| EPS | 0x02 | 1 Hz | 32 bytes |
| ADCS | 0x03 | 10 Hz | 48 bytes |
| RF | 0x04 | 0.1 Hz | 16 bytes |
| Payload | 0x05 | 0.1 Hz | 24 bytes |
| Event | 0x06 | On event | Variable |

### 9.2 Telemetry Packet Structure

```
┌────────────────────────────────────────────────────┐
│              Telemetry Packet Format               │
├──────┬──────┬──────┬───────┬──────────────┬───────┤
│ SYNC │ LEN  │ PKT  │ TIME  │    DATA      │  CRC  │
│ 2B   │ 1B   │ 1B   │  4B   │   Variable   │  2B   │
└──────┴──────┴──────┴───────┴──────────────┴───────┘

SYNC:  0x1ACF (magic bytes)
LEN:   Total packet length
PKT:   Packet type ID
TIME:  Timestamp (seconds since boot)
DATA:  Packet-specific telemetry
CRC:   CRC-16-CCITT
```

## 10. Housekeeping Telemetry (0x01)

| Offset | Name | Type | Units | Description |
|--------|------|------|-------|-------------|
| 0 | boot_count | u16 | - | Total boot count |
| 2 | uptime | u32 | s | Seconds since boot |
| 6 | mission_mode | u8 | enum | Current mission mode |
| 7 | power_mode | u8 | enum | Current power mode |
| 8 | fault_flags | u16 | bitmap | Active fault flags |
| 10 | reset_cause | u8 | enum | Last reset cause |
| 11 | wdt_resets | u8 | - | Watchdog reset count |
| 12 | cpu_temp | i8 | °C | CPU temperature |
| 13 | cpu_load | u8 | % | CPU utilization |
| 14 | mem_free | u16 | KB | Free memory |
| 16 | flash_writes | u32 | - | Flash write count |
| 20 | cmd_count | u32 | - | Commands received |
| 24 | cmd_errors | u16 | - | Command errors |
| 26 | last_contact | u32 | s | Seconds since ground |

## 11. EPS Telemetry (0x02)

| Offset | Name | Type | Units | Description |
|--------|------|------|-------|-------------|
| 0 | battery_v | u16 | mV | Battery voltage |
| 2 | battery_i | i16 | mA | Battery current |
| 4 | battery_soc | u8 | % | State of charge |
| 5 | battery_temp | i8 | °C | Battery temperature |
| 6 | solar_v | u16 | mV | Solar array voltage |
| 8 | solar_i | u16 | mA | Solar array current |
| 10 | solar_power | u16 | mW | Solar power |
| 12 | bus_3v3_i | u16 | mA | 3.3V bus current |
| 14 | bus_5v0_i | u16 | mA | 5.0V bus current |
| 16 | power_mode | u8 | enum | Power mode |
| 17 | rail_status | u8 | bitmap | Rail enable status |
| 18 | heater_status | u8 | bitmap | Heater status |
| 19 | eps_temp | i8 | °C | EPS board temp |
| 20 | charge_cycles | u16 | - | Battery charge cycles |

## 12. ADCS Telemetry (0x03)

| Offset | Name | Type | Units | Description |
|--------|------|------|-------|-------------|
| 0 | adcs_mode | u8 | enum | ADCS mode |
| 1 | adcs_state | u8 | enum | ADCS state |
| 2 | mag_x | i16 | nT | Magnetometer X |
| 4 | mag_y | i16 | nT | Magnetometer Y |
| 6 | mag_z | i16 | nT | Magnetometer Z |
| 8 | gyro_x | i16 | mdeg/s | Gyroscope X |
| 10 | gyro_y | i16 | mdeg/s | Gyroscope Y |
| 12 | gyro_z | i16 | mdeg/s | Gyroscope Z |
| 14 | sun_x | u16 | - | Sun sensor +X |
| 16 | sun_y | u16 | - | Sun sensor +Y |
| 18 | sun_z | u16 | - | Sun sensor +Z |
| 20 | sun_valid | u8 | bitmap | Sun sensor validity |
| 21 | eclipse | u8 | bool | In eclipse |
| 22 | rate_mag | u16 | mdeg/s | Angular rate magnitude |
| 24 | point_error | u16 | cdeg | Pointing error |
| 26 | mtq_x | i8 | % | Magnetorquer X cmd |
| 27 | mtq_y | i8 | % | Magnetorquer Y cmd |
| 28 | mtq_z | i8 | % | Magnetorquer Z cmd |

## 13. RF Telemetry (0x04)

| Offset | Name | Type | Units | Description |
|--------|------|------|-------|-------------|
| 0 | rf_state | u8 | enum | RF state |
| 1 | tx_power | u8 | enum | TX power level |
| 2 | tx_count | u16 | - | Packets transmitted |
| 4 | rx_count | u16 | - | Packets received |
| 6 | beacon_count | u16 | - | Beacons sent |
| 8 | last_rssi | i8 | dBm | Last received RSSI |
| 9 | pa_temp | i8 | °C | PA temperature |
| 10 | ant_deployed | u8 | bool | Antenna deployed |
| 11 | beacon_interval | u8 | s | Current beacon interval |

## 14. Payload Telemetry (0x05)

| Offset | Name | Type | Units | Description |
|--------|------|------|-------|-------------|
| 0 | jetson_state | u8 | enum | Jetson state |
| 1 | jetson_temp | i8 | °C | Jetson temperature |
| 2 | jetson_power | u16 | mW | Jetson power |
| 4 | ai_mode | u8 | enum | AI operating mode |
| 5 | gen_count | u16 | - | Generations completed |
| 7 | gen_errors | u8 | - | Generation errors |
| 8 | template_mode | u8 | bool | Using templates |
| 9 | queue_depth | u8 | - | Generation queue depth |
| 10 | last_gen_time | u16 | ms | Last generation time |
| 12 | duty_cycle | u8 | % | Actual duty cycle |

## 15. Event Telemetry (0x06)

| Offset | Name | Type | Description |
|--------|------|------|-------------|
| 0 | event_type | u8 | Event category |
| 1 | event_id | u8 | Specific event |
| 2 | severity | u8 | Event severity |
| 3 | data_len | u8 | Event data length |
| 4 | data | var | Event-specific data |

### 15.1 Event Types

| Type | Name | Description |
|------|------|-------------|
| 0x00 | BOOT | Boot events |
| 0x01 | MODE | Mode transitions |
| 0x02 | FAULT | Fault events |
| 0x03 | COMMAND | Command events |
| 0x04 | AUTONOMY | Autonomous actions |
| 0x05 | COMM | Communication events |

## 16. Beacon Format

### 16.1 AX.25 Beacon Content

```
Field              Bytes   Description
─────────────────────────────────────────
Callsign           8       Source callsign
Timestamp          8       UTC time string
Battery V          4       Voltage (V)
Battery I          4       Current (mA)
SOC                3       State of charge (%)
Mode               8       Mission mode
Temperature        4       Average temp (°C)
ADCS State         12      ADCS mode/status
AI Content         0-100   AI-generated text
CRC                4       CRC32 checksum
```

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
