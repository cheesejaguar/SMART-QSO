//! Rigid-body attitude dynamics and the environmental torques.
//!
//! State: `q` (body → ECI, see [`Quat`]) and the body rate `w` (rad/s), with
//! a diagonal principal inertia. [`Attitude::step`] integrates Euler's
//! equations I ω̇ = τ − ω × (I ω) and q̇ = ½ q ⊗ ω with classical RK4,
//! sub-stepping so that no sub-step turns more than 0.1 rad.
//!
//! # Mass properties (assumptions)
//!
//! * The inertia is the CAD's (`docs/environment/MASS_BUDGET.md` v3.1 §6.1
//!   and §6.5; four AntS elements, the 2 × 10 g ballast of decision C-13,
//!   the harness): the mean of its CBE (1,513.9 g) and MEV (1,747.8 g)
//!   rows, stowed ([`INERTIA_STOWED`]) and deployed ([`INERTIA_DEPLOYED`],
//!   the flight software's `INERTIA`). The products of inertia (at most
//!   1.7 × 10⁻⁵ deployed, 3.8 × 10⁻⁵ stowed) are left out: the principal
//!   axes are within about 1° of the body axes deployed (the major axis
//!   1.0° from +Y) and 6–7° stowed.
//! * Mass 1.6 kg for the drag (the lifetime-analysis mass, BASELINE §4);
//!   each wing 55 g (`docs/environment/MASS_BUDGET.md` §4.2: substrate
//!   24.7 g, four cells 14.4 g, hinge and release 16 g), a thin 83 × 98 mm
//!   plate hinged on the ±X/+Y edge and deployed into the plane of the +Y
//!   face, centred at (±100, +50, 0) mm, which moves the CG for the
//!   aerodynamic torque.
//!
//! # Disturbances (assumptions)
//!
//! * Residual magnetic dipole 3 mA·m² (typical of 1U CubeSats with air-core
//!   torquers: harness and battery current loops), along a fixed body
//!   direction.
//! * Aerodynamic torque from the flat-plate projected area of the body faces
//!   and both wings, with the centre of pressure as the area-weighted
//!   centroid of the plates facing the flow, plus a 5 mm CG offset per axis
//!   from manufacturing (CDS allows 2 cm).
//!
//! Initial condition: tumbling at 10°/s about a random axis (BASELINE §8:
//! "B-dot detumble from 10°/s within 3 orbits").

use super::math::{MU_EARTH, Quat, Rng, V3};

/// Total mass used for the mass properties (kg).
pub const MASS_KG: f64 = 1.6;
/// Mass of one wing (kg).
pub const WING_MASS_KG: f64 = 0.055;
/// Body edge lengths (m): X, Y, Z.
pub const BODY_DIMS_M: V3 = V3::new(0.100, 0.100, 0.1135);
/// Wing panel extent across (along X when deployed) and along Z (m).
pub const WING_PANEL_M: (f64, f64) = (0.083, 0.098);
/// Wing area for drag (m²): one 1U face, as the BASELINE §3 drag area
/// (0.028 m² tumbling-averaged, wings deployed) assumes.
pub const WING_DRAG_AREA_M2: f64 = 0.100 * 0.1135;
/// Initial tumble rate (rad/s): 10°/s (BASELINE §8).
pub const INITIAL_TUMBLE_RAD_S: f64 = 10.0 * core::f64::consts::PI / 180.0;
/// Largest rotation per RK4 sub-step (rad).
const MAX_SUBSTEP_ANGLE: f64 = 0.1;

/// Inertia about the CG, wings and antennas stowed (kg·m², body axes): the
/// mean of the CAD's CBE and MEV rows (MASS_BUDGET §6.1, final CAD of CDR
/// round 3)
pub const INERTIA_STOWED: V3 = V3::new(2.960e-3, 3.292e-3, 2.970e-3);
/// Inertia about the CG, wings and antennas deployed (kg·m², body axes):
/// the mean of the CAD's CBE (2.853, 4.200, 3.978) and MEV (3.276, 4.758,
/// 4.545) × 10⁻³ (MASS_BUDGET §6.1 and §6.5, with the C-13 ballast, final
/// CAD of CDR round 3); Iyy/Izz 1.051. The same diagonal as
/// `sqso_fsw::adcs::INERTIA`
pub const INERTIA_DEPLOYED: V3 = V3::new(3.0645e-3, 4.479e-3, 4.2615e-3);
/// One wing deployed (W1 or W2) and the AntS out: the diagonal of the
/// CAD's one-wing tensors (STR-27; their products tilt the major axis 8.3°
/// and 9.1° from +Y, which this principal-axis model leaves out)
pub const INERTIA_ONE_WING: [V3; 2] = [V3::new(3.027e-3, 4.165e-3, 3.910e-3), V3::new(3.027e-3, 4.166e-3, 3.911e-3)];

