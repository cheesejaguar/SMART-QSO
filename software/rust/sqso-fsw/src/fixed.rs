//! Fixed-point text for an `f64`: the same characters as
//! `format!("{:.N}", v)` for N up to [`MAX_DECIMALS`], from a stack frame of
//! a few hundred bytes. Core's exact float formatting keeps a 1 KB digit
//! buffer and several bignums on the stack (about 2 KB), and the flight
//! software's stack budget is 8 KB (SRS-C003, `scripts/stack_usage.py`).
//!
//! Like core, the result is exact: |v| = m * 2^e is scaled by 10^N and
//! rounded half to even, with integer arithmetic only. NaN is `NaN` and
//! infinities `inf` and `-inf`; a negative value (or -0.0) keeps its sign
//! even when it rounds to zero. Width, fill and alignment flags are not
//! supported (no caller uses them). The digits go out through `write_str`
//! only: formatting nothing else keeps the call graph free of cycles
//! through `core::fmt::write`.
//!
//! @requirement SRS-C003 Stack size shall not exceed 8 KB

use core::fmt;

/// Most decimals [`Fixed`] prints
pub const MAX_DECIMALS: u8 = 9;

/// 2^1024 (more than any finite f64) needs 33 32-bit limbs
const LIMBS: usize = 33;
/// 10^9: the decimal chunk the integer printer divides by
const CHUNK: u32 = 1_000_000_000;
/// 2^1024 has 309 decimal digits: 35 chunks of 9
const CHUNKS: usize = 35;

/// `value` with `decimals` decimal places (at most [`MAX_DECIMALS`])
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fixed {
    value: f64,
    decimals: u8,
}

impl Fixed {
    /// `decimals` above [`MAX_DECIMALS`] is taken as [`MAX_DECIMALS`]
    pub fn new(value: f64, decimals: u8) -> Self {
        Fixed { value, decimals: decimals.min(MAX_DECIMALS) }
    }
}

impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = self.value;
        if v.is_nan() {
            return f.write_str("NaN");
        }
        if v.is_sign_negative() {
            f.write_str("-")?;
        }
        if v.is_infinite() {
            return f.write_str("inf");
        }
        let (m, e) = decompose(v);
        let d = u32::from(self.decimals);
        if e >= 0 {
            // An integer, m * 2^e: no rounding, the fraction is zeros
            write_integer(f, m, e.unsigned_abs())?;
            return write_fraction(f, 0, d);
        }
        let pow = 10u64.pow(d);
        let scaled = round_shift(u128::from(m) * u128::from(pow), e.unsigned_abs());
        // scaled < 2^53 * 10^9 / 2, so the quotient is below 2^53 + 1
        let int = u64::try_from(scaled / u128::from(pow)).unwrap_or(u64::MAX);
        let frac = u64::try_from(scaled % u128::from(pow)).unwrap_or(0);
        write_digits(f, int, 0)?;
        write_fraction(f, frac, d)
    }
}

/// |v| = m * 2^e with m < 2^53 (subnormals included)
fn decompose(v: f64) -> (u64, i32) {
    let bits = v.to_bits();
    let exp = i32::from(u16::try_from((bits >> 52) & 0x7FF).unwrap_or(0));
    let frac = bits & ((1 << 52) - 1);
    if exp == 0 { (frac, -1074) } else { (frac | (1 << 52), exp - 1075) }
}

/// n / 2^k rounded half to even
fn round_shift(n: u128, k: u32) -> u128 {
    if k >= 127 {
        // n < 2^83 is below half of 2^k
        return 0;
    }
    let q = n >> k;
    let rem = n & ((1u128 << k) - 1);
    let half = 1u128 << (k - 1);
    if rem > half || (rem == half && q & 1 == 1) { q + 1 } else { q }
}

/// "." and `frac` with `d` digits (nothing for 0 decimals)
fn write_fraction(f: &mut fmt::Formatter<'_>, frac: u64, d: u32) -> fmt::Result {
    if d == 0 {
        return Ok(());
    }
    f.write_str(".")?;
    write_digits(f, frac, d)
}

/// `n` in decimal, zero-padded to at least `width` digits (up to 20)
fn write_digits(f: &mut fmt::Formatter<'_>, mut n: u64, width: u32) -> fmt::Result {
    let mut buf = [b'0'; 20];
    let mut len = 0u32;
    for slot in buf.iter_mut().rev() {
        *slot = b'0' + u8::try_from(n % 10).unwrap_or(0);
        n /= 10;
        len += 1;
        if n == 0 && len >= width {
            break;
        }
    }
    let digits = buf.get(buf.len() - usize::try_from(len).unwrap_or(0)..).unwrap_or(&[]);
    f.write_str(core::str::from_utf8(digits).unwrap_or(""))
}

/// The decimal digits of m * 2^e (e >= 0, m < 2^53, m * 2^e < 2^1024).
/// Not inlined: only values of 2^53 and above need its 272 bytes of limbs
/// and chunks
#[inline(never)]
fn write_integer(f: &mut fmt::Formatter<'_>, m: u64, e: u32) -> fmt::Result {
    let mut limbs = [0u32; LIMBS];
    let word = usize::try_from(e / 32).unwrap_or(LIMBS);
    let shifted = u128::from(m) << (e % 32);
    for (i, limb) in limbs.iter_mut().skip(word).take(3).enumerate() {
        *limb = u32::try_from((shifted >> (32 * i)) & 0xFFFF_FFFF).unwrap_or(0);
    }
    // Least significant chunk first
    let mut chunks = [0u32; CHUNKS];
    let mut count = 0;
    for chunk in &mut chunks {
        *chunk = div_chunk(&mut limbs);
        count += 1;
        if limbs.iter().all(|&l| l == 0) {
            break;
        }
    }
    let mut digits = chunks.iter().take(count).rev();
    if let Some(&first) = digits.next() {
        write_digits(f, u64::from(first), 0)?;
    }
    for &chunk in digits {
        write_digits(f, u64::from(chunk), 9)?;
    }
    Ok(())
}

/// Divide the number in `limbs` by 10^9 in place; the remainder
fn div_chunk(limbs: &mut [u32; LIMBS]) -> u32 {
    let mut rem = 0u64;
    for limb in limbs.iter_mut().rev() {
        let cur = (rem << 32) | u64::from(*limb);
        *limb = u32::try_from(cur / u64::from(CHUNK)).unwrap_or(0);
        rem = cur % u64::from(CHUNK);
    }
    u32::try_from(rem).unwrap_or(0)
}

#[cfg(test)]
#[path = "fixed_tests.rs"]
mod tests;
