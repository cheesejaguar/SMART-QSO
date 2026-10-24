//! `contact_monitor.h`: the loss-of-contact transmit inhibit
//! (`sqso_fsw::contact_monitor`), on the flash HAL and rf_control.
//!
//! @requirement SYS-205 Cease transmissions after N days without an authenticated telecommand

use sqso_fsw::contact_monitor::{ContactIo, ContactMonitor, Error, Report, TIMEOUT_DEFAULT_DAYS};
use sqso_fsw::rf_control::INHIBIT_CONTACT;

use crate::c::{self, HalFlash, fault_type, rc, severity};
use crate::cell::Global;
use crate::rf_control;

/// `FLASH_REGION_CONTACT`
pub(crate) const FLASH_REGION_CONTACT: u32 = 8;

static MONITOR: Global<ContactMonitor> = Global::new(ContactMonitor::new(FLASH_REGION_CONTACT));

/// rf_control takes the inhibit, the fault log takes the reports
impl ContactIo for HalFlash {
    fn set_inhibit(&mut self, active: bool) {
        rf_control::rf_control_set_inhibit(INHIBIT_CONTACT, active);
    }

    fn report(&mut self, report: Report) {
        match report {
            Report::NoContact(days) => {
                c::log(fault_type::COMMAND, severity::WARNING, c"No contact: transmitters inhibited", f64::from(days))
            }
            Report::CounterUnreadable => {
                c::log(fault_type::INIT, severity::ERROR, c"Contact counter unreadable: transmitters inhibited", 0.0)
            }
        }
    }
}

/// Load the counter and timeout, apply the boot penalty, persist, set the
/// inhibit
#[unsafe(no_mangle)]
pub extern "C" fn contact_monitor_init() -> i32 {
    MONITOR
        .with(|m| {
            m.init(&mut HalFlash);
            rc::OK
        })
        .unwrap_or(rc::BUSY)
}

/// Advance the counter by elapsed time; inhibit at the timeout
#[unsafe(no_mangle)]
pub extern "C" fn contact_monitor_tick(elapsed_ms: u32) {
    let _ = MONITOR.with(|m| m.tick(elapsed_ms, &mut HalFlash));
}

/// An authenticated telecommand was received
#[unsafe(no_mangle)]
pub extern "C" fn contact_monitor_contact() {
    let _ = MONITOR.with(|m| m.contact(&mut HalFlash));
}

/// Set the contact timeout and persist it
#[unsafe(no_mangle)]
pub extern "C" fn contact_monitor_set_timeout_days(days: u16) -> i32 {
    match MONITOR.with(|m| m.set_timeout_days(days, &mut HalFlash)) {
        Some(Ok(())) => rc::OK,
        Some(Err(Error::Invalid)) => rc::INVALID,
        Some(Err(Error::Io)) => rc::IO,
        None => rc::BUSY,
    }
}

/// Current contact timeout in days
#[unsafe(no_mangle)]
pub extern "C" fn contact_monitor_get_timeout_days() -> u16 {
    MONITOR.with(|m| m.timeout_days()).unwrap_or(TIMEOUT_DEFAULT_DAYS)
}

/// Accumulated seconds without contact
#[unsafe(no_mangle)]
pub extern "C" fn contact_monitor_seconds_since_contact() -> u32 {
    MONITOR.with(|m| m.seconds_since_contact()).unwrap_or(0)
}

/// Whether the loss-of-contact inhibit is in force
#[unsafe(no_mangle)]
pub extern "C" fn contact_monitor_inhibited() -> bool {
    MONITOR.with(|m| m.inhibited()).unwrap_or(false)
}
