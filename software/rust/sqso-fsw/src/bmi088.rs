//! Bosch BMI088, the cold-spare gyro (SPI2: gyro on `hal_spi` bus 0, CS
//! PD0; accelerometer on bus 2, CS PD4; SQSO-OBC-BRD-001 B §9.3).
//!
//! From the BMI088 data sheet (BST-BMI088-DS001): the gyro's SPI read is
//! `0x80 | register` followed by the data bytes; the accelerometer's sends
//! one dummy byte before the data, and starts in I2C mode until a first
//! read toggles its chip select (the "dummy read").
//!
//! Gyro bring-up, one step a pass: [`START_MS`] after power-on, `GYRO_CHIP_ID`
//! (0x00) = 0x0F, soft reset (0x14 ← 0xB6), then after [`START_MS`] again
//! ±250 °/s (0x0F ← 0x03, 131.072 LSB per °/s), 100 Hz output with the
//! 32 Hz filter (0x10 ← 0x07), data-ready interrupt on INT3, push-pull
//! active high (0x15 ← 0x80, 0x16 ← 0x01, 0x18 ← 0x01), and the built-in
//! self-test (0x3C ← 0x01; after [`SELF_TEST_MS`] `bist_rdy` set, `bist_fail`
//! clear). The accelerometer is not used by the attitude control: the driver
//! puts it in SPI mode, checks `ACC_CHIP_ID` = 0x1E and leaves it in its
//! power-on suspend state (`ACC_PWR_CTRL` 0).
//!
//! @requirement SRS-F140 ADCS sensor drivers on the OBC buses

use crate::bus::{BusError, SpiDevice};

/// Power-on and soft-reset time, ms (data sheet: 30 ms)
pub const START_MS: u64 = 40;
/// Self-test time, ms
pub const SELF_TEST_MS: u64 = 60;
/// Gyro chip ID
pub const GYRO_ID: u8 = 0x0F;
/// Accelerometer chip ID
pub const ACCEL_ID: u8 = 0x1E;
/// Rate scale at ±250 °/s, LSB per °/s
pub const LSB_PER_DPS: f64 = 131.072;
/// Bad reads in a row that fail the part
pub const MAX_ERRORS: u32 = 3;

/// Gyro registers
pub mod reg {
    /// GYRO_CHIP_ID
    pub const CHIP_ID: u8 = 0x00;
    /// RATE_X_LSB (six bytes: X, Y, Z, LSB first)
    pub const RATE_X_LSB: u8 = 0x02;
    /// GYRO_RANGE
    pub const RANGE: u8 = 0x0F;
    /// GYRO_BANDWIDTH
    pub const BANDWIDTH: u8 = 0x10;
    /// GYRO_SOFTRESET
    pub const SOFTRESET: u8 = 0x14;
    /// GYRO_INT_CTRL
    pub const INT_CTRL: u8 = 0x15;
    /// INT3_INT4_IO_CONF
    pub const IO_CONF: u8 = 0x16;
    /// INT3_INT4_IO_MAP
    pub const IO_MAP: u8 = 0x18;
    /// GYRO_SELF_TEST
    pub const SELF_TEST: u8 = 0x3C;
}

/// The configuration writes after the soft reset
pub const CONFIG: [(u8, u8); 5] =
    [(reg::RANGE, 0x03), (reg::BANDWIDTH, 0x07), (reg::INT_CTRL, 0x80), (reg::IO_CONF, 0x01), (reg::IO_MAP, 0x01)];

/// Why the part failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// No answer or a wrong gyro chip ID
    WrongId,
    /// A configuration register did not read back
    Config,
    /// The built-in self-test failed
    SelfTest,
    /// [`MAX_ERRORS`] bad reads in a row
    Data,
}

/// Where the bring-up is
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Rail off
    Off,
    /// Powered since this time, ms
    PowerOn(u64),
    /// Soft reset at this time, ms
    Reset(u64),
    /// Self-test started at this time, ms
    SelfTest(u64),
    /// Measuring
    Running,
    /// Failed
    Failed(Failure),
}

/// The driver's state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bmi088 {
    /// Where the gyro's bring-up is
    pub state: State,
    /// The accelerometer answered with its chip ID
    pub accel_present: bool,
    /// Bad reads in a row
    pub errors: u32,
}

