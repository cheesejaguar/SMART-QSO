//! GNSS receiver NMEA driver (`gnss.h`, decision D6).
//!
//! The receiver's UART output is one NMEA 0183 sentence per line,
//! `$<body>*<XX>\r\n` with XX the XOR of the body. Two sentences are used:
//!
//! - `$--GGA`: fix quality (field 6). 1 (GPS) and 2 (differential) are real
//!   fixes; 0 is none and 6 is dead reckoning (the piNAV-NG DROP propagator).
//! - `$PSLSP`: piNAV-NG state record: GPS time of week, GPS week, ECEF X, Y,
//!   Z (m, WGS-84), satellites used, PDOP.
//!
//! A `$PSLSP` record becomes the latest fix only if a GGA with a real fix
//! arrived within [`GGA_MAX_AGE_MS`] before it and the record is plausible
//! for LEO, so dead-reckoned output never reaches the ephemeris.
//!
//! [`Receiver`] and [`Fix`] have the C layouts (`GnssRx_t`, `GnssFix_t`):
//! the C caller owns the receiver and `sqso-ffi` works on it in place. The
//! bytes come from outside the flight software; nothing here can index out
//! of bounds or panic (fuzz target `gnss_nmea`).

/// Longest sentence kept, including `$`, `*XX` and CR LF (`GNSS_LINE_MAX`)
pub const LINE_MAX: usize = 96;
/// A `$PSLSP` record needs a real-fix GGA no older than this (`GNSS_GGA_MAX_AGE_MS`)
pub const GGA_MAX_AGE_MS: u32 = 2000;
/// Plausible orbit radius for LEO, lower bound (`GNSS_LEO_MIN_RADIUS_M`)
pub const LEO_MIN_RADIUS_M: f64 = 6.5e6;
/// Plausible orbit radius for LEO, upper bound (`GNSS_LEO_MAX_RADIUS_M`)
pub const LEO_MAX_RADIUS_M: f64 = 7.5e6;
/// Fewest satellites for a 3D fix (`GNSS_MIN_SATS`)
pub const MIN_SATS: u8 = 4;
/// Largest PDOP accepted (`GNSS_MAX_PDOP`; NMEA's own limit, RID P3F-A-05)
pub const MAX_PDOP: f64 = 99.9;

/// GPS time of week wraps at one week
const TOW_LIMIT_S: f64 = 604_800.0;
/// GGA quality: GPS fix and differential fix
const GGA_GPS: u8 = 1;
const GGA_DGPS: u8 = 2;
const MAX_FIELDS: usize = 16;
const GGA_QUALITY_FIELD: usize = 6;

/// Result of a completed line (`GnssStatus_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Status {
    /// Sentence used: GGA recorded or fix accepted
    Ok = 0,
    /// Valid sentence of a type not used
    Ignored = 1,
    /// `$PSLSP` refused: no recent real fix, or implausible
    Rejected = 2,
    /// Not a sentence, overlong, or malformed fields
    ErrFormat = 3,
    /// Checksum mismatch
    ErrChecksum = 4,
}

/// A position record (`GnssFix_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Fix {
    /// The record was accepted
    pub valid: bool,
    /// Time the record was received (caller's clock)
    pub rx_ms: u64,
    /// GPS time of week, s
    pub tow_s: f64,
    /// GPS week as reported
    pub week: u16,
    /// ECEF X, m (WGS-84)
    pub x_m: f64,
    /// ECEF Y, m
    pub y_m: f64,
    /// ECEF Z, m
    pub z_m: f64,
    /// Satellites used
    pub sats: u8,
    /// Position dilution of precision
    pub pdop: f64,
}

impl Fix {
    /// An all-zero record
    pub const ZERO: Fix =
        Fix { valid: false, rx_ms: 0, tow_s: 0.0, week: 0, x_m: 0.0, y_m: 0.0, z_m: 0.0, sats: 0, pdop: 0.0 };
}

