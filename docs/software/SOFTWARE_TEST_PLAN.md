# Software Test Plan (STP)

**Document Number**: SMART-QSO-STP-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

**Governing Standards**:
- NPR 7150.2 Rev B - NASA Software Engineering Requirements
- GSFC-STD-7000 (GEVS) - General Environmental Verification Standard

**Heritage**: NASA Ames Research Center Small Satellite Division

---

## 1. Introduction

### 1.1 Purpose

This Software Test Plan (STP) defines the test strategy, approach, and procedures for verifying the SMART-QSO flight software meets all requirements specified in the Software Requirements Specification (SRS).

### 1.2 Scope

This plan covers:
- Unit testing
- Integration testing
- System testing
- Acceptance testing
- Environmental testing support

### 1.3 Test Philosophy

Following NASA Ames Small Satellite heritage:
1. **Test Early, Test Often** - Continuous integration testing
2. **Protoflight Approach** - Single unit for qualification and flight
3. **Defense in Depth** - Multiple test levels catch different defects
4. **Automation First** - Maximize automated test coverage

---

## 2. Test Organization

### 2.1 Test Roles

| Role | Responsibility |
|------|----------------|
| Test Lead | Overall test strategy and coordination |
| Software Developer | Unit test development and execution |
| Integration Engineer | Integration test development |
| Systems Engineer | System test development |
| Quality Assurance | Test review and verification |

### 2.2 Test Schedule

| Phase | Activity | Entry Criteria | Exit Criteria |
|-------|----------|----------------|---------------|
| Phase B | Unit test framework | SRS approved | Framework operational |
| Phase C | Unit tests complete | CDR | 80% coverage |
| Phase C | Integration tests | Unit tests pass | All interfaces verified |
| Phase D | System tests | Integration complete | All requirements verified |
| Phase D | Acceptance tests | TRR | Mission scenarios pass |

---

## 3. Test Strategy

### 3.1 Test Levels

```
                    +------------------+
                    |   Acceptance     |  Mission scenarios
                    |   Testing        |  Day-in-the-life
                    +------------------+
                           ↑
                    +------------------+
                    |    System        |  End-to-end functions
                    |    Testing       |  Performance
                    +------------------+
                           ↑
                    +------------------+
                    |  Integration     |  Module interfaces
                    |    Testing       |  Data flow
                    +------------------+
                           ↑
                    +------------------+
                    |     Unit         |  Individual functions
                    |    Testing       |  80% coverage
                    +------------------+
```

### 3.2 Test Coverage Requirements

| Level | Coverage Type | Target | Tool |
|-------|---------------|--------|------|
| Unit | Statement | 80% | lcov |
| Unit | Branch | 75% | lcov |
| Integration | Interface | 100% | Manual |
| System | Requirements | 100% | RTM |

### 3.3 Test Environment

| Environment | Purpose | Configuration |
|-------------|---------|---------------|
| Development | Unit tests | Native x86 + mocks |
| Flatsat | Integration tests | Flight hardware + breakout |
| Flight Unit | System tests | Flight configuration |
| TVAC | Environmental | Thermal vacuum chamber |

---

## 4. Unit Testing

### 4.1 Framework

- **Framework**: CMocka (LGPL-compatible)
- **Build System**: CMake with CTest
- **Coverage**: lcov/gcov
- **Mocking**: CMocka mock functions

### 4.2 Unit Test Structure

```
software/flight/
├── tests/
│   ├── CMakeLists.txt
│   ├── test_main.c              # Test runner
│   ├── test_eps_control.c       # EPS module tests
│   ├── test_fault_mgmt.c        # Fault module tests
│   ├── test_sensors.c           # Sensor module tests
│   ├── test_mission_data.c      # Mission data tests
│   ├── test_uart_comm.c         # UART module tests
│   ├── test_crc32.c             # CRC32 tests
│   └── mocks/
│       ├── mock_hal.c           # HAL mocks
│       └── mock_hal.h
```

### 4.3 Unit Test Cases

#### 4.3.1 EPS Control Tests (test_eps_control.c)

