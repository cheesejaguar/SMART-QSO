//! GNSS receiver bytes (NMEA) into the line receiver, with receive times
//! taken from the input: no panic, the line stays within its buffer, and
//! an accepted fix always meets the acceptance rules (a plausible LEO
//! record, received within GGA_MAX_AGE_MS of a real-fix GGA). Some input
//! bytes splice in whole reference sentences, so the accepting paths are
//! reached as well as the refusing ones.
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_fsw::gnss::{self, GGA_MAX_AGE_MS, LINE_MAX, Receiver, Status};

const PIECES: [&[u8]; 8] = [
    b"$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n",
    b"$PSLSP,3061.1001020,801,-6602427.515,97187.285,-1721790.821,7,2.6*7F\r\n",
    b"$GPGGA,123519,4807.038,N,01131.000,E,6,08,0.9,545.4,M,46.9,M,,*40\r\n",
    b"$PSLSP,",
    b"*7F",
    b"\r\n",
    b"-6602427.515,",
    b"$GPGGA,",
];

fn check(rx: &mut Receiver, byte: u8, now: u64) {
    let before = rx.fix;
    let status = rx.feed(byte, now);
    assert!(rx.len < LINE_MAX);
    // Only an accepted record changes the fix
    if !matches!(status, None | Some(Status::Ok)) {
        assert_eq!(rx.fix, before);
    }
    if rx.fix != before {
        let fix = rx.fix;
        assert_eq!(status, Some(Status::Ok));
        assert!(fix.valid && fix.rx_ms == now);
        assert!(gnss::plausible(&fix));
        assert!(rx.gga_seen && (rx.gga_quality == 1 || rx.gga_quality == 2));
        assert!(now >= rx.gga_ms && now - rx.gga_ms <= u64::from(GGA_MAX_AGE_MS));
    }
    if let Some(fix) = rx.latest_fix(now, u32::MAX) {
        assert!(fix.sats >= gnss::MIN_SATS && fix.pdop > 0.0);
    }
}

fuzz_target!(|data: &[u8]| {
    let mut rx = Receiver::new();
    let mut now: u64 = 0;
    for pair in data.chunks(2) {
        let [byte, rest @ ..] = pair else { continue };
        let extra = rest.first().copied().unwrap_or(0);
        now += u64::from(extra & 0x0F);
        if *byte == 0xFF {
            let piece = PIECES.get(usize::from(extra >> 5)).copied().unwrap_or(b"");
            for b in piece {
                check(&mut rx, *b, now);
            }
        } else {
            check(&mut rx, *byte, now);
        }
    }
    let _ = gnss::checksum_ok(data);
});
