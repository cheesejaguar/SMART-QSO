#!/usr/bin/env python3
"""
Ephemeris products of the SMART-QSO orbit determination (SQSO-GND-002 §5):

- CCSDS Orbit Ephemeris Message, KVN (CCSDS 502.0-B-3, version 3.0): UTC,
  ITRF or EME2000, a state every step with a 6 x 6 covariance block per
  record, as the launch provider asks for (forward-predicted, with
  covariance; SpaceX Rideshare UG v11 §9.5.7.2);
- two-line elements fitted to the predicted trajectory with SGP4 (the
  reference implementation, python-sgp4), for the integrator, the 18 SDS
  and the amateur users' tracking programs.

read_oem() reads back what write_oem() writes (used by the tests and to
check a file before upload).
"""

from __future__ import annotations

import math
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

import numpy as np

import orbit_frames as fr
from orbit_determination import Prediction

FRAMES = ("ITRF", "EME2000")
#: The REF_FRAME keyword for the Earth-fixed product: the receiver's WGS-84
#: (G2139) agrees with ITRF2000 and its successors to a few centimetres
ITRF_NAME = "ITRF2000"


def iso(unix_s: float) -> str:
    """CCSDS ASCII time, UTC, milliseconds."""
    stamp = datetime.fromtimestamp(round(unix_s * 1000) / 1000.0, tz=timezone.utc)
    return stamp.strftime("%Y-%m-%dT%H:%M:%S.") + f"{stamp.microsecond // 1000:03d}"


def parse_iso(text: str) -> float:
    stamp = datetime.fromisoformat(text.strip().replace("Z", ""))
    return stamp.replace(tzinfo=timezone.utc).timestamp()


def to_frame(pred: Prediction, frame: str, dut1_s: Optional[float], tai_utc_s: float,
             xp_arcsec: float = 0.0, yp_arcsec: float = 0.0) -> Tuple[np.ndarray, np.ndarray]:
    """States (n, 6) and covariances (n, 6, 6) of a prediction in the output frame."""
    if frame == "ITRF":
        # The fit does not depend on DUT1 (orbit_frames docstring): any value
        # gives the same Earth-fixed product, so 0 is used when none is given
        d = dut1_s or 0.0
        r, v = fr.teme_to_itrf(pred.times, pred.states[:, :3], pred.states[:, 3:], d, xp_arcsec, yp_arcsec)
        jac = fr.teme_to_itrf_jacobian(pred.times, d, xp_arcsec, yp_arcsec)
        return np.concatenate([r, v], axis=1), jac @ pred.covariances @ np.swapaxes(jac, -1, -2)
    if frame == "EME2000":
        if dut1_s is None:
            raise ValueError("EME2000 needs DUT1 (UT1 - UTC, IERS Bulletin A): up to 450 m of error without it")
        # The TEME of the fit is tied to GMST from UTC; re-reference it to UT1
        # (orbit_frames.teme_utc_to_ut1), then rotate to J2000
        tt_utc = fr.tt_minus_utc(tai_utc_s)
        states = np.empty_like(pred.states)
        covs = np.empty_like(pred.covariances)
        for i, t in enumerate(pred.times):
            m = fr.teme_to_eme2000_matrix(float(t), tt_utc) @ fr.teme_utc_to_ut1(float(t), dut1_s)
            block = np.zeros((6, 6))
            block[:3, :3] = block[3:, 3:] = m
            states[i] = block @ pred.states[i]
            covs[i] = block @ pred.covariances[i] @ block.T
        return states, covs
    raise ValueError(f"frame must be one of {FRAMES}")


@dataclass
class OemMeta:
    """Header and metadata of an OEM."""

    object_name: str = "SMART-QSO"
    object_id: str = "UNKNOWN"
    originator: str = "KK6GDB"
    frame: str = "ITRF"
    comments: Sequence[str] = ()
    creation_unix: Optional[float] = None


