# SMART-QSO Flight Software API Documentation

**Document ID**: SMART-QSO-API-DOC-001
**Version**: 1.0
**Classification**: Engineering

## Overview

This is the API documentation for the SMART-QSO 1U CubeSat flight software. The software is designed following NASA NPR 7150.2 software engineering requirements and JPL Power of Ten coding rules for safety-critical embedded systems.

## Architecture

The flight software follows a modular architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                        main.c                               │
│                   (System Orchestration)                    │
└─────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌───────────────┐
│ state_machine │   │   system_state  │   │  fault_mgmt   │
│   (Modes)     │   │ (Global State)  │   │   (FDIR)      │
└───────────────┘   └─────────────────┘   └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌───────────────┐
│  eps_control  │   │   adcs_control  │   │   cmd_handler │
│   (Power)     │   │   (Attitude)    │   │  (Commands)   │
└───────────────┘   └─────────────────┘   └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │    telemetry    │
                    │   (Downlink)    │
                    └─────────────────┘
```

## Module Categories

### Core Infrastructure
- **smart_qso.h** - Common types, error codes, and system constants
- **system_state** - Global system state management
- **state_machine** - Operational mode control (BOOT→SAFE→DETUMBLE→IDLE→ACTIVE→EMERGENCY)
- **fault_mgmt** - Fault Detection, Isolation, and Recovery (FDIR)

### Hardware Abstraction
- **eps_control** - Electrical Power System management
- **adcs_control** - Attitude Determination and Control System
- **thermal_ctrl** - Thermal management
- **watchdog_mgr** - Hardware and task watchdog management

### Communication
- **cmd_handler** - Uplink command processing and routing
- **telemetry** - Downlink telemetry frame generation
- **beacon** - Beacon message composition

### Safety & Reliability
- **assert_handler** - JPL Rule 5 compliant assertions
- **input_validation** - Parameter validation framework
- **safe_string** - Bounds-checked string operations
- **crc** - CRC32 data integrity checking

### Logging & Diagnostics
- **flight_log** - MISRA-compliant logging subsystem

### AI/ML Integration
- **ai_manager** - AI payload coordination
- **power_prediction** - ML-based power forecasting
- **qso_priority** - QSO prioritization model

## Coding Standards

This software adheres to:

1. **JPL Power of Ten Rules** - See JPL_POWER_OF_TEN_AUDIT.md
2. **MISRA C:2012** - See MISRA_DEVIATIONS.md
3. **NASA NPR 7150.2** - Software Engineering Requirements

### Key Patterns

#### Error Handling
All functions return `SmartQsoResult_t`:
```c
SmartQsoResult_t result = some_function();
if (result != SMART_QSO_OK) {
    // Handle error
}
```

#### Assertions (JPL Rule 5)
```c
SMART_QSO_REQUIRE(condition);           // Precondition
SMART_QSO_ENSURE(condition);            // Postcondition
SMART_QSO_REQUIRE_NOT_NULL(ptr);        // NULL check
SMART_QSO_REQUIRE_BOUNDS(idx, size);    // Array bounds
```

#### Safe String Operations
```c
safe_strncpy(dest, dest_size, src, src_len, &chars_copied);
safe_memcpy(dest, dest_size, src, src_len);
```

## Build Configuration

```bash
# Debug build with coverage
cmake -B build -DCMAKE_BUILD_TYPE=Debug -DENABLE_COVERAGE=ON

# Release build with warnings as errors
cmake -B build -DCMAKE_BUILD_TYPE=Release -DENABLE_WERROR=ON

# Flight build (no debug, malloc wrapped)
cmake -B build -DCMAKE_BUILD_TYPE=Release -DFLIGHT_BUILD=ON
```

## Documentation Files

| Document | Purpose |
|----------|---------|
| FDIR_DESIGN.md | Fault Detection, Isolation, Recovery design |
| MEMORY_POLICY.md | Static allocation policy |
| LOOP_BOUNDS.md | Loop iteration limits |
| MISRA_DEVIATIONS.md | MISRA C:2012 deviation record |
| JPL_POWER_OF_TEN_AUDIT.md | JPL coding rules compliance |

## Safety Classification

This software is classified as **Class C** per NPR 7150.2:
- Mission-critical but not life-critical
- Loss would significantly impact mission success
- Requires formal verification and validation

## Contact

SMART-QSO Team
https://github.com/smart-qso

---

*Generated with Doxygen - Following NASA Ames SmallSat heritage patterns*
