//! Attitude control (`adcs_control.h`, docs/hardware/ADCS_DESIGN.md):
//! B-dot detumble, sun pointing with magnetorquers, the body-rate estimate
//! from the change of the magnetic field, and the mode logic.
//!
//! The arithmetic is the C's, in double precision and in the same order,
//! so the results are the same to the bit. The record types have the C
//! layouts (`#[repr(C)]`).
//!
//! SUNPOINT runs the spin-stabilized coarse sun pointing of the baseline
//! (BASELINE §8): [`spin_sunpoint`] spins the spacecraft about +Y, the
//! normal of the deployed wings and the major axis of inertia, and holds
//! the spin axis on the Sun, so the array faces the Sun and the spin keeps
//! the axis through eclipse with the coils off. Its rate input is the
//! gyro's ([`Adcs::update_with_gyro`]), or without one the magnetometer
//! estimate. [`sunpoint`] is the proportional law of ADCS_DESIGN §4.2
//! (`adcs_compute_sunpoint`), kept for its interface.
//!
//! The magnetometer is read with the coils off. The coils' own field at the
//! magnetometer is of the order of the Earth's (tens of µT at full dipole a
//! few centimetres away), so a sample taken while they are driven is
//! useless to B-dot and to the rate estimate. Each command is a burst of
//! [`CoilTiming::on_ms`] (900 ms of the 1 s cycle): the coil driver's timer
//! ends it by itself, so the coils are off for the last 100 ms of the cycle
//! even if the OBC stops. A sample is used only if the coils have been off
//! for [`CoilTiming::settle_ms`] (50 ms); otherwise it is discarded and
//! counted ([`Adcs::mag_rejected`]), so a missing window is visible in
//! telemetry and in the Monte Carlo. The burst's dipole is the commanded
//! one scaled by the period over the on-time (saturated), so the impulse
//! per cycle is the one the control law asked for.
//!
//! @requirement SRS-F090 The software shall use a magnetometer sample only when the magnetorquers have been off for at least 50 ms, discard and count any other sample
//! @requirement SRS-F091 The software shall command each magnetorquer burst for 900 ms of the 1 s control period, with the dipole scaled by the period over the on-time
//! @requirement SRS-F092 The software shall not complete a detumble on a magnetometer rate estimate that a discarded sample left stale
//! @requirement SRS-F093 The software shall run B-dot detumble and spin-stabilized sun pointing of +Y at 8 deg/s at 1 Hz, and treat a detumble lasting over 285 minutes as a fault
//! @requirement SRS-F094 The software shall let the ground select the spin axis and inertia tensor from the presets by telecommand, and refuse any other preset
//! @requirement SRS-F142 Gyro alignment correction: a persisted 3×3 matrix applied to the gyro rates, identity by default, set by telecommand
//! @requirement SRS-F143 No magnetometer sample used for actuation while the Jetson carrier (JET_PG) or the exciter (PTT_EXCITER) is on
//! @requirement SRS-F144 Magnetometer bias per polarity state of the three rods, zeros by default, set by telecommand
//!
//! **Calibration and gating (RID GNC-08).** The gyro rates pass through the
//! alignment matrix of VI-MCAD-05 ([`Calibration::gyro`], identity until
//! the ground sets it); the magnetometer reading has the bias of the rods'
//! polarity state subtracted ([`Calibration::mag_bias`], one vector for
//! each of the 2³ states of [`crate::mtq::polarity`], zeros until AV-V4
//! measures them). A sample taken while the Jetson carrier is powered
//! (`JET_PG`) or the exciter keyed (`PTT_EXCITER`) is counted
//! ([`Adcs::mag_gated`]) and not used: no rate estimate from it and the
//! coils off for that cycle; the spin holds the attitude meanwhile.

use crate::fmath::sqrt;

/// Largest dipole per axis, A m^2
pub const MAX_DIPOLE: f64 = 0.2;
/// B-dot gain, A m^2 s/T: about the optimum 2 n (1 + sin ξ) J_min / |B|^2 for
/// this inertia at 500 km (Avanzini and Giulietti, JGCD 35(4), 2012); the
/// mission simulation's Monte Carlo puts its fastest detumble at 1e4. A
/// gain far above it removes the rate across the field as fast as it
/// appears, so the momentum follows the turning field and its magnitude
/// decays only slowly (5e6 left 15 of 40 trials tumbling after 1.5 days)
pub const BDOT_GAIN: f64 = 1.0e4;
/// Sun-pointing proportional gain
pub const SUNPOINT_KP: f64 = 0.001;
/// Rate below which the detumble settles, rad/s (~0.5 deg/s)
pub const DETUMBLE_RATE_THRESHOLD: f64 = 0.0087;
/// Time below the threshold that completes the detumble, ms (5 min)
pub const DETUMBLE_SETTLING_MS: u64 = 300_000;
/// Longest detumble, ms (285 min, three orbits: BASELINE §8 "B-dot
/// detumble from 10°/s within 3 orbits"; the mission simulation's Monte
/// Carlo puts the slowest of its tip-offs, up to 30°/s, at 1.5 h)
pub const DETUMBLE_TIMEOUT_MS: u64 = 17_100_000;
/// Sun sensors
pub const SUN_SENSOR_COUNT: usize = 6;
/// Sun intensity below which it is eclipse (5 %)
pub const ECLIPSE_INTENSITY: f64 = 0.05;
/// The axis held on the Sun: +Y, the normal of the deployed wings
/// (hardware/eps/SOLAR_ARRAY_DESIGN.md §8.1). With the CAD tensor of
/// [`Inertia::DEPLOYED`] the major axis is 0.6° from +Y (it was about 14°
/// with the reference layout's products); the spin is held about +Y by the
/// nutation damping (Monte Carlo, ADCS_DESIGN §11.4; STR-01).
/// [`Geometry::PRESETS`] (ADCS_SET_GEOMETRY) selects other axes
pub const SUN_AXIS: Vec3 = Vec3::new(0.0, 1.0, 0.0);
/// Spin rate about [`SUN_AXIS`] in SUNPOINT, rad/s (8 deg/s). Chosen at CDR
/// (STR-01) with the full inertia tensor of layout B over Iyy/Izz 1.00 to
/// 1.05 (`docs/hardware/ADCS_DESIGN.md` §11.4,
/// `software/simulation/cdr/avionics/adcs_mc_cdr.sh`). Re-run with the final
/// CAD tensor of [`Inertia::DEPLOYED`] (2026-10-27, flight software round 3,
/// 40 trials of 2 days): at least 99.9 % of the sunlit time within 15° from
/// Iyy/Izz 1.005 up (mean Sun angle 1.8° to 2.0°, the coils 6 to 7 mW), and
/// the CBE and MEV plants 100 % (1.8°); but 76 % at 1.000, where the Y and Z
/// axes are degenerate; the CAD value is 1.051
pub const SPIN_RATE: f64 = 0.1396;
/// Moments of inertia about the body axes with the wings and the AntS
/// deployed, kg m^2: the final CAD of CDR round 3 (`docs/environment/
/// MASS_BUDGET.md` §6.1; MECHANICAL_DESIGN §5; layout B, CubeSpace rods, all
/// four AntS elements, the 2 × 10 g ballast of decision C-13 as re-placed,
/// the harness with its bundles under the EPS), the mean of its CBE (2.853,
/// 4.200, 3.978 × 10⁻³) and MEV (3.276, 4.758, 4.545 × 10⁻³) rows, as §6.5
/// directs: Iyy/Izz 1.051 (CBE 1.056, MEV 1.047; the major axis 0.6° from
/// +Y). The products are in [`Inertia::DEPLOYED`]. To be replaced by the
/// measured stowed inertia plus the weighed wings before flight
pub const INERTIA: Vec3 = Vec3::new(0.003_064_5, 0.004_479, 0.004_261_5);
/// Momentum gain of the spin law: torque per unit momentum error, 1/s
pub const MOMENTUM_GAIN: f64 = 0.005;
/// Nutation damping: torque per rad/s of rate across the spin axis, N m s
/// (with a momentum gain this high, much less damping is unstable; 3e-4
/// with [`SPIN_RATE`] is the robust pair of the CDR sweep, STR-01)
pub const NUTATION_GAIN: f64 = 3.0e-4;
/// Nutation damping without a gyro (the rate from the magnetometer, which
/// misses the rate about the field), N m s: 3e-4 there puts the mean Sun
/// angle at 55°, 1e-4 at 17° (CDR sweep, `docs/hardware/ADCS_DESIGN.md` §11.4)
pub const NUTATION_GAIN_NO_GYRO: f64 = 1.0e-4;
/// Spin hold: torque about +Y per unit spin-momentum error, 1/s. It holds
/// the spin positive about +Y, so that −Y on the Sun (with the spin
/// reversed), which the momentum law alone also accepts, is not a rest state
pub const SPIN_HOLD_GAIN: f64 = 0.005;

