//! Sensor configuration file (SENSORS_YAML) into the parser, fed in pieces
//! whose sizes the input chooses: no panic, every field of every sensor
//! stays NUL-terminated within its array, and the pieces do not matter
//! (the same file read at once gives the same sensors)
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_fsw::sensors::{SensorDef, YamlParser};

const MAX_ITEMS: usize = 64;

fn parse(data: &[u8], piece: usize) -> ([SensorDef; MAX_ITEMS], usize, usize) {
    let mut out = [SensorDef::EMPTY; MAX_ITEMS];
    let mut kept = 0;
    let mut total = 0;
    let mut sink = |d: &SensorDef| {
        for field in [&d.id[..], &d.name, &d.units, &d.type_, &d.channel] {
            assert_eq!(field.last(), Some(&0));
        }
        if let Some(slot) = out.get_mut(kept) {
            *slot = *d;
            kept += 1;
        }
        total += 1;
    };
    let mut p = YamlParser::new();
    for chunk in data.chunks(piece.max(1)) {
        p.feed(chunk, &mut sink);
    }
    p.finish(&mut sink);
    (out, kept, total)
}

fuzz_target!(|data: &[u8]| {
    let Some((&first, rest)) = data.split_first() else { return };
    let (whole, kept, total) = parse(rest, rest.len());
    let (pieces, kept_p, total_p) = parse(rest, usize::from(first));
    assert_eq!(total, total_p);
    assert_eq!(kept, kept_p);
    assert_eq!(whole.get(..kept), pieces.get(..kept));
    for d in whole.iter().take(kept) {
        let _ = d.kind();
    }
});
