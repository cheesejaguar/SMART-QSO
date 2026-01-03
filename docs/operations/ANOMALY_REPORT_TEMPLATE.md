# SMART-QSO Anomaly Report Template

**Document ID**: SQSO-AR-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document provides the template for Anomaly Reports per NASA mission operations practices. Anomaly reports document unexpected spacecraft behavior and track resolution.

---

## ANOMALY REPORT

### Report Header

```
┌────────────────────────────────────────────────────────────┐
│  ANOMALY REPORT                                            │
├────────────────────────────────────────────────────────────┤
│  Anomaly ID:      AR-XXXX                                  │
│  Date Discovered: YYYY-MM-DD HH:MM UTC                     │
│  Reported By:     ________________                         │
│                                                            │
│  Severity:        [ ] Critical  [ ] High  [ ] Medium       │
│                   [ ] Low       [ ] Observation            │
│                                                            │
│  Status:          [ ] Open  [ ] In Work  [ ] Closed        │
│                                                            │
│  Spacecraft State at Discovery: ________________           │
│  Pass ID:         ________________                         │
└────────────────────────────────────────────────────────────┘
```

### Anomaly Description

```
WHAT HAPPENED:
_______________________________________________________________
_______________________________________________________________
_______________________________________________________________

EXPECTED BEHAVIOR:
_______________________________________________________________
_______________________________________________________________

ACTUAL BEHAVIOR:
_______________________________________________________________
_______________________________________________________________

SYMPTOMS OBSERVED:
[ ] Unexpected telemetry value
[ ] Command not acknowledged
[ ] State change
[ ] Communication issue
[ ] Power anomaly
[ ] Thermal anomaly
[ ] Software behavior
[ ] Other: ________________
```

### Timeline

```
ANOMALY TIMELINE

Time (UTC)   Event
─────────── ─────────────────────────────────────────────────
            Last known good state
            First indication of anomaly
            Anomaly confirmed
            Initial response actions
            Current state
```

### Telemetry at Time of Anomaly

```
TELEMETRY SNAPSHOT

Parameter              Value      Expected    Status
────────────────────── ────────── ────────── ────────
STATE                                         [ ]
BATTERY_SOC                                   [ ]
BATTERY_V                                     [ ]
TEMP_OBC                                      [ ]
TEMP_BATTERY                                  [ ]
TX_POWER                                      [ ]
BOOT_COUNT                                    [ ]
FAULT_FLAGS                                   [ ]
```

### Impact Assessment

```
IMPACT ASSESSMENT

Mission Impact:
[ ] Mission threatening
[ ] Mission degrading
[ ] Capability reduced
[ ] No impact
[ ] TBD

Affected Systems:
[ ] Power
[ ] Thermal
[ ] Communications
[ ] OBC/Software
[ ] AI Payload
[ ] Structure
[ ] None determined

Workaround Available:
[ ] Yes - Describe: ________________________________
[ ] No
[ ] TBD
```

### Immediate Actions Taken

```
IMMEDIATE RESPONSE

Action                              Time       By      Result
─────────────────────────────────── ────────── ─────── ───────




Spacecraft safed: [ ] Yes [ ] No [ ] N/A
```

### Root Cause Analysis

```
ROOT CAUSE ANALYSIS

Probable Cause:
_______________________________________________________________
_______________________________________________________________

Contributing Factors:
_______________________________________________________________
_______________________________________________________________

Similar Previous Anomalies:
[ ] None known
[ ] AR-____ : ________________

Evidence:
_______________________________________________________________
_______________________________________________________________

Root Cause Category:
[ ] Hardware failure
[ ] Software defect
[ ] Operations error
[ ] Environment
[ ] Design limitation
[ ] Unknown
```

### Corrective Actions

```
CORRECTIVE ACTIONS

Immediate Actions (Completed):
1. _______________________________________________________________
2. _______________________________________________________________

Short-Term Actions (In Work):
1. _______________________________________________________________
   Owner: _____________ Due: _____________
2. _______________________________________________________________
   Owner: _____________ Due: _____________

Long-Term Actions (Planned):
1. _______________________________________________________________
   Owner: _____________ Due: _____________

Preventive Actions:
1. _______________________________________________________________
```

### Closure Information

```
ANOMALY CLOSURE

Resolution Summary:
_______________________________________________________________
_______________________________________________________________

Verification Method:
[ ] Telemetry review
[ ] Ground test
[ ] On-orbit verification
[ ] Analysis
[ ] Other: ________________

Lessons Learned:
_______________________________________________________________
_______________________________________________________________

Closed By: ________________ Date: ________________

Reviewed By: ________________ Date: ________________
```

### Attachments

```
ATTACHMENTS

[ ] Telemetry plots
[ ] Command logs
[ ] Pass timeline
[ ] As-run procedure
[ ] Photos/screenshots
[ ] Analysis reports
[ ] Other: ________________

Attachment Location: ________________________________
```

### Signatures

```
ANOMALY REPORT SIGNATURES

Reported By:   ________________ Date: ________

Analyzed By:   ________________ Date: ________

Approved By:   ________________ Date: ________
               (Mission Director)

Closed By:     ________________ Date: ________
```

---

## 2. Severity Definitions

| Severity | Definition | Response |
|----------|------------|----------|
| Critical | Mission loss imminent | Immediate action, all hands |
| High | Mission significantly degraded | Same day resolution |
| Medium | Reduced capability | Resolution within 1 week |
| Low | Minor issue | Scheduled resolution |
| Observation | No impact, noted for awareness | Log only |

## 3. Anomaly Process

```
ANOMALY PROCESS FLOW

1. Discovery
   └─ Recognize unexpected behavior
   └─ Initial documentation

2. Notification
   └─ Notify shift lead
   └─ Notify Mission Director (if High/Critical)
   └─ Create anomaly report

3. Safe the Spacecraft (if needed)
   └─ Execute contingency procedure
   └─ Stabilize spacecraft state

4. Investigation
   └─ Gather telemetry/data
   └─ Root cause analysis
   └─ Impact assessment

5. Resolution
   └─ Develop corrective action
   └─ Implement fix
   └─ Verify resolution

6. Closure
   └─ Document lessons learned
   └─ Close anomaly report
   └─ Brief team
```

## 4. Notification Matrix

| Severity | Immediate | Within 1 Hour | Within 24 Hours |
|----------|-----------|---------------|-----------------|
| Critical | Shift Lead, MD, PM | All team | Stakeholders |
| High | Shift Lead, MD | PM | Team |
| Medium | Shift Lead | MD | PM |
| Low | - | Shift Lead | MD |
| Observation | - | - | Log only |

## 5. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial template |
