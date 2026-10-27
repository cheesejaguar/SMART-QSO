//! TI INA228-Q1 power monitors on the EPS board, read by the OBC directly
//! over I2C2 (`hal_i2c` bus 0), independently of the EPS controller
//! (SQSO-EPS-BRD-001 B §11; SMART-QSO-ICD-001 §2.3): the pack gauge U28 at
//! 0x44 on the 10 mΩ high-side shunt RS6, and the array-output monitor U29
//! at 0x40 on the 20 mΩ shunt RS5.
//!
//! From the INA228 data sheet (SBOSA20): 16-bit `CONFIG` (0x00),
//! `ADC_CONFIG` (0x01), `SHUNT_CAL` (0x02); 24-bit `VBUS` (0x05, 195.3125 µV
//! per LSB in bits 23–4) and `CURRENT` (0x07, `CURRENT_LSB` per LSB in bits
//! 23–4, two's complement); 16-bit `DIETEMP` (0x06, 7.8125 m°C per LSB); 40-bit
//! `ENERGY` (0x09, 3.2 × 16 × `CURRENT_LSB` J per LSB) and `CHARGE` (0x0A,
//! `CURRENT_LSB` C per LSB, two's complement); `MANUFACTURER_ID` (0x3E) 0x5449
//! and `DEVICE_ID` (0x3F) 0x228x. Registers are MSB first.
//! `SHUNT_CAL` = 13107.2 × 10⁶ × `CURRENT_LSB` × R_shunt (`ADCRANGE` 0,
//! ±163.84 mV).
//!
//! Current sign: positive charging the pack (the convention of IBATT,
//! SMART-QSO-ICD-001 §5.2), and positive out of the array.
//!
//! @requirement SRS-F147 Pack and array power monitors read directly by the OBC

use crate::bus::{BusError, I2cBus};

/// The pack gauge U28
pub const ADDRESS_PACK: u8 = 0x44;
/// The array-output monitor U29
pub const ADDRESS_ARRAY: u8 = 0x40;
/// `MANUFACTURER_ID`
pub const MANUFACTURER_ID: u16 = 0x5449;
/// `DEVICE_ID` without its revision nibble
pub const DEVICE_ID: u16 = 0x228;
/// `ADC_CONFIG`: continuous bus, shunt and temperature, 1052 µs each,
/// 64 averages (a result every 202 ms)
pub const ADC_CONFIG: u16 = 0xFB6B;
/// VBUS scale, V per LSB
pub const VBUS_V_PER_LSB: f64 = 195.3125e-6;
/// Die temperature scale, °C per LSB
pub const TEMP_C_PER_LSB: f64 = 7.8125e-3;

/// Registers
pub mod reg {
    /// CONFIG
    pub const CONFIG: u8 = 0x00;
    /// ADC_CONFIG
    pub const ADC_CONFIG: u8 = 0x01;
    /// SHUNT_CAL
    pub const SHUNT_CAL: u8 = 0x02;
    /// VBUS (24 bits)
    pub const VBUS: u8 = 0x05;
    /// DIETEMP (16 bits)
    pub const DIETEMP: u8 = 0x06;
    /// CURRENT (24 bits)
    pub const CURRENT: u8 = 0x07;
    /// ENERGY (40 bits)
    pub const ENERGY: u8 = 0x09;
    /// CHARGE (40 bits)
    pub const CHARGE: u8 = 0x0A;
    /// MANUFACTURER_ID
    pub const MANUFACTURER_ID: u8 = 0x3E;
    /// DEVICE_ID
    pub const DEVICE_ID: u8 = 0x3F;
}

/// One monitor's shunt and scale
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Monitor {
    /// I2C address
    pub address: u8,
    /// Shunt, Ω
    pub shunt_ohms: f64,
    /// `CURRENT_LSB`, A
    pub current_lsb_a: f64,
}

/// The pack gauge: 10 µA LSB (±5.2 A full scale)
pub const PACK: Monitor = Monitor { address: ADDRESS_PACK, shunt_ohms: 0.010, current_lsb_a: 10e-6 };
/// The array monitor: 5 µA LSB (±2.6 A full scale)
pub const ARRAY: Monitor = Monitor { address: ADDRESS_ARRAY, shunt_ohms: 0.020, current_lsb_a: 5e-6 };

