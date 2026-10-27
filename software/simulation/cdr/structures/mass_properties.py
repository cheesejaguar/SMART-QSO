#!/usr/bin/env python3
"""
SMART-QSO CDR mass properties: bottom-up MEL, growth, margin, CG and inertia.

Analysis tool (software Class D). Results go into docs/environment/MASS_BUDGET.md
(SQSO-MB-001 Version 3.0) and docs/environment/STRUCTURAL_ANALYSIS.md.

Method
  * MEL rebuilt from the build-versus-buy selections (docs/hardware/BUILD_VS_BUY.md
    section 1-2) and BASELINE section 6 (GNSS).  Two magnetorquer options:
    "rods" (CubeSpace CR0002, BUILD_VS_BUY section 1) and "coils" (the 250-turn
    air-core coils of BASELINE section 8).
  * Mass growth allowance (MGA) per maturity code (ANSI/AIAA S-120A-2015 method;
    the percentages are this project's, see MATURITY below).
  * Each item is a set of primitives (box, bar, square ring) with a position in the
    CDS body frame: origin at the geometric centre, +Z along the rails, -Z inserted
    first, wings hinged on the +-X/+Y edges (hardware/eps/SOLAR_ARRAY_DESIGN.md 3.1).
  * The layout is the reference layout of STRUCTURAL_ANALYSIS.md section 3 (no CAD
    exists).  CG uncertainty: each item's position is varied by +-5 mm (1 sigma) per
    axis.

Usage
    python3 mass_properties.py            # print the tables
    python3 mass_properties.py --json out.json

Only numpy is needed.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import dataclass, field
from typing import Dict, List, Tuple

import numpy as np

# --------------------------------------------------------------------------------------
# Maturity codes and growth allowances (fraction)
# --------------------------------------------------------------------------------------
MATURITY = {
    "SPEC": (0.00, 0.00, "Published not-to-exceed mass"),
    "C1": (0.03, 0.02, "Vendor-published mass of a selected part, configuration fixed"),
    "C2": (None, None, "Vendor-published range, vendor or configuration open: CBE at the "
           "middle of the range, MGA up to the top of the range"),
    "E2": (0.15, 0.12, "Layout: calculated from assumed dimensions and materials"),
    "E1": (0.25, 0.18, "Estimate: no drawing, earlier estimate or similar item"),
}

CDS_LIMIT_G = 2000.0          # SYS-002; CDS Rev 14.1 Table 1 (a "typical maximum" note, 2.2.10)
NOVA_LIMIT_G = 2500.0         # EXOpod Nova User Manual Rev 1.2 (June 2024) Table 1, 1U
NRCSD_LIMIT_G = 2400.0        # BASELINE section 4 (ISS backup)
CG_LIMIT_MM = 20.0            # CDS Rev 14.1 Table 2 (1U, each axis); EXOpod Table 2


@dataclass
class Prim:
    """A mass primitive: kind 'box' (dims dx, dy, dz), 'bar' (axis 'x'|'y'|'z', length,
    diameter), 'ring' (square loop of four bars: normal axis, side, bar diameter)."""
    kind: str
    frac: float                      # fraction of the item mass
    c: Tuple[float, float, float]    # centre, mm
    dims: Tuple = ()
    axis: str = "z"


@dataclass
class Item:
    row: str
    name: str
    qty: str
    cbe_g: float
    maturity: str
    basis: str
    prims: List[Prim]
    mga: float | None = None          # explicit MGA for C2 items
    mga_low: float | None = None
    moves_with: str = ""             # "W1" or "W2": moves with a wing on deployment
    deployed_prims: List[Prim] = field(default_factory=list)  # replaces prims when deployed
    options: Tuple[str, ...] = ("rods", "coils")

    def growth(self, low: bool = False) -> float:
        hi, lo, _ = MATURITY[self.maturity]
        if self.maturity == "C2":
            return (self.mga_low if low else self.mga) or 0.0
        return lo if low else hi


def box(frac, c, d):
    return Prim("box", frac, c, d)


def bar(frac, c, axis, length, dia=3.0):
    return Prim("bar", frac, c, (length, dia), axis)


def ring(frac, c, axis, side, dia=4.0):
    return Prim("ring", frac, c, (side, dia), axis)


# --------------------------------------------------------------------------------------
# Reference layout (mm).  Body faces at +-50 (X, Y) and +-50 (Z faces; rails to +-56.75)
# --------------------------------------------------------------------------------------
R = 45.75   # rail centre line, X and Y
HINGE = (54.0, 39.5)   # wing hinge axis (x, y) for W1; W2 at (-54.0, 39.5)
WING = (83.0, 98.0, 2.2)  # wing: span from hinge (83), length along Z (98), thickness

def wing_prims(sign: float, deployed: bool, frac: float = 1.0) -> List[Prim]:
    hx, hy = HINGE
    if not deployed:   # lying over the +-X face, spanning y = -41.5 .. 41.5
        return [box(frac, (sign * hx, 0.0, 0.0), (WING[2], WING[0], WING[1]))]
    # rotated 90 deg about the hinge axis: spans outward in +-X in the plane y = hy
    return [box(frac, (sign * (hx + 39.5), hy, 0.0), (WING[0], WING[2], WING[1]))]


def items() -> List[Item]:
    L: List[Item] = []
    a = L.append
    # Structure ---------------------------------------------------------------------
    a(Item("1", "1U frame: rails, side frames, end frames (bought)", "1", 115.0, "C2",
           "Spacemanic 115 g, ISIS 108 g with secondary structure, EnduroSat 120 g (delisted), "
           "Pumpkin not stated (BUILD_VS_BUY 2.2). MGA to 120 g",
           [bar(15 / 115, (sx * R, sy * R, 0.0), "z", 113.5, 8.5) for sx in (1, -1) for sy in (1, -1)] +
           [ring(27.5 / 115, (0, 0, z), "z", 91.0, 6.0) for z in (-50.0, 50.0)],
           mga=0.044, mga_low=0.0))
    a(Item("2", "Deployment switches (3): two in -Z rail standoffs, one in a +Z standoff", "3", 6.0, "E1",
           "3 x 2 g roller-plunger switches; three switches give the three physical inhibits "
           "(BASELINE 4)", [box(1 / 3, (R, R, -54.0), (6, 6, 6)), box(1 / 3, (-R, -R, -54.0), (6, 6, 6)),
                            box(1 / 3, (R, -R, 54.0), (6, 6, 6))]))
    a(Item("3", "Separation spring plungers (2), -Z standoffs", "2", 3.0, "E1",
           "CDS 2.2.14 (shared dispenser): < 6.7 N, stroke > 2.5 mm",
           [box(0.5, (-R, R, -54.0), (5, 5, 8)), box(0.5, (R, -R, -54.0), (5, 5, 8))]))
    a(Item("4", "RBF pin and housing, -Y face", "1", 4.0, "E1", "Estimate",
           [box(1.0, (20.0, -47.0, -20.0), (10, 8, 10))]))
    a(Item("5", "Internal brackets (Jetson heat path, GNSS patch, rods, hinge doublers)", "1 set", 30.0,
           "E1", "Custom brackets on a bought frame (BUILD_VS_BUY 2.2); MB v2.0 had 20 g",
           [box(1.0, (0, 0, 0), (85, 85, 90))]))
    # Solar array ---------------------------------------------------------------------
    a(Item("6", "Body solar panels (4), vendor, 2 cells each, photodiode, thermistor", "4", 160.0, "C2",
           "Spacemanic Ra ~30 g, DHV < 50 g, ISIS 50 g per panel incl. cells (BUILD_VS_BUY 2.3): "
           "CBE 40 g, MGA to 50 g",
           [box(0.25, (0, 51.0, 0), (83, 2.0, 98)), box(0.25, (0, -51.0, 0), (83, 2.0, 98)),
            box(0.25, (51.0, 0, 0), (2.0, 83, 98)), box(0.25, (-51.0, 0, 0), (2.0, 83, 98))],
           mga=0.25, mga_low=0.10))
    for w, s in (("W1", 1.0), ("W2", -1.0)):
        a(Item("7" if w == "W1" else "8", f"Wing {w} panel: FR-4 1.6 mm substrate, photodiode, thermistor, "
               "connector, back-wiring (custom, panel vendor)", "1", 28.7, "E2",
               "83 x 98 x 1.6 mm FR-4 at 1.9 g/cm3 = 24.7 g + 4 g parts",
               wing_prims(s, False), moves_with=w, deployed_prims=wing_prims(s, True)))
    a(Item("9", "Solar cells on the wings, AZUR 3G30A (8)", "8", 28.8, "SPEC",
           "<= 3.6 g each, AZUR datasheet DB 00010891-01",
           wing_prims(1.0, False, 0.5) + wing_prims(-1.0, False, 0.5),
           deployed_prims=wing_prims(1.0, True, 0.5) + wing_prims(-1.0, True, 0.5)))
    for w, s in (("W1", 1.0), ("W2", -1.0)):
        hx, hy = HINGE
        a(Item("10" if w == "W1" else "11", f"Wing {w} hinges (2), torsion springs, hold-down, stop, "
               "deployed-state sensor, hinge flex harness", "1", 14.0, "E1",
               "Bottom-up: hinges 6 g, springs 1 g, hold-down 3 g, bumpers 1 g, sensor 1 g, flex 2 g",
               [box(8 / 14, (s * hx, hy, 0.0), (4, 4, 98)), box(6 / 14, (s * 54.0, -38.0, 0.0), (3, 6, 30))],
               deployed_prims=[box(8 / 14, (s * hx, hy, 0.0), (4, 4, 98)),
                               box(6 / 14, (s * (hx + 77.5), hy, 0.0), (6, 3, 30))]))
    # EPS and battery -----------------------------------------------------------------
    a(Item("12", "EPS board (MPPT, BMS, PDU, burn drivers), in-house", "1", 90.0, "E1",
           "MB v2.0 row 6 (board 60, connectors 20, protection 10)", [box(1.0, (0, 0, -38.0), (90, 96, 10))]))
    a(Item("13", "Battery cells, Samsung INR18650-35E (4S1P)", "4", 200.0, "SPEC",
           "<= 50.0 g each, Samsung SDI specification", [box(1.0, (0, 0, -20.0), (74, 65, 18.5))]))
    a(Item("14", "Battery interconnect / BMS-sense board", "1", 15.0, "E1", "MB v2.0 row 8",
           [box(1.0, (0, 0, -9.0), (78, 60, 3))]))
    a(Item("15", "Isolated battery mount: Al 6061 clamp bracket", "1", 34.0, "E2",
           "MB v2.0 4.3; BUILD_VS_BUY 4.3", [box(1.0, (0, 0, -20.0), (80, 71, 24))]))
    a(Item("16", "Isolated mount hardware: 4 G-10CR/PEEK bushings, 4 Ti M2.5 screws, disc springs, "
           "heater (2 elements), cutout, VDA tape, thermal pad", "1 set", 8.0, "E2",
           "Bushings 4 x 0.3 g, screws 4 x 0.4 g, disc springs 4 x 0.2 g, heaters 2 g, cutout 1.5 g, "
           "tape and pad 1 g", [box(1.0, (0, 0, -20.0), (80, 71, 24))]))
    # Avionics --------------------------------------------------------------------------
    a(Item("17", "OBC (STM32L4R5ZI, BMI088 backup gyro, coil drivers), in-house", "1", 45.0, "E1",
           "MB v2.0 row 10", [box(1.0, (0, 0, -2.0), (90, 96, 8))]))
    a(Item("18", "RF: transponder receiver, 2 m transmitter, PA, filters, shield, in-house", "1", 120.0,
           "E1", "MB v2.0 row 11", [box(1.0, (0, 0, 10.0), (90, 96, 12))]))
    a(Item("19", "Antenna system, ISIS AntS 1U-3U, monopole/hybrid (4 tape elements)", "1", 85.0, "C2",
           "77-85 g by configuration (ISIS brochure), 89 g (SatNow), < 90 g (BUILD_VS_BUY 2.6): "
           "CBE 85 g, MGA to 90 g",
           [box(1.0, (0, 0, -53.0), (98, 98, 7))], mga=0.06, mga_low=0.0,
           deployed_prims=[]))     # filled in by antenna_deployed()
    a(Item("20", "Jetson Orin Nano 8 GB module (900-13767-0030-000)", "1", 31.0, "C2",
           "30.8 g distributor shipping weight (Farnell 4200232); NVIDIA publishes no module mass. "
           "MGA 10 %", [box(1.0, (0, 0, 26.0), (69.6, 45, 5))], mga=0.10, mga_low=0.05))
    a(Item("21", "Jetson carrier board, in-house", "1", 40.0, "E1", "MB v2.0 4.4",
           [box(1.0, (0, 0, 22.0), (90, 96, 6))]))
    a(Item("22", "Jetson thermal interface and module clamp", "1", 20.0, "E1",
           "MB v2.0 4.4 (interface 10, mounting 10)", [box(1.0, (0, 0, 29.0), (70, 50, 4))]))
    a(Item("23", "Jetson heat spreader to the frame, Al 6061 80 x 60 x 5 mm, with the SoC pedestal", "1", 66.6, "E2",
           "Plate 64.8 g Al + pedestal and second face 1.8 g (SQSO-MECH-CAD-001 4.5; carrier item M1); "
           ">= 1 W/K, BASELINE 6", [box(1.0, (0, 0, 33.5), (80, 60, 5))]))
    a(Item("24", "Magnetometer (M315, MM200 or Lodestone)", "1", 10.0, "C2",
           "8-12 g (BUILD_VS_BUY 2.7): CBE 10 g, MGA to 12 g",
           [box(1.0, (30.0, 30.0, -46.0), (25, 25, 8))], mga=0.20, mga_low=0.0))
    a(Item("25", "Gyro, Epson M-G370PDT0", "1", 10.0, "C1", "10 g (BUILD_VS_BUY 5.2)",
           [box(1.0, (-25.0, 25.0, -2.0), (24, 24, 10))]))
    a(Item("26", "ADCS sensor brackets and cables", "1 set", 9.0, "E1", "Sensor cables W6B-W6E 6.6 g (SQSO-HAR-001 "
           "2.0 sec. 11), magnetometer board A2 fasteners 2.0 g, +-Z sun-sensor carriers 0.4 g (the side sun sensors "
           "are the panel photodiodes, row 6)", [box(1.0, (0, 0, 0), (80, 80, 80))]))
    a(Item("27r", "Magnetorquer rods, CubeSpace CR0002 (3)", "3", 49.5, "C1",
           "16.5 g each (BUILD_VS_BUY 2.7)",
           [bar(1 / 3, (0.0, -38.0, -44.0), "x", 70.0, 10.0), bar(1 / 3, (-38.0, 0.0, -44.0), "y", 70.0, 10.0),
            bar(1 / 3, (-38.0, -38.0, 0.0), "z", 70.0, 10.0)], options=("rods",)))
    a(Item("28r", "Rod bracket", "1", 8.0, "E1", "Estimate", [box(1.0, (-25.0, -25.0, -40.0), (50, 50, 10))],
           options=("rods",)))
    a(Item("27c", "Magnetorquers, air-core coils 80 x 80 mm, 250 turns (3)", "3", 138.9, "E2",
           "MB v2.0 4.5 (copper 36.3 g + bobbin 10 g each)",
           [ring(1 / 3, (0.0, 0.0, -46.0), "z", 80.0, 5.0), ring(1 / 3, (-45.0, 0.0, 0.0), "x", 80.0, 5.0),
            ring(1 / 3, (0.0, -45.0, 0.0), "y", 80.0, 5.0)], options=("coils",)))
    a(Item("29", "GNSS receiver, SkyFox piNAV-NG/FM", "1", 24.0, "C1", "Datasheet (D6 research)",
           [box(1.0, (0, 0, 40.0), (50, 50, 8))]))
    a(Item("30", "GNSS patch antenna, Spacemanic ZENITH V3a, +Z face", "1", 20.0, "C2",
           "About 20 g (D6 research): MGA 10 %", [box(1.0, (0, 0, 52.5), (50, 50, 5))], mga=0.10, mga_low=0.05))
    a(Item("31", "GNSS cable, adapter and bracket", "1", 26.0, "E1", "Balance of the 70 g GNSS allocation",
           [box(1.0, (0, 0, 46.0), (60, 60, 6))]))
    # Distributed ------------------------------------------------------------------------
    a(Item("32", "Harness (incl. 120 mm battery flying leads, access-port board A3)", "1", 75.6, "E1",
           "SQSO-HAR-001 2.0 sec. 11: wire, W10 coax, harness-side connector halves, A3, tabs, lacing "
           "(wing flex jumpers in rows 10/11, W6B-W6E in row 26, W10E in row 31)",
           [box(1.0, (0, 0, -5.0), (90, 90, 90))]))
    a(Item("33", "Thermal control (tapes, interface materials, MLI patches)", "1", 30.0, "E1",
           "MB v2.0 row 21", [box(1.0, (0, 0, 0.0), (90, 90, 90))]))
    a(Item("34", "Fasteners, PCB-stack spacers, inserts", "many", 35.0, "E1",
           "MB v2.0 row 22 (25 g) + 4 columns of M3 spacers (10 g)",
           [bar(0.25, (sx * 40.0, sy * 43.0, 0.0), "z", 95.0, 5.0) for sx in (1, -1) for sy in (1, -1)]))
    a(Item("35", "Staking, conformal coat, adhesives", "1", 10.0, "E1", "Estimate",
           [box(1.0, (0, 0, 0), (90, 90, 90))]))
    # Inertia ballast (chair decision C-13, SQSO-MECH-CAD-001 5.2): raises Iyy / max(Ixx, Izz) with the four AntS
    # elements; on the +Z end frame inside, near the Z axis, symmetric in y (no Iyz), outside the magnetometer's
    # 20 mm zone; W-Ni-Cu (non-magnetic) discs; their mass is the integration trim (VI-MCAD-03)
    a(Item("36", "Inertia ballast, 2 x 10 g W-Ni-Cu discs on the +Z end frame (Z-axis trim)", "2", 20.0, "C1",
           "Machined to mass; trimmed at integration from the measured inertia",
           [box(0.5, (-17.0, 15.5, 46.6), (14, 14, 3.6)), box(0.5, (-17.0, 0.5, 46.6), (14, 14, 3.6))]))
    a(Item("37", "Battery cell retention: 2 G-10CR cell-locator plates, -X cap disc-spring stacks and Ti shoulder "
           "screws, polyimide insulators, shims (RID STR-08)", "1 set", 5.0, "E2",
           "Locator plates 3.0 g, 4 shoulder screws 1.0 g, 8 disc springs 0.4 g, polyimide 0.4 g, shims 0.2 g "
           "(SQSO-MECH-CAD-001 4.6)", [box(0.5, (-31.5, 0, -20.0), (2, 74, 20)), box(0.5, (31.5, 0, -20.0), (2, 74, 20))]))
    return L


# Layout variants -----------------------------------------------------------------------
#: "ref": the reference stack of STRUCTURAL_ANALYSIS.md section 3 (battery near the middle).
#: "endmass": the recommended stack (STR-01): the battery, its mount and blanket at the -Z
#: end, the RF board at the +Z end, so that +Y (the spin axis) is the major axis of
#: inertia with the wings deployed.  New centre z (mm) by MEL row.
#: A float moves every primitive of the row to that centre z; a dict moves single primitives
#: (index: new z).
LAYOUTS: Dict[str, Dict] = {
    "ref": {},
    "endmass": {"13": -35.0, "14": -22.0, "15": -35.0, "16": -35.0, "37": -35.0, "33": -35.0,
                "27r": {0: -19.5, 1: -19.5}, "28r": -19.5, "27c": {0: -19.5},
                "12": -12.0, "17": -3.0, "21": 6.0, "20": 10.0, "22": 13.0, "23": 19.0,
                "18": 36.0, "29": 45.0, "31": 47.0, "24": 46.0},
}


def apply_layout(its: List[Item], name: str) -> List[Item]:
    import copy
    new = LAYOUTS[name]
    out = []
    for it in its:
        it = copy.deepcopy(it)
        if it.row in new:
            v = new[it.row]
            if isinstance(v, dict):
                it.prims = [Prim(p.kind, p.frac, (p.c[0], p.c[1], v.get(k, p.c[2])), p.dims, p.axis)
                            for k, p in enumerate(it.prims)]
            else:
                dz = v - it.prims[0].c[2]
                it.prims = [Prim(p.kind, p.frac, (p.c[0], p.c[1], p.c[2] + dz), p.dims, p.axis)
                            for p in it.prims]
        out.append(it)
    return out


# AntS elements, deployed ---------------------------------------------------------------
VHF_TAPE_G = 2.5      # 0.50 m x 6 mm x 0.13 mm spring steel (7.85 g/cm3) ~ 3.1 g; ISIS: tape 5 mm
UHF_TAPE_G = 1.0
VHF_LEN = 500.0
UHF_LEN = 170.0
ANTS_Z = -53.0


def antenna_deployed(clock: str) -> List[Prim]:
    """AntS with its four tape elements out in the plane of the -Z face.  clock 'vhf_x':
    the two VHF elements along +-X (parallel to the wings), UHF along +-Y; 'vhf_y': the
    reverse."""
    m_tot = 85.0
    plate = (m_tot - 2 * VHF_TAPE_G - 2 * UHF_TAPE_G) / m_tot
    p = [box(plate, (0, 0, ANTS_Z), (98, 98, 7))]
    vx = clock == "vhf_x"
    for s in (1.0, -1.0):
        for length, mass, along_x in ((VHF_LEN, VHF_TAPE_G, vx), (UHF_LEN, UHF_TAPE_G, not vx)):
            r0 = 49.0 + length / 2
            c = (s * r0, 0.0, ANTS_Z - 2.0) if along_x else (0.0, s * r0, ANTS_Z - 2.0)
            p.append(bar(mass / m_tot, c, "x" if along_x else "y", length, 1.0))
    return p


#: The AntS configuration options of hardware/rf/ANTENNA_DESIGN.md v2.0 section 3:
#: (axis, sign, length mm, mass g) per element.
ANTS_OPTIONS = {
    "A: one 2 m element +X, 70 cm -X": [("x", 1, VHF_LEN, VHF_TAPE_G), ("x", -1, UHF_LEN, UHF_TAPE_G)],
    "B: one 2 m element +Y, 70 cm -Y": [("y", 1, VHF_LEN, VHF_TAPE_G), ("y", -1, UHF_LEN, UHF_TAPE_G)],
    "C: 2 m +X and +Y (quadrature), 70 cm -X, -Y": [("x", 1, VHF_LEN, VHF_TAPE_G), ("y", 1, VHF_LEN, VHF_TAPE_G),
                                                     ("x", -1, UHF_LEN, UHF_TAPE_G), ("y", -1, UHF_LEN, UHF_TAPE_G)],
    "hybrid: 2 m +-X, 70 cm +-Y (MEL case)": [("x", 1, VHF_LEN, VHF_TAPE_G), ("x", -1, VHF_LEN, VHF_TAPE_G),
                                               ("y", 1, UHF_LEN, UHF_TAPE_G), ("y", -1, UHF_LEN, UHF_TAPE_G)],
}


def antenna_elements(elems) -> List[Prim]:
    m_tot = 85.0
    p = [box((m_tot - sum(e[3] for e in elems)) / m_tot, (0, 0, ANTS_Z), (98, 98, 7))]
    for ax, sgn, length, mass in elems:
        r0 = 49.0 + length / 2
        c = (sgn * r0, 0.0, ANTS_Z - 2.0) if ax == "x" else (0.0, sgn * r0, ANTS_Z - 2.0)
        p.append(bar(mass / m_tot, c, ax, length, 1.0))
    return p


# --------------------------------------------------------------------------------------
# Inertia of primitives
# --------------------------------------------------------------------------------------
def prim_inertia(p: Prim, m: float) -> np.ndarray:
    """Inertia about the primitive's own centre (kg m^2), body axes; m in kg, dims in mm."""
    mm2 = 1e-6
    if p.kind == "box":
        dx, dy, dz = (d for d in p.dims)
        return np.diag([m * (dy * dy + dz * dz) / 12, m * (dx * dx + dz * dz) / 12,
                        m * (dx * dx + dy * dy) / 12]) * mm2
    if p.kind == "bar":
        length, dia = p.dims
        il = m * length * length / 12 + m * dia * dia / 16
        ia = m * dia * dia / 8
        idx = "xyz".index(p.axis)
        d = [il, il, il]
        d[idx] = ia
        return np.diag(d) * mm2
    if p.kind == "ring":
        side, dia = p.dims
        # four bars of m/4, length side, at +-side/2 from the centre in the loop plane
        i_n = m * side * side / 3            # about the loop normal
        i_p = m * side * side / 6            # about an in-plane axis through the centre
        idx = "xyz".index(p.axis)
        d = [i_p, i_p, i_p]
        d[idx] = i_n
        return np.diag(d) * mm2
    raise ValueError(p.kind)


