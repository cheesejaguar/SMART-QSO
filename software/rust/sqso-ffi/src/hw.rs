//! The OBC board's buses and pins for the device drivers of `sqso-fsw`
//! (`bus::SpiDevice`, `bus::I2cBus`, `adcs_sensors::Pins`), on the HAL
//! (`hal_spi.h`, `hal_i2c.h`, `hal_gpio.h`, `hal_adc.h`).
//!
//! The SPI adapter is used by the target's sensor drivers only (the host
//! has the simulated sensors).
//!
//! @requirement SRS-F140 ADCS sensor drivers on the OBC buses
#![cfg_attr(not(target_os = "none"), allow(dead_code))]

use sqso_fsw::adcs_sensors::{Pin, Pins};
use sqso_fsw::bus::{BusError, I2cBus, SpiDevice};

use crate::c::{self, rc};
use crate::pins;

fn ok(code: i32) -> Result<(), BusError> {
    if code == rc::OK { Ok(()) } else { Err(BusError) }
}

/// One `hal_spi` bus (one device: its own chip select)
pub(crate) struct HalSpi(pub(crate) u32);

impl SpiDevice for HalSpi {
    fn select(&mut self) -> Result<(), BusError> {
        // SAFETY: plain value (the HAL)
        ok(unsafe { c::hal_spi_cs_assert(self.0) })
    }

    fn deselect(&mut self) {
        // SAFETY: plain value (the HAL)
        let _ = unsafe { c::hal_spi_cs_deassert(self.0) };
    }

    fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), BusError> {
        let len = tx.len().min(rx.len());
        // SAFETY: both slices hold at least len bytes
        ok(unsafe { c::hal_spi_transfer(self.0, tx.as_ptr(), rx.as_mut_ptr(), len) })
    }

    fn delay_us(&mut self, us: u32) {
        // SAFETY: plain value (the HAL)
        unsafe { c::hal_timer_delay_us(us) };
    }
}

/// One `hal_i2c` bus
pub(crate) struct HalI2c(pub(crate) u32);

impl I2cBus for HalI2c {
    fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), BusError> {
        // SAFETY: data is a live slice
        ok(unsafe { c::hal_i2c_write(self.0, addr, data.as_ptr(), data.len()) })
    }

    fn write_read(&mut self, addr: u8, w: &[u8], r: &mut [u8]) -> Result<(), BusError> {
        // SAFETY: both are live slices
        ok(unsafe { c::hal_i2c_write_read(self.0, addr, w.as_ptr(), w.len(), r.as_mut_ptr(), r.len()) })
    }

    fn read(&mut self, addr: u8, r: &mut [u8]) -> Result<(), BusError> {
        // SAFETY: r is a live slice
        ok(unsafe { c::hal_i2c_read(self.0, addr, r.as_mut_ptr(), r.len()) })
    }
}

/// The ADCS sensors' pins on `hal_gpio`
pub(crate) struct HalPins;

impl HalPins {
    const fn id(pin: Pin) -> u32 {
        match pin {
            Pin::GyroEn => pins::GYRO_EN,
            Pin::Imu2En => pins::IMU2_EN,
            Pin::SnsEn => pins::SNS_EN,
            Pin::GyroRb => pins::GYRO_RB,
            Pin::Imu2Rb => pins::IMU2_RB,
            Pin::SnsRb => pins::SNS_RB,
            Pin::GyroRstN => pins::GYRO_RST_N,
            Pin::GyroDrdy => pins::GYRO_DRDY,
            Pin::Imu2Drdy => pins::IMU2_DRDY,
            Pin::MagDrdy => pins::MAG_DRDY,
        }
    }

    /// Outputs low, inputs with the board's pull-downs
    pub(crate) fn configure() {
        for p in [Pin::GyroEn, Pin::Imu2En, Pin::SnsEn, Pin::GyroRstN] {
            // SAFETY: plain values (the HAL): output, pull-down
            let _ = unsafe { c::hal_gpio_config(HalPins::id(p), 1, 2) };
        }
        for p in [Pin::GyroRb, Pin::Imu2Rb, Pin::SnsRb, Pin::GyroDrdy, Pin::Imu2Drdy, Pin::MagDrdy] {
            // SAFETY: plain values (the HAL): input, no pull (the board's)
            let _ = unsafe { c::hal_gpio_config(HalPins::id(p), 0, 0) };
        }
    }
}

