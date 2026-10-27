//! `sqso_adcs_mc`: Monte Carlo of the attitude control (`sqso_sil::adcs_mc`).
//!
//! ```text
//! sqso_adcs_mc [--trials N] [--threads N] [--days D] [--seed S]
//!              [--spin LIST] [--momentum LIST] [--nutation LIST] [--bdot LIST] [--spin-hold LIST]
//!              [--nutation-no-gyro G]
//!              [--tipoff-sigma DEG] [--tipoff-max DEG] [--dipole AM2]
//!              [--gyro none|bmi088|bmi088_uncal|mg370|adis16505] [--gyro-bias DEG_S]
//!              [--mag-noise NT] [--css DEG]
//!              [--mtq-on MS] [--settle MS] [--coupling T_PER_AM2]
//!              [--plant XX,YY,ZZ,XY,XZ,YZ] [--ratio R] [--wings both|w1|w2|none]
//!              [--geometry N] [--fsw-axis plant]
//!              [--ants-h H] [--gating JETSON_H,KEYED_MIN] [--css-model generic|pd|pd_cutoff|pd_noalbedo|pd_noshadow|pd_cosine|pd_ideal]
//!              [--wing-id]
//! ``````
//!
//! `--plant` is the deployed plant's inertia tensor, 10⁻³ kg·m² (default the
//! flight value `Inertia::DEPLOYED`, products included); `--ratio` then sets
//! Izz = Iyy / R. `--wings w1` deploys only wing W1 (and uses its tensor
//! unless `--plant` is given; `none`: neither wing). `--geometry N` gives the flight software
//! preset N of ADCS_SET_GEOMETRY; `--fsw-axis plant` gives it the plant's own
//! tensor and its major axis.
//!
//! `--mtq-on` and `--settle` set the coil burst and the settling time the
//! flight software waits before it uses a magnetometer sample (flight: 900
//! and 50 ms; `--mtq-on 1000 --settle 0` is the coils left on through the
//! sample, with the flight software unaware of it). `--coupling` is the
//! coil-to-magnetometer coupling (default 1e-4 T per A·m²; 0 for none).
//!
//! `--ants-h` deploys the AntS H hours after separation (default: with the
//! wings at 30 min; the plant has the elements stowed until then).
//! `--gating` gates the magnetometer and the coils (SRS-F143) for JETSON_H
//! hours a day of Jetson and KEYED_MIN minutes a day of exciter keying, in
//! blocks (`adcs_mc::Gating::SIL`: 1.5,53). `--css-model pd` replaces the generic
//! sun-vector error by the face photodiodes (`adcs_mc::Photodiodes::FLIGHT`;
//! `pd_cutoff`: no response beyond 85° incidence; `pd_noalbedo`: no albedo; `pd_noshadow`: no wing shadow; `pd_cosine`: no
//! grazing loss; `pd_ideal`: none of these and equal gains).
//! `--wing-id` prints the one-wing identification signatures (C-21) instead
//! of running trials.
//!
//! LIST is comma-separated; every combination is run. One CSV row per
//! combination goes to stdout (summary statistics over the trials).

use sqso_fsw::adcs::{Gains, Geometry, Inertia, Vec3};
use sqso_sil::adcs_mc;
use sqso_sil::adcs_mc::{Gating, Photodiodes};

fn list(s: &str) -> Vec<f64> {
    s.split(',').filter_map(|x| x.trim().parse().ok()).collect()
}

