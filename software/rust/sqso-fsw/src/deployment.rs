//! Deployment sequence: the 30-minute wait, burn wires, persistence and
//! reset rules (`deployment.h`).
//!
//! Timing uses elapsed ticks, not absolute timestamps, and the sequence is
//! persisted so resets neither shorten the 30-minute wait nor repeat burns:
//!  - 30-minute wait (CDS 2.4.4): a reset that lost power restarts it (a
//!    separation-switch toggle removes power, CDS 2.3.2.4); a watchdog or
//!    software reset resumes it; a stowed switch restarts it.
//!  - Burns: each element has its own burn and sense line; a burn needs the
//!    master arm and the element line. The attempt is persisted before the
//!    line is driven, and nothing fires if that fails. An element whose
//!    sense reads deployed is never fired. A reset during a burn spends the
//!    attempt. Attempts are bounded across resets.
//!  - COMPLETE and FAILED ignore the switch (re-burning is not harmless);
//!    only GSE ARM resets them. ARMED does not progress while powered.
//!  - Blank or corrupt record: start over with zero attempts (reported);
//!    sense lines skip anything already deployed.
//!
//! @requirement SYS-221 Deployables wait 30 minutes after separation; switch toggle resets the timer

use core::ffi::CStr;
use core::fmt;

use crate::contact_monitor::carry_seconds;
use crate::nvm::{self, Flash, RecordStatus};
use crate::rf_silence::Reset;

/// Wait after separation, ms (CDS)
pub const WAIT_TIME_MS: u32 = 30 * 60 * 1000;
/// Wait after separation, s
pub const WAIT_TIME_S: u32 = WAIT_TIME_MS / 1000;
/// Antenna burn time, ms
pub const ANTENNA_BURN_TIME_MS: u32 = 3000;
/// Solar-panel burn time, ms
pub const SOLAR_BURN_TIME_MS: u32 = 5000;
/// Attempts per element, across resets
pub const MAX_RETRIES: u8 = 3;
/// Wait before retrying an unconfirmed deployment, ms
pub const RETRY_DELAY_MS: u32 = 60_000;
/// Wait-count persistence interval, s
pub const PERSIST_INTERVAL_S: u32 = 60;

/// Record ("DEPL"): state | attempts (2) | flags | wait seconds (4)
pub const RECORD_MAGIC: u32 = 0x4445_504C;
const FLAG_ANTENNA_DEPLOYED: u8 = 0x01;
const FLAG_SOLAR_DEPLOYED: u8 = 0x02;
const FLAG_ENABLED: u8 = 0x04;
const FLAG_SEPARATED: u8 = 0x08;

/// `DeploymentState_t`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum State {
    /// Waiting for switch release
    Inhibited = 0,
    /// Separation detected, waiting out the timer
    Waiting = 1,
    /// Ready to deploy the antenna
    AntennaPending = 2,
    /// Antenna burn in progress
    AntennaActive = 3,
    /// Verifying the antenna
    AntennaVerify = 4,
    /// Ready to deploy the solar panels
    SolarPending = 5,
    /// Solar burn in progress
    SolarActive = 6,
    /// Verifying the solar panels
    SolarVerify = 7,
    /// All deployments confirmed
    Complete = 8,
    /// Failed after the retries
    Failed = 9,
    /// GSE-armed for flight; starts at the next boot
    Armed = 10,
}

/// `DEPLOY_STATE_COUNT`
pub const STATE_COUNT: u32 = 11;

impl State {
    /// Every state, in C order
    pub const ALL: [State; STATE_COUNT as usize] = [
        State::Inhibited,
        State::Waiting,
        State::AntennaPending,
        State::AntennaActive,
        State::AntennaVerify,
        State::SolarPending,
        State::SolarActive,
        State::SolarVerify,
        State::Complete,
        State::Failed,
        State::Armed,
    ];

    /// The state with this C value
    pub fn from_raw(raw: u32) -> Option<State> {
        State::ALL.into_iter().find(|s| *s as u32 == raw)
    }

