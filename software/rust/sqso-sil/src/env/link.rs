//! RF link budget: 2 m downlink (beacon, transponder) and 70 cm uplink
//! (telecommand, transponder), after `docs/environment/LINK_BUDGET_ANALYSIS.md`
//! v2.0 (SQSO-LBA-001), which BASELINE §7 cites.
//!
//! * Free-space loss 32.45 + 20 log f(MHz) + 20 log d(km); absorption 1.0 dB
//!   below 20° elevation, 0.5 dB above (§2.6).
//! * Spacecraft antennas: deployable λ/4 monopoles modelled as a half-wave
//!   dipole pattern G(θ) ∝ [cos(π/2 cos θ)/sin θ]² about the element axis,
//!   peak 0 dBi as filed (§2.4), with the null along the element floored at
//!   −40 dBi. Element axis: −Z (`hardware/rf/ANTENNA_DESIGN.md` §10.3, the 2 m
//!   element deploys along −Z from a −Z corner). The 70 cm element's
//!   direction is not documented; **assumed** also −Z.
//! * Ground stations A (cross-Yagi 12 dBic, 500 K, 25 W) and B (portable
//!   5 dBi Yagi, 1,000 K, 5 W) from §2.5; 3 dB polarization loss for both.
//! * Spacecraft: 0.5 W nominal, 1 W maximum, 1 dB filter and cable loss
//!   (§3.1); receiver 1,000 K (§4.3).
//! * Required C/N₀: 52.8 dB-Hz for 1200 bps AFSK on FM (C/N 12 dB in
//!   12 kHz), 54.0 dB-Hz for FM voice (12 dB in 16 kHz) (§2.3).
//! * Frame success: the margin is referred to the required C/N₀, taken as the
//!   point where the post-discriminator bit error rate is 1e-5; BER =
//!   ½ exp(−γ₀ 10^(margin/10)) with γ₀ = ln(0.5/1e-5) (non-coherent FSK form;
//!   steep like the FM threshold), frame success (1 − BER)^bits. For a
//!   200-byte frame: 98 % at 0 dB, 86 % at −1 dB, 42 % at −2 dB, 3 % at −3 dB.
//!   **Assumption** until the packet-error-rate test of §7.1.

use core::f64::consts::FRAC_PI_2;

use super::math::{C_LIGHT, V3};

/// Downlink frequency used for the budget (Hz; 145.800–146.000 MHz band).
pub const DOWNLINK_HZ: f64 = 145.9e6;
/// Uplink frequency used for the budget (Hz; 435–438 MHz band).
pub const UPLINK_HZ: f64 = 435.5e6;
/// Nominal transmitter power (W).
pub const TX_NOMINAL_W: f64 = 0.5;
/// Maximum transmitter power (W).
pub const TX_MAX_W: f64 = 1.0;
/// Spacecraft filter, cable and connector loss (dB).
pub const SC_LINE_LOSS_DB: f64 = 1.0;
/// Spacecraft receive system noise temperature (K).
pub const SC_RX_TSYS_K: f64 = 1000.0;
/// Peak gain of the spacecraft monopoles (dBi).
pub const SC_PEAK_GAIN_DBI: f64 = 0.0;
/// Floor of the pattern null (dBi).
pub const SC_GAIN_FLOOR_DBI: f64 = -40.0;
/// 2 m element axis in the body frame.
pub const VHF_AXIS_BODY: V3 = V3::new(0.0, 0.0, -1.0);
/// 70 cm element axis in the body frame (assumed).
pub const UHF_AXIS_BODY: V3 = V3::new(0.0, 0.0, -1.0);
/// Required C/N₀ for 1200 bps AFSK telemetry and telecommand (dB-Hz).
pub const REQ_CN0_DATA_DBHZ: f64 = 52.8;
/// Required C/N₀ for the FM voice transponder (dB-Hz).
pub const REQ_CN0_VOICE_DBHZ: f64 = 54.0;
/// Boltzmann constant in dBm/K/Hz.
pub const K_DBM: f64 = -198.6;
/// BER at zero margin.
pub const BER_AT_THRESHOLD: f64 = 1e-5;

/// A ground station's radio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundRadio {
    /// Antenna gain (dBi or dBic).
    pub gain_dbi: f64,
    /// Polarization loss against the linear spacecraft antenna (dB).
    pub pol_loss_db: f64,
    /// Receive system noise temperature at 145.9 MHz (K).
    pub tsys_k: f64,
    /// Transmit power at 435.5 MHz (W).
    pub tx_w: f64,
    /// Transmit line loss (dB).
    pub tx_line_loss_db: f64,
}

/// Station A: telecommand / well-equipped amateur (§2.5).
pub const STATION_A: GroundRadio =
    GroundRadio { gain_dbi: 12.0, pol_loss_db: 3.0, tsys_k: 500.0, tx_w: 25.0, tx_line_loss_db: 1.5 };
