//! Earth geometry shared by the models: ECI ↔ ECEF (rotation by GMST only;
//! polar motion and nutation ignored), WGS84 geodetic coordinates, and the
//! Earth's albedo and infrared loads on a flat plate.

use core::f64::consts::{FRAC_PI_2, PI};

use super::math::{F_EARTH, R_EARTH, V3};
use super::time::gmst;

/// Mean Bond albedo of the Earth (`hardware/structure/THERMAL_DESIGN.md`
/// §2.2).
pub const ALBEDO: f64 = 0.30;
/// Orbit-average outgoing long-wave radiation at the top of the atmosphere
/// (W/m², THERMAL_DESIGN §2.2).
pub const EARTH_IR_W_M2: f64 = 237.0;
/// Mean Earth radius for radiation view factors (m).
pub const R_EARTH_MEAN: f64 = 6_371_000.0;

/// ECI → ECEF position at `t` (rotation about Z by −GMST).
pub fn eci_to_ecef(r_eci: V3, t: f64) -> V3 {
    r_eci.rot_z(-gmst(t))
}

/// ECEF → ECI position at `t`.
pub fn ecef_to_eci(r_ecef: V3, t: f64) -> V3 {
    r_ecef.rot_z(gmst(t))
}

/// WGS84 geodetic latitude, longitude (rad, longitude in −π..π) and height
/// (m) of an ECEF position (fixed-point iteration on the latitude, four
/// passes: < 1 mm in LEO).
pub fn ecef_to_geodetic(r: V3) -> (f64, f64, f64) {
    let e2 = F_EARTH * (2.0 - F_EARTH);
    let lon = r.y.atan2(r.x);
    let p = (r.x * r.x + r.y * r.y).sqrt();
    if p < 1.0 {
        let lat = if r.z >= 0.0 { FRAC_PI_2 } else { -FRAC_PI_2 };
        return (lat, lon, r.z.abs() - R_EARTH * (1.0 - F_EARTH));
    }
    let mut lat = (r.z / (p * (1.0 - e2))).atan();
    for _ in 0..4 {
        let s = lat.sin();
        let n = R_EARTH / (1.0 - e2 * s * s).sqrt();
        lat = (r.z + e2 * n * s).atan2(p);
    }
    let (s, c) = lat.sin_cos();
    let n = R_EARTH / (1.0 - e2 * s * s).sqrt();
    // well conditioned at all latitudes
    let h = p * c + (r.z + e2 * n * s) * s - n;
    (lat, lon, h)
}

/// ECEF position of a WGS84 geodetic point (lat, lon in rad, height in m).
pub fn geodetic_to_ecef(lat: f64, lon: f64, h: f64) -> V3 {
    let e2 = F_EARTH * (2.0 - F_EARTH);
    let (sl, cl) = lat.sin_cos();
    let n = R_EARTH / (1.0 - e2 * sl * sl).sqrt();
    let (so, co) = lon.sin_cos();
    V3::new((n + h) * cl * co, (n + h) * cl * so, (n * (1.0 - e2) + h) * sl)
}

/// Sub-satellite point: geodetic latitude, longitude (rad) and height (m) of
/// an ECI position at `t`.
pub fn subpoint(r_eci: V3, t: f64) -> (f64, f64, f64) {
    ecef_to_geodetic(eci_to_ecef(r_eci, t))
}

