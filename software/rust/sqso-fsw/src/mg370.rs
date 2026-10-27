//! Epson M-G370PDT0 gyro (SPI3, `hal_spi` bus 1; SQSO-OBC-BRD-001 B §9.2).
//!
//! The register interface of the M-G370 family (Epson's M-G370 data sheet
//! and its reference driver, `ess_imu_driver` `sensor_epsonCommon.c`):
//! 16-bit SPI frames in mode 3, two register windows selected through
//! `WIN_CTRL` (0x7E). A write is one frame, `0x80 | address` then the byte;
//! a read is the frame `address & 0x7E, 0x00`, a stall of at least 20 µs,
//! then the 16-bit value in the next frame. A burst read is the frame
//! `0x80, 0x00`, a stall of 45 µs, then the configured words.
//!
//! Bring-up, one step a pass so no pass waits for the part:
//!
//! 1. `/RST` low at least 100 ms ([`RESET_LOW_MS`]), then high;
//! 2. after [`BOOT_MS`] (the PDT0's 800 ms), `GLOB_CMD.NOT_READY` clear,
//!    `ID` = 0x5345 and no `DIAG_STAT` error;
//! 3. configuration: data ready on, active high (`MSC_CTRL` 0x06), 100 Sps
//!    (`SMPL_CTRL` 0x0A) with the 32-tap moving average (`FILTER_CTRL`
//!    0x05), UART auto off, burst of flags, temperature and the three
//!    16-bit rates with the count and the checksum (`BURST_CTRL1` 0xE003,
//!    `BURST_CTRL2` 0); `BURST_CTRL1` read back;
//! 4. self-test (`MSC_CTRL.SELF_TEST`), [`SELF_TEST_MS`], then the bit clear
//!    and `DIAG_STAT` bits 14–11 clear;
//! 5. sampling mode (`MODE_CTRL` 0x01).
//!
//! Then each read is a burst when `DRDY` is high: the words are checked by
//! their checksum and the `EA` flag, the rates scaled by 1/150 °/s per LSB
//! and the temperature by 1/256 °C per LSB about 25 °C (PDT0 values).
//! [`MAX_ERRORS`] bad reads in a row fail the part (the sensor manager then
//! power-cycles it).
//!
//! @requirement SRS-F140 ADCS sensor drivers on the OBC buses

use crate::bus::{BusError, SpiDevice, be16};

/// `/RST` low at least this long, ms (data sheet: 100 ms; margin)
pub const RESET_LOW_MS: u64 = 120;
/// Power-on (reset release) to the first command, ms (PDT0: 800 ms)
pub const BOOT_MS: u64 = 800;
/// Still not ready this long after the reset release: failed, ms
pub const BOOT_TIMEOUT_MS: u64 = 2_000;
/// Self-test time, ms (PDT0: 80 ms; margin)
pub const SELF_TEST_MS: u64 = 100;
/// Bad reads in a row that fail the part
pub const MAX_ERRORS: u32 = 3;
/// Rate scale, °/s per LSB (16-bit output)
pub const RATE_DPS_PER_LSB: f64 = 1.0 / 150.0;
/// Temperature scale, °C per LSB about 25 °C
pub const TEMP_C_PER_LSB: f64 = 0.003_906_25;
/// The `ID` register's value
pub const ID: u16 = 0x5345;

/// Register addresses (window 0 unless noted)
pub mod reg {
    /// MODE_CTRL (high byte 0x03: the mode command)
    pub const MODE_CTRL: u8 = 0x02;
    /// DIAG_STAT
    pub const DIAG_STAT: u8 = 0x04;
    /// ID
    pub const ID: u8 = 0x4C;
    /// MSC_CTRL (window 1)
    pub const MSC_CTRL: u8 = 0x02;
    /// SMPL_CTRL (window 1)
    pub const SMPL_CTRL: u8 = 0x04;
    /// FILTER_CTRL (window 1)
    pub const FILTER_CTRL: u8 = 0x06;
    /// UART_CTRL (window 1)
    pub const UART_CTRL: u8 = 0x08;
    /// GLOB_CMD (window 1)
    pub const GLOB_CMD: u8 = 0x0A;
    /// BURST_CTRL1 (window 1)
    pub const BURST_CTRL1: u8 = 0x0C;
    /// BURST_CTRL2 (window 1)
    pub const BURST_CTRL2: u8 = 0x0E;
    /// WIN_CTRL (both windows)
    pub const WIN_CTRL: u8 = 0x7E;
}

