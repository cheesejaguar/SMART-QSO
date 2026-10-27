# Structural Design Document
## SMART-QSO 1U CubeSat

**Document Number:** SQSO-STR-001
**Version:** 1.3
**Date:** 2026-10-27
**Status:** Draft, CDR issue. Design description of the build-versus-buy structure; the analyses are in
`docs/environment/STRUCTURAL_ANALYSIS.md` (SQSO-SA-001), `docs/environment/MECHANISMS_ANALYSIS.md` (SQSO-MECH-001) and
`docs/environment/MASS_BUDGET.md` (SQSO-MB-001 v3.0). No CAD model, finite-element model or hardware exists

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
| Mass | ≤ 2.00 kg. Estimate: CBE 1.49 kg (CR0002 rods) / 1.57 kg (air-core coils), MEV 1.73 / 1.82 kg, margin +15.9 / +9.6 % | SYS-002; CDS Rev 14.1 Table 1 (typical maximum); `docs/environment/MASS_BUDGET.md` v3.0 |
| CG location | ≤ 2 cm from the geometric centre on each axis. Estimate: within 5 mm | CDS Rev 14.1 §2.2.11, Table 2; MASS_BUDGET §5 |
| Deployed inertia (proposed, STR-01) | Iyy ≥ 1.03 × max(Ixx, Izz) with the wings and antennas deployed (target 1.05); major principal axis within 5° of +Y | MASS_BUDGET §2.1, §6 |
| Rail dimensions | ≥ 8.5 mm to the first protrusion | CDS Rev 14.1 §2.2.5 |
| Rail surface finish | Ra < 1.6 µm, hard anodized | CDS Rev 14.1 §2.2.6, §2.2.13 |
| Protrusions on ±X/±Y | ≤ 6.5 mm from the rail plane | CDS Rev 14.1 §2.2.3 |
| First natural frequency | > 40 Hz (no elastic mode below 40 Hz); design target ≥ 100 Hz for every element | UG v11 §4.1.1; BASELINE §8 |
| Random vibration | Test at ≥ 5.57 Grms MPE (protoflight, 1 min/axis); designed to MPE + 3 dB (7.86 Grms) | UG v11 §4.1.6 Table 4-6, Table 6-2; STRUCTURAL_ANALYSIS §4.2 |
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

| z (mm, centre) | Item | Mounting |
|----------------|------|----------|
| −53 (outside) | ISIS AntS, VHF elements along ±X | Vendor screw pattern on the −Z end frame |
| −35 | Battery pack (4S1P, Al clamp) | 4 G-10CR isolators to the −Z end frame / rails (§8.1) |
| −19.5 | Magnetorquer X/Y rods (or the air-core Z coil) | Bracket; the Z rod at a corner needs board notches, or use the air-core coil for Z |
| −12 | EPS board | PCB stack |
| −3 | OBC (STM32L4R5ZI, BMI088) and M-G370PDT0 | PCB stack |
| +6 / +10 / +13 | Jetson carrier / module / module clamp | PCB stack; module clamped at both short edges, SODIMM latches staked |
| +19 | Jetson heat spreader (Al 6061, 80 × 60 × 5 mm) | 4 × M2.5 to the frame side walls (≥ 1 W/K path, BASELINE §6) |
| +36 | RF board with shield | PCB stack; shield frame used as a stiffener |
| +45 / +47 | GNSS receiver / cable bracket | PCB stack / bracket |
| +52.5 (outside) | GNSS patch, 50 × 50 mm | +Z end frame, below the rail ends |
| ±X faces | Body panels, wings W1/W2 stowed over them | Panel screws; hinges on the ±X/+Y edges at y = 39.5 mm |
| ±Y faces | Body panels; RBF on −Y | Panel screws |

Rules that the analyses set:

- **Heavy items at the ±Z ends** (battery at −Z, RF at +Z) and the **AntS VHF elements along ±X**: with the wings and
  antennas deployed this makes +Y the major axis of inertia by 3–4 % (MASS_BUDGET §6). The mid-stack battery of the
  earlier layout gives 0.5–1 %, which degrades the sun-pointing (STR-01).
- **Board supports:** every board heavier than 50 g has a centre support in addition to the four corner columns
  (STRUCTURAL_ANALYSIS §8; Steinberg margins −0.03 to −0.48 without it, +0.43 to +1.64 with it).
- **The stack is tied to both end frames** (289 Hz; 45 Hz if cantilevered from one end).
- **Products of inertia:** keep the rods, the magnetometer and the RBF off the ±Y/±Z diagonals where possible and keep
  the CG within ±2 mm in Z, so that |Iyz| ≤ 1.5 × 10⁻⁵ kg·m² (major axis within 5° of +Y).
