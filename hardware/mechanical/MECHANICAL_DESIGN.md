# SMART-QSO Mechanical Design: CAD Model and Structural Finite-Element Model

**Document ID**: SQSO-MECH-CAD-001
**Version**: B
**Date**: 2026-10-27
**Classification**: Unclassified
**Status**: CDR design completion (RIDs SE-01 mechanical part, SE-12). Parametric CAD of layout B to the level of each
part's envelope, mass and position, with automated envelope and interference checks, drawings and STEP exports, and a
beam-shell finite-element model of the stowed spacecraft. **Models and analysis only: no part has been bought, made or
weighed. Vendor envelopes are published values where a source is cited; everything else is an allocation.**
**Sources**: `hardware/mechanical/model.py` (CAD, CadQuery 2.8 / OpenCascade 7.9), `hardware/mechanical/fem.py` (FE,
numpy/scipy). Outputs are in the same directory: `results.json`, `fem_results.json`, four STEP files and two SVG sheets.
**Inputs**: `docs/environment/MASS_BUDGET.md` (SQSO-MB-001), `docs/environment/STRUCTURAL_ANALYSIS.md` (SQSO-SA-001),
`docs/environment/MECHANISMS_ANALYSIS.md` (SQSO-MECH-001), `hardware/structure/STRUCTURAL_DESIGN.md` (SQSO-STR-001),
`docs/hardware/BUILD_VS_BUY.md`, [BASELINE_PARAMETERS.md](../../docs/BASELINE_PARAMETERS.md) Rev D.

## 1. Result

