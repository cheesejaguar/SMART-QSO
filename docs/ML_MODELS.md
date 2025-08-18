# ML Models

- **Power Forecast (LSTM‑style, INT8, <100 kB)**
- **Signal Classifier (tiny CNN, <50 kB)**
- **Anomaly Detector (autoencoder, <50 kB)**
- **TinyLM for Beacons (Transformer ~0.8–1.2 MB, INT8)**

## On‑orbit Learning
- Gradient accumulation with bounded steps; sunlit only; checkpoint to FRAM.
- Federated updates: community submits small, bounded patches; sandbox validated.

## TinyLM on Jetson Orin Nano Super — Recommended Implementation

Goal: Generate one‑sentence, first‑person status lines for 1200 bps AFSK beacons (see `docs/BEACON_SPEC.md`). Run on a declocked Jetson Orin Nano Super (DVFS‑limited, TensorRT INT8) with aggressive duty‑cycling. MCU provides fallback templates when Jetson is idle.

### Model choice
- Primary: Micro‑Transformer decoder (causal)
  - Vocabulary: 512 BPE tokens (ASCII and mission words; add special control tokens)
  - Context length: 64 tokens (prompt with telemetry control tokens + short prefix)
  - Architecture: 4 layers, d_model=128, n_heads=4, d_ff=256, rotary or learned pos‑emb
  - Params: ≈1.0–1.3 M unquantized; ≈0.8–1.2 MB INT8 weights
  - Output cap: ≤ 32 tokens (kept ≤ 120 ASCII bytes)
- Fallback: MCU template generation or micro‑RNN if Jetson is offline.

Rationales and examples: nano LMs for edge (TinyLlama, Phi‑3 variants) and an existence proof on Coral Micro via llama4micro.

### Training and distillation
1. Data
   - Curate domain corpus: thousands of beacon‑style sentences conditioned on telemetry fields (SOC, SUN, RF, MODE, forecast) and phrasing.
   - Add control tokens: `<SOC_78> <SUN_1> <RF_1> <MODE_ACTIVE> <PWR_GOOD>` prepended to the prompt.
2. Teacher → Student distillation
   - Sample teacher (desktop) responses from a small instruction model (e.g., TinyLlama or Phi‑3‑mini) using structured prompts containing the control tokens.
   - Train student micro‑Transformer on teacher pairs; KL loss on distributions + CE on tokens; enforce brevity via length penalties.
3. Quantization & pruning
   - Quantization‑aware training to INT8; structured pruning 20–30%; fine‑tune.
4. Export
   - Convert to ONNX → TensorRT INT8 engine (calibration dataset from domain prompts).

### On‑device runtime (Jetson declocked)
- Inference library: TensorRT INT8 (batch=1, low‑latency tactics); ONNX Runtime as alternative.
- Prompting: Build a short, deterministic prompt from current OBC state:
  - Prefix control tokens (discrete buckets for SOC, QSO, etc.)
  - Add a fixed textual prelude like: `de <CALL>:`
- Decoding: Greedy or top‑k=3 with temperature 0.7; hard cap 32 tokens or 120 bytes; banned‑token list; fall back to a terse template if limits exceeded or invalid byte emitted.
- Latency target: < 50–80 ms for 24 tokens at declocked settings; schedule generation just before beacon transmit window.

Minimal host‑side sketch (Python; training/export side):
```python
import tensorflow as tf

# Assume a trained Keras micro-transformer
import onnx
import tensorrt as trt

# export to ONNX then build INT8 engine with TensorRT (pseudo-code)
onnx.save(student_onnx, 'tinylm_beacon_int8.onnx')
builder = trt.Builder(trt.Logger(trt.Logger.WARNING))
network = builder.create_network(1 << int(trt.NetworkDefinitionCreationFlag.EXPLICIT_BATCH))
# ... parse ONNX, configure INT8 calibrator, set low-latency profile ...
engine = builder.build_serialized_network(network, config)
open('tinylm_beacon_int8.plan', 'wb').write(engine)
```

On‑board invocation (Jetson, Python/C++):
```c
// Setup once
engine = load_trt_engine('/flash/tinylm_beacon_int8.plan')

// Build prompt tokens from telemetry
prompt = encode_control_tokens(state)

// Decode loop (greedy)
for (int t = 0; t < MAX_STEPS && bytes_out < 120; ++t) {
  tok = trt_decode_step(engine, prompt)
  if (is_banned(tok)) break;
  append_ascii(tok, out_buf, &bytes_out);
  prompt[n++] = tok; // next‑token conditioning (windowed)
}
```

### Resource budgets (target)
- Weights (flash): ≤ 1.2 MB (INT8)
- Runtime memory: ≤ 128 MB allocated; power‑saving via declocking/DVFS and process suspension
- Compute: sized to run at declocked clocks within 50–80 ms per sentence
- Power: generate only on beacon cadence; Jetson power‑gated otherwise

### Integration with Beacon Spec
- Generator must always produce a second, machine‑parsable tail (key–values) as defined in `docs/BEACON_SPEC.md`.
- If generation fails, use the SAFE template from the spec.

### References
- Jetson runtime & TensorRT docs: NVIDIA developer resources
- Edge/embedded LLM notes (TinyLlama, Phi‑3 on edge): [knowledgeinme.com](https://knowledgeinme.com/micro-llms/)
- Edge AI quantization/pruning overview: [next.gr](https://next.gr/ai/edge-ai-iot/running-llms-on-raspberry-pi-and-microcontrollers)
- TensorFlow Lite Micro (background): [arXiv:2010.08678](https://arxiv.org/abs/2010.08678)
