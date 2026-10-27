#!/usr/bin/env python3
"""
Time scales and reference frames for the SMART-QSO orbit determination
(SQSO-GND-002 §4.2).

Times are POSIX seconds (UTC). Frames:

- ITRF: Earth-fixed (the GNSS receiver's WGS-84 ECEF, which agrees with the
  ITRF to centimetres). PEF is ITRF without polar motion.
- TEME: true equator, mean equinox of date, r_PEF = R3(GMST) r_TEME
  (Vallado, Fundamentals of Astrodynamics, 4th ed., §3.7). It is the frame
  of SGP4 and two-line elements, and the quasi-inertial frame the orbit is
  integrated in: over a few days its pole moves by milliarcseconds.
- EME2000 (J2000 mean equator and equinox): TEME -> TOD -> MOD -> J2000 with
  IAU-1976 precession and the IAU-1980 nutation, truncated to its 18 largest
  terms (omitted terms < 0.0005" each, ~2 cm at the orbit; total truncation
  error well under 1 m).

Earth orientation: UT1 = UTC + DUT1 (IERS Bulletin A). The fit itself does
not depend on DUT1: the fixes go to TEME and the products back to ITRF with
the same rotation, and the force model is invariant to a rotation about the
pole. DUT1 matters only for EME2000 and TEME products (up to 0.9 s, ~450 m
at the orbit), so the EME2000 product requires it. Polar motion (xp, yp,
< 0.6", < 20 m) is optional.
"""

from __future__ import annotations

import math
from typing import Tuple

import numpy as np

#: Earth rotation rate, rad/s (Vallado; IERS)
OMEGA_EARTH = 7.292115146706979e-5
ARCSEC = math.pi / (180.0 * 3600.0)
JD_UNIX_EPOCH = 2440587.5
JD_J2000 = 2451545.0
#: TT - TAI, s
TT_TAI_S = 32.184


def jd(unix_s):
    """Julian date (same time scale as the input)."""
    return np.asarray(unix_s, dtype=float) / 86400.0 + JD_UNIX_EPOCH


def gmst(unix_utc, dut1_s: float = 0.0):
    """Greenwich mean sidereal time (IAU-1982), rad."""
    d = jd(unix_utc) + dut1_s / 86400.0 - JD_J2000
    t = d / 36525.0
    deg = 280.46061837 + 360.98564736629 * d + 0.000387933 * t * t - t ** 3 / 38710000.0
    return np.radians(np.mod(deg, 360.0))


def rot3(angle):
    """Frame rotation about z (Vallado ROT3), stacked for an array of angles."""
    c, s = np.cos(angle), np.sin(angle)
    zero, one = np.zeros_like(c), np.ones_like(c)
    return np.stack([np.stack([c, s, zero], -1), np.stack([-s, c, zero], -1),
                     np.stack([zero, zero, one], -1)], -2)


def rot2(angle):
    c, s = np.cos(angle), np.sin(angle)
    zero, one = np.zeros_like(c), np.ones_like(c)
    return np.stack([np.stack([c, zero, -s], -1), np.stack([zero, one, zero], -1),
                     np.stack([s, zero, c], -1)], -2)


def rot1(angle):
    c, s = np.cos(angle), np.sin(angle)
    zero, one = np.zeros_like(c), np.ones_like(c)
    return np.stack([np.stack([one, zero, zero], -1), np.stack([zero, c, s], -1),
                     np.stack([zero, -s, c], -1)], -2)


def polar_motion(xp_arcsec: float = 0.0, yp_arcsec: float = 0.0) -> np.ndarray:
    """W with r_PEF = W r_ITRF (IAU-1980 convention, Vallado eq. 3-78)."""
    xp, yp = xp_arcsec * ARCSEC, yp_arcsec * ARCSEC
    return rot1(yp) @ rot2(xp)


def teme_to_itrf_matrix(unix_utc, dut1_s=0.0, xp_arcsec=0.0, yp_arcsec=0.0):
    """M with r_ITRF = M r_TEME (an array of matrices for an array of times)."""
    w = polar_motion(xp_arcsec, yp_arcsec)
    return np.swapaxes(w, -1, -2) @ rot3(gmst(unix_utc, dut1_s))


