//! EPS controller telemetry (SMART-QSO-ICD-001 §4.2–§5): the register
//! block the OBC reads from the controller at 0x48 on I2C bus 0, decoded
//! and checked for plausibility.
//!
//! The executive takes the state of charge and sunlight from here, and the
//! EPS sensors their readings, when the controller answers with a plausible
//! 4S pack voltage; otherwise they fall back to the stand-ins (SW-01).
//!
//! @requirement SRS-F010 Monitor battery state of charge

/// EPS controller I2C address (SMART-QSO-ICD-001 §2.3)
pub const ADDRESS: u8 = 0x48;
/// Bytes read from register 0x00: STATUS through IBUS (0x00–0x33)
pub const BLOCK_LEN: usize = 0x34;

/// STATUS bits (§4.3)
pub mod status {
    /// Battery charging active
    pub const CHARGING: u8 = 0x80;
    /// Solar power available
    pub const SUNLIT: u8 = 0x40;
    /// Low battery warning
    pub const LOW_BATT: u8 = 0x20;
    /// Over temperature
    pub const OVER_TEMP: u8 = 0x10;
    /// Under temperature
    pub const UNDER_TEMP: u8 = 0x08;
    /// Fault condition active
    pub const FAULT: u8 = 0x04;
}

/// Register offsets (§4.2)
mod reg {
    pub const STATUS: usize = 0x00;
    pub const VBATT: usize = 0x10;
    pub const IBATT: usize = 0x12;
    pub const TBATT: usize = 0x14;
    pub const SOC: usize = 0x16;
    pub const VSOLAR: usize = 0x20;
    pub const ISOLAR: usize = 0x22;
    pub const VBUS: usize = 0x30;
    pub const IBUS: usize = 0x32;
}

/// The recharge lockout (RID EPS-12; `docs/hardware/EPS_DESIGN.md` §3.3):
/// after any cell has been below 2.0 V the EPS latches its charge path off
/// until the ground clears it, which takes both an unlock write over I2C
/// and a pulse on the separate `LOCK_CLR` line (`EPS_BOARD_DESIGN.md`)
pub mod lockout {
    /// The unlock register (an addition to SMART-QSO-ICD-001 §4.6, for the
    /// EPS controller's firmware)
    pub const REG_UNLOCK: u8 = 0x7D;
    /// The unlock value
    pub const UNLOCK_KEY: u8 = 0xC5;
    /// `CHG_INHIBIT` (0x7A) bit of the latched lockout (also an addition)
    pub const CHG_INHIBIT_LOCKOUT: u8 = 0x20;
    /// `LOCK_CLR` pulse, ms (the EPS needs at least 100 ms)
    pub const PULSE_MS: u64 = 150;
    /// EPS_CLEAR_LOCKOUT's argument
    pub const CODE: [u8; 4] = *b"CHG!";
}

/// A plausible 4S pack voltage, V: below it the block is not a live
/// controller (a blank or absent device reads 0)
pub const VBATT_PLAUSIBLE_V: (f64, f64) = (9.0, 17.6);

/// Decoded EPS telemetry
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Telemetry {
    /// STATUS register
    pub status: u8,
    /// State of charge, 0–1
    pub soc: f64,
    /// Battery voltage, V
    pub vbatt_v: f64,
    /// Battery current, A (positive: charging)
    pub ibatt_a: f64,
    /// Battery temperature, °C
    pub tbatt_c: f64,
    /// Solar array voltage, V
    pub vsolar_v: f64,
    /// Solar array current, A
    pub isolar_a: f64,
    /// 5 V payload bus voltage, V
    pub vbus_v: f64,
    /// 5 V payload bus current, A
    pub ibus_a: f64,
    /// The Jetson rail's current at VBAT, A: the eFuse U19's IMON on the
    /// controller's ADC (`IMON_JET`, [`IMON_JET`]; 0 when not read)
    pub ijet_a: f64,
}

/// The controller's `IMON_JET` register (an addition to SMART-QSO-ICD-001
/// §4.6): the Jetson eFuse current, mA, 16-bit unsigned, LSB first
pub const IMON_JET: u8 = 0x8A;
/// The `EOL_STATE` register: bit 0 armed, 1 fired (charge path open), 2 the
/// fuse F1 intact (SMART-QSO-ICD-001 §4.6)
pub const EOL_STATE: u8 = 0x7C;

/// `IMON_JET`'s two bytes, A
pub fn imon_jet_a(b: [u8; 2]) -> f64 {
    f64::from(u16::from_le_bytes(b)) / 1000.0
}

impl Telemetry {
    /// In sunlight (STATUS.SUNLIT)
    pub const fn sunlit(&self) -> bool {
        self.status & status::SUNLIT != 0
    }

    /// Battery discharge current, A (0 while charging)
    pub fn discharge_a(&self) -> f64 {
        if self.ibatt_a < 0.0 { -self.ibatt_a } else { 0.0 }
    }
}

fn u16_at(block: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*block.get(at)?, *block.get(at + 1)?]))
}

fn i16_at(block: &[u8], at: usize) -> Option<i16> {
    Some(i16::from_le_bytes([*block.get(at)?, *block.get(at + 1)?]))
}

