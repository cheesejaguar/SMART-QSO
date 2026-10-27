# SMART-QSO Known Issues Document

**Document ID**: SQSO-KI-001
**Version**: 1.4
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft, live (updated as issues change). No issue acceptance has been signed; no independent PDR or CDR has
been held (BASELINE §1)

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

KI-002 is counted as open: resolved in code, not yet verified on hardware.

### 3.2 Summary by Component

| Component | Issues | Critical/High | Accepted |
|-----------|--------|---------------|----------|
| Flight Software | 3 | 0 | 1 |
| Payload Software | 1 | 0 | 1 |
| Ground Software | 1 | 0 | 1 |
| **Total** | **5** | **0** | **3** |

## 4. Known Issues

---

### KI-001: Firmware Not Run on Hardware; Sensor Table Simulated

**Severity**: Medium
**Status**: Accepted (until the EM flatsat)
**Component**: Flight Software
**Affected Files**: `software/rust/sqso-ffi/src/sensors.rs`, `software/rust/firmware/src/`

**Description**:
The OBC is the STM32L4R5ZI (`docs/BASELINE_PARAMETERS.md` §6); the Apollo4 SDK named in Version 1.0 is superseded. The
flight software is Rust (2026-10-24) and runs both as a host simulation and as the Embassy firmware for the STM32L4R5ZI
(`software/rust/firmware`), which CI builds and checks but which has not run on hardware: no board has been built. The
firmware's HAL drives GPIO, the time base, the independent watchdog and reset cause, the Jetson and debug UARTs, I2C,
SPI, the ADC, the FM25V20A FRAM (`hal_flash_*`), the magnetorquer bridges (`mtq.rs`) and the KISS TNC on the radio UART
for both the telecommand receiver and the downlink transmitter (`hal_radio_*`). Its pin assignment follows the master
allocation of `hardware/electrical/INTERCONNECT.md` (SQSO-ICX-001) and stays provisional until a board exists
(`software/rust/firmware/src/gpio.rs`; CDR 2026 review §7.2).

Since the CDR 2026 design review the ADCS drives the OBC board's sensors on the target (the M-G370PDT0 gyro with the
BMI088 as cold spare, the RM3100 magnetometer, the ADS7138 sun-sensor ADC; `software/rust/sqso-ffi/src/adcs_control.rs`)
and commands the coils through `hal_mtq.h`; the host build keeps the simulated field and sun vector. The downlink
transmitter driver exists since 2026-10-25 (CR-2026-005): `beacon_transmit()` hands the frame to the downlink
(`software/flight/include/downlink.h`), which gates it, limits its key-down and duty, selects the power and watches the
transmitter's monitors, and the HAL writes it as KISS to the TNC (`software/rust/firmware/src/radio.rs`). The RF board's
parts are chosen in its build-to design (`hardware/electrical/rf/RF_BOARD_DESIGN.md`, SQSO-RF-BRD-001), but the
transmitter's thresholds and monitor scaling stay open in the software requirements (TBR-SRS-003) until measured on
hardware, and nothing has transmitted from hardware.

Still simulated on the target: the sensor table (`sensors.rs`), whose readings are the power model's, not ADC
conversions (finding SW-01); the ADC front-end scaling in `firmware/src/adc.rs` follows the EPS and OBC board designs
but is unmeasured.

**Impact**:
On the target the sensor table returns simulated values, and no driver has been exercised against a real part. The
logic above them is exercised by the unit tests in `software/flight/tests/` and `software/rust`.

**Workaround**:
None needed before hardware.

**Acceptance Rationale**:
Expected before engineering-model hardware exists (EM integration 2027-06-01 to 2027-07-27, `mission/SCHEDULE.md`).

**Closure Plan**:
Sensor-table readers on the HAL, and every driver run on the EM boards during TP-FLATSAT-001.

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
**Affected Files**: `software/rust/sqso-ffi/src/beacon.rs`, `software/rust/sqso-ffi/src/system_state.rs`, `software/rust/sqso-ffi/src/cmd_handler.rs` (were `beacon.c`, `system_state.c`, `cmd_handler.c`)

**Description**:
Version 1.0 said the beacon interval is fixed at 60 seconds and not ground-commandable. In the code the interval follows
the power mode: 60 s in ACTIVE, 120 s in IDLE and 180 s in SAFE (`BEACON_INTERVAL_*_MS` in
`software/flight/include/beacon.h`, set by `beacon_update_interval()`), as `docs/BASELINE_PARAMETERS.md` §8 and SYS-211
require. The SET_BEACON telecommand exists: it accepts 10 to 120 s and stores the value
(`sys_set_beacon_interval()` in `software/rust/sqso-ffi/src/system_state.rs`), but the beacon schedule never reads the stored value, so the command
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
**Affected Files**: `software/payload/llama_runner.py`

**Description**:
The language model is loaded into memory when the payload agent starts. No hot-swap capability.

**Impact**:
None in the baseline: the Jetson is powered for at most 10 minutes per burst and boots each time (BASELINE §6), so the
model is loaded at every burst. The generative beacon is a gated experiment; the flight baseline is the template
selector on the OBC (decision D5). No model upload exists: there is no software-update mechanism
(`software/flight/docs/BOOTLOADER_REQUIREMENTS.md`), and community model updates are not in the flight baseline
(BASELINE §2).

**Workaround**:
None needed.

**Acceptance Rationale**:
Loading at start-up matches the burst operation. Acceptable for a Class D payload (NPR 8705.4B, `docs/BASELINE_PARAMETERS.md` §1).

**Closure Plan**:
Accepted as-is.

---

### KI-005: Beacon Decoder Not Tested with a TNC

**Severity**: Low
**Status**: Accepted
**Component**: Ground Software
**Affected Files**: `software/ground/beacon_decoder.py`

**Description**:
The ground beacon decoder is tested only by unit tests on synthetic AX.25 and KISS data
(`software/ground/tests/test_beacon_decoder.py`); no TNC or radio has been connected to it. Compatibility with real TNCs
is not verified.

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
| 1.2 | 2026-10-25 | SMART-QSO Team | KI-001: the downlink transmitter driver is missing too (`hal_radio.h` has a receive side only; beacon frames are counted and shown on the console, not transmitted). KI-004: "Class III mission" corrected to the Class D payload risk class (finding PI-18) |
| 1.3 | 2026-10-25 | SMART-QSO Team | KI-001: the downlink transmitter driver is written (CR-2026-005, `downlink.h`, the firmware's `hal_radio_transmit`); the TNC and transmitter are still not selected (TBR-SRS-003). `beacon_transmit()` removed from the affected functions |
| 1.1, note | 2026-10-24 | SMART-QSO Team | KI-003 affected files name the Rust files of the 2026-10-24 port; the issue is unchanged |
| 1.4 | 2026-10-27 | SMART-QSO Team | Documentation audit (post-CDR 2026): status "Draft, live" (was "CDR Baseline"; no CDR has been held). KI-001 to the code after the CDR 2026 design review: ADCS sensor and magnetorquer drivers exist on the target, pin map from SQSO-ICX-001, RF parts chosen in SQSO-RF-BRD-001; the sensor table remains simulated; closure on the EM flatsat. KI-004: the model is loaded at every burst; no model upload in the baseline (was "model updates are rare"). KI-005: tested by unit tests on synthetic data only (was "with development TNC"); paths given in full |
