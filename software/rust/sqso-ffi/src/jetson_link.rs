//! `jetson_link.h`: OBC side of the OBC-Jetson line protocol

use core::ffi::c_char;
use core::mem::{align_of, size_of};

use sqso_protocol::jetson::{self, FromJetson, LineRx, LinkError, TEXT_MAX};

use crate::ptr;

/// `JetsonMsgType_t`
const MSG_BEACON_RSP: u32 = 0;
const MSG_STATUS: u32 = 1;
const MSG_ACK: u32 = 2;
const MSG_NACK: u32 = 3;

/// `JetsonStatus_t`
const JETSON_OK: u32 = 0;
const JETSON_ERR_FORMAT: u32 = 1;
const JETSON_ERR_CRC: u32 = 2;
const JETSON_ERR_TYPE: u32 = 3;

/// `JetsonMsg_t`
#[repr(C)]
pub struct JetsonMsg {
    msg_type: u32,
    seq: u16,
    text: [u8; TEXT_MAX + 1],
    text_len: usize,
    health: u8,
    temp_c: i32,
    power_mw: u32,
    mem_pct: u8,
    uptime_s: u32,
}

/// `JetsonRx_t` is laid out as `LineRx` (line, len, overflow)
#[repr(C)]
pub struct JetsonRx {
    line: [u8; jetson::LINE_MAX],
    len: usize,
    overflow: bool,
}

const _: () = assert!(size_of::<JetsonRx>() == size_of::<LineRx>());
const _: () = assert!(align_of::<JetsonRx>() == align_of::<LineRx>());

fn status(e: LinkError) -> u32 {
    match e {
        LinkError::Format => JETSON_ERR_FORMAT,
        LinkError::Crc => JETSON_ERR_CRC,
        LinkError::Type => JETSON_ERR_TYPE,
    }
}

fn with_text(msg_type: u32, text: &str) -> JetsonMsg {
    let mut msg = JetsonMsg {
        msg_type,
        seq: 0,
        text: [0; TEXT_MAX + 1],
        text_len: text.len(),
        health: 0,
        temp_c: 0,
        power_mw: 0,
        mem_pct: 0,
        uptime_s: 0,
    };
    // The parser bounds every text to TEXT_MAX, leaving the NUL in place
    for (dst, src) in msg.text.iter_mut().take(TEXT_MAX).zip(text.bytes()) {
        *dst = src;
    }
    msg
}

fn convert(m: FromJetson<'_>) -> JetsonMsg {
    match m {
        FromJetson::BeaconRsp { seq, text } => JetsonMsg { seq, ..with_text(MSG_BEACON_RSP, text) },
        FromJetson::Status(s) => JetsonMsg {
            health: s.health,
            temp_c: s.temp_c,
            power_mw: s.power_mw,
            mem_pct: s.mem_pct,
            uptime_s: s.uptime_s,
            ..with_text(MSG_STATUS, "")
        },
        FromJetson::Ack(t) => with_text(MSG_ACK, t),
        FromJetson::Nack(t) => with_text(MSG_NACK, t),
    }
}

/// CRC-16/CCITT-FALSE
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_crc16(data: *const u8, len: usize) -> u16 {
    // SAFETY: the C contract
    sqso_protocol::checksum::crc16_ccitt_false(unsafe { ptr::bytes(data, len) }.unwrap_or(&[]))
}

/// Append ",<CRC>\n" to the message in buf
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_link_seal(buf: *mut c_char, len: usize, cap: usize) -> usize {
    // SAFETY: the C contract: buf has cap bytes
    let Some(buf) = (unsafe { ptr::bytes_mut(buf.cast(), cap) }) else { return 0 };
    jetson::seal_in_place(buf, len).unwrap_or(0)
}

fn built(r: Result<usize, LinkError>) -> usize {
    r.unwrap_or(0)
}

/// BEACON_REQ,<seq>,<max_len>
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_link_beacon_req(seq: u16, max_len: u8, out: *mut c_char, cap: usize) -> usize {
    // SAFETY: the C contract: out has cap bytes
    let Some(out) = (unsafe { ptr::bytes_mut(out.cast(), cap) }) else { return 0 };
    built(jetson::beacon_req(seq, max_len, out))
}

/// STATUS_REQ
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_link_status_req(out: *mut c_char, cap: usize) -> usize {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::bytes_mut(out.cast(), cap) }) else { return 0 };
    built(jetson::status_req(out))
}

/// SHUTDOWN,<reason>
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_link_shutdown(reason: *const c_char, out: *mut c_char, cap: usize) -> usize {
    // SAFETY: the C contract: reason is a C string, out has cap bytes
    let (Some(reason), Some(out)) = (unsafe { ptr::cstr(reason) }, unsafe { ptr::bytes_mut(out.cast(), cap) }) else {
        return 0;
    };
    match core::str::from_utf8(reason) {
        Ok(reason) => built(jetson::shutdown(reason, out)),
        Err(_) => 0,
    }
}

/// Check and parse one received line (trailing newline optional)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_link_parse(line: *const c_char, len: usize, msg: *mut JetsonMsg) -> u32 {
    // SAFETY: the C contract: line has len bytes, msg points to a JetsonMsg_t
    let (Some(line), Some(msg)) = (unsafe { ptr::bytes(line.cast(), len) }, unsafe { ptr::out(msg) }) else {
        return JETSON_ERR_FORMAT;
    };
    if line.is_empty() {
        return JETSON_ERR_FORMAT;
    }
    match jetson::parse_from_jetson(line) {
        Ok(m) => {
            *msg = convert(m);
            JETSON_OK
        }
        Err(e) => status(e),
    }
}

fn rx(rx: &mut JetsonRx) -> &mut LineRx {
    // SAFETY: JetsonRx and LineRx have the same size, alignment and fields
    // (line, len, overflow; both repr(C)), and LineRx accepts any field values
    unsafe { &mut *(rx as *mut JetsonRx).cast::<LineRx>() }
}

/// Reset a receive accumulator
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_rx_init(rx_state: *mut JetsonRx) {
    // SAFETY: the C contract
    if let Some(r) = unsafe { ptr::out(rx_state) } {
        rx(r).reset();
    }
}

/// Feed one received byte; true when a line ended (then *status and *msg)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn jetson_rx_feed(rx_state: *mut JetsonRx, c: c_char, msg: *mut JetsonMsg, status_out: *mut u32) -> bool {
    // SAFETY: the C contract: all three point to their structs
    let (Some(r), Some(msg), Some(status_out)) =
        (unsafe { ptr::out(rx_state) }, unsafe { ptr::out(msg) }, unsafe { ptr::out(status_out) })
    else {
        return false;
    };
    #[allow(clippy::cast_sign_loss)] // a C char is a byte
    let Some(line) = rx(r).feed(c as u8) else { return false };
    *status_out = match line.and_then(jetson::parse_from_jetson) {
        Ok(m) => {
            *msg = convert(m);
            JETSON_OK
        }
        Err(e) => status(e),
    };
    true
}
