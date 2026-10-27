# Battery Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-EPS-BAT-001
**Version:** 2.8
**Date:** 2026-10-27
**Status:** Battery definition and cycle-life analysis for the baseline in `docs/BASELINE_PARAMETERS.md` §5 (finding TB-08). **Analysis, not measured; no cell has been bought or tested.**

---

## 1. Overview

This document defines the battery pack of the SMART-QSO 1U CubeSat and analyses its cycle life. BASELINE §5 governs:
4S1P Li-ion 18650, ≈ 3.5 Ah, 14.4 V nominal, ≈ 50 Wh; depth of discharge (DoD) ≤ 10 % per eclipse nominal and ≤ 30 %
contingency (SYS-112); charging only between 0 °C and +45 °C (SYS-602); cell over-voltage protection ≤ 4.25 V; permanent
charge-path disconnect at end of life.

### 1.1 Corrections to Version 1.0

| Version 1.0 | Correction |
|-------------|------------|
| 2S1P, 7.2 V, 2,800 mAh, 20.2 Wh (Molicel P28A) | 4S1P, 14.4 V, ≈ 50 Wh (BASELINE §5) with a 3.45 Ah cell (§2) |
| Cycle life "> 500 cycles to 80 %", unsupported, against about 5,600 eclipses a year | Cycle-life analysis from the cell's published data (§3.3) |
| Charging allowed from −10 °C (BMS) and −20 °C (cell table) | Charging only from 0 °C to +45 °C, the cell specification's window and BASELINE §5 |
| Passivation "controlled discharge to 50 % SOC" | Permanent, non-resettable charge-path disconnect (BASELINE §5, 47 CFR 97.207(g)(1)(iii)) |
| High-temperature cutoff +65 °C | Above the cell's +60 °C discharge limit; corrected to +60 °C (§4.1) |
| Over-current trip 10 A | Above the cell's 8 A continuous limit; corrected (§4.1) |
| Software timer listed as a launch inhibit | A timer is not an inhibit (BASELINE §4) |

---

## 2. Cell Selection

### 2.1 Cell Specification

Candidate cell: **Samsung SDI INR18650-35E** (named in `docs/hardware/EPS_DESIGN.md`; it matches the ≈ 3.5 Ah in
BASELINE §5). Values from the Samsung SDI product specification as published by distributors:

| Parameter | Value |
|-----------|-------|
| Chemistry | Lithium-ion |
| Form factor | 18650 cylindrical |
| Capacity | 3,450 mAh typical, 3,350 mAh minimum (0.2C) |
| Nominal voltage | 3.60 V |
| Charge | CC-CV to 4.20 V |
| Maximum continuous discharge | 8 A |
| Operating temperature (cell surface) | Charge 0 °C to +45 °C; discharge −10 °C to +60 °C |
| Mass | ≤ 50.0 g |
| Cycle life (specification criterion) | ≥ 2,010 mAh (60 % of the 3,350 mAh minimum) after 500 cycles, each a full charge at 1.02 A CC-CV to 4.2 V and a full discharge at 3.4 A (≈ 1C) to 2.65 V, at room temperature. These are the specification's Version 1.0 test conditions; Version 1.1 revised the cycle-life condition, so confirm against the current Samsung SDI specification |

**Cell choice is open.** Other documents name different cells: Panasonic NCR18650GA
(`docs/manufacturing/EEE_PARTS_LIST.md`), LG INR18650-MJ1 (`docs/hardware/PARTS_LIST.md`,
`docs/project/HARDWARE_READINESS_TRACKER.md`) and Molicel P28A (Version 1.0). This analysis uses the 35E; it must be
repeated if another cell is chosen.

### 2.2 Selected Cell

| Parameter | Value |
|-----------|-------|
| Model | Samsung INR18650-35E (candidate; PI to confirm) |
| Nominal Capacity | 3,450 mAh (12.4 Wh at 3.6 V) |
| Max Continuous Discharge | 8 A |
| Mass | ≤ 50.0 g per cell |
| Screening | Capacity, internal resistance and OCV matching across the four flight cells (§9.1) |

---

## 3. Pack Configuration

### 3.1 Configuration

