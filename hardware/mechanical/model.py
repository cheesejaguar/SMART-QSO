#!/usr/bin/env python3
"""
SMART-QSO parametric mechanical CAD model (SQSO-MECH-CAD-001, hardware/mechanical/MECHANICAL_DESIGN.md).

Layout B of docs/environment/MASS_BUDGET.md (battery at the -Z end, RF at the +Z end, ISIS AntS on a recessed
-Z face, one 2 m element along +X), modelled to the level of each part's envelope, mass and position.
CadQuery 2.8 / OpenCascade 7.9. Units: mm, g. Axes: CDS Rev 14.1 body frame (origin at the geometric centre,
+Z along the rails, -Z inserted first).

What it does
  * builds every part as a solid (stowed and deployed), assigns each part its MEL mass (MASS_BUDGET v3.0
    section 3.1, read from software/simulation/cdr/structures/mass_properties.py so that the MEL has one
    source), and computes mass, CG and the full inertia tensor from the solids (uniform density per part);
  * runs the CDS Rev 14.1 envelope checks, the protrusion tolerance stack, the standoff checks, pairwise
    interference (boolean common volume) and the clearance checks (wing sweep, AntS tapes, hinge harness);
  * exports STEP (stowed, deployed) and SVG drawings (hidden-line projections with dimensions).

Usage (from the repository root):
    python3 hardware/mechanical/model.py                    # all: properties, checks, STEP, SVG, JSON
    python3 hardware/mechanical/model.py --no-export        # properties and checks only
Outputs go to hardware/mechanical/ (results.json, *.step, *.svg).

Only the envelopes of bought parts come from published data (cited in PARTS below and in
MECHANICAL_DESIGN.md section 3); where a value is an allocation it is marked "allocation".
"""
from __future__ import annotations

import argparse
import copy
import json
import math
import os
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Tuple

import numpy as np
import cadquery as cq
from OCP.BRepGProp import BRepGProp
from OCP.GProp import GProp_GProps
from OCP.BRepAlgoAPI import BRepAlgoAPI_Common
from OCP.BRepExtrema import BRepExtrema_DistShapeShape

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "software", "simulation", "cdr", "structures"))
import mass_properties as mp  # noqa: E402  (the MEL: MASS_BUDGET v3.0 section 3.1)

# ======================================================================================================
# Parameters (mm).  Every dimension the checks use is here.
# ======================================================================================================
P: Dict[str, float] = dict(
    # CDS Rev 14.1 Appendix B, 1U (drawing CDS-14-001)
    W=100.0, W_tol=0.1, L=113.5, L_tol=0.1, rail=8.5, rail_t=1.5, face=50.0, standoff_min=0.1,
    protrusion_max=6.5, rail_edge_r=1.0, rail_end_min=6.5,
    # frame (bought; ISIS / Spacemanic class, inside envelope 98.4 mm per Spacemanic datasheet).  Allocation:
    endplate_t=1.6, mz_recess=1.5,          # -Z end plate outer face at -(face - mz_recess): AntS recess (STR-11)
    shear_t=1.0, panel_recess=0.5,          # side shear panel outer face 0.5 mm below the rail plane
    # body panels (vendor, 2 x AZUR 3G30A each) and wings
    bp_w=82.0, bp_h=98.0, sub_t=1.6, cell_t=0.35, cell_y=80.7, cell_z=80.15, cell_zc=3.0,
    gap=1.0,                                # body cells to wing back cells (bumpers)
    wing_y0=-41.0, wing_y1=40.6, wing_z0=-43.0, wing_z1=49.0,
    hinge_x=54.1, hinge_y=38.75, knuckle_r=1.75, spring_r=2.0,   # hinge.py: springs OD 4.0 mm on the axis
    holddown_y=-38.5, holddown_z=-40.0, line_h=0.5, snub_z=46.0,
    # AntS (ISIS AntS 1U-3U: 98 x 98 x 7 mm, < 90 g, BUILD_VS_BUY 2.6; ANTENNA_DESIGN 2.0 section 2)
    ants_w=98.0, ants_t=7.0, ants_notch=10.0, tape_w=5.0, tape_t=0.5, vhf_len=500.0, uhf_len=170.0,
    tape_y=0.0,
    # GNSS patch (Spacemanic ZENITH V3a, 50 x 50 mm; thickness allocation 5 mm)
    patch_w=50.0, patch_t=5.0,
    # PC/104 boards
    pcb_x=90.17, pcb_y=95.89, pcb_t=1.6, col_x=40.0, col_y=43.0, col_d=5.8, cs_x=0.0, cs_y=21.0, cs2_y=-21.0, jet_y=-6.0,
    # M-G370PDT0 in its socket X1 on the OBC: 24 x 24 mm, 14.0 mm above the OBC top face (OBC_BOARD_DESIGN 9.2); it
    # stands through a window in the Jetson carrier (OBC item C3, MECHANICAL_DESIGN 4.2 D-11)
    gyro_x=-20.0, gyro_y=30.5, gyro_w=24.0, gyro_h=14.0, car_win=(-33.5, -6.5, 17.0, 44.0),
    # Jetson carrier item M1 (SQSO-JET-BRD-001 14, 18): module standoffs MP2/MP3 at the module's far-edge holes (positions
    # from the module outline, 3.5 mm in from the side and 3.9 mm from the far edge: to be checked against the NVIDIA drawing),
    # SoC lid 20 x 20 mm (4.0 cm2 allocation) at the module centre, spreader pedestal over it
    mod_hole=(31.3, 18.6), mod_so_d=5.1, soc_w=20.0, ped_w=22.0, pad_soc=0.75, pad_other=1.5,
    car_frame_w=2.5, car_frame_h=3.0,       # Al 6061 stiffening frame round the window, carrier bottom side
    # frame screws H10-H13 (carrier design 18.1, amended): M2 A286 from the top into 3.8 x 5.0 mm tapped pads on the side bars
    frame_screws=((-35.5, 24.0), (-35.5, 36.0), (-4.5, 30.0), (-4.5, 42.0)), frame_pad=(3.8, 5.0),
    # magnetometer board A2 (OBC item C13): 24 x 24 x 1.6 mm, back face on the +Z end frame, parts (Sen-Z-f 5.75 mm) below;
    # three Ti M2 screws through the end frame at the corners clear of the GNSS patch footprint
    a2_w=24.0, a2_t=1.6, a2_parts=5.75, a2_screws=((33.5, 17.5), (14.5, 36.5), (33.5, 36.5)),
    # stack levels: substrate bottom z
    z_eps=-3.0, z_obc=5.0, z_car=17.6, z_rf=34.8, car_t=2.0, car_frame=False,
    # magnetometer and RF shield (positions chosen to balance Iyz, MECHANICAL_DESIGN section 5)
    mag_x=24.0, mag_y=27.0, shield_y0=-40.0, shield_y1=14.0,
    rbf_x=20.0, rbf_z=-43.0, port_x=-20.0, port_z=-43.0, rbf_face=-1.0,
    ants_config="A",
)


def tol_table():
    """Tolerances for the stack-ups (mm, +-). Sources: frame vendor drawing class (ISO 2768-m), IPC-6013/6012
    board thickness +-10 %, cell and coverglass (AZUR 3G30A datasheet thickness range), assumptions marked."""
    return dict(rail_plane=0.05, shear_face=0.10, sub=0.16, cell=0.05, gap=0.20, line=0.10, post=0.05,
                hinge_axis=0.10, knuckle_r=0.03, ants_t=0.20, endplate=0.10, rail_len=0.05, patch_t=0.20)


# ======================================================================================================
# Geometry helpers
# ======================================================================================================
def B(x0, x1, y0, y1, z0, z1) -> cq.Shape:
    return cq.Solid.makeBox(x1 - x0, y1 - y0, z1 - z0, cq.Vector(x0, y0, z0))


def C(axis: str, c: Tuple[float, float, float], r: float, a0: float, a1: float) -> cq.Shape:
    """Cylinder of radius r along axis from coordinate a0 to a1 (c gives the two other coordinates)."""
    x, y, z = c
    if axis == "z":
        return cq.Solid.makeCylinder(r, a1 - a0, cq.Vector(x, y, a0), cq.Vector(0, 0, 1))
    if axis == "x":
        return cq.Solid.makeCylinder(r, a1 - a0, cq.Vector(a0, y, z), cq.Vector(1, 0, 0))
    return cq.Solid.makeCylinder(r, a1 - a0, cq.Vector(x, a0, z), cq.Vector(0, 1, 0))


def cut(a: cq.Shape, *bs: cq.Shape) -> cq.Shape:
    for b in bs:
        a = a.cut(b)
    return a


def fuse(*ss: cq.Shape) -> cq.Shape:
    a = ss[0]
    for b in ss[1:]:
        a = a.fuse(b)
    return a.clean()


def mirror_x(s: cq.Shape) -> cq.Shape:
    return s.mirror("YZ")


# ======================================================================================================
# Parts
# ======================================================================================================
@dataclass
class Part:
    name: str
    group: str
    mel: Dict[str, float]            # MEL row -> grams of that row's CBE carried by this part (or "vol" share)
    stowed: cq.Shape
    deployed: Optional[cq.Shape] = None
    color: Tuple[float, float, float] = (0.7, 0.7, 0.7)
    ghost: bool = False              # distributed mass allocation: no interference check, not in STEP
    note: str = ""
    moves: str = ""                  # "W1"/"W2"/"ANT": moves on deployment


COL = dict(frame=(0.75, 0.75, 0.78), panel=(0.10, 0.15, 0.45), cell=(0.05, 0.05, 0.25), pcb=(0.1, 0.5, 0.2),
           comp=(0.3, 0.3, 0.3), batt=(0.8, 0.4, 0.1), ants=(0.9, 0.9, 0.9), mech=(0.8, 0.1, 0.1),
           harness=(0.9, 0.7, 0.1), rod=(0.4, 0.2, 0.5), al=(0.85, 0.85, 0.85), gnss=(0.6, 0.6, 0.1))


def z_planes(p=P):
    f = p["face"]
    return dict(mz_out=-(f - p["mz_recess"]), mz_in=-(f - p["mz_recess"]) + p["endplate_t"],
                pz_out=f, pz_in=f - p["endplate_t"], rail_end=p["L"] / 2.0)


def wing_stowed_x(p=P):
    """x of the wing layers (W1, +X side): returns dict of layer bounds."""
    shear_out = p["face"] - p["panel_recess"]
    bp0 = shear_out
    bp1 = bp0 + p["sub_t"]
    bc1 = bp1 + p["cell_t"]
    wb0 = bc1 + p["gap"]
    wb1 = wb0 + p["cell_t"]
    ws1 = wb1 + p["sub_t"]
    wf1 = ws1 + p["cell_t"]
    return dict(shear_in=shear_out - p["shear_t"], shear_out=shear_out, bp0=bp0, bp1=bp1, bc1=bc1, wb0=wb0,
                wb1=wb1, ws1=ws1, wf1=wf1, line=wf1 + p["line_h"])


