# System Requirements Specification
## SMART-QSO CubeSat Mission

**Document Number:** SQSO-REQ-002
**Revision:** A
**Date:** 2026-01-02
**Reference:** NPR 7123.1, NASA Ames SmallSat Heritage

---

## 1. Purpose

This document defines the Level 2 (System) requirements for the SMART-QSO CubeSat. These requirements are derived from the Mission Requirements Document (MRD) and flow down to subsystem and software requirements.

## 2. Scope

This document covers:
- Functional requirements
- Performance requirements
- Interface requirements
- Environmental requirements
- Verification approach

## 3. Applicable Documents

| Document | Title |
|----------|-------|
| SQSO-REQ-001 | Mission Requirements Document |
| NPR 7123.1 | NASA Systems Engineering Processes and Requirements |
| Cal Poly CDS Rev 14.1 | CubeSat Design Specification |
| GSFC-STD-7000 | GEVS Environmental Standard |

---

## 4. System Overview

### 4.1 System Description

SMART-QSO is a 1U CubeSat consisting of:
- Electrical Power Subsystem (EPS)
- Command & Data Handling (C&DH/OBC)
- Communication Subsystem (RF)
- Attitude Determination & Control (ADCS)
- AI Payload (Jetson Orin Nano)
- Structure and Thermal

### 4.2 Operational Modes

| Mode | Description | Power |
|------|-------------|-------|
| SAFE | Minimum operations, beacon only | <1W |
| IDLE | Normal standby, monitoring | <2W |
| ACTIVE | Full operations, no AI | <4W |
| BURST | AI inference active | <10W |

---

## 5. General Requirements

### 5.1 Physical Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-001 | The spacecraft shall conform to 1U CubeSat form factor per Cal Poly CDS Rev 14.1 | MRD-080 | I,T |
| SYS-002 | The spacecraft mass shall not exceed 2.0 kg | MRD-001 | T |
| SYS-003 | The spacecraft center of gravity shall be within 2 cm of the geometric center | MRD-080 | T |
| SYS-004 | The spacecraft shall have deployment switches per CDS requirements | MRD-080 | I,T |
| SYS-005 | The spacecraft shall have a Remove Before Flight (RBF) pin | MRD-080 | I |

### 5.2 Lifetime Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-010 | The spacecraft shall be designed for 12-month operational life | MRD-001 | A |
| SYS-011 | All components shall be derated per EEE-INST-002 guidelines | MRD-001 | A |
| SYS-012 | The spacecraft shall survive 10,000 thermal cycles | MRD-001 | A |
| SYS-013 | The spacecraft shall survive the radiation environment for mission duration | MRD-001 | A |

### 5.3 Reliability Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-020 | Single-point failures shall be identified and mitigated where practical | MRD-001 | A |
| SYS-021 | The spacecraft shall implement watchdog protection on all processors | MRD-002 | D |
| SYS-022 | The spacecraft shall be capable of autonomous recovery from single-event upsets | MRD-002 | D |
| SYS-023 | Critical data shall be protected by CRC or equivalent integrity mechanism | MRD-002 | D |

---

## 6. Electrical Power Subsystem Requirements

### 6.1 Power Generation

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-100 | The EPS shall generate minimum 2W orbit-average power | MRD-050 | A,T |
| SYS-101 | The EPS shall incorporate body-mounted and/or deployable solar arrays | MRD-050 | I |
| SYS-102 | The EPS shall track maximum power point | MRD-050 | T |
| SYS-103 | Solar array output shall support battery charging plus loads in sunlight | MRD-050 | A,T |

### 6.2 Energy Storage

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-110 | The EPS shall include rechargeable battery storage | MRD-051 | I |
| SYS-111 | Battery capacity shall support 35-minute eclipse at IDLE mode | MRD-051 | A,T |
| SYS-112 | Battery depth of discharge shall not exceed 30% during normal operations | MRD-051 | A |
| SYS-113 | The EPS shall provide battery charge and discharge protection | MRD-051 | D |
| SYS-114 | The EPS shall report battery state of charge | MRD-051 | D |

### 6.3 Power Distribution

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-120 | The EPS shall provide regulated 3.3V rail | MRD-050 | T |
| SYS-121 | The EPS shall provide regulated 5V rail for AI payload | MRD-050 | T |
| SYS-122 | The EPS shall provide switchable power to subsystems | MRD-052 | D |
| SYS-123 | The EPS shall provide overcurrent protection on all rails | MRD-050 | T |
| SYS-124 | The EPS shall report voltage, current, and temperature telemetry | MRD-021 | D |

