# Structural Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-STR-001
**Version:** 1.5
**Date:** 2026-10-27
**Status:** Draft, CDR issue. Design description of the build-versus-buy structure; the analyses are in
`docs/environment/STRUCTURAL_ANALYSIS.md` (SQSO-SA-001), `docs/environment/MECHANISMS_ANALYSIS.md` (SQSO-MECH-001) and
`docs/environment/MASS_BUDGET.md` (SQSO-MB-001 v3.1); the CAD, mass properties and finite-element model of the design are
SQSO-MECH-CAD-001 Version B (`hardware/mechanical/MECHANICAL_DESIGN.md`, `model.py`, `fem.py`, `hinge.py`), which this
document summarises and does not repeat. No hardware exists

> **Funding audit (2026-10-16, TB-15).** Launch environments in this document come from one source: **SpaceX
> Rideshare Payload User's Guide v11, August 2026** ("UG v11",
> <https://storage.googleapis.com/rideshare-static/Rideshare_Payload_Users_Guide.pdf>; confirmed current on 2026-10-27)
> as the **interim provider, pending the launch contract and PI confirmation** (BASELINE §3, decision D7). The
> integrator's ICD will supersede it. The same environment is used by `docs/test_procedures/TP-ENV-002_Vibration.md` and
> `docs/compliance/LAUNCH_PROVIDER_ICD.md`.
>
> **CDR structures review (2026-10-27).** Version 1.3 replaces the custom 7075 frame with the bought frame of
> `docs/hardware/BUILD_VS_BUY.md` §2.2, sets the stack order and the board supports that the analyses require, and
> replaces the stale mass properties, factors of safety, switch count, separation-spring values and vent area.

---

## 1. Overview

This document defines the structural design for the SMART-QSO 1U CubeSat per CubeSat Design Specification (CDS)
Rev 14.1.

### 1.1 Design Requirements

