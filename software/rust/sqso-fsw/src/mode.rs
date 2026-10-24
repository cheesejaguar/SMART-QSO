//! The guarded mode manager (SW-14; `state_machine.h`).
//!
//! [`transition`] decides every (state, event) pair explicitly: each state's
//! `match` lists every event, with no wildcard arm. Adding a state or an
//! event therefore fails to compile until every state says what it does
//! with it, so a transition can neither be forgotten nor shadowed by an
//! earlier row (the C table had shadowed POWER_CRITICAL rows that made
//! EMERGENCY unreachable).
//!
//! The entry action of each state sets the EPS power mode ([`entry`]), so
//! the EPS holds the only power mode and this module decides it. ACTIVE
//! powers the payload, so every way into ACTIVE carries
//! [`Guard::ActiveAllowed`] (battery at or above the payload threshold and
//! the payload guard's consent).
//!
//! @requirement SRS-F012 Implement power modes SAFE, IDLE, ACTIVE (formal state machine)

/// Operational states (values are the C `SmState_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u32)]
pub enum State {
    /// Initial power-on state
    Boot = 0,
    /// Minimum power, beacon only
    Safe = 1,
    /// Attitude stabilization after deployment
    Detumble = 2,
    /// Normal operations without the AI payload
    Idle = 3,
    /// Full operations with the AI payload
    Active = 4,
    /// Battery critical while SAFE: beacon shed too
    Emergency = 5,
}

impl State {
    /// Every state
    pub const ALL: [State; 6] =
        [State::Boot, State::Safe, State::Detumble, State::Idle, State::Active, State::Emergency];

    /// From the C value
    pub const fn from_raw(raw: u32) -> Option<State> {
        match raw {
            0 => Some(State::Boot),
            1 => Some(State::Safe),
            2 => Some(State::Detumble),
            3 => Some(State::Idle),
            4 => Some(State::Active),
            5 => Some(State::Emergency),
            _ => None,
        }
    }

    /// Name for logs and telemetry
    pub const fn name(self) -> &'static core::ffi::CStr {
        match self {
            State::Boot => c"BOOT",
            State::Safe => c"SAFE",
            State::Detumble => c"DETUMBLE",
            State::Idle => c"IDLE",
            State::Active => c"ACTIVE",
            State::Emergency => c"EMERGENCY",
        }
    }
}

/// Events (values are the C `SmEvent_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u32)]
pub enum Event {
    /// No event
    None = 0,
    /// Boot sequence completed
    BootComplete = 1,
    /// Detumble achieved
    DetumbleComplete = 2,
    /// Communication link established
    CommAcquired = 3,
    /// Sufficient power available
    PowerOk = 4,
    /// Low power
    PowerLow = 5,
    /// Critical power
    PowerCritical = 6,
    /// Thermal limit exceeded
    ThermalFault = 7,
    /// Communication timeout
    CommTimeout = 8,
    /// Watchdog timeout
    WatchdogTimeout = 9,
    /// Ground command: SAFE
    GroundCmdSafe = 10,
    /// Ground command: IDLE
    GroundCmdIdle = 11,
    /// Ground command: ACTIVE
    GroundCmdActive = 12,
    /// AI payload ready
    AiReady = 13,
    /// AI payload fault
    AiFault = 14,
    /// Recovery successful
    RecoveryOk = 15,
}

impl Event {
    /// Every event
    pub const ALL: [Event; 16] = [
        Event::None,
        Event::BootComplete,
        Event::DetumbleComplete,
        Event::CommAcquired,
        Event::PowerOk,
        Event::PowerLow,
        Event::PowerCritical,
        Event::ThermalFault,
        Event::CommTimeout,
        Event::WatchdogTimeout,
        Event::GroundCmdSafe,
        Event::GroundCmdIdle,
        Event::GroundCmdActive,
        Event::AiReady,
        Event::AiFault,
        Event::RecoveryOk,
    ];

    /// From the C value
    pub const fn from_raw(raw: u32) -> Option<Event> {
        let mut i = 0;
        while i < Event::ALL.len() {
            #[allow(clippy::indexing_slicing)] // i < len
            let e = Event::ALL[i];
            if e as u32 == raw {
                return Some(e);
            }
            i += 1;
        }
        None
    }

