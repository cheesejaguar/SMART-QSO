# Software Development Plan (SDP)

**Document Number**: SMART-QSO-SDP-001
**Version**: 1.0
**Date**: 2026-01-02
**Status**: Draft

**Governing Standards**:
- NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
- NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements
- NPR 7150.2 Rev B - NASA Software Engineering Requirements

**Process Heritage**: NASA Ames Research Center Small Satellite Division (GeneSat, PharmaSat, O/OREOS, EcAMSat)

---

## 1. Introduction

### 1.1 Purpose

This Software Development Plan (SDP) defines the technical and management approach for developing the SMART-QSO flight software. It establishes processes, standards, and controls to ensure software quality and mission success.

### 1.2 Scope

This plan covers:
- On-Board Computer (OBC) flight software
- Jetson AI payload software
- Ground support software
- Simulation and test software

### 1.3 Document Organization

| Section | Content |
|---------|---------|
| 2 | Project Overview |
| 3 | Software Classification |
| 4 | Lifecycle Model |
| 5 | Development Environment |
| 6 | Coding Standards |
| 7 | Configuration Management |
| 8 | Testing Strategy |
| 9 | Verification & Validation |
| 10 | Risk Management |

---

## 2. Project Overview

### 2.1 Mission Description

SMART-QSO is a 1U CubeSat experimenting with onboard agentic AI to manage an amateur radio transponder. The satellite uses TinyML/TinyLM models to learn power availability, predict regional demand, prioritize QSOs fairly, and compose dynamic beacons.

### 2.2 Software Architecture Overview

```
+------------------+     UART     +------------------+
|   OBC (MCU)      |<------------>|   Jetson Orin    |
|   FreeRTOS       |              |   AI Payload     |
+------------------+              +------------------+
        |                                 |
        v                                 v
+------------------+              +------------------+
|   EPS Control    |              |   TinyLM Model   |
|   ADCS Control   |              |   Beacon Gen     |
|   RF Beacon TX   |              +------------------+
+------------------+
```

### 2.3 Software Components

| Component | Language | Target | Classification |
|-----------|----------|--------|----------------|
| OBC Flight Software | C | STM32L4/Apollo4 | Class D |
| Jetson Payload SW | Python | Jetson Orin Nano | Class D |
| Ground Software | Python | Linux/Windows | Class E |
| Simulation | Python/C | Linux | Class E |

---

## 3. Software Classification

### 3.1 Classification per NPR 7150.2

Per NPR 7150.2 Table 1-1, SMART-QSO software is classified as:

**Class D** - Non-safety-critical, recoverable consequences of software failure

Rationale:
- CubeSat mission with acceptable risk tolerance
- No crewed operations
- Fail-graceful design with fallback to conventional transponder
- Limited science objectives (technology demonstration)

### 3.2 Safety Classification per NASA-STD-8719.13

The software is classified as:
- **Not safety-critical** - No software-controlled functions can cause personnel hazards
- **Mission-critical** - Software failures can affect mission success but are recoverable

### 3.3 Tailoring Rationale

Per NPR 7150.2 Appendix C, the following tailoring is applied:

| Requirement | Tailoring | Rationale |
|-------------|-----------|-----------|
| Formal inspection | Peer review | Small team, Class D |
| MC/DC coverage | Branch coverage | Class D missions |
| Independent V&V | Team V&V | Limited budget |
| Tool qualification | Documented tools | Class D |

---

## 4. Lifecycle Model

### 4.1 Development Lifecycle

SMART-QSO follows an incremental development model with the following phases:

```
Phase A ──> Phase B ──> Phase C ──> Phase D ──> Phase E
(Concept)   (Prelim)   (Final     (System    (Operations)
            (Design)    Design)    Assembly)

           SRR       PDR        CDR       TRR/FRR
            │         │          │          │
            ▼         ▼          ▼          ▼
```

### 4.2 Phase Activities

