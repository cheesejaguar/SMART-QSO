//! Thermospheric density and the solar-activity driver.
//!
//! # Density model
//!
//! Diffusive equilibrium above 120 km with the Bates temperature profile in
//! its closed form (Walker, *J. Atmos. Sci.* 22, 1965; the profile Jacchia and
//! MSIS use above 120 km):
//!
//! * T(ξ) = T∞ − (T∞ − T₁₂₀) e^(−s ξ), with ξ the geopotential height above
//!   120 km;
//! * nᵢ(ξ) = nᵢ(120) (T₁₂₀/T)^(1+αᵢ+γᵢ) e^(−σ γᵢ ξ), γᵢ = mᵢ g₁₂₀ / (σ k T∞),
//!   σ = s + 1/(R + 120 km), for N₂, O₂, O, He (thermal diffusion α = −0.38)
//!   and Ar (H is neglected: < 3 % of the density below 700 km);
//! * exospheric temperature in the Jacchia-1971 form,
//!   T∞ = T₀ + c₁ F̄ + c₂ (F − F̄) + ΔT(Ap), with ΔT = 28 Kp + 0.03 e^Kp.
//!
//! The coefficients (T₀, c₁, s(T∞), nᵢ(120) and T₁₂₀ as functions of F̄) were
//! **fitted to NRLMSIS 2.1** (`pymsis` 0.13), averaged exactly as
//! `software/simulation/orbit_lifetime/orbit_lifetime.py` averages it (24
//! points of argument of latitude at 97.4°, 24 local times, four seasons),
//! for F10.7 = F10.7A = 65…220 sfu with that script's Ap(F10.7). Over
//! 150–750 km the fit is within ±5 % of the averaged NRLMSIS density (±10 %
//! above 600 km at F10.7 = 220). c₂ keeps Jacchia's ratio c₂/c₁ = 1.3/3.24,
//! which the averaged table cannot separate.
//!
//! The result is a **global, diurnally averaged** density: no day/night
//! bulge, latitude or season. That is what the secular decay needs; drag
//! torque and density telemetry see the mean value.
//!
//! # Solar activity
//!
//! [`f107_profile`] is the half-cosine interpolation between the turning
//! points of `orbit_lifetime.py` (`CYCLE_POINTS`: cycle 25 maximum 185 sfu
//! at 2024.8, minimum 68 sfu at 2030.5, cycle 26 maximum 150 sfu at 2035.0),
//! the "nominal" profile behind the BASELINE §3 lifetimes, with a ±8 %
//! 27-day solar-rotation modulation on the daily value.

use core::f64::consts::TAU;

use super::math::{DAY_S, K_BOLTZMANN};
use super::time::decimal_year;

/// Atomic mass unit (kg).
const AMU: f64 = 1.660_539_07e-27;
/// Effective Earth radius for geopotential height (km, MSIS).
const RE_GEO_KM: f64 = 6356.77;
/// Standard gravity (m/s²).
const G0: f64 = 9.80665;

/// One species: molecular mass (u), thermal diffusion factor, and
/// ln n(120 km) (m⁻³) = `c0 + c1 F̄` (fit to NRLMSIS 2.1).
struct Species {
    mass_u: f64,
    alpha: f64,
    c0: f64,
    c1: f64,
}

const SPECIES: [Species; 5] = [
    Species { mass_u: 28.0134, alpha: 0.0, c0: 40.064_327, c1: 6.105_618e-4 }, // N2
    Species { mass_u: 31.9988, alpha: 0.0, c0: 38.542_667, c1: -1.832_990e-3 }, // O2
    Species { mass_u: 15.9994, alpha: 0.0, c0: 38.298_753, c1: 2.620_019e-3 }, // O
    Species { mass_u: 4.0026, alpha: -0.38, c0: 31.110_464, c1: 2.801_567e-4 }, // He
    Species { mass_u: 39.948, alpha: 0.0, c0: 34.741_404, c1: 8.658_297e-4 },  // Ar
];

/// Exospheric temperature fit: T∞ = T0 + C1 F̄ + C2 (F − F̄) + ΔT(Ap) (K).
const T_INF0: f64 = 491.32;
const T_INF_C1: f64 = 3.4918;
const T_INF_C2: f64 = 3.4918 * 1.3 / 3.24;
/// Temperature at 120 km: T120 = A + B F̄ (K).
const T120_A: f64 = 338.88;
const T120_B: f64 = 0.236_80;
/// Shape parameter s = S0 + S1 (T∞ − 1000)/1000 (km⁻¹).
const S0: f64 = 0.026_165;
const S1: f64 = -0.022_792;
/// Density scale height used below 120 km (km).
const H_BELOW_120_KM: f64 = 6.0;

