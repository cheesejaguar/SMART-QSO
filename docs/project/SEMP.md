# Systems Engineering Management Plan (SEMP)

**Document Number**: SMART-QSO-SEMP-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Draft

**Governing Standard**: NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements

---

## 1. Introduction

### 1.1 Purpose

This Systems Engineering Management Plan (SEMP) defines the technical approach, processes, and procedures for systems engineering on the SMART-QSO CubeSat mission.

### 1.2 Scope

This plan covers all systems engineering activities from Phase B (Preliminary Design) through Phase E (Operations).

### 1.3 Process Heritage

This SEMP follows the NASA Ames Research Center Small Satellite Division approach, successfully used on:
- GeneSat-1 (2006)
- PharmaSat (2009)
- O/OREOS (2010)
- EcAMSat (2017)

---

## 2. Technical Approach

### 2.1 Mission Overview

SMART-QSO is a 1U CubeSat demonstrating onboard AI for amateur radio transponder management. Key technical objectives:

1. Demonstrate TinyML/TinyLM inference on COTS hardware in orbit
2. Validate adaptive beacon generation using AI
3. Characterize power-constrained AI operations
4. Support amateur radio community with open data

### 2.2 System Architecture

```
+-------------------------------------------------------+
|                    SMART-QSO 1U CubeSat               |
|                                                       |
|  +-------------+    +-------------+    +----------+   |
|  |   Structure |    |     EPS     |    |   ADCS   |   |
|  | (1U frame)  |    | (solar+batt)|    | (B-dot)  |   |
|  +-------------+    +------+------+    +----+-----+   |
|                            |                |         |
|                     +------+------+         |         |
|                     |     OBC     +---------+         |
|                     | (Apollo4/   |                   |
|                     |  STM32L4)   |                   |
|                     +------+------+                   |
|                            |                          |
|               UART  +------+------+                   |
|              +------|   Jetson    |                   |
|              |      | Orin Nano   |                   |
|              |      +-------------+                   |
|              |                                        |
|       +------+------+                                 |
|       | RF Subsystem|    --> 2m/70cm Antennas         |
|       | (TX + XCVR) |                                 |
|       +-------------+                                 |
+-------------------------------------------------------+
```

### 2.3 Key Technical Decisions

| Decision | Selected Option | Rationale |
|----------|-----------------|-----------|
| OBC | Ultra-low-power MCU | Heritage, power efficiency |
| AI Payload | Jetson Orin Nano | TensorRT support, performance |
| ADCS | Passive magnetic + B-dot | Simplicity, reliability |
| RF | UHF downlink, VHF optional | Amateur band allocation |
| Structure | PC-104 stacking | Standard CubeSat form factor |

---

## 3. Systems Engineering Processes

### 3.1 NPR 7123.1 Process Tailoring

Per NPR 7123.1, the 17 SE processes are tailored for Class D CubeSat:

| Process | Application | Tailoring |
|---------|-------------|-----------|
| Stakeholder Expectations | Full | Amateur radio community input |
| Technical Requirements | Full | SRS/ICD documentation |
| Logical Decomposition | Tailored | Simplified functional analysis |
| Design Solution | Full | Standard design process |
| Product Realization | Full | COTS + limited fabrication |
| Product Integration | Full | Flatsat then flight unit |
| Product Verification | Full | Protoflight approach |
| Product Validation | Full | End-to-end demonstration |
| Product Transition | Tailored | Launch provider handoff |

### 3.2 Technical Reviews

| Review | Entrance Criteria | Exit Criteria |
|--------|-------------------|---------------|
| SRR | ConOps approved | Requirements baselined |
| PDR | Requirements complete | Design approach approved |
| CDR | Detailed design complete | Fabrication authorized |
| TRR | Integration complete | Test readiness confirmed |
| FRR | Verification complete | Launch readiness confirmed |

### 3.3 Technical Performance Measures (TPMs)

| TPM | Threshold | Goal | Current |
|-----|-----------|------|---------|
| Total mass | < 1.5 kg | < 1.2 kg | TBD |
| Power generation (EOL) | > 2.0 W | > 2.5 W | TBD |
| Beacon interval | < 60 s | < 30 s | 30 s |
| AI inference latency | < 10 s | < 5 s | TBD |
| Link margin (downlink) | > 6 dB | > 10 dB | TBD |

---

## 4. Integration Strategy

### 4.1 Integration Approach

Following NASA Ames heritage, SMART-QSO uses a **flatsat-first** integration strategy:

