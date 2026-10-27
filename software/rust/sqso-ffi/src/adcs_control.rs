//! `adcs_control.h`: attitude control (`sqso_fsw::adcs`).
//!
//! Sensors: on the target the OBC board's (`sqso_fsw::adcs_sensors` through
//! [`crate::hw`]: the M-G370PDT0 with the BMI088 as cold spare, the RM3100
//! triggered in the coils-off window, the ADS7138 sun-sensor ADC); on the
//! host the simulated values `adcs_set_sim_sensors()` sets (the C suites,
//! the mission simulation's models). Either way the calibration
//! (`FLASH_REGION_ADCS_CAL`: the gyro alignment matrix, the magnetometer
//! bias per polarity state) is applied, and a sample taken while
//! `GPIO_PIN_JET_PG` or `GPIO_PIN_PTT_EXC` is high is not used for
//! actuation (RID GNC-08).
//!
//! Coils: each command is driven through `hal_mtq.h` (the direction lines
//! first, then the duties and the hardware-timed burst; a zero command
//! holds MTQ_ARM low), and the coil-current readbacks are checked at the
//! magnetometer sample.
//!
//! @requirement SRS-F141 Magnetorquer drive: PH before ARM, PWM duty, hardware-timed burst
//! @requirement SRS-F142 Gyro alignment correction: a persisted 3×3 matrix applied to the gyro rates, identity by default, set by telecommand
//! @requirement SRS-F144 Magnetometer bias per polarity state of the three rods, zeros by default, set by telecommand

use sqso_fsw::adcs::{self, Adcs, AdcsError, Calibration, MagData, Reading, State, SunSensorData, Telemetry, Vec3};
use sqso_fsw::condition::{Change, Condition};
use sqso_fsw::nvm::{self, RecordStatus};

use crate::c::{self, HalFlash, rc, severity};
use crate::cell::Global;
use crate::{hw, pins, ptr};

/// The calibration records ("GYAL", "MGBS") and where they sit in the region
const GYRO_MAGIC: u32 = 0x4759_414C;
const BIAS_MAGIC: u32 = 0x4D47_4253;
const GYRO_BASE: u32 = 0;
const BIAS_BASE: u32 = nvm::FOOTPRINT;

/// `FAULT_TYPE_ADCS`
pub(crate) const FAULT_TYPE_ADCS: u32 = 17;

struct Module {
    adcs: Adcs,
    /// Simulated magnetometer field, uT
    sim_mag: Vec3,
    /// Simulated sun vector (its length is the intensity)
    sim_sun: Vec3,
    /// Simulated gyro body rate, rad/s (`None` until one is set: then the
    /// rate is estimated from the field)
    sim_rate: Option<Vec3>,
    /// The OBC board's sensors (target)
    #[cfg_attr(not(target_os = "none"), allow(dead_code))]
    sensors: sqso_fsw::adcs_sensors::Sensors,
    /// Bridge faults and off-timer expiries, logged at onset and clearance
    mtq_fault: Condition,
    mtq_expired: Condition,
    /// Their episodes counted
    mtq_counts: (u32, u32),
    /// The coils were not off at the magnetometer sample (logged once)
    coil_current: Condition,
}

static ADCS: Global<Module> = Global::new(Module {
    adcs: Adcs::new(),
    sim_mag: Vec3::new(30.0, 0.0, 40.0), // typical LEO field
    sim_sun: Vec3::new(1.0, 0.0, 0.0),
    sim_rate: None,
    sensors: sqso_fsw::adcs_sensors::Sensors::new(),
    mtq_fault: Condition::new(),
    mtq_expired: Condition::new(),
    mtq_counts: (0, 0),
    coil_current: Condition::new(),
});

/// The stored calibration, identity and zeros where a record is missing
fn load_calibration() -> Calibration {
    let mut cal = Calibration::IDENTITY;
    let mut g = [0u8; 36];
    if nvm::load(&mut HalFlash, pins::FLASH_REGION_ADCS_CAL, GYRO_BASE, GYRO_MAGIC, &mut g) == Ok(RecordStatus::Valid)
        && let Some(m) = Calibration::gyro_from_record(&g)
    {
        cal.gyro = m;
    }
    let mut b = [0u8; 48];
    if nvm::load(&mut HalFlash, pins::FLASH_REGION_ADCS_CAL, BIAS_BASE, BIAS_MAGIC, &mut b) == Ok(RecordStatus::Valid)
        && let Some(t) = Calibration::bias_from_record(&b)
    {
        cal.mag_bias = t;
    }
    cal
}

