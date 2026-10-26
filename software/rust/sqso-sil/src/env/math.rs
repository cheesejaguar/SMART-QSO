//! Small linear algebra (3-vectors, unit quaternions), a seeded random number
//! generator and physical constants.
//!
//! Everything is `f64`, SI units and `Copy`; nothing allocates.

use core::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub, SubAssign};

/// Earth gravitational parameter μ (m³/s², EGM96/WGS84).
pub const MU_EARTH: f64 = 3.986_004_418e14;
/// Earth equatorial radius (m, WGS84).
pub const R_EARTH: f64 = 6_378_137.0;
/// WGS84 flattening.
pub const F_EARTH: f64 = 1.0 / 298.257_223_563;
/// Second zonal harmonic J2 of the Earth's gravity field.
pub const J2: f64 = 1.082_63e-3;
/// Earth rotation rate (rad/s, sidereal).
pub const OMEGA_EARTH: f64 = 7.292_115_146_7e-5;
/// Astronomical unit (m).
pub const AU: f64 = 1.495_978_707e11;
/// Total solar irradiance at 1 AU (W/m², the mean value used by
/// `hardware/eps/SOLAR_ARRAY_DESIGN.md` §9.1).
pub const SOLAR_CONSTANT: f64 = 1361.0;
/// Stefan–Boltzmann constant (W/m²/K⁴).
pub const SIGMA: f64 = 5.670_374_419e-8;
/// Boltzmann constant (J/K).
pub const K_BOLTZMANN: f64 = 1.380_649e-23;
/// Speed of light (m/s).
pub const C_LIGHT: f64 = 299_792_458.0;
/// Solar radius (m), for the penumbra model.
pub const R_SUN: f64 = 6.957e8;
/// 0 °C in kelvin.
pub const T0_K: f64 = 273.15;
/// Seconds per day.
pub const DAY_S: f64 = 86_400.0;
/// Seconds per Julian year (365.25 days).
pub const YEAR_S: f64 = 365.25 * DAY_S;

/// A 3-vector (m, m/s, T, N·m: whatever the caller stores in it).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
}

impl V3 {
    /// The zero vector.
    pub const ZERO: V3 = V3 { x: 0.0, y: 0.0, z: 0.0 };
    /// Unit vector along +X.
    pub const X: V3 = V3 { x: 1.0, y: 0.0, z: 0.0 };
    /// Unit vector along +Y.
    pub const Y: V3 = V3 { x: 0.0, y: 1.0, z: 0.0 };
    /// Unit vector along +Z.
    pub const Z: V3 = V3 { x: 0.0, y: 0.0, z: 1.0 };

    /// A vector from its components.
    pub const fn new(x: f64, y: f64, z: f64) -> V3 {
        V3 { x, y, z }
    }

