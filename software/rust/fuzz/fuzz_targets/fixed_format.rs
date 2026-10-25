//! Fixed-point text of any f64 with 0 to 9 decimals: exactly what core's
//! `{:.N}` writes (`sqso_fsw::fixed` replaces it on the target, SRS-C003)
#![no_main]

use libfuzzer_sys::fuzz_target;
use sqso_fsw::fixed::{Fixed, MAX_DECIMALS};

fuzz_target!(|data: &[u8]| {
    let Some((bits, rest)) = data.split_first_chunk::<8>() else { return };
    let value = f64::from_bits(u64::from_le_bytes(*bits));
    let decimals = rest.first().map_or(3, |d| d % (MAX_DECIMALS + 1));
    assert_eq!(
        format!("{}", Fixed::new(value, decimals)),
        format!("{:.*}", usize::from(decimals), value),
        "{:#018x} with {decimals} decimals",
        value.to_bits()
    );
});
