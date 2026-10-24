//! Fault log (`fault_mgmt.h`): the newest [`MAX_ENTRIES`] faults, each
//! with a CRC-32 so a corrupted entry is discarded when the log is loaded.
//!
//! [`Entry`] has the C `FaultLogEntry_t` layout with its padding as named
//! fields, so every byte of an entry is initialized and the log can be
//! stored as the raw array. It has no invalid bit patterns (the `recovered`
//! flag is a byte, checked at load), so raw bytes can be loaded back into it.
//!
//! @requirement SRS-F040 Detect and log all fault conditions
//! @requirement SRS-F041 Maintain the fault log in non-volatile memory

use sqso_protocol::checksum::Crc32Digest;

/// Entries kept (`SMART_QSO_MAX_FAULT_ENTRIES`); the oldest is dropped
pub const MAX_ENTRIES: usize = 100;
/// Description bytes including the NUL (`SMART_QSO_FAULT_DESC_LEN`)
pub const DESC_LEN: usize = 64;

/// One fault (the C `FaultLogEntry_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    /// Time of the fault, ms
    pub timestamp_ms: u64,
    /// `FaultType_t`
    pub fault_type: u8,
    /// `FaultSeverity_t`
    pub severity: u8,
    /// Description, NUL-terminated
    pub description: [u8; DESC_LEN],
    _pad0: [u8; 6],
    /// Battery state of charge at the fault
    pub soc_at_fault: f64,
    recovered: u8,
    _pad1: [u8; 3],
    /// CRC-32 of every byte before this field
    pub crc32: u32,
}

/// Bytes of one entry
pub const ENTRY_SIZE: usize = 96;
const _: () = assert!(core::mem::size_of::<Entry>() == ENTRY_SIZE);

impl Entry {
    /// An all-zero entry
    pub const ZERO: Entry = Entry {
        timestamp_ms: 0,
        fault_type: 0,
        severity: 0,
        description: [0; DESC_LEN],
        _pad0: [0; 6],
        soc_at_fault: 0.0,
        recovered: 0,
        _pad1: [0; 3],
        crc32: 0,
    };

    /// A new entry; the description is cut at its first NUL or 63 bytes
    pub fn new(timestamp_ms: u64, fault_type: u8, severity: u8, description: &[u8], soc: f64) -> Entry {
        let mut e = Entry { timestamp_ms, fault_type, severity, soc_at_fault: soc, ..Entry::ZERO };
        for (dst, src) in e.description.iter_mut().take(DESC_LEN - 1).zip(description.iter().take_while(|b| **b != 0)) {
            *dst = *src;
        }
        e.crc32 = e.compute_crc();
        e
    }

    /// Whether the fault was recovered
    pub fn recovered(&self) -> bool {
        self.recovered != 0
    }

    /// The description up to its NUL
    pub fn description(&self) -> &[u8] {
        let len = self.description.iter().position(|b| *b == 0).unwrap_or(DESC_LEN);
        self.description.get(..len).unwrap_or(&[])
    }

    /// CRC-32 over the bytes before the CRC field, in memory order
    pub fn compute_crc(&self) -> u32 {
        let mut d = Crc32Digest::new();
        d.update(&self.timestamp_ms.to_ne_bytes());
        d.update(&[self.fault_type, self.severity]);
        d.update(&self.description);
        d.update(&self._pad0);
        d.update(&self.soc_at_fault.to_ne_bytes());
        d.update(&[self.recovered]);
        d.update(&self._pad1);
        d.finalize()
    }

    /// Intact: the CRC matches and the recovered flag is 0 or 1
    pub fn is_intact(&self) -> bool {
        self.recovered <= 1 && self.compute_crc() == self.crc32
    }
}

// The CRC covers the 92 bytes before its field
const _: () = assert!(core::mem::offset_of!(Entry, crc32) == 92);

/// The fault log
#[derive(Clone, Debug)]
pub struct FaultLog {
    entries: [Entry; MAX_ENTRIES],
    count: usize,
}

impl FaultLog {
    /// An empty log
    pub const fn new() -> Self {
        FaultLog { entries: [Entry::ZERO; MAX_ENTRIES], count: 0 }
    }

    /// Entries held
    pub fn len(&self) -> usize {
        self.count
    }

    /// No entries
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Append, dropping the oldest entry if full
    pub fn add(&mut self, entry: Entry) {
        if self.count >= MAX_ENTRIES {
            self.entries.copy_within(1.., 0);
            self.count = MAX_ENTRIES - 1;
        }
        if let Some(slot) = self.entries.get_mut(self.count) {
            *slot = entry;
            self.count += 1;
        }
    }

