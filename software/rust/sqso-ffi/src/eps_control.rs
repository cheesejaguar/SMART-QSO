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
//! The telemetry is the controller's register block (0x48), its `IMON_JET`
//! register (the Jetson rail's current, which the sensor `JPC` reads), and,
//! when they answer, the two INA228 monitors read directly over the same bus
//! (the pack gauge at 0x44, the array monitor at 0x40; RID EPSB): their
//! pack voltage, pack current and array current replace the controller's,
//! so the OBC's power figures do not depend on the controller's firmware.
//!
//! The Jetson's two switches go off in order (SQSO-OBC-BRD-001 B §11.2,
//! SMART-QSO-ICD-002 1.3 §10.4): JETSON_PWR at once, PAYLOAD_EN
//! `sqso_fsw::jetson_power::PAYLOAD_EN_DELAY_MS` later
//! ([`eps_payload_tick`], every pass).
//!
//! @requirement SRS-F011 Control power to subsystems via load switches
//! @requirement SRS-F147 Pack and array power monitors read directly by the OBC
//! @requirement SRS-F148 Jetson switch-off: SHUTDOWN, JET_PG fall within 5 s, then JETSON_PWR and PAYLOAD_EN 10 ms apart
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
    fn payload_guard_may_power_on(now_ms: u64) -> bool;
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

/// PAYLOAD_EN goes low at this time, ms (after JETSON_PWR)
static PAYLOAD_EN_RELEASE: Global<Option<u64>> = Global::new(None);

/// The INA228 monitors: configured, and when they were last tried, ms
struct Monitors {
    ready: [bool; 2],
    tried_ms: Option<u64>,
}

static MONITORS: Global<Monitors> = Global::new(Monitors { ready: [false; 2], tried_ms: None });
/// An INA228 that did not configure is tried again after this long, ms
const MONITOR_RETRY_MS: u64 = 60_000;

/// Drive the payload and ADCS load-switch enables (and the Jetson power
/// gate with the payload's) from the EPS state: the switches follow the
/// state, so a reset that restores the state restores the switches. A
/// payload switch-off drops JETSON_PWR now and PAYLOAD_EN 10 ms later
/// ([`eps_payload_tick`])
fn drive_load_pins(state: &EpsState) {
    let now = c::now_ms();
    // SAFETY: plain value (the HAL)
    let payload_high = unsafe { c::hal_gpio_get(GPIO_PIN_PAYLOAD_EN) };
    let delay_off = !state.payload_enabled && payload_high;
    let _ = PAYLOAD_EN_RELEASE.with(|r| {
        *r = if delay_off {
            Some(r.unwrap_or(now.saturating_add(sqso_fsw::jetson_power::PAYLOAD_EN_DELAY_MS)))
        } else {
            None
        };
    });
    // SAFETY: plain values (the HAL); a pin that is not configured is refused
    unsafe {
        let _ = c::hal_gpio_set(GPIO_PIN_JETSON_PWR, state.payload_enabled);
        if !delay_off {
            let _ = c::hal_gpio_set(GPIO_PIN_PAYLOAD_EN, state.payload_enabled);
        }
        let _ = c::hal_gpio_set(GPIO_PIN_ADCS_EN, state.adcs_enabled);
    }
}

/// Every pass: PAYLOAD_EN low once its delay after JETSON_PWR has run
pub fn eps_payload_tick(now_ms: u64) {
    let due = PAYLOAD_EN_RELEASE
        .with(|r| {
            let due = r.is_some_and(|t| now_ms >= t);
            if due {
                *r = None;
            }
            due
        })
        .unwrap_or(false);
    if due && !eps_is_payload_enabled() {
        // SAFETY: plain values (the HAL)
        let _ = unsafe { c::hal_gpio_set(GPIO_PIN_PAYLOAD_EN, false) };
    }
}

