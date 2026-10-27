# D5 research: onboard AI model selection by energy per beacon

> **Historical record (2026-10-15).** Superseded for current values by `docs/BASELINE_PARAMETERS.md` and
> `docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md`. Still good for: the model survey and energy reasoning behind decision D5.

> Research input to decision D5, prepared 2026-10-15 and imported unchanged into the repository. Where it mentions working files (raw downloads, extracted texts, scripts) or says no repository files were modified, it refers to the research session; those files were not kept. How the decision was taken and applied is recorded in `docs/BASELINE_PARAMETERS.md` section 9.


Project: SMART-QSO (1U amateur CubeSat). Date of research: 2026-10-15. Author: technical research analyst (Claude).
Status: research input to decision D5 (owner: payload lead). No repository files were modified.

Labels used throughout:
- **[SRC]** = taken from a cited source (section 11 gives dates and URLs).
- **[ROM]** = rough-order-of-magnitude estimate by me; arithmetic shown; NOT a measurement.
- **[JUDGMENT]** = my opinion or scoring, not a fact.
- **[UNVERIFIED]** = stated from general knowledge or a single weak source; not confirmed in this session.

---

## 1. Executive summary

**Recommendation (a hybrid, with the discriminative selector as the flight baseline):**

1. **Flight baseline = an MCU-resident, "Jev-style" discriminative selector.** A small calibrated classifier (gradient-boosted trees or a tiny INT8 MLP) running on the STM32L4R5ZI OBC picks (a) one of a fixed set of pre-approved, human-reviewed beacon templates, (b) the power-management mode, (c) transponder on/off scheduling and (d) a QSO priority score. Flight software fills all telemetry slots. It never wakes the Jetson. [ROM] energy per decision is about 3 µJ to 10 mJ, versus [ROM] about 110 to 330 J for any path that boots the Jetson. That is 4 to 7 orders of magnitude, and it passes the D5 criterion (0.1 Wh = 360 J) trivially.
2. **Jetson generative model = gated experiment, not the beacon path.** Candidate: **SmolLM2-360M-Instruct or Qwen3-0.6B (non-thinking mode), INT4, Apache 2.0**, producing only non-numeric "flavor" lines in batches. It is enabled only after (i) measured energy at the carrier input is at or below 0.1 Wh per accepted beacon, and (ii) ground validation of the output filter. The choice among sub-1.2B models is within noise on energy (section 5) and should be made on license, toolchain and measured filter-reject rate [JUDGMENT].
3. **Jev itself is ground-side only.** It cannot fly (proprietary, weights unpublished). It may be useful on the ground for labeling synthetic training scenarios and decision support, but its terms are unresolved for that use (section 4).

**The finding that matters most [ROM, high confidence in the structure, low in the constants]:** on the Jetson path, boot, model load and shutdown are about 87 to 96 percent of the energy of a cold-started beacon. Choosing between a 135M and a 1.2B model moves the total by under about 10 percent; boot optimisation (36 to 43 s stock versus 16 s documented-optimised versus under 5 s vendor-claimed) moves it by 2 to 3 times. The model-ranking question in D5 is therefore second-order. The first-order questions are boot time, whether the Jetson wakes per beacon at all, and whether the 7 W mode is actually available (section 9, open item 1).

**Two further structural facts:**
- At a 60 s beacon cadence (about 92 beacons per orbit), one Jetson cold start per beacon would use about 92 x 230 J = 21 kJ per orbit [ROM], versus an AI-payload orbit budget of about 2.2 to 4.5 kJ (0.4 to 0.8 W average from `BASELINE_PARAMETERS.md` lines 96-98, x about 5,600 s). That is 5 to 10 times over budget. Per-beacon Jetson generation is infeasible at that cadence; only batching (N of about 10 or more candidates per burst) or an MCU-only path works.
- Batching closes the gap [ROM]: about 5 bursts per orbit of 20 candidates each is about 5 x (112 J + 20 x 10.7 J) = 1.6 kJ per orbit, inside the 2.2 kJ tumbling budget, at about 5.4 percent duty (edge of the 5 percent tumbling cap; inside the 10 percent sun-pointed cap). Details in section 5.5.
- The RF energy to transmit one beacon is about 0.10 W x 60 s = 6 J (baseline line 96), so even the cheapest Jetson beacon (N=1, about 110 J) costs about 19 times the transmission itself.
- **The repo's existing energy model omits these overheads.** `docs/ML_MODELS.md` estimates energy per beacon as tokens x J/token (for example 25 tokens x 1.5 J/token = 37.5 J for the 3B model). My cold-start estimate for a 3B-class model is about 300 J or more, and for a 0.5B model about 230 J versus about 5 J by the J/token method. See section 10.0.

---

## 2. Baseline context (read from the repo; read-only)

From `docs/BASELINE_PARAMETERS.md` (audit 2026-10-14):
- Line 114: AI model "selected by measured energy ≤ 0.1 Wh per generated beacon [D5]". The 3B model in `software/payload/agent.py` (about 2 GB) and the "about 1 MB TinyLM" in older docs are both unconfirmed.
- Line 112: Jetson Orin Nano 8 GB module (P/N 900-13767-0030-000) on a custom carrier, **7 W power mode**, **≤ 8 W input including carrier**, no fan.
- Line 113: AI burst ≤ 120 s including boot; sunlit only; SOC ≥ 55 percent; OBC-enforced current and temperature limits.
- Lines 96-98: beacon about 0.10 W average (about 2 s per 60 s); Jetson 0.40 W (tumbling, ≤5 percent duty) to 0.80 W (sun-pointed, ≤10 percent).
- Line 153: D5 owner is the payload lead; due before the EM thermal test.

Derived numbers [ROM, arithmetic]:
- D5 threshold: 0.1 Wh x 3,600 = **360 J**.
- Burst envelope: 120 s x 8 W = 960 J = 0.267 Wh, which is only about 2.7 times the D5 threshold; there is little slack for a slow boot.
- Orbit AI budget: 0.4 W x 5,600 s = 2,240 J; 0.8 W x 5,600 s = 4,480 J (orbit period of about 93 min assumed).
- Beacon text: 179 characters is about 45 to 60 tokens at roughly 3 to 4 characters per token [ROM].

---

## 3. Technical overview

### 3.1 What a "Jev-style" architecture means here

Jev (TypeSafe AI, early access since 2026-09-15) is a **discriminative** model. It takes a "state" (string, JSON, or array) plus typed questions: `choice` (up to 255 options), `score` (2 to 10 ordered levels), `noul` (true/false). It returns probability distributions, not text. Questions in one call are evaluated independently and in parallel. [SRC: Wikipedia; TypeSafe docs; noze.it]

The pattern worth borrowing is architectural, not the model:
1. The output space is a **closed, typed set** (template ID, mode enum, boolean, ordinal score). There is no free-form string, so content risk is bounded by what is in the set.
2. The output is a **probability distribution** with an explicit abstain path (if top-1 probability is below a threshold, fall back to a deterministic rule).
3. Calibration is a first-class requirement.

Onboard we do not need Jev's zero-shot flexibility (categories supplied at inference time, natural-language state). The categories are fixed and the state is structured telemetry. So a plain supervised model suffices. [JUDGMENT: "Jev-style" onboard is a conventional small supervised classifier with calibration and abstention. That is a sound design; it is not novel ML.]

Caveat for anyone copying Jev's API shape: Jev's `confidence` field is a statistic of the distribution's shape, **not** the probability that the answer is correct (TypeSafe docs, via noze.it). Onboard, use `max(p)` after temperature scaling plus a held-out reliability check, not a shape statistic.

### 3.2 Proposed onboard selector (concrete)