/// Station B: portable amateur (§2.5).
pub const STATION_B: GroundRadio =
    GroundRadio { gain_dbi: 5.0, pol_loss_db: 3.0, tsys_k: 1000.0, tx_w: 5.0, tx_line_loss_db: 0.5 };

/// Free-space path loss (dB).
pub fn fspl_db(freq_hz: f64, range_m: f64) -> f64 {
    32.45 + 20.0 * (freq_hz * 1e-6).log10() + 20.0 * (range_m * 1e-3).max(1e-6).log10()
}

/// Atmospheric and ionospheric allowance (dB) at an elevation (rad).
pub fn atmospheric_loss_db(el_rad: f64) -> f64 {
    if el_rad < 20f64.to_radians() { 1.0 } else { 0.5 }
}

/// Gain (dBi) of the dipole-shaped pattern at `off_axis` (rad) from the
/// element axis.
pub fn dipole_gain_dbi(off_axis: f64) -> f64 {
    let s = off_axis.sin();
    if s.abs() < 1e-6 {
        return SC_GAIN_FLOOR_DBI;
    }
    let f = (FRAC_PI_2 * off_axis.cos()).cos() / s;
    (SC_PEAK_GAIN_DBI + 10.0 * (f * f).max(1e-12).log10()).max(SC_GAIN_FLOOR_DBI)
}

/// Spacecraft antenna gain (dBi) towards a station, for the element axis and
/// the direction to the station, both in the body frame.
pub fn sc_gain_dbi(axis_body: V3, to_station_body: V3) -> f64 {
    dipole_gain_dbi(axis_body.angle(to_station_body))
}

/// Noise density (dBm/Hz) of a system temperature.
pub fn noise_density_dbm_hz(t_k: f64) -> f64 {
    K_DBM + 10.0 * t_k.log10()
}

/// Result of one link evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinkBudget {
    /// EIRP (dBm).
    pub eirp_dbm: f64,
    /// Free-space plus atmospheric plus polarization loss (dB).
    pub path_loss_db: f64,
    /// Received carrier power (dBm).
    pub rx_dbm: f64,
    /// Carrier-to-noise density (dB-Hz).
    pub cn0_dbhz: f64,
    /// Margin over the required C/N₀ (dB).
    pub margin_db: f64,
}

fn dbm(w: f64) -> f64 {
    10.0 * (w.max(1e-12) * 1e3).log10()
}

/// Downlink from the spacecraft (`tx_w`, antenna gain `sc_gain_dbi`) to a
/// ground radio at `range_m` and elevation `el_rad`, against the required
/// C/N₀ `req_cn0_dbhz`.
pub fn downlink(
    tx_w: f64,
    sc_gain_dbi: f64,
    range_m: f64,
    el_rad: f64,
    gs: &GroundRadio,
    req_cn0_dbhz: f64,
) -> LinkBudget {
    let eirp = dbm(tx_w) - SC_LINE_LOSS_DB + sc_gain_dbi;
    let loss = fspl_db(DOWNLINK_HZ, range_m) + atmospheric_loss_db(el_rad) + gs.pol_loss_db;
    let rx = eirp - loss + gs.gain_dbi;
    let cn0 = rx - noise_density_dbm_hz(gs.tsys_k);
    LinkBudget { eirp_dbm: eirp, path_loss_db: loss, rx_dbm: rx, cn0_dbhz: cn0, margin_db: cn0 - req_cn0_dbhz }
}

/// Uplink from a ground radio to the spacecraft receiver (antenna gain
/// `sc_gain_dbi`).
pub fn uplink(gs: &GroundRadio, sc_gain_dbi: f64, range_m: f64, el_rad: f64, req_cn0_dbhz: f64) -> LinkBudget {
    let eirp = dbm(gs.tx_w) - gs.tx_line_loss_db + gs.gain_dbi;
    let loss = fspl_db(UPLINK_HZ, range_m) + atmospheric_loss_db(el_rad) + gs.pol_loss_db;
    let rx = eirp - loss + sc_gain_dbi;
    let cn0 = rx - noise_density_dbm_hz(SC_RX_TSYS_K);
    LinkBudget { eirp_dbm: eirp, path_loss_db: loss, rx_dbm: rx, cn0_dbhz: cn0, margin_db: cn0 - req_cn0_dbhz }
}

/// Bit error rate after the FM discriminator at a margin (dB) over the
/// required C/N₀.
pub fn bit_error_rate(margin_db: f64) -> f64 {
    let gamma0 = (0.5 / BER_AT_THRESHOLD).ln();
    0.5 * (-gamma0 * 10f64.powf(margin_db / 10.0)).exp()
}

/// Probability that a frame of `frame_bits` bits is received without error
/// at a margin (dB).
pub fn frame_success_prob(margin_db: f64, frame_bits: u32) -> f64 {
    let ber = bit_error_rate(margin_db).min(0.5);
    (f64::from(frame_bits) * (-ber).ln_1p()).exp()
}

