# Software Design Description (SDD)

**Document Number**: SMART-QSO-SDD-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

**Governing Standards**:
- NPR 7150.2 Rev B - NASA Software Engineering Requirements
- IEEE 1016-2009 - Software Design Description

---

## 1. Introduction

### 1.1 Purpose

This Software Design Description (SDD) defines the software architecture, detailed design, and data structures for the SMART-QSO flight software.

### 1.2 Scope

This document covers:
- OBC Flight Software architecture and design
- Jetson Payload Software interface design
- Module interfaces and data structures
- Algorithms and state machines

### 1.3 Design Approach

The SMART-QSO flight software follows a modular, layered architecture based on NASA Ames Small Satellite heritage:

1. **Layered Architecture**: Application → Services → HAL → Hardware
2. **Static Memory**: No dynamic allocation after initialization
3. **Defensive Programming**: Assertions, bounds checking, CRC protection
4. **Fail-Safe Design**: Graceful degradation to safe modes

---

## 2. Architectural Design

### 2.1 System Context

```
                          +------------------+
                          |   Ground Station |
                          +--------+---------+
                                   |
                               RF Link
                                   |
+------------------------------------------------------------------+
|                         SMART-QSO CubeSat                        |
|                                                                  |
|  +------------------+                    +------------------+    |
|  |   OBC Software   |      UART          |  Jetson Payload  |    |
|  |                  |<------------------>|    Software      |    |
|  |  +------------+  |                    |  +------------+  |    |
|  |  |Application |  |                    |  | TinyLM     |  |    |
|  |  +------------+  |                    |  | Inference  |  |    |
|  |  |  Services  |  |                    |  +------------+  |    |
|  |  +------------+  |                    +------------------+    |
|  |  |    HAL     |  |                                           |
|  |  +------------+  |                                           |
|  +--------+---------+                                           |
|           |                                                      |
|   +-------+-------+-------+-------+                             |
|   |       |       |       |       |                             |
|   v       v       v       v       v                             |
|  EPS    ADCS     RF    Sensors  Deploy                          |
+------------------------------------------------------------------+
```

### 2.2 Software Architecture

```
+------------------------------------------------------------------+
|                        APPLICATION LAYER                          |
|  +-------------+  +-------------+  +-------------+  +-----------+ |
|  | Main Task   |  | Beacon Task |  | Telemetry   |  | Command   | |
|  | (Scheduler) |  |             |  | Task        |  | Handler   | |
|  +-------------+  +-------------+  +-------------+  +-----------+ |
+------------------------------------------------------------------+
|                         SERVICES LAYER                            |
|  +----------+  +----------+  +----------+  +----------+           |
|  | EPS      |  | Fault    |  | Sensors  |  | Mission  |           |
|  | Control  |  | Mgmt     |  |          |  | Data     |           |
|  +----------+  +----------+  +----------+  +----------+           |
|  +----------+  +----------+  +----------+                         |
|  | UART     |  | CRC32    |  | Time     |                         |
|  | Comm     |  |          |  | Utils    |                         |
|  +----------+  +----------+  +----------+                         |
+------------------------------------------------------------------+
|                     HARDWARE ABSTRACTION LAYER                    |
|  +--------+  +--------+  +--------+  +--------+  +--------+       |
|  | GPIO   |  | I2C    |  | SPI    |  | UART   |  | Timer  |       |
|  +--------+  +--------+  +--------+  +--------+  +--------+       |
|  +--------+  +--------+  +--------+                               |
|  | ADC    |  | Flash  |  | WDT    |                               |
|  +--------+  +--------+  +--------+                               |
+------------------------------------------------------------------+
|                      HARDWARE (MCU + Peripherals)                 |
+------------------------------------------------------------------+
```

### 2.3 Module Hierarchy

