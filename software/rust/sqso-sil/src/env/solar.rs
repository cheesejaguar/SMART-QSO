//! The 16-cell solar array: eight strings of two AZUR SPACE 3G30A cells.
//!
//! # Layout (`hardware/eps/SOLAR_ARRAY_DESIGN.md` §3.1)
//!
//! Body axes: +Z along the 113.5 mm rails; ±X, ±Y the long faces. One string
//! (two 40 × 80 mm cells, long side along Z, side by side across the face)
//! on each long body face, and one on each side of the two wings:
//!
//! | String | Deployed | Stowed |
//! |--------|----------|--------|
//! | 0 `PX` | +X body face (shadowed by W1 when the Sun is on the +Y side) | covered by W1: dark |
//! | 1 `MX` | −X body face (shadowed by W2) | covered by W2: dark |
//! | 2 `PY` | +Y body face | +Y |
//! | 3 `MY` | −Y body face | −Y |
//! | 4 `W1F` | W1 front, faces +Y | faces +X (outward) |
//! | 5 `W1B` | W1 back, faces −Y (shadowed by the body) | against the body: dark |
//! | 6 `W2F` | W2 front, faces +Y | faces −X (outward) |
//! | 7 `W2B` | W2 back, faces −Y (shadowed by the body) | against the body: dark |
//!
//! W1 is hinged on the +X/+Y edge and W2 on the −X/+Y edge (axes parallel to
//! Z); deployed, each swings 90° into the plane of the +Y face (§8.1), so the
//! +Y side presents six coplanar cells. Each panel is 83 × 98 mm (§3.1),
//! centred 50 mm out from the hinge. Each wing deploys (or fails) on its own.
//!
//! # Cell model
//!
//! Datasheet DB 00010891-01 values at AM0 1367 W/m² and 28 °C (§2.1): BOL
//! Voc 2.700 V, Isc 0.515 A, Vmp 2.411 V, Imp 0.499 A, and the 5E13 and 1E15
//! 1-MeV electron columns. (Isc 0.5196 A and Imp 0.5044 A, quoted elsewhere
//! for this cell, disagree with the design document; its values from datasheet
//! DB 00010891-01 are used.) Temperature coefficients: dVmp/dT −6.7 mV/K, dImp/dT
//! +0.24 mA/K at BOL, −7.2 mV/K and +0.28 mA/K at 1E15 (§2.1); dVoc/dT
//! −6.2 mV/K and dIsc/dT +0.36 mA/K (datasheet BOL values, not in the design
//! document). Currents scale with irradiance; voltages shift by
//! 3 kT/q · ln(G/G₀) (three junctions, ideality ≈ 1).
//!
//! **Angle of incidence:** cos θ times an approximate Kelly cosine for
//! coverglassed cells (1.0 to 30°, 0.96 at 60°, 0.90 at 70°, 0.62 at 80°,
//! 0 beyond 85°).
//!
//! **Degradation:** fluence Φ(t) = 2.5E13 e/cm² per year (§9.1 takes 5E13
//! for two years at ≤ 525 km SSO behind 100 µm coverglass), each parameter
//! following the remaining-factor form 1 − C log₁₀(1 + Φ/Φₓ) with
//! Φₓ = 1.8E13, through the BOL, 5E13 and 1E15 columns (Pmp −2.4 % after one
//! year, −3.7 % after two). UV darkening and contamination: −1 %/yr, floor
//! −3 % (§9.1 takes × 0.98 at the 2-year EOL). Harness, string diode and
//! mismatch: × 0.97 (§9.1). The MPPT conversion (× 0.95, §9.1) is **not**
//! applied here: [`StringOut::pmp`] is the string's maximum power at its
//! terminals, before the EPS converter.
//!
//! **Shading:** direct sunlight on the four shadowable strings is ray-cast
//! against the body box and the deployed wings, 8 points per cell
//! (§9.2). A string with unequal cells either runs at the weaker cell's
//! current or bypasses it through its diode (0.7 V), whichever gives more
//! power.
//!
//! **Albedo** (optional input): the irradiance on a nadir-facing plate,
//! applied to each cell with the ratio of its Earth view factor to the nadir
//! plate's, without the Kelly correction or shading.

use super::earth::plate_view_factor;
use super::math::{V3, interp};

