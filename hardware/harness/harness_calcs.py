#!/usr/bin/env python3
"""
SMART-QSO harness calculations (SQSO-HAR-001 2.0, hardware/harness/HARNESS_DESIGN.md).

Reads the wire list (wire_list.csv, one row per conductor) and the routes (routes.csv, waypoints in the stack frame of
hardware/mechanical/MECHANICAL_DESIGN.md, each segment tagged with the harness zone it runs in) and computes:

  * the length of every wire (route length x 1.10 + 15 mm per end, rounded up to 5 mm, or the stated minimum),
  * the fill of every zone and, for the vertical wall-gap bundles, of every height band A-E, against 60 % of the
    zone's section,
  * the ampacity of every wire: NASA EEE-INST-002 section W1 single-wire currents for 200 C-rated wire with the
    bundle factor (29 - N)/28 for N <= 15 and 0.5 above (N = conductors in the busiest section the wire passes),
  * the contact current of every wire end against 50 % of the contact rating (EEE-INST-002 connector derating),
  * the voltage drop of every power circuit at its design current, conductor at +60 C, with contact resistance,
  * the harness mass (wire, coax, harness-side connector halves, splices, the access-port board A3, tabs, lacing),
  * the band-B demand the interconnect (SQSO-ICX-001 B 2.1) estimated, recomputed with these gauges, to show why the
    EPS-only harnesses are routed out of the four bundles,
  * the field at the magnetometer of the PA supply pair's untwisted ends.

    python3 hardware/harness/harness_calcs.py              # tables, writes harness_results.json
    python3 hardware/harness/harness_calcs.py --cad        # also checks every zone against the CAD solids
                                                           # (needs CadQuery; imports hardware/mechanical/model.py
                                                           # read-only)
    python3 hardware/harness/harness_calcs.py --wireviz    # also checks the gauges and lengths in
                                                           # hardware/diagrams/wiring/W*.yml against the wire list
Exit status 1 if any check fails.  Python 3.8+, standard library (PyYAML only for --wireviz).
"""
from __future__ import annotations

import argparse
import csv
import json
import math
import os
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))

# ------------------------------------------------------------------------------------------------ wire data
# SAE AS22759/33 (crosslinked ETFE, silver-coated high-strength copper alloy, 200 C, 600 V), 19-strand.
# Finished diameter nominal 0.024/0.027/0.032/0.037/0.043/0.050 in +-0.002 in (NASA NEPP AS22759/33 page); the
# maximum is used for fill.  Weight: specification maximum, lb/1000 ft x 1.48816 = g/m (Glenair wire tables).
# Single-wire current: EEE-INST-002 W1 (200 C rated wire, 70 C, vacuum).  Samtec SFSDT PTFE wire is taken as 30 AWG
# with the /33-30 diameter and weight.
WIRE = {
    # awg: (OD max mm, g/m, strand dia mm (19 strands), single-wire A)
    30: (0.66, 0.98, 0.0633, 1.3),
    28: (0.74, 1.35, 0.0799, 1.8),
    26: (0.86, 2.08, 0.1007, 2.5),
    24: (0.99, 2.98, 0.1270, 3.3),
    22: (1.14, 4.32, 0.1601, 5.0),
    20: (1.32, 6.55, 0.2019, 7.5),
}
COAX = {"COAX137": (1.37, 4.0), "COAX113": (1.13, 2.7)}      # OD mm, g/m (FEP micro-coax, allocation)
RHO_CU = 0.017241                                             # ohm mm2/m, annealed copper at 20 C (100 % IACS)
IACS = 0.85                                                   # high-strength copper alloy, lower bound (ASTM B624)
LAY = 1.03                                                    # stranding lay factor
T_HOT = 60.0                                                  # harness temperature for drops (C)
ALPHA = 0.00393
FILL = 0.60                                                   # allowance: 60 % of the section filled with wire
DERATE_CONTACT = 0.50                                         # EEE-INST-002: contact current <= 50 % of rating
LEN_FACTOR, LEN_PER_END, LEN_STEP = 1.10, 15.0, 5.0           # length rule (mm)

# connector families: (contact rating A, contact resistance ohm, harness-side mass: housing g, per-contact g)
FAMILY = {
    "PC": (1.0, 0.020, 0.03, 0.035),     # Molex Pico-Clasp 501330 housing, 501334 gold terminal (28-32 AWG)
    "MF": (3.0, 0.010, 0.20, 0.10),      # Molex Micro-Fit 3.0 43025 receptacle, 43030 gold terminal (20-24 AWG);
                                         # 3 A taken for 22-24 AWG wire (5 A is the 20 AWG rating)
    "TFM": (2.0, 0.020, 0.30, 0.04),     # Samtec SFSDT socket on a TFM header (1.27 mm, latched)
    "MMCX": (0.0, 0.0, 0.90, 0.0),       # Molex 73414 right-angle crimp plug
    "SOL": (0.0, 0.0, 0.0, 0.0),         # solder pad, terminal or turret (no contact)
    "SPL": (0.0, 0.002, 0.0, 0.15),      # SAE AS81824/1 crimp splice under heat shrink, staggered
    "RX": (1.0, 0.020, 0.05, 0.035),     # GNSS receiver mating connector (28 AWG contacts), allocation
}