/// ADCS_SET_GYRO_ALIGN: a new alignment matrix (rows), persisted;
/// `INVALID` for one [`Calibration::gyro_valid`] refuses, `IO` if the
/// record cannot be written (the RAM copy is then unchanged)
pub fn set_gyro_alignment(m: [[f64; 3]; 3]) -> i32 {
    if !Calibration::gyro_valid(&m) {
        return rc::INVALID;
    }
    let Some(mut cal) = ADCS.with(|a| a.adcs.calibration) else { return rc::BUSY };
    cal.gyro = m;
    if nvm::save(&mut HalFlash, pins::FLASH_REGION_ADCS_CAL, GYRO_BASE, GYRO_MAGIC, &cal.gyro_record()).is_err() {
        return rc::IO;
    }
    // The stored f32 values are the ones flown
    cal.gyro = Calibration::gyro_from_record(&cal.gyro_record()).unwrap_or(m);
    let _ = ADCS.with(|a| a.adcs.calibration = cal);
    c::console(format_args!("[ADCS] Gyro alignment set"));
    rc::OK
}

/// ADCS_SET_MAG_BIAS: the bias of one polarity state (0–7), µT, persisted
pub fn set_mag_bias(state: u8, bias: Vec3) -> i32 {
    if usize::from(state) >= adcs::POLARITY_STATES || !Calibration::bias_valid(&bias) {
        return rc::INVALID;
    }
    let Some(mut cal) = ADCS.with(|a| a.adcs.calibration) else { return rc::BUSY };
    if let Some(slot) = cal.mag_bias.get_mut(usize::from(state)) {
        *slot = bias;
    }
    if nvm::save(&mut HalFlash, pins::FLASH_REGION_ADCS_CAL, BIAS_BASE, BIAS_MAGIC, &cal.bias_record()).is_err() {
        return rc::IO;
    }
    cal.mag_bias = Calibration::bias_from_record(&cal.bias_record()).unwrap_or(cal.mag_bias);
    let _ = ADCS.with(|a| a.adcs.calibration = cal);
    c::console(format_args!("[ADCS] Magnetometer bias {state} set"));
    rc::OK
}

/// ADCS_SET_GYRO_ALIGN's 36 argument bytes: nine big-endian `i32` in
/// units of 1e-6, row by row
pub fn gyro_alignment_command(args: &[u8]) -> i32 {
    let mut m = [[0.0f64; 3]; 3];
    if args.len() != 36 {
        return rc::INVALID;
    }
    for (dst, src) in m.iter_mut().flatten().zip(args.chunks_exact(4)) {
        let mut w = [0u8; 4];
        w.copy_from_slice(src);
        *dst = f64::from(i32::from_be_bytes(w)) * 1e-6;
    }
    set_gyro_alignment(m)
}

/// ADCS_SET_MAG_BIAS's 13 argument bytes: the state, then three
/// big-endian `i32` in nT
pub fn mag_bias_command(args: &[u8]) -> i32 {
    let [state, rest @ ..] = args else { return rc::INVALID };
    if rest.len() != 12 {
        return rc::INVALID;
    }
    let mut v = [0.0f64; 3];
    for (dst, src) in v.iter_mut().zip(rest.chunks_exact(4)) {
        let mut w = [0u8; 4];
        w.copy_from_slice(src);
        *dst = f64::from(i32::from_be_bytes(w)) * 1e-3;
    }
    set_mag_bias(*state, Vec3::new(v[0], v[1], v[2]))
}

/// The calibration in use
pub fn calibration() -> Calibration {
    ADCS.with(|a| a.adcs.calibration).unwrap_or(Calibration::IDENTITY)
}

/// Drive the coils for the command in the state: the direction lines,
/// then the hardware-timed burst; a zero or disabled command holds the
/// bridges asleep (`hal_mtq_off()`)
fn drive_coils(mtq: &adcs::MtqCommand) {
    let drive = sqso_fsw::mtq::drive(&mtq.dipole, hw::adc(pins::ADC_VBATT));
    if !mtq.enabled || drive.is_zero() {
        // SAFETY: no arguments (the HAL)
        let _ = unsafe { c::hal_mtq_off() };
        return;
    }
    for (&pin, &positive) in pins::MTQ_DIR.iter().zip(drive.positive.iter()) {
        hw::set(pin, positive);
    }
    #[allow(clippy::cast_possible_truncation)] // 900 ms
    let on = adcs::MTQ_ON_MS as u32;
    // SAFETY: three u16 (the HAL)
    let _ = unsafe { c::hal_mtq_burst(drive.duty.as_ptr(), on) };
}

