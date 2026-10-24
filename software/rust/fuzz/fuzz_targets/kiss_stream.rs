//! KISS decoder (the telecommand receiver's TNC stream): on any byte stream
//! it never yields an empty frame or one longer than its buffer, and a frame
//! encoded with kiss_encode after arbitrary noise is decoded exactly.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_protocol::ax25::{MAX_FRAME_LEN, kiss_encode};
use sqso_protocol::kiss::{Decoder, FEND};

fuzz_target!(|input: &[u8]| {
    // Any stream
    let mut d = Decoder::<MAX_FRAME_LEN>::new(0);
    for &b in input {
        if let Some(frame) = d.feed(b) {
            assert!(!frame.is_empty() && frame.len() <= MAX_FRAME_LEN);
        }
    }

    // Noise, then a genuine frame: the frame comes out unchanged
    let Some((&split, rest)) = input.split_first() else { return };
    let cut = usize::from(split).min(rest.len());
    let (noise, payload) = rest.split_at(cut);
    let payload = &payload[..payload.len().min(MAX_FRAME_LEN)];
    if payload.is_empty() {
        return;
    }
    let mut wire = [0u8; 2 * MAX_FRAME_LEN + 3];
    let n = kiss_encode(payload, 0, &mut wire).unwrap();
    let mut d = Decoder::<MAX_FRAME_LEN>::new(0);
    for &b in noise {
        let _ = d.feed(b);
    }
    // A FEND ends whatever the noise left open
    let _ = d.feed(FEND);
    let mut got = None;
    for &b in &wire[..n] {
        if let Some(frame) = d.feed(b) {
            got = Some(frame.to_vec());
        }
    }
    assert_eq!(got.as_deref(), Some(payload));
});
