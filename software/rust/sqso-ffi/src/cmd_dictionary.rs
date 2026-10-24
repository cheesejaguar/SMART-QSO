//! `cmd_dictionary.h`: the command table, built at compile time from the
//! generated `sqso_protocol::Command` (one source: command_dictionary.json)
#![allow(non_upper_case_globals)]

use sqso_protocol::{Command, KeyClass};

/// `CmdDictEntry_t`
#[repr(C)]
pub struct CmdDictEntry {
    cmd_id: u8,
    min_args: u8,
    max_args: u8,
    key: u32,
    without_key: bool,
}

const COUNT: usize = Command::ALL.len();

const fn table() -> [CmdDictEntry; COUNT] {
    const BLANK: CmdDictEntry = CmdDictEntry { cmd_id: 0, min_args: 0, max_args: 0, key: 0, without_key: false };
    let mut out = [BLANK; COUNT];
    let mut i = 0;
    while i < COUNT {
        #[allow(clippy::indexing_slicing)] // i < COUNT, evaluated at compile time
        {
            let cmd = Command::ALL[i];
            let spec = cmd.spec();
            out[i] = CmdDictEntry {
                cmd_id: cmd.id(),
                min_args: spec.min_args,
                max_args: spec.max_args,
                key: match spec.key {
                    KeyClass::Operational => 0,
                    KeyClass::Recovery => 1,
                    KeyClass::Any => 2,
                },
                without_key: spec.without_key,
            };
        }
        i += 1;
    }
    out
}

/// The dictionary, in command-ID order
#[unsafe(no_mangle)]
pub static g_cmd_dictionary: [CmdDictEntry; COUNT] = table();

/// Number of entries in g_cmd_dictionary
#[unsafe(no_mangle)]
pub static g_cmd_dictionary_count: usize = COUNT;