| Parameter | Value | Basis |
|-----------|-------|-------|
| Configuration | 4S1P (4 series, 1 parallel) | BASELINE §5 |
| Nominal Voltage | 14.4 V | 4 × 3.60 V |
| Voltage Range | 12.0 V (3.0 V/cell, operational floor) to 16.4 V (4.10 V/cell, the charge set point; the cell's maximum is 4.20 V) | Charge set point by decision D8 (BASELINE §5); floor set above the 2.65 V cut-off **[floor is a design choice]** |
| Capacity | 3.45 Ah typical, **49.7 Wh** (48.2 Wh at the 3,350 mAh minimum) | 4 × 3.45 Ah × 3.6 V |
| Total Mass | 249 g CBE (cells 200 g, holder 34 g, interconnect board 15 g) | `docs/environment/MASS_BUDGET.md` rows 7–9 |

### 3.2 Energy Budget

Orbit at 500 km: period 94.6 min, eclipse 35.8 min at β = 0 (the longest), 5,559 orbits a year
(`hardware/eps/SOLAR_ARRAY_DESIGN.md` §9.2). Loads are the BASELINE §5 allocations. The battery-to-load conversion
efficiency is taken as 0.90 **[assumption]**.

| Case | Load | Duration | Battery energy | DoD (49.7 Wh) |
|------|------|----------|----------------|---------------|
| Eclipse, nominal: housekeeping 0.70 W + GNSS 0.05 W + receiver 0.15 W + beacon 0.10 W (transponder and Jetson are sunlit-only) | 1.00 W | 35.8 min | 1.00 × 35.8/60 / 0.90 = **0.663 Wh** | **1.33 %** |
| Eclipse, bounding: as above plus the 0.5 W battery heater on for the whole eclipse | 1.50 W | 35.8 min | **0.994 Wh** | **2.00 %** |
| Jetson burst in sunlight, sun-pointed: 8 W Jetson + 1.5 W transponder + 1.0 W base, less 5.76 W array | 4.74 W deficit | 120 s | 0.176 Wh | 0.35 % |
| Jetson burst in sunlight, tumbling: same, less 3.43 W mean sunlit array | 7.07 W deficit | 120 s | 0.262 Wh | 0.53 % |
| **Nominal limit (BASELINE §5)** | | | 4.97 Wh | 10 % |

The eclipse DoD is 1.3–2.0 %, a fifth of the 10 % nominal limit or less. Jetson bursts add shallow discharges in sunlight
that the array recharges in the same sunlit period. Charge current is limited by the array to at most 5.76 W / 14.4 V =
0.40 A (≈ C/9); the eclipse discharge current is about 0.08 A (≈ C/45).

### 3.3 Cycle-Life Analysis

**Cycles.** At 500 km the battery sees one eclipse per orbit, **5,559 a year and 11,118 in two years**, assuming an
eclipse every orbit (conservative: some SSO local times have eclipse-free seasons). This is the cycle count Version 1.0
compared with "> 500 cycles".

**Cell data available.** The only cycle data found for the 35E is the specification criterion in §2.1: 500 full-depth,
≈ 1C cycles at room temperature leave at least 60 % of the minimum capacity. No published shallow-DoD or LEO-profile
cycling data for this cell was found. **Assumption:** degradation per unit of charge throughput in SMART-QSO's shallow,
slow cycles is no worse than in the specification's full-depth 1C cycles, so cycles are counted as equivalent full
cycles (EFC = discharge energy / pack energy). No credit is taken for the benefit usually expected from shallow cycling.
The assumption must be confirmed by test (§9.3).

| Case | Discharge per orbit | EFC per year | EFC in 2 years | Against 500 EFC (specification point) |
|------|---------------------|--------------|----------------|---------------------------------------|
| Nominal: 1.00 W eclipse only | 0.663 Wh | 0.663 × 5,559 / 49.7 = 74 | 148 | 3.4× |
| Bounding: 1.50 W eclipse plus sun-pointed Jetson bursts at the 10 % duty limit (4.7 bursts per orbit × 0.176 Wh) with the transponder on during every burst | 0.994 + 0.831 = 1.825 Wh | 204 | 408 | 1.2× |

**Capacity needed at end of life.** The nominal 10 % DoD limit is met as long as the pack holds at least 0.994 Wh / 0.10
= **9.9 Wh, 20 % of beginning-of-life capacity**, in the bounding eclipse case. At the specification's end point
(2,010 mAh per cell, 28.9 Wh) the bounding eclipse DoD would be 3.4 %. Cycle count is therefore not what limits this
battery: even in the bounding case the two-year throughput is below the specification's 500 cycles, and the pack could
lose most of its capacity before the DoD limit is reached.

**What the analysis does not cover.**

1. **Calendar aging.** In LEO the pack is recharged to near full every orbit, so it spends the mission at high state of
   charge. No cell-specific calendar-aging data was found, so this fade is **not quantified**. A lower end-of-charge
   voltage would reduce it at the cost of capacity; that is an EPS design choice for the PI (§10).
2. **Temperature.** The specification point is at room temperature. Predicted battery temperatures come from the thermal
   model (TB-07); charging is inhibited outside 0–45 °C in any case (§4.1).
3. **No redundancy.** With one string of four cells, an open cell ends battery operation (single point of failure SPF-001
   in `docs/environment/RELIABILITY_ANALYSIS.md`). Cell screening and matching (§9.1) are the mitigation.

**Comparison with earlier figures.** RELIABILITY_ANALYSIS Version 1.0 claimed "> 10,000 cycles at 15 % DoD, 1.8×
margin"; 10,000 / 11,320 two-year cycles is 0.88×, and no source for the 10,000 was given. POWER_BUDGET_ANALYSIS
Version 1.0 allowed 70 % usable DoD and "> 500 cycles at 70 % DoD", which contradicts SYS-112 (≤ 30 %) and the
BASELINE §5 nominal 10 %. Both documents now cite this section.

---

## 4. Battery Management System (BMS)

### 4.1 BMS Features

| Feature | Specification | Basis |
|---------|---------------|-------|
| Cell Balancing | Passive, resistor-based | Version 1.0 |
| Balance Current | ≈ 9 mA, one cell at a time (BQ76952 internal balancing through the 2 × 214 Ω tap loop: 3 × 68.1 Ω current-limiting chain per tap on the sense board, 10 Ω at the AFE) | SQSO-EPS-BRD-001 D §6.1 |
| Balance Threshold | 50 mV | Version 1.0 |
| Overcharge Protection | ≤ 4.25 V per cell | BASELINE §5 |
| Overdischarge Protection | 2.8 V per cell | Above the 2.65 V cut-off of the cell specification's cycle test |
| Overcurrent Protection | 4 A for 0.5 s; short-circuit trip ≤ 200 µs | Below the 5 A pack fuse (the resettable trip acts first) and the cell's 8 A continuous limit; largest legitimate load 1.40 A, largest software-fault load 2.0 A (all four burn drivers on, held by the 7.0 V burn buck's current limit; SQSO-EPS-BRD-001 §6.1, §9) |
| Secondary over-voltage protector | 4.35 V/cell, fixed threshold, blows a three-terminal fuse in the charge path | Second, dissimilar overcharge control (SQSO-BSA-001 §3, H1; RID EPS-11) |
| Charge temperature window | Charge inhibited below 0 °C and above +45 °C, in hardware at the charge regulator's TS input | Cell specification; BASELINE §5; SYS-602 |
| Pack over-temperature | Both FETs open above +60 °C (hardware comparator) | Cell specification |
| Low-temperature discharge | **No cut-off.** Discharge below −10 °C is outside the cell specification for performance, not a safety hazard; a cut-off at −10 °C would keep the bus dark after a cold deployment (the pack is at −10 °C after a 2 h cold soak, −16 °C after 3 h; SQSO-THA-001 v2.0 §6.7). The flight software limits the burn current instead (RID EPS-08) | CDR review |

