//! The OBC's watch on the exciter's keying line (`rf_control.h`,
//! `rf_control_ptt_supervise()`; SOFTWARE_HAZARD_ANALYSIS §9.8.1, control
//! C-014-9 of SW-HAZ-014).
//!
//! The beacon path's keying line TNC_PTT bypasses the transponder's
//! heartbeat (EN_OK) and its 120 s key-down timer (SQSO-RF-BRD-001 §8.4);
//! only RF_TX_EN bounds it in hardware, and the TNC's own 5 s timer
//! (TNC-F023) is firmware. The OBC reads the result on PTT_EXCITER
//! (`TX_PERMIT_HW · (TNC_PTT + XPDR_KEY)`, PC7) and the repeat path's
//! XPDR_KEY (`EN_OK · TONE_OK · NOT_TOT`, PG12). The watch looks only while
//! PTT_EXCITER is high (the exciter keyed) and latches TX-OFF (RF_TX_EN
//! low, persisted; the caller logs it) on any of three faults:
//!
//! - **Unexplained keying** ([`Trip::Unexplained`]): PTT_EXCITER high
//!   with XPDR_KEY low and no downlink frame outstanding (the frame's
//!   estimated air window, `downlink.rs`) for more than [`LIMIT_MS`] in
//!   all, counted across the gaps between transponder overs: only
//!   PTT_EXCITER seen low (which a TNC_PTT stuck high never allows) clears
//!   the count, so a stuck TNC_PTT is found in a busy transponder session
//!   too (RID P3F-A-02), within [`LIMIT_MS`] of unexplained gaps.
//! - **Repeat path keyed without the heartbeat**
//!   ([`Trip::RepeatWithoutHeartbeat`]): XPDR_KEY explains the keying only
//!   while the flight software issues the transponder heartbeat; XPDR_KEY
//!   high for more than [`REPEAT_DROPOUT_MS`] after the heartbeat stopped
//!   (the radio board unkeys within 59 ms of the last edge) is a stuck key
//!   gate or line.
//! - **Repeat over too long** ([`Trip::RepeatTooLong`]): XPDR_KEY high
//!   without a break for more than [`REPEAT_MAX_MS`] (the 120 ± 4.6 s
//!   key-down timer and a margin) is past the hardware timer: a stuck key
//!   gate.
//!
//! The recovery is the TX-OFF latch's: TX_ENABLE from the ground. The watch
//! re-arms when PTT_EXCITER reads low (TX-OFF takes TX_PERMIT_HW and with it
//! PTT_EXCITER low), so a fault still present after TX_ENABLE trips it
//! again.
//!
//! @requirement SRS-F168 TX-OFF when PTT_EXCITER reads high for more than 6 s in all with the repeat path not keyed and no downlink frame outstanding (cleared only by PTT_EXCITER low), when XPDR_KEY reads high more than 1 s after the transponder heartbeat stopped, or for more than 130 s without a break; logged; recovered by TX_ENABLE

/// Longest the exciter may stay keyed with no frame outstanding and the
/// repeat path not keyed, in all until it reads low, ms (C-014-9; above the
/// TNC's own 5.0 s limit)
pub const LIMIT_MS: u64 = 6_000;
/// XPDR_KEY may stay high this long after the transponder heartbeat
/// stopped, ms (the 40 ms lease, the radio board's 59 ms unkey, and the
/// passes between them)
pub const REPEAT_DROPOUT_MS: u64 = 1_000;
/// Longest one repeat over may key the exciter, ms (the 120 ± 4.6 s
/// key-down timer, SQSO-RF-BRD-001 §8.2, and 5.4 s of margin)
pub const REPEAT_MAX_MS: u64 = 130_000;

/// Why the watch tripped
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trip {
    /// Keyed with nothing to explain it for more than [`LIMIT_MS`]
    Unexplained,
    /// XPDR_KEY high with the heartbeat stopped
    RepeatWithoutHeartbeat,
    /// XPDR_KEY high for more than [`REPEAT_MAX_MS`]
    RepeatTooLong,
}

