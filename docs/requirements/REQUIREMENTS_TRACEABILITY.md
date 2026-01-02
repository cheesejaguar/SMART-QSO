# Requirements Traceability Matrix (RTM)

**Document Number**: SMART-QSO-RTM-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Preliminary

**Governing Standards**:
- NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements
- NPR 7150.2 Rev B - NASA Software Engineering Requirements

---

## 1. Introduction

### 1.1 Purpose

This Requirements Traceability Matrix (RTM) provides bidirectional traceability between system requirements, software requirements, design elements, code implementation, and verification activities.

### 1.2 Scope

This document covers traceability for:
- Mission-level requirements
- System requirements
- Software requirements (OBC and Payload)
- Interface requirements

### 1.3 Traceability Structure

```
Mission Objectives (MO-xxx)
         │
         ▼
System Requirements (SYS-xxx)
         │
         ▼
Software Requirements (SRS-xxx)
         │
         ├──────────────────────────┐
         ▼                          ▼
Design Elements (DES-xxx)    Interface Requirements (ICD-xxx)
         │
         ▼
Code Implementation (file:line)
         │
         ▼
Test Cases (TC-xxx)
```

---

## 2. Mission Objectives Traceability

| ID | Mission Objective | Parent | Child Requirements |
|----|-------------------|--------|-------------------|
| MO-001 | Demonstrate onboard AI for beacon generation | - | SYS-040, SYS-041, SYS-042 |
| MO-002 | Validate power-constrained AI operations | - | SYS-020, SYS-021, SYS-022 |
| MO-003 | Support amateur radio community with open data | - | SYS-030, SYS-031 |
| MO-004 | Characterize TinyML/TinyLM in orbital environment | - | SYS-050, SYS-051 |

---

## 3. System to Software Requirements Traceability

### 3.1 Power Management

| System Req | Description | Software Req | Design Element | Implementation | Test |
|------------|-------------|--------------|----------------|----------------|------|
| SYS-020 | Power mode management | SRS-F012, SRS-F013, SRS-F014 | DES-EPS-001 | eps_control.c:eps_set_power_mode() | TC-EPS-001 |
| SYS-021 | SOC monitoring | SRS-F010, SRS-F011 | DES-EPS-002 | eps_control.c:eps_update_state() | TC-EPS-002 |
| SYS-022 | Payload power control | SRS-F015, SRS-F016 | DES-EPS-003 | eps_control.c:eps_control_payload() | TC-EPS-003 |

### 3.2 Beacon Operations

| System Req | Description | Software Req | Design Element | Implementation | Test |
|------------|-------------|--------------|----------------|----------------|------|
| SYS-030 | Beacon transmission | SRS-F020, SRS-F021, SRS-F022 | DES-BCN-001 | main.c:transmit_beacon() | TC-BCN-001 |
| SYS-031 | AX.25 compliance | SRS-I021, SRS-I022 | DES-BCN-002 | main.c:format_ax25_frame() | TC-BCN-002 |
| SYS-032 | Fallback beacon | SRS-F024 | DES-BCN-003 | main.c:get_fallback_beacon() | TC-BCN-003 |

### 3.3 AI Payload

| System Req | Description | Software Req | Design Element | Implementation | Test |
|------------|-------------|--------------|----------------|----------------|------|
| SYS-040 | Jetson communication | SRS-F030, SRS-F031 | DES-PAY-001 | uart_comm.c:uart_send() | TC-PAY-001 |
| SYS-041 | Beacon text generation | SRS-F023 | DES-PAY-002 | main.c:request_beacon_text() | TC-PAY-002 |
| SYS-042 | Payload timeout | SRS-F032, SRS-F033 | DES-PAY-003 | uart_comm.c:uart_receive() | TC-PAY-003 |

### 3.4 Fault Management

| System Req | Description | Software Req | Design Element | Implementation | Test |
|------------|-------------|--------------|----------------|----------------|------|
| SYS-060 | Fault detection | SRS-F040, SRS-F041 | DES-FLT-001 | fault_mgmt.c:fault_log_add() | TC-FLT-001 |
| SYS-061 | Autonomous recovery | SRS-F042, SRS-F043 | DES-FLT-002 | fault_mgmt.c:fault_recover_*() | TC-FLT-002 |
| SYS-062 | Watchdog | SRS-F050, SRS-F051 | DES-FLT-003 | main.c:kick_watchdog() | TC-FLT-003 |