```
smart_qso
├── main.c                    # Application entry, scheduler
├── services/
│   ├── eps_control.c         # Power management service
│   ├── fault_mgmt.c          # FDIR service
│   ├── sensors.c             # Sensor polling service
│   ├── mission_data.c        # Data persistence service
│   └── uart_comm.c           # UART communication service
├── utilities/
│   ├── crc32.c               # CRC32 calculation
│   └── time_utils.c          # Time utilities
└── hal/
    ├── hal_gpio.c            # GPIO abstraction
    ├── hal_i2c.c             # I2C abstraction
    ├── hal_spi.c             # SPI abstraction
    ├── hal_uart.c            # UART abstraction
    ├── hal_timer.c           # Timer abstraction
    ├── hal_adc.c             # ADC abstraction
    ├── hal_flash.c           # Flash abstraction
    └── hal_wdt.c             # Watchdog abstraction
```

---

## 3. Detailed Design

### 3.1 Main Application (main.c)

#### 3.1.1 Purpose
Provides system initialization, main control loop scheduling, and shutdown procedures.

#### 3.1.2 State Machine

```
                    +----------------+
                    |                |
          +---------+     BOOT       +----------+
          |         |                |          |
          |         +-------+--------+          |
          |                 |                   |
          |           init_ok                   |
          |                 |                   |
          v                 v                   v
    +-----------+    +-----------+    +-----------+
    |           |    |           |    |           |
    |   SAFE    |<-->|   IDLE    |<-->|  ACTIVE   |
    |           |    |           |    |           |
    +-----------+    +-----------+    +-----------+
          |                 |                   |
          |                 |                   |
          +--------+--------+-------------------+
                   |
                   v
            +-----------+
            |           |
            |  SHUTDOWN |
            |           |
            +-----------+
```

#### 3.1.3 Functions

| Function | Description | Complexity |
|----------|-------------|------------|
| main() | Entry point, initialization | Low |
| init_system() | System initialization sequence | Medium |
| run_main_loop() | Main scheduler loop | Medium |
| shutdown_system() | Graceful shutdown | Low |
| kick_watchdog() | Watchdog timer refresh | Low |

#### 3.1.4 Initialization Sequence

```
1. Reset cause detection
2. HAL initialization (GPIO, I2C, SPI, UART, Timer)
3. Mission data load/init
4. EPS control initialization
5. Sensor framework initialization
6. Fault management initialization
7. UART communication initialization
8. Watchdog start
9. Self-test execution
10. Enter main loop
```

### 3.2 EPS Control Module (eps_control.c)

#### 3.2.1 Purpose
Manages power modes, load switches, and power budgets based on battery state of charge.

#### 3.2.2 Data Structures

```c
typedef struct {
    PowerMode_t current_mode;
    PowerMode_t previous_mode;
    double last_soc;
    double power_budget_w;
    uint32_t mode_entry_time_ms;
    bool payload_enabled;
    bool sunlit;
    uint32_t crc32;
} EpsControlState_t;

typedef struct {
    double safe_threshold;      // Default: 0.25
    double idle_threshold;      // Default: 0.40
    double active_threshold;    // Default: 0.55
    double max_payload_power;   // Default: 7.5W
    uint32_t crc32;
} EpsConfig_t;
```

#### 3.2.3 Power Mode State Machine

```
                         SOC < 25%
    +---------------+<-----------------+---------------+
    |               |                  |               |
    |     SAFE      |    SOC >= 40%    |     IDLE      |
    |   (0.5W)      +----------------->|    (1.5W)     |
    |               |                  |               |
    +---------------+                  +-------+-------+
                                              |
                                        SOC >= 55%
                                        && sunlit
                                              |
                                              v
                                       +------+------+
                                       |             |
                                       |   ACTIVE    |
                                       |   (3.0W)    |
                                       |             |
                                       +-------------+
                                              |
                                        SOC < 40%
                                        || !sunlit
                                              |
                                              v
                                       Back to IDLE
```

#### 3.2.4 Functions

| Function | Description | Req Trace |
|----------|-------------|-----------|
| eps_init() | Initialize EPS control | SRS-F011 |
| eps_update_state() | Update from sensors | SRS-F010 |
| eps_set_power_mode() | Set power mode | SRS-F012-F014 |
| eps_control_payload() | Control payload power | SRS-F015 |
| eps_control_load() | Control load switch | SRS-F011 |
| eps_check_budget() | Verify power budget | SRS-F016 |
| eps_save_config() | Save to NVM | SRS-F017 |