/// The bridges' fault line and the off-timer, logged on onset and
/// clearance
fn check_bridges() {
    let fault = !hw::level(pins::MTQ_FAULT_N);
    let expired = hw::level(pins::MTQ_EXPIRED);
    let Some((f, e)) = ADCS.with(|a| {
        let f = a.mtq_fault.check(fault);
        let e = a.mtq_expired.check(expired);
        if f == Change::Onset {
            a.mtq_counts.0 = a.mtq_counts.0.wrapping_add(1);
        }
        if e == Change::Onset {
            a.mtq_counts.1 = a.mtq_counts.1.wrapping_add(1);
        }
        (f, e)
    }) else {
        return;
    };
    if f == Change::Onset {
        c::log(FAULT_TYPE_ADCS, severity::WARNING, c"Magnetorquer bridge fault (MTQ_FAULT_N)", 0.0);
    }
    if e == Change::Onset {
        c::log(FAULT_TYPE_ADCS, severity::ERROR, c"Magnetorquer off-timer expired: burst timer failed", 0.0);
    }
}

/// Magnetorquer bridge-fault and off-timer-expiry episodes since boot
pub fn mtq_faults() -> (u32, u32) {
    ADCS.with(|a| a.mtq_counts).unwrap_or((0, 0))
}

/// Magnetometer samples gated (Jetson on, exciter keyed) since init
pub fn mag_gated() -> u32 {
    ADCS.with(|a| a.adcs.mag_gated).unwrap_or(0)
}

/// The gyro in use (0 M-G370, 1 BMI088, 2 none) and the rail trips
pub fn sensor_status() -> (u8, [u32; 3]) {
    use sqso_fsw::adcs_sensors::Gyro;
    ADCS.with(|a| {
        let g = match a.sensors.gyro {
            Gyro::Primary => 0,
            Gyro::Backup => 1,
            Gyro::None => 2,
        };
        (g, a.sensors.trips)
    })
    .unwrap_or((0, [0; 3]))
}

/// Every pass: the OBC board's sensors (rails, bring-up, the
/// magnetometer's trigger in the coils-off window). The host build has the
/// simulated sensors instead and does nothing here
pub fn adcs_poll(now_ms: u64, powered: bool) {
    #[cfg(target_os = "none")]
    target::poll(now_ms, powered);
    #[cfg(not(target_os = "none"))]
    let _ = (now_ms, powered);
}

/// The readings of a control cycle: the board's sensors on the target, the
/// simulated ones on the host
fn cycle_reading(m: &mut Module) -> Reading {
    #[cfg(target_os = "none")]
    {
        target::read(m)
    }
    #[cfg(not(target_os = "none"))]
    {
        Reading { field: m.sim_mag, sun: m.sim_sun, gyro: m.sim_rate, ..Reading::default() }
    }
}

#[cfg(target_os = "none")]
mod target {
    use sqso_fsw::adcs::{Reading, Vec3};
    use sqso_fsw::adcs_sensors::{Devices, Events};

    use super::{ADCS, FAULT_TYPE_ADCS, Module};
    use crate::c::{self, severity};
    use crate::hw::{HalI2c, HalPins, HalSpi};
    use crate::pins;

    fn log(ev: &Events) {
        let texts: [(bool, &core::ffi::CStr); 6] = [
            (ev.tripped[0], c"Gyro rail latch-up trip"),
            (ev.tripped[1], c"BMI088 rail latch-up trip"),
            (ev.tripped[2], c"Sensor rail latch-up trip"),
            (ev.mg370_failed.is_some(), c"Gyro M-G370 failed: rail cycled"),
            (ev.bmi088_failed.is_some(), c"Gyro BMI088 failed: rail cycled"),
            (ev.sns_failed, c"Magnetometer or sun-sensor ADC not answering"),
        ];
        for (on, text) in texts {
            if on {
                c::log(FAULT_TYPE_ADCS, severity::WARNING, text, 0.0);
            }
        }
        if ev.to_backup {
            c::log(FAULT_TYPE_ADCS, severity::ERROR, c"Gyro M-G370 given up: BMI088 in use", 0.0);
        }
        if ev.no_gyro {
            c::log(FAULT_TYPE_ADCS, severity::ERROR, c"No gyro: magnetometer rate estimate in use", 0.0);
        }
    }