/// Decode the register block read from 0x00 (at least [`BLOCK_LEN`]
/// bytes); `None` if it is short, the SOC is over 100 % or the pack voltage
/// is not plausible
pub fn decode(block: &[u8]) -> Option<Telemetry> {
    let mv = |at| u16_at(block, at).map(|v| f64::from(v) / 1000.0);
    let ma = |at| i16_at(block, at).map(|v| f64::from(v) / 1000.0);
    let soc = *block.get(reg::SOC)?;
    let t = Telemetry {
        status: *block.get(reg::STATUS)?,
        soc: f64::from(soc) / 100.0,
        vbatt_v: mv(reg::VBATT)?,
        ibatt_a: ma(reg::IBATT)?,
        tbatt_c: f64::from(i16_at(block, reg::TBATT)?) / 10.0,
        vsolar_v: mv(reg::VSOLAR)?,
        isolar_a: ma(reg::ISOLAR)?,
        vbus_v: mv(reg::VBUS)?,
        ibus_a: ma(reg::IBUS)?,
        ijet_a: 0.0,
    };
    let plausible = t.vbatt_v >= VBATT_PLAUSIBLE_V.0 && t.vbatt_v <= VBATT_PLAUSIBLE_V.1;
    (soc <= 100 && plausible).then_some(t)
}

/// Encode telemetry as the register block (the simulated controller's
/// side; values are rounded and saturated to the register formats)
pub fn encode(t: &Telemetry, block: &mut [u8; BLOCK_LEN]) {
    use crate::fmath::round;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // saturated by the clamps
    fn u16v(v: f64) -> [u8; 2] {
        (round(v * 1000.0).clamp(0.0, 65_535.0) as u16).to_le_bytes()
    }
    #[allow(clippy::cast_possible_truncation)] // saturated by the clamp
    fn i16v(v: f64, scale: f64) -> [u8; 2] {
        (round(v * scale).clamp(-32_768.0, 32_767.0) as i16).to_le_bytes()
    }
    let mut put = |at: usize, bytes: [u8; 2]| {
        if let Some(dst) = block.get_mut(at..at + 2) {
            dst.copy_from_slice(&bytes);
        }
    };
    put(reg::VBATT, u16v(t.vbatt_v));
    put(reg::IBATT, i16v(t.ibatt_a, 1000.0));
    put(reg::TBATT, i16v(t.tbatt_c, 10.0));
    put(reg::VSOLAR, u16v(t.vsolar_v));
    put(reg::ISOLAR, i16v(t.isolar_a, 1000.0));
    put(reg::VBUS, u16v(t.vbus_v));
    put(reg::IBUS, i16v(t.ibus_a, 1000.0));
    if let Some(s) = block.get_mut(reg::STATUS) {
        *s = t.status;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0–100
    if let Some(s) = block.get_mut(reg::SOC) {
        *s = round(t.soc * 100.0).clamp(0.0, 100.0) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Telemetry {
        Telemetry {
            status: status::SUNLIT | status::CHARGING,
            soc: 0.83,
            vbatt_v: 15.912,
            ibatt_a: 0.312,
            tbatt_c: -4.5,
            vsolar_v: 13.2,
            isolar_a: 0.4,
            vbus_v: 5.01,
            ibus_a: 1.25,
            ijet_a: 0.0,
        }
    }

    #[test]
    fn round_trip() {
        let mut block = [0u8; BLOCK_LEN];
        encode(&sample(), &mut block);
        let t = decode(&block).unwrap();
        assert_eq!(t, sample());
        assert!(t.sunlit());
        assert_eq!(t.discharge_a(), 0.0);
    }

    #[test]
    fn icd_examples() {
        let mut block = [0u8; BLOCK_LEN];
        block[0x10..0x12].copy_from_slice(&0x2F_DAu16.to_le_bytes()); // 12.25 V
        block[0x12..0x14].copy_from_slice(&0xFC18u16.to_le_bytes()); // -1000 mA
        block[0x14..0x16].copy_from_slice(&0xFF38u16.to_le_bytes()); // -20.0 C
        block[0x16] = 0x4B; // 75 %
        let t = decode(&block).unwrap();
        assert!((t.vbatt_v - 12.25).abs() < 1e-9);
        assert!((t.ibatt_a + 1.0).abs() < 1e-9);
        assert!((t.discharge_a() - 1.0).abs() < 1e-9);
        assert!((t.tbatt_c + 20.0).abs() < 1e-9);
        assert!((t.soc - 0.75).abs() < 1e-9);
        assert!(!t.sunlit());
    }

    #[test]
    fn blank_short_or_implausible_blocks_are_refused() {
        assert_eq!(decode(&[0u8; BLOCK_LEN]), None);
        assert_eq!(decode(&[0u8; BLOCK_LEN - 1]), None);
        let mut block = [0u8; BLOCK_LEN];
        let mut t = sample();
        t.vbatt_v = 8.4; // a 2S value
        encode(&t, &mut block);
        assert_eq!(decode(&block), None);
        encode(&sample(), &mut block);
        block[0x16] = 101;
        assert_eq!(decode(&block), None);
    }

    #[test]
    fn encode_saturates() {
        let mut block = [0u8; BLOCK_LEN];
        let mut t = sample();
        t.ibatt_a = 99.0;
        t.soc = 1.7;
        encode(&t, &mut block);
        assert_eq!(i16::from_le_bytes([block[0x12], block[0x13]]), i16::MAX);
        assert_eq!(block[0x16], 100);
    }

    #[test]
    fn imon_jet_is_milliamperes_lsb_first() {
        assert_eq!(imon_jet_a([0xF4, 0x01]), 0.5);
    }
}
