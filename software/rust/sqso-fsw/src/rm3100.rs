//! PNI RM3100 magnetometer on board A2 (I2C4, `hal_i2c` bus 1, address
//! 0x20; DRDY on PD11; SQSO-OBC-BRD-001 B §9.4).
//!
//! From the RM3100 user manual (PNI 1017252 R07): cycle counts at 0x04–0x09
//! (MSB first per axis), a single measurement of the three axes started by
//! writing 0x70 to `POLL` (0x00), `DRDY` (and `STATUS` bit 7) high when it
//! is done, the result at `MX` 0x24 to `MZ` 0x2C as 24-bit two's complement
//! counts, `REVID` 0x36 = 0x22. At 200 cycle counts the gain is 75 counts
//! per µT (13 nT resolution, about 15 nT noise) and the three axes take
//! about 7 ms, so a conversion started at the beginning of the coils-off
//! window (950 ms) is read in the same window (ICD-004 §12.2).
//!
//! @requirement SRS-F140 ADCS sensor drivers on the OBC buses

use crate::bus::{BusError, I2cBus};

/// I2C address (SA0 = SA1 = 0)
pub const ADDRESS: u8 = 0x20;
/// `REVID`
pub const REVID: u8 = 0x22;
/// Cycle count on every axis
pub const CYCLE_COUNT: u16 = 200;
/// Gain at [`CYCLE_COUNT`], counts per µT
pub const COUNTS_PER_UT: f64 = 75.0;
/// Reads in a row without a result (or failing) that fail the part
pub const MAX_ERRORS: u32 = 5;

/// Registers
pub mod reg {
    /// POLL: start a single measurement
    pub const POLL: u8 = 0x00;
    /// CMM: continuous mode (kept off)
    pub const CMM: u8 = 0x01;
    /// CCX (MSB), the first cycle-count register
    pub const CCX: u8 = 0x04;
    /// MX (MSB), the first result register
    pub const MX: u8 = 0x24;
    /// STATUS (bit 7: data ready)
    pub const STATUS: u8 = 0x34;
    /// REVID
    pub const REVID: u8 = 0x36;
}

/// The driver's state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rm3100 {
    /// Configured and answering
    pub ready: bool,
    /// A measurement was started and not read
    pub pending: bool,
    /// Failed reads in a row
    pub errors: u32,
    /// Failed: the manager power-cycles the sensor rail
    pub failed: bool,
}

impl Rm3100 {
    /// Not configured
    pub const fn new() -> Rm3100 {
        Rm3100 { ready: false, pending: false, errors: 0, failed: false }
    }

    /// Probe and configure (after the rail is up): `REVID`, continuous mode
    /// off, the cycle counts written and read back
    pub fn init(&mut self, bus: &mut impl I2cBus) -> bool {
        *self = Rm3100::new();
        let mut id = [0u8];
        let [h, l] = CYCLE_COUNT.to_be_bytes();
        let counts = [h, l, h, l, h, l];
        let mut back = [0u8; 6];
        let ok = bus.write_read(ADDRESS, &[reg::REVID], &mut id).is_ok()
            && id[0] == REVID
            && bus.write(ADDRESS, &[reg::CMM, 0]).is_ok()
            && bus.write(ADDRESS, &[reg::CCX, h, l, h, l, h, l]).is_ok()
            && bus.write_read(ADDRESS, &[reg::CCX], &mut back).is_ok()
            && back == counts;
        self.ready = ok;
        self.failed = !ok;
        ok
    }

    /// Start a single measurement of the three axes
    pub fn trigger(&mut self, bus: &mut impl I2cBus) -> Result<(), BusError> {
        if !self.ready {
            return Err(BusError);
        }
        let r = bus.write(ADDRESS, &[reg::POLL, 0x70]);
        self.pending = r.is_ok();
        if r.is_err() {
            self.count_error();
        }
        r
    }

    fn count_error(&mut self) {
        self.errors = self.errors.saturating_add(1);
        if self.errors >= MAX_ERRORS {
            self.failed = true;
            self.ready = false;
        }
    }

    /// 24-bit two's complement, MSB first
    pub fn counts(b: [u8; 3]) -> i32 {
        i32::from_be_bytes([b[0], b[1], b[2], 0]) >> 8
    }

    /// The field (sensor axes), µT, of the pending measurement if it is
    /// done (`drdy`: the DRDY line); `None` if none is pending, it is not
    /// done, or the read fails
    pub fn read_field(&mut self, drdy: bool, bus: &mut impl I2cBus) -> Option<[f64; 3]> {
        if !self.ready || !self.pending {
            return None;
        }
        let mut status = [0u8];
        let done = drdy || (bus.write_read(ADDRESS, &[reg::STATUS], &mut status).is_ok() && status[0] & 0x80 != 0);
        let mut b = [0u8; 9];
        if !done || bus.write_read(ADDRESS, &[reg::MX], &mut b).is_err() {
            self.pending = false;
            self.count_error();
            return None;
        }
        self.pending = false;
        self.errors = 0;
        let [x0, x1, x2, y0, y1, y2, z0, z1, z2] = b;
        let axis = |c: [u8; 3]| f64::from(Rm3100::counts(c)) / COUNTS_PER_UT;
        Some([axis([x0, x1, x2]), axis([y0, y1, y2]), axis([z0, z1, z2])])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::mock::I2cRegs;

    fn part() -> I2cRegs {
        let mut m = I2cRegs::new(ADDRESS);
        m.regs[usize::from(reg::REVID)] = REVID;
        // X +30 µT (2250), Y -15 µT (-1125), Z 0
        m.regs[0x24..0x2D].copy_from_slice(&[0x00, 0x08, 0xCA, 0xFF, 0xFB, 0x9B, 0, 0, 0]);
        m
    }

    #[test]
    fn configures_triggers_and_reads() {
        let mut bus = part();
        let mut mag = Rm3100::new();
        assert_eq!(mag.read_field(true, &mut bus), None, "not configured");
        assert!(mag.init(&mut bus));
        assert_eq!(&bus.regs[4..10], &[0, 200, 0, 200, 0, 200]);
        assert_eq!(mag.read_field(true, &mut bus), None, "nothing pending");
        mag.trigger(&mut bus).unwrap();
        assert_eq!(bus.regs[0], 0x70);
        assert_eq!(mag.read_field(false, &mut bus), None, "not done (STATUS clear): counted");
        assert_eq!(mag.errors, 1);
        mag.trigger(&mut bus).unwrap();
        bus.regs[usize::from(reg::STATUS)] = 0x80;
        let f = mag.read_field(false, &mut bus).unwrap();
        assert_eq!(f, [30.0, -15.0, 0.0]);
        assert_eq!(mag.errors, 0);
        assert_eq!(Rm3100::counts([0x80, 0, 0]), -8_388_608);
    }

    #[test]
    fn wrong_revision_and_repeated_errors_fail_it() {
        let mut bus = part();
        bus.regs[usize::from(reg::REVID)] = 0x21;
        let mut mag = Rm3100::new();
        assert!(!mag.init(&mut bus));
        assert!(mag.failed);
        assert_eq!(mag.trigger(&mut bus), Err(BusError));

        let mut bus = part();
        assert!(mag.init(&mut bus));
        bus.fail = true;
        for _ in 0..MAX_ERRORS {
            let _ = mag.trigger(&mut bus);
        }
        assert!(mag.failed && !mag.ready);
    }
}
