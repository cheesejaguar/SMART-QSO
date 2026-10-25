//! `contact_monitor.h`: the loss-of-contact transmit inhibit
//! (`sqso_fsw::contact_monitor`), on the flash HAL and rf_control.
//!
//! Coverage: only a nested call (an interrupt) finds `MONITOR` in use (busy:
//! `Global::with` gives `None`); no C callback runs while it is held. The
//! unit test `tests::busy_while_the_state_is_in_use` covers those paths
//! (`software/flight/scripts/safety_coverage.py` counts the unit tests'
//! export too).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// A nested call (an interrupt) while the state is in use gets "busy"
    /// and the defaults
    #[test]
    fn busy_while_the_state_is_in_use() {
        let _serial = serial();
        let nested = MONITOR.with(|_| {
            let calls = [contact_monitor_init(), contact_monitor_set_timeout_days(14)];
            (
                calls,
                contact_monitor_get_timeout_days(),
                contact_monitor_seconds_since_contact(),
                contact_monitor_inhibited(),
            )
        });
        assert_eq!(nested, Some(([rc::BUSY; 2], TIMEOUT_DEFAULT_DAYS, 0, false)));
    }
}
