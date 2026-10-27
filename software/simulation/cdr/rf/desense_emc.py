#!/usr/bin/env python3
"""Receiver desense and RF-chain EMC for the SMART-QSO U/v transponder (CDR).

Feeds SQSO-LBA-001 v3.0 §7 and SQSO-RF-001 v2.0 §6.

1. Harmonics of the 2 m downlink against the 70 cm uplink band; the uplink
   frequencies the 3rd harmonic excludes.
2. The output low-pass filter in SQSO-RF-TX-001 §7.3 (component values as
   written) analysed by ABCD matrices, ideal and with finite inductor Q, and a
   correctly scaled 7th-order Chebyshev for comparison.
3. Full-duplex desense budget at the uplink frequency: the transmitter's
   3rd-harmonic phase-noise skirt and its wideband noise floor, coupled through
   the antenna isolation, against the receiver noise density.
4. Receive-filter rejection at the downlink frequency needed to keep the
   receiver front end linear (blocking, and receiver-generated 3rd harmonic).

Every input is stated in INPUTS; the outputs are printed and written as JSON.
Usage: python3 desense_emc.py [--out desense.json]
"""
from __future__ import annotations

import argparse
import json
import math

import numpy as np

K_DBM = -198.6

INPUTS = {
    "f_down_mhz": 145.9,            # planning value inside 145.800-146.000
    "f_up_mhz": 435.5,              # planning value inside 435-438
    "p_tx_dbm": 30.0,               # 1 W, the filed maximum (worst case)
    "rx_tsys_k": 1000.0,            # SQSO-LBA-001 §2.7
    "rx_noise_bw_hz": 15e3,
    "h3_suppression_dbc": 60.0,     # BASELINE §7 target, at the transmitter port
    "antenna_isolation_db_437": 15.0,   # 2 m element (3 lambda/4 at 437 MHz) to 70 cm element on one 1U face [assumption]
    "antenna_isolation_db_146": 25.0,   # 2 m element to the off-resonant 70 cm element at 146 MHz [assumption]
    # transmitter single-sideband phase noise of the fundamental, dBc/Hz [assumption:
    # a TCXO-referenced integer-N PLL with a 5 kHz loop, typical of the parts in SQSO-RF-TX-001]
    "pn_fund_dbc_hz": {1e3: -85.0, 1e4: -100.0, 1e5: -118.0, 1e6: -135.0, 1e7: -145.0},
    "tx_far_noise_floor_dbc_hz": -150.0,   # PA output noise far from the carrier, before the LPF
    "lna_iip3_dbm": 0.0,            # an example receiver front end (SA818-class module or discrete LNA)
    "rx_blocking_dbm_1db": -25.0,   # 1 dB-desense blocking level far off channel [assumption]
}


def harmonics():
    f1, f2 = 145.800, 146.000
    out = {}
    for n in (2, 3, 4, 5):
        out[n] = (n * f1, n * f2)
    # FM transponder emission tripled: deviation x3 (+/-15 kHz) with 3 kHz audio: Carson 2(15+3) = 36 kHz
    h3_bw_khz = 2 * (3 * 5.0 + 3.0)
    return out, h3_bw_khz


def abcd_series(z):
    return np.array([[1, z], [0, 1]], complex)


def abcd_shunt(y):
    return np.array([[1, 0], [y, 1]], complex)


def ladder_s21_db(f_hz, elements, z0=50.0, q_l=None, q_c=None):
    """elements: list of ('L', H) series and ('C', F) shunt in order"""
    w = 2 * math.pi * f_hz
    m = np.eye(2, dtype=complex)
    for kind, v in elements:
        if kind == "L":
            r = (w * v / q_l) if q_l else 0.0
            m = m @ abcd_series(r + 1j * w * v)
        else:
            g = (w * v / q_c) if q_c else 0.0
            m = m @ abcd_shunt(g + 1j * w * v)
    a, b, c, d = m[0, 0], m[0, 1], m[1, 0], m[1, 1]
    s21 = 2 / (a + b / z0 + c * z0 + d)
    s11 = (a + b / z0 - c * z0 - d) / (a + b / z0 + c * z0 + d)
    return 20 * math.log10(abs(s21)), 20 * math.log10(max(abs(s11), 1e-12))


