# TP-DITL-001: Day-in-the-Life Test Procedure

**Document ID**: SQSO-TP-DITL-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Test Overview

### 1.1 Purpose
Validate mission operations by executing a 24-hour operational simulation that includes all planned activities, shift handoffs, and contingency responses.

### 1.2 Objectives

| Objective | Success Criteria |
|-----------|------------------|
| Validate procedures | All procedures executable |
| Verify tools | All ops tools functional |
| Train team | Team executes operations |
| Find issues | Identify procedural gaps |
| Build confidence | Team ready for operations |

### 1.3 Test Duration
- **Total Duration**: 24-30 hours
- **Shift Structure**: Three 8-hour shifts
- **Pass Count**: 5-6 simulated passes

## 2. Test Configuration

### 2.1 Equipment

| Item | Description | Required |
|------|-------------|----------|
| Ground Station | Ops workstation | [ ] |
| Radio/TNC | Ground equipment | [ ] |
| Test Unit | Flatsat or EM | [ ] |
| RF Simulator | Path simulation | [ ] |
| Displays | Status boards | [ ] |
| Communications | Voice loops | [ ] |

### 2.2 Personnel

| Shift | Duration | Positions |
|-------|----------|-----------|
| Shift 1 | 00:00-08:00 | Mission Director, FC, Ground |
| Shift 2 | 08:00-16:00 | Mission Director, FC, Ground |
| Shift 3 | 16:00-24:00 | FC, Ground (reduced) |

### 2.3 Simulation Fidelity

| Aspect | Fidelity | Notes |
|--------|----------|-------|
| Timing | Real-time | Actual pass durations |
| Commands | Full | Real command formats |
| Telemetry | Full | Actual telemetry |
| RF Link | Simulated | Attenuated/wired |
| Orbital Events | Scripted | Pass times from schedule |

## 3. Pre-Test Setup

### 3.1 Setup Checklist

```
DITL PRE-TEST SETUP

Date: ___________
Test Director: ___________

GROUND SEGMENT:
[ ] Ground station powered on
[ ] Ops software loaded and running
[ ] TLE loaded (simulated orbit)
[ ] Pass schedule distributed
[ ] Voice loops tested
[ ] Logging enabled

SPACE SEGMENT:
[ ] Test unit configured
[ ] Flight software loaded
[ ] Initial state: SAFE
[ ] RF connection verified
[ ] Power supply set

DOCUMENTATION:
[ ] Procedures at each station
[ ] Log sheets distributed
[ ] Contact list posted
[ ] Schedule visible

TEAM:
[ ] All positions staffed
[ ] Roles briefed
[ ] Safety briefing complete
[ ] Go/No-Go poll complete

Start Time: ___:___:___
```

### 3.2 Initial Configuration

| Parameter | Setting |
|-----------|---------|
| Spacecraft State | SAFE |
| Battery SOC | 90% simulated |
| AI Status | OFF |
| Beacon Mode | Standard |
| TX Power | Level 1 |

## 4. DITL Timeline

### 4.1 Master Timeline

