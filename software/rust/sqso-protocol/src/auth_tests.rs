//! Unit tests of `auth.rs` (a separate file: code scanning skips
//! test code, whose keys are fixtures; see .github/codeql/codeql-config.yml)

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
