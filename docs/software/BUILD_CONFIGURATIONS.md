# Build Configurations
## SMART-QSO Flight Software

**Document Number:** SQSO-SW-008
**Revision:** E
**Date:** 2026-10-27
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

### 2.1 Build Configurations as Built (Revision C, 2026-10-24)

The flight software is Rust (`software/rust/README.md`). Sections 3 to 11 are the Revision A plan. Where they disagree
with this section, this section describes the build. Revision A's `TARGET_PLATFORM`, `ENABLE_TESTS` and `BUILD_DOCS`
options, the FreeRTOS configuration and the C cross build for the STM32L4 do not exist. The C cross build (toolchain
file, linker script, `hal_stm32l4.c`, startup, clock and system calls) was removed on 2026-10-24.

| Configuration | Command | Produces |
|---------------|---------|----------|
| Host build and tests | `cmake -S software/flight -B build && cmake --build build && ctest --test-dir build` | The Rust library `libsqso_flight.a` (crate `sqso-clib`), the C test suites linked against it, and the host simulation `build/smart_qso_flight` (crate `sqso-sim`, 1000 passes) |
| Coverage | add `-DENABLE_COVERAGE=ON` | gcov on the C; the Rust library built with `-C instrument-coverage` (`software/flight/cmake/rust.cmake`) |
| Sanitizers | add `-DENABLE_SANITIZERS=ON` | AddressSanitizer and UBSan on the C suites and the shim (the Rust library is not sanitized; its parsers are fuzzed under AddressSanitizer instead) |
| Flight configuration (host) | add `-DFLIGHT_BUILD=ON` | `FLIGHT_BUILD` and `NDEBUG` defined for the C; the simulation built with the cargo feature `flight` (runs until stopped) |
| Flight image (STM32L4R5ZI) | `cd software/rust/firmware && cargo build --release` | `target/thumbv7em-none-eabihf/release/sqso-firmware` (ELF) |
| Flight software library for the target | `cd software/rust && cargo build --release -p sqso-ffi --target thumbv7em-none-eabihf` | CI checks that the library builds for the Cortex-M4F |

CMake options (`software/flight/CMakeLists.txt`): `FLIGHT_BUILD` (OFF), `ENABLE_COVERAGE` (OFF), `ENABLE_SANITIZERS`
(OFF), `ENABLE_WERROR` (ON). A `docs` target exists when Doxygen is found. CMake always builds the Rust with cargo's
release profile, whatever `CMAKE_BUILD_TYPE` is.

Rust release profile (`software/rust/Cargo.toml`; the firmware's `software/rust/firmware/Cargo.toml` is the same, plus
`debug = 2`): `opt-level = "s"`, `lto = true`, `codegen-units = 1`, `overflow-checks = true`, `panic = "abort"`,
`debug-assertions = false`. The toolchain is pinned by `software/rust/rust-toolchain.toml` (1.97.0, target
`thumbv7em-none-eabihf`).

Image (release, 2026-10-27, flight software round 3, third part): text 207,260 B, data 26,400 B, bss 17,052 B. Vector table at
0x08000000, initial SP 0x20040000, entry `Reset`. Linked with cortex-m-rt's `link.x` and the firmware's own `memory.x`
(2 MB flash; RAM SRAM2 alone, 64 KB with parity, holding the statics and the 8 KB stack). CI checks the entry point, the vector table, the initial SP, the presence of the flight symbols, and the absence
of `malloc`, `free`, `_sbrk` and `printf` (job STM32L4 Cross-Compile (library)).

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

Revision C: the flight image is built by cargo, not by CMake (§2.1). The C flags, linker flags and defines below were
for the removed C cross build.

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
| RTOS | FreeRTOS 10.5.1 *(superseded 2026-10-25: no RTOS, Embassy executor; BASELINE §6, CR-2026-002)* |
| Flash | 2 MB |
| RAM | 640 KB |

Revision C, as built: compiler rustc 1.97.0 for `thumbv7em-none-eabihf`; language Rust; HAL `software/rust/firmware/src`
(Embassy); RTOS none (Embassy executor, one task; `software/flight/docs/RTOS_ARCHITECTURE.md`). The Linux platform's HAL
in §8.1 is `software/rust/sqso-ffi/src/hal_sim.rs`.

### 8.3 FreeRTOS Configuration

Revision C: not applicable. FreeRTOS was never integrated. Since 2026-10-25 the baseline is the Embassy executor with no
RTOS (BASELINE §6, CR-2026-002).

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

Revision C: the linker does not split SRAM or flash this way. RAM is one 640 KB region from 0x20000000, flash starts
with the vector table at 0x08000000, and there is no bootloader or A/B image
(`software/flight/docs/MEMORY_POLICY.md` §3, `software/flight/docs/BOOTLOADER_REQUIREMENTS.md` §1.1).

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

Revision C: cargo produces the ELF `sqso-firmware`. CI prints its section sizes (`arm-none-eabi-size -A`). No HEX or
BIN file and no map file are produced yet.

### 10.2 Size Limits

| Section | Maximum | Purpose |
|---------|---------|---------|
| .text | 200 KB | Code |
| .rodata | 50 KB | Constants |
| .data | 20 KB | Initialized data |
| .bss | 30 KB | Uninitialized data |
| Stack | 8 KB | Per task |
| Heap | 32 KB | Dynamic allocation |

Revision C: the limits are not checked by CI. The image measures text 115,876 B (code and constants together) and data
19,764 B, both within the limits above. Its bss of 16,496 B is within the 30 KB limit. It has no heap, and its one stack
has a worst case of 5,444 B of its 8 KB budget by static analysis, a CI gate (`software/flight/docs/MEMORY_POLICY.md` §6).

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
| B | 2026-10-16 | Document number changed from SQSO-SW-007, which `software/flight/docs/LOOP_BOUNDS.md` also carries, to SQSO-SW-008 (finding PI-17). No content change | SMART-QSO Team |
| C | 2026-10-24 | The flight software is Rust: §2.1 the build configurations as built (CMake options, cargo builds, Rust release profile, image and its CI checks); notes on §7 to §10 where the Revision A plan (C cross build, FreeRTOS, memory split, artifacts) no longer applies | SMART-QSO Team |
| D | 2026-10-27 | Stack worst case 5,444 B (was 5,004 B) after the CDR flight-software changes | SMART-QSO Team (CDR avionics review) |
| E | 2026-10-27 | Flight software round 3: the image of the round-3 build (text 190,532 B), the firmware's own `memory.x` with RAM in SRAM2 alone, initial SP 0x20040000 (RID AV-13) | SMART-QSO Team |
| C, note | 2026-10-25 | §8.2 and §8.3: the Embassy executor with no RTOS is the baseline (BASELINE §6, CR-2026-002) | SMART-QSO Team |
