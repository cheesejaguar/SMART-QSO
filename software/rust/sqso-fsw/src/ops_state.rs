//! Operations state kept across resets (`ops_state.h`): the commissioning
//! hold and the ground's transponder enable.
//!
//! **Commissioning hold.** GSE ARM_FLIGHT sets it; only the authenticated
//! COMMISSIONING_RELEASE telecommand clears it. While it holds, the mode
//! manager refuses every way into ACTIVE (`Guard::ActiveAllowed`), so the
//! payload is never powered, and the transponder stays off: the spacecraft
//! flies beacon and telecommand only until the ground has heard it, proven
//! TX-OFF and released it (LV-01).
//!
//! **Transponder enable.** XPDR_ENABLE / XPDR_DISABLE: the ground can stop
//! the repeater on its own, durably, without TX-OFF (which silences the
//! beacon and telecommand responses too). It adds a gate; it never
//! overrides one (`transponder.rs`).
//!
//! The record is a two-slot NVM record (`nvm.rs`). Rules at boot:
//!  - blank (never written: a bench unit before ARM_FLIGHT): no hold,
//!    transponder allowed;
//!  - corrupt (neither slot passes its CRC): **hold**, transponder allowed
//!    once released. A lost record must not release a held spacecraft;
//!    the ground releases it again.
//!
//! @requirement SRS-F082 The software shall hold the mode at IDLE, the payload and the transponder off from ARM_FLIGHT until an authenticated commissioning release
//! @requirement SRS-F083 The software shall let the ground stop and allow the transponder by telecommand, persisted across resets

use crate::nvm::{self, Flash, RecordStatus};

/// Record ("OPST"): flags (1) | version (1) | spare (2)
pub const RECORD_MAGIC: u32 = 0x4F50_5354;
/// Record layout version
pub const VERSION: u8 = 1;
const FLAG_HOLD: u8 = 0x01;
const FLAG_XPDR_DISABLED: u8 = 0x02;

/// What the boot found
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loaded {
    /// A valid record
    Valid,
    /// No record: defaults (no hold)
    Blank,
    /// Unreadable: held until released
    Corrupt,
}

/// The record could not be persisted
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IoError;

/// The operations state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpsState {
    region: u32,
    hold: bool,
    xpdr_disabled: bool,
    initialized: bool,
}

impl OpsState {
    /// Before init: no hold, as for a blank record. The executive reads the
    /// record before it starts the mode manager, so in flight nothing asks
    /// for ACTIVE before [`OpsState::init`]; module tests that never load a
    /// record see a bench unit
    pub const fn new(region: u32) -> OpsState {
        OpsState { region, hold: false, xpdr_disabled: false, initialized: false }
    }

    /// Whether the record has been read
    pub const fn initialized(&self) -> bool {
        self.initialized
    }

    fn record(&self) -> [u8; 4] {
        let mut flags = 0;
        if self.hold {
            flags |= FLAG_HOLD;
        }
        if self.xpdr_disabled {
            flags |= FLAG_XPDR_DISABLED;
        }
        [flags, VERSION, 0, 0]
    }

    fn save<F: Flash>(&self, flash: &mut F) -> Result<(), IoError> {
        nvm::save(flash, self.region, 0, RECORD_MAGIC, &self.record()).map_err(|_| IoError)
    }

    /// Load the record (§ boot rules in the module documentation)
    pub fn init<F: Flash>(&mut self, flash: &mut F) -> Loaded {
        let mut rec = [0u8; 4];
        let loaded = match nvm::load(flash, self.region, 0, RECORD_MAGIC, &mut rec) {
            Ok(RecordStatus::Valid) => {
                let [flags, ..] = rec;
                self.hold = flags & FLAG_HOLD != 0;
                self.xpdr_disabled = flags & FLAG_XPDR_DISABLED != 0;
                Loaded::Valid
            }
            Ok(RecordStatus::Blank) => {
                self.hold = false;
                self.xpdr_disabled = false;
                Loaded::Blank
            }
            Ok(RecordStatus::Corrupt) | Err(_) => {
                self.hold = true;
                self.xpdr_disabled = false;
                Loaded::Corrupt
            }
        };
        self.initialized = true;
        loaded
    }

