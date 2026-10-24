//! OBC-Jetson line protocol (`docs/interfaces/ICD_OBC_JETSON.md` §4, Rev 1.1).
//!
//! Every message is one line of printable ASCII (0x20-0x7E):
//!
//! ```text
//! <TYPE>,<fields...>,<CRC>\n
//! ```
//!
//! CRC is four upper-case hex digits of CRC-16/CCITT-FALSE over every byte
//! before the last comma. A BEACON_RSP text may contain commas: it runs from
//! after the sequence number to the last comma.
//!
//! - OBC to Jetson: TELEMETRY, BEACON_REQ, STATUS_REQ, SHUTDOWN
//! - Jetson to OBC: BEACON_RSP, STATUS, ACK, NACK
//!
//! Both directions are implemented here, so the OBC (through `sqso-ffi`) and
//! the Jetson (through the `sqso_protocol` Python module) run the same code.

use core::fmt::Write as _;

use crate::checksum::crc16_ccitt_false;
use crate::write::SliceWriter;

/// Longest line, including the CRC field and the newline
pub const LINE_MAX: usize = 512;
/// Longest beacon text the OBC accepts (the beacon's free-text space)
pub const TEXT_MAX: usize = 176;
/// ",XXXX"
const CRC_FIELD: usize = 5;
/// ",XXXX\n"
const SEAL_LEN: usize = CRC_FIELD + 1;

const HEX: &[u8; 16] = b"0123456789ABCDEF";
/// [`TEXT_MAX`] as a field limit
const TEXT_MAX_FIELD: u32 = 176;
const _: () = assert!(TEXT_MAX_FIELD as usize == TEXT_MAX);

/// Why a line was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkError {
    /// Not a well-formed line, or a field out of range
    Format,
    /// CRC mismatch
    Crc,
    /// A message type not expected in this direction
    Type,
}

fn printable(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| (0x20..=0x7E).contains(b))
}

/// Append `,<CRC>\n` to the `len`-byte message at the start of `buf` (and a
/// NUL if there is room). Returns the line length, or `None` if the message
/// is empty, not printable, or the line would not fit in `buf` or
/// [`LINE_MAX`].
pub fn seal_in_place(buf: &mut [u8], len: usize) -> Option<usize> {
    let line_len = len.checked_add(SEAL_LEN)?;
    if len == 0 || line_len > buf.len() || line_len > LINE_MAX {
        return None;
    }
    let (body, tail) = buf.split_at_mut(len);
    if !printable(body) {
        return None;
    }
    let crc = crc16_ccitt_false(body);
    let digit = |shift: u16| HEX.get(usize::from((crc >> shift) & 0xF)).copied().unwrap_or(b'0');
    let seal = [b',', digit(12), digit(8), digit(4), digit(0), b'\n'];
    tail.get_mut(..SEAL_LEN)?.copy_from_slice(&seal);
    if let Some(nul) = tail.get_mut(SEAL_LEN) {
        *nul = 0;
    }
    Some(line_len)
}

/// Write `body` and its seal into `out`; returns the line length
pub fn seal(body: &[u8], out: &mut [u8]) -> Result<usize, LinkError> {
    out.get_mut(..body.len()).ok_or(LinkError::Format)?.copy_from_slice(body);
    seal_in_place(out, body.len()).ok_or(LinkError::Format)
}

/// Format a message body into `out` and seal it
fn build(out: &mut [u8], args: core::fmt::Arguments<'_>) -> Result<usize, LinkError> {
    let mut w = SliceWriter::new(out);
    w.write_fmt(args).map_err(|_| LinkError::Format)?;
    let len = w.len();
    seal_in_place(out, len).ok_or(LinkError::Format)
}

