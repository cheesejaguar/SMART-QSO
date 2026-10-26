#!/usr/bin/env python3
"""Report of a SMART-QSO SIL run (software/rust/sqso-sil).

Reads a run's output directory (config.json, summary.json, timeseries.csv,
orbits.csv, passes.csv, events.jsonl) and writes report.md with plots in
plots/ beside it.

    python3 software/simulation/sil/report.py OUT_DIR [--title TEXT]
"""

import argparse
import csv
import json
import math
import os
from collections import Counter

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402

DAY = 86400.0
STATES = {0: "BOOT", 1: "DETUMBLE", 2: "SAFE", 3: "IDLE", 4: "ACTIVE", 5: "EMERGENCY"}


def read_csv(path):
    if not os.path.exists(path):
        return {}
    with open(path, newline="") as f:
        rows = list(csv.DictReader(f))
    if not rows:
        return {}
    cols = {k: [] for k in rows[0]}
    for r in rows:
        for k, v in r.items():
            try:
                cols[k].append(float(v))
            except (TypeError, ValueError):
                cols[k].append(float("nan"))
    return cols


def read_events(path):
    if not os.path.exists(path):
        return []
    with open(path) as f:
        return [json.loads(line) for line in f if line.strip()]


def save(fig, out, name):
    os.makedirs(os.path.join(out, "plots"), exist_ok=True)
    path = os.path.join("plots", name)
    fig.tight_layout()
    fig.savefig(os.path.join(out, path), dpi=110)
    plt.close(fig)
    return path


def days(xs):
    return [x / DAY for x in xs]


def plot_orbits(o, out):
    figs = []
    if not o:
        return figs
    d = days(o["start_s"])
    fig, ax = plt.subplots(3, 1, figsize=(10, 8), sharex=True)
    ax[0].plot(d, o["beta_deg"], lw=0.8)
    ax[0].set_ylabel("beta (°)")
    ax[1].plot(d, [e / 60 for e in o["eclipse_s"]], lw=0.8, color="k")
    ax[1].set_ylabel("eclipse (min)")
    ax[2].plot(d, o["rate_dps"], lw=0.8, color="tab:purple")
    ax[2].set_ylabel("body rate (°/s)")
    ax[2].set_xlabel("days after deployment")
    ax[0].set_title("Orbit geometry and attitude, per orbit")
    figs.append(("Orbit geometry and attitude", save(fig, out, "orbit_geometry.png")))

    fig, ax = plt.subplots(3, 1, figsize=(10, 8), sharex=True)
    ax[0].plot(d, o["gen_wh"], lw=0.6, label="generated")
    ax[0].plot(d, o["load_wh"], lw=0.6, label="consumed")
    ax[0].set_ylabel("Wh per orbit")
    ax[0].legend(loc="upper right")
    ax[1].fill_between(d, [100 * v for v in o["soc_min"]], [100 * v for v in o["soc_max"]], alpha=0.5)
    ax[1].axhline(55, ls="--", lw=0.6, color="g")
    ax[1].axhline(40, ls="--", lw=0.6, color="orange")
    ax[1].axhline(25, ls="--", lw=0.6, color="r")
    ax[1].set_ylabel("SOC range (%)")
    ax[2].plot(d, [100 * v for v in o["dod"]], lw=0.6, color="tab:red")
    ax[2].axhline(10, ls="--", lw=0.6, color="k")
    ax[2].set_ylabel("DoD per eclipse (%)")
    ax[2].set_xlabel("days after deployment")
    ax[0].set_title("Power, per orbit")
    figs.append(("Power", save(fig, out, "power_orbits.png")))

    fig, ax = plt.subplots(3, 1, figsize=(10, 8), sharex=True)
    ax[0].fill_between(d, o["tbatt_min"], o["tbatt_max"], alpha=0.5, color="tab:orange")
    ax[0].axhline(0, ls="--", lw=0.6, color="b")
    ax[0].axhline(45, ls="--", lw=0.6, color="r")
    ax[0].set_ylabel("battery (°C)")
    ax[1].plot(d, o["tpa_max"], lw=0.6, label="PA max")
    ax[1].plot(d, o["tjetson_max"], lw=0.6, label="Jetson max")
    ax[1].legend(loc="upper right")
    ax[1].set_ylabel("°C")
    ax[2].plot(d, [v / 60 for v in o["heater_s"]], lw=0.6, label="heater on")
    ax[2].plot(d, [v / 60 for v in o["charge_blocked_s"]], lw=0.6, label="charge blocked")
    ax[2].legend(loc="upper right")
    ax[2].set_ylabel("min per orbit")
    ax[2].set_xlabel("days after deployment")
    ax[0].set_title("Thermal, per orbit")
    figs.append(("Thermal", save(fig, out, "thermal_orbits.png")))

    fig, ax = plt.subplots(2, 1, figsize=(10, 6), sharex=True)
    ax[0].plot(d, [v / 60 for v in o["payload_s"]], lw=0.6, color="tab:green")
    ax[0].set_ylabel("Jetson on (min/orbit)")
    ax[1].plot(d, o["radiating_s"], lw=0.6, color="tab:purple")
    ax[1].set_ylabel("downlink keyed (s/orbit)")
    ax[1].set_xlabel("days after deployment")
    ax[0].set_title("Payload and downlink activity, per orbit")
    figs.append(("Payload and downlink", save(fig, out, "activity_orbits.png")))
    return figs


