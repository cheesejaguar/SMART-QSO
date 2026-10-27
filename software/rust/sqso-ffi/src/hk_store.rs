//! `hk_store.h`: stored housekeeping (`sqso_fsw::hk_store`) in the FRAM
//! regions `FLASH_REGION_HK_SUMMARY`, `FLASH_REGION_HK_DETAIL` and
//! `FLASH_REGION_HK_INDEX` (`hardware/obc/STORAGE_DESIGN.md` §3.3: the
//! 64 KB telemetry buffer and the extended storage).
//!
//! The executive gives the latest housekeeping values each pass
//! ([`set_sample`]); [`hk_store_step`] writes the records that are due.
//! A dump frame is peeked, handed to the downlink, and committed only
//! once the downlink took it, so a busy downlink delays a frame and loses
//! nothing.
//!
//! @requirement SRS-F120 Five-minute housekeeping records for at least 7 days in FRAM
//! @requirement SRS-F121 Sixty-second records for the 6 h around each fault and mode change
//! @requirement SRS-F122 DUMP_TLM downlinks a range of stored records within the downlink duty limit

use sqso_fsw::hk_store::{DUMP_INFO_MAX, Dump, Extra, RECORD_LEN, Store, StoreId};
use sqso_fsw::housekeeping::Housekeeping;

use crate::c::{HalFlash, rc};
use crate::cell::Global;
use crate::ptr;

/// `FLASH_REGION_HK_SUMMARY`
pub(crate) const FLASH_REGION_HK_SUMMARY: u32 = 12;
/// `FLASH_REGION_HK_DETAIL`
pub(crate) const FLASH_REGION_HK_DETAIL: u32 = 13;
/// `FLASH_REGION_HK_INDEX`
pub(crate) const FLASH_REGION_HK_INDEX: u32 = 14;

struct State {
    store: Store,
    sample: Housekeeping,
    extra: Extra,
    /// The dump as it stands after the frame last peeked
    peeked: Option<Dump>,
}

static HK: Global<State> = Global::new(State {
    store: Store::new(FLASH_REGION_HK_SUMMARY, FLASH_REGION_HK_DETAIL, FLASH_REGION_HK_INDEX),
    sample: Housekeeping {
        seq: 0,
        uptime_s: 0,
        boots: 0,
        reset_cause: 0,
        mode: 0,
        hold: false,
        xpdr: false,
        tx_permitted: false,
        soc: None,
        vbatt_v: None,
        ibatt_a: None,
        array_w: None,
        tbatt_c: None,
        tobc_c: None,
        tpa_c: None,
        tjet_c: None,
        deploy: (0, None, false),
        rf_silence: (0, 0),
        contact_timeout_d: 0,
        since_contact_s: 0,
        fault_count: 0,
        last_fault: None,
        rate_cdps: [0; 3],
        adcs_mode: 0,
        mag_rejected: 0,
        gnss_fix: None,
        gnss_sessions: (0, 0),
        passivation: 0,
        eol: (None, 0),
        rx: (0, false),
        contacts: 0,
        stored: [0; 2],
    },
    extra: Extra { heater: false, payload: false, contact_inhibit: false, passivated: false },
    peeked: None,
});

/// The values the next records carry (the executive, every pass)
pub(crate) fn set_sample(hk: &Housekeeping, extra: &Extra) {
    let _ = HK.with(|s| {
        s.sample = *hk;
        s.extra = *extra;
    });
}

/// Whether a record is due at `now_ms` (busy: not now)
pub(crate) fn due(now_ms: u64) -> bool {
    HK.with(|s| s.store.due(now_ms)).unwrap_or(false)
}

