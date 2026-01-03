# SMART-QSO Test Procedures

**Document Number**: SMART-QSO-TP-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

**Governing Standards**:
- NPR 7150.2 Rev B - NASA Software Engineering Requirements
- GSFC-STD-7000 (GEVS) - General Environmental Verification Standard

**Heritage**: NASA Ames Research Center Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

---

## 1. Introduction

### 1.1 Purpose

This document provides detailed test procedures for verifying SMART-QSO flight software against the Software Requirements Specification (SRS).

### 1.2 Scope

Procedures cover:
- Unit test execution
- Integration test execution
- System test execution
- Acceptance test execution

### 1.3 Test Levels

| Level | Description | Environment |
|-------|-------------|-------------|
| Unit | Individual module functions | Host (simulation) |
| Integration | Module interactions | Flatsat |
| System | End-to-end functionality | Flight unit |
| Acceptance | Mission scenarios | Flight unit |

---

## 2. Test Environment Setup

### 2.1 Host Development Environment

**Requirements**:
- Linux (Ubuntu 20.04+) or macOS (12+)
- GCC ARM 12.x toolchain
- CMake 3.16+
- CMocka testing framework
- lcov coverage tool

**Setup Commands**:
```bash
# Ubuntu
sudo apt install gcc-arm-none-eabi cmake libcmocka-dev lcov

# macOS
brew install arm-none-eabi-gcc cmake cmocka lcov
```

### 2.2 Flatsat Environment

**Equipment**:
- OBC development board (NUCLEO-L476RG or Apollo4 dev kit)
- Jetson Orin Nano Developer Kit
- EPS emulator or prototype board
- UART USB adapters (x2)
- Power supply (5V/3A, 3.3V/1A)
- Multimeter
- Logic analyzer (optional)

**Connections**:
```
+----------+         +----------+         +----------+
|   Host   |--USB--->|   OBC    |--UART-->|  Jetson  |
|  (Debug) |         |  Board   |         |  Orin    |
+----------+         +----+-----+         +----------+
                          |
                     I2C  |  GPIO
                          v
                    +----------+
                    |   EPS    |
                    | Emulator |
                    +----------+
```

### 2.3 Software Build

```bash
# Clone repository
git clone https://github.com/SMART-QSO/smart-qso.git
cd smart-qso

# Build flight software (simulation)
cmake -S software/flight -B build -DSIMULATION_BUILD=ON
cmake --build build

# Build with coverage
cmake -S software/flight -B build -DENABLE_COVERAGE=ON
cmake --build build

# Run unit tests
ctest --test-dir build --output-on-failure
```

---

## 3. Unit Test Procedures

### 3.1 TP-UNIT-001: CRC32 Module Tests

**Purpose**: Verify CRC32 calculation per SRS-F061

**Preconditions**:
1. Build environment configured
2. CMocka installed

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Build test: `cmake --build build --target test_crc32` | Build successful | |
| 2 | Run test: `./build/tests/test_crc32` | All tests pass | |
| 3 | Verify known vector: CRC32("123456789") | 0xCBF43926 | |
| 4 | Verify empty data handling | No crash | |
| 5 | Verify single-bit detection | CRCs differ | |

**Pass Criteria**: All test cases pass

---

### 3.2 TP-UNIT-002: EPS Control Module Tests

**Purpose**: Verify power mode management per SRS-F010 through SRS-F017

**Preconditions**:
1. Build environment configured
2. test_eps_control built

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Run test: `./build/tests/test_eps_control` | All tests pass | |
| 2 | Verify SAFE mode transition | Mode = 0 when SOC < 25% | |
| 3 | Verify IDLE mode transition | Mode = 1 when SOC 25-55% | |
| 4 | Verify ACTIVE mode transition | Mode = 2 when SOC > 55% | |
| 5 | Verify payload enable blocked | No enable when SOC < 55% | |
| 6 | Verify persistence | State survives reinit | |

**Pass Criteria**: All 17 test cases pass

---

### 3.3 TP-UNIT-003: Fault Management Module Tests

**Purpose**: Verify FDIR per SRS-F040 through SRS-F045

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Run test: `./build/tests/test_fault_mgmt` | All tests pass | |
| 2 | Log single fault | Fault logged with CRC | |
| 3 | Log multiple faults | All faults recorded | |
| 4 | Test overflow handling | Old entries rotated | |
| 5 | Test severity levels | Stats tracked correctly | |
| 6 | Test recovery procedures | Recovery functions work | |

**Pass Criteria**: All 16 test cases pass

---

### 3.4 TP-UNIT-004: Sensor Module Tests

**Purpose**: Verify sensor polling per SRS-F070 through SRS-F073

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Run test: `./build/tests/test_sensors` | All tests pass | |
| 2 | Load YAML config | Sensors configured | |
| 3 | Poll sensors | Values returned | |
| 4 | Validate in-range | Validation passes | |
| 5 | Validate out-of-range | Validation fails | |
| 6 | Format telemetry | String formatted | |

