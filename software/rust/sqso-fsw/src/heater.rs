//! Battery heater thermostat (`hardware/structure/THERMAL_DESIGN.md` §6.2–6.3,
//! `hardware/eps/BATTERY_DESIGN.md` §6.2): the heater on the pack (about
//! 207 Ω, 0.72–1.30 W across the bus) holds the cells above 0 °C, so the
//! charge regulator, which refuses to charge below 0 °C (BASELINE §5,
//! SYS-602), can charge again as soon as the array sees the Sun.
//!
//! Set points (BASELINE §8, `docs/environment/THERMAL_ANALYSIS.md` v2.0;
//! RID THM-02): on below +5 °C, off above +9 °C. The +2/+6 °C of the first
//! design left the coldest cell at +0.07 °C in the thermal model's cold
//! case; +5/+9 °C gives +4.4 °C, margin for the sensor's error and the
//! gradient across the pack. Without a fresh battery temperature the heater
//! is off (it never runs unwatched).
//!
//! @requirement SYS-602 No charging below 0 °C; keep the cells chargeable
//! @requirement SRS-F130 Battery heater on below +5 °C, off above +9 °C

/// Heater on below this battery temperature, °C
pub const ON_BELOW_C: f64 = 5.0;
/// Heater off above this battery temperature, °C
pub const OFF_ABOVE_C: f64 = 9.0;

/// The thermostat's state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Heater {
    /// The heater is on
    pub on: bool,
}

/// What a step decided
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// Nothing changed
    None,
    /// Switched on (cold)
    On,
    /// Switched off (warm again)
    Off,
    /// Switched off: no fresh temperature
    NoSensor,
}

impl Heater {
    /// Off
    pub const fn new() -> Heater {
        Heater { on: false }
    }

    /// The battery temperature now (`None`: no fresh reading); the change
    pub fn step(&mut self, battery_c: Option<f64>) -> Change {
        match battery_c {
            None if self.on => {
                self.on = false;
                Change::NoSensor
            }
            None => Change::None,
            Some(t) if !self.on && t < ON_BELOW_C => {
                self.on = true;
                Change::On
            }
            Some(t) if self.on && t > OFF_ABOVE_C => {
                self.on = false;
                Change::Off
            }
            Some(_) => Change::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hysteresis() {
        let mut h = Heater::new();
        assert_eq!((ON_BELOW_C, OFF_ABOVE_C), (5.0, 9.0), "BASELINE §8");
        assert_eq!(h.step(Some(10.0)), Change::None);
        assert_eq!(h.step(Some(ON_BELOW_C)), Change::None);
        assert_eq!(h.step(Some(4.9)), Change::On);
        assert!(h.on);
        assert_eq!(h.step(Some(8.0)), Change::None);
        assert_eq!(h.step(Some(OFF_ABOVE_C)), Change::None);
        assert_eq!(h.step(Some(9.1)), Change::Off);
        assert!(!h.on);
    }

    #[test]
    fn no_reading_turns_it_off() {
        let mut h = Heater::new();
        assert_eq!(h.step(None), Change::None);
        assert_eq!(h.step(Some(-5.0)), Change::On);
        assert_eq!(h.step(None), Change::NoSensor);
        assert!(!h.on);
    }
}