fn main() {
    let mut p = adcs_mc::default_params();
    p.sensors.gyro = adcs_mc::gyro_preset("mg370").flatten();
    let (mut trials, mut threads, mut seed) = (40usize, 4usize, 1u64);
    let mut spins = vec![adcs_mc_default().spin_rate.to_degrees()];
    let mut moms = vec![adcs_mc_default().momentum];
    let mut nuts = vec![adcs_mc_default().nutation];
    let mut nut_ng = adcs_mc_default().nutation_no_gyro;
    let mut bdots = vec![adcs_mc_default().bdot];
    let mut holds = vec![adcs_mc_default().spin_hold];
    let mut gyro_bias: Option<f64> = None;
    let mut plant: Option<Inertia> = None;
    let mut ratio: Option<f64> = None;
    let mut geometry: Option<u8> = None;
    let mut fsw_plant = false;
    let mut wing_id = false;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--wing-id" {
            wing_id = true;
            continue;
        }
        let v = args.next().unwrap_or_default();
        match a.as_str() {
            "--trials" => trials = v.parse().unwrap_or(trials),
            "--threads" => threads = v.parse().unwrap_or(threads),
            "--seed" => seed = v.parse().unwrap_or(seed),
            "--days" => {
                let d: f64 = v.parse().unwrap_or(2.0);
                p.duration_s = d * 86_400.0;
                p.steady_from_s = (d - 1.25).max(0.25) * 86_400.0;
            }
            "--spin" => spins = list(&v),
            "--momentum" => moms = list(&v),
            "--nutation" => nuts = list(&v),
            "--nutation-no-gyro" => nut_ng = v.parse().unwrap_or(nut_ng),
            "--bdot" => bdots = list(&v),
            "--spin-hold" => holds = list(&v),
            "--tipoff-sigma" => p.tipoff_sigma = v.parse::<f64>().unwrap_or(5.0).to_radians(),
            "--tipoff-max" => p.tipoff_max = v.parse::<f64>().unwrap_or(15.0).to_radians(),
            "--dipole" => p.residual_dipole = v.parse().unwrap_or(p.residual_dipole),
            "--mag-noise" => p.sensors.mag_noise_t = v.parse::<f64>().unwrap_or(50.0) * 1e-9,
            "--css" => p.sensors.css_sigma_rad = v.parse::<f64>().unwrap_or(5.0).to_radians(),
            "--mtq-on" => p.timing.on_ms = v.parse().unwrap_or(p.timing.on_ms),
            "--settle" => p.timing.settle_ms = v.parse().unwrap_or(p.timing.settle_ms),
            "--coupling" => p.mag_coupling = v.parse().unwrap_or(p.mag_coupling),
            "--plant" => {
                let x = list(&v);
                if x.len() != 6 {
                    eprintln!("--plant needs six values");
                    std::process::exit(2);
                }
                let k = 1e-3;
                plant = Some(Inertia {
                    xx: x[0] * k,
                    yy: x[1] * k,
                    zz: x[2] * k,
                    xy: x[3] * k,
                    xz: x[4] * k,
                    yz: x[5] * k,
                });
            }
            "--ratio" => ratio = v.parse().ok(),
            "--geometry" => geometry = v.parse().ok(),
            "--fsw-axis" => fsw_plant = v == "plant",
            "--wings" => {
                p.wings = match v.as_str() {
                    "w1" => [true, false],
                    "w2" => [false, true],
                    "none" => [false, false],
                    _ => [true, true],
                };
                p.plant_deployed = match v.as_str() {
                    "w1" => Inertia::WING_1_ONLY,
                    "w2" => Inertia::WING_2_ONLY,
                    "none" => Inertia::NO_WINGS,
                    _ => Inertia::DEPLOYED,
                };
            }
            "--ants-h" => p.ants_s = v.parse::<f64>().unwrap_or(0.5) * 3600.0,
            "--gating" => {
                let x = list(&v);
                if x.len() != 2 {
                    eprintln!("--gating needs JETSON_H,KEYED_MIN");
                    std::process::exit(2);
                }
                p.gating =
                    Some(Gating { jetson_s_per_day: x[0] * 3600.0, keyed_s_per_day: x[1] * 60.0, ..Gating::SIL });
            }
            "--css-model" => {
                p.photodiodes = match v.as_str() {
                    "generic" => None,
                    "pd" => Some(Photodiodes::FLIGHT),
                    "pd_cutoff" => Some(Photodiodes { cutoff_deg: 85.0, ..Photodiodes::FLIGHT }),
                    "pd_noalbedo" => Some(Photodiodes { albedo: false, ..Photodiodes::FLIGHT }),
                    "pd_noshadow" => Some(Photodiodes { wing_shadow: false, ..Photodiodes::FLIGHT }),
                    "pd_cosine" => Some(Photodiodes { grazing: 0.0, ..Photodiodes::FLIGHT }),
                    "pd_ideal" => Some(Photodiodes {
                        albedo: false,
                        wing_shadow: false,
                        grazing: 0.0,
                        gain_sigma: 0.0,
                        ..Photodiodes::FLIGHT
                    }),
                    _ => {
                        eprintln!("unknown sun-sensor model {v}");
                        std::process::exit(2);
                    }
                }
            }
            "--gyro-bias" => gyro_bias = v.parse::<f64>().ok(),
            "--gyro" => match adcs_mc::gyro_preset(&v) {
                Some(g) => p.sensors.gyro = g,
                None => {
                    eprintln!("unknown gyro {v}");
                    std::process::exit(2);
                }
            },
            _ => {
                eprintln!("unknown argument {a}");
                std::process::exit(2);
            }
        }
    }
    if let (Some(b), Some(g)) = (gyro_bias, p.sensors.gyro.as_mut()) {
        g.bias_sigma = b.to_radians();
    }
    if let Some(t) = plant {
        p.plant_deployed = t;
    }
    if let Some(r) = ratio {
        p.plant_deployed.zz = p.plant_deployed.yy / r;
    }
    if let Some(n) = geometry {
        p.fsw_geometry = Geometry::preset(n).unwrap_or(Geometry::FLIGHT);
    }
    if fsw_plant {
        p.fsw_geometry = Geometry { sun_axis: adcs_mc::major_axis(&p.plant_deployed), inertia: p.plant_deployed };
    }
    if wing_id {
        print_wing_id(p.sensors.gyro.unwrap_or_else(|| adcs_mc::gyro_preset("mg370").flatten().expect("mg370")));
        return;
    }
    let a: Vec3 = p.fsw_geometry.sun_axis;
    eprintln!(
        "plant deployed {:?}; flight software axis ({:.4}, {:.4}, {:.4}); plant major axis {:?}",
        p.plant_deployed,
        a.x,
        a.y,
        a.z,
        adcs_mc::major_axis(&p.plant_deployed)
    );
    println!(
        "spin_dps,momentum,nutation,bdot,spin_hold,trials,not_detumbled,det_med_h,det_p95_h,det_max_h,fsw_med_h,fsw_p95_h,fsw_max_h,\
         with_timeout,sun_mean_deg,sun_p95_worst_deg,within15,eclipse_exit_worst_deg,anti,gen_mean_w,gen_worst_w,\
         spin_mean_dps,transverse_dps,mtq_w,mag_rejected,know_mean_deg,know_p95_worst_deg,know_within5_worst,gated_min_day,\
         rate_tilt_x_mean_deg,rate_tilt_x_min_deg,rate_tilt_x_max_deg,sun_tilt_x_deg,ants_max_worst_deg,id_window_med_min,id_window_min_min,within30_mean,within30_worst,gen_ideal_w,pointing_loss_mean,pointing_loss_worst,p002_loss_failed,p002_within30_failed"
    );
    for &spin in &spins {
        for &mom in &moms {
            for &nut in &nuts {
                for &bd in &bdots {
                    for &hold in &holds {
                        p.gains = Gains {
                            bdot: bd,
                            spin_rate: spin.to_radians(),
                            momentum: mom,
                            nutation: nut,
                            nutation_no_gyro: nut_ng,
                            spin_hold: hold,
                        };
                        let s = adcs_mc::summarize(&adcs_mc::run(&p, trials, seed, threads));
                        println!(
                            "{spin},{mom},{nut},{bd},{hold},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{:.1},{:.1},{:.3},{:.1},{},{:.3},{:.3},{:.2},{:.3},{:.3},{:.0},{:.2},{:.2},{:.4},{:.1},{:.2},{:.2},{:.2},{:.2},{:.1},{:.1},{:.1},{:.4},{:.4},{:.3},{:.4},{:.4},{},{}",
                            s.n,
                            s.not_detumbled,
                            s.detumble_h.0,
                            s.detumble_h.1,
                            s.detumble_h.2,
                            s.fsw_detumble_h.0,
                            s.fsw_detumble_h.1,
                            s.fsw_detumble_h.2,
                            s.with_timeout,
                            s.sun_mean_deg,
                            s.sun_p95_worst_deg,
                            s.within_15,
                            s.eclipse_exit_worst_deg,
                            s.anti,
                            s.gen_w.0,
                            s.gen_w.1,
                            s.spin_dps,
                            s.transverse_dps,
                            s.mtq_w,
                            s.mag_rejected,
                            s.know_mean_deg,
                            s.know_p95_worst_deg,
                            s.know_within_5_worst,
                            s.gated_min_day,
                            s.rate_tilt_x_deg.0,
                            s.rate_tilt_x_deg.1,
                            s.rate_tilt_x_deg.2,
                            s.sun_tilt_x_deg,
                            s.ants_max_worst_deg,
                            s.id_window_min.0,
                            s.id_window_min.1,
                            s.within_30.0,
                            s.within_30.1,
                            s.gen_ideal_w,
                            s.pointing_loss.0,
                            s.pointing_loss.1,
                            s.p002_failed.0,
                            s.p002_failed.1
                        );
                    }
                }
            }
        }
    }
}

