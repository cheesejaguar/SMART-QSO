# Build Configurations
## SMART-QSO Flight Software

**Document Number:** SQSO-SW-007
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7150.2, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines all build configurations for the SMART-QSO flight software, including compiler options, preprocessor defines, and target-specific settings.

## 2. Scope

This document covers:
- CMake build options
- Target platform configurations
- Debug vs. flight settings
- Test and coverage configurations

---

## 3. Build Configuration Matrix

### 3.1 Configuration Overview

| Configuration | Target | Optimization | Debug | Watchdog | Tests |
|---------------|--------|--------------|-------|----------|-------|
| Development | Linux | -O0 | Yes | No | Yes |
| Simulation | Linux | -O2 | Yes | No | Yes |
| Flatsat | STM32L4 | -O2 | Yes | Optional | Yes |
| Flight | STM32L4 | -Os | No | Yes | No |

### 3.2 CMake Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| FLIGHT_BUILD | BOOL | OFF | Enable flight configuration |
| ENABLE_COVERAGE | BOOL | OFF | Enable code coverage instrumentation |
| ENABLE_SANITIZERS | BOOL | OFF | Enable AddressSanitizer/UBSan |
| TARGET_PLATFORM | STRING | LINUX | Target: LINUX, STM32L4, FREERTOS |
| ENABLE_TESTS | BOOL | ON | Build test suite |
| BUILD_DOCS | BOOL | OFF | Build Doxygen documentation |

---

## 4. Development Configuration

### 4.1 Purpose

Development builds are used for:
- Initial software development
- Unit testing on host system
- Code coverage analysis
- Static analysis integration

### 4.2 CMake Invocation

```bash
cmake -S software/flight -B build \
    -DCMAKE_BUILD_TYPE=Debug \
    -DENABLE_COVERAGE=ON \
    -DENABLE_SANITIZERS=ON \
    -DTARGET_PLATFORM=LINUX
```

### 4.3 Compiler Flags

```cmake
# Debug flags
-g3                    # Maximum debug info
-O0                    # No optimization (for debugging)
-Wall -Wextra          # All warnings
-Werror                # Warnings as errors
-fsanitize=address     # AddressSanitizer
-fsanitize=undefined   # UndefinedBehaviorSanitizer
--coverage             # Code coverage
```

### 4.4 Preprocessor Defines

| Define | Value | Purpose |
|--------|-------|---------|
| DEBUG_ENABLED | 1 | Enable debug output |
| SIMULATION_MODE | 1 | Use HAL simulation |
| LOG_LEVEL | LOG_DEBUG | Verbose logging |
| WATCHDOG_ENABLED | 0 | Disable watchdog |
| ASSERT_ENABLED | 1 | Enable assertions |

---

## 5. Simulation Configuration

### 5.1 Purpose

Simulation builds are used for:
- Integration testing with simulated hardware
- Performance testing
- Day-in-the-life testing
- Algorithm validation

### 5.2 CMake Invocation

```bash
cmake -S software/flight -B build \
    -DCMAKE_BUILD_TYPE=RelWithDebInfo \
    -DTARGET_PLATFORM=LINUX
```

### 5.3 Compiler Flags

```cmake
# Simulation flags
-g                     # Debug info
-O2                    # Optimization
-Wall -Wextra          # All warnings
-Werror                # Warnings as errors
-DNDEBUG               # Disable standard assertions
```

### 5.4 Preprocessor Defines

| Define | Value | Purpose |
|--------|-------|---------|
| DEBUG_ENABLED | 1 | Debug output enabled |
| SIMULATION_MODE | 1 | Use HAL simulation |
| LOG_LEVEL | LOG_INFO | Normal logging |
| WATCHDOG_ENABLED | 0 | Watchdog disabled |
| ASSERT_ENABLED | 1 | Assertions enabled |

---

## 6. Flatsat Configuration

### 6.1 Purpose

Flatsat builds are used for:
- Hardware-in-the-loop testing
- Interface validation
- Integration testing with real hardware
- Pre-flight software checkout

### 6.2 CMake Invocation

```bash
cmake -S software/flight -B build \
    -DCMAKE_BUILD_TYPE=RelWithDebInfo \
    -DTARGET_PLATFORM=STM32L4 \
    -DFLATSAT_BUILD=ON
```

### 6.3 Compiler Flags

```cmake
# Flatsat flags
-mcpu=cortex-m4        # Target CPU
-mthumb                # Thumb instruction set
-mfloat-abi=hard       # Hardware FPU
-mfpu=fpv4-sp-d16      # FPU type
-g                     # Debug info
-O2                    # Optimization
-Wall -Wextra          # All warnings
-Werror                # Warnings as errors
```

### 6.4 Preprocessor Defines

| Define | Value | Purpose |
|--------|-------|---------|
| DEBUG_ENABLED | 1 | Debug output |
| SIMULATION_MODE | 0 | Real hardware |
| LOG_LEVEL | LOG_DEBUG | Verbose logging |
| WATCHDOG_ENABLED | 0 | Watchdog optional |
| DEPLOYMENT_ENABLED | 0 | Disable deployment |
| TX_POWER_LEVEL | 0 | Minimum RF power |

