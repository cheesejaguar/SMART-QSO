"""
Re-entry survivability and human-casualty estimate for SMART-QSO
(NASA-STD-8719.14C Requirement 4.7-1; 47 CFR 97.207(g)(1)(vii)(D)(2)).

NASA DAS 3.2.x could not be obtained or run in the review environment. This
script implements the same class of model that DAS uses for its re-entry risk
assessment (object-oriented, lumped-mass "heat of ablation" demise, objects
released at 78 km, NASA-STD-8719.14C Section 4.7.4), so that the candidate
survivors can be identified and the margin to 1:10,000 bounded before the DAS run:

* 3-DOF ballistic trajectory over a spherical, non-rotating Earth; density from
  NRLMSIS 2.1 (pymsis) at moderate activity, orbit-averaged in latitude.
* The intact spacecraft is flown from 120 km to the 78 km breakup altitude that
  DAS uses; every component is then released at that state at 300 K.
* Heating: free-molecular q = 0.5 * alpha * rho * v^3 bridged to the
  Detra-Kemp-Riddell stagnation-point correlation
  q = 1.1035e8 / sqrt(Rn) * sqrt(rho / 1.225) * (v / 7925)^3.15 W/m^2
  by q = (q_fm^-2 + q_c^-2)^-1/2; heat absorbed over the average projected
  (tumbling) area with a shape factor of 0.8 in continuum flow; re-radiation
  eps * sigma * T^4 from the whole surface; no oxidation heating (conservative).
* Demise when the absorbed heat reaches m (cp (Tm - 300) + hf).
* Survivors are flown to the ground; impact kinetic energy above 15 J counts;
  debris casualty area DCA = sum (0.6 + sqrt(A))^2 (NASA-STD-8719.14C eq. 4.7-1);
  casualty expectation E = DCA * PD (eq. 4.7-2), PD averaged over the latitudes
  the orbit covers, from the population table below.

Population table: approximate share of world population by 10-degree latitude
band (built from the gridded population distribution as published in Kummu and
Varis, Applied Geography 31 (2011) 495-507, Fig. 1, read to about +/- 1 point),
scaled to 8.8e9 people for a re-entry around 2034 (UN World Population
Prospects 2024, medium variant, about 8.8 billion in 2034-35). The answer is
reported with a factor-of-two band to cover the table's precision.

Run: python3 reentry_casualty.py      (needs numpy and pymsis)
"""

from __future__ import annotations

import math
from dataclasses import dataclass

import numpy as np

MU = 3.986004418e14
RE = 6378.137e3
G0 = 9.80665
SIGMA = 5.670374e-8

# ----------------------------------------------------------------- atmosphere
_ALT_KM = np.arange(0.0, 130.1, 1.0)
_LOG_RHO = None


def _density_table():
    global _LOG_RHO
    if _LOG_RHO is None:
        import pymsis

        lats = np.array([-60.0, -30.0, 0.0, 30.0, 60.0])
        lons = np.arange(0.0, 360.0, 60.0)
        dates = np.array([np.datetime64("2034-03-20T00:00"), np.datetime64("2034-09-22T00:00")])
        out = pymsis.calculate(dates, lons, lats, _ALT_KM, f107s=np.full(2, 120.0),
                               f107as=np.full(2, 120.0), aps=np.full((2, 7), 8.0), version=2.1)
        _LOG_RHO = np.log(out[..., 0].mean(axis=(0, 1, 2)))
    return _LOG_RHO


def density(h_m: float) -> float:
    h = min(max(h_m / 1e3, 0.0), 130.0)
    return float(math.exp(np.interp(h, _ALT_KM, _density_table())))


def mean_free_path(rho: float) -> float:
    # hard-sphere estimate, air: lambda ~ 1.0e-7 * (1.225 / rho) m at sea level scale
    return 6.6e-8 * 1.225 / max(rho, 1e-15)


# ------------------------------------------------------------------ materials
@dataclass(frozen=True)
class Material:
    name: str
    rho: float  # kg/m^3
    cp: float  # J/kg/K, mean 300 K to melt
    tm: float  # K
    hf: float  # J/kg
    eps: float


