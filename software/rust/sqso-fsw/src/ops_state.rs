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
//! **ADCS geometry.** ADCS_SET_GEOMETRY: which of the spin-axis and
//! inertia presets the attitude control uses (`adcs::Geometry::PRESETS`:
//! both wings, one wing, none; STR-27), persisted so a reset keeps it.
//!
//! The record is a two-slot NVM record (`nvm.rs`). Rules at boot:
//!  - blank (never written: a bench unit before ARM_FLIGHT): no hold,
//!    transponder allowed;
//!  - corrupt (neither slot passes its CRC): **hold**, transponder allowed
//!    once released. A lost record must not release a held spacecraft;
//!    the ground releases it again.
//!
//! [`OpsState::verify`] re-reads the record every minute and settles any
//! disagreement with the RAM copy toward the safe side (AV-07): held if
//! either holds, the transponder stopped if either stops it, the record's
//! geometry preset.
//!
//! @requirement SRS-F082 The software shall hold the mode at IDLE, the payload and the transponder off from ARM_FLIGHT until an authenticated commissioning release
//! @requirement SRS-F083 The software shall let the ground stop and allow the transponder by telecommand, persisted across resets

use crate::fram_health::Shadow;
use crate::nvm::{self, Flash, RecordStatus};

/// Record ("OPST"): flags (1) | version (1) | ADCS geometry preset (1) | spare (1)
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
    adcs_geometry: u8,
    initialized: bool,
    /// The record's CRC as last set (the degraded mode's check, SMA-B10)
    shadow: Option<Shadow>,
}

impl OpsState {
    /// Before init: no hold, as for a blank record. The executive reads the
    /// record before it starts the mode manager, so in flight nothing asks
    /// for ACTIVE before [`OpsState::init`]; module tests that never load a
    /// record see a bench unit
    pub const fn new(region: u32) -> OpsState {
        OpsState { region, hold: false, xpdr_disabled: false, adcs_geometry: 0, initialized: false, shadow: None }
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
        [flags, VERSION, self.adcs_geometry, 0]
    }

    fn save<F: Flash>(&mut self, flash: &mut F) -> Result<(), IoError> {
        self.shadow = Some(Shadow::of(&self.record()));
        nvm::save(flash, self.region, 0, RECORD_MAGIC, &self.record()).map_err(|_| IoError)
    }

