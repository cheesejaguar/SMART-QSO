# SMART-QSO Mechanical Design: CAD Model and Structural Finite-Element Model

**Document ID**: SQSO-MECH-CAD-001
**Version**: A
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
| Rail width to the first protrusion | 9.0 mm (wing free edge, body panels), 9.4 mm (wing hinge edge), 9.5 mm (hinge knuckle) | ≥ 8.5 mm (CDS §2.2.5) | §6.1 |
| Stowed protrusion (STR-10) | 5.85 mm nominal (hinge knuckle); **6.13 mm worst-case tolerance stack, 6.00 mm RSS** | ≤ 6.5 mm (CDS §2.2.3) | §6.2 |
| AntS below the −Z rail ends (STR-11) | 1.25 mm nominal, 0.90 mm worst case, with the −Z face recessed 1.5 mm (unrecessed: 0.25 mm **beyond** the rail ends) | ≥ 0.1 mm (CDS App. B) | §6.3 |
| GNSS patch below the +Z rail ends | 1.75 mm nominal, 1.40 mm worst case | ≥ 0.1 mm | §6.3 |
| Deployment switches, separation plungers, RBF | DS1, DS2 at −Z and DS3 at +Z on three rails; SP1, SP2 at −Z; all flush when actuated; RBF head 5.10 mm from the rail plane | CDS §2.3.2.3, §2.2.14, §2.3.5.2; BASELINE §4 | §6.4 |
| Interference, stowed and deployed | None (259 / 253 solid pairs with overlapping boxes tested by boolean intersection) | — | §6.5 |
| Wing sweep (0–90°) | 0.35 mm minimum clearance (at 45°, wing back cells to body-panel coverglass) | > 0.3 mm allowance | §6.6 |
| AntS tapes to the stowed wings (STR-16) | 6.5 mm (tape band to the wing's −Z edge, wing shortened and moved +3 mm); 0.5 mm with the 98 mm wing centred; deployed 2 m tape to the deployed wing 37.8 mm | ≥ 10 mm or wings first | §6.7 |
| Hinge springs and flex jumper of SQSO-MECH-001 | **Do not fit the hinge envelope** (spring radius 2.28 mm reaches 6.38 mm from the rail plane and y = 41.03 mm; 12 mm long against 5 mm free); flex bend radius 1.25 mm available against the R ≥ 5 mm assumed | Envelope §6.2; MECH-001 §3, §4.2 | §6.8 |
| Mass, CG | 1,492.7 g CBE / 1,725.9 g MEV (= MEL); stowed CG (+0.4, −0.3, −1.7) mm CBE | ≤ 2.00 kg; ±20 mm (5 mm baseline) | §5 |
| Deployed inertia (STR-01) | Iyy/max(Ixx, Izz) **1.057 CBE / 1.047 MEV**; major axis **0.3° / 1.3°** from +Y | ≥ 1.03; ≤ 5° (BASELINE §4) | §5 |
| First natural frequency, stowed (FE) | **170 Hz** (stowed wings, out of plane); deck 197, carrier 325, RF 385, EPS 445, OBC 549, battery 1,288 Hz | > 40 Hz (UG v11 §4.1.1) | §8.2 |
| Margins of safety at the design limit load (larger of the quasi-static 10 g / 17 g and the 3σ random response at 8.18 Grms), NASA-STD-5001B protoflight factors | All positive. Lowest: **+0.72** yield (−Z end frame at the battery isolators), +1.22 (end frames at the rail corners), +1.62 (centre support), +2.17 (rails), +2.43 (stack columns); G-10CR isolator +4.6 (FS 2.0) | MS ≥ 0 | §8.5 |
| PCB fatigue (Steinberg, FE relative displacement, 3σ at 8.18 Grms) | **+0.09 to +0.32** with the second centre support (D-7); −0.10 (EPS) and −0.29 (carrier) with one | MS ≥ 0 | §8.5 |

Verdict: **the mechanical design is complete at envelope level and meets the CDS envelope, the mass-properties
requirements and the structural requirements with positive margins**, with the design changes of §4.2. It is not yet a
drawing package: the bought frame, panels, wings and AntS have no vendor ICD in hand, the wing hinge mechanism does not
yet fit its envelope (§6.8), and the 5° principal-axis requirement depends on placement to about 2 mm (§5.4).

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
| `model.py` | Parameters (dimensions, tolerances), every part as a solid (stowed and deployed), MEL mass assignment, mass properties, checks, STEP and SVG export. `python3 hardware/mechanical/model.py` in the CadQuery environment; about 10 s |
| `results.json` | Parameters, the part list (87 parts: name, MEL rows, mass, centroid, volume), mass properties, every check with its value |
| `smartqso_stowed.step`, `smartqso_deployed.step` | Full assemblies, STEP AP214, 132 solids, 2.5 MB each |
| `smartqso_oml_stowed.step`, `smartqso_oml_deployed.step` | Outer mould line and dispenser interfaces only (rails, end frames, panels, wings, hinges, AntS, GNSS patch, switches, plungers, RBF, access port; 51 solids), 1.1 MB each: the form SpaceX UG v11 Appendix C asks for (no internal components) |
| `drawing_stowed_3view.svg` | Sheet 1: three views of the stowed spacecraft (from +X, +Y, +Z) with the CDS envelope, rail dimensions, the protrusion envelope and the stack, the AntS and patch standoff clearances, and the switch, plunger, RBF and access-port locations |
| `drawing_deployed.svg` | Sheet 2: deployed configuration from −Z and from +Y, wing and antenna extents, deployed inertia |
| `fem.py`, `fem_results.json` | FE model, its self-test, five load-and-boundary cases (§8) |

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
| Epson M-G370PDT0 | 24 × 24 × 10 mm | BUILD_VS_BUY §5.2 |
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
| −16.5 … −15.5 | Torquer deck, Al 1.0 mm, on the stack columns | Columns |
| −15.5 … −5.0 | X rod at (0, +36); GNSS receiver at (+5, −20) | Deck |
| −4.0 … +3.6 | EPS board (substrate −3.0 … −1.4) | Columns, centre supports at (0, ±21) |
| +4.0 … +16.6 | OBC (substrate +5.0 … +6.6) with the M-G370PDT0 at (−20, +22) | Columns, centre supports at (0, ±21) |
| +17.0 … +21.6 | Jetson carrier (substrate +18.0 … +19.6) | Columns, centre supports at (0, ±21) |
| +19.6 … +27.8 | SO-DIMM on the −Y edge, Jetson module centred at (0, −7), clamp at both short edges, TIM | Carrier |
| +27.8 … +32.8 | Heat spreader 80 × 60 × 5 mm at (0, −4), flanges 3 mm to the ±X shear panels (y −20 … +20) | 4 × M2.5 to the shear panels |
| +33.3 … +43.4 | RF board (substrate +34.8 … +36.4) with an 84 × 54 × 7 mm shield; ≤ 2 mm parts outside it | Columns, centre support (0, 0) to the spreader |
| +40.4 … +48.4 | Magnetometer at (+24, +27); GNSS patch adapter at (0, 0) | +Z end frame |
| +48.4 … +50.0 | +Z end frame, 1.6 mm, 30 × 30 mm cut-out | Rails |
| +50.0 … +55.0 | GNSS patch | +Z end frame |
| ±X, ±Y faces | Shear panels 1.0 mm (outer face 0.5 mm below the rail plane), body panels; wings over ±X; harness in four 3 × 10 mm bundles in the ±X wall gaps at y = ±(24 … 34) | Frame |

Stack columns: four M3 A286 studs in Al hex spacers (5 AF) at (±40, ±43), from end frame to end frame (STR-06). Centre
supports (STR-05): (0, +21) deck → EPS → OBC → carrier → spreader; (0, −21) EPS → OBC → carrier (added after the FE, §8.5);
(0, 0) spreader → RF. The spreader is bolted to the ±X shear panels at mid-height, so it is the stiff anchor of the
upper stack.

### 4.2 Design Decisions Made Here (changes to the reference layout and the other documents)

| # | Decision | Reason |
|---|----------|--------|
| D-1 | −Z end-frame outer face recessed 1.5 mm (frame RFQ: ≥ 0.8 mm) | AntS 7.0 mm otherwise stands 0.25 mm beyond the rail ends (STR-11) |
| D-2 | Side shear-panel outer face 0.5 mm below the rail plane (frame RFQ) | Protrusion stack (STR-10): without it the worst case is 6.63 mm |
| D-3 | Wing 81.6 × 92 mm (was 83 × 98), z −43 … +49; cell block 80.7 × 80.15 mm with 5.9 mm end zones | Rail-zone margin (CDS §2.2.5) and the AntS tape clearance (STR-16); end zones carry the hinges, the hold-down and the snubber |
| D-4 | Hinge axis (±54.1, +38.75) mm, knuckles Ø3.5 mm in the end zones (−Z: 5 mm long, +Z: 2.5 mm plus the flex cup) | Knuckle 1.0 mm inside the rail zone limit, 0.65 mm inside the protrusion limit; sweep clear of the body cells |
| D-5 | Hold-down: cup/cone post at the −Z free corner (y −38.5, z −40) with the line over it; silicone snubber at the +Z free corner | No free strip at the free edge: the cells fill the wing width |
| D-6 | Jetson module and spreader centred at y = −7 mm, SO-DIMM on the −Y edge; RBF and access port on −Y in the −Z end zone; harness in four symmetric bundles | Cancels Iyz (§5.3) |
| D-7 | Second centre support at (0, −21) mm, EPS → OBC → carrier | Steinberg margins of the EPS and carrier boards (§8.5) |
| D-8 | Torquer deck (Al 1.0 mm, lightened to ≤ 6 g) for the X rod and the GNSS receiver; Y and Z rods beside the battery | The 47 mm rods and the receiver have no other place in the 94.5 mm between the end frames |
| D-9 | Two 5 g trim-mass positions on the +Z end frame at y = ±45 mm | §5.4 |

## 5. Mass Properties

### 5.1 Mass

Every MEL row (MASS_BUDGET v3.0 §3.1, rods option) is carried by named solids; the CAD total is the MEL total: **1,492.7 g
CBE, 1,725.9 g MEV**, margin +15.9 % to 2.00 kg. By group (CBE): stack 439.0 g, battery 267.0 g, body panels 160.0 g,
frame 125.0 g (row 1 plus brackets and deck), wings 110.2 g, harness 105.0 g, AntS 85.0 g, ADCS 79.5 g, distributed
55.0 g, GNSS 52.0 g, inhibits 15.0 g.

### 5.2 CG and Inertia (about the CG, ×10⁻³ kg·m², tensor convention)

| Case | CG (mm) | Ixx | Iyy | Izz | Ixy | Ixz | Iyz | Iyy/max(Ixx, Izz) | Major axis from +Y |
|------|---------|-----|-----|-----|-----|-----|-----|-------------------|--------------------|
| Stowed, CBE | (+0.43, −0.30, −1.69) | 2.706 | 3.073 | 2.795 | +0.001 | +0.001 | +0.027 | 1.099 | 5.6° |
| Stowed, MEV | (+0.46, −0.36, −0.53) | 3.122 | 3.548 | 3.271 | +0.001 | −0.001 | +0.031 | 1.084 | 6.4° |
| **Deployed, CBE** | (+0.84, +1.97, −1.69) | **2.772** | **3.956** | **3.744** | +0.004 | +0.032 | −0.001 | **1.057** | **0.3°** |
| **Deployed, MEV** | (+0.84, +1.82, −0.53) | **3.195** | **4.515** | **4.311** | +0.004 | +0.033 | +0.004 | **1.047** | **1.3°** |
| W1 deployed only, CBE | (+2.06, +0.93, −1.69) | 2.742 | 3.655 | 3.413 | −0.165 | +0.021 | +0.040 | 1.071 | 12.1° |
| Antennas out, wings stowed, CBE | (+0.84, −0.26, −1.69) | 2.703 | 3.366 | 3.085 | −0.003 | +0.032 | +0.050 | 1.091 | 9.7° |
| Deployed, CBE, MEL hybrid antennas (2 m ±X, 70 cm ±Y) | (+0.43, +1.98, −1.69) | 2.810 | 4.209 | 4.035 | +0.003 | +0.001 | −0.001 | 1.043 | 0.4° |

Against MASS_BUDGET v3.0 §5–§6 (reference layout B): the stowed CG is 3 mm higher in Z (−1.7 against −4.7 mm: the CAD
battery centre is −32.9 mm, not −35, and the boards sit higher), the deployed moments are 3 % lower on every axis, and
the ratio agrees (hybrid antennas: 1.043 CAD against 1.041; option A: 1.057 against 1.054). The difference that matters
is the principal axis: 0.3–1.3° here against 13–15° there (§5.3). The baseline requirement (Iyy ≥ 1.03 × max, axis
within 5°) is **met** at CBE and MEV for the full deployment; with one wing or no wing out the axis is 10–12° off (as in
MECHANISMS_ANALYSIS §8, a degraded mode).

### 5.3 Balancing the Principal Axis

Iyy − Izz is 0.21 × 10⁻³ kg·m², so the axis tilt is θ = ½ atan(2 Iyz / (Iyy − Izz)): 4 × 10⁻⁵ kg·m² of Iyz tilts it by
13°. The deployed wings move the CG +2.0 mm in Y; every part far from the CG in Z then adds to Iyz. The first CAD
placement gave 17°; moving the Jetson module, clamp and spreader to y = −7 mm (with the SO-DIMM on the −Y edge and the
upper centre support at y = +21 mm), the RBF and access port to −Y and the harness into four symmetric bundles cancels it
without ballast (MASS_BUDGET v3.1 §6.6).

### 5.4 Sensitivity and Trim

A placement Monte Carlo (every internal part moved by an independent Gaussian error σ per axis, 2,000 trials) gives the
axis within 5° in 99.8 % / 97.6 % of trials (CBE / MEV) at σ = 1 mm, 84.9 % / 75.5 % at 2 mm, and 42.9 % / 38.7 % at
5 mm; the ratio stays ≥ 1.03 to about σ = 3 mm. The requirement therefore needs drawing-controlled positions and a
measured flight unit. Provision D-9: two 5 g trim masses at (0, ±45, +44) mm on the +Z end frame; each changes Iyz by
1.1 × 10⁻⁵ kg·m² (about 3° of tilt), set from the measured stowed tensor (VI-MCAD-03). Proposed flight constant
`INERTIA` (diagonal, mean of CBE and MEV): **(0.0030, 0.0042, 0.0040) kg·m²** (MASS_BUDGET v3.1 §6.5, for the chair).

## 6. Envelope, Interference and Clearance Checks

All checks are computed by `model.py` from the solids; `results.json` holds the values.

### 6.1 Envelope and Rails (CDS Rev 14.1)

| CDS item | Requirement | CAD | Status |
|----------|-------------|-----|--------|
| App. B, §2.2.1.1 | 100.0 ± 0.1 × 100.0 ± 0.1 mm rails, 113.5 ± 0.1 mm | 100.0 × 100.0 × 113.5 mm (rail solids' extents) | Met (nominal); vendor drawing and VI-LV-01 |
| §2.2.5 | ≥ 8.5 mm from the rail edge to the first protrusion | 9.0 mm (body panels, wing free edge), 9.4 mm (wing hinge edge), 9.5 mm (knuckle) | Met |
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
| Hinge knuckle, axis x = 54.1, r = 1.75 (on the frame-side leaf) | — | **5.85** | axis 0.10, radius 0.03 |

| Chain | Nominal | Worst case | RSS | Margin to 6.5 (worst / RSS) |
|-------|---------|------------|-----|------------------------------|
| Wing front cells | 4.75 | 5.57 | 5.08 | +0.93 / +1.42 |
| Hold-down line | 5.25 | 5.55 | 5.41 | +0.95 / +1.09 |
| **Hinge knuckle (governs)** | **5.85** | **6.13** | **6.00** | **+0.37 / +0.50** |
| Same design without the 0.5 mm recess (D-2) | 6.35 | 6.63 | 6.50 | −0.13 / −0.00 |
| SQSO-SA-001 A §13.2 stack (reference) | 6.1 | 6.87 | 6.40 | −0.37 / +0.10 |

The knuckle and the line are located from the frame (shear panel), not through the panel stack, so their chains are
short. **STR-10 is closed by design**, conditional on the frame RFQ items D-1 and D-2 and the vendor tolerances above
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

Boolean intersection of every pair of solids whose bounding boxes overlap: none stowed (259 pairs) and none deployed
(253 pairs). The only exclusions are the stowed tapes wound inside the AntS housing (by design) and the distributed-mass
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
| Tape sweep during release, deployed wings | Open: the element root and sweep are in the ISIS ICD (not obtained) | Order the element roots on the −Y half so that the sweep moves away from the wings |

With the baseline order (wings first, BASELINE §4 and §7) the clearance in the nominal sequence is 37.8 mm. The ≥ 10 mm
of MECHANISMS_ANALYSIS §7.2 is **not** met in the wing-stuck contingency (6.5 mm), and cannot be with an 80 mm cell block,
two hinge end zones and a 100 mm face (best achievable about 7 mm). The tape's stiff direction is the vertical one (a
5 mm tape on edge), so 6.5 mm against a static tape is a low risk. **STR-16: closed for the nominal sequence by the
order, verification item for the contingency (VI-STR-10, with the ISIS ICD and the deployment test)**.

### 6.8 Hinges, Springs and the Harness past the Hinges (finding for SQSO-MECH-001)

The hinge line is fixed by two limits: the knuckle must stay within y ≤ 41.5 mm (rail zone) and x ≤ 56.5 mm (protrusion),
and the cell block (80.7 mm) fills the wing width, so the hinge hardware can only sit in the two 5.9 mm end zones.

| Item | MECH-001 | Fits? | CAD allocation |
|------|----------|-------|----------------|
| Knuckles | Ø1.5 mm pins | Yes: Ø3.5 mm knuckles, 5 mm (−Z) and 2.5 mm (+Z) long | Modelled |
| Torsion springs | 17-7PH Ø0.55 mm, mean Ø4 mm, ≈ 22 coils (12 mm long), 10 / 5 mN·m | **No**: radius 2.28 mm reaches 6.38 mm from the rail plane and y = 41.03 mm (rail zone); 12 mm length against 5 mm free | None |
| Spring that fits (Ø0.45 mm, mean Ø2.9 mm, 8 coils, 5.4 mN·m/rad) | — | Envelope yes; stress 1.39 GPa at the 11 mN·m stowed torque against 1.0 GPa allowable (1,500 / 1.5): **no** | — |
| Hinge flex jumper | Polyimide flex, R ≥ 5 mm, ≤ 1.5 mN·m | Only as a clock-spring wrap in a Ø3.5 × 2.3 mm cup at the +Z hinge: bend radius ≈ 1.25 mm | Modelled (cup and tail) |

**Finding MCAD-01 (severity 2, for the chair: MECH-001 owner and the panel vendor):** the wing deployment springs of
MECHANISMS_ANALYSIS §3 do not fit the envelope that CDS §2.2.3 and §2.2.5 leave at the hinge line, and the hinge flex
cannot have the R ≥ 5 mm its torque estimate assumes (at R ≈ 1.25 mm the jumper torque, which scales about as 1/R, could
reach 6 mN·m and the cold end-of-travel torque margin of MECH-001 §4.2, +0.29, would go negative). Options, to be
traded by the mechanisms owner with the panel vendor: (a) a flat leaf (kick-off) spring acting on the wing back face in
an end zone, with the hinge pins only guiding; (b) springs inside the frame driving the hinge through a pin extension at
the ±Z end frames (needs a hinge-pin bearing in the end frame); (c) narrower cells (2 × 39 mm) to free a 3 mm hinge strip;
(d) the vendor's own hinge (DHV, ISIS, Spacemanic build custom deployables, BUILD_VS_BUY §2.2), shown to fit the
envelopes of this CAD. For the flex: a clock-spring flex with a measured torque at −40 °C (VI-STR-04), or wiring through
the hinge pin. Until then the CAD carries envelopes only, and STR-13 (flex torque) is reopened.

### 6.9 Harness Routing

Four 3 × 10 mm bundles in the ±X wall gaps (x = ±45.4 … ±48.4, between the board edges and the shear panels) carry the
MEL harness (row 32), the GNSS coax (row 31) and the sensor cables (row 26); they are clear of the columns, the spreader
flanges, the hinge doublers and the burn-resistor boards. Past the hinges: the flex enters the body through the
body-panel notch at the +Z hinge end zone (z 46.5–48.4, y 37.0–37.5), inside the 6.5 mm envelope (5.85 mm) and outside
the rail zone (y ≤ 40.5). The battery flying leads (≥ 100 mm) run from the sense board to the EPS through the −X bundle
gap. Wire counts and connector positions are the harness design's (`hardware/harness/HARNESS_DESIGN.md`).

### 6.10 Jetson Heat Path

Geometry: SoC → TIM pad (40 × 28 mm, 0.5 mm, 3 W/m·K: 6.7 W/K) → spreader (5 mm Al, 60 mm wide, ≈ 40 mm to each flange
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
| Flat 4-node shell: MITC4 plate (assumed transverse shear, no locking), Wilson incompatible-mode membrane, drilling stiffness 10⁻⁴ E t A | −Z and +Z end frames (Al 1.6 mm, cut-outs 71 × 71 and 30 × 30 mm), four side shear panels (Al 1.0 mm), EPS, OBC, carrier and RF boards (FR-4 1.6 mm, E 18.6 GPa), torquer deck (Al 1.0 mm lightened to 6 g: equivalent plate with E × 0.35), heat spreader (Al 5 mm) and flanges (3 mm), stowed wings (FR-4 1.6 mm) | Element mass plus non-structural mass for the components, cells, body panels, AntS, harness and distributed items |
| 3D Euler–Bernoulli beam, consistent mass | Rails (L 8.5 × 8.5 × 1.5 mm between the end frames, solid 8.5 × 8.5 mm standoffs); stack columns and centre supports (Al hex 5 AF + M3 A286 stud, bending × 1/3 for the threaded joints as SQSO-SA-001 A §5); battery isolators (G-10CR Ø6/Ø2.7 mm bushing with the Ti-6Al-4V M2.5 screw in bending; axial and bending × 0.6 for joint compliance, scaled to the 8 mm bushing); clamp ears (Al 10 × 2.5 mm) | — |
| Translational springs | Wing hinges (pins: no moment; 2 × 10⁸ N/m normal, 5 × 10⁶ in plane), hold-down cup/cone post (2 × 10⁸ / 2 × 10⁶), snubber (silicone, 2.2 × 10⁴ N/m) | — |
| Rigid links | Shear-panel edges and end-frame corner squares to the rails; the battery (261 g rigid body, inertia from the CAD solids) to its ears; **spiders** spreading each column, centre-support and isolator attachment over the nodes within 2.6 mm (the spacer and washer face; 224 nodes) | — |

Boundary condition, as in the dispenser: every rail node fixed in x and y (the guide rails), the rail-end nodes fixed in z
(pusher plate and door), rotations free. Sensitivity: the eight rail ends fixed only (§8.6). Mesh: 10,154 nodes, 9,400
shells, 192 beams, 56,258 degrees of freedom; FE mass 1,492.7 g = CAD = MEL CBE. MEV masses are not run (+15.6 % mass
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
| 1, 2 | 170.3 | 0 / 0 / 0; 5.5 / 0 / 0 | Stowed wings W1, W2, out of plane (in and out of phase) |
| 3 | 196.8 | 0 / 0 / 1.3 | Torquer deck (X rod, GNSS receiver, Z-rod clip) |
| 4 | 276.4 | 0 / 0 / 1.3 | Deck, second mode |
| 5 | 324.8 | 0 / 0 / **17.0** | Carrier (0.55), EPS, OBC: stack drumming in Z |
| 6 | 328.9 | **32.0** / 0 / 0 | EPS, OBC, carrier: stack lateral in X |
| 7, 8 | 341.4 | 0 / 0 / 0 | Wings, second mode |
| 9 | 384.8 | 0.1 / 0 / 0 | RF board |
| 11 | 398.4 | 0 / **16.2** / 0 | EPS, RF, OBC: stack lateral in Y |
| 12 | 409.9 | 0 / 5.8 / 0 | RF board, stack |
| 20 | 445.4 | — | EPS board |
| 25, 27, 28, 29, 32 | 518, 546, 549, 612, 630 | small | Spreader; +Z end frame; OBC; ±X and ±Y shear panels |
| 57 | 971 | 0 / 0 / 4.6 | Battery on its isolators, axial |
| 100 | 1,288 | **18.1** / 0 / 0 | Battery on its isolators, lateral |
| 124 | 1,526 | — | −Z end frame with the AntS |

450 modes, to 3,827 Hz. Cumulative effective mass to 2,000 Hz: 77 % (x), 57 % (y), 66 % (z) of the free mass; the rest is
the frame and panel mass next to the held rails, which moves with the base. First mode 170 Hz: **4.3 times the 40 Hz
requirement**; no mode below 150 Hz, so the spacecraft is rigid in the launch-vehicle coupled-loads band (UG Appendix B).

### 8.3 Against the Analytic Values of SQSO-SA-001 A §5

| Element | SA-001 A (closed form) | FE | Comment |
|---------|------------------------|----|---------|
| Stowed wing | 142 Hz (four points: two hinges, hold-down, bumper) | 170 Hz (three points and a soft snubber; 183 Hz with a second hard post) | Shorter wing (92 mm) and the hold-down at the corner |
| PCB stack, lateral, tied at both ends | 289 Hz | 329 Hz (X), 398 Hz (Y) | The spreader bolted to the walls at mid-height stiffens the stack; Y stiffer through the flanges |
| Boards with a centre support | 221 (RF) – 362 (OBC) Hz | carrier 325, RF 385, EPS 445, OBC 549 Hz (in the stack) | Two centre supports (D-7); boards coupled |
| Battery on G-10CR isolators, lateral | 1,280 Hz | 1,288 Hz (944 Hz with the rails held at the ends only) | Agreement |
| Deck (new part) | — | 197 Hz | Lowest internal mode |

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
  BGA, r = 1, L = 1 in), the SQSO-SA-001 A §8 criterion.

### 8.5 Margins of Safety (design case)

| Location | QS (MPa) | 3σ random (MPa) | Allowable | FS | MS yield | MS ultimate | Where |
|----------|----------|-----------------|-----------|----|----------|-------------|-------|
| −Z end frame at the battery isolators | 10.7 | **111.8** | 241 / 290 | 1.25 / 1.40 | **+0.72** | +0.85 | (−40.8, −32.5, −47.7): next to the isolator spider |
| End frames at the rail corners | 25.4 | 87.0 | 241 / 290 | 1.25 / 1.40 | +1.22 | +1.38 | +Z frame at (40.8, 41.2) |
| Centre supports | 30.4 | 73.5 | 241 / 290 | 1.25 / 1.40 | +1.62 | +1.82 | (0, +21) under the spreader |
| Side shear panels | 16.2 | 67.0 | 241 / 290 | 1.25 / 1.40 | +1.88 | +2.09 | −X panel at the spreader flange |
| Rails (L-angle, standoffs) | 14.7 | 60.9 | 241 / 290 | 1.25 / 1.40 | +2.17 | +2.40 | (−47.3, −47.3) at the −Z end frame |
| Stack columns, Al spacer | 16.9 | 56.2 | 241 / 290 | 1.25 / 1.40 | +2.43 | +2.68 | (−40, −43) under the +Z end frame |
| Stack columns, A286 stud thread root | 20.4 | 67.9 | 758 / 1,103 | 1.25 / 1.40 | +7.9 | +10.6 | Same |
| Battery isolator, G-10CR bushing | 6.8 | 34.1 | 380 (flexural) | 2.00 | — | **+4.57** | (−38.5, +31) |
| Battery isolator, Ti-6Al-4V screw | 15.2 | 76.2 | 827 / 896 | 1.25 / 1.40 | +7.7 | +7.4 | Same |
| Battery clamp ears (Al) | 5.0 | 23.0 | 241 / 290 | 1.25 / 1.40 | +7.4 | +8.0 | Ear root |
| Heat spreader and flanges | 7.4 | 21.3 | 241 / 290 | 1.25 / 1.40 | +8.1 | +8.7 | At the upper centre support |
| Torquer deck (equivalent plate) | 7.6 | 34.6 | 241 / 290 | 1.25 / 1.40 | +4.6 | +5.0 | Near the column (−40, −43) |
| PCBs EPS / OBC / carrier / RF (FR-4) | 3.6 / 4.1 / 5.6 / 5.4 | 29.9 / 25.4 / 32.3 / 19.4 | 340 | 1.50 | — | +6.6 / +7.9 / +6.0 / +10.7 | At the columns |
| Stowed wings (FR-4) | 1.7 | 6.4 | 340 | 1.50 | — | +34 | Centre |

| Board | Z₃σ relative (mm) | Steinberg MS, design (two centre supports) | One centre support (SA-001 A design) | Design with 2.0 mm EPS and carrier |
|-------|--------------------|---------------------------------------------|--------------------------------------|------------------------------------|
| EPS | 0.169 | **+0.14** | −0.10 | +0.87 |
| OBC | 0.169 | **+0.14** | +1.16 | +0.87 |
| Jetson carrier (with the module) | 0.176 | **+0.09** | −0.29 | +0.80 |
| RF | 0.145 | **+0.32** | +0.20 | +0.39 |
| Deck (Al, no BGA: criterion for information) | 0.350 | (−0.45) | (−0.46) | (−0.62) |

Hold-down: the post carries 2.0 N quasi-static and **3.9 N 3σ** normal to the wing (SA-001 A §10: 4.9 N); against the
30 N line preload after a 30 % loss (21 N) with 1.15 × 1.20 on the load, separation MS = 21 / (1.38 × 3.9) − 1 = **+2.9**.
Hinges 3.9 N, snubber 3.7 N (3σ).

Findings from the FE:

1. **All strength margins are positive** at the protoflight test level with the 5001B factors. The lowest, +0.72, is a
   local peak in the 1.6 mm −Z end frame beside the isolator attachment (battery 261 g on four points of a ring 13.5 mm
   wide); it is mesh- and spider-dependent. **Design action:** a 1 mm Al doubler (or a 2.5 mm local boss) under each
   isolator on the −Z end frame, specified on the frame drawing or the battery-mount drawing.
2. **The PCB fatigue margins need the second centre support (D-7):** with the single support of SQSO-SA-001 A at the only
   place the Jetson module leaves free (y = +21 mm), the EPS and carrier boards are at −0.10 and −0.29; with the second
   support at y = −21 mm (EPS → OBC → carrier) they are +0.14 and +0.09. The 2.0 mm boards raise them to +0.8 to +0.9 at
   0.8 mm of stack height (available above the RF board, at a re-stack). Steinberg's 2 × 10⁷-cycle criterion is
   conservative for one minute per axis; the measured Q of the EM board survey (VI-STR-02) decides whether the 2.0 mm
   option is needed.
3. **The deck** (a part this design adds) is the lowest internal mode (197 Hz) and moves 0.35 mm (3σ) relative to its
   supports; its strength margin is +4.6. Its payload (GNSS receiver, X rod) is not a PCB on the deck, but the receiver's
   own board sees the deck motion: if the receiver vendor's random qualification is below the deck response, stiffen the
   deck (1.6 mm with ribs, about +10 g) or move the receiver so that the second centre support can start at the deck
   (the deck-to-carrier variant gave the deck 206 Hz). The deck must not be tied to the battery clamp, which is
   isolated.

### 8.6 Sensitivity Cases

| Case | First mode | Lowest MS (where) | Steinberg range | Note |
|------|------------|-------------------|-----------------|------|
| Design (D-7, rails held) | 170 Hz | +0.72 (−Z frame at the isolators) | +0.09 … +0.32 | Baseline |
| Single centre support | 170 Hz | +0.75 | **−0.29 … +1.16** | Rejected (D-7) |
| D-7 with 2.0 mm EPS and carrier | 170 Hz | +0.72 | +0.39 … +0.87 | Option |
| D-7 with a second hard post at the +Z free corner | 183 Hz (wing) | +0.73 | +0.09 … +0.32 | Wing +13 Hz; not needed |
| D-7, rails held at the eight rail ends only | 170 Hz | **+0.82** (rails, 105.9 MPa), +0.81 (−Z frame), +1.35 (centre support) | +0.09 … +0.33 | Bounds the dispenser contact; rails then carry the lateral load in bending |

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
| STR-01 (check) | **Confirmed and closed by design**: CAD gives Iyy/max 1.057 / 1.047 and the major axis within 0.3° / 1.3° of +Y (the reference layout gave 13–15°, outside the 5° baseline); placement sensitivity and the trim provision are verification items (VI-MCAD-03) | §5 |
| STR-10 | **Closed by design**: 5.85 mm nominal, 6.13 mm worst case (D-2 recess, frame-located knuckle and post) | §6.2 |
| STR-11 | **Closed by design** (frame RFQ: −Z face recessed ≥ 0.8 mm; 1.5 mm modelled): 1.25 mm nominal, 0.90 mm worst case | §6.3 |
| STR-16 | **Closed for the nominal sequence** (wings first: 37.8 mm); **verification item** for the wing-stuck contingency (6.5 mm against 10 mm) and the ISIS sweep | §6.7 |
| MCAD-01 (new, severity 2) | **For the chair**: hinge springs and flex do not fit the hinge envelope | §6.8 |
| MCAD-02 (new, severity 3) | **For the chair**: MEL row 5 does not cover the torquer deck (23 g solid, 6 g allotted) | §5.1, MASS_BUDGET v3.1 §1.2 |
| MCAD-03 (new, severity 3) | Wing sweep clearance 0.35 mm: body-panel cell layout action for the panel vendor | §6.6 |

## 11. Interface Changes for Other Documents (for the chair)

1. **Frame RFQ (BUILD_VS_BUY §2.2, STRUCTURAL_DESIGN §2.1):** −Z mounting face recessed ≥ 0.8 mm (1.5 mm modelled) for
   the AntS; side shear-panel outer faces ≥ 0.5 mm below the rail plane; switch provisions (Ø4.5 mm, 12 mm deep) at the
   five standoff positions of §6.4; inserts for the spreader flanges at mid-height of the ±X walls.
2. **Wing outline (SOLAR_ARRAY_DESIGN, panel vendor):** 81.6 × 92 mm, z −43 … +49 mm, cell block 80.7 × 80.15 mm centred
   at z = +3 mm, end zones for the hinges, the hold-down post and the snubber; body-panel cells offset 1 mm toward −Y.
3. **Mechanisms (MECHANISMS_ANALYSIS):** MCAD-01 (springs and flex), the hold-down at the −Z free corner with a cup/cone
   post and a snubber at the +Z free corner (§8.4 gives the line load), hinge axis (±54.1, +38.75) mm.
4. **RF board (RF designer):** shield 84 × 54 × 7 mm at y −40 … +14 mm; parts ≤ 2 mm elsewhere on the top side
   (magnetometer above at (+24, +27)); bottom-side keep-outs Ø7 mm at the columns and at (0, 0).
5. **EPS, OBC and carrier boards:** PC/104 outline 90.17 × 95.89 mm; holes at (±40, ±43); centre supports at (0, +21)
   and (0, −21) (Ø7 mm keep-outs both sides); EPS notch x < −35.5, |y| < 6.5 mm for the Z rod; OBC: M-G370PDT0 at
   (−20, +22), with a carrier bottom-side keep-out above it; carrier: SO-DIMM on the −Y edge, module centred at y = −7,
   no top-side parts within the module footprint.
6. **ADCS:** rod positions and the flight inertia constant (§5.4); the magnetometer at the +Z end, 60–85 mm from the rods.
7. **RBF and access port** on −Y at x = ±20, z = −43 mm, to be matched to the Nova access window (LVR-068).
8. **MASS_BUDGET** (done here, v3.1) and **STRUCTURAL_ANALYSIS** (done here, B).

## 12. Verification Items

| ID | Verification | Pass criterion | Verifies | When |
|----|--------------|----------------|----------|------|
| VI-MCAD-01 | Re-run `model.py` with the vendors' STEP models (frame, AntS, panels, wings) in place of the envelopes | All checks of §6 pass; no interference | §6 | On receipt of each vendor model; drawing release |
| VI-MCAD-02 | First-article measurement of the stowed protrusions (CMM or the CIFP method) | ≤ 6.5 mm everywhere; knuckle ≤ 6.13 mm | STR-10 (with VI-STR-08) | Flight unit |
| VI-MCAD-03 | Mass, CG and full inertia tensor of the stowed flight unit (six-orientation pendulum or spin balance), trim masses set, deployed tensor from the CAD with the weighed wings | Iyy ≥ 1.03 × max(Ixx, Izz), major axis ≤ 5° from +Y | BASELINE §4 (with VI-STR-01) | Flight unit before environmental test |
| VI-MCAD-04 | EM low-level sine survey (0.25 g, 20–2000 Hz, three axes) with accelerometers on the wings, deck, each board centre and the battery; FE updated to the measured modes | First mode > 40 Hz; FE modes within 15 % (else the FE is correlated before the flight margins are reused) | §8 (with VI-STR-02, VI-LV-03) | EM |
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