/// Check framing and CRC of one received line (one trailing newline is
/// allowed); returns the body, the bytes before `,<CRC>`
pub fn check(line: &[u8]) -> Result<&[u8], LinkError> {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    if line.len() < SEAL_LEN || line.len() >= LINE_MAX || !printable(line) {
        return Err(LinkError::Format);
    }
    let (body, crc_field) = line.split_at(line.len() - CRC_FIELD);
    let [b',', digits @ ..] = crc_field else {
        return Err(LinkError::Format);
    };
    let mut received: u16 = 0;
    for d in digits {
        let value = match d {
            b'0'..=b'9' => d - b'0',
            b'A'..=b'F' => d - b'A' + 10,
            _ => return Err(LinkError::Format),
        };
        received = (received << 4) | u16::from(value);
    }
    if crc16_ccitt_false(body) != received {
        return Err(LinkError::Crc);
    }
    Ok(body)
}

/// Split a checked body at its first comma: type and the rest (`None` if
/// there is no comma)
pub fn split_type(body: &[u8]) -> (&[u8], Option<&[u8]>) {
    match body.iter().position(|b| *b == b',') {
        Some(i) => {
            let (ty, rest) = body.split_at(i);
            (ty, rest.get(1..))
        }
        None => (body, None),
    }
}

/// Unsigned decimal of at most `max_digits` digits and no more than `max`
fn parse_uint(field: &[u8], max: u64, max_digits: usize) -> Result<u64, LinkError> {
    if field.is_empty() || field.len() > max_digits {
        return Err(LinkError::Format);
    }
    field.iter().try_fold(0u64, |v, d| {
        let digit = match d {
            b'0'..=b'9' => u64::from(d - b'0'),
            _ => return Err(LinkError::Format),
        };
        v.checked_mul(10).and_then(|v| v.checked_add(digit)).filter(|v| *v <= max).ok_or(LinkError::Format)
    })
}

fn parse_u32(field: &[u8], max: u32) -> Result<u32, LinkError> {
    let value = parse_uint(field, u64::from(max), 10)?;
    u32::try_from(value).map_err(|_| LinkError::Format)
}

fn parse_i32(field: &[u8], min: i32, max: i32) -> Result<i32, LinkError> {
    match field.strip_prefix(b"-") {
        Some(magnitude) => {
            let m = parse_uint(magnitude, u64::from(min.unsigned_abs()), 10)?;
            i32::try_from(-i64::try_from(m).map_err(|_| LinkError::Format)?).map_err(|_| LinkError::Format)
        }
        None => {
            let m = parse_uint(field, u64::try_from(max).map_err(|_| LinkError::Format)?, 10)?;
            i32::try_from(m).map_err(|_| LinkError::Format)
        }
    }
}

fn parse_f64(field: &[u8]) -> Result<f64, LinkError> {
    let text = core::str::from_utf8(field).map_err(|_| LinkError::Format)?;
    text.parse::<f64>().ok().filter(|v| v.is_finite()).ok_or(LinkError::Format)
}

fn ascii(bytes: &[u8]) -> Result<&str, LinkError> {
    core::str::from_utf8(bytes).map_err(|_| LinkError::Format)
}

/// Exactly `N` comma-separated fields
fn fields<const N: usize>(rest: &[u8]) -> Result<[&[u8]; N], LinkError> {
    let mut out: [&[u8]; N] = [&[]; N];
    let mut parts = rest.split(|b| *b == b',');
    for slot in out.iter_mut() {
        *slot = parts.next().ok_or(LinkError::Format)?;
    }
    match parts.next() {
        None => Ok(out),
        Some(_) => Err(LinkError::Format),
    }
}

/// A one-field ACK/NACK text: 1 to [`TEXT_MAX`] characters, no commas
fn single_field(rest: Option<&[u8]>) -> Result<&str, LinkError> {
    let rest = rest.ok_or(LinkError::Format)?;
    if rest.is_empty() || rest.len() > TEXT_MAX || rest.contains(&b',') {
        return Err(LinkError::Format);
    }
    ascii(rest)
}

/// Jetson health report
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Status {
    /// 0 OK .. 3 CRITICAL
    pub health: u8,
    /// Module temperature, -128 to 200 degrees C
    pub temp_c: i32,
    /// Input power, 0 to 100000 mW
    pub power_mw: u32,
    /// Memory used, 0 to 100 %
    pub mem_pct: u8,
    /// Uptime, seconds
    pub uptime_s: u32,
}

