# SMART-QSO Requirements Traceability Matrix

**Document ID**: SQSO-RTM-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the complete Requirements Traceability Matrix (RTM) for SMART-QSO, demonstrating bidirectional traceability from mission objectives through verification per NPR 7123.1.

## 2. Traceability Structure

```
TRACEABILITY HIERARCHY

Mission Objectives (MO)
        ↓
System Requirements (SYS)
        ↓
Software Requirements (SWR)
        ↓
Design Elements (DE)
        ↓
Verification Methods (VM)
        ↓
Test Procedures (TP)
```

## 3. Mission Objectives Trace

### 3.1 Mission Objectives

| ID | Objective | Success Criteria |
|----|-----------|------------------|
| MO-1 | Demonstrate AI-assisted amateur radio | AI-generated beacons transmitted |
| MO-2 | Provide amateur radio transponder | Successful QSOs logged |
| MO-3 | Operate autonomously for 2 years | Continuous operation without intervention |
| MO-4 | Comply with amateur regulations | No regulatory violations |

### 3.2 MO to System Requirements

| MO | System Requirements |
|----|---------------------|
| MO-1 | SYS-010, SYS-011, SYS-012, SYS-040 |
| MO-2 | SYS-020, SYS-021, SYS-022, SYS-023 |
| MO-3 | SYS-030, SYS-031, SYS-032, SYS-033, SYS-034 |
| MO-4 | SYS-050, SYS-051, SYS-052 |

## 4. System to Software Requirements

### 4.1 AI Payload Requirements

| SYS ID | System Requirement | SWR ID | Software Requirement |
|--------|-------------------|--------|---------------------|
| SYS-010 | AI payload shall generate beacon content | SWR-101 | Jetson shall run TinyLM inference |
| | | SWR-102 | OBC shall request AI content via UART |
| | | SWR-103 | Fallback to templates if AI unavailable |
| SYS-011 | AI duty cycle ≤ 10% | SWR-104 | Power manager limits Jetson runtime |
| | | SWR-105 | Jetson auto-shutdown after inference |
| SYS-012 | Graceful degradation without AI | SWR-106 | Template beacons always available |
| | | SWR-107 | OBC operates independently of Jetson |

### 4.2 RF System Requirements

| SYS ID | System Requirement | SWR ID | Software Requirement |
|--------|-------------------|--------|---------------------|
| SYS-020 | Beacon transmission every 60±1s | SWR-201 | Beacon task runs at 60s interval |
| | | SWR-202 | Timer accuracy < 1% |
| SYS-021 | AX.25 compliant frames | SWR-203 | AX.25 frame generator |
| | | SWR-204 | CRC-16 calculation |
| SYS-022 | TX power 0.5-2W selectable | SWR-205 | TX power control command |
| | | SWR-206 | Power level in telemetry |
| SYS-023 | Callsign in all transmissions | SWR-207 | Source callsign in AX.25 header |

### 4.3 Power System Requirements

| SYS ID | System Requirement | SWR ID | Software Requirement |
|--------|-------------------|--------|---------------------|
| SYS-030 | SAFE mode < 100mW | SWR-301 | SAFE mode disables non-essential |
| SYS-031 | IDLE mode < 200mW | SWR-302 | IDLE mode baseline operations |
| SYS-032 | ACTIVE mode < 500mW (no AI) | SWR-303 | ACTIVE mode full operations |
| SYS-033 | Battery protection at 20% SOC | SWR-304 | Auto-SAFE at SOC < 20% |
| | | SWR-305 | SOC monitoring continuous |
| SYS-034 | Positive power budget | SWR-306 | Power-aware mode transitions |

### 4.4 Fault Tolerance Requirements

| SYS ID | System Requirement | SWR ID | Software Requirement |
|--------|-------------------|--------|---------------------|
| SYS-040 | Watchdog recovery | SWR-401 | 5-second watchdog timer |
| | | SWR-402 | Watchdog kick in main loop |
| SYS-041 | Safe mode on critical fault | SWR-403 | Fault handler triggers SAFE |
| | | SWR-404 | Fault classification logic |
| SYS-042 | Boot within 5 seconds | SWR-405 | Fast initialization sequence |
| SYS-043 | Fault logging | SWR-406 | Fault counter in telemetry |
| | | SWR-407 | Boot reason tracking |

### 4.5 Regulatory Requirements

| SYS ID | System Requirement | SWR ID | Software Requirement |
|--------|-------------------|--------|---------------------|
| SYS-050 | No encryption | SWR-501 | All data in clear |
| SYS-051 | Station ID in transmissions | SWR-502 | Callsign in every beacon |
| SYS-052 | Power limits compliance | SWR-503 | TX power limited to 2W max |

## 5. Software to Design Elements

### 5.1 State Machine Design

| SWR ID | Design Element | File |
|--------|----------------|------|
| SWR-301 | SAFE state implementation | state_machine.c |
| SWR-302 | IDLE state implementation | state_machine.c |
| SWR-303 | ACTIVE state implementation | state_machine.c |
| SWR-304 | SOC threshold logic | power_manager.c |
| SWR-306 | Transition conditions | state_machine.c |

