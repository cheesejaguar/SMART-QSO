//! `payload_guard.h`: OBC-enforced Jetson power and temperature limits
//! (SW-16), on `sqso_fsw::payload_guard`.

use core::ffi::c_char;

use sqso_fsw::payload_guard::{self, Guard, Outcome, Sample};

use crate::cell::Global;
use crate::ptr;

/// `PayloadSample_t`
pub type PayloadSample = Sample;

static GUARD: Global<Guard> = Global::new(Guard::new());

/// Forget every reading and trip
#[unsafe(no_mangle)]
pub extern "C" fn payload_guard_init() {
    let _ = GUARD.with(|g| *g = Guard::new());
}

/// Evaluate one set of measurements (NULL counts as none)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn payload_guard_step(sample: *const PayloadSample, payload_on: bool, now_ms: u64) -> u32 {
    // SAFETY: the C contract: NULL or a PayloadSample_t
    let sample = unsafe { ptr::in_(sample) };
    // Busy (a nested call): unsupervised, so the rail goes off
    GUARD.with(|g| g.step(sample, payload_on, now_ms) as u32).unwrap_or(Outcome::NoTelemetry as u32)
}

/// May the rail be turned on now?
#[unsafe(no_mangle)]
pub extern "C" fn payload_guard_may_enable(now_ms: u64) -> bool {
    GUARD.with(|g| g.may_enable(now_ms)).unwrap_or(false)
}

/// Trips since payload_guard_init()
#[unsafe(no_mangle)]
pub extern "C" fn payload_guard_trip_count() -> u32 {
    GUARD.with(|g| g.trip_count()).unwrap_or(0)
}

/// Fault-log text for a result (static)
#[unsafe(no_mangle)]
pub extern "C" fn payload_guard_reason(result: u32) -> *const c_char {
    payload_guard::reason(result).as_ptr()
}