### 3.3 Fault Management Module (fault_mgmt.c)

#### 3.3.1 Purpose
Implements Fault Detection, Isolation, and Recovery (FDIR) per NASA heritage.

#### 3.3.2 Data Structures

```c
typedef enum {
    FAULT_NONE           = 0,
    FAULT_UNDER_VOLTAGE  = 1,
    FAULT_OVER_VOLTAGE   = 2,
    FAULT_OVER_TEMP      = 3,
    FAULT_UNDER_TEMP     = 4,
    FAULT_COMM_TIMEOUT   = 5,
    FAULT_SENSOR_FAIL    = 6,
    FAULT_WATCHDOG       = 7,
    FAULT_PAYLOAD        = 8,
    FAULT_CRC_ERROR      = 9
} FaultType_t;

typedef enum {
    SEVERITY_INFO    = 0,
    SEVERITY_WARNING = 1,
    SEVERITY_ERROR   = 2,
    SEVERITY_CRITICAL = 3
} FaultSeverity_t;

typedef struct {
    uint64_t timestamp_ms;
    FaultType_t fault_type;
    FaultSeverity_t severity;
    char description[64];
    uint32_t fault_data;
    bool recovered;
    uint32_t crc32;
} FaultLogEntry_t;
```

#### 3.3.3 Fault Response Matrix

| Fault Type | Severity | Response | Recovery |
|------------|----------|----------|----------|
| UNDER_VOLTAGE | CRITICAL | Enter SAFE mode | Wait for SOC > 40% |
| OVER_VOLTAGE | ERROR | Disable charging | Auto-resolve |
| OVER_TEMP | ERROR | Reduce power, disable payload | Wait for temp < 50°C |
| UNDER_TEMP | WARNING | Enable heaters if available | Auto-resolve |
| COMM_TIMEOUT | WARNING | Use fallback beacon | Retry after 60s |
| SENSOR_FAIL | ERROR | Use last known value | Retry sensor |
| WATCHDOG | CRITICAL | Log and reboot | Auto-recovery |
| PAYLOAD | ERROR | Disable payload | Power cycle after 5 min |
| CRC_ERROR | WARNING | Use defaults | Rewrite data |

#### 3.3.4 Functions

| Function | Description | Req Trace |
|----------|-------------|-----------|
| fault_mgmt_init() | Initialize fault system | SRS-F040 |
| fault_check() | Check for fault conditions | SRS-F040 |
| fault_log_add() | Add fault to log | SRS-F041 |
| fault_handle_critical() | Handle critical faults | SRS-F043 |
| fault_recover_thermal() | Thermal recovery | SRS-F042 |
| fault_recover_power() | Power recovery | SRS-F044 |
| fault_recover_comm() | Communication recovery | SRS-F042 |
| fault_get_stats() | Get fault statistics | SRS-F045 |

### 3.4 Sensor Module (sensors.c)

#### 3.4.1 Purpose
Provides configurable sensor polling framework with YAML-based configuration.

#### 3.4.2 Data Structures

```c
typedef enum {
    SENSOR_TYPE_VOLTAGE,
    SENSOR_TYPE_CURRENT,
    SENSOR_TYPE_TEMPERATURE,
    SENSOR_TYPE_STATUS,
    SENSOR_TYPE_COUNT
} SensorType_t;

typedef struct {
    char name[32];
    char label[16];
    SensorType_t type;
    uint32_t poll_interval_ms;
    double min_value;
    double max_value;
    double current_value;
    uint64_t last_poll_ms;
    bool valid;
} SensorConfig_t;
```

#### 3.4.3 Default Sensor Configuration

| Sensor | Label | Type | Interval | Min | Max |
|--------|-------|------|----------|-----|-----|
| battery_voltage | BV | Voltage | 1000ms | 6.0V | 8.4V |
| battery_current | BI | Current | 1000ms | -1.0A | 2.0A |
| battery_temp | BT | Temperature | 5000ms | -20°C | 60°C |
| solar_voltage | SV | Voltage | 1000ms | 0V | 10V |
| solar_current | SI | Current | 1000ms | 0A | 1.0A |
| bus_voltage | BUSV | Voltage | 1000ms | 4.5V | 5.5V |
| mcu_temp | MCUT | Temperature | 5000ms | -40°C | 85°C |
| jetson_temp | JT | Temperature | 5000ms | 0°C | 80°C |

