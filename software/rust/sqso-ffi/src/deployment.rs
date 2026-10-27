//! `deployment.h`: the deployment wait and burn sequence
//! (`sqso_fsw::deployment`), on the GPIO and flash HAL, the reset cause,
//! the separation switch and the flight log. Each wing resistor has its own
//! line: wing 1 A `GPIO_PIN_BURN_W1A` (= `GPIO_PIN_SOLAR_BURN`, PB10),
//! wing 1 B `GPIO_PIN_BURN_W1B` (= `GPIO_PIN_SOLAR_BURN_B`, PE6), wing 2 A
//! `GPIO_PIN_BURN_W2A` (= `GPIO_PIN_ANTENNA_BURN`, PB9) and wing 2 B
//! `GPIO_PIN_BURN_W2B` (logical, its STM32 pin to be allocated), one at a
//! time; the antenna is the ISIS AntS, armed and deployed over I2C, sensed
//! by its own element switches (`ants.rs`).
//!
//! @requirement SYS-221 Deployables wait 30 minutes after separation; switch toggle resets the timer

use core::ffi::c_char;
use core::fmt;

use sqso_fsw::deployment::{DeployIo, Deployment, Element, ElementStatus, Level, Line, Report, State, Telemetry};

use crate::c::{self, HalFlash, rc, severity};
use crate::cell::Global;
use crate::rf_control::{GPIO_DIR_INPUT, GPIO_DIR_OUTPUT, gpio_config, gpio_set};
use crate::{ptr, rf_silence, separation_switch};

/// `FLASH_REGION_DEPLOYMENT`
pub(crate) const FLASH_REGION_DEPLOYMENT: u32 = 10;
/// `GPIO_PIN_DEPLOY_EN`
pub(crate) const GPIO_PIN_DEPLOY_EN: u32 = 4;
/// `GPIO_PIN_ANTENNA_BURN`, now `GPIO_PIN_BURN_W2A` (wing 2, resistor A)
pub(crate) const GPIO_PIN_ANTENNA_BURN: u32 = 11;
/// `GPIO_PIN_BURN_W2B` (wing 2, resistor B; logical pin, its STM32 pin
/// is the OBC board's to allocate)
pub(crate) const GPIO_PIN_BURN_W2B: u32 = 23;
/// `GPIO_PIN_SOLAR_BURN`
pub(crate) const GPIO_PIN_SOLAR_BURN: u32 = 12;
/// `GPIO_PIN_SOLAR_BURN_B`
pub(crate) const GPIO_PIN_SOLAR_BURN_B: u32 = 18;
/// `GPIO_PIN_ANTENNA_SENSE`
pub(crate) const GPIO_PIN_ANTENNA_SENSE: u32 = 13;
/// `GPIO_PIN_SOLAR_SENSE`: both wings' hinge switches in series, the EPS's
/// 10 kΩ + 1 kΩ pull-up; **low = both wings deployed** (SQSO-ICX-001 §3.1
/// pin 20). No internal pull: a pull-down would override the pull-up
pub(crate) const GPIO_PIN_SOLAR_SENSE: u32 = 14;
const GPIO_PULL_NONE: u32 = 0;

/// Whether SOLAR_SENSE reads both wings deployed (the line low)
pub(crate) fn wings_sensed() -> bool {
    // SAFETY: plain value (the HAL)
    !unsafe { c::hal_gpio_get(GPIO_PIN_SOLAR_SENSE) }
}
/// `FAULT_TYPE_DEPLOYMENT`
pub(crate) const FAULT_TYPE_DEPLOYMENT: u32 = 21;
/// `LOG_LEVEL_DEBUG`
pub(crate) const LOG_LEVEL_DEBUG: u32 = 1;
/// `LOG_LEVEL_INFO`
pub(crate) const LOG_LEVEL_INFO: u32 = 2;

/// `DeploymentElementStatus_t`
pub type DeploymentElementStatus = ElementStatus;
/// `DeploymentTelemetry_t`
pub type DeploymentTelemetry = Telemetry;

static DEPLOY: Global<Deployment> = Global::new(Deployment::new(FLASH_REGION_DEPLOYMENT));

/// A flight-log message, NUL-terminated (cut at LOG_MAX_MESSAGE_LEN - 1)
struct Message {
    buf: [u8; 128],
    len: usize,
}

impl fmt::Write for Message {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            if let Some(slot) = self.buf.get_mut(self.len).filter(|_| self.len < 127) {
                *slot = if b == 0 { b'?' } else { b };
                self.len += 1;
            }
        }
        Ok(())
    }
}

