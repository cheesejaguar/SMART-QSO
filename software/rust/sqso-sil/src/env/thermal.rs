//! Lumped-parameter thermal network of the 1U with its two wings.
//!
//! Twelve nodes: the six body panels (each the panel substrate with its share
//! of the frame), the two wings, and four internal nodes (battery pack, OBC
//! and EPS board stack, RF board with the PA, Jetson module with its heat
//! spreader). Explicit integration with internal sub-steps of at most half
//! the shortest node time constant, so any caller step (1–60 s) is stable.
//!
//! # External surfaces (`hardware/structure/THERMAL_DESIGN.md` §3.1)
//!
//! * Long faces (±X, ±Y; 100 × 113.5 mm): 53 % solar cells (α 0.92,
//!   ε 0.85), the rest black anodize (α 0.88, ε 0.85): α 0.90, ε 0.85.
//! * End faces (±Z; 100 × 100 mm): black anodize, α 0.88, ε 0.85.
//! * Wings (83 × 98 mm, cells on both sides, 74 % cell coverage): α 0.91,
//!   ε 0.85 on each side.
//! * Deployed, each wing's back faces its adjacent ±X body face at 90°: view
//!   factor 0.2 from the body face (perpendicular plates with a common edge),
//!   modelled as a radiative link and a matching loss of view to space. The
//!   wings' shadow on the body faces is ignored here (it is in the solar
//!   model). Stowed, the wing covers its ±X face: only the wing front sees
//!   space, the wing back exchanges with the covered face (ε_eff 0.74) and
//!   conducts through the hold-down (0.05 W/K).
//! * Loads: direct sun α S A cos⁺θ; albedo α · 0.30 · S · cos⁺ζ · F(θₙ);
//!   Earth IR ε · 237 W/m² · F(θₙ), with F the plate-to-Earth view factor
//!   ([`super::earth::plate_view_factor`]) and ζ the solar zenith angle at the
//!   sub-satellite point. Electrical power taken out by the cells
//!   (`elec_w`) is subtracted from the node that absorbed it; the caller puts
//!   the dissipation back on the internal nodes.
//!
//! # Heat capacities (from `docs/environment/MASS_BUDGET.md` v2.0)
//!
//! | Node | J/K | Basis |
//! |------|-----|-------|
//! | ±X, ±Y panel | 76 | FR-4 substrate 24.7 g × 1.15 J/g/K, two cells, 50 g of frame × 0.90 |
//! | ±Z panel | 30 | end panel 10 g, 20 g of frame and switches, × 0.90 |
//! | Wing | 42 | substrate 24.7 g, four cells 14.4 g × 0.4, hinge 16 g × 0.5 |
//! | Battery | 250 | cells 200 g × 1.0, board 15 g × 1.15, Al holder 34 g × 0.90 |
//! | OBC/EPS stack | 200 | EPS, OBC, ADCS and GNSS boards (≈ 185 g) with harness share |
//! | RF | 130 | 120 g of boards, PA, filters, shields |
//! | Jetson + spreader | 220 | module, carrier, mounts 180 g × 0.9, Al spreader 65 g × 0.9 |
//!
//! THERMAL_DESIGN §4.1 lists smaller capacitances (battery 120 J/K, OBC
//! 15 J/K) that do not match the re-issued mass budget; the masses govern.
//!
//! # Conductances (W/K)
//!
//! * Internal to structure: battery 1.0, OBC stack 0.5, RF 0.3
//!   (THERMAL_DESIGN §4.2), Jetson 0.44 (BASELINE §6: audit estimate of the
//!   documented stack, TB-07; the 2.0 W/K of THERMAL_DESIGN is superseded),
//!   each spread over the frame as 20 % to each long face and 10 % to each
//!   end face.
//! * Internal to internal (stack standoffs, assumed): battery–OBC 0.1,
//!   OBC–RF 0.1, OBC–Jetson 0.05.
//! * Panel to panel through the frame (assumed): adjacent long faces 0.25,
//!   long face to end face 0.15.
//! * Deployed wing hinge (assumed, stainless): 0.01 to each of the two faces
//!   at the hinge edge.

use super::earth::{ALBEDO, EARTH_IR_W_M2, plate_view_factor};
use super::math::{SIGMA, T0_K, V3};
use super::solar::{self, N_STRINGS, StringOut};

