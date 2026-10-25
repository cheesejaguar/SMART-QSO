//! `cmd_auth.h`: telecommand key table and replay counters

use sqso_fsw::keys::{AuthError, CommitError, KeyError, KeyEvent, KeyTable, KeyUsed, Verified};
use sqso_protocol::auth::{KEY_SIZE, Key};

use crate::c::{self, FLASH_REGION_AUTH, HalFlash, fault_type, rc, severity};
use crate::cell::Global;
use crate::ptr;

static KEYS: Global<KeyTable> = Global::new(KeyTable::empty(FLASH_REGION_AUTH));

/// `CmdAuthStatus_t`
const AUTH_OK: u32 = 0;
const AUTH_ERR_NOT_PROVISIONED: u32 = 1;
const AUTH_ERR_FORMAT: u32 = 2;
const AUTH_ERR_TAG: u32 = 3;
const AUTH_ERR_REPLAY: u32 = 4;
const AUTH_ERR_EXHAUSTED: u32 = 5;

/// `CmdAuthVerified_t`
#[repr(C)]
pub struct CmdAuthVerified {
    counter: u32,
    args: *const u8,
    args_len: usize,
    key: u32,
}

/// `CmdAuthInfo_t`
#[repr(C)]
pub struct CmdAuthInfo {
    operational_usable: bool,
    recovery_usable: bool,
    active_slot: u8,
    active_key_id: u8,
    recovery_key_id: u8,
    operational_counter: u32,
    recovery_counter: u32,
}

/// Log key-table events to the fault log
fn log_event(event: KeyEvent) {
    match event {
        KeyEvent::RecordUnusable => {
            c::log(fault_type::INIT, severity::ERROR, c"Telecommand key record unusable", 0.0);
        }
        KeyEvent::RevocationNotPersisted => {
            c::log(fault_type::COMMAND, severity::ERROR, c"Telecommand key revocation not persisted", 0.0);
        }
        KeyEvent::RotationNotPersisted => {
            c::log(fault_type::COMMAND, severity::ERROR, c"Telecommand key rotation not persisted", 0.0);
        }
        KeyEvent::Rotated { slot } => {
            c::log(fault_type::COMMAND, severity::INFO, c"Telecommand key rotated", f64::from(slot));
        }
    }
}

/// Run `f` on the key table (`BUSY` if re-entered)
pub(crate) fn with_keys(f: impl FnOnce(&mut KeyTable) -> i32) -> i32 {
    KEYS.with(f).unwrap_or(rc::BUSY)
}

fn key_result(r: Result<(), KeyError>) -> i32 {
    match r {
        Ok(()) => rc::OK,
        Err(KeyError::Invalid) => rc::INVALID,
        Err(KeyError::Io) => rc::IO,
    }
}

/// Load the key table and counters from flash
#[unsafe(no_mangle)]
pub extern "C" fn cmd_auth_init() -> i32 {
    with_keys(|keys| {
        *keys = KeyTable::load(&mut HalFlash, FLASH_REGION_AUTH, &mut log_event);
        rc::OK
    })
}

/// Write a key into a slot (integration only)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_auth_provision_slot(slot: u8, key: *const [u8; KEY_SIZE], key_id: u8) -> i32 {
    // SAFETY: the C contract: key points to KEY_SIZE bytes
    let Some(bytes) = (unsafe { ptr::in_(key) }) else { return rc::NULL_PTR };
    let key = Key::new(*bytes);
    with_keys(|keys| key_result(keys.provision(&mut HalFlash, slot, &key, key_id)))
}

/// Activate a spare operational key and revoke every other active key
pub(crate) fn rotate(slot: u8) -> i32 {
    with_keys(|keys| key_result(keys.rotate(&mut HalFlash, slot, &mut log_event)))
}

/// Activate a spare operational key and revoke every other active key
#[unsafe(no_mangle)]
pub extern "C" fn cmd_auth_rotate(slot: u8) -> i32 {
    rotate(slot)
}

/// Erase the whole key table (integration only)
pub(crate) fn zeroize() -> i32 {
    with_keys(|keys| match keys.zeroize(&mut HalFlash) {
        Ok(()) => rc::OK,
        Err(e) => e.0,
    })
}

