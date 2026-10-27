#!/usr/bin/env python3
"""Pass geometry, spin fading and ground-segment coverage for SMART-QSO (CDR).

Feeds SQSO-LBA-001 v3.0 §5 (spin fading), SQSO-GND-001 Rev C §4 (coverage,
contact cadence) and SQSO-DB-001 v2.0 (contact time).

Model
  Orbit: circular 475 km (BASELINE Rev E §3 target; --alt for the 465-485 km
  band), i = 97.40 deg (sun-synchronous), LTAN 22:30 (descending node 10:30,
  the Transporter plane; the SIL's default), epoch 2028-11-01 06:00 UTC (the
  planning launch date); RAAN follows the mean Sun (SSO), no J2 short-period
  terms, no drag. Earth rotation by GMST; Sun by the low-precision almanac.
  Eclipse: cylindrical shadow.
  Attitude (BASELINE §8, D9): +Y on the Sun, spin about +Y at W deg/s (8, the
  Rev E baseline; 6, the earlier value, for comparison),
  random initial phase. The +Y error (mean 2.5 deg) is neglected.
  Spacecraft antennas: lambda/4 monopole on a 1U body, modelled as a
  half-wave-dipole-shaped pattern with 0 dBi peak (as filed) and the null
  floored at FLOOR dBi (-20 dBi: a 1U body and wings fill the ideal null; the
  SIL uses -40). Element orientations (ISIS AntS on a +/-Z face deploys its
  tapes in the plane of the face, so a tape lies along +/-X or +/-Y):
     perp   : element along X, perpendicular to the spin axis (the SIL's -Z
              assumption is equivalent: both rotate with the spin)
     para   : element along Y, parallel to the spin axis (no spin modulation;
              the null points at the Sun and away from it)
     cross  : two orthogonal elements, X and Y, fed in quadrature through a
              90 deg hybrid (G = (Gx + Gy)/2)
  Link: SQSO-LBA-001 v3.0 §2: 145.9 MHz, EIRP = P - 1 dB + G; FSPL; 1.0 dB
  below 20 deg / 0.5 dB above; polarization 3 dB to a circular station.
  Frame success from the simulated AFSK/FM BER curve (fm_demod_sim.py,
  results file), each frame evaluated at three points across its time on air.

Usage: python3 pass_geometry_mc.py --fm fm_full.json --days 60 --out geom.json
"""
from __future__ import annotations

import argparse
import json
import math

import numpy as np

RE = 6378.137e3
MU = 3.986004418e14
OMEGA_E = 7.2921150e-5
C = 299_792_458.0
DEG = math.pi / 180

EPOCH_JD = 2462166.75   # 2028-11-01 06:00 UTC (JD 2462166.5 is 2028-11-01 00:00)
ALT_KM = 475.0          # BASELINE Rev E §3 target (465-485 km, cap 485 km)
SPINS = (8.0, 6.0)      # deg/s: the Rev E baseline (D9 as re-tuned by the CDR) and the earlier 6 deg/s


def jd_to_gmst(jd):
    t = (jd - 2451545.0) / 36525.0
    g = 280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * t * t
    return np.mod(g, 360.0) * DEG


def sun_eci(jd):
    n = jd - 2451545.0
    L = np.mod(280.460 + 0.9856474 * n, 360.0) * DEG
    g = np.mod(357.528 + 0.9856003 * n, 360.0) * DEG
    lam = L + (1.915 * np.sin(g) + 0.020 * np.sin(2 * g)) * DEG
    eps = (23.439 - 4e-7 * n) * DEG
    s = np.stack([np.cos(lam), np.cos(eps) * np.sin(lam), np.sin(eps) * np.sin(lam)], -1)
    return s, L   # unit vector, mean longitude (~ RA of the mean Sun)


def orbit_eci(t, alt_m=ALT_KM * 1e3, inc_deg=97.40, ltan_h=22.5, u0=0.0):
    a = RE + alt_m
    n = math.sqrt(MU / a ** 3)
    jd = EPOCH_JD + t / 86400.0
    _, Lmean = sun_eci(jd)
    raan = Lmean + (ltan_h - 12.0) * 15 * DEG
    u = u0 + n * t
    i = inc_deg * DEG
    cu, su, co, so = np.cos(u), np.sin(u), np.cos(raan), np.sin(raan)
    r = a * np.stack([co * cu - so * su * math.cos(i), so * cu + co * su * math.cos(i), su * math.sin(i)], -1)
    v = a * n * np.stack([-co * su - so * cu * math.cos(i), -so * su + co * cu * math.cos(i), cu * math.sin(i)], -1)
    return r, v, jd