| Check | Result | Requirement | Section |
|-------|--------|-------------|---------|
| Envelope, rails | 100.0 × 100.0 × 113.5 mm; rails 8.5 × 8.5 mm, continuous (100 % contact), solid 8.5 × 8.5 × 6.75 mm standoffs | CDS Rev 14.1 §2.2.1, App. B, §2.2.8–§2.2.9 | §6.1 |
| Rail width to the first protrusion | 9.0 mm (wing free edge, body panels), 9.25 mm (hinge springs), 9.4 mm (wing hinge edge) | ≥ 8.5 mm (CDS §2.2.5) | §6.1 |
| Stowed protrusion (STR-10) | 6.10 mm nominal (hinge springs, OD 4.0 mm); **6.38 mm worst-case tolerance stack, 6.25 mm RSS** | ≤ 6.5 mm (CDS §2.2.3) | §6.2 |
| AntS below the −Z rail ends (STR-11) | 1.25 mm nominal, 0.90 mm worst case, with the −Z face recessed 1.5 mm (unrecessed: 0.25 mm **beyond** the rail ends) | ≥ 0.1 mm (CDS App. B) | §6.3 |
| GNSS patch below the +Z rail ends | 1.75 mm nominal, 1.40 mm worst case | ≥ 0.1 mm | §6.3 |
| Deployment switches, separation plungers, RBF | DS1, DS2 at −Z and DS3 at +Z on three rails; SP1, SP2 at −Z; all flush when actuated; RBF head 5.10 mm from the rail plane | CDS §2.3.2.3, §2.2.14, §2.3.5.2; BASELINE §4 | §6.4 |
| Interference, stowed and deployed | None (492 / 486 solid pairs with overlapping boxes tested by boolean intersection); the harness design's 58 zones clear of every solid (`harness_calcs.py` check) | — | §6.5, §6.11 |
| Gyro in its socket (OBC item C3) | M-G370PDT0 stands 14.0 mm above the OBC (11.4 mm to the carrier): **through a 27 × 27 mm window in a 2.0 mm carrier** (no frame; the thicker board keeps the carrier's Steinberg margin, +0.08), 1.5 mm clear of the carrier and 3.6 mm of the Jetson module; raising the carrier 3 mm and a bracket with a ribbon were evaluated and do not fit | OBC_BOARD_DESIGN §9.2, §24 C3 | §4.3 |
| Wing sweep (0–90°) | 0.35 mm minimum clearance (at 45°, wing back cells to body-panel coverglass) | > 0.3 mm allowance | §6.6 |
| AntS tapes to the stowed wings (STR-16) | 6.5 mm (tape band to the wing's −Z edge, wing shortened and moved +3 mm); 0.5 mm with the 98 mm wing centred; deployed 2 m tape to the deployed wing 37.8 mm | ≥ 10 mm or wings first | §6.7 |
| Wing hinge and deployment springs (MCAD-01, STR-13) | Version A springs do not fit; **Version B design fits**: two 17-7PH torsion springs on the hinge axis (primary below the wing, secondary at +Z), clock-spring flex at R 1.0–1.6 mm. Torque margin (5017B, analysis factors) **+0.14** at the cold end of travel, one spring out +0.13; spring stress MS +0.12 / **+0.04** (yield); damped soft stop 0.157 N·m (214 µε); no latch, holding margin +17 | TM ≥ 0 (5017B Eq. 4-1); Table 4; DDMR 14, 53; ≤ 0.3 N·m (STR-14) | §6.8; SQSO-MECH-001 B |
| Mass, CG | 1,513.9 g CBE / 1,747.8 g MEV (= MEL, with the 20 g inertia ballast of row 36, chair decision C-13; rows 32 and 26 to the harness design, D-2; row 23 at 66.6 g with the SoC pedestal; row 5 re-split for the skeleton deck, MCAD-02); stowed CG (+0.1, −0.3, −1.3) mm CBE | ≤ 2.00 kg; ±20 mm (5 mm baseline) | §5 |
| Deployed inertia (STR-01) | Iyy/max(Ixx, Izz) **1.055 CBE / 1.046 MEV** (four AntS elements, 2 × 10 g Z-axis ballast, C-13, harness zones; 1.045 / 1.037 without the ballast); 5th percentile with 2 mm placement scatter 1.045 / 1.036; major axis **1.0° / 1.0°** from +Y | ≥ 1.03; ≤ 5° (BASELINE §4) | §5 |
| First natural frequency, stowed (FE) | **169 Hz** (the deck with its harness dress; stowed wings 170 Hz, out of plane); RF 385, +Z end frame with the ballast 387, EPS 427, carrier 480, OBC 574, battery 1,285 Hz | > 40 Hz (UG v11 §4.1.1) | §8.2 |
| Margins of safety at the design limit load (larger of the quasi-static 10 g / 17 g and the 3σ random response at 8.18 Grms), NASA-STD-5001B protoflight factors | All positive. Lowest: **+1.00** yield (−Z end frame at the battery isolators), +1.17 (end frames at the rail corners), +1.45 (centre support), +2.40 (stack columns), +2.76 (rails); with the 20 g ballast on the +Z end frame (C-13) and the harness design's mass distribution; Ti-6Al-4V stud of the (+40, +43) column +28; G-10CR isolator +4.6 (FS 2.0) | MS ≥ 0 | §8.5 |
| PCB fatigue (Steinberg, FE relative displacement, 3σ at 8.18 Grms) | **+0.08 to +0.37** with the two centre supports (D-7; EPS, OBC and RF boards 1.6 mm) and the gyro window in a 2.0 mm carrier (D-11); −0.30 (EPS) and −0.04 (carrier) with one support; EPS +0.23 with the harness under it tied to the deck straps (−0.52 if tied to the EPS: a tie-down rule, §8.5) | MS ≥ 0 | §8.5 |

Verdict: **the mechanical design is complete at envelope level and meets the CDS envelope, the mass-properties
requirements and the structural requirements with positive margins**, with the design changes of §4.2. It is not yet a
drawing package: the bought frame, panels, wings and AntS have no vendor ICD in hand, the wing hinge closes with thin
margins that rest on measured friction and flex torques (§6.8, VI-STR-04), and the 5° principal-axis requirement depends
on placement to about 2 mm (§5.4).

## 2. Applicable Documents

| Document | Use |
|----------|-----|
| Cal Poly CubeSat Design Specification Rev 14.1 (CP-CDS-R14.1, 2022) and drawing CDS-14-001 (1U, Appendix B) | §2.1.9, §2.2.1–§2.2.14, §2.3.2, §2.3.5, Table 2; standoff note "(0.5–7.0) 0.1 MIN ±Z faces" |
| SpaceX Rideshare Payload User's Guide v11 (August 2026) | §4.1.1 (> 40 Hz), Table 4-1 (10 g / 17 g), Table 4-6 (MPE), Appendix B (dynamic model), Appendix C (CAD model: STEP AP214, outer mould line), Table G-3 |
| NASA-STD-5001B w/Change 3 | Factors of safety (protoflight 1.25 / 1.40; composites 1.50 / 2.00) |
| `docs/environment/ENVIRONMENTAL_TEST_PLAN.md` Rev C §3.1 | Protoflight random level: envelope of MPE + 3 dB and the GEVS-B workmanship spectrum, 8.18 Grms (analysed here) |
| D. S. Steinberg, *Vibration Analysis for Electronic Equipment*, 3rd ed. | PCB displacement criterion |
| A. W. Leissa, *Vibration of Plates*, NASA SP-160 | Corner-supported plate, FE verification |
| SQSO-LVR-001 (`docs/mission/LAUNCH_VEHICLE_REQUIREMENTS.md`) Version B | LVR-004, LVR-005, LVR-020, LVR-025–027, §9 (L-8 M deliverables) |

## 3. The Model

### 3.1 Files and Use

| File | Content |
|------|---------|
| `hinge.py`, `hinge_results.json` | Wing hinge and spring design (Version B): spring search within the envelope, NASA-STD-5017B torque margins, spring stress, deployment dynamics with the damped stop, latch trade, flex torque. `python3 hardware/mechanical/hinge.py` (numpy; about 30 s) |
| `model.py` | Parameters (dimensions, tolerances), every part as a solid (stowed and deployed), MEL mass assignment, mass properties, checks, STEP and SVG export. `python3 hardware/mechanical/model.py` in the CadQuery environment; about 10 s |
| `results.json` | Parameters, the part list (101 parts: name, MEL rows, mass, centroid, volume), mass properties, every check with its value |
| `smartqso_stowed.step`, `smartqso_deployed.step` | Full assemblies, STEP AP214, 156 solids, 3.3 MB each |
| `smartqso_oml_stowed.step`, `smartqso_oml_deployed.step` | Outer mould line and dispenser interfaces only (rails, end frames, panels, wings, hinges, AntS, GNSS patch, switches, plungers, RBF, access port; 59 solids), 1.2 MB each: the form SpaceX UG v11 Appendix C asks for (no internal components) |
| `drawing_stowed_3view.svg` | Sheet 1: three views of the stowed spacecraft (from +X, +Y, +Z) with the CDS envelope, rail dimensions, the protrusion envelope and the stack, the AntS and patch standoff clearances, and the switch, plunger, RBF and access-port locations |
| `drawing_deployed.svg` | Sheet 2: deployed configuration from −Z and from +Y, wing and antenna extents, deployed inertia |
| `fem.py`, `fem_results.json` | FE model, its self-test, eight load, boundary and design-variant cases (§8), with the gyro's acceleration response |

The MEL is read from `software/simulation/cdr/structures/mass_properties.py` (MASS_BUDGET §3.1), so the CAD and the MEL
cannot diverge: the assignment fails if any row is not carried exactly. Axes: CDS body frame (origin at the geometric
centre, +Z along the rails, −Z inserted first).

### 3.2 Bought-Part Envelopes and Sources

| Part | Envelope used | Source |
|------|---------------|--------|
| 1U frame | Rails L 8.5 × 8.5 × 1.5 mm, standoffs solid 8.5 × 8.5 × 6.75 mm, end frames 1.6 mm, side shear panels 1.0 mm, inner envelope 97 mm | CDS App. B; Spacemanic 1U structure (inside envelope 98.4 mm, 115 g, as quoted by a search listing of its datasheet; the datasheet PDF returned 404); ISIS 1U (87 / 108 g, BUILD_VS_BUY §2.2). Wall thicknesses are allocations |
| ISIS AntS 1U–3U | 98 × 98 × 7 mm, 10 × 10 mm corner notches for the standoffs, tapes 5 mm wide, 2 m tape 500 mm, 70 cm tape 170 mm | `hardware/rf/ANTENNA_DESIGN.md` 2.0 §2; BUILD_VS_BUY §2.6. ISIS datasheet ISIS-ANTS-DSH-0001 could not be retrieved (vendor pages 404) |
| CubeSpace CR0002 | 10.5 × 10.5 × 47 mm, 16.5 g | CubeSpace Gen 2 actuators brochure (2023) and cubesatshop.com listing (marked preliminary by the vendor). MASS_BUDGET used 70 mm |
| Jetson Orin Nano 8 GB | 69.6 × 45 mm SO-DIMM module, 260-pin connector, stack height 4 mm (allocation) | NVIDIA module form factor (data sheet not re-read for this issue) |
| Epson M-G370PDT0 | 24 × 24 × 10 mm module; 24 × 24 × 14.0 mm in its socket X1 (Samtec CLM-110 / FTMH-110), modelled as one block | BUILD_VS_BUY §5.2; OBC_BOARD_DESIGN §9.2 |
| INR18650-35E | Ø18.5 × 65 mm, 50.0 g max | Samsung SDI specification |
| GNSS receiver SkyFox piNAV-NG/FM | 50 × 50 × 9 mm allocation, 24 g | BASELINE §6; vendor drawing not obtained |
| GNSS patch Spacemanic ZENITH V3a | 50 × 50 mm, thickness 5 mm (allocation) | BASELINE §6 |
| Body panels and wings | 82 × 98 × 1.6 mm FR-4 (body), 81.6 × 92 × 1.6 mm (wings), cells AZUR 3G30A as one 80.7 × 80.15 × 0.35 mm block per face | BUILD_VS_BUY §2.3; AZUR datasheet DB 00010891-01 (cell size) |

## 4. Layout B as Modelled

### 4.1 Stack (from −Z to +Z; z in mm)

| z range | Item | Mounting |
|---------|------|----------|
| −55.5 … −48.5 | ISIS AntS housing (outside the recessed −Z face) | Vendor pattern on the −Z end frame |
| −48.5 … −46.9 | −Z end frame, 1.6 mm, 70 × 70 mm cut-out | Rails |
| −46.9 … −38.9 | Four G-10CR isolator bushings at (±38.5, ±31) | −Z end frame |
| −44.9 … −20.9 | Battery: four cells along X (y = ±9.25, ±27.75), Al clamp (end caps, side bars, four ears at z −38.9 … −36.4), 2 mm air gap to the end frame | Isolators |
| −38.2 … −27.7 | Y rod at (+42, 0), beside the battery | Clips to the +X wall |
| −44.0 … +3.0 | Z rod at (−41.5, 0), beside the battery, through notches in the deck and the EPS board | Clips to the −X wall and the deck |
| −20.4 … −17.9 | Battery interconnect and sense board on the clamp (flying leads) | Clamp |
| −16.5 … −15.5 | Torquer deck: Al 1.0 mm skeleton (3 mm rim, column bosses, straps under the X rod and the GNSS receiver, strap to the upper centre support), 7.8 g | Columns |
| −15.5 … −5.0 | X rod at (0, +36); GNSS receiver at (+5, −20) | Deck |
| −4.0 … +3.6 | EPS board (substrate −3.0 … −1.4) | Columns, centre supports at (0, ±21) |
| +4.0 … +9.6 | OBC (substrate +5.0 … +6.6), parts ≤ 3 mm top / 1 mm bottom (envelope) | Columns, centre supports at (0, ±21) |
| +6.6 … +20.6 | M-G370PDT0 in socket X1 at (−20, +30.5), 14.0 mm high, standing through the carrier window (§4.3) | Socket and two screws on the OBC |
| +16.6 … +21.6 | Jetson carrier, **2.0 mm** (substrate +17.6 … +19.6, top face unchanged), 27 × 27 mm window at x −33.5 … −6.5, y +17.0 … +44.0, no frame | Columns, centre supports at (0, ±21) |
| +19.6 … +27.8 | SO-DIMM on the −Y edge (4.0 mm stack), Jetson module centred at (0, −6) on two steel standoffs MP2/MP3 at (±31.3, +12.6); SoC lid 20 × 20 mm at (0, −6); gap pads (1.0 mm on the SoC, 2.0 mm on the DRAM and regulators, 25 % deflection); clamp bars at both short edges | SO-DIMM and standoffs on the carrier (§4.5) |
| +27.05 … +32.8 | Heat spreader 80 × 60 × 5 mm at (0, −3) with a 22 × 22 × 0.75 mm pedestal over the SoC lid and a 0.5 mm second face over the DRAM and regulators (§4.5), flanges 3 mm to the ±X shear panels (y −20 … +20) | 3 × M2.5 per side with a thermal gasket, to the shear panels |
| +33.3 … +43.4 | RF board (substrate +34.8 … +36.4) with an 84 × 54 × 7 mm shield; ≤ 2 mm parts outside it | Columns, centre support (0, 0) to the spreader |
| +41.05 … +48.4 | Magnetometer board A2 (RM3100; OBC item C13) at (+24, +27): 24 × 24 × 1.6 mm board on the end frame, parts below it (Sen-Z-f 5.75 mm); GNSS patch adapter at (0, 0); inertia ballast discs (row 36, D-9) at (−17, +15.5) and (−17, +0.5), z +44.8 … +48.4 | +Z end frame: A2 by three Ti M2 screws (heads on the +Z face, outside the GNSS patch), brass nuts, polyimide washers (no frame bond) |
| +48.4 … +50.0 | +Z end frame, 1.6 mm, 30 × 30 mm cut-out | Rails |
| +50.0 … +55.0 | GNSS patch | +Z end frame |
| ±X, ±Y faces | Shear panels 1.0 mm (outer face 0.5 mm below the rail plane), body panels; wings over ±X; harness in four 3 × 10 mm bundles in the ±X wall gaps at y = ±(24 … 34) | Frame |
| Hinge lines (x = ±54.1, y = +38.75) | −Z frame ear −49.0 … −47.6, primary spring −47.6 … −40.1 (below the wing), flex-cup knuckle −39.9 … −37.3; +Z knuckle 43.6 … 44.6, secondary spring 44.8 … 48.4, frame ear 48.6 … 49.6 | Ears bolted to the shear panels through the body-panel notches |

Stack columns: four M3 A286 studs in Al hex spacers (5 AF) at (±40, ±43), from end frame to end frame (STR-06). Centre
supports (STR-05, adopted by the chair 2026-10-27 with 1.6 mm boards): (0, +21) deck → EPS → OBC → carrier → spreader;
(0, −21) EPS → OBC → carrier (§8.5); (0, 0) spreader → RF. The spreader is bolted to the ±X shear panels at mid-height, so it is the stiff anchor of the
upper stack.

### 4.2 Design Decisions Made Here (changes to the reference layout and the other documents)

| # | Decision | Reason |
|---|----------|--------|
| D-1 | −Z end-frame outer face recessed 1.5 mm (frame RFQ: ≥ 0.8 mm) | AntS 7.0 mm otherwise stands 0.25 mm beyond the rail ends (STR-11) |
| D-2 | Side shear-panel outer face 0.5 mm below the rail plane (frame RFQ) | Protrusion stack (STR-10): without it the worst case is 6.63 mm |
| D-3 | Wing 81.6 × 92 mm (was 83 × 98), z −43 … +49; cell block 80.7 × 80.15 mm with 5.9 mm end zones | Rail-zone margin (CDS §2.2.5) and the AntS tape clearance (STR-16); end zones carry the hinges, the hold-down and the snubber |
| D-4 | Hinge axis (±54.1, +38.75) mm; hinge hardware of radius ≤ 2.0 mm on the axis: primary spring below the wing (z −47.6 … −40.1), secondary spring in the +Z end zone, flex cup in the −Z end zone (SQSO-MECH-001 B §3A) | Spring 1.25 mm inside the rail-zone limit and 0.40 mm (nominal) / 0.12 mm (worst case) inside the protrusion limit; sweep clear of the body cells |
| D-5 | Hold-down: cup/cone post at the −Z free corner (y −38.5, z −40) with the line over it; PTFE-faced silicone snubber at the +Z free corner | No free strip at the free edge: the cells fill the wing width; PTFE face removes 0.3 mN·m of breakaway adhesion |
| D-6 | Jetson module centred at y = −6 mm (spreader −3 mm), SO-DIMM on the −Y edge; RBF and access port on −Y in the −Z end zone; harness in four symmetric bundles | Cancels Iyz (§5.3) |
| D-7 | Second centre support at (0, −21) mm, EPS → OBC → carrier; boards stay 1.6 mm (chair decision) | Steinberg margins of the EPS and carrier boards (§8.5) |
| D-8 | Torquer deck as a 1.0 mm Al skeleton (7.8 g; 9 g with clips from MEL row 5) for the X rod and the GNSS receiver; Y and Z rods beside the battery | The 47 mm rods and the receiver have no other place in the 94.5 mm between the end frames; MCAD-02 |
| D-9 | **Inertia ballast** (MEL row 36, chair decision C-13): two 10 g W-Ni-Cu discs Ø14 × 3.6 mm (non-magnetic; not W-Ni-Fe) on the inside face of the +Z end frame at (−17, +15.5) and (−17, +0.5) mm (pair centre at y +8 to cancel the harness distribution's Iyz, §5.3), z +44.8 … +48.4, each on two M2 Ti-6Al-4V screws, 22 mm from A2; machined to mass and trimmed (or removed) at integration from the measured tensor (VI-MCAD-03). Two 5 g Iyz trim-mass positions on the +Z end frame at y = ±45 mm, W-Ni-Cu (ASTM B777, non-magnetic binder) or pure tungsten, not W-Ni-Fe (the +Y one is 13 mm from A2) | §5.2, §5.4, §4.4 |
| D-10 | No deployed-position latch; damped silicone stop (ζ ≥ 0.3); MoS₂ pins; legs 5 mm long | SQSO-MECH-001 B §4A–§5A |
| D-11 | Gyro (M-G370PDT0 in socket X1) at (−20, +30.5) on the OBC, standing 14.0 mm through a 27 × 27 mm window in the Jetson carrier, which is 2.0 mm thick (bottom face 0.4 mm lower, top face and everything above unchanged); no frame, no bond (§4.3.3) | OBC item C3: 14 mm needed, 11.4 mm available; the two options proposed do not fit (§4.3) |
| D-12 | Harness as the harness design's zones (SQSO-HAR-001 2.0 §7.1, M-1): EPS bottom-side connector envelope 9.0 mm (z −3 … −12) in zones Z1–Z5 and J3 at +X; four grommeted 6 × 4 mm slots in the shear panels at z −10 for the body-panel pigtails (M-2); access-port board A3 with its Micro-D 21 in a 30 × 7 × 8.8 mm envelope at x −35 … −5, z −46.8 … −38, through a matching −Y body-panel and shear-panel window (M-3); tabs TB-W1 / TB-W2 at (±48, +37, −37 … −31) with the hinge-flex tails through 1.5 × 2.8 mm shear-panel slots (M-4); ±Z sun sensors at (−36, +36) on the +Z end frame and on the −X body panel's −Z end zone at (−52.6, +28, −50) looking −Z (M-5); W4A flat without the fold (M-8); RF J1 TFM-108 right-angle, shield notch x > +30 (RF_BOARD_DESIGN D §4.3) | §6.9, §6.11 |
| D-13 | AntS and GNSS patch mounts DC-isolated from the frame (≥ 1 MΩ): polyimide or G-10 washers and sleeves under every fastener, isolated faying faces, as for A2 (SQSO-HAR-001 2.0 M-9; single-point ground) | §6.11, VI-MCAD-09 |

### 4.3 Gyro Mounting (OBC Item C3)

The OBC board design (SQSO-OBC-BRD-001 §9.2, §24 item C3) puts the Epson M-G370PDT0 (24 × 24 × 10 mm, 10 g) in a Samtec
CLM-110 socket (X1), so that it stands about 14 mm above the OBC's top face; the stack leaves 11.4 mm (OBC top face
+6.6 mm, carrier substrate +18.0 mm). The chair asked for a choice between raising the carrier 3 mm and a frame bracket
with a 0.1 m ribbon. Neither fits layout B; a third arrangement does, and is adopted (D-11):

| Option | Stack height in the 113.5 mm envelope | Alignment to the body axes | Vibration environment | Mass properties (deployed, CBE / MEV) | Result |
|--------|---------------------------------------|----------------------------|-----------------------|----------------------------------------|--------|
| (1) Raise the carrier, module, spreader and RF board 3 mm | Needs 3.0 mm (2.6 mm with zero clearance at the gyro). Free above the stack: 1.0 mm (RF shield top to the GNSS patch adapter) and 2.0 mm (RF parts to the magnetometer); below the OBC: 1.0 mm (EPS bottom parts to the X rod), 2.5 mm (to the GNSS receiver) and 1.4 mm (deck to the battery sense board). Raised 3 mm, the RF board overlaps the patch adapter by 2.0 mm and the magnetometer by 1.0 mm; lowering the EPS and OBC instead runs into the X rod after 1.0 mm. The 3.0 mm exists only with every one of these gaps at zero, or with a notched RF shield, a shorter patch adapter and an EPS keep-out over the rod | Unchanged (board-mounted) | Unchanged (gyro on the OBC) | CG z +0.6 mm; Iyy/max 1.072 / 1.063, axis 0.9° / 2.3° (better ratio: the upper stack moves away from the CG) | **Does not fit** without three other designs changing |
| (2) Frame bracket and a 0.1 m FFSD-10-D ribbon to an FTSH header at X1 | The module with a mated ribbon socket is as tall as on its socket (about 14 mm), so the bracket needs a free 24 × 24 × 15.5 mm volume (module, connector, 1.5 mm bracket). A voxel search of the CAD (1 mm grid, every solid) finds none in layout B in any orientation; the largest 24 × 24 footprint is 13.9 mm high, through a lightening hole of the torquer deck between the battery sense board and the EPS, with no room for the bracket. The ribbon would also have to pass the EPS board through the 3.3 mm ±X wall gaps | A machined bracket on the frame could be held to about 0.1° (better than a board) | The deck is the softest internal structure (195 Hz, §8.2); a frame bracket elsewhere does not exist | On the deck: CG z −0.16 mm; axis 1.3° / 3.1° (the gyro moves 24 mm toward −Z, adding Iyz) | **Does not fit** |
| **(3) Gyro on its socket on the OBC, through a 27 × 27 mm window in the Jetson carrier, in a 2.0 mm carrier (adopted)** | No stack height change. Gyro at (−20, +30.5) mm, top at z +20.6; the window (x −33.5 … −6.5, y +17.0 … +44.0) clears it by 1.5 mm on every side; 2.0 mm beside and 3.0 mm below the Jetson module board's far edge (y +16.5; 3.6 mm in all); 3.4 mm from the module standoff MP2 at (−31.3, +12.6), whose pad sets the window's −Y edge (§4.5); 5.1 mm to the centre support at (0, +21). Nothing is above the gyro below the spreader (z +27.8): a socketed height up to 20 mm still clears it | As the OBC board: socket on a 1.6 mm board on six supports (§4.3.1) | The OBC board's response (below) | No change to the mass properties beyond the carrier's own | **Fits** (check OBC-C3, `results.json`); carrier Steinberg margin kept by the 2.0 mm board |

**What the carrier gives up.** 729 mm² (8 %) of the carrier's 8,646 mm², in its +Y, −X quadrant outside the module
footprint. The window keeps a 3.9 mm web to the board edge, 3.0 mm to the centre support's keep-out (Ø7 mm) and 3.0 mm to
the corner column's. It was first placed at y +14.0 … +41.0; the carrier design (SQSO-JET-BRD-001, item M1) puts the
module standoff MP2 at the module's far-edge hole, (−31.3, +12.6) mm by the module outline, whose Ø5.1 mm spacer and pad
need board to y ≈ +16, so window and gyro moved 3 mm toward +Y. The window lies in the carrier's −X power zone of its +Y
strip; the carrier designer has placed those parts round it. FE (§8.5): in a 1.6 mm carrier the window alone takes the
carrier's Steinberg margin from +0.08 (same mesh, no window) to **−0.04**: it cuts the board between the corner column
(−40, +43) and the centre support (0, +21). **The design is a 2.0 mm carrier: +0.08** (with the 2.0 mm allowable,
0.154 mm), no frame. The Al frame of the first issue (+0.08 with a 1.6 mm board) is withdrawn because its bond cannot be
closed by analysis (§4.3.3). The 2.0 mm board costs 0.4 mm of height below the carrier (the OBC's free height under it
falls from 9.4 to 9.0 mm, §4.4) and about 5.9 g of FR-4 (2.7 g net of the frame and its four screws), inside MEL row 21's
growth allowance (40 g, 25 %; no MEL change).

#### 4.3.1 Alignment

The sun-pointing law spins the spacecraft at 8°/s about +Y (BASELINE Rev E §8); a gyro axis misaligned by δ reads
8°/s × sin δ on the cross axes: 0.14°/s per degree, as large as the whole 0.1°/s (1σ) bias budget of SENSOR_DESIGN §4.1
at δ = 0.7°. The
board-mounted gyro's mechanical alignment is estimated at 0.6° RSS (1.0° worst case): board bow and twist over the 24 mm
footprint (IPC-6012 Class 3, 0.75 %: up to 0.43°), socket seating under the two screws (0.3°), the stack columns and
supports to the frame (0.1°) and the in-plane rotation of the socket footprint (0.3°). None of the options can rely on
the mechanical tolerance alone (a frame bracket would still be about 0.1–0.2° plus the module's own axis-to-case
alignment, which the data sheet was not read for). **Verification item VI-MCAD-05**: measure the gyro's sensing axes
against the frame (rate table about the three body axes, or an alignment cube on the module) on the integrated flight
stack to ≤ 0.1° and give the flight software the rotation matrix (an interface item for the chair).

#### 4.3.2 Vibration Environment

The FE puts the gyro's 10 g on the OBC shells under the window footprint and gives the absolute acceleration there
(mean of the footprint nodes, 450 modes, the 8.18 Grms protoflight test envelope):

| Input axis | Response x | Response y | Response z |
|------------|-----------|-----------|-----------|
| X | **21.5 Grms** | 4.4 | 7.3 |
| Y | 4.4 | **24.3 Grms** | 12.4 |
| Z | 3.7 | 4.3 | **23.9 Grms** |

About 2.6–3.0 times the input in the input axis (the stack modes at 318–387 Hz; design case with the ballast and the
harness distribution). The bracket on the deck would have put it on the 169 Hz deck, and a frame bracket does not exist (option 2). The 3σ motion of the carrier's window edge
relative to the gyro is 0.07 mm against the 1.5 mm clearance. The comparison with Epson's vibration and shock
ratings for the M-G370 family is the OBC design's verification item V-OBC-16 (SQSO-OBC-BRD-001 §9.2; no isolator), with
the EM board random test (VI-STR-02) and powered checks before and after.

#### 4.3.3 Window Frame Withdrawn: the Bond Does Not Close

The first issue of D-11 kept the 1.6 mm carrier and stiffened the window with a bonded Al 6061 frame (2.5 × 3.0 mm) and
four screws (H10–H13 of SQSO-JET-BRD-001 C §18.1, positions amended in tapped pads). Closing the bond was asked for in
this round. It does not close:

- **Thermal mismatch.** Al (23.6 ppm/K) against FR-4 in plane (about 15 ppm/K): 8.6 ppm/K. From a room-temperature cure to
  the −40 °C non-operating minimum (THERMAL_ANALYSIS: boards −40 … +70 °C non-operating) the free strain is 5.6 × 10⁻⁴.
  The shear-lag peak at the frame's corners is τ = Δα ΔT · G / (t λ) with λ² = (G b / t)(1/E_f A_f + 1/E_b A_b): for a
  0.13 mm bond line of Scotch-Weld 2216 (shear modulus 1,947 MPa at −40 °C, 3M data sheet) about 25 MPa, and 13 MPa even
  at a 0.5 mm bond line, against a lap-shear strength of the same order. A stiffer epoxy is worse (τ ∝ √G).
- **Outgassing against temperature.** The low-outgassing structural epoxy on the project list, 2216 Gray (TML 0.77 %,
  CVCM 0.04 %), has no published strength above room temperature on its data sheet and is known to soften above
  about 60 °C; the high-temperature paste EA 9394 (20.7 MPa lap shear at 82 °C, Henkel data sheet) has a TML of 1.06–1.73 %
  in the published E595 results (1.06 %, 1.72 % and 1.73 % for different lots and cures), above the CDS §2.1.7 limit of 1.0 %.
- **Screws alone.** Four point attachments do not make the frame act with the board along its edges: the stiffening the
  FE credits needs a continuous shear connection.

**Withdrawn.** The window stays; the carrier is 2.0 mm thick with no frame, no adhesive and no screws (FE +0.08,
§8.5). For the carrier design: H10–H13 and the frame keep-outs are deleted; the top-side and inner-layer window keep-out
(window + 0.5 mm) stays; MP2's top-side pad no longer sits over anything. The stack-up becomes 2.0 mm (impedance
re-calculated by the carrier designer). The frame stays in `model.py` and `fem.py` as an option (`car_frame`) for the
record; its FE case is in §8.6.

### 4.4 OBC Board Placement Constraints That the Stack Sets (SQSO-OBC-BRD-001 §21)

The OBC's layout guidance is in its §21 (its §22 is the verification list). The items that touch the stack, and what
the CAD gives them:

| OBC §21 item | Stack constraint (this model) | Status |
|--------------|-------------------------------|--------|
| Outline, holes: PC/104 90.17 × 95.89 mm, four Ø3.18 mm holes, the two centre supports at (0, ±21), Ø6.4 / Ø7 mm copper-free rings; **no frame bond on the OBC** (OBC design B): every hole isolated, the OBC's ground reaches the frame only through the W3 ground wires and the EPS's single bond H1 | Same positions as the CAD; the CAD keeps Ø7 mm free of parts on both sides. The Al spacers are conductive, so the isolation is in the copper-free rings; a second bond, if ever wanted, must be at the EPS H1 column | Consistent |
| Stack-up 6 layers, 1.6 mm | The FE and the Steinberg margins use 1.6 mm (OBC +0.35 with the two supports and the 2.0 mm carrier; §8.5) | Consistent |
| Heights | Top side ≤ 3 mm (envelope) except X1 and the connectors: **connectors and X1 up to 9.0 mm under the 2.0 mm carrier** (its bottom face at z +17.6, its 1 mm bottom-side envelope plus 1 mm clearance; there is no window frame); OBC J2 at (+41.0, −33.5) with an 8 mm mated allocation (§6.9); the mated connectors J1–J10 (header, plug and cable bend) checked at layout against the Samtec and Molex drawings. **Bottom side ≤ 1.0 mm**: the EPS's 5 mm top envelope leaves 0.4 mm below it | Constraint for the layout |
| X1 (gyro) "near the board centre on the spin-axis side" | At (−20, +30.5) mm, 24 × 24 mm, under the carrier window of §4.3; the board centre is under the Jetson module and the centre supports | Placed |
| J1 on the edge facing the EPS's J3 (W3, 40-way ribbon, about 26 mm wide) | J1 cannot be on the bottom side (1.0 mm), and a mated FTSH/ribbon socket on the EPS top side does not fit under the OBC (6.4 mm between the substrates; mated height open, Samtec drawing). The ribbon can only leave the stack in the ±X wall gaps (3.3 mm, x ±45.4 … ±48.4; the ±Y gaps are 0.55 mm): J1 and J3 at the same ±X edge, at |y| < 24 mm (the harness bundles occupy |y| = 24 … 34), on the +X edge to stay clear of the Z rod (x −46.8 … −36.2, |y| < 5.3, up to z +3.0) | Constraint for the OBC and EPS layouts |
| J2 "on the +Z-facing edge toward the RF board" (W4A, FTSH-107, about 9 mm wide) | J2 on the top side; the ribbon passes the carrier and the spreader in a ±X wall gap at |y| = 34 … 44 mm (the spreader flanges fill |y| < 20 at z +29.8 … +32.8, the harness bundles |y| = 24 … 34) | Constraint for the OBC and RF layouts |
| DRV8876s and J7 (rod leads) on the −X edge nearest the Z rod's top | The Z rod ends at z +3.0 under the OBC at (−41.5, 0), 1.0 mm below the OBC's bottom-side envelope: J7 on the top side or outside the rod footprint (x −46.8 … −36.2, |y| < 5.3) | Constraint for the layout |
| Magnetometer: **RM3100 on board A2 at the +Z slot (+24, +27), z +41.05 … +48.4** (OBC design B, item C13; Version A had it in the OBC's (+X, −Y) corner) | A2 in the CAD (check MAG-A2): board 24 × 24 × 1.6 mm on the end frame, parts below it to z +41.05 (Sen-Z-f 3.0 × 3.0 × 5.75 mm, PNI datasheet), 0.65 mm above the 8 mm slot's floor and 2.0 mm above the RF board's top-side parts; three M2 Ti-6Al-4V screws at (+33.5, +17.5), (+14.5, +36.5), (+33.5, +36.5), heads on the +Z face outside the GNSS patch footprint (6.8 mm from it), brass nuts on the parts side, polyimide washers and an isolated back face (no frame bond, as for the OBC). The 20–30 mm keep-out on the carrier round (+33, −38) is withdrawn (OBC item E11) | Placed |
| 20 mm non-ferromagnetic zone round A2 (OBC design §9.4) | Parts within 20 mm of A2 in the CAD, and what each must be: **RF shield lid and clips** (2.0 mm: nickel silver, brass or Al, not tin-plated steel; no ferrite-cored parts on the RF board under A2); **(+40, +43) stack column** (2.8 mm: Ti-6Al-4V stud for that column and Ti or brass nut at the +Z end; the stud carries about 13 % of the column's bending stiffness, 8 % in Ti; FE not re-run); **+Z end frame and rail fasteners** within 20 mm (vendor frame: Ti or A286 screws at the (+X, +Y) corner, no steel); **trim mass at (0, +45, +44)** (13 mm: W-Ni-Cu or pure W, D-9); the **inertia ballast** at (−17, +15.5) / (−17, +0.5) is 22 mm from A2 (outside the zone; W-Ni-Cu, non-magnetic, D-9); harness: only W6E's end (zone TE_PXPY, 0.3 mm from A2, to A2 J1) inside the zone (SQSO-HAR-001 2.0 §7.3); **GNSS patch adapter fasteners** (14 mm: brass or Ti); **wing W1 hinge**: pins A286 (13.5 mm from A2's outline; were 440C; SQSO-MECH-001 D §5A: pin MS +3.3 yield, torque margins unchanged, VI-MECH-03), secondary spring 17-7PH (16.1 mm from the outline, about 32 mm from the coils: a fixed bias that moves once, at deployment; accepted); **module standoff MP3** (17.5 mm, steel SMT spacer: brass of the same size, or accepted as a fixed bias); the carrier's MP1 (steel, about 18 mm below A2, not modelled) and the M.2 card: fixed biases (OBC design §9.4); Al and FR-4 parts (rails, shear panels, end frame, spreader, doublers, boards, cells) are non-magnetic; the body and wing cells' interconnects must not be Kovar or Invar within 20 mm (panel vendor) | For the frame, RF, panel, carrier designers and the frame RFQ |

### 4.5 Jetson Module Mounting and Thermal Interface (Carrier Item M1)

The carrier design (SQSO-JET-BRD-001 A §14, §18, item M1) asks the mechanical design for the spreader pedestal, the
module standoffs, the clamp-force path and a flush fastener at (0, −21). Modelled in `model.py` and checked (JET-M1):

| Item | Design | Check |
|------|--------|-------|
| SoC lid and parts (envelopes until the NVIDIA STEP model is used) | SoC lid 20 × 20 × 1.5 mm at the module centre (0, −6) (the carrier design's 4.0 cm² lid allocation); DRAM and regulators 1.0 mm high over 40 × 28 mm round it | — |
| Gap pads | 1.0 mm on the SoC lid, 2.0 mm on the DRAM and regulators, both at 25 % deflection (0.75 / 1.5 mm installed) | MEL row 22 (pads about 5 g of the 10 g interface allocation) |
| Spreader pedestal | 22 × 22 × 0.75 mm boss machined on the spreader's lower face over the SoC lid (z +27.05 … +27.8), and a 0.5 mm second face over the DRAM and regulators; flatness ≤ 0.05 mm, parallel to the flange plane within 0.05 mm, Ra ≤ 1.6 µm (drawing notes) | 1.8 g of Al: MEL row 23 raised from 65.0 to 66.6 g CBE (chair; MASS_BUDGET v3.1 §1.2) |
| Pad deflection and shims | The gap from the pedestal to the lid is a chain of the SO-DIMM stack (4.0 mm), the module board (1.2 mm, ±10 %), the lid height, the carrier's place in the stack and the flange plane: about ±0.24 mm RSS (allocations), against the 0.25 mm nominal compression of the SoC pad. **Shim the spreader flanges at integration** to the gap measured with the module clamped (the carrier design's 15–35 % deflection band), recorded with a pressure film (V-JET-7) | VI-MCAD-06 |
| Module standoffs MP2/MP3 | Würth 9774040360R steel SMT spacers, M3 × 4.0 mm, at the module's far-edge holes: (±31.3, +12.6) mm in the CAD (3.5 mm from the module's sides and 3.9 mm from its far edge; **to be checked against the NVIDIA module drawing**); module underside envelope relieved round them | 1.85 mm from the gyro window; the steel spacers are 51 mm from the RM3100 corner of §4.4 (beyond its 20 mm rule) |
| Clamp force | ≤ 38 N (28 N SoC pad, 10 N others; 48 % of the 80 N module limit), from the spreader through the pads into the module, the SO-DIMM and the two standoffs, then the carrier and its supports. The spreader, bolted at both flanges, deflects about 0.01 mm under it (5 mm plate over 80 mm): negligible against the 0.05 mm flatness | V-JET-7 (force ≤ 40 N) |
| Fastener at (0, −21) under the module | Low-head M2 A286 screw (ISO 7380 button head, Ø3.5 × 1.1 mm) on the carrier's top face, top at z +20.7: **1.4 mm below the module's underside envelope** (z +22.1), 7.5 mm from the SO-DIMM, 36 mm from the gyro window. A countersunk head in the 1.6 mm board would leave a 0.4 mm land under a structural support and is not used | JET-M1; VI-MCAD-06 (module underside from the NVIDIA STEP model) |
| Carrier top-side heights outside the module | Under the spreader (y < +27): ≤ 7.7 mm (0.5 mm below it); beyond it: ≤ 13.2 mm (0.5 mm below the RF board's bottom envelope) | For the carrier layout |

**Thermal path.** Junction to spreader 1.09–1.16 K/W (carrier design §14: θjc 0.30 K/W allocated, the pad over 4.0 cm²,
spreading 0.10 K/W) against the THM-10 allowable of 5.7 K/W; spreader to frame 1.10 W/K (0.91 K/W, §6.10): junction to
frame about 2.0–2.1 K/W, or about 15 K above the frame at 7 W. Both are estimates (contact conductances, θjc and the lid
area are allocations); V-JET-7 measures the first, the EM thermal balance (TB-07) the second.

## 5. Mass Properties

### 5.1 Mass

Every MEL row (MASS_BUDGET v3.1 §3.1, rods option) is carried by named solids; the CAD total is the MEL total: **1,513.9 g
CBE, 1,747.8 g MEV**, margin +14.4 % to 2.00 kg (row 23 raised to 66.6 g for the SoC pedestal, §4.5; row 36, the 20 g
inertia ballast of §5.2, chair decision C-13; rows 32 and 26 set to the harness design, SQSO-HAR-001 2.0 §11 and item
D-2: harness 75.6 g, from 80.0; ADCS sensor cables 6.6 g, brackets and A2 fasteners 2.0 g and the ±Z sun-sensor
carriers 0.4 g, 9.0 g from 5.0). By group (CBE): stack 437.6 g, battery 267.0 g, body panels 160.0 g, frame 128.0 g
(row 1 plus brackets and deck), wings 110.2 g, harness 106.2 g (rows 32, the 6.6 g of row 26's cables, row 31's 20 g
beyond the patch adapter, the 4 g of wing flex in rows 10/11), ADCS 99.9 g (with the ballast and the ±Z sun sensors),
AntS 85.0 g, distributed 55.0 g, GNSS 52.0 g, inhibits 13.0 g.

**MEL row 5 (internal brackets, 30 g CBE; MCAD-02).** The solid 1.0 mm deck of Version A (23 g of aluminium) exceeded
the 6 g it drew from row 5. The deck is now a skeleton (7.8 g of aluminium: 3 mm rim, Ø10 mm column bosses, a 50 × 12 mm
strap under the X rod, two 84 × 6 mm bars under the GNSS receiver, a 6 mm strap to the upper centre support) and draws
9 g with its clips. The Jetson heat-path flanges, which row 5 carried at 18 g, are 5.5 g of aluminium in the CAD and draw
15 g with the thermal gasket, six M2.5 fasteners and inserts. Row 5 split: deck 9, heat-path hardware 15, hinge doublers 4,
GNSS patch bracket 2 = 30 g: **no MEL change**. If the chair prefers to grow row 5 instead (deck +3 g over its earlier
allocation, flanges unchanged), the MEV becomes 1,749.8 g and the margin to 2.00 kg +14.3 % (from +14.4 %).

**Harness mass distribution (SQSO-HAR-001 2.0 M-6).** The four symmetric full-height bundles of the first issues are
replaced by the harness design's 58 zones (§6.11), each carrying the mass of what is routed through it: every wire's
mass along its route in proportion to segment length (`routes.csv`, `wire_list.csv`), the harness-side connector halves
at the route ends, A3 in its envelope, the tabs on the shear panels, sleeves and ties in proportion to the wire
(`model.py`, `harness_distribution`). Result: 25 g of harness under the EPS (zones U_*, z ≈ −10), centred at x ≈ −7 mm,
as the harness design estimated (22 g); 12.2 g on the W10 coax (with its eight MMCX plugs); 7.4 g of W3 in the +X wall
gap; the corner bundles now carry 2–4 g of harness each (the (+X, −Y) upper section also 7 g of row 31's allocation). Row 31's 17.7 g beyond W10E and the patch adapter (the MEL's GNSS cable
allocation, which no design now fills) is spread along W10E's route by length. The deployed moments fall by 1–2 %
(mass moved from the wall gaps inboard), the deployed CG moves −0.4 to −0.5 mm in Y and the deployed Iyz by +7.5 to +8.3 × 10⁻⁶ kg·m²,
which the ballast pair's y offset cancels (§5.3).

### 5.2 CG and Inertia (about the CG, ×10⁻³ kg·m², tensor convention)

| Case | CG (mm) | Ixx | Iyy | Izz | Ixy | Ixz | Iyz | Iyy/max(Ixx, Izz) | Major axis from +Y |
|------|---------|-----|-----|-----|-----|-----|-----|-------------------|--------------------|
| Stowed, CBE | (+0.14, −0.34, −1.30) | 2.746 | 3.052 | 2.732 | +0.012 | −0.009 | +0.032 | 1.111 | 6.0° |
| Stowed, MEV | (+0.20, −0.45, −0.25) | 3.158 | 3.509 | 3.189 | +0.013 | −0.017 | +0.038 | 1.100 | 6.8° |
| **Deployed, CBE** (four AntS elements: 2 m ±X, 70 cm ±Y; ballast row 36; harness zones) | (+0.14, +1.79, −1.30) | **2.846** | **4.190** | **3.970** | +0.013 | −0.009 | −0.003 | **1.055** | **1.0°** |
| **Deployed, MEV** | (+0.20, +1.60, −0.25) | **3.267** | **4.745** | **4.535** | +0.014 | −0.017 | +0.003 | **1.046** | **1.0°** |
| W1 deployed only, CBE | (+1.34, +0.62, −1.30) | 2.811 | 3.892 | 3.637 | −0.151 | −0.019 | +0.006 | 1.070 | 8.0° |
| Antennas out, wings stowed, CBE | (+0.14, −0.55, −1.30) | 2.771 | 3.599 | 3.305 | +0.012 | −0.009 | +0.016 | 1.089 | 3.1° |
| Deployed, CBE, ballast, four symmetric harness bundles (C-13 issue) | (−0.03, +2.23, −1.23) | 2.861 | 4.268 | 4.043 | −0.010 | +0.017 | −0.003 | 1.056 | 0.9° |
| Deployed, MEV, same | (+0.01, +2.09, −0.18) | 3.286 | 4.843 | 4.627 | −0.013 | +0.016 | +0.003 | 1.047 | 0.8° |
| Deployed, CBE, without the ballast (Version B before C-13) | (+0.19, +2.26, −1.87) | 2.813 | 4.215 | 4.035 | −0.009 | +0.001 | −0.005 | 1.045 | 1.7° |
| Deployed, MEV, without the ballast | (+0.21, +2.12, −0.74) | 3.239 | 4.791 | 4.619 | −0.012 | −0.001 | +0.000 | 1.037 | 0.5° |
| Deployed, CBE, two elements only (2 m +X, 70 cm −X: the former option A, for reference) | (+0.55, +1.79, −1.30) | 2.808 | 3.937 | 3.679 | +0.014 | +0.022 | −0.003 | 1.070 | 0.9° |
| Deployed, MEV, two elements only | (+0.57, +1.60, −0.25) | 3.227 | 4.477 | 4.227 | +0.015 | +0.017 | +0.003 | 1.059 | 1.0° |

Final deployed tensors about the CG (four AntS elements, ballast, harness zones; kg·m², tensor convention,
`results.json`):

- CBE (1,513.9 g): Ixx 2.846 × 10⁻³, Iyy 4.190 × 10⁻³, Izz 3.970 × 10⁻³; Ixy +12.6 × 10⁻⁶, Ixz −8.9 × 10⁻⁶,
  Iyz −3.0 × 10⁻⁶; principal 2.846, 3.970, 4.190 × 10⁻³.
- MEV (1,747.8 g): Ixx 3.267 × 10⁻³, Iyy 4.745 × 10⁻³, Izz 4.535 × 10⁻³; Ixy +13.7 × 10⁻⁶, Ixz −16.9 × 10⁻⁶,
  Iyz +3.1 × 10⁻⁶; principal 3.267, 4.535, 4.746 × 10⁻³.

**Antennas (four elements).** The ISIS AntS 1U–3U deploys up to four tape springs, one from each side of its box (ISIS
datasheet ISIS-ANTS-DSH-0001; brochure), and the RF design (SQSO-RF-BRD-001 C) and ANTENNA_DESIGN 2.1 use all four: 2 m
along ±X and 70 cm along ±Y, which is BASELINE Rev E's "2 m elements along ±X". The CAD carried only two (2 m +X,
70 cm −X) until this issue; it now carries four. The ±Y elements add to Ixx and Izz, so the deployed ratio fell from
1.058 / 1.049 to 1.045 (CBE) / 1.037 (MEV): the requirement (≥ 1.03) met with growth by 0.007 only, and the MEV
5th percentile under placement scatter (§5.4) was 1.032 at σ = 1 mm and 1.027 at 2 mm.

**Inertia ballast (chair decision C-13, D-9, MEL row 36).** The 0.007 margin is too thin to rely on the measurement, so
the ballast is in the design baseline: two 10 g W-Ni-Cu discs (Ø14 × 3.6 mm, ASTM B777 Class 3, non-magnetic Ni-Cu
binder) on the inside face of the +Z end frame at (−17, +15.5) and (−17, +0.5) mm, z +44.8 … +48.4, each on two M2
Ti-6Al-4V screws (row 36's 20 g includes them). A mass near the Z axis at the end of the body adds m(x² + z²) to Iyy
and m(y² + z²) to Ixx but only m(x² + y²) to Izz: at its first placement (CAD, CBE) +5.3 × 10⁻⁵ to Iyy, +4.8 × 10⁻⁵ to
Ixx and +0.8 × 10⁻⁵ kg·m² to Izz. The deployed ratio is **1.055 (CBE) / 1.046 (MEV)** with the harness distribution
(1.056 / 1.047 with the symmetric bundles), the MEV 5th percentile **1.041 at σ = 1 mm and 1.036 at 2 mm** (it stays
≥ 1.03 with growth to σ ≈ 3.1 mm), for 20 g of the 252 g margin to 2.00 kg. Placement: both discs at +Z, because the
−Z end has no room on the axis (the AntS outside, its 70 × 70 mm cut-out and the W10 coax in the frame, the battery
inside); two at one end give the same Iyy as one at each end and move the CG +0.6 mm in Z (stowed CG −1.3 mm, inside
the 5 mm baseline). The pair's centre is at y = +8 mm, not 0: with the harness distribution the deployed Iyz was
+4.6 (CBE) / +10.8 (MEV) × 10⁻⁶ kg·m² (axis 1.3° / 2.9°), and the 8 mm offset supplies −7.7 × 10⁻⁶ at no mass cost
(axis 1.0° / 1.0°). The x offset (−17 mm) clears the GNSS patch adapter at the centre by 5.0 mm and keeps the discs
22 mm from A2; the discs are 1.4 mm above the RF board's shield-lid allocation (7.0 mm; the RF board's 3σ random
displacement is 0.14 mm, §8.5) and 1.6 mm (the end-frame plate) from the GNSS patch outside. **Integration adjustment
(VI-MCAD-03):** the measured stowed tensor plus the weighed wings sets the disc mass, from 10 g each (nominal) down to
removal; each gram of ballast (shared by both discs) moves the ratio by about 0.0005, and unequal discs (10 + Δ,
10 − Δ g, 15 mm apart in y) trim Iyz by about 0.7 × 10⁻⁶ kg·m² per gram of Δ, so the trim is a refinement, not the
means of meeting the requirement. The ADCS Monte Carlo of BASELINE §8 gives the published result at Iyy/Izz ≥ 1.03
(MASS_BUDGET v3.1 §6.4: 1.041–1.054 cases all 2.5–2.6° mean, 100 % of sunlit time within 15°).

Against MASS_BUDGET v3.0 §5–§6 (reference layout B): the stowed CG is 3.4 mm higher in Z (−1.3 against −4.7 mm: the CAD
battery centre is −32.9 mm, not −35, the boards sit higher, and the ballast adds +0.6 mm), the deployed moments are within
3 % on every axis, and without the ballast the ratio agrees (four elements: 1.045 CAD against 1.041; two elements:
1.058 against 1.054). The difference that matters is the principal axis: 1.0° here against 13–15° there (§5.3). The
baseline requirement (Iyy ≥ 1.03 × max, axis within 5°) is **met** at CBE and MEV for the full deployment; with one wing
out the axis is 8.0° off (as in MECHANISMS_ANALYSIS §8, a degraded mode), and with the antennas out and both wings
stowed 3.1°.

### 5.3 Balancing the Principal Axis

Iyy − Izz is 0.22 × 10⁻³ kg·m² (with the ballast), so the axis tilt is θ = ½ atan(2 Iyz / (Iyy − Izz)): 4 × 10⁻⁵ kg·m² of
Iyz tilts it by 10°. The deployed wings move the CG +1.9 mm in Y; every part far from the CG in Z then adds to Iyz. The
first CAD placement gave 17°; moving the Jetson module, clamp and spreader to y = −6 mm (with the SO-DIMM on the −Y edge
and the upper centre support at y = +21 mm), the RBF and access port to −Y and the harness into four symmetric bundles
cancelled it by placement (MASS_BUDGET v3.1 §6.6). Version B moved them from −7 to −6 mm to absorb the Iyz of the hinge
springs, flex cup and skeleton deck (a more negative y made it worse). With the harness design's distribution (§5.1:
more mass under the EPS and at −Y, A3 at −Y) the residual Iyz is taken by the ballast pair's +8 mm offset in y (§5.2),
so no board or module moves.

### 5.4 Sensitivity and Trim

A placement Monte Carlo (every internal part, the ballast and the harness zones included, moved by an independent
Gaussian error σ per axis, 2,000 trials; frame, panels, wings and AntS held) gives, with the ballast, the harness
distribution and four antenna elements:

| σ per axis | Mean angle CBE / MEV | 95th percentile CBE / MEV | Within 5°, CBE / MEV | Iyy/max(Ixx, Izz), 5th percentile CBE / MEV |
|------------|----------------------|---------------------------|----------------------|---------------------------------------------|
| 1 mm | 1.6° / 1.8° | 3.7° / 4.1° | 99.4 % / 98.6 % | 1.050 / 1.041 |
| 2 mm | 2.8° / 3.4° | 6.6° / 7.9° | 85.3 % / 77.5 % | 1.045 / 1.036 |
| 3 mm | 4.1° / 4.8° | 10.1° / 11.7° | 67.3 % / 59.8 % | 1.040 / 1.031 |
| 5 mm | 6.9° / 8.3° | 17.1° / 20.9° | 44.5 % / 38.3 % | 1.028 / 1.021 |

(Without the ballast the MEV 5th percentile was 1.032 at 1 mm and 1.027 at 2 mm.) The ratio holds with growth to
σ ≈ 3.1 mm; the 5° axis requirement still needs drawing-controlled positions and a measured flight unit. Provision D-9:
two 5 g trim masses at (0, ±45, +44) mm on the +Z end frame; each changes Iyz by 1.1 × 10⁻⁵ kg·m² (about 3° of tilt),
set from the measured stowed tensor (VI-MCAD-03), with the ballast discs machined to the masses that tensor calls for
(§5.2). Proposed flight constant `INERTIA` (diagonal, mean of CBE and MEV, four AntS elements, ballast, harness zones):
**(0.0031, 0.0045, 0.0043) kg·m²**, to four figures (3.057, 4.468, 4.253) × 10⁻³ with mean products Ixy +1.3 × 10⁻⁵,
Ixz −1.3 × 10⁻⁵, Iyz +0.0 × 10⁻⁵ kg·m² (MASS_BUDGET v3.1 §6.5, for the chair). ADCS Monte Carlo plant rows: CBE
(2.846, 4.190, 3.970) × 10⁻³, ratio 1.055; MEV (3.267, 4.745, 4.535) × 10⁻³, ratio 1.046 (products as in §5.2).

## 6. Envelope, Interference and Clearance Checks

All checks are computed by `model.py` from the solids; `results.json` holds the values.

### 6.1 Envelope and Rails (CDS Rev 14.1)

| CDS item | Requirement | CAD | Status |
|----------|-------------|-----|--------|
| App. B, §2.2.1.1 | 100.0 ± 0.1 × 100.0 ± 0.1 mm rails, 113.5 ± 0.1 mm | 100.0 × 100.0 × 113.5 mm (rail solids' extents) | Met (nominal); vendor drawing and VI-LV-01 |
| §2.2.5 | ≥ 8.5 mm from the rail edge to the first protrusion | 9.0 mm (body panels, wing free edge), 9.4 mm (wing hinge edge), 9.25 mm (hinge springs, OD 4.0 mm) | Met |
| §2.2.8 | Rail ends ≥ 6.5 × 6.5 mm | 8.5 × 8.5 mm solid; 54.9 mm² at the five standoffs bored for a switch or plunger (Ø4.7) against 42.3 mm² | Met |
| §2.2.9 | ≥ 75 % rail contact | 100 % (continuous) | Met |
| §2.2.7 | Edges R ≥ 1 mm | Not modelled (envelope level) | Frame vendor drawing |

### 6.2 Protrusions and the Stowed-Wing Tolerance Stack (STR-10)

From the rail plane (x = 50 mm), W1 (W2 mirrored):

| Layer | Nominal (mm) | Cumulative | Tolerance (±) |
|-------|--------------|------------|---------------|
| Rail plane datum (100.0 ± 0.1) | 0 | 0 | 0.05 |
| Shear-panel outer face, recessed (D-2) | −0.50 | −0.50 | 0.10 |
| Body-panel substrate | 1.60 | 1.10 | 0.16 (±10 %) |
| Body cells, coverglass, adhesive | 0.35 | 1.45 | 0.05 |
| Bumper gap | 1.00 | 2.45 | 0.20 |
| Wing back cells | 0.35 | 2.80 | 0.05 |
| Wing substrate | 1.60 | 4.40 | 0.16 |
| Wing front cells | 0.35 | **4.75** | 0.05 |
| Hold-down line over the post (post 3.65 mm, loop 0.5 mm) | — | **5.25** | post 0.05, line 0.10 |
| Hinge spring, axis x = 54.1, OD 4.0 mm (r = 2.0; knuckles r = 1.75) | — | **6.10** | shear panel 0.10, axis 0.10, spring OD 0.03 (radius) |

| Chain | Nominal | Worst case | RSS | Margin to 6.5 (worst / RSS) |
|-------|---------|------------|-----|------------------------------|
| Wing front cells | 4.75 | 5.57 | 5.08 | +0.93 / +1.42 |
| Hold-down line | 5.25 | 5.55 | 5.41 | +0.95 / +1.09 |
| **Hinge spring (governs)** | **6.10** | **6.38** | **6.25** | **+0.12 / +0.25** |
| Same design without the 0.5 mm recess (D-2) | 6.60 | 6.88 | 6.75 | −0.38 / −0.25 |
| SQSO-SA-001 A §13.2 stack (reference) | 6.1 | 6.87 | 6.40 | −0.37 / +0.10 |

The hinge springs and the line are located from the frame (shear panel), not through the panel stack, so their chains
are short. Version A's governing item was the knuckle (5.85 / 6.13 mm); the Version B springs (OD 4.0 mm, the
largest that fits, §6.8) take 0.25 mm of that margin and now govern, with +0.12 mm worst case. The spring OD is a
drawing-controlled dimension (wound OD ≤ 4.00 mm over the spring's full deflection: the coil diameter grows as the
spring unwinds toward the deployed position, MECHANISMS_ANALYSIS B §3A), checked by VI-MCAD-02. **STR-10 is closed by design**, conditional on the frame RFQ items D-1 and D-2 and the vendor tolerances above
(to be put on the drawings), and verified by VI-STR-08. Other protrusions: RBF head 5.10 mm, access port 1.6 mm (body
panel 1.45 mm). EXOpod Nova allows 25 mm; the 6.5 mm limit is kept for the NRCSD backup and the other dispensers (D7).

### 6.3 ±Z Standoffs (STR-11)

| Item | Outer face | Rail end | Clearance nominal / worst case | Requirement |
|------|------------|----------|--------------------------------|-------------|
| AntS, −Z face recessed 1.5 mm | z = −55.50 | −56.75 | **1.25 / 0.90 mm** (AntS thickness ±0.2, end frame ±0.1, rail length ±0.05) | ≥ 0.1 mm |
| AntS, unrecessed face (reference) | −57.00 | −56.75 | **−0.25 mm (protrudes)** | — |
| GNSS patch on +Z | +55.00 | +56.75 | 1.75 / 1.40 mm (patch thickness ±0.2 assumed) | ≥ 0.1 mm |

**STR-11 closed by design (D-1)**: the frame RFQ must specify a −Z mounting face at least 0.8 mm below the −Z body plane
(1.5 mm modelled), which the ISIS frame is said to offer (STRUCTURAL_DESIGN §2.1) and the Pumpkin "AntS-compatible plates"
may; VI-STR-11 confirms it on the vendor drawing.

### 6.4 Deployment Switches, Separation Plungers, RBF and Access Port

| Item | Rail / face | Position (mm) | Check |
|------|-------------|---------------|-------|
| DS1 | Rail +X+Y, −Z standoff | (+46.0, +46.0), Ø4.5 × 12 | Flush with the standoff end when actuated (CDS §2.3.2.3) |
| DS2 | Rail −X−Y, −Z standoff | (−46.0, −46.0) | Same |
| DS3 | Rail +X−Y, +Z standoff | (+46.0, −46.0) | Same; the EXOpod door set screws and 3 mm front protrusion to be checked with the integrator ICD |
| SP1, SP2 | Rails −X+Y and +X−Y, −Z standoffs | (−46.0, +46.0), (+46.0, −46.0) | Stroke 3 mm (> 2.5), flush when compressed (CDS §2.2.14.2) |
| RBF | −Y face, −Z end zone of the body panel (no cells) | (+20, −50, −43); head Ø8 to 5.10 mm | ≤ 6.5 mm (CDS §2.3.5.2); Nova access window open (LVR-068) |
| Access port (umbilical, charge, GSE): board A3 with a Micro-D 21 (SQSO-HAR-001 2.0 §3.2, M-3) | −Y face, −Z end zone (no cells) | A3 envelope x −35 … −5, y −48.4 … −41.5, z −46.8 … −38; connector face flush with the body-panel substrate (y −51.1) in a 30.4 × 9.2 mm window | Recessed 0.35 mm below the cell plane, inside 6.5 mm; location against the Nova access window open (LVR-068) |

Three switches on three rails give the three independent inhibits of BASELINE §4. The switches (Ø4.5 mm in Ø4.7 mm bores)
sit inside the L-angle corner with 0.25 mm (0.15 mm at the bore) to the rail legs and 0.86 mm to the stack-column spacers: a tight but feasible fit that the
frame vendor's switch provision must confirm (the CAD puts the switch in a plain bore).

### 6.5 Interference

Boolean intersection of every pair of solids whose bounding boxes overlap: none stowed (492 pairs) and none deployed
(486 pairs). The only exclusions are the stowed tapes wound inside the AntS housing (by design), the distributed-mass
allocations (fasteners, staking, thermal materials, rod clips, heater film), which have no geometry of their own, and
pairs of harness items (the zones of §6.11, A3, the tabs and the EPS connector envelope), which meet where wires end
by design; the W10 coax and the W4A assembly are checked against the zones like any solid. The
check was shown to catch planted clashes (magnetometer over the RF shield, knuckle in the body cells).

### 6.6 Wing Swept Volume

The wing (substrate and both cell layers) rotated about its hinge axis in 13 steps from 1° to 90°: minimum distance to the
body panel, its cells, the shear panel and the rails **0.35 mm at 45°** (back-cell edge on the hinge side against the
body-panel coverglass); 0.97 mm at 1°, 0.80 mm at 90°. Positive, but inside the ±0.3 mm the gap tolerance of §6.2 allows;
**action for the panel vendor drawing**: keep the body-panel cell edge ≥ 1 mm from the hinge-side panel edge (cell
block offset 1 mm toward −Y) to raise it to about 1 mm. Deployed: the wing lies in y = 37.1–39.4 mm, x = 52.3–133.9 mm,
0.8 mm outboard of the body panel and clear of the rail.

### 6.7 AntS Tapes against the Wings (STR-16)

The tapes lie in the plane of the −Z face (tape band 5 mm wide centred in the 7 mm housing: z −54.5 … −49.5):

| Configuration | Clearance | Note |
|---------------|-----------|------|
| Stowed wing (92 mm, z ≥ −43), tape along ±X under it | **6.5 mm** vertical (5.5 mm to the housing face) | Contingency only: a wing that did not release when the antennas deploy |
| Stowed wing of the reference design (98 mm centred, z ≥ −49) | 0.5 mm | Why D-3 moved the wing |
| Deployed 2 m tape (along +X, y = 0) to the deployed W1 | **37.8 mm** | Nominal order |
| −Z hinge ear and primary spring (z ≥ −49.0) to the tape band (z ≤ −49.5) | **0.5 mm** vertical; **36.75 mm** laterally from the 2 m tape line (y = 0) | STR-16d: the ear lies outboard of the tape path at y = 36.75–40.75; the sweep is part of the open item below |
| Tape sweep during release, deployed wings | Open: the element root and sweep are in the ISIS ICD (not obtained) | Order the element roots on the −Y half so that the sweep moves away from the wings |

With the baseline order (wings first, BASELINE §4 and §7) the clearance in the nominal sequence is 37.8 mm. The ≥ 10 mm
of MECHANISMS_ANALYSIS §7.2 is **not** met in the wing-stuck contingency (6.5 mm), and cannot be with an 80 mm cell block,
two hinge end zones and a 100 mm face (best achievable about 7 mm). The tape's stiff direction is the vertical one (a
5 mm tape on edge), so 6.5 mm against a static tape is a low risk. **STR-16: closed for the nominal sequence by the
order, verification item for the contingency (VI-STR-10, with the ISIS ICD and the deployment test)**.

### 6.8 Hinges, Springs and the Harness past the Hinges (MCAD-01, closed by design in Version B)

The hinge line is fixed by two limits: anything on the axis must stay within y ≤ 41.5 mm (rail zone) and x ≤ 56.5 mm
(protrusion; 6.5 mm with the tolerance chain of §6.2), and the cell block (80.7 mm) fills the wing width, so the hinge
hardware sits on the axis in the free lengths at the two ends: z −49.0 … −37.3 mm (below the wing and its −Z end zone)
and 43.3 … 49.6 mm (+Z end zone). Version A found that the MECHANISMS_ANALYSIS A springs (Ø0.55 mm wire, mean Ø4 mm,
22 coils, 12 mm long) did not fit and that the flex could not have R ≥ 5 mm (finding MCAD-01). Of the options listed
then (leaf kick-off spring, springs inside the frame, narrower cells, the vendor's hinge), the chosen design is a
smaller version of the original concept that uses the full free length at −Z: **two coaxial torsion springs of OD 4.0
mm on the hinge pin**, the largest radius the envelope allows (MECHANISMS_ANALYSIS B §3A–§5A, computed by `hinge.py`,
values in `hinge_results.json`):

| Item | Design | CAD (z, mm) | Result |
|------|--------|-------------|--------|
| Pin and bushings | A286 Ø1.5 mm pin (non-ferromagnetic; was 440C), sputtered MoS₂ on the pin and burnished MoS₂ in the bore, Ti-6Al-4V bushings; frame ears (Al 6061) bolted through the body-panel notches to the shear panels | Ears −49.0 … −47.6 and 48.6 … 49.6; wing knuckles r 1.75 | Interference-free (§6.5) |
| Primary spring | 17-7PH, Ø0.470 mm wire, 15.0 coils, 7.5 mm long, 2.89 mN·m/rad, preload 55° past deployed | −47.6 … −40.1 | Stress MS +0.12 yield / +0.36 ultimate (1.5 / 1.65) |
| Secondary spring | 17-7PH, Ø0.343 mm wire, 9.5 coils, 3.6 mm long, 1.25 mN·m/rad | 44.8 … 48.4 | MS +0.04 / +0.27 |
| Hinge flex | 2.0 mm two-layer polyimide flex as a 6-turn clock spring in the −Z wing knuckle cup (hub r 1.0, cup r 1.6) | −39.9 … −37.3 | Bend radius 1.0–1.6 mm ≥ IPC-2223 install minimum 0.82 mm; torque allocation 0.50 mN·m cold |
| Torque margin (5017B Eq. 4-1, analysis factors) | Worst case per DDMR 9 at −40 °C and +60 °C | — | Start +0.98 / +1.31; **end of travel cold +0.14**, hot +0.37; one spring out +0.13 (failure tolerant, DDMR 53) |
| End-of-travel stop | Damped silicone bumper (2 N·m/rad, ζ ≥ 0.3) on the −Z frame ear, non-contact deployed sensor beside it | −Z ear | 0.157 N·m, 214 µε at the wing root, rebound 13° (metal stop: 0.62 N·m, 50°) |
| Deployed hold | No latch; spring preload holds the wing on the stop | — | Holding margin +17 (+4.6 secondary alone); a latch pin was rejected (974 MPa shear against 662 MPa; its drag would lower the end-of-travel margin to +0.07) |
| Snubber and hold-down | PTFE-faced silicone snubber at the +Z free corner (0.20 mN·m breakaway); post and line at the −Z free corner | — | Line 5.25 / 5.55 mm (§6.2) |

The margins are thin, not those of Version A (+0.29), and rest on two measured quantities: the flex torque (zero
margin at 0.64 mN·m cold, against the 0.50 allocation and the 0.06 mN·m elastic estimate; VI-STR-04, VI-MECH-02) and
the spring lot's relaxation and allowables (the springs run at 59–64 % of the bending allowable, above the 50 % of
5017B A.2.5.1; VI-MECH-01). The pin friction gives zero margin at μ = 0.26. With one spring only (the primary alone,
no failure tolerance) the end-of-travel margin is −0.15, which is why the design has two. If the hinge test measures a
flex torque above 0.64 mN·m, the levers left are a stronger spring material or shorter cells to free a longer spring
(MECHANISMS_ANALYSIS B §4A, least-bad note). **MCAD-01 is closed by design; STR-13 (flex torque) is closed by design
with VI-STR-04 as its verification.**

### 6.9 Harness Routing

The harness is the harness design's (`hardware/harness/HARNESS_DESIGN.md`, SQSO-HAR-001 2.0): 58 zones in the stack frame
of §4.1 (§6.11), routes and wire list in `routes.csv` and `wire_list.csv`. The four 3 × 10 mm bundles in the ±X wall
gaps (x = ±45.4 … ±48.4) remain as four of those zones, now carrying only the harnesses with an OBC end, the Y-rod pair,
W10E and W7A's burn pairs (≤ 51 % fill in any band); every EPS-only harness reaches the EPS's bottom-side connectors
through the zones under the EPS, the deck openings, the inboard strips beside the −X battery end cap, the hinge-side
strips, the shear-panel slots and the −Y wall strip. Past the hinges: the flex enters the body through the body-panel
notch at the −Z hinge end zone, out of the clock-spring cup in the −Z wing knuckle (z −39.9 … −37.3, tail at
y 37.0–37.5), inside the 6.5 mm envelope (6.10 mm) and outside the rail zone (y ≤ 40.75), and through a 1.5 × 2.8 mm
shear-panel slot to tab TB-W1 / TB-W2 (§6.11). The battery leads (22 AWG, 0.12 m from the sense board) leave the sense
board at (−30, +20), pass the deck opening and coil in zone Z1 to EPS J1/J2: not in the −X bundle.

**W6E (OBC J10 to A2, 0.09 m) and W8A (OBC J9 to A3), OBC design B, SQSO-HAR-001 2.0.** W6E leaves J10 at the OBC's
+X edge (y +24 … +34), runs up the (+X, +Y) bundle to z ≈ +44 and steps inboard over the RF board's 2 mm parts and
outboard of A2 to A2's J1 at its +X edge (zone TE_PXPY); its service loop stays in the bundle below z +20, its supply and
signal wires run as pairs with their returns (no loop area at the coils). W8A leaves J9 at the −X edge (y −24 … −34),
runs down the (−X, −Y) bundle to z ≈ −43, the corner strip and flat along the −Y shear panel outboard of the (−40, −43)
column spacer to A3.

**(+X, −Y) bundle, W4A and the RF board's connectors (ICX item E12; SQSO-RF-BRD-001 C §4, §15 items 14–15).** Amended
from the proposal (bundle at y −17 … −27 over its whole height), which would cross the Y rod's −Y end (y ≥ −23.5 at
z −38.2 … −27.6) and the +X spreader flange (y ≥ −20 at z +29.8 … +32.8): the bundle keeps y −24.0 … −33.9 below
z −26 and runs at **y −20.5 … −27.0 above it** (0.5 mm from the flange), a 3 × 6.5 mm section (66 % of the others: the
harness design moves the excess to the (+X, +Y) bundle). **W4A** (SFSDT discrete-wire flat assembly, 10 mm wide,
1.32 mm thick; SQSO-HAR-001 2.0 M-8) leaves OBC J2 at (+41.0, −33.5) (top side, right-angle TFM-108 per the harness
design's request O-1, plug back at x ≤ +43.1; the CAD keeps the 8 mm allocation x +39 … +43, z +6.6 … +14.6) toward +X
at z +9.7 … +11.0, bends at 4.0 mm radius into the gap, runs flat on the +X shear panel at x +47.0 … +48.3,
y −28.5 … −38.5 (1.5 mm from the bundle; W3 is at |y| < 17 mm) and enters **RF J1** toward −X at z +39.7 … +41.0: RF J1
is a Samtec TFM-108 right-angle header (RF_BOARD_DESIGN D §4.3), pin field along Y at y −39.3 … −27.7, header and plug
within x +30.0 … +43.1 and 7.0 mm high (z +36.4 … +43.4), in the shield's corner notch now x > +30.0, y < −26.0
(from x > +37.0). No fold over J1. **RF J2** (Pico-Clasp 6 at (−26, −44),
mated allocation 6 mm) sits on the −Y strip outside the shield; W4B leaves it upward, over the shield's (−X, −Y) corner
(5 mm between the shield top and the +Z end frame) to the (−X, −Y) bundle. **J3–J6** (MMCX, Molex 73415-1473, RF C §4.3)
at (−34.5, ±19) and (−34.5, ±25) in envelopes x −45.1 … −31.5, |y| 16.5 … 27.5, z +36.4 … +42.4 (a second shield notch
x < −30.0, y −28 … −16 for J3/J4); the **W10 coax** (four 1.37 mm cables, right-angle plugs, 7 mm bend radius) bends over
the −X board edge into the −X wall gap at y +20 … +24 (70 cm pair) and y −20 … −24 (2 m pair), runs down to the −Z end
frame, inboard 0.6 mm under the battery end caps and through the end frame's cut-out to the AntS ports; the two −X
bands are kept free of harness (the −X bundles stay at |y| = 24 … 33.9 and touch them only at y ±24). All of these are
in the CAD (`ribbon_W4A`, `OBC_J2_mated`, `RF_J1_mated`, `RF_J2_mated`, `RF_J3_J6_MMCX`, `coax_W10`): no interference
(492 / 486 pairs). Mated heights stay allocations until the Samtec, Molex and MMCX drawings are checked (VI-MCAD-01).

### 6.10 Jetson Heat Path

Geometry: SoC lid → 1.0 mm gap pad → spreader pedestal (§4.5; the carrier design's junction-to-spreader estimate is
1.09–1.16 K/W against the 5.7 K/W THM-10 allowable, verified by V-JET-7) → spreader (5 mm Al, 60 mm wide, ≈ 40 mm to each flange
root: 1.25 W/K per side) → flange (3 × 40 × 8.5 mm: 2.36 W/K per side) → bolted joint to the 1.0 mm shear panel. With bare
Al–Al contact and 2 × M2.5 per side (h ≈ 2,000 W/m²·K over 40 × 8.5 mm) the path to the frame is **0.66 W/K**; with a
thermal gasket and 3 × M2.5 per side (h ≈ 10,000) **1.10 W/K**, which meets the ≥ 1 W/K of BASELINE §6. Estimate only
(contact conductances assumed); the EM thermal balance test measures it (TB-07). The design calls out the gasket.

### 6.11 Harness Zones, Slots, Access Port, Tabs and the ±Z Sun Sensors (SQSO-HAR-001 2.0 §15 M-1 to M-9)

| Item | In the CAD (`model.py`) | Check |
|------|-------------------------|-------|
| M-1 Harness zones | 58 zones of `harness_calcs.py` (`ZONES`): 50 as `hz_*` parts carrying their routed mass, the six coax zones as `coax_W10`, W4AZ as `ribbon_W4A`, AP_A3 as `access_port`; **EPS bottom-side connector envelope** `EPS_bottom_connectors`: zones Z1–Z5 (the plan areas of U_MXPY, U_MXMY/2, U_PXPY, U_PXMY/2, U_PY, U_PYN) and J3 (x +33 … +45, \|y\| < 15.5), z −4.05 … −12.0 below the 1 mm parts layer (9.0 mm below the substrate) | Every zone box against every solid (`harness_calcs.cad_check`, run read-only against this CAD): no interference in any of the 58 zones; nearest solids 0.05–0.1 mm (the shear panels, EPS parts, column spacers, hinge doublers) |
| M-2 Shear-panel slots | Four 6 × 4 mm slots at z −12 … −8: +X at y −33 … −27, −X at y −15 … −9, +Y and −Y at x −33 … −27; 0.75 mm silicone or PTFE grommets (`slot_grommets`, mass in the harness ties) | PG zones 0.1 mm from the panels; frame RFQ item 15 |
| M-3 Access port | `access_port` (group harness, 6.0 g of row 32 with W8A's and W11's ends): A3 envelope x −35 … −5, y −48.4 … −41.5, z −46.8 … −38; Micro-D 21 shell x −31 … −9, z −45.4 … −39.4 through a 22.4 × 6.4 mm shear-panel window and the 30.4 × 9.2 mm −Y body-panel window (z −47.0 … −37.8; the cells start at −37.1) | A3 0.1 mm from the −Z end frame and the −Y shear panel; 1.5 mm from the battery clamp bar; RBF at x +20 unchanged |
| M-4 Tabs | `TB_tabs`: polyimide 6 × 10 mm at x ±(48.2 … 48.5), y +32 … +42, z −37 … −31 on the shear panels' inner faces; hinge-flex tail through a 1.5 × 2.8 mm slot at y +36.5 … +38.0, z −40.0 … −37.2 | Tab overlaps the hinge-side strip and 1.9 mm of the corner bundle's band A (5 % full) by design |
| M-5 Sun sensors | **PZ** (`sun_sensor_PZ`): photodiode on a 6 × 6 × 0.8 mm carrier at (−36, +36) on the +Z end frame's outer face (top 1.5 mm above it, below the GNSS patch's 5 mm), leads through a 2 mm hole to zone TE_MXPY. **MZ** (`sun_sensor_MZ`): the −Z face is covered by the AntS (98 × 98 mm, the corners taken by the rail standoffs), so the −Z photodiode stands outboard of it: on an L-carrier bonded to the −X body panel's −Z end zone at y +28 (leg x −51.1 … −51.9, z −49.3 … −45.5; shelf to x −54.1), the diode facing −Z at (−52.6, +28, −50.0), 4.1 mm beyond the rail plane, leads through 1.8 mm holes in the body and shear panels at (y +28, z −45.5) to the MZT termination (about 17 mm of the 30 mm pigtail) | **MZ field of view:** the full cosine response about −Z (±60° to 50 % response) toward −X and ±Y; on the +X side the AntS housing (edge x −49, bottom z −55.5) cuts it at 33° from the boresight; the (−X, +Y) −Z rail standoff clips the +Y side beyond 64°. It sees −Z directly. The ADCS estimator uses its true normal (−Z) and this mask. The stowed wing W2 (z ≥ −43) and the deployed wings (above z −43) are behind its plane; the −X 2 m tape exits at the side centre in the CAD (y 0), 27 mm away; its band (z −54.5 … −49.5) overlaps the diode's lowest 0.5 mm in z, so the element exit position against the sensor is open with the AntS ICD (VI-MCAD-01, VI-MCAD-09) |
| M-6 Mass distribution | §5.1 | §5 |
| M-7 Deck openings | DX_MXPY, DX_MXMY, DX_MXC and the −Y slot (DS_MY_M, DS_MY_P) stay open in the skeleton deck as pass-throughs; their edges carry PTFE edge protection (harness ties) | Zones clear of the deck (0.1 mm or more) |
| M-8 W4A | §6.9 | — |
| M-9 Isolation | AntS housing and GNSS patch (and its adapter) fastened through polyimide or G-10 washers and sleeves, faying faces isolated, ≥ 1 MΩ to the frame at 50 V, as A2 (D-13) | VI-MCAD-09 |

## 7. Drawings and STEP

Sheet 1 (`drawing_stowed_3view.svg`) and sheet 2 (`drawing_deployed.svg`) are hidden-line projections of the CAD solids
(OpenCascade HLR) with dimensions drawn from the model parameters, so they regenerate with the model. They are layout
drawings for review, not manufacturing drawings: GD&T, the fastener call-outs, the vent-area summary
(STRUCTURAL_DESIGN §9) and the detail drawings of the brackets, deck, spreader and battery clamp remain to be made in a
drawing tool at drawing release.

## 8. Structural Finite-Element Model

### 8.1 Idealisation and Verification

`fem.py` builds a beam-shell model from the CAD parameters and solves it with numpy/scipy (no commercial solver):

| Element | Items | Properties |
|---------|-------|------------|
| Flat 4-node shell: MITC4 plate (assumed transverse shear, no locking), Wilson incompatible-mode membrane, drilling stiffness 10⁻⁴ E t A | −Z and +Z end frames (Al 1.6 mm, cut-outs 71 × 71 and 30 × 30 mm), four side shear panels (Al 1.0 mm), EPS, OBC and RF boards (FR-4 1.6 mm, E 18.6 GPa) and the carrier (FR-4 2.0 mm with the 27 × 27 mm gyro window of D-11), torquer deck (Al 1.0 mm skeleton, 9 g with clips: equivalent plate with E × 0.35), heat spreader (Al 5 mm) and flanges (3 mm), stowed wings (FR-4 1.6 mm) | Element mass plus non-structural mass for the components, cells, body panels, AntS, harness and distributed items |
| 3D Euler–Bernoulli beam, consistent mass | Rails (L 8.5 × 8.5 × 1.5 mm between the end frames, solid 8.5 × 8.5 mm standoffs); stack columns and centre supports (Al hex 5 AF + M3 A286 stud; Ti-6Al-4V at the (+40, +43) column next to the magnetometer, bending × 1/3 for the threaded joints as SQSO-SA-001 A §5); battery isolators (G-10CR Ø6/Ø2.7 mm bushing with the Ti-6Al-4V M2.5 screw in bending; axial and bending × 0.6 for joint compliance, scaled to the 8 mm bushing); clamp ears (Al 10 × 2.5 mm); the carrier's window frame (Al 2.5 × 3.0 mm on the bottom face, offset from the board mid-plane by a composite section with a 20 t strip of the board) | — |
| Translational springs | Wing hinges (pins: no moment; 2 × 10⁸ N/m normal, 5 × 10⁶ in plane), hold-down cup/cone post (2 × 10⁸ / 2 × 10⁶), snubber (silicone, 2.2 × 10⁴ N/m) | — |
| Rigid links | Shear-panel edges and end-frame corner squares to the rails; the battery (261 g rigid body, inertia from the CAD solids) to its ears; **spiders** spreading each column, centre-support and isolator attachment over the nodes within 2.6 mm (the spacer and washer face; 224 nodes) | — |

Boundary condition, as in the dispenser: every rail node fixed in x and y (the guide rails), the rail-end nodes fixed in z
(pusher plate and door), rotations free. Sensitivity: the eight rail ends fixed only (§8.6). Mesh: 10,860 nodes, 10,066
shells, 192 beams, 60,144 degrees of freedom; FE mass 1,513.9 g, equal to the CAD and MEL CBE (the inertia ballast, the
W10 coax, W4A, the hinge springs, the harness zones, tabs and sun sensors lumped where the CAD puts them: each harness
zone at three points along its run on what it is tied to, the shear panels in the wall gaps, the end frames, the boards it
crosses; under the EPS the connector halves (6.8 g) on the EPS and the wire (21 g, with 3.8 g of row 31's allocation) on
the deck straps, as the harness design's tie-down rules put it, SQSO-HAR-001 2.0 §10.3). MEV masses are not run (+15.5 % mass
lowers the frequencies by about 7 %; the responses scale with the mass at the same acceleration).

Element verification (`fem.py --selftest-only`), same element code and mesh density:

| Case | FE | Closed form | Error |
|------|----|-------------|-------|
| Simply supported square FR-4 plate 96 × 96 × 1.6 mm, mode (1,1) | 494.1 Hz | 496.8 Hz | −0.55 % |
| Corner-supported square plate (Leissa, NASA SP-160: λ² = 7.12) | 177.4 Hz | 179.2 Hz | −0.99 % |
| Cantilever strip 100 × 10 × 1.6 mm (plate) | 80.94 Hz | 81.55 Hz | −0.75 % |
| In-plane cantilever 100 × 20 × 2 mm, 10 × 2 elements (membrane bending) | 3.691 µm | 3.744 µm (Timoshenko) | −1.42 % |
| Beam cantilever Ø5 × 100 mm Al | 353.35 Hz | 353.31 Hz | +0.01 % |

### 8.2 Modes and Effective Mass (design case: D-7, rails held)

| Mode | f (Hz) | Effective mass x / y / z (% of 1,474 / 1,474 / 1,503 g free) | Dominant (share of kinetic energy) |
|------|--------|---------------------------------------------------------------|------------------------------------|
| 1 | 169.4 | 0 / 0 / 1.9 | Torquer deck with the 21 g of harness wire dressed on it (X rod, GNSS receiver, Z-rod clip) |
| 2, 3 | 170.2 | 0 / 0 / 0; 5.4 / 0 / 0 | Stowed wings W1, W2, out of plane (in and out of phase) |
| 4 | 264 | 0 / 0 / 1.5 | Deck, second mode |
| 5 | 318.0 | **34.0** / 0 / 0 | EPS, OBC, carrier: stack lateral in X |
| 6, 7 | 341.4 | 0 / 0 / 0 | Wings, second mode |
| 9 | 354.3 | 0 / 0.3 / **19.2** | Carrier, EPS, deck: stack drumming in Z |
| 10 | 383.6 | 0.1 / **21.6** / 0 | RF, EPS, deck: stack lateral in Y |
| 11 | 384.5 | 0 / 0.9 / 0.1 | RF board |
| 12 | 387.0 | 0.1 / 0.2 / 5.5 | +Z end frame with the 20 g inertia ballast (546 Hz without it) |
| 19 | 426.5 | — | EPS board (446 Hz before the harness distribution) |
| 25 | 479.8 | — | Carrier board (2.0 mm) |
| — | 484, 536, 574, 623 | small | Spreader; ±X shear panels (with the wall-gap harness); OBC; ±Y shear panels |
| — | 983, 1,096 | 0.3 / 1.3 / 4.5; 0.1 / 6.9 / 2.5 | −Z end frame and the battery on its isolators (battery-dominated from 1,096 Hz) |
| — | 1,285 | **18.5** / 0 / 0.1 | Battery on its isolators, lateral |
| — | 1,517 | — | −Z end frame with the AntS |

450 modes, to 3,689 Hz. Cumulative effective mass to 2,000 Hz (modes kept in the results file): 75 % (x), 58 % (y), 68 % (z) of the free mass; the rest is
the frame and panel mass next to the held rails, which moves with the base. First mode 169 Hz (the deck with its harness
dress; the wings 170 Hz): **4.2 times the 40 Hz requirement**; no mode below 150 Hz, so the spacecraft is rigid in the launch-vehicle coupled-loads band (UG Appendix B).

### 8.3 Against the Analytic Values of SQSO-SA-001 A §5

| Element | SA-001 A (closed form) | FE | Comment |
|---------|------------------------|----|---------|
| Stowed wing | 142 Hz (four points: two hinges, hold-down, bumper) | 170 Hz (three points and a soft snubber; 183 Hz with a second hard post) | Shorter wing (92 mm) and the hold-down at the corner |
| PCB stack, lateral, tied at both ends | 289 Hz | 318 Hz (X), 384 Hz (Y) | The spreader bolted to the walls at mid-height stiffens the stack; Y stiffer through the flanges |
| Boards with a centre support | 221 (RF) – 362 (OBC) Hz | RF 385, EPS 427, carrier 480 (2.0 mm), OBC 574 Hz (in the stack) | Two centre supports (D-7); boards coupled |
| Battery on G-10CR isolators, lateral | 1,280 Hz | 1,285 Hz | Agreement |
| Deck (new part) | — | 169 Hz with the harness dress (195 Hz without) | Lowest mode |

### 8.4 Loads and Criteria

- **Quasi-static:** UG v11 Table 4-1, 10 g axial combined with 17 g lateral in eight directions (every 45°), both signs:
  16 cases, envelope.
- **Random:** the protoflight test level of ETP Rev C §3.1 (envelope of MPE + 3 dB and the GEVS-B workmanship spectrum,
  **8.18 Grms**; it exceeds MPE + 3 dB, 7.86 Grms, between 80 and 500 Hz), each axis, modal superposition over the 450
  modes with Q = max(10, √f) (Steinberg's Q for the boards, 10 for the structure), in the exact modal-covariance form;
  3σ.
- **Design limit stress** at each element: the larger of the quasi-static envelope and the 3σ random value. The random
  value governs at every hot spot. Shells: von Mises at the surfaces (component RMS values combined with the adverse sign);
  beams: |N|/A + |My|/Wy + |Mz|/Wz.
- **Factors (NASA-STD-5001B protoflight):** metals 1.25 yield / 1.40 ultimate; FR-4 1.50 ultimate; G-10CR isolators at a
  discontinuity 2.00 ultimate. MS = allowable / (FS × limit stress) − 1.
- **Steinberg:** 3σ out-of-plane displacement of each board node relative to the mean of its supports (columns and
  centre supports), Z excitation, against Z_allow = 0.00022 B / (c h r √L) = 0.192 mm (B = 3.78 in, h = 1.6 mm, c = 1.75
  BGA, r = 1, L = 1 in), the SQSO-SA-001 A §8 criterion. The allowable scales with 1/h: 0.154 mm for a 2.0 mm board
  (the first issue of Version B used 0.192 mm for the 2.0 mm boards too, which overstated the reserve option:
  corrected below).

### 8.5 Margins of Safety (design case)

| Location | QS (MPa) | 3σ random (MPa) | Allowable | FS | MS yield | MS ultimate | Where |
|----------|----------|-----------------|-----------|----|----------|-------------|-------|
| −Z end frame at the battery isolators | 8.1 | **96.2** | 241 / 290 | 1.25 / 1.40 | **+1.00** | +1.15 | (+40.8, +32.0, −47.7): next to the isolator spider |
| End frames at the rail corners | 24.1 | 88.8 | 241 / 290 | 1.25 / 1.40 | +1.17 | +1.33 | −Z frame at (−40.8, −41.2) |
| Centre supports | 31.9 | 78.7 | 241 / 290 | 1.25 / 1.40 | **+1.45** | +1.63 | (0, +21) under the spreader |
| Side shear panels | 17.3 | 66.9 | 241 / 290 | 1.25 / 1.40 | +1.88 | +2.10 | −X panel at the spreader flange |
| Rails (L-angle, standoffs) | 12.9 | 51.3 | 241 / 290 | 1.25 / 1.40 | +2.76 | +3.04 | (−47.3, +47.3) at the −Z end frame |
| Stack columns, Al spacer | 17.7 | 56.6 | 241 / 290 | 1.25 / 1.40 | +2.40 | +2.66 | (−40, −43) under the +Z end frame |
| Stack columns, A286 stud thread root | 21.3 | 68.3 | 758 / 1,103 | 1.25 / 1.40 | +7.9 | +10.5 | Same |
| Stack column (+40, +43), Ti-6Al-4V stud thread root | 8.3 | 23.5 | 827 / 896 | 1.25 / 1.40 | +28 | +27 | (+40, +43) under the +Z end frame |
| Battery isolator, G-10CR bushing | 6.8 | 35.8 | 380 (flexural) | 2.00 | — | **+4.31** | (−38.5, +31) |
| Battery isolator, Ti-6Al-4V screw | 15.1 | 80.0 | 827 / 896 | 1.25 / 1.40 | +7.3 | +7.0 | Same |
| Battery clamp ears (Al) | 5.0 | 23.4 | 241 / 290 | 1.25 / 1.40 | +7.2 | +7.9 | Ear root |
| Heat spreader and flanges (with the pedestal's 1.8 g) | 8.8 | 26.3 | 241 / 290 | 1.25 / 1.40 | +6.3 | +6.9 | At the upper centre support |
| Torquer deck (equivalent plate, with the harness dress) | 9.7 | 40.5 | 241 / 290 | 1.25 / 1.40 | +3.8 | +4.1 | (−37.0, −39.8), −Y rim |
| PCBs EPS / OBC / carrier / RF (FR-4) | 3.5 / 3.9 / 5.8 / 4.2 | 29.8 / 21.5 / 26.7 / 20.1 | 340 | 1.50 | — | +6.6 / +9.6 / +7.5 / +10.3 | At the columns |
| Stowed wings (FR-4) | 1.7 | 6.4 | 340 | 1.50 | — | +34 | Centre |

| Board | Z₃σ relative (mm) | Steinberg MS, **design** (two centre supports, gyro window in a 2.0 mm carrier) | 1.6 mm carrier, framed window (withdrawn) | 1.6 mm carrier, window, no frame | 1.6 mm carrier, no window (reference) | One centre support | Design with a 2.0 mm EPS board too |
|-------|------|------|------|------|------|------|------|
| EPS | 0.156 | **+0.23** | +0.14 | +0.16 | +0.15 | −0.30 | +0.50 |
| OBC | 0.140 | **+0.37** | +0.14 | +0.16 | +0.15 | +1.24 | +0.63 |
| Jetson carrier (with the module) | 0.142 (2.0 mm) | **+0.08** | +0.08 | **−0.03** | +0.08 | −0.04 | +0.42 |
| RF | 0.142 | **+0.35** | +0.28 | +0.29 | +0.28 | +0.28 | +0.50 |

(FE with the 20 g inertia ballast, chair decision C-13 (the +Z end frame's local mode 387 Hz, from 546; each disc's two
M2 Ti-6Al-4V screws carry under 20 N even at 200 g on the disc, against about 1.6 kN of tensile strength each), and the
harness design's distribution, SQSO-HAR-001 2.0 M-6. **The EPS margin depends on where the harness under it is tied.**
With the connector halves (6.8 g) on the EPS and the wire (21 g) on the deck straps, as above: EPS +0.23 (the maximum at
the Z-rod notch corner, (−35.5, −3.3)), and the deck, which then carries 21 g on 7.8 g of aluminium, drops to 169 Hz and
moves 0.46 mm (3σ) relative to its supports (0.36 mm before). With all 28 g tied to the EPS (the bounding case, run):
EPS −0.52 at the notch's +Y lip (−45.1, +6.5), the EPS mode 315 Hz; with a 2.0 mm EPS board in that case +0.39. Design
rule taken here (for the harness design, §11 item 15): under the EPS only the connector strain-relief ties (within 15 mm
of each housing) are on the EPS; the wire dress is tied to the deck straps and the shear panels.)

The carrier's allowable is the 2.0 mm one (0.154 mm). The window in a 1.6 mm board costs the carrier 0.12 (to −0.05: the
window cuts the board between the corner column (−40, +43) and the centre support (0, +21)); the 2.0 mm board restores
+0.08 and raises the OBC margin (+0.37) through the coupled stack. The framed 1.6 mm alternative gives
+0.08 but depends on a bond that does not close (§4.3.3).

Hold-down: the post carries 2.0 N quasi-static and **3.9 N 3σ** normal to the wing (SA-001 A §10: 4.9 N); against the
30 N line preload after a 30 % loss (21 N) with 1.15 × 1.20 on the load, separation MS = 21 / (1.38 × 3.9) − 1 = **+2.9**.
Hinges 3.9 N, snubber 3.7 N (3σ).

Findings from the FE:

1. **All strength margins are positive** at the protoflight test level with the 5001B factors. The lowest, +1.00, is a
   local peak in the 1.6 mm −Z end frame beside the isolator attachment (battery 261 g on four points of a ring 13.5 mm
   wide); it is mesh- and spider-dependent. **Design action:** a 1 mm Al doubler (or a 2.5 mm local boss) under each
   isolator on the −Z end frame, specified on the frame drawing or the battery-mount drawing.
2. **The PCB fatigue margins need the second centre support (D-7):** with the single support of SQSO-SA-001 A at the only
   place the Jetson module leaves free (y = +21 mm), the EPS and carrier boards are at −0.30 and −0.04 (2.0 mm carrier);
   with the second support at y = −21 mm (EPS → OBC → carrier) they are +0.23 and +0.08. A 2.0 mm EPS board as well would
   raise the EPS to +0.50 (with the 2.0 mm allowable; +0.8 to +0.9 in the first issue of Version B, which used the 1.6 mm
   allowable) at 0.4 mm more of stack height (available above the RF board, at a re-stack). Steinberg's 2 × 10⁷-cycle criterion is
   conservative for one minute per axis. **Chair decision (2026-10-27): two centre supports, boards kept at 1.6 mm**,
   since all four boards are positive; the 2.0 mm option is held in reserve until the measured Q of the EM board survey
   (VI-STR-02) is known.
3. **The deck** (a part this design adds, now a 1.0 mm skeleton) is the lowest mode (169 Hz with the 21 g of harness dress
   tied to it; 195 Hz without) and moves 0.46 mm (3σ) relative to its supports; its strength margin is +3.8 (equivalent plate; local stresses in the skeleton
   straps are higher and are checked at drawing release). Its payload (GNSS receiver, X rod) is not a PCB on the deck, but the receiver's
   own board sees the deck motion: if the receiver vendor's random qualification is below the deck response, stiffen the
   deck (1.6 mm with ribs, about +10 g) or move the receiver so that the second centre support can start at the deck
   (the deck-to-carrier variant gave the deck 206 Hz). The deck must not be tied to the battery clamp, which is
   isolated.

### 8.6 Sensitivity Cases

| Case | First mode | Lowest MS (where) | Steinberg range | Note |
|------|------------|-------------------|-----------------|------|
| Design (D-7, D-11 with the 2.0 mm carrier, M1, Ti stud, inertia ballast, harness zones, rails held) | 169 Hz (deck) | +1.00 (−Z frame at the isolators) | +0.08 … +0.37 | Baseline |
| 1.6 mm carrier, framed window (first issue of D-11) | 169 Hz | +1.00 | +0.08 … +0.28 | Withdrawn: the bond does not close (§4.3.3) |
| 1.6 mm carrier, window, no frame | 169 Hz | +1.00 | **−0.03** … +0.29 | Rejected |
| 1.6 mm carrier, no window (Version B before D-11) | 169 Hz | +1.00 | +0.08 … +0.28 | Reference |
| Single centre support | 169 Hz | +1.00 | **−0.30 … +1.24** | Rejected (D-7) |
| Design with a 2.0 mm EPS board | 169 Hz | +1.00 | +0.42 … +0.63 | Reserve option |
| Design with a second hard post at the +Z free corner | 169 Hz (deck; wing 183 Hz) | +1.00 | +0.08 … +0.37 | Wing +13 Hz; not needed |
| Design, all harness under the EPS tied to the EPS (bounding case) | 170 Hz | +1.00 | **−0.52** (EPS) … +0.69 | Rejected tie-down (§8.5) |
| Design, rails held at the eight rail ends only | 169 Hz | **+0.84** (rails, 105 MPa), +0.93 (−Z frame), +1.24 (centre support) | +0.08 … +0.37 | Bounds the dispenser contact; rails then carry the lateral load in bending |

### 8.7 Limits of the FE Model

- Beam-shell idealisation: joints are rigid links and spiders, the threaded-joint and isolator knock-downs are
  SQSO-SA-001 A's assumptions; local stresses at fasteners and inserts (bearing, pull-out) are not resolved and stay in
  the NASA-STD-5020B joint analysis of SA-001 §9.
- The frame is an allocation (wall thicknesses and the shear panels are not a vendor's design); the vendor's frame
  qualification (VI-STR-11) still governs the frame itself.
- Components are non-structural mass on the boards (no stiffening by the RF shield or the Jetson module: conservative for
  the boards); the AntS is mass on the −Z end frame (its housing would stiffen it).
- Q values are assumed (Q = max(10, √f)); no test data. The EM sine survey (VI-MCAD-04, VI-STR-02) correlates the model.
- No coupled model with the dispenser (EXOpod Nova §3.6); the base input is the test spectrum at the rails.
- The model is not in NASTRAN format; §9 states what the integrator can be given.

## 9. The Integrator's L-8 M CAD and Mass-Properties Deliverables (LVR-004, LVR-005, LVR §9)

| L-8 M item (UG v11 Table G-3; SQSO-LVR-001 §9, 2028-03-01) | What this model provides | What remains |
|-------------------------------------------------------------|--------------------------|--------------|
| CAD model (UG Appendix C: STEP AP214 or lower, simplified to the outer mould line and interfaces, no internal parts, same coordinate system and units as the mass properties) | `smartqso_oml_stowed.step` and `smartqso_oml_deployed.step`: rails, end frames, panels, wings, hinges, AntS (with the deployed tapes), GNSS patch, switches, plungers, RBF and access port, in the CDS frame, mm. For a CubeSat in an EXOpod Nova the integrator uses it for the virtual fit check (Nova §2.3.1) and passes its own dispenser model to SpaceX | Replace the allocations with the vendors' models (frame, AntS, panels, wings); put in the RBF and access-port locations agreed with Exolaunch (LVR-068); re-export from the drawing-release model |
| Mass properties (LVR-005: ±15 % at L-8 M) | Mass 1.49 kg CBE / 1.73 kg MEV, CG and full inertia tensor stowed and deployed, in the CAD frame (§5.2; `results.json`). The value to report remains **1.73 kg ± 15 %** (MASS_BUDGET §3.3) | Weighed components replace the CBE (MGA → 0); measured mass, CG and inertia at L-3.5 M (±5 %) and L-2 M |
| Dynamic model (UG Appendix B: Craig–Bampton in NASTRAN .op4, content to 150 Hz) | The FE of §8: first mode 170 Hz, no mode below 150 Hz, so in the launch-vehicle band the spacecraft is a rigid mass (the mass properties above). Appendix B results are not guaranteed for payloads (UG: environments are enveloping) | If the integrator asks for a model: a NASTRAN bulk-data export of the `fem.py` mesh (nodes, shells, beams, masses) and a Craig–Bampton reduction in NASTRAN; not produced (no NASTRAN licence in the project) |
| Environmental test approach | §8 analyses the protoflight random level of ETP Rev C (8.18 Grms) | — |

## 10. Review Items

| RID | Status | Evidence |
|-----|--------|----------|
| SE-01 (mechanical part) | **Mechanical build-to design at envelope level: done.** Parametric CAD of every MEL item, interference-free, with STEP and layout drawings; the remaining steps to a manufacturing package are listed in §7 and §12 | This document; `model.py`; STEP; SVG |
| SE-12 | **Structural model: closed by analysis.** Verified beam-shell FE model (§8.1), modes with effective mass, quasi-static and random stresses with NASA-STD-5001B factors, Steinberg margins | §8; `fem.py`; SQSO-SA-001 B §5A |
| STR-01 (check) | **Confirmed and closed by design**: CAD gives Iyy/max 1.055 / 1.046 with the four AntS elements, the 2 × 10 g Z-axis ballast (chair decision C-13; 1.045 / 1.037 without it) and the harness design's mass distribution, and the major axis within 1.0° / 1.0° of +Y (the reference layout gave 13–15°, outside the 5° baseline); placement sensitivity and the trim provision are verification items (VI-MCAD-03) | §5 |
| STR-10 | **Closed by design** (Version B): hinge springs 6.10 mm nominal, 6.38 mm worst case, 6.25 mm RSS against 6.5 (D-2 recess, frame-located springs and post); spring OD ≤ 4.00 mm on the drawing; VI-MCAD-02 | §6.2 |
| STR-11 | **Closed by design** (frame RFQ: −Z face recessed ≥ 0.8 mm; 1.5 mm modelled): 1.25 mm nominal, 0.90 mm worst case | §6.3 |
| STR-16 | **Closed for the nominal sequence** (wings first: 37.8 mm); **verification item** for the wing-stuck contingency (6.5 mm against 10 mm) and the ISIS sweep | §6.7 |
| MCAD-01 (severity 2) | **Closed by design (Version B)**: two OD 4.0 mm torsion springs on the hinge pin and a clock-spring flex in the −Z knuckle fit the envelope; torque margin (5017B, analysis factors) end of travel cold **+0.14**, one spring out +0.13; spring MS +0.12 / +0.04 yield; damped stop 0.157 N·m; no latch (holding margin +17). Thin margins: verification by VI-STR-04, VI-MECH-01 and VI-MECH-02 | §6.8; SQSO-MECH-001 B §3A–§5A; `hinge.py` |
| STR-13 (reopened in Version A) | **Closed by design** with the flex at the bend radius that fits (1.0–1.6 mm, allocation 0.50 mN·m cold; zero margin at 0.64); **verification item** VI-STR-04 (measured flex and hinge torque at −40 °C) | §6.8; SQSO-MECH-001 B §3A, §4A |
| MCAD-02 (severity 3) | **Closed by reallocation (Version B)**: skeleton torquer deck 9 g (7.8 g Al) inside a re-split row 5 (deck 9, heat-path 15, doublers 4, patch bracket 2 = 30 g; MEL unchanged). The alternative (growing row 5 by the original 17 g) gives MEV 1,729.7 g and margin +15.6 % | §5.1, MASS_BUDGET v3.1 §1.2 |
| Centre supports (chair decision) | **Adopted**: two centre supports at (0, ±21) mm on the EPS, OBC and carrier boards, EPS, OBC and RF boards 1.6 mm, the carrier 2.0 mm for its gyro window (D-11); Steinberg margins +0.08 (carrier) to +0.37 (with the inertia ballast and the harness distribution; EPS +0.23 with the harness wire tied to the deck); a 2.0 mm EPS board held in reserve (+0.42 to +0.63 with the 2.0 mm allowable; the first issue's +0.83 to +0.90 for 2.0 mm boards used the 1.6 mm allowable) | §4.1, §8; SQSO-SA-001 B §8A |
| OBC item C3 (gyro stack clearance) | **Closed by design**: gyro on its socket on the OBC, through a 27 × 27 mm window in the Jetson carrier, which is 2.0 mm thick (D-11; the bonded frame of the first issue is withdrawn, §4.3.3); the two proposed options (carrier raised 3 mm; bracket and ribbon) do not fit (§4.3). Gyro alignment to be measured and compensated (VI-MCAD-05) | §4.3; check OBC-C3 |
| MCAD-03 (new, severity 3) | Wing sweep clearance 0.35 mm: body-panel cell layout action for the panel vendor | §6.6 |

## 11. Interface Changes for Other Documents (for the chair)

1. **Frame RFQ (BUILD_VS_BUY §2.2, STRUCTURAL_DESIGN §2.1):** −Z mounting face recessed ≥ 0.8 mm (1.5 mm modelled) for
   the AntS; side shear-panel outer faces ≥ 0.5 mm below the rail plane; switch provisions (Ø4.5 mm, 12 mm deep) at the
   five standoff positions of §6.4; inserts for the spreader flanges at mid-height of the ±X walls.
2. **Wing outline (SOLAR_ARRAY_DESIGN, panel vendor):** 81.6 × 92 mm, z −43 … +49 mm, cell block 80.7 × 80.15 mm centred
   at z = +3 mm, end zones for the hinges, the hold-down post and the snubber; body-panel cells offset 1 mm toward −Y.
3. **Mechanisms (MECHANISMS_ANALYSIS, now Version B):** done here: the hinge, springs, flex, stop and hold of §6.8;
   hinge axis (±54.1, +38.75) mm; the hold-down at the −Z free corner with a cup/cone post and a PTFE-faced snubber at
   the +Z free corner (§8.4 gives the line load).
3a. **Panel vendor and spring vendor (from Version B):** the wing's −Z end zone must take the primary spring leg and the
   clock-spring flex cup (OD 4.0 mm, z −39.9 … −37.3) and the +Z end zone the secondary spring leg; the springs are
   17-7PH CH900 to the wire, coil and preload values of §6.8, wound OD ≤ 4.00 mm, with the vendor's bending allowable
   (≥ 1,500 MPa assumed) and relaxation data (VI-MECH-01); pins MoS₂-coated (μ ≤ 0.15 at −40 °C assumed); the stop is a
   damped silicone bumper (ζ ≥ 0.3).
4. **RF board (RF designer):** shield 84 × 54 × 7 mm at y −40 … +14 mm; parts ≤ 2 mm elsewhere on the top side
   (magnetometer above at (+24, +27)); bottom-side keep-outs Ø7 mm at the columns and at (0, 0).
5. **EPS, OBC and carrier boards:** PC/104 outline 90.17 × 95.89 mm; holes at (±40, ±43); centre supports at (0, +21)
   and (0, −21) (Ø7 mm keep-outs both sides); EPS notch x < −35.5, |y| < 6.5 mm for the Z rod; OBC: M-G370PDT0 at
   (−20, +30.5), through the carrier window (item 9); carrier: SO-DIMM on the −Y edge, module centred at y = −6,
   no top-side parts within the module footprint.
6. **ADCS:** rod positions and the flight inertia constant (§5.4: (0.0031, 0.0045, 0.0043) kg·m² with the ballast and
   the harness zones; the ADCS Monte Carlo plant rows CBE (2.846, 4.190, 3.970) × 10⁻³ and MEV (3.267, 4.745, 4.535) ×
   10⁻³ kg·m²); the ±Z sun-sensor normals and the −Z sensor's mask (§6.11); the magnetometer at the +Z end, 60–85 mm from the rods.
   The RM3100 is on board A2 at that slot (OBC item C13): 54.7 mm from the nearest rod end (the X rod's +X end),
   830 nT per mA·m² of rod remanence (OBC design §9.4), with the 20 mm non-ferromagnetic zone of §4.4.
7. **RBF and access port** on −Y at x = ±20, z = −43 mm, to be matched to the Nova access window (LVR-068).
8. **MASS_BUDGET** (done here, v3.1) and **STRUCTURAL_ANALYSIS** (done here, B).
9. **Jetson carrier (OBC item C3, D-11):** 27 × 27 mm window at x −33.5 … −6.5, y +17.0 … +44.0 mm with the window
   + 0.5 mm keep-out on every layer; **board 2.0 mm** (bottom face at z +17.6, top face unchanged at +19.6; stack-up and
   impedances re-done by the carrier designer); **no frame**: H10–H13 and the frame keep-outs are deleted (§4.3.3);
   bottom-side parts ≤ 1 mm. The magnetic keep-out round (+33, −38) is withdrawn (A2, §4.4); MP3 in brass if the AV-V4
   test needs it.
10. **OBC board (SQSO-OBC-BRD-001 §21):** X1 at (−20, +30.5) mm; bottom-side parts ≤ 1.0 mm; top-side parts ≤ 3 mm
    except X1 and the connectors (up to 9.0 mm is free under the 2.0 mm carrier); J1 and the EPS's J3 at the same +X edge with the W3 ribbon in the +X wall gap at
    |y| < 17 mm; J2 at (+41.0, −33.5) on the top side with the W4A ribbon flat in the +X wall gap at y −28.5 … −38.5 (§6.9); J7 on the top side or clear of the
    Z rod's footprint (§4.4).
11. **Flight software (via the chair):** a gyro-to-body rotation matrix from VI-MCAD-05.
13. **Harness (ICX item E12, RF board C §4, §15 items 14–15):** the (+X, −Y) bundle is y −24.0 … −33.9 below z −26 and
    **y −20.5 … −27.0 above it** (3 × 6.5 mm: 66 % of the other bundles' section; the proposed y −17 … −27 over the whole
    height would cross the Y rod's −Y end, y ≥ −23.5 at z −38.2 … −27.6, and the +X spreader flange, y ≥ −20 at
    z +29.8 … +32.8); the W4A ribbon (10 mm) runs flat on the +X shear panel at y −28.5 … −38.5 from OBC J2 to RF J1, both at
    (+41.0, −33.5), 1.5 mm from the bundle; the W10 coax (four 1.37 mm cables) in the −X wall gap at y +20 … +24 (70 cm)
    and y −20 … −24 (2 m), from the MMCX jacks down to z −45.5 and inboard under the battery end caps through the −Z end
    frame's cut-out to the AntS; these two −X bands carry no harness (the −X bundles stay at |y| = 24 … 33.9).
12. **Jetson carrier (item M1, SQSO-JET-BRD-001 §14, §18):** window now at y +17.0 … +44.0 mm (gyro at (−20, +30.5)) to
    clear MP2; §18's bottom-side keep-out "above the M-G370PDT0 at (−20, +22)" becomes the window (no frame, item 9); the
    MP2/MP3 positions (±31.3, +12.6) to be confirmed on the NVIDIA module drawing; the (0, −21) support takes a low-head
    M2 screw (top at 1.1 mm above the board); top-side parts ≤ 7.7 mm under the spreader (y < +27) and ≤ 13.2 mm beyond;
    the spreader pedestal, pads, flatness and shims of §4.5 are on the spreader drawing.
14. **Frame RFQ and RF board (inertia ballast, C-13, D-9):** four M2 holes in the +Z end frame for the ballast discs at
    (−17, +15.5) and (−17, +0.5) mm: the discs lie under the GNSS patch footprint, so the Ti-6Al-4V screws enter from the outer face,
    countersunk flush, into tapped holes in the discs (countersink depth in the 1.6 mm plate open, with the frame and
    patch vendor drawings); the RF board's shield lid
    under the discs (x −24 … −10, y −6.5 … +14) no higher than 7.0 mm and the parts beside the shield (y +14 … +22.5)
    no higher than 2 mm (1.4 mm below the discs).
15. **Frame RFQ, panel vendor, harness and RF designers (SQSO-HAR-001 2.0 M-1 to M-9, §6.11):** shear panels: four
    6 × 4 mm grommeted slots at z −12 … −8 (+X y −33 … −27; −X y −15 … −9; ±Y x −33 … −27), two 1.5 × 2.8 mm flex
    slots at y +36.5 … +38.0, z −40.0 … −37.2 on the ±X panels, a Ø1.8 mm hole at (−X, y +28, z −45.5) and the
    22.4 × 6.4 mm Micro-D window in the −Y panel (x −31.2 … −8.8, z −45.6 … −39.2); +Z end frame: Ø2 mm hole at
    (−36, +36); −Y body panel: the 30.4 × 9.2 mm window at x −35.2 … −4.8, z −47.0 … −37.8 (replaces the 18.4 × 6.4 mm
    port cut-out); −X body panel: Ø1.8 mm hole at (y +28, z −45.5) and the MZ carrier's bond area in the −Z end zone;
    AntS and GNSS patch mounting with isolating washers and sleeves (D-13); RF J1 right-angle TFM-108 in the enlarged
    shield notch x > +30 (RF D §4.3, taken in the CAD); the OBC's J2 right-angle (harness request O-1) is a mated-height
    change inside the existing 8 mm allocation. **Harness tie-down rule (§8.5):** under the EPS, only the connector
    strain-relief ties are on the EPS; the wire dress in zones U_* is tied to the deck straps and the shear panels (the
    EPS Steinberg margin is +0.23 so, −0.52 with the wire on the EPS); the deck then carries about 21 g (169 Hz,
    0.46 mm 3σ): the GNSS receiver's random qualification against the deck response remains the check of finding 3.

## 12. Verification Items

| ID | Verification | Pass criterion | Verifies | When |
|----|--------------|----------------|----------|------|
| VI-MCAD-01 | Re-run `model.py` with the vendors' STEP models (frame, AntS, panels, wings) in place of the envelopes | All checks of §6 pass; no interference | §6 | On receipt of each vendor model; drawing release |
| VI-MCAD-02 | First-article measurement of the stowed protrusions (CMM or the CIFP method) | ≤ 6.5 mm everywhere; hinge springs ≤ 6.38 mm (wound OD ≤ 4.00 mm over the full travel) | STR-10 (with VI-STR-08) | Flight unit |
| VI-MCAD-03 | Mass, CG and full inertia tensor of the stowed flight unit on a spin-balance mass-properties instrument (not a ±2 % trifilar pendulum): MOI ±0.5 % per axis, products of inertia ±1 × 10⁻⁵ kg·m², CG ±0.1 mm, calibrated before and after with a reference mass of known tensor, the fixture's own tensor measured empty and subtracted (one accuracy with VI-LV-02 and VI-STR-01, STR-04), deployed tensor from the CAD with the weighed wings and the four AntS elements; then the ballast discs (D-9, row 36) machined to the mass the measured tensor calls for (10 g each nominal, down to removal) and the Iyz trim masses set | With the uncertainty: deployed Iyy/max(Ixx, Izz) − 0.010 ≥ 1.03 at the as-built mass (analysis 1.056 / 1.047, 1.037 MEV at the 5th percentile of 2 mm placement scatter: the ballast trim refines, it does not rescue); measured major-axis tilt ≤ 2.4° (5° less the 2.6° that ±1 × 10⁻⁵ kg·m² of Iyz is worth), set with the Iyz trim masses | BASELINE §4 (with VI-STR-01) | Flight unit before environmental test |
| VI-MCAD-08 | Stack fit check on the EM with the vendors' connector drawings and the flight harness: mated heights of OBC J1–J10, RF J1, J2 and J3–J6 against the allocations (8 / 8 / 6 / 6 mm), the W3, W4A, W4B and W10 routes and the (+X, −Y) bundle's 3 × 6.5 mm section | Every mated connector inside its allocation with ≥ 1 mm to its neighbour; routes as §6.9; no harness in the W10 bands | §4.4, §6.9 | EM |
| VI-MCAD-05 | Gyro sensing-axis alignment on the integrated flight stack (rate table about the three body axes, or an alignment cube on the module) | Each axis known to ≤ 0.1° (cross-axis error ≤ 0.014°/s at the 8°/s spin); matrix delivered to the flight software | §4.3.1; SENSOR_DESIGN §4.1 | Flight unit |
| VI-MCAD-07 | Magnetic cleanliness of the A2 zone: inspection of every part within 20 mm of A2 (§4.4) against its material, and the magnetometer bias with the flight fasteners, pins and standoffs (with V-OBC-11 / AV-V4) | No ferromagnetic part in the zone except the accepted fixed biases (17-7PH secondary spring, MP1, M.2 card); bias within the hard-iron calibration range | §4.4 | EM, flight |
| VI-MCAD-06 | Jetson module stack at integration: module drawing and STEP model against the standoff positions and the (0, −21) fastener; pedestal-to-lid gap measured with the module clamped, flanges shimmed; pressure film | Standoffs on the module holes; ≥ 1.0 mm from the fastener head to the module's underside parts; SoC pad deflection 15–35 %, force ≤ 40 N | §4.5; SQSO-JET-BRD-001 §14, V-JET-7 | EM, then flight |
| VI-MCAD-04 | EM low-level sine survey (0.25 g, 20–2000 Hz, three axes) with accelerometers on the wings, deck, each board centre and the battery; FE updated to the measured modes | First mode > 40 Hz; FE modes within 15 % (else the FE is correlated before the flight margins are reused) | §8 (with VI-STR-02, VI-LV-03) | EM |
| VI-STR-04 (amended, SQSO-MECH-001 B) | Hinge torque test of each wing (EM, then flight) with the flight-design flex at −40, +20 and +60 °C in vacuum, each spring removed in turn | Flex torque ≤ 0.50 mN·m at −40 °C (zero margin at 0.64); TM ≥ 0 with the 5017B qualification-test factors; one spring out ≥ 0 | STR-13, MCAD-01 | EM |
| VI-MECH-01, VI-MECH-02 (SQSO-MECH-001 B) | Spring lot: torque–angle at −40/+20/+60 °C and relaxation (30 days at the stowed angle, +60 °C); flex: 50 deployment cycles at −40 °C, continuity and insulation monitored | Rate and torque within ±10 %, relaxation ≤ 5 %, no set; flex: no open or short, torque ≤ 0.50 mN·m | MCAD-01 | Spring lot; EM |
| VI-MCAD-09 | Insulation resistance of the AntS housing, the GNSS patch and A2 to the frame after mounting (megohmmeter at 50 V DC), and the −Z sun sensor's mask against the AntS ICD (element exit positions, housing outline) | ≥ 1 MΩ each (SQSO-HAR-001 2.0 M-9); no element path within 1 mm of the MZ carrier | D-13, §6.11 | EM and flight integration |
| VI-STR-10 (existing) | CAD sweep (done, §6.6–§6.7) and the deployment test with the AntS, including a wing-stuck case | ≥ 10 mm in the nominal order; contingency clearance recorded | STR-16 | EM |

## 13. Limits of This Model

- Envelope level: brackets, fasteners, connectors and cable routing are envelopes or distributed masses; bought parts are
  published or allocated envelopes until the vendors' models arrive.
- Uniform density per part: a part's own inertia is approximate (a board's components are not where its envelope puts
  them); the error on the spacecraft inertia is small against the placement sensitivity of §5.4.
- The FE limits are in §8.7.
- The drawings are generated layout drawings, not a manufacturing drawing package.

## 14. Revision History

| Version | Date | Author | Description |
|---------|------|--------|-------------|
| A | 2026-10-27 | Mechanical design (CDR design completion) | Initial issue: parametric CAD of layout B with MEL masses; mass properties and principal-axis balancing; CDS envelope, tolerance-stack, standoff, interference, sweep and clearance checks; STEP (full and outer mould line) and SVG drawings; beam-shell FE model with modes, effective masses, quasi-static and random stresses and margins; L-8 M deliverable statement; RIDs SE-01 (mechanical), SE-12, STR-01, STR-10, STR-11, STR-16; findings MCAD-01 to MCAD-03 |
| B | 2026-10-27 | Mechanical design (CDR design completion) | Chair follow-up: wing hinge designed (two OD 4.0 mm torsion springs, clock-spring flex in the −Z knuckle, damped stop, no latch; `hinge.py`), MCAD-01 and STR-13 closed by design with verification items; protrusion stack now governed by the springs (6.10 / 6.38 mm); skeleton torquer deck and row-5 re-split (MCAD-02 closed by reallocation); two centre supports at (0, ±21) mm adopted (chair decision) with 1.6 mm boards; Jetson to y = −6 mm; carrier 2.0 mm with the gyro window and no frame (the bonded frame withdrawn, §4.3.3); four AntS elements (2 m ±X, 70 cm ±Y; ratio 1.045 / 1.037 without ballast; chair decision C-13: 2 × 10 g W-Ni-Cu Z-axis ballast on the +Z end frame adopted, MEL row 36, ratio 1.056 / 1.047, D-9, §5.2); harness bundle (+X, −Y), W4A, RF connectors and W10 coax (ICX E12, RF C §4; §6.9); Ti stud at (+40, +43); FE re-run (§8); magnetometer board A2 in the +Z slot with its 20 mm non-ferromagnetic zone, W6E/W8A routing, no OBC frame bond (OBC item C13; §4.4, §6.9, VI-MCAD-07); window-frame screws H10–H13 amended into tapped pads (§4.3.3); Jetson carrier item M1 (spreader pedestal, gap pads, module standoffs, (0, −21) fastener, shims and thermal path; §4.5, VI-MCAD-06; gyro and window moved 3 mm toward +Y to clear the module standoff); gyro (OBC item C3) through a framed window in the carrier (§4.3, D-11) after the carrier raise and the bracket were shown not to fit, OBC placement constraints recorded (§4.4), gyro alignment item VI-MCAD-05; Steinberg allowable made thickness-dependent (2.0 mm reserve +0.41 to +0.58); mass properties, interference and FE results recomputed (with the C-13 ballast and the coax, ribbon and hinge springs lumped: FE mass = CAD 1,514.3 g, lowest MS +0.72, Steinberg +0.08 to +0.40, +Z end frame 402 Hz); **harness design SQSO-HAR-001 2.0 §15 M-1 to M-9 and D-2** (§6.11, D-12, D-13): harness zones with their routed mass, EPS bottom-side connector envelope, four grommeted shear-panel slots, A3 and its Micro-D 21 with the −Y window, tabs TB-W1/TB-W2, ±Z sun sensors (the −Z one beside the AntS on the −X body panel, field of view stated), W4A flat, isolated AntS and patch mounts (VI-MCAD-09); RF J1 TFM-108 right-angle and the shield notch x > +30 (RF D §4.3); MEL rows 32 (75.6 g) and 26 (9.0 g): CBE 1,513.9 g, MEV 1,747.8 g; ballast pair moved to y +15.5 / +0.5 to cancel the distribution's Iyz (ratio 1.055 / 1.046, axis 1.0° / 1.0°); FE re-run (first mode 169 Hz, the deck with the harness dress; lowest MS +1.00; Steinberg +0.08 to +0.37, the EPS +0.23 with the tie-down rule of §8.5) |