def family(ref: str) -> str:
    board, _, j = ref.partition("-")
    if board == "EPS":
        n = int(j[1:])
        return {1: "MF", 3: "TFM", 10: "MF", 11: "MF", 13: "MF", 18: "MF"}.get(n, "PC")
    if board == "OBC":
        return "TFM" if j in ("J1", "J2", "J9") else "PC"
    if board == "RF":
        return "TFM" if j == "J1" else ("PC" if j == "J2" else "MMCX")
    if board == "JET":
        return "MF" if j == "J1" else "PC"
    if ref == "A2-J1":
        return "PC"
    if ref in ("GNSS-RF", "PATCH") or ref.startswith("ANTS-"):
        return "MMCX"
    if ref == "GNSS-J1":
        return "RX"
    if ref.startswith(("ROD-", "HS-")) or ref == "ANTS":
        return "SPL"
    return "SOL"                          # BATT, PNL-*, TB-*, CSS-*, BB-*, A3, DS*, RBF


# ------------------------------------------------------------------------------------------------ zones
# name: (x0, x1, y0, y1, z0, z1, run axis, description).  Stack frame of MECHANICAL_DESIGN 4.1 (mm).
ZONES = {
    "PXPY": (45.4, 48.4, 24.0, 33.9, -46.9, 48.4, "z", "(+X,+Y) wall-gap bundle"),
    "MXPY": (-48.4, -45.4, 24.0, 33.9, -46.9, 48.4, "z", "(-X,+Y) wall-gap bundle"),
    "MXMY": (-48.4, -45.4, -33.9, -24.0, -46.9, 48.4, "z", "(-X,-Y) wall-gap bundle"),
    "PXMY_LO": (45.4, 48.4, -33.9, -24.0, -46.9, -26.0, "z", "(+X,-Y) wall-gap bundle below z -26"),
    "PXMY_UP": (45.4, 48.4, -27.0, -20.5, -26.0, 48.4, "z", "(+X,-Y) wall-gap bundle above z -26 (3 x 6.5 mm)"),
    "MXC": (-48.4, -45.4, 7.0, 19.5, -16.0, 18.0, "z", "-X wall gap between the Z rod and the 70 cm coax band"),
    "PXC_B": (45.4, 48.4, -20.0, 20.0, -26.0, -9.0, "y", "+X wall gap at |y| < 20 below the EPS"),
    "PXMY2": (45.4, 48.4, -45.5, -39.5, -12.0, 44.5, "z", "+X wall gap beside W4A, DS3 leads"),
    "HS_PX": (45.4, 48.4, 34.5, 39.5, -42.0, -5.0, "z", "+X hinge-side strip"),
    "HS_MX": (-48.4, -45.4, 34.5, 39.5, -42.0, -5.0, "z", "-X hinge-side strip"),
    "LI_DS1": (45.4, 47.4, 39.5, 43.5, -44.5, -42.0, "y", "DS1 lead-in beside the hinge doubler"),
    "CS_PXMY": (43.2, 46.4, -46.8, -34.0, -44.5, -34.0, "y", "(+X,-Y) corner strip to the W1 burn board"),
    "CS_MXMY": (-46.4, -43.2, -46.8, -34.0, -44.5, -34.0, "y", "(-X,-Y) corner strip: W2 burn board, DS2, W8A"),
    "S_MXMY": (-45.0, -36.0, -34.0, -24.5, -34.0, -17.0, "z", "inboard strip beside the -X battery end cap"),
    "S_MXC": (-45.0, -36.0, -19.5, -7.5, -45.0, -17.0, "z", "inboard strip beside the -X end cap, centre"),
    "UB_MXC": (-45.0, -33.0, -19.5, -12.5, -46.6, -45.1, "x", "under the battery end cap, over the -Z end frame"),
    "EF_MXC": (-35.0, -28.0, -19.5, -12.5, -48.5, -45.1, "z", "through the -Z end-frame cut-out to the AntS"),
    "U_MXPY": (-44.0, -24.5, 7.0, 40.0, -15.0, -4.05, "y", "under the EPS, connector zone Z1"),
    "U_MXMY": (-44.0, -21.0, -40.0, -7.0, -15.0, -4.05, "y", "under the EPS, connector zone Z2"),
    "U_MXMY2": (-36.5, -21.0, -46.0, -40.0, -15.0, -4.05, "y", "under the EPS, zone Z2 (-Y edge)"),
    "U_PXPY": (24.5, 44.0, 6.0, 40.0, -15.0, -4.05, "y", "under the EPS, connector zone Z3"),
    "U_PXMY": (31.0, 44.0, -40.0, -16.0, -15.0, -4.05, "y", "under the EPS, connector zone Z4"),
    "U_PXMY2": (31.0, 36.5, -46.0, -40.0, -15.0, -4.05, "y", "under the EPS, zone Z4 (-Y edge)"),
    "U_PY": (-20.0, 24.0, 6.0, 17.5, -15.0, -4.05, "x", "under the EPS between the GNSS receiver and the X rod, zone Z5"),
    "U_PYN": (-20.0, -3.5, 17.5, 25.0, -15.0, -4.05, "y", "under the EPS, west of the (0, +21) centre support"),
    "U_J3U": (33.0, 44.0, -16.0, 6.0, -15.0, -12.5, "y", "under EPS J3, above the deck"),
    "PX_J45": (42.0, 48.4, 14.0, 24.0, 9.6, 16.6, "y", "OBC top side, entry to J4/J5 from the +X wall gap"),
    "U_X": (-24.5, 31.0, 25.0, 29.5, -14.0, -6.0, "x", "crossing between the centre support and the X rod"),
    "U_C": (-35.5, -21.0, -7.0, 7.0, -15.0, -4.05, "y", "crossing between the Z rod and the GNSS receiver"),
    "PG_PX": (44.0, 48.4, -32.0, -28.0, -12.0, -8.0, "x", "+X body-panel pigtail slot"),
    "PG_MX": (-48.4, -44.0, -14.0, -10.0, -12.0, -8.0, "x", "-X body-panel pigtail slot"),
    "PG_PY": (-32.0, -28.0, 40.0, 48.4, -12.0, -8.0, "y", "+Y body-panel pigtail slot"),
    "PG_MY": (-32.0, -28.0, -48.4, -46.0, -12.0, -8.0, "y", "-Y body-panel pigtail slot"),
    "DX_MXPY": (-34.0, -26.0, 14.0, 26.0, -16.6, -15.4, "z", "deck opening, battery leads"),
    "DX_MXMY": (-41.5, -36.5, -34.0, -24.5, -16.6, -15.4, "z", "deck opening (-X,-Y)"),
    "DX_MXC": (-41.5, -36.5, -19.5, -7.5, -16.6, -15.4, "z", "deck opening -X centre"),
    "DS_MY_M": (-34.0, -23.0, -44.6, -42.3, -16.6, -15.4, "z", "deck slot -Y, -X half"),
    "DS_MY_P": (31.0, 34.5, -44.6, -42.3, -16.6, -15.4, "z", "deck slot -Y, +X half"),
    "YW_MY": (-34.0, 36.0, -48.0, -41.0, -38.5, -17.0, "z", "-Y wall strip beside the battery"),
    "YB_MY": (-44.8, -33.0, -48.4, -46.0, -44.5, -40.0, "x", "-Y shear panel, outboard of the (-40,-43) column"),
    "AP_A3": (-35.0, -5.0, -48.4, -41.5, -46.8, -38.0, "x", "access-port board A3 and Micro-D 21 envelope (CAD request)"),
    "MZT": (-47.4, -43.5, 33.9, 38.0, -45.0, -42.0, "x", "-Z sun-sensor termination"),
    "TE_MXMY1": (-45.4, -24.0, -31.0, -27.0, 43.6, 48.2, "x", "+Z step-over to RF J2 (along -X)"),
    "TE_MXMY2": (-28.0, -24.0, -44.0, -27.0, 43.6, 48.2, "y", "+Z step-over to RF J2 (along -Y)"),
    "TE_PXPY": (36.2, 45.4, 25.0, 30.0, 43.6, 46.6, "x", "+Z step-over to magnetometer board A2"),
    "TE_PXMY": (6.0, 45.4, -24.0, -20.0, 43.6, 47.0, "x", "+Z step-over to the GNSS patch (along X)"),
    "TE_PXMY2": (2.0, 6.0, -24.0, -5.0, 43.6, 47.0, "y", "+Z step-over to the GNSS patch (along Y)"),
    "TE_MXPY": (-45.4, -34.0, 26.0, 38.0, 43.6, 48.2, "x", "+Z step-over to the +Z sun sensor"),
    "CAR_MXMY": (-45.4, -26.0, -40.0, -33.2, 21.7, 26.8, "x", "carrier -Y strip to carrier J1"),
    "CARP": (42.0, 45.4, 26.0, 31.0, 21.7, 27.6, "x", "carrier +X edge to carrier J2"),
    "W3Z": (45.4, 48.4, -17.0, 17.0, -8.0, 12.0, "z", "W3, flat in the +X wall gap at |y| < 17"),
    "W4AZ": (46.9, 48.4, -38.5, -28.5, 13.6, 46.9, "z", "W4A, flat on the +X shear panel"),
    "COAX_P": (-48.4, -45.4, 20.0, 24.0, -46.9, 42.4, "z", "W10 70 cm coax band (keep free of harness)"),
    "COAX_M": (-48.4, -45.4, -24.0, -20.0, -46.9, 42.4, "z", "W10 2 m coax band (keep free of harness)"),
    "COAXU_P": (-45.4, -33.6, 20.0, 24.0, -46.9, -45.5, "x", "W10 70 cm pair under the battery end cap"),
    "COAXU_M": (-45.4, -33.6, -24.0, -20.0, -46.9, -45.5, "x", "W10 2 m pair under the battery end cap"),
    "COAXF_P": (-35.0, -33.6, 20.0, 24.0, -48.5, -46.9, "z", "W10 70 cm pair through the -Z end frame"),
    "COAXF_M": (-35.0, -33.6, -24.0, -20.0, -48.5, -46.9, "z", "W10 2 m pair through the -Z end frame"),
}
BANDS = {"A": (-46.9, -26.0), "B": (-26.0, -1.0), "C": (-1.0, 6.6), "D": (6.6, 18.0), "E": (18.0, 48.4)}
CORNERS = {"(+X,+Y)": ["PXPY"], "(-X,+Y)": ["MXPY"], "(-X,-Y)": ["MXMY"], "(+X,-Y)": ["PXMY_LO", "PXMY_UP"]}