| Requirement | Value | Source |
|-------------|-------|--------|
| Form factor | 1U (100 × 100 × 113.5 mm) | CDS Rev 14.1 §2.2.1, Appendix B |
| Mass | ≤ 2.00 kg. Estimate: CBE 1.52 kg (CR0002 rods) / 1.60 kg (air-core coils), MEV 1.75 / 1.85 kg, margin +14.0 / +7.9 % | SYS-002; CDS Rev 14.1 Table 1 (typical maximum); `docs/environment/MASS_BUDGET.md` v3.1 |
| CG location | ≤ 2 cm from the geometric centre on each axis. CAD: (+0.1, −0.4, −1.4) mm stowed, CBE | CDS Rev 14.1 §2.2.11, Table 2; MASS_BUDGET §5 |
| Deployed inertia (STR-01) | Iyy ≥ 1.03 × max(Ixx, Izz) with the wings and antennas deployed (target 1.05); major principal axis within 5° of +Y. CAD: 1.056 / 1.047 (CBE / MEV), axis 1.2° / 0.7°, with the 2 × 10 g Z-axis ballast (C-13) | MASS_BUDGET §2.1, §6; SQSO-MECH-CAD-001 §5 |
| Rail dimensions | ≥ 8.5 mm to the first protrusion | CDS Rev 14.1 §2.2.5 |
| Rail surface finish | Ra < 1.6 µm, hard anodized | CDS Rev 14.1 §2.2.6, §2.2.13 |
| Protrusions on ±X/±Y | ≤ 6.5 mm from the rail plane | CDS Rev 14.1 §2.2.3 |
| First natural frequency | > 40 Hz (no elastic mode below 40 Hz); design target ≥ 100 Hz for every element | UG v11 §4.1.1; BASELINE §8 |
| Random vibration | Protoflight test at the envelope of MPE + 3 dB and the GSFC-STD-7000B workmanship spectrum, 8.18 Grms on the interim levels, 1 min/axis (MPE 5.57 Grms; MPE + 3 dB 7.86 Grms) | UG v11 §4.1.6 Table 4-6, Table 6-2; BASELINE §8; `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §3.1; STRUCTURAL_ANALYSIS §4.2 |
| Quasi-static load | 10 g axial, 17 g lateral (RSS), or the 3σ random response at the item's mode if larger | UG v11 Table 4-1; STRUCTURAL_ANALYSIS §4.3 |
| Factors of safety | Protoflight: metallic 1.25 yield / 1.40 ultimate; composite and bonded 1.50 (2.00 at discontinuities); softgoods 2.00; joint separation 1.20 (1.25 battery mount) × fitting factor 1.15 | NASA-STD-5001B w/Change 3 Tables 1, 2, 6; NASA-STD-5020B Figure 1 (these exceed UG v11 Table 5-1: 1.10 / 1.25) |
| Venting | Ventable volume / vent area < 50.8 m: ≥ 22.3 mm² | CDS Rev 14.1 §2.1.9; UG v11 §4.1.8 (≤ 400 m) |

---

## 2. Structural Configuration

### 2.1 Primary Structure (bought)

| Candidate | Mass | Notes |
|-----------|------|-------|
| ISIS 1U structure | 87 g primary, 108 g with secondary | Recessed ±Z faces accept the ISIS AntS; page removed from isispace.nl, stock to confirm |
| Spacemanic 1U structure | 115 g | Confirm the AntS fit on −Z |
| Pumpkin CubeSat Kit 1U | Not stated | "AntS-compatible plates" listed; walls "Unavailable" in the store |

MEL allowance: 115 g CBE, 120 g MEV (MASS_BUDGET row 1). The RFQ (BUILD_VS_BUY §2.2) must ask for, and the order must
not be placed without:

1. Rails: 7075-T6, 6061-T6 or 6082 with Type III hard anodize, Ra < 1.6 µm, edges rounded ≥ 1 mm, ±Z rail ends
   ≥ 6.5 × 6.5 mm, ≥ 75 % rail contact (CDS §2.2.5–§2.2.13; EXOpod Nova §2.3 requires Type III and Exolaunch approval of
   any other finish).
2. A −Z face recessed so that the 7.0 mm AntS stays below the rail ends (CDS Appendix B standoff, note §2.2.1.2), and a
   +Z face that accepts the 50 × 50 mm GNSS patch below the rail ends.
3. Three deployment-switch positions and two separation-spring plungers in the rail standoffs (§5.2, §5.3).
4. Inserts for the battery isolators (M2.5, steel or A286 helicoil: Ti screws are not run into aluminium threads), the
   wing hinges (M2) and the heat spreader (M2.5).
5. The vendor's qualification report (levels ≥ MPE + 3 dB), mass and CG data (VI-STR-11).

### 2.2 Internal Configuration (recommended stack order "B")

Stack positions from the CAD (SQSO-MECH-CAD-001 §4.1; z of the substrates or the item, mm):

| z (mm) | Item | Mounting |
|--------|------|----------|
| −55.5 … −48.5 (outside) | ISIS AntS, four elements: 2 m along ±X, 70 cm along ±Y | Vendor screw pattern on the −Z end frame (recessed 1.5 mm), DC-isolated (≥ 1 MΩ) |
| −44.9 … −20.9 | Battery pack (4S1P, cells along X), Al clamp, G-10CR cell locators and per-cell disc springs (no adhesive, no potting) | 4 G-10CR isolators at (±38.5, ±31) to the −Z end frame |
| −38.2 … −27.7 / −44.0 … +3.0 | Y rod at (+42, 0) beside the battery / Z rod at (−41.5, 0) through the deck and EPS notches (no contact) | Clips to the +X / −X shear panel |
| −16.5 … −15.5 | Torquer deck (1.0 mm Al skeleton) with the X rod at (0, +36) and the GNSS receiver at (+5, −20) | Columns |
| −3.4 … −1.4 | EPS board, **2.0 mm**; harness connectors on its bottom side | PCB stack, centre supports at (0, ±21) |
| +5.0 … +6.6 | OBC; M-G370PDT0 gyro at (−20, +30.5) standing through the carrier window | PCB stack, centre supports |
| +17.6 … +19.6 | Jetson carrier, 2.0 mm, 27 × 27 mm gyro window; module at (0, −6) on two standoffs, SO-DIMM on the −Y edge | PCB stack, centre supports |
| +27.05 … +32.8 | Heat spreader 80 × 60 × 5 mm with the SoC pedestal | 3 × M2.5 per side to the ±X shear panels (≥ 1 W/K) |
| +34.8 … +36.4 | RF board with an 84 × 54 × 7 mm shield | PCB stack, centre support (0, 0) to the spreader |
| +41.05 … +48.4 | Magnetometer board A2 at (+24, +27); GNSS patch adapter; inertia ballast (2 × 10 g W-Ni-Cu) at (−17, +15.5) and (−17, +0.5) | +Z end frame |
| +50.0 … +55.0 (outside) | GNSS patch, 50 × 50 mm; +Z sun sensor at (−36, +36) | +Z end frame |
| ±X faces | Body panels, wings W1/W2 stowed over them (81.6 × 92 mm, z −43 … +49) | Hinge axis (±54.1, +38.75) |
| ±Y faces | Body panels; RBF and the access-port board A3 (Micro-D 21) on −Y | Panel screws |

Rules that the analyses set:

- **Heavy items at the ±Z ends** (battery at −Z, RF at +Z), the **AntS 2 m elements along ±X** and the **Z-axis
  ballast** (C-13): with the wings and antennas deployed +Y is the major axis of inertia by 4.7–5.6 % (CAD, MEV–CBE;
  MASS_BUDGET §6). The mid-stack battery of the earlier layout gives 0.5–1 %, which degrades the sun-pointing (STR-01).
- **Board supports (STR-05):** in addition to the four corner columns, the EPS, OBC and Jetson carrier boards have
  two centre supports at (0, ±21) mm (the −21 mm one from the EPS to the carrier, under the module), and the RF board
  one at (0, 0) to the heat spreader. The board centre itself is not available: the Jetson
  module covers it on the carrier and a column must pass all three stack boards. The carrier is 2.0 mm (its gyro window)
  and the EPS 2.0 mm (MEV fatigue, P3S-02). FE Steinberg margins +0.42 to +0.71 at CBE and +0.14 to +0.50 at MEV with
  these supports; one support at (0, +21) mm leaves the carrier at −0.28 (STRUCTURAL_ANALYSIS Rev B §8A).
- **The stack is tied to both end frames** (289 Hz; 45 Hz if cantilevered from one end).
- **Products of inertia:** the Jetson module at y = −6 mm, the RBF and A3 on −Y and the ballast pair's +8 mm offset in y
  hold the deployed |Iyz| ≤ 4 × 10⁻⁶ kg·m² (major axis 0.7–1.2° from +Y); placement scatter of 2 mm already moves the
  95th percentile to 6–8°, so the positions are drawing-controlled and the flight unit is measured and trimmed
  (SQSO-MECH-CAD-001 §5.4, VI-MCAD-03).
- Magnetometer away from the coils, the Jetson and the steel cell cans (BUILD_VS_BUY §2.7).
- Harness routing: `hardware/harness/HARNESS_DESIGN.md` (SQSO-HAR-001 2.0) zones, carried in the CAD with their mass (SQSO-MECH-CAD-001 §6.11).

---

## 3. Material Selection

### 3.1 Primary Materials

| Material | Application | Allowables used (verify against MMPDS at drawing release) |
|----------|-------------|-------------------------------------------------------------|
| Al 7075-T6 / 6061-T6 / 6082 | Frame and rails (vendor) | 6061-T6 plate A-basis Fty 241, Ftu 290 MPa; 7075-T6 sheet Fty 462, Ftu 524 MPa |
| Al 6061-T6 | Brackets, battery clamp, heat spreader, stack spacers | As above |
| A286 | Fasteners, stack studs and the wing hinge pins (MoS₂ film) | Ftu 1,103 MPa, Fty 758 MPa; non-magnetic |
| W-Ni-Cu (ASTM B777 Class 3) | Inertia ballast and Iyz trim masses (non-magnetic binder) | Density ≈ 18.0 g/cm³ |
| Ti-6Al-4V (annealed) | Battery isolator screws (low conductance), with MoS₂ against galling | Fty 827, Ftu 896 MPa |
| G-10CR (MIL-I-24768/27 GEE-F) | Battery isolators (preferred for creep), cell-locator plates and spring seats | Flexural ≈ 380 MPa, compressive ≈ 300 MPa; TML 0.35 %, CVCM 0.02 % |
| PEEK 450G | Isolator alternative, only below 0.15 N·m preload | Flexural 165, compressive 118 MPa; long-term bearing ≤ 20 MPa; TML 0.26 %, CVCM 0.00 % |
| 17-7PH CH900 wire | Wing torsion springs | Supplier allowable to confirm (≈ 1,500 MPa bending used) |
| Vectran or Dyneema line | Wing hold-down | 200 N break; creep test before selection (VI-STR-07) |
| FR-4 / polyimide | PCBs, panels, hinge flex jumper | E ≈ 18.6 GPa, flexural ≈ 340 MPa |

### 3.2 Material Properties

| Property | Al 7075-T6 | Al 6061-T6 |
|----------|------------|------------|
| Density | 2810 kg/m³ | 2700 kg/m³ |
| Yield strength (A-basis) | 462 MPa | 241 MPa |
| Ultimate strength (A-basis) | 524 MPa | 290 MPa |
| Young's modulus | 71.7 GPa | 68.9 GPa |
| CTE | 23.6 µm/m·K | 23.6 µm/m·K |
| Thermal conductivity | 130 W/m·K | 167 W/m·K |

Version 1.2 gave typical values (503/572 and 276/310 MPa); design uses A-basis values.

---

## 4. Mass Properties

From `docs/environment/MASS_BUDGET.md` v3.1 and the CAD (SQSO-MECH-CAD-001 §5; analysis, nothing weighed):

| Quantity | CR0002 rods (CAD) | Air-core coils (MEL only) |
|----------|-------------------|---------------------------|
| CBE / MEV | 1,519.4 / 1,754.2 g | 1,600.8 / 1,852.9 g |
| Margin to 2.00 kg | +14.0 % | +7.9 % |
| CG, stowed, CBE | (+0.1, −0.4, −1.4) mm | — |
| Inertia, stowed, CBE (Ixx, Iyy, Izz) | 2.753, 3.062, 2.741 × 10⁻³ kg·m² | — |
| Inertia, deployed, CBE / MEV | 2.853 / 3.276, 4.200 / 4.758, 3.978 / 4.545 × 10⁻³ kg·m² | — |
| Iyy / max(Ixx, Izz), deployed, CBE / MEV | 1.056 / 1.047 (5th percentile at 2 mm placement scatter 1.045 / 1.037) | — |
| Major axis from +Y, deployed, CBE / MEV | 1.2° / 0.7° | — |

Full tensors with products, the one-wing and no-wing cases and the ADCS presets: MASS_BUDGET v3.1 §6.1, §6.5. The Version
1.3 table (reference layout B, MASS_BUDGET v3.0) is superseded.

---

## 5. Mechanical Interfaces

### 5.1 Rail Specifications (CDS Rev 14.1)

| Parameter | Requirement | Design |
|-----------|-------------|--------|
| Rail outer dimensions | 100.0 ± 0.1 mm (X, Y), 113.5 ± 0.1 mm (Z) (EXOpod Nova: Z ± 0.5 mm) | Bought frame; measured per the CIFP or the Exolaunch rail guide (VI-LV-01) |
| Rail width to the first protrusion | ≥ 8.5 mm (EXOpod: ≥ 6.4 mm) | 9.0 mm (wing free edge, body panels), 9.25 mm (hinge springs), 9.4 mm (hinge edge); body panels 82 mm wide (SQSO-MECH-CAD-001 §6.1) |
| Surface finish | Ra < 1.6 µm | Vendor certificate |
| Edge radius | ≥ 1 mm (CDS §2.2.7, a "should") | Vendor drawing |
| Rail ends (±Z) | ≥ 6.5 × 6.5 mm contact | Vendor drawing |
| Rail contact | ≥ 75 % of the rail length | Vendor drawing |
| Material and finish | Aluminium, Type III hard anodize | Vendor certificate |

### 5.2 Deployment Switch Interface

| Parameter | Value |
|-----------|-------|
| Switch quantity | **3** (one per physical inhibit, BASELINE §4, SYS-225); Version 1.2 had 2 |
| Location | DS1 (+X+Y) and DS2 (−X−Y, two-pole DPDT for the third solar-array inhibit, five leads, harness item M-10) in −Z rail standoffs (pressed by the pusher plate or slot adapter), DS3 (+X−Y) in a +Z standoff (pressed by the neighbouring CubeSat or the door; check against the EXOpod door set screws and the 3 mm front-protrusion limit, EXOpod §4.2.3) |
| Actuated position | At or below the standoff surface (CDS §2.3.2.3) |
| Actuation force | 0.5–2.0 N each (sum with the separation springs below the dispenser's push-off design value) |
| Travel | 1–2 mm |
| Contact | Open: TBR-CDS-005 (`docs/compliance/CDS_COMPLIANCE_MATRIX.md` §9). The flight software reads the sense line high (pulled up) while stowed (`software/flight/include/separation_switch.h`) |

### 5.3 Separation Springs

A 1U shares a dispenser slot (EXOpod: always, with other CubeSats or a slot adapter), so CDS §2.2.14 applies.

| Parameter | Value |
|-----------|-------|
| Quantity and location | 2 spring plungers, in the two −Z standoffs without a switch (combined switch-plunger units acceptable) |
| Force | < 6.7 N maximum each (CDS §2.2.14.1) |
| Stroke | > 2.5 mm |
| Stowed position | Not above the standoff level (CDS §2.2.14.2) |

Separation velocity (1.10–1.48 m/s for 1.5–2.0 kg in an EXOpod 1U slot with its adapter) and tip-off (< 10°/s per axis,
EXOpod Nova §2.5) are dispenser properties; Version 1.2 listed them here as if they were spring requirements.

---

## 6. Deployment Mechanisms

Analysis and requirements: `docs/environment/MECHANISMS_ANALYSIS.md`.

### 6.1 Antenna Deployment

| Parameter | Value |
|-----------|-------|
| Type | ISIS AntS 1U–3U (bought): four tape-spring elements, 2 m along ±X and 70 cm along ±Y; vendor release (redundant thermal knives, I²C), commanded by `sqso_fsw::deployment` after the wings |
| Location | Outside the −Z face, 98 × 98 × 7 mm, below the rail ends (recessed face) |
| Deployment direction | In the plane of the −Z face; **2 m (VHF) elements along ±X only** (inertia, MASS_BUDGET §6.3–§6.4). Options B and C of `hardware/rf/ANTENNA_DESIGN.md` v2.0 §3 put a 2 m element along +Y and make +Y the intermediate axis (52–60 % of sunlit time within 15° in the Monte Carlo): not compatible with the spin unless the chair accepts a compensating design (STR-01) |
| Order | **Wings first, then antennas** (the flight order); VI-STR-10: every element ≥ 10 mm from both deployed wings, and ≥ 5 mm from a wing still stowed (CAD 6.5 mm) (MECHANISMS_ANALYSIS E §7.2) |

Version 1.2 described a custom burn-wire antenna (80 mm diameter × 10 mm, deploying along −Z); withdrawn.

### 6.2 Solar Wing Deployment

Version B hinge (MECHANISMS_ANALYSIS E §3A–§5A; `hardware/mechanical/hinge.py`; the Version A springs did not fit the
envelope):

| Parameter | Value |
|-----------|-------|
| Type | Spring-loaded hinge, no latch; spring preload holds the wing on a damped soft stop |
| Panels | 2 (W1, W2), 81.6 × 92 mm, stowed over the ±X faces, hinged on the ±X/+Y edges |
| Hinge axis | (±54.1, +38.75) mm, parallel to Z; everything on the axis within a 2.0 mm radius (6.10 mm from the rail plane, 6.38 mm worst case) |
| Pins | A286 (AMS 5732) Ø1.5 mm, sputtered MoS₂ on the pin and a burnished MoS₂ film in the bore, Ti-6Al-4V bushings; ground humidity and cycle controls (MECHANISMS_ANALYSIS E §5A) |
| Springs | Two per wing, 17-7PH CH900, OD 4.0 mm: primary 0.470 mm wire, 15 coils, on the axis below the wing (z −47.6 … −40.1); secondary 0.343 mm wire, 9.5 coils, in the +Z end zone; 55° preload |
| Torque margin (NASA-STD-5017B, analysis factors) | +0.08 at the cold end of travel at −55 °C (one spring out +0.07; qualification factors +0.29) |
| Deployment angle | 90° (into a plane parallel to the +Y face) |
| Stop | Damped silicone bumper (2 N·m/rad, ζ ≥ 0.3): 0.157 N·m peak, rebound 13° |
| Retention | Vectran or Dyneema line over the −Z free corner on a cup/cone post, 30 N preload, cut by two burn resistors on their own drivers (8 s each, ratio 1.52 at −55 °C); PTFE-faced snubber at the +Z free corner |
| Deployed-state sensor | Non-contact (reed or Hall) at the stop |
| Hinge harness | Two-layer polyimide clock-spring flex (R 1.0–1.6 mm) in the −Z knuckle, ≤ 0.55 mN·m at −55 °C, ending on tab TB-W1/TB-W2 inside the shear panel |

---

## 7. Structural Analysis

Summary of `docs/environment/STRUCTURAL_ANALYSIS.md`.

### 7.1 Load Cases and Factors of Safety

| Load case | Description | Factor of safety |
|-----------|-------------|------------------|
| Launch | Quasi-static and random vibration (limit load = max(quasi-static, 3σ Miles)) | 1.25 yield, 1.40 ultimate (metallic); 1.50 / 2.00 composite; 2.00 softgoods |
| Joints | Separation; slip (friction only at limit and yield) | FSsep 1.20 (1.25 battery mount) × 1.15 fitting factor |
| Ground operations | Handling and transport | 1.10 yield, 1.40 ultimate |
| Thermal | Thermal stress over the predicted range | Included in the joint preload analysis (isolators, ΔT 30 K); the thermal analysis supplies the final range |

### 7.2 Quasi-Static Loads

| Axis | Load (g) | Notes |
|------|----------|-------|
| Axial | 10 | UG v11 Table 4-1 (dispenser assembly ≥ 20 kg) |
| Lateral | 17 (RSS of the two lateral axes) | UG v11 Table 4-1 |

These are integrated-dispenser load factors (UG v11 §4.1.2.1); §4.1.2.2 says such net-CG loads are not for internal
components, so component loads are the larger of these and the 3σ random response at the component's mode. A
quasi-static load test is not required for a fully containerized CubeSat (UG v11 Table 6-2). The integrator's CubeSat
loads and the EXOpod Nova transfer function are requested (STR-22).

### 7.3 Random Vibration

UG v11 Table 4-6 MPE, all axes: 20 Hz 0.01, 50–700 Hz 0.015, 800–925 Hz 0.03, 2000 Hz 0.00644 g²/Hz; 5.57 Grms.
Protoflight test: the envelope of MPE + 3 dB (7.86 Grms) and the GSFC-STD-7000B workmanship spectrum, 8.18 Grms,
1 minute per axis (BASELINE §8; `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §3.1;
`docs/test_procedures/TP-ENV-002_Vibration.md`). Designed to MPE + 3 dB; the FE margins of §7.5 are at the 8.18 Grms
test level.