/// Find the newest records and load the window table (boot)
#[unsafe(no_mangle)]
pub extern "C" fn hk_store_init() -> i32 {
    let now = crate::c::now_ms();
    HK.with(|s| {
        s.store.init(now, &mut HalFlash);
        s.peeked = None;
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Write the records due at `now_ms`
#[unsafe(no_mangle)]
pub extern "C" fn hk_store_step(now_ms: u64) {
    let _ = HK.with(|s| {
        let (hk, extra) = (s.sample, s.extra);
        s.store.step(now_ms, &hk, &extra, &mut HalFlash);
    });
}

/// A mode change (1) or a fault at ERROR or worse (2)
#[unsafe(no_mangle)]
pub extern "C" fn hk_store_note_event(kind: u8) {
    let _ = HK.with(|s| s.store.note_event(kind));
}

/// Records written so far: the next sequence number of each store
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hk_store_counts(summary: *mut u32, detail: *mut u32) -> i32 {
    // SAFETY: the C contract: NULL or a live uint32_t each
    let (Some(summary), Some(detail)) = (unsafe { ptr::out(summary) }, unsafe { ptr::out(detail) }) else {
        return rc::NULL_PTR;
    };
    match HK.with(|s| s.store.counts()) {
        Some([a, b]) => {
            *summary = a;
            *detail = b;
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Read stored record `seq` of `store` into `out` (44 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hk_store_read(store: u8, seq: u32, out: *mut u8, len: usize) -> i32 {
    // SAFETY: the C contract: out has len writable bytes
    let Some(out) = (unsafe { ptr::bytes_mut(out, len) }) else { return rc::NULL_PTR };
    let Some(store) = StoreId::from_raw(store) else { return rc::INVALID };
    if out.len() < RECORD_LEN {
        return rc::INVALID;
    }
    match HK.with(|s| s.store.read(store, seq, &mut HalFlash)) {
        Some(Some(r)) => {
            for (dst, src) in out.iter_mut().zip(r) {
                *dst = src;
            }
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// DUMP_TLM: start a dump of `count` records from `first` (0 stops one)
#[unsafe(no_mangle)]
pub extern "C" fn hk_store_dump(store: u8, first: u32, count: u16) -> i32 {
    match HK.with(|s| {
        s.peeked = None;
        s.store.start_dump(store, first, count)
    }) {
        Some(Ok(())) => rc::OK,
        Some(Err(_)) => rc::INVALID,
        None => rc::BUSY,
    }
}

/// Stop the dump in progress (the downlink refused it as not permitted)
#[unsafe(no_mangle)]
pub extern "C" fn hk_store_dump_stop() {
    let _ = HK.with(|s| {
        s.store.stop_dump();
        s.peeked = None;
    });
}

/// Whether a dump is in progress
#[unsafe(no_mangle)]
pub extern "C" fn hk_store_dump_active() -> bool {
    HK.with(|s| s.store.dump().is_some()).unwrap_or(false)
}

/// The next dump frame's information field, if one is due at `now_ms`:
/// its length in `out` (0: none due, or `max` too small)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hk_store_dump_peek(now_ms: u64, out: *mut u8, max: usize) -> usize {
    // SAFETY: the C contract: out has max writable bytes
    let Some(out) = (unsafe { ptr::bytes_mut(out, max) }) else { return 0 };
    HK.with(|s| {
        if !s.store.dump_due(now_ms) {
            return 0;
        }
        let mut info = [0u8; DUMP_INFO_MAX];
        match s.store.dump_frame(&mut info, &mut HalFlash) {
            Some((n, after)) if n <= out.len() => {
                for (dst, src) in out.iter_mut().zip(info.iter().take(n)) {
                    *dst = *src;
                }
                s.peeked = Some(after);
                n
            }
            _ => 0,
        }
    })
    .unwrap_or(0)
}

/// The frame last peeked went out at `now_ms`
#[unsafe(no_mangle)]
pub extern "C" fn hk_store_dump_commit(now_ms: u64) {
    let _ = HK.with(|s| {
        if let Some(after) = s.peeked.take() {
            s.store.dump_sent(after, now_ms);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_and_bad_arguments() {
        let mut a = 0u32;
        let mut buf = [0u8; 64];
        let nested = HK.with(|_| {
            // SAFETY: live locals
            unsafe {
                (
                    hk_store_init(),
                    hk_store_counts(&mut a, &mut a),
                    hk_store_read(0, 0, buf.as_mut_ptr(), buf.len()),
                    hk_store_dump(0, 0, 1),
                    hk_store_dump_peek(0, buf.as_mut_ptr(), buf.len()),
                    hk_store_dump_active(),
                )
            }
        });
        assert_eq!(nested, Some((rc::BUSY, rc::BUSY, rc::BUSY, rc::BUSY, 0, false)));
        let _ = HK.with(|_| {
            hk_store_step(0);
            hk_store_note_event(1);
            hk_store_dump_stop();
            hk_store_dump_commit(0);
            set_sample(&Housekeeping::default(), &Extra::default());
            assert!(!due(u64::MAX));
        });
        // SAFETY: NULL pointers and short buffers are refused
        unsafe {
            assert_eq!(hk_store_counts(core::ptr::null_mut(), &mut a), rc::NULL_PTR);
            assert_eq!(hk_store_read(0, 0, core::ptr::null_mut(), 44), rc::NULL_PTR);
            assert_eq!(hk_store_read(2, 0, buf.as_mut_ptr(), 44), rc::INVALID);
            assert_eq!(hk_store_read(0, 0, buf.as_mut_ptr(), 10), rc::INVALID);
            assert_eq!(hk_store_dump_peek(0, core::ptr::null_mut(), 10), 0);
        }
    }
}
