#!/usr/bin/env python3
"""
SMART-QSO wing hinge and deployment-spring design (SQSO-MECH-CAD-001 section 6.8; SQSO-MECH-001 Version B
sections 3-5).  Closes finding MCAD-01 / RID STR-13: a hinge and spring that fit the envelope that CDS Rev 14.1
sections 2.2.3 and 2.2.5 leave at the wing hinge line, with the NASA-STD-5017B torque margin recomputed.

Envelope (from model.py): hinge axis (+-54.1, +38.75) mm, parallel to Z.  Outer radius of anything on the axis
<= 2.0 mm (2.0 + 54.1 - 50 = 6.1 mm from the rail plane, 6.38 mm with the tolerance chain of model.py; y <= 40.75
mm, 9.25 mm from the rail edge).  Free length on the axis: the wing ends at z = -43 mm, so z -49.0 ... -40.4 is free
below the wing (-Z) and the wing end zones carry the knuckles: -Z primary spring 7.7 mm long, +Z secondary spring
3.6 mm long.

  * Spring: helical torsion spring, 17-7PH CH900 wire (E 200 GPa, Fty 1500, Ftu 2000 MPa: supplier values to
    confirm, as SQSO-MECH-001 section 3), close wound, rate k = E d^4 / (64 D n), stress Ki 32 M / (pi d^3);
    5017B Table 4 factors 1.5 (yield) / 1.65 (ultimate) on the maximum torque (nominal x 1.10 tolerance).
  * Torque margin: 5017B Eq. 4-1, TM = T_avail / (FSv Tv + FSf Tf + FSa Ta) - 1, Table 1 "theory or analysis"
    factors 3.00 / 1.50 / 1.25 (and the qualification-test factors 2.50 / 1.35 / 1.15 for reference); worst case per
    DDMR 9: minimum spring torque (-10 % tolerance, -5 % relaxation, modulus at temperature) against maximum
    resistance at the same temperature and point of travel; cold -55 C (the bounding LEOP stowed-wing temperature,
    -45 C predicted less the 10 C thermal margin, P3S-01; -40 C kept for comparison) and +60 C.
  * Deployment dynamics: the planar free-floating two-body model of software/simulation/cdr/structures/mechanisms.py
    (imported) with this spring, the damped soft stop and the CAD wing properties.

    python3 hardware/mechanical/hinge.py [--json hardware/mechanical/hinge_results.json]
"""
from __future__ import annotations

import argparse
import json
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "software", "simulation", "cdr", "structures"))
import mechanisms as MECH  # noqa: E402

E_W, FTY, FTU = 200e9, 1500e6, 2000e6          # 17-7PH CH900 wire
FS_Y, FS_U = 1.5, 1.65                          # 5017B Table 4 (springs), as SQSO-MECH-001
TOL, RELAX = 0.10, 0.05
TEMP_E = {"cold": +0.018, "hot": -0.012}        # modulus change at -40 / +60 C (SQSO-MECH-001), set by use_cold()
OD_MAX = 4.0e-3                                 # envelope (radius 2.0 mm about the hinge axis)
L_PRIMARY, L_SECOND = 7.5e-3, 3.6e-3            # free spring-body lengths on the axis (-Z, +Z)
PIN_R, LEG_ARM = 0.75e-3, 5.0e-3                # pin radius, spring-leg arm to its anchor (legs 5 mm long)
MU = {"cold": 0.15, "hot": 0.12}                # sputtered MoS2 film on an A286 pin in a Ti-6Al-4V bushing: design values
# hinge pin: A286 (AMS 5732, aged; non-ferromagnetic, MECHANICAL_DESIGN 4.4; was 440C), the film sets the friction
PIN = dict(material="A286", Ftu=1103e6, Fty=758e6, Fsu=0.6 * 1103e6, hardness="HRC 30-35")
R_STOP = 4.0e-3                                  # stop contact radius from the hinge axis (CAD: about 4.5 mm)
GAP_ARM = 0.6e-3                                 # pin bending arm across the ear-knuckle interface
                                                # 3-7 x the vacuum typical (0.02-0.05); VI-STR-04 measures them
