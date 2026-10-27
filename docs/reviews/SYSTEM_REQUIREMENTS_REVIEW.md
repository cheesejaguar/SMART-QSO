# SMART-QSO System Requirements Review Package

> **Funding audit notice (2026-10-14): internal self-assessment, not an independent review.**
> This record was generated on 2026-01-02 without named independent reviewers, signatures, or a
> RID/RFA log. Its PASS/APPROVED/COMPLETE statements are **not substantiated** and must not be cited
> to funders or regulators as review outcomes. See `docs/funding_audit/FUNDING_AUDIT_REPORT.md` and
> finding F-PI-03 in `docs/funding_audit/FINDINGS_REGISTER.md`.

**Document ID**: SQSO-SRR-001
**Version**: 1.1
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: Completed (Reference)

## 1. Purpose

This document provides the System Requirements Review (SRR) package for SMART-QSO per NPR 7123.1. SRR demonstrates that mission requirements are complete, consistent, and achievable.

**Note**: This review was completed as part of Phase A/B activities. This document is retained for reference and traceability.

## 2. Review Objectives

### 2.1 SRR Success Criteria

| Criterion | Status |
|-----------|--------|
| Mission objectives defined | [x] Complete |
| System requirements baselined | [x] Complete |
| ConOps approved | [x] Complete |
| Preliminary design feasible | [x] Complete |
| Risks identified | [x] Complete |
| Resources adequate | [x] Complete |

### 2.2 Entrance Criteria (Were)

- [x] Mission objectives defined
- [x] Stakeholder needs identified
- [x] Preliminary ConOps drafted
- [x] Trade studies complete
- [x] System requirements drafted

## 3. Mission Definition

### 3.1 Mission Objectives

| ID | Objective | Verification |
|----|-----------|--------------|
| MO-1 (Rev C) | Demonstrate AI-managed amateur radio operations from LEO | AI-generated beacons transmitted |
| MO-2 (Rev C) | Provide amateur radio operators with transponder access | Successful QSOs logged |
| MO-3 (Rev C) | Demonstrate graceful degradation to conventional operation | Transponder and template beacon continue with the AI payload off (SYS-521, SYS-522) |
| MO-4 (Rev C) | Advance TinyML/TinyLM technology for space applications | Not stated in SQSO-REQ-001 |
| MO-5 (Rev C) | Engage amateur radio community in space communications | Not stated in SQSO-REQ-001 |

*The objectives above are as stated in SQSO-REQ-001 Rev C at the time of this record; SQSO-REQ-001 Rev D (2026-10-27) restated MO-1 to MO-4 to match the baseline, so the old rows carry the suffix "(Rev C)".*

Objectives corrected 2026-10-16 to SQSO-REQ-001 §4.2. This review listed "operate autonomously for 2 years" and "comply
with amateur regulations" as MO-3 and MO-4; those are mission requirements (MRD-001, MRD-002; MRD-024, MRD-090 to
MRD-092).

### 3.2 Mission Success Criteria

| Level | Criteria | Duration |
|-------|----------|----------|
| Level 1 (Minimum) | Beacon operations | 30 days |
| Level 2 (Full) | AI-assisted service | 1 year |
| Level 3 (Extended) | Continuous service | 2+ years |

### 3.3 Concept of Operations Summary

```
MISSION PHASES

Phase 1: Launch & Early Ops (L+0 to L+14)
├── Deployment from launch vehicle
├── Initial acquisition
├── Health assessment
├── Basic beacon operations
└── Commissioning activities

Phase 2: Primary Operations (L+14 to L+2 years)
├── AI-assisted beacon generation
├── Autonomous operations
├── Amateur radio service
└── Periodic ground contact

Phase 3: Extended Operations (L+2 years to EOL)
├── Continued operations
├── Graceful degradation
└── Deorbit preparation

Phase 4: End of Life
├── Passive deorbit
└── Atmospheric reentry
```

## 4. System Requirements Summary

### 4.1 Functional Requirements

| Category | Count | Key Requirements |
|----------|-------|------------------|
| AI Payload | 7 | TinyLM inference, duty cycle, fallback |
| RF System | 7 | Beacon, AX.25, power control |
| Power System | 6 | Mode power, battery protection |
| Fault Tolerance | 7 | Watchdog, safe mode, logging |
| Regulatory | 3 | No encryption, ID, power limits |
| **Total** | **30** | |

### 4.2 Performance Requirements

| Parameter | Requirement | Margin |
|-----------|-------------|--------|
| Beacon interval | 60±1 s | ±1.7% |
| Boot time | <5 s | TBD |
| Command latency | <1 s | TBD |
| TX power | 0.5-2W | Per design |
| AI duty cycle | ≤10% | Per thermal |

### 4.3 Interface Requirements

| Interface | From | To | Type |
|-----------|------|-----|------|
| I2C | OBC | EPS | Data |
| UART | OBC | Jetson | Data |
| SPI | OBC | Radio | Data |
| RF | Spacecraft | Ground | Wireless |

## 5. Trade Studies Completed

