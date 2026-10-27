//! The ADCS sensors on the OBC board (SQSO-OBC-BRD-001 B §9; RID AV-03):
//! their latch-up-protected rails, the drivers' bring-up, the cold-spare
//! gyro and the magnetometer's conversion in the coils-off window.
//!
//! **Rails.** Each sensor group has its own TPS2553-Q1 switch: `GYRO_EN`
//! (M-G370PDT0), `IMU2_EN` (BMI088, cold spare) and `SNS_EN` (RM3100 and
//! the ADS7138 sun-sensor ADC). Each switch's EN node is read back
//! (`*_RB`): low with the enable high is a latch-up trip; the switch retries
//! by itself after 0.18–0.33 s. A trip is counted and the group's driver
//! starts its bring-up again once the rail is back.
//!
//! **Turn-on.** With R_FAULT 220 kΩ a commanded turn-on takes up to 0.55 s
//! (one RC to the EN threshold, SQSO-OBC-BRD-001 E §9.1), so a rail counts
//! as up only [`RAIL_SETTLE_MS`] after its enable, and again only
//! [`RAIL_SETTLE_MS`] after a trip has cleared; until then no driver uses
//! its parts. A readback low inside the first [`RAIL_SETTLE_MS`] after the
//! enable is the turn-on, not a trip.
//!
//! @requirement SRS-F164 A sensor rail is used only 0.6 s after its enable or after a trip clears; a readback low within 0.6 s of the enable is not a trip
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
/// A sensor rail is used this long after its enable, or after a trip on it
/// has cleared, ms (the turn-on takes up to 0.55 s with R_FAULT 220 kΩ)
pub const RAIL_SETTLE_MS: u64 = 600;
/// Latch-up-protected rails: gyro (U5), BMI088 (U6), sun sensors (U7),
/// magnetometer (U35)
pub const RAILS: usize = 4;

