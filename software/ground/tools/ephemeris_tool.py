#!/usr/bin/env python3
"""
SMART-QSO operator ephemeris from onboard GNSS fixes (SQSO-GND-002).

Reads the GNSS state frames from beacon and telemetry archives, fits the orbit
(batch least squares, orbit_determination.py), and writes:

  <object>_<epoch>_<frame>.oem      CCSDS OEM 3.0 (KVN), UTC, states every
                                    --step-s with a covariance per record
  <object>_<epoch>.tle              two-line elements fitted with SGP4
  <object>_<epoch>_report.json      fixes used and rejected, residuals,
                                    drag scale, covariance growth, quality flag

Quality flag (SQSO-GND-002 §4.5): GOOD and FAIR are uploaded (SpaceX Space
Traffic Coordination and Space-Track); POOR is written with the prefix
NOT-FOR-UPLOAD_ and the tool exits 2; a fit that cannot be done (FAIL) writes
no products and exits 3.

    python3 ephemeris_tool.py --frames pass1.jsonl pass2.jsonl --out-dir eph/ \\
        --object-id 2028-123A --norad-id 99999 --frame ITRF
    python3 ephemeris_tool.py --frames *.jsonl --out-dir eph/ --frame EME2000 --dut1 0.0712

Read SQSO-GND-002 §7 (the 24 h first-ephemeris procedure) before an upload.
"""

from __future__ import annotations

import argparse
import json
import math
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Optional, Sequence, Tuple

import numpy as np

import orbit_frames as fr
from ephemeris_products import FRAMES, OemMeta, fit_tle, iso, read_oem, write_oem
from gnss_frame import DEFAULT_PDOP, TAI_UTC_S, Received, fix_time_unix, read_archive, resolve_week
from orbit_determination import (DRAG_FORECAST_SIGMA, FitError, ForceModel, GravityField, Observation,
                                 fit_orbit, predict, quality_flag)

#: Per-axis position sigma of a fix per unit PDOP, m (piNAV-NG: 10 m 2-sigma
#: horizontal at PDOP ~2, D6 research §5)
SIGMA_PER_PDOP_M = 2.5
#: Force-model error floor per axis, m, added in quadrature to each fix's sigma:
#: the gravity left out of the field (and the drag variation within the arc)
#: is correlated along the orbit, not white, so without it the covariance is
#: too small between and after sparse fixes (simulation: 300-500 m between
#: sessions with the 4 x 4 field; SQSO-GND-002 §6)
MODEL_SIGMA_M = ((4, 150.0), (8, 100.0), (12, 60.0))
MODEL_SIGMA_FINEST_M = 30.0


def model_sigma(degree: int) -> float:
    """Default force-model error floor for a gravity field of this degree."""
    for limit, sigma in MODEL_SIGMA_M:
        if degree <= limit:
            return sigma
    return MODEL_SIGMA_FINEST_M


#: The onboard chord velocity's method error, m/s (gnss_frame.rs: 8 mm/s over 120 s)
CHORD_METHOD_SIGMA_MS = 0.01
#: Mean motion at 475 km, rad/s: a position error floor as a velocity floor
MEAN_MOTION = 1.11e-3

#: Fixes closer than this belong to one session
SESSION_GAP_S = 600.0
EXIT_OK, EXIT_POOR, EXIT_FAIL = 0, 2, 3