# design currents and drop limits of the power circuits: (A, limit V, source)
CIRCUITS = {
    "PACK": (2.00, 0.100, "all four burn drivers on, burn buck limit (SQSO-EPS-BRD-001 6.1); 1.71 A SW11 case"),
    "HEATER": (0.079, 0.100, "heater 207 ohm at 16.4 V (SQSO-EPS-BRD-001 10)"),
    "PA5V": (0.469, 0.100, "PA at 1 W (SQSO-RF-BRD-001 4.2); 2 % of 5 V"),
    "RADIO3V3": (0.089, 0.066, "receiver 38 mA + exciter 51 mA keyed (SQSO-RF-BRD-001 4.2); 2 % of 3.3 V"),
    "VJET": (1.40, 0.100, "Jetson carrier input, design current 1.4 A (EPS U19 trip 1.10-1.30 A)"),
    "ADCS3V3": (0.049, 0.066, "sensor rail worst normal (SQSO-ICX-001 3.4)"),
    "MTQ_X": (0.087, 0.100, "rod at 0.2 A.m2 (SQSO-OBC-BRD-001 10.1)"),
    "MTQ_Y": (0.087, 0.100, "as X"),
    "MTQ_Z": (0.087, 0.100, "as X"),
    "GNSS3V3_A": (0.10, 0.033, "receiver peak; W9A and W9B in series share 66 mV"),
    "GNSS3V3_B": (0.10, 0.033, "as W9A"),
    "SNS3V3": (0.001, 0.066, "RM3100"),
    "BURN_W1A": (0.88, 0.088, "0.8 A + 10 %; loop <= 0.10 ohm, 1.4 % of the driver's 6.9 ohm compliance"),
    "BURN_W1B": (0.88, 0.088, "as W1A"),
    "BURN_W2A": (0.88, 0.088, "as W1A"),
    "BURN_W2B": (0.88, 0.088, "as W1A"),
    "ANTS3V3": (0.61, 0.066, "AntS deployment peak (SQSO-EPS-BRD-001 8.3); 2 % of 3.3 V"),
    "STR_PX": (0.55, 0.048, "string Isc; 1 % of the 4.8 V Vmp"),
    "STR_MX": (0.55, 0.048, "as PX"),
    "STR_PY": (0.55, 0.048, "as PX"),
    "STR_MY": (0.55, 0.048, "as PX"),
    "STR_W1": (1.10, 0.048, "front and back strings, three shared returns"),
    "STR_W2": (1.10, 0.048, "as W1"),
    "UMBCHG": (0.50, 0.100, "ground charge current (F5 1 A)"),
    "OBC3V3": (0.040, 0.033, "OBC peak (SQSO-EPS-BRD-001 8.3); 1 % of 3.3 V"),
    "VMTQ": (0.104, 0.100, "three rods at 1.16 W, 11.2 V (SQSO-EPS-BRD-001 8.3)"),
}
# the EPS-only harnesses the interconnect estimated for band B (SQSO-ICX-001 B 2.1: W1, W2, W7, W11)
EPS_ONLY = ("W1", "W2PX", "W2MX", "W2PY", "W2MY", "W2W1", "W2W2", "W7A", "W7B", "W7C", "W11")
# MEL rows the harness mass falls in (MASS_BUDGET 3.1): 31 GNSS cable, 26 ADCS sensor cables, 32 everything else
MEL_ROW = {"W10E": "31", "W6B": "26", "W6C": "26", "W6D": "26", "W6E": "26"}
MEL_CBE = {"32": 80.0, "26": 5.0, "31": 26.0}
# EPS bottom-side connector zones (SQSO-HAR-001 10.1): zone -> (harness zones whose plan area it uses, area excluded)
EPS_ZONES = {"Z1": (("U_MXPY",), 0.0), "Z2": (("U_MXMY", "U_MXMY2"), 0.0), "Z3": (("U_PXPY",), 11.0 * 9.5),
             "Z4": (("U_PXMY", "U_PXMY2"), 0.0), "Z5": (("U_PY", "U_PYN"), 0.0)}
