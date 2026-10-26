//! `eps_control.h`: the EPS power policy (`sqso_fsw::eps`): load switches,
//! power modes and their budgets, the payload enable rule, and the latest
//! state of charge. The state is stored through [`crate::store`]
//! (`EPS_CONFIG_FILE`, or `$SMART_QSO_EPS_CONFIG`, on the host; its flash
//! region on the target) as the C `EpsControlState_t` bytes.
//!
//! The payload guard and the fault log are called with no EPS state
//! borrowed.
//!
//! Coverage: only a nested call (an interrupt) finds `EPS` in use (busy:
//! `Global::with` gives `None`); no C callback runs while it is held. The
//! unit test `tests::busy_while_the_state_is_in_use` covers those paths
//! (`software/flight/scripts/safety_coverage.py` counts the unit tests'
//! export too).
//!
//! @requirement SRS-F011 Control power to subsystems via load switches
//! @requirement SRS-F012 Implement power modes SAFE, IDLE, ACTIVE
//! @requirement SRS-F016 Enforce power budget limits per mode

use core::ffi::CStr;
use core::sync::atomic::{AtomicBool, Ordering};

use sqso_fsw::eps::{self, EpsState, PayloadBlock, RECORD_SIZE};
use sqso_fsw::eps_telemetry::{self, Telemetry};
use sqso_fsw::mode::PowerMode;

use crate::c::{self, fault_type, rc, severity};
use crate::cell::Global;
use crate::ptr;
use crate::store::{self, Store};

unsafe extern "C" {
    // payload_guard.h
    fn payload_guard_may_enable(now_ms: u64) -> bool;
}

pub(crate) struct Eps {
    state: EpsState,
    initialized: bool,
    /// Latest measured state of charge; 0 until the first measurement
    soc: f64,
}

pub(crate) static EPS: Global<Eps> = Global::new(Eps { state: EpsState::DEFAULT, initialized: false, soc: 0.0 });

fn with<R>(busy: R, f: impl FnOnce(&mut Eps) -> R) -> R {
    EPS.with(f).unwrap_or(busy)
}

/// `hal_gpio.h` load-switch pins
const GPIO_PIN_PAYLOAD_EN: u32 = 0;
const GPIO_PIN_ADCS_EN: u32 = 2;
const GPIO_PIN_JETSON_PWR: u32 = 6;
const GPIO_DIR_OUTPUT: u32 = 1;
const GPIO_PULL_DOWN: u32 = 2;
/// I2C bus of the EPS controller (`hal_i2c.h`: bus 0 is I2C2)
const EPS_I2C_BUS: u32 = 0;
/// The controller's telemetry is read again after this long, ms
const TELEMETRY_MAX_AGE_MS: u64 = 500;

/// The latest EPS controller telemetry and when it was read
struct TelemetryCache {
    read_ms: u64,
    read: bool,
    telemetry: Option<Telemetry>,
}

/// The payload schedule holds the payload's switch-on (flight): a mode
/// entry may switch the payload off, but not on
static PAYLOAD_HELD: AtomicBool = AtomicBool::new(false);

static TELEMETRY: Global<TelemetryCache> = Global::new(TelemetryCache { read_ms: 0, read: false, telemetry: None });

/// Drive the payload and ADCS load-switch enables (and the Jetson power
/// gate with the payload's) from the EPS state: the switches follow the
/// state, so a reset that restores the state restores the switches
fn drive_load_pins(state: &EpsState) {
    // SAFETY: plain values (the HAL); a pin that is not configured is refused
    unsafe {
        let _ = c::hal_gpio_set(GPIO_PIN_PAYLOAD_EN, state.payload_enabled);
        let _ = c::hal_gpio_set(GPIO_PIN_JETSON_PWR, state.payload_enabled);
        let _ = c::hal_gpio_set(GPIO_PIN_ADCS_EN, state.adcs_enabled);
    }
}

/// The load-switch pins, as outputs, to the EPS state
fn configure_load_pins() {
    for pin in [GPIO_PIN_PAYLOAD_EN, GPIO_PIN_JETSON_PWR, GPIO_PIN_ADCS_EN] {
        // SAFETY: plain values (the HAL)
        let _ = unsafe { c::hal_gpio_config(pin, GPIO_DIR_OUTPUT, GPIO_PULL_DOWN) };
    }
    if let Some(state) = EPS.with(|e| e.state) {
        drive_load_pins(&state);
    }
}