impl Default for Bmi088 {
    fn default() -> Self {
        Bmi088::new()
    }
}

fn read_gyro(dev: &mut impl SpiDevice, reg: u8, out: &mut [u8]) -> Result<(), BusError> {
    let mut tx = [0u8; 8];
    let mut rx = [0u8; 8];
    let n = out.len().min(7) + 1;
    if let Some(first) = tx.first_mut() {
        *first = 0x80 | reg;
    }
    dev.transaction(tx.get(..n).unwrap_or(&[]), rx.get_mut(..n).unwrap_or(&mut []))?;
    for (o, r) in out.iter_mut().zip(rx.iter().skip(1)) {
        *o = *r;
    }
    Ok(())
}

fn write_gyro(dev: &mut impl SpiDevice, reg: u8, value: u8) -> Result<(), BusError> {
    let mut rx = [0u8; 2];
    dev.transaction(&[reg & 0x7F, value], &mut rx)
}

fn read1(dev: &mut impl SpiDevice, reg: u8) -> Result<u8, BusError> {
    let mut v = [0u8];
    read_gyro(dev, reg, &mut v)?;
    Ok(v[0])
}

impl Bmi088 {
    /// Rail off
    pub const fn new() -> Bmi088 {
        Bmi088 { state: State::Off, accel_present: false, errors: 0 }
    }

    /// The accelerometer's chip ID after the dummy read that selects SPI
    pub fn probe_accel(accel: &mut impl SpiDevice) -> bool {
        let mut rx = [0u8; 3];
        let dummy = accel.transaction(&[0x80, 0, 0], &mut rx);
        let id = accel.transaction(&[0x80, 0, 0], &mut rx);
        dummy.is_ok() && id.is_ok() && rx[2] == ACCEL_ID
    }

    /// One pass of the bring-up (`powered`: the rail is on and not tripped)
    pub fn step(&mut self, now_ms: u64, powered: bool, gyro: &mut impl SpiDevice, accel: &mut impl SpiDevice) {
        if !powered {
            *self = Bmi088::new();
            return;
        }
        match self.state {
            State::Off => self.state = State::PowerOn(now_ms),
            State::PowerOn(since) if now_ms.saturating_sub(since) >= START_MS => {
                self.accel_present = Bmi088::probe_accel(accel);
                if read1(gyro, reg::CHIP_ID) != Ok(GYRO_ID) {
                    self.state = State::Failed(Failure::WrongId);
                } else if write_gyro(gyro, reg::SOFTRESET, 0xB6).is_ok() {
                    self.state = State::Reset(now_ms);
                } else {
                    self.state = State::Failed(Failure::Config);
                }
            }
            State::Reset(since) if now_ms.saturating_sub(since) >= START_MS => {
                let ok = CONFIG.iter().all(|&(r, v)| {
                    // BANDWIDTH reads back with bit 7 set
                    let want = if r == reg::BANDWIDTH { v | 0x80 } else { v };
                    write_gyro(gyro, r, v).is_ok() && read1(gyro, r) == Ok(want)
                });
                if ok && write_gyro(gyro, reg::SELF_TEST, 0x01).is_ok() {
                    self.state = State::SelfTest(now_ms);
                } else {
                    self.state = State::Failed(Failure::Config);
                }
            }
            State::SelfTest(since) if now_ms.saturating_sub(since) >= SELF_TEST_MS => {
                // bist_rdy (bit 1) set, bist_fail (bit 2) clear, rate_ok (bit 4) set
                self.state = match read1(gyro, reg::SELF_TEST) {
                    Ok(v) if v & 0x16 == 0x12 => State::Running,
                    _ => State::Failed(Failure::SelfTest),
                };
            }
            _ => {}
        }
    }