---

## 7. Flight Configuration

### 7.1 Purpose

Flight builds are the certified release configuration for on-orbit operation.

### 7.2 CMake Invocation

```bash
cmake -S software/flight -B build \
    -DCMAKE_BUILD_TYPE=Release \
    -DFLIGHT_BUILD=ON \
    -DTARGET_PLATFORM=STM32L4
```

### 7.3 Compiler Flags

```cmake
# Flight flags
-mcpu=cortex-m4        # Target CPU
-mthumb                # Thumb instruction set
-mfloat-abi=hard       # Hardware FPU
-mfpu=fpv4-sp-d16      # FPU type
-Os                    # Optimize for size
-Wall -Wextra          # All warnings
-Werror                # Warnings as errors
-ffunction-sections    # Function sections
-fdata-sections        # Data sections
-flto                  # Link-time optimization
```

### 7.4 Linker Flags

```cmake
-Wl,--gc-sections      # Remove unused sections
-Wl,-Map,output.map    # Generate map file
-specs=nano.specs      # Use newlib-nano
-specs=nosys.specs     # No syscalls
```

### 7.5 Preprocessor Defines

| Define | Value | Purpose |
|--------|-------|---------|
| DEBUG_ENABLED | 0 | No debug output |
| SIMULATION_MODE | 0 | Real hardware |
| LOG_LEVEL | LOG_INFO | Normal logging |
| WATCHDOG_ENABLED | 1 | Watchdog enabled |
| DEPLOYMENT_ENABLED | 1 | Enable deployment |
| ASSERT_ENABLED | 0 | Assertions as fault |

---

## 8. Platform-Specific Configuration

### 8.1 Linux Platform

| Parameter | Value |
|-----------|-------|
| Compiler | GCC or Clang |
| C Standard | C11 |
| HAL | hal_sim.c (simulation) |
| RTOS | None (POSIX threads) |

### 8.2 STM32L4 Platform

| Parameter | Value |
|-----------|-------|
| Processor | STM32L4R5ZI |
| Compiler | arm-none-eabi-gcc |
| C Standard | C11 |
| HAL | hal_stm32l4.c |
| RTOS | FreeRTOS 10.5.1 |
| Flash | 2 MB |
| RAM | 640 KB |

### 8.3 FreeRTOS Configuration

| Parameter | Value |
|-----------|-------|
| configUSE_PREEMPTION | 1 |
| configUSE_IDLE_HOOK | 1 |
| configUSE_TICK_HOOK | 0 |
| configCPU_CLOCK_HZ | 80000000 |
| configTICK_RATE_HZ | 1000 |
| configMAX_PRIORITIES | 5 |
| configMINIMAL_STACK_SIZE | 128 |
| configTOTAL_HEAP_SIZE | 32768 |

---

## 9. Memory Layout

### 9.1 STM32L4 Memory Map

| Region | Start | End | Size | Content |
|--------|-------|-----|------|---------|
| Flash | 0x08000000 | 0x081FFFFF | 2 MB | Code + const data |
| SRAM1 | 0x20000000 | 0x2004FFFF | 320 KB | Data + heap |
| SRAM2 | 0x20050000 | 0x2007FFFF | 192 KB | Stack + critical |
| SRAM3 | 0x20080000 | 0x2009FFFF | 128 KB | Reserved |

### 9.2 Flash Layout

| Sector | Start | Size | Content |
|--------|-------|------|---------|
| 0 | 0x08000000 | 16 KB | Bootloader |
| 1-7 | 0x08004000 | 112 KB | Application A |
| 8-14 | 0x08020000 | 112 KB | Application B |
| 15-31 | 0x0803C000 | 272 KB | Parameters + logs |

---

## 10. Build Verification

### 10.1 Build Artifacts

| Artifact | Description | Required |
|----------|-------------|----------|
| smart_qso.elf | Executable with debug symbols | Yes |
| smart_qso.hex | Intel HEX for programming | Flight |
| smart_qso.bin | Binary image | Flight |
| smart_qso.map | Linker map file | Yes |
| size_report.txt | Section size report | Yes |

### 10.2 Size Limits

| Section | Maximum | Purpose |
|---------|---------|---------|
| .text | 200 KB | Code |
| .rodata | 50 KB | Constants |
| .data | 20 KB | Initialized data |
| .bss | 30 KB | Uninitialized data |
| Stack | 8 KB | Per task |
| Heap | 32 KB | Dynamic allocation |

### 10.3 Build Verification Checklist

- [ ] No compiler warnings
- [ ] No linker warnings
- [ ] Size within limits
- [ ] All symbols resolved
- [ ] Stack analysis passed
- [ ] Static analysis passed

---

## 11. Configuration Management

### 11.1 Version Tagging

```bash
# Tag format: vX.Y.Z-config
# Examples:
git tag v1.0.0-flight
git tag v1.0.0-flatsat
git tag v1.0.0-test
```

### 11.2 Configuration Traceability

Each flight release includes:
- Git commit hash
- Build configuration used
- Compiler version
- Build timestamp
- SVD document reference

---

## 12. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
