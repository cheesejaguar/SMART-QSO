# SMART-QSO Reliability Analysis

**Document ID**: SQSO-REL-001
**Version**: 2.2
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR analysis. Allocations and models, not measurements: no flight hardware exists. Version 1.2's
predictions (0.98 / 0.85 / 0.72) had no analysis behind them and are withdrawn

## 1. Purpose and Basis

This document gives the reliability block diagram (RBD), the failure-rate allocation, the probability of meeting each
mission success level of `docs/mission/MISSION_SUCCESS_CRITERIA.md` v2.0, the single points of failure and the
wear-out checks for the 2-year design life.

| Basis | Use |
|-------|-----|
| NPR 8705.4B (Risk Classification for NASA Payloads, effective 2024-11-01), Class D | Risk posture: single-string design and medium-to-significant risk of not achieving success are acceptable when stated and accepted |
| NASA-STD-8729.1A (Planning, Developing and Managing an Effective Reliability and Maintainability Program) | Reliability tasks tailored for Class D: FMEA, critical items, RBD, prediction, failure reporting |
| MIL-STD-1629A | FMEA form (`docs/safety/SYSTEM_FMEA_CIL.md`, SQSO-FMEA-002) |
| Jacklin, *Small-Satellite Mission Failure Rates*, NASA/TM-2018-220034 (2019) | Empirical: of small satellites launched 2000–2016, 41.3 % failed or partially failed (24.2 % total failures, 11 % partial, 6.1 % launch failures) |
| Langer and Bouwmeester, *Reliability of CubeSats – Statistical Data, Developers' Beliefs and the Way Forward*, SSC16-X-2 (2016) | Empirical two-Weibull survival of 178 CubeSats (parameters as quoted by TU Delft follow-up work; the original paper could not be retrieved from the review environment) |
| `docs/environment/RADIATION_ANALYSIS.md` Rev C | TID and SEE rates for the baseline orbit |

Version 1.2 cited NASA-HDBK-4002A as its basis; that handbook covers spacecraft charging, not reliability.

## 2. Success Levels Used

| Level | Horizon | Needs (from MISSION_SUCCESS_CRITERIA v2.0) |
|-------|---------|--------------------------------------------|
| L1 | 30 days | Bus, receiver, transmitter and antennas working for 30 days; one transponder relay in the first week |
| L2 | 1 year | L1, plus transponder service through month 12: repeat path working, at least one wing deployed, and with one wing lost the ADCS working; heater loss tolerated in 70 % of cases |
| L3 | 2 years | L2 through month 24 |

The Jetson experiment (E-1) is not in the RBD for any level. Its failures enter only where they can take the bus down
(rail stuck on, short not isolated).

## 3. Reliability Block Diagram

```
 Launch & dispenser ─ Separation switches, power-up ─ EPS (charger, BMS, PDU) ─ Battery 4S (series) ─ Solar array (≥ 4 of 6 strings)
   ─ OBC board ─ Flight software ─ 435 MHz receiver ─ 2 m transmitter/PA ─ AntS (VHF and UHF) ─ Harness ─ Structure
   ─ Telecommand keys ─ [Jetson isolation] ─ [GNSS isolation] ─ [Operations]                    = BUS (series; L1, L2, L3)

 BUS ─ Transponder repeat path                                                                  = L1 relay, L2/L3 service
     ─ Wings: ┌ shared burn supply ┐─┬─ Wing 1 ─ (driver A ∥ driver B) ─┐                          L2/L3: at least one wing,
              │ SW6, U17, DEPLOY_EN │ └─ Wing 2 ─ (driver A ∥ driver B) ─┘                          and if only one, ADCS
              └─────────────────────┘  (1-of-2 wings, in series with the shared supply)
     ─ ADCS (needed only if one wing is lost) ─ Battery heater (loss ends service in 30 % of cases)
```

## 4. Allocation and Model

### 4.1 Model

