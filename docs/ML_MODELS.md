# ML Models

> **Superseded for model selection and energy by decision D5** (`docs/payload/AI_MODEL_SELECTION.md`, 2026-10-15). This
> page estimates energy per beacon as tokens × J/token with no boot, load or shutdown term; a cold-started Jetson spends
> 87–96 % of its energy on those, about 230 J per beacon rather than 5–38 J. The 3B model below exceeds the 8 W input
> budget. The flight baseline is a selector on the OBC; Jetson generation is a gated experiment.
>
> **Documentation audit (2026-10-27): historical planning note.** Below the federated-learning section, nothing on this
> page is current; it is kept only for its early trade table. Current values: the Jetson is an Orin Nano 8 GB module in
> the 7 W mode, not "Super" (BASELINE §6, TB-13); the Jetson candidates are SmolLM2-360M-Instruct or Qwen3-0.6B, INT4,
> run with llama.cpp (`docs/payload/AI_MODEL_SELECTION.md`); generated text carries no numbers and no call sign, and
> the OBC writes the call sign and every value from measurements (`docs/BEACON_SPEC.md` §3, §6); the gate is ≤ 0.1 Wh
> per accepted beacon line, measured on the EM (BASELINE §6). The models in the first list (power forecast, signal
> classifier, anomaly detector, TinyLM) are not in the flight baseline; the OBC selector is specified in
> `docs/payload/OBC_SELECTOR_DESIGN.md`.

- **Power Forecast (LSTM‑style, INT8, <100 kB)**
- **Signal Classifier (tiny CNN, <50 kB)**
- **Anomaly Detector (autoencoder, <50 kB)**
- **TinyLM for Beacons (Transformer ~0.8–1.2 MB, INT8)**

## On‑orbit Learning (not in the flight baseline)
- On-orbit training (gradient accumulation with bounded steps, sunlit only, checkpoint to FRAM) is **not in the flight
  baseline** and has no onboard implementation. The flight baseline is a calibrated selector on the OBC (decision D5).
- Community model updates ("federated learning"): **not in the flight baseline**; ground-side research only
  (`software/ground/tools/fl_update_builder.py`). There is no onboard receiver for these frames, and the frames carry
  only a CRC-16, no authentication. Any future upload would be data only, adopted solely by authenticated telecommand
  from a designated station (47 CFR 97.211), with no path to power, RF or mode control. Whether to keep or drop the idea
  is a PI decision (`docs/BASELINE_PARAMETERS.md` §2; finding SW-11).

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
