# RALPH.md - Repository Audit for Launch-ready Program Hardening

## NASA Flight Software Code Quality Audit Report
**Repository**: SMART-QSO
**Audit Date**: 2026-01-02
**Governing Standards**: NPR 7120.5 (Program/Project Management), NPR 7123.1 (Systems Engineering)
**Technical Standards**: NPR 7150.2, NASA-STD-8719.13, NASA-STD-8739.8, JPL Institutional Coding Standard, MISRA C:2012

---

## Executive Summary

This document provides a comprehensive audit of the SMART-QSO flight software repository against NASA code quality standards. All documentation, processes, and lifecycle activities follow **NPR 7120.5** (NASA Space Flight Program and Project Management Requirements) and **NPR 7123.1** (NASA Systems Engineering Processes and Requirements) as the governing project management and systems engineering framework.

This audit draws process heritage from NASA Ames Research Center small satellite missions including **GeneSat-1**, **PharmaSat**, **O/OREOS**, and **EcAMSat**, which successfully demonstrated rigorous yet appropriately tailored approaches for CubeSat-class missions.

Each section contains actionable checklist items with detailed implementation instructions for a coding agent to bring this software to flight-ready status.

**Current State**: Pre-CDR prototype with basic functionality
**Target State**: Flight-qualified software per NASA Class C/D requirements (appropriate for CubeSat missions)
**Lifecycle Phase**: Phase B (Preliminary Design) → Phase C (Final Design & Fabrication)
**Process Heritage**: NASA Ames Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

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

