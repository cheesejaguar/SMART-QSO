//! The Jetson's switch-off and its power-good (SQSO-JET-BRD-001 §7.4,
//! SMART-QSO-ICD-002 1.3 §10.4, RID JETB-03), and the readbacks of its two
//! switches (SQSO-OBC-BRD-001 B §11.2, RID AV-06).
//!
//! **Normal end of a burst.** The OBC sends `SHUTDOWN`; the payload runs
//! `poweroff` and the module asserts SHUTDOWN_REQ*, which the carrier
//! latches; the carrier's 3.3 V (JET_PG) falls; the OBC then drives
//! JETSON_PWR low and PAYLOAD_EN [`PAYLOAD_EN_DELAY_MS`] later
//! (`eps_control`). If JET_PG has not fallen [`SHUTDOWN_WAIT_MS`] after
//! `SHUTDOWN`, both lines are cut anyway.
//!
//! **Power-good.** With the rail commanded on, JET_PG rises about 0.1 s
//! after the switches; a fall while the rail is on and no `SHUTDOWN` was
//! sent is a payload fault (a carrier trip or a module thermal shutdown):
//! the rail is cut and the fault logged. JET_PG not up [`PG_RISE_MS`] after
//! the switch-on is a fault too.
//!
//! **Readbacks.** VSW1_RB (SW1's output) and VJET_RB (the eFuse's output)
//! are compared with the commands [`READBACK_SETTLE_MS`] after each
//! change: an output above [`READBACK_ON_V`] means on (1.0 V at the pin,
//! through the ×6 divider). A disagreement is a fault: logged at onset and
//! clearance (`condition`).
//!
//! @requirement SRS-F148 Jetson switch-off: SHUTDOWN, JET_PG fall within 5 s, then JETSON_PWR and PAYLOAD_EN 10 ms apart
//! @requirement SRS-F149 Jetson power-good and switch readbacks: an unexpected JET_PG fall cuts the rail; a readback that disagrees with its command is a fault

/// Wait at most this long for JET_PG to fall after `SHUTDOWN`, ms
pub const SHUTDOWN_WAIT_MS: u64 = 5_000;
/// PAYLOAD_EN goes low at least this long after JETSON_PWR, ms
pub const PAYLOAD_EN_DELAY_MS: u64 = 10;
/// JET_PG must be up this long after the switch-on, ms
pub const PG_RISE_MS: u64 = 2_000;
/// Readbacks are compared this long after a command change, ms
pub const READBACK_SETTLE_MS: u64 = 200;
/// A switch output above this is on, V (1.0 V at the ADC pin, × 6)
pub const READBACK_ON_V: f64 = 6.0;

/// What the caller does
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing
    None,
    /// Send `SHUTDOWN` to the payload (the switch-off has started)
    SendShutdown,
    /// Switch the rail off now: JET_PG fell (or the wait ran out)
    PowerOff,
    /// Switch the rail off and log: JET_PG fell unexpectedly
    FaultPgFell,
    /// Switch the rail off and log: JET_PG never came up
    FaultNoPg,
}

/// Where the switch-off is
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    /// Rail off
    Off,
    /// Rail on since this time, ms; JET_PG seen
    On(u64, bool),
    /// `SHUTDOWN` sent at this time, ms
    ShuttingDown(u64),
}

/// The supervisor's state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JetsonPower {
    phase: Phase,
    /// Commanded switch-offs completed by a JET_PG fall
    pub clean: u32,
    /// Switch-offs cut after the wait
    pub timed_out: u32,
    /// Unexpected JET_PG falls and power-good failures
    pub faults: u32,
    /// The commands last changed at this time, ms
    changed_ms: u64,
    last_cmd: (bool, bool),
}

impl Default for JetsonPower {
    fn default() -> Self {
        JetsonPower::new()
    }
}

/// The readbacks' verdict
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Readback {
    /// VSW1_RB disagrees with PAYLOAD_EN
    pub sw1_bad: bool,
    /// VJET_RB disagrees with both commands
    pub efuse_bad: bool,
}

impl JetsonPower {
    /// Rail off
    pub const fn new() -> JetsonPower {
        JetsonPower { phase: Phase::Off, clean: 0, timed_out: 0, faults: 0, changed_ms: 0, last_cmd: (false, false) }
    }

    /// The normal end of a burst: start the commanded switch-off (the rail
    /// stays on until JET_PG falls); `SendShutdown` if one starts
    pub fn request_off(&mut self, now_ms: u64) -> Action {
        match self.phase {
            Phase::On(..) => {
                self.phase = Phase::ShuttingDown(now_ms);
                Action::SendShutdown
            }
            Phase::Off | Phase::ShuttingDown(_) => Action::None,
        }
    }

    /// Whether a commanded switch-off is under way
    pub const fn shutting_down(&self) -> bool {
        matches!(self.phase, Phase::ShuttingDown(_))
    }

