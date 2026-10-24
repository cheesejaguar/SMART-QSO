//! Telecommand authentication block
//! (`docs/interfaces/TELECOMMAND_AUTHENTICATION.md`).
//!
//! Every command payload is
//!
//! ```text
//! counter (4, big-endian) | args (N) | tag (16)
//! tag = HMAC-SHA-256(key, "SQSO-TC-v1" | cmd_id | N | counter | args)[..16]
//! ```
//!
//! The command ID and argument length are inside the MAC, so a captured
//! frame cannot be relabelled as another command. HMAC and SHA-256 are the
//! RustCrypto implementations; the tag comparison is constant-time.
//! Authentication, not encryption, is what 47 CFR 97.211(b) permits.

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use zeroize::Zeroize;

use crate::command::MAX_ARGS;
use crate::frame;
use crate::write::be_u32;

/// Domain-separation prefix of the MAC input
pub const DOMAIN: &[u8; 10] = b"SQSO-TC-v1";
/// Key length
pub const KEY_SIZE: usize = 32;
/// Truncated tag length
pub const TAG_SIZE: usize = 16;
/// Counter length
pub const COUNTER_SIZE: usize = 4;
/// Bytes authentication adds to every payload
pub const OVERHEAD: usize = COUNTER_SIZE + TAG_SIZE;
/// Highest counter ever accepted (`u32::MAX` never is)
pub const COUNTER_LIMIT: u32 = 0xFFFF_FFFE;

type HmacSha256 = Hmac<Sha256>;

/// A telecommand key; zeroized when dropped
#[derive(Clone)]
pub struct Key([u8; KEY_SIZE]);

impl Drop for Key {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Key {
    /// Wrap key bytes
    pub const fn new(bytes: [u8; KEY_SIZE]) -> Self {
        Key(bytes)
    }

    /// Key from a slice of exactly [`KEY_SIZE`] bytes
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        Some(Key(bytes.try_into().ok()?))
    }

    /// The key bytes, for persisting the key table
    pub fn expose(&self) -> &[u8; KEY_SIZE] {
        &self.0
    }

    /// HMAC keyed with this key (HMAC takes a key of any length, so this
    /// cannot fail; it reports one as `FormatError` all the same)
    fn mac(&self) -> Result<HmacSha256, FormatError> {
        HmacSha256::new_from_slice(&self.0).map_err(|_| FormatError)
    }
}

impl core::fmt::Debug for Key {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Key(..)")
    }
}

/// The fields of an authentication block (not yet verified)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuthBlock<'a> {
    /// Command counter
    pub counter: u32,
    /// Command arguments
    pub args: &'a [u8],
    /// Truncated tag
    pub tag: &'a [u8; TAG_SIZE],
}

/// The payload is shorter than [`OVERHEAD`] or its arguments are longer
/// than [`MAX_ARGS`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormatError;

/// Split a payload into counter, arguments and tag
pub fn split(payload: &[u8]) -> Result<AuthBlock<'_>, FormatError> {
    let args_len = payload.len().checked_sub(OVERHEAD).ok_or(FormatError)?;
    if args_len > MAX_ARGS {
        return Err(FormatError);
    }
    let (head, tag) = payload.split_at(COUNTER_SIZE + args_len);
    let (counter, args) = head.split_at(COUNTER_SIZE);
    Ok(AuthBlock { counter: be_u32(counter).ok_or(FormatError)?, args, tag: tag.try_into().map_err(|_| FormatError)? })
}

fn keyed_mac(key: &Key, cmd_id: u8, counter: u32, args: &[u8]) -> Result<HmacSha256, FormatError> {
    let args_len = u8::try_from(args.len()).map_err(|_| FormatError)?;
    if args.len() > MAX_ARGS {
        return Err(FormatError);
    }
    let mut mac = key.mac()?;
    mac.update(DOMAIN);
    mac.update(&[cmd_id, args_len]);
    mac.update(&counter.to_be_bytes());
    mac.update(args);
    Ok(mac)
}

/// The tag for a command
pub fn tag(key: &Key, cmd_id: u8, counter: u32, args: &[u8]) -> Result<[u8; TAG_SIZE], FormatError> {
    let full = keyed_mac(key, cmd_id, counter, args)?.finalize().into_bytes();
    let mut out = [0u8; TAG_SIZE];
    out.copy_from_slice(full.get(..TAG_SIZE).ok_or(FormatError)?);
    Ok(out)
}

/// Whether the block's tag is the key's tag for this command (constant time)
pub fn verify(key: &Key, cmd_id: u8, block: &AuthBlock<'_>) -> bool {
    match keyed_mac(key, cmd_id, block.counter, block.args) {
        Ok(mac) => mac.verify_truncated_left(block.tag).is_ok(),
        Err(FormatError) => false,
    }
}

/// HMAC-SHA-256 of a message (RFC 2104; a key longer than one block is
/// hashed first), for the C `hmac_sha256()` API
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> Option<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(msg);
    Some(mac.finalize().into_bytes().into())
}

/// Why a command could not be built
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildError {
    /// Counter 0 or above [`COUNTER_LIMIT`] (never accepted onboard)
    Counter,
    /// More than [`MAX_ARGS`] argument bytes
    Args,
    /// Output buffer too small
    Space,
}

