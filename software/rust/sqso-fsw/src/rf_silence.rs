//! 45-minute post-deployment RF silence (CDS Rev 14.1 §2.4.5, §2.3.2.4;
//! `rf_silence.h`).
//!
//! No RF signal until [`DURATION_S`] after deployment; while the silence is
//! in force the whole radio is unpowered. The deployment switches are power
//! inhibits, so deployment and any switch toggle reach the OBC as a
//! power-on reset. Therefore:
//!  - power-on, brownout, pin or unknown reset: count from 0;
//!  - watchdog or software reset: resume from the persisted count (which
//!    lags by up to [`PERSIST_INTERVAL_S`], so the silence only lengthens);
//!  - a stowed switch seen while powered: count from 0, until the silence
//!    has been served;
//!  - blank or corrupt record: count from 0.
//!
//! Once complete, the silence is latched: it persists across later resets,
//! and a sense line reading stowed afterwards does not restart it (LV-15).
//! In orbit that reading can only be a failed switch or sense line, and a
//! real re-stow removes power, which the boot handles; restarting on it
//! would let one failed line silence the station indefinitely. GSE ARM
//! resets it for flight; ARMED does not count while powered.
//!
//! @requirement SYS-224 No RF signal earlier than 45 minutes after deployment; switch toggle resets the timer
//! @requirement SRS-F087 The software shall latch the served 45-minute RF silence, so that a sense line reading stowed afterwards does not restart it

use crate::contact_monitor::carry_seconds;
use crate::nvm::{self, Flash, RecordStatus};

/// Silence after deployment, s
pub const DURATION_S: u32 = 45 * 60;
/// Count persistence interval, s
pub const PERSIST_INTERVAL_S: u32 = 60;
/// Record ("RFSI"): state (4) | elapsed seconds (4)
pub const RECORD_MAGIC: u32 = 0x5246_5349;

/// `RfSilenceState_t`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum State {
    /// Set by GSE ARM; the count starts at the next boot
    Armed = 1,
    /// Silent, counting toward [`DURATION_S`]
    Counting = 2,
    /// Silence served; the radio may operate
    Complete = 3,
}

/// Why the processor last reset (`HalResetCause_t`), as far as the silence
/// cares
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reset {
    /// Watchdog or software reset: power stayed on, the count resumes
    PowerKept,
    /// Power-on, brownout, pin or unknown: the count restarts
    PowerLost,
}

/// `HAL_RESET_WATCHDOG`
pub const HAL_RESET_WATCHDOG: u32 = 2;
/// `HAL_RESET_SOFTWARE`
pub const HAL_RESET_SOFTWARE: u32 = 3;

impl Reset {
    /// From a `HalResetCause_t`: only a watchdog or software reset kept
    /// power (the deployment switches are power inhibits)
    pub const fn from_hal(cause: u32) -> Reset {
        match cause {
            HAL_RESET_WATCHDOG | HAL_RESET_SOFTWARE => Reset::PowerKept,
            _ => Reset::PowerLost,
        }
    }
}

/// The flash, the separation switch, the radio inhibit and fault reports
pub trait SilenceIo: Flash {
    /// Configure the separation-switch input
    fn switch_init(&mut self);
    /// Whether the separation switches read stowed
    fn stowed(&mut self) -> bool;
    /// Apply or lift the silence inhibit (`RF_INHIBIT_RF_SILENCE`)
    fn set_inhibit(&mut self, active: bool);
    /// Log an event
    fn report(&mut self, report: Report);
}

/// Events for the fault log
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// Boot: the record could not be read; counting from 0
    Unreadable,
    /// The silence was served: radio enabled
    Complete,
}

/// The ARM record could not be persisted
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IoError;

/// Module state
#[derive(Clone, Copy, Debug)]
pub struct RfSilence {
    region: u32,
    state: State,
    elapsed_s: u32,
    unpersisted_s: u32,
    remainder_ms: u32,
    initialized: bool,
}

