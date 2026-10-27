//! Simulation time: seconds since J2000.0 (2000-01-01 12:00:00 UTC) as `f64`.
//!
//! UTC is treated as uniform time: leap seconds and TT − UTC (≈ 69 s) are
//! ignored, which shifts the Sun by < 0.001° and the Earth's rotation angle by
//! < 0.3° — below every other error in these models.

use core::f64::consts::TAU;

use super::math::DAY_S;

/// Julian date of J2000.0.
pub const JD_J2000: f64 = 2_451_545.0;

/// Days from 1970-01-01 to a proleptic Gregorian civil date (Hinnant's
/// algorithm).
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Inverse of [`days_from_civil`].
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Days from 1970-01-01 to 2000-01-01.
const UNIX_DAYS_2000: i64 = 10_957;

/// Seconds since J2000.0 of a UTC calendar date and time.
pub fn from_utc(y: i32, mo: u32, d: u32, h: u32, mi: u32, s: f64) -> f64 {
    let days = days_from_civil(i64::from(y), mo, d) - UNIX_DAYS_2000;
    days as f64 * DAY_S + f64::from(h) * 3600.0 + f64::from(mi) * 60.0 + s - 43_200.0
}

/// Julian date of `t`.
pub fn jd(t: f64) -> f64 {
    JD_J2000 + t / DAY_S
}

/// Julian centuries since J2000.0.
pub fn centuries(t: f64) -> f64 {
    t / (36_525.0 * DAY_S)
}

/// Greenwich mean sidereal time (rad, 0..2π), IAU 1982 expression
/// (Vallado, *Fundamentals of Astrodynamics*, eq. 3-47), with UT1 = UTC.
pub fn gmst(t: f64) -> f64 {
    let tc = centuries(t);
    // seconds of time
    let sec =
        67_310.548_41 + (876_600.0 * 3600.0 + 8_640_184.812_866) * tc + 0.093_104 * tc * tc - 6.2e-6 * tc * tc * tc;
    (sec * (TAU / DAY_S)).rem_euclid(TAU)
}

/// UTC calendar date and time of `t`: (year, month, day, hour, minute,
/// second).
pub fn to_utc(t: f64) -> (i32, u32, u32, u32, u32, f64) {
    let since_midnight_2000 = t + 43_200.0;
    let days = (since_midnight_2000 / DAY_S).floor();
    let mut sod = since_midnight_2000 - days * DAY_S;
    if sod >= DAY_S {
        sod = DAY_S - 1e-9;
    }
    let (y, m, d) = civil_from_days(days as i64 + UNIX_DAYS_2000);
    let h = (sod / 3600.0).floor();
    let mi = ((sod - h * 3600.0) / 60.0).floor();
    let s = sod - h * 3600.0 - mi * 60.0;
    (y as i32, m, d, h as u32, mi as u32, s)
}

/// Decimal year (2000.0 = 2000-01-01 00:00 UTC; years of 365.25 days).
pub fn decimal_year(t: f64) -> f64 {
    2000.0 + (t + 43_200.0) / (365.25 * DAY_S)
}

/// ISO 8601 UTC timestamp, to the second (for reports only; allocates).
pub fn iso(t: f64) -> String {
    let (y, mo, d, h, mi, s) = to_utc(t.round());
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{:02}Z", s.round() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_epoch() {
        assert_eq!(from_utc(2000, 1, 1, 12, 0, 0.0), 0.0);
        assert_eq!(jd(0.0), JD_J2000);
        // 2028-06-01 00:00 UTC: JD 2461923.5
        let t = from_utc(2028, 6, 1, 0, 0, 0.0);
        assert!((jd(t) - 2_461_923.5).abs() < 1e-9);
    }

    #[test]
    fn utc_round_trip() {
        for &(y, mo, d, h, mi, s) in &[
            (2028, 6, 1, 0, 0, 0.0),
            (1999, 12, 31, 23, 59, 59.5),
            (2032, 2, 29, 13, 7, 1.25),
            (2036, 12, 31, 0, 0, 0.0),
        ] {
            let t = from_utc(y, mo, d, h, mi, s);
            let (y2, mo2, d2, h2, mi2, s2) = to_utc(t);
            assert_eq!((y, mo, d, h, mi), (y2, mo2, d2, h2, mi2));
            assert!((s - s2).abs() < 1e-6);
        }
        assert_eq!(iso(from_utc(2028, 6, 1, 3, 4, 5.0)), "2028-06-01T03:04:05Z");
    }

    #[test]
    fn gmst_reference_value() {
        // Vallado example 3-5: 1992-08-20 12:14:00 UT1 → GMST 152.578788°
        let t = from_utc(1992, 8, 20, 12, 14, 0.0);
        let deg = gmst(t).to_degrees();
        assert!((deg - 152.578_788).abs() < 1e-4, "{deg}");
    }

    #[test]
    fn decimal_year_values() {
        assert!((decimal_year(from_utc(2000, 1, 1, 0, 0, 0.0)) - 2000.0).abs() < 1e-12);
        assert!((decimal_year(from_utc(2028, 7, 1, 0, 0, 0.0)) - 2028.5).abs() < 0.01);
    }
}
