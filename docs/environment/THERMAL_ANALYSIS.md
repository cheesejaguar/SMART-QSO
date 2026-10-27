# SMART-QSO Thermal Analysis

**Document ID**: SQSO-THA-001
**Version**: 2.0
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR analysis (uncorrelated model). Replaces version 1.0, which analysed a 400 km ISS orbit and an Apollo4
OBC, inverted the β cases and asserted results without a model (finding TB-07).

`docs/BASELINE_PARAMETERS.md` governs every value: orbit (§3), thermal cases and the ≥ 10 °C model-uncertainty
margin (§8), Jetson heat path (§6), battery and heater (§5; `docs/hardware/BUILD_VS_BUY.md` §4). This document is
the CDR thermal analysis behind them. Nothing here has been measured; every result is a prediction of an
uncorrelated lumped model and carries the 10 °C uncertainty margin until the engineering-model thermal balance test
(§9).

## 1. Summary

| Item | Result | Verdict |
|------|--------|---------|
| Hot case (BASELINE §8: no eclipse, 1414 W/m², maximum power) | Internal nodes 28–39 °C, body panels ≤ 38 °C, wings ≤ 75 °C | Pass against SYS-600 (+50 °C) with ≥ 1 °C after the 10 °C margin; Jetson 31 °C below its 70 °C gate |
| Hottest real orbit (LTAN 21:30, β 40°, January) | Wings −33 … +83 °C | Wing cells need a qualified range of at least −56 … +93 °C (RID THM-05) |
| Cold case (BASELINE §8: β = 0, cold environment, SAFE) | Battery held at +1.3 °C minimum by the heater at 0.72 W (12.0 V), duty 43 %; OBC and RF boards −11.6 °C | Battery passes the GEVS heater-duty rule; boards **fail** SYS-600 (−20 °C) by 1.6 °C after the 10 °C margin, pass their part ratings (−40 °C) |
| Battery heater, set points +2/+6 °C | Battery minimum +0.1 °C in the simulated commissioning (+0.4 °C in a year at LTAN 23:30 without the radiation term), −1.0 °C if the mount is 0.10 W/K | Too little cold margin; raise to +5/+9 °C (RID THM-02) |
| Wings stowed, β = 0, sun-pointed | Heater at 100 % duty, battery +0.8 °C (0.95 W) | Contingency: positive, no duty margin (RID THM-03) |
| Jetson 8 W bursts (120 s, and 600 s batches) | Node ≤ 39 °C at 0.44 W/K, ≤ 35 °C at 1.0 W/K | The ≥ 1 W/K path is not driving at these duties; stuck-on 8 W gives 61 °C (0.44 W/K) |
| Dispenser cold soak (SpaceX bounding −20 °C) | Battery −10 °C after 2 h, −16 °C after 3 h; first charge ≈ 35 min after deployment with the 1 W heater | Below the cell's −10 °C discharge limit if deployment is later than 2 h (RID THM-06, EPS-08) |
| Dispenser hot soak (SpaceX bounding +69 °C) | Battery +57 °C after 2 h, +65 °C after 3 h; boards +66 °C | **Exceeds** the battery's +60 °C limit after about 2.3 h and SYS-601 (+60 °C) for the boards (RID THM-07) |

## 2. Model

The analysis uses the mission simulation's 12-node network (`software/rust/sqso-sil/src/env/thermal.rs`): the six
body panels with their share of the frame, the two wings, the battery pack, the OBC/EPS/ADCS stack, the RF board with
the PA, and the Jetson module with its spreader. Heat capacities come from the mass budget; surface properties from
`hardware/structure/THERMAL_DESIGN.md` §3.1; solar, albedo and Earth-IR loads use plate-to-sphere view factors; the
solar array model (`env/solar.rs`) takes out the electrical power. The cases are an ignored test beside the network
(`cdr_thermal_cases`), run and tabulated by `software/simulation/cdr/power_thermal/thermal_cases.py`.

### 2.1 Changes made to the model for this analysis

