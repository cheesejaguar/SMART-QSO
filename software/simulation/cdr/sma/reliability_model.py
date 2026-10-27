"""
Mission-success probability for SMART-QSO (success levels L1, L2, L3 of
docs/mission/MISSION_SUCCESS_CRITERIA.md v2.0) and the probability that the
FCC post-mission disposal rule is met, by Monte Carlo over the reliability
block diagram of docs/environment/RELIABILITY_ANALYSIS.md v2.0 section 4.

Each block has
  p_inf : probability of an early (infant-mortality) failure: design,
          workmanship or first-flight defects, which dominate CubeSat losses;
          sampled at an exponential time with a 10-day mean after deployment;
  lam   : constant hazard per year afterwards (random part failures,
          single-event latch-up or burn-out not cleared, TID);
and the success logic of each level (series, k-of-n and degraded modes) is in
`evaluate()`. The values are this review's engineering allocations, not
measurements; their basis is in RELIABILITY_ANALYSIS v2.0 Table 4-2. The
result is cross-checked against the empirical CubeSat survival curve
(`empirical_cubesat()`).

Run: python3 reliability_model.py [--trials N] [--alt 475]
"""

from __future__ import annotations

import argparse
import math

import numpy as np

DAY = 1.0 / 365.25

# name: (p_inf, lam per year, role)
#   role "series"   : required for every level from deployment
#   role "xpdr"     : required for transponder service (L1 demonstration, L2, L3)
#   role "wing"     : one of two wings (k-of-n logic below)
#   role "wingcommon": the burn-wire drive both wings share; its failure loses both
#   role "adcs"     : sun-pointing; loss is a degraded mode unless a wing is lost too
#   role "heater"   : battery heater; loss is a degraded mode (cold charging inhibit)
BLOCKS = {
    "Separation, deployment switches, power-up": (0.010, 0.000, "series"),
    "EPS: MPPT charger, BMS, PDU, safety board (custom)": (0.050, 0.030, "series"),
    "Battery pack 4S1P INR18650-35E (series string)": (0.005, 0.020, "series"),
    "Solar array strings (loss of more than 2 of 6)": (0.002, 0.002, "series"),
    "OBC STM32L4R5 board (custom), NVM, watchdog": (0.040, 0.030, "series"),
    "Flight software and firmware (never flown)": (0.040, 0.020, "series"),
    "435 MHz receiver (telecommand, transponder input)": (0.030, 0.020, "series"),
    "2 m transmitter and PA (beacon, transponder output)": (0.030, 0.030, "series"),
    "Antennas, ISIS AntS (VHF and UHF elements)": (0.015, 0.000, "series"),
    "Harness, connectors, workmanship": (0.020, 0.005, "series"),
    "Structure (COTS frame)": (0.001, 0.000, "series"),
    "Telecommand keys and counters (lock-out)": (0.005, 0.005, "series"),
    "Jetson faults not isolated by the OBC trip": (0.003, 0.003, "series"),
    "GNSS faults not isolated by its load switch": (0.001, 0.001, "series"),
    "Operations error (unrecoverable command)": (0.003, 0.005, "series"),
    "Transponder repeat path (CTCSS, heartbeat gate, timer)": (0.030, 0.010, "xpdr"),
    "Wing burn driver, SW6 arm switch, timer (one driver for both wings, S5)": (0.015, 0.000, "wingcommon"),
    "Wing 1 deployment (burn-wire, hinge)": (0.030, 0.000, "wing"),
    "Wing 2 deployment (burn-wire, hinge)": (0.030, 0.000, "wing"),
    "ADCS: magnetometer, gyros (M-G370 + BMI088), torquers": (0.030, 0.020, "adcs"),
    "Battery heater and cut-out": (0.010, 0.010, "heater"),
}
P_LAUNCH = 0.99  # Falcon 9 rideshare plus integrator dispenser (see v2.0 section 4.3)
HEATER_LOSS_KILLS_L2 = 0.3  # fraction of heater failures that end transponder service


def sample(n, rng, scale_inf=1.0, scale_lam=1.0):
    t = {}
    for name, (p_inf, lam, _role) in BLOCKS.items():
        early = rng.random(n) < min(1.0, p_inf * scale_inf)
        t_early = rng.exponential(10.0 * DAY, n)
        t_rand = rng.exponential(1.0 / (lam * scale_lam), n) if lam > 0 else np.full(n, np.inf)
        t[name] = np.where(early, t_early, t_rand)
    return t


def evaluate(t, rng, n):
    series = np.min([t[k] for k, v in BLOCKS.items() if v[2] == "series"], axis=0)
    xpdr = t["Transponder repeat path (CTCSS, heartbeat gate, timer)"]
    # A wing either deploys at T+30 min or never does (lam = 0, so any finite time is a failure)
    w1 = np.isfinite(t["Wing 1 deployment (burn-wire, hinge)"])
    w2 = np.isfinite(t["Wing 2 deployment (burn-wire, hinge)"])
    common = np.isfinite(t["Wing burn driver, SW6 arm switch, timer (one driver for both wings, S5)"])
    wings_lost = np.where(common, 2, w1.astype(int) + w2.astype(int))
    adcs = t["ADCS: magnetometer, gyros (M-G370 + BMI088), torquers"]
    heater = t["Battery heater and cut-out"]
    heater_kill = rng.random(n) < HEATER_LOSS_KILLS_L2
    launch = rng.random(n) < P_LAUNCH

    def service_until(horizon_yr):
        ok = launch & (series > horizon_yr) & (xpdr > horizon_yr)
        ok &= wings_lost < 2
        ok &= ~((wings_lost == 1) & (adcs <= horizon_yr))
        ok &= ~((heater <= horizon_yr) & heater_kill)
        return ok

    l1 = launch & (series > 30 * DAY) & (xpdr > 7 * DAY)  # bus 30 days; first relay in week 1
    l2 = l1 & service_until(1.0)
    l3 = l2 & service_until(2.0)
    # time the mission ends (loss of the ability to operate the station)
    end = np.where(launch, series, 0.0)
    return l1, l2, l3, end, wings_lost


