//! End-of-life passivation by telecommand (`passivation.h`; BASELINE §5,
//! 47 CFR 97.207(g)(1)(iii); RIDs REG-01, EPS-06, FSW-07, SMA-13).
//!
//! Two authenticated telecommands from two key custodians:
//!
//! 1. `PASSIVATE_ARM` (operational key, argument the ASCII code `EOL!`)
//!    arms the sequence for [`ARM_WINDOW_MS`]. Nothing is driven. The arm
//!    is kept in RAM only: a reset, or the window running out, disarms.
//! 2. `PASSIVATE_EXECUTE` (the offline recovery key) inside that window
//!    runs, in this order:
//!    - TX-OFF, latched (`rf_control_tx_off()`): the transmitter enable
//!      line drops first;
//!    - the passivated record, persisted (the commit point), and the
//!      passivation inhibit on every transmitter (`RF_INHIBIT_PASSIVATED`),
//!      which no telecommand clears;
//!    - the EPS one-shot charge disconnect (`docs/hardware/EPS_DESIGN.md`
//!      §7; `hardware/electrical/eps/EPS_BOARD_DESIGN.md`), in sunlight
//!      only (the array then carries the loads while the heater draws from
//!      the pack): `EOL_ARM` rises (its rising edge sets the EPS's arm
//!      latch), [`ARM_SETTLE_MS`] later `EOL_FIRE` goes high (a level, in
//!      series with the latch through the disconnect's heater) and is held
//!      at least [`FIRE_MIN_MS`]; it drops once the EPS reports the charge
//!      path open (`EOL_STATE`) after that, or at [`FIRE_MAX_MS`], then
//!      both lines are low. An eclipse before [`FIRE_MIN_MS`] drops both and
//!      the fire starts again in the next sunlight.
//!
//! After that the transmitters stay off for good: at every boot the record
//! (or the EPS reporting the charge path open, should the record be lost)
//! sets the inhibit again before anything can transmit. The receiver stays
//! powered until the pack is drained. A fire interrupted by a reset is run
//! again at the next boot, at most [`MAX_FIRE_ATTEMPTS`] times.
//!
//! @requirement SRS-F110 Two-step authenticated end-of-life passivation
//! @requirement SRS-F111 Passivation order: TX-OFF, record, charge disconnect through two series lines
//! @requirement SRS-F112 Transmitters off for good after passivation

use crate::nvm::{self, Flash, RecordStatus};

/// How long PASSIVATE_ARM holds, ms
pub const ARM_WINDOW_MS: u64 = 600_000;
/// PASSIVATE_ARM's argument
pub const ARM_CODE: [u8; 4] = *b"EOL!";
/// `EOL_ARM` is high this long before `EOL_FIRE`, ms (the arm latch sets
/// before the series fire switch closes)
pub const ARM_SETTLE_MS: u64 = 200;
/// `EOL_FIRE` is held at least this long, ms (the EPS board's requirement:
/// the thermal cut-off's heater needs it to open the disconnect)
pub const FIRE_MIN_MS: u64 = 60_000;
/// and at most this long, ms, when the EPS does not confirm
pub const FIRE_MAX_MS: u64 = 120_000;
/// Fire sequences at most, across resets
pub const MAX_FIRE_ATTEMPTS: u8 = 3;
/// Record ("EOLP"): flags (1) | version (1) | fire attempts (1) | spare (1)
pub const RECORD_MAGIC: u32 = 0x454F_4C50;
/// Record layout version
pub const VERSION: u8 = 1;
const FLAG_PASSIVATED: u8 = 0x01;
const FLAG_FIRE_DONE: u8 = 0x02;
const FLAG_CONFIRMED: u8 = 0x04;

/// The two series lines of the charge disconnect
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    /// `GPIO_PIN_EOL_ARM`: the arm latch
    Arm,
    /// `GPIO_PIN_EOL_FIRE`: the fire switch
    Fire,
}

/// Events for the fault log
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// Armed: PASSIVATE_EXECUTE accepted for [`ARM_WINDOW_MS`]
    Armed,
    /// The arm lapsed unused
    Lapsed,
    /// Executed: TX-OFF latched, the inhibit set, the fire started
    Executed,
    /// The passivated record could not be written (the act goes on)
    NotPersisted,
    /// A line could not be driven (no pin on this build)
    LineUnavailable,
    /// The EPS reports the charge path open
    ChargePathOpen,
    /// The fire ran its full time without the EPS confirming
    NotConfirmed,
    /// Boot: the record is unreadable (not passivated unless the EPS says so)
    Unreadable,
    /// Boot: passivated (the record or the EPS); transmitters inhibited
    Passivated,
}

