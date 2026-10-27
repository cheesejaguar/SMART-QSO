# OBC Selector and Transponder Scheduler: Detailed Design

**Document ID**: SQSO-PAY-SEL-001
**Version**: B
**Date**: 2026-10-27
**Status**: CDR detailed design; not implemented (fabrication-phase task, §11)

**Parents**: `docs/BASELINE_PARAMETERS.md` §2 and §6 (decision D5); `docs/payload/AI_MODEL_SELECTION.md` (SQSO-AUD-006) §1, §5;
`docs/BEACON_SPEC.md` §6 (content filter); `software/rust/sqso-fsw/src/transponder.rs` (the gates).
**Requirements**: SRS-F100 to SRS-F107, SRS-F170 (`docs/software/SOFTWARE_REQUIREMENTS.md` §3.11).

## 1. Purpose and scope

Decision D5 makes the flight AI "a small calibrated selector on the OBC choosing among pre-approved templates and
proposing power mode, transponder schedule and QSO priority; hard limits stay outside the model". SQSO-AUD-006 records
the decision and the measurement plan, not a design. As built (2026-10-27) the OBC sends its templates in turn and the
transponder follows its gates alone (`mission/CONOPS.md` §5). This document is the detailed design of the selector and
of the transponder scheduler it feeds, complete enough to implement, train and verify without further design
decisions. It also records one change to D5 (§3.4: no QSO-priority output) for the chair.

## 2. Architecture

```
measurements (EPS telemetry, ADC, mode, transponder record, clock)
        │
        ▼
 feature builder (sqso_fsw::selector::features)   ── stale / missing / out of range ──► ABSTAIN
        │ 16 × i16, quantized
        ▼
 model (sqso_fsw::selector::Model): gradient-boosted trees, integer arithmetic,
        three heads: H1 template, H2 transponder bucket, H3 power-mode preference
        │ logits (i32)
        ▼
 calibration + abstention (per head)                ── confidence below threshold ──────► ABSTAIN
        │ proposal
        ▼
 hard limits (outside the model, unchanged code):   transponder gates (transponder.rs), mode manager
                                                     thresholds (mode.rs), TX-OFF, RF silence,
                                                     contact inhibit, payload guard, content filter
        │
        ▼
 actuation: template id → beacon.rs fills every value from measurements; XPDR_EN heartbeat;
            mode event
```

ABSTAIN means the fixed rules of today decide: templates in turn, the transponder on whenever its gates hold, the mode
manager alone. The selector can only choose among pre-approved outputs and can only make the transponder and the power
mode more conservative than the rules (§5).

## 3. Outputs

### 3.1 H1: beacon template

A class among K ≤ 16 pre-approved templates (the template table of `software/rust/sqso-fsw/src/beacon.rs`, each
reviewed against `docs/BEACON_SPEC.md` §6 before launch). Templates contain no numbers; the OBC inserts every value from
measurements. Examples of the intended classes: transponder open; transponder closed until sunrise; low power, beacon
only; thermal hold; commissioning; health summary; quiet pass greeting. Evaluated once per beacon (every 60, 120 or 180
s by mode).

### 3.2 H2: transponder bucket

