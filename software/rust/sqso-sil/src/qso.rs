//! Amateur demand on the FM transponder: how many stations under the
//! footprint want a contact, by region and local time.
//!
//! The footprint of a 500 km spacecraft at 0° elevation is about 2500 km in
//! radius. Licensed amateurs are concentrated in North America, Europe and
//! Japan; FM satellite operators follow (AMSAT/SatNOGS activity maps).
//! Activity peaks in the local evening and at weekends. The model gives an
//! expected number of stations calling, from which the transponder's
//! keyed fraction (a carrier on the uplink) and the contacts it carries
//! follow. Regions and weights are assumptions for the simulation.

/// A region of operators: centre (lat, lon degrees), radius km, relative
/// weight (stations calling at peak with the region fully in view)
const REGIONS: [(f64, f64, f64, f64, &str); 10] = [
    (39.0, -98.0, 2200.0, 6.0, "North America"),
    (50.0, 10.0, 1800.0, 6.0, "Europe"),
    (36.0, 138.0, 900.0, 4.0, "Japan"),
    (37.0, 127.5, 400.0, 1.0, "Korea"),
    (-25.0, 134.0, 1800.0, 1.0, "Australia"),
    (-15.0, -55.0, 2200.0, 1.5, "South America"),
    (22.0, 79.0, 1600.0, 1.0, "India"),
    (55.0, 50.0, 2000.0, 1.0, "Russia"),
    (-30.0, 24.0, 900.0, 0.5, "Southern Africa"),
    (-41.0, 174.0, 500.0, 0.5, "New Zealand"),
];

/// Footprint radius of the spacecraft at `alt_km` for a minimum operator
/// elevation of 10°, km
pub fn footprint_km(alt_km: f64) -> f64 {
    let re = 6378.0;
    let el = 10f64.to_radians();
    // Earth central angle to the horizon at elevation el
    let lambda = (re / (re + alt_km) * el.cos()).acos() - el;
    re * lambda
}

fn great_circle_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let (p1, p2) = (lat1.to_radians(), lat2.to_radians());
    let dl = (lon2 - lon1).to_radians();
    let c = (p1.sin() * p2.sin() + p1.cos() * p2.cos() * dl.cos()).clamp(-1.0, 1.0);
    6371.0 * c.acos()
}

/// Local activity factor at `local_hour` (0–24): low overnight, evening peak
pub fn diurnal(local_hour: f64) -> f64 {
    let h = local_hour.rem_euclid(24.0);
    let evening = (-((h - 20.0) / 2.5).powi(2)).exp();
    let morning = 0.5 * (-((h - 8.0) / 2.0).powi(2)).exp();
    let day = if (7.0..23.0).contains(&h) { 0.25 } else { 0.05 };
    (day + morning + evening).min(1.0)
}

/// Expected stations calling with the sub-satellite point at (lat, lon),
/// altitude `alt_km`, at `utc_hour`; and whether it is a weekend (more)
pub fn stations_calling(lat: f64, lon: f64, alt_km: f64, utc_hour: f64, weekend: bool) -> f64 {
    let fp = footprint_km(alt_km);
    let mut n = 0.0;
    for (rlat, rlon, rkm, w, _) in REGIONS {
        let d = great_circle_km(lat, lon, rlat, rlon);
        // Overlap of the footprint with the region, as a smooth fraction
        let overlap = ((fp + rkm - d) / (2.0 * rkm.min(fp))).clamp(0.0, 1.0);
        if overlap > 0.0 {
            let local = utc_hour + rlon / 15.0;
            n += w * overlap * diurnal(local);
        }
    }
    if weekend { n * 1.5 } else { n }
}

/// Fraction of the time the uplink carries a carrier with `n` stations
/// calling (an FM channel: one at a time; short exchanges with gaps)
pub fn carrier_fraction(n: f64) -> f64 {
    1.0 - (-0.35 * n).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footprint_is_about_1600_km_at_500_km_for_10_degrees() {
        let f = footprint_km(500.0);
        assert!(f > 1400.0 && f < 1800.0, "{f}");
    }

    #[test]
    fn demand_over_europe_in_the_evening_exceeds_the_mid_pacific() {
        let eu = stations_calling(50.0, 10.0, 500.0, 19.0, false);
        let pac = stations_calling(0.0, -150.0, 500.0, 19.0, false);
        assert!(eu > 2.0, "{eu}");
        assert!(pac < 0.1, "{pac}");
        assert!(carrier_fraction(eu) > 0.5);
        assert!(carrier_fraction(0.0) == 0.0);
    }
}
