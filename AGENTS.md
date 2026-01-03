# Repository Guidelines

## Project Structure & Module Organization
- `software/flight/`: C11 flight skeleton built with CMake; sources in `src/`, tests in `tests/`.
- `software/payload/`: Python agent and helpers for Jetson; tests in `tests/`, sample data in `testdata/`.
- `software/ground/`, `software/simulation/`: utilities and notebooks; see local READMEs.
- `docs/`, `hardware/`, `mission/`: design docs, trade studies, and program artifacts.

## Build, Test, and Development Commands
- Build flight (host): `cmake -S software/flight -B build && cmake --build build`.
- Run flight tests: `ctest --test-dir build`.
- Run payload tests: `pytest software/payload -q`.
- Llama.cpp (payload integration): see `README.md` for CUDA/Metal examples; typical server: `./build/bin/llama-server -m /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf --gpu-layers 999`.

## Coding Style & Naming Conventions
- C (flight): C11, 2 or 4 spaces consistently; files in `software/flight/src`. Types `PascalCase` for structs, functions `snake_case`, macros `UPPER_SNAKE_CASE`.
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
- Flight UART config by env (see `software/flight/src/main.c`); validate device paths like `/dev/ttyUSB0`.
