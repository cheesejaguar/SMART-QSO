//! The thermal cases of THERMAL_ANALYSIS 2.2 (CDR round 3, the thermal
//! analyst's harness folded in, RIDs P3S-01, THM-03, THM-04): replicates the
//! `sqso_sil::env::thermal` tests `run_case` / `dispenser_soak` on the public
//! API, and adds (1) the wings stowed until the burn, (2) one-way-coupled
//! sub-nodes for each wing (hinge frame side, hinge wing side, burn board
//! with the resistor, the line's free span), (3) a stack heater and the
//! board's detumble power, (4) separation-phase sweeps, and case L4 (β = 0,
//! separation at eclipse entry, wings stowed until the burn, 2 to 3 h
//! dispenser cold soak). The heater is 207 Ω: 0.70 W at 12.0 V.
//!
//! Run (about a minute in release):
//!
//!   cargo test -p sqso-sil --release --test thermal_round3 -- --ignored --nocapture
//!
//! `THM_CASES` selects a part: all (default), valid, c10, h070, thm03, p3s,
//! thm04. Rows print as `ROW|...`, `LEOP|...`, `SUBMIN|...` and so on.

// The analyst's harness as written: plain arrays by node index, some
// fields kept for printing variants
#![allow(dead_code, clippy::type_complexity, clippy::needless_range_loop)]

use sqso_sil::env::earth::{ALBEDO, EARTH_IR_W_M2, plate_view_factor};
use sqso_sil::env::eclipse::illumination;
use sqso_sil::env::math::{Quat, SIGMA, T0_K, V3, interp};
use sqso_sil::env::orbit::Orbit;
use sqso_sil::env::solar::{Array, ArrayInputs};
use sqso_sil::env::sun::{solar_flux, sun_eci};
use sqso_sil::env::thermal::*;
use sqso_sil::env::time::from_utc;

const ALT: f64 = 475e3;

#[derive(Clone, Copy, PartialEq)]
enum Att {
    SpinPointed,
    Tumble,
    TipOff,
}
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Safe,
    Emergency,
    Leop,
}

fn quat_from_axes(xb: V3, yb: V3, zb: V3) -> Quat {
    let (m00, m11, m22) = (xb.x, yb.y, zb.z);
    let tr = m00 + m11 + m22;
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        Quat { w: 0.25 * s, x: (yb.z - zb.y) / s, y: (zb.x - xb.z) / s, z: (xb.y - yb.x) / s }
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        Quat { w: (yb.z - zb.y) / s, x: 0.25 * s, y: (yb.x + xb.y) / s, z: (zb.x + xb.z) / s }
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        Quat { w: (zb.x - xb.z) / s, x: (yb.x + xb.y) / s, y: 0.25 * s, z: (zb.y + yb.z) / s }
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        Quat { w: (xb.y - yb.x) / s, x: (zb.x + xb.z) / s, y: (zb.y + yb.z) / s, z: 0.25 * s }
    };
    q.normalize()
}

fn attitude(att: Att, o: &Orbit, s: V3, t: f64) -> Quat {
    match att {
        Att::SpinPointed => {
            let yb = s;
            let zb = o.normal().cross(s).cross(s).unit() * -1.0;
            let xb = yb.cross(zb);
            quat_from_axes(xb, yb, zb).mul(Quat::from_axis_angle(V3::Y, sqso_fsw::adcs::SPIN_RATE * t))
        }
        Att::Tumble => Quat::from_axis_angle(V3::new(0.3, 0.8, 0.5).unit(), 2.0_f64.to_radians() * t)
            .mul(Quat::from_axis_angle(V3::new(-0.6, 0.2, 0.77).unit(), 3.3_f64.to_radians() * t)),
        Att::TipOff => Quat::from_axis_angle(V3::new(0.5, -0.4, 0.77).unit(), 10.0_f64.to_radians() * t),
    }
}

fn mode_loads(mode: Mode) -> ([f64; N_NODES], f64) {
    let mut q = [0.0; N_NODES];
    let beacon = |period: f64| (0.31 + 200.0 * 8.0 * 1.02 / 1200.0) / period;
    match mode {
        Mode::Safe | Mode::Emergency => {
            let b = if mode == Mode::Safe { beacon(180.0) } else { 0.0 };
            q[OBC] = 0.165 * (1.0 / 0.85) + 0.08;
            q[RF] = 0.15 + b * 1.0;
            (q, b * 0.5)
        }
        Mode::Leop => {
            q[OBC] = (0.165 + 0.12 + 0.30) / 0.85 + 0.08;
            q[RF] = 0.15;
            (q, 0.0)
        }
    }
}

