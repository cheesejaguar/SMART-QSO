//! Loss-of-contact transmit inhibit (`contact_monitor.h`).
//!
//! If no authenticated telecommand arrives for the contact timeout (N
//! days), every transmitter is inhibited; the receiver stays on, and the
//! next authenticated telecommand lifts the inhibit (SQSO-ICD-013 §6.2).
//!
//! There is no reset-proof clock, so the monitor keeps an accumulated
//! seconds-without-contact counter, advanced by [`ContactMonitor::tick`] and
//! persisted every [`PERSIST_INTERVAL_S`]. Every error makes the inhibit come
//! earlier, never later: each boot that finds a record adds
//! [`PERSIST_INTERVAL_S`] (time that may have passed unrecorded), and a
//! corrupt counter record counts as timed out.
//!
//! @requirement SYS-205 Cease transmissions after N days without an authenticated telecommand

use crate::nvm::{self, Flash, RecordStatus};

/// Default timeout, days (decision D3)
pub const TIMEOUT_DEFAULT_DAYS: u16 = 30;
/// Shortest settable timeout, days
pub const TIMEOUT_MIN_DAYS: u16 = 1;
/// Longest settable timeout, days
pub const TIMEOUT_MAX_DAYS: u16 = 90;
/// Counter persistence interval, also the boot penalty, s
pub const PERSIST_INTERVAL_S: u32 = 60;
/// Offset of the counter record in the region
pub const NVM_COUNTER: u32 = 0;
/// Offset of the timeout record in the region
pub const NVM_TIMEOUT: u32 = nvm::FOOTPRINT;

/// Counter record ("LOCC")
pub const MAGIC_COUNTER: u32 = 0x4C4F_4343;
/// Timeout record ("LOCT")
pub const MAGIC_TIMEOUT: u32 = 0x4C4F_4354;
const SECONDS_PER_DAY: u32 = 86_400;

/// The flash, the inhibit on the transmitters, and fault reports
pub trait ContactIo: Flash {
    /// Apply or lift the loss-of-contact inhibit (`RF_INHIBIT_CONTACT`)
    fn set_inhibit(&mut self, active: bool);
    /// Log an event
    fn report(&mut self, report: Report);
}

/// Events for the fault log
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// The timeout was reached: transmitters inhibited (timeout in days)
    NoContact(u16),
    /// Boot: the counter record could not be read; counted as timed out
    CounterUnreadable,
}

/// Why setting the timeout failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Outside [`TIMEOUT_MIN_DAYS`]..=[`TIMEOUT_MAX_DAYS`]
    Invalid,
    /// Not persisted (applied until reset)
    Io,
}

/// Module state
#[derive(Clone, Copy, Debug)]
pub struct ContactMonitor {
    region: u32,
    /// Seconds without contact
    seconds: u32,
    /// Seconds since the last save
    unpersisted_s: u32,
    /// Sub-second carry
    remainder_ms: u32,
    timeout_days: u16,
    inhibited: bool,
    initialized: bool,
}

/// Whole seconds in `remainder + elapsed`, and the new remainder (in u64,
/// so a large `elapsed` cannot overflow)
pub(crate) fn carry_seconds(remainder_ms: u32, elapsed_ms: u32) -> (u32, u32) {
    let total = u64::from(remainder_ms) + u64::from(elapsed_ms);
    (u32::try_from(total / 1000).unwrap_or(u32::MAX), u32::try_from(total % 1000).unwrap_or(0))
}

impl ContactMonitor {
    /// Before init: default timeout, nothing counted
    pub const fn new(region: u32) -> Self {
        ContactMonitor {
            region,
            seconds: 0,
            unpersisted_s: 0,
            remainder_ms: 0,
            timeout_days: TIMEOUT_DEFAULT_DAYS,
            inhibited: false,
            initialized: false,
        }
    }

    fn timeout_seconds(&self) -> u32 {
        u32::from(self.timeout_days) * SECONDS_PER_DAY
    }

    fn save_counter<F: Flash>(&mut self, flash: &mut F) {
        if nvm::save(flash, self.region, NVM_COUNTER, MAGIC_COUNTER, &self.seconds.to_le_bytes()).is_ok() {
            self.unpersisted_s = 0;
        }
    }

    /// Apply the inhibit when the counter reaches the timeout
    fn evaluate<IO: ContactIo>(&mut self, io: &mut IO) {
        let inhibit = self.seconds >= self.timeout_seconds();
        if inhibit && !self.inhibited {
            io.report(Report::NoContact(self.timeout_days));
        }
        self.inhibited = inhibit;
        io.set_inhibit(inhibit);
    }

