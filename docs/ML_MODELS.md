# ML Models

- **Power Forecast (LSTM‑style, INT8, <100 kB)**
- **Signal Classifier (tiny CNN, <50 kB)**
- **Anomaly Detector (autoencoder, <50 kB)**
- **TinyLM for Beacons (Transformer ~0.8–1.2 MB, INT8)**

## On‑orbit Learning
- Gradient accumulation with bounded steps; sunlit only; checkpoint to FRAM.
- Federated updates: community submits small, bounded patches; sandbox validated.

## Off‑the‑Shelf LLM on Jetson Orin Nano Super — Recommended Implementation

Goal: Generate one‑sentence, first‑person status lines for 1200 bps AFSK beacons (see `docs/BEACON_SPEC.md`) using an off‑the‑shelf model, duty‑cycled on a declocked Jetson Orin Nano Super (DVFS‑limited). MCU provides the template fallback when Jetson is idle.

### Primary model
- **Llama 3.2 3B (Instruct), quantized (INT8/FP8)**
  - Rationale: strong instruction‑following and fluent, concise generations with minimal prompt engineering.
  - Deployment: TensorRT engine (batch=1, low‑latency tactics) with DVFS/declocking and process suspension between beacons.
  - Prompting: fixed system prompt to interpret CubeSat health fields and output a single, ASCII‑only line ≤120 bytes beginning `de <CALL>:` and ending `73!` unless in SAFE.

### Trade study — Joules per token (declocked Jetson, indicative)

| Model (quant) | Params | Est. tokens/s | Est. power (W) | Est. J/token | Notes |
|---|---:|---:|---:|---:|---|
| Llama 3.2 3B (INT8/FP8) | 3B | 4–8 | 7–12 | 1.0–3.0 | Highest quality; acceptable when sunlit & SOC high |
| Phi‑3 mini 3.8B (INT8) | 3.8B | 3–6 | 8–12 | 1.3–4.0 | Strong but costlier energy per token |
| Llama 3.2 1B (INT8) | 1B | 8–15 | 5–8 | 0.3–1.0 | Good balance; solid fallback under tighter power |
| TinyLlama 1.1B (INT8) | 1.1B | 10–20 | 5–8 | 0.25–0.8 | Efficient; slightly weaker phrasing |
| Qwen‑2.5 0.5B (INT8) | 0.5B | 20–40 | 4–6 | 0.1–0.3 | Very efficient; risk of terse/flat outputs |

Assumptions: declocked DVFS profile, batch=1, greedy/top‑k≤3 decoding. Values are indicative; flight values to be derived from flatsat measurements. Energy per beacon ≈ tokens_per_beacon × J/token (e.g., 25 tokens × 1.5 J/token ≈ 37.5 J).

### Selection and operating doctrine
- **Primary (sunlit, healthy margins):** Llama 3.2 3B INT8/FP8 when `SOC ≥ 60%`, `SUN=1`, `payload_temp < 55 °C`.
- **Fallback (reduced margins):** Llama 3.2 1B or TinyLlama 1.1B when `SOC 40–60%` or elevated temps.
- **MCU template:** when `SOC < 40%`, eclipse, or thermal limit; ensure beacon continuity within 120‑byte cap.

### Runtime and constraints
- Inference: TensorRT INT8/FP8, batch=1, low‑latency tactics; process suspended between beacons.
- Decoding: greedy or top‑k=3, temperature ≤0.7; enforce ASCII‑only and ≤120‑byte cap; reject/regen once; then fallback to template if still non‑compliant.
- Prompt: structured health prefix (CALL, TIME, MODE, SOC, SUN, RF, PWR, QSO) followed by a one‑line instruction; no user data beyond health fields.

### Resource budgets (target)
- Runtime memory: ≤ 128–256 MB allocated per engine profile; Jetson power‑gated between uses.
- Latency: 250–400 ms for ~24 tokens (3B); 80–200 ms for ~24 tokens (1B/1.1B) under declocked profiles.
- Power: generate only on beacon cadence; MCU covers template fallback.

### Integration with Beacon Spec
- Generator must always produce the machine‑parsable tail (key–values) as defined in `docs/BEACON_SPEC.md`.
- If generation fails or violates format/length/ASCII constraints, emit the SAFE template.

### Notes
- Numbers above are planning estimates; we will record per‑model tokens/s, Watts, and compute Joules/token on the flatsat and adopt thresholds accordingly.
