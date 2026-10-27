# Orbital Debris Assessment Report (ODAR)
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-REG-018 (supersedes SQSO-ODAR-001, `docs/compliance/ORBITAL_DEBRIS_ASSESSMENT.md`)
**Revision:** Draft D.3
**Date:** 2026-10-27
**Format:** NASA-STD-8719.14C (approved 2021-11-05) Appendix A.1, in the section order of the NASA Small Spacecraft Technology Program *ODAR & End of Mission Plan Instructional Guide* (spacecraft Sections 1–8; launch-vehicle Sections 9–14 are the launch provider's)
**Regulatory use:** basis for the orbital-debris disclosures in the FCC pre-space notification, 47 CFR 97.207(g)(1)(i)–(vii) ([draft](FCC_97-207_PRE_SPACE_NOTIFICATION.md))
**Status:** **NASA DAS 3.2.x has not been run.** DAS is distributed under a NASA software usage agreement and runs on Windows; neither was available to the CDR review. The licensee requests the agreement by 2026-11-06; S&MA runs DAS by 2027-01-08 so that Draft E carries the DAS results into the early pre-space filing of 2027-01-19 (`docs/regulatory/REGULATORY_ROADMAP.md` Rev D §7; open item 1). Every number here comes from the project's own models, which follow the methods of NASA-STD-8719.14C (cited section by section) and are in `software/simulation/cdr/sma/` and `software/simulation/orbit_lifetime/`. Values tagged **[DAS]** must be reproduced in DAS 3.2.6 (or a higher-fidelity tool, as the FCC rule allows) with the controlled mass properties before filing.

---

## Self-Assessment Summary

| Requirement (NASA-STD-8719.14C) | Topic | Result (project models) | Margin | Status |
|---------------------------------|-------|-------------------------|--------|--------|
| 4.3-1a, 4.3-1b | Released debris in LEO | None released | — | Compliant |
| 4.3-2 | Debris near GEO | N/A | — | N/A |
| 4.4-1 | Accidental explosion < 0.001 | **5.8 × 10⁻⁵** over a 10-year bound, with the latched recharge lockout now designed (1.4 × 10⁻⁴ without it) | ×17 | Compliant by analysis (§4) |
| 4.4-2 | Passivation design | Option 2 (energy controlled by the battery protections) met by design; option 1 (two-key passivation telecommand, SQSO-ICD-013 v0.10 §6.4, firing the one-shot charge disconnect F1 of SQSO-EPS-BRD-001 §7 while still transmitting, then reporting it, then TX-OFF) implemented in the flight software and designed in the EPS, **not yet run on hardware** (V-EPS-8) | — | Compliant via option 2; option 1 to be verified on the EM (§4) |
| 4.4-3, 4.4-4 | Planned breakups | None | — | N/A |
| 4.5-1 | Collision with objects ≥ 10 cm < 0.001 | **≤ 1.2 × 10⁻⁴** (worst case: 525 km, 2.0 kg, wings failed, F10.7 = 70) | ×8 | Compliant by analysis **[DAS]** (§5) |
| 4.5-2 | Small-debris impact preventing disposal < 0.01 | Disposal is passive; no function is needed after wing deployment at T + 30 min | — | Compliant by design (§5) |
| 4.6-1a | Natural re-entry ≤ 25 yr after mission, **conservative solar projection** | ≤ 12.6 yr post-mission at ≤ 485 km in every case; **fails at 525 km** for ≥ 1.85 kg with wings failed under the F10.7 = 70 bound (28.1–30.5 yr) | — | Compliant at ≤ 500 km (§6) |
| 4.6-4 | Disposal reliability ≥ 0.9 | Passive; P(both wings fail) = **0.003** with four independent burn lines, one driver per resistor, as designed (§6.4; 0.016 with the former single driver for both wings); compliant if the wings-deployed case meets the rule | — | See FCC row |
| **47 CFR 97.207(g)(1)(vii)(D)(1)** | **Re-entry ≤ 5 yr after mission completion, P ≥ 0.9** | **Not robustly met at the former 500 km target**: compliant only with the nominal solar projection and only if the mission is not cut short; **fails at the former 525 km cap** in every case modelled | — | **Compliant at the baseline 475 km target and 485 km cap** (BASELINE Rev E §3, adopted from §6.5) under the planned-mission reading, P ≥ 0.997 with the conservative solar projection (§6.4); ≤ 460 km if an early failure counts as mission end (§6) |
| 4.7-1 / 97.207(g)(1)(vii)(D)(2) | Casualty risk < 1:10,000, fragments > 15 J | **No surviving fragment > 15 J** with nominal heating (the two W-Ni-Cu ballast discs survive at 6.9 J); worst sensitivity (heating halved, whole cells modelled as steel) **1:47,000**, 1:32,000 if the discs are also counted at their edge-on bound (15.8 J) | ≥ ×3.2 | Compliant by analysis **[DAS]** (§7) |
| 4.8-1 | Tethers | None | — | N/A |
| 97.207(g)(1)(v) | Trackability | Smallest dimension 10 cm (1U), presumed trackable | — | Compliant |

---

## 1. Program Management and Mission Overview

| Item | Entry |
|------|-------|
| Responsible organization / licensee | Space station licensee **KK6GDB** (individual amateur licensee), station KK6GDB-11 (decision D1, [BASELINE_PARAMETERS.md](../../BASELINE_PARAMETERS.md) §1, §9); host organization: none yet |
| Program manager / PI | `[name, affiliation]` |
| Mission | SMART-QSO: 1U amateur-radio CubeSat with a duty-gated U/V FM transponder, AX.25 telemetry beacon and a power-gated onboard AI experiment |
| Milestones | No independent PDR or CDR yet (independent PDR planned 2027-01-12, CDR 2027-09-28, `mission/SCHEDULE.md`); this is the draft from the 2026 design review · planning launch 2028-11-01 (BASELINE Rev E §3) |
| Launch vehicle and site | SpaceX Falcon 9 Transporter rideshare via a 1U integrator, Vandenberg SFB (baseline, D7); backup ISS deployment via the Voyager NRCSD |
| Deployment | Ejected from the EXOpod Nova at 1.10–1.48 m/s (1.51–2.00 kg in a 1U slot with its adapter; SQSO-LVR-001 LVR-031) into a circular SSO |
| Orbit | Circular, inclination ≈ 97.4°; **target 475 km, 465–485 km, not above 485 km** (BASELINE Rev E §3, which adopted the band this report recommends in §6.5; it was ≤ 500 km target, 525 km cap) |
| Mission duration | 2 years design life (12 months minimum) |
| Interaction with other spacecraft | Shared SSO altitude band; no proximity operations; no manoeuvring capability; conjunction data via 18 SDS / Space-Track (`SPACE_SAFETY_REGISTRATION_AND_CONJUNCTION_PLAN.md`) |

## 2. Spacecraft Description

**Physical:** 1U CubeSat, 100 × 100 × 113.5 mm stowed. Two deployable solar wings (83 × 98 mm panels hinged on the ±X
edges of +Y, deployed into the +Y plane: `hardware/eps/SOLAR_ARRAY_DESIGN.md` §3.1), an ISIS AntS antenna (two 2 m tape
elements of ≈ 0.5 m and two 70 cm elements of ≈ 0.17 m).

**Mass:** the controlled MEL (`docs/environment/MASS_BUDGET.md` v3.1, with the build-versus-buy selections, the CAD
heat-spreader pedestal and the 20 g inertia ballast of decision C-13) gives CBE **1.52 kg** and MEV **1.75 kg** (BASELINE
Rev E amended §4; v3.0, used for the runs of Draft D, gave 1.49 / 1.73 kg). This report uses **1.6 kg (nominal) and 1.85 kg (bound)** as
BASELINE §4 directs for lifetime analyses (the MEV lies between them; a heavier spacecraft lives longer, so the bound is
the conservative case for disposal), adds **2.0 kg (the CDS Rev 14.1 design limit)** so that no mass the spacecraft can
have is outside the analysis, and uses the v3.0 CBE, 1.49 kg, for the shortest-life case (§6.2; the current CBE, 1.52 kg (MASS_BUDGET v3.4), lives slightly longer, so
1.49 kg remains the bounding short case).

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

No beryllium is in the baseline. **Tungsten:** decision C-13 added two 10 g W-Ni-Cu inertia ballast discs (Ø14 × 3.6 mm,
ASTM B777 Class 3) on the +Z end frame, each on two M2 Ti-6Al-4V screws (MASS_BUDGET row 36); a tungsten heavy alloy
survives re-entry. Draft D.2 adds them to the §7 demise run: they reach the ground at 37 m/s with **6.9 J**, below
15 J (§7.2); they stay in the DAS run (open item 7); the earlier statement "no tungsten" is withdrawn. Titanium: the battery-mount screws (or standoffs) and the ballast screws; the
previous revision's statement "no titanium" is withdrawn.

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
| Over-discharge, then recharge | BMS and hardware under-voltage both fail, the pack is drained below 2 V (0.05), **and the latched recharge lockout fails** (0.01/yr latent), then recharged | **3.6 × 10⁻⁶** (9.0 × 10⁻⁵ without the lockout) |
| **Total** | | **5.8 × 10⁻⁵** |

Compliant (margin ×17). Draft D.1 carried 9.0 × 10⁻⁵ for the over-discharge row because the EPS had no recharge
lockout. It now has one (SQSO-EPS-BRD-001 D §5.4, RIDs EPS-12, SMA-06): the AFE's permanent fail at a cell below
2.00 V, or the EPS controller, sets a two-coil latching relay K1 that inhibits charging by two paths (charge-current set
point to zero and the AFE charge FET open); only the authenticated EPS_CLEAR_LOCKOUT (0x14, operational key, argument
`CHG!`) clears it, through two signals (the OBC's LOCK_CLR and the controller's UNLOCK, SRS-F129). The row therefore
needs a third latent failure, the lockout's, taken at the same 0.01/yr allocation as the other two protections (the
board FMEA's lockout modes, SQSO-FMEA-003 BF-EPS-21 to BF-EPS-24, fit inside it): λ_UV · λ_HW · λ_lock · t³/3 · 0.05 =
3.6 × 10⁻⁶ with t = 6 years of charging. This is the formula of `explosion_probability.py` with one more latent
factor; the script still carries the Draft D.1 chain and takes the factor at its next revision. The overcharge row assumes
the dissimilar secondary protector (EPS-11); without it the overcharge term is 3.6 × 10⁻⁴ and the total
**4.0 × 10⁻⁴** (margin ×2.5). A spurious lockout does not raise the probability (charging stops); the board FMEA's
lockout findings (SMA-B12) are about the mission, not about this term. The end-of-life disconnect now needs two
processors to arm (§4.4-2); it does not enter 4.4-1, which already assumes passivation in only half the missions.

**4.4-2 (passivation design).** NASA-STD-8719.14C Req. 4.4-2 accepts either (1) depleting stored energy and
disconnecting generation, or (2) controlling it to a level that cannot cause an explosion or deflagration large enough
to release debris or break up the spacecraft.

- **Option 2 is met by design:** a 50 Wh pack of four sealed 18650 cells, with over-voltage, under-voltage, over-current,
  short-circuit and temperature protections in hardware, cannot release debris by the analysis above even if never
  passivated (the 10-year bound already assumes no passivation half the time).
- **Option 1, the FCC's "permanent discharge state" (97.207(g)(1)(iii)), is the baseline (BASELINE §5, SYS-800).**
  The telecommands are implemented in the flight software and tested in the host simulation
  (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md` v0.10 §6.4; `test_passivation`); the EPS disconnect is designed
  (SQSO-EPS-BRD-001 §7); neither has run on hardware (V-EPS-8 on the EM). The sequence
  (`docs/operations/END_OF_LIFE_PLAN.md` Rev D §3):
  1. `PASSIVATE_ARM` (0x70), authenticated with the **operational key**, argument `EOL!`, arms for 600 s;
  2. `PASSIVATE_EXECUTE` (0x71), authenticated with the **offline recovery key**, within those 600 s (two keys from two
     custodians: a two-person rule for an irreversible act); no first-year lockout;
  3. on execute, **while the spacecraft still transmits**: in sunlight, the non-resettable charge-path disconnect
     (fuse F1 between the solar-array trackers and the pack, fired only through both the arm latch and EOL_FIRE,
     held 60–120 s, up to three attempts; since SQSO-EPS-BRD-001 D the latch is set only by the OBC's EOL_ARM **and**
     the EPS controller's EOL_KEY, raised after a keyed I2C sequence, so no single OBC pin or GPIO fault can arm and
     fire; a dead EPS controller now prevents passivation, and option 2 is then the claim, SQSO-FMEA-003 BF-EPS-48); then the final report, three housekeeping frames a minute apart with the EPS
     `EOL_STATE` (disconnect fired, fuse intact) and the attempts, so the ground sees the result; then, 180 s after the
     first report, the TX-OFF latch and a passivation inhibit that nothing clears (an unconfirmed fire is reported and
     silenced the same way). The loads stay on and drain the pack to the BMS under-voltage cut-off, after which the
     receiver goes dark. (Draft D described TX-OFF first, as SQSO-ICD-013 up to v0.9 did.)
- **The passivation command is the 4.4-2 control.** The autonomous backup (M = 180 days after the loss-of-contact
  inhibit, BASELINE §7, proposed, not implemented) is **not needed** for 4.4-2: it would act only when the bus works but
  no command can be received (receiver or key failure, about 5 % of missions in the reliability model), and in that case
  option 2 already holds, since the explosion probability above assumes no passivation in half the missions. Being
  irreversible and triggered by an absence of commands, it adds a premature-passivation failure mode for little debris
  benefit. Recommendation: do not implement it (for the chair; D3 consultation).
- If the spacecraft fails before end of mission, passivation is impossible and option 2 is the claim.

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
| 475 km | 1.6 kg | nominal | 3.5 × 10⁻⁵ / 2.2 × 10⁻⁵ |
| 475 km | 2.0 kg | F10.7 = 70 bound | 5.4 × 10⁻⁵ / 3.5 × 10⁻⁵ |
| 485 km (cap) | 2.0 kg | F10.7 = 70 bound | 8.0 × 10⁻⁵ / 5.2 × 10⁻⁵ |
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
**2028-11-01** (the planning launch date, BASELINE Rev E §3; Draft C used 2028-06-01), re-entry at 180 km (`software/simulation/orbit_lifetime/orbit_lifetime.py`; scenarios and cross-sections in
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
(it is slightly lower, i.e. conservative), so the funding-audit lifetime table is reproduced exactly with its inputs
(A = 0.028 m², 1.6 kg, deployment 2028-06-01: 500 km 5.1 / 9.3 yr wings deployed, 6.2 / 15.8 yr wings failed). The
differences below come from the cross-section, the masses, the cycle-26 assumption and the deployment date: deploying
on 2028-11-01 instead of 2028-06-01 shortens the decaying lives by 0.1–0.3 yr (the spacecraft meets the cycle-26
maximum earlier in its decay) and leaves the F10.7 = 70 bound unchanged.

### 6.2 Orbital lifetime

Total life in years from deployment (life after a 2-year mission in brackets); **bold** = exceeds the FCC 5-year rule.

| Deployment | Mass | Wings deployed: nominal / conservative / bound | Wings failed: nominal / conservative / bound |
|------------|------|-----------------------------------------------|---------------------------------------------|
| 450 km | 1.6 kg | 3.2 (1.2) / 3.7 (1.7) / 3.7 (1.7) | 4.5 (2.5) / 5.0 (3.0) / 5.5 (3.5) |
| 450 km | 2.0 kg | 4.0 (2.0) / 4.4 (2.4) / 4.6 (2.6) | 5.1 (3.1) / 5.7 (3.7) / 6.9 (4.9) |
| 465 km | 1.85 kg | 4.7 (2.7) / 5.2 (3.2) / 5.9 (3.9) | 5.6 (3.6) / 6.4 (4.4) / **8.9 (6.9)** |
| 475 km | 1.6 kg | 4.8 (2.8) / 5.3 (3.3) / 6.3 (4.3) | 5.7 (3.7) / 6.6 (4.6) / **9.5 (7.5)** |
| 475 km | 1.85 kg | 5.1 (3.1) / 5.7 (3.7) / **7.3 (5.3)** | 6.1 (4.1) / **7.1 (5.1)** / **10.9 (8.9)** |
| 475 km | 2.0 kg | 5.3 (3.3) / 6.0 (4.0) / **7.9 (5.9)** | 6.2 (4.2) / **7.4 (5.4)** / **11.8 (9.8)** |
| 485 km | 1.6 kg | 5.3 (3.3) / 5.9 (3.9) / **7.7 (5.7)** | 6.2 (4.2) / **7.2 (5.2)** / **11.7 (9.7)** |
| 485 km | 1.85 kg | 5.6 (3.6) / 6.3 (4.3) / **9.0 (7.0)** | 6.5 (4.5) / **7.9 (5.9)** / **13.5 (11.5)** |
| 485 km | 2.0 kg | 5.7 (3.7) / 6.6 (4.6) / **9.7 (7.7)** | 6.7 (4.7) / **8.2 (6.2)** / **14.6 (12.6)** |
| 500 km | 1.6 kg | 5.9 (3.9) / 6.8 (4.8) / **10.5 (8.5)** | 6.8 (4.8) / **8.6 (6.6)** / **15.9 (13.9)** |
| 500 km | 1.85 kg | 6.2 (4.2) / **7.3 (5.3)** / **12.2 (10.2)** | **7.3 (5.3)** / **9.6 (7.6)** / **18.3 (16.3)** |
| 500 km | 2.0 kg | 6.4 (4.4) / **7.6 (5.6)** / **13.2 (11.2)** | **7.5 (5.5)** / **10.4 (8.4)** / **19.8 (17.8)** |
| 525 km | 1.6 kg | 6.9 (4.9) / **8.9 (6.9)** / **17.3 (15.3)** | **8.4 (6.4)** / **14.7 (12.7)** / **26.0 (24.0)** |
| 525 km | 1.85 kg | **7.3 (5.3)** / **10.1 (8.1)** / **20.0 (18.0)** | **9.3 (7.3)** / **16.4 (14.4)** / **30.1 (28.1)** |
| 525 km | 2.0 kg | **7.6 (5.6)** / **11.0 (9.0)** / **21.6 (19.6)** | **10.0 (8.0)** / **17.2 (15.2)** / **32.5 (30.5)** |

**Highest deployment altitude that meets the FCC rule after a 2-year mission** (bisection):

| Configuration | Mass | Nominal | Conservative | Bound |
|---------------|------|---------|--------------|-------|
| Wings deployed | 1.6 / 1.85 / 2.0 kg | 527 / 518 / 514 km | 503 / 496 / 491 km | 480 / 473 / 470 km |
| Wings failed | 1.6 / 1.85 / 2.0 kg | 503 / 495 / 491 km | 482 / 474 / 470 km | 461 / 454 / 451 km |

**Shortest life** (NOAA high band and a strong cycle 26, maximum 180; largest area): at the CBE, 1.49 kg: 450 km 1.7 yr,
460 km 2.3 yr, 465 km 2.6 yr, 470 km 3.0 yr, 475 km 3.4 yr, 485 km 4.1 yr; at 1.6 kg: 1.9, 2.5, 2.9, 3.3, 3.7 and
4.3 yr. Below about 455 km the 2-year design life (and Level 3) is at risk.

### 6.3 NASA 25-year rule (4.6-1a)

Met at ≤ 500 km in every case (worst 17.8 yr post-mission; at the 485 km cap, worst 12.6 yr). **At 525 km (the former
cap) it fails** with wings failed under the bound projection for 1.85 kg (28.1 yr) and 2.0 kg (30.5 yr). Draft A's
"compliant in all cases tabulated" held only for 1.6 kg. (8719.14C Req. 4.6-1a has no separate 30-years-after-launch limit; the previous revision's
statement of one is withdrawn.)

### 6.4 FCC 5-year rule with P ≥ 0.9 (97.207(g)(1)(vii)(D)(1))

The rule asks for re-entry "as soon as practicable, but no later than five years following completion of the mission",
with a probability of success of at least 0.9. Probability is computed by Monte Carlo
(`software/simulation/cdr/sma/reliability_model.py`) over the spacecraft's failure time (reliability model of
`docs/environment/RELIABILITY_ANALYSIS.md` v2.1) and the wings-failed probability, at 1.85 kg (the BASELINE §4 bound),
deployment 2028-11-01, for two readings of "completion of the mission".

**Wings-failed probability: 0.29 %** (Draft C: 1.6 %). The deployment of the EPS board (SQSO-EPS-BRD-001 §9; BASELINE
Rev E §4) has two burn resistors per wing, each on its own 0.8 A driver and its own OBC line (PB10 → W1A, PE6 → W1B,
PB9 → W2A, PE7 → W2B), burned one at a time. A wing stays stowed if its hinge or restraint fails (0.030 per wing, the
v2.0 allocation, unchanged) or both of its drivers fail (0.005 each); both wings stay stowed if both wings fail
independently (0.030² ≈ 0.09 %) or the supply the four drivers share fails: the SW6 load switch, the U17 burn buck and
the DEPLOY_EN line, with the deployment sequence in software, which a ground-commanded retry covers (0.2 %, allocated).
The shared supply is now the larger term; the consistency pass's ≈ 0.1 % counted only the independent part. The
reliability model gives 0.0029 (closed form 0.0029; `reliability_model.py --design shared` reproduces Draft C's 0.016).

The two readings:

- **Reading A:** the mission ends at 2 years or at the earlier loss of the station (a spacecraft that dies early has
  completed its mission early).
- **Reading B:** the mission ends at the planned 2 years whatever happens.

| Deployment | Solar | Life, wings deployed / failed (yr) | P, reading A | P, reading B |
|------------|-------|-------------------------------------|--------------|--------------|
| 450 km | nominal / conservative | 3.7 / 4.9; 4.2 / 5.4 | 1.00 / 1.00 | 1.00 / 1.00 |
| 460 km | nominal / conservative | 4.4 / 5.4; 4.9 / 6.1 | 1.00 / 1.00 | 1.00 / 1.00 |
| 465 km | nominal / conservative | 4.7 / 5.6; 5.2 / 6.4 | 1.00 / **0.74** | 1.00 / 1.00 |
| 470 km | nominal / conservative | 4.9 / 5.8; 5.5 / 6.7 | 1.00 / **0.71** | 1.00 / 1.00 |
| 475 km | nominal / conservative | 5.1 / 6.1; 5.7 / 7.1 | **0.75** / **0.67** | 1.00 / 1.00 (0.997) |
| 485 km | nominal / conservative | 5.6 / 6.5; 6.3 / 7.9 | **0.69** / **0.61** | 1.00 / 1.00 (0.997) |
| 500 km | nominal / conservative | 6.2 / 7.3; 7.3 / 9.6 | **0.62** / **0.00** | 1.00 (0.997) / **0.00** |

Under reading B the only failing trials at 475–485 km are the wings-failed ones under the conservative projection
(life 7.1–7.9 yr against 7 yr), so P = 1 − 0.0029 = 0.997; with Draft C's 1.6 % it was 0.98. Under reading A the
probability falls with the early-failure probability, because a dead spacecraft starts the 5-year clock at once and a
475 km orbit lasts more than 5 years; the wings-failed probability hardly enters. Which reading the FCC applies to a failed amateur satellite is
not stated in the rule text; it is a question for the FCC consultation already planned (D3).

### 6.5 Conclusions and recommendation

1. **The 525 km hard cap (BASELINE §3, MRD-013) is not compliant with the FCC rule in any case modelled**, and fails the
   NASA 25-year rule in the bounding wings-failed case at ≥ 1.85 kg.
2. **The 500 km target is compliant only under the nominal solar projection and reading B**; with the conservative
   projection 8719.14C requires, it fails for ≥ 1.85 kg even with the wings deployed.
3. **Recommended band: 465–485 km, target 475 km**, with a **hard cap of 485 km**: at 485 km the conservative,
   wings-deployed case meets the rule up to 2.0 kg (491 km limit) and the nominal wings-failed case up to 2.0 kg
   (491 km), so P ≥ 0.997 under reading B (the conservative wings-failed case fails above 474 km at 1.85 kg, but carries
   only the 0.29 % wings-failed probability). If the FCC adopts reading A, the cap becomes **460 km** (P ≥ 0.99 under
   both projections; 465 km gives 0.74 conservative), with the 2-year design life at risk below about 455 km. This is a change to BASELINE §3 and
   MRD-013, raised to the chair (RID ODA-01); **adopted** in BASELINE Rev E and MRD-013 Rev E (CR-2026-007).
4. **Even the wings-deployed case fails under the bound projection at the target** (chair note C-2): with F10.7 = 70
   held throughout, 475 km at 1.85 kg lives 7.3 years, 5.3 years after the 2-year mission (§6.2), as does 485 km at
   every mass. Compliance therefore rests on the nominal and conservative projections that NASA-STD-8719.14C uses; the
   bound is a sensitivity case, stated as such in the pre-space notification (vii).
5. If no integrator offers ≤ 485 km (SpaceX's drop-off menu on open 2027 flights is 500–600 km, BASELINE §3), the
   options are: an orbital transfer vehicle drop-off at the band (integrators with OTVs were in the D7 request list);
   adding drag area (a deployable drag device was removed from the baseline; it would need to roughly double the
   average area to make 525 km compliant); or the ISS backup (compliant, but Level 3 unachievable).

**4.6-2, 4.6-3:** N/A. **4.6-4 (≥ 0.9):** the passive disposal needs only the wings, and the wings-deployed case meets
the rule in the recommended band; P(both wings fail) = 0.0029 with one driver per burn resistor on four independent
lines (SQSO-EPS-BRD-001 §9; §6.4), against 0.016 with the former single driver for both wings (SQSO-FMEA-002 HF-13).
**4.6-5 summary:** natural decay from 465–485 km in 4.3–5.7 years with the wings deployed and the nominal solar
projection (6.4 years at 485 km, 1.85 kg, conservative); uncontrolled re-entry.

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
  stainless 1700 K, 286 kJ/kg; steel can 1800 K; aluminium 867 K; NiFe 1720 K; alumina-class ceramic 2327 K, 1070 kJ/kg;
  W-Ni-Cu heavy alloy (ASTM B777 Class 3, Draft D.2) taken as tungsten for survival: 18.0 g/cm³, c_p 160 J/kg·K, T_m 3695 K,
  h_f 285 kJ/kg, ε 0.35 (the Ni-Cu binder melts below 1700 K; the tungsten skeleton is taken to hold the shape, the survival case).
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
| **W-Ni-Cu inertia ballast discs** (2 × 10.0 g, Ø14 × 3.6 mm; Draft D.2) | **Survive**, 37 m/s, **6.9 J** | Survive, 37 m/s, 6.9 J |
| Ti-6Al-4V ballast screws M2 (4 × 0.15 g) | Demise 77 km | Demise 74 km |

- **Nominal heating:** one survivor (the ceramic patch) at 1.8 J; **total DCA 0 m²; casualty risk ≈ 0.**
- **Heating halved, every bounding representation at once:** only the four cells modelled as solid steel exceed
  15 J; DCA = 4 × (0.6 + √0.00107)² = 1.60 m²; **risk 1:47,000**, compliant with margin ×4.7. With the cells as cans,
  no fragment exceeds 15 J.
- **The W-Ni-Cu ballast discs survive in every case** (gross heat input 2.3–5.6 times the heat to melt them, but
  re-radiation keeps them below the tungsten melting point) but land at the terminal speed of a tumbling 10 g disc, 37 m/s, with **6.9 J**:
  below 15 J, no DCA. Terminal speed brackets: face-on 30 m/s (4.4 J), tumbling with C_D 0.8 41 m/s (8.5 J), stable
  edge-on fall 56 m/s (15.8 J), the last not a stable attitude for a disc. Counting both discs even at the edge-on bound
  adds 2 × (0.6 + √0.000117)² = 0.75 m²: alone 1:101,000, with the bounding cells 2.35 m², **1:32,000**, compliant
  (margin ×3.2). No demisable replacement is needed for 4.7-1. If DAS gives more than 15 J, the replacement would be a
  copper-alloy disc of the same 10 g (Ø20 × 3.7 mm brass or Ø20 × 3.6 mm bronze, same mounting; its demise to be run,
  and the mass properties to be checked by the mechanical designer). Model: `reentry_casualty.py` with the two discs and
  their screws as added parts (the run of Draft D.2; the parts list in the script takes them at its next revision).
- The **Ti-6Al-4V screws do not change the result**: even when they survive, at 0.3 g and 12 m/s they carry 0.02 J, far
  below 15 J. The stainless parts are likewise below 15 J by two orders of magnitude.

**4.7-1:** compliant by analysis, worst case 1:47,000 (1:32,000 with the ballast discs at their edge-on bound). **[DAS]**
DAS models the battery cells, the patch, the ballast discs and the Ti parts explicitly; report its result here.

**Hazardous materials:** Li-ion cells (NCA chemistry, ≈ 50 Wh at launch; passivated or protection-controlled at end of
mission, §4). No propellants, toxics or radioactive materials.

## 8. Tether Missions (Req. 4.8-1)

N/A.

---

## Open Items Before Filing

| # | Item | Owner |
|---|------|-------|
| 1 | Obtain NASA DAS 3.2.6 (NASA Software Catalog usage agreement, **requested by 2026-11-06**, in hand by 2026-12-15) and reproduce §4–§7 with the controlled MEL (MASS_BUDGET v3.1) **by 2027-01-08**, before the early pre-space filing of 2027-01-19, which needs the results ((g)(1)(ii), (iv)(A)(1)). **Pass criterion:** DAS reproduces this draft: re-entry ≤ 5 years after the 2-year mission at 485 km with 1.85 kg and the wings deployed (P ≥ 0.9, reading B); casualty risk ≤ 1:10,000 with no fragment > 15 J; 4.5-1 < 0.001. If DAS gives a longer life, the cap is lowered to the highest altitude DAS passes before filing (REGULATORY_ROADMAP Rev D §7) | Licensee (agreement); S&MA (run) |
| 2 | Put the deployment band and cap (decided: 465–485 km, 485 km cap, BASELINE Rev E §3; RID ODA-01) in the launch contract, the cap on the 3σ-high mean altitude of the drop-off orbit with the injection dispersion, and a 460 km quote for reading A (SQSO-OAP-001 §2.2) | PI, launch |
| 3 | Passivation: telecommands implemented (0x70/0x71, SQSO-ICD-013 v0.10 §6.4: the fire while transmitting, the final report, then TX-OFF); charge-path disconnect designed (F1, two independent signals, SQSO-EPS-BRD-001 §7); recharge lockout designed (K1, two-signal clear, SQSO-EPS-BRD-001 D §5.4; 4.4-1 re-run in Draft D.2); arm now also needs the EPS controller's EOL_KEY (Rev D); run V-EPS-8 and V-EPS-9 on the EM, and add the EM result to the pre-space notification's passivation statement (Rev A.3 describes the implemented sequence) | EPS, FSW, licensee |
| 4 | Ask the FCC (D3 consultation, REGULATORY_ROADMAP Rev D §6 Q5, by 2027-01-05) which reading of "completion of the mission" applies to a failed spacecraft | Licensee |
| 5 | Replace the cycle-26 assumption with the NOAA/NASA prediction when issued; DAS uses its own solar-flux table | S&MA |
| 6 | Separate burn drivers for the two wings: designed, one driver per resistor on four lines (SQSO-EPS-BRD-001 §9); §6.4 re-run in Draft D (wings-failed 0.29 %, of which 0.2 % is the shared burn supply SW6/U17); the board FMEA (SQSO-FMEA-003 BF-EPS-53, BF-EPS-55) confirms the shared elements: SW6, U17, V_BURN and its W7 feeds, and DEPLOY_EN, which also arms the AntS supply (finding SMA-B06) | EPS, S&MA |
| 7 | W-Ni-Cu inertia ballast discs (C-13): added to the demise model in Draft D.2 (survive, 6.9 J < 15 J; 1:32,000 worst with the edge-on bound, §7.2); keep them in the DAS run and confirm < 15 J there; if DAS exceeds 15 J, change to a 10 g copper-alloy disc and re-run (mechanical designer: mass properties) | S&MA |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| Draft A | 2026-10-14 | New ODAR (funding audit); supersedes SQSO-ODAR-001 Rev A |
| Draft A.1 | 2026-10-16 | §2 notes the re-issued mass budget (TB-06) |
| Draft B | 2026-10-27 | CDR S&MA review: average cross-sections by the 8719.14C method (wings deployed 0.0247 m², not 0.028); masses 1.6/1.8/2.0 kg; NOAA-anchored nominal and conservative solar projections; FCC 5-year compliance by altitude and its probability with early failure; 525 km cap and 500 km target found non-compliant, 465–485 km recommended; 4.4-1 fault tree (1.4 × 10⁻⁴); 4.5-1 from the public catalogue (≤ 1.2 × 10⁻⁴); re-entry demise model with the Ti-6Al-4V and stainless parts (no fragment > 15 J nominal; 1:47,000 worst); "no titanium" and the 30-year-after-launch limit withdrawn |
| Draft C | 2026-10-27 | CDR 2026 consistency pass: compliance summary and orbit row at the baseline 475 km target and 485 km cap (BASELINE Rev E, which adopted §6.5); one burn driver per resistor as designed (4.6-4, open items 2 and 6). Analysis values unchanged |
| Draft D | 2026-10-27 | CDR S&MA re-run at the Rev E baseline (RID SE-16): deployment 2028-11-01 (was 2028-06-01), masses 1.6 / 1.85 / 2.0 kg (was 1.8 kg bound) and CBE 1.49 kg for the shortest life, 465 and 485 km rows (§6.2, §5); §6.4 with the four-line deployment: wings-failed 0.29 % (was 1.6 %), reading B P = 0.997 at 475–485 km (was 0.98), reading A cap 460 km (was 470 km); §2 mass from MASS_BUDGET v3.0; 4.6-1a, 4.6-4, 4.6-5 and open item 6; §4 4.4-2 and open item 3 brought to SQSO-ICD-013 v0.8 (passivation implemented as 0x70/0x71, no first-year lockout) |
| Draft D.1 | 2026-10-27 | CDR round 3 (RIDs ODA-02, SE-14, REG-12; chair note C-2); analysis values unchanged: DAS agreement requested by 2026-11-06 and DAS run by 2027-01-08, before the early filing, with its pass criterion (status, open item 1); §2 mass from MASS_BUDGET v3.1 (CBE 1.51 kg, MEV 1.75 kg) and the W-Ni-Cu inertia ballast ("no tungsten" withdrawn; open item 7); §4 4.4-2 passivation in the implemented order (SQSO-ICD-013 v0.10: the fire while transmitting, the final report, then TX-OFF); §6.5 item 4: the F10.7 = 70 bound fails at 475 km, 1.85 kg even with the wings deployed (5.3 years after the mission) |
| Draft D.2 | 2026-10-27 | CDR round 3 S&MA (coordinator items): §7 W-Ni-Cu ballast discs added to the demise run (survive at 6.9 J, below 15 J; edge-on bound 15.8 J gives 1:32,000 worst; no demisable replacement needed), §2 tungsten note, open item 7; §4 4.4-1 re-run with the designed recharge lockout (over-discharge term 9.0 × 10⁻⁵ → 3.6 × 10⁻⁶, total 1.4 × 10⁻⁴ → 5.8 × 10⁻⁵, ×17) and the two-processor EOL arm (EOL_ARM and EOL_KEY, SQSO-EPS-BRD-001 D); open items 3 and 6 |
| Draft D.3 | 2026-10-27 | Documentation audit (post-CDR 2026): §1 ejection velocity from SQSO-LVR-001 LVR-031 (was 0.3–1.0 m/s) and the review and launch dates; §2 the current CBE 1.52 kg (was "v3.1 CBE, 1.51 kg"). Analysis values unchanged; DAS not run |
