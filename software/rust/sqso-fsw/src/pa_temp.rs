//! The transmitter temperature's plausibility (RID SMA-B19, SQSO-ICX-001 D
//! §6 row 36): TX_TEMP (PA0) is the PA's sensor through 10 kΩ, with a 1 MΩ
//! pull-down on the OBC so an open line reads 0 V, −50 °C.
//!
//! The downlink holds every frame above 70 °C (`downlink::TEMP_LIMIT_C`).
//! A failed sensor must not hold it for ever, nor let a hot PA through: a
//! reading at or below [`MIN_C`] (the open line), above [`MAX_C`], rising
//! faster than [`MAX_RISE_C_PER_S`] (faster than the PA can heat) or more
//! than [`MAX_BOARD_DIFF_C`] from the OBC board's temperature is a sensor
//! fault. While the sensor is faulted the downlink uses the board
//! temperature plus [`BOARD_MARGIN_C`] instead, with its duty limit as
//! before; [`GOOD_TO_CLEAR`] plausible readings in a row clear the fault.
//!
//! @requirement SRS-F161 TX_TEMP at or below -50 °C or above +125 °C, rising faster than 5 °C/s (over at least 15 s) or 60 °C from the board temperature is a sensor fault: the downlink then uses the board temperature plus 15 °C

/// At or below this, the open line (0 V), °C
pub const MIN_C: f64 = -50.0;
/// Above this, not a PA temperature, °C
pub const MAX_C: f64 = 125.0;
/// Faster than the PA can heat (1 W dissipated in its few grams), °C/s
pub const MAX_RISE_C_PER_S: f64 = 5.0;
/// The rise is measured against a plausible reading at least this old (or
/// over this long, if younger), ms: a step of up to 75 °C between two
/// readings is not by itself a fault
pub const RATE_WINDOW_MS: u64 = 15_000;
/// Farther than this from the board temperature, °C
pub const MAX_BOARD_DIFF_C: f64 = 60.0;
/// Added to the board temperature while the sensor is faulted, °C (the PA's
/// rise over the board at the 25 % duty limit, SQSO-RF-TX-001 §6.3)
pub const BOARD_MARGIN_C: f64 = 15.0;
/// Plausible readings in a row that clear a fault
pub const GOOD_TO_CLEAR: u32 = 10;

/// The sensor's state
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PaTemp {
    /// The reference for the rise: a plausible reading, when, ms, and the
    /// value, °C (renewed once it is [`RATE_WINDOW_MS`] old)
    last: Option<(u64, f64)>,
    good_run: u32,
    /// The sensor is faulted
    pub faulted: bool,
    /// Faults since boot (episodes)
    pub faults: u32,
}

impl PaTemp {
    /// At boot: trusted
    pub const fn new() -> PaTemp {
        PaTemp { last: None, good_run: 0, faulted: false, faults: 0 }
    }

    /// Whether `pa_c` at `now_ms` is plausible against the last plausible
    /// reading and the board's `board_c`
    fn plausible(&self, now_ms: u64, pa_c: f64, board_c: f64) -> bool {
        if !pa_c.is_finite() || pa_c <= MIN_C || pa_c > MAX_C {
            return false;
        }
        if board_c.is_finite() && (pa_c - board_c).abs() > MAX_BOARD_DIFF_C {
            return false;
        }
        match self.last {
            Some((t, prev)) => {
                #[allow(clippy::cast_precision_loss)] // ms between readings
                let dt_s = now_ms.saturating_sub(t).max(RATE_WINDOW_MS) as f64 / 1000.0;
                (pa_c - prev) / dt_s <= MAX_RISE_C_PER_S
            }
            None => true,
        }
    }

    /// The temperature the downlink is to use at `now_ms`: the PA's reading
    /// `pa_c`, or with the sensor faulted the board's `board_c` plus
    /// [`BOARD_MARGIN_C`] (not a number if neither is available)
    pub fn select(&mut self, now_ms: u64, pa_c: f64, board_c: f64) -> f64 {
        if self.plausible(now_ms, pa_c, board_c) {
            if self.last.is_none_or(|(t, _)| now_ms.saturating_sub(t) >= RATE_WINDOW_MS) {
                self.last = Some((now_ms, pa_c));
            }
            self.good_run = self.good_run.saturating_add(1);
            if self.faulted && self.good_run >= GOOD_TO_CLEAR {
                self.faulted = false;
            }
        } else {
            self.good_run = 0;
            if !self.faulted {
                self.faulted = true;
                self.faults = self.faults.saturating_add(1);
            }
        }
        if self.faulted { board_c + BOARD_MARGIN_C } else { pa_c }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plausible_reading_is_used() {
        let mut p = PaTemp::new();
        assert!((p.select(0, 35.0, 20.0) - 35.0).abs() < 1e-9);
        assert!((p.select(1_000, 39.0, 20.0) - 39.0).abs() < 1e-9);
        assert!(!p.faulted);
        assert_eq!(PaTemp::default(), PaTemp::new());
    }

    #[test]
    fn the_open_line_falls_back_to_the_board() {
        let mut p = PaTemp::new();
        assert!((p.select(0, MIN_C, 22.0) - (22.0 + BOARD_MARGIN_C)).abs() < 1e-9);
        assert!(p.faulted);
        assert_eq!(p.faults, 1);
        // Out of range high, and not a number
        let mut q = PaTemp::new();
        assert!((q.select(0, 125.1, 22.0) - 37.0).abs() < 1e-9);
        let mut r = PaTemp::new();
        assert!(r.select(0, f64::NAN, f64::NAN).is_nan());
        assert!(r.faulted);
    }

    #[test]
    fn a_jump_and_a_disagreement_are_faults_that_clear() {
        let mut p = PaTemp::new();
        p.select(0, 30.0, 60.0);
        // 80 °C in 2 s: faster than the PA heats (more than 75 °C in the window)
        assert!((p.select(2_000, 110.0, 60.0) - 75.0).abs() < 1e-9);
        assert!(p.faulted);
        for i in 0..GOOD_TO_CLEAR - 1 {
            p.select(3_000 + u64::from(i) * 1_000, 31.0, 25.0);
            assert!(p.faulted);
        }
        assert!((p.select(20_000, 31.0, 25.0) - 31.0).abs() < 1e-9);
        assert!(!p.faulted);
        // 70 °C within the window is a step the PA may show (a test)
        let mut s = PaTemp::new();
        s.select(0, 25.0, 25.0);
        assert!((s.select(500, 75.0, 25.0) - 75.0).abs() < 1e-9);
        assert!(!s.faulted);
        // 61 °C above the board
        assert!((p.select(30_000, 86.0, 25.0) - 40.0).abs() < 1e-9);
        assert_eq!(p.faults, 2);
        // The reference is renewed once it is a window old
        let mut q = PaTemp::new();
        q.select(5_000, 30.0, 25.0);
        q.select(25_000, 40.0, 25.0);
        assert!((q.select(26_000, 120.0, 70.0) - 85.0).abs() < 1e-9, "80 °C in 1 s after the renewal");
    }

    #[test]
    fn a_hot_pa_is_not_hidden() {
        // A real over-temperature, reached at the PA's rate, is used as is
        let mut p = PaTemp::new();
        let mut t = 40.0;
        for s in 0..20u64 {
            assert!((p.select(s * 1_000, t, 30.0) - t).abs() < 1e-9);
            t += 2.0;
        }
        assert!(!p.faulted);
        assert!(t > 70.0);
    }
}