### 4.2 BMS IC Selection

- Primary: **TI BQ76952** (thresholds in OTP, high-side CHG/DSG FETs, coulomb counter, SUV permanent fail at 2.00 V that
  sets the latched recharge lockout), on the EPS board (SQSO-EPS-BRD-001 §5.4, §6.1)
- Secondary over-voltage: **TI BQ771806** (4.35 V fixed) blowing a Littelfuse ITV4030L1412NR three-terminal fuse in the pack line
- Hardware comparators on NTC1: charge window +1.1…+44.0 °C, heater cutout +40 °C, pack over-temperature +60 °C
- Fuel gauge: TI INA228 coulomb counter on the OBC's I2C (0x44)

### 4.3 Charge Control

| Mode | Current | Condition |
|------|---------|-----------|
| Precharge | 100 mA | Vcell < 3.0 V |
| CC Charge | ≤ 0.40 A (array-limited, ≈ C/9) | Vcell < 4.10 V and cell temperature 0 to +45 °C |
| CV Charge | Tapering | Vcell = 4.10 V (D8) |
| Termination | 50 mA | End of charge |
| Inhibit | 0 A | Any cell outside 0 to +45 °C, or over-voltage |

---

## 5. Safety Features

### 5.1 Hardware Protections

| Protection | Implementation |
|------------|----------------|
| Fuse | On the sense board: **F0A** (FM12A72V5.0A, AEM, MIL-PRF-23419/12, 1000 A interrupt) and **F0B** (FM08A125V5A, Littelfuse, MIL-PRF-23419/8, 300 A at 125 VDC: a diverse part) in series in BAT_TOP at the cell-4 positive, and **F0N** (FM12A72V5.0A) in BAT_BOT at the cell-1 negative (the grounded negative leg); the EPS pack fuse F2 (FM12A72V5.0A) behind them; all non-resettable. Cell taps: 3 × 68.1 Ω current-limiting chain and an FM12A72V3/4A fuse each. Resettable PTC RT1 on the EPS (hold 2.6 A, 1.92 A at 60 °C; not an interrupter) |
| Overcharge | Charger CV at 4.10 V/cell, BMS AFE at 4.25 V, secondary protector at 4.35 V with fuse (two-fault tolerant; SQSO-BSA-001 §3) |
| Overdischarge | BMS + hardware comparator |
| Overcurrent | PTC + electronic switch |
| Short Circuit | Electronic switch, < 10 µs response |
| Temperature | NTC sensors; charge inhibit outside 0 to +45 °C in hardware, independent of software |