---

## 7. Communication Subsystem Requirements

### 7.1 Transmitter Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-200 | The transmitter shall operate on 145.825 MHz (or IARU-coordinated frequency) | MRD-023 | T |
| SYS-201 | The transmitter shall support 0.5W to 2W selectable output power | MRD-022 | T |
| SYS-202 | The transmitter shall support AFSK 1200 bps modulation | MRD-022 | T |
| SYS-203 | Spurious emissions shall be at least 40 dB below carrier | MRD-024 | T |
| SYS-204 | Frequency accuracy shall be within 5 kHz | MRD-024 | T |

### 7.2 Beacon Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-210 | The spacecraft shall transmit AX.25 formatted beacons | MRD-022 | T |
| SYS-211 | Beacon interval shall be configurable from 10 to 120 seconds | MRD-020 | D |
| SYS-212 | Beacons shall include spacecraft callsign | MRD-024 | I,T |
| SYS-213 | Beacons shall include health telemetry in machine-readable format | MRD-021 | D |
| SYS-214 | Beacons shall include human-readable information field | MRD-044 | D |
| SYS-215 | Beacon content shall be generated by AI when available, template fallback otherwise | MRD-034 | D |

### 7.3 Antenna Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-220 | The antenna shall deploy after separation from deployer | MRD-080 | T |
| SYS-221 | Antenna deployment shall be inhibited until 30 minutes after deployment switch activation | MRD-080 | D |
| SYS-222 | The antenna shall provide omnidirectional coverage | MRD-022 | A |
| SYS-223 | Antenna VSWR shall be less than 2:1 at operating frequency | MRD-022 | T |

---

## 8. Command & Data Handling Requirements

### 8.1 OBC Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-300 | The OBC shall execute flight software within 30 seconds of power-on | MRD-002 | T |
| SYS-301 | The OBC shall maintain mission elapsed time | MRD-002 | D |
| SYS-302 | The OBC shall interface with all subsystems | MRD-001 | D |
| SYS-303 | The OBC shall collect and store telemetry | MRD-021 | D |
| SYS-304 | The OBC shall manage operational mode transitions | MRD-052 | D |
| SYS-305 | The OBC shall implement fault detection and recovery | MRD-002 | D |

### 8.2 Data Storage Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-310 | The OBC shall store at least 1 MB of mission data | MRD-001 | I |
| SYS-311 | Stored data shall be protected against corruption | MRD-002 | D |
| SYS-312 | Critical configuration shall persist across power cycles | MRD-002 | D |
| SYS-313 | Fault log shall persist across resets | MRD-002 | D |

### 8.3 Interface Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-320 | The OBC shall communicate with EPS via UART or I2C | MRD-001 | D |
| SYS-321 | The OBC shall communicate with AI payload via UART | MRD-040 | D |
| SYS-322 | The OBC shall control RF transmitter enable | MRD-020 | D |
| SYS-323 | The OBC shall read ADCS sensors | MRD-060 | D |
| SYS-324 | The OBC shall control ADCS actuators | MRD-062 | D |

---

## 9. Attitude Determination & Control Requirements

### 9.1 Sensor Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-400 | The ADCS shall include a 3-axis magnetometer | MRD-061 | I |
| SYS-401 | Magnetometer accuracy shall be better than 5 degrees | MRD-061 | T |
| SYS-402 | The ADCS shall include sun sensors on at least 4 faces | MRD-061 | I |
| SYS-403 | Sun sensor accuracy shall be better than 10 degrees | MRD-061 | T |

### 9.2 Actuator Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-410 | The ADCS shall include 3-axis magnetorquers | MRD-060 | I |
| SYS-411 | Magnetorquer dipole moment shall be at least 0.1 Am² per axis | MRD-060 | T |
| SYS-412 | Magnetorquers shall be independently controllable | MRD-062 | D |

### 9.3 Control Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-420 | The ADCS shall detumble the spacecraft to <5 deg/s within 7 days | MRD-060 | T |
| SYS-421 | The ADCS shall implement B-dot detumble control | MRD-060 | D |
| SYS-422 | The ADCS shall achieve gravity gradient or sun-pointing stabilization | MRD-062 | D |

---

