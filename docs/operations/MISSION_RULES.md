# SMART-QSO Mission Rules and Constraints

**Document ID**: SQSO-MRC-001
**Version**: 1.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft. Limits follow `docs/BASELINE_PARAMETERS.md` (Rev E), which governs; no independent CDR has been held (BASELINE §1). The command set has no software upload (SQSO-ICD-013 §7)

## 1. Purpose

This document defines the mission rules and flight constraints for SMART-QSO operations. These rules are mandatory and shall not be violated without Mission Director approval and documented rationale.

## 2. Applicable Documents

| Document ID | Title |
|-------------|-------|
| NPR 8705.4 | Risk Classification for NASA Payloads |
| 47 CFR Part 97 | Amateur Radio Service Regulations |
| SQSO-MOP-001 | Mission Operations Plan |
| SQSO-SHA-001 | Software Hazard Analysis |

## 3. Mission Rules Authority

### 3.1 Rule Categories

| Category | Authority to Waive | Documentation Required |
|----------|-------------------|------------------------|
| MANDATORY | None (inviolable) | N/A |
| CRITICAL | Mission Director + Safety | Waiver + Risk Assessment |
| STANDARD | Mission Director | Documented rationale |
| ADVISORY | Flight Controller | Log entry |

### 3.2 Rule Notation

- **MR-XXX**: Mission Rule (operational)
- **FC-XXX**: Flight Constraint (spacecraft limitation)
- **GR-XXX**: Ground Rule (ground segment)

## 4. Mandatory Rules (Inviolable)

### MR-001: RF Transmission Authorization
**NO RF TRANSMISSION SHALL OCCUR WITHOUT VALID FCC AMATEUR LICENSE AND COORDINATION**

Rationale: Legal requirement per 47 CFR Part 97. Violation constitutes federal offense.

Verification: License number logged before any transmission.

### MR-002: Two-Person Rule for Critical Commands
**ALL CRITICAL COMMANDS REQUIRE TWO CERTIFIED OPERATORS**

Critical commands include:
- Mode changes (SAFE/IDLE/ACTIVE)
- RF power level changes (`SET_POWER`)
- Safe mode release (`SET_MODE` from SAFE)
- `COMMISSIONING_RELEASE`, `DEPLOY_RETRY`, `KEY_ROTATE`, `EPS_CLEAR_LOCKOUT`, `FRAM_BYPASS`
- `PASSIVATE_ARM` and `PASSIVATE_EXECUTE` (two key custodians, `docs/operations/END_OF_LIFE_PLAN.md` §3)

Rationale: Prevents inadvertent critical commanding errors.

### MR-003: Command Verification
**ALL COMMANDS SHALL BE VERIFIED BEFORE TRANSMISSION**

Verification includes:
1. Command syntax check
2. Parameter range check
3. Operator verbal readback
4. Second operator confirmation

### MR-004: Safe Mode Respect
**SPACECRAFT IN SAFE MODE SHALL NOT BE COMMANDED OUT WITHOUT MISSION DIRECTOR APPROVAL**

Rationale: Safe mode indicates fault condition requiring investigation.

### MR-005: Emergency Stop Authority
**ANY OPERATOR MAY CALL "HOLD" TO STOP OPERATIONS**

Hold requires:
1. Immediate cessation of commanding
2. Documentation of reason
3. Mission Director notification
4. Formal resumption authorization

## 5. Critical Flight Constraints

### FC-001: Battery State of Charge Limits
**BATTERY SOC SHALL NOT BE DEPLETED BELOW 15%**

| Threshold | Action |
|-----------|--------|
| SOC < 55 % | No ACTIVE entry, no Jetson, no transponder (ACTIVE allowed at ≥ 55 % in sunlight) |
| SOC < 40 % | ACTIVE → IDLE |
| SOC < 25 % | Transition to SAFE mode |
| SOC < 15 % | SAFE → EMERGENCY (beacon shed); back to SAFE at 25 %, to IDLE above 40 % |

Rationale: the mode thresholds of BASELINE §5 (`eps_control.h`, `sm_power_event()`); depth of discharge ≤ 10 % per eclipse nominal, ≤ 30 % contingency (SYS-112).

### FC-002: Temperature Limits
**SPACECRAFT TEMPERATURES SHALL REMAIN WITHIN OPERATIONAL LIMITS**