### 5.2 Launch Safety (Inhibits)

BASELINE §4 requires at least three independent physical inhibits for RF and for each deployable (CDS 2.3.7, 2.3.8);
the deployment switches are physical inhibits, and the RBF pin and a software timer are not. The inhibit chains and
their independence are in `docs/hardware/EPS_DESIGN.md` §8 and `hardware/electrical/eps/EPS_BOARD_DESIGN.md` §6: three
deployment switches DS1–DS3 on three rails, DS3 also holding the trackers' enables low, so the array path has its third
inhibit (RID EPS-04). The pack's negative is switched by DS2, so both poles must be isolated from the frame (≥ 1 MΩ at 50 VDC).
If the ISS/NRCSD backup is used, the JSC 20793 Rev D requirements apply (BASELINE §3); the assessment for both paths
is `docs/environment/BATTERY_SAFETY_ASSESSMENT.md` (SQSO-BSA-001).

### 5.3 Passivation

End-of-life passivation per BASELINE §5 and 47 CFR 97.207(g)(1)(iii):
- Permanently disconnect the solar-array charge path through a non-resettable latch (fuse or latching relay)
- Disable all transmitters
- Leave the battery to discharge through the remaining loads to a permanent discharge state

The hardware is defined in `docs/hardware/EPS_DESIGN.md` v2.0 §7 (a one-shot thermal cut-off between the trackers and
BATT+, fired by two independent series switches on two authenticated commands). The flight software has no command or
pin for it yet (RID EPS-06).

---

## 6. Thermal Design

### 6.1 Temperature Limits

| Condition | Limit | Basis |
|-----------|-------|-------|
| Charge Enable | 0 °C to +45 °C | Cell specification; BASELINE §5 |
| Optimal Charge | +10 °C to +30 °C | Version 1.0 design goal |
| Discharge (specification) | −10 °C to +60 °C | Cell specification; no hardware cut-off below −10 °C (§4.1) |
| High Temp Cutoff | +60 °C | Cell specification |

### 6.2 Thermal Interface

- Isolated mount (THERMAL_DESIGN 1.1; `docs/hardware/BUILD_VS_BUY.md` §4): four G-10CR or PEEK 450G shouldered
  bushings with Ti-6Al-4V screws, a low-emissivity exterior, flying leads and no stack connector; ≈ 0.03 W/K to the
  frame and ≤ 0.01 W/K to the OBC stack. The thermal pad stays between the cells and the holder, not to the structure
- Heat spreading via aluminum cell holders
- Heater circuit to hold the cells above 0 °C so charging can resume after eclipse

### 6.3 Heater Sizing

