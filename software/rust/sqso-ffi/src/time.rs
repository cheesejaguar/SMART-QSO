//! `smart_qso.h` mission clock.
//!
//! On the host it is wall-clock time plus the offset tests add with
//! `smart_qso_sim_advance_ms()`, so time-dependent code runs without
//! sleeping. On the target it is milliseconds since boot from the HAL time
//! base (monotonic, not wall-clock time).

#[cfg(not(target_os = "none"))]
use core::sync::atomic::{AtomicU64, Ordering};

/// Time added by tests
#[cfg(not(target_os = "none"))]
static SIM_OFFSET_MS: AtomicU64 = AtomicU64::new(0);

/// Current mission time, ms
#[unsafe(no_mangle)]
pub extern "C" fn smart_qso_now_ms() -> u64 {
    now_ms()
}

/// Current mission time, ms (for Rust callers)
#[cfg(not(target_os = "none"))]
pub(crate) fn now_ms() -> u64 {
    crate::os::realtime_ms().wrapping_add(SIM_OFFSET_MS.load(Ordering::Relaxed))
}

/// Current mission time, ms (for Rust callers)
#[cfg(target_os = "none")]
pub(crate) fn now_ms() -> u64 {
    // SAFETY: no arguments
    unsafe { crate::c::hal_timer_get_ms() }
}

/// Simulation only: move the clock forward
#[cfg(not(target_os = "none"))]
#[unsafe(no_mangle)]
pub extern "C" fn smart_qso_sim_advance_ms(ms: u64) {
    let _ = SIM_OFFSET_MS.fetch_add(ms, Ordering::Relaxed);
}

/// Simulation only: total time added by smart_qso_sim_advance_ms()
#[cfg(not(target_os = "none"))]
#[unsafe(no_mangle)]
pub extern "C" fn smart_qso_sim_offset_ms() -> u64 {
    SIM_OFFSET_MS.load(Ordering::Relaxed)
}
