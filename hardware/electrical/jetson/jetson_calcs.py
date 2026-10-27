#!/usr/bin/env python3
"""SMART-QSO Jetson carrier board: design calculations (SQSO-JET-BRD-001, CARRIER_BOARD_DESIGN.md).

Run with any Python 3 (numpy not required):

    python3 hardware/electrical/jetson/jetson_calcs.py

Prints the calculation tables of CARRIER_BOARD_DESIGN.md and writes jetson_calcs.json next to this file.
Every input is named with its source. Derating: NASA EEE-INST-002 (as tailored by SQSO-HAR-001 5.2) with the
stricter of it and the common 50 % rule (EPS_BOARD_DESIGN 2.3).
"""

import cmath
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = {}


def table(title, head, rows):
    print(f"\n### {title}\n")
    print("| " + " | ".join(head) + " |")
    print("|" + "|".join("---" for _ in head) + "|")
    for r in rows:
        print("| " + " | ".join(str(x) for x in r) + " |")


def par(*zs):
    return 1.0 / sum(1.0 / z for z in zs)


# ------------------------------------------------------------------------------------------------ inputs
VBAT_MIN = 11.2       # BMS cut, BASELINE 5 / brief
VBAT_NOM = 14.4
VBAT_MAX = 16.4       # 4.10 V/cell, BASELINE 5 [D8]
VBAT_ABS = 18.4       # EPS U19 OVP (EPS design.py: 1.43 M / 100 k): the highest voltage that can reach the carrier
VBAT_LOWBUS = 13.4    # the EPS designer's low-bus point for current (eps_calcs.py)
EFUSE_EPS = (1.10, 1.20, 1.30)  # TPS16630 trip, A (EPS_BOARD_DESIGN 8.3)
EPS_SR = 2e-6 * 25 / 10e-9      # TPS16630 dVdT: I 2 uA x gain 25 / 10 nF = V/s (TPS1663 data sheet 7.5)
P_MOD_7W = 7.0        # nvpmodel 7W power budget, Orin Nano 8GB, original flash configuration (Jetson Linux r36.4.4)
P_MOD_CAP = 6.5       # proposed VDD_IN average-power cap (INA3221 ch.1 curr1_max = 1300 mA at 5.0 V)
VDD_IN_MIN, VDD_IN_MAX, VDD_IN_ABS = 4.75, 5.25, 5.5  # DS-11105-001 v1.5 Tables 5-1, 5-2 (MODULE_ID low = Nano)
FSW = 1.000e6         # carrier converters synchronised to Y1 (EMC rule E1 frequency plan, EPS_BOARD_DESIGN 8.2)

# ------------------------------------------------------------------------------------------------ 1 5 V buck U1
# LM61440-Q1 (SNVSBE4C): VFB 1.000 V +/-1 %; RDS(on) HS 41 (82 max) mOhm, LS 21 (45 max) mOhm; IL-HS 6-8.1 A;
# tON,min 70 ns max; EN 1.263 V +/-5 %, hysteresis 28 % typ (24-32 %); fsw by RT 200 kHz-2.2 MHz, sync same range.
RFBT, RFBB = 100e3, 24.9e3
V5 = 1.0 * (1 + RFBT / RFBB)
v5_lo = 0.99 * (1 + RFBT * 0.999 / (RFBB * 1.001))
v5_hi = 1.01 * (1 + RFBT * 1.001 / (RFBB * 0.999))
RRT = (1 / (FSW / 1e3) - 3.3e-5) * 1.346e4  # kOhm, data sheet Eq. 2
RENT, RENB = 698e3, 100e3
U1_ON = 1.263 * (1 + RENT / RENB)
U1_OFF = 1.263 * 0.72 * (1 + RENT / RENB)
d_max = V5 / VBAT_MIN
d_min = V5 / VBAT_ABS
ton_min = d_min / FSW
L1_H = 3.3e-6
ripple = [(VBAT_MAX - V5) / (FSW * L1_H) * V5 / VBAT_MAX, (VBAT_ABS - V5) / (FSW * L1_H) * V5 / VBAT_ABS]
L_eq9 = (VBAT_MAX - V5) / (FSW * 0.3 * 4.0) * V5 / VBAT_MAX
L_eq10 = 0.5 * V5 / (FSW / 1e6) * 1e-6
rows = [("Output (RFBT 100 k / RFBB 24.9 k, 0.1 %)", f"{V5:.3f} V ({v5_lo:.3f}-{v5_hi:.3f})", "VDD_IN 4.75-5.25 V after U2 and the connector"),
        ("RT for 1.000 MHz (Eq. 2)", f"{RRT:.1f} k -> 13.0 k (0.1 %)", "free-running fallback 0.90-1.10 MHz"),
        ("EN UVLO (698 k / 100 k)", f"on {U1_ON:.2f} V, off {U1_OFF:.2f} V", "starts above 10 V, below the 11.2 V BMS cut (rule E8)"),
        ("Duty range", f"{d_min:.3f}-{d_max:.3f}", ""),
        ("Shortest on-time at 18.4 V", f"{ton_min * 1e9:.0f} ns", "> 70 ns tON,min: no pulse skipping"),
        ("Inductance, Eq. 9 (K 0.3 of 4 A) / Eq. 10", f"{L_eq9 * 1e6:.2f} / {L_eq10 * 1e6:.2f} uH", "3.3 uH selected (XGL6030-332MEC)"),
        ("Ripple at 16.4 / 18.4 V", f"{ripple[0]:.2f} / {ripple[1]:.2f} A p-p", "Isat >= IL-HS 8.1 A required by the data sheet")]
table("1. 5 V buck U1 (LM61440AANQRJRRQ1)", ["Item", "Value", "Note"], rows)
OUT["u1"] = dict(v5=V5, v5_range=[v5_lo, v5_hi], rt_k=RRT, on=U1_ON, off=U1_OFF, ton_min_ns=ton_min * 1e9,
                 ripple_pp=ripple)

