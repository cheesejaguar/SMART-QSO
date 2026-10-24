//! `uplink.h`: receiver to AX.25 to command handler
//!
//! @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))

use core::ffi::c_char;

use sqso_protocol::ax25::{self, MAX_FRAME_LEN};

use crate::c::{self, rc};
use crate::cell::Global;
use crate::cmd_handler::{CmdResponse, cmd_process};
use crate::ptr;

/// `UplinkStats_t`
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct UplinkStats {
    frames: u32,
    rejected: u32,
    not_for_us: u32,
    commands: u32,
}

struct Uplink {
    callsign: [u8; 6],
    callsign_len: usize,
    ssid: u8,
    stats: UplinkStats,
    initialized: bool,
}

static UPLINK: Global<Uplink> = Global::new(Uplink {
    callsign: [0; 6],
    callsign_len: 0,
    ssid: 0,
    stats: UplinkStats { frames: 0, rejected: 0, not_for_us: 0, commands: 0 },
    initialized: false,
});

/// Set this station's address and start the receiver
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uplink_init(callsign: *const c_char, ssid: u8) -> i32 {
    // SAFETY: the C contract: a C string
    let Some(call) = (unsafe { ptr::cstr(callsign) }) else { return rc::NULL_PTR };
    if call.is_empty() || call.len() > 6 || ssid > 15 {
        return rc::INVALID;
    }
    let stored = UPLINK.with(|u| {
        u.callsign = [0; 6];
        for (dst, src) in u.callsign.iter_mut().zip(call) {
            *dst = *src;
        }
        u.callsign_len = call.len();
        u.ssid = ssid;
        u.stats = UplinkStats::default();
        u.initialized = true;
    });
    if stored.is_none() {
        return rc::BUSY;
    }
    // SAFETY: no arguments
    unsafe { c::hal_radio_init() }
}

fn count(f: impl FnOnce(&mut UplinkStats)) {
    let _ = UPLINK.with(|u| f(&mut u.stats));
}

/// Process at most one received frame; true if it reached the command handler
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uplink_poll(response: *mut CmdResponse) -> bool {
    let Some((callsign, len, ssid)) =
        UPLINK.with(|u| u.initialized.then_some((u.callsign, u.callsign_len, u.ssid))).flatten()
    else {
        return false;
    };
    let mut frame = [0u8; MAX_FRAME_LEN];
    let mut received = 0usize;
    // SAFETY: frame is a local buffer of MAX_FRAME_LEN bytes
    if unsafe { c::hal_radio_receive(frame.as_mut_ptr(), frame.len(), &mut received) } != rc::OK {
        return false;
    }
    count(|s| s.frames = s.frames.wrapping_add(1));

    let Some(frame) = frame.get(..received) else {
        count(|s| s.rejected = s.rejected.wrapping_add(1));
        return false;
    };
    let Ok(ui) = ax25::parse_ui(frame) else {
        count(|s| s.rejected = s.rejected.wrapping_add(1));
        return false;
    };
    if !ax25::address_matches(ui.dest, callsign.get(..len).unwrap_or(&[]), ssid) {
        count(|s| s.not_for_us = s.not_for_us.wrapping_add(1));
        return false;
    }

    let mut local = CmdResponse::zeroed();
    // SAFETY: the C contract: response is NULL or points to a CmdResponse_t
    let out = unsafe { ptr::out(response) }.unwrap_or(&mut local);
    // SAFETY: info is a live slice; out is a valid response
    if unsafe { cmd_process(ui.info.as_ptr(), ui.info.len(), out) } != rc::OK {
        count(|s| s.rejected = s.rejected.wrapping_add(1));
        return false;
    }
    count(|s| s.commands = s.commands.wrapping_add(1));
    true
}

/// Receive statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uplink_get_stats(stats: *mut UplinkStats) {
    // SAFETY: the C contract
    if let Some(out) = unsafe { ptr::out(stats) } {
        *out = UPLINK.with(|u| u.stats).unwrap_or_default();
    }
}