### 3.5 Data Management

| System Req | Description | Software Req | Design Element | Implementation | Test |
|------------|-------------|--------------|----------------|----------------|------|
| SYS-070 | Data persistence | SRS-F060, SRS-F061 | DES-DAT-001 | mission_data.c:mission_data_save() | TC-DAT-001 |
| SYS-071 | Data integrity | SRS-F062 | DES-DAT-002 | crc32.c:crc32() | TC-DAT-002 |
| SYS-072 | Telemetry collection | SRS-F070, SRS-F071 | DES-DAT-003 | sensors.c:sensors_poll() | TC-DAT-003 |

---

## 4. Interface Requirements Traceability

| Interface | ICD Section | Software Req | Design Element | Implementation | Test |
|-----------|-------------|--------------|----------------|----------------|------|
| OBC-Jetson UART | ICD-002 §5 | SRS-I001, SRS-I002 | DES-IFC-001 | uart_comm.c | TC-IFC-001 |
| OBC-EPS I2C | ICD-001 §3 | SRS-I010, SRS-I011 | DES-IFC-002 | eps_control.c | TC-IFC-002 |
| OBC-RF SPI | ICD-004 §4 | SRS-I020, SRS-I021 | DES-IFC-003 | main.c | TC-IFC-003 |

---

## 5. Software Requirements to Code Mapping

### 5.1 Functional Requirements (SRS-Fxxx)

| Req ID | Description | Source File | Function/Line | Status |
|--------|-------------|-------------|---------------|--------|
| SRS-F001 | Initialize within 30s | main.c | init_system():45 | Implemented |
| SRS-F002 | Load config from NVM | mission_data.c | mission_data_init():78 | Implemented |
| SRS-F003 | Self-test on boot | main.c | perform_self_test():112 | Implemented |
| SRS-F004 | Increment reset counter | mission_data.c | mission_data_init():85 | Implemented |
| SRS-F005 | Log boot reason | fault_mgmt.c | fault_log_add():45 | Implemented |
| SRS-F010 | Monitor battery SOC | eps_control.c | eps_update_state():67 | Implemented |
| SRS-F011 | Control load switches | eps_control.c | eps_control_load():134 | Implemented |
| SRS-F012 | Power mode states | eps_control.c | eps_set_power_mode():89 | Implemented |
| SRS-F013 | SAFE mode at SOC<25% | eps_control.c | eps_set_power_mode():102 | Implemented |
| SRS-F014 | IDLE mode at SOC<40% | eps_control.c | eps_set_power_mode():108 | Implemented |
| SRS-F015 | Payload enable rules | eps_control.c | eps_control_payload():156 | Implemented |
| SRS-F016 | Power budget limits | eps_control.c | eps_check_budget():178 | Implemented |
| SRS-F017 | Persist EPS state | eps_control.c | eps_save_config():201 | Implemented |
| SRS-F020 | AX.25 beacon TX | main.c | transmit_beacon():234 | Implemented |
| SRS-F021 | Callsign in beacon | main.c | format_ax25_frame():267 | Implemented |
| SRS-F022 | Status in beacon | main.c | build_beacon_info():289 | Implemented |
| SRS-F023 | AI-generated text | main.c | request_beacon_text():312 | Implemented |
| SRS-F024 | Fallback beacon | main.c | get_fallback_beacon():334 | Implemented |
| SRS-F025 | 256 byte info limit | main.c | build_beacon_info():291 | Implemented |
| SRS-F030 | Telemetry to Jetson | uart_comm.c | uart_send():56 | Implemented |
| SRS-F031 | Receive from Jetson | uart_comm.c | uart_receive():89 | Implemented |
| SRS-F032 | Communication timeout | uart_comm.c | uart_receive():95 | Implemented |
| SRS-F033 | Disable on timeout | main.c | handle_payload_timeout():356 | Implemented |
| SRS-F034 | Validate beacon text | main.c | validate_beacon_text():378 | Implemented |
| SRS-F035 | Monitor Jetson power | eps_control.c | eps_monitor_payload():223 | Partial |
| SRS-F040 | Detect faults | fault_mgmt.c | fault_check():45 | Implemented |
| SRS-F041 | NVM fault log | fault_mgmt.c | fault_log_add():78 | Implemented |
| SRS-F042 | Autonomous recovery | fault_mgmt.c | fault_recover_*():112 | Implemented |
| SRS-F043 | SAFE mode on critical | fault_mgmt.c | fault_handle_critical():156 | Implemented |
| SRS-F044 | Recovery from SAFE | eps_control.c | eps_attempt_recovery():245 | Partial |
| SRS-F045 | Fault statistics | fault_mgmt.c | fault_get_stats():189 | Implemented |
| SRS-F050 | Watchdog kick | main.c | kick_watchdog():45 | Implemented |
| SRS-F051 | Save before reset | main.c | prepare_for_reset():67 | Implemented |
| SRS-F052 | Detect WDT reset | main.c | detect_reset_cause():89 | Implemented |
| SRS-F060 | Persist mission data | mission_data.c | mission_data_save():56 | Implemented |
| SRS-F061 | CRC32 protection | crc32.c | crc32():23 | Implemented |
| SRS-F062 | Handle corruption | mission_data.c | mission_data_verify():89 | Implemented |
| SRS-F063 | Mission statistics | mission_data.c | mission_data_update():112 | Implemented |
| SRS-F070 | Collect telemetry | sensors.c | sensors_poll():67 | Implemented |
| SRS-F071 | Configurable rates | sensors.c | sensors_load_yaml():34 | Implemented |
| SRS-F072 | Validate readings | sensors.c | sensors_validate():123 | Implemented |
| SRS-F073 | Telemetry in beacon | main.c | build_beacon_info():289 | Implemented |

