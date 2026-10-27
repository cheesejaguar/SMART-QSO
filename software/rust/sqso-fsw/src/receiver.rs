//! Supervision of the 435 MHz command receiver (`receiver.h`): recovery of
//! a hung receiver by a brief power cycle (RID FSW-09), and the evidence
//! that the receiver has failed for good (RID REL-04).
//!
//! **Activity** is a valid uplink frame (any AX.25 UI frame whose FCS
//! checks, addressed to anyone) or the receiver's carrier detect (`RX_CD`,
//! the squelch open on a carrier in the passband). A working receiver
//! under the transponder's users sees one or the other most hours of the
//! day; a hung one (a single-event functional interrupt in its MCU,
//! synthesizer or demodulator) sees neither.
//!
//! **Recovery (FSW-09).** After [`SILENCE_MS`] with the radio rail powered
//! and no activity, `RADIO_EN` is switched off for [`CYCLE_OFF_MS`] and on
//! again (through `rf_control`'s `RF_INHIBIT_RX_CYCLE`, so the transmitter
//! enable drops first), logged. Never during a pass in progress: not within
//! [`PASS_GUARD_MS`] of the last activity, nor while the downlink or the
//! transponder is keyed. After a cycle the next one waits another
//! [`SILENCE_MS`] of silence, so at most one cycle a day is spent.
//!
//! **Failed for good (REL-04).** The receiver is *known failed* when, all at
//! once: the radio rail is powered; there has been no activity for longer
//! than the loss-of-contact timeout; the receiver's own health line reports
//! a fault (`RX_FAULT` high: synthesizer out of lock or supply out of its
//! window); and at least one recovery cycle has been tried in that silence.
//! The health line is active high so that a missing or open line reads
//! "healthy", and the exception below can never come from a wiring fault.
//! Any activity clears it. The silence is counted from boot (RAM): a reset
//! restarts it, which delays the exception (the conservative direction).
//!
//! @requirement SRS-F115 Hung-receiver recovery by a brief power cycle, never during a pass
//! @requirement SRS-F116 Evidence that the command receiver has failed

/// No activity for this long, the rail on, before a recovery cycle, ms
pub const SILENCE_MS: u64 = 24 * 3_600_000;
/// No recovery cycle within this long of the last activity, ms (a pass is
/// at most about 13 minutes)
pub const PASS_GUARD_MS: u64 = 15 * 60_000;
/// `RADIO_EN` off this long, ms
pub const CYCLE_OFF_MS: u64 = 2_000;

/// What one pass sees
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inputs {
    /// The radio rail is powered (`RADIO_EN` high, the RF silence over)
    pub powered: bool,
    /// A valid uplink frame arrived since the last pass
    pub frame: bool,
    /// Carrier detect (`RX_CD`) is high
    pub carrier: bool,
    /// The receiver's health line (`RX_FAULT`) reports a fault
    pub fault: bool,
    /// The downlink or the transponder is keyed
    pub keyed: bool,
    /// The loss-of-contact timeout, ms
    pub contact_timeout_ms: u64,
}

/// What the caller must do
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing
    None,
    /// Switch the radio off (start of a recovery cycle)
    PowerOff,
    /// Switch it on again (end of the cycle)
    PowerOn,
}

/// The supervisor's state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Receiver {
    /// Last activity, ms (the start of the count at boot)
    last_activity_ms: u64,
    /// The last recovery cycle (start), ms
    last_cycle_ms: Option<u64>,
    /// A recovery cycle has been tried since the last activity
    cycled_in_silence: bool,
    /// The radio is off for a cycle since this time, ms
    off_since: Option<u64>,
    /// Recovery cycles since boot
    pub cycles: u32,
    /// Known failed (REL-04 evidence)
    failed: bool,
}

impl Receiver {
    /// Counting from `now_ms` (boot)
    pub const fn new(now_ms: u64) -> Receiver {
        Receiver {
            last_activity_ms: now_ms,
            last_cycle_ms: None,
            cycled_in_silence: false,
            off_since: None,
            cycles: 0,
            failed: false,
        }
    }

