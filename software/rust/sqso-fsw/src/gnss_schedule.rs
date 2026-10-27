//! GNSS receiver sessions (decision D6; BASELINE §5 and §6): the receiver is
//! duty-cycled, powered through its load switch (`GPIO_PIN_GNSS_EN`) for a
//! session, then off.
//!
//! - **When.** The first session [`FIRST_SESSION_DELAY_MS`] after boot (the
//!   receiver only listens, so it may run inside the RF silence: the launch
//!   provider wants ephemeris early, SpaceX User's Guide v11 §9.5.7.2). Then
//!   every orbit ([`LEOP_PERIOD_MS`]) while the commissioning hold is set
//!   (launch and early operations), and four a day ([`OPS_PERIOD_MS`]) after
//!   release; GNSS_REQUEST starts one at once.
//! - **Gates.** A session starts and continues only while the caller's gates
//!   hold: Jetson off and the state of charge above the contingency floor
//!   (BASELINE §5 GNSS row). The transponder is held off during a session
//!   (the L1 receiver beside the 2 m PA, `transponder.rs`). A session the
//!   gates stop is retried after [`RETRY_MS`].
//! - **Length.** Until [`AFTER_FIX_MS`] after the first accepted fix of the
//!   session, or [`SESSION_MAX_MS`] without one (cold start in LEO about
//!   80 s). [`FAULT_AFTER_NO_FIX`] sessions in a row without a fix are a
//!   fault the caller logs (the receiver is power-cycled by the next session
//!   anyway; a latch-up is tripped by the load switch in hardware).
//!
//! Average power: a 5-minute session at 125 mW every 95 min is 6.6 mW, every
//! 6 h 1.7 mW, inside the 50 mW allocation of BASELINE §5.
//!
//! @requirement SRS-F084 The software shall power the GNSS receiver for sessions, every orbit in launch and early operations then four a day, keep the latest fix with its time and quality, and start a session on command

/// First session after boot, ms
pub const FIRST_SESSION_DELAY_MS: u64 = 5 * 60_000;
/// Between sessions while the commissioning hold is set (an orbit), ms
pub const LEOP_PERIOD_MS: u64 = 95 * 60_000;
/// Between sessions in operations (four a day), ms
pub const OPS_PERIOD_MS: u64 = 6 * 3_600_000;
/// Longest session without a fix, ms
pub const SESSION_MAX_MS: u64 = 300_000;
/// A session ends this long after its first fix, ms (a few more records)
pub const AFTER_FIX_MS: u64 = 30_000;
/// A session the gates refused or stopped is retried after this, ms
pub const RETRY_MS: u64 = 10 * 60_000;
/// Sessions in a row without a fix that make a fault
pub const FAULT_AFTER_NO_FIX: u32 = 3;

/// What the schedule looks at
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inputs {
    /// The commissioning hold is set (sessions every orbit)
    pub leop: bool,
    /// The gates hold: Jetson off, SOC above the floor
    pub allowed: bool,
    /// When the latest accepted fix arrived, ms (`None`: none yet)
    pub fix_ms: Option<u64>,
}

/// How a session ended
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// With a fix
    Fix,
    /// Without one, after [`SESSION_MAX_MS`]
    NoFix,
    /// Stopped by the gates
    Stopped,
}

impl Outcome {
    /// For the console
    pub const fn text(self) -> &'static str {
        match self {
            Outcome::Fix => "fix",
            Outcome::NoFix => "no fix",
            Outcome::Stopped => "stopped by the gates",
        }
    }
}

/// What a pass changed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing
    None,
    /// Power the receiver
    PowerOn,
    /// Remove its power, and why
    PowerOff(Outcome),
}

/// The session schedule
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sessions {
    on_since_ms: Option<u64>,
    /// The session's first accepted fix, ms
    first_fix_ms: Option<u64>,
    next_due_ms: u64,
    requested: bool,
    /// Sessions in a row without a fix
    pub no_fix_run: u32,
    /// Sessions run
    pub sessions: u32,
    /// Sessions that gave a fix
    pub fixes: u32,
}

impl Default for Sessions {
    fn default() -> Self {
        Sessions::new()
    }
}

impl Sessions {
    /// At boot: the first session after [`FIRST_SESSION_DELAY_MS`]
    pub const fn new() -> Sessions {
        Sessions {
            on_since_ms: None,
            first_fix_ms: None,
            next_due_ms: FIRST_SESSION_DELAY_MS,
            requested: false,
            no_fix_run: 0,
            sessions: 0,
            fixes: 0,
        }
    }

    /// GNSS_REQUEST: a session as soon as the gates allow
    pub fn request(&mut self) {
        self.requested = true;
    }

    /// Whether a session is running (the receiver powered)
    pub const fn session_on(&self) -> bool {
        self.on_since_ms.is_some()
    }

    const fn period(leop: bool) -> u64 {
        if leop { LEOP_PERIOD_MS } else { OPS_PERIOD_MS }
    }

    fn end(&mut self, now_ms: u64, next_ms: u64, outcome: Outcome) -> Action {
        self.on_since_ms = None;
        self.first_fix_ms = None;
        self.next_due_ms = now_ms.saturating_add(next_ms);
        match outcome {
            Outcome::Fix => {
                self.fixes = self.fixes.saturating_add(1);
                self.no_fix_run = 0;
            }
            Outcome::NoFix => self.no_fix_run = self.no_fix_run.saturating_add(1),
            Outcome::Stopped => {}
        }
        Action::PowerOff(outcome)
    }

