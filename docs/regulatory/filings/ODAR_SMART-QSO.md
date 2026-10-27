# Orbital Debris Assessment Report (ODAR)
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-REG-018 (supersedes SQSO-ODAR-001, `docs/compliance/ORBITAL_DEBRIS_ASSESSMENT.md`)
**Revision:** Draft B
**Date:** 2026-10-27
**Format:** NASA-STD-8719.14C (approved 2021-11-05) Appendix A.1, in the section order of the NASA Small Spacecraft Technology Program *ODAR & End of Mission Plan Instructional Guide* (spacecraft Sections 1–8; launch-vehicle Sections 9–14 are the launch provider's)
**Regulatory use:** basis for the orbital-debris disclosures in the FCC pre-space notification, 47 CFR 97.207(g)(1)(i)–(vii) ([draft](FCC_97-207_PRE_SPACE_NOTIFICATION.md))
**Status:** **NASA DAS 3.2.x has not been run.** DAS is distributed under a NASA software usage agreement and runs on Windows; neither was available to the CDR review. Every number here comes from the project's own models, which follow the methods of NASA-STD-8719.14C (cited section by section) and are in `software/simulation/cdr/sma/` and `software/simulation/orbit_lifetime/`. Values tagged **[DAS]** must be reproduced in DAS 3.2.6 (or a higher-fidelity tool, as the FCC rule allows) with the controlled mass properties before filing.

---

## Self-Assessment Summary

| Requirement (NASA-STD-8719.14C) | Topic | Result (project models) | Margin | Status |
|---------------------------------|-------|-------------------------|--------|--------|
| 4.3-1a, 4.3-1b | Released debris in LEO | None released | — | Compliant |
| 4.3-2 | Debris near GEO | N/A | — | N/A |
| 4.4-1 | Accidental explosion < 0.001 | **1.4 × 10⁻⁴** over a 10-year bound | ×7 | Compliant by analysis (§4) |
| 4.4-2 | Passivation design | Option 2 (energy controlled by the battery protections) met by design; option 1 (charge-path disconnect) **designed as a placeholder only** | — | Compliant via option 2; option 1 open (§4) |
| 4.4-3, 4.4-4 | Planned breakups | None | — | N/A |
| 4.5-1 | Collision with objects ≥ 10 cm < 0.001 | **≤ 1.2 × 10⁻⁴** (worst case: 525 km, 2.0 kg, wings failed, F10.7 = 70) | ×8 | Compliant by analysis **[DAS]** (§5) |
| 4.5-2 | Small-debris impact preventing disposal < 0.01 | Disposal is passive; no function is needed after wing deployment at T + 30 min | — | Compliant by design (§5) |
| 4.6-1a | Natural re-entry ≤ 25 yr after mission, **conservative solar projection** | ≤ 13 yr post-mission at ≤ 485 km in every case; **fails at 525 km** for ≥ 1.8 kg with wings failed under the F10.7 = 70 bound (27.3–30.5 yr) | — | Compliant at ≤ 500 km (§6) |
| 4.6-4 | Disposal reliability ≥ 0.9 | Passive; P(both wings fail) = 0.016; compliant if the wings-deployed case meets the rule | — | See FCC row |
| **47 CFR 97.207(g)(1)(vii)(D)(1)** | **Re-entry ≤ 5 yr after mission completion, P ≥ 0.9** | **Not robustly met at the baseline 500 km target**: compliant only with the nominal solar projection and only if the mission is not cut short; **fails at the 525 km cap** in every case modelled | — | **Non-compliant at the baseline cap; compliant at ≤ 485 km (planned-mission reading) or ≤ 465–470 km (early failure counted as mission end)** (§6) |
| 4.7-1 / 97.207(g)(1)(vii)(D)(2) | Casualty risk < 1:10,000, fragments > 15 J | **No surviving fragment > 15 J** with nominal heating; worst sensitivity (heating halved, whole cells modelled as steel) **1:47,000** | ≥ ×4.7 | Compliant by analysis **[DAS]** (§7) |
| 4.8-1 | Tethers | None | — | N/A |
| 97.207(g)(1)(v) | Trackability | Smallest dimension 10 cm (1U), presumed trackable | — | Compliant |

---

## 1. Program Management and Mission Overview

| Item | Entry |
|------|-------|
| Responsible organization / licensee | Space station licensee **KK6GDB** (individual amateur licensee), station KK6GDB-11 (decision D1, [BASELINE_PARAMETERS.md](../../BASELINE_PARAMETERS.md) §1, §9); host organization: none yet |
| Program manager / PI | `[name, affiliation]` |
| Mission | SMART-QSO: 1U amateur-radio CubeSat with a duty-gated U/V FM transponder, AX.25 telemetry beacon and a power-gated onboard AI experiment |
| Milestones | No independent PDR or CDR yet; this is the CDR draft · launch NET 2028 |
| Launch vehicle and site | SpaceX Falcon 9 Transporter rideshare via a 1U integrator, Vandenberg SFB (baseline, D7); backup ISS deployment via the Voyager NRCSD |
| Deployment | Ejected from a CubeSat dispenser at 0.3–1.0 m/s into a circular SSO |
| Orbit | Circular, inclination ≈ 97.4°; baseline target ≤ 500 km, cap 525 km (BASELINE §3); **this report recommends a 465–485 km band (§6.5)** |
| Mission duration | 2 years design life (12 months minimum) |
| Interaction with other spacecraft | Shared SSO altitude band; no proximity operations; no manoeuvring capability; conjunction data via 18 SDS / Space-Track (`SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md`) |

## 2. Spacecraft Description

**Physical:** 1U CubeSat, 100 × 100 × 113.5 mm stowed. Two deployable solar wings (83 × 98 mm panels hinged on the ±X
edges of +Y, deployed into the +Y plane: `hardware/eps/SOLAR_ARRAY_DESIGN.md` §3.1), an ISIS AntS antenna (two 2 m tape
elements of ≈ 0.5 m and two 70 cm elements of ≈ 0.17 m).

**Mass:** the controlled MEL (`docs/environment/MASS_BUDGET.md` v2.0) gives CBE 1.69 kg and a predicted 1.92–2.00 kg; the
build-versus-buy decisions (`docs/hardware/BUILD_VS_BUY.md` §1) take about 0.20–0.25 kg off, and the MEL is being re-run
with quoted masses. This report therefore uses **1.6 kg (nominal) and 1.8 kg (bound)** as BASELINE §4 directs, and adds
**2.0 kg (the CDS Rev 14.1 limit)** so that no mass the spacecraft can have is outside the analysis.

| Item | Propulsion | Fluids | Pressure vessels | Pyrotechnics | Radioactive | Other stored energy |
|------|-----------|--------|------------------|--------------|-------------|---------------------|
| SMART-QSO | None | None | None (sealed 18650 cells are "other pressurized equipment", SpaceX UG v11 §5.5.4) | None (burn-wire releases) | None | Hinge springs and antenna tapes (released at T + 30 min); Li-ion battery |

**Electrical generation and storage:** 16 triple-junction cells (AZUR 3G30A). Battery: 4S1P Samsung INR18650-35E,
3.45 Ah, ≈ 50 Wh, charged to 4.10 V/cell (D8). Protections and their fault tolerance: `docs/environment/BATTERY_SAFETY_ASSESSMENT.md`
(SQSO-BSA-001) and `docs/safety/SYSTEM_SAFETY_DATA_PACKAGE.md` (SQSO-SDP-002) HR-01.

**Attitude control:** magnetorquers (air-core coils or CubeSpace CR0002 rods), magnetometer, sun sensors, Epson
M-G370PDT0 gyro with a BMI088 backup; B-dot detumble, then spin-stabilised coarse sun-pointing. No wheels.

**Components relevant to re-entry survival** (the demise model input, §7; masses per item, CBE):

| Component | Qty | Mass each (g) | Material | Notes |
|-----------|-----|---------------|----------|-------|
| 1U frame (bought) | 1 | 110 | Al 7075/6061 | ISIS 87–108 g, Spacemanic 115 g (BUILD_VS_BUY §2.2) |
| Jetson heat spreader | 1 | 65 | Al | MASS_BUDGET row 14 |
| Battery clamp plates | 2 | 17 | Al 6061-T6 | Isolated mount (BUILD_VS_BUY §4.3) |
| 18650 cells | 4 | 50 | Ni-plated steel can (≈ 8 g), Cu/Al foil, NCA, graphite, electrolyte | Modelled two ways: can only, and the whole cell as a steel cylinder (bounding) |
| Stainless fasteners (A286 / 300-series) | ≈ 80 | 0.4–0.9 | Stainless steel | MASS_BUDGET row 22, 25 g in all |
| **Ti-6Al-4V screws, battery mount** | 4 | 0.3 | Ti-6Al-4V | M2.5; alternative necked Ti standoffs ≈ 1.5 g each, also modelled |
| Hinge pins and springs | 2 + 4 | 2.5, 0.6 | Stainless steel | Assumed 2 mm × 98 mm pins |
| AntS tape elements | 4 | ≤ 2 | 301 stainless | Thin; the 70 cm tapes are lighter |
| Gyro M-G370PDT0 | 1 | 10 | Case material not published: modelled as aluminium and as stainless | |
| GNSS ceramic patch | 1 | 9.5 | Alumina-class ceramic | ZENITH V3a; dimensions assumed 25 × 25 × 4 mm |
| CR0002 magnetorquer rods (if bought) | 3 | 16.5 | Modelled as solid NiFe (bounding) | |
| Air-core coils (if kept) | 3 | 46 | Copper | |
| RF connector bodies | 4 | 2 | Stainless | |

No beryllium, tungsten or other refractory parts are in the baseline. The **only titanium parts are the battery-mount
screws** (or standoffs); the previous revision's statement "no titanium" is withdrawn.

## 3. Debris Released During Normal Operations (Req. 4.3-1, 4.3-2)

- Objects > 1 mm intentionally released: **none.** Deployables stay attached (CDS Rev 14.1 §2.1.1). Burn-wire restraint
  lines are retained at captive tie points on both ends of the cut; melt residue is < 1 mm.
- **4.3-1a, 4.3-1b:** compliant. **4.3-2:** N/A.

## 4. Intentional Breakups and Potential for Explosions (Req. 4.4-1 to 4.4-4)

**4.4-1 (integrated probability of explosion < 0.001).** The only source is the Li-ion pack. Every venting or runaway
of any cell is counted as an explosion, although an 18650 vents through its cap and is not expected to fragment the
spacecraft. Fault-tree estimate over a 10-year orbital-life bound, with charging possible until passivation (50 %
chance it is commanded at 2 years, otherwise until re-entry) (`software/simulation/cdr/sma/explosion_probability.py`):

| Cause | Controls in the chain | Probability |
|-------|----------------------|-------------|
| Overcharge | Charger CV fails high (0.02/yr), then BMS over-voltage (0.01/yr latent), independent protector (0.01/yr latent) and the cell CID (0.1 on demand) | 1.4 × 10⁻⁵ |
| Internal short, spontaneous | Screened cells, 10⁻⁶ per cell-year (the order of the field rate commonly quoted for quality 18650 cells; an assumption) | 4.0 × 10⁻⁵ |
| External short | Insulation breach, then fuse, PTC and BMS short-circuit trip all failed | 2.5 × 10⁻⁹ |
| Over-discharge, then recharge | BMS and hardware under-voltage both fail, the pack is drained below 2 V (0.05), then recharged | **9.0 × 10⁻⁵** |
| **Total** | | **1.4 × 10⁻⁴** |

Compliant (margin ×7). The largest term disappears with a latched recharge lockout below ≈ 2.0 V/cell, which the EPS
design does not have yet (RIDs EPS-12, SMA-06). The overcharge row assumes the dissimilar secondary protector (EPS-11);
without it (one latent protection fewer) the overcharge term is 3.6 × 10⁻⁴ and the total **4.9 × 10⁻⁴**, still
compliant but with a margin of only ×2.

**4.4-2 (passivation design).** NASA-STD-8719.14C Req. 4.4-2 accepts either (1) depleting stored energy and
disconnecting generation, or (2) controlling it to a level that cannot cause an explosion or deflagration large enough
to release debris or break up the spacecraft.

- **Option 2 is met by design:** a 50 Wh pack of four sealed 18650 cells, with over-voltage, under-voltage, over-current,
  short-circuit and temperature protections in hardware, cannot release debris by the analysis above even if never
  passivated (the 10-year bound already assumes no passivation half the time).
- **Option 1, the FCC's "permanent discharge state" (97.207(g)(1)(iii)), is the baseline (BASELINE §5)** but exists only
  as a placeholder: fuse F1 "EOL passivation, non-resettable (TBD)" in schematic S1, with no arming circuit and no flight
  software. SQSO-BSA-001 H9 specifies it (armed and fired by two independent signals; V-BSA-7). At end of mission an
  authenticated telecommand (and, if adopted after the D3 consultation, the autonomous backup M = 180 days after the
  loss-of-contact inhibit, never in the first year) opens F1 permanently, disables the transmitters, and the loads then
  drain the pack to the BMS under-voltage cut-off. If the spacecraft fails before end of mission, passivation by
  command is impossible and option 2 is the claim.

**4.4-3, 4.4-4:** N/A.

## 5. Potential for On-Orbit Collisions (Req. 4.5-1, 4.5-2)

**4.5-1 (collision with objects ≥ 10 cm < 0.001 over the orbital lifetime).** Method: NASA-STD-8719.14C eq. 4.5-1,
P = F·A·T, summed along the decay (`software/simulation/cdr/sma/collision_probability.py`):

- **Flux F:** spatial density of every catalogued object below 2,000 km in 10 km shells from the public CelesTrak
  SATCAT (31,396 objects on orbit; file of 2026-10-25), with each object's time at radius from Kepler's equation,
  multiplied by **2.0** for objects ≥ 10 cm not in the catalogue and for growth, times a relative velocity of 12 km/s.
  Design flux: 5.3 × 10⁻⁵ (450 km), **1.5 × 10⁻⁴ (475 km, the new Starlink shells near 480 km)**, 4.4 × 10⁻⁵ (500 km),
  2.4 × 10⁻⁵ (525 km) per m² per year.
- **Area A:** 0.0314 m², the largest average cross-section (wings and antennas deployed; 8719.14C §4.3.4.2.2(a)(2)(b)).
- **Lifetime T:** from the decay history with the smallest drag area (longest life). Taking area and life from
  different cases over-states the product on purpose.

| Deployment | Mass | Solar case | P(collision), wings failed / deployed |
|------------|------|------------|---------------------------------------|
| 475 km | 1.6 kg | nominal | 3.3 × 10⁻⁵ / 2.0 × 10⁻⁵ |
| 475 km | 2.0 kg | F10.7 = 70 bound | 5.4 × 10⁻⁵ / 3.5 × 10⁻⁵ |
| 500 km | 2.0 kg | F10.7 = 70 bound | 1.0 × 10⁻⁴ / 7.0 × 10⁻⁵ |
| 525 km | 2.0 kg | F10.7 = 70 bound | **1.2 × 10⁻⁴** / 7.8 × 10⁻⁵ |

**Compliant in every case, worst 1.2 × 10⁻⁴ (margin ×8). [DAS]** DAS uses ORDEM fluxes; the catalogue method here does
not credit the conjunction avoidance of manoeuvring constellations, so it is conservative for them. SMART-QSO cannot
manoeuvre; it will cross the ISS altitude during decay. Conjunction warnings will be monitored and shared (UG v11
§9.5.7.2 ephemeris, LVR-089; 97.207(g)(1)(iv)).

**4.5-2 (small-debris impact preventing disposal < 0.01).** Disposal is passive drag. The only disposal-relevant
function is wing deployment at T + 30 min, before any meaningful exposure, and the wings-failed case is analysed (§6).
No later function is needed, so the probability that small-debris impact prevents disposal is **effectively zero**;
compliant by design (DAS reports 4.5-2 only for components needed for disposal). The commanded passivation (§4) is not
a disposal function; its loss to impact is covered by option 2.

## 6. Post-Mission Disposal (Req. 4.6-1 to 4.6-5; FCC 97.207(g)(1)(vii))

### 6.1 Method and inputs

Uncontrolled re-entry by natural decay (4.6-1a). Estimator: NRLMSIS 2.1, circular decay, Cd 2.2, deployment
2028-06-01, re-entry at 180 km (`software/simulation/orbit_lifetime/orbit_lifetime.py`; scenarios and cross-sections in
`software/simulation/cdr/sma/debris_lifetime.py`).

**Average cross-sections** (NASA-STD-8719.14C §4.3.4.2.2(a); smaller area is conservative for lifetime):

| Configuration | Method | Area |
|---------------|--------|------|
| Wings failed (stowed) | Convex: surface / 4 = (2 × 0.0100 + 4 × 0.01135) / 4 | **0.0164 m²** |
| Wings deployed, no antenna credit | Complex shape: (Amax + A1 + A2) / 2 = (0.02762 + 0.01135 + 0.01033) / 2 | **0.0247 m²** |
| Wings and antennas deployed | Same, with the AntS tapes (5 mm wide) | 0.0314 m² |

The BASELINE value 0.028 m² for "wings deployed" assumed full-face (100 × 113.5 mm) wings without antennas; with the
83 × 98 mm panels of the array design the conservative value is 0.0247 m². This alone lengthens the wings-deployed life
by about 10–15 %.

**Solar activity** (4.6-1a requires a *conservative* projection):

| Scenario | Profile |
|----------|---------|
| Nominal | NOAA SWPC predicted F10.7 (monthly, retrieved 2026-10-27, to 2030-12), cycle 25/26 minimum mid-2031, cycle 26 maximum 140 in 2035.5 |
| Conservative | NOAA *low* band to 2030-12, then a weak cycle 26 (maximum 110) |
| Bound | F10.7 = 70 held for the whole decay (no solar maximum at all; unphysical, but the bounding case BASELINE §3 uses) |

The estimator's earlier "solar-cycle profile" agrees with the NOAA prediction to within 5–8 F10.7 units over 2028–2030
(it is slightly lower, i.e. conservative), so the BASELINE §3 table is reproduced exactly with its inputs (A = 0.028 m²,
1.6 kg: 500 km 5.1 / 9.3 yr wings deployed, 6.2 / 15.8 yr wings failed). The differences below come from the
cross-section, the masses and the cycle-26 assumption.

### 6.2 Orbital lifetime

Total life in years from deployment (life after a 2-year mission in brackets); **bold** = exceeds the FCC 5-year rule.

| Deployment | Mass | Wings deployed: nominal / conservative / bound | Wings failed: nominal / conservative / bound |
|------------|------|-----------------------------------------------|---------------------------------------------|
| 450 km | 1.6 kg | 2.9 (0.9) / 3.5 (1.5) / 3.7 (1.7) | 4.5 (2.5) / 5.1 (3.1) / 5.5 (3.5) |
| 450 km | 2.0 kg | 3.8 (1.8) / 4.4 (2.4) / 4.6 (2.6) | 5.2 (3.2) / 5.8 (3.8) / 6.9 (4.9) |
| 475 km | 1.6 kg | 4.9 (2.9) / 5.5 (3.5) / 6.3 (4.3) | 6.0 (4.0) / 6.8 (4.8) / **9.5 (7.5)** |
| 475 km | 1.8 kg | 5.2 (3.2) / 5.8 (3.8) / **7.1 (5.1)** | 6.3 (4.3) / **7.2 (5.2)** / **10.7 (8.7)** |
| 475 km | 2.0 kg | 5.5 (3.5) / 6.2 (4.2) / **7.9 (5.9)** | 6.5 (4.5) / **7.6 (5.6)** / **11.8 (9.8)** |
| 500 km | 1.6 kg | 6.1 (4.1) / 7.0 (5.0) / **10.5 (8.5)** | **7.1 (5.1)** / **8.8 (6.8)** / **15.9 (13.9)** |
| 500 km | 1.8 kg | 6.4 (4.4) / **7.4 (5.4)** / **11.8 (9.8)** | **7.5 (5.5)** / **9.5 (7.5)** / **17.9 (15.9)** |
| 500 km | 2.0 kg | 6.7 (4.7) / **7.9 (5.9)** / **13.2 (11.2)** | **7.8 (5.8)** / **10.5 (8.5)** / **19.8 (17.8)** |
| 525 km | 1.6 kg | **7.2 (5.2)** / **9.1 (7.1)** / **17.3 (15.3)** | **8.7 (6.7)** / **14.7 (12.7)** / **26.0 (24.0)** |
| 525 km | 1.8 kg | **7.6 (5.6)** / **9.9 (7.9)** / **19.4 (17.4)** | **9.3 (7.3)** / **16.3 (14.3)** / **29.3 (27.3)** |
| 525 km | 2.0 kg | **7.9 (5.9)** / **11.1 (9.1)** / **21.6 (19.6)** | **10.2 (8.2)** / **17.4 (15.4)** / **32.5 (30.5)** |

**Highest deployment altitude that meets the FCC rule after a 2-year mission** (bisection):

| Configuration | Mass | Nominal | Conservative | Bound |
|---------------|------|---------|--------------|-------|
| Wings deployed | 1.6 / 1.8 / 2.0 kg | 520 / 513 / 508 km | 500 / 493 / 488 km | 480 / 475 / 470 km |
| Wings failed | 1.6 / 1.8 / 2.0 kg | 497 / 491 / 485 km | 478 / 472 / 467 km | 461 / 455 / 451 km |

**Shortest life** (NOAA high band and a strong cycle 26, maximum 180; 1.6 kg; largest area): 450 km 1.6 yr, 460 km
2.1 yr, 465 km 2.4 yr, 470 km 2.8 yr, 475 km 3.2 yr. Below about 460 km the 2-year design life (and Level 3) is at risk.

### 6.3 NASA 25-year rule (4.6-1a)

Met at ≤ 500 km in every case (worst 17.8 yr post-mission). **At the 525 km cap it fails** with wings failed under the
bound projection for 1.8 kg (27.3 yr) and 2.0 kg (30.5 yr). The previous revision's "compliant in all cases tabulated"
held only for 1.6 kg. (8719.14C Req. 4.6-1a has no separate 30-years-after-launch limit; the previous revision's
statement of one is withdrawn.)

### 6.4 FCC 5-year rule with P ≥ 0.9 (97.207(g)(1)(vii)(D)(1))

The rule asks for re-entry "as soon as practicable, but no later than five years following completion of the mission",
with a probability of success of at least 0.9. Probability is computed by Monte Carlo
(`software/simulation/cdr/sma/reliability_model.py`) over the spacecraft's failure time (reliability model of
`docs/environment/RELIABILITY_ANALYSIS.md` v2.0) and the wings-failed probability (1.6 %, dominated by the one burn
driver both wings share), at 1.8 kg, for two readings of "completion of the mission":

