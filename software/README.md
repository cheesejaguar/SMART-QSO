# Software Overview

Design values: `docs/BASELINE_PARAMETERS.md`. Software plans and requirements: `docs/software/` (development plan,
requirements, design, test plan).

- `rust/`: the OBC flight software, in Rust: every flight module, the executive, the host simulation, the mission simulation (`sqso-sil`), the STM32L4R5ZI firmware (Embassy, no RTOS; built and checked in CI, not yet run on hardware) and the `sqso_protocol` Python module used by `payload/` and `ground/`. Overview, build commands and porting table: `rust/README.md`.
- `flight/`: the C headers that are the C ABI of the Rust modules, the C test suites (CMocka) that specify them, the CMake build that drives cargo, and the one remaining C file (the flight log's printf shim). Build: `cmake -S software/flight -B build && cmake --build build && ctest --test-dir build`. Tests: `flight/tests/README.md`.
- `payload/`: the Jetson payload software for the gated generative-beacon experiment (decision D5): the OBC-Jetson link agent, the shared content filter, the llama.cpp runner and the platform configuration (`payload/README.md`).
- `ground/`: ground tools: authenticated telecommands, key provisioning, GNSS orbit determination and ephemeris, pass prediction, beacon decoding and telemetry display (`ground/tools/README.md`).
- `simulation/`: `orbit_lifetime/` (orbital lifetime cross-check, `orbit_lifetime/README.md`), `sil/report.py` (report of a `sqso-sil` run, `docs/test/SIL_MISSION_SIMULATION.md`), `cdr/` (the analysis scripts behind the 2026 design review, by discipline) and `notebooks/` (an early power-balance notebook, `notebooks/README.md`).
