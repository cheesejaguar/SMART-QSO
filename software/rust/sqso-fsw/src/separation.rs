//! The separation reading (`separation_switch.h`, RID SMA-B02): the
//! DEPLOY_SENSE line as the RF silence and the deployment wait use it.
//!
//! The hardware (SQSO-ICX-001 §3.1 pin 16; EPS board R113/R114, D23): the
//! EPS drives DEPLOY_SENSE **high when the separation switch DS2 is
//! released**, through 1 MΩ from DS2's drive node, with 1 MΩ to ground and a
//! clamp to 3V3_AO; stowed, the line is held low by the 1 MΩ. The OBC input
//! has no internal pull: either pull (25–55 kΩ) would override the 1 MΩ
//! source and fix the reading.
//!
//! The OBC is powered only after separation (the switches remove power from
//! the bus, CDS 2.3.2.4), so a power-on in the dispenser cannot happen and
//! the RF silence restarts at every power-on anyway. The sense line
//! therefore restarts the timers only until separation is established on
//! this boot: once the line has read released for [`LATCH_MS`] without a
//! break, the reading latches released until the next reset, and a later
//! stowed reading (an open line, BF-OBC-18) is counted for telemetry and
//! changes nothing. Before the latch a stowed reading counts at once, as on
//! the bench (umbilical power, switches stowed), where the timers must not
//! run.
//!
//! After a reset the latch starts again, but separation established
//! before the reset (the deployment record's separation flag, or the served
//! RF silence: `deployment.rs`) latches it at once ([`Separation::establish`],
//! RID SMA-B02 reopened): a line that failed open cannot then hold the RF
//! silence or the deployment wait for ever.
//!
//! @requirement SRS-F159 DEPLOY_SENSE high = released, no internal pull; released for 10 s latches separation until the next reset, after which a stowed reading is counted and restarts neither timer; separation persisted in the deployment record (or the served RF silence) keeps a stowed reading from restarting either timer after a reset

/// Released this long without a break latches separation, ms
pub const LATCH_MS: u64 = 10_000;

/// The reading's state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Separation {
    /// Reading released without a break since, ms
    released_since: Option<u64>,
    /// Separation latched on this boot
    pub latched: bool,
    /// Stowed readings after the latch (episodes: a run of stowed readings counts once)
    pub stowed_after_latch: u32,
    /// The previous reading was stowed
    last_stowed: bool,
}

impl Separation {
    /// At boot: not latched
    pub const fn new() -> Separation {
        Separation { released_since: None, latched: false, stowed_after_latch: 0, last_stowed: false }
    }

    /// Separation established before this boot (the deployment record or
    /// the served RF silence): latched now
    pub fn establish(&mut self) {
        self.latched = true;
    }

    /// The line's level `high` at `now_ms`: whether the timers are to treat
    /// the switches as stowed
    pub fn stowed(&mut self, high: bool, now_ms: u64) -> bool {
        let raw_stowed = !high;
        let onset = raw_stowed && !self.last_stowed;
        self.last_stowed = raw_stowed;
        if self.latched {
            if onset {
                self.stowed_after_latch = self.stowed_after_latch.saturating_add(1);
            }
            return false;
        }
        if raw_stowed {
            self.released_since = None;
            return true;
        }
        let since = *self.released_since.get_or_insert(now_ms);
        if now_ms.saturating_sub(since) >= LATCH_MS {
            self.latched = true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_is_released_and_low_is_stowed() {
        let mut s = Separation::new();
        assert!(s.stowed(false, 0));
        assert!(!s.stowed(true, 100));
        assert!(!s.latched);
    }

    #[test]
    fn released_for_ten_seconds_latches() {
        let mut s = Separation::new();
        assert!(!s.stowed(true, 1_000));
        assert!(!s.stowed(true, 1_000 + LATCH_MS - 1));
        assert!(!s.latched);
        // A break restarts the latch time
        assert!(s.stowed(false, 1_000 + LATCH_MS - 1));
        assert!(!s.stowed(true, 20_000));
        assert!(!s.stowed(true, 20_000 + LATCH_MS - 1));
        assert!(!s.latched);
        assert!(!s.stowed(true, 20_000 + LATCH_MS));
        assert!(s.latched);
    }

    #[test]
    fn after_the_latch_stowed_is_counted_and_ignored() {
        let mut s = Separation::new();
        assert!(!s.stowed(true, 0));
        assert!(!s.stowed(true, LATCH_MS));
        assert!(s.latched);
        assert!(!s.stowed(false, LATCH_MS + 1));
        assert!(!s.stowed(false, LATCH_MS + 2));
        assert_eq!(s.stowed_after_latch, 1);
        assert!(!s.stowed(true, LATCH_MS + 3));
        assert!(!s.stowed(false, LATCH_MS + 4));
        assert_eq!(s.stowed_after_latch, 2);
        assert!(s.latched);
    }

    /// RID SMA-B02: separation established before the reset latches at
    /// once; a line open from boot is counted and restarts nothing
    #[test]
    fn established_separation_latches_at_once() {
        let mut s = Separation::new();
        s.establish();
        assert!(!s.stowed(false, 0));
        assert!(!s.stowed(false, 1_000));
        assert!(s.latched);
        assert_eq!(s.stowed_after_latch, 1);
    }

    #[test]
    fn stowed_on_the_bench_holds_for_ever() {
        let mut s = Separation::new();
        for t in (0..3_600_000).step_by(60_000) {
            assert!(s.stowed(false, t));
        }
        assert!(!s.latched);
        assert_eq!(s.stowed_after_latch, 0);
        assert_eq!(Separation::default(), Separation::new());
    }
}
