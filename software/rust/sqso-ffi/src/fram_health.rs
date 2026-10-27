//! `fram_health.h`: the FRAM's health and the degraded mode without it
//! (`sqso_fsw::fram_health`, RID SMA-B10).
//!
//! The executive's one-minute check of the critical state ([`check`])
//! runs the TX-OFF latch's and the operations record's checks, in the
//! degraded mode against their CRC-protected RAM copies, and counts the
//! outcome here. FRAM_BYPASS ([`fram_bypass`]) enters the degraded mode
//! once the FRAM has been declared failed.
//!
//! @requirement SRS-F160 FRAM failed after three consecutive failed one-minute checks; FRAM_BYPASS then makes the CRC-protected RAM copies of the critical state authoritative until the FRAM reads back again or the next reset

use sqso_fsw::fram_health::{FramHealth, Refusal};

use crate::c::{self, fault_type, rc, severity};
use crate::cell::Global;
use crate::{ops_state, passivation, rf_control};

static HEALTH: Global<FramHealth> = Global::new(FramHealth::new());

/// Boot: healthy, not degraded
#[unsafe(no_mangle)]
pub extern "C" fn fram_health_init() {
    let _ = HEALTH.with(|h| *h = FramHealth::new());
}

/// The one-minute check of the critical persisted state (AV-07, SRS-F133):
/// the TX-OFF latch, the operations record and the passivation record
#[unsafe(no_mangle)]
pub extern "C" fn fram_health_check() {
    let degraded = degraded();
    let rf_ok = rf_control::check(degraded);
    let ops_ok = ops_state::check(degraded);
    passivation::passivation_verify();
    let Some((before, after)) = HEALTH.with(|h| {
        let before = h.state();
        h.note(rf_ok & ops_ok); // both checks ran; either failing is a failed check
        (before, h.state())
    }) else {
        return;
    };
    match (before, after) {
        (b, sqso_fsw::fram_health::STATE_FAILED) if b != after => c::log(
            fault_type::INIT,
            severity::CRITICAL,
            c"FRAM failed: three checks unreadable; safe defaults kept, FRAM_BYPASS accepted",
            0.0,
        ),
        (sqso_fsw::fram_health::STATE_DEGRADED, sqso_fsw::fram_health::STATE_OK) => {
            c::log(fault_type::INIT, severity::WARNING, c"FRAM reads back again: degraded mode ended", 0.0);
        }
        _ => {}
    }
}

/// FRAM_BYPASS: enter the degraded mode (argument `FRM!`); `ERROR`
/// unless the FRAM has been declared failed, `INVALID` for another argument
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fram_bypass(arg: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: arg points to len bytes
    let Some(arg) = (unsafe { crate::ptr::bytes(arg, len) }) else { return rc::NULL_PTR };
    match HEALTH.with(|h| h.bypass(arg)) {
        Some(Ok(())) => {
            c::log(
                fault_type::COMMAND,
                severity::WARNING,
                c"FRAM_BYPASS: running without the FRAM (RAM copies under CRC)",
                0.0,
            );
            rc::OK
        }
        Some(Err(Refusal::WrongCode)) => rc::INVALID,
        Some(Err(Refusal::NotFailed)) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// HK1 `FR`: the state (0 working, 1 failed, 2 degraded mode) and the
/// failed checks since boot
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fram_health_status(state: *mut u8, failures: *mut u32) -> i32 {
    let Some((st, n)) = HEALTH.with(|h| (h.state(), h.failures)) else { return rc::BUSY };
    // SAFETY: the C contract: each pointer is NULL or a live object
    unsafe {
        if let Some(p) = crate::ptr::out(state) {
            *p = st;
        }
        if let Some(p) = crate::ptr::out(failures) {
            *p = n;
        }
    }
    rc::OK
}

/// Whether the degraded mode is in force
pub(crate) fn degraded() -> bool {
    HEALTH.with(|h| h.degraded).unwrap_or(false)
}

/// The state and the failed checks (housekeeping)
pub(crate) fn status() -> (u8, u32) {
    HEALTH.with(|h| (h.state(), h.failures)).unwrap_or((sqso_fsw::fram_health::STATE_OK, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A nested call (an interrupt) while the state is in use: the check
    /// does nothing, the bypass and the status read as busy; a NULL
    /// argument is refused; the housekeeping status is the module's
    #[test]
    fn busy_null_and_status() {
        let _serial = crate::fault_mgmt::tests::serial();
        fram_health_init();
        assert_eq!(status(), (sqso_fsw::fram_health::STATE_OK, 0));
        // SAFETY: NULL with a length: refused before it is read
        assert_eq!(unsafe { fram_bypass(core::ptr::null(), 4) }, rc::NULL_PTR);
        let nested = HEALTH.with(|_| {
            fram_health_check();
            // SAFETY: live buffer, then NULL outputs (allowed)
            let bypass = unsafe { fram_bypass(b"FRM!".as_ptr(), 4) };
            let st = unsafe { fram_health_status(core::ptr::null_mut(), core::ptr::null_mut()) };
            (bypass, st, status(), degraded())
        });
        assert_eq!(nested, Some((rc::BUSY, rc::BUSY, (sqso_fsw::fram_health::STATE_OK, 0), false)));
        assert_eq!(status().1, 0, "the nested check counted nothing");
    }
}
