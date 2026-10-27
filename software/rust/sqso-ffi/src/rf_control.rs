//! `rf_control.h`: transmitter permission and the latched TX-OFF
//! (`sqso_fsw::rf_control`), on the GPIO and flash HAL.
//!
//! The exciter's keying watch (`sqso_fsw::ptt_watch`, C-014-9) latches
//! TX-OFF through the same latch.
//!
//! @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))

use sqso_fsw::ptt_watch::PttWatch;
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
            Report::ScrubInhibited => {
                (fault_type::INIT, severity::ERROR, c"RF state check: RAM copy disagreed, TX-OFF")
            }
            Report::ScrubRepersisted => (fault_type::INIT, severity::WARNING, c"RF state check: TX-OFF re-persisted"),
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
    match RF.with(|rf| code(rf.tx_on(&mut HalFlash))).unwrap_or(rc::BUSY) {
        // In the degraded mode without the FRAM the RAM copy is the record
        // (RID SMA-B10): applied is done
        rc::IO if crate::fram_health::degraded() => rc::OK,
        other => other,
    }
}

/// Whether any transmitter may radiate now
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_tx_permitted() -> bool {
    RF.with(|rf| rf.permitted()).unwrap_or(false)
}

/// Apply or lift one automatic inhibit (`RfInhibit_t`)
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_set_inhibit(source: u32, active: bool) {
    RF.with(|rf| rf.set_inhibit(source, active, &mut HalFlash)).unwrap_or_else(|| {
        // Busy (a nested call): an inhibit is never lost
        if active {
            gpio_set(GPIO_PIN_RF_TX_EN, false);
            if source & rf_control::RADIO_OFF != 0 {
                gpio_set(GPIO_PIN_RADIO_EN, false);
            }
        }
    });
}

/// Check the RAM copy of the latch against its record (AV-07)
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_verify() {
    let _ = RF.with(|rf| rf.verify(&mut HalFlash));
}

/// `GPIO_PIN_XPDR_KEY`: the transponder's repeat keyed (input)
const GPIO_PIN_XPDR_KEY: u32 = 41;
/// `GPIO_PIN_PTT_EXC`: the exciter keyed, PTT_EXCITER (input)
const GPIO_PIN_PTT_EXC: u32 = 42;

static PTT: Global<PttWatch> = Global::new(PttWatch::new());

/// The exciter's keying watch (C-014-9 of SW-HAZ-014, SRS-F168): PTT_EXCITER
/// read high for more than `sqso_fsw::ptt_watch::LIMIT_MS` with XPDR_KEY low
/// and no downlink frame outstanding latches TX-OFF and logs the fault,
/// once per episode. True when it acted now (busy: does nothing)
///
/// @requirement SRS-F168 TX-OFF when PTT_EXCITER reads high for more than 6 s with the repeat path not keyed and no downlink frame outstanding; logged; recovered by TX_ENABLE
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_ptt_supervise(now_ms: u64, frame_outstanding: bool) -> bool {
    // SAFETY: plain values (the HAL)
    let (ptt, repeat) = unsafe { (c::hal_gpio_get(GPIO_PIN_PTT_EXC), c::hal_gpio_get(GPIO_PIN_XPDR_KEY)) };
    let trip = PTT.with(|w| w.step(now_ms, ptt, repeat, frame_outstanding)).unwrap_or(false);
    if trip {
        let _ = rf_control_tx_off();
        c::log(
            fault_type::COMMAND,
            severity::CRITICAL,
            c"PTT_EXCITER keyed over 6 s with no frame: TX-OFF (C-014-9)",
            0.0,
        );
    }
    trip
}

/// The exciter watch's trips since boot
#[unsafe(no_mangle)]
pub extern "C" fn rf_control_ptt_trips() -> u32 {
    PTT.with(|w| w.trips).unwrap_or(0)
}

/// The automatic inhibit bits in force (busy: none known)
#[cfg(test)]
pub(crate) fn inhibits() -> u32 {
    RF.with(|rf| rf.inhibits()).unwrap_or(0)
}

/// The one-minute check, in the degraded mode without the FRAM against the
/// latch's CRC (RID SMA-B10); whether the record was read (busy: not)
pub(crate) fn check(degraded: bool) -> bool {
    RF.with(|rf| rf.check(&mut HalFlash, degraded)).unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// Radio and transmitter enables configured as outputs, both on
    fn lines_on() {
        for pin in [GPIO_PIN_RADIO_EN, GPIO_PIN_RF_TX_EN] {
            gpio_config(pin, GPIO_DIR_OUTPUT);
            gpio_set(pin, true);
        }
    }

    /// (radio enable, transmitter enable)
    fn lines() -> (bool, bool) {
        // SAFETY: plain values
        unsafe { (c::hal_gpio_get(GPIO_PIN_RADIO_EN), c::hal_gpio_get(GPIO_PIN_RF_TX_EN)) }
    }

    /// A nested call (an interrupt) while the state is in use gets "busy":
    /// TX-OFF and an applied inhibit still drop the lines (the RF silence
    /// the radio too); lifting an inhibit and enabling TX do nothing
    #[test]
    fn busy_still_silences() {
        let _serial = serial();
        // SAFETY: no arguments
        assert_eq!(unsafe { c::hal_gpio_init() }, rc::OK);
        let nested = RF.with(|_| {
            let mut seen = [(true, true); 5];
            lines_on();
            let tx_off = rf_control_tx_off();
            seen[0] = lines();
            lines_on();
            rf_control_set_inhibit(rf_control::INHIBIT_CONTACT, true);
            seen[1] = lines();
            lines_on();
            rf_control_set_inhibit(rf_control::INHIBIT_RF_SILENCE, true);
            seen[2] = lines();
            lines_on();
            rf_control_set_inhibit(rf_control::INHIBIT_RF_SILENCE, false);
            seen[3] = lines();
            let commands = [tx_off, rf_control_tx_on(), rf_control_init(), rf_control_set_power_level(0)];
            seen[4] = lines();
            (commands, seen, rf_control_tx_permitted(), rf_control_get_power_level())
        });
        let seen = [(true, false), (true, false), (false, false), (true, true), (true, true)];
        // The exciter watch, nested: nothing done
        let ptt = PTT.with(|_| (rf_control_ptt_supervise(u64::MAX, false), rf_control_ptt_trips()));
        assert_eq!(ptt, Some((false, 0)));
        assert_eq!(nested, Some(([rc::BUSY; 4], seen, false, rf_control::POWER_0W5)));
    }
}
