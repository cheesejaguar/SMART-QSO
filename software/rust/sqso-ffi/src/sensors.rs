//! `sensors.h`: the sensor table, its configuration file (`SENSORS_YAML`
//! in the host simulation), polling and the telemetry text
//! (`sqso_fsw::sensors`).
//!
//! The readings are simulated from the environment the executive sets,
//! the EPS switches and noise (C `rand()` on the host, as before; a xorshift
//! generator on the target, which has no C library). The configuration file
//! is parsed by `sqso_fsw::sensors::YamlParser`, which is bounded and
//! fuzzed; there is no file system on the target, where loading it fails
//! with `SMART_QSO_ERROR_IO` (the target uses the built-in table).
//!
//! Each `Sensor_t.read` points to one of the `extern "C"` readers below, so
//! a C caller can still call it on a copy. Readers, the EPS and the fault
//! log are called with no sensor state borrowed.
//!
//! The payload spreader temperature (`PT`) is the OBC's own thermistor on
//! `ADC_CHANNEL_PT_NTC` (`sqso_fsw::sensors::spreader_temperature`): an
//! open or shorted thermistor reads NaN and raises the spreader sensor
//! fault ([`spreader_fault`]), which inhibits the Jetson's power-on. The
//! battery alarms are logged at onset and clearance (`sqso_fsw::condition`,
//! RID FSW-14), not at every reading.
//!
//! @requirement SRS-F150 Spreader thermistor: R = 10.0 kΩ N / (4095 − N), Steinhart–Hart 10K3A; outside 0.10–3.25 V a sensor fault that inhibits the Jetson's power-on
//! @requirement SRS-F153 A repeated condition shall be logged at its onset and its clearance (with the number of checks), not at every check
//! @requirement SRS-F070 Collect telemetry from all sensors
//! @requirement SRS-F071 Sample sensors at configurable rates
//! @requirement SRS-F072 Validate sensor readings against limits

use core::ffi::c_char;

use sqso_fsw::sensors::{
    self, Alarm, CHANNEL_LEN, Environment, ID_LEN, Kind, MAX_SENSORS, NAME_LEN, SensorDef, Snprintf, StatusInputs,
    TEXT_LEN, TYPE_LEN, UNITS_LEN, Value, ValueType, until_nul,
};

use core::sync::atomic::{AtomicBool, Ordering};

use sqso_fsw::condition::Condition;

use crate::c::{self, rc, severity};
use crate::cell::Global;
use crate::{eps_control, fault_mgmt, hw, pins, ptr};
use sqso_fsw::eps_telemetry::Telemetry;

#[cfg(not(target_os = "none"))]
unsafe extern "C" {
    // C library pseudo-random numbers (seeded by the executive for
    // simulation noise)
    fn rand() -> core::ffi::c_int;
}

/// `RAND_MAX` of glibc
#[cfg(not(target_os = "none"))]
const RAND_MAX: f64 = 2_147_483_647.0;

/// `FaultType_t` values used here
const FAULT_VOLTAGE_LOW: u32 = 7;
const FAULT_CURRENT_HIGH: u32 = 9;

/// `SensorReadFn_t`
pub type SensorReadFn = unsafe extern "C" fn(this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool;

/// `Sensor_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Sensor {
    id: [u8; ID_LEN],
    name: [u8; NAME_LEN],
    units: [u8; UNITS_LEN],
    pub(crate) type_: [u8; TYPE_LEN],
    channel: [u8; CHANNEL_LEN],
    period_ms: u32,
    next_poll_ms: u64,
    value_type: u32,
    read: Option<SensorReadFn>,
    pub(crate) last_value: f64,
    last_text: [u8; TEXT_LEN],
    pub(crate) last_read_ms: u64,
}

impl Sensor {
    pub(crate) const ZERO: Sensor = Sensor {
        id: [0; ID_LEN],
        name: [0; NAME_LEN],
        units: [0; UNITS_LEN],
        type_: [0; TYPE_LEN],
        channel: [0; CHANNEL_LEN],
        period_ms: 0,
        next_poll_ms: 0,
        value_type: 0,
        read: None,
        last_value: 0.0,
        last_text: [0; TEXT_LEN],
        last_read_ms: 0,
    };