    /// Name for logs and telemetry
    pub const fn name(self) -> &'static core::ffi::CStr {
        match self {
            Event::None => c"NONE",
            Event::BootComplete => c"BOOT_COMPLETE",
            Event::DetumbleComplete => c"DETUMBLE_COMPLETE",
            Event::CommAcquired => c"COMM_ACQUIRED",
            Event::PowerOk => c"POWER_OK",
            Event::PowerLow => c"POWER_LOW",
            Event::PowerCritical => c"POWER_CRITICAL",
            Event::ThermalFault => c"THERMAL_FAULT",
            Event::CommTimeout => c"COMM_TIMEOUT",
            Event::WatchdogTimeout => c"WATCHDOG_TIMEOUT",
            Event::GroundCmdSafe => c"GROUND_CMD_SAFE",
            Event::GroundCmdIdle => c"GROUND_CMD_IDLE",
            Event::GroundCmdActive => c"GROUND_CMD_ACTIVE",
            Event::AiReady => c"AI_READY",
            Event::AiFault => c"AI_FAULT",
            Event::RecoveryOk => c"RECOVERY_OK",
        }
    }
}

/// Conditions a transition can require
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Guard {
    /// Battery at or above [`SOC_PAYLOAD`] and the payload guard allows the
    /// payload on (entry to ACTIVE)
    ActiveAllowed,
}

/// Where an event leads from a state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Transition {
    /// Target state
    pub to: State,
    /// Condition, if any
    pub guard: Option<Guard>,
}

const fn to(state: State) -> Option<Transition> {
    Some(Transition { to: state, guard: None })
}

const fn guarded(state: State, guard: Guard) -> Option<Transition> {
    Some(Transition { to: state, guard: Some(guard) })
}

/// The transition table. `None`: the event is ignored in that state.
pub const fn transition(state: State, event: Event) -> Option<Transition> {
    use Event as E;
    use State as S;
    match state {
        S::Boot => match event {
            E::BootComplete => to(S::Detumble),
            E::PowerCritical => to(S::Safe),
            E::None
            | E::DetumbleComplete
            | E::CommAcquired
            | E::PowerOk
            | E::PowerLow
            | E::ThermalFault
            | E::CommTimeout
            | E::WatchdogTimeout
            | E::GroundCmdSafe
            | E::GroundCmdIdle
            | E::GroundCmdActive
            | E::AiReady
            | E::AiFault
            | E::RecoveryOk => None,
        },
        S::Detumble => match event {
            E::DetumbleComplete => to(S::Idle),
            E::PowerLow | E::PowerCritical | E::WatchdogTimeout | E::GroundCmdSafe => to(S::Safe),
            E::None
            | E::BootComplete
            | E::CommAcquired
            | E::PowerOk
            | E::ThermalFault
            | E::CommTimeout
            | E::GroundCmdIdle
            | E::GroundCmdActive
            | E::AiReady
            | E::AiFault
            | E::RecoveryOk => None,
        },
        S::Safe => match event {
            E::PowerOk | E::RecoveryOk | E::GroundCmdIdle => to(S::Idle),
            E::PowerCritical => to(S::Emergency),
            E::None
            | E::BootComplete
            | E::DetumbleComplete
            | E::CommAcquired
            | E::PowerLow
            | E::ThermalFault
            | E::CommTimeout
            | E::WatchdogTimeout
            | E::GroundCmdSafe
            | E::GroundCmdActive
            | E::AiReady
            | E::AiFault => None,
        },
        S::Idle => match event {
            E::AiReady | E::GroundCmdActive => guarded(S::Active, Guard::ActiveAllowed),
            E::PowerLow
            | E::PowerCritical
            | E::ThermalFault
            | E::CommTimeout
            | E::WatchdogTimeout
            | E::GroundCmdSafe => to(S::Safe),
            E::None
            | E::BootComplete
            | E::DetumbleComplete
            | E::CommAcquired
            | E::PowerOk
            | E::GroundCmdIdle
            | E::AiFault
            | E::RecoveryOk => None,
        },
        S::Active => match event {
            E::AiFault | E::PowerLow | E::CommTimeout | E::GroundCmdIdle => to(S::Idle),
            E::PowerCritical | E::ThermalFault | E::WatchdogTimeout | E::GroundCmdSafe => to(S::Safe),
            E::None
            | E::BootComplete
            | E::DetumbleComplete
            | E::CommAcquired
            | E::PowerOk
            | E::GroundCmdActive
            | E::AiReady
            | E::RecoveryOk => None,
        },
        S::Emergency => match event {
            E::PowerOk | E::RecoveryOk | E::GroundCmdSafe => to(S::Safe),
            E::None
            | E::BootComplete
            | E::DetumbleComplete
            | E::CommAcquired
            | E::PowerLow
            | E::PowerCritical
            | E::ThermalFault
            | E::CommTimeout
            | E::WatchdogTimeout
            | E::GroundCmdIdle
            | E::GroundCmdActive
            | E::AiReady
            | E::AiFault => None,
        },
    }
}