# resistive torques independent of the spring (per wing, N m): (stowed cold, stowed hot, deployed cold, deployed hot)
FLEX = (0.50e-3, 0.35e-3, 0.50e-3, 0.35e-3)     # clock-spring flex, 2 layers, heat-set (allocation; elastic 0.1)
SNUB = (0.20e-3, 0.20e-3, 0.0, 0.0)             # PTFE-faced snubber breakaway (silicone face: 0.5 mN m)
LINE = (0.20e-3, 0.20e-3, 0.20e-3, 0.20e-3)     # cut hold-down line drag
OTHER_PIN = 0.05e-3                             # +Z knuckle bearing, unloaded in orbit
T_F = 0.10e-3                                   # tumbling 50 deg/s inertial torque (fixed)
FS = {"analysis": (3.00, 1.50, 1.25), "qual_test": (2.50, 1.35, 1.15)}
# cold-end values by temperature (C): spring modulus +0.03 %/C below +20 C; MoS2 friction, flex and snubber torques
# raised below -40 C by allowance (polyimide and silicone stiffen; no film data below -40 C: VI-MECH-03 measures it)
COLD = {-40: dict(E=+0.018, mu=0.15, flex=0.50e-3, snub=0.20e-3),
        -55: dict(E=+0.0225, mu=0.16, flex=0.55e-3, snub=0.25e-3)}
T_COLD = -55            # hinge cold bound (THERMAL_ANALYSIS 2.2 4.3: frame side -40.2, wing side -48.8 C margined)
T_BURN = -41            # burn resistor and the line in it at the burn, margined (-39.4; -40.8 after a 3 h soak)
# hold-down line (THERMAL_ANALYSIS 2.2 4.3): free span -81 C margined, anchors and the burn board about -42 C
LINE_T_SPAN, LINE_T_ANCHOR, LINE_T_COLUMN = -81.0, -42.0, -48.0
LINE_F0, LINE_CREEP = 30.0, 0.30                 # N at +20 C; preload loss allowed by VI-STR-07 (30-day creep test)
# loop geometry from the CAD (model.py: burn board at x 47.5, line bridge at x 55.25 over the post): two legs of
# 7.75 mm, 11.3 mm of it outside the body (x 51.1 ... 55.25 twice and the 3 mm bridge), 9.2 mm inside (with the knot)
LINE_L_OUT, LINE_L_IN = 11.3e-3, 9.2e-3
# the compressed column each leg spans: burn board 1.0 mm FR-4 (z 55 ppm/K), shear panel 1.0 Al (23.6), body panel
# 1.6 FR-4 (55), post 1.35 Al (23.6), wing 2.3 FR-4 + cells (50): sum alpha * L per leg (m/K)
LINE_COL_AL = (1.0 * 55 + 1.0 * 23.6 + 1.6 * 55 + 1.35 * 23.6 + 2.3 * 50) * 1e-9
LINE_MAT = {"Dyneema SK75 (UHMWPE)": dict(alpha=-12e-6, e_cold=1.3), "Vectran HT (LCP)": dict(alpha=-4.8e-6, e_cold=1.1)}


def use_cold(t_c):
    """Set the cold-case values for temperature t_c (C)."""
    global FLEX, SNUB
    c = COLD[t_c]
    TEMP_E["cold"] = c["E"]
    MU["cold"] = c["mu"]
    FLEX = (c["flex"], FLEX[1], c["flex"], FLEX[3])
    SNUB = (c["snub"], SNUB[1], 0.0, 0.0)


class Spring:
    def __init__(self, d, L, theta0, od=OD_MAX):
        self.d, self.od, self.theta0 = d, od, theta0
        self.D = od - d
        self.n = max(1.0, L / d - 1.0)                     # one wire diameter for the leg bends
        self.C = self.D / d
        self.Ki = (4 * self.C ** 2 - self.C - 1) / (4 * self.C * (self.C - 1))
        self.k = E_W * d ** 4 / (64 * self.D * self.n)
        # inner diameter after the full wind-up must clear the pin (D' = D n / (n + turns))
        turns = (theta0 + math.pi / 2) / (2 * math.pi)
        self.id_wound = self.D * self.n / (self.n + turns) - d

    def torque(self, theta, case="nom", temp="hot"):
        t = self.k * (self.theta0 + math.pi / 2 - theta)
        if case == "min":
            t *= (1 - TOL) * (1 - RELAX) * (1 + TEMP_E[temp])
        elif case == "max":
            t *= (1 + TOL) * (1 + TEMP_E["cold"])
        return t

    def stress(self):
        return self.Ki * 32 * self.torque(0.0, "max") / (math.pi * self.d ** 3)


