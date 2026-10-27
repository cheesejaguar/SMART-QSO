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
//! once: the radio rail is powered; the loss-of-contact timeout has expired
//! (the contact monitor's count, persisted in FRAM, so a reset does not
//! restart it); the receiver's own health line reports a fault (`RX_FAULT`
//! high: synthesizer out of lock or supply out of its window); and at least
//! one recovery cycle has been tried in the current silence. The health line
//! is active high so that a missing or open line reads "healthy", and the
//! exception can never come from a wiring fault. Any activity clears it.
//! The silence and the cycle are counted from boot (RAM): after a reset the
//! exception waits for a new recovery cycle, [`SILENCE_MS`] later.
//!
//! **Inputs (RID RFB-07).** `RX_CD` and `RX_FAULT` come from the radio
//! board, which the OBC powers through `RADIO_EN`: both are qualified with
//! it (a line from an unpowered board means nothing), and `RX_FAULT` is
//! ignored for [`FAULT_BLANK_MS`] after `RADIO_EN` switches on, while the
//! receiver starts (its TNC heartbeat monostable reads "fault" until the
//! first pulses).
//!
//! **What is not evidence (RID SMA-B11; RF board E §8.6).** A carrier
//! detect high in every pass for more than [`CARRIER_MAX_MS`] (in-band
//! interference, a stuck line) is not activity: it neither clears "known
//! failed" nor postpones the recovery cycle. `RX_FAULT`'s power-on
//! self-test: the line must read high between [`POST_FROM_MS`] and
//! [`POST_TO_MS`] after `RADIO_EN` rises (the TNC holds RX_ALIVE off for
//! 100 ms, TNC-FW-POR); a line seen low throughout that window is
//! unusable (stuck low or open) and its "healthy" is no evidence, until a
//! later power-on passes the test.
//!
//! **The loop query.** Every [`QUERY_PERIOD_MS`] the OBC asks the TNC for
//! its STATUS (KISS SetHardware 0x06/0x01, `sqso_protocol::kiss`); no answer
//! for [`NO_ANSWER_MS`] with the radio on is a receiver-path fault (an open
//! TNC_TXD or TNC_RXD, a USART fault, a TNC that no longer serves KISS) and
//! counts as the receiver's fault for "known failed".
//!
//! @requirement SRS-F152 RX_CD and RX_FAULT qualified with RADIO_EN; RX_FAULT ignored for 1.5 s after RADIO_EN switches on
//! @requirement SRS-F115 Hung-receiver recovery by a brief power cycle, never during a pass
//! @requirement SRS-F116 Evidence that the command receiver has failed

/// `RX_FAULT` is ignored this long after `RADIO_EN` switches on, ms (the
/// line goes low within 1.5 s of the TNC's start, RF board E §8.6)
pub const FAULT_BLANK_MS: u64 = 1_500;
/// The power-on self-test window: `RX_FAULT` reads high from this long after
/// `RADIO_EN` rises, ms
pub const POST_FROM_MS: u64 = 10;
/// ... to this long, ms
pub const POST_TO_MS: u64 = 90;
/// A carrier detect continuously high for longer is not activity, ms
pub const CARRIER_MAX_MS: u64 = 3_600_000;
/// The STATUS loop query's period, ms
pub const QUERY_PERIOD_MS: u64 = 600_000;
/// No STATUS answer for this long with the radio on is a receiver-path
/// fault, ms
pub const NO_ANSWER_MS: u64 = 24 * 3_600_000;
/// No activity for this long, the rail on, before a recovery cycle, ms
pub const SILENCE_MS: u64 = 24 * 3_600_000;
/// No recovery cycle within this long of the last activity, ms (a pass is
/// at most about 13 minutes)
pub const PASS_GUARD_MS: u64 = 15 * 60_000;
/// `RADIO_EN` off this long, ms
pub const CYCLE_OFF_MS: u64 = 2_000;
// A silence long enough for a cycle is longer than any pass, so no cycle
// can fall in a pass in progress
const _: () = assert!(SILENCE_MS > PASS_GUARD_MS);

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
    /// The loss-of-contact timeout has expired (the contact monitor)
    pub contact_lost: bool,
    /// `RADIO_EN` is high (the radio board powered)
    pub radio_en: bool,
    /// A STATUS answer from the TNC arrived since the last pass
    pub answered: bool,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// `RADIO_EN` high since this time, ms
    radio_on_since: Option<u64>,
    /// The power-on self-test under way: `RX_FAULT` sampled in its window,
    /// and seen high
    post: Option<(bool, bool)>,
    /// `RX_FAULT` passed its last self-test (or none has been decided)
    line_usable: bool,
    /// The carrier detect high in every pass since, ms
    carrier_since: Option<u64>,
    /// The last STATUS answer, or the radio's switch-on, ms
    last_answer_ms: u64,
}

