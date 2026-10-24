//! `sha256.h`: SHA-256 and HMAC-SHA-256 from RustCrypto (`sha2`, `hmac`).
//!
//! `Sha256Ctx_t` is opaque storage holding a `sha2::Sha256` after a marker
//! word, so a context that was never initialized is refused rather than read.

use core::mem::{align_of, size_of};

use sha2::{Digest, Sha256};

use crate::c::rc;
use crate::ptr;

/// `SHA256_CTX_WORDS` in sha256.h
const CTX_WORDS: usize = 16;
const MAGIC: u64 = 0x5351_534F_5348_4132; // "SQSOSHA2"

/// `Sha256Ctx_t`
#[repr(C)]
pub struct Sha256Ctx {
    magic: u64,
    state: [u64; CTX_WORDS - 1],
}

const _: () = assert!(size_of::<Sha256>() <= size_of::<[u64; CTX_WORDS - 1]>());
const _: () = assert!(align_of::<Sha256>() <= align_of::<u64>());

fn state_ptr(ctx: &mut Sha256Ctx) -> *mut Sha256 {
    ctx.state.as_mut_ptr().cast()
}

/// Start a computation
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sha256_init(ctx: *mut Sha256Ctx) -> i32 {
    // SAFETY: the C contract: ctx points to a Sha256Ctx_t
    let Some(ctx) = (unsafe { ptr::out(ctx) }) else { return rc::NULL_PTR };
    // SAFETY: state is large and aligned enough (asserted above); Sha256 has
    // no drop glue, so overwriting an old value leaks nothing
    unsafe { state_ptr(ctx).write(Sha256::new()) };
    ctx.magic = MAGIC;
    rc::OK
}

/// Absorb bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sha256_update(ctx: *mut Sha256Ctx, data: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract
    let Some(ctx) = (unsafe { ptr::out(ctx) }) else { return rc::NULL_PTR };
    // SAFETY: the C contract: data points to len bytes (NULL only if len is 0)
    let data = match unsafe { ptr::bytes(data, len) } {
        Some(d) => d,
        None if len == 0 => &[],
        None => return rc::NULL_PTR,
    };
    if ctx.magic != MAGIC {
        return rc::INVALID;
    }
    // SAFETY: magic set means sha256_init wrote a Sha256 there
    unsafe { (*state_ptr(ctx)).update(data) };
    rc::OK
}

/// Finish and write the digest; the context must be initialized again
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sha256_final(ctx: *mut Sha256Ctx, digest: *mut [u8; 32]) -> i32 {
    // SAFETY: the C contract
    let (Some(ctx), Some(digest)) = (unsafe { ptr::out(ctx) }, unsafe { ptr::out(digest) }) else {
        return rc::NULL_PTR;
    };
    if ctx.magic != MAGIC {
        return rc::INVALID;
    }
    ctx.magic = 0;
    // SAFETY: magic was set, so a Sha256 is there; it is moved out once and
    // the magic is cleared so it is not read again
    let hasher = unsafe { state_ptr(ctx).read() };
    *digest = hasher.finalize().into();
    rc::OK
}

/// One-shot SHA-256
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sha256(data: *const u8, len: usize, digest: *mut [u8; 32]) -> i32 {
    // SAFETY: the C contract
    let Some(digest) = (unsafe { ptr::out(digest) }) else { return rc::NULL_PTR };
    // SAFETY: the C contract
    let data = match unsafe { ptr::bytes(data, len) } {
        Some(d) => d,
        None if len == 0 => &[],
        None => return rc::NULL_PTR,
    };
    *digest = Sha256::digest(data).into();
    rc::OK
}

/// HMAC-SHA-256 (RFC 2104)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hmac_sha256(
    key: *const u8,
    key_len: usize,
    msg: *const u8,
    msg_len: usize,
    mac: *mut [u8; 32],
) -> i32 {
    // SAFETY: the C contract
    let Some(mac) = (unsafe { ptr::out(mac) }) else { return rc::NULL_PTR };
    // SAFETY: the C contract
    let Some(key) = (unsafe { ptr::bytes(key, key_len) }) else { return rc::NULL_PTR };
    // SAFETY: the C contract
    let msg = match unsafe { ptr::bytes(msg, msg_len) } {
        Some(m) => m,
        None if msg_len == 0 => &[],
        None => return rc::NULL_PTR,
    };
    match sqso_protocol::auth::hmac_sha256(key, msg) {
        Some(out) => {
            *mac = out;
            rc::OK
        }
        None => rc::ERROR,
    }
}
