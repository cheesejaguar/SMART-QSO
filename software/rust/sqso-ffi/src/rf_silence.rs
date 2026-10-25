//! `rf_silence.h`: the 45-minute post-deployment RF silence
//! (`sqso_fsw::rf_silence`), on the flash HAL, the reset cause and the
//! separation switch.
//!
//! Coverage: only a nested call (an interrupt) finds `SILENCE` in use (busy:
//! `Global::with` gives `None`); no C callback runs while it is held. The unit
//! test `tests::busy_reads_as_silent` asserts what such a call gets, but the
//! safety coverage gate cannot credit it: llvm-cov keeps one record per
//! `#[no_mangle]` function name (the C library's, whose hash differs from the
//! unit-test build's), so those lines are marked COVERAGE-EXEMPT.
//!
//! @requirement SYS-224 No RF signal earlier than 45 minutes after deployment; switch toggle resets the timer

use sqso_fsw::rf_control::INHIBIT_RF_SILENCE;
use sqso_fsw::rf_silence::{Report, Reset, RfSilence, SilenceIo, State};

use crate::c::{self, HalFlash, fault_type, rc, severity};
use crate::cell::Global;
use crate::{rf_control, separation_switch};

/// `FLASH_REGION_RF_SILENCE`
pub(crate) const FLASH_REGION_RF_SILENCE: u32 = 9;

unsafe extern "C" {
    fn hal_reset_cause() -> u32;
}

/// Why the processor last reset
pub(crate) fn reset() -> Reset {
    // SAFETY: no arguments
    Reset::from_hal(unsafe { hal_reset_cause() })
}

static SILENCE: Global<RfSilence> = Global::new(RfSilence::new(FLASH_REGION_RF_SILENCE));

/// The HAL gives the switch, rf_control takes the inhibit, the fault log
/// takes the reports
impl SilenceIo for HalFlash {
    fn switch_init(&mut self) {
        separation_switch::separation_switch_init();
    }

    fn stowed(&mut self) -> bool {
        separation_switch::separation_switch_stowed()
    }

    fn set_inhibit(&mut self, active: bool) {
        rf_control::rf_control_set_inhibit(INHIBIT_RF_SILENCE, active);
    }

    fn report(&mut self, report: Report) {
        match report {
            Report::Unreadable => {
                c::log(fault_type::INIT, severity::ERROR, c"RF silence record unreadable: counting from 0", 0.0)
            }
            Report::Complete => {
                c::log(fault_type::INIT, severity::INFO, c"RF silence complete: radio enabled", 0.0);
            }
        }
    }
}

/// Load the state, apply the reset-cause rules, persist, set the inhibit
#[unsafe(no_mangle)]
pub extern "C" fn rf_silence_init() -> i32 {
    let cause = reset();
    // COVERAGE-EXEMPT: busy, only a nested call (an interrupt) gets here; see "Coverage" in the module doc
    let Some((state, elapsed)) = SILENCE.with(|s| {
        s.init(cause, &mut HalFlash);
        (s.state(), s.elapsed_seconds())
    }) else {
        // COVERAGE-EXEMPT: busy, only a nested call (an interrupt) gets here; see "Coverage" in the module doc
        return rc::BUSY;
    };
    // Observable at every boot (pre-ship check)
    c::console(format_args!(
        "[RF_SILENCE] state={} elapsed={elapsed}s",
        if state == State::Complete { "COMPLETE" } else { "COUNTING" }
    ));
    rc::OK
}

/// Advance the count by elapsed time; restart it while stowed
#[unsafe(no_mangle)]
pub extern "C" fn rf_silence_tick(elapsed_ms: u32) {
    let _ = SILENCE.with(|s| s.tick(elapsed_ms, &mut HalFlash));
}

/// Re-arm for flight (GSE only): silent now, count from the next boot
#[unsafe(no_mangle)]
pub extern "C" fn rf_silence_arm() -> i32 {
    match SILENCE.with(|s| s.arm(&mut HalFlash)) {
        Some(Ok(())) => rc::OK,
        Some(Err(_)) => rc::IO,
        // COVERAGE-EXEMPT: busy, only a nested call (an interrupt) gets here; see "Coverage" in the module doc
        None => rc::BUSY,
    }
}

/// Whether the silence is in force
#[unsafe(no_mangle)]
pub extern "C" fn rf_silence_active() -> bool {
    SILENCE.with(|s| s.active()).unwrap_or(true)
}

/// Current state (`RfSilenceState_t`)
#[unsafe(no_mangle)]
pub extern "C" fn rf_silence_get_state() -> u32 {
    SILENCE.with(|s| s.state()).unwrap_or(State::Counting) as u32
}

/// Seconds counted toward the silence
#[unsafe(no_mangle)]
pub extern "C" fn rf_silence_elapsed_seconds() -> u32 {
    SILENCE.with(|s| s.elapsed_seconds()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// A nested call (an interrupt) while the state is in use gets "busy",
    /// and the silence reads as in force
    #[test]
    fn busy_reads_as_silent() {
        let _serial = serial();
        let nested = SILENCE.with(|_| {
            (
                [rf_silence_init(), rf_silence_arm()],
                rf_silence_active(),
                rf_silence_get_state(),
                rf_silence_elapsed_seconds(),
            )
        });
        assert_eq!(nested, Some(([rc::BUSY; 2], true, State::Counting as u32, 0)));
    }
}