### 5.1 Major Trades

| Trade | Options | Selected | Rationale |
|-------|---------|----------|-----------|
| AI Platform | MCU/TPU/GPU | Jetson Nano | Performance/power |
| OBC | STM32/Apollo4 | Apollo4 | Ultra-low power |
| Radio | Integrated/SDR | Si4463 | Flight heritage |
| Antenna | Dipole/Monopole | Tape spring | CubeSat standard |

### 5.2 Trade Documentation

| Trade Study | Document | Status |
|-------------|----------|--------|
| AI Platform Selection | ARCH-TS-001 | [x] Complete |
| Power Architecture | ARCH-TS-002 | [x] Complete |
| RF System Selection | ARCH-TS-003 | [x] Complete |
| Form Factor | ARCH-TS-004 | [x] Complete |

## 6. Preliminary Design Assessment

### 6.1 Design Feasibility

| Aspect | Assessment | Confidence |
|--------|------------|------------|
| Mass budget | 33% margin | High |
| Power budget | 128% margin (worst) | High |
| Volume | Fits 1U | High |
| Thermal | All within limits | Medium |
| Link budget | Positive margins | Medium |

### 6.2 Technology Readiness

| Component | TRL at SRR | TRL Target |
|-----------|------------|------------|
| OBC | 6 | 7 |
| EPS | 7 | 8 |
| Radio | 7 | 8 |
| Jetson | 5 | 6 |
| Structure | 8 | 9 |

### 6.3 Development Risks at SRR

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AI power consumption | Medium | Medium | Duty cycle limiting |
| Thermal management | Medium | Medium | Analysis and testing |
| Schedule | Low | Medium | Conservative planning |

## 7. Resource Assessment

### 7.1 Cost Estimate

| Category | Estimate | Confidence |
|----------|----------|------------|
| Hardware | $XX,XXX | Medium |
| Software | $XX,XXX | Medium |
| Integration | $XX,XXX | Medium |
| Testing | $XX,XXX | Medium |
| Operations | $XX,XXX | Low |
| **Total** | $XXX,XXX | Medium |

### 7.2 Schedule Assessment

| Phase | Duration | Status |
|-------|----------|--------|
| Phase A | 3 months | [x] Complete |
| Phase B | 6 months | [x] Complete |
| Phase C | 9 months | [x] In progress |
| Phase D | 3 months | [ ] Planned |
| Phase E | 24 months | [ ] Planned |

### 7.3 Team Assessment

| Role | Assigned | Status |
|------|----------|--------|
| Project Manager | Yes | Active |
| Systems Engineer | Yes | Active |
| Software Lead | Yes | Active |
| Hardware Lead | Yes | Active |
| Operations Lead | Yes | Active |

## 8. SRR Findings Summary

### 8.1 Findings (Historical)

| ID | Finding | Resolution | Status |
|----|---------|------------|--------|
| SRR-001 | AI power TBD | Analysis complete | [x] Closed |
| SRR-002 | Thermal analysis needed | Analysis complete | [x] Closed |
| SRR-003 | Link margin TBD | Budget complete | [x] Closed |

### 8.2 Actions (Historical)

| ID | Action | Owner | Status |
|----|--------|-------|--------|
| SRR-A001 | Complete power analysis | SE | [x] Closed |
| SRR-A002 | Complete thermal analysis | SE | [x] Closed |
| SRR-A003 | Baseline requirements | SE | [x] Closed |

## 9. SRR Board Decision (Historical)

```
SYSTEM REQUIREMENTS REVIEW DECISION

Date: [Historical]

The SRR Board reviewed mission requirements and
determined:

[x] PROCEED TO PDR
    - Requirements complete and consistent
    - Preliminary design feasible
    - Resources adequate

[ ] CONDITIONAL PROCEED
[ ] NOT READY

This review has been completed and closed.
All actions resolved by PDR.
```

## 10. Traceability to Later Reviews

### 10.1 SRR to PDR

| SRR Deliverable | PDR Status |
|-----------------|------------|
| Mission requirements | Baselined |
| System requirements | Refined |
| ConOps | Approved |
| Preliminary design | Detailed |

### 10.2 SRR to CDR

| SRR Element | CDR Status |
|-------------|------------|
| Requirements | Verified |
| Design | Complete |
| Risks | Mitigated |
| Resources | On track |

## 11. Lessons for Future Missions

### 11.1 SRR Process Lessons

| Lesson | Recommendation |
|--------|----------------|
| Early risk ID valuable | Start risk register at concept |
| Trade studies important | Complete before SRR |
| Stakeholder alignment | Engage early and often |
| Requirements clarity | Use SHALL/SHOULD/MAY |

### 11.2 Technical Lessons

| Area | Lesson |
|------|--------|
| AI integration | Start TRL advancement early |
| Power budgets | Include margin from start |
| Thermal | Begin analysis at SRR |

## 12. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Reference document |
| 1.1 | 2026-10-16 | SMART-QSO Team | §3.1 objectives corrected to SQSO-REQ-001 §4.2; MO-5 added (finding VV-06) |