def itrf_to_teme(unix_utc, r_itrf, v_itrf=None, dut1_s=0.0, xp_arcsec=0.0, yp_arcsec=0.0):
    """ITRF position (and velocity) to TEME; arrays of shape (..., 3)."""
    m = teme_to_itrf_matrix(unix_utc, dut1_s, xp_arcsec, yp_arcsec)
    mt = np.swapaxes(m, -1, -2)
    r_itrf = np.asarray(r_itrf, dtype=float)
    r = np.einsum("...ij,...j->...i", mt, r_itrf)
    if v_itrf is None:
        return r
    w = polar_motion(xp_arcsec, yp_arcsec)
    r_pef = np.einsum("ij,...j->...i", w, r_itrf)
    v_pef = np.einsum("ij,...j->...i", w, np.asarray(v_itrf, dtype=float))
    v_pef = v_pef + np.cross([0.0, 0.0, OMEGA_EARTH], r_pef)
    r3t = np.swapaxes(rot3(gmst(unix_utc, dut1_s)), -1, -2)
    return r, np.einsum("...ij,...j->...i", r3t, v_pef)


def teme_to_itrf(unix_utc, r_teme, v_teme=None, dut1_s=0.0, xp_arcsec=0.0, yp_arcsec=0.0):
    """TEME position (and velocity) to ITRF; arrays of shape (..., 3)."""
    r3 = rot3(gmst(unix_utc, dut1_s))
    wt = np.swapaxes(polar_motion(xp_arcsec, yp_arcsec), -1, -2)
    r_pef = np.einsum("...ij,...j->...i", r3, np.asarray(r_teme, dtype=float))
    r = np.einsum("ij,...j->...i", wt, r_pef)
    if v_teme is None:
        return r
    v_pef = np.einsum("...ij,...j->...i", r3, np.asarray(v_teme, dtype=float))
    v_pef = v_pef - np.cross([0.0, 0.0, OMEGA_EARTH], r_pef)
    return r, np.einsum("ij,...j->...i", wt, v_pef)


def teme_to_itrf_jacobian(unix_utc, dut1_s=0.0, xp_arcsec=0.0, yp_arcsec=0.0):
    """6x6 d(ITRF state)/d(TEME state), for covariance transformation."""
    r3 = rot3(gmst(unix_utc, dut1_s))
    wt = np.swapaxes(polar_motion(xp_arcsec, yp_arcsec), -1, -2)
    omega = np.array([[0.0, -OMEGA_EARTH, 0.0], [OMEGA_EARTH, 0.0, 0.0], [0.0, 0.0, 0.0]])
    j = np.zeros(r3.shape[:-2] + (6, 6))
    j[..., :3, :3] = wt @ r3
    j[..., 3:, 3:] = wt @ r3
    j[..., 3:, :3] = -(wt @ omega @ r3)
    return j


def teme_utc_to_ut1(unix_utc, dut1_s: float) -> np.ndarray:
    """M with r_TEME = M r, r in the TEME the fit uses (GMST from UTC).

    SGP4 and EME2000 need GMST from UT1: the two differ by a rotation about
    the pole through the Earth's turn in DUT1.
    """
    return rot3(gmst(unix_utc, 0.0) - gmst(unix_utc, dut1_s))


# --- EME2000 ----------------------------------------------------------------