def empirical_cubesat(t_days):
    """Two-Weibull mixture for 178 CubeSats launched 2003-2014 (Langer and
    Bouwmeester, SSC16-X-2, 2016), parameters as quoted by TU Delft follow-up
    work: P_NZ = 0.8146, alpha = 0.2115, theta1 = 57.97 d, beta1 = 0.9017,
    theta2 = 4837.39 d, beta2 = 1.0710. Secondary source: the original paper
    could not be retrieved in the review environment."""
    pnz, a, th1, b1, th2, b2 = 0.8146, 0.2115, 57.9715, 0.9017, 4837.3947, 1.0710
    return pnz * (a * math.exp(-(t_days / th1) ** b1) + (1 - a) * math.exp(-(t_days / th2) ** b2))


def disposal(end, wings_lost, rng, alt_lives):
    """P(re-entry no later than 5 yr after the mission ends), 47 CFR
    97.207(g)(1)(vii)(D)(1). The mission ends at the planned 2 years or at the
    earlier loss of the station, whichever comes first."""
    t_end = np.minimum(end, 2.0)
    out = {}
    for case, (life_dep, life_fail) in alt_lives.items():
        life = np.where(wings_lost == 2, life_fail, life_dep)
        out[case] = float(np.mean(life <= t_end + 5.0))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--trials", type=int, default=400_000)
    ap.add_argument("--seed", type=int, default=20261027)
    args = ap.parse_args()
    rng = np.random.default_rng(args.seed)
    n = args.trials

    print("| Case | L1 (30 d) | L2 (1 yr) | L3 (2 yr) |")
    print("|---|---|---|---|")
    for label, si, sl in (("Allocation as tabulated", 1.0, 1.0),
                          ("Early-failure probabilities x0.5 (full EM/FM burn-in and day-in-the-life tests)", 0.5, 1.0),
                          ("Early-failure probabilities x2 (first-build team, tests cut)", 2.0, 1.0),
                          ("Random hazards x2 (radiation worse than predicted)", 1.0, 2.0)):
        t = sample(n, rng, si, sl)
        l1, l2, l3, end, wl = evaluate(t, rng, n)
        print(f"| {label} | {l1.mean():.2f} | {l2.mean():.2f} | {l3.mean():.2f} |")
        if label.startswith("Allocation"):
            base = (t, end, wl)
    print("\nEmpirical CubeSat survival (spacecraft alive, launch excluded), two-Weibull mixture:")
    for d in (30, 365, 730):
        print(f"  {d} d: {empirical_cubesat(d):.2f}")

    t, end, wl = base
    # contribution of each series block to L1 failure (first-order)
    print("\nShare of L2 losses by first-failing required block (allocation case):")
    req = [k for k, v in BLOCKS.items() if v[2] in ("series", "xpdr")]
    first = np.argmin(np.array([t[k] for k in req]), axis=0)
    tmin = np.min(np.array([t[k] for k in req]), axis=0)
    lost = tmin <= 1.0
    for i, k in sorted(enumerate(req), key=lambda x: -np.mean((first == x[0]) & lost)):
        share = np.mean((first == i) & lost)
        if share > 0.002:
            print(f"  {k}: {share:.3f}")
    print(f"  Both wings failed: {np.mean(wl == 2):.4f}; one wing failed: {np.mean(wl == 1):.3f}")

    # disposal compliance: lifetimes from the project estimator (debris_lifetime.py), 1.8 kg
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "orbit_lifetime"))
    from debris_lifetime import AREAS, SCENARIOS, START  # noqa: E402
    from orbit_lifetime import DensityTable, Spacecraft, propagate  # noqa: E402
    table = DensityTable(97.4)
    dep = Spacecraft(1.8, AREAS["wings deployed, no antenna credit"], 2.2)
    stow = Spacecraft(1.8, AREAS["wings failed (stowed), no antenna credit"], 2.2)
    print("\nP(re-entry no later than 5 yr after mission end), 1.8 kg; mission end = 2 yr or the")
    print("earlier loss of the station (A), or the planned 2 yr regardless of failure (B):")
    print("| Deployment (km) | Solar case | Life wings deployed / failed (yr) | P, reading A | P, reading B |")
    print("|---|---|---|---|---|")
    for alt in (450.0, 460.0, 470.0, 475.0, 485.0, 500.0):
        for sname in list(SCENARIOS)[:2]:
            prof = SCENARIOS[sname]
            ld = propagate(table, dep, alt, prof, START, max_years=60.0).lifetime_years
            lf = propagate(table, stow, alt, prof, START, max_years=60.0).lifetime_years
            pa = disposal(end, wl, rng, {"a": (ld, lf)})["a"]
            pb = disposal(np.full_like(end, 2.0), wl, rng, {"b": (ld, lf)})["b"]
            print(f"| {alt:.0f} | {sname} | {ld:.1f} / {lf:.1f} | {pa:.2f} | {pb:.2f} |")

if __name__ == "__main__":
    main()
