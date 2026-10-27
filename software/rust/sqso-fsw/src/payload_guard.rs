//! OBC-enforced Jetson power and temperature limits (SW-16;
//! `payload_guard.h`).
//!
//! The OBC decides from its own measurements whether the payload rail may be
//! on, whatever the Jetson reports about itself (BASELINE §6). While the
//! rail is on it must be turned off when input power is above
//! [`MAX_INPUT_W`] for [`OVERPOWER_SAMPLES`] samples in a row or any sample
//! reaches [`LATCHUP_INPUT_W`], the heat spreader reaches
//! [`SPREADER_MAX_C`], the Jetson junction reaches [`JUNCTION_MAX_C`], or no
//! power or spreader measurement has arrived for [`TELEMETRY_TIMEOUT_MS`].
//! After a trip the rail stays off for [`COOLDOWN_MS`], and then until a
//! recent spreader reading is [`RESTART_MARGIN_C`] below its limit.
//!
//! A cold module is not powered either (RID THM-08; BASELINE §6): no
//! power-on while a recent spreader reading is below [`MIN_POWER_ON_C`].
//! As with the hot limit before a trip, only a known reading blocks; the
//! spreader thermistor is the OBC's own and its loss is a sensor fault.
//!
//! The spreader thermistor read outside its valid range is a sensor fault
//! ([`Guard::set_spreader_fault`], RID JETB-02): no power-on while it lasts
//! (the gate fails safe); a rail already on is then supervised on the
//! Jetson's own junction report (the executive's fallback).
//!
//! @requirement SRS-F131 The software shall not power the Jetson while its heat spreader reads below −15 °C
//! @requirement SRS-F150 Spreader thermistor: R = 10.0 kΩ N / (4095 − N), Steinhart–Hart 10K3A; outside 0.10–3.25 V a sensor fault that inhibits the Jetson's power-on

use core::ffi::CStr;

/// Carrier input limit, W (BASELINE §6)
pub const MAX_INPUT_W: f64 = 8.0;
/// A single sample this high trips at once (suspected latch-up), W
pub const LATCHUP_INPUT_W: f64 = 12.0;
/// Samples above [`MAX_INPUT_W`] in a row that trip
pub const OVERPOWER_SAMPLES: u32 = 2;
/// Heat-spreader shutdown limit, °C (POWER_MANAGEMENT.md §5.1)
pub const SPREADER_MAX_C: f64 = 70.0;
/// Jetson junction shutdown limit, °C
pub const JUNCTION_MAX_C: f64 = 95.0;
/// The spreader must cool this far below its limit before a restart, °C
pub const RESTART_MARGIN_C: f64 = 10.0;
/// Rail-off time after a trip, ms
pub const COOLDOWN_MS: u64 = 600_000;
/// Longest gap between measurements while the rail is on, ms
pub const TELEMETRY_TIMEOUT_MS: u64 = 5000;
/// No power-on below this heat-spreader temperature, °C (BASELINE §6:
/// "Jetson temperature ≥ −15 °C at power-on")
pub const MIN_POWER_ON_C: f64 = -15.0;

/// One set of measurements (`PayloadSample_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sample {
    /// `input_w` measured (bus voltage x rail current)
    pub power_valid: bool,
    /// Rail input power, W
    pub input_w: f64,
    /// `spreader_c` from the OBC thermistor
    pub spreader_valid: bool,
    /// Heat-spreader temperature, °C
    pub spreader_c: f64,
    /// `junction_c` reported by the Jetson (STATUS)
    pub junction_valid: bool,
    /// Jetson junction temperature, °C
    pub junction_c: f64,
}

/// What a step decided (`PayloadGuardResult_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Outcome {
    /// Nothing to do
    Ok = 0,
    /// Above [`MAX_INPUT_W`] for [`OVERPOWER_SAMPLES`] samples
    Overpower = 1,
    /// A sample at or above [`LATCHUP_INPUT_W`]
    Latchup = 2,
    /// Heat spreader at its limit
    SpreaderTemp = 3,
    /// Jetson junction at its limit
    JunctionTemp = 4,
    /// No power or spreader measurement for [`TELEMETRY_TIMEOUT_MS`]
    NoTelemetry = 5,
    /// Rail on during a cool-down: turn it off
    Cooldown = 6,
}

