//! From received bytes to an authenticated command (`cmd_process()` in
//! `cmd_handler.h`), and which commands each mode accepts.
//!
//! [`admit`] runs the checks in order: framing and checksum, the
//! authentication tag and counter, the argument length against the
//! dictionary, then spends the counter (persisted before anything
//! executes, so a replay fails even after a reset). Executing the command
//! is the caller's.

use sqso_protocol::{Command, MAX_ARGS, frame};

use crate::keys::{AuthError, CommitError, KeyTable, KeyUsed};
use crate::mode::State;
use crate::nvm::Flash;

/// Why a frame was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    /// Framing, length or checksum wrong
    InvalidFrame,
    /// Authentication block malformed
    Malformed,
    /// Tag, counter or key refused
    AuthFailed(AuthError),
    /// Authentic but not in the dictionary (nor a registered handler)
    UnknownCommand,
    /// Argument length outside the dictionary's bounds
    ArgLength,
    /// The counter could not be persisted (TX_DISABLE executes anyway)
    CounterNotPersisted,
}

/// An authenticated command, its counter spent
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Admitted<'a> {
    /// Command ID
    pub cmd_id: u8,
    /// The dictionary command (`None` for a run-time registered handler)
    pub command: Option<Command>,
    /// Command counter
    pub counter: u32,
    /// Arguments
    pub args: &'a [u8],
    /// Key that authenticated it
    pub key: KeyUsed,
}

/// The result of [`admit`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admission<'a> {
    /// Refused
    Rejected {
        /// Command ID from the header (0 if the header could not be read)
        cmd_id: u8,
        /// Counter, if the frame was authentic
        counter: Option<u32>,
        /// Why
        why: Rejection,
        /// The frame was contact with the ground (its counter was spent)
        contact: bool,
    },
    /// Execute it
    Accepted {
        /// The command
        command: Admitted<'a>,
        /// Contact with the ground (false for keyless TX_DISABLE)
        contact: bool,
    },
}

/// Check a received frame and spend its counter. `custom` says whether a
/// handler is registered for an ID outside the dictionary (it may take up
/// to [`MAX_ARGS`] argument bytes).
pub fn admit<'a, F: Flash>(
    raw: &'a [u8],
    keys: &mut KeyTable,
    flash: &mut F,
    custom: impl Fn(u8) -> bool,
) -> Admission<'a> {
    let reject = |cmd_id, counter, why, contact| Admission::Rejected { cmd_id, counter, why, contact };

    let parsed = match frame::parse(raw) {
        Ok(f) => f,
        Err(e) => return reject(e.cmd_id().unwrap_or(0), None, Rejection::InvalidFrame, false),
    };
    let cmd_id = parsed.cmd_id;

    let verified = match keys.verify(cmd_id, parsed.payload) {
        Ok(v) => v,
        Err(AuthError::Format) => return reject(cmd_id, None, Rejection::Malformed, false),
        Err(e) => return reject(cmd_id, None, Rejection::AuthFailed(e), false),
    };
    let counter = Some(verified.counter);

    let command = Command::from_id(cmd_id);
    let length_ok = match command {
        Some(c) => c.spec().accepts_len(verified.args.len()),
        None if custom(cmd_id) => verified.args.len() <= MAX_ARGS,
        None => return reject(cmd_id, counter, Rejection::UnknownCommand, false),
    };
    if !length_ok {
        return reject(cmd_id, counter, Rejection::ArgLength, false);
    }

    // An authenticated frame whose counter was just spent is contact with
    // the ground, whatever its outcome. Earlier rejections leave the counter
    // unspent, so a replay could otherwise hold off the loss-of-contact
    // inhibit. The keyless TX_DISABLE never counts.
    let committed = keys.commit(flash, &verified);
    let contact = verified.key != KeyUsed::None;
    match committed {
        Ok(()) => {}
        // TX-OFF executes even if the counter was not persisted: reducing
        // emissions is always safe, and a replay can only silence the station
        Err(_) if command == Some(Command::TxDisable) => {}
        Err(CommitError::Io | CommitError::NotUsable | CommitError::Stale) => {
            return reject(cmd_id, counter, Rejection::CounterNotPersisted, contact);
        }
    }

    Admission::Accepted {
        command: Admitted { cmd_id, command, counter: verified.counter, args: verified.args, key: verified.key },
        contact,
    }
}

/// When a command may run
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    /// In every mode: TX on/off (47 CFR 97.207(b)), key rotation (restores
    /// control after a key compromise), NOP and status queries
    Always,
    /// Also in SAFE
    AlsoSafe,
    /// Also in EMERGENCY
    AlsoEmergency,
    /// Only outside SAFE and EMERGENCY
    Normal,
}

/// Every dictionary command's availability; exhaustive, so a new command
/// must be classified before the flight software builds
pub const fn availability(command: Command) -> Availability {
    use Command as C;
    match command {
        C::Nop | C::TxDisable | C::TxEnable | C::KeyRotate | C::GetStatus | C::EpsGetTelemetry | C::AdcsGetAttitude => {
            Availability::Always
        }
        C::SetMode | C::SetBeacon => Availability::AlsoSafe,
        C::Reset => Availability::AlsoEmergency,
        C::ClearFaults
        | C::SetContactTimeout
        | C::EpsSetMode
        | C::AdcsDetumble
        | C::SetPower
        | C::PldEnable
        | C::PldDisable => Availability::Normal,
    }
}

/// Whether a command ID may run in a state (`None`: a state value the mode
/// manager does not define, treated as normal operations)
pub fn allowed(cmd_id: u8, state: Option<State>) -> bool {
    let availability = Command::from_id(cmd_id).map_or(Availability::Normal, availability);
    match (availability, state) {
        (Availability::Always, _) => true,
        (Availability::AlsoSafe, Some(State::Safe)) => true,
        (Availability::AlsoEmergency, Some(State::Emergency)) => true,
        (_, Some(State::Safe | State::Emergency)) => false,
        (_, Some(State::Boot | State::Detumble | State::Idle | State::Active) | None) => true,
    }
}

#[cfg(test)]
#[path = "telecommand_tests.rs"]
mod tests;