#### 3.4.4 Functions

| Function | Description | Req Trace |
|----------|-------------|-----------|
| sensors_init() | Initialize sensor framework | SRS-F070 |
| sensors_load_yaml() | Load YAML configuration | SRS-F071 |
| sensors_poll() | Poll all due sensors | SRS-F070 |
| sensors_validate() | Validate sensor readings | SRS-F072 |
| sensors_format_telemetry() | Format for transmission | SRS-F073 |
| sensors_get_value() | Get sensor value | SRS-F070 |

### 3.5 Mission Data Module (mission_data.c)

#### 3.5.1 Purpose
Manages persistent mission data with CRC protection.

#### 3.5.2 Data Structures

```c
typedef struct {
    uint32_t version;
    uint32_t reset_count;
    uint64_t total_uptime_s;
    uint32_t beacon_count;
    uint32_t ai_beacon_count;
    uint32_t fallback_beacon_count;
    double total_energy_wh;
    uint64_t last_contact_time;
    PowerMode_t last_power_mode;
    uint32_t fault_count;
    uint32_t watchdog_reset_count;
    uint32_t crc32;
} MissionData_t;
```

#### 3.5.3 Storage Layout

```
NVM Address Map:
0x0000 - 0x00FF: Mission Data (primary)
0x0100 - 0x01FF: Mission Data (backup)
0x0200 - 0x02FF: EPS Configuration
0x0300 - 0x03FF: Sensor Configuration
0x0400 - 0x07FF: Fault Log (16 entries)
0x0800 - 0x0FFF: Reserved
```

#### 3.5.4 Functions

| Function | Description | Req Trace |
|----------|-------------|-----------|
| mission_data_init() | Initialize/load mission data | SRS-F002 |
| mission_data_save() | Save to NVM | SRS-F060 |
| mission_data_verify() | Verify CRC | SRS-F062 |
| mission_data_update() | Update statistics | SRS-F063 |
| mission_data_get() | Get current data | SRS-F063 |

### 3.6 UART Communication Module (uart_comm.c)

#### 3.6.1 Purpose
Manages UART communication with Jetson payload per ICD-002.

#### 3.6.2 Message Protocol

```
Message Format:
<MSG_TYPE>,<FIELD1>,<FIELD2>,...,<CRC16>\n

Example TELEMETRY:
TELEMETRY,1704153600000,SUNLIT,0.75,2,3.0,BV=8.1V,BT=22.5C,BUSV=5.0V,1A2B\n

Example BEACON_RSP:
BEACON_RSP,42,Hello from SMART-QSO! AI-generated beacon text here.,C3D4\n
```

#### 3.6.3 Communication State Machine

```
          +----------+
          |          |
    +---->|   IDLE   |<----+
    |     |          |     |
    |     +----+-----+     |
    |          |           |
    |     send_msg         |
    |          |           |
    |          v           |
    |   +------+------+    |
    |   |             |    |
    |   | WAIT_ACK   |    |
    |   |             |    |
    |   +------+------+    |
    |          |           |
    |    ack_rcvd    timeout
    |          |           |
    |          v           |
    |   +------+------+    |
    |   |             |    |
    +---+  COMPLETE   +----+
        |             |
        +-------------+
```

#### 3.6.4 Functions

| Function | Description | Req Trace |
|----------|-------------|-----------|
| uart_init() | Initialize UART | SRS-I001 |
| uart_send() | Send message | SRS-F030 |
| uart_receive() | Receive with timeout | SRS-F031 |
| uart_send_telemetry() | Send telemetry packet | SRS-F030 |
| uart_request_beacon() | Request beacon text | SRS-F031 |
| uart_check_health() | Check communication health | SRS-F032 |
| uart_calculate_crc16() | Calculate CRC-16 CCITT | SRS-I005 |