    /// Dot product.
    pub fn dot(self, o: V3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Cross product `self × o`.
    pub fn cross(self, o: V3) -> V3 {
        V3::new(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)
    }

    /// Squared Euclidean norm.
    pub fn norm2(self) -> f64 {
        self.dot(self)
    }

    /// Euclidean norm.
    pub fn norm(self) -> f64 {
        self.norm2().sqrt()
    }

    /// Unit vector in the same direction; the zero vector stays zero.
    pub fn unit(self) -> V3 {
        let n = self.norm();
        if n > 0.0 { self * (1.0 / n) } else { V3::ZERO }
    }

    /// Component-wise product (for example a diagonal inertia times ω).
    pub fn hadamard(self, o: V3) -> V3 {
        V3::new(self.x * o.x, self.y * o.y, self.z * o.z)
    }

    /// Angle between two vectors (rad, 0..π); 0 if either is zero.
    pub fn angle(self, o: V3) -> f64 {
        let d = self.norm() * o.norm();
        if d > 0.0 { (self.dot(o) / d).clamp(-1.0, 1.0).acos() } else { 0.0 }
    }

    /// Rotation about +Z by `angle` (rad, right-handed: active rotation).
    pub fn rot_z(self, angle: f64) -> V3 {
        let (s, c) = angle.sin_cos();
        V3::new(c * self.x - s * self.y, s * self.x + c * self.y, self.z)
    }

    /// Rotation about +X by `angle` (rad, active).
    pub fn rot_x(self, angle: f64) -> V3 {
        let (s, c) = angle.sin_cos();
        V3::new(self.x, c * self.y - s * self.z, s * self.y + c * self.z)
    }
}

impl Add for V3 {
    type Output = V3;
    fn add(self, o: V3) -> V3 {
        V3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for V3 {
    type Output = V3;
    fn sub(self, o: V3) -> V3 {
        V3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Neg for V3 {
    type Output = V3;
    fn neg(self) -> V3 {
        V3::new(-self.x, -self.y, -self.z)
    }
}

impl Mul<f64> for V3 {
    type Output = V3;
    fn mul(self, k: f64) -> V3 {
        V3::new(self.x * k, self.y * k, self.z * k)
    }
}

impl Mul<V3> for f64 {
    type Output = V3;
    fn mul(self, v: V3) -> V3 {
        v * self
    }
}

impl Div<f64> for V3 {
    type Output = V3;
    fn div(self, k: f64) -> V3 {
        V3::new(self.x / k, self.y / k, self.z / k)
    }
}

impl AddAssign for V3 {
    fn add_assign(&mut self, o: V3) {
        *self = *self + o;
    }
}

impl SubAssign for V3 {
    fn sub_assign(&mut self, o: V3) {
        *self = *self - o;
    }
}

impl MulAssign<f64> for V3 {
    fn mul_assign(&mut self, k: f64) {
        *self = *self * k;
    }
}

/// A unit quaternion `w + xi + yj + zk` (Hamilton convention).
///
/// Attitude convention used throughout `env`: `q` maps **body to inertial**,
/// so `q.rotate(v_body)` is the same vector in ECI and `q.rotate_inv(v_eci)`
/// is the vector in the body frame (`v_eci = q ⊗ v_body ⊗ q*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    /// Scalar part.
    pub w: f64,
    /// i component.
    pub x: f64,
    /// j component.
    pub y: f64,
    /// k component.
    pub z: f64,
}

impl Default for Quat {
    fn default() -> Quat {
        Quat::identity()
    }
}

impl Quat {
    /// The identity rotation.
    pub const fn identity() -> Quat {
        Quat { w: 1.0, x: 0.0, y: 0.0, z: 0.0 }
    }

    /// Rotation by `angle` (rad) about `axis` (need not be unit; a zero axis
    /// gives the identity).
    pub fn from_axis_angle(axis: V3, angle: f64) -> Quat {
        let u = axis.unit();
        if u == V3::ZERO {
            return Quat::identity();
        }
        let (s, c) = (0.5 * angle).sin_cos();
        Quat { w: c, x: u.x * s, y: u.y * s, z: u.z * s }
    }

    /// Hamilton product `self ⊗ o` (apply `o` first, then `self`); also
    /// available as `self * o`.
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, o: Quat) -> Quat {
        Quat {
            w: self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
            x: self.w * o.x + self.x * o.w + self.y * o.z - self.z * o.y,
            y: self.w * o.y - self.x * o.z + self.y * o.w + self.z * o.x,
            z: self.w * o.z + self.x * o.y - self.y * o.x + self.z * o.w,
        }
    }

    /// Conjugate (the inverse rotation for a unit quaternion).
    pub fn conj(self) -> Quat {
        Quat { w: self.w, x: -self.x, y: -self.y, z: -self.z }
    }

    /// Norm of the quaternion.
    pub fn norm(self) -> f64 {
        (self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// The same rotation with unit norm (identity if the norm is zero).
    pub fn normalize(self) -> Quat {
        let n = self.norm();
        if n > 0.0 {
            let k = 1.0 / n;
            Quat { w: self.w * k, x: self.x * k, y: self.y * k, z: self.z * k }
        } else {
            Quat::identity()
        }
    }

    /// Rotate a vector: body → inertial for an attitude quaternion.
    pub fn rotate(self, v: V3) -> V3 {
        // v' = v + 2w (u × v) + 2 u × (u × v), u = vector part
        let u = V3::new(self.x, self.y, self.z);
        let t = u.cross(v) * 2.0;
        v + t * self.w + u.cross(t)
    }

    /// Inverse rotation: inertial → body for an attitude quaternion.
    pub fn rotate_inv(self, v: V3) -> V3 {
        self.conj().rotate(v)
    }

    /// Advance an attitude by a body rate `omega_body` (rad/s) held constant
    /// for `dt` seconds: `q ← q ⊗ exp(½ ω dt)` (exact for constant ω).
    pub fn integrate(self, omega_body: V3, dt: f64) -> Quat {
        let angle = omega_body.norm() * dt;
        if angle == 0.0 {
            return self;
        }
        self.mul(Quat::from_axis_angle(omega_body, angle)).normalize()
    }

    /// Time derivative `½ q ⊗ (0, ω_body)` (as a non-unit quaternion).
    pub fn derivative(self, omega_body: V3) -> Quat {
        let p = self.mul(Quat { w: 0.0, x: omega_body.x, y: omega_body.y, z: omega_body.z });
        Quat { w: 0.5 * p.w, x: 0.5 * p.x, y: 0.5 * p.y, z: 0.5 * p.z }
    }

    /// `self + k·o`, component-wise (for Runge–Kutta stages).
    pub fn add_scaled(self, o: Quat, k: f64) -> Quat {
        Quat { w: self.w + k * o.w, x: self.x + k * o.x, y: self.y + k * o.y, z: self.z + k * o.z }
    }
}

impl Mul for Quat {
    type Output = Quat;
    fn mul(self, o: Quat) -> Quat {
        Quat::mul(self, o)
    }
}

/// Seeded pseudo-random number generator: xoshiro256** (Blackman and Vigna),
/// seeded through SplitMix64. Deterministic for a given seed on every
/// platform.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    /// A generator from a 64-bit seed.
    pub fn new(seed: u64) -> Rng {
        let mut st = seed;
        let s = [splitmix64(&mut st), splitmix64(&mut st), splitmix64(&mut st), splitmix64(&mut st)];
        Rng { s }
    }

    /// An independent generator derived from this one (for giving each model
    /// its own stream).
    pub fn fork(&mut self) -> Rng {
        Rng::new(self.next_u64())
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let [s0, s1, s2, s3] = &mut self.s;
        let result = s1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = *s1 << 17;
        *s2 ^= *s0;
        *s3 ^= *s1;
        *s1 ^= *s2;
        *s0 ^= *s3;
        *s2 ^= t;
        *s3 = s3.rotate_left(45);
        result
    }

    /// Uniform in [0, 1) with 53 random bits.
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.uniform()
    }

    /// Standard normal deviate (Box–Muller; one draw per call).
    pub fn normal(&mut self) -> f64 {
        let u1 = 1.0 - self.uniform(); // (0, 1]
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (core::f64::consts::TAU * u2).cos()
    }

    /// Exponential deviate with the given rate (mean 1/rate); infinite for a
    /// rate ≤ 0.
    pub fn exp(&mut self, rate: f64) -> f64 {
        if rate <= 0.0 {
            return f64::INFINITY;
        }
        -(1.0 - self.uniform()).ln() / rate
    }

    /// Poisson deviate with mean `lambda` (Knuth's method below 30, a rounded
    /// normal approximation above).
    pub fn poisson(&mut self, lambda: f64) -> u64 {
        if lambda <= 0.0 {
            return 0;
        }
        if lambda < 30.0 {
            let limit = (-lambda).exp();
            let mut k = 0u64;
            let mut p = self.uniform();
            while p > limit {
                k += 1;
                p *= self.uniform();
            }
            k
        } else {
            let x = lambda + lambda.sqrt() * self.normal() + 0.5;
            if x <= 0.0 { 0 } else { x as u64 }
        }
    }

    /// A direction uniformly distributed on the unit sphere.
    pub fn unit_vector(&mut self) -> V3 {
        let z = 2.0 * self.uniform() - 1.0;
        let phi = core::f64::consts::TAU * self.uniform();
        let r = (1.0 - z * z).max(0.0).sqrt();
        V3::new(r * phi.cos(), r * phi.sin(), z)
    }

    /// A rotation uniformly distributed over SO(3) (Shoemake's method).
    pub fn quat(&mut self) -> Quat {
        let (u1, u2, u3) = (self.uniform(), self.uniform(), self.uniform());
        let (a, b) = ((1.0 - u1).sqrt(), u1.sqrt());
        let (t2, t3) = (core::f64::consts::TAU * u2, core::f64::consts::TAU * u3);
        Quat { w: b * t3.cos(), x: a * t2.sin(), y: a * t2.cos(), z: b * t3.sin() }
    }
}

/// Linear interpolation in a table of increasing `xs`, clamped at the ends.
pub fn interp(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    let n = xs.len().min(ys.len());
    if n == 0 {
        return 0.0;
    }
    if x <= xs[0] {
        return ys[0];
    }
    for i in 1..n {
        if x <= xs[i] {
            let f = (x - xs[i - 1]) / (xs[i] - xs[i - 1]);
            return ys[i - 1] + f * (ys[i] - ys[i - 1]);
        }
    }
    ys[n - 1]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: V3, b: V3, tol: f64) -> bool {
        (a - b).norm() < tol
    }

    #[test]
    fn vector_algebra() {
        let a = V3::new(1.0, 2.0, 3.0);
        let b = V3::new(-2.0, 0.5, 4.0);
        assert_eq!(a.dot(b), -2.0 + 1.0 + 12.0);
        let c = a.cross(b);
        assert!(c.dot(a).abs() < 1e-12 && c.dot(b).abs() < 1e-12);
        assert!(close(V3::X.cross(V3::Y), V3::Z, 1e-15));
        assert!((a.unit().norm() - 1.0).abs() < 1e-15);
        assert_eq!(V3::ZERO.unit(), V3::ZERO);
        assert!(close(a - a, V3::ZERO, 1e-15));
        assert!(close(-a + a * 2.0, a, 1e-15));
        assert!(close(V3::X.rot_z(core::f64::consts::FRAC_PI_2), V3::Y, 1e-15));
    }

    #[test]
    fn quaternion_rotation_convention() {
        // 90° about +Z: body +X is inertial +Y
        let q = Quat::from_axis_angle(V3::Z, core::f64::consts::FRAC_PI_2);
        assert!(close(q.rotate(V3::X), V3::Y, 1e-15));
        assert!(close(q.rotate_inv(V3::Y), V3::X, 1e-15));
        // composition: q2 ⊗ q1 applies q1 first
        let q2 = Quat::from_axis_angle(V3::X, core::f64::consts::FRAC_PI_2);
        let q21 = q2.mul(q);
        assert!(close(q21.rotate(V3::X), q2.rotate(q.rotate(V3::X)), 1e-15));
        assert!(close(q.mul(q.conj()).rotate(V3::new(1.0, 2.0, 3.0)), V3::new(1.0, 2.0, 3.0), 1e-14));
    }

    #[test]
    fn quaternion_integration_matches_constant_spin() {
        let w = V3::new(0.0, 0.0, 0.1);
        let mut q = Quat::identity();
        for _ in 0..100 {
            q = q.integrate(w, 0.1);
        }
        // 1 rad about Z after 10 s
        assert!(close(q.rotate(V3::X), V3::new(1f64.cos(), 1f64.sin(), 0.0), 1e-12));
        assert!((q.norm() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn rng_is_deterministic_and_has_the_right_moments() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        assert_eq!(a.next_u64(), b.next_u64());
        let n = 200_000;
        let (mut s, mut s2, mut e, mut p) = (0.0, 0.0, 0.0, 0.0);
        for _ in 0..n {
            let x = a.normal();
            s += x;
            s2 += x * x;
            e += a.exp(2.0);
            p += a.poisson(3.5) as f64;
            let u = a.uniform();
            assert!((0.0..1.0).contains(&u));
        }
        let nf = n as f64;
        assert!((s / nf).abs() < 0.01);
        assert!((s2 / nf - 1.0).abs() < 0.01);
        assert!((e / nf - 0.5).abs() < 0.005);
        assert!((p / nf - 3.5).abs() < 0.02);
        let big: f64 = (0..10_000).map(|_| a.poisson(100.0) as f64).sum::<f64>() / 10_000.0;
        assert!((big - 100.0).abs() < 0.5);
        let mean_dir = (0..n).fold(V3::ZERO, |acc, _| acc + a.unit_vector()) / nf;
        assert!(mean_dir.norm() < 0.01);
        assert!((a.quat().norm() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn interpolation_clamps() {
        let xs = [0.0, 1.0, 2.0];
        let ys = [0.0, 10.0, 0.0];
        assert_eq!(interp(&xs, &ys, -1.0), 0.0);
        assert_eq!(interp(&xs, &ys, 0.5), 5.0);
        assert_eq!(interp(&xs, &ys, 1.5), 5.0);
        assert_eq!(interp(&xs, &ys, 3.0), 0.0);
    }
}