EPS_PLACE = {"J1": ("Z1", 8), "J2": ("Z1", 6), "J6": ("Z1", 5), "J20": ("Z1", 2),
             "J5": ("Z2", 5), "J7": ("Z2", 5), "J11": ("Z2", 6), "J16": ("Z2", 3), "J18": ("Z2", 4), "J19": ("Z2", 6),
             "J8": ("Z3", 8), "J10": ("Z3", 6), "J15": ("Z3", 3), "J21": ("Z3", 2),
             "J4": ("Z4", 5), "J14": ("Z4", 4), "J17": ("Z4", 3),
             "J9": ("Z5", 8), "J12": ("Z5", 6), "J13": ("Z5", 6)}


def footprint(fam, pins):
    """Mated plan footprint (mm2) with plug and wire exit: Micro-Fit 2xN right-angle, Pico-Clasp 1xN vertical."""
    return (3.0 * pins / 2 + 3.0) * 23.0 if fam == "MF" else (1.0 * pins + 3.0) * 4.5


def ceil_to(v, step):
    return step * math.ceil(v / step - 1e-9)


def area(awg_or_coax):
    od = COAX[awg_or_coax][0] if awg_or_coax in COAX else WIRE[int(awg_or_coax)][0]
    return math.pi / 4.0 * od * od


def r_per_m(awg, t=T_HOT):
    d = WIRE[int(awg)][2]
    a = 19 * math.pi / 4.0 * d * d
    return RHO_CU / IACS * LAY / a * (1.0 + ALPHA * (t - 20.0))


def load():
    with open(os.path.join(HERE, "wire_list.csv")) as f:
        wires = list(csv.DictReader(f))
    routes = defaultdict(list)
    with open(os.path.join(HERE, "routes.csv")) as f:
        for r in csv.DictReader(f):
            routes[r["route"]].append((r["zone"], float(r["x_mm"]), float(r["y_mm"]), float(r["z_mm"])))
    return wires, routes


def segments(pts):
    out = []
    for (_, x0, y0, z0), (zn, x1, y1, z1) in zip(pts, pts[1:]):
        out.append((zn, (x0, y0, z0), (x1, y1, z1), abs(x1 - x0) + abs(y1 - y0) + abs(z1 - z0)))
    return out


