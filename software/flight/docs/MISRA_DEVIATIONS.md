# MISRA C:2012 Deviation Log
## SMART-QSO Flight Software

**Document Number:** SQSO-FSW-010
**Revision:** A
**Date:** 2026-01-02
**Reference:** MISRA C:2012, NPR 7150.2, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document records all deviations from MISRA C:2012 guidelines in the SMART-QSO flight software, including justification, risk assessment, and approval status.

## 2. Scope

This document covers:
- All MISRA C:2012 rule deviations
- Justification for each deviation
- Risk mitigation measures
- Approval chain

---

## 3. MISRA Compliance Summary

> **The compliance figures below (193 of 203 rules compliant, 8 deviations) are withdrawn** (FINDINGS_REGISTER SW-07).
> No MISRA tool had been run. An indicative run of the cppcheck MISRA addon during the audit found about 1,520
> findings, 1,187 of them not covered by any deviation below. CI now runs that checker on every change
> (`.github/workflows/ci.yml`, job "MISRA C:2012 Check"). It allows no increase over `software/flight/quality/misra.baseline`
> and archives the report; the baseline, set from CI with cppcheck 2.13.0, is 1,189 findings (1,458 before the 2026-10-16 cleanup). The cppcheck addon is
> not a qualified MISRA tool. The deviation records in §4 are proposals,
> to be confirmed against a qualified tool's output.

### 3.1 Compliance Level (withdrawn; kept for reference)

| Category | Total Rules | Compliant | Deviations | N/A |
|----------|-------------|-----------|------------|-----|
| Mandatory | 16 | 16 | 0 | 0 |
| Required | 143 | 138 | 5 | 0 |
| Advisory | 44 | 39 | 3 | 2 |
| **Total** | **203** | **193** | **8** | **2** |

### 3.2 Deviation Categories

| Category | Count | Description |
|----------|-------|-------------|
| Hardware Access | 3 | Register access patterns |
| Performance | 2 | Optimization requirements |
| Legacy Interface | 2 | External library compatibility |
| Documentation | 1 | Comment format |

---

## 4. Deviation Records

### 4.1 DEV-001: Rule 11.3 - Cast Between Pointer and Integer

| Field | Value |
|-------|-------|
| Rule | 11.3 (Required) |
| Category | Hardware Access |
| Location | hal_stm32l4.c:45-52 |
| Approved By | Systems Engineer |
| Date | 2026-01-02 |

**Rule Statement:** A cast shall not be performed between a pointer to object type and a pointer to a different object type.

**Deviation:** Cast from `uint32_t` to hardware register pointer.

**Code Example:**
```c
/* DEV-001: Hardware register access requires address cast */
#define GPIOA_BASE  ((GPIO_TypeDef *)0x48000000UL)
```

**Justification:** Hardware register access on Cortex-M4 requires casting fixed addresses to register structure pointers. This is the standard CMSIS pattern used by all STM32 HAL libraries.

**Risk Assessment:** Low. Pattern is well-established and verified across millions of STM32 deployments.

**Mitigation:**
1. All register addresses verified against STM32L4 reference manual
2. CMSIS-provided definitions used where available
3. Addresses defined as macros for single point of maintenance

---

### 4.2 DEV-002: Rule 11.4 - Cast Between Pointer and Integer

| Field | Value |
|-------|-------|
| Rule | 11.4 (Advisory) |
| Category | Hardware Access |
| Location | hal_flash.c:78-85 |
| Approved By | Software Lead |
| Date | 2026-01-02 |

**Rule Statement:** A conversion should not be performed between a pointer to object and an integer type.

**Deviation:** Flash memory address calculation from integer base addresses.

**Code Example:**
```c
/* DEV-002: Flash sector address calculation */
uint32_t sector_addr = FLASH_BASE + (sector * FLASH_SECTOR_SIZE);
uint8_t *data_ptr = (uint8_t *)sector_addr;
```

**Justification:** Flash operations require calculating absolute addresses from sector numbers. The flash memory map is fixed by hardware.

**Risk Assessment:** Low. Addresses computed from verified constants.

**Mitigation:**
1. FLASH_BASE verified against linker script
2. Sector size validated at compile time
3. Bounds checking before all flash operations

---

### 4.3 DEV-003: Rule 11.5 - Cast from Void Pointer

