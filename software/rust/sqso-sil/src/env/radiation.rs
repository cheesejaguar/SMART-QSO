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
//! * Command receiver single-event functional interrupt (the radio board's
//!   MCU or synthesizer hung, silent until its power is cycled; FSW-09):
//!   one per 120 days of powered time.
//!
//! Hazards are accumulated (Λ += λ dt) against exponential thresholds drawn
//! once per event, so a step costs no random draw.
//!
//! **Total ionizing dose** (`docs/environment/RADIATION_ANALYSIS.md` Rev C
//! §4.1, RID RAD-02): behind 2 mm Al at 500 km, 2-year totals of 3,660
//! rad(Si) trapped electrons and protons, 1,494 solar protons (95 %) and
//! 100 GCR, so 2.63 krad(Si)/yr. The trapped part accumulates with the
//! region weights (the SAA weight carries e^((h − 500 km)/150 km), which
//! reproduces §4.1's 500-to-525 km ratio of the trapped dose, 1.18); the
//! solar and GCR part evenly. Other depths scale as (2 mm / t)^1.35 (§4.4's
//! fit between 1 and 3 mm). Revision B's 1.5 krad(Si)/yr left out the solar
//! protons and was 1.6–1.8 times low.

use super::math::{DAY_S, Rng, V3, YEAR_S};

/// Orbit-average OBC reset rate (per s): one per 90 days.
pub const OBC_RESET_MEAN_PER_S: f64 = 1.0 / (90.0 * DAY_S);
/// Orbit-average Jetson latch-up rate while powered (per s): one per 60 days.
pub const JETSON_SEL_MEAN_PER_S: f64 = 1.0 / (60.0 * DAY_S);
/// Orbit-average FRAM bit-flip rate (per s): one per 180 days.
pub const FRAM_FLIP_MEAN_PER_S: f64 = 1.0 / (180.0 * DAY_S);
/// Annual trapped-particle dose at 500 km behind 2 mm Al (rad(Si)/yr):
/// 3,660 in 2 years (RADIATION_ANALYSIS Rev C §4.1)
pub const TID_TRAPPED_RAD_PER_YEAR_2MM: f64 = 1830.0;
/// Annual solar-proton (95 %) and GCR dose behind 2 mm Al (rad(Si)/yr):
/// (1,494 + 100) in 2 years
pub const TID_SOLAR_GCR_RAD_PER_YEAR_2MM: f64 = 797.0;
/// Annual dose at 500 km behind 2 mm Al (rad(Si)/yr): 2.63 krad(Si)
pub const TID_RAD_PER_YEAR_500KM_2MM: f64 = TID_TRAPPED_RAD_PER_YEAR_2MM + TID_SOLAR_GCR_RAD_PER_YEAR_2MM;
/// Depth exponent of the dose (RADIATION_ANALYSIS Rev C §4.4)
pub const TID_DEPTH_EXPONENT: f64 = 1.35;
/// Orbit-average command-receiver hang rate (per s): one per 120 days.
pub const RX_SEFI_MEAN_PER_S: f64 = 1.0 / (120.0 * DAY_S);
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
/// shield thickness (the trapped part scaled with altitude as the SAA
/// weight is).
pub fn tid_rad_per_year(alt_m: f64, shield_mm_al: f64) -> f64 {
    let depth = (2.0 / shield_mm_al.max(0.1)).powf(TID_DEPTH_EXPONENT);
    (TID_TRAPPED_RAD_PER_YEAR_2MM * ((alt_m - 500e3) / 150e3).exp() + TID_SOLAR_GCR_RAD_PER_YEAR_2MM) * depth
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
    /// The command receiver hung (meaningful only while powered).
    pub rx_sefi: bool,
}

/// The radiation environment model.
#[derive(Clone, Debug)]
pub struct Radiation {
    rng: Rng,
    hazard: [f64; 4],
    threshold: [f64; 4],
    /// Accumulated dose behind 2 mm Al (rad(Si)).
    pub dose_rad: f64,
    /// Events so far: OBC resets, Jetson latch-ups, FRAM flips, receiver hangs.
    pub counts: [u64; 4],
}

const MEAN_RATES: [f64; 4] = [OBC_RESET_MEAN_PER_S, JETSON_SEL_MEAN_PER_S, FRAM_FLIP_MEAN_PER_S, RX_SEFI_MEAN_PER_S];

impl Radiation {
    /// A model with its own random stream.
    pub fn new(rng: Rng) -> Radiation {
        let mut r = Radiation { rng, hazard: [0.0; 4], threshold: [0.0; 4], dose_rad: 0.0, counts: [0; 4] };
        for k in 0..4 {
            r.threshold[k] = r.rng.exp(1.0);
        }
        r
    }

    /// Advance by `dt` seconds at a geodetic latitude, longitude (rad) and
    /// height (m).
    pub fn step(&mut self, dt: f64, lat: f64, lon: f64, alt: f64) -> Events {
        let w = weight(lat, lon, alt) / MEAN_WEIGHT;
        let depth = (2.0 / SHIELD_MM_AL).powf(TID_DEPTH_EXPONENT);
        self.dose_rad += (TID_TRAPPED_RAD_PER_YEAR_2MM * w + TID_SOLAR_GCR_RAD_PER_YEAR_2MM) * depth / YEAR_S * dt;
        let mut fired = [false; 4];
        for k in 0..4 {
            self.hazard[k] += MEAN_RATES[k] * w * dt;
            if self.hazard[k] >= self.threshold[k] {
                fired[k] = true;
                self.counts[k] += 1;
                self.hazard[k] = 0.0;
                self.threshold[k] = self.rng.exp(1.0);
            }
        }
        Events { obc_reset: fired[0], jetson_latchup: fired[1], fram_bitflip: fired[2], rx_sefi: fired[3] }
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
        // RADIATION_ANALYSIS Rev C §4.1: 5.25 krad(Si) in 2 years behind
        // 2 mm at 500 km, 5.92 at 525 km; 1 mm 12.6, 3 mm 2.87 (500 km)
        assert!((2.0 * tid_rad_per_year(500e3, 2.0) - 5254.0).abs() < 10.0);
        assert!((2.0 * tid_rad_per_year(525e3, 2.0) / 5917.0 - 1.0).abs() < 0.02);
        assert!((2.0 * tid_rad_per_year(500e3, 1.0) / 12_601.0 - 1.0).abs() < 0.08);
        assert!((2.0 * tid_rad_per_year(500e3, 3.0) / 2866.0 - 1.0).abs() < 0.08);
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
        assert!((rad.dose_rad / TID_RAD_PER_YEAR_500KM_2MM - 1.0).abs() < 0.10, "{}", rad.dose_rad);
        // ≈ 4 OBC resets a year expected
        assert!(rad.counts[0] <= 12);
    }
}
