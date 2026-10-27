# SMART-QSO Thermal Analysis

**Document ID**: SQSO-THA-001
**Version**: 2.3
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
| Cold case (BASELINE §8: β = 0, cold environment, SAFE; 475 km, §4.1–§4.2) | Battery held at **+3.6 °C** minimum by the heater at **0.70 W** (207 Ω at the 12.0 V floor), duty 55 % (C11); OBC and RF boards −11.3 … −11.7 °C | Battery passes the GEVS heater-duty rule; boards pass the proposed per-unit allowable (−30 °C, §6.6) by 8.3 °C after the 10 °C margin; against the present SYS-600 (−20 °C) they fail by 1.3–1.7 °C |
| Battery heater ≈ 207 Ω, set points +5/+9 °C (flight software, SRS-F130; THM-02 closed) | Battery minimum **+3.6 °C at the 12.0 V floor** (0.70 W, C11), +4.4 °C at 14.0 V (0.95 W, C22); with a 0.10 W/K mount **+1.3 °C** at 14.0 V (C21) and 0.0 °C at the floor (C15) | Pass with the design mount; with an as-built 0.10 W/K mount the pack reaches the hardware charge-inhibit threshold (+1.1 °C ± 1 K, SQSO-EPS-BRD-001 §5.3) at the floor: V-THM-2 measures the mount |
| Wings stowed, β = 0, sun-pointed (C14) | Heater at 100 % duty; battery **−1.5 °C at 0.70 W** (12.0 V), ≥ 0 °C from 0.79 W (12.8 V), +2.7 °C at 0.95 W (14.0 V) | Contingency accepted (RID THM-03, §6.8): the stowed generation settles the bus at about 14.0 V, where the pack holds +2.6 °C with no energy margin; below +1.1 °C charging is refused in hardware (fail-safe) |
| LEOP after the 2 h cold soak, separation at eclipse entry, β = 0, wings stowed until the first burn (L4, §4.3) | At the first burn (separation + 30 min): wing panel −40.6 °C, hinge −30.2 °C (frame side) and −38.8 °C (wing side), burn resistor −29.4 °C; first-orbit minima OBC/EPS −27.6 °C, RF −28.6 °C, Jetson (off) −28.9 °C | Mechanisms inside the −55 °C design and test point by ≥ 4.4 °C after the 10 °C margin (3.3 °C after a 3 h soak; RID P3S-01 closed by analysis). Boards and Jetson 8.6–8.9 °C outside a −30 °C allowable after the margin and 1.1–2.4 °C inside the parts' −40 °C: first-orbit allowable −40 °C proposed with a test-margin deviation (RID THM-04, §6.6) |
| Jetson 8 W bursts (120 s, and 600 s batches) | Node ≤ 39 °C at 0.44 W/K, ≤ 35 °C at 1.0 W/K | The ≥ 1 W/K path is not driving at these duties; stuck-on 8 W gives 61 °C (0.44 W/K) |
| Dispenser cold soak (SpaceX bounding −20 °C) | Battery −9 °C after 2 h, −15 °C after 3 h; first charge ≈ 33 min after deployment with the 1 W heater | Below the cell's −10 °C discharge limit if deployment is later than 2 h (RID THM-06, EPS-08) |
| Dispenser hot soak (SpaceX bounding +69 °C) | Battery +55.5 °C after 2 h, +64 °C after 3 h; boards +66 °C | **Exceeds** the battery's +60 °C limit after about 2.4 h and SYS-601 (+60 °C) for the boards (RID THM-07) |

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
| Battery radiation to the panels' inner faces, σ ε_eff A with A = 0.0169 m² (≈ 75 × 70 × 22 mm pack) and ε_eff from the pack's ε (0.10, the specified low-ε finish) and the FR-4 panels (0.85); the conduction set to 0.042 W/K so that the model keeps its 0.05 W/K in all | BUILD_VS_BUY §4.4 item 3: the battery node had no radiative link | ≈ 0.008 W/K at 0 °C with ε 0.10; ≈ 0.07 W/K for a black pack. Added on top of 0.05 W/K it put the 14-day commissioning over the < 1 % charge-refusal check (1.6 %); split as above the run reports 0 anomalies, lowest battery +0.07 °C |
| `ThermalParams` (battery mount, battery ε, Jetson path) | Sensitivity cases | Defaults unchanged: 0.05 W/K mount, 0.44 W/K Jetson path |
| Albedo and Earth IR per case; a radiative sink temperature and a rail conductance | Hot and cold environments; the dispenser soak | Defaults unchanged (0.30, 237 W/m², deep space) |
| (Version 2.2) Wings stowed until the first burn, the separation phase swept over the orbit, and sub-nodes for each wing's hinge (frame side, wing side), burn board and hold-down line, one-way coupled to the network (§4.3) | RID P3S-01: the hinge, line and wing temperatures at the burns; L1–L3 deploy the wings at separation | Run by an analysis harness on the network's public interface (`sqso_sil::env::thermal`, unchanged); the harness reproduces C11 and C14 of §4.1 to 0.1 °C. Folding case L4 and the sub-nodes into `cdr_thermal_cases` is an action on the simulation owner |

The mission simulation keeps a 0.5 W heater (`mission.rs`) and 0.05 W/K of mount conduction, both conservative for
the cold case against the design (1 W heater, ≈ 0.03 W/K mount; BUILD_VS_BUY §4).

### 2.2 Environment

| Parameter | Hot | Cold | Nominal (simulation) | Basis |
|-----------|-----|------|----------------------|-------|
| Solar flux | 1414 W/m² | 1322 W/m² | 1316–1414 W/m² by date | Perihelion / aphelion |
| Albedo | 0.35 | 0.25 | 0.30 | Typical high-inclination LEO design values (Gilmore, *Spacecraft Thermal Control Handbook*, ch. 2) |
| Earth IR | 258 W/m² | 218 W/m² | 237 W/m² | as above |
| Orbit | 475 km SSO (Version 2.1, BASELINE Rev E; 500 km in 2.0), deployment 2028-11-01. Bounding hot: dawn–dusk (β ≈ 74°, no eclipse), the BASELINE §8 definition. Real hot: LTAN 21:30 in January (β 40°, 32 min eclipse) | β ≈ 0 (36.0 min eclipse); real cold: LTAN 23:30 in June (β 6°) | LTAN 22:30, β 16–26° | BASELINE §3, §8; LTAN ±1 h from the launch provider |

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

The OBC board puts 1.16 W in the coils and 0.12 W in the bridges with all three magnetorquers saturated
(`hardware/electrical/obc/OBC_BOARD_DESIGN.md` §10.1, §17), not the 0.30 W of the LEOP row. The cold cases do not credit the
difference: a low tip-off rate ends the detumble early (§6.6 gives its effect).

Heater: ≈ 207 Ω (1.00 W at 14.4 V): **0.70 W** (0.696 W) at the 12.0 V floor, 0.95 W at 14.0 V (the EMERGENCY
threshold, 15 % SOC), 1.30 W at 16.4 V. The cases of §4 and §4.1 ran 0.72 W at the floor (the value of Version 2.0,
also in BASELINE §8 as "0.72–1.30 W"); §4.2 re-runs them at 0.70 W and its values govern. GEVS (GSFC-STD-7000B
§2.6.4.1) asks for < 70 % duty in the worst cold case at the minimum voltage.

## 3. Allowable temperatures and margin rule

