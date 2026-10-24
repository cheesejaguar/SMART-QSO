//! `telemetry.h`: binary telemetry frames (`sqso_fsw::telemetry`) from the
//! system state.

use sqso_fsw::telemetry::{self as tlm, FrameType, Inputs, MAX_FRAME_SIZE, Stats, Telemetry};

use crate::c::{self, rc};
use crate::cell::Global;
use crate::{ptr, system_state};

/// `TlmFrame_t`: packed header, payload and CRC, as bytes
#[repr(C)]
pub struct TlmFrame(pub [u8; MAX_FRAME_SIZE]);

static TLM: Global<Telemetry> = Global::new(Telemetry::new());

/// The system state a frame reports
fn inputs() -> Inputs {
    let power = system_state::power_state();
    let thermal = system_state::thermal_state();
    let comm = system_state::comm_state();
    let adcs = system_state::adcs_state();
    Inputs {
        battery_voltage: power.battery_voltage,
        battery_current: power.battery_current,
        state_of_charge: power.state_of_charge,
        solar_power: power.solar_power,
        power_mode: power.power_mode,
        payload_enabled: power.payload_enabled,
        temps_c: [thermal.obc_temp_c, thermal.eps_temp_c, thermal.battery_temp_c, thermal.payload_temp_c],
        heater_enabled: thermal.heater_enabled,
        op_state: system_state::sys_get_operational_state(),
        thermal_fault: system_state::sys_has_thermal_fault(),
        boot_count: system_state::sys_get_boot_count(),
        uptime_s: system_state::sys_get_uptime_s(),
        packets_sent: comm.packets_sent,
        packets_received: comm.packets_received,
        beacon_count: comm.beacon_count,
        mag_ut: adcs.mag_ut,
        gyro_dps: adcs.gyro_dps,
        sun_vector: adcs.sun_vector,
        detumbled: adcs.detumbled,
        sun_acquired: adcs.sun_acquired,
    }
}

/// Write a frame: payload, header (next sequence number, uptime), CRC
///
/// # Safety
/// `frame` and `frame_len` are NULL or valid for writes (the C contract).
unsafe fn generate(frame: *mut TlmFrame, frame_len: *mut usize, frame_type: FrameType, payload: &[u8]) -> i32 {
    if frame.is_null() || frame_len.is_null() {
        return rc::NULL_PTR;
    }
    let Some(sequence) = TLM.with(Telemetry::next_sequence) else { return rc::BUSY };
    let uptime = system_state::sys_get_uptime_s();
    let header = tlm::header(frame_type, sequence, uptime, u16::try_from(payload.len()).unwrap_or(u16::MAX));
    let crc = tlm::frame_crc(&header, payload);
    // SAFETY: the C contract: frame is a TlmFrame_t (256 bytes); only the
    // header, payload and CRC fields are written
    if let Some(bytes) = unsafe { ptr::bytes_mut(frame.cast(), MAX_FRAME_SIZE) } {
        let fields = [(0, &header[..]), (tlm::HEADER_SIZE, payload), (tlm::CRC_OFFSET, &crc.to_ne_bytes()[..])];
        for (offset, data) in fields {
            if let Some(dst) = bytes.get_mut(offset..offset + data.len()) {
                dst.copy_from_slice(data);
            }
        }
    }
    // SAFETY: checked non-NULL above
    if let Some(len) = unsafe { ptr::out(frame_len) } {
        *len = tlm::HEADER_SIZE + payload.len() + 4;
    }
    let _ = TLM.with(Telemetry::count_frame);
    rc::OK
}

/// Initialize: statistics zero, default rate, a frame due at once
#[unsafe(no_mangle)]
pub extern "C" fn tlm_init() -> i32 {
    TLM.with(Telemetry::init).map_or(rc::BUSY, |()| rc::OK)
}

/// Set the rate (held to 10-300 s)
#[unsafe(no_mangle)]
pub extern "C" fn tlm_set_rate(rate_ms: u32) -> i32 {
    TLM.with(|t| t.set_rate(rate_ms)).map_or(rc::BUSY, |()| rc::OK)
}

/// The rate, ms
#[unsafe(no_mangle)]
pub extern "C" fn tlm_get_rate() -> u32 {
    TLM.with(|t| t.rate_ms).unwrap_or(tlm::DEFAULT_RATE_MS)
}

