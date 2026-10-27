//! Power-fail-safe persistent record: two slots, a sequence number, CRC-32.
//!
//! A record occupies two slots of [`SLOT_SIZE`] bytes (A at the base, B
//! after it). Each save writes the slot that does not hold the newest valid
//! copy and reads it back, so an interrupted write always leaves the
//! previous copy intact. Load returns the valid copy with the newest
//! sequence number (serial-number arithmetic, so it survives wrap-around).
//!
//! Slot: `magic (4) | sequence (4) | payload (len) | CRC-32 (4)`, all
//! big-endian, CRC over magic, sequence and payload.
//!
//! @requirement SRS-F061 Protect persisted data with CRC32

use sqso_protocol::checksum::crc32;

/// Bytes reserved for one slot
pub const SLOT_SIZE: usize = 64;
/// Magic, sequence and CRC
pub const OVERHEAD: usize = 12;
/// Largest payload
pub const MAX_PAYLOAD: usize = SLOT_SIZE - OVERHEAD;
/// Bytes one record (both slots) occupies
pub const FOOTPRINT: u32 = 128;
const _: () = assert!(FOOTPRINT as usize == 2 * SLOT_SIZE);

const OFFSET_PAYLOAD: usize = 8;

/// A flash driver error, carrying the driver's result code
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlashError(pub i32);

/// Non-volatile memory, by region and byte offset
pub trait Flash {
    /// Read `buf.len()` bytes
    fn read(&mut self, region: u32, offset: u32, buf: &mut [u8]) -> Result<(), FlashError>;
    /// Write `data`
    fn write(&mut self, region: u32, offset: u32, data: &[u8]) -> Result<(), FlashError>;
    /// Erase a whole region
    fn erase(&mut self, region: u32) -> Result<(), FlashError>;
    /// Read `buf.len()` bytes from the second FRAM, the mirror every write
    /// also reaches (SQSO-ICX-001 D §6 row 32, RID SMA-B10); none by default
    fn read_mirror(&mut self, region: u32, offset: u32, buf: &mut [u8]) -> Result<(), FlashError> {
        let _ = (region, offset, buf);
        Err(FlashError(-1))
    }
}

/// The mirror seen as a flash of its own (reads only: [`load`] and
/// [`save`] read through it when the first FRAM fails)
struct Mirror<'a, F: Flash>(&'a mut F);

impl<F: Flash> Flash for Mirror<'_, F> {
    fn read(&mut self, region: u32, offset: u32, buf: &mut [u8]) -> Result<(), FlashError> {
        self.0.read_mirror(region, offset, buf)
    }

    fn write(&mut self, _region: u32, _offset: u32, _data: &[u8]) -> Result<(), FlashError> {
        Err(FlashError(-1))
    }

    fn erase(&mut self, _region: u32) -> Result<(), FlashError> {
        Err(FlashError(-1))
    }
}

/// What a load found
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordStatus {
    /// A valid copy, returned
    Valid,
    /// Neither slot carries the magic (never written)
    Blank,
    /// Magic present but no slot passes its CRC
    Corrupt,
}

/// Why a load or save failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NvmError {
    /// Payload length 0 or above [`MAX_PAYLOAD`]
    Invalid,
    /// The flash driver failed
    Flash(FlashError),
    /// The written slot did not read back
    Verify,
}

impl From<FlashError> for NvmError {
    fn from(e: FlashError) -> Self {
        NvmError::Flash(e)
    }
}

#[derive(Clone, Copy, Default)]
struct SlotInfo {
    valid: bool,
    magic_present: bool,
    sequence: u32,
}

fn be(bytes: &[u8]) -> u32 {
    let mut word = [0u8; 4];
    for (dst, src) in word.iter_mut().zip(bytes) {
        *dst = *src;
    }
    u32::from_be_bytes(word)
}

/// True if sequence `a` is newer than `b`
fn newer(a: u32, b: u32) -> bool {
    // Serial-number arithmetic: a is newer when (a - b) mod 2^32 is in the lower half
    a.wrapping_sub(b) != 0 && a.wrapping_sub(b) < 0x8000_0000
}

fn slot_offset(base: u32, slot: usize) -> Result<u32, NvmError> {
    let step = u32::try_from(slot * SLOT_SIZE).map_err(|_| NvmError::Invalid)?;
    base.checked_add(step).ok_or(NvmError::Invalid)
}

fn read_slot<F: Flash>(
    flash: &mut F,
    region: u32,
    base: u32,
    slot: usize,
    magic: u32,
    len: usize,
    buf: &mut [u8; SLOT_SIZE],
) -> Result<SlotInfo, NvmError> {
    let record_len = OFFSET_PAYLOAD + len + 4;
    let record = buf.get_mut(..record_len).ok_or(NvmError::Invalid)?;
    flash.read(region, slot_offset(base, slot)?, record)?;
    let (body, crc) = record.split_at(OFFSET_PAYLOAD + len);
    let magic_present = be(body) == magic;
    let valid = magic_present && crc32(body) == be(crc);
    Ok(SlotInfo { valid, magic_present, sequence: if valid { be(body.get(4..).unwrap_or(&[])) } else { 0 } })
}

