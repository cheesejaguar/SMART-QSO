//! When the Jetson runs (BASELINE §5, §6): the generative-beacon experiment
//! is a burst, not a resident load. In ACTIVE (sunlit, SOC ≥ 55 %), the
//! payload is powered for one batch — it boots, writes up to
//! [`BATCH_LINES`] beacon lines on request, and is switched off — then
//! rests for [`REST_MS`] while the onboard cache of lines is spent, one a
//! beacon. A batch of ten lines a 100-minute orbit holds the Jetson to
//! about 5 % duty (the power budget's ≤ 5 % tumbling, ≤ 10 % sun-pointed)
//! and its energy per accepted beacon well under the 0.1 Wh gate.
//!
//! In flight the schedule owns the payload's switch-on: the mode manager's
//! ACTIVE entry may switch the payload off but not on (the executive holds
//! it, `eps_control`). Should the payload be on outside a batch all the
//! same (a state restored after a reset), the schedule switches it off. A
//! switch-on the EPS refuses (the payload guard's cool-down, low SOC) is
//! asked again after [`RETRY_MS`].
//!
//! **Energy gate (BASELINE §6, decision D5; RIDs JETB-04, JETB-11).** The
//! generated text is worth flying only at no more than 0.1 Wh (360 J) of
//! carrier-input energy per accepted line. The executive meters each
//! burst's energy (rail voltage × rail current, [`Schedule::add_energy`]);
//! once it reaches [`ENERGY_GATE_J`] × [`BATCH_LINES`] the batch can no
//! longer meet the gate even with every line, and it ends: at the carrier's
//! 7.2–7.5 W that is 480–500 s, before the 10-minute limit. The energy and
//! the lines of the last burst are kept for the housekeeping frame.
//!
//! @requirement SRS-F016 Enforce power budget limits per mode
//! @requirement SRS-F151 A Jetson burst ends when its energy reaches 360 J per line of a full batch

/// Lines a batch collects (BASELINE §6: batches of at least 10)
pub const BATCH_LINES: usize = 10;
/// Longest a batch may run, ms (boot up to 30 s, ten lines 30 s apart)
pub const BATCH_TIMEOUT_MS: u64 = 600_000;
/// Rest between the end of one batch and the start of the next, ms
pub const REST_MS: u64 = 6_000_000;
/// A refused switch-on is asked again after this long, ms
pub const RETRY_MS: u64 = 60_000;
/// Energy gate per accepted line, J (0.1 Wh)
pub const ENERGY_GATE_J: f64 = 360.0;
/// Telemetry and request interval during a batch, ms (ICD §5.2 minimum)
pub const BATCH_INTERVAL_MS: u32 = 30_000;

/// What the schedule wants done
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing
    None,
    /// Switch the payload on (a batch starts)
    PowerOn,
    /// Switch it off (the batch is done, timed out, or not allowed yet)
    PowerOff,
}

/// The schedule's state
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Schedule {
    /// When the running batch started, ms
    pub batch_start_ms: Option<u64>,
    /// When the last batch ended, ms
    pub last_end_ms: Option<u64>,
    /// When the payload was last asked to switch on, ms
    pub last_request_ms: Option<u64>,
    /// Energy of the running batch, J
    pub energy_j: f64,
    /// The last batch's energy, J, and lines
    pub last_burst: (f64, usize),
    /// Batches ended by the energy gate
    pub gate_ends: u32,
    /// Batches run
    pub bursts: u32,
}

impl Schedule {
    /// No batch yet
    pub const fn new() -> Schedule {
        Schedule {
            batch_start_ms: None,
            last_end_ms: None,
            last_request_ms: None,
            energy_j: 0.0,
            last_burst: (0.0, 0),
            gate_ends: 0,
            bursts: 0,
        }
    }

    /// Meter `joules` of the payload's input energy into the running batch
    pub fn add_energy(&mut self, joules: f64) {
        if self.batch_start_ms.is_some() && joules.is_finite() && joules > 0.0 {
            self.energy_j += joules;
        }
    }

    /// The running batch has reached the energy gate
    #[allow(clippy::cast_precision_loss)] // ten lines
    pub fn gate_reached(&self) -> bool {
        self.energy_j >= ENERGY_GATE_J * BATCH_LINES as f64
    }

    fn end(&mut self, now_ms: u64, cached: usize) {
        self.batch_start_ms = None;
        self.last_end_ms = Some(now_ms);
        self.last_burst = (self.energy_j, cached);
    }

    /// Whether a batch may start now with `cached` lines left
    pub fn may_start(&self, now_ms: u64, cached: usize) -> bool {
        cached == 0 && self.last_end_ms.is_none_or(|end| now_ms.saturating_sub(end) >= REST_MS)
    }

