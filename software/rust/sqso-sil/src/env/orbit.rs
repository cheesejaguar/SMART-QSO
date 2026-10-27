//! Mean-element orbit with J2 secular drift and drag decay.
//!
//! The orbit is a Keplerian ellipse whose node, perigee and mean anomaly
//! drift at the first-order J2 secular rates (Vallado, *Fundamentals of
//! Astrodynamics*, eqs. 9-41): short-period J2 terms (a few km) are not
//! modelled. Drag shrinks the semi-major axis of the near-circular orbit at
//! `da/dt = −ρ (Cd A / m) √(μ a)`, the same equation as
//! `software/simulation/orbit_lifetime/orbit_lifetime.py`.
//!
//! The angles are **integrated** ([`Orbit::propagate`]): each call advances
//! them from the stored `epoch` with the rates of the current `a`, so the
//! mean anomaly stays continuous as drag changes the mean motion.
//!
//! Baseline orbit (`docs/BASELINE_PARAMETERS.md` §3): circular SSO,
//! inclination ≈ 97.4°, deployment ≤ 500 km (525 km cap), 2028.

use core::f64::consts::{PI, TAU};

use super::math::{DAY_S, J2, MU_EARTH, R_EARTH, V3};
use super::sun::mean_sun_ra;

/// Altitude below which the spacecraft is considered re-entered (m). Below
/// about 120 km the remaining life is minutes.
pub const REENTRY_ALT_M: f64 = 120e3;

/// Required nodal precession of a sun-synchronous orbit (rad/s): one turn per
/// tropical year.
pub const SSO_RAAN_RATE: f64 = TAU / (365.242_19 * DAY_S);

/// Mean orbital elements (SI, angles in rad) valid at `epoch` (s since
/// J2000).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Orbit {
    /// Semi-major axis (m).
    pub a: f64,
    /// Eccentricity.
    pub e: f64,
    /// Inclination (rad).
    pub i: f64,
    /// Right ascension of the ascending node (rad).
    pub raan: f64,
    /// Argument of perigee (rad).
    pub argp: f64,
    /// Mean anomaly (rad).
    pub m: f64,
    /// Time at which the angles are valid (s since J2000).
    pub epoch: f64,
}

/// J2 secular rates (rad/s) of the node, perigee and mean anomaly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rates {
    /// dΩ/dt.
    pub raan: f64,
    /// dω/dt.
    pub argp: f64,
    /// dM/dt (mean motion including the J2 correction).
    pub m: f64,
}

impl Orbit {
    /// Circular sun-synchronous orbit at altitude `alt_m` above the
    /// equatorial radius, with the local time of the ascending node
    /// `ltan_hours` (mean solar time) at `t`, and the spacecraft at the
    /// ascending node at `t`.
    ///
    /// The inclination solves the J2 condition dΩ/dt = 2π / tropical year:
    /// cos i = −Ω̇ / (1.5 n J2 (R/a)²) (≈ 97.4° at 500 km, BASELINE §3).
    pub fn sso(alt_m: f64, ltan_hours: f64, t: f64) -> Orbit {
        let a = R_EARTH + alt_m;
        let n = (MU_EARTH / (a * a * a)).sqrt();
        let k = 1.5 * n * J2 * (R_EARTH / a).powi(2);
        let cos_i = (-SSO_RAAN_RATE / k).clamp(-1.0, 1.0);
        let raan = (mean_sun_ra(t) + (ltan_hours - 12.0) * (PI / 12.0)).rem_euclid(TAU);
        Orbit { a, e: 0.0, i: cos_i.acos(), raan, argp: 0.0, m: 0.0, epoch: t }
    }

    /// Circular orbit with the given altitude, inclination and node, at the
    /// ascending node at `t` (for example the ISS backup: 400–420 km, 51.6°).
    pub fn circular(alt_m: f64, inc_rad: f64, raan_rad: f64, t: f64) -> Orbit {
        Orbit { a: R_EARTH + alt_m, e: 0.0, i: inc_rad, raan: raan_rad, argp: 0.0, m: 0.0, epoch: t }
    }

    /// Keplerian mean motion √(μ/a³) (rad/s).
    pub fn mean_motion(&self) -> f64 {
        (MU_EARTH / (self.a * self.a * self.a)).sqrt()
    }

    /// Keplerian period 2π √(a³/μ) (s): 94.6 min at 500 km.
    pub fn period(&self) -> f64 {
        TAU / self.mean_motion()
    }

    /// Nodal period, 2π / (dM/dt + dω/dt) (s), the time between ascending
    /// node crossings.
    pub fn nodal_period(&self) -> f64 {
        let r = self.rates();
        TAU / (r.m + r.argp)
    }

