//! Telecommand key table and replay counters (the logic of `cmd_auth.c`;
//! `docs/interfaces/TELECOMMAND_AUTHENTICATION.md`).
//!
//! - [`OP_SLOTS`] operational keys: one active, the others spares.
//! - One recovery key, held offline on the ground. Per the command
//!   dictionary it authenticates only KEY_ROTATE and TX_DISABLE.
//!
//! Each key has its own counter; a command's counter must be strictly
//! greater than the last one accepted for its key, and is persisted before
//! the command executes. KEY_ROTATE makes a spare active and revokes
//! (zeroizes) the old key. If no key is usable at all, only commands the
//! dictionary marks `without_key` (TX_DISABLE) are accepted, unauthenticated:
//! TX-OFF can only reduce emissions (47 CFR 97.207(b)).
//!
//! Records in the auth flash region (one [`nvm`](crate::nvm) record each):
//!
//! | base | magic | payload |
//! |------|-------|---------|
//! | 0 | `SQTS` | active slot (1), reserved (3), operational counter (4) |
//! | 128 | `SQTR` | key (32), key ID (1), reserved (3), counter (4) |
//! | 256 + 128 s | `SQTK` | key (32), key ID (1), status (1), reserved (2) |

use sqso_protocol::auth::{self, COUNTER_LIMIT, KEY_SIZE, Key};
use sqso_protocol::{Command, KeyClass};
use zeroize::Zeroize;

use crate::nvm::{self, Flash, FOOTPRINT, NvmError, RecordStatus};

/// Operational key slots
pub const OP_SLOTS: u8 = 4;
/// Slot number of the recovery key in provisioning
pub const RECOVERY_SLOT: u8 = OP_SLOTS;

const MAGIC_STATE: u32 = 0x5351_5453; // "SQTS"
const MAGIC_RECOVERY: u32 = 0x5351_5452; // "SQTR"
const MAGIC_OP_KEY: u32 = 0x5351_544B; // "SQTK"

const BASE_STATE: u32 = 0;
const BASE_RECOVERY: u32 = FOOTPRINT;
const fn base_op(slot: u8) -> u32 {
    (2 + slot as u32) * FOOTPRINT
}

const STATE_LEN: usize = 8;
const RECOVERY_LEN: usize = 40;
const OP_LEN: usize = 36;

const KEY_SPARE: u8 = 0x5A;
const KEY_ACTIVE: u8 = 0xA5;
const KEY_REVOKED: u8 = 0x00;

/// Which key authenticated a command
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum KeyUsed {
    /// Accepted without a key (no key usable; TX_DISABLE only)
    None = 0,
    /// The active operational key
    Operational = 1,
    /// The recovery key
    Recovery = 2,
}

/// An authentic command with a fresh counter (not yet committed)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verified<'a> {
    /// Command counter
    pub counter: u32,
    /// Arguments
    pub args: &'a [u8],
    /// Key that authenticated it
    pub key: KeyUsed,
}

/// Why verification failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthError {
    /// No usable key for this command: fail closed
    NotProvisioned,
    /// Payload too short or arguments too long
    Format,
    /// Tag does not match
    Tag,
    /// Counter not greater than the last accepted
    Replay,
    /// Counter space exhausted: rotate the key
    Exhausted,
}

/// Why a key-table change failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyError {
    /// Bad slot, or the slot does not hold a spare
    Invalid,
    /// Not persisted
    Io,
}

/// Why a commit failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitError {
    /// The key is no longer usable
    NotUsable,
    /// The counter is not fresh
    Stale,
    /// Advanced in memory but not persisted
    Io,
}

/// Something the key table wants logged
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyEvent {
    /// A key record was corrupt or unreadable at load (that key is unusable)
    RecordUnusable,
    /// A stale active key could not be revoked in flash
    RevocationNotPersisted,
    /// A rotation could not be persisted (the old key stays in force)
    RotationNotPersisted,
    /// A spare was activated
    Rotated {
        /// The new active slot
        slot: u8,
    },
}

