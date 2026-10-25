//! Mission data (`mission_data.h`): statistics kept across resets, stored
//! as one record protected by a CRC-32.
//!
//! [`MissionData`] has the C `MissionData_t` layout with its padding as
//! named fields; the stored bytes are that layout in native byte order, as
//! the C module wrote them.
//!
//! @requirement SRS-F060 Persist mission data to non-volatile memory
//! @requirement SRS-F061 Protect persisted data with CRC32
//! @requirement SRS-F063 Maintain mission statistics across resets

use sqso_protocol::checksum::crc32;

/// Last-fault description bytes including the NUL (`SMART_QSO_FAULT_DESC_LEN`)
pub const FAULT_DESC_LEN: usize = 64;
/// Bytes of the stored record (`sizeof(MissionData_t)`)
pub const RECORD_SIZE: usize = 120;
/// Offset of the CRC: it covers every byte before it
const CRC_OFFSET: usize = 112;

/// Mission phase (`MissionPhase_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Phase {
    /// Initial commissioning
    Commissioning = 0,
    /// Idle operations
    Idle = 1,
    /// Active operations
    Active = 2,
    /// End of life
    Eol = 3,
}

impl Phase {
    /// A phase from its C value
    pub fn from_raw(raw: u32) -> Option<Phase> {
        match raw {
            0 => Some(Phase::Commissioning),
            1 => Some(Phase::Idle),
            2 => Some(Phase::Active),
            3 => Some(Phase::Eol),
            _ => None,
        }
    }
}

/// Mission statistics (the C `MissionData_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MissionData {
    /// Mission start, ms
    pub mission_start_ms: u64,
    /// Total uptime across resets, ms
    pub total_uptime_ms: u64,
    /// Number of resets
    pub reset_count: u32,
    pad0: [u8; 4],
    /// Last reset, ms
    pub last_reset_ms: u64,
    /// Total energy consumed, Wh
    pub total_energy_wh: f64,
    /// Total fault occurrences
    pub fault_count: u32,
    /// Current mission phase (`MissionPhase_t` value)
    pub mission_phase: u8,
    /// Description of the last fault, NUL-terminated
    pub last_fault: [u8; FAULT_DESC_LEN],
    pad1: [u8; 3],
    /// CRC-32 of every byte before this field
    pub crc32: u32,
    pad2: [u8; 4],
}

const _: () = assert!(core::mem::size_of::<MissionData>() == RECORD_SIZE);

/// The stored record's CRC does not match
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrcMismatch;

impl MissionData {
    /// All zero (the state before anything is loaded or initialized)
    pub const ZERO: MissionData = MissionData {
        mission_start_ms: 0,
        total_uptime_ms: 0,
        reset_count: 0,
        pad0: [0; 4],
        last_reset_ms: 0,
        total_energy_wh: 0.0,
        fault_count: 0,
        mission_phase: 0,
        last_fault: [0; FAULT_DESC_LEN],
        pad1: [0; 3],
        crc32: 0,
        pad2: [0; 4],
    };

    /// A first boot: one reset, IDLE, "System startup"; the other fields
    /// (the padding) are kept
    pub fn start_new(&mut self, now_ms: u64) {
        self.mission_start_ms = now_ms;
        self.total_uptime_ms = 0;
        self.reset_count = 1;
        self.last_reset_ms = now_ms;
        self.total_energy_wh = 0.0;
        self.fault_count = 0;
        self.mission_phase = Phase::Idle as u8;
        self.set_last_fault(b"System startup");
        self.update_crc();
    }

    /// One more reset, at `now_ms`
    pub fn count_reset(&mut self, now_ms: u64) {
        self.reset_count = self.reset_count.wrapping_add(1);
        self.last_reset_ms = now_ms;
        self.update_crc();
    }

    /// One more fault, with its description
    pub fn record_fault(&mut self, description: &[u8]) {
        self.fault_count = self.fault_count.wrapping_add(1);
        self.set_last_fault(description);
        self.update_crc();
    }

    /// `strncpy(last_fault, description, 63)` and a final NUL: the
    /// description up to its first NUL or 63 bytes, the rest zeroed
    fn set_last_fault(&mut self, description: &[u8]) {
        let mut text = description.iter().take_while(|b| **b != 0).take(FAULT_DESC_LEN - 1);
        for dst in &mut self.last_fault {
            *dst = text.next().copied().unwrap_or(0);
        }
    }

    /// The last-fault description up to its NUL
    pub fn last_fault(&self) -> &[u8] {
        let len = self.last_fault.iter().position(|b| *b == 0).unwrap_or(FAULT_DESC_LEN);
        self.last_fault.get(..len).unwrap_or(&[])
    }

