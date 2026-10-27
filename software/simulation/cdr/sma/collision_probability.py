"""
Large-object collision probability over the orbital lifetime
(NASA-STD-8719.14C Requirement 4.5-1, P = F * A * T, eq. 4.5-1;
47 CFR 97.207(g)(1)(iv)), without NASA DAS or ORDEM.

Method
------
* Spatial density of objects >= ~10 cm by 10 km shell, from the public
  CelesTrak SATCAT (https://celestrak.org/pub/satcat.csv): every object on orbit
  (no decay date, Earth-centred), its time at each radius from Kepler's
  equation, dt ~ r dr / sqrt((r - rp)(ra - r)).
* The catalogue under-counts objects >= 10 cm (uncatalogued and analyst objects);
  ESA's statistical model puts the >= 10 cm population near 1.4 times the
  catalogue (ESA Space Environment Report 2025). This script multiplies by 2.0 to
  cover that and growth in the 450-550 km band before and during the decay.
* Flux F = S * v_rel with v_rel = 12 km/s (upper part of the LEO impact-velocity
  range for a sun-synchronous orbit; conservative).
* Collision cross-section: the largest average cross-section (wings and antennas
  deployed). The decay history comes from the project lifetime estimator with the
  smallest drag area (longest life). Area and life are taken from different cases
  on purpose, which over-states F*A*T.

Run: python3 collision_probability.py path/to/satcat.csv   (needs numpy, pymsis)
"""

from __future__ import annotations

import csv
import math
import sys
from datetime import date
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "orbit_lifetime"))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from orbit_lifetime import DensityTable, Spacecraft, constant_flux, propagate  # noqa: E402
from debris_lifetime import AREAS, SCENARIOS, START  # noqa: E402

RE_KM = 6378.137
BIN_KM = 10.0
EDGES = np.arange(150.0, 2010.0, BIN_KM)
UNDERCOUNT = 2.0
V_REL_KM_S = 12.0


def spatial_density(satcat_path: str):
    counts = np.zeros(EDGES.size - 1)
    n_obj = 0
    newest = ""
    with open(satcat_path, newline="") as f:
        for row in csv.DictReader(f):
            if row["DECAY_DATE"] or row["ORBIT_CENTER"] != "EA":
                continue
            newest = max(newest, row["LAUNCH_DATE"])
            try:
                hp, ha = float(row["PERIGEE"]), float(row["APOGEE"])
            except ValueError:
                continue
            if hp > 2000.0 or ha < 150.0:
                continue
            n_obj += 1
            rp, ra = RE_KM + hp, RE_KM + max(ha, hp)
            if ra - rp < 2.0:
                k = int((hp - EDGES[0]) // BIN_KM)
                if 0 <= k < counts.size:
                    counts[k] += 1.0
                continue
            # time at radius, sampled in eccentric anomaly (uniform in mean anomaly)
            e_anom = np.linspace(0.0, 2 * math.pi, 721)
            a = 0.5 * (rp + ra)
            ecc = (ra - rp) / (ra + rp)
            r = a * (1 - ecc * np.cos(e_anom))
            w = 1 - ecc * np.cos(e_anom)  # dM/dE
            h = r - RE_KM
            hist, _ = np.histogram(h, bins=EDGES, weights=w)
            counts += hist / w.sum()
    r_mid = RE_KM + 0.5 * (EDGES[:-1] + EDGES[1:])
    vol = 4 * math.pi * r_mid ** 2 * BIN_KM  # km^3
    return counts / vol, n_obj, newest  # objects per km^3


def flux_per_m2_yr(s_km3: np.ndarray) -> np.ndarray:
    # objects/km^3 * km/s -> objects/km^2/s -> per m^2 per year
    return s_km3 * UNDERCOUNT * V_REL_KM_S * 3.15576e7 / 1e6


def main(path: str):
    s, n, newest = spatial_density(path)
    flux = flux_per_m2_yr(s)
    mids = 0.5 * (EDGES[:-1] + EDGES[1:])
    print(f"SATCAT objects on orbit below 2000 km: {n} (latest launch date in file {newest})")
    for h in (400, 450, 475, 500, 525, 550, 600):
        k = int((h - EDGES[0]) // BIN_KM)
        print(f"  {h} km: catalogue spatial density {s[k]:.2e} /km^3, "
              f"design flux (x{UNDERCOUNT}, {V_REL_KM_S} km/s) {flux[k]:.2e} /m^2/yr")
    table = DensityTable(97.4)
    a_coll = AREAS["wings deployed, antennas deployed"]
    print(f"\nCollision cross-section used: {a_coll:.4f} m^2")
    print("| Deployment (km) | Mass (kg) | Solar case | Drag case | Life (yr) | P(collision >= 10 cm) | Margin to 0.001 |")
    print("|---|---|---|---|---|---|---|")
    cases = [("F10.7 = 70 constant (bound)", constant_flux(70.0)),
             ("NOAA predicted + cycle 26 max 140 (nominal)", SCENARIOS["NOAA predicted + cycle 26 max 140 (nominal)"])]
    worst = 0.0
    for alt in (475.0, 485.0, 500.0, 525.0):
        for mass in (1.6, 2.0):
            for sname, prof in cases:
                for dname in ("wings failed (stowed), no antenna credit", "wings deployed, no antenna credit"):
                    sc = Spacecraft(mass, AREAS[dname], 2.2)
                    res = propagate(table, sc, alt, prof, START, max_years=60.0)
                    hist = res.history + [(res.lifetime_years, 180.0)]
                    p = 0.0
                    for (t0, h0), (t1, h1) in zip(hist, hist[1:]):
                        hm = 0.5 * (h0 + h1)
                        k = int((hm - EDGES[0]) // BIN_KM)
                        p += flux[max(k, 0)] * a_coll * (t1 - t0)
                    worst = max(worst, p)
                    print(f"| {alt:.0f} | {mass} | {sname} | {dname} | {res.lifetime_years:.1f} | {p:.1e} | x{1e-3/p:.0f} |")
    print(f"\nWorst case P = {worst:.1e} (requirement < 1e-3)")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "satcat.csv")