    /// `deployment_state_name()`
    pub const fn name(self) -> &'static CStr {
        match self {
            State::Inhibited => c"INHIBITED",
            State::Waiting => c"WAITING",
            State::AntennaPending => c"ANTENNA_PENDING",
            State::AntennaActive => c"ANTENNA_ACTIVE",
            State::AntennaVerify => c"ANTENNA_VERIFY",
            State::SolarPending => c"SOLAR_PENDING",
            State::SolarActive => c"SOLAR_ACTIVE",
            State::SolarVerify => c"SOLAR_VERIFY",
            State::Complete => c"COMPLETE",
            State::Failed => c"FAILED",
            State::Armed => c"ARMED",
        }
    }

    fn label(self) -> &'static str {
        self.name().to_str().unwrap_or("?")
    }
}

/// `DeploymentElement_t`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Element {
    /// Antenna
    Antenna = 0,
    /// Solar panels
    SolarPanel = 1,
}

/// `DEPLOY_ELEMENT_COUNT`
pub const ELEMENT_COUNT: u32 = 2;

impl Element {
    /// The element with this C value
    pub const fn from_raw(raw: u32) -> Option<Element> {
        match raw {
            0 => Some(Element::Antenna),
            1 => Some(Element::SolarPanel),
            _ => None,
        }
    }

    const fn burn_time_ms(self) -> u32 {
        match self {
            Element::Antenna => ANTENNA_BURN_TIME_MS,
            Element::SolarPanel => SOLAR_BURN_TIME_MS,
        }
    }

    const fn next(self) -> State {
        match self {
            Element::Antenna => State::SolarPending,
            Element::SolarPanel => State::Complete,
        }
    }

    const fn pending(self) -> State {
        match self {
            Element::Antenna => State::AntennaPending,
            Element::SolarPanel => State::SolarPending,
        }
    }
}

/// `DeploymentElementStatus_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ElementStatus {
    /// Deployment confirmed
    pub deployed: bool,
    /// At least one attempt made
    pub deploy_attempted: bool,
    /// Attempts (persisted)
    pub attempt_count: u8,
    /// Seconds after separation when confirmed
    pub deploy_time_s: u32,
    /// Current sense reading
    pub switch_state: bool,
}

/// `DeploymentTelemetry_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Telemetry {
    /// Current state
    pub state: u8,
    /// Antenna deployed (0/1)
    pub antenna_deployed: u8,
    /// Antenna attempts
    pub antenna_attempts: u8,
    /// Solar panels deployed (0/1)
    pub solar_deployed: u8,
    /// Solar attempts
    pub solar_attempts: u8,
    /// Seconds since separation (0 before separation)
    pub time_since_sep_s: u32,
    /// Deployment inhibited by command (0/1)
    pub inhibit_active: u8,
}

/// A line the sequence drives
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    /// The master arm (`GPIO_PIN_DEPLOY_EN`)
    Arm,
    /// An element's burn wire
    Burn(Element),
}

/// Flight-log level
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// `LOG_LEVEL_DEBUG`
    Debug,
    /// `LOG_LEVEL_INFO`
    Info,
}

/// Events for the fault log
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Report {
    /// The element failed after [`MAX_RETRIES`] attempts
    Failed(Element),
    /// An attempt could not be persisted, so the burn was withheld
    NotPersisted,
    /// Boot: the record could not be read; starting over
    Unreadable,
}

/// The lines, sense inputs, separation switch, flash and logs
pub trait DeployIo: Flash {
    /// Configure the arm and burn lines as outputs
    fn configure_outputs(&mut self);
    /// Configure the sense inputs and the separation switch
    fn configure_inputs(&mut self);
    /// Drive a line
    fn set(&mut self, line: Line, on: bool);
    /// Whether an element's sense line reads deployed
    fn sensed(&mut self, element: Element) -> bool;
    /// Whether the separation switches read stowed
    fn stowed(&mut self) -> bool;
    /// Write a flight-log line
    fn log(&mut self, level: Level, message: fmt::Arguments<'_>);
    /// Log a fault
    fn report(&mut self, report: Report);
}

/// Module state
#[derive(Clone, Copy, Debug)]
pub struct Deployment {
    region: u32,
    state: State,
    wait_elapsed_s: u32,
    state_elapsed_ms: u32,
    since_separation_s: u32,
    elements: [ElementStatus; 2],
    separation_detected: bool,
    deployment_enabled: bool,
    unpersisted_s: u32,
    wait_remainder_ms: u32,
    sep_remainder_ms: u32,
    persist_fault_logged: bool,
    initialized: bool,
}

impl Deployment {
    /// Before init: all zero (INHIBITED, disabled, nothing attempted)
    pub const fn new(region: u32) -> Self {
        const NONE: ElementStatus = ElementStatus {
            deployed: false,
            deploy_attempted: false,
            attempt_count: 0,
            deploy_time_s: 0,
            switch_state: false,
        };
        Deployment {
            region,
            state: State::Inhibited,
            wait_elapsed_s: 0,
            state_elapsed_ms: 0,
            since_separation_s: 0,
            elements: [NONE; 2],
            separation_detected: false,
            deployment_enabled: false,
            unpersisted_s: 0,
            wait_remainder_ms: 0,
            sep_remainder_ms: 0,
            persist_fault_logged: false,
            initialized: false,
        }
    }

