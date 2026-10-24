//! Check sequences, from the `crc` crate's catalogue (each algorithm is
//! named and has a published check value; no hand-written polynomial).
//!
//! @requirement SRS-F061 Protect persisted data with CRC32

use ::crc::{CRC_16_IBM_3740, CRC_16_IBM_SDLC, CRC_32_ISO_HDLC, Crc};

const CCITT_FALSE: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_3740);
const X25: Crc<u16> = Crc::<u16>::new(&CRC_16_IBM_SDLC);
const IEEE: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF, no reflection, no final
/// XOR; check 0x29B1): the OBC-Jetson line CRC (ICD_OBC_JETSON §4.4)
pub fn crc16_ccitt_false(data: &[u8]) -> u16 {
    CCITT_FALSE.checksum(data)
}

/// [`crc16_ccitt_false`] continued from a previous value (this CRC has no
/// reflection or final XOR, so the value is the running register)
pub fn crc16_ccitt_false_from(crc: u16, data: &[u8]) -> u16 {
    let mut digest = CCITT_FALSE.digest_with_initial(crc);
    digest.update(data);
    digest.finalize()
}

/// CRC-16/X.25 (check 0x906E): the AX.25 frame check sequence
pub fn crc16_x25(data: &[u8]) -> u16 {
    X25.checksum(data)
}

/// CRC-32 (IEEE 802.3; check 0xCBF43926): persisted records, GSE frames
pub fn crc32(data: &[u8]) -> u32 {
    IEEE.checksum(data)
}

/// Incremental CRC-32 over several pieces
pub struct Crc32Digest(::crc::Digest<'static, u32>);

impl Crc32Digest {
    /// Start a CRC-32
    pub fn new() -> Self {
        Crc32Digest(IEEE.digest())
    }

    /// Absorb bytes
    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    /// The CRC of everything absorbed
    pub fn finalize(self) -> u32 {
        self.0.finalize()
    }
}

impl Default for Crc32Digest {
    fn default() -> Self {
        Self::new()
    }
}

/// 16-bit sum of the bytes, wrapping: the uplink frame checksum
/// (SQSO-ICD-013 §3; it detects corruption, the HMAC tag authenticates)
pub fn sum16(data: &[u8]) -> u16 {
    data.iter().fold(0u16, |sum, b| sum.wrapping_add(u16::from(*b)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECK: &[u8] = b"123456789";

    #[test]
    fn check_values() {
        assert_eq!(crc16_ccitt_false(CHECK), 0x29B1);
        assert_eq!(crc16_x25(CHECK), 0x906E);
        assert_eq!(crc32(CHECK), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc16_ccitt_false_from(crc16_ccitt_false(b"1234"), b"56789"), 0x29B1);
    }

    #[test]
    fn incremental_crc32_matches() {
        let mut d = Crc32Digest::new();
        d.update(b"1234");
        d.update(b"56789");
        assert_eq!(d.finalize(), 0xCBF4_3926);
    }

    #[test]
    fn sum16_wraps() {
        assert_eq!(sum16(&[0xFF; 300]), 10964); // 255 * 300 mod 65536
    }
}
