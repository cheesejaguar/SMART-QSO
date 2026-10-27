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
//! **Energy gate (BASELINE §6, decisions D5 and C-20; RIDs JETB-04,
//! JETB-11, P3F-G-03).** A burst is one Jetson power-on of at most
//! [`BATCH_TIMEOUT_MS`] (10 min) and [`BURST_ENERGY_MAX_J`] (3.6 kJ) at the
//! carrier input. The generated text is worth flying only at no more than
//! 0.1 Wh ([`ENERGY_GATE_J`], 360 J) of that energy per **accepted** line
//! (one that passes the content filter, [`Schedule::line_accepted`]). The
//! executive meters each burst's energy (rail voltage × rail current,
//! [`Schedule::add_energy`]); the burst ends as soon as its energy exceeds
//! 360 J for each line accepted so far plus the line in progress (the next
//! line can then no longer bring it within 360 J a line), or reaches the
//! 3.6 kJ cap (360 J × [`BATCH_LINES`]): a burst whose lines are refused,
//! or come too slowly, ends early instead of running on. At each accepted
//! line the burst is therefore within 360 J a line; what it spends after
//! its last accepted line (at most one line's 360 J) is counted in the
//! burst's energy, which the housekeeping and the mission simulation report
//! against its accepted lines. The energy and the accepted lines of
//! the last burst are kept for the housekeeping frame (HK2 `JB`).
//!
//! @requirement SRS-F016 Enforce power budget limits per mode
//! @requirement SRS-F151 A Jetson burst (one power-on, at most 10 min and 3.6 kJ at the carrier input) ends when its energy exceeds 360 J for each line accepted by the content filter so far plus the one in progress, or reaches 3.6 kJ

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
/// Most energy one burst may use at the carrier input, J (decision C-20:
/// 3.6 kJ, 360 J for each line of a full batch)
#[allow(clippy::cast_precision_loss)] // ten lines
pub const BURST_ENERGY_MAX_J: f64 = ENERGY_GATE_J * BATCH_LINES as f64;
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
    /// Lines of the running batch accepted by the content filter
    pub accepted: usize,
    /// The last batch's energy, J, and accepted lines
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
            accepted: 0,
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

    /// A line of the running batch passed the content filter
    pub fn line_accepted(&mut self) {
        if self.batch_start_ms.is_some() {
            self.accepted = self.accepted.saturating_add(1);
        }
    }

    /// The running batch has reached the energy gate: more than 360 J for
    /// each line accepted so far and the one in progress, or the 3.6 kJ cap
    #[allow(clippy::cast_precision_loss)] // ten lines
    pub fn gate_reached(&self) -> bool {
        let allowed = ENERGY_GATE_J * (self.accepted.saturating_add(1)) as f64;
        self.energy_j > allowed || self.energy_j >= BURST_ENERGY_MAX_J
    }

    /// The last batch's energy per accepted line, J (`None`: none accepted)
    #[allow(clippy::cast_precision_loss)] // ten lines
    pub fn last_energy_per_line(&self) -> Option<f64> {
        let (energy, lines) = self.last_burst;
        (lines > 0).then(|| energy / lines as f64)
    }

    fn end(&mut self, now_ms: u64) {
        self.batch_start_ms = None;
        self.last_end_ms = Some(now_ms);
        self.last_burst = (self.energy_j, self.accepted);
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
                self.end(now_ms);
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
                self.accepted = 0;
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
            self.end(now_ms);
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

    /// A burst at 7.35 W whose lines come every `every_ms` from `first_ms`,
    /// one in `refused` refused by the filter: when it ends, its energy
    /// and accepted lines
    fn burst(first_ms: u64, every_ms: u64, refused: u64) -> (u64, f64, usize, u32) {
        let mut s = Schedule::new();
        s.add_energy(1e6); // no batch: not metered
        assert_eq!(s.step(0, true, true, 0), Action::None);
        assert_eq!(s.energy_j, 0.0);
        let (mut t, mut lines) = (0, 0u64);
        while t < 2 * BATCH_TIMEOUT_MS {
            t += 20;
            s.add_energy(7.35 * 0.02);
            if t >= first_ms && (t - first_ms).is_multiple_of(every_ms) {
                lines += 1;
                if refused == 0 || !lines.is_multiple_of(refused) {
                    s.line_accepted();
                }
            }
            if s.step(t, true, true, 0) == Action::PowerOff {
                return (t, s.last_burst.0, s.last_burst.1, s.gate_ends);
            }
        }
        panic!("never ended");
    }

    /// Decision C-20: the gate counts accepted lines. Lines every 30 s
    /// after a 30 s boot keep it under 360 J a line, and the 3.6 kJ cap
    /// (about 490 s at 7.35 W) ends the burst before the 10-minute limit
    #[test]
    fn the_energy_gate_ends_a_burst_at_the_cap() {
        let (ended, energy, lines, gates) = burst(30_000, 30_000, 0);
        assert!((480_000..=500_000).contains(&ended), "{ended}");
        assert!((energy - BURST_ENERGY_MAX_J).abs() < 1.0);
        assert_eq!(gates, 1);
        assert!(energy / lines as f64 <= ENERGY_GATE_J, "{energy} J / {lines}");
        // A full batch of ten lines ends by the lines, under the cap
        let mut s = Schedule::new();
        assert_eq!(s.step(0, true, true, 0), Action::None);
        for _ in 0..BATCH_LINES {
            s.line_accepted();
        }
        s.add_energy(2000.0);
        assert_eq!(s.step(300_000, true, true, BATCH_LINES), Action::PowerOff);
        assert_eq!(s.gate_ends, 0);
        assert_eq!(s.last_energy_per_line(), Some(200.0));
        s.add_energy(f64::NAN);
        s.line_accepted(); // no batch: not counted
        assert_eq!((s.bursts, s.accepted), (1, BATCH_LINES));
    }

    /// Decision C-20: refused lines are not counted, so a burst whose lines
    /// come too slowly, or are refused, ends as soon as the next line can
    /// no longer bring it within 360 J a line
    #[test]
    fn the_gate_counts_accepted_lines() {
        // No line accepted: ended at 360 J (49 s), not at 3.6 kJ
        let (ended, energy, lines, gates) = burst(1_000_000, 30_000, 0);
        assert!((48_000..=50_000).contains(&ended), "{ended}");
        assert!(energy > ENERGY_GATE_J && energy < ENERGY_GATE_J + 1.0);
        assert_eq!((lines, gates), (0, 1));
        // Lines every 60 s (441 J a line, more than the 360 J each brings):
        // the second at 661 J is within 720 J, the third cannot come within
        // 1,080 J: ended at 1,080 J, long before the cap
        let (ended, energy, lines, _) = burst(30_000, 60_000, 0);
        assert!((140_000..150_000).contains(&ended), "{ended}");
        assert_eq!(lines, 2);
        assert!(energy <= 3.0 * ENERGY_GATE_J + 1.0);
        // Every second line refused: the accepted lines bound it
        let (_, energy, lines, _) = burst(30_000, 30_000, 2);
        assert!(lines >= 1 && energy <= ENERGY_GATE_J * (lines as f64 + 1.0) + 1.0, "{energy} J, {lines}");
        assert_eq!(Schedule::new().last_energy_per_line(), None);
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
