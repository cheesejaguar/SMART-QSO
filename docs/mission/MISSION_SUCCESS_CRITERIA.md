# SMART-QSO Mission Success Criteria

**Document ID**: SQSO-MSC-001
**Version**: 2.5
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR draft for PI approval. Rewritten to the baseline (BASELINE Rev D §2, §6, decision D5); replaces
Version 1.0, whose Level 2 gate ("AI payload operational, 50+ AI-generated beacons") conflicted with D5

## 1. Purpose and Rules

These criteria say, before launch, what the mission must do to be called a minimum (L1), full (L2) or extended (L3)
success, and how each criterion is measured. They are used for the reliability estimate
(`docs/environment/RELIABILITY_ANALYSIS.md` v2.0 §5) and, after launch, for the success declarations.

1. **The primary objective is transponder service** (BASELINE §2: a single-channel U/V FM transponder, duty-gated by
   power). The AX.25 telemetry beacon is the secondary objective. The onboard AI is an **experiment**: the flight
   baseline is a calibrated selector on the OBC choosing among pre-approved templates and scheduling the transponder;
   Jetson-generated beacon text is a gated experiment that may never be enabled (D5).
2. **The Jetson experiment is not a gate for any level.** It has its own criteria (§6).
3. **Every criterion is measurable** from spacecraft telemetry, ground-station logs (project stations and public
   networks such as SatNOGS) or operator logs, with the source named. A criterion whose measurement does not yet exist
   in the telemetry was marked **[TLM]**; every such field now exists in HK1 or HK2, and the criteria name it (§7).
4. Mission time t is counted from deployment from the dispenser.
5. Levels are cumulative: L2 requires L1, L3 requires L2.

## 2. Mission Objectives

| ID | Objective | Baseline source |
|----|-----------|-----------------|
| MO-1 | Provide a U/V FM transponder for worldwide amateur contacts, duty-gated by available power | BASELINE §2; MRD-030–034 |
| MO-2 | Transmit a 1200 bps AX.25 telemetry beacon in published formats | BASELINE §2; MRD-020–025 |
| MO-3 | Demonstrate an onboard AI (OBC selector) that schedules the transponder within the power budget and selects beacon templates, with hard limits enforced outside the model and fallback to a conventional transponder | BASELINE §2, §6; MRD-040–045 |
| MO-4 | Operate in compliance with Part 97 and the debris rules, including TX-OFF and end-of-mission passivation | BASELINE §5, §7; MRD-080–093 |
| E-1 | Experiment: Jetson-generated beacon text, enabled only when measured energy per accepted beacon line ≤ 0.1 Wh (360 J), counted on accepted lines over ≥ 30 EM bursts (decision C-20) | BASELINE §6 (D5) |

## 3. Level 1: Minimum Success (by t = 30 days)

| ID | Criterion | Threshold | Measurement |
|----|-----------|-----------|-------------|
| L1-01 | Beacon received and decoded | Valid AX.25 frames from KK6GDB-11 decoded by ≥ 2 independent ground stations within 7 days | Ground logs (project station, SatNOGS observation IDs) |
| L1-02 | Telecommand link | ≥ 1 authenticated command accepted, including one TX-OFF / TX-ON cycle from a designated station, confirmed by the carrier dropping and the PA-switch state in telemetry | HK2 `CA` (executed / refused / failed authentication), HK1 `CN`, `TX`; `RF` field; ground recording |
| L1-03 | Bus health | Over days 8–30: no unrecovered fault; minimum SOC ≥ 40 % (the ACTIVE→IDLE threshold); pack temperature inside 0–45 °C while charging | `SOC`, `M` fields; HK1 `BOOT`, `RST`, HK2 `WD`; HK1 `TB` |
| L1-04 | Deployment and attitude | Both antennas deployed (deploy sense); detumble to < 0.5°/s within 3 orbits of the ADCS start (MRD-060 proposal) | HK1 `DEP` (AntS elements and wing sense) and `W` (body rates) |
| L1-05 | First transponder service | ≥ 1 pass in which an uplinked FM signal is heard repeated on the downlink, recorded by a ground station | Ground recording; HK2 `XE` (repeater keyed time) |
| L1-06 | Beacon availability | Beacons decoded on ≥ 80 % of the SatNOGS-scheduled passes with a successful observation of another satellite on the same pass (days 8–30) | SatNOGS observation database |

## 4. Level 2: Full Success (by t = 12 months)