- **Reading A:** the mission ends at 2 years or at the earlier loss of the station (a spacecraft that dies early has
  completed its mission early).
- **Reading B:** the mission ends at the planned 2 years whatever happens.

| Deployment | Solar | P, reading A | P, reading B |
|------------|-------|--------------|--------------|
| 460 km | nominal / conservative | 1.00 / 0.99 | 1.00 / 1.00 |
| 470 km | nominal / conservative | 0.99 / **0.70** | 1.00 / 1.00 |
| 475 km | nominal / conservative | **0.73** / **0.65** | 1.00 / 0.98 |
| 485 km | nominal / conservative | **0.67** / **0.59** | 1.00 / 0.98 |
| 500 km | nominal / conservative | **0.59** / **0.00** | 0.98 / **0.00** |

Under reading A the probability falls with the early-failure probability, because a dead spacecraft starts the 5-year
clock at once and a 475 km orbit lasts more than 5 years. Which reading the FCC applies to a failed amateur satellite is
not stated in the rule text; it is a question for the FCC consultation already planned (D3).

### 6.5 Conclusions and recommendation

1. **The 525 km hard cap (BASELINE §3, MRD-013) is not compliant with the FCC rule in any case modelled**, and fails the
   NASA 25-year rule in the bounding wings-failed case at ≥ 1.8 kg.
