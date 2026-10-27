#!/usr/bin/env python3
"""
LEOP and commissioning ground-station passes for the baseline orbit (CDR, launch and ConOps review).

Orbit: circular sun-synchronous, 475 km (the BASELINE_PARAMETERS Rev E §3 target), LTAN 22:30, J2 secular
drift of the node and of the argument of latitude. No drag over 14 days (the decay is about 0.6 km in 14 days in
the SIL), no SGP4: the purpose is pass statistics, not pointing.

For each deployment case (the UTC time of separation and the argument of latitude at separation, swept on a grid,
because the integrator's deployment slot is not known until the mission ICD) the script finds, for each candidate
telecommand station:

  * the first pass that starts after the 45-minute RF silence (CDS Rev 14.1 §2.4.5) with a maximum elevation of at
    least 5 degrees (the SIL's "beacon heard" mask) and of at least 15 degrees (the SIL's commanding mask,
    `contact_min_elev_deg`);
  * passes a day, minutes a day and the longest gap between commanding passes over the first 14 days;
  * the local time of the commanding passes (who has to be awake).

It also gives, for each deployment case, whether separation and the 30-minute burn happen in eclipse (cylindrical
Earth shadow) and the beta angle over the launch year.

Stations: Santa Clara University and NASA Ames are the D1 prospective partners (BASELINE §1, nothing agreed);
Fairbanks AK and Morehead KY are illustrative geographically separated sites taken from the SIL's network
(`sqso-sil/src/env/ground.rs`), not partners.

    python3 leop_passes.py --out <dir>          # writes passes_summary.json and passes_summary.md
    python3 leop_passes.py --out <dir> --quick  # a coarser grid, for a smoke test

Only numpy is needed.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timedelta, timezone
from pathlib import Path

import numpy as np

MU = 3.986004418e14          # m^3/s^2
RE = 6378137.0               # m (WGS-84 equatorial)
J2 = 1.08262668e-3
F_WGS = 1.0 / 298.257223563
OMEGA_E = 7.2921150e-5       # rad/s
OMEGA_SUN = 2.0 * math.pi / (365.2422 * 86400.0)   # rad/s, mean motion of the Sun

ALT_M = 475e3                # BASELINE Rev E §3 target (465-485 km, cap 485 km)
LAUNCH = datetime(2028, 11, 1, tzinfo=timezone.utc)   # BASELINE Rev E §3 planning date (about 2028-11)
LTAN_H = 22.5
SILENCE_S = 45 * 60          # CDS 2.4.5
BURN_S = 30 * 60             # CDS 2.4.4
DAYS = 14.0
DT = 10.0                    # s

STATIONS = [
    # name, lat, lon, alt (m), role
    ("Santa Clara University CA", 37.349, -121.938, 30.0, "prospective partner (D1)"),
    ("NASA Ames CA", 37.415, -122.064, 10.0, "prospective partner (D1)"),
    ("Fairbanks AK", 64.860, -147.850, 150.0, "illustrative separated site"),
    ("Morehead KY", 38.190, -83.440, 300.0, "illustrative separated site"),
]


def jd(dt: datetime) -> float:
    return dt.timestamp() / 86400.0 + 2440587.5


def gmst_rad(jd_ut: np.ndarray) -> np.ndarray:
    t = (jd_ut - 2451545.0) / 36525.0
    g = 280.46061837 + 360.98564736629 * (jd_ut - 2451545.0) + 0.000387933 * t * t
    return np.radians(np.mod(g, 360.0))


def sun_eci(jd_ut: np.ndarray) -> np.ndarray:
    """Unit vector to the Sun, low-precision (Astronomical Almanac), mean equator and equinox of date."""
    n = jd_ut - 2451545.0
    L = np.radians(np.mod(280.460 + 0.9856474 * n, 360.0))
    g = np.radians(np.mod(357.528 + 0.9856003 * n, 360.0))
    lam = L + np.radians(1.915) * np.sin(g) + np.radians(0.020) * np.sin(2 * g)
    eps = np.radians(23.439 - 0.0000004 * n)
    return np.stack([np.cos(lam), np.cos(eps) * np.sin(lam), np.sin(eps) * np.sin(lam)], axis=-1)


def sun_ra(jd_ut: float) -> float:
    s = sun_eci(np.array([jd_ut]))[0]
    return math.atan2(s[1], s[0])


def sso_inclination(a: float) -> float:
    n = math.sqrt(MU / a**3)
    cos_i = -OMEGA_SUN / (1.5 * n * J2 * (RE / a) ** 2)
    return math.acos(cos_i)


def station_ecef(lat_deg: float, lon_deg: float, alt: float) -> np.ndarray:
    lat, lon = math.radians(lat_deg), math.radians(lon_deg)
    e2 = F_WGS * (2 - F_WGS)
    N = RE / math.sqrt(1 - e2 * math.sin(lat) ** 2)
    return np.array([(N + alt) * math.cos(lat) * math.cos(lon),
                     (N + alt) * math.cos(lat) * math.sin(lon),
                     (N * (1 - e2) + alt) * math.sin(lat)])


def propagate(t: np.ndarray, epoch_jd: float, u0: float, raan0: float, a: float, inc: float):
    """ECI positions (m) of a circular orbit with J2 secular rates."""
    n = math.sqrt(MU / a**3)
    k = 1.5 * J2 * (RE / a) ** 2 * n
    raan_dot = -k * math.cos(inc)
    # argument-of-latitude rate of a circular orbit: the J2 mean-anomaly rate plus the perigee rate
    m_dot = n + k * (1 - 1.5 * math.sin(inc) ** 2)
    w_dot = k * (2 - 2.5 * math.sin(inc) ** 2)
    u_dot = m_dot + w_dot
    u = u0 + u_dot * t
    raan = raan0 + raan_dot * t
    cu, su = np.cos(u), np.sin(u)
    cO, sO = np.cos(raan), np.sin(raan)
    ci, si = math.cos(inc), math.sin(inc)
    x = a * (cO * cu - sO * su * ci)
    y = a * (sO * cu + cO * su * ci)
    z = a * (su * si)
    return np.stack([x, y, z], axis=-1)


def eci_to_ecef(r: np.ndarray, jd_ut: np.ndarray) -> np.ndarray:
    th = gmst_rad(jd_ut)
    c, s = np.cos(th), np.sin(th)
    return np.stack([c * r[:, 0] + s * r[:, 1], -s * r[:, 0] + c * r[:, 1], r[:, 2]], axis=-1)


def elevation_deg(r_ecef: np.ndarray, st: np.ndarray, lat_deg: float, lon_deg: float) -> np.ndarray:
    lat, lon = math.radians(lat_deg), math.radians(lon_deg)
    up = np.array([math.cos(lat) * math.cos(lon), math.cos(lat) * math.sin(lon), math.sin(lat)])
    d = r_ecef - st
    rng = np.linalg.norm(d, axis=1)
    return np.degrees(np.arcsin(d @ up / rng))


def passes(t: np.ndarray, el: np.ndarray, mask0: float = 0.0):
    """Passes above the horizon (0 deg): list of (aos, los, max_el, t_max)."""
    above = el > mask0
    out = []
    if not above.any():
        return out
    edges = np.diff(above.astype(int))
    starts = list(np.where(edges == 1)[0] + 1)
    ends = list(np.where(edges == -1)[0] + 1)
    if above[0]:
        starts = [0] + starts
    if above[-1]:
        ends = ends + [len(el)]
    for s, e in zip(starts, ends):
        seg = el[s:e]
        i = int(np.argmax(seg))
        out.append((t[s], t[e - 1], float(seg[i]), t[s + i]))
    return out


def in_shadow(r_eci: np.ndarray, s_hat: np.ndarray) -> np.ndarray:
    """Cylindrical shadow (umbra and penumbra together, within a few seconds of the conical model)."""
    proj = np.sum(r_eci * s_hat, axis=1)
    perp = np.linalg.norm(r_eci - proj[:, None] * s_hat, axis=1)
    return (proj < 0) & (perp < RE)


def beta_deg(day: datetime, raan_offset_h: float, inc: float) -> float:
    j = jd(day)
    raan = sun_ra(j) + math.radians((raan_offset_h - 12.0) * 15.0)
    h = np.array([math.sin(raan) * math.sin(inc), -math.cos(raan) * math.sin(inc), math.cos(inc)])
    s = sun_eci(np.array([j]))[0]
    return math.degrees(math.asin(float(h @ s)))


def run(out: Path, quick: bool) -> dict:
    a = RE + ALT_M
    inc = sso_inclination(a)
    period = 2 * math.pi * math.sqrt(a**3 / MU)
    date0 = LAUNCH
    hours = np.arange(0.0, 24.0, 2.0 if quick else 0.5)
    us = np.radians(np.arange(0.0, 360.0, 90.0 if quick else 30.0))
    t = np.arange(0.0, DAYS * 86400.0 + DT, DT)
    st_ecef = [station_ecef(la, lo, al) for _, la, lo, al, _ in STATIONS]

    per_station = {s[0]: {"first5_h": [], "first15_h": [], "p5_day": [], "p15_day": [], "min15_day": [],
                          "max_gap15_h": [], "local_h15": [], "dur5_min": []} for s in STATIONS}
    union_first15 = []
    union_sets = {"SCU + Morehead KY": (0, 3), "SCU + Fairbanks AK": (0, 2)}
    union_other = {k: [] for k in union_sets}
    first_day_p15 = {s[0]: [] for s in STATIONS}
    eclipse_sep, eclipse_burn, dark_after_sep_min = [], [], []

    for hh in hours:
        epoch = date0 + timedelta(hours=float(hh))
        j0 = jd(epoch)
        raan0 = sun_ra(j0) + math.radians((LTAN_H - 12.0) * 15.0)
        jt = j0 + t / 86400.0
        s_hat = sun_eci(jt)
        for u0 in us:
            r = propagate(t, j0, u0, raan0, a, inc)
            shadow = in_shadow(r, s_hat)
            eclipse_sep.append(bool(shadow[0]))
            eclipse_burn.append(bool(shadow[int(BURN_S / DT)]))
            if shadow[0]:
                lit = np.where(~shadow)[0]
                dark_after_sep_min.append(float(t[lit[0]] / 60.0) if lit.size else float("nan"))
            else:
                dark_after_sep_min.append(0.0)
            r_ecef = eci_to_ecef(r, jt)
            firsts = []
            for (name, la, lo, _, _), st in zip(STATIONS, st_ecef):
                el = elevation_deg(r_ecef, st, la, lo)
                ps = passes(t, el)
                after = [p for p in ps if p[0] >= SILENCE_S or p[1] >= SILENCE_S + 120]
                p5 = [p for p in after if p[2] >= 5.0]
                p15 = [p for p in after if p[2] >= 15.0]
                d = per_station[name]
                d["first5_h"].append(p5[0][3] / 3600.0 if p5 else float("nan"))
                d["first15_h"].append(p15[0][3] / 3600.0 if p15 else float("nan"))
                d["p5_day"].append(len(p5) / DAYS)
                d["p15_day"].append(len(p15) / DAYS)
                # minutes above 5 deg on the >= 15 deg passes (usable commanding time)
                above5 = el > 5.0
                mins = 0.0
                for p in p15:
                    sel = (t >= p[0]) & (t <= p[1])
                    mins += float(np.count_nonzero(above5 & sel)) * DT / 60.0
                d["min15_day"].append(mins / DAYS)
                for p in p5:
                    sel = (t >= p[0]) & (t <= p[1])
                    d["dur5_min"].append(float(np.count_nonzero(above5 & sel)) * DT / 60.0)
                tm = [p[3] for p in p15]
                gaps = np.diff([SILENCE_S] + tm) / 3600.0 if tm else np.array([DAYS * 24])
                d["max_gap15_h"].append(float(np.max(gaps)))
                for p in p15:
                    utc = epoch + timedelta(seconds=float(p[3]))
                    d["local_h15"].append(((utc.hour + utc.minute / 60.0) + lo / 15.0) % 24.0)
                firsts.append(p15[0][3] / 3600.0 if p15 else float("nan"))
                first_day_p15[name].append(sum(1 for p in p15 if p[3] < 86400.0))
            union_first15.append(float(np.nanmin(firsts[:2])))   # the two Bay Area stations together
            for k, (i, j) in union_sets.items():
                union_other[k].append(float(np.nanmin([firsts[i], firsts[j]])))

    def stats(x):
        x = np.array(x, dtype=float)
        x = x[~np.isnan(x)]
        return {"n": int(x.size), "min": round(float(x.min()), 2), "median": round(float(np.median(x)), 2),
                "p90": round(float(np.percentile(x, 90)), 2), "max": round(float(x.max()), 2)}

    res = {
        "orbit": {"altitude_km": ALT_M / 1e3, "ltan_h": LTAN_H, "inclination_deg": round(math.degrees(inc), 3),
                  "period_min": round(period / 60.0, 2),
                  "epoch": f"{LAUNCH:%Y-%m-%d} (deployment day swept 00-24 UTC)"},
        "cases": len(hours) * len(us),
        "eclipse": {
            "share_separated_in_eclipse": round(float(np.mean(eclipse_sep)), 3),
            "share_burn_at_30min_in_eclipse": round(float(np.mean(eclipse_burn)), 3),
            "dark_after_separation_min": stats([x for x in dark_after_sep_min if x > 0]),
        },
        "beta_deg_first_year": {},
        "stations": {},
        "bay_area_pair_first_command_pass_h": stats(union_first15),
        "separated_pairs_first_command_pass_h": {k: stats(v) for k, v in union_other.items()},
    }
    for k in range(12):   # the 12 months from the launch month
        y, m = LAUNCH.year + (LAUNCH.month - 1 + k) // 12, (LAUNCH.month - 1 + k) % 12 + 1
        day = datetime(y, m, 15, tzinfo=timezone.utc)
        res["beta_deg_first_year"][day.strftime("%Y-%m-%d")] = round(beta_deg(day, LTAN_H, inc), 1)
    for name, la, lo, al, role in STATIONS:
        d = per_station[name]
        lh = np.array(d["local_h15"])
        res["stations"][name] = {
            "role": role, "lat": la, "lon": lo,
            "first_beacon_pass_5deg_h": stats(d["first5_h"]),
            "first_command_pass_15deg_h": stats(d["first15_h"]),
            "passes_per_day_5deg": round(float(np.mean(d["p5_day"])), 2),
            "passes_per_day_15deg": round(float(np.mean(d["p15_day"])), 2),
            "command_passes_in_first_24h": stats(first_day_p15[name]),
            "command_minutes_per_day": round(float(np.mean(d["min15_day"])), 1),
            "pass_duration_above_5deg_min": stats(d["dur5_min"]),
            "longest_gap_between_command_passes_h": stats(d["max_gap15_h"]),
            "command_passes_local_time_share_07_23h": round(float(np.mean((lh >= 7) & (lh < 23))), 3) if lh.size else None,
            "command_pass_local_time_clusters_h": [round(float(np.percentile(lh[lh < 16], q)), 1) for q in (5, 50, 95)]
            + [round(float(np.percentile(lh[lh >= 16], q)), 1) for q in (5, 50, 95)] if lh.size else None,
        }
    out.mkdir(parents=True, exist_ok=True)
    (out / "passes_summary.json").write_text(json.dumps(res, indent=2))
    lines = ["# LEOP passes (generated by leop_passes.py)", "", f"Orbit: {res['orbit']}", f"Cases: {res['cases']}", "",
             f"Eclipse: {res['eclipse']}", f"Beta, first year: {res['beta_deg_first_year']}",
             f"Bay Area pair, first commanding pass (h after separation): {res['bay_area_pair_first_command_pass_h']}",
             f"Separated pairs, first commanding pass (h): {res['separated_pairs_first_command_pass_h']}", ""]
    for k, v in res["stations"].items():
        lines.append(f"## {k}")
        for kk, vv in v.items():
            lines.append(f"- {kk}: {vv}")
        lines.append("")
    (out / "passes_summary.md").write_text("\n".join(lines))
    return res


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[1])
    ap.add_argument("--out", type=Path, required=True)
    ap.add_argument("--quick", action="store_true")
    args = ap.parse_args()
    res = run(args.out, args.quick)
    print(json.dumps(res, indent=2))


if __name__ == "__main__":
    main()
