//! Single-event effects and total ionizing dose.
//!
//! Three Poisson processes whose rate depends on where the spacecraft is:
//!
//! | Region | Test | Rate weight |
//! |--------|------|-------------|
//! | South Atlantic Anomaly | ((lat + 26°)/18°)² + ((lon + 50°)/40°)² < 1 (500 km contour of the > 10 MeV proton belt edge, approximate) | 30 |
//! | Polar | dipole magnetic latitude > 55° (horns of the outer belt, solar particles) | 3 |
//! | Elsewhere | — | 1 |
//!
//! Inside the SAA the weight scales with altitude as e^((h − 500 km)/150 km)
//! (trapped-proton flux grows with height). At 97.4°, 500 km the spacecraft
//! spends ≈ 3.5 % of the time in the SAA box and ≈ 38 % at high magnetic
//! latitude, so the mean weight is ≈ 2.8 (checked by the unit test over a
//! year); the base rates below are set so the orbit-average rates are
//! (**assumptions**, no part-level test data):
//!
//! * OBC (STM32L4R5, COTS 40 nm) single-event functional interrupt or upset
//!   needing a reset: one per 90 days.
//! * Jetson Orin Nano single-event latch-up (over-current event the OBC rail
//!   switch must clear): one per 60 days **of powered time**; the caller
//!   ignores events while the module is off. No heavy-ion data for Orin is
//!   published; BASELINE §6 treats the Jetson as non-critical and expendable.
//! * FRAM bit flip (FRAM cells are SEU-hard; this stands for upsets in its
//!   interface and the write path, to exercise the CRC/EDAC handling): one
//!   per 180 days.
//!
//! Hazards are accumulated (Λ += λ dt) against exponential thresholds drawn
//! once per event, so a step costs no random draw.
//!
//! **Total ionizing dose**: 1.5 krad(Si)/yr at 500 km SSO behind 2 mm Al,
//! scaled e^((h − 500 km)/150 km) and (2 mm / t)^1.2 — consistent with
//! BASELINE §6 ("design to 10 krad, ≥ 2× the 2-year dose behind 2 mm Al").
//! The accumulated dose follows the same region weights.

use super::math::{DAY_S, Rng, V3, YEAR_S};

/// Orbit-average OBC reset rate (per s): one per 90 days.
pub const OBC_RESET_MEAN_PER_S: f64 = 1.0 / (90.0 * DAY_S);
/// Orbit-average Jetson latch-up rate while powered (per s): one per 60 days.
pub const JETSON_SEL_MEAN_PER_S: f64 = 1.0 / (60.0 * DAY_S);
/// Orbit-average FRAM bit-flip rate (per s): one per 180 days.
pub const FRAM_FLIP_MEAN_PER_S: f64 = 1.0 / (180.0 * DAY_S);
/// Annual dose at 500 km behind 2 mm Al (rad(Si)/yr).
pub const TID_RAD_PER_YEAR_500KM_2MM: f64 = 1500.0;
/// Rate weight inside the SAA (at 500 km).
pub const SAA_WEIGHT: f64 = 30.0;
/// Rate weight at high magnetic latitude.
pub const POLAR_WEIGHT: f64 = 3.0;
/// Orbit-average weight at 97.4°, 500 km (normalises the base rates).
pub const MEAN_WEIGHT: f64 = 2.8;
/// Magnetic latitude above which the polar weight applies (rad).
pub const POLAR_MAGLAT_RAD: f64 = 55.0 * core::f64::consts::PI / 180.0;
/// Shielding assumed for the reported dose (mm Al).
pub const SHIELD_MM_AL: f64 = 2.0;

/// Geomagnetic north pole direction in ECEF (IGRF-14 dipole with secular
/// variation at 2028.4: 80.9° N, 72.9° W), fixed over the mission.
const POLE: V3 = V3::new(0.046_374, -0.150_696, 0.987_492);

/// Where the spacecraft is, for the radiation rates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// South Atlantic Anomaly.
    Saa,
    /// High magnetic latitude.
    Polar,
    /// Elsewhere.
    Quiet,
}

/// Region for a geodetic latitude, longitude (rad) and height (m).
pub fn region(lat: f64, lon: f64, _alt_m: f64) -> Region {
    let (la, lo) = (lat.to_degrees(), lon.to_degrees());
    let lo = (lo + 180.0).rem_euclid(360.0) - 180.0;
    if ((la + 26.0) / 18.0).powi(2) + ((lo + 50.0) / 40.0).powi(2) < 1.0 {
        return Region::Saa;
    }
    let (sl, cl) = lat.sin_cos();
    let (so, co) = lon.sin_cos();
    let maglat = V3::new(cl * co, cl * so, sl).dot(POLE).clamp(-1.0, 1.0).asin();
    if maglat.abs() > POLAR_MAGLAT_RAD { Region::Polar } else { Region::Quiet }
}

/// Rate weight at a position (relative to the base rate).
pub fn weight(lat: f64, lon: f64, alt_m: f64) -> f64 {
    match region(lat, lon, alt_m) {
        Region::Saa => SAA_WEIGHT * ((alt_m - 500e3) / 150e3).exp(),
        Region::Polar => POLAR_WEIGHT,
        Region::Quiet => 1.0,
    }
}

