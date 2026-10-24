//! Transmitter permission and the latched TX-OFF (`rf_control.h`).
//!
//! The single authority for whether any transmitter may radiate. TX-OFF
//! drives the transmitter enable line low and latches the inhibit in
//! non-volatile memory, so it survives resets and only TX-ON clears it.
//! Automatic inhibits (loss of contact, post-deployment silence) are
//! separate bits; transmission needs the latch and every bit clear. Only
//! the RF silence also removes radio power, so the telecommand receiver
//! stays on under TX-OFF and loss of contact.
//!
//! Fail-safe rules: not permitted before [`RfControl::init`]; a blank record
//! (first boot) permits; a corrupt record or unreadable flash inhibits until
//! commanded on; TX-OFF drives the line low before persisting.
//!
//! @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))
//! @requirement SYS-224 No RF signal earlier than 45 minutes after deployment

use crate::nvm::{self, Flash, RecordStatus};

/// Persisted latch record ("RFTX")
pub const RECORD_MAGIC: u32 = 0x5246_5458;
/// Persisted power-level record ("RFPL"), apart from the latch
pub const POWER_MAGIC: u32 = 0x5246_504C;
/// Offset of the power-level record in the region
pub const POWER_BASE: u32 = nvm::FOOTPRINT;
/// Latch word: permitted (any other value is inhibited)
pub const STATE_PERMITTED: u32 = 0x5A5A_5A5A;
/// Latch word: inhibited
pub const STATE_INHIBITED: u32 = 0xA5A5_A5A5;

/// `RF_INHIBIT_CONTACT`: loss of contact, transmitters off
pub const INHIBIT_CONTACT: u32 = 0x01;
/// `RF_INHIBIT_RF_SILENCE`: post-deployment silence, whole radio off
pub const INHIBIT_RF_SILENCE: u32 = 0x02;

/// Downlink power levels (`RfPowerLevel_t`)
pub const POWER_0W5: u32 = 0;
/// 1 W
pub const POWER_1W: u32 = 1;
/// `RF_POWER_LEVEL_COUNT`
pub const POWER_LEVEL_COUNT: u32 = 2;

/// The two lines this module drives, and its fault reports
pub trait RfIo: Flash {
    /// Drive radio power and the transmitter enable (transmitter first when
    /// turning off, radio power first when turning on)
    fn drive(&mut self, radio_on: bool, tx_on: bool);
    /// Log an event
    fn report(&mut self, report: Report);
}

/// Events for the fault log
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// Boot: the record could not be read; inhibited
    Unreadable,
    /// Boot: the record is corrupt; inhibited
    Corrupt,
    /// TX-OFF applied and persisted
    TxOffLatched,
    /// TX-OFF applied, not persisted
    TxOffNotPersisted,
    /// TX-ON applied and persisted
    TxEnabled,
    /// TX-ON applied, not persisted (until the next reset)
    TxEnabledNotPersisted,
}

/// Why an operation failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Unknown power level
    Invalid,
    /// Not persisted (applied until reset)
    Io,
}

/// Module state
#[derive(Clone, Copy, Debug)]
pub struct RfControl {
    region: u32,
    initialized: bool,
    inhibited: bool,
    /// Automatic inhibit bits; not reset by init, so they can be set before
    /// the lines are first driven
    auto_inhibits: u32,
    power_level: u32,
}

impl RfControl {
    /// Before init: not permitted, no automatic inhibit, 0.5 W
    pub const fn new(region: u32) -> Self {
        RfControl { region, initialized: false, inhibited: true, auto_inhibits: 0, power_level: POWER_0W5 }
    }

    /// (radio power, transmitter enable) for the current inhibits
    pub const fn lines(&self) -> (bool, bool) {
        let radio_on = self.auto_inhibits & INHIBIT_RF_SILENCE == 0;
        (radio_on, radio_on && !self.inhibited && self.auto_inhibits == 0)
    }

    fn drive<IO: RfIo>(&self, io: &mut IO) {
        let (radio_on, tx_on) = self.lines();
        io.drive(radio_on, tx_on);
    }

    fn persist<IO: RfIo>(&self, io: &mut IO, word: u32) -> Result<(), Error> {
        nvm::save(io, self.region, 0, RECORD_MAGIC, &word.to_le_bytes()).map_err(|_| Error::Io)
    }