### 5.2 Performance Requirements (SRS-Pxxx)

| Req ID | Description | Verification Method | Status |
|--------|-------------|---------------------|--------|
| SRS-P001 | Main loop < 20ms | Test | Pending |
| SRS-P002 | Beacon TX < 5s | Test | Pending |
| SRS-P003 | Fault detect < 100ms | Test | Pending |
| SRS-P004 | Mode transition < 1s | Test | Pending |
| SRS-P005 | UART timeout config | Test | Verified |
| SRS-P006 | Sensor period ±10% | Test | Pending |
| SRS-P007 | Persist < 500ms | Test | Pending |
| SRS-P008 | Init < 30s | Test | Pending |

### 5.3 Interface Requirements (SRS-Ixxx)

| Req ID | Description | ICD Reference | Status |
|--------|-------------|---------------|--------|
| SRS-I001 | 115200 baud 8N1 | ICD-002 §2.3 | Implemented |
| SRS-I002 | 3.3V LVTTL | ICD-002 §2.2 | Design |
| SRS-I003 | CSV telemetry | ICD-002 §4.3 | Implemented |
| SRS-I004 | ASCII newline | ICD-002 §4.3 | Implemented |
| SRS-I005 | CRC integrity | ICD-002 §4.4 | Implemented |
| SRS-I010 | EPS I2C interface | ICD-001 §3 | Design |
| SRS-I011 | Battery readings | ICD-001 §4 | Design |
| SRS-I012 | Solar readings | ICD-001 §4 | Design |
| SRS-I020 | RF GPIO control | ICD-004 §3 | Design |
| SRS-I021 | AX.25 UI frames | ICD-004 §5 | Implemented |
| SRS-I022 | AFSK 1200 bps | ICD-004 §5 | Design |

### 5.4 Quality Requirements (SRS-Qxxx)

| Req ID | Description | Verification Method | Status |
|--------|-------------|---------------------|--------|
| SRS-Q001 | 1 year operation | Analysis | Pending |
| SRS-Q002 | SEU recovery | Test | Pending |
| SRS-Q003 | MTBF > 30 days | Analysis | Pending |
| SRS-Q010 | Modular design | Inspection | Verified |
| SRS-Q011 | 80% code coverage | Test | Pending |
| SRS-Q012 | Doxygen comments | Inspection | Partial |
| SRS-Q020 | ARM/x86 portable | Test | Verified |
| SRS-Q021 | HAL isolation | Inspection | Partial |
| SRS-Q022 | Sim/flight builds | Test | Verified |

### 5.5 Safety Requirements (SRS-Sxxx)

