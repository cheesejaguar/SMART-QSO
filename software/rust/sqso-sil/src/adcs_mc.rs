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

use sqso_fsw::adcs::{self, Adcs, Gains, Vec3};

use crate::env::atmosphere;
use crate::env::attitude::{self, Attitude, Disturbance};
use crate::env::earth;
use crate::env::eclipse;
use crate::env::magfield;
use crate::env::math::{OMEGA_EARTH, Quat, Rng, V3};
use crate::env::orbit::Orbit;
use crate::env::solar::{Array, ArrayInputs, N_STRINGS};
use crate::env::sun;
use crate::env::time;

/// Physics sub-step, s (the control period is 1 s)
const SUBSTEP_S: f64 = 0.25;
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
}

impl Sensors {
    /// Sensors of `model` with their turn-on errors drawn from `rng`
    pub fn new(model: SensorModel, rng: &mut Rng) -> Sensors {
        let mag_bias = V3::new(rng.normal(), rng.normal(), rng.normal()) * model.mag_bias_t;
        let gyro = model.gyro.map(|g| Gyro::new(g, rng));
        Sensors { model, mag_bias, gyro }
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
    let mut att = Attitude::new(rng.quat(), w0, attitude::inertia_stowed());
    let dist = Disturbance { residual_dipole: rng.unit_vector() * p.residual_dipole, ..Disturbance::nominal() };
    let mut sensors = Sensors::new(p.sensors, &mut rng);
    let array = Array::new();

    let mut a = Adcs::new();
    a.init();
    a.gains = p.gains;
    let _ = a.set_mode(adcs::mode::DETUMBLE, 1);

    let mut out = Trial { tipoff_dps: w0.norm().to_degrees(), ..Trial::default() };
    let mut below_since: Option<f64> = None;
    let mut wings = false;
    let mut angles: Vec<f64> = Vec::new();
    let (mut gen_j, mut steady_s, mut spin_sum, mut trans_sq, mut mtq_j) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut was_dark = false;
    let mut anti = 0usize;

    let steps = (p.duration_s as u64).max(1);
    for k in 0..steps {
        let t_rel = k as f64;
        let t = epoch + t_rel;
        if !wings && t_rel >= WINGS_S {
            // The wings swing out: the body's angular momentum is kept
            let h = att.w.hadamard(att.inertia);
            let i = attitude::inertia_deployed();
            att.inertia = i;
            att.w = V3::new(h.x / i.x, h.y / i.y, h.z / i.z);
            wings = true;
        }
        let (r, v) = orbit.state_at(t);
        let (sun_u, sun_d) = sun::sun_eci(t);
        let illum = eclipse::illumination(r, sun_u, sun_d);
        let b_eci = magfield::field_eci(r, t);
        let density = atmosphere::density_nominal(Orbit::altitude_m(r), t);
        let v_rel = v - V3::new(0.0, 0.0, OMEGA_EARTH).cross(r);
        let wd = [wings, wings];

        // Sensors, then the flight law (1 Hz)
        let b_b = att.to_body(b_eci);
        let mag = sensors.magnetometer(&mut rng, b_b);
        let sun_b = att.to_body(sun_u);
        let sun_meas = sensors.sun_sensors(&mut rng, sun_b, illum);
        let gyro_reading = sensors.gyro(&mut rng, att.w, 1.0).map(v3);
        let ms = (t_rel * 1000.0) as u64 + 1000;
        let cycle = a.update_with_gyro(ms, ms, v3(mag * 1e6), v3(sun_meas), gyro_reading);
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

        // Dynamics over the control period
        let n_sub = (1.0 / SUBSTEP_S).round() as u32;
        for _ in 0..n_sub {
            let bb = att.to_body(b_eci);
            let v_body = att.to_body(v_rel);
            let torque = attitude::magnetic_torque(dipole, bb)
                + dist.residual_torque(bb)
                + attitude::gravity_gradient(r, att.q, att.inertia)
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
        if captured(att.w, p.gains.spin_rate) {
            let since = *below_since.get_or_insert(t_rel);
            if out.detumbled_h.is_none() && t_rel - since >= 60.0 {
                out.detumbled_h = Some(since / 3600.0);
            }
        } else {
            below_since = None;
        }
        if t_rel >= p.steady_from_s {
            steady_s += 1.0;
            let sb = att.to_body(sun_u);
            let angle = sb.angle(V3::new(0.0, 1.0, 0.0)).to_degrees();
            if illum > 0.5 {
                angles.push(angle);
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
            spin_sum += att.w.y;
            trans_sq += att.w.x * att.w.x + att.w.z * att.w.z;
            mtq_j += MTQ_FULL_W * (dipole.norm() / (adcs::MAX_DIPOLE * 3f64.sqrt())).min(1.0);
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
    }
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
    }
}