| Change | Why | Effect |
|--------|-----|--------|
| Battery radiation to the panels' inner faces, σ ε_eff A with A = 0.0169 m² (≈ 75 × 70 × 22 mm pack) and ε_eff from the pack's ε (0.10, the specified low-ε finish) and the FR-4 panels (0.85) | The brief and BUILD_VS_BUY §4.4 item 3: the battery node had no radiative link | +0.008 W/K at 0 °C with ε 0.10; +0.07 W/K for a black pack. The 14-day commissioning run's lowest battery temperature falls from +0.5 °C to +0.1 °C, still 0 anomalies |
| `ThermalParams` (battery mount, battery ε, Jetson path) | Sensitivity cases | Defaults unchanged: 0.05 W/K mount, 0.44 W/K Jetson path |
| Albedo and Earth IR per case; a radiative sink temperature and a rail conductance | Hot and cold environments; the dispenser soak | Defaults unchanged (0.30, 237 W/m², deep space) |

The mission simulation keeps a 0.5 W heater (`mission.rs`) and 0.05 W/K of mount conduction, both conservative for
the cold case against the design (1 W heater, ≈ 0.03 W/K mount; BUILD_VS_BUY §4).

### 2.2 Environment

| Parameter | Hot | Cold | Nominal (simulation) | Basis |
|-----------|-----|------|----------------------|-------|
| Solar flux | 1414 W/m² | 1322 W/m² | 1316–1414 W/m² by date | Perihelion / aphelion |
| Albedo | 0.35 | 0.25 | 0.30 | Typical high-inclination LEO design values (Gilmore, *Spacecraft Thermal Control Handbook*, ch. 2) |
| Earth IR | 258 W/m² | 218 W/m² | 237 W/m² | as above |
| Orbit | 500 km SSO. Bounding hot: dawn–dusk (β ≈ 74°, no eclipse), the BASELINE §8 definition. Real hot: LTAN 21:30 in January (β 40°, 32 min eclipse) | β ≈ 0 (36.0 min eclipse); real cold: LTAN 23:30 in June (β 6°) | LTAN 22:30, β 16–26° | BASELINE §3, §8; LTAN ±1 h from the launch provider |

The baseline orbit (LTAN 22:30 ± 1 h) never reaches the BASELINE §8 hot case: β stays between 2° and 41° over a year
(`POWER_BUDGET_ANALYSIS.md` §3), and every orbit has an eclipse of 32–36 min. The no-eclipse hot case is therefore a
bound, not a predicted condition; both are run.

### 2.3 Dissipation by case (W)

From the simulation's device values: OBC 0.165 W, ADCS sensors 0.12 W, magnetorquers ≤ 0.30 W, receiver 0.15 W, PA
2.7 W DC at 1 W RF and 1.5 W DC at 0.5 W RF, GNSS 0.125 W, EPS quiescent 0.08 W, bucks 85 %/90 %.

| Case | OBC/EPS stack | RF board | Jetson | Battery |
|------|---------------|----------|--------|---------|
| ACTIVE hot (bounding): ADCS, GNSS on, PA at 1 W RF keyed 25 % of sunlight plus the 60 s beacon, Jetson 8 W in 120 s bursts every 1200 s (10 %), or one 600 s batch per orbit | 0.74 + 5 V buck losses | 0.15 + 1.7 W × keyed fraction | 8 W while on | I²R, heater off |
| SAFE (ADCS off), beacon every 180 s at 0.5 W RF | 0.27 | 0.16 | 0 | heater: thermostat |
| EMERGENCY (beacon shed) | 0.27 | 0.15 | 0 | heater: thermostat |
| LEOP (detumble at full dipole) | 0.77 | 0.15 | 0 | heater: thermostat |

Heater: ≈ 207 Ω (1 W at 14.4 V): 0.72 W at the 12.0 V floor, 0.95 W at 14.0 V (the EMERGENCY threshold, 15 % SOC),
1.30 W at 16.4 V. GEVS (GSFC-STD-7000B §2.6.4.1) asks for < 70 % duty in the worst cold case at the minimum voltage.