/// Control period, ms (1 Hz: ADCS_DESIGN §5.3)
pub const CONTROL_PERIOD_MS: u64 = 1000;
/// Coil burst after each command, ms: the coil driver's timer switches
/// the coils off at its end by itself (the measurement window is the rest
/// of the period)
pub const MTQ_ON_MS: u64 = 900;
/// Coils off for at least this long before a magnetometer sample is used,
/// ms: the drive current's decay (an air-core coil's L/R is well under
/// 1 ms; rods' eddy currents and the driver's flyback are slower) and the
/// magnetometer's conversion
pub const MAG_SETTLE_MS: u64 = 50;

/// When the coils are driven within a control period and when the
/// magnetometer may be read: the flight values ([`CoilTiming::FLIGHT`]), or
/// others for analysis (the Monte Carlo's missing-window case)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoilTiming {
    /// Coil burst after each command, ms ([`MTQ_ON_MS`])
    pub on_ms: u64,
    /// Coils off at least this long before a sample is used, ms ([`MAG_SETTLE_MS`])
    pub settle_ms: u64,
}

impl CoilTiming {
    /// The flight timing
    pub const FLIGHT: CoilTiming = CoilTiming { on_ms: MTQ_ON_MS, settle_ms: MAG_SETTLE_MS };

    /// The burst dipole that delivers the impulse of `dipole` held for a
    /// whole control period: scaled by the period over the on-time,
    /// saturated to [`MAX_DIPOLE`]
    pub fn burst(&self, dipole: &Vec3) -> Vec3 {
        #[allow(clippy::cast_precision_loss)] // milliseconds
        let k = if self.on_ms > 0 { CONTROL_PERIOD_MS as f64 / self.on_ms as f64 } else { 0.0 };
        let mut d = Vec3 { x: dipole.x * k, y: dipole.y * k, z: dipole.z * k };
        saturate_dipole(&mut d, MAX_DIPOLE);
        d
    }
}

impl Default for CoilTiming {
    fn default() -> CoilTiming {
        CoilTiming::FLIGHT
    }
}

// The window (period minus the burst) holds the settling time and the
// magnetometer's conversion
const _: () = assert!(MTQ_ON_MS + MAG_SETTLE_MS < CONTROL_PERIOD_MS);

/// A symmetric inertia tensor about the centre of mass, body axes, kg m^2
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inertia {
    /// Ixx
    pub xx: f64,
    /// Iyy
    pub yy: f64,
    /// Izz
    pub zz: f64,
    /// Ixy (the tensor's element, −Σ m x y)
    pub xy: f64,
    /// Ixz
    pub xz: f64,
    /// Iyz
    pub yz: f64,
}

impl Inertia {
    /// Both wings and the AntS deployed ([`INERTIA`] with its products, the
    /// tensor's off-diagonal elements −Σm·x·y and so on: the mean of the CAD
    /// CBE (Ixy +11.4, Ixz −11.0, Iyz −4.0) and MEV (+12.2, −19.6, +1.9) ×
    /// 10⁻⁶ kg m², MASS_BUDGET §6.1, final CAD of CDR round 3)
    pub const DEPLOYED: Inertia = Inertia {
        xx: 0.003_064_5,
        yy: 0.004_479,
        zz: 0.004_261_5,
        xy: 0.000_011_8,
        xz: -0.000_015_3,
        yz: -0.000_001_05,
    };
    /// Only wing W1 (+X hinge) deployed: the CAD's mean of CBE and MEV
    /// (STR-27; major axis 8.3° from +Y)
    pub const WING_1_ONLY: Inertia =
        Inertia { xx: 0.003_027, yy: 0.004_165, zz: 0.003_910, xy: -0.000_161_0, xz: -0.000_024_8, yz: 0.000_008_0 };
    /// Only wing W2 (−X hinge) deployed (STR-27; major axis 9.1° from +Y)
    pub const WING_2_ONLY: Inertia =
        Inertia { xx: 0.003_027, yy: 0.004_166, zz: 0.003_911, xy: 0.000_184_3, xz: -0.000_005_8, yz: 0.000_008_0 };
    /// Neither wing deployed, the AntS out (STR-27; major axis 3.3° from +Y)
    pub const NO_WINGS: Inertia =
        Inertia { xx: 0.002_985, yy: 0.003_856, zz: 0.003_560, xy: 0.000_011_5, xz: -0.000_015_3, yz: 0.000_017_1 };

