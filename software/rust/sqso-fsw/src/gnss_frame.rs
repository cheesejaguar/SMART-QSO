//! The GNSS state frame (RID LV-04; `docs/operations/TELEMETRY_DICTIONARY.md`
//! §4.4.2, SQSO-TLM-001 1.3): after each GNSS session that ended with a fix, and again on
//! GET_HOUSEKEEPING, one plain-ASCII UI frame
//!
//! ```text
//! GNS N=<session> WK=<week> TOW=<ms> X=<m> Y=<m> Z=<m> VX=<mm/s> VY=<mm/s> VZ=<mm/s> VDT=<s> Q=<quality> SV=<sats> PD=<PDOP x 10> AGE=<s>
//! ```
//!
//! - `N`: the GNSS session that took the fix (HK1 `GS`, first number); a
//!   frame sent again on GET_HOUSEKEEPING keeps it, so the ground can drop
//!   the repeat.
//! - `WK`, `TOW`: the GPS week as the receiver reports it (10 bits, the
//!   ground resolves the rollover) and the GPS time
//!   of week of the fix, ms (the `$PSLSP` record's epoch).
//! - `X`, `Y`, `Z`: the receiver's ECEF position at that epoch, m (WGS-84).
//! - `VX`, `VY`, `VZ`: ECEF velocity at the same epoch, mm/s. The piNAV-NG
//!   state record carries no velocity, so it is derived: the mean velocity
//!   over the chord from the session's first accepted fix to this one (`VDT`
//!   seconds, accepted between [`CHORD_MIN_S`] and [`CHORD_MAX_S`]) is
//!   carried to the fix epoch with the acceleration in the Earth-fixed frame
//!   (point mass, J2, Coriolis and centrifugal terms):
//!   v₁ = (r₁ − r₀)/Δt + Δt (2 a(mid) + a(r₁))/6, which is
//!   v₁ = (r₁ − r₀)/Δt + (1/Δt) ∫ (t − t₀) a dt by Simpson's rule. The
//!   method's own error is 0.02 mm/s over 30 s and 8 mm/s over 120 s; the
//!   receiver's position noise σ gives about √2 σ / `VDT` (0.5 m/s for 10 m
//!   over 30 s). `-` (and `VDT=-`) when the session had no usable chord.
//! - `Q`: the GGA fix quality with the fix (1 GPS, 2 differential: the only
//!   ones the receiver driver accepts); `SV`: satellites used; `PD`: PDOP × 10.
//! - `AGE`: seconds from the fix to this frame (0 for the frame after the
//!   session; the repeat on GET_HOUSEKEEPING shows how old the fix is).
//!
//! Integers throughout, `-` for a value not available, keys in this order;
//! a decoder ignores keys it does not know (`software/ground/tools/gnss_frame.py`).
//!
//! @requirement SRS-F158 GNSS state frame after each session with a fix and on GET_HOUSEKEEPING: GPS week and time, ECEF position and velocity, fix quality, satellites, PDOP

use core::fmt::Write;

use crate::fmath::{lround, sqrt};
use crate::gnss::Fix;

/// The information field's start
pub const PREFIX: &str = "GNS";
/// Longest information field, bytes
pub const MAX_LEN: usize = 160;
/// Shortest chord the velocity is derived over, s (position noise)
pub const CHORD_MIN_S: f64 = 10.0;
/// Longest chord, s (the method error grows as the chord to the fourth power)
pub const CHORD_MAX_S: f64 = 120.0;

/// GPS week, s
const WEEK_S: f64 = 604_800.0;
/// WGS-84 / EGM96: GM, m³/s²
const GM: f64 = 3.986_004_418e14;
/// Equatorial radius, m
const RE_M: f64 = 6_378_137.0;
/// Second zonal harmonic
const J2: f64 = 1.082_626_68e-3;
/// Earth rotation rate, rad/s
const OMEGA: f64 = 7.292_115_146_7e-5;

type V3 = [f64; 3];

/// One frame's values
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    /// GNSS session that took the fix
    pub session: u32,
    /// GPS week as reported
    pub week: u16,
    /// GPS time of week, ms
    pub tow_ms: i64,
    /// ECEF position, m
    pub pos_m: [i64; 3],
    /// ECEF velocity, mm/s, and the chord it came from, s
    pub vel: Option<([i64; 3], i64)>,
    /// GGA fix quality
    pub quality: u8,
    /// Satellites used
    pub sats: u8,
    /// PDOP × 10
    pub pdop10: i64,
    /// Fix age when the frame is made, s
    pub age_s: u64,
}

