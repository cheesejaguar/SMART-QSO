//! The ADCS sensors on the OBC board (SQSO-OBC-BRD-001 B §9; RID AV-03):
//! their latch-up-protected rails, the drivers' bring-up, the cold-spare
//! gyro and the magnetometer's conversion in the coils-off window.
//!
//! **Rails.** Each sensor group has its own TPS2553-Q1 switch: `GYRO_EN`
//! (M-G370PDT0), `IMU2_EN` (BMI088, cold spare) and `SNS_EN` (RM3100 and
//! the ADS7138 sun-sensor ADC). Each switch's EN node is read back
//! (`*_RB`): low with the enable high is a latch-up trip; the switch retries
//! by itself after about 0.13 s. A trip is counted and the group's driver
//! starts its bring-up again once the rail is back.
//!
//! **Gyros.** The M-G370 is used while it works. A failed bring-up or
//! [`crate::mg370::MAX_ERRORS`] bad reads power-cycle its rail
//! ([`CYCLE_OFF_MS`]); after [`MAX_FAILURES`] failures the BMI088 is powered
//! instead (and the M-G370's rail stays off), with the same rule; with both
//! failed there is no gyro and the control runs on the magnetometer rate
//! estimate (`sqso_fsw::adcs`, the no-gyro gains).
//!
//! **Magnetometer.** A single conversion is started once per control
//! period, [`crate::adcs::MAG_SETTLE_MS`] after the coils' burst ends (the
//! 950 ms point), and read at the next control cycle (1000 ms): the sample
//! falls in the coils-off window of ICD-004 §12.2.
//!
//! @requirement SRS-F140 ADCS sensor drivers on the OBC buses

use crate::ads7138;
use crate::bmi088::{self, Bmi088};
use crate::bus::{I2cBus, SpiDevice};
use crate::mg370::{self, Mg370};
use crate::rm3100::Rm3100;

/// A failed group's rail is off this long before it is tried again, ms
pub const CYCLE_OFF_MS: u64 = 1_000;
/// Failures of a gyro before the next one is used (or none)
pub const MAX_FAILURES: u32 = 3;
/// The sensor rail is up this long before its parts are configured, ms
pub const SNS_SETTLE_MS: u64 = 20;
/// A failed magnetometer or sun-sensor ADC is tried again after this, ms
pub const SNS_RETRY_MS: u64 = 10_000;

/// The pins the sensors use (`hal_gpio.h`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin {
    /// `GPIO_PIN_GYRO_EN`
    GyroEn,
    /// `GPIO_PIN_IMU2_EN`
    Imu2En,
    /// `GPIO_PIN_SNS_EN`
    SnsEn,
    /// `GPIO_PIN_GYRO_RB`
    GyroRb,
    /// `GPIO_PIN_IMU2_RB`
    Imu2Rb,
    /// `GPIO_PIN_SNS_RB`
    SnsRb,
    /// `GPIO_PIN_GYRO_RST_N`
    GyroRstN,
    /// `GPIO_PIN_GYRO_DRDY`
    GyroDrdy,
    /// `GPIO_PIN_IMU2_DRDY`
    Imu2Drdy,
    /// `GPIO_PIN_MAG_DRDY`
    MagDrdy,
}

/// The sensors' pins
pub trait Pins {
    /// Drive an output
    fn set(&mut self, pin: Pin, high: bool);
    /// Read an input
    fn get(&mut self, pin: Pin) -> bool;
}

/// The devices, one borrow each
pub struct Devices<'a, M, G, A, I, P> {
    /// M-G370 (SPI3)
    pub mg370: &'a mut M,
    /// BMI088 gyro (SPI2, CS PD0)
    pub bmi_gyro: &'a mut G,
    /// BMI088 accelerometer (SPI2, CS PD4)
    pub bmi_accel: &'a mut A,
    /// I2C4: RM3100 and ADS7138
    pub i2c: &'a mut I,
    /// The pins
    pub pins: &'a mut P,
}

