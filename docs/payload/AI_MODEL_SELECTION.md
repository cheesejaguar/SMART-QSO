# Onboard AI Model Selection (Decision D5)

**Document Number:** SQSO-AUD-006
**Revision:** A
**Date:** 2026-10-15
**Decision owner:** Payload lead (baseline: `docs/BASELINE_PARAMETERS.md` §6, §9)
**Research:** [D5_AI_MODEL_SELECTION.md](../funding_audit/research/D5_AI_MODEL_SELECTION.md). Numbers marked [ROM] there are
estimates, not measurements.

## 1. Decision

1. **Flight baseline: a selector on the OBC.** A small calibrated model runs on the STM32L4R5ZI: gradient-boosted
   trees or a tiny INT8 MLP. It picks:
   - one of a fixed set of pre-approved beacon templates;
   - a proposed power mode;
   - transponder on/off scheduling;
   - a QSO priority score.

   Below a confidence threshold, or with inputs out of range, it abstains to fixed rules. Hard limits (SOC floor, duty
   caps, temperature, TX-OFF and the other inhibits) stay outside the model. The Jetson is never woken for these
   decisions.
2. **Gated experiment: generative beacon text on the Jetson.**
   - Candidates are SmolLM2-360M-Instruct or Qwen3-0.6B with thinking disabled, both INT4 and Apache 2.0.
   - The model generates non-numeric lines in batches of 10 or more per power-on.
   - It is enabled only when the measured energy per accepted beacon passes the D5 limit (§5) and a ground sweep of the
     content filter passes.
   - The choice between the two is made on the measured filter-reject rate, license and toolchain. Energy does not
     separate them.
3. **Jev (TypeSafe AI) is ground-side only.** It is proprietary and API-only, so it cannot fly. It may label synthetic
   scenarios for the selector's tone and fairness choices, after TypeSafe confirms in writing that its outputs may train
   a shipped model. Power and transponder labels come from the orbit power simulation, not from Jev.

The design borrows Jev's pattern, not its model: a closed, typed output set, a probability distribution and an abstain
path. Onboard, that is a conventional supervised classifier with calibration (temperature scaling, reliability
diagrams) and abstention.

## 2. Why: Energy Is Set by Booting the Jetson, Not by the Model

The D5 criterion is ≤ 0.1 Wh (360 J) per beacon. For a cold-started Jetson, booting, loading the model and shutting
down account for 87–96 % of the energy [ROM]:

| Path | Energy per beacon [ROM] |
|---|---|
| OBC selector (GBDT, about 1 ms at 45 mW) | ≈ 45 µJ (tiny INT8 MLP 36–75 µJ; BERT-tiny-class 5–10 mJ) |
| Jetson, any model of 1.2B parameters or fewer, stock boot (≈ 40 s) | 227–247 J |
| Same, with NVIDIA's documented boot optimization (≈ 16 s) | 111–131 J |
| Jetson, 2B–4B model, stock boot | 303–314 J central; ≈ 435 J pessimistic (fails D5) |
| Jetson classifier instead of a generator | ≈ 220–230 J: no advantage, the boot dominates |

At one beacon per 60 s, booting the Jetson per beacon would need about 21 kJ per orbit against an AI budget of
2.2–4.5 kJ. Only batching works: five bursts of 20 candidates take about 1.6 kJ per orbit at about 5.4 % duty.
That sits at the 5 % tumbling cap and inside the 10 % sun-pointed cap. Reusing candidates across beacons brings it
under.

`docs/ML_MODELS.md` estimates energy as tokens × J/token with no boot term (about 5–38 J per beacon) and names a 3B model.
Both are superseded by this document.

## 3. Decision Matrix

Weights: energy 30, 97.113 content safety 20, failure modes 14, quality 12, toolchain 10, memory 8, license 6.
Scores run 1–5, with 5 best; rubric in the research §7.

| Option | Weighted score |
|---|---|
| A. OBC selector | **4.66** |
| G. Hybrid: A plus gated Jetson generation (adopted) | **4.06** |
| B. Jetson classifier | 3.34 |
| C. Jetson SLM ≤ 0.4B | 2.82 |
| E. Jetson SLM 1–1.2B | 2.76 |
| D. Jetson SLM 0.5–0.8B | 2.68 |
| F. Jetson SLM 2–4B | 2.28 |

The ranking holds when mission appeal is weighted at 30. A scores higher than G, but G is adopted because the mission is
an AI experiment, and G keeps A's safety properties.

## 4. Content Controls (Implemented)

The rule that separates the options is 97.113(a)(4), "false or deceptive messages": a model that writes a number can
state a wrong one. So no generated text carries a number; the OBC writes every value from measurements.

The OBC and the Jetson (`content_filter.py`) apply the same deterministic rules, one implementation (`software/rust/sqso-protocol/src/filter.rs`)
(`docs/BEACON_SPEC.md` §6) and run the same shared test vectors. Generation uses temperature 0.3 and a grammar that
admits no digits. Without a model the runner returns no text, so the beacon falls back to health-derived text or a
template. Before enabling generation, run at least 10,000 ground generations at flight settings and report the reject
rate by rule.

## 5. Measurement Procedure

- **Instrument:** an external meter (Joulescope or SMU class, ≥ 10 kS/s) in series with the power-gated rail on the
  bus side of the load switch, powered from the flight converter. `tegrastats` misses bootloader time, carrier
  regulators and switch losses, so it serves only as a cross-check.
- **Phase markers** (OBC GPIO/UART): gate on, kernel up, model loaded, first token, output committed, gate off.
  Integrate energy per phase. Measure gate-off leakage separately.
- **Test matrix:**
  - Power mode 7 W, and 15 W if the 7 W mode is unavailable (§6).
  - Stock and boot-optimized images on the flight storage medium.
  - llama.cpp and MLC runtimes.
  - SmolLM2-360M, Qwen3-0.6B, one 1B reference, and the 3B model as a negative control.
  - At least 50 fixed prompts with at most 60 new tokens.
  - Batches of N = 1, 5, 10 and 20.
  - Ambient and hot soak.
  - At least 10 cold starts per configuration (30 for a stable p95).
- **Pass:** p95 energy per **accepted** beacon ≤ 360 J, where accepted beacons are those passing the OBC filter and
  energy runs from gate on to gate off. Also burst energy ≤ 960 J (8 W × 120 s), and orbit-average Jetson energy
  within budget at the planned burst count. Otherwise the experiment stays off and A flies alone.
- **Selector:** ECE and reliability diagram on held-out and out-of-distribution sets, abstention rate, and decision
  energy on the OBC rail.

## 6. Open Items

1. **7 W mode versus the "Super" flash configuration** (FINDINGS_REGISTER TB-13). NVIDIA lists 7 W and 15 W only for the
   original configuration, and 15 W, 25 W and MAXN_SUPER for Super. The baseline specifies 7 W. Without a module-enforced
   cap near 8 W, the OBC's per-rail current limit is the only protection against a burst exceeding the carrier budget.
2. Storage medium and load path on the custom carrier.
3. Suspend instead of power-gating: the only way to avoid boot energy. Measure suspend power and resume time.
4. A regulatory read, by FCC counsel or an ARRL coordinator, of autonomous text generation under the control
   operator's responsibility, before enabling generation (D3 consultation).
5. TypeSafe's Master Customer Agreement and early-access terms before any Jev use.
6. The selector needs the OBC target port (FINDINGS_REGISTER SW-01). It can be validated in the host simulation first.

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| A | 2026-10-15 | Decision D5 recorded from the research report |
