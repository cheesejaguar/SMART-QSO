# Coding Standards
## SMART-QSO Flight Software

**Document Number:** SQSO-CS-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** JPL D-60411, MISRA C:2012

---

## 1. Purpose

This document defines the coding standards for SMART-QSO flight software development, based on JPL Institutional Coding Standard (Power of Ten Rules) and MISRA C:2012 guidelines.

## 2. Applicable Standards

| Standard | Application |
|----------|-------------|
| JPL D-60411 | Primary C coding rules (Power of Ten) |
| MISRA C:2012 | Safety-critical coding guidelines |
| NASA-STD-8739.8 | Software assurance |
| C11 (ISO/IEC 9899:2011) | Language standard |

## 3. JPL Power of Ten Rules

### Rule 1: Simple Control Flow

**Requirement:** Restrict to simple control flow constructs.

- No `goto` statements
- No `setjmp`/`longjmp`
- No direct or indirect recursion

```c
// GOOD: Simple loop
for (uint32_t i = 0; i < MAX_SENSORS; i++) {
    process_sensor(i);
}

// BAD: goto
goto error_handler;  // PROHIBITED
```

### Rule 2: Fixed Loop Bounds

**Requirement:** All loops must have a fixed upper bound.

```c
// GOOD: Fixed bound
#define MAX_ITERATIONS 1000
for (uint32_t i = 0; i < MAX_ITERATIONS; i++) {
    if (condition_met()) break;
}

// BAD: Unbounded loop
while (1) {  // Only allowed in main loop with watchdog
    // ...
}
```

### Rule 3: No Dynamic Memory After Initialization

**Requirement:** No dynamic memory allocation after initialization.

```c
// GOOD: Static allocation
static uint8_t buffer[BUFFER_SIZE];

// BAD: Dynamic allocation in operational code
char *ptr = malloc(size);  // PROHIBITED after init
```

### Rule 4: Function Length Limit

**Requirement:** Functions shall not exceed 60 lines of code.

- Excludes blank lines and comments
- Split large functions into smaller helpers
- Each function should do one thing well

### Rule 5: Assertions

**Requirement:** Minimum 2 assertions per function on average.

```c
SmartQsoResult_t sensor_read(uint8_t id, double *value) {
    SMART_QSO_REQUIRE(id < MAX_SENSORS, "Invalid sensor ID");
    SMART_QSO_REQUIRE_NOT_NULL(value);

    // ... implementation ...

    SMART_QSO_ENSURE(result == SMART_QSO_OK, "Read failed");
    return result;
}
```

### Rule 6: Minimal Scope

**Requirement:** Declare data at smallest possible scope.

```c
// GOOD: Declared at point of use
for (uint32_t i = 0; i < count; i++) {
    double value = get_value(i);
    process(value);
}

// BAD: Declared too early
double value;  // Not used until line 50
// ... 40 lines of code ...
value = get_value(i);
```

### Rule 7: Check Return Values

**Requirement:** Check return value of all non-void functions.

```c
// GOOD: Return value checked
SmartQsoResult_t result = sensor_init();
if (result != SMART_QSO_OK) {
    fault_log(FAULT_TYPE_INIT, "Sensor init failed");
    return result;
}

// BAD: Return value ignored
sensor_init();  // PROHIBITED
```

### Rule 8: Limited Preprocessor

**Requirement:** Limit preprocessor to includes and simple macros.

```c
// GOOD: Simple conditional compilation
#ifdef FLIGHT_BUILD
    #define DEBUG_PRINT(x)  // No-op in flight
#else
    #define DEBUG_PRINT(x) printf(x)
#endif

// BAD: Complex preprocessor logic
#define COMPLEX_MACRO(x) \
    do { \
        /* 20 lines of code */ \
    } while(0)  // AVOID
```

### Rule 9: Restrict Pointer Use

**Requirement:** Limit pointer use; no more than one level of dereferencing.

```c
// GOOD: Single level
Sensor_t *sensor = &sensors[id];
sensor->value = reading;

// BAD: Multiple levels
Sensor_t **ptr_ptr = &sensor_ptr;  // AVOID
(**ptr_ptr).value = reading;
```

### Rule 10: Compile with All Warnings

**Requirement:** Compile with all warnings enabled; treat warnings as errors.

```cmake
add_compile_options(
    -Wall -Wextra -Wpedantic -Werror
    -Wconversion -Wshadow -Wcast-qual
    -Wstrict-prototypes -Wmissing-prototypes
)
```

## 4. Naming Conventions

### 4.1 Functions

```c
// Format: module_action_object()
SmartQsoResult_t eps_control_payload(bool enable);
SmartQsoResult_t sensor_read_value(uint8_t id, double *value);
void beacon_transmit_frame(const Ax25Frame_t *frame);
```

### 4.2 Types

```c
// Format: PascalCase with _t suffix
typedef struct {
    uint8_t id;
    double value;
} Sensor_t;

typedef enum {
    POWER_MODE_SAFE = 0,
    POWER_MODE_IDLE = 1,
    POWER_MODE_ACTIVE = 2
} PowerMode_t;
```