# losses: typical model fitted to the data-sheet efficiency points (5 V out, 4 A: 93 % at 2.1 MHz, 95 % at 400 kHz,
# VIN 13.5 V): P = I^2 (D RHS + (1-D) RLS + DCR) + kSW f I VIN + P0 f/2.1MHz; kSW = 7.05e-9 s, P0 = 0.10 W.
DCR_L1 = 0.0115


def u1_loss(p_out, vin, hot=False):
    i = p_out / V5
    d = V5 / vin
    rhs, rls = (0.082, 0.045) if hot else (0.041, 0.021)
    dcr = DCR_L1 * (1.4 if hot else 1.0)
    return i * i * (d * rhs + (1 - d) * rls + dcr) + 7.05e-9 * FSW * i * vin + 0.10 * FSW / 2.1e6 + 0.01


# ------------------------------------------------------------------------------------------------ 2 VDD_IN eFuse U2
# TPS259474LRPWR (SLVSFC9C): UVLO/OVLO/PGTH rising 1.20 V (1.183-1.223), falling 1.09 V; RILM = 3334 / ILIM;
# ILIM +/-10 % (> 1 A); ITIMER: t = dV x C / I, dV 1.51 (1.286-1.741) V, I 1.8 (1.2-2.5) uA; CdVdt(pF) = 2000 / SR(V/ms);
# RON 28.3 typ, 45 mOhm max; circuit breaker 2 us after the timer; latch-off (L).
def div(vth, rt, rb):
    return vth * (1 + rt / rb)


RILM = 1.33e3
ILIM = 3334 / RILM
uvlo = (div(1.20, 28.7e3, 10e3), div(1.09, 28.7e3, 10e3))
ovlo = (div(1.183, 34.8e3, 10e3), div(1.20, 34.8e3, 10e3), div(1.223, 34.8e3, 10e3))
pgth = (div(1.20, 28.7e3, 10e3), div(1.09, 28.7e3, 10e3))
CIT = 2.2e-9
t_it = (1.286 * CIT / 2.5e-6, 1.51 * CIT / 1.8e-6, 1.741 * CIT / 1.2e-6)
SR_VDD = 1.0  # V/ms
CDVDT = 2000 / SR_VDD
C_VDD = 6 * 22e-6 * 0.8 + 100e-6  # carrier 6 x 22 uF (80 % at 5 V) + module input capacitance (assumed 100 uF)
I_INR_VDD = SR_VDD * 1e3 * C_VDD
P_INR = I_INR_VDD * V5 / 2
i_mod_7w = P_MOD_7W / 4.95
rows = [("Circuit-breaker threshold (RILM 1.33 k)", f"{ILIM:.2f} A ({ILIM * 0.9:.2f}-{ILIM * 1.1:.2f})", f"7 W mode average {i_mod_7w:.2f} A: {ILIM * 0.9 / i_mod_7w:.2f}x at the minimum"),
        ("Fast trip", f"{2 * ILIM:.1f} A (2 x ILIM), 22 A fixed", "500 ns"),
        ("Blanking ITIMER 2.2 nF", f"{t_it[1] * 1e3:.2f} ms ({t_it[0] * 1e3:.2f}-{t_it[2] * 1e3:.2f})", "load transients shorter than this pass"),
        ("EN/UVLO from 5V_BUCK (28.7 k / 10 k)", f"on {uvlo[0]:.2f} V, off {uvlo[1]:.2f} V", ""),
        ("OVLO (34.8 k / 10 k)", f"{ovlo[1]:.2f} V ({ovlo[0]:.2f}-{ovlo[2]:.2f}), 1.2 us", f"VDD_IN abs. max {VDD_IN_ABS} V; U1 maximum {v5_hi:.2f} V"),
        ("PGTH on VDD_IN (28.7 k / 10 k)", f"good above {pgth[0]:.2f} V, bad below {pgth[1]:.2f} V", "PG releases POWER_EN"),
        ("dVdt 2.2 nF", f"{2000 / 2200:.2f} V/ms, ramp {V5 / (2000 / 2200):.1f} ms", ""),
        ("Inrush into VDD_IN (carrier 106 uF + module 100 uF assumed)", f"{I_INR_VDD * 2000 / 2200 / SR_VDD:.2f} A", f"{P_INR:.2f} W in U2 for 5.5 ms"),
        ("Conduction loss at 7 W (45 mOhm max)", f"{(P_MOD_7W / 4.95) ** 2 * 0.045 * 1e3:.0f} mW", "")]
table("2. VDD_IN eFuse U2 (TPS259474LRPWR)", ["Item", "Value", "Note"], rows)
OUT["u2"] = dict(ilim=ILIM, t_itimer_ms=[t * 1e3 for t in t_it], uvlo=uvlo, ovlo=ovlo, pgth=pgth)

# VDD_IN voltage at the module: static and transient
drop = lambda i: i * (0.045 + 0.003 + 0.005)  # U2 max, 10 SO-DIMM contacts, plane
v_mod_lo = v5_lo - drop(2.0) - 0.10   # 2 A peak, 100 mV load-step undershoot
v_mod_hi = v5_hi + 0.10
rows = [("Static at 2.0 A (U2 45 mOhm, connector 3 mOhm, plane 5 mOhm)", f"{v5_lo - drop(2.0):.3f}-{v5_hi:.3f} V", "4.75-5.25 V"),
        ("With +/-100 mV load-step excursion", f"{v_mod_lo:.3f}-{v_mod_hi:.3f} V", "inside 4.75-5.25 V" if v_mod_lo >= 4.75 and v_mod_hi <= 5.25 else "OUTSIDE"),
        ("OVLO margin above the highest normal VDD_IN", f"{ovlo[0] - v_mod_hi:.2f} V", "")]
table("3. VDD_IN at the module (DS-11105-001 Table 5-1)", ["Case", "VDD_IN", "Requirement"], rows)
OUT["vdd_in"] = dict(lo=v_mod_lo, hi=v_mod_hi)