/// A message from the Jetson to the OBC
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromJetson<'a> {
    /// `BEACON_RSP,<seq>,<text>`
    BeaconRsp {
        /// Sequence number of the request answered
        seq: u16,
        /// Proposed beacon text, 1 to [`TEXT_MAX`] characters (still to be
        /// content-filtered, [`crate::filter`])
        text: &'a str,
    },
    /// `STATUS,<health>,<temp_c>,<power_mw>,<mem_pct>,<uptime_s>`
    Status(Status),
    /// `ACK,<what>`
    Ack(&'a str),
    /// `NACK,<reason>`
    Nack(&'a str),
}

/// Check and parse a line the OBC received from the Jetson
pub fn parse_from_jetson(line: &[u8]) -> Result<FromJetson<'_>, LinkError> {
    let (ty, rest) = split_type(check(line)?);
    match ty {
        b"BEACON_RSP" => {
            let rest = rest.ok_or(LinkError::Format)?;
            let (seq, text) = split_type(rest);
            let text = text.ok_or(LinkError::Format)?;
            let seq = parse_u32(seq, u32::from(u16::MAX))?;
            if text.is_empty() || text.len() > TEXT_MAX {
                return Err(LinkError::Format);
            }
            Ok(FromJetson::BeaconRsp { seq: u16::try_from(seq).map_err(|_| LinkError::Format)?, text: ascii(text)? })
        }
        b"STATUS" => {
            let rest = rest.filter(|r| !r.is_empty()).ok_or(LinkError::Format)?;
            let [health, temp, power, mem, uptime] = fields::<5>(rest)?;
            Ok(FromJetson::Status(Status {
                health: u8::try_from(parse_u32(health, 3)?).map_err(|_| LinkError::Format)?,
                temp_c: parse_i32(temp, -128, 200)?,
                power_mw: parse_u32(power, 100_000)?,
                mem_pct: u8::try_from(parse_u32(mem, 100)?).map_err(|_| LinkError::Format)?,
                uptime_s: parse_u32(uptime, u32::MAX)?,
            }))
        }
        b"ACK" => Ok(FromJetson::Ack(single_field(rest)?)),
        b"NACK" => Ok(FromJetson::Nack(single_field(rest)?)),
        _ => Err(LinkError::Type),
    }
}

/// OBC telemetry for the Jetson
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Telemetry<'a> {
    /// Mission time, ms
    pub time_ms: u64,
    /// SUNLIT (true) or ECLIPSE
    pub sunlit: bool,
    /// Battery state of charge, 0 to 1
    pub soc: f64,
    /// EPS power mode, 0 SAFE, 1 IDLE, 2 ACTIVE
    pub power_mode: u8,
    /// Payload power limit, W
    pub power_limit_w: f64,
    /// The remaining `key=value` fields, unparsed
    pub extras: &'a str,
}

impl<'a> Telemetry<'a> {
    /// The `key=value` extras (fields without `=` or with an empty key are
    /// skipped)
    pub fn extras(&self) -> impl Iterator<Item = (&'a str, &'a str)> + 'a {
        self.extras.split(',').filter_map(|item| item.split_once('=')).filter(|(key, _)| !key.is_empty())
    }
}

