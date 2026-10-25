//! `system_state.h`: the system state record (`sqso_fsw::system_state`)
//! behind its accessors, persisted through [`crate::store`]
//! (`Store::SystemState`: /tmp/smart_qso_state.dat on the host).
//!
//! The state machine context `SmContext_t` is handed out by pointer
//! (`sys_get_sm_context()`) to the mode manager, so it lives in its own
//! cell, outside the record's [`Global`], and is copied in whenever the
//! record's image is taken (CRC, save, full snapshot).

use core::cell::UnsafeCell;

use sqso_fsw::mode::State;
use sqso_fsw::system_state::{
    self as sys, Comm, IMAGE_SIZE, Image, Mission, Sm, SystemState, Thermal, Uptime, clamp_soc,
};

use crate::c::{self, AdcsState, PowerState, SmContext, rc};
use crate::cell::Global;
use crate::store::{self, Store};
use crate::{ptr, state_machine};

unsafe extern "C" {
    fn eps_get_power_mode() -> u32;
    fn eps_update_soc(soc: f64);
}

struct Sys {
    state: SystemState,
    uptime: Uptime,
}

static SYS: Global<Sys> = Global::new(Sys { state: SystemState::ZERO, uptime: Uptime::new() });

/// The state machine context, at a fixed address for `sys_get_sm_context()`
struct SmCell(UnsafeCell<SmContext>);

// SAFETY: the flight software is one thread; the context is reached through
// the raw pointer only (the mode manager, and the copies below), never
// through a reference held across a call
unsafe impl Sync for SmCell {}

static SM: SmCell = SmCell(UnsafeCell::new(SmContext {
    current_state: 0,
    previous_state: 0,
    transition_count: 0,
    last_transition_ms: 0,
    state_entry_ms: 0,
    is_initialized: false,
}));

fn sm_ptr() -> *mut SmContext {
    SM.0.get()
}

/// The context as stored in the record
fn sm_read() -> Sm {
    let p = sm_ptr();
    // SAFETY: p is the static's valid, aligned pointer; plain reads
    unsafe {
        Sm {
            current_state: (*p).current_state,
            previous_state: (*p).previous_state,
            transition_count: (*p).transition_count,
            last_transition_ms: (*p).last_transition_ms,
            state_entry_ms: (*p).state_entry_ms,
            is_initialized: (*p).is_initialized,
        }
    }
}

fn sm_write(sm: &Sm) {
    let p = sm_ptr();
    // SAFETY: as sm_read; plain writes
    unsafe {
        (*p).current_state = sm.current_state;
        (*p).previous_state = sm.previous_state;
        (*p).transition_count = sm.transition_count;
        (*p).last_transition_ms = sm.last_transition_ms;
        (*p).state_entry_ms = sm.state_entry_ms;
        (*p).is_initialized = sm.is_initialized;
    }
}

