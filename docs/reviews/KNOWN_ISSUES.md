# SMART-QSO Known Issues Document

**Document ID**: SQSO-KI-001
**Version**: 1.1
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
| Medium | 1 | 0 | 1 | 0 | 2 |
| Low | 1 | 0 | 2 | 0 | 3 |
| **Total** | **2** | **0** | **3** | **0** | **5** |

### 3.2 Summary by Component

| Component | Issues | Critical/High | Accepted |
|-----------|--------|---------------|----------|
| Flight Software | 3 | 0 | 1 |
| Payload Software | 1 | 0 | 1 |
| Ground Software | 1 | 0 | 1 |
| **Total** | **5** | **0** | **3** |

## 4. Known Issues

---

### KI-001: HAL Implementation Stubs

**Severity**: Medium
**Status**: Accepted
**Component**: Flight Software
**Affected Files**: `uart_comm.c`, `adcs_control.c`, `assert_handler.c`, `hal/hal_stm32l4.c`

**Description**:
The OBC is the STM32L4R5ZI (`docs/BASELINE_PARAMETERS.md` §6); the Apollo4 SDK named in Version 1.0 is superseded. The
flight software runs as a host simulation. Its target port is a register-level HAL for the STM32L4R5ZI
(`software/flight/src/hal/hal_stm32l4.c`), compiled by CI and not run on hardware. That HAL implements GPIO, the SysTick
time base, the independent watchdog and the reset cause. Not yet written, and returning an error so that the software
fails safe: the FRAM non-volatile memory driver (`hal_flash_*`), UART, I2C, SPI, ADC and the 435 MHz receiver
(`hal_radio_*`). Above the HAL, these functions still carry TODO stubs for the target build:

**Affected Functions**:
- `uart_comm.c`, `uart_init_with_config()` - HAL-based UART initialization
- `adcs_control.c`, `adcs_read_magnetometer()` - I2C magnetometer read
- `adcs_control.c`, `adcs_read_sun_sensors()` - ADC sun sensor read
- `adcs_control.c`, `adcs_apply_dipole()` - Magnetorquer PWM control
- `adcs_control.c`, `adcs_disable_mtq()` - Magnetorquer disable
- `assert_handler.c`, `trigger_reset()` - Watchdog reset trigger

(`time_utils.c` no longer has a stub: it reads the HAL time base.)

**Impact**:
On the target these functions do nothing or return an error. The logic above them runs in the host simulation build,
where the unit tests in `software/flight/tests/` exercise it; nothing has run on hardware.

**Workaround**:
In the simulation build the stubs return simulated values. Each is replaced by a HAL call when its driver is written.

**Acceptance Rationale**:
Expected before engineering-model hardware exists. HAL integration is a flatsat activity per standard development flow.

**Closure Plan**:
Complete during TP-FLATSAT-001 (Flatsat Software Development Unit Test).

---

### KI-002: Fault Log Not Persistent on the Target

**Severity**: Medium
**Status**: Open
**Component**: Flight Software
**Affected Files**: `fault_mgmt.c`, `hal/stm32l4_syscalls.c`, `hal/hal_stm32l4.c`

**Description**:
Version 1.0 said the fault log is kept only in RAM, in `fault_handler.c`. The code is in `fault_mgmt.c`, and it saves the
log after each new entry and reloads it at start-up, checking each entry's CRC and discarding corrupt ones
(`fault_log_save()`, `fault_log_load()`; SRS-F041). The save and load are file operations. In the host simulation build
they use a file, and the unit test `test_fault_persistence` in `software/flight/tests/test_fault_mgmt.c` saves and
reloads a log. On the target there is no file system (`software/flight/src/hal/stm32l4_syscalls.c`: opens fail), and the
FRAM driver that would hold the log is not written (`hal/hal_stm32l4.c`), so on the OBC the log is still lost on reset or
power cycle.

**Impact**:
Post-anomaly analysis on orbit would lack the fault sequence from before a reset until the log is stored in FRAM.

**Workaround**:
- Fault counts are telemetered each beacon
- Boot reason code preserved across reset
- Ground can reconstruct timeline from telemetry

**Acceptance Rationale**:
Not accepted: SRS-F041 requires the fault log in non-volatile memory, and the code intends it.

**Closure Plan**:
Store the fault log in FRAM through `hal_flash_*` on the target (FRAM allocation in `hardware/obc/OBC_DESIGN.md` §3.3),
then show on the engineering model that the log survives a reset and a power cycle.

---

### KI-003: Commanded Beacon Interval Has No Effect

**Severity**: Low
**Status**: Open
**Component**: Flight Software
**Affected Files**: `beacon.c`, `system_state.c`, `cmd_handler.c`

**Description**:
Version 1.0 said the beacon interval is fixed at 60 seconds and not ground-commandable. In the code the interval follows
the power mode: 60 s in ACTIVE, 120 s in IDLE and 180 s in SAFE (`BEACON_INTERVAL_*_MS` in
`software/flight/include/beacon.h`, set by `beacon_update_interval()`), as `docs/BASELINE_PARAMETERS.md` §8 and SYS-211
require. The SET_BEACON telecommand exists: it accepts 10 to 120 s and stores the value
(`sys_set_beacon_interval()` in `system_state.c`), but the beacon schedule never reads the stored value, so the command
has no effect on timing. 180 s, the SAFE interval, cannot be commanded (the note on SYS-211).

**Impact**:
An operator who sends SET_BEACON gets an accepted command and an unchanged beacon rate.

**Workaround**:
None needed for the baseline intervals, which are fixed per mode.

**Acceptance Rationale**:
Not accepted until the PI decides what SET_BEACON should do.

**Closure Plan**:
Either make the beacon schedule use the commanded value (and widen its range to include 180 s), or remove the
telecommand from the command dictionary (SQSO-ICD-013 §7); then update this entry.

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
| 1.1 | 2026-10-16 | SMART-QSO Team | Corrected from the code and the baseline. KI-001: the OBC is the STM32L4R5ZI, not the Apollo4; remaining stubs listed by function, without line numbers. KI-002: the fault log is saved and reloaded (`fault_mgmt.c`), but only to a host file; on the target it is not persistent until the FRAM driver exists; status Open (SRS-F041). KI-003: the interval is 60/120/180 s by mode (SYS-211), and SET_BEACON is accepted but has no effect; status Open. Summary tables updated |
