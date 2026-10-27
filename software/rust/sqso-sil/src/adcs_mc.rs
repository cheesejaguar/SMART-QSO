//! Monte Carlo of the attitude control alone: the flight ADCS logic
//! (`sqso_fsw::adcs::Adcs`, the code the executive runs at 1 Hz) against the
//! rigid-body dynamics, the orbit, the geomagnetic field, the Sun and the
//! Earth's shadow, the disturbance torques and the solar array, with
//! modelled sensor errors (magnetometer, coarse sun sensors, gyro).
//!
//! One trial starts at separation with a random attitude and tip-off rate,
//! detumbles with the wings stowed, deploys the wings at 30 minutes (the
//! angular momentum is conserved across the change of inertia), and runs
//! on in SUNPOINT for the rest of the trial. The trial measures the
//! detumble, the pointing of +Y, the array's generation and the coil duty.
//! [`sweep`] runs trials over a grid of spin rates and gains.
//!
//! The coils are driven in bursts, as the flight timing commands them
//! (`sqso_fsw::adcs::CoilTiming`: 900 ms on, then off until the next cycle),
//! and their own field reaches the magnetometer through a coupling matrix
//! ([`Params::mag_coupling`]) that decays after the burst with the drive's
//! time constant ([`COIL_TAU_S`]). With the flight window the samples are
//! clean; with the coils on through the sample, the coupling corrupts
//! B-dot and the rate estimate, and the flight software's rejection count
//! ([`Trial::mag_rejected`]) shows it.
//!
//! Round 3 closure (P3F-G-05, C-21, C-22): the magnetometer gating of
//! SRS-F143 ([`Gating`]: samples not used and the coils off while the Jetson
//! carrier is powered or the exciter keyed, at the mission simulation's
//! duty); the AntS deploying later than the wings ([`Params::ants_s`], the
//! interval with the elements stowed carried by [`ants_stowed`]); a physical
//! sun-sensor model ([`Photodiodes`]: the six face photodiodes of the
//! ADS7138 inputs, opposite faces subtracted as `sqso_fsw::ads7138` does,
//! with their gains, grazing response, Earth albedo and the wings' shadow);
//! the sun-vector knowledge (the flight software's sun vector against the
//! true one, ADCS-F003/P003); and the wing-identification signature of a
//! one-wing deployment ([`wing_id`]).

use sqso_fsw::adcs::{self, Adcs, CoilTiming, Gains, Geometry, Inertia, Reading, Vec3};

use crate::env::atmosphere;
use crate::env::attitude::{self, Disturbance};
use crate::env::earth::{self, plate_view_factor};
use crate::env::eclipse;
use crate::env::magfield;
use crate::env::math::{MU_EARTH, OMEGA_EARTH, Quat, Rng, V3};
use crate::env::orbit::Orbit;
use crate::env::solar::{Array, ArrayInputs, N_STRINGS};
use crate::env::sun;
use crate::env::time;

/// Physics sub-step, s (the control period is 1 s; ten sub-steps resolve
/// the 900 ms coil burst)
const SUBSTEP_S: f64 = 0.1;
/// The coil field's decay after a burst ends (drive current and eddy
/// currents), s
pub const COIL_TAU_S: f64 = 0.005;
/// Rate across +Y below which the spacecraft counts as captured, rad/s
/// (0.5°/s), with the spin about +Y below twice the SUNPOINT spin
const DETUMBLED_RAD_S: f64 = 0.5 * core::f64::consts::PI / 180.0;

/// Whether the body rate `w` is captured: across +Y below 0.5°/s and the
/// spin about +Y below twice `spin` (rad/s)
pub fn captured(w: V3, spin: f64) -> bool {
    let across = (w.x * w.x + w.z * w.z).sqrt();
    across < DETUMBLED_RAD_S && w.y.abs() < 2.0 * spin.max(DETUMBLED_RAD_S / 2.0)
}
/// Wing deployment after separation, s (the deployment timer)
const WINGS_S: f64 = 1800.0;
/// Magnetorquer drive at full dipole on all three axes, W
const MTQ_FULL_W: f64 = 0.30;

/// The stowed inertia tensor (wings and AntS stowed): MASS_BUDGET 3.2 §6.5
/// "Stowed (simulation `INERTIA_STOWED`)", the mean of the CAD's CBE and MEV
/// stowed rows of §6.1, products included (+11.5, −15.3, +34.5 × 10⁻⁶;
/// round 3 closure, P3F-G-05 (c): the earlier products were those of an
/// older CAD), kg·m²
pub const STOWED: Inertia =
    Inertia { xx: 0.002_960, yy: 0.003_292, zz: 0.002_970, xy: 0.000_011_5, xz: -0.000_015_3, yz: 0.000_034_5 };

/// The tensor `t` (AntS deployed) with the four AntS elements still stowed:
/// `t` less the elements' share, which is the difference of MASS_BUDGET
/// §6.5's "NO_WINGS (antennas out)" and "Stowed" rows (both with the wings
/// stowed). The elements' share is taken as the same with the wings out (the
/// centre of mass moves by under 2 mm); with both wings out it gives
/// (3.038, 3.915, 3.672) × 10⁻³ and products (+11.9, −15.3, +16.3) × 10⁻⁶:
/// the AntS deployment then raises Iyy by 14 % (P3F-G-05 (d)). With no wing
/// out it is [`STOWED`]
pub fn ants_stowed(t: &Inertia) -> Inertia {
    let (a, s) = (Inertia::NO_WINGS, STOWED);
    Inertia {
        xx: t.xx - (a.xx - s.xx),
        yy: t.yy - (a.yy - s.yy),
        zz: t.zz - (a.zz - s.zz),
        xy: t.xy - (a.xy - s.xy),
        xz: t.xz - (a.xz - s.xz),
        yz: t.yz - (a.yz - s.yz),
    }
}

/// A rigid body with a full inertia tensor (products included; the mission
/// simulation's `Attitude` is diagonal): attitude body → ECI, body rate
#[derive(Clone, Copy, Debug)]
pub struct Body {
    /// Attitude quaternion, body → ECI
    pub q: Quat,
    /// Body rate, rad/s
    pub w: V3,
    i: [V3; 3],
    i_inv: [V3; 3],
}

fn rows(t: &Inertia) -> [V3; 3] {
    [V3::new(t.xx, t.xy, t.xz), V3::new(t.xy, t.yy, t.yz), V3::new(t.xz, t.yz, t.zz)]
}

fn mat_vec(m: &[V3; 3], v: V3) -> V3 {
    V3::new(m[0].dot(v), m[1].dot(v), m[2].dot(v))
}

fn inverse(m: &[V3; 3]) -> [V3; 3] {
    let (a, b, c) = (m[0], m[1], m[2]);
    let det = a.dot(b.cross(c));
    // Rows of the inverse are the cross products of the columns' pairs; for
    // a symmetric matrix the columns are the rows
    let r0 = b.cross(c) / det;
    let r1 = c.cross(a) / det;
    let r2 = a.cross(b) / det;
    // (inverse of a symmetric matrix is symmetric: these are its columns, and rows)
    [r0, r1, r2]
}

impl Body {
    /// A body of tensor `t`
    pub fn new(q: Quat, w: V3, t: &Inertia) -> Body {
        let i = rows(t);
        Body { q: q.normalize(), w, i, i_inv: inverse(&i) }
    }

    /// I w
    pub fn momentum(&self, w: V3) -> V3 {
        mat_vec(&self.i, w)
    }

    /// Change the tensor keeping the angular momentum (the wings swing out)
    pub fn set_tensor(&mut self, t: &Inertia) {
        let h = self.momentum(self.w);
        self.i = rows(t);
        self.i_inv = inverse(&self.i);
        self.w = mat_vec(&self.i_inv, h);
    }

    fn wdot(&self, w: V3, tau: V3) -> V3 {
        mat_vec(&self.i_inv, tau - w.cross(self.momentum(w)))
    }

