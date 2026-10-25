//! `assert_handler.h`: the assertion handler (`sqso_fsw::assert_handler`),
//! and `smart_qso_assert_failed()` from `smart_qso.h`.
//!
//! Assertions stay active in every build, flight included (SW-08). A
//! critical failure enters safe mode and a fatal one resets: each is
//! fault-logged and the registered callback called, with no assertion state
//! borrowed, so the fault log or a callback may assert in turn.
//!
//! Coverage: only a nested call (an interrupt) finds `ASSERTS` in use (busy:
//! `Global::with` gives `None`); no C callback runs while it is held. The
//! unit test `tests::busy_while_the_state_is_in_use` covers those paths
//! (`software/flight/scripts/safety_coverage.py` counts the unit tests'
//! export too).
//!
//! @note NPR 7150.2 (standard, not a requirement ID): software safety
//! @note JPL Power of Ten rule 5 (SQSO-CS-001 §3): at least two assertions per function on average

use core::ffi::c_char;

use sqso_fsw::assert_handler::{self as fsw, Action, AssertLog, Record, Stats, kind, severity};

use crate::c::{self, rc};
use crate::cell::Global;
use crate::ptr;

/// A callback for safe mode or reset (`void (*)(void)`)
pub type Callback = unsafe extern "C" fn();

/// `FAULT_TYPE_ASSERTION` (smart_qso.h)
pub(crate) const FAULT_TYPE_ASSERTION: u32 = 18;
/// `ASSERT_TYPE_INVARIANT`
pub(crate) const ASSERT_TYPE_INVARIANT: u32 = 2;
/// `ASSERT_SEVERITY_FATAL`
pub(crate) const ASSERT_SEVERITY_FATAL: u32 = 3;

unsafe extern "C" {
    // system_state.c
    fn sys_get_uptime_s() -> u32;
}

struct Asserts {
    log: AssertLog,
    safe_mode: Option<Callback>,
    reset: Option<Callback>,
}

impl Asserts {
    fn init(&mut self) {
        self.log.init();
        self.safe_mode = None;
        self.reset = None;
    }
}

static ASSERTS: Global<Asserts> = Global::new(Asserts { log: AssertLog::new(), safe_mode: None, reset: None });

/// Initialize: clear the statistics, the log and the callbacks
#[unsafe(no_mangle)]
pub extern "C" fn assert_init() -> i32 {
    ASSERTS.with(Asserts::init).map_or(rc::BUSY, |()| rc::OK)
}

/// Record a failure and take the action its severity calls for
#[unsafe(no_mangle)]
pub unsafe extern "C" fn assert_handle_failure(
    file: *const c_char,
    line: u16,
    kind: u32,
    severity: u32,
    message: *const c_char,
) -> u32 {
    // SAFETY: the C contract: NULL or C strings
    let (file, message) = unsafe { (ptr::cstr(file), ptr::cstr(message)) };
    // SAFETY: no arguments
    let uptime_s = unsafe { sys_get_uptime_s() };
    let handled = ASSERTS.with(|a| {
        if !a.log.is_initialized() {
            a.init();
        }
        (a.log.handle(file, line, kind, severity, message, uptime_s), a.safe_mode, a.reset)
    });
    // Entered again while handling a failure (from the fault log or a
    // callback): not logged, but the response stands
    let (action, safe_mode, reset) = handled.unwrap_or((fsw::response(severity), None, None));
    match action {
        Action::SafeMode => {
            c::log(FAULT_TYPE_ASSERTION, c::severity::CRITICAL, c"Assertion triggered safe mode", 0.0);
            if let Some(cb) = safe_mode {
                // SAFETY: a registered C callback
                unsafe { cb() };
            }
        }
        Action::Reset => {
            c::log(FAULT_TYPE_ASSERTION, c::severity::CRITICAL, c"Assertion triggered reset", 0.0);
            // The callback persists state; the reset itself is the
            // watchdog's (the panic handler waits for it after a Rust panic)
            if let Some(cb) = reset {
                // SAFETY: a registered C callback
                unsafe { cb() };
            }
        }
        Action::Continue | Action::ReturnError => {}
    }
    action as u32
}