OPEN or CLOSE for the next 5-minute bucket (the duty window's bucket, `transponder.rs`). CLOSE saves the duty budget and
energy for a later bucket with more expected demand. Evaluated at each bucket boundary.

### 3.3 H3: power-mode preference

STAY or PREFER_IDLE. PREFER_IDLE in ACTIVE raises the mode manager's ACTIVE → IDLE event; it is ignored in any other
mode. It never proposes ACTIVE and never acts in SAFE or EMERGENCY. Evaluated every 10 s health cycle.

### 3.4 No QSO-priority output (change to D5, for the chair)

The transponder is an analogue FM repeater (`docs/hardware/BUILD_VS_BUY.md` §3.2): it repeats any CTCSS-valid carrier
and has no per-station control. A QSO priority score has no actuator, so it is not an output. Fairness is served by H2
(opening buckets when demand is spread, rather than first-come). BASELINE §6's AI model row should drop "and QSO
priority" (exact text in the CDR report).

## 4. Inputs (features)

All features are computed by one function shared by the flight code and the training pipeline (through `sqso-py`, the
same way the content filter is shared), so training and flight use identical quantization.

| # | Feature | Source | Unit, quantization | Valid range | Stale after | Heads |
|---|---------|--------|--------------------|-------------|-------------|-------|
| F1 | State of charge | EPS telemetry (`eps_telemetry`) | ‰, i16 | 0–1000 | 2 s | all |
| F2 | SOC slope over the last 10 min | 1-min SOC ring buffer | ‰/h | ±500 | 2 min | H2, H3 |
| F3 | Sunlit | EPS telemetry | 0/1 | — | 2 s | all |
| F4 | Time since the last eclipse/sunlit transition | transition timestamps | s | 0–6000 | — (needs one transition since boot) | all |
| F5 | Measured orbit period | transitions | s | 5500–5800 | — | H2 |
| F6 | Battery temperature | sensor BT | 0.1 °C | −200–600 | 10 s | H1, H3 |
| F7 | PA temperature | ADC `TX_TEMP` | 0.1 °C | −200–800 | 2 s | H1, H2 |
| F8 | Transponder keyed share of the duty window | `Transponder::duty()` | ‰ | 0–250 | — | H2 |
| F9 | Keyed share in the last 5 min (demand served now) | forward-power record | ‰ | 0–1000 | — | H2 |
| F10 | Keyed share in the same orbit-phase bucket, 15 orbits ago (ground track ≈ 1 day earlier) | 15 × 20-bucket history, persisted in FRAM | ‰ | 0–1000 | 2 days | H2 |
| F11 | UTC hour | time set by telecommand or GNSS (`software/flight/docs/TIME_MANAGEMENT.md`) | h | 0–23 | missing until set | H2 |
| F12 | Operating mode | mode manager | enum | ACTIVE, IDLE | — | H1, H3 |
| F13 | Jetson texts cached | executive | count | 0–BATCH_LINES | — | H1 |
| F14 | Beacons since each template class was last sent (the oldest) | beacon history | count | 0–255 | — | H1 |
| F15 | Fault-log entries in the last 24 h | fault log | count | 0–100 | — | H1, H3 |
| F16 | Days since separation | mission data | d | 0–1100 | — | H1 |

F9 and F10 measure demand without a position: the transponder keys only on a CTCSS-valid uplink, and an SSO ground
track nearly repeats daily. When the CTCSS decoder's `TONE_OK` readback is wired (RID AV-05), F9 and F10 use it instead,
which also counts demand while the transponder is closed. A missing required feature makes the head abstain.

## 5. Hard limits outside the model

| Decision | Rule (unchanged code) | How the proposal combines |
|----------|-----------------------|---------------------------|
| Transponder | `XPDR_EN` heartbeat only while permitted ∧ ACTIVE ∧ sunlit ∧ SOC ≥ 55 % ∧ PA cool ∧ duty < 25 % (`transponder.rs`) | heartbeat = gates ∧ ¬(H2 = CLOSE). The selector can close an open bucket, never open a gated one |
| Power mode | SOC thresholds and events (`mode.rs`, `state_machine.rs`) | H3 can raise ACTIVE → IDLE only |
| Beacon text | Pre-approved template table; values from measurements; content filter on everything sent | H1 chooses the template index; an index outside the table is a fault (template 0) |
| Emissions | TX-OFF, RF silence, loss-of-contact inhibit, downlink duty and VSWR limits | Not inputs to the selector's authority at all |
| Payload | `payload_guard` limits; the Jetson is never woken by the selector | — |

A property test (§9) checks over random inputs and random model tables that the combined output is never less
conservative than the rules.

## 6. Model

### 6.1 Form

Gradient-boosted decision trees, evaluated in integer arithmetic so the host and the target give bit-identical results:

| Head | Classes | Trees | Depth | Leaf values |
|------|---------|-------|-------|-------------|
| H1 | K ≤ 16, one-vs-rest | 12 per class | 3 | i16 logit, Q8 |
| H2 | 2 | 32 | 4 | i16 logit, Q8 |
| H3 | 2 | 32 | 4 | i16 logit, Q8 |

A node is a feature index (u8), a threshold (i16, the feature's quantization) and child offsets (u8, u8): 5 bytes; a
leaf 2 bytes. Traversal is a fixed-depth loop, no recursion.

### 6.2 Budget on the STM32L4R5

| Item | Value | Basis |
|------|-------|-------|
| Model table (flash, `.rodata`) | H1 16 × 12 × (7 × 5 + 8 × 2) = 9.8 kB; H2, H3 32 × (15 × 5 + 16 × 2) = 3.4 kB each; header, ranges, calibration, CRC 0.2 kB: **≈ 17 kB** | 0.8 % of 2 MB flash; the release text is 106 kB of the 256 kB SRS-C004 limit, so the selector code and table fit with 120 kB to spare |
| RAM | features 32 B, logits 64 B, F10 history 15 × 20 × 2 B = 600 B (persisted), state 64 B: **< 1 kB** | of 640 kB; live state goes in SRAM2 with the rest (RID AV-07) |
| Stack | < 128 B (no recursion) | inside the 8 KB SRS-C003 budget (worst case today 5,444 B, `scripts/stack_usage.py`) |
| CPU | H1 192 trees × 3 nodes + H2, H3 64 × 4 nodes = 832 node visits ≈ 10 cycles each: **≈ 8,300 cycles = 0.10 ms at 80 MHz** | at most three heads in one 20 ms pass: 0.5 % of the pass |
| Energy | 0.10 ms at ≈ 30 mW (80 MHz) ≈ **3 µJ a decision** | SQSO-AUD-006 §2 (45 µJ ROM) |

### 6.3 Calibration and abstention

- Binary heads: p = σ(logit / T), with the temperature T from a held-out calibration split, as a 256-entry Q15 table.
  ABSTAIN if p ∈ (1 − τ, τ); τ is chosen on validation so that accepted decisions are ≥ 98 % in agreement with the
  oracle (default τ = 0.80).
- H1: ABSTAIN if the top two logits differ by less than a margin m chosen the same way.
- ABSTAIN also if any feature is outside the [min, max] seen in training (stored per feature in the table), stale, or
  missing.
- The table carries a version, a schema hash of the feature list (§4) and a CRC-32. If any check fails at boot the
  selector is disabled for the boot and a fault is logged.

### 6.4 Telemetry: decisions and abstentions per head

The mission success criteria count the selector's decisions and abstentions (SQSO-MSC-001 §7; RID SMA-15). The
selector shall count, per head (H1 beacon template, H2 transponder bucket, H3 power-mode preference), each evaluation
that produced a decision and each that abstained (§6.3: the fixed rules decided), since boot, and the flight software
shall report them with the table state in HK2 field `SL` (SRS-F170):

```
SL=<table in use 1/0>/<H1 decisions>/<H1 abstentions>/<H2 decisions>/<H2 abstentions>/<H3 decisions>/<H3 abstentions>
```

The counters and the field exist in the flight software now (`software/rust/sqso-fsw/src/selector.rs`,
`housekeeping.rs`; HK2 stays within its frame split, `docs/operations/TELEMETRY_DICTIONARY.md` §4.4.1): until the
selector is implemented (§11) the table is absent and every count is 0. The selector's implementation feeds them
(`Stats::record` at each head's evaluation); the stored five-minute records keep the ground's view across passes
through the HK2 frames each pass returns. Verification: the HK2 format by unit test now; the counts with the
selector (§9, the abstention tests, and the mission SIL comparing `SL` with the logged decisions).

## 7. Training

1. **Data.** At least 200 runs of the mission simulation (`sqso_sil --scenario year`) with random seeds and the
   configurable failures and environments of `software/rust/sqso-sil/src/config.rs` (wings stowed, ADCS failed, 525 km,
   LTAN spread, aged array, high and low demand from `software/rust/sqso-sil/src/qso.rs`). Each decision instant logs the
   features through the shared builder.
2. **Labels.** H2 and H3: an oracle that sees the whole orbit in hindsight chooses the buckets that maximize the uplink
   demand served subject to the energy and duty limits and to ending the orbit with SOC ≥ the start value. H1: the
   template the PI-approved rule table assigns to the hindsight state (for example "eclipse within 10 min"); Jev labels
   may be added for tone only after TypeSafe's written terms (SQSO-AUD-006 §1.3).
3. **Fitting.** XGBoost or LightGBM constrained to §6.1, then quantized to the integer table. Train, calibration and test
   splits by run, never by sample (instants of one run are correlated).
4. **Export.** The table, the feature ranges and a vector file of at least 1,000 feature/logit/decision triples for the
   bit-exact test (§9).

## 8. Validation (ground, before the table is accepted)

| Metric | Set | Pass |
|--------|-----|------|
| Expected calibration error | held-out runs | ≤ 0.05 per binary head |
| Agreement with the oracle on accepted decisions | held-out runs | ≥ 98 % |
| Abstention rate | held-out runs | ≤ 25 % |
| Abstention rate | out-of-distribution runs (failures, 525 km, demand ×3) | ≥ 2× the in-distribution rate (the range guard works) |
| Mission result | full mission SIL with the selector against fixed rules, same seeds | no check of `sqso_sil` worse; demand served ≥ the fixed rules; lowest SOC not lower by more than 2 points |
| Energy and time | target, HIL | ≤ 0.2 ms and ≤ 10 µJ a decision |

## 9. Verification of the flight code

| Test | What it shows | Requirement |
|------|---------------|-------------|
| Bit-exact vectors (Rust `no_std` against the Python reference, shared vector file, as the content filter does) | flight inference equals the trained model | SRS-F103 |
| Property test: random features and random valid tables; combined decision never less conservative than the rules | §5 holds whatever the model says | SRS-F104 |
| Abstention: stale, missing and out-of-range features; corrupt table (CRC, version, schema) | §6.3 | SRS-F105, SRS-F106 |
| Fuzzing of the table parser (`software/rust/fuzz`) | no panic, bounded time for any table | SRS-F106 |
| Mission SIL `commissioning` and `full` with the selector enabled | 0 anomalies | SRS-F107 |
| Timing on the target (loop timing probe, `software/rust/firmware/src/timing.rs`) | §6.2 | SRS-P001 |
| Coverage | Class C line and branch gates of the SDP | SDP §3.3 |

Safety classification: not safety-critical under NASA-STD-8739.8B, because no selector output can enable an emission, a
load or a mode the rules forbid (§5, shown by the property test); the hazard analysis entry is SW-HAZ-012 in
`docs/safety/SOFTWARE_HAZARD_ANALYSIS.md` §9.4.

## 10. Failure behaviour

| Failure | Effect | Response |
|---------|--------|----------|
| Table corrupt or incompatible at boot | Selector disabled | Fixed rules; fault logged; beacon flag |
| Feature stale or out of range | Head abstains | Fixed rules for that decision |
| Model wrong but confident | A worse template or a closed bucket | Bounded: cannot exceed the rules (§5); ground can disable by telecommand (`SET_SELECTOR`, to be added to the command dictionary) |
| Selector code fault (panic) | Reset as for any module | Watchdog and assertion handler; the selector restarts from the table |
| Upset in the table | CRC check at boot; flash ECC in use | Disabled until the next boot |

The model is data in the image. The flight baseline has no model upload; a future upload would be data only, adopted by
authenticated telecommand from a designated station (47 CFR 97.211; BASELINE §2).

## 11. Implementation phase

Implementing is a **fabrication-phase (post-CDR) task**, not small enough to do safely in the CDR cycle: it needs the
shared feature builder in `sqso-fsw` and `sqso-py`, the F10 history in FRAM, the `SET_SELECTOR` telecommand, the
training pipeline over ≥ 200 SIL-years, and the §9 tests. Dependencies: the UTC source (F11), and preferably the
`TONE_OK` readback (RID AV-05). Estimate: about 600 lines of Rust and 400 of Python, plus the SIL compute. The design
here is the CDR deliverable.

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| A | 2026-10-27 | SMART-QSO Team (CDR avionics and flight-software review) | Initial issue: selector and transponder-scheduler detailed design (inputs, outputs, model form, abstention, hard limits, budget, training, validation, verification, failure behaviour) |
| B | 2026-10-27 | SMART-QSO Team (CDR flight software, round 3, fifth part) | §6.4 added: decisions and abstentions per head in HK2 `SL` (SRS-F170, RID SMA-15 reopened) |
