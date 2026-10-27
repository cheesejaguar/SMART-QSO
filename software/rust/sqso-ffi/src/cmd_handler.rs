//! `cmd_handler.h`: authenticated telecommand processing and dispatch.
//!
//! [`cmd_process`] admits a frame through `sqso_fsw::telecommand::admit`
//! (framing, tag, counter, dictionary, counter spent), then
//! [`cmd_execute`] runs it. Dispatch is one exhaustive `match` on the
//! generated `Command` enum: a command added to command_dictionary.json
//! does not build until it has a handler here (SW-04).

use core::ffi::{CStr, c_char};
use core::sync::atomic::{AtomicU32, Ordering};

use sqso_fsw::ack::{Ack, Queue};

use sqso_fsw::mode::{Event, State};
use sqso_fsw::telecommand::{self, Admission, Rejection};
use sqso_protocol::{Command, MAX_ARGS};

use crate::c::{self, AdcsState, HalFlash, PowerState, fault_type, rc, severity};
use crate::cell::Global;
use crate::state_machine::{self, sm_result};
use crate::{cmd_auth, ptr};

/// `CmdResult_t`
mod result {
    pub(crate) const SUCCESS: u32 = 0;
    pub(crate) const INVALID_CMD: u32 = 1;
    pub(crate) const INVALID_PARAM: u32 = 2;
    pub(crate) const AUTH_FAIL: u32 = 3;
    pub(crate) const NOT_ALLOWED: u32 = 4;
    pub(crate) const EXEC_FAIL: u32 = 5;
}

const MAX_CUSTOM_HANDLERS: usize = 16;

/// `Command_t`
#[repr(C)]
pub struct CmdCommand {
    cmd_id: u8,
    sequence_num: u8,
    payload_len: u8,
    payload: [u8; MAX_ARGS],
    timestamp_ms: u32,
    requires_ack: bool,
}

/// `CmdResponse_t`
#[repr(C)]
pub struct CmdResponse {
    cmd_id: u8,
    sequence_num: u8,
    result: u32,
    data_len: u8,
    data: [u8; 64],
}

impl CmdResponse {
    pub(crate) const fn zeroed() -> Self {
        CmdResponse { cmd_id: 0, sequence_num: 0, result: result::SUCCESS, data_len: 0, data: [0; 64] }
    }
}

/// `CmdStats_t`
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CmdStats {
    /// Commands received
    pub total_received: u32,
    /// Commands executed
    pub total_executed: u32,
    /// Commands refused
    pub total_rejected: u32,
    /// Commands that failed authentication
    pub auth_failures: u32,
    /// When the last command arrived, ms (low 32 bits)
    pub last_cmd_time_ms: u32,
    /// The last command's id
    pub last_cmd_id: u8,
    /// The last command's result
    pub last_result: u32,
}

const NO_STATS: CmdStats = CmdStats {
    total_received: 0,
    total_executed: 0,
    total_rejected: 0,
    auth_failures: 0,
    last_cmd_time_ms: 0,
    last_cmd_id: 0,
    last_result: 0,
};

/// `CmdHandlerFunc_t`
pub type CmdHandlerFunc = unsafe extern "C" fn(cmd: *const CmdCommand, response: *mut CmdResponse) -> u32;

struct Handler {
    stats: CmdStats,
    /// A rejection has been logged since the last accepted command; later
    /// rejections are only counted, so forged frames cannot drive fault-log
    /// flash writes
    reject_logged: bool,
    custom: [Option<(u8, CmdHandlerFunc)>; MAX_CUSTOM_HANDLERS],
    initialized: bool,
}

static HANDLER: Global<Handler> = Global::new(Handler {
    stats: NO_STATS,
    reject_logged: false,
    custom: [None; MAX_CUSTOM_HANDLERS],
    initialized: false,
});