/// View factor from a flat one-sided plate to a spherical Earth.
///
/// `cos_nadir` is the cosine of the angle between the plate normal and the
/// nadir direction, `r_m` the distance of the plate from the Earth's centre.
/// Fully-visible case `cos θ / H²`; partly-visible case from the standard
/// plate-to-sphere configuration factor (Howell, *A Catalog of Radiation Heat
/// Transfer Configuration Factors*, plate tilted towards a sphere; Gilmore, *Spacecraft Thermal
/// Control Handbook*, ch. 2).
pub fn plate_view_factor(cos_nadir: f64, r_m: f64) -> f64 {
    let h = (r_m / R_EARTH_MEAN).max(1.000_001);
    let theta = cos_nadir.clamp(-1.0, 1.0).acos();
    let phi_m = (1.0 / h).asin();
    if theta <= FRAC_PI_2 - phi_m {
        return cos_nadir / (h * h);
    }
    if theta >= FRAC_PI_2 + phi_m {
        return 0.0;
    }
    let x = (h * h - 1.0).sqrt();
    let st = theta.sin();
    let y = (-x * theta.cos() / st).clamp(-1.0, 1.0);
    let sq = (1.0 - y * y).max(0.0).sqrt();
    let f = (cos_nadir * y.acos() - x * st * sq) / (PI * h * h) + (st * sq / x).atan() / PI;
    f.max(0.0)
}

/// Albedo irradiance (W/m²) on a plate facing nadir, at distance `r_m` from
/// the Earth's centre, with the Sun at zenith angle ζ over the sub-satellite
/// point (`cos_zeta = r̂ · ŝ`) and solar flux `flux`.
///
/// Approximation: albedo × flux × F × max(0, cos ζ) with F the nadir-plate
/// view factor (the variation of illumination across the visible cap and the
/// glow past the terminator are neglected).
pub fn albedo_nadir_w_m2(r_m: f64, cos_zeta: f64, flux: f64) -> f64 {
    ALBEDO * flux * plate_view_factor(1.0, r_m) * cos_zeta.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geodetic_round_trip() {
        for &(lat, lon, h) in &[(37.35, -121.94, 30.0), (-33.9, 18.6, 10.0), (89.9, 10.0, 500e3), (0.0, 179.9, 500e3)] {
            let r = geodetic_to_ecef(f64::to_radians(lat), f64::to_radians(lon), h);
            let (la, lo, hh) = ecef_to_geodetic(r);
            assert!((la.to_degrees() - lat).abs() < 1e-8);
            assert!((lo.to_degrees() - lon).abs() < 1e-8);
            assert!((hh - h).abs() < 1e-3);
        }
    }

    #[test]
    fn eci_ecef_inverse() {
        let r = V3::new(7.0e6, -1.0e6, 2.0e6);
        let back = ecef_to_eci(eci_to_ecef(r, 1.0e8), 1.0e8);
        assert!((back - r).norm() < 1e-6);
    }

    #[test]
    fn view_factor_limits_and_continuity() {
        let r = R_EARTH_MEAN + 500e3;
        let h = r / R_EARTH_MEAN;
        assert!((plate_view_factor(1.0, r) - 1.0 / (h * h)).abs() < 1e-12);
        assert_eq!(plate_view_factor(-1.0, r), 0.0);
        // edge-on plate at 500 km still sees part of the Earth
        let edge = plate_view_factor(0.0, r);
        assert!(edge > 0.2 && edge < 0.35, "{edge}");
        // continuous across the fully-visible boundary
        let phi_m = (1.0 / h).asin();
        let c = (FRAC_PI_2 - phi_m).cos();
        assert!((plate_view_factor(c + 1e-9, r) - plate_view_factor(c - 1e-9, r)).abs() < 1e-6);
        // monotonic in the angle
        let mut last = 1.0;
        for i in 0..=180 {
            let f = plate_view_factor(f64::to_radians(f64::from(i)).cos(), r);
            assert!(f <= last + 1e-12);
            last = f;
        }
    }

    #[test]
    fn albedo_magnitude() {
        let a = albedo_nadir_w_m2(R_EARTH_MEAN + 500e3, 1.0, 1361.0);
        assert!(a > 330.0 && a < 370.0, "{a}");
        assert_eq!(albedo_nadir_w_m2(R_EARTH_MEAN + 500e3, -0.5, 1361.0), 0.0);
    }
}
