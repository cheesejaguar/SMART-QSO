//! `rf_control.h`: transmitter permission and the latched TX-OFF
//! (`sqso_fsw::rf_control`), on the GPIO and flash HAL.
//!
//! @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))

use sqso_fsw::rf_control::{self, Error, Report, RfControl, RfIo};

use crate::c::{self, HalFlash, fault_type, rc, severity};
use crate::cell::Global;

/// `FLASH_REGION_RF_CONTROL`
pub(crate) const FLASH_REGION_RF_CONTROL: u32 = 7;
/// `GPIO_PIN_RADIO_EN`
pub(crate) const GPIO_PIN_RADIO_EN: u32 = 1;
/// `GPIO_PIN_RF_TX_EN`
pub(crate) const GPIO_PIN_RF_TX_EN: u32 = 7;
/// `GPIO_DIR_INPUT`
pub(crate) const GPIO_DIR_INPUT: u32 = 0;
/// `GPIO_DIR_OUTPUT`
pub(crate) const GPIO_DIR_OUTPUT: u32 = 1;
/// `GPIO_PULL_DOWN`
pub(crate) const GPIO_PULL_DOWN: u32 = 2;

unsafe extern "C" {
    fn hal_gpio_set(pin: u32, state: bool) -> i32;
}

/// Drive a GPIO output (best effort, as the C did)
pub(crate) fn gpio_set(pin: u32, on: bool) {
    // SAFETY: plain values
    let _ = unsafe { hal_gpio_set(pin, on) };
}

/// Configure a GPIO pin
pub(crate) fn gpio_config(pin: u32, direction: u32) {
    // SAFETY: plain values
    let _ = unsafe { c::hal_gpio_config(pin, direction, GPIO_PULL_DOWN) };
}

static RF: Global<RfControl> = Global::new(RfControl::new(FLASH_REGION_RF_CONTROL));

/// The HAL drives the two lines and the fault log takes the reports
impl RfIo for HalFlash {
    fn drive(&mut self, radio_on: bool, tx_on: bool) {
        // Transmitter first when turning off, radio power first when turning on
        gpio_config(GPIO_PIN_RF_TX_EN, GPIO_DIR_OUTPUT);
        gpio_config(GPIO_PIN_RADIO_EN, GPIO_DIR_OUTPUT);
        if !tx_on {
            gpio_set(GPIO_PIN_RF_TX_EN, false);
        }
        gpio_set(GPIO_PIN_RADIO_EN, radio_on);
        if tx_on {
            gpio_set(GPIO_PIN_RF_TX_EN, true);
        }
    }

    fn report(&mut self, report: Report) {
        let (ty, sev, text) = match report {
            Report::Unreadable => (fault_type::INIT, severity::ERROR, c"RF state unreadable: TX inhibited"),
            Report::Corrupt => (fault_type::INIT, severity::ERROR, c"RF state corrupt: TX inhibited"),
            Report::TxOffLatched => (fault_type::COMMAND, severity::WARNING, c"TX-OFF latched"),
            Report::TxOffNotPersisted => (fault_type::COMMAND, severity::WARNING, c"TX-OFF applied, not persisted"),
            Report::TxEnabled => (fault_type::COMMAND, severity::INFO, c"TX enabled"),
            Report::TxEnabledNotPersisted => (fault_type::COMMAND, severity::INFO, c"TX enabled, not persisted"),
        };
        c::log(ty, sev, text, 0.0);
    }
}

fn code(result: Result<(), Error>) -> i32 {
    match result {
        Ok(()) => rc::OK,
        Err(Error::Invalid) => rc::INVALID,
        Err(Error::Io) => rc::IO,
    }
}

/// Load the latched TX state and drive the transmitter enable line
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_init() -> i32 {
    RF.with(|rf| {
        rf.init(&mut HalFlash);
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Cease all transmissions and latch the inhibit across resets
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_tx_off() -> i32 {
    RF.with(|rf| code(rf.tx_off(&mut HalFlash))).unwrap_or_else(|| {
        // Busy (a nested call): the transmitter goes off all the same
        gpio_set(GPIO_PIN_RF_TX_EN, false);
        rc::BUSY
    })
}

/// Clear the inhibit and permit transmission
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_tx_on() -> i32 {
    RF.with(|rf| code(rf.tx_on(&mut HalFlash))).unwrap_or(rc::BUSY)
}

/// Whether any transmitter may radiate now
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_tx_permitted() -> bool {
    RF.with(|rf| rf.permitted()).unwrap_or(false)
}

/// Apply or lift one automatic inhibit (`RfInhibit_t`)
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_set_inhibit(source: u32, active: bool) {
    if RF.with(|rf| rf.set_inhibit(source, active, &mut HalFlash)).is_none() && active {
        // Busy (a nested call): an inhibit is never lost
        gpio_set(GPIO_PIN_RF_TX_EN, false);
        if source & rf_control::INHIBIT_RF_SILENCE != 0 {
            gpio_set(GPIO_PIN_RADIO_EN, false);
        }
    }
}

/// Select the downlink power level (`RfPowerLevel_t`) and persist it
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_set_power_level(level: u32) -> i32 {
    RF.with(|rf| code(rf.set_power_level(level, &mut HalFlash))).unwrap_or(rc::BUSY)
}

/// Current downlink power level
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_get_power_level() -> u32 {
    RF.with(|rf| rf.power_level()).unwrap_or(rf_control::POWER_0W5)
}
