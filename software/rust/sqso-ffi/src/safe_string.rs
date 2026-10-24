//! `safe_string.h`: bounded string and memory operations
//! (`sqso_fsw::safe_string`).
//!
//! Each function reads a C string only as far as the C implementation did:
//! up to its NUL or the bound the function documents. String functions
//! refuse a source that overlaps the destination (`RESULT_ERROR_PARAM`);
//! `safe_memmove` is the one operation that allows overlap.
//!
//! @note Bounded string and memory copies (MISRA C:2012, SQSO-CS-001); no requirement ID defines this

use core::ffi::{c_char, c_int, c_void};

use sqso_fsw::safe_string as s;

use crate::c::rc;
use crate::ptr;

/// `RESULT_ERROR_TRUNCATED`
const TRUNCATED: i32 = -8;

/// Bytes of the C string at `p` before its NUL, reading at most `max`
///
/// # Safety
/// `p` is non-NULL and readable up to its NUL or `max` bytes, whichever
/// comes first.
pub(crate) unsafe fn strnlen(p: *const c_char, max: usize) -> usize {
    let mut n = 0;
    // SAFETY: per the caller, every byte before the NUL or the bound is readable
    while n < max && unsafe { p.cast::<u8>().add(n).read() } != 0 {
        n += 1;
    }
    n
}

/// The C string at `p` as a slice: its bytes before the NUL, at most `max`;
/// `None` if `p` is NULL
///
/// # Safety
/// As [`strnlen`] for a non-NULL `p`, and the bytes are not written while
/// the slice lives.
pub(crate) unsafe fn bounded<'a>(p: *const c_char, max: usize) -> Option<&'a [u8]> {
    if p.is_null() {
        return None;
    }
    // SAFETY: per the caller
    let len = unsafe { strnlen(p, max) };
    // SAFETY: those len bytes were just read
    unsafe { ptr::bytes(p.cast(), len) }
}

fn addr<T>(p: *const T) -> usize {
    p as usize
}

/// Whether two byte ranges share an address
fn ranges_overlap(a: usize, a_len: usize, b: usize, b_len: usize) -> bool {
    if a <= b { b - a < a_len } else { a - b < b_len }
}

/// Write an optional `size_t` result
///
/// # Safety
/// `out` is NULL or points to a writable `size_t`.
unsafe fn put(out: *mut usize, value: usize) {
    // SAFETY: per the caller
    if let Some(o) = unsafe { ptr::out(out) } {
        *o = value;
    }
}

/// Copy a string into `dest` (common to strcpy and strncpy)
///
/// # Safety
/// `dest` (non-NULL) holds `dest_size` writable bytes; `src` (non-NULL) is
/// readable up to its NUL or `read` bytes.
unsafe fn copy_into(
    dest: *mut c_char,
    dest_size: usize,
    src: *const c_char,
    read: usize,
    max_chars: usize,
) -> Option<(usize, bool)> {
    // SAFETY: per the caller
    let src = unsafe { bounded(src, read) }?;
    // Only the bytes written are borrowed: the characters copied and the NUL
    let copied = src.len().min(dest_size.saturating_sub(1)).min(max_chars);
    let span = copied.saturating_add(1);
    if ranges_overlap(addr(dest), span, addr(src.as_ptr()), src.len().saturating_add(1)) {
        return None;
    }
    // SAFETY: per the caller (span <= dest_size); dest does not overlap src (checked)
    let dest = unsafe { ptr::bytes_mut(dest.cast(), span) }?;
    Some(s::copy(dest, src, max_chars))
}

/// Safe string copy with guaranteed NUL termination
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_strcpy(
    dest: *mut c_char,
    dest_size: usize,
    src: *const c_char,
    chars_copied: *mut usize,
) -> i32 {
    if dest.is_null() || src.is_null() || dest_size == 0 {
        return rc::PARAM;
    }
    // A source of dest_size characters or more does not fit; reading
    // dest_size bytes tells
    // SAFETY: the C contract: dest has dest_size bytes, src is a C string
    match unsafe { copy_into(dest, dest_size, src, dest_size, usize::MAX) } {
        Some((copied, truncated)) => {
            // SAFETY: the C contract: NULL or a size_t
            unsafe { put(chars_copied, copied) };
            if truncated { TRUNCATED } else { rc::OK }
        }
        None => rc::PARAM,
    }
}

