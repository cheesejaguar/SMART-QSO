//! `hal_flash.h`: the non-volatile memory is an FM25V20A FRAM (256 KB) on
//! SPI1 (PG2 SCK / PG3 MISO / PG4 MOSI, CS# PG5, AF5; WP# PE0, HOLD# PE1;
//! SQSO-ICX-001 B §6 row 3, `hardware/obc/STORAGE_DESIGN.md` §3.4). FRAM writes take effect at once
//! and need no erase, so a region "erase" writes 0xFF over it, matching the
//! simulation.
//!
//! The regions sit at the addresses of the memory map, STORAGE_DESIGN §3.3.
//! The fault log takes 9,728 bytes of the event log (the full log, 100
//! entries of 96 bytes); the system state takes boot configuration B and C.
//! The stored housekeeping (`hk_store.h`) takes the 64 KB telemetry buffer
//! (0x10000, the sixty-second records) and 0x20700–0x366FF of the extended
//! storage (the five-minute records), its event-window table at 0x36700;
//! the passivation record is at 0x36800 and the ADCS calibration (gyro
//! alignment, magnetometer bias) at 0x36880.

use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::Spi;
use embassy_stm32::spi::mode::Master;

use crate::cell::Shared;
use crate::rc;

const WREN: u8 = 0x06;
const READ: u8 = 0x03;
const WRITE: u8 = 0x02;
const RDID: u8 = 0x9F;
/// FM25V20A device ID: six continuation bytes (0x7F), manufacturer 0xC2
/// (Ramtron/Cypress), then family/density 0x25 0x08 (RDID, 9 bytes)
const DEVICE_ID: [u8; 9] = [0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0xC2, 0x25, 0x08];
const CAPACITY: u32 = 256 * 1024;

/// (address, size) per `HalFlashRegion_t`
const REGIONS: [(u32, usize); 17] = [
    (0x0_0400, 512),    // MISSION_DATA (mission data A)
    (0x0_0000, 256),    // EPS_CONFIG (boot configuration A)
    (0x2_0000, 512),    // SENSOR_CONFIG (extended storage)
    (0x0_C900, 9728),   // FAULT_LOG (event log)
    (0x2_0200, 1024),   // BACKUP (extended storage)
    (0x0_0100, 512),    // STATE (boot configuration B and C)
    (0x0_C400, 768),    // AUTH (telecommand key table)
    (0x0_0300, 256),    // RF_CONTROL (TX-OFF latch, power level)
    (0x0_C700, 256),    // CONTACT (loss-of-contact records)
    (0x0_C800, 128),    // RF_SILENCE
    (0x0_C880, 128),    // DEPLOYMENT
    (0x2_0600, 128),    // OPS (commissioning hold, transponder enable; extended storage)
    (0x2_0700, 90_112), // HK_SUMMARY (five-minute housekeeping records; extended storage)
    (0x1_0000, 65_536), // HK_DETAIL (sixty-second records: the telemetry buffer)
    (0x3_6700, 256),    // HK_INDEX (event-window table; extended storage)
    (0x3_6800, 128),    // EOL (passivation record; extended storage)
    (0x3_6880, 256),    // ADCS_CAL (gyro alignment, magnetometer bias; extended storage)
];

const _: () = {
    // Every region inside the part, none overlapping
    let mut i = 0;
    while i < REGIONS.len() {
        #[allow(clippy::indexing_slicing)] // i, j < len, at compile time
        {
            let (a, n) = REGIONS[i];
            assert!(a as usize + n <= CAPACITY as usize);
            let mut j = i + 1;
            while j < REGIONS.len() {
                let (b, m) = REGIONS[j];
                assert!(a as usize + n <= b as usize || b as usize + m <= a as usize);
                j += 1;
            }
        }
        i += 1;
    }
};

pub(crate) struct Fram {
    spi: Spi<'static, Blocking, Master>,
    cs: Output<'static>,
    /// WP# and HOLD#, held high (writes enabled, never held)
    _wp: Output<'static>,
    _hold: Output<'static>,
    present: bool,
}

static FRAM: Shared<Fram> = Shared::new();

pub(crate) fn install(
    spi: Spi<'static, Blocking, Master>,
    cs: Output<'static>,
    wp: Output<'static>,
    hold: Output<'static>,
) {
    FRAM.install(Fram { spi, cs, _wp: wp, _hold: hold, present: false });
}

