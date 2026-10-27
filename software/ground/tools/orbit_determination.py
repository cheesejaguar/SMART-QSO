#!/usr/bin/env python3
"""
SMART-QSO orbit determination from onboard GNSS fixes (SQSO-GND-002 §4).

Batch weighted least squares on the downlinked ECEF positions (velocities
optional, off by default: the piNAV-NG velocity accuracy is not confirmed,
D6 research §6), with the numerically integrated force model of
orbit_dynamics.py (EGM96 4 x 4 gravity by default, drag with an estimated
scale factor). The drag scale k has an a-priori 1-sigma of 1.0 (density and
attitude are not known to better than a factor of two), so a short arc keeps
the drag uncertainty in the covariance and a long arc estimates it.

The partial derivatives, and the state transition for the covariance, come
from finite-difference trajectories integrated together with the nominal.

Estimated parameters: TEME position and velocity at the fit epoch, and k.
The formal covariance is scaled by the reduced chi-square when it exceeds 1.
"""

from __future__ import annotations

import math
from dataclasses import dataclass, field
from typing import List, Optional, Sequence, Tuple

import numpy as np

import orbit_frames as fr
from orbit_dynamics import GM, ForceModel, GravityField, propagate  # noqa: F401  (re-exported)

#: A-priori 1-sigma of the drag scale factor
DRAG_SCALE_SIGMA = 1.0


# --- Batch least squares -----------------------------------------------------

#: Finite-difference perturbations: position (m), velocity (m/s), drag scale
_DELTA = np.array([10.0, 10.0, 10.0, 0.01, 0.01, 0.01, 0.02])


@dataclass
class Observation:
    """One GNSS fix prepared for the fit."""

    t: float                         # POSIX UTC
    r_itrf: np.ndarray               # m
    sigma_r: float                   # m, per axis
    v_itrf: Optional[np.ndarray] = None
    sigma_v: Optional[float] = None  # m/s per axis; None: velocity not used
    session: int = 0
    label: str = ""


@dataclass
class FitResult:
    """Outcome of the batch least squares."""

    converged: bool
    epoch: float
    state_teme: np.ndarray           # (6,) at epoch
    drag_scale: float
    covariance: np.ndarray           # (7, 7): TEME state at epoch and k
    used: List[Observation]
    rejected: List[Tuple[Observation, str]]
    residuals: np.ndarray            # (n, 3) ITRF position residuals of the used fixes, m
    rms_m: float                     # RMS position residual per axis
    chi2_reduced: float
    iterations: int
    model: ForceModel
    message: str = ""
    delta: np.ndarray = field(default_factory=lambda: _DELTA.copy())

    @property
    def arc_s(self) -> float:
        times = [o.t for o in self.used]
        return max(times) - min(times) if times else 0.0

    @property
    def sessions(self) -> int:
        return len({o.session for o in self.used})


class FitError(Exception):
    """The fit cannot be done (too few fixes, no initial state, no convergence)."""


def _perturbed(p: np.ndarray, delta: np.ndarray):
    states = np.tile(p[:6], (8, 1))
    k = np.full(8, p[6])
    for j in range(6):
        states[j + 1, j] += delta[j]
    k[7] += delta[6]
    return states, k


def _model_obs(traj: np.ndarray, obs: Sequence[Observation], use_vel: bool):
    """Predicted ITRF measurements (8, m) for every trajectory."""
    times = np.array([o.t for o in obs])
    r, v = fr.teme_to_itrf(times[None, :], traj[..., :3], traj[..., 3:])
    parts = [r.reshape(r.shape[0], -1)]
    if use_vel:
        idx = [i for i, o in enumerate(obs) if o.sigma_v is not None]
        parts.append(v[:, idx].reshape(v.shape[0], -1))
    return np.concatenate(parts, axis=1)


def _measurements(obs: Sequence[Observation], use_vel: bool):
    z = [np.concatenate([o.r_itrf for o in obs])]
    w = [np.repeat([1.0 / o.sigma_r ** 2 for o in obs], 3)]
    if use_vel:
        vel = [o for o in obs if o.sigma_v is not None]
        if vel:
            z.append(np.concatenate([o.v_itrf for o in vel]))
            w.append(np.repeat([1.0 / o.sigma_v ** 2 for o in vel], 3))
    return np.concatenate(z), np.concatenate(w)


