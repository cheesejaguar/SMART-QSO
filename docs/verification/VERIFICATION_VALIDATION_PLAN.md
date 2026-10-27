# SMART-QSO Verification and Validation Plan

**Document ID**: SQSO-VVP-001
**Version**: 1.3
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: Draft (was labelled "CDR Baseline"; no CDR has been held, BASELINE §1)

> **CDR review note (2026-10-27).** This plan keeps the V&V process (methods, levels, flow). The test levels,
> articles, flow, facilities and verification items are in `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C; the
> method, planned event and status of every requirement are in the VCRM, `docs/verification/VERIFICATION_MATRIX.md`
> Rev C; the dates are in `mission/SCHEDULE.md`. Where this plan differs from them, they govern.
> `mission/VNV_PLAN.md` is superseded by this plan.

## 1. Purpose

This document defines the Verification and Validation (V&V) Plan for SMART-QSO per NPR 7123.1 §4.6, establishing the approach, methods, and processes for verifying requirements and validating mission success.

## 2. Scope

### 2.1 Verification Scope

| Level | Scope |
|-------|-------|
| Component | Individual hardware/software elements |
| Subsystem | Integrated subsystem function |
| System | Complete spacecraft |
| Mission | End-to-end operations |

### 2.2 Items to be Verified

| Item | Requirements | Verification Level |
|------|--------------|-------------------|
| Flight Software | 30 SWRs | Component, System |
| OBC Hardware | 8 HWRs | Component, Subsystem |
| EPS | 6 HWRs | Component, Subsystem |
| RF System | 7 HWRs | Component, System |
| AI Payload | 7 HWRs | Component, Subsystem |
| Spacecraft | 30 SYS reqs | System |

## 3. Verification Approach

### 3.1 Verification Methods

| Method | Code | Description | Usage |
|--------|------|-------------|-------|
| Test | T | Execute and measure | Primary method |
| Analysis | A | Calculate/model | When test impractical |
| Inspection | I | Visual examination | Workmanship, config |
| Demonstration | D | Show capability | Procedures |

### 3.2 Method Selection Criteria

```
VERIFICATION METHOD SELECTION

                    Can we measure it?
                          │
              ┌───────────┴───────────┐
              Yes                     No
              │                       │
         Use TEST              Can we model it?
              │                       │
              │               ┌───────┴───────┐
              │              Yes             No
              │               │               │
              │          Use ANALYSIS   Can we see it?
              │               │               │
              │               │       ┌───────┴───────┐
              │               │      Yes             No
              │               │       │               │
              │               │  Use INSPECTION  Use DEMO
              │               │       │               │
              └───────────────┴───────┴───────────────┘
                              │
                         VERIFIED
```

### 3.3 Verification Levels

| Level | Environment | Purpose |
|-------|-------------|---------|
| Development | Lab | Debug, initial verification |
| Qualification | Environmental | Formal verification |
| Acceptance | Ambient | Flight readiness |
| On-Orbit | Space | Mission validation |

## 4. Verification Process

### 4.1 Verification Flow

```
VERIFICATION PROCESS FLOW

1. Requirements Analysis
   └─ Identify verification method
   └─ Define success criteria
   └─ Allocate to test level

2. Test Planning
   └─ Write test procedures
   └─ Identify resources
   └─ Schedule tests

3. Test Execution
   └─ Configure test article
   └─ Execute procedures
   └─ Collect data

4. Results Analysis
   └─ Compare to criteria
   └─ Document results
   └─ Identify discrepancies

5. Verification Closure
   └─ Review results
   └─ Update VCRM
   └─ Obtain approval
