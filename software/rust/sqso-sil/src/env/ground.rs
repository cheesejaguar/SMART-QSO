//! Ground stations and pass geometry.
//!
//! A default network of nine amateur and university stations: one near the
//! PI's call sign KK6GDB (San Francisco Bay Area; Santa Clara University is a
//! prospective telecommand partner, BASELINE §1) and SatNOGS-like receive
//! coverage in North America, Europe, Japan, Australia, South America and
//! southern Africa. The locations are representative city coordinates, not
//! surveyed sites; only the Bay Area station is marked for telecommand (the
//! second, geographically separated telecommand station is TBD, decision
//! D1). Minimum elevation for a usable pass: 5° (LINK_BUDGET_ANALYSIS §5.2).

use super::earth::{eci_to_ecef, geodetic_to_ecef};
use super::link::{GroundRadio, STATION_A, STATION_B};
use super::math::{OMEGA_EARTH, V3};
use super::time::gmst;

/// Minimum elevation for a usable pass (rad): 5°.
pub const MIN_ELEVATION_RAD: f64 = 5.0 * core::f64::consts::PI / 180.0;

/// A ground station.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Station {
    /// Name, for reports.
    pub name: &'static str,
    /// Geodetic latitude (rad).
    pub lat_rad: f64,
    /// Longitude (rad, east positive).
    pub lon_rad: f64,
    /// Height above the WGS84 ellipsoid (m).
    pub alt_m: f64,
    /// Minimum usable elevation (rad).
    pub min_el_rad: f64,
    /// Radio and antenna (station class A or B of the link budget).
    pub radio: GroundRadio,
    /// May send telecommands (designated station).
    pub telecommand: bool,
}

impl Station {
    /// A station from degrees.
    pub const fn deg(
        name: &'static str,
        lat_deg: f64,
        lon_deg: f64,
        alt_m: f64,
        radio: GroundRadio,
        tc: bool,
    ) -> Station {
        Station {
            name,
            lat_rad: lat_deg.to_radians(),
            lon_rad: lon_deg.to_radians(),
            alt_m,
            min_el_rad: MIN_ELEVATION_RAD,
            radio,
            telecommand: tc,
        }
    }

    /// ECEF position (m).
    pub fn ecef(&self) -> V3 {
        geodetic_to_ecef(self.lat_rad, self.lon_rad, self.alt_m)
    }

    /// Local east, north and up unit vectors in ECEF.
    pub fn enu(&self) -> (V3, V3, V3) {
        let (sl, cl) = self.lat_rad.sin_cos();
        let (so, co) = self.lon_rad.sin_cos();
        (V3::new(-so, co, 0.0), V3::new(-sl * co, -sl * so, cl), V3::new(cl * co, cl * so, sl))
    }
}

/// The default network.
pub const DEFAULT_NETWORK: [Station; 9] = [
    Station::deg("Santa Clara CA (KK6GDB area)", 37.349, -121.938, 30.0, STATION_A, true),
    Station::deg("Morehead KY", 38.190, -83.440, 300.0, STATION_A, false),
    Station::deg("Fairbanks AK", 64.860, -147.850, 150.0, STATION_A, false),
    Station::deg("Delft NL", 51.999, 4.373, 0.0, STATION_A, false),
    Station::deg("Athens GR", 37.980, 23.730, 100.0, STATION_B, false),
    Station::deg("Tokyo JP", 35.710, 139.760, 40.0, STATION_A, false),
    Station::deg("Adelaide AU", -34.920, 138.600, 50.0, STATION_B, false),
    Station::deg("Sao Jose dos Campos BR", -23.210, -45.860, 600.0, STATION_B, false),
    Station::deg("Cape Town ZA", -33.930, 18.470, 10.0, STATION_B, false),
];

/// A station with its ECEF position and local frame precomputed (for
/// per-second use).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Site {
    /// The station.
    pub station: Station,
    /// ECEF position (m).
    pub ecef: V3,
    east: V3,
    north: V3,
    up: V3,
}

impl Site {
    /// Precompute a station.
    pub fn new(station: Station) -> Site {
        let (east, north, up) = station.enu();
        Site { station, ecef: station.ecef(), east, north, up }
    }

