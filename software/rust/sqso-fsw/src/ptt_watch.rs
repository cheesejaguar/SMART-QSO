//! The OBC's watch on the exciter's keying line (`rf_control.h`,
//! `rf_control_ptt_supervise()`; SOFTWARE_HAZARD_ANALYSIS G §9.8.1, control
//! C-014-9 of SW-HAZ-014).
//!
//! The beacon path's keying line TNC_PTT bypasses the transponder's
//! heartbeat (EN_OK) and its 120 s key-down timer (SQSO-RF-BRD-001 §8.4);
//! only RF_TX_EN bounds it in hardware, and the TNC's own 5 s timer
//! (TNC-F023) is firmware. The OBC reads the result on PTT_EXCITER
//! (`TX_PERMIT_HW · (TNC_PTT + XPDR_KEY)`, PC7) and the repeat path's
//! XPDR_KEY (PG12). PTT_EXCITER high with XPDR_KEY low and no downlink
//! frame outstanding (the frame's estimated air window, `downlink.rs`) is
//! the TNC keying on its own. Held so for more than [`LIMIT_MS`], the
//! watch trips once: the caller latches TX-OFF (RF_TX_EN low, persisted)
//! and logs the fault. The recovery is the TX-OFF latch's: TX_ENABLE from
//! the ground. The watch re-arms when the condition clears (TX-OFF takes
//! TX_PERMIT_HW and with it PTT_EXCITER low), so a TNC still stuck after
//! TX_ENABLE trips it again [`LIMIT_MS`] later.
//!
//! The repeat path is excluded on purpose: its keying passes the
//! heartbeat and the 120 s timer in hardware, and a transponder over lasts
//! longer than 6 s.
//!
//! @requirement SRS-F168 TX-OFF when PTT_EXCITER reads high for more than 6 s with the repeat path not keyed and no downlink frame outstanding; logged; recovered by TX_ENABLE

/// Longest the exciter may stay keyed with no frame outstanding and the
/// repeat path not keyed, ms (C-014-9; above the TNC's own 5.0 s limit)
pub const LIMIT_MS: u64 = 6_000;

/// The watch's state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PttWatch {
    /// Since when the exciter has been keyed with nothing to explain it, ms
    since_ms: Option<u64>,
    /// Tripped for this episode (re-armed when it clears)
    tripped: bool,
    /// Trips since boot
    pub trips: u32,
}

impl PttWatch {
    /// Nothing seen
    pub const fn new() -> PttWatch {
        PttWatch { since_ms: None, tripped: false, trips: 0 }
    }

    /// One look at the lines at `now_ms`: `ptt` PTT_EXCITER, `repeat`
    /// XPDR_KEY, `frame` a downlink frame outstanding. True once per
    /// episode, when the exciter has been keyed unexplained for more than
    /// [`LIMIT_MS`]: the caller latches TX-OFF and logs it
    pub fn step(&mut self, now_ms: u64, ptt: bool, repeat: bool, frame: bool) -> bool {
        if !ptt || repeat || frame {
            self.since_ms = None;
            self.tripped = false;
            return false;
        }
        let since = *self.since_ms.get_or_insert(now_ms);
        if self.tripped || now_ms.saturating_sub(since) <= LIMIT_MS {
            return false;
        }
        self.tripped = true;
        self.trips = self.trips.saturating_add(1);
        true
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
        assert!(!w.step(1_000, true, false, false));
        assert!(!w.step(1_000 + LIMIT_MS, true, false, false), "at the limit: not yet");
        assert!(w.step(1_001 + LIMIT_MS, true, false, false), "past it: trips");
        assert!(w.tripped());
        assert!(!w.step(20_000, true, false, false), "once per episode");
        assert_eq!(w.trips, 1);
        // TX-OFF drops PTT_EXCITER: re-armed; a TNC still stuck after
        // TX_ENABLE trips it again
        assert!(!w.step(21_000, false, false, false));
        assert!(!w.tripped());
        assert!(!w.step(22_000, true, false, false));
        assert!(w.step(22_001 + LIMIT_MS, true, false, false));
        assert_eq!(w.trips, 2);
    }

    #[test]
    fn a_frame_or_the_repeat_path_explains_the_keying() {
        let mut w = PttWatch::new();
        // Keyed through a frame's window, then on alone: counted from the
        // window's end
        assert!(!w.step(0, true, false, true));
        assert!(!w.step(4_000, true, false, true));
        assert!(!w.step(4_100, true, false, false));
        assert!(!w.step(4_100 + LIMIT_MS, true, false, false));
        assert!(w.step(4_101 + LIMIT_MS, true, false, false));
        // A transponder over keys it for 20 s: never a trip
        let mut w = PttWatch::new();
        for t in (0..20_000).step_by(100) {
            assert!(!w.step(t, true, true, false), "{t}");
        }
        // Gaps restart the count
        let mut w = PttWatch::new();
        for k in 0..10u64 {
            assert!(!w.step(k * 7_000, true, false, false));
            assert!(!w.step(k * 7_000 + 5_000, true, false, false));
            assert!(!w.step(k * 7_000 + 5_500, false, false, false));
        }
        assert_eq!(w.trips, 0);
        assert_eq!(PttWatch::default(), PttWatch::new());
    }
}