| Field | Value |
|-------|-------|
| Rule | 11.5 (Advisory) |
| Category | Legacy Interface |
| Location | Multiple files |
| Approved By | Software Lead |
| Date | 2026-01-02 |

**Rule Statement:** A conversion should not be performed from pointer to void into pointer to object.

**Deviation:** FreeRTOS API uses `void *` for task parameters and queue items.

**Code Example:**
```c
/* DEV-003: FreeRTOS task parameter casting */
void sensor_task(void *pvParameters)
{
    SensorContext_t *ctx = (SensorContext_t *)pvParameters;
    /* ... */
}
```

**Justification:** FreeRTOS API design requires void pointer parameters for generic task creation. This is inherent to the RTOS interface.

**Risk Assessment:** Medium. Type safety relies on correct usage at call sites.

**Mitigation:**
1. Task creation wrapper functions enforce type safety
2. Static analysis verifies parameter types at call sites
3. Defensive NULL checks at cast points
4. Code review checklist includes parameter type verification

---

### 4.4 DEV-004: Rule 14.3 - Controlling Expression Invariant

| Field | Value |
|-------|-------|
| Rule | 14.3 (Required) |
| Category | Documentation |
| Location | main.c:main() |
| Approved By | Systems Engineer |
| Date | 2026-01-02 |

**Rule Statement:** Controlling expressions shall not be invariant.

**Deviation:** Main loop uses `while(1)` infinite loop.

**Code Example:**
```c
/* DEV-004: Intentional infinite main loop */
int main(void)
{
    system_init();

    while (1) {  /* Intentional - embedded system never exits */
        main_loop_iteration();
        hal_watchdog_kick();
    }

    return 0;  /* Never reached */
}
```

**Justification:** Embedded systems run continuously and never exit main(). This is the standard pattern for all embedded firmware.

**Risk Assessment:** None. This is expected behavior.

**Mitigation:**
1. Comment documents intentional infinite loop
2. Watchdog ensures recovery from hangs
3. Safe mode provides fallback behavior

---

### 4.5 DEV-005: Rule 17.7 - Ignored Return Value

| Field | Value |
|-------|-------|
| Rule | 17.7 (Required) |
| Category | Performance |
| Location | beacon.c:125, uart_comm.c:89 |
| Approved By | Software Lead |
| Date | 2026-01-02 |

**Rule Statement:** The value returned by a function having non-void return type shall be used.

**Deviation:** Return values ignored for non-critical logging operations.

**Code Example:**
```c
/* DEV-005: Log failures are non-critical */
(void)log_event(EVENT_BEACON_SENT, timestamp);  /* Cast to void documents intent */
```

**Justification:** Telemetry logging is best-effort. Log buffer full or flash write failures should not impact critical operations.

**Risk Assessment:** Low. Logging is explicitly non-critical.

**Mitigation:**
1. `(void)` cast documents intentional discard
2. Log buffer overflow tracked via separate counter
3. Critical operations use mandatory error handling

---

### 4.6 DEV-006: Rule 20.4 - Macro Definition

| Field | Value |
|-------|-------|
| Rule | 20.4 (Required) |
| Category | Legacy Interface |
| Location | smart_qso_config.h |
| Approved By | Software Lead |
| Date | 2026-01-02 |

**Rule Statement:** A macro shall not be defined with the same name as a keyword.

**Deviation:** N/A - No violations in codebase. Reserved for future use.

---

### 4.7 DEV-007: Rule 21.6 - Standard Library Input/Output

| Field | Value |
|-------|-------|
| Rule | 21.6 (Required) |
| Category | Performance |
| Location | Simulation builds only |
| Approved By | Software Lead |
| Date | 2026-01-02 |

**Rule Statement:** The Standard Library input/output functions shall not be used.

**Deviation:** `printf` used in simulation/debug builds only.

**Code Example:**
```c
#if DEBUG_ENABLED
/* DEV-007: Debug output in simulation only */
#define DEBUG_PRINT(fmt, ...) printf(fmt, ##__VA_ARGS__)
#else
#define DEBUG_PRINT(fmt, ...) ((void)0)
#endif
```

**Justification:** Debug output required for development. Completely compiled out in flight builds.

**Risk Assessment:** None for flight. Debug build only.

**Mitigation:**
1. Conditional compilation ensures exclusion from flight
2. `FLIGHT_BUILD` define verified in build system
3. Binary size check confirms no stdio in flight image