| Parameter | Value |
|-----------|-------|
| Heater Power | ≈ 1 W at 14.4 V (≈ 207 Ω polyimide film, two elements on different cells); on the EPS: SW5 current limit 0.148 A, fuse F4 0.25 A and a series +40 °C hardware cutout (Q22 from NTC1) (SQSO-EPS-BRD-001 §10) |
| Activation Temp | Flight software today: on below +2 °C, off above +6 °C. Recommended: **+5/+9 °C** (SQSO-THA-001 v2.0 §6.1: the +2 °C point leaves the pack at +0.1 °C in the simulated commissioning and below 0 °C with a 0.10 W/K mount) |
| Power at the bus extremes | 0.72 W at 12.0 V, 0.95 W at 14.0 V, 1.30 W at 16.4 V; worst cold case duty 43–59 % (below the 70 % of GSFC-STD-7000B §2.6.4.1) except with the wings stowed (100 %) |
| Thermostat Type | NTC-based control |

---

## 7. Mechanical Packaging

### 7.1 Cell Holder Design

| Parameter | Specification |
|-----------|---------------|
| Material | Aluminum 6061-T6, two clamp plates (34 g, `docs/environment/MASS_BUDGET.md` §4.3) |
| Cell Retention | Cells bonded or potted in the holder, welded nickel interconnects (no spring contacts; RID STR-08) |
| Mounting | M2.5 Ti-6Al-4V screws through isolator bushings to frame inserts (§6.2) |
| Vibration Survival | Integrator/SpaceX levels (BASELINE §8) |

### 7.2 Wiring

- Pack leads **22 AWG M22759/33**, two wires per pole, twisted quad (W1 to EPS J1, Micro-Fit 2×4 right-angle, gold:
  power, NTC1 with its own return, heater pair; 24 AWG for NTC1 and the heater): 1.2 A per wire worst case (2.0 A
  pack current, four burn drivers on, 60/40 split) against 2.50 A derated (SQSO-HAR-001 2.0 §5.2, §9.1)
- Cell taps and NTC2 in 28 AWG on a 6-way Pico-Clasp (W1 to EPS J2, gold); NTC2 returns to the cell-1 negative tap
  inside the pack
- **All leads 0.12 m** from the sense board to the EPS bottom-side zone Z1 (service loop), **PTFE over-sleeve** on the
  pack leads (double insulation: they are upstream of the EPS fuse F2), no metal clamps, tied to the sense board within
  10 mm (SQSO-HAR-001 §7.2)
