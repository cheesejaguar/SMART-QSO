"""
CDR orbital-lifetime and post-mission-disposal analysis for SMART-QSO.

Uses the project estimator (software/simulation/orbit_lifetime/orbit_lifetime.py,
NRLMSIS 2.1, circular decay) with:

* average cross-sections computed by the NASA-STD-8719.14C para 4.3.4.2.2(a)(2)(b)
  rule, (Amax + A1 + A2) / 2, for the stowed and deployed geometry of
  hardware/eps/SOLAR_ARRAY_DESIGN.md section 3.1;
* masses 1.6 kg (nominal) and 1.85 kg (bound) from BASELINE Rev E section 4 (MEV
  1.73 kg lies between them), plus 2.0 kg (the CDS Rev 14.1 design limit); the
  shortest-life case also at the CBE, 1.49 kg (the lightest the spacecraft can be);
* deployment on the planning launch date, 2028-11-01 (BASELINE Rev E section 3,
  the last Transporter flight of 2028), at the band 465-485 km, the 475 km target,
  and 450, 500 and 525 km for the sensitivity;
* solar-activity projections anchored to the NOAA SWPC predicted cycle
  (services.swpc.noaa.gov/json/solar-cycle/predicted-solar-cycle.json,
  retrieved 2026-10-27; embedded below so the run is reproducible offline)
  up to 2030-12, then an assumed cycle 26.

It is not NASA DAS. NASA DAS 3.2.x could not be obtained or run in the review
environment (it is distributed under a NASA software usage agreement and runs on
Windows); every filed value must be reproduced in DAS.

Run: python3 debris_lifetime.py   (needs numpy and pymsis)
"""

from __future__ import annotations

import math
import sys
from datetime import date
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "orbit_lifetime"))

from orbit_lifetime import DensityTable, Spacecraft, constant_flux, propagate  # noqa: E402

# NOAA SWPC predicted F10.7 (monthly): (yyyy-mm, predicted, low, high)
SWPC_PRED = (
    ("2026-04", 133.4, 126.4, 140.8),
    ("2026-05", 132.7, 125.2, 139.7),
    ("2026-06", 131.7, 123.4, 138.3),
    ("2026-07", 129.0, 119.9, 135.4),
    ("2026-08", 126.9, 117.3, 133.3),
    ("2026-09", 125.7, 115.7, 132.5),
    ("2026-10", 125.9, 115.0, 132.8),
    ("2026-11", 126.0, 114.4, 133.1),
    ("2026-12", 125.8, 113.3, 133.3),
    ("2027-01", 124.9, 111.9, 133.3),
    ("2027-02", 123.3, 110.1, 132.6),
    ("2027-03", 121.4, 107.9, 131.3),
    ("2027-04", 118.8, 105.3, 129.6),
    ("2027-05", 116.2, 103.0, 127.4),
    ("2027-06", 113.4, 100.6, 125.1),
    ("2027-07", 110.8, 98.2, 122.9),
    ("2027-08", 108.3, 96.0, 120.6),
    ("2027-09", 105.9, 93.8, 118.3),
    ("2027-10", 103.8, 92.1, 116.2),
    ("2027-11", 101.7, 90.5, 114.0),
    ("2027-12", 99.9, 89.3, 112.0),
    ("2028-01", 98.0, 88.1, 110.0),
    ("2028-02", 96.3, 87.1, 108.1),
    ("2028-03", 94.7, 86.2, 106.2),
    ("2028-04", 93.1, 85.4, 104.3),
    ("2028-05", 91.7, 84.5, 102.6),
    ("2028-06", 90.2, 83.2, 100.9),
    ("2028-07", 88.9, 82.0, 99.2),
    ("2028-08", 87.5, 80.6, 97.6),
    ("2028-09", 86.3, 79.5, 96.0),
    ("2028-10", 85.1, 78.2, 94.4),
    ("2028-11", 84.0, 76.9, 92.8),
    ("2028-12", 83.0, 75.8, 91.4),
    ("2029-01", 82.0, 74.8, 90.0),
    ("2029-02", 81.0, 73.8, 88.8),
    ("2029-03", 80.2, 72.8, 87.8),
    ("2029-04", 79.4, 71.8, 86.7),
    ("2029-05", 78.6, 70.9, 85.7),
    ("2029-06", 77.8, 70.1, 84.9),
    ("2029-07", 77.2, 69.4, 84.3),
    ("2029-08", 76.5, 69.0, 83.8),
    ("2029-09", 75.9, 68.6, 83.2),
    ("2029-10", 75.3, 68.4, 82.7),
    ("2029-11", 74.8, 68.4, 82.2),
    ("2029-12", 74.3, 68.4, 81.7),
    ("2030-01", 73.8, 68.2, 81.0),
    ("2030-02", 73.3, 67.9, 80.3),
    ("2030-03", 73.0, 67.8, 79.6),
    ("2030-04", 72.6, 67.8, 78.8),
    ("2030-05", 72.2, 67.8, 78.0),
    ("2030-06", 71.9, 67.8, 77.2),
    ("2030-07", 71.6, 67.8, 76.3),
    ("2030-08", 71.3, 67.8, 75.5),
    ("2030-09", 71.0, 67.8, 74.7),
    ("2030-10", 70.8, 67.8, 74.0),
    ("2030-11", 70.5, 67.8, 73.3),
    ("2030-12", 70.3, 67.8, 72.7)
)

