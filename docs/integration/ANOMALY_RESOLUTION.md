# Anomaly Resolution Procedure
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-INT-005
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7120.5, NASA Ames Quality Assurance

---

## 1. Purpose

This document defines the procedure for identifying, documenting, investigating, and resolving anomalies during SMART-QSO development, integration, test, and operations.

## 2. Scope

This procedure applies to:
- Hardware anomalies
- Software anomalies
- Test anomalies
- Integration anomalies
- On-orbit anomalies

## 3. Definitions

| Term | Definition |
|------|------------|
| Anomaly | Unexpected behavior or non-conformance |
| Critical | Mission-threatening, immediate action required |
| Major | Significant impact, timely resolution required |
| Minor | Limited impact, scheduled resolution |
| Root Cause | Fundamental reason for the anomaly |
| Corrective Action | Action to prevent recurrence |

## 4. Anomaly Classification

### 4.1 Severity Levels

| Level | Name | Criteria | Response Time |
|-------|------|----------|---------------|
| 1 | Critical | Mission loss, safety hazard | Immediate |
| 2 | Major | Significant degradation | 24 hours |
| 3 | Minor | Limited impact | 1 week |
| 4 | Observation | No impact, informational | As scheduled |

### 4.2 Classification Matrix

| Impact \ Likelihood | Frequent | Probable | Occasional | Remote |
|---------------------|----------|----------|------------|--------|
| Catastrophic | 1 | 1 | 1 | 2 |
| Critical | 1 | 1 | 2 | 2 |
| Marginal | 2 | 2 | 3 | 3 |
| Negligible | 3 | 3 | 4 | 4 |

## 5. Anomaly Report Form

### 5.1 Anomaly Identification

```
ANOMALY REPORT

AR Number: AR-XXXX-NNN
Date Opened: ____________
Reported By: ____________
Phase: ☐ Development ☐ Integration ☐ Test ☐ Operations

Title: _________________________________________________

Description:
____________________________________________________________
____________________________________________________________
____________________________________________________________

Subsystem(s) Affected:
☐ EPS ☐ OBC ☐ ADCS ☐ RF ☐ Payload ☐ Structure ☐ SW ☐ Other

Severity: ☐ 1-Critical ☐ 2-Major ☐ 3-Minor ☐ 4-Observation

Evidence/Data:
____________________________________________________________
```

### 5.2 Investigation Section

```
INVESTIGATION

Investigator: ____________
Date Started: ____________
Date Completed: ____________

Investigation Steps:
1. ________________________________________________________
2. ________________________________________________________
3. ________________________________________________________

Root Cause:
____________________________________________________________
____________________________________________________________

Contributing Factors:
____________________________________________________________
```

### 5.3 Resolution Section

```
RESOLUTION

Corrective Action:
____________________________________________________________
____________________________________________________________

Verification Method:
☐ Test ☐ Analysis ☐ Inspection ☐ Demonstration

Verification Results:
____________________________________________________________

Preventive Action:
____________________________________________________________

Closure Date: ____________
Closed By: ____________
```

## 6. Anomaly Process Flow

```
┌──────────────────┐
│  Anomaly         │
│  Detected        │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Document        │
│  Anomaly         │
│  (AR Form)       │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Classify        │
│  Severity        │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌────────┐ ┌────────────┐
│Sev 1-2 │ │  Sev 3-4   │
│Escalate│ │  Normal    │
└───┬────┘ └─────┬──────┘
    │            │
    └─────┬──────┘
          │
          ▼
┌──────────────────┐
│  Investigate     │
│  Root Cause      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Develop         │
│  Corrective      │
│  Action          │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Implement       │
│  & Verify        │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  Close           │
│  Anomaly         │
└──────────────────┘
```

## 7. Roles and Responsibilities

### 7.1 Reporter
- Document anomaly accurately
- Provide supporting evidence
- Preserve anomaly conditions