Monte Carlo over the RBD (`software/simulation/cdr/sma/reliability_model.py`, 400,000 trials). Each block has an
early-failure probability p_inf (design, workmanship and first-flight defects, sampled at an exponential time with a
10-day mean) and a constant hazard λ afterwards (random part failures, unrecovered single-event effects, TID). Early
failures dominate CubeSat losses (Langer and Bouwmeester's mixture puts 21 % of the population in an early-failure
mode with a 58-day scale and 18.5 % dead on arrival), so the allocation is built the same way.

### 4.2 Allocation

| Block | p_inf | λ (per year) | Basis of the allocation |
|-------|-------|--------------|------------------------|
| Launch and dispenser | — | — | 0.99 success: Falcon 9 record and a flown dispenser |
| Separation switches, power-up | 0.010 | 0 | Simple mechanism; separation-switch failures are a known CubeSat DOA cause |
| EPS: charger, BMS, PDU, safety board (custom) | 0.050 | 0.030 | First build; EPS is the largest single contributor to CubeSat failures in the surveys |
| Battery pack 4S1P (series) | 0.005 | 0.020 | Screened cells, but four in series with no bypass |
| Solar array (loss of > 2 of 6 strings) | 0.002 | 0.002 | Bought panels; many strings |
| OBC board (custom STM32L4R5) | 0.040 | 0.030 | First build; SEFI and SEL rates of RADIATION_ANALYSIS Rev C §5 inside λ |
| Flight software and firmware | 0.040 | 0.020 | Class C, CI-tested, never run on hardware (R18) |
| 435 MHz receiver | 0.030 | 0.020 | Custom; COTS RF parts with no radiation data |
| 2 m transmitter and PA | 0.030 | 0.030 | Custom; PA thermal stress at transponder duty |
| ISIS AntS | 0.015 | 0 | > 500 units flown, redundant release |
| Harness, connectors, workmanship | 0.020 | 0.005 | |
| Structure (bought) | 0.001 | 0 | |
| Telecommand keys and counters | 0.005 | 0.005 | Spares and an offline recovery key (SQSO-ICD-013) |
| Jetson faults not isolated | 0.003 | 0.003 | Two series switches, OBC trip (BUILD_VS_BUY §7.3) |
| GNSS faults not isolated | 0.001 | 0.001 | Load switch with latch-up trip |
| Operations error (unrecoverable) | 0.003 | 0.005 | Two-person rule for hazardous commands |
| Transponder repeat path | 0.030 | 0.010 | Custom; flatsat end-to-end test |
| Wing burn supply shared by the four drivers | 0.002 | 0 | SW6 load switch, U17 burn buck, DEPLOY_EN line (SQSO-EPS-BRD-001 §9), and the deployment sequence in software, which a ground-commanded retry covers. Version 2.0 had one driver, arm switch and timer for both wings at 0.015 |
| Wing burn driver, each of four (PB10 → W1A, PE6 → W1B, PB9 → W2A, PE7 → W2B) | 0.005 | 0 | One-shot, op-amp, FET and shunt; a wing is lost only if both of its drivers fail |
| Wing 1, wing 2 (each) | 0.030 | 0 | Hinge and restraint release |
| ADCS | 0.030 | 0.020 | Bought sensors and rods; custom drivers; backup gyro |
| Battery heater | 0.010 | 0.010 | |

The values are this review's allocations. They are the inputs a PI can challenge; the model and its sensitivity make
the consequence of any change visible.

## 5. Results

| Case | L1 (30 d) | L2 (1 yr) | L3 (2 yr) |
|------|-----------|-----------|-----------|
| **Allocation as tabulated** | **0.75** | **0.61** | **0.51** |
| Early-failure probabilities × 0.5 (§6 test program) | 0.86 | 0.71 | 0.59 |
| Early-failure probabilities × 2 (first-build team, tests cut) | 0.57 | 0.45 | 0.37 |
| Random hazards × 2 (radiation worse than predicted) | 0.74 | 0.51 | 0.35 |
| Empirical CubeSat survival, launch excluded (two-Weibull) | 0.74 | 0.60 | 0.56 |

