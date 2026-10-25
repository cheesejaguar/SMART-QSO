//! `beacon.h`: the beacon (`sqso_fsw::beacon`): text selection through the
//! content filter, telemetry from the EPS, measurements and fault log, the
//! information field and the AX.25 UI frame, gated by `rf_control` and the
//! EPS beacon switch.
#![allow(non_upper_case_globals)]

use core::ffi::c_char;

use sqso_fsw::beacon::{self as bcn, Beacon, Content, Frame, MAX_PAYLOAD_LEN, Measurements, Sources, State, TEMPLATES};
use sqso_fsw::downlink;
use sqso_fsw::fault_log::Entry;
use sqso_protocol::{ax25, filter};

use crate::c::{self, rc};
use crate::cell::Global;
use crate::{ax25 as ax25_ffi, beacon_filter, fault_mgmt, ptr};

/// `EpsControlState_t` (eps_control.h)
#[repr(C)]
pub struct EpsControlState {
    payload_enabled: bool,
    radio_enabled: bool,
    adcs_enabled: bool,
    beacon_enabled: bool,
    power_mode: u8,
    power_limit_w: f64,
    last_control_ms: u32,
    crc32: u32,
}

unsafe extern "C" {
    fn eps_get_power_mode() -> u32;
    fn eps_get_state(state: *mut EpsControlState) -> i32;
    fn eps_is_beacon_enabled() -> bool;
    fn rf_control_tx_permitted() -> bool;
}

/// A C string pointer in a static table
#[repr(transparent)]
pub struct TextPtr(*const c_char);

// SAFETY: points to an immutable static string
unsafe impl Sync for TextPtr {}

const TEMPLATE_COUNT: usize = TEMPLATES.len();

/// Fallback template messages
#[unsafe(no_mangle)]
pub static beacon_templates: [TextPtr; TEMPLATE_COUNT] = {
    let mut out = [const { TextPtr(core::ptr::null()) }; TEMPLATE_COUNT];
    let mut i = 0;
    while i < TEMPLATE_COUNT {
        #[allow(clippy::indexing_slicing)] // i < TEMPLATE_COUNT, at compile time
        {
            out[i] = TextPtr(TEMPLATES[i].as_ptr());
        }
        i += 1;
    }
    out
};

/// Number of entries in beacon_templates
#[unsafe(no_mangle)]
pub static beacon_template_count: usize = TEMPLATE_COUNT;

static BEACON: Global<Beacon> = Global::new(Beacon::new());

/// Largest frame on the air: flags, addresses, control, PID, a full
/// information field and the FCS
const FRAME_MAX: usize = 1 + 2 * ax25::ADDR_LEN + 2 + MAX_PAYLOAD_LEN + 2 + 1;

/// Longest information field `beacon_format_payload()` writes, NUL included
/// (the fixed fields and a whole text array); a larger buffer changes nothing
const PAYLOAD_WRITE_MAX: usize = 2 * (bcn::FIXED_MAX_LEN + bcn::MAX_TEXT_LEN);

/// Whether the radio may transmit and the EPS powers the beacon
fn transmit_permitted() -> bool {
    // SAFETY: C accessors without arguments
    unsafe { rf_control_tx_permitted() && eps_is_beacon_enabled() }
}

/// What the telemetry fields are read from now
fn sources() -> Sources {
    let mut eps = EpsControlState {
        payload_enabled: false,
        radio_enabled: false,
        adcs_enabled: false,
        beacon_enabled: false,
        power_mode: 0,
        power_limit_w: 0.0,
        last_control_ms: 0,
        crc32: 0,
    };
    let mut fault = Entry::ZERO;
    // SAFETY: C EPS accessors; eps is a live EpsControlState_t
    let (power_mode, eps_ok) = unsafe { (eps_get_power_mode(), eps_get_state(&mut eps) == rc::OK) };
    // SAFETY: fault is a live FaultLogEntry_t
    let fault_ok = unsafe { fault_mgmt::fault_log_get_last(&mut fault) } == rc::OK;
    Sources {
        now_ms: c::now_ms(),
        power_mode,
        radio_enabled: eps_ok.then_some(eps.radio_enabled),
        last_fault_severity: fault_ok.then_some(fault.severity),
    }
}