    /// The body rates (sensor axes), rad/s, while running and `drdy`
    pub fn read_rate(&mut self, drdy: bool, gyro: &mut impl SpiDevice) -> Option<[f64; 3]> {
        if self.state != State::Running || !drdy {
            return None;
        }
        let mut b = [0u8; 6];
        if read_gyro(gyro, reg::RATE_X_LSB, &mut b).is_err() {
            self.errors = self.errors.saturating_add(1);
            if self.errors >= MAX_ERRORS {
                self.state = State::Failed(Failure::Data);
            }
            return None;
        }
        self.errors = 0;
        let rad = core::f64::consts::PI / 180.0;
        let axis = |lo: u8, hi: u8| f64::from(i16::from_le_bytes([lo, hi])) / LSB_PER_DPS * rad;
        Some([axis(b[0], b[1]), axis(b[2], b[3]), axis(b[4], b[5])])
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A register model of the BMI088's gyro (or, with `dummy`, of its
    /// accelerometer: one dummy byte before the data)
    pub(crate) struct Model {
        pub(crate) regs: [u8; 128],
        pub(crate) dummy: bool,
        pub(crate) fail: bool,
        pub(crate) bist: u8,
        selected: bool,
    }

    impl Model {
        pub(crate) fn gyro() -> Model {
            let mut regs = [0u8; 128];
            regs[usize::from(reg::CHIP_ID)] = GYRO_ID;
            // +1 °/s, -0.5 °/s, 0
            regs[2..8].copy_from_slice(&[0x83, 0x00, 0xBE, 0xFF, 0, 0]);
            Model { regs, dummy: false, fail: false, bist: 0x12, selected: false }
        }
        pub(crate) fn accel() -> Model {
            let mut m = Model::gyro();
            m.regs[0] = ACCEL_ID;
            m.dummy = true;
            m
        }
    }

    impl SpiDevice for Model {
        fn select(&mut self) -> Result<(), BusError> {
            if self.fail {
                return Err(BusError);
            }
            self.selected = true;
            Ok(())
        }
        fn deselect(&mut self) {
            self.selected = false;
        }
        fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), BusError> {
            let addr = usize::from(tx[0] & 0x7F);
            if tx[0] & 0x80 == 0 {
                self.regs[addr] = if addr == usize::from(reg::SELF_TEST) { self.bist } else { tx[1] };
                if addr == usize::from(reg::BANDWIDTH) {
                    self.regs[addr] |= 0x80;
                }
                return Ok(());
            }
            let skip = 1 + usize::from(self.dummy);
            for (i, r) in rx.iter_mut().enumerate() {
                *r = if i >= skip { self.regs[addr + i - skip] } else { 0 };
            }
            Ok(())
        }
        fn delay_us(&mut self, _us: u32) {}
    }

    fn run(b: &mut Bmi088, g: &mut Model, a: &mut Model) {
        let mut t = 0;
        while t < 1000 && !matches!(b.state, State::Running | State::Failed(_)) {
            b.step(t, true, g, a);
            t += 20;
        }
    }

    #[test]
    fn brings_up_and_reads_rates() {
        let (mut b, mut g, mut a) = (Bmi088::new(), Model::gyro(), Model::accel());
        run(&mut b, &mut g, &mut a);
        assert_eq!(b.state, State::Running);
        assert!(b.accel_present);
        assert_eq!(g.regs[usize::from(reg::RANGE)], 0x03);
        assert_eq!(b.read_rate(false, &mut g), None);
        let w = b.read_rate(true, &mut g).unwrap();
        assert!((w[0] - (131.0 / LSB_PER_DPS).to_radians()).abs() < 1e-12);
        assert!((w[1] - (-66.0 / LSB_PER_DPS).to_radians()).abs() < 1e-12);
        g.fail = true;
        for _ in 0..MAX_ERRORS {
            assert_eq!(b.read_rate(true, &mut g), None);
        }
        assert_eq!(b.state, State::Failed(Failure::Data));
        b.step(0, false, &mut g, &mut a);
        assert_eq!(b, Bmi088::new());
    }

    #[test]
    fn wrong_id_bad_config_and_failed_self_test() {
        let (mut b, mut g, mut a) = (Bmi088::new(), Model::gyro(), Model::accel());
        g.regs[0] = 0;
        a.regs[0] = 0;
        run(&mut b, &mut g, &mut a);
        assert_eq!(b.state, State::Failed(Failure::WrongId));
        assert!(!b.accel_present);

        let (mut b, mut g, mut a) = (Bmi088::new(), Model::gyro(), Model::accel());
        g.bist = 0x06; // rdy and fail
        run(&mut b, &mut g, &mut a);
        assert_eq!(b.state, State::Failed(Failure::SelfTest));
    }
}