| ID | Criterion | Threshold | Measurement |
|----|-----------|-----------|-------------|
| L2-01 | All of Level 1 | — | L1 declaration |
| L2-02 | Transponder service delivered | Mean transponder keyed time ≥ 30 min a day over months 2–12 (the full mission simulation predicts about 53 min a day keyed, SQSO-SIL-001 G §7; the battery-size and charge-voltage trade runs gave 56–82 min a day, `docs/hardware/BUILD_VS_BUY.md` §7.3, §7.5) and enabled in ≥ 80 % of the sunlit windows the power gate allows | HK2 `XE` (enables, keyed time), HK1 `XP`; SIL replay of the power gate on the downlinked SOC |
| L2-03 | Users served | ≥ 100 distinct call signs heard through the transponder over the year | Ground recordings and public log submissions, de-duplicated by the project |
| L2-04 | Fair scheduling (MO-3) | Gini coefficient across regions of available transponder minutes ≤ 0.35 over any 60 days, and no higher than a fixed schedule with the same energy (MRD-043 proposal) | Transponder on/off telemetry and orbit (MRD-043 method) |
| L2-05 | AI selector operating within hard limits (MO-3) | Selector in control ≥ 9 of 12 months; **zero** hard-limit violations (Jetson outside sunlight or SOC < 55 %, burst past 10 min or 3.6 kJ (decision C-20), SOC thresholds, TX-OFF overridden) | HK2 `SL` (selector decisions and abstentions; the selector is designed, not yet implemented, BASELINE §6), HK2 `HL` (hard-limit violations) |
| L2-06 | Fallback demonstrated | ≥ 1 commanded or natural fallback to the conventional transponder and template beacon, and return, without loss of service | HK1 `FC`, `FL`; the 60 s records around each mode change (`DUMP_TLM`) |
| L2-07 | Power model validated | Measured orbit-average generation within ± 20 % of the prediction for the measured attitude | `SUN`; HK1 `PA`, HK2 `IA` (array current) |
| L2-08 | Regulatory operation (MO-4) | Every TX-OFF commanded executed within one pass; no unresolved harmful-interference report | Operator log; FCC/IARU correspondence |

## 5. Level 3: Extended Success (by t = 24 months, the design life)

| ID | Criterion | Threshold | Measurement |
|----|-----------|-----------|-------------|
| L3-01 | All of Level 2 | — | L2 declaration |
| L3-02 | Continued transponder service | Mean keyed time ≥ 30 min a day over months 13–24 | As L2-02 |
| L3-03 | Users served | ≥ 300 distinct call signs over two years | As L2-03 |
| L3-04 | Data release | Two-year telemetry and AI-selector decision log published | Project archive |
| L3-05 | End-of-mission passivation commanded and verified (MO-4) | Charge-path disconnect confirmed by zero charge current in sunlight; transmitters off | Last telemetry |

L3 also needs an orbital lifetime above two years: in the baseline 465–485 km deployment band (BASELINE Rev E §3) the
shortest life, with a strong solar cycle 26 and the largest area, is 2.6 years at 465 km and 3.4 years at the 475 km
target at the 1.49 kg CBE (`docs/regulatory/filings/ODAR_SMART-QSO.md` Draft D §6.2). On the ISS backup (400–420 km,
0.7–1.9 years) L3 is not achievable (BASELINE §3).

## 6. Experiment Criteria (not a success-level gate)

| ID | Criterion | Threshold | Measurement |
|----|-----------|-----------|-------------|
| E-01 | Jetson burst within limits | ≥ 1 burst completed inside the OBC's current, temperature, 10-minute and 3.6 kJ limits (decision C-20) | HK2 `JB`, `JP`, `HL` |
| E-02 | Energy per accepted beacon line measured | Total burst energy at the carrier input, boot to power-off, over the lines the content filter accepted, over ≥ 10 bursts of the flight definition (one power-on, ≤ 10 min, ≤ 3.6 kJ; decision C-20) | HK2 `JB` (burst energy, accepted lines) |
| E-03 | Generated text flown (only if E-02 ≤ 0.1 Wh and the ground filter sweep passes) | ≥ 50 accepted generated beacons transmitted, zero filter escapes found by the ground re-check | Beacon archive; ground filter |

If E-02 exceeds 0.1 Wh, the experiment ends with a negative result, which is a valid outcome.

## 7. Telemetry Needed by These Criteria

