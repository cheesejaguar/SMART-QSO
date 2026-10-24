# Software Overview

- `rust/`: the OBC flight software, in Rust: every flight module, the executive, the host simulation, the STM32L4R5ZI firmware (Embassy, no RTOS) and the `sqso_protocol` Python module used by `payload/` and `ground/`. Overview and porting table: `rust/README.md`.
- `flight/`: the C headers that are the C ABI of the Rust modules, the C test suites (CMocka) that specify them, the CMake build that drives cargo, and the one remaining C file (the flight log's printf shim).
- `payload/`: Jetson Orin Nano Super integration notes (TensorRT INT8 bursts, declocked and power‑optimized).
- `ground/`: Tools for beacons, model frames, dashboards.
- `simulation/`: Power/link notebooks and event replays.