    fn element(&self, e: Element) -> &ElementStatus {
        let [antenna, solar] = &self.elements;
        match e {
            Element::Antenna => antenna,
            Element::SolarPanel => solar,
        }
    }

    fn element_mut(&mut self, e: Element) -> &mut ElementStatus {
        let [antenna, solar] = &mut self.elements;
        match e {
            Element::Antenna => antenna,
            Element::SolarPanel => solar,
        }
    }

    /// Master arm off first, then both element lines
    fn burn_off<IO: DeployIo>(io: &mut IO) {
        io.set(Line::Arm, false);
        io.set(Line::Burn(Element::Antenna), false);
        io.set(Line::Burn(Element::SolarPanel), false);
    }

    /// Element line first, then the master arm: both are needed to burn
    fn burn_on<IO: DeployIo>(io: &mut IO, e: Element) {
        io.set(Line::Burn(e), true);
        io.set(Line::Arm, true);
    }

    fn state_byte(&self) -> u8 {
        u8::try_from(self.state as u32).unwrap_or(u8::MAX)
    }

    fn record(&self) -> [u8; 8] {
        let [antenna, solar] = &self.elements;
        let flags = (if antenna.deployed { FLAG_ANTENNA_DEPLOYED } else { 0 })
            | (if solar.deployed { FLAG_SOLAR_DEPLOYED } else { 0 })
            | (if self.deployment_enabled { FLAG_ENABLED } else { 0 })
            | (if self.separation_detected { FLAG_SEPARATED } else { 0 });
        let [w0, w1, w2, w3] = self.wait_elapsed_s.to_le_bytes();
        [self.state_byte(), antenna.attempt_count, solar.attempt_count, flags, w0, w1, w2, w3]
    }

    fn save<F: Flash>(&mut self, flash: &mut F) -> bool {
        if nvm::save(flash, self.region, 0, RECORD_MAGIC, &self.record()).is_err() {
            return false;
        }
        self.unpersisted_s = 0;
        self.persist_fault_logged = false;
        true
    }

    fn enter_state<IO: DeployIo>(&mut self, io: &mut IO, new: State) {
        io.log(Level::Debug, format_args!("State: {} -> {}", self.state.label(), new.label()));
        self.state = new;
        self.state_elapsed_ms = 0;
    }

    /// Back to waiting for separation with the count at zero
    fn restart_wait<IO: DeployIo>(&mut self, io: &mut IO) {
        self.wait_elapsed_s = 0;
        self.separation_detected = false;
        self.wait_remainder_ms = 0;
        self.enter_state(io, State::Inhibited);
    }

    /// Start from nothing: no attempts, nothing deployed, enabled
    fn fresh_start<IO: DeployIo>(&mut self, io: &mut IO) {
        self.state = State::Inhibited;
        self.wait_elapsed_s = 0;
        self.state_elapsed_ms = 0;
        self.since_separation_s = 0;
        self.elements = [ElementStatus::default(); 2];
        self.separation_detected = false;
        self.deployment_enabled = true;
        self.restart_wait(io);
    }

    fn mark_deployed<IO: DeployIo>(&mut self, io: &mut IO, e: Element) {
        let since = self.since_separation_s;
        let el = self.element_mut(e);
        el.deployed = true;
        el.deploy_time_s = since;
        self.enter_state(io, e.next());
        let _ = self.save(io);
        io.log(Level::Info, format_args!("Element {} deployed", e as u32));
    }