/// EPS power mode (values are the C `PowerMode_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum PowerMode {
    /// Minimum power, beacon only
    Safe = 0,
    /// Normal operations, no AI payload
    Idle = 1,
    /// Full operations including the AI payload
    Active = 2,
}

/// What entering a state does
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entry {
    /// EPS power mode to set
    pub power_mode: PowerMode,
    /// Beacon switch to set (`None`: leave it)
    pub beacon: Option<bool>,
}

/// Entry action of a state
pub const fn entry(state: State) -> Entry {
    match state {
        // SAFE keeps the beacon
        State::Boot | State::Safe => Entry { power_mode: PowerMode::Safe, beacon: Some(true) },
        // Battery critical while already SAFE: shed the beacon too. The
        // receiver stays on for telecommand (47 CFR 97.207(b))
        State::Emergency => Entry { power_mode: PowerMode::Safe, beacon: Some(false) },
        // ADCS on, payload off
        State::Detumble | State::Idle => Entry { power_mode: PowerMode::Idle, beacon: None },
        State::Active => Entry { power_mode: PowerMode::Active, beacon: None },
    }
}

/// SOC from which the payload may run (`EPS_SOC_PAYLOAD_THRESHOLD`)
pub const SOC_PAYLOAD: f64 = 0.55;
/// SOC below which IDLE and ACTIVE go SAFE (`EPS_SOC_SAFE_THRESHOLD`)
pub const SOC_SAFE: f64 = 0.25;
/// SOC below which ACTIVE goes IDLE, and above which SAFE returns to IDLE
/// (`EPS_SOC_IDLE_THRESHOLD`)
pub const SOC_IDLE: f64 = 0.40;
/// SOC below which SAFE goes EMERGENCY (`EPS_SOC_EMERGENCY_THRESHOLD`)
pub const SOC_EMERGENCY: f64 = 0.15;

/// The event power conditions call for in a state, with hysteresis:
/// ACTIVE from [`SOC_PAYLOAD`] (sunlit), back to IDLE below [`SOC_IDLE`];
/// SAFE below [`SOC_SAFE`], back to IDLE above [`SOC_IDLE`]; EMERGENCY
/// only if SOC keeps falling in SAFE (below [`SOC_EMERGENCY`]), back to
/// SAFE at [`SOC_SAFE`]. A NaN SOC calls for nothing.
pub fn power_event(state: State, soc: f64, sunlit: bool) -> Event {
    match state {
        State::Active if soc < SOC_SAFE => Event::PowerCritical,
        State::Active if soc < SOC_IDLE => Event::PowerLow,
        State::Idle if soc < SOC_SAFE => Event::PowerCritical,
        State::Idle if sunlit && soc >= SOC_PAYLOAD => Event::AiReady,
        State::Safe if soc < SOC_EMERGENCY => Event::PowerCritical,
        State::Safe if soc > SOC_IDLE => Event::PowerOk,
        State::Emergency if soc >= SOC_SAFE => Event::PowerOk,
        State::Boot | State::Detumble if soc < SOC_SAFE => Event::PowerCritical,
        State::Boot | State::Detumble | State::Safe | State::Idle | State::Active | State::Emergency => Event::None,
    }
}

/// Mode manager state (the C `SmContext_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Machine {
    /// Current state
    pub current: State,
    /// Previous state
    pub previous: State,
    /// Transitions taken
    pub transitions: u32,
    /// Time of the last transition, ms
    pub last_transition_ms: u64,
    /// Time the current state was entered, ms
    pub state_entry_ms: u64,
}

/// What an event did
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// No transition for this event in this state
    NoTransition,
    /// The transition's guard refused it
    GuardFailed,
    /// The state changed; run the entry action of `to`
    Transitioned {
        /// State left
        from: State,
        /// State entered
        to: State,
    },
}

impl Machine {
    /// Start in `initial` (the caller runs its entry action)
    pub const fn new(initial: State, now_ms: u64) -> Self {
        Machine {
            current: initial,
            previous: initial,
            transitions: 0,
            last_transition_ms: now_ms,
            state_entry_ms: now_ms,
        }
    }