    /// Advance by `dt` under a body torque held over the step (RK4)
    pub fn step(&mut self, dt: f64, tau: V3) {
        let n = ((self.w.norm() * dt / 0.05).ceil() as u32).max(1);
        let h = dt / f64::from(n);
        for _ in 0..n {
            let (q0, w0) = (self.q, self.w);
            let k1q = q0.derivative(w0);
            let k1w = self.wdot(w0, tau);
            let w2 = w0 + k1w * (0.5 * h);
            let k2q = q0.add_scaled(k1q, 0.5 * h).derivative(w2);
            let k2w = self.wdot(w2, tau);
            let w3 = w0 + k2w * (0.5 * h);
            let k3q = q0.add_scaled(k2q, 0.5 * h).derivative(w3);
            let k3w = self.wdot(w3, tau);
            let w4 = w0 + k3w * h;
            let k4q = q0.add_scaled(k3q, h).derivative(w4);
            let k4w = self.wdot(w4, tau);
            self.w = w0 + (k1w + k2w * 2.0 + k3w * 2.0 + k4w) * (h / 6.0);
            self.q = q0
                .add_scaled(k1q, h / 6.0)
                .add_scaled(k2q, h / 3.0)
                .add_scaled(k3q, h / 3.0)
                .add_scaled(k4q, h / 6.0)
                .normalize();
        }
    }

    /// An ECI vector in the body frame
    pub fn to_body(&self, v: V3) -> V3 {
        self.q.rotate_inv(v)
    }

    /// Gravity-gradient torque at ECI position `r`: 3μ/r³ (n̂ × I n̂)
    pub fn gravity_gradient(&self, r: V3) -> V3 {
        let rn = r.norm();
        let n = self.to_body(r / rn);
        n.cross(self.momentum(n)) * (3.0 * MU_EARTH / (rn * rn * rn))
    }
}

/// The major principal axis of `t` (unit, +Y side), by Jacobi rotations
pub fn major_axis(t: &Inertia) -> Vec3 {
    let mut a = [[t.xx, t.xy, t.xz], [t.xy, t.yy, t.yz], [t.xz, t.yz, t.zz]];
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..50 {
        for (p, q) in [(0usize, 1usize), (0, 2), (1, 2)] {
            if a[p][q].abs() < 1e-18 {
                continue;
            }
            let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
            let tt = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
            let c = 1.0 / (tt * tt + 1.0).sqrt();
            let s = tt * c;
            for row in &mut a {
                let (akp, akq) = (row[p], row[q]);
                row[p] = c * akp - s * akq;
                row[q] = s * akp + c * akq;
            }
            let (rp, rq) = (a[p], a[q]);
            for k in 0..3 {
                a[p][k] = c * rp[k] - s * rq[k];
                a[q][k] = s * rp[k] + c * rq[k];
            }
            for row in &mut v {
                let (vp, vq) = (row[p], row[q]);
                row[p] = c * vp - s * vq;
                row[q] = s * vp + c * vq;
            }
        }
    }
    let k = (0..3).max_by(|&i, &j| a[i][i].total_cmp(&a[j][j])).unwrap_or(1);
    let mut axis = Vec3::new(v[0][k], v[1][k], v[2][k]);
    if axis.y < 0.0 {
        axis = Vec3::new(-axis.x, -axis.y, -axis.z);
    }
    axis
}

/// The gyro's errors
#[derive(Clone, Copy, Debug)]
pub struct GyroModel {
    /// Turn-on bias per axis, 1σ, rad/s
    pub bias_sigma: f64,
    /// Bias instability (first-order Gauss-Markov), 1σ, rad/s
    pub bias_instability: f64,
    /// Its correlation time, s
    pub bias_tau_s: f64,
    /// Rate white noise density, rad/s/√Hz
    pub noise_density: f64,
    /// Scale-factor error per axis, 1σ (fraction)
    pub scale_sigma: f64,
    /// Quantization step, rad/s
    pub lsb: f64,
}

/// The magnetometer's and sun sensors' errors
#[derive(Clone, Copy, Debug)]
pub struct SensorModel {
    /// Magnetometer white noise per sample, 1σ, T
    pub mag_noise_t: f64,
    /// Magnetometer residual bias per axis after calibration, 1σ, T
    pub mag_bias_t: f64,
    /// Coarse sun sensor direction error, 1σ, rad
    pub css_sigma_rad: f64,
    /// Gyro (`None`: no gyro, the rate is estimated from the field)
    pub gyro: Option<GyroModel>,
}

/// The magnetometer gating of SRS-F143 (RID GNC-08): while the Jetson
/// carrier is powered (`JET_PG`) or the exciter keyed (`PTT_EXCITER`) the
/// flight software does not use the sample and leaves the coils off for the
/// cycle. The duty is the mission simulation's (SIL Version F,
/// `docs/test/SIL_MISSION_SIMULATION.md`: 9,216 Jetson power-ons and
/// 8,145 Wh at 7.3 W in 732 days, about 12.6 a day of 7.2 min, 1.5 h a day;
/// the exciter keyed 2,344,368 s, 53 min a day), laid out as long blocks:
/// the Jetson's batches of at most 10 min followed by at least 100 min of
/// rest (`sqso_fsw::payload_schedule`), the exciter in blocks of 4 to 10 min
/// started only in sunlight (the transponder's gate) — the panel's worst
/// case (P3F-G-05 (a)), not the beacon's 2 s key-downs that the simulation
/// spreads over the day
#[derive(Clone, Copy, Debug)]
pub struct Gating {
    /// Jetson on time, s a day
    pub jetson_s_per_day: f64,
    /// A Jetson block's length, uniform between, s
    pub jetson_block_s: (f64, f64),
    /// Rest after a Jetson block, at least, s
    pub jetson_rest_s: f64,
    /// Exciter keyed, s a day
    pub keyed_s_per_day: f64,
    /// A keyed block's length, uniform between, s
    pub keyed_block_s: (f64, f64),
    /// No gating before this, s after separation (the AntS hold of C-19 and
    /// commissioning come first)
    pub from_s: f64,
}

impl Gating {
    /// The mission simulation's duty: 1.5 h a day of Jetson, 53 min a day keyed
    pub const SIL: Gating = Gating {
        jetson_s_per_day: 5400.0,
        jetson_block_s: (300.0, 600.0),
        jetson_rest_s: 6000.0,
        keyed_s_per_day: 3180.0,
        keyed_block_s: (240.0, 600.0),
        from_s: 3.0 * 3600.0,
    };
}

/// A trial's gate schedule (its own random stream)
#[derive(Clone, Debug)]
struct GateState {
    g: Gating,
    rng: Rng,
    jet_until: f64,
    jet_next: f64,
    key_until: f64,
    sunlit_s: f64,
    elapsed_s: f64,
}

impl GateState {
    fn new(g: Gating, seed: u64) -> GateState {
        let mut rng = Rng::new(seed ^ 0x6a7e_d00d);
        let mean = 0.5 * (g.jetson_block_s.0 + g.jetson_block_s.1);
        let period = 86_400.0 * mean / g.jetson_s_per_day.max(1.0);
        let jet_next = g.from_s + rng.range(0.0, period);
        GateState { g, rng, jet_until: 0.0, jet_next, key_until: 0.0, sunlit_s: 0.0, elapsed_s: 0.0 }
    }

    /// Whether the cycle at `t` (s after separation) is gated: (Jetson, keyed)
    fn step(&mut self, t: f64, sunlit: bool) -> (bool, bool) {
        let g = self.g;
        if t < g.from_s {
            return (false, false);
        }
        self.elapsed_s += 1.0;
        if sunlit {
            self.sunlit_s += 1.0;
        }
        if t >= self.jet_next && t >= self.jet_until {
            let len = self.rng.range(g.jetson_block_s.0, g.jetson_block_s.1);
            let mean = 0.5 * (g.jetson_block_s.0 + g.jetson_block_s.1);
            let extra = (86_400.0 * mean / g.jetson_s_per_day.max(1.0) - mean - g.jetson_rest_s).max(0.0);
            self.jet_until = t + len;
            self.jet_next = self.jet_until + g.jetson_rest_s + self.rng.exp(1.0 / extra.max(1.0));
        }
        if sunlit && t >= self.key_until {
            // Started in sunlight only, at the rate that gives the daily total
            let mean = 0.5 * (g.keyed_block_s.0 + g.keyed_block_s.1);
            let f_sun = (self.sunlit_s / self.elapsed_s.max(1.0)).max(0.3);
            let hazard = g.keyed_s_per_day / (mean * 86_400.0 * f_sun);
            if self.rng.uniform() < hazard {
                self.key_until = t + self.rng.range(g.keyed_block_s.0, g.keyed_block_s.1);
            }
        }
        (t < self.jet_until, t < self.key_until)
    }
}