/// Key-table status for telemetry (no key material)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Info {
    /// An operational key is active
    pub operational_usable: bool,
    /// The recovery key is present
    pub recovery_usable: bool,
    /// Active operational slot
    pub active_slot: u8,
    /// Active operational key ID
    pub active_key_id: u8,
    /// Recovery key ID
    pub recovery_key_id: u8,
    /// Last accepted operational counter
    pub operational_counter: u32,
    /// Last accepted recovery counter
    pub recovery_counter: u32,
}

#[derive(Clone)]
struct KeyState {
    key: Key,
    key_id: u8,
    counter: u32,
}

/// The key table in RAM
#[derive(Clone)]
pub struct KeyTable {
    region: u32,
    op: Option<KeyState>,
    op_slot: u8,
    rec: Option<KeyState>,
}

fn be_u32(bytes: &[u8]) -> u32 {
    let mut word = [0u8; 4];
    for (dst, src) in word.iter_mut().zip(bytes) {
        *dst = *src;
    }
    u32::from_be_bytes(word)
}

fn key_from(record: &[u8]) -> Key {
    let mut bytes = [0u8; KEY_SIZE];
    for (dst, src) in bytes.iter_mut().zip(record) {
        *dst = *src;
    }
    let key = Key::new(bytes);
    bytes.zeroize();
    key
}


fn load_op<F: Flash>(flash: &mut F, region: u32, slot: u8, rec: &mut [u8; OP_LEN]) -> Result<RecordStatus, NvmError> {
    nvm::load(flash, region, base_op(slot), MAGIC_OP_KEY, rec)
}

fn save_op<F: Flash>(flash: &mut F, region: u32, slot: u8, key: &[u8], key_id: u8, status: u8) -> Result<(), KeyError> {
    let mut rec = [0u8; OP_LEN];
    for (dst, src) in rec.iter_mut().zip(key.iter().take(KEY_SIZE)) {
        *dst = *src;
    }
    rec[32] = key_id;
    rec[33] = status;
    let result = nvm::save(flash, region, base_op(slot), MAGIC_OP_KEY, &rec);
    rec.zeroize();
    result.map_err(|_| KeyError::Io)
}

impl KeyTable {
    /// No keys: every command but TX_DISABLE is refused
    pub const fn empty(region: u32) -> Self {
        KeyTable { region, op: None, op_slot: 0, rec: None }
    }

    /// Load the table from flash. Revokes any operational key left marked
    /// active by an interrupted rotation; a blank or corrupt record leaves
    /// that key unusable (fail closed).
    pub fn load<F: Flash>(flash: &mut F, region: u32, log: &mut impl FnMut(KeyEvent)) -> Self {
        let mut table = KeyTable::empty(region);
        let mut fault = false;

        let mut rec = [0u8; RECOVERY_LEN];
        match nvm::load(flash, region, BASE_RECOVERY, MAGIC_RECOVERY, &mut rec) {
            Ok(RecordStatus::Valid) => {
                table.rec = Some(KeyState {
                    key: key_from(&rec),
                    key_id: rec[32],
                    counter: be_u32(rec.get(36..).unwrap_or(&[])),
                });
            }
            Ok(RecordStatus::Blank) => {}
            Ok(RecordStatus::Corrupt) | Err(_) => fault = true,
        }
        rec.zeroize();

        let mut state = [0u8; STATE_LEN];
        match nvm::load(flash, region, BASE_STATE, MAGIC_STATE, &mut state) {
            Ok(RecordStatus::Valid) => {
                let slot = state[0];
                let mut op = [0u8; OP_LEN];
                if slot < OP_SLOTS
                    && load_op(flash, region, slot, &mut op) == Ok(RecordStatus::Valid)
                    && op[33] == KEY_ACTIVE
                {
                    table.op = Some(KeyState {
                        key: key_from(&op),
                        key_id: op[32],
                        counter: be_u32(state.get(4..).unwrap_or(&[])),
                    });
                    table.op_slot = slot;
                    // A rotation interrupted before its state write left a key
                    // marked active that never took effect: revoke it
                    table.revoke_stale_active(flash, slot, log);
                } else {
                    fault = true;
                }
                op.zeroize();
            }
            Ok(RecordStatus::Blank) => {}
            Ok(RecordStatus::Corrupt) | Err(_) => fault = true,
        }

        if fault {
            log(KeyEvent::RecordUnusable);
        }
        table
    }