/// One reading
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Reading {
    /// Bus voltage, V
    pub vbus_v: f64,
    /// Current, A
    pub current_a: f64,
    /// Die temperature, °C
    pub die_c: f64,
    /// Charge counted since configuration, C
    pub charge_c: f64,
    /// Energy counted since configuration, J
    pub energy_j: f64,
}

impl Monitor {
    /// `SHUNT_CAL` for this shunt and LSB
    pub fn shunt_cal(&self) -> u16 {
        let cal = crate::fmath::round(13_107.2e6 * self.current_lsb_a * self.shunt_ohms);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 15 bits
        let v = cal.clamp(0.0, 32_767.0) as u16;
        v
    }

    fn read(&self, bus: &mut impl I2cBus, reg: u8, out: &mut [u8]) -> Result<(), BusError> {
        bus.write_read(self.address, &[reg], out)
    }

    fn read16(&self, bus: &mut impl I2cBus, reg: u8) -> Result<u16, BusError> {
        let mut b = [0u8; 2];
        self.read(bus, reg, &mut b)?;
        Ok(u16::from_be_bytes(b))
    }

    fn write16(&self, bus: &mut impl I2cBus, reg: u8, v: u16) -> Result<(), BusError> {
        let [h, l] = v.to_be_bytes();
        bus.write(self.address, &[reg, h, l])
    }

    /// Probe the IDs, then configure: `ADCRANGE` 0, [`ADC_CONFIG`],
    /// `SHUNT_CAL`, read back
    pub fn init(&self, bus: &mut impl I2cBus) -> bool {
        self.read16(bus, reg::MANUFACTURER_ID) == Ok(MANUFACTURER_ID)
            && self.read16(bus, reg::DEVICE_ID).map(|d| d >> 4) == Ok(DEVICE_ID)
            && self.write16(bus, reg::CONFIG, 0).is_ok()
            && self.write16(bus, reg::ADC_CONFIG, ADC_CONFIG).is_ok()
            && self.write16(bus, reg::SHUNT_CAL, self.shunt_cal()).is_ok()
            && self.read16(bus, reg::SHUNT_CAL) == Ok(self.shunt_cal())
    }

    /// The die temperature alone, °C, without configuring the part (the
    /// temperature converts in the power-up mode too); `None` if the part
    /// does not identify itself or does not answer
    pub fn die_c(&self, bus: &mut impl I2cBus) -> Option<f64> {
        if self.read16(bus, reg::MANUFACTURER_ID) != Ok(MANUFACTURER_ID) {
            return None;
        }
        #[allow(clippy::cast_possible_wrap)] // the register is two's complement
        let raw = self.read16(bus, reg::DIETEMP).ok()? as i16;
        Some(f64::from(raw) * 7.8125e-3)
    }

    /// Read the measurements
    pub fn reading(&self, bus: &mut impl I2cBus) -> Result<Reading, BusError> {
        let (mut v, mut i, mut e, mut q) = ([0u8; 3], [0u8; 3], [0u8; 5], [0u8; 5]);
        self.read(bus, reg::VBUS, &mut v)?;
        self.read(bus, reg::CURRENT, &mut i)?;
        let t = self.read16(bus, reg::DIETEMP)?;
        self.read(bus, reg::ENERGY, &mut e)?;
        self.read(bus, reg::CHARGE, &mut q)?;
        Ok(self.decode(v, i, t, e, q))
    }

