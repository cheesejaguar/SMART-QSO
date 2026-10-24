//! Attitude control (`adcs_control.h`, docs/hardware/ADCS_DESIGN.md):
//! B-dot detumble, sun pointing with magnetorquers, the body-rate estimate
//! from the change of the magnetic field, and the mode logic.
//!
//! The arithmetic is the C's, in double precision and in the same order,
//! so the results are the same to the bit. The record types have the C
//! layouts (`#[repr(C)]`).

use crate::fmath::sqrt;

/// Largest dipole per axis, A m^2
pub const MAX_DIPOLE: f64 = 0.2;
/// B-dot gain
pub const BDOT_GAIN: f64 = 5.0e6;
/// Sun-pointing proportional gain
pub const SUNPOINT_KP: f64 = 0.001;
/// Rate below which the detumble settles, rad/s (~0.5 deg/s)
pub const DETUMBLE_RATE_THRESHOLD: f64 = 0.0087;
/// Time below the threshold that completes the detumble, ms (5 min)
pub const DETUMBLE_SETTLING_MS: u64 = 300_000;
/// Longest detumble, ms (90 min, one orbit)
pub const DETUMBLE_TIMEOUT_MS: u64 = 5_400_000;
/// Sun sensors
pub const SUN_SENSOR_COUNT: usize = 6;
/// Sun intensity below which it is eclipse (5 %)
pub const ECLIPSE_INTENSITY: f64 = 0.05;

/// `AdcsMode_t`
pub mod mode {
    /// Sensors on, actuators off
    pub const IDLE: u32 = 0;
    /// B-dot detumble
    pub const DETUMBLE: u32 = 1;
    /// Sun pointing
    pub const SUNPOINT: u32 = 2;
    /// Eclipse (drift)
    pub const ECLIPSE: u32 = 3;
    /// Safe (minimal operation)
    pub const SAFE: u32 = 4;
}

/// `Vec3_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    /// X
    pub x: f64,
    /// Y
    pub y: f64,
    /// Z
    pub z: f64,
}

impl Vec3 {
    /// The zero vector
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

    /// A vector
    pub const fn new(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3 { x, y, z }
    }

    /// Magnitude (`vec3_magnitude`)
    pub fn magnitude(&self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y + self.z * self.z)
    }

    /// Normalize in place, unless (almost) zero (`vec3_normalize`)
    pub fn normalize(&mut self) {
        let mag = self.magnitude();
        if mag > 1e-10 {
            self.x /= mag;
            self.y /= mag;
            self.z /= mag;
        }
    }

    /// self x b (`vec3_cross`)
    pub fn cross(&self, b: &Vec3) -> Vec3 {
        Vec3 { x: self.y * b.z - self.z * b.y, y: self.z * b.x - self.x * b.z, z: self.x * b.y - self.y * b.x }
    }

    /// self . b (`vec3_dot`)
    pub fn dot(&self, b: &Vec3) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
}

/// `MagData_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MagData {
    /// Field, uT
    pub field: Vec3,
    /// Field rate of change, uT/s
    pub field_rate: Vec3,
    /// Time of the measurement, ms
    pub timestamp_ms: u64,
    /// Valid
    pub valid: bool,
}

/// `SunSensorData_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SunSensorData {
    /// Raw ADC values
    pub raw: [u16; SUN_SENSOR_COUNT],
    /// Sun unit vector
    pub sun_vector: Vec3,
    /// Total intensity
    pub intensity: f64,
    /// Sun visible
    pub sun_visible: bool,
    /// Time of the measurement, ms
    pub timestamp_ms: u64,
}

/// `MtqCommand_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MtqCommand {
    /// Commanded dipole, A m^2
    pub dipole: Vec3,
    /// Duty cycle per coil, -100 to +100
    pub pwm: [i8; 3],
    /// Coils driven
    pub enabled: bool,
}

/// `AdcsState_t` (adcs_control.h)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    /// `AdcsMode_t`
    pub mode: u32,
    /// Magnetometer
    pub mag: MagData,
    /// Sun sensors
    pub sun: SunSensorData,
    /// Magnetorquer command
    pub mtq: MtqCommand,
    /// Estimated body rate, rad/s
    pub angular_rate_est: Vec3,
    /// Its magnitude, rad/s
    pub rate_magnitude: f64,
    /// Detumble start, ms
    pub detumble_start_ms: u64,
    /// Start of the time below the rate threshold, ms (0: not settling)
    pub settling_start_ms: u64,
    /// Detumble completed
    pub detumble_complete: bool,
    /// Control cycles run
    pub control_cycles: u32,
    /// Faults
    pub fault_count: u32,
}

