//! `adcs_control.h`: attitude control (`sqso_fsw::adcs`) on the simulated
//! magnetometer and sun sensors.

use sqso_fsw::adcs::{self, Adcs, AdcsError, MagData, State, SunSensorData, Telemetry, Vec3};

use crate::c::{self, rc, severity};
use crate::cell::Global;
use crate::ptr;

/// `FAULT_TYPE_ADCS`
pub(crate) const FAULT_TYPE_ADCS: u32 = 17;

struct Module {
    adcs: Adcs,
    /// Simulated magnetometer field, uT
    sim_mag: Vec3,
    /// Simulated sun vector (its length is the intensity)
    sim_sun: Vec3,
    /// Simulated body rate, rad/s (set through the interface, read by
    /// nothing: the simulated field carries the rotation)
    #[allow(dead_code)]
    sim_rate: Vec3,
}

static ADCS: Global<Module> = Global::new(Module {
    adcs: Adcs::new(),
    sim_mag: Vec3::new(30.0, 0.0, 40.0), // typical LEO field
    sim_sun: Vec3::new(1.0, 0.0, 0.0),
    sim_rate: Vec3::new(0.05, 0.05, 0.05), // ~3 deg/s initial
});

/*---------------------------------------------------------------------------
 * Vector utilities (raw pointers: the C allows an output to alias an input)
 *-------------------------------------------------------------------------*/

/// Magnitude (0 for NULL)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec3_magnitude(v: *const Vec3) -> f64 {
    // SAFETY: the C contract: NULL or a Vec3_t
    unsafe { ptr::in_(v) }.map_or(0.0, Vec3::magnitude)
}

/// Normalize in place unless (almost) zero
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec3_normalize(v: *mut Vec3) {
    // SAFETY: the C contract: NULL or a Vec3_t
    if let Some(v) = unsafe { ptr::out(v) } {
        v.normalize();
    }
}

/// result = a x b, component by component as the C (so an output that is
/// also an input gives the C's result)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec3_cross(a: *const Vec3, b: *const Vec3, result: *mut Vec3) {
    if a.is_null() || b.is_null() || result.is_null() {
        return;
    }
    // SAFETY: the C contract: three valid Vec3_t, possibly the same; read
    // and written through the raw pointers only, in the C's order
    unsafe {
        (*result).x = (*a).y * (*b).z - (*a).z * (*b).y;
        (*result).y = (*a).z * (*b).x - (*a).x * (*b).z;
        (*result).z = (*a).x * (*b).y - (*a).y * (*b).x;
    }
}

/// a . b (0 for NULL)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec3_dot(a: *const Vec3, b: *const Vec3) -> f64 {
    // SAFETY: the C contract: NULL or Vec3_t
    match unsafe { (ptr::in_(a), ptr::in_(b)) } {
        (Some(a), Some(b)) => a.dot(b),
        _ => 0.0,
    }
}

/// Limit a dipole's magnitude, keeping its direction
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_saturate_dipole(dipole: *mut Vec3, max_dipole: f64) {
    // SAFETY: the C contract: NULL or a Vec3_t
    if let Some(d) = unsafe { ptr::out(dipole) } {
        adcs::saturate_dipole(d, max_dipole);
    }
}

/*---------------------------------------------------------------------------
 * ADCS
 *-------------------------------------------------------------------------*/

/// Initialize: IDLE, coils off
#[unsafe(no_mangle)]
pub extern "C" fn adcs_init() -> i32 {
    if ADCS.with(|m| m.adcs.init()).is_none() {
        return rc::BUSY;
    }
    c::console(format_args!("[ADCS] Initialized in IDLE mode"));
    rc::OK
}

/// One control cycle
#[unsafe(no_mangle)]
pub extern "C" fn adcs_update(now_ms: u64) -> i32 {
    let read_ms = c::now_ms();
    let cycle = match ADCS.with(|m| m.adcs.update(now_ms, read_ms, m.sim_mag, m.sim_sun)) {
        Some(Some(cycle)) => cycle,
        Some(None) => return rc::ERROR,
        None => return rc::BUSY,
    };
    if cycle.detumble_timeout {
        c::log(FAULT_TYPE_ADCS, severity::WARNING, c"Detumble timeout", 0.0);
    }
    if cycle.detumble_complete {
        c::console(format_args!("[ADCS] Detumble complete!"));
    }
    rc::OK
}

/// Set the mode (one of the five)
#[unsafe(no_mangle)]
pub extern "C" fn adcs_set_mode(mode: u32) -> i32 {
    let now = c::now_ms();
    match ADCS.with(|m| m.adcs.set_mode(mode, now)) {
        Some(Ok(change)) => {
            if change.detumble_started {
                c::console(format_args!("[ADCS] Starting detumble sequence"));
            }
            c::console(format_args!("[ADCS] Mode changed: {} -> {}", change.from, change.to));
            rc::OK
        }
        Some(Err(AdcsError::NotInitialized)) => rc::ERROR,
        Some(Err(AdcsError::InvalidMode)) => rc::INVALID,
        None => rc::BUSY,
    }
}

/// The mode (SAFE before init)
#[unsafe(no_mangle)]
pub extern "C" fn adcs_get_mode() -> u32 {
    ADCS.with(|m| m.adcs.mode()).unwrap_or(adcs::mode::SAFE)
}