    /// J2 secular rates for the current elements.
    pub fn rates(&self) -> Rates {
        let n = self.mean_motion();
        let one_e2 = 1.0 - self.e * self.e;
        let p = self.a * one_e2;
        let k = J2 * (R_EARTH / p).powi(2);
        let c = self.i.cos();
        let c2 = c * c;
        Rates {
            raan: -1.5 * n * k * c,
            argp: 0.75 * n * k * (5.0 * c2 - 1.0),
            m: n * (1.0 + 0.75 * k * one_e2.sqrt() * (3.0 * c2 - 1.0)),
        }
    }

    /// Advance the angles from `epoch` to `t` with the current rates and set
    /// `epoch = t` (also works backwards).
    pub fn propagate(&mut self, t: f64) {
        let dt = t - self.epoch;
        if dt == 0.0 {
            return;
        }
        let r = self.rates();
        self.raan = (self.raan + r.raan * dt).rem_euclid(TAU);
        self.argp = (self.argp + r.argp * dt).rem_euclid(TAU);
        self.m = (self.m + r.m * dt).rem_euclid(TAU);
        self.epoch = t;
    }

    /// ECI position (m) and velocity (m/s) at `t`, extrapolated from `epoch`
    /// with the current rates, without changing the orbit.
    pub fn state_at(&self, t: f64) -> (V3, V3) {
        let mut o = *self;
        o.propagate(t);
        o.state()
    }

    /// ECI position (m) and velocity (m/s) at `epoch`.
    pub fn state(&self) -> (V3, V3) {
        let e = self.e;
        let ea = kepler(self.m, e);
        let (se, ce) = ea.sin_cos();
        let b = (1.0 - e * e).sqrt();
        let r_pqw = V3::new(self.a * (ce - e), self.a * b * se, 0.0);
        let rt = self.rates();
        // v = ∂r/∂M Ṁ + ∂r/∂ω ω̇ + ∂r/∂Ω Ω̇: the time derivative of the
        // drifting ellipse, so that dr/dt matches v
        let k = rt.m * self.a / (1.0 - e * ce);
        let v_pqw = V3::new(-k * se, k * b * ce, 0.0) + V3::Z.cross(r_pqw) * rt.argp;
        let to_eci = |v: V3| v.rot_z(self.argp).rot_x(self.i).rot_z(self.raan);
        let r = to_eci(r_pqw);
        (r, to_eci(v_pqw) + V3::Z.cross(r) * rt.raan)
    }

    /// Unit orbit normal (angular momentum direction) in ECI.
    pub fn normal(&self) -> V3 {
        V3::Z.rot_x(self.i).rot_z(self.raan)
    }

    /// Solar beta angle (rad): elevation of the Sun above the orbit plane,
    /// for the unit Sun vector `sun_eci`.
    pub fn beta(&self, sun_eci: V3) -> f64 {
        self.normal().dot(sun_eci).clamp(-1.0, 1.0).asin()
    }

    /// Local time of the ascending node (hours, mean solar time) at `t`.
    pub fn ltan_hours(&self, t: f64) -> f64 {
        let mut o = *self;
        o.propagate(t);
        (12.0 + (o.raan - mean_sun_ra(t)) * (12.0 / PI)).rem_euclid(24.0)
    }

    /// Height of a position above the equatorial radius (m; spherical Earth,
    /// as the lifetime analysis uses).
    pub fn altitude_m(r: V3) -> f64 {
        r.norm() - R_EARTH
    }

    /// Mean altitude of the orbit, a − R_E (m).
    pub fn mean_altitude_m(&self) -> f64 {
        self.a - R_EARTH
    }

    /// Perigee altitude a(1 − e) − R_E (m).
    pub fn perigee_altitude_m(&self) -> f64 {
        self.a * (1.0 - self.e) - R_EARTH
    }

    /// Secular drag decay of a near-circular orbit over `dt` seconds:
    /// `a ← a − ρ (Cd A / m) √(μ a) dt` (density in kg/m³, area in m²,
    /// mass in kg). The angles are untouched: call [`Orbit::propagate`] to
    /// advance them.
    pub fn apply_drag(&mut self, dt: f64, density: f64, area_m2: f64, mass_kg: f64, cd: f64) {
        let rate = density * (cd * area_m2 / mass_kg) * (MU_EARTH * self.a).sqrt();
        self.a = (self.a - rate * dt).max(R_EARTH);
    }

    /// True once the perigee is below [`REENTRY_ALT_M`].
    pub fn reentered(&self) -> bool {
        self.perigee_altitude_m() < REENTRY_ALT_M
    }
}

