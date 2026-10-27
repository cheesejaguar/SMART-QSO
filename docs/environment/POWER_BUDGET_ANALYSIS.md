# SMART-QSO Power Budget Analysis

**Document ID**: SQSO-PBA-001
**Version**: 2.5
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR power closure (analysis by simulation; nothing measured). Replaces version 1.x, whose array
(0.06 m², 12.3 W peak), 3.3 V bus and loads were superseded (findings TB-02, TB-08, TB-14).

`docs/BASELINE_PARAMETERS.md` governs. This document replaces the orbit-average allocation of BASELINE §5 by
simulated values, as BASELINE §5 asks ("to be replaced by a multi-orbit SOC simulation"), and settles the open
"tumbling margin +19 % against 20 %" note. The margin requirement is SYS-100 Rev G (≥ 2.23 W EOL orbit average tumbling:
the 1.86 W tumbling load plus 20 %, with the 1.85 W DC transponder allocation of BASELINE §5).

## 1. Summary

| Case (end of life, worst 30 days of a year) | Generation delivered to the bus | Load (mode, at the pack) | Margin | Requirement |
|---------------------------------------------|---------------------------------|--------------------------|--------|-------------|
| ACTIVE, sun-pointed, BASELINE duty limits (transponder 25 %, Jetson 10 %); pointing with the face photodiodes and albedo (decision C-23, §4) | 3.84 W (3.79 W in the worst Monte Carlo trial) | 2.60 W | **+45 %** (+43 % worst trial); +48 % (+46 %) by the BASELINE method | ≥ 20 %: met |
| ACTIVE, tumbling (ADCS failed), duty limits (Jetson 5 %) | 2.48 W | 1.91 W | **+27 %** (+30 %) | met |
| ACTIVE, tumbling, isotropic bound (60 °C cells, no albedo) | 2.14 W | 1.91 W | +10 % | not met (bound only, §9) |
| BASELINE §5 tumbling allocation (1.86 W) | 2.48 W | 1.86 W | **+31 %** (+33 % by the BASELINE method) | met |
| IDLE, sun-pointed | 3.84 W | 1.18 W | +220 % | met |
| SAFE (ADCS off, tumbling) | 2.48 W | 0.79 W | +208 % | met |
| EMERGENCY (beacon shed, tumbling) | 2.48 W | 0.72 W | +238 % | met |
| SYS-100 Rev G: 2.23 W EOL tumbling | 2.48 W | — | +11.2 % over the requirement | met |
| As flown by the flight software, a year at LTAN 23:30, EOL | 3.92 W available (worst 30 days), 1.53 W used | — | +142 %; DoD ≤ 2.3 %; SOC ≥ 90.1 % | met |
| Wings stowed (ADCS working) | 1.53 W | SAFE 1.03 W / IDLE 1.18 W / ACTIVE 2.60 W | +46 % / +27 % / **−42 %** | ACTIVE not sustainable; the flight software holds IDLE (§8.1) |
| Wings stowed and ADCS failed | 1.53 W | SAFE 0.79 W | +90 % | positive |
| Jetson rail stuck on (both switches failed) | 2.33 W (14 days, as flown) | 2.38 W | **−4 %** as flown | Simulated: BMS cut-off at 66.6 h, then 233 brown-out cycles in 14 days; the hardware load shed below 13.6 V (RID EPS-09, EPS_DESIGN §6) is not in the simulation |
| LEOP: detumble with the magnetorquers at 1.3 W, cold pack, heater on (§8.6) | 2.48 W (tumbling EOL, conservative) | 3.03 W while detumbling | first orbit −1.20 Wh; 3-orbit bound −3.14 Wh | SOC ≥ 33.7 % from the 40 % floor: met |

All modes close with at least 20 % at end of life and at the β extremes of the LTAN 22:30 ± 1 h dispersion, with the
transponder and the Jetson at their BASELINE duty limits, except ACTIVE with the wings stowed (contingency) and a
stuck-on Jetson (failure). The battery is never the limit: the deepest discharge of any contingency run is 2.8 % per
eclipse (7.1 % with the stuck-on Jetson failure; limit 10 % nominal, 30 % contingency).