def build_parts(p=P) -> List[Part]:
    zp = z_planes(p)
    hw = p["W"] / 2.0
    a = p["rail"]
    t = p["rail_t"]
    re = zp["rail_end"]
    inner = hw - t                     # 48.5: inside of the rail legs / frame inner envelope
    parts: List[Part] = []
    add = parts.append

    # ---------------- frame (MEL row 1, by volume) --------------------------------------------------------
    sw_holes = []                      # switch / plunger bores (x, y, end)
    sw_pos = {"DS1": (46.0, 46.0, -1), "DS2": (-46.0, -46.0, -1), "DS3": (46.0, -46.0, +1),
              "SP1": (-46.0, 46.0, -1), "SP2": (46.0, -46.0, -1)}
    for k, (x, y, e) in sw_pos.items():
        sw_holes.append(C("z", (x, y, 0), 2.35, -re - 1 if e < 0 else 44.0, -44.0 if e < 0 else re + 1))
    for sx in (1, -1):
        for sy in (1, -1):
            legx = B(min(sx * (hw - t), sx * hw), max(sx * (hw - t), sx * hw), min(sy * (hw - a), sy * hw),
                     max(sy * (hw - a), sy * hw), -p["face"], p["face"])
            legy = B(min(sx * (hw - a), sx * hw), max(sx * (hw - a), sx * hw), min(sy * (hw - t), sy * hw),
                     max(sy * (hw - t), sy * hw), -p["face"], p["face"])
            so = [B(min(sx * (hw - a), sx * hw), max(sx * (hw - a), sx * hw), min(sy * (hw - a), sy * hw),
                    max(sy * (hw - a), sy * hw), z0, z1) for z0, z1 in ((-re, -p["face"]), (p["face"], re))]
            rail = fuse(legx, legy, *so)
            rail = cut(rail, *sw_holes)
            add(Part(f"rail_{'p' if sx > 0 else 'm'}{'p' if sy > 0 else 'm'}", "frame", {"1": -1}, rail,
                     color=COL["frame"], note="L-angle 8.5 x 8.5 x 1.5 with solid 8.5 x 8.5 standoffs"))
    colholes = [C("z", (sx * p["col_x"], sy * p["col_y"], 0), 1.6, -60, 60) for sx in (1, -1) for sy in (1, -1)]
    mz = B(-inner, inner, -inner, inner, zp["mz_out"], zp["mz_in"])
    mz = cut(mz, B(-35, 35, -35, 35, -60, 60), *colholes, *sw_holes)
    add(Part("endplate_mz", "frame", {"1": -1}, mz, color=COL["frame"],
             note="-Z end frame, outer face recessed 1.5 mm for the AntS (STR-11)"))
    pz = B(-inner, inner, -inner, inner, zp["pz_in"], zp["pz_out"])
    a2h = [C("z", (p["mag_x"] + (x - 24.0), p["mag_y"] + (y - 27.0), 0), 1.1, 40, 60) for x, y in p["a2_screws"]]
    pz = cut(pz, B(-15, 15, -15, 15, -60, 60), *colholes, *sw_holes, *a2h)
    add(Part("endplate_pz", "frame", {"1": -1}, pz, color=COL["frame"]))
    wx = wing_stowed_x(p)
    for ax, s in (("x", 1), ("x", -1), ("y", 1), ("y", -1)):
        lo, hi = sorted((s * wx["shear_in"], s * wx["shear_out"]))
        if ax == "x":
            sh = B(lo, hi, -(hw - a), hw - a, zp["mz_out"], zp["pz_out"])
        else:
            sh = B(-(hw - a), hw - a, lo, hi, zp["mz_out"], zp["pz_out"])
        add(Part(f"shear_{'p' if s > 0 else 'm'}{ax}", "frame", {"1": -1}, sh, color=COL["frame"],
                 note="side shear panel (frame secondary structure), outer face 0.5 mm below the rail plane"))

    # ---------------- switches, separation plungers, RBF, access port (rows 2, 3, 4) ----------------------
    for k, (x, y, e) in sw_pos.items():
        if k.startswith("DS"):
            s = C("z", (x, y, 0), 2.25, -re, -re + 12.0) if e < 0 else C("z", (x, y, 0), 2.25, re - 12.0, re)
            add(Part(k, "inhibit", {"2": 2.0}, s, color=COL["mech"],
                     note="deployment switch in the rail standoff, actuated flush with the standoff end"))
    for k in ("SP1", "SP2"):
        x, y, _ = sw_pos[k]
        add(Part(k, "inhibit", {"3": 1.5}, C("z", (x, y, 0), 2.25, -re, -re + 10.75), color=COL["mech"],
                 note="separation spring plunger, < 6.7 N, stroke 3 mm, flush when compressed (CDS 2.2.14)"))
    fs = p["rbf_face"]

    def fy(a0, a1):
        return (min(fs * a0, fs * a1), max(fs * a0, fs * a1))
    rbf = fuse(C("y", (p["rbf_x"], 0, p["rbf_z"]), 4.0, *fy(wx["bp1"] + 4.0, wx["bp1"])),
               B(p["rbf_x"] - 5, p["rbf_x"] + 5, *fy(inner, 41.5), p["rbf_z"] - 3.5, p["rbf_z"] + 3.5))
    add(Part("RBF", "inhibit", {"4": 4.0}, rbf, color=COL["mech"],
             note="RBF pin head 4 mm above the -Y panel substrate; switch body inside"))
    port = fuse(B(p["port_x"] - 9, p["port_x"] + 9, *fy(wx["bp1"] + 0.5, wx["bp1"]), p["port_z"] - 3.5,
                  p["port_z"] + 2.5),
                B(p["port_x"] - 9, p["port_x"] + 9, *fy(inner, 41.5), p["port_z"] - 3.5, p["port_z"] + 2.5))
    add(Part("access_port", "inhibit", {"32": 2.0}, port, color=COL["mech"],
             note="umbilical / charge connector (Micro-D 9 class), location open (LVR-068 Nova access window)"))

    # ---------------- body panels (row 6) -----------------------------------------------------------------
    bh = p["bp_h"] / 2.0
    bw = p["bp_w"] / 2.0
    czc, czh, cyh = p["cell_zc"], p["cell_z"] / 2.0, p["cell_y"] / 2.0
    kz = [(-49.0, -37.8), (43.6, 49.6)]                        # hinge hardware z ranges on the axis (hinge.py)
    notch_z = [(-bh - 1.0, -37.3), (43.3, bh)]
    for ax, s in (("x", 1), ("x", -1), ("y", 1), ("y", -1)):
        lo, hi = sorted((s * wx["bp0"], s * wx["bp1"]))
        clo, chi = sorted((s * wx["bp1"], s * wx["bc1"]))
        if ax == "x":
            sub = B(lo, hi, -bw, bw, -bh, bh)
            cells = B(clo, chi, -cyh, cyh, czc - czh, czc + czh)
            notches = [B(lo - 1, hi + 1, 36.0, bw + 1, z0, z1) for z0, z1 in notch_z]
            holes = [C("x", (0, p["holddown_y"], p["holddown_z"]), 1.75, lo - 1, hi + 1)]
            sub = cut(sub, *notches, *holes)
        else:
            sub = B(-bw, bw, lo, hi, -bh, bh)
            cells = B(-cyh, cyh, clo, chi, czc - czh, czc + czh)
            if s == p["rbf_face"]:
                sub = cut(sub, C("y", (p["rbf_x"], 0, p["rbf_z"]), 4.2, lo - 1, hi + 1),
                          B(p["port_x"] - 9.2, p["port_x"] + 9.2, lo - 1, hi + 1, p["port_z"] - 3.7,
                            p["port_z"] + 2.7))
        tag = f"{'p' if s > 0 else 'm'}{ax}"
        add(Part(f"body_panel_{tag}", "panel", {"6": 32.8}, sub, color=COL["panel"]))
        add(Part(f"body_cells_{tag}", "panel", {"6": 7.2}, cells, color=COL["cell"],
                 note="2 x AZUR 3G30A (<= 3.6 g each) with coverglass"))

    # ---------------- wings (rows 7-11), W1 at +X; W2 mirrored ---------------------------------------------
    hx, hy, kr = p["hinge_x"], p["hinge_y"], p["knuckle_r"]
    wy0, wy1, wz0, wz1 = p["wing_y0"], p["wing_y1"], p["wing_z0"], p["wing_z1"]

    def wing_set(sign: int) -> List[Part]:
        out = []
        tag = "W1" if sign > 0 else "W2"
        knotch = [B(wx["wb0"] - 1, wx["wf1"] + 1, 36.5, wy1 + 1, z0 - 0.4, z1 + 0.4) for z0, z1 in
                  [(-43.5, -37.5), (43.2, wz1 + 1)]]
        hd_hole = C("x", (0, p["holddown_y"], p["holddown_z"]), 1.75, wx["wb0"] - 1, wx["wf1"] + 1)
        sub = cut(B(wx["wb1"], wx["ws1"], wy0, wy1, wz0, wz1), *knotch, hd_hole)
        back = B(wx["wb0"], wx["wb1"], -cyh, cyh, czc - czh, czc + czh)
        front = B(wx["ws1"], wx["wf1"], -cyh, cyh, czc - czh, czc + czh)
        line = B(wx["wf1"], wx["line"], p["holddown_y"] - 1.5, p["holddown_y"] + 1.5, p["holddown_z"] - 1.5,
                 p["holddown_z"] + 1.5)
        snub = C("x", (0, p["holddown_y"], p["snub_z"]), 2.0, wx["bp1"], wx["wb1"])
        sr = p["spring_r"]
        # -Z: frame ear (pin bearing, primary-spring frame leg) z -49.0 .. -47.6, leaf bolted to the shear panel;
        # primary spring -47.6 .. -40.1 (below the wing, which ends at z = -43); wing-side knuckle = clock-spring
        # flex cup -39.9 .. -37.3.
        # +Z: wing knuckle 43.6 .. 44.6; secondary spring 44.8 .. 48.4; frame ear 48.6 .. 49.6 (hinge.py)
        ears = [C("z", (hx, hy, 0), kr, -49.0, -47.8), B(wx["shear_out"], hx - 1.0, 36.5, 40.0, -48.5, -47.6),
                C("z", (hx, hy, 0), kr, 48.6, 49.6), B(wx["shear_out"], hx - 1.0, 36.5, 40.0, 48.6, 49.6),
                C("z", (hx, hy, 0), kr, 43.6, 44.6)]
        hinge = fuse(*ears)
        springs = fuse(C("z", (hx, hy, 0), sr, -47.6, -40.1), C("z", (hx, hy, 0), sr, 44.8, 48.4))
        flex = fuse(C("z", (hx, hy, 0), sr, -39.9, -37.3), B(wx["shear_out"], hx, 37.0, 37.5, -39.5, -37.7))
        post = C("x", (0, p["holddown_y"], p["holddown_z"]), 1.5, wx["bp1"], wx["wf1"])
        sensor = B(wx["bp1"], wx["wb1"] - 0.2, 33.5, 36.3, kz[0][0], kz[0][1])
        burn = B(wx["shear_in"] - 2.0, wx["shear_in"], -41.0, -34.0, -45.0, -35.5)
        doubl = [B(wx["shear_in"] - 1.0, wx["shear_in"], 35.0, 41.0, z0, z1) for z0, z1 in ((-46.8, -42.0),
                                                                                           (44.0, 48.4))]
        moving = dict(substrate=sub, back=back, front=front, line=line, snub=snub)
        ax0, ax1 = cq.Vector(hx, hy, 0), cq.Vector(hx, hy, 1)

        def dep(s):
            return s.rotate(ax0, ax1, 90.0)
        rows = ("7", "10", "9") if sign > 0 else ("8", "11", "9")
        spec = [
            (f"{tag}_substrate", "wing", {rows[0]: 28.7}, sub, True, COL["panel"], "FR-4 1.6 mm, 81.6 x 92 mm"),
            (f"{tag}_cells_back", "wing", {rows[2]: 7.2}, back, True, COL["cell"], "2 x 3G30A"),
            (f"{tag}_cells_front", "wing", {rows[2]: 7.2}, front, True, COL["cell"], "2 x 3G30A"),
            (f"{tag}_holddown_line", "wing", {rows[1]: 0.5}, line, True, COL["mech"], "Vectran/Dyneema loop"),
            (f"{tag}_snubber", "wing", {rows[1]: 1.0}, snub, True, COL["mech"],
             "PTFE-faced silicone snubber, +Z free corner"),
            (f"{tag}_hinges", "wing", {rows[1]: 6.0}, hinge, False, COL["mech"],
             "440C pin (MoS2) in Ti-6Al-4V bushings: frame ears at -Z and +Z, +Z wing knuckle"),
            (f"{tag}_springs", "wing", {rows[1]: 1.0}, springs, False, COL["rod"],
             "17-7PH CH900 torsion springs OD 4.0: primary 0.470 mm x 15.4 coils below the wing, secondary "
             "0.343 mm x 9.5 coils at +Z, 55 deg preload (hinge.py)"),
            (f"{tag}_hinge_flex", "harness", {rows[1]: 2.0}, flex, False, COL["harness"],
             "two-layer polyimide clock-spring flex (6 turns, R 1.0-1.6 mm) in the -Z wing knuckle, tail into the body"),
            (f"{tag}_holddown_post", "wing", {rows[1]: 1.0}, post, False, COL["mech"], "cup/cone post"),
            (f"{tag}_deploy_sensor_stop", "wing", {rows[1]: 1.0}, sensor, False, COL["mech"],
             "non-contact deployed-state sensor and soft stop"),
            (f"{tag}_burn_board", "wing", {rows[1]: 1.5}, burn, False, COL["pcb"], "two burn resistors, line tie"),
        ]
        for i, d in enumerate(doubl):
            spec.append((f"{tag}_hinge_doubler_{i}", "frame", {"5": 1.0}, d, False, COL["al"], "inside shear panel"))
        for name, grp, mel, shape, mv, col, note in spec:
            st = shape if sign > 0 else mirror_x(shape)
            if mv:
                dp = dep(shape)
                dp = dp if sign > 0 else mirror_x(dp)
            else:
                dp = st
            out.append(Part(name, grp, mel, st, dp, col, note=note, moves=tag if mv else ""))
        return out
    parts += wing_set(+1)
    parts += wing_set(-1)

    # ---------------- AntS (row 19) -----------------------------------------------------------------------
    aw, at, an = p["ants_w"] / 2.0, p["ants_t"], p["ants_notch"]
    housing = B(-aw, aw, -aw, aw, zp["mz_out"] - at, zp["mz_out"])
    housing = cut(housing, *[B(sx * (aw - an) - (an if sx < 0 else 0), sx * (aw - an) + (an if sx > 0 else 0),
                               sy * (aw - an) - (an if sy < 0 else 0), sy * (aw - an) + (an if sy > 0 else 0),
                               -70, -40) for sx in (1, -1) for sy in (1, -1)])
    add(Part("AntS_housing", "ants", {"19": 81.5}, housing, color=COL["ants"],
             note="ISIS AntS 1U-3U 98 x 98 x 7 mm, corner notches for the rail standoffs"))
    zt = zp["mz_out"] - at / 2.0
    tw, tt = p["tape_w"] / 2.0, p["tape_t"] / 2.0
    # stowed tapes are inside the housing: their mass is carried by the housing until deployment
    # elements: (name, axis, sign, length, grams).  Option A (chair decision, BASELINE 7): one 2 m along +X,
    # one 70 cm along -X.  "hybrid": the MEL case of MASS_BUDGET v3.0 6.1 (2 m +-X, 70 cm +-Y).
    elems = [("VHF", "x", 1, p["vhf_len"], 2.5), ("UHF", "x", -1, p["uhf_len"], 1.0)]
    if p.get("ants_config") == "hybrid":
        elems = [("VHF_px", "x", 1, p["vhf_len"], 2.5), ("VHF_mx", "x", -1, p["vhf_len"], 2.5),
                 ("UHF_py", "y", 1, p["uhf_len"], 1.0), ("UHF_my", "y", -1, p["uhf_len"], 1.0)]
    housing_g = 85.0 - sum(e[4] for e in elems)
    parts[-1].mel = {"19": housing_g}
    for i, (nm, axe, sg, ln, g) in enumerate(elems):
        if axe == "x":
            dep = B(aw, aw + ln, p["tape_y"] - tt, p["tape_y"] + tt, zt - tw, zt + tw) if sg > 0 else \
                B(-aw - ln, -aw, p["tape_y"] - tt, p["tape_y"] + tt, zt - tw, zt + tw)
        else:
            dep = B(-tt, tt, aw, aw + ln, zt - tw, zt + tw) if sg > 0 else B(-tt, tt, -aw - ln, -aw, zt - tw, zt + tw)
        yy = aw - 1.5 - 0.6 * i
        st = B(-aw + 11, aw - 11, yy, yy + 0.5, zt - tw, zt + tw)       # wound inside the housing
        add(Part(f"AntS_{nm}_element", "ants", {"19": g}, st, dep, COL["ants"], moves="ANT",
                 note=f"{'2 m' if nm.startswith('VHF') else '70 cm'} tape spring {ln:.0f} mm x {p['tape_w']} mm, "
                      f"deployed along {'+' if sg > 0 else '-'}{axe.upper()}"))

    # ---------------- battery assembly (rows 13-16) -------------------------------------------------------
    zc0, zc1 = -44.9, -20.9
    zbc = (zc0 + zc1) / 2.0
    cells = [C("x", (0, y, zbc), 9.25, -32.5, 32.5) for y in (-27.75, -9.25, 9.25, 27.75)]
    for i, c in enumerate(cells):
        add(Part(f"cell_{i + 1}", "battery", {"13": 50.0}, c, color=COL["batt"], note="INR18650-35E, 50.0 g max"))
    caps = [B(32.5, 35.5, -40, 40, zc0, zc1), B(-35.5, -32.5, -40, 40, zc0, zc1)]
    caps = [cut(cp, *[C("x", (0, y, zbc), 8.0, -40, 40) for y in (-27.75, -9.25, 9.25, 27.75)]) for cp in caps]
    bars = [B(-32.5, 32.5, 37.0, 40.0, zc0, zc1), B(-32.5, 32.5, -40.0, -37.0, zc0, zc1)]
    ears = [B(min(sx * 35.5, sx * 41.5), max(sx * 35.5, sx * 41.5), sy * 31 - 5, sy * 31 + 5, -38.9, -36.4)
            for sx in (1, -1) for sy in (1, -1)]
    add(Part("battery_clamp", "battery", {"15": 34.0}, fuse(*(caps + bars + ears)), color=COL["al"],
             note="Al 6061-T6 end caps, side bars and four isolator ears"))
    iso = [fuse(C("z", (sx * 38.5, sy * 31, 0), 3.0, zp["mz_in"], -38.9), C("z", (sx * 38.5, sy * 31, 0), 2.25,
                                                                              -36.4, -34.9))
           for sx in (1, -1) for sy in (1, -1)]
    add(Part("battery_isolators", "battery", {"16": 6.0}, fuse(*iso), color=COL["comp"],
             note="4 G-10CR shouldered bushings 6 mm dia x 8 mm, Ti-6Al-4V M2.5, disc springs, heads"))
    add(Part("battery_heaters_wrap", "battery", {"16": 2.0, "33": 10.0},
             B(-32.5, 32.5, -37.0, 37.0, zc1 - 0.3, zc1), color=COL["comp"], ghost=True,
             note="heater films, cutout, VDA tape: carried as a layer on the pack"))
    add(Part("battery_sense_board", "battery", {"14": 15.0}, B(-35, 35, -40, 40, -20.4, -17.9), color=COL["pcb"],
             note="interconnect and sense board on the clamp (flying leads, no stack connector)"))

    # ---------------- deck, rods, GNSS receiver -----------------------------------------------------------
    # skeleton deck (MCAD-02): 3 mm rim, column bosses, a strap under the X rod, two bars under the GNSS receiver and
    # a strap to the upper centre support; Al 6061-T6 1.0 mm, about 8.6 g (row 5 allocates 9 g)
    zd0, zd1 = -16.5, -15.5
    rim = cut(B(-45.0, 45.0, -47.9, 47.9, zd0, zd1), B(-42.0, 42.0, -44.9, 44.9, zd0 - 1, zd1 + 1))
    deck = fuse(rim, *[C("z", (sx * p["col_x"], sy * p["col_y"], 0), 5.0, zd0, zd1) for sx in (1, -1) for sy in (1, -1)],
                B(-25.0, 25.0, 30.0, 42.0, zd0, zd1), B(-42.0, 42.0, -42.0, -36.0, zd0, zd1),
                B(-42.0, 42.0, -4.0, 2.0, zd0, zd1), B(-3.0, 3.0, 2.0, 30.0, zd0, zd1),
                C("z", (p["cs_x"], p["cs_y"], 0), 5.0, zd0, zd1))
    deck = cut(deck, *colholes, B(-60, -35.5, -6.5, 6.5, -60, 60), C("z", (p["cs_x"], p["cs_y"], 0), 1.6, -60, 60))
    add(Part("torquer_deck", "frame", {"5": 9.0}, deck, color=COL["al"],
             note="skeleton deck, Al 1.0 mm (rim, bosses, straps): X rod, GNSS receiver, rod clips (MCAD-02)"))
    add(Part("rod_X", "adcs", {"27r": 16.5}, C("x", (0, 36.0, -10.25), 5.25, -23.5, 23.5), color=COL["rod"],
             note="CubeSpace CR0002, 10.5 x 10.5 x 47 mm, 16.5 g (CubeSpace Gen 2 brochure)"))
    add(Part("rod_Y", "adcs", {"27r": 16.5}, C("y", (42.0, 0, -32.9), 5.25, -23.5, 23.5), color=COL["rod"]))
    add(Part("rod_Z", "adcs", {"27r": 16.5}, C("z", (-41.5, 0.0, 0), 5.25, -44.0, 3.0), color=COL["rod"],
             note="through notches in the deck and the EPS board"))
    clips = fuse(B(-20, -16, 30, 42, -15.5, -14.5), B(16, 20, 30, 42, -15.5, -14.5),
                 B(36.0, 48.0, -20, -16, -38.5, -27.0), B(36.0, 48.0, 14, 18, -38.5, -27.0),
                 B(-48.0, -36.0, -7.0, 7.0, -40.0, -38.0))
    add(Part("rod_clips", "adcs", {"28r": 8.0}, clips, color=COL["al"], ghost=True))
    add(Part("GNSS_receiver", "gnss", {"29": 24.0}, B(-20, 30, -45, 5, -15.5, -6.5), color=COL["gnss"],
             note="SkyFox piNAV-NG/FM, 24 g; envelope allocation 50 x 50 x 9 mm (open: vendor drawing)"))

    # ---------------- PC/104 stack ------------------------------------------------------------------------
    px, py, pt = p["pcb_x"] / 2.0, p["pcb_y"] / 2.0, p["pcb_t"]
    koz = [C("z", (sx * p["col_x"], sy * p["col_y"], 0), 3.5, -60, 60) for sx in (1, -1) for sy in (1, -1)]
    cs = C("z", (p["cs_x"], p["cs_y"], 0), 3.5, -60, 60)
    cs2 = C("z", (p["cs_x"], p["cs2_y"], 0), 3.5, -60, 60)          # second support, EPS -> OBC -> carrier
    cs_rf = C("z", (0, 0, 0), 3.5, -60, 60)
    zrod_notch = B(-60, -35.5, -6.5, 6.5, -60, 60)

    def board(name, z0, bot, top, mel, extra_cut=(), comp_cut=(), top_shape=None, col=COL["pcb"], note=""):
        sub = cut(B(-px, px, -py, py, z0, z0 + pt), *colholes, *extra_cut)
        add(Part(name, "stack", {k: v * 0.35 for k, v in mel.items()}, sub, color=col, note=note))
        comps = []
        if bot > 0:
            comps.append(B(-px + 1, px - 1, -py + 1, py - 1, z0 - bot, z0))
        if top_shape is not None:
            comps.append(top_shape)
        elif top > 0:
            comps.append(B(-px + 1, px - 1, -py + 1, py - 1, z0 + pt, z0 + pt + top))
        if comps:
            cm = cut(fuse(*comps), *koz, *extra_cut, *comp_cut)
            add(Part(name + "_parts", "stack", {k: v * 0.65 for k, v in mel.items()}, cm, color=COL["comp"],
                     note="component envelope"))

    board("EPS", P["z_eps"], 1.0, 5.0, {"12": 90.0}, extra_cut=(zrod_notch,), comp_cut=(cs, cs2),
          note="EPS: MPPT, BMS, PDU, burn drivers; centre support (STR-05); notch for the Z rod")
    gw = p["gyro_w"] / 2.0
    gyro_box = B(p["gyro_x"] - gw, p["gyro_x"] + gw, p["gyro_y"] - gw, p["gyro_y"] + gw, P["z_obc"] + pt,
                 P["z_obc"] + pt + p["gyro_h"])
    win = B(*p["car_win"], P["z_car"] - 3.0, P["z_car"] + 3.0)
    board("OBC", P["z_obc"], 1.0, 3.0, {"17": 45.0}, comp_cut=(cs, cs2, gyro_box, B(38.5, 43.5, -39.8, -27.2, 0, 20)),
          note="OBC STM32L4R5ZI, BMI088")
    add(Part("gyro_MG370", "adcs", {"25": 10.0}, gyro_box,
             color=COL["comp"], note="Epson M-G370PDT0 24 x 24 x 10 mm in socket X1 (14.0 mm above the OBC), "
                                        "through the carrier window"))
    z = P["z_car"]
    pc = p["car_t"]                                   # carrier substrate (2.0 mm from SQSO-MECH-CAD-001 B, D-11)
    jy = p["jet_y"]
    car_top = cut(B(-px + 1, px - 1, -py + 1, jy - 27.0, z + pc, z + pc + 2.0), *koz)
    car_bot = cut(B(-px + 1, px - 1, -py + 1, py - 1, z - 1.0, z), win, *koz, cs, cs2)
    frame = p["car_frame"]
    add(Part("Jetson_carrier", "stack", {"21": 14.0}, cut(B(-px, px, -py, py, z, z + pc), *colholes, win,
                                                          *([C("z", (x_, y_, 0), 1.1, z - 1, z + pc + 1)
                                                             for x_, y_ in p["frame_screws"]] if frame else [])),
             color=COL["pcb"], note=f"custom carrier, {pc:.1f} mm; centre supports; 27 x 27 mm window for the gyro (C3)"))
    if frame:                                         # alternative of D-11 (not the design): Al frame and screws
        fw, fh = p["car_frame_w"], p["car_frame_h"]
        wx0, wx1, wy0, wy1 = p["car_win"]
        wframe = cut(B(wx0 - fw, wx1 + fw, wy0 - fw, wy1 + fw, z - fh, z), B(wx0, wx1, wy0, wy1, z - fh - 1, z + 1))
        pdw, pdl = p["frame_pad"]
        pads, sh_f, heads = [], [], []
        for sx_, sy_ in p["frame_screws"]:
            if sx_ < (wx0 + wx1) / 2:
                pads.append(B(wx0 - pdw, wx0, sy_ - pdl / 2, sy_ + pdl / 2, z - fh, z))
            else:
                pads.append(B(wx1, wx1 + pdw, sy_ - pdl / 2, sy_ + pdl / 2, z - fh, z))
            sh_f.append(C("z", (sx_, sy_, 0), 1.0, z - fh - 1, z + 3))
            heads.append(fuse(C("z", (sx_, sy_, 0), 1.75, z + pc, z + pc + 1.1),
                              C("z", (sx_, sy_, 0), 0.99, z - fh + 0.2, z + pc)))
        wframe = cut(fuse(wframe, *pads), *sh_f)
        car_bot = cut(car_bot, B(wx0 - fw - 0.5, wx1 + fw + 0.5, wy0 - fw - 0.5, wy1 + fw + 0.5, z - 2, z + 0.1),
                      *[B(b.BoundingBox().xmin - 0.5, b.BoundingBox().xmax + 0.5, b.BoundingBox().ymin - 0.5,
                          b.BoundingBox().ymax + 0.5, z - 2, z + 0.1) for b in pads])
        add(Part("Jetson_carrier_parts", "stack", {"21": 23.0}, fuse(car_top, car_bot), color=COL["comp"]))
        add(Part("carrier_window_frame", "stack", {"21": 2.2}, wframe, color=COL["al"],
                 note="Al 6061 frame 2.5 x 3.0 mm bonded round the gyro window, carrier bottom side, four tapped pads"))
        add(Part("carrier_frame_screws", "stack", {"34": 1.0}, fuse(*heads), color=COL["rod"],
                 note="H10-H13: M2 A286 ISO 7380 button-head screws into the frame pads (3 mm engagement)"))
    else:
        add(Part("Jetson_carrier_parts", "stack", {"21": 25.2}, fuse(car_top, car_bot), color=COL["comp"]))
    zs = z + pc
    add(Part("SODIMM_connector", "stack", {"21": 0.0, "22": 2.0}, B(-36, 36, jy - 26.5, jy - 22.5, zs, zs + 4.0),
             color=COL["comp"], note="260-pin SO-DIMM, 4 mm stack height (allocation)"))
    zm = zs + 4.0
    hx, hy = p["mod_hole"]
    so = [C("z", (sx * hx, jy + hy, 0), p["mod_so_d"] / 2, zs, zm) for sx in (1, -1)]
    add(Part("module_standoffs", "stack", {"21": 0.8}, fuse(*so), color=COL["rod"],
             note="MP2/MP3 Wurth 9774040360R steel SMT spacers M3 x 4.0 mm at the module's far-edge holes (M1)"))
    sw, pw = p["soc_w"] / 2, p["ped_w"] / 2
    soc = B(-sw, sw, jy - sw, jy + sw, zm + 1.2, zm + 2.7)                      # SoC lid, 1.5 mm (allocation)
    others = cut(B(-20, 20, jy - 16, jy + 12, zm + 1.2, zm + 2.2), B(-pw, pw, jy - pw, jy + pw, zm, zm + 3))
    add(Part("Jetson_module", "stack", {"20": 31.0},
             fuse(B(-34.8, 34.8, jy - 22.5, jy + 22.5, zm, zm + 1.2),
                  cut(B(-30, 30, jy - 20, jy + 20, zm - 1.5, zm), *[C("z", (sx * hx, jy + hy, 0), 3.5, zm - 2, zm + 1)
                                                                    for sx in (1, -1)]),
                  soc, others),
             color=COL["pcb"], note="Jetson Orin Nano 8 GB 69.6 x 45 mm (NVIDIA SO-DIMM form factor); SoC lid 20 x 20 x 1.5 "
                                    "mm, DRAM and regulators 1.0 mm (envelopes until the NVIDIA STEP model is used)"))
    zt2 = zm + 2.7
    zsp = zt2 + 1.5
    pad_soc = B(-sw, sw, jy - sw, jy + sw, zt2, zt2 + p["pad_soc"])                 # 1.0 mm pad at 25 %
    pad_oth = cut(B(-20, 20, jy - 16, jy + 12, zm + 2.2, zm + 2.2 + p["pad_other"]),   # 2.0 mm pads at 25 %
                  B(-pw, pw, jy - pw, jy + pw, zm, zsp))
    add(Part("Jetson_TIM_clamp", "stack", {"22": 18.0}, fuse(B(-34.8, -31.8, jy - 22.5, jy + 22.5, zm + 1.2, zsp),
                                                          B(31.8, 34.8, jy - 22.5, jy + 22.5, zm + 1.2, zsp),
                                                          pad_soc, pad_oth),
             color=COL["comp"], note="clamp bars at both short edges (STR-07); gap pads on the SoC lid (1.0 mm) and on the "
                                     "DRAM and regulators (2.0 mm), both at 25 % deflection (SQSO-JET-BRD-001 14)"))
    spreader = fuse(B(-40, 40, jy - 27.0, jy + 33.0, zsp, zsp + 5.0),
                    B(-pw, pw, jy - pw, jy + pw, zt2 + p["pad_soc"], zsp),                       # pedestal (M1)
                    B(-20, 20, jy - 16, jy + 12, zm + 2.2 + p["pad_other"], zsp),              # second face
                    B(-wx["shear_in"], -40, -20, 20, zsp + 2.0, zsp + 5.0),
                    B(40, wx["shear_in"], -20, 20, zsp + 2.0, zsp + 5.0))
    add(Part("heat_spreader", "stack", {"23": 66.6, "5": 15.0}, spreader, color=COL["al"],
             note="Al 6061 80 x 60 x 5 mm with 3 mm flanges bolted (4 x M2.5) to the +-X shear panels; pedestal 22 x 22 x "
                  "0.75 mm over the SoC lid (flatness 0.05 mm) and a 0.5 mm second face over the DRAM and regulators"))
    rf_notch = B(37.0, 60, -60, -26.0, P["z_rf"] + pt - 0.1, P["z_rf"] + pt + 8.0)       # shield notch for J1 (RF C16)
    rf_conn = [B(38.5, 43.5, -39.8, -27.2, P["z_rf"] + pt - 0.1, P["z_rf"] + pt + 9.0),
               B(-31.5, -20.5, -47.0, -41.0, P["z_rf"] + pt - 0.1, P["z_rf"] + pt + 9.0)]
    rf_top = fuse(cut(B(-42, 42, P["shield_y0"], P["shield_y1"], P["z_rf"] + pt, P["z_rf"] + pt + 7.0), rf_notch),
                  B(-px + 1, px - 1, P["shield_y1"], py - 1, P["z_rf"] + pt, P["z_rf"] + pt + 2.0),
                  B(-px + 1, px - 1, -py + 1, P["shield_y0"], P["z_rf"] + pt, P["z_rf"] + pt + 2.0))
    rf_top = cut(rf_top, *rf_conn)
    board("RF", P["z_rf"], 1.5, 0.0, {"18": 120.0}, comp_cut=(cs_rf,), top_shape=rf_top,
          note="RF board with shield 84 x 60 x 7 mm (stiffener); keep-out <= 2 mm under the magnetometer")
    # centre supports and columns (row 34)
    levels = [-15.5, P["z_eps"], P["z_eps"] + pt, P["z_obc"], P["z_obc"] + pt, P["z_car"], P["z_car"] + p["car_t"], zsp]
    cseg = [C("z", (p["cs_x"], p["cs_y"], 0), p["col_d"] / 2, levels[i], levels[i + 1]) for i in (0, 2, 4, 6)]
    cseg.append(C("z", (0, 0, 0), p["col_d"] / 2, zsp + 5.0, P["z_rf"]))
    cseg += [C("z", (p["cs_x"], p["cs2_y"], 0), p["col_d"] / 2, levels[i], levels[i + 1]) for i in (2, 4)]
    # (0, -21) under the Jetson module: low-head M2 fastener on the carrier's top face (M1), head Ø3.5 x 1.1 mm (ISO 7380)
    cseg.append(C("z", (p["cs_x"], p["cs2_y"], 0), 1.75, P["z_car"] + p["car_t"], P["z_car"] + p["car_t"] + 1.1))
    add(Part("centre_supports", "stack", {"34": 2.0}, fuse(*cseg), color=COL["al"],
             note="centre supports (STR-05): (0, +21) deck-EPS-OBC-carrier-spreader, (0, -21) EPS-OBC-carrier, "
                  "(0, 0) spreader-RF"))
    stops = [zp["mz_in"], -16.5, -15.5, P["z_eps"], P["z_eps"] + pt, P["z_obc"], P["z_obc"] + pt, P["z_car"],
             P["z_car"] + p["car_t"], P["z_rf"], P["z_rf"] + pt, zp["pz_in"]]
    cols = []
    for sx in (1, -1):
        for sy in (1, -1):
            for i in range(0, len(stops) - 1, 2):
                cols.append(C("z", (sx * p["col_x"], sy * p["col_y"], 0), p["col_d"] / 2, stops[i], stops[i + 1]))
    add(Part("stack_columns", "stack", {"34": 8.0}, fuse(*cols), color=COL["al"],
             note="4 M3 A286 studs in Al hex spacers 5 AF, tied to both end frames (STR-06)"))

    # ---------------- +Z face: magnetometer, GNSS patch, patch connector ----------------------------------
    mxy = (p["mag_x"], p["mag_y"])
    aw, at_, ap = p["a2_w"] / 2.0, p["a2_t"], p["a2_parts"]
    za = zp["pz_in"]
    a2s = [(mxy[0] + (x - 24.0), mxy[1] + (y - 27.0)) for x, y in p["a2_screws"]]
    a2b = cut(B(mxy[0] - aw, mxy[0] + aw, mxy[1] - aw, mxy[1] + aw, za - at_, za), *[C("z", (x, y, 0), 1.1, 0, 60) for x, y in a2s])
    a2p = cut(B(mxy[0] - aw + 1, mxy[0] + aw - 1, mxy[1] - aw + 1, mxy[1] + aw - 1, za - at_ - ap, za - at_),
              *[C("z", (x, y, 0), 2.6, 0, 60) for x, y in a2s])
    add(Part("magnetometer", "adcs", {"24": 2.0}, a2b, color=COL["pcb"],
             note="board A2 (OBC item C13): 24 x 24 x 1.6 mm, 2 layers, back face on the +Z end frame (isolated: no frame bond)"))
    add(Part("magnetometer_parts", "adcs", {"24": 7.5, "26": 2.0}, a2p, color=COL["comp"],
             note="RM3100: MagI2C, Sen-XY-f 6.0 x 2.1 x 2.2, Sen-Z-f 3.0 x 3.0 x 5.75 mm (PNI); J1 toward the +X wall gap"))
    a2f = [fuse(C("z", (x, y, 0), 1.75, zp["pz_out"], zp["pz_out"] + 1.3), C("z", (x, y, 0), 1.0, za - at_ - 1.6, zp["pz_out"]),
                C("z", (x, y, 0), 2.0, za - at_ - 1.6, za - at_)) for x, y in a2s]
    add(Part("magnetometer_fasteners", "adcs", {"24": 0.5}, fuse(*a2f), color=COL["al"],
             note="3 x M2 Ti-6Al-4V screws (heads on the +Z face, outside the GNSS patch), brass nuts, polyimide washers"))
    pw = p["patch_w"] / 2.0
    add(Part("GNSS_patch", "gnss", {"30": 20.0}, B(-pw, pw, -pw, pw, zp["pz_out"], zp["pz_out"] + p["patch_t"]),
             color=COL["gnss"], note="Spacemanic ZENITH V3a 50 x 50 mm; thickness allocation 5 mm"))
    add(Part("GNSS_patch_adapter", "gnss", {"31": 6.0, "5": 2.0}, C("z", (0, 0, 0), 5.0, zp["pz_in"] - 4.0,
                                                                     zp["pz_in"]), color=COL["gnss"]))

    # ---------------- harness and distributed allocations (rows 26, 31, 32, 33, 34, 35) -------------------
    hch = [B(45.4 if sx > 0 else -48.4, 48.4 if sx > 0 else -45.4, 24.0 if sy > 0 else -33.9,
             33.9 if sy > 0 else -24.0, zp["mz_in"], zp["pz_in"]) for sx, sy in ((1, 1), (-1, 1), (-1, -1))]
    # (+X, -Y) bundle (ICX E12, amended): y -24.0 ... -33.9 below z -26 (the Y rod's -Y end reaches y -23.5 at
    # z -38.2 ... -27.6), y -20.5 ... -27.0 above it (0.5 mm from the spreader flange at y -20), leaving
    # y -28.5 ... -38.5 of the +X wall gap to the W4A ribbon from OBC J2 to RF J1
    hch.append(fuse(B(45.4, 48.4, -33.9, -24.0, zp["mz_in"], -26.0), B(45.4, 48.4, -27.0, -20.5, -26.0, zp["pz_in"])))
    add(Part("harness_channels", "harness", {"32": 77.0, "31": 20.0, "26": 3.0}, fuse(*hch),
             color=COL["harness"], note="four bundles in the +-X wall gaps (the +X -Y one narrowed to 3 x 6.5 mm above z -26)"))
    j2z = P["z_obc"] + pt
    w4a = fuse(B(47.4, 48.4, -38.5, -28.5, j2z + 8.0, P["z_rf"] + pt + 9.0),          # flat, vertical in the gap
               B(43.0, 48.4, -38.5, -28.5, P["z_rf"] + pt + 8.0, P["z_rf"] + pt + 9.0),   # over J1 to the gap
               B(43.0, 48.4, -38.5, -28.5, j2z + 7.0, j2z + 8.0))                       # from OBC J2 to the gap
    add(Part("ribbon_W4A", "harness", {"32": 1.0}, w4a, color=COL["harness"],
             note="W4A FTSH-108 ribbon, 10 mm wide, OBC J2 (+41, -33.5) to RF J1 (+41, -33.5) in the +X wall gap"))
    add(Part("OBC_J2_mated", "stack", {"17": 0.0}, B(39.0, 43.0, -39.3, -27.7, j2z, j2z + 8.0), color=COL["comp"],
             note="OBC J2 FTSH-108 with its ribbon socket, mated-height allocation 8 mm (open: Samtec drawing)"))
    add(Part("RF_J1_mated", "stack", {"18": 0.0}, B(39.0, 43.0, -39.3, -27.7, P["z_rf"] + pt, P["z_rf"] + pt + 8.0),
             color=COL["comp"], note="RF J1 FTSH-108 mated, in the shield notch x > 37, y < -26 (allocation 8 mm)"))
    add(Part("RF_J2_mated", "stack", {"18": 0.0}, B(-31.0, -21.0, -46.5, -41.5, P["z_rf"] + pt, P["z_rf"] + pt + 6.0),
             color=COL["comp"], note="RF J2 Pico-Clasp 6 vertical, mated, at (-26, -44) (allocation 6 mm)"))
    add(Part("fasteners_staking_distributed", "dist", {"34": 24.0 if p["car_frame"] else 25.0, "35": 10.0},
             B(-45, 45, -47.9, 47.9, zp["mz_in"], zp["pz_in"]), ghost=True, color=COL["comp"],
             note="fasteners, inserts, staking and conformal coat, smeared over the stack volume"))
    add(Part("thermal_distributed", "dist", {"33": 20.0}, B(-45, 45, -47.9, 47.9, -16.5, zp["pz_in"]), ghost=True,
             color=COL["comp"], note="interface materials and MLI patches (stack)"))
    return parts


