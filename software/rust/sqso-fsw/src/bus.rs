//! The buses the device drivers use, as traits, so each driver is tested on
//! the host against a register model of its part (`mock`) and runs on the
//! target over the HAL (`hal_spi.h`, `hal_i2c.h`, `sqso-ffi`).
//!
//! @requirement SRS-F140 ADCS sensor drivers on the OBC buses

/// A bus transfer failed (no acknowledge, a timeout, a HAL refusal)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BusError;

/// One SPI device: its chip select and the shared clock and data lines
pub trait SpiDevice {
    /// Chip select low
    fn select(&mut self) -> Result<(), BusError>;
    /// Chip select high
    fn deselect(&mut self);
    /// Full-duplex transfer of `tx.len()` bytes (`rx` the same length)
    fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), BusError>;
    /// Busy-wait a few microseconds (the devices' stall times)
    fn delay_us(&mut self, us: u32);

    /// Select, transfer, deselect
    fn transaction(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), BusError> {
        self.select()?;
        let r = self.transfer(tx, rx);
        self.deselect();
        r
    }
}

/// An I2C bus
pub trait I2cBus {
    /// Write `data` to the device at `addr` (7-bit)
    fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), BusError>;
    /// Write `w`, then read `r` with a repeated start
    fn write_read(&mut self, addr: u8, w: &[u8], r: &mut [u8]) -> Result<(), BusError>;
    /// Read `r` (no register address first)
    fn read(&mut self, addr: u8, r: &mut [u8]) -> Result<(), BusError>;
}

/// Big-endian 16-bit word at `at`
pub(crate) fn be16(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes([*b.get(at)?, *b.get(at + 1)?]))
}

/// Register models for the drivers' tests
#[cfg(test)]
pub(crate) mod mock {
    extern crate std;
    use super::{BusError, I2cBus};
    use std::vec::Vec;

    /// A register-file I2C device at one address: a write's first byte is
    /// the register pointer, the rest are stored from it on; a read returns
    /// from the pointer on. `read_hook` overrides reads of a register
    pub(crate) struct I2cRegs {
        pub(crate) addr: u8,
        pub(crate) regs: [u8; 256],
        pub(crate) pointer: u8,
        pub(crate) fail: bool,
        pub(crate) writes: Vec<Vec<u8>>,
    }

    impl I2cRegs {
        pub(crate) fn new(addr: u8) -> I2cRegs {
            I2cRegs { addr, regs: [0; 256], pointer: 0, fail: false, writes: Vec::new() }
        }
    }

    impl I2cBus for I2cRegs {
        fn write(&mut self, addr: u8, data: &[u8]) -> Result<(), BusError> {
            if self.fail || addr != self.addr {
                return Err(BusError);
            }
            self.writes.push(data.to_vec());
            if let [reg, values @ ..] = data {
                self.pointer = *reg;
                for (i, v) in values.iter().enumerate() {
                    self.regs[usize::from(self.pointer) + i] = *v;
                }
            }
            Ok(())
        }
        fn write_read(&mut self, addr: u8, w: &[u8], r: &mut [u8]) -> Result<(), BusError> {
            if self.fail || addr != self.addr {
                return Err(BusError);
            }
            if let [reg, ..] = w {
                self.pointer = *reg;
            }
            self.read(addr, r)
        }
        fn read(&mut self, addr: u8, r: &mut [u8]) -> Result<(), BusError> {
            if self.fail || addr != self.addr {
                return Err(BusError);
            }
            let p = usize::from(self.pointer);
            r.copy_from_slice(&self.regs[p..p + r.len()]);
            Ok(())
        }
    }
}