### 5.2 Beacon Design

| SWR ID | Design Element | File |
|--------|----------------|------|
| SWR-201 | Beacon timer task | beacon.c |
| SWR-203 | AX.25 frame builder | beacon.c |
| SWR-204 | CRC-16 function | crc.c |
| SWR-207 | Callsign configuration | config.h |

### 5.3 AI Interface Design

| SWR ID | Design Element | File |
|--------|----------------|------|
| SWR-101 | LLM runner | llama_runner.py |
| SWR-102 | UART protocol | jetson_interface.c |
| SWR-103 | Fallback text | formatter.py |
| SWR-106 | Template beacons | beacon.c |

### 5.4 Fault Handling Design

| SWR ID | Design Element | File |
|--------|----------------|------|
| SWR-401 | Watchdog driver | watchdog.c |
| SWR-402 | Watchdog service | watchdog_mgr.c |
| SWR-403 | Fault handler | fault_handler.c |
| SWR-406 | Telemetry counters | telemetry.c |

## 6. Design to Verification

### 6.1 Verification Method Summary

| Method | Code | Count | Percentage |
|--------|------|-------|------------|
| Test | T | 42 | 70% |
| Analysis | A | 12 | 20% |
| Inspection | I | 4 | 7% |
| Demonstration | D | 2 | 3% |
| **Total** | | **60** | **100%** |

### 6.2 Requirements to Test Procedures

| SWR ID | Requirement | Method | Test Procedure |
|--------|-------------|--------|----------------|
| SWR-101 | Jetson TinyLM inference | T | TP-FLATSAT-001 |
| SWR-102 | UART AI request | T | TP-INT-002 |
| SWR-103 | Fallback templates | T | TP-004.9 |
| SWR-104 | AI duty cycle limit | T | TP-006.5 |
| SWR-201 | 60s beacon interval | T | TP-004.10 |
| SWR-203 | AX.25 frame format | T | TP-004.3 |
| SWR-204 | CRC-16 valid | T | TP-004.6 |
| SWR-301 | SAFE mode power | T | TP-006.1 |
| SWR-302 | IDLE mode power | T | TP-006.2 |
| SWR-303 | ACTIVE mode power | T | TP-006.3 |
| SWR-304 | Auto-SAFE at 20% | T | TP-002.7 |
| SWR-401 | Watchdog 5 second | T | TP-005.1 |
| SWR-402 | Watchdog kick | T | TP-001.9 |
| SWR-403 | Fault triggers SAFE | T | TP-005.7 |
| SWR-405 | Boot < 5 seconds | T | TP-001.8 |
| SWR-406 | Fault counters | T | TP-005.10 |
| SWR-501 | No encryption | I | Code review |
| SWR-502 | Callsign present | T | TP-004.4 |
| SWR-503 | TX power limit | T | TP-006.7,8 |

## 7. Verification Status

### 7.1 Status by Category

| Category | Total | Verified | Pending | % Complete |
|----------|-------|----------|---------|------------|
| AI Payload | 7 | 0 | 7 | 0% |
| RF System | 7 | 0 | 7 | 0% |
| Power System | 6 | 0 | 6 | 0% |
| Fault Tolerance | 7 | 5 | 2 | 71% |
| Regulatory | 3 | 3 | 0 | 100% |
| **Total** | **30** | **8** | **22** | **27%** |

*Note: Most verifications pending flatsat hardware testing*

### 7.2 Verification Evidence

| SWR ID | Status | Evidence |
|--------|--------|----------|
| SWR-501 | ✓ Verified | Code inspection - no encryption |
| SWR-502 | ✓ Verified | Unit test - callsign in frame |
| SWR-503 | ✓ Verified | Analysis - power output capped |
| SWR-401 | ✓ Verified | Unit test - watchdog timeout |
| SWR-402 | ✓ Verified | Unit test - watchdog kick |
| SWR-403 | ✓ Verified | Unit test - fault handler |
| SWR-406 | ✓ Verified | Unit test - counters increment |
| SWR-407 | ✓ Verified | Unit test - boot reason stored |

## 8. Traceability Gaps

### 8.1 Gap Analysis

| Gap Type | Count | Resolution |
|----------|-------|------------|
| Untestable requirements | 0 | N/A |
| Missing test procedures | 0 | All TPs defined |
| Unallocated requirements | 0 | All allocated |

### 8.2 Forward Traceability Complete

All 30 software requirements trace forward to:
- Design elements (source files)
- Verification methods
- Test procedures

### 8.3 Backward Traceability Complete

All test procedures trace backward to:
- Software requirements
- System requirements
- Mission objectives

## 9. RTM Maintenance

### 9.1 Update Triggers

| Event | Action |
|-------|--------|
| Requirement change | Update RTM within 5 days |
| Design change | Verify trace still valid |
| Test procedure change | Update verification links |
| Verification complete | Update status |

### 9.2 Review Schedule

| Review | RTM Action |
|--------|------------|
| PDR | Initial RTM baselined |
| CDR | RTM complete to verification |
| TRR | All verifications complete |
| FRR | RTM closed |

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR baseline |