## 3. Allowable temperatures and margin rule

| Item | Operating (°C) | Survival / non-operating (°C) | Source | Notes |
|------|----------------|-------------------------------|--------|-------|
| Battery (INR18650-35E) | Charge 0 … +45; discharge −10 … +60 | −20 … +60 | Cell specification (`hardware/eps/BATTERY_DESIGN.md` §2.1) | Heater-controlled at the cold end |
| OBC/EPS stack | −20 … +50 (SYS-600) | −40 … +60 (SYS-601) | SYS-600/601 | Parts: STM32L4R5ZIT6 −40 … +85; BQ76920 −40 … +85; M-G370PDT0 −40 … +85 |
| RF board, PA | −20 … +50 (SYS-600) | −40 … +60 | SYS-600/601 | The flight software stops the transponder at a PA temperature of 70 °C |
| Jetson module and spreader | −20 … +50 (SYS-600); −25 … +105 junction | −40 … +60 | SYS-600; NVIDIA DS-11105-001 v1.7 (Tj −25 … +105 °C since v1.3) | The flight software's guard stops at 70 °C on the spreader and 95 °C junction (`payload_guard.rs`) |
| Body panels, frame | −40 … +80 | −60 … +100 | THERMAL_DESIGN §1.1 | |
| Wings (cell assemblies) | −40 … +85 | −100 … +100 | SOLAR_ARRAY_DESIGN §1.1 | To be replaced by the panel vendor's qualification range |

**Margin rule.** A prediction passes when it is at least 10 °C inside the operating limit (BASELINE §8, before EM
correlation). After correlation GSFC practice keeps 5 °C between the worst-case predictions and the allowable flight
temperatures (GSFC-STD-7000B §2.6.4.1; GSFC-STD-1000 Rev I rule 4.25), and the protoflight test goes 10 °C beyond
them (GSFC-STD-7000B §2.6.1). A heater-controlled item is judged instead by its temperature staying above its limit
with the heater at < 70 % duty in the worst cold case at the minimum voltage (§2.6.4.1).

## 4. Results

Last orbit of each case after 25 orbits (40 for the stuck-load cases); minimum … maximum.

