//! Deployment mechanisms: the separation switches, the two solar wings held
//! by lines that burn-wire resistors cut, and the bought ISIS AntS antenna
//! with its own deployment controller (SQSO-HW-DIAG-001 S5, W7;
//! `docs/hardware/BUILD_VS_BUY.md` §2.6; CDR findings STR-15, STR-16).
//!
//! **Wings.** Each wing's hold-down line has two burn resistors, each on
//! its own line and driver (`hardware/electrical/eps/EPS_BOARD_DESIGN.md`
//! §5: BURN_W1A, W1B, W2A, W2B; CDR EPS board design): four lines, burned
//! one at a time by the flight software. A driver is a 0.80 A current
//! limiter fed from the 7.04 V burn buck behind SW6 (DEPLOY_EN), with a
//! TPL5111 one-shot that ends a burn [`ONE_SHOT_S`] after the line rises.
//! A line parts once either of its resistors has delivered that resistor's
//! release energy, which varies from unit to unit and needs more in the
//! cold (the wing's own temperature). A resistor draws its current for as
//! long as its driver is on, whether or not its line has parted. The pack
//! supplies the buck: [`BURN_BUCK_V`] × the drivers' current over the
//! pack voltage and [`BURN_BUCK_EFF`], at most [`BURN_BUCK_LIMIT_A`] on the
//! buck's side (a four-driver software fault). The two wings'
//! deployed-state switches are in series: the sense reads deployed only
//! when both are.
//!
//! **Antenna.** The AntS deploys its four tape elements itself, one at a
//! time, when the OBC has armed it and asked for an automatic deployment
//! (I2C, `sqso_fsw::ants`): each element burns until its tape is released or
//! the time limit; the status word reports each element's switch. It draws
//! [`ANTS_BURN_W`] from the bus while burning.

use crate::env::math::Rng;

/// Burn resistor, Ω (`hardware/rf/ANTENNA_DESIGN.md`: about 5 Ω, to be measured)
pub const BURN_OHMS: f64 = 5.0;
/// The burn buck's output, V (EPS_BOARD_DESIGN §5: U17)
pub const BURN_BUCK_V: f64 = 7.04;
/// Its efficiency at 0.8 A (an assumption that gives the board's 0.51 A
/// at the pack for one burn at 12.8 V)
pub const BURN_BUCK_EFF: f64 = 0.86;
/// Its current limit, A
pub const BURN_BUCK_LIMIT_A: f64 = 2.8;
/// The driver's one-shot: a burn ends this long after its line rises, s
pub const ONE_SHOT_S: f64 = 12.0;
/// Per-resistor current with the S5 current limit, A (to be set with the power reviewer)
pub const BURN_LIMIT_A: f64 = 0.8;
/// Mean release energy at 20 °C, J (a nichrome coil on a Dyneema line)
pub const RELEASE_J_MEAN: f64 = 6.0;
/// Its unit-to-unit spread (1 σ), J
pub const RELEASE_J_SIGMA: f64 = 1.2;
/// Extra release energy per °C below 20 °C (fraction)
pub const COLD_PER_C: f64 = 0.01;
/// Fraction of the heat a coil keeps between attempts
pub const HEAT_RETAINED: f64 = 0.2;
/// The AntS's burn power from the bus while an element burns, W
pub const ANTS_BURN_W: f64 = 1.5;
/// Mean release energy of an AntS tape element at 20 °C, J
pub const ANTS_RELEASE_J_MEAN: f64 = 4.0;
/// Its spread (1 σ), J
pub const ANTS_RELEASE_J_SIGMA: f64 = 0.8;

fn release(rng: &mut Rng, mean: f64, sigma: f64) -> f64 {
    (mean + sigma * rng.normal()).max(mean / 3.0)
}

/// One wing: two resistors on its hold-down line
#[derive(Clone, Copy, Debug)]
pub struct Wing {
    /// Name, for the record
    pub name: &'static str,
    /// Energy each resistor needs to part the line at 20 °C, J
    pub release_j: [f64; 2],
    /// Energy each resistor has delivered in its current attempt, J
    pub heat_j: [f64; 2],
    /// Deployed
    pub deployed: bool,
    /// Burn time so far (either resistor), s
    pub burn_s: f64,
}

/// The two wings and their four drivers
#[derive(Clone, Copy, Debug)]
pub struct Wings {
    /// W1 and W2
    pub elements: [Wing; 2],
    /// Each driver ([wing][resistor]) was commanded in the last step
    pub was_burning: [[bool; 2]; 2],
    /// How long each driver has been on since its line rose, s
    on_s: [[f64; 2]; 2],
}