def margins(springs, stage="analysis", latch_drag=0.0):
    fsv, fsf, fsa = FS[stage]
    rows = []
    ta = MECH.WING_M * MECH.WING_SPAN ** 2 / 3 * (2 * (math.pi / 2) / 5.0 ** 2)
    for pos, th, ip in (("start (stowed)", 0.0, 0), ("end of travel", math.pi / 2, 2)):
        for temp, j in (("cold", 0), ("hot", 1)):
            col = ip + j
            avail = sum(s.torque(th, "min", temp) for s in springs)
            fric = sum(MU[temp] * 2 * PIN_R / LEG_ARM * s.torque(th, "max") for s in springs) + OTHER_PIN
            tv = fric + FLEX[col] + SNUB[col] + LINE[col] + latch_drag
            tm = avail / (fsv * tv + fsf * T_F + fsa * ta) - 1
            outs = []
            for i in range(len(springs)):                 # one spring out, FS 1.0 (as SQSO-MECH-001)
                rest = [s for k, s in enumerate(springs) if k != i]
                av1 = sum(s.torque(th, "min", temp) for s in rest)
                fr1 = sum(MU[temp] * 2 * PIN_R / LEG_ARM * s.torque(th, "max") for s in rest) + OTHER_PIN
                outs.append(av1 / (fr1 + FLEX[col] + SNUB[col] + LINE[col] + latch_drag + T_F + ta) - 1)
            rows.append(dict(position=pos, temp=temp, T_avail_mNm=avail * 1e3, Tv_mNm=tv * 1e3,
                             friction_mNm=fric * 1e3, TM=tm, one_spring_out=min(outs) if outs else None))
    return rows


def design_search():
    """Two springs (DDMR 53, failure tolerance): maximise the smallest of the analysis-factor torque margin, the
    one-spring-out margin and the spring stress margins, over the wire diameters and the common preload."""
    best = None
    for d in [x / 1e6 for x in range(350, 651, 10)]:
        for d2 in [x / 1e6 for x in range(250, 501, 10)]:
            for th0 in [x / 20 for x in range(0, 141)]:
                sp = [Spring(d, L_PRIMARY, th0), Spring(d2, L_SECOND, th0)]
                if any(s.id_wound < 2 * PIN_R + 0.2e-3 for s in sp):
                    continue
                ms = min(min(FTY / (FS_Y * s.stress()), FTU / (FS_U * s.stress())) - 1 for s in sp)
                rows = margins(sp)
                tm = min(r["TM"] for r in rows)
                one = min(r["one_spring_out"] for r in rows)
                score = min(tm, ms, one)
                if best is None or score > best[0]:
                    best = (score, d, th0, d2, tm, ms, one)
    return best


def spring_report(s: Spring):
    return dict(wire_mm=s.d * 1e3, od_mm=s.od * 1e3, mean_d_mm=s.D * 1e3, index=s.C, coils=s.n,
                body_length_mm=(s.n + 1) * s.d * 1e3, rate_mNm_per_rad=s.k * 1e3, preload_deg=math.degrees(s.theta0),
                torque_stowed_nom_mNm=s.torque(0) * 1e3, torque_deployed_nom_mNm=s.torque(math.pi / 2) * 1e3,
                stress_max_MPa=s.stress() / 1e6, MS_yield=FTY / (FS_Y * s.stress()) - 1,
                MS_ult=FTU / (FS_U * s.stress()) - 1, id_wound_mm=s.id_wound * 1e3)


def dynamics(springs, k_stop=2.0, zeta=0.3, resist=1.5e-3):
    """Release-to-stop dynamics with the imported free-floating model: the spring of this design replaces
    SQSO-MECH-001's (per-wing nominal hot torque), the wing is the CAD wing (MEV)."""
    MECH.WING_M = 0.0503                      # MEV: substrate 33.0, cells 14.4, line, snubber, moving hinge parts
    MECH.WING_SPAN = 0.0816
    MECH.I_W_CG = MECH.WING_M * MECH.WING_SPAN ** 2 / 12
    MECH.RHO0 = MECH.np.array([0.0, -MECH.WING_SPAN / 2])
    MECH.R_H = MECH.np.array([0.0541, 0.0388])                     # hinge axis from the body CG (W1), m
    MECH.spring_torque = lambda th, temp="hot", n=2, worst=True: sum(s.torque(min(th, math.pi / 2), "nom", temp)
                                                                     for s in springs)
    out = {}
    for w0 in (0.0, 15.0, -15.0, 50.0):
        r = MECH.deploy_sim(w0, k_stop=k_stop, zeta=zeta, resist=resist)
        sigma = r["T_stop_max"] / (0.092 * 0.0016 ** 2 / 6)
        r["root_stress_MPa"] = sigma / 1e6
        r["root_strain_ue"] = sigma / 18.6e9 * 1e6
        out[f"w0={w0:+.0f}"] = r
    return out