impl RfSilence {
    /// Before init: silent (counting), nothing counted
    pub const fn new(region: u32) -> Self {
        RfSilence {
            region,
            state: State::Counting,
            elapsed_s: 0,
            unpersisted_s: 0,
            remainder_ms: 0,
            initialized: false,
        }
    }

    /// The persisted record for `state` and `elapsed_s` (the simulation's
    /// start-up hook writes one; `sqso_ffi::executive::run_host`)
    pub fn record(state: State, elapsed_s: u32) -> [u8; 8] {
        let mut rec = [0u8; 8];
        let (s, e) = rec.split_at_mut(4);
        s.copy_from_slice(&(state as u32).to_le_bytes());
        e.copy_from_slice(&elapsed_s.to_le_bytes());
        rec
    }

    fn save<F: Flash>(&mut self, flash: &mut F) {
        if nvm::save(flash, self.region, 0, RECORD_MAGIC, &Self::record(self.state, self.elapsed_s)).is_ok() {
            self.unpersisted_s = 0;
        }
    }

    fn apply<IO: SilenceIo>(&self, io: &mut IO) {
        io.set_inhibit(self.state != State::Complete);
    }

    fn restart_count(&mut self) {
        self.state = State::Counting;
        self.elapsed_s = 0;
        self.remainder_ms = 0;
    }

    /// Load the state, apply the reset-cause rules, persist, and set the
    /// inhibit
    pub fn init<IO: SilenceIo>(&mut self, reset: Reset, io: &mut IO) {
        io.switch_init();
        self.unpersisted_s = 0;
        self.restart_count();

        let mut rec = [0u8; 8];
        match nvm::load(io, self.region, 0, RECORD_MAGIC, &mut rec) {
            Ok(RecordStatus::Valid) => {
                let [s0, s1, s2, s3, e0, e1, e2, e3] = rec;
                let (state, elapsed_s) = (u32::from_le_bytes([s0, s1, s2, s3]), u32::from_le_bytes([e0, e1, e2, e3]));
                if state == State::Complete as u32 {
                    self.state = State::Complete;
                } else if state == State::Counting as u32 && reset == Reset::PowerKept && elapsed_s < DURATION_S {
                    self.elapsed_s = elapsed_s;
                } else {
                    // ARMED, power removed, or unrecognized: count from 0
                }
            }
            Ok(RecordStatus::Corrupt) | Err(_) => io.report(Report::Unreadable),
            Ok(RecordStatus::Blank) => {} // first boot: count from 0
        }

        self.save(io);
        self.initialized = true;
        self.apply(io);
    }

    /// Advance the count by elapsed time; restart it while stowed
    pub fn tick<IO: SilenceIo>(&mut self, elapsed_ms: u32, io: &mut IO) {
        if !self.initialized || self.state == State::Complete {
            return; // the served silence is latched (LV-15)
        }
        if io.stowed() {
            // CDS 2.3.2.4: a switch toggle restarts the timer (ARMED or counting)
            if self.state != State::Counting || self.elapsed_s != 0 {
                self.restart_count();
                self.save(io);
                self.apply(io);
            }
            return;
        }
        if self.state != State::Counting {
            return; // COMPLETE, or ARMED (waits for the next boot)
        }

        let (seconds, remainder) = carry_seconds(self.remainder_ms, elapsed_ms);
        self.remainder_ms = remainder;
        self.elapsed_s = self.elapsed_s.saturating_add(seconds);
        self.unpersisted_s = self.unpersisted_s.saturating_add(seconds);

        if self.elapsed_s >= DURATION_S {
            self.state = State::Complete;
            self.save(io);
            self.apply(io);
            io.report(Report::Complete);
        } else if self.unpersisted_s >= PERSIST_INTERVAL_S {
            self.save(io);
        }
    }

    /// Re-arm for flight (GSE only): silent now, count from the next boot
    pub fn arm<IO: SilenceIo>(&mut self, io: &mut IO) -> Result<(), IoError> {
        self.state = State::Armed;
        self.elapsed_s = 0;
        self.remainder_ms = 0;
        self.apply(io);
        nvm::save(io, self.region, 0, RECORD_MAGIC, &Self::record(State::Armed, 0)).map_err(|_| IoError)
    }

