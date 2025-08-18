# ML Models

- **Power Forecast (LSTM‑style, INT8, <100 kB)**
- **Signal Classifier (tiny CNN, <50 kB)**
- **Anomaly Detector (autoencoder, <50 kB)**
- **TinyLM for Beacons (Transformer ~0.8–1.2 MB, INT8)**

## On‑orbit Learning
- Gradient accumulation with bounded steps; sunlit only; checkpoint to FRAM.
- Federated updates: community submits small, bounded patches; sandbox validated.

## TinyLM on Coral Dev Board Micro — Recommended Implementation

Goal: Generate one‑sentence, first‑person status lines for 1200 bps AFSK beacons (see `docs/BEACON_SPEC.md`). Run fully on the Coral Dev Board Micro (Cortex‑M7 ~800 MHz, ~64 MB RAM) with optional Edge TPU acceleration when ops are supported. Reference prior art shows feasibility of tiny LMs on this board (e.g., llama4micro).

### Model choice
- Primary: Micro‑Transformer decoder (causal)
  - Vocabulary: 512 BPE tokens (ASCII and mission words; add special control tokens)
  - Context length: 64 tokens (prompt with telemetry control tokens + short prefix)
  - Architecture: 4 layers, d_model=128, n_heads=4, d_ff=256, rotary or learned pos‑emb
  - Params: ≈1.0–1.3 M unquantized; ≈0.8–1.2 MB INT8 weights
  - Output cap: ≤ 32 tokens (kept ≤ 120 ASCII bytes)
- Fallback: Char‑RNN/GRU (1–2 layers, 256 units) if transformer ops are unavailable; smaller and simpler to port on TF Lite Micro.

Rationales and examples: nano LMs for edge (TinyLlama, Phi‑3 variants) and an existence proof on Coral Micro via llama4micro.

### Training and distillation
1. Data
   - Curate domain corpus: thousands of beacon‑style sentences conditioned on telemetry fields (SOC, SUN, RF, MODE, forecast) and phrasing.
   - Add control tokens: `<SOC_78> <SUN_1> <RF_1> <MODE_ACTIVE> <PWR_GOOD>` prepended to the prompt.
2. Teacher → Student distillation
   - Sample teacher (desktop) responses from a small instruction model (e.g., TinyLlama or Phi‑3‑mini) using structured prompts containing the control tokens.
   - Train student micro‑Transformer on teacher pairs; KL loss on distributions + CE on tokens; enforce brevity via length penalties.
3. Quantization & pruning
   - Quantization‑aware training to INT8 (per‑channel for matmuls). Optionally prune 20–30% heads/neurons and fine‑tune.
4. Export
   - Convert to TFLite (INT8). Try compiling with Edge TPU compiler; fall back to CPU if ops aren’t supported.

### On‑device runtime (Cortex‑M7, optional Edge TPU)
- Inference library: TensorFlow Lite Micro for MCU portability; use Edge TPU delegate only if the compiled model maps supported ops.
- Prompting: Build a short, deterministic prompt from current OBC state:
  - Prefix control tokens (discrete buckets for SOC, QSO, etc.)
  - Add a fixed textual prelude like: `de <CALL>:`
- Decoding: Greedy or top‑k=3 with temperature 0.7; hard cap 32 tokens or 120 bytes; banned‑token list; fall back to a terse template if limits exceeded or invalid byte emitted.
- Latency target: < 100 ms for 24 tokens on M7; schedule generation immediately prior to beacon transmit window.

Minimal host‑side sketch (Python; training/export side):
```python
import tensorflow as tf

# Assume a trained Keras micro-transformer
converter = tf.lite.TFLiteConverter.from_keras_model(student_model)
converter.optimizations = [tf.lite.Optimize.DEFAULT]
converter.target_spec.supported_ops = [tf.lite.OpsSet.TFLITE_BUILTINS_INT8]
converter.inference_input_type = tf.uint8
converter.inference_output_type = tf.uint8
tflite_model = converter.convert()
open('tinylm_beacon_int8.tflite', 'wb').write(tflite_model)
```

On‑board invocation (C/TFLM pseudocode):
```c
// Setup once
tflite_interpreter_t* it = tflm_create("/flash/tinylm_beacon_int8.tflite");

// Build prompt tokens from telemetry
uint8_t prompt[64]; size_t n = encode_control_tokens(prompt, state);

// Decode loop (greedy)
for (int t = 0; t < MAX_STEPS && bytes_out < 120; ++t) {
  tflm_set_input(it, prompt, n);
  tflm_invoke(it);
  uint8_t tok = tflm_get_next_token(it);
  if (is_banned(tok)) break;
  append_ascii(tok, out_buf, &bytes_out);
  prompt[n++] = tok; // next‑token conditioning (windowed)
}
```

### Resource budgets (target)
- Weights (flash): ≤ 1.2 MB (INT8)
- Runtime arena (RAM): ≤ 2 MB TFLM arena for 64‑token context
- Compute: < 5 GOPS per sentence; acceptable on 800 MHz M7 at low duty cycle
- Power: generate only on beacon cadence; sleep otherwise

### Integration with Beacon Spec
- Generator must always produce a second, machine‑parsable tail (key–values) as defined in `docs/BEACON_SPEC.md`.
- If generation fails, use the SAFE template from the spec.

### References
- Coral Dev Board Micro nano‑LLM demo: llama4micro on Coral Micro — [github.com/maxbbraun/llama4micro](https://github.com/maxbbraun/llama4micro)
- Edge/embedded LLM notes (TinyLlama, Phi‑3 on edge): [knowledgeinme.com](https://knowledgeinme.com/micro-llms/)
- Edge AI quantization/pruning overview: [next.gr](https://next.gr/ai/edge-ai-iot/running-llms-on-raspberry-pi-and-microcontrollers)
- TensorFlow Lite Micro (background): [arXiv:2010.08678](https://arxiv.org/abs/2010.08678)