- Magnetometer away from the coils, the Jetson and the steel cell cans (BUILD_VS_BUY §2.7).
- Harness routing: `hardware/harness/HARNESS_DESIGN.md` §6.

---

## 3. Material Selection

### 3.1 Primary Materials

| Material | Application | Allowables used (verify against MMPDS at drawing release) |
|----------|-------------|-------------------------------------------------------------|
| Al 7075-T6 / 6061-T6 / 6082 | Frame and rails (vendor) | 6061-T6 plate A-basis Fty 241, Ftu 290 MPa; 7075-T6 sheet Fty 462, Ftu 524 MPa |
| Al 6061-T6 | Brackets, battery clamp, heat spreader, stack spacers | As above |
| A286 | Fasteners and stack studs | Ftu 1,103 MPa, Fty 758 MPa; non-magnetic |
| Ti-6Al-4V (annealed) | Battery isolator screws (low conductance), with MoS₂ against galling | Fty 827, Ftu 896 MPa |
| G-10CR (MIL-I-24768/27 GEE-F) | Battery isolators (preferred for creep) | Flexural ≈ 380 MPa, compressive ≈ 300 MPa; TML 0.35 %, CVCM 0.02 % |
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

From `docs/environment/MASS_BUDGET.md` v3.0 (analysis; nothing weighed):

| Quantity | CR0002 rods | Air-core coils |
|----------|-------------|----------------|
| CBE / MEV | 1,492.7 / 1,725.9 g | 1,574.1 / 1,824.6 g |
| Margin to 2.00 kg | +15.9 % | +9.6 % |
| CG, stowed (layout B) | (−0.8, −0.7, −4.7) mm | (−1.2, −1.0, −4.5) mm |
| Inertia, stowed, CBE (Ixx, Iyy, Izz) | 2.79, 3.04, 2.76 × 10⁻³ kg·m² | 3.01, 3.27, 3.02 × 10⁻³ kg·m² |
| Inertia, deployed, CBE | 2.90, 4.31, 4.14 × 10⁻³ kg·m² | 3.13, 4.54, 4.41 × 10⁻³ kg·m² |
| Iyy / max(Ixx, Izz), deployed, CBE / MEV | 1.041 / 1.033 | 1.030 / 1.021 |

Version 1.2's table (1,090 g total, CG "< 5 mm", Ixx = Iyy = 0.0020, Izz = 0.0015 kg·m²) had no model behind it and is
withdrawn.

---

## 5. Mechanical Interfaces

### 5.1 Rail Specifications (CDS Rev 14.1)

| Parameter | Requirement | Design |
|-----------|-------------|--------|
| Rail outer dimensions | 100.0 ± 0.1 mm (X, Y), 113.5 ± 0.1 mm (Z) (EXOpod Nova: Z ± 0.5 mm) | Bought frame; measured per the CIFP or the Exolaunch rail guide (VI-LV-01) |
| Rail width to the first protrusion | ≥ 8.5 mm (EXOpod: ≥ 6.4 mm) | Wing hinge knuckles ≥ 8.5 mm from the rail edge; panels 83 mm wide between the rails |
| Surface finish | Ra < 1.6 µm | Vendor certificate |
| Edge radius | ≥ 1 mm (CDS §2.2.7, a "should") | Vendor drawing |
| Rail ends (±Z) | ≥ 6.5 × 6.5 mm contact | Vendor drawing |
| Rail contact | ≥ 75 % of the rail length | Vendor drawing |
| Material and finish | Aluminium, Type III hard anodize | Vendor certificate |

### 5.2 Deployment Switch Interface

| Parameter | Value |
|-----------|-------|
| Switch quantity | **3** (one per physical inhibit, BASELINE §4, SYS-225); Version 1.2 had 2 |
| Location | Two in −Z rail standoffs (pressed by the pusher plate or slot adapter), one in a +Z standoff (pressed by the neighbouring CubeSat or the door; check against the EXOpod door set screws and the 3 mm front-protrusion limit, EXOpod §4.2.3) |
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
| Type | ISIS AntS 1U–3U (bought), monopole/hybrid: two 2 m and two 70 cm tape-spring elements; vendor release (redundant thermal knives, I²C) |
| Location | Outside the −Z face, 98 × 98 × 7 mm, below the rail ends (recessed face) |
| Deployment direction | In the plane of the −Z face; **VHF elements along ±X** (inertia, MASS_BUDGET §6.3) |
| Order | **Wings first, then antennas**, or a verified ≥ 10 mm clearance between the tapes and the wings (MECHANISMS_ANALYSIS §7.2) |

Version 1.2 described a custom burn-wire antenna (80 mm diameter × 10 mm, deploying along −Z); withdrawn.

### 6.2 Solar Wing Deployment

