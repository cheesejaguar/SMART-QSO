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
| Interference, stowed and deployed | None (284 / 278 solid pairs with overlapping boxes tested by boolean intersection) | — | §6.5 |
| Gyro in its socket (OBC item C3) | M-G370PDT0 stands 14.0 mm above the OBC (11.4 mm to the carrier): **through a 27 × 27 mm window in the carrier, framed by a 2.2 g Al stiffener** (keeps the carrier's Steinberg margin, +0.09), 1.5 mm clear of the carrier and the frame and 3.6 mm of the Jetson module; raising the carrier 3 mm and a bracket with a ribbon were evaluated and do not fit | OBC_BOARD_DESIGN §9.2, §24 C3 | §4.3 |
| Wing sweep (0–90°) | 0.35 mm minimum clearance (at 45°, wing back cells to body-panel coverglass) | > 0.3 mm allowance | §6.6 |
| AntS tapes to the stowed wings (STR-16) | 6.5 mm (tape band to the wing's −Z edge, wing shortened and moved +3 mm); 0.5 mm with the 98 mm wing centred; deployed 2 m tape to the deployed wing 37.8 mm | ≥ 10 mm or wings first | §6.7 |
| Wing hinge and deployment springs (MCAD-01, STR-13) | Version A springs do not fit; **Version B design fits**: two 17-7PH torsion springs on the hinge axis (primary below the wing, secondary at +Z), clock-spring flex at R 1.0–1.6 mm. Torque margin (5017B, analysis factors) **+0.14** at the cold end of travel, one spring out +0.13; spring stress MS +0.12 / **+0.04** (yield); damped soft stop 0.157 N·m (214 µε); no latch, holding margin +17 | TM ≥ 0 (5017B Eq. 4-1); Table 4; DDMR 14, 53; ≤ 0.3 N·m (STR-14) | §6.8; SQSO-MECH-001 B |
| Mass, CG | 1,494.3 g CBE / 1,727.7 g MEV (= MEL, row 23 at 66.6 g with the SoC pedestal; row 5 re-split for the skeleton deck, MCAD-02); stowed CG (+0.4, −0.2, −1.8) mm CBE | ≤ 2.00 kg; ±20 mm (5 mm baseline) | §5 |
| Deployed inertia (STR-01) | Iyy/max(Ixx, Izz) **1.056 CBE / 1.047 MEV**; major axis **0.5° / 1.3°** from +Y | ≥ 1.03; ≤ 5° (BASELINE §4) | §5 |
| First natural frequency, stowed (FE) | **170 Hz** (stowed wings, out of plane); deck 195, carrier 327, RF 385, EPS 445, OBC 574, battery 1,287 Hz | > 40 Hz (UG v11 §4.1.1) | §8.2 |
| Margins of safety at the design limit load (larger of the quasi-static 10 g / 17 g and the 3σ random response at 8.18 Grms), NASA-STD-5001B protoflight factors | All positive. Lowest: **+0.72** yield (−Z end frame at the battery isolators), +1.29 (end frames at the rail corners), +1.68 (centre support), +2.17 (rails), +2.44 (stack columns); G-10CR isolator +4.6 (FS 2.0) | MS ≥ 0 | §8.5 |
| PCB fatigue (Steinberg, FE relative displacement, 3σ at 8.18 Grms) | **+0.09 to +0.36** with the two centre supports (D-7, adopted with 1.6 mm boards) and the gyro window framed (D-11); −0.08 (EPS) and −0.29 (carrier) with one | MS ≥ 0 | §8.5 |

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
| `results.json` | Parameters, the part list (91 parts: name, MEL rows, mass, centroid, volume), mass properties, every check with its value |
| `smartqso_stowed.step`, `smartqso_deployed.step` | Full assemblies, STEP AP214, 143 solids, 2.9 MB each |
| `smartqso_oml_stowed.step`, `smartqso_oml_deployed.step` | Outer mould line and dispenser interfaces only (rails, end frames, panels, wings, hinges, AntS, GNSS patch, switches, plungers, RBF, access port; 57 solids), 1.1 MB each: the form SpaceX UG v11 Appendix C asks for (no internal components) |
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
| +15.0 … +21.6 | Jetson carrier (substrate +18.0 … +19.6), 27 × 27 mm window at x −33.5 … −6.5, y +17.0 … +44.0, Al window frame 2.5 × 3.0 mm below it (z +15.0 … +18.0) | Columns, centre supports at (0, ±21) |
| +19.6 … +27.8 | SO-DIMM on the −Y edge (4.0 mm stack), Jetson module centred at (0, −6) on two steel standoffs MP2/MP3 at (±31.3, +12.6); SoC lid 20 × 20 mm at (0, −6); gap pads (1.0 mm on the SoC, 2.0 mm on the DRAM and regulators, 25 % deflection); clamp bars at both short edges | SO-DIMM and standoffs on the carrier (§4.5) |
| +27.05 … +32.8 | Heat spreader 80 × 60 × 5 mm at (0, −3) with a 22 × 22 × 0.75 mm pedestal over the SoC lid and a 0.5 mm second face over the DRAM and regulators (§4.5), flanges 3 mm to the ±X shear panels (y −20 … +20) | 3 × M2.5 per side with a thermal gasket, to the shear panels |
| +33.3 … +43.4 | RF board (substrate +34.8 … +36.4) with an 84 × 54 × 7 mm shield; ≤ 2 mm parts outside it | Columns, centre support (0, 0) to the spreader |
| +40.4 … +48.4 | Magnetometer at (+24, +27); GNSS patch adapter at (0, 0) | +Z end frame |
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
| D-9 | Two 5 g trim-mass positions on the +Z end frame at y = ±45 mm | §5.4 |
| D-10 | No deployed-position latch; damped silicone stop (ζ ≥ 0.3); MoS₂ pins; legs 5 mm long | SQSO-MECH-001 B §4A–§5A |
| D-11 | Gyro (M-G370PDT0 in socket X1) at (−20, +30.5) on the OBC, standing 14.0 mm through a 27 × 27 mm window in the Jetson carrier, framed by a 2.5 × 3.0 mm Al stiffener on the carrier's bottom face; carrier and stack heights unchanged | OBC item C3: 14 mm needed, 11.4 mm available; the two options proposed do not fit (§4.3) |

### 4.3 Gyro Mounting (OBC Item C3)

The OBC board design (SQSO-OBC-BRD-001 §9.2, §24 item C3) puts the Epson M-G370PDT0 (24 × 24 × 10 mm, 10 g) in a Samtec
CLM-110 socket (X1), so that it stands about 14 mm above the OBC's top face; the stack leaves 11.4 mm (OBC top face
+6.6 mm, carrier substrate +18.0 mm). The chair asked for a choice between raising the carrier 3 mm and a frame bracket
with a 0.1 m ribbon. Neither fits layout B; a third arrangement does, and is adopted (D-11):

| Option | Stack height in the 113.5 mm envelope | Alignment to the body axes | Vibration environment | Mass properties (deployed, CBE / MEV) | Result |
|--------|---------------------------------------|----------------------------|-----------------------|----------------------------------------|--------|
| (1) Raise the carrier, module, spreader and RF board 3 mm | Needs 3.0 mm (2.6 mm with zero clearance at the gyro). Free above the stack: 1.0 mm (RF shield top to the GNSS patch adapter) and 2.0 mm (RF parts to the magnetometer); below the OBC: 1.0 mm (EPS bottom parts to the X rod), 2.5 mm (to the GNSS receiver) and 1.4 mm (deck to the battery sense board). Raised 3 mm, the RF board overlaps the patch adapter by 2.0 mm and the magnetometer by 1.0 mm; lowering the EPS and OBC instead runs into the X rod after 1.0 mm. The 3.0 mm exists only with every one of these gaps at zero, or with a notched RF shield, a shorter patch adapter and an EPS keep-out over the rod | Unchanged (board-mounted) | Unchanged (gyro on the OBC) | CG z +0.6 mm; Iyy/max 1.072 / 1.063, axis 0.9° / 2.3° (better ratio: the upper stack moves away from the CG) | **Does not fit** without three other designs changing |
| (2) Frame bracket and a 0.1 m FFSD-10-D ribbon to an FTSH header at X1 | The module with a mated ribbon socket is as tall as on its socket (about 14 mm), so the bracket needs a free 24 × 24 × 15.5 mm volume (module, connector, 1.5 mm bracket). A voxel search of the CAD (1 mm grid, every solid) finds none in layout B in any orientation; the largest 24 × 24 footprint is 13.9 mm high, through a lightening hole of the torquer deck between the battery sense board and the EPS, with no room for the bracket. The ribbon would also have to pass the EPS board through the 3.3 mm ±X wall gaps | A machined bracket on the frame could be held to about 0.1° (better than a board) | The deck is the softest internal structure (195 Hz, §8.2); a frame bracket elsewhere does not exist | On the deck: CG z −0.16 mm; axis 1.3° / 3.1° (the gyro moves 24 mm toward −Z, adding Iyz) | **Does not fit** |
| **(3) Gyro on its socket on the OBC, through a 27 × 27 mm window in the Jetson carrier, framed by a 2.2 g Al stiffener (adopted)** | No stack height change. Gyro at (−20, +30.5) mm, top at z +20.6; the window (x −33.5 … −6.5, y +17.0 … +44.0) clears it by 1.5 mm on every side; 2.0 mm beside and 3.0 mm below the Jetson module board's far edge (y +16.5; 3.6 mm in all); 3.4 mm from the module standoff MP2 at (−31.3, +12.6), whose pad sets the window's −Y edge (§4.5); 5.1 mm to the centre support at (0, +21). Nothing is above the gyro below the spreader (z +27.8): a socketed height up to 20 mm still clears it | As the OBC board: socket on a 1.6 mm board on six supports (§4.3.1) | The OBC board's response (below) | CG z +0.01 mm; Iyy/max 1.056 / 1.047, axis 0.4° / 1.4° | **Fits** (check OBC-C3, `results.json`); carrier Steinberg margin kept by the frame |

**What the carrier gives up.** 729 mm² (8 %) of the carrier's 8,646 mm², in its +Y, −X quadrant outside the module
footprint, and 2.2 g of FR-4, which the Al frame replaces (row 21 split 14 g substrate, 23.8 g parts, 2.2 g frame: no MEL
change). The window keeps a 3.9 mm web to the board edge (the frame comes to 1.4 mm of it), 3.0 mm to the
centre support's keep-out (Ø7 mm) and 3.0 mm to the corner column's. It was first placed at y +14.0 … +41.0; the
carrier design (SQSO-JET-BRD-001, item M1) puts the module standoff MP2 at the module's far-edge hole, (−31.3, +12.6)
mm by the module outline, whose Ø5.1 mm spacer and pad need board to y ≈ +16, so window and gyro moved 3 mm toward +Y.
The window lies in the carrier's −X power zone of its +Y strip (J1 at the −X edge); the carrier designer is placing
those parts round it. FE (§8.5): the window alone takes the carrier's Steinberg margin from +0.08 (same mesh, no window) to
**−0.03**: it cuts the board between the corner column (−40, +43) and the centre support (0, +21). A 2.0 mm carrier
recovers only +0.09 (the allowable falls as the board thickens). The design therefore frames the window: Al 6061,
2.5 mm wide × 3.0 mm high, on the carrier's bottom face (z +15.0 … +18.0, 5.4 mm above the OBC's 3 mm parts envelope,
1.5 mm from the gyro, 1.1 mm from the centre support and the corner column), bonded and held by four M2 screws; it
restores **+0.09** with the 1.6 mm board (frame MS +10.5 yield).

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
| X | **22.0 Grms** | 4.4 | 12.5 |
| Y | 3.3 | **25.0 Grms** | 10.5 |
| Z | 1.9 | 3.2 | **23.8 Grms** |

