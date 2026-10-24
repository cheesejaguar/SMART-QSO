//! `uart_comm.h`: the OBC–Jetson serial line (`sqso_fsw::uart_comm`).
//!
//! On the host the line is a serial device (or a pseudo-terminal in the
//! tests) opened with termios: 8N1, raw, no flow control. The host build
//! takes the device, baud rate and telemetry interval from `UART_DEVICE`,
//! `UART_BAUDRATE` and `TELEMETRY_INTERVAL_MS`. On the target it is the
//! HAL's Jetson port (`hal_uart.h`), polled without blocking.
//!
//! @requirement SRS-F030 Send health telemetry to Jetson via UART
//! @requirement SRS-F031 Receive beacon text from Jetson via UART
//! @requirement SYS-520 Detect AI payload failure or timeout (UART health check)
//! @requirement SRS-I001 UART at 115200 baud, 8N1 (the host build may override device and baud)

use core::ffi::{c_char, c_int};

use sqso_fsw::flight_log::LineWriter;
use sqso_fsw::uart_comm::{self as settings, DEVICE_MAX_LEN};

use crate::c::{self, fault_type, rc, severity};
use crate::cell::Global;
use crate::ptr;

/// `UartConfig_t`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct UartConfig {
    device: [u8; DEVICE_MAX_LEN],
    baudrate: c_int,
    timeout_ms: u32,
}

impl UartConfig {
    const DEFAULT: UartConfig = {
        let mut device = [0u8; DEVICE_MAX_LEN];
        let mut i = 0;
        while i < settings::DEVICE_DEFAULT.len() {
            #[allow(clippy::indexing_slicing)] // i < len < DEVICE_MAX_LEN, at compile time
            {
                device[i] = settings::DEVICE_DEFAULT[i];
            }
            i += 1;
        }
        UartConfig { device, baudrate: settings::BAUDRATE_DEFAULT, timeout_ms: settings::TIMEOUT_DEFAULT_MS }
    };

    /// The device path up to its NUL (the whole field if it has none)
    fn device(&self) -> &[u8] {
        sqso_fsw::safe_string::until_nul(&self.device)
    }

    /// The device path with a NUL, for `open()`
    #[cfg(not(target_os = "none"))]
    fn device_c(&self) -> [u8; DEVICE_MAX_LEN + 1] {
        let mut path = [0u8; DEVICE_MAX_LEN + 1];
        for (dst, src) in path.iter_mut().zip(self.device()) {
            *dst = *src;
        }
        path
    }
}

/// `HalUartConfig_t` (hal_uart.h)
#[repr(C)]
pub struct HalUartConfig {
    baud_rate: u32,
    data_bits: u8,
    stop_bits: u8,
    parity: u32,
}

struct Uart {
    /// Open descriptor (host), or -1
    fd: c_int,
    config: UartConfig,
    telemetry_interval_ms: u32,
    initialized: bool,
}

static UART: Global<Uart> = Global::new(Uart {
    fd: -1,
    config: UartConfig::DEFAULT,
    telemetry_interval_ms: settings::TELEMETRY_INTERVAL_DEFAULT_MS,
    initialized: false,
});

/// A message for stderr (host) or nothing (target)
macro_rules! report {
    ($write:expr) => {{
        #[cfg(not(target_os = "none"))]
        crate::flight_log::stderr_line($write);
        #[cfg(target_os = "none")]
        let _ = $write;
    }};
}

/// The host's serial line, through libc
#[cfg(not(target_os = "none"))]
mod port {
    use core::ffi::{CStr, c_int};

    use sqso_fsw::flight_log::LineWriter;

    use super::UartConfig;

    fn errno() -> c_int {
        // SAFETY: the thread's errno location
        #[cfg(any(target_os = "linux", target_os = "android"))]
        unsafe {
            *libc::__errno_location()
        }
        // SAFETY: the thread's errno location
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        unsafe {
            *libc::__error()
        }
    }

    /// `strerror(errno)`, appended to a line
    pub(super) fn push_error(out: &mut LineWriter<'_>) {
        // SAFETY: strerror returns a C string (static or thread-local)
        let text = unsafe { libc::strerror(errno()) };
        if !text.is_null() {
            // SAFETY: non-NULL strerror result is NUL-terminated
            out.push(unsafe { CStr::from_ptr(text) }.to_bytes());
        }
    }

    fn speed(baud: c_int) -> libc::speed_t {
        match super::settings::line_baud(baud) {
            9600 => libc::B9600,
            19_200 => libc::B19200,
            38_400 => libc::B38400,
            57_600 => libc::B57600,
            230_400 => libc::B230400,
            _ => libc::B115200,
        }
    }

