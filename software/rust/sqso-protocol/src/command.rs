//! Types of the command dictionary (the table itself is generated into
//! [`crate::dictionary`] from `software/flight/command_dictionary.json`).

/// Largest argument block an authenticated command can carry
pub const MAX_ARGS: usize = 64;

/// Which telecommand key authenticates a command
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KeyClass {
    /// The active operational key
    Operational = 0,
    /// The offline recovery key
    Recovery = 1,
    /// Either key
    Any = 2,
}

/// One dictionary entry
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CommandSpec {
    /// Fewest argument bytes
    pub min_args: u8,
    /// Most argument bytes
    pub max_args: u8,
    /// Authenticating key
    pub key: KeyClass,
    /// Accepted without authentication while no key is usable (TX-OFF only)
    pub without_key: bool,
}

impl CommandSpec {
    /// Whether an argument block of `len` bytes is allowed
    pub const fn accepts_len(&self, len: usize) -> bool {
        len >= self.min_args as usize && len <= self.max_args as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::Command;
    use crate::command::{KeyClass, MAX_ARGS};

    #[test]
    fn ids_round_trip_and_are_sorted() {
        let mut last = None;
        for cmd in Command::ALL {
            assert_eq!(Command::from_id(cmd.id()), Some(cmd));
            assert!(last.is_none_or(|l| l < cmd.id()));
            last = Some(cmd.id());
        }
        let known = (0..=255u8).filter(|id| Command::from_id(*id).is_some()).count();
        assert_eq!(known, Command::ALL.len());
    }

    #[test]
    fn specs_are_bounded() {
        for cmd in Command::ALL {
            let spec = cmd.spec();
            assert!(spec.min_args <= spec.max_args);
            assert!(usize::from(spec.max_args) <= MAX_ARGS);
            assert!(!spec.without_key || spec.key == KeyClass::Any, "{}", cmd.name());
        }
    }

    #[test]
    fn only_tx_disable_works_without_a_key() {
        // Without a key the station must still obey TX-OFF (47 CFR 97.207(b)),
        // and nothing else
        let mut keyless = Command::ALL.iter().filter(|c| c.spec().without_key);
        assert_eq!(keyless.next(), Some(&Command::TxDisable));
        assert_eq!(keyless.next(), None);
    }

    #[test]
    fn key_rotation_needs_the_recovery_key() {
        assert_eq!(Command::KeyRotate.spec().key, KeyClass::Recovery);
    }
}