/// `AdcsTelemetry_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Telemetry {
    /// Mode
    pub mode: u8,
    /// Field X, 0.1 uT
    pub mag_x: i16,
    /// Field Y, 0.1 uT
    pub mag_y: i16,
    /// Field Z, 0.1 uT
    pub mag_z: i16,
    /// Sun X, 0.01
    pub sun_x: i16,
    /// Sun Y, 0.01
    pub sun_y: i16,
    /// Sun Z, 0.01
    pub sun_z: i16,
    /// Rate X, 0.01 deg/s
    pub rate_x: i16,
    /// Rate Y, 0.01 deg/s
    pub rate_y: i16,
    /// Rate Z, 0.01 deg/s
    pub rate_z: i16,
    /// Coil X duty cycle
    pub mtq_x: i8,
    /// Coil Y duty cycle
    pub mtq_y: i8,
    /// Coil Z duty cycle
    pub mtq_z: i8,
    /// Sun visible
    pub sun_visible: u8,
    /// Detumble complete
    pub detumble_done: u8,
}

/// Limit a dipole's magnitude to `max`, keeping its direction
/// (`adcs_saturate_dipole`)
pub fn saturate_dipole(dipole: &mut Vec3, max: f64) {
    let mag = dipole.magnitude();
    if mag > max && mag > 1e-10 {
        let scale = max / mag;
        dipole.x *= scale;
        dipole.y *= scale;
        dipole.z *= scale;
    }
}

/// B-dot: m = -k dB/dt (dB/dt in uT/s), saturated; zero for invalid data
/// (`adcs_compute_bdot`)
pub fn bdot(mag: &MagData) -> Vec3 {
    if !mag.valid {
        return Vec3::ZERO;
    }
    let k = BDOT_GAIN;
    let mut dipole = Vec3 {
        x: -k * mag.field_rate.x * 1e-6, // uT to T
        y: -k * mag.field_rate.y * 1e-6,
        z: -k * mag.field_rate.z * 1e-6,
    };
    saturate_dipole(&mut dipole, MAX_DIPOLE);
    dipole
}

/// Sun pointing: the torque that turns +X toward the sun, as a dipole
/// m = (tau x B) / |B|^2, saturated; zero without sun, valid field or a
/// field of at least 1 uT (`adcs_compute_sunpoint`)
pub fn sunpoint(sun: &SunSensorData, mag: &MagData) -> Vec3 {
    if !sun.sun_visible || !mag.valid {
        return Vec3::ZERO;
    }
    let sun_desired = Vec3::new(1.0, 0.0, 0.0);
    let error = sun.sun_vector.cross(&sun_desired);
    let torque = Vec3 { x: SUNPOINT_KP * error.x, y: SUNPOINT_KP * error.y, z: SUNPOINT_KP * error.z };
    let b_field = mag.field;
    let b_mag_sq = b_field.dot(&b_field);
    let mut dipole = if b_mag_sq > 1.0 {
        let t = torque.cross(&b_field);
        Vec3 { x: t.x / b_mag_sq * 1e6, y: t.y / b_mag_sq * 1e6, z: t.z / b_mag_sq * 1e6 }
    } else {
        Vec3::ZERO
    };
    saturate_dipole(&mut dipole, MAX_DIPOLE);
    dipole
}

/// Duty cycle (-100 to +100 %) for one coil, clamped before the conversion
pub fn dipole_to_pwm(dipole: f64) -> i8 {
    let duty = ((dipole / MAX_DIPOLE) * 100.0).clamp(-100.0, 100.0);
    #[allow(clippy::cast_possible_truncation)] // within +/-100 (NaN gives 0)
    let pwm = duty as i8;
    pwm
}

/// The simulated magnetometer reading `field` at `now_ms` (the rate field
/// is the estimator's and is left alone)
pub fn read_magnetometer(data: &mut MagData, field: Vec3, now_ms: u64) {
    data.field = field;
    data.timestamp_ms = now_ms;
    data.valid = true;
}

