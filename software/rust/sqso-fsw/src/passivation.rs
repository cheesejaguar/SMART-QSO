//! End-of-life passivation by telecommand (`passivation.h`; BASELINE §5,
//! 47 CFR 97.207(g)(1)(iii); RIDs REG-01, REG-12, EPS-06, FSW-07, SMA-13,
//! P3E-11).
//!
//! Two authenticated telecommands from two key custodians, and two
//! processors for the arm (SQSO-EPS-BRD-001 D §7, SMART-QSO-ICD-001 1.5
//! §4.6):
//!
//! 1. `PASSIVATE_ARM` (operational key, argument the ASCII code `EOL!`)
//!    arms the EPS's one-shot charge disconnect: the EPS controller's
//!    `EOL_STATE` must show the arm latch clear and the fuse F1 intact; the
//!    flight software writes the key pair [`KEY`] to `EOL_KEY` (the
//!    controller then raises its own EOL_KEY line for 10 s, `EOL_STATE` bit
//!    3), raises `EOL_ARM` (the latch clocks only while both are high), and
//!    once `EOL_STATE` shows the latch set (at most [`ARM_CONFIRM_MS`])
//!    drops `EOL_ARM`. `PASSIVATE_EXECUTE` is then accepted for
//!    [`ARM_WINDOW_MS`] from the command. The software arm is kept in RAM
//!    only (a reset, or the window running out, disarms it); the EPS's latch
//!    is cleared only at the EPS's power-up.
//! 2. `PASSIVATE_EXECUTE` (the offline recovery key) inside that window,
//!    with the latch still set, commits the passivation (the record,
//!    persisted: from here on every boot carries it through) and runs, in
//!    this order:
//!    - the fire, in sunlight only (the array then carries the loads while
//!      the heater draws from the pack) and only with the EPS board at or
//!      above [`FIRE_MIN_BOARD_C`] by the INA228 die temperatures (the fuse
//!      F1, ITV4030, is rated −10 … +65 °C; SMART-QSO-ICD-001 1.6 §4.6
//!      amended, SQSO-EPS-BRD-001 F §7; an invalid or missing temperature
//!      is not warm enough), **while the spacecraft still
//!      transmits** so the ground can see the result: `EOL_FIRE` (a level, in
//!      series with the latch through F1's heater) high at least
//!      [`FIRE_MIN_MS`]; it drops once the EPS reports the charge path open
//!      (`EOL_STATE` bit 1 set and bit 2, F1 intact, clear) after that, or at
//!      [`FIRE_MAX_MS`]. An eclipse before [`FIRE_MIN_MS`] drops it and the
//!      fire starts again in the next sunlight; a fire the EPS does not
//!      confirm is tried again, at most [`MAX_FIRE_ATTEMPTS`] times in all
//!      (across resets too). The EPS reporting the charge path open ends
//!      the fire whenever it is seen (the array, and with it the sunlight
//!      the software sees, is gone once it opens); an attempt not ended
//!      within [`FIRE_DEADLINE_MS`] (no sunlight, or the board never warm
//!      enough) goes to the report all the same. While it waits, the reason
//!      ([`WAIT_SUN`], [`WAIT_BOARD`]) is kept for housekeeping (HK1 `PS`)
//!      and stays in the final report;
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
//! at the next boot (within the attempts; the EPS's latch survives an OBC
//! reset); a reset during the report sends the report again, then silences.
//!
//! @requirement SRS-F110 Two-step authenticated end-of-life passivation
//! @requirement SRS-F111 Passivation order: record, charge disconnect through two series lines while transmitting, final report, then TX-OFF
//! @requirement SRS-F112 Transmitters off for good after passivation
//! @requirement SRS-F155 The final passivation report: three frames over three minutes carrying EOL_STATE, the fuse state and the attempts, before TX-OFF
//! @requirement SRS-F165 The passivation fire starts only in sunlight with the EPS board at or above 0 °C (an invalid temperature counts as cold); it waits, reporting why, up to the 24 h deadline
//! @requirement SRS-F157 Two-processor passivation arm: the EPS controller's key and the OBC's EOL_ARM, the latch confirmed before PASSIVATE_EXECUTE is accepted

use crate::nvm::{self, Flash, RecordStatus};

