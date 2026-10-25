//! The C math library functions the flight logic needs, for `no_std`
//! (`core` has no `sqrt` or `round`).
//!
//! Each gives the result the C library gives: [`sqrt`] is correctly
//! rounded, as IEEE 754 requires of C `sqrt`, and [`round`] rounds halfway
//! cases away from zero, as C `round` and `lround` do.

/// Square root, correctly rounded (C `sqrt`): NaN for a negative argument,
/// `-0.0` for `-0.0`
pub fn sqrt(x: f64) -> f64 {
    if x.is_nan() || x < 0.0 {
        return f64::NAN;
    }
    if x == 0.0 || x == f64::INFINITY {
        return x;
    }
    let bits = x.to_bits();
    let biased = i32::try_from((bits >> 52) & 0x7FF).unwrap_or(0);
    let mut mant = bits & ((1u64 << 52) - 1);
    let mut exp = if biased == 0 {
        // Subnormal: shift the leading one up to bit 52
        let shift = mant.leading_zeros().saturating_sub(11);
        mant <<= shift;
        1 - i32::try_from(shift).unwrap_or(0)
    } else {
        mant |= 1u64 << 52;
        biased
    };
    // x = mant * 2^e with mant in [2^52, 2^53); make e even
    exp -= 1075;
    if exp % 2 != 0 {
        mant <<= 1;
        exp -= 1;
    }
    // sqrt(x) = sqrt(mant * 2^52) * 2^((e - 52) / 2), and sqrt(mant * 2^52)
    // is in [2^52, 2^53]. No square root lies halfway between two doubles,
    // so the remainder decides the rounding
    let n = u128::from(mant) << 52;
    let mut r = n.isqrt();
    if n - r * r > r {
        r += 1;
    }
    let mut q = exp / 2 - 26;
    if r == 1u128 << 53 {
        r >>= 1;
        q += 1;
    }
    let frac = u64::try_from(r).unwrap_or(0) & ((1u64 << 52) - 1);
    let biased = u64::try_from(q + 1075).unwrap_or(0);
    f64::from_bits((biased << 52) | frac)
}

/// Round to the nearest integer, halfway cases away from zero (C `round`)
pub fn round(x: f64) -> f64 {
    /// From 2^52 up every double is an integer
    const INTEGRAL: f64 = 4_503_599_627_370_496.0;
    if x.is_nan() || x.abs() >= INTEGRAL {
        return x; // integral already, infinite or NaN
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)] // |x| < 2^52: exact
    let whole = (x as i64) as f64;
    // x - trunc(x) is exact
    let frac = x - whole;
    if frac >= 0.5 {
        whole + 1.0
    } else if frac <= -0.5 {
        whole - 1.0
    } else {
        whole
    }
}

/// C `lround`: the nearest integer, halfway cases away from zero (a result
/// beyond `i64` saturates; NaN gives 0, where C leaves both unspecified)
#[allow(clippy::cast_possible_truncation)] // saturating by definition of `as`
pub fn lround(x: f64) -> i64 {
    round(x) as i64
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    /// Doubles of many magnitudes and bit patterns, including subnormals
    fn samples() -> std::vec::Vec<f64> {
        let mut out = std::vec![
            f64::MIN_POSITIVE,
            f64::MAX,
            5e-324,
            1e-310,
            2.0,
            3.0,
            0.5,
            1e-6,
            45.0 * 45.0,
            2025.0,
            0.0087,
            1.0 - f64::EPSILON,
            1.0 + f64::EPSILON,
        ];
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        for _ in 0..200_000 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let v = f64::from_bits(state & 0x7FFF_FFFF_FFFF_FFFF);
            if v.is_finite() {
                out.push(v);
            }
        }
        out
    }

    #[test]
    fn sqrt_matches_the_correctly_rounded_result() {
        for x in samples() {
            assert_eq!(sqrt(x).to_bits(), x.sqrt().to_bits(), "sqrt({x:e})");
        }
    }

    #[test]
    fn sqrt_special_values() {
        assert!(sqrt(-1.0).is_nan());
        assert!(sqrt(f64::NAN).is_nan());
        assert!(sqrt(f64::NEG_INFINITY).is_nan());
        assert_eq!(sqrt(f64::INFINITY), f64::INFINITY);
        assert_eq!(sqrt(0.0).to_bits(), 0.0f64.to_bits());
        assert_eq!(sqrt(-0.0).to_bits(), (-0.0f64).to_bits());
        assert_eq!(sqrt(169.0), 13.0);
    }

    #[test]
    fn round_is_c_round() {
        for x in samples() {
            assert_eq!(round(x), x.round(), "round({x:e})");
            assert_eq!(round(-x), (-x).round(), "round({:e})", -x);
        }
        for (x, r) in [(0.5, 1.0), (-0.5, -1.0), (1.5, 2.0), (2.5, 3.0), (-2.5, -3.0), (0.499_999_999_999_999_94, 0.0)]
        {
            assert_eq!(round(x), r, "round({x})");
        }
        assert!(round(f64::NAN).is_nan());
        assert_eq!(round(f64::INFINITY), f64::INFINITY);
    }

    #[test]
    fn lround_scaled_telemetry() {
        // Values exact in the field's unit round to that value (test_telemetry.c)
        for mv in 0..=20_000u32 {
            assert_eq!(lround(f64::from(mv) / 1000.0 * 1000.0), i64::from(mv));
        }
        for ma in -2000..=2000i32 {
            assert_eq!(lround(f64::from(ma) / 1000.0 * 1000.0), i64::from(ma));
        }
        assert_eq!(lround(f64::NAN), 0);
        assert_eq!(lround(1e300), i64::MAX);
    }
}
