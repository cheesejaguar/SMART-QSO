#!/usr/bin/env python3
"""
SMART-QSO CDR structural hand analysis (no finite-element model exists).

Analysis tool (software Class D).  Results go into docs/environment/STRUCTURAL_ANALYSIS.md
(SQSO-SA-001).  Every input is listed at the top with its source; change it there.

Contents
  1. Random-vibration input: SpaceX Rideshare Payload User's Guide v11 (Aug 2026) Table 4-6
     MPE, integrated on log-log segments; MPE + 3 dB.
  2. Natural frequencies (closed-form): PCB plates, the PCB stack, the battery pack on its
     isolators, the stowed and deployed wings.
  3. Loads: limit load factor = max(quasi-static, 3-sigma Miles response at the local mode).
  4. Margins of safety (NASA-STD-5001B w/Change 3 Table 1/2/6 protoflight factors) for the
     frame, the PCB-stack spacers, the battery mount, the wing hinges and hold-down; PCB
     fatigue by Steinberg's displacement criterion.
  5. Threaded joints (NASA-STD-5020B): preload range, separation margin (Eq. 19, Figure 1).
  6. Venting (CDS Rev 14.1 2.1.9; UG v11 4.1.8).

    python3 structural_analysis.py [--json out.json]
"""

from __future__ import annotations

import argparse
import json
import math

G = 9.80665

# ---------------------------------------------------------------- 1. Environment ------
UG_MPE = [(20, 0.01), (50, 0.015), (700, 0.015), (800, 0.03), (925, 0.03), (2000, 0.00644)]
QSL_AXIAL = 10.0       # g, UG v11 Table 4-1 (integrated dispenser assembly, >= 20 kg)
QSL_LATERAL = 17.0     # g, RSS of the two lateral axes, UG v11 Table 4-1

# NASA-STD-5001B w/Change 3, protoflight
FS_Y, FS_U, TEST_FACTOR = 1.25, 1.40, 1.20          # Table 1, metallic
FS_U_COMPOSITE = 1.50                                # Table 2, protoflight, uniform material
FS_U_COMPOSITE_DISC = 2.00                           # Table 2, discontinuity (reducible to 1.5)
FS_U_SOFTGOOD = 2.00                                 # Table 6, "all others"
# NASA-STD-5020B Figure 1: separation could lead to a critical hazard -> max(1.2, FSy);
# otherwise max(1.0, test factor)
FS_SEP_CRITICAL = max(1.2, FS_Y)
FS_SEP_OTHER = max(1.0, TEST_FACTOR)
FITTING = 1.15                                       # 5020B 4.2.2 (separation-critical)

# Materials (A-basis or typical as noted)
AL6061 = {"E": 68.9e9, "Fty": 241e6, "Ftu": 290e6, "rho": 2700}       # MMPDS A-basis, plate
AL7075 = {"E": 71.7e9, "Fty": 462e6, "Ftu": 524e6, "rho": 2810}       # MMPDS A-basis, sheet
TI64 = {"E": 113.8e9, "Fty": 827e6, "Ftu": 896e6, "alpha": 8.6e-6}    # annealed, MMPDS
A286 = {"Ftu": 1103e6, "Fty": 758e6}                                  # NAS/MS A286 160 ksi
FR4 = {"E": 18.6e9, "nu": 0.12, "rho": 1850, "Fflex": 340e6}         # typical, IPC-4101/21
G10CR = {"E_ax": 15e9, "Fc": 300e6, "Fflex": 380e6, "alpha": 45e-6}  # NIST/ cryogenic G-10CR, typical
PEEK = {"E_ax": 3.6e9, "Fc": 118e6, "Fflex": 165e6, "alpha": 47e-6, "creep_allow": 20e6}  # Victrex 450G


def asd(f: float, prof=UG_MPE) -> float:
    for (f1, a1), (f2, a2) in zip(prof, prof[1:]):
        if f1 <= f <= f2:
            b = math.log(a2 / a1) / math.log(f2 / f1)
            return a1 * (f / f1) ** b
    raise ValueError(f)


def grms(prof=UG_MPE) -> float:
    tot = 0.0
    for (f1, a1), (f2, a2) in zip(prof, prof[1:]):
        b = math.log(a2 / a1) / math.log(f2 / f1)
        tot += a1 * f1 / (b + 1) * ((f2 / f1) ** (b + 1) - 1) if abs(b + 1) > 1e-9 else a1 * f1 * math.log(f2 / f1)
    return math.sqrt(tot)


