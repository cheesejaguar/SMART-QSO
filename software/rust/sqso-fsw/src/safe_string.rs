//! Bounded string and memory operations (`safe_string.h`).
//!
//! A C string reaches these functions as a slice holding its bytes up to
//! (not including) its NUL, or up to the bound the C function reads. A NUL
//! inside a slice also ends the string. Every result is NUL-terminated
//! within the destination slice.
//!
//! Characters compare as C `char` (`core::ffi::c_char`): signed on the
//! host, unsigned on the Cortex-M target, as the C implementation did.
//! Memory compares as unsigned bytes.
//!
//! @note Bounded string and memory copies (MISRA C:2012, SQSO-CS-001); no requirement ID defines this

use core::cmp::Ordering;
use core::ffi::c_char;

/// The string in `s`: its bytes before the first NUL
pub fn until_nul(s: &[u8]) -> &[u8] {
    let len = s.iter().position(|b| *b == 0).unwrap_or(s.len());
    s.get(..len).unwrap_or(&[])
}

/// Copy `src` into `dest` and NUL-terminate it, copying at most
/// `dest.len() - 1` bytes and at most `max_chars`. Returns the bytes copied
/// and whether part of `src` did not fit. An empty `dest` gets nothing.
pub fn copy(dest: &mut [u8], src: &[u8], max_chars: usize) -> (usize, bool) {
    let src = until_nul(src);
    let room = dest.len().saturating_sub(1).min(max_chars);
    let n = src.len().min(room);
    if let (Some(d), Some(s)) = (dest.get_mut(..n), src.get(..n)) {
        d.copy_from_slice(s);
    }
    if let Some(end) = dest.get_mut(n) {
        *end = 0;
    }
    (n, src.len() > n)
}

/// Append `src` to the NUL-terminated string in `dest`. Returns the new
/// length and whether part of `src` did not fit, or `None` if `dest` holds
/// no NUL (nothing is changed then).
pub fn concat(dest: &mut [u8], src: &[u8]) -> Option<(usize, bool)> {
    let start = dest.iter().position(|b| *b == 0)?;
    let (copied, truncated) = copy(dest.get_mut(start..)?, src, usize::MAX);
    Some((start.saturating_add(copied), truncated))
}

/// Length of the string in `s`, read up to `max_length` bytes; and whether
/// no NUL was found in that bound
pub fn length(s: &[u8], max_length: usize) -> (usize, bool) {
    let len = until_nul(s).len().min(max_length);
    (len, len == max_length)
}

fn as_c_char(b: u8) -> c_char {
    c_char::from_ne_bytes([b])
}

/// Compare two strings over at most `max_length` characters, as C `char`.
/// Returns -1, 0 or 1, and whether the bound was reached with no
/// difference and no NUL.
pub fn compare(a: &[u8], b: &[u8], max_length: usize) -> (i32, bool) {
    for i in 0..max_length {
        let c1 = a.get(i).copied().unwrap_or(0);
        let c2 = b.get(i).copied().unwrap_or(0);
        if c1 != c2 {
            return (if as_c_char(c1) < as_c_char(c2) { -1 } else { 1 }, false);
        }
        if c1 == 0 {
            return (0, false);
        }
    }
    (0, true)
}

/// Compare two byte buffers as unsigned bytes: -1, 0 or 1
pub fn compare_bytes(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b) {
        match x.cmp(y) {
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
            Ordering::Equal => {}
        }
    }
    0
}

/// Whether `[dest, dest + count)` and `[src, src + count)` overlap
/// (addresses as integers)
pub fn overlaps(dest: usize, src: usize, count: usize) -> bool {
    if dest >= src { dest - src < count } else { src - dest < count }
}

/// Decimal digits of a signed value: the buffer and the length used
pub fn format_i32(value: i32) -> ([u8; 11], usize) {
    let mut out = [0u8; 11];
    let (digits, n) = format_u32(value.unsigned_abs());
    let mut len = 0;
    if value < 0 {
        if let Some(sign) = out.first_mut() {
            *sign = b'-';
        }
        len = 1;
    }
    for (dst, src) in out.iter_mut().skip(len).zip(digits.iter().take(n)) {
        *dst = *src;
    }
    (out, len + n)
}

/// Decimal digits of an unsigned value: the buffer and the length used
pub fn format_u32(mut value: u32) -> ([u8; 10], usize) {
    let mut rev = [0u8; 10];
    let mut n = 0;
    loop {
        if let Some(slot) = rev.get_mut(n) {
            *slot = b'0' + u8::try_from(value % 10).unwrap_or(0);
        }
        n += 1;
        value /= 10;
        if value == 0 || n >= rev.len() {
            break;
        }
    }
    let mut out = [0u8; 10];
    for (dst, src) in out.iter_mut().zip(rev.iter().take(n).rev()) {
        *dst = *src;
    }
    (out, n)
}

