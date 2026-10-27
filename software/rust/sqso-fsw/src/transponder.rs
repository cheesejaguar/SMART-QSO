//! The FM transponder's schedule (BASELINE §2 primary mission, §5 power
//! closure, decision D4): while `XPDR_EN` is high the transponder repeats
//! CTCSS-gated uplink carriers on the downlink, and the PA is keyed for as
//! long as a carrier is present. This is the rule-based schedule, the
//! fallback within which the onboard AI may propose (BASELINE §6); it
//! enables the transponder only when every gate of the power closure holds:
//!
//! - transmission is permitted (TX-OFF, RF silence, loss-of-contact
//!   inhibit: `rf_control_tx_permitted()`) and the radio is powered;
//! - the ground allows it: no commissioning hold and no XPDR_DISABLE
//!   (`ops_state.rs`), and no GNSS receiver session is running (the L1
//!   receiver needs the PA off: BASELINE §5);
//! - the mode manager is in ACTIVE, the array is sunlit and the state of
//!   charge is at least [`SOC_PAYLOAD`] (55 %);
//! - the PA is below [`PA_HOT_C`] (held until it cools to [`PA_COOL_C`];
//!   the downlink's limits, TRANSMITTER_DESIGN §6.3);
//! - the keyed time, measured from the forward-power monitor, is under
//!   [`DUTY_LIMIT`] of the last [`WINDOW_MS`] (an orbit); the transponder
//!   resumes when it falls to [`DUTY_RESUME`].
//!
//! `XPDR_EN` is a heartbeat, not a level (docs/hardware/BUILD_VS_BUY.md
//! §3.2, hardware/rf/TRANSMITTER_DESIGN.md §10.1): a hardware timer
//! drives it as a [`HEARTBEAT_HZ`] square wave, and a retriggerable
//! monostable on the radio board holds the transponder's enable only while
//! edges keep arriving ([`EN_HOLD_MS`]). Each pass of the main loop renews
//! the wave for [`Transponder::heartbeat_lease_ms`] (`hal_gpio_heartbeat()`):
//! while the gates hold that is [`HEARTBEAT_LEASE_MS`], otherwise 0, which
//! stops it now. A main loop that stalls stops renewing it, so the timer
//! stops at the end of the lease; an OBC that halts, resets (the pin goes
//! high-impedance and is pulled down) or browns out stops the edges at
//! once. Either way the monostable drops the enable within
//! [`EN_HOLD_MS`] of the last edge: no OBC failure can leave the
//! transponder enabled, and a pin stuck high or low is "off".
//!
//! @requirement SRS-F016 Enforce power budget limits per mode
//! @requirement SRS-F080 The software shall enable the FM transponder only while transmission is permitted, the radio is powered, the ground allows it, the mode is ACTIVE, the array is sunlit, the SOC is at least 55 %, the PA is cool and the keyed time is under 25 %
//! @requirement SRS-F081 The software shall drive the transponder enable XPDR_EN as a heartbeat renewed by every main-loop pass while the transponder gates hold

use crate::mode::SOC_PAYLOAD;

/// Largest keyed share of the window (BASELINE §5: duty ≤ 25 %)
pub const DUTY_LIMIT: f64 = 0.25;
/// Keyed share at which a duty hold ends
pub const DUTY_RESUME: f64 = 0.20;
/// The duty window, ms (100 min, about an orbit)
pub const WINDOW_MS: u64 = 6_000_000;
/// PA temperature that stops the transponder, °C
pub const PA_HOT_C: f64 = 70.0;
/// PA temperature at which it may resume, °C
pub const PA_COOL_C: f64 = 60.0;
/// Forward power above which the PA counts as keyed, W
pub const KEYED_FWD_W: f64 = 0.1;
/// The `XPDR_EN` heartbeat's frequency, Hz (a hardware timer toggles the
/// pin every half period: BUILD_VS_BUY §3.2, ≈ 100 Hz)
pub const HEARTBEAT_HZ: u32 = 100;
/// How long one renewal keeps the heartbeat running, ms: two passes of the
/// 20 ms main loop, which is one period of the loop at its worst case (the
/// 20 ms wait plus a pass at the 20 ms budget of SRS-P001). A later pass
/// still renews it before the monostable drops (the gap it leaves is
/// bridged by [`EN_HOLD_MS`] for passes up to about 65 ms), while a stalled
/// loop drops the transponder within [`STALL_DROPOUT_MS`] (90 ms, inside
/// the 100 ms of VI-RF-11; CDR RF board review)
pub const HEARTBEAT_LEASE_MS: u32 = 40;
/// The radio board's retriggerable monostable (SN74LVC1G123), ms: the
/// enable falls this long after the last heartbeat edge. Hardware, not
/// software; here so the tests and the mission simulation use one value
pub const EN_HOLD_MS: u32 = 50;
/// The longest the transponder stays enabled after the main loop's last
/// pass, ms: the lease, then the monostable
pub const STALL_DROPOUT_MS: u32 = HEARTBEAT_LEASE_MS + EN_HOLD_MS;