def miles(f: float, q: float, scale_db: float = 0.0) -> float:
    """Miles' equation: rms acceleration response (g) of an SDOF at f with amplification Q."""
    return math.sqrt(math.pi / 2 * f * q * asd(f) * 10 ** (scale_db / 10))


def q_steinberg(f: float) -> float:
    """Steinberg's transmissibility estimate for PCBs, Q = sqrt(f)."""
    return math.sqrt(f)


# ---------------------------------------------------------------- 2. Frequencies ------
def plate_D(E, t, nu):
    return E * t ** 3 / (12 * (1 - nu ** 2))


def f_plate_corner_supported(a, b, t, mass_kg, E=FR4["E"], nu=FR4["nu"], lam2=7.12):
    """Rectangular plate point-supported at its four corners (Leissa, lambda^2 = 7.12 for a
    square, nu = 0.3); a, b in m; mass is the smeared total (board + parts)."""
    a_eq = math.sqrt(a * b)
    mu = mass_kg / (a * b)
    return lam2 / (2 * math.pi * a_eq ** 2) * math.sqrt(plate_D(E, t, nu) / mu)


def f_plate_cantilever(L, t, areal_kg_m2, E=FR4["E"], nu=FR4["nu"]):
    """Plate clamped on one edge (as a beam strip, beta L = 1.875)."""
    return 3.516 / (2 * math.pi * L ** 2) * math.sqrt(plate_D(E, t, nu) / areal_kg_m2)


def f_plate_ssss(a, b, t, areal, E=FR4["E"], nu=FR4["nu"]):
    return math.pi / 2 * (1 / a ** 2 + 1 / b ** 2) * math.sqrt(plate_D(E, t, nu) / areal)


def plate_ritz(a, b, t, mass_kg, points, E=FR4["E"], nu=FR4["nu"], deg=12, nq=40, kpen=1e9):
    """First natural frequency (Hz) of a free rectangular Kirchhoff plate with point supports
    (stiff penalty springs at `points`, (x, y) in m from a corner), by Rayleigh-Ritz on a
    Legendre-product basis.  Checked: a square on its four corners gives lambda^2 = 7.11
    (Leissa 7.12); adding a centre support gives 15.8."""
    import numpy as np
    from numpy.polynomial import legendre as Lg
    D = plate_D(E, t, nu)
    mu = mass_kg / (a * b)
    xg, wg = Lg.leggauss(nq)
    n = deg + 1
    eye = np.eye(n)
    V = np.array([Lg.legval(xg, eye[i]) for i in range(n)])
    V1 = np.array([Lg.legval(xg, Lg.legder(eye[i])) for i in range(n)])
    V2 = np.array([Lg.legval(xg, Lg.legder(eye[i], 2)) for i in range(n)])

    def I1(A, B, J):
        return (A * wg) @ B.T * J
    sx, sy, Jx, Jy = 2 / a, 2 / b, a / 2, b / 2
    A00, A11, A22, A02 = I1(V, V, Jx), I1(V1, V1, Jx) * sx ** 2, I1(V2, V2, Jx) * sx ** 4, I1(V, V2, Jx) * sx ** 2
    B00, B11, B22, B02 = I1(V, V, Jy), I1(V1, V1, Jy) * sy ** 2, I1(V2, V2, Jy) * sy ** 4, I1(V, V2, Jy) * sy ** 2
    K = D * (np.kron(A22, B00) + np.kron(A00, B22) + nu * (np.kron(A02, B02.T) + np.kron(A02.T, B02))
             + 2 * (1 - nu) * np.kron(A11, B11))
    M = mu * np.kron(A00, B00)
    for px, py in points:
        vx = np.array([Lg.legval(2 * px / a - 1, eye[i]) for i in range(n)])
        vy = np.array([Lg.legval(2 * py / b - 1, eye[i]) for i in range(n)])
        phi = np.kron(vx, vy)
        K += kpen * D / (a * b) * np.outer(phi, phi)
    Li = np.linalg.inv(np.linalg.cholesky(M))
    w = np.linalg.eigvalsh(Li @ K @ Li.T)
    w = np.sort(w[w > 1e-6])
    return float(np.sqrt(w[0]) / (2 * np.pi))