    /// Load the timeout and counter, apply the boot penalty, persist, and
    /// set the inhibit
    pub fn init<IO: ContactIo>(&mut self, io: &mut IO) {
        self.unpersisted_s = 0;
        self.remainder_ms = 0;
        self.inhibited = false;

        // Timeout: default unless a valid in-range record exists
        let mut word = [0u8; 4];
        self.timeout_days = match nvm::load(io, self.region, NVM_TIMEOUT, MAGIC_TIMEOUT, &mut word) {
            Ok(RecordStatus::Valid) => u16::try_from(u32::from_le_bytes(word))
                .ok()
                .filter(|d| (TIMEOUT_MIN_DAYS..=TIMEOUT_MAX_DAYS).contains(d))
                .unwrap_or(TIMEOUT_DEFAULT_DAYS),
            _ => TIMEOUT_DEFAULT_DAYS,
        };

        // Counter: blank -> 0; valid -> + boot penalty; corrupt -> timed out
        let mut word = [0u8; 4];
        self.seconds = match nvm::load(io, self.region, NVM_COUNTER, MAGIC_COUNTER, &mut word) {
            Ok(RecordStatus::Blank) => 0,
            Ok(RecordStatus::Valid) => u32::from_le_bytes(word).saturating_add(PERSIST_INTERVAL_S),
            Ok(RecordStatus::Corrupt) | Err(_) => {
                io.report(Report::CounterUnreadable);
                self.timeout_seconds()
            }
        };

        // Write at once so a reset loop accumulates the penalty
        self.save_counter(io);
        self.initialized = true;
        self.evaluate(io);
    }

    /// Advance the counter by elapsed time; inhibit at the timeout
    pub fn tick<IO: ContactIo>(&mut self, elapsed_ms: u32, io: &mut IO) {
        if !self.initialized {
            return;
        }
        let (seconds, remainder) = carry_seconds(self.remainder_ms, elapsed_ms);
        self.remainder_ms = remainder;
        self.seconds = self.seconds.saturating_add(seconds);
        self.unpersisted_s = self.unpersisted_s.saturating_add(seconds);
        if self.unpersisted_s >= PERSIST_INTERVAL_S {
            self.save_counter(io);
        }
        self.evaluate(io);
    }

    /// An authenticated telecommand: restart the count, lift the inhibit
    pub fn contact<IO: ContactIo>(&mut self, io: &mut IO) {
        if !self.initialized {
            return;
        }
        self.seconds = 0;
        self.remainder_ms = 0;
        self.save_counter(io);
        self.evaluate(io);
    }

    /// Set and persist the timeout (re-evaluated at once once initialized)
    pub fn set_timeout_days<IO: ContactIo>(&mut self, days: u16, io: &mut IO) -> Result<(), Error> {
        if !(TIMEOUT_MIN_DAYS..=TIMEOUT_MAX_DAYS).contains(&days) {
            return Err(Error::Invalid);
        }
        self.timeout_days = days;
        let result = nvm::save(io, self.region, NVM_TIMEOUT, MAGIC_TIMEOUT, &u32::from(days).to_le_bytes());
        if self.initialized {
            self.evaluate(io);
        }
        result.map_err(|_| Error::Io)
    }

    /// Current timeout, days
    pub const fn timeout_days(&self) -> u16 {
        self.timeout_days
    }

    /// Accumulated seconds without contact
    pub const fn seconds_since_contact(&self) -> u32 {
        self.seconds
    }

