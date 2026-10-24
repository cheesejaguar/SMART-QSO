//! `smart_qso.h` CRC-32 (IEEE 802.3)

use core::ffi::c_void;

use sqso_protocol::checksum::crc32;

use crate::ptr;

/// CRC-32 of `len` bytes (0 for NULL or empty)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_qso_crc32(data: *const c_void, len: usize) -> u32 {
    // SAFETY: the C contract: data points to len bytes
    unsafe { ptr::bytes(data.cast(), len) }.map_or(0, crc32)
}

/// Whether the CRC-32 of the data is `expected_crc`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_qso_verify_crc32(data: *const c_void, len: usize, expected_crc: u32) -> bool {
    // SAFETY: as smart_qso_crc32
    unsafe { smart_qso_crc32(data, len) == expected_crc }
}