    fn revoke_stale_active<F: Flash>(&self, flash: &mut F, keep: u8, log: &mut impl FnMut(KeyEvent)) {
        for slot in (0..OP_SLOTS).filter(|s| *s != keep) {
            let mut op = [0u8; OP_LEN];
            if load_op(flash, self.region, slot, &mut op) == Ok(RecordStatus::Valid)
                && op[33] == KEY_ACTIVE
                && save_op(flash, self.region, slot, &[0; KEY_SIZE], 0, KEY_REVOKED).is_err()
            {
                log(KeyEvent::RevocationNotPersisted);
            }
            op.zeroize();
        }
    }

    fn save_state<F: Flash>(&self, flash: &mut F, slot: u8, counter: u32) -> Result<(), KeyError> {
        let mut rec = [0u8; STATE_LEN];
        rec[0] = slot;
        if let Some(dst) = rec.get_mut(4..) {
            dst.copy_from_slice(&counter.to_be_bytes());
        }
        nvm::save(flash, self.region, BASE_STATE, MAGIC_STATE, &rec).map_err(|_| KeyError::Io)
    }

    fn save_recovery<F: Flash>(&self, flash: &mut F) -> Result<(), KeyError> {
        let mut rec = [0u8; RECOVERY_LEN];
        if let Some(ks) = &self.rec {
            for (dst, src) in rec.iter_mut().zip(ks.key.expose()) {
                *dst = *src;
            }
            rec[32] = ks.key_id;
            if let Some(dst) = rec.get_mut(36..) {
                dst.copy_from_slice(&ks.counter.to_be_bytes());
            }
        }
        let result = nvm::save(flash, self.region, BASE_RECOVERY, MAGIC_RECOVERY, &rec);
        rec.zeroize();
        result.map_err(|_| KeyError::Io)
    }

    /// Write a key into a slot (integration only, over the GSE umbilical).
    /// Operational slots are stored as spares (overwriting the active slot
    /// deactivates it); [`RECOVERY_SLOT`] replaces the recovery key and
    /// resets its counter.
    pub fn provision<F: Flash>(&mut self, flash: &mut F, slot: u8, key: &Key, key_id: u8) -> Result<(), KeyError> {
        if slot > RECOVERY_SLOT {
            return Err(KeyError::Invalid);
        }
        if slot == RECOVERY_SLOT {
            self.rec = Some(KeyState { key: key.clone(), key_id, counter: 0 });
            return self.save_recovery(flash);
        }
        if self.op.is_some() && slot == self.op_slot {
            self.op = None;
        }
        save_op(flash, self.region, slot, key.expose(), key_id, KEY_SPARE)
    }

    /// Make a spare operational key active (counter 0) and revoke every
    /// other active key. Used by KEY_ROTATE and by activation at integration.
    pub fn rotate<F: Flash>(&mut self, flash: &mut F, slot: u8, log: &mut impl FnMut(KeyEvent)) -> Result<(), KeyError> {
        if slot >= OP_SLOTS {
            return Err(KeyError::Invalid);
        }
        let mut rec = [0u8; OP_LEN];
        if load_op(flash, self.region, slot, &mut rec) != Ok(RecordStatus::Valid) || rec[33] != KEY_SPARE {
            rec.zeroize();
            return Err(KeyError::Invalid);
        }

        // 1. Mark the spare active. 2. Point the state record at it (the
        // commit point). 3. Revoke the previous key. A power loss before step
        // 2 leaves the old key in force; load() revokes the half-activated one.
        let key_id = rec[32];
        let persisted = save_op(flash, self.region, slot, &rec, key_id, KEY_ACTIVE)
            .and_then(|()| self.save_state(flash, slot, 0));
        if persisted.is_err() {
            rec.zeroize();
            log(KeyEvent::RotationNotPersisted);
            return Err(KeyError::Io);
        }

        self.revoke_stale_active(flash, slot, log);
        self.op = Some(KeyState { key: key_from(&rec), key_id, counter: 0 });
        self.op_slot = slot;
        rec.zeroize();
        log(KeyEvent::Rotated { slot });
        Ok(())
    }