AL = Material("aluminium 6061/7075", 2700, 1000, 867, 386e3, 0.14)
SS = Material("stainless 300-series / A286", 8000, 590, 1700, 286e3, 0.35)
STEEL = Material("Ni-plated steel (cell can)", 7850, 600, 1800, 270e3, 0.35)
TI = Material("Ti-6Al-4V", 4430, 680, 1943, 393e3, 0.30)
CU = Material("copper", 8960, 420, 1358, 205e3, 0.10)
NIFE = Material("NiFe/ferritic core alloy", 8700, 520, 1720, 290e3, 0.30)
ALUMINA = Material("alumina-class ceramic", 3800, 1100, 2327, 1070e3, 0.40)


@dataclass
class Part:
    name: str
    material: Material
    mass: float  # kg, per item
    count: int
    shape: str  # "cyl", "box", "sphere"
    dims: tuple  # cyl (diameter, length); box (a, b, c)

    def areas(self):
        """(average projected area, total surface area) for a tumbling object."""
        if self.shape == "cyl":
            d, l = self.dims
            surf = math.pi * d * l + 2 * math.pi * d * d / 4
        elif self.shape == "box":
            a, b, c = self.dims
            surf = 2 * (a * b + b * c + a * c)
        else:
            d = self.dims[0]
            surf = math.pi * d * d
        return surf / 4.0, surf  # convex: average projected = surface / 4


# ------------------------------------------------------------------ dynamics
def cd_of(kn: float) -> float:
    """Tumbling-object drag coefficient, free-molecular 2.2 to continuum 1.0."""
    if kn >= 10:
        return 2.2
    if kn <= 0.01:
        return 1.0
    x = (math.log10(kn) + 2) / 3.0
    return 1.0 + 1.2 * x


def fly(mass, area, surf, rn, mat, state, heat=True, alpha=0.9, kc=0.8, dt=0.02):
    """Integrate a released component from state (h, v, gamma) until demise or
    ground. Returns (demised, h_end, v_end, q_absorbed, state_at_78km)."""
    h, v, gam = state
    T = 300.0
    q_need = mass * (mat.cp * (mat.tm - 300.0) + mat.hf) if heat else float("inf")
    q_melt = 0.0
    q_abs = 0.0
    s78 = None
    while h > 0.0:
        rho = density(h)
        r = RE + h
        g = MU / r / r
        kn = mean_free_path(rho) / max(rn, 1e-3)
        cd = cd_of(kn)
        drag = 0.5 * rho * v * v * cd * area / mass
        dv = -drag - g * math.sin(gam)
        dg = (v / r - g / v) * math.cos(gam) if v > 1.0 else 0.0
        dh = v * math.sin(gam)
        if heat:
            q_fm = 0.5 * alpha * rho * v ** 3
            q_c = 1.1035e8 / math.sqrt(rn) * math.sqrt(rho / 1.225) * (v / 7925.0) ** 3.15
            q = 1.0 / math.sqrt(q_fm ** -2 + q_c ** -2) if q_fm > 0 and q_c > 0 else 0.0
            p_in = q * area * (1.0 if kn > 1 else kc)
            p_out = mat.eps * SIGMA * surf * (T ** 4 - 250.0 ** 4)
            net = (p_in - p_out) * dt
            q_abs += max(p_in * dt, 0.0)
            if T < mat.tm:
                T += net / (mass * mat.cp)
                T = max(T, 200.0)
            else:
                q_melt += net
                if q_melt >= mass * mat.hf:
                    return True, h, v, q_abs, s78
                if q_melt < 0:
                    T = mat.tm - 1.0
                    q_melt = 0.0
        step = dt if (h < 100e3 or not heat) else dt * 5
        v += dv * step
        gam += dg * step
        h += dh * step
        if s78 is None and h <= 78e3:
            s78 = (h, v, gam)
        if not heat and h <= 78e3:
            return False, h, v, q_abs, s78
    return False, 0.0, v, q_abs, s78


