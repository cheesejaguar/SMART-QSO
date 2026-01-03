# SMART-QSO Mission Assurance Plan

**Document ID**: SQSO-MAP-001
**Version**: 1.0
**Date**: 2026-01-02
**Classification**: Unclassified
**Status**: CDR Baseline

## 1. Purpose

This document defines the Mission Assurance Plan for SMART-QSO per NPR 8735.2, establishing the quality assurance, reliability, and safety practices to ensure mission success.

## 2. Mission Assurance Approach

### 2.1 Philosophy

```
MISSION ASSURANCE PHILOSOPHY

For Class D CubeSat missions, mission assurance
is tailored to balance risk and resources:

┌─────────────────────────────────────────────────┐
│  MISSION ASSURANCE = QUALITY + RELIABILITY +   │
│                      SAFETY + RISK MANAGEMENT  │
└─────────────────────────────────────────────────┘

Principles:
1. Prevention over detection
2. Right-size processes to mission class
3. Focus on critical items
4. Learn from heritage missions
5. Document for traceability
```

### 2.2 Tailoring Rationale

| Full NPR 8735.2 | SMART-QSO Tailoring | Rationale |
|-----------------|---------------------|-----------|
| Independent MA org | Embedded in team | Small team |
| Formal audits | Peer reviews | Efficiency |
| Full documentation | Essential docs | Resources |
| MRB for all NCRs | Engineering disposition | Speed |

### 2.3 Classification

| Item | Classification |
|------|----------------|
| Mission | Class D |
| Safety | Non-hazardous |
| Software | Class C |
| EEE Parts | Level 3 |

## 3. Organization

### 3.1 MA Responsibilities

| Role | Responsibilities |
|------|------------------|
| Project Manager | Overall MA accountability |
| Systems Engineer | Technical assurance |
| QA Lead | Process compliance |
| Safety Engineer | Hazard management |
| Software Lead | Software quality |
| Integration Lead | Workmanship |

### 3.2 Reporting

```
MA REPORTING STRUCTURE

Project Manager
       │
   ┌───┴───┐
   │       │
Systems   QA Lead
Engineer     │
   │     ┌───┴───┐
   │     │       │
   │  Safety  Software
   │  Engr    Lead
   │     │       │
   └─────┴───────┘
         │
   Integration
      Lead
```

## 4. Quality Assurance

### 4.1 QA Activities

| Phase | QA Activities |
|-------|---------------|
| Design | Requirements review, design review |
| Development | Code review, unit test witness |
| Integration | Procedure review, test witness |
| Test | Test witness, data review |
| Operations | Procedure compliance |

### 4.2 Inspections

| Type | When | By |
|------|------|-----|
| In-process | During assembly | Technician + QA |
| Receiving | Parts arrival | QA |
| Final | Before close-out | QA |
| Witness | Key tests | QA |

### 4.3 Quality Records

| Record | Retention |
|--------|-----------|
| Inspection records | Mission life |
| Test data | Mission life |
| NCRs | Mission life |
| Certifications | 3 years |
| Training | 3 years |

## 5. Reliability Program

### 5.1 Reliability Activities

| Activity | Purpose |
|----------|---------|
| FMEA | Identify failure modes |
| Parts selection | Reliability by design |
| Derating | Reduce stress |
| Redundancy analysis | SPF identification |
| Reliability prediction | MTBF estimation |

### 5.2 Critical Items

| Item | Criticality | Control |
|------|-------------|---------|
| Battery | Safety/Mission | Special handling |
| OBC | Mission | ESD controls |
| RF PA | Mission | Thermal control |
| Solar cells | Mission | Handling procedures |

### 5.3 Failure Reporting

| Failure Type | Reporting |
|--------------|-----------|
| Test failure | NCR + investigation |
| Anomaly | Anomaly report |
| Trend | Trend analysis |
| Close call | Lessons learned |

## 6. Safety Program

### 6.1 Safety Activities

| Activity | When |
|----------|------|
| Hazard identification | Design phase |
| Hazard analysis | CDR |
| Safety verification | Integration |
| Safety review | Pre-ship |

### 6.2 Hazard Tracking

| Hazard | Status | Verification |
|--------|--------|--------------|
| RF emission | Controlled | Timer test |
| Battery | Controlled | Protection test |
| Deployment | Controlled | Mechanism test |

### 6.3 Safety Training

| Topic | Audience |
|-------|----------|
| ESD awareness | All handlers |
| Battery safety | Integration team |
| RF safety | Test team |
| Facility safety | All |

## 7. Parts and Materials

### 7.1 Parts Program

| Activity | Purpose |
|----------|---------|
| Parts selection | Appropriate grade |
| Screening | Weed out infant mortality |
| Derating | Reduce failure rate |
| Traceability | Lot control |

### 7.2 Parts Grades

| Grade | Use | Screening |
|-------|-----|-----------|
| Space-grade | Critical | Per spec |
| Industrial | General | Temperature |
| Commercial | Non-critical | Sample lot |

### 7.3 Prohibited Materials

| Material | Reason |
|----------|--------|
| Pure tin | Whiskers |
| Zinc | Corrosion |
| Cadmium | Toxicity |
| PVC | Outgassing |

### 7.4 Materials Control