    fn new(def: &SensorDef, kind: Kind) -> Sensor {
        Sensor {
            id: def.id,
            name: def.name,
            units: def.units,
            type_: def.type_,
            channel: def.channel,
            period_ms: def.period_ms,
            value_type: kind.value_type() as u32,
            read: Some(reader(kind)),
            ..Sensor::ZERO
        }
    }
}

struct Sensors {
    table: [Sensor; MAX_SENSORS],
    kinds: [Option<Kind>; MAX_SENSORS],
    count: usize,
    sunlit: bool,
    soc: f64,
}

static SENSORS: Global<Sensors> = Global::new(Sensors {
    table: [Sensor::ZERO; MAX_SENSORS],
    kinds: [None; MAX_SENSORS],
    count: 0,
    sunlit: true,
    soc: 0.75,
});

/// First reading of the elapsed timer (0: none yet)
static TIMER_START_MS: Global<u64> = Global::new(0);

/// The battery alarms' latches: low voltage, high discharge current
static ALARMS: Global<[Condition; 2]> = Global::new([Condition::new(); 2]);

/// The spreader thermistor reads outside its valid range
static SPREADER_FAULT: AtomicBool = AtomicBool::new(false);

/// Whether the spreader thermistor is faulted (open or shorted) at its
/// latest reading
pub fn spreader_fault() -> bool {
    SPREADER_FAULT.load(Ordering::Relaxed)
}

/// The spreader temperature from the OBC's thermistor, °C; NaN (and the
/// sensor fault) outside the valid range
fn spreader_c() -> f64 {
    let t = hw::adc_raw(pins::ADC_PT_NTC).and_then(|raw| sensors::spreader_temperature(raw, hw::adc(pins::ADC_PT_NTC)));
    SPREADER_FAULT.store(t.is_none(), Ordering::Relaxed);
    t.unwrap_or(f64::NAN)
}

fn with<R>(busy: R, f: impl FnOnce(&mut Sensors) -> R) -> R {
    SENSORS.with(f).unwrap_or(busy)
}

/// Add a configured sensor if its type is known and the table has room
fn add(def: &SensorDef) {
    let Some(kind) = def.kind() else { return };
    let _ = SENSORS.with(|s| {
        if let (Some(slot), Some(k)) = (s.table.get_mut(s.count), s.kinds.get_mut(s.count)) {
            *slot = Sensor::new(def, kind);
            *k = Some(kind);
            s.count += 1;
        }
    });
}

/// Noise in [0, 1]
#[cfg(not(target_os = "none"))]
fn rnd_unit() -> f64 {
    // SAFETY: no arguments
    f64::from(unsafe { rand() }) / RAND_MAX
}

