//! Geomagnetic field: the tilted dipole of IGRF-14 (epoch 2025.0) with its
//! secular variation.
//!
//! Gauss coefficients (nT) from the IGRF-14 model (IAGA Working Group V-MOD,
//! 2024): g₁⁰ = −29350.0, g₁¹ = −1410.3, h₁¹ = 4545.5, secular variation
//! 2025–2030 +12.6, +10.0, −21.5 nT/yr, extrapolated linearly. For a dipole
//! the potential V = a³ (g · r)/r³ with g = (g₁¹, h₁¹, g₁⁰) in ECEF, so
//! B = (a/r)³ [3 (g · r̂) r̂ − g]. The dipole is ≈ 90 % of the field in LEO;
//! the higher harmonics (South Atlantic Anomaly low, ±10 µT) are omitted.
//!
//! The dipole is rotated to ECI with GMST (ECEF = ECI rotated by −GMST).

use super::math::V3;
use super::time::{decimal_year, gmst};

/// IGRF reference radius (m).
pub const A_IGRF: f64 = 6_371_200.0;
/// Coefficient epoch (decimal year).
pub const EPOCH: f64 = 2025.0;
/// g₁⁰, g₁¹, h₁¹ at [`EPOCH`] (T).
pub const G10: f64 = -29_350.0e-9;
/// g₁¹ at [`EPOCH`] (T).
pub const G11: f64 = -1_410.3e-9;
/// h₁¹ at [`EPOCH`] (T).
pub const H11: f64 = 4_545.5e-9;
/// Secular variation of g₁⁰ (T/yr).
pub const DG10: f64 = 12.6e-9;
/// Secular variation of g₁¹ (T/yr).
pub const DG11: f64 = 10.0e-9;
/// Secular variation of h₁¹ (T/yr).
pub const DH11: f64 = -21.5e-9;

/// Dipole coefficient vector g = (g₁¹, h₁¹, g₁⁰) in ECEF at `t` (T).
pub fn dipole_ecef(t: f64) -> V3 {
    let dy = decimal_year(t) - EPOCH;
    V3::new(G11 + DG11 * dy, H11 + DH11 * dy, G10 + DG10 * dy)
}

/// Unit vector to the geomagnetic north pole (boreal dipole pole) in ECEF:
/// −g / |g| (≈ 80.8° N, 72.7° W in 2025).
pub fn north_pole_ecef(t: f64) -> V3 {
    (-dipole_ecef(t)).unit()
}

/// Dipole field (T) at `r` (m) for the coefficient vector `g` in the same
/// frame.
fn dipole_field(r: V3, g: V3) -> V3 {
    let rn = r.norm();
    let rh = r / rn;
    let k = (A_IGRF / rn).powi(3);
    (rh * (3.0 * g.dot(rh)) - g) * k
}

/// Geomagnetic field (T) in ECI at the ECI position `r_eci` (m) at `t`.
pub fn field_eci(r_eci: V3, t: f64) -> V3 {
    let g_eci = dipole_ecef(t).rot_z(gmst(t));
    dipole_field(r_eci, g_eci)
}

/// Geomagnetic field (T) in ECEF at the ECEF position `r_ecef` (m) at `t`.
pub fn field_ecef(r_ecef: V3, t: f64) -> V3 {
    dipole_field(r_ecef, dipole_ecef(t))
}

/// Geomagnetic (dipole) latitude (rad) of an ECEF position at `t`.
pub fn magnetic_latitude(r_ecef: V3, t: f64) -> f64 {
    r_ecef.unit().dot(north_pole_ecef(t)).clamp(-1.0, 1.0).asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::earth::{eci_to_ecef, geodetic_to_ecef};
    use crate::env::orbit::Orbit;
    use crate::env::time::from_utc;

    #[test]
    fn magnitude_in_leo() {
        let t0 = from_utc(2028, 6, 1, 0, 0, 0.0);
        let o = Orbit::sso(500e3, 10.5, t0);
        let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
        for k in 0..600 {
            let t = t0 + 10.0 * f64::from(k);
            let (r, _) = o.state_at(t);
            let b = field_eci(r, t).norm();
            lo = lo.min(b);
            hi = hi.max(b);
        }
        println!("|B| at 500 km: {:.1}–{:.1} µT", lo * 1e6, hi * 1e6);
        assert!(lo > 20e-6 && hi < 50e-6, "{lo} {hi}");
    }

    #[test]
    fn pole_and_direction() {
        let t = from_utc(2025, 1, 1, 0, 0, 0.0);
        let p = north_pole_ecef(t);
        let lat = p.z.asin().to_degrees();
        let lon = p.y.atan2(p.x).to_degrees();
        assert!((lat - 80.8).abs() < 0.3 && (lon + 72.7).abs() < 0.5, "{lat} {lon}");
        // at the geographic equator the field points north (+Z), ~ −g10
        let r = geodetic_to_ecef(0.0, 0.0, 0.0);
        let b = field_ecef(r, t);
        assert!(b.z > 25e-6 && b.z < 32e-6);
        // ECI and ECEF agree after rotation
        let r_eci = V3::new(5e6, 4e6, 2e6);
        let b_eci = field_eci(r_eci, t);
        let b_ecef = field_ecef(eci_to_ecef(r_eci, t), t);
        assert!((eci_to_ecef(b_eci, t) - b_ecef).norm() < 1e-15);
        assert!(magnetic_latitude(p * 7e6, t) > 1.5);
    }
}
