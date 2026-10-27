# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SMART-QSO is a 1U CubeSat mission experimenting with onboard agentic AI to manage an amateur radio transponder. A small calibrated selector on the OBC schedules the transponder within the power budget and chooses pre-approved beacon text (flight baseline); generative beacon text on a power-gated Jetson Orin Nano 8 GB module (7 W mode) is a gated experiment. It fails gracefully to a conventional transponder if AI is unavailable.

## Build Commands

### Flight Software (Rust; C test suites, CMake)
```bash
# Build the flight software library and the host simulation smart_qso_flight
# (CMake runs cargo for software/rust; the Rust toolchain is pinned by
# software/rust/rust-toolchain.toml)
cmake -S software/flight -B build && cmake --build build

# Run the C test suites (the specification) against it, and the simulation
ctest --test-dir build

# STM32L4R5ZI image (Embassy firmware)
(cd software/rust/firmware && cargo build --release)

# Clean build
rm -rf build

# Rust modules alone
(cd software/rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings)
```

## High-Level Architecture

> **Authoritative design values are in `docs/BASELINE_PARAMETERS.md`** (SQSO-BL-001 Rev E, 2026-10-27). Where this file or any
> other document disagrees, the baseline governs. Current review record: `docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md`
> (an engineering assessment by AI reviewers, not an independent board). Funding-audit findings: `docs/funding_audit/FINDINGS_REGISTER.md`.

### System Components
1. **Bus**: 1U CubeSat with deployable solar wings, UHF/VHF transponder, magnetorquer ADCS
2. **Compute**: 
   - Primary: Ultra-low-power OBC (STM32L4R5ZI). The flight software (Rust) runs as a host simulation and as the Embassy firmware for the STM32L4R5ZI (`software/rust/firmware`), which CI builds and checks; it has not run on hardware and its pin map is provisional
   - AI Payload: Jetson Orin Nano 8 GB module on a custom carrier, 7 W power mode (≤ 8 W input), power-gated for burst inference (BASELINE §6; not "Super", TB-13)
3. **AI Agents** (power-aware):
   - OBC selector (flight baseline, decision D5): gradient-boosted trees or a tiny INT8 MLP that proposes the power mode and the transponder schedule and picks beacon templates, abstaining to fixed rules; hard limits stay outside the model. Designed, not yet implemented (`docs/payload/OBC_SELECTOR_DESIGN.md`). It sets no QSO priority: the FM repeater has no per-station control (§3.4 of that design)
   - Smart Beacon Agent: a selector on the MCU picks pre-approved templates (flight baseline, decision D5); Jetson-generated text is a gated experiment (SmolLM2-360M or Qwen3-0.6B), enabled only if measured energy per accepted beacon is ≤ 0.1 Wh. Both sides apply the same content filter (`docs/BEACON_SPEC.md` §6)

### Software Architecture
- **Flight** (`software/flight/`): the C headers (the interface of the flight modules), the CMocka test suites that specify them, and one C shim (`src/flight_log_fmt.c`: printf-style `log_write`, which stable Rust cannot define). No RTOS: the firmware's Embassy executor runs the main loop
- **Rust** (`software/rust/`): all flight software, behind the C headers. `sqso-protocol` (wire formats) and `sqso-fsw` (flight logic) are `no_std` with `unsafe` forbidden; `sqso-ffi` is the flight software behind the headers (with the executive and the simulated HAL); `sqso-clib` is the static library the C tests link; `sqso-sim` is the host simulation; `sqso-sil` is the mission simulation (software in the loop, integration to re-entry: `docs/test/SIL_MISSION_SIMULATION.md`); `firmware` is the STM32L4R5 image; `sqso-py` gives the payload and ground tools the same protocol code. See `software/rust/README.md`
- **Payload** (`software/payload/`): Jetson agent (llama.cpp) for the gated generative-beacon experiment, with the content filter
- **Ground** (`software/ground/`): Tools for beacon processing, model frames, dashboards
- **Simulation** (`software/simulation/`): Power/link notebooks and event replays

### Key Design Principles
- **Fail-graceful**: If AI payload is off or faults, revert to basic transponder + beacon
- **Power-aware**: AI bursts only when power and thermal allow (duty-cycled)
- **Open operation**: All frames in the clear (no encryption) per amateur radio regulations
- **Community model updates ("federated learning")**: not in the flight baseline and not implemented onboard; ground-side research only (`software/ground/tools/fl_update_builder.py`). Any future upload would be data only, adopted solely by authenticated telecommand from a designated station (47 CFR 97.211). Whether to keep or drop the idea is a PI decision

### RF Configuration
- Uplink: 70 cm, one IARU-coordinated frequency requested in 435.222–435.778 MHz (else 436.222–436.500 MHz; BASELINE §7)
- Downlink: 2 m, one IARU-coordinated frequency in 145.800–146.000 MHz, not 145.825 MHz
- Modes: single-channel FM transponder (duty-gated) + 1200 bps AX.25 telemetry; authenticated telecommand with hardware TX-OFF
- Power: 0.5 W nominal, 1 W maximum downlink

### Mission Phases
1. Commissioning (~2 weeks): Detumble, basic beacon, power characterization
2. Primary ops (2 years): duty-gated transponder, onboard scheduling, smart beacons (no federated-learning windows: community model updates are not in the flight baseline)
3. EOL: Permanent passivation (charge path disconnected, transmitters off); passive re-entry with solar wings deployed, no later than 5 years after mission end (no separate drag device)

## Repository Map
- `docs/`: Architecture, ML models, RF plan, regulatory notes, PDR documentation
- `hardware/`: Mechanical/electrical notes for ADCS, EPS, payload, RF, structure
- `mission/`: ConOps, V&V plan, schedule, risk register
- `software/`: Flight, payload, ground, and simulation code