    fn process_pending<IO: DeployIo>(&mut self, io: &mut IO, e: Element, active: State) {
        // Idempotent: never fire an element that is already out
        if self.element(e).deployed || io.sensed(e) {
            self.mark_deployed(io, e);
            return;
        }
        if !self.deployment_enabled {
            return;
        }
        if self.element(e).attempt_count >= MAX_RETRIES {
            io.report(Report::Failed(e));
            self.enter_state(io, State::Failed);
            let _ = self.save(io);
            return;
        }

        // Persist the attempt (and the burning state) before driving the
        // line, so a reset can never repeat a burn; if it cannot be
        // persisted, do not burn
        let previous = self.state;
        let el = self.element_mut(e);
        let previously_attempted = el.deploy_attempted;
        el.attempt_count = el.attempt_count.saturating_add(1);
        el.deploy_attempted = true;
        self.enter_state(io, active);
        if !self.save(io) {
            let el = self.element_mut(e);
            el.attempt_count = el.attempt_count.saturating_sub(1);
            el.deploy_attempted = previously_attempted;
            self.state = previous;
            if !self.persist_fault_logged {
                self.persist_fault_logged = true;
                io.report(Report::NotPersisted);
            }
            return;
        }

        Self::burn_on(io, e);
        io.log(Level::Info, format_args!("Element {} burn attempt {}", e as u32, self.element(e).attempt_count));
    }

    fn process_active<IO: DeployIo>(&mut self, io: &mut IO, e: Element, verify: State) {
        if self.state_elapsed_ms >= e.burn_time_ms() {
            Self::burn_off(io);
            self.enter_state(io, verify);
            let _ = self.save(io);
        }
    }

    fn process_verify<IO: DeployIo>(&mut self, io: &mut IO, e: Element) {
        if io.sensed(e) {
            self.mark_deployed(io, e);
        } else if self.state_elapsed_ms >= RETRY_DELAY_MS {
            self.enter_state(io, e.pending());
            let _ = self.save(io);
        } else {
            // Keep waiting for the sense line
        }
    }

    fn process_waiting<IO: DeployIo>(&mut self, io: &mut IO, elapsed_ms: u32) {
        let (seconds, remainder) = carry_seconds(self.wait_remainder_ms, elapsed_ms);
        self.wait_remainder_ms = remainder;
        self.wait_elapsed_s = self.wait_elapsed_s.saturating_add(seconds);
        self.unpersisted_s = self.unpersisted_s.saturating_add(seconds);

        if self.wait_elapsed_s >= WAIT_TIME_S && self.deployment_enabled {
            self.enter_state(io, State::AntennaPending);
            let _ = self.save(io);
            io.log(Level::Info, format_args!("Wait complete, starting antenna deploy"));
        } else if self.unpersisted_s >= PERSIST_INTERVAL_S {
            let _ = self.save(io);
        } else {
            // Persist at the next interval
        }
    }

    /// Resume from a valid record. ARMED starts fresh; COMPLETE and FAILED
    /// are kept; other states resume unless power was lost (the wait
    /// restarts); a burn interrupted by a reset goes to verification.
    fn restore(&mut self, rec: &[u8; 8], state: State, reset: Reset) {
        let [_, antenna_attempts, solar_attempts, flags, w0, w1, w2, w3] = *rec;
        if state == State::Armed {
            return; // the sequence starts fresh at this boot
        }
        let [antenna, solar] = &mut self.elements;
        for (el, attempts) in [(antenna, antenna_attempts), (solar, solar_attempts)] {
            el.attempt_count = attempts;
            el.deploy_attempted = attempts > 0;
        }
        let [antenna, solar] = &mut self.elements;
        antenna.deployed = flags & FLAG_ANTENNA_DEPLOYED != 0;
        solar.deployed = flags & FLAG_SOLAR_DEPLOYED != 0;
        self.deployment_enabled = flags & FLAG_ENABLED != 0;

        match (state, reset) {
            (State::Complete | State::Failed, _) => {
                self.state = state;
                self.separation_detected = flags & FLAG_SEPARATED != 0;
            }
            // Power removed (a switch toggle does this): the wait restarts
            (_, Reset::PowerLost) => {}
            (_, Reset::PowerKept) => {
                self.separation_detected = flags & FLAG_SEPARATED != 0;
                self.wait_elapsed_s = u32::from_le_bytes([w0, w1, w2, w3]);
                // A reset during a burn spends the attempt: verify, never re-fire
                self.state = match state {
                    State::AntennaActive => State::AntennaVerify,
                    State::SolarActive => State::SolarVerify,
                    other => other,
                };
            }
        }
    }

