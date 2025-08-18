# SMART-QSO (Self‑Managing Amateur Radio Transponder with Quantum State Optimization)

An open, community‑driven **1U CubeSat** mission that experiments with onboard **agentic AI** to manage an amateur radio transponder. The satellite learns power availability and regional demand, prioritizes fair access, and composes dynamic beacons — while always **failing gracefully** to a conventional transponder if AI is unavailable.

**Key technologies**: TinyML/TinyLM, Google Coral Dev Board Micro (Edge TPU), FreeRTOS OBC, UHF/VHF transponder, federated learning via open amateur uplinks, power‑aware scheduling.

> This repository contains the public mission docs, flight/ground software stubs, and hardware interface notes. See [`docs/pdr/SMART-QSO_PDR.md`](docs/pdr/SMART-QSO_PDR.md) for the NASA‑style PDR.

## Status
- Stage: Pre‑Phase B → Phase B (PDR complete, CDR in preparation)
- License: **MIT**
- Community: Issues and PRs welcome. Please read [CONTRIBUTING](CONTRIBUTING.md) and [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md).

## Quick Start
```bash
# clone and enter
 git clone https://github.com/<your-org>/smart-qso.git
 cd smart-qso

# build (host) the flight skeleton to run unit tests
 cmake -S software/flight -B build && cmake --build build
 ctest --test-dir build
```

## Repository Structure
- `docs/` architecture, ML, RF plan, regulatory notes, and the PDR.
- `hardware/` mechanical/electrical notes and BOMs (MIT).
- `software/` flight OBC tasks (FreeRTOS), Edge‑TPU payload stubs, ground tools, simulations.
- `mission/` ConOps, V&V, schedule, and risk register.

## Governance & Safety
- Open operations under **Amateur‑Satellite Service**. All frames are **in the clear** (no encryption).
- “Federated” updates are **bounded**, rate‑limited, sandboxed, and publicly logged (see `docs/BEACON_SPEC.md`).

## Citing This Work
See [CITATION.cff](CITATION.cff).