**Pass Criteria**: All 22 test cases pass

---

### 3.5 TP-UNIT-005: Mission Data Module Tests

**Purpose**: Verify data persistence per SRS-F060 through SRS-F063

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Run test: `./build/tests/test_mission_data` | All tests pass | |
| 2 | Initialize fresh | Reset count = 1 | |
| 3 | Save and reload | Data persists | |
| 4 | Corrupt file | Handled gracefully | |
| 5 | Update statistics | Counters increment | |

**Pass Criteria**: All 17 test cases pass

---

### 3.6 TP-UNIT-006: UART Communication Module Tests

**Purpose**: Verify Jetson communication per SRS-F030 through SRS-F035

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Run test: `./build/tests/test_uart_comm` | All tests pass | |
| 2 | Test CRC-16 calculation | Known values match | |
| 3 | Format telemetry | Message valid | |
| 4 | Parse beacon response | Fields extracted | |
| 5 | Validate beacon text | ASCII check works | |

**Pass Criteria**: All 21 test cases pass

---

## 4. Integration Test Procedures

### 4.1 TP-INT-001: EPS + Sensors Integration

**Purpose**: Verify EPS correctly reads sensor data

**Equipment**: Flatsat with OBC and EPS emulator

**Preconditions**:
1. Flatsat assembled and powered
2. Flight software loaded
3. Serial console connected

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Power on flatsat | Boot message on console | |
| 2 | Set EPS emulator to 7.8V | | |
| 3 | Check telemetry output | BV=7.8V ±0.2V | |
| 4 | Set emulator to 6.5V | | |
| 5 | Check power mode | Mode = SAFE | |
| 6 | Set emulator to 8.0V | | |
| 7 | Check power mode | Mode = ACTIVE | |

**Pass Criteria**: Telemetry within ±2% of expected

---

### 4.2 TP-INT-002: OBC + Jetson Communication

**Purpose**: Verify UART communication per ICD-002

**Equipment**: Flatsat with OBC and Jetson

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Power on OBC | Boot complete | |
| 2 | Power on Jetson | Boot complete (~20s) | |
| 3 | Monitor UART | TELEMETRY messages sent | |
| 4 | Send BEACON_REQ | | |
| 5 | Wait for BEACON_RSP | Response < 10s | |
| 6 | Verify CRC | CRC matches | |
| 7 | Verify text | ASCII, ≤ 256 bytes | |

**Pass Criteria**: Beacon response within timeout with valid CRC

---

### 4.3 TP-INT-003: Power Mode Sequence

**Purpose**: Verify power mode transitions work end-to-end

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Set SOC = 20% | Mode = SAFE | |
| 2 | Verify loads disabled | Payload, radio off | |
| 3 | Set SOC = 50% | Mode = IDLE | |
| 4 | Verify radio enabled | Radio on | |
| 5 | Set SOC = 80% | Mode = ACTIVE | |
| 6 | Verify payload enabled | Jetson powered | |
| 7 | Set SOC = 30% | Mode = IDLE | |
| 8 | Verify graceful transition | No anomalies | |

**Pass Criteria**: All transitions occur correctly

---

### 4.4 TP-INT-004: Fault Injection

**Purpose**: Verify fault handling

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Inject UART timeout | Fallback beacon used | |
| 2 | Inject low voltage | SAFE mode entered | |
| 3 | Inject high temperature | Payload disabled | |
| 4 | Check fault log | All faults logged | |
| 5 | Remove fault conditions | Recovery occurs | |

**Pass Criteria**: All faults detected and handled

---

## 5. System Test Procedures

### 5.1 TP-SYS-001: End-to-End Beacon Generation

**Purpose**: Verify complete beacon chain from sensor to RF

**Equipment**: Flight unit, RF receiver

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Power on flight unit | Boot complete | |
| 2 | Verify power mode ACTIVE | Mode = 2 | |
| 3 | Wait for Jetson ready | STATUS received | |
| 4 | Wait for beacon interval | 30s | |
| 5 | Monitor RF output | Beacon transmitted | |
| 6 | Decode AX.25 frame | Valid frame | |
| 7 | Extract info field | Telemetry + AI text | |
| 8 | Verify callsign | Callsign present | |

**Pass Criteria**: Valid AX.25 beacon with AI text

---

### 5.2 TP-SYS-002: Day-in-the-Life Test

**Purpose**: Verify 24-hour autonomous operation

**Equipment**: Flight unit in thermal chamber (optional)

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Configure orbital simulation | 90-min orbit | |
| 2 | Start at full charge | SOC = 100% | |
| 3 | Begin eclipse cycling | 35 min sunlit, 30 min eclipse | |
| 4 | Monitor for 24 hours | | |
| 5 | Record all beacons | Count, timing | |
| 6 | Record mode transitions | Log entries | |
| 7 | Inject fault at T+12h | | |
| 8 | Verify recovery | System recovers | |
| 9 | Check final state | No anomalies | |