/// `GLOB_CMD.NOT_READY`, `MSC_CTRL.SELF_TEST` (in the 16-bit value)
const BIT10: u16 = 1 << 10;
/// `DIAG_STAT` self-test result bits
const DIAG_SELF_TEST: u16 = 0x7800;
/// `DIAG_STAT` hard error bits (flash, hardware)
const DIAG_ERRORS: u16 = 0x0064;
/// `BURST_CTRL1` as configured: FLAG, TEMP, GYRO (high byte), COUNT and
/// checksum (low byte)
pub const BURST_CTRL1: u16 = 0xE003;
/// Words in a burst: flags, temperature, three rates, count, checksum
pub const BURST_WORDS: usize = 7;
/// The burst command
const CMD_BURST: u8 = 0x80;
/// Register stall, µs, and the burst stalls
const STALL_US: u32 = 20;
const BURST_STALL_US: u32 = 45;
const WORD_STALL_US: u32 = 4;

/// One gyro sample
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sample {
    /// Body rates (sensor axes), rad/s
    pub rate: [f64; 3],
    /// Sensor temperature, °C
    pub temp_c: f64,
    /// The part's sample counter
    pub count: u16,
}

/// Why the part failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    /// No answer, or still not ready after [`BOOT_TIMEOUT_MS`]
    NotReady,
    /// The `ID` register is not [`ID`]
    WrongId,
    /// `DIAG_STAT` reports an error, or the configuration did not read back
    Config,
    /// The self-test failed
    SelfTest,
    /// [`MAX_ERRORS`] bad reads in a row
    Data,
}

/// Where the bring-up is
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Rail off
    Off,
    /// `/RST` low since this time, ms
    Reset(u64),
    /// Released since this time, ms
    Boot(u64),
    /// Self-test started at this time, ms
    SelfTest(u64),
    /// Sampling
    Running,
    /// Failed (the manager power-cycles it)
    Failed(Failure),
}

/// The driver's state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mg370 {
    /// Where the bring-up is
    pub state: State,
    /// Bad reads in a row
    pub errors: u32,
    /// Good samples since power-on
    pub samples: u32,
}

impl Default for Mg370 {
    fn default() -> Self {
        Mg370::new()
    }
}

impl Mg370 {
    /// Rail off
    pub const fn new() -> Mg370 {
        Mg370 { state: State::Off, errors: 0, samples: 0 }
    }

    fn write(dev: &mut impl SpiDevice, window: u8, addr: u8, value: u8) -> Result<(), BusError> {
        let mut rx = [0u8; 2];
        dev.transaction(&[0x80 | reg::WIN_CTRL, window], &mut rx)?;
        dev.delay_us(STALL_US);
        dev.transaction(&[0x80 | addr, value], &mut rx)?;
        dev.delay_us(STALL_US);
        Ok(())
    }

    fn read(dev: &mut impl SpiDevice, window: u8, addr: u8) -> Result<u16, BusError> {
        let mut rx = [0u8; 2];
        dev.transaction(&[0x80 | reg::WIN_CTRL, window], &mut rx)?;
        dev.delay_us(STALL_US);
        dev.select()?;
        let r = dev.transfer(&[addr & 0x7E, 0], &mut rx).and_then(|()| {
            dev.delay_us(STALL_US);
            dev.transfer(&[0, 0], &mut rx)
        });
        dev.deselect();
        dev.delay_us(STALL_US);
        r?;
        Ok(u16::from_be_bytes(rx))
    }