impl Default for Receiver {
    fn default() -> Self {
        Receiver::new(0)
    }
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
            radio_on_since: None,
            post: None,
            line_usable: true,
            carrier_since: None,
            last_answer_ms: now_ms,
        }
    }

    /// The inputs qualified with `RADIO_EN`: no carrier and no fault from
    /// an unpowered radio board, and no fault in its first
    /// [`FAULT_BLANK_MS`]
    pub fn qualify(&mut self, now_ms: u64, i: &Inputs) -> Inputs {
        if i.radio_en && self.radio_on_since.is_none() {
            self.post = Some((false, false));
        }
        self.radio_on_since = if i.radio_en { Some(self.radio_on_since.unwrap_or(now_ms)) } else { None };
        let on_ms = self.radio_on_since.map(|t| now_ms.saturating_sub(t));
        self.self_test(on_ms, i.fault);
        let started = on_ms.is_some_and(|t| t >= FAULT_BLANK_MS);
        Inputs { carrier: i.carrier && i.radio_en, fault: i.fault && started && self.line_usable, ..*i }
    }

    /// The power-on self-test of `RX_FAULT`, `on_ms` after `RADIO_EN` rose
    fn self_test(&mut self, on_ms: Option<u64>, fault: bool) {
        let Some(t) = on_ms else {
            self.post = None;
            return;
        };
        let Some((sampled, high)) = self.post else { return };
        if (POST_FROM_MS..=POST_TO_MS).contains(&t) {
            self.post = Some((true, high || fault));
        } else if t > POST_TO_MS {
            // No pass fell in the window: undecided, the last result stands
            if sampled {
                self.line_usable = high;
            }
            self.post = None;
        } else {
            // Before the window
        }
    }

    /// Whether `RX_FAULT` passed its last power-on self-test
    pub const fn line_usable(&self) -> bool {
        self.line_usable
    }

    /// Whether the TNC has not answered a STATUS query for
    /// [`NO_ANSWER_MS`] with the radio on, at `now_ms`
    pub const fn path_fault(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.last_answer_ms) >= NO_ANSWER_MS
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
        self.carrier_since = if i.carrier { Some(self.carrier_since.unwrap_or(now_ms)) } else { None };
        let carrier = self.carrier_since.is_some_and(|t| now_ms.saturating_sub(t) < CARRIER_MAX_MS);
        if i.frame || carrier {
            self.last_activity_ms = now_ms;
            self.cycled_in_silence = false;
            self.failed = false;
        }
        if i.answered || !i.powered || !i.radio_en {
            self.last_answer_ms = now_ms;
        }
        if !i.powered {
            // Nothing heard is evidence of nothing while the rail is off
            self.last_activity_ms = now_ms;
            return Action::None;
        }
        let silent = now_ms.saturating_sub(self.last_activity_ms);
        let fault = i.fault || self.path_fault(now_ms);
        self.failed = self.cycled_in_silence && fault && i.contact_lost;
        let since_cycle = self.last_cycle_ms.map_or(u64::MAX, |t| now_ms.saturating_sub(t));
        if silent >= SILENCE_MS && since_cycle >= SILENCE_MS && !i.keyed {
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

    /// A working TNC answers its STATUS queries
    fn quiet() -> Inputs {
        Inputs { powered: true, radio_en: true, answered: true, ..Inputs::default() }
    }

    #[test]
    fn a_carrier_stuck_for_an_hour_is_not_activity() {
        let mut r = Receiver::new(0);
        let cd = Inputs { carrier: true, ..quiet() };
        for t in (0..=SILENCE_MS + 2 * H).step_by(60_000) {
            if r.step(t, &cd) == Action::PowerOff {
                assert!(t >= SILENCE_MS, "{t}");
                assert_eq!(r.cycles, 1);
                return;
            }
        }
        panic!("the stuck carrier postponed the cycle");
    }

    #[test]
    fn no_status_answer_for_a_day_is_a_path_fault() {
        let mut r = Receiver::new(0);
        let deaf = Inputs { answered: false, contact_lost: true, ..quiet() };
        assert!(!r.path_fault(NO_ANSWER_MS - 1));
        assert_eq!(r.step(SILENCE_MS, &deaf), Action::PowerOff);
        let _ = r.step(SILENCE_MS + CYCLE_OFF_MS, &deaf);
        let _ = r.step(SILENCE_MS + CYCLE_OFF_MS + 1, &Inputs { keyed: true, ..deaf });
        assert!(r.path_fault(SILENCE_MS + CYCLE_OFF_MS + 1));
        assert!(r.failed(), "health line good, but the TNC does not answer");
        let _ = r.step(SILENCE_MS + CYCLE_OFF_MS + 2, &Inputs { keyed: true, answered: true, ..deaf });
        assert!(!r.path_fault(SILENCE_MS + CYCLE_OFF_MS + 2));
        assert!(!r.failed());
        // The radio off: the count waits
        let mut r = Receiver::new(0);
        let _ = r.step(NO_ANSWER_MS, &Inputs { radio_en: false, ..deaf });
        assert!(!r.path_fault(NO_ANSWER_MS + 1));
    }

    #[test]
    fn the_fault_line_self_test_at_power_on() {
        let mut r = Receiver::new(0);
        assert!(r.line_usable());
        // High in the window: usable
        let on = Inputs { fault: true, ..quiet() };
        let _ = r.qualify(1_000, &on);
        let _ = r.qualify(1_050, &on);
        let _ = r.qualify(1_100, &Inputs { fault: false, ..on });
        assert!(r.line_usable());
        // Off, then on with the line low throughout the window: unusable,
        // and its fault no longer counts
        let _ = r.qualify(5_000, &Inputs { radio_en: false, ..on });
        let low = quiet();
        let _ = r.qualify(6_000, &low);
        let _ = r.qualify(6_040, &low);
        let _ = r.qualify(6_200, &low);
        assert!(!r.line_usable());
        assert!(!r.qualify(9_000, &on).fault, "an unusable line is no evidence");
        // A switch-on with no pass in the window decides nothing
        let _ = r.qualify(10_000, &Inputs { radio_en: false, ..on });
        let _ = r.qualify(10_005, &on);
        let _ = r.qualify(10_200, &on);
        assert!(!r.line_usable());
        // A good self-test makes it usable again
        let _ = r.qualify(20_000, &Inputs { radio_en: false, ..on });
        let _ = r.qualify(20_001, &on);
        let _ = r.qualify(20_030, &on);
        let _ = r.qualify(20_100, &low);
        assert!(r.line_usable());
    }

    #[test]
    fn the_lines_are_qualified_with_radio_en() {
        let mut r = Receiver::new(0);
        let lines = Inputs { carrier: true, fault: true, ..quiet() };
        let off = r.qualify(0, &Inputs { radio_en: false, ..lines });
        assert!(!off.carrier, "unpowered board");
        assert!(!off.fault, "unpowered board");
        let q = r.qualify(100, &lines);
        assert!(q.carrier, "fault blanked while the receiver starts");
        assert!(!q.fault, "fault blanked while the receiver starts");
        assert!(!r.qualify(100 + FAULT_BLANK_MS - 1, &lines).fault);
        assert!(r.qualify(100 + FAULT_BLANK_MS, &lines).fault);
        let _ = r.qualify(2000, &Inputs { radio_en: false, ..lines });
        assert!(!r.qualify(2020, &lines).fault, "blanked again after a switch-on");
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
        let q = quiet();
        let bad = Inputs { fault: true, ..q };
        assert_eq!(r.step(SILENCE_MS, &bad), Action::PowerOff);
        let _ = r.step(SILENCE_MS + CYCLE_OFF_MS, &bad);
        assert!(!r.failed(), "contact not yet lost");
        // Keyed from here on, so that no second cycle starts
        let t = 2 * DAY;
        let _ = r.step(t + 1, &Inputs { keyed: true, contact_lost: true, ..q });
        assert!(!r.failed(), "health line good");
        let _ = r.step(t + 2, &Inputs { keyed: true, contact_lost: true, ..bad });
        assert!(r.failed());
        let _ = r.step(t + 3, &Inputs { carrier: true, contact_lost: true, ..bad });
        assert!(!r.failed(), "a carrier clears it");
        // No cycle tried in this silence (a reset): not failed, until one is
        let mut r = Receiver::new(t);
        let lost = Inputs { contact_lost: true, ..bad };
        let _ = r.step(t + 10, &Inputs { keyed: true, ..lost });
        assert!(!r.failed());
        assert_eq!(r.step(t + SILENCE_MS, &lost), Action::PowerOff);
        assert!(!r.failed(), "the cycle is under way");
        assert_eq!(r.step(t + SILENCE_MS + CYCLE_OFF_MS, &lost), Action::PowerOn);
        let _ = r.step(t + SILENCE_MS + CYCLE_OFF_MS + 1, &lost);
        assert!(r.failed(), "known failed a day after the reset");
        assert_eq!(Receiver::default(), Receiver::new(0));
    }
}