def seg_keys(zn, p0, p1):
    """(zone, band) keys a segment occupies; vertical bundles are split into bands."""
    if zn not in ZONES or ZONES[zn][6] != "z" or zn.startswith(("COAX",)):
        return [(zn, "")]
    lo, hi = sorted((p0[2], p1[2]))
    keys = []
    for b, (b0, b1) in BANDS.items():
        if (hi - lo < 0.2 and b0 <= lo <= b1) or (min(hi, b1) - max(lo, b0) > 0.2):
            keys.append((zn, b))
    return keys or [(zn, "")]


def section(zn):
    x0, x1, y0, y1, z0, z1, ax, _ = ZONES[zn]
    dx, dy, dz = x1 - x0, y1 - y0, z1 - z0
    return {"z": dx * dy, "x": dy * dz, "y": dx * dz}[ax]


def compute():
    wires, routes = load()
    problems = []
    # route geometry: every waypoint within 1.5 mm of its zone box
    rlen = {}
    for r, pts in routes.items():
        rlen[r] = sum(s[3] for s in segments(pts))
        for i, (zn, x, y, z) in enumerate(pts[1:], 1):
            tol = 4.5 if i == len(pts) - 1 else 1.5
            if zn not in ZONES:
                problems.append(f"route {r}: unknown zone {zn}")
                continue
            x0, x1, y0, y1, z0, z1 = ZONES[zn][:6]
            if not (x0 - tol <= x <= x1 + tol and y0 - tol <= y <= y1 + tol and z0 - tol <= z <= z1 + tol):
                problems.append(f"route {r}: waypoint ({x}, {y}, {z}) outside zone {zn}")
    # lengths
    for w in wires:
        if w["route"] not in routes:
            problems.append(f"{w['wire']}: unknown route {w['route']}")
            continue
        L = ceil_to(LEN_FACTOR * rlen[w["route"]] + 2 * LEN_PER_END, LEN_STEP)
        w["length_mm"] = max(L, float(w["min_length_mm"] or 0))
        w["route_mm"] = rlen[w["route"]]
    # occupancy
    occ = defaultdict(list)
    for w in wires:
        keys = set()
        for zn, p0, p1, _ in segments(routes[w["route"]]):
            keys.update(seg_keys(zn, p0, p1))
        w["keys"] = sorted(keys)
        for k in keys:
            occ[k].append(w)
    fills = {}
    for k, ws in occ.items():
        a = sum(area(w["awg"]) for w in ws)
        allow = FILL * section(k[0])
        fills[k] = dict(zone=k[0], band=k[1], wires=len(ws), area_mm2=a, allow_mm2=allow, ratio=a / allow,
                        harnesses=sorted({w["harness"] for w in ws}))
        if a > allow:
            problems.append(f"fill {k[0]} {k[1]}: {a:.1f} > {allow:.1f} mm2")
    # coax bands must hold coax only
    for k, f in fills.items():
        if k[0].startswith("COAX") and any(h != "W10" for h in f["harnesses"]):
            problems.append(f"coax band {k[0]} carries {f['harnesses']}")
    # ampacity and contacts
    for w in wires:
        if w["awg"] in COAX:
            w["N"] = w["I_allow"] = w["amp_ratio"] = None
            continue
        n = max((fills[k]["wires"] for k in w["keys"]), default=1)
        fac = 1.0 if n <= 1 else ((29 - n) / 28.0 if n <= 15 else 0.5)
        w["N"] = n
        w["I_allow"] = WIRE[int(w["awg"])][3] * fac
        w["amp_ratio"] = float(w["i_max_A"]) / w["I_allow"]
        if w["amp_ratio"] > 1.0:
            problems.append(f"{w['wire']}: {w['i_max_A']} A > {w['I_allow']:.2f} A derated")
        worst = 0.0
        for ref in (w["from_ref"], w["to_ref"]):
            rating = FAMILY[family(ref)][0]
            if rating > 0:
                worst = max(worst, float(w["i_nom_A"]) / (DERATE_CONTACT * rating))
        w["contact_ratio"] = worst
        if worst > 1.0:
            problems.append(f"{w['wire']}: contact current {w['i_nom_A']} A above 50 % of the rating")
        lo, hi = {"PC": (28, 32), "MF": (20, 24), "TFM": (28, 30), "RX": (28, 28)}.get(family(w["from_ref"]), (0, 99))
        lo2, hi2 = {"PC": (28, 32), "MF": (20, 24), "TFM": (28, 30), "RX": (28, 28)}.get(family(w["to_ref"]), (0, 99))
        g = int(w["awg"])
        if not (lo <= g <= hi and lo2 <= g <= hi2):
            problems.append(f"{w['wire']}: {g} AWG outside the terminal range of {w['from_ref']} / {w['to_ref']}")
    # voltage drops
    drops = {}
    by_c = defaultdict(lambda: {"+": [], "-": []})
    for w in wires:
        if w["circuit"]:
            by_c[w["circuit"]][w["pole"]].append(w)
    for c, poles in by_c.items():
        i_d, lim, src = CIRCUITS[c]
        rp = {}
        for pole, ws in poles.items():
            if not ws:
                problems.append(f"circuit {c}: no {pole} conductor")
                rp[pole] = 0.0
                continue
            g = sum(1.0 / (r_per_m(w["awg"]) * w["length_mm"] / 1000.0 +
                           FAMILY[family(w["from_ref"])][1] + FAMILY[family(w["to_ref"])][1]) for w in ws)
            rp[pole] = 1.0 / g
        drop = i_d * (rp["+"] + rp["-"])
        drops[c] = dict(I_A=i_d, R_loop_ohm=rp["+"] + rp["-"], drop_V=drop, limit_V=lim, source=src,
                        harness=sorted({w["harness"] for ws in poles.values() for w in ws}))
        if drop > lim:
            problems.append(f"drop {c}: {drop * 1000:.1f} mV > {lim * 1000:.0f} mV")
    # mass
    m_wire = defaultdict(float)
    m_row = defaultdict(float)
    ends = defaultdict(set)
    for w in wires:
        gpm = COAX[w["awg"]][1] if w["awg"] in COAX else WIRE[int(w["awg"])][1]
        m_wire[w["harness"]] += gpm * w["length_mm"] / 1000.0
        m_row["31" if w["route"] == "W10E" else MEL_ROW.get(w["harness"], "32")] += gpm * w["length_mm"] / 1000.0
        for ref, pin in ((w["from_ref"], w["from_pin"]), (w["to_ref"], w["to_pin"])):
            ends[(w["harness"], ref)].add(pin)
    m_conn = defaultdict(float)
    for (h, ref), pins in ends.items():
        fam = FAMILY[family(ref)]
        if family(ref) == "MMCX":
            m = fam[2] * len(pins) if ref.startswith(("RF-", "GNSS", "PATCH", "ANTS")) else 0.0
        else:
            m = fam[2] + fam[3] * len(pins) if (fam[2] or fam[3]) else 0.0
        m_conn[h] += m
        m_row["31" if ref in ("GNSS-RF", "PATCH") else MEL_ROW.get(h, "32")] += m
    tie_pts = 0
    for (zn, band) in fills:
        x0, x1, y0, y1, z0, z1, ax, _ = ZONES[zn]
        ext = {"x": x1 - x0, "y": y1 - y0, "z": z1 - z0}[ax]
        if band:
            ext = BANDS[band][1] - BANDS[band][0]
        tie_pts += int(math.ceil(ext / 30.0))
    extras = {
        "access-port board A3 (FR-4 30 x 12 x 1.6 mm, Micro-D 21 socket receptacle, two M2 Ti screws, staking)": 6.0,
        "termination tabs TB-W1, TB-W2 (polyimide, 6 x 10 mm, bonded)": 0.4,
        "PTFE over-sleeve on the four pack leads": 0.3,
        "heat shrink and identification sleeves (one per wire end)": round(0.01 * 2 * len(wires), 1),
        f"lacing tape spot ties and Scotch-Weld 2216 tie-down dots ({tie_pts} points x 0.10 g)": round(0.10 * tie_pts, 1),
    }
    per_h = {}
    for h in sorted(set(w["harness"] for w in wires)):
        per_h[h] = dict(wire_g=m_wire[h], conn_g=m_conn[h])
    total = sum(m_wire.values()) + sum(m_conn.values()) + sum(extras.values())
    m_row["32"] += sum(extras.values())
    # band-B demand of the EPS-only harnesses had they used the four bundles (the interconnect's method)
    corner_b = {}                                        # band-B fill of the harnesses with an OBC end (and W10E)
    for c, zs in CORNERS.items():
        corner_b[c] = sum(area(w["awg"]) for z in zs for w in occ.get((z, "B"), []) if w["harness"] not in EPS_ONLY)
    allow_b = {"(+X,+Y)": FILL * section("PXPY"), "(-X,+Y)": FILL * section("MXPY"), "(-X,-Y)": FILL * section("MXMY"),
               "(+X,-Y)": FILL * section("PXMY_UP")}
    left_b = sum(allow_b[c] - corner_b[c] for c in CORNERS)
    eps_only_area = sum(area(w["awg"]) for w in wires if w["harness"] in EPS_ONLY)
    # PA pair untwisted ends at the magnetometer (dipole on axis, worst orientation)
    a2 = (24.0, 27.0, 44.0)
    mag = {}
    for name, p in (("RF J2 end", (-26.0, -44.0, 42.4)), ("EPS J19 end", (-40.0, -30.0, -8.0))):
        r = math.dist(a2, p) / 1000.0
        m = CIRCUITS["PA5V"][0] * 5e-6               # 5 mm untwisted x 1.0 mm pitch
        mag[name] = dict(r_mm=r * 1000, moment_Am2=m, B_nT=1e-7 * 2 * m / r ** 3 * 1e9)
    # EPS bottom-side connector zones: footprint budget, and every EPS connector of the wire list placed
    ezones = {}
    used = {w["from_ref"][4:] for w in wires if w["from_ref"].startswith("EPS-")} - {"J3"}
    for j in sorted(used - set(EPS_PLACE)):
        problems.append(f"EPS {j} has no bottom-side zone")
    for z, (zs, excl) in EPS_ZONES.items():
        a_zone = sum((ZONES[n][1] - ZONES[n][0]) * (ZONES[n][3] - ZONES[n][2]) for n in zs) - excl
        conns = [j for j, (zz, _) in EPS_PLACE.items() if zz == z]
        a_conn = sum(footprint(family("EPS-" + j), EPS_PLACE[j][1]) for j in conns)
        ezones[z] = dict(connectors=conns, footprint_mm2=a_conn, area_mm2=a_zone, ratio=a_conn / a_zone)
        if a_conn > 0.8 * a_zone:
            problems.append(f"EPS zone {z}: connector footprints {a_conn:.0f} > 80 % of {a_zone:.0f} mm2")
    return dict(wires=wires, routes=routes, rlen=rlen, fills=fills, drops=drops, per_h=per_h, extras=extras, total=total,
                ezones=ezones,
                corner_b=corner_b, allow_b=allow_b, left_b=left_b, eps_only_area=eps_only_area, mag=mag,
                tie_pts=tie_pts, m_row=dict(m_row), problems=problems)