    pub(super) fn poll(now_ms: u64, powered: bool) {
        let ev = ADCS.with(|m| {
            let (mut g, mut b, mut a, mut i, mut p) = (
                HalSpi(pins::SPI_MG370),
                HalSpi(pins::SPI_BMI088_GYRO),
                HalSpi(pins::SPI_BMI088_ACCEL),
                HalI2c(pins::I2C_ADCS),
                HalPins,
            );
            let mut d = Devices { mg370: &mut g, bmi_gyro: &mut b, bmi_accel: &mut a, i2c: &mut i, pins: &mut p };
            let ev = m.sensors.step(now_ms, powered, &mut d);
            // The window opens 50 ms after the burst ends; without one, 50 ms
            // before the next cycle
            let last = m.adcs.state.mag.timestamp_ms;
            let window = m.adcs.coils_off_ms.filter(|&off| off > last).map_or(
                last.saturating_add(sqso_fsw::adcs::CONTROL_PERIOD_MS - sqso_fsw::adcs::MAG_SETTLE_MS),
                |off| off.saturating_add(m.adcs.timing.settle_ms),
            );
            m.sensors.trigger_mag(now_ms, window, &mut i);
            ev
        });
        if let Some(ev) = ev {
            log(&ev);
        }
    }

    pub(super) fn read(m: &mut Module) -> Reading {
        let (mut g, mut b, mut a, mut i, mut p) = (
            HalSpi(pins::SPI_MG370),
            HalSpi(pins::SPI_BMI088_GYRO),
            HalSpi(pins::SPI_BMI088_ACCEL),
            HalI2c(pins::I2C_ADCS),
            HalPins,
        );
        let mut d = Devices { mg370: &mut g, bmi_gyro: &mut b, bmi_accel: &mut a, i2c: &mut i, pins: &mut p };
        let r = m.sensors.read(&mut d);
        let v = |x: [f64; 3]| Vec3::new(x[0], x[1], x[2]);
        Reading {
            field: r.field.map_or(Vec3::ZERO, v),
            sun: r.sun.map_or(Vec3::ZERO, v),
            gyro: r.rate.map(v),
            mag_missing: r.field.is_none(),
            ..Reading::default()
        }
    }
}

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

/// Initialize: IDLE, coils off, the stored calibration, the coil and
/// sensor pins configured
#[unsafe(no_mangle)]
pub extern "C" fn adcs_init() -> i32 {
    let cal = load_calibration();
    if ADCS
        .with(|m| {
            m.adcs.calibration = cal;
            m.adcs.init();
        })
        .is_none()
    {
        return rc::BUSY;
    }
    for pin in pins::MTQ_DIR {
        hw::configure(pin, true);
    }
    hw::configure(pins::MTQ_FAULT_N, false);
    hw::configure(pins::MTQ_EXPIRED, false);
    hw::HalPins::configure();
    // SAFETY: no arguments (the HAL)
    let _ = unsafe { c::hal_mtq_off() };
    c::console(format_args!("[ADCS] Initialized in IDLE mode"));
    rc::OK
}

