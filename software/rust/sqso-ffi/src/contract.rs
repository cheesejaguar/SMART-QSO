//! `rust_contract.h`: what this library assumes about the C headers, for
//! `tests/test_rust_contract.c` to compare

use core::ffi::c_char;
use core::mem::{align_of, size_of};

use sqso_fsw::telemetry as tlm;
use sqso_fsw::{
    adcs, beacon, contact_monitor, deployment, eps, fault_log, gnss, keys, mission_data, mode, nvm, payload_guard,
    rf_control, rf_silence, scheduler, sensors, watchdog,
};
use sqso_protocol::{auth, ax25, filter, gse, jetson, station};

use crate::ptr;

const fn layout<T>() -> (usize, usize) {
    (size_of::<T>(), align_of::<T>())
}

/// Size and alignment of each mirrored type, by `RustLayout_t` id
const LAYOUTS: &[(u32, (usize, usize))] = &[
    (0, layout::<crate::ax25::Ax25UiView>()),
    (1, layout::<crate::jetson_link::JetsonMsg>()),
    (2, layout::<crate::jetson_link::JetsonRx>()),
    (3, layout::<crate::input_validation::CommandValidation>()),
    (4, layout::<crate::input_validation::Vector3>()),
    (5, layout::<crate::cmd_dictionary::CmdDictEntry>()),
    (6, layout::<crate::cmd_auth::CmdAuthVerified>()),
    (7, layout::<crate::cmd_auth::CmdAuthInfo>()),
    (8, layout::<crate::cmd_handler::CmdCommand>()),
    (9, layout::<crate::cmd_handler::CmdResponse>()),
    (10, layout::<crate::cmd_handler::CmdStats>()),
    (11, layout::<crate::uplink::UplinkStats>()),
    (12, layout::<crate::sha256::Sha256Ctx>()),
    (13, layout::<crate::c::PowerState>()),
    (14, layout::<crate::c::AdcsState>()),
    (15, layout::<crate::c::SmContext>()),
    (16, layout::<crate::state_machine::SmTransition>()),
    (17, layout::<crate::scheduler::CTaskConfig>()),
    (18, layout::<crate::scheduler::CTaskStats>()),
    (19, layout::<crate::scheduler::CSchedStats>()),
    (20, layout::<fault_log::Entry>()),
    // 100-149: payload guard, deployment
    (100, layout::<crate::payload_guard::PayloadSample>()),
    (101, layout::<crate::deployment::DeploymentElementStatus>()),
    (102, layout::<crate::deployment::DeploymentTelemetry>()),
    // assert_handler.h, flight_log.h, uart_comm.h, hal_uart.h
    (250, layout::<sqso_fsw::assert_handler::Record>()),
    (251, layout::<sqso_fsw::assert_handler::Stats>()),
    (252, layout::<sqso_fsw::flight_log::Entry>()),
    (253, layout::<sqso_fsw::flight_log::Stats>()),
    (254, layout::<crate::uart_comm::UartConfig>()),
    (255, layout::<crate::uart_comm::HalUartConfig>()),
    // eps_control.h, sensors.h, mission_data.h, gnss.h
    (150, layout::<eps::EpsState>()),
    (151, layout::<crate::sensors::Sensor>()),
    (152, layout::<mission_data::MissionData>()),
    (153, layout::<gnss::Fix>()),
    (154, layout::<gnss::Receiver>()),
    // system_state.h, telemetry.h, beacon.h, adcs_control.h, watchdog_mgr.h
    (200, layout::<sqso_fsw::system_state::Thermal>()),
    (201, layout::<sqso_fsw::system_state::Comm>()),
    (202, layout::<sqso_fsw::system_state::Mission>()),
    (203, layout::<sqso_fsw::system_state::Image>()),
    (204, layout::<crate::telemetry::TlmFrame>()),
    (205, layout::<sqso_fsw::telemetry::Stats>()),
    (206, layout::<sqso_fsw::beacon::Telemetry>()),
    (207, layout::<sqso_fsw::beacon::Measurements>()),
    (208, layout::<sqso_fsw::beacon::Content>()),
    (209, layout::<sqso_fsw::beacon::Frame>()),
    (210, layout::<sqso_fsw::beacon::State>()),
    (211, layout::<crate::beacon::EpsControlState>()),
    (212, layout::<sqso_fsw::watchdog::TaskEntry>()),
    (213, layout::<sqso_fsw::watchdog::Stats>()),
    (214, layout::<sqso_fsw::adcs::Vec3>()),
    (215, layout::<sqso_fsw::adcs::MagData>()),
    (216, layout::<sqso_fsw::adcs::SunSensorData>()),
    (217, layout::<sqso_fsw::adcs::MtqCommand>()),
    (218, layout::<sqso_fsw::adcs::State>()),
    (219, layout::<sqso_fsw::adcs::Telemetry>()),
];

