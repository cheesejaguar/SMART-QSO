#!/usr/bin/env python3
"""
SMART-QSO CDR mechanisms analysis: solar-wing hinge torque margin, deployment dynamics,
end-of-travel shock, burn-wire release energy, and partial-deployment cases.

Analysis tool (software Class D).  Results go into docs/environment/MECHANISMS_ANALYSIS.md
(SQSO-MECH-001).

  * Torque margin per NASA-STD-5017B (2022-12-06) Equation 4-1 and Table 1 ("theory or
    analysis" factors FSv 3.00, FSf 1.50, FSa 1.25 until tests replace them; one-spring-out
    case 1.00), at the cold and hot extremes and at both ends of travel.
  * Deployment dynamics: planar two-body model (body + one wing) about the hinge axis (+Z),
    free-floating, angular momentum conserved, the body tumbling at the start; the stop as a
    contact spring with damping.
  * Burn wire: the mission simulation's release model (software/rust/sqso-sil/src/hw/deploy.rs:
    release energy N(6.0 J, 1.2 J) at 20 C, +1 %/C below 20 C, 20 % of the heat kept between
    attempts) with the flight timing (sqso_fsw::deployment: wing burn 5 s, antenna burn 3 s,
    3 attempts 60 s apart) and the S5 current limit (0.8 A into 5 ohm per resistor).

    python3 mechanisms.py [--json out.json]
"""

from __future__ import annotations

import argparse
import json
import math

import numpy as np

# ---------------------------------------------------------------- inputs --------------
WING_M = 0.0504            # kg, MEV: panel 33.0 + cells 14.4 + moving hinge parts 3.0 g
WING_SPAN = 0.083          # m, hinge line to free edge
I_W_CG = WING_M * WING_SPAN ** 2 / 12      # about its own CG, axis parallel to the hinge
RHO0 = np.array([0.0, -WING_SPAN / 2])     # wing CG from the hinge, wing frame (stowed: -Y)
R_H = np.array([0.054, 0.0395])            # hinge axis from the body CG (W1), m
M_BODY = 1.726 - 2 * WING_M                # kg (MEV, rods), other wing counted in the body
I_BODY_Z = 3.0e-3                          # kg m^2, body about +Z through its CG, wings stowed

# Torsion springs, two per wing, 17-7PH or 302 stainless, wound so the load winds them up
SPRING_T_DEPLOYED = 5.0e-3   # N m per spring at the deployed stop (90 deg)
SPRING_T_STOWED = 10.0e-3    # N m per spring stowed (0 deg): rate 3.18 mN m/rad
SPRING_TOL = 0.10            # manufacturing tolerance on torque
SPRING_RELAX = 0.05          # stress relaxation, 2 years stowed (5017B A.2.5.1: design to limit)
SPRING_TEMP = {"cold": +0.018, "hot": -0.012}   # modulus change at -40 C / +60 C (about 0.03 %/C)

# Resistive torques per wing (N m): variable (Tv) and fixed (Tf), by end of travel and temperature
RESIST = {
    # name: (stowed cold, stowed hot, deployed cold, deployed hot, kind)
    "hinge friction (2 pins, mu 0.5 vacuum, unlubricated)": (1.0e-3, 0.8e-3, 0.5e-3, 0.4e-3, "v"),
    "flex-circuit harness across the hinge (set + cold stiffening)": (1.5e-3, 1.0e-3, 1.5e-3, 1.0e-3, "v"),
    "snubber / bumper adhesion (breakaway)": (0.5e-3, 0.5e-3, 0.0, 0.0, "v"),
    "cut hold-down line drag": (0.2e-3, 0.2e-3, 0.2e-3, 0.2e-3, "v"),
    "tumbling inertial torque at 50 deg/s": (0.1e-3, 0.1e-3, 0.1e-3, 0.1e-3, "f"),
}
FS = {"analysis": (3.00, 1.50, 1.25), "qual_test": (2.50, 1.35, 1.15), "accept_test": (2.00, 1.25, 1.10)}

# Burn wire (deploy.rs) and flight timing (deployment.rs)
BURN_R = 5.0
BURN_I = 0.8
REL_MEAN, REL_SIG, COLD_PER_C, KEEP = 6.0, 1.2, 0.01, 0.2
WING_BURN_S, ANT_BURN_S, ATTEMPTS = 5.0, 3.0, 3


def spring_torque(theta, temp="hot", n=2, worst=True):
    """Available spring torque per wing at deployment angle theta (rad, 0 stowed, pi/2 deployed)."""
    k = (SPRING_T_STOWED - SPRING_T_DEPLOYED) / (math.pi / 2)
    t = SPRING_T_STOWED - k * theta
    if worst:
        t *= (1 - SPRING_TOL) * (1 - SPRING_RELAX) * (1 + SPRING_TEMP[temp])
    return n * t


