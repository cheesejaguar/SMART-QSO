# SMART-QSO Lessons Learned

**Document ID**: SQSO-LL-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: Living Document

## 1. Purpose

This document captures lessons learned during the SMART-QSO mission development per NPR 7120.6 (Lessons Learned) requirements. It serves as institutional knowledge for future missions.

## 2. Lessons Learned Process

### 2.1 Collection Process

```
LESSONS LEARNED LIFECYCLE

1. IDENTIFY
   - Team member identifies lesson
   - Documents in LL submission form

2. REVIEW
   - Project management reviews
   - Validates applicability
   - Assigns category and priority

3. DOCUMENT
   - Added to this document
   - Cross-referenced to phase
   - Root cause identified

4. SHARE
   - Distributed to team
   - Submitted to NASA LLIS
   - Briefed at reviews

5. APPLY
   - Incorporated into processes
   - Updated procedures
   - Training updated
```

### 2.2 Categories

| Category | Description |
|----------|-------------|
| Technical | Hardware, software, design |
| Programmatic | Schedule, cost, management |
| Process | Procedures, reviews, testing |
| Team | Communication, training, resources |

### 2.3 Priority Levels

| Priority | Definition |
|----------|------------|
| High | Significantly impacted mission/schedule |
| Medium | Caused rework or inefficiency |
| Low | Minor improvement opportunity |

## 3. Development Phase Lessons

### 3.1 Requirements Phase

#### LL-001: Early AI Feasibility Assessment
**Category**: Technical | **Priority**: High

**Observation**: Initial AI payload requirements were ambitious given CubeSat power constraints.

**Impact**: Required multiple design iterations to balance AI capability vs. power budget.

**Root Cause**: Insufficient early analysis of power-limited inference scenarios.

**Recommendation**: Conduct detailed power budget analysis before finalizing AI requirements. Include duty cycle constraints in initial trade studies.

**Application**: Power-aware design pattern adopted. AI duty cycle limited to 10%.

---

#### LL-002: Amateur Radio Regulatory Clarity
**Category**: Process | **Priority**: Medium

**Observation**: Amateur radio regulations (47 CFR Part 97) require specific operational constraints not initially captured in requirements.

**Impact**: Late requirements additions for open protocols, no encryption, operator licensing.

**Root Cause**: Regulatory review conducted after initial requirements freeze.

**Recommendation**: Engage regulatory experts during requirements phase. Include regulatory constraints in ConOps from the start.

**Application**: Added regulatory compliance section to SRS. Created dedicated REGULATORY.md document.

---

### 3.2 Design Phase

#### LL-003: Graceful Degradation Architecture
**Category**: Technical | **Priority**: High

**Observation**: Designing for graceful degradation from the start simplified fault handling significantly.

**Impact**: Positive - reduced fault handling complexity, improved reliability.

**Root Cause**: Deliberate architecture decision following NASA Ames heritage.

**Recommendation**: Always design primary functions to operate without optional capabilities. Never make mission success dependent on experimental features.

**Application**: Flight software operates fully without AI payload. Template beacons available as fallback.

---

#### LL-004: State Machine Simplicity
**Category**: Technical | **Priority**: Medium

**Observation**: Three-state model (SAFE/IDLE/ACTIVE) proved sufficient for all operational scenarios.

**Impact**: Positive - reduced testing complexity, easier to verify.

**Root Cause**: Deliberate simplification based on SmallSat heritage.

**Recommendation**: Start with minimal state machine. Add states only when operationally necessary, not for design elegance.

**Application**: Avoided separate COMMISSIONING, ECLIPSE, SCIENCE states.

---

#### LL-005: Hardware Abstraction Layer
**Category**: Technical | **Priority**: Medium

**Observation**: Clean HAL separation enabled parallel software/hardware development.

**Impact**: Positive - software development proceeded without target hardware.

**Root Cause**: Deliberate architecture decision.

**Recommendation**: Define HAL interfaces early. Use stubs for all hardware dependencies. Enables comprehensive unit testing without hardware.

**Application**: All sensor reads, actuator writes abstracted. 300+ unit tests run without hardware.

---

### 3.3 Implementation Phase

#### LL-006: Test-Driven Development Value
**Category**: Process | **Priority**: High

**Observation**: Writing tests before implementation caught design issues early.

**Impact**: Reduced integration issues. Higher confidence in software quality.

**Root Cause**: TDD methodology adoption.

**Recommendation**: Mandate unit tests for all safety-critical functions. Target >80% coverage. Integrate testing into CI pipeline.

**Application**: 478+ tests across flight, payload, ground software.

---

#### LL-007: MISRA C Compliance Effort
**Category**: Technical | **Priority**: Medium

**Observation**: MISRA C:2012 compliance required significant upfront effort but caught subtle bugs.

**Impact**: Cleaner code, fewer runtime issues, but initial velocity slower.

**Root Cause**: Coding standard enforcement.

**Recommendation**: Adopt coding standard from day one. Configure static analysis in CI. Address issues immediately, not in batches.

