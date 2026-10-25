//! C pointers to Rust references, after a NULL check.

use core::ffi::{CStr, c_char};

/// `len` bytes at `data`, or `None` if `data` is NULL. A zero length gives
/// an empty slice whatever the pointer.
///
/// # Safety
/// A non-NULL `data` must point to `len` readable bytes that are not
/// written for the lifetime `'a`.
pub(crate) unsafe fn bytes<'a>(data: *const u8, len: usize) -> Option<&'a [u8]> {
    if data.is_null() {
        return None;
    }
    if len == 0 {
        return Some(&[]);
    }
    // SAFETY: non-NULL; the caller guarantees len readable bytes
    Some(unsafe { core::slice::from_raw_parts(data, len) })
}

/// `len` writable bytes at `data`, or `None` if `data` is NULL
///
/// # Safety
/// A non-NULL `data` must point to `len` writable bytes not otherwise
/// accessed for the lifetime `'a`.
pub(crate) unsafe fn bytes_mut<'a>(data: *mut u8, len: usize) -> Option<&'a mut [u8]> {
    if data.is_null() {
        return None;
    }
    if len == 0 {
        return Some(&mut []);
    }
    // SAFETY: non-NULL; the caller guarantees len writable bytes
    Some(unsafe { core::slice::from_raw_parts_mut(data, len) })
}

/// A NUL-terminated C string's bytes (without the NUL), or `None` if NULL
///
/// # Safety
/// A non-NULL `s` must point to a NUL-terminated string valid for `'a`.
pub(crate) unsafe fn cstr<'a>(s: *const c_char) -> Option<&'a [u8]> {
    if s.is_null() {
        return None;
    }
    // SAFETY: non-NULL and NUL-terminated per the caller
    Some(unsafe { CStr::from_ptr(s) }.to_bytes())
}

/// `&mut T` from a C pointer, or `None` if NULL
///
/// # Safety
/// A non-NULL `p` must point to a valid, exclusively accessible `T`.
pub(crate) unsafe fn out<'a, T>(p: *mut T) -> Option<&'a mut T> {
    // SAFETY: per the caller
    unsafe { p.as_mut() }
}

/// `&T` from a C pointer, or `None` if NULL
///
/// # Safety
/// A non-NULL `p` must point to a valid `T`.
pub(crate) unsafe fn in_<'a, T>(p: *const T) -> Option<&'a T> {
    // SAFETY: per the caller
    unsafe { p.as_ref() }
}