impl Outcome {
    /// Every outcome, in C order
    pub const ALL: [Outcome; 7] = [
        Outcome::Ok,
        Outcome::Overpower,
        Outcome::Latchup,
        Outcome::SpreaderTemp,
        Outcome::JunctionTemp,
        Outcome::NoTelemetry,
        Outcome::Cooldown,
    ];

    /// The outcome with this C value
    pub fn from_raw(raw: u32) -> Option<Outcome> {
        Outcome::ALL.into_iter().find(|o| *o as u32 == raw)
    }
}

/// Fault-log text for a C result value (`payload_guard_reason()`)
pub fn reason(raw: u32) -> &'static CStr {
    match Outcome::from_raw(raw) {
        Some(Outcome::Ok) => c"OK",
        Some(Outcome::Overpower) => c"Payload over 8 W input: rail off",
        Some(Outcome::Latchup) => c"Payload latch-up current: rail off",
        Some(Outcome::SpreaderTemp) => c"Payload heat spreader over limit: rail off",
        Some(Outcome::JunctionTemp) => c"Jetson junction over limit: rail off",
        Some(Outcome::NoTelemetry) => c"Payload unsupervised (no measurements): rail off",
        Some(Outcome::Cooldown) => c"Payload rail on during cool-down: rail off",
        None => c"Payload guard: unknown result",
    }
}

/// The guard's state
#[derive(Clone, Copy, Debug, Default)]
pub struct Guard {
    was_on: bool,
    /// Last valid power reading (or rail-on time)
    power_ms: u64,
    /// Last valid spreader reading (or rail-on time)
    spreader_ms: u64,
    spreader_seen: bool,
    spreader_c: f64,
    spreader_seen_ms: u64,
    over_count: u32,
    tripped: bool,
    trip_ms: u64,
    trips: u32,
    spreader_fault: bool,
}

fn age_ok(then_ms: u64, now_ms: u64) -> bool {
    now_ms >= then_ms && now_ms - then_ms <= TELEMETRY_TIMEOUT_MS
}

impl Guard {
    /// Nothing seen, no trip (`payload_guard_init()`)
    pub const fn new() -> Self {
        Guard {
            was_on: false,
            power_ms: 0,
            spreader_ms: 0,
            spreader_seen: false,
            spreader_c: 0.0,
            spreader_seen_ms: 0,
            over_count: 0,
            tripped: false,
            trip_ms: 0,
            trips: 0,
            spreader_fault: false,
        }
    }

    /// The spreader thermistor is (or is no longer) faulted
    pub fn set_spreader_fault(&mut self, fault: bool) {
        self.spreader_fault = fault;
    }

    fn in_cooldown(&self, now_ms: u64) -> bool {
        self.tripped && now_ms < self.trip_ms.saturating_add(COOLDOWN_MS)
    }

    /// Record the readings; restart the supervision clocks when the rail
    /// turns on
    fn record(&mut self, sample: Option<&Sample>, payload_on: bool, now_ms: u64) {
        if payload_on && !self.was_on {
            self.power_ms = now_ms;
            self.spreader_ms = now_ms;
            self.over_count = 0;
        }
        self.was_on = payload_on;
        let Some(s) = sample else { return };
        if s.power_valid {
            self.power_ms = now_ms;
        }
        if s.spreader_valid {
            self.spreader_ms = now_ms;
            self.spreader_seen = true;
            self.spreader_c = s.spreader_c;
            self.spreader_seen_ms = now_ms;
        }
    }