def plot_timeseries(ts, out, detail_days):
    figs = []
    if not ts:
        return figs
    t = ts["t_s"]
    n = next((i for i, v in enumerate(t) if v > detail_days * DAY), len(t))
    h = [v / 3600 for v in t[:n]]
    if n < 2:
        return figs
    fig, ax = plt.subplots(5, 1, figsize=(11, 11), sharex=True)
    ax[0].plot(h, [100 * v for v in ts["soc"][:n]], label="true")
    ax[0].plot(h, [100 * v for v in ts["soc_fsw"][:n]], lw=0.7, label="flight software")
    ax[0].set_ylabel("SOC (%)")
    ax[0].legend(loc="lower right")
    ax[1].plot(h, ts["array_w"][:n], lw=0.6, label="array")
    ax[1].plot(h, ts["load_w"][:n], lw=0.6, label="load")
    ax[1].set_ylabel("W")
    ax[1].legend(loc="upper right")
    ax[2].plot(h, ts["rate_dps"][:n], color="tab:purple")
    ax[2].set_ylabel("rate (°/s)")
    ax[2].set_yscale("symlog", linthresh=0.5)
    ax[3].plot(h, ts["t_batt"][:n], label="battery")
    ax[3].plot(h, ts["t_obc"][:n], label="OBC")
    ax[3].plot(h, ts["t_pa"][:n], label="PA")
    ax[3].set_ylabel("°C")
    ax[3].legend(loc="upper right")
    ax[4].step(h, ts["op_state"][:n], where="post")
    ax[4].set_yticks(list(STATES))
    ax[4].set_yticklabels([STATES[k] for k in STATES])
    ax[4].set_xlabel("hours after deployment")
    ax[0].set_title(f"First {detail_days:g} days")
    figs.append(("Launch and early operations", save(fig, out, "leop.png")))

    fig, ax = plt.subplots(2, 1, figsize=(10, 6), sharex=True)
    dd = days(t)
    ax[0].plot(dd, ts["alt_km"], lw=0.8)
    ax[0].set_ylabel("altitude (km)")
    ax[1].plot(dd, ts["capacity_ah"], lw=0.8, color="tab:brown")
    ax[1].set_ylabel("capacity (Ah)")
    ax[1].set_xlabel("days after deployment")
    ax[0].set_title("Altitude and battery capacity")
    figs.append(("Altitude and capacity", save(fig, out, "life.png")))
    return figs


