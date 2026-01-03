# Include Dependencies
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-008
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the include hierarchy and dependency structure for SMART-QSO flight software to ensure clean architecture and prevent circular dependencies.

## 2. Scope

This document covers:
- Header file organization
- Include dependency rules
- Module dependency graph
- Layered architecture compliance

---

## 3. Architecture Layers

### 3.1 Layer Hierarchy

```
┌─────────────────────────────────────────────────┐
│                  Application                     │
│  (main.c, mission_data.c, beacon.c)             │
├─────────────────────────────────────────────────┤
│                   Services                       │
│  (sensors.c, fault_mgmt.c, adcs_control.c)      │
├─────────────────────────────────────────────────┤
│                   Drivers                        │
│  (eps_control.c, uart_comm.c)                   │
├─────────────────────────────────────────────────┤
│              Hardware Abstraction               │
│  (hal_*.c, hal_*.h)                             │
├─────────────────────────────────────────────────┤
│               Platform / RTOS                   │
│  (FreeRTOS, CMSIS)                              │
└─────────────────────────────────────────────────┘
```

### 3.2 Dependency Rules

| From Layer | May Include | Must Not Include |
|------------|-------------|------------------|
| Application | Services, Drivers, HAL | - |
| Services | Drivers, HAL | Application |
| Drivers | HAL | Application, Services |
| HAL | Platform | Application, Services, Drivers |
| Platform | - | All above |

---

## 4. Header File Organization

### 4.1 Directory Structure

```
software/flight/
├── include/
│   ├── smart_qso.h          # Master header
│   ├── smart_qso_types.h    # Common types
│   ├── smart_qso_config.h   # Configuration
│   ├── sensors.h            # Sensor interface
│   ├── eps_control.h        # EPS interface
│   ├── fault_mgmt.h         # Fault management
│   ├── uart_comm.h          # UART interface
│   ├── mission_data.h       # Mission data
│   ├── beacon.h             # Beacon generation
│   ├── adcs_control.h       # ADCS interface
│   ├── crc32.h              # CRC utilities
│   └── time_utils.h         # Time utilities
├── include/hal/
│   ├── hal_types.h          # HAL type definitions
│   ├── hal_gpio.h           # GPIO interface
│   ├── hal_uart.h           # UART HAL
│   ├── hal_spi.h            # SPI HAL
│   ├── hal_i2c.h            # I2C HAL
│   ├── hal_timer.h          # Timer HAL
│   ├── hal_watchdog.h       # Watchdog HAL
│   ├── hal_flash.h          # Flash HAL
│   └── hal_adc.h            # ADC HAL
└── src/
    └── hal/
        ├── hal_sim.c        # Linux simulation
        ├── hal_stm32l4.c    # STM32L4 implementation
        └── hal_freertos.c   # FreeRTOS wrapper
```

### 4.2 Header Naming Convention

| Pattern | Purpose | Example |
|---------|---------|---------|
| `smart_qso_*.h` | Project-wide headers | smart_qso_types.h |
| `<module>.h` | Module public interface | sensors.h |
| `hal_*.h` | HAL interface | hal_gpio.h |
| `*_internal.h` | Module internals | sensors_internal.h |

---

## 5. Include Dependency Matrix

### 5.1 Module Dependencies

| Module | Depends On |
|--------|------------|
| main.c | smart_qso.h, sensors.h, eps_control.h, fault_mgmt.h, beacon.h, adcs_control.h, mission_data.h |
| sensors.c | sensors.h, hal_i2c.h, hal_adc.h, hal_timer.h |
| eps_control.c | eps_control.h, hal_gpio.h, hal_adc.h |
| fault_mgmt.c | fault_mgmt.h, hal_flash.h, hal_watchdog.h |
| uart_comm.c | uart_comm.h, hal_uart.h, crc32.h |
| mission_data.c | mission_data.h, hal_flash.h, crc32.h |
| beacon.c | beacon.h, uart_comm.h, crc32.h, time_utils.h |
| adcs_control.c | adcs_control.h, sensors.h, hal_timer.h |
| crc32.c | crc32.h |
| time_utils.c | time_utils.h, hal_timer.h |
| hal_sim.c | hal_*.h |

### 5.2 Dependency Graph

```
                    ┌─────────────┐
                    │   main.c    │
                    └──────┬──────┘
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
    ┌────────────┐  ┌────────────┐  ┌────────────┐
    │ sensors.c  │  │  beacon.c  │  │   adcs_    │
    │            │  │            │  │ control.c  │
    └─────┬──────┘  └─────┬──────┘  └─────┬──────┘
          │               │               │
          ▼               ▼               ▼
    ┌────────────┐  ┌────────────┐  ┌────────────┐
    │    eps_    │  │   uart_    │  │   fault_   │
    │ control.c  │  │  comm.c    │  │  mgmt.c    │
    └─────┬──────┘  └─────┬──────┘  └─────┬──────┘
          │               │               │
          └───────────────┼───────────────┘
                          ▼
                   ┌────────────┐
                   │   crc32.c  │
                   │ time_utils │
                   └─────┬──────┘
                         ▼
                   ┌────────────┐
                   │    HAL     │
                   │ hal_*.h    │
                   └────────────┘
```

---

