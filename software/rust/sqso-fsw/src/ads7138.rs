//! TI ADS7138-Q1 sun-sensor ADC (I2C4, `hal_i2c` bus 1, address 0x10;
//! SQSO-OBC-BRD-001 B §9.5; ICD-004 §7): AIN0–AIN5 the six panel
//! photodiodes (+X, −X, +Y, −Y, +Z, −Z) through transimpedance amplifiers of
//! 619 Ω, AIN6 the TMP235 beside the gyros, AIN7 VBAT_MTQ / 6; reference
//! AVDD = 3V3_SNS.
//!
//! From the ADS7138 data sheet (SBAS868): opcodes single-register read
//! 0x10 and write 0x08; manual mode (`SEQUENCE_CFG` 0), the channel in
//! `CHANNEL_SEL` (0x11), and each 2-byte read frame returns a 12-bit
//! conversion, left-aligned. At power-up `SYSTEM_STATUS` (0x00) reads with
//! bit 7 set and `BOR` (bit 0) set; the driver clears `BOR` (write 1) and
//! starts the offset calibration (`GENERAL_CFG.CAL`, bit 1).
//!
//! @requirement SRS-F140 ADCS sensor drivers on the OBC buses

use crate::bus::{BusError, I2cBus};

/// I2C address
pub const ADDRESS: u8 = 0x10;
/// Channels converted
pub const CHANNELS: usize = 8;
/// Reference (AVDD = 3V3_SNS), V
pub const VREF: f64 = 3.3;
/// Transimpedance, Ω (photocurrent = V / 619 Ω)
pub const TIA_OHMS: f64 = 619.0;
/// Photocurrent at AM0, normal incidence (the panel vendor's limit), A:
/// intensity 1
pub const FULL_SUN_A: f64 = 0.004;

const OP_READ: u8 = 0x10;
const OP_WRITE: u8 = 0x08;
/// Registers
pub mod reg {
    /// SYSTEM_STATUS
    pub const SYSTEM_STATUS: u8 = 0x00;
    /// GENERAL_CFG
    pub const GENERAL_CFG: u8 = 0x01;
    /// PIN_CFG (0: every pin analog)
    pub const PIN_CFG: u8 = 0x05;
    /// SEQUENCE_CFG (0: manual)
    pub const SEQUENCE_CFG: u8 = 0x10;
    /// CHANNEL_SEL
    pub const CHANNEL_SEL: u8 = 0x11;
}

/// One reading of the eight inputs
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    /// Raw 12-bit codes, AIN0–AIN7
    pub raw: [u16; CHANNELS],
}

impl Reading {
    /// Volts at an input
    pub fn volts(&self, ch: usize) -> f64 {
        f64::from(self.raw.get(ch).copied().unwrap_or(0)) / 4095.0 * VREF
    }

    /// The sun vector in body axes, its length the intensity (1 for full
    /// sun at normal incidence): opposite faces subtracted
    pub fn sun(&self) -> [f64; 3] {
        let i = |ch: usize| self.volts(ch) / TIA_OHMS / FULL_SUN_A;
        [i(0) - i(1), i(2) - i(3), i(4) - i(5)]
    }

    /// The TMP235 beside the gyros, °C (10 mV/°C, 500 mV at 0 °C)
    pub fn sensor_temp_c(&self) -> f64 {
        (self.volts(6) - 0.5) * 100.0
    }

    /// The coil supply VBAT_MTQ, V (1 MΩ / 200 kΩ divider)
    pub fn vbat_mtq(&self) -> f64 {
        self.volts(7) * 6.0
    }
}

fn write(bus: &mut impl I2cBus, reg: u8, value: u8) -> Result<(), BusError> {
    bus.write(ADDRESS, &[OP_WRITE, reg, value])
}

fn read_reg(bus: &mut impl I2cBus, reg: u8) -> Result<u8, BusError> {
    let mut v = [0u8];
    bus.write_read(ADDRESS, &[OP_READ, reg], &mut v)?;
    Ok(v[0])
}

/// Probe and configure (after the rail is up): `SYSTEM_STATUS` bit 7,
/// `BOR` cleared, every pin analog, manual mode, offset calibration
pub fn init(bus: &mut impl I2cBus) -> bool {
    let status = read_reg(bus, reg::SYSTEM_STATUS);
    matches!(status, Ok(s) if s & 0x80 != 0)
        && write(bus, reg::SYSTEM_STATUS, 0x01).is_ok()
        && write(bus, reg::PIN_CFG, 0x00).is_ok()
        && write(bus, reg::SEQUENCE_CFG, 0x00).is_ok()
        && write(bus, reg::GENERAL_CFG, 0x02).is_ok()
}

/// Convert the eight inputs, one manual conversion each
pub fn read(bus: &mut impl I2cBus) -> Result<Reading, BusError> {
    let mut r = Reading::default();
    for (ch, raw) in (0u8..).zip(r.raw.iter_mut()) {
        write(bus, reg::CHANNEL_SEL, ch)?;
        let mut b = [0u8; 2];
        bus.read(ADDRESS, &mut b)?;
        *raw = u16::from_be_bytes(b) >> 4;
    }
    Ok(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    /// A model of the opcode protocol: `codes` per channel
    struct Model {
        regs: [u8; 32],
        codes: [u16; CHANNELS],
        fail: bool,
    }

    impl I2cBus for Model {
        fn write(&mut self, addr: u8, d: &[u8]) -> Result<(), BusError> {
            if self.fail || addr != ADDRESS {
                return Err(BusError);
            }
            if let [OP_WRITE, r, v] = d {
                self.regs[usize::from(*r)] = if *r == reg::SYSTEM_STATUS { self.regs[0] & !*v } else { *v };
            }
            Ok(())
        }
        fn write_read(&mut self, addr: u8, w: &[u8], r: &mut [u8]) -> Result<(), BusError> {
            self.write(addr, &[])?;
            if let [OP_READ, reg] = w {
                r[0] = self.regs[usize::from(*reg)];
            }
            Ok(())
        }
        fn read(&mut self, addr: u8, r: &mut [u8]) -> Result<(), BusError> {
            self.write(addr, &[])?;
            let code = self.codes[usize::from(self.regs[usize::from(reg::CHANNEL_SEL)])] << 4;
            r.copy_from_slice(&code.to_be_bytes());
            Ok(())
        }
    }

    #[test]
    fn configures_and_converts() {
        let mut m = Model { regs: [0; 32], codes: [3084, 0, 0, 1542, 0, 0, 1117, 1489], fail: false };
        m.regs[0] = 0x81;
        assert!(init(&mut m));
        assert_eq!(m.regs[0], 0x80, "BOR cleared");
        assert_eq!(m.regs[usize::from(reg::GENERAL_CFG)], 0x02);
        let r = read(&mut m).unwrap();
        let s = r.sun();
        // 3084 counts = 2.485 V = 4.015 mA: about full sun on +X; half on -Y
        assert!((s[0] - 1.0).abs() < 0.01 && (s[1] + 0.5).abs() < 0.01 && s[2] == 0.0, "{s:?}");
        assert!((r.sensor_temp_c() - 40.0).abs() < 0.1);
        assert!((r.vbat_mtq() - 7.2).abs() < 0.01);
        assert_eq!(r.volts(99), 0.0);
        m.regs[0] = 0;
        assert!(!init(&mut m), "no power-up status: not the part");
        m.fail = true;
        assert!(!init(&mut m));
        assert_eq!(read(&mut m), Err(BusError));
    }
}