2. **The 500 km target is compliant only under the nominal solar projection and reading B**; with the conservative
   projection 8719.14C requires, it fails for ≥ 1.8 kg even with the wings deployed.
3. **Recommended band: 465–485 km, target 475 km**, with a **hard cap of 485 km**: at 485 km the conservative,
   wings-deployed case meets the rule up to 2.0 kg (488 km limit) and the nominal wings-failed case up to 2.0 kg
   (485 km), so P ≥ 0.98 under reading B. If the FCC adopts reading A, the cap becomes **470 km** (P ≥ 0.99 nominal;
   0.70 conservative) and the floor stays at 465 km for the 2-year design life. This is a change to BASELINE §3 and
   MRD-013, raised to the chair (RID ODA-01).
4. If no integrator offers ≤ 485 km (SpaceX's drop-off menu on open 2027 flights is 500–600 km, BASELINE §3), the
   options are: an orbital transfer vehicle drop-off at the band (integrators with OTVs were in the D7 request list);
   adding drag area (a deployable drag device was removed from the baseline; it would need to roughly double the
   average area to make 525 km compliant); or the ISS backup (compliant, but Level 3 unachievable).

**4.6-2, 4.6-3:** N/A. **4.6-4 (≥ 0.9):** the passive disposal needs only the wings, and the wings-deployed case meets
the rule in the recommended band; P(both wings fail) = 0.016. A second burn driver (one per wing) reduces it to ≈ 0.001
(SQSO-FMEA-002 HF-13). **4.6-5 summary:** natural decay from 465–485 km in about 4–6 years nominal; uncontrolled re-entry.

## 7. Re-entry Hazards (Req. 4.7-1) and Hazardous Materials

### 7.1 Method

`software/simulation/cdr/sma/reentry_casualty.py` implements the class of model that DAS uses (NASA-STD-8719.14C §4.7.4):

- 3-DOF ballistic flight of the intact spacecraft from 120 km to the **78 km breakup altitude**; every component then
  released at that state (7.19 km/s, flight-path angle −1.28°) at 300 K, with no credit for the time spent inside its
  parent (conservative).
- Heating: free-molecular 0.5·α·ρ·v³ (α = 0.9) bridged to the Detra–Kemp–Riddell stagnation-point correlation
  (1.1035 × 10⁸ / √Rn · √(ρ/1.225) · (v/7925)^3.15 W/m²), applied to the tumbling average projected area with a 0.8
  continuum shape factor; re-radiation from the whole surface; **no oxidation heating** (conservative, notably for
  titanium and steel). Atmosphere NRLMSIS 2.1.
- Demise when the absorbed heat reaches m (c_p (T_m − 300 K) + h_f). Material data: Ti-6Al-4V T_m 1943 K, h_f 393 kJ/kg;
  stainless 1700 K, 286 kJ/kg; steel can 1800 K; aluminium 867 K; NiFe 1720 K; alumina-class ceramic 2327 K, 1070 kJ/kg.
- Survivors flown to the ground; those with impact energy > 15 J count, with DCA = Σ(0.6 + √A)² (eq. 4.7-1) and
  E = DCA × PD (eq. 4.7-2).
- **Population density** under a 97.4° orbit in the re-entry era (≈ 2034): 1.32 × 10⁻⁵ per m², from a 10° latitude-band
  population table (built from Kummu and Varis, *Applied Geography* 31 (2011) 495–507, Fig. 1, ± 1 point per band)
  scaled to 8.8 billion people (UN WPP 2024 medium variant). **1:10,000 corresponds to DCA = 7.6 m²** (band 3.8–15 m² for
  a factor-2 error in PD).

### 7.2 Results

| Component | Nominal heating | Heating halved (sensitivity) |
|-----------|-----------------|------------------------------|
| Aluminium frame, spreader, clamps, Al gyro case, copper coils | Demise at 72–77 km | Demise at 72–76 km |
| 18650 can only (8 g steel) | Demise 76 km | Survives, 11 m/s, 0.5 J |
| 18650 **whole cell as a 50 g steel cylinder** (bounding) | Demise 71 km | **Survives, 27 m/s, 18.7 J > 15 J** |
| Stainless fasteners, hinge pins, springs, connector bodies, antenna tapes | Demise 76–78 km | Survive, ≤ 21 m/s, ≤ 0.5 J |
| **Ti-6Al-4V screws** (0.3 g) / standoffs (1.5 g) | Demise 77 / 75 km | Survive, 12 / 21 m/s, < 0.3 J |
| Gyro as stainless (bounding) | Demise 76 km | Survives, 17 m/s, 1.5 J |
| GNSS ceramic patch | **Survives**, 19 m/s, 1.8 J | Survives, 1.8 J |
| Magnetorquer rods as solid NiFe (bounding) | Demise 75 km | Survive, 21 m/s, 3.7 J |

- **Nominal heating:** one survivor (the ceramic patch) at 1.8 J; **total DCA 0 m²; casualty risk ≈ 0.**
- **Heating halved, every bounding representation at once:** only the four cells modelled as solid steel exceed
  15 J; DCA = 4 × (0.6 + √0.00107)² = 1.60 m²; **risk 1:47,000**, compliant with margin ×4.7. With the cells as cans,
  no fragment exceeds 15 J.
- The **Ti-6Al-4V screws do not change the result**: even when they survive, at 0.3 g and 12 m/s they carry 0.02 J, far
  below 15 J. The stainless parts are likewise below 15 J by two orders of magnitude.

**4.7-1:** compliant by analysis, worst case 1:47,000. **[DAS]** DAS models the battery cells, the patch and the Ti
parts explicitly; report its result here.

**Hazardous materials:** Li-ion cells (NCA chemistry, ≈ 50 Wh at launch; passivated or protection-controlled at end of
mission, §4). No propellants, toxics or radioactive materials.

## 8. Tether Missions (Req. 4.8-1)

N/A.

---

## Open Items Before Filing

| # | Item | Owner |
|---|------|-------|
| 1 | Obtain NASA DAS 3.2.6 (NASA Software Catalog usage agreement) and reproduce §4–§7 with the controlled MEL | S&MA |
| 2 | Decide the deployment band and cap (RID ODA-01) and put it in the launch contract | PI, launch |
| 3 | Design and implement the non-resettable charge-path disconnect (F1, two independent signals) and the recharge lockout | EPS, FSW |
| 4 | Ask the FCC (D3 consultation) which reading of "completion of the mission" applies to a failed spacecraft | Licensee |
| 5 | Replace the cycle-26 assumption with the NOAA/NASA prediction when issued; DAS uses its own solar-flux table | S&MA |
| 6 | Separate burn drivers for the two wings (lowers the wings-failed probability from 1.6 % to ≈ 0.1 %) | EPS |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| Draft A | 2026-10-14 | New ODAR (funding audit); supersedes SQSO-ODAR-001 Rev A |
| Draft A.1 | 2026-10-16 | §2 notes the re-issued mass budget (TB-06) |
| Draft B | 2026-10-27 | CDR S&MA review: average cross-sections by the 8719.14C method (wings deployed 0.0247 m², not 0.028); masses 1.6/1.8/2.0 kg; NOAA-anchored nominal and conservative solar projections; FCC 5-year compliance by altitude and its probability with early failure; 525 km cap and 500 km target found non-compliant, 465–485 km recommended; 4.4-1 fault tree (1.4 × 10⁻⁴); 4.5-1 from the public catalogue (≤ 1.2 × 10⁻⁴); re-entry demise model with the Ti-6Al-4V and stainless parts (no fragment > 15 J nominal; 1:47,000 worst); "no titanium" and the 30-year-after-launch limit withdrawn |
