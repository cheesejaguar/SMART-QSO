//! Deployment sequence: the 30-minute wait, burn wires, persistence and
//! reset rules (`deployment.h`).
//!
//! Order and hardware (CDR, STR-15, STR-16, LV-07): **the solar wings
//! first, then the antenna**: the bought ISIS AntS's tape elements pass
//! 4–6 mm below the stowed wings. Each wing has two burn resistors (A and
//! B), each on its own line and current-limited driver: four lines
//! (`Line::Wing`). One resistor burns at a time, never two (the pack's
//! 1.0 A burn limit; CDR EPS board design): an attempt burns wing 1's
//! resistor for [`SOLAR_BURN_TIME_MS`], then wing 2's; attempts 1 and 3 use
//! the A resistors, attempt 2 the B resistors, so one failed driver or
//! resistor costs one attempt, not a wing. The antenna element is the
//! AntS: its "burn line" is the AntS controller's arm and automatic
//! deployment over I2C (`ants.rs`, mapped by the caller), and its sense is
//! every tape element's switch. An element whose attempts run out is
//! reported and the sequence goes on to the next one, so failed wings do
//! not leave the antenna stowed; the sequence ends COMPLETE only if both
//! deployed, otherwise FAILED.
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
//! **The AntS temperature gate** (decision C-18, THERMAL_ANALYSIS 2.4
//! §4.4): the AntS is qualified for operation only from −20 °C, and an
//! ungated release at separation + 31 min after an eclipse separation finds
//! it at −44 °C. Before each antenna attempt the sequence powers the AntS
//! controller (the master arm, which is its supply SW8 and the supply of the
//! AntS side of the I2C3 buffer), lets it start for [`ANTS_START_MS`], reads
//! its status (a deployed AntS is never fired again) and its own temperature
//! (`ants::CMD_TEMPERATURE`). Below [`ANTS_MIN_TEMP_C`] (−20 °C plus 3 °C of
//! sensor tolerance), or with no valid answer, the attempt is deferred: the
//! controller is powered down again and the sequence waits the
//! [`RETRY_DELAY_MS`] slot without spending an attempt. The slot's elapsed
//! time, the deferrals, the consecutive unknown readings and the ground's
//! force flag are persisted, so a reset inside a slot resumes it and spends
//! no deferral (RID P3F-A-04). Two ends bound the deferrals (RID P3F-G-02):
//!  - **The cap**: after [`ANTS_MAX_DEFERRALS`] deferrals (about 25 h, 16
//!    orbits: the wings-stowed, β = 0 contingency, in which the AntS never
//!    warms enough) it deploys whatever the temperature, but only in
//!    sunlight after [`ANTS_SUN_MS`] of continuous sun (the analysis's
//!    advice: late in a sunlit period, about −13 °C, not at a random phase
//!    that can be −36 °C).
//!  - **The schedule fallback**: after [`ANTS_UNKNOWN_LIMIT`] unknown
//!    readings in a row (a failed temperature path, not a cold AntS) the
//!    analysis's schedule replaces the gate: the AntS deploys after the
//!    first [`ANTS_SUN_MS`] of continuous sun from separation +
//!    [`ANTS_SCHEDULE_S`] (69 min, the gate's earliest nominal opening), so
//!    a temperature-path fault does not hold the transmitters for 25 h. A
//!    valid cold reading returns to the gate.
//!
//! Either waits at most [`ANTS_SUN_WAIT_MS`] (more than an orbit) for the
//! sun, so failed sun sensors cannot hold it either. DEPLOY_RETRY with its
//! force flag deploys the antenna at once. The deferrals and the last
//! temperature are in HK1 `AD`. The 30-minute wait and the wings-first order
//! are unchanged.
//!
//! **Read only while powered** (RID P3F-A-01): the AntS answers on I2C only
//! while SW8 is on. Every status read (the check before an attempt, the
//! verification after it) therefore raises the master arm, with no burn line,
//! for [`ANTS_START_MS`] first, and lowers it again after the read; the
//! end of a burn (which powers the AntS down with its STOP) is followed by
//! such a read. A read is not made while deployment is inhibited.
//!
//! **Separation, once established, is kept across resets** (RID SMA-B02):
//! a non-ARMED record with the separation flag, or a served RF silence,
//! shows the spacecraft has separated, so after a reset a stowed reading
//! (DEPLOY_SENSE open, BOARD_FMEA BF-OBC-18) restarts neither the wait nor
//! anything else; a power-losing reset still restarts the 30-minute wait
//! from zero, as before. The flag is cleared only by a stowed reading before
//! separation is established, by GSE ARM, or by a blank or corrupt record.
//!
//! **Silent while the AntS is pending** (decision C-19): keying the
//! downlink into the stowed elements trips its VSWR protection (SIL-37) and
//! stresses the PA, so the transmitters stay off while the AntS deployment
//! is pending, deferred by the gate or not yet attempted after the wings
//! ([`Deployment::downlink_held`], applied by the caller as the transmitter
//! inhibit `RF_INHIBIT_ANTS_PENDING`; the receiver stays on). The hold ends,
//! for good, when the AntS reads deployed (every element's switch, the
//! deployment rule of SRS-F089), or when its sequence ends: its attempts
//! spent, or the attempt made at the deferral cap finished. That end is
//! latched in the record, so a later DEPLOY_RETRY never silences the
//! stowed-antenna beacon again; only GSE ARM, or a blank or corrupt record
//! (the sequence starts over), clears it. The hold is never in force while
//! the ground has inhibited deployment, and never shortens the 45-minute RF
//! silence, which is separate. Its longest case is the cap: 30 min, the
//! wings' attempts, about 25 h of deferrals and one 45 s attempt.
//!
//! @requirement SRS-F167 Transmitters silent while the AntS deployment is pending (deferred or not yet attempted after the wings); the downlink starts when the AntS reads deployed or its sequence ends (attempts spent, or the attempt at the deferral cap made), latched, and never before the 45-minute RF silence
//! @requirement SRS-F165 AntS deployment only at an AntS temperature of at least −17 °C; colder or unknown defers to the next 50-minute slot without spending an attempt, at most 30 times, then deploys in sunlight after 20 min of continuous sun; three unknown readings in a row: the analysis schedule (the first 20 min of sun from separation + 69 min); slot, deferrals and force flag persisted; DEPLOY_RETRY with the force flag deploys at once
//! @requirement SRS-F169 The AntS is read (status, temperature) only after its supply has been on for 1 s, the supply raised for the read and lowered after it
//! @requirement SRS-F159 DEPLOY_SENSE high = released, no internal pull; released for 10 s latches separation until the next reset, after which a stowed reading is counted and restarts neither timer; separation persisted in the deployment record (or the served RF silence) keeps a stowed reading from restarting either timer after a reset
//! @requirement SYS-221 Deployables wait 30 minutes after separation; switch toggle resets the timer
//! @requirement SRS-F088 The software shall space deployment attempts 50 minutes apart and accept a telecommand to retry an element with fresh attempts

use core::ffi::CStr;
use core::fmt;

use crate::contact_monitor::carry_seconds;
use crate::nvm::{self, Flash, RecordStatus};
use crate::rf_silence::Reset;

/// Wait after separation, ms (CDS)
pub const WAIT_TIME_MS: u32 = 30 * 60 * 1000;
/// Wait after separation, s
pub const WAIT_TIME_S: u32 = WAIT_TIME_MS / 1000;
/// Antenna deployment window, ms: the AntS deploys its four elements in
/// turn, each for at most `ants::ELEMENT_LIMIT_S`, then the OBC cancels and
/// disarms it
pub const ANTENNA_BURN_TIME_MS: u32 = 45_000;
/// Burn time of one wing resistor, ms: 10 s at the 0.8 A the burn driver
/// allows, 32 J (decision C-17: at −55 °C 10 s gives a release-energy ratio
/// of 1.90 against the 1.5 required, 8 s only 1.52); inside the EPS's 12 s
/// one-shot
pub const SOLAR_BURN_TIME_MS: u32 = 10_000;
/// A wing attempt: wing 1's resistor, then wing 2's, one at a time, ms
pub const WINGS_BURN_TIME_MS: u32 = 2 * SOLAR_BURN_TIME_MS;
/// Attempts per element, across resets
pub const MAX_RETRIES: u8 = 3;
/// Wait before retrying an unconfirmed deployment, ms: 50 minutes, so the
/// three attempts fall at different points of the 94.6-minute orbit and
/// at least one is in sunlight (an eclipse is at most 36 minutes, and no
/// two attempts 50 or 44.6 minutes apart can share one), when the burn
/// wire and the hold-down are warm and the array helps the battery (LV-07)
pub const RETRY_DELAY_MS: u32 = 50 * 60 * 1000;
/// The AntS is deployed only at or above this temperature, °C (its −20 °C
/// qualified minimum plus 3 °C of sensor tolerance; decision C-18)
pub const ANTS_MIN_TEMP_C: f64 = -17.0;
/// Deferrals after which the AntS deploys whatever its temperature (30
/// slots of 50 min, about 25 h, 16 orbits)
pub const ANTS_MAX_DEFERRALS: u8 = 30;
/// The AntS controller is powered this long before it is read, ms (its
/// start-up; to be confirmed with the ISIS ICD, V-1)
pub const ANTS_START_MS: u32 = 1_000;
/// Continuous sunlight before a release at the deferral cap or by the
/// schedule fallback, ms (THERMAL_ANALYSIS 2.4 §4.4: late in a sunlit
/// period, about −13 °C in the wings-stowed case)
pub const ANTS_SUN_MS: u32 = 20 * 60 * 1000;
/// The longest a release waits for [`ANTS_SUN_MS`] of sun, ms (more than an
/// orbit of 94.6 min: failed sun sensors cannot hold it)
pub const ANTS_SUN_WAIT_MS: u32 = 2 * RETRY_DELAY_MS;
/// Unknown AntS temperatures in a row after which the analysis schedule
/// replaces the gate
pub const ANTS_UNKNOWN_LIMIT: u8 = 3;
/// The schedule fallback releases no earlier than this after separation, s
/// (the gate's earliest nominal opening, THERMAL_ANALYSIS 2.4 §4.4)
pub const ANTS_SCHEDULE_S: u32 = 69 * 60;
/// Wait-count persistence interval, s
pub const PERSIST_INTERVAL_S: u32 = 60;