    /// One pass: `rail_on` is the payload's command (both switches),
    /// `pg` the JET_PG line
    pub fn step(&mut self, now_ms: u64, rail_on: bool, pg: bool) -> Action {
        if !rail_on {
            self.phase = Phase::Off;
            return Action::None;
        }
        match self.phase {
            Phase::Off => {
                self.phase = Phase::On(now_ms, pg);
                Action::None
            }
            Phase::On(since, seen) => {
                if pg {
                    self.phase = Phase::On(since, true);
                    Action::None
                } else if seen {
                    self.faults = self.faults.saturating_add(1);
                    Action::FaultPgFell
                } else if now_ms.saturating_sub(since) >= PG_RISE_MS {
                    self.faults = self.faults.saturating_add(1);
                    Action::FaultNoPg
                } else {
                    Action::None
                }
            }
            Phase::ShuttingDown(since) => {
                if !pg {
                    self.clean = self.clean.saturating_add(1);
                    Action::PowerOff
                } else if now_ms.saturating_sub(since) >= SHUTDOWN_WAIT_MS {
                    self.timed_out = self.timed_out.saturating_add(1);
                    Action::PowerOff
                } else {
                    Action::None
                }
            }
        }
    }

    /// Compare the readbacks (`sw1_v`, `jet_v`: the switch outputs, V; NaN:
    /// not read) with the commands (`payload_en`, `jetson_pwr`), once they
    /// have been steady for [`READBACK_SETTLE_MS`]
    pub fn readback(&mut self, now_ms: u64, payload_en: bool, jetson_pwr: bool, sw1_v: f64, jet_v: f64) -> Readback {
        if (payload_en, jetson_pwr) != self.last_cmd {
            self.last_cmd = (payload_en, jetson_pwr);
            self.changed_ms = now_ms;
        }
        if now_ms.saturating_sub(self.changed_ms) < READBACK_SETTLE_MS {
            return Readback::default();
        }
        let bad = |v: f64, on: bool| v.is_finite() && (v > READBACK_ON_V) != on;
        Readback { sw1_bad: bad(sw1_v, payload_en), efuse_bad: bad(jet_v, payload_en && jetson_pwr) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commanded_switch_off_waits_for_power_good_to_fall() {
        let mut j = JetsonPower::new();
        assert_eq!(j.request_off(0), Action::None, "rail off: nothing to shut down");
        assert_eq!(j.step(0, true, false), Action::None);
        assert_eq!(j.step(100, true, true), Action::None);
        assert_eq!(j.request_off(200_000), Action::SendShutdown);
        assert!(j.shutting_down());
        assert_eq!(j.request_off(200_001), Action::None, "only once");
        assert_eq!(j.step(203_000, true, true), Action::None, "poweroff running");
        assert_eq!(j.step(204_000, true, false), Action::PowerOff);
        assert_eq!(j.clean, 1);
        assert_eq!(j.step(204_020, false, false), Action::None);
        assert!(!j.shutting_down());
    }

    #[test]
    fn the_wait_runs_out_and_unexpected_falls_are_faults() {
        let mut j = JetsonPower::new();
        let _ = j.step(0, true, true);
        let _ = j.step(20, true, true);
        assert_eq!(j.request_off(1000), Action::SendShutdown);
        assert_eq!(j.step(1000 + SHUTDOWN_WAIT_MS - 1, true, true), Action::None);
        assert_eq!(j.step(1000 + SHUTDOWN_WAIT_MS, true, true), Action::PowerOff);
        assert_eq!(j.timed_out, 1);
        let _ = j.step(7000, false, false);
        let _ = j.step(8000, true, false);
        let _ = j.step(8100, true, true);
        assert_eq!(j.step(9000, true, false), Action::FaultPgFell);
        let mut j = JetsonPower::new();
        let _ = j.step(0, true, false);
        assert_eq!(j.step(PG_RISE_MS - 1, true, false), Action::None);
        assert_eq!(j.step(PG_RISE_MS, true, false), Action::FaultNoPg);
        assert_eq!(j.faults, 1);
    }

    #[test]
    fn readbacks_agree_or_not() {
        let mut j = JetsonPower::new();
        assert_eq!(j.readback(0, true, true, 0.0, 0.0), Readback::default(), "settling");
        assert_eq!(j.readback(READBACK_SETTLE_MS, true, true, 14.0, 14.0), Readback::default());
        assert_eq!(j.readback(READBACK_SETTLE_MS, true, true, 0.1, 14.0), Readback { sw1_bad: true, efuse_bad: false });
        assert_eq!(j.readback(1000, false, false, 0.0, 0.0), Readback::default(), "settling again");
        assert_eq!(j.readback(1300, false, false, 0.0, 13.0), Readback { sw1_bad: false, efuse_bad: true });
        assert_eq!(j.readback(1300, false, false, f64::NAN, f64::NAN), Readback::default(), "not read");
    }
}
