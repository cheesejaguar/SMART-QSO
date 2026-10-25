//! `fault_mgmt.h`: the fault log (`sqso_fsw::fault_log`) and fault recovery.
//!
//! The log is stored through `store.rs` (a file on the host, its flash region
//! on the target) and printed through the console. Recovery raises
//! events through the mode manager and never sets the EPS power mode
//! directly (SW-14).
//!
//! Coverage: only a nested call (an interrupt) finds `FAULTS` in use (busy:
//! `Global::with` gives `None`); no C callback runs while it is held. The unit
//! test `tests::busy_while_the_log_is_in_use` asserts what such a call gets,
//! but the safety coverage gate cannot credit it: llvm-cov keeps one record per
//! `#[no_mangle]` function name (the C library's, whose hash differs from the
//! unit-test build's), so those lines are marked COVERAGE-EXEMPT.
//!
//! @requirement SRS-F040 Detect and log all fault conditions
//! @requirement SRS-F042 Implement autonomous fault recovery

use core::mem::size_of;

use sqso_fsw::fault_log::{ENTRY_SIZE, Entry, FaultLog, MAX_ENTRIES};
use sqso_fsw::mode::{self, Event};

use crate::c::{self, fault_type, rc, severity};
use crate::cell::Global;
use crate::store::{self, Store};
use crate::{ptr, state_machine};

struct Faults {
    log: FaultLog,
    watchdog_triggered: bool,
}

static FAULTS: Global<Faults> = Global::new(Faults { log: FaultLog::new(), watchdog_triggered: false });

const _: () = assert!(size_of::<Entry>() == ENTRY_SIZE);

/// The held entries as stored bytes
fn as_bytes(entries: &[Entry]) -> &[u8] {
    // SAFETY: Entry is repr(C) with its padding as named fields (size
    // asserted), so every byte of the slice is initialized
    unsafe { core::slice::from_raw_parts(entries.as_ptr().cast(), core::mem::size_of_val(entries)) }
}

/// Every slot as bytes, to load stored bytes into
fn as_bytes_mut(entries: &mut [Entry; MAX_ENTRIES]) -> &mut [u8] {
    // SAFETY: as above, and every bit pattern is a valid Entry (integers,
    // byte arrays, an f64 and a byte flag), so any bytes may be written
    unsafe { core::slice::from_raw_parts_mut(entries.as_mut_ptr().cast(), core::mem::size_of_val(entries)) }
}

/// Persist the log; where the store is smaller than the log (the target's
/// flash region), the newest entries that fit
fn save(log: &FaultLog) -> i32 {
    let held = log.held();
    let fit = store::capacity(Store::FaultLog) / core::mem::size_of::<Entry>();
    store::save(Store::FaultLog, as_bytes(held.get(held.len().saturating_sub(fit)..).unwrap_or(&[])))
}

/// Add an entry (arguments already checked); prints and persists it
pub(crate) fn add(fault_type: u32, severity: u32, description: &[u8], soc: f64) -> i32 {
    let (Ok(ty), Ok(sev)) = (u8::try_from(fault_type), u8::try_from(severity)) else { return rc::INVALID };
    let entry = Entry::new(c::now_ms(), ty, sev, description, soc);
    // Without core::fmt: this is the fault path (c::Line)
    let text = entry.description();
    c::Line::new()
        .text(b"[FAULT] Type=")
        .dec(fault_type)
        .text(b" Severity=")
        .dec(severity)
        .text(b": ")
        .text(if core::str::from_utf8(text).is_ok() { text } else { b"?" })
        .print();
    FAULTS
        .with(|f| {
            f.log.add(entry);
            let _ = save(&f.log);
            rc::OK
        })
        .unwrap_or(rc::BUSY)
}

/// Initialize: clear the log, then load it from storage
#[unsafe(no_mangle)]
pub extern "C" fn fault_mgmt_init() -> i32 {
    let _ = FAULTS.with(|f| {
        f.log.clear();
        f.watchdog_triggered = false;
    });
    if fault_log_load() == rc::OK {
        let count = fault_log_get_count();
        c::console(format_args!("[FAULT] Loaded {count} fault log entries"));
    }
    rc::OK
}

