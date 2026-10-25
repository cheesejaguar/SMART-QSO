//! The whole telecommand path (frame, tag, counter, dictionary, commit)
//! against a provisioned key table.
//!
//! Safety: nothing is accepted unless its tag is the key's tag, and no
//! counter is accepted twice. Liveness: a genuine operational-key frame for
//! a dictionary command, with in-range arguments and a fresh counter, is
//! accepted. Frames are either raw fuzzer bytes or genuine frames the
//! fuzzer then corrupts, so the checks past the tag are reached.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_fsw::keys::{KeyTable, KeyUsed, RECOVERY_SLOT};
use sqso_fsw::nvm::MemFlash;
use sqso_fsw::telecommand::{Admission, admit};
use sqso_protocol::auth::{self, Key};
use sqso_protocol::{Command, KeyClass, frame};

const OP: Key = Key::new([0x5A; 32]);
const REC: Key = Key::new([0xA5; 32]);
const WRONG: Key = Key::new([0x00; 32]);

struct State {
    flash: MemFlash<1024>,
    keys: KeyTable,
    last: [u32; 2],
}

fn check(state: &mut State, raw: &[u8], genuine_op: Option<(u8, u32, usize)>) {
    let outcome = admit(raw, &mut state.keys, &mut state.flash, |_| false);
    if let Admission::Accepted { command, .. } = outcome {
        let f = frame::parse(raw).expect("accepted frames parse");
        let block = auth::split(f.payload).expect("accepted frames carry a block");
        let key = match command.key {
            KeyUsed::Operational => &OP,
            KeyUsed::Recovery => &REC,
            KeyUsed::None => panic!("keyless acceptance with keys provisioned"),
        };
        assert!(auth::verify(key, f.cmd_id, &block), "accepted without a valid tag");
        let slot = &mut state.last[usize::from(command.key == KeyUsed::Recovery)];
        assert!(command.counter > *slot, "counter accepted twice");
        *slot = command.counter;
    } else if let Some((cmd_id, counter, args)) = genuine_op {
        // Genuine, uncorrupted, operational key: refused only for a reason
        let fresh = counter > state.last[0] && counter <= auth::COUNTER_LIMIT;
        let in_dictionary =
            Command::from_id(cmd_id).is_some_and(|c| c.spec().key != KeyClass::Recovery && c.spec().accepts_len(args));
        assert!(!(fresh && in_dictionary), "a genuine fresh command was refused: {outcome:?}");
    }
}

fuzz_target!(|data: &[u8]| {
    let mut state = State { flash: MemFlash::new(), keys: KeyTable::empty(6), last: [0; 2] };
    state.keys.provision(&mut state.flash, 0, &OP, 1).unwrap();
    state.keys.provision(&mut state.flash, RECOVERY_SLOT, &REC, 2).unwrap();
    state.keys.rotate(&mut state.flash, 0, &mut |_| {}).unwrap();

    let mut rest = data;
    while let [mode, tail @ ..] = rest {
        if mode & 1 == 0 {
            // Raw frame: next byte is its length
            let Some((len, tail)) = tail.split_first() else { break };
            let take = usize::from(*len).min(tail.len());
            let (raw, next) = tail.split_at(take);
            check(&mut state, raw, None);
            rest = next;
        } else {
            // Genuine frame: key, cmd_id, counter (2 bytes, scaled), args
            // length, args, then an optional corruption (position, xor)
            let [key, cmd_id, c_hi, c_lo, n_args, tail @ ..] = tail else { break };
            let n_args = usize::from(*n_args % 70).min(tail.len());
            let (args, tail) = tail.split_at(n_args);
            let [flip_at, flip, next @ ..] = tail else { break };
            rest = next;
            let key = match key % 3 {
                0 => &OP,
                1 => &REC,
                _ => &WRONG,
            };
            let counter = u32::from_be_bytes([0, *c_hi, *c_lo, 0]) | u32::from(*mode >> 1);
            let mut out = [0u8; frame::MAX_LEN];
            let Ok(n) = auth::build_frame(key, *cmd_id, counter, args, &mut out) else { continue };
            let corrupted = *flip != 0 && usize::from(*flip_at) < n;
            if corrupted {
                out[usize::from(*flip_at)] ^= *flip;
            }
            let genuine_op = (!corrupted && std::ptr::eq(key, &OP)).then_some((*cmd_id, counter, args.len()));
            check(&mut state, &out[..n], genuine_op);
        }
    }
});
