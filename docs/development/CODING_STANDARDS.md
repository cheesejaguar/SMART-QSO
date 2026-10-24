# Coding Standards
## SMART-QSO Flight Software

**Document Number:** SQSO-CS-001
**Revision:** B
**Date:** 2026-10-24
**Reference:** JPL D-60411, MISRA C:2012, the Rust lints of `software/rust/Cargo.toml`

---

## 1. Purpose

This document defines the coding standards for SMART-QSO flight software development, based on JPL Institutional Coding Standard (Power of Ten Rules) and MISRA C:2012 guidelines.

**Scope (Revision B, 2026-10-24).** The flight software is Rust (`software/rust`). Its coding standard is §11.
Sections 3 to 10 are the C rules, and they now apply only to the C that remains:

- `software/flight/src/flight_log_fmt.c`, the printf-style shim of the flight log (stable Rust cannot define a C
  variadic function);
- the C headers in `software/flight/include`, which are the C ABI of the Rust modules;
- the C test suites in `software/flight/tests`. MISRA C does not apply to them, as before
  (`software/flight/docs/MISRA_DEVIATIONS.md`).

The examples in §3 to §8 illustrate the rules and are not quotations of flight code. The `goto` cleanup pattern in §8.2
conflicts with Rule 1, and the remaining C does not use it.

## 2. Applicable Standards

| Standard | Application |
|----------|-------------|
| JPL D-60411 | Primary C coding rules (Power of Ten) |
| MISRA C:2012 | Safety-critical coding guidelines |
| NASA-STD-8739.8 | Software assurance |
| C11 (ISO/IEC 9899:2011) | Language standard of the remaining C (§1) |
| Rust 2024 edition; toolchain 1.97.0 (`software/rust/rust-toolchain.toml`), minimum 1.88 | Language of the flight software (§11) |
| Lints of `software/rust/Cargo.toml` and `software/rust/clippy.toml` | Machine-checked Rust rules (§11.2) |

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
 * @copyright Copyright (c) 2026 SMART-QSO Team
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

MISRA C applies to the remaining C only. The current deviation records and the shim's baseline of 42 findings are in
`software/flight/docs/MISRA_DEVIATIONS.md`.

| Rule | Deviation | Rationale |
|------|-----------|-----------|
| 21.6 | Use of stdio | Debug builds only, disabled in flight |
| 15.1 | goto for cleanup | Approved pattern for resource cleanup |

## 10. Static Analysis

All C code must pass:
- cppcheck with `--enable=all`
- clang-tidy with project configuration
- No warnings with `-Wall -Wextra -Wpedantic`

In CI (job Static Analysis), cppcheck, clang-tidy (baseline 0) and the function-length ratchet (baseline 0) run on
`software/flight/src`, which now holds only the shim. The Rust checks are in §11.10.

## 11. Rust Coding Standard

### 11.1 Scope

This section applies to every crate in `software/rust`: `sqso-protocol`, `sqso-fsw`, `sqso-ffi`, `sqso-clib`,
`sqso-sim`, `sqso-py`, `fuzz` and `firmware` (`software/rust/README.md`). Test code (`#[cfg(test)]`) in
`sqso-protocol` and `sqso-fsw` may use `unwrap`, `expect`, indexing and `panic!`. Their `lib.rs` allows these lints
only under `cfg(test)`.

### 11.2 Lints

The workspace lints are in `software/rust/Cargo.toml` (`[workspace.lints]`). `software/rust/firmware/Cargo.toml` has
the same set. CI runs clippy with `-D warnings`, so a lint at `warn` also fails the build.

| Lint | Level | Rule |
|------|-------|------|
| `clippy::indexing_slicing` | deny | No `a[i]` or `a[i..j]` that can panic. Use `get()`, `get_mut()`, slice patterns or iterators |
| `clippy::unwrap_used`, `clippy::expect_used` | deny | No `unwrap()` or `expect()`. Handle the `None` or `Err` |
| `clippy::panic`, `clippy::unreachable`, `clippy::todo`, `clippy::unimplemented` | deny | No explicit panics |
| `clippy::too_many_lines` | deny, threshold 60 (`clippy.toml`) | JPL rule 4. Counts code lines, not blank lines or comments |
| `clippy::cast_possible_truncation` | warn (an error in CI) | A narrowing `as` cast needs `try_from`, a saturating conversion, or an `#[allow]` that gives the reason |
| `missing_docs` | warn (an error in CI) | Every public item is documented |
| `unsafe_op_in_unsafe_fn` | deny | Each unsafe operation inside an `unsafe fn` needs its own `unsafe` block, with its `// SAFETY:` comment |

Every `#[allow(...)]` gives its reason in a comment on the same line or the line before. Review checks this; no tool
does. An `#[allow(clippy::indexing_slicing)]` is acceptable only in a `const` item or `const fn`, where `get()` is not
available, with the index bounded by the loop condition. In a `const` item an out-of-bounds index is a compile error,
not a run-time panic.

### 11.3 Function length

No function is longer than 60 lines (`clippy::too_many_lines`). There are two documented exceptions. Each is an
exhaustive `match` with one arm per case, where splitting the function would hide the case analysis:

