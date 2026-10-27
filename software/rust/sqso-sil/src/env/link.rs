//! RF link budget: 2 m downlink (beacon, transponder) and 70 cm uplink
//! (telecommand, transponder), after `docs/environment/LINK_BUDGET_ANALYSIS.md`
//! v3.0 (SQSO-LBA-001; RID RF-22), which BASELINE §7 cites.
//!
//! * Free-space loss 32.45 + 20 log f(MHz) + 20 log d(km); absorption and
//!   scintillation 1.0 dB below 20° elevation, 0.5 dB above (§2.3).
//! * Spacecraft antennas: the ISIS AntS tapes, half-wave-dipole-shaped
//!   pattern G(θ) ∝ [cos(π/2 cos θ)/sin θ]² about the element axis, 0 dBi
//!   peak as filed, the null filled to −20 dBi by the body and wings (§2.4,
//!   §5.1). Element axes: the 2 m tape along +X (BASELINE §7: across the
//!   spin axis) and the 70 cm tape along −X (`docs/environment/MASS_BUDGET.md`
//!   v3.1 §6.6, antenna option A); a dipole pattern is the same for ±.
//! * Polarization (§2.3, §5.1): to a circular ground antenna a fixed 3 dB;
//!   to a linear one the geometric mismatch of the two elements, with a
//!   Faraday rotation drifting from a random start ([`linear_pol_factor`]).
//! * Ground stations (§2.5): class A (2 m cross-Yagi 12 dBic, 500 K, circular,
//!   Doppler corrected; telecommand 70 cm cross-Yagi 14 dBic at 50 W PEP,
//!   1.5 dB line, BASELINE §7) and class B (handheld 3-element 2 m Yagi
//!   5 dBi, 1,000 K, linear, not corrected).
//! * Spacecraft: 0.5 W nominal, 1 W maximum, 1 dB line loss (§2.4);
//!   receiver 1,000 K with the +6 dB terrestrial-interference allowance of
//!   the telecommand design case (§2.4, §6.1).
//! * Demodulators (§3): the simulated AFSK bit error rate against C/N₀
//!   (`software/simulation/cdr/rf/fm_demod_sim.py`, 120,000 bits a point)
//!   with the 1.0 dB implementation loss added: beacon to a corrected
//!   station, beacon with 3.5 kHz of uncorrected Doppler, telecommand with
//!   1 kHz residual. Required C/N₀ 52.9, 55.0 and 53.0 dB-Hz. Frame success
//!   (1 − BER)^bits.

use core::f64::consts::FRAC_PI_2;

use super::math::{C_LIGHT, V3};

/// Downlink frequency used for the budget (Hz; 145.800–146.000 MHz band).
pub const DOWNLINK_HZ: f64 = 145.9e6;
/// Uplink frequency used for the budget (Hz; 435.000–437.300 MHz).
pub const UPLINK_HZ: f64 = 435.5e6;
/// Nominal transmitter power (W).
pub const TX_NOMINAL_W: f64 = 0.5;
/// Maximum transmitter power (W).
pub const TX_MAX_W: f64 = 1.0;
/// Spacecraft filter, cable and connector loss (dB).
pub const SC_LINE_LOSS_DB: f64 = 1.0;
/// Spacecraft receive system noise temperature (K).
pub const SC_RX_TSYS_K: f64 = 1000.0;
/// Terrestrial interference on the 435 MHz receiver, dB (§2.4, §6.1)
pub const SC_RX_INTERFERENCE_DB: f64 = 6.0;
/// Peak gain of the spacecraft monopoles (dBi).
pub const SC_PEAK_GAIN_DBI: f64 = 0.0;
/// Floor of the pattern null (dBi): filled by the body and wings (§2.4).
pub const SC_GAIN_FLOOR_DBI: f64 = -20.0;
/// 2 m element axis in the body frame (+X).
pub const VHF_AXIS_BODY: V3 = V3::new(1.0, 0.0, 0.0);
/// 70 cm element axis in the body frame (−X).
pub const UHF_AXIS_BODY: V3 = V3::new(-1.0, 0.0, 0.0);
/// Required C/N₀ for the beacon to a Doppler-corrected station (dB-Hz, §3.2).
pub const REQ_CN0_DATA_DBHZ: f64 = 52.9;
/// Required C/N₀ for the beacon with 3.5 kHz uncorrected Doppler (dB-Hz).
pub const REQ_CN0_UNCORRECTED_DBHZ: f64 = 55.0;
/// Required C/N₀ for telecommand, 1 kHz residual (dB-Hz).
pub const REQ_CN0_TELECOMMAND_DBHZ: f64 = 53.0;
/// C/N₀ for 12 dB SINAD on the FM voice transponder (dB-Hz, §3.2).
pub const REQ_CN0_VOICE_DBHZ: f64 = 47.4;
/// Boltzmann constant in dBm/K/Hz.
pub const K_DBM: f64 = -198.6;
/// The demodulators' implementation loss over the simulation, dB (§3.1)
pub const IMPLEMENTATION_LOSS_DB: f64 = 1.0;

