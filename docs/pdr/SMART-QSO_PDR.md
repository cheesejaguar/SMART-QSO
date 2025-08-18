# SMART-QSO (Self‑Managing Amateur Radio Transponder with Quantum State Optimization)

**Mission Phase:** Pre‑Phase B / Phase B — Preliminary Design Review (PDR)

**Spacecraft Class:** 1U CubeSat (10 × 10 × 11.35 cm max envelope per CDS)

**Primary Novelty:** Autonomous, community‑taught amateur radio transponder using onboard TinyML/LLM models accelerated by a Jetson Orin Nano Super (declocked, power‑optimized). Operates as a conventional ham transponder if AI is unavailable (fail‑graceful).

---

## 1. Executive Summary
SMART‑QSO is a 1U CubeSat that experiments with **agentic AI** to manage an amateur radio linear/FM transponder in orbit. The satellite learns power availability, predicts regional demand, prioritizes QSOs fairly, and composes context‑aware telemetry beacons. A **Jetson Orin Nano Super** (declocked, DVFS‑limited) serves as the AI payload for bursty inference and offline micro‑training; everyday autonomy runs on an **ultra‑low‑power controller** (Ambiq Apollo4 or STM32L4 class) to preserve energy.

**Key Outcomes at PDR:**
- Mission concept validated by system budgets and regulatory path.
- Payload and bus interfaces defined; harness and software ICDs drafted.
- Power/thermal duty‑cycling concept supports AI bursts in a 1U power envelope with deployable arrays.
- Link budgets close for UHF↔VHF amateur transponder with omnidirectional tape antennas and typical ground stations.
- De‑orbit compliance via passive drag/tether device sized for ≤5‑year disposal at ≤500 km.

---

## 2. Mission Objectives & Success Criteria
### 2.1 Level‑1 (Mission) Objectives
1. **Provide amateur transponder service** (UHF/VHF) with ≥12 months on‑orbit availability and ≥90% autonomous ops.
2. **Demonstrate onboard learning & scheduling** that reduces energy per served minute by ≥20% vs. fixed‑duty operation.
3. **Show fairness improvements**: allocate access across regions so the Gini coefficient of minutes‑served per region is ≤0.35 over 60 days.
4. **Demonstrate community‑driven model updates** via open amateur uplink (no encryption): accept, validate, and apply at least one model/parameter update per quarter without impacting TT&C.
5. **End‑of‑Life compliance:** passively de‑orbit to reentry within 5 years after mission end.

### 2.2 Level‑2 (System) Requirements (selected)
- 1U form factor, ≤1.80 kg wet mass; CG within CDS rail limits; deployables remain within envelope pre‑deploy.
- Average orbit power **≥4 W** (sunlit average ≥8–10 W) using two deployable wings; energy storage ≥50 Wh.
- RF: **Transmit‑only 2 m downlink 145.8–146 MHz** (final per IARU coordination). Beacon‑only operations via Stensat module; conducted power selectable; duty‑cycle adaptive.
- **Payload computing:** Jetson Orin Nano Super (declocked to minimum stable clocks) for burst inference; MCU for background autonomy ≤50 mW average.
- **Software:** RTOS on low‑power MCU; TinyML for detectors; TensorRT/ONNX Runtime INT8 on Jetson for TinyLM (~1 MB) when sun & thermal allow.
- **ADCS:** 3‑axis magnetorquers + coarse sun sensors; detumble ≤1 orbit; pointing knowledge ≤10° (for power biasing only; comms omni).
- **Thermal:** keep Jetson module case ≤70 °C; battery 0–40 °C nominal; heaters allowed ≤1 W burst.
- **Regulatory:** Amateur‑satellite service & IARU coordination; operations in the clear (no encryption); separate EOL compliance documentation.

---

## 3. Concept of Operations (ConOps)
### 3.1 Orbit & Lifetime
- **Baseline orbit:** 475–525 km SSO, LTAN ~10:30. **Design life:** 12–18 months; **EOL:** drag/tether deployment for ≤5‑year reentry.

### 3.2 Ops Timeline by Mission Phase
- **Phase 0–A (pre‑launch):** licensing, IARU coordination, flat‑sat tests, environmental qual.
- **Phase B–C (integration):** functional test, sun‑sim power‑positive tests, RF compatibility testing with partner ground stations.
- **Phase D (commissioning, ~2 weeks):** detumble; basic beacon at 1200 bps AX.25; characterize power/thermal; enable Jetson bursts only after margins verified.
- **Phase E (primary ops):** adaptive beacon with personified text when power/thermal allow; template fallback otherwise; cadence 60–120 s adaptive.
- **Phase F (EOL):** deploy drag/tether; send final beacon; passivation.

---