    /// 8N1 framing without flow control, raw input and output
    fn set_raw_8n1(tty: &mut libc::termios) {
        tty.c_cflag = (tty.c_cflag & !libc::CSIZE) | libc::CS8;
        tty.c_cflag &= !(libc::PARENB | libc::PARODD);
        tty.c_cflag &= !libc::CSTOPB;
        tty.c_cflag &= !libc::CRTSCTS;
        tty.c_cflag |= libc::CREAD | libc::CLOCAL;
        tty.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ECHOE | libc::ISIG);
        tty.c_oflag &= !libc::OPOST;
    }

    /// Open and configure the line; the descriptor
    pub(super) fn open(config: &UartConfig) -> Option<c_int> {
        let path = config.device_c();
        // SAFETY: path is NUL-terminated
        let fd = unsafe { libc::open(path.as_ptr().cast(), libc::O_RDWR | libc::O_NOCTTY | libc::O_SYNC) };
        if fd < 0 {
            report!(|out: &mut LineWriter<'_>| {
                out.push(b"[UART] Failed to open ");
                out.push(config.device());
                out.push(b": ");
                push_error(out);
            });
            return None;
        }
        // SAFETY: termios is plain data; tcgetattr fills it
        let mut tty: libc::termios = unsafe { core::mem::zeroed() };
        // SAFETY: fd is open; tty is a valid termios
        if unsafe { libc::tcgetattr(fd, &mut tty) } != 0 {
            report!(|out: &mut LineWriter<'_>| {
                out.push(b"[UART] Failed to get attributes: ");
                push_error(out);
            });
            close(fd);
            return None;
        }
        let speed = speed(config.baudrate);
        // SAFETY: tty is a valid termios
        if unsafe { libc::cfsetospeed(&mut tty, speed) } != 0 {
            report!(|out: &mut LineWriter<'_>| out.push(b"[UART] Failed to set output speed"));
        }
        // SAFETY: as above
        if unsafe { libc::cfsetispeed(&mut tty, speed) } != 0 {
            report!(|out: &mut LineWriter<'_>| out.push(b"[UART] Failed to set input speed"));
        }
        set_raw_8n1(&mut tty);
        // SAFETY: fd is open; tty is a valid termios
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &tty) } != 0 {
            report!(|out: &mut LineWriter<'_>| {
                out.push(b"[UART] Failed to set attributes: ");
                push_error(out);
            });
            close(fd);
            return None;
        }
        Some(fd)
    }

    pub(super) fn close(fd: c_int) {
        // SAFETY: fd is a descriptor this module opened, closed once
        let _ = unsafe { libc::close(fd) };
    }

    /// Write all of `data`
    pub(super) fn write(fd: c_int, data: &[u8]) -> bool {
        // SAFETY: data is a live slice
        let n = unsafe { libc::write(fd, data.as_ptr().cast(), data.len()) };
        usize::try_from(n).is_ok_and(|n| n == data.len())
    }

    /// Wait until the output is sent
    pub(super) fn drain(fd: c_int) {
        // SAFETY: plain descriptor
        if unsafe { libc::tcdrain(fd) } != 0 {
            report!(|out: &mut LineWriter<'_>| {
                out.push(b"[UART] tcdrain failed: ");
                push_error(out);
            });
        }
    }

    /// Read what is there (blocking as the line is configured); the count
    pub(super) fn read(fd: c_int, buf: &mut [u8]) -> Option<usize> {
        // SAFETY: buf is a live, writable slice
        let n = unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) };
        usize::try_from(n).ok()
    }

    /// Whether the descriptor is still a working terminal
    pub(super) fn healthy(fd: c_int) -> bool {
        // SAFETY: termios is plain data
        let mut tty: libc::termios = unsafe { core::mem::zeroed() };
        // SAFETY: tty is a valid termios
        unsafe { libc::tcgetattr(fd, &mut tty) == 0 }
    }

    /// An environment variable (an empty value counts as set)
    pub(super) fn env(name: &CStr) -> Option<&'static [u8]> {
        // SAFETY: name is NUL-terminated; the environment is not modified
        // while the value is used (uart_init reads it once)
        let value = unsafe { libc::getenv(name.as_ptr()) };
        if value.is_null() {
            return None;
        }
        // SAFETY: non-NULL getenv result is NUL-terminated
        Some(unsafe { CStr::from_ptr(value) }.to_bytes())
    }
}

/// The target's Jetson port, through the HAL
#[cfg(target_os = "none")]
mod port {
    use super::UartConfig;
    use crate::c::{self, rc};

    /// `UART_PORT_JETSON`
    const JETSON: u32 = 0;
    /// `SMART_QSO_ERROR_TIMEOUT`
    const TIMEOUT: i32 = -4;