```
Phase 1: Component Development
  └── Individual board bringup
  └── Unit software testing

Phase 2: Flatsat Integration
  └── Engineering model boards
  └── Full system software
  └── Extended operation testing

Phase 3: Flight Unit Integration
  └── Flight boards step-by-step
  └── Functional testing at each step
  └── Final assembly and closeout

Phase 4: Environmental Testing
  └── Vibration
  └── Thermal vacuum
  └── EMC/EMI
```

### 4.2 Integration Sequence

1. Structure + rails
2. EPS board integration
3. OBC integration + power verification
4. ADCS board integration
5. RF subsystem integration
6. Jetson payload integration
7. Solar panel integration
8. Antenna stowage + deployment mechanism
9. Final assembly and mass properties

---

## 5. Verification and Validation

### 5.1 Verification Approach

| Method | Application |
|--------|-------------|
| Analysis | Power budgets, link budgets, thermal |
| Inspection | Workmanship, configuration |
| Demonstration | Operational functions |
| Test | Performance, environmental |

### 5.2 Protoflight Testing

Per GSFC-STD-7000 (GEVS), SMART-QSO follows protoflight approach:

| Test | Level | Duration |
|------|-------|----------|
| Random vibration | Protoflight (3 dB above qual) | 1 min/axis |
| Thermal vacuum | Protoflight | 4 cycles |
| Functional | Comprehensive | Per procedure |

### 5.3 Validation Approach

Validation demonstrates the system meets mission needs:

1. **End-to-end beacon demonstration** - Full chain from sensor to RF
2. **AI inference validation** - Beacon text meets quality criteria
3. **Power mode transitions** - Autonomous operation demonstrated
4. **Day-in-the-life test** - 24+ hour continuous operation

---

## 6. Risk Management

### 6.1 Technical Risk Categories

| Category | Examples |
|----------|----------|
| Design | Jetson power consumption, thermal management |
| Integration | Interface compatibility, timing |
| Test | Facility availability, schedule |
| Operations | Ground station coverage, communication |

### 6.2 Risk Mitigation

See `mission/RISK_REGISTER.md` for detailed risk tracking.

---

## 7. Configuration Management

### 7.1 Configuration Items

| CI Type | Examples |
|---------|----------|
| Hardware | Schematics, PCB layouts, CAD |
| Software | Source code, configurations |
| Documents | Requirements, design, procedures |
| Firmware | Binary images, version info |

### 7.2 Baselines

| Baseline | Established At | Contents |
|----------|----------------|----------|
| Functional | PDR | Requirements |
| Allocated | CDR | Design, ICDs |
| Product | FRR | As-built configuration |

---

## 8. Technical Planning

### 8.1 Work Breakdown Structure

```
1.0 SMART-QSO Mission
├── 1.1 Project Management
├── 1.2 Systems Engineering
├── 1.3 Spacecraft Bus
│   ├── 1.3.1 Structure
│   ├── 1.3.2 EPS
│   ├── 1.3.3 ADCS
│   ├── 1.3.4 C&DH (OBC)
│   └── 1.3.5 Communications
├── 1.4 AI Payload
├── 1.5 Ground Segment
├── 1.6 Integration & Test
├── 1.7 Launch Services
└── 1.8 Mission Operations
```

### 8.2 Technical Schedule

| Milestone | Target Date |
|-----------|-------------|
| PDR | TBD |
| CDR | TBD + 3 months |
| Integration Complete | TBD + 6 months |
| Environmental Test | TBD + 8 months |
| Ship to Launch | TBD + 10 months |

---

## 9. Interface Management

### 9.1 Interface Control

All interfaces are documented in ICDs and controlled through:
1. Interface identification in requirements
2. Interface documentation in ICDs
3. Interface verification in test procedures
4. Interface change control via CCB

### 9.2 Key Interfaces

| Interface | ICD |
|-----------|-----|
| OBC-EPS | ICD-001 |
| OBC-Jetson | ICD-002 |
| OBC-ADCS | ICD-003 |
| OBC-RF | ICD-004 |
| Spacecraft-Deployer | ICD-005 |

---

## 10. SE Metrics

### 10.1 Tracked Metrics

| Metric | Target | Frequency |
|--------|--------|-----------|
| Requirements volatility | < 10% after PDR | Monthly |
| TBD/TBR closure rate | 100% by CDR | Weekly |
| RID closure rate | > 80% at review | Per review |
| TPM status | All green at FRR | Monthly |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
