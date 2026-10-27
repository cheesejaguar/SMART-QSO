#!/usr/bin/env python3
"""SMART-QSO RF board (SQSO-RF-BRD-001): design calculations.

Every number in RF_BOARD_DESIGN.md that is calculated comes from this script. Inputs are in INPUTS with their
sources; outputs are printed and written to rf_calcs.json next to this file (design.py reads the stresses from it
for the BOM).

Sections
  1  receiver noise figure, system noise temperature and sensitivity
  2  receive filter FL1 (pre-LNA high-pass with 146 MHz traps) with tolerances and Q, plus the SAW
  3  transmitter output network: PA match + elliptic low-pass, with tolerances, Q, SRF and ESL (Monte Carlo)
  4  harmonic budget at the antenna port and the 3rd-harmonic suppression
  5  full-duplex desense with the selected synthesizer's phase noise
  6  PA output power, efficiency, DC power and junction temperatures
  7  DC power of the always-on receive section and the repeat path
  8  timing: heartbeat monostable, 120 s key-down timer, CTCSS, synthesizer lock
  9  deviation scaling (audio ADC full scale, synthesizer FSK word, AFSK and voice levels)
  10 forward/reflected power detector scaling and the APC references
  11 frequency reference error budget
  12 isolation budget: the 2 m fundamental at the 70 cm LNA
  13 clock harmonics against the uplink band
  14 part stress and derating (NASA EEE-INST-002 / project 50 % rule, EEE_PARTS_LIST Rev C §12)
  15 magnetic cleanliness at the magnetometer on board A2 (OBC item C16): keep-out radius, PA current field
  16 antenna ports (Version C): MMCX connectors and 1.37 mm cables to the AntS, route length, loss and return loss
  17 shield-can vents (Version D amended): leakage of a 1.5 mm vent and the cavities' volume-to-vent-area ratio

Usage: python3 rf_calcs.py [--mc 4000] [--quiet]
Needs numpy only.
"""
from __future__ import annotations

import argparse
import json
import math
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
K_DBM = -198.6          # 10 log10(k) in dBm/K/Hz
T0 = 290.0

INPUTS = {
    # ---- frequencies (BASELINE §7; SQSO-LBA-001 v3.0 §2.2)
    "f_down_mhz": 145.9, "f_down_band_mhz": (145.8, 146.0),
    "f_up_mhz": 435.5, "f_up_band_mhz": (435.0, 437.3),
    "p_tx_nom_dbm": 27.0, "p_tx_max_dbm": 30.0,
    # ---- link-budget assumptions this board must meet (SQSO-LBA-001 §2.4, §3.2, §7)
    "lba_tsys_k": 1000.0, "lba_rx_req_k_290src": 650.0,
    "cn0_voice_12db_sinad": 47.4, "cn0_tc_90pct": 53.0, "cn0_afsk_corr": 52.9,
    "ant_iso_437_db": 15.0, "ant_iso_146_db": 25.0,
    # ---- receive chain (part data; see RF_BOARD_DESIGN §3)
    "coax_uhf_db": 0.25,           # W10 v2 value (RG-316); replaced in main() by section 16 (W10 v3)
    # ---- antenna cables W10 v3 (section 16): 1.37 mm micro-coax, MMCX at both ends. Attenuation of the 1.37 mm
    #      class assumed 2.6 dB/m at 2.4 GHz, scaled by sqrt(f), +20 % at +60 C [to confirm with the cable data sheet]
    "cab137_db_per_m_2g4": 2.6, "cab_hot_factor": 1.20, "cab_len_m": 0.22,
    "mmcx_il_db_per_sqrtGHz": 0.10,   # per mated pair (MMCX class: 0.1 x sqrt(f GHz) dB max)
    "mmcx_vswr": 1.12,                # Molex MMCX series, DC-2.5 GHz
    "cab_vswr": 1.10,                 # cable termination, each end [assumption]
    "balun_uhf_db": 0.30,          # lumped lattice balun, Q 60 inductors (computed below as a check)
    "lna_nf_db": 0.80,             # BFP640F at 6 mA: NFmin 0.55 dB at 450 MHz (Infineon datasheet) + 0.25 dB match loss
    "lna_gain_db": 18.0,
    "saw_il_db_typ": 2.1, "saw_il_db_max": 4.2,   # Tai-Saw TA2209A approval sheet
    "ic_match_db": 0.5,            # SAW to CC1200 differential input (balun + match)
    "rx_ic_nf_db": 7.0,            # CC1200 in low-power RX mode [estimate: TI gives sensitivity, not NF; -122 dBm at
                                   # 1.2 kbps 2-FSK, 11 kHz filter, high-performance mode]; +1 dB for low-power mode
    "impl_loss_db": 2.0,           # CFM (soft frequency) output demodulated in the TNC MCU, not yet characterized (VI-RFB-01)
    # ---- transmitter (part data)
    "lmx_pn_146": {1e3: -100.0, 1e4: -125.0, 1e5: -140.0, 1e6: -152.0, 1e7: -157.0},  # LMX2571 at 480 MHz -123@12.5k,
    #   -145@1M (datasheet), divided to 146 MHz (-10.3 dB), floor limited to -157 dBc/Hz [estimate]
    "lba_pn_146": {1e3: -85.0, 1e4: -100.0, 1e5: -118.0, 1e6: -135.0, 1e7: -145.0},  # the LBA §7.3 assumption
    "pa_h2_dbc": -20.0, "pa_h3_dbc": -15.0,       # saturated single-ended Si MOSFET, worst case [assumption, VI-RF-04]
    "board_leak_floor_dbc": -90.0,                # coupling around the filter allowed by the shield-can rules (§9)
    "vdd_pa": 5.0, "vdd_pa_tol": 0.03,            # EPS 5V_PA buck +-3 % (EPS_DESIGN 2.0 §4)
    "rd01mus3_eta_sat": 0.59, "rd01mus3_vknee": 0.7,  # RD01MUS3 datasheet (Jan 2026): 58 % at 3.6 V, 60 % at 7.2 V
    "rd01mus3_rthjc2_max": 15.0, "rd01mus3_rthjc1_max": 23.0, "rth_case_board": 4.0, "rth_spread_board": 15.0,
    "pa_RL_ohm": 7.5,                             # chosen load line (section 6)
    # ---- losses after the PA drain, at 145.9 MHz (section 3 computes the filter value)
    "loss_trace_conn_db": 0.05, "loss_sense_db": 0.03, "balun_vhf_db": 0.25,
    # ---- board environment (THERMAL_ANALYSIS v2.0; THERMAL_DESIGN 1.3 §1.1 per-unit AFT)
    "t_board_hot_c": 60.0, "t_board_cold_c": -30.0,
    # ---- rails
    "v_radio_nom": 3.30, "v_radio_tol": 0.03,     # 3V3_AO through SW2 (EPS); power at +3 % (conservative)
    "v_radio_tol_eps": 0.02,                      # EPS_DESIGN 2.0 §4: 3V3_AO 3.3 V +-2 % (rail minimum)
}


# ------------------------------------------------------------------------------------------------- utilities
def db10(x):
    return 10.0 * math.log10(x)


def undb(x):
    return 10.0 ** (x / 10.0)


def t_from_nf(nf_db):
    return T0 * (undb(nf_db) - 1.0)


def pn_interp(off_hz, table):
    xs = sorted(table)
    return float(np.interp(math.log10(max(off_hz, xs[0])), [math.log10(x) for x in xs], [table[x] for x in xs]))


def abcd_series(z):
    m = np.zeros(z.shape + (2, 2), complex)
    m[..., 0, 0] = 1
    m[..., 0, 1] = z
    m[..., 1, 1] = 1
    return m


def abcd_shunt(y):
    m = np.zeros(y.shape + (2, 2), complex)
    m[..., 0, 0] = 1
    m[..., 1, 0] = y
    m[..., 1, 1] = 1
    return m


def s21_s11(mats, z0=50.0, zl=None):
    m = mats[0]
    for x in mats[1:]:
        m = m @ x
    a, b, c, d = m[..., 0, 0], m[..., 0, 1], m[..., 1, 0], m[..., 1, 1]
    z_s, z_l = z0, (z0 if zl is None else zl)
    # power gain into z_l from a source z_s (transducer gain), and input reflection
    zin = (a * z_l + b) / (c * z_l + d)
    s11 = (zin - z_s) / (zin + z_s)
    vout_over_vs = z_l / (a * z_l + b + z_s * (c * z_l + d))
    gt = (np.abs(2 * vout_over_vs) ** 2) * z_s / z_l      # transducer gain for real source and load
    return 10 * np.log10(gt), 20 * np.log10(np.maximum(np.abs(s11), 1e-12))


def z_ind(w, L, q_ref, f_ref, srf=None):
    """Inductor: series R from Q (Q rising as sqrt(f)), parallel C from the self-resonant frequency"""
    q = q_ref * np.sqrt(w / (2 * math.pi * f_ref))
    z = 1j * w * L + w * L / q
    if srf:
        cp = 1.0 / ((2 * math.pi * srf) ** 2 * L)
        z = 1.0 / (1.0 / z + 1j * w * cp)
    return z


def z_cap(w, C, q=800.0, esl=0.0):
    z = 1.0 / (1j * w * C) + 1.0 / (w * C * q) + 1j * w * esl
    return z