| Item | Operating (°C) | Survival / non-operating (°C) | Source | Notes |
|------|----------------|-------------------------------|--------|-------|
| Battery (INR18650-35E) | Charge 0 … +45; discharge −10 … +60 | −20 … +60 | Cell specification (`hardware/eps/BATTERY_DESIGN.md` §2.1) | Heater-controlled at the cold end |
| OBC/EPS stack | −30 … +50; −40 in the first orbit (THERMAL_DESIGN §1.1; proposed SYS-600, CR-2026-017) | −30 … +60; −40 in the first orbit (proposed SYS-601) | THERMAL_DESIGN §1.1; parts audit SQSO-MFG-003 | Parts −40 … +85 or wider, line by line (e.g. STM32L4R5ZIT6 −40 … +85; BQ76952 −40 … +85; M-G370PDT0 −40 … +85); exceptions: EPS fuses F1/F3 and the rods (rows below) |
| RF board, PA | −30 … +50; −40 in the first orbit | −30 … +60; −40 in the first orbit | THERMAL_DESIGN §1.1; SQSO-MFG-003 | Parts −40 … +85 or wider; PA Q601 has no published minimum (row below). The flight software stops the transponder at a PA temperature of 70 °C |
| Jetson module and spreader | −15 (power-on gate) … +50 at the spreader; −25 … +105 junction | −30 … +60; −40 in the first orbit (TSTG −40 … +80, V-THM-8) | THERMAL_DESIGN §1.1; NVIDIA DS-11105-001 v1.7 (Tj −25 … +105 °C since v1.3) | The flight software's guard stops at 70 °C on the spreader and 95 °C junction (`payload_guard.rs`) |
| Body panels, frame | −40 … +80 | −60 … +100 | THERMAL_DESIGN §1.1 | |
| Wings (cell assemblies) | −40 … +85 | −100 … +100 | SOLAR_ARRAY_DESIGN §1.1 | To be replaced by the panel vendor's qualification range |
| EPS fuses F1, F3 (ITV4030L1412NR) | −10 … +65 (data sheet) | No mounted rating published | THERMAL_DESIGN §1.1; SQSO-MFG-003 §4 | Below the stack AFT at both ends: V-EPS-8 extended to −40 °C and +70 °C, or a design change |
| RF PA Q601 (RD01MUS3) | Tj ≤ +150; no published minimum | Tstg −40 … +125 | THERMAL_DESIGN §1.1; SQSO-MFG-003 §5 | Vendor statement and EM characterisation at −40 °C |
| ISIS AntS (−Z face) | −20 … +60 (vendor qualified) | −50 … +85 | THERMAL_DESIGN §1.1; SQSO-MFG-003 §12 | **Not a node of this network.** At its first-orbit release (after the wing burns, separation + ≈ 31 min, eclipse) the nearest nodes are −30.2 °C (frame-side hinge) to −40.6 °C (stowed wing), §4.3; an AntS node is an action on this analysis (SQSO-MFG-003 §12 item V-1) |
| Magnetorquer rods (CR0002) | −20 … +80 thermal vacuum, cold start −35 (family data) | Not published | THERMAL_DESIGN §1.1; SQSO-MFG-003 §12 | Not a node; inside the structure near the stack (−27.6 °C first orbit). Vendor rating or qualification at −40 °C |

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
| H2 Hot bounding, 600 s batch, 0.44 W/K | 27.8 … 28.9 | 27.5 … 31.9 | 27.8 … 32.2 | 26.1 … **39.0** | 21.6 … 39.0 | 63.8 … 75.2 | 0 % |
| H3 Hot bounding, 600 s batch, 1.0 W/K | 27.7 … 28.9 | 27.4 … 31.7 | 27.8 … 32.1 | 25.8 … 35.3 | 22.0 … 38.2 | 63.9 … 75.1 | 0 % |
| H5 Hot real (LTAN 21:30, January, β 40°) | 8.8 … 15.3 | −0.4 … 24.8 | −1.0 … 25.0 | −0.4 … 27.1 | −7.2 … 32.4 | −32.8 … **82.9** | 0 % |
| H6 Hot bounding, black battery (ε 0.85) | 27.7 … 28.8 | 28.7 … 31.1 | 28.9 … 31.1 | 28.0 … 33.3 | 22.7 … 37.8 | 67.2 … 75.1 | 0 % |
| H7 Hot bounding, Jetson stuck on at 8 W, 0.44 W/K | 43.5 … 44.1 | 45.8 … 48.1 | 44.2 … 46.5 | 58.4 … **60.6** | 38.1 … 51.5 | 62.6 … 67.1 | 0 % |
| H8 as H7, 1.0 W/K | 43.3 … 43.9 | 45.1 … 47.4 | 44.0 … 46.3 | 49.3 … 51.8 | 38.3 … 50.9 | 62.6 … 67.1 | 0 % |
| H9 Hot bounding, heater stuck on at 1.37 W, no cutout | 48.9 … **49.5** | 31.4 … 33.7 | 31.1 … 33.4 | 29.9 … 35.4 | 24.8 … 39.8 | 65.6 … 72.6 | 100 % |
| C1 Cold, SAFE, tumbling, 1.0 W | 2.5 … 9.5 | −8.9 … 19.5 | −9.1 … 19.7 | −9.0 … 19.1 | −14.7 … 27.5 | −40.8 … 37.5 | 0 % |
| C4 Cold, SAFE, spin-pointed, 1.0 W | 2.0 … 8.2 | **−11.7** … 13.0 | −11.9 … 13.1 | −11.9 … 12.5 | −16.8 … 22.0 | −39.2 … 78.9 | 32 % |
| C11 as C4, 0.72 W (12.0 V) | 1.7 … 7.9 | −11.7 … 12.9 | −11.9 … 13.0 | −11.9 … 12.4 | −16.9 … 22.0 | −39.3 … 78.6 | 40 % |
| C12 as C4, 0.5 W (the simulation's heater) | 1.0 … 7.4 | −11.9 … 12.8 | −12.0 … 12.9 | −12.0 … 12.3 | −17.0 … 21.8 | −39.3 … 79.1 | 49 % |
| C13 Cold, EMERGENCY, spin-pointed, 0.72 W | 1.7 … 7.9 | −11.8 … 12.9 | −12.0 … 13.0 | −12.0 … 12.4 | −16.9 … 21.9 | −39.3 … 78.6 | 40 % |
| C9 Cold, spin-pointed, heater failed off | **−2.8** … 3.4 | −12.5 … 12.2 | −12.6 … 12.5 | −12.6 … 11.9 | −17.5 … 21.4 | −39.5 … 80.0 | — |
| C15 as C11, mount 0.10 W/K (as-built twice the model) | **−1.0** … 8.8 | −11.1 … 12.7 | −11.2 … 12.8 | −11.2 … 12.1 | −16.0 … 21.6 | −39.0 … 78.6 | 51 % |
| C7 Cold, tumbling, black battery, 0.72 W | 1.8 … 13.6 | −8.0 … 19.3 | −8.1 … 19.4 | −8.0 … 18.8 | −13.6 … 27.1 | −40.5 … 37.1 | 26 % |
| C14 Cold, spin-pointed, wings stowed, 0.72 W | **−1.3** … 3.0 | −18.3 … −0.5 | **−18.6** … −0.6 | −18.8 … −1.2 | −22.7 … 7.3 | −31.6 … −14.5 | 100 % |
| C16 as C14, 0.95 W (14.0 V) | −0.1 … 6.0 | −18.2 … −0.4 | −18.5 … −0.6 | −18.7 … −1.2 | −22.6 … 7.1 | −31.5 … −14.5 | 81 % |
| C10 Cold real (LTAN 23:30, June, β 6°), tumbling, 0.72 W | 3.8 … 10.9 | −7.8 … 21.0 | −7.9 … 21.3 | −7.8 … 20.6 | −13.6 … 27.7 | −39.9 … 46.9 | 0 % |
| C22 Cold, spin-pointed, 0.95 W, **set points +5/+9 °C** | 4.4 … 10.1 | −11.3 … 13.3 | −11.6 … 13.4 | −11.6 … 12.7 | −16.5 … 22.3 | −39.1 … 78.2 | 46 % |
| C23 as C22, mount 0.03 W/K (the design) | 4.9 … 9.9 | −11.6 … 13.3 | −11.8 … 13.4 | −11.8 … 12.7 | −16.8 … 22.3 | −39.2 … 78.1 | 40 % |
| C21 as C22, mount 0.10 W/K | 1.2 … 10.5 | −10.6 … 13.2 | −10.7 … 13.2 | −10.8 … 12.6 | −15.5 … 22.0 | −38.8 … 78.2 | 59 % |
| C20 as C22, wings stowed | 2.5 … 6.7 | −17.6 … −0.1 | −18.0 … −0.3 | −18.2 … −0.9 | −22.1 … 7.4 | −31.2 … −14.2 | 100 % |
| C25 as C22 at 1.30 W (16.4 V, full pack) | 5.0 … 10.1 | −11.3 … 13.5 | −11.5 … 13.5 | −11.5 … 12.9 | −16.4 … 22.4 | −39.1 … 79.2 | 36 % |
| C24 Cold, tumbling, 1.30 W, +5/+9 °C | 5.0 … 12.8 | −8.4 … 19.9 | −8.6 … 20.1 | −8.5 … 19.4 | −14.3 … 27.9 | −40.6 … 37.2 | 18 % |

Model mount: 0.042 W/K conduction plus ≈ 0.008 W/K radiation (the simulation's 0.05 W/K in all); "mount 0.10 W/K" and
"0.03 W/K" cases set the conduction.

**Dispenser soak, powered off** (§6): the dispenser wall follows the SpaceX bounding interface temperature (UG v11
Table 4-11: hot 40 → 69 °C by 3000 s, cold −5 → −20 °C by 3000 s, held), ε_eff 0.70 to the panels and 0.5 W/K through
the rails; start at the pad environment (Table 4-12: 15–30 °C).

| Soak | Time after lift-off | Battery | OBC | RF | Jetson | +X panel |
|------|---------------------|---------|-----|----|--------|----------|
| Hot (start 25 °C) | 1 h / 2 h / 3 h | 37.4 / **55.5** / **63.8** | 55.0 / 66.4 / 68.1 | 55.2 / 66.6 / 68.3 | 54.7 / 66.6 / 68.3 | 58.9 / 67.1 / 68.4 |
| Cold (start 15 °C) | 1 h / 2 h / 3 h | 3.4 / **−8.9** / **−15.1** | −10.3 / −17.4 / −19.0 | −10.5 / −17.6 / −19.1 | −10.3 / −17.6 / −19.1 | −12.6 / −18.0 / −19.3 |

**Deployment after the 2 h cold soak** (L1, L2; β = 0, tip-off 10°/s, detumble at full dipole): with the 1 W heater
the battery rises from −8.9 °C to 0 °C in 33 min and reaches +4.3 °C by the end of the first orbit; with 0.5 W it is
still at −1.5 °C after one orbit. OBC −17.4 °C, wings −45.4 °C. The mission simulation with the whole spacecraft at
−20 °C at separation (`--set storage_temp_c=-20`, flight software in the loop) deployed both antennas and wings on
time; the battery stayed below −10 °C for 53 min, first rose above 0 °C after 1.5 h (2.1 h below 0 °C in all,
2.3 h at LTAN 23:30), and the OBC reached −22.9 °C (−23.3 °C at LTAN 23:30). From −15 °C: 40 min below −10 °C,
1.15 h below 0 °C. The same run from +55 °C (a hot soak) kept the pack above the 45 °C charge limit for 0.8 h; the
bus stayed up in every case (lowest recorded pack voltage 14.65 V).

### 4.1 Re-run at 475 km (Version 2.1)

The cases of §4 re-run at the BASELINE Rev E deployment altitude, 475 km (465–485 km acceptable), with the launch
planning date 2028-11-01 (the seasonal cases fall in the first year after it: January, March and June 2029), the heater
set points of the flight software (+5/+9 °C, SRS-F130; §4 ran several cases at +2/+6 °C) and one new case, L3: the
deployment itself on 2028-11-01 at LTAN 22:30 from the 2 h cold soak (`software/rust/sqso-sil/src/env/thermal.rs`
`cdr_thermal_cases`, flight software round 3; tabulated by `thermal_cases.py`). Last orbit of each case; minimum …
maximum, °C; the margin column is the worst of every node against §3 after the 10 °C uncertainty.

| Case | β (°) | Eclipse (min) | Battery | OBC/EPS | RF (PA) | Jetson | Body | Wings | Heater duty | Worst margin after 10 °C |
|---|---|---|---|---|---|---|---|---|---|---|
| H1 hot bounding, no eclipse, bursts, 0.44 W/K | -73.8 | 0.0 | 28.4 … 29.0 | 28.9 … 31.4 | 29.1 … 31.4 | 28.2 … 33.6 | 22.8 … 38.2 | 67.4 … 75.3 | 0 % | -0.3 (wing hot) |
| H2 hot bounding, no eclipse, 600 s batch, 0.44 W/K | -73.8 | 0.0 | 28.0 … 29.2 | 27.7 … 32.2 | 28.1 … 32.5 | 26.3 … 39.3 | 21.8 … 39.3 | 64.1 … 75.4 | 0 % | -0.4 (wing hot) |
| H3 hot bounding, no eclipse, 600 s batch, 1.0 W/K | -73.8 | 0.0 | 28.0 … 29.2 | 27.6 … 32.0 | 28.0 … 32.4 | 26.0 … 35.6 | 22.2 … 38.4 | 64.2 … 75.4 | 0 % | -0.4 (wing hot) |
| H4 hot bounding, no eclipse, bursts, 1.0 W/K | -73.8 | 0.0 | 28.4 … 29.0 | 28.9 … 31.3 | 29.1 … 31.5 | 27.3 … 32.8 | 23.2 … 37.7 | 67.3 … 75.3 | 0 % | -0.3 (wing hot) |
| H5 hot real, LTAN 21:30 Jan, bursts, 0.44 W/K | 40.0 | 32.7 | 8.8 … 15.2 | -0.5 … 24.8 | -1.1 … 25.0 | -0.5 … 26.4 | -7.2 … 32.5 | -32.7 … 83.1 | 0 % | -8.1 (wing hot) |
| H7 hot bounding, Jetson stuck on 8 W, 0.44 W/K | -73.6 | 0.0 | 43.7 … 44.3 | 46.0 … 48.4 | 44.4 … 46.8 | 58.6 … 60.9 | 38.3 … 51.7 | 62.8 … 67.4 | 0 % | -20.9 (Jetson hot) |
| H8 hot bounding, Jetson stuck on 8 W, 1.0 W/K | -73.6 | 0.0 | 43.5 … 44.1 | 45.3 … 47.7 | 44.2 … 46.6 | 49.5 … 52.0 | 38.5 … 51.1 | 62.7 … 67.4 | 0 % | -12.0 (Jetson hot) |
| H9 hot bounding, heater stuck on 1.37 W (16.4 V), no cutout | -73.6 | 0.0 | 49.2 … 49.8 | 31.6 … 34.0 | 31.4 … 33.7 | 30.2 … 35.6 | 25.0 … 40.0 | 65.8 … 72.9 | 100 % | -14.8 (battery hot) |
| H6 hot bounding, black battery (eps 0.85) | -73.8 | 0.0 | 27.9 … 29.0 | 29.0 … 31.4 | 29.2 … 31.4 | 28.3 … 33.6 | 22.9 … 38.1 | 67.4 … 75.3 | 0 % | -0.3 (wing hot) |
| C1 cold, beta 0, SAFE tumbling, heater 1.0 W | -3.4 | 36.1 | 5.0 … 12.7 | -8.3 … 20.0 | -8.4 … 20.1 | -8.3 … 19.4 | -14.2 … 26.9 | -40.4 … 37.2 | 22 % | -10.4 (wing cold) |
| C2 cold, beta 0, SAFE tumbling, heater 0.72 W (12.0 V) | -3.4 | 36.1 | 5.0 … 12.5 | -8.3 … 19.9 | -8.5 … 20.1 | -8.4 … 19.4 | -14.2 … 26.8 | -40.5 … 37.1 | 28 % | -10.5 (wing cold) |
| C3 cold, beta 0, SAFE tumbling, heater 0.5 W | -3.4 | 36.1 | 4.9 … 12.2 | -8.4 … 19.8 | -8.6 … 20.0 | -8.5 … 19.3 | -14.3 … 26.8 | -40.5 … 36.8 | 34 % | -10.5 (wing cold) |
| C4 cold, beta 0, SAFE spin-pointed, heater 1.0 W | -3.4 | 36.1 | 4.6 … 10.2 | -11.2 … 13.5 | -11.4 … 13.5 | -11.5 … 12.9 | -16.3 … 22.5 | -39.0 … 78.2 | 44 % | -9.0 (wing cold) |
| C5 cold, beta 0, EMERGENCY tumbling, heater 0.72 W | -3.4 | 36.1 | 5.0 … 12.5 | -8.3 … 19.9 | -8.5 … 20.0 | -8.4 … 19.4 | -14.3 … 26.8 | -40.5 … 37.1 | 28 % | -10.5 (wing cold) |
| C6 cold, beta 0, SAFE tumbling, wings stowed, 0.72 W | -3.4 | 36.1 | 4.2 … 9.7 | -9.6 … 12.7 | -9.8 … 12.7 | -9.8 … 12.0 | -15.0 … 19.7 | -24.6 … 22.1 | 52 % | +0.2 (RF cold) |
| C7 cold, beta 0, SAFE tumbling, black battery, 0.72 W | -3.4 | 36.1 | 3.2 … 14.5 | -7.6 … 19.6 | -7.7 … 19.7 | -7.7 … 19.0 | -13.3 … 26.4 | -40.2 … 36.9 | 40 % | -10.2 (wing cold) |
| C8 cold, beta 0, SAFE tumbling, mount 0.10 W/K, 0.72 W | -3.4 | 36.1 | 3.1 … 14.2 | -7.6 … 19.6 | -7.7 … 19.7 | -7.7 … 19.0 | -13.3 … 26.4 | -40.2 … 36.9 | 41 % | -10.2 (wing cold) |
| C9 cold, beta 0, SAFE spin-pointed, heater failed off | -3.4 | 36.1 | -2.6 … 3.5 | -12.4 … 12.4 | -12.5 … 12.6 | -12.5 … 12.0 | -17.4 … 21.5 | -39.4 … 80.2 | 100 % | -9.4 (wing cold) |
| C11 cold, beta 0, SAFE spin-pointed, heater 0.72 W (12.0 V) | -3.4 | 36.1 | 3.7 … 9.8 | -11.3 … 13.3 | -11.6 … 13.4 | -11.6 … 12.7 | -16.4 … 22.3 | -39.0 … 78.8 | 54 % | -9.0 (wing cold) |
| C12 cold, beta 0, SAFE spin-pointed, heater 0.5 W | -3.4 | 36.1 | 2.6 … 9.3 | -11.5 … 13.1 | -11.7 … 13.2 | -11.7 … 12.5 | -16.6 … 22.1 | -39.1 … 79.3 | 66 % | -9.1 (wing cold) |
| C13 cold, beta 0, EMERGENCY spin-pointed, heater 0.72 W | -3.4 | 36.1 | 3.7 … 9.8 | -11.4 … 13.3 | -11.6 … 13.3 | -11.6 … 12.7 | -16.5 … 22.3 | -39.0 … 78.8 | 54 % | -9.0 (wing cold) |
| C14 cold, beta 0, SAFE spin-pointed, wings stowed, 0.72 W | -3.4 | 36.1 | -1.1 … 3.2 | -18.1 … -0.3 | -18.5 … -0.4 | -18.6 … -1.1 | -22.5 … 7.5 | -31.4 … -14.3 | 100 % | -8.5 (RF cold) |
| C15 cold, beta 0, SAFE spin-pointed, mount 0.10 W/K, 0.72 W | -3.4 | 36.1 | 0.2 … 10.1 | -10.7 … 13.0 | -10.8 … 13.1 | -10.8 … 12.5 | -15.5 … 21.9 | -38.8 … 78.8 | 65 % | -8.8 (wing cold) |
| C16 cold, beta 0, SAFE spin-pointed, wings stowed, 0.95 W (14.0 V) | -3.4 | 36.1 | 2.7 … 6.9 | -17.5 … 0.1 | -17.9 … -0.1 | -18.0 … -0.7 | -21.9 … 7.6 | -30.9 … -14.0 | 100 % | -7.9 (RF cold) |
| C17 cold, beta 0, SAFE spin-pointed, wings stowed, 1.09 W (180 ohm, 14.0 V) | -3.4 | 36.1 | 3.2 … 8.0 | -17.4 … -0.1 | -17.8 … -0.3 | -18.0 … -0.9 | -21.9 … 7.3 | -30.9 … -14.1 | 100 % | -7.8 (RF cold) |
| C20 cold, beta 0, wings stowed, 0.95 W, set points +5/+9 C | -3.4 | 36.1 | 2.7 … 6.9 | -17.5 … 0.1 | -17.9 … -0.1 | -18.0 … -0.7 | -21.9 … 7.6 | -30.9 … -14.0 | 100 % | -7.9 (RF cold) |
| C21 cold, beta 0, mount 0.10 W/K, 0.95 W, set points +5/+9 C | -3.4 | 36.1 | 1.3 … 10.6 | -10.5 … 13.3 | -10.6 … 13.4 | -10.6 … 12.7 | -15.3 … 22.2 | -38.7 … 78.4 | 58 % | -8.7 (wing cold) |
| C22 cold, beta 0, nominal, 0.95 W, set points +5/+9 C | -3.4 | 36.1 | 4.4 … 10.1 | -11.2 … 13.4 | -11.5 … 13.5 | -11.5 … 12.8 | -16.4 … 22.4 | -39.0 … 78.3 | 45 % | -9.0 (wing cold) |
| C23 cold, beta 0, nominal mount 0.03 W/K, 0.95 W, set points +5/+9 C | -3.4 | 36.1 | 4.9 … 10.0 | -11.5 … 13.4 | -11.7 … 13.5 | -11.7 … 12.8 | -16.6 … 22.4 | -39.1 … 78.3 | 40 % | -9.1 (wing cold) |
| C24 cold, beta 0, SAFE tumbling, 1.30 W (16.4 V), set points +5/+9 C | -3.4 | 36.1 | 5.0 … 12.8 | -8.2 … 20.0 | -8.4 … 20.2 | -8.3 … 19.5 | -14.2 … 26.9 | -40.4 … 37.4 | 18 % | -10.4 (wing cold) |
| C25 cold, beta 0, SAFE spin-pointed, 1.30 W (16.4 V), set points +5/+9 C | -3.4 | 36.1 | 5.0 … 10.2 | -11.2 … 13.6 | -11.4 … 13.7 | -11.4 … 13.0 | -16.3 … 22.6 | -38.9 … 79.3 | 35 % | -8.9 (wing cold) |
| C18 cold, beta 0, SAFE spin-pointed, mount 0.10 W/K, 0.95 W | -3.4 | 36.1 | 1.3 … 10.6 | -10.5 … 13.3 | -10.6 … 13.4 | -10.6 … 12.7 | -15.3 … 22.2 | -38.7 … 78.4 | 58 % | -8.7 (wing cold) |
| C19 cold, beta 0, SAFE spin-pointed, mount 0.10 W/K, 1.09 W | -3.4 | 36.1 | 1.9 … 10.8 | -10.4 … 13.4 | -10.5 … 13.5 | -10.6 … 12.8 | -15.2 … 22.3 | -38.7 … 78.1 | 55 % | -8.7 (wing cold) |
| C10 cold real, LTAN 23:30 June, SAFE tumbling, 0.72 W | 5.8 | 35.9 | 5.0 … 11.6 | -7.4 … 21.3 | -7.5 … 21.6 | -7.4 … 20.9 | -13.5 … 28.2 | -39.6 … 47.0 | 0 % | -9.6 (wing cold) |
| L1 LEOP from cold soak, heater 1.0 W | -1.9 | 36.1 | -8.9 … 4.3 | -17.4 … 3.3 | -17.6 … 2.7 | -17.6 … 1.6 | -22.3 … 10.4 | -45.2 … 33.8 | 100 % | -15.2 (wing cold); battery ≥ 0 °C after 33 min |
| L2 LEOP from cold soak, heater 0.5 W | -1.9 | 36.1 | -8.9 … -1.5 | -17.4 … 3.5 | -17.6 … 3.0 | -17.6 … 1.9 | -22.6 … 10.2 | -45.3 … 34.8 | 100 % | -15.3 (wing cold) |
| L3 LEOP real, 2028-11-01 LTAN 22:30, from cold soak, heater 1.0 W | 19.6 | 35.3 | -8.9 … 9.9 | -24.0 … 19.4 | -24.8 … 19.0 | -25.1 … 18.1 | -29.1 … 22.1 | -45.4 … 45.0 | 83 % | -15.4 (wing cold); battery ≥ 0 °C after 58 min |

Extremes at 475 km, against 500 km (§4):

| Node | Hot bounding (H1–H6) | Failure hot (H7 Jetson stuck on, H9 heater stuck on) | Cold, wings deployed (C1–C25 nominal set) | Wings stowed (C6, C14, C16, C17, C20) | LEOP from the cold soak (L1–L3) |
|------|----------------------|------------------------------------------------------|-------------------------------------------|----------------------------------------|----------------------------------|
| Battery | 27.9 … 29.2 (was 27.7 … 28.9) | 44.3 (H7); 49.8 (H9; was 49.5) | +3.6 at 0.70 W spin-pointed (C11, §4.2; +3.7 at 0.72 W); 0.0 with a 0.10 W/K mount at 0.70 W (C15, §4.2; +0.2 at 0.72 W), +1.3 at 0.95 W (C21); −2.6 heater failed (C9; was −2.8) | −1.5 at 0.70 W (C14, §4.2; −1.1 at 0.72 W, was −1.3); +2.7 at 0.95 W (C16, C20) | −8.9 at the start; 0 °C after 33 min (L1), 58 min (L3) and 80 min (L4, §4.3) |
| OBC/EPS | 32.2 (H2; was 31.9) | 48.4 (H7; was 48.1) | −11.5 (C12; was −11.9) | −18.2 (C14 at 0.70 W; was −18.3) | −17.4 (L1); −24.0 (L3); **−27.6 (L4)** |
| RF (PA) | 32.5 (H2; was 32.2) | 46.8 (H7; was 46.5) | −11.7 (C12; was −12.0) | −18.5 (C14; was −18.6) | −17.6 (L1); −24.8 (L3); **−28.6 (L4)** |
| Jetson | 39.3 (H2; was 39.0) | 60.9 (H7; was 60.6) | −11.7 (C12; was −12.0) | −18.7 (C14 at 0.70 W; was −18.8) | −17.6 (L1); −25.1 (L3); **−28.9 (L4)** |

The lower altitude changes the orbit-average results by 0.2–0.4 °C (a slightly larger Earth view and a shorter
orbit). The verdicts of §1 and §5 stand. The new deployment case L3 is the coldest for the boards: −24 °C on the OBC
and −25 °C on the RF board and the Jetson during the first orbit after a 2 h cold soak, below SYS-600
(−20 °C operating) and inside the part ratings (−40 °C); it agrees with the mission simulation's −22.9 °C from a
−20 °C spacecraft (§4). The battery leaves the cell's 0 °C charge limit 58 min after deployment (L1, β = 0: 33 min).
RID THM-06 (deployment within 2 h of the soak) covers it. L1–L3 deploy the wings at separation and L1 separates in
sunlight; case L4 (§4.3: β = 0, separation at eclipse entry, wings stowed until the first burn) is colder for the
boards than L3 and governs THM-04 (§6.6).

### 4.2 Heater at 0.70 W (Version 2.2)

The ≈ 207 Ω heater gives 12.0² / 207 = 0.696 W at the 12.0 V floor, not the 0.72 W of the §4.1 rows above. The rows of
§4.1 run at 0.72 W, re-run at 0.70 W with the §4.1 conditions (475 km, set points +5/+9 °C; the analysis harness of
§2.1, which reproduces C11 and C14 of §4.1 at 0.72 W exactly). Minimum … maximum, °C, last orbit; these supersede the
0.72 W rows.

| Case | Battery | OBC/EPS | RF (PA) | Jetson | Body | Wings | Heater duty | Battery minimum against 0.72 W |
|---|---|---|---|---|---|---|---|---|
| C2 cold, β 0, SAFE tumbling, 0.70 W | 5.0 … 12.5 | −8.3 … 19.9 | −8.5 … 20.1 | −8.4 … 19.4 | −14.3 … 26.8 | −40.5 … 37.0 | 28 % | 0.0 |
| C5 cold, β 0, EMERGENCY tumbling, 0.70 W | 5.0 … 12.5 | −8.3 … 19.9 | −8.5 … 20.0 | −8.4 … 19.4 | −14.3 … 26.8 | −40.5 … 37.0 | 28 % | 0.0 |
| C6 cold, β 0, SAFE tumbling, wings stowed, 0.70 W | 4.1 … 9.7 | −9.6 … 12.6 | −9.8 … 12.7 | −9.8 … 12.0 | −15.0 … 19.7 | −24.6 … 22.0 | 53 % | −0.1 |
| C7 cold, β 0, SAFE tumbling, black battery, 0.70 W | 3.1 … 14.4 | −7.7 … 19.6 | −7.8 … 19.7 | −7.7 … 19.0 | −13.3 … 26.4 | −40.2 … 36.9 | 40 % | −0.1 |
| C8 cold, β 0, SAFE tumbling, mount 0.10 W/K, 0.70 W | 3.0 … 14.2 | −7.6 … 19.6 | −7.8 … 19.7 | −7.7 … 19.0 | −13.3 … 26.4 | −40.2 … 36.9 | 41 % | −0.1 |
| **C11** cold, β 0, SAFE spin-pointed, 0.70 W (12.0 V) | **3.6** … 9.8 | −11.3 … 13.3 | −11.6 … 13.3 | −11.6 … 12.7 | −16.5 … 22.3 | −39.0 … 78.9 | 55 % | −0.1 |
| C13 cold, β 0, EMERGENCY spin-pointed, 0.70 W | 3.6 … 9.8 | −11.4 … 13.3 | −11.6 … 13.3 | −11.6 … 12.7 | −16.5 … 22.3 | −39.0 … 78.9 | 55 % | −0.1 |
| **C14** cold, β 0, SAFE spin-pointed, wings stowed, 0.70 W | **−1.5** … 2.8 | −18.2 … −0.4 | −18.5 … −0.4 | −18.7 … −1.1 | −22.6 … 7.4 | −31.4 … −14.3 | 100 % | −0.4 |
| **C15** cold, β 0, SAFE spin-pointed, mount 0.10 W/K, 0.70 W | **0.0** … 10.0 | −10.7 … 13.0 | −10.8 … 13.1 | −10.9 … 12.4 | −15.5 … 21.9 | −38.8 … 78.9 | 66 % | −0.2 |
| C10 cold real, LTAN 23:30 June, SAFE tumbling, 0.70 W | 5.0 … 13.0 | −7.4 … 21.3 | −7.5 … 21.5 | −7.4 … 20.8 | −13.5 … 28.1 | −39.6 … 46.0 | 18 % | 0.0 |

The change is 0.0–0.4 °C on the battery and ≤ 0.1 °C elsewhere; the verdicts stand except that C15 (an as-built mount
twice the model's) now reaches 0 °C, at the hardware charge-inhibit threshold (+1.1 °C ± 1 K, SQSO-EPS-BRD-001 §5.3).
C10's pack sits at the +5 °C switch-on point, so its last-orbit duty depends on the cycle phase (0 % at 0.72 W, 18 %
at 0.70 W). C6 and C14, wings stowed, are discussed in §6.8 with a heater-power sweep.

### 4.3 LEOP with the wings stowed until the burn; hinge, burn line and wing (Version 2.2, RID P3S-01)

L1–L3 deploy the wings at separation. In flight they stay stowed until the first burn, 30 min after separation
(`sqso_fsw::deployment`, `WAIT_TIME_MS`), and, if a burn fails, until the retries 50 min later (`RETRY_DELAY_MS`):
separation + 80 and + 130 min. **Cold case:** the 2 h dispenser cold soak of §4 (D3), β = 0 with the cold environment
(1322 W/m², albedo 0.25, Earth IR 218 W/m²), tip-off tumble 10°/s, LEOP dissipation, battery heater 0.70 W, the wings
stowed throughout (the burns have failed), and the separation time swept over the orbit in 2 min steps: separation at
eclipse entry and the coldest separation phase for each burn. Sensitivities: a 3 h soak, the deployment orbit of L3
(2028-11-01, LTAN 22:30, β 19.6°) and the hinge couplings. Sub-nodes of each wing, driven by the network (one-way):

| Sub-node | Heat capacity | Couplings | Represents |
|---|---|---|---|
| Hinge, frame side | 4 J/K (ears, pin, bushings, springs, ≈ 5 g) | 0.20 W/K to its ±X body panel (ears bolted to the shear panel), 0.01 W/K to the wing side (the network's deployed-hinge value); 3 cm², ε 0.85, half its view to space with Earth IR, albedo and Sun on the hinge-corner normal | A286 pin and MoS₂ film, Ti-6Al-4V bushings, frame ears, primary spring below the wing |
| Hinge, wing side | 2 J/K | 0.05 W/K to the wing panel (knuckles in the end zones), 0.01 W/K to the frame side; 1 cm² to space | Wing knuckles, clock-spring flex cup |
| Burn board and resistor | 1.5 J/K | 0.015 W/K each to the ±X and −Y body panels (corner strip at the −Y free corner, `hardware/harness/HARNESS_DESIGN.md` §7.2); radiation to the interior; 10 % of its view through the line slot to the stowed wing | The line inside the resistor: the release energy |
| Hold-down line, free span | 0.034 J/K (0.1 m of Ø0.5 mm line) | Radiation only: half its view to the stowed wing's outer face, half to space (α 0.5, ε 0.85) | The line over the post: preload, not the cut |
| Snubber | — | Midway between the wing and its body panel | PTFE-faced silicone snubber |

Predicted temperatures at the burns, °C, colder wing (W2, on −X, for the tip-off axis), and with the 10 °C margin of
BASELINE §8:

| Node | Burn 1, sep. + 30 min | Retry 1, sep. + 80 min | Retry 2, sep. + 130 min |
|---|---|---|---|
| *Separation at eclipse entry* | *eclipse* | *sunlit* | *eclipse exit* |
| Wing panel | −40.1 (−50.1) | +2.1 (−7.9) | −30.4 (−40.4) |
| Hinge, frame side | −30.1 (−40.1) | +0.6 (−9.4) | −19.0 (−29.0) |
| Hinge, wing side | −38.4 (−48.4) | +1.8 (−8.2) | −29.1 (−39.1) |
| Burn resistor and line in it | −29.3 (−39.3) | +1.1 (−8.9) | −18.1 (−28.1) |
| Line, free span over the post | −69.7 (−79.7) | −9.8 (−19.8) | −58.8 (−68.8) |
| Snubber | −34.9 (−44.9) | +1.4 (−8.6) | −24.3 (−34.3) |
| *Coldest separation phase for that burn* | *34 min after the node (eclipse)* | *80 min (sunlit)* | *28 min (eclipse)* |
| Wing panel | **−40.6 (−50.6)** | −29.9 (−39.9) | −30.6 (−40.6) |
| Hinge, frame side | **−30.2 (−40.2)** | −18.1 (−28.1) | −18.5 (−28.5) |
| Hinge, wing side | **−38.8 (−48.8)** | −28.2 (−38.2) | −28.5 (−38.5) |
| Burn resistor and line in it | **−29.4 (−39.4)** | −17.1 (−27.1) | −17.6 (−27.6) |
| Line, free span over the post | −70.8 (−80.8) | −59.2 (−69.2) | −63.3 (−73.3) |
| Snubber | −35.2 (−45.2) | −23.7 (−33.7) | −24.3 (−34.3) |

Sensitivities at burn 1 (coldest phase): a **3 h soak** gives wing −41.7 (−51.7), hinge −31.6 (−41.6) frame side and
−40.0 (−50.0) wing side, resistor −30.8 (−40.8), line −71.6, snubber −36.4 (−46.4); at the retries it is 0.5–0.8 °C
colder than the 2 h soak. The **L3 deployment orbit** is 0.2–0.7 °C warmer than β = 0 (wing −39.9, wing-side hinge
−38.1, resistor −29.2). A poor ear joint (0.05 W/K) with a better pin path (0.03 W/K) puts the frame side at −32.5
(−42.5) and the wing side at −37.2. **Later attempts** commanded from the ground with the wings still stowed see the
stowed steady state (C14-like, SAFE): wing −31.4, wing-side hinge −29.6 at any point of the orbit, sun-pointed; −24.2 /
−22.5 tumbling. **The −45 °C** quoted for the stowed wing before (L1–L3, MECHANISMS_ANALYSIS §4A) is the deployed wing
in the eclipse after deployment (−45.3 °C here after a burn at + 30 min), not the wing at the burn.

**For the mechanisms (RID P3S-01).** With the margin, the coldest node at any burn is −51.7 °C (wing panel, 3 h soak;
−50.6 °C after 2 h), the wing side of the hinge −50.0 °C, the frame side (pin, springs) −41.6 °C (−42.5 °C with the
poor ear joint, about −44 °C with it and the 3 h soak together), the snubber −46.4 °C
and the burn resistor −40.8 °C: all inside the −55 °C design and test point of SQSO-MECH-001 Version E (torque margin
+0.08, burn ratio 1.52 at −55 °C; VI-STR-04/-05/-06, VI-MECH-01/-02/-03) by 3.3 °C or more. The release energy depends
on the resistor's temperature, −39.4 °C (−40.8 °C after 3 h) with the margin, so the 8 s burn's ratio lies between the
−40 °C (1.67) and −55 °C (1.52) rows. The only node below −55 °C is the free span of the line, −81 °C with the margin
(radiating half to space): not in the release path, but it is where the line's negative expansion coefficient
(Dyneema and Vectran lengthen as they cool) and the frame's contraction act together on the 30 N preload; the
mechanical designer checks the preload and slack with the free span at −81 °C and the anchors at −42 °C.

## 5. Margins

| Item | Worst prediction | Limit | Margin | After 10 °C | Verdict |
|------|------------------|-------|--------|-------------|---------|
| Battery, hot, operations | 28.9 °C (H2) | +45 (charge) | +16.1 | +6.1 | Pass |
| Battery, hot, Jetson stuck on | 44.1 °C (H7) | +45 (charge) | +0.9 | −9.1 | Charge inhibited hot (fail-safe); +16 to the +60 °C discharge limit |
| Battery, hot, heater stuck on | 49.5 °C (H9) | +45 / cutout +40 | — | — | The series hardware cutout at +40 °C is required (BUILD_VS_BUY §4.3) and must be verified |
| Battery, cold, heater-controlled (Version 2.2: 475 km, +5/+9 °C, 0.70 W at the 12.0 V floor) | +3.6 °C at 0.70 W, 55 % duty (C11, §4.2) | 0 °C (charge; hardware inhibit +1.1 °C) | +3.6 | duty 55 % < 70 % | Pass by the duty rule; at the limit with mount 0.10 W/K (0.0 °C, C15); below it with the heater failed (−2.6 °C, C9) |
| Battery, cold, 14.0 V | +4.4 °C at 0.95 W, 45 % (C22); +1.3 °C at 0.10 W/K (C21) | 0 °C | +4.4 / +1.3 | duty 45 % / 58 % | Pass |
| Battery, cold, wings stowed (contingency) | −1.5 °C at 0.70 W, +2.7 °C at 0.95 W, 100 % duty (C14, C16) | 0 °C | −1.5 / +2.7 | duty 100 % | Contingency accepted (§6.8, THM-03) |
| OBC/EPS stack, cold, orbit | −11.5 °C (C12) | −30 (proposed AFT, §6.6); −20 (SYS-600) | +18.5 / +8.5 | +8.5 / −1.5 | Pass against the proposed AFT |
| OBC/EPS stack and RF board, LEOP first orbit | −27.6 / −28.6 °C (L4, §4.3); −29.1 / −30.0 °C after a 3 h soak | −30 (AFT); −40 (parts; proposed first-orbit AFT) | +2.4 / +1.4 to −30 | **−7.6 / −8.6** to −30; +2.4 / +1.4 to −40 | Outside the −30 °C AFT; inside the parts' −40 °C: first-orbit AFT −40 °C with a test-margin deviation (§6.6, THM-04) |
| Jetson (off), LEOP first orbit | −28.9 °C (L4) | −25 (module Tj, the only data-sheet minimum); −40 proposed first-orbit non-operating AFT | −3.9 / +11.1 | **−13.9** / +1.1 | Needs the EM non-operating soak (V-THM-8) |
| RF board, cold, wings stowed | −18.5 °C (C14) | −30 (proposed AFT); −20 (SYS-600) | +11.5 / +1.5 | +1.5 / **−8.5** | Pass against the proposed AFT |
| OBC/EPS stack, hot | 31.9 °C (H2) | +50 | +18.1 | +8.1 | Pass |
| Jetson, hot, bursts and batches | 39.0 °C (H2) | +50 (SYS-600); 70 (guard) | +11.0 / +31.0 | +1.0 / +21.0 | Pass |
| Jetson, stuck on | 60.6 °C (H7) | +50 | −10.6 | −20.6 | Failure case; the energy loss ends it first (POWER_BUDGET_ANALYSIS §8.4) |
| Body panels | −22.7 … +51.5 °C | −40 … +80 | +17.3 / +28.5 | +7.3 / +18.5 | Pass |
| Wings | −45.4 … +82.9 °C | −40 … +85 | −5.4 / +2.1 | **−15.4 / −7.9** | Needs the vendor's qualification range (RID THM-05) |
| Battery, dispenser hot soak at 2 h | 55.5 °C | +60 | +4.5 | −5.5 | **Fails** at ≈ 2.4 h (bounding environment; RID THM-07) |
| Boards, dispenser hot soak at 2 h | 66.6 °C | +60 (SYS-601) | −6.6 | −16.6 | **Fails** (bounding) |
| Battery, dispenser cold soak at 2 h | −8.9 °C | −20 survival; −10 discharge | +11.1 / +1.1 | +1.1 / −8.9 | Survives; at the discharge limit at deployment |

## 6. Discussion

### 6.1 Battery heater and set points

The isolated mount works: in the hot case the pack follows the frame mean (28 °C) and in the cold case its time
constant (≈ 250 J/K / 0.08 W/K ≈ 50 min) carries it through the eclipse. But the flight software's thermostat (on
below +2 °C, off above +6 °C, `sqso_fsw::heater`) leaves 2 °C between the switch-on point and the 0 °C charge limit,
and the pack keeps cooling for several minutes after the heater comes on. The mission simulation's 14-day
commissioning with the battery radiation added reaches +0.07 °C; the cold case with an as-built mount of 0.10 W/K
(BUILD_VS_BUY §4.5 allows for it) reaches −1.0 °C even with the heater at 51 % duty; the 1 W heater cycles with an
overshoot that can leave the pack lower than a weaker heater would.

**Recommendation (RID THM-02, flight software, for the chair):** `heater::ON_BELOW_C` 2.0 → **5.0**,
`OFF_ABOVE_C` 6.0 → **9.0**. With 0.95 W (14.0 V) the minimum is then +4.4 °C nominal, +4.9 °C at the design mount
and +1.2 °C at 0.10 W/K, at 40–59 % duty. The energy cost is ≈ 0.1 W orbit average (POWER_BUDGET_ANALYSIS §6). The
hardware cutout stays at +40 °C.

### 6.2 Heater failure cases

Failed off (C9): the pack reaches −2.8 °C in the cold case, so the charge regulator refuses charge at sunrise until
the pack warms (fail-safe: no lithium plating), but SYS-602 is lost for part of each orbit near β = 0. Failed on
(H9): 1.37 W at a full pack raises it to 49.5 °C in the hot case, above the 45 °C charge limit; the series +40 °C
hardware cutout (BUILD_VS_BUY §4.3) is the control and must be in the heater circuit drawing and its test
(BATTERY_SAFETY_ASSESSMENT §4).

### 6.3 Jetson heat path

At the BASELINE duty (120 s bursts at 8 W, 10 % of the orbit) the module and spreader node rises 5 K per burst at
0.44 W/K and 4 K at 1.0 W/K: the node's 220 J/K, not the path, sets the burst rise. A 600 s batch (which the payload
schedule of the simulation allows, SIL-01) gives 39.0 °C at 0.44 W/K and 35.3 °C at 1.0 W/K. TB-07's "77 °C for a
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
cold case that cuts the heater's margin like a 0.10 W/K mount does (C7: +1.8 °C at 26 % duty, but the hot side of
the cycle rises to 13.6 °C). The finish is a drawing requirement (BUILD_VS_BUY §4.3) and must be inspected.