/// The HAL drives the lines and reads the sense inputs and the switch; the
/// flight log and the fault log take the messages
impl DeployIo for HalFlash {
    fn configure_outputs(&mut self) {
        for pin in
            [GPIO_PIN_DEPLOY_EN, GPIO_PIN_ANTENNA_BURN, GPIO_PIN_SOLAR_BURN, GPIO_PIN_SOLAR_BURN_B, GPIO_PIN_BURN_W2B]
        {
            gpio_config(pin, GPIO_DIR_OUTPUT);
        }
    }

    fn configure_inputs(&mut self) {
        gpio_config(GPIO_PIN_ANTENNA_SENSE, GPIO_DIR_INPUT);
        // SAFETY: plain values (the HAL): input, no pull (the EPS's pull-up)
        let _ = unsafe { c::hal_gpio_config(GPIO_PIN_SOLAR_SENSE, GPIO_DIR_INPUT, GPIO_PULL_NONE) };
        separation_switch::separation_switch_init();
    }

    fn set(&mut self, line: Line, on: bool) {
        match line {
            Line::Arm => gpio_set(GPIO_PIN_DEPLOY_EN, on),
            Line::Wing(0, 0) => gpio_set(GPIO_PIN_SOLAR_BURN, on),
            Line::Wing(0, _) => gpio_set(GPIO_PIN_SOLAR_BURN_B, on),
            Line::Wing(_, 0) => gpio_set(GPIO_PIN_ANTENNA_BURN, on),
            Line::Wing(_, _) => gpio_set(GPIO_PIN_BURN_W2B, on),
            Line::Ants => {
                // A refused write shows as an element not sensed: the
                // sequence retries
                let _ = crate::ants::command(on);
            }
        }
    }

    fn sensed(&mut self, element: Element) -> bool {
        match element {
            Element::SolarPanel => wings_sensed(),
            Element::Antenna => crate::ants::status().is_some_and(|s| s.all_deployed()),
        }
    }

    fn stowed(&mut self) -> bool {
        separation_switch::separation_switch_stowed()
    }

    fn ants_temperature_c(&mut self) -> Option<f64> {
        crate::ants::temperature_c()
    }

    fn log(&mut self, level: Level, message: fmt::Arguments<'_>) {
        let mut m = Message { buf: [0; 128], len: 0 };
        let _ = fmt::write(&mut m, message);
        let level = match level {
            Level::Debug => LOG_LEVEL_DEBUG,
            Level::Info => LOG_LEVEL_INFO,
        };
        let _ = crate::flight_log::write_str(level, b"DEPLOY", m.buf.get(..m.len).unwrap_or(&[]));
    }

    fn report(&mut self, report: Report) {
        let text = match report {
            Report::Failed(Element::Antenna) => c"Antenna deployment failed",
            Report::Failed(Element::SolarPanel) => c"Solar panel deployment failed",
            Report::NotPersisted => c"Deployment attempt not persisted: burn withheld",
            Report::Unreadable => c"Deployment record unreadable: starting over",
            Report::AntsDeferred => c"AntS deployment deferred: below -17 C or no temperature",
        };
        c::log(FAULT_TYPE_DEPLOYMENT, severity::ERROR, text, 0.0);
    }
}

/// Boot: burn lines low, load the sequence, apply the reset-cause rules
#[unsafe(no_mangle)]
pub extern "C" fn deployment_init() -> bool {
    crate::ants::reset();
    let cause = rf_silence::reset();
    DEPLOY.with(|d| d.init(cause, &mut HalFlash)).is_some()
}

/// Advance the sequence by elapsed time
#[unsafe(no_mangle)]
pub extern "C" fn deployment_tick(elapsed_ms: u32) {
    let _ = DEPLOY.with(|d| d.tick(elapsed_ms, &mut HalFlash));
}

/// Re-arm for flight (GSE only)
#[unsafe(no_mangle)]
pub extern "C" fn deployment_arm() -> i32 {
    match DEPLOY.with(|d| d.arm(&mut HalFlash)) {
        Some(true) => rc::OK,
        Some(false) => rc::IO,
        None => rc::BUSY,
    }
}

/// Whether separation has been detected
#[unsafe(no_mangle)]
pub extern "C" fn deployment_is_separated() -> bool {
    DEPLOY.with(|d| d.is_separated()).unwrap_or(false)
}

/// Current state (`DeploymentState_t`)
#[unsafe(no_mangle)]
pub extern "C" fn deployment_get_state() -> u32 {
    DEPLOY.with(|d| d.state()).unwrap_or(State::Inhibited) as u32
}