**Correction of the earlier profile (Version 1.1).** Version 1.0 gave 20 Hz 0.01, 50 Hz 0.04, 800 Hz 0.04, 2000 Hz
0.01 g²/Hz and called it 6.8 grms; it integrates to 7.36 Grms. Withdrawn.

### 7.4 Natural Frequencies

From the shell finite-element model (STRUCTURAL_ANALYSIS Rev B §5A; SQSO-MECH-CAD-001 §8; `hardware/mechanical/fem.py`),
design case (two centre supports, 2.0 mm carrier and EPS, ballast, harness zones, cell retention), with the hand values
of STRUCTURAL_ANALYSIS §5 beside them:

| Mode | Frequency (FE, CBE; MEV) | Hand analysis | Description |
|------|--------------------------|---------------|-------------|
| Stowed wing panel | **170 Hz**; 162 Hz | 142 Hz | Lowest mode of the stowed spacecraft (two hinges, the hold-down post, a soft snubber) |
| Torquer deck | 247 Hz; 232 Hz | — | 1.0 mm skeleton with the GNSS receiver, the X rod and 21 g of harness dress |
| PCB stack, lateral, tied at both ends | 322 Hz (X), 391 Hz (Y) | 289 Hz | Beam model with a joint knock-down for the hand value |
| Boards in the stack: RF / carrier / EPS / OBC | 385 / 480 / 560 / 575 Hz | 221–362 Hz (one support each) | Two supports at (0, ±21) mm; RF one at (0, 0) |
| +Z end frame with the 20 g ballast | 387 Hz | — | — |
| Battery on G-10CR isolators, lateral / axial | 1,273 / 975–1,087 Hz | 1,280 / 1,890 Hz | The FE includes the bending of the −Z end-frame ring |