/// The six face photodiodes as the flight software reads them
/// (`sqso_fsw::ads7138`: AIN0–AIN5, +X, −X, +Y, −Y, +Z, −Z, 619 Ω
/// transimpedance, 12-bit, intensity 1 at full sun; the sun vector is the
/// difference of opposite faces, its length the intensity)
#[derive(Clone, Copy, Debug)]
pub struct Photodiodes {
    /// Noise per sample and channel, 1σ, fraction of full sun (the ADC's
    /// LSB is 3.3·10⁻⁴)
    pub noise: f64,
    /// Gain error per channel (not calibrated in flight), 1σ, fraction
    pub gain_sigma: f64,
    /// Grazing loss: the response is cos θ to 60° incidence, then falls
    /// linearly to (1 − this) × cos θ at 90° (a windowed package's Fresnel
    /// loss; to be measured, SENSOR_DESIGN §8.2)
    pub grazing: f64,
    /// No response beyond this incidence, deg (90: none; 85: a coverglass
    /// cell's Kelly cut-off)
    pub cutoff_deg: f64,
    /// Earth albedo on each face (its view factor of the sunlit Earth)
    pub albedo: bool,
    /// The wings' shadow: a stowed wing covers its ±X face's photodiode; a
    /// deployed one shadows it by the geometry of `env::solar` (the
    /// photodiode at the face centre, the panel 0.0585–0.1415 m out in the
    /// +Y plane)
    pub wing_shadow: bool,
}

impl Photodiodes {
    /// The flight sensors as modelled at CDR
    pub const FLIGHT: Photodiodes =
        Photodiodes { noise: 1e-3, gain_sigma: 0.03, grazing: 0.5, cutoff_deg: 90.0, albedo: true, wing_shadow: true };

    /// The response at incidence cosine `c`
    pub fn response(&self, c: f64) -> f64 {
        if c <= 0.0 {
            return 0.0;
        }
        let th = c.min(1.0).acos().to_degrees();
        if th >= self.cutoff_deg {
            return 0.0;
        }
        let f = if th <= 60.0 { 1.0 } else { 1.0 - self.grazing * (th - 60.0) / 30.0 };
        c * f
    }
}

/// Wing panel centre along X, m, and half extents across and along Z, m
/// (`env::solar`'s `WING_CX`, `WING_HALF`), and the body's half width, m
const WING_CX: f64 = 0.10;
const WING_HALF: (f64, f64) = (0.0415, 0.049);
const BODY_HALF_XY: f64 = 0.05;

/// Whether the ray from `p` toward `s` crosses the deployed wing centred at
/// x = `cx` (in the +Y face's plane)
fn hits_wing(p: V3, s: V3, cx: f64) -> bool {
    if s.y <= 1e-12 {
        return false;
    }
    let t = (BODY_HALF_XY - p.y) / s.y;
    if t <= 1e-9 {
        return false;
    }
    let q = p + s * t;
    (q.x - cx).abs() <= WING_HALF.0 && q.z.abs() <= WING_HALF.1
}

/// A set of photodiodes in a run: their drawn gains (own random stream)
#[derive(Clone, Debug)]
pub struct PdSensors {
    m: Photodiodes,
    gain: [f64; 6],
    rng: Rng,
}

impl PdSensors {
    /// The sensors of `m` with their gains drawn for `seed`
    pub fn new(m: Photodiodes, seed: u64) -> PdSensors {
        let mut rng = Rng::new(seed ^ 0x0c55_5eed);
        let mut gain = [1.0; 6];
        for g in &mut gain {
            *g = 1.0 + rng.normal() * m.gain_sigma;
        }
        PdSensors { m, gain, rng }
    }

    /// The six channels' intensities for the body sun direction `s` at
    /// illumination `illum`, the body nadir `nadir`, the albedo irradiance
    /// on a nadir plate over the solar flux `albedo_ratio`, at radius `r_m`,
    /// with `wings` deployed
    pub fn channels(
        &mut self,
        s: V3,
        illum: f64,
        nadir: V3,
        albedo_ratio: f64,
        r_m: f64,
        wings: [bool; 2],
    ) -> [f64; 6] {
        let normals = [V3::X, -V3::X, V3::Y, -V3::Y, V3::Z, -V3::Z];
        let alb_ref = plate_view_factor(1.0, r_m);
        let mut out = [0.0; 6];
        for (k, (o, n)) in out.iter_mut().zip(normals).enumerate() {
            let covered = self.m.wing_shadow && k < 2 && !wings[k];
            let mut i = 0.0;
            if !covered {
                let shadowed = self.m.wing_shadow
                    && k < 2
                    && wings[k]
                    && hits_wing(n * (BODY_HALF_XY + 1e-4), s, if k == 0 { WING_CX } else { -WING_CX });
                if !shadowed {
                    i += illum * self.m.response(n.dot(s));
                }
                if self.m.albedo && albedo_ratio > 0.0 {
                    i += albedo_ratio * plate_view_factor(n.dot(nadir), r_m) / alb_ref;
                }
            }
            *o = self.gain[k] * i + self.rng.normal() * self.m.noise;
        }
        out
    }

    /// The sun vector as `sqso_fsw::ads7138::Reading::sun` forms it
    pub fn read(&mut self, s: V3, illum: f64, nadir: V3, albedo_ratio: f64, r_m: f64, wings: [bool; 2]) -> V3 {
        let c = self.channels(s, illum, nadir, albedo_ratio, r_m, wings);
        V3::new(c[0] - c[1], c[2] - c[3], c[4] - c[5])
    }
}

/// One trial's inputs that the sweep holds fixed
#[derive(Clone, Copy, Debug)]
pub struct Params {
    /// Altitude, m
    pub alt_m: f64,
    /// Local time of the ascending node, h
    pub ltan_h: f64,
    /// Deployment epoch, s since J2000 (each trial adds a random day of the year)
    pub epoch: f64,
    /// Trial length, s
    pub duration_s: f64,
    /// Start of the steady-state statistics, s after separation
    pub steady_from_s: f64,
    /// Tip-off: per-axis rate, 1σ, rad/s
    pub tipoff_sigma: f64,
    /// Tip-off: largest per-axis rate (the draw is clipped), rad/s
    pub tipoff_max: f64,
    /// Residual dipole magnitude, A·m² (random direction)
    pub residual_dipole: f64,
    /// Sensor errors
    pub sensors: SensorModel,
    /// Control gains
    pub gains: Gains,
    /// Coil burst and magnetometer window
    pub timing: CoilTiming,
    /// Coil-to-magnetometer coupling: the magnetometer reads this many
    /// tesla per A·m² of coil dipole (a random matrix of this norm per
    /// trial; to be measured on the engineering model)
    pub mag_coupling: f64,
    /// The plant's inertia with the wings stowed and deployed (full tensors)
    pub plant_stowed: Inertia,
    /// With the wings (as many as `wings` says) deployed
    pub plant_deployed: Inertia,
    /// Which wings deploy (W1, W2)
    pub wings: [bool; 2],
    /// What the flight software is told: spin axis and inertia
    /// (ADCS_SET_GEOMETRY)
    pub fsw_geometry: Geometry,
    /// The AntS deploys at this time, s after separation (at or before the
    /// wings: together with them; later: the plant is [`ants_stowed`] of
    /// `plant_deployed` from the wings until then)
    pub ants_s: f64,
    /// The magnetometer gating (`None`: never gated)
    pub gating: Option<Gating>,
    /// The face photodiodes (`None`: the generic sun-vector error of
    /// [`SensorModel::css_sigma_rad`])
    pub photodiodes: Option<Photodiodes>,
}

/// What one trial measured
#[derive(Clone, Copy, Debug, Default)]
pub struct Trial {
    /// Tip-off rate, deg/s
    pub tipoff_dps: f64,
    /// Captured (rate across +Y below 0.5°/s, spin below twice the target)
    /// for 60 s, h after separation
    pub detumbled_h: Option<f64>,
    /// The flight software left DETUMBLE, h after separation
    pub fsw_detumbled_h: Option<f64>,
    /// Detumble timeouts
    pub timeouts: u32,
    /// Steady state, sunlit: mean angle of the Sun from +Y, deg
    pub sun_angle_mean_deg: f64,
    /// Steady state, sunlit: 95th percentile, deg
    pub sun_angle_p95_deg: f64,
    /// Steady state, sunlit: share of time within 15°
    pub within_15: f64,
    /// Steady state: largest Sun angle at eclipse exit, deg
    pub eclipse_exit_max_deg: f64,
    /// Steady state: −Y facing the Sun (angle > 135°) share of sunlit time
    pub anti_share: f64,
    /// Steady state: orbit-average array power (end of life), W
    pub gen_w: f64,
    /// Steady state: mean spin about +Y, deg/s
    pub spin_dps: f64,
    /// Steady state: RMS rate across +Y, deg/s
    pub transverse_dps: f64,
    /// Steady state: mean coil power, W
    pub mtq_w: f64,
    /// Magnetometer samples the flight software discarded (coils on or not settled)
    pub mag_rejected: u32,
    /// Steady state, sunlit: mean angle between the flight software's sun
    /// vector and the true one (sun-vector knowledge, ADCS-F003), deg
    pub know_mean_deg: f64,
    /// Steady state, sunlit: its 95th percentile, deg
    pub know_p95_deg: f64,
    /// Steady state, sunlit: share of samples with the knowledge error within 5°
    pub know_within_5: f64,
    /// Steady state: gated (Jetson or keyed) time, min a day
    pub gated_min_day: f64,
    /// Steady state: the mean body-rate estimate's tilt from +Y toward +X,
    /// atan2(ω̄x, ω̄y), deg (the wing signature of C-21)
    pub rate_tilt_x_deg: f64,
    /// Steady state, sunlit: the mean measured sun vector's tilt from +Y
    /// toward +X, deg
    pub sun_tilt_x_deg: f64,
    /// Largest sunlit angle of the Sun from +Y within 3 h after a late AntS
    /// deployment, deg (0: none)
    pub ants_max_deg: f64,
    /// Sunlit time with the spin at 6°/s or more and the Sun 20–55° from +Y,
    /// after the wings, min (the window of the identification of [`wing_id`])
    pub id_window_min: f64,
}