#### Phase B (Preliminary Design)
- Define software requirements
- Develop preliminary architecture
- Prototype critical algorithms
- Establish development environment

#### Phase C (Final Design & Implementation)
- Complete detailed design
- Implement all modules
- Unit testing
- Integration testing

#### Phase D (System Assembly & Test)
- System integration
- Environmental testing
- Flight software verification
- Configuration audit

### 4.3 Milestones and Reviews

| Review | Entry Criteria | Exit Criteria |
|--------|----------------|---------------|
| SRR | ConOps complete | Requirements approved |
| PDR | Requirements baselined | Design approach approved |
| CDR | Detailed design complete | Build/test authorized |
| TRR | Testing complete | Test readiness confirmed |
| FRR | Verification complete | Flight readiness confirmed |

---

## 5. Development Environment

### 5.1 Hardware Platforms

| Platform | Purpose |
|----------|---------|
| Development PC | Code development, simulation |
| STM32L4 NUCLEO | MCU firmware development |
| Jetson Orin Nano Dev Kit | AI payload development |
| Flatsat | System integration testing |

### 5.2 Software Tools

| Tool | Version | Purpose |
|------|---------|---------|
| GCC ARM | 12.x | Flight software compilation |
| CMake | 3.16+ | Build system |
| CMocka | 1.1+ | Unit testing framework |
| LLVM/Clang | 15.x | Static analysis, formatting |
| cppcheck | 2.10+ | Static analysis |
| lcov | 1.16+ | Code coverage |
| Git | 2.x | Version control |
| GitHub Actions | - | CI/CD pipeline |

### 5.3 Target Platforms

| Target | Processor | OS | Description |
|--------|-----------|----|----|
| OBC Flight | STM32L4 or Apollo4 | FreeRTOS | Primary flight computer |
| AI Payload | Jetson Orin Nano | Linux | AI inference engine |
| Ground | x86/ARM | Linux | Ground station software |

---

## 6. Coding Standards

### 6.1 C Language (OBC Flight Software)

The following coding standards apply:

1. **JPL Institutional Coding Standard** (Power of Ten Rules)
2. **MISRA C:2012** Guidelines for critical sections
3. **CERT C** Secure coding guidelines

### 6.2 Key Rules

| Rule | Description | Enforcement |
|------|-------------|-------------|
| JPL-1 | Simple control flow | Code review |
| JPL-2 | Fixed loop bounds | Static analysis |
| JPL-3 | No dynamic memory after init | Linker wrap |
| JPL-4 | Function length < 60 lines | clang-tidy |
| JPL-5 | Minimum 2 assertions per function | Code review |
| JPL-6 | Smallest scope for data | clang-tidy |
| JPL-7 | Check all return values | Compiler warnings |
| JPL-8 | Limited preprocessor use | Code review |
| JPL-9 | Restrict pointer use | Static analysis |
| JPL-10 | Compile with all warnings | CMake |

### 6.3 Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Functions | module_verb_object | `eps_control_payload()` |
| Types | PascalCase + _t | `EpsControlState_t` |
| Constants | UPPER_SNAKE_CASE | `MAX_SENSORS` |
| Global vars | g_ prefix | `g_system_state` |
| Static vars | s_ prefix | `s_initialized` |
| Local vars | lower_snake_case | `current_soc` |

### 6.4 Python (Payload Software)

- PEP 8 style guide
- Type hints required (mypy strict)
- Docstrings for all public functions

---

## 7. Configuration Management

### 7.1 Version Control

- Git repository hosted on GitHub
- Branch protection on main branch
- Signed commits required for releases

### 7.2 Branching Strategy

```
main ─────────────────────────────────────────>
       \           \           \
        feature/xyz develop     release/v1.0
```