/// The main loop's period, ms
#[allow(clippy::cast_possible_truncation)] // 20
const LOOP_MS: u32 = crate::executive::TICK_MS as u32;
// The lease covers one worst-case loop period (the wait plus a pass at its
// budget), not three
const _: () = assert!(HEARTBEAT_LEASE_MS >= 2 * LOOP_MS && HEARTBEAT_LEASE_MS < 3 * LOOP_MS);
// The monostable spans several heartbeat periods (no drop-out while the
// wave runs), and a stalled loop stops the transponder within 90 ms (VI-RF-11: 100 ms)
const _: () = assert!(EN_HOLD_MS >= 4 * (1000 / HEARTBEAT_HZ));
const _: () = assert!(STALL_DROPOUT_MS <= 90);

/// The window in buckets (5 minutes each)
const BUCKETS: usize = 20;
const BUCKET_MS: u64 = WINDOW_MS / BUCKETS as u64;

/// What the schedule looks at, each pass
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inputs {
    /// Transmission permitted and the radio powered
    pub permitted: bool,
    /// The ground allows the transponder (no commissioning hold, no
    /// XPDR_DISABLE) and no GNSS session is running
    pub allowed: bool,
    /// The mode manager is in ACTIVE
    pub active: bool,
    /// The array is sunlit
    pub sunlit: bool,
    /// State of charge, 0–1
    pub soc: f64,
    /// PA temperature, °C (not a number: the monitor did not answer)
    pub pa_c: f64,
    /// Forward power, W (not a number: the monitor did not answer)
    pub fwd_w: f64,
}

/// Why the transponder is off
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// Transmission not permitted, or the radio unpowered
    NotPermitted,
    /// Held for commissioning, disabled by command, or a GNSS session
    NotAllowed,
    /// Not in ACTIVE
    NotActive,
    /// In eclipse
    Eclipse,
    /// State of charge below 55 %
    LowSoc,
    /// The PA is hot, or its monitor does not answer
    PaTemperature,
    /// The keyed-time budget of the window is spent
    Duty,
}

impl Reason {
    /// For the console
    pub const fn text(self) -> &'static str {
        match self {
            Reason::NotPermitted => "transmission not permitted",
            Reason::NotAllowed => "held, disabled by command or GNSS session",
            Reason::NotActive => "not ACTIVE",
            Reason::Eclipse => "eclipse",
            Reason::LowSoc => "low SOC",
            Reason::PaTemperature => "PA temperature",
            Reason::Duty => "duty limit",
        }
    }
}

/// What a pass changed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// Nothing
    None,
    /// Switched on
    On,
    /// Switched off, and why
    Off(Reason),
}

/// The schedule's state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Transponder {
    /// `XPDR_EN` is high
    pub enabled: bool,
    /// Held for PA temperature
    hot: bool,
    /// Held for duty
    duty_hold: bool,
    /// Keyed time in each bucket of the window, ms
    buckets: [u32; BUCKETS],
    /// The bucket now being filled
    bucket: usize,
    /// When it started, ms
    bucket_start_ms: u64,
    /// The previous pass, ms
    last_ms: Option<u64>,
}

impl Transponder {
    /// Off, nothing keyed yet
    pub const fn new() -> Transponder {
        Transponder {
            enabled: false,
            hot: false,
            duty_hold: false,
            buckets: [0; BUCKETS],
            bucket: 0,
            bucket_start_ms: 0,
            last_ms: None,
        }
    }