/// What the passivation drives and reads
pub trait PassivationIo: Flash {
    /// Latch TX-OFF (the enable line first, then its record)
    fn tx_off(&mut self);
    /// Set the passivation inhibit on every transmitter
    fn inhibit(&mut self);
    /// Drive a line; false if it cannot be driven
    fn drive(&mut self, line: Line, on: bool) -> bool;
    /// Whether the EPS reports the charge path open (`None`: no answer)
    fn charge_path_open(&mut self) -> Option<bool>;
    /// Log an event
    fn report(&mut self, report: Report);
}

/// Why a command was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The arming code is wrong
    BadCode,
    /// PASSIVATE_EXECUTE without a live arm
    NotArmed,
    /// Already passivated
    Passivated,
}

/// Where the sequence is (`PassivationPhase_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Nothing armed
    Idle = 0,
    /// Armed until the window ends
    Armed = 1,
    /// Passivated, the charge disconnect being fired
    Firing = 2,
    /// Passivated, the fire sequence done
    Passivated = 3,
}

/// A step of the fire sequence
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    /// Waiting for sunlight; both lines low
    WaitSun,
    /// `EOL_ARM` high since this time, ms
    Settle(u64),
    /// `EOL_FIRE` high since this time, ms
    Fire(u64),
}

/// Module state
#[derive(Clone, Copy, Debug)]
pub struct Passivation {
    region: u32,
    passivated: bool,
    fire_done: bool,
    confirmed: bool,
    attempts: u8,
    /// Armed until this time, ms
    armed_until: Option<u64>,
    /// Where the fire sequence is (`None`: not firing)
    fire: Option<Step>,
    initialized: bool,
}

impl Passivation {
    /// Before init: not passivated, not armed
    pub const fn new(region: u32) -> Passivation {
        Passivation {
            region,
            passivated: false,
            fire_done: false,
            confirmed: false,
            attempts: 0,
            armed_until: None,
            fire: None,
            initialized: false,
        }
    }

    fn record(&self) -> [u8; 4] {
        let mut flags = 0;
        if self.passivated {
            flags |= FLAG_PASSIVATED;
        }
        if self.fire_done {
            flags |= FLAG_FIRE_DONE;
        }
        if self.confirmed {
            flags |= FLAG_CONFIRMED;
        }
        [flags, VERSION, self.attempts, 0]
    }

    fn save<IO: PassivationIo>(&self, io: &mut IO) {
        if nvm::save(io, self.region, 0, RECORD_MAGIC, &self.record()).is_err() {
            io.report(Report::NotPersisted);
        }
    }

