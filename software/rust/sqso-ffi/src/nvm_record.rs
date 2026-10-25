//! `nvm_record.h`: power-fail-safe two-slot records

use core::ffi::c_void;

use sqso_fsw::nvm::{self, NvmError, RecordStatus};

use crate::c::{HalFlash, rc};
use crate::ptr;

/// `NvmRecordStatus_t`
const VALID: u32 = 0;
const BLANK: u32 = 1;
const CORRUPT: u32 = 2;

pub(crate) fn code(e: NvmError) -> i32 {
    match e {
        NvmError::Invalid => rc::INVALID,
        NvmError::Flash(f) => f.0,
        NvmError::Verify => rc::IO,
    }
}

/// Load the newest valid copy of the record at `base`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nvm_record_load_at(
    region: u32,
    base: u32,
    magic: u32,
    payload: *mut c_void,
    len: usize,
    status: *mut u32,
) -> i32 {
    // SAFETY: the C contract: payload has len writable bytes
    let (Some(payload), Some(status)) = (unsafe { ptr::bytes_mut(payload.cast(), len) }, unsafe { ptr::out(status) })
    else {
        return rc::NULL_PTR;
    };
    match nvm::load(&mut HalFlash, region, base, magic, payload) {
        Ok(s) => {
            *status = match s {
                RecordStatus::Valid => VALID,
                RecordStatus::Blank => BLANK,
                RecordStatus::Corrupt => CORRUPT,
            };
            rc::OK
        }
        Err(e) => code(e),
    }
}

/// Save into the slot not holding the newest valid copy
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nvm_record_save_at(
    region: u32,
    base: u32,
    magic: u32,
    payload: *const c_void,
    len: usize,
) -> i32 {
    // SAFETY: the C contract: payload has len bytes
    let Some(payload) = (unsafe { ptr::bytes(payload.cast(), len) }) else { return rc::NULL_PTR };
    match nvm::save(&mut HalFlash, region, base, magic, payload) {
        Ok(()) => rc::OK,
        Err(e) => code(e),
    }
}

/// nvm_record_load_at() at base 0
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nvm_record_load(
    region: u32,
    magic: u32,
    payload: *mut c_void,
    len: usize,
    status: *mut u32,
) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { nvm_record_load_at(region, 0, magic, payload, len, status) }
}

/// nvm_record_save_at() at base 0
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nvm_record_save(region: u32, magic: u32, payload: *const c_void, len: usize) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { nvm_record_save_at(region, 0, magic, payload, len) }
}