/// The EPS controller's telemetry (SMART-QSO-ICD-001 §4.2), read over I2C
/// at most every 500 ms; `None` while the controller does not answer with a
/// plausible block (then the stand-ins are used, SW-01)
pub fn eps_telemetry() -> Option<Telemetry> {
    let now = c::now_ms();
    let cached = TELEMETRY
        .with(|t| (t.read && now >= t.read_ms && now - t.read_ms < TELEMETRY_MAX_AGE_MS).then_some(t.telemetry))
        .flatten();
    if let Some(telemetry) = cached {
        return telemetry;
    }
    let mut block = [0u8; eps_telemetry::BLOCK_LEN];
    let start = [0u8];
    // SAFETY: start is one byte, block BLOCK_LEN writable bytes (the HAL)
    let ok = unsafe {
        c::hal_i2c_write_read(EPS_I2C_BUS, eps_telemetry::ADDRESS, start.as_ptr(), 1, block.as_mut_ptr(), block.len())
    } == rc::OK;
    let telemetry = if ok { eps_telemetry::decode(&block) } else { None };
    let _ = TELEMETRY.with(|t| {
        t.read_ms = now;
        t.read = true;
        t.telemetry = telemetry;
    });
    telemetry
}

/// The low 32 bits of the mission clock (`last_control_ms`)
fn now_low32() -> u32 {
    let [b0, b1, b2, b3, ..] = c::now_ms().to_le_bytes();
    u32::from_le_bytes([b0, b1, b2, b3])
}

/// The load switches other than the payload
#[derive(Clone, Copy)]
enum Switch {
    Radio,
    Adcs,
    Beacon,
}

impl Switch {
    fn slot(self, s: &mut EpsState) -> &mut bool {
        match self {
            Switch::Radio => &mut s.radio_enabled,
            Switch::Adcs => &mut s.adcs_enabled,
            Switch::Beacon => &mut s.beacon_enabled,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Switch::Radio => "Radio",
            Switch::Adcs => "ADCS",
            Switch::Beacon => "Beacon",
        }
    }
}

/// Set a load switch: logged and persisted if it changes
fn control(switch: Switch, enable: bool) -> i32 {
    let now = now_low32();
    let changed = with(None, |e| {
        let slot = switch.slot(&mut e.state);
        if *slot == enable {
            return Some(false);
        }
        *slot = enable;
        e.state.last_control_ms = now;
        e.state.update_crc();
        Some(true)
    });
    match changed {
        None => rc::BUSY,
        Some(false) => rc::OK,
        Some(true) => {
            if let Some(state) = EPS.with(|e| e.state) {
                drive_load_pins(&state);
            }
            let what = if enable { "enabled" } else { "disabled" };
            c::console(format_args!("[EPS] {} {what}", switch.name()));
            let _ = eps_save_config();
            rc::OK
        }
    }
}

/// Initialize: the stored state, or the defaults
#[unsafe(no_mangle)]
pub extern "C" fn eps_init() -> i32 {
    if eps_load_config() != rc::OK {
        let now = now_low32();
        let set = with(false, |e| {
            e.state.reset_defaults(now);
            true
        });
        if set {
            c::console(format_args!("[EPS] Using default configuration"));
        }
    }
    let result = with(rc::BUSY, |e| {
        e.initialized = true;
        rc::OK
    });
    configure_load_pins();
    result
}