/// Noise in [0, 1]: there is no C library on the target, so a xorshift32
/// generator (the simulated readings are a stand-in until the sensor
/// drivers exist, SW-01)
#[cfg(target_os = "none")]
fn rnd_unit() -> f64 {
    use core::sync::atomic::{AtomicU32, Ordering};
    static STATE: AtomicU32 = AtomicU32::new(0x2545_F491);
    let mut x = STATE.load(Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    STATE.store(x, Ordering::Relaxed);
    f64::from(x) / f64::from(u32::MAX)
}

/// A reading
#[derive(Clone, Copy)]
enum Reading {
    Numeric(f64),
    Text([u8; TEXT_LEN]),
}

/// An EPS or payload reading: the EPS controller's telemetry (and the OBC
/// thermistor) when the controller answers, the stand-in otherwise
fn eps_numeric(kind: Kind, channel: &[u8], measured: Option<&Telemetry>, env: &Environment) -> sensors::Numeric {
    let mut noise = rnd_unit;
    let plain = |value| sensors::Numeric { value, alarm: None };
    match (kind, measured) {
        (Kind::EpsVoltage, Some(t)) => sensors::measured_voltage(channel, t),
        (Kind::EpsVoltage, None) => sensors::eps_voltage(channel, env, &mut noise),
        (Kind::EpsCurrent, Some(t)) => sensors::measured_current(channel, t),
        (Kind::EpsCurrent, None) => sensors::eps_current(channel, env, &mut noise),
        (Kind::EpsTemperature, Some(t)) => plain(t.tbatt_c),
        (Kind::PayloadTemperature, Some(_)) => plain(spreader_c()),
        (Kind::PayloadTemperature, None) => plain(sensors::payload_temperature(env, &mut noise)),
        _ => plain(sensors::eps_temperature(env, &mut noise)),
    }
}

/// Take a reading of `kind` on `channel` (no sensor state borrowed)
fn read(kind: Kind, channel: &[u8]) -> Reading {
    // The EPS controller's telemetry when it answers, the stand-ins otherwise
    let measured = eps_control::eps_telemetry();
    let (sunlit, soc) = match measured {
        Some(t) => (t.sunlit(), t.soc),
        None => with((true, 0.75), |s| (s.sunlit, s.soc)),
    };
    let payload_enabled = eps_control::eps_is_payload_enabled();
    let env = Environment { sunlit, soc, payload_enabled };
    let channel = until_nul(channel);
    let numeric = match kind {
        Kind::SoftwareTimer => {
            let now = c::now_ms();
            let start = TIMER_START_MS
                .with(|start| {
                    if *start == 0 {
                        *start = now;
                    }
                    *start
                })
                .unwrap_or(now);
            sensors::Numeric { value: sensors::elapsed_s(start, c::now_ms()), alarm: None }
        }
        Kind::EpsVoltage | Kind::EpsCurrent | Kind::EpsTemperature | Kind::PayloadTemperature => {
            eps_numeric(kind, channel, measured.as_ref(), &env)
        }
        Kind::StatusHex2 => {
            let bits = sensors::status_bits(&StatusInputs {
                beacon_enabled: eps_control::eps_is_beacon_enabled(),
                payload_enabled,
                now_ms: c::now_ms(),
                sunlit,
                soc,
                safe_mode: eps_control::eps_get_power_mode() == 0,
                faults: fault_mgmt::fault_log_get_count() > 0,
                watchdog: fault_mgmt::fault_was_watchdog_triggered(),
            });
            return Reading::Text(sensors::hex2(bits));
        }
    };
    // Each alarm is checked by the reading that can raise it
    let checks = match (kind, channel) {
        (Kind::EpsVoltage, b"battery") => Some(0),
        (Kind::EpsCurrent, b"battery_discharge") => Some(1),
        _ => None,
    };
    if let Some(i) = checks {
        let present = match i {
            0 => numeric.alarm == Some(Alarm::LowBatteryVoltage),
            _ => numeric.alarm == Some(Alarm::HighDischargeCurrent),
        };
        let change = ALARMS.with(|a| a.get_mut(i).map(|cnd| cnd.check(present))).flatten();
        if let Some(change) = change {
            if i == 0 {
                c::log_condition(change, FAULT_VOLTAGE_LOW, severity::ERROR, c"Low battery voltage detected", soc);
            } else {
                c::log_condition(
                    change,
                    FAULT_CURRENT_HIGH,
                    severity::ERROR,
                    c"Excessive battery discharge current",
                    soc,
                );
            }
        }
    }
    Reading::Numeric(numeric.value)
}

/// `Sensor_t.read` for a kind: the sensor's channel (only the voltage and
/// current kinds look at it), the value or text out
///
/// # Safety
/// The C contract of `SensorReadFn_t`: `this` is NULL or a `Sensor_t`,
/// `out_value` NULL or a writable `double`, `out_text` NULL or a writable
/// buffer of 8 bytes (`Sensor_t.last_text`)
unsafe fn read_into(kind: Kind, this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool {
    let channel = match kind {
        Kind::EpsVoltage | Kind::EpsCurrent => {
            // SAFETY: the C contract: NULL or a Sensor_t
            match unsafe { ptr::in_(this.cast_const()) } {
                Some(s) => s.channel,
                None => return false,
            }
        }
        _ => [0; CHANNEL_LEN],
    };
    match read(kind, &channel) {
        Reading::Numeric(v) => {
            // SAFETY: the C contract: NULL or a double
            let Some(out) = (unsafe { ptr::out(out_value) }) else { return false };
            *out = v;
        }
        Reading::Text(t) => {
            // SAFETY: the C contract: NULL or 8 writable bytes ("XX" and a NUL written)
            let Some(out) = (unsafe { ptr::bytes_mut(out_text.cast(), 3) }) else { return false };
            out.copy_from_slice(t.get(..3).unwrap_or(&[0; 3]));
        }
    }
    true
}

unsafe extern "C" fn read_software_timer(this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool {
    // SAFETY: as read_into
    unsafe { read_into(Kind::SoftwareTimer, this, out_value, out_text) }
}

unsafe extern "C" fn read_eps_voltage(this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool {
    // SAFETY: as read_into
    unsafe { read_into(Kind::EpsVoltage, this, out_value, out_text) }
}

unsafe extern "C" fn read_eps_current(this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool {
    // SAFETY: as read_into
    unsafe { read_into(Kind::EpsCurrent, this, out_value, out_text) }
}

unsafe extern "C" fn read_eps_temperature(this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool {
    // SAFETY: as read_into
    unsafe { read_into(Kind::EpsTemperature, this, out_value, out_text) }
}

unsafe extern "C" fn read_payload_temperature(this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool {
    // SAFETY: as read_into
    unsafe { read_into(Kind::PayloadTemperature, this, out_value, out_text) }
}

unsafe extern "C" fn read_status_hex2(this: *mut Sensor, out_value: *mut f64, out_text: *mut c_char) -> bool {
    // SAFETY: as read_into
    unsafe { read_into(Kind::StatusHex2, this, out_value, out_text) }
}

fn reader(kind: Kind) -> SensorReadFn {
    match kind {
        Kind::SoftwareTimer => read_software_timer,
        Kind::EpsVoltage => read_eps_voltage,
        Kind::EpsCurrent => read_eps_current,
        Kind::EpsTemperature => read_eps_temperature,
        Kind::PayloadTemperature => read_payload_temperature,
        Kind::StatusHex2 => read_status_hex2,
    }
}

/// Store a reading in sensor `index`
fn store_reading(s: &mut Sensors, index: usize, reading: Reading, at_ms: u64) {
    if let Some(sensor) = s.table.get_mut(index) {
        match reading {
            Reading::Numeric(v) => sensor.last_value = v,
            Reading::Text(t) => sensors::copy_cstr(&mut sensor.last_text, &t),
        }
        sensor.last_read_ms = at_ms;
    }
}

/// The `[READ]` console line of sensor `index`, built from the table in
/// place. Not inlined: its line buffer is not in `sensors_poll`'s frame
#[inline(never)]
fn print_reading(index: usize) {
    let mut line = [0u8; 256];
    let len = with(0, |s| s.table.get(index).map_or(0, |sensor| reading_line(sensor, &mut line)));
    c::console_line(line.get(..len).unwrap_or(&[]));
}

/// Write the `[READ]` line of `sensor`; its length (at most 255)
fn reading_line(sensor: &Sensor, line: &mut [u8; 256]) -> usize {
    let mut w = Snprintf::new(line);
    w.bytes(b"[READ] id=");
    w.bytes(until_nul(&sensor.id));
    w.bytes(b" name=\"");
    w.bytes(until_nul(&sensor.name));
    w.bytes(b"\" value=");
    if sensor.value_type == ValueType::Numeric as u32 {
        w.fixed3(sensor.last_value);
    } else {
        w.bytes(until_nul(&sensor.last_text));
    }
    w.bytes(b" units=");
    w.bytes(until_nul(&sensor.units));
    w.finish().min(255)
}

/// Initialize: no sensors, sunlit, SOC 0.75
#[unsafe(no_mangle)]
pub extern "C" fn sensors_init() -> i32 {
    with(rc::BUSY, |s| {
        s.table = [Sensor::ZERO; MAX_SENSORS];
        s.kinds = [None; MAX_SENSORS];
        s.count = 0;
        s.sunlit = true;
        s.soc = 0.75;
        rc::OK
    })
}

/// Add the sensors of a YAML configuration file
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sensors_load_yaml(path: *const c_char) -> i32 {
    if path.is_null() {
        c::assert_failed(c"Null pointer: path", line!());
        return rc::NULL_PTR;
    }
    // SAFETY: non-NULL; the C contract: a C string
    let path = unsafe { core::ffi::CStr::from_ptr(path) };
    if let Err(e) = load_file(path) {
        return e;
    }
    if with(0, |s| s.count) > 0 { rc::OK } else { rc::ERROR }
}

/// No file system on the target: the built-in table is used
#[cfg(target_os = "none")]
fn load_file(_path: &core::ffi::CStr) -> Result<(), i32> {
    Err(rc::IO)
}

#[cfg(not(target_os = "none"))]
use host::load_file;

#[cfg(not(target_os = "none"))]
mod host {
    use core::ffi::CStr;

    use sqso_fsw::sensors::{SensorDef, YamlParser};

    use crate::c::rc;

    /// Parse the file, adding its sensors
    pub(super) fn load_file(path: &CStr) -> Result<(), i32> {
        // SAFETY: both are NUL-terminated
        let f = unsafe { libc::fopen(path.as_ptr(), c"r".as_ptr()) };
        if f.is_null() {
            return Err(rc::IO);
        }
        let mut parser = YamlParser::new();
        let mut sink = |def: &SensorDef| super::add(def);
        let mut buf = [0u8; 512];
        loop {
            // SAFETY: buf is a live, writable buffer; f is open
            let n = unsafe { libc::fread(buf.as_mut_ptr().cast(), 1, buf.len(), f) };
            parser.feed(buf.get(..n).unwrap_or(&[]), &mut sink);
            if n < buf.len() {
                break;
            }
        }
        parser.finish(&mut sink);
        // SAFETY: f is open, closed once
        if unsafe { libc::fclose(f) } != 0 {
            return Err(rc::IO);
        }
        Ok(())
    }
}

/// Add the built-in sensor table
#[unsafe(no_mangle)]
pub extern "C" fn sensors_load_defaults() -> i32 {
    for def in &sensors::defaults() {
        add(def);
    }
    rc::OK
}

/// Number of sensors
#[unsafe(no_mangle)]
pub extern "C" fn sensors_get_count() -> usize {
    with(0, |s| s.count)
}

/// Copy of sensor `index`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sensors_get(index: usize, sensor: *mut Sensor) -> i32 {
    // SAFETY: the C contract: sensor points to a Sensor_t (same layout)
    let Some(out) = (unsafe { ptr::out(sensor) }) else {
        c::assert_failed(c"Null pointer: sensor", line!());
        return rc::NULL_PTR;
    };
    match SENSORS.with(|s| s.table.get(..s.count).and_then(|t| t.get(index)).copied()) {
        Some(Some(found)) => {
            *out = found;
            rc::OK
        }
        Some(None) => {
            c::assert_failed(c"Index out of range", line!());
            rc::INVALID
        }
        None => rc::BUSY,
    }
}

/// Copy of the sensor with identifier `id`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sensors_get_by_id(id: *const c_char, sensor: *mut Sensor) -> i32 {
    // SAFETY: the C contract: a C string
    let Some(id) = (unsafe { ptr::cstr(id) }) else {
        c::assert_failed(c"Null pointer: id", line!());
        return rc::NULL_PTR;
    };
    // SAFETY: the C contract: sensor points to a Sensor_t
    let Some(out) = (unsafe { ptr::out(sensor) }) else {
        c::assert_failed(c"Null pointer: sensor", line!());
        return rc::NULL_PTR;
    };
    let found = with(None, |s| s.table.get(..s.count).and_then(|t| t.iter().find(|x| until_nul(&x.id) == id)).copied());
    match found {
        Some(f) => {
            *out = f;
            rc::OK
        }
        None => rc::ERROR,
    }
}

/// Read every sensor that is due at `current_ms`; the number read
#[unsafe(no_mangle)]
pub extern "C" fn sensors_poll(current_ms: u64) -> usize {
    let mut polled = 0;
    let mut index = 0;
    // Only what a read needs is copied out of the table, not the whole
    // Sensor_t (SRS-C003); None: past the end of the table, or busy
    while let Some(job) = with(None, |s| poll_job(s, index, current_ms)) {
        if let Some(Job { kind, channel, period_ms }) = job {
            let reading = kind.map(|k| read(k, &channel));
            let next =
                current_ms.wrapping_add(u64::from(if period_ms > 0 { period_ms } else { sensors::DEFAULT_PERIOD_MS }));
            let updated = with(false, |s| {
                if let Some(r) = reading {
                    store_reading(s, index, r, current_ms);
                }
                s.table.get_mut(index).map(|slot| slot.next_poll_ms = next).is_some()
            });
            if reading.is_some() && updated {
                print_reading(index);
                polled += 1;
            }
        }
        index += 1;
    }
    polled
}

/// What a poll needs of a due sensor
struct Job {
    kind: Option<Kind>,
    channel: [u8; CHANNEL_LEN],
    period_ms: u32,
}

/// Sensor `index`: None past the end of the table; Some(None) if it is not
/// due at `current_ms`
#[allow(clippy::option_option)] // the two levels are "end of table" and "not due"
fn poll_job(s: &Sensors, index: usize, current_ms: u64) -> Option<Option<Job>> {
    let sensor = s.table.get(..s.count)?.get(index)?;
    let kind = s.kinds.get(index).copied().flatten();
    Some((current_ms >= sensor.next_poll_ms).then_some(Job {
        kind,
        channel: sensor.channel,
        period_ms: sensor.period_ms,
    }))
}

/// Read sensor `index` now
#[unsafe(no_mangle)]
pub extern "C" fn sensors_poll_one(index: usize) -> i32 {
    let Some(found) = SENSORS
        .with(|s| if index < s.count { s.table.get(index).copied().zip(s.kinds.get(index).copied()) } else { None })
    else {
        return rc::BUSY;
    };
    let Some((sensor, kind)) = found else {
        c::assert_failed(c"Index out of range", line!());
        return rc::INVALID;
    };
    let Some(kind) = kind else { return rc::ERROR };
    let reading = read(kind, &sensor.channel);
    let now = c::now_ms();
    with(rc::BUSY, |s| {
        store_reading(s, index, reading, now);
        rc::OK
    })
}

/// Set the simulated environment
#[unsafe(no_mangle)]
pub extern "C" fn sensors_set_environment(sunlit: bool, soc: f64) {
    with((), |s| {
        s.sunlit = sunlit;
        s.soc = soc;
    });
}

/// `id=value units,` for every sensor, as many as fit with a NUL; the
/// length written
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sensors_format_telemetry(buffer: *mut c_char, buffer_len: usize) -> usize {
    if buffer.is_null() {
        c::assert_failed(c"Null pointer: buffer", line!());
        return 0;
    }
    if buffer_len == 0 {
        c::assert_failed(c"Invalid buffer length", line!());
        return 0;
    }
    // SAFETY: non-NULL; the C contract: buffer_len writable bytes
    let Some(buf) = (unsafe { ptr::bytes_mut(buffer.cast(), buffer_len) }) else { return 0 };
    with(0, |s| {
        let entries = s.table.get(..s.count).unwrap_or(&[]).iter().map(|x| {
            let value = if x.value_type == ValueType::Numeric as u32 {
                Value::Numeric(x.last_value)
            } else {
                Value::Text(&x.last_text)
            };
            (&x.id[..], value, &x.units[..])
        });
        sensors::format_telemetry(entries, buf)
    })
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn sensor(value_type: ValueType) -> Sensor {
        let mut s = Sensor::ZERO;
        sensors::copy_cstr(&mut s.id, b"BV");
        sensors::copy_cstr(&mut s.name, b"Battery Voltage");
        sensors::copy_cstr(&mut s.units, b"V");
        sensors::copy_cstr(&mut s.last_text, b"0F");
        s.value_type = value_type as u32;
        s.last_value = 8.1375;
        s
    }

    /// RID FSW-14: a condition that lasts (here a low battery for 200
    /// polls) is logged at its onset and at its clearance, with the count;
    /// the older entries of the fault log stay
    #[test]
    fn a_long_episode_leaves_older_entries_intact() {
        use crate::c::fault_type;
        use crate::fault_mgmt::{self, fault_log_get_count, fault_log_get_entry};
        use sqso_fsw::fault_log::Entry;
        use sqso_fsw::eps_telemetry::{self, Telemetry};
        let _serial = crate::fault_mgmt::tests::serial();
        let block = |vbatt_v: f64| {
            let mut b = [0u8; eps_telemetry::BLOCK_LEN];
            eps_telemetry::encode(&Telemetry { soc: 0.5, vbatt_v, ..Telemetry::default() }, &mut b);
            let mut regs = [0u8; 1 + eps_telemetry::BLOCK_LEN];
            regs[1..].copy_from_slice(&b);
            // SAFETY: regs is a live buffer of its length
            let w = unsafe { crate::hal_sim::hal_i2c_write(0, eps_telemetry::ADDRESS, regs.as_ptr(), regs.len()) };
            assert_eq!(w, rc::OK);
        };
        let description = |i: usize| {
            let mut e = Entry::new(0, 0, 0, b"", 0.0);
            // SAFETY: a live entry
            assert_eq!(unsafe { fault_log_get_entry(i, &mut e) }, rc::OK);
            e.description().to_vec()
        };
        assert_eq!(crate::hal_sim::hal_flash_init(), rc::OK);
        assert_eq!(crate::hal_sim::hal_i2c_init(0, 100_000), rc::OK);
        assert_eq!(fault_mgmt::fault_mgmt_init(), rc::OK);
        assert_eq!(fault_mgmt::fault_log_clear(), rc::OK);
        for text in [&b"older entry 1"[..], b"older entry 2", b"older entry 3"] {
            assert_eq!(fault_mgmt::add(fault_type::POWER, severity::INFO, text, 0.5), rc::OK);
        }
        let _ = ALARMS.with(|a| *a = [Condition::new(); 2]);
        assert_eq!(sensors_init(), rc::OK);
        assert_eq!(sensors_load_defaults(), rc::OK);
        block(11.5);
        let mut now = 1_000_000;
        for _ in 0..200 {
            crate::time::smart_qso_sim_advance_ms(1000);
            now += 1000;
            let _ = sensors_poll(now);
        }
        assert_eq!(fault_log_get_count(), 4, "the onset only");
        assert_eq!(description(0), b"older entry 1");
        assert_eq!(description(2), b"older entry 3");
        assert_eq!(description(3), b"Low battery voltage detected");
        block(15.0);
        crate::time::smart_qso_sim_advance_ms(1000);
        let _ = sensors_poll(now + 1000);
        assert_eq!(fault_log_get_count(), 5);
        assert_eq!(description(4), b"Low battery voltage detected cleared after 200 checks", "the count");
        assert_eq!(fault_mgmt::fault_log_clear(), rc::OK);
    }

    #[test]
    fn reading_lines() {
        let mut line = [0u8; 256];
        let n = reading_line(&sensor(ValueType::Numeric), &mut line);
        assert_eq!(&line[..n], b"[READ] id=BV name=\"Battery Voltage\" value=8.137 units=V");
        let n = reading_line(&sensor(ValueType::Hex2), &mut line);
        assert_eq!(&line[..n], b"[READ] id=BV name=\"Battery Voltage\" value=0F units=V");
    }

    #[test]
    fn reading_line_is_cut_at_255_bytes() {
        let mut s = sensor(ValueType::Numeric);
        s.name = [b'n'; NAME_LEN];
        s.units = [b'u'; UNITS_LEN];
        let mut line = [0u8; 256];
        assert!(reading_line(&s, &mut line) <= 255);
    }
}
