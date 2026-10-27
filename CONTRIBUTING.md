# Contributing to SMART-QSO

Thanks for helping build an open amateur-radio satellite! Please:

1. **Discuss first** via an Issue.
2. **Fork** → create a branch (`feature/<topic>` or `fix/<topic>`).
3. **Conventional Commits** (e.g., `feat(power): add MPPT telemetry parser`).
4. **Tests**: add/keep unit tests passing (`ctest`, and `cargo test --workspace` in `software/rust`).
5. **PR**: link issues; describe motivation, design, and risks.

## Development Environment
- Toolchain: Rust for the flight software, host and target (rustup; the version is pinned in `software/rust/rust-toolchain.toml`); CMake + GCC/Clang for the host build and the C test suites. No ARM C compiler is needed: the STM32L4R5ZI image is built by cargo (`software/rust/firmware`).
- OBC: no RTOS; the firmware runs on the Embassy executor (`software/flight/docs/RTOS_ARCHITECTURE.md`).
- Payload (Jetson Orin Nano 8 GB module, 7 W mode; `docs/BASELINE_PARAMETERS.md` §6): Python agent with llama.cpp for the gated generative-beacon experiment (SmolLM2-360M-Instruct or Qwen3-0.6B, INT4); the flight baseline is the template selector on the OBC (decision D5). Tests: `pip install software/rust/sqso-py && pip install -e software/payload`, then `python -m pytest -q software/payload/tests` (`software/payload/README.md`).

## Style
- Rust: `cargo fmt` and clippy with warnings as errors; coding standard in `docs/development/CODING_STANDARDS.md` §11.
- C/C++: clang‑format (LLVM style).
- Markdown: markdownlint.

## DCO
Sign‑off each commit (`git commit -s`).