| Parameter | Value |
|-----------|-------|
| Type | Spring-loaded hinge, no latch; spring preload holds the wing on a soft stop |
| Panels | 2 (W1, W2), stowed over the ±X faces, hinged on the ±X/+Y edges |
| Hinges | 2 per wing, Ø1.5 mm 300-series pins, dry-film lubricated; axis at (±54.0, +39.5) mm |
| Springs | 2 per wing, 17-7PH Ø0.55 mm wire, 10.0 mN·m stowed / 5.0 mN·m deployed each (MECHANISMS_ANALYSIS §3) |
| Deployment angle | 90° ± 2° (into a plane parallel to the +Y face) |
| Stop | Soft or damped, peak ≤ 0.3 N·m at end of travel |
| Retention | Vectran or Dyneema line over the free edge, 30 N preload, cut by two burn resistors on separate drivers; line ends captured after the cut |
| Deployed-state sensor | Non-contact (reed or Hall) at the stop |
| Hinge harness | Polyimide flex jumper, torque ≤ 1.5 mN·m at −40 °C |

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
Protoflight test: MPE, 1 minute per axis (UG v11 Table 6-2; `docs/test_procedures/TP-ENV-002_Vibration.md`). Designed
to MPE + 3 dB (7.86 Grms).

**Correction of the earlier profile (Version 1.1).** Version 1.0 gave 20 Hz 0.01, 50 Hz 0.04, 800 Hz 0.04, 2000 Hz
0.01 g²/Hz and called it 6.8 grms; it integrates to 7.36 Grms. Withdrawn.

### 7.4 Natural Frequencies

| Mode | Frequency | Description |
|------|-----------|-------------|
| Stowed wing panel | ≈ 140 Hz | Lowest mode of the stowed spacecraft with the §2.2 rules |
| RF / EPS / Jetson carrier / OBC boards, with a centre support | 221 / 256 / 260 / 362 Hz | Rayleigh–Ritz plate model |
| PCB stack, lateral, tied at both ends | 289 Hz | Beam model with a joint knock-down |
| Battery on G-10CR isolators | ≥ 1,280 Hz lateral | Including a 0.6 joint factor |

Requirement > 40 Hz: met by a factor of 3.5. Verified by the low-level sine sweep before and after random vibration (UG
v11 §6.7.1, Table 6-3; VI-LV-03) and the EM board survey (VI-STR-02).

### 7.5 Margin Summary

MS = allowable / (FS × limit stress) − 1.

| Location | Stress (MPa) | Allowable (MPa) | MS |
|----------|--------------|-----------------|-----|
| Rail, axial (own 2 kg + 4 kg neighbour, 10 g) | 6.3 | 241 | +29 |
| Stack spacer root (24.8 g lateral) | 21.2 | 241 | +8.1 (+5.4 at MPE + 3 dB) |
| Battery isolator, G-10CR, bending | 10.3 | 380 | +17.5 |
| Battery Ti screw, bending (PEEK case) | 84.5 | 827 | +6.8 |
| Wing hold-down line (softgood) | 34.9 N | 200 N | +1.87 (+1.71) |
| PCB boards > 50 g, Steinberg fatigue, with centre support | — | — | +0.43 to +1.64 (+0.01 to +0.87) |
| Battery isolator joint separation, with disc spring | — | — | +2.3 to +7.9 |

---

## 8. Fastener Specifications

### 8.1 Fastener Types

| Application | Fastener | Torque | Locking |
|-------------|----------|--------|---------|
| Structure assembly | M2.5 × 6 A286 | 0.35 N·m | Staking compound or prevailing-torque insert |
| PCB stack columns | M3 A286 studs through Al spacers | 0.60 N·m | Staked |
| PCB mounting, centre supports | M2 × 4 A286 | 0.15 N·m | Staked |
| Wing hinges to frame | M2 A286 into steel inserts | 0.15 N·m | Staked; not reworked after vibration |
| Battery isolators | M2.5 Ti-6Al-4V, MoS₂, through G-10CR shouldered bushings, disc spring and capture washer under the head, into steel inserts | 0.20 N·m ± 10 % | Staked |
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
| Native CAD | Design master; mass properties (CG and inertia tensor) from weighed components |
| STEP AP214 | Exchange format; simplified model for the Exolaunch virtual fit check (EXOpod Nova §2.3.1) |
| 2D drawings | Manufacturing |
| STL | 3D printing (prototypes) |

### 10.2 Drawing Package

- Assembly drawing with BOM
- Detail drawings for all machined parts
- Fastener callouts with torque specs and locking features
- Surface finish callouts
- GD&T per ASME Y14.5
- **Tolerance stack of the stowed wings on the ±X faces** (≤ 6.0 mm nominal from the rail plane, STRUCTURAL_ANALYSIS
  §13.2) and of the AntS and the GNSS patch against the ±Z rail ends
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
