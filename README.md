# SMART-QSO CubeSat Mission

**Self-Managing Amateur Radio Transponder — QSO**

A 1U CubeSat mission experimenting with onboard agentic AI to manage an amateur radio transponder. The satellite learns power availability and regional demand, prioritizes fair access, and composes dynamic beacons while always failing gracefully to a conventional transponder if AI is unavailable.

[![Flight Software CI](https://github.com/cheesejaguar/SMART-QSO/actions/workflows/ci.yml/badge.svg)](https://github.com/cheesejaguar/SMART-QSO/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## Mission Status

| Milestone | Status | Date |
|-----------|--------|------|
| System Requirements Review (SRR) | Complete | - |
| Preliminary Design Review (PDR) | Complete | - |
| Critical Design Review (CDR) | **Baseline** | 2026-01-02 |
| Test Readiness Review (TRR) | Pending | - |
| Flight Readiness Review (FRR) | Pending | - |

**Current Phase**: Phase C - Detailed Design & Development

---

## Key Technologies

- **Compute**: Ultra-low-power OBC (STM32L4/Apollo4) + Jetson Orin Nano Super (declocked, power-gated)
- **AI**: TinyML/TinyLM models for power-aware beacon generation
- **RF**: VHF downlink (145.825 MHz), UHF uplink (435 MHz), AX.25 protocol
- **ADCS**: Magnetorquer-based B-dot detumbling and sun-pointing
- **Power**: Deployable solar arrays, 50Wh battery, MPPT charging

---

## Quick Start

```bash
# Clone repository
git clone https://github.com/cheesejaguar/SMART-QSO.git
cd SMART-QSO

# Build flight software
cmake -S software/flight -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build

# Run unit tests
ctest --test-dir build --output-on-failure

# Run static analysis
cppcheck --enable=all --std=c11 -I software/flight/include software/flight/src/
```

---

## Repository Structure

```
SMART-QSO/
├── docs/                    # Project documentation
│   ├── compliance/          # CDS, export control, ODAR
│   ├── environment/         # Power, thermal, mass, link budgets
│   ├── hardware/            # Subsystem design documents
│   ├── interfaces/          # Interface Control Documents
│   ├── operations/          # Mission ops, contingency procedures
│   ├── reviews/             # PDR, CDR, TRR, FRR packages
│   ├── safety/              # Hazard analysis, FMEA, safety plan
│   ├── software/            # Software design, requirements
│   ├── test_procedures/     # 40+ formal test procedures
│   └── verification/        # V&V plan, traceability matrices
├── hardware/                # Hardware design documents
│   ├── adcs/                # ADCS electrical/mechanical
│   ├── eps/                 # EPS, battery, solar array
│   ├── obc/                 # OBC design
│   ├── payload-jetson/      # Jetson integration
│   ├── rf/                  # RF subsystem
│   └── structure/           # Mechanical, thermal
├── mission/                 # Mission documents
│   ├── CONOPS.md            # Concept of Operations
│   ├── RISK_REGISTER.md     # Risk management
│   └── VNV_PLAN.md          # Verification & Validation
├── software/                # Source code
│   ├── flight/              # OBC flight software (C)
│   ├── ground/              # Ground station tools (Python)
│   ├── payload/             # Jetson payload software
│   └── tests/               # Integration tests
└── RALPH.md                 # NASA code quality audit checklist
```

---

## Flight Software Architecture

The flight software follows NASA Class C software standards with influences from NASA Ames SmallSat heritage (GeneSat, PharmaSat, O/OREOS, EcAMSat):

- **Scheduler**: Cooperative task scheduling with priority levels
- **State Machine**: INIT → SAFE → IDLE → ACTIVE → AI_BURST modes
- **FDIR**: Fault Detection, Isolation, and Recovery per NASA-STD-8719.13
- **Watchdog**: Hardware and software watchdog management
- **Telemetry**: AX.25 beacon generation with AI-enhanced content

### Software Modules

| Module | Description | Test Coverage |
|--------|-------------|---------------|
| `eps_control` | Power management, battery protection | 90%+ |
| `fault_mgmt` | Fault logging, recovery actions | 90%+ |
| `beacon` | AX.25 frame generation | 85%+ |
| `adcs_control` | B-dot and sun-pointing algorithms | 80%+ |
| `state_machine` | Mode transitions, safe mode | 90%+ |
| `scheduler` | Task scheduling, timing | 85%+ |

---

## Documentation Highlights

### Design Documents
- [Software Design](docs/software/SOFTWARE_DESIGN.md) - Architecture and module design
- [FDIR Design](software/flight/docs/FDIR_DESIGN.md) - Fault handling approach
- [Memory Policy](software/flight/docs/MEMORY_POLICY.md) - No dynamic allocation

### Interface Control Documents
- [OBC-EPS ICD](docs/interfaces/ICD_OBC_EPS.md) - Power subsystem interface
- [OBC-Jetson ICD](docs/interfaces/ICD_OBC_JETSON.md) - AI payload interface
- [Space-Ground ICD](docs/interfaces/ICD_SPACE_GROUND.md) - RF link specification

### Review Packages
- [CDR Package](docs/reviews/CDR_PACKAGE.md) - Critical Design Review materials
- [V&V Package](docs/reviews/VERIFICATION_VALIDATION_PACKAGE.md) - Test results

### Safety & Compliance
- [Software Hazard Analysis](docs/safety/SOFTWARE_HAZARD_ANALYSIS.md)
- [Software FMEA](docs/safety/SOFTWARE_FMEA.md)
- [MISRA Deviations](software/flight/docs/MISRA_DEVIATIONS.md)

---

## NASA Standards Compliance

This project follows NASA software engineering standards:

| Standard | Description | Status |
|----------|-------------|--------|
| NPR 7150.2 | NASA Software Engineering Requirements | Compliant |
| NPR 7123.1 | Systems Engineering Processes | Compliant |
| NASA-STD-8719.13 | Software Safety Standard | Compliant |
| JPL Power of Ten | Coding rules for safety-critical software | Audited |

---

## Contributing

Contributions are welcome! Please read:
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community standards
- [docs/development/CODING_STANDARDS.md](docs/development/CODING_STANDARDS.md) - Code style

All pull requests require:
- Passing CI checks (build, tests, static analysis)
- Code review approval
- Requirements traceability for new features

---

## Governance & Safety

- **Amateur-Satellite Service**: All frames transmitted in the clear (no encryption)
- **Open Operations**: Beacons receivable by any licensed amateur station
- **Federated Updates**: Bounded, rate-limited, sandboxed, publicly logged

---

## License

MIT License - See [LICENSE](LICENSE) for details.

---

## Acknowledgments

This project draws inspiration from the NASA Ames Research Center Small Satellite Division and their heritage missions including GeneSat, PharmaSat, O/OREOS, and EcAMSat.

---

## Citing This Work

See [CITATION.cff](CITATION.cff) for citation information.