def board_supports(a, b, layout, inset=4e-3):
    c = [(inset, inset), (a - inset, inset), (inset, b - inset), (a - inset, b - inset)]
    if layout == "4 corners":
        return c
    if layout == "4 corners + centre":
        return c + [(a / 2, b / 2)]
    if layout == "6: corners + mid long edges":
        return c + [(a / 2, inset), (a / 2, b - inset)]
    raise ValueError(layout)


def hex_I(af, hole):
    return 5 * math.sqrt(3) / 144 * af ** 4 - math.pi * hole ** 4 / 64


def tube_I(od, idia):
    return math.pi * (od ** 4 - idia ** 4) / 64


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json")
    a = ap.parse_args()
    R = {}

    # 1. environment -------------------------------------------------------------------
    g_mpe = grms()
    g_p3 = g_mpe * math.sqrt(10 ** 0.3)
    R["grms_mpe"], R["grms_mpe3"] = g_mpe, g_p3
    print(f"UG v11 MPE overall {g_mpe:.2f} Grms; MPE+3 dB {g_p3:.2f} Grms")

    # 2. frequencies ---------------------------------------------------------------------
    boards = {  # name: MEV mass kg incl. parts
        "EPS board": 0.1125, "OBC": 0.0562, "RF board (shield not counted)": 0.150,
        "Jetson carrier + module + clamp": 0.109,
    }
    layouts = ("4 corners", "4 corners + centre", "6: corners + mid long edges")
    freqs = {}
    print("\nPCB plate modes, 90 x 96 mm, supports 4 mm in from the edges (Rayleigh-Ritz)")
    for n, m in boards.items():
        for lay in layouts:
            for t in (1.6e-3, 2.4e-3):
                f = plate_ritz(0.090, 0.096, t, m * (1.0 if t < 2e-3 else 1.08), board_supports(0.090, 0.096, lay))
                freqs[(n, lay, t)] = f
            print(f"  {n:34s} {m*1e3:6.1f} g  {lay:28s} f1 = {freqs[(n, lay, 1.6e-3)]:5.0f} Hz (1.6 mm), "
                  f"{freqs[(n, lay, 2.4e-3)]:5.0f} Hz (2.4 mm)")
    R["pcb_f"] = {f"{k[0]}|{k[1]}|{k[2]*1e3:.1f}": v for k, v in freqs.items()}

    # Steinberg fatigue criterion: Z_allow = 0.00022 B / (c h r sqrt(L)) [in], B = 3.78 in (96 mm),
    # r = 1 (component at the centre), L = 1.0 in, c = 1.75 (BGA / the SODIMM socket); 2e7 cycles
    stein = {}
    print("\nSteinberg PCB fatigue check (3-sigma displacement vs allowable)")
    for (n, lay, t), ff in freqs.items():
        h_in = t / 25.4e-3
        for db in (0.0, 3.0):
            q = q_steinberg(ff)
            g3 = 3 * miles(ff, q, db)
            z3 = 9.8 * g3 / ff ** 2
            zal = 0.00022 * 3.78 / (1.75 * h_in)
            stein[(n, lay, t, db)] = (ff, q, g3, z3, zal, zal / z3 - 1)
    for n in boards:
        for lay in layouts:
            row = []
            for t in (1.6e-3, 2.4e-3):
                for db in (0.0, 3.0):
                    row.append(f"{t*1e3:.1f} mm +{db:.0f} dB MS {stein[(n, lay, t, db)][5]:+.2f}")
            print(f"  {n:34s} {lay:28s} " + "; ".join(row))
    R["steinberg"] = {f"{k[0]}|{k[1]}|{k[2]*1e3:.1f}|{k[3]}": v for k, v in stein.items()}

    # PCB stack: 4 columns of Al 6061 M3 spacers (hex 5.0 mm AF, 2.5 mm tapped bore), length 98 mm
    m_stack = 0.1125 + 0.0562 + 0.150 + 0.109 + 0.0247 + 0.0748   # boards + heat spreader (MEV), kg
    L = 0.098
    EI = 4 * AL6061["E"] * hex_I(5e-3, 2.5e-3)
    kd = 1 / 3          # knock-down for the spacer-PCB threaded joints (no test data)
    mu = m_stack / L
    f_cc = 22.37 / (2 * math.pi * L ** 2) * math.sqrt(kd * EI / mu)
    f_cf = 3.516 / (2 * math.pi * L ** 2) * math.sqrt(kd * EI / mu)
    print(f"\nPCB stack lateral (mass {m_stack*1e3:.0f} g): clamped both ends {f_cc:.0f} Hz; "
          f"cantilever from one end {f_cf:.0f} Hz (EI x {kd:.2f} knock-down)")
    R["stack"] = {"m": m_stack, "f_cc": f_cc, "f_cf": f_cf}

    # Battery pack on 4 isolators: guided-cantilever bushing + Ti screw in bending (lateral),
    # bushing axial (normal); pack MEV mass 0.267 kg
    m_b = 0.200 + 0.0188 + 0.0391 + 0.0092
    Lb = 8e-3
    EI_ti = TI64["E"] * math.pi * (2.0e-3) ** 4 / 64            # M2.5 minor diameter ~2.0 mm
    res_b = {}
    for name, mat in (("G-10CR", G10CR), ("PEEK 450G", PEEK)):
        EI_b = mat["E_ax"] * tube_I(5e-3, 2.7e-3)
        k_lat = 4 * 12 * (EI_b + EI_ti) / Lb ** 3
        A_b = math.pi * ((5e-3) ** 2 - (2.7e-3) ** 2) / 4
        k_ax = 4 * mat["E_ax"] * A_b / Lb
        f_lat = math.sqrt(k_lat / m_b) / (2 * math.pi)
        f_ax = math.sqrt(k_ax / m_b) / (2 * math.pi)
        res_b[name] = {"f_lat": f_lat, "f_ax": f_ax, "EI_b": EI_b, "k_ax_each": k_ax / 4}
        print(f"Battery on {name:9s} bushings: lateral {f_lat:5.0f} Hz, axial {f_ax:5.0f} Hz "
              f"(mass {m_b*1e3:.0f} g; joint compliance not included, expect 0.5-0.8x)")
    R["battery_f"] = res_b

    # Wings (MEV): 33.0 + 14.4 cells + 3 g moving hinge parts = 50.4 g over 83 x 98 mm
    m_w = 0.0330 + 0.0144 + 0.003
    areal = m_w / (0.083 * 0.098)
    f_w_stow_corner = f_plate_corner_supported(0.083, 0.098, 1.6e-3, m_w)
    f_w_stow_ssss = f_plate_ssss(0.083, 0.098, 1.6e-3, areal)
    f_w_dep = f_plate_cantilever(0.083, 1.6e-3, areal)
    I_h = m_w * 0.083 ** 2 / 3
    f_stop = {k: math.sqrt(k / I_h) / (2 * math.pi) for k in (2.0, 25.0)}
    print(f"Wing stowed: {f_w_stow_corner:.0f} Hz (hinges + hold-down + bumper as 4 point supports) to "
          f"{f_w_stow_ssss:.0f} Hz (edge-supported); deployed bending {f_w_dep:.0f} Hz; "
          f"rigid rotation on the stop {f_stop[2.0]:.0f} Hz (soft bumper, 2 N m/rad) / "
          f"{f_stop[25.0]:.0f} Hz (metal stop, 25 N m/rad)")
    R["wing"] = {"m": m_w, "f_stow_corner": f_w_stow_corner, "f_stow_ssss": f_w_stow_ssss,
                 "f_dep_bending": f_w_dep, "f_stop": f_stop, "I_hinge": I_h}

    # 3/4. loads and margins ------------------------------------------------------------
    print("\nMargins (limit load = max(QSL, 3-sigma Miles); protoflight factors 1.25 yield / 1.4 ult)")
    out = []

    def ms(name, stress_limit, allow, fs, kind):
        m = allow / (fs * stress_limit) - 1
        out.append((name, stress_limit / 1e6, allow / 1e6, fs, kind, m))
        print(f"  {name:62s} {stress_limit/1e6:8.2f} MPa x {fs:.2f} vs {allow/1e6:7.0f} MPa ({kind}) MS {m:+.2f}")
        return m

    # Frame: rail compression in the dispenser, own 2.0 kg + a 2U neighbour (4.0 kg) at 10 g x MPE peak
    # factor (axial QSL already "combined"); 4 rails, L-section 8.5 x 8.5 x 1.5 mm (23.3 mm^2)
    F_ax = (2.0 + 4.0) * QSL_AXIAL * G
    A_rail = 2 * 8.5e-3 * 1.5e-3 - (1.5e-3) ** 2
    ms("Frame rail, axial compression (own 2 kg + 2U neighbour, 10 g)", F_ax / 4 / A_rail, AL6061["Fty"], FS_Y, "yield")
    A_end = 6.5e-3 * 6.5e-3
    ms("Rail end bearing on the pusher plate (6.5 x 6.5 mm)", F_ax / 4 / A_end, AL6061["Fty"], FS_Y, "yield")

    # PCB stack spacers: lateral 3-sigma at the clamped-clamped mode vs 17 g QSL
    g_stack = max(QSL_LATERAL, 3 * miles(f_cc, 10.0))
    g_stack3 = max(QSL_LATERAL, 3 * miles(f_cc, 10.0, 3.0))
    w = m_stack * g_stack * G / L
    M_col = w * L ** 2 / 12 / 4
    Z = hex_I(5e-3, 2.5e-3) / (5e-3 / math.sqrt(3))
    ms(f"Stack spacer bending at the end frame ({g_stack:.1f} g lateral)", M_col / Z, AL6061["Fty"], FS_Y, "yield")
    ms(f"Stack spacer bending at the end frame (MPE+3 dB, {g_stack3:.1f} g)", M_col * g_stack3 / g_stack / Z,
       AL6061["Fty"], FS_Y, "yield")
    # spacer thread root (M3 minor 2.39 mm) if the joint gaps: steel stud A286
    Z_root = math.pi * (2.39e-3) ** 3 / 32
    ms("Stack M3 A286 stud at the thread root (if the joint gaps)", M_col / Z_root, A286["Fty"], FS_Y, "yield")
    R["stack"].update({"g_lat": g_stack, "g_lat_p3": g_stack3, "M_col": M_col})

    # Battery mount (PEEK and G-10CR): lateral 3-sigma at the lateral mode, Q = 10
    bres = {}
    for name, mat in (("G-10CR", G10CR), ("PEEK 450G", PEEK)):
        fl = res_b[name]["f_lat"] * 0.6        # joint-compliance knock-down
        fa = res_b[name]["f_ax"] * 0.6
        gl = max(QSL_LATERAL, 3 * miles(fl, 10.0))
        ga = max(QSL_AXIAL, 3 * miles(fa, 10.0))
        Fl = m_b * gl * G / 4                          # shear per isolator
        Fa = m_b * ga * G / 4                          # axial per isolator (uniform)
        # rocking adds tension: pack CG 12 mm above the mounting plane, isolators +-35 mm apart
        Fa += m_b * gl * G * 0.012 / (2 * 0.035) / 2
        M = Fl * Lb / 2
        EIb = res_b[name]["EI_b"]
        share = EIb / (EIb + EI_ti)
        s_b = M * share * 2.5e-3 / tube_I(5e-3, 2.7e-3)
        s_s = M * (1 - share) * 1.0e-3 / (math.pi * (2.0e-3) ** 4 / 64)
        print(f"  Battery mount {name}: lateral {gl:.1f} g, axial {ga:.1f} g -> {Fl:.1f} N shear, "
              f"{Fa:.1f} N tension per isolator")
        ms(f"Battery isolator ({name}) bending", s_b, mat["Fflex"], FS_U_COMPOSITE_DISC, "ult, 5001B Table 2 disc.")
        ms(f"Battery Ti-6Al-4V M2.5 screw bending ({name} mount)", s_s, TI64["Fty"], FS_Y, "yield")
        # shoulder bearing at preload + tension: shoulder annulus 7 / 2.7 mm
        A_sh = math.pi * ((7e-3) ** 2 - (2.7e-3) ** 2) / 4
        bres[name] = {"g_lat": gl, "g_ax": ga, "F_shear": Fl, "F_tension": Fa, "A_shoulder": A_sh,
                      "f_lat_kd": fl, "f_ax_kd": fa}
    R["battery_mount"] = bres

    # Wing hinge pins (stowed): 3-sigma at the stowed mode, Q = 20; two hinges, 60 % of the load
    g_w = 3 * miles(f_w_stow_corner, 20.0)
    F_hinge = m_w * g_w * G * 0.6 / 2
    A_pin = math.pi * (1.5e-3) ** 2 / 4
    ms(f"Wing hinge pin, 1.5 mm 300-series, double shear ({g_w:.0f} g)", F_hinge / (2 * A_pin), 0.6 * 515e6, FS_Y, "yield (0.6 Fty)")
    # hold-down line: 40 % of the out-of-plane load at the hold-down, plus the line preload 30 N
    F_hd = m_w * g_w * G * 0.4
    P_line = 30.0
    line_break = 200.0
    m_line = line_break / (FS_U_SOFTGOOD * (P_line + F_hd)) - 1
    out.append(("Wing hold-down line (Vectran/Dyneema, 200 N break), 30 N preload", P_line + F_hd, line_break, FS_U_SOFTGOOD,
                "ult, 5001B Table 6", m_line))
    print(f"  Wing hold-down line: {P_line:.0f} N preload + {F_hd:.1f} N dynamic vs {line_break:.0f} N / 2.0 -> MS {m_line:+.2f}")
    sep_line = P_line * 0.7 / (FITTING * FS_SEP_OTHER * F_hd) - 1
    print(f"  Wing hold-down gapping: preload after 30 % creep/relaxation {P_line*0.7:.0f} N vs "
          f"{FITTING:.2f} x {FS_SEP_OTHER:.2f} x {F_hd:.1f} N -> MS {sep_line:+.2f}")
    R["wing"].update({"g_stowed_3s": g_w, "F_hinge": F_hinge, "F_holddown": F_hd, "MS_line": m_line,
                      "MS_line_gap": sep_line})
    # deployed wing root bending at end-of-travel (from mechanisms.py, peak stop torque)
    R["margins"] = out

    # 5. threaded joints (5020B) --------------------------------------------------------------
    print("\nThreaded joints (NASA-STD-5020B; K = 0.20 dry / 0.15 lubricated, Gamma 35 % / 25 %)")
    joints = []

    def joint(name, d, T, K, gamma, P_tL, sep_crit, n=4, ctol=0.05, relax=0.05, dthermal=0.0, creep=0.0):
        P_nom = T / (K * d)
        if sep_crit:
            P_min = (1 - ctol) * (1 - gamma) * P_nom
            fs = FS_SEP_CRITICAL
            ff = FITTING
        else:
            P_min = (1 - ctol) * (1 - gamma / math.sqrt(n)) * P_nom
            fs = FS_SEP_OTHER
            ff = 1.0
        P_min = P_min * (1 - relax) - dthermal - creep
        P_max = (1 + ctol) * (1 + gamma) * P_nom
        m = P_min / (ff * fs * P_tL) - 1 if P_tL > 0 else float("inf")
        joints.append((name, d * 1e3, T, P_nom, P_min, P_max, P_tL, fs * ff, m))
        print(f"  {name:58s} P_nom {P_nom:6.0f} N  P_min {P_min:6.0f} N  P_max {P_max:6.0f} N  "
              f"P_tL {P_tL:5.1f} N x {fs*ff:.2f}  MS_sep {m:+.2f}")
        return P_min, P_max

    # Jetson heat spreader to frame (thermal joint: separation-critical, critical hazard no)
    g_spr = max(QSL_LATERAL, 3 * miles(300.0, 10.0))
    joint("Heat spreader to frame, 4 x M2.5 A286, 0.35 N m (thermal joint)", 2.5e-3, 0.35, 0.20, 0.35,
          (0.0748 + 0.034) * g_spr * G / 4, True)
    joint("PCB stack column, M3 A286 stud, 0.60 N m, dry", 3.0e-3, 0.60, 0.20, 0.35,
          m_stack * g_stack * G * 0.098 / 2 / (2 * 0.080) / 2, False)
    joint("Wing hinge to frame, 2 x M2 A286, 0.15 N m", 2.0e-3, 0.15, 0.20, 0.35, F_hinge, False, n=2)
    # Battery isolators: Ti M2.5, lubricated (MoS2), creep and thermal loss through the polymer
    for name, mat in (("G-10CR", G10CR), ("PEEK 450G", PEEK)):
        Fa = bres[name]["F_tension"]
        k_s = TI64["E"] * 3.39e-6 / 12e-3
        clamp_len = 8e-3
        k_b = mat["E_ax"] * bres[name]["A_shoulder"] * 0 + mat["E_ax"] * (math.pi * ((5e-3) ** 2 - (2.7e-3) ** 2) / 4) / clamp_len
        k_ser = 1 / (1 / k_s + 1 / k_b)
        dT = 30.0     # K, assembly at +20 C to a -10 C isolator
        d_th = (mat["alpha"] - TI64["alpha"]) * dT * clamp_len * k_ser
        creep_strain = 0.002 if "PEEK" in name else 0.0005   # 2-year creep at ~10 MPa: PEEK, G-10CR
        d_cr = creep_strain * clamp_len * k_ser
        K_DISC = 2.0e6     # N/m: an M2.5 disc spring (or two in series) rated >= 600 N flat
        for torque in (0.12, 0.30):
            joint(f"Battery isolator {name}, Ti M2.5 {torque:.2f} N m, no disc spring", 2.5e-3, torque, 0.15, 0.25, Fa,
                  True, dthermal=d_th, creep=d_cr)
        k_ser2 = 1 / (1 / k_s + 1 / k_b + 1 / K_DISC)
        for torque in (0.12, 0.30):
            joint(f"Battery isolator {name}, Ti M2.5 {torque:.2f} N m, with disc spring", 2.5e-3, torque, 0.15, 0.25,
                  Fa, True, dthermal=d_th * k_ser2 / k_ser, creep=d_cr * k_ser2 / k_ser)
        R.setdefault("isolator_loss", {})[name] = {"k_series": k_ser, "thermal_N": d_th, "creep_N": d_cr,
                                                   "k_series_disc": k_ser2,
                                                   "thermal_N_disc": d_th * k_ser2 / k_ser,
                                                   "creep_N_disc": d_cr * k_ser2 / k_ser}
        # shoulder bearing at maximum preload (0.12 N m) plus the tension
        for torque in (0.12, 0.30):
            Pmax = 1.05 * 1.25 * torque / (0.15 * 2.5e-3)
            s_b = (Pmax + Fa) / bres[name]["A_shoulder"]
            ms(f"Isolator shoulder bearing ({name}), {torque:.2f} N m max preload + tension", s_b,
               mat["Fc"], FS_U_COMPOSITE, "ult")
            print(f"    long-term bearing stress at nominal preload {torque/(0.15*2.5e-3)/bres[name]['A_shoulder']/1e6:.1f}"
                  f" MPa (PEEK creep allowable ~20 MPa)")
    R["joints"] = joints

    # 6. venting ---------------------------------------------------------------------------
    V = 0.098 * 0.098 * 0.098            # m^3, internal volume (no deduction for parts: conservative)
    A_cds = V / 50.8
    A_ug = V / 400.0
    A_doc = 4e-6
    print(f"\nVenting: V = {V*1e6:.0f} cm^3; CDS 2.1.9 needs A >= {A_cds*1e6:.1f} mm^2 (V/A < 50.8 m); "
          f"UG v11 4.1.8 A >= {A_ug*1e6:.1f} mm^2; STRUCTURAL_DESIGN 1.2 '> 4 mm^2' gives V/A = {V/A_doc:.0f} m")

    def dp(VA, rate, p, cd=0.6, T=290.0):
        # quasi-steady incompressible orifice: dp = (V/(Cd A))^2 * (rho/(2 p^2)) * (dp/dt)^2, rho = p/(R T)
        rho = p / (287.0 * T)
        return (VA / cd) ** 2 * rho / (2 * p * p) * rate ** 2
    for VA in (50.8, V / A_doc):
        print(f"  V/A = {VA:6.1f} m: dp at 101 kPa, 2.8 kPa/s {dp(VA, 2800, 101e3):7.1f} Pa; at 4.5 kPa/s "
              f"{dp(VA, 4500, 101e3):7.1f} Pa; at 20 kPa, 2.8 kPa/s {dp(VA, 2800, 20e3):7.1f} Pa")
    R["vent"] = {"V": V, "A_cds_mm2": A_cds * 1e6, "A_ug_mm2": A_ug * 1e6,
                 "dp_cds_Pa": dp(50.8, 4500, 20e3), "dp_doc_Pa": dp(V / A_doc, 4500, 20e3)}

    if a.json:
        with open(a.json, "w") as f:
            json.dump(R, f, indent=1, default=str)


if __name__ == "__main__":
    main()