    /// Set or clear the commissioning hold, persisted. The state changes even
    /// if the record cannot be written (a hold set must hold now)
    pub fn set_hold<F: Flash>(&mut self, hold: bool, flash: &mut F) -> Result<(), IoError> {
        self.hold = hold;
        self.save(flash)
    }

    /// Allow or stop the transponder, persisted
    pub fn set_xpdr_enabled<F: Flash>(&mut self, enabled: bool, flash: &mut F) -> Result<(), IoError> {
        self.xpdr_disabled = !enabled;
        self.save(flash)
    }

    /// Whether the commissioning hold is in force
    pub const fn hold(&self) -> bool {
        self.hold
    }

    /// Whether the ground allows the transponder (false while held)
    pub const fn xpdr_allowed(&self) -> bool {
        !self.hold() && !self.xpdr_disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvm::{FlashError, MemFlash, SLOT_SIZE};
    extern crate std;

    const REGION: u32 = 0;

    struct Failing;
    impl Flash for Failing {
        fn read(&mut self, _: u32, _: u32, _: &mut [u8]) -> Result<(), FlashError> {
            Err(FlashError(-7))
        }
        fn write(&mut self, _: u32, _: u32, _: &[u8]) -> Result<(), FlashError> {
            Err(FlashError(-7))
        }
        fn erase(&mut self, _: u32) -> Result<(), FlashError> {
            Err(FlashError(-7))
        }
    }

    #[test]
    fn free_before_init_and_on_a_blank_record() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        assert!(!s.hold() && s.xpdr_allowed() && !s.initialized());
        assert_eq!(s.init(&mut f), Loaded::Blank);
        assert!(!s.hold() && s.xpdr_allowed() && s.initialized());
    }

    #[test]
    fn hold_persists_until_released() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        let _ = s.init(&mut f);
        assert_eq!(s.set_hold(true, &mut f), Ok(()));
        let mut after_reset = OpsState::new(REGION);
        assert_eq!(after_reset.init(&mut f), Loaded::Valid);
        assert!(after_reset.hold() && !after_reset.xpdr_allowed());
        assert_eq!(after_reset.set_hold(false, &mut f), Ok(()));
        let mut released = OpsState::new(REGION);
        let _ = released.init(&mut f);
        assert!(!released.hold() && released.xpdr_allowed());
    }

    #[test]
    fn transponder_disable_persists_and_never_lifts_the_hold() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        let _ = s.init(&mut f);
        assert_eq!(s.set_xpdr_enabled(false, &mut f), Ok(()));
        let mut r = OpsState::new(REGION);
        let _ = r.init(&mut f);
        assert!(!r.xpdr_allowed() && !r.hold());
        assert_eq!(r.set_hold(true, &mut f), Ok(()));
        assert_eq!(r.set_xpdr_enabled(true, &mut f), Ok(()));
        assert!(!r.xpdr_allowed(), "the hold still holds");
    }

    #[test]
    fn a_corrupt_record_holds() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        let _ = s.init(&mut f);
        assert_eq!(s.set_hold(false, &mut f), Ok(()));
        // Corrupt both slots behind their magic
        for slot in 0..2u32 {
            let mut b = [0u8; 1];
            f.read(REGION, slot * SLOT_SIZE as u32 + 12, &mut b).unwrap();
            f.write(REGION, slot * SLOT_SIZE as u32 + 12, &[b[0] ^ 0xFF]).unwrap();
        }
        let mut r = OpsState::new(REGION);
        assert_eq!(r.init(&mut f), Loaded::Corrupt);
        assert!(r.hold());
        let mut e = OpsState::new(REGION);
        assert_eq!(e.init(&mut Failing), Loaded::Corrupt);
        assert!(e.hold());
        assert_eq!(e.set_hold(false, &mut Failing), Err(IoError));
        assert!(!e.hold(), "the state follows the command even if it cannot be saved");
    }
}