def _gamma_fix(mass, area):
    """Intact spacecraft from 120 km, circular speed, gamma -0.1 deg, down to the
    78 km breakup altitude; returns (h, v, gamma) there."""
    h0 = 120e3
    v0 = math.sqrt(MU / (RE + h0))
    h, v, gam = h0, v0, math.radians(-0.1)
    dt = 0.05
    while h > 78e3:
        rho = density(h)
        r = RE + h
        g = MU / r / r
        drag = 0.5 * rho * v * v * 2.2 * area / mass
        v += (-drag - g * math.sin(gam)) * dt
        gam += ((v / r - g / v) * math.cos(gam)) * dt
        h += v * math.sin(gam) * dt
    return h, v, gam


# ------------------------------------------------------------- population
# share of world population (percent) by latitude band (lower edge, deg)
POP_SHARE = {
    60: 0.3, 50: 6.0, 40: 13.0, 30: 23.0, 20: 26.0, 10: 12.0, 0: 8.0,
    -10: 7.0, -20: 2.5, -30: 3.0, -40: 1.5, -50: 0.1, -60: 0.0, -70: 0.0, 70: 0.05,
    80: 0.0, -80: 0.0,
}
WORLD_POP = 8.8e9


def orbit_population_density(inc_deg: float) -> float:
    """Average population density (1/m^2) under the ground track of a circular
    orbit of inclination inc (latitude dwell weighting, uniform longitude)."""
    i = math.radians(min(inc_deg, 180 - inc_deg))
    tot = sum(POP_SHARE.values())
    pd = 0.0
    for lo, share in POP_SHARE.items():
        p1, p2 = math.radians(lo), math.radians(lo + 10)
        # dwell fraction: integral of cos(phi)/(pi sqrt(sin^2 i - sin^2 phi)) dphi
        a = max(min(math.sin(p1) / math.sin(i), 1.0), -1.0)
        b = max(min(math.sin(p2) / math.sin(i), 1.0), -1.0)
        frac = (math.asin(b) - math.asin(a)) / math.pi
        area = 2 * math.pi * RE ** 2 * (math.sin(p2) - math.sin(p1))
        pd += frac * (WORLD_POP * share / tot) / area
    return pd


# ------------------------------------------------------------------- parts
PARTS = [
    Part("Structure frame (COTS 1U, Al)", AL, 0.110, 1, "box", (0.100, 0.100, 0.1135)),
    Part("Jetson heat spreader (Al)", AL, 0.065, 1, "box", (0.090, 0.090, 0.003)),
    Part("Battery clamp plates (Al)", AL, 0.017, 2, "box", (0.080, 0.070, 0.002)),
    Part("18650 cell, whole cell as steel cylinder (bounding)", STEEL, 0.050, 4, "cyl", (0.0184, 0.065)),
    Part("18650 cell can only (Ni-plated steel)", STEEL, 0.008, 4, "cyl", (0.0184, 0.065)),
    Part("Stainless fastener M3 x 10 (largest)", SS, 0.0009, 40, "cyl", (0.004, 0.012)),
    Part("Stainless fastener M2.5 x 6", SS, 0.0004, 40, "cyl", (0.0035, 0.008)),
    Part("Ti-6Al-4V screw M2.5 x 8 (battery mount)", TI, 0.00030, 4, "cyl", (0.0035, 0.010)),
    Part("Ti-6Al-4V necked standoff (alternative mount)", TI, 0.0015, 4, "cyl", (0.005, 0.012)),
    Part("Stainless hinge pin 2 mm x 98 mm", SS, 0.0025, 2, "cyl", (0.002, 0.098)),
    Part("Stainless hinge spring", SS, 0.0006, 4, "cyl", (0.006, 0.006)),
    Part("Antenna tape, 301 stainless, 0.1 x 5 x 500 mm", SS, 0.0020, 2, "box", (0.500, 0.005, 0.0001)),
    Part("Gyro M-G370PDT0, case as aluminium", AL, 0.010, 1, "box", (0.024, 0.024, 0.010)),
    Part("Gyro M-G370PDT0, case as stainless (bounding)", SS, 0.010, 1, "box", (0.024, 0.024, 0.010)),
    Part("GNSS ceramic patch 25 x 25 x 4 mm (alumina-class)", ALUMINA, 0.0095, 1, "box", (0.025, 0.025, 0.004)),
    Part("Magnetorquer rod CR0002 as solid NiFe (bounding)", NIFE, 0.0165, 3, "cyl", (0.010, 0.070)),
    Part("Air-core magnetorquer coil (Cu), if kept", CU, 0.046, 3, "box", (0.090, 0.090, 0.005)),
    Part("RF connector body, stainless", SS, 0.002, 4, "cyl", (0.006, 0.012)),
]


