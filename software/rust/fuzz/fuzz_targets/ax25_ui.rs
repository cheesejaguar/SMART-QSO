//! AX.25 UI-frame parser: views stay inside the frame and add up to it. An
//! odd first byte appends a correct FCS to the rest, so the address parser
//! is reached.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_protocol::ax25;

fuzz_target!(|input: &[u8]| {
    let mut buf = [0u8; 512];
    let data = match input.split_first() {
        Some((mode, body)) if mode & 1 == 1 && body.len() <= 510 => {
            buf[..body.len()].copy_from_slice(body);
            buf[body.len()..body.len() + 2].copy_from_slice(&ax25::fcs(body).to_le_bytes());
            &buf[..body.len() + 2]
        }
        Some((_, body)) => body,
        None => return,
    };
    if let Ok(ui) = ax25::parse_ui(data) {
        let total = 2 * ax25::ADDR_LEN + ui.digipeaters.len() + 2 + ui.info.len() + 2;
        assert_eq!(total, data.len());
        assert_eq!(ui.digipeaters.len() % ax25::ADDR_LEN, 0);
        assert_eq!(ax25::fcs(&data[..data.len() - 2]).to_le_bytes(), data[data.len() - 2..]);
        let _ = ax25::address_matches(ui.dest, b"KK6GDB", 11);
    }
});