def observations(received: Sequence[Received], sigma_per_pdop: float = SIGMA_PER_PDOP_M,
                 sigma_v: Optional[float] = None, tai_utc_s: float = TAI_UTC_S,
                 since: Optional[float] = None, until: Optional[float] = None,
                 model_sigma_m: float = 0.0, use_velocity: bool = True
                 ) -> Tuple[List[Observation], List[Tuple[str, str]]]:
    """Fit observations from decoded frames, and the frames not used (label, reason)."""
    seen = {}
    rejected = []
    for rec in received:
        fix = rec.fix
        week = resolve_week(fix.week, rec.rx_unix)
        label = f"GPS {week}/{fix.tow_ms / 1000:.3f}" + (f" ({rec.station})" if rec.station else "")
        problems = fix.problems()
        if problems:
            rejected.append((label, "; ".join(problems)))
            continue
        key = (week, round(fix.tow_ms))
        if key in seen:
            continue  # the same frame heard by another station
        t = fix_time_unix(fix, rec.rx_unix, tai_utc_s)
        if (since is not None and t < since) or (until is not None and t > until):
            rejected.append((label, "outside the fit window"))
            continue
        pdop = fix.pdop or DEFAULT_PDOP
        sv = None
        if fix.v_ecef_ms is not None and use_velocity:
            # The chord velocity's noise (gnss_frame.rs), its method error, and
            # the force-model floor carried to velocity (times the mean motion)
            chord = math.sqrt(2) * sigma_per_pdop * pdop / fix.vdt_s if fix.vdt_s else 1.0
            sv = math.sqrt(chord ** 2 + CHORD_METHOD_SIGMA_MS ** 2 + (model_sigma_m * MEAN_MOTION) ** 2)
            if sigma_v is not None:
                sv = sigma_v
        seen[key] = Observation(
            t=t, r_itrf=np.array(fix.r_ecef_m, dtype=float),
            sigma_r=math.hypot(sigma_per_pdop * pdop, model_sigma_m),
            v_itrf=None if fix.v_ecef_ms is None else np.array(fix.v_ecef_ms, dtype=float),
            sigma_v=sv, label=label)
    obs = sorted(seen.values(), key=lambda o: o.t)
    session, last = 0, None
    for o in obs:
        if last is not None and o.t - last > SESSION_GAP_S:
            session += 1
        o.session, last = session, o.t
    return obs, rejected


def prior_from_tle(path: Path) -> Tuple[float, np.ndarray]:
    """TEME state at a TLE's epoch (e.g. the launch provider's predicted elements)."""
    from sgp4.api import Satrec

    lines = [ln for ln in Path(path).read_text().splitlines() if ln.startswith(("1 ", "2 "))]
    sat = Satrec.twoline2rv(lines[0], lines[1])
    err, r, v = sat.sgp4(sat.jdsatepoch, sat.jdsatepochF)
    if err:
        raise ValueError(f"SGP4 error {err} at the TLE epoch")
    epoch = (sat.jdsatepoch + sat.jdsatepochF - fr.JD_UNIX_EPOCH) * 86400.0
    return epoch, np.array(r + v) * 1000.0