    pub(super) fn open(config: &UartConfig) -> Option<core::ffi::c_int> {
        let hal = super::HalUartConfig {
            baud_rate: u32::try_from(super::settings::line_baud(config.baudrate)).unwrap_or(115_200),
            data_bits: 8,
            stop_bits: 1,
            parity: 0,
        };
        // SAFETY: hal is a live HalUartConfig_t
        (unsafe { c::hal_uart_init(JETSON, (&raw const hal).cast()) } == rc::OK).then_some(-1)
    }

    pub(super) fn close(_fd: core::ffi::c_int) {}

    pub(super) fn write(_fd: core::ffi::c_int, data: &[u8]) -> bool {
        // SAFETY: data is a live slice
        unsafe { c::hal_uart_write(JETSON, data.as_ptr(), data.len()) == rc::OK }
    }

    pub(super) fn drain(_fd: core::ffi::c_int) {}

    /// Read what has arrived, without waiting
    pub(super) fn read(_fd: core::ffi::c_int, buf: &mut [u8]) -> Option<usize> {
        let mut got = 0usize;
        // SAFETY: buf is a live, writable slice; got a live usize
        match unsafe { c::hal_uart_read(JETSON, buf.as_mut_ptr(), buf.len(), 0, &mut got) } {
            rc::OK => Some(got.min(buf.len())),
            TIMEOUT => Some(0),
            _ => None,
        }
    }

    pub(super) fn healthy(_fd: core::ffi::c_int) -> bool {
        true
    }

    /// No error text on the target
    pub(super) fn push_error(_out: &mut sqso_fsw::flight_log::LineWriter<'_>) {}
}

impl Uart {
    fn is_open(&self) -> bool {
        // The target's port has no descriptor
        self.initialized && (cfg!(target_os = "none") || self.fd >= 0)
    }

    fn close(&mut self) {
        if self.fd >= 0 {
            port::close(self.fd);
            self.fd = -1;
        }
        self.initialized = false;
    }

    /// Close if open, then open with `config`
    fn open(&mut self, config: UartConfig) -> i32 {
        if self.initialized {
            self.close();
        }
        self.config = config;
        let Some(fd) = port::open(&self.config) else { return rc::IO };
        self.fd = fd;
        self.initialized = true;
        let baud = self.config.baudrate;
        let device = self.config.device();
        report!(|out: &mut LineWriter<'_>| {
            out.push(b"[UART] Initialized ");
            out.push(device);
            let _ = core::fmt::write(out, format_args!(" at {baud} baud"));
        });
        rc::OK
    }

    /// Apply the environment overrides (host)
    #[cfg(not(target_os = "none"))]
    fn configure_from_env(&mut self) {
        if let Some(device) = port::env(c"UART_DEVICE").filter(|d| settings::device_fits(d)) {
            self.config.device = [0; DEVICE_MAX_LEN];
            let _ = sqso_fsw::safe_string::copy(&mut self.config.device, device, DEVICE_MAX_LEN - 1);
        }
        if let Some(text) = port::env(c"UART_BAUDRATE") {
            match settings::parse_baud(text) {
                Some(baud) => self.config.baudrate = baud,
                None => report!(|out: &mut LineWriter<'_>| {
                    out.push(b"[UART] Invalid baudrate ");
                    out.push(text);
                    out.push(b", using default");
                }),
            }
        }
        if let Some(text) = port::env(c"TELEMETRY_INTERVAL_MS") {
            match settings::parse_interval(text) {
                Some(ms) => self.telemetry_interval_ms = ms,
                None => report!(|out: &mut LineWriter<'_>| {
                    out.push(b"[UART] Invalid interval ");
                    out.push(text);
                    out.push(b", using default");
                }),
            }
        }
        c::console(format_args!(
            "[UART] Configuration: device={}, baudrate={}, interval={}ms",
            core::str::from_utf8(self.config.device()).unwrap_or("?"),
            self.config.baudrate,
            self.telemetry_interval_ms
        ));
    }

    /// The target has no environment: the settings stay as they are
    #[cfg(target_os = "none")]
    fn configure_from_env(&mut self) {
        c::console(format_args!(
            "[UART] Configuration: device={}, baudrate={}, interval={}ms",
            core::str::from_utf8(self.config.device()).unwrap_or("?"),
            self.config.baudrate,
            self.telemetry_interval_ms
        ));
    }
}

/// Initialize with the default configuration and the environment
/// overrides; nothing if already initialized
#[unsafe(no_mangle)]
pub extern "C" fn uart_init() -> i32 {
    UART.with(|u| {
        if u.initialized {
            return rc::OK;
        }
        u.configure_from_env();
        let config = u.config;
        u.open(config)
    })
    .unwrap_or(rc::BUSY)
}

