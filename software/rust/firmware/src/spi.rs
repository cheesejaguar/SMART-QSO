//! `hal_spi.h`: bus 0 is SPI2 (the RF transceiver; PB13 SCK / PB14 MISO /
//! PB15 MOSI, chip select PD0), bus 1 is SPI3 (PC10 / PC11 / PC12, chip
//! select PD1). Pins TBR (gpio.rs). The FRAM has its own bus (fram.rs).

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

pub(crate) struct Bus {
    spi: Spi<'static, Blocking, Master>,
    cs: Output<'static>,
    ready: bool,
}

static BUSES: [Shared<Bus>; 2] = [Shared::new(), Shared::new()];

pub(crate) fn install(rf: (Spi<'static, Blocking, Master>, Output<'static>), aux: (Spi<'static, Blocking, Master>, Output<'static>)) {
    let [a, b] = &BUSES;
    a.install(Bus { spi: rf.0, cs: rf.1, ready: false });
    b.install(Bus { spi: aux.0, cs: aux.1, ready: false });
}

fn with_bus(bus: u32, f: impl FnOnce(&mut Bus) -> i32) -> i32 {
    match usize::try_from(bus).ok().and_then(|b| BUSES.get(b)) {
        Some(shared) => shared.with(f).unwrap_or(rc::BUSY),
        None => rc::INVALID,
    }
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
    with_bus(bus, |b| {
        let mut cfg = spi::Config::default();
        cfg.frequency = Hertz(c.clock_hz);
        cfg.mode = mode;
        cfg.bit_order = if c.msb_first { BitOrder::MsbFirst } else { BitOrder::LsbFirst };
        if b.spi.set_config(&cfg).is_err() {
            return rc::INVALID;
        }
        b.cs.set_high();
        b.ready = true;
        rc::OK
    })
}

/// Full-duplex transfer of `len` bytes (either buffer may be NULL)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_transfer(bus: u32, tx_data: *const u8, rx_data: *mut u8, len: usize) -> i32 {
    // SAFETY: the C contract: each is NULL or points to len bytes
    let (tx, rx) = unsafe { (rc::bytes(tx_data, len), rc::bytes_mut(rx_data, len)) };
    with_bus(bus, |b| {
        if !b.ready {
            return rc::ERROR;
        }
        match (tx, rx) {
            (Some(tx), Some(rx)) => result(b.spi.blocking_transfer(rx, tx)),
            (Some(tx), None) => result(b.spi.blocking_write(tx)),
            (None, Some(rx)) => result(b.spi.blocking_read(rx)),
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

/// Select the device (chip select low)
#[unsafe(no_mangle)]
pub extern "C" fn hal_spi_cs_assert(bus: u32) -> i32 {
    with_bus(bus, |b| {
        b.cs.set_low();
        rc::OK
    })
}

/// Deselect the device
#[unsafe(no_mangle)]
pub extern "C" fn hal_spi_cs_deassert(bus: u32) -> i32 {
    with_bus(bus, |b| {
        b.cs.set_high();
        rc::OK
    })
}
