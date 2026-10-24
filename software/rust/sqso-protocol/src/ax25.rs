//! AX.25 v2.2 address encoding, frame check sequence and UI-frame parsing.
//!
//! Frames here run from the first address byte through the 2-byte FCS
//! (little-endian); HDLC flags and bit stuffing belong to the modem.

use crate::checksum::crc16_x25;
use crate::write::SliceWriter;

/// HDLC flag
pub const FLAG: u8 = 0x7E;
/// Control field of an unnumbered-information frame
pub const CTRL_UI: u8 = 0x03;
/// Protocol ID: no layer 3
pub const PID_NO_L3: u8 = 0xF0;
/// One address field entry
pub const ADDR_LEN: usize = 7;
/// Destination, source and up to eight digipeaters
pub const MAX_ADDRS: usize = 10;
/// Longest frame handled
pub const MAX_FRAME_LEN: usize = 330;

const CALL_LEN: usize = 6;
const FCS_LEN: usize = 2;
const SSID_MASK: u8 = 0x1E;
const END_OF_ADDRESS: u8 = 0x01;

/// One 7-byte address field entry
pub type Address = [u8; ADDR_LEN];

/// Encode a call sign (first six bytes, upper-cased, space-padded) and SSID
pub fn encode_address(callsign: &[u8], ssid: u8, is_last: bool) -> Address {
    let mut out = [b' ' << 1; ADDR_LEN];
    for (dst, c) in out.iter_mut().zip(callsign.iter().take(CALL_LEN)) {
        *dst = c.to_ascii_uppercase() << 1;
    }
    out[CALL_LEN] = 0x60 | ((ssid & 0x0F) << 1) | if is_last { END_OF_ADDRESS } else { 0 };
    out
}

/// Whether an address entry names this call sign and SSID (the end bit and
/// reserved bits are ignored)
pub fn address_matches(addr: &Address, callsign: &[u8], ssid: u8) -> bool {
    if callsign.len() > CALL_LEN {
        return false;
    }
    let expected = encode_address(callsign, ssid, false);
    addr[..CALL_LEN] == expected[..CALL_LEN] && (addr[CALL_LEN] & SSID_MASK) == (expected[CALL_LEN] & SSID_MASK)
}

/// AX.25 frame check sequence (CRC-16/X.25)
pub fn fcs(data: &[u8]) -> u16 {
    crc16_x25(data)
}

/// Why a frame was not accepted, in the order the checks run
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ax25Error {
    /// Too short, or a malformed address field
    Format,
    /// Frame check sequence mismatch
    Fcs,
    /// Not a UI frame with PID 0xF0
    NotUi,
}

/// A checked UI frame (views into the frame buffer)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiFrame<'a> {
    /// Destination address
    pub dest: &'a Address,
    /// Source address
    pub src: &'a Address,
    /// Digipeater addresses (a multiple of [`ADDR_LEN`] bytes)
    pub digipeaters: &'a [u8],
    /// Information field
    pub info: &'a [u8],
}

