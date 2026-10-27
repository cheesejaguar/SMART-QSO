//! Sun position: the low-precision solar coordinates of the *Astronomical
//! Almanac* (section C, "Low precision formulas for the Sun"), stated there as
//! accurate to 0.01° between 1950 and 2050.
//!
//! The direction is in the mean equator and equinox of date, which the
//! simulation uses as its ECI frame (the < 0.4° precession from J2000 over the
//! mission is ignored consistently with [`super::time::gmst`]).

use super::math::{AU, SOLAR_CONSTANT, V3};
use super::time::{JD_J2000, jd};

/// Unit vector from the Earth to the Sun in ECI, and the Earth–Sun distance
/// (m).
pub fn sun_eci(t: f64) -> (V3, f64) {
    let n = jd(t) - JD_J2000;
    let l = (280.460 + 0.985_647_4 * n).to_radians();
    let g = (357.528 + 0.985_600_3 * n).to_radians();
    let lambda = l + (1.915f64.to_radians()) * g.sin() + (0.020f64.to_radians()) * (2.0 * g).sin();
    let eps = (23.439 - 0.000_000_4 * n).to_radians();
    let r_au = 1.000_14 - 0.016_71 * g.cos() - 0.000_14 * (2.0 * g).cos();
    let (sl, cl) = lambda.sin_cos();
    let (se, ce) = eps.sin_cos();
    (V3::new(cl, ce * sl, se * sl), r_au * AU)
}

/// Solar irradiance (W/m²) at a distance from the Sun: 1361 W/m² × (1 AU/d)².
pub fn solar_flux(distance_m: f64) -> f64 {
    let k = AU / distance_m;
    SOLAR_CONSTANT * k * k
}

/// Right ascension of the mean Sun (rad, 0..2π): the Sun's mean longitude,
/// which defines mean solar time and so the local time of an orbit's node.
pub fn mean_sun_ra(t: f64) -> f64 {
    let n = jd(t) - JD_J2000;
    (280.460 + 0.985_647_4 * n).to_radians().rem_euclid(core::f64::consts::TAU)
}

/// Right ascension of the Sun (rad, 0..2π) at `t`.
pub fn sun_ra(t: f64) -> f64 {
    let (s, _) = sun_eci(t);
    s.y.atan2(s.x).rem_euclid(core::f64::consts::TAU)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::time::from_utc;

    #[test]
    fn equinox_and_solstice() {
        // March equinox 2028: 2028-03-20 02:17 UTC → declination ≈ 0
        let (s, d) = sun_eci(from_utc(2028, 3, 20, 2, 17, 0.0));
        assert!(s.z.asin().to_degrees().abs() < 0.02, "{}", s.z.asin().to_degrees());
        assert!((d / AU - 0.996).abs() < 0.003);
        // June solstice 2028-06-20 20:02 UTC → declination 23.43°
        let (s, _) = sun_eci(from_utc(2028, 6, 20, 20, 2, 0.0));
        assert!((s.z.asin().to_degrees() - 23.436).abs() < 0.02);
        assert!((s.norm() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn vallado_example() {
        // Vallado example 5-1 (same algorithm): 2006-04-02 00:00 UT1,
        // r = (0.977_194_5, 0.192_442_4, 0.083_430_8) AU
        let (s, d) = sun_eci(from_utc(2006, 4, 2, 0, 0, 0.0));
        let r = s * (d / AU);
        assert!((r.x - 0.977_194_5).abs() < 2e-5);
        assert!((r.y - 0.192_442_4).abs() < 2e-5);
        assert!((r.z - 0.083_430_8).abs() < 2e-5);
    }

    #[test]
    fn flux_at_perihelion_and_aphelion() {
        assert!((solar_flux(AU) - 1361.0).abs() < 1e-9);
        let (_, dp) = sun_eci(from_utc(2029, 1, 3, 0, 0, 0.0));
        let (_, da) = sun_eci(from_utc(2029, 7, 4, 0, 0, 0.0));
        assert!((solar_flux(dp) / 1361.0 - 1.034).abs() < 0.003);
        assert!((solar_flux(da) / 1361.0 - 0.967).abs() < 0.003);
    }
}