    /// The stored bytes (C memory layout, native byte order)
    pub fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut b = [0u8; RECORD_SIZE];
        let parts: [&[u8]; 12] = [
            &self.mission_start_ms.to_ne_bytes(),
            &self.total_uptime_ms.to_ne_bytes(),
            &self.reset_count.to_ne_bytes(),
            &self.pad0,
            &self.last_reset_ms.to_ne_bytes(),
            &self.total_energy_wh.to_ne_bytes(),
            &self.fault_count.to_ne_bytes(),
            core::slice::from_ref(&self.mission_phase),
            &self.last_fault,
            &self.pad1,
            &self.crc32.to_ne_bytes(),
            &self.pad2,
        ];
        let mut at = 0;
        for part in parts {
            if let Some(dst) = b.get_mut(at..at + part.len()) {
                dst.copy_from_slice(part);
            }
            at += part.len();
        }
        b
    }

    /// CRC-32 over the bytes before the CRC field, in memory order
    pub fn compute_crc(&self) -> u32 {
        crc32(self.to_bytes().get(..CRC_OFFSET).unwrap_or(&[]))
    }

    /// Set the CRC from the other fields
    pub fn update_crc(&mut self) {
        self.crc32 = self.compute_crc();
    }

    /// A stored record, if its CRC matches
    pub fn from_bytes(b: &[u8; RECORD_SIZE]) -> Result<MissionData, CrcMismatch> {
        let mut r = Reader { b, at: 0 };
        let d = MissionData {
            mission_start_ms: u64::from_ne_bytes(r.take()),
            total_uptime_ms: u64::from_ne_bytes(r.take()),
            reset_count: u32::from_ne_bytes(r.take()),
            pad0: r.take(),
            last_reset_ms: u64::from_ne_bytes(r.take()),
            total_energy_wh: f64::from_ne_bytes(r.take()),
            fault_count: u32::from_ne_bytes(r.take()),
            mission_phase: u8::from_ne_bytes(r.take()),
            last_fault: r.take(),
            pad1: r.take(),
            crc32: u32::from_ne_bytes(r.take()),
            pad2: r.take(),
        };
        if crc32(b.get(..CRC_OFFSET).unwrap_or(&[])) != d.crc32 {
            return Err(CrcMismatch);
        }
        Ok(d)
    }
}

/// Fixed-size fields of a record, in order
struct Reader<'a> {
    b: &'a [u8; RECORD_SIZE],
    at: usize,
}

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let mut out = [0u8; N];
        if let Some(src) = self.b.get(self.at..self.at + N) {
            out.copy_from_slice(src);
        }
        self.at += N;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_mission_and_resets() {
        let mut d = MissionData::ZERO;
        d.start_new(5000);
        assert_eq!(d.reset_count, 1);
        assert_eq!(d.mission_phase, 1);
        assert_eq!(d.last_fault(), b"System startup");
        assert_eq!(d.crc32, d.compute_crc());
        d.count_reset(6000);
        assert_eq!((d.reset_count, d.last_reset_ms), (2, 6000));
        d.reset_count = u32::MAX;
        d.count_reset(7000);
        assert_eq!(d.reset_count, 0);
    }

    #[test]
    fn fault_description_is_cut_and_zero_filled() {
        let mut d = MissionData::ZERO;
        d.record_fault(&[b'x'; 100]);
        assert_eq!(d.last_fault().len(), 63);
        d.record_fault(b"short\0ignored");
        assert_eq!(d.last_fault(), b"short");
        assert!(d.last_fault[5..].iter().all(|b| *b == 0));
        assert_eq!(d.fault_count, 2);
    }

    #[test]
    fn record_layout_round_trip_and_corruption() {
        let mut d = MissionData::ZERO;
        d.start_new(0x0102_0304_0506_0708);
        d.total_energy_wh = 1.5;
        d.update_crc();
        let b = d.to_bytes();
        assert_eq!(&b[0..8], &0x0102_0304_0506_0708u64.to_ne_bytes());
        assert_eq!(b[44], 1); // mission_phase
        assert_eq!(&b[45..59], b"System startup");
        assert_eq!(&b[112..116], &d.crc32.to_ne_bytes());
        assert_eq!(MissionData::from_bytes(&b), Ok(d));
        let mut bad = b;
        bad[10] = 0xFF;
        assert_eq!(MissionData::from_bytes(&bad), Err(CrcMismatch));
    }

    #[test]
    fn phases() {
        assert_eq!(Phase::from_raw(3), Some(Phase::Eol));
        assert_eq!(Phase::from_raw(4), None);
    }

    #[test]
    fn every_phase_value() {
        for p in [Phase::Commissioning, Phase::Idle, Phase::Active, Phase::Eol] {
            assert_eq!(Phase::from_raw(p as u32), Some(p));
        }
    }
}