impl Wings {
    fn new(rng: &mut Rng) -> Wings {
        let mut w = |name| Wing {
            name,
            release_j: [release(rng, RELEASE_J_MEAN, RELEASE_J_SIGMA), release(rng, RELEASE_J_MEAN, RELEASE_J_SIGMA)],
            heat_j: [0.0; 2],
            deployed: false,
            burn_s: 0.0,
        };
        Wings { elements: [w("wing W1"), w("wing W2")], was_burning: [[false; 2]; 2], on_s: [[0.0; 2]; 2] }
    }

    /// Both deployed (the series sense chain is closed)
    pub fn sensed(&self) -> bool {
        self.elements.iter().all(|e| e.deployed)
    }

    /// One step with the lines ([wing][resistor]) as driven; the current
    /// the drivers draw from the burn buck, A
    fn step(&mut self, dt: f64, lines: [[bool; 2]; 2], temps_c: [f64; 2], limit: Option<f64>) -> f64 {
        let per = limit.map_or(BURN_BUCK_V / BURN_OHMS, |l| l.min(BURN_BUCK_V / BURN_OHMS));
        let mut current = 0.0;
        for (k, (w, temp_c)) in self.elements.iter_mut().zip(temps_c).enumerate() {
            let mut any = false;
            for r in 0..2 {
                let line = lines[k][r];
                self.on_s[k][r] = if line { self.on_s[k][r] + dt } else { 0.0 };
                // The one-shot ends the burn 12 s after the line rose
                let driven = line && self.on_s[k][r] <= ONE_SHOT_S;
                if driven {
                    any = true;
                    current += per;
                    w.heat_j[r] += per * per * BURN_OHMS * dt;
                    let need = w.release_j[r] * (1.0 + COLD_PER_C * (20.0 - temp_c).max(0.0));
                    if !w.deployed && w.heat_j[r] >= need {
                        w.deployed = true;
                    }
                } else if self.was_burning[k][r] {
                    w.heat_j[r] *= HEAT_RETAINED;
                }
                self.was_burning[k][r] = driven;
            }
            if any && !w.deployed {
                w.burn_s += dt;
            }
        }
        current.min(BURN_BUCK_LIMIT_A)
    }
}

/// One AntS tape element
#[derive(Clone, Copy, Debug)]
pub struct AntsElement {
    /// Energy to release it at 20 °C, J
    pub release_j: f64,
    /// Energy delivered in the current activation, J
    pub heat_j: f64,
    /// Its switch reads deployed
    pub deployed: bool,
    /// Its last activation stopped at the time limit
    pub timed_out: bool,
}

/// The AntS and its controller
#[derive(Clone, Copy, Debug)]
pub struct Ants {
    /// The four tape elements
    pub elements: [AntsElement; 4],
    /// The element burning now and for how long, s
    active: Option<(usize, f64)>,
    /// The last command's time limit (0: none, or cancelled)
    last_limit_s: u8,
}

impl Ants {
    fn new(rng: &mut Rng) -> Ants {
        let mut e = || AntsElement {
            release_j: release(rng, ANTS_RELEASE_J_MEAN, ANTS_RELEASE_J_SIGMA),
            heat_j: 0.0,
            deployed: false,
            timed_out: false,
        };
        Ants { elements: [e(), e(), e(), e()], active: None, last_limit_s: 0 }
    }

    /// Every element out
    pub fn deployed(&self) -> bool {
        self.elements.iter().all(|e| e.deployed)
    }

    /// The status word the controller reports (`sqso_fsw::ants`)
    pub fn status(&self, armed: bool) -> [u8; 2] {
        let mut s = sqso_fsw::ants::Status { armed, ..Default::default() };
        for (k, e) in self.elements.iter().enumerate() {
            s.elements[k] = sqso_fsw::ants::ElementState {
                deployed: e.deployed,
                timed_out: e.timed_out,
                burning: self.active.is_some_and(|(a, _)| a == k),
            };
        }
        s.encode()
    }