    /// A diagonal tensor
    pub const fn diagonal(d: Vec3) -> Inertia {
        Inertia { xx: d.x, yy: d.y, zz: d.z, xy: 0.0, xz: 0.0, yz: 0.0 }
    }

    /// I w
    pub fn apply(&self, w: &Vec3) -> Vec3 {
        Vec3 {
            x: self.xx * w.x + self.xy * w.y + self.xz * w.z,
            y: self.xy * w.x + self.yy * w.y + self.yz * w.z,
            z: self.xz * w.x + self.yz * w.y + self.zz * w.z,
        }
    }

    /// The moment about the unit axis `a`: aᵀ I a
    pub fn about(&self, a: &Vec3) -> f64 {
        a.dot(&self.apply(a))
    }
}

/// What the spin law spins about and with what inertia: the axis held on
/// the Sun and the inertia tensor. Selectable by telecommand
/// (ADCS_SET_GEOMETRY, STR-27) when the wings did not both deploy
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Geometry {
    /// The body axis held on the Sun (unit)
    pub sun_axis: Vec3,
    /// The inertia tensor
    pub inertia: Inertia,
}

impl Geometry {
    /// Both wings deployed: +Y on the Sun
    pub const FLIGHT: Geometry = Geometry { sun_axis: SUN_AXIS, inertia: Inertia::DEPLOYED };

    /// The telecommand's presets: 0 both wings, 1 wing W1 only, 2 wing W2
    /// only, 3 no wing (the CAD's tensors, STR-27). With one wing the array
    /// normal is still +Y but the major axis tilts 8.3° (W1) or 9.1° (W2)
    /// toward the wing (`MASS_BUDGET.md` §6.1): the spin is held about the
    /// major axis, so the Sun sits that far off the remaining wing's normal
    /// (cos 9.1° = 0.99 of its power) and the spin needs no coil effort to
    /// stay about it
    pub const PRESETS: [Geometry; 4] = [
        Geometry::FLIGHT,
        Geometry { sun_axis: Vec3::new(-0.1381, 0.9896, 0.0410), inertia: Inertia::WING_1_ONLY },
        Geometry { sun_axis: Vec3::new(0.1557, 0.9875, 0.0247), inertia: Inertia::WING_2_ONLY },
        Geometry { sun_axis: Vec3::new(0.0121, 0.9983, 0.0568), inertia: Inertia::NO_WINGS },
    ];

    /// A preset by its telecommand number
    pub fn preset(n: u8) -> Option<Geometry> {
        Geometry::PRESETS.get(usize::from(n)).copied()
    }
}

impl Default for Geometry {
    fn default() -> Geometry {
        Geometry::FLIGHT
    }
}

/// The control gains the modes use: the flight values ([`Gains::FLIGHT`]),
/// or others for analysis (the mission simulation's Monte Carlo sweeps)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gains {
    /// B-dot gain ([`BDOT_GAIN`])
    pub bdot: f64,
    /// Spin rate about [`SUN_AXIS`] in SUNPOINT, rad/s ([`SPIN_RATE`])
    pub spin_rate: f64,
    /// Momentum gain, 1/s ([`MOMENTUM_GAIN`])
    pub momentum: f64,
    /// Nutation damping, N m s ([`NUTATION_GAIN`])
    pub nutation: f64,
    /// Nutation damping when there is no gyro, N m s ([`NUTATION_GAIN_NO_GYRO`])
    pub nutation_no_gyro: f64,
    /// Spin hold, 1/s ([`SPIN_HOLD_GAIN`])
    pub spin_hold: f64,
}

impl Gains {
    /// The flight gains
    pub const FLIGHT: Gains = Gains {
        bdot: BDOT_GAIN,
        spin_rate: SPIN_RATE,
        momentum: MOMENTUM_GAIN,
        nutation: NUTATION_GAIN,
        nutation_no_gyro: NUTATION_GAIN_NO_GYRO,
        spin_hold: SPIN_HOLD_GAIN,
    };
}

impl Default for Gains {
    fn default() -> Gains {
        Gains::FLIGHT
    }
}

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
    bdot_with(BDOT_GAIN, mag)
}

/// [`bdot`] with the gain `k`
pub fn bdot_with(k: f64, mag: &MagData) -> Vec3 {
    if !mag.valid {
        return Vec3::ZERO;
    }
    let mut dipole = Vec3 {
        x: -k * mag.field_rate.x * 1e-6, // uT to T
        y: -k * mag.field_rate.y * 1e-6,
        z: -k * mag.field_rate.z * 1e-6,
    };
    saturate_dipole(&mut dipole, MAX_DIPOLE);
    dipole
}

/// Sun pointing: the torque that turns +X toward the sun, as a dipole
/// m = (tau x B) / |B|^2, saturated; zero without sun, without a valid
/// field, or unless the field is stronger than 1 uT (|B|^2 > 1 uT^2)
/// (`adcs_compute_sunpoint`)
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

/// Spin-stabilized sun pointing: the momentum h = I w is driven to
/// `SPIN_RATE` I_yy along the sun direction, and the rate across +Y is
/// damped, so the only rest state is a spin about +Y with +Y on the Sun:
/// tau = -k_h (h - h_s s) - k_n (w - (w . y) y). In eclipse (no sun) only
/// the damping acts. As a dipole m = (B x tau) / |B|^2 (then m x B is the
/// part of tau across the field), saturated; zero without a valid field
/// stronger than 1 uT
pub fn spin_sunpoint(sun: &SunSensorData, mag: &MagData, rate: Vec3) -> Vec3 {
    spin_sunpoint_with(&Gains::FLIGHT, sun, mag, rate)
}

/// [`spin_sunpoint`] with the gains `g`
pub fn spin_sunpoint_with(g: &Gains, sun: &SunSensorData, mag: &MagData, rate: Vec3) -> Vec3 {
    spin_sunpoint_geo(g, &Geometry::FLIGHT, sun, mag, rate)
}

