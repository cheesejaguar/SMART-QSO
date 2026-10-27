# JPL Power of Ten Rules Compliance Audit

**Document ID**: SMART-QSO-JPL-POT-001
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Engineering

## 1. Introduction

This document provides a comprehensive compliance audit of the SMART-QSO flight software against the JPL Laboratory for Reliable Software "Power of Ten" coding rules. These rules are derived from MISRA-C and represent best practices for safety-critical embedded systems.

**Reference**: Gerard J. Holzmann, "The Power of 10: Rules for Developing Safety-Critical Code", IEEE Computer, June 2006.

### 1.1 Scope of revision 1.1 (2026-10-24)

Since 2026-10-24 the flight software is Rust (`software/rust/README.md`). The only C left in the flight code is
`software/flight/src/flight_log_fmt.c`, the printf-style shim of the flight log. §2 maps each rule to the Rust and to
that shim and says how each is enforced:

- **compiler**: rustc or the C compiler refuses the code;
- **lint**: a clippy or rustc lint, which CI runs with `-D warnings`, so a warning fails the build;
- **CI**: a CI step or ctest entry outside the compiler and the lints;
- **review**: no tool checks it, and code review is the only control.

The rules were written for C. Where a rule has no direct Rust meaning, the table says what stands in for it. This is a
description of mechanisms and of what the tools check. It is not a certification. §2.2 keeps the measured status of the
C code of 2026-10-15 as history. Sections 3 onward are the original text of revision 1.0.

## 2. Compliance Summary

### 2.1 The Rust flight software and the remaining C (2026-10-24)

| Rule | Rust (`software/rust`) | Enforced by | Remaining C (`flight_log_fmt.c`) | Enforced by |
|------|------------------------|-------------|----------------------------------|-------------|
| 1. Simple control flow | Rust has no `goto`, `setjmp` or `longjmp`. Recursion is checked on the flight image: `scripts/stack_usage.py` builds the image's call graph (direct and tail calls, and indirect calls resolved by a reviewed table) and fails on any cycle. On 2026-10-25 the only cycle is the panic handler entered once more by a panic while it reports one; the firmware guards it (`PANICKING`, `software/rust/firmware/src/main.rs`) and the analysis bounds it at one nesting. The recursion that registered C callbacks allow by design (`software/rust/README.md`, behaviour change 15) is not in the image: the flight configuration registers only the watchdog pre-reset callback, which saves mission data and the EPS state | compiler (no `goto`); CI on the image (call-graph cycle check, STM32L4 Cross-Compile) | Two functions, no `goto`, no recursion (`log_write` calls `log_vwrite`) | review |
| 2. Fixed loop bounds | `for` loops run over fixed-size arrays, slices and constant ranges, and `while` loops over an index or a shrinking slice pattern. The other `loop`s end on a count, a deadline or the end of input. The exceptions are the main loops, which run until stopped, and the waits for the watchdog reset (panic handlers, `wdt_mgr_force_reset`). Inventory: `software/flight/docs/LOOP_BOUNDS.md` | review; fuzzing exercises the parser loops on arbitrary input | No loops | review |
| 3. No dynamic memory | No crate uses `alloc`: the crates are `no_std` with no allocator, and the firmware links no heap and no C library | CI: ctest `no_dynamic_memory` (library); image check for `malloc`, `free`, `_sbrk`, `printf` (job STM32L4 Cross-Compile (library)) | No allocation (`vsnprintf` into a stack buffer) | CI: ctest `no_dynamic_memory` |
| 4. Function length ≤ 60 lines | `clippy::too_many_lines`, denied, threshold 60 (`software/rust/clippy.toml`). Two documented exceptions, each one exhaustive `match`: `dispatch` (`sqso-ffi/src/cmd_handler.rs`) and `transition` (`sqso-fsw/src/mode.rs`) | lint | `software/flight/scripts/function_length.py`; count held at 0 | CI (ratchet `function-length.baseline` = 0) |
| 5. Assertions | Assertion density is not measured. Indexing that can fail is denied by lint, and code uses `get()` and checked arithmetic. Integer overflow checks are on in the release profile (`overflow-checks = true`). A panic is a fatal assertion (`report_panic` in `sqso-ffi/src/lib.rs`). Parsers return `Result`/`Option` and are fuzzed with property checks | compiler (bounds, overflow at run time); lint (`indexing_slicing`, `unwrap_used`, `panic`); CI (fuzzing) | Not measured | — |
| 6. Smallest scope | Block scope. Module state is private to its module, in one `cell::Global` per module in `sqso-ffi` | compiler (visibility, borrow rules) | One stack buffer in the block that uses it | review |
| 7. Check return values | `Result` and `Option` are `#[must_use]`, and a value discarded on purpose is written `let _ =`. **Not covered by a tool:** the C-ABI functions return plain `i32` or `u32` codes, which are not `#[must_use]`, so a Rust call to one of them that ignores the code compiles without a warning | lint (`unused_must_use` for `Result`/`Option`); review (C-ABI codes) | The `vsnprintf` result is passed on; `flight_log_record` result returned | cppcheck, clang-tidy (baseline 0) |
| 8. Limited preprocessor | No preprocessor. `#[cfg]` selects host or target (`target_os = "none"`), tests and the `flight` feature. One `macro_rules!` (`report!` in `sqso-ffi/src/uart_comm.rs`) | review | Includes and one `#pragma` pair around `vsnprintf` | MISRA addon (42 findings baseline); review |
| 9. Restrict pointers | No raw-pointer dereference outside `sqso-ffi` and the firmware: `sqso-protocol` and `sqso-fsw` declare `#![forbid(unsafe_code)]`. In `sqso-ffi`, C pointers become references after a NULL check (`ptr.rs`). Each `unsafe` block carries a `// SAFETY:` comment | compiler; CI (forbid check); review (SAFETY comments) | Pointer arguments passed through to Rust, which checks them | review |
| 10. All warnings, zero warnings | clippy with `-D warnings` for the workspace (host), for `sqso-ffi` on `thumbv7em-none-eabihf`, and for the firmware; `rustfmt --check` | CI (job Static Analysis; job STM32L4 Cross-Compile (library)) | `-Wall -Wextra -Wpedantic` and the further warning flags of `software/flight/CMakeLists.txt` with `-Werror`; cppcheck blocking; clang-tidy and the MISRA addon allow no increase | CI |