/// Write `counter | args | tag`; returns its length (ground side)
pub fn build_payload(key: &Key, cmd_id: u8, counter: u32, args: &[u8], out: &mut [u8]) -> Result<usize, BuildError> {
    if counter == 0 || counter > COUNTER_LIMIT {
        return Err(BuildError::Counter);
    }
    let tag = tag(key, cmd_id, counter, args).map_err(|_| BuildError::Args)?;
    let len = OVERHEAD + args.len();
    let dst = out.get_mut(..len).ok_or(BuildError::Space)?;
    let (head, dst_tag) = dst.split_at_mut(COUNTER_SIZE + args.len());
    let (dst_counter, dst_args) = head.split_at_mut(COUNTER_SIZE);
    dst_counter.copy_from_slice(&counter.to_be_bytes());
    dst_args.copy_from_slice(args);
    dst_tag.copy_from_slice(&tag);
    Ok(len)
}

/// Write a complete uplink frame carrying an authenticated command
pub fn build_frame(key: &Key, cmd_id: u8, counter: u32, args: &[u8], out: &mut [u8]) -> Result<usize, BuildError> {
    let mut payload = [0u8; OVERHEAD + MAX_ARGS];
    let len = build_payload(key, cmd_id, counter, args, &mut payload)?;
    let payload = payload.get(..len).ok_or(BuildError::Space)?;
    frame::encode(cmd_id, payload, out).map_err(|_| BuildError::Space)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: Key = Key::new([0x42; KEY_SIZE]);

    #[test]
    fn rfc4231_case_2() {
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?").unwrap();
        assert_eq!(mac[..4], [0x5b, 0xdc, 0xc1, 0x46]);
        assert_eq!(mac[28..], [0x64, 0xec, 0x38, 0x43]);
    }

    #[test]
    fn built_payload_verifies_and_binds_the_command() {
        let mut out = [0u8; 100];
        let n = build_payload(&KEY, 0x02, 7, &[1], &mut out).unwrap();
        let block = split(&out[..n]).unwrap();
        assert_eq!((block.counter, block.args), (7, &[1u8][..]));
        assert!(verify(&KEY, 0x02, &block));
        assert!(!verify(&KEY, 0x10, &block), "relabelled command must fail");
        assert!(!verify(&Key::new([0x43; KEY_SIZE]), 0x02, &block));
        let mut forged = out;
        forged[4] ^= 1;
        assert!(!verify(&KEY, 0x02, &split(&forged[..n]).unwrap()));
    }

    #[test]
    fn split_bounds() {
        assert_eq!(split(&[0u8; OVERHEAD - 1]), Err(FormatError));
        assert!(split(&[0u8; OVERHEAD]).is_ok());
        assert!(split(&[0u8; OVERHEAD + MAX_ARGS]).is_ok());
        assert_eq!(split(&[0u8; OVERHEAD + MAX_ARGS + 1]), Err(FormatError));
    }

    #[test]
    fn counter_limits() {
        let mut out = [0u8; 300];
        assert_eq!(build_payload(&KEY, 0, 0, &[], &mut out), Err(BuildError::Counter));
        assert_eq!(build_payload(&KEY, 0, u32::MAX, &[], &mut out), Err(BuildError::Counter));
        assert_eq!(build_payload(&KEY, 0, 1, &[0; 65], &mut out), Err(BuildError::Args));
        assert_eq!(build_payload(&KEY, 0, 1, &[], &mut out[..10]), Err(BuildError::Space));
        assert!(build_frame(&KEY, 0, COUNTER_LIMIT, &[0; 64], &mut out).is_ok());
    }

    #[test]
    fn debug_hides_the_key() {
        use core::fmt::Write;
        let mut buf = [0u8; 16];
        let mut s = crate::write::SliceWriter::new(&mut buf[..]);
        write!(s, "{:?}", KEY).unwrap();
        assert_eq!(s.len(), "Key(..)".len());
    }

    #[test]
    fn key_from_slice_takes_exactly_key_size_bytes() {
        let key = Key::from_slice(&[0x42; KEY_SIZE]).unwrap();
        assert_eq!(key.expose(), KEY.expose());
        assert!(Key::from_slice(&[0x42; KEY_SIZE - 1]).is_none());
        assert!(Key::from_slice(&[0x42; KEY_SIZE + 1]).is_none());
    }

    #[test]
    fn built_frame_carries_the_authenticated_payload() {
        // sync | cmd_id | len | counter | args | tag | sum16 (SQSO-ICD-013 §3)
        let mut out = [0u8; frame::MAX_LEN];
        let n = build_frame(&KEY, 0x02, 7, &[1], &mut out).unwrap();
        assert_eq!(n, frame::HEADER_LEN + OVERHEAD + 1 + frame::CHECKSUM_LEN);
        let f = frame::parse(&out[..n]).unwrap();
        assert_eq!(f.cmd_id, 0x02);
        let block = split(f.payload).unwrap();
        assert_eq!((block.counter, block.args), (7, &[1u8][..]));
        assert!(verify(&KEY, 0x02, &block));
        assert_eq!(build_frame(&KEY, 0x02, 7, &[1], &mut out[..n - 1]), Err(BuildError::Space));
    }
}