/// How long PASSIVATE_ARM holds, ms
pub const ARM_WINDOW_MS: u64 = 600_000;
/// PASSIVATE_ARM's argument
pub const ARM_CODE: [u8; 4] = *b"EOL!";
/// The two bytes written to the EPS controller's `EOL_KEY` register (0x7E),
/// in this order, within 1 s
pub const KEY: [u8; 2] = [0x3C, 0xC3];
/// `EOL_ARM` is held high at most this long for the latch to show set, ms
/// (inside the EPS controller's 10 s key)
pub const ARM_CONFIRM_MS: u64 = 2_000;
/// `EOL_FIRE` is held at least this long, ms (the EPS board's requirement:
/// the thermal cut-off's heater needs it to open the disconnect)
pub const FIRE_MIN_MS: u64 = 60_000;
/// and at most this long, ms, when the EPS does not confirm
pub const FIRE_MAX_MS: u64 = 120_000;
/// The fire starts only with the EPS board at or above this, °C (F1's
/// rating starts at −10 °C)
pub const FIRE_MIN_BOARD_C: f64 = 0.0;
/// An INA228 die temperature outside this range is not a reading, °C (the
/// part's specified range)
pub const BOARD_VALID_C: (f64, f64) = (-40.0, 125.0);
/// Waiting reason: no sunlight
pub const WAIT_SUN: u8 = 0x01;
/// Waiting reason: the EPS board below [`FIRE_MIN_BOARD_C`] or its
/// temperature invalid
pub const WAIT_BOARD: u8 = 0x02;

/// The EPS board's temperature from the two INA228 die temperatures: the
/// lower of the valid ones (`None`: neither valid)
pub fn board_c(dies: [Option<f64>; 2]) -> Option<f64> {
    dies.into_iter()
        .flatten()
        .filter(|t| (BOARD_VALID_C.0..=BOARD_VALID_C.1).contains(t))
        .reduce(f64::min)
}

/// Fire sequences at most, across resets
pub const MAX_FIRE_ATTEMPTS: u8 = 3;
/// The fire gives up this long after it started (or resumed after a
/// reset) without the attempts ending, ms: the emissions cease even if
/// the sunlight that the fire waits for is never seen again
pub const FIRE_DEADLINE_MS: u64 = 86_400_000;
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

/// The bits of the EPS controller's `EOL_STATE` register (0x7C)
pub mod eol_state {
    /// The arm latch is set
    pub const ARMED: u8 = 0x01;
    /// The fire has opened the charge path
    pub const FIRED: u8 = 0x02;
    /// The fuse F1 is intact (F1_SNS)
    pub const FUSE_INTACT: u8 = 0x04;
    /// The controller's EOL_KEY is high (the key accepted, for 10 s)
    pub const KEY: u8 = 0x08;

    /// The charge path is open: fired and the fuse no longer intact
    pub const fn charge_path_open(state: u8) -> bool {
        state & FIRED != 0 && state & FUSE_INTACT == 0
    }
}
use eol_state::{ARMED, FUSE_INTACT};

/// Whether the EPS reports the charge path open
fn path_open<IO: PassivationIo>(io: &mut IO) -> bool {
    io.eol_state().is_some_and(eol_state::charge_path_open)
}

/// The two series lines of the charge disconnect
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    /// `GPIO_PIN_EOL_ARM`: clocks the arm latch (with the EPS's key)
    Arm,
    /// `GPIO_PIN_EOL_FIRE`: the fire switch
    Fire,
}

/// Events for the fault log
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// Armed: the latch set, PASSIVATE_EXECUTE accepted for [`ARM_WINDOW_MS`]
    Armed,
    /// The arm lapsed unused
    Lapsed,
    /// The EPS controller refused the key (or did not answer)
    KeyRefused,
    /// `EOL_ARM` rose but the latch did not show set in [`ARM_CONFIRM_MS`]
    ArmFailed,
    /// Executed: the record committed, the fire started (still transmitting)
    Executed,
    /// The passivated record could not be written (the act goes on)
    NotPersisted,
    /// A line could not be driven
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
    /// The EPS controller's `EOL_STATE` (`None`: no answer)
    fn eol_state(&mut self) -> Option<u8>;
    /// The EPS board's temperature, °C ([`board_c`]; `None`: invalid)
    fn eps_board_c(&mut self) -> Option<f64>;
    /// Write [`KEY`] to the EPS controller's `EOL_KEY`; false if a write failed
    fn write_key(&mut self) -> bool;
    /// Log an event
    fn report(&mut self, report: Report);
}

/// Why a command was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The arming code is wrong
    BadCode,
    /// PASSIVATE_EXECUTE without a live arm, or with the latch not set
    NotArmed,
    /// Already passivated (committed)
    Passivated,
    /// The EPS controller does not answer, or shows the fuse not intact
    EpsNotReady,
    /// The EPS controller refused the key
    KeyRefused,
    /// `EOL_ARM` cannot be driven
    LineUnavailable,
}