/// Solve Kepler's equation E − e sin E = M by Newton's method.
pub fn kepler(m: f64, e: f64) -> f64 {
    if e == 0.0 {
        return m;
    }
    let mut ea = if e < 0.8 { m } else { PI };
    for _ in 0..12 {
        let f = ea - e * ea.sin() - m;
        let d = f / (1.0 - e * ea.cos());
        ea -= d;
        if d.abs() < 1e-13 {
            break;
        }
    }
    ea
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::time::from_utc;

    fn t0() -> f64 {
        from_utc(2028, 6, 1, 0, 0, 0.0)
    }

    #[test]
    fn sso_at_500_km() {
        let o = Orbit::sso(500e3, 10.5, t0());
        assert!((o.i.to_degrees() - 97.4).abs() < 0.1, "{}", o.i.to_degrees());
        assert!((o.period() / 60.0 - 94.6).abs() < 0.1);
        assert!((o.rates().raan - SSO_RAAN_RATE).abs() < 1e-15);
        assert!((o.ltan_hours(t0()) - 10.5).abs() < 1e-9);
    }

    #[test]
    fn local_time_of_node_holds_for_a_year() {
        let mut o = Orbit::sso(500e3, 22.5, t0());
        let mut t = t0();
        for _ in 0..365 {
            t += DAY_S;
            o.propagate(t);
        }
        assert!((o.ltan_hours(t) - 22.5).abs() < 0.02, "{}", o.ltan_hours(t));
    }

    #[test]
    fn state_is_a_consistent_kepler_orbit() {
        let mut o = Orbit::sso(500e3, 6.0, t0());
        o.e = 0.01;
        o.argp = 1.0;
        for k in 0..20 {
            let mut ok = o;
            ok.propagate(t0() + 300.0 * f64::from(k));
            let (r, v) = ok.state();
            let energy = 0.5 * v.norm2() - MU_EARTH / r.norm();
            // mean elements: the J2 rates change the speed by ~0.1 %
            assert!((energy + MU_EARTH / (2.0 * o.a)).abs() < 5e-3 * MU_EARTH / o.a);
            // angular momentum along the orbit normal (to the nodal drift)
            assert!(r.cross(v).unit().cross(ok.normal()).norm() < 1e-3);
        }
        // at the ascending node of a circular orbit, z = 0 and vz > 0
        let c = Orbit::sso(500e3, 6.0, t0());
        let (r, v) = c.state();
        assert!(r.z.abs() < 1e-6 && v.z > 0.0);
        assert!((v.norm() - 7610.0).abs() < 15.0, "{}", v.norm());
    }

    #[test]
    fn mean_anomaly_is_continuous_under_drag() {
        let mut o = Orbit::sso(500e3, 6.0, t0());
        let mut t = t0();
        let (r0, _) = o.state();
        for _ in 0..10 {
            t += 1.0;
            o.apply_drag(1.0, 1e-9, 0.028, 1.6, 2.2); // exaggerated density
            o.propagate(t);
        }
        let (r1, v1) = o.state();
        // ten seconds of flight at the stated velocity, not a jump
        assert!(((r1 - r0).norm() - 10.0 * v1.norm()).abs() < 5.0, "{}", (r1 - r0).norm() - 10.0 * v1.norm());
        assert!(o.a < R_EARTH + 500e3);
    }

    #[test]
    fn reentry_flag() {
        let mut o = Orbit::sso(500e3, 6.0, t0());
        assert!(!o.reentered());
        o.a = R_EARTH + 119e3;
        assert!(o.reentered());
        assert!((Orbit::altitude_m(V3::new(R_EARTH + 1000.0, 0.0, 0.0)) - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn kepler_solution() {
        for &e in &[0.0, 0.001, 0.1, 0.7] {
            for k in 0..12 {
                let m = f64::from(k) * 0.5;
                let ea = kepler(m, e);
                assert!((ea - e * ea.sin() - m).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn beta_of_dawn_dusk_orbit_is_large() {
        let o = Orbit::sso(500e3, 6.0, from_utc(2028, 6, 21, 0, 0, 0.0));
        let (s, _) = crate::env::sun::sun_eci(from_utc(2028, 6, 21, 0, 0, 0.0));
        assert!(o.beta(s).to_degrees().abs() > 60.0);
        let noon = Orbit::sso(500e3, 12.0, from_utc(2028, 3, 20, 0, 0, 0.0));
        let (s, _) = crate::env::sun::sun_eci(from_utc(2028, 3, 20, 0, 0, 0.0));
        assert!(noon.beta(s).to_degrees().abs() < 3.0);
    }
}