### 6.6 Cold boards, SYS-600/601 and the LEOP cold case (RID THM-04, Version 2.2)

SYS-600 ("all components shall operate within −20 … +50 °C") and SYS-601 (survive −40 … +60 °C) are not per-unit
allowables and cannot be met with the 10 °C margin by the boards. Version 2.1 proposed per-unit values without deriving
them; the round-3 panel reopened THM-04 on the LEOP cases (L3: OBC −24.0 °C, RF −24.8 °C; Jetson −17.6 °C in L1 and
−19.1 °C after the 3 h soak). Case L4 (§4.3), the cold case for the burns, is colder still.

**Allowables from the parts.** The lowest data-sheet minimum on each unit (board BOMs
`hardware/electrical/*/*_bom.csv`, board design documents):

| Unit | Parts at the lowest rating | Unit minimum from the parts | Exceptions |
|---|---|---|---|
| OBC board | STM32L4R5ZIT6, FM25V20A, RV-3028-C7, M-G370PDT0, BMI088, 32.768 kHz crystal: −40 … +85 °C; the rest AEC-Q100/Q200 grade 1 or −55 °C; connectors −40 … +105 °C | −40 °C | — |
| EPS board | BQ76952, BQ771806, TQ2SA relay, 2920L PTC: −40 … +85 °C; MSP430FR2355 −40 … +105 °C; the rest AEC-Q100/Q101/Q200 | −40 °C | ITV4030 fuses F1, F3: −10 … +65 °C (`EPS_BOARD_DESIGN.md` §14, V-EPS-8, EPSB-05) |
| Magnetometer, A3 boards | RM3100 and its coils −40 … +85 °C | −40 °C | — |
| RF board | CC1200, CMX138A, STM32L431, LMX2571, PCM1808, AD8361, TA2209A SAW, TG2016SMN TCXO: −40 … +85 °C; logic and regulators −40 … +125 °C | −40 °C | Y451 (ABLS 3.6864 MHz crystal): the BOM leaves the grade unconfirmed; the −40 … +85 °C variant must be the one ordered |
| Jetson carrier | N2600 NVMe −40 … +85 °C; the rest −40 … +105/125 °C | −40 °C | — |
| Jetson module | Orin Nano: junction −25 … +105 °C (DS-11105-001, `CARRIER_BOARD_DESIGN.md` §2 item 5); no non-operating range below it in the project's reading | −25 °C | Below −25 °C only by qualification test (V-THM-8) |
| Battery | INR18650-35E | §3 | Heater-controlled |

