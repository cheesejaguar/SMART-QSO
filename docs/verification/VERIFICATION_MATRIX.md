# Verification Cross-Reference Matrix (VCRM)
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-VCRM-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7123.1 SE Process 13 (Verification)

---

## 1. Purpose

This Verification Cross-Reference Matrix provides traceability from requirements to verification methods and status for the SMART-QSO CubeSat mission.

## 2. Verification Methods

| Code | Method | Description |
|------|--------|-------------|
| A | Analysis | Mathematical analysis, simulation, modeling |
| D | Demonstration | Functional demonstration of capability |
| I | Inspection | Visual examination, measurement |
| T | Test | Formal test with documented procedure |

## 3. Mission Requirements Verification

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| MRD-001 | Spacecraft shall operate in LEO (400-500 km) | A | Orbital analysis | Complete |
| MRD-002 | Mission duration shall be ≥ 12 months | A | Power/thermal analysis | Complete |
| MRD-003 | Spacecraft shall transmit beacons receivable by amateur stations | T | TP-RF-001 | Complete |
| MRD-004 | Beacons shall include AI-generated content | T | TP-SYS-001 | Complete |
| MRD-005 | Spacecraft shall deorbit within 25 years | A | ODAR analysis | Complete |

## 4. System Requirements Verification

### 4.1 Electrical Power Subsystem

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SYS-EPS-001 | EPS shall provide 5.0W orbit average power | A | Power budget | Complete |
| SYS-EPS-002 | Battery capacity shall be ≥ 50 Wh | I, T | TP-EPS-003 | Pending |
| SYS-EPS-003 | MPPT efficiency shall be ≥ 95% | T | TP-EPS-001 | Pending |
| SYS-EPS-004 | EPS shall support three power modes | T | TP-SYS-001 | Complete |
| SYS-EPS-005 | Battery shall be protected from overcharge | T | TP-EPS-001 | Pending |
| SYS-EPS-006 | Battery shall be protected from overdischarge | T | TP-EPS-001 | Pending |
| SYS-EPS-007 | Battery shall be protected from overtemperature | T | TP-EPS-001 | Pending |

### 4.2 Command and Data Handling

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SYS-CDH-001 | OBC shall boot within 30 seconds | T | TP-SYS-001 | Complete |
| SYS-CDH-002 | OBC shall implement watchdog with ≤ 30s timeout | T | TP-OBC-004 | Pending |
| SYS-CDH-003 | OBC shall store mission data with CRC protection | T | Unit tests | Complete |
| SYS-CDH-004 | OBC shall interface with all subsystems | T | TP-SYS-001 | Complete |
| SYS-CDH-005 | OBC shall log all faults with timestamps | T | Unit tests | Complete |

### 4.3 Attitude Determination and Control

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SYS-ADCS-001 | ADCS shall detumble within 2 orbits | A, T | TP-ADCS-003 | Pending |
| SYS-ADCS-002 | ADCS shall maintain sun-pointing ± 10° | A, T | TP-ADCS-003 | Pending |
| SYS-ADCS-003 | Magnetometers shall measure ± 60 µT | I, T | TP-ADCS-001 | Pending |
| SYS-ADCS-004 | Magnetorquers shall provide 0.2 Am² dipole | T | TP-ADCS-002 | Pending |
| SYS-ADCS-005 | Sun sensors shall cover all 6 faces | I | Inspection | Pending |

### 4.4 Communication

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SYS-RF-001 | Transmitter output shall be 0.5-2W selectable | T | TP-RF-001 | Pending |
| SYS-RF-002 | Frequency shall be 145.825 MHz ± 1 kHz | T | TP-RF-001 | Pending |
| SYS-RF-003 | Modulation shall be AFSK 1200 bps | T | TP-RF-001 | Pending |
| SYS-RF-004 | Beacons shall use AX.25 protocol | T | TP-RF-001 | Complete |
| SYS-RF-005 | Beacon interval shall be configurable 30-120s | T | TP-SYS-001 | Complete |
| SYS-RF-006 | Spurious emissions shall be < -43 dBc | T | TP-RF-003 | Pending |
| SYS-RF-007 | Antenna shall deploy after 30 minute inhibit | T | TP-RF-005 | Pending |

### 4.5 AI Payload

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SYS-PAY-001 | Jetson shall operate at ≤ 10W average | T | TP-JETSON-001 | Pending |
| SYS-PAY-002 | Jetson shall generate beacon text within 5s | T | TP-JETSON-003 | Pending |
| SYS-PAY-003 | Jetson shall communicate via UART at 115200 bps | T | TP-JETSON-003 | Pending |
| SYS-PAY-004 | System shall fallback to templates if Jetson unavailable | T | TP-SYS-001 | Complete |
| SYS-PAY-005 | Jetson temperature shall not exceed 70°C | T | TP-JETSON-002 | Pending |