---

### 4.8 DEV-008: Rule 8.13 - Const Pointer Parameter

| Field | Value |
|-------|-------|
| Rule | 8.13 (Advisory) |
| Category | Legacy Interface |
| Location | hal_i2c.c, hal_spi.c |
| Approved By | Software Lead |
| Date | 2026-01-02 |

**Rule Statement:** A pointer should point to a const-qualified type whenever possible.

**Deviation:** HAL function signatures match vendor HAL for compatibility.

**Code Example:**
```c
/* DEV-008: Match STM32 HAL signature */
HAL_StatusTypeDef HAL_I2C_Master_Transmit(I2C_HandleTypeDef *hi2c,
                                          uint16_t DevAddress,
                                          uint8_t *pData,  /* Could be const */
                                          uint16_t Size,
                                          uint32_t Timeout);
```

**Justification:** Maintaining compatibility with STM32 HAL API for drop-in replacement and vendor support.

**Risk Assessment:** Low. Data not modified despite non-const signature.

**Mitigation:**
1. Wrapper functions add const where appropriate
2. Static analysis verifies no modification
3. Documented in ICD

---

### 4.9 DEV-009: Target startup and C-library interface (rules 8.7, 8.11, 11.4, 11.6, 17.3)

| Field | Value |
|-------|-------|
| Rules | 8.7, 8.11, 11.4, 11.6 (Advisory); 17.3 (Mandatory, reported by the addon for inline assembly) |
| Category | Hardware access and C-library interface |
| Location | `src/hal/stm32l4_startup.c`, `src/hal/stm32l4_syscalls.c`, `src/hal/stm32l4_clock.c` |
| Approved By | Proposed, for the PI to ratify |
| Date | 2026-10-16 |

**Deviation and justification:**
- **8.7** (external linkage used in one file): `Reset_Handler` is the image entry point named by the linker script, and `_write`, `_sbrk`, `_open` and the other system calls are the names newlib calls. They must be external.
- **8.11** (extern array without a size): the linker-script symbols `_sdata`, `_edata`, `_sbss`, `_ebss`, `_sidata` and `_estack` have no size in C; only their addresses are used, and the section sizes are computed from them as integers.
- **11.4** (pointer/integer conversion): memory-mapped registers at fixed addresses from ST's `stm32l4r5xx.h` (RCC, FLASH, SCB), and the linker-symbol address arithmetic. Same pattern as `hal_stm32l4.c`.
- **11.6** (`(void *)-1`): `_sbrk` must return that value to report "no memory"; it is how the image has no heap.
- **17.3**: the addon reports `__asm volatile ("dsb; isb")` as an undeclared function. The barrier is required after enabling the FPU (Arm Cortex-M4 FPU enable sequence) and is the only assembly in the image.

**Risk assessment:** Low. Register addresses and bit positions are cited in each file; the linker script fails the link if sections overflow; CI checks the entry point, the vector table address and the reset vector.

---

## 5. Deviation Approval Process

### 5.1 Required Information

Each deviation SHALL include:
- Rule number and category
- Exact code location (file:line)
- Justification for deviation
- Risk assessment (High/Medium/Low)
- Mitigation measures
- Approver and date

### 5.2 Approval Authority

| Deviation Risk | Approval Authority |
|----------------|-------------------|
| Low | Software Lead |
| Medium | Systems Engineer |
| High | Project Manager + Safety |

### 5.3 Review Schedule

- All deviations reviewed at each release
- High-risk deviations reviewed monthly
- New deviations require CDR-level review

---

## 6. Analysis Tools

### 6.1 Static Analysis Configuration

| Tool | Configuration |
|------|---------------|
| cppcheck | `--enable=all --std=c11 --suppress=misra-c2012-*` |
| PC-lint | `au-misra3.lnt` |
| Polyspace | MISRA C:2012 checker enabled |
| clang-tidy | `cert-*,bugprone-*,misc-*` |

### 6.2 Suppression Format

```c
/* cppcheck-suppress misra-c2012-11.3 ; DEV-001: Hardware register access */
#define GPIOA_BASE  ((GPIO_TypeDef *)0x48000000UL)
```

---

## 7. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
| B | 2026-10-16 | DEV-009 proposed for the STM32L4 startup, system calls and clock (SW-01) | Funding-audit follow-up |
