//! `hal_spi.h` (SQSO-ICX-001 B §4.3): a logical bus is one device.
//!
//! - bus 0: SPI2 (PB13 SCK / PB14 MISO / PB15 MOSI), chip select PD0: the
//!   BMI088's gyro;
//! - bus 1: SPI3 (PC10 / PC11 / PC12), chip select PD1: the Epson
//!   M-G370PDT0 (mode 3, at most 1 MHz in burst mode);
//! - bus 2: SPI2 again, chip select PD4: the BMI088's accelerometer.
//!
//! Buses 0 and 2 share SPI2: each keeps its own configuration, applied when
//! its chip select is asserted. The FRAM has its own bus (fram.rs).

use embassy_stm32::gpio::Output;
use embassy_stm32::mode::Blocking;
use embassy_stm32::spi::mode::Master;
use embassy_stm32::spi::{self, BitOrder, Spi};
use embassy_stm32::time::Hertz;

use crate::cell::Shared;
use crate::rc;

/// `HalSpiConfig_t`
#[repr(C)]
pub struct HalSpiConfig {
    clock_hz: u32,
    mode: u32,
    bits_per_word: u8,
    msb_first: bool,
}

/// One SPI peripheral with the chip selects of its devices
pub(crate) struct Bus<const N: usize> {
    spi: Spi<'static, Blocking, Master>,
    cs: [Output<'static>; N],
    config: [Option<spi::Config>; N],
}

static SPI2: Shared<Bus<2>> = Shared::new();
static SPI3: Shared<Bus<1>> = Shared::new();

pub(crate) fn install(
    spi2: Spi<'static, Blocking, Master>,
    spi2_cs: [Output<'static>; 2],
    spi3: Spi<'static, Blocking, Master>,
    spi3_cs: Output<'static>,
) {
    SPI2.install(Bus { spi: spi2, cs: spi2_cs, config: [None; 2] });
    SPI3.install(Bus { spi: spi3, cs: [spi3_cs], config: [None] });
}

/// Run `f` on logical bus `bus`: its peripheral, chip select and
/// configuration
fn with_bus(
    bus: u32,
    f: impl FnOnce(&mut Spi<'static, Blocking, Master>, &mut Output<'static>, &mut Option<spi::Config>) -> i32,
) -> i32 {
    let found = match bus {
        0 | 2 => {
            let i = usize::from(bus == 2);
            SPI2.with(|b| match (b.cs.get_mut(i), b.config.get_mut(i)) {
                (Some(cs), Some(cfg)) => f(&mut b.spi, cs, cfg),
                _ => rc::INVALID,
            })
        }
        1 => SPI3.with(|b| match (b.cs.get_mut(0), b.config.get_mut(0)) {
            (Some(cs), Some(cfg)) => f(&mut b.spi, cs, cfg),
            _ => rc::INVALID,
        }),
        _ => return rc::INVALID,
    };
    found.unwrap_or(rc::BUSY)
}

fn result(r: Result<(), spi::Error>) -> i32 {
    if r.is_ok() { rc::OK } else { rc::IO }
}

/// Set clock, mode and bit order (8-bit words only)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_init(bus: u32, config: *const HalSpiConfig) -> i32 {
    // SAFETY: the C contract: NULL or a HalSpiConfig_t
    let Some(c) = (unsafe { config.as_ref() }) else { return rc::NULL_PTR };
    if c.bits_per_word != 8 || c.clock_hz == 0 {
        return rc::INVALID;
    }
    let mode = match c.mode {
        0 => spi::MODE_0,
        1 => spi::MODE_1,
        2 => spi::MODE_2,
        3 => spi::MODE_3,
        _ => return rc::INVALID,
    };
    let mut new = spi::Config::default();
    new.frequency = Hertz(c.clock_hz);
    new.mode = mode;
    new.bit_order = if c.msb_first { BitOrder::MsbFirst } else { BitOrder::LsbFirst };
    with_bus(bus, |s, cs, cfg| {
        if s.set_config(&new).is_err() {
            return rc::INVALID;
        }
        cs.set_high();
        *cfg = Some(new);
        rc::OK
    })
}

/// Full-duplex transfer of `len` bytes (either buffer may be NULL)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_transfer(bus: u32, tx_data: *const u8, rx_data: *mut u8, len: usize) -> i32 {
    // SAFETY: the C contract: each is NULL or points to len bytes
    let (tx, rx) = unsafe { (rc::bytes(tx_data, len), rc::bytes_mut(rx_data, len)) };
    with_bus(bus, |s, _, cfg| {
        if cfg.is_none() {
            return rc::ERROR;
        }
        match (tx, rx) {
            (Some(tx), Some(rx)) => result(s.blocking_transfer(rx, tx)),
            (Some(tx), None) => result(s.blocking_write(tx)),
            (None, Some(rx)) => result(s.blocking_read(rx)),
            (None, None) => rc::NULL_PTR,
        }
    })
}

/// Transmit only
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_write(bus: u32, data: *const u8, len: usize) -> i32 {
    // SAFETY: forwarded contract; no receive buffer
    unsafe { hal_spi_transfer(bus, data, core::ptr::null_mut(), len) }
}

/// Receive only (clocks out zeros)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_read(bus: u32, data: *mut u8, len: usize) -> i32 {
    // SAFETY: forwarded contract
    unsafe { hal_spi_transfer(bus, core::ptr::null(), data, len) }
}

/// Select the device: its configuration on the peripheral, chip select low
#[unsafe(no_mangle)]
pub extern "C" fn hal_spi_cs_assert(bus: u32) -> i32 {
    with_bus(bus, |s, cs, cfg| {
        let Some(c) = cfg else { return rc::ERROR };
        if s.set_config(c).is_err() {
            return rc::INVALID;
        }
        cs.set_low();
        rc::OK
    })
}

/// Deselect the device
#[unsafe(no_mangle)]
pub extern "C" fn hal_spi_cs_deassert(bus: u32) -> i32 {
    with_bus(bus, |_, cs, _| {
        cs.set_high();
        rc::OK
    })
}