    /// One pass at `now_ms`
    pub fn step(&mut self, now_ms: u64, inp: &Inputs) -> Action {
        if let Some(since) = self.on_since_ms {
            if !inp.allowed {
                return self.end(now_ms, RETRY_MS, Outcome::Stopped);
            }
            if self.first_fix_ms.is_none() {
                self.first_fix_ms = inp.fix_ms.filter(|&f| f >= since);
            }
            if let Some(fix) = self.first_fix_ms {
                if now_ms.saturating_sub(fix) >= AFTER_FIX_MS {
                    return self.end(now_ms, Self::period(inp.leop), Outcome::Fix);
                }
            } else if now_ms.saturating_sub(since) >= SESSION_MAX_MS {
                return self.end(now_ms, Self::period(inp.leop), Outcome::NoFix);
            } else {
                // Still acquiring
            }
            return Action::None;
        }
        if (self.requested || now_ms >= self.next_due_ms) && inp.allowed {
            self.requested = false;
            self.on_since_ms = Some(now_ms);
            self.sessions = self.sessions.saturating_add(1);
            return Action::PowerOn;
        }
        Action::None
    }

    /// Whether the latest run of sessions without a fix makes a fault (true
    /// once per [`FAULT_AFTER_NO_FIX`] in a row)
    pub const fn no_fix_fault(&self) -> bool {
        self.no_fix_run > 0 && self.no_fix_run.is_multiple_of(FAULT_AFTER_NO_FIX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GO: Inputs = Inputs { leop: true, allowed: true, fix_ms: None };

    #[test]
    fn first_session_after_the_delay_then_a_fix_ends_it() {
        let mut s = Sessions::new();
        assert_eq!(s.step(0, &GO), Action::None);
        assert!(!s.session_on());
        let t = FIRST_SESSION_DELAY_MS;
        assert_eq!(s.step(t, &GO), Action::PowerOn);
        assert!(s.session_on());
        assert_eq!(s.step(t + 60_000, &GO), Action::None, "acquiring");
        let fix = Inputs { fix_ms: Some(t + 80_000), ..GO };
        assert_eq!(s.step(t + 90_000, &fix), Action::None, "a few more records");
        // Later fixes do not extend the session: it ends 30 s after the first
        let later = Inputs { fix_ms: Some(t + 105_000), ..GO };
        assert_eq!(s.step(t + 109_999, &later), Action::None);
        assert_eq!(s.step(t + 110_000, &later), Action::PowerOff(Outcome::Fix));
        assert_eq!((s.sessions, s.fixes, s.no_fix_run), (1, 1, 0));
        assert!(!Outcome::Fix.text().is_empty());
        assert!(!Outcome::NoFix.text().is_empty());
        assert!(!Outcome::Stopped.text().is_empty());
        // Next one an orbit later (LEOP); a fix older than the session does not count
        let old = Inputs { fix_ms: Some(t + 80_000), ..GO };
        assert_eq!(s.step(t + 110_000 + LEOP_PERIOD_MS - 1, &old), Action::None);
        let start = t + 110_000 + LEOP_PERIOD_MS;
        assert_eq!(s.step(start, &old), Action::PowerOn);
        assert_eq!(s.step(start + AFTER_FIX_MS + 1, &old), Action::None, "the old fix is not this session's");
    }

    #[test]
    fn no_fix_times_out_and_three_in_a_row_are_a_fault() {
        let mut s = Sessions::new();
        let ops = Inputs { leop: false, ..GO };
        let mut t = FIRST_SESSION_DELAY_MS;
        for n in 1..=FAULT_AFTER_NO_FIX {
            assert_eq!(s.step(t, &ops), Action::PowerOn);
            assert_eq!(s.step(t + SESSION_MAX_MS - 1, &ops), Action::None);
            assert_eq!(s.step(t + SESSION_MAX_MS, &ops), Action::PowerOff(Outcome::NoFix));
            assert_eq!(s.no_fix_fault(), n == FAULT_AFTER_NO_FIX, "{n}");
            t += SESSION_MAX_MS + OPS_PERIOD_MS;
        }
        assert_eq!(s.no_fix_run, FAULT_AFTER_NO_FIX);
    }

    #[test]
    fn gates_stop_and_hold_sessions_and_a_request_starts_one() {
        let mut s = Sessions::new();
        let shut = Inputs { allowed: false, ..GO };
        let t = FIRST_SESSION_DELAY_MS;
        assert_eq!(s.step(t, &shut), Action::None, "due but gated");
        assert_eq!(s.step(t + 1000, &GO), Action::PowerOn);
        assert_eq!(s.step(t + 2000, &shut), Action::PowerOff(Outcome::Stopped));
        assert_eq!(s.no_fix_run, 0);
        assert_eq!(s.step(t + 3000, &GO), Action::None, "retry later");
        s.request();
        assert_eq!(s.step(t + 4000, &shut), Action::None, "a request obeys the gates");
        assert_eq!(s.step(t + 5000, &GO), Action::PowerOn);
        assert_eq!(Sessions::default(), Sessions::new());
    }
}
