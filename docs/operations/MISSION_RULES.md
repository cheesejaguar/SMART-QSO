# SMART-QSO Mission Rules and Constraints

**Document ID**: SQSO-MRC-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

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
- RF power level changes
- Software uploads
- Stored command sequences
- Safe mode release

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
| SOC < 30% | Reduce operations, no AI |
| SOC < 20% | Transition to SAFE mode |
| SOC < 15% | Absolute minimum, emergency only |

Rationale: Battery damage below 15% SOC; reduced cycle life below 20%.

### FC-002: Temperature Limits
**SPACECRAFT TEMPERATURES SHALL REMAIN WITHIN OPERATIONAL LIMITS**

| Component | Min (°C) | Max (°C) | Action if Exceeded |
|-----------|----------|----------|-------------------|
| Battery | -10 | +45 | Heater control / Reduce ops |
| OBC | -40 | +85 | Thermal throttling |
| Jetson | -20 | +70 | Power off payload |
| RF PA | -20 | +60 | Reduce TX power |

### FC-003: AI Payload Duty Cycle
**JETSON INFERENCE SHALL NOT EXCEED 10% DUTY CYCLE**

Constraints:
- Maximum continuous inference: 60 seconds
- Minimum off-time between bursts: 540 seconds (9 minutes)
- No inference when SOC < 70%
- No inference during eclipse

Rationale: Power budget and thermal constraints.

### FC-004: RF Transmit Power
**RF TRANSMIT POWER SHALL NOT EXCEED 2W WITHOUT THERMAL VERIFICATION**

Power levels:
| Setting | Power | Use Case |
|---------|-------|----------|
| LOW | 0.5W | Standard beacons |
| MEDIUM | 1.0W | Priority communications |
| HIGH | 2.0W | Emergency only, thermally limited |

### FC-005: Watchdog Timeout
**WATCHDOG SHALL NOT BE DISABLED DURING FLIGHT OPERATIONS**

Watchdog timeout: 5 seconds
Exception: Only during ground-commanded software upload with operator monitoring.

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

### MR-107: Software Upload Constraints
**SOFTWARE UPLOADS REQUIRE FORMAL APPROVAL**

Requirements:
1. Change request documented
2. Software tested on ground
3. Rollback procedure verified
4. Two operators on console
5. Full pass available for verification

## 7. Ground Rules

### GR-001: Ground Station Maintenance
**MAINTENANCE SHALL NOT BE SCHEDULED DURING CRITICAL PASSES**

Critical passes:
- First contact after launch
- Anomaly investigation
- Software uploads
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
| SAFE | IDLE | SOC > 20%, stable 60s, no faults | FC + MD approval |
| IDLE | ACTIVE | SOC > 70%, power positive, no faults | FC |
| ACTIVE | IDLE | SOC < 30% OR power negative OR fault | Automatic or FC |
| ACTIVE | SAFE | Critical fault OR ground command | Automatic or FC + MD |
| IDLE | SAFE | SOC < 20% OR critical fault OR cmd | Automatic or FC + MD |

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
