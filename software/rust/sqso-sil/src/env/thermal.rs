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
//! # Environment and sink
//!
//! The albedo and the Earth IR default to the orbit means of
//! [`super::earth`]; the hot and cold design cases set them per case
//! ([`ThermalInputs::albedo`], [`ThermalInputs::earth_ir_w_m2`]). The
//! external surfaces radiate to a sink at [`ThermalInputs::sink_k`] (0 K:
//! deep space); inside the dispenser the sink is the dispenser wall, with an
//! emissivity factor and a rail conductance to it.
//!
//! # Conductances (W/K)
//!
//! * Internal to structure: battery [`BATTERY_MOUNT_W_K`] (an isolating
//!   mount, below), OBC stack 0.5, RF 0.3 (THERMAL_DESIGN §4.2), Jetson 0.44 (BASELINE §6: audit estimate of the
//!   documented stack, TB-07; the 2.0 W/K of THERMAL_DESIGN is superseded),
//!   each spread over the frame as 20 % to each long face and 10 % to each
//!   end face. The Jetson path is a design parameter ([`ThermalParams`]):
//!   BASELINE §6 requires ≥ 1 W/K, demonstrated on the EM.
//! * Internal to internal (stack standoffs, assumed): battery–OBC
//!   [`BATTERY_STACK_W_K`], OBC–RF 0.1, OBC–Jetson 0.05.
//!
//! The battery mount (THERMAL_DESIGN 1.3 §7.2; `docs/hardware/BUILD_VS_BUY.md`
//! §4): the pack sits on isolating bushings (G-10CR or PEEK 450G with
//! Ti-6Al-4V screws), designed for about 0.03 W/K to the frame. The model
//! keeps [`BATTERY_MOUNT_W_K`] = 0.05 W/K of conduction, conservative for the
//! cold case, and 0.02 W/K to the stack. A 1.0 W/K thermal pad (the earlier
//! design) kept the pack below 0 °C for a fifth of every orbit, so the charge
//! regulator refused the Sun (SYS-602).
//! * Battery radiation: the pack's exterior (≈ 75 × 70 × 22 mm,
//!   [`BATTERY_AREA_M2`]) exchanges with the panels' inner faces (FR-4,
//!   ε 0.85) through ε_eff = 1 / (1/ε_b + (A_b/A_enc)(1/0.85 − 1)), split
//!   over the panels like the conduction. With the specified low-emissivity
//!   exterior (ε_b 0.10) this adds ≈ 0.008 W/K at 0 °C; a black pack
//!   (ε_b 0.85) adds ≈ 0.07 W/K ([`ThermalParams`]).
//! * Panel to panel through the frame (assumed): adjacent long faces 0.25,
//!   long face to end face 0.15.
//! * Deployed wing hinge (assumed, stainless): 0.01 to each of the two faces
//!   at the hinge edge.

use super::earth::{ALBEDO, EARTH_IR_W_M2, plate_view_factor};
use super::math::{SIGMA, T0_K, V3};
use super::solar::{self, N_STRINGS, StringOut};

/// Battery pack to the frame by conduction, W/K (isolating mount; see the
/// module notes)
pub const BATTERY_MOUNT_W_K: f64 = 0.05;
/// Battery pack to the OBC stack, W/K
pub const BATTERY_STACK_W_K: f64 = 0.02;
/// Radiating area of the battery pack's exterior, m² (≈ 75 × 70 × 22 mm)
pub const BATTERY_AREA_M2: f64 = 0.0169;
/// Emissivity of the pack's exterior: the specified low-ε finish (VDA tape
/// or bare aluminium, ε < 0.1; BUILD_VS_BUY §4.3)
pub const BATTERY_EPS: f64 = 0.10;
/// Jetson heat path to the frame, W/K: the audit estimate of the documented
/// stack (BASELINE §6, TB-07); the requirement is ≥ 1 W/K
pub const JETSON_PATH_W_K: f64 = 0.44;