    /// The entry at `index` (0 is the oldest)
    pub fn get(&self, index: usize) -> Option<&Entry> {
        self.held().get(index)
    }

    /// The newest entry
    pub fn last(&self) -> Option<&Entry> {
        self.held().last()
    }

    /// Mark an entry recovered (its CRC follows)
    pub fn mark_recovered(&mut self, index: usize) -> bool {
        match self.entries.get_mut(..self.count).and_then(|e| e.get_mut(index)) {
            Some(e) => {
                e.recovered = 1;
                e.crc32 = e.compute_crc();
                true
            }
            None => false,
        }
    }

    /// Remove every entry
    pub fn clear(&mut self) {
        *self = FaultLog::new();
    }

    /// The entries held, oldest first
    pub fn held(&self) -> &[Entry] {
        self.entries.get(..self.count).unwrap_or(&[])
    }

    /// All slots, for loading raw stored bytes into (then call [`Self::adopt`])
    pub fn slots_mut(&mut self) -> &mut [Entry; MAX_ENTRIES] {
        &mut self.entries
    }

    /// After raw entries were loaded into the first `loaded` slots: keep the
    /// intact ones, in order. Returns the indices discarded, through `discarded`.
    pub fn adopt(&mut self, loaded: usize, discarded: &mut impl FnMut(usize)) {
        let mut kept = 0;
        for i in 0..loaded.min(MAX_ENTRIES) {
            let Some(entry) = self.entries.get(i).copied() else { break };
            if entry.is_intact() {
                if let Some(slot) = self.entries.get_mut(kept) {
                    *slot = entry;
                }
                kept += 1;
            } else {
                discarded(i);
            }
        }
        for slot in self.entries.iter_mut().skip(kept) {
            *slot = Entry::ZERO;
        }
        self.count = kept;
    }
}

impl Default for FaultLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_drops_the_oldest() {
        let mut log = FaultLog::new();
        for i in 0..(MAX_ENTRIES as u64 + 5) {
            log.add(Entry::new(i, 1, 1, b"x", 0.5));
        }
        assert_eq!(log.len(), MAX_ENTRIES);
        assert_eq!(log.get(0).unwrap().timestamp_ms, 5);
        assert_eq!(log.last().unwrap().timestamp_ms, MAX_ENTRIES as u64 + 4);
    }

    #[test]
    fn description_is_bounded_and_crc_checked() {
        let long = [b'a'; 100];
        let e = Entry::new(1, 2, 3, &long, 0.1);
        assert_eq!(e.description().len(), DESC_LEN - 1);
        assert!(e.is_intact());
        let mut bad = e;
        bad.description[0] = b'b';
        assert!(!bad.is_intact());
        let mut flag = e;
        flag.recovered = 2;
        flag.crc32 = flag.compute_crc();
        assert!(!flag.is_intact(), "a non-boolean flag is corruption");
    }

    #[test]
    fn adopt_keeps_intact_entries_in_order() {
        let mut log = FaultLog::new();
        log.slots_mut()[0] = Entry::new(1, 1, 1, b"a", 0.0);
        log.slots_mut()[1] = Entry::ZERO;
        log.slots_mut()[1].crc32 = 1;
        log.slots_mut()[2] = Entry::new(3, 1, 1, b"c", 0.0);
        let mut dropped = [usize::MAX; 3];
        let mut n = 0;
        log.adopt(3, &mut |i| {
            dropped[n] = i;
            n += 1;
        });
        assert_eq!((log.len(), n, dropped[0]), (2, 1, 1));
        assert_eq!(log.get(1).unwrap().description(), b"c");
        assert!(log.mark_recovered(1));
        assert!(log.get(1).unwrap().recovered() && log.get(1).unwrap().is_intact());
        assert!(!log.mark_recovered(2));
    }

    #[test]
    fn empty_clear_and_recovered_flag() {
        let mut log = FaultLog::new();
        assert!(log.is_empty());
        let e = Entry::new(7, 1, 2, b"fault", 0.5);
        assert!(!e.recovered(), "a new fault is not recovered");
        log.add(e);
        assert!(!log.is_empty());
        assert_eq!(log.len(), 1);
        assert!(!log.get(0).unwrap().recovered());
        log.clear();
        assert!(log.is_empty());
        assert_eq!((log.len(), log.get(0), log.last()), (0, None, None));
        assert!(log.slots_mut().iter().all(|e| *e == Entry::ZERO), "cleared slots are zeroed");
    }
}