/// Doppler shift (Hz) at a carrier frequency for a range rate (m/s,
/// positive when the range grows).
pub fn doppler_hz(freq_hz: f64, range_rate_m_s: f64) -> f64 {
    -freq_hz * range_rate_m_s / C_LIGHT
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::math::Rng;

    /// Slant range at elevation `el` from 500 km (LINK_BUDGET §2.1).
    fn slant(el_deg: f64) -> f64 {
        let (re, h): (f64, f64) = (6378e3, 500e3);
        let e = el_deg.to_radians();
        ((re + h).powi(2) - (re * e.cos()).powi(2)).sqrt() - re * e.sin()
    }

    #[test]
    fn path_loss_table() {
        assert!((slant(5.0) / 1e3 - 2078.0).abs() < 2.0);
        assert!((fspl_db(DOWNLINK_HZ, slant(5.0)) - 142.1).abs() < 0.05);
        assert!((fspl_db(UPLINK_HZ, slant(90.0)) - 139.2).abs() < 0.05);
        assert!((noise_density_dbm_hz(500.0) + 171.6).abs() < 0.05);
    }

    /// BASELINE §7: the 1200 bps beacon closes at 0.5 W with ≈ +6 dB at 5°
    /// for a reference station; SQSO-LBA-001 §3.4 gives +7.7 dB (−3 dBi,
    /// 63 % of orientations) and +4.7 dB (−6 dBi, 81 %).
    #[test]
    fn beacon_margin_at_five_degrees() {
        let el = 5f64.to_radians();
        let nominal = downlink(0.5, -3.0, slant(5.0), el, &STATION_A, REQ_CN0_DATA_DBHZ);
        let conservative = downlink(0.5, -6.0, slant(5.0), el, &STATION_A, REQ_CN0_DATA_DBHZ);
        println!(
            "beacon 0.5 W, 5°, station A: rx {:.1} dBm, C/N0 {:.1} dB-Hz, margin {:+.1} dB (−3 dBi) / {:+.1} dB (−6 dBi)",
            nominal.rx_dbm, nominal.cn0_dbhz, nominal.margin_db, conservative.margin_db
        );
        assert!((nominal.rx_dbm + 111.1).abs() < 0.1);
        assert!((nominal.margin_db - 7.7).abs() < 0.1);
        assert!((conservative.margin_db - 4.7).abs() < 0.1);
        assert!((0.5 * (nominal.margin_db + conservative.margin_db) - 6.0).abs() < 1.0);
        // station B at 5° does not close at 0.5 W (−2.3 dB), at 30° it does
        let b = downlink(0.5, -3.0, slant(5.0), el, &STATION_B, REQ_CN0_DATA_DBHZ);
        assert!((b.margin_db + 2.3).abs() < 0.1);
        // telecommand, station A, −10 dBi: +4.7 dB (§4.5)
        let tc = uplink(&STATION_A, -10.0, slant(5.0), el, REQ_CN0_DATA_DBHZ);
        assert!((tc.margin_db - 4.7).abs() < 0.1, "{}", tc.margin_db);
    }

    #[test]
    fn pattern_coverage_matches_the_link_budget() {
        // fraction of random orientations with gain ≥ −3 / −6 / −10 dBi:
        // 63 / 81 / 92 % (§2.4)
        let mut rng = Rng::new(9);
        let n = 200_000;
        let mut c = [0u32; 3];
        for _ in 0..n {
            let g = sc_gain_dbi(VHF_AXIS_BODY, rng.unit_vector());
            for (k, th) in [-3.0, -6.0, -10.0].iter().enumerate() {
                if g >= *th {
                    c[k] += 1;
                }
            }
        }
        let f: Vec<f64> = c.iter().map(|&x| f64::from(x) / f64::from(n)).collect();
        println!("pattern coverage: {:.3} {:.3} {:.3}", f[0], f[1], f[2]);
        assert!((f[0] - 0.63).abs() < 0.03 && (f[1] - 0.81).abs() < 0.03 && (f[2] - 0.92).abs() < 0.03);
        assert!(dipole_gain_dbi(FRAC_PI_2).abs() < 1e-12);
        assert_eq!(dipole_gain_dbi(0.0), SC_GAIN_FLOOR_DBI);
    }

    #[test]
    fn frame_success_curve() {
        let bits = 1600;
        assert!(frame_success_prob(0.0, bits) > 0.97);
        assert!(frame_success_prob(3.0, bits) > 0.999_999);
        let m1 = frame_success_prob(-1.0, bits);
        let m2 = frame_success_prob(-2.0, bits);
        assert!(m1 > 0.8 && m1 < 0.9 && m2 > 0.35 && m2 < 0.5, "{m1} {m2}");
        assert!(frame_success_prob(-6.0, bits) < 1e-6);
        assert!((doppler_hz(DOWNLINK_HZ, -7060.0) - 3436.0).abs() < 5.0);
    }
}
