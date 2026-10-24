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
**Affected Files**: `software/rust/sqso-ffi/src/adcs_control.rs`, `software/rust/firmware/src/`

**Description**:
The OBC is the STM32L4R5ZI (`docs/BASELINE_PARAMETERS.md` §6); the Apollo4 SDK named in Version 1.0 is superseded. The
flight software is Rust (2026-10-24) and runs both as a host simulation and as the Embassy firmware for the STM32L4R5ZI
(`software/rust/firmware`), which CI builds and checks but which has not run on hardware. The firmware's HAL drives
GPIO, the time base, the independent watchdog and reset cause, the Jetson and debug UARTs, I2C, SPI, the ADC, the FM25V20A
FRAM (`hal_flash_*`) and the telecommand receiver (KISS frames from the TNC UART, `hal_radio_*`). Its pin assignment is
provisional until the OBC schematic baselines one (`software/rust/firmware/src/gpio.rs`).

Not yet written, because the parts are not selected: the ADCS sensor and actuator drivers. On the target, as in the
simulation, the ADCS reads its simulated field and sun vector:

**Affected Functions**:
- `adcs_read_magnetometer()` - I2C magnetometer read (part not selected)
- `adcs_read_sun_sensors()` - sun sensor read (part not selected)
- `adcs_apply_dipole()`, `adcs_disable_mtq()` - magnetorquer drive (no PWM driver yet)

The sensor table's readings (`sensors.rs`) are likewise the power model's, not ADC conversions, until the EPS sense
circuits are designed (the ADC front-end scaling in `firmware/src/adc.rs` is provisional until then).

**Impact**:
On the target these functions return simulated values. The logic above them is exercised by the unit tests in
`software/flight/tests/` and `software/rust`; nothing has run on hardware.

**Workaround**:
None needed before hardware: each is replaced by a HAL call when its part is selected and its driver written.

**Acceptance Rationale**:
Expected before engineering-model hardware exists. HAL integration is a flatsat activity per standard development flow.

**Closure Plan**:
Complete during TP-FLATSAT-001 (Flatsat Software Development Unit Test).

---

### KI-002: Fault Log Not Persistent on the Target

**Severity**: Medium
**Status**: Resolved in code (2026-10-24); to be verified on hardware
**Component**: Flight Software
**Affected Files**: `software/rust/sqso-ffi/src/fault_mgmt.rs`, `software/rust/sqso-ffi/src/store.rs`,
`software/rust/firmware/src/fram.rs`

**Description**:
The fault log is saved after each new entry and reloaded at start-up, each entry's CRC checked and corrupt ones discarded
(`fault_log_save()`, `fault_log_load()`; SRS-F041). In the host simulation it is a file, and `test_fault_persistence` in
`software/flight/tests/test_fault_mgmt.c` saves and reloads a log. Until 2026-10-24 the target had no file system and no
FRAM driver, so the log was lost on reset. The Rust port stores it in `FLASH_REGION_FAULT_LOG`, which the firmware maps to
9,728 bytes of the FRAM event log (`STORAGE_DESIGN.md` §3.3), enough for all 100 entries.

**Impact**:
None expected once verified; until then post-anomaly analysis cannot rely on the stored sequence.

**Workaround**:
- Fault counts are telemetered each beacon
- Boot reason code preserved across reset
- Ground can reconstruct timeline from telemetry

**Acceptance Rationale**:
SRS-F041 requires the fault log in non-volatile memory; the code now does this on both builds.

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