### 4.3 Constants and Macros

```c
// Format: UPPER_SNAKE_CASE
#define MAX_SENSORS           32
#define BEACON_INTERVAL_MS    60000
#define SMART_QSO_VERSION     "0.2.0"
```

### 4.4 Variables

```c
// Local variables: lower_snake_case
uint32_t sensor_count = 0;
double current_value = 0.0;

// Static variables: s_ prefix
static uint32_t s_beacon_count = 0;
static bool s_initialized = false;

// Global variables: g_ prefix (avoid when possible)
extern uint32_t g_system_time_ms;
```

### 4.5 Files

```c
// Source files: module_name.c
sensors.c
eps_control.c
fault_mgmt.c

// Header files: module_name.h
sensors.h
eps_control.h
fault_mgmt.h
```

## 5. Header File Structure

```c
/**
 * @file module_name.h
 * @brief Brief description of module
 * @copyright NASA Ames Research Center
 */

#ifndef SMART_QSO_MODULE_NAME_H
#define SMART_QSO_MODULE_NAME_H

#ifdef __cplusplus
extern "C" {
#endif

/* Includes */
#include <stdint.h>
#include <stdbool.h>

/* Constants */
#define MODULE_CONSTANT 42

/* Types */
typedef struct {
    uint32_t field1;
    uint32_t field2;
} ModuleData_t;

/* Function prototypes */
SmartQsoResult_t module_init(void);
SmartQsoResult_t module_process(ModuleData_t *data);

#ifdef __cplusplus
}
#endif

#endif /* SMART_QSO_MODULE_NAME_H */
```

## 6. Source File Structure

```c
/**
 * @file module_name.c
 * @brief Implementation of module functionality
 */

/* Includes - system headers first, then project headers */
#include <string.h>
#include "module_name.h"
#include "smart_qso.h"

/* Private constants */
#define INTERNAL_BUFFER_SIZE 256

/* Private types */
typedef struct {
    uint32_t internal_field;
} InternalState_t;

/* Private variables */
static InternalState_t s_state;
static bool s_initialized = false;

/* Private function prototypes */
static void helper_function(void);

/* Public function implementations */
SmartQsoResult_t module_init(void) {
    memset(&s_state, 0, sizeof(s_state));
    s_initialized = true;
    return SMART_QSO_OK;
}

/* Private function implementations */
static void helper_function(void) {
    // Implementation
}
```

## 7. Documentation Requirements

### 7.1 Function Documentation

```c
/**
 * @brief Read sensor value by ID
 *
 * @param[in]  id     Sensor identifier (0 to MAX_SENSORS-1)
 * @param[out] value  Pointer to store the sensor reading
 *
 * @return SMART_QSO_OK on success
 * @return SMART_QSO_ERROR_INVALID if id out of range
 * @return SMART_QSO_ERROR_NULL_PTR if value is NULL
 *
 * @pre Sensor module must be initialized
 * @post value contains the current sensor reading
 *
 * @requirement SRS-SENSOR-001
 */
SmartQsoResult_t sensor_read(uint8_t id, double *value);
```

### 7.2 Requirement Traceability

```c
/**
 * @requirement SRS-001 System shall provide modular architecture
 * @requirement SRS-002 System shall comply with JPL Power of Ten
 */
```

## 8. Error Handling

### 8.1 Return Codes

```c
typedef enum {
    SMART_QSO_OK              =  0,
    SMART_QSO_ERROR           = -1,
    SMART_QSO_ERROR_NULL_PTR  = -2,
    SMART_QSO_ERROR_INVALID   = -3,
    SMART_QSO_ERROR_TIMEOUT   = -4,
    SMART_QSO_ERROR_BUSY      = -5,
    SMART_QSO_ERROR_NO_MEM    = -6,
    SMART_QSO_ERROR_IO        = -7
} SmartQsoResult_t;
```

### 8.2 Error Handling Pattern

```c
SmartQsoResult_t function_with_cleanup(void) {
    SmartQsoResult_t result = SMART_QSO_OK;
    Resource_t *resource = NULL;

    // Acquire resources
    resource = acquire_resource();
    if (resource == NULL) {
        result = SMART_QSO_ERROR_NO_MEM;
        goto cleanup;
    }

    // Do work
    result = do_work(resource);
    if (result != SMART_QSO_OK) {
        goto cleanup;
    }

cleanup:
    if (resource != NULL) {
        release_resource(resource);
    }
    return result;
}
```

## 9. MISRA C:2012 Deviations

| Rule | Deviation | Rationale |
|------|-----------|-----------|
| 21.6 | Use of stdio | Debug builds only, disabled in flight |
| 15.1 | goto for cleanup | Approved pattern for resource cleanup |

## 10. Static Analysis

All code must pass:
- cppcheck with `--enable=all`
- clang-tidy with project configuration
- No warnings with `-Wall -Wextra -Wpedantic`

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