/// Principal inertia, wings stowed (kg·m²).
pub fn inertia_stowed() -> V3 {
    INERTIA_STOWED
}

/// CG position (m) in the geometric body frame, wings deployed.
pub fn cg_deployed() -> V3 {
    V3::new(0.0, 2.0 * WING_MASS_KG * 0.05 / MASS_KG, 0.0)
}

/// Principal inertia about the CG, wings deployed (kg·m²).
pub fn inertia_deployed() -> V3 {
    INERTIA_DEPLOYED
}

/// Rigid-body attitude state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Attitude {
    /// Attitude quaternion, body → ECI.
    pub q: Quat,
    /// Body angular rate (rad/s, body frame).
    pub w: V3,
    /// Principal moments of inertia (kg·m², body axes).
    pub inertia: V3,
}

impl Attitude {
    /// A state from its parts.
    pub fn new(q: Quat, w: V3, inertia: V3) -> Attitude {
        Attitude { q: q.normalize(), w, inertia }
    }

    /// Random orientation tumbling at `rate` (rad/s) about a random body
    /// axis.
    pub fn tumbling(rng: &mut Rng, rate: f64, inertia: V3) -> Attitude {
        Attitude::new(rng.quat(), rng.unit_vector() * rate, inertia)
    }

    /// Angular acceleration from Euler's equations (rad/s²).
    fn wdot(&self, w: V3, torque: V3) -> V3 {
        let i = self.inertia;
        let h = w.hadamard(i);
        let rhs = torque - w.cross(h);
        V3::new(rhs.x / i.x, rhs.y / i.y, rhs.z / i.z)
    }

    /// Advance by `dt` seconds under a body-frame torque (N·m) held constant
    /// over the step (RK4, sub-stepped).
    pub fn step(&mut self, dt: f64, torque_body: V3) {
        if dt <= 0.0 {
            return;
        }
        let n = ((self.w.norm() * dt / MAX_SUBSTEP_ANGLE).ceil() as u32).max(1);
        let h = dt / f64::from(n);
        for _ in 0..n {
            self.rk4(h, torque_body);
        }
        self.q = self.q.normalize();
    }

    fn rk4(&mut self, h: f64, tau: V3) {
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
        let q = q0.add_scaled(k1q, h / 6.0).add_scaled(k2q, h / 3.0).add_scaled(k3q, h / 3.0).add_scaled(k4q, h / 6.0);
        self.q = q.normalize();
    }

    /// Rotational kinetic energy (J).
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.w.dot(self.w.hadamard(self.inertia))
    }

    /// Angular momentum in ECI (N·m·s).
    pub fn momentum_eci(&self) -> V3 {
        self.q.rotate(self.w.hadamard(self.inertia))
    }

    /// An ECI vector in the body frame.
    pub fn to_body(&self, v_eci: V3) -> V3 {
        self.q.rotate_inv(v_eci)
    }

    /// A body vector in ECI.
    pub fn to_eci(&self, v_body: V3) -> V3 {
        self.q.rotate(v_body)
    }
}

/// Torque (N·m) of a magnetic dipole `dipole_body` (A·m²) in the field
/// `b_body` (T): m × B.
pub fn magnetic_torque(dipole_body: V3, b_body: V3) -> V3 {
    dipole_body.cross(b_body)
}

/// Gravity-gradient torque (N·m) at ECI position `r_eci` (m) for attitude
/// `q` and principal inertia `inertia`: 3μ/r³ (n̂ × I n̂), n̂ the position
/// direction in the body frame.
pub fn gravity_gradient(r_eci: V3, q: Quat, inertia: V3) -> V3 {
    let r = r_eci.norm();
    let n = q.rotate_inv(r_eci / r);
    n.cross(n.hadamard(inertia)) * (3.0 * MU_EARTH / (r * r * r))
}

/// One flat plate for the aerodynamic model: outward normal, centroid in the
/// geometric body frame (m), area (m²).
#[derive(Clone, Copy, Debug)]
struct Plate {
    normal: V3,
    centroid: V3,
    area: f64,
    two_sided: bool,
}

