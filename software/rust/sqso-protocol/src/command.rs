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

    #[test]
    fn accepts_len_is_inclusive_at_both_ends() {
        let spec = crate::CommandSpec { min_args: 1, max_args: 2, key: KeyClass::Operational, without_key: false };
        assert_eq!([0, 1, 2, 3].map(|n| spec.accepts_len(n)), [false, true, true, false]);
        let timeout = Command::SetContactTimeout.spec();
        assert_eq!([1, 2, 3].map(|n| timeout.accepts_len(n)), [false, true, false]);
        assert!(Command::Nop.spec().accepts_len(0));
        assert!(!Command::Nop.spec().accepts_len(1));
    }

    /// The generated table's source (scripts/gen_command_dictionary.py)
    const DICTIONARY_JSON: &str = include_str!("../../../flight/command_dictionary.json");

    /// The string value after `"<key>": "` on a JSON line
    fn json_str<'a>(line: &'a str, key: &str) -> &'a str {
        let (_, rest) = line.split_once(key).unwrap();
        let rest = rest.trim_start_matches([':', ' ']).strip_prefix('"').unwrap();
        rest.split_once('"').unwrap().0
    }

    #[test]
    fn names_and_summaries_match_the_json_dictionary() {
        let mut count = 0;
        for line in DICTIONARY_JSON.lines().filter(|l| l.trim_start().starts_with("{\"name\"")) {
            let name = json_str(line, "\"name\"");
            let id = json_str(line, "\"id\"").strip_prefix("0x").unwrap();
            let cmd = Command::from_id(u8::from_str_radix(id, 16).unwrap()).unwrap();
            assert_eq!(cmd.name(), name);
            assert_eq!(cmd.c_name().to_str(), Ok(name));
            assert_eq!(cmd.summary(), json_str(line, "\"summary\""), "{name}");
            count += 1;
        }
        assert_eq!(count, Command::ALL.len());
    }
}
