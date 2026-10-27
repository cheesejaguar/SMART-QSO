//! End-of-life passivation by telecommand (`passivation.h`; BASELINE §5,
//! 47 CFR 97.207(g)(1)(iii); RIDs REG-01, REG-12, EPS-06, FSW-07, SMA-13).
//!
//! Two authenticated telecommands from two key custodians:
//!
//! 1. `PASSIVATE_ARM` (operational key, argument the ASCII code `EOL!`)
//!    arms the sequence for [`ARM_WINDOW_MS`]. Nothing is driven. The arm
//!    is kept in RAM only: a reset, or the window running out, disarms.
//! 2. `PASSIVATE_EXECUTE` (the offline recovery key) inside that window
//!    commits the passivation (the record, persisted: from here on every
//!    boot carries it through) and runs, in this order:
//!    - the EPS one-shot charge disconnect (`docs/hardware/EPS_DESIGN.md`
//!      §7; `hardware/electrical/eps/EPS_BOARD_DESIGN.md`), in sunlight
//!      only (the array then carries the loads while the heater draws from
//!      the pack), **while the spacecraft still transmits** so the ground can
//!      see the result: `EOL_ARM` rises (its rising edge sets the EPS's arm
//!      latch), [`ARM_SETTLE_MS`] later `EOL_FIRE` goes high (a level, in
//!      series with the latch through the disconnect's heater) and is held
//!      at least [`FIRE_MIN_MS`]; it drops once the EPS reports the charge
//!      path open (`EOL_STATE`) after that, or at [`FIRE_MAX_MS`], then
//!      both lines are low. An eclipse before [`FIRE_MIN_MS`] drops both and
//!      the fire starts again in the next sunlight; a fire the EPS does not
//!      confirm is tried again, at most [`MAX_FIRE_ATTEMPTS`] times in all
//!      (across resets too);
//!    - the final report (RID REG-12, finding EOL-1 of the end-of-life
//!      plan): [`FINAL_REPORTS`] housekeeping frames, [`REPORT_INTERVAL_MS`]
//!      apart, carrying `EOL_STATE`, the fuse state and the attempts;
//!    - then, [`REPORT_WINDOW_MS`] after the first report, TX-OFF latched
//!      (`rf_control_tx_off()`: the transmitter enable line drops first) and
//!      the passivation inhibit on every transmitter
//!      (`RF_INHIBIT_PASSIVATED`), which no telecommand clears. A failed fire
//!      is silenced the same way after its report: the emissions cease.
//!
//! After that the transmitters stay off for good: at every boot the record
//! (or the EPS reporting the charge path open, should the record be lost)
//! sets the inhibit again before anything can transmit. The receiver stays
//! powered until the pack is drained. A reset during the fire runs it again
//! at the next boot (within the attempts); a reset during the report sends
//! the report again, then silences.
//!
//! @requirement SRS-F110 Two-step authenticated end-of-life passivation
//! @requirement SRS-F111 Passivation order: record, charge disconnect through two series lines while transmitting, final report, then TX-OFF
//! @requirement SRS-F112 Transmitters off for good after passivation
//! @requirement SRS-F155 The final passivation report: three frames over three minutes carrying EOL_STATE, the fuse state and the attempts, before TX-OFF

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
/// Final report frames
pub const FINAL_REPORTS: u8 = 3;
/// Between report frames, ms
pub const REPORT_INTERVAL_MS: u64 = 60_000;
/// From the first report to TX-OFF, ms
pub const REPORT_WINDOW_MS: u64 = 180_000;
const _: () = assert!(REPORT_INTERVAL_MS * (FINAL_REPORTS as u64 - 1) < REPORT_WINDOW_MS);
/// Record ("EOLP"): flags (1) | version (1) | fire attempts (1) | spare (1)
pub const RECORD_MAGIC: u32 = 0x454F_4C50;
/// Record layout version
pub const VERSION: u8 = 2;
const FLAG_COMMITTED: u8 = 0x01;
const FLAG_FIRE_DONE: u8 = 0x02;
const FLAG_CONFIRMED: u8 = 0x04;
const FLAG_SILENCED: u8 = 0x08;

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
    /// Executed: the record committed, the fire started (still transmitting)
    Executed,
    /// The passivated record could not be written (the act goes on)
    NotPersisted,
    /// A line could not be driven (no pin on this build)
    LineUnavailable,
    /// The EPS reports the charge path open
    ChargePathOpen,
    /// A fire ran its full time without the EPS confirming
    NotConfirmed,
    /// Boot: the record is unreadable (not passivated unless the EPS says so)
    Unreadable,
    /// Boot: passivated (the record or the EPS); transmitters inhibited
    Passivated,
    /// The final report is done: TX-OFF latched, transmitters inhibited
    Silenced,
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
    /// Already passivated (committed)
    Passivated,
}

