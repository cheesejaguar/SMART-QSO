//! `ax25.h`: address encoding, FCS and UI-frame parsing

use core::ffi::c_char;

use sqso_protocol::ax25::{self, ADDR_LEN, Ax25Error};

use crate::ptr;

/// `Ax25Status_t`
const AX25_OK: u32 = 0;
const AX25_ERR_FORMAT: u32 = 1;
const AX25_ERR_FCS: u32 = 2;
const AX25_ERR_NOT_UI: u32 = 3;

/// `Ax25UiView_t`
#[repr(C)]
pub struct Ax25UiView {
    dest: *const u8,
    src: *const u8,
    info: *const u8,
    info_len: usize,
}

/// Encode a call sign and SSID as a 7-byte address
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ax25_encode_address(callsign: *const c_char, ssid: u8, out: *mut [u8; ADDR_LEN], is_last: bool) {
    // SAFETY: the C contract: out points to 7 bytes; callsign is a C string or NULL
    if let Some(out) = unsafe { ptr::out(out) } {
        let call = unsafe { ptr::cstr(callsign) }.unwrap_or(&[]);
        *out = ax25::encode_address(call, ssid, is_last);
    }
}

/// Whether an address names the call sign and SSID
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ax25_address_matches(addr: *const [u8; ADDR_LEN], callsign: *const c_char, ssid: u8) -> bool {
    // SAFETY: the C contract: addr points to 7 bytes; callsign is a C string
    match unsafe { (ptr::in_(addr), ptr::cstr(callsign)) } {
        (Some(addr), Some(call)) => ax25::address_matches(addr, call, ssid),
        _ => false,
    }
}

/// AX.25 frame check sequence (CRC-16/X.25)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ax25_fcs(data: *const u8, len: usize) -> u16 {
    // SAFETY: the C contract
    ax25::fcs(unsafe { ptr::bytes(data, len) }.unwrap_or(&[]))
}

/// Check the FCS and parse a UI frame with PID 0xF0
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ax25_parse_ui(frame: *const u8, len: usize, view: *mut Ax25UiView) -> u32 {
    // SAFETY: the C contract: frame points to len bytes, view to an Ax25UiView_t
    let (Some(frame), Some(view)) = (unsafe { ptr::bytes(frame, len) }, unsafe { ptr::out(view) }) else {
        return AX25_ERR_FORMAT;
    };
    match ax25::parse_ui(frame) {
        Ok(ui) => {
            *view = Ax25UiView { dest: ui.dest.as_ptr(), src: ui.src.as_ptr(), info: ui.info.as_ptr(), info_len: ui.info.len() };
            AX25_OK
        }
        Err(Ax25Error::Format) => AX25_ERR_FORMAT,
        Err(Ax25Error::Fcs) => AX25_ERR_FCS,
        Err(Ax25Error::NotUi) => AX25_ERR_NOT_UI,
    }
}