    /// One step with the OBC's command (armed, the deployment's per-element
    /// limit, 0 for none); the current drawn, A
    fn step(&mut self, dt: f64, armed: bool, limit_s: u8, vbat: f64, temp_c: f64) -> f64 {
        let commanded = armed && limit_s > 0;
        if commanded && self.last_limit_s == 0 {
            // A new deployment: every element not out is tried again
            for e in &mut self.elements {
                e.timed_out = false;
                e.heat_j = 0.0;
            }
            self.active = None;
        }
        self.last_limit_s = if commanded { limit_s } else { 0 };
        if !commanded {
            self.active = None;
            return 0.0;
        }
        if self.active.is_none() {
            self.active = self.elements.iter().position(|e| !e.deployed && !e.timed_out).map(|k| (k, 0.0));
        }
        let Some((k, t)) = self.active else { return 0.0 };
        let e = &mut self.elements[k];
        e.heat_j += ANTS_BURN_W * dt;
        let t = t + dt;
        if e.heat_j >= e.release_j * (1.0 + COLD_PER_C * (20.0 - temp_c).max(0.0)) {
            e.deployed = true;
            self.active = None;
        } else if t >= f64::from(limit_s) {
            e.timed_out = true;
            self.active = None;
        } else {
            self.active = Some((k, t));
        }
        ANTS_BURN_W / vbat.max(1.0)
    }
}

/// The separation switches and the deployables
#[derive(Clone, Debug)]
pub struct Deployables {
    /// Separated from the dispenser (the separation switches released)
    pub separated: bool,
    /// The AntS antenna
    pub ants: Ants,
    /// The solar wings (W1, W2)
    pub wings: Wings,
    /// Per-resistor current limit of the burn driver, A (`None`: direct drive)
    pub limit_a: Option<f64>,
    /// Burn current drawn from the pack now, A
    pub current_a: f64,
    /// Most wing resistors burning at once
    pub max_wing_lines: usize,
}

/// What the OBC drives in one step
#[derive(Clone, Copy, Debug, Default)]
pub struct Commands {
    /// The master arm (`DEPLOY_EN`): the wing drivers need it
    pub armed: bool,
    /// The wing lines, [wing][resistor]: W1A, W1B; W2A, W2B
    pub wing: [[bool; 2]; 2],
    /// The AntS is armed (I2C)
    pub ants_armed: bool,
    /// The AntS deployment's per-element limit, s (0: none)
    pub ants_limit_s: u8,
}

impl Deployables {
    /// Stowed in the dispenser
    pub fn new(rng: &mut Rng, limit_a: Option<f64>) -> Deployables {
        Deployables { separated: false, ants: Ants::new(rng), wings: Wings::new(rng), limit_a, current_a: 0.0, max_wing_lines: 0 }
    }

    /// Advance with the OBC's lines and commands; the burn current drawn
    /// from the pack, A. `wing_c` are the wings' temperatures, `ants_c` the
    /// antenna face's
    pub fn step(&mut self, dt: f64, cmd: Commands, vbat: f64, wing_c: [f64; 2], ants_c: f64) -> f64 {
        let lines = cmd.wing.map(|w| w.map(|l| cmd.armed && l));
        let on = lines.iter().flatten().filter(|l| **l).count();
        self.max_wing_lines = self.max_wing_lines.max(on);
        let buck_a = self.wings.step(dt, lines, wing_c, self.limit_a);
        let w = buck_a * BURN_BUCK_V / (vbat.max(1.0) * BURN_BUCK_EFF);
        let a = self.ants.step(dt, cmd.ants_armed, cmd.ants_limit_s, vbat, ants_c);
        self.current_a = a + w;
        self.current_a
    }

    /// Every antenna element out
    pub fn antennas_deployed(&self) -> bool {
        self.ants.deployed()
    }