/// Approximate Kp for a 3-hour ap (inverse of the ap(Kp) table, ±0.3).
pub fn kp_from_ap(ap: f64) -> f64 {
    (1.75 * ap.max(1e-3).ln() - 1.6).clamp(0.0, 9.0)
}

/// Exospheric temperature (K) for daily F10.7, its 81-day mean F10.7A (sfu)
/// and the daily Ap.
pub fn exospheric_temperature(f107: f64, f107a: f64, ap: f64) -> f64 {
    let kp = kp_from_ap(ap);
    T_INF0 + T_INF_C1 * f107a + T_INF_C2 * (f107 - f107a) + 28.0 * kp + 0.03 * kp.exp()
}

/// Total mass density (kg/m³) at geometric altitude `alt_m`, for daily F10.7,
/// 81-day mean F10.7A (sfu) and daily Ap. Valid 120–1000 km; below 120 km an
/// exponential with a 6 km scale height continues it to re-entry.
pub fn density(alt_m: f64, f107: f64, f107a: f64, ap: f64) -> f64 {
    let z_km = alt_m * 1e-3;
    if z_km < 120.0 {
        return density(120e3, f107, f107a, ap) * ((120.0 - z_km) / H_BELOW_120_KM).exp();
    }
    let fa = f107a.clamp(50.0, 300.0);
    let tinf = exospheric_temperature(f107.clamp(50.0, 400.0), fa, ap).max(500.0);
    let t120 = T120_A + T120_B * fa;
    let s = (S0 + S1 * (tinf - 1000.0) / 1000.0).max(0.005);
    let g120 = G0 * (RE_GEO_KM / (RE_GEO_KM + 120.0)).powi(2);
    let sigma = s + 1.0 / (RE_GEO_KM + 120.0); // 1/km
    let xi = (z_km - 120.0) * (RE_GEO_KM + 120.0) / (RE_GEO_KM + z_km);
    let t = tinf - (tinf - t120) * (-s * xi).exp();
    let ln_t_ratio = (t120 / t).ln();
    let mut rho = 0.0;
    for sp in &SPECIES {
        let m = sp.mass_u * AMU;
        let gamma = m * g120 / (sigma * 1e-3 * K_BOLTZMANN * tinf);
        let ln_n = sp.c0 + sp.c1 * fa + (1.0 + sp.alpha + gamma) * ln_t_ratio - sigma * gamma * xi;
        rho += m * ln_n.exp();
    }
    rho
}

/// Smoothed-F10.7 turning points (decimal year, sfu) of
/// `orbit_lifetime.py` `CYCLE_POINTS`: cycle 25 minimum and maximum
/// observed approximately, later values assumed.
const CYCLE_POINTS: [(f64, f64); 6] =
    [(2019.9, 69.0), (2024.8, 185.0), (2030.5, 68.0), (2035.0, 150.0), (2041.0, 68.0), (2045.5, 150.0)];

/// Relative amplitude of the 27-day rotational modulation of daily F10.7.
const ROTATION_AMPLITUDE: f64 = 0.08;
/// Synodic solar rotation period (s).
const ROTATION_PERIOD_S: f64 = 27.0 * DAY_S;

/// Smoothed (81-day) F10.7 at `t` (sfu), the half-cosine profile.
pub fn f107_smoothed(t: f64) -> f64 {
    let y = decimal_year(t);
    let first = CYCLE_POINTS[0];
    if y <= first.0 {
        return first.1;
    }
    for w in CYCLE_POINTS.windows(2) {
        let ((t0, v0), (t1, v1)) = (w[0], w[1]);
        if y <= t1 {
            let x = (y - t0) / (t1 - t0);
            return v0 + (v1 - v0) * 0.5 * (1.0 - (core::f64::consts::PI * x).cos());
        }
    }
    CYCLE_POINTS[CYCLE_POINTS.len() - 1].1
}

/// Solar-cycle driver: (daily F10.7, 81-day mean F10.7A) at `t` (sfu).
pub fn f107_profile(t: f64) -> (f64, f64) {
    let fa = f107_smoothed(t);
    let f = fa * (1.0 + ROTATION_AMPLITUDE * (TAU * t / ROTATION_PERIOD_S).sin());
    (f, fa)
}

/// Daily Ap paired with a given F10.7 (quiet to moderate; the
/// `ap_for_f107` assumption of `orbit_lifetime.py`).
pub fn ap_nominal(f107a: f64) -> f64 {
    4.0 + 0.08 * (f107a - 65.0).max(0.0)
}

