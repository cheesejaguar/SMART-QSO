//! `hal_uart.h`: the Jetson link (USART1, PA9 TX / PA10 RX), the debug
//! console (USART3, PD8 TX / PD9 RX) and the GNSS receiver (LPUART1, PB11 TX /
//! PG8 RX), interrupt-driven with receive buffers
//! so bytes arriving between main-loop passes are kept. Pins TBR (gpio.rs).

use embassy_stm32::usart::{self, BufferedUart, DataBits, Parity, StopBits};
use embassy_time::{Duration, Instant};
use embedded_io::{Read, ReadReady, Write};

use crate::cell::Shared;
use crate::rc;

/// `HalUartConfig_t`
#[repr(C)]
pub struct HalUartConfig {
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: u32,
}

const PARITY_ODD: u32 = 1;
const PARITY_EVEN: u32 = 2;

/// Bytes held between reads; matches the simulation's receive FIFO
const RX_HOLD: usize = 512;

pub(crate) struct Port {
    uart: BufferedUart<'static>,
    ready: bool,
    /// Bytes taken from the driver but not yet read (for hal_uart_available)
    held: [u8; RX_HOLD],
    held_len: usize,
}

impl Port {
    pub(crate) fn new(uart: BufferedUart<'static>) -> Self {
        Port { uart, ready: false, held: [0; RX_HOLD], held_len: 0 }
    }

    /// Move what the driver has received into `held`
    fn drain(&mut self) {
        while self.held_len < RX_HOLD && self.uart.read_ready().unwrap_or(false) {
            let Some(room) = self.held.get_mut(self.held_len..) else { break };
            match self.uart.read(room) {
                Ok(0) | Err(_) => break,
                Ok(n) => self.held_len += n,
            }
        }
    }

    /// Take up to `out.len()` held bytes
    fn take(&mut self, out: &mut [u8]) -> usize {
        let n = out.len().min(self.held_len);
        if let (Some(dst), Some(src)) = (out.get_mut(..n), self.held.get(..n)) {
            dst.copy_from_slice(src);
        }
        self.held.copy_within(n..self.held_len, 0);
        self.held_len -= n;
        n
    }
}

/// `HalUartPort_t` 0 (Jetson), 1 (debug) and 2 (GNSS)
static PORTS: [Shared<Port>; 3] = [Shared::new(), Shared::new(), Shared::new()];

/// The Jetson port opens with `hal_uart_init`. The debug port (the console)
/// is ready at once, at the 115200 8N1 it was built with: the lines printed
/// before `gse_init` opens it (the RF-silence state at every boot, an early
/// init failure) would otherwise be lost
pub(crate) fn install(jetson: BufferedUart<'static>, debug: BufferedUart<'static>, gnss: BufferedUart<'static>) {
    let [j, d, g] = &PORTS;
    j.install(Port::new(jetson));
    g.install(Port::new(gnss));
    let mut console = Port::new(debug);
    console.ready = true;
    d.install(console);
}

fn with_port<R>(port: u32, missing: R, f: impl FnOnce(&mut Port) -> R) -> R {
    match usize::try_from(port).ok().and_then(|p| PORTS.get(p)) {
        Some(shared) => shared.with(f).unwrap_or(missing),
        None => missing,
    }
}

fn config(c: &HalUartConfig) -> usart::Config {
    let mut cfg = usart::Config::default();
    cfg.baudrate = c.baud_rate;
    let parity = match c.parity {
        PARITY_ODD => Parity::ParityOdd,
        PARITY_EVEN => Parity::ParityEven,
        _ => Parity::ParityNone,
    };
    cfg.parity = parity;
    // The STM32 word length includes the parity bit
    cfg.data_bits = match (c.data_bits, parity == Parity::ParityNone) {
        (7, true) => DataBits::DataBits7,
        (7, false) | (_, true) => DataBits::DataBits8,
        (_, false) => DataBits::DataBits9,
    };
    cfg.stop_bits = if c.stop_bits == 2 { StopBits::STOP2 } else { StopBits::STOP1 };
    cfg
}

