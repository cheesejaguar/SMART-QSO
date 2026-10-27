# SMART-QSO Power Budget Analysis

**Document ID**: SQSO-PBA-001
**Version**: 2.1
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR power closure (analysis by simulation; nothing measured). Replaces version 1.x, whose array
(0.06 m², 12.3 W peak), 3.3 V bus and loads were superseded (findings TB-02, TB-08, TB-14).

`docs/BASELINE_PARAMETERS.md` governs. This document replaces the orbit-average allocation of BASELINE §5 by
simulated values, as BASELINE §5 asks ("to be replaced by a multi-orbit SOC simulation"), and settles the open
"tumbling margin +19 % against 20 %" note. The margin requirement is SYS-100 (≥ 2.16 W EOL orbit average tumbling:
the 1.80 W tumbling load plus 20 %).

## 1. Summary

| Case (end of life, worst 30 days of a year) | Generation delivered to the bus | Load (mode, at the pack) | Margin | Requirement |
|---------------------------------------------|---------------------------------|--------------------------|--------|-------------|
| ACTIVE, sun-pointed, BASELINE duty limits (transponder 25 %, Jetson 10 %) | 3.93 W | 2.50 W | **+54 %** | ≥ 20 %: met |
| ACTIVE, tumbling (ADCS failed), duty limits (Jetson 5 %) | 2.43 W | 1.82 W | **+31 %** | met |
| ACTIVE, tumbling, isotropic bound (60 °C cells, no albedo) | 2.14 W | 1.82 W | +15 % | not met (bound only, §9) |
| BASELINE §5 tumbling allocation (1.80 W) | 2.43 W | 1.80 W | **+32 %** (+35 % by the BASELINE method) | met |
| IDLE, sun-pointed | 3.93 W | 1.18 W | +228 % | met |
| SAFE (ADCS off, tumbling) | 2.43 W | 0.79 W | +203 % | met |
| EMERGENCY (beacon shed, tumbling) | 2.43 W | 0.72 W | +230 % | met |
| SYS-100: 2.16 W EOL tumbling | 2.43 W | — | +12.5 % over the requirement | met |
| As flown by the flight software, a year at LTAN 23:30, EOL | 3.93 W available, 1.32 W used | — | +182 %; DoD ≤ 2.1 %; SOC ≥ 90.4 % | met |
| Wings stowed (ADCS working) | 1.47 W | SAFE 1.03 W / IDLE 1.18 W / ACTIVE 2.50 W | +40 % / +23 % / **−42 %** | ACTIVE not sustainable; the SOC gates hold it (§8.1) |
| Wings stowed and ADCS failed | 1.41 W | SAFE 0.79 W | +76 % | positive |
| Jetson rail stuck on (both switches failed) | 3.93 W | 9.4 W | **−5.7 W** | Loss of the bus in ≈ 41 h (simulated); needs a hardware load shed (RID EPS-09) |
| LEOP: detumble with the magnetorquers at 1.3 W, cold pack, heater on (§8.6) | 2.43 W (tumbling EOL, conservative) | 3.03 W while detumbling | first orbit −1.30 Wh; 3-orbit bound −3.44 Wh | SOC ≥ 33 % from the 40 % floor: met |

All modes close with at least 20 % at end of life and at the β extremes of the LTAN 22:30 ± 1 h dispersion, with the
transponder and the Jetson at their BASELINE duty limits, except ACTIVE with the wings stowed (contingency) and a
stuck-on Jetson (failure). The battery is never the limit: the deepest discharge of any run, failures included, is
4.5 % per eclipse (limit 10 % nominal, 30 % contingency).

## 2. Method

1. **Simulation.** `sqso_sil` (SQSO-SIL-001) flies the flight software against orbit, attitude, array, EPS, battery
   and thermal models. Runs: a year at each LTAN of the dispersion (21:30, 22:30, 23:30) with a new array; a year at
   22:30 and 23:30 with the array aged one year at deployment (so the year covers the second year, end of life); a
   year at 23:30, end of life, with the ADCS failed; 14-day runs for the contingencies. Run matrix:
   `software/simulation/cdr/power_thermal/run_matrix.txt`; analysis: `power_closure.py` beside it. The year and
   contingency runs used the simulation as of 2026-10-27 before the battery radiation term was added to the thermal
   model (SQSO-THA-001 §2.1); that term changes the pack temperature by ≤ 0.4 °C and the energy balance by nothing
   measurable (the 14-day commissioning run with it: delivered 4.18 W, used 1.40 W, 0 anomalies).
