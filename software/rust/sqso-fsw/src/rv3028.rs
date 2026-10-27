//! Micro Crystal RV-3028-C7 mission clock (I2C1, `hal_i2c` bus 2, address
//! 0x52; on the always-on rail; SQSO-OBC-BRD-001 B §8, RID AV-10).
//!
//! From the RV-3028-C7 application manual: the 32-bit UNIX time counter at
//! 0x1B–0x1E (LSB first), the calendar at 0x00–0x06 (BCD), `STATUS` 0x0E
//! (bit 0 `PORF`: the clock lost power; bit 1 `EVF`: an event on EVI),
//! `CONTROL2` 0x10 (bit 7 `TSE`: time stamp enable), `EVENT_CONTROL` 0x13
//! (bit 1 `TSOW`: the stamp keeps the latest event; `EHL` 0: the falling
//! edge), the time stamp at 0x14 (`COUNT_TS`) to 0x1A (calendar, BCD),
//! `ID` 0x28 (hardware ID in the high nibble, 3), the EEPROM's `CLKOUT`
//! mirror 0x35 (bit 7 `CLKOE`, bits 2–0 `FD`; 1 Hz is 0b101).
//!
//! EVI is wired to NRST, so every reset of the OBC (and its power loss) is
//! stamped by the clock, which keeps running. At boot the driver reads the
//! counter, the stamp of the reset just past and the event count, converts
//! the stamp to UNIX time through the calendar (the counter and the
//! calendar count the same seconds), clears `EVF` and arms the stamp for
//! the next reset. The 1 Hz `CLKOUT` is set in the EEPROM at integration;
//! the driver only checks it.
//!
//! @requirement SRS-F145 Mission clock: reset time stamps and the clock check

use crate::bus::{BusError, I2cBus};

/// I2C address
pub const ADDRESS: u8 = 0x52;
/// Hardware ID (the `ID` register's high nibble)
pub const HID: u8 = 0x3;

/// Registers
pub mod reg {
    /// Seconds (the calendar starts here, seven registers)
    pub const SECONDS: u8 = 0x00;
    /// STATUS
    pub const STATUS: u8 = 0x0E;
    /// CONTROL2
    pub const CONTROL2: u8 = 0x10;
    /// EVENT_CONTROL
    pub const EVENT_CONTROL: u8 = 0x13;
    /// COUNT_TS (then the stamp's calendar, six registers)
    pub const COUNT_TS: u8 = 0x14;
    /// UNIX time, byte 0
    pub const UNIX: u8 = 0x1B;
    /// ID
    pub const ID: u8 = 0x28;
    /// EEPROM CLKOUT mirror
    pub const CLKOUT: u8 = 0x35;
}

const PORF: u8 = 0x01;
const EVF: u8 = 0x02;
const TSE: u8 = 0x80;
const TSOW: u8 = 0x02;
/// `CLKOUT` for 1 Hz with the output enabled
pub const CLKOUT_1HZ: u8 = 0x85;

/// What the clock reported at boot
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boot {
    /// UNIX time now, s
    pub unix_s: u32,
    /// UNIX time of the reset just past (the EVI stamp), if one was stamped
    pub reset_unix_s: Option<u32>,
    /// Resets counted by the stamp since it was last cleared
    pub reset_count: u8,
    /// The clock lost power since it was set (its time is not valid)
    pub lost_power: bool,
    /// The 1 Hz `CLKOUT` is configured
    pub clkout_1hz: bool,
}

fn bcd(b: u8) -> u32 {
    u32::from(b >> 4) * 10 + u32::from(b & 0x0F)
}

