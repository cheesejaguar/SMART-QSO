//! `watchdog_mgr.h`: the supervised hardware watchdog (`hal_wdt.h`) and the
//! task table (`sqso_fsw::watchdog`). The hardware is refreshed only while
//! every registered task checks in (SW-15); a stall is logged and the
//! pre-reset callback saves state before the hardware resets the OBC.
//!
//! The pre-reset callback and the fault log are called with no borrow of
//! the module state held (the callback may call back into this module).
//!
//! @requirement SRS-F050 Kick hardware watchdog at least every 30 seconds
//! @requirement SRS-F051 Save state before watchdog reset

use core::ffi::{c_char, c_void};

use sqso_fsw::fault_log::DESC_LEN;
use sqso_fsw::watchdog::{self as wdt, HW_TIMEOUT_MS, MAX_TASKS, NAME_LEN, Stats, TaskEntry, Watchdog, WdtError};

use crate::c::{self, fault_type, rc, severity};
use crate::cell::Global;
use crate::{fault_mgmt, ptr};

/// `WdtPreResetCallback_t`
pub type PreResetCallback = extern "C" fn();

/// `HalWdtMode_t`: reset on timeout
const WDT_MODE_RESET: u32 = 0;

unsafe extern "C" {
    fn hal_wdt_init(timeout_ms: u32, mode: u32) -> i32;
    fn hal_wdt_start() -> i32;
    fn hal_wdt_kick();
    fn hal_wdt_remaining_ms() -> u32;
    fn hal_wdt_caused_reset() -> bool;
    fn hal_wdt_clear_reset_flag();
    fn hal_wdt_set_warning_callback(callback: Option<extern "C" fn(*mut c_void)>, context: *mut c_void) -> i32;
}

struct Module {
    wdt: Watchdog,
    callback: Option<PreResetCallback>,
}

static WDT: Global<Module> = Global::new(Module { wdt: Watchdog::new(), callback: None });

fn callback() -> Option<PreResetCallback> {
    WDT.with(|m| m.callback).flatten()
}

/// The hardware's early warning: log, save state, count the reset
extern "C" fn hw_wdt_warning_handler(_context: *mut c_void) {
    c::log(fault_type::WATCHDOG, severity::CRITICAL, c"Hardware watchdog timeout imminent", 0.0);
    if let Some(cb) = callback() {
        cb();
    }
    let _ = WDT.with(|m| m.wdt.stats.resets_caused = m.wdt.stats.resets_caused.wrapping_add(1));
}

/// Check every registered task; a task that has just stalled is logged and
/// state is saved (the refresh stops now, so the hardware will reset the
/// OBC: SRS-F051)
fn check_task_watchdogs(now_ms: u64) {
    for index in 0..MAX_TASKS {
        let stalled = WDT.with(|m| {
            let name = m.wdt.tasks.get(index).map(|t| t.name).unwrap_or_default();
            m.wdt.check_task(index, now_ms).then_some((name, m.callback))
        });
        if let Some(Some((name, cb))) = stalled {
            let entry = TaskEntry { name, ..TaskEntry::default() };
            let msg = wdt::stall_message::<DESC_LEN>(entry.name());
            let _ = fault_mgmt::add(fault_type::WATCHDOG, severity::WARNING, &msg, 0.0);
            if let Some(cb) = cb {
                cb();
            }
        }
    }
    let _ = WDT.with(|m| m.wdt.finish_check(now_ms));
}

/// Initialize: tasks and statistics cleared (the reset count kept), the
/// hardware watchdog configured but not started
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_init() -> i32 {
    let cleared = WDT.with(|m| {
        m.wdt.reset();
        m.callback = None;
    });
    if cleared.is_none() {
        return rc::BUSY;
    }
    // SAFETY: plain values
    let result = unsafe { hal_wdt_init(HW_TIMEOUT_MS, WDT_MODE_RESET) };
    if result != rc::OK {
        return result;
    }
    // Early warning, if the hardware has it
    // SAFETY: a static function; no context
    let _ = unsafe { hal_wdt_set_warning_callback(Some(hw_wdt_warning_handler), core::ptr::null_mut()) };
    WDT.with(|m| m.wdt.initialized = true).map_or(rc::BUSY, |()| rc::OK)
}

/// Start the hardware watchdog (it cannot be stopped)
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_start() -> i32 {
    if !WDT.with(|m| m.wdt.initialized).unwrap_or(false) {
        return rc::ERROR;
    }
    // SAFETY: no arguments
    let result = unsafe { hal_wdt_start() };
    if result == rc::OK {
        let now = c::now_ms();
        let _ = WDT.with(|m| {
            m.wdt.stats.hw_wdt_running = true;
            m.wdt.last_task_check_ms = now;
        });
    }
    result
}

