# Hardware-in-the-Loop (HIL) Test Plan

**Document ID**: SMART-QSO-HIL-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Engineering

## 1. Introduction

### 1.1 Purpose

This document defines the Hardware-in-the-Loop (HIL) testing approach for SMART-QSO flight software verification. HIL testing bridges the gap between unit/integration testing and full system testing by executing flight software on target-representative hardware.

### 1.2 Scope

HIL testing covers:
- OBC flight software execution on STM32L4/Apollo4 target
- Simulated subsystem interfaces (EPS, ADCS, RF, Payload)
- Real-time execution with hardware timers and watchdogs
- Communication protocol validation

### 1.3 References

- NPR 7150.2 - NASA Software Engineering Requirements
- NASA-STD-8739.8 - Software Assurance Standard
- SMART-QSO-STP-001 - Software Test Plan
- SMART-QSO-SDD-001 - Software Design Document

## 2. HIL Test Architecture

### 2.1 System Configuration

```
+------------------------------------------------------------------+
|                    HIL Test Configuration                         |
|                                                                    |
|  +------------------+         +------------------+                 |
|  |  HOST COMPUTER   |         |  OBC TARGET      |                 |
|  |                  |  UART   |  (STM32L4)       |                 |
|  |  Test Executive  |<------->|  Flight SW       |                 |
|  |  Subsystem Sims  |         |  Real HAL        |                 |
|  |  Data Logging    |         |  HW Watchdog     |                 |
|  +------------------+         +--------+---------+                 |
|                                        |                           |
|                               +--------+---------+                 |
|                               |  BREAKOUT BOARD  |                 |
|                               |  GPIO Monitoring |                 |
|                               |  I2C Simulation  |                 |
|                               |  SPI Simulation  |                 |
|                               +------------------+                 |
+------------------------------------------------------------------+
```

### 2.2 Hardware Components

| Component | Description | Purpose |
|-----------|-------------|---------|
| OBC Dev Board | STM32L476 Nucleo or Apollo4 EVK | Target execution |
| USB-UART Adapter | Debug/command interface | Host communication |
| Logic Analyzer | 8+ channel digital analyzer | Protocol verification |
| Breakout Board | GPIO/I2C/SPI breakout | Interface testing |
| Power Supply | Programmable 3.3V/5V supply | Power testing |

### 2.3 Software Components

| Component | Description | Location |
|-----------|-------------|----------|
| Test Executive | Python test framework | `tests/hil/hil_executive.py` |
| EPS Simulator | Power subsystem simulation | `tests/hil/sim_eps.py` |
| ADCS Simulator | Attitude simulation | `tests/hil/sim_adcs.py` |
| RF Simulator | Beacon/telemetry simulation | `tests/hil/sim_rf.py` |
| Jetson Simulator | Payload response simulation | `tests/hil/sim_jetson.py` |

## 3. HIL Test Cases

### 3.1 Startup Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-001 | Cold Start | Power-on initialization | System reaches SAFE mode within 5s |
| HIL-002 | Warm Start | Reset after fault | State restored from NVM |
| HIL-003 | Watchdog Timeout | No SW activity | Hardware reset occurs |
| HIL-004 | Stack Overflow | Stack guard violation | Fault logged, safe mode entered |

### 3.2 EPS Interface Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-010 | I2C Read | Read EPS telemetry | Correct values within 100ms |
| HIL-011 | I2C Write | Set power mode | Mode confirmed within 50ms |
| HIL-012 | I2C Timeout | No EPS response | Timeout fault logged |
| HIL-013 | Battery Monitor | Low SOC simulation | Power mode transitions |

### 3.3 ADCS Interface Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-020 | Magnetometer Read | Read mag sensor | Data within valid range |
| HIL-021 | Gyro Read | Read rate gyro | Data within valid range |
| HIL-022 | Actuator Command | Set MTQ polarity | GPIO state correct |
| HIL-023 | Detumble Algorithm | Rate reduction | Rates decrease over time |

