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

### System Components
1. **Bus**: 1U CubeSat with deployable solar wings, UHF/VHF transponder, magnetorquer ADCS
2. **Compute**: 
   - Primary: Ultra-low-power OBC (Ambiq Apollo4/STM32L4) running FreeRTOS
   - AI Payload: Jetson Orin Nano Super - declocked and power-gated for burst inference
3. **AI Agents** (power-aware):
   - Intelligent Power Management Agent (MCU, ~0.2W)
   - QSO Priority Agent (MCU, ~0.1W when active) - tiny CNN <50kB INT8
   - Adaptive Transponder Controller (MCU supervising RF)
   - Smart Beacon Agent (Coral when available, MCU fallback) - TinyLM ~1MB

### Software Architecture
- **Flight** (`software/flight/`): OBC firmware skeleton with FreeRTOS tasks for power, QSO, and beacon management
- **Payload** (`software/payload/`): Jetson integration for TensorRT INT8 bursts
- **Ground** (`software/ground/`): Tools for beacon processing, model frames, dashboards
- **Simulation** (`software/simulation/`): Power/link notebooks and event replays

### Key Design Principles
- **Fail-graceful**: If AI payload is off or faults, revert to basic transponder + beacon
- **Power-aware**: AI bursts only when power and thermal allow (duty-cycled)
- **Open operation**: All frames in the clear (no encryption) per amateur radio regulations
- **Federated learning**: Community can submit bounded, rate-limited model updates via public uplink

### RF Configuration
- Uplink: 435-438 MHz (70cm)
- Downlink: 145.8-146 MHz (2m)
- Modes: Linear/FM transponder + 1200/9600 bps AX.25 telemetry
- Power: 0.5-2W selectable downlink

### Mission Phases
1. Commissioning (~2 weeks): Detumble, basic beacon, power characterization
2. Primary ops: Adaptive transponder control, QSO prioritization, smart beacons, federated learning windows
3. EOL: Deploy drag/tether device for passive deorbit within 5 years

## Repository Map
- `docs/`: Architecture, ML models, RF plan, regulatory notes, PDR documentation
- `hardware/`: Mechanical/electrical notes for ADCS, EPS, payload, RF, structure
- `mission/`: ConOps, V&V plan, schedule, risk register
- `software/`: Flight, payload, ground, and simulation code