About 2.7–3.1 times the input in the input axis (the stack modes at 326–396 Hz). The bracket on the deck would have
put it on the 195 Hz deck, and a frame bracket does not exist (option 2). The 3σ motion of the carrier's window edge
relative to the gyro is 0.09 mm against the 1.5 mm clearance. Epson's vibration and shock ratings for
the M-G370 were not read for this issue; the comparison is an open item for the OBC designer, closed by the EM board
random test (VI-STR-02) with the gyro fitted and powered checks before and after.

### 4.4 OBC Board Placement Constraints That the Stack Sets (SQSO-OBC-BRD-001 §21)

The OBC's layout guidance is in its §21 (its §22 is the verification list). The items that touch the stack, and what
the CAD gives them:

| OBC §21 item | Stack constraint (this model) | Status |
|--------------|-------------------------------|--------|
| Outline, holes: PC/104 90.17 × 95.89 mm, four Ø3.18 mm holes, the two centre supports at (0, ±21), Ø6.4 mm copper keep-outs, one GND-bonded hole matching the EPS's | Same positions as the CAD; the CAD keeps Ø7 mm free of parts on both sides. The single frame bond must be at the same corner column on the EPS and the OBC, and the other columns and supports must not touch copper (the Al spacers are conductive) | Consistent |
| Stack-up 6 layers, 1.6 mm | The FE and the Steinberg margins use 1.6 mm (OBC +0.14 with the two supports; §8.5) | Consistent |
| Heights | Top side ≤ 3 mm (envelope) except X1; up to 9.4 mm is free under the carrier's 1 mm bottom-side envelope with 1 mm clearance (7.4 mm under the window frame). **Bottom side ≤ 1.0 mm**: the EPS's 5 mm top envelope leaves 0.4 mm below it | Constraint for the layout |
| X1 (gyro) "near the board centre on the spin-axis side" | At (−20, +30.5) mm, 24 × 24 mm, under the carrier window of §4.3; the board centre is under the Jetson module and the centre supports | Placed |
| J1 on the edge facing the EPS's J3 (W3, 40-way ribbon, about 26 mm wide) | J1 cannot be on the bottom side (1.0 mm), and a mated FTSH/ribbon socket on the EPS top side does not fit under the OBC (6.4 mm between the substrates; mated height open, Samtec drawing). The ribbon can only leave the stack in the ±X wall gaps (3.3 mm, x ±45.4 … ±48.4; the ±Y gaps are 0.55 mm): J1 and J3 at the same ±X edge, at |y| < 24 mm (the harness bundles occupy |y| = 24 … 34), on the +X edge to stay clear of the Z rod (x −46.8 … −36.2, |y| < 5.3, up to z +3.0) | Constraint for the OBC and EPS layouts |
| J2 "on the +Z-facing edge toward the RF board" (W4A, FTSH-107, about 9 mm wide) | J2 on the top side; the ribbon passes the carrier and the spreader in a ±X wall gap at |y| = 34 … 44 mm (the spreader flanges fill |y| < 20 at z +29.8 … +32.8, the harness bundles |y| = 24 … 34) | Constraint for the OBC and RF layouts |
| DRV8876s and J7 (rod leads) on the −X edge nearest the Z rod's top | The Z rod ends at z +3.0 under the OBC at (−41.5, 0), 1.0 mm below the OBC's bottom-side envelope: J7 on the top side or outside the rod footprint (x −46.8 … −36.2, |y| < 5.3) | Constraint for the layout |
| RM3100 (U15, L1–L3) in the (+X, −Y) corner; no ferromagnetic parts within 20 mm and no unpaired current within 30 mm | Inside the OBC the rule is the OBC's. In the stack, within 20 mm of that corner (about (+33, −38, +7)): the carrier's bottom-side parts 10 mm above, the SO-DIMM connector (about 14 mm, its latches) and the Jetson module's corner (about 18 mm); the battery cans are 31 mm away. **The rule therefore extends to the carrier**: no ferromagnetic parts and no unpaired high-current traces (the Jetson input, up to the 5 V burst current) within 20–30 mm of (+33, −38) on either side of the carrier; the SO-DIMM connector's latches non-magnetic or the RM3100 moved toward +Y | New interface item for the carrier and the OBC (§11) |
| Magnetometer to the rods (ADCS: 60–85 mm at the +Z position of §4.1) | From that corner: 43 mm to the end of the Y rod (+42, −23.5, −32.9), 77 mm to the X rod, 84 mm to the Z rod. A dipole field falls as 1/r³, so the Y rod's remanent field at the sensor is 2.7–7.7 times what it is at the +Z module position; of the four corners this one is the farthest from the rods (the others are 20–34 mm from the X or Z rod) | For the chair and the ADCS owner with OBC item C1 (AV-V4 decides) |

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