With the four-line wing deployment of the EPS board (Version 2.1) L2 and L3 rise by about 0.01 against Version 2.0's
0.60 and 0.50 (`reliability_model.py --design shared` reproduces those). The bottom-up estimate and the empirical CubeSat
curve agree within 0.06 at every horizon. The goals of Version 1.2
(L1 > 0.95, L2 > 0.80, L3 > 0.60) are **not met**, and the program cannot reach them with a single-string first-build
1U; even the halved-early-failure case misses L1 and L2. Recommended disposition: replace them with the estimates
above as accepted Class D risk, with the §6 program as the means to approach the halved case (RID REL-01). **PI decision
required by 2026-12-15** (`mission/SCHEDULE.md` DA-02): accept the estimates and keep the §6 programme, or drop the
programme and accept the allocation case.

**Contributors to the L2 loss** (share of trials in which the block was the first required block to fail within a
year): EPS 6.6 %, OBC 5.7 %, flight software 4.8 %, transmitter 4.7 %, receiver 4.0 %, repeat path 3.3 %, harness
2.1 %, battery 1.8 %, antennas 1.3 %, others < 1 % each. **Wings:** both lost in 0.29 % of trials (0.2 % the shared burn
supply, 0.09 % both wings independently; Version 2.0, with one driver for both wings: 1.6 %), one lost in 5.8 %. The
both-wings-lost probability is the wings-failed input of the ODAR's disposal probability (ODAR Draft D §6.4).

## 6. Means to Reduce Early Failures (the dominant term)

| Activity | Target | Effect in the model |
|----------|--------|---------------------|
| Flatsat with EM boards, flight software on the STM32L4R5, end-to-end RF and repeat path | ≥ 500 h powered before the flight build: ETP VI-11, by the CDR (2027-09-28), schedule task FLATSAT_500H | Early-failure allocations of OBC, software, radios and repeat path toward × 0.5 |
| Flight-unit burn-in | ≥ 200 h powered, the last 100 h failure-free, including 4 ambient thermal cycles: ETP VI-12, 2028-02-22 to 2028-03-03, task FM_BURNIN | EPS, harness, radios |
| Day-in-the-life tests | 72 h of the commissioning sequence, then 7 days of operations, on the flight unit | Software and operations |
| Protoflight TVAC with functional tests at every plateau | Per `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` | Workmanship |
| Deployment tests at cold and minimum voltage | VI-LV-17 | Wings and antennas |
| Separate burn driver per wing | Done in the EPS board design: one driver per resistor on four lines | Both-wings-lost 1.6 % → 0.29 % (the shared SW6/U17 supply is now the larger term) |
| Beacon kept when the receiver is known failed (D3 consultation) | Rule change | Removes the receiver from the L1/L2 beacon chain after day 30 |

## 7. Single Points of Failure

The single points and their retention rationale are in SQSO-FMEA-002 §3–§4 (CIL-05 to CIL-13). Class D accepts them;
two are recommended for removal because the cost is small (separate wing burn drivers; the receiver and loss-of-contact
coupling).

## 8. Wear-Out and Life-Limited Items

### 8.1 Battery

```
BATTERY LIFE ANALYSIS (summary of BATTERY_DESIGN §3.3; analysis, not measured)

Cell: Samsung INR18650-35E (candidate), 4S1P, 3.45 Ah, 49.7 Wh (BASELINE §5), charged to 4.10 V/cell (D8)

Cycling:
- Eclipses: 5,559 per year at 500 km; 11,118 in 2 years
- DoD per eclipse: 1.3 % nominal, 2.0 % bounding (limit 10 %, BASELINE §5); mission simulation 2.0–3.2 %
- Equivalent full cycles in 2 years: 148 nominal, 408 bounding
- Cell data: specification criterion, >= 60 % of minimum capacity after
  500 full-depth 1C cycles; no shallow-DoD data found for this cell
- Margin on charge throughput: 3.4x nominal, 1.2x bounding
  (no credit taken for shallow cycling)
- Capacity needed at end of life for the 10 % DoD limit: 20 % of BOL

Calendar ageing: the pack sits at the end of charge most of the mission; 4.10 V reduces it (D8). Not quantified for
this cell; LEO-profile cycling and calendar test of flight-lot cells (BATTERY_DESIGN §9.3, V-BSA-8)

Status: cycle count is not life-limiting by analysis; calendar ageing and the cycle test are open
```