The protoflight test runs at the AFT ± 10 °C (ETP §3.4, SYS-707), and a test plateau should not take a part below its
data-sheet minimum, so a unit whose parts stop at −40 °C has a cold AFT of **−30 °C**, operating and non-operating
alike (the harness connectors' −40 °C is their only rating, and the ETP non-operating plateau is −40 °C at the
structure). That is the Version 2.1 operating value; its −40 °C non-operating value would have needed a −50 °C plateau.

**Against −30 °C with the 10 °C margin:** the orbit cold cases pass (−11.7 °C → −21.7 °C, C12; wings stowed −18.5 °C →
−28.5 °C, C14); the dispenser soak passes (3 h, unpowered: −19.1 °C → −29.1 °C); L1 passes (−27.6 °C); **L3 fails by
4.0 (OBC/EPS) to 5.1 °C (Jetson) and L4 by 7.6 °C (OBC/EPS), 8.6 °C (RF) and 8.9 °C (Jetson, off)**, and after a 3 h soak L4 reaches −29.1,
−30.0 and −30.3 °C, that is −40 °C with the margin: the parts' rating. The excursion is one transient: in L4 the OBC
is below −20 °C for 37 min, the RF board for 40 min, the Jetson below −25 °C for 21 min, in the first orbit only.