impl Pins for HalPins {
    fn set(&mut self, pin: Pin, high: bool) {
        // SAFETY: plain values (the HAL)
        let _ = unsafe { c::hal_gpio_set(HalPins::id(pin), high) };
    }

    fn get(&mut self, pin: Pin) -> bool {
        // SAFETY: plain value (the HAL)
        unsafe { c::hal_gpio_get(HalPins::id(pin)) }
    }
}

/// A GPIO input or output's level (false if it cannot be read)
pub(crate) fn level(pin: u32) -> bool {
    // SAFETY: plain value (the HAL)
    unsafe { c::hal_gpio_get(pin) }
}

/// Configure a pin: an output (low) or an input
pub(crate) fn configure(pin: u32, output: bool) {
    // SAFETY: plain values (the HAL); outputs with the pull-down
    let _ = unsafe { c::hal_gpio_config(pin, u32::from(output), if output { 2 } else { 0 }) };
}

/// Drive an output
pub(crate) fn set(pin: u32, high: bool) {
    // SAFETY: plain values (the HAL)
    let _ = unsafe { c::hal_gpio_set(pin, high) };
}

/// One ADC channel in its own unit; NaN if it cannot be read
pub(crate) fn adc(channel: u32) -> f64 {
    let mut v = 0.0;
    // SAFETY: v is a live f64
    if unsafe { c::hal_adc_read_voltage(channel, &mut v) } == rc::OK { v } else { f64::NAN }
}

/// One ADC channel's 12-bit code
pub(crate) fn adc_raw(channel: u32) -> Option<u16> {
    let mut v = 0u16;
    // SAFETY: v is a live u16
    (unsafe { c::hal_adc_read_raw(channel, &mut v) } == rc::OK).then_some(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// The adapters on the simulated HAL: the SPI bus reads zeros, the I2C
    /// bus reaches the simulated EPS controller, the pins follow
    #[test]
    fn adapters_on_the_simulated_hal() {
        let _serial = serial();
        let _ = crate::hal_sim::hal_gpio_init();
        let cfg = [0u8; 16];
        // SAFETY: a live buffer standing in for HalSpiConfig_t (not read by the simulation)
        assert_eq!(unsafe { c::hal_spi_init(pins::SPI_MG370, cfg.as_ptr().cast()) }, rc::OK);
        let mut spi = HalSpi(pins::SPI_MG370);
        let mut rx = [0xFFu8; 2];
        assert_eq!(spi.transaction(&[0x80, 0], &mut rx), Ok(()));
        assert_eq!(rx, [0, 0]);
        spi.delay_us(1);
        assert_eq!(HalSpi(9).transfer(&[0], &mut [0]), Err(BusError));
        assert_eq!(crate::hal_sim::hal_i2c_init(pins::I2C_EPS, 100_000), rc::OK);
        let mut i2c = HalI2c(pins::I2C_EPS);
        assert_eq!(i2c.write(0x48, &[0x60, 1, 2]), Ok(()));
        let mut b = [0u8; 2];
        assert_eq!(i2c.write_read(0x48, &[0x60], &mut b), Ok(()));
        assert_eq!(b, [1, 2]);
        assert_eq!(i2c.read(0x48, &mut b), Ok(()));
        assert_eq!(i2c.write(0x7F, &[0]), Err(BusError));
        HalPins::configure();
        let mut p = HalPins;
        p.set(Pin::GyroEn, true);
        assert!(p.get(Pin::GyroEn) && p.get(Pin::GyroRb), "the readback follows the enable");
        p.set(Pin::GyroEn, false);
        assert!(!p.get(Pin::GyroRb));
        for pin in [Pin::Imu2En, Pin::SnsEn, Pin::Imu2Rb, Pin::SnsRb, Pin::GyroRstN, Pin::GyroDrdy, Pin::Imu2Drdy] {
            p.set(pin, false);
        }
        assert!(!p.get(Pin::MagDrdy));
        configure(pins::WDI, true);
        set(pins::WDI, true);
        assert!(level(pins::WDI));
        set(pins::WDI, false);
        assert_eq!(crate::hal_sim::hal_adc_init(12, 2), rc::OK);
        assert!((adc(pins::ADC_PT_NTC) - 1.65).abs() < 1e-12);
        assert_eq!(adc_raw(pins::ADC_PT_NTC), Some(2047));
        assert!(adc(99).is_nan() && adc_raw(99).is_none());
    }
}