/// [`spin_sunpoint`] with the gains `g` about the axis and with the inertia
/// of `geo`: h = I w is driven to `spin_rate` (aᵀ I a) along the sun
/// direction, the rate across the axis a is damped, and the spin about a is
/// held positive
pub fn spin_sunpoint_geo(g: &Gains, geo: &Geometry, sun: &SunSensorData, mag: &MagData, rate: Vec3) -> Vec3 {
    let b_field = mag.field;
    let b_mag_sq = b_field.dot(&b_field);
    if !mag.valid || b_mag_sq <= 1.0 {
        return Vec3::ZERO;
    }
    let a = geo.sun_axis;
    let along = rate.dot(&a);
    let mut torque = Vec3 {
        x: -g.nutation * (rate.x - along * a.x),
        y: -g.nutation * (rate.y - along * a.y),
        z: -g.nutation * (rate.z - along * a.z),
    };
    if sun.sun_visible {
        let h_s = g.spin_rate * geo.inertia.about(&a);
        let h = geo.inertia.apply(&rate);
        let s = sun.sun_vector;
        torque.x -= g.momentum * (h.x - h_s * s.x);
        torque.y -= g.momentum * (h.y - h_s * s.y);
        torque.z -= g.momentum * (h.z - h_s * s.z);
        // Spin hold about the axis
        let spin_err = h.dot(&a) - h_s;
        torque.x -= g.spin_hold * spin_err * a.x;
        torque.y -= g.spin_hold * spin_err * a.y;
        torque.z -= g.spin_hold * spin_err * a.z;
    }
    let t = b_field.cross(&torque);
    let mut dipole = Vec3 { x: t.x / b_mag_sq * 1e6, y: t.y / b_mag_sq * 1e6, z: t.z / b_mag_sq * 1e6 };
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

/// Largest element of a gyro alignment matrix accepted (a misalignment
/// correction is close to the identity)
pub const GYRO_ALIGN_MAX: f64 = 1.1;
/// The determinant of an accepted alignment matrix is within this of 1
pub const GYRO_ALIGN_DET_TOL: f64 = 0.05;
/// Largest magnetometer bias component accepted, µT
pub const MAG_BIAS_MAX_UT: f64 = 50.0;
/// Polarity states of the three rods
pub const POLARITY_STATES: usize = 8;

/// The ADCS's ground-set calibration (persisted by `sqso-ffi`)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Calibration {
    /// Gyro alignment: body rate = `gyro` × sensor rate (rows)
    pub gyro: [[f64; 3]; 3],
    /// Magnetometer bias per polarity state, µT (subtracted)
    pub mag_bias: [Vec3; POLARITY_STATES],
}

impl Default for Calibration {
    fn default() -> Self {
        Calibration::IDENTITY
    }
}

impl Calibration {
    /// Identity alignment, zero biases
    pub const IDENTITY: Calibration = Calibration {
        gyro: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        mag_bias: [Vec3::ZERO; POLARITY_STATES],
    };

    /// The aligned rate
    pub fn gyro_apply(&self, w: Vec3) -> Vec3 {
        let [a, b, c] = self.gyro;
        let row = |r: [f64; 3]| r[0] * w.x + r[1] * w.y + r[2] * w.z;
        Vec3::new(row(a), row(b), row(c))
    }

    /// The field with the bias of polarity `state` removed
    pub fn mag_apply(&self, field: Vec3, state: u8) -> Vec3 {
        let b = self.mag_bias.get(usize::from(state & 0x07)).copied().unwrap_or(Vec3::ZERO);
        Vec3::new(field.x - b.x, field.y - b.y, field.z - b.z)
    }

    /// Whether a matrix is acceptable as a misalignment correction: every
    /// element finite and within ±[`GYRO_ALIGN_MAX`], determinant within
    /// [`GYRO_ALIGN_DET_TOL`] of 1
    pub fn gyro_valid(m: &[[f64; 3]; 3]) -> bool {
        let [[a, b, c], [d, e, f], [g, h, i]] = *m;
        let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
        m.iter().flatten().all(|v| v.is_finite() && v.abs() <= GYRO_ALIGN_MAX)
            && (det - 1.0).abs() <= GYRO_ALIGN_DET_TOL
    }

    /// Whether a bias is acceptable: finite, each component within
    /// ±[`MAG_BIAS_MAX_UT`]
    pub fn bias_valid(b: &Vec3) -> bool {
        [b.x, b.y, b.z].iter().all(|v| v.is_finite() && v.abs() <= MAG_BIAS_MAX_UT)
    }

    /// The gyro record: nine `f32`, big-endian (36 bytes)
    pub fn gyro_record(&self) -> [u8; 36] {
        let mut out = [0u8; 36];
        #[allow(clippy::cast_possible_truncation)] // stored as f32 (1e-7 relative)
        for (dst, v) in out.chunks_exact_mut(4).zip(self.gyro.iter().flatten()) {
            dst.copy_from_slice(&(*v as f32).to_be_bytes());
        }
        out
    }

    /// A gyro record's matrix, if valid
    pub fn gyro_from_record(rec: &[u8; 36]) -> Option<[[f64; 3]; 3]> {
        let mut v = [0.0f64; 9];
        for (dst, src) in v.iter_mut().zip(rec.chunks_exact(4)) {
            *dst = f64::from(f32::from_be_bytes([*src.first()?, *src.get(1)?, *src.get(2)?, *src.get(3)?]));
        }
        let [a, b, c, d, e, f, g, h, i] = v;
        let m = [[a, b, c], [d, e, f], [g, h, i]];
        Calibration::gyro_valid(&m).then_some(m)
    }

    /// The bias record: 8 × 3 `i16` in units of 10 nT, big-endian (48 bytes)
    pub fn bias_record(&self) -> [u8; 48] {
        let mut out = [0u8; 48];
        let comps = self.mag_bias.iter().flat_map(|b| [b.x, b.y, b.z]);
        for (dst, v) in out.chunks_exact_mut(2).zip(comps) {
            #[allow(clippy::cast_possible_truncation)] // within ±5000 by bias_valid
            let q = crate::fmath::round(v * 100.0).clamp(-32_768.0, 32_767.0) as i16;
            dst.copy_from_slice(&q.to_be_bytes());
        }
        out
    }