    /// Load the latch and the power level, then drive the lines
    pub fn init<IO: RfIo>(&mut self, io: &mut IO) {
        let mut word = [0u8; 4];
        self.inhibited = match nvm::load(io, self.region, 0, RECORD_MAGIC, &mut word) {
            Err(_) => {
                io.report(Report::Unreadable);
                true
            }
            // First boot: nothing has ever commanded TX-OFF
            Ok(RecordStatus::Blank) => false,
            Ok(RecordStatus::Corrupt) => {
                io.report(Report::Corrupt);
                true
            }
            Ok(RecordStatus::Valid) => u32::from_le_bytes(word) != STATE_PERMITTED,
        };
        self.drive(io);

        // Power level: 0.5 W unless a valid record says otherwise
        let mut word = [0u8; 4];
        self.power_level = match nvm::load(io, self.region, POWER_BASE, POWER_MAGIC, &mut word) {
            Ok(RecordStatus::Valid) if u32::from_le_bytes(word) < POWER_LEVEL_COUNT => u32::from_le_bytes(word),
            _ => POWER_0W5,
        };
        self.initialized = true;
    }

    /// Cease all transmissions (the line first, so a flash failure cannot
    /// leave it on) and latch the inhibit
    pub fn tx_off<IO: RfIo>(&mut self, io: &mut IO) -> Result<(), Error> {
        self.inhibited = true;
        self.drive(io);
        let result = self.persist(io, STATE_INHIBITED);
        io.report(if result.is_ok() { Report::TxOffLatched } else { Report::TxOffNotPersisted });
        result
    }

    /// Clear the latch and permit transmission
    pub fn tx_on<IO: RfIo>(&mut self, io: &mut IO) -> Result<(), Error> {
        let result = self.persist(io, STATE_PERMITTED);
        self.inhibited = false;
        self.drive(io);
        io.report(if result.is_ok() { Report::TxEnabled } else { Report::TxEnabledNotPersisted });
        result
    }

    /// Whether any transmitter may radiate now
    pub const fn permitted(&self) -> bool {
        self.initialized && !self.inhibited && self.auto_inhibits == 0
    }

    /// Apply or lift automatic inhibit bits; the lines are driven once
    /// initialized
    pub fn set_inhibit<IO: RfIo>(&mut self, source: u32, active: bool, io: &mut IO) {
        if active {
            self.auto_inhibits |= source;
        } else {
            self.auto_inhibits &= !source;
        }
        if self.initialized {
            self.drive(io);
        }
    }

    /// Select and persist the downlink power level (never touches the latch)
    pub fn set_power_level<F: Flash>(&mut self, level: u32, flash: &mut F) -> Result<(), Error> {
        if level >= POWER_LEVEL_COUNT {
            return Err(Error::Invalid);
        }
        self.power_level = level;
        nvm::save(flash, self.region, POWER_BASE, POWER_MAGIC, &level.to_le_bytes()).map_err(|_| Error::Io)
    }

