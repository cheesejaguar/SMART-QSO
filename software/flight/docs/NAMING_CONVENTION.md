# Naming Convention
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-009
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, MISRA C:2012, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines naming conventions for SMART-QSO flight software to ensure consistency, readability, and traceability across all source code.

## 2. Scope

This document covers:
- Variable and constant naming
- Function naming
- Type naming
- Macro and preprocessor naming
- File naming

---

## 3. General Principles

### 3.1 Core Rules

| Rule | Rationale |
|------|-----------|
| Descriptive names | Self-documenting code |
| Consistent style | Reduce cognitive load |
| Prefix modules | Avoid namespace collision |
| Avoid abbreviations | Except well-known (e.g., ADC, I2C) |

### 3.2 Approved Abbreviations

| Abbreviation | Meaning |
|--------------|---------|
| adc | Analog-to-digital converter |
| adcs | Attitude determination and control system |
| cfg | Configuration |
| cmd | Command |
| crc | Cyclic redundancy check |
| ctx | Context |
| eps | Electrical power system |
| gpio | General-purpose input/output |
| hal | Hardware abstraction layer |
| i2c | Inter-integrated circuit |
| isr | Interrupt service routine |
| max | Maximum |
| min | Minimum |
| mtq | Magnetorquer |
| obc | On-board computer |
| pdu | Power distribution unit |
| ptr | Pointer |
| rf | Radio frequency |
| rx | Receive |
| spi | Serial peripheral interface |
| tlm | Telemetry |
| tx | Transmit |
| uart | Universal async receiver-transmitter |
| wdt | Watchdog timer |

---

## 4. Variable Naming

### 4.1 Local Variables

Use snake_case for local variables:

```c
/* Correct */
uint32_t sensor_reading;
float voltage_level;
bool is_valid;
int32_t loop_count;

/* Incorrect */
uint32_t SensorReading;    /* No PascalCase */
float voltageLevel;        /* No camelCase */
bool isvalid;              /* Missing underscore */
```

### 4.2 Global Variables

Prefix with `g_` and use snake_case:

```c
/* Correct */
static SystemState_t g_system_state;
static uint32_t g_boot_count;

/* Incorrect */
SystemState_t system_state;  /* Missing g_ prefix */
uint32_t gBootCount;         /* No camelCase */
```

### 4.3 Static Variables

Prefix with `s_` for file-scope static:

```c
/* Correct */
static uint32_t s_last_sample_time;
static bool s_initialized = false;

/* Incorrect */
static uint32_t lastSampleTime;  /* Wrong style */
```

### 4.4 Pointer Variables

Suffix with `_ptr` or use Hungarian notation:

```c
/* Correct */
uint8_t *buffer_ptr;
const char *message_str;
SensorData_t *p_sensor_data;

/* Incorrect */
uint8_t *buffer;  /* Unclear it's a pointer */
```

### 4.5 Boolean Variables

Prefix with `is_`, `has_`, `can_`, `should_`:

```c
/* Correct */
bool is_initialized;
bool has_valid_data;
bool can_transmit;
bool should_reset;

/* Incorrect */
bool initialized;   /* Unclear it's boolean */
bool valid_data;    /* Not a question */
```

---

## 5. Constant Naming

### 5.1 Compile-Time Constants

Use UPPER_SNAKE_CASE:

```c
/* Correct */
static const uint32_t MAX_SENSOR_COUNT = 6;
static const float VOLTAGE_THRESHOLD = 7.0f;

/* Incorrect */
static const uint32_t max_sensor_count = 6;  /* Not uppercase */
```

### 5.2 Enumeration Values

Use UPPER_SNAKE_CASE with type prefix:

```c
/* Correct */
typedef enum {
    POWER_MODE_SAFE,
    POWER_MODE_IDLE,
    POWER_MODE_ACTIVE,
    POWER_MODE_BURST
} PowerMode_t;

typedef enum {
    FAULT_CODE_NONE,
    FAULT_CODE_UNDERVOLTAGE,
    FAULT_CODE_OVERTEMP
} FaultCode_t;

/* Incorrect */
typedef enum {
    Safe,           /* PascalCase */
    idle,           /* lowercase */
    ACTIVE_MODE     /* Missing type prefix */
} PowerMode_t;
```