**Design changes evaluated (L4):**

| Change | OBC/EPS | RF | Jetson | Cost | Verdict |
|---|---|---|---|---|---|
| None | −27.6 | −28.6 | −28.9 | — | — |
| Stack heater 1 W, on below −15 °C | −24.8 | −26.9 | −27.5 | 0.9 Wh | Insufficient |
| Stack heater 3 W, thermostat | −19.1 | −23.5 | −24.6 | 2.5 Wh | RF still 3.5 °C short |
| 3 W stack and 2 W RF heaters, on | −17.3 | −17.6 | −22.0 | 7.8 Wh in the first orbit, 16 % of the 49.7 Wh pack, while the pack (−8.9 °C at separation) cannot charge for about 80 min; two new switched heaters | Closes; rejected |
| RF board kept off until warm (power-on delay) | −27.6 | colder | −28.9 | — | Off, it is non-operating at the same −30 °C AFT: no gain. The OBC/EPS stack cannot wait (separation timer, burns, detumble) |
| Board detumble power (1.16 W coils for the 0.8 h of the simulated detumble, §2.3), not credited | −24.8 | −26.9 | −27.5 | — | A low tip-off rate ends it early |
| EM correlation (margin 5 °C) | −32.6 with margin | −33.6 | −33.9 | V-THM-1 | Does not close alone |