# ------------------------------------------------------------------------------------------------ 3 3.3 V and SSD
# LMR43620MC3: 3.27-3.33 V fixed, peak limit 2.8-3.9 A, synced. Loads: SSD (Swissbit N2600 2230, 3.3 V only; no power
# figures published: allocation 0.30 W average over a burst, 2.0 W peak during boot reads), JET_3V3 (<= 1 mA), U9-U11,
# Y1 (SG-8018CA <= 4 mA at 1 MHz, allocation), U12.
P_SSD_AVG, P_SSD_PK = 0.30, 2.0
P_3V3_MISC = 3.3 * (0.001 + 0.004 + 0.0005 + 0.0002 + 0.001)
I3_PK = (P_SSD_PK + P_3V3_MISC) / 3.3
eta3_light, eta3_pk = 0.80, 0.88  # FPWM at 0.1 A / 0.6 A, 1 MHz (allocation from the family's efficiency curves)
rows = [("3V3_PRE average load", f"{(P_SSD_AVG + P_3V3_MISC):.3f} W", f"input {(P_SSD_AVG + P_3V3_MISC) / eta3_light:.3f} W at 80 %"),
        ("3V3_PRE peak load", f"{I3_PK:.2f} A", f"U3 limit 2.8 A min: {2.8 / I3_PK:.1f}x"),
        ("U6 load switch TPS22918 (2 A at 70 C)", f"{I3_PK:.2f} A", f"{I3_PK / 2.0:.0%} of rating"),
        ("JET_3V3 to the OBC (E9)", "U28/U29 VCCB + 10 k/100 k JET_PG: 33 uA + 2 x 10 uA", "<= 1 mA")]
table("4. 3.3 V rails", ["Item", "Value", "Note"], rows)

# ------------------------------------------------------------------------------------------------ 4 power budget
def budget(p_mod, vin, hot, eta_bound=None):
    pe = (p_mod / 4.95) ** 2 * (0.045 if hot else 0.028)
    p5 = p_mod + pe + 0.001  # + the POWER_EN pull-up and dividers on 5 V
    l1 = u1_loss(p5, vin, hot) if eta_bound is None else p5 * (1 / eta_bound - 1)
    p3 = (P_SSD_AVG + P_3V3_MISC) / (eta3_light if hot else 0.85)
    i_in = (p5 + l1 + p3) / vin
    pf = i_in ** 2 * 0.030 + vin * 2e-6  # filter inductor DCR, TVS leakage
    return dict(module=p_mod, efuse=pe, u1_loss=l1, eta_u1=p5 / (p5 + l1), rail_3v3_in=p3, filt=pf,
                total=p5 + l1 + p3 + pf, i_in=i_in)


rows = []
for name, pm in (("7 W mode at its full budget", P_MOD_7W), ("capped at 6.5 W (INA3221 curr1_max 1300 mA)", P_MOD_CAP)):
    for vin, hot in ((VBAT_MIN, True), (VBAT_NOM, False), (VBAT_MAX, True)):
        b = budget(pm, vin, hot)
        rows.append((name, f"{vin:.1f} V {'hot' if hot else 'typ'}", f"{b['module']:.2f}", f"{b['efuse'] * 1e3:.0f} mW",
                     f"{b['u1_loss']:.2f} W ({b['eta_u1']:.1%})", f"{b['rail_3v3_in']:.2f}", f"{b['total']:.2f} W",
                     f"{(8.0 - b['total']) / 8.0:+.1%}", f"{b['i_in']:.3f} A"))
        OUT.setdefault("budget", []).append(dict(case=name, vin=vin, hot=hot, **b))
    b = budget(pm, VBAT_MAX, True, eta_bound=0.93)
    rows.append((name + ", U1 at the data-sheet bound 93 %", "any", f"{b['module']:.2f}", f"{b['efuse'] * 1e3:.0f} mW",
                 f"{b['u1_loss']:.2f} W (93.0%)", f"{b['rail_3v3_in']:.2f}", f"{b['total']:.2f} W", f"{(8.0 - b['total']) / 8.0:+.1%}",
                 f"{b['i_in']:.3f} A"))
    OUT.setdefault("budget", []).append(dict(case=name + " bound", vin=VBAT_MAX, hot=True, **b))
table("5. Burst power at the carrier input (BASELINE 6: <= 8 W including the carrier)",
      ["Module", "VBAT", "Module W", "U2", "U1 loss", "3.3 V in W", "Carrier input", "Margin to 8 W", "Input current"], rows)

# ------------------------------------------------------------------------------------------------ 5 input filter, CE102
# Elements: C_F1 1 x 4.7 uF + 100 nF at J1; L_F 4.7 uH (DCR 30 mOhm); C_F2 3 x 4.7 uF at U1/U3; damper R_D 1.0 Ohm
# + C_D 2 x 4.7 uF. Ceramic 1210 50 V X7R at 14.4 V: 65 % of nominal (DC bias); ESL 1 nH, ESR 5 mOhm each.
def ccap(c, n, f, eff=0.65, esl=1e-9, esr=0.005):
    w = 2 * math.pi * f
    z = complex(esr, w * esl - 1 / (w * c * eff))
    return z / n


L_F, DCR_F = 4.7e-6, 0.030


def zout(f):
    w = 2 * math.pi * f
    zsrc = complex(0.05, w * 50e-9)        # EPS side: eFuse + 0.1 m harness (with its 4.7 uF lumped into C_F1)
    zc1 = par(ccap(4.7e-6, 2, f), zsrc)
    zl = complex(DCR_F, w * L_F)
    zc2 = ccap(4.7e-6, 3, f)
    zd = complex(1.0, 0) + ccap(4.7e-6, 2, f)
    return par(zc1 + zl, zc2, zd)


fs = [10 ** (3 + i * 0.01) for i in range(301)]
zpk = max(fs, key=lambda f: abs(zout(f)))
zin = VBAT_MIN ** 2 / budget(P_MOD_7W, VBAT_MIN, True)["total"]
rows = [("Filter resonance (L_F with C_F2 + C_D)", f"{zpk / 1e3:.1f} kHz", ""),
        ("Peak filter output impedance", f"{abs(zout(zpk)):.2f} Ohm", f"converter input |Zin| = V^2/P = {zin:.1f} Ohm at 11.2 V, 8 W"),
        ("Middlebrook margin", f"{20 * math.log10(zin / abs(zout(zpk))):.1f} dB", ">= 6 dB required, 10 dB target")]
