//! The EPS controller's supervision registers (SQSO-ICD-001 1.6 §4.6;
//! RIDs SMA-B09, SMA-B06, SMA-B17).
//!
//! - **OBC_HEARTBEAT** (0x8C, W): the flight software writes
//!   [`HEARTBEAT_VALUE`] every [`HEARTBEAT_PERIOD_MS`] (the controller wants
//!   one at least every 60 s). After 600 s without one the controller
//!   power-cycles the OBC rail SW10 once (≤ 3 cycles in 24 h, then it stops
//!   supervising: EPS_SUPV bit 7); no command disables it.
//! - **ANTS_PWR** (0x8D, W): [`ANTS_PWR_ON`] raises the AntS supply for at
//!   most 30 min, in OR with DEPLOY_EN; the deployment's retry path when the
//!   AntS does not answer with DEPLOY_EN high; any other value lowers it.
//! - **TRK_DISABLE** (0x8E, W, 2 bytes): [`TRK_KEY`] then the mask of
//!   trackers T1–T4 to disable (EPS_DISABLE_TRACKERS).
//! - **EPS_SUPV** (0x8F, R): bits 0–3 trackers disabled, bit 4 ANTS_EN from
//!   the controller, bits 5–6 OBC power cycles in 24 h, bit 7 supervision
//!   stopped (HK2 `SV`).
//!
//! @requirement SRS-F162 OBC heartbeat to the EPS controller (0x8C, 0x5A) every 30 s; EPS_SUPV in HK2; ANTS_PWR as the AntS supply retry path; EPS_DISABLE_TRACKERS writes TRK_DISABLE with its key

/// `OBC_HEARTBEAT`
pub const REG_HEARTBEAT: u8 = 0x8C;
/// The heartbeat (any other value is not one)
pub const HEARTBEAT_VALUE: u8 = 0x5A;
/// Heartbeat period, ms (half the controller's 60 s)
pub const HEARTBEAT_PERIOD_MS: u64 = 30_000;
/// `ANTS_PWR`
pub const REG_ANTS_PWR: u8 = 0x8D;
/// AntS supply on (≤ 30 min); any other value off
pub const ANTS_PWR_ON: u8 = 0xA6;
/// AntS supply off
pub const ANTS_PWR_OFF: u8 = 0x00;
/// `TRK_DISABLE`
pub const REG_TRK_DISABLE: u8 = 0x8E;
/// `TRK_DISABLE`'s key byte
pub const TRK_KEY: u8 = 0x96;
/// Tracker mask bits (T1–T4)
pub const TRK_MASK: u8 = 0x0F;
/// `EPS_SUPV`
pub const REG_SUPV: u8 = 0x8F;
/// `CHG_INHIBIT` bit 6: NTC1 implausible (its window released to the AFE)
pub const CHG_INHIBIT_NTC1_IMPLAUSIBLE: u8 = 0x40;

/// `EPS_SUPV` decoded
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Supv {
    /// Trackers disabled (bit n: T(n+1))
    pub trackers_disabled: u8,
    /// The controller holds the AntS supply on
    pub ants_on: bool,
    /// OBC power cycles in the last 24 h (0–3)
    pub obc_cycles: u8,
    /// Heartbeat supervision stopped (the limit reached)
    pub stopped: bool,
}

impl Supv {
    /// Decode the register
    pub const fn decode(b: u8) -> Supv {
        Supv {
            trackers_disabled: b & TRK_MASK,
            ants_on: b & 0x10 != 0,
            obc_cycles: (b >> 5) & 0x03,
            stopped: b & 0x80 != 0,
        }
    }
}

/// The TRK_DISABLE write for a mask (`None`: bits beyond T1–T4)
pub const fn trk_disable(mask: u8) -> Option<[u8; 3]> {
    if mask & !TRK_MASK != 0 { None } else { Some([REG_TRK_DISABLE, TRK_KEY, mask]) }
}

/// When the heartbeat is due
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Heartbeat {
    last_ms: Option<u64>,
    /// Writes the controller did not acknowledge, since boot
    pub failures: u32,
}

impl Heartbeat {
    /// At boot: due at once
    pub const fn new() -> Heartbeat {
        Heartbeat { last_ms: None, failures: 0 }
    }

    /// Whether a heartbeat is due at `now_ms`
    pub fn due(&self, now_ms: u64) -> bool {
        self.last_ms.is_none_or(|t| now_ms.saturating_sub(t) >= HEARTBEAT_PERIOD_MS)
    }

    /// A heartbeat was written at `now_ms` (`acked`: the controller took it;
    /// a refused one is tried again at the next pass)
    pub fn written(&mut self, now_ms: u64, acked: bool) {
        if acked {
            self.last_ms = Some(now_ms);
        } else {
            self.failures = self.failures.saturating_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_every_thirty_seconds_and_a_refusal_retries() {
        let mut h = Heartbeat::new();
        assert!(h.due(0));
        h.written(0, true);
        assert!(!h.due(HEARTBEAT_PERIOD_MS - 1));
        assert!(h.due(HEARTBEAT_PERIOD_MS));
        h.written(HEARTBEAT_PERIOD_MS, false);
        assert!(h.due(HEARTBEAT_PERIOD_MS + 20));
        assert_eq!(h.failures, 1);
        assert_eq!(Heartbeat::default(), Heartbeat::new());
        const { assert!(HEARTBEAT_PERIOD_MS * 2 <= 60_000) };
    }

    #[test]
    fn supv_and_the_tracker_write() {
        let s = Supv::decode(0b1101_0101);
        assert_eq!(s, Supv { trackers_disabled: 0b0101, ants_on: true, obc_cycles: 2, stopped: true });
        assert_eq!(Supv::decode(0), Supv::default());
        assert_eq!(trk_disable(0b1001), Some([0x8E, 0x96, 0b1001]));
        assert_eq!(trk_disable(0x10), None);
    }
}