No mode is below 150 Hz. Requirement > 40 Hz: met by a factor of 4.3 (4.1 at MEV). Verified by the low-level sine sweep
before and after random vibration (UG v11 §6.7.1, Table 6-3; VI-LV-03) and the EM board survey (VI-STR-02).

### 7.5 Margin Summary

MS = allowable / (FS × limit stress) − 1; FE at the 8.18 Grms protoflight level, CBE (MEV where it governs):

| Location | Stress (MPa) | Allowable (MPa) | MS |
|----------|--------------|-----------------|-----|
| Rail, axial (own 2 kg + 4 kg neighbour, 10 g) | 6.3 | 241 | +29 |
| −Z end frame beside the battery isolators (FE, the lowest margin; doubler per STRUCTURAL_ANALYSIS §12) | 98.3 (MEV 104.1) | 241 | +0.96 yield / +1.11 ultimate (MEV +0.85 / +0.99) |
| End frames at the rail corners (FE) | 87.3 | 241 | +1.21 |
| Centre supports (FE) | 74.4 | 241 | +1.59 |
| Stack columns, Al spacer (FE) | 57.3 | 241 | +2.37 |
| Battery isolator, G-10CR, bending (FE, FS 2.0) | 37.1 | 380 | +4.1 (ultimate) |
| Battery isolator Ti screw (FE) | 82.8 | 827 | +7.0 |
| Battery cell retention, axial separation (3σ 142 g; SQSO-MECH-CAD-001 §4.6) | 69.7 N per cell | 119 N minimum preload | +0.19 (FS 1.25 × 1.15) |
| Wing hold-down line (softgood) | 34.9 N | 200 N | +1.87 (+1.71) |
| Hold-down post separation (FE, 3.9 N 3σ) | — | 21 N after 30 % preload loss | +2.9 |
| PCBs, Steinberg fatigue, FE, two centre supports, 2.0 mm carrier and EPS | — | — | +0.42 to +0.71 (MEV +0.14 to +0.50) |
| Battery isolator joint separation, with disc spring | — | — | +2.3 to +7.9 |

