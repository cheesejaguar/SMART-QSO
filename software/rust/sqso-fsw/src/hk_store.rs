//! Stored housekeeping (`hk_store.h`; BASELINE §8 Data; RID DB-01;
//! `docs/environment/DATA_BUDGET.md` §4).
//!
//! Two stores of fixed 44-byte records in FRAM:
//!
//! - **Five-minute records** ([`StoreId::Summary`]): one every
//!   [`SUMMARY_PERIOD_MS`], in a ring of [`SUMMARY_SLOTS`] (7.1 days,
//!   90,112 bytes, region `FLASH_REGION_HK_SUMMARY`).
//! - **Sixty-second records** ([`StoreId::Detail`]): one every
//!   [`DETAIL_PERIOD_MS`], in a ring of [`RING_SLOTS`] (6.8 h), and the
//!   6 h around each event (a mode change, or a fault logged at ERROR or
//!   worse) kept in one of [`WINDOWS`] windows of [`WINDOW_SLOTS`]: the
//!   [`PRE_SLOTS`] records before the event are copied from the ring
//!   (a few each pass), the ones after it are written there as they come.
//!   An event while a window is still filling is inside that window; a
//!   new window takes the place of the oldest. 1,489 slots, 65,516 bytes
//!   (region `FLASH_REGION_HK_DETAIL`); the window table is a two-slot
//!   record (`FLASH_REGION_HK_INDEX`).
//!
//! Every record carries its store, its sequence number and a CRC-32, so a
//! boot finds the newest of each store by reading them all (no head record
//! to wear or lose) and a record half written at a reset reads as missing.
//! Sequence numbers continue across resets.
//!
//! **Dump** (`DUMP_TLM`): a store, the first sequence number and a count
//! (at most [`MAX_DUMP`]). Frames of up to [`DUMP_PER_FRAME`] records go out
//! at most one every [`DUMP_FRAME_INTERVAL_MS`], through the downlink and
//! its duty limit, after the beacon and housekeeping frames; records no
//! longer stored are skipped.
//!
//! Record (little-endian): store id (1, `S` or `D`) | version (1) |
//! sequence (4) | uptime s (4) | boots (2) | mode (1) | flags (1: hold,
//! transponder, TX permitted, heater, payload, event, contact inhibit,
//! passivated) | SOC ‰ (2) | battery mV (2) | battery mA (2, signed) |
//! array mW (2) | battery, OBC, PA, Jetson 0.1 °C (4 × 2, signed) |
//! hours since contact (2) | fault-log entries (2) | newest fault type (1),
//! severity (1) | body rate 0.01 °/s (2) | ADCS mode (1) | deployment state
//! (1) | CRC-32 over the 40 bytes before it (4). Not measured: 0xFFFF
//! (0x8000 signed).
//!
//! @requirement SRS-F120 Five-minute housekeeping records for at least 7 days in FRAM
//! @requirement SRS-F121 Sixty-second records for the 6 h around each fault and mode change
//! @requirement SRS-F122 DUMP_TLM downlinks a range of stored records within the downlink duty limit

use sqso_protocol::checksum::crc32;

use crate::housekeeping::Housekeeping;
use crate::nvm::{self, Flash, RecordStatus};