    /// Elevation (rad), azimuth (rad, from north through east, 0..2π) and
    /// range (m) of a satellite at ECEF position `r_sat_ecef`.
    pub fn look_ecef(&self, r_sat_ecef: V3) -> (f64, f64, f64) {
        let d = r_sat_ecef - self.ecef;
        let range = d.norm();
        let (e, n, u) = (d.dot(self.east), d.dot(self.north), d.dot(self.up));
        let el = (u / range).clamp(-1.0, 1.0).asin();
        let az = e.atan2(n).rem_euclid(core::f64::consts::TAU);
        (el, az, range)
    }

    /// Range rate (m/s, positive receding) of a satellite with ECI position
    /// and velocity at `t`.
    pub fn range_rate(&self, r_eci: V3, v_eci: V3, t: f64) -> f64 {
        let g = gmst(t);
        let st = self.ecef.rot_z(g);
        let vst = V3::new(0.0, 0.0, OMEGA_EARTH).cross(st);
        let d = r_eci - st;
        d.unit().dot(v_eci - vst)
    }
}

/// ECI position of a station at `t` (m).
pub fn station_eci(station: &Station, t: f64) -> V3 {
    station.ecef().rot_z(gmst(t))
}

/// Elevation (rad), azimuth (rad) and range (m) of a satellite at ECI
/// position `r_sat_eci` seen from a station at `t`.
pub fn look(station: &Station, r_sat_eci: V3, t: f64) -> (f64, f64, f64) {
    Site::new(*station).look_ecef(eci_to_ecef(r_sat_eci, t))
}

/// Default network with the local frames precomputed.
pub fn default_sites() -> [Site; 9] {
    DEFAULT_NETWORK.map(Site::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::orbit::Orbit;
    use crate::env::time::from_utc;

    #[test]
    fn zenith_and_horizon() {
        let st = DEFAULT_NETWORK[0];
        let site = Site::new(st);
        let (_, _, up) = st.enu();
        let (el, _, range) = site.look_ecef(site.ecef + up * 500e3);
        assert!((el.to_degrees() - 90.0).abs() < 1e-9 && (range - 500e3).abs() < 1e-6);
        let (e, n, _) = st.enu();
        let (el, az, _) = site.look_ecef(site.ecef + e * 1000e3);
        assert!(el.abs() < 1e-9 && (az.to_degrees() - 90.0).abs() < 1e-6);
        let (_, az, _) = site.look_ecef(site.ecef + n * 1000e3 + up * 10e3);
        assert!(az.to_degrees().abs() < 1e-3 || (az.to_degrees() - 360.0).abs() < 1e-3);
        // the ECI form agrees
        let t = from_utc(2028, 6, 1, 0, 0, 0.0);
        let r = station_eci(&st, t) + up.rot_z(gmst(t)) * 600e3;
        assert!((look(&st, r, t).0.to_degrees() - 90.0).abs() < 1e-6);
    }

    #[test]
    fn passes_per_day_and_doppler() {
        let t0 = from_utc(2028, 6, 1, 0, 0, 0.0);
        let o = Orbit::sso(500e3, 10.5, t0);
        let sites = default_sites();
        let mut visible_s = [0u32; 9];
        let mut passes = [0u32; 9];
        let mut was = [false; 9];
        let mut max_rr: f64 = 0.0;
        for k in 0..86_400u32 / 10 {
            let t = t0 + 10.0 * f64::from(k);
            let (r, v) = o.state_at(t);
            let re = eci_to_ecef(r, t);
            for (j, s) in sites.iter().enumerate() {
                let (el, _, _) = s.look_ecef(re);
                let vis = el >= s.station.min_el_rad;
                if vis {
                    visible_s[j] += 10;
                    max_rr = max_rr.max(s.range_rate(r, v, t).abs());
                }
                if vis && !was[j] {
                    passes[j] += 1;
                }
                was[j] = vis;
            }
        }
        println!("passes/day above 5°: {passes:?}; minutes {:?}", visible_s.map(|s| s / 60));
        // a mid-latitude station sees 3–6 passes a day from a 500 km SSO;
        // Fairbanks more
        assert!((3..=7).contains(&passes[0]), "{passes:?}");
        assert!(passes[2] > passes[0]);
        // maximum range rate ≈ 7 km/s (LINK_BUDGET §5.3: 7.06 km/s at the horizon)
        assert!(max_rr > 6000.0 && max_rr < 7600.0, "{max_rr}");
    }
}