    /// The registers' values in units
    pub fn decode(&self, vbus: [u8; 3], current: [u8; 3], dietemp: u16, energy: [u8; 5], charge: [u8; 5]) -> Reading {
        let v20 = |b: [u8; 3]| i32::from_be_bytes([b[0], b[1], b[2], 0]) >> 12;
        let u40 = |b: [u8; 5]| u64::from_be_bytes([0, 0, 0, b[0], b[1], b[2], b[3], b[4]]);
        let i40 = |b: [u8; 5]| i64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], 0, 0, 0]) >> 24;
        #[allow(clippy::cast_possible_wrap)] // the register is two's complement
        let die = f64::from(dietemp as i16);
        #[allow(clippy::cast_precision_loss)] // 40-bit counters, well inside 2^53
        Reading {
            vbus_v: f64::from(v20(vbus)) * VBUS_V_PER_LSB,
            current_a: f64::from(v20(current)) * self.current_lsb_a,
            die_c: die * TEMP_C_PER_LSB,
            energy_j: u40(energy) as f64 * 3.2 * 16.0 * self.current_lsb_a,
            charge_c: i40(charge) as f64 * self.current_lsb_a,
        }
    }

    /// The register bytes of a reading (the simulated monitor's side)
    pub fn encode(&self, r: &Reading) -> ([u8; 3], [u8; 3], u16) {
        use crate::fmath::round;
        #[allow(clippy::cast_possible_truncation)] // clamped to 20 bits
        let v20 = |x: f64| ((round(x).clamp(-524_288.0, 524_287.0) as i32) << 12).to_be_bytes();
        let [a, b, c, _] = v20(r.vbus_v / VBUS_V_PER_LSB);
        let [d, e, f, _] = v20(r.current_a / self.current_lsb_a);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 16 bits
        let t = (round(r.die_c / TEMP_C_PER_LSB).clamp(-32_768.0, 32_767.0) as i16) as u16;
        ([a, b, c], [d, e, f], t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::mock::I2cRegs;

    /// The register model with the two ID registers apart (each register
    /// of the INA228 is wider than a byte; the byte model overlaps them)
    struct Part {
        regs: I2cRegs,
        device_id: u16,
    }

    impl I2cBus for Part {
        fn write(&mut self, a: u8, d: &[u8]) -> Result<(), BusError> {
            self.regs.write(a, d)
        }
        fn write_read(&mut self, a: u8, w: &[u8], r: &mut [u8]) -> Result<(), BusError> {
            match w {
                [reg::MANUFACTURER_ID] if a == self.regs.addr => r.copy_from_slice(&MANUFACTURER_ID.to_be_bytes()),
                [reg::DEVICE_ID] if a == self.regs.addr => r.copy_from_slice(&self.device_id.to_be_bytes()),
                _ => return self.regs.write_read(a, w, r),
            }
            Ok(())
        }
        fn read(&mut self, a: u8, r: &mut [u8]) -> Result<(), BusError> {
            self.regs.read(a, r)
        }
    }

    #[test]
    fn calibration_values() {
        assert_eq!(PACK.shunt_cal(), 1311);
        assert_eq!(ARRAY.shunt_cal(), 1311);
    }

    #[test]
    fn decode_encode_round_trip() {
        let r = Reading { vbus_v: 14.4, current_a: -0.735, die_c: 21.5, ..Reading::default() };
        let (v, i, t) = PACK.encode(&r);
        let d = PACK.decode(v, i, t, [0, 0, 0, 0x01, 0x00], [0xFF, 0xFF, 0xFF, 0xFF, 0x9C]);
        assert!((d.vbus_v - 14.4).abs() < 2e-4);
        assert!((d.current_a + 0.735).abs() < 1e-5);
        assert!((d.die_c - 21.5).abs() < 1e-2);
        assert!((d.energy_j - 256.0 * 3.2 * 16.0 * 10e-6).abs() < 1e-9);
        assert!((d.charge_c + 100.0 * 10e-6).abs() < 1e-12);
    }

    #[test]
    fn init_checks_the_ids_and_the_calibration() {
        let mut m = Part { regs: I2cRegs::new(ADDRESS_PACK), device_id: 0x1111 };
        assert!(!PACK.init(&mut m), "wrong device ID");
        m.device_id = 0x2281;
        assert!(PACK.init(&mut m));
        assert_eq!(&m.regs.regs[0x02..0x04], &1311u16.to_be_bytes());
        assert!(!ARRAY.init(&mut m), "no part at 0x40");
        let (_, i, _) = PACK.encode(&Reading { current_a: 0.5, ..Reading::default() });
        m.regs.regs[0x07..0x0A].copy_from_slice(&i);
        let r = PACK.reading(&mut m).unwrap();
        assert!((r.current_a - 0.5).abs() < 1e-5);
        m.regs.fail = true;
        assert_eq!(PACK.reading(&mut m), Err(BusError));
    }
}