/// A gyro by name: `none`, `bmi088` (calibrated), `bmi088_uncal`, `mg370`,
/// `adis16505` (`None` if unknown)
pub fn gyro_preset(name: &str) -> Option<Option<GyroModel>> {
    let d = |x: f64| x.to_radians();
    match name {
        "none" => Some(None),
        // Bosch BMI088 gyro, ground thermal calibration and the in-orbit bias
        // estimate: bias 0.1 °/s 1σ; bias instability 2 °/h (Gauss-Markov
        // σ 5.6e-4 °/s, 100 s); 0.014 °/s/√Hz; scale 1 % (gyro_selection §6.1)
        "bmi088" => Some(Some(GyroModel {
            bias_sigma: d(0.1),
            bias_instability: d(5.6e-4),
            bias_tau_s: 100.0,
            noise_density: d(0.014),
            scale_sigma: 0.01,
            lsb: d(125.0 / 32768.0),
        })),
        // The same, uncalibrated: the datasheet's ±1 °/s typical zero-rate offset
        "bmi088_uncal" => Some(Some(GyroModel {
            bias_sigma: d(1.0),
            bias_instability: d(5.6e-4),
            bias_tau_s: 100.0,
            noise_density: d(0.014),
            scale_sigma: 0.01,
            lsb: d(125.0 / 32768.0),
        })),
        // Epson M-G370PDT0: turn-on bias 0.10 °/s 1σ (no user calibration);
        // bias instability 0.8 °/h (σ 2.2e-4 °/s, 300 s); ARW 0.03 °/√h
        // (5e-4 °/s/√Hz); scale 0.1 % (gyro_selection §6)
        "mg370" => Some(Some(GyroModel {
            bias_sigma: d(0.10),
            bias_instability: d(2.2e-4),
            bias_tau_s: 300.0,
            noise_density: d(5.0e-4),
            scale_sigma: 0.001,
            lsb: 0.0,
        })),
        // ADI ADIS16505-1: 0.0016 °/s/√Hz (ARW 0.1 °/√h), bias instability 2 °/h
        "adis16505" => Some(Some(GyroModel {
            bias_sigma: d(0.05),
            bias_instability: d(2.0 / 3600.0),
            bias_tau_s: 300.0,
            noise_density: d(0.0016),
            scale_sigma: 0.002,
            lsb: d(0.0001),
        })),
        _ => None,
    }
}

/// The magnetometer, sun sensors and gyro as the flight software reads them:
/// the true values with each sensor's errors
#[derive(Clone, Debug)]
pub struct Sensors {
    model: SensorModel,
    mag_bias: V3,
    gyro: Option<Gyro>,
    /// Coil dipole to magnetometer field, T per A·m² (rows: field axes)
    coupling: [V3; 3],
}

impl Sensors {
    /// Sensors of `model` with their turn-on errors drawn from `rng`
    pub fn new(model: SensorModel, rng: &mut Rng) -> Sensors {
        let mag_bias = V3::new(rng.normal(), rng.normal(), rng.normal()) * model.mag_bias_t;
        let gyro = model.gyro.map(|g| Gyro::new(g, rng));
        Sensors { model, mag_bias, gyro, coupling: [V3::ZERO; 3] }
    }

    /// With a coil-to-magnetometer coupling of norm `k` (T per A·m²):
    /// a random matrix, each element normal, scaled to Frobenius norm `k`
    pub fn with_coupling(mut self, rng: &mut Rng, k: f64) -> Sensors {
        let rows = [rng.unit_vector(), rng.unit_vector(), rng.unit_vector()];
        let n = (rows.iter().map(|r| r.norm2()).sum::<f64>()).sqrt();
        self.coupling = rows.map(|r| r * (k / n));
        self
    }

    /// The field the coils make at the magnetometer for the coil dipole `m`
    pub fn coil_field(&self, m: V3) -> V3 {
        V3::new(self.coupling[0].dot(m), self.coupling[1].dot(m), self.coupling[2].dot(m))
    }

    /// The magnetometer reading of the body field `b_body` (T)
    pub fn magnetometer(&self, rng: &mut Rng, b_body: V3) -> V3 {
        b_body + self.mag_bias + V3::new(rng.normal(), rng.normal(), rng.normal()) * self.model.mag_noise_t
    }

    /// The coarse sun sensors' vector for the body sun direction `sun_body`
    /// at illumination `illum` (its length is the intensity)
    pub fn sun_sensors(&self, rng: &mut Rng, sun_body: V3, illum: f64) -> V3 {
        perturb(rng, sun_body, self.model.css_sigma_rad) * illum
    }

    /// The gyro's reading of the body rate `w` a period `dt` after the last
    /// (`None`: no gyro)
    pub fn gyro(&mut self, rng: &mut Rng, w: V3, dt: f64) -> Option<V3> {
        self.gyro.as_mut().map(|g| g.read(rng, w, dt))
    }
}

/// The flight-like default inputs (the gyro set by the caller)
pub fn default_params() -> Params {
    Params {
        alt_m: 500e3,
        ltan_h: 22.5,
        epoch: time::from_utc(2028, 6, 1, 6, 0, 0.0),
        duration_s: 2.0 * 86_400.0,
        steady_from_s: 0.75 * 86_400.0,
        tipoff_sigma: 5.0_f64.to_radians(),
        tipoff_max: 15.0_f64.to_radians(),
        residual_dipole: 3e-3,
        timing: CoilTiming::FLIGHT,
        mag_coupling: 1e-4,
        plant_stowed: STOWED,
        plant_deployed: Inertia::DEPLOYED,
        wings: [true, true],
        fsw_geometry: Geometry::FLIGHT,
        ants_s: WINGS_S,
        gating: None,
        photodiodes: None,
        sensors: SensorModel {
            mag_noise_t: 50e-9,
            mag_bias_t: 300e-9,
            css_sigma_rad: 5.0_f64.to_radians(),
            gyro: None,
        },
        gains: Gains::FLIGHT,
    }
}

fn clip(x: f64, max: f64) -> f64 {
    x.clamp(-max, max)
}

/// A small rotation of `v` by a random angle of 1σ `sigma` (rad)
fn perturb(rng: &mut Rng, v: V3, sigma: f64) -> V3 {
    if sigma <= 0.0 {
        return v;
    }
    let axis = rng.unit_vector();
    let angle = rng.normal() * sigma;
    Quat::from_axis_angle(axis, angle).rotate(v)
}

fn v3(v: V3) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

/// A gyro in a run: its drawn bias and scale factors, and its drifting bias
#[derive(Clone, Copy, Debug)]
pub struct Gyro {
    m: GyroModel,
    bias: V3,
    drift: V3,
    scale: V3,
}

impl Gyro {
    /// A gyro of model `m` with its turn-on errors drawn from `rng`
    pub fn new(m: GyroModel, rng: &mut Rng) -> Gyro {
        let n = |rng: &mut Rng| rng.normal();
        Gyro {
            m,
            bias: V3::new(n(rng), n(rng), n(rng)) * m.bias_sigma,
            drift: V3::ZERO,
            scale: V3::new(1.0 + n(rng) * m.scale_sigma, 1.0 + n(rng) * m.scale_sigma, 1.0 + n(rng) * m.scale_sigma),
        }
    }