    /// Whether the silence is in force
    pub fn active(&self) -> bool {
        self.state != State::Complete
    }

    /// Current state
    pub const fn state(&self) -> State {
        self.state
    }

    /// Seconds counted toward the silence
    pub const fn elapsed_seconds(&self) -> u32 {
        self.elapsed_s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvm::{FlashError, MemFlash, SLOT_SIZE};

    const MINUTE_MS: u32 = 60_000;

    struct Bench {
        flash: MemFlash<256>,
        stowed: bool,
        inhibit: Option<bool>,
        last: Option<Report>,
    }

    impl Flash for Bench {
        fn read(&mut self, r: u32, o: u32, b: &mut [u8]) -> Result<(), FlashError> {
            self.flash.read(r, o, b)
        }
        fn write(&mut self, r: u32, o: u32, d: &[u8]) -> Result<(), FlashError> {
            self.flash.write(r, o, d)
        }
        fn erase(&mut self, r: u32) -> Result<(), FlashError> {
            self.flash.erase(r)
        }
    }

    impl SilenceIo for Bench {
        fn switch_init(&mut self) {}
        fn stowed(&mut self) -> bool {
            self.stowed
        }
        fn set_inhibit(&mut self, active: bool) {
            self.inhibit = Some(active);
        }
        fn report(&mut self, report: Report) {
            self.last = Some(report);
        }
    }

    fn bench() -> Bench {
        Bench { flash: MemFlash::new(), stowed: false, inhibit: None, last: None }
    }

    fn boot(io: &mut Bench, reset: Reset) -> RfSilence {
        let mut s = RfSilence::new(9);
        s.init(reset, io);
        s
    }

    fn minutes(s: &mut RfSilence, io: &mut Bench, n: u32) {
        for _ in 0..n {
            s.tick(MINUTE_MS, io);
        }
    }

    #[test]
    fn silent_for_exactly_45_minutes() {
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        assert!(s.active());
        assert_eq!(io.inhibit, Some(true));
        minutes(&mut s, &mut io, 44);
        s.tick(MINUTE_MS - 1000, &mut io);
        assert!(s.active());
        s.tick(1000, &mut io);
        assert!(!s.active());
        assert_eq!((io.inhibit, io.last), (Some(false), Some(Report::Complete)));
    }

    #[test]
    fn nothing_before_init() {
        let mut io = bench();
        let mut s = RfSilence::new(9);
        assert!(s.active(), "silent before init");
        minutes(&mut s, &mut io, 60);
        assert_eq!((s.elapsed_seconds(), io.inhibit), (0, None));
    }

    /// Mutation check: the reset-cause rule. Only a reset that kept power
    /// resumes the count; any other restarts it (a switch toggle removes
    /// power). Fails if `PowerKept` is ignored or `PowerLost` resumes.
    #[test]
    fn reset_cause_rule_resume_only_when_power_was_kept() {
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        minutes(&mut s, &mut io, 30);
        let mut s = boot(&mut io, Reset::PowerKept);
        assert_eq!(s.elapsed_seconds(), 30 * 60, "watchdog/software reset resumes");
        assert!(s.active());
        minutes(&mut s, &mut io, 10);
        let s = boot(&mut io, Reset::PowerLost);
        assert_eq!(s.elapsed_seconds(), 0, "power loss restarts");
        assert_eq!(s.state(), State::Counting);
        assert!(s.active());
        // A persisted count at or past the duration is not trusted: restart
        nvm::save(&mut io, 9, 0, RECORD_MAGIC, &RfSilence::record(State::Counting, DURATION_S)).unwrap();
        assert_eq!(boot(&mut io, Reset::PowerKept).elapsed_seconds(), 0);

        // HalResetCause_t: power-on 0, brownout 1, watchdog 2, software 3, pin 4, unknown 5
        let kept = [false, false, true, true, false, false];
        for (cause, kept) in (0u32..).zip(kept) {
            assert_eq!(Reset::from_hal(cause) == Reset::PowerKept, kept, "cause {cause}");
        }
        assert_eq!(Reset::from_hal(99), Reset::PowerLost);
    }

    /// Mutation check: completion persistence. COMPLETE is written when it
    /// is reached and survives every later reset, power-on included. Fails
    /// if completion is not saved or a power-on reset discards it.
    #[test]
    fn completion_persists_across_power_on_resets() {
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        minutes(&mut s, &mut io, 45);
        assert!(!s.active());
        for reset in [Reset::PowerLost, Reset::PowerKept, Reset::PowerLost] {
            let s = boot(&mut io, reset);
            assert_eq!(s.state(), State::Complete);
            assert!(!s.active());
            assert_eq!(io.inhibit, Some(false));
        }
    }

    /// Mutation check: switch reset. A stowed switch seen while powered
    /// restarts the count from 0, even after completion, and that is
    /// persisted. Fails if the switch is ignored in any state.
    #[test]
    fn stowed_switch_restarts_until_completion_then_is_ignored() {
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        minutes(&mut s, &mut io, 40);
        io.stowed = true;
        s.tick(MINUTE_MS, &mut io);
        assert_eq!(s.elapsed_seconds(), 0);
        io.stowed = false;
        minutes(&mut s, &mut io, 44);
        assert!(s.active());
        minutes(&mut s, &mut io, 1);
        assert!(!s.active());

        io.stowed = true;
        s.tick(MINUTE_MS, &mut io);
        assert!(!s.active(), "the served silence is latched (LV-15)");
        assert_eq!(io.inhibit, Some(false));
        assert_eq!(boot(&mut io, Reset::PowerKept).state(), State::Complete);
        // While stowed before completion the count does not advance
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        io.stowed = true;
        minutes(&mut s, &mut io, 50);
        assert_eq!((s.elapsed_seconds(), s.active()), (0, true));
    }

    #[test]
    fn arm_holds_until_the_next_boot() {
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        minutes(&mut s, &mut io, 45);
        assert_eq!(s.arm(&mut io), Ok(()));
        assert_eq!((s.state(), io.inhibit), (State::Armed, Some(true)));
        minutes(&mut s, &mut io, 120);
        assert_eq!(s.state(), State::Armed);
        let s = boot(&mut io, Reset::PowerKept);
        assert_eq!((s.state(), s.elapsed_seconds()), (State::Counting, 0));
        io.flash.writes_left = Some(0);
        let mut s = boot(&mut io, Reset::PowerKept);
        assert_eq!(s.arm(&mut io), Err(IoError));
        assert!(s.active());
    }

    #[test]
    fn corrupt_record_counts_from_zero_and_persist_interval() {
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        minutes(&mut s, &mut io, 45);
        io.flash.bytes[8] ^= 0xFF;
        io.flash.bytes[SLOT_SIZE + 8] ^= 0xFF;
        let mut s = boot(&mut io, Reset::PowerKept);
        assert_eq!(io.last, Some(Report::Unreadable));
        assert_eq!((s.active(), s.elapsed_seconds()), (true, 0));

        // A count under the interval is not persisted: a resume loses it
        s.tick(59_999, &mut io);
        assert_eq!(boot(&mut io, Reset::PowerKept).elapsed_seconds(), 0);
        let mut s = boot(&mut io, Reset::PowerKept);
        s.tick(60_000, &mut io);
        assert_eq!(boot(&mut io, Reset::PowerKept).elapsed_seconds(), 60);
    }

    /// Mutation check: a stowed switch also restarts an ARMED silence (the
    /// count then runs while powered)
    #[test]
    fn stowed_switch_restarts_armed() {
        let mut io = bench();
        let mut s = boot(&mut io, Reset::PowerLost);
        assert_eq!(s.arm(&mut io), Ok(()));
        io.stowed = true;
        s.tick(MINUTE_MS, &mut io);
        assert_eq!((s.state(), s.elapsed_seconds()), (State::Counting, 0));
        assert_eq!(boot(&mut io, Reset::PowerKept).state(), State::Counting, "persisted");
        io.stowed = false;
        minutes(&mut s, &mut io, 45);
        assert_eq!(s.state(), State::Complete);
    }
}