Every MEL row (MASS_BUDGET v3.1 §3.1, rods option) is carried by named solids; the CAD total is the MEL total: **1,494.3 g
CBE, 1,727.7 g MEV**, margin +15.8 % to 2.00 kg (row 23 raised to 66.6 g for the SoC pedestal, §4.5). By group (CBE): stack 437.6 g, battery 267.0 g, body panels 160.0 g,
frame 128.0 g (row 1 plus brackets and deck), wings 110.2 g, harness 105.0 g, AntS 85.0 g, ADCS 79.5 g, distributed
55.0 g, GNSS 52.0 g, inhibits 15.0 g.

**MEL row 5 (internal brackets, 30 g CBE; MCAD-02).** The solid 1.0 mm deck of Version A (23 g of aluminium) exceeded
the 6 g it drew from row 5. The deck is now a skeleton (7.8 g of aluminium: 3 mm rim, Ø10 mm column bosses, a 50 × 12 mm
strap under the X rod, two 84 × 6 mm bars under the GNSS receiver, a 6 mm strap to the upper centre support) and draws
9 g with its clips. The Jetson heat-path flanges, which row 5 carried at 18 g, are 5.5 g of aluminium in the CAD and draw
15 g with the thermal gasket, six M2.5 fasteners and inserts. Row 5 split: deck 9, heat-path hardware 15, hinge doublers 4,
GNSS patch bracket 2 = 30 g: **no MEL change**. If the chair prefers to grow row 5 instead (deck +3 g over its earlier
allocation, flanges unchanged), the MEV becomes 1,729.7 g and the margin to 2.00 kg +15.6 % (from +15.9 %).

### 5.2 CG and Inertia (about the CG, ×10⁻³ kg·m², tensor convention)