    /// Load the record (§ boot rules in the module documentation)
    pub fn init<F: Flash>(&mut self, flash: &mut F) -> Loaded {
        let mut rec = [0u8; 4];
        let loaded = match nvm::load(flash, self.region, 0, RECORD_MAGIC, &mut rec) {
            Ok(RecordStatus::Valid) => {
                let [flags, _, geometry, _] = rec;
                self.hold = flags & FLAG_HOLD != 0;
                self.xpdr_disabled = flags & FLAG_XPDR_DISABLED != 0;
                self.adcs_geometry = geometry;
                Loaded::Valid
            }
            Ok(RecordStatus::Blank) => {
                self.hold = false;
                self.xpdr_disabled = false;
                self.adcs_geometry = 0;
                Loaded::Blank
            }
            Ok(RecordStatus::Corrupt) | Err(_) => {
                self.hold = true;
                self.xpdr_disabled = false;
                self.adcs_geometry = 0;
                Loaded::Corrupt
            }
        };
        self.initialized = true;
        self.shadow = Some(Shadow::of(&self.record()));
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

    /// Select the ADCS geometry preset, persisted (the caller checks the
    /// number against the presets)
    pub fn set_adcs_geometry<F: Flash>(&mut self, preset: u8, flash: &mut F) -> Result<(), IoError> {
        self.adcs_geometry = preset;
        self.save(flash)
    }

    /// Check the RAM copy against the record: merge toward held and
    /// stopped, persist the merge if it differs from the record; whether
    /// the two disagreed
    pub fn verify<F: Flash>(&mut self, flash: &mut F) -> bool {
        self.check(flash, false).0
    }

    /// [`OpsState::verify`], or in the degraded mode without the FRAM
    /// (`degraded`, RID SMA-B10): the RAM copy against its CRC (a mismatch,
    /// an upset, sets the hold and stops the transponder), then written and
    /// read back. (whether the RAM copy disagreed, whether the record was
    /// read: in the degraded mode, written and read back)
    pub fn check<F: Flash>(&mut self, flash: &mut F, degraded: bool) -> (bool, bool) {
        if !self.initialized {
            return (false, true);
        }
        if degraded {
            let upset = !self.shadow.is_some_and(|sh| sh.holds(&self.record()));
            if upset {
                self.hold = true;
                self.xpdr_disabled = true;
            }
            let written = self.save(flash).is_ok();
            let mut back = [0u8; 4];
            let read = nvm::load(flash, self.region, 0, RECORD_MAGIC, &mut back) == Ok(RecordStatus::Valid);
            return (upset, written && read && back == self.record());
        }
        let (disagreed, read) = self.settle(flash);
        (disagreed, read)
    }

    fn settle<F: Flash>(&mut self, flash: &mut F) -> (bool, bool) {
        let mut rec = [0u8; 4];
        let stored = match nvm::load(flash, self.region, 0, RECORD_MAGIC, &mut rec) {
            Ok(RecordStatus::Valid) => Some(rec),
            Ok(RecordStatus::Blank) => Some(OpsState::new(self.region).record()),
            Ok(RecordStatus::Corrupt) | Err(_) => None,
        };
        let Some([flags, _, geometry, _]) = stored else {
            self.hold = true;
            self.shadow = Some(Shadow::of(&self.record()));
            return (true, false);
        };
        let before = self.record();
        self.hold |= flags & FLAG_HOLD != 0;
        self.xpdr_disabled |= flags & FLAG_XPDR_DISABLED != 0;
        self.adcs_geometry = geometry;
        let disagreed = before != self.record() || stored != Some(self.record());
        if stored != Some(self.record()) {
            let _ = self.save(flash);
        }
        (disagreed, true)
    }

    /// The ADCS geometry preset
    pub const fn adcs_geometry(&self) -> u8 {
        self.adcs_geometry
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

    /// Writes report success and store nothing
    struct Ignoring(MemFlash<256>);
    impl Flash for Ignoring {
        fn read(&mut self, r: u32, o: u32, b: &mut [u8]) -> Result<(), FlashError> {
            self.0.read(r, o, b)
        }
        fn write(&mut self, _: u32, _: u32, _: &[u8]) -> Result<(), FlashError> {
            Ok(())
        }
        fn erase(&mut self, _: u32) -> Result<(), FlashError> {
            Ok(())
        }
    }

    /// After each write one read succeeds (the write's own read-back) and
    /// the rest fail: a device that drops off the bus mid-check
    struct DropsOff {
        f: MemFlash<256>,
        reads_left: Option<u32>,
    }
    impl Flash for DropsOff {
        fn read(&mut self, r: u32, o: u32, b: &mut [u8]) -> Result<(), FlashError> {
            if self.reads_left == Some(0) {
                return Err(FlashError(-7));
            }
            self.reads_left = self.reads_left.map(|n| n - 1);
            self.f.read(r, o, b)
        }
        fn write(&mut self, r: u32, o: u32, d: &[u8]) -> Result<(), FlashError> {
            self.reads_left = Some(1);
            self.f.write(r, o, d)
        }
        fn erase(&mut self, r: u32) -> Result<(), FlashError> {
            self.f.erase(r)
        }
    }

    /// The degraded mode without the FRAM (RID SMA-B10): the RAM record
    /// holds while the FRAM is dead; an upset (its CRC no longer matches)
    /// sets the hold and stops the transponder; the check reports whether
    /// the record was written and read back
    #[test]
    fn degraded_check_runs_from_the_ram_copy() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        assert_eq!(s.check(&mut f, true), (false, true), "not initialized: nothing to check");
        let _ = s.init(&mut f);
        assert_eq!(s.check(&mut f, true), (false, true));
        // Dead: the release holds in RAM, not outside the degraded mode
        assert_eq!(s.set_hold(false, &mut Failing), Err(IoError));
        assert_eq!(s.check(&mut Failing, true), (false, false));
        assert!(!s.hold());
        assert_eq!(s.check(&mut Failing, false), (true, false));
        assert!(s.hold());
        // An upset of the RAM copy
        assert_eq!(s.set_hold(false, &mut f), Ok(()));
        s.adcs_geometry = 2; // changed in RAM without the CRC
        let (upset, read) = s.check(&mut f, true);
        assert!(upset);
        assert!(read);
        assert!(s.hold());
        assert!(!s.xpdr_allowed());
        // Written and verified, then the device drops off before the check reads it
        let mut drops = DropsOff { f: f.clone(), reads_left: None };
        assert_eq!(s.check(&mut drops, true), (false, false));
        // Written, not read back as written
        let mut ig = Ignoring(f.clone());
        let _ = s.set_xpdr_enabled(true, &mut ig);
        s.hold = false;
        s.shadow = Some(Shadow::of(&s.record()));
        assert_eq!(s.check(&mut ig, true), (false, false));
    }

    #[test]
    fn free_before_init_and_on_a_blank_record() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        assert!(!s.hold());
        assert!(s.xpdr_allowed());
        assert!(!s.initialized());
        assert_eq!(s.init(&mut f), Loaded::Blank);
        assert!(!s.hold());
        assert!(s.xpdr_allowed());
        assert!(s.initialized());
    }

