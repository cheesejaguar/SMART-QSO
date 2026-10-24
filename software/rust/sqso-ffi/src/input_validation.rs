//! `input_validation.h`: command framing and range checks

use sqso_protocol::frame::{self, FrameError};

use crate::c::rc;
use crate::ptr;

/// `VALIDATION_ERR_*`
const ERR_TOO_SHORT: u32 = 1;
const ERR_TOO_LONG: u32 = 2;
const ERR_SYNC: u32 = 3;
const ERR_LENGTH: u32 = 5;
const ERR_CHECKSUM: u32 = 6;

/// `CommandValidation_t`
#[repr(C)]
pub struct CommandValidation {
    is_valid: bool,
    error_code: u32,
    cmd_id: u8,
    payload_length: u8,
}

/// `Vector3_t`
#[repr(C)]
pub struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

/// Validate framing, length and checksum of an uplink frame
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_command(cmd_data: *const u8, cmd_length: usize, result: *mut CommandValidation) -> i32 {
    // SAFETY: the C contract: cmd_data has cmd_length bytes
    let (Some(data), Some(result)) = (unsafe { ptr::bytes(cmd_data, cmd_length) }, unsafe { ptr::out(result) }) else {
        return rc::PARAM;
    };
    *result = CommandValidation { is_valid: false, error_code: 0, cmd_id: 0, payload_length: 0 };
    match frame::parse(data) {
        Ok(f) => {
            result.is_valid = true;
            result.cmd_id = f.cmd_id;
            result.payload_length = u8::try_from(f.payload.len()).unwrap_or(u8::MAX);
        }
        Err(e) => {
            result.error_code = match e {
                FrameError::TooShort => ERR_TOO_SHORT,
                FrameError::TooLong => ERR_TOO_LONG,
                FrameError::Sync => ERR_SYNC,
                FrameError::Length { .. } => ERR_LENGTH,
                FrameError::Checksum { .. } => ERR_CHECKSUM,
            };
            if let FrameError::Length { cmd_id, declared } | FrameError::Checksum { cmd_id, declared } = e {
                result.cmd_id = cmd_id;
                result.payload_length = declared;
            }
        }
    }
    rc::OK
}

fn in_range(value: f32, min: f32, max: f32) -> bool {
    !value.is_nan() && value >= min && value <= max
}

unsafe fn report(is_valid: *mut bool, valid: bool) -> i32 {
    // SAFETY: the C contract: is_valid points to a bool
    match unsafe { ptr::out(is_valid) } {
        Some(out) => {
            *out = valid;
            rc::OK
        }
        None => rc::PARAM,
    }
}

/// Bus or battery voltage, 0 to 10 V
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_voltage(voltage: f32, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, in_range(voltage, 0.0, 10.0)) }
}

/// Current, -5 to 5 A
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_current(current: f32, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, in_range(current, -5.0, 5.0)) }
}

/// Temperature, -50 to 100 degrees C
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_temperature(temperature: f32, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, in_range(temperature, -50.0, 100.0)) }
}

/// Magnetometer, each axis -100 to 100 uT
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_magnetometer(mag: *const Vector3, is_valid: *mut bool) -> i32 {
    // SAFETY: the C contract
    let Some(m) = (unsafe { ptr::in_(mag) }) else { return rc::PARAM };
    let ok = [m.x, m.y, m.z].iter().all(|v| in_range(*v, -100.0, 100.0));
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, ok) }
}

/// Sun sensor, 0 to 3.3 V
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_sun_sensor(voltage: f32, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, in_range(voltage, 0.0, 3.3)) }
}

/// Configuration value within [min, max]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_config_uint32(value: u32, min: u32, max: u32, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, value >= min && value <= max) }
}

/// Configuration value within [min, max]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_config_int32(value: i32, min: i32, max: i32, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, value >= min && value <= max) }
}

/// Configuration value within [min, max], not NaN
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_config_float(value: f32, min: f32, max: f32, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, in_range(value, min, max)) }
}

/// Printable ASCII, NUL-terminated within max_length bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_string(s: *const u8, max_length: usize, is_valid: *mut bool) -> i32 {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::out(is_valid) }) else { return rc::PARAM };
    if s.is_null() {
        return rc::PARAM;
    }
    *out = false;
    for i in 0..max_length {
        // SAFETY: the C contract: s is readable up to its NUL or max_length bytes
        let c = unsafe { s.add(i).read() };
        if c == 0 {
            *out = true;
            return rc::OK;
        }
        if !(0x20..=0x7E).contains(&c) {
            return rc::OK;
        }
    }
    rc::OK
}

/// Non-NULL buffer of at least required_size bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_buffer(buffer: *const core::ffi::c_void, buffer_size: usize, required_size: usize, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, !buffer.is_null() && buffer_size >= required_size) }
}

/// Non-NULL pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_pointer(p: *const core::ffi::c_void, is_valid: *mut bool) -> i32 {
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, !p.is_null()) }
}

/// Pointer aligned to `alignment` bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn input_validate_alignment(p: *const core::ffi::c_void, alignment: usize, is_valid: *mut bool) -> i32 {
    let ok = !p.is_null() && alignment != 0 && (p as usize) % alignment == 0;
    // SAFETY: forwarded C contract
    unsafe { report(is_valid, ok) }
}