def torque_margin(stage="analysis"):
    fsv, fsf, fsa = FS[stage]
    rows = []
    for pos, ip in (("start (stowed)", 0), ("end of travel", 2)):
        for temp, j in (("cold", 0), ("hot", 1)):
            col = ip + j
            tv = sum(v[col] for v in RESIST.values() if v[4] == "v")
            tf = sum(v[col] for v in RESIST.values() if v[4] == "f")
            # acceleration to deploy within 5 s from rest: theta = a t^2 / 2
            ta = WING_M * WING_SPAN ** 2 / 3 * (2 * (math.pi / 2) / 5.0 ** 2)
            theta = 0.0 if ip == 0 else math.pi / 2
            avail = spring_torque(theta, temp)
            tm = avail / (fsv * tv + fsf * tf + fsa * ta) - 1
            one_out = spring_torque(theta, temp, n=1) / (tv + tf + ta) - 1
            rows.append((pos, temp, avail, tv, tf, ta, tm, one_out))
    return rows


# ---------------------------------------------------------------- dynamics ------------
def rot(a):
    c, s = math.cos(a), math.sin(a)
    return np.array([[c, -s], [s, c]])


J90 = rot(math.pi / 2)


def mass_matrix(theta):
    mu = M_BODY * WING_M / (M_BODY + WING_M)
    p = rot(theta) @ RHO0
    q = R_H + p
    m11 = I_BODY_Z + I_W_CG + mu * q @ q
    m22 = I_W_CG + mu * p @ p
    m12 = I_W_CG + mu * q @ p
    return np.array([[m11, m12], [m12, m22]])


def deploy_sim(w0_dps=15.0, k_stop=2.0, zeta=0.05, resist=2.2e-3, temp="hot", t_end=1.5, dt=2e-5,
               spring_scale=1.0):
    """Integrate the free-floating body + wing (W1) about +Z.  Returns peak values.
    w0: initial body rate about +Z (deg/s), the worst case for the hinge (it adds to or
    opposes the swing; both signs are run by the caller).  k_stop: stop contact stiffness
    (N m/rad); zeta: contact damping ratio."""
    theta, thd = 0.0, 0.0
    M0 = mass_matrix(0.0)
    phd = math.radians(w0_dps)
    h = M0[0, 0] * phd + M0[0, 1] * thd      # conserved angular momentum about +Z
    I_h = I_W_CG + WING_M * (WING_SPAN / 2) ** 2
    c_stop = 2 * zeta * math.sqrt(k_stop * I_h)
    t = 0.0
    out = {"t_reach": None, "w_reach": None, "T_stop_max": 0.0, "body_rate_max_dps": abs(w0_dps),
           "theta_max_deg": 0.0, "rebound_deg": 0.0, "t_settle": None}
    min_after = None
    reached = False
    eps = 1e-6
    while t < t_end:
        def acc(th, thd_):
            M = mass_matrix(th)
            dM = (mass_matrix(th + eps) - mass_matrix(th - eps)) / (2 * eps)
            # phi_dot from momentum
            phd_ = (h - M[0, 1] * thd_) / M[0, 0]
            qd = np.array([phd_, thd_])
            # Lagrange for theta: d/dt(M21 phd + M22 thd) - 1/2 qd^T dM qd = Q
            T_spring = spring_torque(th, temp) * spring_scale if th < math.pi / 2 + 0.3 else 0.0
            T_res = -math.copysign(resist, thd_) if abs(thd_) > 1e-6 else 0.0
            T_c = 0.0
            if th > math.pi / 2:
                T_c = -k_stop * (th - math.pi / 2) - c_stop * thd_
                if T_c > 0:
                    T_c = 0.0
            Q = T_spring + T_res + T_c
            # eliminate phd_dot: from momentum conservation M11 phdd + M12 thdd + (dM11 phd + dM12 thd) thd = 0
            A = np.array([[M[0, 0], M[0, 1]], [M[1, 0], M[1, 1]]])
            b0 = -(dM[0, 0] * phd_ + dM[0, 1] * thd_) * thd_
            b1 = Q - (dM[1, 0] * phd_ + dM[1, 1] * thd_) * thd_ + 0.5 * qd @ dM @ qd
            sol = np.linalg.solve(A, np.array([b0, b1]))
            return sol[1], phd_, T_c
        a1, phd_, Tc = acc(theta, thd)
        # semi-implicit Euler (adequate at dt = 20 us; energy checked by the caller)
        thd += a1 * dt
        theta += thd * dt
        t += dt
        out["T_stop_max"] = max(out["T_stop_max"], -Tc)
        out["body_rate_max_dps"] = max(out["body_rate_max_dps"], abs(math.degrees(phd_)))
        out["theta_max_deg"] = max(out["theta_max_deg"], math.degrees(theta))
        if not reached and theta >= math.pi / 2:
            reached = True
            out["t_reach"], out["w_reach"] = t, thd
        if reached and thd < 0:
            min_after = theta if min_after is None else min(min_after, theta)
    if min_after is not None:
        out["rebound_deg"] = 90.0 - math.degrees(min_after)
    out["final_body_rate_dps"] = math.degrees((h - mass_matrix(theta)[0, 1] * thd) / mass_matrix(theta)[0, 0])
    return out