## 10. AI Payload Requirements

### 10.1 Hardware Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-500 | The AI payload shall use NVIDIA Jetson Orin Nano or equivalent | MRD-040 | I |
| SYS-501 | The AI payload peak power shall not exceed 10W | MRD-053 | T |
| SYS-502 | The AI payload idle power shall not exceed 5W | MRD-053 | T |
| SYS-503 | The AI payload shall be power-gated by OBC command | MRD-052 | D |
| SYS-504 | The AI payload shall operate within thermal limits passively | MRD-001 | A,T |

### 10.2 Software Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-510 | The AI payload shall run TinyLM model for text generation | MRD-044 | D |
| SYS-511 | The AI payload shall receive health data from OBC | MRD-041 | D |
| SYS-512 | The AI payload shall generate beacon text within 10 seconds | MRD-044 | T |
| SYS-513 | Generated text shall be ASCII-only and within length limits | MRD-044 | D |
| SYS-514 | The AI payload shall report health status to OBC | MRD-045 | D |

### 10.3 Fallback Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-520 | The OBC shall detect AI payload failure or timeout | MRD-034 | D |
| SYS-521 | The OBC shall use template beacons if AI is unavailable | MRD-034 | D |
| SYS-522 | AI failure shall not affect other spacecraft functions | MRD-034 | D |
| SYS-523 | The OBC shall automatically power-cycle AI payload on repeated failures | MRD-034 | D |

---

## 11. Thermal Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-600 | All components shall operate within -20C to +50C | MRD-001 | A,T |
| SYS-601 | All components shall survive -40C to +60C | MRD-001 | A,T |
| SYS-602 | Battery temperature shall be maintained above 0C for charging | MRD-051 | A |
| SYS-603 | AI payload thermal path shall be designed for passive cooling | MRD-001 | A |

---

## 12. Environmental Requirements

### 12.1 Launch Environment

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-700 | The spacecraft shall survive launch vibration per GEVS protoflight levels | MRD-080 | T |
| SYS-701 | The spacecraft shall survive acoustic environment per launch provider spec | MRD-080 | A |
| SYS-702 | The spacecraft shall survive depressurization | MRD-080 | T |

### 12.2 Space Environment

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-710 | The spacecraft shall survive radiation environment for mission duration | MRD-001 | A |
| SYS-711 | The spacecraft shall tolerate single-event upsets | MRD-002 | D |
| SYS-712 | All materials shall meet outgassing requirements per ASTM E595 | MRD-080 | I |

---

## 13. End-of-Life Requirements

| ID | Requirement | Parent | Verification |
|----|-------------|--------|--------------|
| SYS-800 | The spacecraft shall passivate batteries at end of life | MRD-082 | D |
| SYS-801 | The spacecraft shall disable RF transmitter at end of life | MRD-082 | D |
| SYS-802 | The spacecraft shall deploy drag device for deorbit (if applicable) | MRD-083 | D |
| SYS-803 | The spacecraft shall deorbit within 25 years | MRD-083 | A |

---

## 14. Verification Matrix Summary

| Req Range | Analysis | Demonstration | Inspection | Test |
|-----------|:--------:|:-------------:|:----------:|:----:|
| SYS-001-005 | 1 | 0 | 4 | 3 |
| SYS-010-023 | 5 | 4 | 0 | 0 |
| SYS-100-124 | 3 | 3 | 1 | 5 |
| SYS-200-223 | 1 | 3 | 2 | 6 |
| SYS-300-324 | 0 | 12 | 1 | 1 |
| SYS-400-422 | 0 | 3 | 2 | 4 |
| SYS-500-523 | 2 | 9 | 1 | 3 |
| SYS-600-603 | 4 | 0 | 0 | 2 |
| SYS-700-712 | 3 | 1 | 1 | 2 |
| SYS-800-803 | 1 | 3 | 0 | 0 |

---

## 15. TBDs and TBRs

| ID | Item | Closure Plan | Target |
|----|------|--------------|--------|
| TBD-001 | Final IARU coordinated frequency | IARU coordination | TBD |
| TBD-002 | Launch provider vibration spec | Manifest assignment | TBD |
| TBR-001 | AI model final specification | CDR | TBD |

---

## 16. Revision History

| Rev | Date | Description | Author |
|-----|------|-------------|--------|
| A | 2026-01-02 | Initial release | SMART-QSO Team |