/// Number of strings.
pub const N_STRINGS: usize = 8;
/// +X body string.
pub const PX: usize = 0;
/// −X body string.
pub const MX: usize = 1;
/// +Y body string.
pub const PY: usize = 2;
/// −Y body string.
pub const MY: usize = 3;
/// Wing 1 front string.
pub const W1F: usize = 4;
/// Wing 1 back string.
pub const W1B: usize = 5;
/// Wing 2 front string.
pub const W2F: usize = 6;
/// Wing 2 back string.
pub const W2B: usize = 7;

/// Reference irradiance of the datasheet (W/m², AM0).
pub const G_REF: f64 = 1367.0;
/// Reference temperature of the datasheet (°C).
pub const T_REF_C: f64 = 28.0;
/// Cell active area (m²).
pub const CELL_AREA_M2: f64 = 30.18e-4;
/// Harness, string diode and mismatch factor (§9.1).
pub const HARNESS_FACTOR: f64 = 0.97;
/// Bypass-diode drop used when a cell is bypassed (V).
pub const BYPASS_DROP_V: f64 = 0.7;
/// 1-MeV equivalent electron fluence per year (e/cm²).
pub const FLUENCE_PER_YEAR: f64 = 2.5e13;

/// Fluence columns of the datasheet (e/cm²): BOL, 5E13, 1E15.
const FLUENCE_COLS: [f64; 3] = [0.0, 5e13, 1e15];
/// Remaining-factor knee Φₓ (e/cm²).
const PHI_X: f64 = 1.8e13;
const VOC: [f64; 3] = [2.700, 2.633, 2.519];
const ISC: [f64; 3] = [0.515, 0.513, 0.496];
const VMP: [f64; 3] = [2.411, 2.336, 2.230];
const IMP: [f64; 3] = [0.499, 0.496, 0.480];
const DVMP_DT: [f64; 3] = [-6.7e-3, -6.95e-3, -7.2e-3];
const DIMP_DT: [f64; 3] = [0.24e-3, 0.26e-3, 0.28e-3];
const DVOC_DT: f64 = -6.2e-3;
const DISC_DT: f64 = 0.36e-3;

/// Kelly-cosine table: incidence angle (deg) and factor applied to cos θ.
const KELLY_DEG: [f64; 9] = [0.0, 30.0, 40.0, 50.0, 60.0, 70.0, 75.0, 80.0, 85.0];
const KELLY_F: [f64; 9] = [1.0, 1.0, 0.995, 0.985, 0.96, 0.90, 0.81, 0.62, 0.0];

/// Effective cosine of incidence (cos θ × Kelly factor), 0 behind the cell.
pub fn kelly_cosine(cos_theta: f64) -> f64 {
    if cos_theta <= 0.0 {
        return 0.0;
    }
    let deg = cos_theta.min(1.0).acos().to_degrees();
    cos_theta * interp(&KELLY_DEG, &KELLY_F, deg)
}

/// Electrical operating point of one string (or cell).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StringOut {
    /// Maximum-power voltage (V).
    pub vmp: f64,
    /// Maximum-power current (A).
    pub imp: f64,
    /// Maximum power (W), before the EPS converter.
    pub pmp: f64,
    /// Open-circuit voltage (V).
    pub voc: f64,
    /// Short-circuit current (A).
    pub isc: f64,
}

impl StringOut {
    /// Current (A) at terminal voltage `v` from the explicit "simplified"
    /// I–V model I = Isc [1 − C₁ (e^(V/(C₂ Voc)) − 1)], with C₁, C₂ fitted
    /// through (0, Isc), (Vmp, Imp) and (Voc, 0).
    pub fn current_at(&self, v: f64) -> f64 {
        if self.isc <= 0.0 || self.voc <= 0.0 || v >= self.voc {
            return 0.0;
        }
        if v <= 0.0 {
            return self.isc;
        }
        let ri = (self.imp / self.isc).clamp(0.01, 0.995);
        let rv = (self.vmp / self.voc).clamp(0.01, 0.995);
        let c2 = (rv - 1.0) / (1.0 - ri).ln();
        let c1 = (1.0 - ri) * (-rv / c2).exp();
        let i = self.isc * (1.0 - c1 * ((v / (c2 * self.voc)).exp() - 1.0));
        i.max(0.0)
    }
}