def nelder_mead(f, x0, step, iters=3000, tol=1e-9):
    n = len(x0)
    pts = [np.array(x0, float)]
    for i in range(n):
        p = np.array(x0, float)
        p[i] += step[i]
        pts.append(p)
    vals = [f(p) for p in pts]
    for _ in range(iters):
        order = np.argsort(vals)
        pts = [pts[i] for i in order]
        vals = [vals[i] for i in order]
        if abs(vals[-1] - vals[0]) < tol:
            break
        cen = np.mean(pts[:-1], axis=0)
        xr = cen + (cen - pts[-1])
        fr = f(xr)
        if fr < vals[0]:
            xe = cen + 2 * (cen - pts[-1])
            fe = f(xe)
            pts[-1], vals[-1] = (xe, fe) if fe < fr else (xr, fr)
        elif fr < vals[-2]:
            pts[-1], vals[-1] = xr, fr
        else:
            xc = cen + 0.5 * (pts[-1] - cen)
            fc = f(xc)
            if fc < vals[-1]:
                pts[-1], vals[-1] = xc, fc
            else:
                for i in range(1, len(pts)):
                    pts[i] = pts[0] + 0.5 * (pts[i] - pts[0])
                    vals[i] = f(pts[i])
    i = int(np.argmin(vals))
    return pts[i], vals[i]


E24 = [1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0, 3.3, 3.6, 3.9, 4.3, 4.7, 5.1, 5.6, 6.2, 6.8, 7.5,
       8.2, 9.1]


def snap_cap_pf(c_pf):
    """Nearest standard value: 0.1 pF steps below 10 pF (Murata GJM/GRM C0G), E24 above"""
    if c_pf < 10.0:
        return max(0.5, round(c_pf * 10) / 10.0)
    dec = 10 ** math.floor(math.log10(c_pf))
    return min((v * dec for v in E24 + [10.0]), key=lambda v: abs(v - c_pf))


# Inductor series used (values in nH). Coilcraft 1812SMS "Midi Spring" (air core) for the TX filter and match;
# Coilcraft 0603HP (wire-wound ceramic) for the receive filter. Values are the series' published steps.
SMS_1812 = [22, 27, 33, 39, 43, 47, 56, 68, 82, 90, 100, 120]
HP_0603 = [1.6, 1.8, 2.2, 2.7, 3.3, 3.9, 4.3, 4.7, 5.6, 6.8, 7.5, 8.2, 8.7, 9.5, 10, 11, 12, 15, 18, 20, 22, 24, 27,
           30, 33, 36, 39, 47, 51, 56, 68, 72, 82, 100]


def snap(v, series):
    return min(series, key=lambda s: abs(s - v))


# ---------------------------------------------------------------------------------------- 1 receiver cascade
def receiver(p):
    out = {}
    fl1_il = p["_fl1_il_worst_db"]
    for case, saw in (("typ", p["saw_il_db_typ"]), ("worst", p["saw_il_db_max"])):
        stages = [("balun", p["balun_uhf_db"], -p["balun_uhf_db"]),
                  ("FL1", fl1_il, -fl1_il),
                  ("LNA", p["lna_nf_db"], p["lna_gain_db"]),
                  ("SAW", saw, -saw),
                  ("CC1200 match", p["ic_match_db"], -p["ic_match_db"]),
                  ("CC1200", p["rx_ic_nf_db"] if case == "worst" else p["rx_ic_nf_db"] - 1.0, 0.0)]
        te, g = 0.0, 1.0
        for _, nf, gain in stages:
            te += t_from_nf(nf) / g
            g *= undb(gain)
        nf_board = db10(1 + te / T0)
        # referred to the antenna terminals: add the coax
        te_ant = t_from_nf(p["coax_uhf_db"]) + te * undb(p["coax_uhf_db"])
        nf_ant = db10(1 + te_ant / T0)
        tsys_290 = T0 + te_ant
        out[case] = {"Te_board_K": round(te, 1), "NF_board_dB": round(nf_board, 2), "Te_ant_K": round(te_ant, 1),
                     "NF_ant_dB": round(nf_ant, 2), "Tsys_290K_source_K": round(tsys_290, 1)}
        n0 = K_DBM + db10(tsys_290)
        out[case]["sens_12dB_SINAD_dBm"] = round(p["cn0_voice_12db_sinad"] + p["impl_loss_db"] + n0, 1)
        out[case]["sens_TC_90pct_dBm"] = round(p["cn0_tc_90pct"] + p["impl_loss_db"] + n0, 1)
        # the link budget's 1,000 K: Earth 290 K + receiver + 350 K interference allowance
        out[case]["Tsys_with_350K_allowance_K"] = round(tsys_290 + 350.0, 1)
        out[case]["margin_vs_1000K_dB"] = round(db10(1000.0 / (tsys_290 + 350.0)), 2)
        out[case]["margin_vs_650K_req_dB"] = round(db10(650.0 / tsys_290), 2)
    return out


# --------------------------------------------------------------------------- 2 receive filter FL1 (high-pass)
def fl1_network(w, v, q_l=(60.0, 436e6), srf=None, cq=600.0, esl=0.4e-9):
    """shunt (L1+C1s) - series C2 - shunt (L3+C3s) - series C4 - shunt (L5+C5s) - series C6 - shunt L7
    v: [L1, C1s, C2, L3, C3s, C4, L5, C5s, C6, L7] in H/F"""
    L1, C1s, C2, L3, C3s, C4, L5, C5s, C6, L7 = v
    qr, fr = q_l
    mats = []
    for Ls, Cs in ((L1, C1s),):
        mats.append(abcd_shunt(1.0 / (z_ind(w, Ls, qr, fr, srf) + z_cap(w, Cs, cq, esl))))
    mats.append(abcd_series(z_cap(w, C2, cq)))
    mats.append(abcd_shunt(1.0 / (z_ind(w, L3, qr, fr, srf) + z_cap(w, C3s, cq, esl))))
    mats.append(abcd_series(z_cap(w, C4, cq)))
    mats.append(abcd_shunt(1.0 / (z_ind(w, L5, qr, fr, srf) + z_cap(w, C5s, cq, esl))))
    mats.append(abcd_series(z_cap(w, C6, cq)))
    mats.append(abcd_shunt(1.0 / z_ind(w, L7, qr, fr, srf)))
    return mats


def design_fl1(p, n_mc, rng):
    """7th-order 0.1 dB Chebyshev high-pass, fc 350 MHz, shunt-inductor first (DC path to ground for the antenna's
    static charge), with the first three shunt arms made series-resonant (traps) at 146.5, 145.0 and 143.5 MHz.
    Each trap inductor is raised so that the arm keeps the prototype reactance at 436 MHz."""
    f_pass = np.linspace(435.0e6, 437.3e6, 9)
    f_stop = np.linspace(144.0e6, 146.0e6, 9)
    f_250 = np.linspace(250e6, 330e6, 17)
    g = [1.1812, 1.4228, 2.0967, 1.5734, 2.0967, 1.4228, 1.1812]
    wc = 2 * math.pi * 350e6
    w4 = 2 * math.pi * 436e6
    traps = [146.5e6, 145.0e6, 143.5e6]
    v = []
    t = 0
    for k, gk in enumerate(g):
        if k % 2 == 0:
            L = 50.0 / (gk * wc)
            if k < 6:
                wt = 2 * math.pi * traps[t]
                t += 1
                # arm: L' in series with Cs, resonant at wt, reactance at 436 MHz equal to w4 L
                Lp = L / (1 - (wt / w4) ** 2)
                Cs = 1.0 / (wt ** 2 * Lp)
                v += [snap(Lp * 1e9, HP_0603) * 1e-9, Cs]
            else:
                v += [snap(L * 1e9, HP_0603) * 1e-9]
        else:
            v += [snap_cap_pf(1.0 / (gk * wc * 50.0) * 1e12) * 1e-12]
    # re-tune the trap capacitors to the snapped inductors, to E24 / 0.1 pF
    for li, ci, ft in ((0, 1, traps[0]), (3, 4, traps[1]), (6, 7, traps[2])):
        v[ci] = snap_cap_pf(1.0 / ((2 * math.pi * ft) ** 2 * v[li]) * 1e12) * 1e-12
    v = np.array(v)
    scale = np.array([1e-9, 1e-12, 1e-12, 1e-9, 1e-12, 1e-12, 1e-9, 1e-12, 1e-12, 1e-9])
    names = ["L1", "C1s", "C2", "L3", "C3s", "C4", "L5", "C5s", "C6", "L7"]
    # Monte Carlo: L +-2 %, C +-2 % (>= 10 pF) or +-0.1 pF, inductor Q 45-75 at 436 MHz, SRF 2.5-4 GHz
    f_all = {"pass": f_pass, "stop146": f_stop, "250_330": f_250,
             "560_620": np.linspace(560e6, 620e6, 7)}
    res = {k: [] for k in ("il_pass", "rl_pass", "att146", "att250_330", "att560_620")}
    for _ in range(n_mc):
        vv = v.copy()
        for i in range(len(vv)):
            if scale[i] == 1e-9:
                vv[i] *= 1 + rng.uniform(-0.02, 0.02)
            else:
                vv[i] = vv[i] * (1 + rng.uniform(-0.02, 0.02)) if vv[i] >= 10e-12 else vv[i] + rng.uniform(-0.1e-12, 0.1e-12)
        q = rng.uniform(45, 75)
        srf = rng.uniform(2.5e9, 4.0e9)
        out = {}
        for k, fs in f_all.items():
            out[k] = s21_s11(fl1_network(2 * np.pi * fs, vv, (q, 436e6), srf))
        res["il_pass"].append(-out["pass"][0].min())
        res["rl_pass"].append(-out["pass"][1].max())
        res["att146"].append(-out["stop146"][0].max())
        res["att250_330"].append(-out["250_330"][0].max())
        res["att560_620"].append(-out["560_620"][0].max())
    nom = {k: s21_s11(fl1_network(2 * np.pi * fs, v, (60.0, 436e6), 3.0e9)) for k, fs in f_all.items()}
    summ = {
        "elements": {n: (round(x * 1e9, 2) if scale[i] == 1e-9 else round(x * 1e12, 2)) for i, (n, x) in enumerate(zip(names, v))},
        "units": "nH for L, pF for C",
        "nominal": {"il_435_437.3_dB": round(-nom["pass"][0].min(), 2), "rl_dB": round(-nom["pass"][1].max(), 1),
                    "att_144_146_dB": round(-nom["stop146"][0].max(), 1),
                    "att_250_330_dB": round(-nom["250_330"][0].max(), 1),
                    "att_560_620_dB": round(-nom["560_620"][0].max(), 1)},
        "monte_carlo_n": n_mc,
        "worst": {"il_dB": round(max(res["il_pass"]), 2), "rl_dB": round(min(res["rl_pass"]), 1),
                  "att_144_146_dB": round(min(res["att146"]), 1),
                  "att_250_330_dB": round(min(res["att250_330"]), 1),
                  "att_560_620_dB": round(min(res["att560_620"]), 1)},
        "p99": {"il_dB": round(float(np.percentile(res["il_pass"], 99)), 2),
                "att_144_146_dB": round(float(np.percentile(res["att146"], 1)), 1)},
    }
    return summ