table("6. Input filter stability", ["Item", "Value", "Note"], rows)
OUT["filter"] = dict(f_res=zpk, z_peak=abs(zout(zpk)), z_in=zin)


# CE102: ripple current of U1 (and U3, 180 deg) at the buck input; LISN 50 Ohm at the carrier input (MIL-STD-461G
# CE102 basic curve: 94 dBuV at 10 kHz falling to 60 dBuV at 500 kHz, 60 dBuV to 10 MHz; no relaxation below 28 V).
def ce102_limit(f):
    if f <= 500e3:
        return 94 - (94 - 60) * math.log10(f / 10e3) / math.log10(50)
    return 60.0


def harm_current(n, i_out, d):
    return 2 * i_out / (n * math.pi) * abs(math.sin(n * math.pi * d))


rows = []
worst = None
for vin in (VBAT_MIN, VBAT_NOM, VBAT_MAX):
    i_out = budget(P_MOD_7W, vin, False)["module"] / V5
    d = V5 / vin
    for n in (1, 2, 3, 5, 7, 10):
        f = n * FSW
        w = 2 * math.pi * f
        zc2 = par(ccap(4.7e-6, 3, f), complex(1.0, 0) + ccap(4.7e-6, 2, f))
        v_node = harm_current(n, i_out, d) * abs(zc2)  # peak volts at the converter input node
        zl = complex(DCR_F, w * L_F) if f < 30e6 else complex(5000, 0)  # SRF of L_F about 30 MHz (allocation)
        zc1 = par(ccap(4.7e-6, 1, f), ccap(100e-9, 1, f, eff=0.9, esl=0.5e-9), complex(50, 0))
        v_lisn = v_node * abs(zc1) / abs(zc1 + zl) / math.sqrt(2)
        dbuv = 20 * math.log10(v_lisn / 1e-6)
        margin = ce102_limit(f) - dbuv
        if worst is None or margin < worst[0]:
            worst = (margin, vin, n)
        if vin == VBAT_NOM or n == 1:
            rows.append((f"{vin:.1f} V", f"{f / 1e6:.0f} MHz", f"{harm_current(n, i_out, d) * 1e3:.0f} mA", f"{v_node * 1e3:.2f} mV",
                         f"{dbuv:.1f} dBuV", f"{ce102_limit(f):.0f}", f"{margin:.0f} dB"))
table("7. CE102 estimate at the carrier input (differential mode; test V-EMC-1 governs)",
      ["VBAT", "Harmonic", "Input current", "Ripple at the converter input", "At the LISN", "Limit dBuV", "Margin"], rows)
print(f"\nWorst differential-mode margin: {worst[0]:.0f} dB ({worst[1]} V, harmonic {worst[2]}).")
OUT["ce102_worst_margin_db"] = worst[0]

# ------------------------------------------------------------------------------------------------ 6 frequency plan
UP_WINDOWS = [(435.222, 435.778), (436.222, 436.500)]   # BASELINE 7 uplink request windows, MHz
L1 = (1575.42 - 1.023, 1575.42 + 1.023)
def nearest(fsw_mhz, lo, hi, ppm=50):
    best = 1e9
    for n in range(int(lo / fsw_mhz) - 2, int(hi / fsw_mhz) + 3):
        f = n * fsw_mhz
        tol = f * ppm * 1e-6
        if lo - tol <= f <= hi + tol:
            return 0.0
        best = min(best, abs(f - lo) - tol, abs(f - hi) - tol)
    return best


rows = []
for name, fm, ppm in (("carrier converters (Y1 1.000 MHz, +/-50 ppm)", 1.000, 50), ("2.000 MHz alternative", 2.000, 50),
                      ("U1 free-running on RT, Y1 failed (0.90-1.10 MHz)", 1.0, 1e5),
                      ("U3 free-running, Y1 failed or before lock (2.1-2.3 MHz)", 2.2, 4.6e4),
                      ("module reference 38.4 MHz (DS-11105-001 2.10.10)", 38.4, 50),
                      ("module CLK_M 19.2 MHz", 19.2, 50), ("PCIe reference 100 MHz (no SSC)", 100.0, 300)):
    rows.append((name, *(f"{nearest(fm, lo, hi, ppm) * 1e3:.0f} kHz" for lo, hi in UP_WINDOWS), f"{nearest(fm, *L1, ppm) * 1e3:.0f} kHz"))
table("8. Nearest harmonic to the victim bands (0 = a harmonic falls inside)",
      ["Source", "Uplink 435.222-435.778", "Uplink 436.222-436.500", "L1 C/A main lobe +/-1.023 MHz"], rows)
print("\n41 x 38.4 MHz = %.1f MHz: %.2f MHz below L1 (EMC-04)." % (41 * 38.4, 1575.42 - 41 * 38.4))

# ------------------------------------------------------------------------------------------------ 7 sequencing
C_IN_EFF = (1 + 3 + 2) * 4.7e-6 * 0.65 + 4.7e-6 * 0.65  # carrier ceramics + EPS output 4.7 uF
I_INRUSH = C_IN_EFF * EPS_SR
SENSE = (0.405 * 0.985 * (1 + 249e3 * 1.001 / (10e3 * 0.999)), 0.405 * (1 + 249e3 / 10e3), 0.405 * 1.015 * (1 + 249e3 * 0.999 / (10e3 * 1.001)))
CT = 3.3e-9
TD = CT * 1e9 / 175 + 0.5e-3
t_ramp = VBAT_NOM / EPS_SR
seq = [("0", "JETSON_PWR and PAYLOAD_EN high (OBC)", "EPS U19 ramps VBAT_JET at %.1f V/ms" % (EPS_SR / 1e3)),
       ("%.1f ms" % (U1_ON / EPS_SR * 1e3), "U1, U3 enabled (EN UVLO %.1f V)" % U1_ON, "soft start 3.5-7 ms"),
       ("%.1f ms" % (t_ramp * 1e3), "VBAT_IN at 14.4 V", f"inrush {I_INRUSH:.2f} A into {C_IN_EFF * 1e6:.0f} uF"),
       ("~9 ms", "5V_BUCK and 3V3_PRE in regulation; U2 starts (UVLO %.2f V), VDD_IN ramps 0.91 V/ms" % uvlo[0], ""),
       ("~15 ms", "VDD_IN above PGTH %.2f V: U2 PG released" % pgth[0], ""),
       ("%.1f ms" % ((U1_ON / EPS_SR + TD) * 1e3 + 2.0), "U4 RESET released (VBAT_IN > %.2f V for tD %.1f ms)" % (SENSE[1], TD * 1e3), "POWER_EN high: last open-drain released"),
       ("~26 ms", "Y1 enabled (OE RC), U1/U3 lock after 2048 cycles (2 ms)", "converters free-run until then"),
       ("+80 ms (NVIDIA, forum guidance; DG-10931-001 governs)", "module power sequence complete: SYS_RESET* high", ""),
       ("+1-2 ms", "U7, U8 on (CT 1 nF): 3V3_IO, 1V8_IO, 3V3_SSD", "JET_3V3 high: OBC JET_PG (PG6)"),
       ("+20-45 s (to be measured)", "UEFI from QSPI, kernel and rootfs from NVMe, application", "STATUS line to the OBC")]