    /// One pass
    pub fn step(&mut self, now_ms: u64, i: &Inputs) -> Action {
        if let Some(since) = self.off_since {
            if now_ms.saturating_sub(since) >= CYCLE_OFF_MS {
                self.off_since = None;
                return Action::PowerOn;
            }
            return Action::None;
        }
        if i.frame || i.carrier {
            self.last_activity_ms = now_ms;
            self.cycled_in_silence = false;
            self.failed = false;
        }
        if !i.powered {
            // Nothing heard is evidence of nothing while the rail is off
            self.last_activity_ms = now_ms;
            return Action::None;
        }
        let silent = now_ms.saturating_sub(self.last_activity_ms);
        self.failed = self.cycled_in_silence && i.fault && silent > i.contact_timeout_ms;
        let since_cycle = self.last_cycle_ms.map_or(u64::MAX, |t| now_ms.saturating_sub(t));
        if silent >= SILENCE_MS && silent >= PASS_GUARD_MS && since_cycle >= SILENCE_MS && !i.keyed {
            self.last_cycle_ms = Some(now_ms);
            self.cycled_in_silence = true;
            self.off_since = Some(now_ms);
            self.cycles = self.cycles.saturating_add(1);
            return Action::PowerOff;
        }
        Action::None
    }

    /// Whether the receiver is known failed (REL-04)
    pub const fn failed(&self) -> bool {
        self.failed
    }

    /// Whether a recovery cycle has the radio off now
    pub const fn cycling(&self) -> bool {
        self.off_since.is_some()
    }

    /// Time without activity, ms
    pub const fn silent_ms(&self, now_ms: u64) -> u64 {
        now_ms.saturating_sub(self.last_activity_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: u64 = 3_600_000;
    const DAY: u64 = 24 * H;

    fn quiet() -> Inputs {
        Inputs { powered: true, contact_timeout_ms: 30 * DAY, ..Inputs::default() }
    }

    #[test]
    fn a_silent_receiver_is_cycled_once_a_day_never_in_a_pass() {
        let mut r = Receiver::new(0);
        let q = quiet();
        assert_eq!(r.step(SILENCE_MS - 1, &q), Action::None);
        // Keyed: not now
        assert_eq!(r.step(SILENCE_MS, &Inputs { keyed: true, ..q }), Action::None);
        assert_eq!(r.step(SILENCE_MS + 1, &q), Action::PowerOff);
        assert!(r.cycling());
        assert_eq!(r.cycles, 1);
        assert_eq!(r.step(SILENCE_MS + CYCLE_OFF_MS, &q), Action::None);
        assert_eq!(r.step(SILENCE_MS + 1 + CYCLE_OFF_MS, &q), Action::PowerOn);
        assert!(!r.cycling());
        // The next cycle waits a whole silence again
        assert_eq!(r.step(2 * SILENCE_MS, &q), Action::None);
        assert_eq!(r.step(2 * SILENCE_MS + 1, &q), Action::PowerOff);
        assert_eq!(r.cycles, 2);
    }

    #[test]
    fn activity_and_an_unpowered_rail_restart_the_count() {
        let mut r = Receiver::new(0);
        let q = quiet();
        assert_eq!(r.step(SILENCE_MS - 1, &Inputs { carrier: true, ..q }), Action::None);
        assert_eq!(r.step(SILENCE_MS + 10, &q), Action::None);
        assert_eq!(r.silent_ms(SILENCE_MS + 10), 11);
        assert_eq!(r.step(2 * SILENCE_MS - 1, &Inputs { frame: true, ..q }), Action::None);
        assert_eq!(r.step(3 * SILENCE_MS - 2, &Inputs { powered: false, ..q }), Action::None);
        assert_eq!(r.step(4 * SILENCE_MS - 3, &q), Action::None);
        assert_eq!(r.step(4 * SILENCE_MS - 2, &q), Action::PowerOff);
    }

    #[test]
    fn known_failed_needs_every_piece_of_evidence() {
        let mut r = Receiver::new(0);
        let timeout = 2 * DAY;
        let q = Inputs { contact_timeout_ms: timeout, ..quiet() };
        let bad = Inputs { fault: true, ..q };
        assert_eq!(r.step(SILENCE_MS, &bad), Action::PowerOff);
        let _ = r.step(SILENCE_MS + CYCLE_OFF_MS, &bad);
        assert!(!r.failed(), "silence not yet past the contact timeout");
        // Keyed from here on, so that no second cycle starts
        let _ = r.step(timeout + 1, &Inputs { keyed: true, ..q });
        assert!(!r.failed(), "health line good");
        let _ = r.step(timeout + 2, &Inputs { keyed: true, ..bad });
        assert!(r.failed());
        let _ = r.step(timeout + 3, &Inputs { carrier: true, ..bad });
        assert!(!r.failed(), "a carrier clears it");
        // No cycle tried in this silence: not failed
        let mut r = Receiver::new(0);
        let _ = r.step(timeout + 10, &Inputs { keyed: true, ..bad });
        assert!(!r.failed());
        assert_eq!(Receiver::default(), Receiver::new(0));
    }
}