2. **Generation.** The simulation's array power at the maximum power point (strings' terminals, with harness and
   mismatch 0.97, UV 0.97–0.99, radiation, temperature from the thermal model, albedo and shadowing), time-averaged
   over each 30-day window after LEOP. Delivered to the bus: × 0.97 (blocking diodes) × 0.92 (MPPT) = × 0.892, the
   simulation's EPS chain (`hw/eps.rs`). The design chain (SOLAR_ARRAY_DESIGN §9.1: MPPT 0.95, no second diode term)
   would give 3.4 % more; it is not credited.
3. **Loads.** Bottom-up by mode from the simulation's device values (§5) at the BASELINE §5 duty limits, at the pack
   (converter losses included), plus the heater energy from the thermal analysis (SQSO-THA-001 v2.0, cold case at
   16.4 V: 0.51 W sun-pointed, 0.27 W tumbling, with the recommended +5/+9 °C set points) and a 5 % battery
   round-trip loss on the eclipse share (38 %).
4. **Margin** = generation delivered / (load × round-trip factor) − 1. The BASELINE method (generation / load − 1)
   is 3 points higher.

## 3. Orbit, β angle and eclipse

Simulated at 500 km sun-synchronous with deployment on 2028-06-01, the simulation's orbit when these runs were made.
The baseline is now a 475 km target (465–485 km) and a planning launch of about 2028-11 (BASELINE Rev E §3), which the
simulation has since adopted. At 475 km the longest eclipse is about 0.3 % of the orbit longer (≈ 0.5 % less
generation), well inside the margins of §6; V-PWR-4 re-runs the matrix. The simulated β range over a year and the
longest eclipse fraction:

| LTAN | β over a year | Longest eclipse | Note |
|------|---------------|-----------------|------|
| 21:30 (−1 h) | 29.2° … 40.9° | 36.4 % (34.4 min) | Hottest; most sun |
| **22:30 (baseline)** | 16.0° … 26.4° | 37.7 % (35.7 min) | |
| 23:30 (+1 h) | 2.4° … 11.9° | 38.2 % (36.1 min) | Coldest; least sun |

β never exceeds 41°, so there is always an eclipse (no eclipse needs β > 68° at 500 km). The BASELINE §8 hot case
(no eclipse) is a bound, not an orbit this mission flies. LTAN drift over two years from an inclination injection
error was not modelled; ±1 h covers the provider's stated dispersion.

## 4. Generation (simulated)

Orbit-average array power at the maximum power point (MPP) and delivered to the bus (× 0.892), operations only:

| Run | Year mean MPP | Worst 30-day MPP | **Worst 30-day delivered** | Sunlit mean angle of +Y to the Sun |
|-----|---------------|------------------|----------------------------|-----------------------------------|
| Sun-pointed, BOL (year 1), LTAN 21:30 | 4.78 W | 4.71 W | 4.20 W | ≈ 2–3° |
| Sun-pointed, BOL, LTAN 22:30 | 4.72 W | 4.52 W | 4.03 W | |
| Sun-pointed, BOL, LTAN 23:30 | 4.71 W | 4.52 W | 4.03 W | |
| Sun-pointed, EOL (year 2), LTAN 22:30 | 4.58 W | 4.41 W | **3.93 W** | |
| Sun-pointed, EOL, LTAN 23:30 | 4.57 W | 4.41 W | **3.93 W** | |
| Tumbling (ADCS failed, free tip-off spin), EOL, LTAN 23:30 | 2.88 W | 2.72 W | **2.43 W** | 90° (random) |
| Tumbling, EOL, LTAN 22:30, 14 days | 2.72 W | — | 2.43 W | 90° |
| Wings stowed, EOL, sun-pointing working | 1.65 W | — | 1.47 W | 58° |
| Wings stowed and ADCS failed, EOL, LTAN 22:30 / 23:30 | 1.58 / 1.63 W | — | 1.41 / 1.45 W | 90° |