/// Initialize: ACTIVE rate, templates from the first, no history
#[unsafe(no_mangle)]
pub extern "C" fn beacon_init() -> i32 {
    BEACON.with(Beacon::init).map_or(rc::BUSY, |()| rc::OK)
}

/// Whether a beacon is due and may be sent
#[unsafe(no_mangle)]
pub extern "C" fn beacon_should_transmit(now_ms: u64) -> bool {
    // No beacon while a transmit inhibit is active or the EPS has shed the
    // beacon (state EMERGENCY)
    if !BEACON.with(|b| b.initialized).unwrap_or(false) || !transmit_permitted() {
        return false;
    }
    BEACON.with(|b| b.interval_elapsed(now_ms)).unwrap_or(false)
}

/// The interval for the power mode
#[unsafe(no_mangle)]
pub extern "C" fn beacon_update_interval(power_mode: u32) {
    let _ = BEACON.with(|b| b.update_interval(power_mode));
}

/// The next beacon's content
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_generate_content(
    content: *mut Content,
    ai_text: *const c_char,
    ai_text_len: usize,
) -> i32 {
    // SAFETY: the C contract: NULL or a BeaconContent_t
    let Some(out) = (unsafe { ptr::out(content) }) else { return rc::NULL_PTR };
    // SAFETY: the C contract: NULL or ai_text_len bytes
    let ai = unsafe { ptr::bytes(ai_text.cast(), ai_text_len) };
    let src = sources();
    match BEACON.with(|b| b.generate(ai, &src)) {
        Some(c) => {
            *out = c;
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// The information field, NUL-terminated; its length, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_format_payload(
    content: *const Content,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    // SAFETY: the C contract: content is a BeaconContent_t, buffer has
    // buffer_len bytes (at most PAYLOAD_WRITE_MAX of them are used)
    let (Some(content), Some(buf)) =
        (unsafe { ptr::in_(content) }, unsafe { ptr::bytes_mut(buffer.cast(), buffer_len.min(PAYLOAD_WRITE_MAX)) })
    else {
        return 0;
    };
    bcn::format_payload(content, buf)
}

/// The AX.25 UI frame of a beacon
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_build_ax25_frame(content: *const Content, frame: *mut Frame) -> i32 {
    // SAFETY: the C contract: NULL or a BeaconContent_t / Ax25Frame_t
    let (Some(content), Some(out)) = (unsafe { ptr::in_(content) }, unsafe { ptr::out(frame) }) else {
        return rc::NULL_PTR;
    };
    let (built, ok) = bcn::build_frame(content);
    *out = built;
    if ok { rc::OK } else { rc::ERROR }
}

/// The frame's bytes on the air; their number, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_serialize_ax25(frame: *const Frame, buffer: *mut u8, buffer_len: usize) -> usize {
    // SAFETY: the C contract: frame is an Ax25Frame_t, buffer has
    // buffer_len bytes (at most FRAME_MAX of them are used)
    let (Some(frame), Some(buf)) =
        (unsafe { ptr::in_(frame) }, unsafe { ptr::bytes_mut(buffer, buffer_len.min(FRAME_MAX)) })
    else {
        return 0;
    };
    bcn::serialize(frame, buf)
}

/// Transmit a beacon: never while TX-OFF is latched (47 CFR 97.207(b)) or
/// the EPS has the beacon powered off. The frame goes to the downlink
/// (`downlink.h`), which may still refuse it (an inhibit, the duty limit,
/// the transmitter's protections); only a frame it takes is counted and
/// shown on the console
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_transmit(content: *const Content) -> i32 {
    // SAFETY: the C contract: NULL or a BeaconContent_t
    let Some(content) = (unsafe { ptr::in_(content) }) else { return rc::NULL_PTR };
    if !transmit_permitted() {
        return rc::ERROR;
    }
    let (frame, ok) = bcn::build_frame(content);
    if !ok {
        return rc::ERROR;
    }
    let tx_len = bcn::serialized_len(&frame);
    let mut body = [0u8; downlink::MAX_FRAME_LEN];
    let body_len = bcn::serialize_body(&frame, &mut body);
    if tx_len == 0 || body_len == 0 {
        return rc::ERROR;
    }
    // SAFETY: body holds body_len bytes
    let sent = unsafe { crate::downlink::downlink_transmit(body.as_ptr(), body_len) };
    if sent != rc::OK {
        return sent;
    }
    if BEACON.with(|b| b.count_transmit(tx_len, content.source)).is_none() {
        return rc::BUSY;
    }
    c::console_parts(&[b"[BEACON] TX: ", frame.info.get(..frame.info_len).unwrap_or(&[])]);
    rc::OK
}

/// Record that a beacon was sent at `now_ms`
#[unsafe(no_mangle)]
pub extern "C" fn beacon_mark_transmitted(now_ms: u64) {
    let _ = BEACON.with(|b| {
        if b.initialized {
            b.state.last_beacon_ms = now_ms;
        }
    });
}

/// The module state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_get_state(state: *mut State) -> i32 {
    // SAFETY: the C contract: NULL or a BeaconState_t
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    match BEACON.with(|b| b.initialized.then_some(b.state)) {
        Some(Some(s)) => {
            *out = s;
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Set whether Jetson text is available
#[unsafe(no_mangle)]
pub extern "C" fn beacon_set_ai_available(available: bool) {
    let _ = BEACON.with(|b| {
        if b.initialized {
            b.state.ai_available = available;
        }
    });
}

/// Store Jetson text for the next beacon (once), if the filter passes it
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_store_ai_text(text: *const c_char, len: usize) -> i32 {
    // SAFETY: the C contract: NULL or len bytes
    let Some(text) = (unsafe { ptr::bytes(text.cast(), len) }) else { return rc::NULL_PTR };
    match BEACON.with(|b| b.initialized.then(|| b.store_ai_text(text))) {
        Some(Some(true)) => rc::OK,
        Some(_) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// The next template (in turn)
#[unsafe(no_mangle)]
pub extern "C" fn beacon_get_template() -> *const c_char {
    let index = BEACON.with(Beacon::next_template).unwrap_or(0);
    beacon_templates.get(index).or(beacon_templates.first()).map_or(core::ptr::null(), |t| t.0)
}

/// CRC-16 (the AX.25 FCS)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_crc16(data: *const u8, len: usize) -> u16 {
    // SAFETY: the C contract, passed on
    unsafe { ax25_ffi::ax25_fcs(data, len) }
}

/// Record the latest measurements
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_update_measurements(m: *const Measurements, sample_ms: u64) -> i32 {
    // SAFETY: the C contract: NULL or a BeaconMeasurements_t
    let Some(m) = (unsafe { ptr::in_(m) }) else { return rc::NULL_PTR };
    match BEACON.with(|b| b.update_measurements(m, sample_ms)) {
        Some(true) => rc::OK,
        Some(false) => rc::INVALID,
        None => rc::BUSY,
    }
}

/// Whether a text passes the content filter
#[unsafe(no_mangle)]
pub unsafe extern "C" fn beacon_validate_text(text: *const c_char, len: usize) -> bool {
    // SAFETY: the C contract, passed on
    unsafe { beacon_filter::beacon_filter_check(text, len) == filter::Verdict::Ok as u32 }
}

/// Beacons transmitted
#[unsafe(no_mangle)]
pub extern "C" fn beacon_get_total_count() -> u32 {
    BEACON.with(|b| b.total_count()).unwrap_or(0)
}

/// Percentage of beacons with Jetson text
#[unsafe(no_mangle)]
pub extern "C" fn beacon_get_ai_percentage() -> u8 {
    BEACON.with(|b| b.ai_percentage()).unwrap_or(0)
}

/// Clear the transmission statistics
#[unsafe(no_mangle)]
pub extern "C" fn beacon_reset_stats() {
    let _ = BEACON.with(Beacon::reset_stats);
}