/// The watch's state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PttWatch {
    /// The last look while keyed, ms, and whether it was unexplained
    last: Option<(u64, bool)>,
    /// Unexplained keyed time since PTT_EXCITER last read low, ms
    unexplained_ms: u64,
    /// XPDR_KEY high without a break since, ms
    repeat_since: Option<u64>,
    /// The heartbeat stopped at, ms (`None`: issued)
    heartbeat_off_since: Option<u64>,
    /// Tripped for this episode (re-armed when PTT_EXCITER reads low)
    tripped: bool,
    /// Trips since boot
    pub trips: u32,
}

impl PttWatch {
    /// Nothing seen
    pub const fn new() -> PttWatch {
        PttWatch {
            last: None,
            unexplained_ms: 0,
            repeat_since: None,
            heartbeat_off_since: None,
            tripped: false,
            trips: 0,
        }
    }

    /// One look at the lines at `now_ms`: `ptt` PTT_EXCITER, `repeat`
    /// XPDR_KEY, `frame` a downlink frame outstanding, `heartbeat` the
    /// transponder heartbeat issued. The fault once per episode: the caller
    /// latches TX-OFF and logs it
    pub fn step(&mut self, now_ms: u64, ptt: bool, repeat: bool, frame: bool, heartbeat: bool) -> Option<Trip> {
        self.heartbeat_off_since = if heartbeat { None } else { Some(self.heartbeat_off_since.unwrap_or(now_ms)) };
        if !ptt {
            self.last = None;
            self.unexplained_ms = 0;
            self.repeat_since = None;
            self.tripped = false;
            return None;
        }
        // The time since the last look counts if that look was unexplained
        if let Some((t, true)) = self.last {
            self.unexplained_ms = self.unexplained_ms.saturating_add(now_ms.saturating_sub(t));
        }
        self.repeat_since = if repeat { Some(self.repeat_since.unwrap_or(now_ms)) } else { None };
        let unexplained = !repeat && !frame;
        self.last = Some((now_ms, unexplained));
        let fault = if self.unexplained_ms > LIMIT_MS {
            Some(Trip::Unexplained)
        } else if repeat && self.heartbeat_off_since.is_some_and(|t| now_ms.saturating_sub(t) > REPEAT_DROPOUT_MS) {
            Some(Trip::RepeatWithoutHeartbeat)
        } else if self.repeat_since.is_some_and(|t| now_ms.saturating_sub(t) > REPEAT_MAX_MS) {
            Some(Trip::RepeatTooLong)
        } else {
            None
        };
        if self.tripped || fault.is_none() {
            return None;
        }
        self.tripped = true;
        self.trips = self.trips.saturating_add(1);
        fault
    }

