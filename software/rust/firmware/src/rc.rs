//! `SmartQsoResult_t` (smart_qso.h)

pub(crate) const OK: i32 = 0;
pub(crate) const ERROR: i32 = -1;
pub(crate) const NULL_PTR: i32 = -2;
pub(crate) const INVALID: i32 = -3;
pub(crate) const TIMEOUT: i32 = -4;
pub(crate) const BUSY: i32 = -5;
pub(crate) const IO: i32 = -7;

/// A C buffer as a slice (NULL is `None`)
///
/// # Safety
/// `data` is NULL or points to `len` readable bytes that stay valid and
/// unchanged for the slice's use
pub(crate) unsafe fn bytes<'a>(data: *const u8, len: usize) -> Option<&'a [u8]> {
    // SAFETY: the caller's contract
    (!data.is_null()).then(|| unsafe { core::slice::from_raw_parts(data, len) })
}

/// A writable C buffer as a slice (NULL is `None`)
///
/// # Safety
/// `data` is NULL or points to `len` writable bytes, not otherwise
/// referenced during the slice's use
pub(crate) unsafe fn bytes_mut<'a>(data: *mut u8, len: usize) -> Option<&'a mut [u8]> {
    // SAFETY: the caller's contract
    (!data.is_null()).then(|| unsafe { core::slice::from_raw_parts_mut(data, len) })
}

/// A C out-parameter as a reference (NULL is `None`)
///
/// # Safety
/// `p` is NULL or points to a writable, aligned `T`, not otherwise
/// referenced during the reference's use
pub(crate) unsafe fn out<'a, T>(p: *mut T) -> Option<&'a mut T> {
    // SAFETY: the caller's contract
    unsafe { p.as_mut() }
}