def flex_torque():
    """Clock-spring flex in the -Z wing knuckle (cup inner radius 1.6 mm, hub radius 1.0 mm on the pin), 2.0 mm
    wide, two copper layers (18 um each) in 0.10 mm polyimide, 6 turns, heat-set at mid-travel."""
    b, t_pi, t_cu, off = 2.0e-3, 0.10e-3, 18e-6, 25e-6
    EI = 2.5e9 * b * t_pi ** 3 / 12 + 2 * 117e9 * (b * 0.6) * (t_cu ** 3 / 12 + t_cu * off ** 2)
    turns, r_mean = 6, 1.3e-3
    L = turns * 2 * math.pi * r_mean
    m_el = EI * (math.pi / 4) / L                    # +-45 deg about the heat-set state
    strain_set = (off + t_cu / 2) / 1.0e-3           # copper strain at the hub radius (formed, plastic, once)
    strain_amp = (off + t_cu / 2) * (math.pi / 4) / L
    return dict(EI_Nm2=EI, length_mm=L * 1e3, elastic_torque_mNm=m_el * 1e3, allocation_cold_mNm=FLEX[0] * 1e3,
                allocation_hot_mNm=FLEX[1] * 1e3, bend_radius_mm=(1.0, 1.6), copper_strain_formed=strain_set,
                copper_strain_amplitude=strain_amp,
                ipc2223_flex_to_install_min_radius_mm=6 * (t_pi + 2 * t_cu) * 1e3)


def latch_trade(springs, dyn):
    """Positive latch (spring pin into a hole at 90 deg) against hold-by-preload on a damped stop."""
    t_stop = max(v["T_stop_max"] for v in dyn.values())
    # latch: rebound energy fraction e^2 with the stop damping; impact on the latch pin at r = 1.2 mm
    zeta = 0.3
    e2 = math.exp(-2 * math.pi * zeta / math.sqrt(1 - zeta ** 2))
    pin_d, r_pin = 0.8e-3, 1.2e-3
    k_latch = 200.0                                   # N m/rad, steel pin in an Al knuckle hole
    i_h = MECH.I_W_CG + MECH.WING_M * (MECH.WING_SPAN / 2) ** 2
    e_imp = 0.5 * t_stop ** 2 / 2.0                   # energy stored in the 2 N m/rad stop at the peak
    t_latch = math.sqrt(2 * e2 * e_imp * k_latch)
    tau = t_latch / r_pin / (math.pi * pin_d ** 2 / 4)
    hold_min = min(s.torque(math.pi / 2, "min", "hot") for s in springs[:1]) + sum(
        s.torque(math.pi / 2, "min", "hot") for s in springs[1:])
    return dict(stop_peak_Nm=t_stop, restitution_energy=e2, latch_impact_torque_Nm=t_latch,
                latch_pin_shear_MPa=tau / 1e6, A286_shear_allow_MPa=0.6 * 1103, latch_drag_mNm=0.07,
                hold_by_preload_min_mNm=hold_min * 1e3, tumbling_torque_mNm=0.06,
                holding_margin=hold_min / (3.0 * 0.06e-3) - 1, wing_inertia_about_hinge=i_h)