/// Safe string copy of at most `max_chars` characters
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_strncpy(
    dest: *mut c_char,
    dest_size: usize,
    src: *const c_char,
    max_chars: usize,
    chars_copied: *mut usize,
) -> i32 {
    if dest.is_null() || src.is_null() || dest_size == 0 {
        return rc::PARAM;
    }
    let limit = (dest_size - 1).min(max_chars);
    // SAFETY: the C contract: dest has dest_size bytes, src is readable to
    // its NUL or max_chars bytes
    match unsafe { copy_into(dest, dest_size, src, limit, limit) } {
        Some((copied, _)) => {
            // SAFETY: the C contract
            unsafe { put(chars_copied, copied) };
            rc::OK
        }
        None => rc::PARAM,
    }
}

/// Safe string concatenation
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_strcat(
    dest: *mut c_char,
    dest_size: usize,
    src: *const c_char,
    total_length: *mut usize,
) -> i32 {
    if dest.is_null() || src.is_null() || dest_size == 0 {
        return rc::PARAM;
    }
    // SAFETY: the C contract: dest has dest_size bytes
    let dest_len = unsafe { strnlen(dest, dest_size) };
    if dest_len >= dest_size {
        return rc::PARAM; // not NUL-terminated within its size
    }
    let room = dest_size - dest_len - 1;
    // SAFETY: the C contract: src is a C string (read up to room + 1 bytes)
    let Some(src) = (unsafe { bounded(src, room.saturating_add(1)) }) else { return rc::PARAM };
    // Borrowed: the string, the characters appended and the NUL
    let span = dest_len.saturating_add(src.len().min(room)).saturating_add(1);
    if ranges_overlap(addr(dest), span, addr(src.as_ptr()), src.len().saturating_add(1)) {
        return rc::PARAM;
    }
    // SAFETY: the C contract (span <= dest_size); no overlap with src (checked)
    let Some(dest) = (unsafe { ptr::bytes_mut(dest.cast(), span) }) else { return rc::PARAM };
    match s::concat(dest, src) {
        Some((total, truncated)) => {
            // SAFETY: the C contract
            unsafe { put(total_length, total) };
            if truncated { TRUNCATED } else { rc::OK }
        }
        None => rc::PARAM,
    }
}

/// Safe string length with a maximum
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_strlen(string: *const c_char, max_length: usize, length: *mut usize) -> i32 {
    if string.is_null() || length.is_null() {
        return rc::PARAM;
    }
    // SAFETY: the C contract: string is readable to its NUL or max_length bytes
    let Some(text) = (unsafe { bounded(string, max_length) }) else { return rc::PARAM };
    let (len, truncated) = s::length(text, max_length);
    // SAFETY: non-NULL size_t (C contract)
    unsafe { put(length, len) };
    if truncated { TRUNCATED } else { rc::OK }
}

/// Compare two strings over at most `max` characters
///
/// # Safety
/// Both are non-NULL C strings (or readable for `max` bytes); `result` is
/// a writable `int`.
unsafe fn compare(str1: *const c_char, str2: *const c_char, max: usize, result: *mut c_int) -> Option<bool> {
    // SAFETY: per the caller
    let (a, b) = unsafe { (bounded(str1, max)?, bounded(str2, max)?) };
    let (order, exhausted) = s::compare(a, b, max);
    // SAFETY: per the caller
    *unsafe { ptr::out(result) }? = order;
    Some(exhausted)
}

/// Safe string comparison with a maximum length; truncated if the bound
/// was reached with no difference and no NUL
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_strcmp(
    str1: *const c_char,
    str2: *const c_char,
    max_length: usize,
    result: *mut c_int,
) -> i32 {
    // SAFETY: the C contract
    match unsafe { compare(str1, str2, max_length, result) } {
        Some(true) => TRUNCATED,
        Some(false) => rc::OK,
        None => rc::PARAM,
    }
}