/// Run `f` on the handler state; if re-entered, `R::default()` (no
/// result codes go through here: those use `HANDLER.with` directly)
fn with<R: Default>(f: impl FnOnce(&mut Handler) -> R) -> R {
    HANDLER.with(f).unwrap_or_default()
}

fn custom_handler(cmd_id: u8) -> Option<CmdHandlerFunc> {
    with(|h| h.custom.iter().flatten().find(|(id, _)| *id == cmd_id).map(|(_, f)| *f))
}

/// Initialize the handler and load the telecommand key table
#[unsafe(no_mangle)]
pub extern "C" fn cmd_handler_init() -> i32 {
    with(|h| {
        h.stats = NO_STATS;
        h.custom = [None; MAX_CUSTOM_HANDLERS];
        h.reject_logged = false;
    });
    CONTACTS.store(0, Ordering::Relaxed);
    let _ = ACKS.with(|q| *q = Queue::new());
    // Unprovisioned: fail closed
    let _ = cmd_auth::cmd_auth_init();
    with(|h| h.initialized = true);
    rc::OK
}

/// Record a rejected command; only the first of a run is logged
fn reject(response: &mut CmdResponse, result: u32, reason: &CStr) {
    response.result = result;
    let first = with(|h| {
        h.stats.total_rejected = h.stats.total_rejected.wrapping_add(1);
        h.stats.last_result = result;
        !core::mem::replace(&mut h.reject_logged, true)
    });
    if first {
        c::log(fault_type::COMMAND, severity::WARNING, reason, 0.0);
    }
}

/// Authenticated commands counted as contact since boot (HK1 `CN`)
static CONTACTS: AtomicU32 = AtomicU32::new(0);

/// Acknowledgements waiting for the downlink (GND-03)
static ACKS: Global<Queue> = Global::new(Queue::new());

fn contact() {
    // SAFETY: no arguments
    unsafe { c::contact_monitor_contact() };
    CONTACTS.fetch_add(1, Ordering::Relaxed);
}

/// Authenticated commands counted as contact since boot
pub(crate) fn contacts() -> u32 {
    CONTACTS.load(Ordering::Relaxed)
}

/// Queue the acknowledgement of a command whose counter was spent
fn acknowledge(counter: u32, result: u32) {
    let days = crate::contact_monitor::contact_monitor_seconds_since_contact() / 86_400;
    let queued_ms = c::now_ms();
    let _ = ACKS.with(|q| q.push(Ack { counter, result, days, queued_ms }));
}

/// The oldest acknowledgement still to send
pub(crate) fn next_ack(now_ms: u64) -> Option<Ack> {
    ACKS.with(|q| q.front(now_ms)).flatten()
}

/// The oldest acknowledgement went out
pub(crate) fn ack_sent() {
    let _ = ACKS.with(Queue::pop);
}

/// The oldest acknowledgement waiting (`cmd_handler.h`): false if none
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_ack_pending(counter: *mut u32, result: *mut u32, days: *mut u32) -> bool {
    // SAFETY: the C contract: NULL or a live uint32_t each
    let (Some(counter), Some(result), Some(days)) =
        (unsafe { ptr::out(counter) }, unsafe { ptr::out(result) }, unsafe { ptr::out(days) })
    else {
        return false;
    };
    match next_ack(c::now_ms()) {
        Some(a) => {
            *counter = a.counter;
            *result = a.result;
            *days = a.days;
            true
        }
        None => false,
    }
}

/// Authenticated commands counted as contact since boot (HK1 `CN`)
#[unsafe(no_mangle)]
pub extern "C" fn cmd_contact_count() -> u32 {
    contacts()
}

