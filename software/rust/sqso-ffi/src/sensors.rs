//! `sensors.h`: the sensor table, its configuration file (`SENSORS_YAML`
//! in the host simulation), polling and the telemetry text
//! (`sqso_fsw::sensors`).
//!
//! The readings are simulated from the environment the executive sets,
//! the EPS switches and C `rand()` noise, as before. The configuration file
//! is parsed by `sqso_fsw::sensors::YamlParser`, which is bounded and
//! fuzzed; there is no file system on the target, where loading it fails
//! with `SMART_QSO_ERROR_IO` (the target uses the built-in table).
//!
//! Each `Sensor_t.read` points to one of the `extern "C"` readers below, so
//! a C caller can still call it on a copy. Readers, the EPS and the fault
//! log are called with no sensor state borrowed.
//!
//! @requirement SRS-F070 Collect telemetry from all sensors
//! @requirement SRS-F071 Sample sensors at configurable rates
//! @requirement SRS-F072 Validate sensor readings against limits

use core::ffi::{c_char, c_int};

use sqso_fsw::sensors::{
    self, Alarm, CHANNEL_LEN, Environment, ID_LEN, Kind, MAX_SENSORS, NAME_LEN, SensorDef, Snprintf, StatusInputs,
    TEXT_LEN, TYPE_LEN, UNITS_LEN, Value, ValueType, until_nul,
};

use crate::c::{self, rc, severity};
use crate::cell::Global;
use crate::{eps_control, fault_mgmt, ptr};

unsafe extern "C" {
    // C library pseudo-random numbers (seeded by main.c for simulation noise)
    fn rand() -> c_int;
}

/// `RAND_MAX` of glibc and newlib
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
    type_: [u8; TYPE_LEN],
    channel: [u8; CHANNEL_LEN],
    period_ms: u32,
    next_poll_ms: u64,
    value_type: u32,
    read: Option<SensorReadFn>,
    last_value: f64,
    last_text: [u8; TEXT_LEN],
    last_read_ms: u64,
}

impl Sensor {
    const ZERO: Sensor = Sensor {
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
fn rnd_unit() -> f64 {
    // SAFETY: no arguments
    f64::from(unsafe { rand() }) / RAND_MAX
}

/// A reading
#[derive(Clone, Copy)]
enum Reading {
    Numeric(f64),
    Text([u8; TEXT_LEN]),
}

/// Take a reading of `kind` on `channel` (no sensor state borrowed)
fn read(kind: Kind, channel: &[u8]) -> Reading {
    let (sunlit, soc) = with((true, 0.75), |s| (s.sunlit, s.soc));
    let payload_enabled = eps_control::eps_is_payload_enabled();
    let env = Environment { sunlit, soc, payload_enabled };
    let mut noise = rnd_unit;
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
        Kind::EpsVoltage => sensors::eps_voltage(channel, &env, &mut noise),
        Kind::EpsCurrent => sensors::eps_current(channel, &env, &mut noise),
        Kind::EpsTemperature => sensors::Numeric { value: sensors::eps_temperature(&env, &mut noise), alarm: None },
        Kind::PayloadTemperature => {
            sensors::Numeric { value: sensors::payload_temperature(&env, &mut noise), alarm: None }
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
    match numeric.alarm {
        Some(Alarm::LowBatteryVoltage) => {
            c::log(FAULT_VOLTAGE_LOW, severity::ERROR, c"Low battery voltage detected", soc);
        }
        Some(Alarm::HighDischargeCurrent) => {
            c::log(FAULT_CURRENT_HIGH, severity::ERROR, c"Excessive battery discharge current", soc);
        }
        None => {}
    }
    Reading::Numeric(numeric.value)
}

/// `Sensor_t.read` for a kind: the sensor's channel (only the voltage and
/// current kinds look at it), the value or text out
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

/// The `[READ]` console line of a sensor
fn print_reading(sensor: &Sensor) {
    let mut line = [0u8; 256];
    let mut w = Snprintf::new(&mut line);
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
    let len = w.finish().min(line.len() - 1);
    c::console_line(line.get(..len).unwrap_or(&[]));
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
    while let Some((sensor, kind)) =
        with(
            None,
            |s| if index < s.count { s.table.get(index).copied().zip(s.kinds.get(index).copied()) } else { None },
        )
    {
        if current_ms >= sensor.next_poll_ms {
            let reading = kind.map(|k| read(k, &sensor.channel));
            let next = current_ms.wrapping_add(u64::from(if sensor.period_ms > 0 {
                sensor.period_ms
            } else {
                sensors::DEFAULT_PERIOD_MS
            }));
            let updated = with(None, |s| {
                if let Some(r) = reading {
                    store_reading(s, index, r, current_ms);
                }
                let slot = s.table.get_mut(index)?;
                slot.next_poll_ms = next;
                Some(*slot)
            });
            if let (Some(_), Some(after)) = (reading, updated) {
                print_reading(&after);
                polled += 1;
            }
        }
        index += 1;
    }
    polled
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