/// Housekeeping frame
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlm_generate_housekeeping(frame: *mut TlmFrame, frame_len: *mut usize) -> i32 {
    if frame.is_null() || frame_len.is_null() {
        return rc::NULL_PTR;
    }
    let payload = tlm::housekeeping(&inputs());
    // SAFETY: the C contract
    unsafe { generate(frame, frame_len, FrameType::Housekeeping, &payload) }
}

/// EPS frame
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlm_generate_eps(frame: *mut TlmFrame, frame_len: *mut usize) -> i32 {
    if frame.is_null() || frame_len.is_null() {
        return rc::NULL_PTR;
    }
    let payload = tlm::eps(&inputs());
    // SAFETY: the C contract
    unsafe { generate(frame, frame_len, FrameType::Eps, &payload) }
}

/// ADCS frame
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlm_generate_adcs(frame: *mut TlmFrame, frame_len: *mut usize) -> i32 {
    if frame.is_null() || frame_len.is_null() {
        return rc::NULL_PTR;
    }
    let payload = tlm::adcs(&inputs());
    // SAFETY: the C contract
    unsafe { generate(frame, frame_len, FrameType::Adcs, &payload) }
}

/// Status beacon frame (counted as a beacon in the system state)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlm_generate_beacon(frame: *mut TlmFrame, frame_len: *mut usize) -> i32 {
    if frame.is_null() || frame_len.is_null() {
        return rc::NULL_PTR;
    }
    let payload = tlm::beacon(&inputs());
    // SAFETY: the C contract
    let result = unsafe { generate(frame, frame_len, FrameType::Beacon, &payload) };
    if result == rc::OK {
        let _ = system_state::sys_increment_beacon_count();
    }
    result
}

/// Whether a frame is due (never before `tlm_init()`)
#[unsafe(no_mangle)]
pub extern "C" fn tlm_is_due() -> bool {
    let now = c::now_ms();
    TLM.with(|t| t.is_due(now)).unwrap_or(false)
}

/// Record a transmission attempt (a success counts a packet sent)
#[unsafe(no_mangle)]
pub extern "C" fn tlm_mark_transmitted(success: bool) -> i32 {
    let now = c::now_ms();
    match TLM.with(|t| t.mark_transmitted(now, success)) {
        Some(true) => {
            let _ = system_state::sys_increment_packets_sent();
            rc::OK
        }
        Some(false) => rc::OK,
        None => rc::BUSY,
    }
}

/// The statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlm_get_stats(stats: *mut Stats) -> i32 {
    // SAFETY: the C contract: NULL or a TlmStats_t (same layout)
    let Some(out) = (unsafe { ptr::out(stats) }) else { return rc::NULL_PTR };
    match TLM.with(|t| t.stats) {
        Some(s) => {
            *out = s;
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Header, `payload_len` payload bytes and the CRC (big-endian) into
/// `buffer`; NO_MEM if the payload is beyond the frame's or the buffer is
/// too small (nothing written)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tlm_serialize(
    frame: *const TlmFrame,
    payload_len: usize,
    buffer: *mut u8,
    buffer_size: usize,
    bytes_written: *mut usize,
) -> i32 {
    if frame.is_null() || buffer.is_null() || bytes_written.is_null() {
        return rc::NULL_PTR;
    }
    if payload_len > tlm::PAYLOAD_MAX || buffer_size < tlm::HEADER_SIZE + payload_len + 4 {
        return rc::NO_MEM;
    }
    let total = tlm::HEADER_SIZE + payload_len + 4;
    // SAFETY: the C contract: frame is a TlmFrame_t, buffer has
    // buffer_size >= total bytes
    let (Some(frame), Some(out)) = (unsafe { ptr::in_(frame) }, unsafe { ptr::bytes_mut(buffer, total) }) else {
        return rc::NULL_PTR;
    };
    match tlm::serialize(&frame.0, payload_len, out) {
        Ok(n) => {
            // SAFETY: checked non-NULL above
            if let Some(w) = unsafe { ptr::out(bytes_written) } {
                *w = n;
            }
            rc::OK
        }
        Err(tlm::NoRoom) => rc::NO_MEM,
    }
}