/// Seconds since 2000-01-01 of a calendar time (BCD seconds, minutes,
/// hours (24 h), date, month, year 00–99): `None` if out of range
pub fn calendar_s(sec: u8, min: u8, hour: u8, date: u8, month: u8, year: u8) -> Option<u32> {
    let (s, mi, h, d, mo, y) = (bcd(sec), bcd(min), bcd(hour), bcd(date), bcd(month), bcd(year));
    if s > 59 || mi > 59 || h > 23 || d == 0 || d > 31 || mo == 0 || mo > 12 || y > 99 {
        return None;
    }
    const DAYS: [u32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap_before = y.div_ceil(4); // leap years 2000..2000+y-1
    let leap_now = u32::from(y % 4 == 0 && mo > 2);
    let days = y * 365 + leap_before + DAYS.get(usize::try_from(mo - 1).ok()?)? + leap_now + d - 1;
    Some(((days * 24 + h) * 60 + mi) * 60 + s)
}

fn read(bus: &mut impl I2cBus, reg: u8, out: &mut [u8]) -> Result<(), BusError> {
    bus.write_read(ADDRESS, &[reg], out)
}

/// Whether the part answers with its ID
pub fn present(bus: &mut impl I2cBus) -> bool {
    let mut id = [0u8];
    read(bus, reg::ID, &mut id).is_ok() && id[0] >> 4 == HID
}

/// At boot: read the counter, the reset's stamp and the status, clear the
/// event flag and arm the stamp for the next reset
pub fn boot(bus: &mut impl I2cBus) -> Result<Boot, BusError> {
    if !present(bus) {
        return Err(BusError);
    }
    let mut unix = [0u8; 4];
    let mut cal = [0u8; 7];
    let mut ts = [0u8; 7];
    let mut status = [0u8];
    let mut clk = [0u8];
    read(bus, reg::UNIX, &mut unix)?;
    read(bus, reg::SECONDS, &mut cal)?;
    read(bus, reg::COUNT_TS, &mut ts)?;
    read(bus, reg::STATUS, &mut status)?;
    read(bus, reg::CLKOUT, &mut clk)?;
    let unix_s = u32::from_le_bytes(unix);
    let [s, mi, h, _wd, d, mo, y] = cal;
    let [count, ts_s, ts_mi, ts_h, ts_d, ts_mo, ts_y] = ts;
    let stamped = status[0] & EVF != 0;
    let reset_unix_s = match (stamped, calendar_s(s, mi, h, d, mo, y), calendar_s(ts_s, ts_mi, ts_h, ts_d, ts_mo, ts_y)) {
        (true, Some(now), Some(then)) if then <= now => unix_s.checked_sub(now - then),
        _ => None,
    };
    bus.write(ADDRESS, &[reg::STATUS, status[0] & !(EVF | PORF)])?;
    bus.write(ADDRESS, &[reg::EVENT_CONTROL, TSOW])?;
    let mut c2 = [0u8];
    read(bus, reg::CONTROL2, &mut c2)?;
    bus.write(ADDRESS, &[reg::CONTROL2, c2[0] | TSE])?;
    Ok(Boot {
        unix_s,
        reset_unix_s,
        reset_count: count,
        lost_power: status[0] & PORF != 0,
        clkout_1hz: clk[0] & 0x87 == CLKOUT_1HZ,
    })
}

/// UNIX time now, s
pub fn now(bus: &mut impl I2cBus) -> Result<u32, BusError> {
    let mut unix = [0u8; 4];
    read(bus, reg::UNIX, &mut unix)?;
    Ok(u32::from_le_bytes(unix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::mock::I2cRegs;

    #[test]
    fn calendar_seconds() {
        assert_eq!(calendar_s(0, 0, 0, 1, 1, 0), Some(0));
        // 2000-03-01 00:00:00: 31 + 29 days
        assert_eq!(calendar_s(0, 0, 0, 1, 3, 0), Some(60 * 86_400));
        // 2028-11-27 12:34:56 (BCD); 10,558 days
        assert_eq!(calendar_s(0x56, 0x34, 0x12, 0x27, 0x11, 0x28), Some(10_558 * 86_400 + 45_296));
        assert_eq!(calendar_s(0x60, 0, 0, 1, 1, 0), None);
        assert_eq!(calendar_s(0, 0, 0, 0, 1, 0), None);
        assert_eq!(calendar_s(0, 0, 0, 1, 0x13, 0), None);
    }

    #[test]
    fn boot_reads_the_reset_stamp_and_rearms_it() {
        let mut m = I2cRegs::new(ADDRESS);
        m.regs[usize::from(reg::ID)] = 0x31;
        m.regs[0x1B..0x1F].copy_from_slice(&1_900_000_000u32.to_le_bytes());
        // now 12:00:30, stamp 12:00:00, 2028-11-27
        m.regs[0..7].copy_from_slice(&[0x30, 0x00, 0x12, 1, 0x27, 0x11, 0x28]);
        m.regs[0x14..0x1B].copy_from_slice(&[3, 0x00, 0x00, 0x12, 0x27, 0x11, 0x28]);
        m.regs[usize::from(reg::STATUS)] = EVF;
        m.regs[usize::from(reg::CLKOUT)] = CLKOUT_1HZ;
        let b = boot(&mut m).unwrap();
        assert_eq!(b.unix_s, 1_900_000_000);
        assert_eq!(b.reset_unix_s, Some(1_900_000_000 - 30));
        assert_eq!(b.reset_count, 3);
        assert!(!b.lost_power);
        assert!(b.clkout_1hz);
        assert_eq!(m.regs[usize::from(reg::STATUS)], 0, "EVF cleared");
        assert_eq!(m.regs[usize::from(reg::CONTROL2)], TSE);
        assert_eq!(m.regs[usize::from(reg::EVENT_CONTROL)], TSOW);
        assert_eq!(now(&mut m), Ok(1_900_000_000));

        // No event, the clock lost power, CLKOUT not set
        m.regs[usize::from(reg::STATUS)] = PORF;
        m.regs[usize::from(reg::CLKOUT)] = 0;
        let b = boot(&mut m).unwrap();
        assert_eq!(b.reset_unix_s, None);
        assert!(b.lost_power);
        assert!(!b.clkout_1hz);

        m.regs[usize::from(reg::ID)] = 0x10;
        assert_eq!(boot(&mut m), Err(BusError));
    }
}