/// A rejection's result code and fault-log text
fn rejection(why: Rejection) -> (u32, &'static CStr) {
    match why {
        Rejection::InvalidFrame => (result::INVALID_CMD, c"Invalid command frame"),
        Rejection::Malformed => (result::INVALID_PARAM, c"Telecommand payload malformed"),
        Rejection::AuthFailed(_) => {
            with(|h| h.stats.auth_failures = h.stats.auth_failures.wrapping_add(1));
            (result::AUTH_FAIL, c"Telecommand authentication failed")
        }
        Rejection::UnknownCommand => (result::INVALID_CMD, c"Unknown command ID"),
        Rejection::ArgLength => (result::INVALID_PARAM, c"Command argument length"),
        Rejection::CounterNotPersisted => (result::EXEC_FAIL, c"Command counter not persisted"),
    }
}

/// Process a received uplink frame (see cmd_handler.h)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_process(raw_data: *const u8, data_len: usize, response: *mut CmdResponse) -> i32 {
    // SAFETY: the C contract: raw_data has data_len bytes; response points to a CmdResponse_t
    let (Some(raw), Some(response)) = (unsafe { ptr::bytes(raw_data, data_len) }, unsafe { ptr::out(response) }) else {
        return rc::NULL_PTR;
    };
    if !with(|h| h.initialized) {
        return rc::INVALID;
    }
    *response = CmdResponse::zeroed();
    with(|h| h.stats.total_received = h.stats.total_received.wrapping_add(1));

    let mut admission = None;
    let status = cmd_auth::with_keys(|keys| {
        admission = Some(telecommand::admit(raw, keys, &mut HalFlash, |id| custom_handler(id).is_some()));
        rc::OK
    });
    let Some(admission) = admission else { return status };

    match admission {
        Admission::Rejected { cmd_id, counter, why, contact: was_contact } => {
            response.cmd_id = cmd_id;
            if let Some(counter) = counter {
                response.sequence_num = counter.to_le_bytes()[0];
            }
            let (code, reason) = rejection(why);
            reject(response, code, reason);
            if was_contact {
                contact();
                acknowledge(counter.unwrap_or(0), code);
            }
            rc::OK
        }
        Admission::Accepted { command, contact: was_contact } => {
            if was_contact {
                contact();
            }
            with(|h| h.reject_logged = false);
            let mut cmd = CmdCommand {
                cmd_id: command.cmd_id,
                sequence_num: command.counter.to_le_bytes()[0],
                payload_len: u8::try_from(command.args.len()).unwrap_or(u8::MAX),
                payload: [0; MAX_ARGS],
                // Low 32 bits of the mission clock, as Command_t has always held
                #[allow(clippy::cast_possible_truncation)]
                timestamp_ms: c::now_ms() as u32,
                requires_ack: true,
            };
            for (dst, src) in cmd.payload.iter_mut().zip(command.args) {
                *dst = *src;
            }
            // SAFETY: both point to live locals / the caller's response
            let status = unsafe { cmd_execute(&cmd, response) };
            // Acknowledged only when it counted as contact: never for the
            // keyless TX_DISABLE, nor for a frame whose counter was not spent
            if was_contact {
                acknowledge(command.counter, response.result);
            }
            status
        }
    }
}

/// Execute a command already authenticated or generated onboard
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_execute(cmd: *const CmdCommand, response: *mut CmdResponse) -> i32 {
    // SAFETY: the C contract
    let (Some(cmd), Some(response)) = (unsafe { ptr::in_(cmd) }, unsafe { ptr::out(response) }) else {
        return rc::NULL_PTR;
    };
    response.cmd_id = cmd.cmd_id;
    response.sequence_num = cmd.sequence_num;
    response.result = result::SUCCESS;
    response.data_len = 0;

    if !cmd_is_allowed(cmd.cmd_id) {
        response.result = result::NOT_ALLOWED;
        with(|h| h.stats.total_rejected = h.stats.total_rejected.wrapping_add(1));
        c::log(fault_type::UART, severity::WARNING, c"Command not allowed in state", 0.0);
        return rc::OK;
    }

    response.result = match custom_handler(cmd.cmd_id) {
        // SAFETY: a handler registered by C code, called with valid pointers
        Some(handler) => unsafe { handler(cmd, response) },
        None => match Command::from_id(cmd.cmd_id) {
            Some(command) => dispatch(command, cmd, response),
            None => result::INVALID_CMD,
        },
    };

    let outcome = response.result;
    let success = outcome == result::SUCCESS;
    with(|h| {
        h.stats.last_cmd_time_ms = cmd.timestamp_ms;
        h.stats.last_cmd_id = cmd.cmd_id;
        h.stats.last_result = outcome;
        if success {
            h.stats.total_executed = h.stats.total_executed.wrapping_add(1);
        } else {
            h.stats.total_rejected = h.stats.total_rejected.wrapping_add(1);
        }
    });
    if success {
        // SAFETY: no arguments
        let _ = unsafe { c::sys_increment_command_count() };
    }
    rc::OK
}