#[derive(Clone)]
struct Case {
    name: String,
    t0: f64,
    ltan: f64,
    flux: Option<f64>,
    albedo: f64,
    ir: f64,
    att: Att,
    mode: Mode,
    heater_w: f64,
    setpoints: (f64, f64),
    params: ThermalParams,
    wings: [bool; 2],
    /// time after separation at which the wings deploy (None: never change)
    deploy_at: Option<f64>,
    years: f64,
    orbits: f64,
    /// run length override, s
    run_s: Option<f64>,
    init: Option<[f64; N_NODES]>,
    init_c: f64,
    /// separation this long after the orbit epoch (ascending node), s
    phase_s: f64,
    /// stack heater (W, on below, off above) on the OBC node
    stack_heater: Option<(f64, f64, f64)>,
    /// extra stack dissipation (W) until this time (s): board detumble power
    extra_obc: (f64, f64),
    /// RF board off until its temperature reaches this (°C)
    rf_gate_c: Option<f64>,
    /// heater on the RF node (W, always on)
    rf_heater: f64,
    snaps: Vec<f64>,
    sub: SubParams,
}

#[derive(Clone, Copy)]
struct SubParams {
    g_hinge_frame: f64,
    g_hinge_wing: f64,
}

fn cold_case(name: &str, att: Att, mode: Mode, heater: f64) -> Case {
    Case {
        name: name.to_string(),
        t0: from_utc(2029, 3, 20, 0, 0, 0.0),
        ltan: 12.0,
        flux: Some(1322.0),
        albedo: 0.25,
        ir: 218.0,
        att,
        mode,
        heater_w: heater,
        setpoints: (sqso_fsw::heater::ON_BELOW_C, sqso_fsw::heater::OFF_ABOVE_C),
        params: ThermalParams::default(),
        wings: [true, true],
        deploy_at: None,
        years: 2.0,
        orbits: 25.0,
        run_s: None,
        init: None,
        init_c: 10.0,
        phase_s: 0.0,
        stack_heater: None,
        extra_obc: (0.0, 0.0),
        rf_gate_c: None,
        rf_heater: 0.0,
        snaps: vec![],
        sub: SubParams { g_hinge_frame: 0.20, g_hinge_wing: 0.01 },
    }
}

/// Sub-node state per wing: hinge frame side, hinge wing side (knuckle), burn board, free line span
#[derive(Clone, Copy, Debug)]
struct Sub {
    h: f64,
    k: f64,
    b: f64,
    l: f64,
}

const C_H: f64 = 4.0; // ears, pin, bushings, springs (≈ 5 g), J/K
const C_K: f64 = 2.0; // wing knuckles and flex cup, J/K
const C_B: f64 = 1.5; // burn board, resistor, bracket, J/K
const C_L: f64 = 0.34e-1; // 0.1 m of Ø0.5 mm line, J/K
const A_H: f64 = 3.0e-4; // exposed hinge area, m²
const EPS_H: f64 = 0.85;
const D_LINE: f64 = 0.5e-3;
const LEN_LINE: f64 = 0.1;
const EPS_L: f64 = 0.85;
const ALPHA_L: f64 = 0.5;

struct Out {
    tmin: [f64; N_NODES],
    tmax: [f64; N_NODES],
    tmean: [f64; N_NODES],
    allmin: [f64; N_NODES],
    duty: f64,
    harvest_avg: f64,
    snaps: Vec<(f64, [f64; N_NODES], [Sub; 2], bool, f64)>,
    final_k: [f64; N_NODES],
    stack_heater_wh: f64,
    sub_min: [Sub; 2],
    below: [f64; 3], // time (min) OBC < -20, RF < -20, Jetson < -25
    t_above0: f64,
}