    /// A bias record's table, if every entry is valid
    pub fn bias_from_record(rec: &[u8; 48]) -> Option<[Vec3; POLARITY_STATES]> {
        let mut c = [0.0f64; 24];
        for (dst, src) in c.iter_mut().zip(rec.chunks_exact(2)) {
            *dst = f64::from(i16::from_be_bytes([*src.first()?, *src.get(1)?])) / 100.0;
        }
        let mut t = [Vec3::ZERO; POLARITY_STATES];
        for (b, v) in t.iter_mut().zip(c.chunks_exact(3)) {
            *b = Vec3::new(*v.first()?, *v.get(1)?, *v.get(2)?);
        }
        t.iter().all(Calibration::bias_valid).then_some(t)
    }
}

/// One cycle's sensor readings and gates
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    /// Magnetometer, sensor = body axes, µT (before the bias correction)
    pub field: Vec3,
    /// Sun vector, its length the intensity
    pub sun: Vec3,
    /// Gyro, sensor axes, rad/s (before the alignment), if there is one
    pub gyro: Option<Vec3>,
    /// The Jetson carrier is powered or the exciter keyed: the sample is
    /// not used for actuation
    pub gated: bool,
    /// The coil-current readbacks were not zero at the sample
    pub coils_driven: bool,
    /// No magnetometer reading this cycle (the part not answering): no
    /// actuation either
    pub mag_missing: bool,
}