    /// Whether this episode has tripped
    pub const fn tripped(&self) -> bool {
        self.tripped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trips_once_after_six_seconds_unexplained() {
        let mut w = PttWatch::new();
        assert_eq!(w.step(1_000, true, false, false, false), None);
        assert_eq!(w.step(1_000 + LIMIT_MS, true, false, false, false), None, "at the limit: not yet");
        assert_eq!(w.step(1_001 + LIMIT_MS, true, false, false, false), Some(Trip::Unexplained), "past it: trips");
        assert!(w.tripped());
        assert_eq!(w.step(20_000, true, false, false, false), None, "once per episode");
        assert_eq!(w.trips, 1);
        // TX-OFF drops PTT_EXCITER: re-armed; a TNC still stuck after
        // TX_ENABLE trips it again
        assert_eq!(w.step(21_000, false, false, false, false), None);
        assert!(!w.tripped());
        assert_eq!(w.step(22_000, true, false, false, false), None);
        assert_eq!(w.step(22_001 + LIMIT_MS, true, false, false, false), Some(Trip::Unexplained));
        assert_eq!(w.trips, 2);
    }

    #[test]
    fn a_frame_or_the_repeat_path_explains_the_keying() {
        let mut w = PttWatch::new();
        // Keyed through a frame's window, then on alone: counted from the
        // window's end
        assert_eq!(w.step(0, true, false, true, false), None);
        assert_eq!(w.step(4_000, true, false, true, false), None);
        assert_eq!(w.step(4_100, true, false, false, false), None);
        assert_eq!(w.step(4_100 + LIMIT_MS, true, false, false, false), None);
        assert_eq!(w.step(4_101 + LIMIT_MS, true, false, false, false), Some(Trip::Unexplained));
        // A transponder over keys it for 100 s with the heartbeat issued:
        // never a trip
        let mut w = PttWatch::new();
        for t in (0..100_000).step_by(100) {
            assert_eq!(w.step(t, true, true, false, true), None, "{t}");
        }
        // Gaps in which the exciter reads low clear the count
        let mut w = PttWatch::new();
        for k in 0..10u64 {
            assert_eq!(w.step(k * 7_000, true, false, false, false), None);
            assert_eq!(w.step(k * 7_000 + 5_000, true, false, false, false), None);
            assert_eq!(w.step(k * 7_000 + 5_500, false, false, false, false), None);
        }
        assert_eq!(w.trips, 0);
        assert_eq!(PttWatch::default(), PttWatch::new());
    }

    /// RID P3F-A-02: a TNC_PTT stuck high in a busy transponder session.
    /// Overs 30 s long with 2 s gaps: the exciter never reads low, and the
    /// unexplained gaps add up to a trip at the fourth gap; with the TNC
    /// healthy the exciter reads low in each gap and nothing trips
    #[test]
    fn a_stuck_tnc_is_found_between_overs() {
        for stuck in [true, false] {
            let mut w = PttWatch::new();
            let mut trip = None;
            'session: for over in 0..20u64 {
                let start = over * 32_000;
                for t in (start..start + 30_000).step_by(100) {
                    if let Some(f) = w.step(t, true, true, false, true) {
                        trip = Some((f, over));
                        break 'session;
                    }
                }
                for t in (start + 30_000..start + 32_000).step_by(100) {
                    if let Some(f) = w.step(t, stuck, false, false, true) {
                        trip = Some((f, over));
                        break 'session;
                    }
                }
            }
            if stuck {
                assert_eq!(trip, Some((Trip::Unexplained, 3)), "found at the fourth gap");
            } else {
                assert_eq!(trip, None);
            }
        }
    }

    /// RID P3F-A-02: XPDR_KEY explains the keying only with the heartbeat
    /// issued, and only for one over's length
    #[test]
    fn the_repeat_path_is_bounded() {
        // The heartbeat stops: the radio board unkeys within REPEAT_DROPOUT_MS
        let mut w = PttWatch::new();
        assert_eq!(w.step(0, true, true, false, true), None);
        assert_eq!(w.step(100, true, true, false, false), None);
        assert_eq!(w.step(100 + REPEAT_DROPOUT_MS, true, true, false, false), None);
        assert_eq!(w.step(101 + REPEAT_DROPOUT_MS, true, true, false, false), Some(Trip::RepeatWithoutHeartbeat));
        // ... a stuck key with the heartbeat never issued
        let mut w = PttWatch::new();
        assert_eq!(w.step(5_000, true, true, false, false), None);
        assert_eq!(w.step(6_001, true, true, false, false), Some(Trip::RepeatWithoutHeartbeat));
        // An over within the key-down timer, and one past it
        let mut w = PttWatch::new();
        assert_eq!(w.step(0, true, true, false, true), None);
        assert_eq!(w.step(REPEAT_MAX_MS, true, true, false, true), None);
        assert_eq!(w.step(REPEAT_MAX_MS + 1, true, true, false, true), Some(Trip::RepeatTooLong));
        assert_eq!(w.step(REPEAT_MAX_MS + 500, true, true, false, true), None, "once");
        // XPDR_KEY dropping restarts the over's count
        let mut w = PttWatch::new();
        for over in 0..5u64 {
            let start = over * 121_000;
            assert_eq!(w.step(start, true, true, false, true), None);
            assert_eq!(w.step(start + 120_000, true, true, false, true), None);
            assert_eq!(w.step(start + 120_500, false, false, false, true), None);
        }
        assert_eq!(w.trips, 0);
        // TX-OFF drops PTT_EXCITER: XPDR_KEY alone is no hazard
        let mut w = PttWatch::new();
        assert_eq!(w.step(0, false, true, false, false), None);
        assert_eq!(w.step(10_000, false, true, false, false), None);
        assert!(!w.tripped());
    }
}
