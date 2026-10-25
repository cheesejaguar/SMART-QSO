//! `beacon_filter.h`: the beacon content filter (shared with the Jetson)
#![allow(non_upper_case_globals)]

use core::ffi::{CStr, c_char};

use sqso_protocol::filter::{self, COMMERCIAL, PROFANITY, Verdict};

use crate::ptr;

/// A C string pointer in a static table
#[repr(transparent)]
pub struct WordPtr(*const c_char);

// SAFETY: points to an immutable static string
unsafe impl Sync for WordPtr {}

const fn pointers<const N: usize>(words: &[&CStr]) -> [WordPtr; N] {
    let mut out = [const { WordPtr(core::ptr::null()) }; N];
    let mut i = 0;
    while i < N {
        #[allow(clippy::indexing_slicing)] // i < N == words.len(), checked at compile time
        {
            out[i] = WordPtr(words[i].as_ptr());
        }
        i += 1;
    }
    out
}

const PROFANITY_COUNT: usize = PROFANITY.len();
const COMMERCIAL_COUNT: usize = COMMERCIAL.len();

/// Obscene or indecent words
#[unsafe(no_mangle)]
pub static beacon_filter_profanity: [WordPtr; PROFANITY_COUNT] = pointers(PROFANITY);
/// Number of entries in beacon_filter_profanity
#[unsafe(no_mangle)]
pub static beacon_filter_profanity_count: usize = PROFANITY_COUNT;
/// Business, advertising or pecuniary words
#[unsafe(no_mangle)]
pub static beacon_filter_commercial: [WordPtr; COMMERCIAL_COUNT] = pointers(COMMERCIAL);
/// Number of entries in beacon_filter_commercial
#[unsafe(no_mangle)]
pub static beacon_filter_commercial_count: usize = COMMERCIAL_COUNT;

/// Check a beacon text (NUL bytes are refused, as any other control byte)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_filter_check(text: *const c_char, len: usize) -> u32 {
    // SAFETY: the C contract: text has len bytes
    match unsafe { ptr::bytes(text.cast(), len) } {
        Some(t) => filter::check(t) as u32,
        None => Verdict::Length as u32,
    }
}

/// Name of a result ("OK", "LENGTH", ...)
#[unsafe(no_mangle)]
pub extern "C" fn beacon_filter_reason(result: u32) -> *const c_char {
    Verdict::from_raw(result).map_or(c"UNKNOWN", Verdict::c_name).as_ptr()
}
