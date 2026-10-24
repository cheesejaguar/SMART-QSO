//! Uplink command frame (SQSO-ICD-013 §3):
//!
//! ```text
//! 0xAA 0x55 | cmd_id (1) | len (1) | payload (len) | sum16 (2, big-endian)
//! ```
//!
//! The payload is the authentication block ([`crate::auth`]). The sum only
//! catches corruption; the HMAC tag in the payload is what authenticates.

use crate::checksum::sum16;
use crate::write::SliceWriter;

/// Frame start
pub const SYNC: [u8; 2] = [0xAA, 0x55];
/// Sync, command ID and length
pub const HEADER_LEN: usize = 4;
/// Trailing checksum
pub const CHECKSUM_LEN: usize = 2;
/// Shortest buffer looked at further
pub const MIN_LEN: usize = 4;
/// Longest frame accepted
pub const MAX_LEN: usize = 256;

/// A frame whose framing and checksum are correct
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UplinkFrame<'a> {
    /// Command ID (not yet checked against the dictionary)
    pub cmd_id: u8,
    /// Payload: counter | args | tag
    pub payload: &'a [u8],
}

/// Why a buffer is not a frame, in the order the checks run
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// Fewer than [`MIN_LEN`] bytes
    TooShort,
    /// More than [`MAX_LEN`] bytes
    TooLong,
    /// Does not start with [`SYNC`]
    Sync,
    /// Length byte disagrees with the buffer length
    Length {
        /// Command ID from the header
        cmd_id: u8,
        /// Payload length from the header
        declared: u8,
    },
    /// Checksum mismatch
    Checksum {
        /// Command ID from the header
        cmd_id: u8,
        /// Payload length from the header
        declared: u8,
    },
}

impl FrameError {
    /// Command ID, if the header was read
    pub fn cmd_id(&self) -> Option<u8> {
        match *self {
            FrameError::Length { cmd_id, .. } | FrameError::Checksum { cmd_id, .. } => Some(cmd_id),
            FrameError::TooShort | FrameError::TooLong | FrameError::Sync => None,
        }
    }
}

/// Check framing, length and checksum
pub fn parse(buf: &[u8]) -> Result<UplinkFrame<'_>, FrameError> {
    if buf.len() < MIN_LEN {
        return Err(FrameError::TooShort);
    }
    if buf.len() > MAX_LEN {
        return Err(FrameError::TooLong);
    }
    let [s0, s1, cmd_id, declared, rest @ ..] = buf else {
        return Err(FrameError::TooShort);
    };
    if [*s0, *s1] != SYNC {
        return Err(FrameError::Sync);
    }
    let (cmd_id, declared) = (*cmd_id, *declared);
    if rest.len() != usize::from(declared) + CHECKSUM_LEN {
        return Err(FrameError::Length { cmd_id, declared });
    }
    let (payload, sum) = rest.split_at(usize::from(declared));
    let body_len = HEADER_LEN + usize::from(declared);
    let body = buf.get(..body_len).ok_or(FrameError::Length { cmd_id, declared })?;
    if sum != sum16(body).to_be_bytes() {
        return Err(FrameError::Checksum { cmd_id, declared });
    }
    Ok(UplinkFrame { cmd_id, payload })
}

/// The payload does not fit in a frame, or the output buffer is too small
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncodeError;

/// Write a frame; returns its length
pub fn encode(cmd_id: u8, payload: &[u8], out: &mut [u8]) -> Result<usize, EncodeError> {
    let declared = u8::try_from(payload.len()).map_err(|_| EncodeError)?;
    let mut w = SliceWriter::new(out);
    w.push(&SYNC).map_err(|_| EncodeError)?;
    w.push(&[cmd_id, declared]).map_err(|_| EncodeError)?;
    w.push(payload).map_err(|_| EncodeError)?;
    let body_len = w.len();
    if body_len + CHECKSUM_LEN > MAX_LEN {
        return Err(EncodeError);
    }
    let sum = sum16(out.get(..body_len).ok_or(EncodeError)?);
    let tail = out.get_mut(body_len..body_len + CHECKSUM_LEN).ok_or(EncodeError)?;
    tail.copy_from_slice(&sum.to_be_bytes());
    Ok(body_len + CHECKSUM_LEN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(cmd_id: u8, payload: &[u8]) -> ([u8; MAX_LEN], usize) {
        let mut buf = [0u8; MAX_LEN];
        let n = encode(cmd_id, payload, &mut buf).unwrap_or(0);
        (buf, n)
    }

    #[test]
    fn round_trip() {
        let (buf, n) = frame(0x32, &[1, 2, 3]);
        assert_eq!(n, 9);
        assert_eq!(parse(&buf[..n]), Ok(UplinkFrame { cmd_id: 0x32, payload: &[1, 2, 3] }));
    }

    #[test]
    fn known_frame() {
        // NOP with an empty payload: AA 55 00 00, sum 0x00FF
        assert_eq!(parse(&[0xAA, 0x55, 0x00, 0x00, 0x00, 0xFF]).map(|f| f.cmd_id), Ok(0));
    }

    #[test]
    fn errors_in_order() {
        assert_eq!(parse(&[0xAA, 0x55, 0]), Err(FrameError::TooShort));
        assert_eq!(parse(&[0u8; 257]), Err(FrameError::TooLong));
        assert_eq!(parse(&[0xAA, 0x56, 0, 0, 0, 0]), Err(FrameError::Sync));
        assert_eq!(parse(&[0xAA, 0x55, 7, 0]), Err(FrameError::Length { cmd_id: 7, declared: 0 }));
        assert_eq!(parse(&[0xAA, 0x55, 7, 1, 0, 0]), Err(FrameError::Length { cmd_id: 7, declared: 1 }));
        assert_eq!(parse(&[0xAA, 0x55, 7, 0, 0, 0]), Err(FrameError::Checksum { cmd_id: 7, declared: 0 }));
    }

    #[test]
    fn encode_refuses_oversized() {
        let mut out = [0u8; 400];
        assert_eq!(encode(1, &[0u8; 251], &mut out), Err(EncodeError));
        assert!(encode(1, &[0u8; 250], &mut out).is_ok());
        assert_eq!(encode(1, &[0u8; 10], &mut [0u8; 15]), Err(EncodeError));
    }
}