Against the earlier analysis (SOLAR_ARRAY_DESIGN §9.3, 500 km, β = 0, cells at an assumed 60 °C, no albedo): sun-pointed
3.59 W and isotropic tumbling 2.14 W after an MPPT of 0.95. The simulation is higher mainly because the thermal model
puts the body cells at −15 … +38 °C and the wings at −40 … +80 °C (THERMAL_ANALYSIS §4) instead of 60 °C (≈ +10 %
for the body cells), and because it counts albedo (≈ +5 %). The seasonal swing is ±4 % (perihelion in January).

## 5. Loads by mode

Device values from the simulation (`sqso-sil/src/mission.rs`, `hw/radio.rs`, `hw/eps.rs`, `jetson.rs`): OBC with FRAM
and sensors 0.165 W and the receiver 0.15 W on 3V3_AO (buck 85 %), EPS quiescent 0.08 W; ADCS sensors 0.12 W
(magnetorquers ≈ 0 W in the spin hold, measured in the simulation; 1.16–1.3 W while detumbling, §8.6); PA 1.5 W DC at 0.5 W RF on 5 V (buck 90 %);
beacon 1.67 s on air per frame; GNSS at its BASELINE allocation 0.05 W (not modelled in the simulation); Jetson 8 W.
The transponder and Jetson duty limits are orbit fractions (BASELINE §5).

| Item (W at the pack) | EMERGENCY | SAFE | IDLE | ACTIVE, sun-pointed | ACTIVE, tumbling |
|----------------------|-----------|------|------|---------------------|------------------|
| OBC, EPS, receiver (always on, 97.207(b)) | 0.451 | 0.451 | 0.451 | 0.451 | 0.451 |
| Beacon (period) | — (shed) | 0.015 (180 s) | 0.023 (120 s) | 0.046 (60 s) | 0.046 |
| ADCS | off | off | 0.141 | 0.141 | 0.141 |
| GNSS (allocation) | — | 0.050 | 0.050 | 0.050 | 0.050 |
| Transponder, 25 % duty | — | — | — | 0.417 | 0.417 |
| Jetson, 10 % / 5 % duty | — | — | — | 0.889 | 0.444 |
| **Subtotal** | **0.451** | **0.516** | **0.665** | **1.994** | **1.549** |
| Heater (cold case, 16.4 V, +5/+9 °C) | 0.27 | 0.27 | 0.51 | 0.51 | 0.27 |
| **Total** | **0.72** | **0.79** | **1.18** | **2.50** | **1.82** |
| BASELINE §5 allocation | — | — | — | 2.20 | 1.80 |

The bottom-up ACTIVE load is below the BASELINE allocation before the heater (1.99 W against 2.20 W; 1.55 W against
1.80 W). The allocation's housekeeping line (0.70 W) holds 0.11 W more than the bottom-up housekeeping (0.59 W with the
ADCS) but nothing for the heater, which the BASELINE lists in that line; with the heater the bottom-up loads are
0.30 W (sun-pointed) and 0.02 W (tumbling) above the allocation.