## 4. Space Segment Architecture
### 4.1 Structure & Mechanisms
- **Bus:** 1U aluminum 6061‑T6 skeleton; rails to CDS; internal stack with corner posts; thermal straps to radiator face.
- **Deployables:** two hinged solar wings (each ~10×20 cm) with hold‑down/release and burn‑wire HDRM; UHF/VHF tape antennas with burn‑wire release.
- **Deorbit:** compact passive device (drag sail or electrodynamic tether module) mounted zenith panel; 1‑shot deployment at EOL.

### 4.2 Electrical Power Subsystem (EPS)
- **Generation:** body‑mounted + 2 deployables, ≥30% GaAs cells; **sunlit average** 8–10 W at nominal pointing; **orbit‑average target** 3.5–5 W (SSO).
- **Storage:** 4s Li‑ion pack, **≥50 Wh**; battery protection & balancing; heaters with thermostat.
- **Power Mgmt:** MPPT + load‑shedding; AI and RF on separate power domains with supervised latching; inrush limiters for Jetson bursts.

### 4.3 Command & Data Handling (C&DH)
- **Low‑power OBC:** Ambiq Apollo4 (or STM32L4) running FreeRTOS; always‑on autonomy; <1 mA sleep; watchdog and brown‑out.
- **AI payload:** **Jetson Orin Nano Super** on isolated 5 V/12 V rails; UART/USB link to OBC; power‑gated by EPS; DVFS profiles managed by OBC.
- **Mass memory:** FRAM for state/params (≥4 MB); microSD/NAND for logs (≥128 MB).

### 4.4 RF & Transponder
- **Bands:** Downlink 2 m (145.8–146.0 MHz); no uplink.
- **Mode:** Transmit‑only beacon — AX.25 UI at 1200 bps AFSK via Stensat beacon module.
- **Antennas:** deployable quarter‑wave tape monopole for 2 m (near‑omni pattern).
- **Beacon:** 1200 bps AFSK every 60–120 s; adaptive cadence per power/thermal.

### 4.5 ADCS
- **Sensors:** 3‑axis rate gyro (MEMS), coarse sun sensors (4–6 diodes), magnetometer.
- **Actuators:** 3‑axis **magnetorquers** (board or rod type). No reaction wheels to save mass/power.
- **Modes:** Launch‑lock → Detumble → Power‑bias Sun‑seek → Safe → Nominal.

### 4.6 Thermal
- Conductive paths from Jetson module and PA to radiator face; pyrolytic graphite sheet + aluminum spreader; black‑paint radiator.
- Jetson duty‑cycle & throttling from onboard thermistors; battery insulated, kept 0–40 °C via heaters.

### 4.7 Fault Mgmt & FDIR
- **Hard partitions:** TT&C and transponder operate independent of AI.
- **Latched states:** Auto‑recover to "dumb transponder + basic beacon" on watchdog trip.
- **Safe‑mode:** RX‑only + beacons; AI disabled until power/thermal OK.

---

## 5. Agentic AI Payload & Software
### 5.1 Agent Suite (power‑aware)
1. **Intelligent Power Management Agent** (MCU; avg ~0.2 W ensemble with house‑keeping)
   - Learns per‑orbit insolation and seasonal drift; 90‑min ahead energy forecast; selects cadence and Jetson enablement.
2. **Smart Beacon Agent** (Jetson when available; MCU fallback)
   - Composes human‑readable telemetry using a **TinyLM (eg. llama 3.2 3B)** with deterministic tail; template fallback when constrained.
3. **Signal/Anomaly Classifier** (MCU; ~0.1 W when active)
   - Fine-tuned nano-model to detect signal anomalies, and performed limited FDIR (Fault Detection, Isolation & Recovery).

### 5.2 Edge LLM Design
- **Model selection (off‑the‑shelf):** Llama 3.2 3B (Instruct) quantized (INT8/FP8) and compiled with TensorRT for the declocked Jetson Orin Nano Super. Treat as opportunistic: duty‑cycled and disabled under low‑power/thermal limits with template fallback.
- **Prompting (system prompt):** Use a fixed system prompt that instructs the model to interpret CubeSat health inputs and output a single, friendly, first‑person sentence ≤120 ASCII bytes, suitable for AX.25 beacons, without emojis or non‑ASCII. Example (abridged):
  - System: "You are SMART‑QSO, a 1U CubeSat. Given health fields (TIME, MODE, SOC, SUN, RF, PWR, QSO, CALL), write one short, human‑readable status line that begins with 'de <CALL>:' and ends with '73!' unless MODE=SAFE. Keep ≤120 ASCII bytes; no emojis; plain text."
  - User: "CALL=SMARTQ-1 TIME=2025-08-18Z MODE=ACTIVE SOC=78 SUN=1 RF=1 PWR=GOOD QSO=12"