### 4.6 Mechanical/Thermal

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SYS-MECH-001 | Spacecraft mass shall be ≤ 2.0 kg | I | Measurement | Pending |
| SYS-MECH-002 | Spacecraft shall fit 1U envelope | I | Fit check | Pending |
| SYS-MECH-003 | CG shall be within 2 cm of geometric center | I | Measurement | Pending |
| SYS-MECH-004 | Spacecraft shall survive launch vibration | T | TP-ENV-002 | Pending |
| SYS-THERM-001 | Spacecraft shall operate -20°C to +50°C | T | TP-ENV-003 | Pending |
| SYS-THERM-002 | Spacecraft shall survive -30°C to +60°C | T | TP-ENV-003 | Pending |

## 5. Software Requirements Verification

### 5.1 Functional Requirements

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SRS-001 | Software shall implement EPS power mode control | T | test_eps_control | Complete |
| SRS-002 | Software shall implement fault detection and logging | T | test_fault_mgmt | Complete |
| SRS-003 | Software shall implement sensor data collection | T | test_sensors | Complete |
| SRS-004 | Software shall implement AX.25 beacon generation | T | Unit tests | Complete |
| SRS-005 | Software shall implement ADCS control algorithms | T | Unit tests | Complete |
| SRS-006 | Software shall implement UART communication | T | test_uart_comm | Complete |
| SRS-007 | Software shall implement mission data persistence | T | test_mission_data | Complete |
| SRS-008 | Software shall implement watchdog servicing | T | TP-OBC-004 | Pending |

### 5.2 Performance Requirements

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SRS-101 | Main loop shall execute at ≥ 1 Hz | T | Timing test | Complete |
| SRS-102 | Beacon generation shall complete in < 100 ms | T | Timing test | Complete |
| SRS-103 | Fault detection shall respond in < 1 second | T | Fault injection | Pending |
| SRS-104 | State transitions shall complete in < 100 ms | T | State test | Complete |

### 5.3 Safety Requirements

| Req ID | Requirement | Method | Procedure | Status |
|--------|-------------|--------|-----------|--------|
| SRS-201 | Software shall enter safe mode on critical faults | T | TP-SYS-001 | Complete |
| SRS-202 | Software shall protect data with CRC32 | T | test_crc32 | Complete |
| SRS-203 | Software shall validate all input parameters | A, T | Code review | Complete |
| SRS-204 | Software shall limit actuator commands | A, T | Code review | Complete |

## 6. Verification Status Summary

### 6.1 By Requirement Category

| Category | Total | Complete | Pending | N/A |
|----------|-------|----------|---------|-----|
| Mission | 5 | 5 | 0 | 0 |
| System - EPS | 7 | 2 | 5 | 0 |
| System - CDH | 5 | 4 | 1 | 0 |
| System - ADCS | 5 | 0 | 5 | 0 |
| System - RF | 7 | 2 | 5 | 0 |
| System - Payload | 5 | 1 | 4 | 0 |
| System - Mech/Therm | 6 | 0 | 6 | 0 |
| Software - Functional | 8 | 7 | 1 | 0 |
| Software - Performance | 4 | 3 | 1 | 0 |
| Software - Safety | 4 | 4 | 0 | 0 |
| **Total** | **56** | **28** | **28** | **0** |

### 6.2 Overall Progress

```
Verification Progress: ████████████░░░░░░░░ 50%

Complete: 28 requirements
Pending:  28 requirements
Total:    56 requirements
```

### 6.3 By Verification Method

| Method | Requirements | Complete |
|--------|--------------|----------|
| Analysis | 8 | 8 (100%) |
| Demonstration | 0 | 0 |
| Inspection | 6 | 0 (0%) |
| Test | 48 | 22 (46%) |

## 7. Verification Closure Plan

### 7.1 Remaining Activities

| Phase | Requirements | Schedule |
|-------|--------------|----------|
| Unit testing | SRS-008 | Pre-integration |
| Subsystem test | EPS, RF, ADCS | Integration |
| System test | All system req | Post-integration |
| Environmental test | Mech/Therm | Environmental campaign |

### 7.2 Closure Schedule

| Milestone | Target | Requirements Closed |
|-----------|--------|---------------------|
| PDR | Complete | 50% (28/56) |
| CDR | TBD | 75% (42/56) |
| TRR | TBD | 95% (53/56) |
| FRR | TBD | 100% (56/56) |

## 8. Non-Compliance Tracking

| Req ID | Non-Compliance | Disposition | Status |
|--------|----------------|-------------|--------|
| None | - | - | - |

## 9. Verification Evidence

All verification evidence shall be maintained in the project archive:
- Test reports
- Analysis reports
- Inspection records
- Data files
- Photographs

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