/// Number of nodes.
pub const N_NODES: usize = 12;
/// +X body panel.
pub const PX: usize = 0;
/// −X body panel.
pub const MX: usize = 1;
/// +Y body panel.
pub const PY: usize = 2;
/// −Y body panel.
pub const MY: usize = 3;
/// +Z end panel.
pub const PZ: usize = 4;
/// −Z end panel.
pub const MZ: usize = 5;
/// Wing 1 (hinged on the +X/+Y edge).
pub const W1: usize = 6;
/// Wing 2 (hinged on the −X/+Y edge).
pub const W2: usize = 7;
/// Battery pack.
pub const BAT: usize = 8;
/// OBC / EPS / ADCS board stack.
pub const OBC: usize = 9;
/// RF board (transmitter, PA, receiver).
pub const RF: usize = 10;
/// Jetson module with heat spreader.
pub const JET: usize = 11;

/// Node names, for reports.
pub const NODE_NAMES: [&str; N_NODES] =
    ["+X", "-X", "+Y", "-Y", "+Z", "-Z", "W1", "W2", "battery", "OBC", "RF", "Jetson"];

/// Heat capacity of each node (J/K).
pub const CAPACITY_J_K: [f64; N_NODES] = [76.0, 76.0, 76.0, 76.0, 30.0, 30.0, 42.0, 42.0, 250.0, 200.0, 130.0, 220.0];

const A_LONG: f64 = 0.100 * 0.1135;
const A_END: f64 = 0.100 * 0.100;
const A_WING: f64 = 0.083 * 0.098;
const ALPHA_LONG: f64 = 0.90;
const ALPHA_END: f64 = 0.88;
const ALPHA_WING: f64 = 0.91;
const EPS: f64 = 0.85;
const F_BODY_TO_WING: f64 = 0.20;
const EPS_EFF_STOWED: f64 = 1.0 / (2.0 / EPS - 1.0);
const T_MAX_K: f64 = 420.0;

/// One radiating surface.
#[derive(Clone, Copy, Debug, Default)]
struct Surface {
    node: usize,
    normal: V3,
    area: f64,
    alpha: f64,
    /// ε × area × fraction of the hemisphere that sees space/Earth.
    eps_area: f64,
}

/// A conductive (G in W/K) or radiative (coefficient σ ε A F, W/K⁴) link.
#[derive(Clone, Copy, Debug, Default)]
struct Link {
    a: usize,
    b: usize,
    g: f64,
}

const MAX_SURF: usize = 10;
const MAX_LINKS: usize = 48;
const MAX_RAD: usize = 2;

/// The network for one wing configuration.
#[derive(Clone, Copy, Debug)]
struct Network {
    surf: [Surface; MAX_SURF],
    n_surf: usize,
    links: [Link; MAX_LINKS],
    n_links: usize,
    rad: [Link; MAX_RAD],
    n_rad: usize,
    dt_max: f64,
}

impl Network {
    fn push_surf(&mut self, node: usize, normal: V3, area: f64, alpha: f64, view: f64) {
        if self.n_surf < MAX_SURF {
            self.surf[self.n_surf] = Surface { node, normal, area, alpha, eps_area: EPS * area * view };
            self.n_surf += 1;
        }
    }

    fn push_link(&mut self, a: usize, b: usize, g: f64) {
        if self.n_links < MAX_LINKS {
            self.links[self.n_links] = Link { a, b, g };
            self.n_links += 1;
        }
    }

    fn push_rad(&mut self, a: usize, b: usize, coeff: f64) {
        if self.n_rad < MAX_RAD {
            self.rad[self.n_rad] = Link { a, b, g: coeff };
            self.n_rad += 1;
        }
    }