# ---------------------------------------------------------------- 3 TX output network: PA match + elliptic LPF
def lpf_network(w, v, qL=(110.0, 146e6), srf=None, cq=800.0, esl=0.6e-9):
    """C1 - (L2 || C2t) - C3 - (L4 || C4t) - C5 - (L6 || C6t) - C7, 50 ohm
    v: [C1, L2, C2t, C3, L4, C4t, C5, L6, C6t, C7]"""
    C1, L2, C2t, C3, L4, C4t, C5, L6, C6t, C7 = v
    q, fr = qL
    mats = [abcd_shunt(1.0 / z_cap(w, C1, cq, esl))]
    for L, Ct, Csh in ((L2, C2t, C3), (L4, C4t, C5), (L6, C6t, C7)):
        zl = z_ind(w, L, q, fr, srf)
        zt = 1.0 / (1.0 / zl + 1.0 / z_cap(w, Ct, cq, 0.3e-9))
        mats.append(abcd_series(zt))
        mats.append(abcd_shunt(1.0 / z_cap(w, Csh, cq, esl)))
    return mats


def match_network(w, v, qL=(110.0, 146e6)):
    """PA load R_L to 50 ohm: two low-pass L sections, series L then shunt C each (from the drain side).
    v: [La, Ca, Lb, Cb]"""
    La, Ca, Lb, Cb = v
    q, fr = qL
    return [abcd_series(z_ind(w, La, q, fr)), abcd_shunt(1.0 / z_cap(w, Ca)),
            abcd_series(z_ind(w, Lb, q, fr)), abcd_shunt(1.0 / z_cap(w, Cb))]


def design_match(rl):
    """Two-section L match rl -> sqrt(rl*50) -> 50 at 145.9 MHz (equal Q)"""
    w = 2 * math.pi * 145.9e6
    rm = math.sqrt(rl * 50.0)
    q1 = math.sqrt(rm / rl - 1)
    q2 = math.sqrt(50.0 / rm - 1)
    La = q1 * rl / w
    Ca = q1 / (w * rm)
    Lb = q2 * rm / w
    Cb = q2 / (w * 50.0)
    return np.array([La, Ca, Lb, Cb]), {"R_mid_ohm": round(rm, 2), "Q1": round(q1, 2), "Q2": round(q2, 2)}


def design_lpf(p, n_mc, rng):
    fb = p["f_down_band_mhz"]
    f_pass = np.linspace(fb[0] * 1e6, fb[1] * 1e6, 5)
    bands = {n: np.linspace(n * fb[0] * 1e6, n * fb[1] * 1e6, 7) for n in (2, 3, 4, 5)}
    fu = p["f_up_band_mhz"]
    f_up = np.linspace(fu[0] * 1e6, fu[1] * 1e6, 9)
    # starting point: 7th-order 0.1 dB Chebyshev low-pass, fc 175 MHz; traps at 3f, 2f, 3f
    g = [1.1812, 1.4228, 2.0967, 1.5734, 2.0967, 1.4228, 1.1812]
    wc = 2 * math.pi * 175e6
    C = lambda gk: gk / (50.0 * wc)
    L = lambda gk: gk * 50.0 / wc
    w2, w3 = 2 * math.pi * 291.8e6, 2 * math.pi * 437.7e6
    v0 = np.array([C(g[0]), L(g[1]) * 0.9, 1 / (w3 ** 2 * L(g[1]) * 0.9), C(g[2]), L(g[3]) * 0.9,
                   1 / (w2 ** 2 * L(g[3]) * 0.9), C(g[4]), L(g[5]) * 0.9, 1 / (w3 ** 2 * L(g[5]) * 0.9), C(g[6])])
    is_l = np.array([0, 1, 0, 0, 1, 0, 0, 1, 0, 0], bool)
    scale = np.where(is_l, 1e-9, 1e-12)
    wp = 2 * np.pi * f_pass

    def cost(x):
        v = x * scale
        if np.any(v <= 0):
            return 1e9
        s21p, s11p = s21_s11(lpf_network(wp, v))
        a2 = -s21_s11(lpf_network(2 * np.pi * bands[2], v))[0].max()
        a3 = -s21_s11(lpf_network(2 * np.pi * bands[3], v))[0].max()
        il = -s21p.min()
        rl = -s11p.max()
        return il * 60 + max(0, 22 - rl) * 4 + max(0, 75 - a2) * 0.5 + max(0, 110 - a3) * 1.0

    x, _ = nelder_mead(cost, v0 / scale, (v0 / scale) * 0.06, iters=5000)
    v = x * scale
    snapped = []
    for i, val in enumerate(v):
        snapped.append(snap(val * 1e9, SMS_1812) * 1e-9 if is_l[i] else snap_cap_pf(val * 1e12) * 1e-12)
    v = np.array(snapped)
    # re-tune the trap capacitors to the snapped inductors (0.1 pF steps)
    for li, ci, wt in ((1, 2, w3), (4, 5, w2), (7, 8, w3)):
        v[ci] = round(1.0 / (wt ** 2 * v[li]) * 1e12 * 10) / 10 * 1e-12
    names = ["C1", "L2", "C2t", "C3", "L4", "C4t", "C5", "L6", "C6t", "C7"]

    m_vals, m_info = design_match(p["pa_RL_ohm"])
    m_vals_s = np.array([snap(m_vals[0] * 1e9, [5.0, 6.0, 7.0, 8.0, 10.0, 12.5, 15.0, 17.5, 22, 27]) * 1e-9,
                         snap_cap_pf(m_vals[1] * 1e12) * 1e-12,
                         snap(m_vals[2] * 1e9, [10.0, 12.5, 15.0, 17.5, 22, 27, 33]) * 1e-9,
                         snap_cap_pf(m_vals[3] * 1e12) * 1e-12])

    def chain(w, vl, vm, q=110.0, srf=None, rl=p["pa_RL_ohm"]):
        # source impedance R_L (the PA's load line seen as a Thevenin source); compute transducer gain rl -> 50
        mats = match_network(w, vm, (q, 146e6)) + lpf_network(w, vl, (q, 146e6), srf)
        m = mats[0]
        for x_ in mats[1:]:
            m = m @ x_
        a, b, c, d = m[..., 0, 0], m[..., 0, 1], m[..., 1, 0], m[..., 1, 1]
        zl = 50.0
        v2 = zl / (a * zl + b + rl * (c * zl + d))
        gt = np.abs(2 * v2) ** 2 * rl / zl
        zin = (a * zl + b) / (c * zl + d)
        gam = (zin - rl) / (zin + rl)
        return 10 * np.log10(gt), 20 * np.log10(np.abs(gam) + 1e-12)

    res = {k: [] for k in ("il", "rl", "a2", "a3", "a4", "a5", "a_up")}
    lres = {k: [] for k in ("il", "a2", "a3", "a_up")}
    for _ in range(n_mc):
        vl = v.copy()
        for i in range(len(vl)):
            if is_l[i]:
                vl[i] *= 1 + rng.uniform(-0.02, 0.02)
            elif vl[i] >= 10e-12:
                vl[i] *= 1 + rng.uniform(-0.02, 0.02)
            else:
                vl[i] += rng.uniform(-0.1e-12, 0.1e-12)
        vm = m_vals_s * (1 + rng.uniform(-0.02, 0.02, 4))
        q = rng.uniform(80, 150)
        srf = rng.uniform(1.0e9, 2.0e9)
        il, rl = chain(2 * np.pi * f_pass, vl, vm, q, srf)
        res["il"].append(-il.max() if False else -il.min())
        res["rl"].append(-rl.max())
        for n in (2, 3, 4, 5):
            res[f"a{n}"].append(-chain(2 * np.pi * bands[n], vl, vm, q, srf)[0].max())
        res["a_up"].append(-chain(2 * np.pi * f_up, vl, vm, q, srf)[0].max())
        s21p, _ = s21_s11(lpf_network(2 * np.pi * f_pass, vl, (q, 146e6), srf))
        lres["il"].append(-s21p.min())
        lres["a2"].append(-s21_s11(lpf_network(2 * np.pi * bands[2], vl, (q, 146e6), srf))[0].max())
        lres["a3"].append(-s21_s11(lpf_network(2 * np.pi * bands[3], vl, (q, 146e6), srf))[0].max())
        lres["a_up"].append(-s21_s11(lpf_network(2 * np.pi * f_up, vl, (q, 146e6), srf))[0].max())
    il_n, rl_n = chain(2 * np.pi * f_pass, v, m_vals_s)
    nominal = {"il_dB": round(-il_n.min(), 2), "rl_dB": round(-rl_n.max(), 1)}
    for n in (2, 3, 4, 5):
        nominal[f"att_{n}f_dB"] = round(-chain(2 * np.pi * bands[n], v, m_vals_s)[0].max(), 1)
    nominal["att_uplink_dB"] = round(-chain(2 * np.pi * f_up, v, m_vals_s)[0].max(), 1)
    s21n, s11n = s21_s11(lpf_network(2 * np.pi * f_pass, v))
    # -3 dB point of the LPF alone
    fs = np.linspace(140e6, 260e6, 1201)
    s = s21_s11(lpf_network(2 * np.pi * fs, v))[0]
    f3 = float(fs[np.argmax(s < -3.0)]) / 1e6
    out = {
        "lpf_elements": {n: (round(x * 1e9, 1) if is_l[i] else round(x * 1e12, 2)) for i, (n, x) in enumerate(zip(names, v))},
        "match_elements": {"La_nH": round(m_vals_s[0] * 1e9, 1), "Ca_pF": round(m_vals_s[1] * 1e12, 1),
                           "Lb_nH": round(m_vals_s[2] * 1e9, 1), "Cb_pF": round(m_vals_s[3] * 1e12, 1), **m_info},
        "lpf_alone_nominal": {"il_dB": round(-s21n.min(), 2), "rl_dB": round(-s11n.max(), 1), "f_3dB_MHz": round(f3, 1)},
        "lpf_alone_worst": {"il_dB": round(max(lres["il"]), 2), "att_2f_dB": round(min(lres["a2"]), 1),
                            "att_3f_dB": round(min(lres["a3"]), 1), "att_uplink_dB": round(min(lres["a_up"]), 1)},
        "chain_nominal": nominal,
        "chain_worst": {"il_dB": round(max(res["il"]), 2), "rl_dB": round(min(res["rl"]), 1),
                        **{f"att_{n}f_dB": round(min(res[f"a{n}"]), 1) for n in (2, 3, 4, 5)},
                        "att_uplink_dB": round(min(res["a_up"]), 1)},
        "chain_p1": {f"att_{n}f_dB": round(float(np.percentile(res[f"a{n}"], 1)), 1) for n in (2, 3)},
        "monte_carlo_n": n_mc,
        "model": "L +-2 %, C +-2 % (>= 10 pF) or +-0.1 pF, inductor Q 80-150 at 146 MHz rising as sqrt(f), "
                 "inductor SRF 1.0-2.0 GHz, capacitor Q 800 and ESL 0.6 nH (0603 plus via), trap capacitor ESL 0.3 nH",
    }
    return out