    /// Erase the whole table (integration only)
    pub fn zeroize<F: Flash>(&mut self, flash: &mut F) -> Result<(), nvm::FlashError> {
        self.op = None;
        self.rec = None;
        self.op_slot = 0;
        flash.erase(self.region)
    }

    /// An operational key is active
    pub fn is_provisioned(&self) -> bool {
        self.op.is_some()
    }

    /// Any key is usable
    pub fn has_key_material(&self) -> bool {
        self.op.is_some() || self.rec.is_some()
    }

    /// Status for telemetry
    pub fn info(&self) -> Info {
        Info {
            operational_usable: self.op.is_some(),
            recovery_usable: self.rec.is_some(),
            active_slot: self.op_slot,
            active_key_id: self.op.as_ref().map_or(0, |k| k.key_id),
            recovery_key_id: self.rec.as_ref().map_or(0, |k| k.key_id),
            operational_counter: self.op.as_ref().map_or(0, |k| k.counter),
            recovery_counter: self.rec.as_ref().map_or(0, |k| k.counter),
        }
    }

    /// Verify a command payload (`counter | args | tag`); changes nothing.
    /// The key is chosen by the dictionary's key class; IDs outside the
    /// dictionary use the operational key.
    pub fn verify<'a>(&self, cmd_id: u8, payload: &'a [u8]) -> Result<Verified<'a>, AuthError> {
        let spec = Command::from_id(cmd_id).map(Command::spec);
        let class = spec.map_or(KeyClass::Operational, |s| s.key);

        if !self.has_key_material() {
            if !spec.is_some_and(|s| s.without_key) {
                return Err(AuthError::NotProvisioned);
            }
            let block = auth::split(payload).map_err(|_| AuthError::Format)?;
            return Ok(Verified { counter: block.counter, args: block.args, key: KeyUsed::None });
        }

        let op = self.op.as_ref().filter(|_| class != KeyClass::Recovery);
        let rec = self.rec.as_ref().filter(|_| class != KeyClass::Operational);
        if op.is_none() && rec.is_none() {
            return Err(AuthError::NotProvisioned);
        }
        let block = auth::split(payload).map_err(|_| AuthError::Format)?;

        let (matched, key) = match (op, rec) {
            (Some(ks), _) if auth::verify(&ks.key, cmd_id, &block) => (ks, KeyUsed::Operational),
            (_, Some(ks)) if auth::verify(&ks.key, cmd_id, &block) => (ks, KeyUsed::Recovery),
            _ => return Err(AuthError::Tag),
        };

        // Counter checks only once the frame is authentic
        if matched.counter >= COUNTER_LIMIT || block.counter > COUNTER_LIMIT {
            return Err(AuthError::Exhausted);
        }
        if block.counter <= matched.counter {
            return Err(AuthError::Replay);
        }
        Ok(Verified { counter: block.counter, args: block.args, key })
    }

    /// Accept a verified counter and persist it; call before executing. The
    /// counter advances in RAM even if persisting fails, so the frame cannot
    /// be replayed before the next reset.
    pub fn commit<F: Flash>(&mut self, flash: &mut F, verified: &Verified<'_>) -> Result<(), CommitError> {
        let slot = match verified.key {
            KeyUsed::None => return Ok(()), // no key, no counter
            KeyUsed::Operational => &mut self.op,
            KeyUsed::Recovery => &mut self.rec,
        };
        let ks = slot.as_mut().ok_or(CommitError::NotUsable)?;
        if verified.counter > COUNTER_LIMIT || verified.counter <= ks.counter {
            return Err(CommitError::Stale);
        }
        ks.counter = verified.counter;
        let counter = ks.counter;
        let saved = match verified.key {
            KeyUsed::Recovery => self.save_recovery(flash),
            KeyUsed::Operational | KeyUsed::None => self.save_state(flash, self.op_slot, counter),
        };
        saved.map_err(|_| CommitError::Io)
    }
}

#[cfg(test)]
mod tests {
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
}
