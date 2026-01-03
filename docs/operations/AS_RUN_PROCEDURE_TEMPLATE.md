# SMART-QSO As-Run Procedure Template

**Document ID**: SQSO-ARP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the template for As-Run Procedures per NASA operations heritage. As-Run procedures document the actual execution of procedures including deviations, timing, and results.

---

## AS-RUN PROCEDURE

### Header Information

```
┌────────────────────────────────────────────────────────────┐
│  AS-RUN PROCEDURE                                          │
├────────────────────────────────────────────────────────────┤
│  Procedure ID:     ________________                        │
│  Procedure Title:  ________________                        │
│  Procedure Rev:    ________________                        │
│                                                            │
│  Execution Date:   ________________                        │
│  Start Time (UTC): ________________                        │
│  End Time (UTC):   ________________                        │
│                                                            │
│  Executor:         ________________                        │
│  Verifier:         ________________                        │
│                                                            │
│  Pass ID:          ________________                        │
│  Spacecraft State: ________________                        │
└────────────────────────────────────────────────────────────┘
```

### Pre-Procedure Checklist

```
PRE-PROCEDURE VERIFICATION

[ ] Procedure current revision verified
[ ] Prerequisites satisfied
[ ] Required equipment available
[ ] Required personnel present
[ ] Spacecraft in required state
[ ] Ground system ready
[ ] Logging enabled

Pre-conditions verified by: _________ Time: _____
```

### Procedure Steps (Example Format)

```
STEP  TIME    EXEC  ACTION                      RESULT/NOTES
────  ─────── ──── ─────────────────────────── ──────────────
1.1   ___:___ [ ]  Verify ground station ready  [ ] GO
                   Expected: All systems green
                   Actual: _________________

1.2   ___:___ [ ]  Verify spacecraft state      [ ] GO
                   Expected: STATE = ________
                   Actual: _________________

1.3   ___:___ [ ]  Send command: ____________   [ ] ACK
                   Expected: ACK within 5s
                   Actual: _________________

1.4   ___:___ [ ]  Verify telemetry             [ ] NOMINAL
                   Expected: _________________
                   Actual: _________________

1.5   ___:___ [ ]  [Continue with steps...]     [ ] _______
```

### Contingency Actions (If Any)

```
CONTINGENCY LOG

Step | Time    | Issue Description          | Action Taken
─────┼─────────┼────────────────────────────┼──────────────
     |         |                            |
     |         |                            |
```

### Deviations from Procedure

```
DEVIATION LOG

Step | Deviation Description              | Approval
─────┼────────────────────────────────────┼──────────────
     |                                    | [ ] MD
     |                                    | [ ] MD
     |                                    | [ ] MD

Deviation authority: Mission Director verbal approval
required for any deviation from approved procedure.
```

### Data Collected

```
DATA COLLECTION LOG

Parameter          | Time    | Value    | Expected  | Status
───────────────────┼─────────┼──────────┼───────────┼────────
Battery SOC        |         |          |           | [ ] OK
Battery Voltage    |         |          |           | [ ] OK
OBC Temperature    |         |          |           | [ ] OK
State              |         |          |           | [ ] OK
                   |         |          |           |
```

### Commands Sent

```
COMMAND LOG

Time (UTC) | Command          | Parameters | Response | Status
───────────┼──────────────────┼────────────┼──────────┼────────
           |                  |            |          | [ ] OK
           |                  |            |          | [ ] OK
           |                  |            |          | [ ] OK
```

### Post-Procedure Checklist

```
POST-PROCEDURE VERIFICATION

[ ] All steps completed or properly terminated
[ ] Spacecraft in expected final state
[ ] All data archived
[ ] Anomalies documented
[ ] Handoff complete (if applicable)

Final state verified by: _________ Time: _____
```

### Procedure Summary

```
EXECUTION SUMMARY

Start Time:      ___:___:___ UTC
End Time:        ___:___:___ UTC
Duration:        ___:___

Steps Completed: ___ of ___
Steps Skipped:   ___ (with approval)
Deviations:      ___
Anomalies:       ___

Overall Result:  [ ] SUCCESS  [ ] PARTIAL  [ ] ABORT

Comments:
_________________________________________________
_________________________________________________
```

### Signatures

```
PROCEDURE CLOSURE

Executor:    _______________ Date: _______ Time: _____

Verifier:    _______________ Date: _______ Time: _____

Reviewed by: _______________ Date: _______ Time: _____
             (Shift Lead)
```

---

## 2. Instructions for Use

### 2.1 When to Use As-Run

| Procedure Type | As-Run Required |
|----------------|-----------------|
| Routine pass | Yes |
| Commanding | Yes |
| Contingency | Yes |
| Test execution | Yes |
| Configuration change | Yes |

### 2.2 Documentation Requirements

| Element | Requirement |
|---------|-------------|
| Time | UTC, HH:MM:SS |
| Steps | Mark complete or N/A |
| Values | Record actual values |
| Deviations | Document with approval |
| Anomalies | Separate anomaly report |

### 2.3 Archival

| After Execution | Action |
|-----------------|--------|
| Immediate | Review for anomalies |
| Within 24 hours | File in ops archive |
| Weekly | Batch review |
| Monthly | Trend analysis |

### 2.4 Version Control

| Procedure Change | As-Run Action |
|------------------|---------------|
| New revision | Use latest approved |
| Pen-and-ink change | Note change in As-Run |
| Real-time change | MD approval + note |

## 3. Example As-Run

### 3.1 Nominal Pass Example

```
AS-RUN: NOMINAL PASS OPERATIONS
Procedure: OP-PASS-001 Rev 1.0
Date: 2026-01-15
Pass: #0147

STEP  TIME     EXEC  ACTION                    RESULT
────  ──────── ──── ───────────────────────── ─────────
1.1   14:32:15 [X]  Ground station ready       GO
1.2   14:32:30 [X]  Await AOS                  14:33:00
1.3   14:33:05 [X]  Signal acquired            -95 dBm
1.4   14:33:20 [X]  First beacon decoded       OK
1.5   14:33:45 [X]  Verify health telemetry    NOMINAL
1.6   14:34:00 [X]  Send REQUEST_TLM           ACK
1.7   14:34:10 [X]  Receive full telemetry     OK
1.8   14:42:00 [X]  LOS                        14:42:15
1.9   14:42:30 [X]  Archive data               Complete

SUMMARY:
Duration: 10:15
Beacons received: 10
Commands sent: 1
Result: SUCCESS

Executor: J. Smith    14:45:00
Verifier: A. Jones    14:46:00
```

## 4. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