The shortfall is set by the separation temperature: from the bounding 2 h soak (SpaceX interface bound used as a
strongly coupled wall, §6.7) the first eclipse takes the boards down by a further 10–11 K. A heater large enough to
cover it costs more of the LEOP energy than the detumble itself, and the parts are rated to −40 °C.

**Recommendation (THM-04; change request raised by the programmatics owner):**

1. Replace SYS-600/601 by per-unit AFTs (THERMAL_DESIGN §1.1): boards and Jetson carrier −30 … +50 °C operating and
   −30 … +60 °C non-operating; Jetson module −15 °C (power-on gate, SRS-F131) … +50 °C at the spreader operating
   (SYS-603 for the junction), −30 … +60 °C non-operating; battery as §3; structure −60 … +100 °C; wings and body
   panels the vendor's qualified range (≥ −56 … +93 °C, V-THM-4).
2. **First orbit after separation:** the boards' operating and the Jetson's non-operating AFT extend to **−40 °C**, the
   parts' rating, demonstrated by a cold start at −40 °C (EM, V-THM-8; FM, the ETP cold start moved from −30 °C to
   −40 °C). The L4 minimum with the margin (−38.6 °C; −40.0 °C after 3 h) is inside it but without the 10 °C test
   margin beyond it: a GEVS test-margin deviation for the first orbit only, for the chair, to be recorded in ETP §9.
   Rationale: one transient of ≤ 46 min below −20 °C, from a bounding soak, with parts rated to −40 °C and the
   detumble's 1.2 W uncredited.