# ======================================================================================================
# MEL mass assignment
# ======================================================================================================
def mel_rows(option="rods"):
    its = mp.items()
    return {it.row: it for it in its if option in it.options}


def assign_masses(parts: List[Part], grown=False, option="rods") -> Dict[str, float]:
    """Mass (g) of each part.  A value v > 0 in part.mel is grams of the row's CBE; v = -1 shares the remainder
    of that row by volume.  Scaled to MEV by the row's growth when grown."""
    rows = mel_rows(option)
    masses = {pt.name: 0.0 for pt in parts}
    for r, it in rows.items():
        fixed = sum(pt.mel[r] for pt in parts if r in pt.mel and pt.mel[r] >= 0)
        shared = [pt for pt in parts if r in pt.mel and pt.mel[r] < 0]
        rem = it.cbe_g - fixed
        if not shared and abs(rem) > 1e-6:
            raise ValueError(f"MEL row {r}: CAD carries {fixed:.2f} g of {it.cbe_g:.2f} g")
        vol = sum(pt.stowed.Volume() for pt in shared)
        g = (1 + it.growth()) if grown else 1.0
        for pt in parts:
            if r in pt.mel and pt.mel[r] >= 0:
                masses[pt.name] += pt.mel[r] * g
        for pt in shared:
            masses[pt.name] += rem * pt.stowed.Volume() / vol * g
    for pt in parts:
        for r in pt.mel:
            if r not in rows:
                raise ValueError(f"part {pt.name}: MEL row {r} not in the {option} MEL")
    return masses


