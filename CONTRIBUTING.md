# Contributing to SMART-QSO

Thanks for helping build an open, community‑taught satellite! Please:

1. **Discuss first** via an Issue.
2. **Fork** → create a branch (`feature/<topic>` or `fix/<topic>`).
3. **Conventional Commits** (e.g., `feat(power): add MPPT telemetry parser`).
4. **Tests**: add/keep unit tests passing (`ctest`).
5. **PR**: link issues; describe motivation, design, and risks.

## Development Environment
- Toolchain: CMake + GCC/Clang for host; ARM GCC for target.
- OBC RTOS: FreeRTOS (portable stubs in this repo).
- Payload (Coral Micro): Edge TPU models run via TFLite‑int8; code stubs only.

## Style
- C/C++: clang‑format (LLVM style).
- Markdown: markdownlint.

## DCO
Sign‑off each commit (`git commit -s`).
