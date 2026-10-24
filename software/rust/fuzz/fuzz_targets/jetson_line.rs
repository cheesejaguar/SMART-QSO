//! OBC-Jetson line parsers, both directions: anything parsed rebuilds into
//! a line that parses to the same message. An odd first byte seals the rest
//! with a correct CRC first, so the field parsers are reached.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_protocol::jetson::{self, FromJetson, FromObc, LINE_MAX};

fuzz_target!(|input: &[u8]| {
    let mut sealed = [0u8; LINE_MAX + 8];
    let data = match input.split_first() {
        Some((mode, body)) if mode & 1 == 1 => match jetson::seal(body, &mut sealed) {
            Ok(n) => &sealed[..n],
            Err(_) => body,
        },
        Some((_, body)) => body,
        None => return,
    };
    let mut out = [0u8; LINE_MAX + 8];
    if let Ok(msg) = jetson::parse_from_jetson(data) {
        let n = match msg {
            FromJetson::BeaconRsp { seq, text } => jetson::beacon_rsp(seq, text, &mut out),
            FromJetson::Status(s) => jetson::status(&s, &mut out),
            FromJetson::Ack(t) => jetson::ack(t, &mut out),
            FromJetson::Nack(t) => jetson::nack(t, &mut out),
        }
        .expect("a parsed message rebuilds");
        assert_eq!(jetson::parse_from_jetson(&out[..n]), Ok(msg));
    }
    if let Ok(msg) = jetson::parse_from_obc(data) {
        let n = match msg {
            FromObc::BeaconReq { seq, max_len } => jetson::beacon_req(seq, max_len, &mut out),
            FromObc::StatusReq => jetson::status_req(&mut out),
            FromObc::Shutdown(r) => jetson::shutdown(r, &mut out),
            FromObc::Telemetry(t) => {
                for (k, v) in t.extras() {
                    assert!(!k.is_empty() && !k.contains(',') && !v.contains(','));
                }
                return;
            }
        }
        .expect("a parsed message rebuilds");
        assert_eq!(jetson::parse_from_obc(&out[..n]), Ok(msg));
    }
    if let Ok(body) = jetson::check(data) {
        let n = jetson::seal(body, &mut out).expect("a checked body reseals");
        assert_eq!(&out[..n - 1], data.strip_suffix(b"\n").unwrap_or(data));
    }
});
