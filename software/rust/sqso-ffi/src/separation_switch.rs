//! `separation_switch.h`: the one reading of the separation (deployment)
//! switches, used by the RF silence and the deployment sequence so the
//! polarity is defined in exactly one place.
//!
//! TBR: the line is pulled up and reads high while stowed (in the
//! dispenser). Confirm against the separation-switch schematic.

use crate::c;

const GPIO_PIN_DEPLOY_SENSE: u32 = 9;
const GPIO_DIR_INPUT: u32 = 0;
const GPIO_PULL_UP: u32 = 1;

/// Configure the sense input
#[unsafe(no_mangle)]
pub extern "C" fn separation_switch_init() {
    // SAFETY: plain values
    let _ = unsafe { c::hal_gpio_config(GPIO_PIN_DEPLOY_SENSE, GPIO_DIR_INPUT, GPIO_PULL_UP) };
}

/// Whether the switches read stowed (TBR polarity: high = stowed)
#[unsafe(no_mangle)]
pub extern "C" fn separation_switch_stowed() -> bool {
    // SAFETY: plain value
    unsafe { c::hal_gpio_get(GPIO_PIN_DEPLOY_SENSE) }
}
