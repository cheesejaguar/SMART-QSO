#!/usr/bin/env python3
"""SMART-QSO controlled link budget, CDR re-issue (SQSO-LBA-001 v3.0).

Computes every table in SQSO-LBA-001 v3.0 §2-§6 from the inputs below and the
demodulator results of fm_demod_sim.py (required C/N0 and the SINAD curve).

Usage:
  python3 fm_demod_sim.py --out fm.json        # ~12 min
  python3 link_budget_cdr.py --fm fm.json [--out lb.json] [--md lb.md]
"""
from __future__ import annotations

import argparse
import json
import math

K_DBM = -198.6          # Boltzmann constant, dBm/K/Hz
RE = 6378.137           # km
F_DOWN = 145.9          # MHz (planning value inside 145.800-146.000)
F_UP = 435.5            # MHz (planning value inside 435.0-437.3, §7)
C_KMS = 299_792.458
ELEVATIONS = (5, 10, 20, 30, 45, 90)

IMPLEMENTATION_LOSS_DB = 1.0   # real clock recovery, filters, deviation tolerance +/-0.5 kHz

# ---------------------------------------------------------------- stations
# downlink receive classes: gain (dBi/dBic), Tsys (K), circular?, Doppler corrected?
RX = {
    "A": dict(name="Telecommand / well-equipped: 2 m cross-Yagi 12 dBic, mast LNA, tracking", g=12.0, t=500.0, circ=True, corr=True),
    "S": dict(name="SatNOGS omni (turnstile/QFH), 0 dBic, LNA + SDR", g=0.0, t=1000.0, circ=True, corr=True),
    "B": dict(name="Portable: handheld 3-element Yagi 5 dBi, FM handheld radio", g=5.0, t=1000.0, circ=False, corr=False),
    "W": dict(name="Handheld radio, quarter-wave whip, -2 dBi (with the operator's body)", g=-2.0, t=1000.0, circ=False, corr=False),
    "D": dict(name="Handheld radio, stock helical 'rubber duck', -8 dBi", g=-8.0, t=1000.0, circ=False, corr=False),
}
# uplink transmit classes: power W, line loss dB, gain, circular?
TX = {
    "TC": dict(name="Telecommand station: 25 W, 70 cm cross-Yagi 14 dBic, 1.5 dB line", p=25.0, l=1.5, g=14.0, circ=True),
    "M": dict(name="Mobile: 50 W, roof vertical 2 dBi at low elevation, 1 dB line", p=50.0, l=1.0, g=2.0, circ=False),
    "BY": dict(name="Handheld 5 W with handheld 7-element 70 cm Yagi, 8 dBi", p=5.0, l=0.3, g=8.0, circ=False),
    "HW": dict(name="Handheld 5 W, whip 0 dBi, -2 dB body", p=5.0, l=0.0, g=-2.0, circ=False),
}
SC = dict(line_loss_db=1.0, rx_tsys_k=1000.0, gains=(0.0, -3.0, -10.0))
POL_LOSS_DB = 3.0       # linear to circular: constant; linear to linear: the average (fades in §5)


def slant_km(el_deg, h_km):
    e = math.radians(el_deg)
    return math.sqrt((RE + h_km) ** 2 - (RE * math.cos(e)) ** 2) - RE * math.sin(e)


def fspl(f_mhz, d_km):
    return 32.45 + 20 * math.log10(f_mhz) + 20 * math.log10(d_km)


def atm(el):
    return 1.0 if el < 20 else 0.5


def dbm(w):
    return 10 * math.log10(w * 1e3)


def n0(t_k):
    return K_DBM + 10 * math.log10(t_k)


def interp(x, xs, ys):
    if x <= xs[0]:
        return ys[0] + (ys[1] - ys[0]) / (xs[1] - xs[0]) * (x - xs[0])
    for i in range(len(xs) - 1):
        if xs[i] <= x <= xs[i + 1]:
            return ys[i] + (ys[i + 1] - ys[i]) * (x - xs[i]) / (xs[i + 1] - xs[i])
    return ys[-1]