table("9. Power-up sequence", ["t", "Event", "Note"], seq)
OUT["sequence"] = dict(inrush_a=I_INRUSH, c_in_eff_uF=C_IN_EFF * 1e6, sense=SENSE, td_ms=TD * 1e3)
print(f"\nInrush {I_INRUSH:.2f} A vs the EPS eFuse minimum {EFUSE_EPS[0]} A: {EFUSE_EPS[0] / I_INRUSH:.1f}x; "
      f"largest carrier input capacitance the EPS ramp allows at 80 % of the trip: {0.8 * EFUSE_EPS[0] / EPS_SR * 1e6:.0f} uF.")

# hold-up after an uncommanded cut, from the U4 threshold down to U1 drop-out (VIN = 5.6 V)
e_hold = 0.5 * C_IN_EFF * (SENSE[1] ** 2 - 5.6 ** 2)
t_hold = e_hold / budget(P_MOD_7W, VBAT_NOM, False)["total"]
print(f"Hold-up after the U4 trip: {e_hold * 1e3:.2f} mJ, {t_hold * 1e3:.2f} ms at 7 W (POWER_EN low {20e-6 * 1e6:.0f} us after the trip).")
OUT["holdup_ms"] = t_hold * 1e3

# power-down: rail discharge after SYS_RESET* low (DS-11105-001 3.6.2: 3.3 V I/O < 0.5 V within 1.5 ms, 1.8 V within 4 ms)
t33 = 35 * (1e-6 + 0.1e-6 + 0.2e-6) * math.log(3.33 / 0.5)
t18 = 2e-6 + 120 * (1e-6 + 0.2e-6) * math.log(1.8 / 0.5)
print(f"3V3_IO discharge to 0.5 V: {t33 * 1e3:.3f} ms (U7 QOD 35 Ohm max); 1V8_IO: {t18 * 1e3:.3f} ms (U9 120 Ohm pull-down): "
      "limits 1.5 ms and 4 ms.")
OUT["discharge_ms"] = dict(v33=t33 * 1e3, v18=t18 * 1e3)

# ------------------------------------------------------------------------------------------------ 8 latch-up coverage
p_in_trip_u2 = ILIM * 0.9 * 4.95 / 0.93
rows = [("Module or VDD_IN load, any size above %.1f A for > %.1f ms" % (ILIM * 0.9, t_it[2] * 1e3), "U2 circuit breaker, latched", "<= 3.2 ms",
         "JET_PG falls (module loses VDD_IN, SYS_RESET* low)"),
        ("Carrier input 8.0 W to %.0f W (module or 3.3 V side)" % p_in_trip_u2, "OBC payload_guard (EPS IMON_JET, BUSV)", "2 s (> 8 W twice) or 1 s (>= 12 W)", "both lines low"),
        ("3.3 V side above 2.8 A", "U3 peak current limit (hiccup)", "us", "3V3_PRE collapses, JET_PG falls"),
        ("Anything above 1.10-1.30 A at VBAT (14.7-17.4 W)", "EPS U19 eFuse, latched", "<= 1 ms", "OBC re-enables"),
        ("Below the 8 W guard (< 0.5 W above the capped 7.5 W burst)", "not detected", "", "ends with the burst (<= 10 min): RAD-V5 accepts it for an expendable payload")]
table("10. Latch-up and over-current coverage", ["Fault", "Protection", "Time", "Effect"], rows)

# ------------------------------------------------------------------------------------------------ 9 thermal interface
# Junction-to-spreader (THM-10): SoC junction to lid theta_jc (no published value: TDG-11127-001 not retrieved;
# allocation 0.30 K/W for a lidded FCBGA), gap pad over the SoC lid (area allocation 4.0 cm^2 until the NVIDIA STEP model
# is read), spreading from the pedestal into the 5 mm Al spreader.
A_LID = 4.0e-4
cases = []
for t_pad, k, comp in ((1.0e-3, 3.5, 0.25), (1.0e-3, 3.5, 0.15), (0.5e-3, 3.0, 0.20)):
    r_pad = t_pad * (1 - comp) / (k * A_LID) + 2 * 0.3e-4 / A_LID  # + 2 interfaces at 0.3 K cm^2/W each
    r_total = 0.30 + r_pad + 0.10
    cases.append((f"{t_pad * 1e3:.1f} mm, {k} W/m K, {comp:.0%} deflection", f"{r_pad:.2f}", f"{r_total:.2f}",
                  f"{39.0 + 7.0 * r_total:.1f}", f"{5.7 / r_total:.1f}x"))
table("11. Junction-to-spreader resistance (THM-10 design input; allowable 5.7 K/W)",
      ["Pad over the SoC", "Pad K/W", "Junction-to-spreader K/W", "Tj at 39 C spreader, 7 W (C)", "Margin on 5.7 K/W"], cases)