| Task | Output type | Inputs (structured) | Guardrail |
|---|---|---|---|
| Beacon template choice | choice over K templates (K in about 64 to 512) | SOC, bus voltage, temperature, sunlit flag, mode, pass count, last-QSO count, orbit phase | Low confidence yields default template; every template reviewed pre-launch |
| Power mode | choice over mode enum | SOC, SOC trend, eclipse timing, temperature | OBC hard limits (SOC, temperature) override the model |
| Transponder on/off | yes/no probability | SOC, predicted energy, regional demand estimate | Hard duty cap (25 percent) and SOC floor outside the model |
| QSO priority | ordinal score 1 to 5 per station | grid square, elevation, prior contacts, region demand | Round-robin or fairness quota enforced outside the model; the model only ranks within it |

Calibration and abstention: temperature scaling (Guo et al., arXiv:1706.04599, 2017-06-14: a one-parameter method that calibrates well on most datasets tested) plus reliability diagrams and ECE on a held-out set; optionally split-conformal prediction sets (Angelopoulos and Bates, arXiv:2107.07511, 2021-07-15: sets with a user-specified coverage guarantee, no distributional assumptions) so the selector abstains when the set is not a singleton. Out-of-range input features should force the deterministic fallback.

Model family choice [JUDGMENT]: gradient-boosted trees (code-generated C, no runtime) for tabular state; tiny INT8 MLP/CNN (the repo's QSO agent is already specified at under 50 kB INT8) if a neural net is preferred. Tool names such as emlearn, m2cgen, Treelite, X-CUBE-AI, CMSIS-NN, TFLite Micro are from general knowledge [UNVERIFIED]; pick after a spike. Note `CLAUDE.md` states flight software is currently a host simulation with no target port, so the MCU port is itself a work item.

### 3.3 Template diversity (to protect mission appeal)

Quality risk of templates is repetition. Mitigation [JUDGMENT]: a combinatorial template grammar (for example 40 openers x 30 bodies x 20 closers x slot variants gives tens of thousands of strings), every production of which can be enumerated and reviewed mechanically before launch. Optional ground-uploaded phrases would be data only, adopted by authenticated telecommand (consistent with the project's 47 CFR 97.211 design principle in `CLAUDE.md`).

---

## 4. Jev on the ground: what is and is not known

### 4.1 Facts [SRC]
- Developer TypeSafe AI (San Francisco; founded 2024; CEO Diogo Almeida). Limited early access 2026-09-15; waitlist. Proprietary. Architecture, weights, technical paper unpublished (Wikipedia, last edited 2026-10-15).
- Access: hosted API at `https://api.typesafe.ai/v1/systemone`, model name `jev-latest`, bearer key, Python/JS SDKs (noze.it quoting TypeSafe docs). Vendor pricing: $0.042 per million input tokens, output listed as free (TypeSafe launch post and site; the site's own FAQ asks whether pricing is temporary or subsidised, answer not visible).
- Vendor claims: 70 to 500 ms end-to-end latency; 40 to 200 times faster and cheaper than frontier LLMs on its own workflows; "calibrated"; "zero hallucinations". The launch post reports **no ECE or Brier metrics**; its workflow eval uses the average of two frontier LLMs as the reference, not human labels, and the authors state this biases results. The "cannot hallucinate" claim means it cannot return a malformed or off-schema answer, not that it is correct.
- Input limits: text only; 64k tokens total (32k state plus longest question); English primary.
- "API-only": every source reviewed is consistent with this, but I did not find TypeSafe stating it explicitly. The site has an "Enterprise" page I could not read; on-prem or self-hosting is not mentioned anywhere I could read. Even if offered, Jetson-scale self-hosting is implausible given the unpublished architecture.
- Source conflict: Wikipedia says Jev was trained exclusively on synthetic data; TypeSafe's launch post lists "Where does our training data come from?" as an FAQ without a visible answer. Treat the synthetic-only claim as unconfirmed.
- Vendor volatility: Bloomberg reportedly (2026-10-09, via secondary aggregators) had TypeSafe raising about $870M at $7.5B valuation, versus the $40M seed in September. Not material to engineering except that terms and prices are likely to change.

### 4.2 Terms of service: what I read versus what I did not
Read: `typesafe.ai/legal/terms` (website Terms of Use, last updated 2026-09-19), `/legal/acceptable-use-policy` (last updated 2026-09-23), `/legal/privacy-policy`.
- Output reuse or distillation into another model: **not addressed** in any of the three. Closest clauses are reverse-engineering bans (Terms §3(b)(v); AUP §3.1).
- Terms say the licence is personal, non-transferable, and users should not submit confidential or proprietary information (§3(a), §3(c)); TypeSafe may modify or discontinue at any time (§6, §7); liability capped at $100 (§11(b)); US-oriented (§13(g)).
- Privacy policy: states TypeSafe will not train or fine-tune models on Input; retention not stated beyond "as long as reasonably necessary"; outputs not addressed.
- The AUP refers to a **"Master Customer Agreement"** that I could not read. The API very likely falls under it. **That document, any developer/early-access agreement, and the Enterprise terms are the unknowns that decide whether using Jev outputs as training labels is permitted.** The Terms of Use say nothing about ownership of outputs.
- No export-control or high-risk (aerospace) language was found in what I read. Satellite design data can be export-controlled; keep any Jev input to synthetic scenarios.

### 4.3 Recommended ground uses (all conditional on written confirmation from TypeSafe)
1. **Labeling synthetic scenarios for distillation** [JUDGMENT: plausible, with caveats]. Generate synthetic states, ask Jev `choice`/`score`/`noul` questions (for example "which template fits this state?"), keep the full probability vectors as soft labels, and distill into the onboard GBDT/MLP (soft-target distillation is Hinton et al. 2015, arXiv:1503.02531 [UNVERIFIED in this session]). Caveats:
   - For **power management and transponder scheduling the correct label comes from the physics**: use the orbit/power simulation (optimal-control or MPC oracle over SOC), not an LLM-like judge. Jev is only appropriate for subjective calls (template tone, fairness-rubric ranking).
   - Jev's calibration is unquantified, so its soft labels must be re-checked against held-out human or oracle labels before use.
   - Obtain written confirmation that using outputs to train a model that ships in a product is permitted.
2. **Ground-side decision support** (not time-critical): triage of ground-station reports, pass-planning rubric scoring. Low cost at the vendor's quoted price; risk is vendor dependence and data egress.
3. **Second-opinion content check** on ground-generated candidate beacons using a `noul` question ("contains profanity, commercial content, or a factual claim about spacecraft state?"). Useful as triage only. The compliance control remains the deterministic allowlist filter plus human review, because Jev's calibration is unverified.
4. **Never in the flight loop**: no weights, link availability in LEO is intermittent, and it would break the fail-graceful principle in `CLAUDE.md`.

---

## 5. Energy model (the core of D5)

### 5.1 Method
A cold start of a power-gated Jetson has four phases:

E_beacon(N=1) = E_boot + E_load + E_gen + E_shutdown

For N beacons generated in one burst: E_per_beacon = (E_boot + E_load + E_shutdown + N x E_gen) / N.

### 5.2 Inputs, with sources

**Boot time [SRC]:**
- NVIDIA Boot Time Optimization guide (r35.6.0, JetPack 5.1.2): Orin Nano developer kit about **36 s** from cold power-on to login prompt; about **16 s** with the listed steps (disable MB1/MB2 logs, clear combined UART, trim UEFI components and set timeout to 0, `quiet` kernel command line). No per-step savings published.
- NVIDIA r39.2.1 guide: AGX Orin (JetPack 6.0) about 43 s to about 26 s; Thor (JetPack 7.0) about 38 s to about 24 s. Not Orin Nano; shows scale only.
- TkOS (third-party BSP vendor, post dated 2026-07-15): claims **under 5 s** to Linux userspace on JetPack 6.2 by booting the kernel from QSPI and re-engineering rootfs init; quotes 43 s for stock JetPack 6.2. Vendor claim, commercial licence, "boot" ends at userspace, not at model ready, and the stock figure refers to "Jetson NANO" ambiguously. Unverified.

**Power [SRC, all user-reported or snippet-level, none controlled]:**
- NVIDIA forum (JetPack 6.2, Orin Nano Super dev kit, default 7 W mode, 2 CSI cameras; posts 2026-01-23 to 2026-02-06): idle 4.7 W, active 6.2 W; after removing SSD/display/audio/PCIe in device tree, idle 2.5 W. Measured via `tegrastats` VDD_IN.
- A second forum thread (search snippet only, not fetched in full): about 2.6 W whole carrier idle and VDD_IN about 2.18 W after extensive tuning, measured with a Joulescope; S2RAM suspend judged impractical for a once-per-second workload. [UNVERIFIED]
- Earlier forum report (JetPack 5.1.1 dev kit): idle VDD_IN 4.7 W; "less than 7 W" idle in 7 W mode, over 9 W under load. NVIDIA staff: real power is workload dependent; no maximum published.
- NVIDIA power-mode table (Jetson Linux r36.4.3): 7 W mode on the standard 8 GB flash config = 4 CPU cores at 960 MHz, GPU 408 MHz, EMC 2133 MHz. 15 W (Super config) = 6 cores 1497.6 MHz, GPU 612 MHz, EMC 2133 MHz. 25 W (Super) = GPU 918 MHz, EMC 3199 MHz. MAXN_SUPER = GPU 1020 MHz, EMC 3199 MHz. VDD_IN limits: 15 W mode average 15 W, instantaneous 20 W; 25 W mode both 25 W.

**Throughput [SRC]:**
- NVIDIA JetPack 6.2 blog, Table 4 (Orin Nano 8 GB, MLC INT4, tokens/s, "original" versus "Super"): SmolLM2 1.7B 41 to 64.5; Gemma 2 2B 21.5 to 35.0; Llama 3.2 3B 27.7 to 43.1; Phi-3.5 3.8B 24.7 to 38.1; Qwen2.5 7B 14.2 to 21.8. Power mode and test date not stated in the table; "best case".
- NVIDIA forum benchmark thread dated 2025-01-23 (Orin Nano 8 GB devkit, MLC q4f16, power mode not stated, probably MAXN_SUPER): decode tokens/s: SmolLM2-135M **179.6**; SmolLM2-360M **147.9**; Qwen2.5-0.5B **126.3**; Llama-3.2-1B **86.5**; Qwen2.5-1.5B 62.7; SmolLM2-1.7B 59.5; Phi-3.5-mini 32.5; gemma-2-2b 31.9. Prefill tokens/s: 1237 / 1141 / 940 / 1279 / 345 / 897 / 330 / 122 respectively. Inputs only 7 to 20 tokens, outputs 107 to 128 tokens, so prefill figures are for tiny prompts.
- ClawBox figures quoted by ComputingForgeeks (updated 2026-10-13, explicitly not benchmarked by the author): Llama 3.2 1B 44.9 tok/s (Ollama, power mode and quantisation not stated).
- Gemma 3n e2b and Gemma 3 4B via Ollama (NVIDIA forum, Aug to Sep 2025, JetPack 6.2.1): generation 16.6 and 10.1 tok/s.
- Gemma 4 E2B: Jetson AI Lab says it is the Gemma 4 variant that fits Orin Nano, via llama.cpp Q4_K_S; Ollama does not currently work with Gemma 4 on Orin Nano; no tok/s published. HF card: 2.3B effective, 5.1B total with embeddings, Apache 2.0.
- Qwen3.5-0.8B: Jetson AI Lab page lists vLLM only, and shows "No command for this module and engine" for the Orin Nano selection; no performance data.
- **No published tokens/s or power for any model at the 7 W power mode was found.** No published figure for Gemma 3 270M, Gemma 3 1B, TinyLlama or Qwen3-0.6B on Orin Nano was found.

**Energy per token [SRC, secondary]:**
- Reddit r/LocalLLaMA (approximately February 2026, via a mirror snapshot of 2026-06-13): Orin Nano Super 8 GB, llama.cpp CUDA, 1-bit/1.58-bit "Bonsai" models. Ternary-Bonsai-1.7B Q2_0 at 25 W mode: up to 38.4 tok/s, **5.74 output tok/J**, 6.83 W; Bonsai-1.7B Q1_0: 26.0 tok/s, 5.84 tok/J, 4.51 W average, 237 MB. Power was sampled from `VDD_CPU_GPU_CV` **only** (a rail, not module input), so these are lower bounds on whole-module energy. 25 W was the efficiency sweet spot for models up to 4B; MAXN used 10 to 11 percent more energy per token than 25 W. Single unit, unreviewed. The post's JetPack version is not stated.
- EdgeReasoning (arXiv:2511.01866, 2025): AGX Orin 64 GB in MAXN only; not Orin Nano. Qualitatively: decode dominates energy, W4 AWQ reduces energy per token, a 1.5B model draws about 6 W at 4K input. Of limited use here.

**Storage and load [ROM, assumption]:** effective sequential read 0.25 GB/s (microSD to NVMe range; storage medium on the custom carrier is undecided), plus 2 s of CUDA/runtime initialisation. Load energy at about 5.5 W.

### 5.3 Parameter choices (all [ROM])

| Symbol | Value used | Basis |
|---|---|---|
| Boot, stock | 40 s x 4.7 W = **188 J** | 36 to 43 s [SRC] x devkit idle 4.7 W [SRC]; midpoint 40 s |
| Boot, NVIDIA-documented optimised | 16 s x 4.5 W = **72 J** | 16 s [SRC]; 4.5 W assumed, between idle 2.5 to 4.7 W and active 6.2 W |
| Boot, vendor-claimed | 5 s x 4.5 W = 22.5 J | TkOS [SRC, unverified]; reported only for sensitivity |
| Load | (2 s + size / 0.25 GB/s) x 5.5 W | assumption above |
| Generation | (120 prompt tokens / prefill rate + 50 output tokens / decode rate) x P_gen | P_gen = 6.5 W (sub-1.2B) or 7.5 W (2B+); under the ≤ 8 W cap and near the 6.2 W devkit active figure |
| Shutdown | 5 s x 4 W = **20 J** | not researched; assumption. With a read-only rootfs and an OBC hard power cut after the output is committed this could be about 0 J |

**7 W derating [ROM, weakest assumption in the report]:** the published decode rates were taken at an unstated (probably MAXN_SUPER) mode. 7 W mode has 4 CPU cores at 960 MHz (versus 1728 MHz), GPU 408 MHz (versus 1020), EMC 2133 MHz (versus 3199). Small models are launch-overhead bound: SmolLM2-135M reaches only 180 tok/s against a bandwidth bound of roughly 68e9 B/s / 0.1e9 B = 680 tok/s, so CPU clock matters. I apply **x0.4 to published decode and prefill rates** (plausible range 0.3 to 0.6, that is 1.7 to 3.3 times slower). This is a guess to be replaced by measurement (section 8).

### 5.4 Results (cold start, N = 1, [ROM])

Model memory: INT4 weights ≈ parameters x 0.5 B x about 1.1 to 1.3 (scales, higher-precision embeddings). Totals below use the arithmetic above, verified with a script.

| Model | License | Params (total) | INT4 weights [ROM] | Decode @7 W [ROM] | Load | Gen energy | Total, stock boot | Total, documented-optimised boot | Total, vendor-claimed boot |
|---|---|---|---|---|---|---|---|---|---|
| SmolLM2-135M | Apache 2.0 | 135M | 0.10 GB | 72 tok/s (180 x 0.4) | 13 J | 6 J | 227 J (0.063 Wh) | 111 J (0.031 Wh) | 62 J |
| SmolLM2-360M-Instruct | Apache 2.0 | 360M | 0.20 GB | 59 (148 x 0.4) | 15 J | 7 J | **231 J (0.064 Wh)** | **115 J (0.032 Wh)** | 65 J |
| Gemma 3 270M-it | Gemma Terms + Prohibited Use Policy | 270M (0.3B listed) | 0.17 GB | about 60 (assumed like 360M; no Jetson data) | 15 J | 7 J | 230 J (0.064 Wh) | 114 J | 64 J |
| Qwen2.5-0.5B | Apache 2.0 | 494M | 0.30 GB | 50 (126 x 0.4) | 18 J | 9 J | 234 J (0.065 Wh) | 118 J | 69 J |
| Qwen3-0.6B | Apache 2.0 | 0.6B (0.44B non-emb) | 0.40 GB | about 40 (assumed 0.8x of Qwen2.5-0.5B, 28 vs 24 layers) | 20 J | 11 J | **239 J (0.066 Wh)** | **123 J (0.034 Wh)** | 73 J |
| Qwen3.5-0.8B | Apache 2.0 | 0.8B (0.9B listed) | 0.50 GB | about 30 (assumed; hybrid Gated-DeltaNet, no Orin Nano data) | 22 J | 14 J | 244 J (0.068 Wh) | 128 J | 79 J |
| TinyLlama-1.1B | Apache 2.0 | 1.1B | 0.65 GB | about 30 (assumed like Llama 3.2 1B) | 25 J | 13 J | 246 J (0.068 Wh) | 130 J | 80 J |
| Llama 3.2 1B | Llama 3.2 Community License | 1.23B | 0.75 GB | 34 (86 x 0.4) | 28 J | 11 J | 247 J (0.069 Wh) | 131 J | 81 J |
| Gemma 3 1B-it | Gemma Terms + Prohibited Use Policy (same family terms as the 270M card I read; the 1B card itself was not fetched) | about 1B | about 0.65 GB (assumed) | about 34 (assumed like Llama 3.2 1B; no Orin Nano data found) | 25 J | 11 J | 244 J (0.068 Wh) | 128 J | 79 J |
| Phi-3.5-mini | MIT [UNVERIFIED in session] | 3.8B | 2.2 GB | 13 (32.5 x 0.4) | 59 J | 36 J | 303 J (0.084 Wh) | 187 J | 138 J |
| Gemma 4 E2B | Apache 2.0 [SRC: HF card, Jetson AI Lab] | 2.3B effective / 5.1B total | about 2.7 GB | about 13 (assumed like gemma-2-2b) | 70 J | 36 J | 314 J (0.087 Wh) | 198 J | 149 J |

Phi family note: the smallest current Phi models I found are about 3.8B-class (Phi-3.5-mini; Phi-4-mini is of similar size [UNVERIFIED]); the March 2026 Phi-4-reasoning-vision-15B is far larger. No Phi release at or below about 1B was found, and no Phi-5 coverage was found. Any Phi model therefore falls in option F for the 7 W regime whichever release is current. For sub-1B "current" models as of October 2026, the relevant 2026 additions are Qwen3.5-0.8B (March 2026) and the Gemma 4 E2B/E4B edge models (April 2026 per a secondary source; E2B is 2.3B effective, 5.1B total). I found no new sub-1B SmolLM release in 2026 (SmolLM3 is 3B, July 2025).

Reading the table:
- Every sub-1.2B model lands at **227 to 247 J (0.063 to 0.069 Wh)** with stock boot, a spread of about 8 percent. With documented-optimised boot, **111 to 131 J**. Both are below the 360 J D5 limit.
- **Pessimistic case [ROM]** (boot 43 s x 5.5 W = 237 J; load x1.5; generation x2, for example thinking-mode leakage or slower decode; shutdown 25 J): Qwen3-0.6B about **313 J**; 1B-class about **325 J**; 2.5 GB-class about **435 J (fails D5)**. So sub-1.2B models pass with only about 10 to 15 percent margin in the worst case; 2B+ models fail it.
- **Overhead share** (boot + load + shutdown) for Qwen3-0.6B, stock boot: (188 + 20 + 20) / 239 = **95 percent**. Documented-optimised: (72 + 20 + 20) / 123 = 91 percent. Vendor-claimed: (22.5 + 20 + 20) / 73 = 86 percent.
- **Calibration check against the Bonsai figure [ROM]:** 50 tokens at 5.84 tok/J = 8.6 J on the CPU/GPU rail alone for a 1.7B 1-bit model at 25 W. Whole-module energy is plausibly 1.5 to 2 times that, giving about 13 to 17 J, consistent with the 7 to 14 J generation range above.
- Qwen3 defaults to **thinking mode** (HF card). Left on, it can multiply output tokens by 10 or more and the generation term from about 11 J to over 100 J. It must be disabled (`enable_thinking=False`) and capped with `max_new_tokens`. Qwen3.5-0.8B defaults to non-thinking but the card warns it is more prone to thinking loops with recommended sampling.

### 5.5 Batching and the orbit budget [ROM]

Using Qwen3-0.6B with documented-optimised boot: overhead 72 + 20 + 20 = 112 J; generation 10.7 J per candidate.

| N candidates per burst | Energy per candidate |
|---|---|
| 1 | 123 J |
| 5 | 33 J |
| 10 | 22 J |
| 20 | 16 J |

A 20-candidate burst takes about 16 s boot + 4 s load + 20 x 1.7 s = 54 s plus shutdown, inside the 120 s limit. **Does batching close the orbit budget? Yes, narrowly [ROM].** 92 beacons per orbit / 20 candidates per burst is about 5 bursts. Each burst: 112 J overhead + 20 x 10.7 J = 326 J, so 5 bursts = about 1.6 kJ per orbit, inside the 2.2 kJ tumbling-case budget (0.4 W x 5,600 s). Each burst is about 54 s plus shutdown, say 60 s, so 5 bursts is about 300 s per orbit, which is about 5.4 percent duty: at the edge of the 5 percent tumbling cap, inside the 10 percent sun-pointed cap. Reusing candidates across beacons (not every transmitted beacon needs a freshly generated flavor line), or bursts of 30 candidates, brings it comfortably under. Compare the per-beacon cold-start alternative at about 21 kJ per orbit.

Constraint: candidates cannot embed live telemetry, so the OBC must slot-fill at transmit time and filter on acceptance. Report **energy per accepted beacon** (total burst energy / number accepted after the filter), not per generated candidate.

### 5.6 Discriminative path energy [ROM]

**On the MCU (STM32L4R5ZI, Cortex-M4F, 120 MHz, 2 MB flash, 640 KB SRAM):**
- Run-mode current per ST product page: 43 µA/MHz in SMPS mode at 3.3 V, 110 µA/MHz in LDO mode [SRC; conditions not given in the page; verify in datasheet].
- SMPS: 43e-6 A/MHz x 120 MHz x 3.3 V = **17 mW**. LDO: 110e-6 x 120 x 3.3 = **44 mW**. I use 45 mW (conservative).
- GBDT, 100 trees depth 4, about 400 node visits at about 20 cycles = about 8,000 cycles = 67 µs, E = 45 mW x 67 µs = **3 µJ**. Allow 1 ms with feature preparation: **45 µJ**.
- Tiny INT8 MLP/CNN of about 100k MACs at 0.5 to 1 MAC/cycle = 100k to 200k cycles = 0.8 to 1.7 ms: **36 to 75 µJ**.
- BERT-tiny-class transformer (2 layers, hidden 128): non-embedding parameters 2 x (4 x 128^2 + 2 x 128 x 512) = 393,216. For a 32-token input: 393,216 x 32 = 12.6M MACs plus about 0.5M attention = 13.1M MACs, which is 109 to 218 ms at 0.5 to 1 MAC/cycle, **5 to 10 mJ**. The stock 30,522-token vocabulary embedding (about 3.9M parameters, 3.9 MB at INT8) does not fit in 2 MB flash; it needs a trimmed vocabulary (about 2,000 tokens, about 256 KB). **MobileBERT (about 25M parameters) and DistilBERT (about 66M) do not fit** on the MCU at INT8.
- Conclusion: structured telemetry does not need a text encoder; GBDT or a tiny MLP is sufficient and 5 to 7 orders of magnitude below the Jetson path (230 J / 45 µJ = 5 x 10^6).

**On the Jetson (a MobileBERT or BERT-tiny classifier):** inference is about 0.05 to 0.3 s, adding about 0.3 to 2 J. Total is dominated by the same boot/load/shutdown, so about 220 to 230 J cold. **A discriminative model on the Jetson has no energy advantage over a generative one.** The advantage exists only if the model runs on the MCU and the Jetson is never woken for the decision.

---

## 6. Regulatory and content safety (47 CFR 97.113, 97.207)

Verified against eCFR text fetched 2026-10-15:
- **97.113(a)(3)**: no communications in which the licensee or control operator has a pecuniary interest (commercial content).
- **97.113(a)(4)**: prohibits, among others, "messages encoded for the purpose of obscuring their meaning, except as otherwise provided herein", "obscene or indecent words or language", and **"false or deceptive messages, signals or identification"**.
- **97.113(b)**: no broadcasting.
- **97.207(e)**: a space station may transmit one-way communications. **97.207(f)**: space telemetry may use specially coded messages that facilitate communications or relate to spacecraft function. (Beacon text is plain; this note matters only if telemetry encodings are ever considered.) **97.207(b)**: a space station must be able to cease transmitting by telecommand.

Analysis [JUDGMENT unless stated]:
1. The sub-clause that discriminates between the two architectures is **"false or deceptive messages"**, not obscenity. A generative model that writes a number (state of charge, temperature, pass count) can state a wrong value. That is arguably a false message. Obscenity and commercial content are the more obvious risks; wrong telemetry is the more likely one.
2. **Template selection with flight-software-filled slots** has near-zero exposure: the universe of strings is finite, enumerable, and can be mechanically and manually reviewed before launch; slot values are produced by C code from telemetry.
3. **Generative path mitigations** (all required): (a) the model never emits digits and never writes telemetry; templates own all numeric content; (b) constrained decoding to an allowlisted vocabulary (llama.cpp supports GBNF grammar-constrained sampling via `--grammar` / `grammar` field, with documented performance caveats for pathological grammars) or a post-hoc allowlist filter; (c) length assertion of 179 characters or fewer; (d) profanity and commercial-keyword denylist; (e) fail closed to a template; (f) a ground validation sweep (for example 10,000+ prompts at flight settings) with the measured reject rate reported; (g) the control operator remains responsible for content, so recommend an FCC-counsel or ARRL-coordinator read of the "autonomous text generation" posture before enabling.
4. SEU risk: a flipped weight bit on a COTS Jetson can silently garble output; the OBC-side filter must be independent of the Jetson, and must not trust any Jetson-reported "ok".
5. Licence acceptable-use clauses that flow down: Gemma Prohibited Use Policy; Llama 3.2 Acceptable Use Policy (prohibits deceptive content). Neither conflicts with an amateur beacon, but they add compliance paperwork.

---

## 7. Weighted decision matrix

### 7.1 Options
- **A**: MCU-resident discriminative selector (GBDT/tiny MLP over pre-approved templates and decisions). No Jetson wake.
- **B**: Jetson-resident discriminative model (BERT-tiny/MobileBERT-class classifier). Jetson wake per decision.
- **C**: Jetson SLM, up to 0.4B: SmolLM2-135M/360M, Gemma 3 270M.
- **D**: Jetson SLM, 0.5 to 0.8B: Qwen2.5-0.5B, Qwen3-0.6B, Qwen3.5-0.8B.
- **E**: Jetson SLM, about 1 to 1.2B: Llama 3.2 1B, TinyLlama-1.1B, Gemma 3 1B.
- **F**: Jetson SLM, 2 to 4B: Gemma 4 E2B, Phi-3.5-mini (and any current Phi model, all about 3.8B-class or larger).
- **G**: Hybrid (recommended): A as flight baseline plus gated batch generation from C or D for non-numeric flavor lines.

### 7.2 Criteria, weights and rubric

| Criterion | Weight | Scoring rubric (1 to 5) |
|---|---|---|
| Energy per beacon (D5) | **30** | Stock-boot central estimate per beacon: 5 = ≤ 1 J; 4 = 1 to 60 J; 3 = 60 to 180 J; 2 = 180 to 360 J; 1 = over 360 J, or fails D5 in the pessimistic case. Documented boot optimisation would raise every Jetson N=1 option by one point |
| Regulatory / content safety (97.113) | 20 | 5 = finite pre-reviewed string set, no model-authored values; 2 = open-ended text needing filter plus validation |
| Memory / fit | 8 | Footprint versus 8 GB shared (Jetson) or 2 MB flash / 640 KB SRAM (MCU) |
| Quality / novelty (mission appeal) | 12 | Variety and fluency of beacons |
| Maturity / toolchain on Jetson (or MCU for A) | 10 | Published Orin Nano numbers, supported runtimes |
| License | 6 | 5 = Apache 2.0/MIT/own code; 3 = custom licence with obligations |
| Failure modes | 14 | Determinism, boot hang, SEU, runaway generation, thermal, latch-up exposure |

Weights sum to 100. Highest weight on energy per D5; second on 97.113 because a violation is mission-ending (cessation of emissions) while a slow boot is not.

### 7.3 Scores (1 to 5; 5 best) [JUDGMENT, rubric above]

| Criterion (weight) | A MCU selector | B Jetson classifier | C ≤0.4B SLM | D 0.5 to 0.8B SLM | E 1 to 1.2B SLM | F 2 to 4B SLM | G Hybrid |
|---|---|---|---|---|---|---|---|
| Energy per beacon (30) | 5 | 2 | 2 | 2 | 2 | 1 | 4 |
| Regulatory / content safety (20) | 5 | 5 | 2 | 2 | 2 | 2 | 4 |
| Memory / fit (8) | 5 | 4 | 5 | 4 | 4 | 3 | 4 |
| Quality / novelty (12) | 3 | 3 | 3 | 4 | 4 | 5 | 4 |
| Maturity / toolchain (10) | 4 | 4 | 4 | 3 | 5 | 3 | 4 |
| License (6) | 5 | 4 | 4 | 5 | 3 | 5 | 5 |
| Failure modes (14) | 5 | 3 | 3 | 2 | 2 | 1 | 4 |
| **Weighted total (out of 5)** | **4.66** | 3.34 | 2.82 | 2.68 | 2.76 | 2.28 | **4.06** |

Score notes:
- Energy: A 5 (about µJ to mJ). B to E 2 (227 to 247 J stock boot). F 1 (303 to 314 J central, 435 J pessimistic). G 4 (flight path is A; Jetson bursts amortised at N of about 10 or more give about 22 to 33 J per candidate).
- Safety: A and B 5 (output is a template ID). C to F 2 (same risk class regardless of size; larger models follow instructions somewhat better but are no more verifiable). G 4 (generative text limited to a filtered, non-numeric slot).
- Memory: A needs tens of kB of flash; C needs 0.1 to 0.25 GB; F needs about 2.5 to 3 GB of the 8 GB shared pool and enlarges the SEU exposure surface.
- Quality: instruction-following indicators from model cards [SRC]: SmolLM2-360M-Instruct IFEval 41.0; Gemma 3 270M-it IFEval 51.2; Qwen3.5-0.8B non-thinking IFEval 52.1. These are general benchmarks, not beacon-quality measures. A scores 3 only if the template grammar yields thousands of reviewed variants, otherwise 2.
- Toolchain: A is 4 (not 5) because there is no MCU target port yet. D is 3 because the Orin Nano evidence for Qwen3.5 is thin (Jetson AI Lab lists vLLM only; Qwen3-0.6B has no Orin Nano numbers found). E is 5 because Llama 3.2 1B has published MLC (86 tok/s) and Ollama (44.9 tok/s) numbers. F is 3 because Ollama does not work for Gemma 4 on Orin Nano and the E2B audio path has an open issue.
- License: Llama 3.2 Community License carries "Built with Llama" attribution, a naming rule for derivatives, a 700M-MAU clause and an acceptable-use policy; Gemma 3 uses Gemma Terms and a Prohibited Use Policy; the rest are Apache 2.0 (Gemma 4 E2B Apache 2.0 per its HF card). MIT for Phi-3.5-mini is from general knowledge [UNVERIFIED].
- Failure modes: A is deterministic and the OBC runs anyway; Jetson options add boot hang, latch-up, thermal (audit estimate of the heat path is about 0.44 W/K against the 1 W/K requirement), SEU and runaway generation; scale with burst length and memory footprint.

### 7.4 Sensitivity [JUDGMENT]
Re-weighting toward mission appeal (energy 20, safety 15, memory 6, quality **30**, maturity 8, license 5, failure 16; sums to 100):

| | A | B | C | D | E | F | G |
|---|---|---|---|---|---|---|---|
| Score | 4.32 | 3.29 | 2.90 | 2.95 | 3.01 | 2.83 | 4.05 |

Ranking is unchanged (A > G > B > E ≈ D ≈ C > F). C, D, E are within noise of each other under both weightings (2.68 to 2.82; 2.90 to 3.01), so **the matrix does not distinguish among the sub-1.2B SLMs; pick one by measured filter-reject rate, license and toolchain**. A beats G on the numbers; G is recommended because the mission goal is an AI experiment (see section 9), and G keeps A's safety properties.

---

## 8. Measurement procedure for D5 (measured energy per beacon)

**Objective:** decide whether a Jetson configuration meets "≤ 0.1 Wh (360 J) per accepted beacon" including boot and shutdown, and measure the same for the MCU path.

### 8.1 Instrumentation
1. **Measure at the carrier input, not with `tegrastats`.** The baseline limit is "≤ 8 W input including carrier". `VDD_IN` (INA3221) is module-level, sampled at 500 ms, and only starts after boot; it omits bootloader time, carrier regulators and load-switch losses. Use an external source-measure unit or a Joulescope-class meter (confirm range and bandwidth; for example 10 kS/s or better) in series with the **power-gated rail on the bus side of the load switch**, powered from the same bus voltage and converter as flight (converter efficiency changes with load).
2. Keep `tegrastats` (VDD_IN, rail breakdown, temperatures) running only as a cross-check and for phase attribution.
3. **Phase markers:** OBC GPIO and UART timestamps, captured by the meter's GPIO input or a logic analyser: T0 gate-on; T1 kernel up; T2 runtime and model loaded; T3 first token; T4 last token and output committed to OBC; T5 OBC drops the gate. Integrate energy between markers to produce a per-phase table (boot, load, prefill, decode, shutdown).
4. **Off-state:** measure gate-off leakage (load switch and any always-on rail) over at least 60 s. It counts for the whole orbit: 5 mW x 5,600 s = 28 J per orbit as an example.
5. **MCU path:** measure the OBC rail with a µA-range meter (Joulescope or Otii-class) during a decision cycle; report incremental energy over the idle OBC, per decision, and the wake overhead.

### 8.2 Test matrix
- Power modes: **7 W (standard flash config, if retained) and 15 W (Super flash config)**; 25 W optional for reference (see open item 1).
- Images: stock JetPack versus boot-optimised image (NVIDIA steps; optionally vendor BSP if licensed), with the **flight storage medium** and a read-only rootfs/overlay.
- Runtime and model: llama.cpp (Q4_K_M or Q4_0), MLC (q4f16), TensorRT-LLM/Edge-LLM if available; models: SmolLM2-360M-Instruct, Qwen3-0.6B (non-thinking), one 1B reference (Llama 3.2 1B or TinyLlama), and the repo's 3B model as a negative control.
- Prompts: at least 50 fixed slot-filled prompts, about 120 prompt tokens each; `max_new_tokens` of 60; fixed sampling parameters and seeds; thinking mode disabled; token counts logged.
- Burst size: N = 1, 5, 10, 20 candidates per boot; fit E = E_fixed + N x E_gen.
- Temperature: ambient (about 20 °C) and a hot soak (for example 40 to 50 °C, matching the thermal model) because leakage and throttling change both time and power.
- Repetitions: **cold-start N ≥ 10 per configuration (30 for stable p95)**; also warm repeats to isolate E_gen.

### 8.3 Metrics and acceptance
- Report **median and p95** of total gate-on to gate-off energy, the per-phase table, tokens/s, accepted fraction after the OBC filter, and **energy per accepted beacon**.
- **D5 pass:** p95 energy per accepted beacon ≤ 360 J for the proposed flight configuration, and burst energy ≤ 960 J (8 W x 120 s), and orbit-average Jetson energy within the 2.2 to 4.5 kJ budget at the planned burst count. Otherwise fall back to A.

> **Note (2026-10-27, CDR decision C-20, CR-2026-018; this research record is kept as written).** The burst definition the flight design implements, and the one SYS-512, VI-02 and MSC E-02 now use, is a Jetson power-on of at most 10 min and 3.6 kJ at the carrier input, ended early by the energy gate; the 120 s × 8 W = 960 J burst envelope above is superseded. The go/no-go metric stays 0.1 Wh (360 J), counted per accepted beacon line over all accepted lines (SRS-F151), not as a p95 per burst.
- Record the definition used. Recommend D5 be defined as "gate-on to gate-off energy divided by beacons accepted by the OBC filter", and that results be stated for N = 1 and for the planned N.
- Content validation (separate from energy): 10,000+ ground generations at flight settings; report the reject rate by reason (digits, denylist, length, off-vocabulary), the maximum observed length, and zero tolerance for any output that reaches the transmitter without passing the OBC-side filter.
- Selector validation: ECE and reliability diagram on held-out and out-of-distribution sets; abstention rate; decision-latency and energy on target hardware.

---

## 9. Recommendations, risks and open items

### 9.1 Recommendation (detail)
1. **Adopt A as the D5 flight baseline.** It meets D5 with a margin of about 4.5 orders of magnitude (about 10 mJ for a BERT-tiny-class model) to about 7 orders (about 45 µJ for GBDT) against the 360 J limit, has the lowest 97.113 exposure, and keeps all four small decisions on the OBC.
2. **Keep the Jetson generative layer as a gated experiment (G).** Preferred first candidates [JUDGMENT]: **SmolLM2-360M-Instruct** (Apache 2.0, published Orin Nano MLC numbers, smallest footprint) and **Qwen3-0.6B** (Apache 2.0, stronger instruction following, non-thinking only). Fine-tune (LoRA) on the closed set of approved phrases and use constrained decoding. Do not shortlist Gemma 4 E2B or the 3B repo model for the 7 W regime. Gemma 3 270M is capable per its card but carries the Gemma Terms.
3. **Make boot time the gating engineering task.** If the generative layer is wanted, budget effort for the NVIDIA boot-optimisation steps (36 to 16 s, documented), a read-only rootfs with hard power cut (removes shutdown), and, if the budget allows, evaluate the vendor sub-5 s claim on an EM.
4. **Do not boot the Jetson per beacon.** Generate batches (N of about 10 or more), slot-fill on the OBC, and report energy per accepted beacon.
5. **Use Jev only on the ground**, and only after TypeSafe confirms in writing that outputs may be used as training labels (or choose an alternative teacher).

### 9.2 Risks and mitigations
| Risk | Mitigation |
|---|---|
| Boot time and power are unmeasured on the flight carrier | Section 8; keep A as the default so a bad measurement cannot block the mission |
| 7 W derating guess (x0.4) is wrong | Measure at 7 W and 15 W; the conclusion is insensitive because boot dominates |
| Generated text states a false value or contains banned content | Section 6 mitigations; fail closed; ground validation sweep |
| Thinking-mode or runaway generation inflates a burst | `enable_thinking=False`, `max_new_tokens`, OBC wall-clock timer that drops the gate |
| SEU corrupts weights or output | OBC-side independent filter; checksum model on load; burst duty already limited |
| Vendor (TypeSafe) terms or prices change; early access ends | No flight dependency; keep synthetic labels cached with licence evidence |
| MCU port does not exist yet | Plan the port; the selector can first be validated in the host simulation |

### 9.3 Open items for the payload lead (flag, do not resolve here)
1. **7 W mode versus Super flash configuration.** NVIDIA's JetPack 6.2 blog (Table 1) and the r36.4.3 power-mode table list 7 W and 15 W only for the *original* flash configuration, and 15 W, 25 W and MAXN_SUPER for the *Super* configuration. `BASELINE_PARAMETERS.md` line 112 specifies "7 W power mode". Either the baseline uses the original configuration (and then every "Super" tokens/s number above overstates performance, hence the x0.4 derating), or the design needs a 15 W mode. The 15 W (or 25 W) nvpmodel budget is a ceiling the module enforces by throttling, not a draw, so a sub-1B model in 15 W mode may well stay under 8 W input in practice. The real risk is that without a module-enforced cap near 7 to 8 W, the OBC's per-rail current limit becomes the only protection against a burst exceeding the carrier budget (the 15 W mode permits 20 W instantaneous on VDD_IN per the r36.4.3 page). Whether a custom nvpmodel definition with a lower cap is supported would be documented in the unread tail (about 31k characters) of the r36.4.3 power-mode page; I did not read it [UNVERIFIED]. This needs a decision independent of D5.
2. **Storage medium and load path** on the custom carrier (microSD, NVMe, or other): drives load time, power and radiation exposure.
3. **Suspend versus full power-gating.** A suspended Jetson would avoid the 16 to 40 s boot. I found one report that suspend-to-RAM overhead was impractical for a once-per-second workload and no measured suspend power or resume time; this should be measured, because it is the only way to avoid boot energy while keeping per-beacon wake feasible.
4. **Definition of "per generated beacon"** for D5 (N = 1 cold start versus amortised batch; accepted versus generated). Section 8.3 recommends a definition.
5. **Regulatory read** of autonomous text generation by the control operator (97.113 responsibility) before enabling any generative text.
6. TypeSafe **Master Customer Agreement**, early-access agreement and Enterprise terms: obtain and read before any Jev use beyond experimentation.

### 9.4 Resource estimate [JUDGMENT]
- A: about 2 to 4 engineer-weeks (feature definition, synthetic data and training, calibration, C export, MCU port stub, test harness, template grammar and review).
- G generative layer: additional about 3 to 6 engineer-weeks (runtime on Jetson, constrained decoding, filter, LoRA fine-tune, boot optimisation, measurement rig, ground validation sweep).
- Measurement rig: one meter-class instrument plus an EM-grade carrier with a gated rail; about 1 to 2 weeks to run the matrix.

### 9.5 Gaps in the evidence
- No published Orin Nano LLM throughput or power at the 7 W mode.
- No Orin Nano numbers for Gemma 3 270M/1B, TinyLlama, Qwen3-0.6B, Qwen3.5-0.8B.
- Boot figures are for the dev kit, not a custom carrier; JetPack 6 Orin Nano NVIDIA-official boot time not found (the 36 to 16 s figures are JetPack 5.1.2).
- Power during boot was not measured by anyone I could find; I used idle and active figures.
- Shutdown energy not researched.
- Jev's calibration and output-reuse terms are unverified.

---

## 10. Technical deep dive: illustrative designs

### 10.0 Alignment with existing repo docs (read-only check, `docs/BEACON_SPEC.md` and `docs/ML_MODELS.md`)
- **A template and slot format already exists** in `BEACON_SPEC.md` ("Template Set (non-exhaustive)": for example `de {CALL}: Sunlit but conserving—SOC {SOC}%. RF {RFSTATE}. 73!`), with a compact fallback `de <CALL>: SOC=<pct> <SUN|ECLIPSE> RF=<ON|OFF>. 73!`, plus a machine-readable tail (`T= M= SOC= SUN= RF= PWR= QSO=`). Option A can reuse this directly: the selector chooses among these templates (expanded to a larger reviewed set) and flight software fills `{CALL}`, `{SOC}`, `{RFSTATE}`.
- **Length limit mismatch:** `BEACON_SPEC.md` limits the text to 120 ASCII bytes, the tail to 80 bytes, and total Info to 200 bytes; the D5 task statement says up to 179 characters of text. Reconcile before sizing token counts (120 bytes is about 30 to 40 tokens; 179 characters is about 45 to 60).
- **Non-ASCII characters in the spec's own examples:** the template and example lines contain a typographic apostrophe (’) and an em dash (—), which are not ASCII and are 3 bytes each in UTF-8, while the same file requires ASCII-only text. Any template set derived from it needs an ASCII pass, and the OBC-side filter should enforce it.
- **`ML_MODELS.md` energy method omits overheads:** it computes "Energy per beacon = tokens_per_beacon x J/token (e.g., 25 tokens x 1.5 J/token = 37.5 J)", with no boot, load, or shutdown term, and states both "process suspended between beacons" and "Jetson power-gated between uses" (these imply very different wake costs). Its per-token numbers are broadly consistent with mine (Qwen2.5 0.5B 0.1 to 0.3 J/token versus about 0.17 J/token here), but the per-beacon totals are not: about 5 J (25 tokens x 0.2 J) versus about 230 J cold-start for the same model.
- **`ML_MODELS.md` names Llama 3.2 3B as the primary model** with 4 to 8 tok/s at 7 to 12 W. Even before the boot overhead, 7 to 12 W exceeds the ≤ 8 W input budget, and a 3B model would fall in option F (about 300 J or more per cold start, over the 360 J limit in the pessimistic case). Its "TinyLM ~0.8 to 1.2 MB transformer" line is not an off-the-shelf model I could identify.
- The doc's J/token table lists Llama 3.2 1B, TinyLlama, Qwen2.5 0.5B and Phi-3 mini 3.8B; the candidates not yet in it that this report covers are SmolLM2, Qwen3-0.6B, Qwen3.5-0.8B, Gemma 3 270M/1B and Gemma 4 E2B.

### 10.1 Slot-filled template (flight-software side, illustrative; reuse the `BEACON_SPEC.md` format)
```
template[17] = "de {CALL}: Sunlit but conserving - SOC {SOC}%. RF {RFSTATE}. 73!"      # ASCII hyphen, not an em dash
```
Rules: all `{...}` slots are filled by C code from telemetry; the model emits only the template index (A) or a non-numeric flavor phrase from an allowlist (G). Assert the length limit (120 bytes in `BEACON_SPEC.md`, or 179 characters per the D5 task; reconcile) and ASCII-only before framing into the AX.25 UI frame.

### 10.2 Selector decision flow (pseudocode)
```
p = model(features)              # calibrated probability vector
if out_of_range(features): use_rule_default()
elif max(p) < tau:                use_rule_default()   # abstain
else:                             act(argmax(p))
hard_limits_enforced_outside_model()   # SOC floor, duty cap, temperature, TX-OFF
```

### 10.3 Generative burst (Jetson side, sketch)
1. OBC gates on the rail when SOC, sunlit and temperature conditions hold (baseline limits).
2. Jetson boots the optimised image, loads the model, generates N candidates with a grammar restricting vocabulary and forbidding digits, `max_new_tokens` of 60, thinking disabled.
3. OBC receives candidates, verifies length, character set, denylist; stores accepted ones; drops the gate (hard cut, read-only rootfs).
4. OBC slot-fills telemetry at transmit time.

---

## 11. References (with dates)

**Jev / TypeSafe AI**
1. Wikipedia, "Jev (AI model)", last edited 2026-10-15. https://en.wikipedia.org/wiki/Jev_(AI_model). Release date, architecture description, proprietary status; cites Forbes (2026-09-15), TechCrunch (2026-09-18), The Register (2026-09-16), TypeSafe docs. Used for general facts only.
2. TypeSafe AI, home page, retrieved 2026-10-15. https://typesafe.ai. Early access, pricing, links to legal pages.
3. TypeSafe AI docs, "Jev" introduction, retrieved 2026-10-15. https://docs.typesafe.ai/introduction. Question types, independence of questions.
4. TypeSafe AI blog, "Introducing System One Models & Jev" (byline 2026-09-15). https://typesafe.ai/blog/introducing-system-one-models-and-jev. Vendor claims, no calibration metrics, evaluation caveats.
5. TypeSafe AI Terms of Use (last updated 2026-09-19), Acceptable Use Policy (2026-09-23), Privacy Policy. https://typesafe.ai/legal/terms, /legal/acceptable-use-policy, /legal/privacy-policy. Output reuse unaddressed; Master Customer Agreement referenced but not seen.
6. noze.it, "Jev and System One: typed decisions". https://www.noze.it/en/insights/jev-system-one-typed-decisions/ (date not shown). Secondary summary of API, limits, confidence definition, pricing.
7. Secondary news on funding (Dealroom; aggregators citing Bloomberg 2026-10-09). Low reliability, not material.

**Jetson hardware, power, boot**
8. NVIDIA, "JetPack 6.2 Brings Super Mode to Jetson Orin Nano and Orin NX" (Dec 2024 to Jan 2025). https://developer.nvidia.com/blog/nvidia-jetpack-6-2-brings-super-mode-to-nvidia-jetson-orin-nano-and-jetson-orin-nx-modules. Power-mode configurations, clocks, 102 GB/s, MLC INT4 tokens/s table.
9. NVIDIA Jetson Linux r36.4.3 Developer Guide, Platform Power and Performance, Orin Nano series. https://docs.nvidia.com/jetson/archives/r36.4.3/DeveloperGuide/SD/PlatformPowerAndPerformance/JetsonOrinNanoSeriesJetsonOrinNxSeriesAndJetsonAgxOrinSeries.html. Per-mode CPU/GPU/EMC clocks, VDD_IN limits.
10. NVIDIA Jetson Linux r35.6.0 Developer Guide, Boot Time Optimization (JetPack 5.1.2). https://docs.nvidia.com/jetson/archives/r35.6.0/DeveloperGuide/SD/BootTimeOptimization.html. Orin Nano devkit 36 s to 16 s.
11. NVIDIA Jetson Linux r39.2.1 Boot Time Optimization. https://docs.nvidia.com/jetson/archives/r39.2.1/DeveloperGuide/SD/BootTimeOptimization.html. AGX Orin 43 to 26 s; Thor 38 to 24 s.
12. TkOS, "Sub five second boot on NVIDIA Orin NANO", dated 2026-07-15. https://tkos.co.il/news/Sub-five-second%20boot%20on%20NVIDIA%20Orin%20NANO/. Vendor claim, unverified.
13. NVIDIA Developer Forums, "Reducing idle power on Orin Nano Super Dev Kit", posts 2026-01-23 to 2026-02-06. https://forums.developer.nvidia.com/t/reducing-idle-power-on-orin-nano-super-dev-kit/358482. Idle 4.7 W, active 6.2 W, tuned idle 2.5 W.
14. NVIDIA Developer Forums, "Jetson Orin Nano 8GB - Lowest Theoretical Operating Power Consumption". https://forums.developer.nvidia.com/t/jetson-orin-nano-8gb-lowest-theoretical-operating-power-consumption/348662. Fetch failed (HTTP 503); figures from a search snippet only.
15. NVIDIA Developer Forums, "Orin nano benchmark on R36.4.3 (JetPack 6.2)", rows dated 2025-01-23. https://forums.developer.nvidia.com/t/orin-nano-benchmark-on-r36-4-3-jetpack6-2/321250. Decode and prefill tokens/s for SmolLM2, Qwen2.5, Llama 3.2.

**LLM performance and energy on Jetson**
16. Reddit r/LocalLLaMA, Bonsai on Orin Nano Super (approx. February 2026; mirror snapshot 2026-06-13). https://reddit.sentinel-team.org/posts/1u21lhp/snapshots/2026-06-13T02%3A56%3A06.64152Z. Tokens per joule by power mode; rail-only power sampling; unreviewed.
17. Kubwimana and Huang, "EdgeReasoning", arXiv:2511.01866 (2025). https://arxiv.org/abs/2511.01866. AGX Orin only; qualitative energy trends.
18. hokwangchoi, jetson-orin-nano-benchmarks (JetPack 6.2.2, MAXN_SUPER; 2026). https://github.com/hokwangchoi/jetson-orin-nano-benchmarks. Cosmos-Reason2-2B numbers; no power data.
19. NVIDIA Developer Forums, "Gemma 3 and Gemma 3n on Jetson Orin Nano Super" (2025-08 to 2025-09, reply 2026-01-03). https://forums.developer.nvidia.com/t/gemma-3-and-gemma-3n-on-jetson-orin-nano-super/337513.
20. Jetson AI Lab: model index, Gemma 4 on Jetson tutorial, Gemma 4 E4B and Qwen3.5-0.8B pages (retrieved 2026-10-15). https://www.jetson-ai-lab.com/models/ and /tutorials/gemma4-on-jetson/. Fit and runtime notes; no tokens/s.
21. ComputingForgeeks, "Edge LLM: Jetson Orin vs AI HAT 2 vs Hailo 10H" (updated 2026-10-13). https://computingforgeeks.com/edge-llm-jetson-orin-vs-ai-hat-2-vs-hailo-10h/. Secondary; explicitly quotes others.

**Models and licenses (Hugging Face model cards, retrieved 2026-10-15)**
22. HuggingFaceTB/SmolLM2-360M-Instruct (Apache 2.0; IFEval 41.0). 23. Qwen/Qwen3-0.6B (Apache 2.0; thinking mode default). 24. Qwen/Qwen3.5-0.8B (Apache 2.0; released early March 2026; non-thinking default; intended for prototyping and fine-tuning). 25. google/gemma-3-270m-it (Gemma license; IFEval 51.2). 26. google/gemma-4-E2B-it (Apache 2.0; 2.3B effective, 5.1B total). 27. meta-llama/Llama-3.2-1B-Instruct (Llama 3.2 Community License). 28. TinyLlama/TinyLlama-1.1B-Chat-v1.0 (Apache 2.0). Gemma 4 release context: April 2, 2026 per one secondary source (not confirmed from Google).

**Regulatory**
29. 47 CFR 97.113 and 97.207 via eCFR, retrieved 2026-10-15. https://www.ecfr.gov/current/title-47/part-97/section-97.113 and /section-97.207.

**MCU and methods**
30. STMicroelectronics STM32L4R5ZI product page: 110 µA/MHz (LDO), 43 µA/MHz (SMPS at 3.3 V), 120 MHz, 2 MB flash, 640 KB SRAM. https://www.st.com/en/microcontrollers-microprocessors/stm32l4r5zi.html (conditions not read; datasheet to be checked).
31. Guo et al., "On Calibration of Modern Neural Networks", arXiv:1706.04599, 2017-06-14. 32. Angelopoulos and Bates, "A Gentle Introduction to Conformal Prediction...", arXiv:2107.07511, 2021-07-15 (revised 2022-12-07). 33. llama.cpp grammars README (GBNF). https://github.com/ggml-org/llama.cpp/blob/master/grammars/README.md. 34. Hinton, Vinyals, Dean, "Distilling the Knowledge in a Neural Network", arXiv:1503.02531, 2015 [UNVERIFIED in this session].