/// Interpolation weight in the remaining-factor form between two fluence
/// columns.
fn fluence_params(phi: f64) -> (usize, f64) {
    let l = |p: f64| (1.0 + p / PHI_X).log10();
    let lphi = l(phi.max(0.0));
    let (l1, l2) = (l(FLUENCE_COLS[1]), l(FLUENCE_COLS[2]));
    if lphi <= l1 { (0, lphi / l1) } else { (1, (lphi - l1) / (l2 - l1)) }
}

fn lerp3(tab: &[f64; 3], k: usize, f: f64) -> f64 {
    let (a, b) = if k == 0 { (tab[0], tab[1]) } else { (tab[1], tab[2]) };
    a + f * (b - a)
}

/// Cell parameters at 28 °C and 1367 W/m² after a fluence, with the
/// temperature coefficients that apply there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellRef {
    /// Open-circuit voltage (V).
    pub voc: f64,
    /// Short-circuit current (A).
    pub isc: f64,
    /// Maximum-power voltage (V).
    pub vmp: f64,
    /// Maximum-power current (A).
    pub imp: f64,
    /// dVmp/dT (V/K).
    pub dvmp_dt: f64,
    /// dImp/dT (A/K).
    pub dimp_dt: f64,
}

impl CellRef {
    /// Datasheet values after a 1-MeV electron fluence `phi` (e/cm²).
    pub fn at_fluence(phi: f64) -> CellRef {
        let (k, f) = fluence_params(phi);
        CellRef {
            voc: lerp3(&VOC, k, f),
            isc: lerp3(&ISC, k, f),
            vmp: lerp3(&VMP, k, f),
            imp: lerp3(&IMP, k, f),
            dvmp_dt: lerp3(&DVMP_DT, k, f),
            dimp_dt: lerp3(&DIMP_DT, k, f),
        }
    }

    /// Operating point of one cell at irradiance `g` (W/m², effective at
    /// normal incidence) and temperature `temp_c`.
    pub fn cell(&self, g: f64, temp_c: f64) -> StringOut {
        if g < 1.0 {
            return StringOut::default();
        }
        let s = g / G_REF;
        let dt = temp_c - T_REF_C;
        let vt = 8.617_333e-5 * (temp_c + 273.15);
        let dv_g = 3.0 * vt * s.ln();
        let voc = (self.voc + DVOC_DT * dt + dv_g).max(0.0);
        let vmp = (self.vmp + self.dvmp_dt * dt + dv_g).clamp(0.0, voc);
        let isc = ((self.isc + DISC_DT * dt) * s).max(0.0);
        let imp = ((self.imp + self.dimp_dt * dt) * s).clamp(0.0, isc);
        StringOut { vmp, imp, pmp: vmp * imp, voc, isc }
    }
}

/// Two cells in series with bypass diodes: the better of running both at
/// the weaker cell's current or bypassing the weaker cell.
fn series_pair(a: StringOut, b: StringOut) -> StringOut {
    let (strong, weak) = if a.imp >= b.imp { (a, b) } else { (b, a) };
    if strong.imp <= 0.0 {
        return StringOut::default();
    }
    let both = StringOut {
        vmp: strong.vmp + weak.vmp,
        imp: weak.imp,
        pmp: (strong.vmp + weak.vmp) * weak.imp,
        voc: strong.voc + weak.voc,
        isc: weak.isc,
    };
    let v_bypass = (strong.vmp - BYPASS_DROP_V).max(0.0);
    let bypass = StringOut {
        vmp: v_bypass,
        imp: strong.imp,
        pmp: v_bypass * strong.imp,
        voc: strong.voc + weak.voc,
        isc: strong.isc,
    };
    if bypass.pmp > both.pmp { bypass } else { both }
}

/// A cell: centre (m, body frame), outward normal, and the two in-plane
/// half-extent axes (m).
#[derive(Clone, Copy, Debug)]
struct Cell {
    centre: V3,
    normal: V3,
    u: V3,
    v: V3,
}

/// What can shadow a string.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Occluder {
    None,
    Wing1,
    Wing2,
    Body,
}

/// Half extents of the body box (m).
const BODY_HALF: V3 = V3::new(0.05, 0.05, 0.056_75);
/// Wing panel centre offset from the body axis along X (m) and half extents
/// across and along Z (m).
const WING_CX: f64 = 0.10;
const WING_HALF: (f64, f64) = (0.0415, 0.049);
/// Cell centre offset across a panel (m) and cell half extents across and
/// along Z (m).
const CELL_DX: f64 = 0.0205;
const CELL_HALF: (f64, f64) = (0.020, 0.040);
/// Stand-off of a stowed wing from the body face (m).
const STOWED_GAP: f64 = 0.004;