**Deviations.** The two `too_many_lines` exceptions under rule 4. Rule 5 is not measured for the Rust. Rule 7 is not
enforced by a tool for C-ABI result codes called from Rust.

**Overall status: not certified compliant.** Rules 3, 4 and 10 are enforced by tools. Rule 9 is enforced by tools for
the crates with `#![forbid(unsafe_code)]`. Rules 1, 2, 6, 7 and 8 rely on review in part or in full, as the table says.

### 2.2 The C code, measured 2026-10-15 (history)

The C sources this table measured were replaced by the Rust (2026-10-24). It is kept as the record of the C status.

> **The "FULLY COMPLIANT" status of revision 1.0 is withdrawn** (FINDINGS_REGISTER SW-07). It was not supported by tool
> output, and two of its claims were false: the `-Wl,--wrap=malloc` link option it cites did not exist in the build, and
> 14 functions exceeded 60 lines. The table below is the measured status. Sections 3 onward are the original text, kept
> for reference and superseded by this table where they disagree.

| Rule | Description | Status | Evidence |
|------|-------------|--------|----------|
| 1 | Simple control flow | Partly verified: no `goto`, `setjmp` or `longjmp` in `src/`; recursion not checked by a tool | grep |
| 2 | Fixed loop bounds | Not verified by a tool | LOOP_BOUNDS.md is a manual review |
| 3 | No dynamic memory | **Met for flight code:** no flight object references `malloc`, `calloc`, `realloc` or `free` (the simulated HAL is excluded) | ctest `no_dynamic_memory` (`software/flight/scripts/check_no_dynamic_memory.sh`) on every build |
| 4 | Function length ≤ 60 lines | **Met:** none of the 681 functions in `src/` is longer (the last nine were split into helpers, 2026-10-16) | `software/flight/scripts/function_length.py`; CI holds the count at 0 (`software/flight/quality/function-length.baseline`) |
| 5 | Two assertions per function | **Not met:** 30 assertion uses across 629 functions | Assertions are active in every build since SW-08 |
| 6 | Smallest scope | Not verified by a tool | — |
| 7 | Check return values | Partly: results are often discarded with `(void)` | cppcheck (blocking) and clang-tidy (no increase) in CI |
| 8 | Limited preprocessor | Not verified by a tool | — |
| 9 | Restrict pointers | Not verified by a tool | — |
| 10 | All warnings, zero warnings | Met for the GCC and Clang `-Werror` builds; cppcheck blocking; clang-tidy and MISRA checker allow no increase | CI workflows |