---

## 8. Fastener Specifications

### 8.1 Fastener Types

| Application | Fastener | Torque | Locking |
|-------------|----------|--------|---------|
| Structure assembly | M2.5 × 6 A286 | 0.35 N·m | Staking compound or prevailing-torque insert |
| PCB stack columns | M3 A286 studs through Al spacers | 0.60 N·m | Staked |
| PCB mounting, centre supports | M2 × 4 A286 | 0.15 N·m | Staked |
| Wing hinges to frame | M2 A286 into steel inserts (Ti-6Al-4V within 20 mm of the magnetometer board A2) | 0.15 N·m | Staked; not reworked after vibration |
| Battery isolators | M2.5 Ti-6Al-4V, MoS₂, through G-10CR shouldered bushings, disc spring and capture washer under the head, into steel inserts | 0.20 N·m ± 10 % | Staked |
| Battery cell retention | Per-cell disc-spring stacks under the −X end cap, set to 140 N per cell by shims (no torque-set preload); locator plates by M2 Ti-6Al-4V screws | 0.10 N·m (M2) | Staked |
| Z rod clips, inertia ballast | M2 Ti-6Al-4V (non-magnetic zone) | 0.15 N·m | Staked |
| Heat spreader to frame | M2.5 A286 | 0.35 N·m | Staked |
| Access panels | M2 × 6 A286 | 0.15 N·m | Staked (exempt from the UG rework rule, but CDS §2.1.1 still applies) |
| Inserts | Helicoil M2.5 × 1D / M2 × 1D, stainless | As specified | — |