/// Bytes a record
pub const RECORD_LEN: usize = 44;
/// Bytes a record, as a flash offset
#[allow(clippy::cast_possible_truncation)] // 44
const RECORD_LEN_U32: u32 = RECORD_LEN as u32;
/// Bytes covered by the CRC
const BODY_LEN: usize = RECORD_LEN - 4;
/// Record layout version
pub const VERSION: u8 = 1;
/// Five-minute records
pub const SUMMARY_PERIOD_MS: u64 = 300_000;
/// Sixty-second records
pub const DETAIL_PERIOD_MS: u64 = 60_000;
/// Five-minute slots: 7.1 days
pub const SUMMARY_SLOTS: u32 = 2048;
/// Sixty-second ring slots: 6.8 h, more than the [`PRE_SLOTS`] a window copies
pub const RING_SLOTS: u32 = 409;
/// Event windows
pub const WINDOWS: usize = 3;
/// Records a window: 6 h of 60 s records
pub const WINDOW_SLOTS: u32 = 360;
/// Of those, before the event (3 h)
pub const PRE_SLOTS: u32 = 180;
/// After it, the event's own record included
pub const POST_SLOTS: u32 = WINDOW_SLOTS - PRE_SLOTS;
/// Sixty-second slots in all
#[allow(clippy::cast_possible_truncation)] // 3
pub const DETAIL_SLOTS: u32 = RING_SLOTS + WINDOWS as u32 * WINDOW_SLOTS;
/// The five-minute region's size, bytes
pub const SUMMARY_BYTES: usize = SUMMARY_SLOTS as usize * RECORD_LEN;
/// The sixty-second region's size, bytes
pub const DETAIL_BYTES: usize = DETAIL_SLOTS as usize * RECORD_LEN;
/// Largest DUMP_TLM count
pub const MAX_DUMP: u16 = 600;
/// Records a dump frame
pub const DUMP_PER_FRAME: usize = 4;
/// At most one dump frame this often, ms (with the 60 s beacon, about
/// 22 % of the time keyed at most, under the downlink's 25 %)
pub const DUMP_FRAME_INTERVAL_MS: u64 = 8_000;
/// A dump frame's information field: `HKD` | store | count | records
pub const DUMP_PREFIX: &[u8; 3] = b"HKD";
/// The longest dump information field, bytes
pub const DUMP_INFO_MAX: usize = DUMP_PREFIX.len() + 2 + DUMP_PER_FRAME * RECORD_LEN;
/// Records copied into a new window each pass
pub const COPY_PER_STEP: u32 = 8;
/// Window table record ("HKIX")
pub const INDEX_MAGIC: u32 = 0x484B_4958;
/// Event kind: a mode change
pub const EVENT_MODE: u8 = 1;
/// Event kind: a fault at ERROR or worse
pub const EVENT_FAULT: u8 = 2;

const _: () = assert!(SUMMARY_SLOTS as u64 * SUMMARY_PERIOD_MS >= 7 * 86_400_000);
const _: () = assert!(RING_SLOTS > PRE_SLOTS + COPY_PER_STEP);
const _: () = assert!(DETAIL_BYTES <= 65_536);
const _: () = assert!(DUMP_INFO_MAX <= 256);

/// Flag bits of a record
pub mod flag {
    /// Commissioning hold
    pub const HOLD: u8 = 0x01;
    /// Transponder enabled
    pub const XPDR: u8 = 0x02;
    /// Transmission permitted
    pub const TX: u8 = 0x04;
    /// Battery heater on
    pub const HEATER: u8 = 0x08;
    /// Payload powered
    pub const PAYLOAD: u8 = 0x10;
    /// An event (mode change or fault) since the record before
    pub const EVENT: u8 = 0x20;
    /// Loss-of-contact inhibit
    pub const CONTACT: u8 = 0x40;
    /// Passivated
    pub const PASSIVATED: u8 = 0x80;
}

/// Which store
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreId {
    /// Five-minute records
    Summary = 0,
    /// Sixty-second records
    Detail = 1,
}

impl StoreId {
    /// From the DUMP_TLM argument
    pub const fn from_raw(raw: u8) -> Option<StoreId> {
        match raw {
            0 => Some(StoreId::Summary),
            1 => Some(StoreId::Detail),
            _ => None,
        }
    }

    const fn tag(self) -> u8 {
        match self {
            StoreId::Summary => b'S',
            StoreId::Detail => b'D',
        }
    }
}

/// What a record holds beyond the housekeeping frame's values
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Extra {
    /// Battery heater on
    pub heater: bool,
    /// Payload powered
    pub payload: bool,
    /// Loss-of-contact inhibit in force
    pub contact_inhibit: bool,
    /// Passivated
    pub passivated: bool,
}