def plot_reentry(path, out):
    d = read_csv(path)
    if not d:
        return []
    fig, ax = plt.subplots(figsize=(10, 4))
    ax.plot([v / 365.25 for v in days(d["t_s"])], d["alt_km"])
    ax.set_xlabel("years after deployment")
    ax.set_ylabel("mean altitude (km)")
    ax.set_title("Orbit decay to re-entry")
    return [("Orbit decay", save(fig, out, "decay.png"))]


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("out")
    p.add_argument("--title", default="SMART-QSO SIL mission run")
    a = p.parse_args()
    out = a.out
    cfg = json.load(open(os.path.join(out, "config.json")))
    summ = json.load(open(os.path.join(out, "summary.json")))
    orbits = read_csv(os.path.join(out, "orbits.csv"))
    ts = read_csv(os.path.join(out, "timeseries.csv"))
    events = read_events(os.path.join(out, "events.jsonl"))
    passes = read_csv(os.path.join(out, "passes.csv"))

    figs = []
    figs += plot_timeseries(ts, out, float(cfg.get("detail_days", 3)))
    figs += plot_orbits(orbits, out)
    figs += plot_reentry(os.path.join(out, "decay.csv"), out)

    checks = summ["checks"]
    verdicts = Counter(c["verdict"] for c in checks)
    lines = [f"# {a.title}", ""]
    lines.append(f"Scenario `{cfg['scenario']}`, seed {cfg['seed']}, deployment {cfg['deploy_utc']}, "
                 f"{cfg['altitude_km']} km SSO, LTAN {cfg['ltan_h']} h, design life {cfg['mission_days']} days.")
    lines.append("")
    lines.append(f"**{verdicts.get('Anomaly', 0)} anomalies, {verdicts.get('Warning', 0)} warnings, "
                 f"{verdicts.get('Pass', 0)} passes.**")
    lines.append("")
    lines.append("## Checks")
    lines.append("")
    lines.append("| Check | Value | Limit | Verdict |")
    lines.append("|-------|-------|-------|---------|")
    for c in checks:
        mark = {"Pass": "pass", "Warning": "**warning**", "Anomaly": "**ANOMALY**"}[c["verdict"]]
        lines.append(f"| {c['title']} | {c['value']} | {c['limit']} | {mark} |")
    lines.append("")
    extra = summ.get("totals", {})
    keys = [k for k in extra if k != "debug"]
    if keys:
        lines.append("## Totals")
        lines.append("")
        lines.append("| Quantity | Value |")
        lines.append("|----------|-------|")
        for k in keys:
            lines.append(f"| {k} | {extra[k]} |")
        lines.append("")
    faults = summ.get("faults", {})
    if faults:
        lines.append("## Flight-software fault log entries")
        lines.append("")
        lines.append("| Entry | Count |")
        lines.append("|-------|-------|")
        for k, v in sorted(faults.items(), key=lambda kv: -kv[1]):
            lines.append(f"| {k} | {v} |")
        lines.append("")
    kinds = Counter(e["kind"] for e in events)
    lines.append("## Events")
    lines.append("")
    lines.append(", ".join(f"{k}: {v}" for k, v in sorted(kinds.items())))
    lines.append("")
    key = [e for e in events if e["kind"] in ("phase", "deploy", "reset", "anomaly", "passivation", "reentry", "command")]
    if key:
        lines.append("| Time | UTC | Kind | Detail |")
        lines.append("|------|-----|------|--------|")
        for e in key[:80]:
            t = e["t_s"]
            when = f"{t / 3600:.2f} h" if abs(t) < 3 * DAY else f"{t / DAY:.1f} d"
            lines.append(f"| {when} | {e['utc']} | {e['kind']} | {e['detail']} |")
        if len(key) > 80:
            lines.append(f"| … | | | {len(key) - 80} more in events.jsonl |")
        lines.append("")
    if passes:
        n = len(passes["max_elev_deg"])
        heard = sum(1 for v in passes["heard"] if v > 0)
        lines.append(f"## Ground passes\n\n{n} passes above 5°, {heard} with at least one beacon decoded.\n")
    for title, path in figs:
        lines.append(f"## {title}\n\n![{title}]({path})\n")
    with open(os.path.join(out, "report.md"), "w") as f:
        f.write("\n".join(lines))
    print(os.path.join(out, "report.md"))


if __name__ == "__main__":
    main()