- **Inputs (structured features):** CALL, TIME (UTC, ISO‑8601 Z), MODE {ACTIVE|IDLE|SAFE}, SOC (0–100), SUN {0|1}, RF {0|1}, PWR {GOOD|FAIR|LOW}, QSO (recent count). Passed as a concise text prefix to the model.
- **Decoding constraints:** greedy or top‑k=3, temperature ≤0.7; enforce ASCII‑only and ≤120‑byte cap; reject/regen on violations; if constraints cannot be met within budget, fall back to the template per `docs/BEACON_SPEC.md`.
- **Tokenizer:** use the model’s native tokenizer (no custom tokens). Domain terms (callsigns, grids) are passed literally in the prompt.
- **Latency/power target:** declocked Jetson with TensorRT INT8/FP8 aims for <250–400 ms per 24‑token generation at batch=1; disabled in eclipse/low‑SOC. MCU continues basic operations and can emit template beacons without the LLM.

### 5.3 On‑Orbit Learning (No Uplink) - Stretch Goal
- **Learning mode:** gradient accumulation on MCU with **very small steps**; apply during sunlit surplus only.
- **No uplink updates:** no over‑the‑air model updates; models are fixed at launch with on‑orbit micro‑tuning only. All beacon content remains plain‑text and logged publicly.

### 5.4 Software Stack
- **RTOS:** FreeRTOS (OBC), baremetal control loops for power critical paths.
- **ML runtimes:** TFLM on MCU; TensorRT/ONNX Runtime INT8 on Jetson for supported ops; custom lightweight kernels as needed.
- **Languages:** C/C++ for flight; Python only for ground tools.

---

## 6. Regulatory & Ethical Considerations
- **Amateur‑satellite service:** apply for IARU coordination; use in‑the‑clear protocols (AX.25, documented frames). No encryption/obscuration. Control commands limited and documented.
- **Spectrum plan:** 70 cm uplink / 2 m downlink (final per IARU panel); beacon cadence adaptive but ≤ 1/min nominal.
- **Safety of Life:** emergency detection prioritizes access but never blocks manual use.
- **Data policy:** all model updates, logs, and beacons are openly documented; community participation welcomed.

---

## 7. Budgets & Performance
### 7.1 Mass Budget (goal ≤1.80 kg; PDR est. 1.30 kg before margin)
| Subsystem | Mass [kg] |
|---|---:|
| Structure & rails | 0.18 |
| Deployable panels & hinges | 0.12 |
| OBC (MCU board) | 0.06 |
| **Jetson Orin Nano Super** | 0.10 |
| EPS PCB & MPPT | 0.10 |
| Battery pack (~50 Wh) | 0.20 |
| UHF/VHF radio & PA | 0.20 |
| Antennas & HDRMs | 0.05 |
| ADCS (magnetorquers+sensors) | 0.15 |
| Deorbit device | 0.12 |
| Harness & fasteners | 0.08 |
| Thermal (straps, PGS, MLI) | 0.03 |
| **Subtotal** | **1.30** |
| **System margin (20%)** | **0.26** |
| **PDR total** | **1.56** |

### 7.2 Power Modes & Budget
**Generation assumptions:** two deployable wings + body panels; sunlit average 8–10 W; orbit‑average 3.5–5 W (SSO). Battery 50 Wh.

| Mode | Load Elements | Power [W] | Notes |
|---|---|---:|---|
| **Idle** | MCU sleep, RX monitor, 1/min TinyML ping | **0.3** | RX squelch duty‑cycled; MCU deep sleep <1 mW, wake housekeeping. |
| **Active Transponder** | RF TX/RX 0.5–2 W RF (bus 2–3 W), AI bursts (Jetson), ADCS actuation | **~4.0 (avg)** | AI bursts ≤10–15% duty; magnetorquers ≤0.5 W avg during passes. |
| **Learning** | MCU micro‑training + occasional Jetson assist | **~1.0 (avg)** | Sunlit only; paused on battery <40%. |

**Energy balance:** With orbit‑avg 4 W available, transponder can operate at **~20–30% duty per orbit** while preserving SOC; higher duty in continuous sun or with reduced PA power.

### 7.3 RF Link Budget (indicative)
- **Downlink 145.9 MHz (2 m):** TX 2 W (33 dBm) + antenna 2 dBi → EIRP 35 dBm. Slant range 1000 km → FSPL ≈135.7 dB. Ground Yagi 12 dBi; misc losses 4 dB → **Pr ≈ −92.7 dBm**, margin suitable for 1200 bps AFSK.

### 7.4 Data Handling
- Housekeeping ≤50 kB/day; ML logs ≤1 MB/day (throttled). Beacons ≤1 kB/day. Storage headroom ≥128 MB.

---