- `dispatch` in `software/rust/sqso-ffi/src/cmd_handler.rs` (one arm per telecommand);
- `transition` in `software/rust/sqso-fsw/src/mode.rs` (every pair of state and event).

### 11.4 No panics on input

Input from outside a module never causes a panic. That covers frames, lines, files, C arguments and stored records.
Parsers return `Option` or `Result`. Arithmetic on lengths and counters that could wrap uses `checked_*`,
`saturating_*` or 64-bit types. Float-to-integer conversions saturate. Each parser has a fuzz target
(`software/rust/fuzz/fuzz_targets`) that checks its properties, not only that it does not crash.

### 11.5 Overflow checks and panics

The release profiles (`software/rust/Cargo.toml`, `software/rust/firmware/Cargo.toml`) keep `overflow-checks = true`
and use `panic = "abort"`. Debug assertions are off. An overflow or a broken invariant is therefore a panic, never a
wrapped index. A panic is a fatal assertion: `report_panic` in `software/rust/sqso-ffi/src/lib.rs` reports it to the
assertion handler. Then the host aborts. The firmware waits for the IWDG to reset it, or resets itself if the IWDG was
never started. Code must not use panics for control flow.

### 11.6 `unsafe` policy

- `sqso-protocol` and `sqso-fsw` declare `#![forbid(unsafe_code)]`, and CI checks that they keep it. `sqso-py` has no
  `unsafe`, which CI checks with grep.
- `unsafe` is allowed only in `sqso-ffi` and the firmware, and only for these purposes: converting C pointers to
  references after a NULL check (`sqso-ffi/src/ptr.rs`), calling C and HAL functions, holding module state in the
  busy-flag cell (§11.8), accessing registers, exporting the HAL functions, and the two factory-calibration reads in
  `firmware/src/adc.rs`.
- Every `unsafe` block and `unsafe impl` has a `// SAFETY:` comment stating why the operation is sound at that point.
  Every `unsafe fn` has a `# Safety` section stating the caller's obligations. Review checks both. No lint enforces
  them.
- An exported function's pointer contract is the one documented in its C header.

### 11.7 No allocation

No crate uses `alloc`. Buffers are fixed-size arrays, in statics or on the stack, and their bounds are constants. The
ctest `no_dynamic_memory` checks that the library references no `malloc`, `calloc`, `realloc` or `free`. The firmware
links no heap and no C library. CI (job STM32L4 Cross-Compile (library)) checks that the image has no `malloc`, `free`,
`_sbrk` or `printf`.

### 11.8 Module state

Each module's state lives in one `cell::Global` (`software/rust/sqso-ffi/src/cell.rs`). `Global::with` hands out a
single `&mut` while a busy flag is set. A nested call into the same module, from a C callback or an interrupt, gets
`None`. The export then returns `SMART_QSO_ERROR_BUSY` or the getter's default value. A safety-relevant export fails
safe instead: for example, `rf_silence_active()` returns true (`software/rust/README.md`, behaviour change 10).
Callbacks, the fault log and other modules are called with no borrow of the module state held.

### 11.9 C ABI

- Exports are `#[unsafe(no_mangle)] pub extern "C"` functions with the names and types of the C header.
- C enums cross the boundary as `u32`, or `i32` where they have negative values, never as Rust enums. Rust converts
  the value to its own enum and handles an out-of-range value explicitly, with an error result or a default.
- Structs shared with C are `#[repr(C)]` mirrors of the header types. Padding is named fields, so every byte is
  initialized. `software/flight/tests/test_rust_contract.c` (with `test_rust_contract_adcs.c`) compares the size and
  alignment of every mirror, and every constant Rust keeps, with the C headers.
- A `bool` byte read from storage is never reinterpreted as a Rust `bool`. Its value is checked: a record with a byte
  other than 0 or 1 is refused, or the byte is read as `!= 0` (behaviour changes 9 and 19).
- The `-fno-short-enums` flag of the former STM32L4 C build is no longer needed, because no C is cross-compiled. The
  firmware contains no C.

### 11.10 Format, documentation and checks

- `rustfmt` with `software/rust/rustfmt.toml` (width 120). CI runs `cargo fmt --check` for the workspace, `fuzz` and
  `firmware`.
- Module documentation (`//!`) names the C header the module implements and the requirements it traces to.
  `scripts/gen_rtm.py` reads the `@requirement` lines.
- CI runs clippy with `-D warnings` for the host and for `thumbv7em-none-eabihf`, the unsafe check, the unit tests,
  the Rust coverage gate (≥ 90 % of lines), fuzzing (30 s per target), `cargo audit` and CodeQL for Rust
  (`software/rust/README.md`, section CI).

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-24 | The flight software is Rust. New §11, the Rust coding standard: lints, function length and its two exceptions, no panics on input, overflow checks, `unsafe` policy and SAFETY comments, no allocation, module state, C ABI. §1 scope: §3–§10 apply to the remaining C (flight-log shim, headers, tests). §2, §9 and §10 updated | SMART-QSO Team |