fn be16(payload: &[u8; MAX_ARGS]) -> u16 {
    u16::from_be_bytes([payload[0], payload[1]])
}

fn ok_invalid_or_fail(code: i32) -> u32 {
    match code {
        rc::OK => result::SUCCESS,
        rc::INVALID => result::INVALID_PARAM,
        _ => result::EXEC_FAIL,
    }
}

/// A refusal by the module (`ERROR`) is NOT_ALLOWED: passivation not armed,
/// or already done
fn allowed_or(code: i32) -> u32 {
    match code {
        rc::ERROR => result::NOT_ALLOWED,
        other => ok_invalid_or_fail(other),
    }
}

fn flag(b: bool) -> u8 {
    u8::from(b)
}

/// One arm per dictionary command; no wildcard
// JPL rule 4 deviation: one exhaustive match, one short arm per command.
// Split up, the compiler could no longer show that every command is handled.
#[allow(clippy::too_many_lines)]
fn dispatch(command: Command, cmd: &CmdCommand, response: &mut CmdResponse) -> u32 {
    // SAFETY (all calls below): C functions taking no pointers or pointers
    // to locals that live for the call
    match command {
        Command::Nop => result::SUCCESS,
        Command::Reset => {
            // Logged only until a hardware reset exists
            c::log(fault_type::INIT, severity::INFO, c"Reset commanded", 0.0);
            result::SUCCESS
        }
        // Power modes SAFE/IDLE/ACTIVE are the operational states of the same
        // name: one path, through the mode manager (SW-14)
        Command::SetMode | Command::EpsSetMode => set_mode(cmd),
        Command::GetStatus => {
            let status = unsafe {
                [
                    c::sys_get_operational_state().to_le_bytes()[0],
                    c::sys_get_power_mode().to_le_bytes()[0],
                    flag(c::sys_is_payload_enabled()),
                    flag(c::sys_has_thermal_fault()),
                ]
            };
            response.data[..4].copy_from_slice(&status);
            response.data_len = 4;
            result::SUCCESS
        }
        Command::ClearFaults => {
            let _ = crate::fault_mgmt::fault_log_clear();
            result::SUCCESS
        }
        Command::KeyRotate => ok_invalid_or_fail(cmd_auth::rotate(cmd.payload[0])),
        Command::SetContactTimeout => {
            ok_invalid_or_fail(unsafe { c::contact_monitor_set_timeout_days(be16(&cmd.payload)) })
        }
        Command::EpsGetTelemetry => {
            let mut power = PowerState::default();
            let _ = unsafe { c::sys_get_power_state(&mut power) };
            // Saturating conversions (0.1 V and 1 % resolution)
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let packed = [
                (power.battery_voltage * 10.0) as u8,
                (power.state_of_charge * 100.0) as u8,
                power.power_mode.to_le_bytes()[0],
            ];
            response.data[..3].copy_from_slice(&packed);
            response.data_len = 3;
            result::SUCCESS
        }
        Command::AdcsDetumble => {
            let _ = state_machine::process(unsafe { c::sys_get_sm_context() }, Event::BootComplete as u32);
            result::SUCCESS
        }
        Command::AdcsGetAttitude => {
            let mut adcs = AdcsState::default();
            let _ = unsafe { c::sys_get_adcs_state(&mut adcs) };
            response.data[..2].copy_from_slice(&[flag(adcs.detumbled), flag(adcs.sun_acquired)]);
            response.data_len = 2;
            result::SUCCESS
        }
        Command::SetBeacon => {
            if cmd.payload_len < 2 {
                return result::INVALID_PARAM;
            }
            match unsafe { c::sys_set_beacon_interval(be16(&cmd.payload)) } {
                rc::OK => result::SUCCESS,
                _ => result::INVALID_PARAM,
            }
        }
        Command::TxEnable => match unsafe { c::rf_control_tx_on() } {
            rc::OK => result::SUCCESS,
            _ => result::EXEC_FAIL,
        },
        Command::TxDisable => {
            // The hardware line is dropped before persistence, so the
            // transmitter is off even if the latch cannot be saved
            let _ = unsafe { c::rf_control_tx_off() };
            result::SUCCESS
        }
        Command::SetPower => ok_invalid_or_fail(unsafe { c::rf_control_set_power_level(u32::from(cmd.payload[0])) }),
        Command::PldEnable => {
            // Not while the commissioning hold is set (LV-01)
            if crate::ops_state::ops_commissioning_hold() {
                return result::NOT_ALLOWED;
            }
            let _ = unsafe { c::sys_set_payload_enabled(true) };
            result::SUCCESS
        }
        Command::PldDisable => {
            let _ = unsafe { c::sys_set_payload_enabled(false) };
            result::SUCCESS
        }
        Command::CommissioningRelease => ok_invalid_or_fail(crate::ops_state::ops_set_commissioning_hold(false)),
        Command::GetHousekeeping => {
            crate::executive::request_housekeeping();
            result::SUCCESS
        }
        Command::XpdrEnable => ok_invalid_or_fail(crate::ops_state::ops_set_xpdr_enabled(true)),
        Command::XpdrDisable => ok_invalid_or_fail(crate::ops_state::ops_set_xpdr_enabled(false)),
        Command::DeployRetry => match cmd.payload[0] {
            0 | 1 if crate::deployment::deployment_force_retry(u32::from(cmd.payload[0])) => result::SUCCESS,
            0 | 1 => result::NOT_ALLOWED,
            _ => result::INVALID_PARAM,
        },
        Command::GnssRequest => {
            crate::executive::request_gnss_session();
            result::SUCCESS
        }
        Command::AdcsSetGeometry => ok_invalid_or_fail(crate::ops_state::ops_set_adcs_geometry(cmd.payload[0])),
        Command::AdcsSetGyroAlign => {
            ok_invalid_or_fail(crate::adcs_control::gyro_alignment_command(cmd.payload.get(..36).unwrap_or(&[])))
        }
        Command::AdcsSetMagBias => {
            ok_invalid_or_fail(crate::adcs_control::mag_bias_command(cmd.payload.get(..13).unwrap_or(&[])))
        }
        Command::DumpTlm => {
            let p = &cmd.payload;
            let first = u32::from_be_bytes([p[1], p[2], p[3], p[4]]);
            ok_invalid_or_fail(crate::hk_store::hk_store_dump(p[0], first, u16::from_be_bytes([p[5], p[6]])))
        }
        Command::FramBypass => {
            // SAFETY: the payload holds payload_len bytes
            allowed_or(unsafe { crate::fram_health::fram_bypass(cmd.payload.as_ptr(), usize::from(cmd.payload_len)) })
        }
        Command::EpsClearLockout => {
            // SAFETY: the payload holds payload_len bytes
            ok_invalid_or_fail(unsafe {
                crate::eps_lockout::eps_lockout_clear(cmd.payload.as_ptr(), usize::from(cmd.payload_len))
            })
        }
        Command::PassivateArm => {
            // SAFETY: the payload holds payload_len bytes
            allowed_or(unsafe {
                crate::passivation::passivation_arm(cmd.payload.as_ptr(), usize::from(cmd.payload_len))
            })
        }
        Command::PassivateExecute => allowed_or(crate::passivation::passivation_execute()),
    }
}