/// The simulated sun sensors seeing `sun` (a vector whose length is the
/// intensity, 1 for full sun) at `now_ms`
pub fn read_sun_sensors(data: &mut SunSensorData, sun: Vec3, now_ms: u64) {
    data.sun_vector = sun;
    data.sun_vector.normalize();
    data.intensity = sun.magnitude();
    data.sun_visible = data.intensity > ECLIPSE_INTENSITY; // ADCS_DESIGN 5.3
    data.timestamp_ms = now_ms;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // C (uint16_t) of a double
    let raw = (1000.0 * data.intensity) as u16;
    data.raw = [raw; SUN_SENSOR_COUNT];
}

/// What a control cycle did that the caller reports
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cycle {
    /// The detumble timed out (a fault: back to IDLE)
    pub detumble_timeout: bool,
    /// The detumble completed (now SUNPOINT)
    pub detumble_complete: bool,
}

/// A mode change
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModeChange {
    /// The mode before
    pub from: u32,
    /// The mode now
    pub to: u32,
    /// A new detumble started (its clock reset)
    pub detumble_started: bool,
}

/// Why a request was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdcsError {
    /// `adcs_init()` has not run
    NotInitialized,
    /// Not one of the five modes
    InvalidMode,
}

/// The ADCS module
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Adcs {
    /// What `adcs_get_state()` reports
    pub state: State,
    /// The previous magnetometer reading, for the rate estimate
    pub mag_prev: MagData,
    /// Initialized
    pub initialized: bool,
}

impl Adcs {
    /// Before `adcs_init()`
    pub const fn new() -> Adcs {
        const MAG: MagData = MagData { field: Vec3::ZERO, field_rate: Vec3::ZERO, timestamp_ms: 0, valid: false };
        Adcs {
            state: State {
                mode: mode::IDLE,
                mag: MAG,
                sun: SunSensorData {
                    raw: [0; SUN_SENSOR_COUNT],
                    sun_vector: Vec3::ZERO,
                    intensity: 0.0,
                    sun_visible: false,
                    timestamp_ms: 0,
                },
                mtq: MtqCommand { dipole: Vec3::ZERO, pwm: [0; 3], enabled: false },
                angular_rate_est: Vec3::ZERO,
                rate_magnitude: 0.0,
                detumble_start_ms: 0,
                settling_start_ms: 0,
                detumble_complete: false,
                control_cycles: 0,
                fault_count: 0,
            },
            mag_prev: MAG,
            initialized: false,
        }
    }

    /// `adcs_init()`: IDLE, coils off, everything cleared
    pub fn init(&mut self) {
        *self = Adcs { initialized: true, ..Adcs::new() };
    }

    /// The mode (SAFE before init)
    pub fn mode(&self) -> u32 {
        if self.initialized { self.state.mode } else { mode::SAFE }
    }

    /// Body rate from the change in the field between two valid readings
    /// at least 10 ms apart: an inertially fixed field gives
    /// dB/dt = -w x B, so w = (dB/dt x B) / |B|^2 (the component along B is
    /// not observable)
    fn estimate_rate(&mut self, now_ms: u64) {
        let s = &mut self.state;
        if self.mag_prev.valid && s.mag.valid {
            #[allow(clippy::cast_precision_loss)] // as the C (double) conversion
            let dt = now_ms.wrapping_sub(self.mag_prev.timestamp_ms) as f64 / 1000.0;
            if dt > 0.01 {
                let b_mag = s.mag.field.magnitude();
                if b_mag > 1.0 {
                    let b_dot = Vec3 {
                        x: (s.mag.field.x - self.mag_prev.field.x) / dt,
                        y: (s.mag.field.y - self.mag_prev.field.y) / dt,
                        z: (s.mag.field.z - self.mag_prev.field.z) / dt,
                    };
                    s.mag.field_rate = b_dot;
                    let field_sq = b_mag * b_mag;
                    let omega = b_dot.cross(&s.mag.field);
                    s.angular_rate_est = Vec3 { x: omega.x / field_sq, y: omega.y / field_sq, z: omega.z / field_sq };
                }
            }
        }
        self.mag_prev = s.mag;
        s.rate_magnitude = s.angular_rate_est.magnitude();
    }