fn pos(f: &Fix) -> V3 {
    [f.x_m, f.y_m, f.z_m]
}

fn add(a: V3, b: V3, k: f64) -> V3 {
    let ([ax, ay, az], [bx, by, bz]) = (a, b);
    [ax + k * bx, ay + k * by, az + k * bz]
}

/// GPS time from `a` to `b`, s (the reported week may wrap at 1024)
fn elapsed_s(a: &Fix, b: &Fix) -> f64 {
    let weeks = b.week.wrapping_sub(a.week) & 0x3FF;
    f64::from(weeks) * WEEK_S + (b.tow_s - a.tow_s)
}

/// Acceleration in the Earth-fixed frame at position `r` with velocity `v`
/// (ECEF): point mass and J2, Coriolis, centrifugal
pub fn accel_ecef(r: V3, v: V3) -> V3 {
    let ([x, y, z], [vx, vy, _]) = (r, v);
    let r2 = x * x + y * y + z * z;
    let rn = sqrt(r2);
    let mu_r3 = GM / (r2 * rn);
    let k = 1.5 * J2 * (RE_M * RE_M) / r2;
    let zz = 5.0 * z * z / r2;
    let w2 = OMEGA * OMEGA;
    [
        -mu_r3 * x * (1.0 + k * (1.0 - zz)) + 2.0 * OMEGA * vy + w2 * x,
        -mu_r3 * y * (1.0 + k * (1.0 - zz)) - 2.0 * OMEGA * vx + w2 * y,
        -mu_r3 * z * (1.0 + k * (3.0 - zz)),
    ]
}

/// ECEF velocity at `last`'s epoch, m/s, from the chord `first` → `last`,
/// and the chord, s; `None` outside [`CHORD_MIN_S`]..=[`CHORD_MAX_S`]
pub fn velocity(first: &Fix, last: &Fix) -> Option<(V3, f64)> {
    let dt = elapsed_s(first, last);
    if !(CHORD_MIN_S..=CHORD_MAX_S).contains(&dt) {
        return None;
    }
    let (r0, r1) = (pos(first), pos(last));
    // The mean velocity, close to the velocity at the chord's midpoint
    let vm = add(r1, r0, -1.0).map(|d| d / dt);
    // The midpoint position: the chord's middle less a dt²/8
    let chord_mid = add(r0, r1, 1.0).map(|c| c / 2.0);
    let am = accel_ecef(chord_mid, vm);
    let rm = add(chord_mid, am, -dt * dt / 8.0);
    let am = accel_ecef(rm, vm);
    let a1 = accel_ecef(r1, add(vm, am, dt / 2.0));
    // v1 = vm + (1/dt) ∫ (t - t0) a dt, Simpson's rule (the weight at t0 is 0)
    let v1 = add(vm, add(a1, am, 2.0), dt / 6.0);
    Some((v1, dt))
}

/// The frame's values for the fix `last`, the session's first fix `first`
/// (`None`: no chord), the GGA quality with it, at `now_ms`
pub fn state(session: u32, first: Option<&Fix>, last: &Fix, quality: u8, now_ms: u64) -> State {
    let vel = first.and_then(|f| velocity(f, last)).map(|(v, dt)| (v.map(|c| lround(c * 1000.0)), lround(dt)));
    State {
        session,
        week: last.week,
        tow_ms: lround(last.tow_s * 1000.0),
        pos_m: pos(last).map(lround),
        vel,
        quality,
        sats: last.sats,
        pdop10: lround(last.pdop * 10.0),
        age_s: now_ms.saturating_sub(last.rx_ms) / 1000,
    }
}

struct Out<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl Write for Out<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(core::fmt::Error)?;
        self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// Largest magnitude printed for a position (m) or velocity (mm/s)
/// component: eight digits (a LEO position is at most 7.5e6 m, a velocity
/// 8e6 mm/s); a wilder value prints saturated
const COMPONENT_MAX: i64 = 99_999_999;

