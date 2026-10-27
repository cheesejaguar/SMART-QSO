# SMART-QSO Mission Assurance Plan

**Document ID**: SQSO-MAP-001
**Version**: 2.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR draft. §1, §2.2–§2.3, §5.2, §6.2, §7 and §8 rewritten by the CDR S&MA review; the other sections are
the Version 1.0 framework, still to be staffed (risk R14)

## 1. Purpose

This document defines the Mission Assurance Plan for SMART-QSO: the quality assurance, reliability, safety, parts,
materials, workmanship and contamination practices for a **Class D** payload (NPR 8705.4B, Risk Classification for NASA
Payloads, effective 2024-11-01). SMART-QSO is not a NASA mission; NASA documents are used as the recognised standard of
practice. The structure follows SMD SPD-39 (*Standard Mission Assurance Requirements for Payload Classification D*,
2021) as a template, tailored below. Version 1.0 cited NPR 8735.2 as its basis; NPR 8735.2 covers hardware quality
assurance surveillance of NASA contracts, and is used here only for the inspection practices of §4.

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

| Practice (SPD-39 / NASA norm) | SMART-QSO tailoring | Rationale |
|-------------------------------|---------------------|-----------|
| Independent S&MA organisation | S&MA lead role independent of the design leads; independent review at PDR, CDR and before shipment | Small team (R14) |
| Formal audits | Peer reviews plus mandatory inspection points (§4.2) | Resources |
| Material Review Board for every NCR | Engineering disposition for minor NCRs; MRB (PI, systems engineer, S&MA lead) for major ones and any use-as-is on flight hardware | Speed with control |
| Space-grade EEE parts | Commercial parts with screening, traceability and finish controls (§7) | Class D; cost |
| Full M&P control (NASA-STD-6016C) | Outgassing, prohibited materials, stress-corrosion and fastener controls only (§7.4) | Containerized 1U, no crew contact on the baseline path |

### 2.3 Classification

| Item | Classification |
|------|----------------|
| Mission | Class D |
| Safety | The software has safety-critical components (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.2; Safety Data Package §3.2) |
| Software | Class C per NPR 7150.2D (flight software and ground software used in operations); Class D for research and analysis tools (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1) |
| EEE Parts | NASA-STD-8739.10 Level 4 (commercial) with project screening for mission-critical parts (§7; `docs/manufacturing/EEE_PARTS_LIST.md` Rev G §2A) |

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

The critical items list is `docs/safety/SYSTEM_FMEA_CIL.md` (SQSO-FMEA-002) §3: inhibits, battery and its protections,
burn circuits and restraints, structure and fasteners (Category 1), and the bus single points (Category 2). Each has its
controls and verification there. Reliability estimates and the early-failure program: `docs/environment/RELIABILITY_ANALYSIS.md` v2.2.

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

The system hazard log is `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md` (SQSO-SDP-002) §5; software hazards are in
SQSO-SDP-001. **Every hazard is open** until its verification evidence is filed. Version 1.0 listed RF emission,
battery and deployment as "Controlled"; that is withdrawn (finding VV-05).

### 6.3 Safety Training

| Topic | Audience |
|-------|----------|
| ESD awareness | All handlers |
| Battery safety | Integration team |
| RF safety | Test team |
| Facility safety | All |

## 7. Parts and Materials

### 7.1 EEE parts

Policy, screening, traceability, radiation acceptance and finishes: `docs/manufacturing/EEE_PARTS_LIST.md` Rev G §2A.
In brief:

| Element | Requirement | Reference |
|---------|-------------|-----------|
| Grade | NASA-STD-8739.10 Level 4 (commercial); automotive or industrial grade preferred | NASA-STD-8739.10 (2017); SPD-39 |
| Screening of mission-critical parts | EEE-INST-002 Level 3 principles applied at board level: receiving inspection, thermal cycling powered, ≥ 200 h burn-in with 100 h failure-free | GSFC EEE-INST-002 (2003, Addendum 1 2008) |
| Lot traceability | Franchised sources only; one lot per part type; lot recorded on the traveler | SAE AS6081 practice for any other source |
| Radiation | TID ≥ 10 krad by data, heritage or lot test; latch-up current limiters on every mission-critical rail | NASA/TM-20210018053; RADIATION_ANALYSIS Rev F |
| Finishes | No pure tin (re-tin with Sn63Pb37 or conformal coat), no cadmium, no zinc plating | UG v11 Table 5-5; GEIA-STD-0005-2, GEIA-STD-0006 |
| Derating | EEE_PARTS_LIST §12 | — |

### 7.2 Parts grades

| Grade | Use |
|-------|-----|
| Space-grade or heritage assemblies | Where available at comparable cost (AntS, bought panels, magnetometer) |
| Automotive / industrial commercial with screening | All mission-critical custom boards (OBC, EPS, radios) |
| Commercial, isolated by a load switch | Jetson, GNSS, backup gyro |

### 7.3 Prohibited materials

Per the SpaceX Rideshare Payload User's Guide v11 Table 5-5, which applies to a containerized CubeSat (§5.3): cadmium and
cadmium plating, zinc plating, mercury and its compounds, pure tin and tin electroplate. Also PVC (outgassing, chlorine)
and Sharpie markers. Where each could enter the design and its control: `docs/compliance/MATERIALS_LIST.md` Rev E §2.2.

### 7.4 Materials and processes (NASA-STD-6016C with Change 1, tailored)