def write_oem(path: Path, pred: Prediction, meta: OemMeta, dut1_s: Optional[float] = None,
              tai_utc_s: float = 37.0, cov_every: int = 1, xp_arcsec: float = 0.0, yp_arcsec: float = 0.0) -> Path:
    """Write a KVN OEM: states every grid point, a covariance block every cov_every points."""
    states, covs = to_frame(pred, meta.frame, dut1_s, tai_utc_s, xp_arcsec, yp_arcsec)
    frame_name = ITRF_NAME if meta.frame == "ITRF" else "EME2000"
    created = meta.creation_unix if meta.creation_unix is not None else datetime.now(timezone.utc).timestamp()
    lines = [
        "CCSDS_OEM_VERS = 3.0",
        *[f"COMMENT {c}" for c in meta.comments],
        f"CREATION_DATE = {iso(created)}",
        f"ORIGINATOR = {meta.originator}",
        f"MESSAGE_ID = {meta.object_name}-{iso(pred.times[0]).replace(':', '').replace('-', '')[:15]}-{frame_name}",
        "",
        "META_START",
        f"OBJECT_NAME = {meta.object_name}",
        f"OBJECT_ID = {meta.object_id}",
        "CENTER_NAME = EARTH",
        f"REF_FRAME = {frame_name}",
        "TIME_SYSTEM = UTC",
        f"START_TIME = {iso(pred.times[0])}",
        f"USEABLE_START_TIME = {iso(pred.times[0])}",
        f"USEABLE_STOP_TIME = {iso(pred.times[-1])}",
        f"STOP_TIME = {iso(pred.times[-1])}",
        "INTERPOLATION = LAGRANGE",
        "INTERPOLATION_DEGREE = 7",
        "META_STOP",
        "",
    ]
    for t, s in zip(pred.times, states):
        km = s / 1000.0
        lines.append(f"{iso(t)} {km[0]:.6f} {km[1]:.6f} {km[2]:.6f} {km[3]:.9f} {km[4]:.9f} {km[5]:.9f}")
    lines += ["", "COVARIANCE_START"]
    for i in range(0, len(pred.times), max(cov_every, 1)):
        c = covs[i] / 1e6  # m^2 -> km^2 (and m^2/s, m^2/s^2 alike)
        lines += [f"EPOCH = {iso(pred.times[i])}", f"COV_REF_FRAME = {frame_name}"]
        for row in range(6):
            lines.append(" ".join(f"{c[row, col]: .10e}" for col in range(row + 1)))
    lines += ["COVARIANCE_STOP", ""]
    path = Path(path)
    path.write_text("\n".join(lines), encoding="ascii")
    return path


@dataclass
class Oem:
    """An OEM read back: header keywords, states (km, km/s) and covariances (km^2...)."""

    header: Dict[str, str]
    meta: Dict[str, str]
    times: np.ndarray
    states: np.ndarray
    cov_times: np.ndarray
    covariances: np.ndarray


def read_oem(path: Path) -> Oem:
    """Parse a single-segment KVN OEM, checking the keywords this tool relies on."""
    header, meta = {}, {}
    times, states, cov_times, covs = [], [], [], []
    section = "header"
    rows: List[List[float]] = []
    for raw in Path(path).read_text(encoding="ascii").splitlines():
        line = raw.strip()
        if not line or line.startswith("COMMENT"):
            continue
        if line == "META_START":
            section = "meta"
            continue
        if line == "META_STOP":
            section = "data"
            continue
        if line == "COVARIANCE_START":
            section = "cov"
            continue
        if line == "COVARIANCE_STOP":
            section = "end"
            continue
        if section in ("header", "meta"):
            key, sep, value = line.partition("=")
            if not sep:
                raise ValueError(f"not a KVN line: {line!r}")
            (header if section == "header" else meta)[key.strip()] = value.strip()
        elif section == "data":
            parts = line.split()
            if len(parts) != 7:
                raise ValueError(f"ephemeris line needs epoch and 6 values: {line!r}")
            times.append(parse_iso(parts[0]))
            states.append([float(x) for x in parts[1:]])
        elif section == "cov":
            if line.startswith("EPOCH"):
                cov_times.append(parse_iso(line.partition("=")[2]))
                rows = []
            elif line.startswith("COV_REF_FRAME"):
                continue
            else:
                rows.append([float(x) for x in line.split()])
                if len(rows) == 6:
                    m = np.zeros((6, 6))
                    for i, row in enumerate(rows):
                        if len(row) != i + 1:
                            raise ValueError("covariance rows must be lower triangular")
                        m[i, : i + 1] = row
                    covs.append(m + np.tril(m, -1).T)
    for key in ("CCSDS_OEM_VERS", "CREATION_DATE", "ORIGINATOR"):
        if key not in header:
            raise ValueError(f"header lacks {key}")
    for key in ("OBJECT_NAME", "OBJECT_ID", "CENTER_NAME", "REF_FRAME", "TIME_SYSTEM", "START_TIME", "STOP_TIME"):
        if key not in meta:
            raise ValueError(f"metadata lacks {key}")
    return Oem(header, meta, np.array(times), np.array(states), np.array(cov_times), np.array(covs))