    /// Process an event; `guard` evaluates a guard when one applies
    pub fn process(&mut self, event: Event, now_ms: u64, guard: impl FnOnce(Guard) -> bool) -> Step {
        match transition(self.current, event) {
            None => Step::NoTransition,
            Some(Transition { guard: Some(g), .. }) if !guard(g) => Step::GuardFailed,
            Some(t) => {
                let from = self.current;
                self.enter(t.to, now_ms);
                Step::Transitioned { from, to: t.to }
            }
        }
    }

    /// Change state without the table (fault recovery only)
    pub fn force(&mut self, state: State, now_ms: u64) {
        self.enter(state, now_ms);
    }

    fn enter(&mut self, state: State, now_ms: u64) {
        self.previous = self.current;
        self.current = state;
        self.transitions = self.transitions.wrapping_add(1);
        self.last_transition_ms = now_ms;
        self.state_entry_ms = now_ms;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_is_reachable_from_boot() {
        let mut seen = [false; 6];
        seen[State::Boot as usize] = true;
        let mut changed = true;
        while changed {
            changed = false;
            for s in State::ALL {
                if !seen[s as usize] {
                    continue;
                }
                for e in Event::ALL {
                    if let Some(t) = transition(s, e)
                        && !seen[t.to as usize]
                    {
                        seen[t.to as usize] = true;
                        changed = true;
                    }
                }
            }
        }
        assert_eq!(seen, [true; 6]);
    }

    #[test]
    fn every_way_into_active_is_guarded() {
        for s in State::ALL {
            for e in Event::ALL {
                if let Some(t) = transition(s, e) {
                    assert_eq!(t.to == State::Active, t.guard == Some(Guard::ActiveAllowed), "{s:?} {e:?}");
                }
            }
        }
    }

    #[test]
    fn ground_cannot_jump_safe_to_active() {
        assert_eq!(transition(State::Safe, Event::GroundCmdActive), None);
    }

    #[test]
    fn emergency_is_reached_from_safe_only() {
        for s in State::ALL {
            for e in Event::ALL {
                if transition(s, e).is_some_and(|t| t.to == State::Emergency) {
                    assert_eq!((s, e), (State::Safe, Event::PowerCritical));
                }
            }
        }
    }

    #[test]
    fn guard_and_steps() {
        let mut m = Machine::new(State::Idle, 0);
        assert_eq!(m.process(Event::GroundCmdActive, 5, |_| false), Step::GuardFailed);
        assert_eq!(m.current, State::Idle);
        assert_eq!(
            m.process(Event::GroundCmdActive, 6, |_| true),
            Step::Transitioned { from: State::Idle, to: State::Active }
        );
        assert_eq!((m.previous, m.transitions, m.state_entry_ms), (State::Idle, 1, 6));
        assert_eq!(m.process(Event::BootComplete, 7, |_| true), Step::NoTransition);
    }

    #[test]
    fn power_policy_hysteresis() {
        use Event as E;
        use State as S;
        let cases = [
            (S::Active, 0.20, true, E::PowerCritical),
            (S::Active, 0.30, true, E::PowerLow),
            (S::Active, 0.45, false, E::None),
            (S::Idle, 0.20, true, E::PowerCritical),
            (S::Idle, 0.55, true, E::AiReady),
            (S::Idle, 0.55, false, E::None),
            (S::Idle, 0.50, true, E::None),
            (S::Safe, 0.10, true, E::PowerCritical),
            (S::Safe, 0.40, true, E::None),
            (S::Safe, 0.41, true, E::PowerOk),
            (S::Emergency, 0.24, true, E::None),
            (S::Emergency, 0.25, true, E::PowerOk),
            (S::Boot, 0.2, true, E::PowerCritical),
            (S::Detumble, 0.9, true, E::None),
            (S::Safe, f64::NAN, true, E::None),
        ];
        for (s, soc, sun, expected) in cases {
            assert_eq!(power_event(s, soc, sun), expected, "{s:?} {soc}");
        }
    }

    #[test]
    fn raw_values_round_trip() {
        for s in State::ALL {
            assert_eq!(State::from_raw(s as u32), Some(s));
        }
        for e in Event::ALL {
            assert_eq!(Event::from_raw(e as u32), Some(e));
        }
        assert_eq!(State::from_raw(6), None);
        assert_eq!(Event::from_raw(16), None);
    }
}