def pin_strength(dyn, launch_normal_3s=3.85, launch_qs=2.29):
    """A286 hinge pin, d = 1.5 mm: shear and bending at the end-of-travel impact on the damped stop and at launch
    (hinge normal loads of the FE, fem_results.json), NASA-STD-5001B 1.25 / 1.40 with a 1.15 fitting factor."""
    t_stop = max(v["T_stop_max"] for v in dyn.values())
    F_stop = t_stop / R_STOP
    R_pin = 1.5 * F_stop                            # reaction incl. the wing's momentum change (conservative)
    R_launch = launch_normal_3s + launch_qs        # upper bound: QS and random added
    out = {}
    d = 2 * PIN_R
    A = math.pi * d ** 2 / 4
    Z = math.pi * d ** 3 / 32
    for case, Rf in (("end-of-travel impact", R_pin), ("launch, hinge normal", R_launch)):
        tau = Rf / A
        sig = Rf * GAP_ARM / Z
        vm = math.sqrt(sig ** 2 + 3 * tau ** 2)
        out[case] = dict(load_N=Rf, shear_MPa=tau / 1e6, bending_MPa=sig / 1e6, von_mises_MPa=vm / 1e6,
                         MS_yield=PIN["Fty"] / (1.25 * 1.15 * vm) - 1, MS_ult=PIN["Ftu"] / (1.40 * 1.15 * vm) - 1,
                         MS_shear_ult=PIN["Fsu"] / (1.40 * 1.15 * tau) - 1)
    out["stop_torque_Nm"] = t_stop
    out["bushing_bearing_MPa"] = R_pin / (d * 1.0e-3) / 1e6   # 1.0 mm bearing length (shortest knuckle)
    return out


