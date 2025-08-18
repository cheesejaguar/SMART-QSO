# Edge LLM Payload (Jetson Orin Nano Super + llama.cpp)

- Declocked, DVFS‑limited profiles; aggressive power‑gating of GPU/CPU clusters
- IPC with OBC via UART/SPI; watchdog supervised; optional reboot on thermal trip
- Thermal: radiator strap + thermistor feedback; duty‑cycle heavy workloads

## Data flow and constraints
- OBC sends health fields (see `software/flight/src/main.c`): `CALL, TIME, MODE, SOC, SUN, RF, PWR, QSO`
- Payload formats a short prompt and queries the LLM to produce a single sentence:
  - ASCII‑only, ≤ 120 bytes; begins with `de <CALL>:` and ends with `73!` unless `MODE=SAFE`
  - Always emits the machine‑parsable tail per `docs/BEACON_SPEC.md`
- Operations policy: enable LLM only when sunlit and margins are healthy (e.g., `SOC ≥ 60%`, payload temp < 55 °C); otherwise use template fallback

System prompt (example):
```text
You are SMART‑QSO, a 1U CubeSat. Given health fields (TIME, MODE, SOC, SUN, RF, PWR, QSO, CALL),
write one short, human‑readable status line that begins with 'de <CALL>:' and ends with '73!'
unless MODE=SAFE. Keep ≤120 ASCII bytes; no emojis; plain text.
```

## Short version: minimizing Joules/token with llama.cpp (3B model)
- Quantize weights to 4–5 bits (GGUF): start with `Q4_K_M`; for a touch more quality use `Q5_K_M`
- Full accelerator offload (CUDA/Metal/HIP/Vulkan): offload all layers to avoid CPU↔GPU transfers
- Quantize the KV‑cache: start with `q8_0` (safe), try `q4_0` for more savings after validation
- Speculative decoding: pair the 3B main model with a 1B draft (e.g., Llama‑3.2‑1B `Q2_K/Q3_K`)
- Tune batching: right‑size `--batch-size` (host) and `--ubatch-size` (device) to fit VRAM; keep `ubatch ≤ batch`
- Keep context tight and cache prompts: use smaller `--ctx-size`; reuse the system prompt via server cache/slots

## Concrete starting points

### NVIDIA (CUDA)
```bash
# build (one‑time)
cmake -B build -DLLAMA_CUBLAS=ON -DCMAKE_BUILD_TYPE=Release && cmake --build build -j

# run a server with full GPU offload, kv‑cache quantized, and healthy batching
./build/bin/llama-server \
  -m /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf \
  --gpu-layers 999 \
  --ctx-size 4096 \
  --batch-size 512 --ubatch-size 64 \
  --cache-type-k q8_0 --cache-type-v q8_0
```

Tip: fitting all weights in VRAM is best for speed/energy; pick a quant small enough to do that.

### Apple Silicon (Metal)
```bash
# build (one‑time)
cmake -B build -DLLAMA_METAL=ON -DCMAKE_BUILD_TYPE=Release && cmake --build build -j

./build/bin/llama-server \
  -m /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf \
  --gpu-layers 999 \
  --ctx-size 4096 \
  --batch-size 512 --ubatch-size 64 \
  --cache-type-k q8_0 --cache-type-v q8_0
```

### Add speculative decoding (big energy saver)
```bash
./build/bin/llama-speculative \
  -m  /models/Llama-3.2-3B-Instruct-Q4_K_M.gguf \
  -md /models/Llama-3.2-1B-Instruct-Q2_K.gguf \
  --gpu-layers 999 --draft-gpu-layers 999 \
  --ctx-size 4096 --batch-size 512 --ubatch-size 64
```

Expect 1.5–3× speedups in many cases.

### Python (llama-cpp-python) with KV‑cache quant + speculative
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

## Tuning checklist (energy first)
1. Quant choice: `Q4_K_M` → test → `Q5_K_M` if quality loss is noticeable; avoid `Q8_0` unless needed
2. All layers on GPU if possible; partial offload can waste energy through transfers
3. KV‑cache quant: start `q8_0`, then try `q4_0` with validation/speculative lookahead
4. Batch/ubatch: increase until tokens/s plateaus; keep headroom to prevent OOM
5. Speculative decoding with a small draft (≈1B) for interactive, low‑J/token operation
6. Prompt discipline: smaller contexts and reused system prompts = less KV memory/compute

## Measuring J/token quickly
- Log power while generating and divide integrated watt‑seconds by tokens emitted:
```bash
nvidia-smi --query-gpu=power.draw --format=csv -l 1 &  # log Watts
# compare to llama.cpp tokens/s; integrate over time → Joules/token
```

## Operating modes
- Primary model: **Llama‑3.2‑3B Instruct (Q4_K_M/Q5_K_M)** via llama.cpp
- Fallbacks: **Llama‑3.2‑1B** or **TinyLlama‑1.1B** under tighter power/thermal limits
- Template fallback: MCU emits a concise, compliant line whenever the payload is disabled