| Case | Battery | OBC/EPS | RF | Jetson | Body | Wings | Heater duty |
|------|---------|---------|----|--------|------|-------|-------------|
| H1 Hot bounding, 120 s bursts, Jetson path 0.44 W/K | 28.1 … 28.7 | 28.7 … 31.2 | 28.9 … 31.2 | 28.0 … 33.3 | 22.6 … 38.0 | 67.2 … 75.1 | 0 % |
| H2 Hot bounding, 600 s batch, 0.44 W/K | 27.6 … 28.9 | 27.5 … 31.9 | 27.9 … 32.2 | 26.1 … **39.0** | 21.6 … 39.0 | 63.8 … 75.2 | 0 % |
| H3 Hot bounding, 600 s batch, 1.0 W/K | 27.6 … 28.9 | 27.4 … 31.7 | 27.8 … 32.1 | 25.8 … 35.4 | 22.0 … 38.2 | 63.9 … 75.1 | 0 % |
| H5 Hot real (LTAN 21:30, January, β 40°) | 8.5 … 15.6 | −0.3 … 24.7 | −0.9 … 24.9 | −0.3 … 27.1 | −7.1 … 32.3 | −32.8 … **82.9** | 0 % |
| H6 Hot bounding, black battery (ε 0.85) | 27.7 … 28.8 | 28.8 … 31.1 | 28.9 … 31.1 | 28.0 … 33.3 | 22.7 … 37.8 | 67.2 … 75.1 | 0 % |
| H7 Hot bounding, Jetson stuck on at 8 W, 0.44 W/K | 43.4 … 44.0 | 45.8 … 48.1 | 44.2 … 46.5 | 58.4 … **60.6** | 38.1 … 51.5 | 62.6 … 67.1 | 0 % |
| H8 as H7, 1.0 W/K | 43.2 … 43.8 | 45.1 … 47.4 | 44.0 … 46.3 | 49.3 … 51.8 | 38.3 … 50.9 | 62.6 … 67.1 | 0 % |
| H9 Hot bounding, heater stuck on at 1.37 W, no cutout | 47.0 … **47.7** | 31.3 … 33.7 | 31.1 … 33.4 | 29.9 … 35.4 | 24.8 … 39.8 | 65.6 … 72.6 | 100 % |
| C1 Cold, SAFE, tumbling, 1.0 W | 2.1 … 9.9 | −8.9 … 19.4 | −9.0 … 19.6 | −8.9 … 19.0 | −14.6 … 27.4 | −40.8 … 37.5 | 0 % |
| C4 Cold, SAFE, spin-pointed, 1.0 W | 1.9 … 8.4 | **−11.6** … 13.0 | −11.7 … 13.1 | −11.8 … 12.5 | −16.6 … 22.0 | −39.2 … 78.7 | 35 % |
| C11 as C4, 0.72 W (12.0 V) | 1.3 … 8.1 | −11.7 … 12.9 | −11.8 … 13.0 | −11.8 … 12.3 | −16.7 … 21.9 | −39.2 … 78.6 | 43 % |
| C12 as C4, 0.5 W (the simulation's heater) | 0.6 … 7.5 | −11.8 … 12.7 | −11.9 … 12.9 | −12.0 … 12.2 | −16.8 … 21.8 | −39.3 … 79.1 | 51 % |
| C13 Cold, EMERGENCY, spin-pointed, 0.72 W | 1.3 … 8.1 | −11.7 … 12.9 | −11.9 … 13.0 | −11.9 … 12.3 | −16.7 … 21.9 | −39.2 … 78.6 | 43 % |
| C9 Cold, spin-pointed, heater failed off | **−3.1** … 3.7 | −12.4 … 12.2 | −12.5 … 12.4 | −12.5 … 11.8 | −17.4 … 21.3 | −39.5 … 80.0 | — |
| C15 as C11, mount 0.10 W/K (as-built twice the estimate) | **−1.0** … 8.8 | −11.1 … 12.7 | −11.2 … 12.8 | −11.2 … 12.1 | −16.0 … 21.6 | −39.0 … 78.6 | 51 % |
| C7 Cold, tumbling, black battery, 0.72 W | 1.7 … 13.9 | −7.9 … 19.3 | −8.0 … 19.4 | −7.9 … 18.7 | −13.5 … 27.1 | −40.4 … 37.1 | 28 % |
| C14 Cold, spin-pointed, wings stowed, 0.72 W | **−2.6** … 2.2 | −18.3 … −0.6 | **−18.6** … −0.7 | −18.7 … −1.3 | −22.7 … 7.2 | −31.6 … −14.6 | 100 % |
| C16 as C14, 0.95 W (14.0 V) | 0.8 … 5.5 | −17.6 … −0.2 | −18.0 … −0.4 | −18.2 … −1.0 | −22.0 … 7.3 | −31.1 … −14.3 | 100 % |
| C10 Cold real (LTAN 23:30, June, β 6°), tumbling, 0.72 W | 3.4 … 11.3 | −7.7 … 20.9 | −7.8 … 21.2 | −7.7 … 20.5 | −13.5 … 27.6 | −39.9 … 46.9 | 0 % |
| C22 Cold, spin-pointed, 0.95 W, **set points +5/+9 °C** | 4.0 … 10.2 | −11.2 … 13.3 | −11.4 … 13.4 | −11.4 … 12.7 | −16.3 … 22.3 | −39.1 … 78.2 | 48 % |
| C23 as C22, mount 0.03 W/K (the design) | 4.9 … 9.9 | −11.6 … 13.3 | −11.8 … 13.4 | −11.8 … 12.7 | −16.8 … 22.3 | −39.2 … 78.1 | 40 % |
| C21 as C22, mount 0.10 W/K | 1.2 … 10.5 | −10.6 … 13.2 | −10.7 … 13.2 | −10.8 … 12.6 | −15.5 … 22.0 | −38.8 … 78.2 | 59 % |
| C20 as C22, wings stowed | 0.8 … 5.5 | −17.6 … −0.2 | −18.0 … −0.4 | −18.2 … −1.0 | −22.0 … 7.3 | −31.1 … −14.3 | 100 % |

**Dispenser soak, powered off** (§6): the dispenser wall follows the SpaceX bounding interface temperature (UG v11
Table 4-11: hot 40 → 69 °C by 3000 s, cold −5 → −20 °C by 3000 s, held), ε_eff 0.70 to the panels and 0.5 W/K through
the rails; start at the pad environment (Table 4-12: 15–30 °C).

| Soak | Time after lift-off | Battery | OBC | RF | Jetson | +X panel |
|------|---------------------|---------|-----|----|--------|----------|
| Hot (start 25 °C) | 1 h / 2 h / 3 h | 38.3 / **56.8** / **64.7** | 54.9 / 66.4 / 68.2 | 55.1 / 66.6 / 68.3 | 54.6 / 66.6 / 68.3 | 58.8 / 67.1 / 68.4 |
| Cold (start 15 °C) | 1 h / 2 h / 3 h | 2.6 / **−10.0** / **−15.9** | −10.2 / −17.4 / −19.1 | −10.5 / −17.6 / −19.2 | −10.2 / −17.6 / −19.2 | −12.5 / −18.0 / −19.3 |

**Deployment after the 2 h cold soak** (L1, L2; β = 0, tip-off 10°/s, detumble at full dipole): with the 1 W heater
the battery rises from −10.0 °C to 0 °C in 35 min and reaches +3.5 °C by the end of the first orbit; with 0.5 W it is
still at −2.0 °C after one orbit. OBC −17.4 °C, wings −45.4 °C. The mission simulation with the whole spacecraft at
−20 °C at separation (`--set storage_temp_c=-20`, flight software in the loop) deployed both antennas and wings on
time; the battery stayed below −10 °C for 52 min, first rose above 0 °C after 1.35 h (1.8 h below 0 °C in all,
2.4 h at LTAN 23:30), and the OBC reached −23.0 °C. From −15 °C: 40 min below −10 °C, 1.1 h below 0 °C. The same
run from +55 °C (a hot soak) kept the pack above the 45 °C charge limit for 0.7 h; the bus stayed up in every case
(lowest pack voltage 14.65 V during the 1.6 A burn).

## 5. Margins

| Item | Worst prediction | Limit | Margin | After 10 °C | Verdict |
|------|------------------|-------|--------|-------------|---------|
| Battery, hot, operations | 28.9 °C (H2) | +45 (charge) | +16.1 | +6.1 | Pass |
| Battery, hot, Jetson stuck on | 44.0 °C (H7) | +45 (charge) | +1.0 | −9.0 | Charge inhibited hot (fail-safe); +16 to the +60 °C discharge limit |
| Battery, hot, heater stuck on | 47.7 °C (H9) | +45 / cutout +40 | — | — | The series hardware cutout at +40 °C is required (BUILD_VS_BUY §4.3) and must be verified |
| Battery, cold, heater-controlled | +1.3 °C at 0.72 W, 43 % duty (C11) | 0 °C (charge) | +1.3 | duty 43 % < 70 % | Pass by the duty rule; **fails** with mount 0.10 W/K (−1.0 °C, C15) or heater failed (−3.1 °C, C9) |
| Battery, cold, set points +5/+9 °C | +4.0 °C at 0.95 W, 48 % (C22); +1.2 °C at 0.10 W/K (C21) | 0 °C | +4.0 / +1.2 | duty 48 % / 59 % | Pass |
| OBC/EPS stack, cold | −11.6 °C (C4); −17.4 °C (LEOP after soak) | −20 (SYS-600) | +8.4 / +2.6 | **−1.6 / −7.4** | Fails SYS-600; +18 / +13 to the parts' −40 °C |
| RF board, cold, wings stowed | −18.6 °C (C14) | −20 | +1.4 | **−8.6** | Fails SYS-600 in the contingency |
| OBC/EPS stack, hot | 31.9 °C (H2) | +50 | +18.1 | +8.1 | Pass |
| Jetson, hot, bursts and batches | 39.0 °C (H2) | +50 (SYS-600); 70 (guard) | +11.0 / +31.0 | +1.0 / +21.0 | Pass |
| Jetson, stuck on | 60.6 °C (H7) | +50 | −10.6 | −20.6 | Failure case; the energy loss ends it first (POWER_BUDGET_ANALYSIS §8.4) |
| Body panels | −22.7 … +51.5 °C | −40 … +80 | +17.3 / +28.5 | +7.3 / +18.5 | Pass |
| Wings | −45.4 … +82.9 °C | −40 … +85 | −5.4 / +2.1 | **−15.4 / −7.9** | Needs the vendor's qualification range (RID THM-05) |
| Battery, dispenser hot soak at 2 h | 56.8 °C | +60 | +3.2 | −6.8 | **Fails** at 2.3 h (bounding environment; RID THM-07) |
| Boards, dispenser hot soak at 2 h | 66.6 °C | +60 (SYS-601) | −6.6 | −16.6 | **Fails** (bounding) |
| Battery, dispenser cold soak at 2 h | −10.0 °C | −20 survival; −10 discharge | +10.0 / 0.0 | 0.0 / −10.0 | Survives; at the discharge limit at deployment |

## 6. Discussion

### 6.1 Battery heater and set points

The isolated mount works: in the hot case the pack follows the frame mean (28 °C) and in the cold case its time
constant (≈ 250 J/K / 0.08 W/K ≈ 50 min) carries it through the eclipse. But the flight software's thermostat (on
below +2 °C, off above +6 °C, `sqso_fsw::heater`) leaves 2 °C between the switch-on point and the 0 °C charge limit,
and the pack keeps cooling for several minutes after the heater comes on. The mission simulation's 14-day
commissioning with the battery radiation added reaches +0.08 °C; the cold case with an as-built mount of 0.10 W/K
(BUILD_VS_BUY §4.5 allows for it) reaches −1.0 °C even with the heater at 51 % duty; the 1 W heater cycles with an
overshoot that can leave the pack lower than a weaker heater would.