#[allow(clippy::too_many_lines)]
fn run(c: &Case) -> Out {
    let o = Orbit::sso(ALT, c.ltan, c.t0);
    let array = Array::new();
    let mut th = Thermal::with_params(c.init_c, &c.params);
    if let Some(init) = c.init {
        th.temp_k = init;
    }
    let dt = 5.0;
    let p = o.period();
    let end = c.run_s.unwrap_or(c.orbits * p);
    let mut heater = false;
    let mut sh = false;
    let mut sh_j = 0.0;
    let (mut tmin, mut tmax, mut tsum) = ([f64::INFINITY; N_NODES], [f64::NEG_INFINITY; N_NODES], [0.0; N_NODES]);
    let mut allmin = [f64::INFINITY; N_NODES];
    let (mut n, mut hon, mut harv_sum) = (0.0, 0.0, 0.0);
    let t_init = th.temp_k[PX];
    let mut sub = [Sub { h: t_init, k: t_init, b: t_init, l: t_init }; 2];
    if c.init.is_some() {
        for (k, (w, f)) in [(W1, PX), (W2, MX)].into_iter().enumerate() {
            sub[k] = Sub { h: th.temp_k[f], k: th.temp_k[w], b: th.temp_k[f], l: th.temp_k[w] };
        }
    }
    let mut sub_min = [Sub { h: 1e9, k: 1e9, b: 1e9, l: 1e9 }; 2];
    let mut snaps = Vec::new();
    let mut si = 0;
    let mut below = [0.0; 3];
    let mut rf_on = c.rf_gate_c.is_none();
    let mut t_above0 = if th.battery_c() >= 0.0 { 0.0 } else { -1.0 };
    let mut t = 0.0;
    while t < end - 0.5 * dt {
        let tt = c.t0 + c.phase_s + t;
        let (r, _) = o.state_at(tt);
        let (s, d) = sun_eci(tt);
        let q = attitude(c.att, &o, s, t);
        let illum = illumination(r, s, d);
        let sun_b = q.rotate_inv(s);
        let nadir_b = q.rotate_inv(-r.unit());
        let flux = c.flux.unwrap_or_else(|| solar_flux(d));
        let wings = match c.deploy_at {
            Some(td) if t >= td => [true, true],
            Some(_) => [false, false],
            None => c.wings,
        };
        let mut inp = ThermalInputs::new(sun_b, illum, flux, nadir_b, r.norm(), r.unit().dot(s));
        inp.albedo = c.albedo;
        inp.earth_ir_w_m2 = c.ir;
        inp.wings_deployed = wings;
        let strings = array.strings(&ArrayInputs {
            sun_body: sun_b,
            illum,
            flux,
            nadir_body: nadir_b,
            albedo_w_m2: 0.0,
            r_m: r.norm(),
            temp_c: th.string_temps_c(),
            years_in_orbit: c.years,
            wings_deployed: wings,
        });
        let (mut qi, rf_out) = mode_loads(c.mode);
        if !rf_on {
            if th.rf_c() >= c.rf_gate_c.unwrap_or(-1e9) {
                rf_on = true;
            } else {
                qi[RF] = 0.0;
            }
        }
        if t < c.extra_obc.1 {
            qi[OBC] += c.extra_obc.0;
        }
        let tb = th.battery_c();
        if tb < c.setpoints.0 {
            heater = true;
        } else if tb > c.setpoints.1 {
            heater = false;
        }
        if heater {
            qi[BAT] += c.heater_w;
        }
        if let Some((w, on, off)) = c.stack_heater {
            let to = th.obc_c();
            if to < on {
                sh = true;
            } else if to > off {
                sh = false;
            }
            if sh {
                qi[OBC] += w;
                sh_j += w * dt;
            }
        }
        qi[RF] += c.rf_heater;
        sh_j += c.rf_heater * dt;
        inp.internal_w = qi;
        inp.elec_w = elec_from_strings(&strings);
        let harvested: f64 = inp.elec_w.iter().sum();
        let demand: f64 = qi.iter().sum::<f64>() + rf_out;
        if harvested > demand && harvested > 0.0 {
            let k = demand / harvested;
            inp.elec_w.iter_mut().for_each(|e| *e *= k);
        }
        th.step(dt, &inp);
        // sub-nodes (one-way coupled to the network)
        let tk = th.temp_k;
        let direct = flux * illum.clamp(0.0, 1.0);
        let alb = c.albedo * flux * r.unit().dot(s).max(0.0);
        for (k, (w, f, sx)) in [(W1, PX, 1.0), (W2, MX, -1.0)].into_iter().enumerate() {
            let st = &mut sub[k];
            // hinge frame side: ears on the shear panel, pin, springs below the wing; corner normal (±X+Y)/√2
            let nh = (V3::X * sx + V3::Y).unit();
            let fe = plate_view_factor(nh.dot(nadir_b), r.norm());
            let qh_env =
                EPS_H * A_H * 0.5 * c.ir * fe + 0.88 * A_H * 0.5 * (direct * nh.dot(sun_b).max(0.0) + alb * fe);
            let qh = qh_env
                - EPS_H * SIGMA * A_H * 0.5 * st.h.powi(4)
                - c.sub.g_hinge_frame * (st.h - tk[f])
                - c.sub.g_hinge_wing * (st.h - st.k);
            // wing side: knuckles bonded into the wing end zone (0.05 W/K), flex cup
            let qk = -0.05 * (st.k - tk[w]) + c.sub.g_hinge_wing * (st.h - st.k)
                - EPS_H * SIGMA * 1.0e-4 * 0.5 * st.k.powi(4)
                + EPS_H * 1.0e-4 * 0.5 * c.ir * fe;
            // burn board in the body corner strip (±X, −Y): screws to both shear panels, interior radiation,
            // 10 % view through the line slot to the stowed wing's back
            let a_b = 4.0e-4;
            let t_int = 0.5 * (tk[f] + tk[MY]);
            let qb = -0.015 * (st.b - tk[f])
                - 0.015 * (st.b - tk[MY])
                - 0.9 * SIGMA * a_b * 0.9 * (st.b.powi(4) - t_int.powi(4))
                - 0.9 * SIGMA * a_b * 0.1 * (st.b.powi(4) - tk[w].powi(4));
            // free line span over the post, on the stowed wing's outer face (normal ±X): half its view to the
            // wing, half to the outer hemisphere
            let nl = V3::X * sx;
            let fl = plate_view_factor(nl.dot(nadir_b), r.norm());
            let per = std::f64::consts::PI * D_LINE * LEN_LINE;
            let ql = EPS_L * per * 0.5 * SIGMA * tk[w].powi(4)
                + EPS_L * per * 0.5 * c.ir * fl
                + ALPHA_L * D_LINE * LEN_LINE * direct * nl.dot(sun_b).max(0.0)
                + ALPHA_L * per * 0.5 * alb * fl
                - EPS_L * per * SIGMA * st.l.powi(4);
            st.h += dt * qh / C_H;
            st.k += dt * qk / C_K;
            st.b += dt * qb / C_B;
            st.l += dt * ql / C_L;
            if !wings[k] {
                let m = &mut sub_min[k];
                m.h = m.h.min(st.h);
                m.k = m.k.min(st.k);
                m.b = m.b.min(st.b);
                m.l = m.l.min(st.l);
            }
        }
        t += dt;
        if t_above0 < 0.0 && th.battery_c() >= 0.0 {
            t_above0 = t / 60.0;
        }
        for k in 0..N_NODES {
            allmin[k] = allmin[k].min(th.temp_c(k));
        }
        if th.obc_c() < -20.0 {
            below[0] += dt / 60.0;
        }
        if th.rf_c() < -20.0 {
            below[1] += dt / 60.0;
        }
        if th.jetson_c() < -25.0 {
            below[2] += dt / 60.0;
        }
        while si < c.snaps.len() && t >= c.snaps[si] - 1e-6 {
            let mut v = [0.0; N_NODES];
            for k in 0..N_NODES {
                v[k] = th.temp_c(k);
            }
            snaps.push((c.snaps[si], v, sub, illum > 0.5, o.beta(s).to_degrees()));
            si += 1;
        }
        if t > end - p {
            for k in 0..N_NODES {
                let v = th.temp_c(k);
                tmin[k] = tmin[k].min(v);
                tmax[k] = tmax[k].max(v);
                tsum[k] += v;
            }
            n += 1.0;
            if heater {
                hon += 1.0;
            }
            harv_sum += harvested;
        }
    }
    let mut tmean = [0.0; N_NODES];
    for k in 0..N_NODES {
        tmean[k] = tsum[k] / n;
    }
    Out {
        tmin,
        tmax,
        tmean,
        allmin,
        duty: hon / n,
        harvest_avg: harv_sum / n,
        snaps,
        final_k: th.temp_k,
        stack_heater_wh: sh_j / 3600.0,
        sub_min,
        below,
        t_above0,
    }
}