The beacon format (`docs/BEACON_SPEC.md` §4) carries `T`, `M`, `SOC`, `BV`, `SUN`, `RF`. The other values the criteria
need are in the housekeeping frames (`docs/operations/TELEMETRY_DICTIONARY.md` §4.4, §4.4.1; RID SMA-15): command
counters HK2 `CA` and HK1 `CN`; resets HK1 `BOOT`, `RST` and watchdog resets HK2 `WD`; pack temperature HK1 `TB`;
deployment HK1 `DEP`; body rate HK1 `W`; transponder HK1 `XP` and HK2 `XE`; array power and current HK1 `PA`, HK2 `IA`;
selector HK2 `SL`; hard-limit violations HK2 `HL`; Jetson bursts HK2 `JB` and `JP`; stored records by `DUMP_TLM`.
The selector counters stay empty until the selector is implemented (BASELINE §6).

## 8. Probability of Success (estimate)

From the reliability block diagram and Monte Carlo of `docs/environment/RELIABILITY_ANALYSIS.md` v2.1 §4–§5
(`software/simulation/cdr/sma/reliability_model.py`, 400,000 trials), launch included at 0.99:

| Level | Estimate (allocation case) | With early-failure probabilities halved by the burn-in and day-in-the-life program | Empirical CubeSat survival (two-Weibull fit, launch excluded) |
|-------|---------------------------|-------------------------------------------------------------------------------------|-------------------------------------------------------------|
| L1 (30 days) | **0.75** | 0.86 | 0.74 |
| L2 (1 year) | **0.61** | 0.71 | 0.60 |
| L3 (2 years) | **0.51** | 0.59 | 0.56 |

These are lower than Version 1.0's reliability goals (0.95 / 0.80 / 0.60 in RELIABILITY_ANALYSIS v1.2 §2.1), which were
not supported by any analysis. A Class D payload may accept them; the PI decides by 2026-12-15 (RIDs SMA-14, REL-01;
`mission/SCHEDULE.md` DA-02). The halved case needs the reliability programme of `docs/environment/ENVIRONMENTAL_TEST_PLAN.md`
Rev D §3.11 (VI-11 ≥ 500 h on the EM boards, VI-12 ≥ 200 h FM burn-in).

## 9. Evaluation and Declaration

| Milestone | Evaluation | Declared by |
|-----------|------------|-------------|
| t + 30 days | L1 | PI, with the licensee (KK6GDB) |
| t + 12 months | L2 | PI |
| t + 24 months | L3 | PI |
| End of mission | Final report including E-1 | PI |

A level is met when every criterion of it is met; a level with some criteria unmet is reported as "partial", listing
them. The data behind each criterion (telemetry extracts, ground-log queries, recordings) is archived with the
declaration.

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial issue |
| 2.0 | 2026-10-27 | CDR S&MA review | Rewritten to the baseline: transponder service is the primary objective; L2 no longer gated on AI-generated beacons (D5); the Jetson text experiment has separate criteria; every criterion has a measurement source; telemetry gaps listed; success probabilities from the reliability model |
| 2.1 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | §8 from RELIABILITY_ANALYSIS v2.1 (four-line wing deployment): L2 0.61, L3 0.51 (were 0.60, 0.50); halved-early-failure case 0.71 / 0.59 |
| 2.2 | 2026-10-27 | CDR round 3, systems and programmatics (REL-01) | §8: the PI's acceptance dated (DA-02, 2026-12-15); the reliability programme cited as planned (ETP D §3.11) |
| 2.3 | 2026-10-27 | CDR round 3, flight software (decision C-20, CR-2026-018) | E-02: energy per accepted beacon line over the accepted lines of ≥ 10 bursts of the flight burst definition (≤ 10 min, ≤ 3.6 kJ), from HK2 `JB` (was a p95 per beacon) |
| 2.4 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026): the **[TLM]** gaps closed by HK1/HK2 fields (§3–§7); burst limits 10 min and 3.6 kJ (C-20; were 120 s) in L2-05 and E-01; E-1 per accepted line (was p95); §5 shortest life from ODAR Draft D §6.2 in the baseline band (was Draft B, 2.4 yr) |
| 2.5 | 2026-10-27 | Documentation audit | Documentation audit (post-CDR 2026), follow-up: L2-02 prediction from the full mission simulation, about 53 min a day keyed (SQSO-SIL-001 G §7); the 56–82 min a day of the build-versus-buy trade runs kept as the range of those runs |