def _iterate(model, epoch, p, obs, use_vel, k_prior, k_sigma, max_iter=25):
    times = np.array([o.t for o in obs])
    z, w = _measurements(obs, use_vel)
    prior_inv = np.zeros((7, 7))
    prior_inv[6, 6] = 1.0 / k_sigma ** 2
    lam = 0.0
    last_cost = None
    for it in range(1, max_iter + 1):
        states, ks = _perturbed(p, _DELTA)
        traj = propagate(model, epoch, states, ks, times)
        pred = _model_obs(traj, obs, use_vel)
        res = z - pred[0]
        h = ((pred[1:] - pred[0]) / _DELTA[:, None]).T
        cost = float(res @ (w * res)) + float((p[6] - k_prior) ** 2 / k_sigma ** 2)
        normal = h.T @ (w[:, None] * h) + prior_inv
        rhs = h.T @ (w * res) + prior_inv[:, 6] * (k_prior - p[6])
        if last_cost is not None and cost > last_cost * 1.5 and lam < 1e6:
            lam = max(lam * 10, 1e-3)
        scale = np.sqrt(np.diag(normal))
        try:
            dp = np.linalg.solve(normal + lam * np.diag(scale ** 2), rhs)
        except np.linalg.LinAlgError as exc:
            raise FitError("normal matrix singular: the fixes do not determine the orbit") from exc
        p = p + dp
        p[6] = max(p[6], 0.05)
        last_cost = cost
        if np.all(np.abs(dp / _DELTA) < 1e-3):
            return p, normal, res, w, it, True
    return p, normal, res, w, max_iter, False


def initial_state(obs: Sequence[Observation], prior: Optional[Tuple[float, np.ndarray]] = None):
    """(epoch, TEME state) to start the fit from.

    Order of preference: a fix with velocity; two fixes of one session 2-600 s
    apart; a prior TEME state (epoch, (6,)) such as the launch provider's
    deployment state or a TLE.
    """
    with_v = [o for o in obs if o.v_itrf is not None]
    if with_v:
        o = with_v[-1]
        r, v = fr.itrf_to_teme(o.t, o.r_itrf, o.v_itrf)
        return o.t, np.concatenate([r, v])
    by_time = sorted(obs, key=lambda o: o.t)
    best = None
    for i, a in enumerate(by_time):
        within = [b for b in by_time[i + 1:] if b.t - a.t <= 600.0]
        if within and within[-1].t - a.t >= 2.0 and (best is None or within[-1].t - a.t > best[1].t - best[0].t):
            best = (a, within[-1])
    if best is not None:
        a, b = best
        ra = fr.itrf_to_teme(a.t, a.r_itrf)
        rb = fr.itrf_to_teme(b.t, b.r_itrf)
        dt = b.t - a.t
        acc = -GM * ra / np.linalg.norm(ra) ** 3
        return a.t, np.concatenate([ra, (rb - ra) / dt - 0.5 * acc * dt])
    if prior is not None:
        return prior[0], np.asarray(prior[1], dtype=float)
    raise FitError("no initial state: the fixes carry no velocity, no session has two fixes, and no prior state was given")


def fit_orbit(obs: Sequence[Observation], model: Optional[ForceModel] = None,
              prior: Optional[Tuple[float, np.ndarray]] = None,
              use_velocity: bool = False, k_prior: float = 1.0, k_sigma: float = DRAG_SCALE_SIGMA,
              edit_sigma: float = 5.0) -> FitResult:
    """Batch least squares on the fixes; raises FitError when it cannot be done."""
    model = model or ForceModel()
    obs = sorted(obs, key=lambda o: o.t)
    if len(obs) < 3:
        raise FitError(f"{len(obs)} usable fixes: at least 3 are needed")
    epoch, x0 = initial_state(obs, prior)
    p = np.concatenate([x0, [k_prior]])
    rejected: List[Tuple[Observation, str]] = []
    used = list(obs)

    # Grow the arc from the initial state outwards, so that a poor start
    # (finite-difference velocity) cannot lock onto the wrong revolution
    span = 1800.0
    total = max(abs(o.t - epoch) for o in used)
    iterations = 0
    converged = False
    while True:
        window = [o for o in used if abs(o.t - epoch) <= span] if span < total else used
        if len(window) >= 3:
            p, normal, res, w, its, converged = _iterate(model, epoch, p, window, use_velocity, k_prior, k_sigma)
            iterations += its
        if span >= total:
            break
        span *= 3.0

    # Outlier editing: drop fixes whose residual exceeds edit_sigma (RMS of the
    # three axes, normalized), refit, at most three rounds
    for _ in range(3):
        states, ks = _perturbed(p, _DELTA)
        traj = propagate(model, epoch, states[:1], ks[:1], [o.t for o in used])
        pred = _model_obs(traj, used, False)[0].reshape(-1, 3)
        res = np.array([o.r_itrf for o in used]) - pred
        norm = np.sqrt(np.mean((res / np.array([[o.sigma_r] for o in used])) ** 2, axis=1))
        bad = [i for i in np.argsort(-norm) if norm[i] > edit_sigma]
        if not bad or len(used) - len(bad) < 3:
            break
        for i in sorted(bad, reverse=True):
            rejected.append((used[i], f"residual {np.linalg.norm(res[i]):.0f} m ({norm[i]:.1f} sigma)"))
            del used[i]
        p, normal, res, w, its, converged = _iterate(model, epoch, p, used, use_velocity, k_prior, k_sigma)
        iterations += its

    p, normal, res, w, its, converged = _iterate(model, epoch, p, used, use_velocity, k_prior, k_sigma)
    iterations += its
    m = len(res)
    dof = max(m - 7 + 1, 1)  # the drag prior adds one pseudo-measurement
    chi2 = float(res @ (w * res)) / dof
    try:
        cov = np.linalg.inv(normal)
    except np.linalg.LinAlgError as exc:
        raise FitError("normal matrix singular") from exc
    if chi2 > 1.0:
        cov = cov * chi2
    pos_res = res[: 3 * len(used)].reshape(-1, 3)
    return FitResult(
        converged=converged, epoch=epoch, state_teme=p[:6].copy(), drag_scale=float(p[6]),
        covariance=cov, used=used, rejected=rejected, residuals=pos_res,
        rms_m=float(np.sqrt(np.mean(pos_res ** 2))), chi2_reduced=chi2, iterations=iterations, model=model,
    )