    fn configure(dev: &mut impl SpiDevice) -> Result<bool, BusError> {
        let writes: [(u8, u8); 8] = [
            (reg::MSC_CTRL, 0x06),
            (reg::SMPL_CTRL + 1, 0x0A),
            (reg::FILTER_CTRL, 0x05),
            (reg::UART_CTRL, 0x00),
            (reg::BURST_CTRL1, 0x03),
            (reg::BURST_CTRL1 + 1, 0xE0),
            (reg::BURST_CTRL2 + 1, 0x00),
            (reg::MSC_CTRL + 1, 0x00),
        ];
        for (addr, value) in writes {
            Mg370::write(dev, 1, addr, value)?;
        }
        Ok(Mg370::read(dev, 1, reg::BURST_CTRL1)? == BURST_CTRL1)
    }

    fn fail(&mut self, why: Failure) {
        self.state = State::Failed(why);
    }

    /// One pass of the bring-up: `powered` is the rail (on and not
    /// tripped); `reset_low` is called with the level `/RST` must have
    pub fn step(&mut self, now_ms: u64, powered: bool, dev: &mut impl SpiDevice, reset_low: &mut impl FnMut(bool)) {
        if !powered {
            reset_low(true);
            *self = Mg370::new();
            return;
        }
        match self.state {
            State::Off => {
                reset_low(true);
                self.state = State::Reset(now_ms);
            }
            State::Reset(since) if now_ms.saturating_sub(since) >= RESET_LOW_MS => {
                reset_low(false);
                self.state = State::Boot(now_ms);
            }
            State::Boot(since) if now_ms.saturating_sub(since) >= BOOT_MS => self.boot(now_ms, since, dev),
            State::SelfTest(since) if now_ms.saturating_sub(since) >= SELF_TEST_MS => self.self_test_result(dev),
            _ => {}
        }
    }

    fn boot(&mut self, now_ms: u64, since: u64, dev: &mut impl SpiDevice) {
        let ready = Mg370::read(dev, 1, reg::GLOB_CMD).map(|v| v & BIT10 == 0);
        if ready != Ok(true) {
            if now_ms.saturating_sub(since) >= BOOT_TIMEOUT_MS {
                self.fail(Failure::NotReady);
            }
            return;
        }
        match (Mg370::read(dev, 0, reg::ID), Mg370::read(dev, 0, reg::DIAG_STAT)) {
            (Ok(ID), Ok(diag)) if diag & DIAG_ERRORS == 0 => {}
            (Ok(ID), _) => return self.fail(Failure::Config),
            _ => return self.fail(Failure::WrongId),
        }
        match Mg370::configure(dev) {
            Ok(true) => {}
            _ => return self.fail(Failure::Config),
        }
        if Mg370::write(dev, 1, reg::MSC_CTRL + 1, 0x04).is_err() {
            return self.fail(Failure::SelfTest);
        }
        self.state = State::SelfTest(now_ms);
    }

    fn self_test_result(&mut self, dev: &mut impl SpiDevice) {
        let done = Mg370::read(dev, 1, reg::MSC_CTRL).map(|v| v & BIT10 == 0);
        let diag = Mg370::read(dev, 0, reg::DIAG_STAT);
        match (done, diag) {
            (Ok(true), Ok(d)) if d & DIAG_SELF_TEST == 0 => {
                if Mg370::write(dev, 0, reg::MODE_CTRL + 1, 0x01).is_ok() {
                    self.state = State::Running;
                } else {
                    self.fail(Failure::Config);
                }
            }
            _ => self.fail(Failure::SelfTest),
        }
    }

    /// The burst words decoded and checked: `None` for a bad checksum or
    /// the error flag
    pub fn decode(words: &[u16; BURST_WORDS]) -> Option<Sample> {
        let [flags, temp, gx, gy, gz, count, checksum] = *words;
        let sum = words.iter().take(BURST_WORDS - 1).fold(0u16, |a, w| a.wrapping_add(*w));
        if sum != checksum || flags & 0x0001 != 0 {
            return None;
        }
        #[allow(clippy::cast_possible_wrap)] // the words are two's complement
        let s = |w: u16| f64::from(w as i16);
        let rad = core::f64::consts::PI / 180.0;
        Some(Sample {
            rate: [s(gx) * RATE_DPS_PER_LSB * rad, s(gy) * RATE_DPS_PER_LSB * rad, s(gz) * RATE_DPS_PER_LSB * rad],
            temp_c: s(temp) * TEMP_C_PER_LSB + 25.0,
            count,
        })
    }

