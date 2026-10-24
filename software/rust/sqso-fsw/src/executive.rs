//! The executive's own decisions (the main loop is `sqso-ffi/src/executive.rs`):
//! the stand-in power model, what the payload guard's trips are logged as,
//! the beacon measurements, and the cleaning of text put in a line field.
//!
//! @requirement SRS-F001 Initialize all subsystems within 30 seconds of power-on
//! @requirement SRS-P001 Main control loop executes within 20 ms

use crate::payload_guard::Outcome;

/// Main-loop period, ms (one tick)
pub const TICK_MS: u64 = 20;
/// Ticks the host simulation runs for (about 20 s)
pub const SIMULATION_TICKS: u64 = 1000;
/// Health check interval, ms
pub const HEALTH_CHECK_INTERVAL_MS: u64 = 10_000;
/// Ticks between saves of mission data and EPS configuration
pub const PERSISTENCE_INTERVAL_TICKS: u64 = 100;
/// Board temperature above which the health check logs a thermal fault, °C
pub const HEALTH_TEMP_LIMIT_C: f64 = 40.0;
/// State of charge below which the health check logs a power warning
pub const HEALTH_SOC_LOW: f64 = 0.30;
/// Battery voltage accepted for the beacon, V (exclusive bounds)
pub const BEACON_BV_RANGE_V: (f64, f64) = (0.0, 65.0);

/// The power model that stands in for the EPS gauge while there is no EPS
/// driver (SW-01): a 180-tick orbit, sunlit for the first 100 ticks, that
/// charges in sunlight and discharges in eclipse (faster with the payload on)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PowerModel {
    /// In sunlight this tick
    pub sunlit: bool,
    /// State of charge, 0–1
    pub soc: f64,
}

impl PowerModel {
    /// Start of mission: sunlit, 75 %
    pub const START: PowerModel = PowerModel { sunlit: true, soc: 0.75 };

    /// Advance to `tick`; returns the discharge rate used (per tick)
    pub fn step(&mut self, tick: u64, payload_on: bool) -> f64 {
        self.sunlit = (tick % 180) < 100;
        let discharge = if payload_on { 0.0040 } else { 0.0010 };
        if self.sunlit {
            self.soc += 0.0020;
        } else {
            self.soc -= discharge;
        }
        self.soc = self.soc.clamp(0.0, 1.0);
        discharge
    }

    /// Energy the payload used this tick, Wh (counted in sunlight only)
    pub fn payload_energy_wh(&self, discharge: f64, payload_on: bool) -> Option<f64> {
        (self.sunlit && payload_on).then(|| discharge * 3.7 / 3600.0)
    }
}

/// How a payload-guard trip is logged (`FaultType_t`): over-power and
/// latch-up as high current, temperatures as high temperature, the rest
/// (no telemetry, cool-down) as power. `None` for no trip.
pub fn guard_fault_type(outcome: Outcome) -> Option<u32> {
    const POWER: u32 = 1;
    const CURRENT_HIGH: u32 = 9;
    const TEMP_HIGH: u32 = 10;
    match outcome {
        Outcome::Ok => None,
        Outcome::Overpower | Outcome::Latchup => Some(CURRENT_HIGH),
        Outcome::SpreaderTemp | Outcome::JunctionTemp => Some(TEMP_HIGH),
        Outcome::NoTelemetry | Outcome::Cooldown => Some(POWER),
    }
}

/// Replace what would break a comma-separated line field: commas and bytes
/// outside printable ASCII become ';'
pub fn sanitize_field(text: &mut [u8]) {
    for b in text {
        if *b == b',' || !(0x20..=0x7E).contains(b) {
            *b = b';';
        }
    }
}

/// State of charge as a whole percentage, rounded (the beacon field)
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0–100 first
pub fn soc_percent(soc: f64) -> u8 {
    ((soc * 100.0) + 0.5).clamp(0.0, 255.0) as u8
}

/// Battery voltage in mV for the beacon, if it is a plausible reading
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // range checked first
pub fn battery_mv(volts: f64) -> Option<u16> {
    (volts > BEACON_BV_RANGE_V.0 && volts < BEACON_BV_RANGE_V.1).then_some(((volts * 1000.0) + 0.5) as u16)
}

