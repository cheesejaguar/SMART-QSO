# Include Dependencies
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-008
**Revision:** B
**Date:** 2026-10-24
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

**Revision B (2026-10-24).** The flight software is Rust (`software/rust/README.md`). Its module dependencies are
crate and module dependencies (§3, §5), and Cargo refuses a dependency cycle between crates. The C headers in
`software/flight/include` stay the C ABI of the Rust modules. §6 to §10 are the rules for those headers, for the
remaining C file `software/flight/src/flight_log_fmt.c` and for the C test suites. Revision A's layer diagram, directory
tree and dependency matrix named C files, headers and a FreeRTOS wrapper that no longer exist, or never existed
(`smart_qso_types.h`, `smart_qso_config.h`, `crc32.h`, `time_utils.h`, `hal_watchdog.h`, `hal_freertos.c`). §3 to §5
are replaced by the structure as built.

---

## 3. Architecture Layers

### 3.1 Layer Hierarchy

```
┌──────────────────────────────────────────────────────────────────┐
│ Programs: sqso-sim (host simulation)  firmware (STM32L4R5 image)  │
│           sqso-clib (static library for the C test suites)        │
├──────────────────────────────────────────────────────────────────┤
│ sqso-ffi: the flight software behind the C headers                │
│   executive, module state, C ABI exports, store, hal_sim (host)   │
├──────────────────────────────────────────────────────────────────┤
│ sqso-fsw: flight logic of every module (no I/O, no unsafe)        │
├──────────────────────────────────────────────────────────────────┤
│ sqso-protocol: wire formats (no I/O, no unsafe)                   │
├──────────────────────────────────────────────────────────────────┤
│ HAL (hal/*.h): hal_sim.rs on the host; firmware/src on the target │
│ Platform: embassy-stm32, embassy-executor, cortex-m-rt (target);  │
│           libc (host)                                             │
└──────────────────────────────────────────────────────────────────┘
```

`sqso-ffi` declares the HAL functions it calls as `extern "C"` (`software/rust/sqso-ffi/src/c.rs`). The linker resolves
them to `hal_sim.rs` in the host builds and to the firmware's `#[no_mangle]` functions in the image. The flight logic
therefore does not depend on which HAL is linked.

### 3.2 Dependency Rules

| From | May use | Must not use | Enforced by |
|------|---------|--------------|-------------|
| Programs | `sqso-ffi` | — | Cargo |
| `sqso-ffi` | `sqso-fsw`, `sqso-protocol`, the HAL through `extern "C"`, libc (host only) | The programs | Cargo (no cycle), `cfg(not(target_os = "none"))` for libc |
| `sqso-fsw` | `sqso-protocol`, `zeroize` | `sqso-ffi`, the HAL, any I/O | Cargo; `#![no_std]` and `#![forbid(unsafe_code)]` (no `extern "C"` calls) |
| `sqso-protocol` | `crc`, `hmac`, `sha2`, `zeroize` | Every flight crate | Cargo; `#![no_std]`, `#![forbid(unsafe_code)]` |
| `sqso-py` | `sqso-protocol`, `pyo3` | The flight logic | Cargo |

Within a crate, modules use each other through `crate::` paths. No tool enforces a layering inside `sqso-ffi`; review
does.

---

## 4. Header File Organization

### 4.1 Directory Structure

```
software/flight/
├── include/                 # The C ABI of the Rust modules (unchanged headers)
│   ├── smart_qso.h          # Result codes, constants, mission clock, assertions
│   ├── <module>.h           # One per module: adcs_control.h, ax25.h, beacon.h, ..., watchdog_mgr.h
│   ├── rust_contract.h      # What the Rust library assumes (checked by test_rust_contract.c)
│   └── hal/                 # hal.h, hal_adc.h, hal_flash.h, hal_gpio.h, hal_i2c.h, hal_radio.h,
│                            # hal_reset.h, hal_sim.h, hal_spi.h, hal_timer.h, hal_uart.h, hal_wdt.h
├── src/
│   └── flight_log_fmt.c     # The printf-style shim of the flight log (the only C left)
└── tests/                   # The C test suites (CMocka), the specification of the Rust

software/rust/
├── sqso-protocol/src/       # Wire formats
├── sqso-fsw/src/            # Flight logic, one module per flight module
├── sqso-ffi/src/            # One module per C header, the executive, hal_sim.rs, store.rs
├── sqso-clib/, sqso-sim/, sqso-py/, fuzz/
└── firmware/src/            # The HAL on the STM32L4R5 and the Embassy main task
```

### 4.2 Header Naming Convention

| Pattern | Purpose | Example |
|---------|---------|---------|
| `smart_qso.h` | Project-wide header | smart_qso.h |
| `<module>.h` | Module public interface, implemented by `software/rust/sqso-ffi/src/<module>.rs` | sensors.h |
| `hal/hal_*.h` | HAL interface | hal/hal_gpio.h |

---

## 5. Include Dependency Matrix

### 5.1 Crate Dependencies

| Crate | Depends on (flight and third-party) |
|-------|--------------------------------------|
| `sqso-protocol` | `crc`, `hmac`, `sha2`, `zeroize` |
| `sqso-fsw` | `sqso-protocol`, `zeroize` |
| `sqso-ffi` | `sqso-protocol`, `sqso-fsw`, `sha2`, `zeroize`; `libc` on the host |
| `sqso-clib`, `sqso-sim` | `sqso-ffi` |
| firmware | `sqso-ffi`, `sqso-protocol`, embassy-stm32, embassy-executor, embassy-time, embassy-embedded-hal, cortex-m, cortex-m-rt, critical-section, embedded-io, static_cell |
| `sqso-py` | `sqso-protocol`, `pyo3` |

Sources: the `Cargo.toml` of each crate; versions are pinned in `software/rust/Cargo.lock` and
`software/rust/firmware/Cargo.lock`.

### 5.2 Header Dependencies

`smart_qso.h` is included by 25 of the 36 headers in `include/` and by the HAL headers. Seven includes go from one
module header to another: `beacon.h` → `ax25.h`; `cmd_auth.h` and `contact_monitor.h` → `nvm_record.h`;
`cmd_handler.h` → `input_validation.h` and `cmd_dictionary.h`; `system_state.h` → `state_machine.h`; `uplink.h` →
`cmd_handler.h`. `hal/hal.h` collects the HAL headers, and `hal/hal_sim.h` includes three of them. None of these
includes forms a cycle (checked by reading the includes, 2026-10-24).

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

/* 3. Platform headers (none in this project; there is no RTOS) */

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
| No circular dependencies (crates) | Crate graph of §5.1 | Cargo refuses a cycle |
| No circular dependencies (C headers) | Forward declarations | Build test |
| Layer compliance (crates) | §3.2 | Cargo; `#![forbid(unsafe_code)]` and `#![no_std]` in `sqso-protocol` and `sqso-fsw` |
| Layer compliance (inside `sqso-ffi`) | `crate::` paths | Code review |
| Include guards | Standard pattern | Code review |
| Include order | Defined sequence | clang-format |

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-24 | The flight software is Rust: §3 to §5 replaced by the structure as built (crate layers and their enforcement, the header and crate directories, crate and header dependencies); FreeRTOS removed from the include order; §11 updated. §6 to §10 kept for the C headers, the shim and the tests | SMART-QSO Team |