## 2. Method

1. **Simulation.** `sqso_sil` (SQSO-SIL-001) flies the flight software against orbit, attitude, array, EPS, battery
   and thermal models. Runs: a year at each LTAN of the dispersion (21:30, 22:30, 23:30) with a new array; a year at
   22:30 and 23:30 with the array aged one year at deployment (so the year covers the second year, end of life); a
   year at 23:30, end of life, with the ADCS failed; 14-day runs for the contingencies. Run matrix:
   `software/simulation/cdr/power_thermal/run_matrix.txt`; analysis: `power_closure.py` beside it. Version 2.2 re-ran
   the whole matrix with the simulation's Rev E defaults and flight software round 2 (475 km from 2028-11-01, LTAN
   22:30 ± 1 h, 60 % SOC at integration with 47 days of storage, four wing burn lines, the commissioning hold, the
   battery radiation term of SQSO-THA-001 §2.1); every run reported 0 anomalies except the injected failures' own
   checks.
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

Simulated at 475 km sun-synchronous with deployment on 2028-11-01 (BASELINE Rev E §3). The simulated β range over a
year and the longest eclipse fraction:

| LTAN | β over a year | Longest eclipse | Note |
|------|---------------|-----------------|------|
| 21:30 (−1 h) | 29.8° … 41.4° | 36.6 % (34.4 min) | Hottest; most sun |
| **22:30 (baseline)** | 16.7° … 26.8° | 37.9 % (35.7 min) | |
| 23:30 (+1 h) | 3.1° … 12.4° | 38.5 % (36.2 min) | Coldest; least sun |

β never exceeds 42°, so there is always an eclipse (no eclipse needs β > 69° at 475 km). The BASELINE §8 hot case
(no eclipse) is a bound, not an orbit this mission flies. LTAN drift over two years from an inclination injection
error was not modelled; ±1 h covers the provider's stated dispersion.

## 4. Generation (simulated)

Orbit-average array power at the maximum power point (MPP) and delivered to the bus (× 0.892), operations only:

| Run | Year mean MPP | Worst 30-day MPP | **Worst 30-day delivered** | Sunlit mean angle of +Y to the Sun |
|-----|---------------|------------------|----------------------------|-----------------------------------|
| Sun-pointed, BOL (year 1), LTAN 21:30 | 4.75 W | 4.56 W | 4.07 W | ≈ 2–3° |
| Sun-pointed, BOL, LTAN 22:30 | 4.70 W | 4.53 W | 4.04 W | |
| Sun-pointed, BOL, LTAN 23:30 | 4.69 W | 4.52 W | 4.03 W | |
| Sun-pointed, EOL (year 2), LTAN 22:30 | 4.56 W | 4.41 W | **3.93 W** | |
| Sun-pointed, EOL, LTAN 23:30 | 4.55 W | 4.40 W | **3.92 W** | |
| Tumbling (ADCS failed, free tip-off spin), EOL, LTAN 23:30 | 2.87 W | 2.78 W | **2.48 W** | 90° (random) |
| Tumbling, EOL, LTAN 22:30, 14 days | 2.82 W | — | 2.52 W | 90° |
| Wings stowed, EOL, sun-pointing working | 1.72 W | — | 1.53 W | 58° |
| Wings stowed and ADCS failed, EOL, LTAN 22:30 / 23:30 | 1.73 / 1.72 W | — | 1.55 / 1.53 W | 90° |

Against the earlier analysis (SOLAR_ARRAY_DESIGN §9.3, 500 km, β = 0, cells at an assumed 60 °C, no albedo): sun-pointed
3.59 W and isotropic tumbling 2.14 W after an MPPT of 0.95. The simulation is higher mainly because the thermal model
puts the body cells at −15 … +38 °C and the wings at −40 … +80 °C (THERMAL_ANALYSIS §4) instead of 60 °C (≈ +10 %
for the body cells), and because it counts albedo (≈ +5 %). The seasonal swing is ±4 % (perihelion in January).