| Requirement | Method | Applies |
|-------------|--------|---------|
| Outgassing TML ≤ 1.0 %, CVCM ≤ 0.1 % (CDS 2.1.7; UG v11 RML ≤ 1.0 %) | ASTM E595 data with a cited source for every non-metallic material (MATERIALS_LIST Rev E); bakeout in TVAC | All |
| Stress-corrosion cracking | Alloys from MSFC-STD-3029 Table I (high resistance): 6061-T6, 7075-T73 preferred to 7075-T6 for stressed parts, A286, 300-series, Ti-6Al-4V | Structure, fasteners |
| Threaded fasteners | Locking feature on every fastener (UG v11 §5.2), torque table, NASA-STD-5020B practice for preload; staking with an E595-compliant adhesive; no thread-locker alone on polymer-bushed joints | Structure, battery mount |
| Galvanic compatibility | Conversion-coated aluminium against stainless or titanium; no bare dissimilar contact in moisture-exposed stowage | Structure |
| Flammability and offgassing toxicity | NASA-STD-6001B, **only for the ISS backup path** | ISS path |
| Lubricants | Dry-film on Ti threads; no silicone oils near optics or sun sensors | Mechanisms |

### 7.5 Materials control

Every material is entered in MATERIALS_LIST before use on flight hardware; a material without data is used only with an
NCR disposition or a UG v11 rationale code agreed with the integrator.

## 8. Process Control

### 8.1 Controlled Processes

| Process | Standard | Notes |
|---------|----------|-------|
| Implementation of the NASA workmanship standards, ESD control | **NASA-STD-8739.6B** (effective 2021-02-04) | Adopts IPC J-STD-001 with its space addendum, IPC/WHMA-A-620 with its space addendum, and ANSI/ESD S20.20 with NASA additions |
| Soldering | IPC J-STD-001 (current revision) with the Space and Military Applications Electronic Hardware Addendum, as invoked by NASA-STD-8739.6B; acceptance to IPC-A-610 Class 3 | NASA-STD-8739.3 (soldering) was cancelled and is not used |
| Crimping, interconnecting cables, harnesses and wiring | **NASA-STD-8739.4A** (2016), with IPC/WHMA-A-620 space addendum per 8739.6B | Lacing instead of nylon ties |
| Polymeric application: conformal coating, staking, bonding of parts on assemblies | **NASA-STD-8739.1B** (*Workmanship Standard for Polymeric Application on Electronic Assemblies*) | Version 1.0 listed "bonding" under 8739.6; that standard is the implementation standard, not bonding |
| Fiber optics | NASA-STD-8739.5 | Not applicable (no fiber) |
| Cleaning and contamination | `docs/integration/CONTAMINATION_CONTROL_PLAN.md` v2.0 | §8.4 |
| ESD | ANSI/ESD S20.20 (current revision) as supplemented by NASA-STD-8739.6B | §8.3 |

### 8.2 Operator Certification

| Process | Certification |
|---------|---------------|
| Soldering, inspection | IPC J-STD-001 space addendum certified operator or inspector (CIS), per NASA-STD-8739.6B training requirements |
| Crimping and harness | NASA-STD-8739.4A / IPC/WHMA-A-620 space addendum |
| Conformal coat and staking | NASA-STD-8739.1B |
| ESD | ANSI/ESD S20.20 awareness, annual |

Until the team has certified operators (R14, R23), flight boards are assembled by a contract manufacturer certified to
J-STD-001 with the space addendum.

### 8.3 ESD Control

ESD-protected area per ANSI/ESD S20.20 and NASA-STD-8739.6B for all work on flight and EM boards: grounded work
surfaces and wrist straps verified at each use, ionisers for insulators that cannot be removed, ESD-shielding bags for
storage and transport, and the device sensitivity assumed to be 100 V HBM unless the data sheet says otherwise.

### 8.4 Contamination Control

Per `docs/integration/CONTAMINATION_CONTROL_PLAN.md` v2.0: assembly in an ISO 14644-1 Class 8 area (the PPF class,
UG v11 Table 4-12), visual cleanliness to the integrator's level before delivery, bakeout in thermal vacuum, bagging
in ESD-safe low-outgassing film.

### 8.5 Equipment Calibration

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
| Requirements | CCB (the PI; no customer: CM plan §2.2) |

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
| 1.0, note | 2026-10-25 | SMART-QSO Team | §2.3 software row cites the NPR 7150.2D determination (SDP §3.1); the class is unchanged. Safety row: the software has safety-critical components (SDP §3.2), not "non-hazardous" |
| 2.0 | 2026-10-27 | CDR S&MA review | Class D basis (NPR 8705.4B, SPD-39 template) replaces the NPR 8735.2 citation; tailoring table; critical items and hazard log point to SQSO-FMEA-002 and SQSO-SDP-002 (hazards reopened); §7 EEE policy (NASA-STD-8739.10 Level 4, EEE-INST-002 screening, traceability, radiation, tin and prohibited finishes), M&P per NASA-STD-6016C tailored; §8 workmanship standards corrected (8739.6B, J-STD-001 space addendum, 8739.4A, 8739.1B; 8739.3 cancelled), ESD and contamination |
| 2.1 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): cited revisions (EEE_PARTS_LIST G, MATERIALS_LIST E, RADIATION_ANALYSIS F, RELIABILITY_ANALYSIS 2.2); requirements change authority is the PI as CCB |