/// Run `f` on the record, then mark it changed; BUSY if it is in use
fn modify(f: impl FnOnce(&mut SystemState)) -> i32 {
    let now = c::now_ms();
    SYS.with(|s| {
        f(&mut s.state);
        s.state.mark_dirty(now);
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Read from the record (`default` if it is in use)
fn read<R>(default: R, f: impl FnOnce(&SystemState) -> R) -> R {
    SYS.with(|s| f(&s.state)).unwrap_or(default)
}

fn now32() -> u32 {
    #[allow(clippy::cast_possible_truncation)] // the low 32 bits, as the C fields
    let t = c::now_ms() as u32;
    t
}

/// The power state with the EPS's power mode and state of charge
pub(crate) fn power_state() -> PowerState {
    let p = read(sys::Power::default(), |s| s.power);
    // SAFETY: C EPS accessors without arguments
    let (power_mode, state_of_charge) = unsafe { (eps_get_power_mode(), c::eps_get_soc()) };
    PowerState {
        battery_voltage: p.battery_voltage,
        battery_current: p.battery_current,
        state_of_charge,
        solar_power: p.solar_power,
        total_energy_wh: p.total_energy_wh,
        power_mode,
        payload_enabled: p.payload_enabled,
        mode_entry_time_ms: p.mode_entry_time_ms,
    }
}

/// The thermal state
pub(crate) fn thermal_state() -> Thermal {
    read(Thermal::default(), |s| s.thermal)
}

/// The communications state
pub(crate) fn comm_state() -> Comm {
    read(Comm::default(), |s| s.comm)
}

/// The ADCS state
pub(crate) fn adcs_state() -> AdcsState {
    let a = read(sys::Adcs::default(), |s| s.adcs);
    AdcsState {
        mag_ut: a.mag_ut,
        gyro_dps: a.gyro_dps,
        sun_vector: a.sun_vector,
        detumbled: a.detumbled,
        sun_acquired: a.sun_acquired,
        last_update_ms: a.last_update_ms,
    }
}

/*---------------------------------------------------------------------------
 * Initialization and persistence
 *-------------------------------------------------------------------------*/

/// Defaults, the state machine in BOOT (its entry action runs), CRC set
#[unsafe(no_mangle)]
pub extern "C" fn sys_state_init() -> i32 {
    // Zero, then the defaults; initialized once the state machine is
    let defaults =
        SystemState { initialized: false, watchdog_ok: false, last_update_ms: 0, ..SystemState::defaults(0) };
    if SYS.with(|s| s.state = defaults).is_none() {
        return rc::BUSY;
    }
    sm_write(&Sm::default());
    // SAFETY: the context's own static pointer
    let result = unsafe { state_machine::sm_init(sm_ptr(), State::Boot as u32) };
    if result != rc::OK {
        return result;
    }
    let now = c::now_ms();
    let done = SYS.with(|s| {
        s.state.initialized = true;
        s.state.watchdog_ok = true;
        s.state.persistence_dirty = false;
        s.state.last_update_ms = now;
    });
    if done.is_none() {
        return rc::BUSY;
    }
    let _ = sys_state_update_crc();
    rc::OK
}

/// Load the record from storage; defaults (`sys_state_init()`) if there is
/// none, it is short or its CRC does not match
#[unsafe(no_mangle)]
pub extern "C" fn sys_state_load() -> i32 {
    let mut image = Image([0; IMAGE_SIZE]);
    match store::load(Store::SystemState, &mut image.0) {
        Ok(IMAGE_SIZE) => {}
        _ => return sys_state_init(), // no saved state, or a read error
    }
    if sys::image_stored_crc(&image) != sys::image_crc(&image) {
        return sys_state_init();
    }
    let (state, sm) = SystemState::from_image(&image);
    if SYS.with(|s| s.state = state).is_none() {
        return rc::BUSY;
    }
    sm_write(&sm);
    // Re-initialize the state machine in the saved state (its entry action
    // runs; an invalid saved state leaves the context as loaded)
    // SAFETY: the context's own static pointer
    let _ = unsafe { state_machine::sm_init(sm_ptr(), sm.current_state) };
    SYS.with(|s| {
        s.state.initialized = true;
        s.state.persistence_dirty = false;
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Save the record (CRC updated first)
#[unsafe(no_mangle)]
pub extern "C" fn sys_state_save() -> i32 {
    let image = SYS.with(|s| {
        if !s.state.initialized {
            return None;
        }
        let sm = sm_read();
        s.state.crc32 = s.state.compute_crc(&sm);
        Some(s.state.image(&sm))
    });
    let image = match image {
        Some(Some(image)) => image,
        Some(None) => return rc::INVALID,
        None => return rc::BUSY,
    };
    // Flash is erased before it is written; a file is replaced
    #[cfg(target_os = "none")]
    store::erase(Store::SystemState);
    if store::save(Store::SystemState, &image.0) != rc::OK {
        return rc::IO;
    }
    let now = c::now_ms();
    SYS.with(|s| {
        s.state.persistence_dirty = false;
        s.state.last_persist_ms = now;
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Whether the record changed since it was saved
#[unsafe(no_mangle)]
pub extern "C" fn sys_state_is_dirty() -> bool {
    read(false, |s| s.persistence_dirty)
}

/// Clear the changed flag
#[unsafe(no_mangle)]
pub extern "C" fn sys_state_clear_dirty() {
    let _ = SYS.with(|s| s.state.persistence_dirty = false);
}

/*---------------------------------------------------------------------------
 * Power
 *-------------------------------------------------------------------------*/

/// The power state (power mode and SOC are the EPS's)
#[unsafe(no_mangle)]
#[allow(private_interfaces)] // the mirror is crate-private (c.rs); C sees the symbol only
pub unsafe extern "C" fn sys_get_power_state(state: *mut PowerState) -> i32 {
    // SAFETY: the C contract: NULL or a PowerState_t
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    *out = power_state();
    rc::OK
}

/// Record the battery voltage
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_battery_voltage(voltage_v: f64) -> i32 {
    modify(|s| s.power.battery_voltage = voltage_v)
}

/// Record the battery current
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_battery_current(current_a: f64) -> i32 {
    modify(|s| s.power.battery_current = current_a)
}

/// Give the EPS the state of charge, held to 0-1
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_state_of_charge(soc: f64) -> i32 {
    // SAFETY: C EPS call with a plain value
    unsafe { eps_update_soc(clamp_soc(soc)) };
    modify(|_| {})
}

/// The EPS state of charge
#[unsafe(no_mangle)]
pub extern "C" fn sys_get_state_of_charge() -> f64 {
    // SAFETY: C EPS accessor
    unsafe { c::eps_get_soc() }
}

/// Set the EPS power mode directly (operational mode changes go through the
/// state machine, whose entry actions set it)
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_power_mode(mode: u32) -> i32 {
    // SAFETY: C EPS calls with plain values
    let result = unsafe { c::eps_set_power_mode(mode, c::eps_get_soc()) };
    let now = now32();
    let _ = modify(|s| s.power.mode_entry_time_ms = now);
    result
}

/// The EPS power mode
#[unsafe(no_mangle)]
pub extern "C" fn sys_get_power_mode() -> u32 {
    // SAFETY: C EPS accessor
    unsafe { eps_get_power_mode() }
}

/// Record the payload switch
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_payload_enabled(enabled: bool) -> i32 {
    modify(|s| s.power.payload_enabled = enabled)
}

/// Whether the payload is enabled
#[unsafe(no_mangle)]
pub extern "C" fn sys_is_payload_enabled() -> bool {
    read(false, |s| s.power.payload_enabled)
}

/*---------------------------------------------------------------------------
 * Thermal
 *-------------------------------------------------------------------------*/

/// The thermal state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_get_thermal_state(state: *mut Thermal) -> i32 {
    // SAFETY: the C contract: NULL or a ThermalState_t (same layout)
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    *out = thermal_state();
    rc::OK
}

/// Record a temperature (0 OBC, 1 EPS, 2 battery, 3 payload, 4 external)
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_temperature(sensor: u8, temp_c: f32) -> i32 {
    let now = c::now_ms();
    match SYS.with(|s| s.state.set_temperature(sensor, temp_c, now)) {
        Some(true) => rc::OK,
        Some(false) => rc::INVALID,
        None => rc::BUSY,
    }
}

/// Whether the last temperature reading was out of limits
#[unsafe(no_mangle)]
pub extern "C" fn sys_has_thermal_fault() -> bool {
    read(false, SystemState::has_thermal_fault)
}

/*---------------------------------------------------------------------------
 * ADCS
 *-------------------------------------------------------------------------*/

/// The ADCS state
#[unsafe(no_mangle)]
#[allow(private_interfaces)] // the mirror is crate-private (c.rs); C sees the symbol only
pub unsafe extern "C" fn sys_get_adcs_state(state: *mut AdcsState) -> i32 {
    // SAFETY: the C contract: NULL or an AdcsState_t (system_state.h)
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    *out = adcs_state();
    rc::OK
}

/// Record a magnetometer reading
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_magnetometer(x: f32, y: f32, z: f32) -> i32 {
    let now = now32();
    modify(|s| {
        s.adcs.mag_ut = [x, y, z];
        s.adcs.last_update_ms = now;
    })
}

/// Record a gyroscope reading
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_gyroscope(x: f32, y: f32, z: f32) -> i32 {
    let now = now32();
    modify(|s| {
        s.adcs.gyro_dps = [x, y, z];
        s.adcs.last_update_ms = now;
    })
}

/// Record whether the detumble is achieved
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_detumbled(achieved: bool) -> i32 {
    modify(|s| s.adcs.detumbled = achieved)
}

/// Whether the spacecraft is detumbled
#[unsafe(no_mangle)]
pub extern "C" fn sys_is_detumbled() -> bool {
    read(false, |s| s.adcs.detumbled)
}

/*---------------------------------------------------------------------------
 * Communications
 *-------------------------------------------------------------------------*/

/// The communications state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_get_comm_state(state: *mut Comm) -> i32 {
    // SAFETY: the C contract: NULL or a CommState_t (same layout)
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    *out = comm_state();
    rc::OK
}

/// Count a packet sent
#[unsafe(no_mangle)]
pub extern "C" fn sys_increment_packets_sent() -> i32 {
    modify(|s| s.comm.packets_sent = s.comm.packets_sent.wrapping_add(1))
}

/// Count a packet received
#[unsafe(no_mangle)]
pub extern "C" fn sys_increment_packets_received() -> i32 {
    modify(|s| s.comm.packets_received = s.comm.packets_received.wrapping_add(1))
}

/// Count a beacon
#[unsafe(no_mangle)]
pub extern "C" fn sys_increment_beacon_count() -> i32 {
    modify(|s| s.comm.beacon_count = s.comm.beacon_count.wrapping_add(1))
}

/// Record a ground contact now
#[unsafe(no_mangle)]
pub extern "C" fn sys_update_ground_contact() -> i32 {
    let now = now32();
    modify(|s| {
        s.comm.last_ground_contact_ms = now;
        s.comm.comm_active = true;
    })
}

/// Set the beacon interval (10-120 s)
#[unsafe(no_mangle)]
pub extern "C" fn sys_set_beacon_interval(interval_s: u16) -> i32 {
    let now = c::now_ms();
    match SYS.with(|s| s.state.set_beacon_interval(interval_s, now)) {
        Some(true) => rc::OK,
        Some(false) => rc::INVALID,
        None => rc::BUSY,
    }
}

/*---------------------------------------------------------------------------
 * Mission
 *-------------------------------------------------------------------------*/

/// The mission state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_get_mission_state(state: *mut Mission) -> i32 {
    // SAFETY: the C contract: NULL or a MissionState_t (same layout)
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    *out = read(Mission::default(), |s| s.mission);
    rc::OK
}

/// Count a boot
#[unsafe(no_mangle)]
pub extern "C" fn sys_increment_boot_count() -> i32 {
    modify(|s| s.mission.boot_count = s.mission.boot_count.wrapping_add(1))
}

/// The boot count
#[unsafe(no_mangle)]
pub extern "C" fn sys_get_boot_count() -> u32 {
    read(0, |s| s.mission.boot_count)
}

/// Add the whole seconds since the last call to the uptime counters
#[unsafe(no_mangle)]
pub extern "C" fn sys_update_uptime() -> i32 {
    let now = c::now_ms();
    SYS.with(|s| {
        if s.uptime.update(now, &mut s.state.mission) {
            s.state.mark_dirty(now);
        }
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Uptime, s
#[unsafe(no_mangle)]
pub extern "C" fn sys_get_uptime_s() -> u32 {
    read(0, |s| s.mission.uptime_s)
}

/// Count a QSO
#[unsafe(no_mangle)]
pub extern "C" fn sys_increment_qso_count() -> i32 {
    modify(|s| s.mission.qso_count = s.mission.qso_count.wrapping_add(1))
}

/// Count a command
#[unsafe(no_mangle)]
pub extern "C" fn sys_increment_command_count() -> i32 {
    modify(|s| s.mission.command_count = s.mission.command_count.wrapping_add(1))
}

/// Count an anomaly
#[unsafe(no_mangle)]
pub extern "C" fn sys_increment_anomaly_count() -> i32 {
    modify(|s| s.mission.anomaly_count = s.mission.anomaly_count.wrapping_add(1))
}

/*---------------------------------------------------------------------------
 * State machine
 *-------------------------------------------------------------------------*/

/// The state machine context
#[unsafe(no_mangle)]
pub extern "C" fn sys_get_sm_context() -> *mut SmContext {
    sm_ptr()
}

/// The operational state
#[unsafe(no_mangle)]
pub extern "C" fn sys_get_operational_state() -> u32 {
    sm_read().current_state
}

/*---------------------------------------------------------------------------
 * Integrity
 *-------------------------------------------------------------------------*/

/// Compute and store the record's CRC
#[unsafe(no_mangle)]
pub extern "C" fn sys_state_update_crc() -> i32 {
    let sm = sm_read();
    SYS.with(|s| {
        s.state.crc32 = s.state.compute_crc(&sm);
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Whether the stored CRC matches the record
#[unsafe(no_mangle)]
pub extern "C" fn sys_state_verify_crc() -> bool {
    let sm = sm_read();
    read(false, |s| s.crc32 == s.compute_crc(&sm))
}

/// The whole record (power mode and SOC are the EPS's)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sys_get_full_state(state: *mut Image) -> i32 {
    // SAFETY: the C contract: NULL or a SystemState_t (same size and alignment)
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    let sm = sm_read();
    let Some(mut snapshot) = SYS.with(|s| s.state) else { return rc::BUSY };
    // SAFETY: C EPS accessors
    unsafe {
        snapshot.power.power_mode = eps_get_power_mode();
        snapshot.power.state_of_charge = c::eps_get_soc();
    }
    *out = snapshot.image(&sm);
    rc::OK
}