/// Check the tasks (once a second) and refresh the hardware while none is
/// stalled
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_kick() {
    let now = c::now_ms();
    let Some((true, due)) = WDT.with(|m| (m.wdt.initialized, m.wdt.check_due(now))) else { return };
    if due {
        check_task_watchdogs(now);
    }
    // A stalled task lets the hardware reset the OBC (SW-15)
    if WDT.with(|m| m.wdt.any_stalled()) == Some(false) {
        // SAFETY: no arguments
        unsafe { hal_wdt_kick() };
        let _ = WDT.with(|m| m.wdt.count_kick());
    }
}

/// Register a task (name cut to 15 characters)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wdt_mgr_register_task(task_id: u32, name: *const c_char, expected_period_ms: u32) -> i32 {
    if !Watchdog::valid_id(task_id) {
        return rc::PARAM;
    }
    if name.is_null() {
        return rc::NULL_PTR;
    }
    // At most NAME_LEN - 1 characters are read, up to the NUL
    let mut copy = [0u8; NAME_LEN];
    for (i, slot) in copy.iter_mut().enumerate().take(NAME_LEN - 1) {
        // SAFETY: the C contract: a C string; nothing past its NUL is read
        let b = unsafe { name.add(i).cast::<u8>().read() };
        if b == 0 {
            break;
        }
        *slot = b;
    }
    let len = copy.iter().position(|b| *b == 0).unwrap_or(NAME_LEN);
    let now = c::now_ms();
    match WDT.with(|m| m.wdt.register(task_id, copy.get(..len).unwrap_or(&[]), expected_period_ms, now)) {
        Some(Ok(())) => rc::OK,
        Some(Err(_)) => rc::PARAM,
        None => rc::BUSY,
    }
}

/// Stop monitoring a task
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_unregister_task(task_id: u32) -> i32 {
    match WDT.with(|m| m.wdt.unregister(task_id)) {
        Some(Ok(())) => rc::OK,
        Some(Err(_)) => rc::PARAM,
        None => rc::BUSY,
    }
}

/// A task's heartbeat
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_checkin(task_id: u32) -> i32 {
    let now = c::now_ms();
    match WDT.with(|m| m.wdt.checkin(task_id, now)) {
        Some(Ok(())) => rc::OK,
        Some(Err(WdtError::Param)) => rc::PARAM,
        Some(Err(WdtError::NotRegistered)) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Whether a task is stalled
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_is_task_stalled(task_id: u32) -> bool {
    WDT.with(|m| m.wdt.is_stalled(task_id)).unwrap_or(false)
}

/// Whether any registered task is stalled
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_any_task_stalled() -> bool {
    WDT.with(|m| m.wdt.any_stalled()).unwrap_or(false)
}

/// A task's entry
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wdt_mgr_get_task_info(task_id: u32, entry: *mut TaskEntry) -> i32 {
    // SAFETY: the C contract: NULL or a WdtTaskEntry_t (same layout)
    let Some(out) = (unsafe { ptr::out(entry) }) else { return rc::NULL_PTR };
    match WDT.with(|m| m.wdt.entry(task_id).copied()) {
        Some(Some(e)) => {
            *out = e;
            rc::OK
        }
        Some(None) => rc::PARAM,
        None => rc::BUSY,
    }
}

/// The statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wdt_mgr_get_stats(stats: *mut Stats) -> i32 {
    // SAFETY: the C contract: NULL or a WdtStats_t (same layout)
    let Some(out) = (unsafe { ptr::out(stats) }) else { return rc::NULL_PTR };
    match WDT.with(|m| m.wdt.stats) {
        Some(s) => {
            *out = s;
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Set the function that saves state before a watchdog reset (NULL: none)
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_register_prereset_callback(callback: Option<PreResetCallback>) -> i32 {
    WDT.with(|m| m.callback = callback).map_or(rc::BUSY, |()| rc::OK)
}

/// Whether the last reset was the watchdog's
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_was_wdt_reset() -> bool {
    // SAFETY: no arguments
    unsafe { hal_wdt_caused_reset() }
}

/// Clear the watchdog reset flag
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_clear_reset_status() {
    // SAFETY: no arguments
    unsafe { hal_wdt_clear_reset_flag() }
}

/// Time left before the hardware watchdog expires, ms
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_time_remaining() -> u32 {
    // SAFETY: no arguments
    unsafe { hal_wdt_remaining_ms() }
}

/// Log, save state, then stop refreshing and wait for the hardware reset
/// (does not return)
#[unsafe(no_mangle)]
pub extern "C" fn wdt_mgr_force_reset() {
    c::log(fault_type::WATCHDOG, severity::CRITICAL, c"Forced watchdog reset", 0.0);
    if let Some(cb) = callback() {
        cb();
    }
    loop {
        // Wait for the hardware watchdog to reset the OBC
        core::hint::spin_loop();
    }
}