F_MAX = 8.2 * 9.80665
P_PAD = 70e3  # Pa, maximum pad stress at the largest deflection (requirement on the pad)
F_SOC = P_PAD * A_LID
F_DRAM = 0.5 * P_PAD * 3.0e-4  # DRAM and regulator pads: 3 cm2 allocation, softer pad at half the stress
print(f"\nClamp: SoC pad {F_SOC:.0f} N + other pads {F_DRAM:.0f} N = {F_SOC + F_DRAM:.0f} N at the largest deflection; "
      f"module limit {F_MAX:.0f} N (8.2 kgf, DS-11105-001 Table 5-2): {(F_SOC + F_DRAM) / F_MAX:.0%} (derated limit 50 %).")
OUT["clamp"] = dict(f_soc=F_SOC, f_other=F_DRAM, f_max=F_MAX)

# ------------------------------------------------------------------------------------------------ 10 part stress
# (name, applied, rating, kind) with kind V (ceramic/IC voltage, 50 % / 80 %), P (power 50 %), I (current 50 %)
stress = [
    ("C 4.7 uF 50 V X7R on VBAT_IN", VBAT_ABS, 50, 0.5),
    ("C 10 uF 25 V X7R on 5V_BUCK (survives a U1 high-side short to 18.4 V)", 5.16, 25, 0.5),
    ("C 22 uF 16 V X7R on VDD_IN", 5.21, 16, 0.5),
    ("C 22 uF 16 V X7R on 3V3_SSD", 3.33, 16, 0.5),
    ("D1 TVS2200 standoff 22 V", VBAT_ABS, 22, 1.0),
    ("U1 LM61440 VIN (36 V operating, 42 V abs)", VBAT_ABS, 36, 0.8),
    ("U3 LMR43620 VIN (36 V operating)", VBAT_ABS, 36, 0.8),
    ("U2 TPS25947 IN (23 V operating) in a U1 short", VBAT_ABS, 23, 0.8),
    ("U4 TPS3808 VDD on 3V3_PRE (6.5 V)", 3.33, 6.5, 0.8),
    ("U4 RESET pulled to VDD_IN (open drain, 7 V abs)", 5.21, 7.0, 0.8),
    ("U5 SN74LVC1G74 VCC on 3V3_PRE (5.5 V operating max)", 3.33, 5.5, 0.8),
    ("U6 SN74LVC1G07 input behind the 10 k / 20 k divider (5.5 V)", 5.21 * 20 / 30, 5.5, 0.8),
    ("U7, U8 TPS22918 VIN (5.5 V)", 3.33, 5.5, 0.8),
    ("U9 TLV73318 IN (5.5 V)", 3.33, 5.5, 0.8),
    ("U10, U11 SN74LVC1T45 (5.5 V)", 3.33, 5.5, 0.8),
    ("L1 XGL4030-472 at 1.30 A (eFuse max)", 1.30, 4.0, 0.5),
    ("L2 XGL6030-332 at 2.75 A (U2 trip max) + ripple/2", 2.75 + ripple[1] / 2, 8.1, 0.5),
    ("J1 Micro-Fit 2 contacts at 1.30 A", 0.65, 5.0, 0.5),
    ("J4 SO-DIMM VDD_IN 10 contacts at 2.26 A (highest steady current below the U2 trip)", 0.226, 0.5, 0.5),
    ("J5 M.2 3.3 V 9 contacts at 0.61 A", 0.61 / 9, 0.5, 0.5),
    ("R damper 1.0 Ohm 0805 (ripple 0.1 A rms)", 0.1 ** 2 * 1.0, 0.125, 0.5),
]
rows = []
for n, a, r, lim in stress:
    ratio = a / r
    rows.append((n, f"{a:.3g}", f"{r:.3g}", f"{ratio:.0%}", f"{lim:.0%}", "OK" if ratio <= lim else "EXCEEDS"))
table("12. Part stress against the derating rule", ["Part", "Applied", "Rating", "Ratio", "Limit", "Result"], rows)
OUT["stress_ok"] = all(a / r <= lim for _, a, r, lim in stress)

# junction temperatures, board at +60 C (hot acceptance limit of SYS-601 applied to an operating board: conservative)
TB = 60.0
tj = [("U1 LM61440 (loss at 16.4 V hot, 7 W)", budget(P_MOD_7W, VBAT_MAX, True)["u1_loss"] - DCR_L1 * 1.4 * (7.1 / 5.05) ** 2, 30.0),
      ("U2 TPS25947 at 1.42 A, 45 mOhm", (7.0 / 4.95) ** 2 * 0.045, 60.0),
      ("U3 LMR43620 at 0.6 A peak", 0.6 * 3.3 * (1 / 0.88 - 1) * 0.7, 45.0),
      ("U7/U8 TPS22918 at 0.61 A, 60 mOhm", 0.61 ** 2 * 0.06, 120.0)]
rows = [(n, f"{p * 1e3:.0f} mW", f"{th:.0f} K/W", f"{TB + p * th:.0f} C", "110 C") for n, p, th in tj]
table("13. Junction temperatures (board 60 C; theta from the EVM / psi values with 4 cm2 copper)",
      ["Part", "Dissipation", "Theta", "Tj", "Derated limit"], rows)
carrier_diss = budget(P_MOD_7W, VBAT_NOM, False)
print(f"\nCarrier dissipation (7 W module, 14.4 V): U1 {carrier_diss['u1_loss']:.2f} W, U2 {carrier_diss['efuse']:.3f} W, "
      f"3.3 V side {carrier_diss['rail_3v3_in']:.2f} W (SSD included), filter {carrier_diss['filt']:.3f} W.")

# ------------------------------------------------------------------------------------------------ 11 burst energy
E_BOOT = 30 * 4.5  # 30 s at 4.5 W carrier input (allocation until measured)
for t_gen, lines in ((150, 10), (570, 10)):
    e = E_BOOT + t_gen * budget(P_MOD_CAP, VBAT_NOM, False)["total"]
    print(f"Burst of 30 s boot + {t_gen} s generating ({lines} lines): {e:.0f} J = {e / 3600:.3f} Wh, {e / lines:.0f} J per line "
          f"(gate 360 J per accepted line)")
    OUT.setdefault("burst", []).append(dict(t_gen=t_gen, energy_j=e))