| Requirement | Method |
|-------------|--------|
| Outgassing | ASTM E595 compliance |
| Flammability | NASA-STD-6001 |
| Toxicity | Review |
| Corrosion | Material selection |

## 8. Process Control

### 8.1 Controlled Processes

| Process | Control Document |
|---------|------------------|
| Soldering | NASA-STD-8739.3 |
| Crimping | NASA-STD-8739.4 |
| Bonding | NASA-STD-8739.6 |
| Cleaning | SQSO-WKS-001 |
| ESD handling | ANSI/ESD S20.20 |

### 8.2 Operator Certification

| Process | Certification |
|---------|---------------|
| Soldering | J-STD-001 |
| Crimping | NASA-STD-8739.4 |
| Inspection | IPC-A-610 |

### 8.3 Equipment Calibration

| Equipment | Interval |
|-----------|----------|
| Multimeters | 1 year |
| Power supplies | 1 year |
| Torque tools | 1 year |
| Temperature | 1 year |

## 9. Nonconformance Control

### 9.1 NCR Process

```
NONCONFORMANCE PROCESS

1. Identify discrepancy
   └─ Tag item HOLD

2. Document on NCR
   └─ Description, cause

3. Engineering review
   └─ Disposition

4. Disposition options:
   ├─ Use as-is (with rationale)
   ├─ Rework (to drawing)
   ├─ Repair (deviation)
   └─ Scrap

5. Implement disposition
   └─ Verify correction

6. Close NCR
   └─ Approval signatures
```

### 9.2 Disposition Authority

| Disposition | Authority |
|-------------|-----------|
| Use as-is (minor) | Systems Engineer |
| Use as-is (major) | PM + customer |
| Rework | Systems Engineer |
| Repair | PM |
| Scrap | QA |

### 9.3 NCR Tracking

| Metric | Target |
|--------|--------|
| Open NCRs | <5 |
| Closure time | <30 days |
| Recurrence | 0 |

## 10. Configuration Management

### 10.1 CM Activities

| Activity | Purpose |
|----------|---------|
| Identification | Baseline definition |
| Control | Change management |
| Status accounting | Tracking |
| Audits | Verification |

### 10.2 Baselines

| Baseline | When |
|----------|------|
| Functional | SRR |
| Allocated | PDR |
| Product | CDR |
| As-built | FRR |

### 10.3 Change Control

| Change Type | Authority |
|-------------|-----------|
| Document | Document owner |
| Design (minor) | Systems Engineer |
| Design (major) | CCB |
| Requirements | CCB + customer |

## 11. Reviews and Audits

### 11.1 Technical Reviews

| Review | Purpose |
|--------|---------|
| SRR | Requirements complete |
| PDR | Design feasible |
| CDR | Design complete |
| TRR | Test ready |
| FRR | Flight ready |

### 11.2 Audits

| Audit | When | Focus |
|-------|------|-------|
| FCA | Pre-FRR | Function matches spec |
| PCA | Pre-FRR | Build matches design |
| Safety | Pre-ship | Hazards closed |

### 11.3 Peer Reviews

| Type | Frequency |
|------|-----------|
| Code review | Per module |
| Design review | Per subsystem |
| Test review | Per procedure |
| Document review | Per release |

## 12. Supplier Control

### 12.1 Supplier Categories

| Category | Control Level |
|----------|---------------|
| Critical | Audit + source inspection |
| Important | Receiving inspection |
| Standard | Sampling |

### 12.2 Procurement QA

| Activity | Purpose |
|----------|---------|
| Vendor qualification | Capability assessment |
| PO requirements | Flow-down specs |
| Receiving inspection | Verify compliance |
| Feedback | Corrective action |

## 13. Risk Management

### 13.1 Risk Process

| Step | Activity |
|------|----------|
| Identify | Find risks |
| Analyze | Assess L×C |
| Plan | Develop mitigation |
| Track | Monitor status |
| Control | Execute mitigation |

### 13.2 Risk Categories

| Category | Examples |
|----------|----------|
| Technical | Design, performance |
| Schedule | Delays, dependencies |
| Cost | Overruns |
| Programmatic | Resources, external |

### 13.3 Risk Acceptance

| Risk Level | Authority |
|------------|-----------|
| Low | PM |
| Medium | PM + SE |
| High | PM + customer |
| Unacceptable | Must mitigate |

## 14. Training

### 14.1 Required Training

| Training | Audience | Frequency |
|----------|----------|-----------|
| MA overview | All | Initial |
| ESD | Handlers | Annual |
| Workmanship | Technicians | Initial |
| Safety | All | Annual |
| Procedures | Operators | Per procedure |

### 14.2 Training Records

| Record | Content |
|--------|---------|
| Certification | Skill demonstrated |
| Attendance | Training completed |
| Recertification | Currency maintained |

## 15. Metrics and Reporting

### 15.1 MA Metrics

| Metric | Target | Frequency |
|--------|--------|-----------|
| Open NCRs | <5 | Weekly |
| Test failures | Trend down | Monthly |
| Defect escape | 0 | Per test |
| Process compliance | 100% | Per audit |

### 15.2 MA Reports

| Report | Frequency | Audience |
|--------|-----------|----------|
| Weekly status | Weekly | PM |
| NCR summary | Monthly | Team |
| Review report | Per review | All |
| Final MA report | FRR | Archive |

## 16. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