/// Where the sequence is (`PassivationPhase_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Nothing armed
    Idle = 0,
    /// Armed until the window ends
    Armed = 1,
    /// Committed, the charge disconnect being fired (still transmitting)
    Firing = 2,
    /// Silenced: TX-OFF latched, transmitters off for good
    Passivated = 3,
    /// The fire done, the final report going out (TX-OFF follows)
    Reporting = 4,
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
    committed: bool,
    fire_done: bool,
    confirmed: bool,
    silenced: bool,
    attempts: u8,
    /// Armed until this time, ms
    armed_until: Option<u64>,
    /// Where the fire sequence is (`None`: not firing)
    fire: Option<Step>,
    /// The final report: when it started, ms, and the frames handed out
    report: Option<(u64, u8)>,
    initialized: bool,
}

impl Passivation {
    /// Before init: not passivated, not armed
    pub const fn new(region: u32) -> Passivation {
        Passivation {
            region,
            committed: false,
            fire_done: false,
            confirmed: false,
            silenced: false,
            attempts: 0,
            armed_until: None,
            fire: None,
            report: None,
            initialized: false,
        }
    }

    fn record(&self) -> [u8; 4] {
        let mut flags = 0;
        for (on, bit) in [
            (self.committed, FLAG_COMMITTED),
            (self.fire_done, FLAG_FIRE_DONE),
            (self.confirmed, FLAG_CONFIRMED),
            (self.silenced, FLAG_SILENCED),
        ] {
            if on {
                flags |= bit;
            }
        }
        [flags, VERSION, self.attempts, 0]
    }

    fn set_flags(&mut self, flags: u8) {
        self.committed = flags & FLAG_COMMITTED != 0;
        self.fire_done = flags & FLAG_FIRE_DONE != 0;
        self.confirmed = flags & FLAG_CONFIRMED != 0;
        // A version-1 record (TX-OFF first) was silenced when committed
        self.silenced = flags & FLAG_SILENCED != 0;
    }

    fn save<IO: PassivationIo>(&self, io: &mut IO) {
        if nvm::save(io, self.region, 0, RECORD_MAGIC, &self.record()).is_err() {
            io.report(Report::NotPersisted);
        }
    }