**Pointing with the flight sensors (decision C-23, Version 2.5).** The sun-pointed runs above hold +Y on the Sun as
the simulation's attitude model does. The ADCS Monte Carlo with the face photodiodes and Earth albedo
(`docs/hardware/ADCS_DESIGN.md` 1.9 §11.6, requirement ADCS-P002) loses 2.0 % of the sunlit generation on average and
3.2 % in its worst trial, so the sun-pointed EOL generation used for closure is **3.92 × 0.980 = 3.84 W** (mean) and
**3.92 × 0.968 = 3.79 W** (worst trial); BOL 4.03 × 0.980 = 3.95 W. The tumbling case has no pointing and is unchanged
(2.48 W). This analysis has no one-wing case (a single deployed wing is a contingency of ADCS_DESIGN §11.6: preset 0
costs ≤ 2.0 % against the matching preset there). **The two margin methods:** this document's margin is generation /
(load × 1.019) − 1 (the 5 % battery round trip on the 38 % eclipse share); BASELINE §5 uses generation / load − 1 on
its allocation loads (1.86 W tumbling, 2.26 W sun-pointed). For the same generation the BASELINE method reads 3 points
higher on the bottom-up loads; on the allocation loads it gives +33 % tumbling and +70 % (+68 % worst trial)
sun-pointed.

## 5. Loads by mode

Device values from the simulation (`sqso-sil/src/mission.rs`, `hw/radio.rs`, `hw/eps.rs`, `jetson.rs`): OBC with FRAM
and sensors 0.165 W and the receiver 0.15 W on 3V3_AO (buck 85 %), EPS quiescent 0.08 W; ADCS sensors 0.12 W
(magnetorquers ≈ 0 W in the spin hold, measured in the simulation; 1.16–1.3 W while detumbling, §8.6); transponder 1.84 W DC at 0.5 W RF on the PA rail (RF board E, with the synthesizer's own regulator; 2.42 W at 1 W RF; buck 90 %; the simulation still models 1.5 W);
beacon 1.67 s on air per frame; GNSS at its BASELINE allocation 0.05 W (not modelled in the simulation); Jetson 8 W.
The transponder and Jetson duty limits are orbit fractions (BASELINE §5).

| Item (W at the pack) | EMERGENCY | SAFE | IDLE | ACTIVE, sun-pointed | ACTIVE, tumbling |
|----------------------|-----------|------|------|---------------------|------------------|
| OBC, EPS, receiver (always on, 97.207(b)) | 0.451 | 0.451 | 0.451 | 0.451 | 0.451 |
| Beacon (period) | — (shed) | 0.015 (180 s) | 0.023 (120 s) | 0.046 (60 s) | 0.046 |
| ADCS | off | off | 0.141 | 0.141 | 0.141 |
| GNSS (allocation) | — | 0.050 | 0.050 | 0.050 | 0.050 |
| Transponder, 25 % duty (1.84 W DC / 0.90) | — | — | — | 0.511 | 0.511 |
| Jetson, 10 % / 5 % duty | — | — | — | 0.889 | 0.444 |
| **Subtotal** | **0.451** | **0.516** | **0.665** | **2.088** | **1.643** |
| Heater (cold case, 16.4 V, +5/+9 °C) | 0.27 | 0.27 | 0.51 | 0.51 | 0.27 |
| **Total** | **0.72** | **0.79** | **1.18** | **2.60** | **1.91** |
| BASELINE §5 allocation (transponder 1.85 W DC, 0.46 W orbit average) | — | — | — | 2.26 | 1.86 |

The bottom-up ACTIVE load is below the BASELINE allocation before the heater (2.09 W against 2.26 W; 1.64 W against
1.86 W). The allocation's housekeeping line (0.70 W) holds 0.11 W more than the bottom-up housekeeping (0.59 W with the
ADCS) but nothing for the heater, which the BASELINE lists in that line; with the heater the bottom-up loads are
0.34 W (sun-pointed) and 0.05 W (tumbling) above the allocation.