| Component | Min (°C) | Max (°C) | Action if Exceeded |
|-----------|----------|----------|-------------------|
| Battery (charging) | 0 | +45 | No charging outside 0 to +45 °C (SYS-602); heater on below +5 °C, off above +9 °C |
| OBC | -40 | +85 | Thermal throttling |
| Jetson heat spreader | -15 (power-on gate) | +40 in a batch; +70 stuck on (SYS-603 Rev G) | Burst refused or ended; payload off |
| RF PA | — | +70 (resume at +60) | Transponder stopped (SQSO-ICD-003 §5.5) |

### FC-003: AI Payload Duty Cycle
**JETSON DUTY SHALL STAY WITHIN THE BASELINE BURST RULES**

Constraints (BASELINE §6; decision C-20; enforced by `sqso_fsw::payload_schedule`):
- Each power-on ≤ 10 minutes including boot and ≤ 3.6 kJ, ended early by the energy gate (360 J per accepted line)
- ≥ 100 minutes off after each burst; duty ≤ 5 % tumbling, ≤ 10 % sun-pointed (BASELINE §5)
- Only in ACTIVE, sunlit, SOC ≥ 55 %, spreader ≥ −15 °C
- Only after `COMMISSIONING_RELEASE` and `PLD_ENABLE`

Rationale: Power budget and thermal constraints.

### FC-004: RF Transmit Power
**RF TRANSMIT POWER SHALL NOT EXCEED 1 W** (BASELINE §7: 0.5 W nominal, 1 W maximum, the hardware maximum filed)

Power levels (`SET_POWER` 0x33):
| Setting | Power | Use Case |
|---------|-------|----------|
| 0 | 0.5 W | Nominal |
| 1 | 1.0 W | Commissioning check (C-6) and weak-link cases |

### FC-005: Watchdog Timeout
**WATCHDOG SHALL NOT BE DISABLED DURING FLIGHT OPERATIONS**

Watchdog: the STM32 IWDG at 25.6 s (`firmware/src/wdt.rs`) and the external TPS3850, kicked every 10 s while every task
checks in. No command disables either; the GSE watchdog-disable line exists only on the ground access port.

## 6. Standard Mission Rules

### MR-101: Pass Operations
**ALL PASSES SHALL BE LOGGED WITH REQUIRED DATA**

Minimum log data:
- Pass start/end times (UTC)
- Operator on console
- Commands sent
- Telemetry received
- Anomalies observed
- Actions taken

### MR-102: Telemetry Recording
**ALL RECEIVED TELEMETRY SHALL BE RECORDED**

Requirements:
- Raw data recording (always on)
- Decoded telemetry display
- Automatic limit checking
- Archive within 24 hours

### MR-103: Anomaly Reporting
**ALL ANOMALIES SHALL BE REPORTED WITHIN 24 HOURS**

Anomaly categories:
| Category | Response Time | Escalation |
|----------|---------------|------------|
| Critical | Immediate | Mission Director |
| Major | 4 hours | Flight Director |
| Minor | 24 hours | Shift log |

### MR-104: Shift Handoff
**COMPLETE SHIFT HANDOFF REQUIRED AT OPERATOR CHANGE**

Handoff includes:
1. Current spacecraft state
2. Recent anomalies
3. Pending actions
4. Upcoming passes
5. Open issues

### MR-105: Ground Station Readiness
**GROUND STATION SHALL BE VERIFIED READY BEFORE PASS**

T-30 minutes: System power-up and initialization
T-15 minutes: Antenna pointing verification
T-5 minutes: Transceiver configuration check
T-1 minute: Operator ready call

### MR-106: Command Uplink Windows
**COMMANDING LIMITED TO DESIGNATED WINDOWS**

- Primary window: First 10 minutes of pass
- Secondary window: Last 5 minutes of pass
- No commanding during data download segments
- Exception: Emergency commands

### MR-107: Software Changes
**THERE IS NO IN-FLIGHT SOFTWARE UPLOAD**

The command dictionary has no upload command (SQSO-ICD-013 §7). Flight-software changes are made only on the ground
before integration, under configuration control. In flight, behaviour is changed only through the parameters the
dictionary allows (`SET_CONTACT_TIMEOUT`, `SET_BEACON`, `SET_POWER`, `ADCS_SET_GEOMETRY`, the calibration commands).

