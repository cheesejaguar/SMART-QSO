"""
Orbital lifetime estimator for SMART-QSO (circular LEO, uncontrolled decay).

Independent cross-check for the orbital debris disclosure required by
47 CFR 97.207(g)(1)(vii) and the ODAR (NASA-STD-8719.14C). It is NOT a
substitute for NASA Debris Assessment Software (DAS) 3.2.x, the tool the
FCC rule names; every number used in a filing must be reproduced in DAS.

Model
-----
* Density: NRLMSIS 2.1 (``pymsis``), orbit-averaged over argument of
  latitude (latitude follows ``asin(sin i * sin u)``), 24 local solar times
  and four seasons.
* Decay: circular orbit, ``da/dt = -rho * (Cd * A / m) * sqrt(mu * a)``.
* Solar activity: constant (F10.7, Ap) bounds, or an approximate
  solar-cycle profile (``cycle_profile``) whose parameters are assumptions to
  be replaced with the current NOAA SWPC prediction before formal use.

Run ``python orbit_lifetime.py --help`` for the command line.
"""

from __future__ import annotations

import argparse
import math
from dataclasses import dataclass
from datetime import date, timedelta
from typing import Callable, Dict, List, Tuple

import numpy as np

MU_EARTH = 3.986004418e14  # m^3/s^2
R_EARTH = 6378.137e3  # m
REENTRY_ALT_KM = 180.0  # remaining life below this is < ~1 day

TABLE_ALTS_KM = np.arange(150.0, 805.0, 10.0)
TABLE_F107 = np.array([65.0, 70.0, 80.0, 100.0, 120.0, 150.0, 180.0, 220.0])
SEASON_DATES = ("2027-03-20", "2027-06-21", "2027-09-22", "2027-12-21")


def ap_for_f107(f107: float) -> float:
    """Daily Ap paired with a given F10.7 (assumption: quiet-to-moderate)."""
    return 4.0 + 0.08 * max(0.0, f107 - 65.0)


@dataclass(frozen=True)
class Spacecraft:
    """Drag-relevant properties of the spacecraft."""

    mass_kg: float
    area_m2: float  # orbit-averaged drag area (tumbling/attitude-averaged)
    cd: float = 2.2

    @property
    def ballistic_coeff(self) -> float:
        """Ballistic coefficient m / (Cd A) in kg/m^2."""
        return self.mass_kg / (self.cd * self.area_m2)


class DensityTable:
    """Orbit-averaged NRLMSIS 2.1 density versus altitude and F10.7."""

    def __init__(self, inclination_deg: float):
        import pymsis  # deferred so the module imports without it

        sin_i = math.sin(math.radians(min(inclination_deg, 180.0 - inclination_deg)))
        u = (np.arange(24) + 0.5) * (2.0 * math.pi / 24.0)
        lats = np.degrees(np.arcsin(sin_i * np.sin(u)))
        lons = np.arange(0.0, 360.0, 15.0)  # 00:00 UT -> 24 local times
        dates = np.array([np.datetime64(d + "T00:00") for d in SEASON_DATES])

        log_rho = np.empty((TABLE_F107.size, TABLE_ALTS_KM.size))
        for k, f107 in enumerate(TABLE_F107):
            ap = ap_for_f107(f107)
            out = pymsis.calculate(
                dates,
                lons,
                lats,
                TABLE_ALTS_KM,
                f107s=np.full(dates.size, f107),
                f107as=np.full(dates.size, f107),
                aps=np.full((dates.size, 7), ap),
                version=2.1,
            )
            rho = out[..., 0]  # total mass density, kg/m^3
            log_rho[k, :] = np.log(rho.mean(axis=(0, 1, 2)))
        self._log_rho = log_rho

    def density(self, alt_km: float, f107: float) -> float:
        """Orbit-averaged density (kg/m^3), log-linear interpolation."""
        f = float(np.clip(f107, TABLE_F107[0], TABLE_F107[-1]))
        h = float(np.clip(alt_km, TABLE_ALTS_KM[0], TABLE_ALTS_KM[-1]))
        by_f = np.array([np.interp(h, TABLE_ALTS_KM, row) for row in self._log_rho])
        return float(math.exp(np.interp(f, TABLE_F107, by_f)))


def constant_flux(f107: float) -> Callable[[date], float]:
    """Solar activity held at one F10.7 value (bounding case)."""
    return lambda _day: f107


