//! When the Jetson runs (BASELINE §5, §6): the generative-beacon experiment
//! is a burst, not a resident load. In ACTIVE (sunlit, SOC ≥ 55 %), the
//! payload is powered for one batch — it boots, writes up to
//! [`BATCH_LINES`] beacon lines on request, and is switched off — then
//! rests for [`REST_MS`] while the onboard cache of lines is spent, one a
//! beacon. A batch of ten lines a 100-minute orbit holds the Jetson to
//! about 5 % duty (the power budget's ≤ 5 % tumbling, ≤ 10 % sun-pointed)
//! and its energy per accepted beacon well under the 0.1 Wh gate.
//!
//! The mode manager's ACTIVE entry still switches the payload on (its
//! contract); while resting, or with lines still cached, the schedule
//! vetoes that at once, so the rail never comes up.
//!
//! @requirement SRS-F016 Enforce power budget limits per mode

/// Lines a batch collects (BASELINE §6: batches of at least 10)
pub const BATCH_LINES: usize = 10;
/// Longest a batch may run, ms (boot up to 30 s, ten lines 30 s apart)
pub const BATCH_TIMEOUT_MS: u64 = 600_000;
/// Rest between the end of one batch and the start of the next, ms
pub const REST_MS: u64 = 6_000_000;
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Schedule {
    /// When the running batch started, ms
    pub batch_start_ms: Option<u64>,
    /// When the last batch ended, ms
    pub last_end_ms: Option<u64>,
}

impl Schedule {
    /// No batch yet
    pub const fn new() -> Schedule {
        Schedule { batch_start_ms: None, last_end_ms: None }
    }

    /// Whether a batch may start now with `cached` lines left
    pub fn may_start(&self, now_ms: u64, cached: usize) -> bool {
        cached == 0 && self.last_end_ms.is_none_or(|end| now_ms.saturating_sub(end) >= REST_MS)
    }

    /// One pass: whether the payload is on, the mode is ACTIVE, and the
    /// lines cached; what to do
    pub fn step(&mut self, now_ms: u64, payload_on: bool, active: bool, cached: usize) -> Action {
        if !payload_on {
            if self.batch_start_ms.take().is_some() {
                // Switched off by someone else (guard, mode change): the batch ends
                self.last_end_ms = Some(now_ms);
            }
            return if active && self.may_start(now_ms, cached) { Action::PowerOn } else { Action::None };
        }
        let Some(start) = self.batch_start_ms else {
            if self.may_start(now_ms, cached) {
                self.batch_start_ms = Some(now_ms);
                return Action::None;
            }
            return Action::PowerOff; // not yet: veto the mode manager's switch-on
        };
        if cached >= BATCH_LINES || now_ms.saturating_sub(start) >= BATCH_TIMEOUT_MS || !active {
            self.batch_start_ms = None;
            self.last_end_ms = Some(now_ms);
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
    fn nothing_starts_outside_active() {
        let mut s = Schedule::new();
        assert_eq!(s.step(0, false, false, 0), Action::None);
    }
}