fn face_cells(centre: V3, normal: V3, across: V3) -> [Cell; 2] {
    let u = across * CELL_HALF.0;
    let v = V3::Z * CELL_HALF.1;
    [Cell { centre: centre + across * CELL_DX, normal, u, v }, Cell { centre: centre - across * CELL_DX, normal, u, v }]
}

/// Geometry of the strings for one wing configuration.
#[derive(Clone, Copy, Debug)]
struct Layout {
    cells: [[Cell; 2]; N_STRINGS],
    lit_possible: [bool; N_STRINGS],
    occluder: [Occluder; N_STRINGS],
}

fn layout(wings: [bool; 2]) -> Layout {
    let (x, y, mx, my) = (V3::X, V3::Y, -V3::X, -V3::Y);
    let h = BODY_HALF.x;
    let w1c = V3::new(WING_CX, h, 0.0);
    let w2c = V3::new(-WING_CX, h, 0.0);
    let cells = [
        face_cells(V3::new(h, 0.0, 0.0), x, y),
        face_cells(V3::new(-h, 0.0, 0.0), mx, y),
        face_cells(V3::new(0.0, h, 0.0), y, x),
        face_cells(V3::new(0.0, -h, 0.0), my, x),
        if wings[0] { face_cells(w1c, y, x) } else { face_cells(V3::new(h + STOWED_GAP, 0.0, 0.0), x, y) },
        if wings[0] { face_cells(w1c, my, x) } else { face_cells(V3::new(h + STOWED_GAP, 0.0, 0.0), mx, y) },
        if wings[1] { face_cells(w2c, y, x) } else { face_cells(V3::new(-h - STOWED_GAP, 0.0, 0.0), mx, y) },
        if wings[1] { face_cells(w2c, my, x) } else { face_cells(V3::new(-h - STOWED_GAP, 0.0, 0.0), x, y) },
    ];
    let lit_possible = [wings[0], wings[1], true, true, true, wings[0], true, wings[1]];
    let o = |c: bool, k: Occluder| if c { k } else { Occluder::None };
    let occluder = [
        o(wings[0], Occluder::Wing1),
        o(wings[1], Occluder::Wing2),
        Occluder::None,
        Occluder::None,
        Occluder::None,
        o(wings[0], Occluder::Body),
        Occluder::None,
        o(wings[1], Occluder::Body),
    ];
    Layout { cells, lit_possible, occluder }
}

/// Does the ray p + t s (t > 0) cross the deployed wing centred at x = `cx`?
fn hits_wing(p: V3, s: V3, cx: f64) -> bool {
    if s.y.abs() < 1e-12 {
        return false;
    }
    let t = (BODY_HALF.y - p.y) / s.y;
    if t <= 1e-9 {
        return false;
    }
    let q = p + s * t;
    (q.x - cx).abs() <= WING_HALF.0 && q.z.abs() <= WING_HALF.1
}