All torques with a calibrated tool and recorded (UG v11 §5.2.1). After environmental test: M2.5–M4 fasteners that are
reworked need two locking features that do not depend on preload; fasteners smaller than M2.5 that are structural are
not reworked (UG v11 Table 5-3).

### 8.2 Thread Locking

| Application | Method |
|-------------|--------|
| Primary structure | Staking compound or prevailing-torque insert |
| PCB standoffs | Staking compound |
| Access panels | Staking compound (removable) |

Version 1.2 named Loctite 242 and 222 and no locking for access panels. Any thread-locker or staking compound must be
shown to meet TML ≤ 1.0 % and CVCM ≤ 0.1 % (CDS §2.1.7) in the NASA outgassing database before it is used
(VI-LV-09, VI-STR-12).

---

## 9. Venting

### 9.1 Venting Requirements

| Parameter | Requirement |
|-----------|-------------|
| Ventable volume / vent area | < 50.8 m (CDS Rev 14.1 §2.1.9); ≤ 400 m (UG v11 §4.1.8) |
| Vent area | **≥ 22.3 mm²** for the gross 1U volume (Version 1.2: "> 4 mm²", which gives 235 m and fails the CDS) |
| Sub-volumes | Each vented on its own: RF shield can (Ø 1.5 mm per 80 × 80 × 10 mm can), battery clamp cavity, pockets behind the heat spreader |
| Depressurization | 2.8 kPa/s (4.5 kPa/s for ≤ 5 s): differential pressure ≤ 50 Pa at the required area (STRUCTURAL_ANALYSIS §11) |

