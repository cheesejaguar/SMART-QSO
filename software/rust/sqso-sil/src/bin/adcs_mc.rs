//! `sqso_adcs_mc`: Monte Carlo of the attitude control (`sqso_sil::adcs_mc`).
//!
//! ```text
//! sqso_adcs_mc [--trials N] [--threads N] [--days D] [--seed S]
//!              [--spin LIST] [--momentum LIST] [--nutation LIST] [--bdot LIST] [--spin-hold LIST]
//!              [--tipoff-sigma DEG] [--tipoff-max DEG] [--dipole AM2]
//!              [--gyro none|bmi088|bmi088_uncal|mg370|adis16505] [--gyro-bias DEG_S]
//!              [--mag-noise NT] [--css DEG]
//! ```
//!
//! LIST is comma-separated; every combination is run. One CSV row per
//! combination goes to stdout (summary statistics over the trials).

use sqso_fsw::adcs::Gains;
use sqso_sil::adcs_mc;

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
    let mut bdots = vec![adcs_mc_default().bdot];
    let mut holds = vec![adcs_mc_default().spin_hold];
    let mut gyro_bias: Option<f64> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
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
            "--bdot" => bdots = list(&v),
            "--spin-hold" => holds = list(&v),
            "--tipoff-sigma" => p.tipoff_sigma = v.parse::<f64>().unwrap_or(5.0).to_radians(),
            "--tipoff-max" => p.tipoff_max = v.parse::<f64>().unwrap_or(15.0).to_radians(),
            "--dipole" => p.residual_dipole = v.parse().unwrap_or(p.residual_dipole),
            "--mag-noise" => p.sensors.mag_noise_t = v.parse::<f64>().unwrap_or(50.0) * 1e-9,
            "--css" => p.sensors.css_sigma_rad = v.parse::<f64>().unwrap_or(5.0).to_radians(),
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
    println!(
        "spin_dps,momentum,nutation,bdot,spin_hold,trials,not_detumbled,det_med_h,det_p95_h,det_max_h,fsw_med_h,fsw_p95_h,fsw_max_h,\
         with_timeout,sun_mean_deg,sun_p95_worst_deg,within15,eclipse_exit_worst_deg,anti,gen_mean_w,gen_worst_w,\
         spin_mean_dps,transverse_dps,mtq_w"
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
                            spin_hold: hold,
                        };
                        let s = adcs_mc::summarize(&adcs_mc::run(&p, trials, seed, threads));
                        println!(
                            "{spin},{mom},{nut},{bd},{hold},{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{},{:.1},{:.1},{:.3},{:.1},{},{:.3},{:.3},{:.2},{:.3},{:.3}",
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
                            s.mtq_w
                        );
                    }
                }
            }
        }
    }
}

fn adcs_mc_default() -> Gains {
    Gains::FLIGHT
}