/// Open a port with `config` (NULL keeps 115200 8N1)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_uart_init(port: u32, config_ptr: *const HalUartConfig) -> i32 {
    // SAFETY: the C contract: NULL or a HalUartConfig_t
    let cfg = unsafe { config_ptr.as_ref() }.map(config);
    let result = match usize::try_from(port).ok().and_then(|p| PORTS.get(p)) {
        Some(shared) => shared.with(|p| {
            if let Some(cfg) = &cfg
                && p.uart.set_config(cfg).is_err()
            {
                return rc::INVALID;
            }
            p.ready = true;
            rc::OK
        }),
        None => return rc::INVALID,
    };
    result.unwrap_or(rc::BUSY)
}

/// Transmit (blocks until the bytes are in the transmit buffer)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_uart_write(port: u32, data: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len bytes
    let data = unsafe { rc::bytes(data, len) };
    with_port(port, rc::ERROR, |p| {
        if !p.ready {
            return rc::ERROR;
        }
        let Some(data) = data else { return rc::NULL_PTR };
        match p.uart.write_all(data) {
            Ok(()) => rc::OK,
            Err(_) => rc::IO,
        }
    })
}

/// Read what has arrived, waiting up to `timeout_ms` for the first byte
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_uart_read(
    port: u32,
    data: *mut u8,
    len: usize,
    timeout_ms: u32,
    bytes_read: *mut usize,
) -> i32 {
    // SAFETY: the C contract: data points to len writable bytes, bytes_read to a size_t
    let (data, out) = unsafe { (rc::bytes_mut(data, len), rc::out(bytes_read)) };
    let (Some(data), Some(out)) = (data, out) else { return rc::NULL_PTR };
    let deadline = Instant::now() + Duration::from_millis(u64::from(timeout_ms));
    loop {
        let n = with_port(port, None, |p| {
            if !p.ready {
                return None;
            }
            p.drain();
            Some(p.take(data))
        });
        match n {
            None => return rc::ERROR,
            Some(0) if Instant::now() < deadline => {}
            Some(n) => {
                *out = n;
                return if n > 0 { rc::OK } else { rc::TIMEOUT };
            }
        }
    }
}

/// Read one line (up to and without the '\n'; a '\r' before it is dropped),
/// NUL-terminated, waiting up to `timeout_ms` in all
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_uart_read_line(
    port: u32,
    buffer: *mut core::ffi::c_char,
    buffer_len: usize,
    timeout_ms: u32,
    line_len: *mut usize,
) -> i32 {
    // SAFETY: the C contract: buffer points to buffer_len writable bytes, line_len to a size_t
    let (buf, out) = unsafe { (rc::bytes_mut(buffer.cast(), buffer_len), rc::out(line_len)) };
    let (Some(buf), Some(out)) = (buf, out) else { return rc::NULL_PTR };
    *out = 0;
    let Some(cap) = buf.len().checked_sub(1) else { return rc::INVALID };
    let deadline = Instant::now() + Duration::from_millis(u64::from(timeout_ms));
    let mut n = 0usize;
    loop {
        let mut byte = [0u8];
        let got = with_port(port, None, |p| {
            if !p.ready {
                return None;
            }
            p.drain();
            Some(p.take(&mut byte))
        });
        match got {
            None => return rc::ERROR,
            Some(1) => match byte {
                [b'\n'] => break,
                [b'\r'] => {}
                [b] => {
                    if n >= cap {
                        return rc::ERROR;
                    }
                    if let Some(slot) = buf.get_mut(n) {
                        *slot = b;
                    }
                    n += 1;
                }
            },
            Some(_) if Instant::now() >= deadline => return rc::TIMEOUT,
            Some(_) => {}
        }
    }
    if let Some(nul) = buf.get_mut(n) {
        *nul = 0;
    }
    *out = n;
    rc::OK
}

/// Bytes received and not yet read
#[unsafe(no_mangle)]
pub extern "C" fn hal_uart_available(port: u32) -> usize {
    with_port(port, 0, |p| {
        p.drain();
        p.held_len
    })
}

/// Discard everything received
#[unsafe(no_mangle)]
pub extern "C" fn hal_uart_flush(port: u32) -> i32 {
    if usize::try_from(port).ok().and_then(|p| PORTS.get(p)).is_none() {
        return rc::INVALID;
    }
    with_port(port, rc::BUSY, |p| {
        p.drain();
        p.held_len = 0;
        rc::OK
    })
}