### 3.4 Communication Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-030 | UART TX | Beacon transmission | Frame correct, timing met |
| HIL-031 | UART RX | Command reception | Command executed correctly |
| HIL-032 | Jetson Protocol | Payload request/response | Response within timeout |
| HIL-033 | Fallback Mode | Jetson timeout | Fallback beacon used |

### 3.5 Timing Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-040 | Main Loop Timing | Loop iteration time | < 100ms per iteration |
| HIL-041 | Beacon Interval | Periodic beacon | Within ±1s of target |
| HIL-042 | Telemetry Rate | Telemetry generation | 1Hz ± 10% |
| HIL-043 | Watchdog Kick | WDT service interval | < 1s between kicks |

### 3.6 Fault Injection Tests

| ID | Test | Description | Pass Criteria |
|----|------|-------------|---------------|
| HIL-050 | Sensor Fault | Invalid sensor data | Fault logged, safe values used |
| HIL-051 | Comm Fault | UART disconnect | Timeout handled, no crash |
| HIL-052 | Power Fault | Low voltage | Emergency mode entered |
| HIL-053 | Memory Fault | CRC mismatch | Data rejected, default used |

## 4. Test Procedures

### 4.1 Test Environment Setup

1. Connect OBC target to host via USB-UART
2. Connect breakout board for GPIO/I2C monitoring
3. Configure logic analyzer for protocol capture
4. Start test executive software
5. Verify communication with target

### 4.2 Test Execution

```bash
# Run all HIL tests
cd software/flight/tests/hil
python hil_executive.py --port /dev/ttyUSB0 --all

# Run specific test suite
python hil_executive.py --port /dev/ttyUSB0 --suite startup

# Run single test
python hil_executive.py --port /dev/ttyUSB0 --test HIL-001
```

### 4.3 Data Collection

| Data Type | Collection Method | Storage |
|-----------|-------------------|---------|
| Test results | JSON/XML report | `results/hil_results.json` |
| Protocol captures | Logic analyzer export | `results/captures/` |
| Console logs | Serial log capture | `results/logs/` |
| Timing data | Timestamp analysis | `results/timing/` |

## 5. Pass/Fail Criteria

### 5.1 Individual Test

- Test passes if all assertions satisfied
- Test passes if no unexpected faults occur
- Test passes if timing requirements met

### 5.2 Test Suite

- Suite passes if all tests pass
- Suite passes with notes if <5% tests require rerun

### 5.3 HIL Campaign

- Campaign passes if all suites pass
- All safety-critical tests must pass
- Known issues must be documented

## 6. Test Schedule

| Phase | Tests | Duration |
|-------|-------|----------|
| Setup Verification | HIL-001 to HIL-004 | 1 day |
| Interface Tests | HIL-010 to HIL-033 | 2 days |
| Timing Tests | HIL-040 to HIL-043 | 1 day |
| Fault Injection | HIL-050 to HIL-053 | 2 days |
| Regression | All tests | 1 day |

## 7. Resources Required

### 7.1 Hardware

- 1x STM32L476 Nucleo-64 board
- 1x FTDI USB-UART adapter
- 1x Saleae Logic 8 analyzer
- 1x Custom breakout board
- 1x Programmable power supply

### 7.2 Software

- Python 3.8+ with pyserial, pytest
- STM32CubeProgrammer
- Logic 2 analyzer software
- Serial terminal (minicom/screen)

### 7.3 Personnel

- 1x Software test engineer
- 1x Embedded software engineer (support)

## 8. Risk Management

| Risk | Mitigation |
|------|------------|
| Hardware unavailable | Use simulation mode fallback |
| Protocol timing issues | Add configurable delays |
| Flaky tests | Implement retry logic with limits |
| Environment differences | Document and control test environment |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |

---

*This document follows NASA Ames SmallSat HIL testing heritage patterns.*