### 3.7 CRC32 Module (crc32.c)

#### 3.7.1 Purpose
Provides IEEE 802.3 CRC32 calculation for data integrity.

#### 3.7.2 Algorithm

- Polynomial: 0xEDB88320 (reflected)
- Initial value: 0xFFFFFFFF
- Final XOR: 0xFFFFFFFF
- Lookup table for performance

#### 3.7.3 Functions

| Function | Description | Req Trace |
|----------|-------------|-----------|
| crc32() | Calculate CRC32 | SRS-F061 |
| crc32_init_table() | Initialize lookup table | - |

---

## 4. Interface Design

### 4.1 Module Interface Summary

| Module | Provides | Requires |
|--------|----------|----------|
| main | System scheduling | All services |
| eps_control | Power management | HAL GPIO, mission_data, crc32 |
| fault_mgmt | FDIR | mission_data, crc32, time_utils |
| sensors | Sensor data | HAL I2C, HAL ADC |
| mission_data | Persistence | HAL Flash, crc32 |
| uart_comm | Communication | HAL UART |

### 4.2 HAL Interface

```c
// GPIO HAL
SmartQsoResult_t hal_gpio_init(void);
SmartQsoResult_t hal_gpio_set(uint8_t pin, bool state);
bool hal_gpio_get(uint8_t pin);

// I2C HAL
SmartQsoResult_t hal_i2c_init(uint8_t bus);
SmartQsoResult_t hal_i2c_write(uint8_t bus, uint8_t addr,
                               const uint8_t *data, size_t len);
SmartQsoResult_t hal_i2c_read(uint8_t bus, uint8_t addr,
                              uint8_t *data, size_t len);

// UART HAL
SmartQsoResult_t hal_uart_init(uint32_t baud);
SmartQsoResult_t hal_uart_write(const uint8_t *data, size_t len);
SmartQsoResult_t hal_uart_read(uint8_t *data, size_t len,
                               uint32_t timeout_ms);

// Flash HAL
SmartQsoResult_t hal_flash_read(uint32_t addr, uint8_t *data, size_t len);
SmartQsoResult_t hal_flash_write(uint32_t addr, const uint8_t *data,
                                 size_t len);

// Timer HAL
uint64_t hal_timer_get_ms(void);
void hal_timer_delay_ms(uint32_t ms);

// Watchdog HAL
SmartQsoResult_t hal_wdt_init(uint32_t timeout_ms);
void hal_wdt_kick(void);
```

---

## 5. Data Flow

### 5.1 Telemetry Data Flow

```
Sensors → sensors_poll() → Sensor Values
                              ↓
                    sensors_format_telemetry()
                              ↓
                         Telemetry String
                              ↓
              +---------------+---------------+
              ↓                               ↓
       uart_send_telemetry()          build_beacon_info()
              ↓                               ↓
         To Jetson                       AX.25 Frame
                                              ↓
                                         RF Transmit
```

### 5.2 Beacon Generation Data Flow

```
+-------------+     TELEMETRY      +-------------+
|     OBC     |------------------>|   Jetson    |
|             |                    |             |
|             |     BEACON_REQ     |             |
|             |------------------>|             |
|             |                    |  TinyLM    |
|             |                    |  Inference  |
|             |     BEACON_RSP     |             |
|             |<------------------|             |
+-------------+                    +-------------+
      |
      v
validate_beacon_text()
      |
      +--> valid: Use AI beacon
      |
      +--> invalid/timeout: Use fallback beacon
      |
      v
format_ax25_frame()
      |
      v
RF Transmit
```

### 5.3 Fault Handling Data Flow

```
Sensor/Event → fault_check() → Fault Detected?
                                    |
                    +---------------+---------------+
                    | No                            | Yes
                    v                               v
               Continue                     fault_log_add()
                                                   |
                                           Severity Check
                                                   |
                    +---------------+---------------+
                    | CRITICAL                      | Other
                    v                               v
           fault_handle_critical()          Attempt Recovery
                    |                               |
                    v                               v
              Enter SAFE Mode              Continue Monitoring
```