const fn body_plates() -> [Plate; 6] {
    let ax = 0.100 * 0.1135;
    let az = 0.100 * 0.100;
    [
        Plate { normal: V3::X, centroid: V3::new(0.05, 0.0, 0.0), area: ax, two_sided: false },
        Plate { normal: V3::new(-1.0, 0.0, 0.0), centroid: V3::new(-0.05, 0.0, 0.0), area: ax, two_sided: false },
        Plate { normal: V3::Y, centroid: V3::new(0.0, 0.05, 0.0), area: ax, two_sided: false },
        Plate { normal: V3::new(0.0, -1.0, 0.0), centroid: V3::new(0.0, -0.05, 0.0), area: ax, two_sided: false },
        Plate { normal: V3::Z, centroid: V3::new(0.0, 0.0, 0.05675), area: az, two_sided: false },
        Plate { normal: V3::new(0.0, 0.0, -1.0), centroid: V3::new(0.0, 0.0, -0.05675), area: az, two_sided: false },
    ]
}

const WING_PLATES: [Plate; 2] = [
    Plate { normal: V3::Y, centroid: V3::new(0.10, 0.05, 0.0), area: WING_DRAG_AREA_M2, two_sided: true },
    Plate { normal: V3::Y, centroid: V3::new(-0.10, 0.05, 0.0), area: WING_DRAG_AREA_M2, two_sided: true },
];

/// Projected area (m²) and centre of pressure (m, geometric body frame) for
/// a flow arriving along the unit body vector `-v_hat` (spacecraft moving
/// along `v_hat`).
fn projected(v_hat: V3, wings: [bool; 2]) -> (f64, V3) {
    let mut a = 0.0;
    let mut m = V3::ZERO;
    let mut add = |p: &Plate| {
        let c = p.normal.dot(v_hat);
        let f = if p.two_sided { c.abs() } else { c.max(0.0) };
        a += p.area * f;
        m += p.centroid * (p.area * f);
    };
    body_plates().iter().for_each(&mut add);
    for (k, p) in WING_PLATES.iter().enumerate() {
        if wings[k] {
            add(p);
        }
    }
    if a > 0.0 { (a, m / a) } else { (0.0, V3::ZERO) }
}

/// Drag-projected area (m²) for motion along the body direction `v_body`
/// (any length), with each wing deployed or stowed (stowed wings lie on the
/// body and add nothing). Tumbling average: 0.0164 m² stowed, 0.028 m²
/// deployed (BASELINE §3).
pub fn projected_area(v_body: V3, wings_deployed: [bool; 2]) -> f64 {
    projected(v_body.unit(), wings_deployed).0
}

/// Environmental-torque parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Disturbance {
    /// Residual magnetic dipole (A·m², body frame).
    pub residual_dipole: V3,
    /// CG offset from the geometric centre (m, body frame) on top of the
    /// wing-induced shift.
    pub cg_offset: V3,
    /// Drag coefficient.
    pub cd: f64,
}

impl Default for Disturbance {
    fn default() -> Disturbance {
        Disturbance::nominal()
    }
}

impl Disturbance {
    /// The documented assumptions: 3 mA·m² residual dipole, 5 mm CG offset
    /// per axis, Cd 2.2.
    pub fn nominal() -> Disturbance {
        Disturbance {
            residual_dipole: V3::new(1.0, -1.0, 1.0).unit() * 3e-3,
            cg_offset: V3::new(0.005, 0.005, 0.005),
            cd: 2.2,
        }
    }

    /// Torque (N·m) of the residual dipole in the body field `b_body` (T).
    pub fn residual_torque(&self, b_body: V3) -> V3 {
        magnetic_torque(self.residual_dipole, b_body)
    }

