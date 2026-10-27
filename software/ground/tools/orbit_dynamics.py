#!/usr/bin/env python3
"""
Force model and orbit propagator for the SMART-QSO orbit determination
(SQSO-GND-002 §4.3).

Accelerations in TEME (orbit_frames.py), m and s:

- Gravity: spherical harmonics to degree and order N in the Earth-fixed
  frame (the V/W recursion of Montenbruck and Gill, Satellite Orbits, §3.2).
  Built in: EGM96 to 4 x 4 (the coefficients below). A larger field (EGM2008
  to 8 x 8 or more, as the D6 research recommends) is read from an ICGEM
  .gfc file. A zonal-only field takes a faster closed form.
- Drag: piecewise-exponential atmosphere (Vallado, Fundamentals of
  Astrodynamics, Table 8-4) at the approximate geodetic height, co-rotating;
  the ballistic term Cd A / m of the baseline (BASELINE §3: Cd 2.2,
  0.0247 m^2 with the wings deployed, 1.6 kg) times a scale factor k that the
  fit estimates. An optional diurnal bulge (used by the simulated truth).
- Sun and Moon (low-precision ephemerides, Vallado Algorithms 29 and 31):
  optional; their effect over a day is tens of metres.

Integration: fixed-step Adams-Bashforth-Moulton (predict-evaluate-correct-
evaluate, order 10) on a uniform 60 s grid, started with Dormand-Prince 5
substeps, then Lagrange interpolation (degree 9) to the requested times. Two
force evaluations a step. Against a 1 s Runge-Kutta reference the error is
under 0.1 m a day (test_orbit_determination.py). Many trajectories (the
nominal and the finite-difference perturbations) are integrated together.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Optional, Sequence, Tuple

import numpy as np

import orbit_frames as fr

#: EGM96 constants (the built-in field uses them)
GM = 3.986004415e14
RE = 6378136.3
#: WGS-84 flattening, for the geodetic height of the drag model
FLATTENING = 1.0 / 298.257223563
GM_SUN, GM_MOON = 1.32712440018e20, 4.9028e12
AU = 1.495978707e11

#: Baseline ballistic term, m^2/kg (BASELINE §3)
BC_NOMINAL = 2.2 * 0.0247 / 1.6

#: EGM96 fully normalized coefficients to degree and order 4: (n, m): (C, S)
EGM96_4X4: Dict[Tuple[int, int], Tuple[float, float]] = {
    (2, 0): (-4.84165371736e-4, 0.0),
    (2, 1): (-1.86987635955e-10, 1.19528012031e-9),
    (2, 2): (2.43914352398e-6, -1.40016683654e-6),
    (3, 0): (9.57254173792e-7, 0.0),
    (3, 1): (2.03046201047e-6, 2.48200415856e-7),
    (3, 2): (9.04787894809e-7, -6.19005475177e-7),
    (3, 3): (7.21321757121e-7, 1.41434926192e-6),
    (4, 0): (5.39873863789e-7, 0.0),
    (4, 1): (-5.36157389388e-7, -4.73567346518e-7),
    (4, 2): (3.50501623962e-7, 6.62480026275e-7),
    (4, 3): (9.90856766672e-7, -2.00956723567e-7),
    (4, 4): (-1.88519633023e-7, 3.08803882149e-7),
}

# Piecewise-exponential atmosphere: base height (m), density (kg/m^3), scale height (m)
_ATMOSPHERE = np.array([
    (200e3, 2.789e-10, 37.105e3), (250e3, 7.248e-11, 45.546e3), (300e3, 2.418e-11, 53.628e3),
    (350e3, 9.518e-12, 53.298e3), (400e3, 3.725e-12, 58.515e3), (450e3, 1.585e-12, 60.828e3),
    (500e3, 6.967e-13, 63.822e3), (600e3, 1.454e-13, 71.835e3), (700e3, 3.614e-14, 88.667e3),
    (800e3, 1.170e-14, 124.64e3), (900e3, 5.245e-15, 181.05e3), (1000e3, 3.019e-15, 268.00e3),
])


def density(h_m):
    """Atmospheric density, kg/m^3, at height h (m)."""
    h = np.maximum(np.asarray(h_m, dtype=float), 200e3)
    idx = np.searchsorted(_ATMOSPHERE[:, 0], h, side="right") - 1
    return _ATMOSPHERE[idx, 1] * np.exp((_ATMOSPHERE[idx, 0] - h) / _ATMOSPHERE[idx, 2])


def sun_position(unix_utc: float) -> np.ndarray:
    """Sun, m, mean equator of date (Vallado Algorithm 29, ~0.01 deg)."""
    t = (float(fr.jd(unix_utc)) - fr.JD_J2000) / 36525.0
    m = math.radians(357.5291092 + 35999.05034 * t)
    lam = math.radians(280.460 + 36000.771 * t + 1.914666471 * math.sin(m) + 0.019994643 * math.sin(2 * m))
    dist = (1.000140612 - 0.016708617 * math.cos(m) - 0.000139589 * math.cos(2 * m)) * AU
    eps = math.radians(23.439291 - 0.0130042 * t)
    return dist * np.array([math.cos(lam), math.cos(eps) * math.sin(lam), math.sin(eps) * math.sin(lam)])


def moon_position(unix_utc: float) -> np.ndarray:
    """Moon, m, mean equator of date (Vallado Algorithm 31, ~0.3 deg)."""
    t = (float(fr.jd(unix_utc)) - fr.JD_J2000) / 36525.0
    d = math.radians
    lam = d(218.32 + 481267.8813 * t + 6.29 * math.sin(d(134.9 + 477198.85 * t))
            - 1.27 * math.sin(d(259.2 - 413335.38 * t)) + 0.66 * math.sin(d(235.7 + 890534.23 * t))
            + 0.21 * math.sin(d(269.9 + 954397.70 * t)) - 0.19 * math.sin(d(357.5 + 35999.05 * t))
            - 0.11 * math.sin(d(186.6 + 966404.05 * t)))
    phi = d(5.13 * math.sin(d(93.3 + 483202.03 * t)) + 0.28 * math.sin(d(228.2 + 960400.87 * t))
            - 0.28 * math.sin(d(318.3 + 6003.18 * t)) - 0.17 * math.sin(d(217.6 - 407332.20 * t)))
    par = d(0.9508 + 0.0518 * math.cos(d(134.9 + 477198.85 * t)) + 0.0095 * math.cos(d(259.2 - 413335.38 * t))
            + 0.0078 * math.cos(d(235.7 + 890534.23 * t)) + 0.0028 * math.cos(d(269.9 + 954397.70 * t)))
    eps = d(23.439291 - 0.0130042 * t)
    dist = 6378137.0 / math.sin(par)
    return dist * np.array([
        math.cos(phi) * math.cos(lam),
        math.cos(eps) * math.cos(phi) * math.sin(lam) - math.sin(eps) * math.sin(phi),
        math.sin(eps) * math.cos(phi) * math.sin(lam) + math.cos(eps) * math.sin(phi),
    ])


def _norm_factor(n: int, m: int) -> float:
    """Fully normalized -> unnormalized: C_nm = N_nm * Cbar_nm."""
    return math.sqrt((2 - (m == 0)) * (2 * n + 1) * math.factorial(n - m) / math.factorial(n + m))


@dataclass
class GravityField:
    """Unnormalized coefficients C[n, m], S[n, m] to degree and order N."""

    gm: float
    radius: float
    c: np.ndarray
    s: np.ndarray
    name: str = ""

    @property
    def degree(self) -> int:
        return self.c.shape[0] - 1

    @property
    def zonal_only(self) -> bool:
        return not (np.any(self.c[:, 1:]) or np.any(self.s))

    @classmethod
    def from_normalized(cls, coeffs: Dict[Tuple[int, int], Tuple[float, float]], degree: int,
                        order: Optional[int] = None, gm: float = GM, radius: float = RE, name: str = ""):
        order = degree if order is None else order
        c = np.zeros((degree + 1, degree + 1))
        s = np.zeros((degree + 1, degree + 1))
        c[0, 0] = 1.0
        for (n, m), (cn, sn) in coeffs.items():
            if 2 <= n <= degree and m <= min(n, order):
                f = _norm_factor(n, m)
                c[n, m], s[n, m] = cn * f, sn * f
        return cls(gm, radius, c, s, name or f"{degree} x {order}")

    @classmethod
    def builtin(cls, degree: int = 4, order: Optional[int] = None) -> "GravityField":
        """EGM96 truncated to degree (2 to 4) and order."""
        if not 2 <= degree <= 4:
            raise ValueError("the built-in field goes to degree 4; use an ICGEM file for more")
        order = degree if order is None else order
        return cls.from_normalized(EGM96_4X4, degree, order, name=f"EGM96 {degree} x {order}")

    @classmethod
    def from_icgem(cls, path: Path, degree: int, order: Optional[int] = None) -> "GravityField":
        """Read an ICGEM .gfc file (e.g. EGM2008) to degree and order."""
        gm, radius, normalized = GM, RE, True
        coeffs = {}
        in_head = True
        for line in Path(path).read_text(encoding="ascii", errors="replace").splitlines():
            parts = line.split()
            if not parts:
                continue
            if in_head:
                key = parts[0].lower()
                if key == "earth_gravity_constant":
                    gm = float(parts[1].replace("D", "E"))
                elif key == "radius":
                    radius = float(parts[1].replace("D", "E"))
                elif key == "norm":
                    normalized = parts[1].lower() != "unnormalized"
                elif key == "end_of_head":
                    in_head = False
                continue
            if parts[0] == "gfc":
                n, m = int(parts[1]), int(parts[2])
                if n <= degree:
                    coeffs[(n, m)] = (float(parts[3].replace("D", "E")), float(parts[4].replace("D", "E")))
        if not normalized:
            coeffs = {nm: (c / _norm_factor(*nm), s / _norm_factor(*nm)) for nm, (c, s) in coeffs.items()}
        return cls.from_normalized(coeffs, degree, order, gm, radius, f"{Path(path).name} {degree} x {order or degree}")

    def __post_init__(self):
        n1 = self.degree + 2
        # Recursion weights of V_nm, W_nm (Montenbruck and Gill eq. 3.31), kept
        # as one complex array Z = V + iW
        self._ra = np.zeros((n1, n1))
        self._rb = np.zeros((n1, n1))
        for n in range(n1):
            for m in range(max(n - 1, 0)):
                self._ra[n, m] = (2 * n - 1) / (n - m)
                self._rb[n, m] = (n + m - 1) / (n - m)
        # Acceleration weights (eq. 3.33) with the coefficients folded in
        nd = self.degree + 1
        cs = self.c - 1j * self.s
        self._zonal_c = self.c[:, 0].copy()
        self._t1 = np.zeros((nd, nd), complex)   # times Z[n+1, m+1], m >= 1
        self._t2 = np.zeros((nd, nd), complex)   # times conj(Z[n+1, m-1]), m >= 1
        self._t3 = np.zeros((nd, nd), complex)   # times Z[n+1, m] (z component)
        for n in range(nd):
            for m in range(n + 1):
                self._t3[n, m] = -(n - m + 1) * cs[n, m]
                if m >= 1:
                    f = math.factorial(n - m + 2) / math.factorial(n - m)
                    self._t1[n, m] = -0.5 * cs[n, m]
                    self._t2[n, m] = 0.5 * f * (self.c[n, m] + 1j * self.s[n, m])

    def accel_ecef(self, r: np.ndarray) -> np.ndarray:
        """Acceleration (..., 3) at Earth-fixed positions r (..., 3)."""
        if self.zonal_only:
            return self._zonal(r)
        nd = self.degree + 1
        shape = r.shape[:-1]
        x, y, z = r[..., 0], r[..., 1], r[..., 2]
        r2 = x * x + y * y + z * z
        rr = self.radius / r2
        xy0, z0, rho = (x + 1j * y) * rr, z * rr, self.radius * rr
        zz = np.zeros((nd + 1, nd + 1) + shape, complex)
        zz[0, 0] = self.radius / np.sqrt(r2)
        for n in range(1, nd + 1):
            zz[n, n] = (2 * n - 1) * xy0 * zz[n - 1, n - 1]
            zz[n, n - 1] = (2 * n - 1) * z0 * zz[n - 1, n - 1]
            if n >= 2:
                zz[n, :n - 1] = (self._ra[n, :n - 1, None] * z0 * zz[n - 1, :n - 1]
                                 - self._rb[n, :n - 1, None] * rho * zz[n - 2, :n - 1]) if shape else (
                    self._ra[n, :n - 1] * z0 * zz[n - 1, :n - 1] - self._rb[n, :n - 1] * rho * zz[n - 2, :n - 1])
        flat = zz.reshape(nd + 1, nd + 1, -1)
        up = flat[1:nd + 1]                          # Z[n+1, .] for n = 0..degree
        axy = -np.einsum("n,nk->k", self._zonal_c, up[:, 1])
        axy = axy + np.einsum("nm,nmk->k", self._t1[:, 1:], up[:, 2:nd + 1])
        axy = axy + np.einsum("nm,nmk->k", self._t2[:, 1:], np.conj(up[:, 0:nd - 1]))
        az = np.einsum("nm,nmk->k", self._t3, up[:, :nd]).real
        out = np.empty(axy.shape + (3,))
        g = self.gm / self.radius ** 2
        out[..., 0], out[..., 1], out[..., 2] = g * axy.real, g * axy.imag, g * az
        return out.reshape(shape + (3,))

    def _zonal(self, r: np.ndarray) -> np.ndarray:
        rn = np.sqrt(np.sum(r * r, axis=-1, keepdims=True))
        rhat = r / rn
        s = rhat[..., 2:3]
        a = -self.gm * r / rn ** 3
        zhat = np.array([0.0, 0.0, 1.0])
        for n in range(2, self.degree + 1):
            jn = -self.c[n, 0]
            if jn == 0.0:
                continue
            p, dp = _legendre(n, s)
            a = a + self.gm * jn * self.radius ** n / rn ** (n + 2) * ((n + 1) * p * rhat + dp * (s * rhat - zhat))
        return a


def _legendre(n: int, s):
    if n == 2:
        return (3 * s * s - 1) / 2, 3 * s
    if n == 3:
        return (5 * s ** 3 - 3 * s) / 2, (15 * s * s - 3) / 2
    if n == 4:
        return (35 * s ** 4 - 30 * s * s + 3) / 8, (140 * s ** 3 - 60 * s) / 8
    raise ValueError("closed-form zonals to degree 4")


@dataclass
class ForceModel:
    """Accelerations in TEME. The defaults are the fitting model."""

    gravity: GravityField = field(default_factory=GravityField.builtin)
    drag: bool = True
    bc: float = BC_NOMINAL
    third_body: bool = False
    diurnal_bulge: float = 0.0

    def describe(self) -> str:
        parts = [f"gravity {self.gravity.name}"]
        if self.third_body:
            parts.append("Sun and Moon")
        if self.drag:
            parts.append(f"drag (Cd A/m {self.bc:.4f} m^2/kg times an estimated scale factor)")
        return ", ".join(parts)

    def accel(self, t: float, r: np.ndarray, v: np.ndarray, k: np.ndarray) -> np.ndarray:
        if self.gravity.zonal_only:
            a = self.gravity.accel_ecef(r)  # zonal: the same in TEME
        else:
            theta = float(fr.gmst(t))
            c, s = math.cos(theta), math.sin(theta)
            re = np.empty_like(r)
            re[..., 0] = c * r[..., 0] + s * r[..., 1]
            re[..., 1] = c * r[..., 1] - s * r[..., 0]
            re[..., 2] = r[..., 2]
            ae = self.gravity.accel_ecef(re)
            a = np.empty_like(ae)
            a[..., 0] = c * ae[..., 0] - s * ae[..., 1]
            a[..., 1] = s * ae[..., 0] + c * ae[..., 1]
            a[..., 2] = ae[..., 2]
        if self.third_body:
            for mu, body in ((GM_SUN, sun_position(t)), (GM_MOON, moon_position(t))):
                rel = body - r
                a = a + mu * (rel / np.linalg.norm(rel, axis=-1, keepdims=True) ** 3 - body / np.linalg.norm(body) ** 3)
        if self.drag:
            rn = np.sqrt(np.sum(r * r, axis=-1))
            s = r[..., 2] / rn
            rho = density(rn - 6378137.0 * (1 - FLATTENING * s * s))
            if self.diurnal_bulge:
                sun = sun_position(t)
                lag = math.radians(30.0)
                c, si = math.cos(lag), math.sin(lag)
                bulge = np.array([c * sun[0] - si * sun[1], si * sun[0] + c * sun[1], sun[2]])
                bulge /= np.linalg.norm(bulge)
                rho = rho * (1 + self.diurnal_bulge * (r @ bulge) / rn)
            om = fr.OMEGA_EARTH
            vrel = v.copy()
            vrel[..., 0] += om * r[..., 1]
            vrel[..., 1] -= om * r[..., 0]
            speed = np.sqrt(np.sum(vrel * vrel, axis=-1))
            a = a - (0.5 * k * self.bc * rho * speed)[..., None] * vrel
        return a


def _deriv(model: ForceModel, t: float, y: np.ndarray, k: np.ndarray) -> np.ndarray:
    return np.concatenate([y[..., 3:], model.accel(t, y[..., :3], y[..., 3:], k)], axis=-1)


# Dormand-Prince 5(4) tableau, 5th-order weights (the multistep start)
_DP_A = ((), (1 / 5,), (3 / 40, 9 / 40), (44 / 45, -56 / 15, 32 / 9),
         (19372 / 6561, -25360 / 2187, 64448 / 6561, -212 / 729),
         (9017 / 3168, -355 / 33, 46732 / 5247, 49 / 176, -5103 / 18656))
_DP_C = (0.0, 1 / 5, 3 / 10, 4 / 5, 8 / 9, 1.0)
_DP_B = (35 / 384, 0.0, 500 / 1113, 125 / 192, -2187 / 6784, 11 / 84)


def _dp5(model, t, y, k, h, substeps):
    h = h / substeps
    for _ in range(substeps):
        ks = []
        for i in range(6):
            yi = y
            for j, a in enumerate(_DP_A[i]):
                yi = yi + (h * a) * ks[j]
            ks.append(_deriv(model, t + _DP_C[i] * h, yi, k))
        y = y + h * sum(b * kk for b, kk in zip(_DP_B, ks) if b)
        t += h
    return y


def _adams_coefficients(order: int) -> Tuple[np.ndarray, np.ndarray]:
    """Adams-Bashforth (order) and Adams-Moulton (order + 1) weights, newest first."""
    def weights(nodes):
        out = []
        for j, nj in enumerate(nodes):
            poly = np.poly1d([1.0])
            for i, ni in enumerate(nodes):
                if i != j:
                    poly = poly * np.poly1d([1.0, -ni]) / (nj - ni)
            integral = poly.integ()
            out.append(integral(1.0) - integral(0.0))
        return np.array(out)
    return weights([-float(j) for j in range(order)]), weights([1.0 - j for j in range(order + 1)])


ORDER = 10
STEP_S = 60.0
_AB, _AM = _adams_coefficients(ORDER)
_INTERP_POINTS = 10


def _integrate_grid(model, t0, y0, k, n_steps, h):
    """States on t0 + j h, j = 0..n_steps (n_steps + 1, N, 6)."""
    ys = [y0]
    fs = [_deriv(model, t0, y0, k)]
    for j in range(min(n_steps, ORDER - 1)):
        y = _dp5(model, t0 + j * h, ys[-1], k, h, 6)
        ys.append(y)
        fs.append(_deriv(model, t0 + (j + 1) * h, y, k))
    for j in range(ORDER - 1, n_steps):
        tn = t0 + (j + 1) * h
        back = fs[-1:-ORDER - 1:-1]
        pred = ys[-1] + h * sum(c * f for c, f in zip(_AB, back))
        fp = _deriv(model, tn, pred, k)
        corr = ys[-1] + h * (_AM[0] * fp + sum(c * f for c, f in zip(_AM[1:], back)))
        ys.append(corr)
        fs.append(_deriv(model, tn, corr, k))
        if len(fs) > ORDER + 1:
            fs.pop(0)
    return np.array(ys)


def _lagrange(grid_t: np.ndarray, grid_y: np.ndarray, times: np.ndarray) -> np.ndarray:
    """Interpolate grid_y (G, N, 6) at times, degree _INTERP_POINTS - 1, centred stencils."""
    h = grid_t[1] - grid_t[0]
    x = (np.asarray(times, dtype=float) - grid_t[0]) / h
    j0 = np.clip(np.floor(x).astype(int) - _INTERP_POINTS // 2 + 1, 0, len(grid_t) - _INTERP_POINTS)
    offsets = np.arange(_INTERP_POINTS)
    u = x - j0  # position inside the stencil, in steps
    weights = np.ones((len(x), _INTERP_POINTS))
    for a in offsets:
        for b in offsets:
            if a != b:
                weights[:, a] *= (u - b) / (a - b)
    stencil = grid_y[j0[:, None] + offsets[None, :]]  # (T, P, N, 6)
    return np.moveaxis(np.einsum("tp,tpnk->tnk", weights, stencil), 0, 1)


def propagate(model: ForceModel, t0: float, y0: np.ndarray, k, times: Sequence[float],
              step: float = STEP_S) -> np.ndarray:
    """States (N, len(times), 6) of N trajectories from TEME states y0 (N, 6) at t0."""
    times = np.asarray(times, dtype=float)
    y0 = np.atleast_2d(np.asarray(y0, dtype=float))
    k = np.broadcast_to(np.asarray(k, dtype=float), (y0.shape[0],)).copy()
    margin = (_INTERP_POINTS // 2 + 1) * step
    n_fwd = max(int(math.ceil((max(times.max(), t0) - t0 + margin) / step)), ORDER)
    n_bwd = int(math.ceil((t0 - times.min() + margin) / step)) if times.min() < t0 else 0
    fwd = _integrate_grid(model, t0, y0, k, n_fwd, step)
    if n_bwd:
        bwd = _integrate_grid(model, t0, y0, k, max(n_bwd, ORDER), -step)[:0:-1]
        grid_y = np.concatenate([bwd, fwd])
        grid_t = t0 + step * np.arange(-len(bwd), n_fwd + 1)
    else:
        grid_y, grid_t = fwd, t0 + step * np.arange(n_fwd + 1)
    return _lagrange(grid_t, grid_y, times)