    /// Boot: burn lines low, load the record and apply the reset rules
    /// (safe to call again: a simulated reset)
    pub fn init<IO: DeployIo>(&mut self, reset: Reset, io: &mut IO) {
        // Burn lines low before anything else
        io.configure_outputs();
        Self::burn_off(io);
        io.configure_inputs();

        self.fresh_start(io);
        self.unpersisted_s = 0;
        self.sep_remainder_ms = 0;
        self.persist_fault_logged = false;

        let mut rec = [0u8; 8];
        let loaded = nvm::load(io, self.region, 0, RECORD_MAGIC, &mut rec);
        let [state_raw, ..] = rec;
        match (loaded, State::from_raw(u32::from(state_raw))) {
            (Ok(RecordStatus::Valid), Some(state)) => self.restore(&rec, state, reset),
            (Ok(RecordStatus::Blank), _) => {} // first boot
            _ => io.report(Report::Unreadable),
        }

        let _ = self.save(io);
        self.initialized = true;
        io.log(Level::Info, format_args!("Deployment initialized in {}", self.state.label()));
    }

    /// Advance the timers and read the sense lines
    fn update_timers_and_sensors<IO: DeployIo>(&mut self, io: &mut IO, elapsed_ms: u32) {
        self.state_elapsed_ms = self.state_elapsed_ms.saturating_add(elapsed_ms);
        if self.separation_detected {
            let (seconds, remainder) = carry_seconds(self.sep_remainder_ms, elapsed_ms);
            self.since_separation_s = self.since_separation_s.saturating_add(seconds);
            self.sep_remainder_ms = remainder;
        }
        for e in [Element::Antenna, Element::SolarPanel] {
            let sensed = io.sensed(e);
            self.element_mut(e).switch_state = sensed;
        }
    }

    /// One step of the sequence for the current state
    fn step_sequence<IO: DeployIo>(&mut self, io: &mut IO, elapsed_ms: u32) {
        match self.state {
            State::Inhibited => {
                self.separation_detected = true;
                self.since_separation_s = 0;
                self.enter_state(io, State::Waiting);
                let _ = self.save(io);
                io.log(Level::Info, format_args!("Separation detected"));
            }
            State::Waiting => self.process_waiting(io, elapsed_ms),
            State::AntennaPending => self.process_pending(io, Element::Antenna, State::AntennaActive),
            State::AntennaActive => self.process_active(io, Element::Antenna, State::AntennaVerify),
            State::AntennaVerify => self.process_verify(io, Element::Antenna),
            State::SolarPending => self.process_pending(io, Element::SolarPanel, State::SolarActive),
            State::SolarActive => self.process_active(io, Element::SolarPanel, State::SolarVerify),
            State::SolarVerify => self.process_verify(io, Element::SolarPanel),
            // Terminal or waiting for a boot: handled by the caller
            State::Complete | State::Failed | State::Armed => {}
        }
    }

    /// Advance the sequence by elapsed time
    pub fn tick<IO: DeployIo>(&mut self, elapsed_ms: u32, io: &mut IO) {
        if !self.initialized {
            return;
        }
        self.update_timers_and_sensors(io, elapsed_ms);

        // Terminal states ignore the switch: re-burning is not harmless.
        // ARMED waits for the next boot.
        if matches!(self.state, State::Complete | State::Failed | State::Armed) {
            return;
        }

        // CDS 2.3.2.4: a stowed switch restarts the wait (and stops any burn)
        if io.stowed() {
            if self.state != State::Inhibited || self.wait_elapsed_s != 0 {
                Self::burn_off(io);
                self.restart_wait(io);
                let _ = self.save(io);
            }
            return;
        }

        self.step_sequence(io, elapsed_ms);
    }

    /// Re-arm for flight (GSE only); false if not persisted
    pub fn arm<IO: DeployIo>(&mut self, io: &mut IO) -> bool {
        Self::burn_off(io);
        self.fresh_start(io);
        self.state = State::Armed;
        self.save(io)
    }

    /// Enable or inhibit deployment (ground command); false if not persisted
    pub fn set_enabled<IO: DeployIo>(&mut self, enable: bool, io: &mut IO) -> bool {
        self.deployment_enabled = enable;
        io.log(Level::Info, format_args!("Deployment {} by command", if enable { "enabled" } else { "inhibited" }));
        self.save(io)
    }