3. Actions: Y451 in its −40 … +85 °C grade (RF BOM); V-EPS-8 (ITV4030) at −40 °C instead of −30 °C; the Jetson module's
   non-operating soak at −40 °C (V-THM-8); THM-06 (deployment within 2 h of the soak) stays, since at 3 h the margined
   minimum reaches the −40 °C rating; the ETP cold operational plateau becomes −40 °C (AFT − 10 °C, was −30 °C).
4. The deviation lapses if the integrator's dispenser analysis (V-THM-5) puts the boards and the Jetson at or above
   about −8 °C at separation (−13 °C after correlation), which brings L4 inside −30 °C with the margin.

**Proposed SYS-600 text:** "Each unit shall operate within its operating allowable flight temperatures (AFT) of
`hardware/structure/THERMAL_DESIGN.md` §1.1: the OBC/EPS stack with the ADCS and GNSS units and the magnetometer board,
and the RF board, −30 °C to +50 °C, and down to −40 °C during the first orbit after separation; the battery 0 °C to
+45 °C charging and −10 °C to +60 °C discharging (SYS-602); the Jetson module and carrier −15 °C (power-on gate) to
+50 °C at the heat spreader, with SYS-603 for the junction; the solar panels and wings over the panel vendor's
qualified range, at least −56 °C to +93 °C. Verified by analysis, the predictions plus 10 °C before engineering-model
correlation (5 °C after) lying inside the AFT, and by test: protoflight thermal vacuum at the AFT ± 10 °C capped at
+60 °C (SYS-707), and for the first-orbit extension a cold start at −40 °C, tested at its AFT without the 10 °C test
margin (GEVS deviation, ETP §9)."