---

## 6. Timing Design

### 6.1 Task Timing

| Task | Period | Max Duration | Priority |
|------|--------|--------------|----------|
| Watchdog kick | 10s | 1ms | Highest |
| Sensor polling | 1s | 50ms | High |
| Fault check | 100ms | 10ms | High |
| Beacon generation | 30s | 5s | Medium |
| Data persistence | 60s | 500ms | Low |
| Telemetry to Jetson | 60s | 100ms | Medium |

### 6.2 Main Loop Timing

```
T=0ms    : Kick watchdog
T=1ms    : Check fault conditions
T=10ms   : Poll sensors (if due)
T=15ms   : Update EPS state
T=18ms   : Check beacon timing
         : (if due) Request/generate beacon
T=20ms   : Loop complete (< 20ms requirement)
```

---

## 7. Memory Design

### 7.1 Memory Map (STM32L4 256KB)

| Region | Start | End | Size | Content |
|--------|-------|-----|------|---------|
| Flash | 0x08000000 | 0x0803FFFF | 256KB | Code + Constants |
| SRAM1 | 0x20000000 | 0x20017FFF | 96KB | Stack + Heap + Data |
| SRAM2 | 0x10000000 | 0x10007FFF | 32KB | DMA buffers |

### 7.2 Stack Allocation

| Task | Stack Size | Notes |
|------|------------|-------|
| Main | 4KB | Largest task |
| Idle | 512B | Minimal |
| Timer | 1KB | Callbacks |
| UART | 1KB | Receive buffer |
| Total | < 8KB | Per SRS-C003 |

### 7.3 Static Data Allocation

| Data | Size | Location |
|------|------|----------|
| Sensor configs | 512B | .data |
| Fault log | 2KB | .bss |
| TX buffers | 1KB | SRAM2 |
| RX buffers | 1KB | SRAM2 |
| Mission data | 256B | .data |

---

## 8. Security Design

### 8.1 Input Validation

All external inputs are validated:
- UART data: Length limits, ASCII check, CRC verification
- Sensor data: Range checking, sanity validation
- NVM data: CRC verification, version check

### 8.2 Buffer Protection

- Fixed-size buffers with length parameters
- Bounds checking on all array access
- No unbounded string operations

### 8.3 State Protection

- CRC32 on all persistent data
- Dual-copy storage for critical data
- Watchdog for software hangs

---

## 9. Test Design

### 9.1 Unit Test Strategy

Each module has corresponding unit tests:
- test_eps_control.c
- test_fault_mgmt.c
- test_sensors.c
- test_mission_data.c
- test_uart_comm.c
- test_crc32.c

### 9.2 Mock Architecture

```
+-------------+     +-------------+
| Unit Test   |<--->| Mock HAL    |
+-------------+     +-------------+
      |                   |
      v                   v
+-------------+     +-------------+
| Module      |<--->| Real HAL    |
| Under Test  |     | (flight)    |
+-------------+     +-------------+
```

---

## Appendix A: Design Patterns

### A.1 Static Initialization Pattern

```c
static bool s_initialized = false;

SmartQsoResult_t module_init(void) {
    if (s_initialized) {
        return SMART_QSO_OK;
    }

    // Initialization code...

    s_initialized = true;
    return SMART_QSO_OK;
}
```

### A.2 CRC-Protected Structure Pattern

```c
typedef struct {
    // Data fields...
    uint32_t crc32;  // Always last field
} ProtectedData_t;

SmartQsoResult_t save_data(const ProtectedData_t *data) {
    ProtectedData_t copy = *data;
    copy.crc32 = crc32(&copy, sizeof(copy) - sizeof(uint32_t));
    return hal_flash_write(ADDR, &copy, sizeof(copy));
}
```

### A.3 State Machine Pattern

```c
typedef enum { STATE_A, STATE_B, STATE_C } State_t;

static State_t s_state = STATE_A;

void process_event(Event_t event) {
    switch (s_state) {
        case STATE_A:
            if (event == EVENT_X) {
                s_state = STATE_B;
                enter_state_b();
            }
            break;
        // ...
    }
}
```

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