/// Ground mode change through the mode manager's table, so the ground obeys
/// the same rules as onboard events (SAFE to ACTIVE needs IDLE first).
/// Requesting the current state succeeds without a transition.
fn set_mode(cmd: &CmdCommand) -> u32 {
    if cmd.payload_len != 1 {
        return result::INVALID_PARAM;
    }
    let (target, event) = match cmd.payload[0] {
        0 => (State::Safe, Event::GroundCmdSafe),
        1 => (State::Idle, Event::GroundCmdIdle),
        2 => (State::Active, Event::GroundCmdActive),
        _ => return result::INVALID_PARAM,
    };
    // SAFETY: the context pointer comes from system_state.c
    let ctx = unsafe { c::sys_get_sm_context() };
    match state_machine::current(ctx) {
        Err(_) => return result::EXEC_FAIL,
        Ok(current) if current == target as u32 => return result::SUCCESS,
        Ok(_) => {}
    }
    match state_machine::process(ctx, event as u32) {
        (rc::OK, sm_result::NO_TRANSITION | sm_result::GUARD_FAILED) => result::NOT_ALLOWED,
        (rc::OK, sm_result::OK) => result::SUCCESS,
        _ => result::EXEC_FAIL,
    }
}

/// Whether a command is allowed in the current state
#[unsafe(no_mangle)]
pub extern "C" fn cmd_is_allowed(cmd_id: u8) -> bool {
    // SAFETY: no arguments
    let state = State::from_raw(unsafe { c::sys_get_operational_state() });
    telecommand::allowed(cmd_id, state)
}