    /// How long to keep the `XPDR_EN` heartbeat running from this pass, ms
    /// (`hal_gpio_heartbeat()`): [`HEARTBEAT_LEASE_MS`] while the gates
    /// hold, 0 (stop now) otherwise. Renewed on every pass, not only on a
    /// change: the heartbeat lasts only as long as the main loop runs
    pub const fn heartbeat_lease_ms(&self) -> u32 {
        if self.enabled { HEARTBEAT_LEASE_MS } else { 0 }
    }

    /// Keyed share of the window
    pub fn duty(&self) -> f64 {
        let keyed: u64 = self.buckets.iter().map(|&b| u64::from(b)).sum();
        #[allow(clippy::cast_precision_loss)] // at most WINDOW_MS
        let share = keyed as f64 / WINDOW_MS as f64;
        share
    }

    /// Count the time since the previous pass as keyed or not, moving the
    /// window on a bucket at a time
    fn record(&mut self, now_ms: u64, keyed: bool) {
        let Some(last) = self.last_ms.replace(now_ms) else {
            self.bucket_start_ms = now_ms;
            return;
        };
        while now_ms.saturating_sub(self.bucket_start_ms) >= BUCKET_MS {
            self.bucket_start_ms += BUCKET_MS;
            self.bucket = (self.bucket + 1) % BUCKETS;
            // COVERAGE-EXEMPT: bucket < BUCKETS (kept so by the modulo), so the slot exists
            if let Some(b) = self.buckets.get_mut(self.bucket) {
                *b = 0;
                // COVERAGE-EXEMPT: (the brace below) the missing-slot path, as above
            }
        }
        // COVERAGE-EXEMPT: (the missing slot) bucket < BUCKETS, as above; both outcomes of keyed are taken
        if keyed && let Some(b) = self.buckets.get_mut(self.bucket) {
            let elapsed = u32::try_from(now_ms.saturating_sub(last)).unwrap_or(u32::MAX);
            *b = b.saturating_add(elapsed);
        }
    }

    /// The first gate that fails, if any
    fn gate(&mut self, inp: &Inputs) -> Option<Reason> {
        if inp.pa_c.is_nan() || inp.pa_c >= PA_HOT_C {
            self.hot = true;
        } else if inp.pa_c <= PA_COOL_C {
            self.hot = false;
        } else {
            // Between the limits: the hold stays as it is
        }
        let duty = self.duty();
        if duty >= DUTY_LIMIT {
            self.duty_hold = true;
        } else if duty <= DUTY_RESUME {
            self.duty_hold = false;
        } else {
            // Between the limits: the hold stays as it is
        }
        if !inp.permitted {
            Some(Reason::NotPermitted)
        } else if !inp.allowed {
            Some(Reason::NotAllowed)
        } else if !inp.active {
            Some(Reason::NotActive)
        } else if !inp.sunlit {
            Some(Reason::Eclipse)
        } else if inp.soc < SOC_PAYLOAD {
            Some(Reason::LowSoc)
        } else if self.hot {
            Some(Reason::PaTemperature)
        } else if self.duty_hold {
            Some(Reason::Duty)
        } else {
            None
        }
    }