    /// Whether the inhibit is in force
    pub const fn inhibited(&self) -> bool {
        self.inhibited
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvm::{FlashError, MemFlash, SLOT_SIZE};

    const DAY_MS: u32 = 86_400_000;

    struct Bench {
        flash: MemFlash<512>,
        inhibit: Option<bool>,
        reports: usize,
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

    impl ContactIo for Bench {
        fn set_inhibit(&mut self, active: bool) {
            self.inhibit = Some(active);
        }
        fn report(&mut self, report: Report) {
            self.reports += 1;
            self.last = Some(report);
        }
    }

    fn boot(io: &mut Bench) -> ContactMonitor {
        let mut m = ContactMonitor::new(8);
        m.init(io);
        m
    }

    fn bench() -> Bench {
        Bench { flash: MemFlash::new(), inhibit: None, reports: 0, last: None }
    }

    #[test]
    fn inhibits_exactly_at_the_timeout_and_contact_lifts_it() {
        let mut io = bench();
        let mut m = boot(&mut io);
        assert_eq!((m.seconds_since_contact(), m.timeout_days(), m.inhibited()), (0, 30, false));
        assert_eq!(io.inhibit, Some(false));
        for _ in 0..29 {
            m.tick(DAY_MS, &mut io);
        }
        m.tick(DAY_MS - 1000, &mut io);
        assert!(!m.inhibited());
        m.tick(1000, &mut io);
        assert!(m.inhibited());
        assert_eq!(io.inhibit, Some(true));
        assert_eq!(io.last, Some(Report::NoContact(30)));
        m.tick(DAY_MS, &mut io);
        assert_eq!(io.reports, 1, "reported once, on the transition");
        m.contact(&mut io);
        assert_eq!((m.inhibited(), m.seconds_since_contact(), io.inhibit), (false, 0, Some(false)));
    }

    #[test]
    fn nothing_before_init() {
        let mut io = bench();
        let mut m = ContactMonitor::new(8);
        m.tick(DAY_MS, &mut io);
        m.contact(&mut io);
        assert_eq!((m.seconds_since_contact(), io.inhibit), (0, None));
        // The timeout can be set (and persisted) before init
        assert_eq!(m.set_timeout_days(5, &mut io), Ok(()));
        assert_eq!(io.inhibit, None);
        assert_eq!(boot(&mut io).timeout_days(), 5);
    }

    #[test]
    fn boot_penalty_reset_loop_and_rate_limited_persistence() {
        let mut io = bench();
        let mut m = boot(&mut io);
        m.tick(1000, &mut io);
        let mut m2 = boot(&mut io);
        assert_eq!(m2.seconds_since_contact(), PERSIST_INTERVAL_S, "the 1 s was lost; penalty added");
        m2.tick(PERSIST_INTERVAL_S * 1000, &mut io);
        assert_eq!(boot(&mut io).seconds_since_contact(), 3 * PERSIST_INTERVAL_S);

        m = boot(&mut io);
        m.set_timeout_days(1, &mut io).unwrap();
        for _ in 0..=(86_400 / PERSIST_INTERVAL_S) {
            m = boot(&mut io);
        }
        assert!(m.inhibited());
    }

    #[test]
    fn corrupt_counter_counts_as_timed_out() {
        let mut io = bench();
        let mut m = boot(&mut io);
        m.tick(120_000, &mut io);
        io.flash.bytes[8] ^= 0xFF;
        io.flash.bytes[SLOT_SIZE + 8] ^= 0xFF;
        let mut m = boot(&mut io);
        assert_eq!(io.reports, 2, "unreadable, then the inhibit");
        assert_eq!(io.last, Some(Report::NoContact(30)));
        assert!(m.inhibited());
        m.contact(&mut io);
        assert!(!m.inhibited());
    }

    #[test]
    fn timeout_bounds_and_shortening_below_elapsed() {
        let mut io = bench();
        let mut m = boot(&mut io);
        assert_eq!(m.set_timeout_days(0, &mut io), Err(Error::Invalid));
        assert_eq!(m.set_timeout_days(TIMEOUT_MAX_DAYS + 1, &mut io), Err(Error::Invalid));
        for _ in 0..5 {
            m.tick(DAY_MS, &mut io);
        }
        assert_eq!(m.set_timeout_days(4, &mut io), Ok(()));
        assert!(m.inhibited());
        io.flash.writes_left = Some(0);
        assert_eq!(m.set_timeout_days(6, &mut io), Err(Error::Io));
        assert_eq!(m.timeout_days(), 6, "applied until reset");
        assert!(!m.inhibited());
        // An out-of-range stored timeout reads as the default
        io.flash.writes_left = None;
        nvm::save(&mut io, 8, NVM_TIMEOUT, MAGIC_TIMEOUT, &91u32.to_le_bytes()).unwrap();
        assert_eq!(boot(&mut io).timeout_days(), TIMEOUT_DEFAULT_DAYS);
    }

    #[test]
    fn counter_saturates() {
        let mut io = bench();
        nvm::save(&mut io, 8, NVM_COUNTER, MAGIC_COUNTER, &(u32::MAX - 10).to_le_bytes()).unwrap();
        let mut m = boot(&mut io);
        assert_eq!(m.seconds_since_contact(), u32::MAX);
        m.tick(u32::MAX, &mut io);
        assert_eq!(m.seconds_since_contact(), u32::MAX);
        assert_eq!(carry_seconds(999, u32::MAX), (4_294_968, 294));
    }
}