def burn_release(temp_c, burn_s, attempts=ATTEMPTS, n=200000, seed=1):
    rng = np.random.default_rng(seed)
    need = np.maximum(REL_MEAN + REL_SIG * rng.standard_normal(n), 2.0) * (1 + COLD_PER_C * max(0.0, 20 - temp_c))
    e = BURN_I ** 2 * BURN_R * burn_s
    heat = np.zeros(n)
    released = np.zeros(n, bool)
    first = None
    for k in range(attempts):
        heat = heat * KEEP + e
        released |= heat >= need
        if k == 0:
            first = released.mean()
    need3 = (REL_MEAN + 3 * REL_SIG) * (1 + COLD_PER_C * max(0.0, 20 - temp_c))
    return {"E_attempt_J": e, "need_3sigma_J": need3, "ratio_3sigma": e / need3, "p_first": float(first),
            "p_fail_all": float(1 - released.mean()), "E_max_J": e / (1 - KEEP)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json")
    args = ap.parse_args()
    R = {}
    print("Wing torque margin, NASA-STD-5017B Eq. 4-1 (per wing, two springs)")
    for stage in ("analysis", "qual_test"):
        rows = torque_margin(stage)
        R[f"tm_{stage}"] = rows
        for pos, temp, avail, tv, tf, ta, tm, one in rows:
            print(f"  [{stage:9s}] {pos:15s} {temp:4s}: T_avail {avail*1e3:5.2f} mN m, Tv {tv*1e3:4.2f}, Tf {tf*1e3:4.2f}, "
                  f"Ta {ta*1e6:5.1f} uN m -> TM {tm:+.2f}; one spring out (FS 1.0) {one:+.2f}")
    # with a contact microswitch at the stop instead of a reed/Hall sensor: +0.4 N at 10 mm
    sw = 0.4 * 0.010
    avail = spring_torque(math.pi / 2, "cold")
    tv = 0.5e-3 + 1.5e-3 + 0.2e-3
    tm_sw = avail / (3.0 * (tv + sw) + 1.5 * 0.1e-3) - 1
    print(f"  with a 0.4 N plunger switch at the stop (10 mm arm), cold, end of travel: TM {tm_sw:+.2f}")
    R["tm_with_switch"] = tm_sw

    print("\nDeployment dynamics (W1, free-floating body, tumbling about +Z at the start)")
    dyn = {}
    for k_stop, lab in ((2.0, "soft bumper 2 N m/rad"), (25.0, "metal stop 25 N m/rad")):
        for w0 in (0.0, 15.0, -15.0, 50.0):
            r = deploy_sim(w0, k_stop)
            dyn[(lab, w0)] = r
            m_root = r["T_stop_max"]
            sigma = m_root / (0.098 * 0.0016 ** 2 / 6)
            strain = sigma / 18.6e9
            print(f"  {lab:22s} w0 {w0:+5.0f} deg/s: reaches 90 deg at {r['t_reach']*1e3:5.0f} ms, "
                  f"{r['w_reach']:5.1f} rad/s; peak stop torque {m_root:5.3f} N m (root stress {sigma/1e6:5.1f} MPa, "
                  f"{strain*1e6:4.0f} ue); body rate peak {r['body_rate_max_dps']:5.1f} deg/s; rebound "
                  f"{r['rebound_deg']:4.1f} deg; final body rate {r['final_body_rate_dps']:+6.2f} deg/s")
    R["dynamics"] = {f"{k[0]}|{k[1]}": v for k, v in dyn.items()}

    print("\nBurn-wire release (deploy.rs model, 0.8 A x 5 ohm = 3.2 W per resistor)")
    br = {}
    for temp in (20.0, -20.0, -40.0):
        for name, s in (("wing, 5 s", WING_BURN_S), ("wing, 8 s", 8.0), ("antenna line, 3 s", ANT_BURN_S)):
            r = burn_release(temp, s)
            br[f"{name}|{temp}"] = r
            print(f"  {name:18s} at {temp:+4.0f} C: {r['E_attempt_J']:5.1f} J per attempt, 3-sigma need "
                  f"{r['need_3sigma_J']:5.1f} J (ratio {r['ratio_3sigma']:.2f}); released on attempt 1 "
                  f"{r['p_first']*100:6.2f} %; not released after 3 attempts {r['p_fail_all']*100:7.4f} %")
    R["burn"] = br
    # pack current and energy for the wing pair
    print(f"  Wing pair: {2*BURN_I:.1f} A from the pack for <= {WING_BURN_S:.0f} s per attempt, "
          f"{2*BURN_I*14.4*WING_BURN_S*3/3600:.3f} Wh for three attempts")

    if args.json:
        with open(args.json, "w") as f:
            json.dump(R, f, indent=1, default=str)


if __name__ == "__main__":
    main()