## 8. Verification & Validation (V&V) Plan
**Standards:** GEVS vibe/thermal‑vac; EMI/EMC screening; functional and RF compatibility testing.

**Verification methods:**
- **Analysis (A):** Power/thermal, link budgets, FMECA, deorbit performance.
- **Test (T):** TVAC, random vibe, shock (HDRM), deployment tests, RF radiated/ conducted, sun‑sim MPPT.
- **Inspection (I):** fit checks to CDS; harness; workmanship.
- **Demonstration (D):** over‑the‑air end‑to‑end passes with partner ground stations.

**Qualification approach:** Proto‑flight at the unit level; development flatsat for software & ML regression; Jetson thermal throttling characterization with radiator mock‑ups and DVFS sweeps.

---

## 9. Risk Assessment (Top‑10)
| ID | Risk | Likelihood | Consequence | Mitigation |
|---|---|---|---|---|
| R1 | **Jetson Orin Nano Super** power spikes/thermal overlimit | M | H | Declock + DVFS limiting; power‑gate with soft‑start; radiator strap; temperature derates; inference duty‑cycle caps. |
| R2 | Jetson module not space‑qualified (TID/SEU) | M | H | Treat as opportunistic payload; watchdog‑supervised; reboot isolation; AI optional; flight software runs without it. |
| R3 | Power shortfall vs ops demand | M | M | Deployables sized for ≥8 W sunlit; AI scheduler ensures energy‑positive plan; reduce PA power; shed learning mode. |
| R4 | IARU frequency coordination changes | L | M | Wide‑tunable radio; reconfigurable beacons; finalize bands early; public documentation. |
| R5 | Community updates abused (spam/poisoning) | M | M | Public, rate‑limited frames; bounded update schema; sandbox; quorum voting; quick rollback; logs in beacon. |
| R6 | EMI between TPU, PA, and RX | M | M | Segregated grounds; filters; spread‑spectrum clocks; conducted emissions tests; physical separation. |
| R7 | Battery low‑temp capacity loss | M | M | Heaters with strict SOC gates; eclipse‑mode power saving; charge protocol tuned. |
| R8 | Antenna/array deployment failure | L | H | Redundant burn‑wires; visual inspections; environmental/TVAC deployment testing. |
| R9 | ADCS under‑performance | M | M | Conservative power‑bias sun tracking; omni antennas; no precision pointing required. |
| R10 | EOL device fails to deploy | L | H | Redundant timers/commands; device with flight heritage; passive drag bias from panels provides partial help. |

---

## 10. Schedule (Notional, Accelerated)
- **PDR (T‑8–9 mo)** → flatsat bring‑up; beacon validation; thermal/power DVFS plan
- **CDR (T‑7 mo)** → EPS/array CDR; RF beacon CDR; EMI pre‑scan
- **EM/PFU build (T‑6–3 mo)** → vibe/TVAC protoflight; functional conformance
- **FRR (T‑1 mo)** → configuration freeze; documentation pack‑out

---

## 11. Cost (ROM, hardware only)
- Structure & mechanisms: $12k
- Deployable arrays (2 wings, MPPT): $20k
- EPS + battery (50 Wh): $12k
- UHF/VHF transceiver + PA + antennas: $25k
- ADCS (mag torquers + sensors): $8k
- **Jetson Orin Nano Super** + harness & rad mitigations: $0.5k–3k
- Deorbit device: $8k–15k
- Misc (harnessing, machining, environmental test): $20k
- **Total hardware ROM:** **$105k–$125k** (ex‑launch & ops)

---

## 12. Ground Segment & Community Program
- **Primary stations:** university/club stations for commissioning and ops rehearsals.
- **Open data:** publish beacon fields, update schemas, and fairness metrics; leaderboards for “teaching the sat.”
- **Ops tools:** web dashboard (open source) to view energy budget, fairness, and model diff.

---

## 13. Open Issues & Key Trades (Carry to CDR)
1. **Beacon configuration:** deviation/bandwidth, cadence, and EIRP margins in shared 2 m band.
2. **Arrays:** 2‑wing vs. 3‑wing deployables; hinge torque margins.
3. **Memory tech:** FRAM vs. MRAM for higher TID margin; vendor selection.
4. **LLM runtime:** TensorRT/ONNX op coverage and latency vs. declocking levels; MCU fallback templates.
5. **Regulatory detail:** finalize IARU frequency coordination specifics and any national administration requirements; no uplink operations.

---

### PDR Disposition
The SMART‑QSO design is **feasible within a 1U** envelope with margin provided by deployable solar wings, strict power scheduling, and a fail‑graceful architecture that treats AI as an opportunistic enhancement to a conventional amateur transponder. The team recommends proceeding to CDR with focused engineering on EPS thermal‑power margins, Jetson DVFS/declocking management, and regulatory confirmation of the open update workflow.