/// Add a fault log entry
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fault_log_add(
    fault_type: u32,
    severity: u32,
    description: *const core::ffi::c_char,
    soc: f64,
) -> i32 {
    // SAFETY: the C contract: a C string
    let Some(description) = (unsafe { ptr::cstr(description) }) else {
        c::assert_failed(c"Null pointer: description", line!());
        return rc::NULL_PTR;
    };
    if fault_type == 0 {
        c::assert_failed(c"Invalid fault type", line!());
        return rc::INVALID;
    }
    if !(severity::INFO..=severity::CRITICAL).contains(&severity) {
        c::assert_failed(c"Invalid severity", line!());
        return rc::INVALID;
    }
    add(fault_type, severity, description, soc)
}

/// Number of entries
#[unsafe(no_mangle)]
pub extern "C" fn fault_log_get_count() -> usize {
    FAULTS.with(|f| f.log.len()).unwrap_or(0)
}

/// Entry at `index` (0 is the oldest)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fault_log_get_entry(index: usize, entry: *mut Entry) -> i32 {
    // SAFETY: the C contract: entry points to a FaultLogEntry_t (same layout)
    let Some(out) = (unsafe { ptr::out(entry) }) else {
        c::assert_failed(c"Null pointer: entry", line!());
        return rc::NULL_PTR;
    };
    match FAULTS.with(|f| f.log.get(index).copied()) {
        Some(Some(e)) => {
            *out = e;
            rc::OK
        }
        Some(None) => {
            c::assert_failed(c"Index out of range", line!());
            rc::INVALID
        }
        // COVERAGE-EXEMPT: busy, only a nested call (an interrupt) gets here; see "Coverage" in the module doc
        None => rc::BUSY,
    }
}

/// The newest entry
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fault_log_get_last(entry: *mut Entry) -> i32 {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::out(entry) }) else {
        c::assert_failed(c"Null pointer: entry", line!());
        return rc::NULL_PTR;
    };
    match FAULTS.with(|f| f.log.last().copied()).flatten() {
        Some(e) => {
            *out = e;
            rc::OK
        }
        None => rc::ERROR,
    }
}

/// Mark an entry recovered and persist the log
#[unsafe(no_mangle)]
pub extern "C" fn fault_log_mark_recovered(index: usize) -> i32 {
    match FAULTS.with(|f| f.log.mark_recovered(index).then(|| save(&f.log))) {
        Some(Some(result)) => result,
        Some(None) => {
            c::assert_failed(c"Index out of range", line!());
            rc::INVALID
        }
        // COVERAGE-EXEMPT: busy, only a nested call (an interrupt) gets here; see "Coverage" in the module doc
        None => rc::BUSY,
    }
}

/// Persist the log
#[unsafe(no_mangle)]
pub extern "C" fn fault_log_save() -> i32 {
    FAULTS.with(|f| save(&f.log)).unwrap_or(rc::BUSY)
}

/// Load the log from storage, discarding corrupt entries
#[unsafe(no_mangle)]
pub extern "C" fn fault_log_load() -> i32 {
    FAULTS
        .with(|f| {
            let len = match store::load(Store::FaultLog, as_bytes_mut(f.log.slots_mut())) {
                Ok(len) => len,
                Err(e) => return e,
            };
            f.log.adopt(len / ENTRY_SIZE, &mut |i| c::console(format_args!("[FAULT] Discarded corrupt entry {i}")));
            rc::OK
        })
        .unwrap_or(rc::BUSY)
}

/// Clear the log and remove it from storage
#[unsafe(no_mangle)]
pub extern "C" fn fault_log_clear() -> i32 {
    let _ = FAULTS.with(|f| f.log.clear());
    store::erase(Store::FaultLog);
    rc::OK
}

