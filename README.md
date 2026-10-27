# SMART-QSO CubeSat Mission

**Self-Managing Amateur Radio Transponder — QSO**

A 1U CubeSat mission for the amateur-satellite service: a single-channel U/V FM transponder for worldwide amateur contacts, duty-gated by the power available, with a 1200 bps AX.25 telemetry beacon. Its experiment is onboard AI that schedules the transponder within the power budget and writes plain-language beacon text. If the AI is unavailable, the spacecraft works as a conventional transponder with template beacons.

[![Flight Software CI](https://github.com/cheesejaguar/SMART-QSO/actions/workflows/ci.yml/badge.svg)](https://github.com/cheesejaguar/SMART-QSO/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

---

## Status

SMART-QSO is in **Phase B (pre-PDR)**. No hardware has been built: there is no engineering model and no PCB layout,
and every mass, power, thermal, structural, RF and attitude figure is predicted, not measured. The boards exist as
schematics with selected parts; the flight software is written in Rust, runs as a host simulation and as a full
mission simulation (integration to re-entry), and builds as an STM32L4R5ZI image that CI checks but no hardware has
run (its pin map is provisional). The regulatory filings are drafted, not submitted, and no launch contract exists.
The plan is a Falcon 9 Transporter rideshare through Exolaunch in an EXOpod Nova, about 2028-11-01, into a 475 km
sun-synchronous orbit, for a 2-year mission ([baseline](docs/BASELINE_PARAMETERS.md) §2–§3;
[schedule](mission/SCHEDULE.md)).

**CDR 2026 design review (2026-10-27).** An engineering assessment of the design against the NPR 7123.1D CDR
criteria, carried out by AI reviewers (Claude), closed with no review action open: all 412 RIDs carry a final
disposition, every margin is positive against its requirement, and what only hardware or an outside party can show is
carried as dated verification items and decision points. Its verdict is that the design is at the level of a strong
PDR with CDR-depth analysis, not past a CDR. It is **not an independent review board**: there are no named human
reviewers or signatures, and its chair decisions C-1 to C-24 bind nothing until the PI ratifies them (by 2027-01-12)
([review record](docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md) §1.2, §7).

| Milestone | Status | Date |
|-----------|--------|------|
| SRR, PDR and CDR records of 2026-01-02 | Internal self-assessments, not independent reviews (funding audit PI-03, PI-04) | 2026-01-02 |
| Funding audit for prospective funders | Done ([report](docs/funding_audit/FUNDING_AUDIT_REPORT.md)); a historical record | 2026-10-14 |
| CDR 2026 design review | Engineering assessment by AI reviewers, not an independent board; no review action open | 2026-10-27 |
| PI ratification of the review's chair decisions | Planned (DA-01) | 2027-01-12 |
| Independent PDR | Planned | 2027-01-12 |
| IARU coordination request | Planned | 2027-01-26 |
| Launch services contract | Planned, with the 475 km target and 485 km cap (decision D7) | by 2027-06-30 |
| Independent CDR, with engineering-model test data | Planned | 2027-09-28 |
| Launch | Planning date (last Transporter flight of 2028) | about 2028-11-01 |
| Hardware | None built | - |

Dates: [`mission/SCHEDULE.md`](mission/SCHEDULE.md) (SQSO-SCH-001 Rev E) and the review record §7.3. Design decisions
D1–D9: [baseline](docs/BASELINE_PARAMETERS.md) §9.

---

## Governing Documents

| Document | What it governs |
|----------|-----------------|
| [docs/BASELINE_PARAMETERS.md](docs/BASELINE_PARAMETERS.md) (SQSO-BL-001 Rev E) | Every design value. Where another document disagrees, the baseline governs |
| [docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md](docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md) (SQSO-REV-CDR-002 Rev B) | The current review record: the mission as reviewed, the margins, the chair decisions, the verdict |
| [docs/DOCUMENT_REGISTER.md](docs/DOCUMENT_REGISTER.md) | Every controlled document and its revision |
| [docs/verification/VERIFICATION_MATRIX.md](docs/verification/VERIFICATION_MATRIX.md) | The only verification-status record (VCRM) |
| [mission/SCHEDULE.md](mission/SCHEDULE.md) | The integrated master schedule and the dated decision actions |
| [docs/project/CHANGE_REQUEST_LOG.md](docs/project/CHANGE_REQUEST_LOG.md) | Changes to the baselines |

---

## Key Technologies

- **Compute**: STM32L4R5ZI OBC (flight software in Rust on the Embassy executor, no RTOS) + NVIDIA Jetson Orin Nano 8 GB module on a custom carrier (7 W mode, ≤ 8 W input, power-gated, burst-only)
- **AI**: a calibrated selector on the OBC chooses pre-approved beacon text and proposes the power mode and the transponder schedule (flight baseline, decision D5; designed, not yet implemented). Jetson-generated text is a gated experiment (SmolLM2-360M-Instruct or Qwen3-0.6B), enabled only if the measured energy per accepted beacon line is ≤ 0.1 Wh. All beacon text passes a deterministic content filter ([selection](docs/payload/AI_MODEL_SELECTION.md))
- **RF**: U/V FM transponder; downlink one IARU-coordinated frequency in 145.800–146.000 MHz, uplink requested in 435.222–435.778 MHz (else 436.222–436.500 MHz); 0.5 W nominal, 1 W maximum; 1200 bps AX.25 telemetry; HMAC-authenticated telecommand with hardware TX-OFF; ISIS AntS antennas
- **ADCS**: CubeSpace CR0002 magnetorquer rods; B-dot detumbling and spin-stabilized coarse sun-pointing of +Y at 8°/s
- **Navigation**: duty-cycled GNSS receiver (SkyFox piNAV-NG/FM) for operator ephemeris (SpaceX rideshare requirement; decision D6)
- **Power**: 16 triple-junction cells on the body and two deployable solar wings; 4S1P 18650 battery, ≈ 50 Wh, charged to 4.10 V/cell
- **Disposal**: passive re-entry with the wings deployed, within 5 years of mission end (no separate drag device)

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

# Mission simulation (software in the loop), first day; docs/test/SIL_MISSION_SIMULATION.md
(cd software/rust && cargo build --profile sil -p sqso-sil && ./target/sil/sqso_sil --scenario smoke)
```

---

## Repository Structure

```
SMART-QSO/
├── docs/                    # Project documentation
│   ├── BASELINE_PARAMETERS.md  # Single source of truth for design values
│   ├── DOCUMENT_REGISTER.md    # Controlled documents and revisions
│   ├── compliance/          # CDS, export control, ODAR, materials
│   ├── environment/         # Power, thermal, mass, link, radiation, structural analyses
│   ├── funding_audit/       # Funding audit of 2026-10-14 (historical record) and decision research
│   ├── hardware/            # Subsystem and board designs, build-versus-buy
│   ├── interfaces/          # Interface control documents
│   ├── mission/             # Mission success criteria, launch-vehicle requirements
│   ├── operations/          # Mission operations, commissioning, contingency, end of life
│   ├── payload/             # AI model selection, OBC selector design
│   ├── project/             # SEMP, project plan, CM plan, change requests
│   ├── regulatory/          # Licensing pathway, IARU, draft filings
│   ├── requirements/        # Mission, system and traceability requirements
│   ├── reviews/             # CDR/ (the 2026 design review); earlier self-assessments; review templates
│   ├── safety/              # Hazard analyses, FMEAs, safety data package
│   ├── software/            # Software plans, requirements, design
│   ├── test/                # System test plans, mission simulation
│   ├── test_procedures/     # Test procedures (none run: no hardware)
│   └── verification/        # V&V plan, VCRM, verification-item register
├── hardware/                # Hardware design notes (ADCS, EPS, OBC, Jetson, RF, structure, harness, solar)
├── mission/                 # ConOps, schedule, risk register, V&V summary
├── software/                # Source code
│   ├── flight/              # C headers (the interface of the flight software), CMocka suites, CMake
│   ├── rust/                # The flight software (Rust), STM32L4R5ZI firmware, host and mission simulations, sqso_protocol
│   ├── ground/              # Ground station tools (Python)
│   ├── payload/             # Jetson payload software (beacon-text experiment)
│   ├── simulation/          # Orbit lifetime, design-review analyses, mission-simulation reports, notebooks
│   └── tests/               # Integration tests
├── scripts/                 # Document and traceability checks run in CI
└── RALPH.md                 # Code quality audit checklist of 2026-01-02 (a dated agent task list)
```

---

## Flight Software Architecture

The flight software applies tailored NASA practices (NPR 7150.2D Class C, [SDP](docs/software/SOFTWARE_DEVELOPMENT_PLAN.md) §3.1), with design ideas drawn from public NASA Ames SmallSat missions (GeneSat, PharmaSat, O/OREOS, EcAMSat). **Current state:** all flight software is Rust (`software/rust/README.md`, with a porting table from every former C file). It runs as a host simulation, as the mission simulation (`sqso-sil`, [SIL_MISSION_SIMULATION](docs/test/SIL_MISSION_SIMULATION.md)) and as the STM32L4R5ZI firmware on the Embassy executor (no RTOS; BASELINE §6), which CI builds and checks but no hardware has run. The only C left in the flight code is the flight log's printf shim. The C headers are the interface of the Rust modules, and the C test suites run unchanged against it.

- **Executive**: one main loop of stages, every 20 ms, each stage supervised by the watchdog ([run-time architecture](software/flight/docs/RTOS_ARCHITECTURE.md))
- **State Machine**: BOOT, SAFE, DETUMBLE, IDLE, ACTIVE and EMERGENCY modes (one guarded mode manager)
- **FDIR**: Fault Detection, Isolation, and Recovery, with software safety per NASA-STD-8739.8B
- **Watchdog**: Hardware and software watchdog management
- **Telemetry**: AX.25 beacon generation with AI-enhanced content

### Software Modules

At the CDR 2026 design review (2026-10-27, review record §7.1) 622 Rust tests and 56 C suites passed, with the
safety-critical coverage gate (32 files, 100 % of lines and conditions) and the image and stack checks. CI also gates
line coverage of the Rust flight crates (≥ 90 %), of the flight software as a whole (≥ 80 %) and Rust branch and
condition coverage (≥ 80 %); the gates, the last measured values and how to reproduce them are in
`software/flight/tests/README.md`. The per-file report is the CI artifact `rust-coverage`. All of this runs on the
host: no test has run on flight hardware.

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

### Reviews and Verification
- [CDR 2026 Design Review](docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md) - Current review record (engineering assessment, not an independent board)
- [CDR Criteria Matrix](docs/project/CDR_CRITERIA_MATRIX.md) - The project against the NPR 7123.1D criteria
- [Verification Matrix](docs/verification/VERIFICATION_MATRIX.md) - The only verification-status record
- The earlier [CDR package](docs/reviews/CDR_PACKAGE.md) and [V&V package](docs/reviews/VERIFICATION_VALIDATION_PACKAGE.md) of 2026-01-02 are self-assessments whose claims are not substantiated (funding audit PI-03)

### Safety & Compliance
- [Software Hazard Analysis](docs/safety/SOFTWARE_HAZARD_ANALYSIS.md)
- [Software FMEA](docs/safety/SOFTWARE_FMEA.md)
- [MISRA Deviations](software/flight/docs/MISRA_DEVIATIONS.md)

---

## NASA Standards

No NASA directive is mandatory for this privately funded Part 97 mission; NASA standards are used as standards of
practice, and "Class D" is the project's self-designation (review record §1.3). No standard has yet been demonstrated
compliant, because no signed compliance matrices or verification evidence exist. Status per
[docs/funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md](docs/funding_audit/NASA_STANDARDS_COMPLIANCE_MATRIX.md)
(Rev C, 2026-10-25), with later documents noted:

| Standard | Description | Status |
|----------|-------------|--------|
| NPR 7150.2D | NASA Software Engineering Requirements | Partial: classified Class C (2026-10-25); Appendix C compliance matrix drafted ([SQSO-FSW-012](docs/software/NPR7150_COMPLIANCE_MATRIX.md), draft) |
| NPR 7123.1D | Systems Engineering Processes and Requirements | Partial |
| NASA-STD-8739.8B | Software Assurance and Software Safety (replaced the cancelled NASA-STD-8719.13) | Gap in the audit matrix; safety criticality since determined (SDP §3.2), software safety plan and hazard analysis drafted (`docs/safety/`) |
| NASA-STD-8719.14C | Process for Limiting Orbital Debris | Tailored plan (ODAR Draft D; NASA DAS run planned by 2027-01-08) |
| CDS Rev 14.1 | CubeSat Design Specification | Partial |
| JPL Power of Ten | Coding rules | Not met ([audit](software/flight/docs/JPL_POWER_OF_TEN_AUDIT.md); the earlier "fully compliant" claim is withdrawn) |

---

## Contributing

Contributions are welcome! Please read:
- [CONTRIBUTING.md](CONTRIBUTING.md) - Contribution guidelines
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community standards
- [docs/development/CODING_STANDARDS.md](docs/development/CODING_STANDARDS.md) - Code style

All pull requests require:
- Passing CI checks (build, tests, static analysis)
- Approval by the maintainer ([GOVERNANCE.md](GOVERNANCE.md)); a baseline change also needs a change request
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
