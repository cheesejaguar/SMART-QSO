//! `state_machine.h`: the guarded mode manager (`sqso_fsw::mode`) on the C
//! context `SmContext_t`.
//!
//! @requirement SRS-F018 Log every operating-mode transition ([Proposed])

use core::ffi::c_char;

use sqso_fsw::mode::{self, Event, Guard, Machine, State, Step, Transition};

use crate::c::{self, SmContext, fault_type, rc, severity};
use crate::ptr;

/// `SmResult_t`
pub(crate) mod sm_result {
    pub(crate) const OK: u32 = 0;
    pub(crate) const NO_TRANSITION: u32 = 1;
    pub(crate) const GUARD_FAILED: u32 = 2;
    pub(crate) const INVALID_EVENT: u32 = 4;
}

/// `SmGuardFunc_t`
pub type SmGuardFunc = extern "C" fn() -> bool;
/// `SmActionFunc_t`
pub type SmActionFunc = extern "C" fn() -> i32;

/// `SmTransition_t`
#[repr(C)]
pub struct SmTransition {
    from_state: u32,
    event: u32,
    to_state: u32,
    guard: Option<SmGuardFunc>,
    action: Option<SmActionFunc>,
}

/// Guard: ACTIVE powers the payload, so the battery and the payload guard
/// must allow it
extern "C" fn guard_active_allowed() -> bool {
    // SAFETY: C accessors without pointer arguments
    unsafe { c::eps_get_soc() >= mode::SOC_PAYLOAD && c::payload_guard_may_enable(c::now_ms()) }
}

fn guard(g: Guard) -> bool {
    match g {
        Guard::ActiveAllowed => guard_active_allowed(),
    }
}

/// Run a state's entry action: the EPS power mode, then the beacon switch
fn enter(state: State) -> i32 {
    let entry = mode::entry(state);
    // SAFETY: C EPS calls with plain values
    unsafe {
        let result = c::eps_set_power_mode(entry.power_mode as u32, c::eps_get_soc());
        if let Some(on) = entry.beacon {
            let _ = c::eps_control_beacon(on);
        }
        result
    }
}

const TRANSITION_COUNT: usize = {
    let mut n = 0;
    let mut s = 0;
    while s < State::ALL.len() {
        let mut e = 0;
        while e < Event::ALL.len() {
            #[allow(clippy::indexing_slicing)] // bounded by the loops, at compile time
            if mode::transition(State::ALL[s], Event::ALL[e]).is_some() {
                n += 1;
            }
            e += 1;
        }
        s += 1;
    }
    n
};

/// The table `sm_transitions()` exposes, derived at compile time from
/// `sqso_fsw::mode::transition` (so it cannot differ from what runs)
static TRANSITIONS: [SmTransition; TRANSITION_COUNT] = {
    const BLANK: SmTransition = SmTransition { from_state: 0, event: 0, to_state: 0, guard: None, action: None };
    let mut table = [BLANK; TRANSITION_COUNT];
    let mut n = 0;
    let mut s = 0;
    while s < State::ALL.len() {
        let mut e = 0;
        while e < Event::ALL.len() {
            #[allow(clippy::indexing_slicing)] // bounded by the loops and the count, at compile time
            {
                let (state, event) = (State::ALL[s], Event::ALL[e]);
                if let Some(Transition { to, guard }) = mode::transition(state, event) {
                    table[n] = SmTransition {
                        from_state: state as u32,
                        event: event as u32,
                        to_state: to as u32,
                        guard: match guard {
                            Some(Guard::ActiveAllowed) => Some(guard_active_allowed as SmGuardFunc),
                            None => None,
                        },
                        action: None,
                    };
                    n += 1;
                }
            }
            e += 1;
        }
        s += 1;
    }
    table
};

fn machine(ctx: &SmContext) -> Option<Machine> {
    Some(Machine {
        current: State::from_raw(ctx.current_state)?,
        previous: State::from_raw(ctx.previous_state).unwrap_or(State::Boot),
        transitions: ctx.transition_count,
        last_transition_ms: ctx.last_transition_ms,
        state_entry_ms: ctx.state_entry_ms,
    })
}

fn store(ctx: &mut SmContext, m: &Machine) {
    ctx.current_state = m.current as u32;
    ctx.previous_state = m.previous as u32;
    ctx.transition_count = m.transitions;
    ctx.last_transition_ms = m.last_transition_ms;
    ctx.state_entry_ms = m.state_entry_ms;
}

/// Log the new state and run its entry action
fn entered(state: State) {
    c::log(fault_type::MODE_CHANGE, severity::INFO, state.name(), 0.0);
    if enter(state) != rc::OK {
        c::log(fault_type::MODE_CHANGE, severity::WARNING, c"Entry action failed", 0.0);
    }
}

/// Initialize the state machine in `initial_state` (its entry action runs)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_init(ctx: *mut SmContext, initial_state: u32) -> i32 {
    // SAFETY: the C contract
    let Some(ctx) = (unsafe { ptr::out(ctx) }) else { return rc::NULL_PTR };
    let Some(initial) = State::from_raw(initial_state) else { return rc::INVALID };
    store(ctx, &Machine::new(initial, c::now_ms()));
    ctx.is_initialized = true;
    let _ = enter(initial);
    rc::OK
}