/// What changed this pass, for the fault log
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Events {
    /// A rail tripped: gyro, BMI088, sensors
    pub tripped: [bool; 3],
    /// The M-G370 failed (its rail is being cycled)
    pub mg370_failed: Option<mg370::Failure>,
    /// The BMI088 failed
    pub bmi088_failed: Option<bmi088::Failure>,
    /// The M-G370 was given up for the BMI088
    pub to_backup: bool,
    /// Both gyros given up
    pub no_gyro: bool,
    /// The magnetometer or the sun-sensor ADC did not configure
    pub sns_failed: bool,
}

/// Which gyro is in use
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gyro {
    /// The M-G370PDT0
    Primary,
    /// The BMI088
    Backup,
    /// Neither (both failed)
    None,
}

/// The manager's state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sensors {
    /// The primary gyro's driver
    pub mg370: Mg370,
    /// The backup gyro's driver
    pub bmi088: Bmi088,
    /// The magnetometer's driver
    pub mag: Rm3100,
    /// The sun-sensor ADC configured
    pub sun_ready: bool,
    /// The gyro in use
    pub gyro: Gyro,
    /// Failures of the gyro in use
    pub failures: u32,
    /// Trips counted: gyro, BMI088, sensors
    pub trips: [u32; 3],
    /// A rail held off for a power cycle until this time, ms (gyro rail)
    gyro_off_until: Option<u64>,
    /// The sensor rail has been up since, ms (`None`: off or tripped)
    sns_up_since: Option<u64>,
    /// The magnetometer and sun ADC were last tried, ms
    sns_tried_ms: Option<u64>,
    /// A trip seen on each rail at the last pass
    tripped: [bool; 3],
    /// When the magnetometer was last triggered, ms
    mag_trigger_ms: Option<u64>,
}

impl Default for Sensors {
    fn default() -> Self {
        Sensors::new()
    }
}

/// The readings of one control cycle
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Readings {
    /// Field, µT (sensor axes = body axes on board A2)
    pub field: Option<[f64; 3]>,
    /// Sun vector, length the intensity
    pub sun: Option<[f64; 3]>,
    /// Gyro rate, rad/s, sensor axes
    pub rate: Option<[f64; 3]>,
    /// The sun-sensor ADC's other inputs: the gyro-area temperature, °C,
    /// and the coil supply, V
    pub board: Option<(f64, f64)>,
}

impl Sensors {
    /// Everything off
    pub const fn new() -> Sensors {
        Sensors {
            mg370: Mg370::new(),
            bmi088: Bmi088::new(),
            mag: Rm3100::new(),
            sun_ready: false,
            gyro: Gyro::Primary,
            failures: 0,
            trips: [0; 3],
            gyro_off_until: None,
            sns_up_since: None,
            sns_tried_ms: None,
            tripped: [false; 3],
            mag_trigger_ms: None,
        }
    }

