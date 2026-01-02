# Project Plan
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-PM-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7120.5 Section 3.3

---

## 1. Purpose

This document defines the project plan for the SMART-QSO CubeSat mission per NPR 7120.5 requirements, establishing objectives, organization, schedule, and management approach.

## 2. Mission Overview

### 2.1 Mission Statement

SMART-QSO demonstrates onboard agentic AI to manage an amateur radio transponder from a 1U CubeSat platform, advancing TinyML/TinyLM technology for space applications.

### 2.2 Objectives

| ID | Objective | Priority |
|----|-----------|----------|
| O1 | Demonstrate AI-managed amateur radio operations | Primary |
| O2 | Provide amateur radio operators with transponder access | Primary |
| O3 | Demonstrate graceful degradation to conventional operation | Primary |
| O4 | Advance TinyML/TinyLM for space applications | Secondary |

### 2.3 Success Criteria

| Level | Criteria |
|-------|----------|
| Minimum | Beacon received on ground |
| Partial | Conventional transponder operation |
| Full | AI-managed transponder demonstrated |

---

## 3. Project Organization

### 3.1 Organizational Structure

```
                    Project Manager
                          │
         ┌────────────────┼────────────────┐
         │                │                │
    Systems Engr     SW Lead          HW Lead
         │                │                │
    ├─ Mission       ├─ OBC FSW      ├─ EPS
    ├─ Requirements  ├─ Payload SW   ├─ C&DH
    ├─ V&V           ├─ Ground SW    ├─ RF
    └─ Safety        └─ Test         ├─ ADCS
                                     └─ Structure
```

### 3.2 Roles and Responsibilities

| Role | Responsibilities |
|------|------------------|
| Project Manager | Overall project execution, schedule, budget |
| Systems Engineer | Requirements, integration, V&V |
| Software Lead | Flight and ground software development |
| Hardware Lead | Subsystem design and integration |
| Quality Assurance | Process compliance, inspections |

---

## 4. Work Breakdown Structure

### 4.1 WBS Summary

| WBS | Element | Description |
|-----|---------|-------------|
| 1.0 | Project Management | Planning, control, reviews |
| 2.0 | Systems Engineering | Requirements, design, V&V |
| 3.0 | Flight Hardware | EPS, C&DH, RF, ADCS, Structure |
| 4.0 | Flight Software | OBC FSW, Payload SW |
| 5.0 | Ground Segment | Ground station, operations |
| 6.0 | Integration & Test | Assembly, environmental test |
| 7.0 | Mission Operations | Launch, on-orbit ops |

### 4.2 WBS Detail

**1.0 Project Management**
- 1.1 Project Planning
- 1.2 Schedule Management
- 1.3 Risk Management
- 1.4 Configuration Management
- 1.5 Reviews (SRR, PDR, CDR, TRR, FRR)

**2.0 Systems Engineering**
- 2.1 Requirements Development
- 2.2 System Design
- 2.3 Interface Management
- 2.4 Verification & Validation
- 2.5 Safety & Mission Assurance

**3.0 Flight Hardware**
- 3.1 EPS Design & Build
- 3.2 C&DH/OBC Design & Build
- 3.3 RF Subsystem Design & Build
- 3.4 ADCS Design & Build
- 3.5 Structure & Thermal
- 3.6 Harness & Integration

**4.0 Flight Software**
- 4.1 OBC Flight Software
- 4.2 Payload (Jetson) Software
- 4.3 Software Integration
- 4.4 Software V&V

**5.0 Ground Segment**
- 5.1 Ground Station Setup
- 5.2 Mission Control Software
- 5.3 Operations Procedures

**6.0 Integration & Test**
- 6.1 Flatsat Development
- 6.2 Flight Unit Integration
- 6.3 Environmental Testing
- 6.4 Pre-Ship Review

**7.0 Mission Operations**
- 7.1 Launch Campaign
- 7.2 Commissioning
- 7.3 Nominal Operations
- 7.4 End of Life

---

## 5. Schedule

### 5.1 Master Schedule

| Phase | Milestone | Target |
|-------|-----------|--------|
| Phase A | Mission Concept Review | Complete |
| Phase B | System Requirements Review | Complete |
| Phase B | Preliminary Design Review | Complete |
| Phase C | Critical Design Review | Current |
| Phase C | Hardware Fabrication | CDR + 3 months |
| Phase C/D | Integration & Test | CDR + 6 months |
| Phase D | Flight Readiness Review | CDR + 9 months |
| Phase D | Launch | TBD |
| Phase E | Mission Operations | 12 months |
| Phase F | Decommissioning | L + 12-18 months |

### 5.2 Key Decision Points

| KDP | Gate | Criteria |
|-----|------|----------|
| KDP-A | Mission Approval | Concept feasibility |
| KDP-B | Formulation Approval | Requirements baseline |
| KDP-C | Implementation Approval | Design complete |
| KDP-D | Operations Approval | Ready for launch |

### 5.3 Review Schedule

| Review | Purpose | Entry Criteria |
|--------|---------|----------------|
| SRR | Requirements baseline | ConOps approved |
| PDR | Preliminary design approval | Requirements baselined |
| CDR | Final design approval | Design complete |
| TRR | Test readiness | Integration complete |
| FRR | Flight readiness | All tests passed |

---

## 6. Resource Plan

### 6.1 Team Size

| Phase | FTE |
|-------|-----|
| Phase A/B | 2-3 |
| Phase C | 4-6 |
| Phase D | 3-4 |
| Phase E | 1-2 |

### 6.2 Key Resources

| Resource | Source |
|----------|--------|
| Development Lab | University/Organization |
| Clean Room | Shared facility |
| Environmental Test | External lab |
| Ground Station | Amateur radio club |

---

## 7. Risk Management

### 7.1 Top Project Risks

| Risk | L | C | Mitigation |
|------|---|---|------------|
| Schedule slip | M | M | Margin in schedule |
| Technical challenges | M | M | Prototyping, heritage |
| Resource constraints | L | M | Phased staffing |
| Launch delay | M | L | Flexible manifest |

### 7.2 Risk Process

- Monthly risk reviews
- Risk register maintained in mission/RISK_REGISTER.md
- Escalation to PM for high risks

---

## 8. Configuration Management

### 8.1 Baselines

| Baseline | Established At |
|----------|----------------|
| Functional | SRR |
| Allocated | PDR |
| Product | CDR |

### 8.2 Change Control

- Change requests via GitHub issues
- CCB review for Class I changes
- Documented in CM Plan

---

## 9. Communication Plan

### 9.1 Meetings

| Meeting | Frequency | Participants |
|---------|-----------|--------------|
| Team standup | Weekly | All |
| Technical review | Bi-weekly | Leads |
| Management review | Monthly | PM, Leads |
| Stakeholder update | Quarterly | All stakeholders |

### 9.2 Documentation

- All documents in Git repository
- Reviews via pull requests
- Status in project wiki

---

## 10. Quality Assurance

### 10.1 QA Activities

| Activity | Frequency |
|----------|-----------|
| Code reviews | Per commit |
| Design reviews | Per phase |
| Inspections | Per assembly |
| Audits | Pre-review |

### 10.2 Metrics

| Metric | Target |
|--------|--------|
| Requirements coverage | 100% |
| Test coverage | >80% |
| Defect closure | <30 days |
| Review action closure | <14 days |

---

## 11. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