**Peak currents at the pack** (for fuses and harness, EPS_DESIGN §5; SQSO-EPS-BRD-001 §6.1, §9): housekeeping 0.05 A;
transponder keyed at 1 W RF 0.20 A (2.42 W DC; PA rail 0.485 A, inside SW3's 0.92 A minimum trip); Jetson 8 W 0.62 A (12 W latch-up guard trip 0.93 A); heater 0.08 A at 16.4 V;
magnetorquers 0.083 A with three rods saturated at 14.4 V (0.104 A at 11.2 V; 0.23 A at the bridges' hardware limit);
one burn resistor 0.51 A for ≤ 12 s (0.8 A at the resistor from the 7.0 V burn buck). Worst simultaneous credible, in
LEOP (the Jetson and the transponder are off): burn + housekeeping + heater 0.67 A, **0.75 A** with the magnetorquers
detumbling (0.90 A with the bridges at their limit), against the 1.0 A limit for a burn; four drivers on at once (a
software fault) are held to 2.0 A by the burn buck. Outside LEOP the largest legitimate load is 1.36 A (Jetson at the
guard, transponder at 1 W, housekeeping, heater, magnetorquers). A burn with the transponder keyed at 1 W would draw
0.94 A (1.09 A with the coil bridges at their fault limit), but they are not concurrent by design: the RF silence runs
to 45 min and the burn is at 30 min (SQSO-EPS-BRD-001 E §6.1).

## 6. Mode-by-mode closure

End of life, worst 30 days, LTAN dispersion extremes (generation from §4, loads from §5):

| Mode | Attitude flown | Generation | Load | Margin |
|------|----------------|------------|------|--------|
| EMERGENCY | Tumbling (ADCS off) | 2.48 W (2.14 W bound) | 0.72 W | +238 % (+191 %) |
| SAFE | Tumbling (ADCS off) | 2.48 W (2.14 W bound) | 0.79 W | +208 % (+167 %) |
| IDLE | Sun-pointed | 3.84 W (3.79 W worst trial) | 1.18 W | +220 % (+215 %) |
| ACTIVE | Sun-pointed | 3.84 W (3.79 W worst trial; BOL 3.95 W) | 2.60 W | **+45 %** (+43 % worst trial; BOL +49 %) |
| ACTIVE | Tumbling (ADCS failed) | 2.48 W (2.14 W bound) | 1.91 W | **+27 %** (+10 % bound) |

SAFE and EMERGENCY switch the ADCS off (`sqso_fsw::eps::loads`), so they are closed on the tumbling generation. The
spin left when the coils stop keeps +Y near the Sun for a while (the residual-dipole torque precesses it by about a
degree a minute at the 3 mA·m² design value), so this is conservative.

**Heater energy and the recommended set points.** Raising the thermostat from +2/+6 °C to +5/+9 °C (THERMAL_ANALYSIS
§6.1) costs ≈ 0.1 W orbit average in the cold case; it is included above. The simulation's 0.5 W heater at +2/+6 °C
averaged 0.17–0.29 W over 30-day windows in the earlier runs; the Version 2.2 runs, at +5 °C, keep the battery at
≥ 4.9 °C with a worst-orbit heater duty of 38–39 % sun-pointed.

## 7. As flown (flight software in the loop)

The flight software runs the transponder on demand and the Jetson in 10-line batches, well below the duty limits.
Year runs (operations, Rev E defaults, flight software round 2):

| Run | Delivered (year mean) | Used | Margin, worst 30 d | Max DoD | Lowest SOC | Battery | Heater duty, worst orbit | Charge refused |
|-----|-----------------------|------|--------------------|---------|------------|---------|--------------------------|----------------|
| BOL, LTAN 21:30 | 4.24 W | 1.57 W | +142 % | 2.3 % | 90.2 % | 4.9 … 10.9 °C | 39 % | 0 s |
| BOL, LTAN 22:30 | 4.19 W | 1.56 W | +143 % | 2.3 % | 90.1 % | 4.9 … 12.6 °C | 39 % | 0 s |
| BOL, LTAN 23:30 | 4.18 W | 1.54 W | +148 % | 2.3 % | 90.1 % | 4.9 … 11.6 °C | 38 % | 0 s |
| EOL, LTAN 22:30 | 4.07 W | 1.55 W | +139 % | 2.3 % | 90.1 % | 4.9 … 12.2 °C | 38 % | 0 s |
| EOL, LTAN 23:30 | 4.06 W | 1.53 W | +142 % | 2.3 % | 90.1 % | 4.9 … 12.7 °C | 38 % | 0 s |
| EOL, LTAN 23:30, ADCS failed | 2.56 W | 1.23 W | +86 % | 2.3 % | 90.1 % | 4.9 … 15.1 °C | 20 % | 0 s |

Every run reported 0 anomalies. The lowest SOC (90.1 %) is the 4.10 V/cell set point (D8). Against Version 2.1 the
as-flown load is 0.2 W higher (1.53–1.57 W against 1.31–1.35 W), mostly the heater at its +5 °C set point and the
flight software's round-2 functions (daily stored-housekeeping dumps, acknowledgements); the margins remain above
+139 % sun-pointed and +86 % tumbling, and the battery's coldest point rises from +0.4 °C to +4.9 °C.

## 8. Contingencies

### 8.1 Wings not deployed

1.53 W delivered with the sun pointing working. SAFE (+46 %) and IDLE (+27 %) close; ACTIVE at the duty limits does
not (−42 %). The flight software now holds IDLE when the wings read not deployed (the rule recommended under RID
EPS-10): in the 14-day run it never entered ACTIVE, the transponder and the Jetson stayed off, and the as-flown margin
was +38 % with a lowest SOC of 70.0 % and DoD 2.3 %. In the bounding cold case the heater alone needs 0.95 W at 100 %
duty with the wings stowed (THERMAL_ANALYSIS C16), which leaves 0.58 W: SAFE (0.52 W before the heater) closes at +10 %.

### 8.2 ADCS failed (tumbling)

2.48 W delivered (worst 30 days, EOL, LTAN 23:30, β 3.2–12.4°). ACTIVE at the tumbling duty limits +27 %; as flown
+86 %; DoD 2.3 %; heater duty ≤ 20 % in the worst orbit (battery 4.9–15.1 °C).

### 8.3 Wings and ADCS both failed

1.53–1.55 W. SAFE +90 %, EMERGENCY +109 %. As flown (IDLE held by the wing rule) +53 to +56 %, SOC ≥ 65.8 %, DoD
2.8 %.

### 8.4 Stuck loads

| Stuck load | Power at the pack | Balance in SAFE, EOL | Result |
|------------|-------------------|----------------------|--------|
| Jetson rail on (both series switches failed closed) | 8.9 W inferring; ≈ 3.9 W idle | −5.7 W sun-pointed, −7.2 W tumbling (inferring) | Simulated (the module idling, as the flight software never sends it work; as flown −4 % over 14 days): SOC 58 % → SAFE at 56.9 h, EMERGENCY at 64.5 h, BMS under-voltage cut at 66.6 h, then 233 brown-out cycles in 14 days (uptime 55 %). Loss of mission in the simulation. The EPS's hardware load shed (13.6 V for 5 s, RID EPS-09, EPS_DESIGN §6) gates the enables of **both** Jetson series switches (SW1 and the eFuse U19, SQSO-EPS-BRD-001 D §8.4), so it covers either switch failed closed, a stuck enable line or software that never turns the Jetson off: in those cases the Jetson is removed at 13.6 V and the pack stops at ≈ 3.4 V/cell, with the OBC, receiver and TX-OFF alive. **Not covered (residual):** both series switches failed closed in their power stages, the case simulated here (two independent part failures; recorded in the FMEA notes, SQSO-EPS-BRD-001 §18). The shed is not modelled in the simulation yet (flight-software/SIL item) |
| Heater on (thermostat and switch failed) | 1.3 W | +0.58 W tumbling; −0.38 W with the wings stowed | Survivable except with stowed wings; the +40 °C cutout limits the temperature (THERMAL_ANALYSIS §6.2) |
| PA keyed continuously (PTT stuck, 0.5 W RF) | 1.67 W | +0.20 W tumbling | Energy-positive; the 120 s hardware key-down timer (BUILD_VS_BUY §3.2) and TX-OFF are the controls |
| GNSS continuous (0.125 W typical, 0.32 W worst, BASELINE §5 note) | 0.32 W | ACTIVE tumbling +31 % → +16 % | Within margin only at the typical draw; keep it duty-cycled |

### 8.5 Cold-soak deployment

With the whole spacecraft at −20 °C at separation, the deployment burn at 30 min (one resistor at a time, 0.75 A at
the pack with the magnetorquers detumbling) leaves the bus at ≥ 13.8 V at the ≈ 58 % deployment SOC (EMC_ANALYSIS Version C §4;
11.7 V in the four-driver software fault) and the charge regulator refuses charge for the first 1.5 h (pack below 0 °C). The pack must leave the dispenser with ≥ 40 % SOC (BATTERY_SAFETY_ASSESSMENT §6).

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
| Separation in eclipse, first sun after 28.8 min (worst case at 475 km from 2028-11-01, `leop_passes.py`), cold pack | 3.03 W × 0.48 h = 1.45 Wh | 0 | −1.45 Wh | −2.9 points (55.1 %) |
| Same, warm pack (no heater) | 1.94 W × 0.48 h = 0.93 Wh | 0 | −0.93 Wh | −1.9 points |
| First orbit (1.57 h), detumbled after 1.4 h, cold pack, worst eclipse phasing | 0.64 W × 1.57 h + 1.30 W × 1.37 h + 1.09 W × 1.57 h + 0.12 Wh = 4.62 Wh | 2.48 W × 1.57 h − 0.48 Wh (wings stowed for the first 30 min) = 3.42 Wh, tumbling, EOL value used | −1.20 Wh | −2.4 points (55.6 %) |
| Bound: detumble over 3 orbits (4.70 h), heater on throughout | 0.64 W × 4.70 h + 1.30 W × 4.67 h + 1.09 W × 4.70 h + 0.12 Wh = 14.32 Wh | 2.48 W × 4.70 h − 0.48 Wh = 11.18 Wh | −3.14 Wh | −6.3 points (51.7 %); from the 40 % separation floor, 33.7 % |

The wings-stowed deficit is (2.48 − 1.53) W × 0.5 h = 0.48 Wh (§4). Generation is taken at the tumbling end-of-life
value although the array is new and the spacecraft sun-points once detumbled, so every line is conservative. The worst
LEOP balance, the 3-orbit bound from the 40 % separation floor, leaves 34 % SOC, above the SAFE threshold (25 %) and
the deepest-discharge limit for a contingency (30 % DoD); the first orbit costs at most 2.4 points. The magnetorquers
are the largest LEOP load after the heater: without them the first-orbit balance would be +0.58 Wh. The mission
simulation's LEOP (`docs/operations/MISSION_TIMELINE.md` §5: 0.42–0.46 W while detumbling, 0.15 Wh before first sun in
the reference case) does not carry the board's detumble power; this section bounds it.

## 9. The tumbling margin (BASELINE §5 "under review")

**Question.** With the wings' self-shadowing the isotropic-tumble analysis gave 2.14 W against the 1.80 W tumbling
allocation: +19 %, below the 20 % requirement.

**Answer.** The 2.14 W is a bound built on two assumptions the CDR analyses replace: cells at 60 °C (the thermal
model gives −15 … +27 °C on the body panels of a tumbling spacecraft, C1) and no albedo. The simulation, with the
same cell data, shadowing and degradation, gives 2.48 W delivered at end of life in the worst 30 days of a year at the
coldest LTAN of the dispersion (475 km, Version 2.2): **+31 % against the 1.86 W allocation** (+33 % by the BASELINE
method), +27 % against the bottom-up tumbling ACTIVE load with the heater, and 11.2 % above SYS-100 Rev G's 2.23 W. The requirement is met. The
residual risk is the attitude: the simulated tumble is the free spin left by the tip-off; an isotropic tumble at the
model's temperatures and albedo would give about 2.14 × 1.10 × 1.05 ≈ 2.47 W, consistent with it.

**Recommended BASELINE §5 text (for the chair):**

> | Generation, EOL orbit average delivered to the bus | **2.48 W tumbling; 3.84 W sun-pointed** (3.79 W in the worst ADCS Monte Carlo trial; 3.92 W with +Y held on the Sun; mission simulation at 475 km, worst 30 days of the second year at LTAN 23:30, pointing loss per ADCS_DESIGN 1.9 §11.6; SQSO-PBA-001 v2.5 §4). The isotropic-tumble analysis (2.14 W, SOLAR_ARRAY_DESIGN §9.3, cells at 60 °C, no albedo) is kept as a bound |
>
> **Power closure (SQSO-PBA-001 v2.5 §6):** ACTIVE at the duty limits 2.60 W sun-pointed (+45 %; +43 % in the worst pointing trial), 1.91 W tumbling (+27 %); IDLE 1.18 W; SAFE 0.79 W; EMERGENCY 0.72 W, heater included. Margin ≥ 20 % in every mode at end of life and at LTAN 22:30 ± 1 h. Wings stowed: SAFE and IDLE only.

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
| EPS-10 | 3 | Wings-stowed ACTIVE relies on the SOC gate limit cycle | Flight software holds IDLE with the wings stowed (simulated, §8.1) |

## 13. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| 1.0 | 2026-01-02 | SMART-QSO Team | Initial release |
| 1.1 | 2026-10-16 | Funding audit | Notice: generation superseded by SOLAR_ARRAY_DESIGN §9 (TB-14); battery §2.2 and §5.4 corrected to BASELINE §5 (TB-08) |
| 2.0 | 2026-10-27 | CDR review, EPS/thermal | Rewritten: CDR power closure from the mission simulation (year runs at LTAN 21:30/22:30/23:30, BOL and EOL, contingencies), bottom-up mode loads, the tumbling-margin question settled (finding TB-02) |
| 2.1 | 2026-10-27 | CDR documentation review | CDR 2026 consistency pass: LEOP case added (§8.6) with the magnetorquers' 1.16–1.3 W during detumble and its energy balance; peak pack currents from the EPS board (one resistor at a time, 0.75 A with the magnetorquers; §5); cold-soak burn bus voltage (§8.5); §3 note on the 475 km baseline |
| 2.2 | 2026-10-27 | CDR S&MA analysis (RID SE-16) | Run matrix re-run with the simulation's Rev E defaults and flight software round 2 (475 km, 2028-11-01, four burn lines): EOL generation 3.92 W sun-pointed (was 3.93) and 2.48 W tumbling (was 2.43); margins ACTIVE +54 % / +34 % (was +54 / +31 %), SYS-100 +14.8 %; as-flown year at LTAN 23:30 EOL 1.53 W used, +142 % (was 1.32 W, +182 %), DoD ≤ 2.3 %, battery ≥ 4.9 °C; wings stowed held in IDLE (+38 % as flown); stuck Jetson cut-off at 66.6 h (the hardware shed not simulated); §3 β and eclipse at 475 km; §8.6 LEOP with the 28.8 min worst darkness |
| 2.3 | 2026-10-27 | CDR design (EPS board) | §8.4 stuck-Jetson row: the hardware load shed gates both series switches (SQSO-EPS-BRD-001 D); covered failure modes and the residual (both power stages failed closed) stated |
| 2.4 | 2026-10-27 | CDR design (EPS board) | Transponder 1.84 W DC at 0.5 W RF (RF board E; BASELINE §5 allocation 1.85 W DC, 0.46 W orbit average): §5 bottom-up ACTIVE loads 2.60 / 1.91 W, margins +48 % / +27 % (+51 % / +30 % by the BASELINE method), allocation 2.26 / 1.86 W (+31 % tumbling), SYS-100 Rev G 2.23 W (+11.2 %); peak currents (1.36 A largest, transponder 0.20 A at the pack, burn + keyed 0.94 A, not concurrent); the as-flown runs of §7 are not re-run with the new transponder (it runs far below its 25 % duty limit) |
| 2.5 | 2026-10-27 | CDR design (EPS board) | Decision C-23 carried through: sun-pointed EOL generation with the photodiode-and-albedo pointing loss of ADCS_DESIGN 1.9 §11.6 (× 0.980 mean, × 0.968 worst trial): 3.84 / 3.79 W; ACTIVE sun-pointed +45 % (+43 %), IDLE +220 %; both margin methods stated (§4); tumbling unchanged |
