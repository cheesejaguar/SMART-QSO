#!/usr/bin/env python3
"""CDR thermal design cases on the mission simulation's 12-node network (SQSO-THA-001 v2.0).

Runs the ignored test `env::thermal::tests::cdr_thermal_cases` of `sqso-sil`
(the cases are defined there, next to the network they use), parses its
`CDR|case|node|min|mean|max` lines and prints, for every case, each node's
predicted range, its limits and the margin after the 10 °C model-uncertainty
allowance of BASELINE §8 (GSFC-STD-7000B §2.6.1 and §2.6.4.1 for the test and
heater margins).

Usage (from the repository root):
    python3 software/simulation/cdr/power_thermal/thermal_cases.py [--raw out.txt] [--from out.txt]
"""

import argparse
import collections
import os
import subprocess
import sys

UNCERTAINTY_C = 10.0

# Allowable flight temperatures used for the margins, °C: (op min, op max, survival min, survival max, basis)
LIMITS = {
    "battery": (0.0, 45.0, -20.0, 60.0, "INR18650-35E: charge 0…45, discharge −10…60 (BATTERY_DESIGN §2.1); storage −20…60"),
    "OBC": (-20.0, 50.0, -40.0, 60.0, "SYS-600/601 (parts: STM32L4R5ZIT6 −40…85)"),
    "RF": (-20.0, 50.0, -40.0, 60.0, "SYS-600/601 (PA gate 70 °C in the flight software)"),
    "Jetson": (-20.0, 50.0, -40.0, 60.0, "SYS-600/601; module Tj −25…105 (DS-11105-001 v1.7); flight-software spreader limit 70"),
    "body": (-40.0, 80.0, -60.0, 100.0, "THERMAL_DESIGN §1.1 structure"),
    "wing": (-40.0, 85.0, -100.0, 100.0, "SOLAR_ARRAY_DESIGN §1.1 (to be confirmed with AZUR SPACE qualification)"),
}
PANELS = ["+X", "-X", "+Y", "-Y", "+Z", "-Z"]


def run_cases(repo):
    cmd = ["cargo", "test", "-p", "sqso-sil", "--release", "--lib", "cdr_thermal_cases", "--", "--ignored", "--nocapture"]
    p = subprocess.run(cmd, cwd=os.path.join(repo, "software", "rust"), capture_output=True, text=True, check=True)
    return [l for l in p.stdout.splitlines() if l.startswith("CDR|")]


def parse(lines):
    cases = collections.OrderedDict()
    soak = []
    for l in lines:
        f = l.split("|")
        if f[2].startswith("t="):
            soak.append(f[1:])
            continue
        cases.setdefault(f[1], {})[f[2]] = tuple(float(x) for x in f[3:6])
    return cases, soak


def margin(pred_min, pred_max, lim):
    lo, hi = lim[0], lim[1]
    return pred_min - lo - UNCERTAINTY_C, hi - pred_max - UNCERTAINTY_C


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--raw", help="also write the raw CDR lines here")
    ap.add_argument("--from", dest="src", help="parse this file instead of running the test")
    args = ap.parse_args()
    repo = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", ".."))
    lines = open(args.src).read().splitlines() if args.src else run_cases(repo)
    if args.raw:
        with open(args.raw, "w") as f:
            f.write("\n".join(lines) + "\n")
    cases, soak = parse(lines)
    print("| Case | β (°) | Eclipse (min) | Battery (°C) | OBC (°C) | RF (°C) | Jetson (°C) | Body panels (°C) | Wings (°C) | Heater duty | Worst margin after 10 °C |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for name, v in cases.items():
        body = (min(v[p][0] for p in PANELS), max(v[p][2] for p in PANELS))
        wing = (min(v["W1"][0], v["W2"][0]), max(v["W1"][2], v["W2"][2]))
        rows = {"battery": (v["battery"][0], v["battery"][2]), "OBC": (v["OBC"][0], v["OBC"][2]),
                "RF": (v["RF"][0], v["RF"][2]), "Jetson": (v["Jetson"][0], v["Jetson"][2]),
                "body": body, "wing": wing}
        worst = None
        for node, (lo, hi) in rows.items():
            lim = LIMITS[node]
            if node == "battery":
                # heater-controlled at the cold end: judged by the heater duty and 0 °C, not by +10 °C
                m_lo, m_hi = lo - lim[0], lim[1] - hi - UNCERTAINTY_C
            elif node == "Jetson" and not name.startswith(("H",)):
                # unpowered in SAFE/EMERGENCY/LEOP: survival limit at the cold end
                m_lo, m_hi = lo - LIMITS["Jetson"][2] - UNCERTAINTY_C, lim[1] - hi - UNCERTAINTY_C
            else:
                m_lo, m_hi = margin(lo, hi, lim)
            for m, side in [(m_lo, "cold"), (m_hi, "hot")]:
                if worst is None or m < worst[0]:
                    worst = (m, f"{node} {side}")
        hd = v["heater_duty"][0]
        up = v.get("battery_above_0C_min", (-1,))[0]
        extra = f"; battery ≥ 0 °C after {up:.0f} min" if name.startswith("L") and up >= 0 else ""
        print(f"| {name} | {v['beta_deg'][0]:.1f} | {v['eclipse_min'][0]:.1f} | {rows['battery'][0]:.1f} … {rows['battery'][1]:.1f} | "
              f"{rows['OBC'][0]:.1f} … {rows['OBC'][1]:.1f} | {rows['RF'][0]:.1f} … {rows['RF'][1]:.1f} | "
              f"{rows['Jetson'][0]:.1f} … {rows['Jetson'][1]:.1f} | {body[0]:.1f} … {body[1]:.1f} | {wing[0]:.1f} … {wing[1]:.1f} | "
              f"{hd * 100:.0f} % | {worst[0]:+.1f} ({worst[1]}){extra} |")
    if soak:
        print("\n| Dispenser soak | Time | Wall | Battery | OBC | RF | Jetson | +X panel |")
        print("|---|---|---|---|---|---|---|---|")
        for f in soak:
            print("| " + " | ".join(x.replace("wall ", "").replace("battery ", "").replace("OBC ", "").replace("RF ", "")
                                     .replace("Jetson ", "").replace("+X ", "") for x in f) + " |")
    print("\nLimits used:")
    for k, v in LIMITS.items():
        print(f"- {k}: operating {v[0]:+.0f} … {v[1]:+.0f} °C, survival {v[2]:+.0f} … {v[3]:+.0f} °C ({v[4]})")


if __name__ == "__main__":
    sys.exit(main())