/// Seconds counted toward the 30-minute wait
#[unsafe(no_mangle)]
pub extern "C" fn deployment_wait_elapsed_s() -> u32 {
    DEPLOY.with(|d| d.wait_elapsed_s()).unwrap_or(0)
}

/// An element's status; false for an unknown element or NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn deployment_get_element_status(element: u32, status: *mut DeploymentElementStatus) -> bool {
    // SAFETY: the C contract: NULL or a DeploymentElementStatus_t
    let (Some(e), Some(out)) = (Element::from_raw(element), unsafe { ptr::out(status) }) else { return false };
    match DEPLOY.with(|d| d.element_status(e)) {
        Some(s) => {
            *out = s;
            true
        }
        None => false,
    }
}

/// Deployment telemetry; false for NULL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn deployment_get_telemetry(telemetry: *mut DeploymentTelemetry) -> bool {
    // SAFETY: the C contract: NULL or a DeploymentTelemetry_t
    let Some(out) = (unsafe { ptr::out(telemetry) }) else { return false };
    match DEPLOY.with(|d| d.telemetry()) {
        Some(t) => {
            *out = t;
            true
        }
        None => false,
    }
}

/// Enable or inhibit deployment (ground command); persisted
#[unsafe(no_mangle)]
pub extern "C" fn deployment_set_enabled(enable: bool) -> bool {
    DEPLOY.with(|d| d.set_enabled(enable, &mut HalFlash)).unwrap_or(false)
}

/// Retry an element (ground command)
#[unsafe(no_mangle)]
pub extern "C" fn deployment_force_retry(element: u32) -> bool {
    let Some(e) = Element::from_raw(element) else { return false };
    DEPLOY.with(|d| d.force_retry(e, &mut HalFlash)).unwrap_or(false)
}

/// DEPLOY_RETRY with its force flag (the antenna only: past the AntS
/// temperature gate)
#[unsafe(no_mangle)]
pub extern "C" fn deployment_retry(element: u32, force: bool) -> bool {
    let Some(e) = Element::from_raw(element) else { return false };
    DEPLOY.with(|d| d.retry(e, force, &mut HalFlash)).unwrap_or(false)
}

/// The AntS gate's deferrals and last temperature (NaN: none valid)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn deployment_ants_gate(deferrals: *mut u8, temp_c: *mut f64) -> bool {
    let Some((n, t)) = DEPLOY.with(|d| d.ants_gate_status()) else { return false };
    // SAFETY: the C contract: each pointer is NULL or a live object
    unsafe {
        if let Some(p) = crate::ptr::out(deferrals) {
            *p = n;
        }
        if let Some(p) = crate::ptr::out(temp_c) {
            *p = t.unwrap_or(f64::NAN);
        }
    }
    true
}

/// The AntS gate for housekeeping: deferrals, last temperature
pub(crate) fn ants_gate() -> (u8, Option<f64>) {
    DEPLOY.with(|d| d.ants_gate_status()).unwrap_or((0, None))
}

/// State name (static); "UNKNOWN" for a value outside the enum
#[unsafe(no_mangle)]
pub extern "C" fn deployment_state_name(state: u32) -> *const c_char {
    State::from_raw(state).map_or(c"UNKNOWN", State::name).as_ptr()
}

/// Whether both elements are confirmed deployed
#[unsafe(no_mangle)]
pub extern "C" fn deployment_all_complete() -> bool {
    DEPLOY.with(|d| d.all_complete()).unwrap_or(false)
}

/// Whether the wings' attempts are spent with the wings not deployed and
/// their sense line reading stowed (EPS-10: the mode is held at IDLE; busy
/// reads as failed, the safe answer)
#[unsafe(no_mangle)]
pub extern "C" fn deployment_wings_failed() -> bool {
    DEPLOY.with(|d| d.wings_exhausted()).unwrap_or(true) && !wings_sensed()
}

/// Persist the sequence state
#[unsafe(no_mangle)]
pub extern "C" fn deployment_save_state() -> bool {
    DEPLOY.with(|d| d.save_state(&mut HalFlash)).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing)]

    use core::fmt::Write;

    use super::*;

    fn text(m: &Message) -> &[u8] {
        &m.buf[..m.len]
    }

    /// A flight-log message shows a NUL as '?' and is cut at 127 bytes,
    /// leaving its terminating NUL
    #[test]
    fn message_is_cut_and_keeps_its_nul() {
        let mut m = Message { buf: [0; 128], len: 0 };
        assert!(m.write_str("a\0b").is_ok());
        assert_eq!(text(&m), b"a?b");
        assert!(m.write_str(&"x".repeat(200)).is_ok());
        assert_eq!(m.len, 127);
        assert_eq!(&m.buf[124..], b"xxx\0");
    }
}