    fn burst(dev: &mut impl SpiDevice) -> Result<[u16; BURST_WORDS], BusError> {
        let mut rx = [0u8; 2];
        let mut words = [0u16; BURST_WORDS];
        dev.select()?;
        let mut r = dev.transfer(&[CMD_BURST, 0], &mut rx);
        dev.delay_us(BURST_STALL_US);
        for w in &mut words {
            if r.is_err() {
                break;
            }
            r = dev.transfer(&[0, 0], &mut rx);
            *w = be16(&rx, 0).unwrap_or(0);
            dev.delay_us(WORD_STALL_US);
        }
        dev.deselect();
        r.map(|()| words)
    }

    /// The latest sample while running and `drdy` is high (`None`
    /// otherwise, or for a bad read: [`MAX_ERRORS`] in a row fail the part)
    pub fn read_sample(&mut self, drdy: bool, dev: &mut impl SpiDevice) -> Option<Sample> {
        if self.state != State::Running || !drdy {
            return None;
        }
        match Mg370::burst(dev).ok().and_then(|w| Mg370::decode(&w)) {
            Some(s) => {
                self.errors = 0;
                self.samples = self.samples.wrapping_add(1);
                Some(s)
            }
            None => {
                self.errors = self.errors.saturating_add(1);
                if self.errors >= MAX_ERRORS {
                    self.fail(Failure::Data);
                }
                None
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    extern crate std;
    use std::vec::Vec;

    /// A register model of the M-G370: two windows, the commands the driver
    /// uses, a burst output
    pub(crate) struct Model {
        pub(crate) window: u8,
        pub(crate) w: [[u8; 128]; 2],
        selected: bool,
        frame: Vec<u8>,
        pending_read: Option<u16>,
        burst: Option<usize>,
        pub(crate) burst_words: [u16; BURST_WORDS],
        pub(crate) fail: bool,
        pub(crate) self_test_fails: bool,
        pub(crate) not_ready: bool,
        pub(crate) id: u16,
    }

    impl Model {
        pub(crate) fn new() -> Model {
            let mut m = Model {
                window: 0,
                w: [[0; 128]; 2],
                selected: false,
                frame: Vec::new(),
                pending_read: None,
                burst: None,
                burst_words: [0; BURST_WORDS],
                fail: false,
                self_test_fails: false,
                not_ready: false,
                id: ID,
            };
            m.set_rates([150, -300, 0], 0);
            m
        }

        pub(crate) fn set_rates(&mut self, lsb: [i16; 3], flags: u16) {
            let mut w = [flags | 0x7000, 256, lsb[0] as u16, lsb[1] as u16, lsb[2] as u16, 7, 0];
            w[6] = w.iter().take(6).fold(0u16, |a, x| a.wrapping_add(*x));
            self.burst_words = w;
        }

        fn reg16(&self, window: u8, addr: u8) -> u16 {
            let a = usize::from(addr & 0x7E);
            let r = &self.w[usize::from(window)];
            let v = u16::from_be_bytes([r[a + 1], r[a]]);
            match (window, addr & 0x7E) {
                (0, reg::ID) => self.id,
                (1, reg::GLOB_CMD) if self.not_ready => BIT10,
                (0, reg::DIAG_STAT) if self.self_test_fails => 0x0800,
                _ => v,
            }
        }

        fn frame_done(&mut self) {
            let [a, b] = [self.frame[0], self.frame[1]];
            self.frame.clear();
            if self.burst.is_some() {
                return;
            }
            if a == CMD_BURST {
                self.burst = Some(0);
            } else if a & 0x80 != 0 {
                let addr = a & 0x7F;
                if addr == reg::WIN_CTRL {
                    self.window = b;
                } else {
                    // MSC_CTRL.SELF_TEST completes at once in the model
                    let v = if self.window == 1 && addr == reg::MSC_CTRL + 1 { b & !0x04 } else { b };
                    self.w[usize::from(self.window)][usize::from(addr)] = v;
                }
            } else {
                self.pending_read = Some(self.reg16(self.window, a));
            }
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
            self.burst = None;
            self.frame.clear();
        }
        fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), BusError> {
            assert!(self.selected);
            for (i, b) in tx.iter().enumerate() {
                rx[i] = 0;
                if let Some(n) = self.burst {
                    let word = self.burst_words.get(n / 2).copied().unwrap_or(0);
                    rx[i] = word.to_be_bytes()[n % 2];
                    self.burst = Some(n + 1);
                    continue;
                }
                if let Some(v) = self.pending_read {
                    rx[i] = v.to_be_bytes()[self.frame.len()];
                }
                self.frame.push(*b);
                if self.frame.len() == 2 {
                    if self.pending_read.is_some() && self.frame == [0, 0] {
                        self.pending_read = None;
                        self.frame.clear();
                    } else {
                        self.frame_done();
                    }
                }
            }
            Ok(())
        }
        fn delay_us(&mut self, _us: u32) {}
    }

    /// Run the bring-up from power-on to the end; the time it ended
    pub(crate) fn bring_up(g: &mut Mg370, m: &mut Model) -> u64 {
        let mut rst = Vec::new();
        let mut t = 0;
        while t < 5_000 && !matches!(g.state, State::Running | State::Failed(_)) {
            g.step(t, true, m, &mut |low| rst.push(low));
            t += 20;
        }
        assert_eq!(rst.first(), Some(&true), "reset asserted first");
        t
    }

    #[test]
    fn brings_up_self_tests_and_reads_bursts() {
        let (mut g, mut m) = (Mg370::new(), Model::new());
        let t = bring_up(&mut g, &mut m);
        assert_eq!(g.state, State::Running);
        assert!(t >= RESET_LOW_MS + BOOT_MS + SELF_TEST_MS);
        assert_eq!(m.w[1][usize::from(reg::BURST_CTRL1 + 1)], 0xE0);
        assert_eq!(m.w[0][usize::from(reg::MODE_CTRL + 1)], 0x01, "sampling");
        assert_eq!(g.read_sample(false, &mut m), None, "no data ready");
        let s = g.read_sample(true, &mut m).unwrap();
        assert!((s.rate[0] - 1.0f64.to_radians()).abs() < 1e-12);
        assert!((s.rate[1] + 2.0f64.to_radians()).abs() < 1e-12);
        assert!((s.temp_c - 26.0).abs() < 1e-12);
        assert_eq!((s.count, g.samples), (7, 1));
    }

    #[test]
    fn bad_bursts_fail_the_part_after_three() {
        let (mut g, mut m) = (Mg370::new(), Model::new());
        bring_up(&mut g, &mut m);
        m.burst_words[6] ^= 1; // checksum wrong
        for _ in 0..MAX_ERRORS {
            assert_eq!(g.read_sample(true, &mut m), None);
        }
        assert_eq!(g.state, State::Failed(Failure::Data));
        let mut w = [0u16; BURST_WORDS];
        w[0] = 1; // EA flag
        w[6] = 1;
        assert_eq!(Mg370::decode(&w), None);
    }

    #[test]
    fn failures_at_each_step_of_the_bring_up() {
        let mut m = Model::new();
        m.not_ready = true;
        let mut g = Mg370::new();
        bring_up(&mut g, &mut m);
        assert_eq!(g.state, State::Failed(Failure::NotReady));

        let mut m = Model::new();
        m.id = 0x1234;
        let mut g = Mg370::new();
        bring_up(&mut g, &mut m);
        assert_eq!(g.state, State::Failed(Failure::WrongId));

        let mut m = Model::new();
        m.self_test_fails = true;
        let mut g = Mg370::new();
        bring_up(&mut g, &mut m);
        assert_eq!(g.state, State::Failed(Failure::SelfTest));

        let mut m = Model::new();
        m.fail = true;
        let mut g = Mg370::new();
        bring_up(&mut g, &mut m);
        assert_eq!(g.state, State::Failed(Failure::NotReady), "no answer");

        // Power off resets the driver and holds /RST low
        let mut low = false;
        g.step(0, false, &mut m, &mut |l| low = l);
        assert!(low);
        assert_eq!(g.state, State::Off);
    }
}