/// Index of the newest valid slot
fn newest(info: &[SlotInfo; 2]) -> Option<usize> {
    match (info[0].valid, info[1].valid) {
        (true, true) => Some(if newer(info[1].sequence, info[0].sequence) { 1 } else { 0 }),
        (true, false) => Some(0),
        (false, true) => Some(1),
        (false, false) => None,
    }
}

fn read_both<F: Flash>(
    flash: &mut F,
    region: u32,
    base: u32,
    magic: u32,
    len: usize,
    bufs: &mut [[u8; SLOT_SIZE]; 2],
) -> Result<[SlotInfo; 2], NvmError> {
    if len == 0 || len > MAX_PAYLOAD {
        return Err(NvmError::Invalid);
    }
    let [a, b] = bufs;
    Ok([read_slot(flash, region, base, 0, magic, len, a)?, read_slot(flash, region, base, 1, magic, len, b)?])
}

/// Load the newest valid copy into `payload` (written only when `Valid`);
/// from the mirror when the first FRAM gives no valid copy (it fails or
/// fails its CRC) and the mirror does
pub fn load<F: Flash>(
    flash: &mut F,
    region: u32,
    base: u32,
    magic: u32,
    payload: &mut [u8],
) -> Result<RecordStatus, NvmError> {
    match load_one(flash, region, base, magic, payload) {
        Ok(RecordStatus::Valid) => Ok(RecordStatus::Valid),
        first => match load_one(&mut Mirror(flash), region, base, magic, payload) {
            Ok(RecordStatus::Valid) => Ok(RecordStatus::Valid),
            _ => first,
        },
    }
}

fn load_one<F: Flash>(
    flash: &mut F,
    region: u32,
    base: u32,
    magic: u32,
    payload: &mut [u8],
) -> Result<RecordStatus, NvmError> {
    let mut bufs = [[0u8; SLOT_SIZE]; 2];
    let info = read_both(flash, region, base, magic, payload.len(), &mut bufs)?;
    let status = match newest(&info) {
        Some(slot) => {
            let src = bufs
                .get(slot)
                .and_then(|b| b.get(OFFSET_PAYLOAD..OFFSET_PAYLOAD + payload.len()))
                .ok_or(NvmError::Invalid)?;
            payload.copy_from_slice(src);
            RecordStatus::Valid
        }
        None if info[0].magic_present || info[1].magic_present => RecordStatus::Corrupt,
        None => RecordStatus::Blank,
    };
    zeroize::Zeroize::zeroize(&mut bufs);
    Ok(status)
}

/// Save into the slot not holding the newest valid copy, then read it back
/// (the slots as the first FRAM holds them, or the mirror if that cannot be
/// read; the write reaches both, the read-back either)
pub fn save<F: Flash>(flash: &mut F, region: u32, base: u32, magic: u32, payload: &[u8]) -> Result<(), NvmError> {
    let mut bufs = [[0u8; SLOT_SIZE]; 2];
    let info = match read_both(flash, region, base, magic, payload.len(), &mut bufs) {
        Ok(info) => info,
        Err(NvmError::Invalid) => return Err(NvmError::Invalid),
        Err(e) => read_both(&mut Mirror(flash), region, base, magic, payload.len(), &mut bufs).map_err(|_| e)?,
    };
    zeroize::Zeroize::zeroize(&mut bufs);
    let newest = newest(&info);
    let target = if newest == Some(0) { 1 } else { 0 };
    let sequence = newest.and_then(|s| info.get(s)).map_or(1, |i| i.sequence.wrapping_add(1));

    let record_len = OFFSET_PAYLOAD + payload.len() + 4;
    let mut record = [0u8; SLOT_SIZE];
    {
        let (body, rest) = record.split_at_mut(OFFSET_PAYLOAD + payload.len());
        let (head, data) = body.split_at_mut(OFFSET_PAYLOAD);
        let (m, s) = head.split_at_mut(4);
        m.copy_from_slice(&magic.to_be_bytes());
        s.copy_from_slice(&sequence.to_be_bytes());
        data.copy_from_slice(payload);
        let crc = crc32(body).to_be_bytes();
        rest.get_mut(..4).ok_or(NvmError::Invalid)?.copy_from_slice(&crc);
    }
    let record = record.get(..record_len).ok_or(NvmError::Invalid)?;
    let offset = slot_offset(base, target)?;
    flash.write(region, offset, record)?;

    let mut readback = [0u8; SLOT_SIZE];
    let readback = readback.get_mut(..record_len).ok_or(NvmError::Invalid)?;
    let ok = (flash.read(region, offset, readback).is_ok() && readback == record)
        || (flash.read_mirror(region, offset, readback).is_ok() && readback == record);
    zeroize::Zeroize::zeroize(readback);
    if ok { Ok(()) } else { Err(NvmError::Verify) }
}