# ------------------------------------------------------------------------------------------------ 12 placement checks
# Stack coordinates (mm), carrier top side unless noted. Outline PC/104 90.17 x 95.89; columns (+-40, +-43) keep-out
# r 3.2; centre supports (0, +-21) r 3.5; module footprint x +-34.8, y -28.5..+16.5 (no top parts); gyro window
# x -33.5..-6.5, y +17.0..+44.0 (MECHANICAL_DESIGN 4.5, 11 item 12); no frame (withdrawn, board 2.0 mm): keep-out =
# window + 0.5 mm on both faces and all layers;
# top-side heights (MECHANICAL_DESIGN 4.5): <= 7.7 mm under the spreader (x -40..+40, y -33..+27), <= 13.2 mm elsewhere;
# magnetometer: the version-B keep-out round (+33, -38) is withdrawn (RM3100 moved to board A2 at (+24, +27) in the
# +Z end-frame slot, OBC item E11); the column below gives the in-plane distance to the old corner for reference only.
BX, BY = 90.17 / 2, 95.89 / 2
HOLES = [((40, 43), 3.2), ((-40, 43), 3.2), ((-40, -43), 3.2), ((40, -43), 3.2), ((0, 21), 3.5), ((0, -21), 3.5)]
WIN_KO = (-34.0, -6.0, 16.5, 44.5)      # both faces, all layers: window + 0.5 mm
SPREADER = (-40.0, 40.0, -33.0, 27.0)
GYRO = (-20.0, 30.5)
MODULE = (-34.8, 34.8, -28.5, 16.5)
MAG = (33.0, -38.0)
# name, (x, y) centre, (w, h), magnetic (ferromagnetic body or carries burst current), exempt
PLACE = [  # name, centre, (w, h), magnetic, exempt, height above the board (mm; mated where a harness plugs in)
    ("J4 SO-DIMM socket (its +X latch is the exemption)", (0, -31), (72, 8), True, "socket", 4.0),
    ("J5 M.2 connector + 2230 card (S4)", (18.75, 36), (34.5, 22), False, "", 5.5),
    ("MP1 M.2 standoff (steel)", (34, 36), (5, 5), True, "", 3.5),
    ("MP2 module standoff (steel)", (-31.3, 12.6), (5, 5), True, "module", 4.0),
    ("MP3 module standoff (steel)", (31.3, 12.6), (5, 5), True, "module", 4.0),
    ("J2 Pico-Clasp 6 (W5B), mated", (41.5, 28.5), (5, 10.6), False, "", 7.5),
    ("J3 Pico-Clasp 2 (spreader NTC), mated", (41.5, 37.0), (5, 5), False, "", 7.5),
    ("D2 ESD", (37.5, 24.5), (2.5, 2), False, "", 0.6),
    ("U10/U11 translators", (38.0, 19.5), (6, 2.5), False, "", 1.1),
    ("U7 3V3_IO switch, U6, U9", (12, 21), (12, 3), False, "", 1.45),
    ("U8 3V3_SSD switch and bulk", (26, 21), (14, 3.2), False, "", 2.5),
    ("J1 Micro-Fit 2x2 (W5A), mated (allocation)", (-29, -41.5), (10, 10), False, "", 13.0),
    ("D1, C_F1", (-22, -41.5), (3.5, 8), False, "", 2.5),
    ("L1 filter inductor", (-17.5, -41.5), (4, 4), True, "", 3.1),
    ("damper R_D, C_D", (-13, -41.5), (3.5, 8), False, "", 2.5),
    ("U1 LM61440 and input caps", (-8, -41.5), (4.5, 8), True, "", 2.5),
    ("L2 U1 inductor", (-1.0, -42.0), (6.5, 6.5), True, "", 3.1),
    ("U1 output caps 4 x 1206", (-8, -46.2), (9, 2.0), False, "", 1.8),
    ("U2 VDD_IN eFuse and 6 x 22 uF (if the VDD_IN pins are at the -X end)", (-40.5, -27), (6, 15), True, "", 2.5),
    ("J6 debug header (unmated in flight)", (-41.0, -9), (6, 7.6), False, "", 4.7),
    ("U3 LMR43620, L3 and caps", (-41.0, 3), (6, 13), True, "", 3.1),
    ("Y1 oscillator (metal lid)", (-41.0, 15.5), (5, 7), True, "", 1.5),
    ("U12, U4, U5 and passives", (-41.5, 25), (6, 10), False, "", 1.45),
]


def rect_overlap(a, b):
    return a[0] < b[1] and b[0] < a[1] and a[2] < b[3] and b[2] < a[3]


def rect_point_dist(r, p):
    dx = max(r[0] - p[0], 0, p[0] - r[1])
    dy = max(r[2] - p[1], 0, p[1] - r[3])
    return math.hypot(dx, dy)


rows, bad = [], 0
for name, (x, y), (w, h), magnetic, exempt, hgt in PLACE:
    r = (x - w / 2, x + w / 2, y - h / 2, y + h / 2)
    issues = []
    if r[0] < -BX + 0.5 or r[1] > BX - 0.5 or r[2] < -BY + 0.5 or r[3] > BY - 0.5:
        issues.append("outline")
    if rect_overlap(r, WIN_KO):
        issues.append("gyro window")
    if exempt not in ("socket", "module") and rect_overlap(r, MODULE):
        issues.append("module footprint")
    for (hx, hy), rad in HOLES:
        if rect_point_dist(r, (hx, hy)) < rad and exempt != "module":
            issues.append(f"keep-out ({hx}, {hy})")
    dmag = rect_point_dist(r, MAG)
    # (magnetometer keep-out withdrawn in version D: no check)
    hlim = 7.7 if rect_overlap(r, SPREADER) else 13.2
    if exempt not in ("socket", "module") and hgt > hlim:
        issues.append(f"height {hgt} > {hlim} mm")
    bad += bool(issues)
    rows.append((name, f"({x:.1f}, {y:.1f})", f"{dmag:.1f}", f"{rect_point_dist(r, GYRO):.1f}", f"{hgt} / {hlim}",
                 "; ".join(issues) or "clear"))