    /// One reading of the true body rate `w`, a control period `dt` after the last
    pub fn read(&mut self, rng: &mut Rng, w: V3, dt: f64) -> V3 {
        let a = (-dt / self.m.bias_tau_s).exp();
        let q = self.m.bias_instability * (1.0 - a * a).sqrt();
        self.drift = self.drift * a + V3::new(rng.normal(), rng.normal(), rng.normal()) * q;
        // White noise over the sample's bandwidth (1 Hz output, the averaged samples)
        let white = self.m.noise_density / dt.sqrt();
        let raw =
            w.hadamard(self.scale) + self.bias + self.drift + V3::new(rng.normal(), rng.normal(), rng.normal()) * white;
        let lsb = self.m.lsb;
        if lsb > 0.0 {
            V3::new((raw.x / lsb).round() * lsb, (raw.y / lsb).round() * lsb, (raw.z / lsb).round() * lsb)
        } else {
            raw
        }
    }
}

/// Run one trial with the random draws of `seed`
pub fn trial(p: &Params, seed: u64) -> Trial {
    let mut rng = Rng::new(seed);
    let epoch = p.epoch + rng.range(0.0, 365.25) * 86_400.0;
    let orbit = Orbit::sso(p.alt_m, p.ltan_h, epoch);
    let w0 = V3::new(
        clip(rng.normal() * p.tipoff_sigma, p.tipoff_max),
        clip(rng.normal() * p.tipoff_sigma, p.tipoff_max),
        clip(rng.normal() * p.tipoff_sigma, p.tipoff_max),
    );
    let mut att = Body::new(rng.quat(), w0, &p.plant_stowed);
    let dist = Disturbance { residual_dipole: rng.unit_vector() * p.residual_dipole, ..Disturbance::nominal() };
    let mut sensors = Sensors::new(p.sensors, &mut rng);
    if p.mag_coupling > 0.0 {
        // A separate stream, so the other draws are the same with or without it
        let mut crng = Rng::new(seed ^ 0x5eed_c011);
        sensors = sensors.with_coupling(&mut crng, p.mag_coupling);
    }
    let array = Array::new();
    let mut gate = p.gating.map(|g| GateState::new(g, seed));
    let mut pd = p.photodiodes.map(|m| PdSensors::new(m, seed));
    let ants_late = p.ants_s > WINGS_S;

    let mut a = Adcs::new();
    a.init();
    a.gains = p.gains;
    a.timing = p.timing;
    a.geometry = p.fsw_geometry;
    let axis = V3::new(p.fsw_geometry.sun_axis.x, p.fsw_geometry.sun_axis.y, p.fsw_geometry.sun_axis.z);
    // The coils' burst in the last control period: dipole and length, s
    let mut burst = (V3::ZERO, 0.0f64);
    let _ = a.set_mode(adcs::mode::DETUMBLE, 1);

    let mut out = Trial { tipoff_dps: w0.norm().to_degrees(), ..Trial::default() };
    let mut below_since: Option<f64> = None;
    let mut wings = false;
    let mut ants = !ants_late;
    let mut ants_at: Option<f64> = None;
    let mut angles: Vec<f64> = Vec::new();
    let mut know: Vec<f64> = Vec::new();
    let (mut gated_s, mut rate_sum, mut sun_sum, mut id_s) = (0.0, V3::ZERO, V3::ZERO, 0.0);
    let (mut gen_j, mut steady_s, mut spin_sum, mut trans_sq, mut mtq_j) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut was_dark = false;
    let mut anti = 0usize;

    let steps = (p.duration_s as u64).max(1);
    for k in 0..steps {
        let t_rel = k as f64;
        let t = epoch + t_rel;
        if !wings && t_rel >= WINGS_S {
            // The wings swing out: the body's angular momentum is kept
            att.set_tensor(&if ants_late { ants_stowed(&p.plant_deployed) } else { p.plant_deployed });
            wings = true;
        }
        if !ants && t_rel >= p.ants_s {
            // The AntS elements unroll: the momentum is kept
            att.set_tensor(&p.plant_deployed);
            ants = true;
            ants_at = Some(t_rel);
        }
        let (r, v) = orbit.state_at(t);
        let (sun_u, sun_d) = sun::sun_eci(t);
        let illum = eclipse::illumination(r, sun_u, sun_d);
        let b_eci = magfield::field_eci(r, t);
        let density = atmosphere::density_nominal(Orbit::altitude_m(r), t);
        let v_rel = v - V3::new(0.0, 0.0, OMEGA_EARTH).cross(r);
        let wd = [wings && p.wings[0], wings && p.wings[1]];

        // Sensors, then the flight law (1 Hz). The coils' field at the
        // magnetometer: full while the last burst runs, decaying after it
        let b_b = att.to_body(b_eci);
        let since_off = 1.0 - burst.1;
        let coil_share = if since_off <= 0.0 { 1.0 } else { (-since_off / COIL_TAU_S).exp() };
        let mag = sensors.magnetometer(&mut rng, b_b) + sensors.coil_field(burst.0 * coil_share);
        let sun_b = att.to_body(sun_u);
        let sun_meas = match pd.as_mut() {
            Some(s) => {
                let flux = sun::solar_flux(sun_d);
                let albedo = earth::albedo_nadir_w_m2(r.norm(), r.unit().dot(sun_u), flux) / flux;
                s.read(sun_b, illum, att.to_body(-r.unit()), albedo, r.norm(), wd)
            }
            None => sensors.sun_sensors(&mut rng, sun_b, illum),
        };
        let gyro_reading = sensors.gyro(&mut rng, att.w, 1.0).map(v3);
        let (jet, keyed) = gate.as_mut().map_or((false, false), |g| g.step(t_rel, illum > 0.5));
        let ms = (t_rel * 1000.0) as u64 + 1000;
        let reading = Reading {
            field: v3(mag * 1e6),
            sun: v3(sun_meas),
            gyro: gyro_reading,
            gated: jet || keyed,
            ..Reading::default()
        };
        let cycle = a.update_cycle(ms, ms, &reading);
        // Sun-vector knowledge: the flight software's vector against the truth at the sample
        let sv = a.state.sun.sun_vector;
        let know_err =
            if a.state.sun.sun_visible { Some(V3::new(sv.x, sv.y, sv.z).angle(sun_b).to_degrees()) } else { None };
        if cycle.is_some_and(|c| c.detumble_timeout) {
            out.timeouts += 1;
        }
        // The executive's rule: an idle ADCS in the mission is told to detumble again
        if a.mode() == adcs::mode::IDLE {
            let _ = a.set_mode(adcs::mode::DETUMBLE, ms);
        }
        if out.fsw_detumbled_h.is_none() && a.state.detumble_complete {
            out.fsw_detumbled_h = Some(t_rel / 3600.0);
        }
        let m = a.state.mtq;
        let dipole = if m.enabled { V3::new(m.dipole.x, m.dipole.y, m.dipole.z) } else { V3::ZERO };
        let on_s = if m.enabled { (p.timing.on_ms as f64 / 1000.0).min(1.0) } else { 0.0 };
        burst = (dipole, on_s);

        // Dynamics over the control period: the burst, then the window
        let n_sub = (1.0 / SUBSTEP_S).round() as u32;
        for j in 0..n_sub {
            let bb = att.to_body(b_eci);
            let v_body = att.to_body(v_rel);
            let coil = if (f64::from(j) + 0.5) * SUBSTEP_S < on_s { dipole } else { V3::ZERO };
            let torque = attitude::magnetic_torque(coil, bb)
                + dist.residual_torque(bb)
                + att.gravity_gradient(r)
                + dist.aero_torque(v_body, density, wd);
            att.step(SUBSTEP_S, torque);
        }

        // Measurements
        let rate = att.w.norm();
        if std::env::var_os("SQSO_MC_TRACE").is_some() && k % 600 == 0 {
            eprintln!(
                "{:.2} h rate {:.3} dps est {:.3} mode {} dip {:.3} illum {:.2} sun {:.1} B {:.1} uT alt {:.0} km",
                t_rel / 3600.0,
                rate.to_degrees(),
                a.state.rate_magnitude.to_degrees(),
                a.mode(),
                dipole.norm(),
                illum,
                att.to_body(sun_u).angle(V3::new(0.0, 1.0, 0.0)).to_degrees(),
                b_eci.norm() * 1e6,
                Orbit::altitude_m(r) / 1e3
            );
        }
        // Captured about the flight software's spin axis
        let along_axis = att.w.dot(axis);
        let across_axis = (att.w - axis * along_axis).norm();
        if across_axis < DETUMBLED_RAD_S && along_axis.abs() < 2.0 * p.gains.spin_rate.max(DETUMBLED_RAD_S / 2.0) {
            let since = *below_since.get_or_insert(t_rel);
            if out.detumbled_h.is_none() && t_rel - since >= 60.0 {
                out.detumbled_h = Some(since / 3600.0);
            }
        } else {
            below_since = None;
        }
        let sb = att.to_body(sun_u);
        let angle = sb.angle(V3::new(0.0, 1.0, 0.0)).to_degrees();
        if let Some(t0) = ants_at
            && t_rel - t0 <= 3.0 * 3600.0
            && illum > 0.5
        {
            out.ants_max_deg = out.ants_max_deg.max(angle);
        }
        if wings && illum > 0.5 && along_axis >= 6.0_f64.to_radians() && (20.0..=55.0).contains(&angle) {
            id_s += 1.0;
        }
        if t_rel >= p.steady_from_s {
            steady_s += 1.0;
            if jet || keyed {
                gated_s += 1.0;
            }
            let w = a.state.angular_rate_est;
            rate_sum += V3::new(w.x, w.y, w.z);
            if illum > 0.5 {
                angles.push(angle);
                if let Some(e) = know_err {
                    know.push(e);
                    sun_sum += V3::new(sv.x, sv.y, sv.z);
                }
                if angle > 135.0 {
                    anti += 1;
                }
                if was_dark {
                    out.eclipse_exit_max_deg = out.eclipse_exit_max_deg.max(angle);
                }
                was_dark = false;
            } else if illum < 0.01 {
                was_dark = true;
            }
            spin_sum += along_axis;
            trans_sq += across_axis * across_axis;
            mtq_j += MTQ_FULL_W * (dipole.norm() / (adcs::MAX_DIPOLE * 3f64.sqrt())).min(1.0) * on_s;
            if k % 10 == 0 {
                let cos_zeta = r.unit().dot(sun_u);
                let inp = ArrayInputs {
                    sun_body: sb,
                    illum,
                    flux: sun::solar_flux(sun_d),
                    nadir_body: att.to_body(-r.unit()),
                    albedo_w_m2: earth::albedo_nadir_w_m2(r.norm(), cos_zeta, sun::solar_flux(sun_d)),
                    r_m: r.norm(),
                    temp_c: [25.0; N_STRINGS],
                    years_in_orbit: 2.0,
                    wings_deployed: wd,
                };
                let p_w: f64 = array.strings(&inp).iter().map(|s| s.pmp).sum();
                gen_j += p_w * 10.0;
            }
        }
    }
    if steady_s > 0.0 {
        angles.sort_by(f64::total_cmp);
        let n = angles.len().max(1) as f64;
        out.sun_angle_mean_deg = angles.iter().sum::<f64>() / n;
        out.sun_angle_p95_deg =
            angles.get(((n * 0.95) as usize).min(angles.len().saturating_sub(1))).copied().unwrap_or(0.0);
        out.within_15 = angles.iter().filter(|&&a| a <= 15.0).count() as f64 / n;
        out.anti_share = anti as f64 / n;
        out.gen_w = gen_j / steady_s;
        out.spin_dps = (spin_sum / steady_s).to_degrees();
        out.transverse_dps = (trans_sq / steady_s).sqrt().to_degrees();
        out.mtq_w = mtq_j / steady_s;
        out.gated_min_day = gated_s / steady_s * 1440.0;
        out.rate_tilt_x_deg = rate_sum.x.atan2(rate_sum.y).to_degrees();
        out.sun_tilt_x_deg = sun_sum.x.atan2(sun_sum.y).to_degrees();
        if !know.is_empty() {
            know.sort_by(f64::total_cmp);
            let nk = know.len() as f64;
            out.know_mean_deg = know.iter().sum::<f64>() / nk;
            out.know_p95_deg = know[((nk * 0.95) as usize).min(know.len() - 1)];
            out.know_within_5 = know.iter().filter(|&&e| e <= 5.0).count() as f64 / nk;
        }
    }
    out.id_window_min = id_s / 60.0;
    out.mag_rejected = a.mag_rejected;
    out
}