def cad_check(res):
    """Boolean intersection of every zone box with every CAD solid (hardware/mechanical/model.py, read-only)."""
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(REPO, "hardware", "mechanical"))
    import model as M                                    # noqa: E402  (CadQuery environment)
    from OCP.BRepAlgoAPI import BRepAlgoAPI_Common      # noqa: E402
    from OCP.BRepGProp import BRepGProp                  # noqa: E402
    from OCP.GProp import GProp_GProps                   # noqa: E402
    allowed = {"AP_A3": {"access_port"}}                 # the zone that replaces the CAD part (request to the CAD)
    parts = [p for p in M.build_parts(M.P) if not p.ghost and p.group not in ("harness", "dist")]
    out = {}
    for zn, (x0, x1, y0, y1, z0, z1, *_rest) in ZONES.items():
        zb = M.B(x0, x1, y0, y1, z0, z1)
        hits, near = [], []
        for p in parts:
            bb = p.stowed.BoundingBox()
            if bb.xmax < x0 - 3 or bb.xmin > x1 + 3 or bb.ymax < y0 - 3 or bb.ymin > y1 + 3 or bb.zmax < z0 - 3 \
                    or bb.zmin > z1 + 3:
                continue
            c = BRepAlgoAPI_Common(zb.wrapped, p.stowed.wrapped)
            c.Build()
            g = GProp_GProps()
            BRepGProp.VolumeProperties_s(c.Shape(), g)
            if g.Mass() > 1e-3:
                if p.name not in allowed.get(zn, set()):
                    hits.append(p.name)
            else:
                near.append((round(M.min_dist(zb, p.stowed), 2), p.name))
        near.sort()
        out[zn] = dict(interferes=hits, nearest=near[:2])
        if hits:
            res["problems"].append(f"CAD: zone {zn} intersects {hits}")
    return out