**Recommendation (RID THM-02, flight software, for the chair):** `heater::ON_BELOW_C` 2.0 → **5.0**,
`OFF_ABOVE_C` 6.0 → **9.0**. With 0.95 W (14.0 V) the minimum is then +4.0 °C nominal, +4.9 °C at the design mount
and +1.2 °C at 0.10 W/K, at 40–59 % duty. The energy cost is ≈ 0.1 W orbit average (POWER_BUDGET_ANALYSIS §6). The
hardware cutout stays at +40 °C.

### 6.2 Heater failure cases

Failed off (C9): the pack reaches −3.1 °C in the cold case, so the charge regulator refuses charge at sunrise until
the pack warms (fail-safe: no lithium plating), but SYS-602 is lost for part of each orbit near β = 0. Failed on
(H9): 1.37 W at a full pack raises it to 47.7 °C in the hot case, above the 45 °C charge limit; the series +40 °C
hardware cutout (BUILD_VS_BUY §4.3) is the control and must be in the heater circuit drawing and its test
(BATTERY_SAFETY_ASSESSMENT §4).

### 6.3 Jetson heat path

At the BASELINE duty (120 s bursts at 8 W, 10 % of the orbit) the module and spreader node rises 5 K per burst at
0.44 W/K and 4 K at 1.0 W/K: the node's 220 J/K, not the path, sets the burst rise. A 600 s batch (which the payload
schedule of the simulation allows, SIL-01) gives 39.0 °C at 0.44 W/K and 35.4 °C at 1.0 W/K. TB-07's "77 °C for a
10-minute burst" assumed a +55 °C structure (version 1.0's unsupported hot case); the model's hot-case structure is
22–39 °C. The ≥ 1 W/K requirement (BASELINE §6, SYS-603) is therefore not driving at the baseline duty; it limits
a stuck-on module to 52 °C instead of 61 °C (H7, H8). The junction is above the node by 8 W times the
junction-to-spreader resistance, which nobody has measured: the guard's 95 °C junction limit allows
(95 − 39 − 10) / 8 = 5.7 K/W. Keep the 1 W/K requirement (it costs little and protects the stuck-on case) and add
the junction-to-spreader measurement to the EM test (§9, item V-THM-3).