## 6. Header Include Rules

### 6.1 Include Order

Each source file SHALL include headers in this order:

```c
/* 1. Own header (for .c files) */
#include "module.h"

/* 2. Standard C library */
#include <stdint.h>
#include <stdbool.h>
#include <string.h>

/* 3. Platform headers */
#include "FreeRTOS.h"
#include "task.h"

/* 4. HAL headers */
#include "hal/hal_gpio.h"
#include "hal/hal_uart.h"

/* 5. Project headers (alphabetical) */
#include "crc32.h"
#include "fault_mgmt.h"
```

### 6.2 Include Guards

All headers SHALL use include guards:

```c
#ifndef SMART_QSO_MODULE_NAME_H
#define SMART_QSO_MODULE_NAME_H

#ifdef __cplusplus
extern "C" {
#endif

/* Header content */

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_MODULE_NAME_H */
```

### 6.3 Prohibited Includes

| Prohibited | Reason | Alternative |
|------------|--------|-------------|
| Application in HAL | Layer violation | Define callback interface |
| Internal headers externally | Encapsulation | Use public interface |
| Circular includes | Compilation failure | Forward declarations |
| Unused includes | Code bloat | Remove |

---

## 7. Forward Declarations

### 7.1 When to Use

Use forward declarations when:
- Only pointers are needed
- Breaking circular dependencies
- Reducing compile time

```c
/* Forward declaration instead of include */
struct SensorData;  /* Only need pointer */

/* Now can declare function */
void process_sensor(const struct SensorData *data);
```

### 7.2 Forward Declaration Example

```c
/* In eps_control.h */
#ifndef SMART_QSO_EPS_CONTROL_H
#define SMART_QSO_EPS_CONTROL_H

/* Forward declaration - avoids including full header */
typedef struct PowerState PowerState_t;

/* Function using forward-declared type */
SmartQsoResult_t eps_get_state(PowerState_t *state);

#endif
```

---

## 8. Module Interface Design

### 8.1 Public Interface Pattern

```c
/* sensors.h - Public interface */
#ifndef SMART_QSO_SENSORS_H
#define SMART_QSO_SENSORS_H

#include "smart_qso_types.h"

/* Opaque types */
typedef struct SensorContext SensorContext_t;

/* Public functions */
SmartQsoResult_t sensors_init(SensorContext_t *ctx);
SmartQsoResult_t sensors_read_all(SensorContext_t *ctx, SensorData_t *data);
void sensors_deinit(SensorContext_t *ctx);

#endif
```

### 8.2 Internal Implementation

```c
/* sensors_internal.h - Internal details */
#ifndef SMART_QSO_SENSORS_INTERNAL_H
#define SMART_QSO_SENSORS_INTERNAL_H

#include "sensors.h"
#include "hal/hal_i2c.h"
#include "hal/hal_adc.h"

/* Full structure definition */
struct SensorContext {
    HalI2cHandle_t i2c_handle;
    HalAdcHandle_t adc_handle;
    bool initialized;
    uint32_t sample_count;
};

/* Internal functions */
static SmartQsoResult_t read_magnetometer(SensorContext_t *ctx, Vector3_t *mag);
static SmartQsoResult_t read_sun_sensors(SensorContext_t *ctx, float *sun);

#endif
```

---

## 9. Circular Dependency Prevention

### 9.1 Detection

Build with:
```bash
# GCC circular dependency check
gcc -H source.c 2>&1 | grep "Multiple include"
```

### 9.2 Resolution Strategies

| Strategy | When to Use |
|----------|-------------|
| Forward declaration | Types only used as pointers |
| Split header | Mixed concerns in one header |
| Callback interface | Upward dependencies |
| Intermediate module | Shared functionality |

### 9.3 Example: Breaking Circular Dependency

```c
/* BEFORE: Circular dependency */
/* sensors.h includes fault_mgmt.h */
/* fault_mgmt.h includes sensors.h */

/* AFTER: Using callback */
/* sensors.h - no fault_mgmt.h */
typedef void (*FaultCallback_t)(FaultCode_t code);
void sensors_set_fault_callback(FaultCallback_t cb);

/* In main.c - wire up */
sensors_set_fault_callback(fault_mgmt_report);
```

---

## 10. Dependency Analysis Tools

### 10.1 Include-What-You-Use (IWYU)

```bash
# Run IWYU
include-what-you-use -Iinclude source.c
```

### 10.2 Graphviz Dependency Graph

```bash
# Generate dependency graph
gcc -MM src/*.c | ./scripts/deps2dot.sh > deps.dot
dot -Tpng deps.dot -o deps.png
```

### 10.3 Verification Script

```bash
#!/bin/bash
# verify_includes.sh

echo "Checking include hierarchy..."

# Check for layer violations
if grep -r "#include.*main.h" src/hal/; then
    echo "ERROR: HAL including application header"
    exit 1
fi

echo "Include hierarchy OK"
```

---

## 11. Compliance Matrix

| Requirement | Implementation | Verification |
|-------------|----------------|--------------|
| No circular dependencies | Forward declarations | Build test |
| Layer compliance | Dependency rules | Static analysis |
| Include guards | Standard pattern | Code review |
| Include order | Defined sequence | clang-format |

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