/// The statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn assert_get_stats(stats: *mut Stats) -> i32 {
    // SAFETY: the C contract: NULL or an AssertStats_t (same layout)
    let Some(out) = (unsafe { ptr::out(stats) }) else { return rc::NULL_PTR };
    match ASSERTS.with(|a| a.log.stats()) {
        Some(s) => {
            *out = s;
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Copy up to `max_entries` records, oldest first
#[unsafe(no_mangle)]
pub unsafe extern "C" fn assert_get_log(log: *mut Record, max_entries: u16, count: *mut u16) -> i32 {
    // SAFETY: the C contract: NULL or a uint16_t
    let Some(count) = (unsafe { ptr::out(count) }) else { return rc::NULL_PTR };
    if log.is_null() {
        return rc::NULL_PTR;
    }
    if max_entries == 0 {
        *count = 0;
        return rc::OK;
    }
    let copied = ASSERTS.with(|a| {
        let n = a.log.len().min(usize::from(max_entries));
        for i in 0..n {
            // COVERAGE-EXEMPT: record(i) is Some for every i below len() (n is at most len()), so the else never runs
            let Some(r) = a.log.record(i) else { continue };
            // SAFETY: the C contract: log has max_entries records (i < max_entries)
            unsafe { log.add(i).write(*r) };
        }
        n
    });
    match copied {
        Some(n) => {
            *count = u16::try_from(n).unwrap_or(max_entries);
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Clear the counts and the log (the trigger counts are kept)
#[unsafe(no_mangle)]
pub extern "C" fn assert_clear_stats() -> i32 {
    ASSERTS.with(|a| a.log.clear_stats()).map_or(rc::BUSY, |()| rc::OK)
}

/// Whether a critical or fatal assertion has failed
#[unsafe(no_mangle)]
pub extern "C" fn assert_has_critical_failures() -> bool {
    ASSERTS.with(|a| a.log.has_critical_failures()).unwrap_or(false)
}

/// Register the safe-mode callback (NULL removes it)
#[unsafe(no_mangle)]
pub extern "C" fn assert_register_safe_mode_callback(callback: Option<Callback>) -> i32 {
    ASSERTS.with(|a| a.safe_mode = callback).map_or(rc::BUSY, |()| rc::OK)
}

/// Register the reset callback (NULL removes it)
#[unsafe(no_mangle)]
pub extern "C" fn assert_register_reset_callback(callback: Option<Callback>) -> i32 {
    ASSERTS.with(|a| a.reset = callback).map_or(rc::BUSY, |()| rc::OK)
}

/// A failed `SMART_QSO_REQUIRE` (smart_qso.h): an error-level precondition
/// failure. The host build also prints it to stderr.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_qso_assert_failed(file: *const c_char, line: u32, message: *const c_char) {
    let short_line = u16::try_from(line).unwrap_or(u16::MAX);
    // SAFETY: the C contract: NULL or C strings
    let _ = unsafe { assert_handle_failure(file, short_line, kind::PRECONDITION, severity::ERROR, message) };
    #[cfg(not(target_os = "none"))]
    {
        // SAFETY: as above
        let (file, message) = unsafe { (ptr::cstr(file), ptr::cstr(message)) };
        crate::flight_log::stderr_line(|out| {
            out.push(b"[ASSERT] ");
            out.push(file.unwrap_or(b"?"));
            let _ = core::fmt::write(out, format_args!(":{line}: "));
            out.push(message.unwrap_or(b""));
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// A nested call (an interrupt) while the handler state is in use gets
    /// "busy" and nothing is written; a nested failure still gets its
    /// response
    #[test]
    fn busy_while_the_state_is_in_use() {
        let _serial = serial();
        let mut stats = Stats { fatals: 7, ..Stats::default() };
        let mut count = 7u16;
        // Never written: the call is refused before any record is copied
        let log = core::ptr::NonNull::<Record>::dangling().as_ptr();
        let nested = ASSERTS.with(|_| {
            // SAFETY: live outputs; log is not written (busy)
            let (got_stats, got_log, action) = unsafe {
                (
                    assert_get_stats(&mut stats),
                    assert_get_log(log, 1, &mut count),
                    assert_handle_failure(c"f.c".as_ptr(), 1, kind::PRECONDITION, severity::ERROR, c"m".as_ptr()),
                )
            };
            let calls = [
                got_stats,
                got_log,
                assert_init(),
                assert_clear_stats(),
                assert_register_safe_mode_callback(None),
                assert_register_reset_callback(None),
            ];
            (calls, action, assert_has_critical_failures())
        });
        assert_eq!(nested, Some(([rc::BUSY; 6], Action::ReturnError as u32, false)));
        assert_eq!((stats.fatals, count), (7, 7));
    }
}