| Test ID | Description | Requirement |
|---------|-------------|-------------|
| TC-EPS-U001 | Init with valid config | SRS-F011 |
| TC-EPS-U002 | Init with corrupted config | SRS-F062 |
| TC-EPS-U003 | Mode transition SAFE→IDLE | SRS-F014 |
| TC-EPS-U004 | Mode transition IDLE→ACTIVE | SRS-F015 |
| TC-EPS-U005 | Mode transition ACTIVE→SAFE | SRS-F013 |
| TC-EPS-U006 | Payload enable in sunlit | SRS-F015 |
| TC-EPS-U007 | Payload disable in eclipse | SRS-F015 |
| TC-EPS-U008 | Power budget check | SRS-F016 |
| TC-EPS-U009 | Config persistence | SRS-F017 |
| TC-EPS-U010 | SOC boundary conditions | SRS-F013, F014 |

#### 4.3.2 Fault Management Tests (test_fault_mgmt.c)

| Test ID | Description | Requirement |
|---------|-------------|-------------|
| TC-FLT-U001 | Init fault system | SRS-F040 |
| TC-FLT-U002 | Log single fault | SRS-F041 |
| TC-FLT-U003 | Log multiple faults | SRS-F041 |
| TC-FLT-U004 | Log overflow handling | SRS-F041 |
| TC-FLT-U005 | Critical fault handling | SRS-F043 |
| TC-FLT-U006 | Thermal recovery | SRS-F042 |
| TC-FLT-U007 | Power recovery | SRS-F044 |
| TC-FLT-U008 | Comm recovery | SRS-F042 |
| TC-FLT-U009 | Fault statistics | SRS-F045 |
| TC-FLT-U010 | CRC verification | SRS-F061 |

#### 4.3.3 Sensor Tests (test_sensors.c)

| Test ID | Description | Requirement |
|---------|-------------|-------------|
| TC-SEN-U001 | Init sensor framework | SRS-F070 |
| TC-SEN-U002 | Load YAML config | SRS-F071 |
| TC-SEN-U003 | Poll all sensors | SRS-F070 |
| TC-SEN-U004 | Validate in-range | SRS-F072 |
| TC-SEN-U005 | Detect out-of-range | SRS-F072 |
| TC-SEN-U006 | Format telemetry | SRS-F073 |
| TC-SEN-U007 | Configurable poll rate | SRS-F071 |
| TC-SEN-U008 | Sensor timeout | SRS-F072 |

#### 4.3.4 Mission Data Tests (test_mission_data.c)

| Test ID | Description | Requirement |
|---------|-------------|-------------|
| TC-DAT-U001 | Init new mission | SRS-F002 |
| TC-DAT-U002 | Load existing data | SRS-F002 |
| TC-DAT-U003 | Save mission data | SRS-F060 |
| TC-DAT-U004 | CRC verification | SRS-F061 |
| TC-DAT-U005 | Handle corruption | SRS-F062 |
| TC-DAT-U006 | Update statistics | SRS-F063 |
| TC-DAT-U007 | Reset counter | SRS-F004 |
| TC-DAT-U008 | Uptime tracking | SRS-F063 |

#### 4.3.5 UART Communication Tests (test_uart_comm.c)

| Test ID | Description | Requirement |
|---------|-------------|-------------|
| TC-UAR-U001 | Init UART | SRS-I001 |
| TC-UAR-U002 | Send message | SRS-F030 |
| TC-UAR-U003 | Receive message | SRS-F031 |
| TC-UAR-U004 | Timeout handling | SRS-F032 |
| TC-UAR-U005 | CRC-16 calculation | SRS-I005 |
| TC-UAR-U006 | Message parsing | SRS-I004 |
| TC-UAR-U007 | Retry logic | SRS-F032 |
| TC-UAR-U008 | Health check | SRS-F032 |

#### 4.3.6 CRC32 Tests (test_crc32.c)

| Test ID | Description | Requirement |
|---------|-------------|-------------|
| TC-CRC-U001 | Known vector test | SRS-F061 |
| TC-CRC-U002 | Empty data | SRS-F061 |
| TC-CRC-U003 | Large data | SRS-F061 |
| TC-CRC-U004 | Single byte changes | SRS-F061 |

### 4.4 Unit Test Execution

```bash
# Build with tests enabled
cmake -S software/flight -B build -DENABLE_COVERAGE=ON
cmake --build build

# Run all unit tests
ctest --test-dir build --output-on-failure

# Generate coverage report
lcov --capture --directory build --output-file coverage.info
genhtml coverage.info --output-directory coverage_report
```

---

## 5. Integration Testing

### 5.1 Integration Test Approach

Integration tests verify module interfaces and data flow:
1. Bottom-up integration from HAL to application
2. Interface-focused testing
3. Executed on flatsat with flight hardware

### 5.2 Integration Test Cases