/// Copy of the EPS state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn eps_get_state(state: *mut EpsState) -> i32 {
    // SAFETY: the C contract: state points to an EpsControlState_t (same layout)
    let Some(out) = (unsafe { ptr::out(state) }) else {
        c::assert_failed(c"Null pointer: state", line!());
        return rc::NULL_PTR;
    };
    match EPS.with(|e| e.initialized.then_some(e.state)) {
        Some(Some(s)) => {
            *out = s;
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Switch the Jetson payload; switching on needs the payload guard's
/// consent and a state of charge from `EPS_SOC_PAYLOAD_THRESHOLD`
#[unsafe(no_mangle)]
pub extern "C" fn eps_control_payload(enable: bool, soc: f64) -> i32 {
    if !eps::soc_in_range(soc) {
        c::assert_failed(c"SOC out of range", line!());
        return rc::INVALID;
    }
    match EPS.with(|e| e.state.payload_enabled == enable) {
        None => return rc::BUSY,
        Some(true) => return rc::OK,
        Some(false) => {}
    }
    // The OBC's own power and temperature limits come first (SW-16)
    // SAFETY: plain value; no EPS state is borrowed
    let guard_allows = !enable || unsafe { payload_guard_may_enable(c::now_ms()) };
    match eps::payload_rule(enable, guard_allows, soc) {
        Err(PayloadBlock::Guard) => {
            c::log(
                fault_type::POWER,
                severity::WARNING,
                c"Payload enable blocked: guard cool-down or hot spreader",
                soc,
            );
            return rc::ERROR;
        }
        Err(PayloadBlock::LowSoc) => {
            c::log(fault_type::POWER, severity::WARNING, c"Payload enable blocked: low SOC", soc);
            return rc::ERROR;
        }
        Ok(()) => {}
    }
    let now = now_low32();
    // COVERAGE-EXEMPT: busy here needs a caller that takes the EPS state between the check at the top (busy there returns first) and this line; nothing in between borrows it, only an interrupt could
    if !with(false, |e| {
        e.state.payload_enabled = enable;
        e.state.last_control_ms = now;
        e.state.update_crc();
        true
    }) {
        // COVERAGE-EXEMPT: as above (the EPS state is free between the two borrows)
        return rc::BUSY;
    }
    if let Some(state) = EPS.with(|e| e.state) {
        drive_load_pins(&state);
    }
    if enable {
        c::log(fault_type::POWER, severity::INFO, c"Payload enabled", soc);
        c::console(format_args!("[EPS] Payload enabled"));
    } else {
        c::log(fault_type::POWER, severity::INFO, c"Payload disabled", soc);
        c::console(format_args!("[EPS] Payload disabled"));
    }
    let _ = eps_save_config();
    rc::OK
}

/// Switch the radio
#[unsafe(no_mangle)]
pub extern "C" fn eps_control_radio(enable: bool) -> i32 {
    control(Switch::Radio, enable)
}

/// Switch the ADCS
#[unsafe(no_mangle)]
pub extern "C" fn eps_control_adcs(enable: bool) -> i32 {
    control(Switch::Adcs, enable)
}

/// Switch the beacon
#[unsafe(no_mangle)]
pub extern "C" fn eps_control_beacon(enable: bool) -> i32 {
    control(Switch::Beacon, enable)
}

/// Enter a power mode: its load switches, then its power limit
#[unsafe(no_mangle)]
pub extern "C" fn eps_set_power_mode(mode: u32, soc: f64) -> i32 {
    let Some(power_mode) = eps::power_mode(mode) else {
        c::assert_failed(c"Invalid power mode", line!());
        return rc::INVALID;
    };
    let now = now_low32();
    let changed = with(None, |e| {
        if u32::from(e.state.power_mode) == mode {
            return Some(false);
        }
        e.state.power_mode = power_mode as u8;
        e.state.last_control_ms = now;
        Some(true)
    });
    match changed {
        None => return rc::BUSY,
        Some(false) => return rc::OK,
        Some(true) => {}
    }
    let loads = eps::loads(power_mode);
    // A refused payload enable is logged by eps_control_payload; a held
    // one is the payload schedule's to make
    if !(loads.payload && PAYLOAD_HELD.load(Ordering::Relaxed)) {
        let _ = eps_control_payload(loads.payload, soc);
    }
    let _ = eps_control_radio(loads.radio);
    let _ = eps_control_adcs(loads.adcs);
    let _ = eps_control_beacon(loads.beacon);
    let limit = eps::power_limit(power_mode);
    with((), |e| e.state.power_limit_w = limit);
    let (sev, text): (u32, &CStr) = match power_mode {
        PowerMode::Safe => (severity::WARNING, c"Entered SAFE mode"),
        PowerMode::Idle => (severity::INFO, c"Entered IDLE mode"),
        PowerMode::Active => (severity::INFO, c"Entered ACTIVE mode"),
    };
    c::log(fault_type::MODE_CHANGE, sev, text, soc);
    with((), |e| e.state.update_crc());
    let _ = eps_save_config();
    rc::OK
}

/// Current power mode (the stored byte)
#[unsafe(no_mangle)]
pub extern "C" fn eps_get_power_mode() -> u32 {
    with(PowerMode::Idle as u32, |e| u32::from(e.state.power_mode))
}

/// Whether the payload is powered
#[unsafe(no_mangle)]
pub extern "C" fn eps_is_payload_enabled() -> bool {
    with(false, |e| e.state.payload_enabled)
}

/// Hold (or release) the payload's switch-on by a mode entry: in flight
/// the payload schedule switches the payload on (`sqso_fsw::payload_schedule`)
pub fn eps_hold_payload(hold: bool) {
    PAYLOAD_HELD.store(hold, Ordering::Relaxed);
}

/// Whether the ADCS is powered
pub fn eps_is_adcs_enabled() -> bool {
    with(false, |e| e.state.adcs_enabled)
}

/// Whether the beacon transmitter is powered
#[unsafe(no_mangle)]
pub extern "C" fn eps_is_beacon_enabled() -> bool {
    with(false, |e| e.state.beacon_enabled)
}

/// Record the latest measured state of charge (values outside 0..1 are ignored)
#[unsafe(no_mangle)]
pub extern "C" fn eps_update_soc(soc: f64) {
    if eps::soc_in_range(soc) {
        with((), |e| e.soc = soc);
    }
}

/// Latest measured state of charge
#[unsafe(no_mangle)]
pub extern "C" fn eps_get_soc() -> f64 {
    with(0.0, |e| e.soc)
}

/// Current power limit, W
#[unsafe(no_mangle)]
pub extern "C" fn eps_get_power_limit() -> f64 {
    with(eps::POWER_LIMIT_IDLE_W, |e| e.state.power_limit_w)
}

/// Whether a measured bus voltage is within regulation (SW-16)
#[unsafe(no_mangle)]
pub extern "C" fn eps_bus_voltage_ok(volts: f64) -> bool {
    eps::bus_voltage_ok(volts)
}

/// Persist the EPS state
#[unsafe(no_mangle)]
pub extern "C" fn eps_save_config() -> i32 {
    let Some(bytes) = EPS.with(|e| {
        e.state.update_crc();
        e.state.to_bytes()
    }) else {
        return rc::BUSY;
    };
    store::save(Store::EpsConfig, &bytes)
}

/// Load the stored EPS state; a corrupt record keeps the current state
#[unsafe(no_mangle)]
pub extern "C" fn eps_load_config() -> i32 {
    let mut bytes = [0u8; RECORD_SIZE];
    match store::load(Store::EpsConfig, &mut bytes) {
        Ok(RECORD_SIZE) => {}
        Ok(_) => return rc::IO,
        Err(e) => return e,
    }
    let Ok(loaded) = EpsState::from_bytes(&bytes) else {
        c::console(format_args!("[EPS] CRC mismatch in config file, using defaults"));
        return rc::ERROR;
    };
    with(rc::BUSY, |e| {
        e.state = loaded;
        rc::OK
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// A nested call (an interrupt) while the EPS state is in use gets
    /// "busy", changes nothing, and readers get the safe defaults
    #[test]
    fn busy_while_the_state_is_in_use() {
        let _serial = serial();
        let mut copy = EpsState::DEFAULT;
        copy.power_mode = 7;
        let nested = EPS.with(|_| {
            // SAFETY: a live EpsState
            let got = unsafe { eps_get_state(&mut copy) };
            let switches = [eps_control_radio(true), eps_control_adcs(false), eps_control_beacon(true)];
            let modes = (eps_init(), eps_control_payload(true, 0.9), eps_set_power_mode(0, 0.5), eps_save_config());
            let readers = (eps_get_power_mode(), eps_is_payload_enabled(), eps_is_beacon_enabled(), eps_get_soc());
            assert!(!eps_is_adcs_enabled());
            (got, switches, modes, readers, eps_get_power_limit())
        });
        let busy = rc::BUSY;
        assert_eq!(
            nested,
            Some((
                busy,
                [busy; 3],
                (busy, busy, busy, busy),
                (PowerMode::Idle as u32, false, false, 0.0),
                eps::POWER_LIMIT_IDLE_W
            ))
        );
        assert_eq!(copy.power_mode, 7);
    }

    /// While the payload schedule holds the payload, an ACTIVE entry leaves
    /// it off (the other loads follow the mode); a direct switch-on, and a
    /// mode entry that switches the payload off, still work
    #[test]
    fn a_held_payload_is_not_switched_on_by_a_mode_entry() {
        let _serial = serial();
        let _ = EPS.with(|e| {
            e.state = EpsState::DEFAULT;
            e.initialized = true;
        });
        let active = PowerMode::Active as u32;
        let idle = PowerMode::Idle as u32;

        eps_hold_payload(true);
        assert_eq!(eps_set_power_mode(active, 0.9), rc::OK);
        assert!(!eps_is_payload_enabled());
        assert!(eps_is_adcs_enabled());
        assert_eq!(eps_control_payload(true, 0.9), rc::OK); // the schedule's own switch-on
        assert!(eps_is_payload_enabled());
        assert_eq!(eps_set_power_mode(idle, 0.9), rc::OK); // a held payload still goes off
        assert!(!eps_is_payload_enabled());

        eps_hold_payload(false);
        assert_eq!(eps_set_power_mode(active, 0.9), rc::OK);
        assert!(eps_is_payload_enabled());
        assert_eq!(eps_set_power_mode(idle, 0.9), rc::OK);
        store::erase(Store::EpsConfig);
    }
}
