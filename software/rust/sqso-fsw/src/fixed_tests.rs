//! `Fixed` against core's `{:.N}` formatting

extern crate std;

use std::format;
use std::string::String;

use super::*;

fn ours(v: f64, d: u8) -> String {
    format!("{}", Fixed::new(v, d))
}

fn check(v: f64) {
    for d in 0..=MAX_DECIMALS {
        let core = format!("{:.*}", usize::from(d), v);
        assert_eq!(ours(v, d), core, "{v:e} ({:#018x}) with {d} decimals", v.to_bits());
    }
}

/// xorshift64*: deterministic, so a failure reproduces
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

#[test]
fn special_values() {
    for v in [0.0, -0.0, f64::NAN, -f64::NAN, f64::INFINITY, f64::NEG_INFINITY, f64::MAX, f64::MIN] {
        check(v);
    }
    for v in [f64::MIN_POSITIVE, 5e-324, -5e-324, f64::EPSILON, 1.0, -1.0, 0.5, 1e-10, -0.04] {
        check(v);
    }
}

#[test]
fn exact_ties_round_half_to_even() {
    assert_eq!(ours(0.125, 2), "0.12");
    assert_eq!(ours(0.375, 2), "0.38");
    assert_eq!(ours(2.5, 0), "2");
    assert_eq!(ours(3.5, 0), "4");
    assert_eq!(ours(-0.0625, 3), "-0.062");
    // Every k / 2^n with small n: all the ties a decimal scale can hit
    for n in 0..=12 {
        for k in -4096i32..=4096 {
            check(f64::from(k) / f64::from(1u32 << n));
        }
    }
}

#[test]
fn powers_and_their_neighbours() {
    for p in -1074..=1023 {
        let v = 2f64.powi(p);
        for x in [v, -v, f64::from_bits(v.to_bits() + 1), f64::from_bits(v.to_bits().saturating_sub(1))] {
            check(x);
        }
    }
    for p in -20..=22 {
        let v = 10f64.powi(p);
        check(v);
        check(f64::from_bits(v.to_bits() + 1));
        check(f64::from_bits(v.to_bits() - 1));
    }
}

#[test]
fn values_that_round_up_into_the_next_digit() {
    for v in [0.95, 0.995, 0.9995, 9.9995, 99.95, 999.9999999995, 0.05, 0.0005, 1.0e-9, 4.9999999995e-10] {
        check(v);
        check(-v);
    }
}

#[test]
fn random_bit_patterns() {
    let mut rng = Rng(0x05EE_D0F5_A7A7);
    for _ in 0..50_000 {
        check(f64::from_bits(rng.next()));
    }
}

#[test]
fn random_telemetry_range() {
    // Voltages, currents, temperatures and SOC: the values the flight software prints
    let mut rng = Rng(0x07E1_E3E7);
    for _ in 0..50_000 {
        let unit = (rng.next() >> 11) as f64 / (1u64 << 53) as f64;
        check((unit - 0.5) * 400.0);
    }
}

#[test]
fn decimals_are_capped() {
    assert_eq!(ours(1.5, 200), format!("{:.9}", 1.5));
}