# ------------------------------------------------------------------------------------ 4 harmonic budget
def harmonics(p, lpf):
    w = lpf["chain_worst"]
    floor = p["board_leak_floor_dbc"]
    out = {}
    for n, pa in ((2, p["pa_h2_dbc"]), (3, p["pa_h3_dbc"]), (4, -25.0), (5, -30.0)):
        filt = pa - w[f"att_{n}f_dB"]
        tot = db10(undb(filt) + undb(floor))
        out[f"H{n}"] = {"pa_dbc": pa, "after_network_worst_dbc": round(filt, 1), "with_leak_floor_dbc": round(tot, 1)}
    # 97.307(e) at 1 W: <= 25 uW (-16 dBm) and >= 40 dB below => -46 dBc
    out["limit_97307e_dbc_at_1W"] = -46.0
    out["H3_requirement_dbc"] = -70.0
    out["H3_margin_dB"] = round(-70.0 - out["H3"]["with_leak_floor_dbc"], 1)
    out["note"] = ("The leak floor stands for coupling around the filter; the shield and layout rules of "
                   "RF_BOARD_DESIGN §9 are set so that it stays at or below -90 dBc. Only the measured value at "
                   "the port with the antennas deployed (VI-RF-04) closes RF-07.")
    return out


# ---------------------------------------------------------------------------------------- 5 desense
def desense(p, h3_port_dbc):
    n0 = K_DBM + db10(p["lba_tsys_k"])
    rows = []
    for d_mhz in (0.1, 0.5, 1.0, 2.2, 2.4, 5.0):
        for label, tab, scale3 in (("LBA model (offset/3), LBA phase noise", p["lba_pn_146"], True),
                                   ("LBA model (offset/3), LMX2571", p["lmx_pn_146"], True),
                                   ("harmonic at the same offset, LMX2571", p["lmx_pn_146"], False)):
            for h3 in (h3_port_dbc, -70.0):
                pn = pn_interp(d_mhz * 1e6 / 3 if scale3 else d_mhz * 1e6, tab) + 20 * math.log10(3)
                skirt = p["p_tx_max_dbm"] + h3 - p["ant_iso_437_db"] + pn
                des = 10 * math.log10(1 + undb(skirt - n0))
                rows.append({"offset_MHz": d_mhz, "model": label, "h3_dbc": round(h3, 1),
                             "skirt_dBm_Hz": round(skirt, 1), "desense_dB": round(des, 3)})
    return {"rx_n0_dBm_Hz": round(n0, 1), "rows": rows}


# ------------------------------------------------------------------- 6 PA power, efficiency and thermal
def pa_budget(p, lpf):
    loss_net = lpf["chain_worst"]["il_dB"]          # match + LPF, worst case
    loss_after = p["loss_sense_db"] + p["balun_vhf_db"] + p["loss_trace_conn_db"]
    loss_total = loss_net + loss_after
    vk = p["rd01mus3_vknee"]
    out = {"loss_match_lpf_worst_dB": loss_net, "loss_after_lpf_dB": round(loss_after, 2),
           "loss_drain_to_port_dB": round(loss_total, 2)}
    rl = p["pa_RL_ohm"]
    rows = []
    for vdd_case, vdd in (("nominal", p["vdd_pa"]), ("low", p["vdd_pa"] * (1 - p["vdd_pa_tol"])),
                          ("high", p["vdd_pa"] * (1 + p["vdd_pa_tol"]))):
        psat = (vdd - vk) ** 2 / (2 * rl)
        rows.append({"vdd": round(vdd, 3), "psat_drain_W": round(psat, 3),
                     "pmax_port_dBm": round(10 * math.log10(psat * 1e3) - loss_total, 2)})
    out["psat"] = rows
    eta_s = p["rd01mus3_eta_sat"]
    psat_nom = (p["vdd_pa"] - vk) ** 2 / (2 * rl)
    exc = exciter_power(p)
    lv = {}
    for name, port_dbm in (("0.5 W", p["p_tx_nom_dbm"]), ("1 W", p["p_tx_max_dbm"])):
        pd = undb(port_dbm + loss_total) / 1e3
        if pd > psat_nom:
            pd_eff = psat_nom
            eta = eta_s
        else:
            pd_eff = pd
            eta = eta_s * math.sqrt(pd / psat_nom)   # class-B-like back-off under gate-bias power control
        dc_pa = pd_eff / eta
        dc_total = dc_pa + exc["keyed_W"]
        p_port = undb(10 * math.log10(pd_eff * 1e3) - loss_total) / 1e3
        diss_pa = dc_pa - pd_eff
        # FET2 carries ~90 % of the PA dissipation; FET1 the rest
        d2, d1 = 0.9 * diss_pa, 0.1 * diss_pa
        tj2 = p["t_board_hot_c"] + d2 * (p["rd01mus3_rthjc2_max"] + p["rth_case_board"]) + diss_pa * p["rth_spread_board"]
        tj1 = p["t_board_hot_c"] + d1 * (p["rd01mus3_rthjc1_max"] + p["rth_case_board"]) + diss_pa * p["rth_spread_board"]
        lv[name] = {"p_drain_W": round(pd_eff, 3), "p_port_W": round(p_port, 3), "eta_pa": round(eta, 3),
                    "dc_pa_W": round(dc_pa, 3), "dc_exciter_W": round(exc["keyed_W"], 3), "dc_total_W": round(dc_total, 3),
                    "i_5v_A": round(dc_total / p["vdd_pa"] + 0.0, 3),
                    "overall_eff": round(p_port / dc_total, 3), "pa_dissipation_W": round(diss_pa, 3),
                    "board_dissipation_keyed_W": round(dc_total - p_port, 3),
                    "tj_fet2_C_at_board_60C": round(tj2, 1), "tj_fet1_C_at_board_60C": round(tj1, 1)}
    out["levels"] = lv
    out["exciter"] = exc
    out["load_line_ohm"] = rl
    # trade: other load lines
    trade = []
    for r in (7.0, 7.5, 8.5, 10.0, 12.0):
        ps = (p["vdd_pa"] - vk) ** 2 / (2 * r)
        pd05 = undb(p["p_tx_nom_dbm"] + loss_total) / 1e3
        eta05 = eta_s * math.sqrt(min(1.0, pd05 / ps))
        trade.append({"RL_ohm": r, "psat_W": round(ps, 3),
                      "max_port_dBm": round(10 * math.log10(ps * 1e3) - loss_total, 2),
                      "dc_at_0.5W_W": round(pd05 / eta05 + exc["keyed_W"], 3)})
    out["load_line_trade"] = trade
    return out