@dataclass
class Prediction:
    """Predicted states and covariances on a time grid (TEME, m and m/s)."""

    times: np.ndarray
    states: np.ndarray       # (n, 6)
    covariances: np.ndarray  # (n, 6, 6)

    def sigma_pos_3d(self) -> np.ndarray:
        return np.sqrt(np.trace(self.covariances[:, :3, :3], axis1=1, axis2=2))


def predict(fit: FitResult, times: Sequence[float]) -> Prediction:
    """Propagate the fitted state and map its covariance (state and drag scale)."""
    p = np.concatenate([fit.state_teme, [fit.drag_scale]])
    states, ks = _perturbed(p, fit.delta)
    traj = propagate(fit.model, fit.epoch, states, ks, times)
    phi = np.moveaxis((traj[1:] - traj[0]) / fit.delta[:, None, None], 0, -1)  # (n, 6, 7)
    cov = phi @ fit.covariance @ np.swapaxes(phi, -1, -2)
    return Prediction(np.asarray(times, dtype=float), traj[0], cov)


# --- Quality -----------------------------------------------------------------

#: Quality flag thresholds on the 3D position 1-sigma 24 h after the last fix
GOOD_SIGMA_M = 2_000.0
FAIR_SIGMA_M = 10_000.0
#: One orbit: an arc shorter than this rests on one session
MIN_GOOD_ARC_S = 5_400.0
MAX_GOOD_CHI2 = 4.0
MAX_FAIR_CHI2 = 9.0
MAX_REJECTED_FRACTION = 0.2


def quality_flag(fit: FitResult, sigma_24h_m: float) -> Tuple[str, List[str]]:
    """GOOD, FAIR or POOR, with the reasons for anything below GOOD."""
    reasons = []
    rejected_fraction = len(fit.rejected) / max(len(fit.rejected) + len(fit.used), 1)
    if not fit.converged:
        return "POOR", ["the fit did not converge"]
    if fit.chi2_reduced > MAX_FAIR_CHI2:
        reasons.append(f"residuals {math.sqrt(fit.chi2_reduced):.1f} times the fix noise")
    if rejected_fraction > MAX_REJECTED_FRACTION:
        reasons.append(f"{rejected_fraction:.0%} of the fixes rejected")
    if sigma_24h_m > FAIR_SIGMA_M:
        reasons.append(f"position 1-sigma {sigma_24h_m / 1000:.1f} km at +24 h")
    if reasons:
        return "POOR", reasons
    if sigma_24h_m > GOOD_SIGMA_M:
        reasons.append(f"position 1-sigma {sigma_24h_m / 1000:.1f} km at +24 h (GOOD: <= {GOOD_SIGMA_M / 1000:.0f} km)")
    if fit.arc_s < MIN_GOOD_ARC_S:
        reasons.append(f"arc {fit.arc_s / 60:.0f} min, shorter than one orbit")
    if fit.chi2_reduced > MAX_GOOD_CHI2:
        reasons.append(f"residuals {math.sqrt(fit.chi2_reduced):.1f} times the fix noise")
    return ("FAIR" if reasons else "GOOD"), reasons