def wireviz_check(res):
    """Cable designators in the WireViz sources: a harness or a route of the wire list, an optional -<AWG> suffix for a
    harness of several gauges and an optional _A/_B suffix for two cables on one route: W1-22, W6C_X, W10_M_A."""
    import yaml                                          # noqa: E402
    by_h, by_r = defaultdict(list), defaultdict(list)
    for w in res["wires"]:
        by_h[w["harness"]].append(w)
        by_r[w["route"]].append(w)
    out = []
    wdir = os.path.join(REPO, "hardware", "diagrams", "wiring")
    for fn in sorted(os.listdir(wdir)):
        if not fn.endswith(".yml"):
            continue
        doc = yaml.safe_load(open(os.path.join(wdir, fn)))
        for name, c in (doc.get("cables") or {}).items():
            base, _, g = name.partition("-")
            if base in by_r:
                ws = by_r[base]
            elif base in by_h:
                ws = by_h[base]
            elif base[-2:] in ("_A", "_B") and base[:-2] in by_r:
                ws = by_r[base[:-2]]
            else:
                continue
            h = ws[0]["harness"] if ws else base
            msg = []
            gs = {str(w["awg"]) for w in ws}
            want_g = g if g.isdigit() else (next(iter(gs)) if len(gs) == 1 else None)
            have_g = str(c.get("gauge", "")).split()[0] if c.get("gauge") else ""
            if want_g and not want_g.startswith("COAX") and have_g != want_g:
                msg.append(f"gauge {c.get('gauge')} != {want_g} AWG")
            if g.isdigit() and g not in gs:
                msg.append(f"{g} AWG not used by {h}")
            want = max(w["length_mm"] for w in ws) / 1000.0
            if "length" in c and abs(float(c["length"]) - want) > 0.0026:
                msg.append(f"length {c['length']} m != {want:.3f} m")
            out.append((fn, name, "; ".join(msg) or "ok"))
            if msg:
                res["problems"].append(f"WireViz {fn} {name}: {'; '.join(msg)}")
    return out


