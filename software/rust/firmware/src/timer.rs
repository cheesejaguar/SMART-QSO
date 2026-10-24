//! `hal_timer.h`: the Embassy time driver (a 32.768 kHz-tick hardware timer
//! extended to 64 bits) is the time base. Milliseconds since boot,
//! monotonic, not wall-clock time.

use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_time::{Duration, Instant, block_for};

use crate::rc;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// The time driver runs from reset; this marks the time base as in use
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_init() -> i32 {
    INITIALIZED.store(true, Ordering::Relaxed);
    rc::OK
}

/// Milliseconds since boot (0 before hal_timer_init)
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_get_ms() -> u64 {
    if INITIALIZED.load(Ordering::Relaxed) { Instant::now().as_millis() } else { 0 }
}

/// Microseconds since boot (0 before hal_timer_init)
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_get_us() -> u64 {
    if INITIALIZED.load(Ordering::Relaxed) { Instant::now().as_micros() } else { 0 }
}

/// Busy-wait (the main loop's idle time is spent asleep in the executor
/// instead; see main.rs)
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_delay_ms(ms: u32) {
    block_for(Duration::from_millis(u64::from(ms)));
}

/// Busy-wait
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_delay_us(us: u32) {
    block_for(Duration::from_micros(u64::from(us)));
}

/// Whether `timeout_ms` has passed since `start_ms`
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_timeout(start_ms: u64, timeout_ms: u32) -> bool {
    hal_timer_get_ms().wrapping_sub(start_ms) >= u64::from(timeout_ms)
}

/// Milliseconds since `start_ms`
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_elapsed_ms(start_ms: u64) -> u64 {
    hal_timer_get_ms().wrapping_sub(start_ms)
}

/// Callback timers are not provided: the flight software polls from its
/// main loop, and a callback from interrupt context would bypass the
/// module state guards. Refused, as in the C port.
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_start_periodic(
    _period_ms: u32,
    _callback: Option<unsafe extern "C" fn(*mut c_void)>,
    _context: *mut c_void,
    _timer_id: *mut u8,
) -> i32 {
    rc::ERROR
}

/// See hal_timer_start_periodic
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_stop(_timer_id: u8) -> i32 {
    rc::ERROR
}