# ======================================================================================================
# Mass properties from the solids
# ======================================================================================================
def solid_props(shape: cq.Shape):
    pr = GProp_GProps()
    BRepGProp.VolumeProperties_s(shape.wrapped, pr)
    c = pr.CentreOfMass()
    m = pr.MatrixOfInertia()
    I = np.array([[m.Value(i, j) for j in (1, 2, 3)] for i in (1, 2, 3)])
    return pr.Mass(), np.array([c.X(), c.Y(), c.Z()]), I


@dataclass
class MassProps:
    mass_g: float
    cg_mm: np.ndarray
    I: np.ndarray                       # kg m^2 about the CG, tensor convention (Ixy = -sum m x y)

    def principal(self):
        w, v = np.linalg.eigh(self.I)
        return w, v

    def summary(self):
        w, v = self.principal()
        k = int(np.argmax(w))
        major = v[:, k]
        ang = math.degrees(math.acos(min(1.0, abs(major[1]))))
        d = np.diag(self.I)
        return dict(mass_g=self.mass_g, cg_mm=self.cg_mm.tolist(),
                    Ixx=d[0], Iyy=d[1], Izz=d[2], Ixy=self.I[0, 1], Ixz=self.I[0, 2], Iyz=self.I[1, 2],
                    principal=w.tolist(), major_axis=major.tolist(), major_from_Y_deg=ang,
                    ratio_Iyy=d[1] / max(d[0], d[2]), ratio_major=float(w[2] / w[1]))