# --- Two-line elements --------------------------------------------------------

#: Earth radius and gravity of SGP4 (WGS-72), for the initial mean motion
_WGS72_MU_KM = 398600.8
MINUTES_PER_DAY = 1440.0


def _checksum(line: str) -> int:
    return sum(int(c) if c.isdigit() else (1 if c == "-" else 0) for c in line[:68]) % 10


def _exp_field(value: float) -> str:
    """TLE assumed-decimal exponent field, 8 characters (e.g. ' 12345-4')."""
    if value == 0.0:
        return " 00000+0"
    exponent = math.floor(math.log10(abs(value))) + 1
    mantissa = round(abs(value) / 10.0 ** exponent * 1e5)
    if mantissa >= 100000:
        mantissa //= 10
        exponent += 1
    return f"{'-' if value < 0 else ' '}{mantissa:05d}{'-' if exponent < 0 else '+'}{abs(exponent) % 10}"


def format_tle(satnum: int, intl_designator: str, epoch_unix: float, ndot2: float, bstar: float,
               inc_deg: float, raan_deg: float, ecc: float, argp_deg: float, ma_deg: float,
               n_rev_day: float, elnum: int = 1, revnum: int = 0, classification: str = "U") -> Tuple[str, str]:
    """The two lines, with checksums."""
    stamp = datetime.fromtimestamp(epoch_unix, tz=timezone.utc)
    start = datetime(stamp.year, 1, 1, tzinfo=timezone.utc)
    doy = (stamp - start).total_seconds() / 86400.0 + 1.0
    nd = f"{ndot2:.8f}".replace("0.", ".", 1)
    nd = nd if nd.startswith("-") else " " + nd
    line1 = (f"1 {satnum:05d}{classification} {intl_designator:<8.8} {stamp.year % 100:02d}{doy:012.8f} "
             f"{nd} {_exp_field(0.0)} {_exp_field(bstar)} 0 {elnum % 10000:4d}")
    line2 = (f"2 {satnum:05d} {inc_deg % 360:8.4f} {raan_deg % 360:8.4f} {round(ecc * 1e7):07d} "
             f"{argp_deg % 360:8.4f} {ma_deg % 360:8.4f} {n_rev_day:11.8f}{revnum % 100000:5d}")
    return line1 + str(_checksum(line1)), line2 + str(_checksum(line2))


def _rv_to_elements(r: np.ndarray, v: np.ndarray, mu: float):
    """Classical elements (a, e, i, raan, argp, mean anomaly) from a state (km, km/s)."""
    h = np.cross(r, v)
    nvec = np.cross([0.0, 0.0, 1.0], h)
    rn, vn = np.linalg.norm(r), np.linalg.norm(v)
    evec = ((vn * vn - mu / rn) * r - np.dot(r, v) * v) / mu
    e = np.linalg.norm(evec)
    a = 1.0 / (2.0 / rn - vn * vn / mu)
    i = math.acos(h[2] / np.linalg.norm(h))
    raan = math.atan2(nvec[1], nvec[0])
    argp = math.atan2(np.dot(np.cross(nvec, evec), h) / np.linalg.norm(h), np.dot(nvec, evec))
    nu = math.atan2(np.dot(np.cross(evec, r), h) / np.linalg.norm(h), np.dot(evec, r))
    ea = 2 * math.atan(math.sqrt((1 - e) / (1 + e)) * math.tan(nu / 2))
    return a, e, i, raan, argp, ea - e * math.sin(ea)