| Test ID | Description | Modules | Requirement |
|---------|-------------|---------|-------------|
| TC-INT-001 | EPS + Sensors | eps_control, sensors | SRS-F010, F011 |
| TC-INT-002 | EPS + Fault | eps_control, fault_mgmt | SRS-F013, F043 |
| TC-INT-003 | UART + Beacon | uart_comm, main | SRS-F030, F031 |
| TC-INT-004 | Sensors + Beacon | sensors, main | SRS-F073 |
| TC-INT-005 | All + Mission Data | All modules | SRS-F060, F063 |
| TC-INT-006 | Fault + Recovery | fault_mgmt, eps_control | SRS-F042, F044 |
| TC-INT-007 | Power Mode Flow | eps_control, main | SRS-F012-F015 |
| TC-INT-008 | Watchdog Chain | main, fault_mgmt | SRS-F050, F051 |

### 5.3 Integration Test Procedures

#### TC-INT-001: EPS + Sensors Integration

**Purpose**: Verify EPS module correctly reads sensor data

**Preconditions**:
1. Flatsat powered and connected
2. Test software loaded

**Procedure**:
1. Initialize sensor framework
2. Initialize EPS control
3. Apply known voltage to battery monitor
4. Call eps_update_state()
5. Verify SOC value matches expected

**Expected Results**:
- SOC within ±2% of expected value
- Power mode transitions correctly

---

## 6. System Testing

### 6.1 System Test Approach

System tests verify end-to-end functionality and requirements:
1. Test complete system as integrated unit
2. Cover all functional requirements
3. Include performance verification
4. Execute on flight unit

### 6.2 System Test Cases

| Test ID | Description | Requirements | Pass Criteria |
|---------|-------------|--------------|---------------|
| TC-SYS-001 | System initialization | SRS-F001-F005 | Init < 30s |
| TC-SYS-002 | Power mode SAFE | SRS-F012, F013 | Correct behavior |
| TC-SYS-003 | Power mode IDLE | SRS-F012, F014 | Correct behavior |
| TC-SYS-004 | Power mode ACTIVE | SRS-F012, F015 | Correct behavior |
| TC-SYS-005 | Beacon transmission | SRS-F020-F025 | AX.25 valid |
| TC-SYS-006 | AI beacon generation | SRS-F023, F030-F034 | Valid text |
| TC-SYS-007 | Fallback beacon | SRS-F024 | Template used |
| TC-SYS-008 | Fault detection | SRS-F040-F045 | Faults logged |
| TC-SYS-009 | Watchdog operation | SRS-F050-F052 | Reset on timeout |
| TC-SYS-010 | Data persistence | SRS-F060-F063 | Data survives reset |
| TC-SYS-011 | Telemetry collection | SRS-F070-F073 | All sensors read |
| TC-SYS-012 | Main loop timing | SRS-P001 | < 20ms |
| TC-SYS-013 | Beacon timing | SRS-P002 | < 5s |
| TC-SYS-014 | UART timing | SRS-P005 | Configurable |

### 6.3 System Test Procedure Example

#### TC-SYS-006: AI Beacon Generation

**Document**: SMART-QSO-STP-001
**Test ID**: TC-SYS-006
**Revision**: A
**Date**: 2026-01-02

**Purpose**: Verify AI-generated beacon text is correctly received and transmitted

**Equipment**:
1. Flight unit or flatsat
2. Jetson payload (powered)
3. RF receiver/decoder
4. Test computer

**Preconditions**:
1. System in ACTIVE power mode
2. Jetson payload booted and ready
3. RF receiver configured for beacon frequency

**Procedure**:

| Step | Action | Expected Result | Actual | P/F |
|------|--------|-----------------|--------|-----|
| 1 | Power on system | Boot complete < 30s | | |
| 2 | Verify power mode ACTIVE | Mode indicator correct | | |
| 3 | Wait for Jetson ready | STATUS message received | | |
| 4 | Trigger beacon request | BEACON_REQ sent | | |
| 5 | Wait for response | BEACON_RSP < 10s | | |
| 6 | Verify beacon text | ASCII, length ≤ 256 | | |
| 7 | Verify CRC | CRC matches | | |
| 8 | Monitor RF beacon | Beacon transmitted | | |
| 9 | Decode beacon | AI text in info field | | |

**Pass Criteria**:
- AI-generated beacon text received within 10s
- Text is valid ASCII, ≤ 256 bytes
- CRC verification passes
- Beacon transmitted with AI text in info field