```
DITL MASTER TIMELINE

TIME    EVENT                           RESPONSIBLE
────────────────────────────────────────────────────
00:00   SHIFT 1 START
        - Handoff briefing              Mission Director
        - System status review          All

00:30   PRE-PASS 1 PREP
        - Configure for pass            Ground
        - Review pass plan              FC

01:00   PASS 1 - ACQUISITION SIM
        - Simulate first contact        All
        - Basic health check            FC
        - Record beacon data            Ground

01:15   POST-PASS 1
        - Log pass results              Ground
        - Update status board           FC

02:00   QUIET PERIOD
        - Monitor autonomous ops        Ground
        - Review procedures             FC

04:00   PRE-PASS 2 PREP
        - Configure for commanding      Ground

04:30   PASS 2 - COMMANDING
        - Release from SAFE             FC
        - Transition to ACTIVE          FC
        - Enable AI payload             FC
        - Collect AI beacons            Ground

04:45   POST-PASS 2
        - Log results                   Ground
        - Assess AI content             FC

06:00   QUIET PERIOD
        - Extended monitoring           Ground
        - Shift prep                    Shift 2

08:00   SHIFT 2 START / SHIFT 1 END
        *** HANDOFF EXERCISE ***
        - Status briefing               Off-going
        - Open items                    Off-going
        - Acknowledgment                On-coming

08:30   PASS 3 - ROUTINE OPS
        - Health request                FC
        - Telemetry verification        Ground
        - Nominal operations            All

09:00   QUIET PERIOD
        - Training activity             FC
        - System monitoring             Ground

10:30   ANOMALY INJECTION #1
        *** COMM TIMEOUT SCENARIO ***
        - Simulate lost command         Test Director
        - Execute contingency           FC
        - Document response             All

11:00   RECOVERY ACTIVITIES
        - Re-establish nominal          FC
        - Document lessons              All

12:00   PASS 4 - RECOVERY VERIFY
        - Verify recovery complete      FC
        - Nominal operations            All
        - Collect status                Ground

13:00   QUIET PERIOD
        - Lunch rotation                Per plan
        - Continuous monitoring         Ground

14:30   ANOMALY INJECTION #2
        *** LOW POWER SCENARIO ***
        - Simulate SOC drop             Test Director
        - Observe auto-SAFE             FC
        - Execute recovery              FC

15:30   PASS 5 - POWER RECOVERY
        - Monitor power trends          FC
        - Plan mode restoration         FC

16:00   SHIFT 3 START / SHIFT 2 END
        *** HANDOFF EXERCISE ***
        - Status briefing               Off-going
        - Document anomalies            Off-going
        - Plan forward                  On-coming

17:00   QUIET PERIOD
        - Reduced manning ops           Ground
        - Documentation                 FC

19:00   PASS 6 - NIGHT OPS
        - Routine monitoring            FC
        - Minimal commanding            FC

20:00   QUIET PERIOD
        - Extended autonomous           Ground

22:00   PRE-WRAP ACTIVITIES
        - Status assessment             FC
        - Documentation review          Ground

23:30   DITL WRAP-UP
        - Return to safe config         FC
        - Final logs                    Ground

24:00   DITL COMPLETE
        - Debrief all shifts            Test Director
        - Collect feedback              All
```

### 4.2 Pass Schedule

| Pass | Time | Duration | Activity | Priority |
|------|------|----------|----------|----------|
| 1 | 01:00 | 10 min | Acquisition sim | Critical |
| 2 | 04:30 | 12 min | Commanding | High |
| 3 | 08:30 | 10 min | Routine | Normal |
| 4 | 12:00 | 10 min | Recovery verify | High |
| 5 | 15:30 | 8 min | Power recovery | High |
| 6 | 19:00 | 10 min | Night ops | Normal |

## 5. Shift Procedures

### 5.1 Shift 1 (00:00-08:00)

```
SHIFT 1 ACTIVITIES

Personnel:
- Mission Director: _________
- Flight Controller: _________
- Ground Controller: _________

Key Activities:
[ ] Initial status review
[ ] Pass 1 execution
[ ] Pass 2 execution (AI enable)
[ ] Prepare handoff package

Shift Log:
TIME   EVENT                    INITIALS
────────────────────────────────────────



Handoff Notes:
_________________________________
_________________________________
```

### 5.2 Shift 2 (08:00-16:00)

```
SHIFT 2 ACTIVITIES

Personnel:
- Mission Director: _________
- Flight Controller: _________
- Ground Controller: _________

Key Activities:
[ ] Receive handoff
[ ] Pass 3 execution
[ ] Anomaly injection #1
[ ] Anomaly injection #2
[ ] Pass 4, 5 execution
[ ] Prepare handoff

Shift Log:
TIME   EVENT                    INITIALS
────────────────────────────────────────



Handoff Notes:
_________________________________
```

### 5.3 Shift 3 (16:00-24:00)

```
SHIFT 3 ACTIVITIES

Personnel:
- Flight Controller: _________
- Ground Controller: _________

Key Activities:
[ ] Receive handoff
[ ] Reduced manning operations
[ ] Pass 6 execution
[ ] DITL closeout
[ ] Final documentation

Shift Log:
TIME   EVENT                    INITIALS
────────────────────────────────────────


```

## 6. Anomaly Injection Scripts

### 6.1 Anomaly #1: Comm Timeout

```
ANOMALY INJECTION #1

Scenario: Command sent but no acknowledgment
Inject Time: 10:30 (±15 min)

SETUP:
- Test Director blocks command response
- FC sends routine command

EXPECTED RESPONSE:
1. FC notices no ACK within timeout (30s)
2. FC consults contingency procedures (CP-003)
3. FC attempts retry per procedure
4. FC escalates if retries fail
5. Ground logs all actions
6. Mission Director notified

EVALUATION:
[ ] Timeout detected within 60s
[ ] Correct procedure referenced
[ ] Retry executed correctly
[ ] Escalation appropriate
[ ] Documentation complete

NOTES:
_________________________________
```

