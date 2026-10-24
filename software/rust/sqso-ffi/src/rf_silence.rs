//! `rf_silence.h`: the 45-minute post-deployment RF silence
//! (`sqso_fsw::rf_silence`), on the flash HAL, the reset cause and the
//! separation switch.
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
    let Some((state, elapsed)) = SILENCE.with(|s| {
        s.init(cause, &mut HalFlash);
        (s.state(), s.elapsed_seconds())
    }) else {
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