    fn build(wings: [bool; 2]) -> Network {
        let mut n = Network {
            surf: [Surface::default(); MAX_SURF],
            n_surf: 0,
            links: [Link::default(); MAX_LINKS],
            n_links: 0,
            rad: [Link::default(); MAX_RAD],
            n_rad: 0,
            dt_max: 1.0,
        };
        n.structure_links();
        let x = V3::X;
        let y = V3::Y;
        let z = V3::Z;
        n.push_surf(PY, y, A_LONG, ALPHA_LONG, 1.0);
        n.push_surf(MY, -y, A_LONG, ALPHA_LONG, 1.0);
        n.push_surf(PZ, z, A_END, ALPHA_END, 1.0);
        n.push_surf(MZ, -z, A_END, ALPHA_END, 1.0);
        for (k, (wing, face, sign)) in [(W1, PX, 1.0), (W2, MX, -1.0)].into_iter().enumerate() {
            let out = x * sign;
            if wings[k] {
                n.push_surf(face, out, A_LONG, ALPHA_LONG, 1.0 - F_BODY_TO_WING);
                n.push_surf(wing, y, A_WING, ALPHA_WING, 1.0);
                n.push_surf(wing, -y, A_WING, ALPHA_WING, 1.0 - F_BODY_TO_WING * A_LONG / A_WING);
                n.push_rad(face, wing, SIGMA * EPS * EPS * A_LONG * F_BODY_TO_WING);
                n.push_link(wing, face, 0.01);
                n.push_link(wing, PY, 0.01);
            } else {
                n.push_surf(face, out, A_LONG - A_WING, ALPHA_LONG, 1.0);
                n.push_surf(wing, out, A_WING, ALPHA_WING, 1.0);
                n.push_rad(face, wing, SIGMA * EPS_EFF_STOWED * A_WING);
                n.push_link(wing, face, 0.05);
            }
        }
        n.dt_max = n.stable_dt();
        n
    }

    fn structure_links(&mut self) {
        for (a, b) in [(PX, PY), (PY, MX), (MX, MY), (MY, PX)] {
            self.push_link(a, b, 0.25);
        }
        for a in [PX, MX, PY, MY] {
            self.push_link(a, PZ, 0.15);
            self.push_link(a, MZ, 0.15);
        }
        for (node, g) in [(BAT, 1.0), (OBC, 0.5), (RF, 0.3), (JET, 0.44)] {
            for p in [PX, MX, PY, MY] {
                self.push_link(node, p, 0.2 * g);
            }
            self.push_link(node, PZ, 0.1 * g);
            self.push_link(node, MZ, 0.1 * g);
        }
        self.push_link(BAT, OBC, 0.1);
        self.push_link(OBC, RF, 0.1);
        self.push_link(OBC, JET, 0.05);
    }

    /// Half the shortest node time constant, with radiation linearised at
    /// 420 K.
    fn stable_dt(&self) -> f64 {
        let mut g = [0.0; N_NODES];
        for l in &self.links[..self.n_links] {
            g[l.a] += l.g;
            g[l.b] += l.g;
        }
        let lin = 4.0 * T_MAX_K.powi(3);
        for s in &self.surf[..self.n_surf] {
            g[s.node] += SIGMA * s.eps_area * lin;
        }
        for r in &self.rad[..self.n_rad] {
            g[r.a] += r.g * lin;
            g[r.b] += r.g * lin;
        }
        let mut tau = f64::INFINITY;
        for k in 0..N_NODES {
            if g[k] > 0.0 {
                tau = tau.min(CAPACITY_J_K[k] / g[k]);
            }
        }
        0.5 * tau
    }
}

/// Inputs of one thermal step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThermalInputs {
    /// Unit Sun direction in the body frame.
    pub sun_body: V3,
    /// Illuminated fraction of the solar disc (0..1).
    pub illum: f64,
    /// Solar irradiance (W/m²).
    pub flux: f64,
    /// Unit nadir direction in the body frame.
    pub nadir_body: V3,
    /// Distance from the Earth's centre (m).
    pub r_m: f64,
    /// Cosine of the solar zenith angle at the sub-satellite point (r̂ · ŝ),
    /// for the albedo.
    pub cos_zeta: f64,
    /// Heat dissipated in each node (W): electronics, heaters, battery I²R.
    pub internal_w: [f64; N_NODES],
    /// Electrical power taken out by the solar cells on each node (W).
    pub elec_w: [f64; N_NODES],
    /// Whether each wing (W1, W2) is deployed.
    pub wings_deployed: [bool; 2],
}

impl ThermalInputs {
    /// Inputs with no internal power, no electrical extraction and both wings
    /// deployed.
    pub fn new(sun_body: V3, illum: f64, flux: f64, nadir_body: V3, r_m: f64, cos_zeta: f64) -> ThermalInputs {
        ThermalInputs {
            sun_body,
            illum,
            flux,
            nadir_body,
            r_m,
            cos_zeta,
            internal_w: [0.0; N_NODES],
            elec_w: [0.0; N_NODES],
            wings_deployed: [true, true],
        }
    }
}