    /// Retry an element (ground command): refused until the wait has been
    /// served (CDS 2.4.4) and during a burn
    pub fn force_retry<IO: DeployIo>(&mut self, e: Element, io: &mut IO) -> bool {
        match self.state {
            State::Inhibited | State::Waiting | State::Armed | State::AntennaActive | State::SolarActive => {
                return false;
            }
            State::AntennaPending
            | State::AntennaVerify
            | State::SolarPending
            | State::SolarVerify
            | State::Complete
            | State::Failed => {}
        }
        let el = self.element_mut(e);
        el.attempt_count = 0;
        el.deployed = false;
        self.enter_state(io, e.pending());
        let _ = self.save(io);
        io.log(Level::Info, format_args!("Forced retry for element {}", e as u32));
        true
    }

    /// Persist the sequence state
    pub fn save_state<F: Flash>(&mut self, flash: &mut F) -> bool {
        self.save(flash)
    }

    /// Current state
    pub const fn state(&self) -> State {
        self.state
    }

    /// Whether separation has been detected
    pub const fn is_separated(&self) -> bool {
        self.separation_detected
    }

    /// Seconds counted toward the wait
    pub const fn wait_elapsed_s(&self) -> u32 {
        self.wait_elapsed_s
    }

    /// An element's status
    pub fn element_status(&self, e: Element) -> ElementStatus {
        *self.element(e)
    }

    /// Both elements confirmed deployed
    pub fn all_complete(&self) -> bool {
        self.elements.iter().all(|e| e.deployed)
    }

