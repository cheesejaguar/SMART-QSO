# RALPH.md - Repository Audit for Launch-ready Program Hardening

## NASA Flight Software Code Quality Audit Report
**Repository**: SMART-QSO
**Audit Date**: 2026-01-02
**Governing Standards**: NPR 7120.5 (Program/Project Management), NPR 7123.1 (Systems Engineering)
**Technical Standards**: NPR 7150.2, NASA-STD-8719.13, NASA-STD-8739.8, JPL Institutional Coding Standard, MISRA C:2012

---

## Executive Summary

This document provides a comprehensive audit of the SMART-QSO flight software repository against NASA code quality standards. All documentation, processes, and lifecycle activities follow **NPR 7120.5** (NASA Space Flight Program and Project Management Requirements) and **NPR 7123.1** (NASA Systems Engineering Processes and Requirements) as the governing project management and systems engineering framework.

Each section contains actionable checklist items with detailed implementation instructions for a coding agent to bring this software to flight-ready status.

**Current State**: Pre-CDR prototype with basic functionality
**Target State**: Flight-qualified software per NASA Class C/D requirements (appropriate for CubeSat missions)
**Lifecycle Phase**: Phase B (Preliminary Design) → Phase C (Final Design & Fabrication)

---

## Governing Framework: NPR 7120.5 & NPR 7123.1

All project documentation and processes shall comply with:

- **NPR 7120.5** - NASA Space Flight Program and Project Management Requirements
  - Defines lifecycle phases, key decision points (KDPs), and required reviews
  - Establishes project planning, control, and reporting requirements
  - Mandates configuration management and risk management processes

- **NPR 7123.1** - NASA Systems Engineering Processes and Requirements
  - Defines the 17 SE processes and their required products
  - Establishes technical review requirements (SRR, SDR, PDR, CDR, etc.)
  - Mandates requirements development, verification, and validation processes

**Project Classification**: Class D (per NPR 7120.5 Appendix G - appropriate for CubeSat/small satellite missions)

---

## Table of Contents

