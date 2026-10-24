//! Byte stream into the line accumulator: every line it returns is within
//! the buffer, and the accumulator always recovers at a newline
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_protocol::jetson::{self, LINE_MAX, LineRx};

fuzz_target!(|data: &[u8]| {
    let mut rx = LineRx::new();
    for b in data {
        if let Some(Ok(line)) = rx.feed(*b) {
            assert!(line.len() < LINE_MAX);
            assert!(!line.contains(&b'\n'));
            let _ = jetson::parse_from_jetson(line);
        }
    }
    let line = rx.feed(b'\n');
    assert!(line.is_some());
});