The flight software has no lower temperature limit for powering the Jetson. The node reaches −12 °C in the cold
case and −18 °C in LEOP after a cold soak; the module's junction minimum is −25 °C. Recommendation (THM-08, for the
chair): no Jetson power-on below −15 °C on the spreader sensor.

### 6.4 PA

The RF board stays below 32 °C in the hot case with the PA at 1 W RF and the transponder keyed 25 % of the sunlit
time (1.7 W dissipated while keyed). The board node is not the PA junction: the PA's case-to-board resistance and
the 70 °C gate in the flight software (`sqso_fsw::transponder`) must be checked on the EM with the PA keyed for 120 s
(the hardware key-down timer, BUILD_VS_BUY §3.2).

### 6.5 Battery radiation and surface finish

With the specified low-ε exterior (ε 0.10) the radiation adds 0.008 W/K. A black pack (ε 0.85) adds 0.07 W/K: in the
cold case that cuts the heater's margin like a 0.10 W/K mount does (C7: +1.7 °C at 28 % duty, but the hot side of
the cycle rises to 14 °C). The finish is a drawing requirement (BUILD_VS_BUY §4.3) and must be inspected.

### 6.6 Cold boards and SYS-600

SYS-600 ("all components shall operate within −20 … +50 °C") cannot be met with the 10 °C margin by the OBC/EPS
stack and the RF board in the cold case (−11.6 °C) or after a cold soak (−17.4 °C; −23.0 °C in the simulation with
the whole spacecraft at −20 °C at separation). The parts are rated −40 … +85 °C. Recommendation (THM-04, for the
chair): replace SYS-600/601 by per-unit allowable flight temperatures in a thermal requirements table
(THERMAL_DESIGN §1.1): OBC/EPS and RF boards −30 … +60 °C operating, −40 … +70 °C non-operating (part ratings
−40 … +85 °C with 10 °C derating); battery as §3; Jetson −15 … +50 °C operating (power-on gate −15 °C);
wings per the vendor's qualification.

