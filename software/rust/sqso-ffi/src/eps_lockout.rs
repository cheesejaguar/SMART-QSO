//! `eps_lockout.h`: clearing the EPS's latched recharge lockout by
//! telecommand (RID EPS-12; `docs/hardware/EPS_DESIGN.md` §3.3).
//!
//! EPS_CLEAR_LOCKOUT (argument `CHG!`) writes the unlock key to the EPS
//! controller (register 0x7D at 0x48 on I2C bus 0) and then pulses the
//! logical pin `GPIO_PIN_LOCK_CLR` high for
//! `sqso_fsw::eps_telemetry::lockout::PULSE_MS`; the EPS clears the lockout
//! only with both. The pulse ends in [`eps_lockout_poll`], called every
//! pass. Logged.
//!
//! @requirement SRS-F129 The software shall clear the EPS's latched recharge lockout only on an authenticated command with its code, by an I2C unlock and a LOCK_CLR pulse

use sqso_fsw::eps_telemetry::lockout::{CODE, PULSE_MS, REG_UNLOCK, UNLOCK_KEY};

use crate::c::{self, fault_type, rc, severity};
use crate::cell::Global;
use crate::ptr;

/// `GPIO_PIN_LOCK_CLR`
pub(crate) const GPIO_PIN_LOCK_CLR: u32 = 24;
const EPS_BUS: u32 = 0;
const EPS_ADDRESS: u8 = 0x48;

/// The pulse started at this time, ms
static PULSE: Global<Option<u64>> = Global::new(None);

/// Boot: no pulse running, the line low
#[unsafe(no_mangle)]
pub extern "C" fn eps_lockout_init() {
    let _ = PULSE.with(|p| *p = None);
    // SAFETY: plain values (the HAL)
    let _ = unsafe { c::hal_gpio_config(GPIO_PIN_LOCK_CLR, 1, 2) };
}

/// EPS_CLEAR_LOCKOUT with its argument: `INVALID` for a wrong code, `IO`
/// if the EPS controller did not take the unlock or the line cannot be
/// driven
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eps_lockout_clear(code: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: code has len bytes
    let Some(code) = (unsafe { ptr::bytes(code, len) }) else { return rc::NULL_PTR };
    if code != CODE {
        return rc::INVALID;
    }
    let write = [REG_UNLOCK, UNLOCK_KEY];
    // SAFETY: a live 2-byte buffer; plain values (the HAL)
    let unlocked = unsafe { c::hal_i2c_write(EPS_BUS, EPS_ADDRESS, write.as_ptr(), write.len()) } == rc::OK;
    // SAFETY: plain values (the HAL)
    let pulsed = unlocked
        && unsafe {
            c::hal_gpio_config(GPIO_PIN_LOCK_CLR, 1, 2) == rc::OK && c::hal_gpio_set(GPIO_PIN_LOCK_CLR, true) == rc::OK
        };
    if !pulsed {
        c::log(fault_type::POWER, severity::ERROR, c"EPS lockout clear failed", 0.0);
        return rc::IO;
    }
    let now = c::now_ms();
    let _ = PULSE.with(|p| *p = Some(now));
    c::log(fault_type::POWER, severity::WARNING, c"EPS recharge lockout clear commanded", 0.0);
    rc::OK
}

/// Every pass: end the `LOCK_CLR` pulse once it has lasted long enough
#[unsafe(no_mangle)]
pub extern "C" fn eps_lockout_poll() {
    let now = c::now_ms();
    let done = PULSE.with(|p| match *p {
        Some(since) if now.saturating_sub(since) >= PULSE_MS => {
            *p = None;
            true
        }
        _ => false,
    });
    if done == Some(true) {
        // SAFETY: plain values (the HAL)
        let _ = unsafe { c::hal_gpio_set(GPIO_PIN_LOCK_CLR, false) };
    }
}

/// Whether the `LOCK_CLR` pulse is running
#[unsafe(no_mangle)]
pub extern "C" fn eps_lockout_pulsing() -> bool {
    PULSE.with(|p| p.is_some()).unwrap_or(false)
}