/// Statistics of a set of trials
#[derive(Clone, Debug, Default)]
pub struct Summary {
    /// Trials
    pub n: usize,
    /// Trials that never detumbled (true rate)
    pub not_detumbled: usize,
    /// Detumble time, h: median, 95th percentile, worst
    pub detumble_h: (f64, f64, f64),
    /// Flight-software detumble declared, h: median, 95th percentile, worst
    pub fsw_detumble_h: (f64, f64, f64),
    /// Trials with a detumble timeout
    pub with_timeout: usize,
    /// Sun angle mean (mean over trials), and the worst trial's 95th percentile, deg
    pub sun_mean_deg: f64,
    /// Worst trial's 95th percentile, deg
    pub sun_p95_worst_deg: f64,
    /// Mean share within 15°
    pub within_15: f64,
    /// Worst eclipse-exit angle over the trials, deg
    pub eclipse_exit_worst_deg: f64,
    /// Trials that settled with −Y on the Sun (more than half their sunlit time)
    pub anti: usize,
    /// Orbit-average generation: mean and worst trial, W
    pub gen_w: (f64, f64),
    /// Mean spin, deg/s; mean transverse rate, deg/s
    pub spin_dps: f64,
    /// Mean transverse rate, deg/s
    pub transverse_dps: f64,
    /// Mean coil power, W
    pub mtq_w: f64,
    /// Magnetometer samples the flight software discarded, mean per trial
    pub mag_rejected: f64,
    /// Sun-vector knowledge: mean error (mean over trials), deg
    pub know_mean_deg: f64,
    /// Sun-vector knowledge: the worst trial's 95th percentile, deg
    pub know_p95_worst_deg: f64,
    /// Sun-vector knowledge: share within 5°, the worst trial
    pub know_within_5_worst: f64,
    /// Gated time, min a day (mean over trials)
    pub gated_min_day: f64,
    /// Mean body-rate tilt toward +X: mean, least and greatest over the trials, deg
    pub rate_tilt_x_deg: (f64, f64, f64),
    /// Mean measured sun-vector tilt toward +X, mean over trials, deg
    pub sun_tilt_x_deg: f64,
    /// Worst sunlit angle within 3 h of a late AntS deployment, deg
    pub ants_max_worst_deg: f64,
    /// The identification window: median and least over the trials, min
    pub id_window_min: (f64, f64),
}

fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[((sorted.len() as f64 - 1.0) * q).round() as usize]
}

/// The statistics of `trials`
pub fn summarize(trials: &[Trial]) -> Summary {
    let n = trials.len();
    let mut det: Vec<f64> = trials.iter().filter_map(|t| t.detumbled_h).collect();
    let mut fsw: Vec<f64> = trials.iter().filter_map(|t| t.fsw_detumbled_h).collect();
    det.sort_by(f64::total_cmp);
    fsw.sort_by(f64::total_cmp);
    let nf = n.max(1) as f64;
    Summary {
        n,
        not_detumbled: n - det.len(),
        detumble_h: (pct(&det, 0.5), pct(&det, 0.95), det.last().copied().unwrap_or(f64::NAN)),
        fsw_detumble_h: (pct(&fsw, 0.5), pct(&fsw, 0.95), fsw.last().copied().unwrap_or(f64::NAN)),
        with_timeout: trials.iter().filter(|t| t.timeouts > 0).count(),
        sun_mean_deg: trials.iter().map(|t| t.sun_angle_mean_deg).sum::<f64>() / nf,
        sun_p95_worst_deg: trials.iter().map(|t| t.sun_angle_p95_deg).fold(0.0, f64::max),
        within_15: trials.iter().map(|t| t.within_15).sum::<f64>() / nf,
        eclipse_exit_worst_deg: trials.iter().map(|t| t.eclipse_exit_max_deg).fold(0.0, f64::max),
        anti: trials.iter().filter(|t| t.anti_share > 0.5).count(),
        gen_w: (
            trials.iter().map(|t| t.gen_w).sum::<f64>() / nf,
            trials.iter().map(|t| t.gen_w).fold(f64::INFINITY, f64::min),
        ),
        spin_dps: trials.iter().map(|t| t.spin_dps).sum::<f64>() / nf,
        transverse_dps: trials.iter().map(|t| t.transverse_dps).sum::<f64>() / nf,
        mtq_w: trials.iter().map(|t| t.mtq_w).sum::<f64>() / nf,
        mag_rejected: trials.iter().map(|t| f64::from(t.mag_rejected)).sum::<f64>() / nf,
        know_mean_deg: trials.iter().map(|t| t.know_mean_deg).sum::<f64>() / nf,
        know_p95_worst_deg: trials.iter().map(|t| t.know_p95_deg).fold(0.0, f64::max),
        know_within_5_worst: trials.iter().map(|t| t.know_within_5).fold(1.0, f64::min),
        gated_min_day: trials.iter().map(|t| t.gated_min_day).sum::<f64>() / nf,
        rate_tilt_x_deg: (
            trials.iter().map(|t| t.rate_tilt_x_deg).sum::<f64>() / nf,
            trials.iter().map(|t| t.rate_tilt_x_deg).fold(f64::INFINITY, f64::min),
            trials.iter().map(|t| t.rate_tilt_x_deg).fold(f64::NEG_INFINITY, f64::max),
        ),
        sun_tilt_x_deg: trials.iter().map(|t| t.sun_tilt_x_deg).sum::<f64>() / nf,
        ants_max_worst_deg: trials.iter().map(|t| t.ants_max_deg).fold(0.0, f64::max),
        id_window_min: {
            let mut w: Vec<f64> = trials.iter().map(|t| t.id_window_min).collect();
            w.sort_by(f64::total_cmp);
            (pct(&w, 0.5), w.first().copied().unwrap_or(f64::NAN))
        },
    }
}

