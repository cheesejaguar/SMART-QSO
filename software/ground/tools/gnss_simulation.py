#!/usr/bin/env python3
"""
Simulated GNSS fixes for the orbit-determination tests and rehearsals
(SQSO-GND-002 §6; VI-LV-13, VI-GND-06).

A truth orbit (475 km sun-synchronous, LTAN 22:30, BASELINE §3) is integrated
with a richer force model than the fit uses (C22/S22, Sun and Moon, a diurnal
density bulge, a drag scale that is not the nominal), and the receiver's fixes
are made from it with noise, written as GNSS state frames in the layout of
gnss_frame.py.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import List, Optional, Sequence

import numpy as np

import orbit_frames as fr
from gnss_frame import GnssFix, format_gnss_frame, unix_to_gps
from orbit_dynamics import EGM96_4X4, GM, RE, ForceModel, GravityField, propagate, sun_position

J2 = -EGM96_4X4[(2, 0)][0] * 5 ** 0.5


def kaula_field(degree: int = 20, seed: int = 2028) -> GravityField:
    """EGM96 to 4 x 4, then random coefficients of degree 5 to degree with
    Kaula's rule (rms 1e-5 / n^2, normalized): a stand-in for the part of the
    real field the fit leaves out."""
    rng = np.random.default_rng(seed)
    coeffs = dict(EGM96_4X4)
    for n in range(5, degree + 1):
        for m in range(n + 1):
            sigma = 1e-5 / n ** 2
            coeffs[(n, m)] = (rng.normal(0, sigma), 0.0 if m == 0 else rng.normal(0, sigma))
    return GravityField.from_normalized(coeffs, degree, name=f"EGM96 4 x 4 + Kaula {degree} x {degree}")


#: Truth model of the simulations: what the fit does not model (degree 5-8
#: gravity, Sun and Moon, the diurnal bulge) and a drag scale it must find
TRUTH_MODEL = ForceModel(gravity=kaula_field(), drag=True, third_body=True, diurnal_bulge=0.3)
TRUTH_DRAG_SCALE = 1.4


def sso_inclination(a_m: float, e: float = 0.0) -> float:
    """Sun-synchronous inclination, rad (J2 nodal rate = 360 deg per year)."""
    n = math.sqrt(GM / a_m ** 3)
    rate = 2 * math.pi / (365.2422 * 86400)
    return math.acos(-rate / (1.5 * n * J2 * (RE / (a_m * (1 - e * e))) ** 2))


def elements_to_state(a, e, i, raan, argp, nu):
    """Position and velocity (m, m/s) from classical elements (rad)."""
    p = a * (1 - e * e)
    r_pf = p / (1 + e * math.cos(nu)) * np.array([math.cos(nu), math.sin(nu), 0.0])
    v_pf = math.sqrt(GM / p) * np.array([-math.sin(nu), e + math.cos(nu), 0.0])
    rot = (fr.rot3(-raan) @ fr.rot1(-i) @ fr.rot3(-argp))
    return np.concatenate([rot @ r_pf, rot @ v_pf])


def sso_state(epoch_unix: float, altitude_m: float = 475e3, ltan_h: float = 22.5,
              e: float = 0.0012, argp_deg: float = 90.0, u_deg: float = 30.0) -> np.ndarray:
    """TEME state of the baseline orbit at an epoch."""
    a = RE + altitude_m
    sun = sun_position(epoch_unix)
    raan = math.atan2(sun[1], sun[0]) + math.radians((ltan_h - 12.0) * 15.0)
    argp = math.radians(argp_deg)
    return elements_to_state(a, e, sso_inclination(a, e), raan, argp, math.radians(u_deg) - argp)


@dataclass
class Truth:
    """A truth trajectory to sample."""

    epoch: float
    state: np.ndarray
    drag_scale: float = TRUTH_DRAG_SCALE
    model: ForceModel = field(default_factory=lambda: TRUTH_MODEL)

    def states(self, times: Sequence[float]) -> np.ndarray:
        """TEME states (n, 6) at the times."""
        return propagate(self.model, self.epoch, self.state[None, :], np.array([self.drag_scale]), times)[0]


def simulate_fixes(truth: Truth, times: Sequence[float], sigma_m_per_pdop: float = 2.5,
                   sigma_v: Optional[float] = 0.5, pdop: float = 2.0, rng=None,
                   sats: int = 7) -> List[GnssFix]:
    """Noisy receiver fixes (ITRF) at the times; velocity noise sigma_v (None: no velocity)."""
    rng = rng or np.random.default_rng(0)
    times = np.asarray(times, dtype=float)
    st = truth.states(times)
    r, v = fr.teme_to_itrf(times, st[:, :3], st[:, 3:])
    out = []
    for n, t in enumerate(times):
        week, tow = unix_to_gps(float(t))
        rr = r[n] + rng.normal(0.0, sigma_m_per_pdop * pdop, 3)
        vv = None if sigma_v is None else tuple(v[n] + rng.normal(0.0, sigma_v, 3))
        out.append(GnssFix(week=week % 1024, tow_ms=round(tow * 1000.0), r_ecef_m=tuple(rr), v_ecef_ms=vv,
                           quality=1, sats=sats, pdop=pdop, seq=n))
    return out


def frames(fixes: Sequence[GnssFix]) -> List[str]:
    """The fixes as downlinked information fields."""
    return [format_gnss_frame(f) for f in fixes]