/// Process an event: (status, SmResult_t)
pub(crate) fn process(ctx: *mut SmContext, event: u32) -> (i32, u32) {
    // SAFETY: ctx comes from C (system_state.c or a caller), NULL-checked
    let Some(ctx) = (unsafe { ptr::out(ctx) }) else { return (rc::NULL_PTR, sm_result::NO_TRANSITION) };
    if !ctx.is_initialized {
        return (rc::INVALID, sm_result::NO_TRANSITION);
    }
    let Some(event) = Event::from_raw(event) else { return (rc::OK, sm_result::INVALID_EVENT) };
    let Some(mut m) = machine(ctx) else { return (rc::OK, sm_result::NO_TRANSITION) };
    let step = m.process(event, c::now_ms(), guard);
    match step {
        Step::NoTransition => (rc::OK, sm_result::NO_TRANSITION),
        Step::GuardFailed => (rc::OK, sm_result::GUARD_FAILED),
        Step::Transitioned { to, .. } => {
            store(ctx, &m);
            entered(to);
            (rc::OK, sm_result::OK)
        }
    }
}

/// Process an event and execute any resulting transition
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_process_event(ctx: *mut SmContext, event: u32, result: *mut u32) -> i32 {
    let (status, outcome) = process(ctx, event);
    if status == rc::OK {
        // SAFETY: the C contract: result is NULL or points to an SmResult_t
        if let Some(out) = unsafe { ptr::out(result) } {
            *out = outcome;
        }
    }
    status
}

/// Current state
pub(crate) fn current(ctx: *const SmContext) -> Result<u32, i32> {
    // SAFETY: ctx comes from C, NULL-checked
    let ctx = unsafe { ptr::in_(ctx) }.ok_or(rc::NULL_PTR)?;
    if ctx.is_initialized { Ok(ctx.current_state) } else { Err(rc::INVALID) }
}

/// Current state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_get_state(ctx: *const SmContext, state: *mut u32) -> i32 {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::out(state) }) else { return rc::NULL_PTR };
    match current(ctx) {
        Ok(s) => {
            *out = s;
            rc::OK
        }
        Err(e) => e,
    }
}

/// The event power conditions call for in a state
#[unsafe(no_mangle)]
pub extern "C" fn sm_power_event(state: u32, soc: f64, sunlit: bool) -> u32 {
    State::from_raw(state).map_or(Event::None, |s| mode::power_event(s, soc, sunlit)) as u32
}

/// The transition table, read-only
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_transitions(count: *mut usize) -> *const SmTransition {
    // SAFETY: the C contract: count is NULL or points to a size_t
    if let Some(out) = unsafe { ptr::out(count) } {
        *out = TRANSITION_COUNT;
    }
    TRANSITIONS.as_ptr()
}

/// State name, or "UNKNOWN"
#[unsafe(no_mangle)]
pub extern "C" fn sm_get_state_name(state: u32) -> *const c_char {
    State::from_raw(state).map_or(c"UNKNOWN", State::name).as_ptr()
}

/// Event name, or "UNKNOWN"
#[unsafe(no_mangle)]
pub extern "C" fn sm_get_event_name(event: u32) -> *const c_char {
    Event::from_raw(event).map_or(c"UNKNOWN", Event::name).as_ptr()
}

/// Time in the current state, ms
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_get_state_duration(ctx: *const SmContext, duration_ms: *mut u64) -> i32 {
    // SAFETY: the C contract
    let (Some(ctx), Some(out)) = (unsafe { ptr::in_(ctx) }, unsafe { ptr::out(duration_ms) }) else {
        return rc::NULL_PTR;
    };
    if !ctx.is_initialized {
        return rc::INVALID;
    }
    *out = c::now_ms().wrapping_sub(ctx.state_entry_ms);
    rc::OK
}

/// Whether the current state has a transition for `event`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_is_transition_valid(ctx: *const SmContext, event: u32, is_valid: *mut bool) -> i32 {
    // SAFETY: the C contract
    let (Some(ctx), Some(out)) = (unsafe { ptr::in_(ctx) }, unsafe { ptr::out(is_valid) }) else {
        return rc::NULL_PTR;
    };
    if !ctx.is_initialized {
        return rc::INVALID;
    }
    *out = match (State::from_raw(ctx.current_state), Event::from_raw(event)) {
        (Some(s), Some(e)) => mode::transition(s, e).is_some(),
        _ => false,
    };
    rc::OK
}

/// Force a state (fault recovery only; bypasses the table and guards)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_force_state(ctx: *mut SmContext, new_state: u32) -> i32 {
    // SAFETY: the C contract
    let Some(ctx) = (unsafe { ptr::out(ctx) }) else { return rc::NULL_PTR };
    let Some(state) = State::from_raw(new_state) else { return rc::INVALID };
    c::log(fault_type::RECOVERY, severity::WARNING, c"Forced state change", 0.0);
    let now = c::now_ms();
    ctx.previous_state = ctx.current_state;
    ctx.current_state = state as u32;
    ctx.transition_count = ctx.transition_count.wrapping_add(1);
    ctx.last_transition_ms = now;
    ctx.state_entry_ms = now;
    let _ = enter(state);
    rc::OK
}

/// Transitions taken and the previous state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_get_statistics(
    ctx: *const SmContext,
    transition_count: *mut u32,
    previous_state: *mut u32,
) -> i32 {
    // SAFETY: the C contract
    let Some(ctx) = (unsafe { ptr::in_(ctx) }) else { return rc::NULL_PTR };
    if !ctx.is_initialized {
        return rc::INVALID;
    }
    // SAFETY: the C contract: each is NULL or points to its type
    if let Some(out) = unsafe { ptr::out(transition_count) } {
        *out = ctx.transition_count;
    }
    // SAFETY: as above
    if let Some(out) = unsafe { ptr::out(previous_state) } {
        *out = ctx.previous_state;
    }
    rc::OK
}
