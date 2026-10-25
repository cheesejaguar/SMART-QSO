//! GSE umbilical frame: a parsed frame re-encodes to the input. An odd
//! first byte gives the rest a correct CRC-32, so the operations are reached.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_protocol::checksum::crc32;
use sqso_protocol::gse::{self, FRAME_SIZE, GseOp, op};

fuzz_target!(|input: &[u8]| {
    let mut buf = [0u8; FRAME_SIZE];
    let data = match input.split_first() {
        Some((mode, body)) if mode & 1 == 1 && body.len() == FRAME_SIZE - 4 => {
            buf[..FRAME_SIZE - 4].copy_from_slice(body);
            buf[FRAME_SIZE - 4..].copy_from_slice(&crc32(body).to_be_bytes());
            &buf[..]
        }
        Some((_, body)) => body,
        None => return,
    };
    if let Ok(parsed) = gse::parse(data) {
        let (opcode, slot, key_id, key) = match &parsed {
            GseOp::Provision { slot, key_id, key } => (op::PROVISION, *slot, *key_id, key.0),
            GseOp::Activate { slot } => (op::ACTIVATE, *slot, data[6], data[7..39].try_into().unwrap()),
            GseOp::Zeroize => (op::ZEROIZE, data[5], data[6], data[7..39].try_into().unwrap()),
            GseOp::ArmFlight => (op::ARM_FLIGHT, data[5], data[6], data[7..39].try_into().unwrap()),
        };
        assert_eq!(gse::encode(opcode, slot, key_id, &key), data);
    }
});