    /// Aerodynamic torque (N·m) for the velocity relative to the atmosphere
    /// `v_rel_body` (m/s, body frame) and density (kg/m³).
    pub fn aero_torque(&self, v_rel_body: V3, density: f64, wings_deployed: [bool; 2]) -> V3 {
        let v = v_rel_body.norm();
        if v == 0.0 {
            return V3::ZERO;
        }
        let v_hat = v_rel_body / v;
        let (area, cp) = projected(v_hat, wings_deployed);
        let force = v_hat * (-0.5 * density * self.cd * area * v * v);
        let n_deployed = wings_deployed.iter().filter(|&&d| d).count() as f64;
        let cg = cg_deployed() * (n_deployed / 2.0) + self.cg_offset;
        (cp - cg).cross(force)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::magfield::field_eci;
    use crate::env::orbit::Orbit;
    use crate::env::time::from_utc;

    #[test]
    fn inertia_values() {
        let s = inertia_stowed();
        let d = inertia_deployed();
        assert!(d.x > s.x && d.y > d.x && d.z > s.z);
        // The flight software's model is the same (sqso_fsw::adcs::INERTIA)
        let f = sqso_fsw::adcs::INERTIA;
        assert!((d.x - f.x).abs() < 1e-12 && (d.y - f.y).abs() < 1e-12 && (d.z - f.z).abs() < 1e-12);
        assert!((d.y / d.z - 1.051).abs() < 0.001, "Iyy/Izz {}", d.y / d.z);
    }

    #[test]
    fn torque_free_motion_conserves_energy_and_momentum() {
        let mut rng = Rng::new(7);
        let mut a = Attitude::tumbling(&mut rng, INITIAL_TUMBLE_RAD_S, inertia_deployed());
        let (e0, h0) = (a.kinetic_energy(), a.momentum_eci());
        for _ in 0..6000 {
            a.step(1.0, V3::ZERO);
        }
        assert!((a.kinetic_energy() / e0 - 1.0).abs() < 1e-6);
        assert!((a.momentum_eci() - h0).norm() / h0.norm() < 1e-6);
        assert!((a.q.norm() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn spin_about_principal_axis_is_steady() {
        let mut a = Attitude::new(Quat::identity(), V3::new(0.0, 0.0, 0.2), inertia_deployed());
        a.step(10.0, V3::ZERO);
        assert!((a.w - V3::new(0.0, 0.0, 0.2)).norm() < 1e-12);
        assert!((a.to_eci(V3::X) - V3::new(2f64.cos(), 2f64.sin(), 0.0)).norm() < 1e-6);
    }

    #[test]
    fn torque_formulas() {
        assert_eq!(magnetic_torque(V3::X, V3::Y), V3::Z);
        // gravity gradient vanishes with a principal axis along nadir
        let i = inertia_deployed();
        let r = V3::new(6.878e6, 0.0, 0.0);
        assert!(gravity_gradient(r, Quat::identity(), i).norm() < 1e-20);
        let tilted = Quat::from_axis_angle(V3::Z, 0.3);
        let gg = gravity_gradient(r, tilted, i);
        assert!(gg.norm() > 1e-10 && gg.norm() < 1e-8, "{}", gg.norm());
        // mean projected areas match BASELINE §3
        let mut rng = Rng::new(3);
        let (mut a_dep, mut a_sto) = (0.0, 0.0);
        let n = 100_000;
        for _ in 0..n {
            let v = rng.unit_vector();
            a_dep += projected_area(v, [true, true]);
            a_sto += projected_area(v, [false, false]);
        }
        let (a_dep, a_sto) = (a_dep / f64::from(n), a_sto / f64::from(n));
        println!("mean projected area: deployed {a_dep:.4} m², stowed {a_sto:.4} m²");
        assert!((a_dep - 0.028).abs() < 0.001 && (a_sto - 0.0164).abs() < 0.0005);
        let d = Disturbance::nominal();
        let tq = d.aero_torque(V3::new(7600.0, 0.0, 0.0), 4e-13, [true, true]);
        assert!(tq.norm() > 1e-10 && tq.norm() < 1e-7, "{}", tq.norm());
        assert!(d.residual_torque(V3::new(0.0, 3e-5, 0.0)).norm() < 1e-7);
    }

    /// Proportional B-dot (gain 3e4 A·m²·s/T, saturated at 0.15 A·m² per axis,
    /// BASELINE §8) detumbles the
    /// deployed spacecraft from 10°/s within 3 orbits at 500 km.
    #[test]
    fn bdot_detumbles_within_three_orbits() {
        let t0 = from_utc(2028, 6, 1, 0, 0, 0.0);
        let o = Orbit::sso(500e3, 10.5, t0);
        let mut rng = Rng::new(11);
        let mut a = Attitude::tumbling(&mut rng, INITIAL_TUMBLE_RAD_S, inertia_deployed());
        let mut b_prev = V3::ZERO;
        let steps = (3.0 * o.period()) as u32;
        for k in 0..steps {
            let t = t0 + f64::from(k);
            let (r, _) = o.state_at(t);
            let b = a.to_body(field_eci(r, t));
            let bdot = if k == 0 { V3::ZERO } else { b - b_prev };
            b_prev = b;
            // proportional B-dot, saturated per axis (as sqso-fsw `adcs::bdot`)
            let sat = |x: f64| (-3e4 * x).clamp(-0.15, 0.15);
            let m = V3::new(sat(bdot.x), sat(bdot.y), sat(bdot.z));
            a.step(1.0, magnetic_torque(m, b) + gravity_gradient(r, a.q, a.inertia));
        }
        let deg_s = a.w.norm().to_degrees();
        println!("rate after 3 orbits of B-dot: {deg_s:.3}°/s");
        assert!(deg_s < 1.0, "{deg_s}");
    }
}