def shift(m: float, r_mm: np.ndarray) -> np.ndarray:
    r = r_mm * 1e-3
    return m * (np.dot(r, r) * np.eye(3) - np.outer(r, r))


@dataclass
class Props:
    mass_g: float
    cg_mm: np.ndarray
    inertia: np.ndarray        # about the CG, kg m^2

    def principal(self):
        w, v = np.linalg.eigh(self.inertia)
        return w, v


def assemble(its: List[Item], option: str, deployed: bool, clock: str = "vhf_x",
             grown: bool = False, wings: Tuple[bool, bool] = (True, True)) -> Props:
    parts = []   # (mass kg, centre mm, prim)
    for it in its:
        if option not in it.options:
            continue
        m_item = it.cbe_g * (1 + it.growth()) if grown else it.cbe_g
        prims = it.prims
        if deployed:
            if it.row == "19":
                prims = antenna_elements(ANTS_OPTIONS[clock]) if clock in ANTS_OPTIONS else antenna_deployed(clock)
            elif it.deployed_prims:
                if it.row in ("7", "10"):
                    prims = it.deployed_prims if wings[0] else it.prims
                elif it.row in ("8", "11"):
                    prims = it.deployed_prims if wings[1] else it.prims
                elif it.row == "9":
                    prims = (it.deployed_prims[:1] if wings[0] else it.prims[:1]) + \
                            (it.deployed_prims[1:] if wings[1] else it.prims[1:])
                else:
                    prims = it.deployed_prims
        for p in prims:
            parts.append((m_item * p.frac / 1000.0, np.array(p.c, float), p))
    m = sum(x[0] for x in parts)
    cg = sum(x[0] * x[1] for x in parts) / m
    inertia = np.zeros((3, 3))
    for mi, c, p in parts:
        inertia += prim_inertia(p, mi) + shift(mi, c - cg)
    return Props(m * 1000.0, cg, inertia)


