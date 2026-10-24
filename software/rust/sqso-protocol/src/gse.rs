//! GSE umbilical provisioning frame (integration only; `gse_provision.h`).
//!
//! ```text
//! "SQGP" | op (1) | slot (1) | key_id (1) | key (32) | CRC-32 (4, big-endian)
//! ```
//!
//! The reply is `"SQGR" | status (1)`. Frames are honoured only while the
//! umbilical is present; that check belongs to the caller.

use crate::auth::{KEY_SIZE, Key};
use crate::checksum::crc32;
use crate::write::be_u32;

/// Frame length
pub const FRAME_SIZE: usize = 43;
/// Reply length
pub const RESPONSE_SIZE: usize = 5;
/// Frame magic
pub const FRAME_MAGIC: [u8; 4] = *b"SQGP";
/// Reply magic
pub const REPLY_MAGIC: [u8; 4] = *b"SQGR";

/// Provisioning operation
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GseOp {
    /// Write a key into a slot (operational 0-3, recovery 4)
    Provision {
        /// Key-table slot
        slot: u8,
        /// Key identifier reported in telemetry
        key_id: u8,
        /// Key material
        key: KeyBytes,
    },
    /// Activate a spare operational key
    Activate {
        /// Key-table slot
        slot: u8,
    },
    /// Erase the key table
    Zeroize,
    /// Re-arm RF silence and deployment (last action before shipping)
    ArmFlight,
}

/// Key bytes from a frame, compared without exposing them in `Debug`
#[derive(Clone, PartialEq, Eq)]
pub struct KeyBytes(pub [u8; KEY_SIZE]);

impl KeyBytes {
    /// As a telecommand key
    pub fn key(&self) -> Key {
        Key::new(self.0)
    }
}

impl core::fmt::Debug for KeyBytes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("KeyBytes(..)")
    }
}

impl Drop for KeyBytes {
    fn drop(&mut self) {
        zeroize::Zeroize::zeroize(&mut self.0);
    }
}

/// Why a frame was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GseError {
    /// Length, magic or CRC wrong
    BadFrame,
    /// Unknown operation code
    BadOp,
}

/// Operation codes
pub mod op {
    /// Write a key into a slot
    pub const PROVISION: u8 = 0x01;
    /// Activate a spare operational key
    pub const ACTIVATE: u8 = 0x02;
    /// Erase the key table
    pub const ZEROIZE: u8 = 0x03;
    /// Re-arm RF silence and deployment
    pub const ARM_FLIGHT: u8 = 0x04;
}

/// Check and decode a frame
pub fn parse(frame: &[u8]) -> Result<GseOp, GseError> {
    let frame: &[u8; FRAME_SIZE] = frame.try_into().map_err(|_| GseError::BadFrame)?;
    let (body, crc) = frame.split_at(FRAME_SIZE - 4);
    let [m0, m1, m2, m3, opcode, slot, key_id, key @ ..] = body else {
        return Err(GseError::BadFrame);
    };
    if [*m0, *m1, *m2, *m3] != FRAME_MAGIC || be_u32(crc) != Some(crc32(body)) {
        return Err(GseError::BadFrame);
    }
    match *opcode {
        op::PROVISION => Ok(GseOp::Provision {
            slot: *slot,
            key_id: *key_id,
            key: KeyBytes(key.try_into().map_err(|_| GseError::BadFrame)?),
        }),
        op::ACTIVATE => Ok(GseOp::Activate { slot: *slot }),
        op::ZEROIZE => Ok(GseOp::Zeroize),
        op::ARM_FLIGHT => Ok(GseOp::ArmFlight),
        _ => Err(GseError::BadOp),
    }
}

/// Build a frame (GSE side and tests)
pub fn encode(opcode: u8, slot: u8, key_id: u8, key: &[u8; KEY_SIZE]) -> [u8; FRAME_SIZE] {
    let mut frame = [0u8; FRAME_SIZE];
    let (body, crc) = frame.split_at_mut(FRAME_SIZE - 4);
    let (head, key_dst) = body.split_at_mut(7);
    head.copy_from_slice(&[FRAME_MAGIC[0], FRAME_MAGIC[1], FRAME_MAGIC[2], FRAME_MAGIC[3], opcode, slot, key_id]);
    key_dst.copy_from_slice(key);
    crc.copy_from_slice(&crc32(body).to_be_bytes());
    frame
}

/// The reply to a frame
pub fn reply(status: u8) -> [u8; RESPONSE_SIZE] {
    [REPLY_MAGIC[0], REPLY_MAGIC[1], REPLY_MAGIC[2], REPLY_MAGIC[3], status]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_rejections() {
        let frame = encode(op::PROVISION, 4, 9, &[7; KEY_SIZE]);
        assert_eq!(parse(&frame), Ok(GseOp::Provision { slot: 4, key_id: 9, key: KeyBytes([7; KEY_SIZE]) }));
        assert_eq!(parse(&encode(op::ZEROIZE, 0, 0, &[0; 32])), Ok(GseOp::Zeroize));
        assert_eq!(parse(&encode(9, 0, 0, &[0; 32])), Err(GseError::BadOp));
        let mut bad = frame;
        bad[10] ^= 1;
        assert_eq!(parse(&bad), Err(GseError::BadFrame));
        assert_eq!(parse(&frame[..42]), Err(GseError::BadFrame));
        assert_eq!(reply(4), *b"SQGR\x04");
    }

    #[test]
    fn every_operation_decodes() {
        assert_eq!(parse(&encode(op::ACTIVATE, 2, 0, &[0; KEY_SIZE])), Ok(GseOp::Activate { slot: 2 }));
        assert_eq!(parse(&encode(op::ARM_FLIGHT, 0, 0, &[0; KEY_SIZE])), Ok(GseOp::ArmFlight));
        assert_eq!(parse(&encode(0, 0, 0, &[0; KEY_SIZE])), Err(GseError::BadOp));
    }

    #[test]
    fn debug_hides_the_key() {
        use core::fmt::Write;
        let op = GseOp::Provision { slot: 4, key_id: 9, key: KeyBytes([0x77; KEY_SIZE]) };
        let mut buf = [0u8; 64];
        let mut s = crate::write::SliceWriter::new(&mut buf[..]);
        write!(s, "{op:?}").unwrap();
        let n = s.len();
        assert_eq!(&buf[..n], b"Provision { slot: 4, key_id: 9, key: KeyBytes(..) }");
    }
}