START = date(2028, 11, 1)  # BASELINE Rev E section 3 planning launch date (about 2028-11)
MISSION_YEARS = 2.0  # design life (BASELINE section 2)
FCC_POST_MISSION_YEARS = 5.0  # 47 CFR 97.207(g)(1)(vii)(D)(1)
NASA_POST_MISSION_YEARS = 25.0  # NASA-STD-8719.14C Requirement 4.6-1a


def _dec_year(tag: str) -> float:
    y, m = (int(x) for x in tag.split("-"))
    return y + (m - 0.5) / 12.0


def _half_cos(t, t0, t1, v0, v1):
    x = (t - t0) / (t1 - t0)
    return v0 + (v1 - v0) * 0.5 * (1.0 - math.cos(math.pi * x))


def swpc_profile(column: int, c26_max: float, c26_peak: float = 2035.5,
                 c26_min_end: float = 2041.5, floor: float = 68.0):
    """F10.7(day): NOAA column (1 predicted, 2 low, 3 high) to 2030-12, then
    a half-cosine rise to an assumed cycle 26 maximum and decline."""
    pts = [(_dec_year(r[0]), r[column]) for r in SWPC_PRED]
    t_end, v_end = pts[-1]
    t_min = t_end + 0.5  # cycle 25/26 minimum assumed mid-2031

    def f(day: date) -> float:
        t = day.year + (day.timetuple().tm_yday - 0.5) / 365.25
        if t <= pts[0][0]:
            return pts[0][1]
        if t <= t_end:
            for (a, va), (b, vb) in zip(pts, pts[1:]):
                if t <= b:
                    return va + (vb - va) * (t - a) / (b - a)
        if t <= t_min:
            return _half_cos(t, t_end, t_min, v_end, floor)
        if t <= c26_peak:
            return _half_cos(t, t_min, c26_peak, floor, c26_max)
        if t <= c26_min_end:
            return _half_cos(t, c26_peak, c26_min_end, c26_max, floor)
        # cycle 27 assumed equal to cycle 26
        period = c26_min_end - t_min
        return f(date.fromordinal(day.toordinal() - int(period * 365.25)))

    return f


SCENARIOS = {
    "NOAA predicted + cycle 26 max 140 (nominal)": swpc_profile(1, 140.0),
    "NOAA low + weak cycle 26 max 110 (conservative)": swpc_profile(2, 110.0),
    "F10.7 = 70 constant (bound)": constant_flux(70.0),
}

# Short-life case, for mission success (does the spacecraft last its design life?)
SHORT_LIFE = ("NOAA high + strong cycle 26 max 180", swpc_profile(3, 180.0))


def avg_cross_section(views):
    """NASA-STD-8719.14C 4.3.4.2.2(a)(2)(b): (Amax + A1 + A2) / 2."""
    return sum(views) / 2.0


# Geometry, metres. Body 0.100 x 0.100 x 0.1135 (CDS Rev 14.1); wings 0.083 x 0.098
# panels hinged on the +/-X edges of +Y, deployed into the +Y plane
# (SOLAR_ARRAY_DESIGN section 3.1); panel thickness 2 mm.
BODY_Y = 0.100 * 0.1135  # view along Y (or X)
BODY_Z = 0.100 * 0.100  # view along Z
WING = 0.083 * 0.098
WING_EDGE_Z = 0.083 * 0.002
# Antenna tapes (ISIS AntS monopole set, 2 x 0.50 m + 2 x 0.17 m), 5 mm wide,
# deployed in the end-face plane: each tape seen face-on in one view, edge-on in the others.
TAPE = (2 * 0.50 + 2 * 0.17) * 0.005