/// The one-wing identification signatures (C-21): the lag of the array
/// power's first harmonic behind the +Z photodiode's over three spins at
/// 8°/s, for each wing, Sun angle and albedo
fn print_wing_id(gyro: adcs_mc::GyroModel) {
    println!(
        "wing,theta_deg,albedo_w_m2,draws,lag_mean_deg,lag_sd_deg,lag_worst_dev_deg,amp_mean_w,amp_min_w,mean_w,correct"
    );
    for albedo in [0.0, 100.0, 300.0] {
        for theta in [10.0, 15.0, 20.0, 30.0, 45.0, 55.0, 60.0, 70.0] {
            // The signatures are the noise-free lags at 45° (both wings)
            let sig = |w: usize| {
                let quiet = Photodiodes { noise: 0.0, gain_sigma: 0.0, ..Photodiodes::FLIGHT };
                let g0 = adcs_mc::GyroModel { bias_sigma: 0.0, bias_instability: 0.0, noise_density: 0.0, ..gyro };
                adcs_mc::wing_id(w, 45.0, 8.0, 3.0, 1, 7, quiet, g0, 0.0, None).lag_deg.0
            };
            for wing in 0..4 {
                let s = adcs_mc::wing_id(
                    wing,
                    theta,
                    8.0,
                    3.0,
                    500,
                    1,
                    Photodiodes::FLIGHT,
                    gyro,
                    albedo,
                    Some(sig(wing.min(1))),
                );
                println!(
                    "{},{theta},{albedo},{},{:.1},{:.1},{:.1},{:.3},{:.3},{:.3},{}",
                    ["W1", "W2", "both", "none"][wing],
                    s.n,
                    s.lag_deg.0,
                    s.lag_deg.1,
                    s.lag_deg.2,
                    s.amp_w.0,
                    s.amp_w.1,
                    s.mean_w,
                    s.correct
                );
            }
        }
    }
}

fn adcs_mc_default() -> Gains {
    Gains::FLIGHT
}