/// Check the FCS and parse a UI frame with PID 0xF0
pub fn parse_ui(frame: &[u8]) -> Result<UiFrame<'_>, Ax25Error> {
    const MINIMUM: usize = 2 * ADDR_LEN + 2 + FCS_LEN;
    if frame.len() < MINIMUM {
        return Err(Ax25Error::Format);
    }
    let (body, received) = frame.split_at(frame.len() - FCS_LEN);
    let received = u16::from_le_bytes(received.try_into().map_err(|_| Ax25Error::Format)?);
    if fcs(body) != received {
        return Err(Ax25Error::Fcs);
    }

    // Address field: 7-byte entries until one has the end bit set, with
    // room after it for control, PID and FCS
    let mut count = 0;
    for (i, entry) in frame.chunks_exact(ADDR_LEN).take(MAX_ADDRS).enumerate() {
        if (i + 1) * ADDR_LEN + 2 + FCS_LEN > frame.len() {
            return Err(Ax25Error::Format);
        }
        let (call, ssid) = entry.split_at(CALL_LEN);
        if call.iter().any(|c| c & END_OF_ADDRESS != 0) {
            return Err(Ax25Error::Format); // call sign bytes never have bit 0 set
        }
        if ssid.first().is_some_and(|s| s & END_OF_ADDRESS != 0) {
            count = i + 1;
            break;
        }
    }
    if count < 2 {
        return Err(Ax25Error::Format);
    }

    let (addresses, rest) = body.split_at(count * ADDR_LEN);
    let [ctrl, pid, info @ ..] = rest else {
        return Err(Ax25Error::Format);
    };
    if (*ctrl, *pid) != (CTRL_UI, PID_NO_L3) {
        return Err(Ax25Error::NotUi);
    }
    let (dest, rest) = addresses.split_at(ADDR_LEN);
    let (src, digipeaters) = rest.split_at(ADDR_LEN);
    Ok(UiFrame {
        dest: dest.try_into().map_err(|_| Ax25Error::Format)?,
        src: src.try_into().map_err(|_| Ax25Error::Format)?,
        digipeaters,
        info,
    })
}

/// A station's address on the link: call sign and SSID
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Station<'a> {
    /// Call sign, 1 to 6 ASCII letters and digits
    pub call: &'a str,
    /// SSID, 0 to 15
    pub ssid: u8,
}

/// Not a valid AX.25 call sign
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallsignError;

impl<'a> Station<'a> {
    /// Parse `CALL` or `CALL-SSID`
    pub fn parse(text: &'a str) -> Result<Self, CallsignError> {
        let (call, ssid) = match text.split_once('-') {
            Some((call, ssid)) => (call, ssid.parse::<u8>().map_err(|_| CallsignError)?),
            None => (text, 0),
        };
        if call.is_empty() || call.len() > CALL_LEN || !call.bytes().all(|c| c.is_ascii_alphanumeric()) || ssid > 15 {
            return Err(CallsignError);
        }
        Ok(Station { call, ssid })
    }
}

/// The output buffer is too small or the frame too long
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncodeError;

/// Write a UI frame (PID 0xF0) from the address field through the FCS;
/// with `with_fcs` false the FCS is left off (a KISS TNC appends it)
pub fn encode_ui(
    dest: Station<'_>,
    src: Station<'_>,
    info: &[u8],
    with_fcs: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    let mut w = SliceWriter::new(out);
    w.push(&encode_address(dest.call.as_bytes(), dest.ssid, false)).map_err(|_| EncodeError)?;
    w.push(&encode_address(src.call.as_bytes(), src.ssid, true)).map_err(|_| EncodeError)?;
    w.push(&[CTRL_UI, PID_NO_L3]).map_err(|_| EncodeError)?;
    w.push(info).map_err(|_| EncodeError)?;
    let body_len = w.len();
    let total = body_len + if with_fcs { FCS_LEN } else { 0 };
    if total > MAX_FRAME_LEN {
        return Err(EncodeError);
    }
    if with_fcs {
        let sum = fcs(out.get(..body_len).ok_or(EncodeError)?);
        out.get_mut(body_len..total).ok_or(EncodeError)?.copy_from_slice(&sum.to_le_bytes());
    }
    Ok(total)
}