### 6.7 Dispenser soak

SpaceX gives a bounding conductive interface temperature at its side of the dispenser mount (UG v11 §4.1.11,
Table 4-11), not a dispenser or CubeSat temperature, and states it does not envelope payload temperatures. Treating
it as the dispenser wall with strong coupling is a bound. In that bound the pack exceeds +60 °C 2.3 h after lift-off
and the boards exceed SYS-601's +60 °C; in the cold bound the pack is at its −10 °C discharge limit at 2 h. The
isolated mount is what keeps the pack 10 K behind the boards. What closes this is the integrator's dispenser thermal
analysis for the assigned Transporter flight and slot, with the deployment time (RID THM-07, verification item V-THM-5).
Until then: deliver the pack at the storage state of charge (BATTERY_SAFETY_ASSESSMENT §6), and do not give the BMS a
discharge cut-off at −10 °C (EPS-08), or a cold deployment would leave the bus dark until the Sun warms the pack.

## 7. Thermal control hardware (CDR baseline)

| Element | Specification | Source |
|---------|---------------|--------|
| External finishes | Long faces: cells (α 0.92, ε 0.85) and black anodize between; ends black anodize (α 0.88, ε 0.85) | THERMAL_DESIGN §3.1 |
| Battery mount | 4 × M2.5 G-10CR or PEEK 450G shouldered bushings, Ti-6Al-4V screws, flying leads ≥ 100 mm, ≈ 0.03 W/K to the frame, ≤ 0.01 W/K to the stack | BUILD_VS_BUY §4.3 |
| Battery exterior | ε ≤ 0.10 (VDA tape or bare aluminium) | BUILD_VS_BUY §4.3; §6.5 here |
| Battery heater | ≈ 200 Ω polyimide film, two elements on different cells, ≥ 0.95 W at 14.0 V; thermostat +5/+9 °C (recommended); series hardware cutout +40 °C; fuse or current limit | §6.1; BATTERY_DESIGN §6.3 |
| Jetson path | ≥ 1 W/K module to frame, measured on the EM | BASELINE §6; SYS-603 |
| Temperature sensors | Two battery NTCs, OBC, RF/PA (TX_TEMP), Jetson spreader, one per long panel (the panel vendors build thermistors in) | THERMAL_DESIGN §10 |