| Case | CG (mm) | Ixx | Iyy | Izz | Ixy | Ixz | Iyz | Iyy/max(Ixx, Izz) | Major axis from +Y |
|------|---------|-----|-----|-----|-----|-----|-----|-------------------|--------------------|
| Stowed, CBE | (+0.43, −0.16, −1.82) | 2.716 | 3.080 | 2.805 | +0.003 | +0.001 | +0.027 | 1.098 | 5.6° |
| Stowed, MEV | (+0.47, −0.23, −0.68) | 3.133 | 3.557 | 3.283 | +0.002 | −0.000 | +0.032 | 1.083 | 6.6° |
| **Deployed, CBE** | (+0.84, +2.10, −1.82) | **2.780** | **3.963** | **3.753** | +0.005 | +0.032 | −0.002 | **1.056** | **0.5°** |
| **Deployed, MEV** | (+0.84, +1.95, −0.68) | **3.205** | **4.524** | **4.322** | +0.005 | +0.033 | +0.004 | **1.047** | **1.3°** |
| W1 deployed only, CBE | (+2.06, +0.91, −1.82) | 2.746 | 3.663 | 3.419 | −0.159 | +0.021 | +0.009 | 1.071 | 9.6° |
| Antennas out, wings stowed, CBE | (+0.84, −0.27, −1.82) | 2.708 | 3.373 | 3.090 | +0.002 | +0.032 | +0.019 | 1.091 | 3.8° |
| Deployed, CBE, MEL hybrid antennas (2 m ±X, 70 cm ±Y) | (+0.43, +2.10, −1.82) | 2.819 | 4.217 | 4.044 | +0.004 | +0.001 | −0.002 | 1.043 | 0.6° |
| Deployed, MEV, MEL hybrid antennas | (+0.47, +1.95, −0.68) | 3.245 | 4.792 | 4.631 | +0.003 | −0.000 | +0.004 | 1.035 | 1.6° |

Against MASS_BUDGET v3.0 §5–§6 (reference layout B): the stowed CG is 3 mm higher in Z (−1.8 against −4.7 mm: the CAD
battery centre is −32.9 mm, not −35, and the boards sit higher), the deployed moments are 3 % lower on every axis, and
the ratio agrees (hybrid antennas: 1.043 CAD against 1.041; option A: 1.056 against 1.054). The difference that matters
is the principal axis: 0.5–1.6° here against 13–15° there (§5.3). The baseline requirement (Iyy ≥ 1.03 × max, axis
within 5°) is **met** at CBE and MEV for the full deployment; with one wing out the axis is 9.6° off (as in MECHANISMS_ANALYSIS §8, a degraded
mode), and with the antennas out and both wings stowed 3.8°.

### 5.3 Balancing the Principal Axis

Iyy − Izz is 0.21 × 10⁻³ kg·m², so the axis tilt is θ = ½ atan(2 Iyz / (Iyy − Izz)): 4 × 10⁻⁵ kg·m² of Iyz tilts it by
13°. The deployed wings move the CG +2.0 mm in Y; every part far from the CG in Z then adds to Iyz. The first CAD
placement gave 17°; moving the Jetson module, clamp and spreader to y = −6 mm (with the SO-DIMM on the −Y edge and the
upper centre support at y = +21 mm), the RBF and access port to −Y and the harness into four symmetric bundles cancels it
without ballast (MASS_BUDGET v3.1 §6.6). Version B moved them from −7 to −6 mm to absorb the Iyz of the hinge
springs, flex cup and skeleton deck (a more negative y made it worse).

### 5.4 Sensitivity and Trim

A placement Monte Carlo (every internal part moved by an independent Gaussian error σ per axis, 2,000 trials) gives the
axis within 5° in 99.5 % / 96.4 % of trials (CBE / MEV) at σ = 1 mm, 86.7 % / 76.1 % at 2 mm, and 42.8 % / 37.7 % at
5 mm (95th-percentile tilt 3.6° / 4.7° at 1 mm); the 5th-percentile ratio is 1.045 / 1.037 at 2 mm and 1.027 / 1.020 at
5 mm, so it stays ≥ 1.03 to about σ = 3 mm. The requirement therefore needs drawing-controlled positions and a
measured flight unit. Provision D-9: two 5 g trim masses at (0, ±45, +44) mm on the +Z end frame; each changes Iyz by
1.1 × 10⁻⁵ kg·m² (about 3° of tilt), set from the measured stowed tensor (VI-MCAD-03). Proposed flight constant
`INERTIA` (diagonal, mean of CBE and MEV): **(0.0030, 0.0042, 0.0040) kg·m²** (MASS_BUDGET v3.1 §6.5, for the chair).

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
| Access port (umbilical, charge) | −Y face | (−20, −50, −43) | 0.5 mm above the body panel; location open with LVR-068 |

Three switches on three rails give the three independent inhibits of BASELINE §4. The switches (Ø4.5 mm in Ø4.7 mm bores)
sit inside the L-angle corner with 0.25 mm (0.15 mm at the bore) to the rail legs and 0.86 mm to the stack-column spacers: a tight but feasible fit that the
frame vendor's switch provision must confirm (the CAD puts the switch in a plain bore).

### 6.5 Interference

Boolean intersection of every pair of solids whose bounding boxes overlap: none stowed (284 pairs) and none deployed
(278 pairs). The only exclusions are the stowed tapes wound inside the AntS housing (by design) and the distributed-mass
allocations (fasteners, staking, thermal materials, rod clips, heater film), which have no geometry of their own. The
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
| Pin and bushings | 440C Ø1.5 mm pin, MoS₂, Ti-6Al-4V bushings; frame ears (Al 6061) bolted through the body-panel notches to the shear panels | Ears −49.0 … −47.6 and 48.6 … 49.6; wing knuckles r 1.75 | Interference-free (§6.5) |
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

Four 3 × 10 mm bundles in the ±X wall gaps (x = ±45.4 … ±48.4, between the board edges and the shear panels) carry the
MEL harness (row 32), the GNSS coax (row 31) and the sensor cables (row 26); they are clear of the columns, the spreader
flanges, the hinge doublers and the burn-resistor boards. Past the hinges: the flex enters the body through the
body-panel notch at the −Z hinge end zone, out of the clock-spring cup in the −Z wing knuckle (z −39.9 … −37.3, tail at
y 37.0–37.5), inside the 6.5 mm envelope (6.10 mm) and outside the rail zone (y ≤ 40.75). The battery flying leads (≥ 100 mm) run from the sense board to the EPS through the −X bundle
gap. Wire counts and connector positions are the harness design's (`hardware/harness/HARNESS_DESIGN.md`).