/// Record ("DEPL"): state | attempts (2) | flags | wait seconds (4) |
/// AntS deferrals | the deferral slot's seconds (2) | unknown AntS
/// temperatures in a row
pub const RECORD_MAGIC: u32 = 0x4445_504C;
/// The record's length, bytes
pub const RECORD_LEN: usize = 12;
const FLAG_ANTENNA_DEPLOYED: u8 = 0x01;
const FLAG_SOLAR_DEPLOYED: u8 = 0x02;
const FLAG_ENABLED: u8 = 0x04;
const FLAG_SEPARATED: u8 = 0x08;
/// The AntS sequence has ended (deployed, attempts spent, or the attempt at
/// the deferral cap made): the transmitter hold of decision C-19 is over
const FLAG_ANTS_DONE: u8 = 0x10;
/// An AntS attempt is deferred: its slot runs (RID P3F-A-04)
const FLAG_ANTS_DEFERRED: u8 = 0x20;
/// The ground forced the antenna (DEPLOY_RETRY's flag, RID P3F-A-04)
const FLAG_ANTS_FORCED: u8 = 0x40;

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
            Element::SolarPanel => WINGS_BURN_TIME_MS,
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
    /// The antenna's deployment: the AntS controller's arm and automatic
    /// deployment (I2C)
    Ants,
    /// A wing's burn resistor: wing (0 is W1, 1 is W2) and resistor
    /// (0 A, 1 B): `GPIO_PIN_BURN_W1A`, `_W1B`, `_W2A`, `_W2B`
    Wing(u8, u8),
}