    /// Telemetry
    pub fn telemetry(&self) -> Telemetry {
        let [antenna, solar] = &self.elements;
        Telemetry {
            state: self.state_byte(),
            antenna_deployed: u8::from(antenna.deployed),
            antenna_attempts: antenna.attempt_count,
            solar_deployed: u8::from(solar.deployed),
            solar_attempts: solar.attempt_count,
            time_since_sep_s: if self.separation_detected { self.since_separation_s } else { 0 },
            inhibit_active: u8::from(!self.deployment_enabled),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvm::{FlashError, MemFlash, SLOT_SIZE};

    const SECOND_MS: u32 = 1000;
    const MINUTE_MS: u32 = 60_000;

    struct Bench {
        flash: MemFlash<256>,
        arm: bool,
        burn: [bool; 2],
        sense: [bool; 2],
        stowed: bool,
        /// Attempts in the persisted record when each burn line went high
        persisted_at_burn: Option<u8>,
        reports: usize,
        last: Option<Report>,
    }

    impl Flash for Bench {
        fn read(&mut self, r: u32, o: u32, b: &mut [u8]) -> Result<(), FlashError> {
            self.flash.read(r, o, b)
        }
        fn write(&mut self, r: u32, o: u32, d: &[u8]) -> Result<(), FlashError> {
            self.flash.write(r, o, d)
        }
        fn erase(&mut self, r: u32) -> Result<(), FlashError> {
            self.flash.erase(r)
        }
    }

    impl DeployIo for Bench {
        fn configure_outputs(&mut self) {}
        fn configure_inputs(&mut self) {}
        fn set(&mut self, line: Line, on: bool) {
            match line {
                Line::Arm => self.arm = on,
                Line::Burn(e) => {
                    self.burn[e as usize] = on;
                    if on {
                        let mut rec = [0u8; 8];
                        let ok = nvm::load(&mut self.flash, 10, 0, RECORD_MAGIC, &mut rec);
                        self.persisted_at_burn = (ok == Ok(RecordStatus::Valid)).then_some(rec[1 + e as usize]);
                    }
                }
            }
        }
        fn sensed(&mut self, e: Element) -> bool {
            self.sense[e as usize]
        }
        fn stowed(&mut self) -> bool {
            self.stowed
        }
        fn log(&mut self, _level: Level, _message: fmt::Arguments<'_>) {}
        fn report(&mut self, report: Report) {
            self.reports += 1;
            self.last = Some(report);
        }
    }

    impl Bench {
        fn new() -> Self {
            Bench {
                flash: MemFlash::new(),
                arm: false,
                burn: [false; 2],
                sense: [false; 2],
                stowed: false,
                persisted_at_burn: None,
                reports: 0,
                last: None,
            }
        }
        fn burning(&self, e: Element) -> bool {
            self.arm && self.burn[e as usize]
        }
        fn no_burn(&self) -> bool {
            !self.arm && self.burn == [false; 2]
        }
    }

    fn boot(io: &mut Bench, reset: Reset) -> Deployment {
        let mut d = Deployment::new(10);
        d.init(reset, io);
        d
    }

    fn advance(d: &mut Deployment, io: &mut Bench, total_ms: u32, step_ms: u32) {
        let mut t = 0;
        while t < total_ms {
            d.tick(step_ms, io);
            t += step_ms;
        }
    }

    fn wait_out(d: &mut Deployment, io: &mut Bench) {
        d.tick(SECOND_MS, io);
        advance(d, io, WAIT_TIME_MS, MINUTE_MS);
    }

    #[test]
    fn full_sequence_one_element_at_a_time() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        assert_eq!(d.state(), State::Inhibited);
        assert!(io.no_burn());
        d.tick(SECOND_MS, &mut io);
        assert_eq!((d.state(), d.is_separated()), (State::Waiting, true));
        advance(&mut d, &mut io, WAIT_TIME_MS - SECOND_MS, MINUTE_MS);
        assert!(io.no_burn());
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::Antenna) && !io.burn[1]);
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS, SECOND_MS);
        assert!(io.no_burn());
        assert_eq!(d.state(), State::AntennaVerify);
        io.sense[0] = true;
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::SolarPanel) && !io.burn[0]);
        advance(&mut d, &mut io, SOLAR_BURN_TIME_MS, SECOND_MS);
        io.sense[1] = true;
        d.tick(SECOND_MS, &mut io);
        assert_eq!(d.state(), State::Complete);
        assert!(d.all_complete());
        let t = d.telemetry();
        assert_eq!((t.state, t.antenna_attempts, t.solar_attempts, t.inhibit_active), (8, 1, 1, 0));
        assert!(t.time_since_sep_s >= WAIT_TIME_S);
        assert!(d.element_status(Element::SolarPanel).switch_state);
    }

    /// The attempt is in flash before the burn line goes high, and a failed
    /// write withholds the burn
    #[test]
    fn attempt_persisted_before_the_burn() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        io.flash.writes_left = Some(0);
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(io.no_burn());
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 0);
        assert_eq!((io.reports, io.last), (1, Some(Report::NotPersisted)));
        assert_eq!(d.state(), State::AntennaPending);
        io.flash.writes_left = None;
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::Antenna));
        assert_eq!(io.persisted_at_burn, Some(1));
    }

    #[test]
    fn already_deployed_is_never_fired() {
        let mut io = Bench::new();
        io.sense[0] = true;
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(!io.burn[0]);
        assert!(io.burning(Element::SolarPanel));
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 0);
    }

    #[test]
    fn wait_resumes_only_when_power_was_kept_and_restarts_when_stowed() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        d.tick(SECOND_MS, &mut io);
        advance(&mut d, &mut io, 20 * MINUTE_MS, MINUTE_MS);
        let mut d = boot(&mut io, Reset::PowerKept);
        assert!(io.no_burn());
        assert_eq!(d.wait_elapsed_s(), 20 * 60);
        advance(&mut d, &mut io, 5 * MINUTE_MS, MINUTE_MS);
        let mut d = boot(&mut io, Reset::PowerLost);
        assert_eq!(d.wait_elapsed_s(), 0);

        d.tick(SECOND_MS, &mut io);
        advance(&mut d, &mut io, 25 * MINUTE_MS, MINUTE_MS);
        io.stowed = true;
        d.tick(SECOND_MS, &mut io);
        assert_eq!((d.state(), d.wait_elapsed_s(), d.is_separated()), (State::Inhibited, 0, false));
        io.stowed = false;
        d.tick(SECOND_MS, &mut io);
        advance(&mut d, &mut io, 29 * MINUTE_MS, MINUTE_MS);
        d.tick(SECOND_MS, &mut io);
        assert!(io.no_burn());
    }

    #[test]
    fn reset_during_burn_spends_the_attempt() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::Antenna));
        let d = boot(&mut io, Reset::PowerKept);
        assert!(io.no_burn());
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 1);
        assert_eq!(d.state(), State::AntennaVerify);
    }

    #[test]
    fn brownout_loop_is_bounded_by_persisted_attempts() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        let mut burns = 0;
        for _ in 0..(MAX_RETRIES + 2) {
            wait_out(&mut d, &mut io);
            for _ in 0..120 {
                d.tick(SECOND_MS, &mut io);
                if io.burning(Element::Antenna) {
                    burns += 1;
                    break;
                }
            }
            d = boot(&mut io, Reset::PowerLost);
            assert!(io.no_burn());
        }
        assert_eq!(burns, MAX_RETRIES);
        assert_eq!(d.element_status(Element::Antenna).attempt_count, MAX_RETRIES);
    }

    #[test]
    fn terminal_states_persist_and_ignore_the_switch_arm_resets() {
        let mut io = Bench::new();
        io.sense = [true; 2];
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 5 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::Complete);
        let mut d = boot(&mut io, Reset::PowerLost);
        assert_eq!(d.state(), State::Complete);
        io.stowed = true;
        advance(&mut d, &mut io, 40 * MINUTE_MS, MINUTE_MS);
        assert_eq!(d.state(), State::Complete);
        assert!(io.no_burn());

        io.stowed = false;
        io.sense = [false; 2];
        assert!(d.arm(&mut io));
        assert_eq!(d.state(), State::Armed);
        advance(&mut d, &mut io, 60 * MINUTE_MS, MINUTE_MS);
        assert_eq!(d.state(), State::Armed);
        assert!(!d.force_retry(Element::Antenna, &mut io));
        let mut d = boot(&mut io, Reset::PowerLost);
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 0);
        assert!(!d.all_complete());
        wait_out(&mut d, &mut io);
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::Antenna));
    }

    #[test]
    fn corrupt_record_starts_over() {
        let mut io = Bench::new();
        io.sense = [true; 2];
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 5 * SECOND_MS, SECOND_MS);
        io.flash.bytes[8] ^= 0xFF;
        io.flash.bytes[SLOT_SIZE + 8] ^= 0xFF;
        let d = boot(&mut io, Reset::PowerKept);
        assert_eq!((d.state(), io.last), (State::Inhibited, Some(Report::Unreadable)));
        // A valid record with an unknown state is unreadable too
        nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &[11, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        let before = io.reports;
        let d = boot(&mut io, Reset::PowerKept);
        assert_eq!((d.state(), io.reports), (State::Inhibited, before + 1));
    }

    #[test]
    fn disabled_holds_and_failure_allows_a_ground_retry() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        assert!(d.set_enabled(false, &mut io));
        assert_eq!(d.telemetry().inhibit_active, 1);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 5 * MINUTE_MS, MINUTE_MS);
        assert!(io.no_burn());
        let mut d = boot(&mut io, Reset::PowerKept);
        advance(&mut d, &mut io, 5 * MINUTE_MS, MINUTE_MS);
        assert!(io.no_burn());
        assert!(d.set_enabled(true, &mut io));
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::Antenna));

        for _ in 0..20 {
            advance(&mut d, &mut io, 2 * MINUTE_MS, SECOND_MS);
        }
        assert_eq!(d.state(), State::Failed);
        assert_eq!(io.last, Some(Report::Failed(Element::Antenna)));
        assert!(d.force_retry(Element::Antenna, &mut io));
        assert_eq!(d.state(), State::AntennaPending);
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::Antenna));
    }

    #[test]
    fn force_retry_cannot_bypass_the_wait_and_names() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        assert!(!d.force_retry(Element::Antenna, &mut io));
        d.tick(SECOND_MS, &mut io);
        assert!(!d.force_retry(Element::SolarPanel, &mut io));
        assert_eq!(d.state(), State::Waiting);
        assert!(io.no_burn());
        assert_eq!(d.telemetry().time_since_sep_s, 0);
        for s in State::ALL {
            assert_eq!(State::from_raw(s as u32), Some(s));
            assert!(!s.name().is_empty());
        }
        assert_eq!(State::from_raw(STATE_COUNT), None);
        assert_eq!(Element::from_raw(ELEMENT_COUNT), None);
        // Before init nothing moves
        let mut d = Deployment::new(10);
        d.tick(MINUTE_MS, &mut io);
        assert_eq!((d.state(), d.is_separated()), (State::Inhibited, false));
    }
}