**Overall status: not compliant.** CI gates now prevent regressions on rules 3, 4 and 10.

---

## 3. Rule-by-Rule Analysis

> **Historical (revision 1.0, 2026-01-02).** Sections 3 to 5 describe the former C flight code, most of which no longer
> exists (`main.c`, `eps_control.c`, `telemetry.c` and the other `.c` modules were replaced by Rust on 2026-10-24).
> Their "COMPLIANT" and "PASS" statuses, the metrics of §4 and the "no deviations" of §5 are not current. The current
> status is §2. Current exceptions: rule 4 (`too_many_lines`, denied at 60 lines) has two documented exceptions, the
> exhaustive `match` of `dispatch` (`sqso-ffi/src/cmd_handler.rs`) and of `transition` (`sqso-fsw/src/mode.rs`)
> (`software/rust/README.md`, Rules).

### 3.1 Rule 1: Simple Control Flow

> **Requirement**: Restrict all code to very simple control flow constructs - do not use goto statements, setjmp or longjmp constructs, and direct or indirect recursion.

**Status**: COMPLIANT

**Evidence**:
- No `goto` statements in codebase
- No `setjmp`/`longjmp` usage
- No recursive function calls
- All control flow uses structured constructs (if/else, switch, for, while)

**Verification Method**: Static analysis with cppcheck, grep-based code search

```bash
# Verification commands
grep -r "goto " src/ include/     # No matches
grep -r "setjmp\|longjmp" src/    # No matches
```

---

### 3.2 Rule 2: Fixed Loop Bounds

> **Requirement**: All loops must have a fixed upper-bound. It must be trivially possible for a checking tool to prove statically that a preset upper-bound on the number of iterations of a loop cannot be exceeded.

**Status**: COMPLIANT

**Evidence**:
- All loops have explicit bounds defined in `smart_qso.h`
- Loop inventory documented in `LOOP_BOUNDS.md`
- Runtime loop counters with fault logging for overflow

**Key Constants**:
```c
#define MAX_SENSORS             32
#define MAX_FAULT_ENTRIES      100
#define MAX_BEACON_LENGTH      256
#define MAX_CMD_DATA_LEN       256
#define MAX_LOOP_ITERATIONS   1000
#define LOG_BUFFER_SIZE         64
#define ASSERT_MAX_FAILURE_LOG  16
#define WDT_MAX_TASKS            8
```

**Loop Pattern**:
```c
for (uint16_t i = 0; i < MAX_SENSORS; i++) {
    /* Bounded iteration */
}
```

See: `software/flight/docs/LOOP_BOUNDS.md` for complete inventory.

---

### 3.3 Rule 3: No Dynamic Memory After Initialization

> **Requirement**: Do not use dynamic memory allocation after initialization.

**Status**: COMPLIANT

**Evidence**:
- All data structures use static allocation
- No calls to `malloc()`, `calloc()`, `realloc()`, `free()`
- Stack-only local variables
- Build system configured to detect violations

**CMakeLists.txt Configuration**:
```cmake
# Flight build catches dynamic allocation
if(FLIGHT_BUILD)
    add_link_options(-Wl,--wrap=malloc -Wl,--wrap=calloc
                     -Wl,--wrap=realloc -Wl,--wrap=free)
endif()
```

**Verification**:
```bash
grep -r "malloc\|calloc\|realloc\|free(" src/ | grep -v "wrap"  # No matches
```

See: `software/flight/docs/MEMORY_POLICY.md` for complete policy.

---

### 3.4 Rule 4: Function Length Limit

> **Requirement**: No function should be longer than what can be printed on a single sheet of paper (approximately 60 lines).

**Status**: COMPLIANT

**Evidence**:
- Original monolithic `main.c` refactored into 20 modules
- Each module has focused, single-purpose functions
- Longest functions are under 60 lines

