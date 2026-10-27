# SMART-QSO Structural Analysis

**Document ID**: SQSO-SA-001
**Version**: B
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR issue. Closed-form hand analysis of the build-versus-buy configuration and, from Version B, the
finite-element results of the beam-shell model of the mechanical design (SQSO-MECH-CAD-001,
`hardware/mechanical/MECHANICAL_DESIGN.md` §8; `hardware/mechanical/fem.py`) beside them (§5A, §7A, §8A). No hardware
exists. Scripts: `software/simulation/cdr/structures/structural_analysis.py` (loads, frequencies, margins, joints,
venting) and `mass_properties.py` (mass, CG, inertia). Mechanisms: `docs/environment/MECHANISMS_ANALYSIS.md`.

## 1. Purpose and Result

This document gives the structural analyses that the Critical Design Review needs for SMART-QSO (1U, Class D, protoflight
verification): launch loads, natural frequencies, random-vibration responses, margins of safety, threaded-joint preload
and separation, fracture-control classification, venting, and the CubeSat Design Specification (CDS) envelope.

| Check | Result | Section |
|-------|--------|---------|
| Lowest natural frequency, stowed, with the design changes of §12 | ≈ 140 Hz (stowed wing panel); next the PCBs 221–362 Hz, the stack 289 Hz, the battery ≥ 780 Hz | §5 |
| Same, without them | 99 Hz (RF board on four corner supports); 45 Hz if the stack is fixed at one end only | §5 |
| Requirement | No elastic mode below 40 Hz (SpaceX Rideshare Payload User's Guide v11 §4.1.1) | — |
| Frame, stack spacers, battery mount, hinges, hold-down: margins of safety | All positive at the MPE and at MPE + 3 dB; lowest +1.2 (stack stud thread root at MPE + 3 dB, only if a spacer joint gaps, which §9 shows it does not) and +1.71 (hold-down line, softgood factor 2.0) | §7 |
| PCB fatigue (Steinberg), boards on four corner supports | **Negative**: −0.03 to −0.48 at the MPE for the EPS, OBC, RF and Jetson carrier boards | §8 |
| Same, with a fifth (centre) support | +0.43 to +1.64 at the MPE; +0.01 to +0.87 at MPE + 3 dB | §8 |
| Battery isolators, joint separation (NASA-STD-5020B) | Without a disc spring, creep and thermal contraction of the polymer can take the 0.12 N·m preload to 33 N (MS −0.43); with a disc spring MS ≥ +1.36 | §9 |
| Venting | The documented "> 4 mm²" fails CDS §2.1.9 (V/A = 235 m against 50.8 m); ≥ 22.3 mm² needed. Pressure differential is structurally negligible either way (< 1 kPa) | §11 |
| CDS envelope | Stowed-wing stack 6.1 mm against the 6.5 mm limit (0.4 mm margin before tolerances); the 7 mm AntS needs a recessed −Z face | §13 |
| **FE (Version B)**, first mode of the stowed spacecraft | **170 Hz** (stowed wings; 162 Hz at MEV); deck 247 Hz, RF 385, +Z end frame with the inertia ballast 387, carrier 480, EPS 560 (2.0 mm), OBC 575, battery 1,087–1,273 Hz; no mode below 150 Hz | §5A |
| **FE**, margins at the larger of the quasi-static load and the 3σ response to the protoflight test level (8.18 Grms) | All positive at CBE and MEV: lowest +0.97 (+0.86 MEV; −Z end frame beside the battery isolators), +1.21 (end frames at the rail corners), +1.59 (centre support), +1.80 (deck), +2.37 (stack columns), +2.65 (rails), +4.1 (G-10CR isolator, FS 2.0); with the ballast (C-13), the harness design's distribution, the battery cell retention and the 2.0 mm EPS | §7A |
| **FE**, PCB fatigue (Steinberg, 3σ relative displacement) | +0.42 to +0.72 at CBE, **+0.14 to +0.51 at MEV** with two centre supports at (0, ±21) mm (chair decision), the carrier 2.0 mm with its gyro window and the **EPS 2.0 mm** (P3S-02: with a 1.6 mm EPS the carrier was −0.07 at MEV); one support: carrier −0.28 | §8A |
| **CAD (Version B)**, stowed protrusion | 6.10 mm nominal (hinge springs, OD 4.0 mm), 6.38 mm worst-case tolerance stack (redesigned stack, SQSO-MECH-CAD-001 §6.2); AntS 1.25 mm inside the −Z rail ends with a 1.5 mm recess | §13 |

Verdict for this discipline: **ready for fabrication with actions**. The structure itself has large margins. The open
items are design choices that must be on the drawings (centre PCB supports, the stack tied at both ends, the battery
isolator stack with disc springs, the vent area, the stowed-wing thickness stack) and the verification items of §14.
Version B adds the finite-element model the review asked for (RID SE-12): it confirms the first mode (170 Hz against the
analytic 142 Hz wing; the battery mode 1,287 Hz against 1,280 Hz) and the strength margins, and changes one design
requirement: **every board heavier than 50 g needs two centre supports, at (0, +21) and (0, −21) mm** (the Jetson module
leaves no room at the board centre), and the −Z end frame needs a local doubler under the isolators (§12).

## 2. Applicable Documents

| Document | Paragraphs used |
|----------|-----------------|
| NASA-STD-5001B w/Change 3 (2022-10-24), *Structural Design and Test Factors of Safety for Spaceflight Hardware* | §4.1.1 (protoflight), §4.1.2.3 (test vs no-test), §4.2 FSR 18–22, §4.2.1 Table 1, §4.2.2 FSR 27–28, §4.2.3 Table 2, §4.2.6 Table 6, §4.4 FSR 51, §4.5 |
| NASA-STD-5020B (2021-08-06), *Requirements for Threaded Fastening Systems in Spaceflight Hardware* | §4.2.2 (fitting factor 1.15), §4.2.3 TFSR 4 and Figure 1 (separation factor), §4.3.1 Eq. 1–5 and Table 1, §4.3.2 Table 3, §4.4.3 Eq. 19, §4.4.6 TFSR 13–14 |
| NASA-STD-5019A w/Change 4 (2024-07-29, revalidated 2025-09-05), *Fracture Control Requirements for Spaceflight Hardware* | §1.2 (applicability), §6.1.6, §6.2.1, §6.2.2, §6.2.6 |
| SpaceX *Rideshare Payload User's Guide*, Version 11, August 2026 (downloaded 2026-10-27; current) | §4.1.1, §4.1.2.1 Table 4-1, §4.1.5 Table 4-5, §4.1.6 Table 4-6, §4.1.8, §5.1 Table 5-1, §5.2.1 Table 5-3, §5.2.3, §5.4, §5.7, §6.3, §6.6 Table 6-2, §6.7.1 Table 6-3, §6.7.6, §6.7.7, §6.7.9, §6.7.13 |
| Cal Poly *CubeSat Design Specification* Rev 14.1, CP-CDS-R14.1 (2022) | §2.1.1, §2.1.7, §2.1.9, §2.2.1–§2.2.14, Table 1, Table 2, §2.3.2, §2.3.5.2 |
| Exolaunch *EXOpod Nova User Manual* Rev 1.2 (June 2024) and *EXOpod User Manual* Rev 6.0 (2022-09-06) | Nova Table 1, Table 2, §2.4, §2.5, §3.5, §3.6; EXOpod Tables 3, 4, 7, §4.2.3 |
| D. S. Steinberg, *Vibration Analysis for Electronic Equipment*, 3rd ed. (2000) | PCB displacement criterion, Q ≈ √f |
| A. W. Leissa, *Vibration of Plates*, NASA SP-160 (1969) | Point-supported plate, used to check the Rayleigh–Ritz solver (λ² = 7.12) |

Material allowables are MMPDS-style A-basis values for 6061-T6 plate (Fty 241, Ftu 290 MPa), 7075-T6 sheet (462, 524 MPa)
and annealed Ti-6Al-4V (827, 896 MPa); A286 fasteners Ftu 1,103 MPa; G-10CR and PEEK 450G typical (manufacturer) values.
They were not re-read from MMPDS for this issue; the drawings must call out the material specification, and the values
are re-checked at drawing release (STR-20).

## 3. Configuration and Reference Layout

Axes: CDS body frame, origin at the geometric centre; +Z along the rails, −Z inserted first (CDS §2.2.2); faces at
±50 mm, rail ends at ±56.75 mm. Wings W1/W2 hinged on the +X/+Y and −X/+Y edges with the axis parallel to Z, stowed over
±X, deployed 90° into a plane parallel to the +Y face (`hardware/eps/SOLAR_ARRAY_DESIGN.md` §3.1). Because the rails must
stay clear for 8.5 mm (CDS §2.2.5), the hinge axis is at (±54.0, +39.5) mm and the deployed wing centre at
(±93.5, +39.5, 0) mm.

| Item | Bought / built | Location (recommended layout B, centre z) |
|------|----------------|--------------------------------------------|
| 1U frame (ISIS, Pumpkin or Spacemanic) | Bought | Rails at the corners; end frames at z = ±50 mm |
| ISIS AntS 1U–3U (98 × 98 × 7 mm, 4 tape elements) | Bought | Outside the −Z face, −53 mm; VHF elements along ±X |
| Battery, 4S1P INR18650-35E, Al clamp, 4 G-10CR isolators | Built | −Z end, −35 mm (bracket to the −Z end frame and rails) |
| Magnetorquers (CR0002 X/Y rods or the Z air-core coil) | Bought / built | −19.5 mm; Z rod at a corner (needs board notches) or X/Y coils on the side walls |
| EPS board | Built | −12 mm |
| OBC (STM32L4R5ZI, BMI088) with the M-G370PDT0 | Built | −3 mm |
| Jetson carrier, module, clamp; heat spreader | Built / bought | +6, +10, +13; spreader +19 mm to the frame |
| RF board and shield | Built | +Z end, +36 mm |
| GNSS receiver, cable bracket | Bought | +45, +47 mm |
| GNSS patch (50 × 50 mm) | Bought | Outside the +Z face, +52.5 mm (inside the +Z standoff) |
| Body panels (4) and wings (2) | Bought (panel vendor) | Faces |

The PCB stack runs on four M3 columns (Al 6061 hex spacers, 5 mm across flats, A286 studs) between the two end frames,
with centre supports on every board heavier than 50 g (§8; Version B, chair decision: two supports at (0, ±21) mm on the
EPS, OBC and carrier boards and one at (0, 0) on the RF board, §8A). Layout A ("ref") puts the battery mid-stack; it is
kept as the comparison case for the inertia (`docs/environment/MASS_BUDGET.md` §4, §6).

## 4. Design Criteria and Loads

### 4.1 Factors of Safety (protoflight, flight unit tested)

The governing factor is the larger of NASA-STD-5001B and the SpaceX User's Guide (UG v11 Table 5-1: 1.10 yield, 1.25
ultimate, 1.40 ultimate for ground operations, 1.10 joint gap and slip).