fn dispenser_soak(start_c: f64, end_s: f64) -> [f64; N_NODES] {
    let tab_t = [0.0, 1000.0, 2000.0, 3000.0, 7200.0, 1.0e6];
    let tab_cold = [-5.0, -10.0, -15.0, -20.0, -20.0, -20.0];
    let mut th = Thermal::new(start_c);
    let dt = 5.0;
    let mut t = 0.0;
    while t < end_s + 0.5 * dt {
        let wall = interp(&tab_t, &tab_cold, t) + T0_K;
        let mut inp = ThermalInputs::new(V3::Y, 0.0, 0.0, -V3::Y, 1e12, 0.0);
        inp.wings_deployed = [false, false];
        inp.albedo = 0.0;
        inp.earth_ir_w_m2 = 0.0;
        inp.sink_k = wall;
        inp.sink_eps_factor = 0.70 / 0.85;
        inp.boundary_w_k = 0.5;
        inp.boundary_k = wall;
        th.step(dt, &inp);
        t += dt;
    }
    th.temp_k
}

fn c(k: f64) -> f64 {
    k - T0_K
}

fn row(name: &str, o: &Out) {
    let body_min = [PX, MX, PY, MY, PZ, MZ].iter().map(|&k| o.tmin[k]).fold(f64::INFINITY, f64::min);
    let body_max = [PX, MX, PY, MY, PZ, MZ].iter().map(|&k| o.tmax[k]).fold(f64::NEG_INFINITY, f64::max);
    println!(
        "ROW|{name}|bat {:.1} … {:.1}|OBC {:.1} … {:.1}|RF {:.1} … {:.1}|Jet {:.1} … {:.1}|body {:.1} … {:.1}|wing {:.1} … {:.1}|duty {:.0} %|harvest {:.2} W",
        o.tmin[BAT],
        o.tmax[BAT],
        o.tmin[OBC],
        o.tmax[OBC],
        o.tmin[RF],
        o.tmax[RF],
        o.tmin[JET],
        o.tmax[JET],
        body_min,
        body_max,
        o.tmin[W1].min(o.tmin[W2]),
        o.tmax[W1].max(o.tmax[W2]),
        o.duty * 100.0,
        o.harvest_avg
    );
}