@dataclass
class TleFit:
    """Two-line elements fitted to a trajectory."""

    line1: str
    line2: str
    rms_m: float
    max_m: float
    epoch: float


def fit_tle(times: np.ndarray, r_teme_m: np.ndarray, v_teme_ms: np.ndarray, epoch_unix: float,
            satnum: int = 99999, intl_designator: str = "", iterations: int = 15) -> TleFit:
    """SGP4 mean elements (and B*) fitted by least squares to TEME positions."""
    from sgp4.api import WGS72, Satrec, jday

    times = np.asarray(times, dtype=float)
    r_km = np.asarray(r_teme_m) / 1000.0
    i0 = int(np.argmin(np.abs(times - epoch_unix)))
    a, e, inc, raan, argp, ma = _rv_to_elements(r_km[i0], np.asarray(v_teme_ms[i0]) / 1000.0, _WGS72_MU_KM)
    n = math.sqrt(_WGS72_MU_KM / a ** 3) * 60.0  # rad/min
    # Parameters: n, e cos w, e sin w, i, raan, mean longitude (M + w), B*
    p = np.array([n, e * math.cos(argp), e * math.sin(argp), inc, raan, ma + argp, 1e-4])
    stamp = datetime.fromtimestamp(epoch_unix, tz=timezone.utc)
    jd0, fr0 = jday(stamp.year, stamp.month, stamp.day, stamp.hour, stamp.minute,
                    stamp.second + stamp.microsecond * 1e-6)
    epoch_1949 = jd0 + fr0 - 2433281.5
    jd_all = np.full(len(times), jd0)
    fr_all = fr0 + (times - epoch_unix) / 86400.0

    def positions(q):
        ecc = math.hypot(q[1], q[2])
        w = math.atan2(q[2], q[1])
        if not 0 <= ecc < 0.5 or q[0] <= 0:
            return None
        sat = Satrec()
        sat.sgp4init(WGS72, "i", satnum, epoch_1949, q[6], 0.0, 0.0, ecc, w, q[3], (q[5] - w) % (2 * math.pi),
                     q[0], q[4])
        err, rr, _ = sat.sgp4_array(jd_all, fr_all)
        if np.any(err):
            return None
        return rr

    def cost(q):
        rr = positions(q)
        return math.inf if rr is None else float(np.sum((rr - r_km) ** 2))

    step = np.array([1e-7, 1e-6, 1e-6, 1e-6, 1e-6, 1e-6, 1e-6])
    lam = 1e-3
    current = cost(p)
    for _ in range(iterations):
        base = positions(p)
        if base is None:
            break
        res = (r_km - base).ravel()
        jac = np.empty((res.size, 7))
        for j in range(7):
            q = p.copy()
            q[j] += step[j]
            rr = positions(q)
            if rr is None:
                rr = base
            jac[:, j] = (rr - base).ravel() / step[j]
        normal = jac.T @ jac
        rhs = jac.T @ res
        while lam < 1e10:
            dp = np.linalg.solve(normal + lam * np.diag(np.diag(normal)), rhs)
            trial = cost(p + dp)
            if trial < current:
                p, current, lam = p + dp, trial, lam / 10
                break
            lam *= 10
        else:
            break
        if np.all(np.abs(dp) < step * 1e-2):
            break

    rr = positions(p)
    dev = np.linalg.norm(rr - r_km, axis=1) * 1000.0
    ecc = math.hypot(p[1], p[2])
    w = math.atan2(p[2], p[1])
    line1, line2 = format_tle(
        satnum, intl_designator, epoch_unix, 0.0, p[6], math.degrees(p[3]), math.degrees(p[4]), ecc,
        math.degrees(w), math.degrees((p[5] - w) % (2 * math.pi)), p[0] * MINUTES_PER_DAY / (2 * math.pi))
    return TleFit(line1, line2, float(np.sqrt(np.mean(dev ** 2))), float(dev.max()), epoch_unix)