/// What a control cycle did that the caller reports
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cycle {
    /// The detumble timed out (a fault: back to IDLE)
    pub detumble_timeout: bool,
    /// The detumble completed (now SUNPOINT)
    pub detumble_complete: bool,
    /// The magnetometer sample was taken with the coils on or not yet
    /// settled, and was not used
    pub mag_rejected: bool,
    /// The sample was taken with the Jetson on or the exciter keyed, and
    /// not used for actuation (coils off this cycle)
    pub mag_gated: bool,
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
    /// The gyro's body rate in the latest cycle, rad/s (`None`: no gyro)
    pub gyro: Option<Vec3>,
    /// The control gains ([`Gains::FLIGHT`])
    pub gains: Gains,
    /// Coil burst and measurement window ([`CoilTiming::FLIGHT`])
    pub timing: CoilTiming,
    /// Spin axis and inertia ([`Geometry::FLIGHT`]; ADCS_SET_GEOMETRY)
    pub geometry: Geometry,
    /// When the coils' latest burst ends, ms (`None`: not driven since init)
    pub coils_off_ms: Option<u64>,
    /// Magnetometer samples discarded because the coils were on or had not
    /// settled
    pub mag_rejected: u32,
    /// Samples taken while gated (Jetson on, exciter keyed), not used
    pub mag_gated: u32,
    /// Gyro alignment and magnetometer biases
    pub calibration: Calibration,
    /// The rods' polarity state ([`crate::mtq::polarity`])
    pub polarity: u8,
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
            gyro: None,
            gains: Gains::FLIGHT,
            timing: CoilTiming::FLIGHT,
            geometry: Geometry::FLIGHT,
            coils_off_ms: None,
            mag_rejected: 0,
            mag_gated: 0,
            calibration: Calibration::IDENTITY,
            polarity: 0,
        }
    }

    /// Whether a magnetometer sample at `read_ms` saw the coils off and
    /// settled
    pub fn coils_quiet(&self, read_ms: u64) -> bool {
        self.coils_off_ms.is_none_or(|off| read_ms >= off.saturating_add(self.timing.settle_ms))
    }

    /// `adcs_init()`: IDLE, coils off, everything cleared but the
    /// calibration
    pub fn init(&mut self) {
        *self = Adcs { initialized: true, calibration: self.calibration, ..Adcs::new() };
    }

    /// The mode (SAFE before init)
    pub fn mode(&self) -> u32 {
        if self.initialized { self.state.mode } else { mode::SAFE }
    }

    /// Body rate from the change in the field between two valid readings
    /// more than 10 ms apart: an inertially fixed field gives
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
        if let Some(w) = self.gyro {
            // The gyro measures all three axes, the field only two
            s.angular_rate_est = w;
        }
        s.rate_magnitude = s.angular_rate_est.magnitude();
    }

    /// One DETUMBLE cycle: timeout check, B-dot, rate convergence.
    /// `detumble_start_ms` 0 means "not started", so a detumble that began
    /// at time 0 is never timed out (as in the C)
    fn detumble_step(&mut self, now_ms: u64, dipole: &mut Vec3, cycle: &mut Cycle) {
        let s = &mut self.state;
        if s.detumble_start_ms > 0 && now_ms.wrapping_sub(s.detumble_start_ms) > DETUMBLE_TIMEOUT_MS {
            // A fault, not a completed detumble (ADCS_DESIGN 5.2, 5.3)
            cycle.detumble_timeout = true;
            s.fault_count = s.fault_count.wrapping_add(1);
            s.mode = mode::IDLE;
            return;
        }
        *dipole = bdot_with(self.gains.bdot, &s.mag);
        // With the gyro the spacecraft is captured once the rate across +Y
        // is below the threshold and the spin about +Y below twice the
        // SUNPOINT spin: SUNPOINT then sets the spin, and a reset in the
        // mission (the spacecraft already spinning about +Y) needs no despin.
        // Without it, the magnetometer estimate's magnitude (ADCS_DESIGN §5.1)
        let settled = if self.gyro.is_some() {
            let w = s.angular_rate_est;
            let a = self.geometry.sun_axis;
            let along = w.dot(&a);
            let across = Vec3 { x: w.x - along * a.x, y: w.y - along * a.y, z: w.z - along * a.z };
            across.magnitude() < DETUMBLE_RATE_THRESHOLD && along.abs() < 2.0 * self.gains.spin_rate
        } else {
            // A rejected sample leaves the estimate stale: not evidence of a low rate
            s.mag.valid && s.rate_magnitude < DETUMBLE_RATE_THRESHOLD
        };
        if settled {
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
        self.update_with_gyro(now_ms, read_ms, field, sun, None)
    }

    /// [`Adcs::update`] with the gyro's body rate (rad/s), when there is one:
    /// it is then the rate estimate, so the detumble ends only when the
    /// rate about the field is low too, and the sun-pointing law's rate
    pub fn update_with_gyro(
        &mut self,
        now_ms: u64,
        read_ms: u64,
        field: Vec3,
        sun: Vec3,
        gyro: Option<Vec3>,
    ) -> Option<Cycle> {
        self.update_cycle(now_ms, read_ms, &Reading { field, sun, gyro, ..Reading::default() })
    }

    /// One control cycle on `r`: the calibration applied, then as
    /// [`Adcs::update_with_gyro`]; a gated sample is not used and the coils
    /// stay off for the cycle
    pub fn update_cycle(&mut self, now_ms: u64, read_ms: u64, r: &Reading) -> Option<Cycle> {
        if !self.initialized {
            return None;
        }
        self.gyro = r.gyro.map(|w| self.calibration.gyro_apply(w));
        let mut cycle = Cycle::default();
        let mut hold = false;
        read_magnetometer(&mut self.state.mag, self.calibration.mag_apply(r.field, self.polarity), read_ms);
        if r.mag_missing {
            self.state.mag.valid = false;
            hold = true;
        } else if !self.coils_quiet(read_ms) || r.coils_driven {
            // Taken in the coils' own field: not used (no B-dot, no rate
            // estimate this cycle)
            self.state.mag.valid = false;
            self.mag_rejected = self.mag_rejected.wrapping_add(1);
            cycle.mag_rejected = true;
        } else if r.gated {
            self.state.mag.valid = false;
            self.mag_gated = self.mag_gated.wrapping_add(1);
            cycle.mag_gated = true;
            hold = true;
        }
        let sun = r.sun;
        read_sun_sensors(&mut self.state.sun, sun, read_ms);
        self.estimate_rate(now_ms);

        let mut dipole = Vec3::ZERO;
        match self.state.mode {
            mode::DETUMBLE => self.detumble_step(now_ms, &mut dipole, &mut cycle),
            mode::SUNPOINT => {
                if self.state.sun.sun_visible {
                    let gains = if self.gyro.is_some() {
                        self.gains
                    } else {
                        Gains { nutation: self.gains.nutation_no_gyro, ..self.gains }
                    };
                    dipole = spin_sunpoint_geo(
                        &gains,
                        &self.geometry,
                        &self.state.sun,
                        &self.state.mag,
                        self.state.angular_rate_est,
                    );
                } else {
                    self.state.mode = mode::ECLIPSE; // no sun
                }
            }
            // Eclipse: drift, no control, until the sun is back
            mode::ECLIPSE if self.state.sun.sun_visible => self.state.mode = mode::SUNPOINT,
            _ => {} // ECLIPSE without sun, SAFE, IDLE: no active control
        }

        if (self.state.mode == mode::DETUMBLE || self.state.mode == mode::SUNPOINT) && !hold {
            let burst = self.timing.burst(&dipole);
            self.apply_burst(&burst, now_ms);
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

    /// Command the coils for one burst from `now_ms`: the driver switches
    /// them off [`CoilTiming::on_ms`] later
    pub fn apply_burst(&mut self, dipole: &Vec3, now_ms: u64) {
        self.apply_dipole(dipole);
        self.polarity = crate::mtq::polarity(self.polarity, &crate::mtq::drive(dipole, crate::mtq::VM_DEFAULT_V));
        self.coils_off_ms = Some(now_ms.saturating_add(self.timing.on_ms));
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

    fn sun(v: Vec3) -> SunSensorData {
        SunSensorData { sun_vector: v, intensity: 1.0, sun_visible: true, ..SunSensorData::default() }
    }

    fn field(v: Vec3) -> MagData {
        MagData { field: v, valid: true, ..MagData::default() }
    }

    /// The torque the coils give: m x B (B in T)
    fn torque(m: Vec3, b_ut: Vec3) -> Vec3 {
        m.cross(&Vec3::new(b_ut.x * 1e-6, b_ut.y * 1e-6, b_ut.z * 1e-6))
    }

    #[test]
    fn spin_sunpoint_rests_spinning_about_its_axis_on_the_sun() {
        let b = Vec3::new(20.0, -10.0, 35.0);
        // A diagonal inertia: the rest state is the spin about +Y
        let diag = Geometry { sun_axis: SUN_AXIS, inertia: Inertia::diagonal(INERTIA) };
        let spin = Vec3::new(0.0, SPIN_RATE, 0.0);
        let m = spin_sunpoint_geo(&Gains::FLIGHT, &diag, &sun(SUN_AXIS), &field(b), spin);
        assert!(m.magnitude() < 1e-12, "at rest: {m:?}");
        // Each one-wing preset spins about its major axis (an eigenvector of
        // its tensor, to the 4 decimals given): at rest there too
        for n in 1..4u8 {
            let g = Geometry::preset(n).unwrap();
            let a = g.sun_axis;
            let w = Vec3::new(a.x * SPIN_RATE, a.y * SPIN_RATE, a.z * SPIN_RATE);
            let m = spin_sunpoint_geo(&Gains::FLIGHT, &g, &sun(a), &field(b), w);
            assert!(m.magnitude() < 2e-3, "preset {n}: {m:?}");
            assert!((a.magnitude() - 1.0).abs() < 1e-3, "{n}");
            assert!(a.y > 0.95, "{n}: within 18° of +Y");
        }
        assert_eq!(Geometry::preset(4), None);
        assert_eq!(Geometry::default(), Geometry::FLIGHT);
        // The tensor's moment about +Y is Iyy
        assert!((Inertia::DEPLOYED.about(&SUN_AXIS) - INERTIA.y).abs() < 1e-12);
    }

    #[test]
    fn spin_sunpoint_turns_the_momentum_toward_the_sun_and_damps_nutation() {
        let b = Vec3::new(20.0, -10.0, 35.0);
        let x = Vec3::new(1.0, 0.0, 0.0);
        // At rest, sun on +X: the momentum wanted is along +X
        let m = spin_sunpoint(&sun(x), &field(b), Vec3::ZERO);
        assert!(m.magnitude() > 0.0);
        assert!(m.magnitude() <= MAX_DIPOLE + 1e-12);
        assert!(torque(m, b).dot(&x) > 0.0);
        // Eclipse: only the rate across +Y is damped, the spin is kept
        let shade = SunSensorData::default();
        let w = Vec3::new(0.01, SPIN_RATE, -0.02);
        let t = torque(spin_sunpoint(&shade, &field(b), w), b);
        assert!(t.dot(&Vec3::new(w.x, 0.0, w.z)) < 0.0);
        let spin_only = spin_sunpoint(&shade, &field(b), Vec3::new(0.0, SPIN_RATE, 0.0));
        assert!(spin_only.magnitude() < 1e-12);
    }

    #[test]
    fn spin_sunpoint_needs_a_field() {
        let w = Vec3::new(0.1, 0.0, 0.0);
        assert_eq!(spin_sunpoint(&sun(SUN_AXIS), &MagData::default(), w), Vec3::ZERO);
        assert_eq!(spin_sunpoint(&sun(SUN_AXIS), &field(Vec3::new(0.5, 0.0, 0.0)), w), Vec3::ZERO);
    }

    #[test]
    fn the_gyro_is_the_rate_estimate() {
        let mut a = Adcs::new();
        a.init();
        let b = Vec3::new(30.0, 0.0, 40.0);
        let w = Vec3::new(0.0, 0.0, 0.05); // about an axis the field cannot see alone
        let _ = a.update_with_gyro(1000, 1000, b, Vec3::ZERO, Some(w));
        let _ = a.update_with_gyro(2000, 2000, b, Vec3::ZERO, Some(w));
        assert_eq!(a.state.angular_rate_est, w);
        assert_eq!(a.state.rate_magnitude, 0.05);
        assert_eq!(a.gyro, Some(w));
    }

    #[test]
    fn with_the_gyro_a_fast_spin_about_the_axis_is_not_captured() {
        let b = Vec3::new(30.0, 0.0, 40.0);
        for (spin, captured) in [(1.9 * SPIN_RATE, true), (2.1 * SPIN_RATE, false)] {
            let mut a = Adcs::new();
            a.init();
            assert!(a.set_mode(mode::DETUMBLE, 1000).is_ok());
            let w = Vec3::new(0.001, spin, 0.0);
            let mut t = 1000;
            while t <= 1000 + DETUMBLE_SETTLING_MS + 3000 {
                let _ = a.update_with_gyro(t, t, b, Vec3::ZERO, Some(w));
                t += 1000;
            }
            assert_eq!(a.state.detumble_complete, captured, "{spin}");
        }
    }

    #[test]
    fn the_nutation_gain_depends_on_whether_there_is_a_gyro() {
        const { assert!(NUTATION_GAIN_NO_GYRO < NUTATION_GAIN) };
        let b = Vec3::new(30.0, 0.0, 40.0);
        let sun_y = Vec3::new(0.0, 1.0, 0.0);
        let w = Vec3::new(0.02, SPIN_RATE, -0.01);
        for gyro in [Some(w), None] {
            let mut a = Adcs::new();
            a.init();
            let _ = a.update_with_gyro(1000, 1000, b, Vec3::ZERO, gyro);
            assert!(a.set_mode(mode::SUNPOINT, 1000).is_ok());
            // The field turns, so the estimate without the gyro is not zero
            let _ = a.update_with_gyro(2000, 2000, Vec3::new(31.0, 1.0, 39.0), sun_y, gyro);
            let g =
                if gyro.is_some() { Gains::FLIGHT } else { Gains { nutation: NUTATION_GAIN_NO_GYRO, ..Gains::FLIGHT } };
            let want = spin_sunpoint_geo(&g, &Geometry::FLIGHT, &a.state.sun, &a.state.mag, a.state.angular_rate_est);
            let got = a.state.mtq.dipole;
            let other = Gains { nutation: if gyro.is_some() { NUTATION_GAIN_NO_GYRO } else { NUTATION_GAIN }, ..g };
            let other =
                spin_sunpoint_geo(&other, &Geometry::FLIGHT, &a.state.sun, &a.state.mag, a.state.angular_rate_est);
            assert!((other.x - want.x).abs() + (other.z - want.z).abs() > 1e-9, "the test tells the gains apart");
            let want = a.timing.burst(&want);
            assert!((got.x - want.x).abs() < 1e-15, "{gyro:?}");
            assert!((got.y - want.y).abs() < 1e-15, "{gyro:?}");
            assert!((got.z - want.z).abs() < 1e-15, "{gyro:?}");
        }
    }

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
        mag.field_rate = Vec3::new(5.0, -10.0, 2.0);
        let m = bdot(&mag);
        assert!(close(m.x, -0.05, 1e-12));
        assert!(close(m.y, 0.10, 1e-12));
        assert!(close(m.z, -0.02, 1e-12));
        mag.field_rate = Vec3::new(1500.0, -2000.0, 0.0);
        let m = bdot(&mag);
        assert!(close(m.magnitude(), MAX_DIPOLE, 1e-12));
        assert!(close(m.x, -0.6 * MAX_DIPOLE, 1e-12));
        assert!(close(m.y, 0.8 * MAX_DIPOLE, 1e-12));
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
    fn the_magnetometer_is_read_only_with_the_coils_off_and_settled() {
        let mut a = Adcs::new();
        a.init();
        let t0 = 10_000u64;
        let _ = a.set_mode(mode::DETUMBLE, t0);
        let sun = Vec3::new(1.0, 0.0, 0.0);
        let mut field = Vec3::new(45.0, 0.0, 0.0);
        // Never driven: the first sample is good, and starts a burst
        assert!(a.coils_quiet(t0));
        let c = a.update(t0, t0, field, sun).unwrap();
        assert!(!c.mag_rejected);
        assert!(a.state.mag.valid);
        assert_eq!(a.coils_off_ms, Some(t0 + MTQ_ON_MS));
        // The next cycle, 1 s on: the coils went off 100 ms ago
        field = turned(field, 0.1, 1.0);
        let c = a.update(t0 + 1000, t0 + 1000, field, sun).unwrap();
        assert!(!c.mag_rejected);
        assert!(a.state.mag.valid);
        assert!(a.state.mtq.enabled);
        assert!(a.state.mtq.dipole.magnitude() > 0.0, "B-dot acts on a clean sample");
        // Read 20 ms after the burst ended (not settled), or during it: not used
        for late in [MTQ_ON_MS + 20, MTQ_ON_MS - 100] {
            let mut b = a;
            let t = t0 + 1000 + late;
            let c = b.update(t, t, turned(field, 0.1, 1.0), sun).unwrap();
            assert!(c.mag_rejected, "{late}");
            assert!(!b.state.mag.valid, "{late}");
            assert_eq!(b.mag_rejected, 1);
            assert_eq!(b.state.mtq.dipole.magnitude(), 0.0, "no B-dot on a rejected sample");
        }
        // Exactly settled is good
        let t = t0 + 1000 + MTQ_ON_MS + MAG_SETTLE_MS;
        assert!(a.coils_quiet(t));
        assert!(!a.coils_quiet(t - 1));
        // Coils driven for the whole period (no window): every sample after
        // the first is rejected, and counted
        let mut n = Adcs::new();
        n.init();
        n.timing = CoilTiming { on_ms: CONTROL_PERIOD_MS, settle_ms: MAG_SETTLE_MS };
        let _ = n.set_mode(mode::DETUMBLE, t0);
        for k in 0..5u64 {
            let t = t0 + k * 1000;
            let _ = n.update(t, t, turned(field, 0.1, k as f64), sun);
        }
        assert_eq!(n.mag_rejected, 4);
        // Without a gyro the stale estimate never completes the detumble
        for k in 5..400u64 {
            let t = t0 + k * 1000;
            let c = n.update(t, t, field, sun).unwrap();
            assert!(!c.detumble_complete);
        }
        assert_eq!(n.state.settling_start_ms, 0);
    }

    #[test]
    fn the_burst_delivers_the_commanded_impulse() {
        let t = CoilTiming::FLIGHT;
        let cmd = Vec3::new(0.09, -0.045, 0.0);
        let b = t.burst(&cmd);
        // dipole x on-time = command x period
        let period = CONTROL_PERIOD_MS as f64;
        let on = MTQ_ON_MS as f64;
        assert!((b.x * on - cmd.x * period).abs() < 1e-12);
        assert!((b.y * on - cmd.y * period).abs() < 1e-12);
        // Saturated at the coil limit, direction kept
        let big = t.burst(&Vec3::new(0.2, 0.2, 0.0));
        assert!((big.magnitude() - MAX_DIPOLE).abs() < 1e-12);
        assert!((big.x - big.y).abs() < 1e-12);
        assert_eq!(CoilTiming { on_ms: 0, settle_ms: 0 }.burst(&cmd), Vec3::ZERO);
        assert_eq!(CoilTiming::default(), CoilTiming::FLIGHT);
        // The direct command is one burst too
        let mut a = Adcs::new();
        a.init();
        a.apply_burst(&cmd, 5_000);
        assert_eq!(a.state.mtq.dipole, cmd);
        assert_eq!(a.coils_off_ms, Some(5_000 + MTQ_ON_MS));
    }

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
        for _ in 0..=(DETUMBLE_TIMEOUT_MS / 10_000) {
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
        assert!(close(m.x, 0.064, 1e-12), "{m:?}");
        assert!(close(m.y, -0.036, 1e-12), "{m:?}");
        assert!(close(m.z, -0.048, 1e-12), "{m:?}");
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
        assert!(close(w.x, 80.0 / 2196.0, 1e-12));
        assert!(close(w.y, -130.0 / 2196.0, 1e-12));
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

    /// Mutation check: the detumble timeout is strictly after the limit, is
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

    #[test]
    fn calibration_records_and_limits() {
        let mut c = Calibration::IDENTITY;
        assert_eq!(Calibration::default(), c);
        assert!(Calibration::gyro_valid(&c.gyro));
        // A 0.1° rotation about Z
        let (sn, cs) = (0.1f64.to_radians().sin(), 0.1f64.to_radians().cos());
        c.gyro = [[cs, -sn, 0.0], [sn, cs, 0.0], [0.0, 0.0, 1.0]];
        let w = c.gyro_apply(Vec3::new(1.0, 0.0, 0.0));
        assert!((w.y - sn).abs() < 1e-15);
        assert!((w.x - cs).abs() < 1e-15);
        let back = Calibration::gyro_from_record(&c.gyro_record()).unwrap();
        for (a, b) in back.iter().flatten().zip(c.gyro.iter().flatten()) {
            assert!((a - b).abs() < 1e-7);
        }
        assert!(!Calibration::gyro_valid(&[[2.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.5]]), "element too big");
        assert!(!Calibration::gyro_valid(&[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 0.9]]), "determinant");
        assert!(!Calibration::gyro_valid(&[[f64::NAN, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]));
        assert_eq!(Calibration::gyro_from_record(&[0u8; 36]), None);
        c.mag_bias[5] = Vec3::new(0.25, -1.5, 49.99);
        let t = Calibration::bias_from_record(&c.bias_record()).unwrap();
        assert_eq!(t[5], Vec3::new(0.25, -1.5, 49.99));
        assert_eq!(c.mag_apply(Vec3::new(30.0, 0.0, 40.0), 5), Vec3::new(29.75, 1.5, 40.0 - 49.99));
        assert!(!Calibration::bias_valid(&Vec3::new(0.0, 51.0, 0.0)));
        let mut rec = c.bias_record();
        rec[0..2].copy_from_slice(&6000i16.to_be_bytes()); // 60 µT
        assert_eq!(Calibration::bias_from_record(&rec), None);
    }

    #[test]
    fn a_gated_sample_is_counted_and_the_coils_stay_off() {
        let mut a = Adcs::new();
        a.init();
        a.calibration.mag_bias[0] = Vec3::new(1.0, 0.0, 0.0);
        let _ = a.set_mode(mode::DETUMBLE, 1);
        let r = Reading { field: Vec3::new(30.0, 0.0, 40.0), sun: Vec3::new(1.0, 0.0, 0.0), ..Reading::default() };
        let c = a.update_cycle(1000, 1000, &r).unwrap();
        assert!(!c.mag_gated);
        assert!(a.state.mag.valid);
        assert_eq!(a.state.mag.field, Vec3::new(29.0, 0.0, 40.0), "bias of state 0 removed");
        let c = a.update_cycle(2000, 2000, &Reading { gated: true, field: Vec3::new(31.0, 1.0, 40.0), ..r }).unwrap();
        assert!(c.mag_gated);
        assert!(!a.state.mag.valid);
        assert_eq!(a.mag_gated, 1);
        assert!(!a.state.mtq.enabled, "coils off in a gated cycle");
        let c = a.update_cycle(3000, 3000, &Reading { coils_driven: true, ..r }).unwrap();
        assert!(c.mag_rejected, "coil current at the sample: rejected");
        assert!(!c.mag_gated, "coil current at the sample: rejected");
        let c = a.update_cycle(3500, 3500, &Reading { mag_missing: true, ..r }).unwrap();
        assert!(!c.mag_rejected, "no magnetometer: no actuation");
        assert!(!a.state.mag.valid, "no magnetometer: no actuation");
        assert!(!a.state.mtq.enabled, "no magnetometer: no actuation");
        // The gyro passes through the alignment
        a.calibration.gyro = [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        let _ = a.update_cycle(4000, 4000, &Reading { gyro: Some(Vec3::new(0.1, 0.0, 0.0)), ..r });
        assert_eq!(a.gyro, Some(Vec3::new(0.0, 0.1, 0.0)));
        // The calibration survives init
        a.init();
        assert_eq!(a.calibration.gyro[0], [0.0, 1.0, 0.0]);
    }
}
