//! Uplink frame and authentication block: parse, then re-encode
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_protocol::{auth, frame};

fuzz_target!(|data: &[u8]| {
    if let Ok(f) = frame::parse(data) {
        // A parsed frame re-encodes to exactly the input
        let mut out = [0u8; frame::MAX_LEN];
        let n = frame::encode(f.cmd_id, f.payload, &mut out).expect("a parsed frame re-encodes");
        assert_eq!(&out[..n], data);
        if let Ok(block) = auth::split(f.payload) {
            assert_eq!(auth::COUNTER_SIZE + block.args.len() + auth::TAG_SIZE, f.payload.len());
        }
    }
});