/// Write `digits` and a NUL into `buffer`. If they do not fit, `buffer`
/// becomes the empty string and the result is `None`.
pub fn write_digits(buffer: &mut [u8], digits: &[u8]) -> Option<usize> {
    let n = digits.len();
    if n >= buffer.len() {
        if let Some(first) = buffer.first_mut() {
            *first = 0;
        }
        return None;
    }
    if let Some(d) = buffer.get_mut(..n) {
        d.copy_from_slice(digits);
    }
    if let Some(end) = buffer.get_mut(n) {
        *end = 0;
    }
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_truncates_and_terminates() {
        let mut d = [0xAAu8; 8];
        assert_eq!(copy(&mut d, b"Hello, World!", usize::MAX), (7, true));
        assert_eq!(&d, b"Hello, \0");
        let mut d = [0xAAu8; 8];
        assert_eq!(copy(&mut d, b"Hello", usize::MAX), (5, false));
        assert_eq!(&d[..6], b"Hello\0");
        assert_eq!(copy(&mut d, b"", usize::MAX), (0, false));
        assert_eq!(d[0], 0);
        // Exactly fits
        let mut d = [0u8; 6];
        assert_eq!(copy(&mut d, b"Hello", usize::MAX), (5, false));
        // A NUL inside the slice ends the source
        assert_eq!(copy(&mut d, b"ab\0cd", usize::MAX), (2, false));
    }

    #[test]
    fn copy_respects_max_chars() {
        let mut d = [0u8; 32];
        assert_eq!(copy(&mut d, b"Hello, World!", 5).0, 5);
        assert_eq!(&d[..6], b"Hello\0");
        assert_eq!(copy(&mut d, b"Hi", 0).0, 0);
        assert_eq!(d[0], 0);
    }

    #[test]
    fn concat_appends() {
        let mut d = [0u8; 32];
        d[..5].copy_from_slice(b"Hello");
        assert_eq!(concat(&mut d, b", World!"), Some((13, false)));
        assert_eq!(until_nul(&d), b"Hello, World!");

        let mut d = [0u8; 12];
        d[..5].copy_from_slice(b"Hello");
        assert_eq!(concat(&mut d, b", World!"), Some((11, true)));
        assert_eq!(d[11], 0);
        assert_eq!(until_nul(&d), b"Hello, Worl");

        let mut full = [b'x'; 4];
        assert_eq!(concat(&mut full, b"y"), None);
        assert_eq!(&full, b"xxxx");
    }

    #[test]
    fn length_bounded() {
        assert_eq!(length(b"Hello", 100), (5, false));
        assert_eq!(length(b"Hello", 5), (5, true));
        assert_eq!(length(b"", 100), (0, false));
        assert_eq!(length(b"", 0), (0, true));
    }

    #[test]
    fn compare_strings() {
        assert_eq!(compare(b"Hello", b"Hello", 100), (0, false));
        assert_eq!(compare(b"Apple", b"Banana", 100), (-1, false));
        assert_eq!(compare(b"Zebra", b"Apple", 100), (1, false));
        assert_eq!(compare(b"Hello, World", b"Hello, Galaxy", 5), (0, true));
        assert_eq!(compare(b"abc", b"abcd", 100), (-1, false));
        assert_eq!(compare(b"", b"", 0), (0, true));
        // C char signedness decides bytes above 0x7F
        let high = compare(&[0x80], b"a", 4).0;
        assert_eq!(high, if c_char::MIN == 0 { 1 } else { -1 });
    }

    #[test]
    fn compare_memory() {
        assert_eq!(compare_bytes(&[1, 2, 3], &[1, 2, 3]), 0);
        assert_eq!(compare_bytes(&[1, 2, 3], &[1, 2, 4]), -1);
        assert_eq!(compare_bytes(&[0x80], &[0x01]), 1);
        assert_eq!(compare_bytes(&[], &[]), 0);
    }

    #[test]
    fn overlap() {
        assert!(overlaps(105, 100, 10));
        assert!(overlaps(100, 105, 10));
        assert!(!overlaps(110, 100, 10));
        assert!(!overlaps(100, 110, 10));
        assert!(!overlaps(100, 100, 0));
        assert!(overlaps(100, 100, 1));
        assert!(!overlaps(usize::MAX, 0, usize::MAX));
    }

    #[test]
    fn integers() {
        let (d, n) = format_i32(12345);
        assert_eq!(&d[..n], b"12345");
        let (d, n) = format_i32(-12345);
        assert_eq!(&d[..n], b"-12345");
        let (d, n) = format_i32(0);
        assert_eq!(&d[..n], b"0");
        let (d, n) = format_i32(i32::MIN);
        assert_eq!(&d[..n], b"-2147483648");
        let (d, n) = format_u32(u32::MAX);
        assert_eq!(&d[..n], b"4294967295");

        let mut buf = [0xAAu8; 4];
        assert_eq!(write_digits(&mut buf, b"12345"), None);
        assert_eq!(buf[0], 0);
        assert_eq!(write_digits(&mut buf, b"1234"), None);
        assert_eq!(write_digits(&mut buf, b"123"), Some(3));
        assert_eq!(&buf, b"123\0");
    }
}