def exciter_power(p):
    # LMX2571 ICC 44 mA typ (config B), 51 mA max table value, on the switched 3.3 V (from RADIO_3V3)
    lmx = 0.046 * 3.3
    pcm = 0.0086 * 5.0 + 0.0059 * 5.0          # VCC 5 V analog; VDD 3.3 V from the PA-domain LDO fed at 5 V
    det = 2 * 0.0011 * 5.0                     # two RMS detectors on the PA-domain 3.3 V LDO (fed at 5 V)
    logic = 0.004 * 5.0                        # squarer, buffers, APC op-amp, reference, gate network
    keyed = lmx + pcm + det + logic
    idle_pa = (0.0009 * 3.3 / 5.0 + 0.0002 + 0.00006 + 2 * 0.00005 + 0.0003) * 5.0  # LMX CE low via switch, PCM halted
    return {"lmx2571_W": round(lmx, 3), "pcm1808_W": round(pcm, 3), "detectors_W": round(det, 3), "logic_W": round(logic, 3),
            "keyed_W": round(keyed, 3), "idle_permitted_W": round(idle_pa, 4)}


# --------------------------------------------------------------------- 7 always-on DC power and repeat path
def rx_power(p):
    v_lo = p["v_radio_nom"] * (1 - p["v_radio_tol"])
    v_hi = p["v_radio_nom"] * (1 + p["v_radio_tol"])
    # (name, typ mA, worst mA) drawn from RADIO_3V3 (LDO currents pass through)
    loads = [("LNA BFP640F (6 mA bias, 3.0 V rail)", 6.0, 7.0),
             ("CC1200 receive, low-power mode (19 mA at 3.0 V; +10 % worst)", 19.0, 21.0),
             ("TCXO TG2016SMN 40 MHz", 1.5, 2.0),
             ("STM32L431 at 16 MHz HSI16 (84 uA/MHz) + SPI/DMA at 12.5 kS/s + DAC1/DAC2 idle + USART", 3.0, 5.0),
             ("CMX138A powersaved (transponder off)", 0.035, 0.12),
             ("audio filter op-amps TLV9002 x 2 (4 x 60 uA)", 0.24, 0.3),
             ("SN74LVC1G123 active (Rext/Cext at 0.5 VCC: 280 uA)", 0.28, 0.4),
             ("TMP235 (14.5 uA), REF/bias dividers", 0.05, 0.1),
             ("LDO ground currents (TPS7A2030 6.5 uA, TPS73101 ~400 uA), pull-ups, misc", 0.45, 0.9),
             ("LMX2571 switched rail, power-down (CE low, 0.9 mA) while TX is permitted", 0.9, 1.2),
             ("Receiver supervision: LVC1G123 RX-health monostable 0.28 mA, LVC1G17/LVC1G04, pull-downs", 0.30, 0.45)]
    typ = sum(x[1] for x in loads)
    worst = sum(x[2] for x in loads)
    rp = [("CMX138A receive mode (7.0 mA typ; 8.5 mA tx-mode figure as worst)", 7.0, 8.5),
          ("MCU DAC2 output buffer (repeat audio to the CMX138A)", 0.3, 0.4),
          ("HC4060 key-down timer running, mute and select switches", 0.1, 0.3)]
    rp_typ = sum(x[1] for x in rp)
    rp_worst = sum(x[2] for x in rp)
    return {"loads_mA": [{"load": n, "typ": t, "worst": w} for n, t, w in loads],
            "receive_typ_mA": round(typ, 2), "receive_worst_mA": round(worst, 2),
            "receive_typ_W": round(typ * 1e-3 * p["v_radio_nom"], 4), "receive_worst_W": round(worst * 1e-3 * v_hi, 4),
            "allocation_W": 0.15,
            "margin_typ": round(1 - typ * 1e-3 * p["v_radio_nom"] / 0.15, 3),
            "margin_worst": round(1 - worst * 1e-3 * v_hi / 0.15, 3),
            "repeat_path": [{"load": n, "typ": t, "worst": w} for n, t, w in rp],
            "repeat_typ_W": round(rp_typ * 1e-3 * p["v_radio_nom"], 4), "repeat_worst_W": round(rp_worst * 1e-3 * v_hi, 4),
            "repeat_allocation_W": 0.030,
            "receive_plus_repeat_worst_W": round((worst + rp_worst) * 1e-3 * v_hi, 4),
            "rail_min_at_board": radio_rail_min(p)}


def radio_rail_min(p, i_keyed=0.089):
    """Lowest RADIO_3V3 at the LMX2571 (needs >= 3.15 V) while keyed: 3V3_AO at EPS_DESIGN 2.0's -2 %, SW2
    (TPS2553-Q1, 0.12 ohm assumed), W4B (28 AWG, 0.145 m, SQSO-HAR-001 2.0 §9.2: 13.7 mV at 0.089 A), FB801 (0.10 ohm
    max DCR), TPS22918 (0.052 ohm) at the LMX2571's 46 mA. Power figures above stay at the conservative +3 %."""
    v = p["v_radio_nom"] * (1 - p["v_radio_tol_eps"])
    drops = {"SW2_TPS2553": 0.12 * i_keyed, "W4B_28AWG": 0.0137 * i_keyed / 0.089, "FB801": 0.10 * i_keyed,
             "TPS22918": 0.052 * 0.046}
    vmin = v - sum(drops.values())
    return {"v_3v3_ao_min_V": round(v, 3), "drops_mV": {k: round(x * 1e3, 1) for k, x in drops.items()},
            "v_lmx_min_V": round(vmin, 3), "margin_to_3V15_mV": round((vmin - 3.15) * 1e3, 0),
            "keyed_current_A": i_keyed}


# --------------------------------------------------------------------------------------- 8 timing
def timing():
    out = {}
    # heartbeat monostable SN74LVC1G123: tw = K Rext Cext, K = 1.0 (datasheet: 10 k / 0.1 uF -> 1.0-1.1 ms)
    rext, cext = 226e3, 0.22e-6
    tw = rext * cext
    out["monostable"] = {"Rext_ohm": rext, "Cext_F": cext, "tw_nom_ms": round(tw * 1e3, 1),
                         "tw_range_ms": [round(tw * 0.95 * 0.99 * 1e3, 1), round(tw * 1.10 * 1.05 * 1.01 * 1e3, 1)],
                         "heartbeat_period_ms": 10.0, "retrigger_ratio": round(tw / 0.010, 1),
                         "unkey_after_last_edge_ms_max": round(tw * 1.10 * 1.05 * 1.01 * 1e3 + 0.1, 1),
                         "note": "K 1.00-1.10 (datasheet table), C0G +-5 %, R +-1 %; leakage 0.25 uA x 226 k = 57 mV"}
    # key-down timer CD74HC4060 RC oscillator, Q14 high after 2^13 periods; oscillator stopped by a diode from Q14
    n = 2 ** 13
    t_target = 120.0
    f = n / t_target
    ct = 100e-9
    rt = 1.0 / (2.3 * f * ct)
    out["keydown_timer"] = {"stages": "Q14 high after 8192 oscillator periods", "f_osc_Hz": round(f, 2),
                            "Ct_F": ct, "Rt_nom_ohm": round(rt), "Rt_fitted": "56.2 k + select-at-test 0-10 k (R_TOT_TRIM)",
                            "Rs_ohm": 120e3, "formula": "f = 1/(2.3 Rt Ct) (CD4060-family RC oscillator)",
                            "tolerance": "trimmed to 120 +- 1 s at 25 C; drift budget +-3 % (-20..+60 C) => 120 +- 4.6 s",
                            "reset": "MR = NOT TONE_OK: the count restarts each time the tone drops"}
    # CTCSS (CMX138A datasheet): response 190 typ / 250 max ms composite; de-response 240 ms typ
    out["ctcss"] = {"response_ms_typ": 190, "response_ms_max": 250, "deresponse_ms_typ": 240,
                    "key_path_after_TONE_OK_ns": 30,
                    "tnc_relay_latency_ms_max": 2.0,
                    "key_within_ms_typ": 192.0, "key_within_ms_max": 252.0, "requirement_RF_P012_ms": 250.0,
                    "finding": "2 ms over at the CMX138A specification limit; RF-P012 amended to typ <= 200 ms, "
                               "max <= 260 ms (RF_DESIGN 2.1)"}
    # LMX2571 lock after CE: loop bandwidth 40 kHz -> ~4/BW; FastLock < 1.5 ms (datasheet)
    out["synth"] = {"loop_bw_kHz": 40.0, "lock_ms_est": round(4 / 40e3 * 1e3, 2), "fastlock_spec_ms": 1.5,
                    "apc_ramp_ms": 1.0, "txdelay_ms": 300.0}
    # receiver-health monostable (RID RFB-07): SN74LVC1G123, 470 k x 1.0 uF X7R, retriggered by RX_ALIVE every 100 ms
    rh, ch = 470e3, 1.0e-6
    out["rx_health"] = {"Rext_ohm": rh, "Cext_F": ch, "tw_nom_s": round(rh * ch, 3),
                        "tw_range_s": [round(rh * ch * 1.00 * 0.85 * 0.99 * 0.90, 3),
                                       round(rh * ch * 1.10 * 1.10 * 1.01 * 1.0 + 0.25e-6 * rh * ch / 3.1 * 1e0, 3)],
                        "pulse_period_s": 0.100,
                        "fault_after_last_pulse_s_max": round(rh * ch * 1.10 * 1.10 * 1.01 + 0.05, 2),
                        "note": "K 1.00-1.10; X7R -15 %/+10 % (temperature, 3 V bias), R 1 %; ~10 % for 0.25 uA leakage"
                                " x 470 k on the low side; RX_FAULT asserts 0.36-0.62 s after the last RX_ALIVE pulse"}
    # heartbeat stop to unkey: monostable max + gate delays; ICD-003 §6 asks <= 100 ms
    out["heartbeat_to_unkey_ms_max"] = round(out["monostable"]["unkey_after_last_edge_ms_max"] + 1.0, 1)
    return out