1. [Code Architecture & Structure](#1-code-architecture--structure)
2. [Coding Standards Compliance](#2-coding-standards-compliance)
3. [Static Analysis & Code Quality Tools](#3-static-analysis--code-quality-tools)
4. [Testing Infrastructure](#4-testing-infrastructure)
5. [Safety-Critical Software Requirements](#5-safety-critical-software-requirements)
6. [NPR 7120.5/7123.1 Documentation Requirements](#6-npr-712057123-documentation-requirements)
7. [Configuration Management](#7-configuration-management)
8. [Build System & CI/CD](#8-build-system--cicd)
9. [Security & Data Integrity](#9-security--data-integrity)
10. [Payload Software (Python)](#10-payload-software-python)
11. [Hardware Abstraction & Portability](#11-hardware-abstraction--portability)
12. [Verification & Validation](#12-verification--validation)
13. [Lifecycle Phase Gate Requirements](#13-lifecycle-phase-gate-requirements)

---

## 1. Code Architecture & Structure

### 1.1 File Organization
- [ ] **Create modular source file structure**: Split `software/flight/src/main.c` (941 lines) into separate modules:
  - [ ] Create `software/flight/src/sensors.c` and `software/flight/include/sensors.h` - Extract sensor framework code (lines 70-142, 544-642, 673-734)
  - [ ] Create `software/flight/src/eps_control.c` and `software/flight/include/eps_control.h` - Extract EPS control functions (lines 36-45, 103-316)
  - [ ] Create `software/flight/src/fault_mgmt.c` and `software/flight/include/fault_mgmt.h` - Extract fault management (lines 59-67, 184-211, 318-342, 786-817)
  - [ ] Create `software/flight/src/uart_comm.c` and `software/flight/include/uart_comm.h` - Extract UART communication (lines 344-459, 461-542)
  - [ ] Create `software/flight/src/mission_data.c` and `software/flight/include/mission_data.h` - Extract persistence functions (lines 47-57, 140-211)
  - [ ] Create `software/flight/src/yaml_parser.c` and `software/flight/include/yaml_parser.h` - Extract YAML parsing (lines 652-708)
  - [ ] Create `software/flight/src/main.c` - Keep only initialization and main loop
  - [ ] Create `software/flight/include/smart_qso.h` - Master header with common types and configuration

- [ ] **Create include directory structure**:
  ```
  software/flight/
  ├── include/
  │   ├── smart_qso.h        # Master header
  │   ├── sensors.h
  │   ├── eps_control.h
  │   ├── fault_mgmt.h
  │   ├── uart_comm.h
  │   ├── mission_data.h
  │   ├── yaml_parser.h
  │   └── hal/               # Hardware abstraction
  │       ├── hal_gpio.h
  │       ├── hal_uart.h
  │       ├── hal_timer.h
  │       └── hal_watchdog.h
  ├── src/
  │   ├── main.c
  │   ├── sensors.c
  │   ├── eps_control.c
  │   ├── fault_mgmt.c
  │   ├── uart_comm.c
  │   ├── mission_data.c
  │   ├── yaml_parser.c
  │   └── hal/
  │       ├── hal_linux.c    # Linux simulation HAL
  │       └── hal_freertos.c # FreeRTOS flight HAL (stub)
  └── tests/
  ```

### 1.2 Header Guards & Include Structure
- [ ] **Add proper header guards** to all header files using the format:
  ```c
  #ifndef SMART_QSO_<MODULE>_H
  #define SMART_QSO_<MODULE>_H

  #ifdef __cplusplus
  extern "C" {
  #endif

  /* Content */

  #ifdef __cplusplus
  }
  #endif

  #endif /* SMART_QSO_<MODULE>_H */
  ```

- [ ] **Create include dependency documentation**: Add `software/flight/docs/INCLUDE_DEPENDENCIES.md` documenting the include hierarchy

### 1.3 Global State Management
- [ ] **Encapsulate global state**: Current global variables in `main.c` (lines 92-121) violate NASA modularity requirements. Refactor to:
  - [ ] Create a `SystemState` struct containing all system state
  - [ ] Pass state via function parameters instead of global access
  - [ ] Use accessor functions for state modification
  - [ ] Document thread-safety requirements for each state variable

---

## 2. Coding Standards Compliance

### 2.1 JPL Institutional Coding Standard (Power of Ten Rules)
- [ ] **Rule 1 - Simple control flow**: Audit all control flow; currently compliant
- [ ] **Rule 2 - Fixed loop bounds**: Add explicit loop bounds documentation:
  - [ ] Line 878: Main loop has bound of 1000 - document this is intentional for simulation
  - [ ] Add `#define MAX_LOOP_ITERATIONS` constants for all loops
  - [ ] Add runtime loop counter checks with fault logging if bounds exceeded

- [ ] **Rule 3 - No dynamic memory after init**: Currently compliant (stack-only), but:
  - [ ] Add explicit prohibition comment in coding standard doc
  - [ ] Add `-Wl,--wrap=malloc` linker flag to catch accidental dynamic allocation in flight build
  - [ ] Create `software/flight/docs/MEMORY_POLICY.md` documenting memory allocation policy

- [ ] **Rule 4 - Function length limit**: Refactor functions exceeding 60 lines:
  - [ ] `main()` (lines 819-941, 122 lines) - Split into `init_system()`, `run_main_loop()`, `shutdown_system()`
  - [ ] `send_telemetry_to_jetson()` (lines 462-526, 64 lines) - Split into `format_telemetry()` and `transmit_telemetry()`
  - [ ] `update_mission_state()` (lines 737-783, 46 lines) - Currently acceptable

- [ ] **Rule 5 - Assertions**: Add defensive assertions throughout:
  - [ ] Add `#include <assert.h>` or custom `SMART_QSO_ASSERT()` macro
  - [ ] Add precondition assertions to all public functions
  - [ ] Add postcondition assertions for critical operations
  - [ ] Minimum 2 assertions per function average

- [ ] **Rule 6 - Declare data at smallest scope**: Review and fix:
  - [ ] Line 465: `char buffer[2048]` - consider if this stack allocation is safe
  - [ ] Move variable declarations to point of first use where possible

- [ ] **Rule 7 - Check return values**: Add return value checking:
  - [ ] Line 363-364: `cfsetospeed`/`cfsetispeed` return values not checked
  - [ ] Line 380: `tcsetattr` return value handling incomplete
  - [ ] Line 439: `write()` partial write not fully handled
  - [ ] Line 457: `tcdrain()` return value not checked
  - [ ] All `fwrite()`/`fread()` calls need explicit error handling
  - [ ] All `snprintf()` calls need truncation detection

- [ ] **Rule 8 - Limited preprocessor use**: Currently mostly compliant; document any macros

- [ ] **Rule 9 - Restrict pointer use**: Add pointer validation:
  - [ ] Add NULL checks on all pointer parameters
  - [ ] Add bounds checking for array access via pointers
  - [ ] Document pointer ownership semantics

- [ ] **Rule 10 - Compile with all warnings**: Enhance compiler flags:
  - [ ] Add `-Werror` to treat warnings as errors
  - [ ] Add `-Wconversion` for implicit conversion warnings
  - [ ] Add `-Wshadow` for variable shadowing
  - [ ] Add `-Wcast-qual` for const correctness
  - [ ] Add `-Wstrict-prototypes` for function declarations
  - [ ] Add `-Wmissing-prototypes` for missing declarations

### 2.2 MISRA C:2012 Compliance
- [ ] **Create MISRA deviation log**: Create `software/flight/docs/MISRA_DEVIATIONS.md`
- [ ] **Fix MISRA violations**:
  - [ ] Rule 8.4: Add declarations before definitions
  - [ ] Rule 10.1-10.8: Fix implicit type conversions (e.g., line 137: `rand()` returns `int`, cast needed)
  - [ ] Rule 11.5: Remove casts that remove `const`/`volatile`
  - [ ] Rule 14.4: Fix controlling expressions (use explicit comparisons)
  - [ ] Rule 17.7: Check all function return values
  - [ ] Rule 21.3: Remove use of `<stdlib.h>` memory functions in flight code
  - [ ] Rule 21.6: Replace `printf`/`fprintf` with proper logging subsystem

### 2.3 Naming Conventions
- [ ] **Document naming convention**: Create `software/flight/docs/NAMING_CONVENTION.md`:
  - [ ] Functions: `module_action_object()` format (e.g., `eps_control_payload()`)
  - [ ] Types: `PascalCase` with `_t` suffix for typedefs
  - [ ] Constants: `UPPER_SNAKE_CASE`
  - [ ] Global variables: `g_` prefix
  - [ ] Static variables: `s_` prefix
  - [ ] Local variables: `lower_snake_case`

- [ ] **Apply naming convention** to existing code:
  - [ ] Rename `Sensor` to `Sensor_t`
  - [ ] Rename `EPSControlState` to `EpsControlState_t`
  - [ ] Rename `MissionData` to `MissionData_t`
  - [ ] Rename `FaultLogEntry` to `FaultLogEntry_t`
  - [ ] Add `s_` prefix to static variables (e.g., `s_sensors`, `s_num_sensors`)

---

## 3. Static Analysis & Code Quality Tools

### 3.1 Static Analysis Configuration
- [ ] **Configure cppcheck**: Create `software/flight/.cppcheck`:
  ```
  --enable=all
  --inconclusive
  --std=c11
  --suppress=missingIncludeSystem
  --error-exitcode=1
  --xml
  --output-file=cppcheck-results.xml
  ```

- [ ] **Configure clang-tidy**: Create `software/flight/.clang-tidy`:
  ```yaml
  Checks: >
    -*,
    bugprone-*,
    cert-*,
    clang-analyzer-*,
    cppcoreguidelines-*,
    misc-*,
    modernize-*,
    performance-*,
    readability-*,
    -readability-magic-numbers,
    -cppcoreguidelines-avoid-magic-numbers
  WarningsAsErrors: '*'
  HeaderFilterRegex: '.*'
  ```

- [ ] **Configure PC-lint/Polyspace** (if available): Create configuration for MISRA checking

- [ ] **Create analysis runner script**: Create `software/flight/scripts/run_analysis.sh`:
  ```bash
  #!/bin/bash
  set -e
  echo "Running cppcheck..."
  cppcheck --project=compile_commands.json --cppcheck-build-dir=.cppcheck-build
  echo "Running clang-tidy..."
  run-clang-tidy -p build
  echo "Static analysis complete"
  ```

### 3.2 Code Formatting
- [ ] **Configure clang-format**: Create `software/flight/.clang-format`:
  ```yaml
  BasedOnStyle: LLVM
  IndentWidth: 2
  ColumnLimit: 100
  BreakBeforeBraces: Attach
  AllowShortFunctionsOnASingleLine: None
  AllowShortIfStatementsOnASingleLine: false
  AlwaysBreakAfterReturnType: None
  ```

- [ ] **Apply formatting** to all source files:
  ```bash
  find software/flight -name "*.c" -o -name "*.h" | xargs clang-format -i
  ```

### 3.3 Complexity Metrics
- [ ] **Configure lizard** for cyclomatic complexity: Create `software/flight/scripts/check_complexity.sh`:
  ```bash
  #!/bin/bash
  lizard software/flight/src --CCN 15 --length 60 --warnings_only
  ```

- [ ] **Reduce complexity** in identified functions:
  - [ ] Target: All functions below CCN (cyclomatic complexity) of 15
  - [ ] Target: All functions below 60 lines

---

## 4. Testing Infrastructure

### 4.1 Unit Testing Framework Enhancement
- [ ] **Restructure test suite**: Current tests in `test_main.c` don't actually test `main.c` functions; they only test isolated copies of data structures

- [ ] **Create testable architecture**:
  - [ ] Extract all functions from `main.c` into separate modules (per Section 1.1)
  - [ ] Create test doubles for hardware dependencies
  - [ ] Use dependency injection pattern for testability

- [ ] **Implement proper unit tests** for each module:
  - [ ] `test_sensors.c` - Test sensor initialization, reading, YAML parsing
  - [ ] `test_eps_control.c` - Test power mode transitions, control logic
  - [ ] `test_fault_mgmt.c` - Test fault logging, recovery procedures
  - [ ] `test_uart_comm.c` - Test UART initialization, send/receive
  - [ ] `test_mission_data.c` - Test persistence save/load, data integrity

- [ ] **Add missing test cases**:
  - [ ] Boundary value tests for all numeric parameters
  - [ ] Error path tests for all failure conditions
  - [ ] State machine transition tests
  - [ ] Timer/timing behavior tests

### 4.2 Code Coverage
- [ ] **Configure gcov/lcov**: Update `CMakeLists.txt`:
  ```cmake
  option(ENABLE_COVERAGE "Enable code coverage" OFF)
  if(ENABLE_COVERAGE)
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} --coverage -fprofile-arcs -ftest-coverage")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} --coverage")
  endif()
  ```

- [ ] **Create coverage report script**: Create `software/flight/scripts/coverage.sh`:
  ```bash
  #!/bin/bash
  cmake -B build -DENABLE_COVERAGE=ON
  cmake --build build
  ctest --test-dir build
  lcov --capture --directory build --output-file coverage.info
  lcov --remove coverage.info '/usr/*' --output-file coverage.info
  genhtml coverage.info --output-directory coverage-report
  echo "Coverage report: coverage-report/index.html"
  ```

- [ ] **Achieve coverage targets**:
  - [ ] Statement coverage: ≥80%
  - [ ] Branch coverage: ≥75%
  - [ ] MC/DC coverage for safety-critical paths: 100%
  - [ ] Document untested code with justification

### 4.3 Integration Testing
- [ ] **Create integration test framework**: Create `software/flight/tests/integration/`:
  - [ ] `test_system_startup.c` - Test full system initialization sequence
  - [ ] `test_mode_transitions.c` - Test all operational mode transitions
  - [ ] `test_fault_scenarios.c` - Test fault injection and recovery
  - [ ] `test_telemetry_flow.c` - Test end-to-end telemetry generation

- [ ] **Create Hardware-in-the-Loop (HIL) test stubs**:
  - [ ] Create `software/flight/tests/hil/` directory
  - [ ] Document HIL test requirements in `HIL_TEST_PLAN.md`

### 4.4 Regression Testing
- [ ] **Create regression test suite**: Create `software/flight/tests/regression/`:
  - [ ] Add test for each closed bug/issue
  - [ ] Create `REGRESSION_TESTS.md` documenting each test case

---

## 5. Safety-Critical Software Requirements

### 5.1 Fault Detection, Isolation, and Recovery (FDIR)
- [ ] **Formalize FDIR architecture**: Create `software/flight/docs/FDIR_DESIGN.md`:
  - [ ] Document all detectable fault conditions
  - [ ] Document fault detection mechanisms
  - [ ] Document isolation strategies
  - [ ] Document recovery procedures
  - [ ] Map to risk register items

- [ ] **Implement comprehensive fault detection**:
  - [ ] Add watchdog timer with hardware reset capability (currently simulated)
  - [ ] Add stack overflow detection
  - [ ] Add memory corruption detection (CRC on critical data structures)
  - [ ] Add communication timeout detection
  - [ ] Add sensor range validation for all readings

- [ ] **Implement safe state**: Create explicit safe state definition:
  ```c
  typedef struct {
      bool payload_disabled;
      bool radio_beacon_only;
      bool adcs_minimal;
      uint8_t beacon_interval_s;
      uint8_t max_power_mw;
  } SafeStateConfig_t;
  ```

### 5.2 Watchdog Implementation
- [ ] **Implement proper watchdog**: Replace simulation with proper implementation:
  - [ ] Create `software/flight/src/hal/hal_watchdog.h` with hardware abstraction
  - [ ] Implement task-level watchdog monitoring
  - [ ] Implement multi-stage watchdog (warning → reset)
  - [ ] Add watchdog kick timing validation

### 5.3 Critical Data Protection
- [ ] **Add CRC protection** to persisted data:
  - [ ] Add CRC32 field to `MissionData` structure
  - [ ] Add CRC32 field to `EPSControlState` structure
  - [ ] Add CRC32 field to `FaultLogEntry` structure
  - [ ] Validate CRC on load, log corruption if mismatch
  - [ ] Create `software/flight/src/crc32.c` with CRC implementation

- [ ] **Implement triple modular redundancy (TMR)** for critical parameters:
  - [ ] Store critical config in 3 locations
  - [ ] Use voting logic for reads
  - [ ] Log disagreements as faults

### 5.4 Deterministic Behavior
- [ ] **Remove non-determinism**: Current use of `rand()` (line 137) is unsuitable for flight:
  - [ ] Replace with deterministic pseudo-random number generator for testing
  - [ ] For flight, use actual sensor readings for jitter/noise
  - [ ] Document all sources of non-determinism

- [ ] **Add timing constraints**:
  - [ ] Document worst-case execution time (WCET) for critical paths
  - [ ] Add timing assertions for deadline monitoring
  - [ ] Create `software/flight/docs/TIMING_ANALYSIS.md`

### 5.5 Command Validation
- [ ] **Implement command validation** (when uplink added):
  - [ ] Create command whitelist
  - [ ] Add parameter range validation
  - [ ] Implement command authentication if required
  - [ ] Add command logging

### 5.6 State Machine Formalization
- [ ] **Formalize operational modes** as explicit state machine:
  - [ ] Create `software/flight/src/state_machine.c`
  - [ ] Define all states (SAFE, IDLE, ACTIVE, etc.)
  - [ ] Define all valid transitions
  - [ ] Add transition guards and actions
  - [ ] Add state entry/exit actions
  - [ ] Log all state transitions

---

## 6. NPR 7120.5/7123.1 Documentation Requirements

All documentation shall comply with NPR 7120.5 (Program/Project Management) and NPR 7123.1 (Systems Engineering) requirements. Documentation is organized by lifecycle phase and SE process area.

### 6.1 Project Planning Documents (NPR 7120.5 Chapter 3)

#### 6.1.1 Project Plan
- [ ] **Create Project Plan**: Create `docs/project/PROJECT_PLAN.md` (per NPR 7120.5 §3.3):
  - [ ] Project objectives and success criteria
  - [ ] Work Breakdown Structure (WBS)
  - [ ] Schedule with milestones and KDPs
  - [ ] Resource allocation
  - [ ] Roles and responsibilities
  - [ ] Communication plan
  - [ ] Decision authority matrix

#### 6.1.2 Software Development Plan (SDP)
- [ ] **Create SDP**: Create `docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` (per NPR 7150.2 §3.1):
  - [ ] Development lifecycle model (tailored per NPR 7123.1)
  - [ ] Software classification (Class C/D per NPR 7150.2)
  - [ ] Coding standards reference (JPL/MISRA)
  - [ ] Review procedures and entry/exit criteria
  - [ ] Testing approach and coverage requirements
  - [ ] Configuration management procedures
  - [ ] Risk management approach
  - [ ] Metrics collection plan
  - [ ] Tool qualification requirements

#### 6.1.3 Systems Engineering Management Plan (SEMP)
- [ ] **Create SEMP**: Create `docs/project/SEMP.md` (per NPR 7123.1 §4.1):
  - [ ] Technical approach
  - [ ] SE process tailoring rationale
  - [ ] Technical performance measures (TPMs)
  - [ ] Technical review schedule
  - [ ] Integration strategy
  - [ ] Verification and validation approach

### 6.2 Requirements Documentation (NPR 7123.1 SE Process 3)

#### 6.2.1 Mission Requirements Document
- [ ] **Create Mission Requirements**: Create `docs/requirements/MISSION_REQUIREMENTS.md`:
  - [ ] Level 1 (Mission) requirements with unique IDs (MRD-xxx)
  - [ ] Derived from ConOps and mission objectives
  - [ ] Success criteria for each requirement
  - [ ] Verification method assignment (A/I/D/T)

#### 6.2.2 System Requirements Specification
- [ ] **Create System Requirements**: Create `docs/requirements/SYSTEM_REQUIREMENTS.md`:
  - [ ] Level 2 (System) requirements with unique IDs (SYS-xxx)
  - [ ] Traceability to mission requirements
  - [ ] Functional, performance, and interface requirements
  - [ ] Environmental requirements
  - [ ] Verification cross-reference matrix

#### 6.2.3 Software Requirements Specification (SRS)
- [ ] **Create SRS**: Create `docs/software/SOFTWARE_REQUIREMENTS.md` (per NPR 7150.2 §3.2):
  - [ ] Level 3 (Software) requirements with unique IDs (SRS-xxx)
  - [ ] Extract requirements from ARCHITECTURE.md and CONOPS.md
  - [ ] Classify requirements (functional, performance, safety, security)
  - [ ] Define verification method for each requirement (A/I/D/T)
  - [ ] Traceability to system requirements
  - [ ] Safety-critical requirements flagged per NASA-STD-8719.13

### 6.3 Requirements Traceability (NPR 7123.1 SE Process 4)
- [ ] **Create RTM**: Create `docs/requirements/REQUIREMENTS_TRACEABILITY.md`:
  - [ ] Bidirectional traceability: Mission → System → Software → Design → Code → Test
  - [ ] Map requirements to design elements
  - [ ] Map requirements to code modules (file:line references)
  - [ ] Map requirements to test cases
  - [ ] Ensure 100% requirements coverage
  - [ ] Identify orphan requirements and orphan code
  - [ ] Generate traceability metrics

### 6.4 Design Documentation (NPR 7123.1 SE Process 5)

#### 6.4.1 Software Design Description (SDD)
- [ ] **Create SDD**: Create `docs/software/SOFTWARE_DESIGN.md` (per NPR 7150.2 §3.3):
  - [ ] System context and boundaries
  - [ ] Software architecture diagrams
  - [ ] Module decomposition
  - [ ] Data flow diagrams
  - [ ] State machine definitions
  - [ ] Algorithm descriptions
  - [ ] Design rationale and trade studies
  - [ ] Traceability to SRS

#### 6.4.2 Interface Control Documents (ICDs)
- [ ] **Create Master ICD**: Create `docs/interfaces/INTERFACE_CONTROL_MASTER.md`:
  - [ ] Interface identification matrix
  - [ ] Interface ownership assignments
  - [ ] Change control procedures

- [ ] **Create OBC-Jetson ICD**: Create `docs/interfaces/ICD_OBC_JETSON.md`:
  - [ ] UART physical interface specification (voltage, timing, pinout)
  - [ ] Message format specification (header, payload, CRC)
  - [ ] Protocol state machine
  - [ ] Timing requirements and timeouts
  - [ ] Error handling and recovery

- [ ] **Create OBC-EPS ICD**: Create `docs/interfaces/ICD_OBC_EPS.md`:
  - [ ] Control interface specification
  - [ ] Telemetry interface specification
  - [ ] Command set with parameters
  - [ ] Power sequencing requirements

- [ ] **Create Beacon Format ICD**: Expand `docs/BEACON_SPEC.md`:
  - [ ] Add byte-level format specification
  - [ ] Add timing specification
  - [ ] Add error handling specification
  - [ ] AX.25 compliance verification

### 6.5 Verification & Validation Documentation (NPR 7123.1 SE Processes 13-14)

#### 6.5.1 Software Test Plan
- [ ] **Create Test Plan**: Create `docs/software/SOFTWARE_TEST_PLAN.md` (per NPR 7150.2 §3.5):
  - [ ] Test strategy (unit, integration, system, acceptance)
  - [ ] Test environment requirements
  - [ ] Test case development procedures
  - [ ] Entry and exit criteria for each test level
  - [ ] Test data requirements
  - [ ] Roles and responsibilities
  - [ ] Schedule and resources

#### 6.5.2 Software Test Procedures
- [ ] **Create Test Procedures**: Create `docs/software/SOFTWARE_TEST_PROCEDURES.md`:
  - [ ] Step-by-step test procedures for each test case
  - [ ] Expected results and pass/fail criteria
  - [ ] Test setup and teardown procedures
  - [ ] Data recording requirements

#### 6.5.3 Verification Cross-Reference Matrix (VCRM)
- [ ] **Create VCRM**: Create `docs/verification/VERIFICATION_MATRIX.md`:
  - [ ] Requirements to verification method mapping
  - [ ] Verification status tracking
  - [ ] Verification closure documentation
  - [ ] Non-compliance tracking

### 6.6 Configuration Management Documentation (NPR 7120.5 §3.4)

#### 6.6.1 Configuration Management Plan
- [ ] **Create CM Plan**: Create `docs/project/CONFIGURATION_MANAGEMENT_PLAN.md`:
  - [ ] Configuration identification procedures
  - [ ] Configuration control procedures (change control board)
  - [ ] Configuration status accounting
  - [ ] Configuration audits (FCA/PCA)
  - [ ] Baseline management

#### 6.6.2 Software Version Description (SVD)
- [ ] **Create SVD template**: Create `docs/software/SOFTWARE_VERSION_DESCRIPTION.md` (per NPR 7150.2):
  - [ ] Version numbering scheme (semantic versioning)
  - [ ] Build identification
  - [ ] Change log from previous version
  - [ ] Known issues/limitations
  - [ ] Build configuration and environment
  - [ ] Compiler/tool versions

### 6.7 Risk Management Documentation (NPR 7120.5 §3.5)
- [ ] **Enhance Risk Register**: Update `mission/RISK_REGISTER.md`:
  - [ ] Add software-specific risks
  - [ ] Add likelihood × consequence scoring
  - [ ] Add risk mitigation plans with owners and due dates
  - [ ] Add risk tracking and closure criteria
  - [ ] Monthly risk status updates

### 6.8 Safety Documentation (NPR 7120.5 §3.6, NASA-STD-8719.13)
- [ ] **Create Software Safety Plan**: Create `docs/safety/SOFTWARE_SAFETY_PLAN.md`:
  - [ ] Safety organization and responsibilities
  - [ ] Hazard identification methodology
  - [ ] Safety requirements flow-down
  - [ ] Safety verification approach

- [ ] **Create Hazard Analysis**: Create `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md`:
  - [ ] Preliminary Hazard List (PHL)
  - [ ] System Hazard Analysis (SHA) for software
  - [ ] Subsystem Hazard Analysis (SSHA)
  - [ ] Hazard tracking and closure

### 6.9 Code Documentation
- [ ] **Add Doxygen comments** to all public functions:
  ```c
  /**
   * @brief Brief description
   * @param[in] param1 Description of input parameter
   * @param[out] param2 Description of output parameter
   * @return Description of return value
   * @pre Preconditions
   * @post Postconditions
   * @note Additional notes
   * @warning Warnings
   * @requirement SRS-xxx
   */
  ```

- [ ] **Configure Doxygen**: Create `software/flight/Doxyfile`:
  - [ ] Enable call graphs
  - [ ] Enable include dependency graphs
  - [ ] Generate PDF and HTML output
  - [ ] Enable requirement tracing tags

- [ ] **Generate API documentation**: Add Doxygen to build process

### 6.10 Review Packages (NPR 7123.1 Appendix G)

#### 6.10.1 PDR Package
- [ ] **Prepare PDR Package**: Create `docs/reviews/PDR/` directory:
  - [ ] System requirements baseline
  - [ ] Preliminary design documentation
  - [ ] Interface definitions
  - [ ] Preliminary V&V approach
  - [ ] Risk assessment
  - [ ] Resource estimates

#### 6.10.2 CDR Package
- [ ] **Prepare CDR Package**: Create `docs/reviews/CDR/` directory:
  - [ ] Final design documentation
  - [ ] As-built interface documentation
  - [ ] V&V plan and procedures
  - [ ] Updated risk assessment
  - [ ] Build/integration plan
  - [ ] Test readiness assessment

---

## 7. Configuration Management

### 7.1 Version Control Practices
- [ ] **Create branching strategy document**: Create `docs/development/BRANCHING_STRATEGY.md`:
  - [ ] Main branch protection rules
  - [ ] Feature branch naming convention
  - [ ] Release branch procedures
  - [ ] Hotfix procedures

- [ ] **Create commit message standard**: Create `docs/development/COMMIT_MESSAGES.md`:
  - [ ] Use conventional commits format
  - [ ] Require issue/ticket reference
  - [ ] Require sign-off for traceability

### 7.2 Release Management
- [ ] **Create release procedure**: Create `docs/development/RELEASE_PROCEDURE.md`:
  - [ ] Version numbering (semantic versioning)
  - [ ] Release checklist
  - [ ] Build verification
  - [ ] Tagging procedure

- [ ] **Create software configuration index**: Create `docs/software/CONFIGURATION_INDEX.md`:
  - [ ] List all configuration files
  - [ ] Document default values
  - [ ] Document valid ranges
  - [ ] Document flight vs. test configurations

### 7.3 Build Configuration
- [ ] **Separate flight and test builds**: Update CMakeLists.txt:
  ```cmake
  option(FLIGHT_BUILD "Build for flight hardware" OFF)
  option(TEST_BUILD "Build with test instrumentation" ON)

  if(FLIGHT_BUILD)
    add_definitions(-DFLIGHT_BUILD)
    # Disable all printf/debug output
    # Enable hardware watchdog
    # Disable simulation code
  endif()
  ```

- [ ] **Create build configuration matrix**: Document in `docs/software/BUILD_CONFIGURATIONS.md`:
  - [ ] Development build (Linux simulation)
  - [ ] Test build (with coverage/instrumentation)
  - [ ] Flight build (FreeRTOS, hardware HAL)

---

## 8. Build System & CI/CD

### 8.1 Enhanced CMake Configuration
- [ ] **Update CMakeLists.txt** with NASA-compliant build options:
  ```cmake
  cmake_minimum_required(VERSION 3.16)
  project(smart_qso_flight C)

  set(CMAKE_C_STANDARD 11)
  set(CMAKE_C_STANDARD_REQUIRED ON)
  set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

  # Compiler warnings (NASA/JPL standard)
  add_compile_options(
    -Wall -Wextra -Wpedantic -Werror
    -Wconversion -Wshadow -Wcast-qual
    -Wstrict-prototypes -Wmissing-prototypes
    -Wformat=2 -Wformat-security
    -fstack-protector-strong
    -D_FORTIFY_SOURCE=2
  )

  # Options
  option(ENABLE_COVERAGE "Enable code coverage" OFF)
  option(ENABLE_SANITIZERS "Enable address/UB sanitizers" OFF)
  option(FLIGHT_BUILD "Build for flight hardware" OFF)

  # ... rest of configuration
  ```

- [ ] **Add compile_commands.json generation** for static analysis tools

### 8.2 CI/CD Pipeline
- [ ] **Create GitHub Actions workflow**: Create `.github/workflows/ci.yml`:
  ```yaml
  name: Flight Software CI
  on: [push, pull_request]
  jobs:
    build-and-test:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - name: Install dependencies
          run: |
            sudo apt-get update
            sudo apt-get install -y cmake libcmocka-dev cppcheck clang-tidy lcov
        - name: Build
          run: |
            cmake -B build -DENABLE_COVERAGE=ON
            cmake --build build
        - name: Run tests
          run: ctest --test-dir build --output-on-failure
        - name: Static analysis
          run: |
            cppcheck --error-exitcode=1 software/flight/src/
        - name: Coverage report
          run: |
            lcov --capture --directory build --output-file coverage.info
            lcov --remove coverage.info '/usr/*' --output-file coverage.info
        - name: Upload coverage
          uses: codecov/codecov-action@v3
          with:
            files: coverage.info
  ```

- [ ] **Add pre-commit hooks**: Create `.pre-commit-config.yaml`:
  ```yaml
  repos:
    - repo: https://github.com/pre-commit/pre-commit-hooks
      rev: v4.4.0
      hooks:
        - id: trailing-whitespace
        - id: end-of-file-fixer
        - id: check-yaml
    - repo: https://github.com/pocc/pre-commit-hooks
      rev: v1.3.5
      hooks:
        - id: clang-format
        - id: cppcheck
          args: [--error-exitcode=1]
  ```

### 8.3 Build Verification
- [ ] **Create build verification script**: Create `software/flight/scripts/verify_build.sh`:
  - [ ] Compile with all warning levels
  - [ ] Run static analysis
  - [ ] Run all tests
  - [ ] Check code coverage thresholds
  - [ ] Verify no debug code in flight build

---

## 9. Security & Data Integrity

### 9.1 Input Validation
- [ ] **Add comprehensive input validation**:
  - [ ] Validate all UART received data (line 462+)
  - [ ] Validate all YAML configuration values
  - [ ] Validate all environment variable inputs (lines 401-434)
  - [ ] Add bounds checking for all array indexing

- [ ] **Create input validation module**: Create `software/flight/src/input_validation.c`:
  ```c
  bool validate_sensor_value(const char* id, double value);
  bool validate_power_mode(uint8_t mode);
  bool validate_uart_config(const char* device, int baudrate);
  ```

### 9.2 Buffer Overflow Protection
- [ ] **Review all buffer operations**:
  - [ ] Line 195, 201: `strncpy` usage - ensure null termination
  - [ ] Line 465: `char buffer[2048]` - add overflow check for `snprintf`
  - [ ] Line 624: `snprintf(out_text, 8, ...)` - verify buffer size adequate
  - [ ] All `strncpy` calls need explicit null termination

- [ ] **Replace unsafe functions**:
  - [ ] Create safe string functions in `software/flight/src/safe_string.c`
  - [ ] `safe_strcpy()` - always null-terminates
  - [ ] `safe_snprintf()` - checks for truncation

### 9.3 Data Integrity
- [ ] **Add checksums to all persisted data**:
  - [ ] Add CRC32 to mission data file format
  - [ ] Add CRC32 to EPS configuration file format
  - [ ] Add CRC32 to fault log file format
  - [ ] Verify checksums on load, reject corrupted data

- [ ] **Add telemetry integrity**:
  - [ ] Add sequence numbers to telemetry packets
  - [ ] Add CRC to UART communication
  - [ ] Implement retry logic for corrupted messages

### 9.4 TOCTOU Prevention
- [ ] **Review file operations** for Time-of-Check-Time-of-Use vulnerabilities:
  - [ ] Use atomic file operations where possible
  - [ ] Lock files during read-modify-write operations
  - [ ] Use `O_EXCL` flag for exclusive file creation

---

## 10. Payload Software (Python)

### 10.1 Code Quality
- [ ] **Add type hints** to all functions:
  - [ ] `agent.py`: Add complete type annotations
  - [ ] `formatter.py`: Add complete type annotations
  - [ ] `llama_runner.py`: Add complete type annotations

- [ ] **Configure mypy**: Create `software/payload/mypy.ini`:
  ```ini
  [mypy]
  python_version = 3.9
  strict = true
  ignore_missing_imports = true
  ```

- [ ] **Configure pylint**: Create `software/payload/.pylintrc`

- [ ] **Configure black**: Create `software/payload/pyproject.toml`

### 10.2 Testing
- [ ] **Expand test coverage**:
  - [ ] Create `software/payload/tests/test_formatter.py` - Unit tests for formatter functions
  - [ ] Create `software/payload/tests/test_agent.py` - Unit tests for agent functions
  - [ ] Create `software/payload/tests/test_llama_runner.py` - Mock tests for LLM runner
  - [ ] Add pytest configuration in `software/payload/pytest.ini`

- [ ] **Add property-based testing**: Use hypothesis for input validation testing

### 10.3 Error Handling
- [ ] **Improve error handling**:
  - [ ] Line 26-33 `agent.py`: Replace bare `except Exception` with specific exceptions
  - [ ] Line 35-36 `llama_runner.py`: Replace bare `except Exception` with specific handling
  - [ ] Add logging instead of silent failure
  - [ ] Add health status reporting for failures

### 10.4 Resource Management
- [ ] **Add resource limits**:
  - [ ] Add timeout to LLM generation
  - [ ] Add memory usage monitoring
  - [ ] Add CPU usage monitoring
  - [ ] Implement graceful degradation

---

## 11. Hardware Abstraction & Portability

### 11.1 Hardware Abstraction Layer (HAL)
- [ ] **Create HAL interfaces**: Create `software/flight/include/hal/`:
  - [ ] `hal_gpio.h` - GPIO interface
  - [ ] `hal_uart.h` - UART interface
  - [ ] `hal_spi.h` - SPI interface
  - [ ] `hal_i2c.h` - I2C interface
  - [ ] `hal_timer.h` - Timer interface
  - [ ] `hal_watchdog.h` - Watchdog interface
  - [ ] `hal_flash.h` - Flash storage interface

- [ ] **Create Linux HAL implementation**: `software/flight/src/hal/hal_linux.c`
  - [ ] Wrap current Linux-specific code
  - [ ] Use for development/simulation

- [ ] **Create FreeRTOS HAL stubs**: `software/flight/src/hal/hal_freertos.c`
  - [ ] Interface stubs for flight implementation
  - [ ] Document implementation requirements

### 11.2 Platform Abstraction
- [ ] **Abstract time functions**:
  ```c
  // hal_timer.h
  uint64_t hal_get_time_ms(void);
  void hal_delay_ms(uint32_t ms);
  ```

- [ ] **Abstract file operations**:
  ```c
  // hal_flash.h
  bool hal_flash_write(uint32_t addr, const void* data, size_t len);
  bool hal_flash_read(uint32_t addr, void* data, size_t len);
  ```

### 11.3 FreeRTOS Integration Preparation
- [ ] **Create RTOS abstraction**: Create `software/flight/include/rtos_abstraction.h`:
  - [ ] Task creation/deletion wrappers
  - [ ] Mutex/semaphore wrappers
  - [ ] Queue wrappers
  - [ ] Timer wrappers

- [ ] **Document FreeRTOS task structure**: Create `software/flight/docs/RTOS_ARCHITECTURE.md`:
  - [ ] Define task priorities
  - [ ] Define stack sizes
  - [ ] Define inter-task communication
  - [ ] Define timing requirements

---

## 12. Verification & Validation

### 12.1 Software Safety Analysis
- [ ] **Create software hazard analysis**: Create `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md`:
  - [ ] Identify software contributions to system hazards
  - [ ] Map to risk register items
  - [ ] Define safety controls in software
  - [ ] Define verification approach for safety controls

- [ ] **Create FMEA for software**: Create `docs/safety/SOFTWARE_FMEA.md`:
  - [ ] Identify failure modes for each module
  - [ ] Assess severity, occurrence, detection
  - [ ] Calculate RPN (Risk Priority Number)
  - [ ] Define mitigation for high-RPN items

### 12.2 Code Inspection Checklists
- [ ] **Create inspection checklists**: Create `docs/review/CODE_REVIEW_CHECKLIST.md`:
  - [ ] Coding standard compliance
  - [ ] Error handling completeness
  - [ ] Input validation
  - [ ] Resource management
  - [ ] Thread safety (for future RTOS)
  - [ ] Comments and documentation

### 12.3 Test Procedure Documentation
- [ ] **Document test procedures**: Create `software/flight/tests/TEST_PROCEDURES.md`:
  - [ ] Unit test execution procedure
  - [ ] Integration test procedure
  - [ ] Regression test procedure
  - [ ] Coverage measurement procedure

### 12.4 Verification Matrix Completion
- [ ] **Complete verification matrix**: Update `mission/VNV_PLAN.md`:
  - [ ] Add software-specific verification items
  - [ ] Map each requirement to verification method
  - [ ] Document verification status
  - [ ] Create verification closure summary template

---

## 13. Lifecycle Phase Gate Requirements (NPR 7120.5/7123.1)

This section defines the requirements for each lifecycle phase gate per NPR 7120.5 and NPR 7123.1. All items must be completed before proceeding to the next phase.

### 13.1 Phase A Exit / Phase B Entry (SRR/MDR)
*System Requirements Review / Mission Definition Review*

- [ ] **Mission Requirements Complete**:
  - [ ] Mission objectives defined and approved
  - [ ] ConOps document approved (exists: `mission/CONOPS.md`)
  - [ ] Preliminary mission requirements documented
  - [ ] Stakeholder requirements captured

- [ ] **Preliminary Technical Approach**:
  - [ ] System architecture concept defined (exists: `docs/ARCHITECTURE.md`)
  - [ ] Key trade studies documented
  - [ ] Technology readiness assessed

- [ ] **Project Planning**:
  - [ ] Preliminary project plan
  - [ ] Preliminary cost estimate
  - [ ] Preliminary schedule

### 13.2 Phase B Exit / Phase C Entry (PDR)
*Preliminary Design Review*

- [ ] **Requirements Baseline (NPR 7123.1 §4.3.1)**:
  - [ ] System requirements specification complete and approved
  - [ ] Software requirements specification complete
  - [ ] All requirements have unique identifiers
  - [ ] Requirements traceability matrix established
  - [ ] All TBDs/TBRs identified with closure plans

- [ ] **Preliminary Design Complete (NPR 7123.1 §4.4)**:
  - [ ] System architecture defined
  - [ ] Software design description complete
  - [ ] Interface control documents defined
  - [ ] FDIR design documented
  - [ ] State machine diagrams complete

- [ ] **Verification Approach (NPR 7123.1 §4.6)**:
  - [ ] Verification and validation plan drafted
  - [ ] Test strategy defined
  - [ ] Verification methods assigned to all requirements

- [ ] **Risk Assessment (NPR 7120.5 §3.5)**:
  - [ ] Risk register populated (exists: `mission/RISK_REGISTER.md`)
  - [ ] Software-specific risks identified
  - [ ] Mitigation plans defined

- [ ] **Safety Assessment (NASA-STD-8719.13)**:
  - [ ] Preliminary hazard analysis complete
  - [ ] Safety requirements identified

- [ ] **Project Readiness**:
  - [ ] Resource plan approved
  - [ ] Development environment established
  - [ ] Configuration management plan in place

### 13.3 Phase C Milestones (CDR)
*Critical Design Review*

- [ ] **Design Complete (NPR 7123.1 §4.4)**:
  - [ ] Detailed software design complete
  - [ ] All modules designed to code level
  - [ ] Interface specifications finalized
  - [ ] Database schemas defined
  - [ ] Design reviews completed for all modules

- [ ] **Implementation Ready (NPR 7150.2)**:
  - [ ] Coding standards documented and approved
  - [ ] Development environment fully configured
  - [ ] Build system verified
  - [ ] CI/CD pipeline operational
  - [ ] Static analysis tools configured

- [ ] **Test Readiness (NPR 7123.1 §4.6)**:
  - [ ] Test plan approved
  - [ ] Test procedures documented
  - [ ] Test environment specified
  - [ ] Test data requirements defined

- [ ] **Safety Analysis Complete (NASA-STD-8719.13)**:
  - [ ] System hazard analysis complete
  - [ ] Software hazard analysis complete
  - [ ] All hazards have mitigation or controls

- [ ] **Interfaces Finalized (NPR 7123.1 §4.5)**:
  - [ ] All ICDs approved and baselined
  - [ ] Interface verification approach defined

### 13.4 Phase C/D Transition (TRR/ORR)
*Test Readiness Review / Operational Readiness Review*

- [ ] **Code Complete (NPR 7150.2)**:
  - [ ] All flight code implemented
  - [ ] Code reviews completed for all modules
  - [ ] Static analysis clean (no critical/high findings)
  - [ ] Unit test coverage ≥80%
  - [ ] All code committed and baselined

- [ ] **Integration Complete (NPR 7123.1 §4.5)**:
  - [ ] Software integration testing complete
  - [ ] Hardware/software integration verified
  - [ ] All interfaces verified

- [ ] **Verification Progress (NPR 7123.1 §4.6)**:
  - [ ] All unit tests passing
  - [ ] Integration tests passing
  - [ ] System tests executed
  - [ ] Verification matrix shows >90% complete

- [ ] **Documentation Complete**:
  - [ ] All design documents updated as-built
  - [ ] User/operator documentation complete
  - [ ] Maintenance documentation complete

### 13.5 Phase D Exit (FRR/PSR)
*Flight Readiness Review / Pre-Ship Review*

- [ ] **Verification Complete (NPR 7123.1 §4.6.1)**:
  - [ ] 100% of requirements verified
  - [ ] All test procedures executed
  - [ ] All anomalies resolved or waived
  - [ ] Verification cross-reference matrix complete

- [ ] **Validation Complete (NPR 7123.1 §4.6.2)**:
  - [ ] System validates against ConOps
  - [ ] End-to-end mission scenarios tested
  - [ ] Operator training complete

- [ ] **Configuration Audit (NPR 7120.5 §3.4)**:
  - [ ] Functional Configuration Audit (FCA) complete
  - [ ] Physical Configuration Audit (PCA) complete
  - [ ] As-built documentation matches flight software

- [ ] **Safety Closure (NASA-STD-8719.13)**:
  - [ ] All hazards closed or accepted
  - [ ] Safety verification complete
  - [ ] Safety data package approved

- [ ] **Flight Software Delivery**:
  - [ ] Flight software version baselined
  - [ ] Software Version Description complete
  - [ ] Known issues documented and accepted
  - [ ] Rollback/recovery procedures verified

### 13.6 Documentation Checklist by Phase Gate

| Document | SRR | PDR | CDR | TRR | FRR |
|----------|:---:|:---:|:---:|:---:|:---:|
| Project Plan | Draft | Approved | Updated | Updated | Final |
| ConOps | Draft | Approved | — | — | — |
| SEMP | Draft | Approved | Updated | — | Final |
| Mission Requirements | Draft | Baselined | — | — | — |
| System Requirements | — | Baselined | — | — | — |
| Software Requirements | — | Draft | Baselined | — | — |
| RTM | — | Draft | Complete | Verified | Closed |
| Software Design | — | Preliminary | Complete | As-built | Final |
| ICDs | — | Draft | Baselined | Verified | Final |
| Test Plan | — | Draft | Approved | — | — |
| Test Procedures | — | — | Draft | Approved | Executed |
| VCRM | — | — | Draft | >90% | 100% |
| Risk Register | Initial | Updated | Updated | Updated | Closed |
| Hazard Analysis | PHL | SHA | Complete | Verified | Closed |
| SVD | — | — | — | Draft | Final |
| CM Plan | Draft | Approved | — | — | Audited |

---

## Summary Checklist

### Phase 1: Immediate Code Quality (Priority: Critical)
- [ ] Split main.c into modules (Section 1.1)
- [ ] Add compiler warnings and treat as errors (Section 2.1)
- [ ] Configure static analysis tools (Section 3.1)
- [ ] Fix return value checking (Section 2.1, Rule 7)
- [ ] Add defensive assertions (Section 2.1, Rule 5)
- [ ] Create CI/CD pipeline (Section 8.2)

### Phase 2: Testing Infrastructure (Priority: High)
- [ ] Restructure unit tests to actually test code (Section 4.1)
- [ ] Configure code coverage (Section 4.2)
- [ ] Achieve 80% statement coverage (Section 4.2)
- [ ] Add integration tests (Section 4.3)

### Phase 3: Safety-Critical Features (Priority: High)
- [ ] Implement proper watchdog (Section 5.2)
- [ ] Add CRC protection to persisted data (Section 5.3)
- [ ] Formalize state machine (Section 5.6)
- [ ] Create FDIR documentation (Section 5.1)

### Phase 4: Documentation (Priority: Medium)
- [ ] Create SRS with requirement IDs (Section 6.2)
- [ ] Create RTM (Section 6.3)
- [ ] Create ICDs (Section 6.4)
- [ ] Add Doxygen comments (Section 6.6)

### Phase 5: Hardware Abstraction (Priority: Medium)
- [ ] Create HAL layer (Section 11.1)
- [ ] Abstract platform dependencies (Section 11.2)
- [ ] Prepare FreeRTOS integration (Section 11.3)

### Phase 6: Final Verification (Priority: High)
- [ ] Complete software hazard analysis (Section 12.1)
- [ ] Complete FMEA (Section 12.1)
- [ ] Complete verification matrix (Section 12.4)
- [ ] Conduct code inspection (Section 12.2)

---

## Appendix A: NASA Standards Reference

### Governing Standards (Project Management & Systems Engineering)

| Standard | Title | Applicability |
|----------|-------|---------------|
| **NPR 7120.5** | **NASA Space Flight Program and Project Management Requirements** | **Primary governing standard for all project activities, lifecycle phases, KDPs, and reviews** |
| **NPR 7123.1** | **NASA Systems Engineering Processes and Requirements** | **Primary governing standard for all SE processes, technical reviews, and V&V activities** |

### Technical Implementation Standards

| Standard | Title | Applicability |
|----------|-------|---------------|
| NPR 7150.2 | NASA Software Engineering Requirements | Software development processes and products |
| NASA-STD-8719.13 | Software Safety Standard | Safety-critical software requirements |
| NASA-STD-8739.8 | Software Assurance Standard | Quality assurance processes |
| GSFC-STD-1000 | Rules for Flight Systems | Flight systems design guidelines |
| JPL D-60411 | JPL Institutional Coding Standard | C language coding rules (Power of Ten) |
| MISRA C:2012 | Guidelines for C | Industry coding standard for safety-critical systems |

### NPR 7120.5 Key References

| Section | Topic | Application to SMART-QSO |
|---------|-------|-------------------------|
| Chapter 2 | Program/Project Life Cycle | Phase definitions (A-F), KDPs |
| §3.3 | Project Planning | Project Plan requirements |
| §3.4 | Configuration Management | CM Plan, baselines, audits |
| §3.5 | Risk Management | Risk register, mitigation tracking |
| §3.6 | Safety & Mission Assurance | Safety plans, hazard tracking |
| Appendix G | Project Classification | Class D tailoring for CubeSats |
| Appendix J | Technical Reviews | SRR, PDR, CDR, TRR, FRR requirements |

### NPR 7123.1 Key References

| Section | Topic | Application to SMART-QSO |
|---------|-------|-------------------------|
| §4.1 | SE Planning | SEMP requirements |
| §4.2 | Requirements Development | Requirements documentation |
| §4.3 | Technical Solution | Design process |
| §4.4 | Design Solution | Design documentation |
| §4.5 | Product Integration | Integration planning |
| §4.6 | Verification & Validation | V&V plans, VCRM |
| Appendix C | SE Products | Required work products by phase |
| Appendix G | Technical Reviews | Review entry/exit criteria |

## Appendix B: Tool Requirements

| Tool | Purpose | Required Version |
|------|---------|-----------------|
| CMake | Build system | ≥3.16 |
| GCC/Clang | C compiler | GCC ≥9.0 / Clang ≥10.0 |
| cppcheck | Static analysis | ≥2.7 |
| clang-tidy | Static analysis | ≥10.0 |
| CMocka | Unit testing | ≥1.1 |
| lcov | Code coverage | ≥1.14 |
| Doxygen | Documentation | ≥1.9 |
| Python | Payload software | ≥3.9 |
| mypy | Python type checking | ≥1.0 |
| pytest | Python testing | ≥7.0 |

---

*Document Version: 1.1*
*Generated: 2026-01-02*
*Audit Scope: SMART-QSO Repository @ commit 883378e*
*Governing Framework: NPR 7120.5 Rev. F, NPR 7123.1 Rev. E*