/// Whether a reading taken at `read_ms` is usable at `now_ms`: taken at all
/// (not 0), not in the future, and at most `max_age_ms` old
pub fn fresh(read_ms: u64, now_ms: u64, max_age_ms: u64) -> bool {
    read_ms != 0 && now_ms >= read_ms && now_ms - read_ms <= max_age_ms
}

/// Milliseconds between two timer ticks for the 32-bit tick interfaces
/// (contact monitor, RF silence, deployment); saturates instead of wrapping
pub fn elapsed_ms(now_ms: u64, last_ms: u64) -> u32 {
    u32::try_from(now_ms.saturating_sub(last_ms)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orbit_is_100_sunlit_ticks_in_180() {
        let mut m = PowerModel::START;
        let sunlit = (0..180).filter(|&t| {
            m.step(t, false);
            m.sunlit
        });
        assert_eq!(sunlit.count(), 100);
        m.step(99, false);
        assert!(m.sunlit);
        m.step(100, false);
        assert!(!m.sunlit);
        m.step(180, false);
        assert!(m.sunlit);
    }

    #[test]
    fn charge_and_discharge_rates() {
        let mut m = PowerModel { sunlit: true, soc: 0.5 };
        assert_eq!(m.step(0, true), 0.0040);
        assert!((m.soc - 0.502).abs() < 1e-12);
        m.step(150, false);
        assert!((m.soc - 0.501).abs() < 1e-12);
        m.step(150, true);
        assert!((m.soc - 0.497).abs() < 1e-12);
    }

    #[test]
    fn soc_is_clamped() {
        let mut m = PowerModel { sunlit: true, soc: 0.9995 };
        m.step(0, false);
        assert_eq!(m.soc, 1.0);
        let mut m = PowerModel { sunlit: false, soc: 0.002 };
        m.step(120, true);
        assert_eq!(m.soc, 0.0);
    }

    #[test]
    fn payload_energy_only_in_sunlight_with_payload_on() {
        let sun = PowerModel { sunlit: true, soc: 0.5 };
        let ecl = PowerModel { sunlit: false, soc: 0.5 };
        assert_eq!(sun.payload_energy_wh(0.004, true), Some(0.004 * 3.7 / 3600.0));
        assert_eq!(sun.payload_energy_wh(0.004, false), None);
        assert_eq!(ecl.payload_energy_wh(0.004, true), None);
    }

    #[test]
    fn guard_trips_map_to_fault_types() {
        assert_eq!(guard_fault_type(Outcome::Ok), None);
        assert_eq!(guard_fault_type(Outcome::Overpower), Some(9));
        assert_eq!(guard_fault_type(Outcome::Latchup), Some(9));
        assert_eq!(guard_fault_type(Outcome::SpreaderTemp), Some(10));
        assert_eq!(guard_fault_type(Outcome::JunctionTemp), Some(10));
        assert_eq!(guard_fault_type(Outcome::NoTelemetry), Some(1));
        assert_eq!(guard_fault_type(Outcome::Cooldown), Some(1));
    }

    #[test]
    fn sanitize_replaces_commas_and_control_bytes() {
        let mut t = *b"a,b\nc\x7fd~ ";
        sanitize_field(&mut t);
        assert_eq!(&t, b"a;b;c;d~ ");
    }

    #[test]
    fn beacon_conversions_round_and_range_check() {
        assert_eq!(soc_percent(0.754), 75);
        assert_eq!(soc_percent(0.755), 76);
        assert_eq!(soc_percent(1.0), 100);
        assert_eq!(soc_percent(0.0), 0);
        assert_eq!(battery_mv(7.8), Some(7800));
        assert_eq!(battery_mv(7.8004), Some(7800));
        assert_eq!(battery_mv(0.0), None);
        assert_eq!(battery_mv(65.0), None);
        assert_eq!(battery_mv(64.9), Some(64_900));
    }

    #[test]
    fn freshness() {
        assert!(!fresh(0, 100, 1000));
        assert!(fresh(100, 100, 0));
        assert!(fresh(100, 1100, 1000));
        assert!(!fresh(100, 1101, 1000));
        assert!(!fresh(200, 100, 1000));
    }

    #[test]
    fn elapsed_saturates() {
        assert_eq!(elapsed_ms(120, 100), 20);
        assert_eq!(elapsed_ms(100, 120), 0);
        assert_eq!(elapsed_ms(u64::MAX, 0), u32::MAX);
    }
}