    /// One pass: the rails (`powered`: the ADCS rail is on), trips, the
    /// gyros' bring-up, the sensor rail's parts
    pub fn step<M, G, A, I, P>(&mut self, now_ms: u64, powered: bool, d: &mut Devices<'_, M, G, A, I, P>) -> Events
    where
        M: SpiDevice,
        G: SpiDevice,
        A: SpiDevice,
        I: I2cBus,
        P: Pins,
    {
        let mut ev = Events::default();
        if self.gyro_off_until.is_some_and(|t| now_ms >= t) {
            self.gyro_off_until = None;
        }
        let cycling = self.gyro_off_until.is_some();
        let rails = [
            powered && self.gyro == Gyro::Primary && !cycling,
            powered && self.gyro == Gyro::Backup && !cycling,
            powered,
        ];
        let pins_en = [Pin::GyroEn, Pin::Imu2En, Pin::SnsEn];
        let pins_rb = [Pin::GyroRb, Pin::Imu2Rb, Pin::SnsRb];
        let mut up = [false; 3];
        for i in 0..3 {
            let (Some(&on), Some(&en), Some(&rb)) = (rails.get(i), pins_en.get(i), pins_rb.get(i)) else { continue };
            d.pins.set(en, on);
            let trip = on && !d.pins.get(rb);
            if let (Some(seen), Some(count), Some(e), Some(u)) =
                (self.tripped.get_mut(i), self.trips.get_mut(i), ev.tripped.get_mut(i), up.get_mut(i))
            {
                if trip && !*seen {
                    *count = count.saturating_add(1);
                    *e = true;
                }
                *seen = trip;
                *u = on && !trip;
            }
        }
        let [gyro_up, imu2_up, sns_up] = up;
        let pins = &mut *d.pins;
        self.mg370.step(now_ms, gyro_up, d.mg370, &mut |low| pins.set(Pin::GyroRstN, !low));
        self.bmi088.step(now_ms, imu2_up, d.bmi_gyro, d.bmi_accel);
        self.gyro_health(now_ms, &mut ev);
        self.sensor_rail(now_ms, sns_up, d.i2c, &mut ev);
        ev
    }

    fn gyro_health(&mut self, now_ms: u64, ev: &mut Events) {
        let failed = match (self.gyro, self.mg370.state, self.bmi088.state) {
            (Gyro::Primary, mg370::State::Failed(f), _) => {
                ev.mg370_failed = Some(f);
                self.mg370 = Mg370::new();
                true
            }
            (Gyro::Backup, _, bmi088::State::Failed(f)) => {
                ev.bmi088_failed = Some(f);
                self.bmi088 = Bmi088::new();
                true
            }
            _ => false,
        };
        if !failed {
            return;
        }
        self.failures = self.failures.saturating_add(1);
        self.gyro_off_until = Some(now_ms.saturating_add(CYCLE_OFF_MS));
        if self.failures >= MAX_FAILURES {
            self.failures = 0;
            self.gyro = match self.gyro {
                Gyro::Primary => {
                    ev.to_backup = true;
                    Gyro::Backup
                }
                _ => {
                    ev.no_gyro = true;
                    Gyro::None
                }
            };
        }
    }

    fn sensor_rail(&mut self, now_ms: u64, up: bool, i2c: &mut impl I2cBus, ev: &mut Events) {
        if !up {
            self.sns_up_since = None;
            self.mag = Rm3100::new();
            self.sun_ready = false;
            self.sns_tried_ms = None;
            return;
        }
        let since = *self.sns_up_since.get_or_insert(now_ms);
        let settled = now_ms.saturating_sub(since) >= SNS_SETTLE_MS;
        let retry = self.sns_tried_ms.is_none_or(|t| now_ms.saturating_sub(t) >= SNS_RETRY_MS);
        if settled && retry && !(self.mag.ready && self.sun_ready) {
            self.sns_tried_ms = Some(now_ms);
            if !self.mag.ready {
                let _ = self.mag.init(i2c);
            }
            if !self.sun_ready {
                self.sun_ready = ads7138::init(i2c);
            }
            ev.sns_failed = !(self.mag.ready && self.sun_ready);
        }
        if self.mag.failed && self.sns_tried_ms.is_some_and(|t| now_ms.saturating_sub(t) >= SNS_RETRY_MS) {
            self.mag = Rm3100::new();
        }
    }

    /// Start the magnetometer's conversion when the window opens: at
    /// `window_ms` (the coils' burst end plus the settle time, or the next
    /// control cycle less the window when the coils were not driven), once
    /// per control period
    pub fn trigger_mag(&mut self, now_ms: u64, window_ms: u64, i2c: &mut impl I2cBus) {
        let fresh = self.mag_trigger_ms.is_some_and(|t| t >= window_ms);
        if now_ms >= window_ms && !fresh && self.mag.ready && !self.mag.pending {
            if self.mag.trigger(i2c).is_ok() {
                self.mag_trigger_ms = Some(now_ms);
            }
        }
    }

    /// The control cycle's readings
    pub fn read<M, G, A, I, P>(&mut self, d: &mut Devices<'_, M, G, A, I, P>) -> Readings
    where
        M: SpiDevice,
        G: SpiDevice,
        A: SpiDevice,
        I: I2cBus,
        P: Pins,
    {
        let field = self.mag.read_field(d.pins.get(Pin::MagDrdy), d.i2c);
        let adc = if self.sun_ready { ads7138::read(d.i2c).ok() } else { None };
        let rate = match self.gyro {
            Gyro::Primary => self.mg370.read_sample(d.pins.get(Pin::GyroDrdy), d.mg370).map(|s| s.rate),
            Gyro::Backup => self.bmi088.read_rate(d.pins.get(Pin::Imu2Drdy), d.bmi_gyro),
            Gyro::None => None,
        };
        Readings { field, sun: adc.map(|r| r.sun()), rate, board: adc.map(|r| (r.sensor_temp_c(), r.vbat_mtq())) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bmi088::tests::Model as BmiModel;
    use crate::bus::BusError;
    use crate::mg370::tests::Model as MgModel;

    /// The I2C4 bus with the RM3100 (0x20) and ADS7138 (0x10) answering
    struct Bus4 {
        mag: crate::bus::mock::I2cRegs,
        adc_status: u8,
        channel: u8,
        fail: bool,
    }

    impl I2cBus for Bus4 {
        fn write(&mut self, a: u8, d: &[u8]) -> Result<(), BusError> {
            if self.fail {
                return Err(BusError);
            }
            match (a, d) {
                (ads7138::ADDRESS, [0x08, 0x11, ch]) => self.channel = *ch,
                (ads7138::ADDRESS, _) => {}
                _ => self.mag.write(a, d)?,
            }
            Ok(())
        }
        fn write_read(&mut self, a: u8, w: &[u8], r: &mut [u8]) -> Result<(), BusError> {
            if self.fail {
                return Err(BusError);
            }
            if a == ads7138::ADDRESS {
                r[0] = self.adc_status;
                return Ok(());
            }
            self.mag.write_read(a, w, r)
        }
        fn read(&mut self, a: u8, r: &mut [u8]) -> Result<(), BusError> {
            if a == ads7138::ADDRESS {
                let code: u16 = if self.channel == 0 { 3084 << 4 } else { 0 };
                r.copy_from_slice(&code.to_be_bytes());
                return Ok(());
            }
            self.mag.read(a, r)
        }
    }

    #[derive(Default)]
    struct Board {
        en: [bool; 3],
        trip: [bool; 3],
        rst_low: bool,
        drdy: bool,
    }

    impl Pins for Board {
        fn set(&mut self, pin: Pin, high: bool) {
            match pin {
                Pin::GyroEn => self.en[0] = high,
                Pin::Imu2En => self.en[1] = high,
                Pin::SnsEn => self.en[2] = high,
                Pin::GyroRstN => self.rst_low = !high,
                _ => {}
            }
        }
        fn get(&mut self, pin: Pin) -> bool {
            match pin {
                Pin::GyroRb => self.en[0] && !self.trip[0],
                Pin::Imu2Rb => self.en[1] && !self.trip[1],
                Pin::SnsRb => self.en[2] && !self.trip[2],
                Pin::GyroDrdy | Pin::Imu2Drdy | Pin::MagDrdy => self.drdy,
                _ => false,
            }
        }
    }

    struct Bench {
        mg: MgModel,
        bg: BmiModel,
        ba: BmiModel,
        bus: Bus4,
        pins: Board,
    }

    impl Bench {
        fn new() -> Bench {
            let mut mag = crate::bus::mock::I2cRegs::new(crate::rm3100::ADDRESS);
            mag.regs[usize::from(crate::rm3100::reg::REVID)] = crate::rm3100::REVID;
            mag.regs[0x24..0x27].copy_from_slice(&[0x00, 0x08, 0xCA]);
            Bench {
                mg: MgModel::new(),
                bg: BmiModel::gyro(),
                ba: BmiModel::accel(),
                bus: Bus4 { mag, adc_status: 0x81, channel: 0, fail: false },
                pins: Board { drdy: true, ..Board::default() },
            }
        }
        fn run(&mut self, s: &mut Sensors, from: u64, to: u64, powered: bool) -> Events {
            let mut all = Events::default();
            let mut t = from;
            while t < to {
                let mut d = Devices {
                    mg370: &mut self.mg,
                    bmi_gyro: &mut self.bg,
                    bmi_accel: &mut self.ba,
                    i2c: &mut self.bus,
                    pins: &mut self.pins,
                };
                let e = s.step(t, powered, &mut d);
                for i in 0..3 {
                    all.tripped[i] |= e.tripped[i];
                }
                all.to_backup |= e.to_backup;
                all.no_gyro |= e.no_gyro;
                all.sns_failed |= e.sns_failed;
                all.mg370_failed = all.mg370_failed.or(e.mg370_failed);
                all.bmi088_failed = all.bmi088_failed.or(e.bmi088_failed);
                t += 20;
            }
            all
        }
        fn read(&mut self, s: &mut Sensors) -> Readings {
            let mut d = Devices {
                mg370: &mut self.mg,
                bmi_gyro: &mut self.bg,
                bmi_accel: &mut self.ba,
                i2c: &mut self.bus,
                pins: &mut self.pins,
            };
            s.read(&mut d)
        }
    }

    #[test]
    fn powers_brings_up_and_reads_every_sensor() {
        let (mut b, mut s) = (Bench::new(), Sensors::new());
        let e = b.run(&mut s, 0, 2_000, true);
        assert_eq!(e, Events::default());
        assert_eq!(b.pins.en, [true, false, true], "the backup gyro is cold");
        assert_eq!(s.mg370.state, mg370::State::Running);
        assert!(s.mag.ready);
        assert!(s.sun_ready);
        s.trigger_mag(2_000, 1_950, &mut b.bus);
        assert!(s.mag.pending);
        s.trigger_mag(2_010, 1_950, &mut b.bus);
        let r = b.read(&mut s);
        assert_eq!(r.field, Some([30.0, 0.0, 0.0]));
        assert!(r.rate.is_some());
        assert!((r.sun.unwrap()[0] - 1.0).abs() < 0.01);
        assert!(r.board.is_some());
        // The rail off: everything off
        b.run(&mut s, 2_000, 2_040, false);
        assert_eq!(b.pins.en, [false; 3]);
        assert!(b.pins.rst_low);
        assert!(!s.mag.ready);
    }

    #[test]
    fn a_trip_is_counted_and_the_parts_restart() {
        let (mut b, mut s) = (Bench::new(), Sensors::new());
        b.run(&mut s, 0, 2_000, true);
        b.pins.trip = [true, false, true];
        let e = b.run(&mut s, 2_000, 2_140, true);
        assert_eq!(e.tripped, [true, false, true]);
        assert_eq!(s.trips, [1, 0, 1]);
        assert_eq!(s.mg370.state, mg370::State::Off);
        b.pins.trip = [false; 3];
        b.run(&mut s, 2_140, 4_500, true);
        assert_eq!(s.mg370.state, mg370::State::Running, "back after the retry");
        assert!(s.mag.ready);
    }

    #[test]
    fn a_failing_primary_hands_over_to_the_backup_then_to_none() {
        let (mut b, mut s) = (Bench::new(), Sensors::new());
        b.mg.id = 0;
        let e = b.run(&mut s, 0, 20_000, true);
        assert_eq!(e.mg370_failed, Some(mg370::Failure::WrongId));
        assert!(e.to_backup);
        assert_eq!(s.gyro, Gyro::Backup);
        assert_eq!(b.pins.en[..2], [false, true]);
        assert_eq!(s.bmi088.state, bmi088::State::Running);
        assert!(b.read(&mut s).rate.is_some());
        b.bg.fail = true;
        for _ in 0..bmi088::MAX_ERRORS {
            assert_eq!(b.read(&mut s).rate, None);
        }
        let e = b.run(&mut s, 20_000, 40_000, true);
        assert!(e.bmi088_failed.is_some());
        assert!(e.no_gyro);
        assert_eq!(s.gyro, Gyro::None);
        assert_eq!(b.read(&mut s).rate, None);
    }

    #[test]
    fn a_missing_sensor_part_is_retried() {
        let (mut b, mut s) = (Bench::new(), Sensors::new());
        b.bus.fail = true;
        let e = b.run(&mut s, 0, 100, true);
        assert!(e.sns_failed);
        b.bus.fail = false;
        b.run(&mut s, 100, 5_000, true);
        assert!(!s.mag.ready, "not before the retry time");
        b.run(&mut s, 5_000, 10_100, true);
        assert!(s.mag.ready);
        assert!(s.sun_ready);
        assert_eq!(b.read(&mut s).field, None, "nothing triggered");
    }
}