/// The information field in `out`; its length. Every field is clamped to
/// a fixed width (RID P3F-A-05), so with `out` at least [`MAX_LEN`] bytes
/// the frame always fits and is never empty (the widest is
/// `frame_without_a_chord_and_the_longest_frame`'s); 0 only for a smaller
/// `out`
pub fn format(s: &State, out: &mut [u8]) -> usize {
    let mut w = Out { buf: out, len: 0 };
    let c = |v: i64| v.clamp(-COMPONENT_MAX, COMPONENT_MAX);
    let [x, y, z] = s.pos_m.map(c);
    let tow = s.tow_ms.clamp(0, 604_799_999);
    let ok = write!(w, "{PREFIX} N={} WK={} TOW={tow} X={x} Y={y} Z={z} ", s.session, s.week)
        .and_then(|()| match s.vel {
            Some((v, dt)) => {
                let [vx, vy, vz] = v.map(c);
                write!(w, "VX={vx} VY={vy} VZ={vz} VDT={} ", dt.clamp(0, 999))
            }
            None => w.write_str("VX=- VY=- VZ=- VDT=- "),
        })
        .and_then(|()| {
            let pd = s.pdop10.clamp(0, 999);
            write!(w, "Q={} SV={} PD={pd} AGE={}", s.quality, s.sats, s.age_s.min(99_999_999))
        })
        .is_ok();
    if ok { w.len } else { 0 }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::format;

    use super::*;

    /// Truth: the same force model in the inertial frame (J2 is symmetric
    /// about the axis), integrated by RK4 at 0.1 s
    fn inertial_accel(r: V3) -> V3 {
        let [x, y, z] = r;
        let r2 = x * x + y * y + z * z;
        let rn = sqrt(r2);
        let mu_r3 = GM / (r2 * rn);
        let k = 1.5 * J2 * (RE_M * RE_M) / r2;
        let zz = 5.0 * z * z / r2;
        [-mu_r3 * x * (1.0 + k * (1.0 - zz)), -mu_r3 * y * (1.0 + k * (1.0 - zz)), -mu_r3 * z * (1.0 + k * (3.0 - zz))]
    }

    fn rk4(r: V3, v: V3, h: f64) -> (V3, V3) {
        let k1v = inertial_accel(r);
        let k1r = v;
        let k2v = inertial_accel(add(r, k1r, h / 2.0));
        let k2r = add(v, k1v, h / 2.0);
        let k3v = inertial_accel(add(r, k2r, h / 2.0));
        let k3r = add(v, k2v, h / 2.0);
        let k4v = inertial_accel(add(r, k3r, h));
        let k4r = add(v, k3v, h);
        let dr = add(add(k1r, k2r, 2.0), add(k4r, k3r, 2.0), 1.0);
        let dv = add(add(k1v, k2v, 2.0), add(k4v, k3v, 2.0), 1.0);
        (add(r, dr, h / 6.0), add(v, dv, h / 6.0))
    }

    /// Inertial state at t (from the ECEF axes at t = 0) to ECEF position and velocity
    fn to_ecef(r: V3, v: V3, t: f64) -> (V3, V3) {
        let (s, c) = ((OMEGA * t).sin(), (OMEGA * t).cos());
        let rot = |[x, y, z]: V3| [c * x + s * y, -s * x + c * y, z];
        let re = rot(r);
        // v_ecef = R (v - ω × r)
        let [x, y, _] = r;
        let ve = rot(add(v, [-y, x, 0.0], -OMEGA));
        (re, ve)
    }

    fn fix(r: V3, tow_s: f64, week: u16) -> Fix {
        let [x_m, y_m, z_m] = r;
        Fix { valid: true, rx_ms: 1_000_000, tow_s, week, x_m, y_m, z_m, sats: 8, pdop: 1.84 }
    }

    /// A 475 km, 53° orbit: the fixes at 0 s and `dt` s, and the truth at `dt`
    fn chord(dt: f64) -> (Fix, Fix, V3) {
        let a = 6_378_137.0 + 475e3;
        let vc = sqrt(GM / a);
        let inc = 53.0_f64.to_radians();
        let (mut r, mut v) = ([a, 0.0, 0.0], [0.0, vc * inc.cos(), vc * inc.sin()]);
        let (r0, _) = to_ecef(r, v, 0.0);
        let steps = lround(dt * 10.0);
        for _ in 0..steps {
            (r, v) = rk4(r, v, 0.1);
        }
        let (r1, v1) = to_ecef(r, v, dt);
        (fix(r0, 345_570.0, 2391), fix(r1, 345_570.0 + dt, 2391), v1)
    }

    fn err(a: V3, b: V3) -> f64 {
        let [x, y, z] = add(a, b, -1.0);
        sqrt(x * x + y * y + z * z)
    }

    #[test]
    fn velocity_matches_the_orbit() {
        for dt in [10.0, 30.0, 60.0, 120.0] {
            let (f0, f1, truth) = chord(dt);
            let (v, got_dt) = velocity(&f0, &f1).unwrap_or_default();
            assert!((got_dt - dt).abs() < 1e-9);
            let e = err(v, truth);
            assert!(e < 0.01, "chord {dt} s: {e} m/s");
        }
    }

    #[test]
    fn the_acceleration_terms_matter() {
        // Without the correction the mean velocity is several m/s off
        let (f0, f1, truth) = chord(30.0);
        let mean = add(pos(&f1), pos(&f0), -1.0).map(|d| d / 30.0);
        assert!(err(mean, truth) > 50.0);
    }

    #[test]
    fn chord_limits() {
        let (f0, f1, _) = chord(30.0);
        assert!(velocity(&f0, &f0).is_none());
        let short = Fix { tow_s: f0.tow_s + 9.9, ..f1 };
        assert!(velocity(&f0, &short).is_none());
        let long = Fix { tow_s: f0.tow_s + 120.5, ..f1 };
        assert!(velocity(&f0, &long).is_none());
        let backwards = Fix { tow_s: f0.tow_s - 30.0, ..f1 };
        assert!(velocity(&f0, &backwards).is_none());
        // Across the end of the week, and across a 10-bit week rollover
        let start = Fix { tow_s: 604_790.0, week: 1023, ..f0 };
        let end = Fix { tow_s: 20.0, week: 0, ..f1 };
        assert!(velocity(&start, &end).is_some_and(|(_, dt)| (dt - 30.0).abs() < 1e-9));
    }

    #[test]
    fn frame_text() {
        let (f0, f1, truth) = chord(30.0);
        let s = state(17, Some(&f0), &f1, 1, f1.rx_ms + 2_500);
        assert_eq!(s.vel.map(|(_, dt)| dt), Some(30));
        let mut out = [0u8; MAX_LEN];
        let n = format(&s, &mut out);
        let text = core::str::from_utf8(out.get(..n).unwrap_or(&[])).unwrap_or("");
        let [x, y, z] = pos(&f1).map(lround);
        let [vx, vy, vz] = truth.map(|c| lround(c * 1000.0));
        let near = |got: i64, want: i64| (got - want).abs() <= 20;
        let (gx, gy, gz) = s.vel.map(|([a, b, c], _)| (a, b, c)).unwrap_or_default();
        assert!(near(gx, vx));
        assert!(near(gy, vy));
        assert!(near(gz, vz));
        let want_start = format!("GNS N=17 WK=2391 TOW=345600000 X={x} Y={y} Z={z} VX={gx} VY={gy} VZ={gz} VDT=30 ");
        assert!(text.starts_with(&want_start), "{text}");
        assert!(text.ends_with(" Q=1 SV=8 PD=18 AGE=2"), "{text}");
    }

    #[test]
    fn frame_without_a_chord_and_the_longest_frame() {
        let (_, f1, _) = chord(30.0);
        let s = state(3, None, &f1, 2, f1.rx_ms);
        let mut out = [0u8; MAX_LEN];
        let n = format(&s, &mut out);
        let text = core::str::from_utf8(out.get(..n).unwrap_or(&[])).unwrap_or("");
        assert!(text.contains(" VX=- VY=- VZ=- VDT=- Q=2 SV=8 PD=18 AGE=0"), "{text}");
        // RID P3F-A-05: every field at its type's extreme fits, clamped
        let widest = State {
            session: u32::MAX,
            week: u16::MAX,
            tow_ms: i64::MAX,
            pos_m: [i64::MIN; 3],
            vel: Some(([i64::MIN; 3], i64::MAX)),
            quality: u8::MAX,
            sats: u8::MAX,
            pdop10: i64::MAX,
            age_s: u64::MAX,
        };
        let n = format(&widest, &mut out);
        let text = core::str::from_utf8(out.get(..n).unwrap_or(&[])).unwrap_or("");
        assert_eq!(
            text,
            "GNS N=4294967295 WK=65535 TOW=604799999 X=-99999999 Y=-99999999 Z=-99999999 VX=-99999999 \
             VY=-99999999 VZ=-99999999 VDT=999 Q=255 SV=255 PD=999 AGE=99999999"
        );
        assert!(n <= MAX_LEN, "{n}");
        let low = State { tow_ms: i64::MIN, pdop10: i64::MIN, vel: Some(([i64::MAX; 3], i64::MIN)), ..widest };
        let n = format(&low, &mut out);
        assert!(n > 0 && n <= MAX_LEN);
        assert_eq!(format(&widest, &mut [0u8; 40]), 0);
    }
}