---

## 6. Macro Naming

### 6.1 Object-Like Macros

Use UPPER_SNAKE_CASE:

```c
/* Correct */
#define BEACON_INTERVAL_MS      30000U
#define MAX_RETRY_COUNT         3U
#define VOLTAGE_SCALE_FACTOR    1000.0f

/* Incorrect */
#define beacon_interval         30000   /* Not uppercase */
#define BeaconInterval          30000   /* PascalCase */
```

### 6.2 Function-Like Macros

Use UPPER_SNAKE_CASE with MODULE_ prefix:

```c
/* Correct */
#define SENSOR_GET_CHANNEL(x)   ((x) & 0x0FU)
#define EPS_VOLTAGE_TO_MV(v)    ((uint32_t)((v) * 1000.0f))
#define ARRAY_SIZE(arr)         (sizeof(arr) / sizeof((arr)[0]))

/* Incorrect */
#define getChannel(x)           ((x) & 0x0F)  /* Function style */
#define voltage_to_mv(v)        ((v) * 1000)  /* Not uppercase */
```

### 6.3 Macro Parameters

Wrap parameters in parentheses:

```c
/* Correct */
#define MIN(a, b)    (((a) < (b)) ? (a) : (b))
#define CLAMP(x, lo, hi)  (((x) < (lo)) ? (lo) : (((x) > (hi)) ? (hi) : (x)))

/* Incorrect - potential issues */
#define MIN(a, b)    (a < b ? a : b)  /* Operator precedence issues */
```

---

## 7. Type Naming

### 7.1 Typedef Structs

Use PascalCase with `_t` suffix:

```c
/* Correct */
typedef struct {
    float x;
    float y;
    float z;
} Vector3_t;

typedef struct {
    uint32_t timestamp;
    Vector3_t magnetic_field;
    float temperature;
} SensorData_t;

/* Incorrect */
typedef struct {
    float x, y, z;
} vector3;              /* No _t suffix, lowercase */

typedef struct {
    float x, y, z;
} Vector3;              /* No _t suffix */
```

### 7.2 Typedef Enums

Use PascalCase with `_t` suffix:

```c
/* Correct */
typedef enum {
    RESULT_OK,
    RESULT_ERROR,
    RESULT_TIMEOUT
} SmartQsoResult_t;

/* Incorrect */
typedef enum {
    OK, ERROR, TIMEOUT
} result;               /* No prefix, no _t */
```

### 7.3 Function Pointer Types

Use PascalCase with descriptive name and `Fn_t` suffix:

```c
/* Correct */
typedef void (*FaultCallbackFn_t)(FaultCode_t code);
typedef SmartQsoResult_t (*SensorReadFn_t)(uint8_t channel, uint16_t *value);

/* Incorrect */
typedef void (*callback)(int);  /* Not descriptive */
```

---

## 8. Function Naming

### 8.1 Module Functions

Use snake_case with module prefix:

```c
/* Correct */
SmartQsoResult_t sensors_init(void);
SmartQsoResult_t sensors_read_magnetometer(Vector3_t *mag);
void eps_set_power_mode(PowerMode_t mode);
bool fault_mgmt_check_voltage(float voltage);

/* Incorrect */
SmartQsoResult_t init(void);           /* No module prefix */
SmartQsoResult_t sensorInit(void);     /* camelCase */
void SetPowerMode(PowerMode_t mode);   /* PascalCase */
```

### 8.2 Static (Private) Functions

Prefix with module and use snake_case:

```c
/* Correct */
static SmartQsoResult_t sensors_read_raw_adc(uint8_t channel, uint16_t *value);
static void eps_update_state_machine(void);

/* Incorrect */
static SmartQsoResult_t readRawAdc(uint8_t channel, uint16_t *value);
```

### 8.3 HAL Functions

Prefix with `hal_` and subsystem:

```c
/* Correct */
SmartQsoResult_t hal_gpio_write(HalGpioPin_t pin, bool state);
SmartQsoResult_t hal_uart_read(HalUartPort_t port, uint8_t *data, size_t len);
SmartQsoResult_t hal_i2c_transfer(HalI2cBus_t bus, uint8_t addr, ...);

/* Incorrect */
SmartQsoResult_t gpio_write(int pin, bool state);  /* Missing hal_ */
```