def station_eci(lat_deg, lon_deg, jd, alt=0.0):
    lat, lon = lat_deg * DEG, lon_deg * DEG
    # spherical Earth for the station (adequate for elevation to ~0.2 deg)
    rs = (RE + alt)
    th = jd_to_gmst(jd) + lon
    r = rs * np.stack([math.cos(lat) * np.cos(th), math.cos(lat) * np.sin(th), np.full_like(th, math.sin(lat))], -1)
    up = r / rs
    return r, up


def dipole_gain_lin(cos_off, floor_db):
    s2 = np.clip(1 - cos_off ** 2, 1e-12, None)
    f = np.cos(0.5 * math.pi * cos_off) ** 2 / s2
    return np.maximum(f, 10 ** (floor_db / 10))


def fspl_db(f_hz, d_m):
    return 32.45 + 20 * np.log10(f_hz / 1e6) + 20 * np.log10(d_m / 1e3)


class BerCurve:
    def __init__(self, fm_json: str, case: str):
        d = json.load(open(fm_json))["afsk"][case]["points"]
        pts = [(p["cn0_dbhz"], math.log10(p["ber"])) for p in d if p["ber"] > 0]
        # a run of equal values at the bottom is the simulation's floor (one or two errors), not the curve
        keep = [pts[0]]
        for x, y in pts[1:]:
            if y < keep[-1][1] - 1e-9:
                keep.append((x, y))
        self.x = np.array([k[0] for k in keep], float)
        self.y = np.array([k[1] for k in keep], float)
        # extrapolate beyond the last point with the slope of the last three (the waterfall)
        n = min(3, len(self.x))
        self.slope = float(np.polyfit(self.x[-n:], self.y[-n:], 1)[0])

    def ber(self, cn0):
        cn0 = np.asarray(cn0, float)
        y = np.interp(cn0, self.x, self.y, left=math.log10(0.5))
        y = np.where(cn0 > self.x[-1], self.y[-1] + self.slope * (cn0 - self.x[-1]), y)
        return np.minimum(0.5, 10 ** y)


def body_axes(sun, phase):
    """+Y on the Sun; X, Z spin about it. Returns X, Y, Z unit vectors (N,3)"""
    y = sun
    ref = np.array([0.0, 0.0, 1.0])
    x0 = np.cross(y, ref)
    x0 /= np.linalg.norm(x0, axis=-1, keepdims=True)
    z0 = np.cross(x0, y)
    cp, sp = np.cos(phase)[:, None], np.sin(phase)[:, None]
    x = cp * x0 + sp * z0
    z = -sp * x0 + cp * z0
    return x, y, z


def sc_gain_db(option, los_sc_to_gs, X, Y, floor_db):
    gx = dipole_gain_lin(np.sum(los_sc_to_gs * X, -1), floor_db)
    gy = dipole_gain_lin(np.sum(los_sc_to_gs * Y, -1), floor_db)
    if option == "perp":
        g = gx
    elif option == "para":
        g = gy
    elif option == "cross":
        g = 0.5 * (gx + gy)
    else:
        raise ValueError(option)
    return 10 * np.log10(g)


STATIONS = {
    "SantaClara_CA": (37.349, -121.938),
    "Fairbanks_AK": (64.860, -147.850),
    "Blacksburg_VA": (37.229, -80.414),
    "Honolulu_HI": (21.300, -157.817),
    "Boulder_CO": (40.015, -105.270),
    "SanDiego_CA": (32.880, -117.234),
}


