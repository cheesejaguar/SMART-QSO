# Software Configuration Index
## SMART-QSO Flight Software

**Document Number:** SQSO-SW-006
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document provides an index of all configurable parameters in the SMART-QSO flight software, including default values, valid ranges, and configuration methods.

## 2. Scope

This document covers:
- OBC flight software configuration
- Build-time configuration
- Runtime configuration
- Hardware-dependent configuration

---

## 3. Build-Time Configuration

### 3.1 CMake Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| FLIGHT_BUILD | BOOL | OFF | Enable flight configuration |
| ENABLE_COVERAGE | BOOL | OFF | Enable code coverage |
| ENABLE_SANITIZERS | BOOL | OFF | Enable address sanitizers |
| TARGET_PLATFORM | STRING | LINUX | Target: LINUX, STM32L4, FREERTOS |

### 3.2 Preprocessor Defines

| Define | Default | Description |
|--------|---------|-------------|
| DEBUG_ENABLED | 1 (dev) / 0 (flight) | Enable debug output |
| WATCHDOG_ENABLED | 0 (dev) / 1 (flight) | Enable hardware watchdog |
| SIMULATION_MODE | 1 (dev) / 0 (flight) | Enable HAL simulation |
| LOG_LEVEL | LOG_DEBUG / LOG_INFO | Logging verbosity |

### 3.3 Memory Configuration

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| STACK_SIZE_MAIN | 4096 | 2048-8192 | bytes |
| STACK_SIZE_SENSOR | 1024 | 512-2048 | bytes |
| STACK_SIZE_BEACON | 2048 | 1024-4096 | bytes |
| HEAP_SIZE | 32768 | 16384-65536 | bytes |
| TLM_BUFFER_COUNT | 8 | 4-16 | count |
| FAULT_LOG_SIZE | 64 | 32-128 | entries |

---

## 4. Power Management Configuration

### 4.1 Power Mode Thresholds

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| SAFE_MODE_VOLTAGE | 7.0 | 6.5-7.2 | V |
| IDLE_MODE_VOLTAGE | 7.4 | 7.0-7.6 | V |
| ACTIVE_MODE_VOLTAGE | 7.6 | 7.4-7.8 | V |
| BURST_MODE_VOLTAGE | 7.8 | 7.6-8.0 | V |
| CRITICAL_VOLTAGE | 6.5 | 6.0-6.8 | V |

### 4.2 Power Limits

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| MAX_SYSTEM_POWER | 10.0 | 5.0-15.0 | W |
| JETSON_POWER_LIMIT | 8.0 | 5.0-10.0 | W |
| RF_POWER_HIGH | 2.0 | 0.5-2.0 | W |
| RF_POWER_LOW | 0.5 | 0.1-1.0 | W |

### 4.3 Battery Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| BATTERY_CAPACITY | 5200 | 2600-10400 | mAh |
| MAX_DOD | 30 | 20-50 | % |
| CHARGE_TEMP_MIN | 0 | -10 to +10 | C |
| CHARGE_TEMP_MAX | 45 | 40-50 | C |
| DISCHARGE_TEMP_MIN | -20 | -30 to -10 | C |
| DISCHARGE_TEMP_MAX | 60 | 50-70 | C |

---

## 5. Communication Configuration

### 5.1 Beacon Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| BEACON_INTERVAL_SAFE | 60 | 30-120 | seconds |
| BEACON_INTERVAL_IDLE | 30 | 15-60 | seconds |
| BEACON_INTERVAL_ACTIVE | 20 | 10-30 | seconds |
| BEACON_MAX_LENGTH | 256 | 128-512 | bytes |
| BEACON_CALLSIGN | "NOCALL" | 4-7 chars | string |

### 5.2 RF Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| TX_FREQUENCY | 145825000 | 145800000-145850000 | Hz |
| TX_DEVIATION | 3500 | 3000-4000 | Hz |
| TX_POWER_LEVEL | 1 | 0-3 | enum |
| BAUD_RATE | 1200 | 1200/9600 | bps |
| PREAMBLE_FLAGS | 50 | 20-100 | flags |

### 5.3 UART Configuration

| Interface | Baud | Data | Parity | Stop |
|-----------|------|------|--------|------|
| DEBUG | 115200 | 8 | None | 1 |
| JETSON | 115200 | 8 | None | 1 |
| EPS | 9600 | 8 | None | 1 |

---

## 6. ADCS Configuration

### 6.1 Sensor Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| MAG_SAMPLE_RATE | 10 | 1-50 | Hz |
| MAG_FILTER_ALPHA | 0.3 | 0.1-0.9 | - |
| SUN_SAMPLE_RATE | 5 | 1-20 | Hz |
| SUN_THRESHOLD | 0.1 | 0.05-0.5 | V |

### 6.2 Control Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| BDOT_GAIN | 1.0e6 | 1e5-1e7 | - |
| DETUMBLE_THRESHOLD | 5.0 | 1.0-10.0 | deg/s |
| MTQ_MAX_DIPOLE | 0.2 | 0.1-0.5 | Am^2 |
| CONTROL_PERIOD | 1000 | 100-5000 | ms |