/// The AFSK demodulator cases of §3.2
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demod {
    /// Beacon at a Doppler-corrected station (classes A, S)
    Corrected,
    /// Beacon with 3.5 kHz uncorrected Doppler (classes B, W, D)
    Uncorrected,
    /// Telecommand at the spacecraft, 1 kHz residual after the ground's correction
    Telecommand,
}

/// (C/N₀ dB-Hz, log10 BER) points of `fm_demod_sim.py` (the full run of
/// 2026-10-27), before the implementation loss; the repeated value at the
/// bottom of each run (one or two errors in 120,000 bits) is the
/// simulation's floor and is left out
const CURVE_CORRECTED: [(f64, f64); 8] = [
    (44.0, -0.4647),
    (46.0, -0.8665),
    (47.0, -1.1543),
    (48.0, -1.5452),
    (49.0, -2.0315),
    (50.0, -2.6440),
    (51.0, -3.3010),
    (52.0, -4.4776),
];
const CURVE_UNCORRECTED: [(f64, f64); 10] = [
    (44.0, -0.3010),
    (46.0, -0.4921),
    (47.0, -0.6882),
    (48.0, -0.9393),
    (49.0, -1.2725),
    (50.0, -1.6716),
    (51.0, -2.1726),
    (52.0, -2.8153),
    (53.0, -3.5735),
    (54.0, -4.3010),
];
const CURVE_TELECOMMAND: [(f64, f64); 8] = [
    (44.0, -0.4449),
    (46.0, -0.8447),
    (47.0, -1.1337),
    (48.0, -1.4976),
    (49.0, -1.9747),
    (50.0, -2.6073),
    (51.0, -3.4776),
    (52.0, -4.3010),
];

impl Demod {
    fn curve(self) -> &'static [(f64, f64)] {
        match self {
            Demod::Corrected => &CURVE_CORRECTED,
            Demod::Uncorrected => &CURVE_UNCORRECTED,
            Demod::Telecommand => &CURVE_TELECOMMAND,
        }
    }

    /// The required C/N₀ of §3.2 (dB-Hz)
    pub const fn required_cn0_dbhz(self) -> f64 {
        match self {
            Demod::Corrected => REQ_CN0_DATA_DBHZ,
            Demod::Uncorrected => REQ_CN0_UNCORRECTED_DBHZ,
            Demod::Telecommand => REQ_CN0_TELECOMMAND_DBHZ,
        }
    }

    /// Bit error rate at a C/N₀ (dB-Hz), with the implementation loss:
    /// interpolated in log BER, the waterfall's slope beyond the last point
    pub fn ber(self, cn0_dbhz: f64) -> f64 {
        let x = cn0_dbhz - IMPLEMENTATION_LOSS_DB;
        let c = self.curve();
        let (x0, y0) = c[0];
        if x <= x0 {
            return 0.5f64.min(10f64.powf(y0));
        }
        for w in c.windows(2) {
            let ((xa, ya), (xb, yb)) = (w[0], w[1]);
            if x <= xb && xb > xa {
                let y = ya + (yb - ya) * (x - xa) / (xb - xa);
                return 10f64.powf(y).min(0.5);
            }
        }
        // Beyond the last point: the slope of the last three
        let n = c.len();
        let (xl, yl) = c[n - 1];
        let (xf, yf) = c[n - 3];
        let slope = (yl - yf) / (xl - xf);
        10f64.powf(yl + slope * (x - xl)).min(0.5)
    }
}

/// A ground station's radio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundRadio {
    /// 2 m receive antenna gain (dBi or dBic).
    pub gain_dbi: f64,
    /// Circularly polarized (else linear: [`linear_pol_factor`]).
    pub circular: bool,
    /// Receive system noise temperature at 145.9 MHz (K).
    pub tsys_k: f64,
    /// The station corrects Doppler on receive.
    pub doppler_corrected: bool,
    /// Transmit power at 435.5 MHz (W).
    pub tx_w: f64,
    /// Transmit line loss (dB).
    pub tx_line_loss_db: f64,
    /// 70 cm transmit antenna gain (dBi or dBic).
    pub tx_gain_dbi: f64,
}