| Application | Yield | Ultimate | Test factor | Source |
|-------------|-------|----------|-------------|--------|
| Metallic structure, fasteners, inserts | **1.25** | **1.40** | 1.20 × limit (protoflight) | 5001B Table 1 (protoflight); FSR 28 for fasteners |
| Composite and bonded (FR-4 panels, G-10CR isolators), uniform material | — | **1.50** | 1.20 | 5001B Table 2 (protoflight) |
| Composite/bonded at discontinuities (bushing shoulders, bonded inserts) | — | **2.00** (1.50 allowed for non-safety-critical) | 1.20 | 5001B Table 2 |
| Structural softgoods (hold-down lines) | — | **2.00** | 2.0 prototype test; 1.2 proof of each flight line | 5001B Table 6 ("all others") |
| Joint separation | FSsep = 1.20 (test factor; separation not hazardous); **1.25** where separation could cause a critical hazard (battery mount); × fitting factor 1.15 for separation-critical joints | — | — | 5020B Figure 1, §4.2.2 |
| Joint slip | Friction only at limit or yield load; μ ≤ 0.10 (coated or anodized surfaces), ≤ 0.20 (clean bare metal) | — | — | 5020B TFSR 13–14 |
| Ground handling | 1.10 | 1.40 | — | UG Table 5-1 |
| Fatigue and creep | Service-life factor 4 | — | — | 5001B FSR 51 |
| Buckling | Design to ultimate loads | — | — | 5001B §4.5 |

### 4.2 Random Vibration