### Part I: Software Development
1. [Code Architecture & Structure](#1-code-architecture--structure)
2. [Coding Standards Compliance](#2-coding-standards-compliance)
3. [Static Analysis & Code Quality Tools](#3-static-analysis--code-quality-tools)
4. [Testing Infrastructure](#4-testing-infrastructure)
5. [Safety-Critical Software Requirements](#5-safety-critical-software-requirements)

### Part II: Project Documentation
6. [NPR 7120.5/7123.1 Documentation Requirements](#6-npr-712057123-documentation-requirements)
7. [Configuration Management](#7-configuration-management)
8. [Build System & CI/CD](#8-build-system--cicd)
9. [Security & Data Integrity](#9-security--data-integrity)

### Part III: Payload & Platform Software
10. [Payload Software (Python)](#10-payload-software-python)
11. [Hardware Abstraction & Portability](#11-hardware-abstraction--portability)

### Part IV: Verification & Lifecycle
12. [Verification & Validation](#12-verification--validation)
13. [Lifecycle Phase Gate Requirements](#13-lifecycle-phase-gate-requirements)
14. [CubeSat-Specific Requirements & Standards](#14-cubesat-specific-requirements--standards)
15. [Additional Software Considerations for Space Environment](#15-additional-software-considerations-for-space-environment)

### Part V: Hardware & Software Design Completion
16. [Hardware Subsystem Design Completion](#16-hardware-subsystem-design-completion)
17. [Flight Software Implementation](#17-flight-software-implementation)

### Part VI: Test Procedures & Integration (NASA Ames Heritage)
18. [Hardware Verification Test Procedures](#18-hardware-verification-test-procedures)
19. [System Integration & Test Campaign](#19-system-integration--test-campaign)

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

## 14. CubeSat-Specific Requirements & Standards

This section covers CubeSat-specific standards, launch provider requirements, and regulatory compliance not addressed elsewhere. These are critical for mission success and launch approval.

### 14.1 CubeSat Design Specification (CDS) Compliance

#### 14.1.1 Cal Poly CubeSat Standard
- [ ] **Verify CDS compliance**: Create `docs/compliance/CDS_COMPLIANCE_MATRIX.md`:
  - [ ] Mechanical dimensions compliance (100mm × 100mm × 113.5mm for 1U)
  - [ ] Mass compliance (≤2.0 kg for 1U per Rev 14.1)
  - [ ] Rail specifications and surface finish
  - [ ] Center of gravity location (within 2cm of geometric center)
  - [ ] Deployment switch locations and operation
  - [ ] RBF (Remove Before Flight) pin requirements
  - [ ] Separation spring requirements

#### 14.1.2 Launch Provider Interface Requirements
- [ ] **Create Launch Provider Compliance Matrix**: Create `docs/compliance/LAUNCH_PROVIDER_ICD.md`:
  - [ ] Document target deployer (e.g., NanoRacks NRCSD, Rocket Lab Maxwell, ISIS QuadPack)
  - [ ] Deployer-specific ICD compliance matrix
  - [ ] Mechanical interface verification plan
  - [ ] Electrical interface requirements (deployment switches, inhibits)
  - [ ] Keep-out zones and protrusion limits
  - [ ] Mass properties reporting format

### 14.2 Environmental Testing Standards

#### 14.2.1 GEVS Compliance
- [ ] **Create Environmental Test Plan**: Create `docs/verification/ENVIRONMENTAL_TEST_PLAN.md` (per GSFC-STD-7000):
  - [ ] Test article definition (EM, Protoflight, Flight)
  - [ ] Test sequence and rationale
  - [ ] Proto-flight vs. qualification levels selection
  - [ ] Test facility requirements
  - [ ] Success criteria for each test

- [ ] **Vibration Testing**:
  - [ ] Random vibration test specification
  - [ ] Sine burst/sweep requirements
  - [ ] Shock test requirements (if required by provider)
  - [ ] Pre/post-test functional verification
  - [ ] Notching strategy for sensitive components

- [ ] **Thermal Vacuum Testing**:
  - [ ] Temperature range (operational and survival)
  - [ ] Number of thermal cycles
  - [ ] Dwell times at temperature extremes
  - [ ] Vacuum level requirements
  - [ ] Thermal balance test requirements
  - [ ] Functional test at temperature

- [ ] **EMC/EMI Testing** (per MIL-STD-461G tailored):
  - [ ] Conducted emissions (CE)
  - [ ] Radiated emissions (RE)
  - [ ] Conducted susceptibility (CS)
  - [ ] Radiated susceptibility (RS)
  - [ ] Launch provider EMC limits compliance

#### 14.2.2 Outgassing Compliance
- [ ] **Materials Outgassing**: Create `docs/compliance/MATERIALS_LIST.md`:
  - [ ] ASTM E595 compliance for all materials
  - [ ] TML < 1.0%, CVCM < 0.1% requirements
  - [ ] NASA Outgassing Database cross-reference
  - [ ] Non-compliant material waiver process

### 14.3 Orbital Debris Requirements (NASA-STD-8719.14)

- [ ] **Create Orbital Debris Assessment**: Create `docs/compliance/ORBITAL_DEBRIS_ASSESSMENT.md`:
  - [ ] 25-year deorbit compliance demonstration
  - [ ] Deorbit analysis with drag device (reference `mission/CONOPS.md`)
  - [ ] Casualty risk assessment (Dc < 1:10,000)
  - [ ] Passivation plan (battery, propulsion, pressure vessels)
  - [ ] Debris release assessment (no intentional debris)
  - [ ] Collision avoidance capability (if any)

- [ ] **Passivation Requirements**:
  - [ ] Battery passivation procedure
  - [ ] Stored energy depletion plan
  - [ ] RF transmitter disable at EOL
  - [ ] Documentation for launch provider

### 14.4 Workmanship Standards (NASA-STD-8739 Family)

- [ ] **Create Workmanship Plan**: Create `docs/manufacturing/WORKMANSHIP_PLAN.md`:
  - [ ] Reference applicable NASA-STD-8739.x standards
  - [ ] Inspection criteria and procedures
  - [ ] Non-conformance handling

- [ ] **Soldering/Wiring Standards**:
  - [ ] NASA-STD-8739.3 (Soldered Electrical Connections) or IPC J-STD-001 Space
  - [ ] NASA-STD-8739.4 (Crimping, Interconnecting Cables)
  - [ ] IPC-A-610 Class 3 (Space Addendum) acceptability criteria
  - [ ] Solder joint inspection requirements
  - [ ] Wire harness documentation

- [ ] **Conformal Coating & Staking**:
  - [ ] NASA-STD-8739.1 (Workmanship for Staking and Conformal Coating)
  - [ ] Coating material selection (polyurethane, silicone, etc.)
  - [ ] Coverage requirements and keep-out areas

- [ ] **ESD Control**:
  - [ ] ANSI/ESD S20.20 compliance
  - [ ] ESD-sensitive component handling procedures
  - [ ] ESD training requirements

### 14.5 Parts, Materials & Radiation

#### 14.5.1 EEE Parts
- [ ] **Create Parts List**: Create `docs/manufacturing/EEE_PARTS_LIST.md`:
  - [ ] All electronic parts with manufacturer P/N
  - [ ] Part classification (space-grade, hi-rel, COTS)
  - [ ] Derating analysis per EEE-INST-002 (or project derating policy)
  - [ ] Single-point failure identification
  - [ ] Obsolescence assessment

#### 14.5.2 Radiation Environment
- [ ] **Create Radiation Analysis**: Create `docs/environment/RADIATION_ANALYSIS.md`:
  - [ ] Mission orbital radiation environment (trapped protons, electrons, solar events)
  - [ ] Total Ionizing Dose (TID) assessment for mission duration
  - [ ] Single Event Effects (SEE) assessment
  - [ ] Component TID/SEE tolerance summary
  - [ ] Shielding analysis (if applicable)
  - [ ] Radiation mitigation strategies (especially for Jetson - reference R2 in Risk Register)

#### 14.5.3 Restricted/Prohibited Materials
- [ ] **Materials Compliance**: Update `docs/compliance/MATERIALS_LIST.md`:
  - [ ] No prohibited materials per launch provider
  - [ ] Flammability requirements (NASA-STD-6001 if applicable)
  - [ ] Magnetic materials assessment
  - [ ] Hazardous materials identification

### 14.6 RF & Spectrum Regulatory

#### 14.6.1 IARU Coordination
- [ ] **IARU Coordination Package**: Create `docs/regulatory/IARU_COORDINATION.md`:
  - [ ] Coordination application status tracker
  - [ ] Frequency coordination letter (when received)
  - [ ] Coordinated parameters record
  - [ ] Reference existing `docs/REGULATORY.md` for process

#### 14.6.2 National Spectrum Authority
- [ ] **National Licensing**: Create `docs/regulatory/SPECTRUM_LICENSE.md`:
  - [ ] FCC Experimental License or Part 97 coordination (if US)
  - [ ] National administration filing requirements
  - [ ] ITU coordination/notification status
  - [ ] Callsign assignment documentation

#### 14.6.3 RF Compliance Testing
- [ ] **RF Test Documentation**: Create `docs/verification/RF_TEST_RESULTS.md`:
  - [ ] Transmitter power output verification
  - [ ] Frequency accuracy and stability
  - [ ] Modulation characteristics (deviation, bandwidth)
  - [ ] Spurious emissions compliance
  - [ ] Occupied bandwidth measurement
  - [ ] EIRP calculation and verification

### 14.7 Ground Segment Documentation

#### 14.7.1 Ground Station Design
- [ ] **Ground Station Documentation**: Create `docs/ground/GROUND_STATION_DESIGN.md`:
  - [ ] Station architecture (antenna, TNC, software)
  - [ ] Link budget analysis (uplink if applicable, downlink)
  - [ ] Pass prediction and scheduling
  - [ ] Data handling and archiving

#### 14.7.2 Mission Operations
- [ ] **Mission Operations Plan**: Create `docs/operations/MISSION_OPERATIONS_PLAN.md`:
  - [ ] Operations team organization
  - [ ] Shift schedules and coverage
  - [ ] Nominal operations procedures
  - [ ] Contingency operations procedures
  - [ ] Handover procedures

#### 14.7.3 Command & Telemetry
- [ ] **Command/Telemetry Database**: Create `docs/operations/CMD_TLM_DATABASE.md`:
  - [ ] Telemetry point definitions (ID, type, units, limits)
  - [ ] Command definitions (if uplink capability added)
  - [ ] Derived parameters and calculations
  - [ ] Display page definitions

### 14.8 Export Control & Technology Transfer

- [ ] **Export Control Assessment**: Create `docs/compliance/EXPORT_CONTROL.md`:
  - [ ] ITAR applicability assessment (if US)
  - [ ] EAR/Commerce Control List review
  - [ ] Technology Assessment/Jurisdiction determination
  - [ ] Export license requirements (if any)
  - [ ] Foreign person access controls
  - [ ] International partner agreements (if applicable)

### 14.9 End-of-Life & Disposal

- [ ] **End-of-Life Plan**: Create `docs/operations/END_OF_LIFE_PLAN.md`:
  - [ ] Mission termination criteria
  - [ ] Passivation sequence
  - [ ] Final beacon/goodbye message
  - [ ] Battery discharge procedure
  - [ ] RF transmitter disable command/timer
  - [ ] Deorbit device activation (if applicable)
  - [ ] Post-mission tracking coordination

### 14.10 Pre-Launch & Launch Operations

#### 14.10.1 Integration & Test at Launch Site
- [ ] **Launch Site Operations Plan**: Create `docs/operations/LAUNCH_SITE_OPS.md`:
  - [ ] Pre-shipment checklist
  - [ ] Transport and handling procedures
  - [ ] Launch site integration procedures
  - [ ] Battery charging timeline
  - [ ] Final functional test procedures
  - [ ] RBF pin removal sequence
  - [ ] Deployment switch arming verification
  - [ ] Closeout inspection

#### 14.10.2 Launch Campaign Documentation
- [ ] **Launch Campaign Checklist**: Create `docs/operations/LAUNCH_CAMPAIGN_CHECKLIST.md`:
  - [ ] Required deliverables to launch provider
  - [ ] Safety data package
  - [ ] Mass properties final report
  - [ ] Frequency coordination confirmation
  - [ ] Insurance documentation
  - [ ] Flight prediction/TLE coordination

---

## 15. Additional Software Considerations for Space Environment

### 15.1 Radiation Effects on Software

- [ ] **Radiation Mitigation in Software**: Create `software/flight/docs/RADIATION_MITIGATION.md`:
  - [ ] Memory scrubbing implementation plan
  - [ ] EDAC (Error Detection and Correction) usage
  - [ ] Critical data voting/TMR strategy
  - [ ] Checkpoint and restart capability
  - [ ] Watchdog response to radiation upsets
  - [ ] Register refresh for configuration registers

### 15.2 Bootloader & Software Update

- [ ] **Bootloader Requirements**: Create `software/flight/docs/BOOTLOADER_REQUIREMENTS.md`:
  - [ ] Bootloader architecture (if applicable)
  - [ ] Golden image protection
  - [ ] Software update mechanism (if uplink added)
  - [ ] Rollback capability
  - [ ] Boot sequence and timeout handling

### 15.3 Time Management

- [ ] **Time Management Design**: Create `software/flight/docs/TIME_MANAGEMENT.md`:
  - [ ] Onboard time reference source
  - [ ] Time synchronization with ground (if applicable)
  - [ ] Time stamping of telemetry and events
  - [ ] Leap second handling
  - [ ] Time format standards (UTC, GPS, MET)

### 15.4 Autonomous Operations

- [ ] **Autonomy Design**: Create `software/flight/docs/AUTONOMY_DESIGN.md`:
  - [ ] Autonomous safe mode entry conditions
  - [ ] Beacon-only mode operation (no ground contact required)
  - [ ] Power-based operational mode selection
  - [ ] Thermal protection autonomous actions
  - [ ] Communication loss response

---

## 16. Hardware Subsystem Design Completion

This section provides detailed design completion checklists for each hardware subsystem. Design artifacts shall be documented and reviewed per NPR 7123.1 before fabrication.

### 16.1 Electrical Power Subsystem (EPS) Design

#### 16.1.1 Solar Array Design
- [ ] **Complete Solar Array Design**: Create `hardware/eps/SOLAR_ARRAY_DESIGN.md`:
  - [ ] Solar cell selection and procurement specification
  - [ ] String configuration (series/parallel) with voltage/current calculations
  - [ ] Interconnect design (welded vs. soldered tabs)
  - [ ] Coverglass selection and bonding
  - [ ] Bypass diode placement and ratings
  - [ ] Thermal expansion considerations
  - [ ] Wiring harness design to MPPT
  - [ ] Deployment mechanism interface (if deployable)
  - [ ] Shadow analysis for deployed configuration

#### 16.1.2 Battery Design
- [ ] **Complete Battery Design**: Create `hardware/eps/BATTERY_DESIGN.md`:
  - [ ] Cell selection (vendor, chemistry, capacity, cycle life)
  - [ ] Pack configuration (series/parallel)
  - [ ] Battery Management System (BMS) design
    - [ ] Cell balancing approach
    - [ ] Overcharge/overdischarge protection
    - [ ] Temperature monitoring and protection
    - [ ] Charge inhibit for launch
  - [ ] Mechanical packaging and thermal interface
  - [ ] Heater circuit design (if required)
  - [ ] Wiring and connector selection
  - [ ] Safety features (venting, fusing)

#### 16.1.3 Power Distribution Unit (PDU) Design
- [ ] **Complete PDU Design**: Create `hardware/eps/PDU_DESIGN.md`:
  - [ ] Voltage rail definitions (3.3V, 5V, 12V, battery bus)
  - [ ] Load switch design for each rail
  - [ ] Current sensing circuit design
  - [ ] Overcurrent protection sizing
  - [ ] Inrush current limiting
  - [ ] Power sequencing logic
  - [ ] Connector and harness interface
  - [ ] Telemetry points definition

#### 16.1.4 EPS Schematics and Layout
- [ ] **Complete EPS Electrical Design**:
  - [ ] Schematic capture (all sheets)
  - [ ] Bill of Materials (BOM) with alternates
  - [ ] PCB layout with design rule check (DRC) clean
  - [ ] Gerber files and fabrication notes
  - [ ] Assembly drawings
  - [ ] Thermal analysis of power dissipating components
  - [ ] EMC considerations (filtering, grounding)

### 16.2 Command & Data Handling (C&DH) / OBC Design

#### 16.2.1 OBC Hardware Design
- [ ] **Complete OBC Design**: Create `hardware/obc/OBC_DESIGN.md`:
  - [ ] Microcontroller selection justification
  - [ ] Memory architecture (RAM, Flash, FRAM/MRAM)
  - [ ] Clock source and distribution
  - [ ] Reset and watchdog circuitry
  - [ ] Power supply design (LDOs/DC-DC)
  - [ ] Debug and programming interface
  - [ ] Peripheral interfaces:
    - [ ] UART (to Jetson, to beacon TX)
    - [ ] SPI (to sensors, to storage)
    - [ ] I2C (to EPS, to ADCS)
    - [ ] GPIO (deployment switches, status LEDs)
    - [ ] ADC (analog telemetry)

#### 16.2.2 Data Storage Design
- [ ] **Complete Storage Design**: Create `hardware/obc/STORAGE_DESIGN.md`:
  - [ ] Primary storage (FRAM/MRAM) sizing and interface
  - [ ] Secondary storage (SD card or NAND) sizing
  - [ ] Wear leveling considerations
  - [ ] Data integrity features (ECC, CRC)
  - [ ] File system selection

#### 16.2.3 OBC Schematics and Layout
- [ ] **Complete OBC Electrical Design**:
  - [ ] Schematic capture (all sheets)
  - [ ] BOM with alternates
  - [ ] PCB layout (4-6 layer) with DRC clean
  - [ ] Stack-up definition
  - [ ] Impedance control for high-speed signals
  - [ ] EMC filtering and grounding

### 16.3 Communication Subsystem Design

#### 16.3.1 Transmitter Design
- [ ] **Complete TX Design**: Create `hardware/rf/TRANSMITTER_DESIGN.md`:
  - [ ] Transceiver/transmitter module selection
  - [ ] Frequency synthesizer/crystal reference
  - [ ] Modulator configuration (AFSK 1200 bps)
  - [ ] Power amplifier design/selection
  - [ ] Output matching network
  - [ ] Harmonic filtering (low-pass filter design)
  - [ ] RF power detection/monitoring
  - [ ] Temperature compensation

#### 16.3.2 Antenna Design
- [ ] **Complete Antenna Design**: Create `hardware/rf/ANTENNA_DESIGN.md`:
  - [ ] Antenna type selection (quarter-wave monopole)
  - [ ] Element material (tape spring, Nitinol, etc.)
  - [ ] Deployment mechanism design
  - [ ] Deployment actuation (burn wire, shape memory)
  - [ ] RF matching to 50Ω
  - [ ] Radiation pattern analysis
  - [ ] Ground plane considerations

#### 16.3.3 RF Schematics and Layout
- [ ] **Complete RF Electrical Design**:
  - [ ] Schematic capture
  - [ ] BOM with RF-grade components
  - [ ] PCB layout with RF design rules
  - [ ] Controlled impedance traces (50Ω)
  - [ ] Shielding considerations
  - [ ] Connector selection (SMA, U.FL)

### 16.4 Attitude Determination & Control System (ADCS) Design

#### 16.4.1 Sensor Design
- [ ] **Complete ADCS Sensor Design**: Create `hardware/adcs/SENSOR_DESIGN.md`:
  - [ ] Magnetometer selection and placement
  - [ ] Sun sensor design/selection (6-face coverage)
  - [ ] Gyroscope (if included) selection
  - [ ] Sensor interface circuits
  - [ ] Calibration approach

#### 16.4.2 Actuator Design
- [ ] **Complete Actuator Design**: Create `hardware/adcs/ACTUATOR_DESIGN.md`:
  - [ ] Magnetorquer coil design:
    - [ ] Core material selection (air core vs. permalloy)
    - [ ] Wire gauge and turns calculation
    - [ ] Dipole moment requirement
    - [ ] Driver circuit design (H-bridge)
    - [ ] Current sensing
  - [ ] Mechanical mounting and alignment

#### 16.4.3 ADCS Electronics
- [ ] **Complete ADCS Electrical Design**:
  - [ ] Schematic capture
  - [ ] BOM
  - [ ] PCB layout
  - [ ] Interface to OBC (I2C/SPI)

### 16.5 AI Payload (Jetson) Integration Design

#### 16.5.1 Jetson Integration
- [ ] **Complete Jetson Integration Design**: Create `hardware/payload-jetson/INTEGRATION_DESIGN.md`:
  - [ ] Carrier board design (if custom)
  - [ ] Power supply design (5V @ 3A capability)
  - [ ] Power gating circuit (enable/disable from OBC)
  - [ ] UART interface to OBC (level shifting if needed)
  - [ ] Storage interface (NVMe/SD)
  - [ ] Thermal interface design:
    - [ ] Heat spreader material selection
    - [ ] Thermal path to chassis
    - [ ] Thermal pad/paste selection
  - [ ] EMC shielding considerations

#### 16.5.2 Jetson Power Management
- [ ] **Jetson Power Design**: Create `hardware/payload-jetson/POWER_MANAGEMENT.md`:
  - [ ] DVFS profile configuration
  - [ ] Power limit settings
  - [ ] Fan disable/passive cooling verification
  - [ ] Power monitoring points
  - [ ] Soft shutdown sequence

### 16.6 Structure and Thermal Design

#### 16.6.1 Structural Design
- [ ] **Complete Structural Design**: Create `hardware/structure/STRUCTURAL_DESIGN.md`:
  - [ ] CAD model (STEP/IGES) of flight configuration
  - [ ] Mass properties (mass, CG, MOI)
  - [ ] Finite Element Analysis (FEA) for launch loads
  - [ ] Fastener torque specifications
  - [ ] Rail finish requirements
  - [ ] Deployment mechanism design (solar panels, antenna)
  - [ ] Venting paths for pressure equalization

#### 16.6.2 Thermal Design
- [ ] **Complete Thermal Design**: Create `hardware/structure/THERMAL_DESIGN.md`:
  - [ ] Thermal model (nodes, conductances)
  - [ ] Orbital thermal environment definition
  - [ ] Worst-case hot/cold analysis
  - [ ] Surface finish and coatings (α/ε selection)
  - [ ] Heater sizing (if required)
  - [ ] Thermal straps/interfaces
  - [ ] Jetson thermal path analysis

### 16.7 Harness and Interconnect Design

- [ ] **Complete Harness Design**: Create `hardware/harness/HARNESS_DESIGN.md`:
  - [ ] Harness block diagram
  - [ ] Connector selection and pin assignments
  - [ ] Wire gauge and insulation selection
  - [ ] Wire routing and tie-down points
  - [ ] Harness drawings with lengths
  - [ ] Shielding and grounding scheme
  - [ ] Harness mass budget

---

## 17. Flight Software Implementation

This section details the complete flight software implementation tasks, organized by module. Each module shall be implemented, unit tested, and code reviewed before integration.

### 17.1 OBC Flight Software Modules

#### 17.1.1 System Initialization Module
- [ ] **Implement `sw/flight/src/system_init.c`**:
  - [ ] Hardware initialization sequence
  - [ ] Clock configuration
  - [ ] Peripheral initialization
  - [ ] Watchdog initialization
  - [ ] Memory test (optional, for SRAM)
  - [ ] Load configuration from non-volatile storage
  - [ ] Boot reason determination and logging
  - [ ] Self-test execution
  - [ ] Unit tests in `sw/flight/tests/test_system_init.c`

#### 17.1.2 Scheduler/Executive Module
- [ ] **Implement `sw/flight/src/scheduler.c`**:
  - [ ] Time-triggered scheduler (if bare metal)
  - [ ] Task table definition
  - [ ] Task priority and period configuration
  - [ ] Deadline monitoring
  - [ ] CPU utilization measurement
  - [ ] Idle task handling
  - [ ] Unit tests in `sw/flight/tests/test_scheduler.c`

#### 17.1.3 Sensor Management Module
- [ ] **Implement `sw/flight/src/sensors.c`** (refactor from main.c):
  - [ ] Sensor initialization
  - [ ] Sensor polling (per configured period)
  - [ ] Sensor data conversion (raw to engineering units)
  - [ ] Sensor health monitoring (range checks)
  - [ ] Sensor data averaging/filtering
  - [ ] Sensor configuration from YAML
  - [ ] Unit tests in `sw/flight/tests/test_sensors.c`

#### 17.1.4 EPS Control Module
- [ ] **Implement `sw/flight/src/eps_control.c`** (refactor from main.c):
  - [ ] Power mode state machine (SAFE/IDLE/ACTIVE)
  - [ ] Load switch control functions
  - [ ] Battery SOC estimation
  - [ ] Solar array power tracking
  - [ ] Power budget enforcement
  - [ ] Autonomous load shedding logic
  - [ ] EPS telemetry collection
  - [ ] Unit tests in `sw/flight/tests/test_eps_control.c`

#### 17.1.5 Fault Management Module
- [ ] **Implement `sw/flight/src/fault_mgmt.c`** (refactor from main.c):
  - [ ] Fault detection functions
  - [ ] Fault isolation logic
  - [ ] Fault recovery procedures
  - [ ] Fault logging (with timestamps)
  - [ ] Fault persistence across resets
  - [ ] Safe mode entry logic
  - [ ] Fault counters and trending
  - [ ] Unit tests in `sw/flight/tests/test_fault_mgmt.c`

#### 17.1.6 Telemetry Module
- [ ] **Implement `sw/flight/src/telemetry.c`**:
  - [ ] Telemetry packet formatting
  - [ ] Telemetry point registration
  - [ ] Telemetry sampling
  - [ ] Telemetry storage (circular buffer)
  - [ ] Telemetry downlink interface
  - [ ] Telemetry compression (if needed)
  - [ ] Unit tests in `sw/flight/tests/test_telemetry.c`

#### 17.1.7 Beacon Generation Module
- [ ] **Implement `sw/flight/src/beacon.c`**:
  - [ ] AX.25 frame formatting
  - [ ] Callsign encoding
  - [ ] Info field composition
  - [ ] Beacon scheduling (adaptive cadence)
  - [ ] Beacon transmission trigger
  - [ ] Integration with Jetson-generated text
  - [ ] Fallback to template text
  - [ ] Unit tests in `sw/flight/tests/test_beacon.c`

#### 17.1.8 UART Communication Module
- [ ] **Implement `sw/flight/src/uart_comm.c`** (refactor from main.c):
  - [ ] UART initialization
  - [ ] Transmit buffer management
  - [ ] Receive buffer management
  - [ ] Protocol framing (if applicable)
  - [ ] CRC calculation and verification
  - [ ] Timeout handling
  - [ ] Error recovery
  - [ ] Unit tests in `sw/flight/tests/test_uart_comm.c`

#### 17.1.9 ADCS Control Module
- [ ] **Implement `sw/flight/src/adcs.c`**:
  - [ ] Magnetometer driver
  - [ ] Sun sensor driver
  - [ ] Attitude determination (TRIAD or similar)
  - [ ] B-dot detumble controller
  - [ ] Sun-pointing controller
  - [ ] Magnetorquer driver
  - [ ] ADCS telemetry
  - [ ] Unit tests in `sw/flight/tests/test_adcs.c`

#### 17.1.10 Data Persistence Module
- [ ] **Implement `sw/flight/src/persistence.c`** (refactor from main.c):
  - [ ] Mission data structure definition
  - [ ] Save to non-volatile storage
  - [ ] Load from non-volatile storage
  - [ ] CRC protection
  - [ ] Redundant storage with voting
  - [ ] Corruption detection and recovery
  - [ ] Unit tests in `sw/flight/tests/test_persistence.c`

#### 17.1.11 Watchdog Module
- [ ] **Implement `sw/flight/src/watchdog.c`**:
  - [ ] Hardware watchdog initialization
  - [ ] Watchdog kick function
  - [ ] Multi-task watchdog monitoring
  - [ ] Watchdog timeout configuration
  - [ ] Pre-reset data save
  - [ ] Unit tests in `sw/flight/tests/test_watchdog.c`

#### 17.1.12 Deployment Control Module
- [ ] **Implement `sw/flight/src/deployment.c`**:
  - [ ] Deployment timer (T+30min from separation)
  - [ ] Deployment switch monitoring
  - [ ] Antenna deployment actuation
  - [ ] Solar panel deployment actuation (if applicable)
  - [ ] Deployment status telemetry
  - [ ] Deployment retry logic
  - [ ] Unit tests in `sw/flight/tests/test_deployment.c`

### 17.2 Payload (Jetson) Software Modules

#### 17.2.1 Health Interface Module
- [ ] **Implement `sw/payload/health_interface.py`**:
  - [ ] UART receive from OBC
  - [ ] Health data parsing (JSON)
  - [ ] Health data validation
  - [ ] Connection health monitoring
  - [ ] Reconnection logic
  - [ ] Unit tests in `sw/payload/tests/test_health_interface.py`

#### 17.2.2 LLM Inference Module
- [ ] **Enhance `sw/payload/llama_runner.py`**:
  - [ ] Model loading with error handling
  - [ ] Inference timeout enforcement
  - [ ] Memory usage monitoring
  - [ ] GPU utilization monitoring
  - [ ] Temperature monitoring
  - [ ] Graceful degradation on errors
  - [ ] Unit tests in `sw/payload/tests/test_llama_runner.py`

#### 17.2.3 Beacon Text Formatter
- [ ] **Enhance `sw/payload/formatter.py`**:
  - [ ] Output validation (ASCII only, length limit)
  - [ ] Content filtering (no inappropriate content)
  - [ ] Template fallback selection
  - [ ] Info field composition
  - [ ] Unit tests in `sw/payload/tests/test_formatter.py`

#### 17.2.4 Payload Supervisor
- [ ] **Implement `sw/payload/supervisor.py`**:
  - [ ] Process lifecycle management
  - [ ] Health reporting to OBC
  - [ ] Graceful shutdown handling
  - [ ] Power state awareness
  - [ ] Thermal throttling response
  - [ ] Unit tests in `sw/payload/tests/test_supervisor.py`

### 17.3 Integration Software

#### 17.3.1 OBC-Jetson Integration
- [ ] **Implement integration layer**:
  - [ ] Protocol definition (message types, formats)
  - [ ] Handshaking sequence
  - [ ] Health data transmission (OBC → Jetson)
  - [ ] Beacon text reception (Jetson → OBC)
  - [ ] Timeout handling
  - [ ] Integration tests in `sw/tests/integration/test_obc_jetson.c`

#### 17.3.2 End-to-End Beacon Flow
- [ ] **Implement E2E beacon generation**:
  - [ ] Sensor → Telemetry → Health data
  - [ ] Health data → Jetson → Beacon text
  - [ ] Beacon text → AX.25 frame → Transmitter
  - [ ] E2E tests in `sw/tests/integration/test_beacon_e2e.c`

### 17.4 Ground Software

#### 17.4.1 Beacon Decoder
- [ ] **Implement `sw/ground/beacon_decoder.py`**:
  - [ ] AX.25 frame parsing
  - [ ] Info field extraction
  - [ ] Telemetry parsing from machine-readable tail
  - [ ] Timestamp handling
  - [ ] Database storage
  - [ ] Unit tests in `sw/ground/tests/test_beacon_decoder.py`

#### 17.4.2 Telemetry Display
- [ ] **Implement `sw/ground/telemetry_display.py`**:
  - [ ] Real-time telemetry display
  - [ ] Historical trending
  - [ ] Limit monitoring and alarms
  - [ ] Export to CSV/JSON

#### 17.4.3 Pass Prediction
- [ ] **Implement `sw/ground/pass_prediction.py`**:
  - [ ] TLE ingestion
  - [ ] Pass calculation (using SGP4)
  - [ ] AOS/LOS prediction
  - [ ] Elevation/azimuth calculation

---

## 18. Hardware Verification Test Procedures

This section provides detailed test procedures for hardware verification, following NASA Ames SmallSat heritage (GeneSat, PharmaSat, O/OREOS, EcAMSat protoflight approach).

### 18.1 Test Procedure Documentation Standard

All test procedures shall follow this format:

```
PROCEDURE: [Test Name]
DOCUMENT: TP-[Subsystem]-[Number]
REVISION: [Rev Letter]
DATE: [Date]

1. PURPOSE
   [Why this test is performed]

2. SCOPE
   [What is tested, what is not]

3. APPLICABLE DOCUMENTS
   [Reference documents]

4. TEST ARTICLE
   [Description of unit under test]

5. TEST EQUIPMENT
   [List of required equipment with calibration requirements]

6. PRECONDITIONS
   [Required state before test]

7. SAFETY
   [Safety considerations]

8. PROCEDURE
   [Step-by-step instructions with expected results]

9. PASS/FAIL CRITERIA
   [Explicit criteria]

10. DATA RECORDING
    [What data to record]

11. POST-TEST
    [Actions after test completion]
```

### 18.2 Electrical Power Subsystem (EPS) Test Procedures

#### 18.2.1 EPS Functional Test
- [ ] **Create TP-EPS-001**: `docs/test_procedures/TP-EPS-001_Functional_Test.md`:
  - [ ] Verify all voltage rails (3.3V, 5V, 12V, battery bus)
  - [ ] Verify voltage regulation under load
  - [ ] Verify load switch operation
  - [ ] Verify current sensing accuracy
  - [ ] Verify overcurrent protection
  - [ ] Verify battery charging operation
  - [ ] Verify MPPT tracking
  - [ ] Verify telemetry accuracy

#### 18.2.2 EPS Load Profile Test
- [ ] **Create TP-EPS-002**: `docs/test_procedures/TP-EPS-002_Load_Profile.md`:
  - [ ] Simulate mission power profile
  - [ ] Verify power budget margins
  - [ ] Verify load transient response
  - [ ] Verify battery SOC tracking
  - [ ] Verify power mode transitions

#### 18.2.3 Battery Characterization Test
- [ ] **Create TP-EPS-003**: `docs/test_procedures/TP-EPS-003_Battery_Characterization.md`:
  - [ ] Measure cell capacity (C/5 discharge)
  - [ ] Measure internal resistance
  - [ ] Verify BMS protection functions
  - [ ] Verify cell balancing
  - [ ] Verify temperature protection
  - [ ] Verify charge inhibit function

#### 18.2.4 Solar Array Characterization
- [ ] **Create TP-EPS-004**: `docs/test_procedures/TP-EPS-004_Solar_Array.md`:
  - [ ] Measure I-V curve under sun simulator
  - [ ] Verify bypass diode function
  - [ ] Verify string isolation
  - [ ] Verify power output at various angles

### 18.3 Communication Subsystem Test Procedures

#### 18.3.1 RF Functional Test
- [ ] **Create TP-RF-001**: `docs/test_procedures/TP-RF-001_Functional_Test.md`:
  - [ ] Verify transmitter power output
  - [ ] Verify frequency accuracy
  - [ ] Verify modulation (AFSK deviation)
  - [ ] Verify AX.25 frame generation
  - [ ] Verify beacon timing
  - [ ] Verify callsign encoding

#### 18.3.2 RF Conducted Power Test
- [ ] **Create TP-RF-002**: `docs/test_procedures/TP-RF-002_Conducted_Power.md`:
  - [ ] Measure output power at antenna connector
  - [ ] Measure power at multiple temperatures
  - [ ] Verify power control (if adjustable)
  - [ ] Equipment: spectrum analyzer, power meter

#### 18.3.3 RF Spurious Emissions Test
- [ ] **Create TP-RF-003**: `docs/test_procedures/TP-RF-003_Spurious_Emissions.md`:
  - [ ] Measure harmonic levels (2nd, 3rd, etc.)
  - [ ] Measure out-of-band emissions
  - [ ] Verify compliance with IARU limits
  - [ ] Equipment: spectrum analyzer

#### 18.3.4 RF Modulation Quality Test
- [ ] **Create TP-RF-004**: `docs/test_procedures/TP-RF-004_Modulation.md`:
  - [ ] Measure FM deviation
  - [ ] Verify modulation index
  - [ ] Measure occupied bandwidth
  - [ ] Verify decodability with reference receiver

#### 18.3.5 Antenna Deployment Test
- [ ] **Create TP-RF-005**: `docs/test_procedures/TP-RF-005_Antenna_Deployment.md`:
  - [ ] Verify deployment mechanism function
  - [ ] Measure deployment time
  - [ ] Verify deployed antenna position
  - [ ] Verify antenna impedance (VNA measurement)
  - [ ] Repeat for multiple deployment cycles

### 18.4 OBC/C&DH Test Procedures

#### 18.4.1 OBC Functional Test
- [ ] **Create TP-OBC-001**: `docs/test_procedures/TP-OBC-001_Functional_Test.md`:
  - [ ] Verify boot sequence
  - [ ] Verify watchdog function
  - [ ] Verify UART interfaces
  - [ ] Verify SPI interfaces
  - [ ] Verify I2C interfaces
  - [ ] Verify GPIO functions
  - [ ] Verify ADC accuracy
  - [ ] Verify memory read/write

#### 18.4.2 OBC Software Load Test
- [ ] **Create TP-OBC-002**: `docs/test_procedures/TP-OBC-002_Software_Load.md`:
  - [ ] Verify bootloader operation
  - [ ] Verify software image loading
  - [ ] Verify software version reporting
  - [ ] Verify CRC verification

#### 18.4.3 OBC Timing Test
- [ ] **Create TP-OBC-003**: `docs/test_procedures/TP-OBC-003_Timing.md`:
  - [ ] Verify RTC accuracy
  - [ ] Verify task timing
  - [ ] Verify beacon cadence accuracy
  - [ ] Measure CPU utilization

#### 18.4.4 Watchdog Test
- [ ] **Create TP-OBC-004**: `docs/test_procedures/TP-OBC-004_Watchdog.md`:
  - [ ] Verify watchdog timeout
  - [ ] Verify reset recovery
  - [ ] Verify state preservation across reset

### 18.5 ADCS Test Procedures

#### 18.5.1 ADCS Sensor Test
- [ ] **Create TP-ADCS-001**: `docs/test_procedures/TP-ADCS-001_Sensor_Test.md`:
  - [ ] Magnetometer calibration
  - [ ] Magnetometer accuracy vs. reference
  - [ ] Sun sensor calibration
  - [ ] Sun sensor field-of-view verification

#### 18.5.2 ADCS Actuator Test
- [ ] **Create TP-ADCS-002**: `docs/test_procedures/TP-ADCS-002_Actuator_Test.md`:
  - [ ] Magnetorquer polarity verification
  - [ ] Magnetorquer dipole moment measurement
  - [ ] Magnetorquer driver operation
  - [ ] Equipment: Helmholtz cage or magnetometer

#### 18.5.3 ADCS Algorithm Test
- [ ] **Create TP-ADCS-003**: `docs/test_procedures/TP-ADCS-003_Algorithm_Test.md`:
  - [ ] B-dot algorithm verification (Helmholtz cage)
  - [ ] Attitude determination verification
  - [ ] Control law verification

### 18.6 Jetson Payload Test Procedures

#### 18.6.1 Jetson Power Test
- [ ] **Create TP-JETSON-001**: `docs/test_procedures/TP-JETSON-001_Power_Test.md`:
  - [ ] Measure idle power consumption
  - [ ] Measure inference power consumption
  - [ ] Measure peak power draw
  - [ ] Verify power gating function
  - [ ] Verify power sequencing

#### 18.6.2 Jetson Thermal Test
- [ ] **Create TP-JETSON-002**: `docs/test_procedures/TP-JETSON-002_Thermal_Test.md`:
  - [ ] Measure steady-state temperature at idle
  - [ ] Measure temperature during inference
  - [ ] Verify thermal throttling behavior
  - [ ] Verify thermal interface effectiveness

#### 18.6.3 Jetson Functional Test
- [ ] **Create TP-JETSON-003**: `docs/test_procedures/TP-JETSON-003_Functional_Test.md`:
  - [ ] Verify boot sequence
  - [ ] Verify UART communication with OBC
  - [ ] Verify LLM model loading
  - [ ] Verify inference operation
  - [ ] Verify fallback behavior

### 18.7 Mechanical/Deployment Test Procedures

#### 18.7.1 Fit Check
- [ ] **Create TP-MECH-001**: `docs/test_procedures/TP-MECH-001_Fit_Check.md`:
  - [ ] Verify CubeSat rail dimensions
  - [ ] Verify CG location
  - [ ] Verify mass
  - [ ] Verify fit in deployment pod mockup

#### 18.7.2 Deployment Test
- [ ] **Create TP-MECH-002**: `docs/test_procedures/TP-MECH-002_Deployment.md`:
  - [ ] Antenna deployment functional test
  - [ ] Solar panel deployment functional test (if applicable)
  - [ ] Deployment timing measurement
  - [ ] Multiple deployment cycle test

#### 18.7.3 Deployment Switch Test
- [ ] **Create TP-MECH-003**: `docs/test_procedures/TP-MECH-003_Deployment_Switches.md`:
  - [ ] Verify switch actuation
  - [ ] Verify switch inhibit function
  - [ ] Verify switch electrical parameters

---

## 19. System Integration & Test Campaign

This section describes the system-level integration and test campaign, following NASA Ames SmallSat heritage for protoflight testing.

### 19.1 Integration Philosophy (NASA Ames Heritage)

Following GeneSat/PharmaSat/O/OREOS/EcAMSat heritage:
- **Flatsat First**: Develop and test software on flatsat before flight hardware
- **Protoflight Approach**: Single flight unit tested to protoflight levels
- **Day-in-the-Life Testing**: Extended functional testing simulating on-orbit operations
- **Early Integration**: Start integration early to find issues early

### 19.2 Flatsat Development and Testing

#### 19.2.1 Flatsat Configuration
- [ ] **Create Flatsat**: Create `docs/integration/FLATSAT_CONFIGURATION.md`:
  - [ ] Engineering model or development boards for each subsystem
  - [ ] Equivalent interfaces to flight configuration
  - [ ] Ground support equipment (GSE) connections
  - [ ] Telemetry monitoring capability
  - [ ] Power supply configuration

#### 19.2.2 Flatsat Software Development
- [ ] **Flatsat Testing**: Create `docs/test_procedures/TP-FLATSAT-001_Software_Development.md`:
  - [ ] OBC software execution on flatsat
  - [ ] Subsystem interface verification
  - [ ] Fault injection testing
  - [ ] Regression testing
  - [ ] Day-in-the-life testing (72+ hours)

#### 19.2.3 Flatsat Test Campaign
- [ ] **Execute Flatsat Tests**:
  - [ ] Boot and initialization test
  - [ ] Mode transition test
  - [ ] Telemetry generation test
  - [ ] Beacon generation test
  - [ ] Fault response test
  - [ ] Power mode test
  - [ ] Extended duration test (week-long)

### 19.3 Flight Unit Integration

#### 19.3.1 Integration Sequence
- [ ] **Create Integration Plan**: Create `docs/integration/INTEGRATION_PLAN.md`:
  - [ ] Integration sequence (order of subsystem integration)
  - [ ] Integration test at each step
  - [ ] Acceptance criteria for each integration step
  - [ ] Fallback/rework plan

#### 19.3.2 Integration Procedures
- [ ] **Create TP-INT-001**: `docs/test_procedures/TP-INT-001_EPS_Integration.md`:
  - [ ] EPS to structure integration
  - [ ] Power bus verification
  - [ ] Load testing

- [ ] **Create TP-INT-002**: `docs/test_procedures/TP-INT-002_OBC_Integration.md`:
  - [ ] OBC to EPS integration
  - [ ] Power verification
  - [ ] Boot test
  - [ ] Interface test

- [ ] **Create TP-INT-003**: `docs/test_procedures/TP-INT-003_RF_Integration.md`:
  - [ ] RF board integration
  - [ ] Antenna cable routing
  - [ ] RF power test
  - [ ] Beacon test (into dummy load)

- [ ] **Create TP-INT-004**: `docs/test_procedures/TP-INT-004_ADCS_Integration.md`:
  - [ ] ADCS board integration
  - [ ] Sensor verification
  - [ ] Actuator verification

- [ ] **Create TP-INT-005**: `docs/test_procedures/TP-INT-005_Jetson_Integration.md`:
  - [ ] Jetson module integration
  - [ ] Thermal interface verification
  - [ ] Power gating verification
  - [ ] UART interface test
  - [ ] Inference test

- [ ] **Create TP-INT-006**: `docs/test_procedures/TP-INT-006_Final_Assembly.md`:
  - [ ] Final mechanical assembly
  - [ ] Harness installation
  - [ ] Antenna stowage
  - [ ] Solar panel installation
  - [ ] Mass properties measurement

### 19.4 System Functional Test

#### 19.4.1 Comprehensive Functional Test (CFT)
- [ ] **Create TP-SYS-001**: `docs/test_procedures/TP-SYS-001_Comprehensive_Functional.md`:
  - [ ] Boot and initialization
  - [ ] All telemetry points verification
  - [ ] All mode transitions
  - [ ] Beacon generation (all modes)
  - [ ] ADCS sensor verification
  - [ ] ADCS actuator verification
  - [ ] Power mode transitions
  - [ ] Fault injection and recovery
  - [ ] Deployment sequence (inhibited)

#### 19.4.2 Day-in-the-Life (DITL) Test
- [ ] **Create TP-SYS-002**: `docs/test_procedures/TP-SYS-002_Day_In_The_Life.md`:
  - [ ] 24-hour simulated mission operation
  - [ ] Orbital light/dark cycling simulation
  - [ ] Beacon cadence verification
  - [ ] Power budget verification
  - [ ] Thermal stability monitoring
  - [ ] Anomaly response verification
  - [ ] Data recording and analysis

### 19.5 Environmental Test Campaign

Following protoflight approach per GSFC-STD-7000 (GEVS).

#### 19.5.1 Pre-Environmental Baseline
- [ ] **Create TP-ENV-001**: `docs/test_procedures/TP-ENV-001_Pre_Environmental_Baseline.md`:
  - [ ] Mass measurement
  - [ ] CG measurement
  - [ ] Full functional test
  - [ ] RF characterization
  - [ ] Photograph configuration

#### 19.5.2 Vibration Test Procedure
- [ ] **Create TP-ENV-002**: `docs/test_procedures/TP-ENV-002_Vibration.md`:
  - [ ] Pre-test functional check
  - [ ] Low-level sine sweep (signature)
  - [ ] Random vibration (X, Y, Z axes):
    - [ ] Workmanship level (1 minute/axis) or protoflight level
    - [ ] Levels per launch provider spec
  - [ ] Post-axis sine sweep (signature comparison)
  - [ ] Post-axis functional check
  - [ ] Post-test full functional test
  - [ ] Post-test visual inspection

#### 19.5.3 Thermal Vacuum Test Procedure
- [ ] **Create TP-ENV-003**: `docs/test_procedures/TP-ENV-003_Thermal_Vacuum.md`:
  - [ ] Pre-test functional check
  - [ ] Chamber setup and configuration
  - [ ] Pump-down and leak check
  - [ ] Thermal cycling (protoflight: 4 cycles minimum):
    - [ ] Hot operational plateau (+50°C or per spec)
    - [ ] Cold operational plateau (-20°C or per spec)
    - [ ] Hot survival plateau (if different)
    - [ ] Cold survival plateau (if different)
    - [ ] Dwell time at each plateau (2+ hours)
  - [ ] Functional test at each temperature plateau:
    - [ ] Abbreviated functional (boot, beacon, telemetry)
    - [ ] Hot case Jetson operation
  - [ ] Return to ambient and vent
  - [ ] Post-test full functional test

#### 19.5.4 EMC/EMI Test Procedure
- [ ] **Create TP-ENV-004**: `docs/test_procedures/TP-ENV-004_EMC.md`:
  - [ ] Pre-test functional check
  - [ ] Radiated emissions test (RE102 tailored)
  - [ ] Conducted emissions test (CE102 tailored)
  - [ ] Radiated susceptibility test (RS103 tailored, if required)
  - [ ] Self-compatibility test (all systems operating)
  - [ ] Post-test functional check

#### 19.5.5 Deployment Test (Post-Environmental)
- [ ] **Create TP-ENV-005**: `docs/test_procedures/TP-ENV-005_Deployment_Post_Env.md`:
  - [ ] Antenna deployment test (flight-like actuation)
  - [ ] Solar panel deployment test (if applicable)
  - [ ] Deployment sequence timing verification

### 19.6 Final Acceptance Testing

#### 19.6.1 Final Functional Test
- [ ] **Create TP-FAT-001**: `docs/test_procedures/TP-FAT-001_Final_Functional.md`:
  - [ ] Complete functional test
  - [ ] RF characterization (final)
  - [ ] Battery capacity verification
  - [ ] Mass properties final measurement
  - [ ] Configuration verification

#### 19.6.2 Flight Configuration Verification
- [ ] **Create TP-FAT-002**: `docs/test_procedures/TP-FAT-002_Flight_Config.md`:
  - [ ] Software version verification
  - [ ] Configuration parameter verification
  - [ ] Callsign verification
  - [ ] Beacon format verification
  - [ ] All inhibits armed

### 19.7 Test Data Management

- [ ] **Create Test Data Management Plan**: Create `docs/integration/TEST_DATA_MANAGEMENT.md`:
  - [ ] Data recording requirements
  - [ ] Data format specifications
  - [ ] Data storage and backup
  - [ ] Test report template
  - [ ] Anomaly reporting procedure
  - [ ] Test log format

### 19.8 Anomaly Resolution

- [ ] **Create Anomaly Resolution Procedure**: Create `docs/integration/ANOMALY_RESOLUTION.md`:
  - [ ] Anomaly identification and logging
  - [ ] Anomaly categorization (severity, priority)
  - [ ] Investigation process
  - [ ] Root cause analysis requirements
  - [ ] Corrective action tracking
  - [ ] Verification of corrective action
  - [ ] Anomaly closure criteria

### 19.9 Pre-Ship Review Checklist

- [ ] **Create Pre-Ship Checklist**: Create `docs/integration/PRE_SHIP_CHECKLIST.md`:
  - [ ] All environmental tests complete and passed
  - [ ] All anomalies resolved or waived
  - [ ] All test data reviewed
  - [ ] Flight software version verified
  - [ ] Configuration baseline verified
  - [ ] Mass properties final
  - [ ] Frequency coordination confirmed
  - [ ] All documentation complete
  - [ ] Shipping container ready
  - [ ] RBF pin installed
  - [ ] Batteries charged to shipping SOC
  - [ ] All required deliverables to launch provider complete

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

### CubeSat & Launch Vehicle Standards

| Standard | Title | Applicability |
|----------|-------|---------------|
| Cal Poly CDS Rev 14.1 | CubeSat Design Specification | Mechanical/electrical form factor requirements |
| GSFC-STD-7000 (GEVS) | General Environmental Verification Standard | Environmental testing requirements |
| NASA-STD-8719.14 | Process for Limiting Orbital Debris | Deorbit and passivation requirements |
| MIL-STD-1540E | Test Requirements for Launch Vehicles | Environmental test levels |
| MIL-STD-461G | EMI/EMC Requirements | Electromagnetic compatibility testing |

### Workmanship & Manufacturing Standards

| Standard | Title | Applicability |
|----------|-------|---------------|
| NASA-STD-8739.1 | Workmanship for Staking/Conformal Coating | PCB protection |
| NASA-STD-8739.3 | Soldered Electrical Connections | Soldering workmanship |
| NASA-STD-8739.4 | Crimping, Interconnecting Cables | Cable/harness assembly |
| IPC J-STD-001 Space | Soldering for Electrical/Electronic Assemblies | Industry soldering standard |
| IPC-A-610 Class 3 | Acceptability of Electronic Assemblies | Inspection criteria |
| ANSI/ESD S20.20 | ESD Control Program | Electrostatic discharge protection |
| ASTM E595 | Outgassing of Spacecraft Materials | Materials qualification |

### Parts & Radiation Standards

| Standard | Title | Applicability |
|----------|-------|---------------|
| EEE-INST-002 | Instructions for EEE Parts Selection | Parts derating and selection |
| NASA-HDBK-4002A | Mitigating In-Space Charging Effects | Spacecraft charging |
| ECSS-E-HB-10-12A | Calculation of Radiation Effects | Radiation environment analysis |

### RF & Regulatory Standards

| Standard | Title | Applicability |
|----------|-------|---------------|
| IARU Satellite Coordination | Amateur Satellite Frequency Coordination | Spectrum coordination |
| ITU Radio Regulations | International Telecommunication Union | International spectrum rules |
| FCC Part 97 | Amateur Radio Service Rules | US amateur radio regulations |
| 47 CFR Part 25 | Satellite Communications (if non-amateur) | Commercial satellite licensing |

### Export Control

| Regulation | Title | Applicability |
|------------|-------|---------------|
| ITAR (22 CFR 120-130) | International Traffic in Arms Regulations | Defense articles/services |
| EAR (15 CFR 730-774) | Export Administration Regulations | Dual-use items |

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

*Document Version: 1.2*
*Generated: 2026-01-02*
*Audit Scope: SMART-QSO Repository @ commit f0186ce*
*Governing Framework: NPR 7120.5 Rev. F, NPR 7123.1 Rev. E*
*CubeSat Standard: Cal Poly CDS Rev 14.1*
