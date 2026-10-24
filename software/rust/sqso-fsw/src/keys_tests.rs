//! Unit tests of `keys.rs` (a separate file: code scanning skips
//! test code, whose keys are fixtures; see .github/codeql/codeql-config.yml)

use super::*;
use crate::nvm::MemFlash;
use sqso_protocol::auth::{OVERHEAD, build_payload};

type F = MemFlash<1024>;
const OP: Key = Key::new([0x11; 32]);
const REC: Key = Key::new([0x22; 32]);

fn payload(key: &Key, cmd: Command, counter: u32, args: &[u8]) -> ([u8; OVERHEAD + 64], usize) {
    let mut out = [0u8; OVERHEAD + 64];
    let n = build_payload(key, cmd.id(), counter, args, &mut out).unwrap();
    (out, n)
}

fn provisioned() -> (F, KeyTable) {
    let mut f = F::new();
    let mut t = KeyTable::empty(6);
    t.provision(&mut f, 0, &OP, 1).unwrap();
    t.provision(&mut f, RECOVERY_SLOT, &REC, 9).unwrap();
    t.rotate(&mut f, 0, &mut |_| {}).unwrap();
    (f, t)
}

#[test]
fn unprovisioned_accepts_only_tx_disable() {
    let t = KeyTable::empty(6);
    let (p, n) = payload(&OP, Command::Nop, 1, &[]);
    assert_eq!(t.verify(Command::Nop.id(), &p[..n]), Err(AuthError::NotProvisioned));
    let v = t.verify(Command::TxDisable.id(), &p[..n]).unwrap();
    assert_eq!(v.key, KeyUsed::None);
}

#[test]
fn verify_commit_replay_and_persistence() {
    let (mut f, mut t) = provisioned();
    let (p, n) = payload(&OP, Command::SetMode, 5, &[1]);
    let v = t.verify(Command::SetMode.id(), &p[..n]).unwrap();
    assert_eq!((v.counter, v.args, v.key), (5, &[1u8][..], KeyUsed::Operational));
    t.commit(&mut f, &v).unwrap();
    assert_eq!(t.verify(Command::SetMode.id(), &p[..n]), Err(AuthError::Replay));
    // After a reset the counter is still spent
    let t2 = KeyTable::load(&mut f, 6, &mut |e| panic!("{e:?}"));
    assert_eq!(t2.verify(Command::SetMode.id(), &p[..n]), Err(AuthError::Replay));
    assert_eq!(t2.info().operational_counter, 5);
    assert_eq!(t2.info().recovery_key_id, 9);
}

#[test]
fn key_classes() {
    let (_f, t) = provisioned();
    let (p, n) = payload(&OP, Command::KeyRotate, 1, &[1]);
    assert_eq!(t.verify(Command::KeyRotate.id(), &p[..n]), Err(AuthError::Tag), "operational key cannot rotate");
    let (p, n) = payload(&REC, Command::KeyRotate, 1, &[1]);
    assert_eq!(t.verify(Command::KeyRotate.id(), &p[..n]).unwrap().key, KeyUsed::Recovery);
    let (p, n) = payload(&REC, Command::Nop, 1, &[]);
    assert_eq!(t.verify(Command::Nop.id(), &p[..n]), Err(AuthError::Tag), "recovery key cannot NOP");
    let (p, n) = payload(&REC, Command::TxDisable, 1, &[]);
    assert_eq!(t.verify(Command::TxDisable.id(), &p[..n]).unwrap().key, KeyUsed::Recovery);
}

#[test]
fn rotation_revokes_the_old_key() {
    let (mut f, mut t) = provisioned();
    let new = Key::new([0x33; 32]);
    t.provision(&mut f, 2, &new, 2).unwrap();
    let mut events = 0;
    t.rotate(&mut f, 2, &mut |_| events += 1).unwrap();
    assert_eq!(events, 1);
    let (p, n) = payload(&OP, Command::Nop, 1, &[]);
    assert_eq!(t.verify(0, &p[..n]), Err(AuthError::Tag));
    let (p, n) = payload(&new, Command::Nop, 1, &[]);
    assert!(t.verify(0, &p[..n]).is_ok());
    assert_eq!(t.rotate(&mut f, 0, &mut |_| {}), Err(KeyError::Invalid), "revoked slot is not a spare");
}

#[test]
fn interrupted_rotation_keeps_the_old_key() {
    let (mut f, mut t) = provisioned();
    t.provision(&mut f, 1, &Key::new([0x44; 32]), 3).unwrap();
    f.writes_left = Some(1); // marks the spare active, then the state write fails
    assert_eq!(t.rotate(&mut f, 1, &mut |_| {}), Err(KeyError::Io));
    f.writes_left = None;
    let mut events = 0;
    let t2 = KeyTable::load(&mut f, 6, &mut |_| events += 1);
    assert_eq!(events, 0);
    assert_eq!(t2.info().active_slot, 0);
    let (p, n) = payload(&OP, Command::Nop, 1, &[]);
    assert!(t2.verify(0, &p[..n]).is_ok());
    let mut op = [0u8; OP_LEN];
    assert_eq!(load_op(&mut f, 6, 1, &mut op), Ok(RecordStatus::Valid));
    assert_eq!(op[33], KEY_REVOKED, "half-activated key revoked at load");
}

#[test]
fn exhausted_counter() {
    let (_f, t) = provisioned();
    let mut p = [0u8; OVERHEAD];
    let tag = auth::tag(&OP, 0, u32::MAX, &[]).unwrap();
    p[..4].copy_from_slice(&u32::MAX.to_be_bytes());
    p[4..].copy_from_slice(&tag);
    assert_eq!(t.verify(0, &p), Err(AuthError::Exhausted));
}

#[test]
fn corrupt_record_fails_closed() {
    let (mut f, _t) = provisioned();
    f.bytes[10] ^= 0xFF; // state slot A
    f.bytes[64 + 10] ^= 0xFF; // state slot B
    let mut events = 0;
    let t = KeyTable::load(&mut f, 6, &mut |_| events += 1);
    assert_eq!(events, 1);
    assert!(!t.is_provisioned());
    assert!(t.has_key_material(), "recovery key still usable");
}