**Peak currents at the pack** (for fuses and harness, EPS_DESIGN §5; SQSO-EPS-BRD-001 §6.1, §9): housekeeping 0.05 A;
PA keyed at 1 W RF 0.21 A; Jetson 8 W 0.62 A (12 W latch-up guard trip 0.93 A); heater 0.08 A at 16.4 V;
magnetorquers 0.083 A with three rods saturated at 14.4 V (0.104 A at 11.2 V; 0.23 A at the bridges' hardware limit);
one burn resistor 0.51 A for ≤ 12 s (0.8 A at the resistor from the 7.0 V burn buck). Worst simultaneous credible, in
LEOP (the Jetson and the transponder are off): burn + housekeeping + heater 0.67 A, **0.75 A** with the magnetorquers
detumbling (0.90 A with the bridges at their limit), against the 1.0 A limit for a burn; four drivers on at once (a
software fault) are held to 2.0 A by the burn buck. Outside LEOP the largest legitimate load is 1.40 A (Jetson at the
guard, PA at 1 W, housekeeping, heater, magnetorquers).

## 6. Mode-by-mode closure

End of life, worst 30 days, LTAN dispersion extremes (generation from §4, loads from §5):

| Mode | Attitude flown | Generation | Load | Margin |
|------|----------------|------------|------|--------|
| EMERGENCY | Tumbling (ADCS off) | 2.43 W (2.14 W bound) | 0.72 W | +230 % (+191 %) |
| SAFE | Tumbling (ADCS off) | 2.43 W (2.14 W bound) | 0.79 W | +203 % (+167 %) |
| IDLE | Sun-pointed | 3.93 W | 1.18 W | +228 % |
| ACTIVE | Sun-pointed | 3.93 W (BOL 4.03 W) | 2.50 W | **+54 %** (BOL +58 %) |
| ACTIVE | Tumbling (ADCS failed) | 2.43 W (2.14 W bound) | 1.82 W | **+31 %** (+15 % bound) |

SAFE and EMERGENCY switch the ADCS off (`sqso_fsw::eps::loads`), so they are closed on the tumbling generation. The
spin left when the coils stop keeps +Y near the Sun for a while (the residual-dipole torque precesses it by about a
degree a minute at the 3 mA·m² design value), so this is conservative.

**Heater energy and the recommended set points.** Raising the thermostat from +2/+6 °C to +5/+9 °C (THERMAL_ANALYSIS
§6.1) costs ≈ 0.1 W orbit average in the cold case; it is included above. The simulation's 0.5 W heater at +2/+6 °C
averaged 0.17–0.29 W over 30-day windows.

## 7. As flown (flight software in the loop)

The flight software runs the transponder on demand and the Jetson in 10-line batches, well below the duty limits.
Year runs (operations; heater 0.5 W at +2/+6 °C, the simulation's values):

| Run | Delivered (year mean) | Used | Margin, worst 30 d | Max DoD | Lowest SOC | Battery | Heater duty, worst orbit | Charge refused |
|-----|-----------------------|------|--------------------|---------|------------|---------|--------------------------|----------------|
| BOL, LTAN 21:30 | 4.26 W | 1.35 W | +186 % | 2.1 % | 90.4 % | 0.6 … 9.0 °C | 58 % | 0 s |
| BOL, LTAN 22:30 | 4.21 W | 1.32 W | +193 % | 2.1 % | 90.4 % | 0.5 … 8.8 °C | 58 % | 0 s |
| BOL, LTAN 23:30 | 4.20 W | 1.33 W | +188 % | 2.1 % | 90.4 % | 0.4 … 8.8 °C | 57 % | 0 s |
| EOL, LTAN 22:30 | 4.09 W | 1.31 W | +187 % | 2.1 % | 90.4 % | 0.6 … 8.9 °C | 56 % | 0 s |
| EOL, LTAN 23:30 | 4.08 W | 1.32 W | +182 % | 2.1 % | 90.4 % | 0.6 … 8.9 °C | 55 % | 0 s |
| EOL, LTAN 23:30, ADCS failed | 2.57 W | 1.16 W | +102 % | 3.8 % | 88.2 % | 2.5 … 15.1 °C | 0 % | 0 s |

Every run reported 0 anomalies except the injected failure's own check (detumble never completes). The lowest SOC
(90.4 %) is the 4.10 V/cell set point (D8). The battery's coldest point, +0.4 °C, is the thin heater margin of
THERMAL_ANALYSIS §6.1, not an energy limit.

## 8. Contingencies

### 8.1 Wings not deployed

1.47 W delivered with the sun pointing working. SAFE (+40 %) and IDLE (+23 %) close; ACTIVE at the duty limits does
not (−42 %). In the 14-day run the flight software stayed in ACTIVE and the SOC gate (transponder and Jetson only at
SOC ≥ 55 %) held the pack between 57 % and 63 %: as-flown margin +7 %, DoD 2.6 %. That is a limit cycle on the ACTIVE
gate, not a closed budget. In the bounding cold case the heater alone needs 0.95 W at 100 % duty with the wings
stowed (THERMAL_ANALYSIS C16), which leaves 0.5 W: SAFE closes at 0 %. **Recommendation (RID EPS-10, flight software,
for the chair):** when the wing sense lines read "not deployed" after the retries, hold the mode at IDLE (transponder
and Jetson by telecommand only), so that the pack keeps a reserve for the heater.

### 8.2 ADCS failed (tumbling)

2.43 W delivered (worst 30 days, EOL, LTAN 23:30, β 2.5–12°). ACTIVE at the tumbling duty limits +31 %; as flown
+102 %; DoD 3.8 %; no heater needed (tumbling keeps the pack at ≥ 2.5 °C).

### 8.3 Wings and ADCS both failed

1.41–1.45 W. SAFE +76 %, EMERGENCY +92 %. As flown (the flight software stayed in ACTIVE on the SOC gate) +3 to +9 %,
SOC ≥ 56.7 %, DoD 3.3 %. The same EPS-10 rule applies.

### 8.4 Stuck loads

| Stuck load | Power at the pack | Balance in SAFE, EOL | Result |
|------------|-------------------|----------------------|--------|
| Jetson rail on (both series switches failed closed) | 8.9 W inferring; ≈ 3.9 W idle | −5.7 W sun-pointed, −7.2 W tumbling (inferring) | Simulated (the module idling, as the flight software never sends it work): SOC 58 % → SAFE at 31.8 h, EMERGENCY at 38.1 h, BMS under-voltage cut at 40.9 h, then 224 brown-out cycles in 14 days. Loss of mission. Mitigation: an independent hardware low-voltage load shed of the payload and PA rails (RID EPS-09) |
| Heater on (thermostat and switch failed) | 1.3 W | +0.58 W tumbling; −0.38 W with the wings stowed | Survivable except with stowed wings; the +40 °C cutout limits the temperature (THERMAL_ANALYSIS §6.2) |
| PA keyed continuously (PTT stuck, 0.5 W RF) | 1.67 W | +0.20 W tumbling | Energy-positive; the 120 s hardware key-down timer (BUILD_VS_BUY §3.2) and TX-OFF are the controls |
| GNSS continuous (0.125 W typical, 0.32 W worst, BASELINE §5 note) | 0.32 W | ACTIVE tumbling +31 % → +16 % | Within margin only at the typical draw; keep it duty-cycled |

### 8.5 Cold-soak deployment

With the whole spacecraft at −20 °C at separation, the deployment burn at 30 min (one resistor at a time, 0.75 A at
the pack with the magnetorquers detumbling) leaves the bus at ≥ 13.8 V at the ≈ 58 % deployment SOC (EMC_ANALYSIS §4;
12.2 V in the two-resistor bound) and the charge regulator refuses charge for the first 1.5 h (pack below 0 °C). The pack must leave the dispenser with ≥ 40 % SOC (BATTERY_SAFETY_ASSESSMENT §6).

### 8.6 Launch and early operations (separation to the end of detumble)

The year runs of §6–§7 start after LEOP. This case adds the loads that only LEOP has: the B-dot detumble with the
magnetorquers at full dipole, the burns, and the heater after a cold dispenser soak. The magnetorquers draw
**1.16 W** with all three rods saturated (0.39 W per rod at 0.2 A·m²) and **1.3 W** with the three bridges awake
(+0.12 W; `hardware/electrical/obc/OBC_BOARD_DESIGN.md` §10.1, §17); 1.3 W is used throughout. They run from 120 s
after separation (BASELINE §4) until detumbled: 0.8 h in the mission simulation, 1.4 h in the slowest Monte Carlo
case, 3 orbits as the requirement and the bound. The Jetson and the transponder are off in LEOP.

| Load at the pack, LEOP | Power | Note |
|------------------------|-------|------|
| OBC, EPS, receiver (always on) | 0.45 W | §5 (the receiver is off for the first 45 min: not credited) |
| ADCS sensors | 0.14 W | §5 |
| GNSS (every orbit in LEOP) | 0.05 W | allocation |
| Magnetorquers, detumble | **1.30 W** | new in this revision (was 0 in the LEOP figures) |
| Heater after the cold soak (207 Ω at ≈ 15 V, on through the first orbit) | 1.09 W | THERMAL_ANALYSIS §4: the pack reaches 0 °C after 33 min with it |
| Burns: four resistors at most, ≤ 12 s each, 6.1 W each at the pack from the burn buck | 0.08 Wh in all | SQSO-EPS-BRD-001 §9 |
| AntS deployment, 3V3_ANTS at 0.61 A, ≤ 60 s with retries | 0.04 Wh | |
| **Total while detumbling** | **3.03 W** (1.94 W without the heater) | |

| Interval | Load energy | Generation delivered | Balance | SOC from 58 % (49.7 Wh pack) |
|----------|-------------|----------------------|---------|------------------------------|
| Separation in eclipse, first sun after 34.5 min (worst case), cold pack | 3.03 W × 0.575 h = 1.74 Wh | 0 | −1.74 Wh | −3.5 points (54.5 %) |
| Same, warm pack (no heater) | 1.94 W × 0.575 h = 1.12 Wh | 0 | −1.12 Wh | −2.3 points |
| First orbit (1.58 h), detumbled after 1.4 h, cold pack, worst eclipse phasing | 0.64 W × 1.58 h + 1.30 W × 1.37 h + 1.09 W × 1.58 h + 0.12 Wh = 4.63 Wh | 2.43 W × 1.58 h − 0.51 Wh (wings stowed for the first 30 min) = 3.33 Wh, tumbling, EOL value used | −1.30 Wh | −2.6 points (55.4 %) |
| Bound: detumble over 3 orbits (4.75 h), heater on throughout | 0.64 W × 4.75 h + 1.30 W × 4.72 h + 1.09 W × 4.75 h + 0.12 Wh = 14.47 Wh | 2.43 W × 4.75 h − 0.51 Wh = 11.03 Wh | −3.44 Wh | −6.9 points (51.1 %); from the 40 % separation floor, 33.1 % |

The wings-stowed deficit is (2.43 − 1.41) W × 0.5 h = 0.51 Wh (§4). Generation is taken at the tumbling end-of-life
value although the array is new and the spacecraft sun-points once detumbled, so every line is conservative. The worst
LEOP balance, the 3-orbit bound from the 40 % separation floor, leaves 33 % SOC, above the SAFE threshold (25 %) and
the deepest-discharge limit for a contingency (30 % DoD); the first orbit costs at most 2.6 points. The magnetorquers
are the largest LEOP load after the heater: without them the first-orbit balance would be +0.48 Wh. The earlier LEOP
figures (`docs/operations/MISSION_TIMELINE.md` §5: magnetorquers ≤ 0.3 W average; worst case −0.25 Wh before first
sun) did not carry the detumble power.

## 9. The tumbling margin (BASELINE §5 "under review")

**Question.** With the wings' self-shadowing the isotropic-tumble analysis gave 2.14 W against the 1.80 W tumbling
allocation: +19 %, below the 20 % requirement.

**Answer.** The 2.14 W is a bound built on two assumptions the CDR analyses replace: cells at 60 °C (the thermal
model gives −15 … +27 °C on the body panels of a tumbling spacecraft, C1) and no albedo. The simulation, with the
same cell data, shadowing and degradation, gives 2.43 W delivered at end of life in the worst 30 days of a year at the
coldest LTAN of the dispersion: **+32 % against the 1.80 W allocation** (+35 % by the BASELINE method), +31 % against
the bottom-up tumbling ACTIVE load with the heater, and 12.5 % above SYS-100's 2.16 W. The requirement is met. The
residual risk is the attitude: the simulated tumble is the free spin left by the tip-off; an isotropic tumble at the
model's temperatures and albedo would give about 2.14 × 1.10 × 1.05 ≈ 2.47 W, consistent with it.

**Recommended BASELINE §5 text (for the chair):**

> | Generation, EOL orbit average delivered to the bus | **2.43 W tumbling; 3.93 W sun-pointed** (mission simulation, worst 30 days of the second year at LTAN 23:30; SQSO-PBA-001 v2.0 §4). The isotropic-tumble analysis (2.14 W, SOLAR_ARRAY_DESIGN §9.3, cells at 60 °C, no albedo) is kept as a bound |
>
> **Power closure (SQSO-PBA-001 v2.0 §6):** ACTIVE at the duty limits 2.50 W sun-pointed (+54 %), 1.82 W tumbling (+31 %); IDLE 1.18 W; SAFE 0.79 W; EMERGENCY 0.72 W, heater included. Margin ≥ 20 % in every mode at end of life and at LTAN 22:30 ± 1 h. Wings stowed: SAFE and IDLE only.

and delete the "Under review" paragraph below the table.

## 10. Sensitivities and model limits

| Item | Effect |
|------|--------|
| EPS chain: the simulation counts a 0.97 diode term on top of the strings' 0.97 harness factor and an MPPT of 0.92 | Conservative by 3.4 % against the design chain |
| One MPPT for strings at different temperatures (a COTS EPS: three trackers for eight strings) | −2.7 … −3.9 % on the sun-pointed face if the body and wing strings share a tracker; 0 % with separate trackers (`mppt_mismatch.py`; EPS_DESIGN §3) |
| Heater 1 W at +5/+9 °C instead of the simulation's 0.5 W at +2/+6 °C | +0.1 … +0.3 W orbit average; included in §5–§6 |
| Jetson 7 W mode unavailable for "Super" (TB-13) | The guard's 8 W input limit bounds it; the 10 % duty at 8 W is in §5 |
| Calendar and cycle fade | 1.4 % capacity fade in two years (SQSO-SIL-001 §7); not limiting |
| Attitude model | Free rigid body; no flexible modes; the tumbling case is one tip-off draw |

## 11. Verification items

| ID | Test | Pass criterion | Verifies | When |
|----|------|----------------|----------|------|
| V-PWR-1 | Mode power on the EM flatsat: each mode of §5 measured at the pack for ≥ 1 orbit-equivalent, including the Jetson burst and the transponder at 25 % | Each within 10 % of §5 | §5, SYS-100 | EM flatsat |
| V-PWR-2 | Solar panel I-V under AM0 simulator at 25 °C and hot; illuminated-EM shadowing check (SOLAR_ARRAY_DESIGN §10.2) | Pmp within 3 % of the cell model | §4 | Panel acceptance, EM |
| V-PWR-3 | MPPT efficiency over 3.5–6.3 V input, 0.05–0.5 A per channel, at −20, +25, +60 °C | ≥ 0.92 (the simulation's value) | §2 | EPS EM |
| V-PWR-4 | Re-run of this analysis with the measured V-PWR-1 to -3 values and the EM thermal correlation | All modes ≥ 20 % at EOL | SYS-100 | Before FM integration |
| V-PWR-5 | On-orbit SOC and energy trending during commissioning | Generation within 10 % of §4 for the measured attitude | §4 | Commissioning |

## 12. RIDs raised here

| RID | Severity | Title | Status |
|-----|----------|-------|--------|
| EPS-01 | 1 | Power closure not done at CDR; version 1.x used an impossible array | Closed by analysis (this document) |
| EPS-02 | 2 | BASELINE §5 tumbling margin "+19 % against 20 %" open | Closed by analysis (§9); BASELINE text for the chair |
| EPS-09 | 2 | A stuck-on Jetson drains the pack in < 2 days; no hardware load shed | For the chair / EPS design (EPS_DESIGN §6) |
| EPS-10 | 3 | Wings-stowed ACTIVE relies on the SOC gate limit cycle | For the chair (flight software, §8.1) |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | Funding audit | Notice: generation superseded by SOLAR_ARRAY_DESIGN §9 (TB-14); battery §2.2 and §5.4 corrected to BASELINE §5 (TB-08) |
| 2.0 | 2026-10-27 | CDR review, EPS/thermal | Rewritten: CDR power closure from the mission simulation (year runs at LTAN 21:30/22:30/23:30, BOL and EOL, contingencies), bottom-up mode loads, the tumbling-margin question settled (finding TB-02) |
| 2.1 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: LEOP case added (§8.6) with the magnetorquers' 1.16–1.3 W during detumble and its energy balance; peak pack currents from the EPS board (one resistor at a time, 0.75 A with the magnetorquers; §5); cold-soak burn bus voltage (§8.5); §3 note on the 475 km baseline |