def line_cold(ea20=5.0e3, k_series=None, mat="Dyneema SK75 (UHMWPE)", l_out=LINE_L_OUT, l_in=LINE_L_IN):
    """Hold-down line tension at the LEOP cold case (free span at LINE_T_SPAN) after the allowed creep loss: the
    loop is stretched at +20 C to LINE_F0 (braid EA ea20, N), creeps by LINE_CREEP of its stretch, then the line's
    negative axial CTE lengthens it and the compressed column (panels, post, wing) shortens: both slacken it.
    An optional series spring (N/m) at the anchor adds compliance."""
    m = LINE_MAT[mat]
    L = l_out + l_in
    k_line = ea20 / L
    k = k_line if k_series is None else 1.0 / (1.0 / k_line + 1.0 / k_series)
    stretch0 = LINE_F0 / k
    d_line = m["alpha"] * ((LINE_T_SPAN - 20) * l_out + (LINE_T_ANCHOR - 20) * l_in)      # > 0: the line grows
    d_col = 2 * LINE_COL_AL * (LINE_T_COLUMN - 20)                                          # < 0: the column shrinks
    slack = d_line - d_col
    k_cold = (1.0 / (1.0 / (k_line * m["e_cold"]) + (0.0 if k_series is None else 1.0 / k_series)))
    rem = stretch0 * (1 - LINE_CREEP) - slack
    return dict(material=mat, EA20_N=ea20, series_spring_N_per_mm=None if k_series is None else k_series / 1e3,
                loop_mm=L * 1e3, preload_stretch_mm=stretch0 * 1e3, thermal_slack_mm=slack * 1e3,
                line_growth_mm=d_line * 1e3, column_shrink_mm=-d_col * 1e3,
                tension_cold_N=max(0.0, rem * k_cold), slack_mm=max(0.0, -rem) * 1e3,
                tension_cold_no_creep_N=max(0.0, (stretch0 - slack) * k_cold))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", default=os.path.join(HERE, "hinge_results.json"))
    args = ap.parse_args()
    R = {}
    # the -40 C case of the earlier issues, for comparison (P3S-01)
    use_cold(-40)
    sp40 = [Spring(0.470e-3, L_PRIMARY, math.radians(55.0)), Spring(0.343e-3, L_SECOND, math.radians(55.0))]
    R["cold_-40C_reference"] = dict(TM_analysis=margins(sp40, "analysis"), TM_qual=margins(sp40, "qual_test"),
                                    springs=[spring_report(x) for x in sp40])
    use_cold(T_COLD)
    R["cold_case_C"] = T_COLD
    R["cold_values"] = COLD[T_COLD]
    # burn-wire release energy at the cold bound (the mechanisms.py model; STR-15: >= 1.5 x 3-sigma)
    # hold-down line at the LEOP cold case (P3S-01 follow-up)
    R["line_cold"] = [line_cold(ea, ks, mat) for mat in LINE_MAT for ea in (5.0e3, 10.0e3) for ks in (None, 40e3)]
    for v in R["line_cold"]:
        print(f"line {v['material']:22s} EA {v['EA20_N']/1e3:4.1f} kN spring {v['series_spring_N_per_mm']}: stretch "
              f"{v['preload_stretch_mm']:.3f} mm, thermal slack {v['thermal_slack_mm']:.3f} mm -> tension at -81 C "
              f"{v['tension_cold_N']:.1f} N after creep ({v['tension_cold_no_creep_N']:.1f} N without), slack {v['slack_mm']:.3f} mm")
    # flight timing (sqso_fsw::deployment): 8 s per resistor, retries 50 min apart, so no heat is kept between attempts
    keep0 = MECH.KEEP
    MECH.KEEP = 0.0
    R["burn_release"] = {f"{t:+.0f}C_{b:.0f}s": MECH.burn_release(float(t), b) for t in (T_BURN, T_COLD) for b in (5.0, 8.0, 10.0)}
    R["burn_temperature_C"] = T_BURN
    MECH.KEEP = keep0
    for k, v in R["burn_release"].items():
        print(f"burn {k}: {v['E_attempt_J']:.1f} J, 3-sigma need {v['need_3sigma_J']:.1f} J, ratio {v['ratio_3sigma']:.2f}, "
              f"first {v['p_first']*100:.2f} %, fail-all {v['p_fail_all']*100:.4f} %")
    # reference: SQSO-MECH-001 A springs (do not fit) for comparison
    best = design_search()
    score, d, th0, d2, tm, ms, one = best
    R["search_best"] = dict(score=score, d_mm=d * 1e3, theta0_deg=math.degrees(th0), d2_mm=d2 * 1e3, TM=tm,
                            MS_spring=ms, one_spring_out=one)
    print("search best (max of min(TM, spring MS)):", R["search_best"])
    # chosen design (rounded to stock wire and a checkable preload)
    prim = Spring(0.470e-3, L_PRIMARY, math.radians(55.0))
    sec = Spring(0.343e-3, L_SECOND, math.radians(55.0))
    designs = {"primary only (not failure tolerant)": [prim], "primary + secondary": [prim, sec]}
    for name, sp in designs.items():
        R[name] = dict(springs=[spring_report(s) for s in sp],
                       TM_analysis=margins(sp, "analysis"), TM_qual=margins(sp, "qual_test"),
                       TM_analysis_with_latch=margins(sp, "analysis", latch_drag=0.07e-3))
        print(f"\n== {name}")
        for s in R[name]["springs"]:
            print("  spring:", {k: round(v, 3) for k, v in s.items()})
        for stage in ("TM_analysis", "TM_qual", "TM_analysis_with_latch"):
            for r in R[name][stage]:
                print(f"  {stage:24s} {r['position']:15s} {r['temp']:4s} T_avail {r['T_avail_mNm']:5.2f} "
                      f"Tv {r['Tv_mNm']:5.2f} (friction {r['friction_mNm']:4.2f}) -> TM {r['TM']:+.2f}; "
                      f"one spring out {r['one_spring_out']:+.2f}")
    sp = designs["primary + secondary"]
    dyn = dynamics(sp)
    R["dynamics_soft_damped"] = dyn
    dyn_m = dynamics(sp, k_stop=25.0, zeta=0.05)
    R["dynamics_metal_stop"] = dyn_m
    for lab, dd in (("soft damped stop 2 N m/rad, zeta 0.3", dyn), ("metal stop 25 N m/rad, zeta 0.05", dyn_m)):
        for k, v in dd.items():
            print(f"  {lab:38s} {k}: t90 {v['t_reach']*1e3:5.0f} ms, w {v['w_reach']:5.1f} rad/s, stop peak "
                  f"{v['T_stop_max']:.3f} N m ({v['root_strain_ue']:.0f} ue), rebound {v['rebound_deg']:.1f} deg, "
                  f"body peak {v['body_rate_max_dps']:.0f} deg/s")
    R["flex"] = flex_torque()
    print("\nflex:", {k: v for k, v in R["flex"].items()})
    R["pin_strength_A286"] = pin_strength(dyn)
    print("pin (A286):", R["pin_strength_A286"])
    R["latch_trade"] = latch_trade(sp, dyn)
    print("latch trade:", R["latch_trade"])
    # the SQSO-MECH-001 A spring for the record
    ref = Spring(0.55e-3, 22 * 0.55e-3 + 0.55e-3, math.pi / 2, od=4.55e-3)
    R["mech001A_spring_envelope"] = dict(od_mm=4.55, length_mm=12.65, radius_mm=2.275,
                                         x_from_rail_mm=54.1 + 2.275 - 50, y_max_mm=38.75 + 2.275)
    with open(args.json, "w") as f:
        json.dump(R, f, indent=1, default=float)


if __name__ == "__main__":
    main()