def _stamp(unix_s: float) -> str:
    return datetime.fromtimestamp(unix_s, tz=timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def run(argv: Optional[Sequence[str]] = None) -> int:
    opts = _parse_args(argv)
    if opts.frame == "EME2000" and opts.dut1 is None:
        print("error: EME2000 needs --dut1 (UT1 - UTC from IERS Bulletin A): up to 450 m of error without it",
              file=sys.stderr)
        return EXIT_FAIL
    received, errors = read_archive(opts.frames)
    for err in errors:
        print(f"warning: {err}", file=sys.stderr)
    since = _time_arg(opts.since)
    until = _time_arg(opts.until)
    degree = opts.gravity_degree if opts.gravity_file else min(opts.gravity_degree, 4)
    floor = opts.model_sigma if opts.model_sigma is not None else model_sigma(degree)
    obs, refused = observations(received, opts.sigma_per_pdop, opts.sigma_vel, opts.tai_utc, since, until, floor,
                                use_velocity=not opts.no_velocity)
    if opts.window_h and obs:
        newest = obs[-1].t
        refused += [(o.label, "older than the fit window") for o in obs if o.t < newest - opts.window_h * 3600]
        obs = [o for o in obs if o.t >= newest - opts.window_h * 3600]
    print(f"{len(received)} GNSS frames, {len(obs)} fixes for the fit, {len(refused)} not used")

    gravity = (GravityField.from_icgem(opts.gravity_file, opts.gravity_degree) if opts.gravity_file
               else GravityField.builtin(min(opts.gravity_degree, 4)))
    model = ForceModel(gravity=gravity, bc=opts.cd * opts.area / opts.mass)
    prior = prior_from_tle(opts.prior_tle) if opts.prior_tle else None
    try:
        fit = fit_orbit(obs, model=model, prior=prior, use_velocity=not opts.no_velocity)
    except FitError as exc:
        print(f"FAIL: {exc}. No products written (SQSO-GND-002 §7: request more fixes or use the contingency)",
              file=sys.stderr)
        return EXIT_FAIL

    last = max(o.t for o in fit.used)
    start = _time_arg(opts.start) if opts.start else math.ceil(last / opts.step_s) * opts.step_s
    grid = start + np.arange(0, opts.span_h * 3600 + 1e-6, opts.step_s)
    pred = predict(fit, np.concatenate([[last + 86400.0], grid]), opts.drag_forecast)
    sigma_24h = float(pred.sigma_pos_3d()[0])
    pred.times, pred.states, pred.covariances = pred.times[1:], pred.states[1:], pred.covariances[1:]
    flag, reasons = quality_flag(fit, sigma_24h)
    sig = pred.sigma_pos_3d()

    out = Path(opts.out_dir)
    out.mkdir(parents=True, exist_ok=True)
    prefix = "" if flag in ("GOOD", "FAIR") else "NOT-FOR-UPLOAD_"
    base = f"{prefix}{opts.object_name}_{_stamp(last)}"
    comments = [
        f"Forward-predicted operator ephemeris from onboard GNSS fixes; SQSO-GND-002; quality {flag}",
        f"Fit: {len(fit.used)} fixes in {fit.sessions} sessions over {fit.arc_s / 3600:.1f} h, last fix {iso(last)} UTC",
        f"RMS position residual {fit.rms_m:.1f} m per axis; drag scale {fit.drag_scale:.3f} "
        f"+/- {math.sqrt(fit.covariance[6, 6]):.3f} on Cd A/m {model.bc:.4f} m2/kg",
        f"Force model: {model.describe()}; model error floor {floor:.0f} m per axis",
        f"Position 1-sigma (3D): {sig[0] / 1000:.2f} km at start, {sigma_24h / 1000:.2f} km at last fix + 24 h, "
        f"{sig[-1] / 1000:.2f} km at stop; includes a {opts.drag_forecast:.0%} drag forecast uncertainty",
        f"Contact: {opts.originator} (licensee); "
        + (f"DUT1 {opts.dut1} s (IERS Bulletin A)" if opts.dut1 is not None else "DUT1 not needed (Earth-fixed frame)"),
    ]
    files = {}
    try:
        oem = write_oem(out / f"{base}_{opts.frame}.oem", pred,
                        OemMeta(opts.object_name, opts.object_id, opts.originator, opts.frame, comments),
                        dut1_s=opts.dut1, tai_utc_s=opts.tai_utc, cov_every=opts.cov_every)
    except ValueError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return EXIT_FAIL
    check = read_oem(oem)  # the file reads back with every keyword the upload needs
    if len(check.times) != len(grid) or not np.all(np.isfinite(check.states)):
        print(f"error: {oem} does not read back", file=sys.stderr)
        return EXIT_FAIL
    files["oem"] = str(oem)

    tle = None
    if not opts.no_tle:
        try:
            dut1 = opts.dut1 or 0.0
            r_teme = np.einsum("nij,nj->ni", fr.teme_utc_to_ut1(pred.times, dut1), pred.states[:, :3])
            v_teme = np.einsum("nij,nj->ni", fr.teme_utc_to_ut1(pred.times, dut1), pred.states[:, 3:])
            sample = slice(None, None, max(1, int(600 / opts.step_s)))
            tle = fit_tle(pred.times[sample], r_teme[sample], v_teme[sample], round(last), opts.norad_id,
                          opts.intl_designator)
            path = out / f"{base}.tle"
            path.write_text(f"{opts.object_name}\n{tle.line1}\n{tle.line2}\n", encoding="ascii")
            files["tle"] = str(path)
        except ImportError:
            print("warning: sgp4 not installed (pip install sgp4): no TLE", file=sys.stderr)

    report = {
        "document": "SQSO-GND-002",
        "quality": flag,
        "quality_reasons": reasons,
        "fixes_used": len(fit.used),
        "sessions": fit.sessions,
        "arc_h": round(fit.arc_s / 3600, 3),
        "last_fix_utc": iso(last),
        "rms_residual_m": round(fit.rms_m, 2),
        "chi2_reduced": round(fit.chi2_reduced, 3),
        "drag_scale": round(fit.drag_scale, 4),
        "drag_scale_sigma": round(math.sqrt(fit.covariance[6, 6]), 4),
        "converged": fit.converged,
        "iterations": fit.iterations,
        "force_model": model.describe(),
        "model_sigma_m": floor,
        "sigma_3d_km": {"start": round(sig[0] / 1000, 3), "last_fix_plus_24h": round(sigma_24h / 1000, 3),
                        "stop": round(sig[-1] / 1000, 3)},
        "frame": opts.frame,
        "span": [iso(grid[0]), iso(grid[-1])],
        "tle_fit_rms_m": None if tle is None else round(tle.rms_m, 1),
        "residuals": [{"fix": o.label, "utc": iso(o.t), "dx_m": round(float(r[0]), 2), "dy_m": round(float(r[1]), 2),
                       "dz_m": round(float(r[2]), 2)} for o, r in zip(fit.used, fit.residuals)],
        "rejected": [{"fix": o.label, "reason": why} for o, why in fit.rejected] +
                    [{"fix": label, "reason": why} for label, why in refused],
        "files": files,
    }
    path = out / f"{base}_report.json"
    path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    files["report"] = str(path)

    print(f"quality {flag}" + (": " + "; ".join(reasons) if reasons else ""))
    print(f"fit: {len(fit.used)} fixes, {fit.sessions} sessions, arc {fit.arc_s / 3600:.1f} h, RMS {fit.rms_m:.1f} m, "
          f"drag scale {fit.drag_scale:.3f} +/- {math.sqrt(fit.covariance[6, 6]):.3f}")
    print(f"position 1-sigma: {sig[0] / 1000:.2f} km at start, {sigma_24h / 1000:.2f} km at +24 h, "
          f"{sig[-1] / 1000:.2f} km at stop")
    for kind, name in files.items():
        print(f"{kind}: {name}")
    return EXIT_OK if flag in ("GOOD", "FAIR") else EXIT_POOR


def _time_arg(text: Optional[str]) -> Optional[float]:
    if not text:
        return None
    stamp = datetime.fromisoformat(text.replace("Z", "+00:00"))
    return (stamp if stamp.tzinfo else stamp.replace(tzinfo=timezone.utc)).timestamp()


def _parse_args(argv):
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--frames", nargs="+", type=Path, required=True, help="Archive files (JSON lines or text)")
    p.add_argument("--out-dir", type=Path, required=True)
    p.add_argument("--object-name", default="SMART-QSO")
    p.add_argument("--object-id", default="UNKNOWN", help="International designator once assigned (e.g. 2028-123A)")
    p.add_argument("--norad-id", type=int, default=99999, help="Catalog number for the TLE (99999 until assigned)")
    p.add_argument("--intl-designator", default="", help="TLE international designator field (e.g. 28123A)")
    p.add_argument("--originator", default="KK6GDB")
    p.add_argument("--frame", choices=FRAMES, default="ITRF")
    p.add_argument("--dut1", type=float, help="UT1 - UTC, s (IERS Bulletin A); required for EME2000")
    p.add_argument("--tai-utc", type=float, default=TAI_UTC_S, help="TAI - UTC, s (IERS Bulletin C)")
    p.add_argument("--span-h", type=float, default=72.0, help="Prediction span, h (3 to 7 days)")
    p.add_argument("--step-s", type=float, default=60.0)
    p.add_argument("--cov-every", type=int, default=1, help="A covariance block every N records")
    p.add_argument("--start", help="First epoch (ISO UTC); default: the first step after the last fix")
    p.add_argument("--since", help="Ignore fixes before this time (ISO UTC)")
    p.add_argument("--until", help="Ignore fixes after this time (ISO UTC)")
    p.add_argument("--window-h", type=float, default=48.0, help="Fit the fixes of the last N hours (0: all)")
    p.add_argument("--sigma-per-pdop", type=float, default=SIGMA_PER_PDOP_M)
    p.add_argument("--model-sigma", type=float,
                   help="Force-model error floor per axis, m (default by gravity degree: 150 m to 4, 100 to 8, 60 to 12, 30 above)")
    p.add_argument("--no-velocity", action="store_true",
                   help="Do not fit the frames' chord velocities (they still start the iteration)")
    p.add_argument("--sigma-vel", type=float,
                   help="Velocity sigma per axis, m/s (default: from VDT, sqrt(2) x position sigma / VDT)")
    p.add_argument("--gravity-file", type=Path, help="ICGEM .gfc gravity field (e.g. EGM2008)")
    p.add_argument("--gravity-degree", type=int, default=4)
    p.add_argument("--cd", type=float, default=2.2)
    p.add_argument("--area", type=float, default=0.0247, help="Drag area, m^2 (0.0164 with the wings failed)")
    p.add_argument("--mass", type=float, default=1.6, help="kg")
    p.add_argument("--drag-forecast", type=float, default=DRAG_FORECAST_SIGMA,
                   help="1-sigma drag change after the last fix, fraction")
    p.add_argument("--prior-tle", type=Path, help="Initial orbit when the fixes carry no velocity")
    p.add_argument("--no-tle", action="store_true")
    return p.parse_args(argv)


if __name__ == "__main__":
    sys.exit(run())