UG v11 Table 4-6 MPE (P95/50, all axes): 20 Hz 0.01, 50–700 Hz 0.015, 800–925 Hz 0.03, 2000 Hz 0.00644 g²/Hz. Integrated
on log-log segments: **5.57 Grms** (the table's value). MPE + 3 dB: 7.86 Grms.

For a fully containerized CubeSat the protoflight test is at the MPE for 1 minute per axis (UG Table 6-2; SQSO-LVR-001
LVR-044). **This analysis designs to MPE + 3 dB** as well, because 5001B's protoflight test factor (1.20 × limit)
otherwise has no counterpart in the required test (§4.1, STR-21): every margin of §7–§9 is given at both levels.

### 4.3 Design Limit Loads

UG Table 4-1 gives 10 g axial and 17 g lateral (RSS) for an integrated CubeSat dispenser assembly ≥ 20 kg, and §4.1.2.2
says such net-CG loads "shall not be utilized for individual internal components". The CubeSat-level and component limit
load is therefore taken as the larger of the quasi-static factor and the 3σ response by Miles' equation at the item's own
mode:

  g_3σ = 3 · √(π/2 · f_n · Q · ASD(f_n))

with Q = 10 for bolted metal structure (the lower end of UG §4.1.1's Q = 10–50 window; no test data), Q = √f_n for PCBs
(Steinberg), Q = 20 for the stowed wing panels and Q = 10 for the battery on its isolators. EXOpod Nova §3.5 states that
the deployer amplifies near its own first mode and attenuates above it; no transfer function has been obtained, so the
UG MPE is applied at the CubeSat rails (STR-22: ask Exolaunch for the Nova 1U-slot transfer function).

Shock: UG Table 4-5 (1,000 g SRS from 1 to 10 kHz at the payload interface, Q = 10) is advised, not required, for a
containerized CubeSat (Table 6-2); CDS §3.3.1.1 notes that shock tests are typically not required. Nothing in this design
is shock-sensitive except the Jetson SODIMM socket and the cells, both covered by §8 and §10.

## 5. Natural Frequencies

| Element | Model | f₁ |
|---------|-------|----|
| EPS board, 112.5 g (MEV), 90 × 96 × 1.6 mm FR-4 | Rayleigh–Ritz plate, supports 4 mm in from the edges: 4 corners / 4 corners + centre / 6 points | 114 / **256** / 141 Hz |
| OBC, 56.2 g | Same | 162 / **362** / 199 Hz |
| RF board, 150 g (shield not counted) | Same | 99 / **221** / 122 Hz |
| Jetson carrier + module + clamp, 109 g | Same | 116 / **260** / 143 Hz |
| Same boards at 2.4 mm | Same | 175–286 / 392–640 / 216–353 Hz |
| PCB stack, 527 g on 4 Al M3 spacer columns, 98 mm | Euler beam, EI × 1/3 for the threaded joints: clamped at both end frames / cantilevered from one | **289** / 45 Hz |
| Battery, 267 g, on 4 isolators (8 mm), lateral / axial | Guided-cantilever bushing + Ti screw; × 0.6 for joint compliance | G-10CR 1,280 / 1,890 Hz; PEEK 780 / 920 Hz |
| Stowed wing, 50.4 g, 83 × 98 × 1.6 mm | Plate on 4 points (2 hinges, hold-down, 1 bumper) / edge-supported | **142** / 399 Hz |
| Deployed wing | Cantilever plate from a rigid hinge | 83 Hz |
| Deployed wing on its stop | Rigid rotation on the stop: soft bumper 2 N·m/rad / metal stop 25 N·m/rad | 21 / 74 Hz |
| Frame | Vendor data not available (no RFQ answers yet) | Expected > 500 Hz for a 1U frame; VI-STR-11 |
| AntS, stowed | Vendor qualified; no public frequency | VI-STR-13 |

The Ritz solver reproduces Leissa's corner-supported square (λ² = 7.11 against 7.12) and gives 15.8 with a centre
support. The stowed spacecraft's lowest elastic mode, with the changes of §12, is the stowed wing at ≈ 140 Hz, 3.5 times
the 40 Hz requirement; it is verified by the low-level sine sweep before and after random vibration (UG §6.7.1 Table 6-3;
VI-LV-03).

### 5A Finite-Element Results (Version B)

Model (SQSO-MECH-CAD-001 §8.1): MITC4 shells for the end frames, side shear panels, boards, deck, spreader and stowed
wings; beams for the rails, stack columns, centre supports, isolators and clamp ears; the battery as a rigid body;
every MEL mass where the CAD puts it (1,520.0 g with row 23 at 66.6 g, the 20 g inertia ballast of row 36, rows 26 and 32 and the mass distribution of the harness design with its M-11 splices and tabs and the 5 g cell retention of row 37; FE 1,520.0 g; and every row at its MEV, 1,754.9 g, in `design_MEV`); rails held laterally along their length and
axially at their ends; 60,144 degrees of freedom; 450 modes to 3.7 kHz. Element verification against closed-form plate, membrane and beam
cases: within 1.5 %.

| Element | Closed form (§5) | FE | Effective mass |
|---------|------------------|----|----------------|
| Stowed wing | 142 Hz (four points) | **170 Hz** (two hinges, the hold-down post, a soft snubber) | 5.4 % x |
| Torquer deck (added by the mechanical design, 1.0 mm skeleton), with 21 g of harness dress (the Z rod now on the −X wall) | — | 247 Hz | 2.8 % z |
| PCB stack, lateral, tied at both ends | 289 Hz | 322 Hz (X), 391 Hz (Y) | 34 % x; 20 % y |
| Boards with centre supports | 221–362 Hz (one support each) | stack drumming 374 Hz (15 % z), RF 385, carrier 480, EPS 560 (2.0 mm) (2.0 mm, gyro window), OBC 574 Hz (two supports) | — |
| Battery on G-10CR isolators, lateral / axial | 1,280 / 1,890 Hz | 1,273 / 975–1,087 Hz | 19 % x / 5.1 % z |
| Frame panels | > 500 Hz expected | +Z end frame 387 Hz with the 20 g ballast (546 Hz without it; 7.2 % z), shear panels 490–624 Hz (with the wall-gap harness and the Z rod), −Z end frame (with the AntS) 1,526 Hz | — |

The lowest mode is 4.3 times the 40 Hz requirement (4.1 at MEV). The analytic axial battery value (1,890 Hz) is higher than the FE
(975–1,087 Hz) because the FE includes the bending of the 1.6 mm −Z end-frame ring under the isolators.

## 6. Random-Vibration Responses (Miles)

| Element | f₁ (Hz) | Q | ASD at f₁ (g²/Hz) | 3σ response, MPE | 3σ, MPE + 3 dB |
|---------|---------|---|-------------------|------------------|----------------|
| PCB stack, clamped both ends | 289 | 10 | 0.015 | 24.8 g | 35.0 g |
| Battery on G-10CR isolators (lateral) | 1,280 | 10 | 0.0157 | 53.3 g | 75.4 g |
| Battery on PEEK isolators (lateral) | 780 | 10 | 0.026 | 54.0 g | 76.4 g |
| Stowed wing | 142 | 20 | 0.015 | 24.5 g | 34.7 g |
| Heat spreader and bracket (assumed) | 300 | 10 | 0.015 | 25.2 g | 35.6 g |

Each design load is the larger of this and the quasi-static factor (17 g lateral, 10 g axial).

## 7. Margins of Safety

MS = allowable / (FS × limit stress) − 1. Limit loads per §4.3 at the MPE; the MPE + 3 dB value is given where it changes
the conclusion.

| Location | Limit stress | Allowable | FS | MS (MPE) | MS (MPE + 3 dB) |
|----------|--------------|-----------|----|----------|-----------------|
| Rail, axial compression in the dispenser: own 2.0 kg plus a 2U neighbour (4.0 kg) at 10 g, four 8.5 × 8.5 × 1.5 mm angle rails | 6.3 MPa | Fty 241 MPa (6061) | 1.25 | **+29** | — (quasi-static) |
| Rail end bearing on the pusher plate (6.5 × 6.5 mm) | 3.5 MPa | Fty 241 MPa | 1.25 | +54 | — |
| Stack spacer bending at the end frame (24.8 g lateral) | 21.2 MPa | Fty 241 MPa | 1.25 | **+8.1** | +5.4 |
| Stack M3 A286 stud at the thread root, if a spacer joint gaps | 195 MPa | Fty 758 MPa | 1.25 | +2.1 | +1.2 |
| Battery isolator bending, G-10CR | 10.3 MPa | 380 MPa flexural | 2.00 | +17.5 | +12.1 |
| Battery isolator bending, PEEK 450G | 6.7 MPa | 165 MPa flexural | 2.00 | +11.3 | +7.7 |
| Battery Ti-6Al-4V M2.5 screw bending (PEEK mount, the worse) | 84.5 MPa | Fty 827 MPa | 1.25 | +6.8 | +4.5 |
| Isolator shoulder bearing at maximum preload plus tension, G-10CR, 0.30 N·m | 33.3 MPa | 300 MPa | 1.50 | +5.0 | — |
| Isolator shoulder bearing, PEEK, 0.30 N·m | 33.7 MPa | 118 MPa | 1.50 | +1.3 | — |
| Wing hinge pin, Ø1.5 mm 300-series, double shear (stowed, 24.5 g) | 1.0 MPa | 0.6 × Fty 515 MPa | 1.25 | +239 | — |
| Wing hold-down line (200 N break), 30 N preload + 4.9 N dynamic | 34.9 N | 200 N | 2.00 | **+1.87** | +1.71 |
| Hold-down gapping (separation): preload after 30 % creep and relaxation, 21 N, against 1.15 × 1.20 × 4.9 N | — | — | 1.38 | +2.14 | +1.22 |
| Deployed wing root, end-of-travel impact, metal stop (MECHANISMS_ANALYSIS §4) | 20.2 MPa | 340 MPa flexural (FR-4) | 1.50 | +10 | — |

The frame is bought: its strength is the vendor's qualification (VI-STR-11). The rail check above uses a generic angle
section and shows that rail compression cannot govern, even with a heavy neighbour in a shared slot. Nothing in the
structure is close to its strength; the governing structural risks are PCB fatigue (§8), preload loss in the polymer
joint (§9) and the end-of-travel shock on the cells (MECHANISMS_ANALYSIS §4).

### 7A Finite-Element Margins (Version B)

Design limit stress at each element: the larger of the quasi-static envelope (UG Table 4-1: 10 g axial with 17 g lateral
in eight directions) and the 3σ random response to the protoflight test level of `docs/environment/ENVIRONMENTAL_TEST_PLAN.md`
Rev C §3.1 (8.18 Grms, the envelope of MPE + 3 dB and the GEVS workmanship spectrum, which exceeds the MPE + 3 dB design
level of §4.2), Q = max(10, √f). The random value governs everywhere. Factors of §4.1.

| Location | Limit stress (MPa) | Hand analysis (§7), MPE + 3 dB | FE MS yield / ultimate |
|----------|--------------------|-------------------------------|------------------------|
| −Z end frame beside the battery isolators | 97.7 | not analysed | **+0.97 / +1.12** (MEV +0.86 / +1.00) |
| End frames at the rail corners | 87.3 | not analysed | +1.21 / +1.37 |
| Centre supports | 74.4 | not analysed | +1.59 / +1.78 |
| Side shear panels | 67.5 | not analysed | +1.86 / +2.07 |
| Rails | 53.0 (112.6 with the rails held at the ends only) | +29 (axial, quasi-static) | +2.64 / +2.91 (+0.72 / +0.85) |
| Stack columns, Al spacer | 57.3 | +5.4 | +2.37 / +2.62 |
| Stack columns, A286 stud | 69.1 | +1.2 (only if a joint gaps) | +7.8 / +10.4 |
| Stack column (+40, +43), Ti-6Al-4V stud (next to the magnetometer board, SQSO-MECH-CAD-001 §4.4) | 23.5 | — | +27 / +26 |
| Battery isolator, G-10CR (FS 2.0) | 37.1 | +12.1 | — / +4.13 |
| Battery isolator, Ti-6Al-4V screw | 82.8 | +4.5 | +7.0 / +6.7 |
| Battery clamp ears | 25.7 | not analysed | +6.5 / +7.1 |
| PCBs (FR-4, FS 1.5) | 17.6–26.2 | — | — / +7.6 to +11.9 |
| Battery cell retention, axial separation (SQSO-MECH-CAD-001 §4.6) | 69.7 N per cell (3σ 142 g) against 119 N minimum preload | — | MS_sep +0.19 (FS 1.25 × 1.15) |
| Stowed wings (FR-4) | 6.4 | — | — / +34 |
| Wing hold-down post, normal load | 3.9 N (3σ) | 4.9 N | Separation +2.9 (21 N preload after 30 % loss) |

The FE margins are lower than the hand values for the spacers and isolators because the FE responds to the 8.18 Grms
test level with the modal coupling of the whole stack, and higher for the stud (the joints do not gap). The new hot spot is
the 1.6 mm −Z end-frame ring that carries the 261 g battery on four points: positive, but the lowest margin of the
spacecraft, and a local peak beside a rigid spider. Design change 10 of §12 adds a doubler there.

## 8. PCB Fatigue (Steinberg)

Criterion: the 3σ centre displacement Z₃σ = 9.8 · g₃σ / f² (inches) must not exceed
Z_allow = 0.00022 · B / (c · h · r · √L) with B = 3.78 in (96 mm edge), h the board thickness, r = 1 (component at the
centre), L = 1 in and c = 1.75 (BGA; also used for the Jetson SODIMM socket). This is Steinberg's 2·10⁷-cycle criterion,
conservative for the few minutes of protoflight test and flight.

| Board | Supports | 1.6 mm, MPE | 1.6 mm, +3 dB | 2.4 mm, MPE | 2.4 mm, +3 dB |
|-------|----------|-------------|---------------|-------------|---------------|
| EPS | 4 corners | −0.37 | −0.56 | −0.15 | −0.40 |
| EPS | 4 corners + centre | **+0.71** | **+0.21** | +1.32 | +0.64 |
| OBC | 4 corners | −0.03 | −0.32 | +0.31 | −0.07 |
| OBC | 4 corners + centre | **+1.64** | **+0.87** | +2.59 | +1.54 |
| RF (shield not counted) | 4 corners | −0.48 | −0.63 | −0.29 | −0.50 |
| RF | 4 corners + centre | **+0.43** | **+0.01** | +0.94 | +0.37 |
| Jetson carrier + module | 4 corners | −0.36 | −0.55 | −0.13 | −0.39 |
| Jetson carrier + module | 4 corners + centre | **+0.74** | **+0.23** | +1.37 | +0.68 |
| Any of the above | 6 points (corners + mid long edges) | −0.32 to +0.25 | −0.52 to −0.11 | −0.08 to +0.70 | −0.35 to +0.21 |

**Design requirement (STR-05):** every board heavier than 50 g gets a centre support (a fifth spacer column, or a
bonded stiffener giving the same frequency, ≥ 220 Hz); the RF board, at +0.01 at MPE + 3 dB, also uses its shield frame
as a stiffener or goes to 2.0 mm. Thickening alone (2.4 mm on four corners) does not close the EPS, RF and carrier boards.
Board-level frequencies and Q are measured on the engineering model (VI-STR-02).

**Jetson module retention (STR-07):** a SODIMM socket's latches are not a flight retention. The module is clamped to the
carrier by the heat-spreader clamp (row 22 of the MEL) at both short edges, with the clamp load carried to the carrier's
centre support, and the latches staked.

### 8A Finite-Element Steinberg Margins (Version B)

The FE gives the 3σ out-of-plane displacement of every board node relative to the mean of its supports, under the
8.18 Grms Z excitation, with the board coupled to the stack (criterion as above, Z_allow = 0.192 mm for a 1.6 mm board, 0.154 mm for 2.0 mm: the
allowable scales with 1/h):

| Board | One centre support at (0, +21) | **Design: two centre supports, 2.0 mm carrier with the gyro window, 2.0 mm EPS, CBE** | **Design, MEV** | 1.6 mm EPS (Version B before P3S-02), CBE / MEV | 1.6 mm carrier, framed window (withdrawn) | 1.6 mm carrier, window, no frame |
|-------|------|------|------|------|------|------|
| EPS | +0.73 | **+0.51** | **+0.20** | +0.23 / +0.04 | +0.13 | +0.19 |
| OBC | +0.79 | **+0.72** | **+0.51** | +0.37 / +0.17 | +0.42 | +0.49 |
| Jetson carrier and module | **−0.28** | **+0.42** | **+0.14** | +0.08 / **−0.07** | +0.06 | **−0.28** |
| RF (support at (0, 0) to the spreader) | +0.22 | **+0.49** | **+0.18** | +0.36 / +0.26 | +0.34 | +0.34 |

**Mass basis (P3S-02):** CBE of every MEL row in the design columns, every row at its MEV (1,754.9 g, boards +25 %) in
the MEV column. With the 1.6 mm EPS the carrier went negative at MEV (−0.07); **fix adopted: the EPS board is 2.0 mm**
(SQSO-MECH-CAD-001 D-14; bottom face 0.4 mm lower, top unchanged, about 6 g of FR-4 inside row 12's growth), which
raises the coupled stack: all four boards positive at MEV. All columns include the 20 g inertia ballast on the +Z end
frame (C-13; +Z end frame 387 Hz), the harness design's mass distribution (under the EPS the connector halves, 6.8 g, on
the EPS and the wire, 21 g, on the deck straps; with all of it on the EPS the 2.0 mm board is at +0.38, the 1.6 mm board
was at −0.52) and the cell retention of row 37.

The centre of the EPS, OBC and carrier boards is not available for a support: the Jetson module (69.6 × 45 mm, centred at
y = −6 mm to balance the products of inertia) covers it on the carrier, and a column through the stack must pass all
three boards. One support at y = +21 mm leaves the −Y half of the EPS and carrier boards unsupported (−0.30, −0.04 with the 2.0 mm carrier). **The
design requirement of STR-05 becomes: two supports at (0, ±21) mm on the EPS, OBC and carrier boards (the −21 mm one
from the EPS to the carrier, under the module), one at (0, 0) on the RF board from the spreader.** Adopted by the chair
(2026-10-27), with the OBC and RF boards at 1.6 mm, the carrier at 2.0 mm for its gyro window and, since review round 3
(P3S-02, MEV masses), the EPS at 2.0 mm: all margins positive at CBE and MEV. (The first issue of Version B quoted
+0.8 to +0.9 for 2.0 mm boards because it applied the 1.6 mm allowable.) The EM board survey (VI-STR-02) measures the Q.

**Gyro window (OBC item C3; SQSO-MECH-CAD-001 §4.3, D-11).** The M-G370PDT0 in its socket stands 14.0 mm above the OBC
and passes through a 27 × 27 mm window in the carrier at x −33.5 … −6.5, y +17.0 … +44.0 mm (moved 3 mm toward +Y to clear the Jetson module's standoff, SQSO-JET-BRD-001
item M1). In a 1.6 mm carrier the window alone takes the carrier from +0.08 (same mesh, no window) to −0.04. The
design is a **2.0 mm carrier without a frame: +0.08** (and OBC +0.37 through the coupled stack). The bonded
2.5 × 3.0 mm Al frame of the first issue (+0.08 on a 1.6 mm board) is withdrawn: its bond to the FR-4 sees a thermal
mismatch shear of about 13–25 MPa at −40 °C (Al 23.6 against FR-4 about 15 ppm/K, shear lag over the bar, 0.5–0.13 mm bond
lines), of the order of the adhesive's strength, and no low-outgassing epoxy with data at the hot end was found
(SQSO-MECH-CAD-001 §4.3.3). The gyro's footprint on the OBC responds at 37.2 Grms out of plane (Z input; 23.9 with the 1.6 mm EPS) and
21.8 / 24.0 Grms in plane (X / Y inputs) to the 8.18 Grms test level; the carrier window edge moves 0.08 mm (3σ) relative
to the gyro against 1.5 mm of clearance.

## 9. Threaded Joints (NASA-STD-5020B)

Preload P = T / (K · d), K = 0.20 dry (A286 into A286 or steel inserts) or 0.15 lubricated (Ti-6Al-4V with MoS₂, to
prevent galling). Preload variation Γ from Table 3 for non-separation-critical torque control: 35 % unlubricated, 25 %
lubricated; torque tolerance ±5 %; short-term relaxation 5 % (Table 1); for the polymer joint, creep and thermal loss are
calculated, not assumed. Minimum preload per Eq. 4 (separation-critical) or Eq. 5 (others, with √n), MS_sep per Eq. 19.

| Joint | T (N·m) | P_nom | P_min | P_max | P_tL | FF × FSsep | MS_sep |
|-------|---------|-------|-------|-------|------|------------|--------|
| Heat spreader to frame, 4 × M2.5 A286 (thermal joint, separation-critical) | 0.35 | 700 N | 411 N | 992 N | 6.7 N | 1.44 | +41 |
| PCB stack column, M3 A286 stud, dry | 0.60 | 1,000 N | 745 N | 1,418 N | 19.6 N | 1.20 | +31 |
| Wing hinge to frame, 2 × M2 A286 | 0.15 | 375 N | 255 N | 532 N | 3.6 N | 1.20 | +57 |
| Battery isolator, G-10CR, Ti M2.5, **no disc spring** | 0.12 | 320 N | **33 N** | 420 N | 40.7 N | 1.44 | **−0.43** |
| Same, 0.30 N·m, no disc spring | 0.30 | 800 N | 358 N | 1,050 N | 40.7 N | 1.44 | +5.1 |
| **Battery isolator, G-10CR, Ti M2.5 with an M2.5 disc spring** | 0.12 / 0.30 | 320 / 800 N | 194 / 519 N | 420 / 1,050 N | 40.7 N | 1.44 | **+2.3 / +7.9** |
| Battery isolator, PEEK 450G, with a disc spring | 0.12 / 0.30 | 320 / 800 N | 180 / 505 N | 420 / 1,050 N | 53.1 N | 1.44 | +1.4 / +5.6 |

Polymer-joint losses (clamped polymer length 8 mm, ΔT = 30 K from assembly at +20 °C to a −10 °C isolator, two-year
creep strain 0.05 % G-10CR / 0.2 % PEEK at ≈ 10 MPa): without a disc spring the series stiffness is 1.4·10⁷ N/m
(G-10CR) and the loss is 126 N thermal + 58 N creep; a disc spring of 2·10⁶ N/m cuts the loss to about 20 N. The
battery-mount tension P_tL includes the rocking moment of the pack (CG 12 mm above the mounting plane, isolators
±35 mm apart).

**Design (STR-08):** G-10CR (MIL-I-24768/27 GEE-F) shouldered isolators, 4 × Ti-6Al-4V M2.5 with MoS₂ at
**0.20 N·m ± 10 %**, a disc spring (rated ≥ 600 N flat) under each head, a Ti or A286 washer whose outer diameter exceeds
the bracket hole so that the screw captures the bracket if the polymer fails (the all-metallic fail-safe of UG §5.4), and
the screw heads staked. The disc spring and the washer bear on the bushing shoulder, never on the bracket, so the
bushings also keep the pack isolated from the frame electrically (≥ 1 MΩ at 50 VDC, `hardware/eps/BATTERY_DESIGN.md`
§5.2); the fail-safe capture only makes metal contact after a bushing has failed. PEEK is acceptable only below 0.15 N·m (its long-term bearing stress at 0.30 N·m, 24 MPa, exceeds
the ≈ 20 MPa creep allowance). The joint is separation-critical (a gapped battery bracket rattles and loses stiffness):
FSsep 1.25 × FF 1.15. Preload retention is verified by test (VI-STR-03), because Table 1 requires test-derived relaxation
for joints with non-metallic parts.

**Locking and rework (STR-18):** UG §5.2.1 Table 5-3 requires two locking features that do not depend on preload for
M2.5–M4 fasteners reworked after environmental test, and does not allow rework of fasteners smaller than M2.5 except on
access panels and non-structural components of a fully containerized CubeSat. Every flight fastener therefore gets at
least one locking feature (staking compound or a prevailing-torque insert), the M2 hinge and PCB screws are not reworked
after the protoflight test, and the access panels (exempt) are still staked. No liquid thread-locker is used:
Loctite 242 and 222 have no E595 data and are non-compliant (`docs/compliance/MATERIALS_LIST.md`); the locking features
are prevailing-torque (self-locking) inserts, screws and jackposts, with Scotch-Weld 2216 as staking only (VI-LV-09). Harness retention:
MIL-T-43435 lacing tape at the tie points of SQSO-HAR-001 2.0 §10.3 (no nylon ties), present in the vibration test (UG §5.2.3).

## 10. Battery, Wings and Antennas in the Stowed State

- **Battery cells:** `hardware/eps/BATTERY_DESIGN.md` §7.1 specifies "spring-loaded contacts" for cell retention. Under a
  53 g (3σ) lateral response that is not a structural retention: the cells must be bonded or potted in the clamp with welded
  nickel interconnects (for the chair, STR-08).
- **Stowed wing (Version B: 170 Hz in the FE, §5A; hinge and springs of SQSO-MECH-001 B §3A):** 142 Hz with two hinges,
  the hold-down at the free edge and one bumper; 24.5 g (3σ); the hold-down line
  sees 4.9 N dynamic on 30 N preload (MS +1.87 strength, +2.14 gapping, §7). Two bumpers at the free corners (silicone,
  low outgassing) keep the panel from slapping the body panel.
- **AntS:** vendor qualified (ISIS heritage, > 500 units, BUILD_VS_BUY §2.6). Mounted with the vendor's screw pattern and
  torque; included in the protoflight random test (UG §6.1 requires the integrated flight configuration).

## 11. Venting and Depressurization

Internal volume between the end frames ≈ 941 cm³ (gross 1U volume 1,135 cm³). CDS §2.1.9 requires ventable volume / vent
area < 50.8 m; UG v11 §4.1.8 recommends ≤ 400 m. Required area: **≥ 18.5 mm²** for the internal volume, **≥ 22.3 mm²** for
the gross volume (the value used in SQSO-LVR-001 LVR-048; adopted). `hardware/structure/STRUCTURAL_DESIGN.md` §9 (v1.2)
specified "> 4 mm²", which gives V/A = 235 m and fails the CDS.

Pressure differential, quasi-steady orifice flow (Cd 0.6) at the UG decay rates (2.8 kPa/s, and 4.5 kPa/s for ≤ 5 s):
3–9 Pa at sea level and ≈ 44 Pa at 20 kPa for V/A = 50.8 m; ≈ 0.9 kPa at 20 kPa for V/A = 235 m. Structurally negligible
in both cases; the requirement is a CDS compliance item. Sub-volumes each need their own vent: the RF shield can (a
Ø 1.5 mm hole per 80 × 80 × 10 mm can), the battery clamp cavity and any pocket behind the heat spreader. The cells are
sealed commercial cells with a CID and vent, not vented volumes.

## 12. Design Changes Required by This Analysis

These are carried into `hardware/structure/STRUCTURAL_DESIGN.md` v1.3:

1. Stack order B (battery at the −Z end, RF at the +Z end) and the AntS VHF elements along ±X (inertia; STR-01).
2. Centre support on every board > 50 g; RF shield as a stiffener (STR-05). Version B: two supports, item 11.
3. PCB stack tied to both end frames (STR-06: 289 Hz instead of 45 Hz).
4. Jetson module clamped at both short edges (STR-07).
5. Battery isolator stack of §9 (STR-08).
6. Vent area ≥ 22.3 mm², every sub-volume vented (STR-09).
7. Wing stowed stack ≤ 6.0 mm from the rail plane, hinge knuckle included (STR-10, §13).
8. Three deployment switches and two separation springs allocated to the rail standoffs (STR-12, §13).
9. Factors of safety of §4.1 (STR-21).
10. (Version B, from the FE) A 1 mm Al doubler (or a 2.5 mm boss) under each battery isolator on the −Z end frame (§7A).
11. (Version B) Two centre supports on the EPS, OBC and carrier boards at (0, ±21) mm; RF board supported at (0, 0)
    from the heat spreader (§8A).
12. (Version B, SQSO-MECH-CAD-001 §4.2) −Z end-frame face recessed 1.5 mm (RFQ: ≥ 0.8 mm) for the AntS; side shear-panel
    faces 0.5 mm below the rail plane; wing outline 81.6 × 92 mm at z −43 … +49 mm; hinge axis (±54.1, +38.75) mm;
    hold-down post at the −Z free corner (§13).
13. (Version B, SQSO-MECH-CAD-001 §4.3, D-11) The carrier has a 27 × 27 mm window for the gyro and is 2.0 mm thick
    (no frame, no bond) (§8A); the (+40, +43) stack column has a Ti-6Al-4V stud (magnetometer zone).
14. (Version B, chair decision C-13, SQSO-MECH-CAD-001 D-9) Two 10 g W-Ni-Cu inertia-ballast discs on the inside of the
    +Z end frame at (−17, ±7.5) mm (MEL row 36), each on two M2 Ti-6Al-4V screws: +Z end-frame mode 402 Hz, RF Steinberg
    +0.40, no strength margin changed by more than 0.03 (§5A, §7A, §8A); moved to (−17, +15.5) / (−17, +0.5) to cancel the
    Iyz of the harness distribution.
15. (Version B, SQSO-HAR-001 2.0 §15 M-1 to M-9; SQSO-MECH-CAD-001 §6.11) Harness on the harness design's zones with its
    routed mass; under the EPS the wire dress is tied to the deck straps and the shear panels, only the connector strain
    relief to the EPS (EPS Steinberg +0.23; −0.52 otherwise) (§8A); four grommeted shear-panel slots, A3 window, tab and
    flex slots, sun-sensor holes in the frame and panels.
16. (Version B, review round 3, P3S-02; SQSO-MECH-CAD-001 D-14) EPS board 2.0 mm (Steinberg at MEV, §8A).
17. (Version B, RID STR-08; SQSO-MECH-CAD-001 §4.6, D-15) Battery cells retained mechanically: G-10CR locator plates,
    per-cell disc-spring preload 140 N, no adhesive or potting (MEL row 37, 5 g; MS_sep +0.19 at the 3σ response, §7A).
18. (Version B, P3S-12; SQSO-MECH-CAD-001 D-16) Z rod tied to the −X shear panel only (clip loads 19–21 N at 3σ).

## 13. CDS Envelope, Rails, Switches and Separation Springs

### 13.1 Envelope and Rails

| CDS Rev 14.1 item | Requirement | Design | Status |
|-------------------|-------------|--------|--------|
| §2.2.1.1, Appendix B | 100.0 ± 0.1 mm rails, 113.5 ± 0.1 mm long | Bought frame | Vendor drawing; measured (VI-LV-01) |
| §2.2.3 | No component on ±X/±Y more than 6.5 mm from the rail plane | Stowed wing stack 6.1 mm (§13.2); body panels 2.0 mm; RBF ≤ 6.5 mm | Thin margin (STR-10) |
| §2.2.5 | Rails ≥ 8.5 mm wide to the first protrusion (EXOpod: 6.4 mm) | Wing hinge knuckles at ≥ 8.5 mm from the rail edge (hinge axis y = 39.5 mm); body panels between the rails (83 mm) | Design |
| §2.2.6–§2.2.7 | Ra < 1.6 µm; edges rounded ≥ 1 mm (should) | Frame vendor: hard-anodized rails | Vendor certificate (VI-STR-11) |
| §2.2.8 | ±Z rail ends ≥ 6.5 × 6.5 mm contact | Frame vendor | Vendor drawing |
| §2.2.9 | ≥ 75 % rail contact | Frame vendor | Vendor drawing |
| §2.2.12–§2.2.13 | Aluminium structure; rails and standoffs hard anodized (Type III, EXOpod: any other finish needs Exolaunch approval) | Frame vendor | Vendor certificate |
| Appendix B note §2.2.1.2 | ±Z standoffs (0.5–7.0 mm): nothing on a ±Z face may stand above the rail ends | AntS 7.0 mm on −Z: fits only with a recessed −Z face (ISIS) or an AntS-compatible plate (Pumpkin); GNSS patch on +Z ≤ standoff − 0.5 mm | **Frame RFQ must state it** (STR-11) |

### 13.2 Stowed-Wing Thickness Budget (±X faces)

Conservative: body panel mounted with its substrate starting at the rail plane.

| Layer | Thickness (mm) | Cumulative (mm) |
|-------|----------------|-----------------|
| Body panel substrate (vendor, 1.0–1.6) | 1.6 | 1.6 |
| Body cells, coverglass, adhesive (3G30A) | 0.35 | 1.95 |
| Clearance (bumpers) | 1.0 | 2.95 |
| Wing back cells | 0.35 | 3.30 |
| Wing substrate | 1.6 | 4.90 |
| Wing front cells | 0.35 | 5.25 |
| Hold-down line over the front face | 0.5 | 5.75 |
| Hinge knuckle Ø 4 mm on the wing mid-plane (4.1 mm), outer edge | — | **6.1** |
| Limit (CDS §2.2.3) | | 6.5 |

Margin 0.4 mm before tolerances (±0.2 mm per layer RSS ≈ ±0.5 mm). Options: a 1.0 mm wing substrate (−0.6 mm; stowed
mode falls to ≈ 78 Hz, deployed bending to ≈ 45 Hz, both above 40 Hz), body panels recessed into the frame, or a hinge
knuckle offset into the wing outline. The drawings must carry the tolerance stack, and the assembled unit is measured
per the CIFP (VI-STR-08 with VI-LV-01). EXOpod Nova allows 25 mm (Nova Table 1); the 6.5 mm CDS figure is kept so that
the spacecraft stays compatible with the other dispensers of decision D7 and the NRCSD backup.

**Version B (CAD, STR-10 and STR-11).** The mechanical design (SQSO-MECH-CAD-001 §6.2–§6.3) rebuilt the stack: side
shear-panel faces 0.5 mm below the rail plane, the hold-down as a cup/cone post with the line over it, and the hinge
hardware (axis 54.1 mm) located from the frame rather than through the panel stack. Nominal: wing front cells 4.75 mm,
hold-down line 5.25 mm, **hinge springs (OD 4.0 mm, SQSO-MECH-001 B §3A) 6.10 mm (governs)**; worst-case tolerance
stack **6.38 mm**, RSS 6.25 mm (margins +0.12 / +0.25 mm). Without the 0.5 mm recess the worst case is 6.88 mm. The AntS on a −Z face recessed 1.5 mm
sits 1.25 mm (worst case 0.90 mm) inside the rail ends; on a flush face it would stand 0.25 mm beyond them. STR-10 and
STR-11 are closed by design, conditional on the frame RFQ items.

### 13.3 Deployment Switches, RBF and Separation Springs

- **Switches:** BASELINE §4 requires ≥ 3 independent physical inhibits; `STRUCTURAL_DESIGN.md` §5.2 v1.2 had two
  switches. Three roller-plunger switches: two in −Z standoffs (pressed by the pusher plate or the adapter) and one in a
  +Z standoff (pressed by the neighbouring CubeSat or the door). In the actuated state they sit at or below the standoff
  surface (CDS §2.3.2.3). The switch at +Z must be checked against the EXOpod door: the EXOpod set screws close the gap
  without a clamping force and the permitted front protrusion is 3 mm (EXOpod §4.2.3). Contact form and inhibit wiring:
  EPS and launch disciplines.
- **Separation springs:** a 1U in an EXOpod always shares a slot (EXOpod Table 7 note), so CDS §2.2.14 applies: two
  spring plungers in the remaining −Z standoffs, maximum force < 6.7 N, stroke > 2.5 mm, not above the standoff when
  compressed (§2.2.14.1–§2.2.14.2). Combined switch-plunger units are acceptable.
- **Separation velocity and tip-off** are dispenser properties, not design values: EXOpod 1U with adapter 1.10–1.48 m/s
  (EXOpod Tables 3–4, SQSO-LVR-001), tip-off < 10°/s per axis (Nova §2.5). `STRUCTURAL_DESIGN.md` §5.3 v1.2 listed them as
  if they were spring requirements; corrected.
- **RBF:** on the −Y face (no wing), ≤ 6.5 mm from the rail surface when inserted (CDS §2.3.5.2), in a Nova access window
  (SQSO-LVR-001 LVR-068).

## 14. Fracture Control Classification

NASA-STD-5019A applies to human-rated systems (§1.2); a free-flying Class D CubeSat on an uncrewed Falcon 9 is not
required to apply it, and SpaceX asks for no fracture-control plan for containerized CubeSats. It is applied here as a
tailored classification, so that the integrator and any later ISS (NRCSD) path have the rationale:

| Part | Classification (5019A) | Rationale |
|------|------------------------|-----------|
| Whole spacecraft, until separation | NFC contained (§6.2.2) | Fully containerized in the dispenser (UG §6.6: no hole > 6.35 mm, holes < 10 % of the surface); the dispenser is the enclosure and carries its own qualification |
| Whole spacecraft, after separation | NFC, documented non-hazardous failure mode (§6.2.6) | No crew, no co-located vehicle; failure is loss of mission only |
| Battery cells (INR18650-35E) | NFC battery (§6.1.6 a) | Sealed commercial cells with CID and vent; leak-before-burst behaviour from the cell vendor data; not pressure vessels in UG §5.5 terms |
| Fasteners, switches, hinge pins, small brackets | NFC low-released mass (§6.2.1) | Inside the dispenser; release causes no catastrophic hazard |
| Wing and AntS springs, tape elements | NFC low-released mass / contained | Stored energy small (wing ≈ 24 mJ, MECHANISMS_ANALYSIS §4); retained by the hinge |
| FR-4 panels, bonded cells, G-10CR isolators | NFC (composite and bonded parts are covered by protoflight testing) | Non-structural for any hazard; 5001B Table 2 factors applied |

Debris: CDS §2.1.1 (all parts remain attached) and NASA-STD-8719.14 require that no part is released on orbit. The
burn-wire line must be captured at both ends after the cut (tie points on both sides of the resistor), and the AntS line
remnants are retained by the vendor design (VI-STR-13).

## 15. Verification Items

| ID | Verification | Pass criterion | Verifies | When |
|----|--------------|----------------|----------|------|
| VI-STR-01 | Mass, CG and stowed inertia tensor of the flight unit on a spin-balance mass-properties instrument (not a ±2 % trifilar pendulum): MOI ±0.5 % per axis, products of inertia ±1 × 10⁻⁵ kg·m², CG ±0.1 mm, calibrated before and after with a reference mass of known tensor, the fixture's own tensor measured empty and subtracted (one accuracy with VI-LV-02 and VI-MCAD-03, STR-04); component weighing feeding the CAD inertia | Mass ≤ 2.00 kg; CG ±20 mm; deployed inertia (CAD with the measured stowed tensor and the weighed parts): pass criteria applied with the measurement uncertainty: deployed Iyy/max(Ixx, Izz) − 0.010 ≥ 1.03 (the ±0.5 % of both moments added) and the major-axis tilt with Iyz moved 1 × 10⁻⁵ kg·m² toward the worse sign (about 2.6°) ≤ 5°, i.e. a measured tilt ≤ 2.4° (set with the Iyz trim masses, D-9) | SYS-002, SYS-003, proposed inertia requirement (MASS_BUDGET §2.1) | Engineering model (method), flight unit before environmental test |
| VI-STR-02 | Board-level sine survey (0.25 g, 20–2,000 Hz) of the EM stack with accelerometers on the centre of each board > 50 g, on the deck at the GNSS receiver mounts and at the Z-rod notch, and on the −X shear panel at the Z-rod clips (P3S-12) | f₁ ≥ 200 Hz; measured Q used to repeat §8 with MS ≥ 0 at MPE + 3 dB | §8, STR-05 | EM, before the flight boards are released |
| VI-STR-03 | Battery isolator joint test: 4 instrumented joints (load washers), 20 thermal cycles −20/+50 °C and 30 days at +40 °C, then a sine survey of the pack on the EM bracket | Preload loss ≤ 30 % with the disc spring; pack lateral and axial modes ≥ 500 Hz; Q ≥ 10 (else the mount is an "isolator" under UG §5.4 and §6.7.13 applies) | §9, STR-08; UG §5.4 | EM |
| VI-STR-08 | Stowed protrusion measurement per the CIFP / Exolaunch rail guide | Every point on ±X/±Y ≤ 6.5 mm from the rail plane | CDS §2.2.3, STR-10 | With VI-LV-01 |
| VI-STR-09 | Vent-area inspection against the drawing | ≥ 22.3 mm² total; each sub-volume V/A < 50.8 m | CDS §2.1.9, STR-09 | Flight unit, before close-out |
| VI-STR-11 | Frame vendor evidence: qualification report (levels ≥ MPE + 3 dB), rail anodize and Ra certificates, edge-radius drawing, mass and CG data, −Z face recess for the AntS | All meet §13.1 and §4.2 | CDS §2.2, STR-11, STR-20 | Frame order |
| VI-STR-12 | Fastener audit: torque records (calibrated tool), locking feature on every flight fastener, outgassing data of every staking compound, no rework of < M2.5 structural fasteners after vibration | UG §5.2.1, §5.2.3; CDS §2.1.7 | STR-18 | Flight assembly and after protoflight |
| VI-STR-13 | AntS vendor data and acceptance: qualification levels, deployment test at cold, line-remnant retention, ICD | Levels ≥ MPE + 3 dB; deployment at the minimum predicted temperature; no released parts | STR-15, §14 | AntS order |
| VI-LV-02, -03, -04 | Mass properties, low-level sine sweep before and after, protoflight random with power-inhibit monitoring (SQSO-LVR-001) | As defined there; all modes > 40 Hz, first-mode shift < 10 % | UG §6.7.1, §6.7.6, §6.7.7 | Flight unit |

Mechanism verification items (VI-STR-04 to -07, -10) are in `docs/environment/MECHANISMS_ANALYSIS.md` §8. Version B: the FE
correlation survey and the mass-properties items are VI-MCAD-01 to -04 of `hardware/mechanical/MECHANICAL_DESIGN.md` §12;
VI-STR-02 now has accelerometers at (0, ±21) mm as well as the board centres.

## 16. Limits of This Analysis

- Sections 5–13 are closed-form models: plates, beams and single-degree-of-freedom responses; no coupled model of the
  CubeSat in the deployer (EXOpod Nova §3.6 discourages one without test). Protoflight testing with the sine sweeps is
  the verification (5001B §4.1.2.3: analysis plus test). Version B adds the beam-shell FE of SQSO-MECH-CAD-001 §8
  (its own limits in §8.7 there: joints idealised, frame an allocation, assumed Q, no NASTRAN deck); it is correlated
  by the EM survey (VI-STR-02, VI-MCAD-04) before its margins are reused for the flight unit.
- Q values are assumptions; Steinberg's criterion is conservative for the short exposure.
- Sections 3–13 use the reference layout; Version B's CAD (SQSO-MECH-CAD-001) gives the positions as designed.
- The frame and the AntS are vendor items without quotes or data packages (no RFQ answers yet).
- Allowables were not re-read from MMPDS (§2).

## 17. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | CDR structures review | Initial issue: launch environment and factors, natural frequencies, Miles responses, margins of safety, PCB fatigue, NASA-STD-5020B joints and the battery isolator stack, venting, fracture-control classification, CDS envelope and switch allocation, verification items |
| B | 2026-10-27 | Mechanical design (SQSO-MECH-CAD-001) | Finite-element results beside the analytic ones (RID SE-12): modes and effective masses (§5A), margins at the protoflight test level (§7A), Steinberg margins with the board-to-stack coupling (§8A); design changes 10–12 (isolator doubler, two centre supports at (0, ±21) mm on 1.6 mm boards as decided by the chair, CAD envelope decisions); CAD tolerance stack closing STR-10 with the Version B hinge springs and the −Z recess closing STR-11 (§13.2); skeleton torquer deck; gyro window in the carrier with its stiffening frame (OBC item C3, design change 13, §8A); Steinberg allowable for the 2.0 mm reserve boards corrected to 0.154 mm (+0.40 to +0.60, not +0.8 to +0.9); Jetson carrier item M1 (spreader pedestal, module standoffs, flush fastener at (0, −21); window moved 3 mm toward +Y) with the FE re-run; carrier 2.0 mm without the window frame, Ti stud at (+40, +43), FE re-run with row 23 at 66.6 g and the A2 and harness masses; summary, limits and verification references updated; chair decision C-13: FE re-run with the 20 g Z-axis inertia ballast on the +Z end frame (FE mass 1,514.3 g = CAD; lowest MS +0.72, Steinberg +0.08 to +0.40, design change 14); harness design SQSO-HAR-001 2.0: FE re-run with its mass distribution and MEL rows 26 and 32 (FE mass 1,513.9 g; first mode 169 Hz, the deck with the harness dress; lowest MS +1.00; Steinberg +0.08 to +0.37, the EPS +0.23 with the tie-down rule, design change 15); **review round 3:** one mass-properties accuracy for VI-STR-01 (STR-04); FE at MEV masses and the 2.0 mm EPS (P3S-02; Steinberg +0.14 to +0.50 at MEV), the battery cell retention (STR-08), the Z rod on the −X wall only (P3S-12), probes at the GNSS receiver and Z-rod mounts (MCAD-12), the EPS-tied harness case kept as a run case (P3S-09); design changes 16–18; deck accelerometer in VI-STR-02; harness parts audit (SQSO-HAR-001 2.0 amended, M-11: four D-200-82 splices, tabs TB-RX/RY/RZ/ANT): FE re-run, FE mass 1,520.0 g (MEV 1,754.9 g), lowest MS +0.97 (MEV +0.86), Steinberg +0.42 to +0.72 (MEV +0.14 to +0.51) |