def run(alpha=0.9, kc=0.8, label="nominal heating"):
    s78 = _gamma_fix(1.8, 0.0164)
    print(f"[{label}] breakup state: h {s78[0]/1e3:.1f} km, v {s78[1]:.0f} m/s, "
          f"gamma {math.degrees(s78[2]):.2f} deg")
    pd = orbit_population_density(97.4)
    rows = []
    dca_total = 0.0
    for p in PARTS:
        a, s = p.areas()
        rn = math.sqrt(a / math.pi)
        dem, h, v, qabs, _ = fly(p.mass, a, s, rn, p.material, s78, alpha=alpha, kc=kc)
        ke = 0.0 if dem else 0.5 * p.mass * v * v
        dca = 0.0
        if not dem and ke > 15.0:
            dca = p.count * (0.6 + math.sqrt(a)) ** 2
        need = p.mass * (p.material.cp * (p.material.tm - 300) + p.material.hf)
        rows.append((p, dem, h, v, ke, dca, qabs / need))
    print("| Component | Material | Mass each (g) | Qty | Demise altitude (km) or impact | "
          "Heat absorbed / heat to demise | Impact KE (J) | DCA (m^2) |")
    print("|---|---|---|---|---|---|---|---|")
    for p, dem, h, v, ke, dca, ratio in rows:
        where = f"{h/1e3:.0f} km" if dem else f"survives, {v:.0f} m/s"
        print(f"| {p.name} | {p.material.name} | {p.mass*1e3:.1f} | {p.count} | {where} | "
              f"{ratio:.2f} | {ke:.1f} | {dca:.2f} |")
    return rows, pd


def summarise(rows, pd, exclude=()):
    dca = sum(r[5] for r in rows if r[0].name not in exclude)
    return dca, dca * pd


if __name__ == "__main__":
    pd = orbit_population_density(97.4)
    print(f"Average population density under a 97.4 deg orbit (2034): {pd:.3e} per m^2")
    print(f"DCA that gives 1:10,000: {1e-4/pd:.1f} m^2 (factor-2 band "
          f"{0.5e-4/pd:.1f}-{2e-4/pd:.1f} m^2)")
    for alpha, kc, label in ((0.9, 0.8, "nominal heating"), (0.45, 0.4, "heating halved (sensitivity)")):
        rows, pd = run(alpha, kc, label)
        # one battery representation and one gyro representation at a time
        excl_bound = ("18650 cell can only (Ni-plated steel)", "Gyro M-G370PDT0, case as aluminium",
                      "Ti-6Al-4V screw M2.5 x 8 (battery mount)", "Air-core magnetorquer coil (Cu), if kept")
        dca, e = summarise(rows, pd, excl_bound)
        print(f"[{label}] bounding set (whole cells as steel, steel gyro case, Ti standoffs, rods): "
              f"DCA {dca:.2f} m^2, casualty risk 1:{(1/e) if e > 0 else float('inf'):,.0f}")
        excl_nom = ("18650 cell, whole cell as steel cylinder (bounding)",
                    "Gyro M-G370PDT0, case as stainless (bounding)",
                    "Ti-6Al-4V necked standoff (alternative mount)",
                    "Magnetorquer rod CR0002 as solid NiFe (bounding)")
        dca, e = summarise(rows, pd, excl_nom)
        print(f"[{label}] nominal set (cans, Al gyro case, Ti screws, air-core coils): "
              f"DCA {dca:.2f} m^2, casualty risk 1:{(1/e) if e > 0 else float('inf'):,.0f}")
        print()
