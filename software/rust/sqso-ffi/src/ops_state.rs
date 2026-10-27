//! `ops_state.h`: the commissioning hold and the ground's transponder
//! enable (`sqso_fsw::ops_state`), on the flash HAL (`FLASH_REGION_OPS`).
//!
//! Coverage: only a nested call finds `OPS` in use (busy: `Global::with`
//! gives `None`); the hold then reads as held, the safe answer.
//!
//! @requirement SRS-F082 The software shall hold the mode at IDLE, the payload and the transponder off from ARM_FLIGHT until an authenticated commissioning release
//! @requirement SRS-F083 The software shall let the ground stop and allow the transponder by telecommand, persisted across resets

use sqso_fsw::ops_state::{Loaded, OpsState};

use crate::c::{self, HalFlash, fault_type, rc, severity};
use crate::cell::Global;

/// `FLASH_REGION_OPS`
pub(crate) const FLASH_REGION_OPS: u32 = 11;

static OPS: Global<OpsState> = Global::new(OpsState::new(FLASH_REGION_OPS));

/// Load the record (blank: no hold; corrupt: held until released)
#[unsafe(no_mangle)]
pub extern "C" fn ops_state_init() -> i32 {
    match OPS.with(|o| o.init(&mut HalFlash)) {
        Some(Loaded::Valid | Loaded::Blank) => rc::OK,
        Some(Loaded::Corrupt) => {
            c::log(fault_type::INIT, severity::ERROR, c"Operations record unreadable: commissioning hold set", 0.0);
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Whether the commissioning hold is in force (held if it cannot be read)
#[unsafe(no_mangle)]
pub extern "C" fn ops_commissioning_hold() -> bool {
    OPS.with(|o| o.hold()).unwrap_or(true)
}

/// Set (ARM_FLIGHT) or clear (COMMISSIONING_RELEASE) the hold, persisted;
/// the hold changes even when the record cannot be written (`IO`)
#[unsafe(no_mangle)]
pub extern "C" fn ops_set_commissioning_hold(hold: bool) -> i32 {
    match OPS.with(|o| o.set_hold(hold, &mut HalFlash)) {
        Some(Ok(())) => {
            c::log(
                fault_type::COMMAND,
                severity::INFO,
                if hold { c"Commissioning hold set" } else { c"Commissioning hold released" },
                0.0,
            );
            rc::OK
        }
        // In the degraded mode without the FRAM the RAM copy is the record
        // (RID SMA-B10): applied is done
        Some(Err(_)) if crate::fram_health::degraded() => rc::OK,
        Some(Err(_)) => rc::IO,
        None => rc::BUSY,
    }
}

/// Check the RAM copy against the record (AV-07); a disagreement is logged
#[unsafe(no_mangle)]
pub extern "C" fn ops_state_verify() {
    let _ = check(false);
}

/// The one-minute check, in the degraded mode without the FRAM against the
/// record's CRC (RID SMA-B10); a disagreement is logged; whether the record
/// was read (busy: not)
pub(crate) fn check(degraded: bool) -> bool {
    let Some((disagreed, read)) = OPS.with(|o| o.check(&mut HalFlash, degraded)) else { return false };
    if disagreed {
        c::log(fault_type::INIT, severity::ERROR, c"Operations state check: RAM copy disagreed, merged", 0.0);
    }
    read
}

/// The ADCS geometry preset (ADCS_SET_GEOMETRY; 0 if it cannot be read)
pub fn ops_adcs_geometry() -> u8 {
    OPS.with(|o| o.adcs_geometry()).unwrap_or(0)
}

/// Select and persist the ADCS geometry preset and apply it; `INVALID` for
/// a preset that does not exist
pub fn ops_set_adcs_geometry(preset: u8) -> i32 {
    if sqso_fsw::adcs::Geometry::preset(preset).is_none() {
        return rc::INVALID;
    }
    let _ = crate::adcs_control::set_geometry(preset);
    match OPS.with(|o| o.set_adcs_geometry(preset, &mut HalFlash)) {
        Some(Ok(())) => rc::OK,
        Some(Err(_)) => rc::IO,
        None => rc::BUSY,
    }
}

/// Whether the ground allows the transponder (false while held)
#[unsafe(no_mangle)]
pub extern "C" fn ops_xpdr_allowed() -> bool {
    OPS.with(|o| o.xpdr_allowed()).unwrap_or(false)
}

/// Allow (XPDR_ENABLE) or stop (XPDR_DISABLE) the transponder, persisted
#[unsafe(no_mangle)]
pub extern "C" fn ops_set_xpdr_enabled(enabled: bool) -> i32 {
    match OPS.with(|o| o.set_xpdr_enabled(enabled, &mut HalFlash)) {
        Some(Ok(())) => rc::OK,
        Some(Err(_)) => rc::IO,
        None => rc::BUSY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_reads_as_held() {
        // The other tests use the HAL and these globals: one at a time
        let _serial = crate::fault_mgmt::tests::serial();
        let held = OPS.with(|_| (ops_commissioning_hold(), ops_xpdr_allowed()));
        assert_eq!(held, Some((true, false)));
        assert_eq!(OPS.with(|_| ops_set_commissioning_hold(false)), Some(rc::BUSY));
        assert_eq!(OPS.with(|_| ops_set_xpdr_enabled(true)), Some(rc::BUSY));
        assert_eq!(OPS.with(|_| ops_state_init()), Some(rc::BUSY));
        assert_eq!(OPS.with(|_| ops_adcs_geometry()), Some(0));
        assert_eq!(OPS.with(|_| ops_set_adcs_geometry(1)), Some(rc::BUSY));
        assert_eq!(OPS.with(|_| ops_state_verify()), Some(()));
        assert_eq!(ops_set_adcs_geometry(9), rc::INVALID);
    }
}