def _half_cosine(t: float, t0: float, t1: float, v0: float, v1: float) -> float:
    x = (t - t0) / (t1 - t0)
    return v0 + (v1 - v0) * 0.5 * (1.0 - math.cos(math.pi * x))


# (decimal year, smoothed F10.7) turning points. Cycle 25 minimum/maximum are
# observed approximately; the cycle 25 end and cycle 26 values are ASSUMED.
CYCLE_POINTS: Tuple[Tuple[float, float], ...] = (
    (2019.9, 69.0),  # cycle 25 minimum (observed, approx.)
    (2024.8, 185.0),  # cycle 25 maximum (observed, approx.)
    (2030.5, 68.0),  # cycle 25/26 minimum (assumed)
    (2035.0, 150.0),  # cycle 26 maximum (assumed amplitude)
    (2041.0, 68.0),  # cycle 26/27 minimum (assumed)
    (2045.5, 150.0),  # cycle 27 maximum (assumed)
)


def cycle_profile(day: date) -> float:
    """Approximate smoothed F10.7 for a calendar day (see CYCLE_POINTS)."""
    t = day.year + (day.timetuple().tm_yday - 0.5) / 365.25
    pts = CYCLE_POINTS
    if t <= pts[0][0]:
        return pts[0][1]
    for (t0, v0), (t1, v1) in zip(pts, pts[1:]):
        if t <= t1:
            return _half_cosine(t, t0, t1, v0, v1)
    return pts[-1][1]


@dataclass
class DecayResult:
    """Outcome of one decay propagation."""

    lifetime_years: float
    reentered: bool
    history: List[Tuple[float, float]]  # (years since start, altitude km)


def propagate(
    table: DensityTable,
    sc: Spacecraft,
    start_alt_km: float,
    flux: Callable[[date], float],
    start: date = date(2027, 6, 1),
    max_years: float = 50.0,
) -> DecayResult:
    """Integrate circular-orbit decay until re-entry or ``max_years``."""
    a = R_EARTH + start_alt_km * 1e3
    inv_beta = 1.0 / sc.ballistic_coeff
    t_days = 0.0
    history = [(0.0, start_alt_km)]
    next_record = 30.0
    while t_days < max_years * 365.25:
        alt_km = (a - R_EARTH) / 1e3
        if alt_km <= REENTRY_ALT_KM:
            return DecayResult(t_days / 365.25, True, history)
        rho = table.density(alt_km, flux(start + timedelta(days=t_days)))
        rate = rho * inv_beta * math.sqrt(MU_EARTH * a)  # m/s
        step_days = 1.0 if alt_km > 250.0 else 0.05
        a -= rate * step_days * 86400.0
        t_days += step_days
        if t_days >= next_record:
            history.append((t_days / 365.25, (a - R_EARTH) / 1e3))
            next_record += 30.0
    return DecayResult(max_years, False, history)


SCENARIOS: Dict[str, Callable[[date], float]] = {
    "low (F10.7=70, bounding)": constant_flux(70.0),
    "moderate (F10.7=120)": constant_flux(120.0),
    "solar-cycle profile (nominal)": cycle_profile,
}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--alt", type=float, nargs="+", default=[400.0, 450.0, 500.0, 525.0, 550.0])
    parser.add_argument("--inc", type=float, default=97.4, help="inclination, deg")
    parser.add_argument("--mass", type=float, default=1.33, help="kg")
    parser.add_argument("--area", type=float, nargs="+", default=[0.015], help="mean drag area, m^2")
    parser.add_argument("--cd", type=float, default=2.2)
    parser.add_argument("--start", type=str, default="2027-06-01", help="deployment date")
    args = parser.parse_args()

    table = DensityTable(args.inc)
    start = date.fromisoformat(args.start)
    print(f"inclination {args.inc} deg, mass {args.mass} kg, Cd {args.cd}, start {start}")
    print("area_m2  alt_km  " + "  ".join(f"{name:>32s}" for name in SCENARIOS))
    for area in args.area:
        sc = Spacecraft(args.mass, area, args.cd)
        for alt in args.alt:
            cells = []
            for flux in SCENARIOS.values():
                res = propagate(table, sc, alt, flux, start)
                cells.append(f"{res.lifetime_years:>29.1f} yr" + ("" if res.reentered else "+"))
            print(f"{area:7.4f}  {alt:6.0f}  " + "  ".join(cells))


if __name__ == "__main__":
    main()