AREAS = {
    "wings deployed, no antenna credit": avg_cross_section(
        [BODY_Y + 2 * WING, BODY_Y, BODY_Z + 2 * WING_EDGE_Z]),
    "wings deployed, antennas deployed": avg_cross_section(
        [BODY_Y + 2 * WING + TAPE / 2, BODY_Y + TAPE / 2, BODY_Z + 2 * WING_EDGE_Z + TAPE]),
    "wings failed (stowed), no antenna credit": 0.25 * (2 * BODY_Z + 4 * BODY_Y),
}

MASSES = (1.6, 1.85, 2.0)
ALTS = (450.0, 465.0, 475.0, 485.0, 500.0, 525.0)
SHORT_LIFE_MASSES = (1.49, 1.6)  # CBE and the nominal analysis mass


def run():
    print(f"deployment {START}, Cd 2.2, mission {MISSION_YEARS} yr")
    for name, a in AREAS.items():
        print(f"  average cross-section, {name}: {a:.4f} m^2")
    table = DensityTable(97.4)
    results = {}
    for aname in ("wings deployed, no antenna credit", "wings failed (stowed), no antenna credit"):
        area = AREAS[aname]
        for mass in MASSES:
            sc = Spacecraft(mass, area, 2.2)
            for alt in ALTS:
                for sname, flux in SCENARIOS.items():
                    r = propagate(table, sc, alt, flux, START, max_years=60.0)
                    results[(aname, mass, alt, sname)] = r.lifetime_years
    print()
    snames = list(SCENARIOS)
    print("| Case | Mass (kg) | Alt (km) | " + " | ".join(snames) + " |")
    print("|---|---|---|" + "---|" * len(snames))
    for aname in ("wings deployed, no antenna credit", "wings failed (stowed), no antenna credit"):
        for mass in MASSES:
            for alt in ALTS:
                cells = []
                for s in snames:
                    life = results[(aname, mass, alt, s)]
                    post = max(0.0, life - MISSION_YEARS)
                    flag = "" if post <= FCC_POST_MISSION_YEARS else " **X**"
                    cells.append(f"{life:.1f} ({post:.1f}){flag}")
                print(f"| {aname} | {mass} | {alt:.0f} | " + " | ".join(cells) + " |")
    print("\nCells: total life in years (post-mission life after a 2-year mission); "
          "X = exceeds the FCC 5-year rule.")

    # Highest deployment altitude meeting the FCC rule (life <= mission + 5 yr), by bisection
    print("\nHighest deployment altitude (km) with post-mission life <= 5 yr after a 2-yr mission:")
    print("| Case | Mass (kg) | " + " | ".join(snames) + " |")
    print("|---|---|" + "---|" * len(snames))
    limit = MISSION_YEARS + FCC_POST_MISSION_YEARS
    for aname in ("wings deployed, no antenna credit", "wings failed (stowed), no antenna credit"):
        for mass in MASSES:
            sc = Spacecraft(mass, AREAS[aname], 2.2)
            cells = []
            for s_ in snames:
                lo, hi = 400.0, 560.0
                for _ in range(12):
                    mid = 0.5 * (lo + hi)
                    life = propagate(table, sc, mid, SCENARIOS[s_], START, max_years=60.0).lifetime_years
                    lo, hi = (mid, hi) if life <= limit else (lo, mid)
                cells.append(f"{lo:.0f}")
            print(f"| {aname} | {mass} | " + " | ".join(cells) + " |")

    # Shortest life (design-life check, mission success L2/L3)
    print(f"\nShortest life, {SHORT_LIFE[0]}, largest area (antennas deployed):")
    for mass in SHORT_LIFE_MASSES:
        sc = Spacecraft(mass, AREAS["wings deployed, antennas deployed"], 2.2)
        cells = []
        for alt in (450.0, 460.0, 465.0, 470.0, 475.0, 485.0):
            r = propagate(table, sc, alt, SHORT_LIFE[1], START, max_years=60.0)
            cells.append(f"{alt:.0f} km {r.lifetime_years:.1f} yr")
        print(f"  {mass} kg: " + ", ".join(cells))
    return results


if __name__ == "__main__":
    run()