# ------------------------------------------------------------------------------- 15 magnetic cleanliness (C16)
def _biot_savart(path, current, z, sensor, step=0.1):
    """Field (T) at sensor (mm) of a current (A) along a polyline path [(x, y), ...] (mm) at height z (mm)."""
    mu0_4pi = 1e-7
    b = np.zeros(3)
    s = np.array(sensor, dtype=float)
    for (x0, y0), (x1, y1) in zip(path[:-1], path[1:]):
        a, c = np.array([x0, y0, z], float), np.array([x1, y1, z], float)
        n = max(2, int(np.linalg.norm(c - a) / step))
        t = (np.arange(n) + 0.5) / n
        pts = a + np.outer(t, c - a)
        dl = (c - a) / n * 1e-3
        r = (s - pts) * 1e-3
        rn = np.linalg.norm(r, axis=1)[:, None]
        b += mu0_4pi * current * np.sum(np.cross(dl, r) / rn ** 3, axis=0)
    return b


def magnetics(pa):
    """Geometry in the stack frame of MECHANICAL_DESIGN §4.1 (mm): RF board substrate z +34.8 ... +36.4, top side up;
    board A2 (RM3100) at (+24, +27), z +40.4 ... +48.4, sensor centre taken at z +44 (OBC_BOARD_DESIGN §9.4)."""
    sensor = (24.0, 27.0, 44.0)
    a2_bottom = 40.4
    top, part_h = 36.4, 2.0
    out = {"sensor_mm": sensor, "A2_bottom_z_mm": a2_bottom}
    # plan radius on the RF board that keeps every part >= 20 mm (3D) from the nearest point of A2 above (24, 27)
    out["plan_radius_for_20mm_3D_mm"] = {"board_surface": round(math.sqrt(20 ** 2 - (a2_bottom - top) ** 2), 1),
                                         "top_of_2mm_parts": round(math.sqrt(20 ** 2 - (a2_bottom - top - part_h) ** 2), 1),
                                         "adopted_plan_radius_mm": 20.0}
    # shield: 84 x 54 x 7 mm at y -40 ... +14 (MECHANICAL_DESIGN §11 item 4): nearest point of wall and lid
    near = (24.0, 14.0, top + 7.0)
    out["shield_nearest_to_sensor_mm"] = round(math.dist(near, sensor), 1)
    # dipole scale: 1 uA.m2 remanent moment, on axis (worst orientation)
    out["field_nT_per_uAm2_on_axis"] = {f"{r} mm": round(1e-7 * 2 * 1e-6 / (r * 1e-3) ** 3 * 1e9, 1)
                                        for r in (13, 20, 30, 40, 60)}
    # PA supply current (1 W level) from J2 to the PA: J2 at (-26, -44) on the -Y strip, FB803, +5V_PA on L4, the
    # drain feeds L602/L604 and Q601 at (-32, -30); return on L5 under the supply (paired) or detoured (unpaired)
    i = pa["levels"]["1 W"]["i_5v_A"]
    supply = [(-26.0, -44.0), (-26.0, -36.0), (-32.0, -30.0)]
    z_l4, z_l5 = 35.3, 35.0          # L4/L5 0.30 mm apart (stack-up rule of §10)
    paired = _biot_savart(supply, i, z_l4, sensor) + _biot_savart(supply[::-1], i, z_l5, sensor)
    detour = [(-32.0, -30.0), (-12.0, -30.0), (-12.0, -44.0), (-26.0, -44.0)]
    unpaired = _biot_savart(supply, i, z_l4, sensor) + _biot_savart(detour, i, z_l5, sensor)
    drain = [(-32.0, -24.0), (-32.0, -30.0)]          # L604 to Q601 on L1, return in the L2 plane 0.2 mm below
    drain_b = _biot_savart(drain, i, top, sensor) + _biot_savart(drain[::-1], i, top - 0.2, sensor)
    # the same paired run if it had been routed past the magnetometer (20 mm in plan from (24, 27))
    near_run = [(4.0, 27.0), (4.0, 7.0)]
    near_b = _biot_savart(near_run, i, z_l4, sensor) + _biot_savart(near_run[::-1], i, z_l5, sensor)
    single_wire_20mm = 2e-7 * i / 0.020
    out["pa_current_A_at_1W"] = i
    out["pa_field_at_sensor_nT"] = {
        "paired_supply_return_L4_L5": round(np.linalg.norm(paired) * 1e9, 2),
        "drain_loop_L1_L2": round(np.linalg.norm(drain_b) * 1e9, 2),
        "unpaired_return_detour_20x14mm": round(np.linalg.norm(unpaired) * 1e9, 1),
        "paired_but_routed_20mm_from_sensor": round(np.linalg.norm(near_b) * 1e9, 1),
        "single_unpaired_wire_at_20mm": round(single_wire_20mm * 1e9, 0),
    }
    out["pa_distance_to_sensor_mm"] = {"J2": round(math.dist((-26, -44, top), sensor), 1),
                                       "Q601": round(math.dist((-32, -30, top), sensor), 1)}
    out["targets_nT"] = {"keyed_minus_unkeyed_change": 15.0, "note": "RM3100 noise 15 nT at 200 cycle counts "
                         "(OBC_BOARD_DESIGN §9.4); the change on keying is what calibration cannot remove"}
    return out


# ----------------------------------------------------------------------------------------- 9 deviation
def deviation(p):
    fpd = 40e6
    prescaler = 2
    chdiv = 32
    fvco = 145.9e6 * chdiv
    n_div = fvco / (fpd * prescaler)
    fs_dev_hz = 6300.0     # ADC full scale (+32767) maps to +6.3 kHz
    lsb = fs_dev_hz / 32767
    den = fpd * prescaler / chdiv / lsb
    # audio levels into the ADC (full scale 0.6 VCC p-p)
    vcc = p["vdd_pa"]
    fs_vpp = 0.6 * vcc
    cmx_max_vpp = 2 * math.sqrt(2) * 0.838 * (3.10 / 3.30)   # CMX138A max undistorted output, scales with AVDD
    v_repeat_limit_khz = cmx_max_vpp / fs_vpp * fs_dev_hz / 1e3
    afsk_vpp = 3.0 / (fs_dev_hz / 1e3) * fs_vpp
    tol = math.sqrt(p["vdd_pa_tol"] ** 2 + 0.015 ** 2 + 0.002 ** 2)
    worst = p["vdd_pa_tol"] + 0.015 + 0.002
    return {"fPD_MHz": fpd / 1e6, "prescaler": prescaler, "chdiv": chdiv, "fVCO_MHz": round(fvco / 1e6, 3),
            "N": round(n_div, 6), "full_scale_dev_kHz": fs_dev_hz / 1e3, "dev_per_lsb_Hz": round(lsb, 4),
            "DEN": int(round(den)), "DEN_max": 2 ** 24 - 1,
            "adc_fullscale_Vpp": round(fs_vpp, 3), "cmx_max_Vpp": round(cmx_max_vpp, 3),
            "repeat_limit_kHz_nom": round(v_repeat_limit_khz, 2),
            "repeat_limit_kHz_worst_high": round(v_repeat_limit_khz * (1 + worst), 2),
            "afsk_level_Vpp_for_3kHz": round(afsk_vpp, 3),
            "afsk_dev_kHz_range": [round(3.0 * (1 - worst), 2), round(3.0 * (1 + worst), 2)],
            "carson_afsk_kHz": round(2 * (2.2 + 3.0 * (1 + worst)), 2),
            "carson_voice_kHz_worst": round(2 * (3.0 + v_repeat_limit_khz * (1 + worst)), 2),
            "pcm1808_mode": "master 512 fs from the 40 MHz reference", "pcm1808_fs_kHz": round(40e6 / 512 / 1e3, 3),
            "bck_MHz": round(40e6 / 512 * 64 / 1e6, 4)}


# ------------------------------------------------------------------------------ 10 detectors and APC
def detectors(p):
    # sense line after the LPF: coupling computed for a 45 mm line, 0.15 mm gap, on the 0.2 mm prepreg (§9)
    # coupled-line model: |C| ~ k * sin(beta l), k for edge-coupled microstrip at s/h = 0.75 [estimate]
    k = 0.22
    eps_eff = 3.3
    lam = 3e8 / 145.9e6 / math.sqrt(eps_eff)
    l = 0.045
    c_db = 20 * math.log10(k * math.sin(2 * math.pi * l / lam))
    # AD8361 (ground-referenced mode): Vout = 7.5 x Vrms_in. 2.5 V at 1 W at the coupler => 0.333 Vrms at its input
    vin_fs = 2.5 / 7.5
    pin_dbm = 10 * math.log10(vin_fs ** 2 / 50 * 1e3)
    pad = 30.0 + c_db - pin_dbm
    v05 = 2.5 * math.sqrt(0.5)
    vrev_3to1 = 2.5 * 0.5
    return {"sense_line_coupling_dB": round(c_db, 1), "detector_fs_input_dBm": round(pin_dbm, 1),
            "pad_needed_dB": round(pad, 1), "pad_fitted": "select-at-test pi pad 0-4 dB (nominal 1 dB: 5.6 ohm series, 910 ohm shunts)",
            "TX_FWD_at_1W_V": 2.5, "TX_FWD_at_0.5W_V": round(v05, 3), "TX_REV_at_3to1_at_1W_V": vrev_3to1,
            "adc_rs_scale": "P = 0.16 V^2 W (firmware adc.rs SquareLaw(0.16))",
            "apc_ref_hi_V": 2.500, "apc_ref_lo_V": round(2.5 * 24.1 / (10.0 + 24.1), 4),
            "apc_ref_lo_target_V": round(v05, 4),
            "apc_power_error_dB": round(20 * math.log10(1 + 0.0005 + 0.002 + 0.0025), 3),
            "detector_temp_error_dB": 0.25,
            "TX_TEMP": "TMP235A2: 10 mV/C, 500 mV at 0 C, +-2 C (-40..150 C); 1.2 V at 70 C"}