def p372_noise():
    """ITU-R P.372-16 §5 (man-made, Fam = c - d log f) and galactic (52 - 23 log f), as antenna temperatures"""
    f = F_DOWN
    out = {}
    for k, (c, d) in {"city": (76.8, 27.7), "residential": (72.5, 27.7), "rural": (67.2, 27.7),
                      "quiet_rural": (53.6, 28.6)}.items():
        fa = c - d * math.log10(f)
        out[k] = {"Fa_dB": round(fa, 1), "T_K": round(290 * 10 ** (fa / 10))}
    fa = 52 - 23 * math.log10(f)
    out["galactic"] = {"Fa_dB": round(fa, 1), "T_K": round(290 * 10 ** (fa / 10))}
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--fm", required=True)
    ap.add_argument("--out", default=None)
    ap.add_argument("--md", default=None)
    args = ap.parse_args()
    fm = json.load(open(args.fm))
    a = fm["afsk"]
    req = {
        "data_corrected": a["beacon_bif15k_no_doppler"]["cn0_frame_90"] + IMPLEMENTATION_LOSS_DB,
        "data_uncorrected": a["beacon_bif15k_doppler3.5k"]["cn0_frame_90"] + IMPLEMENTATION_LOSS_DB,
        "telecommand": a["telecommand_bif15k_residual1k"]["cn0_frame_90"] + IMPLEMENTATION_LOSS_DB,
    }
    sv = fm["sinad"]["voice_bif15k_no_offset"]["points"]
    sv_off = fm["sinad"]["voice_bif15k_offset3.5k"]["points"]
    sx, sy = [p["cn0_dbhz"] for p in sv], [p["sinad_db"] for p in sv]
    sox, soy = [p["cn0_dbhz"] for p in sv_off], [p["sinad_db"] for p in sv_off]

    def sinad(cn0, offset=False):
        xs, ys = (sox, soy) if offset else (sx, sy)
        return max(0.0, interp(cn0, xs, ys))

    md = []
    res = {"required_cn0_dbhz": req, "implementation_loss_db": IMPLEMENTATION_LOSS_DB,
           "sinad_12dB_cn0": fm["sinad"]["voice_bif15k_no_offset"]["cn0_sinad_12"],
           "sinad_12dB_cn0_offset3.5k": fm["sinad"]["voice_bif15k_offset3.5k"]["cn0_sinad_12"],
           "p372": p372_noise()}
    md.append("Required C/N0 (dB-Hz), 90 %% frame success + %.1f dB implementation loss: corrected %.1f, "
              "uncorrected (3.5 kHz) %.1f, telecommand (1 kHz residual) %.1f" % (
                  IMPLEMENTATION_LOSS_DB, req["data_corrected"], req["data_uncorrected"], req["telecommand"]))
    md.append("12 dB SINAD at C/N0 %.1f dB-Hz (%.1f with a 3.5 kHz offset)" % (
        res["sinad_12dB_cn0"], res["sinad_12dB_cn0_offset3.5k"]))
    md.append("P.372 at 145.9 MHz: " + json.dumps(res["p372"]))

    # geometry
    geo = []
    for h in (500.0, 525.0):
        for el in ELEVATIONS:
            d = slant_km(el, h)
            geo.append({"h_km": h, "el": el, "slant_km": round(d), "fspl_145": round(fspl(F_DOWN, d), 1),
                        "fspl_435": round(fspl(F_UP, d), 1)})
    res["geometry"] = geo
    md.append("\n| h | " + " | ".join(f"{e}°" for e in ELEVATIONS) + " |")
    for h in (500.0, 525.0):
        rows = [g for g in geo if g["h_km"] == h]
        md.append(f"| {h:g} km slant | " + " | ".join(f"{g['slant_km']:,} km" for g in rows) + " |")
        md.append(f"| {h:g} km FSPL 145.9 | " + " | ".join(f"{g['fspl_145']}" for g in rows) + " |")
        md.append(f"| {h:g} km FSPL 435.5 | " + " | ".join(f"{g['fspl_435']}" for g in rows) + " |")

    # downlink
    def down_cn0(p_w, g_sc, el, cls, h=500.0):
        r = RX[cls]
        d = slant_km(el, h)
        return dbm(p_w) - SC["line_loss_db"] + g_sc - fspl(F_DOWN, d) - atm(el) - POL_LOSS_DB + r["g"] - n0(r["t"])

    beacon = {}
    md.append("\n**Beacon margin (dB) against the required C/N0**")
    md.append("| Station | P, G_sc | " + " | ".join(f"{e}°" for e in ELEVATIONS) + " |")
    for cls in RX:
        rq = req["data_corrected"] if RX[cls]["corr"] else req["data_uncorrected"]
        for p_w in (0.5, 1.0):
            for g in (-3.0, -10.0):
                row = [round(down_cn0(p_w, g, el, cls) - rq, 1) for el in ELEVATIONS]
                beacon[f"{cls}|{p_w}|{g}"] = row
                md.append(f"| {cls} | {p_w} W, {g:g} dBi | " + " | ".join(f"{v:+.1f}" for v in row) + " |")
    res["beacon_margin"] = beacon
    # 525 km cap, station A
    res["beacon_margin_A_525km_5deg"] = {g: round(down_cn0(0.5, g, 5, "A", 525.0) - req["data_corrected"], 1) for g in (-3.0, -10.0)}
    # noise sensitivity, station A
    sens = {}
    for t_k in (300, 500, 1000, 1500, 2000, 5800):
        r = RX["A"]
        c = dbm(0.5) - 1 - 3 - fspl(F_DOWN, slant_km(5, 500)) - 1 - 3 + r["g"] - n0(t_k)
        sens[t_k] = round(c - req["data_corrected"], 1)
    res["beacon_A_5deg_vs_tsys"] = sens
    md.append("Station A, 0.5 W, -3 dBi, 5°: margin by Tsys " + json.dumps(sens))

    # transponder downlink SINAD
    sin_down = {}
    md.append("\n**Transponder downlink SINAD (dB), 0.5 W, before the uplink's own noise**")
    md.append("| Station | G_sc | " + " | ".join(f"{e}°" for e in ELEVATIONS) + " |")
    for cls in RX:
        for g in (-3.0, -10.0):
            row = [round(sinad(down_cn0(0.5, g, el, cls), offset=not RX[cls]["corr"]), 1) for el in ELEVATIONS]
            sin_down[f"{cls}|{g}"] = row
            md.append(f"| {cls} | {g:g} dBi | " + " | ".join(f"{v:.1f}" for v in row) + " |")
    res["xpdr_down_sinad"] = sin_down

    # uplink
    def up_cn0(cls, g_sc, el, interference_db=0.0, h=500.0):
        t = TX[cls]
        d = slant_km(el, h)
        return (dbm(t["p"]) - t["l"] + t["g"] - fspl(F_UP, d) - atm(el) - POL_LOSS_DB + g_sc
                - (n0(SC["rx_tsys_k"]) + interference_db))

    tc = {}
    md.append("\n**Telecommand margin (dB), station TC, 25 W, against %.1f dB-Hz**" % req["telecommand"])
    md.append("| G_sc, interference | " + " | ".join(f"{e}°" for e in ELEVATIONS) + " |")
    for g in SC["gains"]:
        for i_db in (0.0, 6.0, 10.0):
            row = [round(up_cn0("TC", g, el, i_db) - req["telecommand"], 1) for el in ELEVATIONS]
            tc[f"{g}|{i_db}"] = row
            md.append(f"| {g:g} dBi, +{i_db:g} dB | " + " | ".join(f"{v:+.1f}" for v in row) + " |")
    res["telecommand_margin"] = tc

    up = {}
    md.append("\n**Transponder uplink SINAD at the spacecraft (dB), Doppler corrected to < 2.5 kHz**")
    md.append("| Uplink | G_sc, interference | " + " | ".join(f"{e}°" for e in ELEVATIONS) + " |")
    for cls in ("M", "BY", "HW"):
        for g in (-3.0, -10.0):
            for i_db in (0.0, 6.0):
                row = [round(sinad(up_cn0(cls, g, el, i_db)), 1) for el in ELEVATIONS]
                up[f"{cls}|{g}|{i_db}"] = row
                md.append(f"| {cls} | {g:g} dBi, +{i_db:g} dB | " + " | ".join(f"{v:.1f}" for v in row) + " |")
    res["xpdr_up_sinad"] = up

    # end to end (both stations at the same elevation, nominal -3 dBi both ways, no interference)
    def comb(s1, s2):
        n1 = 1 / max(10 ** (s1 / 10) - 1, 1e-3)
        n2 = 1 / max(10 ** (s2 / 10) - 1, 1e-3)
        return 10 * math.log10(1 + 1 / (n1 + n2))

    e2e = {}
    md.append("\n**End-to-end SINAD (dB): uplink from X, heard by Y, both at the elevation shown, -3 dBi both ways**")
    md.append("| Uplink -> listener | " + " | ".join(f"{e}°" for e in ELEVATIONS) + " |")
    for u in ("M", "BY", "HW"):
        for l in ("B", "W", "D"):
            row = []
            for k, el in enumerate(ELEVATIONS):
                su = up[f"{u}|-3.0|0.0"][k]
                sd = sin_down[f"{l}|-3.0"][k]
                row.append(round(comb(su, sd), 1))
            e2e[f"{u}->{l}"] = row
            md.append(f"| {u} -> {l} | " + " | ".join(f"{v:.1f}" for v in row) + " |")
    res["xpdr_end_to_end_sinad"] = e2e

    # Doppler, analytic: max range rate at the horizon, overhead pass, Earth rotation neglected
    h = 500.0
    v = math.sqrt(398600.4418 / (RE + h))
    nadir_h = math.asin(RE / (RE + h))
    rr = v * math.sin(nadir_h)
    res["doppler"] = {"v_kms": round(v, 3), "range_rate_max_kms": round(rr, 3),
                      "dop_145_khz": round(F_DOWN * 1e3 * rr / C_KMS, 2), "dop_435_khz": round(F_UP * 1e3 * rr / C_KMS, 2),
                      "rate_145_hz_s_overhead": round(F_DOWN * 1e6 / (C_KMS * 1e3) * (v * 1e3) ** 2 / (h * 1e3), 1),
                      "rate_435_hz_s_overhead": round(F_UP * 1e6 / (C_KMS * 1e3) * (v * 1e3) ** 2 / (h * 1e3), 1)}
    md.append("\nDoppler " + json.dumps(res["doppler"]))
    text = "\n".join(md)
    print(text)
    if args.out:
        json.dump(res, open(args.out, "w"), indent=1)
    if args.md:
        open(args.md, "w").write(text + "\n")


if __name__ == "__main__":
    main()