    fn check_limits(&mut self, sample: Option<&Sample>, now_ms: u64) -> Outcome {
        if !age_ok(self.power_ms, now_ms) || !age_ok(self.spreader_ms, now_ms) {
            return Outcome::NoTelemetry;
        }
        let Some(s) = sample else { return Outcome::Ok };
        if s.power_valid {
            if s.input_w >= LATCHUP_INPUT_W {
                return Outcome::Latchup;
            }
            self.over_count = if s.input_w > MAX_INPUT_W { self.over_count.saturating_add(1) } else { 0 };
            if self.over_count >= OVERPOWER_SAMPLES {
                return Outcome::Overpower;
            }
        }
        if s.spreader_valid && s.spreader_c >= SPREADER_MAX_C {
            return Outcome::SpreaderTemp;
        }
        if s.junction_valid && s.junction_c >= JUNCTION_MAX_C {
            return Outcome::JunctionTemp;
        }
        Outcome::Ok
    }

    /// Evaluate one set of measurements (`None` counts as none): `Ok`, or
    /// why the rail must be turned off now
    pub fn step(&mut self, sample: Option<&Sample>, payload_on: bool, now_ms: u64) -> Outcome {
        self.record(sample, payload_on, now_ms);
        if !payload_on {
            return Outcome::Ok;
        }
        if self.in_cooldown(now_ms) {
            return Outcome::Cooldown;
        }
        let result = self.check_limits(sample, now_ms);
        if result != Outcome::Ok {
            self.tripped = true;
            self.trip_ms = now_ms;
            self.trips = self.trips.wrapping_add(1);
            self.over_count = 0;
        }
        result
    }

    /// May the rail be turned on now?
    pub fn may_enable(&self, now_ms: u64) -> bool {
        if self.in_cooldown(now_ms) {
            return false;
        }
        let spreader_recent = self.spreader_seen && age_ok(self.spreader_seen_ms, now_ms);
        let spreader_cool = self.spreader_c < SPREADER_MAX_C - RESTART_MARGIN_C;
        if self.tripped {
            spreader_recent && spreader_cool // after a trip, show it has cooled
        } else {
            !spreader_recent || spreader_cool // otherwise refuse only if known hot
        }
    }

    /// May the rail be switched on now: [`Guard::may_enable`], and the
    /// module not known to be colder than [`MIN_POWER_ON_C`]
    pub fn may_power_on(&self, now_ms: u64) -> bool {
        let spreader_recent = self.spreader_seen && age_ok(self.spreader_seen_ms, now_ms);
        self.may_enable(now_ms) && !(spreader_recent && self.spreader_c < MIN_POWER_ON_C) && !self.spreader_fault
    }