/// In-memory flash for tests and fuzzing (erased bytes read as 0xFF)
#[derive(Clone)]
pub struct MemFlash<const N: usize> {
    /// Region contents
    pub bytes: [u8; N],
    /// Fail every write after this many (`None`: never)
    pub writes_left: Option<usize>,
}

impl<const N: usize> MemFlash<N> {
    /// Erased flash
    pub const fn new() -> Self {
        MemFlash { bytes: [0xFF; N], writes_left: None }
    }

    fn span(&mut self, offset: u32, len: usize) -> Result<&mut [u8], FlashError> {
        let start = usize::try_from(offset).map_err(|_| FlashError(-3))?;
        let end = start.checked_add(len).ok_or(FlashError(-3))?;
        self.bytes.get_mut(start..end).ok_or(FlashError(-3))
    }
}

impl<const N: usize> Default for MemFlash<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Flash for MemFlash<N> {
    fn read(&mut self, _region: u32, offset: u32, buf: &mut [u8]) -> Result<(), FlashError> {
        buf.copy_from_slice(self.span(offset, buf.len())?);
        Ok(())
    }

    fn write(&mut self, _region: u32, offset: u32, data: &[u8]) -> Result<(), FlashError> {
        match self.writes_left {
            Some(0) => return Err(FlashError(-7)),
            Some(ref mut n) => *n -= 1,
            None => {}
        }
        self.span(offset, data.len())?.copy_from_slice(data);
        Ok(())
    }

    fn erase(&mut self, _region: u32) -> Result<(), FlashError> {
        self.bytes = [0xFF; N];
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAGIC: u32 = 0x5351_5453;

    #[test]
    fn blank_then_round_trip_alternating_slots() {
        let mut f = MemFlash::<256>::new();
        let mut out = [0u8; 8];
        assert_eq!(load(&mut f, 0, 0, MAGIC, &mut out), Ok(RecordStatus::Blank));
        for i in 0..5u8 {
            save(&mut f, 0, 0, MAGIC, &[i; 8]).unwrap();
            assert_eq!(load(&mut f, 0, 0, MAGIC, &mut out), Ok(RecordStatus::Valid));
            assert_eq!(out, [i; 8]);
        }
    }

    #[test]
    fn torn_write_keeps_the_previous_copy() {
        let mut f = MemFlash::<256>::new();
        save(&mut f, 0, 0, MAGIC, &[1; 8]).unwrap();
        save(&mut f, 0, 0, MAGIC, &[2; 8]).unwrap();
        // Corrupt the newest copy (slot B)
        f.bytes[SLOT_SIZE + 9] ^= 0xFF;
        let mut out = [0u8; 8];
        assert_eq!(load(&mut f, 0, 0, MAGIC, &mut out), Ok(RecordStatus::Valid));
        assert_eq!(out, [1; 8]);
        f.bytes[9] ^= 0xFF;
        assert_eq!(load(&mut f, 0, 0, MAGIC, &mut out), Ok(RecordStatus::Corrupt));
    }

    #[test]
    fn sequence_wrap_is_newer() {
        assert!(newer(0, u32::MAX));
        assert!(newer(5, 4));
        assert!(!newer(4, 5));
        assert!(!newer(4, 4));
    }

    #[test]
    fn invalid_lengths_and_write_failure() {
        let mut f = MemFlash::<256>::new();
        assert_eq!(save(&mut f, 0, 0, MAGIC, &[]), Err(NvmError::Invalid));
        assert_eq!(save(&mut f, 0, 0, MAGIC, &[0; MAX_PAYLOAD + 1]), Err(NvmError::Invalid));
        assert!(save(&mut f, 0, 0, MAGIC, &[0; MAX_PAYLOAD]).is_ok());
        f.writes_left = Some(0);
        assert_eq!(save(&mut f, 0, 0, MAGIC, &[0; 4]), Err(NvmError::Flash(FlashError(-7))));
    }

    /// Mutation check: exactly half the sequence space apart, neither copy
    /// is newer (so load keeps slot A)
    #[test]
    fn half_range_is_not_newer() {
        assert!(!newer(0x8000_0000, 0));
        assert!(!newer(0, 0x8000_0000));
        assert!(newer(0x7FFF_FFFF, 0));
        let a = SlotInfo { valid: true, magic_present: true, sequence: 0 };
        let b = SlotInfo { valid: true, magic_present: true, sequence: 0x8000_0000 };
        assert_eq!(newest(&[a, b]), Some(0));
    }

    #[test]
    fn mem_flash_out_of_range() {
        let mut f = MemFlash::<16>::new();
        let mut buf = [0u8; 4];
        assert_eq!(f.read(0, 13, &mut buf), Err(FlashError(-3)));
        assert_eq!(f.write(0, 13, &buf), Err(FlashError(-3)));
        assert_eq!(f.read(0, 12, &mut buf), Ok(()));
        assert_eq!(buf, [0xFF; 4]);
    }
}