/// Safe comparison of at most `n` characters
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_strncmp(str1: *const c_char, str2: *const c_char, n: usize, result: *mut c_int) -> i32 {
    // SAFETY: the C contract
    match unsafe { compare(str1, str2, n, result) } {
        Some(_) => rc::OK,
        None => rc::PARAM,
    }
}

/// Safe memory copy; overlapping buffers are refused
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_memcpy(dest: *mut c_void, dest_size: usize, src: *const c_void, count: usize) -> i32 {
    if dest.is_null() || src.is_null() || count > dest_size {
        return rc::PARAM;
    }
    if s::overlaps(addr(dest), addr(src), count) {
        return rc::PARAM;
    }
    // SAFETY: the C contract: count readable bytes at src and writable at
    // dest (count <= dest_size), not overlapping (checked)
    unsafe { core::ptr::copy_nonoverlapping(src.cast::<u8>(), dest.cast::<u8>(), count) };
    rc::OK
}

/// Safe memory move (overlap allowed)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_memmove(dest: *mut c_void, dest_size: usize, src: *const c_void, count: usize) -> i32 {
    if dest.is_null() || src.is_null() || count > dest_size {
        return rc::PARAM;
    }
    // SAFETY: the C contract: count readable bytes at src and writable at dest
    unsafe { core::ptr::copy(src.cast::<u8>(), dest.cast::<u8>(), count) };
    rc::OK
}

/// Safe memory set
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_memset(dest: *mut c_void, dest_size: usize, value: u8, count: usize) -> i32 {
    if dest.is_null() || count > dest_size {
        return rc::PARAM;
    }
    // SAFETY: the C contract: count writable bytes at dest (count <= dest_size)
    unsafe { core::ptr::write_bytes(dest.cast::<u8>(), value, count) };
    rc::OK
}

/// Safe memory comparison
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_memcmp(
    ptr1: *const c_void,
    ptr2: *const c_void,
    count: usize,
    result: *mut c_int,
) -> i32 {
    // SAFETY: the C contract: count readable bytes at each
    let (Some(a), Some(b)) = (unsafe { ptr::bytes(ptr1.cast(), count) }, unsafe { ptr::bytes(ptr2.cast(), count) })
    else {
        return rc::PARAM;
    };
    // SAFETY: the C contract: NULL or an int
    let Some(out) = (unsafe { ptr::out(result) }) else { return rc::PARAM };
    *out = s::compare_bytes(a, b);
    rc::OK
}

/// Write digits into a C buffer
///
/// # Safety
/// `buffer` (non-NULL) holds `buffer_size` writable bytes; `length` is NULL
/// or a writable `size_t`.
unsafe fn write_digits(digits: &[u8], buffer: *mut c_char, buffer_size: usize, length: *mut usize) -> i32 {
    // Only the bytes written are borrowed: the digits and the NUL, or the
    // first byte if they do not fit
    let span = buffer_size.min(digits.len().saturating_add(1));
    // SAFETY: per the caller (span <= buffer_size)
    let Some(out) = (unsafe { ptr::bytes_mut(buffer.cast(), span) }) else { return rc::PARAM };
    match s::write_digits(out, digits) {
        Some(n) => {
            // SAFETY: per the caller
            unsafe { put(length, n) };
            rc::OK
        }
        None => TRUNCATED,
    }
}

/// Signed integer to decimal string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_itoa(value: i32, buffer: *mut c_char, buffer_size: usize, length: *mut usize) -> i32 {
    if buffer.is_null() || buffer_size < 2 {
        return rc::PARAM;
    }
    let (digits, n) = s::format_i32(value);
    // SAFETY: the C contract
    unsafe { write_digits(digits.get(..n).unwrap_or(&[]), buffer, buffer_size, length) }
}

/// Unsigned integer to decimal string
#[unsafe(no_mangle)]
pub unsafe extern "C" fn safe_utoa(value: u32, buffer: *mut c_char, buffer_size: usize, length: *mut usize) -> i32 {
    if buffer.is_null() || buffer_size < 2 {
        return rc::PARAM;
    }
    let (digits, n) = s::format_u32(value);
    // SAFETY: the C contract
    unsafe { write_digits(digits.get(..n).unwrap_or(&[]), buffer, buffer_size, length) }
}