| Req ID | Description | Verification Method | Status |
|--------|-------------|---------------------|--------|
| SRS-S001 | RF after separation | Test | Design |
| SRS-S002 | SAFE mode loads | Test | Implemented |
| SRS-S003 | Hardware watchdog | Test | Design |
| SRS-S004 | Command validation | Test | Partial |
| SRS-S005 | Thermal protection | Test | Implemented |

---

## 6. Test Case Traceability

### 6.1 Unit Tests

| Test ID | Description | Requirements Verified | Status |
|---------|-------------|----------------------|--------|
| TC-EPS-001 | Power mode transitions | SRS-F012, SRS-F013, SRS-F014 | Pending |
| TC-EPS-002 | SOC monitoring accuracy | SRS-F010 | Pending |
| TC-EPS-003 | Payload enable logic | SRS-F015, SRS-F016 | Pending |
| TC-BCN-001 | Beacon formatting | SRS-F020, SRS-F022 | Pending |
| TC-BCN-002 | AX.25 frame structure | SRS-I021 | Pending |
| TC-BCN-003 | Fallback beacon | SRS-F024 | Pending |
| TC-PAY-001 | UART communication | SRS-F030, SRS-F031 | Pending |
| TC-PAY-002 | Beacon text validation | SRS-F034 | Pending |
| TC-PAY-003 | Timeout handling | SRS-F032, SRS-F033 | Pending |
| TC-FLT-001 | Fault logging | SRS-F040, SRS-F041 | Pending |
| TC-FLT-002 | Recovery procedures | SRS-F042, SRS-F043 | Pending |
| TC-FLT-003 | Watchdog operation | SRS-F050, SRS-F051 | Pending |
| TC-DAT-001 | Data persistence | SRS-F060 | Pending |
| TC-DAT-002 | CRC verification | SRS-F061, SRS-F062 | Pending |
| TC-DAT-003 | Telemetry collection | SRS-F070, SRS-F071 | Pending |

### 6.2 Integration Tests

| Test ID | Description | Requirements Verified | Status |
|---------|-------------|----------------------|--------|
| TC-INT-001 | End-to-end beacon | SRS-F020, SRS-F023 | Pending |
| TC-INT-002 | Power mode operations | SRS-F012, SRS-F015 | Pending |
| TC-INT-003 | Fault recovery sequence | SRS-F042, SRS-F043 | Pending |
| TC-INT-004 | Data across resets | SRS-F017, SRS-F063 | Pending |

### 6.3 System Tests

| Test ID | Description | Requirements Verified | Status |
|---------|-------------|----------------------|--------|
| TC-SYS-001 | Mission scenario nominal | MO-001, MO-002 | Pending |
| TC-SYS-002 | Eclipse operations | SYS-020, SYS-022 | Pending |
| TC-SYS-003 | Communication loss | SYS-042, SRS-F033 | Pending |
| TC-SYS-004 | Day-in-the-life | All | Pending |

---

## 7. Verification Status Summary

### 7.1 Requirements Coverage

| Category | Total | Verified | Partial | Pending |
|----------|-------|----------|---------|---------|
| Functional (F) | 45 | 38 | 4 | 3 |
| Performance (P) | 8 | 1 | 0 | 7 |
| Interface (I) | 12 | 5 | 0 | 7 |
| Quality (Q) | 9 | 3 | 2 | 4 |
| Safety (S) | 5 | 2 | 1 | 2 |
| **Total** | **79** | **49** | **7** | **23** |

### 7.2 Test Coverage

| Level | Total Tests | Passed | Failed | Pending |
|-------|-------------|--------|--------|---------|
| Unit | 15 | 0 | 0 | 15 |
| Integration | 4 | 0 | 0 | 4 |
| System | 4 | 0 | 0 | 4 |
| **Total** | **23** | **0** | **0** | **23** |

---

## 8. TBD/TBR Items

| ID | Item | Resolution Target | Owner |
|----|------|-------------------|-------|
| TBD-RTM-001 | Final line numbers after code freeze | CDR | SW Lead |
| TBD-RTM-002 | Test case procedures | TRR | Test Lead |
| TBR-RTM-001 | Performance test thresholds | PDR | SE Lead |

---

## Appendix A: Requirements Change Log

| Date | Requirement | Change | Rationale |
|------|-------------|--------|-----------|
| 2026-01-02 | Initial | Baseline created | PDR preparation |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