/// A message from the OBC to the Jetson
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FromObc<'a> {
    /// `TELEMETRY,<time_ms>,<SUNLIT|ECLIPSE>,<soc>,<mode>,<limit_w>,<key=value>...`
    Telemetry(Telemetry<'a>),
    /// `BEACON_REQ,<seq>,<max_len>`
    BeaconReq {
        /// Sequence number to echo in the answer
        seq: u16,
        /// Longest text wanted, 1 to [`TEXT_MAX`]
        max_len: u8,
    },
    /// `STATUS_REQ`
    StatusReq,
    /// `SHUTDOWN,<reason>`
    Shutdown(&'a str),
}

/// Check and parse a line the Jetson received from the OBC
pub fn parse_from_obc(line: &[u8]) -> Result<FromObc<'_>, LinkError> {
    let (ty, rest) = split_type(check(line)?);
    match ty {
        b"TELEMETRY" => {
            let rest = rest.ok_or(LinkError::Format)?;
            let mut parts = rest.splitn(6, |b| *b == b',');
            let mut next = || parts.next().ok_or(LinkError::Format);
            let (time_ms, sun, soc, mode, limit) = (next()?, next()?, next()?, next()?, next()?);
            let extras = parts.next().unwrap_or(&[]);
            let sunlit = match sun {
                b"SUNLIT" => true,
                b"ECLIPSE" => false,
                _ => return Err(LinkError::Format),
            };
            Ok(FromObc::Telemetry(Telemetry {
                time_ms: parse_uint(time_ms, u64::MAX, 20)?,
                sunlit,
                soc: parse_f64(soc)?,
                power_mode: u8::try_from(parse_u32(mode, 2)?).map_err(|_| LinkError::Format)?,
                power_limit_w: parse_f64(limit)?,
                extras: ascii(extras)?,
            }))
        }
        b"BEACON_REQ" => {
            let [seq, max_len] = fields::<2>(rest.ok_or(LinkError::Format)?)?;
            let seq = u16::try_from(parse_u32(seq, u32::from(u16::MAX))?).map_err(|_| LinkError::Format)?;
            let max_len = parse_u32(max_len, TEXT_MAX_FIELD)?;
            if max_len == 0 {
                return Err(LinkError::Format);
            }
            Ok(FromObc::BeaconReq { seq, max_len: u8::try_from(max_len).map_err(|_| LinkError::Format)? })
        }
        b"STATUS_REQ" if rest.is_none() => Ok(FromObc::StatusReq),
        b"SHUTDOWN" => Ok(FromObc::Shutdown(single_field(rest)?)),
        b"STATUS_REQ" => Err(LinkError::Format),
        _ => Err(LinkError::Type),
    }
}

/// `BEACON_REQ,<seq>,<max_len>` (max_len 1 to [`TEXT_MAX`])
pub fn beacon_req(seq: u16, max_len: u8, out: &mut [u8]) -> Result<usize, LinkError> {
    if max_len == 0 || usize::from(max_len) > TEXT_MAX {
        return Err(LinkError::Format);
    }
    build(out, format_args!("BEACON_REQ,{seq},{max_len}"))
}

/// `STATUS_REQ`
pub fn status_req(out: &mut [u8]) -> Result<usize, LinkError> {
    build(out, format_args!("STATUS_REQ"))
}

/// `SHUTDOWN,<reason>` (one field: no commas)
pub fn shutdown(reason: &str, out: &mut [u8]) -> Result<usize, LinkError> {
    single_field(Some(reason.as_bytes()))?;
    build(out, format_args!("SHUTDOWN,{reason}"))
}

/// `BEACON_RSP,<seq>,<text>` (text: 1 to [`TEXT_MAX`] printable characters)
pub fn beacon_rsp(seq: u16, text: &str, out: &mut [u8]) -> Result<usize, LinkError> {
    if text.is_empty() || text.len() > TEXT_MAX {
        return Err(LinkError::Format);
    }
    build(out, format_args!("BEACON_RSP,{seq},{text}"))
}

/// `STATUS,<health>,<temp_c>,<power_mw>,<mem_pct>,<uptime_s>`, fields in range
pub fn status(s: &Status, out: &mut [u8]) -> Result<usize, LinkError> {
    if s.health > 3 || !(-128..=200).contains(&s.temp_c) || s.power_mw > 100_000 || s.mem_pct > 100 {
        return Err(LinkError::Format);
    }
    build(out, format_args!("STATUS,{},{},{},{},{}", s.health, s.temp_c, s.power_mw, s.mem_pct, s.uptime_s))
}

/// `ACK,<what>` (one field: no commas)
pub fn ack(what: &str, out: &mut [u8]) -> Result<usize, LinkError> {
    single_field(Some(what.as_bytes()))?;
    build(out, format_args!("ACK,{what}"))
}

/// `NACK,<reason>` (one field: no commas)
pub fn nack(reason: &str, out: &mut [u8]) -> Result<usize, LinkError> {
    single_field(Some(reason.as_bytes()))?;
    build(out, format_args!("NACK,{reason}"))
}