### 9.2 Vent Implementation

- Machined vent holes in the end frames and brackets (count the effective area on the drawing)
- Gaps around PCB edges between the stack levels
- No hermetically sealed volumes other than the cells

---

## 10. CAD Model

### 10.1 CAD Deliverables

| Format | Purpose |
|--------|---------|
| Native CAD | Design master; mass properties (CG and inertia tensor) from weighed components. Today: the parametric CadQuery model `hardware/mechanical/model.py` (SQSO-MECH-CAD-001) |
| STEP AP214 | Exchange format; simplified model for the Exolaunch virtual fit check (EXOpod Nova §2.3.1): `smartqso_stowed.step`, `smartqso_deployed.step` and the outer-mould-line versions |
| 2D drawings | Manufacturing |
| STL | 3D printing (prototypes) |

### 10.2 Drawing Package

- Assembly drawing with BOM
- Detail drawings for all machined parts
- Fastener callouts with torque specs and locking features
- Surface finish callouts
- GD&T per ASME Y14.5
- **Tolerance stack of the stowed wings on the ±X faces** (6.10 mm nominal, 6.38 mm worst case against 6.5 mm,
  SQSO-MECH-CAD-001 §6.2) and of the AntS and the GNSS patch against the ±Z rail ends
- Vent-area summary (§9)
- Deployment sweep envelopes of the wings and the AntS elements (MECHANISMS_ANALYSIS §7.2)