**Module Structure**:
| Module | Purpose | Max Function Length |
|--------|---------|---------------------|
| main.c | Init, main loop | ~50 lines |
| eps_control.c | Power management | ~45 lines |
| mode.rs (Rust) | Mode transitions | ~40 lines |
| telemetry.c | Frame generation | ~55 lines |
| cmd_handler.rs (Rust) | Command routing | ~50 lines |

**Refactoring Pattern**:
```c
/* Split long function into helpers */
SmartQsoResult_t process_command(const uint8_t *data, size_t len) {
    if (validate_command(data, len) != SMART_QSO_OK) { return error; }
    if (authorize_command(data) != SMART_QSO_OK) { return error; }
    return execute_command(data, len);
}
```

---

### 3.5 Rule 5: Assertions

> **Requirement**: The code's assertion density should be at least 2 assertions per function on average.

**Status**: COMPLIANT

**Evidence**:
- Custom assertion framework: `assert_handler.c/h`
- Assertions are NEVER compiled out in flight builds
- Assertion macros for all common checks

**Assertion Framework**:
```c
/* Precondition check */
SMART_QSO_REQUIRE(condition);

/* Postcondition check */
SMART_QSO_ENSURE(condition);

/* NULL pointer validation */
SMART_QSO_REQUIRE_NOT_NULL(ptr);

/* Range validation */
SMART_QSO_REQUIRE_RANGE(value, min, max);

/* Array bounds check */
SMART_QSO_REQUIRE_BOUNDS(index, array_size);

/* Critical assertion (triggers safe mode) */
SMART_QSO_ASSERT_CRITICAL(condition);
```

**Assertion Statistics**:
- All public functions have precondition checks
- Critical operations have postcondition checks
- Array accesses have bounds checks
- Average assertion density: >2 per function

---

### 3.6 Rule 6: Data Scope

> **Requirement**: Declare all data objects at the smallest possible scope.

**Status**: COMPLIANT

**Evidence**:
- Module-level static variables for persistent state
- Function-local variables declared at point of use
- No unnecessary global visibility

**Pattern**:
```c
/* Module-private static data */
static SystemState_t s_state;

/* Function-local variables at point of use */
SmartQsoResult_t function(void) {
    /* Declared where first used */
    uint32_t value = calculate_something();

    if (condition) {
        /* Scoped to block */
        char buffer[64];
        format_output(buffer, sizeof(buffer));
    }
    return SMART_QSO_OK;
}
```

---

### 3.7 Rule 7: Return Value Checking

> **Requirement**: Check the return value of all non-void functions, and check all function parameters for validity.

**Status**: COMPLIANT

**Evidence**:
- Consistent `SmartQsoResult_t` return type pattern
- All return values explicitly checked or cast to `(void)`
- Parameter validation at function entry

**Return Value Pattern**:
```c
/* All functions return result code */
SmartQsoResult_t result = some_function();
if (result != SMART_QSO_OK) {
    /* Handle error */
    return result;
}

/* Intentionally ignored returns marked with cast */
(void)function_with_optional_result();
```

**Parameter Validation**:
```c
SmartQsoResult_t process_data(const uint8_t *data, size_t len) {
    /* Validate all parameters */
    if (data == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    if (len == 0 || len > MAX_DATA_LEN) {
        return SMART_QSO_ERROR_PARAM;
    }
    /* Proceed with valid parameters */
}
```

---

### 3.8 Rule 8: Limited Preprocessor

> **Requirement**: Limit the use of the preprocessor to header file inclusion and simple conditional compilation macros.

**Status**: COMPLIANT

**Evidence**:
- Preprocessor used only for:
  - Header guards (`#ifndef`/`#define`/`#endif`)
  - Simple compile-time constants (`#define MAX_VALUE 100`)
  - Conditional compilation (`#ifdef SIMULATION_BUILD`)
  - Simple function-like macros with no side effects

**Acceptable Usage**:
```c
/* Header guards */
#ifndef SMART_QSO_MODULE_H
#define SMART_QSO_MODULE_H
...
#endif

/* Compile-time constants */
#define MAX_SENSORS 32

/* Conditional compilation */
#ifdef FLIGHT_BUILD
    #define LOG_MIN_LEVEL LOG_LEVEL_WARNING
#else
    #define LOG_MIN_LEVEL LOG_LEVEL_DEBUG
#endif
```

**Prohibited Usage** (not present in codebase):
- Complex macros with multiple statements
- Macros that modify control flow
- Token pasting (`##`) outside of designated areas