**Proposed SYS-601 text:** "Each unit shall survive unpowered, without degradation, its non-operating AFT of
`hardware/structure/THERMAL_DESIGN.md` §1.1: the OBC/EPS stack, the RF board, the magnetometer board, the Jetson carrier
and the Jetson module −30 °C to +60 °C, and down to −40 °C during the first orbit after separation (the Jetson module
qualified below its data sheet's −25 °C junction minimum by the engineering-model soak, V-THM-8); the battery −20 °C to
+60 °C; the structure −60 °C to +100 °C. Verified as SYS-600, with the non-operating plateaus at −40 °C and +60 °C (the
battery cap, ETP §9 deviation 1)."

### 6.7 Dispenser soak

SpaceX gives a bounding conductive interface temperature at its side of the dispenser mount (UG v11 §4.1.11,
Table 4-11), not a dispenser or CubeSat temperature, and states it does not envelope payload temperatures. Treating
it as the dispenser wall with strong coupling is a bound. In that bound the pack exceeds +60 °C 2.3 h after lift-off
and the boards exceed SYS-601's +60 °C; in the cold bound the pack is at its −10 °C discharge limit at 2 h. The
isolated mount is what keeps the pack 10 K behind the boards. What closes this is the integrator's dispenser thermal
analysis for the assigned Transporter flight and slot, with the deployment time (RID THM-07, verification item V-THM-5).
Until then: deliver the pack at the storage state of charge (BATTERY_SAFETY_ASSESSMENT §6), and do not give the BMS a
discharge cut-off at −10 °C (EPS-08), or a cold deployment would leave the bus dark until the Sun warms the pack.

### 6.8 Wings stowed: heater at 100 % duty (RID THM-03, Version 2.2)

With both wings stowed (both hold-downs failed after every attempt: P = 0.0029, ODAR §6.5) the cold case C14 (β = 0,
cold environment, SAFE, sun-pointed) keeps the heater on all orbit, so the pack's temperature is set by the heater's
power, V² / 207 Ω, that is by the bus voltage. Sweep (475 km, last of 25 orbits):

| Bus (V) | 12.0 | 12.4 | 12.8 | 13.2 | 13.6 | 14.0 | 14.4 | 14.8 |
|---|---|---|---|---|---|---|---|---|
| Heater (W) | 0.70 | 0.74 | 0.79 | 0.84 | 0.89 | 0.95 | 1.00 | 1.06 |
| Battery minimum (°C), duty 100 % | −1.5 | −0.7 | +0.1 | +0.9 | +1.7 | +2.6 | +3.5 | +4.4 |

EMERGENCY (beacon shed) is within 0.1 °C of SAFE; tumbling with the wings stowed (C6) is warmer, +4.1 °C at 0.70 W and
53 % duty; the real coldest orbit (LTAN 23:30, June, cold environment) is −1.7 °C at 0.70 W. The pack is at or above
0 °C from 0.79 W (12.8 V), and above the hardware charge-inhibit threshold with its tolerance (+1.1 °C + 1 K,
SQSO-EPS-BRD-001 §5.3) from about 0.92 W (13.8 V).

**Energy.** The generation with the wings stowed is 1.47 W delivered (BASELINE §5; worst 30 days of the second year,
sun-pointed); the bounding case's own array (β = 0, 1322 W/m², end of life) gives 1.40 W at the maximum power points,
about 1.33 W delivered. SAFE takes 0.52 W before the heater (POWER_BUDGET_ANALYSIS §8.1), which leaves 0.95 W (as
flown) or about 0.81 W (bound) for the heater, orbit average. Because the heater draws V² / R, the bus settles where
generation and load balance: about 14.0 V with 1.47 W (heater 0.95 W, pack **+2.6 °C**, charging permitted all orbit,
no energy margin), about 12.9 V at the bound (0.81 W, pack about +0.3 °C, charge refused while the pack is below
+1.1 °C). The 12.0 V point (−1.5 °C) is reached only from an energy deficit, with a pack already near empty. A
lower-resistance heater does not help: at the balance the heater power is the surplus whatever its resistance; only a
lower load or a better-isolated pack (the design mount, ≈ 0.03 W/K, V-THM-2) moves the pack temperature.

**At that power, with the loads shed to SAFE, the heater holds the battery at or above 0 °C** with the as-flown
generation (+2.6 °C at about 14.0 V), with no energy margin, and not with certainty at the bound. Below +1.1 °C the
charge path is refused in hardware (two independent layers, SQSO-EPS-BRD-001 §5.3), which is fail-safe: no charge
below 0 °C, no lithium plating, SYS-602 holds. The array still supplies the bus in sunlight (§5.2 there). What is lost:
the recharge during the cold part of each orbit, so the pack settles at a low state of charge near the EMERGENCY
threshold (14.0 V), with the transponder and the Jetson off (the wings-stowed rule already holds them off) and, in
EMERGENCY, the beacon shed until the season or the attitude raises the generation.

**Recommended disposition: accept as a contingency**, with (a) a flight-software rule: with the wings stowed, hold
SAFE (not IDLE) while the battery's minimum over the last orbit is below +3 °C, so the heater gets the whole surplus
(IDLE takes about 0.4 W more, POWER_BUDGET_ANALYSIS §8.1); (b) the stowed points of V-THM-1 to correlate the case; no
hardware change.

## 7. Thermal control hardware (CDR baseline)

| Element | Specification | Source |
|---------|---------------|--------|
| External finishes | Long faces: cells (α 0.92, ε 0.85) and black anodize between; ends black anodize (α 0.88, ε 0.85) | THERMAL_DESIGN §3.1 |
| Battery mount | 4 × M2.5 G-10CR or PEEK 450G shouldered bushings, Ti-6Al-4V screws, flying leads ≥ 100 mm, ≈ 0.03 W/K to the frame, ≤ 0.01 W/K to the stack | BUILD_VS_BUY §4.3 |
| Battery exterior | ε ≤ 0.10 (VDA tape or bare aluminium) | BUILD_VS_BUY §4.3; §6.5 here |
| Battery heater | ≈ 207 Ω polyimide film, two elements on different cells: 0.70 W at 12.0 V, 0.95 W at 14.0 V, 1.30 W at 16.4 V; thermostat +5/+9 °C (SRS-F130); series hardware cutout +40 °C; fuse or current limit | §2.3, §6.1; BATTERY_DESIGN §6.3 |
| Jetson path | ≥ 1 W/K module to frame, measured on the EM | BASELINE §6; SYS-603 |
| Temperature sensors | Two battery NTCs, OBC, RF/PA (TX_TEMP), Jetson spreader, one per long panel (the panel vendors build thermistors in) | THERMAL_DESIGN §10 |

## 8. RIDs raised by this analysis

The full RID table is in the CDR reviewer report; the thermal RIDs closed or opened here:

| RID | Title | Status |
|-----|-------|--------|
| THM-01 | Thermal analysis v1.0 was for the wrong orbit and computer, with no model | Closed by analysis (this document) |
| THM-02 | Heater thermostat +2/+6 °C leaves too little margin to the 0 °C charge limit | For the chair: flight-software constants (§6.1) |
| THM-03 | Wings-stowed cold case saturates the heater | Version 2.2: contingency accepted by analysis (§6.8: at the stowed generation the bus settles near 14.0 V and the pack at +2.6 °C, no energy margin; below +1.1 °C charging is refused in hardware); flight-software rule (SAFE, not IDLE, while the stowed pack's orbit minimum is below +3 °C) for the chair; V-THM-1 stowed points |
| THM-04 | SYS-600/601 cannot be met by the boards with the 10 °C margin | Version 2.2: per-unit AFTs derived from the parts (§6.6): −30 °C cold for the boards; first-orbit extension to −40 °C with a GEVS test-margin deviation for the chair; SYS-600/601 text for the change request (§6.6); V-THM-8, V-EPS-8 at −40 °C, Y451 grade |
| P3S-01 | Mechanism cold design and test point (−40 °C) warmer than the predicted stowed-wing temperature | Version 2.2: closed by analysis (§4.3): at every burn the margined wing, hinge, snubber and burn-resistor temperatures are inside the −55 °C point of SQSO-MECH-001 Version E by ≥ 3.3 °C; the line's free span (−81 °C with the margin) is for the preload check |
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
| V-THM-6 | Protoflight thermal vacuum and cycling | ETP Rev D §3.4: 8 thermal-vacuum cycles at AFT ± 10 °C capped at +60 °C, plus 4 board-level cycles and, if the integrator requires 20, 8 ambient cycles (VI-08); the SpaceX ± 5 °C range is enveloped except at the battery cap (deviation 4) | SYS-600/601 or their replacement | FM |
| V-THM-7 | Heater circuit: thermostat, cutout at +40 °C, fuse | Cutout opens at 40 ± 3 °C with the heater forced on; heater current at 12.0 and 16.4 V | §6.2 | EM, FM |
| V-THM-8 | EM cold start and Jetson non-operating soak (THM-04, §6.6) | After ≥ 2 h unpowered at −40 °C, the OBC/EPS stack, RF board and Jetson carrier power up at −40 °C and pass their functional tests (RF: receiver sensitivity, TCXO within ±1 ppm; EPS: the ITV4030 per V-EPS-8 at −40 °C); the Jetson module, soaked unpowered at −40 °C for 3 cycles, boots at the −15 °C gate and passes its functional test | THM-04, SYS-600/601 first-orbit extension | EM, before FM build |

## 10. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 2.0 | 2026-10-27 | CDR review, EPS/thermal | Rewritten for the baseline orbit and hardware on the simulation's 12-node network (battery radiation added): hot, cold, dispenser and LEOP cases with margins (finding TB-07) |
| 2.1 | 2026-10-27 | CDR flight software, round 3 | §4.1 new: the cases re-run at 475 km (BASELINE Rev E) with the 2028-11-01 launch date and the +5/+9 °C set points, and the deployment case L3 (2028-11-01, LTAN 22:30, from the cold soak: boards −24 to −25 °C); §2.2 orbit row |
| 2.2 | 2026-10-27 | CDR thermal analyst, round 3 | Panel reopenings P3S-01, THM-03, THM-04: §4.3 new, LEOP with the wings stowed until the burn, separation phase swept, hinge, burn-board and line sub-nodes; burn temperatures for the mechanisms (wing −40.6 °C, hinge −30.2 / −38.8 °C, resistor −29.4 °C at separation + 30 min; inside −55 °C with the margin: P3S-01 closed by analysis); case L4 (boards −27.6 / −28.6 °C, Jetson −28.9 °C); §6.6 per-unit allowables from the board BOMs, design changes evaluated, first-orbit −40 °C extension with a test-margin deviation, SYS-600/601 text for the change request; §6.8 wings-stowed heater sweep and energy balance, contingency accepted; §4.2 heater 0.70 W at 12.0 V (207 Ω; was 0.72 W): C11 +3.6 °C, C15 0.0 °C, C14 −1.5 °C; §1, §2.1, §2.3, §4.1 extremes, §5, §7, §8 updated; V-THM-6 restated to ETP Rev D §3.4; V-THM-8 new |
| 2.3 | 2026-10-27 | EEE parts engineer (chair action, RID P3T-01) | §3 allowables aligned with THERMAL_DESIGN 1.6 §1.1 (−30 °C AFT, −40 °C first orbit; Jetson −15 °C gate) and the parts audit SQSO-MFG-003 A; rows for the EPS fuses F1/F3, the RF PA Q601, the ISIS AntS (not modelled) and the CR0002 rods; BQ76920 corrected to BQ76952 |