## 8. RIDs raised by this analysis

The full RID table is in the CDR reviewer report; the thermal RIDs closed or opened here:

| RID | Title | Status |
|-----|-------|--------|
| THM-01 | Thermal analysis v1.0 was for the wrong orbit and computer, with no model | Closed by analysis (this document) |
| THM-02 | Heater thermostat +2/+6 °C leaves too little margin to the 0 °C charge limit | For the chair: flight-software constants (§6.1) |
| THM-03 | Wings-stowed cold case saturates the heater | Verification item V-THM-1; accepted as contingency |
| THM-04 | SYS-600/601 cannot be met by the boards with the 10 °C margin | For the chair (§6.6) |
| THM-05 | Wing cells' qualified range not known; predictions −45 … +83 °C | Verification item V-THM-4 |
| THM-06 | Cold soak in the dispenser puts the pack at its discharge limit at deployment | Verification item V-THM-5 with EPS-08 |
| THM-07 | Hot soak in the dispenser exceeds +60 °C (bounding) | Verification item V-THM-5 |
| THM-08 | No minimum temperature for Jetson power-on | For the chair (§6.3) |
| THM-09 | Battery node had no radiative coupling | Closed by model change (§2.1) |

## 9. Verification items

| ID | Test | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-THM-1 | EM thermal balance in vacuum, hot and cold cases of §4 with the heater power stepped, wings deployed and stowed | Model correlated within ±3 °C on the battery, OBC, RF and Jetson nodes; then the 10 °C margin drops to 5 °C (GSFC-STD-7000B §2.6.4.1) | THM-01, THM-03, SYS-602 | EM, before FM build |
| V-THM-2 | Battery mount conductance and pack finish | G to frame 0.02–0.05 W/K (heater steps), ε ≤ 0.10 by inspection | BUILD_VS_BUY §4.3 | EM thermal balance |
| V-THM-3 | Jetson path and junction-to-spreader resistance at 8 W | Module-to-frame ≥ 1 W/K; junction rise ≤ 5.7 K/W | SYS-603, BASELINE §6 | EM |
| V-THM-4 | Wing and body panel qualification record from the vendor | Qualified range covers −56 … +93 °C (predictions with 10 °C) for ≥ 11,000 cycles | THM-05 | Panel procurement |
| V-THM-5 | Integrator's dispenser thermal analysis for the assigned flight, with the deployment time | Pack ≤ +50 °C and ≥ −15 °C at deployment; boards within non-operating limits | THM-06, THM-07 | Integrator ICD (by L-6 months) |
| V-THM-6 | Protoflight thermal vacuum and cycling | SpaceX UG v11 Table 6-2: acceptance range (envelope of the predictions and −24 … +61 °C) ± 5 °C, 20 cycles, ≥ 4 in vacuum, tailored on the battery to its −20 … +60 °C limits with the integrator's agreement | SYS-600/601 or their replacement | FM |
| V-THM-7 | Heater circuit: thermostat, cutout at +40 °C, fuse | Cutout opens at 40 ± 3 °C with the heater forced on; heater current at 12.0 and 16.4 V | §6.2 | EM, FM |

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR review, EPS/thermal | Rewritten for the baseline orbit and hardware on the simulation's 12-node network (battery radiation added): hot, cold, dispenser and LEOP cases with margins (finding TB-07) |