---

## Revision History

| Rev | Date | Description |
|-----|------|-------------|
| 1.0 | 2026-01-02 | Initial CDR baseline |
| 1.1 | 2026-10-16 | Funding audit TB-15: one launch environment from SpaceX UG v11 (interim provider): first mode > 40 Hz, random vibration 5.57 Grms MPE, quasi-static 10 g / 17 g, factors of safety per Table 5-1. The old profile is shown to integrate to 7.36 Grms, not 6.8. Mode frequencies and margins withdrawn (analysis pending FEM). Rail edge radius ≥ 1 mm per CDS Rev 14.1 §2.2.7. Mass line (§1.1) and §4 mass properties not changed here |
| 1.2 | 2026-10-16 | §5.2 deployment-switch contact: "normally closed" conflicted with "normally open" in the launch ICD and the CDS matrix, and nothing decides it; now open as TBR-CDS-005 (`docs/compliance/CDS_COMPLIANCE_MATRIX.md` §9) |
| 1.3 | 2026-10-27 | CDR structures review: bought frame and its RFQ requirements (§2.1); stack order with the battery at −Z and the RF at +Z, centre supports for boards > 50 g, stack tied at both ends, AntS VHF along ±X (§2.2); A-basis allowables and the isolator, spring and line materials (§3); mass properties from MASS_BUDGET v3.0 (§4); three deployment switches and two separation springs, dispenser values removed (§5.2, §5.3); AntS and wing mechanism description (§6); NASA-STD-5001B/5020B factors and the analysis results of SQSO-SA-001 (§7); fastener torques, isolator stack and locking (§8); vent area ≥ 22.3 mm² (§9); tolerance stack and sweep envelopes on the drawings (§10) |
| 1.4 | 2026-10-27 | CDR 2026 consistency pass: board supports two at (0, ±21) mm (RF one at (0, 0)) per STR-05 as adopted (§2.2); natural frequencies from the shell FE model, first mode 170 Hz (§7.4); FE Steinberg margins +0.09 to +0.36 (§7.5), all from STRUCTURAL_ANALYSIS Rev B; protoflight random level 8.18 Grms per BASELINE §8 (§1.1, §7.3) |
| 1.5 | 2026-10-27 | Mechanical design (SQSO-MECH-CAD-001 B; review items STR-13, STR-26) | Brought up to SQSO-MECH-CAD-001 B, STRUCTURAL_ANALYSIS Rev B and MASS_BUDGET v3.1: mass and properties from the CAD (§1.1, §4: CBE 1.52 kg, MEV 1.75 kg, ratio 1.056 / 1.047); stack z-positions from the CAD with the 2.0 mm EPS and carrier, the ballast, the cell retention, the Z rod on the −X wall (§2.2); materials (A286 pins, W-Ni-Cu ballast, G-10CR locators; §3.1); DS2 DPDT (§5.2); four AntS elements and the flight order (§6.1); the Version B hinge (§6.2); FE modes (§7.4) and margins (§7.5); fasteners (§8.1); tolerance stack (§10.2) |