fn snap_print(tag: &str, o: &Out) {
    for (ts, v, sub, lit, beta) in &o.snaps {
        println!(
            "SNAP|{tag}|T+{:.0} min|{}|beta {:.1}|W1 {:.1} W2 {:.1}|+X {:.1} -X {:.1}|hingeF {:.1}/{:.1}|hingeW {:.1}/{:.1}|burnbd {:.1}/{:.1}|line {:.1}/{:.1}|snub {:.1}/{:.1}|bat {:.1} OBC {:.1}",
            ts / 60.0,
            if *lit { "sun" } else { "eclipse" },
            beta,
            v[W1],
            v[W2],
            v[PX],
            v[MX],
            c(sub[0].h),
            c(sub[1].h),
            c(sub[0].k),
            c(sub[1].k),
            c(sub[0].b),
            c(sub[1].b),
            c(sub[0].l),
            c(sub[1].l),
            0.5 * (v[W1] + v[PX]),
            0.5 * (v[W2] + v[MX]),
            v[BAT],
            v[OBC]
        );
    }
}

/// THERMAL_ANALYSIS 2.2 §4.2, §4.3, §6.6, §6.8: every case
#[test]
#[ignore = "about a minute in release; prints the tables"]
fn thermal_round3_cases() {
    let what = std::env::var("THM_CASES").unwrap_or_else(|_| "all".to_string());
    run_all(&what);
}

/// The default heater at 12.0 V: 207 Ω, 0.70 W (THERMAL_ANALYSIS 2.2 §4.2)
#[test]
fn heater_is_0p70_w_at_12_v() {
    assert!((12.0_f64 * 12.0 / 207.0 - 0.70).abs() < 0.005);
}