    /// Both wings out
    pub fn wings_deployed(&self) -> bool {
        self.wings.sensed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WARM: [f64; 2] = [10.0, 10.0];

    fn wing_line(w: usize, r: usize) -> Commands {
        let mut c = Commands { armed: true, ..Commands::default() };
        c.wing[w][r] = true;
        c
    }

    #[test]
    fn an_eight_second_burn_on_either_resistor_deploys_its_wing() {
        for r in 0..2 {
            let mut rng = Rng::new(3);
            let mut d = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
            for w in 0..2 {
                for _ in 0..80 {
                    d.step(0.1, wing_line(w, r), 15.0, WARM, 10.0);
                }
                assert!(d.wings.elements[w].deployed, "wing {w} resistor {r}");
            }
            assert!(d.wings_deployed());
            assert!(!d.antennas_deployed());
            assert_eq!(d.max_wing_lines, 1);
        }
    }

    #[test]
    fn one_burn_draws_about_half_an_amp_from_the_pack() {
        let mut rng = Rng::new(3);
        let mut d = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        let at_12v8 = d.step(0.1, wing_line(0, 0), 12.8, WARM, 10.0);
        assert!((at_12v8 - 0.51).abs() < 0.01, "{at_12v8}");
        // A parted line still draws its resistor's current
        for _ in 0..90 {
            d.step(0.1, wing_line(0, 0), 15.0, WARM, 10.0);
        }
        assert!(d.wings.elements[0].deployed);
        assert!(d.step(0.1, wing_line(0, 0), 15.0, WARM, 10.0) > 0.4);
        // The one-shot ends it 12 s after the line rose
        for _ in 0..40 {
            d.step(0.1, wing_line(0, 0), 15.0, WARM, 10.0);
        }
        assert_eq!(d.step(0.1, wing_line(0, 0), 15.0, WARM, 10.0), 0.0);
        // A four-driver fault is held by the buck's limit
        let all = Commands { armed: true, wing: [[true; 2]; 2], ..Commands::default() };
        let mut rng = Rng::new(3);
        let mut f = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        let i = f.step(0.1, all, 12.8, WARM, 10.0);
        assert!(i < 2.0 && i > 1.5, "{i}");
        assert_eq!(f.max_wing_lines, 4);
    }

    #[test]
    fn the_cold_wing_needs_more() {
        let mut rng = Rng::new(3);
        let mut cold = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        let mut rng = Rng::new(3);
        let mut warm = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        let (mut t_cold, mut t_warm) = (0, 0);
        for k in 0..110 {
            if !cold.wings.elements[0].deployed {
                cold.step(0.1, wing_line(0, 0), 15.0, [-40.0, -40.0], 10.0);
                t_cold = k;
            }
            if !warm.wings.elements[0].deployed {
                warm.step(0.1, wing_line(0, 0), 15.0, [20.0, 20.0], 10.0);
                t_warm = k;
            }
        }
        assert!(t_cold > t_warm, "{t_cold} {t_warm}");
    }

    #[test]
    fn nothing_burns_without_the_arm_or_a_command() {
        let mut rng = Rng::new(3);
        let mut d = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        let unarmed = Commands { armed: false, wing: [[true; 2]; 2], ..Commands::default() };
        for _ in 0..100 {
            assert_eq!(d.step(0.1, unarmed, 15.0, WARM, 20.0), 0.0);
        }
        assert!(!d.wings_deployed() && !d.antennas_deployed());
    }

    #[test]
    fn the_ants_deploys_its_elements_in_turn_and_reports_them() {
        let mut rng = Rng::new(5);
        let mut d = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        let stowed = sqso_fsw::ants::Status::decode(d.ants.status(false));
        assert!(!stowed.all_deployed() && !stowed.armed);
        let go = Commands { ants_armed: true, ants_limit_s: sqso_fsw::ants::ELEMENT_LIMIT_S, ..Commands::default() };
        let mut drew = 0.0;
        for _ in 0..450 {
            drew += d.step(0.1, go, 15.0, WARM, 10.0);
        }
        assert!(d.antennas_deployed());
        assert!(drew > 0.0);
        let s = sqso_fsw::ants::Status::decode(d.ants.status(true));
        assert!(s.all_deployed() && s.armed);
        // Disarmed, nothing more is drawn
        assert_eq!(d.step(0.1, Commands::default(), 15.0, WARM, 10.0), 0.0);
    }

    #[test]
    fn a_stuck_element_times_out_and_a_new_command_retries_it() {
        let mut rng = Rng::new(5);
        let mut d = Deployables::new(&mut rng, Some(BURN_LIMIT_A));
        d.ants.elements[2].release_j = f64::INFINITY;
        let go = Commands { ants_armed: true, ants_limit_s: 10, ..Commands::default() };
        for _ in 0..600 {
            d.step(0.1, go, 15.0, WARM, 10.0);
        }
        assert!(!d.antennas_deployed());
        assert!(d.ants.elements[2].timed_out);
        let s = sqso_fsw::ants::Status::decode(d.ants.status(true));
        assert!(s.elements[2].timed_out && !s.elements[2].deployed && s.elements[0].deployed);
        // Cancel, then deploy again: the element is tried again
        d.step(0.1, Commands::default(), 15.0, WARM, 10.0);
        d.ants.elements[2].release_j = 1.0;
        for _ in 0..50 {
            d.step(0.1, go, 15.0, WARM, 10.0);
        }
        assert!(d.antennas_deployed());
    }
}