/// Initialize with `config` (closing the line first if open)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uart_init_with_config(config: *const UartConfig) -> i32 {
    // SAFETY: the C contract: NULL or a UartConfig_t (same layout)
    let Some(config) = (unsafe { ptr::in_(config) }).copied() else {
        c::assert_failed(c"Null pointer: config", line!());
        return rc::NULL_PTR;
    };
    UART.with(|u| u.open(config)).unwrap_or(rc::BUSY)
}

/// Close the line
#[unsafe(no_mangle)]
pub extern "C" fn uart_close() -> i32 {
    UART.with(Uart::close).map_or(rc::BUSY, |()| rc::OK)
}

/// Whether the line is initialized
#[unsafe(no_mangle)]
pub extern "C" fn uart_is_initialized() -> bool {
    UART.with(|u| u.initialized).unwrap_or(false)
}

/// Send `len` bytes; on a failed write, reopen the line and try once more
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uart_send(data: *const c_char, len: usize) -> i32 {
    // SAFETY: the C contract: NULL or len readable bytes
    let Some(data) = (unsafe { ptr::bytes(data.cast(), len) }) else {
        c::assert_failed(c"Null pointer: data", line!());
        return rc::NULL_PTR;
    };
    if data.is_empty() {
        c::assert_failed(c"Invalid length", line!());
        return rc::INVALID;
    }
    UART.with(|u| {
        if !u.is_open() {
            return rc::ERROR;
        }
        if !port::write(u.fd, data) {
            report!(|out: &mut LineWriter<'_>| {
                out.push(b"[UART] Write failed: ");
                port::push_error(out);
            });
            u.close();
            let config = u.config;
            if u.open(config) != rc::OK {
                return rc::IO;
            }
            if !port::write(u.fd, data) {
                report!(|out: &mut LineWriter<'_>| {
                    out.push(b"[UART] Retry failed: ");
                    port::push_error(out);
                });
                return rc::IO;
            }
        }
        port::drain(u.fd);
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Receive what has arrived, up to `buffer_len` bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uart_receive(buffer: *mut c_char, buffer_len: usize, received: *mut usize) -> i32 {
    if buffer.is_null() {
        c::assert_failed(c"Null pointer: buffer", line!());
        return rc::NULL_PTR;
    }
    if buffer_len == 0 {
        c::assert_failed(c"Invalid buffer length", line!());
        return rc::INVALID;
    }
    // SAFETY: the C contract: NULL or a size_t
    let Some(received) = (unsafe { ptr::out(received) }) else {
        c::assert_failed(c"Null pointer: received", line!());
        return rc::NULL_PTR;
    };
    // SAFETY: the C contract: buffer_len writable bytes
    let Some(buf) = (unsafe { ptr::bytes_mut(buffer.cast(), buffer_len) }) else { return rc::NULL_PTR };
    let result = UART.with(|u| {
        if !u.is_open() {
            return Err(rc::ERROR);
        }
        port::read(u.fd, buf).ok_or(rc::IO)
    });
    match result {
        Some(Ok(n)) => {
            *received = n;
            rc::OK
        }
        Some(Err(e)) => {
            *received = 0;
            e
        }
        None => {
            *received = 0;
            rc::BUSY
        }
    }
}

/// Check the line; if it fails, fault-log it and reopen the line
#[unsafe(no_mangle)]
pub extern "C" fn uart_check_health() -> i32 {
    match UART.with(|u| u.initialized.then(|| port::healthy(u.fd))) {
        None => return rc::BUSY,
        Some(None) => return rc::ERROR,
        Some(Some(true)) => return rc::OK,
        Some(Some(false)) => {}
    }
    report!(|out: &mut LineWriter<'_>| {
        out.push(b"[UART] Health check failed, attempting recovery");
    });
    c::log(fault_type::UART, severity::WARNING, c"UART health check failed", 0.0);
    UART.with(|u| {
        u.close();
        let config = u.config;
        u.open(config)
    })
    .unwrap_or(rc::BUSY)
}

/// The current configuration
#[unsafe(no_mangle)]
pub unsafe extern "C" fn uart_get_config(config: *mut UartConfig) -> i32 {
    // SAFETY: the C contract: NULL or a UartConfig_t (same layout)
    let Some(out) = (unsafe { ptr::out(config) }) else {
        c::assert_failed(c"Null pointer: config", line!());
        return rc::NULL_PTR;
    };
    match UART.with(|u| u.config) {
        Some(c) => {
            *out = c;
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// The telemetry interval, ms
#[unsafe(no_mangle)]
pub extern "C" fn uart_get_telemetry_interval() -> u32 {
    UART.with(|u| u.telemetry_interval_ms).unwrap_or(settings::TELEMETRY_INTERVAL_DEFAULT_MS)
}