def chebyshev_g(n, ripple_db):
    beta = math.log(1 / math.tanh(ripple_db / 17.37))
    gam = math.sinh(beta / (2 * n))
    a = [math.sin((2 * k - 1) * math.pi / (2 * n)) for k in range(1, n + 1)]
    b = [gam ** 2 + math.sin(k * math.pi / n) ** 2 for k in range(1, n + 1)]
    g = [2 * a[0] / gam]
    for k in range(2, n + 1):
        g.append(4 * a[k - 2] * a[k - 1] / (b[k - 2] * g[-1]))
    return g


def cheb_ladder(n, ripple_db, fc_hz, z0=50.0):
    g = chebyshev_g(n, ripple_db)
    wc = 2 * math.pi * fc_hz
    el = []
    for k, gk in enumerate(g):
        if k % 2 == 0:
            el.append(("L", gk * z0 / wc))
        else:
            el.append(("C", gk / (z0 * wc)))
    return el


def pn_at(offset_hz, table):
    xs = sorted(table)
    lx = [math.log10(x) for x in xs]
    ly = [table[x] for x in xs]
    lo = math.log10(max(offset_hz, xs[0]))
    if lo >= lx[-1]:
        return ly[-1]
    return float(np.interp(lo, lx, ly))


def db_add(*vals_db):
    return 10 * math.log10(sum(10 ** (v / 10) for v in vals_db))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=None)
    args = ap.parse_args()
    p = INPUTS
    res = {"inputs": {k: (v if not isinstance(v, dict) else {str(a): b for a, b in v.items()}) for k, v in p.items()}}

    harm, h3bw = harmonics()
    res["harmonics_mhz"] = {str(n): v for n, v in harm.items()}
    res["h3_occupied_bw_khz"] = h3bw
    # Uplink exclusion: 3 f_down +/- (h3 bw/2 + rx IF/2 + 25 kHz guard)
    excl = h3bw / 2 + 7.5 + 25.0
    res["uplink_exclusion_khz_each_side_of_3fd"] = excl
    res["uplink_band_free_of_3fd_any_downlink"] = [435.0, round(437.4 - excl / 1000, 4)]
    print("3rd harmonic of 145.800-146.000 MHz falls on", harm[3], "MHz; occupied", h3bw, "kHz")
    print("uplink keep-out +/-", excl, "kHz around 3 f_down; for ANY downlink in the segment the uplink must be in",
          res["uplink_band_free_of_3fd_any_downlink"], "MHz")

    # LPF as written in SQSO-RF-TX-001 §7.3: L1 47n, C1 33p, L2 82n, C2 47p, L3 82n, C3 33p, L4 47n
    asw = [("L", 47e-9), ("C", 33e-12), ("L", 82e-9), ("C", 47e-12), ("L", 82e-9), ("C", 33e-12), ("L", 47e-9)]
    fd = p["f_down_mhz"] * 1e6
    lpf = {}
    for name, el in [("as_written_TX001_7.3", asw),
                     ("cheb7_0.1dB_fc175MHz", cheb_ladder(7, 0.1, 175e6)),
                     ("cheb7_0.1dB_fc165MHz", cheb_ladder(7, 0.1, 165e6))]:
        row = {"elements": [(k, v) for k, v in el]}
        for q in (None, 80):
            key = "ideal" if q is None else f"QL{q}_QC500"
            row[key] = {}
            for n in (1, 2, 3, 4):
                s21, s11 = ladder_s21_db(n * fd, el, q_l=q, q_c=(500 if q else None))
                row[key][f"{n}f"] = {"s21_db": round(s21, 2), "s11_db": round(s11, 1)}
            s21u, _ = ladder_s21_db(p["f_up_mhz"] * 1e6, el, q_l=q, q_c=(500 if q else None))
            row[key]["at_f_up"] = round(s21u, 1)
        # -3 dB cutoff (ideal)
        fs = np.linspace(50e6, 400e6, 3501)
        s = np.array([ladder_s21_db(f, el)[0] for f in fs])
        below = fs[s < -3.0]
        row["f_minus3dB_mhz"] = round(float(below[0] / 1e6), 1) if len(below) else None
        lpf[name] = row
        print(name, "f-3dB", row["f_minus3dB_mhz"], "MHz;", json.dumps(row["ideal"]), json.dumps(row.get("QL80_QC500")))
    res["lpf"] = lpf

    # Full-duplex desense at f_up for offsets from 3 f_down
    n0 = K_DBM + 10 * math.log10(p["rx_tsys_k"])
    res["rx_n0_dbm_hz"] = n0
    rows = []
    for d_mhz in (0.05, 0.1, 0.3, 0.5, 1.0, 2.2, 5.0):
        for h3 in (60.0, 70.0, 80.0):
            p_h3_rx = p["p_tx_dbm"] - h3 - p["antenna_isolation_db_437"]
            skirt = p_h3_rx + pn_at(d_mhz * 1e6 / 3, p["pn_fund_dbc_hz"]) + 20 * math.log10(3)
            # fundamental's far noise at f_up, through the LPF (as-written filter, Q 80) and the isolation
            lpf_att = -lpf["cheb7_0.1dB_fc175MHz"]["QL80_QC500"]["at_f_up"]
            far = p["p_tx_dbm"] + p["tx_far_noise_floor_dbc_hz"] - lpf_att - p["antenna_isolation_db_437"]
            i0 = db_add(skirt, far)
            desense = 10 * math.log10(1 + 10 ** ((i0 - n0) / 10))
            rows.append({"offset_mhz": d_mhz, "h3_dbc": h3, "h3_at_rx_dbm": round(p_h3_rx, 1),
                         "skirt_dbm_hz": round(skirt, 1), "far_noise_dbm_hz": round(far, 1),
                         "desense_db": round(desense, 2)})
    res["desense"] = rows
    for r in rows:
        if r["offset_mhz"] in (0.1, 0.5, 1.0, 2.2):
            print("desense", r)
    # The 3rd harmonic needed for <= 0.5 dB desense at each offset
    need = {}
    for d_mhz in (0.1, 0.3, 0.5, 1.0, 2.2, 5.0):
        target = n0 + 10 * math.log10(10 ** (0.5 / 10) - 1)
        pn = pn_at(d_mhz * 1e6 / 3, p["pn_fund_dbc_hz"]) + 20 * math.log10(3)
        h3_needed = p["p_tx_dbm"] - p["antenna_isolation_db_437"] + pn - target
        need[str(d_mhz)] = round(h3_needed, 1)
    res["h3_needed_for_0.5dB_desense_dbc"] = need
    print("H3 suppression needed for 0.5 dB desense, by offset (MHz):", need)

    # Co-channel case: harmonic inside the uplink channel
    cochannel_needed = p["p_tx_dbm"] - p["antenna_isolation_db_437"] - (n0 + 10 * math.log10(p["rx_noise_bw_hz"]) - 10)
    res["h3_needed_if_cochannel_dbc"] = round(cochannel_needed, 1)
    print("If 3 f_down fell in the uplink channel the harmonic would need", round(cochannel_needed, 1),
          "dB suppression: not achievable, so the frequency plan is the protection")

    # RX filter rejection at f_down
    p_fund_rx = p["p_tx_dbm"] - p["antenna_isolation_db_146"]
    rej = {}
    for r_db in (0, 20, 40, 60, 70):
        pin = p_fund_rx - r_db
        hd3_dbm = pin + 2 * (pin - p["lna_iip3_dbm"]) - 9.54
        rej[str(r_db)] = {"fund_at_lna_dbm": round(pin, 1), "lna_generated_h3_dbm": round(hd3_dbm, 1),
                          "blocking_margin_db": round(p["rx_blocking_dbm_1db"] - pin, 1)}
    res["rx_filter_rejection_at_f_down"] = rej
    print("RX filter rejection at f_down:", json.dumps(rej))
    if args.out:
        json.dump(res, open(args.out, "w"), indent=1)


if __name__ == "__main__":
    main()