/// Density along the nominal profile at time `t` and altitude `alt_m`.
pub fn density_nominal(alt_m: f64, t: f64) -> f64 {
    let (f, fa) = f107_profile(t);
    density(alt_m, f, fa, ap_nominal(fa))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::math::{R_EARTH, YEAR_S};
    use crate::env::orbit::Orbit;
    use crate::env::time::from_utc;

    /// Orbit-averaged NRLMSIS 2.1 values (kg/m³) at 500 km, from the fit
    /// data set: (F10.7, ρ).
    const MSIS_500: [(f64, f64); 4] = [(65.0, 6.819e-14), (100.0, 2.364e-13), (150.0, 7.563e-13), (220.0, 2.034e-12)];

    #[test]
    fn matches_nrlmsis_at_500_km() {
        for &(f, rho) in &MSIS_500 {
            let m = density(500e3, f, f, ap_nominal(f));
            assert!((m / rho - 1.0).abs() < 0.06, "F={f}: {m:e} vs {rho:e}");
        }
        // NRLMSIS at 120 km ≈ 1.6e-8 kg/m³
        let r120 = density(120e3, 100.0, 100.0, ap_nominal(100.0));
        assert!((r120 / 1.644e-8 - 1.0).abs() < 0.05);
    }

    #[test]
    fn monotonic_in_altitude_and_activity() {
        let mut last = f64::INFINITY;
        for k in 0..95 {
            let r = density(100e3 + 1e4 * f64::from(k), 120.0, 120.0, 10.0);
            assert!(r < last && r > 0.0);
            last = r;
        }
        assert!(density(450e3, 180.0, 180.0, 10.0) > density(450e3, 70.0, 70.0, 10.0) * 5.0);
        assert!(density(450e3, 120.0, 120.0, 50.0) > density(450e3, 120.0, 120.0, 5.0));
    }

    #[test]
    fn solar_cycle_profile_shape() {
        let (f, fa) = f107_profile(from_utc(2024, 10, 20, 0, 0, 0.0));
        assert!(fa > 180.0 && (f / fa - 1.0).abs() <= ROTATION_AMPLITUDE + 1e-12);
        let min = f107_smoothed(from_utc(2030, 7, 1, 0, 0, 0.0));
        assert!((min - 68.0).abs() < 1.0);
        let max26 = f107_smoothed(from_utc(2035, 1, 1, 0, 0, 0.0));
        assert!((max26 - 150.0).abs() < 1.0);
        let at_launch = f107_smoothed(from_utc(2028, 6, 1, 0, 0, 0.0));
        assert!(at_launch > 90.0 && at_launch < 130.0, "{at_launch}");
    }

    /// Lifetime (years) from a circular orbit at `alt_km` deployed
    /// 2028-06-01, 1-day steps, nominal solar-cycle profile.
    fn lifetime_years(alt_km: f64, mass: f64, area: f64, cd: f64) -> f64 {
        let t0 = from_utc(2028, 6, 1, 0, 0, 0.0);
        let mut o = Orbit::sso(alt_km * 1e3, 10.5, t0);
        let mut t = t0;
        while !o.reentered() && t - t0 < 30.0 * YEAR_S {
            let rho = density_nominal(o.a - R_EARTH, t);
            o.apply_drag(DAY_S, rho, area, mass, cd);
            t += DAY_S;
        }
        (t - t0) / YEAR_S
    }

    /// Calibration against BASELINE §3 (NRLMSIS 2.1, 1.6 kg, Cd 2.2,
    /// A = 0.028 m², deployed 2028-06-01): 500 km → 5.1 yr, 450 km → 2.3 yr.
    #[test]
    fn lifetime_calibration() {
        let l500 = lifetime_years(500.0, 1.6, 0.028, 2.2);
        let l450 = lifetime_years(450.0, 1.6, 0.028, 2.2);
        println!("lifetime 500 km: {l500:.2} yr, 450 km: {l450:.2} yr");
        assert!((4.0..=6.5).contains(&l500), "{l500}");
        assert!((1.5..=3.5).contains(&l450), "{l450}");
        // wings failed (A ≈ 0.0164 m²): BASELINE 6.2 yr at 500 km
        let l500_failed = lifetime_years(500.0, 1.6, 0.0164, 2.2);
        println!("lifetime 500 km, wings failed: {l500_failed:.2} yr");
        assert!((5.0..=8.0).contains(&l500_failed), "{l500_failed}");
    }
}
