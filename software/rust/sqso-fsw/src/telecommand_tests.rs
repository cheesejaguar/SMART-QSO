//! Unit tests of `telecommand.rs` (a separate file: code scanning skips
//! test code, whose keys are fixtures; see .github/codeql/codeql-config.yml)

use super::*;
use crate::nvm::MemFlash;
use sqso_protocol::auth::{Key, build_frame};

const KEY: Key = Key::new([7; 32]);

fn setup() -> (MemFlash<1024>, KeyTable) {
    let mut f = MemFlash::new();
    let mut t = KeyTable::empty(6);
    t.provision(&mut f, 0, &KEY, 1).unwrap();
    t.rotate(&mut f, 0, &mut |_| {}).unwrap();
    (f, t)
}

fn frame(cmd_id: u8, counter: u32, args: &[u8]) -> ([u8; 256], usize) {
    let mut out = [0u8; 256];
    let n = build_frame(&KEY, cmd_id, counter, args, &mut out).unwrap();
    (out, n)
}

#[test]
fn accepted_then_replayed() {
    let (mut f, mut t) = setup();
    let (b, n) = frame(Command::SetMode.id(), 3, &[1]);
    let Admission::Accepted { command, contact } = admit(&b[..n], &mut t, &mut f, |_| false) else { panic!() };
    assert_eq!((command.command, command.args, contact), (Some(Command::SetMode), &[1u8][..], true));
    assert!(matches!(
        admit(&b[..n], &mut t, &mut f, |_| false),
        Admission::Rejected { why: Rejection::AuthFailed(AuthError::Replay), contact: false, .. }
    ));
}

#[test]
fn rejections_spend_no_counter() {
    let (mut f, mut t) = setup();
    let (b, n) = frame(Command::SetMode.id(), 3, &[1, 2]);
    assert!(matches!(admit(&b[..n], &mut t, &mut f, |_| false), Admission::Rejected { why: Rejection::ArgLength, .. }));
    let (b, n) = frame(0x7F, 4, &[]);
    assert!(matches!(
        admit(&b[..n], &mut t, &mut f, |_| false),
        Admission::Rejected { why: Rejection::UnknownCommand, .. }
    ));
    assert!(matches!(admit(&b[..n], &mut t, &mut f, |id| id == 0x7F), Admission::Accepted { .. }));
    assert_eq!(t.info().operational_counter, 4);
    assert!(matches!(
        admit(&[0xAA, 0x55, 9], &mut t, &mut f, |_| false),
        Admission::Rejected { cmd_id: 0, why: Rejection::InvalidFrame, .. }
    ));
}

#[test]
fn tx_disable_survives_a_flash_failure() {
    let (mut f, mut t) = setup();
    f.writes_left = Some(0);
    let (b, n) = frame(Command::TxDisable.id(), 5, &[]);
    assert!(matches!(admit(&b[..n], &mut t, &mut f, |_| false), Admission::Accepted { contact: true, .. }));
    let (b, n) = frame(Command::Nop.id(), 6, &[]);
    assert!(matches!(
        admit(&b[..n], &mut t, &mut f, |_| false),
        Admission::Rejected { why: Rejection::CounterNotPersisted, contact: true, .. }
    ));
}

/// Mutation check: the contact hook's gate and placement. Contact is
/// reported only for a keyed frame whose counter was spent: never for
/// the keyless TX_DISABLE (gate), and never for a rejection before the
/// counter is spent (placement): framing, authentication, an unknown
/// command or the argument length. A spent counter that could not be
/// persisted is still contact.
#[test]
fn contact_hook_gate_and_placement() {
    let contact = |a: Admission<'_>| match a {
        Admission::Accepted { contact, .. } | Admission::Rejected { contact, .. } => contact,
    };
    // Gate: keyless TX_DISABLE on an unprovisioned table executes, no contact
    let mut f = MemFlash::<1024>::new();
    let mut empty = KeyTable::empty(6);
    let (b, n) = frame(Command::TxDisable.id(), 1, &[]);
    let a = admit(&b[..n], &mut empty, &mut f, |_| false);
    assert!(matches!(a, Admission::Accepted { .. }));
    assert!(!contact(a));

    // Placement: rejections before the counter is spent are not contact
    let (mut f, mut t) = setup();
    let (b, n) = frame(Command::SetMode.id(), 3, &[1, 2]);
    assert!(!contact(admit(&b[..n], &mut t, &mut f, |_| false)), "argument length");
    let (b, n) = frame(0x7F, 3, &[]);
    assert!(!contact(admit(&b[..n], &mut t, &mut f, |_| false)), "unknown command");
    let (mut b, n) = frame(Command::Nop.id(), 3, &[]);
    b[n - 3] ^= 0x01; // the tag's last byte; the checksum is fixed below
    let sum = b[..n - 2].iter().fold(0u16, |s, x| s.wrapping_add(u16::from(*x)));
    b[n - 2..n].copy_from_slice(&sum.to_be_bytes());
    assert!(matches!(
        admit(&b[..n], &mut t, &mut f, |_| false),
        Admission::Rejected { why: Rejection::AuthFailed(_), contact: false, .. }
    ));
    assert!(!contact(admit(&b[..n - 1], &mut t, &mut f, |_| false)), "framing");
    assert_eq!(t.info().operational_counter, 0, "no counter was spent above");

    // Spent counter: contact, whether accepted or not persisted
    let (b, n) = frame(Command::Nop.id(), 3, &[]);
    assert!(contact(admit(&b[..n], &mut t, &mut f, |_| false)));
    f.writes_left = Some(0);
    let (b, n) = frame(Command::Nop.id(), 4, &[]);
    assert!(matches!(
        admit(&b[..n], &mut t, &mut f, |_| false),
        Admission::Rejected { why: Rejection::CounterNotPersisted, contact: true, .. }
    ));
}

#[test]
fn mode_policy() {
    use Command as C;
    assert!(allowed(C::TxDisable.id(), Some(State::Emergency)));
    assert!(allowed(C::Reset.id(), Some(State::Emergency)));
    assert!(!allowed(C::SetMode.id(), Some(State::Emergency)));
    assert!(allowed(C::SetMode.id(), Some(State::Safe)));
    assert!(!allowed(C::PldEnable.id(), Some(State::Safe)));
    assert!(!allowed(C::Reset.id(), Some(State::Safe)));
    assert!(allowed(C::PldEnable.id(), Some(State::Active)));
    assert!(allowed(0x7F, None));
    assert!(!allowed(0x7F, Some(State::Safe)));
}