### 6.3 Calibration Data

| Sensor | Offset X | Offset Y | Offset Z | Unit |
|--------|----------|----------|----------|------|
| Magnetometer | 0.0 | 0.0 | 0.0 | uT |
| Sun Sensor +X | 0.0 | - | - | V |
| Sun Sensor -X | 0.0 | - | - | V |
| Sun Sensor +Y | 0.0 | - | - | V |
| Sun Sensor -Y | 0.0 | - | - | V |
| Sun Sensor +Z | 0.0 | - | - | V |
| Sun Sensor -Z | 0.0 | - | - | V |

---

## 7. Timing Configuration

### 7.1 Task Periods

| Task | Period | Priority | Stack |
|------|--------|----------|-------|
| Main loop | 100 ms | Normal | 4 KB |
| Sensor sampling | 100 ms | High | 1 KB |
| Beacon generation | 1000 ms | Normal | 2 KB |
| ADCS control | 1000 ms | High | 2 KB |
| Telemetry | 5000 ms | Low | 1 KB |
| Housekeeping | 10000 ms | Low | 512 B |

### 7.2 Watchdog Timeouts

| Watchdog | Timeout | Action |
|----------|---------|--------|
| Hardware WDT | 30000 ms | Reset |
| Task WDT | 10000 ms | Log + escalate |
| Jetson WDT | 60000 ms | Power cycle |
| Comm WDT | 86400000 ms | Safe mode |

### 7.3 Deployment Timers

| Timer | Value | Valid Range | Unit |
|-------|-------|-------------|------|
| ANTENNA_DEPLOY_DELAY | 1800 | 1800-3600 | seconds |
| ANTENNA_BURN_TIME | 5 | 3-10 | seconds |
| ANTENNA_RETRY_DELAY | 300 | 60-600 | seconds |
| MAX_DEPLOY_ATTEMPTS | 3 | 1-5 | count |

---

## 8. Fault Management Configuration

### 8.1 Fault Thresholds

| Fault | Threshold | Count | Window | Action |
|-------|-----------|-------|--------|--------|
| Undervoltage | <6.5V | 3 | 60s | Safe mode |
| Overvoltage | >8.6V | 1 | - | Disable charge |
| Overtemp | >60C | 3 | 60s | Thermal safe |
| Comm timeout | 24 hr | 1 | - | Safe mode |
| Watchdog | - | 3 | 1 hr | Reset + log |

### 8.2 Recovery Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| AUTO_RECOVERY_DELAY | 3600 | 600-86400 | seconds |
| MAX_RECOVERY_ATTEMPTS | 3 | 1-10 | count |
| SAFE_MODE_DURATION | 86400 | 3600-604800 | seconds |

---

## 9. AI Payload Configuration

### 9.1 Jetson Parameters

| Parameter | Default | Valid Range | Unit |
|-----------|---------|-------------|------|
| JETSON_BOOT_TIMEOUT | 30 | 20-60 | seconds |
| JETSON_INFERENCE_TIMEOUT | 10 | 5-30 | seconds |
| JETSON_HEALTH_INTERVAL | 5 | 1-30 | seconds |
| JETSON_POWER_MODE | 7W | 7W/15W | - |
| MAX_BURST_DURATION | 60 | 30-300 | seconds |

### 9.2 Model Parameters

| Parameter | Default | Description |
|-----------|---------|-------------|
| MODEL_PATH | /models/tinyLM | Model location |
| MAX_TOKENS | 50 | Max output tokens |
| TEMPERATURE | 0.7 | Generation temperature |
| TOP_P | 0.9 | Nucleus sampling |

---

## 10. Configuration Storage

### 10.1 Non-Volatile Storage Map

| Address | Parameter Block | Size | CRC |
|---------|-----------------|------|-----|
| 0x0000 | System config | 256 B | Yes |
| 0x0100 | Power config | 128 B | Yes |
| 0x0180 | Comm config | 128 B | Yes |
| 0x0200 | ADCS config | 256 B | Yes |
| 0x0300 | Timing config | 128 B | Yes |
| 0x0380 | Fault config | 128 B | Yes |
| 0x0400 | Calibration | 256 B | Yes |

### 10.2 Configuration Commands

| Command | Description |
|---------|-------------|
| CFG_LOAD | Load config from NVM |
| CFG_SAVE | Save config to NVM |
| CFG_DEFAULT | Reset to defaults |
| CFG_DUMP | Output all config |

---

## 11. Flight vs. Test Configuration

| Parameter | Flight | Test | Flatsat |
|-----------|--------|------|---------|
| WATCHDOG_ENABLED | 1 | 0 | 0 |
| DEBUG_ENABLED | 0 | 1 | 1 |
| TX_POWER_LEVEL | 1-3 | 0 | 0 |
| SIMULATION_MODE | 0 | 0 | 1 |
| LOG_LEVEL | INFO | DEBUG | DEBUG |
| DEPLOYMENT_ENABLED | 1 | 0 | 0 |

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