def cg_uncertainty(its: List[Item], option: str, sigma_mm: float = 5.0) -> np.ndarray:
    """1-sigma CG uncertainty from independent +-sigma placement of each item."""
    ms = [it.cbe_g for it in its if option in it.options]
    mt = sum(ms)
    return np.full(3, math.sqrt(sum((m * sigma_mm) ** 2 for m in ms)) / mt)


def mel_table(its: List[Item], option: str):
    rows, tot_cbe, tot_pred, tot_low = [], 0.0, 0.0, 0.0
    for it in its:
        if option not in it.options:
            continue
        g = it.growth()
        gl = it.growth(low=True)
        pred = it.cbe_g * (1 + g)
        rows.append((it.row, it.name, it.qty, it.cbe_g, it.maturity, g, pred, it.basis))
        tot_cbe += it.cbe_g
        tot_pred += pred
        tot_low += it.cbe_g * (1 + gl)
    return rows, tot_cbe, tot_pred, tot_low


def margin(limit, pred):
    return (limit - pred) / pred * 100.0


# sqso_fsw::adcs::INERTIA and sqso-sil env/attitude.rs as of 2026-10-27 (the CAD means of MASS_BUDGET v3.1 6.5)
FSW_INERTIA = np.array([0.0030565, 0.0044675, 0.0042525])
SIL_STOWED = np.array([2.952e-3, 3.2805e-3, 2.9605e-3])
SIL_DEPLOYED = np.array([3.0565e-3, 4.4675e-3, 4.2525e-3])


