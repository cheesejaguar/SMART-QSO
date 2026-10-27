# Hardware

Hardware design of the SMART-QSO 1U CubeSat: board build-to designs, harness, mechanical CAD and structural model,
system drawings, and the earlier subsystem design notes. Licensed under **MIT** (`LICENSE` at the repository root).

**Status.** Design and analysis only. No board, harness or structure has been built, and there is no PCB layout and no
engineering model; every number is analysis until its verification item closes. `docs/BASELINE_PARAMETERS.md` governs
every design value; where a document here disagrees with it, the baseline wins. The current review record is
`docs/reviews/CDR/CDR_2026_DESIGN_REVIEW.md` (an engineering assessment, not an independent board).

## Contents

| Directory | What is in it | Governing document |
|-----------|---------------|--------------------|
| `electrical/` | Build-to designs of the four boards, each with its schematic as code (SKiDL `design.py`), generated netlist, BOM, nets and ERC log, and a calculation script with its results: EPS (`eps/EPS_BOARD_DESIGN.md`, SQSO-EPS-BRD-001), OBC with the magnetometer board A2 and the access-port board A3 (`obc/OBC_BOARD_DESIGN.md`, SQSO-OBC-BRD-001), RF (`rf/RF_BOARD_DESIGN.md`, SQSO-RF-BRD-001), Jetson carrier (`jetson/CARRIER_BOARD_DESIGN.md`, SQSO-JET-BRD-001); the master pin allocation and board-to-board signal lists (`INTERCONNECT.md`, SQSO-ICX-001) | these documents; interfaces also in `docs/interfaces/` |
| `harness/` | Harness design (`HARNESS_DESIGN.md`, SQSO-HAR-001), wire list `wire_list.csv`, routes `routes.csv`, `harness_calcs.py` and its results | SQSO-HAR-001 |
| `mechanical/` | CAD of the spacecraft (`model.py`, CadQuery), beam-shell FE model (`fem.py`), wing hinge design (`hinge.py`), placement Monte Carlo, their results, STEP exports and layout drawings (`MECHANICAL_DESIGN.md`, SQSO-MECH-CAD-001) | SQSO-MECH-CAD-001; `docs/environment/STRUCTURAL_ANALYSIS.md` |
| `diagrams/` | Block, wiring (WireViz) and schematic (schemdraw) drawings generated from text sources (`README.md`, SQSO-HW-DIAG-001) | the board designs and SQSO-ICX-001 |
| `eps/`, `obc/`, `rf/`, `adcs/`, `payload-jetson/`, `structure/`, `solar/` | Subsystem design notes from before the board build-to designs (battery, solar array, PDU, OBC architecture and storage, transmitter and antenna, ADCS sensors and actuators, Jetson integration and power, structure and thermal design, solar-panel trade study); where they disagree with the build-to designs above, the build-to designs govern | each directory's documents |

## Regenerating

```bash
# Calculation scripts (Python standard library; rf_calcs.py also needs numpy)
python3 hardware/electrical/eps/eps_calcs.py
python3 hardware/electrical/obc/obc_calcs.py
python3 hardware/electrical/rf/rf_calcs.py
python3 hardware/electrical/jetson/jetson_calcs.py
python3 hardware/harness/harness_calcs.py            # --cad needs CadQuery; --wireviz checks the WireViz sources

# Schematics, netlists, BOMs and ERC logs: SKiDL 3.0 in a separate environment ("CAD venv")
<CAD venv>/bin/python hardware/electrical/eps/design.py    # likewise obc/, rf/, jetson/

# CAD and FE: CadQuery 2.8 (model.py) and numpy/scipy (fem.py); see MECHANICAL_DESIGN.md §3.1
python3 hardware/mechanical/model.py
python3 hardware/mechanical/fem.py

# Drawings: graphviz, WireViz 0.4.1 and schemdraw 0.23 (hardware/diagrams/requirements.txt)
hardware/diagrams/build.sh
```

The scripts rewrite their generated files in place (netlists, BOMs, results JSON, SVG); commit the source and the
output together.