def mass_props(parts: List[Part], masses: Dict[str, float], deployed: bool, cache: Dict) -> MassProps:
    rows = []
    for pt in parts:
        m = masses[pt.name]
        if m <= 0:
            continue
        shp = pt.deployed if (deployed and pt.deployed is not None) else pt.stowed
        key = (pt.name, deployed and pt.deployed is not None)
        if key not in cache:
            cache[key] = solid_props(shp)
        v, c, Iu = cache[key]
        rows.append((m, c, Iu * (m / v)))
    M = sum(r[0] for r in rows)
    cg = sum(r[0] * r[1] for r in rows) / M
    I = np.zeros((3, 3))
    for m, c, Ip in rows:
        d = c - cg
        I += Ip + m * (np.dot(d, d) * np.eye(3) - np.outer(d, d))
    return MassProps(M, cg, I * 1e-9)       # g mm^2 -> kg m^2


# ======================================================================================================
# Checks
# ======================================================================================================
def bbox(shapes):
    bb = None
    for s in shapes:
        b = s.BoundingBox()
        bb = [b.xmin, b.xmax, b.ymin, b.ymax, b.zmin, b.zmax] if bb is None else \
            [min(bb[0], b.xmin), max(bb[1], b.xmax), min(bb[2], b.ymin), max(bb[3], b.ymax),
             min(bb[4], b.zmin), max(bb[5], b.zmax)]
    return bb


def min_dist(a: cq.Shape, b: cq.Shape) -> float:
    d = BRepExtrema_DistShapeShape(a.wrapped, b.wrapped)
    d.Perform()
    return d.Value()


def common_volume(a: cq.Shape, b: cq.Shape) -> float:
    op = BRepAlgoAPI_Common(a.wrapped, b.wrapped)
    op.Build()
    if not op.IsDone():
        return float("nan")
    return cq.Shape.cast(op.Shape()).Volume()


def protrusion_stack(p=P):
    """STR-10: the stowed stack on the +-X faces, nominal and tolerance (worst case and RSS), from the rail plane."""
    t = tol_table()
    wx = wing_stowed_x(p)
    f = p["face"]
    chains = {
        "wing front cells": (wx["wf1"] - f, [t["rail_plane"], t["shear_face"], t["sub"], t["cell"], t["gap"],
                                             t["cell"], t["sub"], t["cell"]]),
        "hold-down line loop": (wx["line"] - f, [t["rail_plane"], t["shear_face"], t["post"], t["line"]]),
        "hinge spring (OD 4.0)": (p["hinge_x"] + p["spring_r"] - f, [t["rail_plane"], t["shear_face"],
                                                                     t["hinge_axis"], t["knuckle_r"]]),
    }
    out = {}
    for k, (nom, tols) in chains.items():
        wc = sum(tols)
        rss = math.sqrt(sum(x * x for x in tols))
        out[k] = dict(nominal=nom, worst=nom + wc, rss=nom + rss, limit=p["protrusion_max"],
                      margin_worst=p["protrusion_max"] - nom - wc, margin_rss=p["protrusion_max"] - nom - rss,
                      tolerances=tols)
    # the earlier analysis stack (SA-001 13.2, body panel on the rail plane, knuckle on the wing mid-plane)
    out["SA-001 A 13.2 stack (reference)"] = dict(nominal=6.1, worst=6.1 + 0.77, rss=6.1 + 0.30, limit=6.5,
                                                  margin_worst=6.5 - 6.87, margin_rss=6.5 - 6.40, tolerances=[])
    out["this stack without the 0.5 mm panel recess"] = dict(
        nominal=max(v["nominal"] for k, v in out.items() if k in chains) + p["panel_recess"],
        worst=max(v["worst"] for k, v in out.items() if k in chains) + p["panel_recess"],
        rss=max(v["rss"] for k, v in out.items() if k in chains) + p["panel_recess"], limit=6.5)
    o = out["this stack without the 0.5 mm panel recess"]
    o["margin_worst"], o["margin_rss"], o["tolerances"] = 6.5 - o["worst"], 6.5 - o["rss"], []
    return out


