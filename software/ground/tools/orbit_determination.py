#!/usr/bin/env python3
"""
SMART-QSO orbit determination from onboard GNSS fixes (SQSO-GND-002 §4).

Batch weighted least squares on the downlinked ECEF positions and the chord
velocities the flight software derives from them (gnss_frame.rs; weighted by
their own noise, sqrt(2) x position sigma / VDT), with the numerically
integrated force model of
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

from dataclasses import dataclass, field
from typing import List, Optional, Sequence, Tuple

import numpy as np

import orbit_frames as fr
from orbit_dynamics import GM, ForceModel, GravityField, propagate  # noqa: F401  (GravityField re-exported)

#: Huber threshold of the robust stage, normalized residual per fix
HUBER_SIGMA = 3.0

#: Fewest measured values (position and velocity components) for a fit: the
#: 7 parameters and 2 degrees of freedom
MIN_VALUES = 9

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
    weight: float = 1.0              # robust down-weighting during the fit (1: full)


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
    w = [np.repeat([o.weight / o.sigma_r ** 2 for o in obs], 3)]
    if use_vel:
        vel = [o for o in obs if o.sigma_v is not None]
        if vel:
            z.append(np.concatenate([o.v_itrf for o in vel]))
            w.append(np.repeat([o.weight / o.sigma_v ** 2 for o in vel], 3))
    return np.concatenate(z), np.concatenate(w)


@dataclass
class _Evaluation:
    cost: float
    res: np.ndarray
    w: np.ndarray
    normal: np.ndarray
    rhs: np.ndarray


def _evaluate(model, epoch, p, obs, use_vel, z, w, k_prior, k_sigma) -> _Evaluation:
    states, ks = _perturbed(p, _DELTA)
    traj = propagate(model, epoch, states, ks, [o.t for o in obs])
    pred = _model_obs(traj, obs, use_vel)
    res = z - pred[0]
    h = ((pred[1:] - pred[0]) / _DELTA[:, None]).T
    prior_inv = np.zeros((7, 7))
    prior_inv[6, 6] = 1.0 / k_sigma ** 2
    cost = float(res @ (w * res)) + float((p[6] - k_prior) ** 2 / k_sigma ** 2)
    normal = h.T @ (w[:, None] * h) + prior_inv
    rhs = h.T @ (w * res) + prior_inv[:, 6] * (k_prior - p[6])
    return _Evaluation(cost, res, w, normal, rhs)


#: Convergence: every correction below this fraction of its finite-difference
#: perturbation (0.5 m, 0.5 mm/s, 0.001 in k)
_TOLERANCE = 0.05


def _iterate(model, epoch, p, obs, use_vel, k_prior, k_sigma, max_iter=20):
    """Levenberg-Marquardt: (p, evaluation at p, iterations, converged)."""
    z, w = _measurements(obs, use_vel)
    ev = _evaluate(model, epoch, p, obs, use_vel, z, w, k_prior, k_sigma)
    lam = 0.0
    for it in range(1, max_iter + 1):
        try:
            dp = np.linalg.solve(ev.normal + lam * np.diag(np.diag(ev.normal)), ev.rhs)
        except np.linalg.LinAlgError as exc:
            raise FitError("normal matrix singular: the fixes do not determine the orbit") from exc
        trial = p + dp
        trial[6] = max(trial[6], 0.05)
        ev_trial = _evaluate(model, epoch, trial, obs, use_vel, z, w, k_prior, k_sigma)
        if ev_trial.cost <= ev.cost * (1 + 1e-9) or lam > 1e8:
            p, ev = trial, ev_trial
            lam = lam / 10 if lam > 1e-6 else 0.0
            if np.all(np.abs(dp / _DELTA) < _TOLERANCE):
                return p, ev, it, True
        else:
            lam = max(lam * 10, 1e-3)
    return p, ev, max_iter, False


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
    values = sum(3 + (3 if use_velocity and o.sigma_v is not None else 0) for o in obs)
    if len(obs) < 2 or values < MIN_VALUES:
        raise FitError(f"{len(obs)} usable fixes ({values} measured values): at least {MIN_VALUES} values "
                       "from at least 2 fixes are needed (3 positions, or 2 fixes with velocity)")
    epoch, x0 = initial_state(obs, prior)
    p = np.concatenate([x0, [k_prior]])
    rejected: List[Tuple[Observation, str]] = []
    used = list(obs)

    # Without a measured velocity the start is rough: grow the arc from the
    # initial state outwards, so the fit cannot lock onto the wrong revolution
    total = max(abs(o.t - epoch) for o in used)
    span = 2700.0 if not any(o.v_itrf is not None for o in used) else total
    iterations = 0
    while span < total:
        window = [o for o in used if abs(o.t - epoch) <= span]
        if len(window) >= 3:
            p, _, its, _ = _iterate(model, epoch, p, window, use_velocity, k_prior, k_sigma, max_iter=8)
            iterations += its
        span *= 4.0
    p, ev, its, converged = _iterate(model, epoch, p, used, use_velocity, k_prior, k_sigma)
    iterations += its

    def normalized(ev_):
        res = ev_.res[: 3 * len(used)].reshape(-1, 3)
        return np.sqrt(np.mean((res / np.array([[o.sigma_r] for o in used])) ** 2, axis=1))

    # Robust stage: with few fixes one gross error can drag the whole fit, so
    # its residual looks no worse than the others'. Reweight (Huber, 3 sigma
    # per fix) until the weights settle, so that a bad fix stands out
    for _ in range(5):
        norm = normalized(ev)
        weights = np.minimum(1.0, HUBER_SIGMA / np.maximum(norm, 1e-9))
        if np.allclose(weights, [o.weight for o in used], atol=0.02):
            break
        for o, wt in zip(used, weights):
            o.weight = float(wt)
        p, ev, its, converged = _iterate(model, epoch, p, used, use_velocity, k_prior, k_sigma)
        iterations += its

    # Outlier editing on the robust fit: drop fixes whose residual (RMS of
    # the three axes over the fix sigma) exceeds edit_sigma times the robust
    # scale of all residuals (1.4826 x median, at least 1: model error raises
    # every residual, an outlier only its own); refit at full weight; at most
    # three rounds
    for _ in range(3):
        if not converged:
            break
        norm = normalized(ev)
        res = ev.res[: 3 * len(used)].reshape(-1, 3)
        scale = max(1.0, 1.4826 * float(np.median(norm)))
        bad = [i for i in range(len(used)) if norm[i] > edit_sigma * scale]
        for o in used:
            o.weight = 1.0
        if not bad or len(used) - len(bad) < 2:
            p, ev, its, converged = _iterate(model, epoch, p, used, use_velocity, k_prior, k_sigma)
            iterations += its
            break
        for i in sorted(bad, reverse=True):
            rejected.append((used[i], f"residual {np.linalg.norm(res[i]):.0f} m ({norm[i]:.1f} sigma)"))
            del used[i]
        p, ev, its, converged = _iterate(model, epoch, p, used, use_velocity, k_prior, k_sigma)
        iterations += its

    if any(o.weight != 1.0 for o in used):  # the fit stopped early: full weights again
        for o in used:
            o.weight = 1.0
        p, ev, its, converged = _iterate(model, epoch, p, used, use_velocity, k_prior, k_sigma)
        iterations += its

    res, w, normal = ev.res, ev.w, ev.normal
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


#: 1-sigma of the drag over the prediction relative to the fitted drag: the
#: density changes with solar and geomagnetic activity after the last fix,
#: which no fit can know (D6 research §6.2 item 4 assumes 20 %)
DRAG_FORECAST_SIGMA = 0.2


def predict(fit: FitResult, times: Sequence[float], drag_forecast_sigma: float = DRAG_FORECAST_SIGMA) -> Prediction:
    """Propagate the fitted state and map its covariance.

    The covariance is the fit's (state and drag scale at the epoch) mapped
    with the state transition, plus the drag forecast term: the sensitivity to
    a change of k that starts at the last fix, times drag_forecast_sigma * k.
    """
    times = np.asarray(times, dtype=float)
    p = np.concatenate([fit.state_teme, [fit.drag_scale]])
    states, ks = _perturbed(p, fit.delta)
    traj = propagate(fit.model, fit.epoch, states, ks, times)
    phi = np.moveaxis((traj[1:] - traj[0]) / fit.delta[:, None, None], 0, -1)  # (n, 6, 7)
    cov = phi @ fit.covariance @ np.swapaxes(phi, -1, -2)
    last = max(o.t for o in fit.used)
    after = times > last
    if drag_forecast_sigma and fit.model.drag and np.any(after):
        start = propagate(fit.model, fit.epoch, states[:1], ks[:1], [last])[0, 0]
        dk = fit.delta[6]
        pair = propagate(fit.model, last, np.tile(start, (2, 1)), [fit.drag_scale, fit.drag_scale + dk], times[after])
        sens = (pair[1] - pair[0]) / dk  # (m, 6)
        sigma = drag_forecast_sigma * fit.drag_scale
        cov[after] += sigma ** 2 * sens[:, :, None] * sens[:, None, :]
    return Prediction(times, traj[0], cov)


# --- Quality -----------------------------------------------------------------

#: Quality flag thresholds on the 3D position 1-sigma 24 h after the last fix
GOOD_SIGMA_M = 5_000.0
FAIR_SIGMA_M = 20_000.0
#: One orbit: an arc shorter than this rests on one session
MIN_GOOD_ARC_S = 5_400.0
#: RMS position residual per axis: above GOOD the force model or the data are
#: suspect, above FAIR the fit is not used
GOOD_RMS_M = 50.0
FAIR_RMS_M = 200.0
MAX_REJECTED_FRACTION = 0.2


def quality_flag(fit: FitResult, sigma_24h_m: float) -> Tuple[str, List[str]]:
    """GOOD, FAIR or POOR, with the reasons for anything below GOOD."""
    reasons = []
    rejected_fraction = len(fit.rejected) / max(len(fit.rejected) + len(fit.used), 1)
    if not fit.converged:
        return "POOR", ["the fit did not converge"]
    if fit.rms_m > FAIR_RMS_M:
        reasons.append(f"RMS residual {fit.rms_m:.0f} m per axis (limit {FAIR_RMS_M:.0f} m)")
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
    if fit.rms_m > GOOD_RMS_M:
        reasons.append(f"RMS residual {fit.rms_m:.0f} m per axis (GOOD: <= {GOOD_RMS_M:.0f} m)")
    return ("FAIR" if reasons else "GOOD"), reasons
