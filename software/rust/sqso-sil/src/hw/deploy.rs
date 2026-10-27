//! Deployment mechanisms: the separation switches, the two antennas and
//! the two solar wings held by lines that burn-wire resistors cut, and
//! their deployed-state switches (SQSO-HW-DIAG-001 S5, W7).
//!
//! One burn line serves both antennas and one both wings (finding F4): the
//! driver feeds two resistors in parallel. The S5 driver limits each
//! resistor's current (finding F7): 5 Ω straight across a 14.4 V pack
//! would draw about 2.9 A each, 5.8 A for the pair, above the 5 A pack
//! fuse. A line parts once its resistor has delivered the release energy,
//! which varies from unit to unit and needs more in the cold. The two
//! deployed-state switches of a pair are in series: the sense reads
//! deployed only when both are.

use crate::env::math::Rng;

/// Burn resistor, Ω (`hardware/rf/ANTENNA_DESIGN.md`: about 5 Ω, TBR)
pub const BURN_OHMS: f64 = 5.0;
/// Per-resistor current with the S5 current limit, A (TBR)
pub const BURN_LIMIT_A: f64 = 0.8;
/// Mean release energy at 20 °C, J (a nichrome coil on a Dyneema line)
pub const RELEASE_J_MEAN: f64 = 6.0;
/// Its unit-to-unit spread (1 σ), J
pub const RELEASE_J_SIGMA: f64 = 1.2;
/// Extra release energy per °C below 20 °C (fraction)
pub const COLD_PER_C: f64 = 0.01;
/// Fraction of the heat a coil keeps between attempts a minute apart
pub const HEAT_RETAINED: f64 = 0.2;

/// One element: an antenna or a wing
#[derive(Clone, Copy, Debug)]
pub struct Element {
    /// Name, for the record
    pub name: &'static str,
    /// Energy to part the line at 20 °C, J
    pub release_j: f64,
    /// Energy delivered so far in the current attempt, J
    pub heat_j: f64,
    /// Deployed
    pub deployed: bool,
    /// Burn time so far, s
    pub burn_s: f64,
}

/// A pair on one burn line and one sense line
#[derive(Clone, Copy, Debug)]
pub struct Pair {
    /// The two elements
    pub elements: [Element; 2],
    /// The line was on in the last step
    pub was_burning: bool,
}

impl Pair {
    fn new(names: [&'static str; 2], rng: &mut Rng) -> Pair {
        let mut e = |name| Element {
            name,
            release_j: (RELEASE_J_MEAN + RELEASE_J_SIGMA * rng.normal()).max(2.0),
            heat_j: 0.0,
            deployed: false,
            burn_s: 0.0,
        };
        Pair { elements: [e(names[0]), e(names[1])], was_burning: false }
    }

    /// Both deployed (the series sense chain is closed)
    pub fn sensed(&self) -> bool {
        self.elements.iter().all(|e| e.deployed)
    }

    /// Current drawn from the pack by the line now, A
    fn current_a(&self, burning: bool, vbat: f64, limit: Option<f64>) -> f64 {
        if !burning {
            return 0.0;
        }
        let per = limit.map_or(vbat / BURN_OHMS, |l| l.min(vbat / BURN_OHMS));
        per * self.elements.iter().filter(|e| !e.deployed).count() as f64
    }

    fn step(&mut self, dt: f64, burning: bool, vbat: f64, temp_c: f64, limit: Option<f64>) -> f64 {
        let per = limit.map_or(vbat / BURN_OHMS, |l| l.min(vbat / BURN_OHMS));
        let current = self.current_a(burning, vbat, limit);
        for e in &mut self.elements {
            if e.deployed {
                continue;
            }
            if burning {
                e.heat_j += per * per * BURN_OHMS * dt;
                e.burn_s += dt;
                let need = e.release_j * (1.0 + COLD_PER_C * (20.0 - temp_c).max(0.0));
                if e.heat_j >= need {
                    e.deployed = true;
                }
            } else if self.was_burning {
                e.heat_j *= HEAT_RETAINED;
            }
        }
        self.was_burning = burning;
        current
    }
}

/// The separation switches and the deployables
#[derive(Clone, Debug)]
pub struct Deployables {
    /// Separated from the dispenser (the separation switches released)
    pub separated: bool,
    /// The antennas (VHF, UHF)
    pub antennas: Pair,
    /// The solar wings (W1, W2)
    pub wings: Pair,
    /// Per-resistor current limit of the burn driver, A (`None`: direct drive)
    pub limit_a: Option<f64>,
    /// Burn current drawn now, A
    pub current_a: f64,
}

impl Deployables {
    /// Stowed in the dispenser
    pub fn new(rng: &mut Rng, limit_a: Option<f64>) -> Deployables {
        Deployables {
            separated: false,
            antennas: Pair::new(["VHF antenna", "UHF antenna"], rng),
            wings: Pair::new(["wing W1", "wing W2"], rng),
            limit_a,
            current_a: 0.0,
        }
    }

    /// Advance with the burn lines as the pins and the arm switch drive
    /// them; the current drawn from the pack, A
    pub fn step(&mut self, dt: f64, armed: bool, antenna_line: bool, wing_line: bool, vbat: f64, temp_c: f64) -> f64 {
        let a = self.antennas.step(dt, armed && antenna_line, vbat, temp_c, self.limit_a);
        let w = self.wings.step(dt, armed && wing_line, vbat, temp_c, self.limit_a);
        self.current_a = a + w;
        self.current_a
    }

    /// Both antennas out
    pub fn antennas_deployed(&self) -> bool {
        self.antennas.sensed()
    }

    /// Both wings out
    pub fn wings_deployed(&self) -> bool {
        self.wings.sensed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_three_second_limited_burn_deploys_both_antennas() {
        let mut rng = Rng::new(3);
        let mut d = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        for _ in 0..30 {
            d.step(0.1, true, true, false, 15.0, 10.0);
        }
        assert!(d.antennas_deployed());
        assert!(!d.wings_deployed());
    }

    #[test]
    fn direct_drive_draws_more_than_the_fuse() {
        let mut rng = Rng::new(3);
        let mut d = Deployables::new(&mut rng, None);
        let i = d.step(0.1, true, true, false, 14.4, 20.0);
        assert!(i > 5.0, "{i}");
    }

    #[test]
    fn nothing_burns_without_the_arm() {
        let mut rng = Rng::new(3);
        let mut d = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        for _ in 0..100 {
            assert_eq!(d.step(0.1, false, true, true, 15.0, 20.0), 0.0);
        }
        assert!(!d.antennas_deployed());
    }
}