def passes_for(t, r_sc, jd, lat, lon, min_el_deg):
    rs, up = station_eci(lat, lon, jd)
    d = r_sc - rs
    rng = np.linalg.norm(d, axis=-1)
    el = np.arcsin(np.sum(d * up, -1) / rng) / DEG
    vis = el >= min_el_deg
    edges = np.diff(vis.astype(int))
    starts = list(np.where(edges == 1)[0] + 1)
    ends = list(np.where(edges == -1)[0] + 1)
    if vis[0]:
        starts = [0] + starts
    if vis[-1]:
        ends = ends + [len(vis)]
    return list(zip(starts, ends)), el, rng, d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fm", required=True)
    ap.add_argument("--days", type=float, default=60.0)
    ap.add_argument("--out", default="geom.json")
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--alt", type=float, default=ALT_KM, help="circular altitude, km")
    args = ap.parse_args()
    rng_ = np.random.default_rng(args.seed)
    dt = 1.0
    t = np.arange(0.0, args.days * 86400.0, dt)
    r_sc, v_sc, jd = orbit_eci(t, alt_m=args.alt * 1e3)
    sun, _ = sun_eci(jd)
    # eclipse (cylindrical)
    proj = np.sum(r_sc * sun, -1)
    perp = np.linalg.norm(r_sc - proj[:, None] * sun, axis=-1)
    eclipse = (proj < 0) & (perp < RE)
    out: dict = {"days": args.days, "orbit": {"alt_km": args.alt, "inc_deg": 97.4, "ltan_h": 22.5,
                                               "epoch": "2028-11-01T06:00Z", "spins_deg_s": list(SPINS)},
                 "eclipse_fraction": float(eclipse.mean())}

    # ---------------- ground coverage ----------------
    cov = {}
    vis_masks = {}
    for name, (la, lo) in STATIONS.items():
        cov[name] = {}
        for mel in (0.0, 5.0, 10.0, 20.0, 25.0):
            ps, el, _, _ = passes_for(t, r_sc, jd, la, lo, mel)
            dur = np.array([(e - s) * dt for s, e in ps])
            maxel = np.array([el[s:e].max() for s, e in ps]) if ps else np.array([])
            vis = el >= mel
            if mel == 5.0:
                vis_masks[name] = vis
                gaps = []
                prev_end = None
                for s, e in ps:
                    if prev_end is not None:
                        gaps.append((s - prev_end) * dt / 3600)
                    prev_end = e
                cov[name]["gap_h_max_5deg"] = float(max(gaps))
                # passes by local hour
            cov[name][f"el{int(mel)}"] = {
                "passes_per_day": len(ps) / args.days,
                "mean_duration_min": float(dur.mean() / 60) if len(dur) else 0,
                "minutes_per_day": float(dur.sum() / 60 / args.days),
                "median_max_el_deg": float(np.median(maxel)) if len(maxel) else 0,
            }
        print(name, json.dumps(cov[name]["el5"]), "max gap h", round(cov[name]["gap_h_max_5deg"], 1), flush=True)
    # pairs with Santa Clara
    pairs = {}
    for other in STATIONS:
        if other == "SantaClara_CA":
            continue
        u = vis_masks["SantaClara_CA"] | vis_masks[other]
        # gaps in the union
        idx = np.where(np.diff(u.astype(int)) != 0)[0]
        runs = np.split(u, idx + 1)
        gap = max((len(r) * dt / 3600 for r in runs if not r[0]), default=0)
        overlap = float((vis_masks["SantaClara_CA"] & vis_masks[other]).sum() / max(1, vis_masks[other].sum()))
        la1, lo1 = STATIONS["SantaClara_CA"]
        la2, lo2 = STATIONS[other]
        dist = 6371 * math.acos(min(1, math.sin(la1 * DEG) * math.sin(la2 * DEG) + math.cos(la1 * DEG) * math.cos(la2 * DEG) * math.cos((lo1 - lo2) * DEG)))
        pairs[other] = {"distance_km": dist, "union_minutes_per_day": float(u.sum() * dt / 60 / args.days),
                        "union_max_gap_h": gap, "share_of_other_visible_also_from_SC": overlap}
        print("pair SC+", other, json.dumps(pairs[other]), flush=True)
    out["coverage"] = cov
    out["pairs_with_SantaClara"] = pairs

    # ---------------- spin fading, Santa Clara, three receive-station classes ----------------
    la, lo = STATIONS["SantaClara_CA"]
    ps, el, rng, d = passes_for(t, r_sc, jd, la, lo, 5.0)
    los = -d / rng[:, None]      # spacecraft to station
    rs_, up_ = station_eci(la, lo, jd)
    frame_on_air_s = 1.2         # 110-byte information field (BEACON_SPEC §3) + 300 ms TXDELAY
    frame_bits = 1060
    # class: (ground gain dBi, Tsys K, circular?, BER curve case)
    classes = {
        "A_crossyagi_12dBic_500K": (12.0, 500.0, True, "beacon_bif15k_no_doppler"),
        "B_yagi_5dBi_linear_1000K": (5.0, 1000.0, False, "beacon_bif15k_doppler3.5k"),
        "S_satnogs_omni_0dBic_1000K": (0.0, 1000.0, True, "beacon_bif15k_no_doppler"),
        "H_handheld_whip_-2dBi_linear_1000K": (-2.0, 1000.0, False, "beacon_bif15k_doppler3.5k"),
    }
    curves = {k: BerCurve(args.fm, v[3]) for k, v in classes.items()}
    idx_all = np.concatenate([np.arange(s, e) for s, e in ps])
    pass_id = np.concatenate([np.full(e - s, k) for k, (s, e) in enumerate(ps)])
    los_a = los[idx_all]
    up_a = up_[idx_all]
    h = np.cross(up_a, -los_a)                      # ground Yagi elements: horizontal, across the line of sight
    h /= np.linalg.norm(h, axis=-1, keepdims=True)
    fade = {}
    for cname, (gg, tsys, circ, _) in classes.items():
        for option in ("perp", "para", "cross"):
            for spin in SPINS:
                for interval in (60.0, 67.0, -60.0, 120.0):
                    # -60: 60 s with a uniform +/-5 s jitter on each interval
                    if option != "perp" and interval in (67.0, -60.0):
                        continue
                    if cname.startswith("H") and interval != 60.0:
                        continue
                    floor_db = -20.0
                    tag = f"{cname}|{option}|spin{spin:g}|int{interval:g}"
                    phase = rng_.uniform(0, 2 * math.pi) + spin * DEG * t[idx_all]
                    X, Y, Z = body_axes(sun[idx_all], phase)
                    gx = dipole_gain_lin(np.sum(los_a * X, -1), floor_db)
                    gy = dipole_gain_lin(np.sum(los_a * Y, -1), floor_db)
                    if circ:
                        g_lin = {"perp": gx, "para": gy, "cross": 0.5 * (gx + gy)}[option] * 0.5   # 3 dB to circular
                    else:
                        # Faraday rotation: random start, drifting 0.5-3 rad over a pass
                        om0 = rng_.uniform(0, math.pi, len(ps))
                        omr = rng_.uniform(0.5, 3.0, len(ps)) / 600.0
                        t0 = np.array([t[s] for s, e in ps])
                        om = om0[pass_id] + omr[pass_id] * (t[idx_all] - t0[pass_id])
                        def proj(ax):
                            e = ax - np.sum(ax * los_a, -1, keepdims=True) * los_a
                            n = np.linalg.norm(e, axis=-1)
                            e = e / np.maximum(n, 1e-9)[:, None]
                            # rotate e about los by om (Rodrigues; e is perpendicular to los)
                            e = e * np.cos(om)[:, None] + np.cross(los_a, e) * np.sin(om)[:, None]
                            return np.sum(e * h, -1) ** 2
                        px, py = proj(X), proj(Y)
                        g_lin = {"perp": gx * px, "para": gy * py, "cross": 0.5 * (gx * px + gy * py)}[option]
                    g_db = 10 * np.log10(np.maximum(g_lin, 1e-6))
                    atm = np.where(el[idx_all] < 20, 1.0, 0.5)
                    cn0 = (10 * math.log10(0.5e3) - 1.0 + g_db - fspl_db(145.9e6, rng[idx_all]) - atm
                           + gg - (-198.6 + 10 * math.log10(tsys)))
                    cn0_full = np.full(len(t), np.nan)
                    cn0_full[idx_all] = cn0
                    heard_pass = 0.0
                    frames = 0
                    frames_ok = 0.0
                    iv = abs(interval)
                    if interval > 0:
                        tb0 = rng_.uniform(0, iv)
                        sched = tb0 + iv * np.arange(0, int(t[-1] / iv) + 2)
                    else:
                        sched = np.cumsum(iv + rng_.uniform(-5.0, 5.0, int(t[-1] / (iv - 5)) + 2))
                    for s, e in ps:
                        starts = sched[(sched >= t[s]) & (sched + frame_on_air_s <= t[e - 1])]
                        p_none = 1.0
                        for st in starts:
                            pts = np.array([st + 0.2, st + 0.6, st + 1.0])
                            c = np.interp(pts, t[s:e], cn0_full[s:e])
                            b = curves[cname].ber(c)
                            pf = float(np.prod((1 - b) ** (frame_bits / 3)))
                            frames += 1
                            frames_ok += pf
                            p_none *= (1 - pf)
                        heard_pass += 1 - p_none
                    fade[tag] = {"passes": len(ps), "p_pass_heard": heard_pass / len(ps),
                                 "frames": frames, "frame_success": frames_ok / max(1, frames),
                                 "cn0_p10": float(np.percentile(cn0, 10)), "cn0_median": float(np.median(cn0)),
                                 "time_frac_voice_12dB_SINAD": float(np.mean(cn0 >= 48.1)),
                                 "time_frac_voice_20dB_SINAD": float(np.mean(cn0 >= 50.8)),
                                 "gain_p10_dBi": float(np.percentile(g_db, 10)),
                                 "gain_p50_dBi": float(np.percentile(g_db, 50))}
                    print(tag, json.dumps(fade[tag]), flush=True)
    out["spin_fading_SantaClara_0.5W"] = fade

    # null-duration statistics during passes (time with g < -10 dBi), perp vs para, by spin rate
    stats = {}
    idx_all = np.concatenate([np.arange(s, e) for s, e in ps])
    for option in ("perp", "para", "cross"):
        for spin in SPINS:
            phase = rng_.uniform(0, 2 * math.pi) + spin * DEG * t[idx_all]
            X, Y, Z = body_axes(sun[idx_all], phase)
            g = sc_gain_db(option, los[idx_all], X, Y, -20.0)
            # fade episodes (g < -10 dBi) lengths
            low = g < -10
            ch = np.diff(np.concatenate([[0], low.astype(int), [0]]))
            lens = (np.where(ch == -1)[0] - np.where(ch == 1)[0]) * dt
            elb = el[idx_all]
            bins = {}
            for lo_, hi_ in ((5, 10), (10, 20), (20, 30), (30, 45), (45, 60), (60, 91)):
                m = (elb >= lo_) & (elb < hi_)
                if m.any():
                    bins[f"{lo_}-{hi_}"] = {"p50": round(float(np.percentile(g[m], 50)), 1),
                                            "p10": round(float(np.percentile(g[m], 10)), 1),
                                            "share": round(float(m.mean()), 3)}
            stats[f"{option}_spin{spin:g}"] = {
                "gain_by_elevation_dBi": bins,
                "time_frac_below_-3dBi": float(np.mean(g < -3)),
                "time_frac_below_-6dBi": float(np.mean(g < -6)),
                "time_frac_below_-10dBi": float(np.mean(low)),
                "fade_episodes_per_pass": float(len(lens) / len(ps)),
                "fade_len_median_s": float(np.median(lens)) if len(lens) else 0.0,
                "fade_len_p95_s": float(np.percentile(lens, 95)) if len(lens) else 0.0,
            }
            print("gain stats", option, spin, json.dumps(stats[f"{option}_spin{spin:g}"]), flush=True)
    out["gain_statistics_SantaClara"] = stats

    # Doppler extremes (Santa Clara)
    rr = np.sum(d * (v_sc - np.cross([0, 0, OMEGA_E], station_eci(la, lo, jd)[0])), -1) / rng
    vis = el >= 0
    out["doppler"] = {
        "max_range_rate_m_s": float(np.abs(rr[vis]).max()),
        "max_doppler_145.9_hz": float(np.abs(rr[vis]).max() * 145.9e6 / C),
        "max_doppler_435.5_hz": float(np.abs(rr[vis]).max() * 435.5e6 / C),
        "max_rate_145.9_hz_s": float(np.abs(np.diff(rr))[vis[1:] & vis[:-1]].max() / dt * 145.9e6 / C),
        "max_rate_435.5_hz_s": float(np.abs(np.diff(rr))[vis[1:] & vis[:-1]].max() / dt * 435.5e6 / C),
    }
    print("doppler", json.dumps(out["doppler"]))
    json.dump(out, open(args.out, "w"), indent=1)


if __name__ == "__main__":
    main()