    /// Current downlink power level
    pub const fn power_level(&self) -> u32 {
        self.power_level
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::nvm::{FlashError, MemFlash, SLOT_SIZE};

    /// Flash plus the lines and reports, for this module's tests and the
    /// modules that set its inhibits
    pub(crate) struct Bench {
        pub(crate) flash: MemFlash<512>,
        pub(crate) lines: Option<(bool, bool)>,
        pub(crate) reports: [Option<Report>; 8],
        pub(crate) count: usize,
    }

    impl Bench {
        pub(crate) fn new() -> Self {
            Bench { flash: MemFlash::new(), lines: None, reports: [None; 8], count: 0 }
        }
        pub(crate) fn last(&self) -> Option<Report> {
            self.count.checked_sub(1).and_then(|i| self.reports[i % 8])
        }
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

    impl RfIo for Bench {
        fn drive(&mut self, radio_on: bool, tx_on: bool) {
            self.lines = Some((radio_on, tx_on));
        }
        fn report(&mut self, report: Report) {
            self.reports[self.count % 8] = Some(report);
            self.count += 1;
        }
    }

    fn booted(io: &mut Bench) -> RfControl {
        let mut rf = RfControl::new(7);
        rf.init(io);
        rf
    }

    #[test]
    fn not_permitted_before_init_blank_permits() {
        let mut io = Bench::new();
        let mut rf = RfControl::new(7);
        assert!(!rf.permitted());
        rf.init(&mut io);
        assert!(rf.permitted());
        assert_eq!(io.lines, Some((true, true)));
        assert_eq!(rf.power_level(), POWER_0W5);
    }

    #[test]
    fn tx_off_latches_across_resets_and_tx_on_clears() {
        let mut io = Bench::new();
        let mut rf = booted(&mut io);
        assert_eq!(rf.tx_off(&mut io), Ok(()));
        assert_eq!(io.last(), Some(Report::TxOffLatched));
        assert!(!rf.permitted());
        assert_eq!(io.lines, Some((true, false)), "receiver stays powered");
        let mut rf = booted(&mut io);
        assert!(!rf.permitted());
        assert_eq!(rf.tx_on(&mut io), Ok(()));
        let rf = booted(&mut io);
        assert!(rf.permitted());
    }

    #[test]
    fn tx_off_with_flash_failure_still_silences() {
        let mut io = Bench::new();
        let mut rf = booted(&mut io);
        io.flash.writes_left = Some(0);
        assert_eq!(rf.tx_off(&mut io), Err(Error::Io));
        assert_eq!(io.last(), Some(Report::TxOffNotPersisted));
        assert_eq!(io.lines, Some((true, false)));
        assert!(!rf.permitted());
        assert_eq!(rf.tx_on(&mut io), Err(Error::Io));
        assert!(rf.permitted(), "permitted until the next reset");
        io.flash.writes_left = None;
        let rf = booted(&mut io);
        assert!(rf.permitted(), "the failed TX-OFF was never latched");
    }

    #[test]
    fn corrupt_or_unrecognized_record_inhibits() {
        let mut io = Bench::new();
        let mut rf = booted(&mut io);
        rf.tx_off(&mut io).unwrap();
        rf.tx_on(&mut io).unwrap();
        io.flash.bytes[8] ^= 0xFF;
        io.flash.bytes[SLOT_SIZE + 8] ^= 0xFF;
        let rf = booted(&mut io);
        assert!(!rf.permitted());
        assert_eq!(io.last(), Some(Report::Corrupt));

        // A valid record with any word but PERMITTED is inhibited
        let mut io = Bench::new();
        nvm::save(&mut io, 7, 0, RECORD_MAGIC, &0x1234u32.to_le_bytes()).unwrap();
        assert!(!booted(&mut io).permitted());
    }

    #[test]
    fn automatic_inhibits_are_independent() {
        let mut io = Bench::new();
        let mut rf = RfControl::new(7);
        // Before init: recorded, lines not driven
        rf.set_inhibit(INHIBIT_RF_SILENCE, true, &mut io);
        assert_eq!(io.lines, None);
        rf.init(&mut io);
        assert_eq!(io.lines, Some((false, false)), "silence removes radio power");
        rf.set_inhibit(INHIBIT_CONTACT, true, &mut io);
        rf.set_inhibit(INHIBIT_RF_SILENCE, false, &mut io);
        assert_eq!(io.lines, Some((true, false)), "contact keeps the receiver on");
        assert!(!rf.permitted());
        rf.tx_on(&mut io).unwrap();
        assert!(!rf.permitted(), "TX-ON does not lift an automatic inhibit");
        rf.set_inhibit(INHIBIT_CONTACT, false, &mut io);
        assert!(rf.permitted());
        assert_eq!(io.lines, Some((true, true)));
        rf.tx_off(&mut io).unwrap();
        rf.set_inhibit(INHIBIT_CONTACT | INHIBIT_RF_SILENCE, false, &mut io);
        assert!(!rf.permitted(), "lifting inhibits does not clear TX-OFF");
    }

    #[test]
    fn power_level_persists_validates_and_leaves_the_latch() {
        let mut io = Bench::new();
        let mut rf = booted(&mut io);
        assert_eq!(rf.set_power_level(POWER_1W, &mut io.flash), Ok(()));
        assert_eq!(rf.set_power_level(POWER_LEVEL_COUNT, &mut io.flash), Err(Error::Invalid));
        assert_eq!(rf.power_level(), POWER_1W);
        rf.tx_off(&mut io).unwrap();
        let mut rf = booted(&mut io);
        assert_eq!(rf.power_level(), POWER_1W);
        assert_eq!(rf.set_power_level(POWER_0W5, &mut io.flash), Ok(()));
        assert!(!rf.permitted());
        let rf = booted(&mut io);
        assert_eq!((rf.power_level(), rf.permitted()), (POWER_0W5, false));
        // An out-of-range stored level reads as 0.5 W
        nvm::save(&mut io, 7, POWER_BASE, POWER_MAGIC, &5u32.to_le_bytes()).unwrap();
        assert_eq!(booted(&mut io).power_level(), POWER_0W5);
    }

    /// Mutation check: a stored level equal to the level count is out of
    /// range and reads as 0.5 W
    #[test]
    fn stored_level_count_reads_as_half_watt() {
        let mut io = Bench::new();
        nvm::save(&mut io, 7, POWER_BASE, POWER_MAGIC, &POWER_LEVEL_COUNT.to_le_bytes()).unwrap();
        assert_eq!(booted(&mut io).power_level(), POWER_0W5);
        nvm::save(&mut io, 7, POWER_BASE, POWER_MAGIC, &POWER_1W.to_le_bytes()).unwrap();
        assert_eq!(booted(&mut io).power_level(), POWER_1W);
    }
}