/// The commissioning identification of a one-wing deployment (decision
/// C-21): what the ground sees over `spins` turns of a spin about +Y at
/// `spin_dps` with the Sun `theta_deg` from +Y
#[derive(Clone, Copy, Debug, Default)]
pub struct WingIdStats {
    /// Draws
    pub n: usize,
    /// Lag of the array power's first harmonic behind the +Z photodiode's,
    /// in spin phase: mean, standard deviation, and the largest departure
    /// from the mean over the draws, deg
    pub lag_deg: (f64, f64, f64),
    /// The power's first-harmonic amplitude (bus side): mean and least, W
    pub amp_w: (f64, f64),
    /// Mean array power (bus side), W
    pub mean_w: f64,
    /// Draws whose lag fell within ±45° of the signature of the wing deployed
    pub correct: usize,
}

fn fit_harmonic(t: &[f64], y: &[f64], w: f64) -> (f64, f64, f64) {
    // Least squares of y = c + a cos wt + b sin wt
    let mut m = [[0.0f64; 3]; 3];
    let mut r = [0.0f64; 3];
    for (&ti, &yi) in t.iter().zip(y) {
        let f = [1.0, (w * ti).cos(), (w * ti).sin()];
        for i in 0..3 {
            r[i] += f[i] * yi;
            for j in 0..3 {
                m[i][j] += f[i] * f[j];
            }
        }
    }
    let det = |m: &[[f64; 3]; 3]| {
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    };
    let d = det(&m);
    let mut x = [0.0; 3];
    for (k, xk) in x.iter_mut().enumerate() {
        let mut mk = m;
        for i in 0..3 {
            mk[i][k] = r[i];
        }
        *xk = det(&mk) / d;
    }
    (x[0], x[1], x[2])
}

fn wrap_deg(a: f64) -> f64 {
    let mut a = a % 360.0;
    if a > 180.0 {
        a -= 360.0;
    }
    if a <= -180.0 {
        a += 360.0;
    }
    a
}

/// The signature for wing `wing` (0: only W1 deployed, 1: only W2; for the
/// amplitude criterion, 2: both, 3: neither) over
/// `n` draws: random spin phase and nadir direction,
/// the photodiodes `pd` (own gains each draw), the gyro `gyro` and the bus
/// power telemetry `PA` with 1σ noise of 20 mW plus 1 %; the array at
/// commissioning (0.1 year), 25 °C, the MPPT's 0.95; `albedo_w_m2` on a
/// nadir plate. The ground fits the +Z photodiode and the power with the
/// gyro's spin rate and takes the lag of the power's harmonic
#[allow(clippy::too_many_arguments)]
pub fn wing_id(
    wing: usize,
    theta_deg: f64,
    spin_dps: f64,
    spins: f64,
    n: usize,
    seed: u64,
    pd: Photodiodes,
    gyro: GyroModel,
    albedo_w_m2: f64,
    signature_deg: Option<f64>,
) -> WingIdStats {
    let array = Array::new();
    let wings = match wing {
        0 => [true, false],
        1 => [false, true],
        2 => [true, true],
        _ => [false, false],
    };
    let w = spin_dps.to_radians();
    let th = theta_deg.to_radians();
    let samples = (spins * 360.0 / spin_dps).ceil() as usize;
    let r_m = 6.878e6;
    let flux = 1361.0;
    let mut lags = Vec::with_capacity(n);
    let (mut amp_sum, mut amp_min, mut mean_sum) = (0.0, f64::INFINITY, 0.0);
    for i in 0..n {
        let mut rng = Rng::new(seed.wrapping_add(i as u64));
        let phi0 = rng.range(0.0, 2.0 * core::f64::consts::PI);
        let s_eci = V3::new(th.sin() * phi0.cos(), th.cos(), th.sin() * phi0.sin());
        // Nadir in a random direction (the albedo then reaches whichever faces see it)
        let u = rng.unit_vector();
        let mut sens = PdSensors::new(pd, seed.wrapping_add(1000 + i as u64));
        let mut g = Gyro::new(gyro, &mut rng);
        let (mut ts, mut zs, mut ps, mut wy) = (Vec::new(), Vec::new(), Vec::new(), 0.0);
        for k in 0..samples {
            let t = k as f64;
            let q = Quat::from_axis_angle(V3::Y, w * t);
            let s_b = q.rotate_inv(s_eci);
            let n_b = q.rotate_inv(u);
            let inp = ArrayInputs {
                sun_body: s_b,
                illum: 1.0,
                flux,
                nadir_body: n_b,
                albedo_w_m2,
                r_m,
                temp_c: [25.0; N_STRINGS],
                years_in_orbit: 0.1,
                wings_deployed: wings,
            };
            let p_bus = 0.95 * array.total_pmp(&inp);
            let pa = p_bus + rng.normal() * (0.02 + 0.01 * p_bus);
            let c = sens.channels(s_b, 1.0, n_b, albedo_w_m2 / flux, r_m, wings);
            wy += g.read(&mut rng, V3::new(0.0, w, 0.0), 1.0).y;
            ts.push(t);
            zs.push(c[4] - c[5]);
            ps.push(pa);
        }
        let w_g = wy / samples as f64;
        let (_, za, zb) = fit_harmonic(&ts, &zs, w_g);
        let (pc, pa_, pb) = fit_harmonic(&ts, &ps, w_g);
        let lag = wrap_deg(pb.atan2(pa_).to_degrees() - zb.atan2(za).to_degrees());
        let amp = pa_.hypot(pb);
        lags.push(lag);
        amp_sum += amp;
        amp_min = amp_min.min(amp);
        mean_sum += pc;
    }
    let nf = n.max(1) as f64;
    // Circular mean of the lag
    let (cs, sn) = lags.iter().fold((0.0, 0.0), |(c, s), l| (c + l.to_radians().cos(), s + l.to_radians().sin()));
    let mean = sn.atan2(cs).to_degrees();
    let dev: Vec<f64> = lags.iter().map(|l| wrap_deg(l - mean)).collect();
    let sd = (dev.iter().map(|d| d * d).sum::<f64>() / nf).sqrt();
    let worst = dev.iter().fold(0.0f64, |m, d| m.max(d.abs()));
    let sig = signature_deg.unwrap_or(mean);
    WingIdStats {
        n,
        lag_deg: (mean, sd, worst),
        amp_w: (amp_sum / nf, amp_min),
        mean_w: mean_sum / nf,
        correct: lags.iter().filter(|l| wrap_deg(*l - sig).abs() <= 45.0).count(),
    }
}

