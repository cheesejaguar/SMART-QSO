# SMART-QSO (Self‑Managing Amateur Radio Transponder with Quantum State Optimization)

An open, community‑driven **1U CubeSat** mission that experiments with onboard **agentic AI** to manage an amateur radio transponder. The satellite learns power availability and regional demand, prioritizes fair access, and composes dynamic beacons — while always **failing gracefully** to a conventional transponder if AI is unavailable.

**Key technologies**: TinyML/TinyLM, Jetson Orin Nano Super (declocked, DVFS‑limited), FreeRTOS OBC, UHF/VHF transponder, federated learning via open amateur uplinks, power‑aware scheduling.

> This repository contains the public mission docs, flight/ground software stubs, and hardware interface notes. See [`docs/pdr/SMART-QSO_PDR.md`](docs/pdr/SMART-QSO_PDR.md) for the NASA‑style PDR.

## Status
- Stage: Pre‑Phase B → Phase B (PDR complete, CDR in preparation)
- License: **MIT**
- Community: Issues and PRs welcome. Please read [CONTRIBUTING](CONTRIBUTING.md) and [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md).

## Quick Start
```bash
# clone and enter
 git clone https://github.com/cheesejaguar/SMART-QSO.git
 cd smart-qso

# build (host) the flight skeleton to run unit tests
 cmake -S software/flight -B build && cmake --build build
 ctest --test-dir build
```

## Repository Structure
- `docs/` architecture, ML, RF plan, regulatory notes, and the PDR.
- `hardware/` mechanical/electrical notes and BOMs (MIT).
- `software/` flight OBC tasks (FreeRTOS), Jetson payload stubs, ground tools, simulations.
- `mission/` ConOps, V&V, schedule, and risk register.

## Edge LLM for Health Analysis (llama.cpp, 3B primary)
- Purpose: The payload analyzes OBC health fields (as produced by `software/flight/src/main.c` and `sensors.yaml`) to generate a short, first‑person status line for the beacon. We use an off‑the‑shelf Llama‑3.2‑3B model via llama.cpp on a declocked Jetson, with strict ASCII/≤120‑byte output and a template fallback.
- Lowest joules/token typically comes from:
  1) 4–5‑bit weight quantization that preserves quality (GGUF `Q4_K_M` → `Q5_K_M` if needed)
  2) Full accelerator offload (CUDA/Metal/HIP) to avoid CPU↔GPU transfers
  3) Quantized KV‑cache (`q8_0` safe; `q4_0` for more savings after quality checks)
  4) Speculative decoding with a 1B draft
  5) Tight runtime settings (batch/ubatch/threads/context) + prompt caching

### Practical recipe
- Quantization: `Q4_K_M` (primary). Consider `Q5_K_M` if quality needs a small bump.
- Run end‑to‑end on the accelerator (all layers on GPU where VRAM allows).
- KV‑cache quant: start `q8_0`; test `q4_0` once stable.
- Speculative: 3B main + 1B draft (e.g., `Llama‑3.2‑1B Q2_K/Q3_K`).

NVIDIA (CUDA)
```bash
# build (one‑time)
cmake -B build -DLLAMA_CUBLAS=ON -DCMAKE_BUILD_TYPE=Release && cmake --build build -j

# run server (full GPU offload, kv‑cache quantized, healthy batching)
./build/bin/llama-server \
  -m /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf \
  --gpu-layers 999 \
  --ctx-size 4096 \
  --batch-size 512 --ubatch-size 64 \
  --cache-type-k q8_0 --cache-type-v q8_0
```

Apple Silicon (Metal)
```bash
cmake -B build -DLLAMA_METAL=ON -DCMAKE_BUILD_TYPE=Release && cmake --build build -j

./build/bin/llama-server \
  -m /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf \
  --gpu-layers 999 \
  --ctx-size 4096 \
  --batch-size 512 --ubatch-size 64 \
  --cache-type-k q8_0 --cache-type-v q8_0
```

Speculative decoding (big energy saver)
```bash
./build/bin/llama-speculative \
  -m  /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf \
  -md /models/Llama-3.2-1B-Instruct-Q2_K.gguf \
  --gpu-layers 999 --draft-gpu-layers 999 \
  --ctx-size 4096 --batch-size 512 --ubatch-size 64
```

Python (llama-cpp-python) with KV‑cache quant + speculative
```python
from llama_cpp import Llama
from llama_cpp.llama_speculative import LlamaPromptLookupDecoding

ll = Llama(
    model_path="Llama-3.2-3B-Instruct-Q4_K_M.gguf",
    n_ctx=4096, n_gpu_layers=99,
    type_k="q8_0", type_v="q8_0",  # KV cache quantization
)
draft = LlamaPromptLookupDecoding(ll)  # or a 1B draft model
out = ll.create_completion("CALL=SMARTQ-1 TIME=2025-08-18Z MODE=ACTIVE SOC=78 SUN=1 RF=1 PWR=GOOD QSO=12",
                           draft_model=draft)
```

Tuning checklist (energy first)
- Quant choice: `Q4_K_M` → test → `Q5_K_M` if quality dips; avoid `Q8_0` unless needed
- Offload all layers to GPU if possible (minimizes transfers)
- KV‑cache quant: start `q8_0`, then evaluate `q4_0`
- Batch/ubatch: increase until tokens/s plateaus, keep headroom to avoid OOM
- Speculative decoding with a tiny draft (1B)
- Prompt discipline: smaller contexts + cached system prompts

Measuring J/token quickly
- Log power while generating (e.g., `nvidia-smi --query-gpu=power.draw --format=csv -l 1`) and divide integrated watt‑seconds by tokens emitted (llama.cpp prints tokens/s). Tune batch/ubatch/cache types to minimize J/token.

## Governance & Safety
- Open operations under **Amateur‑Satellite Service**. All frames are **in the clear** (no encryption).
- “Federated” updates are **bounded**, rate‑limited, sandboxed, and publicly logged (see `docs/BEACON_SPEC.md`).

## Citing This Work
See [CITATION.cff](CITATION.cff).
