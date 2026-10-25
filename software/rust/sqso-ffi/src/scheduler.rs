//! `scheduler.h`: the time-triggered scheduler (`sqso_fsw::scheduler`).
//!
//! Task functions and the deadline callback are C function pointers, always
//! called with no scheduler state borrowed, so a task may call any
//! scheduler function (stop it, disable itself, change its period).

use core::ffi::c_char;

use sqso_fsw::scheduler::{NAME_LEN, SchedError, SchedStats, Scheduler, TaskConfig, TaskStats};

use crate::cell::Global;
use crate::ptr;

/// `task_func_t`
pub type TaskFunc = unsafe extern "C" fn();
/// `deadline_miss_cb_t`
pub type DeadlineMissCb = unsafe extern "C" fn(handle: u8, overrun_us: u32);

const SCHED_OK: u32 = 0;
const SCHED_ERROR_INVALID_PARAM: u32 = SchedError::InvalidParam as u32;
const SCHED_INVALID_HANDLE: u8 = 0xFF;

/// `task_config_t`
#[repr(C)]
pub struct CTaskConfig {
    name: [u8; NAME_LEN],
    func: Option<TaskFunc>,
    period_ms: u32,
    offset_ms: u32,
    deadline_ms: u32,
    priority: u32,
    enabled: bool,
}

/// `task_stats_t`
#[repr(C)]
pub struct CTaskStats {
    run_count: u32,
    last_run_time_us: u32,
    max_run_time_us: u32,
    min_run_time_us: u32,
    avg_run_time_us: u32,
    deadline_misses: u32,
    skip_count: u32,
}

/// `sched_stats_t`
#[repr(C)]
pub struct CSchedStats {
    tick_count: u32,
    cpu_utilization: u32,
    idle_time_us: u32,
    active_tasks: u32,
    total_deadline_misses: u32,
}

struct Sched {
    table: Scheduler<TaskFunc>,
    deadline_cb: Option<DeadlineMissCb>,
}

static SCHED: Global<Sched> = Global::new(Sched { table: Scheduler::new(), deadline_cb: None });

fn status(r: Result<(), SchedError>) -> u32 {
    match r {
        Ok(()) => SCHED_OK,
        Err(e) => e as u32,
    }
}

fn with<R>(busy: R, f: impl FnOnce(&mut Sched) -> R) -> R {
    SCHED.with(f).unwrap_or(busy)
}

/// Execution time, us. No timer is wired to the scheduler yet (the target
/// will use the DWT cycle counter), so run times read 0 and no deadline is
/// missed, as before.
fn time_us() -> u32 {
    0
}

/// Clear the task table
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_init() -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| {
        s.table.init();
        s.deadline_cb = None;
        SCHED_OK
    })
}

/// Register a task
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scheduler_register_task(config: *const CTaskConfig, handle: *mut u8) -> u32 {
    // SAFETY: the C contract
    let (Some(c), Some(out)) = (unsafe { ptr::in_(config) }, unsafe { ptr::out(handle) }) else {
        return SCHED_ERROR_INVALID_PARAM;
    };
    let Some(func) = c.func else { return SCHED_ERROR_INVALID_PARAM };
    let config = TaskConfig {
        name: c.name,
        func,
        period_ms: c.period_ms,
        offset_ms: c.offset_ms,
        deadline_ms: c.deadline_ms,
        priority: c.priority,
        enabled: c.enabled,
    };
    with(SCHED_ERROR_INVALID_PARAM, |s| match s.table.register(config) {
        Ok(h) => {
            *out = h;
            SCHED_OK
        }
        Err(e) => e as u32,
    })
}

/// Unregister a task (not the running one)
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_unregister_task(handle: u8) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.unregister(handle)))
}

/// Run ticks until a task calls scheduler_stop()
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_start() -> u32 {
    let started = with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.start()));
    if started != SCHED_OK {
        return started;
    }
    while scheduler_is_running() {
        scheduler_tick();
    }
    SCHED_OK
}

/// Stop the tick loop at the next opportunity
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_stop() {
    with((), |s| s.table.stop());
}

fn run_task(handle: u8) {
    let Some(func) = with(None, |s| s.table.begin_run(handle).ok()) else { return };
    let start = time_us();
    // SAFETY: a task function registered by C code
    unsafe { func() };
    let run_time = time_us().wrapping_sub(start);
    let missed = with(None, |s| s.table.finish_run(handle, run_time).map(|overrun| (s.deadline_cb, overrun)));
    if let Some((Some(cb), overrun)) = missed {
        // SAFETY: a callback registered by C code
        unsafe { cb(handle, overrun) };
    }
}