/// One control cycle: the readings with the gates (`JET_PG`, `PTT_EXC`)
/// and the coil-current check, the control law, then the coils
#[unsafe(no_mangle)]
pub extern "C" fn adcs_update(now_ms: u64) -> i32 {
    let read_ms = c::now_ms();
    let gated = hw::level(pins::JET_PG) || hw::level(pins::PTT_EXC);
    let currents = pins::ADC_MTQ_I.map(hw::adc);
    let driven = !sqso_fsw::mtq::coils_off(currents);
    let step = ADCS.with(|m| {
        let r = Reading { gated, coils_driven: driven, ..cycle_reading(m) };
        let new_current = m.coil_current.check(driven) == Change::Onset;
        m.adcs.update_cycle(now_ms, read_ms, &r).map(|c| (c, m.adcs.mag_rejected, m.adcs.state.mtq, new_current))
    });
    let (cycle, rejected, mtq, new_current) = match step {
        Some(Some(cycle)) => cycle,
        Some(None) => return rc::ERROR,
        None => return rc::BUSY,
    };
    drive_coils(&mtq);
    check_bridges();
    if new_current {
        c::log(FAULT_TYPE_ADCS, severity::WARNING, c"Coil current at the magnetometer sample", 0.0);
    }
    if cycle.mag_rejected && rejected == 1 {
        // The first since init: the coil timing is wrong (the rest are counted)
        c::log(FAULT_TYPE_ADCS, severity::WARNING, c"Magnetometer read with coils on", 0.0);
    }
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

/// Use spin-axis and inertia preset `preset` (`Geometry::PRESETS`); `INVALID`
/// for one that does not exist
pub fn set_geometry(preset: u8) -> i32 {
    let Some(g) = sqso_fsw::adcs::Geometry::preset(preset) else { return rc::INVALID };
    match ADCS.with(|m| m.adcs.geometry = g) {
        Some(()) => {
            c::console(format_args!("[ADCS] Geometry preset {preset}"));
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Magnetometer samples rejected since init (coils on or not settled)
pub fn mag_rejected() -> u32 {
    ADCS.with(|m| m.adcs.mag_rejected).unwrap_or(0)
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
            m.sim_rate = Some(v);
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

/// Command the coils for one burst (the driver ends it after
/// `sqso_fsw::adcs::MTQ_ON_MS`)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn adcs_apply_dipole(dipole: *const Vec3) -> i32 {
    // SAFETY: the C contract: NULL or a Vec3_t
    let Some(d) = (unsafe { ptr::in_(dipole) }).copied() else { return rc::NULL_PTR };
    let now = c::now_ms();
    match ADCS.with(|m| {
        m.adcs.initialized.then(|| {
            m.adcs.apply_burst(&d, now);
            m.adcs.state.mtq
        })
    }) {
        Some(Some(mtq)) => {
            drive_coils(&mtq);
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Coils off
#[unsafe(no_mangle)]
pub extern "C" fn adcs_disable_mtq() -> i32 {
    match ADCS.with(|m| m.adcs.initialized.then(|| m.adcs.disable_mtq())) {
        Some(Some(())) => {
            // SAFETY: no arguments (the HAL)
            let _ = unsafe { c::hal_mtq_off() };
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    fn be(v: &[f64], scale: f64) -> Vec<u8> {
        #[allow(clippy::cast_possible_truncation)] // test values
        v.iter().flat_map(|x| ((x * scale).round() as i32).to_be_bytes()).collect()
    }

    /// ADCS_SET_GYRO_ALIGN and ADCS_SET_MAG_BIAS: validated, persisted in
    /// FLASH_REGION_ADCS_CAL, loaded at init; a record that cannot be
    /// written leaves the RAM copy; an erased region is the identity
    #[test]
    fn calibration_commands_persist_and_validate() {
        let _serial = serial();
        assert_eq!(crate::hal_sim::hal_flash_init(), rc::OK);
        assert_eq!(crate::hal_sim::hal_flash_erase(pins::FLASH_REGION_ADCS_CAL), rc::OK);
        assert_eq!(adcs_init(), rc::OK);
        assert_eq!(calibration(), Calibration::IDENTITY);

        let (c, s) = (0.999_95_f64, 0.01_f64);
        let m = [c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0];
        assert_eq!(gyro_alignment_command(be(&m, 1e6).get(..35).unwrap_or(&[])), rc::INVALID, "length");
        let twice = [2.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 2.0];
        assert_eq!(gyro_alignment_command(&be(&twice, 1e6)), rc::INVALID, "not a rotation");
        assert_eq!(gyro_alignment_command(&be(&m, 1e6)), rc::OK);
        let close = |a: f64, b: f64| (a - b).abs() < 1e-6;
        assert!(close(calibration().gyro[0][1], -s));
        assert!(close(calibration().gyro[1][0], s));

        assert_eq!(mag_bias_command(&[]), rc::INVALID, "no state");
        assert_eq!(mag_bias_command(&[3, 0, 0]), rc::INVALID, "length");
        let mut bias = vec![3u8];
        bias.extend(be(&[1000.0, -2000.0, 500.0], 1.0));
        let mut bad_state = bias.clone();
        if let Some(state) = bad_state.first_mut() {
            *state = 8;
        }
        assert_eq!(mag_bias_command(&bad_state), rc::INVALID, "eight states");
        let mut big = vec![1u8];
        big.extend(be(&[60_000.0, 0.0, 0.0], 1.0));
        assert_eq!(mag_bias_command(&big), rc::INVALID, "beyond 50 uT");
        assert_eq!(mag_bias_command(&bias), rc::OK);
        let b = calibration().mag_bias[3];
        assert!(close(b.x, 1.0));
        assert!(close(b.y, -2.0));
        assert!(close(b.z, 0.5));

        // Persisted: init loads both records
        let before = calibration();
        let _ = ADCS.with(|a| a.adcs.calibration = Calibration::IDENTITY);
        assert_eq!(adcs_init(), rc::OK);
        assert_eq!(calibration(), before);

        // The record cannot be written: refused, the RAM copy unchanged
        crate::hal_sim::hal_sim_flash_fail_writes(true);
        assert_eq!(set_gyro_alignment([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]), rc::IO);
        assert_eq!(set_mag_bias(0, Vec3::new(1.0, 1.0, 1.0)), rc::IO);
        crate::hal_sim::hal_sim_flash_fail_writes(false);
        assert_eq!(calibration(), before);

        // In use (a nested call): busy
        let nested = ADCS.with(|_| (set_gyro_alignment(before.gyro), set_mag_bias(0, Vec3::new(0.0, 0.0, 0.0))));
        assert_eq!(nested, Some((rc::BUSY, rc::BUSY)));

        assert_eq!(crate::hal_sim::hal_flash_erase(pins::FLASH_REGION_ADCS_CAL), rc::OK);
        assert_eq!(adcs_init(), rc::OK);
        assert_eq!(calibration(), Calibration::IDENTITY, "erased: identity and zeros");
    }

    /// The bridges' fault and off-timer lines: counted and logged once per
    /// episode
    #[test]
    fn bridge_faults_are_counted_per_episode() {
        let _serial = serial();
        assert_eq!(crate::hal_sim::hal_gpio_init(), rc::OK);
        assert_eq!(adcs_init(), rc::OK);
        let (f0, e0) = mtq_faults();
        crate::hal_sim::hal_sim_gpio_drive_input(pins::MTQ_FAULT_N, false);
        check_bridges();
        check_bridges();
        assert_eq!(mtq_faults(), (f0 + 1, e0));
        crate::hal_sim::hal_sim_gpio_drive_input(pins::MTQ_FAULT_N, true);
        crate::hal_sim::hal_sim_gpio_drive_input(pins::MTQ_EXPIRED, true);
        check_bridges();
        check_bridges();
        assert_eq!(mtq_faults(), (f0 + 1, e0 + 1));
        crate::hal_sim::hal_sim_gpio_drive_input(pins::MTQ_EXPIRED, false);
        crate::hal_sim::hal_sim_gpio_drive_input(pins::MTQ_FAULT_N, false);
        check_bridges();
        assert_eq!(mtq_faults(), (f0 + 2, e0 + 1), "a second episode");
        crate::hal_sim::hal_sim_gpio_drive_input(pins::MTQ_FAULT_N, true);
        check_bridges();
        assert_eq!(ADCS.with(|_| check_bridges()), Some(()), "busy: nothing");
    }

    /// The magnetometer is not used while the Jetson is powered or the
    /// exciter keyed
    #[test]
    fn the_magnetometer_is_gated() {
        let _serial = serial();
        assert_eq!(crate::hal_sim::hal_gpio_init(), rc::OK);
        assert_eq!(adcs_init(), rc::OK);
        let before = mag_gated();
        crate::hal_sim::hal_sim_gpio_drive_input(pins::JET_PG, true);
        let _ = adcs_update(c::now_ms());
        crate::hal_sim::hal_sim_gpio_drive_input(pins::JET_PG, false);
        crate::hal_sim::hal_sim_gpio_drive_input(pins::PTT_EXC, true);
        let _ = adcs_update(c::now_ms() + 1000);
        crate::hal_sim::hal_sim_gpio_drive_input(pins::PTT_EXC, false);
        assert_eq!(mag_gated(), before + 2);
        let _ = adcs_update(c::now_ms() + 2000);
        assert_eq!(mag_gated(), before + 2, "neither: used");
    }
}