impl Fram {
    /// One transaction: `command` (opcode and address), then `data` out or in
    fn transact(&mut self, command: &[u8], out: &[u8], into: &mut [u8]) -> bool {
        self.cs.set_low();
        let ok = self.spi.blocking_write(command).is_ok()
            && (out.is_empty() || self.spi.blocking_write(out).is_ok())
            && (into.is_empty() || self.spi.blocking_read(into).is_ok());
        self.cs.set_high();
        ok
    }

    fn read(&mut self, addr: u32, into: &mut [u8]) -> bool {
        let [_, a2, a1, a0] = addr.to_be_bytes();
        self.transact(&[READ, a2, a1, a0], &[], into)
    }

    fn write(&mut self, addr: u32, data: &[u8]) -> bool {
        let [_, a2, a1, a0] = addr.to_be_bytes();
        // WREN is cleared by every write, so it precedes each one
        self.transact(&[WREN], &[], &mut []) && self.transact(&[WRITE, a2, a1, a0], data, &mut [])
    }
}

/// The FRAM address range `offset..offset+len` of `region`
fn span(region: u32, offset: u32, len: usize) -> Result<u32, i32> {
    let (base, size) = usize::try_from(region).ok().and_then(|r| REGIONS.get(r)).copied().ok_or(rc::INVALID)?;
    let start = usize::try_from(offset).map_err(|_| rc::INVALID)?;
    if start.checked_add(len).is_none_or(|end| end > size) {
        return Err(rc::INVALID);
    }
    base.checked_add(offset).ok_or(rc::INVALID)
}

fn with_present(f: impl FnOnce(&mut Fram) -> i32) -> i32 {
    FRAM.with(|m| if m.present { f(m) } else { rc::ERROR }).unwrap_or(rc::BUSY)
}

/// Check the FRAM answers with its device ID; the regions are unusable
/// (SMART_QSO_ERROR_IO, so the flight software fails safe) if not
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_init() -> i32 {
    FRAM.with(|m| {
        let mut id = [0u8; 9];
        m.present = m.transact(&[RDID], &[], &mut id) && id == DEVICE_ID;
        if m.present { rc::OK } else { rc::IO }
    })
    .unwrap_or(rc::BUSY)
}

/// Read stored bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_flash_read(region: u32, offset: u32, data: *mut u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len writable bytes
    let Some(data) = (unsafe { rc::bytes_mut(data, len) }) else { return rc::NULL_PTR };
    let addr = match span(region, offset, data.len()) {
        Ok(a) => a,
        Err(e) => return e,
    };
    with_present(|m| if m.read(addr, data) { rc::OK } else { rc::IO })
}

/// Store bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_flash_write(region: u32, offset: u32, data: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len bytes
    let Some(data) = (unsafe { rc::bytes(data, len) }) else { return rc::NULL_PTR };
    let addr = match span(region, offset, data.len()) {
        Ok(a) => a,
        Err(e) => return e,
    };
    with_present(|m| if m.write(addr, data) { rc::OK } else { rc::IO })
}

/// Fill a region with 0xFF
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_erase(region: u32) -> i32 {
    let Some(&(base, size)) = usize::try_from(region).ok().and_then(|r| REGIONS.get(r)) else { return rc::INVALID };
    with_present(|m| {
        let ff = [0xFFu8; 64];
        let mut done = 0usize;
        while done < size {
            let n = (size - done).min(ff.len());
            let addr = base + u32::try_from(done).unwrap_or(0);
            if !m.write(addr, ff.get(..n).unwrap_or(&[])) {
                return rc::IO;
            }
            done += n;
        }
        rc::OK
    })
}

/// A region's size in bytes (0 for an unknown region)
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_region_size(region: u32) -> usize {
    usize::try_from(region).ok().and_then(|r| REGIONS.get(r)).map_or(0, |&(_, n)| n)
}

/// A region's FRAM address
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_region_base(region: u32) -> u32 {
    usize::try_from(region).ok().and_then(|r| REGIONS.get(r)).map_or(0, |&(a, _)| a)
}

/// FRAM writes complete within the SPI transfer
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_busy() -> bool {
    false
}

/// FRAM has no wear to level (10^14 cycles)
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_wear_level(_region: u32) -> u8 {
    0
}