### 6.2 Anomaly #2: Low Power

```
ANOMALY INJECTION #2

Scenario: Battery SOC drops to critical level
Inject Time: 14:30 (±15 min)

SETUP:
- Test Director reduces simulated SOC to 18%
- Spacecraft should auto-transition to SAFE

EXPECTED RESPONSE:
1. Telemetry shows SOC < 20%
2. Spacecraft transitions to SAFE (auto)
3. FC confirms state change
4. FC consults contingency procedures (CP-002)
5. FC reports to Mission Director
6. Team plans recovery

EVALUATION:
[ ] State change detected
[ ] Correct root cause identified
[ ] Procedure followed
[ ] Recovery plan developed
[ ] Documentation complete

NOTES:
_________________________________
```

## 7. Handoff Procedures

### 7.1 Handoff Checklist

```
SHIFT HANDOFF CHECKLIST

Off-Going Shift: ___
On-Coming Shift: ___
Time: ___:___

STATUS BRIEFING:
[ ] Spacecraft state: _________
[ ] Current mode: _________
[ ] Last pass summary: _________
[ ] Battery SOC: _____%
[ ] AI status: _________
[ ] Any anomalies: _________

OPEN ITEMS:
[ ] _________________________________
[ ] _________________________________

FORWARD PLAN:
[ ] Next pass: ___:___ - Activity: _________
[ ] Key objectives: _________

HANDOFF COMPLETE:
Off-going: _________ Time: ___:___
On-coming: _________ Time: ___:___
```

## 8. Data Collection

### 8.1 Metrics to Collect

| Metric | Collection Method |
|--------|-------------------|
| Command count | Ground SW log |
| Beacon count | Decoder log |
| Procedure execution time | Manual timing |
| Anomaly response time | Shift logs |
| Handoff duration | Manual timing |

### 8.2 Data Sheets

```
DITL DATA SUMMARY

PASS METRICS:
Pass | Beacons | Commands | Errors | Duration
─────┼─────────┼──────────┼────────┼─────────
  1  |         |          |        |
  2  |         |          |        |
  3  |         |          |        |
  4  |         |          |        |
  5  |         |          |        |
  6  |         |          |        |

ANOMALY METRICS:
Anomaly | Detect Time | Response Time | Resolved
────────┼─────────────┼───────────────┼─────────
   1    |             |               |
   2    |             |               |

HANDOFF METRICS:
Handoff | Duration | Issues
────────┼──────────┼────────
  1→2   |          |
  2→3   |          |
```

## 9. Success Criteria

### 9.1 Test Success Criteria

| Criterion | Threshold |
|-----------|-----------|
| All passes executed | 100% |
| All procedures used | Yes |
| Anomalies detected | Within 5 min |
| Anomalies resolved | Per procedure |
| Handoffs complete | All |
| No safety issues | None |

### 9.2 Evaluation Criteria

| Area | Pass | Fail |
|------|------|------|
| Procedures | Executable as written | Require modification |
| Tools | Function correctly | Require fixes |
| Team | Execute operations | Unable to perform |
| Communications | Clear and effective | Breakdowns |
| Documentation | Complete and accurate | Missing or errors |

## 10. Post-Test Activities

### 10.1 Immediate Debrief

```
DITL DEBRIEF AGENDA

Date: ___________
Duration: 1-2 hours

PARTICIPANTS:
- All shift personnel
- Test Director
- Project Manager (optional)

TOPICS:
1. Overall assessment (10 min)
2. Shift 1 feedback (15 min)
3. Shift 2 feedback (15 min)
4. Shift 3 feedback (15 min)
5. Procedure issues (15 min)
6. Tool issues (10 min)
7. Training gaps (10 min)
8. Forward work (10 min)

ACTION ITEMS:
#  | Item          | Owner | Due
───┼───────────────┼───────┼─────
   |               |       |
   |               |       |
```

### 10.2 Report Contents

| Section | Content |
|---------|---------|
| Executive Summary | Overall pass/fail, key issues |
| Timeline Adherence | Planned vs. actual |
| Procedure Assessment | Each procedure used |
| Anomaly Response | Detection and resolution |
| Tool Assessment | Function and usability |
| Team Assessment | Performance and gaps |
| Recommendations | Improvements needed |

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial CDR baseline |