/// Electrical power per thermal node from the solar strings' outputs.
pub fn elec_from_strings(strings: &[StringOut; N_STRINGS]) -> [f64; N_NODES] {
    let mut e = [0.0; N_NODES];
    e[PX] = strings[solar::PX].pmp;
    e[MX] = strings[solar::MX].pmp;
    e[PY] = strings[solar::PY].pmp;
    e[MY] = strings[solar::MY].pmp;
    e[W1] = strings[solar::W1F].pmp + strings[solar::W1B].pmp;
    e[W2] = strings[solar::W2F].pmp + strings[solar::W2B].pmp;
    e
}

/// The thermal network and its node temperatures.
#[derive(Clone, Debug)]
pub struct Thermal {
    nets: [Network; 4],
    /// Node temperatures (K).
    pub temp_k: [f64; N_NODES],
}

impl Thermal {
    /// A network with every node at `initial_c` (°C).
    pub fn new(initial_c: f64) -> Thermal {
        let b = |a: bool, c: bool| Network::build([a, c]);
        Thermal {
            nets: [b(false, false), b(true, false), b(false, true), b(true, true)],
            temp_k: [initial_c + T0_K; N_NODES],
        }
    }

    fn net(&self, wings: [bool; 2]) -> &Network {
        &self.nets[(usize::from(wings[0]) + 2 * usize::from(wings[1])).min(3)]
    }

    /// Absorbed environmental heat minus electrical extraction, per node (W).
    fn absorbed(net: &Network, inp: &ThermalInputs) -> [f64; N_NODES] {
        let mut q = inp.internal_w;
        let direct = inp.flux * inp.illum.clamp(0.0, 1.0);
        let albedo = ALBEDO * inp.flux * inp.cos_zeta.max(0.0);
        for s in &net.surf[..net.n_surf] {
            let f_earth = plate_view_factor(s.normal.dot(inp.nadir_body), inp.r_m);
            let sun = direct * s.normal.dot(inp.sun_body).max(0.0);
            q[s.node] += s.alpha * s.area * (sun + albedo * f_earth) + EPS * s.area * EARTH_IR_W_M2 * f_earth;
        }
        for (qk, e) in q.iter_mut().zip(inp.elec_w.iter()) {
            *qk -= e;
        }
        q
    }

    /// Advance by `dt` seconds.
    pub fn step(&mut self, dt: f64, inp: &ThermalInputs) {
        if dt <= 0.0 {
            return;
        }
        let net = *self.net(inp.wings_deployed);
        let q_ext = Thermal::absorbed(&net, inp);
        let n = ((dt / net.dt_max).ceil() as u32).max(1);
        let h = dt / f64::from(n);
        for _ in 0..n {
            let mut q = q_ext;
            let t = &self.temp_k;
            for s in &net.surf[..net.n_surf] {
                q[s.node] -= SIGMA * s.eps_area * t[s.node].powi(4);
            }
            for l in &net.links[..net.n_links] {
                let f = l.g * (t[l.a] - t[l.b]);
                q[l.a] -= f;
                q[l.b] += f;
            }
            for r in &net.rad[..net.n_rad] {
                let f = r.g * (t[r.a].powi(4) - t[r.b].powi(4));
                q[r.a] -= f;
                q[r.b] += f;
            }
            for k in 0..N_NODES {
                self.temp_k[k] = (self.temp_k[k] + h * q[k] / CAPACITY_J_K[k]).max(3.0);
            }
        }
    }

    /// Temperature of a node (°C).
    pub fn temp_c(&self, node: usize) -> f64 {
        self.temp_k.get(node).map_or(f64::NAN, |t| t - T0_K)
    }

    /// Battery temperature (°C).
    pub fn battery_c(&self) -> f64 {
        self.temp_c(BAT)
    }

    /// OBC/EPS stack temperature (°C).
    pub fn obc_c(&self) -> f64 {
        self.temp_c(OBC)
    }

    /// RF board temperature (°C).
    pub fn rf_c(&self) -> f64 {
        self.temp_c(RF)
    }

    /// Jetson temperature (°C).
    pub fn jetson_c(&self) -> f64 {
        self.temp_c(JET)
    }