/// The module state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_get_state(state: *mut State) -> i32 {
    // SAFETY: the C contract: NULL or an AdcsState_t (adcs_control.h)
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    match ADCS.with(|m| m.adcs.initialized.then_some(m.adcs.state)) {
        Some(Some(s)) => {
            *out = s;
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Telemetry
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_get_telemetry(telemetry: *mut Telemetry) -> i32 {
    // SAFETY: the C contract: NULL or an AdcsTelemetry_t
    let Some(out) = (unsafe { ptr::out(telemetry) }) else { return rc::NULL_PTR };
    match ADCS.with(|m| m.adcs.initialized.then(|| m.adcs.telemetry())) {
        Some(Some(t)) => {
            *out = t;
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Whether the detumble completed
#[unsafe(no_mangle)]
pub extern "C" fn adcs_is_detumbled() -> bool {
    ADCS.with(|m| m.adcs.initialized && m.adcs.state.detumble_complete).unwrap_or(false)
}

/// Whether the sun is visible
#[unsafe(no_mangle)]
pub extern "C" fn adcs_sun_visible() -> bool {
    ADCS.with(|m| m.adcs.initialized && m.adcs.state.sun.sun_visible).unwrap_or(false)
}

/// Estimated rate magnitude, rad/s
#[unsafe(no_mangle)]
pub extern "C" fn adcs_get_rate_magnitude() -> f64 {
    ADCS.with(|m| if m.adcs.initialized { m.adcs.state.rate_magnitude } else { 0.0 }).unwrap_or(0.0)
}

/*---------------------------------------------------------------------------
 * Sensors (simulated)
 *-------------------------------------------------------------------------*/

/// Read the magnetometer (field, time, valid; the rate is left alone)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_read_magnetometer(data: *mut MagData) -> i32 {
    if data.is_null() {
        return rc::NULL_PTR;
    }
    let now = c::now_ms();
    let Some(field) = ADCS.with(|m| m.sim_mag) else { return rc::BUSY };
    // SAFETY: the C contract: a MagData_t; its fields are written through
    // the raw pointer (the rest may be uninitialized)
    unsafe {
        (*data).field = field;
        (*data).timestamp_ms = now;
        (*data).valid = true;
    }
    rc::OK
}

/// Read the sun sensors
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_read_sun_sensors(data: *mut SunSensorData) -> i32 {
    if data.is_null() {
        return rc::NULL_PTR;
    }
    let now = c::now_ms();
    let Some(sun) = ADCS.with(|m| m.sim_sun) else { return rc::BUSY };
    let mut reading = SunSensorData::default();
    adcs::read_sun_sensors(&mut reading, sun, now);
    // SAFETY: the C contract: a SunSensorData_t; every field is written
    unsafe { data.write(reading) };
    rc::OK
}

/// Set the simulated sensors (NULL leaves a value alone)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_set_sim_sensors(mag: *const Vec3, sun: *const Vec3, rate: *const Vec3) {
    // SAFETY: the C contract: NULL or Vec3_t
    let (mag, sun, rate) = unsafe { (ptr::in_(mag).copied(), ptr::in_(sun).copied(), ptr::in_(rate).copied()) };
    let _ = ADCS.with(|m| {
        if let Some(v) = mag {
            m.sim_mag = v;
        }
        if let Some(v) = sun {
            m.sim_sun = v;
        }
        if let Some(v) = rate {
            m.sim_rate = v;
        }
    });
}

/*---------------------------------------------------------------------------
 * Control
 *-------------------------------------------------------------------------*/

/// B-dot dipole command
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_compute_bdot(mag: *const MagData, dipole: *mut Vec3) {
    // SAFETY: the C contract: NULL or a MagData_t / Vec3_t
    if let (Some(mag), false) = (unsafe { ptr::in_(mag) }, dipole.is_null()) {
        let d = adcs::bdot(mag);
        // SAFETY: non-NULL Vec3_t
        unsafe { dipole.write(d) };
    }
}

/// Sun-pointing dipole command
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_compute_sunpoint(sun: *const SunSensorData, mag: *const MagData, dipole: *mut Vec3) {
    // SAFETY: the C contract: NULL or a SunSensorData_t / MagData_t / Vec3_t
    if let (Some(sun), Some(mag), false) = (unsafe { ptr::in_(sun) }, unsafe { ptr::in_(mag) }, dipole.is_null()) {
        let d = adcs::sunpoint(sun, mag);
        // SAFETY: non-NULL Vec3_t
        unsafe { dipole.write(d) };
    }
}

/// Command the coils
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_apply_dipole(dipole: *const Vec3) -> i32 {
    // SAFETY: the C contract: NULL or a Vec3_t
    let Some(d) = (unsafe { ptr::in_(dipole) }).copied() else { return rc::NULL_PTR };
    match ADCS.with(|m| m.adcs.initialized.then(|| m.adcs.apply_dipole(&d))) {
        Some(Some(())) => rc::OK,
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Coils off
#[unsafe(no_mangle)]
pub extern "C" fn adcs_disable_mtq() -> i32 {
    match ADCS.with(|m| m.adcs.initialized.then(|| m.adcs.disable_mtq())) {
        Some(Some(())) => rc::OK,
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}
