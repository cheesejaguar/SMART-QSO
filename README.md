# SMART-QSO CubeSat Mission

**Self-Managing Amateur Radio Transponder — QSO**

A 1U CubeSat mission experimenting with onboard agentic AI to manage an amateur radio transponder. The satellite learns power availability and regional demand, prioritizes fair access, and composes dynamic beacons while always failing gracefully to a conventional transponder if AI is unavailable.

[![Flight Software CI](https://github.com/cheesejaguar/SMART-QSO/actions/workflows/ci.yml/badge.svg)](https://github.com/cheesejaguar/SMART-QSO/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## Mission Status

> **Funding audit (2026-10-14):** an independent audit for prospective funders is in
> [docs/funding_audit/](docs/funding_audit/FUNDING_AUDIT_REPORT.md). It contains the verdict, 103 findings, the
> launch/rideshare trade, a cost ROM, a standards matrix and draft regulatory filings. The single source of truth for
> design values is [docs/BASELINE_PARAMETERS.md](docs/BASELINE_PARAMETERS.md).

| Milestone | Status | Date |
|-----------|--------|------|
| System Requirements Review (SRR) | Internal self-assessment only (no independent review) | 2026-01-02 |
| Preliminary Design Review (PDR) | Internal self-assessment only (no independent review) | 2026-01-02 |
| Critical Design Review (CDR) | Internal self-assessment only (no independent review) | 2026-01-02 |
| Test Readiness Review (TRR) | Not held | - |
| Flight Readiness Review (FRR) | Not held | - |
| Regulatory filings (IARU, FCC Part 97, ITU) | Drafted, not submitted | - |
| Hardware | None built | - |

**Current Phase**: Phase B (pre-PDR). Design decisions D1–D7 taken 2026-10-15 ([baseline](docs/BASELINE_PARAMETERS.md) §9). The flight software is Rust. It runs as a host simulation and builds as an STM32L4R5ZI image (Embassy) that CI checks but no hardware has run; no hardware has been built.

---

## Key Technologies

- **Compute**: Ultra-low-power OBC (STM32L4R5) + Jetson Orin Nano module (7 W mode, power-gated, burst-only)
- **AI**: a calibrated selector on the OBC chooses pre-approved beacon text and schedules the transponder (flight baseline); Jetson-generated text is a gated experiment, enabled only if measured energy per beacon is ≤ 0.1 Wh. All beacon text passes a deterministic content filter ([selection](docs/payload/AI_MODEL_SELECTION.md))
- **RF**: U/V FM transponder; downlink in 145.800–146.000 MHz and uplink in 435–438 MHz (to be IARU-coordinated); 1200 bps AX.25 telemetry; authenticated telecommand with hardware TX-OFF
- **ADCS**: Magnetorquer-based B-dot detumbling and coarse sun-pointing
- **Navigation**: duty-cycled GNSS receiver for operator ephemeris (SpaceX rideshare requirement; decision D6)
- **Power**: Body-mounted cells plus two deployable solar wings, ≈ 50 Wh 4S 18650 battery, MPPT charging

---

## Quick Start

```bash
# Clone repository
git clone https://github.com/cheesejaguar/SMART-QSO.git
cd SMART-QSO

# Build the flight software (Rust, built by CMake through cargo; needs CMake, a C
# compiler and CMocka for the C test suites, and Rust from https://rustup.rs:
# software/rust/rust-toolchain.toml pins it)
cmake -S software/flight -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build

# Run the C test suites (the specification) against the Rust
ctest --test-dir build --output-on-failure

# Rust: unit tests and lints (software/rust/README.md)
(cd software/rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings)

# The STM32L4R5ZI image (Embassy firmware)
(cd software/rust/firmware && cargo build --release)

# Python payload and ground tests (they use the Rust wire formats)
pip install software/rust/sqso-py && pip install -e software/payload
python -m pytest -q software/payload/tests software/ground/tests
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
│   ├── flight/              # C headers (the ABI of the flight software), C test suites, CMake
│   ├── rust/                # The flight software (Rust), the STM32L4R5ZI firmware, sqso_protocol
│   ├── ground/              # Ground station tools (Python)
│   ├── payload/             # Jetson payload software
│   └── tests/               # Integration tests
└── RALPH.md                 # NASA code quality audit checklist
```

---

## Flight Software Architecture

The flight software applies tailored NASA practices (formal software classification per NPR 7150.2D is pending), with design ideas drawn from public NASA Ames SmallSat missions (GeneSat, PharmaSat, O/OREOS, EcAMSat). **Current state (2026-10-24):** all flight software is Rust (`software/rust/README.md`, with a porting table from every former C file). It runs as a host simulation and as the STM32L4R5ZI firmware on the Embassy executor (no RTOS), which CI builds and checks but no hardware has run. The only C left in the flight code is the flight log's printf shim. The C headers are the ABI of the Rust modules, and the C test suites run unchanged against it.

- **Executive**: one main loop of stages, every 20 ms, each stage supervised by the watchdog ([run-time architecture](software/flight/docs/RTOS_ARCHITECTURE.md))
- **State Machine**: BOOT, SAFE, DETUMBLE, IDLE, ACTIVE and EMERGENCY modes (one guarded mode manager)
- **FDIR**: Fault Detection, Isolation, and Recovery per NASA-STD-8719.13
- **Watchdog**: Hardware and software watchdog management
- **Telemetry**: AX.25 beacon generation with AI-enhanced content

### Software Modules

Measured 2026-10-24 (CI job Build and Test): line coverage of the Rust flight crates is **92.1 %**, from the C suites
and 197 Rust unit tests together (CI fails below 90 %), and of the flight software, C and Rust together, **90.0 %** (CI
fails below 80 %). There are 42 CMocka suites with 630 test cases, plus the no-dynamic-memory check. Every line of the
flight crates counts, including lines no test runs. The per-file report is the CI artifact `rust-coverage`. Reproduce
it with `software/flight/scripts/rust_coverage.sh` after a coverage build (`software/flight/tests/README.md`). Branch
coverage is not measured.

| Module (Rust) | Description |
|---------------|-------------|
| `eps_control`, `payload_guard` | Power management, battery protection, OBC-enforced Jetson limits |
| `telemetry`, `beacon` | Telemetry frames, AX.25 beacon generation |
| `adcs_control` | B-dot and sun-pointing algorithms |
| `sensors`, `gnss` | Sensor sampling, GNSS receiver |
| `uart_comm`, `jetson_link` | Serial link to the Jetson |
| `uplink`, `cmd_auth`, `cmd_handler` | Telecommand reception, authentication, dispatch |
| `rf_control`, `rf_silence`, `contact_monitor`, `deployment` | TX-OFF, RF silence, loss-of-contact inhibit, deployment |
| `state_machine`, `fault_mgmt`, `watchdog_mgr` | Mode manager, fault logging and recovery, supervised watchdog |
| `executive` | Bring-up and main loop (was `main.c`); run by the simulation in CI |

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

## NASA Standards

No standard has yet been demonstrated compliant, because no signed compliance matrices or verification evidence
exist. Status per [docs/funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md](docs/funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md):

| Standard | Description | Status |
|----------|-------------|--------|
| NPR 7150.2D | NASA Software Engineering Requirements | Gap: classification and tailored compliance matrix needed |
| NPR 7123.1D | Systems Engineering Processes and Requirements | Partial |
| NASA-STD-8739.8B | Software Assurance and Software Safety (replaced the cancelled NASA-STD-8719.13) | Gap |
| NASA-STD-8719.14C | Process for Limiting Orbital Debris | Tailored plan (ODAR drafted; DAS run pending) |
| CDS Rev 14.1 | CubeSat Design Specification | Partial |
| JPL Power of Ten | Coding rules | Not met: claims in the self-audit are contradicted by measurement |

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

- **Amateur-Satellite Service (FCC Part 97)**: all downlink frames in the clear; only telecommand is authenticated (47 CFR 97.211(b)). See [licensing pathway](docs/regulatory/LICENSING_PATHWAY_DETERMINATION.md)
- **Open Operations**: beacons and the transponder are usable by any licensed amateur station
- **Community model updates ("federated learning")**: not in the flight baseline and not implemented onboard; ground-side research only. Any future upload would be data only, adopted solely by authenticated telecommand from a designated station (47 CFR 97.211). Whether to keep or drop the idea is a PI decision

---

## License

MIT License - See [LICENSE](LICENSE) for details.

---

## Acknowledgments

This project draws inspiration from the NASA Ames Research Center Small Satellite Division and their heritage missions including GeneSat, PharmaSat, O/OREOS, and EcAMSat.

---

## Citing This Work

See [CITATION.cff](CITATION.cff) for citation information.