### 6.10 Jetson Heat Path

Geometry: SoC lid → 1.0 mm gap pad → spreader pedestal (§4.5; the carrier design's junction-to-spreader estimate is
1.09–1.16 K/W against the 5.7 K/W THM-10 allowable, verified by V-JET-7) → spreader (5 mm Al, 60 mm wide, ≈ 40 mm to each flange
root: 1.25 W/K per side) → flange (3 × 40 × 8.5 mm: 2.36 W/K per side) → bolted joint to the 1.0 mm shear panel. With bare
Al–Al contact and 2 × M2.5 per side (h ≈ 2,000 W/m²·K over 40 × 8.5 mm) the path to the frame is **0.66 W/K**; with a
thermal gasket and 3 × M2.5 per side (h ≈ 10,000) **1.10 W/K**, which meets the ≥ 1 W/K of BASELINE §6. Estimate only
(contact conductances assumed); the EM thermal balance test measures it (TB-07). The design calls out the gasket.

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
| Flat 4-node shell: MITC4 plate (assumed transverse shear, no locking), Wilson incompatible-mode membrane, drilling stiffness 10⁻⁴ E t A | −Z and +Z end frames (Al 1.6 mm, cut-outs 71 × 71 and 30 × 30 mm), four side shear panels (Al 1.0 mm), EPS, OBC, carrier and RF boards (FR-4 1.6 mm, E 18.6 GPa; the carrier with the 27 × 27 mm gyro window of D-11), torquer deck (Al 1.0 mm skeleton, 9 g with clips: equivalent plate with E × 0.35), heat spreader (Al 5 mm) and flanges (3 mm), stowed wings (FR-4 1.6 mm) | Element mass plus non-structural mass for the components, cells, body panels, AntS, harness and distributed items |
| 3D Euler–Bernoulli beam, consistent mass | Rails (L 8.5 × 8.5 × 1.5 mm between the end frames, solid 8.5 × 8.5 mm standoffs); stack columns and centre supports (Al hex 5 AF + M3 A286 stud, bending × 1/3 for the threaded joints as SQSO-SA-001 A §5); battery isolators (G-10CR Ø6/Ø2.7 mm bushing with the Ti-6Al-4V M2.5 screw in bending; axial and bending × 0.6 for joint compliance, scaled to the 8 mm bushing); clamp ears (Al 10 × 2.5 mm); the carrier's window frame (Al 2.5 × 3.0 mm on the bottom face, offset from the board mid-plane by a composite section with a 20 t strip of the board) | — |
| Translational springs | Wing hinges (pins: no moment; 2 × 10⁸ N/m normal, 5 × 10⁶ in plane), hold-down cup/cone post (2 × 10⁸ / 2 × 10⁶), snubber (silicone, 2.2 × 10⁴ N/m) | — |
| Rigid links | Shear-panel edges and end-frame corner squares to the rails; the battery (261 g rigid body, inertia from the CAD solids) to its ears; **spiders** spreading each column, centre-support and isolator attachment over the nodes within 2.6 mm (the spacer and washer face; 224 nodes) | — |

