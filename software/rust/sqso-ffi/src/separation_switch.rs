//! `separation_switch.h`: the one reading of the separation (deployment)
//! switch, used by the RF silence and the deployment sequence so the
//! polarity is defined in exactly one place (`sqso_fsw::separation`).
//!
//! DEPLOY_SENSE (PE2, SQSO-ICX-001 §3.1 pin 16): **high = DS2 released**,
//! driven by the EPS through 1 MΩ with 1 MΩ to ground; the input has **no
//! internal pull**, which would override that source (RID SMA-B02).
//!
//! @requirement SRS-F159 DEPLOY_SENSE high = released, no internal pull; released for 10 s latches separation until the next reset, after which a stowed reading is counted and restarts neither timer

use sqso_fsw::separation::Separation;

use crate::c;
use crate::cell::Global;

/// `GPIO_PIN_DEPLOY_SENSE`
pub(crate) const GPIO_PIN_DEPLOY_SENSE: u32 = 9;
const GPIO_DIR_INPUT: u32 = 0;
/// `GPIO_PULL_NONE`: the EPS's 1 MΩ source and pull-down set the level
pub(crate) const GPIO_PULL_NONE: u32 = 0;

static SEPARATION: Global<Separation> = Global::new(Separation::new());

/// Configure the sense input (no pull) and start this boot's reading
#[unsafe(no_mangle)]
pub extern "C" fn separation_switch_init() {
    // SAFETY: plain values
    let _ = unsafe { c::hal_gpio_config(GPIO_PIN_DEPLOY_SENSE, GPIO_DIR_INPUT, GPIO_PULL_NONE) };
    let _ = SEPARATION.with(|s| *s = Separation::new());
}

/// Whether the timers are to treat the switch as stowed: the line low
/// (stowed) before separation latched on this boot; false after it
#[unsafe(no_mangle)]
pub extern "C" fn separation_switch_stowed() -> bool {
    // SAFETY: plain value
    let high = unsafe { c::hal_gpio_get(GPIO_PIN_DEPLOY_SENSE) };
    let now = c::now_ms();
    // Busy only for a nested call: the raw reading
    SEPARATION.with(|s| s.stowed(high, now)).unwrap_or(!high)
}

/// Separation latched on this boot, and the stowed readings since
/// (episodes), for housekeeping
#[unsafe(no_mangle)]
pub unsafe extern "C" fn separation_switch_status(latched: *mut bool, stowed_after: *mut u32) -> i32 {
    let Some((l, n)) = SEPARATION.with(|s| (s.latched, s.stowed_after_latch)) else { return c::rc::BUSY };
    // SAFETY: the C contract: each pointer is NULL or a live object
    unsafe {
        if let Some(p) = crate::ptr::out(latched) {
            *p = l;
        }
        if let Some(p) = crate::ptr::out(stowed_after) {
            *p = n;
        }
    }
    c::rc::OK
}

/// Separation latched and the stowed episodes since (housekeeping)
pub(crate) fn status() -> (bool, u32) {
    SEPARATION.with(|s| (s.latched, s.stowed_after_latch)).unwrap_or((false, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// A nested call (an interrupt) reads the raw line and reports busy
    #[test]
    fn status_and_busy() {
        let _serial = serial();
        let _ = crate::hal_sim::hal_gpio_init();
        separation_switch_init();
        assert!(!separation_switch_stowed(), "undriven: the simulated EPS drives it released");
        assert_eq!(status(), (false, 0));
        let (mut latched, mut after) = (true, 9u32);
        // SAFETY: live objects
        assert_eq!(unsafe { separation_switch_status(&mut latched, &mut after) }, c::rc::OK);
        assert_eq!((latched, after), (false, 0));
        let nested = SEPARATION.with(|_| {
            // SAFETY: NULL pointers are allowed
            (separation_switch_stowed(), status(), unsafe {
                separation_switch_status(core::ptr::null_mut(), core::ptr::null_mut())
            })
        });
        assert_eq!(nested, Some((false, (false, 0), c::rc::BUSY)));
    }
}