/// Receiver state (`GnssRx_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Receiver {
    /// The line being received
    pub line: [u8; LINE_MAX],
    /// Bytes in `line`
    pub len: usize,
    /// The line being received is too long and will be refused
    pub overflow: bool,
    /// A GGA has been recorded
    pub gga_seen: bool,
    /// Quality of the latest GGA
    pub gga_quality: u8,
    /// Time of the latest GGA
    pub gga_ms: u64,
    /// Latest accepted fix
    pub fix: Fix,
    /// `$PSLSP` records refused
    pub rejected: u32,
}

impl Default for Receiver {
    fn default() -> Self {
        Receiver::new()
    }
}

impl Receiver {
    /// A receiver with nothing received (`gnss_rx_init`)
    pub const fn new() -> Receiver {
        Receiver {
            line: [0; LINE_MAX],
            len: 0,
            overflow: false,
            gga_seen: false,
            gga_quality: 0,
            gga_ms: 0,
            fix: Fix::ZERO,
            rejected: 0,
        }
    }

    /// Feed one received byte; the status when it completes a line
    /// (`gnss_rx_feed`). CR is ignored, an empty line completes nothing, and
    /// a line longer than `LINE_MAX - 1` bytes is refused whole at its LF.
    pub fn feed(&mut self, c: u8, now_ms: u64) -> Option<Status> {
        if c == b'\r' {
            return None;
        }
        if c != b'\n' {
            match self.line.get_mut(self.len).filter(|_| self.len < LINE_MAX - 1) {
                Some(slot) => {
                    *slot = c;
                    self.len += 1;
                }
                None => self.overflow = true,
            }
            return None;
        }
        let len = self.len;
        let overflow = self.overflow;
        self.len = 0;
        self.overflow = false;
        if overflow {
            return Some(Status::ErrFormat);
        }
        if len == 0 {
            return None;
        }
        let mut line = [0u8; LINE_MAX];
        let Some(src) = self.line.get(..len) else { return Some(Status::ErrFormat) };
        let Some(dst) = line.get_mut(..len) else { return Some(Status::ErrFormat) };
        dst.copy_from_slice(src);
        Some(self.process_line(dst, now_ms))
    }

    /// One complete line, without its line ending
    fn process_line(&mut self, line: &[u8], now_ms: u64) -> Status {
        // A NUL or a byte XORed in twice would not change the checksum
        if line.iter().any(|b| !(0x20..=0x7E).contains(b)) {
            return Status::ErrFormat;
        }
        let Some(body) = sentence_body(line) else { return Status::ErrFormat };
        if !checksum_ok(line) {
            return Status::ErrChecksum;
        }
        let mut fields: [&[u8]; MAX_FIELDS] = [&[]; MAX_FIELDS];
        let Some(count) = split_fields(body, &mut fields) else { return Status::ErrFormat };
        let fields = fields.get(..count).unwrap_or(&[]);
        match fields {
            [ty, ..] if is_gga(ty) => self.handle_gga(fields, now_ms),
            [ty, ..] if *ty == b"PSLSP" => self.handle_pslsp(fields, now_ms),
            _ => Status::Ignored,
        }
    }

    fn handle_gga(&mut self, fields: &[&[u8]], now_ms: u64) -> Status {
        let Some(quality) = fields.get(GGA_QUALITY_FIELD).and_then(|f| parse_uint(f, 9)) else {
            return Status::ErrFormat;
        };
        self.gga_seen = true;
        self.gga_quality = u8::try_from(quality).unwrap_or(u8::MAX);
        self.gga_ms = now_ms;
        Status::Ok
    }