/// The pins the sensors use (`hal_gpio.h`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin {
    /// `GPIO_PIN_GYRO_EN`
    GyroEn,
    /// `GPIO_PIN_IMU2_EN`
    Imu2En,
    /// `GPIO_PIN_SNS_EN`: the sun sensors (and their ADC)
    SnsEn,
    /// `GPIO_PIN_MAG_EN`: the magnetometer's own switch (PF14, SQSO-ICX-001
    /// D §6 row 33, RID SMA-B18)
    MagEn,
    /// `GPIO_PIN_GYRO_RB`
    GyroRb,
    /// `GPIO_PIN_IMU2_RB`
    Imu2Rb,
    /// `GPIO_PIN_SNS_RB`
    SnsRb,
    /// `GPIO_PIN_MAG_RB` (PF15)
    MagRb,
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
    /// A rail tripped: gyro, BMI088, sun sensors, magnetometer
    pub tripped: [bool; RAILS],
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
    pub trips: [u32; RAILS],
    /// A rail held off for a power cycle until this time, ms (gyro rail)
    gyro_off_until: Option<u64>,
    /// The sensor rail has been up since, ms (`None`: off or tripped)
    sns_up_since: Option<u64>,
    /// The magnetometer's rail has been up since, ms
    mag_up_since: Option<u64>,
    /// The magnetometer and sun ADC were last tried, ms
    sns_tried_ms: Option<u64>,
    /// A trip seen on each rail at the last pass
    tripped: [bool; RAILS],
    /// Each rail enabled since, ms (`None`: off)
    enabled_since: [Option<u64>; RAILS],
    /// Each rail's last trip cleared at, ms
    cleared_at: [Option<u64>; RAILS],
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
            trips: [0; RAILS],
            gyro_off_until: None,
            sns_up_since: None,
            mag_up_since: None,
            sns_tried_ms: None,
            tripped: [false; RAILS],
            enabled_since: [None; RAILS],
            cleared_at: [None; RAILS],
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
            powered,
        ];
        let pins_en = [Pin::GyroEn, Pin::Imu2En, Pin::SnsEn, Pin::MagEn];
        let pins_rb = [Pin::GyroRb, Pin::Imu2Rb, Pin::SnsRb, Pin::MagRb];
        let mut up = [false; RAILS];
        for i in 0..RAILS {
            let (Some(&on), Some(&en), Some(&rb)) = (rails.get(i), pins_en.get(i), pins_rb.get(i)) else { continue };
            d.pins.set(en, on);
            let high = d.pins.get(rb);
            if let (Some(u), Some(e)) = (up.get_mut(i), ev.tripped.get_mut(i)) {
                (*u, *e) = self.rail(i, now_ms, on, high);
            }
        }
        let [gyro_up, imu2_up, sns_up, mag_up] = up;
        let pins = &mut *d.pins;
        self.mg370.step(now_ms, gyro_up, d.mg370, &mut |low| pins.set(Pin::GyroRstN, !low));
        self.bmi088.step(now_ms, imu2_up, d.bmi_gyro, d.bmi_accel);
        self.gyro_health(now_ms, &mut ev);
        self.sensor_rail(now_ms, sns_up, mag_up, d.i2c, &mut ev);
        ev
    }

    /// Rail `i` at `now_ms`, enabled (`on`) with its readback `high`:
    /// whether it is up and settled, and whether a trip starts now
    fn rail(&mut self, i: usize, now_ms: u64, on: bool, high: bool) -> (bool, bool) {
        let (Some(since), Some(seen), Some(cleared), Some(count)) = (
            self.enabled_since.get_mut(i),
            self.tripped.get_mut(i),
            self.cleared_at.get_mut(i),
            self.trips.get_mut(i),
        ) else {
            return (false, false);
        };
        if !on {
            (*since, *seen, *cleared) = (None, false, None);
            return (false, false);
        }
        let enabled = *since.get_or_insert(now_ms);
        let turning_on = now_ms.saturating_sub(enabled) < RAIL_SETTLE_MS;
        let trip = !high && !turning_on;
        let onset = trip && !*seen;
        if onset {
            *count = count.saturating_add(1);
        }
        if *seen && high {
            *cleared = Some(now_ms);
        }
        *seen = trip;
        let settled = cleared.is_none_or(|t| now_ms.saturating_sub(t) >= RAIL_SETTLE_MS);
        (high && !turning_on && settled, onset)
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

    fn sensor_rail(&mut self, now_ms: u64, sns_up: bool, mag_up: bool, i2c: &mut impl I2cBus, ev: &mut Events) {
        // A rail down: its part restarts, and is tried as soon as the rail
        // is back and settled
        if !mag_up && self.mag_up_since.take().is_some() {
            self.sns_tried_ms = None;
        }
        if !mag_up {
            self.mag = Rm3100::new();
        }
        if !sns_up && self.sns_up_since.take().is_some() {
            self.sns_tried_ms = None;
        }
        if !sns_up {
            self.sun_ready = false;
        }
        let settled = |since: Option<u64>| since.is_some_and(|t| now_ms.saturating_sub(t) >= SNS_SETTLE_MS);
        let mag_since = if mag_up { Some(*self.mag_up_since.get_or_insert(now_ms)) } else { None };
        let sun_since = if sns_up { Some(*self.sns_up_since.get_or_insert(now_ms)) } else { None };
        let want_mag = settled(mag_since) && !self.mag.ready;
        let want_sun = settled(sun_since) && !self.sun_ready;
        let retry = self.sns_tried_ms.is_none_or(|t| now_ms.saturating_sub(t) >= SNS_RETRY_MS);
        if retry && (want_mag || want_sun) {
            self.sns_tried_ms = Some(now_ms);
            if want_mag {
                let _ = self.mag.init(i2c);
            }
            if want_sun {
                self.sun_ready = ads7138::init(i2c);
            }
            ev.sns_failed = (want_mag && !self.mag.ready) || (want_sun && !self.sun_ready);
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
        if now_ms >= window_ms && !fresh && self.mag.ready && !self.mag.pending && self.mag.trigger(i2c).is_ok() {
            self.mag_trigger_ms = Some(now_ms);
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
        en: [bool; RAILS],
        trip: [bool; RAILS],
        rst_low: bool,
        drdy: bool,
        /// The time, ms, each enable rose, and how long the EN node takes
        /// to reach its threshold (the readback low until then)
        now: u64,
        en_at: [u64; RAILS],
        turn_on_ms: u64,
    }

    impl Board {
        fn enable(&mut self, i: usize, high: bool) {
            if high && !self.en[i] {
                self.en_at[i] = self.now;
            }
            self.en[i] = high;
        }
        fn rb(&self, i: usize) -> bool {
            self.en[i] && !self.trip[i] && self.now.saturating_sub(self.en_at[i]) >= self.turn_on_ms
        }
    }

    impl Pins for Board {
        fn set(&mut self, pin: Pin, high: bool) {
            match pin {
                Pin::GyroEn => self.enable(0, high),
                Pin::Imu2En => self.enable(1, high),
                Pin::SnsEn => self.enable(2, high),
                Pin::MagEn => self.enable(3, high),
                Pin::GyroRstN => self.rst_low = !high,
                _ => {}
            }
        }
        fn get(&mut self, pin: Pin) -> bool {
            match pin {
                Pin::GyroRb => self.rb(0),
                Pin::Imu2Rb => self.rb(1),
                Pin::SnsRb => self.rb(2),
                Pin::MagRb => self.rb(3),
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
                d.pins.now = t;
                let e = s.step(t, powered, &mut d);
                for i in 0..RAILS {
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
        assert_eq!(b.pins.en, [true, false, true, true], "the backup gyro is cold");
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
        assert_eq!(b.pins.en, [false; RAILS]);
        assert!(b.pins.rst_low);
        assert!(!s.mag.ready);
    }

    #[test]
    fn a_trip_is_counted_and_the_parts_restart() {
        let (mut b, mut s) = (Bench::new(), Sensors::new());
        b.run(&mut s, 0, 2_000, true);
        b.pins.trip = [true, false, true, false];
        let e = b.run(&mut s, 2_000, 2_140, true);
        assert_eq!(e.tripped, [true, false, true, false]);
        assert_eq!(s.trips, [1, 0, 1, 0]);
        assert_eq!(s.mg370.state, mg370::State::Off);
        assert!(s.mag.ready, "the magnetometer has its own switch (SMA-B18)");
        b.pins.trip = [false; RAILS];
        b.run(&mut s, 2_140, 4_500, true);
        assert_eq!(s.mg370.state, mg370::State::Running, "back after the retry");
        assert!(s.mag.ready);
        // The magnetometer's switch trips: the magnetometer restarts, the
        // sun sensors stay
        b.pins.trip = [false, false, false, true];
        let e = b.run(&mut s, 4_500, 4_640, true);
        assert_eq!(e.tripped, [false, false, false, true]);
        assert_eq!(s.trips, [1, 0, 1, 1]);
        assert!(!s.mag.ready);
        assert!(s.sun_ready);
        b.pins.trip = [false; RAILS];
        b.run(&mut s, 4_640, 20_000, true);
        assert!(s.mag.ready);
    }

    /// The turn-on takes up to 0.55 s (R_FAULT 220 kΩ): the readback low
    /// meanwhile is not a trip, and no part is used before 0.6 s; after a
    /// trip clears the rail waits 0.6 s again
    #[test]
    fn a_rail_is_used_only_after_its_turn_on() {
        let (mut b, mut s) = (Bench::new(), Sensors::new());
        b.pins.turn_on_ms = 550;
        let e = b.run(&mut s, 0, RAIL_SETTLE_MS, true);
        assert_eq!(e, Events::default(), "the turn-on is not a trip");
        assert_eq!(s.trips, [0; RAILS]);
        assert_eq!(s.mg370.state, mg370::State::Off, "the gyro not touched yet");
        assert!(!s.mag.ready && !s.sun_ready);
        assert!(b.bus.mag.writes.is_empty(), "no sensor part configured during the turn-on");
        b.run(&mut s, RAIL_SETTLE_MS, RAIL_SETTLE_MS + 40, true);
        assert!(s.mag.ready && s.sun_ready, "configured once the rail settled");
        assert_ne!(s.mg370.state, mg370::State::Off, "the gyro's bring-up started");
        b.run(&mut s, RAIL_SETTLE_MS + 40, 3_000, true);
        assert_eq!(s.mg370.state, mg370::State::Running);
        // A readback still low after 0.6 s is a trip
        let (mut b2, mut s2) = (Bench::new(), Sensors::new());
        b2.pins.turn_on_ms = 700;
        let e = b2.run(&mut s2, 0, 680, true);
        assert_eq!(e.tripped, [true, false, true, true]);
        assert_eq!(s2.trips, [1, 0, 1, 1]);
        // The trip clears at 700 ms: the rails are used 0.6 s later
        b2.run(&mut s2, 680, 1_280, true);
        assert!(!s2.mag.ready && !s2.sun_ready, "inside the 0.6 s after the trip cleared");
        assert_eq!(s2.mg370.state, mg370::State::Off);
        b2.run(&mut s2, 1_280, 1_340, true);
        assert!(s2.mag.ready && s2.sun_ready);
        // Off and on again: the turn-on wait again
        b.run(&mut s, 3_000, 3_040, false);
        b.run(&mut s, 3_040, 3_040 + RAIL_SETTLE_MS, true);
        assert!(!s.mag.ready);
        assert_eq!(s.trips, [0; RAILS]);
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
        let e = b.run(&mut s, 0, RAIL_SETTLE_MS + 100, true);
        assert!(e.sns_failed);
        b.bus.fail = false;
        b.run(&mut s, RAIL_SETTLE_MS + 100, 5_000, true);
        assert!(!s.mag.ready, "not before the retry time");
        b.run(&mut s, 5_000, RAIL_SETTLE_MS + 10_100, true);
        assert!(s.mag.ready);
        assert!(s.sun_ready);
        assert_eq!(b.read(&mut s).field, None, "nothing triggered");
    }
}
