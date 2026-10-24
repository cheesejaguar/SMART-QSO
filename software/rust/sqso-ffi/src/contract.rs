//! `rust_contract.h`: what this library assumes about the C headers, for
//! `tests/test_rust_contract.c` to compare

use core::ffi::c_char;
use core::mem::{align_of, size_of};

use sqso_fsw::telemetry as tlm;
use sqso_fsw::{adcs, beacon, fault_log, keys, mode, nvm, scheduler, watchdog};
use sqso_protocol::{auth, ax25, filter, gse, jetson, station};

use crate::ptr;

fn layout<T>() -> (usize, usize) {
    (size_of::<T>(), align_of::<T>())
}

/// Size and alignment of the mirror of a C type
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqso_rust_layout(which: u32, size: *mut usize, align: *mut usize) -> bool {
    let found = match which {
        0 => layout::<crate::ax25::Ax25UiView>(),
        1 => layout::<crate::jetson_link::JetsonMsg>(),
        2 => layout::<crate::jetson_link::JetsonRx>(),
        3 => layout::<crate::input_validation::CommandValidation>(),
        4 => layout::<crate::input_validation::Vector3>(),
        5 => layout::<crate::cmd_dictionary::CmdDictEntry>(),
        6 => layout::<crate::cmd_auth::CmdAuthVerified>(),
        7 => layout::<crate::cmd_auth::CmdAuthInfo>(),
        8 => layout::<crate::cmd_handler::CmdCommand>(),
        9 => layout::<crate::cmd_handler::CmdResponse>(),
        10 => layout::<crate::cmd_handler::CmdStats>(),
        11 => layout::<crate::uplink::UplinkStats>(),
        12 => layout::<crate::sha256::Sha256Ctx>(),
        13 => layout::<crate::c::PowerState>(),
        14 => layout::<crate::c::AdcsState>(),
        15 => layout::<crate::c::SmContext>(),
        16 => layout::<crate::state_machine::SmTransition>(),
        17 => layout::<crate::scheduler::CTaskConfig>(),
        18 => layout::<crate::scheduler::CTaskStats>(),
        19 => layout::<crate::scheduler::CSchedStats>(),
        20 => layout::<fault_log::Entry>(),
        // system_state.h, telemetry.h, beacon.h, adcs_control.h, watchdog_mgr.h
        200 => layout::<sqso_fsw::system_state::Thermal>(),
        201 => layout::<sqso_fsw::system_state::Comm>(),
        202 => layout::<sqso_fsw::system_state::Mission>(),
        203 => layout::<sqso_fsw::system_state::Image>(),
        204 => layout::<crate::telemetry::TlmFrame>(),
        205 => layout::<sqso_fsw::telemetry::Stats>(),
        206 => layout::<sqso_fsw::beacon::Telemetry>(),
        207 => layout::<sqso_fsw::beacon::Measurements>(),
        208 => layout::<sqso_fsw::beacon::Content>(),
        209 => layout::<sqso_fsw::beacon::Frame>(),
        210 => layout::<sqso_fsw::beacon::State>(),
        211 => layout::<crate::beacon::EpsControlState>(),
        212 => layout::<sqso_fsw::watchdog::TaskEntry>(),
        213 => layout::<sqso_fsw::watchdog::Stats>(),
        214 => layout::<sqso_fsw::adcs::Vec3>(),
        215 => layout::<sqso_fsw::adcs::MagData>(),
        216 => layout::<sqso_fsw::adcs::SunSensorData>(),
        217 => layout::<sqso_fsw::adcs::MtqCommand>(),
        218 => layout::<sqso_fsw::adcs::State>(),
        219 => layout::<sqso_fsw::adcs::Telemetry>(),
        _ => return false,
    };
    // SAFETY: the C contract: NULL or pointers to size_t
    if let Some(out) = unsafe { ptr::out(size) } {
        *out = found.0;
    }
    // SAFETY: as above
    if let Some(out) = unsafe { ptr::out(align) } {
        *out = found.1;
    }
    true
}