    fn handle_pslsp(&mut self, fields: &[&[u8]], now_ms: u64) -> Status {
        // The type and exactly 7 values
        let &[_, tow, week, x, y, z, sats, pdop] = fields else { return Status::ErrFormat };
        let (Some(tow_s), Some(week), Some(x_m), Some(y_m), Some(z_m), Some(sats), Some(pdop)) = (
            parse_decimal(tow),
            parse_uint(week, u32::from(u16::MAX)).and_then(|w| u16::try_from(w).ok()),
            parse_decimal(x),
            parse_decimal(y),
            parse_decimal(z),
            parse_uint(sats, u32::from(u8::MAX)).and_then(|s| u8::try_from(s).ok()),
            parse_decimal(pdop),
        ) else {
            return Status::ErrFormat;
        };
        let fix = Fix { valid: false, rx_ms: 0, tow_s, week, x_m, y_m, z_m, sats, pdop };
        if !self.recent_real_fix(now_ms) || !plausible(&fix) {
            self.rejected = self.rejected.wrapping_add(1);
            return Status::Rejected;
        }
        self.fix = Fix { valid: true, rx_ms: now_ms, ..fix };
        Status::Ok
    }

    fn recent_real_fix(&self, now_ms: u64) -> bool {
        self.gga_seen
            && (self.gga_quality == GGA_GPS || self.gga_quality == GGA_DGPS)
            && now_ms >= self.gga_ms
            && now_ms - self.gga_ms <= u64::from(GGA_MAX_AGE_MS)
    }

    /// The latest accepted fix, if no older than `max_age_ms` at `now_ms`
    /// (`gnss_latest_fix`)
    pub fn latest_fix(&self, now_ms: u64, max_age_ms: u32) -> Option<Fix> {
        let fix = self.fix;
        (fix.valid && now_ms >= fix.rx_ms && now_ms - fix.rx_ms <= u64::from(max_age_ms)).then_some(fix)
    }
}

/// The body of `$<body>*XX` (two hex digits), or `None`
fn sentence_body(line: &[u8]) -> Option<&[u8]> {
    match line {
        [b'$', body @ .., b'*', hi, lo] if hex_value(*hi).is_some() && hex_value(*lo).is_some() => Some(body),
        _ => None,
    }
}

/// Check `$<body>*XX` (no line ending) against its checksum
/// (`gnss_nmea_checksum_ok`)
pub fn checksum_ok(line: &[u8]) -> bool {
    let [b'$', body @ .., b'*', hi, lo] = line else { return false };
    let (Some(hi), Some(lo)) = (hex_value(*hi), hex_value(*lo)) else { return false };
    body.iter().fold(0u8, |sum, b| sum ^ b) == (hi << 4) | lo
}

fn hex_value(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'A'..=b'F' => Some(c - b'A' + 10),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

/// Split at commas; the field count, `None` if more than [`MAX_FIELDS`]
fn split_fields<'a>(body: &'a [u8], fields: &mut [&'a [u8]; MAX_FIELDS]) -> Option<usize> {
    let mut n = 0;
    for field in body.split(|b| *b == b',') {
        *fields.get_mut(n)? = field;
        n += 1;
    }
    Some(n)
}

/// `--GGA` from any talker
fn is_gga(ty: &[u8]) -> bool {
    matches!(ty, [_, _, b'G', b'G', b'A'])
}

/// Decimal digits, at most `max`
fn parse_uint(s: &[u8], max: u32) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    s.iter().try_fold(0u32, |value, c| {
        let digit = u32::from(c.checked_sub(b'0').filter(|d| *d <= 9)?);
        value.checked_mul(10)?.checked_add(digit).filter(|v| *v <= max)
    })
}

/// `[-]digits[.digits]`, at least one digit; no exponent, inf or NaN. The
/// arithmetic is the C driver's, in the same order, so values are bit-equal.
fn parse_decimal(s: &[u8]) -> Option<f64> {
    let (negative, mut rest) = match s {
        [b'-', rest @ ..] => (true, rest),
        _ => (false, s),
    };
    let mut digits = false;
    let mut value = 0.0f64;
    while let [c @ b'0'..=b'9', tail @ ..] = rest {
        value = value * 10.0 + f64::from(c - b'0');
        digits = true;
        rest = tail;
    }
    if let [b'.', tail @ ..] = rest {
        rest = tail;
        let mut scale = 1.0f64;
        while let [c @ b'0'..=b'9', tail @ ..] = rest {
            scale /= 10.0;
            value += f64::from(c - b'0') * scale;
            digits = true;
            rest = tail;
        }
    }
    if !digits || !rest.is_empty() {
        return None;
    }
    Some(if negative { -value } else { value })
}