    /// Trips since initialization
    pub const fn trip_count(&self) -> u32 {
        self.trips
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T0: u64 = 1_000_000;

    fn sample(watts: f64, spreader_c: f64) -> Sample {
        Sample { power_valid: true, input_w: watts, spreader_valid: true, spreader_c, ..Sample::default() }
    }

    #[test]
    fn overpower_needs_two_in_a_row_latchup_one() {
        let mut g = Guard::new();
        let (hot, ok) = (sample(8.5, 45.0), sample(7.9, 45.0));
        assert_eq!(g.step(Some(&hot), true, T0), Outcome::Ok);
        assert_eq!(g.step(Some(&ok), true, T0 + 1000), Outcome::Ok);
        assert_eq!(g.step(Some(&hot), true, T0 + 2000), Outcome::Ok);
        assert_eq!(g.step(Some(&hot), true, T0 + 3000), Outcome::Overpower);
        assert_eq!(g.trip_count(), 1);
        assert!(!g.may_enable(T0 + 3000));

        let mut g = Guard::new();
        assert_eq!(g.step(Some(&sample(LATCHUP_INPUT_W, 45.0)), true, T0), Outcome::Latchup);
        // Exactly the limit is not over it
        let mut g = Guard::new();
        let at = sample(MAX_INPUT_W, 45.0);
        assert_eq!(g.step(Some(&at), true, T0), Outcome::Ok);
        assert_eq!(g.step(Some(&at), true, T0 + 1), Outcome::Ok);
    }

    #[test]
    fn temperatures_and_unreported_junction() {
        let mut g = Guard::new();
        let mut s = sample(6.0, SPREADER_MAX_C - 0.1);
        assert_eq!(g.step(Some(&s), true, T0), Outcome::Ok);
        s.spreader_c = SPREADER_MAX_C;
        assert_eq!(g.step(Some(&s), true, T0 + 1000), Outcome::SpreaderTemp);

        let mut g = Guard::new();
        let mut s = Sample { junction_valid: true, junction_c: JUNCTION_MAX_C, ..sample(6.0, 50.0) };
        assert_eq!(g.step(Some(&s), true, T0), Outcome::JunctionTemp);
        let mut g = Guard::new();
        s.junction_valid = false;
        assert_eq!(g.step(Some(&s), true, T0), Outcome::Ok);
    }

    #[test]
    fn unsupervised_rail_trips_after_the_grace() {
        let mut g = Guard::new();
        assert_eq!(g.step(None, false, T0), Outcome::Ok);
        assert_eq!(g.step(None, true, T0 + 100_000), Outcome::Ok);
        assert_eq!(g.step(None, true, T0 + 100_000 + TELEMETRY_TIMEOUT_MS), Outcome::Ok);
        assert_eq!(g.step(None, true, T0 + 100_001 + TELEMETRY_TIMEOUT_MS), Outcome::NoTelemetry);

        // Power alone is not enough
        let mut g = Guard::new();
        let s = Sample { spreader_valid: false, ..sample(6.0, 45.0) };
        assert_eq!(g.step(Some(&s), true, T0), Outcome::Ok);
        assert_eq!(g.step(Some(&s), true, T0 + TELEMETRY_TIMEOUT_MS + 1), Outcome::NoTelemetry);
    }

    #[test]
    fn cooldown_then_only_when_recently_cool() {
        let mut g = Guard::new();
        assert_eq!(g.step(Some(&sample(LATCHUP_INPUT_W, 50.0)), true, T0), Outcome::Latchup);
        let warm = sample(0.0, SPREADER_MAX_C - RESTART_MARGIN_C);
        let cool = sample(0.0, SPREADER_MAX_C - RESTART_MARGIN_C - 0.1);
        assert_eq!(g.step(Some(&cool), true, T0 + 1000), Outcome::Cooldown);
        assert_eq!(g.step(Some(&cool), false, T0 + COOLDOWN_MS - 1), Outcome::Ok);
        assert!(!g.may_enable(T0 + COOLDOWN_MS - 1));
        assert_eq!(g.step(Some(&warm), false, T0 + COOLDOWN_MS), Outcome::Ok);
        assert!(!g.may_enable(T0 + COOLDOWN_MS));
        assert_eq!(g.step(Some(&cool), false, T0 + COOLDOWN_MS + 1), Outcome::Ok);
        assert!(!g.may_enable(T0 + COOLDOWN_MS + 2 + TELEMETRY_TIMEOUT_MS));
        assert!(g.may_enable(T0 + COOLDOWN_MS + 1));
        assert_eq!(g.trip_count(), 1, "a cool-down result is not another trip");
    }

    /// THM-08: a known cold spreader blocks a power-on, nothing else does
    #[test]
    fn a_cold_module_is_not_powered() {
        let mut g = Guard::new();
        assert!(g.may_power_on(T0), "no reading: allowed, as for the hot limit");
        assert_eq!(g.step(Some(&sample(0.0, MIN_POWER_ON_C - 0.1)), false, T0), Outcome::Ok);
        assert!(g.may_enable(T0), "the ACTIVE guard is not the power-on");
        assert!(!g.may_power_on(T0));
        assert_eq!(g.step(Some(&sample(0.0, MIN_POWER_ON_C)), false, T0 + 1), Outcome::Ok);
        assert!(g.may_power_on(T0 + 1), "at the limit it may");
        let mut g = Guard::new();
        assert_eq!(g.step(Some(&sample(0.0, -30.0)), false, T0), Outcome::Ok);
        assert!(g.may_power_on(T0 + TELEMETRY_TIMEOUT_MS + 1), "a stale reading no longer blocks");
        // A guard that refuses anyway refuses
        let hot = sample(0.0, SPREADER_MAX_C);
        let mut g = Guard::new();
        assert_eq!(g.step(Some(&hot), false, T0), Outcome::Ok);
        assert!(!g.may_power_on(T0));
    }

    #[test]
    fn before_a_trip_only_a_known_hot_spreader_blocks() {
        let mut g = Guard::new();
        assert!(g.may_enable(T0));
        assert_eq!(g.step(Some(&sample(0.0, SPREADER_MAX_C - RESTART_MARGIN_C)), false, T0), Outcome::Ok);
        assert!(!g.may_enable(T0));
        // A stale hot reading no longer blocks
        assert!(g.may_enable(T0 + TELEMETRY_TIMEOUT_MS + 1));
    }

    /// A clock that steps back behind the last reading leaves the rail
    /// unsupervised: it trips, and that reading is not recent enough to
    /// show the spreader has cooled
    #[test]
    fn clock_stepping_back_counts_as_no_telemetry() {
        let mut g = Guard::new();
        let cool = sample(5.0, 30.0);
        assert_eq!(g.step(Some(&cool), true, T0), Outcome::Ok);
        assert_eq!(g.step(None, true, T0 - 1), Outcome::NoTelemetry);
        assert_eq!(g.trip_count(), 1);

        // After a trip and its cool-down, a cool reading stamped ahead of
        // the clock does not yet allow a restart
        let mut g = Guard::new();
        assert_eq!(g.step(Some(&sample(LATCHUP_INPUT_W, 30.0)), true, T0), Outcome::Latchup);
        let later = T0 + COOLDOWN_MS + 10;
        assert_eq!(g.step(Some(&cool), false, later), Outcome::Ok);
        assert!(!g.may_enable(later - 5));
        assert!(g.may_enable(later));
    }

    #[test]
    fn reasons() {
        assert_eq!(reason(0), c"OK");
        assert_eq!(reason(Outcome::Latchup as u32), c"Payload latch-up current: rail off");
        assert_eq!(reason(99), c"Payload guard: unknown result");
        for o in Outcome::ALL {
            assert_eq!(Outcome::from_raw(o as u32), Some(o));
            assert_ne!(reason(o as u32), reason(99));
        }
    }

    /// Mutation check: the cool-down ends exactly COOLDOWN_MS after the
    /// trip, and every trip is counted
    #[test]
    fn cooldown_ends_exactly_and_trips_are_counted() {
        let mut g = Guard::new();
        let cool = sample(0.0, SPREADER_MAX_C - RESTART_MARGIN_C - 1.0);
        assert_eq!(g.step(Some(&sample(LATCHUP_INPUT_W, 50.0)), true, T0), Outcome::Latchup);
        assert_eq!(g.step(Some(&cool), true, T0 + COOLDOWN_MS - 1), Outcome::Cooldown);
        assert!(!g.may_enable(T0 + COOLDOWN_MS - 1));
        assert!(g.may_enable(T0 + COOLDOWN_MS));
        assert_eq!(g.step(Some(&cool), true, T0 + COOLDOWN_MS), Outcome::Ok);
        assert_eq!(g.trip_count(), 1);
        assert_eq!(g.step(Some(&sample(LATCHUP_INPUT_W, 50.0)), true, T0 + COOLDOWN_MS + 1), Outcome::Latchup);
        assert_eq!(g.trip_count(), 2);
        assert_eq!(Guard::new().trip_count(), 0);
    }

    #[test]
    fn a_faulted_spreader_thermistor_inhibits_the_power_on() {
        let mut g = Guard::new();
        assert!(g.may_power_on(T0));
        g.set_spreader_fault(true);
        assert!(!g.may_power_on(T0), "fails safe");
        assert!(g.may_enable(T0), "the ACTIVE guard is not the power-on gate");
        g.set_spreader_fault(false);
        assert!(g.may_power_on(T0));
    }
}