/// Does the ray p + t s (t > 0) cross the body box (slab test)?
fn hits_body(p: V3, s: V3) -> bool {
    let (mut t0, mut t1) = (1e-9, f64::INFINITY);
    for (pi, si, hi) in [(p.x, s.x, BODY_HALF.x), (p.y, s.y, BODY_HALF.y), (p.z, s.z, BODY_HALF.z)] {
        if si.abs() < 1e-12 {
            if pi.abs() > hi {
                return false;
            }
        } else {
            let (a, b) = ((-hi - pi) / si, (hi - pi) / si);
            let (a, b) = if a < b { (a, b) } else { (b, a) };
            t0 = f64::max(t0, a);
            t1 = f64::min(t1, b);
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// Sample-point offsets across (2) and along (4) a cell, in units of the
/// half extents.
const SAMPLES_U: [f64; 2] = [-0.5, 0.5];
const SAMPLES_V: [f64; 4] = [-0.75, -0.25, 0.25, 0.75];

/// Unshadowed fraction of a cell for the Sun along `s` (body frame).
fn lit_fraction(c: &Cell, s: V3, occ: Occluder) -> f64 {
    if occ == Occluder::None {
        return 1.0;
    }
    let mut lit = 0u32;
    for &a in &SAMPLES_U {
        for &b in &SAMPLES_V {
            let p = c.centre + c.u * a + c.v * b + c.normal * 1e-4;
            let blocked = match occ {
                Occluder::Wing1 => hits_wing(p, s, WING_CX),
                Occluder::Wing2 => hits_wing(p, s, -WING_CX),
                Occluder::Body => hits_body(p, s),
                Occluder::None => false,
            };
            if !blocked {
                lit += 1;
            }
        }
    }
    f64::from(lit) / (SAMPLES_U.len() * SAMPLES_V.len()) as f64
}

/// Inputs of one array evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrayInputs {
    /// Unit Sun direction in the body frame.
    pub sun_body: V3,
    /// Illuminated fraction of the solar disc (0..1, from
    /// [`super::eclipse::illumination`]).
    pub illum: f64,
    /// Solar irradiance at the spacecraft (W/m²).
    pub flux: f64,
    /// Unit nadir direction in the body frame (for albedo).
    pub nadir_body: V3,
    /// Albedo irradiance on a nadir-facing plate (W/m²; 0 to ignore albedo),
    /// for example [`super::earth::albedo_nadir_w_m2`].
    pub albedo_w_m2: f64,
    /// Distance from the Earth's centre (m), for the albedo view factors.
    pub r_m: f64,
    /// Cell temperature of each string (°C).
    pub temp_c: [f64; N_STRINGS],
    /// Time since launch (years), for the degradation.
    pub years_in_orbit: f64,
    /// Whether each wing (W1, W2) is deployed.
    pub wings_deployed: [bool; 2],
}

/// The solar array model.
#[derive(Clone, Debug)]
pub struct Array {
    layouts: [Layout; 4],
    /// Ray-cast self-shadowing (true by default; false reproduces the
    /// BASELINE §5 no-shadowing method).
    pub shadowing: bool,
    /// 1-MeV equivalent fluence per year (e/cm²).
    pub fluence_per_year: f64,
}

impl Default for Array {
    fn default() -> Array {
        Array::new()
    }
}

/// UV darkening and contamination factor after `years` in orbit.
pub fn uv_factor(years: f64) -> f64 {
    (1.0 - 0.01 * years.max(0.0)).max(0.97)
}

impl Array {
    /// The baseline array.
    pub fn new() -> Array {
        let l = |a: bool, b: bool| layout([a, b]);
        Array {
            layouts: [l(false, false), l(true, false), l(false, true), l(true, true)],
            shadowing: true,
            fluence_per_year: FLUENCE_PER_YEAR,
        }
    }

    fn layout_for(&self, wings: [bool; 2]) -> &Layout {
        let k = usize::from(wings[0]) + 2 * usize::from(wings[1]);
        &self.layouts[k.min(3)]
    }

    /// Outward normal of each string's cells in the body frame (zero for a
    /// string that faces the body), for the thermal coupling.
    pub fn normals(&self, wings_deployed: [bool; 2]) -> [V3; N_STRINGS] {
        let l = self.layout_for(wings_deployed);
        let mut n = [V3::ZERO; N_STRINGS];
        for (k, out) in n.iter_mut().enumerate() {
            if l.lit_possible[k] {
                *out = l.cells[k][0].normal;
            }
        }
        n
    }

    /// Operating points of the eight strings.
    pub fn strings(&self, inp: &ArrayInputs) -> [StringOut; N_STRINGS] {
        let l = self.layout_for(inp.wings_deployed);
        let cref = CellRef::at_fluence(self.fluence_per_year * inp.years_in_orbit.max(0.0));
        let derate = HARNESS_FACTOR * uv_factor(inp.years_in_orbit);
        let direct = inp.flux * inp.illum.clamp(0.0, 1.0);
        let s = inp.sun_body.unit();
        let alb_ref = if inp.albedo_w_m2 > 0.0 { plate_view_factor(1.0, inp.r_m) } else { 0.0 };
        let mut out = [StringOut::default(); N_STRINGS];
        for (k, slot) in out.iter_mut().enumerate() {
            if !l.lit_possible[k] {
                continue;
            }
            let mut cells = [StringOut::default(); 2];
            for (j, c) in l.cells[k].iter().enumerate() {
                let mut g = 0.0;
                let cs = c.normal.dot(s);
                if direct > 0.0 && cs > 0.0 {
                    let occ = if self.shadowing { l.occluder[k] } else { Occluder::None };
                    g += direct * kelly_cosine(cs) * lit_fraction(c, s, occ);
                }
                if alb_ref > 0.0 {
                    g += inp.albedo_w_m2 * plate_view_factor(c.normal.dot(inp.nadir_body), inp.r_m) / alb_ref;
                }
                cells[j] = cref.cell(g, inp.temp_c[k]);
            }
            let mut st = series_pair(cells[0], cells[1]);
            st.imp *= derate;
            st.isc *= derate;
            st.pmp = st.vmp * st.imp;
            *slot = st;
        }
        out
    }

    /// Sum of the strings' maximum power (W).
    pub fn total_pmp(&self, inp: &ArrayInputs) -> f64 {
        self.strings(inp).iter().map(|s| s.pmp).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::math::Rng;

    /// End of life (2 years), 60 °C, 1361 W/m² (§9.1).
    fn eol(sun_body: V3, wings: [bool; 2]) -> ArrayInputs {
        ArrayInputs {
            sun_body,
            illum: 1.0,
            flux: 1361.0,
            nadir_body: -V3::Z,
            albedo_w_m2: 0.0,
            r_m: 6.878e6,
            temp_c: [60.0; N_STRINGS],
            years_in_orbit: 2.0,
            wings_deployed: wings,
        }
    }

    #[test]
    fn datasheet_points_and_temperature() {
        let bol = CellRef::at_fluence(0.0).cell(G_REF, T_REF_C);
        assert!((bol.pmp - 1.203).abs() < 0.001);
        let c = CellRef::at_fluence(5e13);
        assert!((c.vmp - 2.336).abs() < 1e-9 && (c.imp - 0.496).abs() < 1e-9);
        // §9.1: 5E13, 60 °C → 2,121.6 mV × 503.7 mA = 1.069 W (coefficients at BOL)
        let hot = CellRef { dvmp_dt: -6.7e-3, dimp_dt: 0.24e-3, ..c }.cell(G_REF, 60.0);
        assert!((hot.vmp - 2.1216).abs() < 1e-4 && (hot.imp - 0.5037).abs() < 1e-4);
        let e = CellRef::at_fluence(1e15);
        assert!((e.vmp - 2.230).abs() < 1e-9);
        // one year: between BOL and 5E13, Pmp −2.4 %
        let y1 = CellRef::at_fluence(FLUENCE_PER_YEAR).cell(G_REF, T_REF_C);
        assert!((y1.pmp / bol.pmp - 0.976).abs() < 0.004, "{}", y1.pmp / bol.pmp);
    }

    #[test]
    fn iv_curve_passes_through_the_points() {
        let s = CellRef::at_fluence(0.0).cell(G_REF, T_REF_C);
        assert!((s.current_at(0.0) - s.isc).abs() < 1e-12);
        assert!((s.current_at(s.vmp) - s.imp).abs() < 1e-6);
        assert!(s.current_at(s.voc) == 0.0);
        let p_best = (0..1000).map(|k| f64::from(k) * s.voc / 1000.0).map(|v| v * s.current_at(v)).fold(0.0, f64::max);
        assert!((p_best - s.pmp).abs() / s.pmp < 0.02);
    }

    #[test]
    fn kelly_cosine_shape() {
        assert_eq!(kelly_cosine(1.0), 1.0);
        assert_eq!(kelly_cosine(-0.5), 0.0);
        assert!(kelly_cosine(f64::to_radians(86.0).cos()) == 0.0);
        assert!((kelly_cosine(0.5) - 0.48).abs() < 1e-9);
    }

    #[test]
    fn sun_on_plus_y_lights_six_cells() {
        let a = Array::new();
        let p = a.strings(&eol(V3::Y, [true, true]));
        let lit: Vec<usize> = (0..N_STRINGS).filter(|&k| p[k].pmp > 0.0).collect();
        assert_eq!(lit, vec![PY, W1F, W2F]);
        // 6 cells × 1.064 W × 0.97 × 0.98 (§9.1 before MPPT) = 6.07 W
        let total: f64 = p.iter().map(|s| s.pmp).sum();
        assert!((total - 6.07).abs() < 0.05, "{total}");
        // string voltage ≈ 4.24 V at EOL and 60 °C (§3.2; −6.95 mV/K at 5E13 here)
        assert!((p[PY].vmp - 4.24).abs() < 0.03, "{}", p[PY].vmp);
    }

    #[test]
    fn shadowing_cases() {
        let a = Array::new();
        // Sun from +X+Y at 45°: W1 shades part of the +X body string
        let s = V3::new(1.0, 1.0, 0.0).unit();
        let shaded = a.strings(&eol(s, [true, true]))[PX];
        let mut no = Array::new();
        no.shadowing = false;
        let clear = no.strings(&eol(s, [true, true]))[PX];
        assert!(shaded.pmp < clear.pmp * 0.75, "{} {}", shaded.pmp, clear.pmp);
        // Sun from −X−Y: the body shades the W1 back (cells at +X), not W2's
        let s = V3::new(-1.0, -0.3, 0.0).unit();
        let p = a.strings(&eol(s, [true, true]));
        assert_eq!(p[W1B].pmp, 0.0);
        assert!(p[W2B].pmp > 0.0);
        // stowed: covered strings are dark, the wing fronts face ±X
        let p = a.strings(&eol(V3::X, [false, false]));
        assert_eq!(p[PX].pmp, 0.0);
        assert!(p[W1F].pmp > 0.9);
        assert_eq!(p[W1B].pmp, 0.0);
        // Sun along Z: nothing (§9.3 worst case)
        assert_eq!(a.total_pmp(&eol(V3::Z, [true, true])), 0.0);
        // eclipse
        assert_eq!(a.total_pmp(&ArrayInputs { illum: 0.0, ..eol(V3::Y, [true, true]) }), 0.0);
    }

    #[test]
    fn albedo_adds_power_on_nadir_faces() {
        let a = Array::new();
        let mut inp = eol(V3::Y, [true, true]);
        inp.illum = 0.0;
        inp.nadir_body = -V3::Y;
        inp.albedo_w_m2 = 350.0;
        let p = a.strings(&inp);
        assert!(p[MY].pmp > 0.2 && p[MY].pmp < 0.5, "{}", p[MY].pmp);
        assert!(p[PY].pmp == 0.0);
    }

    /// Calibration: orbit-average power (sum of string Pmp, before MPPT) for
    /// a uniformly tumbling spacecraft and for +Y sun-pointing at EOL, 500 km,
    /// β = 0 (sunlit fraction 0.622, §9.2), against BASELINE §5 and §9.3.
    #[test]
    fn orbit_average_power_calibration() {
        const F_SUN: f64 = 0.622;
        const MPPT: f64 = 0.95;
        let a = Array::new();
        let mut no = Array::new();
        no.shadowing = false;
        let mut rng = Rng::new(2024);
        let n = 40_000;
        let (mut p_sh, mut p_no) = (0.0, 0.0);
        for _ in 0..n {
            let s = rng.unit_vector();
            p_sh += a.total_pmp(&eol(s, [true, true]));
            p_no += no.total_pmp(&eol(s, [true, true]));
        }
        let tumbling = p_sh / f64::from(n) * F_SUN;
        let tumbling_no = p_no / f64::from(n) * F_SUN;
        let pointed = a.total_pmp(&eol(V3::Y, [true, true])) * F_SUN;
        println!(
            "orbit average before MPPT: tumbling {tumbling:.3} W (no shadowing {tumbling_no:.3} W), \
             sun-pointed +Y {pointed:.3} W; after × 0.95 MPPT: {:.3} / {:.3} / {:.3} W; shadow factor {:.3}",
            tumbling * MPPT,
            tumbling_no * MPPT,
            pointed * MPPT,
            tumbling / tumbling_no
        );
        assert!((tumbling - 2.39).abs() < 0.2 * 2.39, "{tumbling}");
        assert!((tumbling * MPPT - 2.14).abs() < 0.2 * 2.14);
        assert!((pointed - 3.58).abs() < 0.2 * 3.58, "{pointed}");
        assert!((pointed * MPPT - 3.59).abs() < 0.05);
        // the ray-cast shadow factor of §9.2 is 0.89
        assert!((tumbling / tumbling_no - 0.89).abs() < 0.04);
        // both wings stowed: 8 exposed cells, ≈ 1.20 W after MPPT (§9.3)
        let mut st = 0.0;
        for _ in 0..n {
            st += a.total_pmp(&eol(rng.unit_vector(), [false, false]));
        }
        let stowed = st / f64::from(n) * F_SUN * MPPT;
        println!("both wings stowed, tumbling, after MPPT: {stowed:.3} W");
        assert!((stowed - 1.20).abs() < 0.15);
    }
}