/// KISS framing: FEND, data-frame command for `port`, escaped frame, FEND
pub fn kiss_encode(frame: &[u8], port: u8, out: &mut [u8]) -> Result<usize, EncodeError> {
    const FEND: u8 = 0xC0;
    const FESC: u8 = 0xDB;
    let mut w = SliceWriter::new(out);
    w.push(&[FEND, (port & 0x0F) << 4]).map_err(|_| EncodeError)?;
    for b in frame {
        match *b {
            FEND => w.push(&[FESC, 0xDC]),
            FESC => w.push(&[FESC, 0xDD]),
            other => w.push(&[other]),
        }
        .map_err(|_| EncodeError)?;
    }
    w.push(&[FEND]).map_err(|_| EncodeError)?;
    Ok(w.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const US: Station<'static> = Station { call: "KK6GDB", ssid: 11 };
    const GROUND: Station<'static> = Station { call: "N0CALL", ssid: 0 };

    fn ui(info: &[u8]) -> ([u8; MAX_FRAME_LEN], usize) {
        let mut buf = [0u8; MAX_FRAME_LEN];
        let n = encode_ui(US, GROUND, info, true, &mut buf).unwrap();
        (buf, n)
    }

    #[test]
    fn address_encoding() {
        assert_eq!(encode_address(b"kk6gdb", 11, true), [0x96, 0x96, 0x6C, 0x8E, 0x88, 0x84, 0x77]);
        assert_eq!(encode_address(b"AB", 0, false)[2..6], [0x40; 4]);
        assert!(address_matches(&encode_address(b"KK6GDB", 11, true), b"kk6gdb", 11));
        assert!(!address_matches(&encode_address(b"KK6GDB", 11, true), b"KK6GDB", 10));
        assert!(!address_matches(&encode_address(b"KK6GDB", 11, true), b"KK6GDBX", 11));
    }

    /// The uplink's callsign filter: another call sign with this station's
    /// SSID is not addressed to it, nor is a shorter or longer call
    #[test]
    fn other_call_with_our_ssid_does_not_match() {
        for other in [&b"KK6GDC"[..], b"W1AW", b"KK6GD", b"AK6GDB"] {
            assert!(!address_matches(&encode_address(other, 11, false), b"KK6GDB", 11), "{other:?}");
        }
        assert!(!address_matches(&encode_address(b"KK6GDB", 11, false), b"KK6GD", 11));
    }

    #[test]
    fn round_trip() {
        let (buf, n) = ui(b"hello");
        let frame = parse_ui(&buf[..n]).unwrap();
        assert!(address_matches(frame.dest, b"KK6GDB", 11));
        assert!(address_matches(frame.src, b"N0CALL", 0));
        assert_eq!(frame.info, b"hello");
        assert!(frame.digipeaters.is_empty());
    }

    #[test]
    fn rejections() {
        let (mut buf, n) = ui(b"x");
        assert_eq!(parse_ui(&buf[..17]), Err(Ax25Error::Format));
        buf[n - 3] ^= 1;
        assert_eq!(parse_ui(&buf[..n]), Err(Ax25Error::Fcs));
        let (mut buf, n) = ui(b"x");
        buf[14] = 0x13; // not UI
        let sum = fcs(&buf[..n - 2]).to_le_bytes();
        buf[n - 2..n].copy_from_slice(&sum);
        assert_eq!(parse_ui(&buf[..n]), Err(Ax25Error::NotUi));
    }

    #[test]
    fn end_bit_in_destination_is_malformed() {
        let mut buf = [0u8; 40];
        buf[..7].copy_from_slice(&encode_address(b"KK6GDB", 11, true));
        buf[7..14].copy_from_slice(&encode_address(b"N0CALL", 0, true));
        buf[14] = CTRL_UI;
        buf[15] = PID_NO_L3;
        let sum = fcs(&buf[..16]).to_le_bytes();
        buf[16..18].copy_from_slice(&sum);
        assert_eq!(parse_ui(&buf[..18]), Err(Ax25Error::Format));
    }

    #[test]
    fn station_parse() {
        assert_eq!(Station::parse("KK6GDB-11"), Ok(US));
        assert_eq!(Station::parse("N0CALL"), Ok(GROUND));
        assert!(Station::parse("KK6GDB-16").is_err());
        assert!(Station::parse("TOOLONG1").is_err());
        assert!(Station::parse("").is_err());
        assert!(Station::parse("AB_C").is_err());
    }

    #[test]
    fn kiss_escapes() {
        let mut out = [0u8; 16];
        let n = kiss_encode(&[0xC0, 0x01, 0xDB], 1, &mut out).unwrap();
        assert_eq!(&out[..n], &[0xC0, 0x10, 0xDB, 0xDC, 0x01, 0xDB, 0xDD, 0xC0]);
    }

    #[test]
    fn shortest_frame_has_an_empty_information_field() {
        // Two addresses, control, PID and FCS: 18 bytes
        let (buf, n) = ui(b"");
        assert_eq!(n, 18);
        let frame = parse_ui(&buf[..n]).unwrap();
        assert!(frame.info.is_empty());
        assert!(address_matches(frame.src, b"N0CALL", 0));
    }

    #[test]
    fn digipeater_and_empty_information_field() {
        let mut buf = [0u8; 25];
        buf[..7].copy_from_slice(&encode_address(b"KK6GDB", 11, false));
        buf[7..14].copy_from_slice(&encode_address(b"N0CALL", 0, false));
        buf[14..21].copy_from_slice(&encode_address(b"RELAY", 1, true));
        buf[21] = CTRL_UI;
        buf[22] = PID_NO_L3;
        let sum = fcs(&buf[..23]).to_le_bytes();
        buf[23..].copy_from_slice(&sum);
        let frame = parse_ui(&buf).unwrap();
        assert_eq!(frame.digipeaters, &encode_address(b"RELAY", 1, true)[..]);
        assert!(frame.info.is_empty());
    }

    #[test]
    fn address_field_running_into_the_fcs_is_malformed() {
        // Three entries fill 21 bytes, but a 22-byte frame has 20 before its
        // FCS: the third entry's SSID byte (end bit set) is the first FCS byte
        let mut tested = 0;
        for last in (0..=0xFEu8).step_by(2) {
            let mut buf = [0u8; 22];
            buf[..7].copy_from_slice(&encode_address(b"KK6GDB", 11, false));
            buf[7..14].copy_from_slice(&encode_address(b"N0CALL", 0, false));
            buf[14..20].copy_from_slice(&encode_address(b"RELAY", 0, false)[..6]);
            buf[19] = last;
            let sum = fcs(&buf[..20]).to_le_bytes();
            buf[20..].copy_from_slice(&sum);
            if buf[20] & END_OF_ADDRESS == 0 {
                continue;
            }
            assert_eq!(parse_ui(&buf), Err(Ax25Error::Format));
            tested += 1;
        }
        assert!(tested > 0);
    }

    #[test]
    fn ssid_15_is_the_highest() {
        assert_eq!(Station::parse("KK6GDB-15"), Ok(Station { call: "KK6GDB", ssid: 15 }));
        assert_eq!(Station::parse("A-0"), Ok(Station { call: "A", ssid: 0 }));
    }

    #[test]
    fn longest_frame_is_max_frame_len() {
        let mut out = [0u8; MAX_FRAME_LEN + 8];
        let info = [b'x'; MAX_FRAME_LEN - 2 * ADDR_LEN - 2 - FCS_LEN];
        assert_eq!(encode_ui(US, GROUND, &info, true, &mut out), Ok(MAX_FRAME_LEN));
        assert_eq!(parse_ui(&out[..MAX_FRAME_LEN]).map(|f| f.info.len()), Ok(info.len()));
        let info = [b'x'; MAX_FRAME_LEN - 2 * ADDR_LEN - 2 - FCS_LEN + 1];
        assert_eq!(encode_ui(US, GROUND, &info, true, &mut out), Err(EncodeError));
        // Without the FCS the information field may be two bytes longer
        let info = [b'x'; MAX_FRAME_LEN - 2 * ADDR_LEN - 2];
        assert_eq!(encode_ui(US, GROUND, &info, false, &mut out), Ok(MAX_FRAME_LEN));
        let info = [b'x'; MAX_FRAME_LEN - 2 * ADDR_LEN - 1];
        assert_eq!(encode_ui(US, GROUND, &info, false, &mut out), Err(EncodeError));
    }
}