/// Run one tick: the most urgent due task, or idle
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_tick() {
    match with(None, |s| s.table.tick()) {
        Some(handle) => run_task(handle),
        None => {
            let start = time_us();
            let idle = time_us().wrapping_sub(start);
            with((), |s| s.table.idle(idle));
        }
    }
}

/// Enable a task (ready now)
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_enable_task(handle: u8) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.enable(handle)))
}

/// Disable a task
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_disable_task(handle: u8) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.disable(handle)))
}

/// Suspend a task
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_suspend_task(handle: u8) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.suspend(handle)))
}

/// Resume a suspended task
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_resume_task(handle: u8) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.resume(handle)))
}

/// A task's state (`task_state_t`)
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_get_task_state(handle: u8) -> u32 {
    with(0, |s| s.table.state(handle) as u32)
}

fn c_stats(t: TaskStats) -> CTaskStats {
    CTaskStats {
        run_count: t.run_count,
        last_run_time_us: t.last_run_time_us,
        max_run_time_us: t.max_run_time_us,
        min_run_time_us: t.min_run_time_us,
        avg_run_time_us: t.avg_run_time_us,
        deadline_misses: t.deadline_misses,
        skip_count: t.skip_count,
    }
}

/// A task's statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scheduler_get_task_stats(handle: u8, stats: *mut CTaskStats) -> u32 {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::out(stats) }) else { return SCHED_ERROR_INVALID_PARAM };
    with(SCHED_ERROR_INVALID_PARAM, |s| match s.table.stats(handle) {
        Ok(t) => {
            *out = c_stats(t);
            SCHED_OK
        }
        Err(e) => e as u32,
    })
}

/// Reset a task's statistics
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_reset_task_stats(handle: u8) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.reset_stats(handle)))
}

/// Scheduler statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scheduler_get_stats(stats: *mut CSchedStats) -> u32 {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::out(stats) }) else { return SCHED_ERROR_INVALID_PARAM };
    let s: SchedStats = with(SchedStats::default(), |s| s.table.sched_stats());
    *out = CSchedStats {
        tick_count: s.tick_count,
        cpu_utilization: s.cpu_utilization,
        idle_time_us: s.idle_time_us,
        active_tasks: s.active_tasks,
        total_deadline_misses: s.total_deadline_misses,
    };
    SCHED_OK
}

/// CPU utilization, %
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_get_cpu_utilization() -> u32 {
    with(0, |s| s.table.cpu_utilization())
}

/// Register the deadline-miss callback (NULL removes it)
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_register_deadline_callback(callback: Option<DeadlineMissCb>) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| {
        s.deadline_cb = callback;
        SCHED_OK
    })
}

/// Handle of the task with this name, or SCHED_INVALID_HANDLE
#[unsafe(no_mangle)]
pub unsafe extern "C" fn scheduler_get_handle_by_name(name: *const c_char) -> u8 {
    // SAFETY: the C contract: a C string
    let Some(name) = (unsafe { ptr::cstr(name) }) else { return SCHED_INVALID_HANDLE };
    with(None, |s| s.table.handle_by_name(name)).unwrap_or(SCHED_INVALID_HANDLE)
}

/// A task's name (valid while it is registered), or NULL
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_get_task_name(handle: u8) -> *const c_char {
    // The name lives in the static task table, so the pointer outlives the borrow
    with(core::ptr::null(), |s| s.table.name(handle).map_or(core::ptr::null(), |n| n.as_ptr().cast()))
}

/// Whether the tick loop is running
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_is_running() -> bool {
    with(false, |s| s.table.is_running())
}

/// Ticks since start
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_get_tick_count() -> u32 {
    with(0, |s| s.table.tick_count())
}

/// Cooperative delay: run scheduler ticks until `ms` ticks have passed.
/// (The C version waited for something else to advance the tick count,
/// which nothing did, so any delay above 0 never returned.)
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_delay_ms(ms: u32) {
    for _ in 0..ms {
        scheduler_tick();
    }
}

/// Run a task now, whatever its schedule
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_run_now(handle: u8) -> u32 {
    let known = with(Err(SchedError::InvalidParam), |s| s.table.stats(handle).map(|_| ()));
    if let Err(e) = known {
        return e as u32;
    }
    run_task(handle);
    SCHED_OK
}

/// Change a task's period
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_set_period(handle: u8, period_ms: u32) -> u32 {
    with(SCHED_ERROR_INVALID_PARAM, |s| status(s.table.set_period(handle, period_ms)))
}

/// A task's period, or 0
#[unsafe(no_mangle)]
pub extern "C" fn scheduler_get_period(handle: u8) -> u32 {
    with(0, |s| s.table.period(handle))
}