### 8.2 Other items

| Item | Mechanism | Mission demand | Capability | Status |
|------|-----------|----------------|------------|--------|
| Solar cells | Radiation and UV degradation | 2 years | BASELINE §5 generation is end-of-life | Covered by the power budget |
| Solder joints, connectors | Thermal cycling, ≈ 11,100 orbits in 2 years | ΔT per orbit from THERMAL_ANALYSIS | No qualification data in the repository (Version 1.2's "20,000 qualification cycles" had no source) | Open: protoflight thermal cycling is a workmanship screen, not a life test; rely on IPC J-STD-001 space-addendum workmanship and the 8739.1 staking of heavy parts |
| STM32L4R5 flash | Write endurance | Housekeeping to FRAM, not flash; flash written only at software load | 10,000 cycles (ST data) | Met by design |
| FRAM | Write endurance | ≈ 1 write a minute for 2 years ≈ 10⁶ | 10¹⁴ (Infineon FM25V20A data) | Met |
| Burn-wire releases, hinges | Single use | 1 | 1 | Tested on the EM and after environments |
| TID | Dose | 5.3 krad(Si) behind 2 mm in 2 years at 500 km | 10 krad design level; margin 1.9 (RADIATION_ANALYSIS Rev C §4.2) | Open: part data (RAD-V3) |

## 9. Software Reliability

Software failure modes and controls: SQSO-FMEA-001 and `docs/safety/SOFTWARE_HAZARD_ANALYSIS.md`. In this model
software appears as one series block (p_inf 0.04, λ 0.02/yr). Version 1.2's "MISRA compliance: Yes" and "cyclomatic
complexity < 15: Yes" were not supported by any tool output (note of 2026-10-24); the flight software is Rust, and
line coverage was 94.8 % (C and Rust) on 2026-10-24.

## 10. Disposal Reliability

The probability that the spacecraft re-enters within 5 years of the end of its mission (FCC) depends on the failure-time
distribution of §5 and the deployment altitude; it is computed with the same Monte Carlo and reported in
`docs/regulatory/filings/ODAR_SMART-QSO.md` Draft B §6.4.

## 11. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | CDR baseline |
| 1.1 | 2026-10-16 | Funding audit | §6.1 battery life replaced by a summary of BATTERY_DESIGN §3.3 (TB-08); orbit cycles at 500 km; battery redundancy corrected to 4S1P |
| 1.2 | 2026-10-16 | SMART-QSO Team | §3.2 link-budget margin: the SEMP threshold and goal and the LINK_BUDGET_ANALYSIS v2.0 result replace "3 dB margin minimum" (finding TB-05). Header revision brought up to the history (it still read 1.0 after 1.1) |
| 1.2, note | 2026-10-24 | SMART-QSO Team | §9.3 note: the MISRA and complexity entries are unsupported; status for the Rust flight software |
| 2.0 | 2026-10-27 | CDR S&MA review | Rewritten: RBD, failure-rate allocation and Monte Carlo for L1/L2/L3 of MISSION_SUCCESS_CRITERIA v2.0 (0.75 / 0.60 / 0.50), empirical cross-check (Jacklin 2019; Langer and Bouwmeester 2016), sensitivity, contributors, early-failure reduction program, wear-out table; Version 1.2 predictions, the Apollo4/Si4463 parts table, the 400 km radiation table and the NASA-HDBK-4002A citation withdrawn |
| 2.1 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | Wing deployment re-modelled for the EPS board's four independent burn lines (one driver per resistor, burned one at a time) with the shared SW6/U17/DEPLOY_EN supply: both wings lost 0.29 % (was 1.6 %); L1/L2/L3 0.75 / 0.61 / 0.51 (was 0.75 / 0.60 / 0.50); RBD, allocation, §5, §6 |
| 2.2 | 2026-10-27 | CDR round 3, systems and programmatics (RIDs REL-01, P3P-03) | §5: acceptance is the PI's decision DA-02 by 2026-12-15; §6: the flatsat and burn-in targets are verification items ETP VI-11 and VI-12 with dates in SQSO-SCH-001 D. Figures unchanged |
