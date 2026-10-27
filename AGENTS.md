# Repository Guidelines

## Project Structure & Module Organization
- `software/flight/`: CMake build, the C headers (the flight modules' interface) and the C test suites in `tests/`; the only C source is `src/flight_log_fmt.c` (the printf-style log entry points).
- `software/rust/`: the flight software (Rust): protocol and flight-logic crates, the modules behind the C headers, the host simulation, the STM32L4R5 Embassy firmware (`firmware/`) and the `sqso_protocol` Python module; see `software/rust/README.md`.
- `software/payload/`: Python agent and helpers for Jetson; tests in `tests/`, sample data in `testdata/`.
- `software/ground/`, `software/simulation/`: utilities and notebooks; see local READMEs.
- `docs/`, `hardware/`, `mission/`: design docs, trade studies, and program artifacts.

## Build, Test, and Development Commands
- Build flight (host): `cmake -S software/flight -B build && cmake --build build`; firmware: `cd software/rust/firmware && cargo build --release`.
- Run flight tests: `ctest --test-dir build`.
- Rust: `cd software/rust && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings`.
- Run payload tests: `pip install software/rust/sqso-py` (once), then `pytest software/payload -q`.
- Llama.cpp (payload integration): see `software/payload/README.md` for CUDA/Metal examples; typical server: `./build/bin/llama-server -m /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf --gpu-layers 999`. Decision D5 (`docs/payload/AI_MODEL_SELECTION.md`) puts a selector on the OBC as the flight baseline; Jetson text generation (SmolLM2-360M-Instruct or Qwen3-0.6B) is a gated experiment, and the agent defaults to SmolLM2-360M. The 3B model is a development example only.

## Coding Style & Naming Conventions
- Rust (flight): `cargo fmt` (software/rust/rustfmt.toml) and clippy with warnings as errors; no unsafe outside `sqso-ffi` and the firmware; see `docs/development/CODING_STANDARDS.md` §11.
- C (headers, tests, shim): C11, 4 spaces. Types `PascalCase` for structs, functions `snake_case`, macros `UPPER_SNAKE_CASE`.
- Python (payload): 4‑space indent, `snake_case` for functions/vars, `CapWords` for classes. Keep modules under `software/payload/`.
- Lint/typing (Python): use `ruff` and `mypy` (defaults acceptable). Example: `ruff software/payload && mypy software/payload`.
- Line length: aim ≤100 chars unless readability suffers.

## Testing Guidelines
- Frameworks: C tests via CTest; Python via `pytest`.
- Naming: Python tests as `tests/test_*.py`; add realistic cases using `testdata/` when possible.
- Coverage: prioritize logic paths (parsing, formatting, error handling). New features include tests.
- Quick example: `pytest software/payload/tests/test_integration.py -q`.

## Commit & Pull Request Guidelines
- Conventional commits: `feat:`, `fix:`, `docs:`, `chore:`, optional scopes (e.g., `feat(flight): …`).
- Commits: small, focused, imperative mood. Reference issues: `fixes #123`.
- PRs: clear description, linked issues, build/test output (`ctest`/`pytest`) and, if applicable, logs/screenshots. Update docs when behavior changes.

## Security & Configuration Tips
- Never ship secrets; all radio frames are in the clear by design.
- Payload env vars: `LLAMA_MODEL`, `LLAMA_CTX`, `LLAMA_GPU_LAYERS`, `LLAMA_CACHE_K/V`.
- Flight UART config by env (see `software/rust/sqso-ffi/src/uart_comm.rs`); validate device paths like `/dev/ttyUSB0`.
