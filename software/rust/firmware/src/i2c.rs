//! `hal_i2c.h` (SQSO-ICX-001 B §4.3): bus 0 is I2C2 (PF1 SCL / PF0 SDA: the
//! EPS controller 0x48, the INA228s 0x40 and 0x44, the AntS 0x31), bus 1 is
//! I2C4 (PD12 SCL / PD13 SDA, through the TCA9517: the ADS7138 0x10 and the
//! RM3100 0x20), bus 2 is I2C1 (PG14 SCL / PG13 SDA: the RV-3028-C7 0x52,
//! alone on its bus).

use embassy_embedded_hal::SetConfig;
use embassy_stm32::i2c::{self, I2c, Master};
use embassy_stm32::mode::Blocking;
use embassy_stm32::time::Hertz;

use crate::cell::Shared;
use crate::rc;

pub(crate) struct Bus {
    i2c: I2c<'static, Blocking, Master>,
    ready: bool,
}

static BUSES: [Shared<Bus>; 3] = [Shared::new(), Shared::new(), Shared::new()];

pub(crate) fn install(
    eps: I2c<'static, Blocking, Master>,
    adcs: I2c<'static, Blocking, Master>,
    rtc: I2c<'static, Blocking, Master>,
) {
    let [a, b, c] = &BUSES;
    a.install(Bus { i2c: eps, ready: false });
    b.install(Bus { i2c: adcs, ready: false });
    c.install(Bus { i2c: rtc, ready: false });
}

fn with_bus(bus: u32, f: impl FnOnce(&mut Bus) -> i32) -> i32 {
    match usize::try_from(bus).ok().and_then(|b| BUSES.get(b)) {
        Some(shared) => shared.with(f).unwrap_or(rc::BUSY),
        None => rc::INVALID,
    }
}

fn result(r: Result<(), i2c::Error>) -> i32 {
    match r {
        Ok(()) => rc::OK,
        Err(i2c::Error::Timeout) => rc::TIMEOUT,
        Err(_) => rc::IO,
    }
}

/// Set the clock (100 kHz, 400 kHz or 1 MHz)
#[unsafe(no_mangle)]
pub extern "C" fn hal_i2c_init(bus: u32, speed: u32) -> i32 {
    if !matches!(speed, 100_000 | 400_000 | 1_000_000) {
        return rc::INVALID;
    }
    with_bus(bus, |b| {
        if b.i2c.set_config(&Hertz(speed)).is_err() {
            return rc::INVALID;
        }
        b.ready = true;
        rc::OK
    })
}

/// Write `len` bytes to the device at `addr` (7-bit)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_i2c_write(bus: u32, addr: u8, data: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len bytes
    let data = unsafe { rc::bytes(data, len) };
    with_bus(bus, |b| {
        if !b.ready {
            return rc::ERROR;
        }
        let Some(data) = data.filter(|d| !d.is_empty()) else { return rc::NULL_PTR };
        result(b.i2c.blocking_write(addr, data))
    })
}

/// Read `len` bytes from the device at `addr`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_i2c_read(bus: u32, addr: u8, data: *mut u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len writable bytes
    let data = unsafe { rc::bytes_mut(data, len) };
    with_bus(bus, |b| {
        if !b.ready {
            return rc::ERROR;
        }
        let Some(data) = data.filter(|d| !d.is_empty()) else { return rc::NULL_PTR };
        result(b.i2c.blocking_read(addr, data))
    })
}

/// Write, then read with a repeated start
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_i2c_write_read(
    bus: u32,
    addr: u8,
    write_data: *const u8,
    write_len: usize,
    read_data: *mut u8,
    read_len: usize,
) -> i32 {
    // SAFETY: the C contract: write_data points to write_len bytes and
    // read_data to read_len writable bytes
    let (write, read) = unsafe { (rc::bytes(write_data, write_len), rc::bytes_mut(read_data, read_len)) };
    with_bus(bus, |b| {
        if !b.ready {
            return rc::ERROR;
        }
        let (Some(write), Some(read)) = (write.filter(|w| !w.is_empty()), read) else { return rc::NULL_PTR };
        result(b.i2c.blocking_write_read(addr, write, read))
    })
}

/// Whether a device acknowledges its address (a zero-length write)
#[unsafe(no_mangle)]
pub extern "C" fn hal_i2c_device_present(bus: u32, addr: u8) -> bool {
    with_bus(bus, |b| if b.ready { result(b.i2c.blocking_write(addr, &[])) } else { rc::ERROR }) == rc::OK
}

/// Bus recovery: the I2C peripheral is re-initialized by its driver after
/// an error, so a stuck bus is retried on the next transfer
#[unsafe(no_mangle)]
pub extern "C" fn hal_i2c_recover(bus: u32) -> i32 {
    with_bus(bus, |b| if b.ready { rc::OK } else { rc::ERROR })
}
