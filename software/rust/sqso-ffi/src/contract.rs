//! `rust_contract.h`: what this library assumes about the C headers, for
//! `tests/test_rust_contract.c` to compare

use core::ffi::c_char;
use core::mem::{align_of, size_of};

use sqso_fsw::{
    contact_monitor, deployment, fault_log, keys, mode, nvm, payload_guard, rf_control, rf_silence, scheduler,
};
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
        // 100-149: payload guard, deployment
        100 => layout::<crate::payload_guard::PayloadSample>(),
        101 => layout::<crate::deployment::DeploymentElementStatus>(),
        102 => layout::<crate::deployment::DeploymentTelemetry>(),
        // assert_handler.h, flight_log.h, uart_comm.h, hal_uart.h
        250 => layout::<sqso_fsw::assert_handler::Record>(),
        251 => layout::<sqso_fsw::assert_handler::Stats>(),
        252 => layout::<sqso_fsw::flight_log::Entry>(),
        253 => layout::<sqso_fsw::flight_log::Stats>(),
        254 => layout::<crate::uart_comm::UartConfig>(),
        255 => layout::<crate::uart_comm::HalUartConfig>(),
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
        // 100-149: RF control, RF silence, contact monitor, deployment,
        // payload guard, and the HAL values they use
        100 => payload_guard::MAX_INPUT_W,
        101 => payload_guard::LATCHUP_INPUT_W,
        102 => f64::from(payload_guard::OVERPOWER_SAMPLES),
        103 => payload_guard::SPREADER_MAX_C,
        104 => payload_guard::JUNCTION_MAX_C,
        105 => payload_guard::RESTART_MARGIN_C,
        106 => payload_guard::COOLDOWN_MS as f64,
        107 => payload_guard::TELEMETRY_TIMEOUT_MS as f64,
        108 => f64::from(payload_guard::Outcome::Cooldown as u32),
        109 => f64::from(rf_control::POWER_LEVEL_COUNT),
        110 => f64::from(rf_control::INHIBIT_CONTACT),
        111 => f64::from(rf_control::INHIBIT_RF_SILENCE),
        112 => f64::from(rf_silence::DURATION_S),
        113 => f64::from(rf_silence::PERSIST_INTERVAL_S),
        114 => f64::from(rf_silence::State::Armed as u32),
        115 => f64::from(rf_silence::State::Complete as u32),
        116 => f64::from(contact_monitor::TIMEOUT_DEFAULT_DAYS),
        117 => f64::from(contact_monitor::TIMEOUT_MIN_DAYS),
        118 => f64::from(contact_monitor::TIMEOUT_MAX_DAYS),
        119 => f64::from(contact_monitor::PERSIST_INTERVAL_S),
        120 => f64::from(contact_monitor::NVM_COUNTER),
        121 => f64::from(contact_monitor::NVM_TIMEOUT),
        122 => f64::from(deployment::WAIT_TIME_MS),
        123 => f64::from(deployment::ANTENNA_BURN_TIME_MS),
        124 => f64::from(deployment::SOLAR_BURN_TIME_MS),
        125 => f64::from(deployment::MAX_RETRIES),
        126 => f64::from(deployment::RETRY_DELAY_MS),
        127 => f64::from(deployment::PERSIST_INTERVAL_S),
        128 => f64::from(deployment::STATE_COUNT),
        129 => f64::from(deployment::State::Armed as u32),
        130 => f64::from(deployment::ELEMENT_COUNT),
        131 => f64::from(crate::rf_control::FLASH_REGION_RF_CONTROL),
        132 => f64::from(crate::contact_monitor::FLASH_REGION_CONTACT),
        133 => f64::from(crate::rf_silence::FLASH_REGION_RF_SILENCE),
        134 => f64::from(crate::deployment::FLASH_REGION_DEPLOYMENT),
        135 => f64::from(crate::rf_control::GPIO_PIN_RADIO_EN),
        136 => f64::from(crate::deployment::GPIO_PIN_DEPLOY_EN),
        137 => f64::from(crate::rf_control::GPIO_PIN_RF_TX_EN),
        138 => f64::from(crate::deployment::GPIO_PIN_ANTENNA_BURN),
        139 => f64::from(crate::deployment::GPIO_PIN_SOLAR_BURN),
        140 => f64::from(crate::deployment::GPIO_PIN_ANTENNA_SENSE),
        141 => f64::from(crate::deployment::GPIO_PIN_SOLAR_SENSE),
        142 => f64::from(crate::rf_control::GPIO_DIR_INPUT),
        143 => f64::from(crate::rf_control::GPIO_DIR_OUTPUT),
        144 => f64::from(crate::rf_control::GPIO_PULL_DOWN),
        145 => f64::from(rf_silence::HAL_RESET_WATCHDOG),
        146 => f64::from(rf_silence::HAL_RESET_SOFTWARE),
        147 => f64::from(crate::deployment::FAULT_TYPE_DEPLOYMENT),
        148 => f64::from(crate::deployment::LOG_LEVEL_DEBUG),
        149 => f64::from(crate::deployment::LOG_LEVEL_INFO),
        // assert_handler.h, flight_log.h, uart_comm.h, smart_qso.h
        250 => sqso_fsw::assert_handler::MAX_MSG_LEN as f64,
        251 => sqso_fsw::assert_handler::MAX_FAILURE_LOG as f64,
        252 => sqso_fsw::assert_handler::MAX_FILENAME_LEN as f64,
        253 => sqso_fsw::flight_log::MAX_MESSAGE_LEN as f64,
        254 => sqso_fsw::flight_log::BUFFER_SIZE as f64,
        255 => sqso_fsw::flight_log::MAX_MODULE_LEN as f64,
        256 => f64::from(sqso_fsw::flight_log::MIN_LEVEL),
        257 => f64::from(sqso_fsw::flight_log::level::OFF),
        258 => sqso_fsw::uart_comm::DEVICE_MAX_LEN as f64,
        259 => f64::from(sqso_fsw::uart_comm::BAUDRATE_DEFAULT),
        260 => f64::from(sqso_fsw::uart_comm::TELEMETRY_INTERVAL_DEFAULT_MS),
        261 => f64::from(crate::assert_handler::FAULT_TYPE_ASSERTION),
        262 => sqso_fsw::uart_comm::DEVICE_DEFAULT.len() as f64,
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
