#!/usr/bin/env python3
"""CDR power closure from mission-simulation runs (SQSO-PWR-CDR-001).

Reads the outputs of `sqso_sil` runs (timeseries.csv, orbits.csv, summary.json)
and computes, per run and per 30-day window:

* the orbit-average array power at its maximum power point (MPP) and what the
  EPS can deliver to the bus from it (the simulation's chain: blocking diode
  and harness 0.97, MPPT 0.92; the strings' own 0.97 harness/mismatch factor is
  already in the MPP figure);
* the as-flown load (what the flight software actually switched on) and the
  margin of the delivered generation over it;
* the eclipse fraction, beta angle, depth of discharge, lowest SOC, heater duty
  and charge-refused time.

It then builds the mode-by-mode closure (SAFE, IDLE, ACTIVE, EMERGENCY) from a
bottom-up load list that uses the simulation's own device values
(`sqso-sil/src/mission.rs`, `hw/radio.rs`, `hw/eps.rs`, `jetson.rs`) at the
duty limits of BASELINE §5, against the simulated generation of the attitude
each mode flies (SAFE and EMERGENCY switch the ADCS off: tumbling; IDLE and
ACTIVE: spin-stabilised +Y sun pointing).

Usage:
    python3 power_closure.py --runs <dir>=<label> [...] [--json out.json]

The run matrix used for the CDR (BASELINE §5 replacement) is in
`run_matrix.txt` beside this script; `run_matrix.sh` runs it.
"""

import argparse
import csv
import json
import math
import os
import sys

# EPS chain of the simulation (hw/eps.rs): array MPP -> bus
DIODE = 0.97
MPPT = 0.92
CHAIN = DIODE * MPPT
# Design chain of SOLAR_ARRAY_DESIGN §9.1 (MPPT 0.95; the harness factor is
# already in the strings): for the sensitivity line only
CHAIN_DESIGN = 0.95
# Battery round trip for the energy that passes through the pack in eclipse
# (coulombic 0.998, about 2 x 1.5 % resistive at C/45; a conservative 0.95)
ROUND_TRIP = 0.95
BUCK_3V3 = 0.85
BUCK_5V = 0.90

# Device values of the simulation (W at the device)
OBC_W = 0.165
EPS_Q_W = 0.08
RX_W = 0.15
ADCS_SENS_W = 0.12
MTQ_SPIN_W = 0.0      # measured: the spin hold uses almost no dipole
MTQ_DETUMBLE_W = 0.30
PA_DC_LOW_W = 1.5     # 0.5 W RF
PA_DC_HIGH_W = 2.7    # 1.0 W RF
BEACON_S = 0.31 + 200 * 8 * 1.02 / 1200  # TX delay + 200 B frame
GNSS_ALLOC_W = 0.05   # BASELINE §5 allocation (not modelled in the SIL)
JETSON_W = 8.0        # BASELINE §6 input limit


def read_ts(path):
    with open(path, newline="") as f:
        r = csv.DictReader(f)
        cols = ["t_s", "array_mpp_w", "array_w", "load_w", "illum", "heater_on", "soc", "t_batt",
                "jetson_w", "xpdr_keyed", "radiating", "power_mode", "op_state", "sun_angle_deg",
                "beta_deg", "t_obc", "t_pa", "t_jetson"]
        out = {c: [] for c in cols}
        for row in r:
            for c in cols:
                out[c].append(float(row[c]))
    return out


def read_orbits(path):
    with open(path, newline="") as f:
        return [{k: float(v) for k, v in row.items()} for row in csv.DictReader(f)]


def weighted_mean(t, x, t0, t1):
    """Time-weighted mean of a sampled series over [t0, t1) (sample-and-hold)."""
    s = w = 0.0
    for i in range(len(t) - 1):
        a, b = t[i], t[i + 1]
        if b <= t0 or a >= t1:
            continue
        dt = min(b, t1) - max(a, t0)
        s += x[i] * dt
        w += dt
    return s / w if w > 0 else float("nan")


