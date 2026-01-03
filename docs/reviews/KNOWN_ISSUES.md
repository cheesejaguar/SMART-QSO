# SMART-QSO Known Issues Document

**Document ID**: SQSO-KI-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document tracks all known issues, limitations, and accepted risks for the SMART-QSO flight software per NPR 7150.2 requirements. It provides transparency for flight readiness decisions.

## 2. Issue Classification

### 2.1 Severity Levels

| Level | Definition | Flight Impact |
|-------|------------|---------------|
| Critical | Prevents mission success | No-fly |
| High | Degrades primary mission | Requires waiver |
| Medium | Workaround available | Document and monitor |
| Low | Cosmetic or minor | Accept |

### 2.2 Status Definitions

| Status | Definition |
|--------|------------|
| Open | Issue identified, not resolved |
| In Work | Resolution in progress |
| Resolved | Fix implemented and verified |
| Closed | Issue closed, no action needed |
| Accepted | Risk accepted for flight |
| Deferred | Deferred to future release |

## 3. Issue Summary

### 3.1 Summary by Severity

| Severity | Open | In Work | Accepted | Closed | Total |
|----------|------|---------|----------|--------|-------|
| Critical | 0 | 0 | 0 | 0 | 0 |
| High | 0 | 0 | 0 | 0 | 0 |
| Medium | 0 | 0 | 2 | 0 | 2 |
| Low | 0 | 0 | 3 | 0 | 3 |
| **Total** | **0** | **0** | **5** | **0** | **5** |

### 3.2 Summary by Component

| Component | Issues | Critical/High | Accepted |
|-----------|--------|---------------|----------|
| Flight Software | 3 | 0 | 3 |
| Payload Software | 1 | 0 | 1 |
| Ground Software | 1 | 0 | 1 |
| **Total** | **5** | **0** | **5** |

## 4. Known Issues

---

### KI-001: HAL Implementation Stubs

**Severity**: Medium
**Status**: Accepted
**Component**: Flight Software
**Affected Files**: `time_utils.c`, `uart_comm.c`, `adcs_control.c`, `assert_handler.c`

**Description**:
Several Hardware Abstraction Layer (HAL) functions contain stub implementations marked with TODO comments. These require target hardware headers (Ambiq Apollo4 SDK) to complete.

**Affected Functions**:
- `time_utils.c:29` - HAL-based time retrieval
- `uart_comm.c:181` - HAL-based UART initialization
- `adcs_control.c:351` - I2C magnetometer read
- `adcs_control.c:377` - ADC sun sensor read
- `adcs_control.c:503` - Magnetorquer PWM control
- `adcs_control.c:522` - Magnetorquer disable
- `assert_handler.c:196` - Watchdog reset trigger

**Impact**:
Software logic is complete and tested. HAL bindings cannot be completed until target SDK is integrated during flatsat testing.

**Workaround**:
Stub functions return simulated values for testing. Will be replaced with actual HAL calls during hardware integration.

**Acceptance Rationale**:
This is expected at CDR stage. HAL integration is a flatsat activity per standard development flow.

**Closure Plan**:
Complete during TP-FLATSAT-001 (Flatsat Software Development Unit Test).

---

### KI-002: No Persistent Fault Logging

**Severity**: Medium
**Status**: Accepted
**Component**: Flight Software
**Affected Files**: `fault_handler.c`

**Description**:
Fault logs are stored in RAM and lost on power cycle or reset. No persistent storage of fault history across reboots.

**Impact**:
Post-anomaly analysis may lack fault sequence data from before reset.

**Workaround**:
- Fault counts are telemetered each beacon
- Boot reason code preserved across reset
- Ground can reconstruct timeline from telemetry

**Acceptance Rationale**:
Adding flash wear-leveling for fault logs adds complexity and flash wear. Telemetry provides sufficient fault visibility for Class III mission.

**Closure Plan**:
Accepted as-is. Consider for v1.1 if operational experience shows need.

---

### KI-003: Fixed Beacon Interval

**Severity**: Low
**Status**: Accepted
**Component**: Flight Software
**Affected Files**: `beacon.c`

**Description**:
Beacon interval is fixed at 60 seconds and not ground-commandable.

**Impact**:
Cannot adjust beacon rate based on operational needs.

**Workaround**:
60-second interval is compliant with IARU coordination and sufficient for amateur tracking.

**Acceptance Rationale**:
Simplifies software and reduces command interface complexity. 60s meets all operational requirements.

**Closure Plan**:
Accepted as-is. Feature request for v1.1 if needed.

---

### KI-004: AI Model Loading at Boot

**Severity**: Low
**Status**: Accepted
**Component**: Payload Software
**Affected Files**: `llama_runner.py`

**Description**:
TinyLM model is loaded into memory at agent startup. No hot-swap capability.

**Impact**:
Cannot update model without Jetson restart.

**Workaround**:
Model updates require Jetson power cycle, which can be commanded from ground.

**Acceptance Rationale**:
Model updates are rare. Power cycle is acceptable for Class III mission.

**Closure Plan**:
Accepted as-is.

---

### KI-005: Single Ground Station Tested

**Severity**: Low
**Status**: Accepted
**Component**: Ground Software
**Affected Files**: `beacon_decoder.py`

**Description**:
Ground software tested only with development TNC. Compatibility with other TNCs not verified.

**Impact**:
May require tuning for different amateur ground stations.

**Workaround**:
KISS protocol is standard; most TNCs should work. Documentation provides configuration guidance.

**Acceptance Rationale**:
Amateur community uses diverse equipment. Cannot test all configurations. KISS standard ensures basic compatibility.

**Closure Plan**:
Document any compatibility issues reported by operators post-launch.

---

## 5. Resolved Issues

*No issues have been resolved and closed at this time.*

---

## 6. Issue Tracking Process

### 6.1 Issue Lifecycle

```
┌────────┐    ┌─────────┐    ┌──────────┐    ┌────────┐
│  Open  │───▶│ In Work │───▶│ Resolved │───▶│ Closed │
└────────┘    └─────────┘    └──────────┘    └────────┘
     │                             │
     │                             │
     ▼                             ▼
┌──────────┐                ┌──────────┐
│ Accepted │                │ Deferred │
└──────────┘                └──────────┘
```

### 6.2 Issue Submission Template

```
KNOWN ISSUE SUBMISSION

Date: ___________
Submitter: ___________
Component: ___________

TITLE: ___________

SEVERITY: [ ] Critical [ ] High [ ] Medium [ ] Low

DESCRIPTION:
_________________________________

AFFECTED FILES:
_________________________________

IMPACT:
_________________________________

WORKAROUND:
_________________________________

SUGGESTED RESOLUTION:
_________________________________
```

### 6.3 Issue Review Criteria

Issues are reviewed at each milestone:
- **PDR**: All Critical/High must have mitigation plan
- **CDR**: All Critical must be resolved; High must have acceptance rationale
- **TRR**: All issues must be Resolved, Accepted, or Deferred with rationale
- **FRR**: No Critical/High open; all Accepted issues documented

## 7. Acceptance Signatures

The undersigned acknowledge and accept the known issues documented herein for flight:

| Role | Name | Signature | Date |
|------|------|-----------|------|
| Software Lead | | | |
| Systems Engineer | | | |
| Mission Assurance | | | |
| Project Manager | | | |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR baseline |