**Application**: Cppcheck integration with MISRA rules. Zero violations in flight software.

---

#### LL-008: Documentation as Code
**Category**: Process | **Priority**: Low

**Observation**: Keeping documentation in repository with code improved consistency.

**Impact**: Documentation always in sync with code. Review process unified.

**Root Cause**: Git-based documentation workflow.

**Recommendation**: Store all project documentation in version control. Use markdown for portability. Review docs with code reviews.

**Application**: All 40+ documents in docs/ directory under Git control.

---

### 3.4 Testing Phase

#### LL-009: Flatsat Test Procedure Detail
**Category**: Process | **Priority**: Medium

**Observation**: Detailed step-by-step procedures with explicit pass/fail criteria essential for repeatable testing.

**Impact**: Reduced test ambiguity. Enabled delegation to operators.

**Root Cause**: Learned from NASA Ames test heritage.

**Recommendation**: Write procedures as if operator has no prior knowledge. Include expected values, tolerances, and contingencies.

**Application**: 7 flatsat procedures with detailed steps and acceptance criteria.

---

#### LL-010: Extended Duration Testing
**Category**: Technical | **Priority**: High

**Observation**: Week-long stability testing essential for finding memory leaks and timing drift.

**Impact**: Would have missed subtle resource exhaustion issues with shorter tests.

**Root Cause**: NASA Ames SmallSat heritage (PharmaSat experience).

**Recommendation**: Always include extended duration test (≥7 days). Monitor memory, timing, and cumulative errors.

**Application**: TP-007 extended duration test included in flatsat procedures.

---

## 4. Lessons from NASA Ames Heritage

### 4.1 GeneSat Lessons Applied

| Lesson | Application to SMART-QSO |
|--------|--------------------------|
| Simple robust design | Three-state machine |
| Extensive ground testing | 7 flatsat test procedures |
| Clear success criteria | Defined for each test |

### 4.2 PharmaSat Lessons Applied

| Lesson | Application to SMART-QSO |
|--------|--------------------------|
| Power margin importance | 50% margin in budget |
| Thermal management | Active monitoring, throttling |
| Long-duration testing | Week-long stability test |

### 4.3 O/OREOS Lessons Applied

| Lesson | Application to SMART-QSO |
|--------|--------------------------|
| Fault tolerance | Watchdog, safe mode, fallback |
| Telemetry completeness | 60+ parameters |
| Ground ops simplicity | Simple command set |

### 4.4 EcAMSat Lessons Applied

| Lesson | Application to SMART-QSO |
|--------|--------------------------|
| Software quality focus | MISRA C, 80% coverage |
| Operator training | 7 training modules |
| Documentation standards | Comprehensive doc set |

## 5. Lessons Submission Form

```
LESSONS LEARNED SUBMISSION

Date: ___________
Submitter: ___________
Phase: [ ] Requirements [ ] Design [ ] Implementation [ ] Test [ ] Ops

CATEGORY: [ ] Technical [ ] Programmatic [ ] Process [ ] Team

PRIORITY: [ ] High [ ] Medium [ ] Low

TITLE: _________________________

OBSERVATION:
What happened? What was observed?
_________________________________
_________________________________

IMPACT:
What was the impact (positive or negative)?
_________________________________
_________________________________

ROOT CAUSE:
Why did this happen?
_________________________________

RECOMMENDATION:
What should be done differently?
_________________________________
_________________________________

ACTION TAKEN:
What action was taken on this project?
_________________________________

APPLICABLE TO:
[ ] This project only
[ ] Similar CubeSat projects
[ ] All spacecraft projects
```

## 6. Lessons Tracking

### 6.1 Lessons Summary

| Phase | High | Medium | Low | Total |
|-------|------|--------|-----|-------|
| Requirements | 1 | 1 | 0 | 2 |
| Design | 2 | 1 | 0 | 3 |
| Implementation | 1 | 2 | 1 | 4 |
| Testing | 1 | 1 | 0 | 2 |
| **Total** | **5** | **5** | **1** | **11** |

### 6.2 Lessons Status

| ID | Title | Status |
|----|-------|--------|
| LL-001 | Early AI Feasibility | Applied |
| LL-002 | Regulatory Clarity | Applied |
| LL-003 | Graceful Degradation | Applied |
| LL-004 | State Machine Simplicity | Applied |
| LL-005 | Hardware Abstraction | Applied |
| LL-006 | Test-Driven Development | Applied |
| LL-007 | MISRA Compliance | Applied |
| LL-008 | Documentation as Code | Applied |
| LL-009 | Test Procedure Detail | Applied |
| LL-010 | Extended Duration Test | Applied |

## 7. NASA LLIS Submission

Per NPR 7120.6, significant lessons will be submitted to NASA Lessons Learned Information System (LLIS):

| Lesson | LLIS Candidate | Status |
|--------|----------------|--------|
| LL-001 | Yes | Pending submission |
| LL-003 | Yes | Pending submission |
| LL-006 | Yes | Pending submission |
| LL-010 | Yes | Pending submission |

## 8. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