/// Plausible for LEO: time of week in range, a 3D fix, a PDOP above 0 and
/// at most [`MAX_PDOP`], and an orbit radius between [`LEO_MIN_RADIUS_M`]
/// and [`LEO_MAX_RADIUS_M`]
pub fn plausible(f: &Fix) -> bool {
    let r2 = f.x_m * f.x_m + f.y_m * f.y_m + f.z_m * f.z_m;
    f.tow_s >= 0.0
        && f.tow_s < TOW_LIMIT_S
        && f.sats >= MIN_SATS
        && f.pdop > 0.0
        && f.pdop <= MAX_PDOP
        && (LEO_MIN_RADIUS_M * LEO_MIN_RADIUS_M..=LEO_MAX_RADIUS_M * LEO_MAX_RADIUS_M).contains(&r2)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GGA_REF: &[u8] = b"$GPGGA,123519,4807.038,N,01131.000,E,1,08,0.9,545.4,M,46.9,M,,*47\r\n";
    const PSLSP_REF: &[u8] = b"$PSLSP,3061.1001020,801,-6602427.515,97187.285,-1721790.821,7,2.6*7F\r\n";

    fn feed(rx: &mut Receiver, s: &[u8], now: u64) -> Option<Status> {
        let mut last = None;
        for b in s {
            if let Some(st) = rx.feed(*b, now) {
                last = Some(st);
            }
        }
        last
    }

    fn sentence(body: &str) -> [u8; 128] {
        let sum = body.bytes().fold(0u8, |s, b| s ^ b);
        let mut out = [0u8; 128];
        let hex = b"0123456789ABCDEF";
        for (n, b) in core::iter::once(b'$')
            .chain(body.bytes())
            .chain([b'*', hex[usize::from(sum >> 4)], hex[usize::from(sum & 15)], b'\r', b'\n'])
            .enumerate()
        {
            out[n] = b;
        }
        out
    }

    fn until_nul(b: &[u8]) -> &[u8] {
        &b[..b.iter().position(|c| *c == 0).unwrap_or(b.len())]
    }

    #[test]
    fn reference_sentences_give_a_fix() {
        let mut rx = Receiver::new();
        assert!(checksum_ok(&GGA_REF[..GGA_REF.len() - 2]));
        assert!(checksum_ok(&PSLSP_REF[..PSLSP_REF.len() - 2]));
        assert_eq!(feed(&mut rx, GGA_REF, 1000), Some(Status::Ok));
        assert_eq!(feed(&mut rx, PSLSP_REF, 1200), Some(Status::Ok));
        let fix = rx.latest_fix(1500, 60_000).unwrap();
        assert_eq!(fix.week, 801);
        assert_eq!(fix.sats, 7);
        assert_eq!(fix.rx_ms, 1200);
        assert!((fix.x_m - -6_602_427.515).abs() < 1e-3);
        assert!((fix.tow_s - 3_061.100_102).abs() < 1e-6);
        assert!(rx.latest_fix(61_201, 60_000).is_none());
        assert!(rx.latest_fix(1199, 60_000).is_none());
    }

    #[test]
    fn record_needs_recent_real_fix() {
        let mut rx = Receiver::new();
        assert_eq!(feed(&mut rx, PSLSP_REF, 1000), Some(Status::Rejected));
        let dead_reckoning = sentence("GPGGA,123519,4807.038,N,01131.000,E,6,08,0.9,545.4,M,46.9,M,,");
        assert_eq!(feed(&mut rx, until_nul(&dead_reckoning), 2000), Some(Status::Ok));
        assert_eq!(feed(&mut rx, PSLSP_REF, 2100), Some(Status::Rejected));
        assert_eq!(rx.rejected, 2);
        assert_eq!(feed(&mut rx, GGA_REF, 3000), Some(Status::Ok));
        assert_eq!(feed(&mut rx, PSLSP_REF, 3000 + u64::from(GGA_MAX_AGE_MS)), Some(Status::Ok));
        assert_eq!(feed(&mut rx, PSLSP_REF, 3001 + u64::from(GGA_MAX_AGE_MS)), Some(Status::Rejected));
    }

    #[test]
    fn framing() {
        let mut rx = Receiver::new();
        assert_eq!(feed(&mut rx, b"$GPGGA,123519\r\n", 0), Some(Status::ErrFormat));
        assert_eq!(feed(&mut rx, b"\r\n\n", 0), None);
        let mut bad = [0u8; 80];
        bad[..PSLSP_REF.len()].copy_from_slice(PSLSP_REF);
        bad[10] = b'9';
        assert_eq!(feed(&mut rx, until_nul(&bad), 0), Some(Status::ErrChecksum));
        for _ in 0..LINE_MAX + 40 {
            assert_eq!(rx.feed(b'A', 0), None);
        }
        assert_eq!(rx.feed(b'\n', 0), Some(Status::ErrFormat));
        assert_eq!(feed(&mut rx, GGA_REF, 0), Some(Status::Ok));
        let gsv = sentence("GPGSV,3,1,11,03,03,111,00");
        assert_eq!(feed(&mut rx, until_nul(&gsv), 0), Some(Status::Ignored));
        let many = sentence("GPGSV,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16");
        assert_eq!(feed(&mut rx, until_nul(&many), 0), Some(Status::ErrFormat));
        let sixteen = sentence("GPGSV,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15");
        assert_eq!(feed(&mut rx, until_nul(&sixteen), 0), Some(Status::Ignored));
    }

    #[test]
    fn numbers() {
        assert_eq!(parse_uint(b"", 9), None);
        assert_eq!(parse_uint(b"9", 9), Some(9));
        assert_eq!(parse_uint(b"10", 9), None);
        assert_eq!(parse_uint(b"65535", 65535), Some(65535));
        assert_eq!(parse_uint(b"65536", 65535), None);
        assert_eq!(parse_uint(b"99999999999", u32::MAX), None);
        assert_eq!(parse_decimal(b"-.5"), Some(-0.5));
        assert_eq!(parse_decimal(b"5."), Some(5.0));
        assert_eq!(parse_decimal(b"-"), None);
        assert_eq!(parse_decimal(b"."), None);
        assert_eq!(parse_decimal(b"+1"), None);
        assert_eq!(parse_decimal(b"1e3"), None);
        assert_eq!(parse_decimal(b"2.6"), Some(2.0 + 6.0 * (1.0 / 10.0)));
    }

    #[test]
    fn implausible_records_refused() {
        let base =
            Fix { valid: false, rx_ms: 0, tow_s: 10.0, week: 1, x_m: 6.9e6, y_m: 0.0, z_m: 0.0, sats: 7, pdop: 1.0 };
        assert!(plausible(&base));
        assert!(!plausible(&Fix { sats: 3, ..base }));
        assert!(!plausible(&Fix { pdop: 0.0, ..base }));
        assert!(!plausible(&Fix { x_m: 6.378e6, ..base }));
        assert!(!plausible(&Fix { x_m: 4.2e7, ..base }));
        assert!(!plausible(&Fix { tow_s: TOW_LIMIT_S, ..base }));
        assert!(!plausible(&Fix { tow_s: -1.0, ..base }));
        assert!(!plausible(&Fix { pdop: f64::NAN, ..base }));
        assert!(plausible(&Fix { pdop: MAX_PDOP, ..base }));
        assert!(!plausible(&Fix { pdop: 100.0, ..base }), "P3F-A-05: PDOP bounded");
    }

    /// Mutation check: a line of LINE_MAX - 1 bytes is kept, one byte more
    /// is refused whole
    #[test]
    fn longest_line() {
        let mut rx = Receiver::new();
        // "$" body "*XX": a body of LINE_MAX - 5 bytes makes LINE_MAX - 1
        let mut body = [b'A'; LINE_MAX - 4];
        body[..6].copy_from_slice(b"GPTXT,");
        let ok = sentence(core::str::from_utf8(&body[..LINE_MAX - 5]).unwrap());
        assert_eq!(until_nul(&ok).len(), LINE_MAX - 1 + 2);
        assert_eq!(feed(&mut rx, until_nul(&ok), 0), Some(Status::Ignored));
        let long = sentence(core::str::from_utf8(&body).unwrap());
        assert_eq!(feed(&mut rx, until_nul(&long), 0), Some(Status::ErrFormat));
    }

    /// Mutation check: the checksum digits must both be hex (else a format
    /// error, not a checksum error), and lowercase hex is accepted
    #[test]
    fn checksum_digits() {
        let mut rx = Receiver::new();
        assert_eq!(feed(&mut rx, b"$GPTXT*G1\r\n", 0), Some(Status::ErrFormat));
        assert_eq!(feed(&mut rx, b"$GPTXT*4G\r\n", 0), Some(Status::ErrFormat));
        assert_eq!(feed(&mut rx, b"$GPTXT*00\r\n", 0), Some(Status::ErrChecksum));
        // "GPTXp": the low nibble of its checksum is a letter
        let sum = b"GPTXp".iter().fold(0u8, |a, b| a ^ b);
        assert!(sum & 0x0F >= 10);
        let hex = |d: u8, digits: &[u8; 16]| digits[usize::from(d)];
        for digits in [b"0123456789ABCDEF", b"0123456789abcdef"] {
            let line = [b'$', b'G', b'P', b'T', b'X', b'p', b'*', hex(sum >> 4, digits), hex(sum & 15, digits)];
            assert!(checksum_ok(&line), "{}", core::str::from_utf8(&line).unwrap());
            let mut framed = [0u8; 11];
            framed[..9].copy_from_slice(&line);
            framed[9..].copy_from_slice(b"\r\n");
            assert_eq!(feed(&mut rx, &framed, 0), Some(Status::Ignored));
        }
        for (c, v) in [(b'a', 10), (b'f', 15), (b'A', 10), (b'F', 15), (b'0', 0), (b'9', 9)] {
            assert_eq!(hex_value(c), Some(v));
        }
        assert_eq!(hex_value(b'g'), None);
    }

    #[test]
    fn fix_age_is_measured_from_reception() {
        let mut rx = Receiver::new();
        assert_eq!(feed(&mut rx, GGA_REF, 1_000_000), Some(Status::Ok));
        assert_eq!(feed(&mut rx, PSLSP_REF, 1_000_100), Some(Status::Ok));
        assert!(rx.latest_fix(1_001_100, 1000).is_some());
        assert!(rx.latest_fix(1_001_101, 1000).is_none());
    }

    #[test]
    fn radius_uses_every_axis() {
        let base =
            Fix { valid: false, rx_ms: 0, tow_s: 10.0, week: 1, x_m: 0.0, y_m: 0.0, z_m: 0.0, sats: 7, pdop: 1.0 };
        assert!(plausible(&Fix { y_m: 6.9e6, ..base }));
        assert!(plausible(&Fix { z_m: -6.9e6, ..base }));
        assert!(plausible(&Fix { x_m: 4.5e6, y_m: 4.5e6, z_m: 2.5e6, ..base }));
        assert!(!plausible(&Fix { x_m: 5.0e6, y_m: 5.0e6, z_m: 5.0e6, ..base }), "8.7e6 m");
    }
}