fn run_all(what: &str) {
    // validation: reproduce C11 and C14 at 0.72 W (THERMAL_ANALYSIS §4.1: 3.7 … 9.8 and −1.1 … 3.2)
    if what == "all" || what == "valid" {
        for (nm, w, wings, att, mode, g) in [
            ("C11@0.72", 0.72, true, Att::SpinPointed, Mode::Safe, BATTERY_MOUNT_W_K),
            ("C14@0.72", 0.72, false, Att::SpinPointed, Mode::Safe, BATTERY_MOUNT_W_K),
        ] {
            let mut k = cold_case(nm, att, mode, w);
            k.wings = [wings, wings];
            k.params.battery_mount_w_k = g;
            row(nm, &run(&k));
        }
    }
    if what == "c10" {
        for w in [0.72, 12.0 * 12.0 / 207.0] {
            let mut k = cold_case("C10", Att::Tumble, Mode::Safe, w);
            k.t0 = from_utc(2029, 6, 21, 0, 0, 0.0);
            k.ltan = 23.5;
            k.flux = None;
            row(&format!("C10 at {w:.3} W"), &run(&k));
        }
    }
    // Task 5: the 0.72 W cases at 0.70 W (207 Ω at 12.0 V = 0.696 W)
    if what == "all" || what == "h070" {
        let hw = 12.0 * 12.0 / 207.0;
        let list: Vec<(&str, Att, Mode, bool, f64, f64)> = vec![
            ("C2 tumbling 0.70 W", Att::Tumble, Mode::Safe, true, BATTERY_MOUNT_W_K, 0.10),
            ("C5 EMERGENCY tumbling 0.70 W", Att::Tumble, Mode::Emergency, true, BATTERY_MOUNT_W_K, 0.10),
            ("C6 tumbling wings stowed 0.70 W", Att::Tumble, Mode::Safe, false, BATTERY_MOUNT_W_K, 0.10),
            ("C7 tumbling black battery 0.70 W", Att::Tumble, Mode::Safe, true, BATTERY_MOUNT_W_K, 0.85),
            ("C8 tumbling mount 0.10 0.70 W", Att::Tumble, Mode::Safe, true, 0.10, 0.10),
            ("C11 spin-pointed 0.70 W", Att::SpinPointed, Mode::Safe, true, BATTERY_MOUNT_W_K, 0.10),
            ("C13 EMERGENCY spin-pointed 0.70 W", Att::SpinPointed, Mode::Emergency, true, BATTERY_MOUNT_W_K, 0.10),
            ("C14 spin-pointed wings stowed 0.70 W", Att::SpinPointed, Mode::Safe, false, BATTERY_MOUNT_W_K, 0.10),
            ("C15 spin-pointed mount 0.10 0.70 W", Att::SpinPointed, Mode::Safe, true, 0.10, 0.10),
        ];
        for (nm, att, mode, wings, g, eps) in list {
            let mut k = cold_case(nm, att, mode, hw);
            k.wings = [wings, wings];
            k.params.battery_mount_w_k = g;
            k.params.battery_eps = eps;
            row(nm, &run(&k));
        }
        let mut k = cold_case("C10 cold real LTAN 23:30 June 0.70 W", Att::Tumble, Mode::Safe, hw);
        k.t0 = from_utc(2029, 6, 21, 0, 0, 0.0);
        k.ltan = 23.5;
        k.flux = None;
        row("C10 cold real LTAN 23:30 June 0.70 W", &run(&k));
    }
    // THM-03: wings stowed, spin-pointed, SAFE: heater power sweep (V²/207)
    if what == "all" || what == "thm03" {
        for v in [12.0, 12.4, 12.8, 13.2, 13.6, 14.0, 14.4, 14.8] {
            let w = v * v / 207.0;
            let nm = format!("C14 stowed spin-pointed SAFE at {v:.1} V ({w:.2} W)");
            let mut k = cold_case(&nm, Att::SpinPointed, Mode::Safe, w);
            k.wings = [false, false];
            row(&nm, &run(&k));
        }
        for v in [12.0, 13.2, 14.0] {
            let w = v * v / 207.0;
            let nm = format!("C14E stowed spin-pointed EMERGENCY at {v:.1} V ({w:.2} W)");
            let mut k = cold_case(&nm, Att::SpinPointed, Mode::Emergency, w);
            k.wings = [false, false];
            row(&nm, &run(&k));
            let nm = format!("C6 stowed tumbling SAFE at {v:.1} V ({w:.2} W)");
            let mut k = cold_case(&nm, Att::Tumble, Mode::Safe, w);
            k.wings = [false, false];
            row(&nm, &run(&k));
        }
        // real orbit, stowed, coldest season: LTAN 23:30 June, sun-pointed
        let w = 12.0 * 12.0 / 207.0;
        let mut k = cold_case("C14R stowed spin-pointed LTAN 23:30 June 12.0 V", Att::SpinPointed, Mode::Safe, w);
        k.t0 = from_utc(2029, 6, 21, 0, 0, 0.0);
        k.ltan = 23.5;
        k.flux = None;
        k.albedo = 0.25;
        k.ir = 218.0;
        k.wings = [false, false];
        row("C14R stowed spin-pointed LTAN 23:30 June 12.0 V", &run(&k));
    }
    // P3S-01: hinge, burn board, line and wing at the burns, cold case
    if what == "all" || what == "p3s" {
        for (soak_h, tag) in [(2.0, "soak2h"), (3.0, "soak3h")] {
            let init = dispenser_soak(15.0, soak_h * 3600.0);
            println!(
                "INIT|{tag}|W1 {:.1} +X {:.1} OBC {:.1} RF {:.1} Jet {:.1} bat {:.1}",
                c(init[W1]),
                c(init[PX]),
                c(init[OBC]),
                c(init[RF]),
                c(init[JET]),
                c(init[BAT])
            );
            for (env, t0, ltan) in
                [("beta0", from_utc(2029, 3, 20, 0, 0, 0.0), 12.0), ("L3real", from_utc(2028, 11, 1, 6, 0, 0.0), 22.5)]
            {
                let base = {
                    let mut k = cold_case("p3s", Att::TipOff, Mode::Leop, 12.0 * 12.0 / 207.0);
                    k.t0 = t0;
                    k.ltan = ltan;
                    if env == "L3real" {
                        k.flux = None;
                        k.albedo = ALBEDO;
                        k.ir = EARTH_IR_W_M2;
                        k.years = 0.0;
                    }
                    k.init = Some(init);
                    k.deploy_at = None;
                    k.wings = [false, false];
                    k.run_s = Some(131.0 * 60.0);
                    k.snaps = vec![1800.0, 4800.0, 7800.0];
                    k
                };
                let o = Orbit::sso(ALT, ltan, t0);
                let p = o.period();
                // sweep the separation phase; find eclipse entry
                let mut worst: [(f64, f64); 3] = [(1e9, 0.0); 3];
                let mut ecl_entry = None;
                let mut prev_lit = true;
                let mut ph = 0.0;
                while ph < p {
                    let tt = t0 + ph;
                    let (r, _) = o.state_at(tt);
                    let (s, d) = sun_eci(tt);
                    let lit = illumination(r, s, d) > 0.5;
                    if prev_lit && !lit && ecl_entry.is_none() && ph > 0.0 {
                        ecl_entry = Some(ph);
                    }
                    prev_lit = lit;
                    let mut k = base.clone();
                    k.phase_s = ph;
                    let out = run(&k);
                    for (i, sn) in out.snaps.iter().enumerate() {
                        let m = c(sn.2[0].k.min(sn.2[1].k)).min(sn.1[W1].min(sn.1[W2]));
                        if m < worst[i].0 {
                            worst[i] = (m, ph);
                        }
                    }
                    ph += 120.0;
                }
                let ee = ecl_entry.unwrap_or(0.0);
                println!(
                    "ECL|{tag}|{env}|eclipse entry {:.1} min after the node; period {:.1} min",
                    ee / 60.0,
                    p / 60.0
                );
                for (lab, ph) in [
                    ("sep at eclipse entry", ee),
                    ("sep 6 min after eclipse entry (burn at exit)", ee + 360.0),
                    ("worst phase for T+30", worst[0].1),
                    ("worst phase for T+80", worst[1].1),
                    ("worst phase for T+130", worst[2].1),
                ] {
                    let mut k = base.clone();
                    k.phase_s = ph;
                    let out = run(&k);
                    snap_print(&format!("{tag}|{env}|{lab} (phase {:.0} min)", ph / 60.0), &out);
                    let m = out.sub_min;
                    println!(
                        "SUBMIN|{tag}|{env}|{lab}|stowed run min: hingeF {:.1} hingeW {:.1} burnbd {:.1} line {:.1}|wing {:.1}",
                        c(m[0].h.min(m[1].h)),
                        c(m[0].k.min(m[1].k)),
                        c(m[0].b.min(m[1].b)),
                        c(m[0].l.min(m[1].l)),
                        out.allmin[W1].min(out.allmin[W2])
                    );
                }
                for i in 0..3 {
                    println!(
                        "WORST|{tag}|{env}|burn {}|coldest wing-side {:.1} at phase {:.0} min",
                        i,
                        worst[i].0,
                        worst[i].1 / 60.0
                    );
                }
                // sensitivity: poor ear joint and a better pin path
                let mut k = base.clone();
                k.phase_s = ee;
                k.sub = SubParams { g_hinge_frame: 0.05, g_hinge_wing: 0.03 };
                snap_print(&format!("{tag}|{env}|sens G_frame 0.05 G_wing 0.03"), &run(&k));
                // the deployed wing after a burn at T+30 (the −45 °C figure)
                let mut k = base.clone();
                k.phase_s = ee;
                k.deploy_at = Some(1800.0);
                k.run_s = Some(p);
                let out = run(&k);
                println!(
                    "DEPLOYED|{tag}|{env}|deploy at T+30 from eclipse entry: wing min over the first orbit {:.1}",
                    out.allmin[W1].min(out.allmin[W2])
                );
            }
        }
        // steady stowed (ground-commanded retry any later orbit): C6/C14-like, LEOP loads absent; sunlit minima
        for (nm, att) in
            [("stowed steady SAFE tumbling", Att::Tumble), ("stowed steady SAFE spin-pointed", Att::SpinPointed)]
        {
            let mut k = cold_case(nm, att, Mode::Safe, 12.0 * 12.0 / 207.0);
            k.wings = [false, false];
            let p = Orbit::sso(ALT, 12.0, k.t0).period();
            let t_last = 24.0 * p;
            k.snaps = (0..95).map(|i| t_last + i as f64 * 60.0).collect();
            let out = run(&k);
            let mut lit_min = (1e9f64, 1e9f64);
            let mut all_min = (1e9f64, 1e9f64);
            for (_, v, sub, lit, _) in &out.snaps {
                let w = v[W1].min(v[W2]);
                let kk = c(sub[0].k.min(sub[1].k));
                all_min = (all_min.0.min(w), all_min.1.min(kk));
                if *lit {
                    lit_min = (lit_min.0.min(w), lit_min.1.min(kk));
                }
            }
            println!(
                "STEADY|{nm}|wing min {:.1}, hinge wing side {:.1}; in sunlight: wing {:.1}, hinge {:.1}",
                all_min.0, all_min.1, lit_min.0, lit_min.1
            );
        }
    }
    // THM-04: LEOP stack temperatures, variants of L3 and L1
    if what == "all" || what == "thm04" {
        let init2 = dispenser_soak(15.0, 7200.0);
        let init3 = dispenser_soak(15.0, 10800.0);
        let mk = |nm: &str, real: bool, init: [f64; N_NODES]| {
            let mut k = cold_case(nm, Att::TipOff, Mode::Leop, 1.0);
            if real {
                k.t0 = from_utc(2028, 11, 1, 6, 0, 0.0);
                k.ltan = 22.5;
                k.flux = None;
                k.albedo = ALBEDO;
                k.ir = EARTH_IR_W_M2;
                k.years = 0.0;
            }
            k.init = Some(init);
            k.orbits = 1.0;
            k
        };
        let pr = |nm: &str, o: &Out| {
            println!(
                "LEOP|{nm}|OBC min {:.1}|RF min {:.1}|Jet min {:.1}|bat min {:.1} (≥0 after {:.0} min)|body min {:.1}|below: OBC<-20 {:.0} min, RF<-20 {:.0} min, Jet<-25 {:.0} min|stack heater {:.2} Wh",
                o.allmin[OBC],
                o.allmin[RF],
                o.allmin[JET],
                o.allmin[BAT],
                o.t_above0,
                [PX, MX, PY, MY, PZ, MZ].iter().map(|&k| o.allmin[k]).fold(f64::INFINITY, f64::min),
                o.below[0],
                o.below[1],
                o.below[2],
                o.stack_heater_wh
            );
        };
        pr("L1 as §4.1", &run(&mk("L1", false, init2)));
        pr("L3 as §4.1", &run(&mk("L3", true, init2)));
        let mut k = mk("L3", true, init2);
        k.deploy_at = Some(1800.0);
        pr("L3 wings stowed to T+30", &run(&k));
        let mut k = mk("L3", true, init2);
        k.deploy_at = Some(1800.0);
        k.extra_obc = (1.0, 1.4 * 3600.0);
        pr("L3 stowed to T+30, detumble 1.30 W (board) for 1.4 h", &run(&k));
        let mut k = mk("L3", true, init3);
        k.deploy_at = Some(1800.0);
        pr("L3 stowed to T+30, 3 h soak", &run(&k));
        let mut k = mk("L1", false, init2);
        k.deploy_at = Some(1800.0);
        pr("L1 wings stowed to T+30", &run(&k));
        // L1 from eclipse separation (β = 0)
        let o = Orbit::sso(ALT, 12.0, from_utc(2029, 3, 20, 0, 0, 0.0));
        let mut ee = 0.0;
        let mut prev = true;
        let mut ph = 0.0;
        while ph < o.period() {
            let tt = from_utc(2029, 3, 20, 0, 0, 0.0) + ph;
            let (r, _) = o.state_at(tt);
            let (s, d) = sun_eci(tt);
            let lit = illumination(r, s, d) > 0.5;
            if prev && !lit && ph > 0.0 {
                ee = ph;
                break;
            }
            prev = lit;
            ph += 30.0;
        }
        let mut k = mk("L1e", false, init2);
        k.deploy_at = Some(1800.0);
        k.phase_s = ee;
        pr("L4 β 0 eclipse separation, stowed to T+30", &run(&k));
        let mut k3 = mk("L1e3", false, init3);
        k3.deploy_at = Some(1800.0);
        k3.phase_s = ee;
        pr("L4 3 h soak", &run(&k3));
        let mut kd = k.clone();
        kd.extra_obc = (1.0, 0.8 * 3600.0);
        pr("L4 + detumble 1.16 W coils for 0.8 h (uncredited)", &run(&kd));
        for w in [1.0, 3.0, 5.0] {
            let mut kh = k.clone();
            kh.stack_heater = Some((w, -15.0, -10.0));
            pr(&format!("L4 + stack heater {w} W thermostat"), &run(&kh));
        }
        for (wo, wr) in [(2.0, 1.0), (3.0, 2.0)] {
            let mut kh = k.clone();
            kh.stack_heater = Some((wo, 100.0, 200.0));
            kh.rf_heater = wr;
            pr(&format!("L4 + {wo} W stack and {wr} W RF heaters on"), &run(&kh));
        }
        for w in [0.5, 1.0, 1.5, 2.0, 3.0] {
            let mut k = mk("L3", true, init2);
            k.deploy_at = Some(1800.0);
            k.stack_heater = Some((w, -15.0, -10.0));
            pr(&format!("L3 stowed to T+30 + stack heater {w} W (on < −15, off > −10 °C)"), &run(&k));
        }
        for w in [1.0, 2.0, 3.0] {
            let mut k = mk("L3", true, init2);
            k.deploy_at = Some(1800.0);
            k.stack_heater = Some((w, 100.0, 200.0));
            pr(&format!("L3 stowed to T+30 + stack heater {w} W always on"), &run(&k));
        }
        let mut k = mk("L3", true, init2);
        k.deploy_at = Some(1800.0);
        k.rf_gate_c = Some(-20.0);
        pr("L3 stowed to T+30, RF board off until −20 °C", &run(&k));
    }
}