# ------------------------------------------------------------------------------ 11 frequency reference
def reference():
    temp = 0.5      # TG2016SMN +-0.5 ppm over -40..+85 C
    cal = 0.1       # residual after acceptance-test calibration (CC1200 FREQOFF and LMX2571 NUM written by the TNC)
    aging_y1 = 1.0  # quartz TCXO aging, first year, maximum class (datasheet value to confirm) [assumption]
    aging_y2 = 0.5
    bol = temp + cal
    eol_worst = temp + cal + aging_y1 + aging_y2
    eol_rss = math.sqrt(temp ** 2 + cal ** 2 + aging_y1 ** 2 + aging_y2 ** 2)
    return {"temp_ppm": temp, "cal_ppm": cal, "aging_y1_ppm": aging_y1, "aging_y2_ppm": aging_y2,
            "bol_ppm": bol, "eol_worst_ppm": eol_worst, "eol_rss_ppm": round(eol_rss, 2),
            "hz_at_435_5_bol": round(bol * 435.5, 0), "hz_at_435_5_eol_worst": round(eol_worst * 435.5, 0),
            "hz_at_145_9_bol": round(bol * 145.9, 0),
            "note": "One TCXO for both bands: the ground measures the downlink error and scales it by 435.5/145.9 "
                    "to pre-correct the uplink; the CC1200 frequency-offset estimate and AFC track the rest"}


# ------------------------------------------------------------------------------- 12 isolation budget
def isolation(p, fl1):
    p_fund = p["p_tx_max_dbm"] - p["ant_iso_146_db"]
    at_lna = p_fund - fl1["worst"]["att_144_146_dB"]
    at_ax = at_lna + p["lna_gain_db"] - 45.0
    return {"fund_at_uhf_port_dBm": p_fund, "fl1_worst_dB": fl1["worst"]["att_144_146_dB"],
            "fund_at_lna_dBm": round(at_lna, 1), "lba_requirement_at_lna_dBm": -55.0,
            "fund_at_cc1200_dBm": round(at_ax, 1), "cc1200_saturation_dBm": 10.0, "cc1200_iip3_dBm": -14.0,
            "board_isolation_required_dB": 85.0,
            "board_isolation_basis": "PA output (+30 dBm) to the LNA input node must stay below -55 dBm, the level "
                                     "the RX filter guarantees; two shield cans, opposite board faces (§9)"}


# ------------------------------------------------------------------------- 13 clock harmonics vs uplink
def clocks(p):
    fu0, fu1 = p["f_up_band_mhz"]
    clk = {"TCXO 40 MHz (CC1200, LMX2571 PFD, PCM1808 SCKI)": 40.0, "PCM1808 BCK 64 fs = 5.000 MHz": 5.0,
           "PCM1808 internal 128 fs = 10 MHz": 10.0, "CMX138A crystal 3.6864 MHz": 3.6864,
           "(rejected) CMX138A 6.144 MHz": 6.144, "(rejected) 38.4 MHz reference, BCK 4.8 MHz": 4.8,
           "(rejected) 26 MHz reference at 512 fs, BCK 3.25 MHz": 3.25,
           "MCU HSI16 16 MHz (core, APB, ADC)": 16.0, "MCU SPI1 at 8 MHz (CC1200 CFM reads, LMX2571)": 8.0,
           "(occasional) SPI1 at 1 MHz for the CMX138A C-BUS, < 1 ms per 10 s": 1.0}
    out = {}
    for name, f in clk.items():
        hits = []
        n0 = math.ceil(fu0 / f)
        n = n0
        while n * f <= fu1 + 1e-9:
            hits.append(round(n * f, 4))
            n += 1
        nearest = [round(math.floor(fu0 / f) * f, 4), round(math.ceil(fu1 / f) * f, 4)]
        out[name] = {"harmonics_in_435.0_437.3": hits, "nearest_outside": nearest}
    return out


# ------------------------------------------------------------------------------- 14 stresses / derating
def stresses(p, pa, lpf):
    """Worst-case stress for the parts whose ratings matter. Rule: NASA EEE-INST-002 Level 3 as adapted in
    EEE_PARTS_LIST Rev C §12 (resistors 50 % power, MLCC 50 % voltage, transistors 50 % power, regulators 70 %
    current, IC junction <= 80 % of maximum; +10 % hot-case allowance not taken)"""
    vswr = 3.0
    p1 = 1.0
    v_peak_50 = math.sqrt(2 * p1 * 50.0)
    v_peak_vswr = v_peak_50 * (1 + (vswr - 1) / (vswr + 1))
    # inside the LPF the node voltage can exceed the port voltage; ABCD estimate of the worst node: x1.6
    v_node = v_peak_vswr * 1.6
    vdd_max = p["vdd_pa"] * (1 + p["vdd_pa_tol"])
    # drain peak voltage of a saturated stage ~ 2 x Vdd, x1.5 under 3:1 mismatch
    v_drain_pk = 2 * vdd_max * 1.5
    lv = pa["levels"]["1 W"]
    s = {
        "RF_caps_LPF": {"stress": f"{v_node:.1f} V peak at 1 W into 3:1 (port {v_peak_vswr:.1f} V x 1.6 node factor)",
                        "rating": "100 V C0G", "ratio": round(v_node / 100.0, 2), "rule": "<= 0.50 (MLCC voltage)"},
        "RF_caps_match": {"stress": f"{v_drain_pk:.1f} V peak at the drain side", "rating": "100 V C0G",
                          "ratio": round(v_drain_pk / 100.0, 2), "rule": "<= 0.50"},
        "RD01MUS3_VDS": {"stress": f"{v_drain_pk:.1f} V peak", "rating": "25 V VDSS", "ratio": round(v_drain_pk / 25.0, 2),
                         "rule": "<= 0.75 of VDSS for RF peak (EEE-INST-002 FET RF voltage; project 50 % applies to DC: "
                                 f"{vdd_max:.2f}/25 = {vdd_max / 25:.2f})"},
        "RD01MUS3_ID2": {"stress": f"{lv['dc_pa_W'] / p['vdd_pa']:.2f} A average", "rating": "0.9 A",
                         "ratio": round(lv["dc_pa_W"] / p["vdd_pa"] / 0.9, 2), "rule": "<= 0.50"},
        "RD01MUS3_PD2": {"stress": f"{0.9 * lv['pa_dissipation_W']:.2f} W", "rating": "8.3 W at Tc 25 C (derate to Tc 60 C: "
                         f"{8.3 * (150 - 60) / (150 - 25):.1f} W)",
                         "ratio": round(0.9 * lv["pa_dissipation_W"] / (8.3 * (150 - 60) / 125), 2), "rule": "<= 0.50"},
        "RD01MUS3_Tj": {"stress": f"{lv['tj_fet2_C_at_board_60C']} C", "rating": "150 C (vendor: keep <= 120 C)",
                        "ratio": round(lv["tj_fet2_C_at_board_60C"] / 150.0, 2), "rule": "<= 0.80 of Tj max (120 C)"},
        "RD01MUS3_Pin1": {"stress": "1 mW (0 dBm, LMX2571 maximum output)", "rating": "2 mW", "ratio": 0.5,
                          "rule": "<= 0.50"},
        "LPF_inductors": {"stress": f"{math.sqrt(2 * p1 / 50 * vswr):.2f} A peak (1 W into 3:1, current maximum)",
                          "rating": "1812SMS Irms >= 2.5 A [series data]", "ratio": round(math.sqrt(p1 / 50 * vswr) / 2.5, 2),
                          "rule": "<= 0.50"},
        "drain_choke": {"stress": f"{lv['dc_pa_W'] / p['vdd_pa']:.2f} A DC", "rating": "1812SMS 2.5 A", "ratio":
                        round(lv["dc_pa_W"] / p["vdd_pa"] / 2.5, 2), "rule": "<= 0.50"},
        "pi_pad_R": {"stress": "1 W x -31 dB = 0.8 mW", "rating": "0.1 W (0603)", "ratio": 0.01, "rule": "<= 0.50"},
        "TPS7A2030_I": {"stress": "30 mA worst (CC1200 21, LNA 7, TCXO 2)", "rating": "300 mA", "ratio": 0.10, "rule": "<= 0.70"},
        "TPS73101_I": {"stress": "14 mA worst (MCU, CMX138A, logic)", "rating": "150 mA", "ratio": 0.09, "rule": "<= 0.70"},
        "TPS7A2033_I": {"stress": "17 mA (PCM1808 VDD 5.9, detectors 2.2, squarer and gates 4, margin)", "rating": "300 mA",
                        "ratio": 0.06, "rule": "<= 0.70",
                        "thermal": "(5.15-3.3) V x 17 mA = 31 mW; X2SON thetaJA ~ 200 C/W => +6 C"},
        "TPS22918_LMX": {"stress": "51 mA", "rating": "2 A", "ratio": 0.03, "rule": "<= 0.70"},
        "bulk_caps_5V": {"stress": f"{vdd_max:.2f} V", "rating": "16 V X7R", "ratio": round(vdd_max / 16, 2), "rule": "<= 0.50"},
        "bulk_caps_3V3": {"stress": "3.40 V", "rating": "10 V X5R/X7R", "ratio": 0.34, "rule": "<= 0.50"},
        "connector_PA_contacts": {"stress": f"{lv['i_5v_A']:.2f} A over two contacts", "rating": "Pico-Clasp 1.0 A/contact",
                                  "ratio": round(lv["i_5v_A"] / 2 / 1.0, 2), "rule": "<= 0.50 (EEE-INST-002 connectors)"},
        "LMX2571_Tj": {"stress": f"{p['t_board_hot_c'] + 0.152 * 31:.0f} C (0.152 W x 31 C/W thetaJA est.)",
                       "rating": "125 C", "ratio": round((p["t_board_hot_c"] + 0.152 * 31) / 125, 2), "rule": "<= 0.80"},
    }
    return s