/// Where the sequence is (`PassivationPhase_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Nothing armed
    Idle = 0,
    /// Arming (the latch being confirmed) or armed until the window ends
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
    /// Waiting for sunlight; `EOL_FIRE` low
    WaitSun,
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
    /// `EOL_ARM` high since this time, ms, and the window's end
    arming: Option<(u64, u64)>,
    /// Armed (the latch confirmed) until this time, ms
    armed_until: Option<u64>,
    /// Where the fire sequence is (`None`: not firing)
    fire: Option<Step>,
    /// The fire started (or resumed) at this time, ms
    fire_from: u64,
    /// The final report: when it started, ms, and the frames handed out
    report: Option<(u64, u8)>,
    /// Why the fire waits ([`WAIT_SUN`], [`WAIT_BOARD`]; 0: not waiting)
    waiting: u8,
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
            arming: None,
            armed_until: None,
            fire: None,
            fire_from: 0,
            report: None,
            waiting: 0,
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
                // A version-1 record (TX-OFF first) was silenced when committed
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
        if !self.committed && io.eol_state().is_some_and(eol_state::charge_path_open) {
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

    /// PASSIVATE_ARM with its argument: the EPS checked, the key written,
    /// `EOL_ARM` raised (the latch confirmed by [`Passivation::tick`]). A
    /// latch already set (an earlier arm, an OBC reset since) is armed at once
    pub fn arm<IO: PassivationIo>(&mut self, code: &[u8], now_ms: u64, io: &mut IO) -> Result<(), Refusal> {
        if self.committed {
            return Err(Refusal::Passivated);
        }
        if code != ARM_CODE {
            return Err(Refusal::BadCode);
        }
        let until = now_ms.saturating_add(ARM_WINDOW_MS);
        let state = io.eol_state().filter(|s| s & FUSE_INTACT != 0).ok_or(Refusal::EpsNotReady)?;
        if state & ARMED != 0 {
            self.armed_until = Some(until);
            io.report(Report::Armed);
            return Ok(());
        }
        let keyed = io.write_key() && io.eol_state().is_some_and(|s| s & eol_state::KEY != 0);
        if !keyed {
            io.report(Report::KeyRefused);
            return Err(Refusal::KeyRefused);
        }
        if !io.drive(Line::Arm, true) {
            io.report(Report::LineUnavailable);
            return Err(Refusal::LineUnavailable);
        }
        self.arming = Some((now_ms, until));
        Ok(())
    }

    /// PASSIVATE_EXECUTE: the record committed, then the fire (the
    /// transmitters stay on until the final report). Refused unless armed
    /// and the EPS shows the latch still set
    pub fn execute<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) -> Result<(), Refusal> {
        if self.committed {
            return Err(Refusal::Passivated);
        }
        let live = self.armed_until.is_some_and(|until| now_ms < until);
        if !live || !io.eol_state().is_some_and(|s| s & ARMED != 0) {
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
        // Each attempt has its own deadline (an attempt starts with no fire
        // under way: at the execute, after a reset, after an unconfirmed one)
        self.fire_from = now_ms;
        self.fire = Some(Step::WaitSun);
    }

    /// `EOL_FIRE` low; a confirmed fire (or the attempts spent) goes to the
    /// final report, an unconfirmed one is tried again
    fn end_attempt<IO: PassivationIo>(&mut self, now_ms: u64, confirmed: bool, io: &mut IO) {
        let _ = io.drive(Line::Fire, false);
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
        self.fire = None;
        self.fire_done = true;
        self.confirmed = confirmed;
        self.save(io);
        self.report = Some((now_ms, 0));
    }

    /// The arm being confirmed: the latch set drops `EOL_ARM` and arms;
    /// no latch in [`ARM_CONFIRM_MS`] drops it and fails
    fn confirm_arm<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) {
        let Some((since, until)) = self.arming else { return };
        let latched = io.eol_state().is_some_and(|s| s & ARMED != 0);
        if !latched && now_ms.saturating_sub(since) < ARM_CONFIRM_MS {
            return;
        }
        let _ = io.drive(Line::Arm, false);
        self.arming = None;
        if latched {
            self.armed_until = Some(until);
            io.report(Report::Armed);
        } else {
            io.report(Report::ArmFailed);
        }
    }

    /// Every pass: the arm's confirmation and window, the fire sequence
    /// (`sunlit`: the array is lit) and the end of the final report
    pub fn tick<IO: PassivationIo>(&mut self, now_ms: u64, sunlit: bool, io: &mut IO) {
        self.confirm_arm(now_ms, io);
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
        if self.fire.is_some() && now_ms.saturating_sub(self.fire_from) >= FIRE_DEADLINE_MS {
            let open = path_open(io);
            self.finish(now_ms, open, io);
            return;
        }
        match self.fire {
            Some(Step::WaitSun) if path_open(io) => self.end_attempt(now_ms, true, io),
            Some(Step::WaitSun) => self.wait_or_fire(now_ms, sunlit, io),
            None => {}
            Some(Step::Fire(since)) => {
                let held = now_ms.saturating_sub(since);
                // The open path takes the array, and the sunlight seen, with it
                if path_open(io) && (held >= FIRE_MIN_MS || !sunlit) {
                    self.end_attempt(now_ms, true, io);
                } else if !sunlit && held < FIRE_MIN_MS {
                    // Eclipse before the minimum: low, again in sunlight
                    let _ = io.drive(Line::Fire, false);
                    self.fire = Some(Step::WaitSun);
                } else if held >= FIRE_MAX_MS || !sunlit {
                    self.end_attempt(now_ms, false, io);
                }
            }
        }
    }

    /// Waiting to fire: in sunlight with the EPS board warm enough, the fire
    /// starts; otherwise the reasons are kept
    fn wait_or_fire<IO: PassivationIo>(&mut self, now_ms: u64, sunlit: bool, io: &mut IO) {
        let warm = io.eps_board_c().is_some_and(|t| (FIRE_MIN_BOARD_C..=BOARD_VALID_C.1).contains(&t));
        self.waiting = if sunlit { 0 } else { WAIT_SUN } | if warm { 0 } else { WAIT_BOARD };
        if self.waiting != 0 {
            return;
        }
        if io.drive(Line::Fire, true) {
            self.fire = Some(Step::Fire(now_ms));
        } else {
            // No retry can drive it either: straight to the report
            io.report(Report::LineUnavailable);
            self.finish(now_ms, false, io);
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

    /// Check the RAM copy against the record (AV-07): committed or silenced
    /// in either is so in both, and the inhibit is applied again once
    /// silenced; a committed record carries on where it is (the fire again,
    /// or the final report again once the fire is done)
    pub fn verify<IO: PassivationIo>(&mut self, now_ms: u64, io: &mut IO) {
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
            } else if self.fire_done && self.report.is_none() {
                self.report = Some((now_ms, 0));
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
        let armed = self.arming.is_some() || self.armed_until.is_some_and(|until| now_ms < until);
        match (self.committed || self.silenced, self.silenced, self.fire_done) {
            (true, true, _) => Phase::Passivated,
            (true, false, true) => Phase::Reporting,
            (true, false, false) => Phase::Firing,
            (false, ..) if armed => Phase::Armed,
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

    /// Why the fire waits, or last waited before the final report
    /// ([`WAIT_SUN`], [`WAIT_BOARD`]; 0: not waiting)
    pub const fn waiting(&self) -> u8 {
        self.waiting
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
    use super::eol_state::{FIRED, KEY as KEY_BIT};
    use super::*;
    use crate::nvm::{FlashError, MemFlash, SLOT_SIZE};
    extern crate std;
    use std::vec::Vec;

    const REGION: u32 = 0;
    const SUN: bool = true;
    /// EOL_STATE after a successful fire: fired, the fuse open, still armed
    const OPEN: u8 = ARMED | FIRED;

    /// The OBC's view of the EPS: the flash, the two lines, the controller's
    /// `EOL_STATE` with the key and the latch (SQSO-EPS-BRD-001 D §7)
    #[derive(Default)]
    struct Bench {
        flash: MemFlash<256>,
        fail_writes: bool,
        tx_off: u32,
        inhibited: bool,
        lines: [bool; 2],
        no_arm_pin: bool,
        no_fire_pin: bool,
        /// `EOL_STATE` (`None`: no answer)
        eps: Option<u8>,
        key_bus_fails: bool,
        key_refused: bool,
        /// The latch does not set on EOL_ARM
        latch_broken: bool,
        log: Vec<Report>,
        /// (line, on) in the order driven
        drives: Vec<(Line, bool)>,
        /// The EPS board's temperature, °C (`None`: invalid)
        board: Option<f64>,
    }

    /// An EPS ready to be armed: the fuse intact, nothing armed
    fn ready() -> Bench {
        Bench { eps: Some(FUSE_INTACT), board: Some(12.0), ..Bench::default() }
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
            let missing = match line {
                Line::Arm => self.no_arm_pin,
                Line::Fire => self.no_fire_pin,
            };
            if on && missing {
                return false;
            }
            // The latch clocks on EOL_ARM's rising edge while the key is up
            if line == Line::Arm && on && !self.latch_broken {
                self.eps = self.eps.map(|s| if s & KEY_BIT != 0 { s | ARMED } else { s });
            }
            self.lines[line as usize] = on;
            self.drives.push((line, on));
            true
        }
        fn eol_state(&mut self) -> Option<u8> {
            self.eps
        }
        fn eps_board_c(&mut self) -> Option<f64> {
            self.board
        }
        fn write_key(&mut self) -> bool {
            if self.key_bus_fails {
                return false;
            }
            let accept = !self.key_refused;
            self.eps = self.eps.map(|s| if accept && s & ARMED == 0 && s & FUSE_INTACT != 0 { s | KEY_BIT } else { s });
            true
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

    /// Armed (the latch confirmed at t = 10 ms)
    fn armed(io: &mut Bench) -> Passivation {
        let mut p = fresh(io);
        assert_eq!(p.arm(&ARM_CODE, 0, io), Ok(()));
        p.tick(10, SUN, io);
        assert_eq!(io.log.last(), Some(&Report::Armed));
        p
    }

    /// Armed, executed, and `EOL_FIRE` high from `t0`
    fn executed(io: &mut Bench, t0: u64) -> Passivation {
        let mut p = armed(io);
        assert_eq!(p.execute(20, io), Ok(()));
        p.tick(t0, SUN, io);
        assert_eq!(io.lines, [false, true]);
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
    fn arm_with_the_key_then_fire_while_transmitting_then_report_then_silence() {
        let mut io = ready();
        let mut p = fresh(&mut io);
        assert_eq!(p.phase(0), Phase::Idle);
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Ok(()));
        assert_eq!(io.drives, [(Line::Arm, true)], "the key, then EOL_ARM");
        assert_eq!(io.eps, Some(FUSE_INTACT | KEY_BIT | ARMED));
        assert_eq!(p.phase(0), Phase::Armed);
        assert_eq!(p.execute(5, &mut io), Err(Refusal::NotArmed), "not before the latch is confirmed");
        p.tick(10, SUN, &mut io);
        assert_eq!(io.drives, [(Line::Arm, true), (Line::Arm, false)]);
        assert_eq!(io.log, [Report::Armed]);
        assert!(!p.committed());
        assert_eq!(io.tx_off, 0);

        assert_eq!(p.execute(1000, &mut io), Ok(()));
        assert!(p.committed());
        assert!(!p.passivated());
        assert_eq!(p.phase(1000), Phase::Firing);
        assert_eq!(p.attempts(), 1);
        p.tick(2000, !SUN, &mut io);
        assert_eq!(io.lines, [false, false], "eclipse: nothing fired");
        p.tick(3000, SUN, &mut io);
        assert_eq!(io.lines, [false, true]);
        io.eps = Some(OPEN);
        p.tick(3000 + FIRE_MIN_MS - 1, SUN, &mut io);
        assert_eq!(io.lines, [false, true], "held for the minimum");
        let t1 = 3000 + FIRE_MIN_MS;
        p.tick(t1, SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        assert!(p.confirmed());
        assert_eq!(io.tx_off, 0, "still transmitting");
        assert_eq!(p.phase(t1), Phase::Reporting);
        let frames = run_report(&mut p, &mut io, t1);
        assert_eq!(frames, [(t1, 1), (t1 + REPORT_INTERVAL_MS, 2), (t1 + 2 * REPORT_INTERVAL_MS, 3)]);
        assert!(p.passivated());
        assert!(io.inhibited);
        assert_eq!(io.tx_off, 1);
        assert_eq!(p.phase(u64::MAX), Phase::Passivated);
        assert_eq!(io.log.last(), Some(&Report::Silenced));
        assert_eq!(p.take_report(u64::MAX), None);
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Err(Refusal::Passivated));
        assert_eq!(p.execute(0, &mut io), Err(Refusal::Passivated));
        // A reset: silenced from the record, before anything transmits
        let mut io2 = Bench { flash: io.flash, eps: Some(OPEN), ..Bench::default() };
        let b = fresh(&mut io2);
        assert!(b.passivated());
        assert!(io2.inhibited);
        assert!(io2.drives.is_empty());
    }

    #[test]
    fn the_arm_is_refused_unless_the_eps_takes_the_key_and_the_latch_sets() {
        let mut io = ready();
        let mut p = fresh(&mut io);
        assert_eq!(p.arm(b"EOL?", 0, &mut io), Err(Refusal::BadCode));
        io.eps = None;
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Err(Refusal::EpsNotReady), "no answer");
        io.eps = Some(0);
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Err(Refusal::EpsNotReady), "the fuse not intact");
        io.eps = Some(FUSE_INTACT);
        io.key_bus_fails = true;
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Err(Refusal::KeyRefused), "the key not written");
        io.key_bus_fails = false;
        io.key_refused = true;
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Err(Refusal::KeyRefused), "the key not accepted");
        io.key_refused = false;
        io.no_arm_pin = true;
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Err(Refusal::LineUnavailable));
        assert_eq!(io.log, [Report::KeyRefused, Report::KeyRefused, Report::LineUnavailable]);
        assert!(io.drives.is_empty(), "nothing driven");
        assert_eq!(p.phase(0), Phase::Idle);

        // EOL_ARM rises but the latch never shows set: dropped, refused
        let mut io = Bench { latch_broken: true, ..ready() };
        let mut p = fresh(&mut io);
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Ok(()));
        p.tick(ARM_CONFIRM_MS - 1, SUN, &mut io);
        assert_eq!(io.lines, [true, false], "still waiting");
        p.tick(ARM_CONFIRM_MS, SUN, &mut io);
        assert_eq!(io.lines, [false, false]);
        assert_eq!(io.log, [Report::ArmFailed]);
        assert_eq!(p.phase(ARM_CONFIRM_MS), Phase::Idle);
        assert_eq!(p.execute(ARM_CONFIRM_MS, &mut io), Err(Refusal::NotArmed));

        // The latch already set (an earlier arm; an OBC reset since): armed at once
        let mut io = Bench { eps: Some(FUSE_INTACT | ARMED), ..Bench::default() };
        let mut p = fresh(&mut io);
        assert_eq!(p.arm(&ARM_CODE, 0, &mut io), Ok(()));
        assert!(io.drives.is_empty());
        assert_eq!(io.log, [Report::Armed]);
        assert_eq!(p.execute(1, &mut io), Ok(()));
    }

    #[test]
    fn the_arm_lapses_and_execute_needs_the_latch() {
        let mut io = ready();
        let mut p = armed(&mut io);
        assert_eq!(p.phase(ARM_WINDOW_MS - 1), Phase::Armed);
        p.tick(ARM_WINDOW_MS, SUN, &mut io);
        assert_eq!(io.log.last(), Some(&Report::Lapsed));
        assert_eq!(p.phase(ARM_WINDOW_MS), Phase::Idle);
        assert_eq!(p.execute(ARM_WINDOW_MS, &mut io), Err(Refusal::NotArmed));
        assert!(!p.committed());
        // Armed, but the EPS lost the latch (a power-up) or does not answer
        let mut io = ready();
        let mut p = armed(&mut io);
        io.eps = Some(FUSE_INTACT);
        assert_eq!(p.execute(20, &mut io), Err(Refusal::NotArmed));
        io.eps = None;
        assert_eq!(p.execute(20, &mut io), Err(Refusal::NotArmed));
        assert!(!p.committed());
    }

    #[test]
    fn an_eclipse_before_the_minimum_restarts_the_fire() {
        let mut io = ready();
        let mut p = executed(&mut io, 1000);
        p.tick(1000 + FIRE_MIN_MS - 1, !SUN, &mut io);
        assert_eq!(io.lines, [false, false], "dropped in eclipse");
        assert_eq!(p.phase(1000 + FIRE_MIN_MS), Phase::Firing);
        p.tick(200_000, SUN, &mut io);
        assert_eq!(io.lines, [false, true], "again in sunlight");
        assert_eq!(p.attempts(), 1, "the same attempt");
        // An eclipse after the minimum ends the attempt, unconfirmed
        p.tick(200_000 + FIRE_MIN_MS, !SUN, &mut io);
        assert_eq!(io.log.last(), Some(&Report::NotConfirmed));
        assert_eq!(p.attempts(), 2);
    }

    /// The fire waits for the EPS board at 0 °C as for the Sun (F1's rating),
    /// reporting why; an invalid temperature is cold; the deadline applies
    #[test]
    fn the_fire_waits_for_a_warm_eps_board() {
        assert_eq!(board_c([Some(3.0), Some(-1.5)]), Some(-1.5), "the colder die");
        assert_eq!(board_c([None, Some(4.0)]), Some(4.0));
        assert_eq!(board_c([Some(-80.0), Some(f64::NAN)]), None, "neither valid");
        assert_eq!(board_c([Some(130.0), None]), None);
        let mut io = ready();
        io.board = Some(-0.5);
        let mut p = armed(&mut io);
        assert_eq!(p.execute(20, &mut io), Ok(()));
        p.tick(2_000, SUN, &mut io);
        assert_eq!(io.lines, [false, false], "the board below 0 °C: no fire");
        assert_eq!(p.waiting(), WAIT_BOARD);
        p.tick(3_000, !SUN, &mut io);
        assert_eq!(p.waiting(), WAIT_SUN | WAIT_BOARD);
        io.board = None;
        p.tick(4_000, SUN, &mut io);
        assert_eq!(p.waiting(), WAIT_BOARD, "an invalid temperature is cold");
        assert_eq!(io.lines, [false, false]);
        assert_eq!(p.phase(4_000), Phase::Firing, "waiting, not aborted");
        assert_eq!(p.attempts(), 1, "no attempt spent while waiting");
        io.board = Some(0.0);
        p.tick(5_000, SUN, &mut io);
        assert_eq!(io.lines, [false, true], "at 0 °C in sunlight: the fire");
        assert_eq!(p.waiting(), 0);
        // Cold for a day: the report with the reason, then silence
        let mut io = ready();
        io.board = Some(-5.0);
        let mut p = armed(&mut io);
        assert_eq!(p.execute(20, &mut io), Ok(()));
        p.tick(1000, SUN, &mut io);
        p.tick(20 + FIRE_DEADLINE_MS - 1, SUN, &mut io);
        assert_eq!(p.phase(0), Phase::Firing);
        p.tick(20 + FIRE_DEADLINE_MS, SUN, &mut io);
        assert_eq!(p.phase(0), Phase::Reporting);
        assert_eq!(p.waiting(), WAIT_BOARD, "the reason stays in the final report");
        assert!(!p.confirmed());
        assert!(io.drives.iter().all(|&(l, on)| !(l == Line::Fire && on)), "never fired");
        let _ = run_report(&mut p, &mut io, 20 + FIRE_DEADLINE_MS);
        assert!(p.passivated());
    }

    /// The charge path opening takes the array (no sunlight seen after it):
    /// the open path ends the fire in eclipse or while waiting for the Sun;
    /// a fire that never sees the Sun gives up at the deadline
    #[test]
    fn the_open_path_ends_the_fire_whatever_the_sunlight() {
        let mut io = ready();
        let mut p = executed(&mut io, 1000);
        io.eps = Some(OPEN);
        p.tick(1000 + 10_000, !SUN, &mut io);
        assert!(p.confirmed(), "open in eclipse (the array gone)");
        assert_eq!(io.lines, [false, false]);
        // Opened while waiting for the Sun
        let mut io = ready();
        let mut p = armed(&mut io);
        assert_eq!(p.execute(20, &mut io), Ok(()));
        io.eps = Some(OPEN);
        p.tick(30, !SUN, &mut io);
        assert!(p.confirmed());
        // No Sun for a day: the report, then silence
        let mut io = ready();
        let mut p = armed(&mut io);
        assert_eq!(p.execute(20, &mut io), Ok(()));
        p.tick(20 + FIRE_DEADLINE_MS - 1, !SUN, &mut io);
        assert_eq!(p.phase(0), Phase::Firing);
        p.tick(20 + FIRE_DEADLINE_MS, !SUN, &mut io);
        assert_eq!(p.phase(0), Phase::Reporting);
        assert!(!p.confirmed());
        let _ = run_report(&mut p, &mut io, 20 + FIRE_DEADLINE_MS);
        assert!(p.passivated());
        // The same with the path found open at the deadline
        let mut io = ready();
        let mut p = executed(&mut io, 1000);
        io.eps = Some(OPEN | FUSE_INTACT);
        p.tick(1000 + 10, SUN, &mut io);
        assert!(!p.confirmed(), "fired but the fuse intact: not open");
        io.eps = Some(OPEN);
        p.fire_from = 0;
        p.tick(FIRE_DEADLINE_MS, SUN, &mut io);
        assert!(p.confirmed());
    }

    #[test]
    fn an_unconfirmed_fire_is_retried_then_reported_and_silenced() {
        let mut io = ready();
        let mut p = executed(&mut io, 1000);
        let mut t = 1000;
        for attempt in 1..=MAX_FIRE_ATTEMPTS {
            assert_eq!(p.attempts(), attempt);
            p.tick(t + FIRE_MAX_MS - 1, SUN, &mut io);
            assert_eq!(io.lines, [false, true]);
            t += FIRE_MAX_MS;
            p.tick(t, SUN, &mut io);
            assert_eq!(io.lines, [false, false]);
            t += 1;
            p.tick(t, SUN, &mut io);
        }
        assert_eq!(p.attempts(), MAX_FIRE_ATTEMPTS, "no fourth");
        assert_eq!(p.phase(t), Phase::Reporting);
        assert!(!p.confirmed());
        let frames = run_report(&mut p, &mut io, t);
        assert_eq!(frames.len(), 3);
        assert!(p.passivated());
        assert_eq!(io.tx_off, 1, "the emissions cease anyway");
        assert!(io.inhibited);
    }

    #[test]
    fn resets_during_the_fire_and_the_report() {
        let mut io = ready();
        let _ = executed(&mut io, 1000);
        // During the fire: the fire again, within the attempts
        let mut io2 = Bench { flash: io.flash, eps: Some(FUSE_INTACT | ARMED), board: Some(12.0), ..Bench::default() };
        let mut b = fresh(&mut io2);
        assert_eq!(b.phase(0), Phase::Firing);
        assert_eq!(b.attempts(), 2);
        b.tick(0, SUN, &mut io2);
        assert_eq!(io2.drives, [(Line::Fire, true)], "fire restarted");
        io2.eps = Some(OPEN);
        b.tick(FIRE_MIN_MS, SUN, &mut io2);
        assert_eq!(b.phase(FIRE_MIN_MS), Phase::Reporting);
        // During the report: the report again, then silenced
        let mut io3 = Bench { flash: io2.flash, eps: Some(OPEN), ..Bench::default() };
        let mut c = fresh(&mut io3);
        assert_eq!(c.phase(0), Phase::Reporting);
        assert_eq!(c.take_report(0), Some(1));
        c.tick(REPORT_WINDOW_MS, SUN, &mut io3);
        assert!(c.passivated());
        assert!(io3.inhibited);
        // A version-1 record (TX-OFF first) reads as silenced
        let mut io4 = Bench::default();
        nvm::save(&mut io4, REGION, 0, RECORD_MAGIC, &[FLAG_COMMITTED, 1, 1, 0]).unwrap();
        let old = fresh(&mut io4);
        assert!(old.passivated());
        assert!(io4.inhibited);
    }

    #[test]
    fn missing_lines_and_unwritable_records() {
        // EOL_FIRE cannot be driven: the fire ends at once, reported, silenced
        let mut io = Bench { no_fire_pin: true, ..ready() };
        let mut p = armed(&mut io);
        assert_eq!(p.execute(20, &mut io), Ok(()));
        p.tick(30, SUN, &mut io);
        assert_eq!(io.log.last(), Some(&Report::LineUnavailable));
        assert_eq!(p.phase(30), Phase::Reporting);
        let _ = run_report(&mut p, &mut io, 30);
        assert!(p.passivated());
        assert_eq!(io.tx_off, 1);
        // The record cannot be written: logged, the act goes on
        let mut io = ready();
        let mut p = armed(&mut io);
        io.fail_writes = true;
        assert_eq!(p.execute(20, &mut io), Ok(()));
        assert!(io.log.contains(&Report::NotPersisted));
        assert!(p.committed());
    }

    /// RID AV-07: the RAM copy against the record
    #[test]
    fn verify_restores_passivation_from_either_copy() {
        let mut io = ready();
        let mut p = Passivation::new(REGION);
        p.verify(0, &mut io);
        assert!(!io.inhibited, "before init: nothing");
        p.init(0, &mut io);
        p.verify(0, &mut io);
        assert!(!io.inhibited);
        assert!(io.log.is_empty());
        // Committed on record, a RAM upset forgot it: carried on
        let mut q = executed(&mut io, 1000);
        q.committed = false;
        q.fire = None;
        q.verify(0, &mut io);
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
        q.verify(0, &mut io);
        assert!(q.passivated());
        assert!(io.inhibited);
        assert_eq!(io.log.last(), Some(&Report::Passivated));
        // The record lost, RAM committed: written again
        let mut io2 = ready();
        let mut r = fresh(&mut io2);
        r.committed = true;
        r.verify(0, &mut io2);
        let after = fresh(&mut io2);
        assert!(after.committed());
    }

    /// Records the software does not write itself: a version-1 record not
    /// committed stays idle; a silenced flag alone (an upset) still reads
    /// as passivated; verify with the fire already done on record restarts
    /// only the report
    #[test]
    fn unusual_records_are_read_safely() {
        let mut io = ready();
        nvm::save(&mut io, REGION, 0, RECORD_MAGIC, &[0, 1, 0, 0]).unwrap();
        let p = fresh(&mut io);
        assert!(!p.committed());
        assert!(!p.passivated());
        assert_eq!(p.phase(0), Phase::Idle);

        let mut io = ready();
        nvm::save(&mut io, REGION, 0, RECORD_MAGIC, &[FLAG_SILENCED, VERSION, 0, 0]).unwrap();
        let p = fresh(&mut io);
        assert!(!p.committed());
        assert_eq!(p.phase(0), Phase::Passivated);
        assert!(io.inhibited);

        // Committed and the fire done on record, RAM forgot both: the report
        let mut io = ready();
        let flags = FLAG_COMMITTED | FLAG_FIRE_DONE;
        nvm::save(&mut io, REGION, 0, RECORD_MAGIC, &[flags, VERSION, 1, 0]).unwrap();
        let mut q = Passivation::new(REGION);
        q.initialized = true;
        q.verify(0, &mut io);
        assert!(q.committed());
        assert!(q.fire.is_none());
        assert_eq!(q.phase(0), Phase::Reporting);
        assert_eq!(q.take_report(0), Some(1), "the final report again");
        q.tick(REPORT_WINDOW_MS, SUN, &mut io);
        assert!(q.passivated());
        // The same while the report already runs in RAM: it carries on
        let mut io = ready();
        nvm::save(&mut io, REGION, 0, RECORD_MAGIC, &[flags, VERSION, 1, 0]).unwrap();
        let mut q = Passivation::new(REGION);
        q.initialized = true;
        q.fire_done = true;
        q.report = Some((0, 2));
        q.verify(0, &mut io);
        assert!(q.committed(), "carried on from the record");
        assert!(!q.passivated());
        assert_eq!(q.report, Some((0, 2)));
        // Committed on record while a fire is already running in RAM: kept
        let mut io = ready();
        let mut r = executed(&mut io, 1000);
        r.committed = false;
        let running = r.fire;
        r.verify(0, &mut io);
        assert_eq!(r.fire, running);
    }

    #[test]
    fn the_eps_saying_open_is_passivation_even_without_a_record() {
        let mut io = Bench { eps: Some(OPEN), ..Bench::default() };
        let p = fresh(&mut io);
        assert!(p.passivated());
        assert!(p.confirmed());
        assert!(io.inhibited);
        assert!(io.drives.is_empty());
        assert!(eol_state::charge_path_open(FIRED));
        assert!(!eol_state::charge_path_open(FIRED | FUSE_INTACT), "fired but the fuse intact: not open");
        assert!(!eol_state::charge_path_open(0));
        // Corrupt record, EPS silent: not passivated, reported
        let mut io = ready();
        let _ = executed(&mut io, 1000);
        let slot = u32::try_from(SLOT_SIZE).unwrap();
        for s in 0..2u32 {
            let mut b = [0u8; 1];
            io.flash.read(REGION, s * slot + 8, &mut b).unwrap();
            io.flash.write(REGION, s * slot + 8, &[b[0] ^ 0xFF]).unwrap();
        }
        let mut io2 = Bench { flash: io.flash, eps: None, ..Bench::default() };
        let b = fresh(&mut io2);
        assert!(!b.committed());
        assert_eq!(io2.log, [Report::Unreadable]);
    }
}