    /// Cell temperature of each solar string (°C), from the panel nodes.
    pub fn string_temps_c(&self) -> [f64; N_STRINGS] {
        let w1 = self.temp_c(W1);
        let w2 = self.temp_c(W2);
        [self.temp_c(PX), self.temp_c(MX), self.temp_c(PY), self.temp_c(MY), w1, w1, w2, w2]
    }

    /// Heat radiated to space by every surface (W), for energy-balance
    /// checks.
    pub fn radiated_w(&self, wings_deployed: [bool; 2]) -> f64 {
        let net = self.net(wings_deployed);
        net.surf[..net.n_surf].iter().map(|s| SIGMA * s.eps_area * self.temp_k[s.node].powi(4)).sum()
    }

    /// Largest internal sub-step (s) for a wing configuration.
    pub fn max_substep_s(&self, wings_deployed: [bool; 2]) -> f64 {
        self.net(wings_deployed).dt_max
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::eclipse::illumination;
    use crate::env::math::{Quat, R_EARTH};
    use crate::env::orbit::Orbit;
    use crate::env::solar::{Array, ArrayInputs};
    use crate::env::sun::{solar_flux, sun_eci};
    use crate::env::time::from_utc;

    #[test]
    fn substep_is_stable() {
        let th = Thermal::new(20.0);
        for w in [[true, true], [false, false]] {
            let d = th.max_substep_s(w);
            assert!(d > 2.0 && d < 60.0, "{d}");
        }
        // a 60 s caller step does not blow up
        let mut th = Thermal::new(20.0);
        let inp = ThermalInputs::new(V3::Y, 1.0, 1361.0, -V3::Y, R_EARTH + 500e3, 1.0);
        for _ in 0..1000 {
            th.step(60.0, &inp);
        }
        for k in 0..N_NODES {
            assert!(th.temp_c(k) > -100.0 && th.temp_c(k) < 150.0);
        }
    }

    #[test]
    fn equilibrium_energy_balance() {
        // deep space, Sun on +Y, 2 W in the OBC: absorbed = radiated at
        // steady state
        let mut th = Thermal::new(0.0);
        let mut inp = ThermalInputs::new(V3::Y, 1.0, 1361.0, -V3::Y, 1e12, 0.0);
        inp.internal_w[OBC] = 2.0;
        for _ in 0..20_000 {
            th.step(30.0, &inp);
        }
        let absorbed = 2.0 + ALPHA_LONG * A_LONG * 1361.0 + 2.0 * ALPHA_WING * A_WING * 1361.0;
        let out = th.radiated_w([true, true]);
        assert!((out / absorbed - 1.0).abs() < 1e-3, "{out} {absorbed}");
        // the sunlit side is warmer than the shaded side, internal nodes in
        // between and the OBC above the structure
        assert!(th.temp_c(PY) > th.temp_c(MY));
        assert!(th.temp_c(OBC) > th.temp_c(BAT));
        // isolated, no input: uniform internal heating only
        let mut th = Thermal::new(0.0);
        let mut inp = ThermalInputs::new(V3::Y, 0.0, 0.0, -V3::Y, 1e12, 0.0);
        inp.internal_w[BAT] = 1.0;
        for _ in 0..20_000 {
            th.step(30.0, &inp);
        }
        assert!((th.radiated_w([true, true]) - 1.0).abs() < 1e-3);
    }

    /// Cold (β ≈ 0, tumbling, safe mode ≈ 1 W) and hot (dawn–dusk, no
    /// eclipse, +Y sun-pointed, ≈ 2.3 W) orbit cases; returns (battery mean,
    /// battery min, battery max, +X panel swing, +X mean) over the last orbit.
    fn orbit_case(hot: bool) -> (f64, f64, f64, f64, f64) {
        let t0 = if hot { from_utc(2028, 6, 21, 0, 0, 0.0) } else { from_utc(2028, 3, 20, 0, 0, 0.0) };
        let o = Orbit::sso(500e3, if hot { 6.0 } else { 12.0 }, t0);
        let array = Array::new();
        let mut th = Thermal::new(10.0);
        let dt = 10.0;
        let p = o.period();
        let orbits = 25.0;
        let spin_axis = V3::new(0.3, 0.8, 0.5).unit();
        let mut samples = Vec::new();
        let mut t = 0.0;
        while t < orbits * p {
            let tt = t0 + t;
            let (r, _) = o.state_at(tt);
            let (s, d) = sun_eci(tt);
            let q = if hot {
                // +Y to the Sun, Z towards the orbit normal
                let yb = s;
                let zb = o.normal().cross(s).cross(s).unit() * -1.0;
                let xb = yb.cross(zb);
                quat_from_axes(xb, yb, zb)
            } else {
                Quat::from_axis_angle(spin_axis, 0.035 * t)
            };
            let illum = illumination(r, s, d);
            let sun_b = q.rotate_inv(s);
            let nadir_b = q.rotate_inv(-r.unit());
            let flux = solar_flux(d);
            let mut inp = ThermalInputs::new(sun_b, illum, flux, nadir_b, r.norm(), r.unit().dot(s));
            let strings = array.strings(&ArrayInputs {
                sun_body: sun_b,
                illum,
                flux,
                nadir_body: nadir_b,
                albedo_w_m2: 0.0,
                r_m: r.norm(),
                temp_c: th.string_temps_c(),
                years_in_orbit: 1.0,
                wings_deployed: [true, true],
            });
            inp.elec_w = elec_from_strings(&strings);
            if hot {
                inp.internal_w[OBC] = 0.8;
                inp.internal_w[RF] = 0.6;
                inp.internal_w[JET] = 0.8;
                inp.internal_w[BAT] = 0.1;
            } else {
                inp.internal_w[OBC] = 0.65;
                inp.internal_w[RF] = 0.3;
                inp.internal_w[BAT] = 0.05;
            }
            // energy-consistent EPS: the array delivers only what the loads
            // dissipate (battery full, MPPT curtails); the rest stays as heat
            // in the cells
            let harvested: f64 = inp.elec_w.iter().sum();
            let load: f64 = inp.internal_w.iter().sum();
            if harvested > load {
                let k = load / harvested;
                inp.elec_w.iter_mut().for_each(|e| *e *= k);
            }
            th.step(dt, &inp);
            if t > (orbits - 1.0) * p {
                samples.push((th.battery_c(), th.temp_c(PX)));
            }
            t += dt;
        }
        let n = samples.len() as f64;
        let bat_mean = samples.iter().map(|s| s.0).sum::<f64>() / n;
        let bat_min = samples.iter().map(|s| s.0).fold(f64::INFINITY, f64::min);
        let bat_max = samples.iter().map(|s| s.0).fold(f64::NEG_INFINITY, f64::max);
        let px_min = samples.iter().map(|s| s.1).fold(f64::INFINITY, f64::min);
        let px_max = samples.iter().map(|s| s.1).fold(f64::NEG_INFINITY, f64::max);
        let px_mean = samples.iter().map(|s| s.1).sum::<f64>() / n;
        (bat_mean, bat_min, bat_max, px_max - px_min, px_mean)
    }

    fn quat_from_axes(xb: V3, yb: V3, zb: V3) -> Quat {
        // rotation matrix with body axes as columns (body → ECI)
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

    #[test]
    fn quat_from_axes_round_trip() {
        let q = Quat::from_axis_angle(V3::new(1.0, -2.0, 0.5), 2.3);
        let r = quat_from_axes(q.rotate(V3::X), q.rotate(V3::Y), q.rotate(V3::Z));
        let v = V3::new(0.3, 0.4, -0.2);
        assert!((r.rotate(v) - q.rotate(v)).norm() < 1e-12);
    }

    #[test]
    fn hot_and_cold_orbit_cases() {
        let (cm, cmin, cmax, cswing, cpx) = orbit_case(false);
        let (hm, hmin, hmax, hswing, hpx) = orbit_case(true);
        println!(
            "cold case: battery {cm:.1} °C ({cmin:.1}…{cmax:.1}), +X panel mean {cpx:.1} °C swing {cswing:.1} K; \
             hot case: battery {hm:.1} °C ({hmin:.1}…{hmax:.1}), +X panel mean {hpx:.1} °C swing {hswing:.1} K"
        );
        assert!((-10.0..=40.0).contains(&cm), "{cm}");
        assert!((-10.0..=40.0).contains(&hm), "{hm}");
        assert!(hm > cm + 5.0);
        assert!(cswing > 10.0 && cswing < 80.0, "{cswing}");
    }
}