### 8.4 Interrupt Service Routines

Suffix with `_isr` or `_IRQHandler`:

```c
/* Correct */
void uart1_rx_isr(void);
void USART1_IRQHandler(void);  /* CMSIS naming */
void tim2_update_isr(void);

/* Incorrect */
void uartHandler(void);  /* Unclear it's ISR */
```

---

## 9. File Naming

### 9.1 Source Files

Use snake_case with `.c` extension:

```c
/* Correct */
sensors.c
eps_control.c
fault_mgmt.c
hal_stm32l4.c

/* Incorrect */
Sensors.c        /* PascalCase */
epsControl.c     /* camelCase */
```

### 9.2 Header Files

Match source file name with `.h` extension:

```c
/* Correct */
sensors.h
eps_control.h
hal/hal_gpio.h

/* Incorrect */
sensor.h         /* Doesn't match sensors.c */
```

### 9.3 Test Files

Prefix with `test_`:

```c
/* Correct */
test_sensors.c
test_eps_control.c
test_fault_mgmt.c

/* Incorrect */
sensors_test.c   /* Prefix, not suffix */
```

---

## 10. Documentation Naming

### 10.1 Doxygen Comments

Use @brief, @param, @return:

```c
/**
 * @brief Initialize sensor subsystem
 *
 * @param[in] config Pointer to sensor configuration
 * @param[out] handle Handle to initialized sensor context
 *
 * @return RESULT_OK on success, error code otherwise
 *
 * @note Must be called before any other sensor functions
 * @warning Config must remain valid for lifetime of sensor
 */
SmartQsoResult_t sensors_init(const SensorConfig_t *config, SensorHandle_t *handle);
```

### 10.2 TODO Comments

Include issue number or owner:

```c
/* Correct */
/* TODO(#123): Implement power-saving sleep mode */
/* TODO(jsmith): Add bounds checking for array access */
/* FIXME: Race condition in ISR handler */

/* Incorrect */
/* TODO: fix this */  /* Not actionable */
```

---

## 11. Examples

### 11.1 Complete Module Example

```c
/**
 * @file sensors.c
 * @brief Sensor subsystem implementation
 */

#include "sensors.h"
#include "hal/hal_i2c.h"

/* Module constants */
#define SENSOR_I2C_ADDR         0x1EU
#define SENSOR_READ_TIMEOUT_MS  100U

/* Module types */
typedef struct {
    bool is_initialized;
    uint32_t sample_count;
} SensorState_t;

/* File-scope variables */
static SensorState_t s_sensor_state;

/* Private function declarations */
static SmartQsoResult_t sensors_read_raw(uint8_t reg, uint8_t *data, size_t len);

/* Public functions */
SmartQsoResult_t sensors_init(void)
{
    SmartQsoResult_t result;

    result = hal_i2c_init(HAL_I2C_BUS_0);
    if (result != RESULT_OK) {
        return result;
    }

    s_sensor_state.is_initialized = true;
    s_sensor_state.sample_count = 0U;

    return RESULT_OK;
}

/* Private functions */
static SmartQsoResult_t sensors_read_raw(uint8_t reg, uint8_t *data, size_t len)
{
    return hal_i2c_read(HAL_I2C_BUS_0, SENSOR_I2C_ADDR, reg, data, len);
}
```

---

## 12. Enforcement

### 12.1 Automated Checks

| Tool | Purpose |
|------|---------|
| clang-tidy | Naming convention checks |
| cppcheck | Identifier style |
| Custom script | Prefix validation |

### 12.2 clang-tidy Configuration

```yaml
# .clang-tidy excerpt
Checks: '-*,readability-identifier-naming'
CheckOptions:
  - key: readability-identifier-naming.GlobalVariablePrefix
    value: 'g_'
  - key: readability-identifier-naming.StaticVariablePrefix
    value: 's_'
  - key: readability-identifier-naming.FunctionCase
    value: 'lower_case'
  - key: readability-identifier-naming.TypedefCase
    value: 'CamelCase'
  - key: readability-identifier-naming.TypedefSuffix
    value: '_t'
```

---

## 13. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