def report(res, cad=None, wv=None):
    p = print
    p("SMART-QSO harness calculations (SQSO-HAR-001 2.0)\n")
    p("Harness lengths (mm, route x 1.10 + 30, rounded up to 5; minimum where stated)")
    seen = {}
    for w in res["wires"]:
        seen.setdefault((w["harness"], w["route"]), (w["route_mm"], w["length_mm"], w["awg"]))
    for (h, r), (rl, L, g) in sorted(seen.items()):
        p(f"  {h:6s} {r:8s} route {rl:6.1f}  wire {L:5.0f}")
    p("\nCorner bundles by band: fill mm2 / allowance mm2 (60 %) / ratio")
    for c, zs in CORNERS.items():
        row = []
        for b in BANDS:
            f = next((res["fills"][(z, b)] for z in zs if (z, b) in res["fills"]), None)
            allow = FILL * section(zs[-1] if (c == "(+X,-Y)" and b != "A") else zs[0])
            row.append(f"{b} {f['area_mm2'] if f else 0.0:4.1f}/{allow:4.1f} {100 * (f['area_mm2'] if f else 0) / allow:3.0f}%")
        p(f"  {c:8s} " + "  ".join(row))
    p("\nOther zones")
    for (zn, b), f in sorted(res["fills"].items()):
        if any(zn in zs for zs in CORNERS.values()):
            continue
        p(f"  {zn:9s}{b:2s} {f['wires']:3d} wires {f['area_mm2']:5.1f}/{f['allow_mm2']:6.1f} mm2 {100 * f['ratio']:4.0f}%  "
          f"{','.join(f['harnesses'])}")
    p(f"\nBand B, the interconnect's method: the four bundles leave {res['left_b']:.1f} mm2 after the harnesses with an OBC "
      f"end; the EPS-only harnesses W1, W2, W7, W11 would need {res['eps_only_area']:.1f} mm2 there with these gauges")
    p("\nAmpacity (worst ratio per harness): wire, i_max, N, derated, ratio; contact ratio")
    worst = {}
    for w in res["wires"]:
        if w["amp_ratio"] is None:
            continue
        if w["harness"] not in worst or w["amp_ratio"] > worst[w["harness"]]["amp_ratio"]:
            worst[w["harness"]] = w
    for h, w in sorted(worst.items()):
        cmax = max(x["contact_ratio"] for x in res["wires"] if x["harness"] == h and x["amp_ratio"] is not None)
        p(f"  {h:6s} {w['wire']:7s} {w['signal'][:16]:16s} {w['awg']:>3s} AWG {float(w['i_max_A']):6.3f} A  N {w['N']:3d}  "
          f"{w['I_allow']:5.2f} A  {100 * w['amp_ratio']:3.0f}%   contacts {100 * cmax:3.0f}%")
    p("\nVoltage drop at the design current (conductor at +60 C, contacts included)")
    for c, d in sorted(res["drops"].items()):
        p(f"  {c:10s} {','.join(d['harness']):10s} {d['I_A']:5.3f} A  R {d['R_loop_ohm'] * 1000:6.1f} mohm  "
          f"{d['drop_V'] * 1000:6.1f} mV  limit {d['limit_V'] * 1000:4.0f} mV")
    p("\nMass (g)")
    for h, m in res["per_h"].items():
        p(f"  {h:6s} wire {m['wire_g']:5.2f}  connectors/splices {m['conn_g']:5.2f}")
    for k, v in res["extras"].items():
        p(f"  {v:5.1f}  {k}")
    p(f"  total {res['total']:.1f} g")
    for row in sorted(res["m_row"]):
        p(f"  MEL row {row}: {res['m_row'][row]:5.1f} g CBE against {MEL_CBE[row]:5.1f} g "
          f"(delta {res['m_row'][row] - MEL_CBE[row]:+5.1f} g)")
    p("\nEPS bottom-side connector zones: mated footprints / zone plan area")
    for z, v in res["ezones"].items():
        p(f"  {z}  {v['footprint_mm2']:5.0f} / {v['area_mm2']:5.0f} mm2  {100 * v['ratio']:3.0f}%  {', '.join(v['connectors'])}")
    p("\nPA supply pair, 5 mm untwisted at each end, at the magnetometer (worst orientation)")
    for k, v in res["mag"].items():
        p(f"  {k:12s} r {v['r_mm']:5.1f} mm  {v['B_nT']:.2f} nT")
    if cad:
        p("\nCAD check (zone box against every CAD solid)")
        for zn, v in cad.items():
            p(f"  {zn:9s} {'INTERFERES ' + str(v['interferes']) if v['interferes'] else 'clear'}  nearest {v['nearest']}")
    if wv:
        p("\nWireViz sources")
        for fn, name, msg in wv:
            p(f"  {fn:24s} {name:10s} {msg}")
    p("\nProblems: " + ("none" if not res["problems"] else ""))
    for x in res["problems"]:
        p("  " + x)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--cad", action="store_true")
    ap.add_argument("--wireviz", action="store_true")
    a = ap.parse_args()
    res = compute()
    cad = cad_check(res) if a.cad else None
    wv = wireviz_check(res) if a.wireviz else None
    report(res, cad, wv)
    out = dict(
        lengths_mm={w["wire"]: w["length_mm"] for w in res["wires"]},
        fills={f"{k[0]}{('/' + k[1]) if k[1] else ''}": {kk: (round(vv, 3) if isinstance(vv, float) else vv)
                                                       for kk, vv in v.items()} for k, v in res["fills"].items()},
        drops={k: {kk: (round(vv, 5) if isinstance(vv, float) else vv) for kk, vv in v.items()}
               for k, v in res["drops"].items()},
        ampacity={w["wire"]: dict(i_max_A=float(w["i_max_A"]), N=w["N"], I_allow_A=round(w["I_allow"], 3),
                                  ratio=round(w["amp_ratio"], 3), contact_ratio=round(w["contact_ratio"], 3))
                  for w in res["wires"] if w["amp_ratio"] is not None},
        mass_g=dict(per_harness={h: {k: round(v, 2) for k, v in m.items()} for h, m in res["per_h"].items()},
                    extras=res["extras"], total=round(res["total"], 1),
                    by_mel_row={k: round(v, 1) for k, v in res["m_row"].items()}, mel_cbe=MEL_CBE),
        band_B=dict(corner_fill_mm2={k: round(v, 2) for k, v in res["corner_b"].items()},
                    left_after_routed_mm2=round(res["left_b"], 1), eps_only_demand_mm2=round(res["eps_only_area"], 1)),
        magnetometer_nT={k: round(v["B_nT"], 2) for k, v in res["mag"].items()},
        eps_connector_zones={k: {kk: (round(vv, 2) if isinstance(vv, float) else vv) for kk, vv in v.items()}
                             for k, v in res["ezones"].items()},
        cad=cad, problems=res["problems"],
    )
    jf = os.path.join(HERE, "harness_results.json")
    if cad is None and os.path.exists(jf):          # keep the last CAD check when it is not re-run
        try:
            out["cad"] = json.load(open(jf)).get("cad")
        except ValueError:
            pass
    with open(jf, "w") as f:
        json.dump(out, f, indent=1, sort_keys=True)
    return 1 if res["problems"] else 0


if __name__ == "__main__":
    sys.exit(main())
