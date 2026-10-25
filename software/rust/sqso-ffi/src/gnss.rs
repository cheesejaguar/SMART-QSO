//! `gnss.h`: the GNSS receiver NMEA driver (`sqso_fsw::gnss`, decision D6)
//! on the caller's `GnssRx_t`.
//!
//! `sqso_fsw::gnss::Receiver` and `Fix` have the C layouts, so the C
//! structs are used in place (sizes checked by `tests/test_rust_contract.c`).

use core::ffi::c_char;

use sqso_fsw::gnss::{self, Fix, Receiver};

use crate::ptr;

/// Reset a receiver
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gnss_rx_init(rx: *mut Receiver) {
    // SAFETY: the C contract: NULL or a GnssRx_t (same layout)
    if let Some(rx) = unsafe { ptr::out(rx) } {
        *rx = Receiver::new();
    }
}

/// Feed one received byte; true when a line completed (`status` set)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gnss_rx_feed(rx: *mut Receiver, c: c_char, now_ms: u64, status: *mut u32) -> bool {
    // SAFETY: the C contract: NULL or a GnssRx_t, NULL or a GnssStatus_t
    let (Some(rx), Some(status)) = (unsafe { ptr::out(rx) }, unsafe { ptr::out(status) }) else { return false };
    let [byte] = c.to_ne_bytes();
    match rx.feed(byte, now_ms) {
        Some(st) => {
            *status = st as u32;
            true
        }
        None => false,
    }
}

/// Check `$<body>*XX` (no line ending) against its checksum
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gnss_nmea_checksum_ok(line: *const c_char, len: usize) -> bool {
    // SAFETY: the C contract: NULL or len readable bytes
    unsafe { ptr::bytes(line.cast(), len) }.is_some_and(gnss::checksum_ok)
}

/// Latest accepted fix, if no older than `max_age_ms` at `now_ms`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gnss_latest_fix(rx: *const Receiver, now_ms: u64, max_age_ms: u32, out: *mut Fix) -> bool {
    // SAFETY: the C contract: NULL or a GnssRx_t, NULL or a GnssFix_t
    let (Some(rx), Some(out)) = (unsafe { ptr::in_(rx) }, unsafe { ptr::out(out) }) else { return false };
    match rx.latest_fix(now_ms, max_age_ms) {
        Some(fix) => {
            *out = fix;
            true
        }
        None => false,
    }
}
