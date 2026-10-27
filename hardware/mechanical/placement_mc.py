#!/usr/bin/env python3
"""
SMART-QSO placement Monte Carlo of the deployed principal axis and inertia ratio (SQSO-MECH-CAD-001 section 5.4,
MASS_BUDGET section 6.6; reproduces the table there, review item P3S-09).

Every internal part of model.py (the ballast and the harness zones included) is moved by an independent Gaussian
error of sigma per axis; the frame (rails, end frames, shear panels, deck), the body panels, the wings and the AntS
are held, because the frame locates them.  Each part keeps its own inertia about its centroid.  For each trial the
deployed tensor about the CG is formed and the major principal axis's angle from +Y and the ratio
Iyy / max(Ixx, Izz) are recorded, at CBE and at MEV.

    python3 hardware/mechanical/placement_mc.py [--sigmas 1,2,3,5] [--trials 2000] [--seed 1]
                                                [--json hardware/mechanical/placement_mc.json]

Needs CadQuery (as model.py).  Output: mean and 95th-percentile angle, fraction within 5 degrees and the
5th-percentile ratio per sigma and mass basis.
"""
from __future__ import annotations

import argparse
import json
import math
import os
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import model as M  # noqa: E402

FIXED_GROUPS = ("frame", "panel", "wing", "ants")


def part_data(parts, masses):
    data = []
    for pt in parts:
        m = masses[pt.name]
        if m <= 0:
            continue
        shp = pt.deployed if pt.deployed is not None else pt.stowed
        v, c, iu = M.solid_props(shp)
        fixed = pt.group in FIXED_GROUPS or pt.name.startswith(("rail", "W1", "W2"))
        data.append((m * 1e-3, np.array(c) * 1e-3, iu * m / v * 1e-9, fixed))   # kg, m, kg m^2
    return data


def trial(data, rng, sigma_mm):
    tot = sum(d[0] for d in data)
    cs = [c if fixed else c + rng.normal(0.0, sigma_mm * 1e-3, 3) for m, c, ip, fixed in data]
    cg = sum(d[0] * c for d, c in zip(data, cs)) / tot
    inertia = np.zeros((3, 3))
    for (m, c, ip, fixed), cc in zip(data, cs):
        r = cc - cg
        inertia += ip + m * (r @ r * np.eye(3) - np.outer(r, r))
    w, v = np.linalg.eigh(inertia)
    ang = math.degrees(math.acos(min(1.0, abs(v[1, 2]))))
    return ang, inertia[1, 1] / max(inertia[0, 0], inertia[2, 2])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sigmas", default="1,2,3,5")
    ap.add_argument("--trials", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=1)
    ap.add_argument("--json", default=os.path.join(HERE, "placement_mc.json"))
    args = ap.parse_args()
    parts = M.build_parts(M.P)
    out = {"trials": args.trials, "seed": args.seed, "fixed_groups": FIXED_GROUPS, "results": {}}
    for grown in (False, True):
        data = part_data(parts, M.assign_masses(parts, grown))
        nominal = trial([(m, c, ip, True) for m, c, ip, _ in data], np.random.default_rng(0), 0.0)
        basis = "mev" if grown else "cbe"
        out["results"][f"{basis}_nominal"] = dict(angle_deg=nominal[0], ratio=nominal[1])
        print(f"{basis} nominal: angle {nominal[0]:.2f} deg, ratio {nominal[1]:.4f}")
        rng = np.random.default_rng(args.seed)
        for s in [float(x) for x in args.sigmas.split(",")]:
            res = np.array([trial(data, rng, s) for _ in range(args.trials)])
            r = dict(angle_mean_deg=float(res[:, 0].mean()), angle_p95_deg=float(np.percentile(res[:, 0], 95)),
                     within_5deg=float((res[:, 0] <= 5.0).mean()), ratio_p5=float(np.percentile(res[:, 1], 5)))
            out["results"][f"{basis}_sigma{s:g}mm"] = r
            print(f"{basis} sigma {s:g} mm: mean {r['angle_mean_deg']:.1f} deg, p95 {r['angle_p95_deg']:.1f} deg, "
                  f"within 5 deg {r['within_5deg'] * 100:.1f} %, ratio p5 {r['ratio_p5']:.3f}")
    with open(args.json, "w") as f:
        json.dump(out, f, indent=1)


if __name__ == "__main__":
    main()