/// Design parameters of the network that the analyses vary.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThermalParams {
    /// Battery pack to the frame by conduction, W/K
    pub battery_mount_w_k: f64,
    /// Emissivity of the pack's exterior, for its radiation to the enclosure
    pub battery_eps: f64,
    /// Jetson module and spreader to the frame, W/K
    pub jetson_w_k: f64,
}

impl Default for ThermalParams {
    fn default() -> ThermalParams {
        ThermalParams { battery_mount_w_k: BATTERY_MOUNT_W_K, battery_eps: BATTERY_EPS, jetson_w_k: JETSON_PATH_W_K }
    }
}

/// Radiative coefficient (σ ε_eff A, W/K⁴) of the pack to the enclosure.
fn battery_rad_coeff(eps_b: f64) -> f64 {
    let a_enc = 4.0 * A_LONG + 2.0 * A_END;
    let eps_b = eps_b.clamp(0.01, 1.0);
    SIGMA * BATTERY_AREA_M2 / (1.0 / eps_b + BATTERY_AREA_M2 / a_enc * (1.0 / EPS - 1.0))
}

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
const MAX_RAD: usize = 8;

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

    fn build(wings: [bool; 2], params: &ThermalParams) -> Network {
        let mut n = Network {
            surf: [Surface::default(); MAX_SURF],
            n_surf: 0,
            links: [Link::default(); MAX_LINKS],
            n_links: 0,
            rad: [Link::default(); MAX_RAD],
            n_rad: 0,
            dt_max: 1.0,
        };
        n.structure_links(params);
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

    fn structure_links(&mut self, params: &ThermalParams) {
        for (a, b) in [(PX, PY), (PY, MX), (MX, MY), (MY, PX)] {
            self.push_link(a, b, 0.25);
        }
        for a in [PX, MX, PY, MY] {
            self.push_link(a, PZ, 0.15);
            self.push_link(a, MZ, 0.15);
        }
        for (node, g) in [(BAT, params.battery_mount_w_k), (OBC, 0.5), (RF, 0.3), (JET, params.jetson_w_k)] {
            for p in [PX, MX, PY, MY] {
                self.push_link(node, p, 0.2 * g);
            }
            self.push_link(node, PZ, 0.1 * g);
            self.push_link(node, MZ, 0.1 * g);
        }
        self.push_link(BAT, OBC, BATTERY_STACK_W_K);
        self.push_link(OBC, RF, 0.1);
        self.push_link(OBC, JET, 0.05);
        // The pack's radiation to the panels' inner faces, shared like the
        // conduction
        let c = battery_rad_coeff(params.battery_eps);
        for (p, share) in [(PX, 0.2), (MX, 0.2), (PY, 0.2), (MY, 0.2), (PZ, 0.1), (MZ, 0.1)] {
            self.push_rad(BAT, p, share * c);
        }
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
    /// Earth's Bond albedo for this step.
    pub albedo: f64,
    /// Earth's outgoing long-wave radiation (W/m²).
    pub earth_ir_w_m2: f64,
    /// Temperature of the radiative sink the external surfaces see (K; 0:
    /// deep space, otherwise for example the dispenser wall).
    pub sink_k: f64,
    /// Factor on the external surfaces' emissivity toward the sink (1 in
    /// space; ε_eff/ε inside a dispenser).
    pub sink_eps_factor: f64,
    /// Conductance from the four long panels to a boundary (W/K in all; the
    /// dispenser rails), 0 in orbit.
    pub boundary_w_k: f64,
    /// Temperature of that boundary (K).
    pub boundary_k: f64,
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
            albedo: ALBEDO,
            earth_ir_w_m2: EARTH_IR_W_M2,
            sink_k: 0.0,
            sink_eps_factor: 1.0,
            boundary_w_k: 0.0,
            boundary_k: 0.0,
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
    /// A network with every node at `initial_c` (°C) and the baseline
    /// parameters.
    pub fn new(initial_c: f64) -> Thermal {
        Thermal::with_params(initial_c, &ThermalParams::default())
    }

    /// A network with every node at `initial_c` (°C) and the given design
    /// parameters.
    pub fn with_params(initial_c: f64, params: &ThermalParams) -> Thermal {
        let b = |a: bool, c: bool| Network::build([a, c], params);
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
        let albedo = inp.albedo * inp.flux * inp.cos_zeta.max(0.0);
        for s in &net.surf[..net.n_surf] {
            let f_earth = plate_view_factor(s.normal.dot(inp.nadir_body), inp.r_m);
            let sun = direct * s.normal.dot(inp.sun_body).max(0.0);
            q[s.node] += s.alpha * s.area * (sun + albedo * f_earth) + EPS * s.area * inp.earth_ir_w_m2 * f_earth;
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
        // A boundary conductance shortens the panels' time constants
        let g_b = inp.boundary_w_k.max(0.0) / 4.0;
        let dt_max = if g_b > 0.0 { net.dt_max.min(0.5 * CAPACITY_J_K[PX] / (4.0 * g_b + 1.5)) } else { net.dt_max };
        let n = ((dt / dt_max).ceil() as u32).max(1);
        let h = dt / f64::from(n);
        let sink4 = inp.sink_k.max(0.0).powi(4);
        let f_sink = inp.sink_eps_factor.clamp(0.0, 1.0);
        for _ in 0..n {
            let mut q = q_ext;
            let t = &self.temp_k;
            for s in &net.surf[..net.n_surf] {
                q[s.node] -= SIGMA * s.eps_area * f_sink * (t[s.node].powi(4) - sink4);
            }
            if g_b > 0.0 {
                for p in [PX, MX, PY, MY] {
                    q[p] -= g_b * (t[p] - inp.boundary_k);
                }
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
                Quat::from_axis_angle(spin_axis, sqso_fsw::adcs::SPIN_RATE * t)
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

    // ---------------------------------------------------------------------
    // CDR design cases (`docs/environment/THERMAL_ANALYSIS.md` v2.0; run by
    // `software/simulation/cdr/power_thermal/thermal_cases.py`):
    //   cargo test -p sqso-sil --release cdr_thermal_cases -- --ignored --nocapture
    // Each case prints `CDR|case|node|min|mean|max` lines for its last orbit.

    /// Attitude of a design case.
    #[derive(Clone, Copy)]
    enum Att {
        /// +Y on the Sun, spinning about +Y at the flight spin rate
        SpinPointed,
        /// Two-axis tumble (about 2°/s and 3.3°/s on oblique axes)
        Tumble,
        /// Detumble start: 10°/s about an oblique axis (tip-off)
        TipOff,
    }

    /// Internal dissipation by mode (W per node), from the mission
    /// simulation's device models (`mission.rs`, `hw/radio.rs`,
    /// `hw/eps.rs`): OBC 0.165, ADCS sensors 0.12, magnetorquers up to 0.30,
    /// receiver 0.15, PA 1.5 W DC at 0.5 W RF and 2.7 W DC at 1 W RF, EPS
    /// quiescent 0.08, bucks 0.85/0.90, MPPT 0.92, GNSS 0.125.
    #[derive(Clone, Copy)]
    enum Mode {
        /// SAFE: OBC, receiver, beacon every 180 s at 0.5 W RF; ADCS off
        Safe,
        /// EMERGENCY: SAFE with the beacon shed
        Emergency,
        /// LEOP: OBC, ADCS detumbling at full dipole, receiver
        Leop,
        /// ACTIVE hot: ADCS, GNSS on, 1 W RF, transponder keyed 25 % of
        /// sunlight, beacon every 60 s, Jetson bursts per `JetLoad`
        ActiveHot(JetLoad),
    }

    #[derive(Clone, Copy)]
    enum JetLoad {
        /// 120 s at 8 W every 1200 s in sunlight (10 % duty, BASELINE §5/§6)
        Bursts,
        /// One 600 s batch at 8 W per orbit (the simulation's payload
        /// schedule allows 10 min)
        Batch600,
        /// Rail stuck on at 8 W, sunlit and eclipse (both series switches
        /// failed closed)
        StuckOn,
    }

    fn mode_loads(mode: Mode, sunlit: bool, t_sun: f64) -> ([f64; N_NODES], f64) {
        // returns (dissipation per node, RF power radiated by the antenna)
        let mut q = [0.0; N_NODES];
        let beacon = |period: f64| (0.31 + 200.0 * 8.0 * 1.02 / 1200.0) / period;
        match mode {
            Mode::Safe | Mode::Emergency => {
                let b = if matches!(mode, Mode::Safe) { beacon(180.0) } else { 0.0 };
                q[OBC] = 0.165 * (1.0 / 0.85) + 0.08;
                q[RF] = 0.15 + b * 1.0;
                return (q, b * 0.5);
            }
            Mode::Leop => {
                q[OBC] = (0.165 + 0.12 + 0.30) / 0.85 + 0.08;
                q[RF] = 0.15;
            }
            Mode::ActiveHot(jet) => {
                let b = beacon(60.0);
                let xp = if sunlit { 0.25 } else { 0.0 };
                let pa_dc = 2.7 * (b + xp);
                let jw = if matches!(jet, JetLoad::StuckOn) {
                    8.0
                } else if sunlit {
                    match jet {
                        JetLoad::Bursts => {
                            if t_sun.rem_euclid(1200.0) < 120.0 {
                                8.0
                            } else {
                                0.0
                            }
                        }
                        JetLoad::Batch600 => {
                            if (600.0..1200.0).contains(&t_sun) {
                                8.0
                            } else {
                                0.0
                            }
                        }
                        JetLoad::StuckOn => 8.0,
                    }
                } else {
                    0.0
                };
                let w3 = 0.165 + 0.12 + 0.15 + 0.125;
                q[OBC] = w3 / 0.85 + 0.08 + (pa_dc + jw) * (1.0 / 0.90 - 1.0);
                q[RF] = 0.15 + pa_dc - (b + xp) * 1.0;
                q[JET] = jw;
                return (q, (b + xp) * 1.0);
            }
        }
        (q, 0.0)
    }

    struct Case {
        name: &'static str,
        t0: f64,
        ltan: f64,
        flux: Option<f64>,
        albedo: f64,
        ir: f64,
        att: Att,
        mode: Mode,
        heater_w: f64,
        heater_stuck: bool,
        /// Heater thermostat (on below, off above), °C
        setpoints: (f64, f64),
        params: ThermalParams,
        wings: [bool; 2],
        years: f64,
        orbits: f64,
        init: Option<[f64; N_NODES]>,
        init_c: f64,
    }

    impl Case {
        fn new(name: &'static str, t0: f64, ltan: f64, att: Att, mode: Mode) -> Case {
            Case {
                name,
                t0,
                ltan,
                flux: None,
                albedo: ALBEDO,
                ir: EARTH_IR_W_M2,
                att,
                mode,
                heater_w: 1.0,
                heater_stuck: false,
                setpoints: (sqso_fsw::heater::ON_BELOW_C, sqso_fsw::heater::OFF_ABOVE_C),
                params: ThermalParams::default(),
                wings: [true, true],
                years: 2.0,
                orbits: 25.0,
                init: None,
                init_c: 10.0,
            }
        }
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

    /// Runs a case; prints its last orbit; returns the final temperatures.
    fn run_case(c: &Case) -> [f64; N_NODES] {
        let o = Orbit::sso(500e3, c.ltan, c.t0);
        let array = Array::new();
        let mut th = Thermal::with_params(c.init_c, &c.params);
        if let Some(init) = c.init {
            th.temp_k = init;
        }
        let dt = 5.0;
        let p = o.period();
        let mut heater = false;
        let mut sun_since = 0.0;
        let (mut tmin, mut tmax, mut tsum) = ([f64::INFINITY; N_NODES], [f64::NEG_INFINITY; N_NODES], [0.0; N_NODES]);
        let (mut n, mut heater_on, mut ecl, mut beta) = (0.0, 0.0, 0.0, 0.0);
        let mut t_above_0 = if th.battery_c() >= 0.0 { Some(0.0) } else { None };
        let mut t = 0.0;
        while t < c.orbits * p {
            let tt = c.t0 + t;
            let (r, _) = o.state_at(tt);
            let (s, d) = sun_eci(tt);
            let q = attitude(c.att, &o, s, t);
            let illum = illumination(r, s, d);
            let sun_b = q.rotate_inv(s);
            let nadir_b = q.rotate_inv(-r.unit());
            let flux = c.flux.unwrap_or_else(|| solar_flux(d));
            let sunlit = illum > 0.5;
            if !sunlit {
                sun_since = t;
            }
            let mut inp = ThermalInputs::new(sun_b, illum, flux, nadir_b, r.norm(), r.unit().dot(s));
            inp.albedo = c.albedo;
            inp.earth_ir_w_m2 = c.ir;
            inp.wings_deployed = c.wings;
            let strings = array.strings(&ArrayInputs {
                sun_body: sun_b,
                illum,
                flux,
                nadir_body: nadir_b,
                albedo_w_m2: 0.0,
                r_m: r.norm(),
                temp_c: th.string_temps_c(),
                years_in_orbit: c.years,
                wings_deployed: c.wings,
            });
            let (mut qi, rf_out) = mode_loads(c.mode, sunlit, (t - sun_since).rem_euclid(p));
            let tb = th.battery_c();
            if c.heater_stuck || tb < c.setpoints.0 {
                heater = true;
            } else if tb > c.setpoints.1 {
                heater = false;
            }
            if heater {
                qi[BAT] += c.heater_w;
            }
            inp.internal_w = qi;
            // the array supplies the loads in sunlight (pack full); the
            // conversion losses are already in the OBC node
            inp.elec_w = elec_from_strings(&strings);
            let harvested: f64 = inp.elec_w.iter().sum();
            let demand: f64 = qi.iter().sum::<f64>() + rf_out;
            if harvested > demand && harvested > 0.0 {
                let k = demand / harvested;
                inp.elec_w.iter_mut().for_each(|e| *e *= k);
            }
            th.step(dt, &inp);
            if t_above_0.is_none() && th.battery_c() >= 0.0 {
                t_above_0 = Some(t);
            }
            if t > (c.orbits - 1.0) * p {
                for k in 0..N_NODES {
                    let v = th.temp_c(k);
                    tmin[k] = tmin[k].min(v);
                    tmax[k] = tmax[k].max(v);
                    tsum[k] += v;
                }
                n += 1.0;
                if heater {
                    heater_on += 1.0;
                }
                if !sunlit {
                    ecl += dt;
                }
                beta = o.beta(s).to_degrees();
            }
            t += dt;
        }
        for k in 0..N_NODES {
            println!("CDR|{}|{}|{:.1}|{:.1}|{:.1}", c.name, NODE_NAMES[k], tmin[k], tsum[k] / n, tmax[k]);
        }
        println!("CDR|{}|heater_duty|{:.3}|{:.3}|{:.3}", c.name, heater_on / n, heater_on / n, heater_on / n);
        println!("CDR|{}|eclipse_min|{:.1}|{:.1}|{:.1}", c.name, ecl / 60.0, ecl / 60.0, ecl / 60.0);
        println!("CDR|{}|beta_deg|{:.1}|{:.1}|{:.1}", c.name, beta, beta, beta);
        let up = t_above_0.map_or(-1.0, |t| t / 60.0);
        println!("CDR|{}|battery_above_0C_min|{up:.1}|{up:.1}|{up:.1}", c.name);
        th.temp_k
    }

    /// Powered-off soak in the dispenser after lift-off: the dispenser wall
    /// follows the SpaceX bounding interface temperature (Rideshare Payload
    /// User's Guide v11, Table 4-11), held at its plateau to `end_s`; the
    /// panels see it through ε_eff `eps_eff` and the rails at `rail_w_k`.
    fn dispenser_soak(name: &str, hot: bool, start_c: f64, eps_eff: f64, rail_w_k: f64, end_s: f64) -> [f64; N_NODES] {
        let tab_t = [0.0, 1000.0, 2000.0, 3000.0, 7200.0, 1.0e6];
        let tab_hot = [40.0, 42.0, 55.0, 69.0, 69.0, 69.0];
        let tab_cold = [-5.0, -10.0, -15.0, -20.0, -20.0, -20.0];
        let mut th = Thermal::new(start_c);
        let dt = 5.0;
        let mut t = 0.0;
        let mut marks = vec![3600.0, 7200.0, 10_800.0, end_s];
        marks.dedup();
        while t < end_s + 0.5 * dt {
            let wall = crate::env::math::interp(&tab_t, if hot { &tab_hot } else { &tab_cold }, t) + T0_K;
            let mut inp = ThermalInputs::new(V3::Y, 0.0, 0.0, -V3::Y, 1e12, 0.0);
            inp.wings_deployed = [false, false];
            inp.albedo = 0.0;
            inp.earth_ir_w_m2 = 0.0;
            inp.sink_k = wall;
            inp.sink_eps_factor = eps_eff / EPS;
            inp.boundary_w_k = rail_w_k;
            inp.boundary_k = wall;
            th.step(dt, &inp);
            t += dt;
            if let Some(&m) = marks.first()
                && t >= m
            {
                marks.remove(0);
                println!(
                    "CDR|{name}|t={:.0}s|wall {:.1}|battery {:.1}|OBC {:.1}|RF {:.1}|Jetson {:.1}|+X {:.1}",
                    t,
                    wall - T0_K,
                    th.battery_c(),
                    th.obc_c(),
                    th.rf_c(),
                    th.jetson_c(),
                    th.temp_c(PX)
                );
            }
        }
        th.temp_k
    }

    #[test]
    #[ignore = "CDR analysis cases; run with --ignored --nocapture"]
    fn cdr_thermal_cases() {
        let equinox = from_utc(2029, 3, 20, 0, 0, 0.0);
        let june = from_utc(2029, 6, 21, 0, 0, 0.0);
        let jan = from_utc(2029, 1, 3, 0, 0, 0.0);
        let hot_env = |c: &mut Case| {
            c.albedo = 0.35;
            c.ir = 258.0;
        };
        let cold_env = |c: &mut Case| {
            c.flux = Some(1322.0);
            c.albedo = 0.25;
            c.ir = 218.0;
        };
        let mut cases = Vec::new();
        // Hot: BASELINE §8 bounding definition (no eclipse, 1414 W/m²,
        // maximum power), and the hottest real orbit (LTAN 21:30, the −1 h
        // dispersion, β ≈ 37° in early January at perihelion)
        for (name, jet, g) in [
            ("H1 hot bounding, no eclipse, bursts, 0.44 W/K", JetLoad::Bursts, 0.44),
            ("H2 hot bounding, no eclipse, 600 s batch, 0.44 W/K", JetLoad::Batch600, 0.44),
            ("H3 hot bounding, no eclipse, 600 s batch, 1.0 W/K", JetLoad::Batch600, 1.0),
            ("H4 hot bounding, no eclipse, bursts, 1.0 W/K", JetLoad::Bursts, 1.0),
        ] {
            let mut c = Case::new(name, june, 6.0, Att::SpinPointed, Mode::ActiveHot(jet));
            hot_env(&mut c);
            c.flux = Some(1414.0);
            c.params.jetson_w_k = g;
            c.years = 0.0;
            cases.push(c);
        }
        let mut c = Case::new(
            "H5 hot real, LTAN 21:30 Jan, bursts, 0.44 W/K",
            jan,
            21.5,
            Att::SpinPointed,
            Mode::ActiveHot(JetLoad::Bursts),
        );
        hot_env(&mut c);
        c.years = 0.0;
        cases.push(c);
        for (name, g) in [
            ("H7 hot bounding, Jetson stuck on 8 W, 0.44 W/K", 0.44),
            ("H8 hot bounding, Jetson stuck on 8 W, 1.0 W/K", 1.0),
        ] {
            let mut c = Case::new(name, june, 6.0, Att::SpinPointed, Mode::ActiveHot(JetLoad::StuckOn));
            hot_env(&mut c);
            c.flux = Some(1414.0);
            c.params.jetson_w_k = g;
            c.years = 0.0;
            c.orbits = 40.0;
            cases.push(c);
        }
        let mut c = Case::new(
            "H9 hot bounding, heater stuck on 1.37 W (16.4 V), no cutout",
            june,
            6.0,
            Att::SpinPointed,
            Mode::ActiveHot(JetLoad::Bursts),
        );
        hot_env(&mut c);
        c.flux = Some(1414.0);
        c.heater_w = 1.37;
        c.heater_stuck = true;
        c.years = 0.0;
        c.orbits = 40.0;
        cases.push(c);
        let mut c = Case::new(
            "H6 hot bounding, black battery (eps 0.85)",
            june,
            6.0,
            Att::SpinPointed,
            Mode::ActiveHot(JetLoad::Bursts),
        );
        hot_env(&mut c);
        c.flux = Some(1414.0);
        c.params.battery_eps = 0.85;
        c.years = 0.0;
        cases.push(c);
        // Cold: β = 0, cold environment, SAFE (ADCS off: tumbling), heater
        // 1 W (design) and 0.5 W (the simulation's value); EMERGENCY;
        // wings stowed; a black pack; the real coldest orbit (LTAN 23:30,
        // β ≈ 4° at the June solstice)
        let cold = |name: &'static str, att: Att, mode: Mode, heater: f64| {
            let mut c = Case::new(name, equinox, 12.0, att, mode);
            cold_env(&mut c);
            c.heater_w = heater;
            c
        };
        cases.push(cold("C1 cold, beta 0, SAFE tumbling, heater 1.0 W", Att::Tumble, Mode::Safe, 1.0));
        cases.push(cold("C2 cold, beta 0, SAFE tumbling, heater 0.72 W (12.0 V)", Att::Tumble, Mode::Safe, 0.72));
        cases.push(cold("C3 cold, beta 0, SAFE tumbling, heater 0.5 W", Att::Tumble, Mode::Safe, 0.5));
        cases.push(cold("C4 cold, beta 0, SAFE spin-pointed, heater 1.0 W", Att::SpinPointed, Mode::Safe, 1.0));
        cases.push(cold("C5 cold, beta 0, EMERGENCY tumbling, heater 0.72 W", Att::Tumble, Mode::Emergency, 0.72));
        let mut c = cold("C6 cold, beta 0, SAFE tumbling, wings stowed, 0.72 W", Att::Tumble, Mode::Safe, 0.72);
        c.wings = [false, false];
        cases.push(c);
        let mut c = cold("C7 cold, beta 0, SAFE tumbling, black battery, 0.72 W", Att::Tumble, Mode::Safe, 0.72);
        c.params.battery_eps = 0.85;
        cases.push(c);
        let mut c = cold("C8 cold, beta 0, SAFE tumbling, mount 0.10 W/K, 0.72 W", Att::Tumble, Mode::Safe, 0.72);
        c.params.battery_mount_w_k = 0.10;
        cases.push(c);
        cases.push(cold("C9 cold, beta 0, SAFE spin-pointed, heater failed off", Att::SpinPointed, Mode::Safe, 0.0));
        cases.push(cold(
            "C11 cold, beta 0, SAFE spin-pointed, heater 0.72 W (12.0 V)",
            Att::SpinPointed,
            Mode::Safe,
            0.72,
        ));
        cases.push(cold("C12 cold, beta 0, SAFE spin-pointed, heater 0.5 W", Att::SpinPointed, Mode::Safe, 0.5));
        cases.push(cold(
            "C13 cold, beta 0, EMERGENCY spin-pointed, heater 0.72 W",
            Att::SpinPointed,
            Mode::Emergency,
            0.72,
        ));
        let mut c =
            cold("C14 cold, beta 0, SAFE spin-pointed, wings stowed, 0.72 W", Att::SpinPointed, Mode::Safe, 0.72);
        c.wings = [false, false];
        cases.push(c);
        let mut c =
            cold("C15 cold, beta 0, SAFE spin-pointed, mount 0.10 W/K, 0.72 W", Att::SpinPointed, Mode::Safe, 0.72);
        c.params.battery_mount_w_k = 0.10;
        cases.push(c);
        for (name, w) in [
            ("C16 cold, beta 0, SAFE spin-pointed, wings stowed, 0.95 W (14.0 V)", 0.95),
            ("C17 cold, beta 0, SAFE spin-pointed, wings stowed, 1.09 W (180 ohm, 14.0 V)", 1.09),
        ] {
            let mut c = cold(name, Att::SpinPointed, Mode::Safe, w);
            c.wings = [false, false];
            cases.push(c);
        }
        for (name, w, wings, g) in [
            ("C20 cold, beta 0, wings stowed, 0.95 W, set points +5/+9 C", 0.95, false, 0.05),
            ("C21 cold, beta 0, mount 0.10 W/K, 0.95 W, set points +5/+9 C", 0.95, true, 0.10),
            ("C22 cold, beta 0, nominal, 0.95 W, set points +5/+9 C", 0.95, true, 0.05),
            ("C23 cold, beta 0, nominal mount 0.03 W/K, 0.95 W, set points +5/+9 C", 0.95, true, 0.03),
        ] {
            let mut c = cold(name, Att::SpinPointed, Mode::Safe, w);
            c.wings = [wings, wings];
            c.params.battery_mount_w_k = g;
            c.setpoints = (5.0, 9.0);
            cases.push(c);
        }
        for (name, att) in [
            ("C24 cold, beta 0, SAFE tumbling, 1.30 W (16.4 V), set points +5/+9 C", Att::Tumble),
            ("C25 cold, beta 0, SAFE spin-pointed, 1.30 W (16.4 V), set points +5/+9 C", Att::SpinPointed),
        ] {
            let mut c = cold(name, att, Mode::Safe, 1.30);
            c.setpoints = (5.0, 9.0);
            cases.push(c);
        }
        for (name, w) in [
            ("C18 cold, beta 0, SAFE spin-pointed, mount 0.10 W/K, 0.95 W", 0.95),
            ("C19 cold, beta 0, SAFE spin-pointed, mount 0.10 W/K, 1.09 W", 1.09),
        ] {
            let mut c = cold(name, Att::SpinPointed, Mode::Safe, w);
            c.params.battery_mount_w_k = 0.10;
            cases.push(c);
        }
        let mut c =
            Case::new("C10 cold real, LTAN 23:30 June, SAFE tumbling, 0.72 W", june, 23.5, Att::Tumble, Mode::Safe);
        c.albedo = 0.25;
        c.ir = 218.0;
        c.heater_w = 0.72;
        cases.push(c);
        for c in &cases {
            let _ = run_case(c);
        }
        // Dispenser soak (powered off) and the cold start that follows
        let _ = dispenser_soak("D1 dispenser hot soak", true, 25.0, 0.70, 0.5, 10_800.0);
        let _ = dispenser_soak("D2 dispenser cold soak", false, 15.0, 0.70, 0.5, 10_800.0);
        let cold_start = dispenser_soak("D3 dispenser cold soak to 2 h", false, 15.0, 0.70, 0.5, 7200.0);
        for (name, heater) in
            [("L1 LEOP from cold soak, heater 1.0 W", 1.0), ("L2 LEOP from cold soak, heater 0.5 W", 0.5)]
        {
            let mut c = cold(name, Att::TipOff, Mode::Leop, heater);
            c.init = Some(cold_start);
            c.orbits = 1.0;
            c.wings = [true, true];
            let _ = run_case(&c);
        }
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
