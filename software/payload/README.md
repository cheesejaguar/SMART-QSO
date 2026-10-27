# Jetson Payload Software (beacon text experiment)

The Jetson side of the SMART-QSO AI payload: an NVIDIA **Jetson Orin Nano 8 GB module** on a custom carrier, in the
**7 W power mode**, ≤ 8 W input including the carrier (`docs/BASELINE_PARAMETERS.md` §6; the "Super" flash
configuration is not used, TB-13). Generative beacon text on the Jetson is a **gated experiment**, not the flight
baseline: the flight baseline is the template selector on the OBC (decision D5, `docs/payload/AI_MODEL_SELECTION.md`,
`docs/payload/OBC_SELECTOR_DESIGN.md`). The experiment is enabled only if the measured energy per accepted beacon line
is ≤ 0.1 Wh (BASELINE §6). If the Jetson is off or faults, the OBC sends its own template beacons.

Software class: NPR 7150.2D Class C (`docs/software/SOFTWARE_DEVELOPMENT_PLAN.md` §3.1).

## What is here

| File | Purpose |
|------|---------|
| `agent.py` | Beacon agent: answers the OBC's `BEACON_REQ` with a `BEACON_RSP` carrying beacon text only; `TELEMETRY` updates the health used for the prompt. Runs as `python -m software.payload.agent` (stdin/stdout) or under the supervisor |
| `obc_link.py` | OBC-Jetson line protocol (`docs/interfaces/ICD_OBC_JETSON.md` §4), through `sqso_protocol` (the OBC's Rust code) |
| `crc16.py` | CRC-16/CCITT-FALSE of the link (ICD §4.4), through `sqso_protocol` |
| `content_filter.py` | The content rules of `docs/BEACON_SPEC.md` §6, the same Rust filter the OBC runs |
| `formatter.py` | Prompts and health-derived fallback text (`docs/BEACON_SPEC.md` §5) |
| `llama_runner.py` | Optional llama.cpp wrapper: grammar with no digits, temperature 0.3; without a model it returns no text |
| `health_interface.py`, `supervisor.py` | UART interface and process lifecycle (status, shutdown) |
| `platform_config.py`, `config/` | Jetson platform settings at boot (6.5 W VDD_IN cap, 7 W nvpmodel mode, PCIe C4 overlay; SQSO-JET-BRD-001) |
| `ansible/deploy_payload.yml` | Deployment to a Jetson (venv, `sqso_protocol`, llama-cpp-python, `run_payload.sh`, `sqso-platform.service`) |
| `tests/`, `testdata/` | pytest suites (run in CI by `.github/workflows/program-checks.yml`) |

## Behaviour

- The OBC sends `TELEMETRY` and `BEACON_REQ` lines (ICD §4). The Jetson returns text only; the OBC adds the call sign
  `KK6GDB-11` and the measured telemetry, applies the content rules again and falls back to a template if the text
  fails (`docs/BEACON_SPEC.md` §3, §5, §6). Beacon text is at most 176 characters and carries no numbers.
- Without a model, or when the model's output fails the rules, the agent sends health in words ("Safe mode, conserving
  power." and similar), never a canned status.
- The OBC, not the Jetson, decides when the Jetson is powered: sunlit, SOC ≥ 55 %, Jetson temperature ≥ −15 °C at
  power-on, each power-on ≤ 10 minutes including boot, then ≥ 100 minutes off, with per-rail current and temperature
  limits (BASELINE §6, AI burst).
- Default model: `/models/SmolLM2-360M-Instruct-Q4_K_M.gguf`, a D5 candidate (Qwen3-0.6B is the other); override with
  `LLAMA_MODEL`. Other settings: `LLAMA_CTX` (1024), `LLAMA_GPU_LAYERS` (0), `LLAMA_CACHE_K`, `LLAMA_CACHE_V`.

## Build and test

`sqso_protocol` is built from the Rust workspace (maturin) and is not on PyPI, so install it first. From the
repository root:

```bash
pip install ./software/rust/sqso-py
pip install -e software/payload          # add [llm] for llama-cpp-python, [dev] for the dev tools
python -m pytest -q software/payload/tests
```

Run the agent on a line stream (for example the recorded OBC lines):

```bash
python -m software.payload.agent < software/payload/testdata/obc_lines.txt
```

## Measuring energy per beacon (D5 gate)

The gate is the energy per *accepted* beacon line at the carrier input, boot to power-off, over ≥ 30 EM bursts
(VI-02; BASELINE §6), with an external meter in series with the power-gated rail; `tegrastats` misses the bootloader,
the carrier regulators and the switch losses and serves only as a cross-check. The procedure is
`docs/payload/AI_MODEL_SELECTION.md` §5.
