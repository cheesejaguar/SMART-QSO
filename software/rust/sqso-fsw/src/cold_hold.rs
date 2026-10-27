//! The wings-stowed cold hold (RID THM-03; THERMAL_ANALYSIS 2.2 §6.8).
//!
//! With the wings stowed the array's generation settles the bus near 14 V,
//! where the battery heater (0.70 W at 12 V, 207 Ω) runs at 100 % duty and
//! the pack holds about +2.6 °C with no margin; below +1.1 °C the hardware
//! refuses charging. While SOLAR_SENSE reads the wings not deployed and the
//! battery's minimum over the last orbit is below [`HOLD_BELOW_C`], the
//! mode is held at SAFE rather than IDLE, so the heater gets the whole
//! surplus. The hold ends when the wings read deployed or the orbit
//! minimum reaches [`RELEASE_AT_C`].
//!
//! The orbit minimum is kept as one minimum per minute over the last
//! [`ORBIT_MIN`] minutes; until an orbit has been seen it is the minimum so
//! far.
//!
//! @requirement SRS-F163 Wings stowed and the battery's orbit minimum below +3 °C: the mode is held at SAFE (not IDLE) until the wings read deployed or the orbit minimum reaches +4 °C

/// The hold starts below this orbit minimum, °C
pub const HOLD_BELOW_C: f64 = 3.0;
/// And ends at this orbit minimum, °C (1 °C of hysteresis)
pub const RELEASE_AT_C: f64 = 4.0;
/// Minutes of one orbit (the 95-minute LEOP period)
pub const ORBIT_MIN: usize = 95;
const MINUTE_MS: u64 = 60_000;

/// The battery's orbit minimum and the hold
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColdHold {
    /// One minimum per minute, the oldest overwritten
    minutes: [f64; ORBIT_MIN],
    /// Minutes kept (up to [`ORBIT_MIN`]) and the next slot
    kept: usize,
    next: usize,
    /// The minute being collected: since when, ms, and its minimum
    minute: Option<(u64, f64)>,
    /// The mode is held at SAFE
    pub held: bool,
}

impl Default for ColdHold {
    fn default() -> Self {
        ColdHold::new()
    }
}

impl ColdHold {
    /// At boot: nothing seen, no hold
    pub const fn new() -> ColdHold {
        ColdHold { minutes: [f64::NAN; ORBIT_MIN], kept: 0, next: 0, minute: None, held: false }
    }

    /// A battery temperature at `now_ms` (`None`: not measured)
    pub fn sample(&mut self, now_ms: u64, tbatt_c: Option<f64>) {
        let Some(t) = tbatt_c.filter(|t| t.is_finite()) else { return };
        match self.minute {
            Some((since, min)) if now_ms.saturating_sub(since) < MINUTE_MS => self.minute = Some((since, min.min(t))),
            Some((_, min)) => {
                if let Some(slot) = self.minutes.get_mut(self.next) {
                    *slot = min;
                }
                self.next = (self.next + 1) % ORBIT_MIN;
                self.kept = (self.kept + 1).min(ORBIT_MIN);
                self.minute = Some((now_ms, t));
            }
            None => self.minute = Some((now_ms, t)),
        }
    }

    /// The minimum over the last orbit (`None`: nothing measured yet)
    pub fn orbit_min(&self) -> Option<f64> {
        let kept = self.minutes.get(..self.kept).unwrap_or(&[]).iter().copied();
        kept.chain(self.minute.map(|(_, m)| m)).reduce(f64::min)
    }

    /// Whether the mode is to be held at SAFE, with the wings read
    /// deployed (`wings_deployed`) or not
    pub fn update(&mut self, wings_deployed: bool) -> bool {
        let min = self.orbit_min();
        self.held = !wings_deployed
            && match min {
                Some(m) if self.held => m < RELEASE_AT_C,
                Some(m) => m < HOLD_BELOW_C,
                None => false,
            };
        self.held
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stowed_and_cold_holds_with_hysteresis() {
        let mut h = ColdHold::new();
        assert!(!h.update(false), "nothing measured: no hold");
        h.sample(0, Some(5.0));
        assert!(!h.update(false));
        h.sample(30_000, Some(2.9));
        assert!(h.update(false));
        assert!(!h.update(true), "the wings deployed release it");
        assert!(h.update(false));
        assert_eq!(ColdHold::default().orbit_min(), None);
    }

    #[test]
    fn the_orbit_minimum_ages_out() {
        let mut h = ColdHold::new();
        h.sample(0, Some(2.0));
        h.sample(1_000, None);
        assert!(h.update(false));
        // An orbit later at +3.5 °C: still held (below the release)
        let mut t = MINUTE_MS;
        for _ in 0..ORBIT_MIN + 1 {
            h.sample(t, Some(3.5));
            t += MINUTE_MS;
        }
        assert_eq!(h.orbit_min(), Some(3.5));
        assert!(h.update(false));
        for _ in 0..ORBIT_MIN + 1 {
            h.sample(t, Some(4.0));
            t += MINUTE_MS;
        }
        assert_eq!(h.orbit_min(), Some(4.0));
        assert!(!h.update(false));
        h.sample(t, Some(f64::NAN));
        assert_eq!(h.orbit_min(), Some(4.0));
    }
}