    /// One DETUMBLE cycle: timeout check, B-dot, rate convergence
    fn detumble_step(&mut self, now_ms: u64, dipole: &mut Vec3, cycle: &mut Cycle) {
        let s = &mut self.state;
        if s.detumble_start_ms > 0 && now_ms.wrapping_sub(s.detumble_start_ms) > DETUMBLE_TIMEOUT_MS {
            // A fault, not a completed detumble (ADCS_DESIGN 5.2, 5.3)
            cycle.detumble_timeout = true;
            s.fault_count = s.fault_count.wrapping_add(1);
            s.mode = mode::IDLE;
            return;
        }
        *dipole = bdot(&s.mag);
        if s.rate_magnitude < DETUMBLE_RATE_THRESHOLD {
            if s.settling_start_ms == 0 {
                s.settling_start_ms = now_ms;
            } else if now_ms.wrapping_sub(s.settling_start_ms) > DETUMBLE_SETTLING_MS {
                // ADCS_DESIGN 5.1: detumble complete enters SUNPOINT
                s.detumble_complete = true;
                s.mode = mode::SUNPOINT;
                cycle.detumble_complete = true;
            }
        } else {
            s.settling_start_ms = 0;
        }
    }

    /// One control cycle (`adcs_update()`): read the sensors (the simulated
    /// field and sun, at `read_ms`), estimate the rate, run the mode, drive
    /// or release the coils. `None` before init
    pub fn update(&mut self, now_ms: u64, read_ms: u64, field: Vec3, sun: Vec3) -> Option<Cycle> {
        if !self.initialized {
            return None;
        }
        read_magnetometer(&mut self.state.mag, field, read_ms);
        read_sun_sensors(&mut self.state.sun, sun, read_ms);
        self.estimate_rate(now_ms);

        let mut dipole = Vec3::ZERO;
        let mut cycle = Cycle::default();
        match self.state.mode {
            mode::DETUMBLE => self.detumble_step(now_ms, &mut dipole, &mut cycle),
            mode::SUNPOINT => {
                if self.state.sun.sun_visible {
                    dipole = sunpoint(&self.state.sun, &self.state.mag);
                } else {
                    self.state.mode = mode::ECLIPSE; // no sun
                }
            }
            // Eclipse: drift, no control, until the sun is back
            mode::ECLIPSE if self.state.sun.sun_visible => self.state.mode = mode::SUNPOINT,
            _ => {} // ECLIPSE without sun, SAFE, IDLE: no active control
        }

        if self.state.mode == mode::DETUMBLE || self.state.mode == mode::SUNPOINT {
            self.apply_dipole(&dipole);
        } else {
            self.disable_mtq();
        }
        self.state.control_cycles = self.state.control_cycles.wrapping_add(1);
        Some(cycle)
    }

    /// Set the mode; entering DETUMBLE from another mode starts its clock
    pub fn set_mode(&mut self, new_mode: u32, now_ms: u64) -> Result<ModeChange, AdcsError> {
        if !self.initialized {
            return Err(AdcsError::NotInitialized);
        }
        if new_mode > mode::SAFE {
            return Err(AdcsError::InvalidMode); // only the five modes of ADCS_DESIGN 5.1
        }
        let from = self.state.mode;
        self.state.mode = new_mode;
        let detumble_started = new_mode == mode::DETUMBLE && from != mode::DETUMBLE;
        if detumble_started {
            self.state.detumble_start_ms = now_ms;
            self.state.settling_start_ms = 0;
            self.state.detumble_complete = false;
        }
        Ok(ModeChange { from, to: new_mode, detumble_started })
    }

    /// Command the coils
    pub fn apply_dipole(&mut self, dipole: &Vec3) {
        self.state.mtq = MtqCommand {
            dipole: *dipole,
            pwm: [dipole_to_pwm(dipole.x), dipole_to_pwm(dipole.y), dipole_to_pwm(dipole.z)],
            enabled: true,
        };
    }

    /// Coils off
    pub fn disable_mtq(&mut self) {
        self.state.mtq = MtqCommand { dipole: Vec3::ZERO, pwm: [0; 3], enabled: false };
    }