---

### 3.9 Rule 9: Pointer Restrictions

> **Requirement**: Restrict the use of pointers. Use no more than one level of dereferencing per expression.

**Status**: COMPLIANT

**Evidence**:
- Single-level pointer dereferencing
- NULL checks on all pointer parameters
- Bounds validation for array access
- No pointer arithmetic beyond array indexing

**Pointer Validation Pattern**:
```c
SmartQsoResult_t copy_data(uint8_t *dest, size_t dest_size,
                           const uint8_t *src, size_t src_len) {
    /* NULL checks */
    if (dest == NULL || src == NULL) {
        return SMART_QSO_ERROR_NULL_PTR;
    }
    /* Bounds check */
    if (src_len > dest_size) {
        return SMART_QSO_ERROR_PARAM;
    }
    /* Safe copy */
    (void)safe_memcpy(dest, dest_size, src, src_len);
    return SMART_QSO_OK;
}
```

**Safe String Library** (`safe_string.c/h`):
- `safe_strncpy()` - Bounds-checked string copy
- `safe_memcpy()` - Bounds-checked memory copy
- `safe_memset()` - Bounds-checked memory set
- `safe_strlen()` - Maximum length enforcement

---

### 3.10 Rule 10: Compiler Warnings

> **Requirement**: Compile with all warnings enabled and treat warnings as errors.

**Status**: COMPLIANT

**Evidence**:
- CMakeLists.txt configured with maximum warning levels
- `-Werror` enabled to treat warnings as errors
- Zero compiler warnings in build

**CMakeLists.txt Configuration**:
```cmake
# Base warning flags (JPL Rule 10)
add_compile_options(
    -Wall
    -Wextra
    -Wpedantic
    -Wconversion
    -Wshadow
    -Wcast-qual
    -Wstrict-prototypes
    -Wmissing-prototypes
    -Wformat=2
    -Wformat-security
    -Wundef
    -Wdouble-promotion
    -Wfloat-equal
    -Wpointer-arith
    -Wwrite-strings
    -Wswitch-enum
    -Wnull-dereference
)

# Treat warnings as errors
if(ENABLE_WERROR)
    add_compile_options(-Werror)
endif()
```

**Build Verification**:
```bash
cmake --build build 2>&1 | grep -E "warning:|error:"  # No output
```

---

## 4. Verification Evidence

### 4.1 Static Analysis Results

```
Tool: cppcheck
Command: cppcheck --enable=all --std=c11 src/
Result: No errors, no warnings

Tool: compiler warnings
Command: cmake --build build (with -Werror)
Result: Build successful with zero warnings
```

### 4.2 Code Metrics

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Max function length | 55 lines | 60 lines | PASS |
| Assertion density | 2.3/function | 2/function | PASS |
| Cyclomatic complexity (avg) | 4.2 | 10 | PASS |
| Compiler warnings | 0 | 0 | PASS |

### 4.3 Module Compliance Matrix

| Module | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 | R9 | R10 |
|--------|----|----|----|----|----|----|----|----|----|----|
| main.c | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| eps_control.c | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| fault_mgmt.rs (Rust) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| mode.rs (Rust) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| system_state.c | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| cmd_handler.rs (Rust) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| telemetry.c | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| assert_handler.c | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| watchdog_mgr.c | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| flight_log.c | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## 5. Deviations

No deviations from JPL Power of Ten rules are present in the codebase.

---

## 6. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial audit |
| 1.1 | 2026-10-24 | SMART-QSO Team | The flight software is Rust. §1.1 and §2.1 map each rule to the Rust and the remaining C (the flight-log shim), with the enforcement of each (compiler, lint, CI or review). The 2026-10-15 measured C status is kept as §2.2 (history); §3 onward unchanged |
| 1.2 | 2026-10-25 | SMART-QSO Team | §2.1 rule 1: recursion now checked by a tool, the call-graph cycle check of the stack analysis on the flight image; the one bounded nesting (the panic handler) |
| 1.3 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): banner on §3 to §5 (revision 1.0 text of the former C code; statuses not current; the two rule 4 exceptions) |

---

*This document certifies JPL Power of Ten compliance for SMART-QSO flight software following NASA Ames SmallSat heritage patterns.* (Revision 1.0 wording; the claim is withdrawn, see §2.)