| Branch | Purpose |
|--------|---------|
| main | Flight-ready code |
| develop | Integration branch |
| feature/* | Feature development |
| release/* | Release preparation |
| hotfix/* | Critical fixes |

### 7.3 Version Numbering

Semantic Versioning: MAJOR.MINOR.PATCH

| Version | Meaning |
|---------|---------|
| 0.x.x | Development (pre-flight) |
| 1.0.0 | Initial flight release |
| 1.x.0 | Feature additions |
| 1.x.x | Bug fixes |

### 7.4 Configuration Items

| Item | Baseline |
|------|----------|
| Flight software source | CDR |
| Test procedures | CDR |
| Configuration files | TRR |
| Flight software binary | FRR |

---

## 8. Testing Strategy

### 8.1 Test Levels

| Level | Description | Coverage Target |
|-------|-------------|-----------------|
| Unit | Individual functions | 80% statement |
| Integration | Module interactions | 75% branch |
| System | End-to-end functionality | 100% requirements |
| Acceptance | Mission scenarios | All use cases |

### 8.2 Test Types

| Type | Purpose | When |
|------|---------|------|
| Functional | Verify behavior | All phases |
| Regression | Prevent regressions | Each build |
| Performance | Timing verification | Phase C/D |
| Stress | Boundary conditions | Phase D |
| Environmental | TVAC, vibration | Phase D |

### 8.3 Test Automation

- Unit tests run on every commit (CI)
- Integration tests run nightly
- Coverage reports generated automatically
- Static analysis on every PR

---

## 9. Verification & Validation

### 9.1 Verification Methods

| Method | Code | Description |
|--------|------|-------------|
| Analysis | A | Mathematical/analytical verification |
| Inspection | I | Visual or manual examination |
| Demonstration | D | Operational demonstration |
| Test | T | Execution with specific inputs |

### 9.2 Requirements Traceability

All requirements shall be:
1. Uniquely identified (SRS-xxx)
2. Traced to design elements
3. Traced to code (file:line)
4. Traced to test cases
5. Verified and recorded in VCRM

### 9.3 Verification Matrix

See `docs/verification/VERIFICATION_MATRIX.md` for complete VCRM.

---

## 10. Risk Management

### 10.1 Software Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Jetson power consumption | M | H | DVFS limiting, power gating |
| UART communication loss | L | M | Fallback to template beacons |
| Memory corruption | L | H | CRC protection, TMR |
| Watchdog failure | L | H | Hardware watchdog |

### 10.2 Risk Monitoring

- Risks reviewed at each milestone
- New risks added to register as identified
- Closed risks archived with rationale

---

## 11. Documentation Requirements

### 11.1 Required Documents

| Document | Phase | Owner |
|----------|-------|-------|
| Software Development Plan | B | SW Lead |
| Software Requirements Spec | B | SE Lead |
| Software Design Description | C | SW Lead |
| Interface Control Documents | C | SE Lead |
| Software Test Plan | C | Test Lead |
| Software Version Description | D | SW Lead |

### 11.2 Code Documentation

- Doxygen comments for all public functions
- Module-level documentation in headers
- Algorithm descriptions in design doc

---

## 12. Training Requirements

### 12.1 Developer Training

| Topic | Required For |
|-------|--------------|
| Coding standards | All developers |
| Tool usage | All developers |
| Safety-critical design | OBC developers |
| RTOS concepts | OBC developers |

---

## Appendix A: References

1. NPR 7120.5 Rev F - NASA Space Flight Program and Project Management Requirements
2. NPR 7123.1 Rev E - NASA Systems Engineering Processes and Requirements
3. NPR 7150.2 Rev B - NASA Software Engineering Requirements
4. NASA-STD-8719.13 - Software Safety Standard
5. JPL D-60411 - JPL Institutional Coding Standard

---

## Appendix B: Acronyms

| Acronym | Definition |
|---------|------------|
| CDR | Critical Design Review |
| FRR | Flight Readiness Review |
| MCU | Microcontroller Unit |
| OBC | On-Board Computer |
| PDR | Preliminary Design Review |
| SDP | Software Development Plan |
| SRR | System Requirements Review |
| TRR | Test Readiness Review |
| VCRM | Verification Cross-Reference Matrix |

---

**Document History**

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