# ------------------------------------------------------------------------- 16 antenna ports (Version C)
def rf_ports(p):
    """Board MMCX jacks J3-J6 to the AntS MMCX ports (W10 v3). Route in the stack frame of MECHANICAL_DESIGN §4.1 (mm):
    plug exit at x -39.9, z +39.4; bend R 7 over the board edge into the -X wall gap (x -45.4 ... -48.4) at
    |y| = 20.2 ... 23.8; down the gap to z -39.2; bend R 7 inboard to z -46.2 above the -Z end frame; on to the AntS
    port (position open, ISIS ICD)."""
    r_bend = 7.0                                     # >= 5 x OD of the 1.37 mm cable (static)
    arc = math.pi / 2 * r_bend
    seg = {"plug_and_exit": 5.0, "top_bend": round(arc, 1), "wall_gap_z_+32.4_to_-39.2": 71.6,
           "bottom_bend": round(arc, 1), "AntS_plug": 5.0}
    fixed = sum(seg.values())
    out = {"route_mm": seg, "end_frame_to_AntS_port_mm": [5.0, 75.0],
           "length_needed_mm": [round(fixed + 5.0), round(fixed + 75.0)], "length_budgeted_m": p["cab_len_m"],
           "bend_radius_mm": r_bend, "cable_od_mm": 1.37,
           "band_fill": "two cables per band side by side: 2 x 1.37 = 2.74 mm of the 4.0 mm band (y 20.0 ... 24.0), "
                        "1.37 of the 3.3 mm gap depth"}

    def gamma(vswr):
        return (vswr - 1) / (vswr + 1)

    def ml(g):
        return -10 * math.log10(1 - g ** 2)
    for band, f in (("2m", 145.9e6), ("70cm", 436.0e6)):
        a = p["cab137_db_per_m_2g4"] * math.sqrt(f / 2.4e9) * p["cab_hot_factor"]
        cab = a * p["cab_len_m"]
        con = 2 * p["mmcx_il_db_per_sqrtGHz"] * math.sqrt(f / 1e9)
        g_c, g_k = gamma(p["mmcx_vswr"]), gamma(p["cab_vswr"])
        g_worst = 2 * g_c + 2 * g_k                  # four reflections adding in phase
        g_rss = math.sqrt(2 * g_c ** 2 + 2 * g_k ** 2)
        out[band] = {"cable_dB_per_m_hot": round(a, 2), "cable_dB": round(cab, 2), "connectors_dB": round(con, 2),
                     "return_loss_worst_dB": round(-20 * math.log10(g_worst), 1),
                     "return_loss_rss_dB": round(-20 * math.log10(g_rss), 1),
                     "mismatch_loss_worst_dB": round(ml(g_worst), 3), "mismatch_loss_rss_dB": round(ml(g_rss), 3),
                     "port_to_antenna_worst_dB": round(cab + con + ml(g_worst), 2)}
    # SQSO-LBA-001 §2.4 carries 1.0 dB "filter, cable and connector loss" after the transmitter power. This board
    # sets its power at the board port (APC on the detector after the LPF), so that term is the port-to-antenna loss
    out["lba_tx_loss_term_dB"] = 1.0
    out["tx_margin_vs_lba_dB"] = round(1.0 - out["2m"]["port_to_antenna_worst_dB"], 2)
    out["rx_coax_dB_used_in_section_1"] = out["70cm"]["port_to_antenna_worst_dB"]
    out["previous_W10_v2_value_dB"] = 0.25
    return out


# ------------------------------------------------------------- 17 shield-can vents (D amended, P3A-06)
def shield_vents():
    """One 1.5 mm vent per cavity lid (STRUCTURAL_ANALYSIS §11). Leakage of a small round hole in a thin wall:
    aperture term 20 log(lambda / 2d) plus the below-cutoff waveguide term of the lid thickness, 32 dB per diameter
    of depth (TE11, f << fc). Venting: CDS §2.1.9 V/A < 50.8 m."""
    c = 299792458.0
    d, t = 1.5e-3, 0.20e-3
    fc = 1.841 * c / (math.pi * d)
    out = {"hole_d_mm": 1.5, "lid_t_mm": 0.2, "te11_cutoff_GHz": round(fc / 1e9, 0), "attenuation_dB": {}}
    for name, f in (("146 MHz", 145.9e6), ("437 MHz (H3)", 437.7e6), ("2.4 GHz", 2.4e9)):
        lam = c / f
        out["attenuation_dB"][name] = {"aperture": round(20 * math.log10(lam / (2 * d)), 1),
                                       "depth": round(32.0 * t / d, 1),
                                       "total": round(20 * math.log10(lam / (2 * d)) + 32.0 * t / d, 1)}
    a = math.pi * (d * 1e3 / 2) ** 2
    cav = {"receive": 37 * 22 * 7, "transmit": 37 * 32 * 7 - 12 * 12 * 7, "digital": 47 * 54 * 7 - 12 * 14 * 7}
    out["vent_area_mm2"] = round(a, 2)
    out["V_over_A_m"] = {k: round(v / a / 1e3, 1) for k, v in cav.items()}
    out["V_over_A_limit_m"] = 50.8
    # hole path from the PA (1 W) to the LNA input region at 146 MHz: out of one can and into the other
    a146 = out["attenuation_dB"]["146 MHz"]["total"]
    out["pa_to_lna_via_two_vents_dB"] = round(2 * a146 + 20.0, 0)   # + 20 dB free-space/near-field coupling [assumed]
    out["pa_to_lna_requirement_dB"] = 85.0
    return out


# --------------------------------------------------------------------------------------------- main
def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mc", type=int, default=3000)
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args()
    rng = np.random.default_rng(20261027)
    p = dict(INPUTS)
    res = {"inputs": {k: (v if not isinstance(v, dict) else {str(x): y for x, y in v.items()}) for k, v in p.items()}}
    res["fl1"] = design_fl1(p, a.mc, rng)
    p["_fl1_il_worst_db"] = res["fl1"]["worst"]["il_dB"]
    res["rf_ports"] = rf_ports(p)
    p["coax_uhf_db"] = res["rf_ports"]["70cm"]["port_to_antenna_worst_dB"]
    res["receiver"] = receiver(p)
    res["lpf"] = design_lpf(p, a.mc, rng)
    res["harmonics"] = harmonics(p, res["lpf"])
    res["desense"] = desense(p, res["harmonics"]["H3"]["with_leak_floor_dbc"])
    res["pa"] = pa_budget(p, res["lpf"])
    res["rx_power"] = rx_power(p)
    res["timing"] = timing()
    res["deviation"] = deviation(p)
    res["detectors"] = detectors(p)
    res["reference"] = reference()
    res["isolation"] = isolation(p, res["fl1"])
    w = res["fl1"]["worst"]
    res["rx_filter"] = {
        "pre_lna_FL1_plus_balun_loss_worst_dB": round(w["il_dB"] + p["balun_uhf_db"], 2), "requirement_loss_dB": 1.5,
        "att_144_146_computed_worst_dB": w["att_144_146_dB"], "att_144_146_specified_dB": 60.0,
        "att_144_146_without_traps_dB": 69.4,
        "att_250_330_total_worst_dB": round(w["att_250_330_dB"] + 45.0, 1),
        "att_560_620_total_worst_dB": round(w["att_560_620_dB"] + 45.0, 1),
        "saw": "TA2209A: >= 45 dB at 10-385 and 485-1000 MHz, 2.1 dB typ / 4.2 dB max at 433-437 MHz, "
               "TCF -36 ppm/K (-15.7 kHz/K at 435 MHz)",
        "saw_center_shift_kHz_-30_to_+60C": [round(-36e-6 * 435e6 * (-30 - 25) / 1e3, 0),
                                              round(-36e-6 * 435e6 * (60 - 25) / 1e3, 0)],
        "lna_im2_from_two_-90dBm_signals_dBm": round(2 * (-90.0) - 20.0, 0),
    }
    res["clocks"] = clocks(p)
    res["stresses"] = stresses(p, res["pa"], res["lpf"])
    res["magnetics"] = magnetics(res["pa"])
    res["shield_vents"] = shield_vents()
    tx_loss = res["rf_ports"]["2m"]["port_to_antenna_worst_dB"]
    res["rf_ports"]["tx_1W_port_dBm_nominal_low_high"] = [r["pmax_port_dBm"] for r in res["pa"]["psat"]]
    res["rf_ports"]["tx_1W_at_antenna_dBm_nominal_low_high"] = [round(r["pmax_port_dBm"] - tx_loss, 2)
                                                           for r in res["pa"]["psat"]]
    (HERE / "rf_calcs.json").write_text(json.dumps(res, indent=1, default=float) + "\n")
    if not a.quiet:
        for k in ("fl1", "receiver", "lpf", "harmonics", "pa", "rx_power", "timing", "deviation", "detectors",
                  "reference", "isolation", "magnetics", "rf_ports"):
            print(f"== {k}")
            print(json.dumps(res[k], indent=1, default=float))
        print("== desense (2.2 MHz rows)")
        for r in res["desense"]["rows"]:
            if r["offset_MHz"] in (1.0, 2.2):
                print(r)
        print("== clocks")
        for k, v in res["clocks"].items():
            print(k, v)
        print("== stresses")
        for k, v in res["stresses"].items():
            print(k, v)


if __name__ == "__main__":
    main()