/// Erase the whole key table (integration only)
#[unsafe(no_mangle)]
pub extern "C" fn cmd_auth_zeroize() -> i32 {
    zeroize()
}

/// Provision operational slot 0 and activate it
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_auth_provision_key(key: *const [u8; KEY_SIZE], key_id: u8) -> i32 {
    // SAFETY: forwarded C contract
    match unsafe { cmd_auth_provision_slot(0, key, key_id) } {
        rc::OK => rotate(0),
        other => other,
    }
}

fn read<R>(f: impl FnOnce(&KeyTable) -> R, busy: R) -> R {
    KEYS.with(|k| f(k)).unwrap_or(busy)
}

/// Whether an operational key is active
#[unsafe(no_mangle)]
pub extern "C" fn cmd_auth_is_provisioned() -> bool {
    read(KeyTable::is_provisioned, false)
}

/// Whether any key is usable
#[unsafe(no_mangle)]
pub extern "C" fn cmd_auth_has_key_material() -> bool {
    read(KeyTable::has_key_material, false)
}

/// Last accepted operational counter
#[unsafe(no_mangle)]
pub extern "C" fn cmd_auth_last_counter() -> u32 {
    read(|k| k.info().operational_counter, 0)
}

/// Key-table status for telemetry
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_auth_get_info(info: *mut CmdAuthInfo) {
    // SAFETY: the C contract
    if let Some(out) = unsafe { ptr::out(info) } {
        let i = read(KeyTable::info, Default::default());
        *out = CmdAuthInfo {
            operational_usable: i.operational_usable,
            recovery_usable: i.recovery_usable,
            active_slot: i.active_slot,
            active_key_id: i.active_key_id,
            recovery_key_id: i.recovery_key_id,
            operational_counter: i.operational_counter,
            recovery_counter: i.recovery_counter,
        };
    }
}

pub(crate) fn auth_status(e: AuthError) -> u32 {
    match e {
        AuthError::NotProvisioned => AUTH_ERR_NOT_PROVISIONED,
        AuthError::Format => AUTH_ERR_FORMAT,
        AuthError::Tag => AUTH_ERR_TAG,
        AuthError::Replay => AUTH_ERR_REPLAY,
        AuthError::Exhausted => AUTH_ERR_EXHAUSTED,
    }
}

/// Verify a command payload; changes nothing
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_auth_verify(
    cmd_id: u8,
    payload: *const u8,
    payload_len: usize,
    verified: *mut CmdAuthVerified,
) -> u32 {
    // SAFETY: the C contract
    let Some(out) = (unsafe { ptr::out(verified) }) else { return AUTH_ERR_FORMAT };
    // SAFETY: the C contract: payload has payload_len bytes (NULL is a format error)
    let payload = unsafe { ptr::bytes(payload, payload_len) }.unwrap_or(&[]);
    read(
        |keys| match keys.verify(cmd_id, payload) {
            Ok(v) => {
                *out = CmdAuthVerified {
                    counter: v.counter,
                    args: v.args.as_ptr(),
                    args_len: v.args.len(),
                    key: v.key as u32,
                };
                AUTH_OK
            }
            Err(e) => auth_status(e),
        },
        AUTH_ERR_NOT_PROVISIONED,
    )
}

/// Accept a verified counter for its key and persist it
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cmd_auth_commit(verified: *const CmdAuthVerified) -> i32 {
    // SAFETY: the C contract
    let Some(v) = (unsafe { ptr::in_(verified) }) else { return rc::NULL_PTR };
    let key = match v.key {
        0 => KeyUsed::None,
        1 => KeyUsed::Operational,
        2 => KeyUsed::Recovery,
        _ => return rc::INVALID,
    };
    let verified = Verified { counter: v.counter, args: &[], key };
    with_keys(|keys| match keys.commit(&mut HalFlash, &verified) {
        Ok(()) => rc::OK,
        Err(CommitError::NotUsable) => rc::ERROR,
        Err(CommitError::Stale) => rc::INVALID,
        Err(CommitError::Io) => rc::IO,
    })
}