fn sm_event(event: Event) -> i32 {
    // SAFETY: the context is system_state.c's
    state_machine::process(unsafe { c::sys_get_sm_context() }, event as u32).0
}

/// Thermal fault: payload off, then THERMAL_FAULT through the mode manager
#[unsafe(no_mangle)]
pub extern "C" fn fault_recover_thermal(soc: f64) -> i32 {
    // SAFETY: C EPS accessors with plain values
    if !unsafe { c::eps_is_payload_enabled() } {
        return rc::OK;
    }
    c::log(fault_type::THERMAL, severity::WARNING, c"Thermal fault recovery: disabling payload", soc);
    // SAFETY: plain values
    let _ = unsafe { c::eps_control_payload(false, soc) };
    sm_event(Event::ThermalFault)
}

/// Power fault: the event the power policy calls for (never into ACTIVE)
#[unsafe(no_mangle)]
pub extern "C" fn fault_recover_power(soc: f64) -> i32 {
    // SAFETY: the context is system_state.c's
    let state = state_machine::current(unsafe { c::sys_get_sm_context() });
    // Not sunlit: recovery sheds load or restores IDLE, never enters ACTIVE
    let event = match state {
        Ok(raw) => mode::State::from_raw(raw).map_or(Event::None, |s| mode::power_event(s, soc, false)),
        Err(_) => Event::None,
    };
    match event {
        Event::PowerCritical => {
            c::log(fault_type::POWER_CRITICAL, severity::ERROR, c"Critical power fault: shedding load", soc);
        }
        Event::PowerLow => {
            c::log(fault_type::POWER, severity::WARNING, c"Power fault recovery: entering IDLE mode", soc);
        }
        _ => {}
    }
    match (state, event) {
        (Err(e), _) => e,
        (Ok(_), Event::None) => rc::OK,
        (Ok(_), event) => sm_event(event),
    }
}

/// Watchdog timeout: logged, flag set
#[unsafe(no_mangle)]
pub extern "C" fn fault_handle_watchdog(soc: f64) -> i32 {
    c::log(fault_type::WATCHDOG, severity::ERROR, c"Watchdog timeout: system reset", soc);
    let _ = FAULTS.with(|f| f.watchdog_triggered = true);
    rc::OK
}

/// Whether the watchdog fired since the flag was cleared
#[unsafe(no_mangle)]
pub extern "C" fn fault_was_watchdog_triggered() -> bool {
    FAULTS.with(|f| f.watchdog_triggered).unwrap_or(false)
}

/// Clear the watchdog flag
#[unsafe(no_mangle)]
pub extern "C" fn fault_clear_watchdog_flag() {
    let _ = FAULTS.with(|f| f.watchdog_triggered = false);
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::{Mutex, MutexGuard, PoisonError};

    use super::*;

    /// Module state, the fault log and the simulated HAL are process-wide:
    /// the unit tests that use them hold this lock
    static SERIAL: Mutex<()> = Mutex::new(());

    pub(crate) fn serial() -> MutexGuard<'static, ()> {
        SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The newest fault log entry's description
    pub(crate) fn last_fault() -> Vec<u8> {
        let mut entry = Entry::new(0, 0, 0, b"", 0.0);
        // SAFETY: a live entry
        assert_eq!(unsafe { fault_log_get_last(&mut entry) }, rc::OK);
        entry.description().to_vec()
    }

    /// A nested call (an interrupt) while the log is in use gets "busy"
    /// and changes nothing
    #[test]
    fn busy_while_the_log_is_in_use() {
        let _serial = serial();
        let mut entry = Entry::new(0, 0, 0, b"untouched", 0.0);
        let nested = FAULTS.with(|_| {
            // SAFETY: a live entry
            let got = unsafe { fault_log_get_entry(0, &mut entry) };
            (got, fault_log_mark_recovered(0), add(fault_type::POWER, severity::INFO, b"nested", 0.0))
        });
        assert_eq!(nested, Some((rc::BUSY, rc::BUSY, rc::BUSY)));
        assert_eq!(entry.description(), b"untouched");
    }
}