table("14. Placement check (zones of section 18; window, heights and magnetic keep-out of MECHANICAL_DESIGN 4.3-4.5)",
      ["Item", "Centre (mm)", "To the old RM3100 corner (mm)", "To the gyro centre (mm)", "Height / limit (mm)", "Result"], rows)
print(f"\nPlacement conflicts: {bad}.")
OUT["placement_conflicts"] = bad

# field of the burst current at the magnetometer: a go-and-return pair, B = mu0 I d / (2 pi r^2)
MU0 = 4e-7 * math.pi
rows = []
for name, i, d, r in (("VBAT_IN pair J1 -> U1 (planes, 0.2 mm)", 0.54, 0.2e-3, 0.075),
                      ("VDD_IN plane pair to the SO-DIMM", 1.41, 0.2e-3, 0.063),
                      ("module input through the SO-DIMM pins (pins 241-260, d about 5 mm), either end", 1.41, 5e-3, 0.063)):
    b = MU0 * i * d / (2 * math.pi * r ** 2)
    rows.append((name, f"{i:.2f} A", f"{r * 1e3:.0f} mm", f"{b * 1e9:.0f} nT"))
table("15. Static field of the burst current at the RM3100 on A2, (+24, +27), 63 mm (line-pair estimate, upper bound)", ["Path", "I", "r", "B"], rows)

# ------------------------------------------------------------------------------------------------ 13 RT1 conversion
# TE GA10K3MCD1: 10 kOhm at 25 C, beta 25/85 3976 K, -40..125 C, +/-0.2 C 0..70 C (TE data sheet, per its listing).
# TE publishes no R-T table in what could be reached; the "10K3A" curve's Steinhart-Hart constants (Betatherm 10K3A,
# the same 3976 K material) are A = 1.129241e-3, B = 2.341077e-4, C = 0.877547e-7 (Newport 710 mount manual addendum).
SH = (1.129241e-3, 2.341077e-4, 0.877547e-7)


def r_of_t(tc):
    a, b, c = SH
    t = tc + 273.15
    x = (a - 1 / t) / c
    y = math.sqrt((b / (3 * c)) ** 3 + x * x / 4)
    return math.exp((y - x / 2) ** (1 / 3) - (y + x / 2) ** (1 / 3))


rows = []
for tc in list(range(-40, 130, 5)):
    r = r_of_t(tc)
    ratio = r / (r + 10e3)
    dr = (r_of_t(tc + 0.5) / (r_of_t(tc + 0.5) + 10e3) - r_of_t(tc - 0.5) / (r_of_t(tc - 0.5) + 10e3))
    rows.append((f"{tc}", f"{r:,.0f}", f"{ratio:.4f}", f"{-dr * 4096:.1f}"))
table("16. RT1 resistance and OBC reading (10K3A curve; 10.0 kOhm bias from the ADC reference, NTC to PT_RTN)",
      ["T (C)", "R (Ohm)", "V/Vref", "12-bit counts per K"], rows)
r25, r85 = r_of_t(25), r_of_t(85)
print(f"\nCheck: R(25 C) = {r25:,.0f} Ohm; beta 25/85 from the curve = "
      f"{math.log(r25 / r85) / (1 / 298.15 - 1 / 358.15):.0f} K (data sheet 3976 K).")
OUT["rt1"] = {str(tc): r_of_t(tc) for tc in range(-40, 130, 5)}

# ------------------------------------------------------------------------------------------------ 14 stack-up (2.0 mm)
# Version E: 2.0 mm, 6 layers, symmetric (MECHANICAL_DESIGN: frame withdrawn, 2.0 mm carrier). Copper 1 oz (35 um).
# L1 parts/signals | 0.20 prepreg | L2 GND | 0.25 core | L3 PCIe/USB/REFCLK striplines | 0.89 core | L4 GND |
# 0.25 core | L5 power (VDD_IN, VBAT_F, 3V3 islands) | 0.20 prepreg | L6 parts <= 1 mm. High-Tg FR-4, er 3.9 at 1-5 GHz.
CU = 0.035
STACK = [0.20, 0.25, 0.89, 0.25, 0.20]
ER = 3.9
print(f"\nStack-up thickness: {sum(STACK) + 6 * CU:.3f} mm (finished board 2.0 mm +/-10 %)")


def z_ms(w, h=0.20, t=CU, er=ER):
    return 87 / math.sqrt(er + 1.41) * math.log(5.98 * h / (0.8 * w + t))


def z_osl(w, h=0.25, c=0.89 - CU, t=CU, er=ER):  # IPC-2141A offset stripline, h to the near plane, c to the far plane
    return 80 / math.sqrt(er) * math.log(1.9 * (2 * h + t) / (0.8 * w + t)) * (1 - h / (4 * (h + c + t)))


def zd_ms(w, sp, h=0.20):
    return 2 * z_ms(w, h) * (1 - 0.48 * math.exp(-0.96 * sp / h))


def zd_osl(w, sp, b=0.25 + CU + 0.89):
    return 2 * z_osl(w) * (1 - 0.374 * math.exp(-2.9 * sp / b))


def solve(fz, target, sp):
    lo, hi = 0.03, 1.0
    for _ in range(60):
        mid = (lo + hi) / 2
        if fz(mid, sp) > target:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


rows = []
for name, fz, target, sp in (("PCIe TX/RX lanes (module to M.2), L3", zd_osl, 85, 0.15),
                             ("PCIe REFCLK, L3", zd_osl, 85, 0.15),
                             ("USB0 to J6, L3", zd_osl, 90, 0.15),
                             ("PCIe breakout at the SO-DIMM/M.2 pads, L1 (<= 3 mm)", zd_ms, 85, 0.15)):
    w = solve(fz, target, sp)
    rows.append((name, f"{target} Ohm", f"{w * 1e3:.0f} um", f"{sp * 1e3:.0f} um", f"{fz(w, sp):.1f}"))
table("17. Differential impedance on the 2.0 mm stack-up (closed-form IPC-2141A estimates; the fabricator's field solver governs)",
      ["Pair", "Target", "Width", "Gap", "Zdiff (Ohm)"], rows)

with open(os.path.join(HERE, "jetson_calcs.json"), "w") as f:
    json.dump(OUT, f, indent=1, default=float)
