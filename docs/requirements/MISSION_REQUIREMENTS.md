# Mission Requirements Document
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REQ-001
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7123.1, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the Level 1 (Mission) requirements for the SMART-QSO CubeSat mission. These requirements are derived from the mission objectives and Concept of Operations (ConOps) and flow down to system-level requirements.

## 2. Scope

This document covers:
- Mission objectives and success criteria
- Top-level mission requirements
- Verification approach for each requirement
- Traceability to stakeholder needs

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| NPR 7123.1 | NASA Systems Engineering Processes and Requirements |
| NPR 7120.5 | NASA Space Flight Program and Project Management |
| SQSO-CONOPS | SMART-QSO Concept of Operations |
| Cal Poly CDS Rev 14.1 | CubeSat Design Specification |

## 4. Mission Overview

### 4.1 Mission Statement

SMART-QSO is a 1U CubeSat mission demonstrating onboard agentic AI to manage an amateur radio transponder, using TinyML/TinyLM models to learn power availability, predict regional demand, prioritize QSOs fairly, and compose dynamic beacons.

### 4.2 Mission Objectives

| ID | Objective | Priority |
|----|-----------|----------|
| MO-1 | Demonstrate AI-managed amateur radio operations from LEO | Primary |
| MO-2 | Provide amateur radio operators with transponder access | Primary |
| MO-3 | Demonstrate graceful degradation to conventional operation | Primary |
| MO-4 | Advance TinyML/TinyLM technology for space applications | Secondary |
| MO-5 | Engage amateur radio community in space communications | Secondary |

### 4.3 Success Criteria

| Level | Criteria |
|-------|----------|
| Minimum | Beacon transmission received on ground |
| Partial | Conventional transponder operation achieved |
| Full | AI-managed transponder operation demonstrated |
| Extended | Federated learning window operation demonstrated |

---

## 5. Mission Requirements

### 5.1 Mission Duration Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-001 | The mission shall have a design life of 12 months minimum | A | Critical |
| MRD-002 | The spacecraft shall be capable of autonomous operations for up to 7 days without ground contact | D | Critical |
| MRD-003 | The spacecraft shall deorbit within 25 years per NASA-STD-8719.14 | A | Critical |

### 5.2 Orbit Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-010 | The mission shall operate in Low Earth Orbit (LEO) | A | Critical |
| MRD-011 | The nominal orbit altitude shall be 400-500 km | A | Critical |
| MRD-012 | The orbit inclination shall support global amateur radio coverage | A | High |

### 5.3 Communication Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-020 | The spacecraft shall transmit a beacon at least every 60 seconds | T | Critical |
| MRD-021 | The beacon shall include spacecraft health telemetry | T | Critical |
| MRD-022 | The spacecraft shall be detectable by amateur ground stations | T | Critical |
| MRD-023 | The downlink frequency shall be coordinated with IARU | I | Critical |
| MRD-024 | The downlink shall comply with all applicable amateur radio regulations | I | Critical |
| MRD-025 | All transmissions shall be in the clear (no encryption) per Part 97 | I | Critical |

### 5.4 Transponder Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-030 | The spacecraft shall provide amateur radio transponder capability | D | Primary |
| MRD-031 | The transponder shall support linear and/or FM modes | T | Primary |
| MRD-032 | The transponder shall be accessible to licensed amateur operators | D | Primary |
| MRD-033 | Transponder operation shall be managed by onboard AI when available | D | Primary |
| MRD-034 | The spacecraft shall revert to conventional transponder mode if AI is unavailable | D | Critical |

### 5.5 AI/ML Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-040 | The spacecraft shall incorporate onboard AI for transponder management | D | Primary |
| MRD-041 | The AI shall learn power availability patterns | D | Primary |
| MRD-042 | The AI shall predict regional amateur radio demand | D | Secondary |
| MRD-043 | The AI shall prioritize QSOs fairly | D | Primary |
| MRD-044 | The AI shall compose dynamic beacon messages | D | Primary |
| MRD-045 | AI operations shall not compromise spacecraft safety | A | Critical |

### 5.6 Power Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-050 | The spacecraft shall generate sufficient power for all mission modes | A,T | Critical |
| MRD-051 | The spacecraft shall survive eclipse periods of up to 35 minutes | A,T | Critical |
| MRD-052 | Power-intensive operations shall be duty-cycled based on power availability | D | High |
| MRD-053 | AI payload power consumption shall be managed to maintain positive power budget | D | Critical |

### 5.7 Attitude Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-060 | The spacecraft shall detumble within 7 days of deployment | T | Critical |
| MRD-061 | The spacecraft shall maintain attitude knowledge sufficient for antenna pointing | A | High |
| MRD-062 | The spacecraft shall stabilize attitude for nominal operations | T | High |

### 5.8 Ground Segment Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-070 | The mission shall support ground station reception using amateur equipment | D | Critical |
| MRD-071 | Beacon telemetry shall be decodable by amateur operators | D | High |
| MRD-072 | The mission shall coordinate with amateur satellite tracking networks | I | High |

### 5.9 Safety Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-080 | The spacecraft shall comply with all launch provider safety requirements | I | Critical |
| MRD-081 | The spacecraft shall not create debris during nominal operations | I | Critical |
| MRD-082 | The spacecraft shall passivate at end of life | D | Critical |
| MRD-083 | The spacecraft shall be trackable by ground-based surveillance | I | Critical |

### 5.10 Regulatory Requirements

| ID | Requirement | Verification | Priority |
|----|-------------|--------------|----------|
| MRD-090 | The mission shall obtain IARU frequency coordination | I | Critical |
| MRD-091 | The mission shall obtain required national spectrum authorization | I | Critical |
| MRD-092 | The mission shall comply with ITU Radio Regulations | I | Critical |
| MRD-093 | The mission shall comply with orbital debris mitigation requirements | I | Critical |

---

## 6. Verification Methods

| Code | Method | Description |
|------|--------|-------------|
| A | Analysis | Mathematical analysis, simulation, or modeling |
| D | Demonstration | Functional demonstration of capability |
| I | Inspection | Review of documentation, certificates, or design |
| T | Test | Physical test of hardware/software |

---

## 7. Requirements Traceability

### 7.1 Traceability to Objectives

| Requirement | MO-1 | MO-2 | MO-3 | MO-4 | MO-5 |
|-------------|:----:|:----:|:----:|:----:|:----:|
| MRD-001 | X | X | X | X | X |
| MRD-020-025 | X | X | X | | X |
| MRD-030-034 | X | X | X | | X |
| MRD-040-045 | X | | X | X | |
| MRD-050-053 | X | X | X | X | |

### 7.2 Flow-down to System Requirements

| Mission Req | System Requirements |
|-------------|---------------------|
| MRD-001 | SYS-001, SYS-010-020 |
| MRD-020-025 | SYS-100-120 |
| MRD-030-034 | SYS-130-150 |
| MRD-040-045 | SYS-200-230 |
| MRD-050-053 | SYS-300-330 |
| MRD-060-062 | SYS-400-420 |

---

## 8. TBDs and TBRs

| ID | Item | Closure Plan | Target Date |
|----|------|--------------|-------------|
| TBD-001 | Exact orbit parameters | Launch manifest assignment | TBD |
| TBD-002 | IARU coordination confirmation | IARU process completion | TBD |
| TBR-001 | AI model final selection | CDR | TBD |

---

## 9. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