    /// One pass: whether the payload is on, the mode is ACTIVE, and the
    /// lines cached; what to do
    pub fn step(&mut self, now_ms: u64, payload_on: bool, active: bool, cached: usize) -> Action {
        if !payload_on {
            if self.batch_start_ms.is_some() {
                // Switched off by someone else (guard, mode change): the batch ends
                self.end(now_ms, cached);
            }
            let asked_lately = self.last_request_ms.is_some_and(|t| now_ms.saturating_sub(t) < RETRY_MS);
            if active && !asked_lately && self.may_start(now_ms, cached) {
                self.last_request_ms = Some(now_ms);
                return Action::PowerOn;
            }
            return Action::None;
        }
        let Some(start) = self.batch_start_ms else {
            // A batch starts only while it is allowed: the payload on just as
            // the hard limits fail (an eclipse a pass after the switch-on)
            // is switched off again, not started (SIL-39)
            if active && self.may_start(now_ms, cached) {
                self.batch_start_ms = Some(now_ms);
                self.energy_j = 0.0;
                self.bursts = self.bursts.wrapping_add(1);
                return Action::None;
            }
            return Action::PowerOff; // not yet: veto the mode manager's switch-on
        };
        let gate = self.gate_reached();
        if cached >= BATCH_LINES || now_ms.saturating_sub(start) >= BATCH_TIMEOUT_MS || !active || gate {
            if gate && cached < BATCH_LINES {
                self.gate_ends = self.gate_ends.wrapping_add(1);
            }
            self.end(now_ms, cached);
            return Action::PowerOff;
        }
        Action::None
    }

    /// Whether a batch is running
    pub const fn in_batch(&self) -> bool {
        self.batch_start_ms.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_energy_gate_ends_a_burst_before_the_time_limit() {
        let mut s = Schedule::new();
        s.add_energy(1e6); // no batch: not metered
        assert_eq!(s.step(0, true, true, 0), Action::None);
        assert_eq!(s.energy_j, 0.0);
        // 7.35 W at the carrier input, 20 ms passes
        let mut t = 0;
        let mut ended = None;
        while t < BATCH_TIMEOUT_MS {
            t += 20;
            s.add_energy(7.35 * 0.02);
            if s.step(t, true, true, 3) == Action::PowerOff {
                ended = Some(t);
                break;
            }
        }
        let ended = ended.unwrap();
        assert!(ended < BATCH_TIMEOUT_MS, "{ended}");
        assert!((480_000..=500_000).contains(&ended), "{ended}");
        assert_eq!(s.gate_ends, 1);
        assert_eq!(s.last_burst.1, 3);
        assert!((s.last_burst.0 - 3600.0).abs() < 1.0);
        s.add_energy(f64::NAN);
        assert_eq!(s.bursts, 1);
    }

    #[test]
    fn a_batch_runs_until_its_lines_are_in_then_rests() {
        let mut s = Schedule::new();
        assert_eq!(s.step(1000, false, true, 0), Action::PowerOn);
        assert_eq!(s.step(1020, true, true, 0), Action::None);
        assert!(s.in_batch());
        assert_eq!(s.step(200_000, true, true, 9), Action::None);
        assert_eq!(s.step(200_020, true, true, BATCH_LINES), Action::PowerOff);
        assert!(!s.in_batch());
        // Resting, and lines cached: no new batch, and a mode-manager switch-on is vetoed
        assert_eq!(s.step(300_000, false, true, 4), Action::None);
        assert_eq!(s.step(300_020, true, true, 4), Action::PowerOff);
        assert_eq!(s.step(200_020 + REST_MS - 1, false, true, 0), Action::None);
        assert_eq!(s.step(200_020 + REST_MS, false, true, 0), Action::PowerOn);
    }

    #[test]
    fn a_batch_times_out_and_ends_outside_active() {
        let mut s = Schedule::new();
        assert_eq!(s.step(0, true, true, 0), Action::None);
        assert_eq!(s.step(BATCH_TIMEOUT_MS, true, true, 3), Action::PowerOff);
        let mut s = Schedule::new();
        assert_eq!(s.step(0, true, true, 0), Action::None);
        assert_eq!(s.step(5000, true, false, 3), Action::PowerOff);
        assert_eq!(s.last_end_ms, Some(5000));
    }

    #[test]
    fn switched_off_by_the_guard_ends_the_batch() {
        let mut s = Schedule::new();
        assert_eq!(s.step(0, true, true, 0), Action::None);
        assert_eq!(s.step(9000, false, true, 2), Action::None);
        assert_eq!(s.last_end_ms, Some(9000));
        assert!(!s.may_start(9000 + REST_MS, 2));
        assert!(s.may_start(9000 + REST_MS, 0));
    }

    #[test]
    fn a_refused_switch_on_is_asked_again_later() {
        let mut s = Schedule::new();
        assert_eq!(s.step(0, false, true, 0), Action::PowerOn);
        assert_eq!(s.step(1000, false, true, 0), Action::None, "refused: not asked again at once");
        assert_eq!(s.step(RETRY_MS - 1, false, true, 0), Action::None);
        assert_eq!(s.step(RETRY_MS, false, true, 0), Action::PowerOn);
    }

    #[test]
    fn nothing_starts_outside_active() {
        let mut s = Schedule::new();
        assert_eq!(s.step(0, false, false, 0), Action::None);
        // Switched on while allowed, no longer allowed a pass later: off, no batch
        assert_eq!(s.step(1000, false, true, 0), Action::PowerOn);
        assert_eq!(s.step(2000, true, false, 0), Action::PowerOff);
        assert!(!s.in_batch());
        assert_eq!(s.bursts, 0);
    }
}