    /// Load the record at boot, before anything can transmit: passivated
    /// (by the record or by the EPS) sets the inhibit at once, and an
    /// unfinished fire starts again
    pub fn init<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) {
        // A reset forgets everything but the record
        *self = Passivation::new(self.region);
        let mut rec = [0u8; 4];
        match nvm::load(io, self.region, 0, RECORD_MAGIC, &mut rec) {
            Ok(RecordStatus::Valid) => {
                let [flags, _, attempts, _] = rec;
                self.passivated = flags & FLAG_PASSIVATED != 0;
                self.fire_done = flags & FLAG_FIRE_DONE != 0;
                self.confirmed = flags & FLAG_CONFIRMED != 0;
                self.attempts = attempts;
            }
            Ok(RecordStatus::Blank) => {}
            Ok(RecordStatus::Corrupt) | Err(_) => io.report(Report::Unreadable),
        }
        self.armed_until = None;
        self.fire = None;
        self.initialized = true;
        // The charge path open is passivation whatever the record says
        if !self.passivated && io.charge_path_open() == Some(true) {
            self.passivated = true;
            self.fire_done = true;
            self.confirmed = true;
            self.save(io);
        }
        if self.passivated {
            io.inhibit();
            io.report(Report::Passivated);
            if !self.fire_done {
                self.start_fire(now_ms, io);
            }
        }
    }

    /// PASSIVATE_ARM with its argument
    pub fn arm<IO: PassivationIo>(&mut self, code: &[u8], now_ms: u64, io: &mut IO) -> Result<(), Refusal> {
        if self.passivated {
            return Err(Refusal::Passivated);
        }
        if code != ARM_CODE {
            return Err(Refusal::BadCode);
        }
        self.armed_until = Some(now_ms.saturating_add(ARM_WINDOW_MS));
        io.report(Report::Armed);
        Ok(())
    }

    /// PASSIVATE_EXECUTE: TX-OFF, the record, the inhibit, then the fire
    pub fn execute<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) -> Result<(), Refusal> {
        if self.passivated {
            return Err(Refusal::Passivated);
        }
        if !self.armed_until.is_some_and(|until| now_ms < until) {
            return Err(Refusal::NotArmed);
        }
        self.armed_until = None;
        io.tx_off();
        self.passivated = true;
        self.save(io);
        io.inhibit();
        io.report(Report::Executed);
        self.start_fire(now_ms, io);
        Ok(())
    }

    fn start_fire<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) {
        if self.attempts >= MAX_FIRE_ATTEMPTS {
            self.finish(false, io);
            return;
        }
        self.attempts = self.attempts.saturating_add(1);
        self.save(io);
        let _ = now_ms;
        self.fire = Some(Step::WaitSun);
    }

    /// Both lines low, the sequence done
    fn finish<IO: PassivationIo>(&mut self, confirmed: bool, io: &mut IO) {
        let _ = io.drive(Line::Fire, false);
        let _ = io.drive(Line::Arm, false);
        self.fire = None;
        self.fire_done = true;
        self.confirmed = confirmed;
        self.save(io);
        io.report(if confirmed { Report::ChargePathOpen } else { Report::NotConfirmed });
    }

    /// Drive a line of the sequence; a line that cannot be driven ends it
    fn drive_or_finish<IO: PassivationIo>(&mut self, line: Line, io: &mut IO) -> bool {
        if io.drive(line, true) {
            return true;
        }
        io.report(Report::LineUnavailable);
        self.finish(false, io);
        false
    }

    /// Every pass: the arm's window, and the fire sequence (`sunlit`: the
    /// array is lit)
    pub fn tick<IO: PassivationIo>(&mut self, now_ms: u64, sunlit: bool, io: &mut IO) {
        if self.armed_until.is_some_and(|until| now_ms >= until) {
            self.armed_until = None;
            io.report(Report::Lapsed);
        }
        match self.fire {
            None => {}
            Some(Step::WaitSun) => {
                if sunlit && self.drive_or_finish(Line::Arm, io) {
                    self.fire = Some(Step::Settle(now_ms));
                }
            }
            Some(Step::Settle(_) | Step::Fire(_)) if !sunlit && !self.fire_min_served(now_ms) => {
                // Eclipse before the minimum: both low, again in sunlight
                let _ = io.drive(Line::Fire, false);
                let _ = io.drive(Line::Arm, false);
                self.fire = Some(Step::WaitSun);
            }
            Some(Step::Settle(since)) => {
                if now_ms.saturating_sub(since) >= ARM_SETTLE_MS && self.drive_or_finish(Line::Fire, io) {
                    self.fire = Some(Step::Fire(now_ms));
                }
            }
            Some(Step::Fire(since)) => {
                let held = now_ms.saturating_sub(since);
                if held >= FIRE_MIN_MS && io.charge_path_open() == Some(true) {
                    self.finish(true, io);
                } else if held >= FIRE_MAX_MS || (held >= FIRE_MIN_MS && !sunlit) {
                    self.finish(false, io);
                }
            }
        }
    }

    fn fire_min_served(&self, now_ms: u64) -> bool {
        matches!(self.fire, Some(Step::Fire(since)) if now_ms.saturating_sub(since) >= FIRE_MIN_MS)
    }

    /// Check the RAM copy against the record (AV-07): passivated in either
    /// is passivated in both, and the inhibit is applied again
    pub fn verify<IO: PassivationIo>(&mut self, io: &mut IO) {
        if !self.initialized {
            return;
        }
        let mut rec = [0u8; 4];
        let stored = nvm::load(io, self.region, 0, RECORD_MAGIC, &mut rec) == Ok(RecordStatus::Valid)
            && rec[0] & FLAG_PASSIVATED != 0;
        if stored && !self.passivated {
            self.passivated = true;
            self.fire_done = rec[0] & FLAG_FIRE_DONE != 0;
            io.report(Report::Passivated);
        } else if self.passivated && !stored {
            self.save(io);
        }
        if self.passivated {
            io.inhibit();
        }
    }

    /// Where the sequence is
    pub fn phase(&self, now_ms: u64) -> Phase {
        match (self.passivated, self.fire_done) {
            (true, true) => Phase::Passivated,
            (true, false) => Phase::Firing,
            (false, _) if self.armed_until.is_some_and(|until| now_ms < until) => Phase::Armed,
            (false, _) => Phase::Idle,
        }
    }

    /// Whether the spacecraft is passivated (transmitters off for good)
    pub const fn passivated(&self) -> bool {
        self.passivated
    }

    /// Whether the EPS confirmed the charge path open
    pub const fn confirmed(&self) -> bool {
        self.confirmed
    }

    /// Fire sequences run
    pub const fn attempts(&self) -> u8 {
        self.attempts
    }

    /// Whether the record has been read
    pub const fn initialized(&self) -> bool {
        self.initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvm::{FlashError, MemFlash, SLOT_SIZE};
    extern crate std;
    use std::vec::Vec;

    const REGION: u32 = 0;
    const SUN: bool = true;

    #[derive(Default)]
    struct Bench {
        flash: MemFlash<256>,
        fail_writes: bool,
        tx_off: u32,
        inhibited: bool,
        lines: [bool; 2],
        no_pins: bool,
        no_fire_pin: bool,
        open: Option<bool>,
        log: Vec<Report>,
        /// (line, on) in the order driven
        drives: Vec<(Line, bool)>,
    }

    impl Flash for Bench {
        fn read(&mut self, r: u32, o: u32, b: &mut [u8]) -> Result<(), FlashError> {
            self.flash.read(r, o, b)
        }
        fn write(&mut self, r: u32, o: u32, d: &[u8]) -> Result<(), FlashError> {
            if self.fail_writes {
                return Err(FlashError(-7));
            }
            self.flash.write(r, o, d)
        }
        fn erase(&mut self, r: u32) -> Result<(), FlashError> {
            self.flash.erase(r)
        }
    }

    impl PassivationIo for Bench {
        fn tx_off(&mut self) {
            self.tx_off += 1;
        }
        fn inhibit(&mut self) {
            self.inhibited = true;
        }
        fn drive(&mut self, line: Line, on: bool) -> bool {
            if on && (self.no_pins || (self.no_fire_pin && line == Line::Fire)) {
                return false;
            }
            self.lines[line as usize] = on;
            self.drives.push((line, on));
            true
        }
        fn charge_path_open(&mut self) -> Option<bool> {
            self.open
        }
        fn report(&mut self, report: Report) {
            self.log.push(report);
        }
    }

    fn fresh(io: &mut Bench) -> Passivation {
        let mut p = Passivation::new(REGION);
        assert!(!p.initialized());
        p.init(0, io);
        assert!(p.initialized());
        p
    }

    /// Arm, execute, and the fire up to its start (EOL_FIRE high at `t0`)
    fn executed(io: &mut Bench, t0: u64) -> Passivation {
        let mut p = fresh(io);
        assert_eq!(p.arm(&ARM_CODE, 0, io), Ok(()));
        assert_eq!(p.execute(0, io), Ok(()));
        p.tick(t0 - ARM_SETTLE_MS, SUN, io);
        p.tick(t0, SUN, io);
        p
    }

    #[test]
    fn arm_then_execute_in_order() {
        let mut io = Bench::default();
        let mut p = fresh(&mut io);
        assert_eq!(p.phase(0), Phase::Idle);
        assert_eq!(p.execute(1000, &mut io), Err(Refusal::NotArmed), "execute alone does nothing");
        assert_eq!(p.arm(b"EOL?", 1000, &mut io), Err(Refusal::BadCode));
        assert_eq!(p.arm(&ARM_CODE, 1000, &mut io), Ok(()));
        assert_eq!(p.phase(1000), Phase::Armed);
        assert!(io.drives.is_empty() && io.tx_off == 0, "arming drives nothing");
        assert_eq!(p.execute(2000, &mut io), Ok(()));
        assert_eq!(io.tx_off, 1);
        assert!(io.inhibited);
        assert!(p.passivated());
        assert_eq!(p.phase(2000), Phase::Firing);
        // In eclipse nothing is driven
        p.tick(3000, !SUN, &mut io);
        assert!(io.drives.is_empty());
        p.tick(4000, SUN, &mut io);
        assert_eq!(io.drives, [(Line::Arm, true)], "the arm latch first");
        p.tick(4000 + ARM_SETTLE_MS - 1, SUN, &mut io);
        assert_eq!(io.drives.len(), 1);
        let t0 = 4000 + ARM_SETTLE_MS;
        p.tick(t0, SUN, &mut io);
        assert_eq!(io.lines, [true, true], "then the fire switch");
        // Confirmed early: still held for the minimum
        io.open = Some(true);
        p.tick(t0 + FIRE_MIN_MS - 1, SUN, &mut io);
        assert_eq!(io.lines, [true, true]);
        p.tick(t0 + FIRE_MIN_MS, SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        assert_eq!(p.phase(t0 + FIRE_MIN_MS), Phase::Passivated);
        assert!(p.confirmed());
        assert_eq!(p.attempts(), 1);
        assert_eq!(io.log, [Report::Armed, Report::Executed, Report::ChargePathOpen]);
        assert_eq!(p.arm(&ARM_CODE, 7000, &mut io), Err(Refusal::Passivated));
        assert_eq!(p.execute(7000, &mut io), Err(Refusal::Passivated));
    }

    #[test]
    fn the_arm_lapses() {
        let mut io = Bench::default();
        let mut p = fresh(&mut io);
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Ok(()));
        p.tick(ARM_WINDOW_MS - 1, SUN, &mut io);
        assert_eq!(p.phase(ARM_WINDOW_MS - 1), Phase::Armed);
        assert_eq!(p.phase(ARM_WINDOW_MS), Phase::Idle);
        assert_eq!(p.execute(ARM_WINDOW_MS, &mut io), Err(Refusal::NotArmed), "the window is over");
        p.tick(ARM_WINDOW_MS, SUN, &mut io);
        assert_eq!(io.log.last(), Some(&Report::Lapsed));
        assert!(!p.passivated() && io.drives.is_empty());
        // A reset disarms
        assert_eq!(p.arm(&ARM_CODE, 10, &mut io), Ok(()));
        let mut after = fresh(&mut io);
        assert_eq!(after.execute(20, &mut io), Err(Refusal::NotArmed));
    }

    #[test]
    fn an_eclipse_before_the_minimum_restarts_the_fire() {
        let mut io = Bench::default();
        let t0 = 10_000;
        let mut p = executed(&mut io, t0);
        assert_eq!(io.lines, [true, true]);
        p.tick(t0 + FIRE_MIN_MS - 1, !SUN, &mut io);
        assert_eq!(io.lines, [false, false], "eclipse: both low");
        p.tick(t0 + FIRE_MIN_MS + 100, !SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        // An eclipse during the arm's settling also drops it
        p.tick(200_000, SUN, &mut io);
        assert_eq!(io.lines, [true, false]);
        p.tick(200_050, !SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        // Sunlight again: the whole minimum again
        p.tick(300_000, SUN, &mut io);
        p.tick(300_000 + ARM_SETTLE_MS, SUN, &mut io);
        assert_eq!(io.lines, [true, true]);
        assert_eq!(p.attempts(), 1, "the same attempt");
        // Past the minimum an eclipse ends it, unconfirmed
        p.tick(300_000 + ARM_SETTLE_MS + FIRE_MIN_MS, !SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        assert_eq!(io.log.last(), Some(&Report::NotConfirmed));
        assert_eq!(p.phase(0), Phase::Passivated);
    }

    #[test]
    fn unconfirmed_fire_times_out_and_passivation_survives_resets() {
        let mut io = Bench::default();
        let t0 = 1000;
        let mut p = executed(&mut io, t0);
        p.tick(t0 + FIRE_MAX_MS - 1, SUN, &mut io);
        assert_eq!(io.lines, [true, true]);
        p.tick(t0 + FIRE_MAX_MS, SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        assert_eq!(io.log.last(), Some(&Report::NotConfirmed));
        assert_eq!(p.phase(u64::MAX), Phase::Passivated);
        // Every boot after: inhibited at once, nothing fired again
        let mut io2 = Bench { flash: io.flash, ..Bench::default() };
        let b = fresh(&mut io2);
        assert!(b.passivated() && io2.inhibited);
        assert!(io2.drives.is_empty());
        assert_eq!(io2.log, [Report::Passivated]);
        assert!(!b.confirmed());
    }

    #[test]
    fn a_fire_interrupted_by_a_reset_runs_again_a_bounded_number_of_times() {
        let mut io = Bench::default();
        let _ = executed(&mut io, 1000);
        for attempt in 2..=MAX_FIRE_ATTEMPTS {
            let mut io2 = Bench { flash: io.flash, ..Bench::default() };
            let mut b = fresh(&mut io2);
            assert_eq!(b.attempts(), attempt);
            assert_eq!(b.phase(0), Phase::Firing);
            b.tick(5, SUN, &mut io2);
            assert_eq!(io2.drives, [(Line::Arm, true)], "fire restarted");
            io.flash = io2.flash;
        }
        let mut io3 = Bench { flash: io.flash, ..Bench::default() };
        let b = fresh(&mut io3);
        assert_eq!(b.phase(0), Phase::Passivated, "attempts spent: done");
        assert_eq!(io3.drives, [(Line::Fire, false), (Line::Arm, false)]);
        assert!(io3.inhibited);
    }

    #[test]
    fn missing_lines_and_unwritable_records() {
        // No pin for the arm latch: TX off for good anyway
        let mut io = Bench { no_pins: true, ..Bench::default() };
        let mut p = fresh(&mut io);
        let _ = p.arm(&ARM_CODE, 0, &mut io);
        assert_eq!(p.execute(0, &mut io), Ok(()));
        p.tick(1, SUN, &mut io);
        assert!(io.inhibited && io.tx_off == 1);
        assert!(io.log.contains(&Report::LineUnavailable));
        assert_eq!(p.phase(0), Phase::Passivated);
        // No pin for the fire switch
        let mut io = Bench { no_fire_pin: true, ..Bench::default() };
        let mut p = fresh(&mut io);
        let _ = p.arm(&ARM_CODE, 0, &mut io);
        let _ = p.execute(0, &mut io);
        p.tick(1, SUN, &mut io);
        p.tick(1 + ARM_SETTLE_MS, SUN, &mut io);
        assert!(io.log.contains(&Report::LineUnavailable));
        assert_eq!(io.lines, [false, false]);
        // The record cannot be written: the act goes on
        let mut io = Bench { fail_writes: true, ..Bench::default() };
        let mut p = fresh(&mut io);
        let _ = p.arm(&ARM_CODE, 0, &mut io);
        assert_eq!(p.execute(0, &mut io), Ok(()));
        assert!(io.log.contains(&Report::NotPersisted));
        assert!(p.passivated() && io.inhibited);
    }

    #[test]
    fn verify_restores_passivation_from_either_copy() {
        let mut io = Bench::default();
        let mut p = Passivation::new(REGION);
        p.verify(&mut io);
        assert!(!io.inhibited, "before init: nothing");
        p.init(0, &mut io);
        p.verify(&mut io);
        assert!(!io.inhibited && io.log.is_empty());
        let mut q = executed(&mut io, 1000);
        q.passivated = false; // a RAM upset
        io.inhibited = false;
        q.verify(&mut io);
        assert!(q.passivated() && io.inhibited);
        assert_eq!(io.log.last(), Some(&Report::Passivated));
        // The record lost, RAM passivated: written again
        let mut io2 = Bench::default();
        let mut r = fresh(&mut io2);
        r.passivated = true;
        r.verify(&mut io2);
        let mut after = fresh(&mut io2);
        assert!(after.passivated());
        after.verify(&mut io2);
    }

    #[test]
    fn the_eps_saying_open_is_passivation_even_without_a_record() {
        let mut io = Bench { open: Some(true), ..Bench::default() };
        let p = fresh(&mut io);
        assert!(p.passivated() && p.confirmed() && io.inhibited);
        assert!(io.drives.is_empty());
        // Corrupt record, EPS silent: not passivated, reported
        let mut io = Bench::default();
        let _ = executed(&mut io, 1000);
        let slot = u32::try_from(SLOT_SIZE).unwrap();
        for s in 0..2u32 {
            let mut b = [0u8; 1];
            io.flash.read(REGION, s * slot + 8, &mut b).unwrap();
            io.flash.write(REGION, s * slot + 8, &[b[0] ^ 0xFF]).unwrap();
        }
        let mut io2 = Bench { flash: io.flash, open: None, ..Bench::default() };
        let b = fresh(&mut io2);
        assert!(!b.passivated());
        assert_eq!(io2.log, [Report::Unreadable]);
    }
}