impl GroundRadio {
    /// The beacon demodulator this station uses
    pub const fn downlink_demod(&self) -> Demod {
        if self.doppler_corrected { Demod::Corrected } else { Demod::Uncorrected }
    }
}

/// Station A: telecommand / well-equipped amateur (§2.5; 50 W PEP, BASELINE §7).
pub const STATION_A: GroundRadio = GroundRadio {
    gain_dbi: 12.0,
    circular: true,
    tsys_k: 500.0,
    doppler_corrected: true,
    tx_w: 50.0,
    tx_line_loss_db: 1.5,
    tx_gain_dbi: 14.0,
};
/// Station B: portable amateur, handheld Yagi (§2.5).
pub const STATION_B: GroundRadio = GroundRadio {
    gain_dbi: 5.0,
    circular: false,
    tsys_k: 1000.0,
    doppler_corrected: false,
    tx_w: 5.0,
    tx_line_loss_db: 0.3,
    tx_gain_dbi: 8.0,
};

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

/// Linear-to-linear polarization factor (0–1): the spacecraft element
/// `element` and the ground antenna's horizontal element `h` (unit vectors,
/// one frame), seen along the line of sight `los`, with the Faraday rotation
/// `faraday` (rad) of the element's field about the line of sight
pub fn linear_pol_factor(element: V3, los: V3, h: V3, faraday: f64) -> f64 {
    let e = element - los * element.dot(los);
    let n = e.norm();
    if n < 1e-9 {
        return 0.5; // looking down the element: the floor's field, unpolarized
    }
    let e = e * (1.0 / n);
    let rotated = e * faraday.cos() + los.cross(e) * faraday.sin();
    let c = rotated.dot(h);
    (c * c).max(1e-4)
}