## 7. Ground Rules

### GR-001: Ground Station Maintenance
**MAINTENANCE SHALL NOT BE SCHEDULED DURING CRITICAL PASSES**

Critical passes:
- First contact after launch
- Anomaly investigation
- The passivation pass
- Scheduled events

### GR-002: Network Connectivity
**BACKUP COMMUNICATION REQUIRED FOR CRITICAL OPS**

Requirements:
- Primary: Internet/VPN
- Backup: Phone bridge
- Test backup before critical ops

### GR-003: Operator Availability
**MINIMUM OPERATOR COVERAGE DURING MISSION-CRITICAL PERIODS**

| Period | Minimum Coverage |
|--------|------------------|
| Launch + 72 hours | Continuous |
| Commissioning | All passes |
| Anomaly response | As required |
| Normal ops | Scheduled passes |

### GR-004: Weather Constraints
**GROUND OPERATIONS LIMITED BY WEATHER**

| Condition | Constraint |
|-----------|------------|
| Lightning within 10 mi | No operations |
| Wind > 40 mph | Antenna stow |
| Heavy precipitation | Degraded operations |

## 8. Flight Rules Matrix

### 8.1 Mode Transition Rules

| Current | Target | Conditions Required | Authority |
|---------|--------|---------------------|-----------|
| SAFE | IDLE | SOC above 40 %, no faults (automatic above 40 %; by `SET_MODE` 1 after investigation) | Automatic, or FC + MD approval |
| IDLE | ACTIVE | Sunlit, SOC ≥ 55 %, commissioning hold released, no faults | Automatic or FC |
| ACTIVE | IDLE | SOC < 40 % OR fault | Automatic or FC |
| ACTIVE | SAFE | Critical fault OR ground command | Automatic or FC + MD |
| IDLE | SAFE | SOC < 25% OR critical fault OR cmd | Automatic or FC + MD |

### 8.2 Response to Off-Nominal Conditions

| Condition | Immediate Action | Follow-up |
|-----------|------------------|-----------|
| Loss of signal | Wait 1 pass, then investigate | Anomaly report |
| Unexpected safe mode | No commanding, investigate | Root cause analysis |
| Battery undervoltage | Monitor, reduce load | Power analysis |
| Thermal limit exceeded | Monitor, reduce operations | Thermal analysis |
| Watchdog reset | Check reset count, monitor | Software analysis |

## 9. Emergency Procedures Summary

### 9.1 Loss of Communication
```
IF no signal for 3 consecutive passes:
1. Verify ground station nominal
2. Check TLE currency
3. Attempt wide-band search
4. Coordinate with other stations
5. If no contact 24 hours: Escalate
```

### 9.2 Uncontrolled Tumble
```
IF tumble detected (from signal strength variation):
1. DO NOT COMMAND
2. Wait for autonomous recovery
3. Monitor for stabilization
4. Resume ops only when stable
```

### 9.3 Battery Emergency
```
IF SOC < 15%:
1. Confirm reading
2. Minimize all loads (should be auto-SAFE)
3. Wait for charging orbit
4. Monitor recovery
5. Do not resume ops until SOC > 40%
```

## 10. Constraint Waivers

### 10.1 Waiver Request Form

```
CONSTRAINT WAIVER REQUEST

Date: ____________
Requestor: ____________

CONSTRAINT ID: ____________
CONSTRAINT TEXT: ____________

REASON FOR WAIVER:
_________________________________
_________________________________

DURATION: From ______ To ______

RISK ASSESSMENT:
_________________________________

MITIGATIONS:
_________________________________

APPROVALS:
[ ] Flight Controller: _______ Date: _____
[ ] Mission Director: _______ Date: _____
[ ] Safety (if critical): _______ Date: _____
```

### 10.2 Waiver Log

| ID | Constraint | Duration | Approved By | Status |
|----|------------|----------|-------------|--------|
| | | | | |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): FC-001 and §8.1 the SOC thresholds of BASELINE §5 (were 70/30/20 %); FC-002 battery charge window, Jetson spreader and PA limits; FC-003 the burst rules of BASELINE §6 and C-20 (were 60 s bursts, 10 % duty, SOC 70 %); FC-004 1 W maximum (was 2 W); FC-005 the IWDG and TPS3850 watchdogs; MR-002, MR-107, GR-001 no in-flight software upload exists; status |