/// Annual total ionizing dose (rad(Si)/yr) at a mean altitude behind a
/// shield thickness.
pub fn tid_rad_per_year(alt_m: f64, shield_mm_al: f64) -> f64 {
    TID_RAD_PER_YEAR_500KM_2MM * ((alt_m - 500e3) / 150e3).exp() * (2.0 / shield_mm_al.max(0.1)).powf(1.2)
}

/// Single-event effects in one step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Events {
    /// The OBC suffered an upset or functional interrupt needing a reset.
    pub obc_reset: bool,
    /// The Jetson latched up (meaningful only while powered).
    pub jetson_latchup: bool,
    /// A bit flipped in FRAM.
    pub fram_bitflip: bool,
}

/// The radiation environment model.
#[derive(Clone, Debug)]
pub struct Radiation {
    rng: Rng,
    hazard: [f64; 3],
    threshold: [f64; 3],
    /// Accumulated dose behind 2 mm Al (rad(Si)).
    pub dose_rad: f64,
    /// Events so far: OBC resets, Jetson latch-ups, FRAM flips.
    pub counts: [u64; 3],
}

const MEAN_RATES: [f64; 3] = [OBC_RESET_MEAN_PER_S, JETSON_SEL_MEAN_PER_S, FRAM_FLIP_MEAN_PER_S];

impl Radiation {
    /// A model with its own random stream.
    pub fn new(rng: Rng) -> Radiation {
        let mut r = Radiation { rng, hazard: [0.0; 3], threshold: [0.0; 3], dose_rad: 0.0, counts: [0; 3] };
        for k in 0..3 {
            r.threshold[k] = r.rng.exp(1.0);
        }
        r
    }

    /// Advance by `dt` seconds at a geodetic latitude, longitude (rad) and
    /// height (m).
    pub fn step(&mut self, dt: f64, lat: f64, lon: f64, alt: f64) -> Events {
        let w = weight(lat, lon, alt) / MEAN_WEIGHT;
        self.dose_rad += tid_rad_per_year(500e3, SHIELD_MM_AL) / YEAR_S * dt * w;
        let mut fired = [false; 3];
        for k in 0..3 {
            self.hazard[k] += MEAN_RATES[k] * w * dt;
            if self.hazard[k] >= self.threshold[k] {
                fired[k] = true;
                self.counts[k] += 1;
                self.hazard[k] = 0.0;
                self.threshold[k] = self.rng.exp(1.0);
            }
        }
        Events { obc_reset: fired[0], jetson_latchup: fired[1], fram_bitflip: fired[2] }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::earth::subpoint;
    use crate::env::orbit::Orbit;
    use crate::env::time::from_utc;

    #[test]
    fn regions() {
        let d = f64::to_radians;
        assert_eq!(region(d(-26.0), d(-50.0), 500e3), Region::Saa);
        assert_eq!(region(d(80.0), d(-70.0), 500e3), Region::Polar);
        assert_eq!(region(d(-70.0), d(120.0), 500e3), Region::Polar);
        assert_eq!(region(d(10.0), d(100.0), 500e3), Region::Quiet);
        assert!((POLE.norm() - 1.0).abs() < 1e-5);
        assert!(tid_rad_per_year(500e3, 2.0) * 2.0 * 2.0 <= 10_000.0);
    }

    #[test]
    fn mean_weight_and_rates_over_a_year() {
        let t0 = from_utc(2028, 6, 1, 0, 0, 0.0);
        let mut o = Orbit::sso(500e3, 10.5, t0);
        let mut rad = Radiation::new(Rng::new(77));
        let (mut wsum, mut saa, mut polar, mut n) = (0.0, 0u32, 0u32, 0u32);
        let dt = 30.0;
        let mut t = t0;
        while t < t0 + 365.25 * DAY_S {
            o.propagate(t);
            let (r, _) = o.state();
            let (lat, lon, h) = subpoint(r, t);
            wsum += weight(lat, lon, h);
            match region(lat, lon, h) {
                Region::Saa => saa += 1,
                Region::Polar => polar += 1,
                Region::Quiet => {}
            }
            n += 1;
            let _ = rad.step(dt, lat, lon, h);
            t += dt;
        }
        let nf = f64::from(n);
        println!(
            "SAA {:.1} %, polar {:.1} %, mean weight {:.2}; one year: {} OBC resets, {} Jetson SEL (if always on), \
             {} FRAM flips, dose {:.0} rad(Si)",
            100.0 * f64::from(saa) / nf,
            100.0 * f64::from(polar) / nf,
            wsum / nf,
            rad.counts[0],
            rad.counts[1],
            rad.counts[2],
            rad.dose_rad
        );
        assert!((wsum / nf / MEAN_WEIGHT - 1.0).abs() < 0.15);
        assert!((rad.dose_rad / 1500.0 - 1.0).abs() < 0.15);
        // ≈ 4 OBC resets a year expected
        assert!(rad.counts[0] <= 12);
    }
}
