//! The FRAM's health and the degraded mode without it (RID SMA-B10).
//!
//! The critical state (the TX-OFF latch, the operations record, the
//! passivation record) lives in the FRAM, and the one-minute check of it
//! (AV-07, SRS-F133) settles an unreadable record toward the safe state.
//! With the FRAM dead that rule silences the transmitter for good: TX_ENABLE
//! holds for at most a minute before the check forces TX-OFF again, and the
//! commissioning hold returns at every check.
//!
//! The safe default stays. The check's reads are counted: after
//! [`FAILURES_TO_DECLARE`] consecutive failed checks the FRAM is declared
//! failed (HK1 `FR`), and only then does the ground's FRAM_BYPASS (argument
//! [`BYPASS_CODE`]) put the software in the degraded mode: the RAM copies,
//! each under a CRC, are authoritative; the one-minute check verifies their
//! CRCs (a mismatch, an upset, settles toward the safe state as before) and
//! writes them to the FRAM, best effort, instead of reading it. TX-OFF by
//! command still latches at once, in RAM. The mode ends by itself when
//! [`READS_TO_RECOVER`] consecutive checks read back what was written (the
//! FRAM then holds the RAM state again) and at every reset: the boot
//! defaults (TX-OFF, commissioning hold) return and the ground repeats
//! FRAM_BYPASS, TX_ENABLE and COMMISSIONING_RELEASE.
//!
//! @requirement SRS-F160 FRAM failed after three consecutive failed one-minute checks; FRAM_BYPASS then makes the CRC-protected RAM copies of the critical state authoritative until the FRAM reads back again or the next reset

use sqso_protocol::checksum::crc32;

/// Consecutive failed one-minute checks that declare the FRAM failed
pub const FAILURES_TO_DECLARE: u32 = 3;
/// Consecutive good checks in the degraded mode that end it
pub const READS_TO_RECOVER: u32 = 3;
/// FRAM_BYPASS's argument
pub const BYPASS_CODE: &[u8; 4] = b"FRM!";

/// HK1 `FR` state: the FRAM works
pub const STATE_OK: u8 = 0;
/// Declared failed, the safe defaults in force
pub const STATE_FAILED: u8 = 1;
/// Degraded mode: the RAM copies authoritative
pub const STATE_DEGRADED: u8 = 2;

/// Why FRAM_BYPASS was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not the argument [`BYPASS_CODE`]
    WrongCode,
    /// The FRAM has not been declared failed
    NotFailed,
}

/// The FRAM's health
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FramHealth {
    run: u32,
    good_run: u32,
    /// Declared failed
    pub failed: bool,
    /// The degraded mode is in force
    pub degraded: bool,
    /// Failed checks since boot
    pub failures: u32,
}

impl FramHealth {
    /// At boot: healthy
    pub const fn new() -> FramHealth {
        FramHealth { run: 0, good_run: 0, failed: false, degraded: false, failures: 0 }
    }

    /// One check's outcome: whether every critical record could be read (in
    /// the degraded mode: written and read back)
    pub fn note(&mut self, ok: bool) {
        if ok {
            self.run = 0;
            self.good_run = self.good_run.saturating_add(1);
            if self.degraded && self.good_run >= READS_TO_RECOVER {
                self.degraded = false;
                self.failed = false;
            }
            return;
        }
        self.good_run = 0;
        self.failures = self.failures.saturating_add(1);
        self.run = self.run.saturating_add(1);
        if self.run >= FAILURES_TO_DECLARE {
            self.failed = true;
        }
    }

    /// FRAM_BYPASS with its argument
    pub fn bypass(&mut self, arg: &[u8]) -> Result<(), Refusal> {
        if arg != BYPASS_CODE {
            return Err(Refusal::WrongCode);
        }
        if !self.failed {
            return Err(Refusal::NotFailed);
        }
        self.degraded = true;
        self.good_run = 0;
        Ok(())
    }

    /// HK1 `FR` state ([`STATE_OK`], [`STATE_FAILED`], [`STATE_DEGRADED`])
    pub const fn state(&self) -> u8 {
        if self.degraded {
            STATE_DEGRADED
        } else if self.failed {
            STATE_FAILED
        } else {
            STATE_OK
        }
    }
}

/// A RAM copy under a CRC (the degraded mode's record)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shadow {
    crc: u32,
}

impl Shadow {
    /// The CRC of `state`
    pub fn of(state: &[u8]) -> Shadow {
        Shadow { crc: crc32(state) }
    }

    /// Whether `state` is the value the shadow was taken of
    pub fn holds(&self, state: &[u8]) -> bool {
        self.crc == crc32(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_failed_checks_declare_and_one_good_resets_the_run() {
        let mut h = FramHealth::new();
        h.note(false);
        h.note(false);
        h.note(true);
        h.note(false);
        h.note(false);
        assert!(!h.failed);
        assert_eq!(h.state(), STATE_OK);
        h.note(false);
        assert!(h.failed);
        assert_eq!(h.state(), STATE_FAILED);
        assert_eq!(h.failures, 5);
        // A good check does not undeclare a failed FRAM outside the degraded mode
        h.note(true);
        assert!(h.failed);
    }

    #[test]
    fn bypass_only_when_failed_and_with_the_code() {
        let mut h = FramHealth::new();
        assert_eq!(h.bypass(BYPASS_CODE), Err(Refusal::NotFailed));
        for _ in 0..FAILURES_TO_DECLARE {
            h.note(false);
        }
        assert_eq!(h.bypass(b"FRM?"), Err(Refusal::WrongCode));
        assert_eq!(h.bypass(b"FRM"), Err(Refusal::WrongCode));
        assert!(!h.degraded);
        assert_eq!(h.bypass(BYPASS_CODE), Ok(()));
        assert_eq!(h.state(), STATE_DEGRADED);
    }

    #[test]
    fn the_degraded_mode_ends_when_the_fram_reads_back() {
        let mut h = FramHealth::new();
        for _ in 0..FAILURES_TO_DECLARE {
            h.note(false);
        }
        assert_eq!(h.bypass(BYPASS_CODE), Ok(()));
        h.note(true);
        h.note(true);
        h.note(false);
        assert!(h.degraded);
        for _ in 0..READS_TO_RECOVER - 1 {
            h.note(true);
        }
        assert!(h.degraded);
        h.note(true);
        assert!(!h.degraded);
        assert!(!h.failed);
        assert_eq!(h.state(), STATE_OK);
        assert_eq!(FramHealth::default(), FramHealth::new());
    }

    #[test]
    fn the_shadow_detects_a_change() {
        let s = Shadow::of(&[1, 0, 2, 0]);
        assert!(s.holds(&[1, 0, 2, 0]));
        assert!(!s.holds(&[0, 0, 2, 0]));
    }
}
