#!/usr/bin/env python3
"""Telecommand contact cadence against the loss-of-contact timer (CDR).

Feeds SQSO-GND-001 Rev C §4-§5. Uses the pass geometry of pass_geometry_mc.py.

Question: with one or two designated telecommand stations, how often would
the loss-of-contact inhibit (N = 30 days at launch, 14 days after
commissioning; SQSO-ICD-013 §6.2) trip for reasons on the ground, and what
is the longest wait for a TX-OFF (97.207(b)) after an FCC order?

Model
  Commanding pass: maximum elevation >= MIN_EL_CMD (15 deg; the SIL's
  contact_min_elev_deg). An attempt is one authenticated command (about
  60 bytes on air); each succeeds with probability P_ATTEMPT (0.90: the
  telecommand budget's -10 dBi, +6 dB-interference case closes at 10 deg
  with +2.4 dB, so about one attempt in ten falls in an antenna or
  polarization fade); the station sends up to K attempts a pass, checking
  the next beacon (telecommand counter) between them.
  Ground availability: each station is unavailable on a random day with
  probability U_DAY (operator away, weather, software), and suffers long
  outages (equipment failure, fire, power) at RATE_LONG per year lasting
  uniformly 3 to MAX_LONG days. Common-mode loss of both (key or counter
  file lost, licensee unavailable) at RATE_COMMON per year lasting 1-30 days.
  Policy: the station(s) send a routine authenticated NOP on the first
  usable pass of each CADENCE-day period, plus any planned commands.
Monte Carlo over 2-year missions.

Usage: python3 contact_cadence.py [--trials 2000] [--out cadence.json]
"""
from __future__ import annotations

import argparse
import json
import math

import numpy as np

from pass_geometry_mc import STATIONS, orbit_eci, passes_for

MIN_EL_CMD = 15.0
P_ATTEMPT = 0.90
K_ATTEMPTS = 4
U_DAY = 0.10
RATE_LONG = 1.0       # per station-year
MAX_LONG = 21.0       # days
RATE_COMMON = 0.25    # per year
MISSION_DAYS = 730


def pass_days(name, days=30.0):
    """Commanding passes (start time, s) over `days` of the baseline orbit, then tiled"""
    t = np.arange(0.0, days * 86400.0, 2.0)
    r, v, jd = orbit_eci(t)
    la, lo = STATIONS[name]
    ps, el, _, _ = passes_for(t, r, jd, la, lo, 5.0)
    starts = [t[s] for s, e in ps if el[s:e].max() >= MIN_EL_CMD]
    return np.array(starts), len(ps) / days, len(starts) / days


def simulate(station_passes, cadence_days, n_days, rng, trials):
    """Returns the list, per trial, of the longest interval without a successful command (days)"""
    tile = {}
    for k, (st, period) in station_passes.items():
        reps = int(math.ceil(MISSION_DAYS / period)) + 1
        tile[k] = np.concatenate([st + i * period * 86400.0 for i in range(reps)])
    longest = []
    trips = 0
    for _ in range(trials):
        down = {}
        for k in station_passes:
            d = rng.random(MISSION_DAYS) < U_DAY
            n_long = rng.poisson(RATE_LONG * MISSION_DAYS / 365)
            for _ in range(n_long):
                s0 = rng.integers(0, MISSION_DAYS)
                d[s0:s0 + int(rng.uniform(3, MAX_LONG))] = True
            down[k] = d
        common = np.zeros(MISSION_DAYS, bool)
        for _ in range(rng.poisson(RATE_COMMON * MISSION_DAYS / 365)):
            s0 = rng.integers(0, MISSION_DAYS)
            common[s0:s0 + int(rng.uniform(1, 30))] = True
        # successful contacts
        events = []
        for k, ts in tile.items():
            ts = ts[ts < MISSION_DAYS * 86400.0]
            day = (ts // 86400).astype(int)
            ok_station = ~down[k][day] & ~common[day]
            p_pass = 1 - (1 - P_ATTEMPT) ** K_ATTEMPTS
            ok = ok_station & (rng.random(len(ts)) < p_pass)
            events.append(ts[ok])
        ev = np.sort(np.concatenate(events))
        # cadence: one contact per period is enough; the stations try on every usable pass
        # until one succeeds in that period (so contact times are the first success per period)
        if cadence_days > 0:
            per = (ev // (cadence_days * 86400)).astype(int)
            _, first = np.unique(per, return_index=True)
            ev = ev[first]
        ev = np.concatenate([[0.0], ev, [MISSION_DAYS * 86400.0]])
        gap = np.diff(ev).max() / 86400
        longest.append(gap)
        if gap > n_days:
            trips += 1
    return np.array(longest), trips / trials


def max_wait_for_txoff(station_passes):
    """Longest time from an arbitrary moment to the next commanding pass (h), from the pass lists"""
    allp = np.sort(np.concatenate([st for st, _ in station_passes.values()]))
    return float(np.diff(allp).max() / 3600)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--trials", type=int, default=2000)
    ap.add_argument("--out", default=None)
    ap.add_argument("--common-rate", type=float, default=RATE_COMMON,
                    help="common-mode ground outages per year (0 isolates the station-independent part)")
    args = ap.parse_args()
    globals()["RATE_COMMON"] = args.common_rate
    rng = np.random.default_rng(14)
    res = {"model": {"min_el_cmd": MIN_EL_CMD, "p_attempt": P_ATTEMPT, "attempts": K_ATTEMPTS, "u_day": U_DAY,
                     "rate_long_per_yr": RATE_LONG, "max_long_d": MAX_LONG, "rate_common_per_yr": args.common_rate}}
    pas = {}
    for name in ("SantaClara_CA", "Fairbanks_AK", "Blacksburg_VA", "Honolulu_HI"):
        st, ppd, cpd = pass_days(name)
        pas[name] = (st, 30.0)
        res[name] = {"passes_per_day_5deg": round(ppd, 2), "commanding_passes_per_day": round(cpd, 2)}
        print(name, res[name])
    cases = {
        "SC_only": ["SantaClara_CA"],
        "SC+Blacksburg": ["SantaClara_CA", "Blacksburg_VA"],
        "SC+Fairbanks": ["SantaClara_CA", "Fairbanks_AK"],
        "SC+Honolulu": ["SantaClara_CA", "Honolulu_HI"],
    }
    for cname, sts in cases.items():
        sp = {k: pas[k] for k in sts}
        out = {"max_wait_txoff_h": round(max_wait_for_txoff(sp), 1)}
        for cad in (1, 3, 7):
            longest, p14 = simulate(sp, cad, 14, rng, args.trials)
            _, p30 = simulate(sp, cad, 30, rng, max(200, args.trials // 4))
            out[f"cadence_{cad}d"] = {"p_trip_N14_per_mission": round(p14, 4), "p_trip_N30_per_mission": round(p30, 4),
                                      "longest_gap_p50_d": round(float(np.median(longest)), 1),
                                      "longest_gap_p99_d": round(float(np.percentile(longest, 99)), 1)}
        res[cname] = out
        print(cname, json.dumps(out))
    if args.out:
        json.dump(res, open(args.out, "w"), indent=1)


if __name__ == "__main__":
    main()