    /// Telemetry (`adcs_get_telemetry()`)
    pub fn telemetry(&self) -> Telemetry {
        #[allow(clippy::cast_possible_truncation)] // C (int16_t) of a double
        fn i16_of(v: f64) -> i16 {
            v as i16
        }
        let s = &self.state;
        #[allow(clippy::approx_constant)] // the C's value, for the same result
        let rad_to_deg = 180.0 / 3.141_592_653_59;
        let rate = |w: f64| i16_of(w * rad_to_deg * 100.0);
        #[allow(clippy::cast_possible_truncation)] // the five modes fit a byte
        let mode = s.mode as u8;
        Telemetry {
            mode,
            mag_x: i16_of(s.mag.field.x * 10.0),
            mag_y: i16_of(s.mag.field.y * 10.0),
            mag_z: i16_of(s.mag.field.z * 10.0),
            sun_x: i16_of(s.sun.sun_vector.x * 100.0),
            sun_y: i16_of(s.sun.sun_vector.y * 100.0),
            sun_z: i16_of(s.sun.sun_vector.z * 100.0),
            rate_x: rate(s.angular_rate_est.x),
            rate_y: rate(s.angular_rate_est.y),
            rate_z: rate(s.angular_rate_est.z),
            mtq_x: s.mtq.pwm[0],
            mtq_y: s.mtq.pwm[1],
            mtq_z: s.mtq.pwm[2],
            sun_visible: u8::from(s.sun.sun_visible),
            detumble_done: u8::from(s.detumble_complete),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn vector_utilities() {
        let mut a = Vec3::new(3.0, 4.0, 12.0);
        assert_eq!(a.magnitude(), 13.0);
        assert_eq!(a.dot(&Vec3::new(0.0, 1.0, 0.0)), 4.0);
        assert_eq!(Vec3::new(1.0, 0.0, 0.0).cross(&Vec3::new(0.0, 1.0, 0.0)), Vec3::new(0.0, 0.0, 1.0));
        a.normalize();
        assert!(close(a.magnitude(), 1.0, 1e-12));
        let mut z = Vec3::ZERO;
        z.normalize();
        assert_eq!(z, Vec3::ZERO);
    }

    #[test]
    fn bdot_law_and_saturation() {
        let mut mag = MagData { valid: true, field: Vec3::new(30.0, 0.0, 40.0), ..MagData::default() };
        mag.field_rate = Vec3::new(0.01, -0.02, 0.004);
        let m = bdot(&mag);
        assert!(close(m.x, -0.05, 1e-12) && close(m.y, 0.10, 1e-12) && close(m.z, -0.02, 1e-12));
        mag.field_rate = Vec3::new(3.0, -4.0, 0.0);
        let m = bdot(&mag);
        assert!(close(m.magnitude(), MAX_DIPOLE, 1e-12));
        assert!(close(m.x, -0.6 * MAX_DIPOLE, 1e-12) && close(m.y, 0.8 * MAX_DIPOLE, 1e-12));
        mag.valid = false;
        assert_eq!(bdot(&mag), Vec3::ZERO);
    }

    #[test]
    fn sunpoint_turns_plus_x_to_the_sun() {
        let sun = SunSensorData { sun_vector: Vec3::new(0.0, 1.0, 0.0), sun_visible: true, ..SunSensorData::default() };
        let mag = MagData { field: Vec3::new(50.0, 0.0, 0.0), valid: true, ..MagData::default() };
        let m = sunpoint(&sun, &mag);
        let torque = m.cross(&Vec3::new(50e-6, 0.0, 0.0));
        let wanted = Vec3::new(1.0, 0.0, 0.0).cross(&sun.sun_vector);
        assert!(torque.dot(&wanted) > 0.0);
        let weak = MagData { field: Vec3::new(0.5, 0.0, 0.0), ..mag };
        assert_eq!(sunpoint(&sun, &weak), Vec3::ZERO);
        let aligned = SunSensorData { sun_vector: Vec3::new(1.0, 0.0, 0.0), ..sun };
        assert_eq!(sunpoint(&aligned, &mag).magnitude(), 0.0);
    }

    #[test]
    fn pwm_scaling_and_clamp() {
        assert_eq!([dipole_to_pwm(0.1), dipole_to_pwm(-0.05), dipole_to_pwm(0.2)], [50, -25, 100]);
        assert_eq!([dipole_to_pwm(0.5), dipole_to_pwm(-1.0e6), dipole_to_pwm(f64::NAN)], [100, -100, 0]);
    }

    #[test]
    fn sensors() {
        let mut sun = SunSensorData::default();
        read_sun_sensors(&mut sun, Vec3::new(0.0, 0.6, 0.8), 5);
        assert!(sun.sun_visible);
        assert!(close(sun.sun_vector.z, 0.8, 1e-12));
        assert_eq!(sun.raw, [1000; SUN_SENSOR_COUNT]);
        read_sun_sensors(&mut sun, Vec3::new(0.0, 0.03, 0.0), 6);
        assert!(!sun.sun_visible);
        let mut mag = MagData { field_rate: Vec3::new(1.0, 2.0, 3.0), ..MagData::default() };
        read_magnetometer(&mut mag, Vec3::new(30.0, -12.5, 40.0), 7);
        assert!(mag.valid);
        assert_eq!(mag.field_rate, Vec3::new(1.0, 2.0, 3.0)); // left to the estimator
    }

    /// Body-frame field turned by -w dt about +Z (an inertially fixed field
    /// seen from a body turning at w)
    fn turned(f: Vec3, w: f64, dt: f64) -> Vec3 {
        let (s, c) = (-w * dt).sin_cos();
        Vec3::new(f.x * c - f.y * s, f.x * s + f.y * c, f.z)
    }

    extern crate std;

    #[test]
    fn detumble_settles_into_sunpoint_or_times_out() {
        let mut a = Adcs::new();
        assert_eq!(a.mode(), mode::SAFE);
        assert_eq!(a.update(0, 0, Vec3::ZERO, Vec3::ZERO), None);
        assert_eq!(a.set_mode(mode::DETUMBLE, 0), Err(AdcsError::NotInitialized));
        a.init();
        let t0 = 1_000_000u64;
        assert_eq!(a.set_mode(mode::DETUMBLE, t0).unwrap(), ModeChange { from: 0, to: 1, detumble_started: true });
        assert_eq!(a.set_mode(9, t0), Err(AdcsError::InvalidMode));
        let mut field = Vec3::new(45.0, 0.0, 0.0);
        let slow = 0.45f64.to_radians();
        let mut t = t0;
        let mut done = None;
        for i in 0..400u64 {
            t += 1000;
            field = turned(field, slow, 1.0);
            let c = a.update(t, t, field, Vec3::new(1.0, 0.0, 0.0)).unwrap();
            assert!(!c.detumble_timeout);
            if c.detumble_complete {
                done = Some(i);
                break;
            }
        }
        // Settling starts on the first cycle (no estimate yet: rate 0) and
        // completes once more than 5 min have passed
        assert_eq!(done, Some(301));
        assert_eq!(a.mode(), mode::SUNPOINT);
        assert!(a.state.mtq.enabled);
        assert_eq!(a.telemetry().detumble_done, 1);

        let mut b = Adcs::new();
        b.init();
        assert!(b.set_mode(mode::DETUMBLE, t0).is_ok());
        let fast = 2.0f64.to_radians();
        let mut field = Vec3::new(45.0, 0.0, 0.0);
        let mut t = t0;
        let mut timeout = false;
        for _ in 0..(91 * 6) {
            t += 10_000;
            field = turned(field, fast, 10.0);
            let c = b.update(t, t, field, Vec3::new(1.0, 0.0, 0.0)).unwrap();
            timeout |= c.detumble_timeout;
        }
        assert!(timeout);
        assert_eq!((b.mode(), b.state.fault_count, b.state.detumble_complete), (mode::IDLE, 1, false));
        assert!(!b.state.mtq.enabled);
    }

    #[test]
    fn rate_estimate_and_eclipse() {
        let mut a = Adcs::new();
        a.init();
        let w = 3.0f64.to_radians();
        let mut field = Vec3::new(45.0, 0.0, 0.0);
        let mut t = 5000u64;
        for _ in 0..5 {
            t += 1000;
            field = turned(field, w, 1.0);
            let _ = a.update(t, t, field, Vec3::ZERO);
        }
        assert!(close(a.state.angular_rate_est.z, w, 0.03 * w));
        assert!(close(a.state.rate_magnitude, w, 0.03 * w));
        assert!(close(f64::from(a.telemetry().rate_z), 300.0, 10.0));
        assert_eq!(a.state.control_cycles, 5);

        assert!(a.set_mode(mode::SUNPOINT, t).is_ok());
        let _ = a.update(t + 1000, t + 1000, field, Vec3::new(0.0, 1.0, 0.0));
        assert!(a.state.mtq.enabled);
        let _ = a.update(t + 2000, t + 2000, field, Vec3::ZERO);
        assert_eq!(a.mode(), mode::ECLIPSE);
        assert!(!a.state.mtq.enabled);
        let _ = a.update(t + 3000, t + 3000, field, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(a.mode(), mode::SUNPOINT);
    }

    /// Mutation check: the near-zero guards of normalize and saturate act
    /// strictly above 1e-10 (as the C), and the eclipse threshold is
    /// strictly above 5 %
    #[test]
    fn thresholds_are_strict() {
        let tiny = Vec3::new(1e-10, 0.0, 0.0);
        assert_eq!(tiny.magnitude(), 1e-10);
        let mut v = tiny;
        v.normalize();
        assert_eq!(v, tiny, "a magnitude of exactly 1e-10 is left alone");
        let mut v = tiny;
        saturate_dipole(&mut v, 0.0);
        assert_eq!(v, tiny, "nor is it scaled");
        let mut v = Vec3::new(2e-10, 0.0, 0.0);
        saturate_dipole(&mut v, 0.0);
        assert_eq!(v, Vec3::ZERO);

        let edge = Vec3::new(ECLIPSE_INTENSITY, 0.0, 0.0);
        assert_eq!(edge.magnitude(), ECLIPSE_INTENSITY);
        let mut sun = SunSensorData::default();
        read_sun_sensors(&mut sun, edge, 1);
        assert!(!sun.sun_visible, "exactly 5 % is eclipse");
        read_sun_sensors(&mut sun, Vec3::new(0.0, 0.3, 0.4), 2);
        assert_eq!((sun.intensity, sun.sun_visible), (0.5, true));
        assert_eq!(sun.raw, [500; SUN_SENSOR_COUNT], "raw is 1000 x intensity");
    }

    /// Mutation check: the sun-pointing dipole's value, unsaturated, and
    /// the |B|^2 > 1 uT^2 guard
    #[test]
    fn sunpoint_dipole_value() {
        let sun = SunSensorData { sun_vector: Vec3::new(0.0, 0.6, 0.8), sun_visible: true, ..SunSensorData::default() };
        let mag = MagData { field: Vec3::new(6000.0, 0.0, 8000.0), valid: true, ..MagData::default() };
        // error = s x X = (0, 0.8, -0.6); tau = Kp error; m = (tau x B) / |B|^2 * 1e6
        let m = sunpoint(&sun, &mag);
        assert!(close(m.x, 0.064, 1e-12) && close(m.y, -0.036, 1e-12) && close(m.z, -0.048, 1e-12), "{m:?}");
        let unit = MagData { field: Vec3::new(1.0, 0.0, 0.0), ..mag };
        assert_eq!(sunpoint(&sun, &unit), Vec3::ZERO, "|B|^2 of exactly 1 is too weak");
        let invisible = SunSensorData { sun_visible: false, ..sun };
        assert_eq!(sunpoint(&invisible, &mag), Vec3::ZERO);
    }

    /// Mutation check: the rate estimate's value (dB/dt over a 2 s step,
    /// w = dB/dt x B / |B|^2) and its guards: two valid readings, more
    /// than 10 ms apart, |B| above 1 uT
    #[test]
    fn rate_estimate_value_and_guards() {
        let mut a = Adcs::new();
        a.init();
        let _ = a.update(1000, 1000, Vec3::new(30.0, 0.0, 40.0), Vec3::ZERO);
        assert_eq!((a.state.mag.field_rate, a.state.angular_rate_est), (Vec3::ZERO, Vec3::ZERO), "one reading");
        let _ = a.update(3000, 3000, Vec3::new(32.0, 4.0, 34.0), Vec3::ZERO);
        assert_eq!(a.state.mag.field_rate, Vec3::new(1.0, 2.0, -3.0));
        let w = a.state.angular_rate_est;
        // (1, 2, -3) x (32, 4, 34) = (80, -130, -60); |B|^2 = 2196
        assert!(close(w.x, 80.0 / 2196.0, 1e-12) && close(w.y, -130.0 / 2196.0, 1e-12));
        assert!(close(w.z, -60.0 / 2196.0, 1e-12), "{w:?}");

        // 10 ms apart is too close; 11 ms is not
        let mut a = Adcs::new();
        a.init();
        let _ = a.update(1000, 1000, Vec3::new(30.0, 0.0, 40.0), Vec3::ZERO);
        let _ = a.update(1010, 1010, Vec3::new(31.0, 0.0, 40.0), Vec3::ZERO);
        assert_eq!(a.state.mag.field_rate, Vec3::ZERO);
        let _ = a.update(1021, 1021, Vec3::new(31.0, 1.0, 40.0), Vec3::ZERO);
        assert_ne!(a.state.mag.field_rate, Vec3::ZERO);
        assert_ne!(a.state.angular_rate_est, Vec3::ZERO);

        // |B| of exactly 1 uT gives no estimate
        let mut a = Adcs::new();
        a.init();
        let _ = a.update(1000, 1000, Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO);
        let _ = a.update(2000, 2000, Vec3::new(0.0, 1.0, 0.0), Vec3::ZERO);
        assert_eq!((a.state.mag.field_rate, a.state.angular_rate_est), (Vec3::ZERO, Vec3::ZERO));
    }

    /// Mutation check: the detumble timeout is strictly after 90 min, is
    /// not armed for a detumble whose clock reads 0, and a rate of exactly
    /// the threshold does not settle
    #[test]
    fn detumble_timeout_and_threshold_edges() {
        let field = Vec3::new(45.0, 0.0, 0.0);
        let t0 = 1000u64;
        let mut a = Adcs::new();
        a.init();
        assert!(a.set_mode(mode::DETUMBLE, t0).is_ok());
        let c = a.update(t0 + DETUMBLE_TIMEOUT_MS, t0 + DETUMBLE_TIMEOUT_MS, field, Vec3::ZERO).unwrap();
        assert!(!c.detumble_timeout);
        assert_eq!(a.mode(), mode::DETUMBLE);
        let c = a.update(t0 + DETUMBLE_TIMEOUT_MS + 1, t0 + DETUMBLE_TIMEOUT_MS + 1, field, Vec3::ZERO).unwrap();
        assert!(c.detumble_timeout);
        assert_eq!(a.mode(), mode::IDLE);

        let mut a = Adcs::new();
        a.init();
        assert!(a.set_mode(mode::DETUMBLE, 0).is_ok());
        let late = 2 * DETUMBLE_TIMEOUT_MS;
        let c = a.update(late, late, field, Vec3::ZERO).unwrap();
        assert!(!c.detumble_timeout, "a start time of 0 arms no timeout (as the C)");
        assert_eq!(a.mode(), mode::DETUMBLE);

        let mut a = Adcs::new();
        a.init();
        assert!(a.set_mode(mode::DETUMBLE, t0).is_ok());
        a.state.angular_rate_est = Vec3::new(DETUMBLE_RATE_THRESHOLD, 0.0, 0.0);
        let _ = a.update(t0 + 1000, t0 + 1000, field, Vec3::ZERO);
        assert_eq!(a.state.rate_magnitude, DETUMBLE_RATE_THRESHOLD);
        assert_eq!(a.state.settling_start_ms, 0, "exactly the threshold is not below it");
    }

    #[test]
    fn modes_and_telemetry_scaling() {
        let mut a = Adcs::new();
        a.init();
        assert_eq!(
            a.set_mode(mode::SAFE, 5),
            Ok(ModeChange { from: mode::IDLE, to: mode::SAFE, detumble_started: false })
        );
        assert_eq!(a.set_mode(mode::SAFE + 1, 5), Err(AdcsError::InvalidMode));
        assert!(a.set_mode(mode::DETUMBLE, 6).unwrap().detumble_started);
        let again = a.set_mode(mode::DETUMBLE, 7).unwrap();
        assert!(!again.detumble_started);
        assert_eq!(a.state.detumble_start_ms, 6);
        assert!(!a.set_mode(mode::IDLE, 8).unwrap().detumble_started);

        // ECLIPSE stays ECLIPSE while there is no sun
        assert!(a.set_mode(mode::ECLIPSE, 9).is_ok());
        let _ = a.update(10, 10, Vec3::new(30.0, 0.0, 40.0), Vec3::ZERO);
        assert_eq!(a.mode(), mode::ECLIPSE);
        assert!(!a.state.mtq.enabled);

        // Field in 0.1 uT, sun vector in 0.01 (truncated, as the C)
        let _ = a.update(20, 20, Vec3::new(12.3, -4.5, 6.7), Vec3::new(2.0, 3.0, 6.0));
        let t = a.telemetry();
        assert_eq!((t.mag_x, t.mag_y, t.mag_z), (123, -45, 67));
        assert_eq!((t.sun_x, t.sun_y, t.sun_z), (28, 42, 85));
        assert_eq!((t.mode, t.sun_visible), (u8::try_from(mode::SUNPOINT).unwrap(), 1));
    }
}
