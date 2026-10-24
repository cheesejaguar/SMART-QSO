//! `rust_contract.h`: what this library assumes about the C headers, for
//! `tests/test_rust_contract.c` to compare

use core::ffi::c_char;
use core::mem::{align_of, size_of};

use sqso_fsw::{eps, fault_log, gnss, keys, mission_data, mode, nvm, scheduler, sensors};
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
        // eps_control.h, sensors.h, mission_data.h, gnss.h
        150 => layout::<eps::EpsState>(),
        151 => layout::<crate::sensors::Sensor>(),
        152 => layout::<mission_data::MissionData>(),
        153 => layout::<gnss::Fix>(),
        154 => layout::<gnss::Receiver>(),
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
        // eps_control.h, sensors.h, mission_data.h, gnss.h
        150 => eps::POWER_LIMIT_SAFE_W,
        151 => eps::POWER_LIMIT_IDLE_W,
        152 => eps::POWER_LIMIT_ACTIVE_W,
        153 => eps::BUS_MIN_V,
        154 => eps::BUS_MAX_V,
        155 => sensors::MAX_SENSORS as f64,
        156 => sensors::ID_LEN as f64,
        157 => sensors::NAME_LEN as f64,
        158 => sensors::UNITS_LEN as f64,
        159 => sensors::TYPE_LEN as f64,
        160 => sensors::CHANNEL_LEN as f64,
        161 => mission_data::FAULT_DESC_LEN as f64,
        162 => f64::from(mission_data::Phase::Eol as u32),
        163 => gnss::LINE_MAX as f64,
        164 => f64::from(gnss::GGA_MAX_AGE_MS),
        165 => gnss::LEO_MIN_RADIUS_M,
        166 => gnss::LEO_MAX_RADIUS_M,
        167 => f64::from(gnss::MIN_SATS),
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
