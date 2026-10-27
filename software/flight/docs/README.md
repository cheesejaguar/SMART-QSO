# SMART-QSO Flight Software API Documentation

**Document ID**: SMART-QSO-API-DOC-001
**Version**: 1.2
**Date**: 2026-10-27
**Classification**: Engineering

## Overview

This is the API documentation for the SMART-QSO 1U CubeSat flight software. The software is designed following NASA NPR 7150.2 software engineering requirements and JPL Power of Ten coding rules for safety-critical embedded systems.

The flight software is Rust (`software/rust`; overview, crates and porting table in `software/rust/README.md`). The API
documented here is the C ABI: the headers in `software/flight/include`, which the Rust modules implement and the C test
suites call. Doxygen generates this documentation from those headers and `software/flight/src` (the flight-log shim,
the only C left in the flight code). For the Rust items, `cargo doc` in `software/rust` gives the documentation.

## Architecture

The flight software follows a modular architecture with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│     executive (software/rust/sqso-ffi/src/executive.rs)     │
│      bring-up, main loop of stages, watchdog check-ins      │
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
│  eps_control  │   │   adcs_control  │   │ uplink,       │
│ payload_guard │   │                 │   │ cmd_handler   │
└───────────────┘   └─────────────────┘   └───────────────┘
        │                     │                     │
        └─────────────────────┼─────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ telemetry,      │
                    │ beacon          │
                    └─────────────────┘
                              │
                              ▼
     HAL (hal/*.h): sqso-ffi/src/hal_sim.rs on the host, software/rust/firmware on the target
```

## Module Categories

Each module is a header in `software/flight/include`, implemented by `software/rust/sqso-ffi/src/<module>.rs`, with its
logic in `software/rust/sqso-fsw` or `software/rust/sqso-protocol`.

### Core Infrastructure
- **smart_qso.h** - Common types, error codes, system constants, the mission clock
- **system_state** - Global system state management
- **state_machine** - Operational mode control (BOOT→SAFE→DETUMBLE→IDLE→ACTIVE→EMERGENCY)
- **fault_mgmt** - Fault Detection, Isolation, and Recovery (FDIR)
- **scheduler** - Time-triggered task table (tested; not used by the executive)

### Power, Attitude and Payload
- **eps_control** - Electrical Power System management
- **payload_guard** - OBC-enforced Jetson power and temperature limits
- **adcs_control** - Attitude Determination and Control System
- **sensors**, **gnss** - Sensor table and GNSS receiver
- **deployment**, **separation_switch** - Deployment sequence
- **watchdog_mgr** - Hardware and task watchdog management
- **hk_store** - Stored housekeeping in FRAM and its dump (`DUMP_TLM`)
- **ops_state** - Operations state across resets (the commissioning hold)
- **passivation**, **eps_lockout** - End-of-life passivation by telecommand; clearing the EPS's latched recharge lockout
- **fram_health** - FRAM health and the degraded mode without it

### Communication
- **uplink**, **ax25**, **input_validation**, **cmd_auth**, **cmd_dictionary**, **cmd_handler** - Uplink command reception, authentication and dispatch
- **rf_control**, **rf_silence**, **contact_monitor** - Transmitter permission and inhibits
- **receiver** - Supervision of the 435 MHz command receiver
- **station_id** - Spacecraft station identity (KK6GDB-11)
- **downlink** - The downlink transmitter: the gate before the KISS TNC, key-down and duty limits, power select, VSWR and thermal protection
- **telemetry** - Downlink telemetry frame generation
- **beacon**, **beacon_filter** - Beacon message composition and content filter
- **jetson_link**, **uart_comm** - OBC–Jetson line protocol and serial line
- **gse_provision** - Key provisioning over the GSE umbilical (integration only)

### Safety & Reliability
- **assert_handler** - JPL Rule 5 compliant assertions
- **safe_string** - Bounds-checked string operations
- **nvm_record**, **mission_data** - Power-fail-safe records and mission statistics
- **crc** - CRC32 data integrity checking (`smart_qso.h`), **sha256** - SHA-256 and HMAC-SHA-256

### Logging & Diagnostics
- **flight_log** - Logging subsystem (`log_write()` and `log_vwrite()` are the C shim `software/flight/src/flight_log_fmt.c`)

### AI/ML Integration
Revision 1.0 listed `thermal_ctrl`, `ai_manager`, `power_prediction` and `qso_priority`. No such modules exist in the
flight software. The AI payload software is in `software/payload`.

## Coding Standards

This software adheres to:

1. **Rust coding standard** - See `docs/development/CODING_STANDARDS.md` §11
2. **JPL Power of Ten Rules** - See JPL_POWER_OF_TEN_AUDIT.md
3. **MISRA C:2012** (the remaining C only) - See MISRA_DEVIATIONS.md
4. **NASA NPR 7150.2** - Software Engineering Requirements

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
For C callers (`smart_qso.h`; Revision 1.0 also listed a `SMART_QSO_REQUIRE_BOUNDS`, which does not exist). The Rust
relies on bounds and overflow checks instead (`docs/development/CODING_STANDARDS.md` §11.4, §11.5):
```c
SMART_QSO_REQUIRE(condition, message);           // Precondition
SMART_QSO_ENSURE(condition, message);            // Postcondition
SMART_QSO_REQUIRE_NOT_NULL(ptr);                 // NULL check
SMART_QSO_REQUIRE_OR(condition, message, retval) // Precondition with a given result
```

#### Safe String Operations
```c
safe_strncpy(dest, dest_size, src, max_chars, &chars_copied);
safe_memcpy(dest, dest_size, src, count);
```

## Build Configuration

From `software/flight` (the repository-wide commands are in `software/rust/README.md`):

```bash
# Debug build with coverage (C and the Rust library)
cmake -B build -DCMAKE_BUILD_TYPE=Debug -DENABLE_COVERAGE=ON

# Release build with warnings as errors
cmake -B build -DCMAKE_BUILD_TYPE=Release -DENABLE_WERROR=ON

# Flight configuration of the host simulation (runs until stopped)
cmake -B build -DCMAKE_BUILD_TYPE=Release -DFLIGHT_BUILD=ON

# The STM32L4R5ZI image
cd ../rust/firmware && cargo build --release
```

## Documentation Files

| Document | Purpose |
|----------|---------|
| FDIR_DESIGN.md | Fault Detection, Isolation, Recovery design |
| MEMORY_POLICY.md | Static allocation policy |
| LOOP_BOUNDS.md | Loop iteration limits |
| MISRA_DEVIATIONS.md | MISRA C:2012 deviation record (the remaining C) |
| JPL_POWER_OF_TEN_AUDIT.md | JPL coding rules compliance |
| RTOS_ARCHITECTURE.md | Run-time architecture (Embassy executor; no RTOS) |
| TIMING_ANALYSIS.md | Timing (no measurements yet) |

## Software Classification

This software is **Class C** per NPR 7150.2D Appendix D: CubeSat flight software on a proposed Class D payload. The
determination and its rationale are in `docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1 (2026-10-25). Safety
criticality is a separate determination under NASA-STD-8739.8 (SDP §3.2, 2026-10-25): the components that implement
the software hazard controls are safety-critical, the rest is mission-critical.

## Contact

SMART-QSO Team
https://github.com/cheesejaguar/SMART-QSO

---

*Generated with Doxygen - Following NASA Ames SmallSat heritage patterns*

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | — | Initial version |
| 1.1 | 2026-10-24 | The flight software is Rust: the API is the C ABI of the Rust modules; architecture and module list as built (non-existent modules removed); Rust coding standard; firmware build |
| 1.1, note | 2026-10-25 | Software classification: NPR 7150.2D Class C with a pointer to the SDP §3.1 rationale; safety criticality open |
| 1.2 | 2026-10-27 | Documentation audit (post-CDR 2026): modules added since 1.1 (hk_store, ops_state, passivation, eps_lockout, fram_health, receiver, station_id); safety criticality as determined in SDP §3.2; build commands' working directory; repository URL |