Boundary condition, as in the dispenser: every rail node fixed in x and y (the guide rails), the rail-end nodes fixed in z
(pusher plate and door), rotations free. Sensitivity: the eight rail ends fixed only (§8.6). Mesh: 10,664 nodes, 9,878
shells, 226 beams, 59,268 degrees of freedom; FE mass 1,490.7 g against the CAD and MEL CBE of 1,492.7 g at the time of the run (the 2 g are
non-structural masses of parts outside the meshed groups); the later +1.6 g of row 23 (spreader pedestal, 0.1 % of the
mass, on a part with MS +6.7) was not re-run. MEV masses are not run (+15.6 % mass
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

| Mode | f (Hz) | Effective mass x / y / z (% of 1,452 / 1,452 / 1,481 g free) | Dominant (share of kinetic energy) |
|------|--------|---------------------------------------------------------------|------------------------------------|
| 1, 2 | 170.2 | 0 / 0 / 0; 5.5 / 0 / 0 | Stowed wings W1, W2, out of plane (in and out of phase) |
| 3 | 194.6 | 0 / 0 / 1.3 | Torquer deck (X rod, GNSS receiver, Z-rod clip) |
| 4 | 270 | 0 / 0 / 1.3 | Deck, second mode |
| 5 | 326.1 | 0 / 0 / **17.2** | Carrier (0.54), EPS, OBC: stack drumming in Z |
| 6 | 327.8 | **32.1** / 0 / 0 | EPS, OBC, carrier: stack lateral in X |
| 7, 8 | 341.4 | 0 / 0 / 0 | Wings, second mode |
| 10 | 384.2 | 0.1 / 0 / 0 | RF board |
| 11 | 396.2 | 0 / **17.3** / 0 | EPS, RF, OBC: stack lateral in Y |
| 12 | 408.3 | 0 / 4.6 / 0 | RF board, stack |
| 20 | 444.7 | — | EPS board |
| — | 488, 546, 574, 612, 630 | small | Spreader; +Z end frame; OBC; ±X and ±Y shear panels |
| 57 | 969 | 0 / 2.6 / 6.8 | Battery on its isolators, axial |
| 101 | 1,287 | **17.4** / 0 / 0 | Battery on its isolators, lateral |
| — | 1,525 | — | −Z end frame with the AntS |

450 modes, to 3,726 Hz. Cumulative effective mass to 2,000 Hz: 77 % (x), 57 % (y), 66 % (z) of the free mass; the rest is
the frame and panel mass next to the held rails, which moves with the base. First mode 170 Hz: **4.3 times the 40 Hz
requirement**; no mode below 150 Hz, so the spacecraft is rigid in the launch-vehicle coupled-loads band (UG Appendix B).

### 8.3 Against the Analytic Values of SQSO-SA-001 A §5

| Element | SA-001 A (closed form) | FE | Comment |
|---------|------------------------|----|---------|
| Stowed wing | 142 Hz (four points: two hinges, hold-down, bumper) | 170 Hz (three points and a soft snubber; 183 Hz with a second hard post) | Shorter wing (92 mm) and the hold-down at the corner |
| PCB stack, lateral, tied at both ends | 289 Hz | 328 Hz (X), 396 Hz (Y) | The spreader bolted to the walls at mid-height stiffens the stack; Y stiffer through the flanges |
| Boards with a centre support | 221 (RF) – 362 (OBC) Hz | carrier 327, RF 385, EPS 445, OBC 574 Hz (in the stack) | Two centre supports (D-7); boards coupled |
| Battery on G-10CR isolators, lateral | 1,280 Hz | 1,287 Hz | Agreement |
| Deck (new part) | — | 195 Hz | Lowest internal mode |

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
| −Z end frame at the battery isolators | 10.7 | **111.5** | 241 / 290 | 1.25 / 1.40 | **+0.73** | +0.86 | (−40.8, −32.5, −47.7): next to the isolator spider |
| End frames at the rail corners | 22.3 | 84.1 | 241 / 290 | 1.25 / 1.40 | +1.29 | +1.46 | +Z frame at (40.8, 41.2) |
| Centre supports | 30.4 | 71.9 | 241 / 290 | 1.25 / 1.40 | +1.68 | +1.88 | (0, +21) under the spreader |
| Side shear panels | 15.7 | 64.5 | 241 / 290 | 1.25 / 1.40 | +1.99 | +2.21 | −X panel at the spreader flange |
| Rails (L-angle, standoffs) | 14.8 | 60.8 | 241 / 290 | 1.25 / 1.40 | +2.17 | +2.41 | (−47.3, −47.3) at the −Z end frame |
| Stack columns, Al spacer | 16.7 | 55.5 | 241 / 290 | 1.25 / 1.40 | +2.47 | +2.73 | (−40, −43) under the +Z end frame |
| Stack columns, A286 stud thread root | 20.2 | 67.0 | 758 / 1,103 | 1.25 / 1.40 | +8.1 | +10.8 | Same |
| Carrier window frame (Al, D-11) | 6.3 | 16.8 | 241 / 290 | 1.25 / 1.40 | +10.5 | +11.3 | Frame corners |
| Battery isolator, G-10CR bushing | 6.8 | 34.1 | 380 (flexural) | 2.00 | — | **+4.57** | (−38.5, +31) |
| Battery isolator, Ti-6Al-4V screw | 15.2 | 76.2 | 827 / 896 | 1.25 / 1.40 | +7.7 | +7.4 | Same |
| Battery clamp ears (Al) | 5.0 | 23.0 | 241 / 290 | 1.25 / 1.40 | +7.4 | +8.0 | Ear root |
| Heat spreader and flanges (with the pedestal's 1.8 g) | 8.4 | 25.1 | 241 / 290 | 1.25 / 1.40 | +6.7 | +7.3 | At the upper centre support |
| Torquer deck (equivalent plate) | 7.8 | 34.9 | 241 / 290 | 1.25 / 1.40 | +4.5 | +4.9 | Near the column (−40, −43) |
| PCBs EPS / OBC / carrier / RF (FR-4) | 3.6 / 4.4 / 5.7 / 4.1 | 29.8 / 25.1 / 32.7 / 19.5 | 340 | 1.50 | — | +6.6 / +8.0 / +5.9 / +10.6 | At the columns |
| Stowed wings (FR-4) | 1.7 | 6.4 | 340 | 1.50 | — | +34 | Centre |

| Board | Z₃σ relative (mm) | Steinberg MS, design (two centre supports, framed gyro window) | Same, no gyro window | Window without its frame | Window, 2.0 mm carrier, no frame | One centre support (SA-001 A design) | Design with 2.0 mm EPS and carrier |
|-------|--------------------|------------------|------|------|------|------|------|
| EPS | 0.168 | **+0.14** | +0.15 | +0.17 | +0.35 | −0.08 | +0.57 |
| OBC | 0.168 | **+0.14** | +0.15 | +0.17 | +0.37 | +1.18 | +0.60 |
| Jetson carrier (with the module) | 0.176 | **+0.09** | +0.08 | **−0.03** | +0.09 | −0.29 | +0.51 |
| RF | 0.141 | **+0.36** | +0.35 | +0.34 | +0.51 | +0.25 | +0.40 |
| Deck (Al, no BGA: criterion for information) | 0.357 | (−0.46) | (−0.46) | (−0.46) | (−0.48) | (−0.47) | (−0.61) |

The 2.0 mm columns use the 2.0 mm allowable (0.154 mm). The gyro window alone costs the carrier 0.11 (to −0.03: the
window cuts the board between the corner column (−40, +43) and the centre support (0, +21)); a 2.0 mm carrier
recovers only +0.09, because the thicker board's allowable falls as its stiffness rises; the 2.2 g Al frame round the
window restores +0.09 with the 1.6 mm board, so the frame is the design (D-11).

Hold-down: the post carries 2.0 N quasi-static and **3.9 N 3σ** normal to the wing (SA-001 A §10: 4.9 N); against the
30 N line preload after a 30 % loss (21 N) with 1.15 × 1.20 on the load, separation MS = 21 / (1.38 × 3.9) − 1 = **+2.9**.
Hinges 3.9 N, snubber 3.7 N (3σ).

Findings from the FE:

1. **All strength margins are positive** at the protoflight test level with the 5001B factors. The lowest, +0.73, is a
   local peak in the 1.6 mm −Z end frame beside the isolator attachment (battery 261 g on four points of a ring 13.5 mm
   wide); it is mesh- and spider-dependent. **Design action:** a 1 mm Al doubler (or a 2.5 mm local boss) under each
   isolator on the −Z end frame, specified on the frame drawing or the battery-mount drawing.
2. **The PCB fatigue margins need the second centre support (D-7):** with the single support of SQSO-SA-001 A at the only
   place the Jetson module leaves free (y = +21 mm), the EPS and carrier boards are at −0.08 and −0.29; with the second
   support at y = −21 mm (EPS → OBC → carrier) they are +0.14 and +0.09. The 2.0 mm boards would raise them to +0.51 to
   +0.60 (with the 2.0 mm allowable; +0.8 to +0.9 in the first issue of Version B, which used the 1.6 mm allowable) at
   0.8 mm of stack height (available above the RF board, at a re-stack). Steinberg's 2 × 10⁷-cycle criterion is
   conservative for one minute per axis. **Chair decision (2026-10-27): two centre supports, boards kept at 1.6 mm**,
   since all four boards are positive; the 2.0 mm option is held in reserve until the measured Q of the EM board survey
   (VI-STR-02) is known.
3. **The deck** (a part this design adds, now a 1.0 mm skeleton) is the lowest internal mode (195 Hz) and moves
   0.36 mm (3σ) relative to its supports; its strength margin is +4.5 (equivalent plate; local stresses in the skeleton
   straps are higher and are checked at drawing release). Its payload (GNSS receiver, X rod) is not a PCB on the deck, but the receiver's
   own board sees the deck motion: if the receiver vendor's random qualification is below the deck response, stiffen the
   deck (1.6 mm with ribs, about +10 g) or move the receiver so that the second centre support can start at the deck
   (the deck-to-carrier variant gave the deck 206 Hz). The deck must not be tied to the battery clamp, which is
   isolated.

### 8.6 Sensitivity Cases

| Case | First mode | Lowest MS (where) | Steinberg range | Note |
|------|------------|-------------------|-----------------|------|
| Design (D-7, D-11, M1, rails held) | 170 Hz | +0.73 (−Z frame at the isolators) | +0.09 … +0.36 | Baseline |
| No gyro window (Version B before D-11) | 170 Hz | +0.73 | +0.08 … +0.35 | Reference |
| Gyro window without its frame | 170 Hz | +0.74 | **−0.03** … +0.34 | Rejected (D-11) |
| Gyro window, 2.0 mm carrier, no frame | 170 Hz | +0.74 | +0.09 … +0.51 | Alternative to the frame (0.4 mm of stack height, about +6 g of FR-4) |
| Single centre support | 170 Hz | +0.76 | **−0.29 … +1.18** | Rejected (D-7) |
| D-7 with 2.0 mm EPS and carrier | 170 Hz | +0.73 | +0.40 … +0.60 | Reserve option |
| D-7 with a second hard post at the +Z free corner | 183 Hz (wing) | +0.73 | +0.09 … +0.36 | Wing +13 Hz; not needed |
| D-7, rails held at the eight rail ends only | 170 Hz | **+0.83** (−Z frame), +0.84 (rails, 105 MPa), +1.41 (centre support) | +0.09 … +0.36 | Bounds the dispenser contact; rails then carry the lateral load in bending |

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
| STR-01 (check) | **Confirmed and closed by design**: CAD gives Iyy/max 1.056 / 1.047 and the major axis within 0.5° / 1.3° of +Y (the reference layout gave 13–15°, outside the 5° baseline); placement sensitivity and the trim provision are verification items (VI-MCAD-03) | §5 |
| STR-10 | **Closed by design** (Version B): hinge springs 6.10 mm nominal, 6.38 mm worst case, 6.25 mm RSS against 6.5 (D-2 recess, frame-located springs and post); spring OD ≤ 4.00 mm on the drawing; VI-MCAD-02 | §6.2 |
| STR-11 | **Closed by design** (frame RFQ: −Z face recessed ≥ 0.8 mm; 1.5 mm modelled): 1.25 mm nominal, 0.90 mm worst case | §6.3 |
| STR-16 | **Closed for the nominal sequence** (wings first: 37.8 mm); **verification item** for the wing-stuck contingency (6.5 mm against 10 mm) and the ISIS sweep | §6.7 |
| MCAD-01 (severity 2) | **Closed by design (Version B)**: two OD 4.0 mm torsion springs on the hinge pin and a clock-spring flex in the −Z knuckle fit the envelope; torque margin (5017B, analysis factors) end of travel cold **+0.14**, one spring out +0.13; spring MS +0.12 / +0.04 yield; damped stop 0.157 N·m; no latch (holding margin +17). Thin margins: verification by VI-STR-04, VI-MECH-01 and VI-MECH-02 | §6.8; SQSO-MECH-001 B §3A–§5A; `hinge.py` |
| STR-13 (reopened in Version A) | **Closed by design** with the flex at the bend radius that fits (1.0–1.6 mm, allocation 0.50 mN·m cold; zero margin at 0.64); **verification item** VI-STR-04 (measured flex and hinge torque at −40 °C) | §6.8; SQSO-MECH-001 B §3A, §4A |
| MCAD-02 (severity 3) | **Closed by reallocation (Version B)**: skeleton torquer deck 9 g (7.8 g Al) inside a re-split row 5 (deck 9, heat-path 15, doublers 4, patch bracket 2 = 30 g; MEL unchanged). The alternative (growing row 5 by the original 17 g) gives MEV 1,729.7 g and margin +15.6 % | §5.1, MASS_BUDGET v3.1 §1.2 |
| Centre supports (chair decision) | **Adopted**: two centre supports at (0, ±21) mm on the EPS, OBC and carrier boards, 1.6 mm boards kept; Steinberg margins +0.09 (carrier) to +0.36 with the framed gyro window (D-11); 2.0 mm boards held in reserve (+0.40 to +0.60 with the 2.0 mm allowable; the first issue's +0.83 to +0.90 used the 1.6 mm allowable) | §4.1, §8; SQSO-SA-001 B §8A |
| OBC item C3 (gyro stack clearance) | **Closed by design**: gyro on its socket on the OBC, through a 27 × 27 mm window in the Jetson carrier with a 2.2 g Al stiffening frame (D-11); the two proposed options (carrier raised 3 mm; bracket and ribbon) do not fit (§4.3). Gyro alignment to be measured and compensated (VI-MCAD-05) | §4.3; check OBC-C3 |
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
6. **ADCS:** rod positions and the flight inertia constant (§5.4); the magnetometer at the +Z end, 60–85 mm from the rods.
   If OBC item C1 moves it to the RM3100 in the OBC's (+X, −Y) corner, it is 43 mm from the Y rod's end (§4.4).
7. **RBF and access port** on −Y at x = ±20, z = −43 mm, to be matched to the Nova access window (LVR-068).
8. **MASS_BUDGET** (done here, v3.1) and **STRUCTURAL_ANALYSIS** (done here, B).
9. **Jetson carrier (OBC item C3, D-11):** 27 × 27 mm window at x −33.5 … −6.5, y +17.0 … +44.0 mm, no copper or parts
   within 0.5 mm of the frame footprint (2.5 mm round the window, bottom side, 3.0 mm high); the frame is bonded and held by
   four M2 screws at its corners (not modelled); no ferromagnetic parts and no unpaired high-current traces within
   20–30 mm of (+33, −38) mm on either side of the carrier (the RM3100 below it, §4.4); bottom-side parts ≤ 1 mm.
10. **OBC board (SQSO-OBC-BRD-001 §21):** X1 at (−20, +30.5) mm; bottom-side parts ≤ 1.0 mm; top-side parts ≤ 3 mm
    except X1 (up to 9.4 mm is free, 7.4 mm under the carrier's window frame); J1 and the EPS's J3 at the same +X edge with the W3 ribbon in the +X wall gap at
    |y| < 24 mm; J2 on the top side with W4A in a ±X wall gap at |y| = 34 … 44 mm; J7 on the top side or clear of the
    Z rod's footprint (§4.4).
11. **Flight software (via the chair):** a gyro-to-body rotation matrix from VI-MCAD-05.
12. **Jetson carrier (item M1, SQSO-JET-BRD-001 §14, §18):** window now at y +17.0 … +44.0 mm (gyro at (−20, +30.5)) to
    clear MP2; §18's bottom-side keep-out "above the M-G370PDT0 at (−20, +22)" becomes the window and its frame; the
    MP2/MP3 positions (±31.3, +12.6) to be confirmed on the NVIDIA module drawing; the (0, −21) support takes a low-head
    M2 screw (top at 1.1 mm above the board); top-side parts ≤ 7.7 mm under the spreader (y < +27) and ≤ 13.2 mm beyond;
    the spreader pedestal, pads, flatness and shims of §4.5 are on the spreader drawing.

## 12. Verification Items

| ID | Verification | Pass criterion | Verifies | When |
|----|--------------|----------------|----------|------|
| VI-MCAD-01 | Re-run `model.py` with the vendors' STEP models (frame, AntS, panels, wings) in place of the envelopes | All checks of §6 pass; no interference | §6 | On receipt of each vendor model; drawing release |
| VI-MCAD-02 | First-article measurement of the stowed protrusions (CMM or the CIFP method) | ≤ 6.5 mm everywhere; hinge springs ≤ 6.38 mm (wound OD ≤ 4.00 mm over the full travel) | STR-10 (with VI-STR-08) | Flight unit |
| VI-MCAD-03 | Mass, CG and full inertia tensor of the stowed flight unit (six-orientation pendulum or spin balance), trim masses set, deployed tensor from the CAD with the weighed wings | Iyy ≥ 1.03 × max(Ixx, Izz), major axis ≤ 5° from +Y | BASELINE §4 (with VI-STR-01) | Flight unit before environmental test |
| VI-MCAD-05 | Gyro sensing-axis alignment on the integrated flight stack (rate table about the three body axes, or an alignment cube on the module) | Each axis known to ≤ 0.1° (cross-axis error ≤ 0.014°/s at the 8°/s spin); matrix delivered to the flight software | §4.3.1; SENSOR_DESIGN §4.1 | Flight unit |
| VI-MCAD-06 | Jetson module stack at integration: module drawing and STEP model against the standoff positions and the (0, −21) fastener; pedestal-to-lid gap measured with the module clamped, flanges shimmed; pressure film | Standoffs on the module holes; ≥ 1.0 mm from the fastener head to the module's underside parts; SoC pad deflection 15–35 %, force ≤ 40 N | §4.5; SQSO-JET-BRD-001 §14, V-JET-7 | EM, then flight |
| VI-MCAD-04 | EM low-level sine survey (0.25 g, 20–2000 Hz, three axes) with accelerometers on the wings, deck, each board centre and the battery; FE updated to the measured modes | First mode > 40 Hz; FE modes within 15 % (else the FE is correlated before the flight margins are reused) | §8 (with VI-STR-02, VI-LV-03) | EM |
| VI-STR-04 (amended, SQSO-MECH-001 B) | Hinge torque test of each wing (EM, then flight) with the flight-design flex at −40, +20 and +60 °C in vacuum, each spring removed in turn | Flex torque ≤ 0.50 mN·m at −40 °C (zero margin at 0.64); TM ≥ 0 with the 5017B qualification-test factors; one spring out ≥ 0 | STR-13, MCAD-01 | EM |
| VI-MECH-01, VI-MECH-02 (SQSO-MECH-001 B) | Spring lot: torque–angle at −40/+20/+60 °C and relaxation (30 days at the stowed angle, +60 °C); flex: 50 deployment cycles at −40 °C, continuity and insulation monitored | Rate and torque within ±10 %, relaxation ≤ 5 %, no set; flex: no open or short, torque ≤ 0.50 mN·m | MCAD-01 | Spring lot; EM |
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
| B | 2026-10-27 | Mechanical design (CDR design completion) | Chair follow-up: wing hinge designed (two OD 4.0 mm torsion springs, clock-spring flex in the −Z knuckle, damped stop, no latch; `hinge.py`), MCAD-01 and STR-13 closed by design with verification items; protrusion stack now governed by the springs (6.10 / 6.38 mm); skeleton torquer deck and row-5 re-split (MCAD-02 closed by reallocation); two centre supports at (0, ±21) mm adopted (chair decision) with 1.6 mm boards; Jetson to y = −6 mm; Jetson carrier item M1 (spreader pedestal, gap pads, module standoffs, (0, −21) fastener, shims and thermal path; §4.5, VI-MCAD-06; gyro and window moved 3 mm toward +Y to clear the module standoff); gyro (OBC item C3) through a framed window in the carrier (§4.3, D-11) after the carrier raise and the bracket were shown not to fit, OBC placement constraints recorded (§4.4), gyro alignment item VI-MCAD-05; Steinberg allowable made thickness-dependent (2.0 mm reserve +0.41 to +0.58); mass properties, interference and FE results recomputed |