    #[test]
    fn hold_persists_until_released() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        let _ = s.init(&mut f);
        assert_eq!(s.set_hold(true, &mut f), Ok(()));
        let mut after_reset = OpsState::new(REGION);
        assert_eq!(after_reset.init(&mut f), Loaded::Valid);
        assert!(after_reset.hold());
        assert!(!after_reset.xpdr_allowed());
        assert_eq!(after_reset.set_hold(false, &mut f), Ok(()));
        let mut released = OpsState::new(REGION);
        let _ = released.init(&mut f);
        assert!(!released.hold());
        assert!(released.xpdr_allowed());
    }

    #[test]
    fn transponder_disable_persists_and_never_lifts_the_hold() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        let _ = s.init(&mut f);
        assert_eq!(s.set_xpdr_enabled(false, &mut f), Ok(()));
        let mut r = OpsState::new(REGION);
        let _ = r.init(&mut f);
        assert!(!r.xpdr_allowed());
        assert!(!r.hold());
        assert_eq!(r.set_hold(true, &mut f), Ok(()));
        assert_eq!(r.set_xpdr_enabled(true, &mut f), Ok(()));
        assert!(!r.xpdr_allowed(), "the hold still holds");
        assert_eq!(r.set_adcs_geometry(2, &mut f), Ok(()));
        let mut g = OpsState::new(REGION);
        let _ = g.init(&mut f);
        assert_eq!(g.adcs_geometry(), 2);
        assert!(g.hold());
        assert!(!g.xpdr_allowed());
    }

    /// AV-07: an upset RAM copy is settled toward held and stopped
    #[test]
    fn verify_merges_toward_the_safe_side() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        assert!(!s.verify(&mut f), "before init: nothing");
        let _ = s.init(&mut f);
        assert!(!s.verify(&mut f), "blank and defaults agree");
        assert_eq!(s.set_hold(true, &mut f), Ok(()));
        assert!(!s.verify(&mut f));
        s.hold = false; // a RAM upset
        assert!(s.verify(&mut f));
        assert!(s.hold());
        s.xpdr_disabled = true; // RAM says stopped, the record not
        s.adcs_geometry = 3; // and a geometry the record does not hold
        assert!(s.verify(&mut f));
        assert!(!s.xpdr_allowed());
        assert_eq!(s.adcs_geometry(), 0, "the record's preset");
        let mut r = OpsState::new(REGION);
        let _ = r.init(&mut f);
        assert!(r.hold() && r.xpdr_disabled, "the merge was persisted");
        // Blank record, RAM held: persisted
        let mut g = MemFlash::<256>::new();
        let mut b = OpsState::new(REGION);
        let _ = b.init(&mut g);
        b.hold = true;
        assert!(b.verify(&mut g));
        let mut c = OpsState::new(REGION);
        assert_eq!(c.init(&mut g), Loaded::Valid);
        assert!(c.hold());
        // Unreadable: held
        let mut e = OpsState::new(REGION);
        let _ = e.init(&mut f);
        assert_eq!(e.set_hold(false, &mut f), Ok(()));
        assert!(e.verify(&mut Failing));
        assert!(e.hold());
    }

    #[test]
    fn a_corrupt_record_holds() {
        let mut f = MemFlash::<256>::new();
        let mut s = OpsState::new(REGION);
        let _ = s.init(&mut f);
        assert_eq!(s.set_hold(false, &mut f), Ok(()));
        // Corrupt both slots behind their magic
        let slot_size = u32::try_from(SLOT_SIZE).unwrap();
        for slot in 0..2u32 {
            let mut b = [0u8; 1];
            f.read(REGION, slot * slot_size + 8, &mut b).unwrap();
            f.write(REGION, slot * slot_size + 8, &[b[0] ^ 0xFF]).unwrap();
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