/// Receive accumulator: bytes in, complete lines out. `repr(C)` so the C
/// `JetsonRx_t` (same fields, same order) can hold one.
#[repr(C)]
pub struct LineRx {
    line: [u8; LINE_MAX],
    len: usize,
    overflow: bool,
}

impl LineRx {
    /// An empty accumulator
    pub const fn new() -> Self {
        LineRx { line: [0; LINE_MAX], len: 0, overflow: false }
    }

    /// Forget any partial line
    pub fn reset(&mut self) {
        self.len = 0;
        self.overflow = false;
    }

    /// Feed one byte. At a newline, returns the line received (without the
    /// newline), or `Err(Format)` once for a line longer than the buffer,
    /// which is dropped up to its newline.
    pub fn feed(&mut self, byte: u8) -> Option<Result<&[u8], LinkError>> {
        if byte == b'\n' {
            let len = core::mem::take(&mut self.len);
            if core::mem::take(&mut self.overflow) {
                return Some(Err(LinkError::Format));
            }
            return Some(self.line.get(..len).ok_or(LinkError::Format));
        }
        if self.overflow {
            return None;
        }
        match self.line.get_mut(self.len) {
            Some(slot) if self.len < LINE_MAX - 1 => {
                *slot = byte;
                self.len += 1;
            }
            _ => self.overflow = true,
        }
        None
    }
}

impl Default for LineRx {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(f: impl FnOnce(&mut [u8]) -> Result<usize, LinkError>) -> ([u8; LINE_MAX], usize) {
        let mut buf = [0u8; LINE_MAX];
        let n = f(&mut buf).unwrap();
        (buf, n)
    }

    #[test]
    fn golden_lines() {
        // The same lines as tests/test_jetson_link.c and payload/tests/test_obc_link.py
        let (b, n) = line(status_req);
        let text = core::str::from_utf8(&b[..n]).unwrap();
        assert!(text.starts_with("STATUS_REQ,") && text.ends_with('\n'));
        assert_eq!(check(&b[..n]), Ok(&b"STATUS_REQ"[..]));
        let (b, n) = line(|o| seal(b"123456789", o));
        assert_eq!(&b[..n], b"123456789,29B1\n");
    }