### 7.2 Project Engineer
- Assign severity
- Assign investigator
- Track to closure

### 7.3 Investigator
- Conduct root cause analysis
- Develop corrective action
- Verify resolution

### 7.4 Quality Assurance
- Verify process followed
- Trend anomalies
- Report metrics

### 7.5 Project Manager
- Review Severity 1-2 anomalies
- Approve major corrective actions
- Chair Anomaly Review Board (ARB)

## 8. Investigation Techniques

### 8.1 Root Cause Analysis Methods

| Method | When to Use |
|--------|-------------|
| 5 Whys | Simple problems |
| Fishbone (Ishikawa) | Multiple factors |
| Fault Tree | Safety-critical |
| Timeline Analysis | Sequence-dependent |
| Change Analysis | Recent changes |

### 8.2 5 Whys Example

```
Problem: Beacon not received

Why 1: Transmitter not keying
Why 2: Enable signal not asserted
Why 3: Timer not expiring
Why 4: Timer value set to 0
Why 5: Configuration not loaded after reset

Root Cause: Missing configuration persistence
```

### 8.3 Data Collection

| Data Type | Purpose |
|-----------|---------|
| Telemetry | System state at failure |
| Logs | Event sequence |
| Photos | Physical evidence |
| Test data | Comparison to nominal |
| Configuration | Settings at failure |

## 9. Corrective Action Types

### 9.1 Immediate (Containment)
- Stop the damage
- Protect hardware/mission
- Temporary workaround

### 9.2 Root Cause Correction
- Fix the fundamental problem
- Prevent this occurrence

### 9.3 Preventive (Systemic)
- Prevent similar problems
- Process improvement
- Lessons learned

## 10. Verification Requirements

### 10.1 By Severity

| Severity | Verification Required |
|----------|----------------------|
| 1 | Test + independent review |
| 2 | Test or analysis + review |
| 3 | Test or inspection |
| 4 | Inspection or observation |

### 10.2 Test Requirements

- Reproduce original failure mode
- Verify fix resolves issue
- Regression test related functions
- Document results

## 11. Anomaly Tracking

### 11.1 Tracking Database

| Field | Purpose |
|-------|---------|
| AR Number | Unique identifier |
| Status | Open/In-Work/Closed |
| Severity | Priority |
| Subsystem | Categorization |
| Assignee | Responsibility |
| Due Date | Schedule |

### 11.2 Status Definitions

| Status | Definition |
|--------|------------|
| Open | New, awaiting investigation |
| In-Work | Under investigation |
| Ready to Verify | Fix implemented |
| Verified | Fix confirmed |
| Closed | Complete |
| Deferred | Postponed (with rationale) |

## 12. Reporting

### 12.1 Weekly Report

| Metric | Value |
|--------|-------|
| New anomalies | |
| Closed anomalies | |
| Open (total) | |
| Overdue | |
| Average age | |

### 12.2 Trends

- Anomalies by subsystem
- Anomalies by phase
- Closure rate
- Escape rate

## 13. On-Orbit Anomalies

### 13.1 Additional Considerations

| Factor | Consideration |
|--------|---------------|
| Limited data | Maximize telemetry capture |
| No physical access | Must resolve via software/ops |
| Mission impact | Balance risk of action |
| Time criticality | Orbital dynamics constraints |

### 13.2 On-Orbit Process

1. Capture all available data
2. Convene anomaly team
3. Develop fault tree
4. Identify possible actions
5. Assess risk of each action
6. Implement safest resolution
7. Monitor results
8. Document lessons learned

## 14. Closure Criteria

### 14.1 Required for Closure

- Root cause identified
- Corrective action implemented
- Verification complete
- Documentation complete
- Lessons learned captured
- QA concurrence

### 14.2 Closure Approval

| Severity | Approval Authority |
|----------|-------------------|
| 1 | Project Manager + ARB |
| 2 | Project Engineer |
| 3 | Subsystem Lead |
| 4 | Investigator |

---

## Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