/// Run `n` trials of `p` on `threads` threads (seeds `seed0 ..`)
pub fn run(p: &Params, n: usize, seed0: u64, threads: usize) -> Vec<Trial> {
    let threads = threads.max(1);
    let mut out = vec![Trial::default(); n];
    std::thread::scope(|s| {
        for (c, chunk) in out.chunks_mut(n.div_ceil(threads).max(1)).enumerate() {
            let p = *p;
            s.spawn(move || {
                let base = c * n.div_ceil(threads).max(1);
                for (i, slot) in chunk.iter_mut().enumerate() {
                    *slot = trial(&p, seed0 + (base + i) as u64);
                }
            });
        }
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_trial_detumbles_and_reports() {
        let mut p = default_params();
        p.duration_s = 6.0 * 3600.0;
        p.steady_from_s = 5.0 * 3600.0;
        p.sensors.gyro = Some(GyroModel {
            bias_sigma: 0.02_f64.to_radians(),
            bias_instability: 0.002_f64.to_radians(),
            bias_tau_s: 300.0,
            noise_density: 0.005_f64.to_radians(),
            scale_sigma: 0.002,
            lsb: 0.0,
        });
        let t = trial(&p, 3);
        assert!(t.tipoff_dps > 0.0);
        assert!(t.detumbled_h.is_some(), "{t:?}");
        assert!(t.gen_w > 0.5, "{t:?}");
        assert_eq!(t.mag_rejected, 0, "the flight window gives clean samples");
    }

    #[test]
    fn a_missing_window_is_detected() {
        let mut p = default_params();
        p.duration_s = 3600.0;
        p.steady_from_s = 1800.0;
        // Coils on through the sample: the flight software sees it
        p.timing = CoilTiming { on_ms: 1000, settle_ms: adcs::MAG_SETTLE_MS };
        let t = trial(&p, 3);
        assert!(t.mag_rejected > 3000, "{t:?}");
        assert!(t.detumbled_h.is_none(), "no control without clean samples: {t:?}");
    }

    #[test]
    fn the_full_tensor_body_keeps_momentum_and_spins_about_its_major_axis() {
        let t = Inertia::DEPLOYED;
        let a = major_axis(&t);
        assert!(a.y > 0.95 && (a.x * a.x + a.y * a.y + a.z * a.z - 1.0).abs() < 1e-9, "{a:?}");
        // I a = λ a
        let b = Body::new(Quat::default(), V3::ZERO, &t);
        let ia = b.momentum(V3::new(a.x, a.y, a.z));
        let lambda = ia.dot(V3::new(a.x, a.y, a.z));
        assert!((ia - V3::new(a.x, a.y, a.z) * lambda).norm() < 1e-12);
        // Torque-free spin about the major axis stays there; about +Y it nutates
        let w0 = V3::new(a.x, a.y, a.z) * 0.105;
        let mut s = Body::new(Quat::default(), w0, &t);
        let h0 = s.q.rotate(s.momentum(s.w));
        for _ in 0..3600 {
            s.step(1.0, V3::ZERO);
        }
        assert!((s.q.rotate(s.momentum(s.w)) - h0).norm() < 1e-9 * h0.norm().max(1e-6) * 1e3);
        assert!((s.w - w0).norm() < 1e-6, "{:?}", s.w);
        let mut y = Body::new(Quat::default(), V3::new(0.0, 0.105, 0.0), &t);
        let mut worst = 0.0f64;
        for _ in 0..3600 {
            y.step(1.0, V3::ZERO);
            worst = worst.max(y.w.angle(V3::Y).to_degrees());
        }
        // It cones about the major axis, 0.7° from +Y with the CAD tensor
        // (MASS_BUDGET v3.1): the rate leaves +Y by about twice that
        let tilt = V3::new(a.x, a.y, a.z).angle(V3::Y).to_degrees();
        assert!(tilt > 0.3 && worst > 1.5 * tilt, "a spin about +Y with products wobbles: {worst} (tilt {tilt})");
        // The wings swing out: momentum kept
        let mut d = Body::new(Quat::default(), V3::new(0.01, 0.02, 0.03), &STOWED);
        let h = d.momentum(d.w);
        d.set_tensor(&t);
        assert!((d.momentum(d.w) - h).norm() < 1e-15);
        // The diagonal case reduces to the axis
        let diag = major_axis(&Inertia::diagonal(Vec3::new(1.0, 3.0, 2.0)));
        assert!((diag.y - 1.0).abs() < 1e-12);
        // Gravity gradient: none along a principal axis toward the Earth
        let g = Body::new(Quat::default(), V3::ZERO, &Inertia::diagonal(Vec3::new(1.0, 2.0, 3.0)));
        assert!(g.gravity_gradient(V3::new(7.0e6, 0.0, 0.0)).norm() < 1e-20);
    }

    #[test]
    fn the_gating_follows_the_duty_and_the_ants_tensor_is_the_elements_less() {
        // About 1.5 h a day of Jetson and 53 min keyed (sunlit all day here)
        let mut g = GateState::new(Gating::SIL, 5);
        let (mut jet, mut key) = (0.0, 0.0);
        for k in 0..(10 * 86_400) {
            let (j, kk) = g.step(f64::from(k), true);
            jet += f64::from(u8::from(j));
            key += f64::from(u8::from(kk));
        }
        let (jet_min, key_min) = (jet / 10.0 / 60.0, key / 10.0 / 60.0);
        assert!((70.0..110.0).contains(&jet_min), "{jet_min}");
        assert!((40.0..70.0).contains(&key_min), "{key_min}");
        // A short trial with the gating: the flight software counts the gated samples
        let mut p = default_params();
        p.duration_s = 6.0 * 3600.0;
        p.steady_from_s = 5.0 * 3600.0;
        p.sensors.gyro = gyro_preset("mg370").flatten();
        p.gating = Some(Gating { from_s: 0.0, ..Gating::SIL });
        let t = trial(&p, 3);
        assert!(t.gated_min_day > 0.0 && t.detumbled_h.is_some(), "{t:?}");
        // The AntS share: Iyy 14 % lower with the elements stowed; none out is the stowed tensor
        let a = ants_stowed(&Inertia::DEPLOYED);
        assert!((Inertia::DEPLOYED.yy / a.yy - 1.14).abs() < 0.01, "{a:?}");
        let s = ants_stowed(&Inertia::NO_WINGS);
        assert!((s.yy - STOWED.yy).abs() < 1e-12 && (s.yz - STOWED.yz).abs() < 1e-12);
    }

    #[test]
    fn the_photodiodes_give_the_sun_vector_and_the_wings_cover_them() {
        let ideal = Photodiodes { noise: 0.0, gain_sigma: 0.0, grazing: 0.0, albedo: false, ..Photodiodes::FLIGHT };
        let mut pd = PdSensors::new(ideal, 1);
        // 6° toward +X: the +X photodiode sees the Sun through the hinge gap
        let s = V3::new(0.1, 0.95, -0.2).unit();
        let both = pd.read(s, 1.0, -V3::Z, 0.0, 6.878e6, [true, true]);
        assert!(both.angle(s) < 1e-9, "{both:?}");
        // At 40° from +Y toward +X the deployed W1 shadows it
        let s40 = V3::new(40f64.to_radians().sin(), 40f64.to_radians().cos(), 0.0);
        assert!(pd.read(s40, 1.0, -V3::Z, 0.0, 6.878e6, [true, true]).x.abs() < 1e-12);
        // A stowed wing covers its face
        assert!(pd.read(s, 1.0, -V3::Z, 0.0, 6.878e6, [false, true]).x.abs() < 1e-12);
        // Albedo on the nadir face biases the vector toward the Earth
        let mut pa = PdSensors::new(Photodiodes { albedo: true, ..ideal }, 1);
        let y = pa.read(V3::Y, 1.0, V3::X, 0.2, 6.878e6, [true, true]);
        assert!(y.x > 0.15, "{y:?}");
    }

    #[test]
    fn one_wing_signatures_are_opposite() {
        let g = gyro_preset("mg370").flatten().expect("mg370");
        let w1 = wing_id(0, 45.0, 8.0, 3.0, 20, 1, Photodiodes::FLIGHT, g, 0.0, None);
        let w2 = wing_id(1, 45.0, 8.0, 3.0, 20, 1, Photodiodes::FLIGHT, g, 0.0, None);
        assert!((w1.lag_deg.0 - 90.0).abs() < 10.0 && (w2.lag_deg.0 + 90.0).abs() < 10.0, "{w1:?} {w2:?}");
        assert!(w1.amp_w.1 > 0.3 && w2.amp_w.1 > 0.3);
        let both = wing_id(2, 45.0, 8.0, 3.0, 20, 1, Photodiodes::FLIGHT, g, 0.0, None);
        assert!(both.amp_w.0 < 0.05, "{both:?}");
    }

    #[test]
    fn the_coils_field_reaches_the_magnetometer() {
        let mut rng = Rng::new(7);
        let s = Sensors::new(default_params().sensors, &mut rng).with_coupling(&mut rng, 1e-4);
        let m = V3::new(0.2, 0.0, 0.0);
        let b = s.coil_field(m);
        assert!(b.norm() > 1e-6 && b.norm() < 2e-5, "{b:?}");
        assert_eq!(s.coil_field(V3::ZERO).norm(), 0.0);
    }
}