    #[test]
    fn both_directions_round_trip() {
        let (b, n) = line(|o| beacon_req(42, 176, o));
        assert_eq!(parse_from_obc(&b[..n]), Ok(FromObc::BeaconReq { seq: 42, max_len: 176 }));
        let (b, n) = line(|o| beacon_rsp(42, "Hello, world from orbit", o));
        assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::BeaconRsp { seq: 42, text: "Hello, world from orbit" }));
        let s = Status { health: 1, temp_c: -12, power_mw: 4500, mem_pct: 40, uptime_s: 99 };
        let (b, n) = line(|o| status(&s, o));
        assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::Status(s)));
        let (b, n) = line(|o| ack("TELEMETRY", o));
        assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::Ack("TELEMETRY")));
        let (b, n) = line(|o| shutdown("POWER", o));
        assert_eq!(parse_from_obc(&b[..n]), Ok(FromObc::Shutdown("POWER")));
    }

    #[test]
    fn telemetry() {
        let (b, n) = line(|o| seal(b"TELEMETRY,1234,SUNLIT,0.812,2,4.5,BV=7.4,JT=41,bad,=x", o));
        let Ok(FromObc::Telemetry(t)) = parse_from_obc(&b[..n]) else { panic!() };
        assert_eq!((t.time_ms, t.sunlit, t.power_mode), (1234, true, 2));
        let mut extras = t.extras();
        assert_eq!(extras.next(), Some(("BV", "7.4")));
        assert_eq!(extras.next(), Some(("JT", "41")));
        assert_eq!(extras.next(), None);
        let (b, n) = line(|o| seal(b"TELEMETRY,1,DUSK,0.5,1,4.5", o));
        assert_eq!(parse_from_obc(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"TELEMETRY,1,ECLIPSE,nan,1,4.5", o));
        assert_eq!(parse_from_obc(&b[..n]), Err(LinkError::Format));
    }

    #[test]
    fn rejections() {
        assert_eq!(check(b"STATUS_REQ,0000\n"), Err(LinkError::Crc));
        assert_eq!(check(b"STATUS_REQ,00g0"), Err(LinkError::Format));
        assert_eq!(check(b"STATUS_REQ;0000"), Err(LinkError::Format));
        assert_eq!(check(b",0000"), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"STATUS,4,0,0,0,0", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"STATUS,0,-129,0,0,0", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"STATUS,0,-128,0,0,4294967295", o));
        assert!(parse_from_jetson(&b[..n]).is_ok());
        let (b, n) = line(|o| seal(b"STATUS,0,0,0,0,4294967296", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"ACK,a,b", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"TELEMETRY,1", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Type));
        assert_eq!(beacon_req(1, 0, &mut [0u8; 64]), Err(LinkError::Format));
        assert_eq!(ack("a,b", &mut [0u8; 64]), Err(LinkError::Format));
        assert_eq!(seal(b"bad\x01", &mut [0u8; 64]), Err(LinkError::Format));
        assert_eq!(seal(b"x", &mut [0u8; 7]), Ok(7));
        assert_eq!(seal(b"x", &mut [0u8; 6]), Err(LinkError::Format));
    }

    #[test]
    fn rx_accumulator() {
        let mut rx = LineRx::new();
        let (b, n) = line(|o| ack("X", o));
        let mut got = None;
        for byte in &b[..n] {
            if let Some(l) = rx.feed(*byte) {
                got = Some(l.map(|l| parse_from_jetson(l).is_ok()));
            }
        }
        assert_eq!(got, Some(Ok(true)));
        for _ in 0..600 {
            assert!(rx.feed(b'A').is_none());
        }
        assert_eq!(rx.feed(b'\n'), Some(Err(LinkError::Format)));
        assert_eq!(rx.feed(b'\n'), Some(Ok(&b""[..])));
    }

    /// `body,<CRC>\n` without any of the builders' checks
    fn raw_line(body: &[u8], out: &mut [u8]) -> usize {
        let crc = crc16_ccitt_false(body);
        let n = body.len();
        out[..n].copy_from_slice(body);
        out[n] = b',';
        for (i, shift) in [12u16, 8, 4, 0].into_iter().enumerate() {
            out[n + 1 + i] = HEX[usize::from((crc >> shift) & 0xF)];
        }
        out[n + CRC_FIELD] = b'\n';
        n + SEAL_LEN
    }

    #[test]
    fn icd_golden_lines() {
        // ICD_OBC_JETSON.md §4.3
        let golden = |expected: &[u8], (b, n): ([u8; LINE_MAX], usize)| assert_eq!(&b[..n], expected);
        golden(
            b"TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON,9C86\n",
            line(|o| seal(b"TELEMETRY,1704153600000,SUNLIT,0.750,2,3.0,BV=8.1,EPS_PAYLOAD=ON", o)),
        );
        golden(b"BEACON_REQ,42,176,5593\n", line(|o| beacon_req(42, 176, o)));
        golden(b"BEACON_RSP,42,Hello, world from orbit,0784\n", line(|o| beacon_rsp(42, "Hello, world from orbit", o)));
        let s = Status { health: 0, temp_c: 45, power_mw: 6800, mem_pct: 37, uptime_s: 1234 };
        golden(b"STATUS,0,45,6800,37,1234,D775\n", line(|o| status(&s, o)));
        golden(b"STATUS_REQ,5D03\n", line(status_req));
        golden(b"SHUTDOWN,POWER,05D4\n", line(|o| shutdown("POWER", o)));
        golden(b"ACK,BEACON_RSP,275C\n", line(|o| ack("BEACON_RSP", o)));
        golden(b"NACK,CRC,49EE\n", line(|o| nack("CRC", o)));
        assert_eq!(parse_from_obc(b"STATUS_REQ,5D03\n"), Ok(FromObc::StatusReq));
        assert_eq!(parse_from_jetson(b"NACK,CRC,49EE\n"), Ok(FromJetson::Nack("CRC")));
    }

    #[test]
    fn line_length_limit() {
        // At most 512 bytes, including the CRC field and the LF
        let body_max = LINE_MAX - SEAL_LEN;
        let mut buf = [b'A'; LINE_MAX + 8];
        assert_eq!(seal_in_place(&mut buf, body_max), Some(LINE_MAX));
        assert_eq!(check(&buf[..LINE_MAX]).map(<[u8]>::len), Ok(body_max));
        let mut buf = [b'A'; LINE_MAX + 8];
        assert_eq!(seal_in_place(&mut buf, body_max + 1), None);
        let n = raw_line(&[b'A'; LINE_MAX - SEAL_LEN + 1], &mut buf);
        assert_eq!(n, LINE_MAX + 1);
        assert_eq!(check(&buf[..n]), Err(LinkError::Format));
        assert_eq!(check(&buf[..n - 1]), Err(LinkError::Format)); // without the LF
    }

    #[test]
    fn seal_refuses_empty_and_out_of_buffer_messages() {
        assert_eq!(seal_in_place(&mut [b'A'; 16], 0), None);
        assert_eq!(seal_in_place(&mut [b'A'; 4], 10), None);
        assert_eq!(seal_in_place(&mut [b'A'; 16], 10), Some(16));
    }

    #[test]
    fn shortest_line_has_a_one_byte_body() {
        let (b, n) = line(|o| seal(b"x", o));
        assert_eq!(n, 7);
        assert_eq!(check(&b[..n]), Ok(&b"x"[..]));
        assert_eq!(check(&b[..n - 1]), Ok(&b"x"[..]));
    }

    #[test]
    fn non_printable_line_is_malformed() {
        let mut buf = [0u8; 32];
        let n = raw_line(b"ACK,A\x01B", &mut buf);
        assert_eq!(check(&buf[..n]), Err(LinkError::Format));
        let n = raw_line(b"ACK,\xC3\xA9", &mut buf);
        assert_eq!(check(&buf[..n]), Err(LinkError::Format));
        let n = raw_line(b"ACK,AB", &mut buf);
        assert_eq!(check(&buf[..n]), Ok(&b"ACK,AB"[..]));
    }

    #[test]
    fn empty_numeric_field_is_malformed() {
        let (b, n) = line(|o| seal(b"BEACON_REQ,,5", o));
        assert_eq!(parse_from_obc(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"STATUS,0,,0,0,0", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
    }

    #[test]
    fn one_field_messages_take_1_to_text_max_characters() {
        let long = [b'A'; TEXT_MAX + 1];
        let max = core::str::from_utf8(&long[..TEXT_MAX]).unwrap();
        let over = core::str::from_utf8(&long).unwrap();
        let (b, n) = line(|o| ack(max, o));
        assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::Ack(max)));
        let (b, n) = line(|o| nack(max, o));
        assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::Nack(max)));
        let (b, n) = line(|o| shutdown(max, o));
        assert_eq!(parse_from_obc(&b[..n]), Ok(FromObc::Shutdown(max)));
        let mut out = [0u8; LINE_MAX];
        for text in ["", over, "a,b"] {
            assert_eq!(ack(text, &mut out), Err(LinkError::Format));
            assert_eq!(nack(text, &mut out), Err(LinkError::Format));
            assert_eq!(shutdown(text, &mut out), Err(LinkError::Format));
        }
        let (b, n) = line(|o| seal(b"ACK,", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
        let mut body = [b'A'; 5 + TEXT_MAX + 1];
        body[..5].copy_from_slice(b"NACK,");
        let (b, n) = line(|o| seal(&body, o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(&body[..body.len() - 1], o));
        assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::Nack(max)));
    }

    #[test]
    fn beacon_text_takes_1_to_text_max_characters() {
        let long = [b'B'; TEXT_MAX + 1];
        let max = core::str::from_utf8(&long[..TEXT_MAX]).unwrap();
        let over = core::str::from_utf8(&long).unwrap();
        let (b, n) = line(|o| beacon_rsp(9, max, o));
        assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::BeaconRsp { seq: 9, text: max }));
        let mut out = [0u8; LINE_MAX];
        assert_eq!(beacon_rsp(9, over, &mut out), Err(LinkError::Format));
        assert_eq!(beacon_rsp(9, "", &mut out), Err(LinkError::Format));
        let mut body = [b'B'; 13 + TEXT_MAX + 1];
        body[..13].copy_from_slice(b"BEACON_RSP,9,");
        let (b, n) = line(|o| seal(&body, o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"BEACON_RSP,9,", o));
        assert_eq!(parse_from_jetson(&b[..n]), Err(LinkError::Format));
    }

    #[test]
    fn beacon_req_max_len_is_1_to_text_max() {
        let (b, n) = line(|o| beacon_req(7, 1, o));
        assert_eq!(parse_from_obc(&b[..n]), Ok(FromObc::BeaconReq { seq: 7, max_len: 1 }));
        assert_eq!(beacon_req(7, 177, &mut [0u8; 64]), Err(LinkError::Format));
        assert_eq!(beacon_req(7, u8::MAX, &mut [0u8; 64]), Err(LinkError::Format));
    }

    #[test]
    fn status_fields_in_range() {
        let max = Status { health: 3, temp_c: 200, power_mw: 100_000, mem_pct: 100, uptime_s: u32::MAX };
        let min = Status { health: 0, temp_c: -128, power_mw: 0, mem_pct: 0, uptime_s: 0 };
        for s in [max, min] {
            let (b, n) = line(|o| status(&s, o));
            assert_eq!(parse_from_jetson(&b[..n]), Ok(FromJetson::Status(s)));
        }
        let out_of_range = [
            Status { health: 4, ..max },
            Status { temp_c: 201, ..max },
            Status { temp_c: -129, ..min },
            Status { power_mw: 100_001, ..max },
            Status { mem_pct: 101, ..max },
        ];
        for s in out_of_range {
            assert_eq!(status(&s, &mut [0u8; LINE_MAX]), Err(LinkError::Format), "{s:?}");
        }
    }

    #[test]
    fn status_req_takes_no_fields() {
        let (b, n) = line(status_req);
        assert_eq!(parse_from_obc(&b[..n]), Ok(FromObc::StatusReq));
        let (b, n) = line(|o| seal(b"STATUS_REQ,x", o));
        assert_eq!(parse_from_obc(&b[..n]), Err(LinkError::Format));
        let (b, n) = line(|o| seal(b"STATUS_REQ,", o));
        assert_eq!(parse_from_obc(&b[..n]), Err(LinkError::Format));
    }

    #[test]
    fn telemetry_in_eclipse() {
        let (b, n) = line(|o| seal(b"TELEMETRY,5,ECLIPSE,0.25,0,1.5", o));
        let Ok(FromObc::Telemetry(t)) = parse_from_obc(&b[..n]) else { panic!() };
        assert_eq!((t.time_ms, t.sunlit, t.soc, t.power_mode, t.power_limit_w), (5, false, 0.25, 0, 1.5));
        assert_eq!(t.extras().next(), None);
    }

    #[test]
    fn rx_holds_a_line_of_line_max_with_its_newline() {
        let mut rx = LineRx::new();
        for _ in 0..LINE_MAX - 1 {
            assert!(rx.feed(b'A').is_none());
        }
        assert_eq!(rx.feed(b'\n').map(|r| r.map(<[u8]>::len)), Some(Ok(LINE_MAX - 1)));
        for _ in 0..LINE_MAX {
            assert!(rx.feed(b'A').is_none());
        }
        assert_eq!(rx.feed(b'\n'), Some(Err(LinkError::Format)));
        assert_eq!(rx.feed(b'\n'), Some(Ok(&b""[..])));
    }

    #[test]
    fn rx_reset_forgets_a_partial_or_overlong_line() {
        let mut rx = LineRx::default();
        for byte in b"partial" {
            assert!(rx.feed(*byte).is_none());
        }
        rx.reset();
        for byte in b"OK" {
            assert!(rx.feed(*byte).is_none());
        }
        assert_eq!(rx.feed(b'\n'), Some(Ok(&b"OK"[..])));
        for _ in 0..LINE_MAX {
            assert!(rx.feed(b'A').is_none());
        }
        rx.reset();
        for byte in b"OK" {
            assert!(rx.feed(*byte).is_none());
        }
        assert_eq!(rx.feed(b'\n'), Some(Ok(&b"OK"[..])));
    }
}