def nutation_ratio(i: np.ndarray, spin_axis: int = 1) -> Dict[str, float]:
    """For spin about body axis `spin_axis`: inertia ratios and the torque-free nutation
    frequency ratio (rigid body; spin stable if the axis is the major or minor axis, and
    stable with energy dissipation only if it is the major axis)."""
    d = np.diag(i)
    s = d[spin_axis]
    o = [d[k] for k in range(3) if k != spin_axis]
    k1, k2 = (s - o[0]) / o[1], (s - o[1]) / o[0]
    lam = math.sqrt(k1 * k2) if k1 * k2 > 0 else float("nan")
    return {"I_spin": s, "ratio_to_next": s / max(o), "nutation_over_spin": lam,
            "major": bool(s > max(o)), "minor": bool(s < min(o))}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json")
    args = ap.parse_args()
    out: Dict = {}
    for lay, opt in (("ref", "rods"), ("ref", "coils"), ("endmass", "rods"), ("endmass", "coils")):
        its = apply_layout(items(), lay)
        rows, cbe, pred, low = mel_table(its, opt)
        flat15 = cbe * 1.15
        o = out[f"{lay}_{opt}"] = {
            "cbe_g": cbe, "mev_g": pred, "mev_low_g": low, "mev_flat15_g": flat15, "mga_pct": (pred / cbe - 1) * 100,
            "margin_cds_pct": margin(CDS_LIMIT_G, pred), "margin_cds_low_pct": margin(CDS_LIMIT_G, low),
            "margin_nova_pct": margin(NOVA_LIMIT_G, pred), "margin_nrcsd_pct": margin(NRCSD_LIMIT_G, pred),
            "rows": rows,
        }
        print(f"\n=== layout {lay}, magnetorquer option: {opt} ===")
        for r in (rows if lay == "ref" else []):
            print(f"{r[0]:>4} {r[1][:70]:70s} {r[3]:7.1f} g {r[4]:4s} {r[5]*100:5.1f}% {r[6]:7.1f} g")
        print(f"CBE {cbe:.1f} g  MEV {pred:.1f} g (MGA {o['mga_pct']:.1f} %)  low-growth MEV {low:.1f} g  "
              f"flat 15 % {flat15:.1f} g")
        print(f"Margin to 2.00 kg: {o['margin_cds_pct']:+.1f} % (low growth {o['margin_cds_low_pct']:+.1f} %); "
              f"to EXOpod Nova 2.5 kg {o['margin_nova_pct']:+.1f} %; to NRCSD 2.4 kg {o['margin_nrcsd_pct']:+.1f} %")
        for state in ("stowed", "deployed"):
            for clock in (("vhf_x", "vhf_y") if state == "deployed" else ("vhf_x",)):
                for grown in (False, True):
                    pp = assemble(its, opt, state == "deployed", clock, grown)
                    w, v = pp.principal()
                    key = f"{state}_{clock}_{'mev' if grown else 'cbe'}"
                    major = int(np.argmax(np.abs(v[:, 2])))
                    ang = math.degrees(math.acos(min(1.0, abs(v[1, 2]))))
                    nut = nutation_ratio(pp.inertia, 1)
                    o[key] = {"mass_g": pp.mass_g, "cg_mm": pp.cg_mm.tolist(),
                              "I_diag": np.diag(pp.inertia).tolist(),
                              "I_offdiag": [pp.inertia[0, 1], pp.inertia[0, 2], pp.inertia[1, 2]],
                              "principal": w.tolist(), "major_axis_body": "xyz"[major],
                              "major_axis_angle_from_Y_deg": ang, "spin_Y": nut}
                    print(f"  {key:22s} m {pp.mass_g:7.1f} g  CG ({pp.cg_mm[0]:+5.1f}, {pp.cg_mm[1]:+5.1f}, "
                          f"{pp.cg_mm[2]:+5.1f}) mm  I = ({np.diag(pp.inertia)[0]*1e3:.3f}, "
                          f"{np.diag(pp.inertia)[1]*1e3:.3f}, {np.diag(pp.inertia)[2]*1e3:.3f})e-3  "
                          f"Ixy {pp.inertia[0,1]*1e6:+.1f} Ixz {pp.inertia[0,2]*1e6:+.1f} "
                          f"Iyz {pp.inertia[1,2]*1e6:+.1f} e-6  principal ({w[0]*1e3:.3f}, {w[1]*1e3:.3f}, "
                          f"{w[2]*1e3:.3f})e-3 major~{'xyz'[major]} {ang:.1f} deg from Y; "
                          f"Iy/max(other) {nut['ratio_to_next']:.3f}")
        u = cg_uncertainty(its, opt)
        o["cg_sigma_mm"] = u.tolist()
        print(f"  CG 1-sigma placement uncertainty {u[0]:.1f} mm per axis")
        # partial deployment: one wing out
        for wings, lab in (((True, False), "W1_only"), ((False, False), "none")):
            pp = assemble(its, opt, True, "vhf_x", False, wings)
            o[f"partial_{lab}"] = {"cg_mm": pp.cg_mm.tolist(), "I_diag": np.diag(pp.inertia).tolist(),
                                   "principal": pp.principal()[0].tolist(),
                                   "I_offdiag": [pp.inertia[0, 1], pp.inertia[0, 2], pp.inertia[1, 2]]}
            w, v = pp.principal()
            major = int(np.argmax(np.abs(v[:, 2])))
            ang = math.degrees(math.acos(min(1.0, abs(v[1, 2]))))
            o[f"partial_{lab}"]["major_axis_angle_from_Y_deg"] = ang
            print(f"  partial {lab:8s} CG ({pp.cg_mm[0]:+5.1f}, {pp.cg_mm[1]:+5.1f}, {pp.cg_mm[2]:+5.1f}) "
                  f"I diag ({np.diag(pp.inertia)[0]*1e3:.3f}, {np.diag(pp.inertia)[1]*1e3:.3f}, "
                  f"{np.diag(pp.inertia)[2]*1e3:.3f})e-3 principal {[round(x*1e3,3) for x in w]} "
                  f"major axis {ang:.1f} deg from Y")
    print("\nAntS configuration options (layout endmass, deployed):")
    for opt in ("rods", "coils"):
        its = apply_layout(items(), "endmass")
        for name in ANTS_OPTIONS:
            for grown in (False, True):
                pp = assemble(its, opt, True, name, grown)
                d = np.diag(pp.inertia)
                out.setdefault("ants_options", {})[f"{opt}|{name}|{'mev' if grown else 'cbe'}"] = d.tolist()
                print(f"  {opt:5s} {name:46s} {'MEV' if grown else 'CBE'} I = ({d[0]*1e3:.3f}, {d[1]*1e3:.3f}, "
                      f"{d[2]*1e3:.3f})e-3  Iyy/max(Ixx, Izz) {d[1]/max(d[0], d[2]):.3f}")
    print("\nFSW INERTIA (deployed):", FSW_INERTIA, " SIL deployed:", SIL_DEPLOYED, " SIL stowed:", SIL_STOWED)
    if args.json:
        def conv(x):
            if isinstance(x, (np.floating, np.integer)):
                return x.item()
            if isinstance(x, np.ndarray):
                return x.tolist()
            raise TypeError(type(x))
        with open(args.json, "w") as f:
            json.dump(out, f, indent=1, default=conv)


if __name__ == "__main__":
    main()