/// Polarization loss (dB): 3 dB to a circular station, else from the factor
pub fn pol_loss_db(circular: bool, linear_factor: f64) -> f64 {
    if circular { 3.0 } else { -10.0 * linear_factor.max(1e-4).log10() }
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
/// ground radio at `range_m` and elevation `el_rad`, with a polarization
/// loss `pol_db`, against the station's demodulator
pub fn downlink(tx_w: f64, sc_gain_dbi: f64, range_m: f64, el_rad: f64, gs: &GroundRadio, pol_db: f64) -> LinkBudget {
    let eirp = dbm(tx_w) - SC_LINE_LOSS_DB + sc_gain_dbi;
    let loss = fspl_db(DOWNLINK_HZ, range_m) + atmospheric_loss_db(el_rad) + pol_db;
    let rx = eirp - loss + gs.gain_dbi;
    let cn0 = rx - noise_density_dbm_hz(gs.tsys_k);
    let req = gs.downlink_demod().required_cn0_dbhz();
    LinkBudget { eirp_dbm: eirp, path_loss_db: loss, rx_dbm: rx, cn0_dbhz: cn0, margin_db: cn0 - req }
}

/// Uplink from a ground radio to the spacecraft receiver (antenna gain
/// `sc_gain_dbi`, polarization loss `pol_db`), telecommand demodulator,
/// with the receiver's interference allowance
pub fn uplink(gs: &GroundRadio, sc_gain_dbi: f64, range_m: f64, el_rad: f64, pol_db: f64) -> LinkBudget {
    let eirp = dbm(gs.tx_w) - gs.tx_line_loss_db + gs.tx_gain_dbi;
    let loss = fspl_db(UPLINK_HZ, range_m) + atmospheric_loss_db(el_rad) + pol_db;
    let rx = eirp - loss + sc_gain_dbi;
    let cn0 = rx - noise_density_dbm_hz(SC_RX_TSYS_K) - SC_RX_INTERFERENCE_DB;
    LinkBudget { eirp_dbm: eirp, path_loss_db: loss, rx_dbm: rx, cn0_dbhz: cn0, margin_db: cn0 - REQ_CN0_TELECOMMAND_DBHZ }
}

/// Probability that a frame of `frame_bits` bits is received without error
/// at a C/N₀ (dB-Hz) through a demodulator
pub fn frame_success(cn0_dbhz: f64, demod: Demod, frame_bits: u32) -> f64 {
    let ber = demod.ber(cn0_dbhz).min(0.5);
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

    /// SQSO-LBA-001 v3.0 §4.1, §4.2, §6.1: the beacon +7.6 dB at 5° (−3 dBi)
    /// and +0.6 dB (−10 dBi) at station A; station B −4.5 dB at 5°, +0.9 dB
    /// at 20°; telecommand at 50 W, 10°, −10 dBi, +6 dB interference +5.3 dB
    #[test]
    fn the_v3_budget() {
        let el = 5f64.to_radians();
        let a = downlink(0.5, -3.0, slant(5.0), el, &STATION_A, 3.0);
        assert!((a.rx_dbm + 111.1).abs() < 0.1, "{}", a.rx_dbm);
        assert!((a.margin_db - 7.6).abs() < 0.1, "{}", a.margin_db);
        assert!((downlink(0.5, -10.0, slant(5.0), el, &STATION_A, 3.0).margin_db - 0.6).abs() < 0.1);
        assert!((downlink(0.5, -3.0, slant(5.0), el, &STATION_B, 3.0).margin_db + 4.5).abs() < 0.1);
        let b20 = downlink(0.5, -3.0, slant(20.0), 20f64.to_radians(), &STATION_B, 3.0);
        assert!((b20.margin_db - 0.9).abs() < 0.1, "{}", b20.margin_db);
        let tc = uplink(&STATION_A, -10.0, slant(10.0), 10f64.to_radians(), 3.0);
        assert!((tc.margin_db - 5.3).abs() < 0.1, "{}", tc.margin_db);
    }

    #[test]
    fn demodulator_curves() {
        // 90 % of 200-byte (1,850-bit) frames at the required C/N0 (§3.2)
        for d in [Demod::Corrected, Demod::Uncorrected, Demod::Telecommand] {
            let p = frame_success(d.required_cn0_dbhz(), d, 1850);
            assert!(p > 0.85 && p < 0.97, "{d:?} {p}");
            assert!(frame_success(d.required_cn0_dbhz() + 3.0, d, 1850) > 0.999);
            assert!(frame_success(d.required_cn0_dbhz() - 4.0, d, 1850) < 0.01);
            assert!(d.ber(30.0) <= 0.5);
        }
        // BER 1e-4 at 51.6 dB-Hz plus the 1 dB loss (corrected)
        assert!((Demod::Corrected.ber(52.6).log10() + 4.0).abs() < 0.3);
        assert!((doppler_hz(DOWNLINK_HZ, -7060.0) - 3436.0).abs() < 5.0);
    }

    #[test]
    fn pattern_and_polarization() {
        // −3 / −6 / −10 dBi over 63 / 81 / 92 % of random orientations with
        // the null at −40 dBi; with the −20 dBi floor the same thresholds
        let mut rng = Rng::new(9);
        let n = 100_000;
        let mut c = [0u32; 3];
        for _ in 0..n {
            let g = sc_gain_dbi(VHF_AXIS_BODY, rng.unit_vector());
            for (k, th) in [-3.0, -6.0, -10.0].iter().enumerate() {
                if g >= *th {
                    c[k] += 1;
                }
            }
            assert!(g >= SC_GAIN_FLOOR_DBI);
        }
        let f: Vec<f64> = c.iter().map(|&x| f64::from(x) / f64::from(n)).collect();
        assert!((f[0] - 0.63).abs() < 0.03 && (f[1] - 0.81).abs() < 0.03 && (f[2] - 0.92).abs() < 0.03, "{f:?}");
        assert_eq!(dipole_gain_dbi(0.0), SC_GAIN_FLOOR_DBI);
        // Linear to linear: aligned 0 dB, crossed deep, Faraday turns it
        let los = V3::new(0.0, 0.0, 1.0);
        let x = V3::new(1.0, 0.0, 0.0);
        let y = V3::new(0.0, 1.0, 0.0);
        assert!((linear_pol_factor(x, los, x, 0.0) - 1.0).abs() < 1e-12);
        assert!(linear_pol_factor(x, los, y, 0.0) < 1e-3);
        assert!((linear_pol_factor(x, los, y, FRAC_PI_2) - 1.0).abs() < 1e-9);
        assert_eq!(linear_pol_factor(los, los, x, 0.0), 0.5);
        assert_eq!(pol_loss_db(true, 0.0), 3.0);
        assert!((pol_loss_db(false, 0.5) - 3.01).abs() < 0.01);
        // Averaged over a uniform angle: 3 dB (the average of §2.3)
        let mean: f64 = (0..360).map(|d| linear_pol_factor(x, los, x, f64::from(d).to_radians())).sum::<f64>() / 360.0;
        assert!((mean - 0.5).abs() < 0.01);
    }
}