```

### 4.2 Verification Activities by Phase

| Phase | Activities |
|-------|------------|
| Design | Allocate methods, plan tests |
| Development | Unit tests, component tests |
| Integration | Interface tests, subsystem tests |
| Qualification | Environmental tests |
| Acceptance | Functional tests |
| Operations | On-orbit validation |

## 5. Validation Approach

### 5.1 Validation Definition

Validation confirms the system meets stakeholder needs and performs as intended in the operational environment.

### 5.2 Validation Methods

| Method | Description |
|--------|-------------|
| Mission Simulation | Simulate full mission scenarios |
| Day-in-the-Life | 24-hour operational exercise |
| End-to-End Test | Complete system demonstration |
| On-Orbit Checkout | Flight performance confirmation |

### 5.3 Validation Criteria

| Criterion | Measure |
|-----------|---------|
| Mission objectives met | L1/L2/L3 success criteria |
| Operations executable | Procedures validated |
| Performance acceptable | Within spec margins |
| Reliability demonstrated | No critical anomalies |

## 6. Verification Requirements Matrix

Withdrawn in version 1.3. The previous tables listed 12 rows, some under undefined IDs (SYS-030, SWR-101 to SWR-501),
one with a 5 s watchdog that contradicts the 30 s of SRS-F050, and three marked verified with no evidence. The VCRM
(`docs/verification/VERIFICATION_MATRIX.md` §3) lists every requirement with its method, planned event and
evidence-based status.

## 7. Test Program

### 7.1 Test Hierarchy

```
TEST HIERARCHY

Level 5: Mission Validation
         └─ On-orbit checkout
         └─ Commissioning

Level 4: System Test
         └─ End-to-end test
         └─ Environmental test
         └─ DITL

Level 3: Integration Test
         └─ Flatsat testing
         └─ Interface testing

Level 2: Subsystem Test
         └─ EPS test
         └─ OBC test
         └─ RF test

Level 1: Component Test
         └─ Unit tests
         └─ Board tests
