//! Earth shadow: conical umbra and penumbra with the Sun and Earth as discs.
//!
//! The illuminated fraction is the visible part of the solar disc seen from
//! the spacecraft, from the apparent radii of the Sun and the Earth and their
//! angular separation (Montenbruck and Gill, *Satellite Orbits*, §3.4.2). The
//! Earth is a sphere of the equatorial radius plus a 20 km absorbing layer
//! (assumption, standing for refraction and absorption in the lower
//! atmosphere); oblateness is ignored.
//!
//! Check (`hardware/eps/SOLAR_ARRAY_DESIGN.md` §9.2): at 500 km with β = 0
//! the eclipse is 35.8 min of a 94.6 min orbit.

use core::f64::consts::PI;

use super::math::{R_EARTH, R_SUN, V3};

/// Radius of the occulting Earth (m): equatorial radius + 20 km of
/// atmosphere.
pub const R_SHADOW: f64 = R_EARTH + 20e3;

/// Fraction of the solar disc visible from `r_sat_eci` (m), 0 (umbra) to 1
/// (full Sun), for the unit Sun direction `sun_unit` and Earth–Sun distance
/// `sun_dist` (m).
pub fn illumination(r_sat_eci: V3, sun_unit: V3, sun_dist: f64) -> f64 {
    // quick exit: on the day side of the terminator plane
    if r_sat_eci.dot(sun_unit) >= 0.0 {
        return 1.0;
    }
    let to_sun = sun_unit * sun_dist - r_sat_eci;
    let d_sun = to_sun.norm();
    let d_earth = r_sat_eci.norm();
    let a = (R_SUN / d_sun).asin(); // apparent solar radius
    let b = (R_SHADOW / d_earth).min(1.0).asin(); // apparent Earth radius
    let c = (-r_sat_eci).angle(to_sun); // separation of the centres
    if c >= a + b {
        1.0
    } else if c <= b - a {
        0.0
    } else if c <= a - b {
        // annular (not reachable in LEO; kept for completeness)
        1.0 - (b * b) / (a * a)
    } else {
        // partial overlap of two discs (small-angle planar approximation)
        let x = (c * c + a * a - b * b) / (2.0 * c);
        let y = (a * a - x * x).max(0.0).sqrt();
        let area = a * a * (x / a).clamp(-1.0, 1.0).acos() + b * b * ((c - x) / b).clamp(-1.0, 1.0).acos() - c * y;
        (1.0 - area / (PI * a * a)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::orbit::Orbit;
    use crate::env::sun::sun_eci;
    use crate::env::time::from_utc;

    #[test]
    fn eclipse_duration_at_500_km_beta_zero() {
        // a circular orbit containing the Sun direction (β = 0)
        let t0 = from_utc(2028, 3, 20, 0, 0, 0.0);
        let (s, d) = sun_eci(t0);
        // node at 90° from the Sun in the plane containing the Sun: choose the
        // orbit normal perpendicular to s
        let n = s.cross(V3::Z).unit();
        let raan = (V3::Z.cross(n)).y.atan2((V3::Z.cross(n)).x);
        let inc = n.z.clamp(-1.0, 1.0).acos();
        let o = Orbit::circular(500e3, inc, raan, t0);
        assert!(o.beta(s).abs() < 1e-6);
        let p = o.period();
        let (mut umbra, mut dark_half, mut pen) = (0.0, 0.0, 0.0);
        let dt = 1.0;
        let mut t = 0.0;
        while t < p {
            let (r, _) = o.state_at(t0 + t);
            let f = illumination(r, s, d);
            if f == 0.0 {
                umbra += dt;
            }
            if f < 0.5 {
                dark_half += dt;
            }
            if f > 0.0 && f < 1.0 {
                pen += dt;
            }
            t += dt;
        }
        println!(
            "eclipse: umbra {:.2} min, <50 % {:.2} min, penumbra {:.1} s, period {:.2} min",
            umbra / 60.0,
            dark_half / 60.0,
            pen,
            p / 60.0
        );
        assert!((dark_half / 60.0 - 35.8).abs() < 0.8, "{}", dark_half / 60.0);
        assert!(pen > 10.0 && pen < 40.0);
        assert!((p / 60.0 - 94.6).abs() < 0.1);
    }

    #[test]
    fn day_side_and_deep_shadow() {
        let s = V3::X;
        assert_eq!(illumination(V3::new(7e6, 0.0, 0.0), s, 1.5e11), 1.0);
        assert_eq!(illumination(V3::new(-7e6, 0.0, 0.0), s, 1.5e11), 0.0);
        // high above the pole on the night side: lit
        assert_eq!(illumination(V3::new(-1e6, 0.0, 7e6), s, 1.5e11), 1.0);
    }
}