**Pass Criteria**:
- No unplanned resets
- All beacons transmitted
- Fault recovery successful
- Battery management nominal

---

### 5.3 TP-SYS-003: Performance Timing

**Purpose**: Verify timing requirements

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Measure main loop | < 20 ms (SRS-P001) | |
| 2 | Measure beacon TX | < 5 s (SRS-P002) | |
| 3 | Measure fault detect | < 100 ms (SRS-P003) | |
| 4 | Measure mode transition | < 1 s (SRS-P004) | |
| 5 | Measure init time | < 30 s (SRS-P008) | |

**Pass Criteria**: All timing requirements met

---

## 6. Acceptance Test Procedures

### 6.1 TP-ACC-001: Mission Scenario - Nominal Pass

**Purpose**: Demonstrate nominal operations during ground station pass

**Duration**: 15 minutes (simulated pass)

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Begin with sunlit, SOC 70% | Mode = ACTIVE | |
| 2 | Receive telemetry beacons | Every 30s | |
| 3 | Verify AI beacon content | Valid text | |
| 4 | Verify telemetry values | Within limits | |
| 5 | Simulate eclipse entry | | |
| 6 | Verify mode transition | Mode changes | |
| 7 | Verify fallback beacon | Template text | |

**Pass Criteria**: All nominal operations successful

---

### 6.2 TP-ACC-002: Mission Scenario - Eclipse Operations

**Purpose**: Demonstrate eclipse survival and recovery

**Duration**: 60 minutes

**Procedure**:

| Step | Action | Expected Result | Pass/Fail |
|------|--------|-----------------|-----------|
| 1 | Begin sunlit, SOC 100% | Mode = ACTIVE | |
| 2 | Enter 35-min eclipse | SOC decreasing | |
| 3 | Monitor mode changes | As SOC drops | |
| 4 | Verify beacon continues | Every 30s or 60s | |
| 5 | Exit eclipse | SOC recovering | |
| 6 | Verify return to ACTIVE | When SOC sufficient | |
| 7 | Verify AI beacons resume | AI text returns | |

**Pass Criteria**: Graceful eclipse handling

---

## 7. Coverage Report

### 7.1 Generating Coverage Report

```bash
# Build with coverage
cmake -S software/flight -B build -DENABLE_COVERAGE=ON
cmake --build build

# Run all tests
ctest --test-dir build

# Generate report
lcov --capture --directory build --output-file coverage.info
lcov --remove coverage.info '/usr/*' '*/tests/*' --output-file coverage.info
genhtml coverage.info --output-directory coverage_report

# View report
open coverage_report/index.html
```

### 7.2 Coverage Requirements

| Module | Target | Actual |
|--------|--------|--------|
| eps_control.c | 80% | TBD |
| fault_mgmt.c | 80% | TBD |
| sensors.c | 80% | TBD |
| mission_data.c | 80% | TBD |
| uart_comm.c | 80% | TBD |
| crc32.c | 80% | TBD |
| **Overall** | **80%** | **TBD** |

---

## 8. Test Data Recording

### 8.1 Test Log Template

```
=====================================
SMART-QSO Test Log
=====================================
Test ID: TP-XXX-XXX
Date: YYYY-MM-DD
Time: HH:MM:SS
Tester: Name
Build: git-hash

Configuration:
- Hardware: [description]
- Software Version: X.X.X
- Environment: [Lab/TVAC/etc]

Results:
Step | Expected | Actual | P/F
-----|----------|--------|----
1    |          |        |
2    |          |        |

Summary: PASS / FAIL
Notes:

Attachments:
- [list files]

Tester Signature: ________________
Date: ________________
=====================================
```

### 8.2 Defect Report Template

```
=====================================
SMART-QSO Defect Report
=====================================
Defect ID: DEF-XXXX
Date: YYYY-MM-DD
Reporter: Name
Severity: Critical/Major/Minor

Test Reference: TP-XXX-XXX

Description:
[Detailed description]

Steps to Reproduce:
1.
2.
3.

Expected Result:

Actual Result:

Root Cause Analysis:

Resolution:

Verification:

Status: Open/In Progress/Resolved/Closed
=====================================
```

---

## Appendix A: Test Equipment List

| Equipment | Model | Calibration Due |
|-----------|-------|-----------------|
| Multimeter | Fluke 87V | TBD |
| Power Supply | Keithley 2200 | TBD |
| Logic Analyzer | Saleae Logic Pro 16 | N/A |
| RF Receiver | SDRPlay RSP1A | N/A |
| Thermal Chamber | TBD | TBD |

---

## Appendix B: Software Versions

| Software | Version |
|----------|---------|
| Flight Software | 0.2.0-dev |
| Payload Software | TBD |
| Test Framework | CMocka 1.1.7 |
| Build System | CMake 3.16+ |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