def run_stats(d, window_days=30.0):
    ts = read_ts(os.path.join(d, "timeseries.csv"))
    orb = read_orbits(os.path.join(d, "orbits.csv"))
    t = ts["t_s"]
    t_end = t[-1]
    t_start = max(0.0, 0.0)
    # skip LEOP (first 0.5 day: detumble, deployment)
    t_ops = 0.5 * 86400.0
    windows = []
    w = window_days * 86400.0
    a = t_ops
    while a < t_end - 0.5 * 86400.0:
        b = min(a + w, t_end)
        orbs = [o for o in orb if a <= o["start_s"] < b]
        if not orbs:
            break
        mpp = weighted_mean(t, ts["array_mpp_w"], a, b)
        load = weighted_mean(t, ts["load_w"], a, b)
        heater = weighted_mean(t, ts["heater_on"], a, b)
        jet = weighted_mean(t, ts["jetson_w"], a, b)
        ecl = sum(o["eclipse_s"] for o in orbs) / sum(o["len_s"] for o in orbs)
        windows.append({
            "day0": a / 86400.0, "day1": b / 86400.0,
            "beta_min": min(o["beta_deg"] for o in orbs), "beta_max": max(o["beta_deg"] for o in orbs),
            "eclipse_frac": ecl, "mpp_w": mpp, "deliverable_w": mpp * CHAIN, "load_w": load,
            "margin": (mpp * CHAIN - load * (1 + ecl * (1 / ROUND_TRIP - 1))) / load,
            "heater_duty": heater, "jetson_w": jet,
            "dod_max": max(o["dod"] for o in orbs), "soc_min": min(o["soc_min"] for o in orbs),
            "tbatt_min": min(o["tbatt_min"] for o in orbs), "tbatt_max": max(o["tbatt_max"] for o in orbs),
            "charge_blocked_s": sum(o["charge_blocked_s"] for o in orbs),
            "heater_orbit_max": max(o["heater_s"] / o["len_s"] for o in orbs),
            "tpa_max": max(o["tpa_max"] for o in orbs), "tjet_max": max(o["tjetson_max"] for o in orbs),
            "xpdr_keyed_min_day": sum(o["xpdr_keyed_s"] for o in orbs) / 60.0 / ((b - a) / 86400.0),
        })
        a = b
    summ = {}
    sp = os.path.join(d, "summary.json")
    if os.path.exists(sp):
        with open(sp) as f:
            summ = json.load(f)
    ops = [i for i in range(len(t)) if t[i] >= t_ops]
    whole = {
        "mpp_w": weighted_mean(t, ts["array_mpp_w"], t_ops, t_end),
        "load_w": weighted_mean(t, ts["load_w"], t_ops, t_end),
        "soc_min": min(ts["soc"][i] for i in ops) if ops else float("nan"),
        "tbatt_min": min(ts["t_batt"][i] for i in ops) if ops else float("nan"),
        "tbatt_max": max(ts["t_batt"][i] for i in ops) if ops else float("nan"),
        "t_obc_min": min(ts["t_obc"][i] for i in ops) if ops else float("nan"),
        "t_obc_max": max(ts["t_obc"][i] for i in ops) if ops else float("nan"),
        "t_pa_max": max(ts["t_pa"][i] for i in ops) if ops else float("nan"),
        "t_jet_max": max(ts["t_jetson"][i] for i in ops) if ops else float("nan"),
        "tbatt_leop_min": min(ts["t_batt"]),
        "anomalies": summ.get("anomalies"),
    }
    whole["deliverable_w"] = whole["mpp_w"] * CHAIN
    return windows, whole