/// The four wing lines, in burn order within an attempt per resistor
const WING_LINES: [Line; 4] = [Line::Wing(0, 0), Line::Wing(0, 1), Line::Wing(1, 0), Line::Wing(1, 1)];

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
    /// An AntS attempt deferred: its temperature below
    /// [`ANTS_MIN_TEMP_C`] or unknown
    AntsDeferred,
    /// [`ANTS_UNKNOWN_LIMIT`] unknown AntS temperatures in a row: the
    /// analysis schedule replaces the gate
    AntsSchedule,
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
    /// The AntS's own temperature, °C, with its controller powered
    /// (`None`: no valid answer)
    fn ants_temperature_c(&mut self) -> Option<f64>;
    /// Whether the spacecraft is in sunlight (the sun sensors or the array)
    fn sunlit(&mut self) -> bool;
    /// Whether the 45-minute RF silence has been served (its record): the
    /// spacecraft has separated
    fn silence_served(&mut self) -> bool;
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
    /// Wing 2's resistor is burning (the second half of a wing attempt)
    second_wing: bool,
    /// AntS attempts deferred for its temperature (persisted)
    ants_deferrals: u8,
    /// The AntS's last temperature reading, °C (`None`: none valid)
    ants_temp_c: Option<f64>,
    /// An attempt is deferred: the slot's wait runs (persisted)
    ants_deferred: bool,
    /// Time in the deferral slot, ms (persisted every minute, in seconds)
    ants_slot_ms: u32,
    /// [`Deployment::ants_slot_ms`] when last persisted
    ants_slot_saved_ms: u32,
    /// Unknown AntS temperatures in a row (persisted)
    ants_unknown: u8,
    /// The ground forced the antenna's deployment (DEPLOY_RETRY's flag;
    /// persisted)
    ants_forced: bool,
    /// The AntS supply raised by the sequence: ms since (`None`: down)
    ants_on_ms: Option<u32>,
    /// The verification read after an AntS attempt is done
    ants_checked: bool,
    /// Continuous sunlight, ms
    sun_ms: u32,
    /// Time waiting for [`ANTS_SUN_MS`] of sun for a release, ms
    ants_sun_wait_ms: u32,
    /// Separation established by the record or the served RF silence: a
    /// stowed reading restarts nothing (RID SMA-B02)
    separation_known: bool,
    /// The AntS sequence has ended once (persisted, latched): the
    /// transmitter hold of decision C-19 is over
    ants_done: bool,
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
            second_wing: false,
            ants_deferrals: 0,
            ants_temp_c: None,
            ants_deferred: false,
            ants_slot_ms: 0,
            ants_slot_saved_ms: 0,
            ants_unknown: 0,
            ants_forced: false,
            ants_on_ms: None,
            ants_checked: false,
            sun_ms: 0,
            ants_sun_wait_ms: 0,
            separation_known: false,
            ants_done: false,
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

    /// Master arm off first, then every element line (the AntS supply with it)
    fn burn_off<IO: DeployIo>(&mut self, io: &mut IO) {
        self.ants_on_ms = None;
        io.set(Line::Arm, false);
        io.set(Line::Ants, false);
        for line in WING_LINES {
            io.set(line, false);
        }
    }

    /// The resistor an attempt uses: A (0) on attempts 1 and 3, B (1) on 2
    const fn resistor(attempt: u8) -> u8 {
        if attempt.is_multiple_of(2) { 1 } else { 0 }
    }

    /// The line an attempt starts with: the antenna's, or wing 1's resistor
    const fn burn_line(e: Element, attempt: u8) -> Line {
        match e {
            Element::SolarPanel => Line::Wing(0, Self::resistor(attempt)),
            Element::Antenna => Line::Ants,
        }
    }

    /// Element line first, then the master arm: both are needed to burn
    fn burn_on<IO: DeployIo>(&mut self, io: &mut IO, e: Element, attempt: u8) {
        self.second_wing = false;
        io.set(Self::burn_line(e, attempt), true);
        io.set(Line::Arm, true);
    }

    /// The state after element `e` is done (deployed or out of attempts):
    /// the antenna after the wings; at the end COMPLETE if both deployed
    fn after(&self, e: Element) -> State {
        match e {
            Element::SolarPanel => State::AntennaPending,
            Element::Antenna if self.all_complete() => State::Complete,
            Element::Antenna => State::Failed,
        }
    }

    fn state_byte(&self) -> u8 {
        u8::try_from(self.state as u32).unwrap_or(u8::MAX)
    }

    fn record(&self) -> [u8; RECORD_LEN] {
        let [antenna, solar] = &self.elements;
        let flags = (if antenna.deployed { FLAG_ANTENNA_DEPLOYED } else { 0 })
            | (if solar.deployed { FLAG_SOLAR_DEPLOYED } else { 0 })
            | (if self.deployment_enabled { FLAG_ENABLED } else { 0 })
            | (if self.separation_detected || self.separation_known { FLAG_SEPARATED } else { 0 })
            | (if self.ants_done { FLAG_ANTS_DONE } else { 0 })
            | (if self.ants_deferred { FLAG_ANTS_DEFERRED } else { 0 })
            | (if self.ants_forced { FLAG_ANTS_FORCED } else { 0 });
        let [w0, w1, w2, w3] = self.wait_elapsed_s.to_le_bytes();
        let [s0, s1] = u16::try_from(self.ants_slot_ms / 1000).unwrap_or(u16::MAX).to_le_bytes();
        [
            self.state_byte(),
            antenna.attempt_count,
            solar.attempt_count,
            flags,
            w0,
            w1,
            w2,
            w3,
            self.ants_deferrals,
            s0,
            s1,
            self.ants_unknown,
        ]
    }

    /// The record of a finished sequence: COMPLETE, each element deployed
    /// at its first attempt, enabled, separated, the wait served and the
    /// AntS done (the host simulation's start-up hook writes one with the
    /// served RF silence; `sqso_ffi::executive::run_host`)
    pub const fn complete_record() -> [u8; RECORD_LEN] {
        let flags = FLAG_ANTENNA_DEPLOYED | FLAG_SOLAR_DEPLOYED | FLAG_ENABLED | FLAG_SEPARATED | FLAG_ANTS_DONE;
        let [w0, w1, w2, w3] = WAIT_TIME_S.to_le_bytes();
        #[allow(clippy::cast_possible_truncation)] // the state values are 0 to 10
        let state = State::Complete as u32 as u8;
        [state, 1, 1, flags, w0, w1, w2, w3, 0, 0, 0, 0]
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
        self.ants_checked = false;
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
        self.ants_deferrals = 0;
        self.ants_deferred = false;
        self.ants_slot_ms = 0;
        self.ants_slot_saved_ms = 0;
        self.ants_unknown = 0;
        self.ants_forced = false;
        self.ants_on_ms = None;
        self.ants_checked = false;
        self.sun_ms = 0;
        self.ants_sun_wait_ms = 0;
        self.separation_known = false;
        self.ants_done = false;
        self.restart_wait(io);
    }

    fn mark_deployed<IO: DeployIo>(&mut self, io: &mut IO, e: Element) {
        let since = self.since_separation_s;
        let el = self.element_mut(e);
        el.deployed = true;
        el.deploy_time_s = since;
        if e == Element::Antenna {
            self.ants_done = true;
            self.ants_off(io);
        }
        let next = self.after(e);
        self.enter_state(io, next);
        let _ = self.save(io);
        io.log(Level::Info, format_args!("Element {} deployed", e as u32));
    }

    /// Raise the AntS supply (the master arm, no burn line) if the
    /// sequence has not; whether it has been up [`ANTS_START_MS`], so the
    /// AntS answers (RID P3F-A-01)
    fn ants_ready<IO: DeployIo>(&mut self, io: &mut IO) -> bool {
        match self.ants_on_ms {
            Some(ms) => ms >= ANTS_START_MS,
            None => {
                io.set(Line::Arm, true);
                self.ants_on_ms = Some(0);
                false
            }
        }
    }

    /// Lower the AntS supply the sequence raised
    fn ants_off<IO: DeployIo>(&mut self, io: &mut IO) {
        if self.ants_on_ms.take().is_some() {
            io.set(Line::Arm, false);
        }
    }

    /// The AntS's status read with its controller powered: whether every
    /// element reads deployed (`None`: not powered long enough yet)
    fn ants_sensed<IO: DeployIo>(&mut self, io: &mut IO) -> Option<bool> {
        if !self.ants_ready(io) {
            return None;
        }
        let sensed = io.sensed(Element::Antenna);
        self.element_mut(Element::Antenna).switch_state = sensed;
        Some(sensed)
    }

    fn process_pending<IO: DeployIo>(&mut self, io: &mut IO, e: Element, active: State) {
        if e == Element::Antenna {
            self.process_antenna_pending(io, active);
            return;
        }
        // Idempotent: never fire an element that is already out
        if self.element(e).deployed || io.sensed(e) {
            self.mark_deployed(io, e);
            return;
        }
        if !self.deployment_enabled {
            return;
        }
        if self.element(e).attempt_count >= MAX_RETRIES {
            // Reported, and on to the next element (failed wings must not
            // leave the antenna stowed)
            io.report(Report::Failed(e));
            let next = self.after(e);
            self.enter_state(io, next);
            let _ = self.save(io);
            return;
        }
        self.attempt(io, e, active);
    }

    /// Persist the attempt (and the burning state) before driving the line,
    /// so a reset can never repeat a burn; if it cannot be persisted, do
    /// not burn
    fn attempt<IO: DeployIo>(&mut self, io: &mut IO, e: Element, active: State) {
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

        let attempt = self.element(e).attempt_count;
        self.burn_on(io, e, attempt);
        io.log(Level::Info, format_args!("Element {} burn attempt {}", e as u32, attempt));
    }

    /// Whether the gate waits for sunlight now: at the deferral cap, or on
    /// the analysis schedule after repeated unknown temperatures (neither
    /// when the ground forced the antenna)
    fn ants_sun_release(&self) -> bool {
        !self.ants_forced && (self.ants_deferrals >= ANTS_MAX_DEFERRALS || self.ants_unknown >= ANTS_UNKNOWN_LIMIT)
    }

    /// A sun-bound release may go now: [`ANTS_SUN_MS`] of continuous sun
    /// (after separation + [`ANTS_SCHEDULE_S`] on the schedule), or the wait
    /// for it spent
    fn ants_sun_ok(&self) -> bool {
        let schedule = self.ants_deferrals < ANTS_MAX_DEFERRALS;
        let early = schedule && self.since_separation_s < ANTS_SCHEDULE_S;
        !early && (self.sun_ms >= ANTS_SUN_MS || self.ants_sun_wait_ms >= ANTS_SUN_WAIT_MS)
    }

    /// The antenna's pending state: the deferral slot and the wait for
    /// sunlight unpowered; then the AntS powered, its status read (a
    /// deployed AntS is never fired), the temperature gate, the attempt
    fn process_antenna_pending<IO: DeployIo>(&mut self, io: &mut IO, active: State) {
        let e = Element::Antenna;
        if self.element(e).deployed {
            self.mark_deployed(io, e);
            return;
        }
        if !self.deployment_enabled {
            self.ants_off(io); // nothing is powered or read while inhibited
            return;
        }
        if self.ants_deferred {
            if self.ants_slot_ms < RETRY_DELAY_MS {
                if self.ants_slot_ms.saturating_sub(self.ants_slot_saved_ms) >= PERSIST_INTERVAL_S * 1000 {
                    self.ants_slot_saved_ms = self.ants_slot_ms;
                    let _ = self.save(io);
                }
                return;
            }
            self.ants_deferred = false;
            self.ants_slot_ms = 0;
            self.ants_slot_saved_ms = 0;
        }
        let spent = self.element(e).attempt_count >= MAX_RETRIES;
        if !spent && self.ants_sun_release() && !self.ants_sun_ok() {
            self.ants_off(io); // unpowered until the sun (or the wait's bound)
            return;
        }
        match self.ants_sensed(io) {
            None => return, // starting
            Some(true) => {
                self.mark_deployed(io, e);
                return;
            }
            Some(false) => {}
        }
        if spent {
            io.report(Report::Failed(e));
            self.ants_done = true; // its attempts are spent
            self.ants_off(io);
            let next = self.after(e);
            self.enter_state(io, next);
            let _ = self.save(io);
            return;
        }
        if !self.ants_gate(io) {
            return;
        }
        self.ants_sun_wait_ms = 0;
        self.attempt(io, e, active);
    }

    /// The AntS temperature gate, with the controller powered and started:
    /// whether the attempt goes ahead now (forced, warm enough, the cap or
    /// the schedule fallback in sunlight); otherwise it is deferred to the
    /// next slot without spending it
    fn ants_gate<IO: DeployIo>(&mut self, io: &mut IO) -> bool {
        if self.ants_forced {
            return true;
        }
        let t = io.ants_temperature_c().filter(|t| t.is_finite());
        self.ants_temp_c = t;
        if t.is_some() {
            self.ants_unknown = 0;
        } else {
            self.ants_unknown = self.ants_unknown.saturating_add(1);
            if self.ants_unknown == ANTS_UNKNOWN_LIMIT {
                io.report(Report::AntsSchedule);
            }
        }
        let warm = t.is_some_and(|t| t >= ANTS_MIN_TEMP_C);
        // The cap and the schedule fallback were waited for in sunlight
        // before the controller was powered (process_antenna_pending); the
        // schedule's third unknown reading may come before that
        let cap = self.ants_deferrals >= ANTS_MAX_DEFERRALS;
        let schedule = t.is_none() && self.ants_unknown >= ANTS_UNKNOWN_LIMIT && self.ants_sun_ok();
        if warm || cap || schedule {
            return true;
        }
        if self.ants_unknown >= ANTS_UNKNOWN_LIMIT {
            // This reading made it the schedule's: wait for the sun unpowered
            self.ants_off(io);
            let _ = self.save(io);
            return false;
        }
        self.ants_deferrals = self.ants_deferrals.saturating_add(1);
        self.burn_off(io);
        self.ants_deferred = true;
        self.ants_slot_ms = 0;
        self.ants_slot_saved_ms = 0;
        let _ = self.save(io);
        io.report(Report::AntsDeferred);
        false
    }

    fn process_active<IO: DeployIo>(&mut self, io: &mut IO, e: Element, verify: State) {
        // The wings: wing 1's resistor off before wing 2's goes on
        let half = self.state_elapsed_ms >= SOLAR_BURN_TIME_MS;
        if e == Element::SolarPanel && half && !self.second_wing {
            let r = Self::resistor(self.element(e).attempt_count);
            io.set(Line::Wing(0, r), false);
            io.set(Line::Wing(1, r), true);
            self.second_wing = true;
        }
        if self.state_elapsed_ms >= e.burn_time_ms() {
            self.burn_off(io);
            if e == Element::Antenna && self.ants_deferrals >= ANTS_MAX_DEFERRALS {
                self.ants_done = true; // the attempt at the deferral cap is made
            }
            self.enter_state(io, verify);
            let _ = self.save(io);
        }
    }

    fn process_verify<IO: DeployIo>(&mut self, io: &mut IO, e: Element) {
        let sensed = match e {
            Element::SolarPanel => Some(io.sensed(e)),
            // The AntS, powered down by the end of the burn, is powered
            // again and read once (RID P3F-A-01)
            Element::Antenna if self.ants_checked || !self.deployment_enabled => Some(false),
            Element::Antenna => {
                let s = self.ants_sensed(io);
                if s.is_some() {
                    self.ants_checked = true;
                    self.ants_off(io);
                }
                s
            }
        };
        if sensed == Some(true) {
            self.mark_deployed(io, e);
        } else if self.state_elapsed_ms >= RETRY_DELAY_MS {
            self.ants_off(io);
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
            self.enter_state(io, State::SolarPending);
            let _ = self.save(io);
            io.log(Level::Info, format_args!("Wait complete, starting wing deploy"));
        } else if self.unpersisted_s >= PERSIST_INTERVAL_S {
            let _ = self.save(io);
        } else {
            // Persist at the next interval
        }
    }

    /// Resume from a valid record. ARMED starts fresh; COMPLETE and FAILED
    /// are kept; other states resume unless power was lost (the wait
    /// restarts); a burn interrupted by a reset goes to verification.
    fn restore(&mut self, rec: &[u8; RECORD_LEN], state: State, reset: Reset) {
        let [_, antenna_attempts, solar_attempts, flags, w0, w1, w2, w3, deferrals, s0, s1, unknown] = *rec;
        if state == State::Armed {
            return; // the sequence starts fresh at this boot
        }
        self.ants_deferrals = deferrals;
        self.ants_unknown = unknown;
        self.ants_deferred = flags & FLAG_ANTS_DEFERRED != 0;
        self.ants_forced = flags & FLAG_ANTS_FORCED != 0;
        // The slot resumes where it was persisted (at most a minute
        // behind: the slot only lengthens), spending no deferral
        self.ants_slot_ms = u32::from(u16::from_le_bytes([s0, s1])).saturating_mul(1000);
        self.ants_slot_saved_ms = self.ants_slot_ms;
        let [antenna, solar] = &mut self.elements;
        for (el, attempts) in [(antenna, antenna_attempts), (solar, solar_attempts)] {
            el.attempt_count = attempts;
            el.deploy_attempted = attempts > 0;
        }
        let [antenna, solar] = &mut self.elements;
        antenna.deployed = flags & FLAG_ANTENNA_DEPLOYED != 0;
        solar.deployed = flags & FLAG_SOLAR_DEPLOYED != 0;
        self.deployment_enabled = flags & FLAG_ENABLED != 0;
        self.ants_done = flags & FLAG_ANTS_DONE != 0;
        // Separation once established (RID SMA-B02): kept whatever the reset
        self.separation_known = flags & FLAG_SEPARATED != 0;

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
        self.burn_off(io);
        io.configure_inputs();

        self.fresh_start(io);
        self.unpersisted_s = 0;
        self.sep_remainder_ms = 0;
        self.persist_fault_logged = false;

        let mut rec = [0u8; RECORD_LEN];
        let loaded = nvm::load(io, self.region, 0, RECORD_MAGIC, &mut rec);
        let [state_raw, ..] = rec;
        let mut armed = false;
        match (loaded, State::from_raw(u32::from(state_raw))) {
            (Ok(RecordStatus::Valid), Some(state)) => {
                armed = state == State::Armed;
                self.restore(&rec, state, reset);
            }
            (Ok(RecordStatus::Blank), _) => {} // first boot
            _ => io.report(Report::Unreadable),
        }
        // The served RF silence shows separation as well (a lost record)
        if !armed && io.silence_served() {
            self.separation_known = true;
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
        // The wings' sense line every step; the AntS only when the sequence
        // has it powered (`ants_sensed`)
        let sensed = io.sensed(Element::SolarPanel);
        self.element_mut(Element::SolarPanel).switch_state = sensed;
        if let Some(ms) = self.ants_on_ms.as_mut() {
            *ms = ms.saturating_add(elapsed_ms);
        }
        if self.ants_deferred {
            self.ants_slot_ms = self.ants_slot_ms.saturating_add(elapsed_ms);
        }
        self.sun_ms = if io.sunlit() { self.sun_ms.saturating_add(elapsed_ms) } else { 0 };
        let waiting = self.state == State::AntennaPending && !self.ants_deferred && self.ants_sun_release();
        self.ants_sun_wait_ms = if waiting { self.ants_sun_wait_ms.saturating_add(elapsed_ms) } else { 0 };
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
            // COVERAGE-EXEMPT: tick, the only caller, returns before this call in these states
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

        // CDS 2.3.2.4: a stowed switch restarts the wait (and stops any
        // burn), until separation has been established (RID SMA-B02)
        if !self.separation_known && io.stowed() {
            if self.state != State::Inhibited || self.wait_elapsed_s != 0 {
                self.burn_off(io);
                self.restart_wait(io);
                let _ = self.save(io);
            }
            return;
        }

        self.step_sequence(io, elapsed_ms);
    }

    /// Re-arm for flight (GSE only); false if not persisted
    pub fn arm<IO: DeployIo>(&mut self, io: &mut IO) -> bool {
        self.burn_off(io);
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
        self.retry(e, false, io)
    }

    /// [`Deployment::force_retry`]; with `force` (the antenna only) the
    /// AntS temperature gate is passed by: the antenna deploys at once
    pub fn retry<IO: DeployIo>(&mut self, e: Element, force: bool, io: &mut IO) -> bool {
        if force && e != Element::Antenna {
            return false;
        }
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
        if e == Element::Antenna {
            self.ants_forced = force;
            self.ants_deferred = false;
            self.ants_slot_ms = 0;
            self.ants_slot_saved_ms = 0;
            self.ants_sun_wait_ms = 0;
        }
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

    /// Whether separation is established by the record or the served RF
    /// silence (RID SMA-B02: a stowed reading then restarts no timer)
    pub const fn separation_known(&self) -> bool {
        self.separation_known
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

    /// The wings' attempts are spent and the wings are not deployed: the
    /// wing phase is over (EPS-10: the mode manager then holds IDLE). A
    /// DEPLOY_RETRY of the wings gives fresh attempts and clears it
    pub fn wings_exhausted(&self) -> bool {
        let wings = self.element(Element::SolarPanel);
        let wing_phase = matches!(self.state, State::SolarPending | State::SolarActive | State::SolarVerify);
        !wings.deployed && wings.attempt_count >= MAX_RETRIES && !wing_phase
    }

    /// Whether the transmitters are held off for the AntS (decision C-19):
    /// after init, deployment enabled, and the AntS sequence not yet ended
    /// (not deployed, attempts not spent, the attempt at the deferral cap
    /// not made). COMPLETE and FAILED end it whatever the record says
    pub fn downlink_held(&self) -> bool {
        self.initialized
            && self.deployment_enabled
            && !self.ants_done
            && !self.element(Element::Antenna).deployed
            && !matches!(self.state, State::Complete | State::Failed)
    }

    /// The AntS temperature gate: deferrals, and the last temperature, °C
    /// (`None`: none valid)
    pub const fn ants_gate_status(&self) -> (u8, Option<f64>) {
        (self.ants_deferrals, self.ants_temp_c)
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
        /// The four wing lines: [wing][resistor]
        wing: [[bool; 2]; 2],
        /// Most wing lines on at once
        wing_max: usize,
        sense: [bool; 2],
        stowed: bool,
        /// Attempts in the persisted record when each burn line went high
        persisted_at_burn: Option<u8>,
        reports: usize,
        last: Option<Report>,
        /// Every flight-log line, newline-separated
        text: Text,
        /// The AntS's temperature (`None`: no answer), and its reads
        ants_c: Option<f64>,
        ants_reads: u32,
        /// AntS status reads made with its controller unpowered (it does
        /// not answer then: RID P3F-A-01)
        unpowered_reads: u32,
        /// In sunlight; the RF silence served
        sunlit: bool,
        silence: bool,
    }

    struct Text {
        buf: [u8; 4096],
        len: usize,
    }

    impl fmt::Write for Text {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for b in s.bytes() {
                if let Some(slot) = self.buf.get_mut(self.len) {
                    *slot = b;
                    self.len += 1;
                }
            }
            Ok(())
        }
    }

    impl Text {
        fn contains(&self, needle: &[u8]) -> bool {
            self.buf[..self.len].windows(needle.len()).any(|w| w == needle)
        }
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
                Line::Wing(w, r) => {
                    self.wing[usize::from(w)][usize::from(r)] = on;
                    let n = self.wing.iter().flatten().filter(|b| **b).count();
                    self.wing_max = self.wing_max.max(n);
                    if on {
                        let mut rec = [0u8; RECORD_LEN];
                        let ok = nvm::load(&mut self.flash, 10, 0, RECORD_MAGIC, &mut rec);
                        self.persisted_at_burn = (ok == Ok(RecordStatus::Valid)).then_some(rec[2]);
                    }
                }
                Line::Ants => {
                    let e = Element::Antenna;
                    self.burn[e as usize] = on;
                    if on {
                        let mut rec = [0u8; RECORD_LEN];
                        let ok = nvm::load(&mut self.flash, 10, 0, RECORD_MAGIC, &mut rec);
                        self.persisted_at_burn = (ok == Ok(RecordStatus::Valid)).then_some(rec[1 + e as usize]);
                    }
                }
            }
        }
        fn sensed(&mut self, e: Element) -> bool {
            if e == Element::Antenna && !self.arm {
                // The AntS answers only while SW8 (the master arm) is on
                self.unpowered_reads += 1;
                return false;
            }
            self.sense[e as usize]
        }
        fn sunlit(&mut self) -> bool {
            self.sunlit
        }
        fn silence_served(&mut self) -> bool {
            self.silence
        }
        fn stowed(&mut self) -> bool {
            self.stowed
        }
        fn ants_temperature_c(&mut self) -> Option<f64> {
            assert!(self.arm, "the AntS is read with its controller powered");
            self.ants_reads += 1;
            self.ants_c
        }
        fn log(&mut self, _level: Level, message: fmt::Arguments<'_>) {
            let _ = fmt::Write::write_fmt(&mut self.text, message);
            let _ = fmt::Write::write_str(&mut self.text, "\n");
        }
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
                wing: [[false; 2]; 2],
                wing_max: 0,
                sense: [false; 2],
                stowed: false,
                persisted_at_burn: None,
                reports: 0,
                last: None,
                text: Text { buf: [0; 4096], len: 0 },
                ants_c: Some(5.0),
                ants_reads: 0,
                unpowered_reads: 0,
                sunlit: true,
                silence: false,
            }
        }
        /// Burning on any of the element's drivers
        fn burning(&self, e: Element) -> bool {
            let wings = self.wing.iter().flatten().any(|b| *b);
            self.arm && (self.burn[e as usize] || (e == Element::SolarPanel && wings))
        }
        fn no_burn(&self) -> bool {
            !self.arm && self.burn == [false; 2] && self.wing == [[false; 2]; 2]
        }
        /// Exactly this wing line burns
        fn only(&self, w: usize, r: usize) -> bool {
            let mut want = [[false; 2]; 2];
            want[w][r] = true;
            self.arm && self.wing == want
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
        // The wings first (STR-16): wing 1's resistor A, then wing 2's
        assert!(io.only(0, 0) && !io.burn[0]);
        // (the burn began on the first of the two ticks)
        advance(&mut d, &mut io, SOLAR_BURN_TIME_MS - 2 * SECOND_MS, SECOND_MS);
        assert!(io.only(0, 0));
        d.tick(SECOND_MS, &mut io);
        assert!(io.only(1, 0), "wing 2 after wing 1, never both");
        advance(&mut d, &mut io, SOLAR_BURN_TIME_MS, SECOND_MS);
        assert!(io.no_burn());
        assert_eq!(io.wing_max, 1);
        assert_eq!(d.state(), State::SolarVerify);
        io.sense[1] = true;
        d.tick(SECOND_MS, &mut io);
        assert_eq!(d.state(), State::AntennaPending);
        // The AntS powered for its start-up, read, then told to deploy
        d.tick(SECOND_MS, &mut io);
        assert!(io.arm && !io.burning(Element::Antenna), "the AntS starting");
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::Antenna) && io.wing == [[false; 2]; 2]);
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS, SECOND_MS);
        assert!(io.no_burn());
        assert_eq!(d.state(), State::AntennaVerify);
        io.sense[0] = true;
        // Powered again for the verification read (RID P3F-A-01)
        d.tick(SECOND_MS, &mut io);
        assert!(io.arm && !io.burning(Element::Antenna));
        d.tick(SECOND_MS, &mut io);
        assert_eq!(d.state(), State::Complete);
        assert!(io.no_burn(), "powered down after the read");
        assert_eq!(io.unpowered_reads, 0);
        assert!(d.all_complete());
        assert!(!d.wings_exhausted(), "deployed: not EPS-10");
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
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, 0);
        assert_eq!((io.reports, io.last), (1, Some(Report::NotPersisted)));
        assert_eq!(d.state(), State::SolarPending);
        io.flash.writes_left = None;
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::SolarPanel));
        assert_eq!(io.persisted_at_burn, Some(1));
    }

    #[test]
    fn already_deployed_is_never_fired() {
        let mut io = Bench::new();
        io.sense[1] = true;
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 3 * SECOND_MS, SECOND_MS);
        assert!(io.wing == [[false; 2]; 2]);
        assert!(io.burning(Element::Antenna));
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, 0);
        // An AntS already out (read with its controller powered) is not fired
        let mut io = Bench::new();
        io.sense = [true; 2];
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 3 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::Complete);
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 0);
        assert!(io.no_burn() && io.unpowered_reads == 0);
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
        let d = boot(&mut io, Reset::PowerLost);
        assert_eq!(d.wait_elapsed_s(), 0);
        assert!(d.separation_known(), "separation was established: kept across the power loss");

        // Before separation is established (a fresh record) a stowed
        // switch restarts the wait
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
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
        assert!(io.burning(Element::SolarPanel));
        let d = boot(&mut io, Reset::PowerKept);
        assert!(io.no_burn());
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, 1);
        assert_eq!(d.state(), State::SolarVerify);
        // And the antenna's burn too
        let mut io = Bench::new();
        io.sense[1] = true;
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna));
        let d = boot(&mut io, Reset::PowerKept);
        assert!(io.no_burn());
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
                if io.burning(Element::SolarPanel) {
                    burns += 1;
                    break;
                }
            }
            d = boot(&mut io, Reset::PowerLost);
            assert!(io.no_burn());
        }
        assert_eq!(burns, MAX_RETRIES);
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, MAX_RETRIES);
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
        assert!(io.burning(Element::SolarPanel));
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
        nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &[11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        let before = io.reports;
        let d = boot(&mut io, Reset::PowerKept);
        assert_eq!((d.state(), io.reports), (State::Inhibited, before + 1));
    }

    /// To the antenna: the wings out and the sequence at ANTENNA_PENDING
    fn at_the_antenna(io: &mut Bench) -> Deployment {
        let mut d = boot(io, Reset::PowerLost);
        io.sense[Element::SolarPanel as usize] = true;
        wait_out(&mut d, io);
        d.tick(SECOND_MS, io);
        assert_eq!(d.state(), State::AntennaPending);
        d
    }

    /// The AntS temperature gate (decision C-18): cold or unknown defers
    /// the attempt by one 50-minute slot without spending it; once warm it
    /// deploys; the controller is powered only to be read, and read only
    /// after its start-up
    #[test]
    fn a_cold_ants_is_deferred_until_warm() {
        let mut io = Bench::new();
        io.ants_c = Some(-44.0);
        let mut d = at_the_antenna(&mut io);
        d.tick(SECOND_MS / 2, &mut io);
        assert!(io.arm && !io.burning(Element::Antenna), "the controller powered to start, nothing burns");
        d.tick(SECOND_MS / 2, &mut io);
        assert_eq!(io.ants_reads, 0, "not read before it has been powered 1 s");
        d.tick(SECOND_MS / 2, &mut io);
        assert_eq!(io.ants_reads, 1);
        assert!(io.no_burn(), "deferred: the controller powered down");
        assert_eq!(io.last, Some(Report::AntsDeferred));
        assert_eq!(d.ants_gate_status(), (1, Some(-44.0)));
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 0, "no attempt spent");
        assert_eq!(d.state(), State::AntennaPending);
        // Nothing until the next slot
        advance(&mut d, &mut io, RETRY_DELAY_MS - MINUTE_MS, MINUTE_MS);
        assert_eq!(io.ants_reads, 1);
        assert!(io.no_burn());
        // Unknown at the next slot: deferred again
        io.ants_c = None;
        advance(&mut d, &mut io, MINUTE_MS + 3 * SECOND_MS, SECOND_MS);
        assert_eq!(d.ants_gate_status(), (2, None));
        // A reset keeps the count
        let mut d = boot(&mut io, Reset::PowerKept);
        assert_eq!(d.ants_gate_status().0, 2, "persisted");
        // Warm (at the threshold) at the next slot: the attempt goes ahead
        io.ants_c = Some(ANTS_MIN_TEMP_C);
        advance(&mut d, &mut io, RETRY_DELAY_MS + 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna));
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 1);
        assert_eq!(d.ants_gate_status(), (2, Some(ANTS_MIN_TEMP_C)));
        io.sense[Element::Antenna as usize] = true;
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS + 3 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::Complete);
        assert_eq!(io.unpowered_reads, 0);
    }

    /// RID P3F-A-04: a reset loop inside a deferral slot spends no
    /// deferral: the slot resumes where it was persisted, whatever the reset
    #[test]
    fn resets_inside_a_slot_spend_no_deferral() {
        let mut io = Bench::new();
        io.ants_c = Some(-40.0);
        let mut d = at_the_antenna(&mut io);
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert_eq!(d.ants_gate_status().0, 1);
        // Forty resets, ten minutes apart: the 50-minute slot ends after five
        let mut reads = io.ants_reads;
        for k in 0..40u32 {
            let reset = if k % 2 == 0 { Reset::PowerKept } else { Reset::PowerLost };
            d = boot(&mut io, reset);
            if reset == Reset::PowerLost {
                wait_out(&mut d, &mut io); // the 30-minute wait restarts
            }
            advance(&mut d, &mut io, 10 * MINUTE_MS, SECOND_MS);
            assert!(io.no_burn() || io.arm, "never burns cold");
            assert!(!io.burning(Element::Antenna));
            reads = reads.max(io.ants_reads);
        }
        let (deferrals, _) = d.ants_gate_status();
        // 40 resets over 600 min plus 20 waits of 30 min: about 1,200 min of
        // pending time at most, 24 slots; without the persisted slot every
        // reset would have spent one
        assert!(deferrals <= 25, "{deferrals}");
        assert!(deferrals < ANTS_MAX_DEFERRALS);
        assert_eq!(io.ants_reads, u32::from(deferrals), "one read per slot, none per reset");
        // The ground's force flag survives a reset too
        let mut io = Bench::new();
        io.ants_c = Some(-40.0);
        let mut d = at_the_antenna(&mut io);
        assert!(d.retry(Element::Antenna, true, &mut io));
        let mut d = boot(&mut io, Reset::PowerKept);
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna), "forced, after the reset");
    }

    /// After 30 deferrals (about 25 h) the AntS deploys whatever its
    /// temperature, but in sunlight after 20 minutes of continuous sun
    /// (RID P3F-G-02); failed sun sensors hold it at most two slots
    #[test]
    fn the_deferrals_are_capped_and_released_in_sunlight() {
        let mut io = Bench::new();
        io.ants_c = Some(-30.0);
        let mut d = at_the_antenna(&mut io);
        for n in 1..=ANTS_MAX_DEFERRALS {
            advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
            assert_eq!(d.ants_gate_status().0, n);
            assert!(io.no_burn());
            advance(&mut d, &mut io, RETRY_DELAY_MS, MINUTE_MS);
        }
        // At the cap, in eclipse: nothing powered
        io.sunlit = false;
        advance(&mut d, &mut io, 30 * MINUTE_MS, SECOND_MS);
        assert!(io.no_burn());
        // Sunrise: 20 minutes of sun, then the controller started and the AntS told to deploy
        io.sunlit = true;
        advance(&mut d, &mut io, ANTS_SUN_MS - SECOND_MS, SECOND_MS);
        assert!(io.no_burn(), "not before 20 minutes of sun");
        advance(&mut d, &mut io, 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna), "the cap: deployed cold, in sunlight");
        assert_eq!(d.ants_gate_status().0, ANTS_MAX_DEFERRALS);
        // Never sunlit (failed sun sensors): released after ANTS_SUN_WAIT_MS
        let mut io = Bench::new();
        io.ants_c = Some(-30.0);
        io.sunlit = false;
        let mut d = at_the_antenna(&mut io);
        d.ants_deferrals = ANTS_MAX_DEFERRALS;
        advance(&mut d, &mut io, ANTS_SUN_WAIT_MS - MINUTE_MS, SECOND_MS);
        assert!(io.no_burn());
        advance(&mut d, &mut io, MINUTE_MS + 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna), "the wait for the sun is bounded");
    }

    /// Three unknown temperatures in a row (a failed temperature path):
    /// the analysis schedule instead of 25 h of deferrals; a valid reading
    /// returns to the gate (RID P3F-G-02)
    #[test]
    fn unknown_temperatures_fall_back_to_the_schedule() {
        let mut io = Bench::new();
        io.ants_c = None;
        io.sunlit = false;
        let mut d = at_the_antenna(&mut io);
        for n in 1..=2 {
            advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
            assert_eq!(d.ants_gate_status().0, n);
            advance(&mut d, &mut io, RETRY_DELAY_MS, MINUTE_MS);
        }
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert_eq!(io.last, Some(Report::AntsSchedule));
        assert_eq!(d.ants_gate_status().0, 2, "the third unknown reading defers no more");
        assert!(io.no_burn(), "waiting for the sun, unpowered");
        let reads = io.ants_reads;
        advance(&mut d, &mut io, 30 * MINUTE_MS, MINUTE_MS);
        assert_eq!(io.ants_reads, reads);
        io.sunlit = true;
        advance(&mut d, &mut io, ANTS_SUN_MS + 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna), "the first 20 minutes of sun after S + 69 min");
        assert!(d.telemetry().time_since_sep_s >= ANTS_SCHEDULE_S);
        assert!(d.telemetry().time_since_sep_s < 5 * 3600, "hours, not 25 h");
        // Not before separation + 69 minutes, even in sunlight
        let mut io = Bench::new();
        io.ants_c = None;
        let mut d = at_the_antenna(&mut io);
        d.ants_unknown = ANTS_UNKNOWN_LIMIT;
        advance(&mut d, &mut io, (ANTS_SCHEDULE_S - WAIT_TIME_S - 2 * 60) * 1000, SECOND_MS);
        assert!(io.no_burn(), "{}", d.telemetry().time_since_sep_s);
        advance(&mut d, &mut io, 4 * MINUTE_MS, SECOND_MS);
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 1, "attempted at S + 69 min");
        // A valid cold reading on the schedule's attempt returns to the gate
        let mut io = Bench::new();
        io.ants_c = Some(-35.0);
        let mut d = at_the_antenna(&mut io);
        d.ants_unknown = ANTS_UNKNOWN_LIMIT;
        d.since_separation_s = ANTS_SCHEDULE_S;
        d.sun_ms = ANTS_SUN_MS;
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert!(io.no_burn());
        assert_eq!(d.ants_gate_status(), (1, Some(-35.0)), "deferred by the gate again");
    }

    /// DEPLOY_RETRY with the force flag deploys the antenna at once, cold
    /// or not (after the status read); the flag is refused for the wings
    #[test]
    fn the_ground_can_force_the_antenna() {
        let mut io = Bench::new();
        io.ants_c = Some(-40.0);
        let mut d = at_the_antenna(&mut io);
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert_eq!(d.ants_gate_status().0, 1);
        assert!(!d.retry(Element::SolarPanel, true, &mut io), "the flag is for the antenna only");
        assert!(d.retry(Element::Antenna, true, &mut io));
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna), "at once, without the slot or the temperature");
        assert_eq!(io.ants_reads, 1);
        // Without the flag the gate applies again
        let mut io = Bench::new();
        io.ants_c = Some(-40.0);
        let mut d = at_the_antenna(&mut io);
        assert!(d.retry(Element::Antenna, false, &mut io));
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert!(io.no_burn());
        assert_eq!(d.ants_gate_status().0, 1);
    }

    /// RID SMA-B02: once the record shows separation (or the RF silence is
    /// served), a stowed reading after a reset (DEPLOY_SENSE open) restarts
    /// nothing; before that it still does
    #[test]
    fn a_stowed_reading_after_separation_restarts_nothing() {
        for reset in [Reset::PowerKept, Reset::PowerLost] {
            let mut io = Bench::new();
            let mut d = boot(&mut io, Reset::PowerLost);
            d.tick(SECOND_MS, &mut io);
            advance(&mut d, &mut io, 10 * MINUTE_MS, MINUTE_MS);
            // The line fails open: stowed from now on; then a reset
            io.stowed = true;
            let mut d = boot(&mut io, reset);
            assert!(d.separation_known(), "{reset:?}");
            io.sense[Element::SolarPanel as usize] = true;
            advance(&mut d, &mut io, WAIT_TIME_MS + 2 * MINUTE_MS, SECOND_MS);
            assert!(
                matches!(d.state(), State::AntennaPending | State::AntennaActive | State::AntennaVerify),
                "{reset:?} {:?}",
                d.state()
            );
            // and across a second reset
            let d = boot(&mut io, Reset::PowerLost);
            assert!(d.separation_known());
        }
        // The served RF silence alone (a lost deployment record)
        let mut io = Bench::new();
        io.stowed = true;
        io.silence = true;
        let mut d = boot(&mut io, Reset::PowerLost);
        assert!(d.separation_known());
        advance(&mut d, &mut io, WAIT_TIME_MS + 2 * MINUTE_MS, MINUTE_MS);
        assert_ne!(d.state(), State::Inhibited);
        // GSE ARM: a fresh sequence, which a stowed switch holds again
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        d.tick(SECOND_MS, &mut io);
        assert!(d.arm(&mut io));
        io.silence = true; // an RF-silence record left from a test: ARMED ignores it
        let mut d = boot(&mut io, Reset::PowerLost);
        assert!(!d.separation_known());
        io.stowed = true;
        advance(&mut d, &mut io, WAIT_TIME_MS + 2 * MINUTE_MS, MINUTE_MS);
        assert_eq!(d.state(), State::Inhibited);
        // Never separated (the bench): stowed holds it after a reset too
        let mut io = Bench::new();
        io.stowed = true;
        let _ = boot(&mut io, Reset::PowerLost);
        let mut d = boot(&mut io, Reset::PowerKept);
        advance(&mut d, &mut io, WAIT_TIME_MS + 2 * MINUTE_MS, MINUTE_MS);
        assert_eq!((d.state(), d.separation_known()), (State::Inhibited, false));
    }

    /// RID P3F-A-01: the AntS answers only while powered; the sequence
    /// confirms a deployed AntS at the first attempt's verification (no
    /// second burn, COMPLETE), and never reads it unpowered
    #[test]
    fn the_ants_is_read_only_while_powered() {
        let mut io = Bench::new();
        let mut d = at_the_antenna(&mut io);
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna));
        io.sense[Element::Antenna as usize] = true; // its elements deploy
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS + 3 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::Complete);
        assert_eq!(d.element_status(Element::Antenna).attempt_count, 1, "no second burn");
        assert!(!d.downlink_held());
        assert_eq!(io.unpowered_reads, 0);
        assert!(io.no_burn());
        // Inhibited as the verification begins: not read, and retried at
        // the window's end without powering
        let mut io = Bench::new();
        let mut d = at_the_antenna(&mut io);
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS + 2 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::AntennaVerify);
        assert!(d.set_enabled(false, &mut io));
        io.sense[Element::Antenna as usize] = true;
        advance(&mut d, &mut io, RETRY_DELAY_MS + MINUTE_MS, SECOND_MS);
        assert_eq!(d.state(), State::AntennaPending);
        assert!(io.no_burn() && io.unpowered_reads == 0);
        // Inhibited in ANTENNA_PENDING: neither powered nor read
        let mut io = Bench::new();
        let mut d = at_the_antenna(&mut io);
        assert!(d.set_enabled(false, &mut io));
        advance(&mut d, &mut io, 10 * MINUTE_MS, SECOND_MS);
        assert!(io.no_burn() && io.ants_reads == 0 && io.unpowered_reads == 0);
    }

    /// Decision C-19: the transmitters are held from boot, through the wait
    /// and the wings, until the AntS reads deployed; the end is latched, so
    /// neither a reset nor a later DEPLOY_RETRY brings the hold back
    #[test]
    fn the_downlink_is_held_until_the_ants_reads_deployed() {
        let mut io = Bench::new();
        assert!(!Deployment::new(10).downlink_held(), "not before init");
        let mut d = boot(&mut io, Reset::PowerLost);
        assert!(d.downlink_held(), "from boot");
        io.sense[Element::SolarPanel as usize] = true;
        wait_out(&mut d, &mut io);
        assert!(d.downlink_held(), "the wings out, the AntS not yet attempted");
        advance(&mut d, &mut io, 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna));
        assert!(d.downlink_held(), "during the AntS's burn");
        io.sense[Element::Antenna as usize] = true;
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS - 2 * SECOND_MS, SECOND_MS);
        assert!(d.downlink_held(), "the AntS's 45 s window runs out first");
        advance(&mut d, &mut io, 4 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::Complete);
        assert!(!d.downlink_held());
        // Latched: across both kinds of reset and a ground retry
        for reset in [Reset::PowerKept, Reset::PowerLost] {
            let d = boot(&mut io, reset);
            assert!(!d.downlink_held(), "{reset:?}");
        }
        let mut d = boot(&mut io, Reset::PowerKept);
        io.sense[Element::Antenna as usize] = false;
        assert!(d.retry(Element::Antenna, false, &mut io));
        assert_eq!(d.state(), State::AntennaPending);
        assert!(!d.downlink_held(), "a ground retry does not silence the beacon again");
        let d = boot(&mut io, Reset::PowerKept);
        assert!(!d.downlink_held(), "the flag is in the record");
        // GSE ARM starts it over
        let mut d = boot(&mut io, Reset::PowerKept);
        assert!(d.arm(&mut io));
        assert!(d.downlink_held());
        // A record written before the hold's flag existed: a deployed
        // antenna, or a sequence already over, is not held either
        let w = WAIT_TIME_S.to_le_bytes();
        for (state, flags) in [(State::SolarPending, 0x0D), (State::Failed, 0x0E)] {
            let mut io = Bench::new();
            let rec = [state as u8, 1, 3, flags, w[0], w[1], w[2], w[3], 0, 0, 0, 0];
            nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &rec).unwrap();
            let d = boot(&mut io, Reset::PowerKept);
            assert_eq!(d.state(), state);
            assert!(!d.downlink_held(), "{state:?}");
        }
        // The host simulation's finished record
        let mut io = Bench::new();
        nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &Deployment::complete_record()).unwrap();
        let d = boot(&mut io, Reset::PowerLost);
        assert_eq!(d.state(), State::Complete);
        assert!(d.all_complete() && d.is_separated() && !d.downlink_held());
    }

    /// Decision C-19 with a cold AntS: held through the deferrals and the
    /// attempt at the cap, released once that attempt has been made even
    /// with the elements stowed; a reset keeps the release
    #[test]
    fn the_downlink_waits_for_the_attempt_at_the_deferral_cap() {
        let mut io = Bench::new();
        io.ants_c = Some(-40.0);
        let mut d = at_the_antenna(&mut io);
        for _ in 0..ANTS_MAX_DEFERRALS {
            advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
            assert!(io.no_burn() && d.downlink_held());
            advance(&mut d, &mut io, RETRY_DELAY_MS, MINUTE_MS);
        }
        // A power-losing reset at the cap: the wait restarts, the hold stays
        let mut d = boot(&mut io, Reset::PowerLost);
        assert_eq!(d.ants_gate_status().0, ANTS_MAX_DEFERRALS);
        assert!(d.downlink_held());
        wait_out(&mut d, &mut io);
        // Sunlit through the 30-minute wait: 20 minutes of sun already
        advance(&mut d, &mut io, 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna), "the cap reached: deployed cold, in sunlight");
        assert!(d.downlink_held(), "held during the attempt");
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS, SECOND_MS);
        assert_eq!(d.state(), State::AntennaVerify);
        assert!(!d.downlink_held(), "the attempt at the cap made: the stowed-antenna beacon");
        let d = boot(&mut io, Reset::PowerKept);
        assert!(!d.downlink_held());
        // Before the cap a warm attempt that leaves the AntS stowed keeps it
        let mut io = Bench::new();
        let mut d = at_the_antenna(&mut io);
        advance(&mut d, &mut io, ANTENNA_BURN_TIME_MS + 5 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::AntennaVerify);
        assert!(d.downlink_held(), "attempt 1 of 3, not at the cap");
        // ... until the attempts are spent
        advance(&mut d, &mut io, 3 * RETRY_DELAY_MS + 2 * MINUTE_MS, SECOND_MS);
        assert_eq!(d.state(), State::Failed);
        assert!(!d.downlink_held());
        assert_eq!(io.unpowered_reads, 0);
    }

    /// Decision C-19: never while the ground has inhibited deployment; a
    /// corrupt record starts the sequence, and the hold, over
    #[test]
    fn the_downlink_hold_yields_to_the_ground_and_restarts_with_the_sequence() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        assert!(d.downlink_held());
        assert!(d.set_enabled(false, &mut io));
        assert!(!d.downlink_held());
        assert!(d.set_enabled(true, &mut io));
        assert!(d.downlink_held());
        // Complete, then a corrupt record: held again
        io.sense = [true; 2];
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 5 * SECOND_MS, SECOND_MS);
        assert!(!d.downlink_held());
        io.flash.bytes[8] ^= 0xFF;
        io.flash.bytes[SLOT_SIZE + 8] ^= 0xFF;
        let d = boot(&mut io, Reset::PowerKept);
        assert_eq!(io.last, Some(Report::Unreadable));
        assert!(d.downlink_held());
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
        assert!(io.burning(Element::SolarPanel));

        // Three wing attempts RETRY_DELAY_MS apart (LV-07), reported, then
        // the antenna anyway: failed wings must not leave it stowed
        advance(&mut d, &mut io, 3 * RETRY_DELAY_MS + 2 * MINUTE_MS, SECOND_MS);
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, MAX_RETRIES);
        assert!(matches!(d.state(), State::AntennaActive | State::AntennaVerify), "{:?}", d.state());
        let reports = io.reports;
        advance(&mut d, &mut io, 3 * RETRY_DELAY_MS + 2 * MINUTE_MS, SECOND_MS);
        assert_eq!(d.state(), State::Failed);
        assert_eq!(io.last, Some(Report::Failed(Element::Antenna)));
        assert!(io.reports > reports);
        assert!(d.force_retry(Element::SolarPanel, &mut io));
        assert_eq!(d.state(), State::SolarPending);
        d.tick(SECOND_MS, &mut io);
        assert!(io.burning(Element::SolarPanel));
        assert!(!d.force_retry(Element::Antenna, &mut io), "not during a burn");
    }

    /// Wings that never sense deployed and an antenna that does: the
    /// antenna is still deployed, and the sequence ends FAILED, not COMPLETE
    #[test]
    fn failed_wings_then_a_deployed_antenna_end_failed() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 3 * RETRY_DELAY_MS + 2 * MINUTE_MS, SECOND_MS);
        assert_eq!(io.last, Some(Report::Failed(Element::SolarPanel)));
        io.sense[0] = true;
        // Confirmed at the next read: after the verification window
        advance(&mut d, &mut io, RETRY_DELAY_MS + 3 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::Failed);
        assert!(d.element_status(Element::Antenna).deployed && !d.all_complete());
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

    /// Mutation check: what a reset restores. The deployed flags (solar as
    /// well as antenna), the attempted flags and the separation flag come
    /// back from the record, in COMPLETE as in a resumed sequence.
    /// Mutation check: what a reset restores. The deployed flags (solar as
    /// well as antenna), the attempted flags and the separation flag come
    /// back from the record, in COMPLETE as in a resumed sequence.
    #[test]
    fn reset_restores_flags_attempts_and_separation() {
        // Wings out, antenna not yet: both flags must survive a reset apart
        let mut io = Bench::new();
        io.sense[1] = true;
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 3 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna));
        io.sense[1] = false; // the sense line no longer confirms it
        let d = boot(&mut io, Reset::PowerKept);
        let (antenna, solar) = (d.element_status(Element::Antenna), d.element_status(Element::SolarPanel));
        assert!(solar.deployed && !solar.deploy_attempted);
        assert!(!antenna.deployed && antenna.deploy_attempted);
        assert_eq!(antenna.attempt_count, 1);
        assert!(d.is_separated());

        // COMPLETE: both deployed flags and the separation flag are kept
        let mut io = Bench::new();
        io.sense = [true; 2];
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        advance(&mut d, &mut io, 5 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::Complete);
        io.sense = [false; 2];
        for reset in [Reset::PowerLost, Reset::PowerKept] {
            let d = boot(&mut io, reset);
            assert_eq!(d.state(), State::Complete);
            assert!(d.all_complete(), "{reset:?}");
            assert!(d.element_status(Element::SolarPanel).deployed);
            assert!(d.is_separated(), "{reset:?}");
        }

        // The separation flag is read, not assumed: a stored COMPLETE or a
        // resumed INHIBITED without it is not separated
        let mut io = Bench::new();
        let flags = FLAG_ANTENNA_DEPLOYED | FLAG_SOLAR_DEPLOYED | FLAG_ENABLED;
        nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &[State::Complete as u8, 1, 1, flags, 0, 0, 0, 0, 0, 0, 0, 0])
            .unwrap();
        let d = boot(&mut io, Reset::PowerKept);
        assert_eq!((d.state(), d.is_separated()), (State::Complete, false));
        assert_eq!(d.telemetry().time_since_sep_s, 0);
        let mut io = Bench::new();
        let _ = boot(&mut io, Reset::PowerLost); // stores INHIBITED, not separated
        let d = boot(&mut io, Reset::PowerKept);
        assert_eq!((d.state(), d.is_separated()), (State::Inhibited, false));
        let s = d.element_status(Element::Antenna);
        assert!(!s.deploy_attempted && !s.deployed);
    }

    /// Mutation check: a stowed switch restarts a wait that has not yet
    /// counted a second (WAITING at 0 s), and, once back in INHIBITED at
    /// 0 s, writes nothing more however long it stays stowed
    #[test]
    fn stowed_restart_from_waiting_and_no_rewrites_while_stowed() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        d.tick(SECOND_MS, &mut io);
        assert_eq!((d.state(), d.wait_elapsed_s()), (State::Waiting, 0));
        io.stowed = true;
        d.tick(SECOND_MS, &mut io);
        assert_eq!((d.state(), d.is_separated()), (State::Inhibited, false));
        let d2 = boot(&mut io, Reset::PowerKept);
        assert_eq!((d2.state(), d2.is_separated()), (State::Inhibited, false), "the restart is persisted");

        let flash = io.flash.bytes;
        let log_len = io.text.len;
        advance(&mut d, &mut io, 10 * MINUTE_MS, SECOND_MS);
        assert!(io.flash.bytes == flash, "no flash writes while stowed and already restarted");
        assert_eq!(io.text.len, log_len, "no state changes logged");
    }

    /// Wings not sensed after their burn are retried after the retry delay,
    /// on the other driver (B), with the attempt counted, and not fired
    /// again before it
    #[test]
    fn solar_panel_not_sensed_is_retried_after_the_delay() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        assert!(io.only(0, 0));
        advance(&mut d, &mut io, WINGS_BURN_TIME_MS, SECOND_MS);
        assert_eq!(d.state(), State::SolarVerify);
        assert!(!d.wings_exhausted());
        advance(&mut d, &mut io, RETRY_DELAY_MS - 2 * SECOND_MS, SECOND_MS);
        assert_eq!(d.state(), State::SolarVerify);
        assert!(io.no_burn());
        while d.state() == State::SolarVerify {
            d.tick(SECOND_MS, &mut io);
        }
        assert_eq!(d.state(), State::SolarPending);
        assert!(io.no_burn());
        d.tick(SECOND_MS, &mut io);
        assert!(io.only(0, 1), "wing 1, resistor B");
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, 2);
        assert_eq!(io.persisted_at_burn, Some(2));
        advance(&mut d, &mut io, SOLAR_BURN_TIME_MS, SECOND_MS);
        assert!(io.only(1, 1), "wing 2, resistor B");
        assert_eq!(io.persisted_at_burn, Some(2));
        // The third attempt is on the A resistors again
        advance(&mut d, &mut io, SOLAR_BURN_TIME_MS + RETRY_DELAY_MS + SECOND_MS, SECOND_MS);
        assert!(io.only(0, 0), "resistor A");
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, 3);
        assert!(!d.wings_exhausted(), "still in the wing phase");
        advance(&mut d, &mut io, WINGS_BURN_TIME_MS + RETRY_DELAY_MS + SECOND_MS, SECOND_MS);
        assert!(matches!(d.state(), State::AntennaPending | State::AntennaActive), "{:?}", d.state());
        assert!(d.wings_exhausted(), "spent, not deployed: EPS-10");
        assert_eq!(io.wing_max, 1);
    }

    /// Inhibited by command during verification: the retry waits in
    /// PENDING without burning or spending an attempt until enabled again
    #[test]
    fn inhibited_during_verify_holds_the_retry() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        wait_out(&mut d, &mut io);
        d.tick(SECOND_MS, &mut io);
        d.tick(SECOND_MS, &mut io);
        advance(&mut d, &mut io, WINGS_BURN_TIME_MS, SECOND_MS);
        assert_eq!(d.state(), State::SolarVerify);
        assert!(d.set_enabled(false, &mut io));
        advance(&mut d, &mut io, RETRY_DELAY_MS + 10 * MINUTE_MS, SECOND_MS);
        assert_eq!(d.state(), State::SolarPending);
        assert!(io.no_burn());
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, 1);
        // The sense line still confirms a deployment while inhibited
        io.sense[1] = true;
        d.tick(SECOND_MS, &mut io);
        assert_eq!(d.state(), State::AntennaPending);
        advance(&mut d, &mut io, 10 * MINUTE_MS, SECOND_MS);
        assert!(io.no_burn());
        assert!(d.set_enabled(true, &mut io));
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna));
    }

    /// A resumed record that already marks the element deployed is never
    /// fired, even with the sense line not confirming it
    /// A resumed record that already marks the element deployed is never
    /// fired, even with the sense line not confirming it
    #[test]
    fn stored_deployed_flag_is_never_fired() {
        let mut io = Bench::new();
        let flags = FLAG_SOLAR_DEPLOYED | FLAG_ENABLED | FLAG_SEPARATED;
        let rec = [State::SolarPending as u8, 0, 0, flags, 0x08, 0x07, 0, 0, 0, 0, 0, 0];
        nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &rec).unwrap();
        let mut d = boot(&mut io, Reset::PowerKept);
        assert_eq!(d.state(), State::SolarPending);
        d.tick(SECOND_MS, &mut io);
        assert_eq!(d.state(), State::AntennaPending);
        assert!(io.no_burn());
        advance(&mut d, &mut io, 2 * SECOND_MS, SECOND_MS);
        assert!(io.burning(Element::Antenna) && io.wing == [[false; 2]; 2]);
        assert_eq!(d.element_status(Element::SolarPanel).attempt_count, 0);
        // A record at ANTENNA_PENDING that already marks the AntS deployed:
        // finished without powering or reading it
        let mut io = Bench::new();
        let flags = FLAG_ANTENNA_DEPLOYED | FLAG_SOLAR_DEPLOYED | FLAG_ENABLED | FLAG_SEPARATED;
        let rec = [State::AntennaPending as u8, 1, 1, flags, 0x08, 0x07, 0, 0, 0, 0, 0, 0];
        nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &rec).unwrap();
        let mut d = boot(&mut io, Reset::PowerKept);
        d.tick(SECOND_MS, &mut io);
        assert_eq!(d.state(), State::Complete);
        assert!(io.no_burn() && io.ants_reads == 0);
    }

    /// A stowed switch clears a stored wait count even in INHIBITED (a
    /// resumed record), and persists the restart
    #[test]
    fn stowed_clears_a_resumed_wait_in_inhibited() {
        let mut io = Bench::new();
        let rec = [State::Inhibited as u8, 0, 0, FLAG_ENABLED, 0x2C, 0x01, 0, 0, 0, 0, 0, 0];
        nvm::save(&mut io.flash, 10, 0, RECORD_MAGIC, &rec).unwrap();
        let mut d = boot(&mut io, Reset::PowerKept);
        assert_eq!((d.state(), d.wait_elapsed_s()), (State::Inhibited, 300));
        io.stowed = true;
        d.tick(SECOND_MS, &mut io);
        assert_eq!((d.state(), d.wait_elapsed_s()), (State::Inhibited, 0));
        assert!(io.no_burn());
        assert_eq!(boot(&mut io, Reset::PowerKept).wait_elapsed_s(), 0, "the restart is persisted");
    }

    #[test]
    fn save_state_reports_persistence() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        d.tick(SECOND_MS, &mut io);
        advance(&mut d, &mut io, 10 * SECOND_MS, SECOND_MS);
        assert!(d.save_state(&mut io.flash));
        assert_eq!(boot(&mut io, Reset::PowerKept).wait_elapsed_s(), 10, "saved at once, not at the interval");
        io.flash.writes_left = Some(0);
        assert!(!d.save_state(&mut io.flash));
    }

    #[test]
    fn log_names_the_states_and_elements() {
        let mut io = Bench::new();
        let mut d = boot(&mut io, Reset::PowerLost);
        assert!(io.text.contains(b"Deployment initialized in INHIBITED\n"));
        d.tick(SECOND_MS, &mut io);
        assert!(io.text.contains(b"State: INHIBITED -> WAITING\n"));
        assert!(io.text.contains(b"Separation detected\n"));
        assert_eq!(Element::from_raw(0), Some(Element::Antenna));
        assert_eq!(Element::from_raw(1), Some(Element::SolarPanel));
        assert_eq!(State::Waiting.label(), "WAITING");
    }
}