def run_checks(parts: List[Part], p=P, interference=True) -> Dict:
    zp = z_planes(p)
    res: Dict = {"items": []}

    def item(cid, req, value, limit, ok, note=""):
        res["items"].append(dict(id=cid, requirement=req, value=value, limit=limit, ok=None if ok is None else bool(ok),
                                 note=note))

    real = [pt for pt in parts if not pt.ghost]
    st = [pt.stowed for pt in real]
    bb = bbox(st)
    hw = p["W"] / 2.0
    # body: everything but the side protrusion zone
    item("CDS-1", "Length along Z (rail end to rail end), 113.5 +- 0.1 mm (CDS 2.2.1.1, App. B)", bb[5] - bb[4],
         "113.5 +- 0.1", abs((bb[5] - bb[4]) - p["L"]) <= p["L_tol"] + 1e-6)
    rails = [pt.stowed for pt in parts if pt.name.startswith("rail_")]
    rb = bbox(rails)
    item("CDS-2", "Rail envelope X and Y, 100.0 +- 0.1 mm", [rb[1] - rb[0], rb[3] - rb[2]], "100.0 +- 0.1",
         abs(rb[1] - rb[0] - 100) <= 0.1 and abs(rb[3] - rb[2] - 100) <= 0.1)
    item("CDS-3", "Rail width to the first protrusion >= 8.5 mm on each face (CDS 2.2.5)",
         dict(hinge_spring_edge_from_rail_edge=hw - (p["hinge_y"] + p["spring_r"]),
              panel_edge_from_rail_edge=hw - p["bp_w"] / 2.0,
              wing_edges=[hw - p["wing_y1"], hw + p["wing_y0"]]),
         ">= 8.5", min(hw - (p["hinge_y"] + p["spring_r"]), hw - p["bp_w"] / 2.0, hw - p["wing_y1"],
                       hw + p["wing_y0"]) >= p["rail"])
    item("CDS-4", "Rail ends >= 6.5 x 6.5 mm contact (CDS 2.2.8)", "8.5 x 8.5 solid standoff (less a 4.7 mm "
         "bore at the switch and plunger positions: 54.9 mm2 >= 42.3 mm2)", ">= 42.25 mm2",
         8.5 * 8.5 - math.pi * 2.35 ** 2 >= 42.25)
    item("CDS-5", "Rail contact >= 75 % of the length (CDS 2.2.9)", "100 % (continuous L-angle rail, 113.5 mm)",
         ">= 75 %", True)
    # protrusions on +-X / +-Y (stowed)
    side = {}
    for pt in real:
        b = pt.stowed.BoundingBox()
        side[pt.name] = max(b.xmax - hw, -b.xmin - hw, b.ymax - hw, -b.ymin - hw)
    worst = max(side, key=side.get)
    item("CDS-6", "No protrusion beyond 6.5 mm from the rail plane on +-X/+-Y (CDS 2.2.3), nominal CAD",
         dict(max_mm=side[worst], part=worst), "<= 6.5", side[worst] <= p["protrusion_max"])
    stack = protrusion_stack(p)
    w = max(("wing front cells", "hold-down line loop", "hinge spring (OD 4.0)"), key=lambda k: stack[k]["worst"])
    item("STR-10", "Stowed-wing stack incl. tolerances <= 6.5 mm (worst case)", dict(
        nominal=stack[w]["nominal"], worst_case=stack[w]["worst"], rss=stack[w]["rss"], governing=w),
         "<= 6.5", stack[w]["worst"] <= 6.5)
    # +-Z standoff clearance
    ants = [pt for pt in parts if pt.name == "AntS_housing"][0].stowed.BoundingBox()
    patch = [pt for pt in parts if pt.name == "GNSS_patch"][0].stowed.BoundingBox()
    t = tol_table()
    a_nom = zp["rail_end"] + ants.zmin
    a_wc = a_nom - t["ants_t"] - t["endplate"] - t["rail_len"]
    item("STR-11", "AntS below the -Z rail ends by >= 0.1 mm (CDS App. B note 2.2.1.2)",
         dict(nominal=a_nom, worst_case=a_wc, unrecessed_nominal=zp["rail_end"] - (p["face"] + p["ants_t"])),
         ">= 0.1", a_wc >= p["standoff_min"])
    p_nom = zp["rail_end"] - patch.zmax
    item("STD-2", "GNSS patch below the +Z rail ends by >= 0.1 mm", dict(
        nominal=p_nom, worst_case=p_nom - t["patch_t"] - t["endplate"] - t["rail_len"]), ">= 0.1",
         p_nom - 0.35 >= p["standoff_min"])
    # switches and plungers: flush with the standoff ends, inside the 8.5 standoff footprint
    sw = {pt.name: pt.stowed.BoundingBox() for pt in parts if pt.name[:2] in ("DS", "SP")}
    flush = all(abs(max(abs(b.zmin), abs(b.zmax)) - zp["rail_end"]) < 1e-6 for b in sw.values())
    inside = all(min(abs(b.xmin), abs(b.xmax)) >= hw - p["rail"] and min(abs(b.ymin), abs(b.ymax)) >= hw - p["rail"]
                 for b in sw.values())
    item("CDS-7", "Deployment switches (3, on 3 rails) and separation plungers (2): actuated flush with the "
         "standoff end, within the standoff footprint (CDS 2.3.2.3, 2.2.14.2)",
         {k: [round((b.xmin + b.xmax) / 2, 2), round((b.ymin + b.ymax) / 2, 2), "-Z" if b.zmin < 0 else "+Z"]
          for k, b in sw.items()}, "flush, inside", flush and inside)
    rb_ = [pt for pt in parts if pt.name == "RBF"][0].stowed.BoundingBox()
    item("CDS-8", "RBF pin <= 6.5 mm from the rail surface when inserted (CDS 2.3.5.2)", -rb_.ymin - hw, "<= 6.5",
         -rb_.ymin - hw <= 6.5, note=f"RBF at x = {p['rbf_x']}, z = {p['rbf_z']} on -Y")

    # interference: pairwise common volume, stowed and deployed
    if interference:
        for state in ("stowed", "deployed"):
            shapes = [(pt.name, pt.deployed if (state == "deployed" and pt.deployed is not None) else pt.stowed)
                      for pt in real]
            bbs = [s.BoundingBox() for _, s in shapes]
            clashes = []
            n = 0
            for i in range(len(shapes)):
                for j in range(i + 1, len(shapes)):
                    a, b = bbs[i], bbs[j]
                    if a.xmax < b.xmin or b.xmax < a.xmin or a.ymax < b.ymin or b.ymax < a.ymin or \
                            a.zmax < b.zmin or b.zmax < a.zmin:
                        continue
                    n += 1
                    pair = {shapes[i][0], shapes[j][0]}
                    if state == "stowed" and "AntS_housing" in pair and any(q.startswith("AntS_") and
                                                                            q.endswith("_element") for q in pair):
                        continue        # the stowed tapes are wound inside the housing by design
                    v = common_volume(shapes[i][1], shapes[j][1])
                    if v > 1e-3:
                        clashes.append((shapes[i][0], shapes[j][0], round(v, 3)))
            item(f"INT-{state}", f"No interference between solids, {state} ({n} pairs with overlapping boxes)",
                 clashes, "none", not clashes)
    # clearances
    byname = {pt.name: pt for pt in parts}
    body_static = fuse(*[byname[n].stowed for n in ("body_panel_px", "body_cells_px", "shear_px", "rail_pp",
                                                     "rail_pm", "body_panel_py", "body_cells_py")])
    wingparts = [byname[n] for n in ("W1_substrate", "W1_cells_back", "W1_cells_front", "W1_snubber",
                                     "W1_holddown_line")]
    ax0, ax1 = cq.Vector(p["hinge_x"], p["hinge_y"], 0), cq.Vector(p["hinge_x"], p["hinge_y"], 1)
    sweep = []
    for ang in [1, 2, 3, 5, 8, 10, 15, 20, 30, 45, 60, 75, 90]:
        wing = fuse(*[wp.stowed.rotate(ax0, ax1, ang) for wp in wingparts[:3]])
        sweep.append((ang, min_dist(wing, body_static)))
    mn = min(sweep, key=lambda x: x[1])
    item("SWP-1", "Wing W1 swept volume clear of the body panels, cells and rails (0-90 deg)", dict(
        min_clearance_mm=mn[1], at_deg=mn[0], samples=sweep), "> 0.3 (tolerance allowance)", mn[1] > 0.3)
    # AntS tapes vs wings
    zt = z_planes(p)["mz_out"] - p["ants_t"] / 2.0
    band_top = zt + p["tape_w"] / 2.0
    stowed_gap = p["wing_z0"] - band_top
    item("STR-16a", "AntS tape band (5 mm, centred in the 7 mm housing) to the stowed wing lower edge",
         dict(gap_mm=stowed_gap, housing_top_to_wing=p["wing_z0"] - z_planes(p)["mz_out"],
              wing_98mm_centred_gap=-49.0 - band_top), ">= 10 (MECH-001 7.2) or wings deploy first",
         stowed_gap >= 10.0, note="wings deploy first (BASELINE 4, 7); the stowed gap is the contingency case")
    vhf = [pt for pt in parts if pt.name.startswith("AntS_VHF")][0].deployed
    wdep = fuse(*[byname[n].deployed for n in ("W1_substrate", "W1_cells_back", "W1_cells_front")])
    d_dep = min_dist(vhf, wdep)
    item("STR-16b", "Deployed AntS VHF element to the deployed wing W1", d_dep, ">= 10", d_dep >= 10.0)
    # tape sweep: the tape springs straight from its root; envelope = band swept from the stowed (edge)
    # position to the deployed line in the face plane, i.e. the quadrant between the +Y edge and +X.
    # the deployed wings lie at y = 37.1-39.4, x >= 52.25: a tape whose root is on the +Y edge sweeps under them.
    item("STR-16c", "AntS element sweep (in the face plane) against the deployed wings: vertical separation",
         dict(band_top_z=band_top, wing_lower_edge_z=p["wing_z0"], vertical_gap_mm=stowed_gap),
         ">= 10 or root on the -Y half", None,
         note="OPEN: ISIS ICD (element root and sweep) not obtained: the root must be ordered on the -Y half "
              "(sweep away from the wings), VI-STR-10")
    # harness past the hinges
    flex = byname["W1_hinge_flex"].stowed.BoundingBox()
    item("HAR-1", "Hinge flex jumper inside the 6.5 mm envelope and clear of the 8.5 mm rail zone",
         dict(x_out=flex.xmax - hw, y_max=flex.ymax, bend_radius_mm=[1.0, 1.6]),
         "<= 6.5; y <= 41.5; bend radius >= IPC-2223 flex-to-install 0.82 mm",
         flex.xmax - hw <= 6.5 and flex.ymax <= 41.5,
         note="clock-spring flex in the -Z wing knuckle; torque allocation 0.5 mN m cold (hinge.py)")
    sp = byname["W1_springs"].stowed.BoundingBox()
    item("MEC-1", "Wing deployment springs (hinge.py: primary 7.5 mm, secondary 3.6 mm, OD 4.0) inside the hinge "
         "envelope", dict(x_out=sp.xmax - hw, y_max=sp.ymax, z_ranges=[[-47.6, -40.1], [44.8, 48.4]]),
         "x_out <= 6.5, y <= 41.5, within the free axis lengths", sp.xmax - hw <= 6.5 and sp.ymax <= 41.5)
    # (z ranges: primary -47.6 .. -40.1, 7.5 mm; secondary 44.8 .. 48.4, 3.6 mm)
    ear = byname["W1_hinges"].stowed.BoundingBox()
    item("STR-16d", "Hinge hardware below the wing to the AntS tape band", dict(
        vertical_gap_mm=ear.zmin - band_top, lateral_from_tape_line_mm=p["hinge_y"] - p["spring_r"] - p["tape_y"]),
         ">= 10 mm in at least one direction", (p["hinge_y"] - p["spring_r"] - p["tape_y"]) >= 10.0,
         note="the -Z ear sits 0.5 mm above the tape band but 36.75 mm from the 2 m tape line (y = 0)")
    # gyro in its socket through the carrier window (OBC_BOARD_DESIGN 9.2 and 24 C3)
    gyb = byname["gyro_MG370"].stowed
    gcl = {n: min_dist(gyb, byname[n].stowed) for n in ("Jetson_carrier", "Jetson_carrier_parts", "carrier_window_frame",
                                                         "Jetson_module", "SODIMM_connector", "centre_supports",
                                                         "stack_columns") if n in byname}
    gbb = gyb.BoundingBox()
    item("OBC-C3", "M-G370PDT0 in socket X1 (14.0 mm above the OBC top face) clear of the carrier, the Jetson module "
         "and the supports", dict(top_z=gbb.zmax, above_obc_mm=p["gyro_h"], carrier_bottom_z=p["z_car"],
                                  clearances_mm=gcl), ">= 1.0 mm (3-sigma relative board motion < 0.2 mm)",
         min(gcl.values()) >= 1.0, note="through a 27 x 27 mm window in the carrier (MECHANICAL_DESIGN D-11)")
    # Jetson carrier item M1: pedestal, module standoffs and the (0, -21) fastener against the window and the module
    wx0, wx1, wy0, wy1 = p["car_win"]
    hx, hy = p["mod_hole"]
    jy = p["jet_y"]
    win_box = B(wx0, wx1, wy0, wy1, p["z_car"], p["z_car"] + p["car_t"])
    so_edge = min(abs(wy0 - (jy + hy + p["mod_so_d"] / 2)), 99.0)
    ped_to_win = min_dist(B(-p["ped_w"] / 2, p["ped_w"] / 2, jy - p["ped_w"] / 2, jy + p["ped_w"] / 2, 27.0, 27.8), win_box)
    head_top = p["z_car"] + p["car_t"] + 1.1
    mod_under = p["z_car"] + p["car_t"] + 4.0 - 1.5
    head_to_win = math.hypot(max(wx0 - 1.75, 0, -1.75 - wx1), (wy0 - (p["cs2_y"] + 1.75)))
    # magnetometer board A2 (OBC item C13): fit in the +Z slot and the 20 mm non-ferromagnetic zone
    a2 = fuse(byname["magnetometer"].stowed, byname["magnetometer_parts"].stowed)
    a2bb = a2.BoundingBox()
    near = {}
    for pt in parts:
        if pt.ghost or pt.name.startswith("magnetometer") or pt.moves in ("ANT",):
            continue
        b = pt.stowed.BoundingBox()
        if b.xmin > a2bb.xmax + 20 or b.xmax < a2bb.xmin - 20 or b.ymin > a2bb.ymax + 20 or b.ymax < a2bb.ymin - 20 \
                or b.zmin > a2bb.zmax + 20 or b.zmax < a2bb.zmin - 20:
            continue
        d = min_dist(a2, pt.stowed)
        if d <= 20.0:
            near[pt.name] = round(d, 2)
    item("MAG-A2", "Magnetometer board A2 in the +Z slot (8 mm): Sen-Z-f 5.75 mm plus the 1.6 mm board; clearances",
         dict(bottom_z=a2bb.zmin, slot_bottom_z=z_planes(p)["pz_in"] - 8.0, rf_parts_mm=near.get("RF_parts"),
              within_20mm=near), "inside the slot; >= 1.0 mm to every part but its mounting face", a2bb.zmin >= z_planes(p)["pz_in"] - 8.0
         and min(v for k, v in near.items() if k != "endplate_pz") >= 1.0, note="every part listed within 20 mm must be non-ferromagnetic (MECHANICAL_DESIGN 4.4)")
    if "carrier_frame_screws" in byname:
        fs = byname["carrier_frame_screws"].stowed
        fcl = {n: round(min_dist(fs, byname[n].stowed), 2) for n in ("module_standoffs", "centre_supports", "stack_columns",
                                                                        "Jetson_module", "gyro_MG370")}
        walls = []
        wx0_, wx1_ = p["car_win"][0], p["car_win"][1]
        for sx_, sy_ in p["frame_screws"]:
            inner = (wx0_ - (sx_ + 0.8)) if sx_ < (wx0_ + wx1_) / 2 else ((sx_ - 0.8) - wx1_)
            outer = p["frame_pad"][0] - p["car_frame_w"] + p["car_frame_w"] - abs(sx_ - (wx0_ if sx_ < (wx0_ + wx1_) / 2 else wx1_)) - 0.8
            walls.append((round(inner, 2), round(outer, 2)))
        item("JET-H10", "Window-frame screws H10-H13 (M2, tap drill 1.6 mm) in their pads, clear of MP2, the supports and the module",
             dict(positions=p["frame_screws"], walls_inner_outer_mm=walls, clearances_mm=fcl), "walls >= 1.0 mm; >= 0.5 mm",
             min(min(w) for w in walls) >= 1.0 and min(fcl.values()) >= 0.5)
    item("JET-M1", "Spreader pedestal, module standoffs and the (0, -21) fastener clear of the gyro window and the module",
         dict(standoff_to_window_mm=so_edge, pedestal_to_window_mm=ped_to_win, fastener_head_to_window_mm=head_to_win,
              fastener_head_top_z=head_top, module_underside_parts_z=mod_under, head_to_module_parts_mm=mod_under - head_top),
         ">= 1.0 mm each", min(so_edge, ped_to_win, head_to_win, mod_under - head_top) >= 1.0,
         note="module hole positions and underside parts from envelopes: check against the NVIDIA drawing and STEP model")
    return res