/// The INA228s' readings over the controller's (configuring them first,
/// and again after a failure every [`MONITOR_RETRY_MS`])
fn apply_monitors(t: &mut Telemetry, now: u64) {
    use sqso_fsw::ina228::{ARRAY, PACK};
    let mut bus = crate::hw::HalI2c(EPS_I2C_BUS);
    let retry = MONITORS
        .with(|m| {
            let due = m.ready != [true; 2] && m.tried_ms.is_none_or(|t0| now.saturating_sub(t0) >= MONITOR_RETRY_MS);
            if due {
                m.tried_ms = Some(now);
            }
            due
        })
        .unwrap_or(false);
    let mut ready = MONITORS.with(|m| m.ready).unwrap_or([false; 2]);
    if retry {
        ready = [ready[0] || PACK.init(&mut bus), ready[1] || ARRAY.init(&mut bus)];
    }
    let pack = ready[0].then(|| PACK.reading(&mut bus).ok()).flatten();
    let array = ready[1].then(|| ARRAY.reading(&mut bus).ok()).flatten();
    if let Some(p) = pack.filter(|p| {
        p.vbus_v >= eps_telemetry::VBATT_PLAUSIBLE_V.0 && p.vbus_v <= eps_telemetry::VBATT_PLAUSIBLE_V.1
    }) {
        t.vbatt_v = p.vbus_v;
        t.ibatt_a = p.current_a;
    }
    if let Some(a) = array {
        t.isolar_a = a.current_a;
    }
    let _ = MONITORS.with(|m| m.ready = [pack.is_some(), array.is_some()]);
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
    let mut telemetry = if ok { eps_telemetry::decode(&block) } else { None };
    if let Some(t) = telemetry.as_mut() {
        let reg = [eps_telemetry::IMON_JET];
        let mut imon = [0u8; 2];
        // SAFETY: both are live buffers (the HAL)
        if unsafe { c::hal_i2c_write_read(EPS_I2C_BUS, eps_telemetry::ADDRESS, reg.as_ptr(), 1, imon.as_mut_ptr(), 2) }
            == rc::OK
        {
            t.ijet_a = eps_telemetry::imon_jet_a(imon);
        }
        apply_monitors(t, now);
    }
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
            let _ = EPS.with(|e| drive_load_pins(&e.state));
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
    let guard_allows = !enable || unsafe { payload_guard_may_power_on(c::now_ms()) };
    match eps::payload_rule(enable, guard_allows, soc) {
        Err(PayloadBlock::Guard) => {
            c::log(
                fault_type::POWER,
                severity::WARNING,
                c"Payload enable blocked: guard cool-down, hot or cold spreader",
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
    let _ = EPS.with(|e| drive_load_pins(&e.state));
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

    /// The controller's telemetry: read over I2C, kept for 500 ms, read
    /// again when the clock is behind the cache (a reset restarts it), and
    /// none when the bus does not answer
    #[test]
    fn telemetry_is_read_cached_and_read_again() {
        let _serial = serial();
        let write = |soc: f64, vbatt_v: f64| {
            let mut block = [0u8; eps_telemetry::BLOCK_LEN];
            eps_telemetry::encode(&Telemetry { soc, vbatt_v, ..Telemetry::default() }, &mut block);
            let mut regs = [0u8; 1 + eps_telemetry::BLOCK_LEN];
            regs[1..].copy_from_slice(&block);
            // SAFETY: regs is a live buffer of its length
            unsafe { crate::hal_sim::hal_i2c_write(EPS_I2C_BUS, eps_telemetry::ADDRESS, regs.as_ptr(), regs.len()) }
        };
        let forget = || {
            let _ = TELEMETRY.with(|t| t.read = false);
        };
        assert_eq!(crate::hal_sim::hal_i2c_init(EPS_I2C_BUS, 100_000), rc::OK);
        assert_eq!(write(0.8, 15.0), rc::OK);
        forget();
        assert_eq!(eps_telemetry().map(|t| t.soc), Some(0.8));
        assert_eq!(write(0.5, 15.0), rc::OK);
        assert_eq!(eps_telemetry().map(|t| t.soc), Some(0.8), "cached");
        let _ = TELEMETRY.with(|t| t.read_ms = c::now_ms() + 10_000);
        assert_eq!(eps_telemetry().map(|t| t.soc), Some(0.5), "the clock is behind the cache: read again");
        forget();
        assert_eq!(crate::hal_sim::holding_the_sim(eps_telemetry), Some(None), "no answer");
        assert_eq!(crate::hal_sim::hal_i2c_init(EPS_I2C_BUS, 100_000), rc::OK);
        forget();
        assert_eq!(eps_telemetry(), None, "an implausible block is no telemetry");
        forget();
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