**Test Data**:
- Recorded BEACON_RSP message
- Decoded AX.25 frame
- Timing measurements

---

## 7. Acceptance Testing

### 7.1 Acceptance Test Approach

Acceptance tests demonstrate mission readiness:
1. Mission scenario based
2. Execute full operational sequences
3. Day-in-the-life testing
4. Stakeholder witness

### 7.2 Acceptance Test Cases

| Test ID | Description | Duration | Pass Criteria |
|---------|-------------|----------|---------------|
| TC-ACC-001 | Nominal sunlit pass | 15 min | All functions nominal |
| TC-ACC-002 | Eclipse transition | 30 min | Mode transitions correct |
| TC-ACC-003 | AI inference cycle | 1 hr | Multiple beacons generated |
| TC-ACC-004 | Fault injection | 2 hr | Recovery successful |
| TC-ACC-005 | Day-in-the-life | 24 hr | No anomalies |
| TC-ACC-006 | Extended operation | 72 hr | Stability demonstrated |

### 7.3 Day-in-the-Life Test Procedure

**Purpose**: Demonstrate 24-hour autonomous operation

**Configuration**:
- Flight unit in thermal chamber
- Simulated orbital environment (sunlit/eclipse cycles)
- Ground station monitoring

**Procedure**:
1. Start with fully charged battery
2. Begin eclipse cycle simulation (35 min sunlit, 30 min eclipse)
3. Monitor for 24 hours
4. Record all telemetry and beacons
5. Inject one fault at T+12 hours
6. Verify recovery

**Pass Criteria**:
- No unplanned resets
- All beacons transmitted on schedule
- Fault recovery successful
- Battery management nominal
- All telemetry within limits

---

## 8. Environmental Testing Support

### 8.1 Software Role in Environmental Tests

| Test | Software Function | Monitoring |
|------|-------------------|------------|
| Vibration | Functional check before/after | Reset detection |
| Thermal Vacuum | Continuous operation | Temperature telemetry |
| EMC/EMI | RF transmission | Beacon quality |

### 8.2 TVAC Test Support

**Pre-TVAC**:
1. Load flight software
2. Verify all functions
3. Baseline telemetry

**During TVAC**:
1. Monitor telemetry throughout
2. Execute functional tests at each temperature plateau
3. Record beacon quality

**Post-TVAC**:
1. Full functional test
2. Compare telemetry to baseline
3. Verify data persistence

---

## 9. Test Data Management

### 9.1 Test Artifacts

| Artifact | Format | Retention |
|----------|--------|-----------|
| Test procedures | Markdown/PDF | Permanent |
| Test results | JSON/CSV | Permanent |
| Coverage reports | HTML | Per release |
| Test logs | Text | 1 year |

### 9.2 Defect Tracking

All test failures tracked in GitHub Issues:
- Label: `test-failure`
- Include: Test ID, expected, actual, steps to reproduce
- Link to requirement

---

## 10. Test Completion Criteria

### 10.1 Unit Test Exit Criteria

- [ ] 80% statement coverage achieved
- [ ] 75% branch coverage achieved
- [ ] All unit tests passing
- [ ] No critical defects open

### 10.2 Integration Test Exit Criteria

- [ ] All interface tests passing
- [ ] All data flows verified
- [ ] Integration with flight hardware complete

### 10.3 System Test Exit Criteria

- [ ] 100% requirements verified
- [ ] All performance requirements met
- [ ] No critical or major defects open

### 10.4 Acceptance Test Exit Criteria

- [ ] All acceptance tests passing
- [ ] Day-in-the-life test complete
- [ ] Stakeholder sign-off obtained

---

## Appendix A: Test Tools

| Tool | Version | Purpose |
|------|---------|---------|
| CMocka | 1.1.7 | Unit test framework |
| CMake | 3.16+ | Build system |
| CTest | 3.16+ | Test runner |
| lcov | 1.16 | Coverage analysis |
| gcov | 12.x | Coverage data |
| Python | 3.10+ | Test automation |

---

## Appendix B: Test Report Template

```
SMART-QSO Software Test Report
==============================

Test ID: TC-XXX-YYY
Date: YYYY-MM-DD
Tester: Name
Build: Version

Configuration:
- Hardware: [Flight/Flatsat/Simulation]
- Software: [Version]
- Environment: [Lab/TVAC/etc]

Results:
| Step | Expected | Actual | P/F |
|------|----------|--------|-----|
| 1    |          |        |     |

Summary: [PASS/FAIL]
Notes:
Attachments:
```

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
