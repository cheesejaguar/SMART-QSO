//! A condition the flight software checks again and again (a low state of
//! charge, a low battery voltage, a bus out of range), logged once when it
//! appears and once when it clears, with the number of checks that found
//! it, instead of at every check (RID FSW-14): a long episode then takes two
//! fault-log entries, not one every 10 s, and the log keeps its history.
//!
//! Only the logging changes. Whatever the software does about the
//! condition (a mode change, a safe-mode event) it still does at every
//! check.
//!
//! @requirement SRS-F153 A repeated condition shall be logged at its onset and its clearance (with the number of checks), not at every check

/// What a check found
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// As before: nothing to log
    None,
    /// The condition appeared: log it
    Onset,
    /// It cleared after being found this many times: log the clearance
    Cleared(u32),
}

/// One repeated condition
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Condition {
    active: bool,
    count: u32,
}

impl Condition {
    /// Not present
    pub const fn new() -> Condition {
        Condition { active: false, count: 0 }
    }

    /// One check
    pub fn check(&mut self, present: bool) -> Change {
        match (self.active, present) {
            (false, true) => {
                self.active = true;
                self.count = 1;
                Change::Onset
            }
            (true, true) => {
                self.count = self.count.saturating_add(1);
                Change::None
            }
            (true, false) => {
                self.active = false;
                Change::Cleared(core::mem::take(&mut self.count))
            }
            (false, false) => Change::None,
        }
    }

    /// Whether the condition is present now
    pub const fn active(&self) -> bool {
        self.active
    }

    /// Checks that found it in the current episode
    pub const fn count(&self) -> u32 {
        self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn onset_repeats_and_clearance() {
        let mut c = Condition::new();
        assert_eq!(c.check(false), Change::None);
        assert_eq!(c.check(true), Change::Onset);
        for _ in 0..1000 {
            assert_eq!(c.check(true), Change::None);
        }
        assert!(c.active());
        assert_eq!(c.count(), 1001);
        assert_eq!(c.check(false), Change::Cleared(1001));
        assert_eq!(c.check(false), Change::None);
        assert_eq!(c.count(), 0);
        assert_eq!(c.check(true), Change::Onset, "a new episode");
    }
}