# ======================================================================================================
# Export: STEP and SVG
# ======================================================================================================
def is_external(pt: Part) -> bool:
    """Outer mould line and dispenser-interface parts (SpaceX UG v11 Appendix C: no internal components)."""
    if pt.ghost or "burn_board" in pt.name or "hinge_doubler" in pt.name:
        return False
    return pt.group in ("panel", "wing", "ants") or pt.name.startswith(("rail_", "endplate_", "shear_", "DS", "SP",
                                                                        "RBF", "access_port", "GNSS_patch")) \
        and pt.name != "GNSS_patch_adapter" or pt.name.endswith("hinge_flex")


def export_step(parts: List[Part], path: str, deployed: bool, oml: bool = False):
    assy = cq.Assembly(name=("SMART-QSO_OML_" if oml else "SMART-QSO_") + ("deployed" if deployed else "stowed"))
    for pt in parts:
        if pt.ghost or (oml and not is_external(pt)):
            continue
        shp = pt.deployed if (deployed and pt.deployed is not None) else pt.stowed
        assy.add(shp, name=pt.name, color=cq.Color(*pt.color))
    assy.export(path) if hasattr(assy, "export") else assy.save(path, exportType="STEP")
    return os.path.getsize(path)


def hlr_lines(shape: cq.Shape, view: Tuple[float, float, float], xdir: Tuple[float, float, float]):
    """Hidden-line projection: returns (visible, hidden) lists of 2D polylines in view coordinates (mm)."""
    from OCP.HLRBRep import HLRBRep_Algo, HLRBRep_HLRToShape
    from OCP.HLRAlgo import HLRAlgo_Projector
    from OCP.gp import gp_Ax2, gp_Pnt, gp_Dir
    from OCP.TopExp import TopExp_Explorer
    from OCP.TopAbs import TopAbs_EDGE
    from OCP.BRepAdaptor import BRepAdaptor_Curve
    from OCP.GCPnts import GCPnts_QuasiUniformDeflection
    from OCP.TopoDS import TopoDS
    algo = HLRBRep_Algo()
    algo.Add(shape.wrapped)
    ax = gp_Ax2(gp_Pnt(0, 0, 0), gp_Dir(*view), gp_Dir(*xdir))
    algo.Projector(HLRAlgo_Projector(ax))
    algo.Update()
    algo.Hide()
    hs = HLRBRep_HLRToShape(algo)

    def lines(comp):
        out = []
        if comp is None or comp.IsNull():
            return out
        ex = TopExp_Explorer(comp, TopAbs_EDGE)
        while ex.More():
            e = TopoDS.Edge_s(ex.Current())
            try:
                c = BRepAdaptor_Curve(e)
                d = GCPnts_QuasiUniformDeflection(c, 0.05)
                pts = [d.Value(i) for i in range(1, d.NbPoints() + 1)]
                out.append([(q.X(), q.Y()) for q in pts])
            except Exception:
                pass
            ex.Next()
        return out
    vis = lines(hs.VCompound()) + lines(hs.OutLineVCompound())
    hid = lines(hs.HCompound())
    return vis, hid


class SVG:
    def __init__(self, w, h, title):
        self.w, self.h = w, h
        self.el = []
        self.title = title

    def poly(self, pts, ox, oy, s, cls):
        d = " ".join(f"{ox + x * s:.2f},{oy - y * s:.2f}" for x, y in pts)
        self.el.append(f'<polyline class="{cls}" points="{d}"/>')

    def text(self, x, y, t, size=9, anchor="start", cls="t"):
        t = t.replace("&", "&amp;").replace("<", "&lt;")
        self.el.append(f'<text class="{cls}" x="{x:.1f}" y="{y:.1f}" font-size="{size}" text-anchor="{anchor}">{t}'
                       f'</text>')

    def line(self, x0, y0, x1, y1, cls="dim"):
        self.el.append(f'<line class="{cls}" x1="{x0:.2f}" y1="{y0:.2f}" x2="{x1:.2f}" y2="{y1:.2f}"/>')

    def rect(self, x0, y0, x1, y1, cls="env"):
        self.el.append(f'<rect class="{cls}" x="{min(x0, x1):.2f}" y="{min(y0, y1):.2f}" width="{abs(x1 - x0):.2f}" '
                       f'height="{abs(y1 - y0):.2f}"/>')

    def dim(self, x0, y0, x1, y1, off, label, vertical=False, size=8):
        """Dimension between two drawing points with extension offset off (px)."""
        if not vertical:
            yy = y0 + off
            self.line(x0, y0, x0, yy, "ext")
            self.line(x1, y1, x1, yy, "ext")
            self.el.append(f'<line class="dim" x1="{x0:.2f}" y1="{yy:.2f}" x2="{x1:.2f}" y2="{yy:.2f}" '
                           f'marker-start="url(#a)" marker-end="url(#a)"/>')
            self.text((x0 + x1) / 2, yy - 2, label, size, "middle")
        else:
            xx = x0 + off
            self.line(x0, y0, xx, y0, "ext")
            self.line(x1, y1, xx, y1, "ext")
            self.el.append(f'<line class="dim" x1="{xx:.2f}" y1="{y0:.2f}" x2="{xx:.2f}" y2="{y1:.2f}" '
                           f'marker-start="url(#a)" marker-end="url(#a)"/>')
            self.el.append(f'<text class="t" x="{xx - 3:.1f}" y="{(y0 + y1) / 2:.1f}" font-size="{size}" '
                           f'text-anchor="middle" transform="rotate(-90 {xx - 3:.1f} {(y0 + y1) / 2:.1f})">'
                           f'{label}</text>')

    def save(self, path):
        css = ("polyline.v{fill:none;stroke:#111;stroke-width:0.45}polyline.h{fill:none;stroke:#999;"
               "stroke-width:0.25;stroke-dasharray:1.5,1.5}line.dim{stroke:#0645ad;stroke-width:0.5}"
               "line.ext{stroke:#0645ad;stroke-width:0.3}line.call{stroke:#b00;stroke-width:0.5}"
               "rect.env{fill:none;stroke:#e07b00;stroke-width:0.5;stroke-dasharray:4,2}"
               "text.t{font-family:Helvetica,Arial,sans-serif;fill:#0645ad}text.c{font-family:Helvetica,Arial,"
               "sans-serif;fill:#b00}text.h{font-family:Helvetica,Arial,sans-serif;fill:#111;font-weight:bold}"
               "rect.frame{fill:none;stroke:#111;stroke-width:0.8}")
        with open(path, "w") as f:
            f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{self.w}" height="{self.h}" '
                    f'viewBox="0 0 {self.w} {self.h}"><title>{self.title}</title><style>{css}</style>'
                    f'<defs><marker id="a" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="5" markerHeight="5" '
                    f'orient="auto-start-reverse"><path d="M0,2 L10,5 L0,8 z" fill="#0645ad"/></marker></defs>'
                    f'<rect x="0" y="0" width="{self.w}" height="{self.h}" fill="#fff"/>'
                    + "".join(self.el) + "</svg>")


def draw_view(svg: SVG, shape, view, xdir, ox, oy, s, hidden=True):
    vis, hid = hlr_lines(shape, view, xdir)
    if hidden:
        for pl in hid:
            svg.poly(pl, ox, oy, s, "h")
    for pl in vis:
        svg.poly(pl, ox, oy, s, "v")