```

### 7.2 Test Campaign Summary

| Campaign | Article | Dates (`mission/SCHEDULE.md`) | Plan |
|----------|---------|-------------------------------|------|
| Unit and integration tests in CI | Host (simulated HAL) | Continuous | `software/rust/README.md`; CI coverage gates |
| Mission simulation (SIL) and ADCS Monte Carlo | Host | 2026-11 to 2027-01 | `docs/test/SIL_MISSION_SIMULATION.md` |
| Flatsat and HIL on the STM32L4R5 | Flatsat, EM | 2027-04 to 2027-06 | TP-FLATSAT-001; `software/flight/tests/hil/HIL_TEST_PLAN.md` |
| EM development tests | EM | 2027-06 to 2027-08 | ENVIRONMENTAL_TEST_PLAN Rev C §6 |
| FM integration, functional, DITL, end-to-end | FM | 2027-11 to 2028-01 | TP-INT, TP-SYS-001, TP-SYS-002, TP-GS-001 |
| Protoflight environmental | FM | 2028-01 to 2028-02 | ENVIRONMENTAL_TEST_PLAN Rev C §4 |

### 7.3 Test Procedures

| Procedure | Purpose | Level |
|-----------|---------|-------|
| TP-001 | Boot and initialization | Flatsat |
| TP-002 | Mode transitions | Flatsat |
| TP-003 | Telemetry generation | Flatsat |
| TP-004 | Beacon generation | Flatsat, System |
| TP-005 | Fault response | Flatsat |
| TP-006 | Power modes | Flatsat, System |
| TP-007 | Extended duration | Flatsat |
| TP-ENV-002 | Vibration | Environmental |
| TP-ENV-003 | Thermal vacuum | Environmental |
| TP-ENV-004 | EMC/EMI | Environmental |
| TP-E2E-001 | Nominal pass | System |
| TP-DITL-001 | Day-in-the-life | System |

## 8. Verification Environments

### 8.1 Test Environments

| Environment | Configuration | Use |
|-------------|---------------|-----|
| Development | PC + simulators | Unit test |
| Flatsat | EDU hardware | Integration |
| HITL | Flight-like | System test |
| Environmental | Chambers | Qualification |
| Ground Station | Ops config | E2E test |

### 8.2 Environment Fidelity

| Environment | HW Fidelity | SW Fidelity |
|-------------|-------------|-------------|
| Simulation | None | Flight |
| Flatsat | EDU | Flight |
| EM | Flight-like | Flight |
| Flight | Flight | Flight |

## 9. Verification Tools

### 9.1 Test Tools

| Tool | Purpose |
|------|---------|
| Unity | Unit test framework |
| CMock | Mocking framework |
| CTest | Test runner |
| J-Link | Debug/programming |
| Logic Analyzer | Signal verification |
| Spectrum Analyzer | RF verification |
| Power Supply | EPS simulation |

### 9.2 Analysis Tools

| Tool | Purpose |
|------|---------|
| STK/GMAT | Orbit analysis |
| Thermal Desktop | Thermal analysis |
| MATLAB/Python | Data analysis |
| Git | Version control |

## 10. Verification Documentation

### 10.1 Required Documents

| Document | Purpose | When |
|----------|---------|------|
| V&V Plan | This document | PDR |
| Test Procedures | Step-by-step | CDR |
| Test Reports | Results | Post-test |
| VCRM | Status tracking | Continuous |
| Verification Summary | Final status | FRR |

### 10.2 Traceability

| From | To | Document |
|------|-----|----------|
| Mission Objectives | System Requirements | RTM |
| System Requirements | Software Requirements | RTM |
| Requirements | Test Procedures | VCRM |
| Test Procedures | Test Reports | Archive |

## 11. Roles and Responsibilities

### 11.1 V&V Team

| Role | Responsibilities |
|------|------------------|
| V&V Lead | Plan, coordinate, report |
| Test Engineers | Execute tests |
| Software Engineers | Support, fix issues |
| QA | Witness, verify |
| Systems Engineer | Review results |

### 11.2 Independence

| Activity | Independence Level |
|----------|-------------------|
| Unit testing | Developer (low) |
| Integration testing | Test team (medium) |
| System testing | Independent (high) |
| Acceptance | QA witness (high) |

## 12. Verification Metrics

### 12.1 Progress Metrics

| Metric | Target |
|--------|--------|
| Requirements verified | 100% |
| Tests executed | 100% |
| Tests passed | 100% (or waived) |
| Open anomalies | 0 critical |

### 12.2 Status Reporting

| Report | Frequency | Audience |
|--------|-----------|----------|
| Weekly status | Weekly | Team |
| VCRM update | Weekly | PM |
| Test report | Per test | Archive |
| Verification summary | Per review | All |

## 13. Risk Management

### 13.1 V&V Risks

| Risk | Mitigation |
|------|------------|
| Test equipment failure | Backup equipment |
| Schedule slip | Buffer time |
| Test failures | Retest procedures |
| Resource constraints | Prioritization |

### 13.2 Contingency

| Issue | Response |
|-------|----------|
| Test failure | Analyze, fix, retest |
| Equipment failure | Use backup, reschedule |
| Schedule slip | Prioritize critical tests |

## 14. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-16 | SMART-QSO Team | Environmental procedure IDs aligned with `docs/test_procedures/` (vibration TP-ENV-002, thermal vacuum TP-ENV-003, EMC TP-ENV-004; TP-ENV-001 is the pre-environmental baseline); TP-FLAT-001 corrected to TP-FLATSAT-001 as in the RTM (finding VV-18) |
| 1.2 | 2026-10-16 | SMART-QSO Team | §6.1 rows cite the defined requirement with the stated meaning (SYS-215, MRD-053, SYS-211, SYS-210, SYS-021, MRD-025) in its wording; status cells unchanged (finding VV-06) |
| 1.3 | 2026-10-27 | CDR review (PM/SE/V&V) | Status corrected to Draft; §6 matrix withdrawn in favour of the VCRM Rev C (undefined IDs, a conflicting watchdog value and unevidenced "verified" marks); §7.2 campaign summary dated from `mission/SCHEDULE.md`; pointer to ENVIRONMENTAL_TEST_PLAN Rev C |