fn put(out: &mut [u8], at: usize, bytes: &[u8]) {
    if let Some(dst) = out.get_mut(at..at + bytes.len()) {
        dst.copy_from_slice(bytes);
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped
fn scaled_u16(v: Option<f64>, scale: f64) -> u16 {
    v.filter(|v| v.is_finite()).map_or(u16::MAX, |v| crate::fmath::round(v * scale).clamp(0.0, 65_534.0) as u16)
}

#[allow(clippy::cast_possible_truncation)] // clamped
fn scaled_i16(v: Option<f64>, scale: f64) -> i16 {
    v.filter(|v| v.is_finite()).map_or(i16::MIN, |v| crate::fmath::round(v * scale).clamp(-32_767.0, 32_767.0) as i16)
}

fn sat16(v: u64) -> u16 {
    u16::try_from(v).unwrap_or(u16::MAX)
}

/// Encode one record
pub fn encode(store: StoreId, seq: u32, hk: &Housekeeping, extra: &Extra, event: bool) -> [u8; RECORD_LEN] {
    let mut r = [0u8; RECORD_LEN];
    let mut flags = 0u8;
    for (on, bit) in [
        (hk.hold, flag::HOLD),
        (hk.xpdr, flag::XPDR),
        (hk.tx_permitted, flag::TX),
        (extra.heater, flag::HEATER),
        (extra.payload, flag::PAYLOAD),
        (event, flag::EVENT),
        (extra.contact_inhibit, flag::CONTACT),
        (extra.passivated, flag::PASSIVATED),
    ] {
        if on {
            flags |= bit;
        }
    }
    let rate = hk.rate_cdps.iter().map(|&c| f64::from(c) * f64::from(c)).sum::<f64>();
    let rate = crate::fmath::sqrt(rate);
    put(&mut r, 0, &[store.tag(), VERSION]);
    put(&mut r, 2, &seq.to_le_bytes());
    put(&mut r, 6, &u32::try_from(hk.uptime_s).unwrap_or(u32::MAX).to_le_bytes());
    put(&mut r, 10, &sat16(u64::from(hk.boots)).to_le_bytes());
    put(&mut r, 12, &[u8::try_from(hk.mode).unwrap_or(u8::MAX), flags]);
    put(&mut r, 14, &scaled_u16(hk.soc, 1000.0).to_le_bytes());
    put(&mut r, 16, &scaled_u16(hk.vbatt_v, 1000.0).to_le_bytes());
    put(&mut r, 18, &scaled_i16(hk.ibatt_a, 1000.0).to_le_bytes());
    put(&mut r, 20, &scaled_u16(hk.array_w, 1000.0).to_le_bytes());
    for (k, t) in [hk.tbatt_c, hk.tobc_c, hk.tpa_c, hk.tjet_c].into_iter().enumerate() {
        put(&mut r, 22 + 2 * k, &scaled_i16(t, 10.0).to_le_bytes());
    }
    put(&mut r, 30, &sat16(u64::from(hk.since_contact_s / 3600)).to_le_bytes());
    put(&mut r, 32, &sat16(u64::from(hk.fault_count)).to_le_bytes());
    let (ft, fs) = hk.last_fault.unwrap_or((u8::MAX, u8::MAX));
    put(&mut r, 34, &[ft, fs]);
    put(&mut r, 36, &scaled_u16(Some(rate), 1.0).to_le_bytes());
    put(&mut r, 38, &[hk.adcs_mode, u8::try_from(hk.deploy.0).unwrap_or(u8::MAX)]);
    let crc = crc32(r.get(..BODY_LEN).unwrap_or(&[]));
    put(&mut r, BODY_LEN, &crc.to_be_bytes());
    r
}

/// A stored record's sequence number, if it is a valid record of `store`
pub fn decode_seq(store: StoreId, r: &[u8; RECORD_LEN]) -> Option<u32> {
    let (body, crc) = r.split_at(BODY_LEN);
    let crc = u32::from_be_bytes(crc.try_into().ok()?);
    let ok = crc == crc32(body) && body.first() == Some(&store.tag()) && body.get(1) == Some(&VERSION);
    let seq = body.get(2..6).and_then(|b| b.try_into().ok()).map(u32::from_le_bytes)?;
    ok.then_some(seq)
}

/// An event window
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Window {
    /// Sequence number of the event's record
    pub trigger: u32,
    /// [`EVENT_MODE`] or [`EVENT_FAULT`]
    pub kind: u8,
    /// In use
    pub valid: bool,
}

impl Window {
    /// The first sequence number the window holds
    pub const fn start(&self) -> u32 {
        self.trigger.saturating_sub(PRE_SLOTS)
    }

    /// Whether the window holds (or will hold) record `seq`
    pub const fn holds(&self, seq: u32) -> bool {
        self.valid && seq >= self.start() && seq < self.trigger.saturating_add(POST_SLOTS)
    }
}

/// Why a DUMP_TLM was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DumpError {
    /// Unknown store or a count above [`MAX_DUMP`]
    Invalid,
}

/// A dump in progress
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dump {
    /// The store
    pub store: StoreId,
    /// The next sequence number to send
    pub next: u32,
    /// Sequence numbers left to examine
    pub left: u16,
}

/// The stores
#[derive(Clone, Copy, Debug)]
pub struct Store {
    regions: [u32; 3],
    next: [u32; 2],
    last_ms: [u64; 2],
    windows: [Window; WINDOWS],
    next_window: usize,
    /// Window being filled from the ring: (window, next sequence, end)
    copy: Option<(usize, u32, u32)>,
    pending_event: u8,
    dump: Option<Dump>,
    last_dump_ms: Option<u64>,
    /// Records that could not be written
    pub write_errors: u32,
    initialized: bool,
}

impl Store {
    /// The summary, detail and index regions
    pub const fn new(summary: u32, detail: u32, index: u32) -> Store {
        Store {
            regions: [summary, detail, index],
            next: [0; 2],
            last_ms: [0; 2],
            windows: [Window { trigger: 0, kind: 0, valid: false }; WINDOWS],
            next_window: 0,
            copy: None,
            pending_event: 0,
            dump: None,
            last_dump_ms: None,
            write_errors: 0,
            initialized: false,
        }
    }

    fn slots(store: StoreId) -> u32 {
        match store {
            StoreId::Summary => SUMMARY_SLOTS,
            StoreId::Detail => DETAIL_SLOTS,
        }
    }

    fn region(&self, store: StoreId) -> u32 {
        self.regions.get(store as usize).copied().unwrap_or(u32::MAX)
    }

    fn read_slot<F: Flash>(&self, store: StoreId, slot: u32, flash: &mut F) -> Option<[u8; RECORD_LEN]> {
        let mut r = [0u8; RECORD_LEN];
        let offset = slot.checked_mul(RECORD_LEN_U32)?;
        flash.read(self.region(store), offset, &mut r).ok().map(|()| r)
    }

    fn write_slot<F: Flash>(&mut self, store: StoreId, slot: u32, r: &[u8; RECORD_LEN], flash: &mut F) {
        let offset = slot.saturating_mul(RECORD_LEN_U32);
        if flash.write(self.region(store), offset, r).is_err() {
            self.write_errors = self.write_errors.saturating_add(1);
        }
    }

    fn save_index<F: Flash>(&mut self, flash: &mut F) {
        let mut rec = [0u8; 1 + 6 * WINDOWS];
        if let Some(first) = rec.first_mut() {
            *first = u8::try_from(self.next_window).unwrap_or(0);
        }
        for (k, w) in self.windows.iter().enumerate() {
            put(&mut rec, 1 + 6 * k, &w.trigger.to_le_bytes());
            put(&mut rec, 5 + 6 * k, &[w.kind, u8::from(w.valid)]);
        }
        if nvm::save(flash, self.regions[2], 0, INDEX_MAGIC, &rec).is_err() {
            self.write_errors = self.write_errors.saturating_add(1);
        }
    }

    /// Find the newest record of each store and load the window table
    pub fn init<F: Flash>(&mut self, now_ms: u64, flash: &mut F) {
        let mut rec = [0u8; 1 + 6 * WINDOWS];
        if nvm::load(flash, self.regions[2], 0, INDEX_MAGIC, &mut rec) == Ok(RecordStatus::Valid) {
            self.next_window = usize::from(rec[0]) % WINDOWS;
            for (k, w) in self.windows.iter_mut().enumerate() {
                let at = 1 + 6 * k;
                let trigger = rec.get(at..at + 4).and_then(|b| b.try_into().ok()).map_or(0, u32::from_le_bytes);
                let byte = |n: usize| rec.get(at + n).copied().unwrap_or(0);
                *w = Window { trigger, kind: byte(4), valid: byte(5) == 1 };
            }
        }
        for store in [StoreId::Summary, StoreId::Detail] {
            let mut newest: Option<u32> = None;
            for slot in 0..Store::slots(store) {
                if let Some(seq) = self.read_slot(store, slot, flash).and_then(|r| decode_seq(store, &r)) {
                    newest = Some(newest.map_or(seq, |n| n.max(seq)));
                }
            }
            if let Some(next) = self.next.get_mut(store as usize) {
                *next = newest.map_or(0, |n| n.saturating_add(1));
            }
            if let Some(last) = self.last_ms.get_mut(store as usize) {
                *last = now_ms;
            }
        }
        self.copy = None;
        self.dump = None;
        self.initialized = true;
    }

    /// A mode change or fault: the next sixty-second record carries it
    pub fn note_event(&mut self, kind: u8) {
        if self.pending_event == 0 {
            self.pending_event = kind;
        }
    }

    /// Whether a window is still collecting its records after the event
    fn collecting(&self, seq: u32) -> bool {
        self.windows.iter().any(|w| w.valid && seq >= w.trigger && seq < w.trigger.saturating_add(POST_SLOTS))
    }

    fn window_slot(&self, w: usize, seq: u32) -> Option<u32> {
        let win = self.windows.get(w).filter(|win| win.holds(seq))?;
        let k = u32::try_from(w).ok()?;
        Some(RING_SLOTS + k * WINDOW_SLOTS + (seq - win.start()))
    }

    /// Every pass: the records due, and a few copies into a new window
    pub fn step<F: Flash>(&mut self, now_ms: u64, hk: &Housekeeping, extra: &Extra, flash: &mut F) {
        if !self.initialized {
            return;
        }
        if now_ms.saturating_sub(self.last_ms[0]) >= SUMMARY_PERIOD_MS {
            self.last_ms[0] = now_ms;
            let seq = self.next[0];
            let r = encode(StoreId::Summary, seq, hk, extra, false);
            self.write_slot(StoreId::Summary, seq % SUMMARY_SLOTS, &r, flash);
            self.next[0] = seq.wrapping_add(1);
        }
        if now_ms.saturating_sub(self.last_ms[1]) >= DETAIL_PERIOD_MS {
            self.last_ms[1] = now_ms;
            self.write_detail(hk, extra, flash);
        }
        self.copy_some(flash);
    }

    fn write_detail<F: Flash>(&mut self, hk: &Housekeeping, extra: &Extra, flash: &mut F) {
        let seq = self.next[1];
        let event = core::mem::take(&mut self.pending_event);
        if event != 0 && !self.collecting(seq) {
            let w = self.next_window;
            if let Some(win) = self.windows.get_mut(w) {
                *win = Window { trigger: seq, kind: event, valid: true };
            }
            self.next_window = (w + 1) % WINDOWS;
            self.save_index(flash);
            self.copy = Some((w, seq.saturating_sub(PRE_SLOTS), seq));
        }
        let r = encode(StoreId::Detail, seq, hk, extra, event != 0);
        self.write_slot(StoreId::Detail, seq % RING_SLOTS, &r, flash);
        for w in 0..WINDOWS {
            let after_trigger = self.windows.get(w).is_some_and(|win| win.valid && seq >= win.trigger);
            if after_trigger && let Some(slot) = self.window_slot(w, seq) {
                self.write_slot(StoreId::Detail, slot, &r, flash);
            }
        }
        self.next[1] = seq.wrapping_add(1);
    }

    /// Copy the ring's records before an event into its window
    fn copy_some<F: Flash>(&mut self, flash: &mut F) {
        let Some((w, mut from, end)) = self.copy else { return };
        for _ in 0..COPY_PER_STEP {
            if from >= end {
                break;
            }
            let ring = self.read_slot(StoreId::Detail, from % RING_SLOTS, flash);
            if let (Some(r), Some(slot)) = (ring.filter(|r| decode_seq(StoreId::Detail, r) == Some(from)), self.window_slot(w, from)) {
                self.write_slot(StoreId::Detail, slot, &r, flash);
            }
            from += 1;
        }
        self.copy = (from < end).then_some((w, from, end));
    }

    /// Read stored record `seq` of `store`, if it is still stored
    pub fn read<F: Flash>(&self, store: StoreId, seq: u32, flash: &mut F) -> Option<[u8; RECORD_LEN]> {
        let check = |r: [u8; RECORD_LEN]| (decode_seq(store, &r) == Some(seq)).then_some(r);
        match store {
            StoreId::Summary => self.read_slot(store, seq % SUMMARY_SLOTS, flash).and_then(check),
            StoreId::Detail => self
                .read_slot(store, seq % RING_SLOTS, flash)
                .and_then(check)
                .or_else(|| (0..WINDOWS).find_map(|w| self.window_slot(w, seq)).and_then(|s| self.read_slot(store, s, flash)).and_then(check)),
        }
    }

    /// DUMP_TLM: start a dump (a count of 0 stops one)
    pub fn start_dump(&mut self, store: u8, first: u32, count: u16) -> Result<(), DumpError> {
        let store = StoreId::from_raw(store).ok_or(DumpError::Invalid)?;
        if count > MAX_DUMP {
            return Err(DumpError::Invalid);
        }
        self.dump = (count > 0).then_some(Dump { store, next: first, left: count });
        self.last_dump_ms = None;
        Ok(())
    }

    /// Stop the dump in progress
    pub fn stop_dump(&mut self) {
        self.dump = None;
    }

    /// The dump in progress
    pub const fn dump(&self) -> Option<Dump> {
        self.dump
    }

    /// Whether a dump frame may go now
    pub fn dump_due(&self, now_ms: u64) -> bool {
        self.dump.is_some() && self.last_dump_ms.is_none_or(|t| now_ms.saturating_sub(t) >= DUMP_FRAME_INTERVAL_MS)
    }

    /// The next dump frame's information field in `out` and where the dump
    /// stands after it; nothing is consumed until [`Store::dump_sent`]
    pub fn dump_frame<F: Flash>(&self, out: &mut [u8; DUMP_INFO_MAX], flash: &mut F) -> Option<(usize, Dump)> {
        let mut d = self.dump?;
        out[..3].copy_from_slice(DUMP_PREFIX);
        out[3] = d.store as u8;
        let mut n = 0usize;
        while n < DUMP_PER_FRAME && d.left > 0 {
            if let Some(r) = self.read(d.store, d.next, flash) {
                put(out, 5 + n * RECORD_LEN, &r);
                n += 1;
            }
            d.next = d.next.wrapping_add(1);
            d.left -= 1;
        }
        out[4] = u8::try_from(n).unwrap_or(0);
        Some((5 + n * RECORD_LEN, d))
    }

    /// The frame from [`Store::dump_frame`] went out at `now_ms`
    pub fn dump_sent(&mut self, after: Dump, now_ms: u64) {
        self.dump = (after.left > 0).then_some(after);
        self.last_dump_ms = Some(now_ms);
    }

    /// The next sequence number of each store (records written so far)
    pub const fn counts(&self) -> [u32; 2] {
        self.next
    }

    /// The window table
    pub const fn windows(&self) -> [Window; WINDOWS] {
        self.windows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nvm::FlashError;
    extern crate std;
    use std::vec;
    use std::vec::Vec;

    /// Three regions of RAM
    struct Fram {
        r: [Vec<u8>; 3],
        fail: bool,
    }

    impl Fram {
        fn new() -> Fram {
            Fram { r: [vec![0; SUMMARY_BYTES], vec![0; DETAIL_BYTES], vec![0; 256]], fail: false }
        }
    }

    impl Flash for Fram {
        fn read(&mut self, region: u32, o: u32, b: &mut [u8]) -> Result<(), FlashError> {
            let o = o as usize;
            let src = self.r[region as usize].get(o..o + b.len()).ok_or(FlashError(-2))?;
            b.copy_from_slice(src);
            Ok(())
        }
        fn write(&mut self, region: u32, o: u32, d: &[u8]) -> Result<(), FlashError> {
            if self.fail {
                return Err(FlashError(-7));
            }
            let o = o as usize;
            self.r[region as usize].get_mut(o..o + d.len()).ok_or(FlashError(-2))?.copy_from_slice(d);
            Ok(())
        }
        fn erase(&mut self, region: u32) -> Result<(), FlashError> {
            self.r[region as usize].fill(0xFF);
            Ok(())
        }
    }

    fn hk(up: u64) -> Housekeeping {
        Housekeeping { uptime_s: up, soc: Some(0.9), rate_cdps: [300, 400, 0], ..Housekeeping::default() }
    }

    fn run(s: &mut Store, f: &mut Fram, from_ms: u64, minutes: u64) -> u64 {
        let mut t = from_ms;
        for _ in 0..minutes * 60 {
            t += 1000;
            s.step(t, &hk(t / 1000), &Extra::default(), f);
        }
        t
    }

    #[test]
    fn record_round_trip_and_corruption() {
        let h = Housekeeping {
            ibatt_a: Some(-0.25),
            tbatt_c: Some(-3.2),
            last_fault: Some((4, 3)),
            since_contact_s: 7300,
            ..hk(99)
        };
        let r = encode(StoreId::Detail, 77, &h, &Extra { heater: true, passivated: true, ..Extra::default() }, true);
        assert_eq!(decode_seq(StoreId::Detail, &r), Some(77));
        assert_eq!(decode_seq(StoreId::Summary, &r), None, "the store is in the record");
        assert_eq!(&r[0..2], &[b'D', VERSION]);
        assert_eq!(i16::from_le_bytes([r[18], r[19]]), -250);
        assert_eq!(i16::from_le_bytes([r[22], r[23]]), -32);
        assert_eq!(i16::from_le_bytes([r[24], r[25]]), i16::MIN, "not measured");
        assert_eq!(u16::from_le_bytes([r[36], r[37]]), 500);
        assert_eq!(r[13], flag::HEATER | flag::EVENT | flag::PASSIVATED);
        assert_eq!(u16::from_le_bytes([r[30], r[31]]), 2);
        let mut bad = r;
        bad[20] ^= 1;
        assert_eq!(decode_seq(StoreId::Detail, &bad), None);
        assert_eq!(StoreId::from_raw(2), None);
    }

    #[test]
    fn seven_days_of_five_minute_records_survive_resets() {
        let mut f = Fram::new();
        let mut s = Store::new(0, 1, 2);
        s.init(0, &mut f);
        let t = run(&mut s, &mut f, 0, 60);
        assert_eq!(s.counts(), [12, 60]);
        // A reset: sequence numbers continue
        let mut s2 = Store::new(0, 1, 2);
        s2.init(t, &mut f);
        assert_eq!(s2.counts(), [12, 60]);
        let _ = run(&mut s2, &mut f, t, 5);
        assert_eq!(s2.counts(), [13, 65]);
        assert!(s2.read(StoreId::Summary, 0, &mut f).is_some());
        assert!(s2.read(StoreId::Summary, 13, &mut f).is_none(), "not yet written");
        // Wrap the summary ring: the oldest goes, 2048 stay
        for seq in 13..SUMMARY_SLOTS + 20 {
            let r = encode(StoreId::Summary, seq, &hk(0), &Extra::default(), false);
            f.write(0, (seq % SUMMARY_SLOTS) * RECORD_LEN_U32, &r).unwrap();
        }
        let mut s3 = Store::new(0, 1, 2);
        s3.init(0, &mut f);
        assert_eq!(s3.counts()[0], SUMMARY_SLOTS + 20);
        assert!(s3.read(StoreId::Summary, 19, &mut f).is_none());
        assert!(s3.read(StoreId::Summary, 20, &mut f).is_some());
    }

    #[test]
    fn an_event_keeps_three_hours_either_side() {
        let mut f = Fram::new();
        let mut s = Store::new(0, 1, 2);
        s.init(0, &mut f);
        let t = run(&mut s, &mut f, 0, 5 * 60); // 300 detail records
        s.note_event(EVENT_FAULT);
        s.note_event(EVENT_MODE); // the first kind is kept
        let t = run(&mut s, &mut f, t, 10 * 60); // 600 more: the ring has wrapped
        let w = s.windows()[0];
        assert!(w.valid && w.trigger == 300 && w.kind == EVENT_FAULT);
        for seq in 120..480 {
            assert!(s.read(StoreId::Detail, seq, &mut f).is_some(), "{seq}");
        }
        assert!(s.read(StoreId::Detail, 119, &mut f).is_none());
        assert!(s.read(StoreId::Detail, 480, &mut f).is_none(), "outside the window and the ring");
        // The window table survives a reset
        let mut s2 = Store::new(0, 1, 2);
        s2.init(t, &mut f);
        assert_eq!(s2.windows()[0], w);
        assert!(s2.read(StoreId::Detail, 200, &mut f).is_some());
        // An event inside a filling window opens no new one; later ones rotate
        let t = run(&mut s2, &mut f, t, 1);
        s2.note_event(EVENT_MODE);
        let t = run(&mut s2, &mut f, t, 1);
        s2.note_event(EVENT_MODE);
        let t = run(&mut s2, &mut f, t, 1);
        assert_eq!(s2.windows()[1].trigger, 901);
        assert!(!s2.windows()[2].valid, "inside window 1's filling time");
        let mut t = t;
        for k in 0..3 {
            t = run(&mut s2, &mut f, t, 4 * 60);
            s2.note_event(EVENT_MODE);
            let _ = k;
        }
        let _ = run(&mut s2, &mut f, t, 1);
        assert!(s2.windows().iter().all(|w| w.valid));
        assert!(s2.windows().iter().all(|w| w.trigger != 300), "the oldest was replaced");
    }

    #[test]
    fn dumps_go_in_paced_frames_and_skip_missing_records() {
        let mut f = Fram::new();
        let mut s = Store::new(0, 1, 2);
        s.init(0, &mut f);
        let _ = run(&mut s, &mut f, 0, 30); // 6 summary records
        assert_eq!(s.start_dump(2, 0, 1), Err(DumpError::Invalid));
        assert_eq!(s.start_dump(0, 0, MAX_DUMP + 1), Err(DumpError::Invalid));
        assert!(!s.dump_due(0));
        assert_eq!(s.start_dump(0, 0, 6), Ok(()));
        assert!(s.dump_due(0));
        let mut out = [0u8; DUMP_INFO_MAX];
        let (len, after) = s.dump_frame(&mut out, &mut f).unwrap();
        assert_eq!(&out[..5], b"HKD\x00\x04");
        assert_eq!(len, DUMP_INFO_MAX);
        assert_eq!(&out[5..7], &[b'S', VERSION]);
        assert_eq!(after.left, 2);
        s.dump_sent(after, 1000);
        assert!(!s.dump_due(1000 + DUMP_FRAME_INTERVAL_MS - 1));
        assert!(s.dump_due(1000 + DUMP_FRAME_INTERVAL_MS));
        let (len, after) = s.dump_frame(&mut out, &mut f).unwrap();
        assert_eq!((len, out[4]), (5 + 2 * RECORD_LEN, 2));
        s.dump_sent(after, 9000);
        assert_eq!(s.dump(), None, "done");
        // Records no longer (or not yet) stored are skipped
        assert_eq!(s.start_dump(0, 3, 6), Ok(()));
        let (len, after) = s.dump_frame(&mut out, &mut f).unwrap();
        assert_eq!((len, out[4], after.left), (5 + 3 * RECORD_LEN, 3, 0), "3, 4, 5; 6 to 8 not written");
        assert_eq!(s.start_dump(1, 0, 10), Ok(()));
        assert_eq!(s.start_dump(1, 0, 0), Ok(()));
        assert_eq!(s.dump(), None, "a count of 0 stops it");
        assert_eq!(s.start_dump(1, 0, 10), Ok(()));
        s.stop_dump();
        assert!(s.dump_frame(&mut out, &mut f).is_none());
    }

    #[test]
    fn write_failures_are_counted_and_nothing_runs_before_init() {
        let mut f = Fram::new();
        let mut s = Store::new(0, 1, 2);
        s.step(1_000_000, &hk(0), &Extra::default(), &mut f);
        assert_eq!(s.counts(), [0, 0]);
        s.init(0, &mut f);
        f.fail = true;
        s.note_event(EVENT_MODE);
        let _ = run(&mut s, &mut f, 0, 5);
        assert!(s.write_errors >= 6, "{}", s.write_errors);
    }
}