# IAU-1980 nutation, largest 18 terms (Meeus, Astronomical Algorithms, Table 22.A):
# multiples of D, M, M', F, Omega; dpsi sin coefficient (0.0001", + T term);
# deps cos coefficient (0.0001", + T term)
_NUTATION = (
    (0, 0, 0, 0, 1, -171996.0, -174.2, 92025.0, 8.9),
    (-2, 0, 0, 2, 2, -13187.0, -1.6, 5736.0, -3.1),
    (0, 0, 0, 2, 2, -2274.0, -0.2, 977.0, -0.5),
    (0, 0, 0, 0, 2, 2062.0, 0.2, -895.0, 0.5),
    (0, 1, 0, 0, 0, 1426.0, -3.4, 54.0, -0.1),
    (0, 0, 1, 0, 0, 712.0, 0.1, -7.0, 0.0),
    (-2, 1, 0, 2, 2, -517.0, 1.2, 224.0, -0.6),
    (0, 0, 0, 2, 1, -386.0, -0.4, 200.0, 0.0),
    (0, 0, 1, 2, 2, -301.0, 0.0, 129.0, -0.1),
    (-2, -1, 0, 2, 2, 217.0, -0.5, -95.0, 0.3),
    (-2, 0, 1, 0, 0, -158.0, 0.0, 0.0, 0.0),
    (-2, 0, 0, 2, 1, 129.0, 0.1, -70.0, 0.0),
    (0, 0, -1, 2, 2, 123.0, 0.0, -53.0, 0.0),
    (2, 0, 0, 0, 0, 63.0, 0.0, 0.0, 0.0),
    (0, 0, 1, 0, 1, 63.0, 0.1, -33.0, 0.0),
    (2, 0, -1, 2, 2, -59.0, 0.0, 26.0, 0.0),
    (0, 0, -1, 0, 1, -58.0, -0.1, 32.0, 0.0),
    (0, 0, 1, 2, 1, -51.0, 0.0, 27.0, 0.0),
)


def nutation(t_tt: float) -> Tuple[float, float, float, float]:
    """(dpsi, deps, mean obliquity, Omega), rad, for Julian centuries TT from J2000."""
    t = t_tt
    d = math.radians(297.85036 + 445267.111480 * t - 0.0019142 * t * t + t ** 3 / 189474.0)
    m = math.radians(357.52772 + 35999.050340 * t - 0.0001603 * t * t - t ** 3 / 300000.0)
    mp = math.radians(134.96298 + 477198.867398 * t + 0.0086972 * t * t + t ** 3 / 56250.0)
    f = math.radians(93.27191 + 483202.017538 * t - 0.0036825 * t * t + t ** 3 / 327270.0)
    om = math.radians(125.04452 - 1934.136261 * t + 0.0020708 * t * t + t ** 3 / 450000.0)
    dpsi = deps = 0.0
    for kd, km, kmp, kf, kom, s0, s1, c0, c1 in _NUTATION:
        arg = kd * d + km * m + kmp * mp + kf * f + kom * om
        dpsi += (s0 + s1 * t) * math.sin(arg)
        deps += (c0 + c1 * t) * math.cos(arg)
    eps0 = (84381.448 - 46.8150 * t - 0.00059 * t * t + 0.001813 * t ** 3) * ARCSEC
    return dpsi * 1e-4 * ARCSEC, deps * 1e-4 * ARCSEC, eps0, om


def teme_to_eme2000_matrix(unix_utc: float, tt_utc_s: float) -> np.ndarray:
    """M with r_EME2000 = M r_TEME at one time (IAU-1976/1980)."""
    t = (float(jd(unix_utc)) + tt_utc_s / 86400.0 - JD_J2000) / 36525.0
    dpsi, deps, eps0, om = nutation(t)
    eqeq = dpsi * math.cos(eps0) + (0.00264 * math.sin(om) + 0.000063 * math.sin(2 * om)) * ARCSEC
    zeta = (2306.2181 * t + 0.30188 * t * t + 0.017998 * t ** 3) * ARCSEC
    theta = (2004.3109 * t - 0.42665 * t * t - 0.041833 * t ** 3) * ARCSEC
    z = (2306.2181 * t + 1.09468 * t * t + 0.018203 * t ** 3) * ARCSEC
    prec = rot3(-z) @ rot2(theta) @ rot3(-zeta)          # r_MOD = P r_J2000
    nut = rot1(-(eps0 + deps)) @ rot3(-dpsi) @ rot1(eps0)  # r_TOD = N r_MOD
    tod_from_teme = rot3(-eqeq)                           # r_TOD = R3(-eqeq) r_TEME
    return prec.T @ nut.T @ tod_from_teme


def tt_minus_utc(tai_utc_s: float) -> float:
    """TT - UTC, s."""
    return tai_utc_s + TT_TAI_S
