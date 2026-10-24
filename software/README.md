# Software Overview

- `flight/`: OBC firmware skeleton in C (FreeRTOS tasks for power, QSO, beacon are planned, not implemented).
- `rust/`: the uplink, telecommand, Jetson link, content filter, mode manager, scheduler and fault log in Rust, linked into `flight/`, and the `sqso_protocol` Python module used by `payload/` and `ground/`.
- `payload/`: Jetson Orin Nano Super integration notes (TensorRT INT8 bursts, declocked and power‑optimized).
- `ground/`: Tools for beacons, model frames, dashboards.
- `simulation/`: Power/link notebooks and event replays.