/// Size and alignment of the mirror of a C type
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sqso_rust_layout(which: u32, size: *mut usize, align: *mut usize) -> bool {
    let Some(&(_, found)) = LAYOUTS.iter().find(|(id, _)| *id == which) else { return false };
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

/// Each kept constant, by `RustConst_t` id
#[allow(clippy::cast_precision_loss, clippy::cast_lossless)] // small integers
const CONSTANTS: &[(u32, f64)] = &[
    (0, mode::SOC_PAYLOAD),
    (1, mode::SOC_SAFE),
    (2, mode::SOC_IDLE),
    (3, mode::SOC_EMERGENCY),
    (4, jetson::LINE_MAX as f64),
    (5, jetson::TEXT_MAX as f64),
    (6, filter::MAX_LEN as f64),
    (7, filter::MAX_WORD as f64),
    (8, auth::KEY_SIZE as f64),
    (9, auth::TAG_SIZE as f64),
    (10, sqso_protocol::MAX_ARGS as f64),
    (11, (keys::OP_SLOTS) as f64),
    (12, nvm::SLOT_SIZE as f64),
    (13, (nvm::FOOTPRINT) as f64),
    (14, gse::FRAME_SIZE as f64),
    (15, ax25::MAX_FRAME_LEN as f64),
    (16, fault_log::MAX_ENTRIES as f64),
    (17, fault_log::DESC_LEN as f64),
    (18, scheduler::MAX_TASKS as f64),
    (19, scheduler::NAME_LEN as f64),
    (20, mode::State::ALL.len() as f64),
    (21, mode::Event::ALL.len() as f64),
    (22, (station::SSID) as f64),
    // 100-149: RF control, RF silence, contact monitor, deployment,
    // payload guard, and the HAL values they use
    (100, payload_guard::MAX_INPUT_W),
    (101, payload_guard::LATCHUP_INPUT_W),
    (102, (payload_guard::OVERPOWER_SAMPLES) as f64),
    (103, payload_guard::SPREADER_MAX_C),
    (104, payload_guard::JUNCTION_MAX_C),
    (105, payload_guard::RESTART_MARGIN_C),
    (106, payload_guard::COOLDOWN_MS as f64),
    (107, payload_guard::TELEMETRY_TIMEOUT_MS as f64),
    (108, (payload_guard::Outcome::Cooldown as u32) as f64),
    (109, (rf_control::POWER_LEVEL_COUNT) as f64),
    (110, (rf_control::INHIBIT_CONTACT) as f64),
    (111, (rf_control::INHIBIT_RF_SILENCE) as f64),
    (112, (rf_silence::DURATION_S) as f64),
    (113, (rf_silence::PERSIST_INTERVAL_S) as f64),
    (114, (rf_silence::State::Armed as u32) as f64),
    (115, (rf_silence::State::Complete as u32) as f64),
    (116, (contact_monitor::TIMEOUT_DEFAULT_DAYS) as f64),
    (117, (contact_monitor::TIMEOUT_MIN_DAYS) as f64),
    (118, (contact_monitor::TIMEOUT_MAX_DAYS) as f64),
    (119, (contact_monitor::PERSIST_INTERVAL_S) as f64),
    (120, (contact_monitor::NVM_COUNTER) as f64),
    (121, (contact_monitor::NVM_TIMEOUT) as f64),
    (122, (deployment::WAIT_TIME_MS) as f64),
    (123, (deployment::ANTENNA_BURN_TIME_MS) as f64),
    (124, (deployment::SOLAR_BURN_TIME_MS) as f64),
    (125, (deployment::MAX_RETRIES) as f64),
    (126, (deployment::RETRY_DELAY_MS) as f64),
    (127, (deployment::PERSIST_INTERVAL_S) as f64),
    (128, (deployment::STATE_COUNT) as f64),
    (129, (deployment::State::Armed as u32) as f64),
    (130, (deployment::ELEMENT_COUNT) as f64),
    (131, (crate::rf_control::FLASH_REGION_RF_CONTROL) as f64),
    (132, (crate::contact_monitor::FLASH_REGION_CONTACT) as f64),
    (133, (crate::rf_silence::FLASH_REGION_RF_SILENCE) as f64),
    (134, (crate::deployment::FLASH_REGION_DEPLOYMENT) as f64),
    (135, (crate::rf_control::GPIO_PIN_RADIO_EN) as f64),
    (136, (crate::deployment::GPIO_PIN_DEPLOY_EN) as f64),
    (137, (crate::rf_control::GPIO_PIN_RF_TX_EN) as f64),
    (138, (crate::deployment::GPIO_PIN_ANTENNA_BURN) as f64),
    (139, (crate::deployment::GPIO_PIN_SOLAR_BURN) as f64),
    (140, (crate::deployment::GPIO_PIN_ANTENNA_SENSE) as f64),
    (141, (crate::deployment::GPIO_PIN_SOLAR_SENSE) as f64),
    (142, (crate::rf_control::GPIO_DIR_INPUT) as f64),
    (143, (crate::rf_control::GPIO_DIR_OUTPUT) as f64),
    (144, (crate::rf_control::GPIO_PULL_DOWN) as f64),
    (145, (rf_silence::HAL_RESET_WATCHDOG) as f64),
    (146, (rf_silence::HAL_RESET_SOFTWARE) as f64),
    (147, (crate::deployment::FAULT_TYPE_DEPLOYMENT) as f64),
    (148, (crate::deployment::LOG_LEVEL_DEBUG) as f64),
    (149, (crate::deployment::LOG_LEVEL_INFO) as f64),
    // assert_handler.h, flight_log.h, uart_comm.h, smart_qso.h
    (250, sqso_fsw::assert_handler::MAX_MSG_LEN as f64),
    (251, sqso_fsw::assert_handler::MAX_FAILURE_LOG as f64),
    (252, sqso_fsw::assert_handler::MAX_FILENAME_LEN as f64),
    (253, sqso_fsw::flight_log::MAX_MESSAGE_LEN as f64),
    (254, sqso_fsw::flight_log::BUFFER_SIZE as f64),
    (255, sqso_fsw::flight_log::MAX_MODULE_LEN as f64),
    (256, (sqso_fsw::flight_log::MIN_LEVEL) as f64),
    (257, (sqso_fsw::flight_log::level::OFF) as f64),
    (258, sqso_fsw::uart_comm::DEVICE_MAX_LEN as f64),
    (259, (sqso_fsw::uart_comm::BAUDRATE_DEFAULT) as f64),
    (260, (sqso_fsw::uart_comm::TELEMETRY_INTERVAL_DEFAULT_MS) as f64),
    (261, (crate::assert_handler::FAULT_TYPE_ASSERTION) as f64),
    (262, sqso_fsw::uart_comm::DEVICE_DEFAULT.len() as f64),
    // eps_control.h, sensors.h, mission_data.h, gnss.h
    (150, eps::POWER_LIMIT_SAFE_W),
    (151, eps::POWER_LIMIT_IDLE_W),
    (152, eps::POWER_LIMIT_ACTIVE_W),
    (153, eps::BUS_MIN_V),
    (154, eps::BUS_MAX_V),
    (155, sensors::MAX_SENSORS as f64),
    (156, sensors::ID_LEN as f64),
    (157, sensors::NAME_LEN as f64),
    (158, sensors::UNITS_LEN as f64),
    (159, sensors::TYPE_LEN as f64),
    (160, sensors::CHANNEL_LEN as f64),
    (161, mission_data::FAULT_DESC_LEN as f64),
    (162, (mission_data::Phase::Eol as u32) as f64),
    (163, gnss::LINE_MAX as f64),
    (164, (gnss::GGA_MAX_AGE_MS) as f64),
    (165, gnss::LEO_MIN_RADIUS_M),
    (166, gnss::LEO_MAX_RADIUS_M),
    (167, (gnss::MIN_SATS) as f64),
    // system_state.h, telemetry.h, beacon.h, adcs_control.h, watchdog_mgr.h
    (200, tlm::MAX_FRAME_SIZE as f64),
    (201, (tlm::SYNC_WORD) as f64),
    (202, (tlm::DEFAULT_RATE_MS) as f64),
    (203, (tlm::MIN_RATE_MS) as f64),
    (204, (tlm::MAX_RATE_MS) as f64),
    (205, tlm::HEADER_SIZE as f64),
    (206, tlm::HOUSEKEEPING_SIZE as f64),
    (207, tlm::EPS_SIZE as f64),
    (208, tlm::ADCS_SIZE as f64),
    (209, beacon::MAX_TEXT_LEN as f64),
    (210, beacon::MAX_PAYLOAD_LEN as f64),
    (211, (beacon::INTERVAL_ACTIVE_MS) as f64),
    (212, (beacon::INTERVAL_IDLE_MS) as f64),
    (213, (beacon::INTERVAL_SAFE_MS) as f64),
    (214, beacon::MEAS_MAX_AGE_MS as f64),
    (215, (beacon::TLM_SOC) as f64),
    (216, (beacon::TLM_BV) as f64),
    (217, (beacon::TLM_SUN) as f64),
    (218, (beacon::TLM_RF) as f64),
    (219, (beacon::TLM_QSO) as f64),
    (220, watchdog::MAX_TASKS as f64),
    (221, watchdog::NAME_LEN as f64),
    (222, (watchdog::HW_TIMEOUT_MS) as f64),
    (223, watchdog::TASK_CHECK_INTERVAL_MS as f64),
    (224, (watchdog::MISS_THRESHOLD) as f64),
    (225, adcs::MAX_DIPOLE),
    (226, adcs::BDOT_GAIN),
    (227, adcs::SUNPOINT_KP),
    (228, adcs::DETUMBLE_RATE_THRESHOLD),
    (229, adcs::DETUMBLE_SETTLING_MS as f64),
    (230, adcs::DETUMBLE_TIMEOUT_MS as f64),
    (231, adcs::SUN_SENSOR_COUNT as f64),
    (232, (adcs::mode::SAFE) as f64),
    (233, (crate::adcs_control::FAULT_TYPE_ADCS) as f64),
];

/// A constant this library keeps
#[unsafe(no_mangle)]
pub extern "C" fn sqso_rust_constant(which: u32) -> f64 {
    CONSTANTS.iter().find(|(id, _)| *id == which).map_or(f64::NAN, |&(_, v)| v)
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