def mode_loads():
    """Orbit-average load at the pack for each mode, W, at the BASELINE §5 duty limits.

    Returns {mode: (total, breakdown)}; the heater is listed separately (it
    depends on the season and the attitude; the thermal analysis gives it).
    """
    base = OBC_W / BUCK_3V3 + EPS_Q_W + RX_W / BUCK_3V3
    beacon = lambda period: PA_DC_LOW_W / BUCK_5V * BEACON_S / period
    adcs = (ADCS_SENS_W + MTQ_SPIN_W) / BUCK_3V3
    gnss = GNSS_ALLOC_W
    out = {}
    out["EMERGENCY"] = {"OBC, EPS, receiver": base, "beacon": 0.0}
    out["SAFE"] = {"OBC, EPS, receiver": base, "beacon (180 s)": beacon(180.0), "GNSS (allocation)": gnss}
    out["IDLE"] = {"OBC, EPS, receiver": base, "beacon (120 s)": beacon(120.0), "ADCS (spin hold)": adcs,
                   "GNSS (allocation)": gnss}
    for name, xp, jd, label in [
        ("ACTIVE, BASELINE duty limits (sun-pointed)", 0.25, 0.10, "sun"),
        ("ACTIVE, BASELINE duty limits (tumbling)", 0.25, 0.05, "tumble"),
    ]:
        # transponder and Jetson are sunlit-only; their duty limits are of the orbit (BASELINE §5)
        out[name] = {"OBC, EPS, receiver": base, "beacon (60 s)": beacon(60.0), "ADCS (spin hold)": adcs,
                     "GNSS (allocation)": gnss,
                     "transponder 0.5 W RF, %d %% duty" % round(xp * 100): PA_DC_LOW_W / BUCK_5V * xp,
                     "Jetson 8 W, %d %% duty" % round(jd * 100): JETSON_W / BUCK_5V * jd}
    return {k: (sum(v.values()), v) for k, v in out.items()}


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--runs", nargs="+", required=True, help="dir=label")
    ap.add_argument("--json", help="write all numbers here")
    args = ap.parse_args()
    allres = {}
    print("## Runs (operations, LEOP excluded)\n")
    print("| Run | Days | β range (°) | Eclipse | MPP avg (W) | Deliverable (W) | As-flown load (W) | Margin | Max DoD | Lowest SOC | Battery (°C) | Heater duty, worst orbit | Charge refused (s) | Anomalies |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|")
    for spec in args.runs:
        d, label = spec.split("=", 1)
        if not os.path.exists(os.path.join(d, "orbits.csv")):
            print(f"| {label} | missing | | | | | | | | | | | | |")
            continue
        wins, whole = run_stats(d)
        allres[label] = {"windows": wins, "whole": whole}
        bmin = min(x["beta_min"] for x in wins)
        bmax = max(x["beta_max"] for x in wins)
        ecl = max(x["eclipse_frac"] for x in wins)
        worst = min(wins, key=lambda x: x["margin"])
        print(f"| {label} | {wins[-1]['day1']:.0f} | {bmin:.1f} … {bmax:.1f} | ≤ {ecl * 100:.1f} % | "
              f"{whole['mpp_w']:.2f} (worst 30 d {worst['mpp_w']:.2f}) | {whole['deliverable_w']:.2f} | {whole['load_w']:.2f} | "
              f"{min(x['margin'] for x in wins) * 100:+.0f} % (worst 30 d) | {max(x['dod_max'] for x in wins) * 100:.1f} % | "
              f"{whole['soc_min'] * 100:.1f} % | {whole['tbatt_min']:.1f} … {whole['tbatt_max']:.1f} | "
              f"{max(x['heater_orbit_max'] for x in wins) * 100:.0f} % | {sum(x['charge_blocked_s'] for x in wins):.0f} | {whole['anomalies']} |")
    print("\n## 30-day windows\n")
    for label, r in allres.items():
        print(f"\n### {label}\n")
        print("| Days | β (°) | Eclipse | MPP (W) | Deliverable (W) | Load (W) | Margin | Heater duty | Max DoD | Battery min/max (°C) |")
        print("|---|---|---|---|---|---|---|---|---|---|")
        for x in r["windows"]:
            print(f"| {x['day0']:.0f}–{x['day1']:.0f} | {x['beta_min']:.1f}…{x['beta_max']:.1f} | {x['eclipse_frac'] * 100:.1f} % | "
                  f"{x['mpp_w']:.2f} | {x['deliverable_w']:.2f} | {x['load_w']:.2f} | {x['margin'] * 100:+.0f} % | "
                  f"{x['heater_duty'] * 100:.0f} % | {x['dod_max'] * 100:.1f} % | {x['tbatt_min']:.1f}/{x['tbatt_max']:.1f} |")
    print("\n## Mode loads at the pack (BASELINE §5 duty limits; heater excluded)\n")
    ml = mode_loads()
    for k, (tot, br) in ml.items():
        parts = ", ".join(f"{n} {v:.3f}" for n, v in br.items())
        print(f"- **{k}: {tot:.3f} W** ({parts})")
    allres["mode_loads"] = {k: {"total": t, "items": b} for k, (t, b) in ml.items()}
    allres["chain"] = {"sim": CHAIN, "design": CHAIN_DESIGN, "round_trip": ROUND_TRIP}
    if args.json:
        with open(args.json, "w") as f:
            json.dump(allres, f, indent=1)


if __name__ == "__main__":
    sys.exit(main())