    /// One pass at `now_ms`: the change to `XPDR_EN`
    pub fn step(&mut self, now_ms: u64, inp: &Inputs) -> Change {
        self.record(now_ms, inp.fwd_w > KEYED_FWD_W);
        match (self.gate(inp), self.enabled) {
            (None, false) => {
                self.enabled = true;
                Change::On
            }
            (Some(reason), true) => {
                self.enabled = false;
                Change::Off(reason)
            }
            _ => Change::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GO: Inputs =
        Inputs { permitted: true, allowed: true, active: true, sunlit: true, soc: 0.8, pa_c: 30.0, fwd_w: 0.0 };

    #[test]
    fn every_reason_has_a_text() {
        for r in [
            Reason::NotPermitted,
            Reason::NotAllowed,
            Reason::NotActive,
            Reason::Eclipse,
            Reason::LowSoc,
            Reason::PaTemperature,
            Reason::Duty,
        ] {
            assert!(!r.text().is_empty(), "{r:?}");
        }
    }

    #[test]
    fn on_when_every_gate_holds_off_with_the_first_that_fails() {
        let mut x = Transponder::new();
        assert_eq!(x.step(0, &GO), Change::On);
        assert_eq!(x.step(1000, &GO), Change::None);
        for (inp, reason) in [
            (Inputs { permitted: false, ..GO }, Reason::NotPermitted),
            (Inputs { allowed: false, ..GO }, Reason::NotAllowed),
            (Inputs { active: false, ..GO }, Reason::NotActive),
            (Inputs { sunlit: false, ..GO }, Reason::Eclipse),
            (Inputs { soc: SOC_PAYLOAD - 0.01, ..GO }, Reason::LowSoc),
            (Inputs { pa_c: PA_HOT_C, ..GO }, Reason::PaTemperature),
            (Inputs { pa_c: f64::NAN, ..GO }, Reason::PaTemperature),
        ] {
            let mut x = Transponder::new();
            assert_eq!(x.step(0, &GO), Change::On);
            assert_eq!(x.step(1000, &inp), Change::Off(reason), "{reason:?}");
            assert!(!x.enabled);
            assert!(!reason.text().is_empty());
        }
        assert_eq!(Inputs { soc: SOC_PAYLOAD, ..GO }.soc, SOC_PAYLOAD);
        let mut x = Transponder::new();
        assert_eq!(x.step(0, &Inputs { soc: SOC_PAYLOAD, ..GO }), Change::On, "55 % is enough");
    }

    #[test]
    fn the_heartbeat_runs_only_while_the_gates_hold() {
        let mut x = Transponder::new();
        assert_eq!(x.heartbeat_lease_ms(), 0, "off at boot: no heartbeat");
        let _ = x.step(0, &GO);
        assert_eq!(x.heartbeat_lease_ms(), HEARTBEAT_LEASE_MS);
        let _ = x.step(20, &GO);
        assert_eq!(x.heartbeat_lease_ms(), HEARTBEAT_LEASE_MS, "renewed every pass");
        let _ = x.step(40, &Inputs { sunlit: false, ..GO });
        assert_eq!(x.heartbeat_lease_ms(), 0, "a failed gate stops it now");
    }

    #[test]
    fn the_pa_cools_to_60_c_before_it_resumes() {
        let mut x = Transponder::new();
        assert_eq!(x.step(0, &Inputs { pa_c: 71.0, ..GO }), Change::None);
        assert_eq!(x.step(1000, &Inputs { pa_c: 65.0, ..GO }), Change::None, "still held");
        assert_eq!(x.step(2000, &Inputs { pa_c: PA_COOL_C, ..GO }), Change::On);
        assert_eq!(x.step(3000, &Inputs { pa_c: 65.0, ..GO }), Change::None, "not hot yet");
    }

    #[test]
    fn keyed_time_is_held_to_a_quarter_of_the_window() {
        let mut x = Transponder::new();
        let keyed = Inputs { fwd_w: 0.5, ..GO };
        let mut t = 0;
        assert_eq!(x.step(t, &keyed), Change::On);
        let mut off_at = None;
        while t < WINDOW_MS {
            t += 1000;
            if let Change::Off(Reason::Duty) = x.step(t, &keyed) {
                off_at = Some(t);
                break;
            }
        }
        let off_at = off_at.expect("the duty limit stops it");
        assert!((off_at as f64 - DUTY_LIMIT * WINDOW_MS as f64).abs() <= 1000.0, "{off_at}");
        // Unkeyed, the window moves on: it resumes at 20 %, after the oldest
        // keyed buckets leave the window
        let mut on_at = None;
        while t < 3 * WINDOW_MS {
            t += 1000;
            if x.step(t, &GO) == Change::On {
                on_at = Some(t);
                break;
            }
        }
        let on_at = on_at.expect("it resumes");
        assert!(x.duty() <= DUTY_RESUME + 1e-9);
        assert!(on_at > WINDOW_MS, "{on_at}");
    }

    #[test]
    fn a_long_gap_clears_the_window() {
        let mut x = Transponder::new();
        let keyed = Inputs { fwd_w: 0.5, ..GO };
        let _ = x.step(0, &keyed);
        let _ = x.step(600_000, &keyed);
        assert!(x.duty() > 0.09);
        let _ = x.step(600_000 + 2 * WINDOW_MS, &GO);
        assert_eq!(x.duty(), 0.0);
        let unread = Inputs { fwd_w: f64::NAN, ..GO };
        let _ = x.step(700_000 + 2 * WINDOW_MS, &unread);
        assert_eq!(x.duty(), 0.0, "an unread monitor counts as unkeyed");
    }
}