    /// Load the record at boot, before anything can transmit: silenced (by
    /// the record or by the EPS) sets the inhibit at once; an unfinished
    /// fire starts again, an unfinished report is sent again
    pub fn init<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) {
        // A reset forgets everything but the record
        *self = Passivation::new(self.region);
        let mut rec = [0u8; 4];
        match nvm::load(io, self.region, 0, RECORD_MAGIC, &mut rec) {
            Ok(RecordStatus::Valid) => {
                let [flags, version, attempts, _] = rec;
                self.set_flags(if version < VERSION && flags & FLAG_COMMITTED != 0 {
                    flags | FLAG_SILENCED
                } else {
                    flags
                });
                self.attempts = attempts;
            }
            Ok(RecordStatus::Blank) => {}
            Ok(RecordStatus::Corrupt) | Err(_) => io.report(Report::Unreadable),
        }
        self.initialized = true;
        // The charge path open with no record is passivation already done
        if !self.committed && io.charge_path_open() == Some(true) {
            self.set_flags(FLAG_COMMITTED | FLAG_FIRE_DONE | FLAG_CONFIRMED | FLAG_SILENCED);
            self.save(io);
        }
        if self.silenced {
            io.inhibit();
            io.report(Report::Passivated);
        } else if self.fire_done {
            self.report = Some((now_ms, 0));
        } else if self.committed {
            self.start_fire(now_ms, io);
        }
    }

    /// PASSIVATE_ARM with its argument
    pub fn arm<IO: PassivationIo>(&mut self, code: &[u8], now_ms: u64, io: &mut IO) -> Result<(), Refusal> {
        if self.committed {
            return Err(Refusal::Passivated);
        }
        if code != ARM_CODE {
            return Err(Refusal::BadCode);
        }
        self.armed_until = Some(now_ms.saturating_add(ARM_WINDOW_MS));
        io.report(Report::Armed);
        Ok(())
    }

    /// PASSIVATE_EXECUTE: the record committed, then the fire (the
    /// transmitters stay on until the final report)
    pub fn execute<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) -> Result<(), Refusal> {
        if self.committed {
            return Err(Refusal::Passivated);
        }
        if !self.armed_until.is_some_and(|until| now_ms < until) {
            return Err(Refusal::NotArmed);
        }
        self.armed_until = None;
        self.committed = true;
        self.save(io);
        io.report(Report::Executed);
        self.start_fire(now_ms, io);
        Ok(())
    }

    fn start_fire<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) {
        if self.attempts >= MAX_FIRE_ATTEMPTS {
            self.finish(now_ms, false, io);
            return;
        }
        self.attempts = self.attempts.saturating_add(1);
        self.save(io);
        self.fire = Some(Step::WaitSun);
    }

    /// Both lines low; a confirmed fire (or the attempts spent) goes to the
    /// final report, an unconfirmed one is tried again
    fn end_attempt<IO: PassivationIo>(&mut self, now_ms: u64, confirmed: bool, io: &mut IO) {
        let _ = io.drive(Line::Fire, false);
        let _ = io.drive(Line::Arm, false);
        self.fire = None;
        io.report(if confirmed { Report::ChargePathOpen } else { Report::NotConfirmed });
        if confirmed {
            self.finish(now_ms, true, io);
        } else {
            self.start_fire(now_ms, io);
        }
    }

    /// The fire sequence done: the final report starts
    fn finish<IO: PassivationIo>(&mut self, now_ms: u64, confirmed: bool, io: &mut IO) {
        let _ = io.drive(Line::Fire, false);
        let _ = io.drive(Line::Arm, false);
        self.fire = None;
        self.fire_done = true;
        self.confirmed = confirmed;
        self.save(io);
        self.report = Some((now_ms, 0));
    }

    /// Drive a line of the sequence; a line that cannot be driven ends the
    /// fire (straight to the report: no retry can drive it either)
    fn drive_or_finish<IO: PassivationIo>(&mut self, now_ms: u64, line: Line, io: &mut IO) -> bool {
        if io.drive(line, true) {
            return true;
        }
        io.report(Report::LineUnavailable);
        self.finish(now_ms, false, io);
        false
    }

    /// Every pass: the arm's window, the fire sequence (`sunlit`: the array
    /// is lit) and the end of the final report
    pub fn tick<IO: PassivationIo>(&mut self, now_ms: u64, sunlit: bool, io: &mut IO) {
        if self.armed_until.is_some_and(|until| now_ms >= until) {
            self.armed_until = None;
            io.report(Report::Lapsed);
        }
        if let Some((start, _)) = self.report
            && now_ms.saturating_sub(start) >= REPORT_WINDOW_MS
        {
            self.report = None;
            self.silence(io);
        }
        match self.fire {
            None => {}
            Some(Step::WaitSun) => {
                if sunlit && self.drive_or_finish(now_ms, Line::Arm, io) {
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
                if now_ms.saturating_sub(since) >= ARM_SETTLE_MS && self.drive_or_finish(now_ms, Line::Fire, io) {
                    self.fire = Some(Step::Fire(now_ms));
                }
            }
            Some(Step::Fire(since)) => {
                let held = now_ms.saturating_sub(since);
                if held >= FIRE_MIN_MS && io.charge_path_open() == Some(true) {
                    self.end_attempt(now_ms, true, io);
                } else if held >= FIRE_MAX_MS || (held >= FIRE_MIN_MS && !sunlit) {
                    self.end_attempt(now_ms, false, io);
                }
            }
        }
    }

    /// TX-OFF, the inhibit, the record: the emissions cease for good
    fn silence<IO: PassivationIo>(&mut self, io: &mut IO) {
        io.tx_off();
        self.silenced = true;
        self.save(io);
        io.inhibit();
        io.report(Report::Silenced);
    }

    /// The next final-report frame, if one is due now: its number (1 to
    /// [`FINAL_REPORTS`]); the caller downlinks a housekeeping frame
    pub fn take_report(&mut self, now_ms: u64) -> Option<u8> {
        let (start, sent) = self.report?;
        let due = now_ms >= start.saturating_add(u64::from(sent) * REPORT_INTERVAL_MS);
        if sent >= FINAL_REPORTS || !due {
            return None;
        }
        self.report = Some((start, sent + 1));
        Some(sent + 1)
    }

    fn fire_min_served(&self, now_ms: u64) -> bool {
        matches!(self.fire, Some(Step::Fire(since)) if now_ms.saturating_sub(since) >= FIRE_MIN_MS)
    }

    /// Check the RAM copy against the record (AV-07): committed or silenced
    /// in either is so in both, and the inhibit is applied again once
    /// silenced
    pub fn verify<IO: PassivationIo>(&mut self, io: &mut IO) {
        if !self.initialized {
            return;
        }
        let mut rec = [0u8; 4];
        let stored =
            if nvm::load(io, self.region, 0, RECORD_MAGIC, &mut rec) == Ok(RecordStatus::Valid) { rec[0] } else { 0 };
        let ram = self.record()[0];
        if stored & FLAG_SILENCED != 0 && !self.silenced {
            self.silenced = true;
            self.committed = true;
            io.report(Report::Passivated);
        } else if stored & FLAG_COMMITTED != 0 && !self.committed {
            // Committed on record, not in RAM: carry on from the record
            self.set_flags(stored);
            if !self.fire_done && self.fire.is_none() {
                self.fire = Some(Step::WaitSun);
            }
        } else if ram & !stored != 0 {
            self.save(io);
        }
        if self.silenced {
            io.inhibit();
        }
    }

    /// Where the sequence is
    pub fn phase(&self, now_ms: u64) -> Phase {
        match (self.committed || self.silenced, self.silenced, self.fire_done) {
            (true, true, _) => Phase::Passivated,
            (true, false, true) => Phase::Reporting,
            (true, false, false) => Phase::Firing,
            (false, ..) if self.armed_until.is_some_and(|until| now_ms < until) => Phase::Armed,
            (false, ..) => Phase::Idle,
        }
    }

    /// Whether the transmitters are off for good (silenced)
    pub const fn passivated(&self) -> bool {
        self.silenced
    }

    /// Whether PASSIVATE_EXECUTE has been accepted (the record committed)
    pub const fn committed(&self) -> bool {
        self.committed
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

    /// Run the final report from `t`: the frames handed out and their times
    fn run_report(p: &mut Passivation, io: &mut Bench, t: u64) -> Vec<(u64, u8)> {
        let mut frames = Vec::new();
        let mut now = t;
        while now <= t + REPORT_WINDOW_MS + 1000 {
            if let Some(n) = p.take_report(now) {
                frames.push((now, n));
            }
            p.tick(now, SUN, io);
            now += 1000;
        }
        frames
    }

    #[test]
    fn fire_while_transmitting_then_report_then_silence() {
        let mut io = Bench::default();
        let mut p = fresh(&mut io);
        assert_eq!(p.phase(0), Phase::Idle);
        assert_eq!(p.execute(1000, &mut io), Err(Refusal::NotArmed), "execute alone does nothing");
        assert_eq!(p.arm(b"EOL?", 1000, &mut io), Err(Refusal::BadCode));
        assert_eq!(p.arm(&ARM_CODE, 1000, &mut io), Ok(()));
        assert_eq!(p.phase(1000), Phase::Armed);
        assert!(io.drives.is_empty(), "arming drives nothing");
        assert!(io.tx_off == 0, "arming drives nothing");
        assert_eq!(p.execute(2000, &mut io), Ok(()));
        assert!(p.committed());
        assert!(!p.passivated());
        assert_eq!(io.tx_off, 0, "still transmitting during the fire");
        assert!(!io.inhibited);
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
        assert_eq!(p.take_report(t0 + FIRE_MIN_MS - 1), None, "no report before the fire ends");
        let t1 = t0 + FIRE_MIN_MS;
        p.tick(t1, SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        assert_eq!(p.phase(t1), Phase::Reporting);
        assert!(p.confirmed());
        assert!(io.tx_off == 0);
        // Three frames a minute apart, then TX-OFF and the inhibit
        let frames = run_report(&mut p, &mut io, t1);
        assert_eq!(frames, [(t1, 1), (t1 + 60_000, 2), (t1 + 120_000, 3)]);
        assert_eq!(io.tx_off, 1);
        assert!(io.inhibited);
        assert!(p.passivated());
        assert_eq!(p.phase(u64::MAX), Phase::Passivated);
        assert_eq!(p.attempts(), 1);
        assert_eq!(io.log, [Report::Armed, Report::Executed, Report::ChargePathOpen, Report::Silenced]);
        assert_eq!(p.arm(&ARM_CODE, 7000, &mut io), Err(Refusal::Passivated));
        assert_eq!(p.execute(7000, &mut io), Err(Refusal::Passivated));
        assert_eq!(p.take_report(u64::MAX), None);
        // Every boot after: inhibited at once
        let mut io2 = Bench { flash: io.flash, ..Bench::default() };
        let b = fresh(&mut io2);
        assert!(b.passivated());
        assert!(io2.inhibited);
        assert!(io2.drives.is_empty());
        assert_eq!(io2.log, [Report::Passivated]);
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
        assert!(!p.committed());
        assert!(io.drives.is_empty());
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
        // Past the minimum an eclipse ends it, unconfirmed: tried again
        p.tick(300_000 + ARM_SETTLE_MS + FIRE_MIN_MS, !SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        assert_eq!(io.log.last(), Some(&Report::NotConfirmed));
        assert_eq!(p.attempts(), 2);
        assert_eq!(p.phase(0), Phase::Firing);
    }

    #[test]
    fn an_unconfirmed_fire_is_retried_then_reported_and_silenced() {
        let mut io = Bench::default();
        let t0 = 1000;
        let mut p = executed(&mut io, t0);
        let mut t = t0;
        for attempt in 1..=MAX_FIRE_ATTEMPTS {
            assert_eq!(p.attempts(), attempt);
            assert_eq!(io.lines, [true, true]);
            p.tick(t + FIRE_MAX_MS - 1, SUN, &mut io);
            assert_eq!(io.lines, [true, true]);
            t += FIRE_MAX_MS;
            p.tick(t, SUN, &mut io);
            assert_eq!(io.lines, [false, false]);
            if attempt < MAX_FIRE_ATTEMPTS {
                p.tick(t + 1, SUN, &mut io);
                t += 1 + ARM_SETTLE_MS;
                p.tick(t, SUN, &mut io);
            }
        }
        assert_eq!(p.phase(t), Phase::Reporting);
        assert!(!p.confirmed());
        assert_eq!(io.tx_off, 0);
        let frames = run_report(&mut p, &mut io, t);
        assert_eq!(frames.len(), usize::from(FINAL_REPORTS));
        assert!(p.passivated(), "the emissions cease anyway");
        assert!(io.tx_off == 1, "the emissions cease anyway");
        assert!(io.inhibited, "the emissions cease anyway");
    }

    #[test]
    fn resets_during_the_fire_and_the_report() {
        let mut io = Bench::default();
        let _ = executed(&mut io, 1000);
        for attempt in 2..=MAX_FIRE_ATTEMPTS {
            let mut io2 = Bench { flash: io.flash, ..Bench::default() };
            let mut b = fresh(&mut io2);
            assert_eq!(b.attempts(), attempt);
            assert_eq!(b.phase(0), Phase::Firing);
            assert!(!io2.inhibited, "still transmitting after the reset");
            b.tick(5, SUN, &mut io2);
            assert_eq!(io2.drives, [(Line::Arm, true)], "fire restarted");
            io.flash = io2.flash;
        }
        // Attempts spent: the next boot goes to the report
        let mut io3 = Bench { flash: io.flash, ..Bench::default() };
        let mut b = fresh(&mut io3);
        assert_eq!(b.phase(0), Phase::Reporting);
        assert_eq!(io3.drives, [(Line::Fire, false), (Line::Arm, false)]);
        assert_eq!(b.take_report(0), Some(1));
        // A reset in the report: the report again from its start
        let mut io4 = Bench { flash: io3.flash, ..Bench::default() };
        let mut c = fresh(&mut io4);
        assert_eq!(c.phase(0), Phase::Reporting);
        assert_eq!(run_report(&mut c, &mut io4, 0).len(), 3);
        assert!(c.passivated());
        assert!(io4.inhibited);
    }

    #[test]
    fn missing_lines_and_unwritable_records() {
        // No pin for the arm latch: straight to the report, then silence
        let mut io = Bench { no_pins: true, ..Bench::default() };
        let mut p = fresh(&mut io);
        let _ = p.arm(&ARM_CODE, 0, &mut io);
        assert_eq!(p.execute(0, &mut io), Ok(()));
        p.tick(1, SUN, &mut io);
        assert!(io.log.contains(&Report::LineUnavailable));
        assert_eq!(p.phase(1), Phase::Reporting);
        run_report(&mut p, &mut io, 1);
        assert!(io.inhibited);
        assert!(io.tx_off == 1);
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
        assert!(p.committed());
    }

    #[test]
    fn verify_restores_passivation_from_either_copy() {
        let mut io = Bench::default();
        let mut p = Passivation::new(REGION);
        p.verify(&mut io);
        assert!(!io.inhibited, "before init: nothing");
        p.init(0, &mut io);
        p.verify(&mut io);
        assert!(!io.inhibited);
        assert!(io.log.is_empty());
        // Committed on record, a RAM upset forgot it: carried on
        let mut q = executed(&mut io, 1000);
        q.committed = false;
        q.fire = None;
        q.verify(&mut io);
        assert!(q.committed());
        assert!(q.fire.is_some());
        assert!(!io.inhibited);
        // Silenced on record, RAM not: silenced and inhibited again
        q.fire = None;
        q.report = Some((0, 3));
        q.tick(REPORT_WINDOW_MS, SUN, &mut io);
        assert!(q.passivated());
        assert!(io.inhibited);
        q.silenced = false;
        io.inhibited = false;
        q.verify(&mut io);
        assert!(q.passivated());
        assert!(io.inhibited);
        assert_eq!(io.log.last(), Some(&Report::Passivated));
        // The record lost, RAM committed: written again
        let mut io2 = Bench::default();
        let mut r = fresh(&mut io2);
        r.committed = true;
        r.verify(&mut io2);
        let after = fresh(&mut io2);
        assert!(after.committed());
        // A version-1 record (TX-OFF first) reads as silenced
        let mut io5 = Bench::default();
        nvm::save(&mut io5, REGION, 0, RECORD_MAGIC, &[FLAG_COMMITTED, 1, 1, 0]).unwrap();
        let old = fresh(&mut io5);
        assert!(old.passivated());
        assert!(io5.inhibited);
    }

    /// Records the software does not write itself: a version-1 record not
    /// committed stays idle; a silenced flag alone (an upset) still reads
    /// as passivated; verify with the fire already done on record restarts
    /// nothing
    #[test]
    fn unusual_records_are_read_safely() {
        let mut io = Bench::default();
        nvm::save(&mut io, REGION, 0, RECORD_MAGIC, &[0, 1, 0, 0]).unwrap();
        let p = fresh(&mut io);
        assert!(!p.committed());
        assert!(!p.passivated());
        assert_eq!(p.phase(0), Phase::Idle);

        let mut io = Bench::default();
        nvm::save(&mut io, REGION, 0, RECORD_MAGIC, &[FLAG_SILENCED, VERSION, 0, 0]).unwrap();
        let p = fresh(&mut io);
        assert!(!p.committed());
        assert_eq!(p.phase(0), Phase::Passivated);
        assert!(io.inhibited);

        // Committed and the fire done on record, RAM forgot both: no fire
        let mut io = Bench::default();
        let flags = FLAG_COMMITTED | FLAG_FIRE_DONE;
        nvm::save(&mut io, REGION, 0, RECORD_MAGIC, &[flags, VERSION, 1, 0]).unwrap();
        let mut q = Passivation::new(REGION);
        q.initialized = true;
        q.verify(&mut io);
        assert!(q.committed());
        assert!(q.fire.is_none());
        assert_eq!(q.phase(0), Phase::Reporting);
        // Committed on record while a fire is already running in RAM: kept
        let mut io = Bench::default();
        let mut r = executed(&mut io, 1000);
        r.committed = false;
        let running = r.fire;
        r.verify(&mut io);
        assert_eq!(r.fire, running);
    }

    #[test]
    fn the_eps_saying_open_is_passivation_even_without_a_record() {
        let mut io = Bench { open: Some(true), ..Bench::default() };
        let p = fresh(&mut io);
        assert!(p.passivated());
        assert!(p.confirmed());
        assert!(io.inhibited);
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
        assert!(!b.committed());
        assert_eq!(io2.log, [Report::Unreadable]);
    }
}