/// A constant this library keeps
#[unsafe(no_mangle)]
#[allow(clippy::cast_precision_loss)] // small integers
pub extern "C" fn sqso_rust_constant(which: u32) -> f64 {
    match which {
        0 => mode::SOC_PAYLOAD,
        1 => mode::SOC_SAFE,
        2 => mode::SOC_IDLE,
        3 => mode::SOC_EMERGENCY,
        4 => jetson::LINE_MAX as f64,
        5 => jetson::TEXT_MAX as f64,
        6 => filter::MAX_LEN as f64,
        7 => filter::MAX_WORD as f64,
        8 => auth::KEY_SIZE as f64,
        9 => auth::TAG_SIZE as f64,
        10 => sqso_protocol::MAX_ARGS as f64,
        11 => f64::from(keys::OP_SLOTS),
        12 => nvm::SLOT_SIZE as f64,
        13 => f64::from(nvm::FOOTPRINT),
        14 => gse::FRAME_SIZE as f64,
        15 => ax25::MAX_FRAME_LEN as f64,
        16 => fault_log::MAX_ENTRIES as f64,
        17 => fault_log::DESC_LEN as f64,
        18 => scheduler::MAX_TASKS as f64,
        19 => scheduler::NAME_LEN as f64,
        20 => mode::State::ALL.len() as f64,
        21 => mode::Event::ALL.len() as f64,
        22 => f64::from(station::SSID),
        // system_state.h, telemetry.h, beacon.h, adcs_control.h, watchdog_mgr.h
        200 => tlm::MAX_FRAME_SIZE as f64,
        201 => f64::from(tlm::SYNC_WORD),
        202 => f64::from(tlm::DEFAULT_RATE_MS),
        203 => f64::from(tlm::MIN_RATE_MS),
        204 => f64::from(tlm::MAX_RATE_MS),
        205 => tlm::HEADER_SIZE as f64,
        206 => tlm::HOUSEKEEPING_SIZE as f64,
        207 => tlm::EPS_SIZE as f64,
        208 => tlm::ADCS_SIZE as f64,
        209 => beacon::MAX_TEXT_LEN as f64,
        210 => beacon::MAX_PAYLOAD_LEN as f64,
        211 => f64::from(beacon::INTERVAL_ACTIVE_MS),
        212 => f64::from(beacon::INTERVAL_IDLE_MS),
        213 => f64::from(beacon::INTERVAL_SAFE_MS),
        214 => beacon::MEAS_MAX_AGE_MS as f64,
        215 => f64::from(beacon::TLM_SOC),
        216 => f64::from(beacon::TLM_BV),
        217 => f64::from(beacon::TLM_SUN),
        218 => f64::from(beacon::TLM_RF),
        219 => f64::from(beacon::TLM_QSO),
        220 => watchdog::MAX_TASKS as f64,
        221 => watchdog::NAME_LEN as f64,
        222 => f64::from(watchdog::HW_TIMEOUT_MS),
        223 => watchdog::TASK_CHECK_INTERVAL_MS as f64,
        224 => f64::from(watchdog::MISS_THRESHOLD),
        225 => adcs::MAX_DIPOLE,
        226 => adcs::BDOT_GAIN,
        227 => adcs::SUNPOINT_KP,
        228 => adcs::DETUMBLE_RATE_THRESHOLD,
        229 => adcs::DETUMBLE_SETTLING_MS as f64,
        230 => adcs::DETUMBLE_TIMEOUT_MS as f64,
        231 => adcs::SUN_SENSOR_COUNT as f64,
        232 => f64::from(adcs::mode::SAFE),
        233 => f64::from(crate::adcs_control::FAULT_TYPE_ADCS),
        _ => f64::NAN,
    }
}

/// The station call sign
#[unsafe(no_mangle)]
pub extern "C" fn sqso_rust_callsign() -> *const c_char {
    c"KK6GDB".as_ptr()
}

const _: () = {
    let a = c"KK6GDB".to_bytes();
    let b = station::CALLSIGN.as_bytes();
    assert!(a.len() == b.len());
    let mut i = 0;
    while i < a.len() {
        #[allow(clippy::indexing_slicing)] // i < len, at compile time
        {
            assert!(a[i] == b[i]);
        }
        i += 1;
    }
};