- **Sense-board protection (Version 2.5, revised 2.7 after the round-3 panel, RID P3E-02).** W1 is upstream of every
  protection on the EPS: a BAT_TOP lead shorted to BAT_BOT, or to the frame (the frame is at GND, which reaches the
  pack negative through DS2 and RS7 once released), would see the cells' short-circuit current with no EPS fuse in
  the loop, and the AFE's short-circuit detection opens FETs that are downstream of the fault. The construction
  controls (double insulation, routing, hipot) are kept but not counted, and the sense board carries:
  - **F0A and F0B** in series in BAT_TOP within 10 mm of the cell-4 positive tab, before the W1 pads, and **F0N** in
    BAT_BOT at the cell-1 negative tab (the interrupter in the grounded negative leg, JSC 20793 Rev D §5.1.3.1a):
    F0A and F0N **FM12A72V5.0A** (AEM, MIL-PRF-23419/12 solid-body fuse, 5 A, 72 VDC, short-circuit interrupt 1000 A at
    the maximum DC voltage; cold resistance 13.5–22.5 mΩ; the same part as the EPS's F2) and F0B **FM08A125V5A**
    (Littelfuse, MIL-PRF-23419/8 subminiature cartridge, 5 A, 125 V, 300 A at 125 VDC per its commercial equivalent,
    the 265 series; cold resistance 12.6–15.4 mΩ; axial, staked on the sense board): two diverse positive-leg fuses
    (Version 2.8, after SQSO-FMEA-003), so they are independent controls by design. F0B never has to interrupt more
    than ≤ 211 A, because F0A and F0N stay in every loop while intact (1.42× its rating); its rating at 16.8 V, the
    FM08 vacuum note (MIL-PRF-23419 §6.1.3) and its rerating are open for procurement (SQSO-EPS-BRD-001 §21).
    Every short loop of the pack passes all three. Carrying: 1.40 A largest legitimate pack current (35 %), 2.0 A with
    four burn drivers on (software fault, 50 %) of the 4.0 A 60 °C rerating (0.80 assumed, to confirm on the slash
    sheet; rule 50 %); 0.13 W in the three at 1.40 A. Coordination: the AFE (OCD1 3.6–4.4 A for 0.5 s, SCD 10 A in
    15 µs, on the FETs) acts first on every fault downstream of its FETs; the fuses act on faults upstream of the FETs
    (W1, J1, the EPS's F2 area) or if the AFE fails. The fuses are in series and any one opening isolates the pack for
    good, so selectivity among them is not needed. **Breaking capacity:** the prospective current of a short in W1 is
    ≤ 211 A at 16.8 V with the cells at a lower-bound 10 mΩ each (about a third of the 35E's typical DC resistance) and
    the three fuses at their minimum resistance, within the FM12's 1000 A and 1.42× the FM08's 300 A; the Nano2 0451005.MRL first chosen (50 A at 125 V, 300 A at 32 VDC) was replaced
  - **Cell taps TAP0–TAP4:** each a 0.75 A **FM12A72V3/4A** fuse (MIL-PRF-23419/12, cold resistance 0.225–0.375 Ω) and
    a current-limiting chain of three 68.1 Ω 2512 1 W thick-film resistors (the AFE's tap filter resistance, moved from
    the EPS board, which keeps 10 Ω + 0.1 µF at the AFE). A tap short in W1 is limited to ≤ 82 mA tap to frame
    (0.46 W per resistor, 46 %) and ≤ 41 mA tap to tap, so the 28 AWG tap wire (0.90 A derated) cannot heat even with
    the fuses closed; a single resistor failed short leaves 136 Ω. The EPS board adds a 1 MΩ ladder across the taps
    (SQSO-EPS-BRD-001 E, RID SMA-B03: an open tap is held between its neighbours, 4.2 µA from the pack). Balancing ≈ 9 mA (was 18 mA): the 50 mV balance
    window is unchanged and the pack's expected spread is corrected in a few charge cycles; measurement error from the
    AFE's input current ≤ 0.2 mV; a blown tap fuse or open chain shows as an open cell input (AFE open-wire check)
  - Sense board (the pack's 15 g interconnect board): FR-4, conformal coated; F0A, F0B, F0N, FT0–FT4 and the 15 tap
    resistors under the coating; the fuses' 72 VDC rating is 4.3× the 16.8 V pack (voltage rule 50 %); terminations in
    the non-tin finish of the slash sheet
- Strain relief at all connections
- Conformal coating on BMS PCB

---

## 8. Telemetry

### 8.1 Telemetry Points

| Parameter | Sensor | Resolution |
|-----------|--------|------------|
| Pack Voltage | ADC | 10 mV |
| Cell 1–4 Voltages | BMS | 10 mV |
| Pack Current | Hall sensor | 10 mA |
| Temperature 1 | NTC | 0.5°C |
| Temperature 2 | NTC | 0.5°C |
| SOC | Calculated | 1% |
| Charge Status and Charge-Inhibit Cause | BMS | Boolean / enum |

---

## 9. Testing Requirements

### 9.1 Cell Acceptance

Screening and lot acceptance follow `docs/environment/BATTERY_SAFETY_ASSESSMENT.md` §4 (JSC 20793 Rev D §4.2.3,
§6.4.3.3: 100 % visual with the wrap removed, mass, OCV retention, AC and DC resistance, capacity; ±3σ screening with
lot uniformity OCV < 1 %, mass < 2 %, capacity < 5 %, resistance < 15 %).

| Test | Requirement |
|------|-------------|
| Capacity | ≥ 3,350 mAh (the specification minimum) and within the lot's ±3σ |
| AC impedance (1 kHz) | < 25 mΩ and within the lot's ±3σ |
| OCV retention | 30 days at 50 % SOC, within the lot's ±3σ |
| Matching | Capacity within 1 %, DC resistance within 5 % across the four flight cells |

### 9.2 Pack-Level Testing

| Test | Specification |
|------|---------------|
| Capacity Verification | C/5 discharge |
| Charge/Discharge Cycling | 10 cycles |
| Protection Trip Test | All protection circuits, including the 0 °C and +45 °C charge inhibits |
| Thermal Cycling | -20°C to +50°C, 5 cycles (non-operating; no charging outside 0 to +45 °C) |
| Vibration | Protoflight levels |

### 9.3 Cycle-Life Verification (to close the §3.3 assumption)

| Test | Purpose |
|------|---------|
| LEO-profile cycling of flight-lot cells: 94.6 min cycles, 35.8 min discharge, CC-CV recharge to the flight end-of-charge voltage, at the 10 % DoD limit and at the nominal 1–2 % DoD | Measure fade per cycle at the mission profile |
| The same at the predicted cold and hot battery temperatures | Temperature dependence |
| Calendar storage at the flight end-of-charge voltage | Calendar fade (§3.3 item 1) |

---

## 10. Compliance

### 10.1 Standards

| Standard | Compliance |
|----------|------------|
| UN 38.3 | Cell summary from the manufacturer; pack tests T1–T5 and T7 on a flight-lot pack (SQSO-BSA-001 §8) |
| NASA-STD-8719.14 | Orbital debris/passivation (§5.3) |
| Launch Provider | Battery safety requirements (planned) |
| IEC 62133 | Safety requirements (planned) |
| JSC 20793 Rev D | Only if the ISS/NRCSD backup is used; assessment in SQSO-BSA-001 |
| SpaceX Rideshare Payload User's Guide v11 | §6.7.7 inhibit test at the flight state of charge (60 ± 5 %, SQSO-BSA-001 §6; LVR-051); §7.6 powered off at launch |

### 10.2 Open Decisions

- Confirm the cell part number (§2.1).
- End-of-charge voltage: **closed, 4.10 V a cell** (decision D8, BASELINE §5; BUILD_VS_BUY §7). The pack sits at the end
  of charge for most of the mission; the lower set point gives up about 10 % of capacity for slower calendar ageing.

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 2.0 | 2026-10-16 | Funding audit (TB-08): 4S1P ≈ 50 Wh pack per BASELINE §5; cell data and cycle-life analysis (§3.3); 0–45 °C charge window; permanent charge-path disconnect for passivation |
| 2.1 | 2026-10-26 | Charge set point 4.10 V a cell (D8, §3.1, §4.3, §10.2); isolated mount and 1 W heater (§6.2, §6.3, §7.1; BUILD_VS_BUY §4) |
| 2.2 | 2026-10-27 | CDR review (EPS): secondary over-voltage protector; no low-temperature discharge cut-off; pack over-temperature in hardware; fuse and PTC ratings (§4.1, §5.1); inhibits and passivation point to EPS_DESIGN v2.0 (§5.2, §5.3); heater set points +5/+9 °C recommended (§6.3); screening per SQSO-BSA-001 (§9.1); compliance (§10.1) |
| 2.3 | 2026-10-27 | CDR design (EPS board) | BMS parts selected (BQ76952, BQ771806, ITV4030 fuse, INA228 gauge; §4.1, §4.2); heater protection as built (§6.3); cell retention bonded/potted with welded interconnects (§7.1, RID STR-08); W1 wiring (§7.2); SQSO-EPS-BRD-001 |
| 2.4 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: §5.2 inhibit text, the array path has its third inhibit DS3 (SQSO-EPS-BRD-001 §6) |
| 2.5 | 2026-10-27 | CDR design (EPS board): §7.2 wiring per SQSO-HAR-001 2.0 (B-1): pack leads 22 AWG, all leads 0.12 m, PTFE over-sleeve; sense-board fuse F0 (5 A) in BAT_TOP and tap fuses FT0–FT4 (0.5 A) added, with their coordination against F2 and the BMS; §5.1 fuse row |
| 2.6 | 2026-10-27 | CDR design (EPS board): sense-board fuses changed for breaking capacity: F0 (and the EPS's F2) FM12A72V5.0A, tap fuses FT0–FT4 FM12A72V3/4A (MIL-PRF-23419/12, 1000 A interrupt) against a 314 A / 63 A prospective-current bound (§5.1, §7.2) |
| 2.7 | 2026-10-27 | CDR design (EPS board): round-3 panel (RID P3E-02): BAT_TOP fuses F0A + F0B and BAT_BOT fuse F0N (FM12A72V5.0A) on the sense board; tap current-limiting chains 3 × 68.1 Ω moved to the sense board (balancing ≈ 9 mA); software-fault pack current 2.0 A (§4.1, §5.1, §7.2) |
| 2.8 | 2026-10-27 | CDR design (EPS board): F0B a diverse part (FM08A125V5A, Littelfuse, MIL-PRF-23419/8) so the two positive-leg fuses are independent by design (SQSO-FMEA-003); the EPS's open-tap ladder noted (§5.1, §7.2) |