/// Command statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_get_stats(stats: *mut CmdStats) -> i32 {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::out(stats) }) else { return rc::NULL_PTR };
    *out = with(|h| h.stats);
    rc::OK
}

/// Clear command statistics
#[unsafe(no_mangle)]
pub extern "C" fn cmd_clear_stats() -> i32 {
    with(|h| h.stats = NO_STATS);
    rc::OK
}

/// Register a handler for a command ID (it is tried before the dictionary)
#[unsafe(no_mangle)]
pub extern "C" fn cmd_register_handler(cmd_id: u8, handler: Option<CmdHandlerFunc>) -> i32 {
    let Some(handler) = handler else { return rc::NULL_PTR };
    HANDLER
        .with(|h| match h.custom.iter_mut().find(|slot| slot.is_none()) {
            Some(slot) => {
                *slot = Some((cmd_id, handler));
                rc::OK
            }
            None => rc::NO_MEM,
        })
        .unwrap_or(rc::BUSY)
}

/// Reserved IDs without a handler (`CmdReserved_t`)
const RESERVED: [(u8, &CStr); 7] = [
    (0x04, c"SET_TIME"),
    (0x11, c"EPS_ENABLE_HEATER"),
    (0x12, c"EPS_DISABLE_HEATER"),
    (0x20, c"ADCS_SET_MODE"),
    (0x22, c"ADCS_POINT_SUN"),
    (0x42, c"PLD_SEND_DATA"),
    (0x43, c"PLD_REQUEST_DATA"),
];

/// Command name (the dictionary name), or "UNKNOWN"
#[unsafe(no_mangle)]
pub extern "C" fn cmd_get_name(cmd_id: u8) -> *const c_char {
    let name = match Command::from_id(cmd_id) {
        Some(command) => command.c_name(),
        None => RESERVED.iter().find(|(id, _)| *id == cmd_id).map_or(c"UNKNOWN", |(_, n)| *n),
    };
    name.as_ptr()
}