def drawings(parts: List[Part], outdir: str, checks: Dict, mprops: Dict):
    """Sheet 1: stowed, three views with the CDS envelope, rails, protrusions, switches and RBF.
       Sheet 2: deployed configuration (wings and AntS elements)."""
    real = [pt for pt in parts if not pt.ghost]
    ext = [pt for pt in real if pt.group in ("frame", "panel", "wing", "ants", "inhibit", "gnss", "harness")]
    stow = cq.Compound.makeCompound([pt.stowed for pt in ext])
    hw, re = P["W"] / 2, P["L"] / 2
    pr = P["protrusion_max"]
    zp = z_planes(P)
    wx = wing_stowed_x(P)
    s = 2.2                                                   # px per mm (sheet 1, about 1:1.3 on A3)
    svg = SVG(1100, 780, "SMART-QSO stowed configuration, three views")
    svg.el.append('<rect class="frame" x="5" y="5" width="1090" height="770"/>')
    # ---- view A: eye on +X (u = +y, v = +z)
    oxA, oyA = 205, 255
    draw_view(svg, stow, (1, 0, 0), (0, 1, 0), oxA, oyA, s)
    svg.text(oxA, oyA - (re + 16) * s, "VIEW A  from +X (wing W1 stowed)", 10, "middle", "h")
    svg.rect(oxA - (hw + pr) * s, oyA - re * s, oxA + (hw + pr) * s, oyA + re * s)
    svg.dim(oxA - hw * s, oyA + re * s, oxA + hw * s, oyA + re * s, 22, "100.0 +- 0.1 (rail to rail)")
    svg.dim(oxA - (hw + pr) * s, oyA + re * s, oxA + (hw + pr) * s, oyA + re * s, 40,
            "113.0 = protrusion envelope, 6.5 per side (CDS 2.2.3)")
    svg.dim(oxA - (hw + pr) * s, oyA - re * s, oxA - (hw + pr) * s, oyA + re * s, -14, "113.5 +- 0.1",
            vertical=True)
    svg.dim(oxA + (hw - P["rail"]) * s, oyA - re * s, oxA + hw * s, oyA - re * s, -8, "8.5", size=7)
    svg.dim(oxA + (hw + pr) * s, oyA - P["face"] * s, oxA + (hw + pr) * s, oyA - re * s, 12, "6.75", vertical=True,
            size=7)
    svg.line(oxA + 20 * s, oyA - (zp["mz_out"] - P["ants_t"] / 2) * s, oxA + 30 * s, oyA + (re + 30) * s, "call")
    svg.text(oxA + 31 * s, oyA + (re + 32) * s, f"AntS 98 x 98 x 7.0 on -Z, face recessed {P['mz_recess']}: outer face "
             f"{re + zp['mz_out'] - P['ants_t']:.2f} inside the rail ends (CDS App. B: >= 0.1)", 7, "middle", "c")
    # ---- view B: eye on +Y (u = -x, v = +z)
    oxB, oyB = 560, 255
    draw_view(svg, stow, (0, 1, 0), (-1, 0, 0), oxB, oyB, s)
    svg.text(oxB, oyB - (re + 16) * s, "VIEW B  from +Y (wing stack, hinge line)", 10, "middle", "h")
    svg.rect(oxB - (hw + pr) * s, oyB - re * s, oxB + (hw + pr) * s, oyB + re * s)
    svg.dim(oxB - (hw + pr) * s, oyB + re * s, oxB - hw * s, oyB + re * s, 22, "6.5", size=7)
    calls = [(wx["wf1"], f"wing front cells {wx['wf1'] - hw:.2f}"), (wx["line"], f"hold-down line {wx['line'] - hw:.2f}"),
             (P["hinge_x"] + P["spring_r"], f"hinge spring {P['hinge_x'] + P['spring_r'] - hw:.2f}")]
    for i, (xv, lab) in enumerate(calls):
        u = -xv
        y0 = oyB + P["wing_z0"] * -s
        yy = oyB + (re + 18 + 9 * i) * s
        svg.line(oxB + u * s, y0, oxB + u * s, yy, "call")
        svg.line(oxB + u * s, yy, oxB + u * s + 18, yy, "call")
        svg.text(oxB + u * s + 20, yy + 3, lab + " mm from the rail plane (nominal)", 7, "start", "c")
    for i, (xx, zz, lab) in enumerate(((P["rbf_x"], P["rbf_z"], f"RBF on -Y (x {P['rbf_x']:+.0f}, z {P['rbf_z']:+.0f}), "
                                        f"head {checks_value(checks, 'CDS-8'):.2f} from the rail plane (hidden)"),
                                       (P["port_x"], P["port_z"], f"access port on -Y (x {P['port_x']:+.0f}, "
                                        f"z {P['port_z']:+.0f}) (hidden)"))):
        u, v = -xx, zz
        yy = oyB + (re + 45 + 9 * i) * s
        svg.line(oxB + u * s, oyB - v * s, oxB + u * s, yy, "call")
        svg.line(oxB + u * s, yy, oxB + u * s + 10, yy, "call")
        svg.text(oxB + u * s + 12, yy + 3, lab, 7, "start", "c")
    # ---- view C: eye on +Z (u = +x, v = +y)
    oxC, oyC = 905, 255
    draw_view(svg, stow, (0, 0, 1), (1, 0, 0), oxC, oyC, s)
    svg.text(oxC, oyC - (re + 16) * s, "VIEW C  from +Z", 10, "middle", "h")
    svg.rect(oxC - (hw + pr) * s, oyC - (hw + pr) * s, oxC + (hw + pr) * s, oyC + (hw + pr) * s)
    svg.dim(oxC - hw * s, oyC + hw * s, oxC + hw * s, oyC + hw * s, 30, "100.0 +- 0.1")
    svg.dim(oxC - (hw + pr) * s, oyC - hw * s, oxC - (hw + pr) * s, oyC + hw * s, -10, "100.0 +- 0.1", vertical=True)
    sw = [("DS1 -Z", 46, 46, 1), ("SP1 -Z", -46, 46, -1), ("DS2 -Z", -46, -46, -1), ("DS3 +Z, SP2 -Z", 46, -46, 1)]
    for k, x, y, side in sw:
        px_, py_ = oxC + x * s, oyC - y * s
        tx = oxC + side * (hw + pr + 1) * s
        ty = py_ + (-16 if y > 0 else 16)
        svg.line(px_, py_, tx, ty, "call")
        svg.text(tx + (2 if side > 0 else -2), ty + (-2 if y > 0 else 9), k, 7, "start" if side > 0 else "end", "c")
    svg.text(oxC, oyC - (re + 10) * s, f"GNSS patch 50 x 50 x {P['patch_t']:.0f} on +Z: "
             f"{checks_value(checks, 'STD-2')['nominal']:.2f} inside the +Z rail ends", 7, "middle", "c")
    # ---- notes
    st = checks_value(checks, "STR-10")
    stack = protrusion_stack(P)
    notes = [
        "SMART-QSO 1U  |  SQSO-MECH-CAD-001 Sheet 1: stowed configuration  |  dimensions mm  |  envelope per CDS Rev 14.1 "
        "drawing CDS-14-001 (1U)",
        "Rails: 8.5 x 8.5 mm square envelope (L-angle with 1.5 mm legs between the end frames, solid 8.5 x 8.5 x 6.75 mm "
        "standoffs at both ends), continuous over 113.5 mm (100 % contact); hard anodized Type III, Ra < 1.6 um, "
        "edges R >= 1 mm (frame vendor).",
        "Dashed orange: the 6.5 mm protrusion envelope on +-X/+-Y. First protrusion >= 9.0 mm from every rail edge "
        "(CDS 2.2.5: >= 8.5).",
        f"Stowed stack from the rail plane: wing front cells {stack['wing front cells']['nominal']:.2f}, hold-down line "
        f"{stack['hold-down line loop']['nominal']:.2f}, hinge spring {stack['hinge spring (OD 4.0)']['nominal']:.2f} mm "
        f"(governing); worst-case tolerance stack {st['worst_case']:.2f} mm, RSS {st['rss']:.2f} mm (STR-10).",
        "Deployment switches (inhibits, BASELINE 4): DS1 rail +X+Y at -Z, DS2 rail -X-Y at -Z, DS3 rail +X-Y at +Z, each "
        "at (+-46.0, +-46.0) mm in its standoff; separation plungers SP1 rail -X+Y and SP2 rail +X-Y at -Z "
        "(CDS 2.2.14).",
        "All switches and plungers actuated or compressed flush with the standoff end (CDS 2.3.2.3, 2.2.14.2). RBF and "
        "access port on -Y in the -Z end zone of the panel (no cells); Nova access window to be confirmed (LVR-068).",
        f"Stowed mass {mprops['stowed_cbe']['mass_g']:.1f} g CBE / {mprops['stowed_mev']['mass_g']:.1f} g MEV; CG "
        f"({', '.join(f'{c:+.1f}' for c in mprops['stowed_cbe']['cg_mm'])}) mm CBE. Hidden lines grey dashed. "
        "Generated by hardware/mechanical/model.py.",
    ]
    for i, n in enumerate(notes):
        svg.text(20, 560 + i * 16, n, 8 if i else 10, "start", "h" if i == 0 else "t")
    svg.save(os.path.join(outdir, "drawing_stowed_3view.svg"))

    # ---- sheet 2: deployed
    depl = cq.Compound.makeCompound([(pt.deployed if pt.deployed is not None else pt.stowed) for pt in ext])
    body_wings = cq.Compound.makeCompound([(pt.deployed if pt.deployed is not None else pt.stowed) for pt in ext
                                          if not pt.name.endswith("_element")])
    svg = SVG(1100, 780, "SMART-QSO deployed configuration")
    svg.el.append('<rect class="frame" x="5" y="5" width="1090" height="770"/>')
    s2 = 0.95
    ox, oy = 330, 200                      # eye on -Z: u = +x, v = -y
    draw_view(svg, depl, (0, 0, -1), (1, 0, 0), ox, oy, s2, hidden=False)
    svg.text(550, 40, "VIEW D  from -Z (AntS face): wings and antenna elements deployed", 10, "middle", "h")
    aw = P["ants_w"] / 2
    svg.dim(ox + aw * s2, oy, ox + (aw + P["vhf_len"]) * s2, oy, -28, "2 m element 500 (along +X; length per ISIS "
            "tuning)", size=8)
    svg.dim(ox - (aw + P["uhf_len"]) * s2, oy, ox - aw * s2, oy, -28, "70 cm element 170 (-X)", size=8)
    wing_x1 = P["hinge_x"] + (P["hinge_y"] - P["wing_y0"])
    svg.dim(ox, oy + 38 * s2, ox + wing_x1 * s2, oy + 38 * s2, 30, f"W1 tip x = +{wing_x1:.1f}", size=8)
    svg.dim(ox - wing_x1 * s2, oy + 38 * s2, ox, oy + 38 * s2, 30, f"W2 tip x = -{wing_x1:.1f}", size=8)
    svg.line(ox, oy, ox, oy + 70, "call")
    svg.text(ox + 3, oy + 78, "+Y (sun, spin axis) is down in this view", 7, "start", "c")
    svg.text(ox + 120 * s2, oy + 30 * s2, "wings in the plane y = 37.1 to 39.4, front cells toward +Y", 7, "start", "c")
    s3 = 2.0
    ox2, oy2 = 550, 520                    # eye on +Y: u = -x, v = +z
    draw_view(svg, body_wings, (0, 1, 0), (-1, 0, 0), ox2, oy2, s3, hidden=False)
    svg.text(550, 380, "VIEW E  from +Y (sun side): front faces of the deployed wings (antenna elements omitted)", 10,
             "middle", "h")
    svg.text(ox2 - 93 * s3, oy2 - 54 * s3, "W1 (+X)", 9, "middle", "h")
    svg.text(ox2 + 93 * s3, oy2 - 54 * s3, "W2 (-X)", 9, "middle", "h")
    svg.dim(ox2 - wing_x1 * s3, oy2 + re * s3, ox2 + wing_x1 * s3, oy2 + re * s3, 18,
            f"{2 * wing_x1:.1f} tip to tip", size=8)
    svg.dim(ox2 + wing_x1 * s3, oy2 - P["wing_z1"] * s3, ox2 + wing_x1 * s3, oy2 - P["wing_z0"] * s3, 12,
            f"{P['wing_z1'] - P['wing_z0']:.0f}", vertical=True, size=8)
    md, mm = mprops["deployed_cbe"], mprops["deployed_mev"]
    notes = [
        "SMART-QSO 1U  |  SQSO-MECH-CAD-001 Sheet 2: deployed configuration  |  dimensions mm",
        f"Wings rotate 90 deg about the hinge axes (x, y) = (+-{P['hinge_x']}, {P['hinge_y']}) mm, parallel to Z, into the "
        "plane y = 37.1 to 39.4 mm; front cells face +Y.",
        f"AntS option A (BASELINE 7): one 2 m tape element along +X, one 70 cm element along -X, in the plane "
        f"z = {zp['mz_out'] - P['ants_t'] / 2:.1f} mm (centre of the housing); root and sweep per the ISIS ICD (open).",
        f"Deployed inertia about the CG, CBE: Ixx {md['Ixx'] * 1e3:.3f}, Iyy {md['Iyy'] * 1e3:.3f}, Izz {md['Izz'] * 1e3:.3f} "
        f"x 1e-3 kg m2; Iyy/max(Ixx, Izz) {md['ratio_Iyy']:.3f} (MEV {mm['ratio_Iyy']:.3f}); major axis "
        f"{md['major_from_Y_deg']:.1f} deg (MEV {mm['major_from_Y_deg']:.1f} deg) from +Y. Requirement: >= 1.03 and <= 5 deg.",
    ]
    for i, n in enumerate(notes):
        svg.text(20, 690 + i * 16, n, 8 if i else 10, "start", "h" if i == 0 else "t")
    svg.save(os.path.join(outdir, "drawing_deployed.svg"))


def checks_value(checks, cid):
    for it in checks["items"]:
        if it["id"] == cid:
            return it["value"]
    return float("nan")


# ======================================================================================================
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--no-export", action="store_true")
    ap.add_argument("--no-interference", action="store_true")
    ap.add_argument("--out", default=HERE)
    args = ap.parse_args()
    parts = build_parts()
    out: Dict = {"parameters": P, "parts": []}
    cache: Dict = {}
    mprops = {}
    for grown in (False, True):
        masses = assign_masses(parts, grown)
        for dep in (False, True):
            key = f"{'deployed' if dep else 'stowed'}_{'mev' if grown else 'cbe'}"
            mprops[key] = mass_props(parts, masses, dep, cache).summary()
    masses = assign_masses(parts, False)
    for pt in parts:
        v, c, _ = cache[(pt.name, False)] if (pt.name, False) in cache else solid_props(pt.stowed)
        out["parts"].append(dict(name=pt.name, group=pt.group, mel=pt.mel, mass_g=masses[pt.name], volume_mm3=v,
                                 centre_mm=c.tolist(), ghost=pt.ghost, moves=pt.moves, note=pt.note))
    # one wing only (W1 deployed, W2 stowed) and wings stowed with antennas out
    for lab, mv in (("W1_only_deployed", ("W1", "ANT")), ("antennas_only", ("ANT",))):
        pp = []
        for pt in parts:
            q = copy.copy(pt)
            if pt.moves and pt.moves not in mv:
                q.deployed = None
            pp.append(q)
        mprops[lab] = mass_props(pp, masses, True, {}).summary()
    # MEL hybrid antenna case of MASS_BUDGET v3.0 (2 m +-X, 70 cm +-Y), for comparison with section 6.1
    out["mass_properties"] = mprops
    for k, v in mprops.items():
        print(f"{k:22s} m {v['mass_g']:7.1f} g CG ({v['cg_mm'][0]:+5.2f}, {v['cg_mm'][1]:+5.2f}, {v['cg_mm'][2]:+5.2f}) "
              f"I ({v['Ixx'] * 1e3:.3f}, {v['Iyy'] * 1e3:.3f}, {v['Izz'] * 1e3:.3f}) Ixy {v['Ixy'] * 1e6:+.1f} "
              f"Ixz {v['Ixz'] * 1e6:+.1f} Iyz {v['Iyz'] * 1e6:+.1f} e-6  ratio {v['ratio_Iyy']:.3f}  "
              f"major {v['major_from_Y_deg']:.1f} deg")
    checks = run_checks(parts, interference=not args.no_interference)
    for key in ("stowed_cbe", "stowed_mev"):
        c = mprops[key]["cg_mm"]
        checks["items"].append(dict(id=f"MP-CG-{key}", requirement="Stowed CG within +-20 mm of the geometric centre "
                                    "(CDS 2.2.11 Table 2); baseline: within 5 mm", value=c, limit="+-20 (5)",
                                    ok=max(abs(x) for x in c) <= 20.0, note="within 5 mm" if max(abs(x) for x in c)
                                    <= 5.0 else "outside the 5 mm baseline statement"))
    for key in ("deployed_cbe", "deployed_mev"):
        v = mprops[key]
        checks["items"].append(dict(id=f"MP-I-{key}", requirement="Iyy >= 1.03 max(Ixx, Izz) and major principal axis "
                                    "within 5 deg of +Y (BASELINE 4)", value=dict(ratio=v["ratio_Iyy"],
                                    major_from_Y_deg=v["major_from_Y_deg"], principal_ratio=v["ratio_major"]),
                                    limit=">= 1.03; <= 5 deg", ok=v["ratio_Iyy"] >= 1.03 and v["major_from_Y_deg"] <= 5.0))
    out["checks"] = checks
    out["protrusion_stack"] = protrusion_stack()
    for it in checks["items"]:
        tag = "OPEN" if it["ok"] is None else ("PASS" if it["ok"] else "FAIL")
        print(f"[{tag}] {it['id']}: {it['requirement']} -> {it['value']}")
    if not args.no_export:
        sizes = {}
        for dep in (False, True):
            fn = os.path.join(args.out, f"smartqso_{'deployed' if dep else 'stowed'}.step")
            sizes[os.path.basename(fn)] = export_step(parts, fn, dep)
            fn = os.path.join(args.out, f"smartqso_oml_{'deployed' if dep else 'stowed'}.step")
            sizes[os.path.basename(fn)] = export_step(parts, fn, dep, oml=True)
        out["step_bytes"] = sizes
        print("STEP", sizes)
        drawings(parts, args.out, checks, mprops)
    with open(os.path.join(args.out, "results.json"), "w") as f:
        json.dump(out, f, indent=1, default=lambda x: x.tolist() if isinstance(x, np.ndarray) else str(x))


if __name__ == "__main__":
    main()
