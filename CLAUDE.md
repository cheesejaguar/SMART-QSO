# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SMART-QSO is a 1U CubeSat mission experimenting with onboard agentic AI to manage an amateur radio transponder. The satellite uses TinyML/TinyLM models accelerated by a Jetson Orin Nano Super (declocked, DVFS‑limited) to learn power availability, predict regional demand, prioritize QSOs fairly, and compose dynamic beacons. It fails gracefully to a conventional transponder if AI is unavailable.

## Build Commands

### Flight Software (C/CMake)
```bash
# Build the flight software skeleton
cmake -S software/flight -B build && cmake --build build

# Run tests
ctest --test-dir build

# Clean build
rm -rf build
```

## High-Level Architecture

> **Authoritative design values are in `docs/BASELINE_PARAMETERS.md`** (funding audit, 2026-10-14). Where this file or any
> other document disagrees, the baseline governs. Funding-audit findings: `docs/funding_audit/FINDINGS_REGISTER.md`.

### System Components
1. **Bus**: 1U CubeSat with deployable solar wings, UHF/VHF transponder, magnetorquer ADCS
2. **Compute**: 
   - Primary: Ultra-low-power OBC (STM32L4R5ZI) running FreeRTOS (flight software is currently a host simulation; no target port yet)
   - AI Payload: Jetson Orin Nano Super - declocked and power-gated for burst inference
3. **AI Agents** (power-aware):
   - Intelligent Power Management Agent (MCU, ~0.2W)
   - QSO Priority Agent (MCU, ~0.1W when active) - tiny CNN <50kB INT8
   - Adaptive Transponder Controller (MCU supervising RF)
   - Smart Beacon Agent: a selector on the MCU picks pre-approved templates (flight baseline, decision D5); Jetson-generated text is a gated experiment (SmolLM2-360M or Qwen3-0.6B), enabled only if measured energy per accepted beacon is ≤ 0.1 Wh. Both sides apply the same content filter (`docs/BEACON_SPEC.md` §6)

### Software Architecture
- **Flight** (`software/flight/`): OBC firmware skeleton with FreeRTOS tasks for power, QSO, and beacon management
- **Payload** (`software/payload/`): Jetson integration for TensorRT INT8 bursts
- **Ground** (`software/ground/`): Tools for beacon processing, model frames, dashboards
- **Simulation** (`software/simulation/`): Power/link notebooks and event replays

### Key Design Principles
- **Fail-graceful**: If AI payload is off or faults, revert to basic transponder + beacon
- **Power-aware**: AI bursts only when power and thermal allow (duty-cycled)
- **Open operation**: All frames in the clear (no encryption) per amateur radio regulations
- **Community model updates**: not in the flight baseline; public uploads would be data only, adopted solely by authenticated telecommand from a designated station (47 CFR 97.211)

### RF Configuration
- Uplink: 435-438 MHz (70cm)
- Downlink: 145.8-146 MHz (2m)
- Modes: single-channel FM transponder (duty-gated) + 1200 bps AX.25 telemetry; authenticated telecommand with hardware TX-OFF
- Power: 0.5 W nominal, 1 W maximum downlink

### Mission Phases
1. Commissioning (~2 weeks): Detumble, basic beacon, power characterization
2. Primary ops: Adaptive transponder control, QSO prioritization, smart beacons, federated learning windows
3. EOL: Permanent passivation (charge path disconnected, transmitters off); passive re-entry with solar wings deployed, no later than 5 years after mission end (no separate drag device)

## Repository Map
- `docs/`: Architecture, ML models, RF plan, regulatory notes, PDR documentation
- `hardware/`: Mechanical/electrical notes for ADCS, EPS, payload, RF, structure
- `mission/`: ConOps, V&V plan, schedule, risk register
- `software/`: Flight, payload, ground, and simulation code