# Orbital Debris Assessment Report (ODAR)
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-REG-018 (supersedes SQSO-ODAR-001, `docs/compliance/ORBITAL_DEBRIS_ASSESSMENT.md`)
**Revision:** Draft A
**Date:** 2026-10-14
**Format:** NASA-STD-8719.14C Appendix A.1, following the NASA Small Spacecraft Technology Program *ODAR & End of Mission Plan Instructional Guide* section order (spacecraft Sections 1–8; launch-vehicle Sections 9–14 are the launch provider's)
**Regulatory use:** basis for the orbital-debris disclosures in the FCC pre-space notification, 47 CFR 97.207(g)(1)(i)–(vii) ([draft](FCC_97-207_PRE_SPACE_NOTIFICATION.md))
**Status:** **NASA DAS 3.2.x has not been run.** The orbital lifetimes below come from the project's NRLMSIS-based estimator (`software/simulation/orbit_lifetime/`). Every value tagged **[DAS]** must be replaced with DAS 3.2.6 output (or a higher-fidelity tool, as the FCC rule allows) using the final mass properties before filing.

---

## Self-Assessment Summary

| Requirement | Topic | Status | Basis |
|-------------|-------|--------|-------|
| 4.3-1.a | Debris passing through LEO: ≤ 25 yr | Compliant (no released debris) | §3 |
| 4.3-1.b | < 100 object-years | Compliant (no released debris) | §3 |
| 4.3-2 | Debris passing near GEO | N/A (LEO) | §3 |
| 4.4-1 | Accidental explosion probability < 0.001 | Compliant by design; FMEA to finalize | §4 |
| 4.4-2 | Passivation at end of mission | Compliant by design (permanent charge-path disconnect); **implementation open** | §4 |
| 4.4-3 / 4.4-4 | Planned breakups | N/A (none) | §4 |
| 4.5-1 | Large-object collision probability < 0.001 | **Incomplete — DAS pending** (small cross-section; ≤ 7.4 yr life) | §5 |
| 4.5-2 | Small-debris impact preventing disposal < 0.01 | Compliant by design (disposal is passive) | §5 |
| 4.6-1(a) | Re-entry ≤ 25 yr after mission and ≤ 30 yr after launch (NASA) | Compliant (nominal and wings-failed cases) — **DAS confirmation pending** | §6 |
| 4.6-1(b)/(c) | Storage orbit / direct retrieval | N/A | §6 |
| 4.6-2 / 4.6-3 | GEO / MEO disposal | N/A (LEO) | §6 |
| 4.6-4 | Disposal reliability ≥ 0.9 | Compliant by design (passive drag) | §6 |
| 4.6-5 | Summary of de-orbit | §6 | §6 |
| 4.7-1 | Casualty risk < 1:10,000; no fragment > 15 J | **Incomplete — DAS pending**; design-to-demise review in §7 | §7 |
| 4.8-1 | Tethers | N/A (no tethers; drag/tether device removed from baseline) | §8 |
| **FCC 97.207(g)(1)(vii)(D)(1)** | **Re-entry ≤ 5 yr after mission completion, P ≥ 0.9** | **Compliant at ≤ 500 km** (nominal, both cases); 525 km only with wings deployed — **DAS pending** | §6 |

---

## 1. Program Management and Mission Overview

| Item | Entry |
|------|-------|
| Responsible organization / licensee | `[legal entity]`, space station licensee `[name, call sign]` (decision D1, [BASELINE_PARAMETERS.md](../../BASELINE_PARAMETERS.md) §9) |
| Program manager / PI | `[name, affiliation]` |
| Mission | SMART-QSO: 1U amateur-radio CubeSat with a duty-gated U/V FM transponder, AX.25 telemetry beacon and a power-gated onboard AI experiment |
| Milestones | PDR `[date]` · CDR `[date]` (no independent reviews held yet; see the funding audit) · integration `[date]` · launch NET 2028 |
| Launch vehicle and site | SpaceX Falcon 9 Transporter rideshare `[mission]`, Vandenberg SFB, via integrator `[ ]` (baseline); backup ISS deployment via NRCSD |
| Deployment | Ejected from a CubeSat dispenser at 0.3–1.0 m/s ([SpaceX User's Guide v11 §3.3.2](https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf)) into a circular SSO |
| Orbit | Apogee/perigee `[≤ 500]` km (525 km maximum), inclination ≈ 97.4° |
| Mission duration | 2 years design life (12 months minimum) |
| Interaction with other spacecraft | Shared SSO altitude band; no proximity operations; no maneuvering capability |

## 2. Spacecraft Description

**Physical:** 1U CubeSat, 100 × 100 × 113.5 mm stowed. Two deployable solar wings (one 1U face each), a deployable VHF
tape monopole (≈ 0.5 m) and a deployable UHF monopole (≈ 0.17 m). Total mass ≈ 1.44 kg CBE, ≤ 1.7 kg MEV (provisional
equipment list below).

| Item | Propulsion | Fluids | Pressure vessels | Pyrotechnics | Radioactive materials | Other stored energy |
|------|-----------|--------|------------------|--------------|-----------------------|---------------------|
| SMART-QSO | **None** | **None** | **None** | **None** (burn-wire / melt-wire releases) | **None** | Deployment springs/hinges (mechanical, released at T+30 min); Li-ion battery (below) |

**Electrical generation and storage:** 16 triple-junction solar cells. Battery: 4S1P Li-ion 18650, ≈ 50 Wh (cell
part number `[ ]`, UL/IEC listing `[ ]`). Protection: per-cell over-voltage (≤ 4.25 V), under-voltage, over-current
and short-circuit, with charge permitted only between 0 and +45 °C. Unpressurized commercial cells with internal CID/vent.

**Attitude control:** three air-core magnetorquers, magnetometer and coarse sun sensors. B-dot detumble, then
spin-stabilized coarse sun-pointing. No momentum wheels.

**Provisional master equipment list (for DAS input; replace with the controlled MEL):**

| Component | Qty | Mass (g, CBE) | Principal material | DAS shape (to model) |
|-----------|-----|---------------|--------------------|----------------------|
| Structure (rails, frame, panels) | 1 | 280 | Aluminum 7075-T6 / 6061-T6 | Box |
| EPS boards and body-mounted solar panels | 1 set | 180 | FR-4, copper, GaAs/Ge cells | Flat plates |
| Battery pack (4 × 18650 + holder + BMS) | 1 | 200 | Nickel-plated **steel** cell cans, Al/FR-4 holder | Cylinders ×4 |
| OBC | 1 | 45 | FR-4, copper | Flat plate |
| RF transceiver / transponder | 1 | 120 | FR-4, Al shield | Box |
| Antennas (VHF steel tape, UHF element) | 2 | 35 | Spring **steel** / beryllium-copper | Flat plates |
| Jetson Orin Nano module + carrier + heat spreader | 1 | 198 | FR-4, Si, Al | Box |
| ADCS sensors | 1 set | 25 | FR-4 | Flat plate |
| Magnetorquers (air-core coils, drivers) | 3 | 115 | Copper wire, polymer forms | Flat plates/rings |
| Solar wings, hinges, hold-downs | 2 | 110 | Al, FR-4, cells; **stainless** hinge pins | Flat plates |
| Harness | 1 | 80 | Copper, PTFE | — |
| Thermal (spreaders, tapes) | 1 | 30 | Al, polyimide | — |
| Fasteners | many | 25 | **Stainless steel** (A286/300-series) | Small cylinders |
| **Total CBE** | | **1,443** | | |

The previous assessment's demise table summed to 0.92 kg against a stated 1.33 kg (finding RG-06). This list is
complete for the baseline configuration and must be kept consistent with `docs/environment/MASS_BUDGET.md` once that
budget is re-issued.

## 3. Debris Released During Normal Operations (Req. 4.3-1, 4.3-2)

- Objects > 1 mm intentionally released: **none.** Deployables (antennas, wings) stay attached (CDS Rev 14.1 §2.1.1).
  Burn-wire restraint lines are retained by captive tie points; melt residue is < 1 mm.
- The drag sail/tether in the superseded ODAR has been **removed** from the baseline.
- **4.3-1.a, 4.3-1.b:** Compliant. **4.3-2:** N/A.

## 4. Intentional Breakups and Potential for Explosions (Req. 4.4-1 to 4.4-4)

**4.4-1 (accidental explosion probability < 0.001).** The only credible source is the Li-ion battery. Failure modes
(over-charge, over-discharge, internal short, external short, over-temperature charging, cell-to-cell propagation)
each need at least two independent faults (cell CID/vent plus BMS protection plus pack fuse) before a cell can rupture.
18650 cells vent through their CID/vent; at ≈ 12.5 Wh per cell, a rupture is not expected to fragment the spacecraft.
The battery FMEA (`docs/safety/`, to be extended to the battery) must document each failure mode. **Expected
probability: ≪ 0.001.**

**4.4-2 (passivation).** At end of mission, an authenticated telecommand (with an autonomous backup after loss of
contact; values with D3):

1. **Permanently disconnects the solar-array charge path** through a non-resettable latch (fuse or latching relay), so
   the battery cannot be recharged.
2. Disables all transmitters. Then normal loads discharge the battery to a **permanent discharge state** (97.207(g)(1)(iii)).

*Correction:* the superseded ODAR passivated only to "< 30 % SOC" with charging still possible. That does not meet
the FCC "permanent discharge state" language. **This design element does not yet exist in the EPS design or flight
software** (findings TB-01, SW-02). It is a closure item.

**4.4-3, 4.4-4 (planned breakups):** N/A.

## 5. Potential for On-Orbit Collisions (Req. 4.5-1, 4.5-2)

**4.5-1 (large-object collision probability < 0.001 over orbital lifetime).** **[DAS]** pending. Inputs: deployment ≤
500 km, mass 1.6 kg, mean cross-section 0.028 m² (wings deployed) / 0.0164 m² (wings failed), lifetime per §6. A
small cross-section and a lifetime of ≤ 7.4 years point to compliance, but **no value is claimed until DAS is run.**

**4.5-2 (small-debris impact preventing disposal < 0.01).** Disposal is passive atmospheric drag and requires no
spacecraft function after the wings deploy at T+30 min, so loss of control cannot prevent disposal. **Compliant by
design.** (DAS reports this as N/A for passive disposal.)

## 6. Post-Mission Disposal (Req. 4.6-1 to 4.6-5; FCC 97.207(g)(1)(vii))

**Method:** uncontrolled atmospheric re-entry by natural decay (4.6-1(a)). No maneuvers. No storage orbit or retrieval.

**Area-to-mass ratio** (tumbling mean area; 1.6 kg):

| Case | Area | A/m |
|------|------|-----|
| Case 1 — wings deployed (nominal) | 0.028 m² | 0.0175 m²/kg |
| Case 2 — wings failed to deploy | 0.0164 m² | 0.0103 m²/kg |

**Orbital lifetime** (NRLMSIS 2.1 estimator; Cd 2.2; deployment 2028-06-01; "nominal" uses an approximate solar-cycle
profile with an assumed cycle-26 amplitude; "bound" holds F10.7 at 70 throughout):

| Deployment | Case 1 nominal / bound | Case 2 nominal / bound | Time after a 2-yr mission, Case 1 / Case 2 (nominal) |
|------------|------------------------|------------------------|-------------------------------------------------------|
| 475 km | 4.0 / 5.5 yr | 5.2 / 9.4 yr | 2.0 / 3.2 yr |
| 500 km | 5.1 / 9.3 yr | 6.2 / 15.8 yr | 3.1 / 4.2 yr |
| 525 km | 6.0 / 15.2 yr | 7.4 / 25.9 yr | 4.0 / **5.4 yr** |

Reproduce with:
`python software/simulation/orbit_lifetime/orbit_lifetime.py --inc 97.4 --alt 475 500 525 --mass 1.6 --area 0.028 0.0164 --start 2028-06-01`.
A 1.8 kg bounding run adds ≈ 0.2–0.5 yr.

**4.6-1(a) (NASA: ≤ 25 yr after mission, ≤ 30 yr after launch):** compliant in all cases tabulated. The tightest is the
bound case at 525 km with wings failed: 25.9 yr after launch, which is 23.9 yr after a 2-year mission. That is inside
both limits but close to the 25-year limit, a further reason for the ≤ 500 km baseline.

**FCC 97.207(g)(1)(vii)(D)(1) (re-entry as soon as practicable, ≤ 5 yr after mission completion, P ≥ 0.9):** at
≤ 500 km, compliant in both cases on the nominal profile. At 525 km only Case 1 complies. The deployment altitude is
therefore a launch-contract requirement ([trade study](../../funding_audit/LAUNCH_RIDESHARE_TRADE_STUDY.md)). **The
≥ 0.9 probability demonstration must use DAS with its solar-flux projection.** Report the DAS result and the
low-activity sensitivity.

**4.6-2, 4.6-3:** N/A (LEO). **4.6-4 (disposal reliability):** passive. It needs no spacecraft function beyond wing
deployment, and Case 2 (wings failed) is analyzed and complies at ≤ 500 km. **4.6-5 (summary):** natural decay from ≤
500 km in ≈ 5–6 years (nominal), uncontrolled re-entry, no propulsive or device-based disposal.

## 7. Re-entry Hazards (Req. 4.7-1) and Hazardous Materials

**4.7-1(a):** casualty risk from surviving debris < 1:10,000, with no fragment impacting at > 15 J (CDS 2.4.3.1).
**[DAS]** pending.

Design-to-demise review of the MEL (§2). Items to model specifically in DAS because of high melting point or steel
construction:

| Item | Concern | Action |
|------|---------|--------|
| 18650 cell cans (nickel-plated steel, ×4) | Steel cans sometimes survive in DAS | Model as steel cylinders; report KE |
| Stainless fasteners, hinge pins | Small steel parts | Model aggregated |
| Spring-steel antenna tape | Thin; expected to demise | Model |
| Jetson module | Silicon/aluminum; no titanium | Model as Al box |

No titanium, beryllium or tungsten parts are in the baseline. The superseded ODAR's casualty calculation left out the
human cross-sectional area (0.36 m² per person), which understates casualty area by orders of magnitude. **It must not
be reused** (finding RG-06).

**Hazardous materials:** Li-ion cells (LiNiCoAlO₂ or similar, `[chemistry]`): ≈ 50 Wh at launch; passivated to permanent
discharge at end of mission; not expected to survive re-entry intact `[DAS]`. No propellants, toxics or radioactive
materials.

## 8. Tether Missions (Req. 4.8-1)

N/A. No tethers. The tether/drag-sail option in the superseded ODAR is removed from the baseline.

---

## Open Items Before Filing

| # | Item | Owner |
|---|------|-------|
| 1 | Obtain NASA DAS 3.2.6 (Software Usage Agreement via the NASA Software Catalog) and run Sections 5–7 with the controlled MEL | Systems |
| 2 | Implement the non-resettable charge-path disconnect (4.4-2) in EPS hardware and flight software | EPS + FSW |
| 3 | Battery FMEA for 4.4-1 | S&MA |
| 4 | Replace the provisional MEL with the controlled MEL; fix the deployment altitude from the launch contract | Systems |
| 5 | Replace the solar-cycle assumption with the current NOAA SWPC prediction for the sensitivity case | Systems |

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| Draft A | 2026-10-14 | New ODAR (funding audit); supersedes SQSO-ODAR-001 Rev A |
