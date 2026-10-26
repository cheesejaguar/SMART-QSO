//! `hal/*.h` for the host simulation, and the test hooks of `hal/hal_sim.h`.
//!
//! The simulated peripherals behave like simple hardware: GPIO pins hold a
//! level, three I2C devices have 256 registers each, UART ports have a
//! receive FIFO that tests fill, the flash regions are RAM, the radio has a
//! queue of injected frames and keeps the frames it was given to transmit
//! (and writes them as KISS to `$TNC_DEVICE` if set, as to a TNC), and the
//! watchdog counts down on the HAL timer. Tests use the `hal_sim_*` hooks to
//! inject inputs and faults.
//!
//! Compiled only for the host; the firmware implements the same functions
//! for the STM32L4 (`software/rust/firmware`).
//!
//! @requirement SRS-Q022 Software shall support simulation and flight builds

use core::ffi::{c_char, c_void};

use crate::c::rc;
use crate::cell::Global;
use crate::{os, ptr, time};

const GPIO_PIN_COUNT: usize = 16;
const GPIO_DIR_INPUT: u32 = 0;
const GPIO_DIR_OUTPUT: u32 = 1;
const I2C_BUS_COUNT: usize = 2;
const UART_PORT_COUNT: usize = 2;
const UART_PORT_DEBUG: u32 = 1;
const SPI_BUS_COUNT: usize = 2;
const ADC_CHANNEL_COUNT: usize = 10;
const FLASH_REGION_COUNT: usize = 11;
const RADIO_MAX_FRAME: usize = 330;
const WDT_DEFAULT_TIMEOUT_MS: u32 = 30_000;
const WDT_MIN_TIMEOUT_MS: u32 = 1000;
const WDT_MAX_TIMEOUT_MS: u32 = 60_000;
const RESET_POWER_ON: u32 = 0;

const I2C_ADDR_EPS_CONTROLLER: u8 = 0x48;
const I2C_ADDR_BATTERY_MONITOR: u8 = 0x55;
const I2C_ADDR_SOLAR_MONITOR: u8 = 0x40;

const UART_RX_SIZE: usize = 512;
const RADIO_QUEUE: usize = 4;

/// Simulated ADC inputs, volts (or the sensor's unit): VBATT, IBATT
/// (discharging), VSOLAR, ISOLAR, VBUS, TEMP_MCU, TEMP_BOARD, and the
/// downlink transmitter's TX_TEMP (°C), TX_FWD and TX_REV (W, matched at
/// 0.5 W). `hal_adc_init()` restores these; `hal_sim_adc_set()` changes one
const ADC_VALUES: [f64; ADC_CHANNEL_COUNT] = [7.8, -0.5, 9.2, 0.45, 5.0, 35.0, 28.0, 30.0, 0.5, 0.01];

/// Region sizes, bytes, in `HalFlashRegion_t` order
const FLASH_SIZES: [usize; FLASH_REGION_COUNT] = [
    512,  // MISSION_DATA
    256,  // EPS_CONFIG
    512,  // SENSOR_CONFIG
    4096, // FAULT_LOG
    1024, // BACKUP
    512,  // STATE (SystemState_t)
    768,  // AUTH (key table: CMD_AUTH_NVM_SIZE, six two-slot records)
    256,  // RF_CONTROL (TX latch and power-level records)
    256,  // CONTACT (loss-of-contact counter and timeout records)
    128,  // RF_SILENCE (post-deployment silence record)
    128,  // DEPLOYMENT (deployment sequence record)
];
const FLASH_TOTAL: usize = {
    let mut total = 0;
    let mut i = 0;
    while i < FLASH_REGION_COUNT {
        #[allow(clippy::indexing_slicing)] // i < len, at compile time
        {
            total += FLASH_SIZES[i];
        }
        i += 1;
    }
    total
};

struct I2cDevice {
    addr: u8,
    registers: [u8; 256],
}

/// A receive FIFO
struct Fifo<const N: usize> {
    buf: [u8; N],
    head: usize,
    count: usize,
}

impl<const N: usize> Fifo<N> {
    const EMPTY: Self = Fifo { buf: [0; N], head: 0, count: 0 };

    fn push(&mut self, byte: u8) {
        if let Some(slot) = self.buf.get_mut((self.head + self.count) % N) {
            *slot = byte;
            self.count += 1;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.count == 0 {
            return None;
        }
        let byte = self.buf.get(self.head).copied();
        self.head = (self.head + 1) % N;
        self.count -= 1;
        byte
    }
}

struct Flash {
    /// Regions are "allocated" (zeroed) by hal_flash_init after
    /// hal_deinit, as the C simulation's calloc did
    allocated: bool,
    initialized: bool,
    data: [u8; FLASH_TOTAL],
    fail_writes: bool,
    /// Writes left before writes fail (-1 = off)
    writes_before_fail: i32,
}

impl Flash {
    /// The bytes of region `region`
    fn region(&mut self, region: u32) -> Option<&mut [u8]> {
        let index = usize::try_from(region).ok()?;
        let size = *FLASH_SIZES.get(index)?;
        let start: usize = FLASH_SIZES.get(..index)?.iter().sum();
        self.data.get_mut(start..start + size)
    }
}

struct Wdt {
    initialized: bool,
    running: bool,
    timeout_ms: u32,
    last_kick_ms: u64,
    reset_flag: bool,
}

struct Radio {
    frames: [[u8; RADIO_MAX_FRAME]; RADIO_QUEUE],
    lens: [usize; RADIO_QUEUE],
    head: usize,
    count: usize,
}

impl Radio {
    const EMPTY: Radio =
        Radio { frames: [[0; RADIO_MAX_FRAME]; RADIO_QUEUE], lens: [0; RADIO_QUEUE], head: 0, count: 0 };

    /// Append `frame`; when full, the oldest is dropped if `overwrite`,
    /// else nothing is stored
    fn push(&mut self, frame: &[u8], overwrite: bool) -> bool {
        if self.count >= RADIO_QUEUE {
            if !overwrite {
                return false;
            }
            self.head = (self.head + 1) % RADIO_QUEUE;
            self.count -= 1;
        }
        let tail = (self.head + self.count) % RADIO_QUEUE;
        let (Some(dst), Some(n)) =
            (self.frames.get_mut(tail).and_then(|f| f.get_mut(..frame.len())), self.lens.get_mut(tail))
        else {
            return false;
        };
        dst.copy_from_slice(frame);
        *n = frame.len();
        self.count += 1;
        true
    }

    /// Take the oldest frame into `buf`
    fn pop(&mut self, buf: &mut [u8]) -> Result<usize, i32> {
        if self.count == 0 {
            return Err(rc::TIMEOUT);
        }
        let n = self.lens.get(self.head).copied().unwrap_or(0);
        let (Some(dst), Some(src)) = (buf.get_mut(..n), self.frames.get(self.head).and_then(|f| f.get(..n))) else {
            return Err(rc::INVALID);
        };
        dst.copy_from_slice(src);
        self.head = (self.head + 1) % RADIO_QUEUE;
        self.count -= 1;
        Ok(n)
    }
}

struct Sim {
    gpio_initialized: bool,
    gpio_state: [bool; GPIO_PIN_COUNT],
    gpio_dir: [u32; GPIO_PIN_COUNT],
    /// Levels the outside world drives (`hal_sim_gpio_drive_input()`): they
    /// survive `hal_gpio_init()`, as a switch's level survives a reset
    gpio_driven: [Option<bool>; GPIO_PIN_COUNT],
    i2c_initialized: [bool; I2C_BUS_COUNT],
    i2c: [I2cDevice; 3],
    uart_initialized: [bool; UART_PORT_COUNT],
    uart_rx: [Fifo<UART_RX_SIZE>; UART_PORT_COUNT],
    spi_initialized: [bool; SPI_BUS_COUNT],
    adc_initialized: bool,
    adc_values: [f64; ADC_CHANNEL_COUNT],
    flash: Flash,
    radio: Radio,
    radio_ready: bool,
    radio_tx: Radio,
    radio_tx_result: i32,
    tnc_fd: i32,
    wdt: Wdt,
    reset_cause: u32,
}

static SIM: Global<Sim> = Global::new(Sim {
    gpio_initialized: false,
    gpio_state: [false; GPIO_PIN_COUNT],
    gpio_dir: [GPIO_DIR_INPUT; GPIO_PIN_COUNT],
    gpio_driven: [None; GPIO_PIN_COUNT],
    i2c_initialized: [false; I2C_BUS_COUNT],
    i2c: [
        I2cDevice { addr: I2C_ADDR_EPS_CONTROLLER, registers: [0; 256] },
        I2cDevice { addr: I2C_ADDR_BATTERY_MONITOR, registers: [0; 256] },
        I2cDevice { addr: I2C_ADDR_SOLAR_MONITOR, registers: [0; 256] },
    ],
    uart_initialized: [false; UART_PORT_COUNT],
    uart_rx: [Fifo::EMPTY, Fifo::EMPTY],
    spi_initialized: [false; SPI_BUS_COUNT],
    adc_initialized: false,
    adc_values: ADC_VALUES,
    flash: Flash {
        allocated: false,
        initialized: false,
        data: [0; FLASH_TOTAL],
        fail_writes: false,
        writes_before_fail: -1,
    },
    radio: Radio::EMPTY,
    radio_ready: false,
    radio_tx: Radio::EMPTY,
    radio_tx_result: rc::OK,
    tnc_fd: -1,
    wdt: Wdt {
        initialized: false,
        running: false,
        timeout_ms: WDT_DEFAULT_TIMEOUT_MS,
        last_kick_ms: 0,
        reset_flag: false,
    },
    reset_cause: RESET_POWER_ON,
});

/// Run `f` on the simulation state; `busy` if it is in use (a HAL call
/// from an interrupt-like context in the middle of another)
fn sim<R>(busy: R, f: impl FnOnce(&mut Sim) -> R) -> R {
    SIM.with(f).unwrap_or(busy)
}

fn index(value: u32, count: usize) -> Option<usize> {
    usize::try_from(value).ok().filter(|&i| i < count)
}

// ---------------------------------------------------------------------------
// hal.h

/// Initialize the timer, GPIO and flash
#[unsafe(no_mangle)]
pub extern "C" fn hal_init() -> i32 {
    match hal_timer_init() {
        rc::OK => {}
        e => return e,
    }
    match hal_gpio_init() {
        rc::OK => {}
        e => return e,
    }
    hal_flash_init()
}

/// Release the simulation: flash contents are lost, as on the C
/// simulation's free()
#[unsafe(no_mangle)]
pub extern "C" fn hal_deinit() -> i32 {
    sim(rc::BUSY, |s| {
        s.flash.allocated = false;
        s.gpio_initialized = false;
        s.flash.initialized = false;
        rc::OK
    });
    let _ = TIMER.with(|t| t.initialized = false);
    rc::OK
}

// ---------------------------------------------------------------------------
// hal_gpio.h

/// All pins inputs, low (or at the level the outside world drives)
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_init() -> i32 {
    sim(rc::BUSY, |s| {
        s.gpio_state = [false; GPIO_PIN_COUNT];
        for (level, driven) in s.gpio_state.iter_mut().zip(s.gpio_driven) {
            if let Some(d) = driven {
                *level = d;
            }
        }
        s.gpio_dir = [GPIO_DIR_INPUT; GPIO_PIN_COUNT];
        s.gpio_initialized = true;
        rc::OK
    })
}

/// Set a pin's direction (pull is not simulated)
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_config(pin: u32, direction: u32, _pull: u32) -> i32 {
    sim(rc::BUSY, |s| {
        if !s.gpio_initialized {
            return rc::ERROR;
        }
        match s.gpio_dir.get_mut(index(pin, GPIO_PIN_COUNT).unwrap_or(usize::MAX)) {
            Some(dir) => {
                *dir = direction;
                rc::OK
            }
            None => rc::INVALID,
        }
    })
}

/// Drive an output pin
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_set(pin: u32, state: bool) -> i32 {
    sim(rc::BUSY, |s| {
        if !s.gpio_initialized {
            return rc::ERROR;
        }
        let Some(i) = index(pin, GPIO_PIN_COUNT) else { return rc::INVALID };
        if s.gpio_dir.get(i) != Some(&GPIO_DIR_OUTPUT) {
            return rc::INVALID;
        }
        if let Some(level) = s.gpio_state.get_mut(i) {
            *level = state;
        }
        rc::OK
    })
}

/// A pin's level (false before init or for an unknown pin)
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_get(pin: u32) -> bool {
    sim(false, |s| {
        s.gpio_initialized && index(pin, GPIO_PIN_COUNT).and_then(|i| s.gpio_state.get(i).copied()).unwrap_or(false)
    })
}

/// Invert a pin's level
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_toggle(pin: u32) -> i32 {
    sim(rc::BUSY, |s| {
        if !s.gpio_initialized {
            return rc::ERROR;
        }
        match s.gpio_state.get_mut(index(pin, GPIO_PIN_COUNT).unwrap_or(usize::MAX)) {
            Some(level) => {
                *level = !*level;
                rc::OK
            }
            None => rc::INVALID,
        }
    })
}

// ---------------------------------------------------------------------------
// hal_i2c.h

/// Reset every simulated device's registers (EPS: charging and sunlit,
/// mode IDLE, SOC 75 %)
#[unsafe(no_mangle)]
pub extern "C" fn hal_i2c_init(bus: u32, _speed: u32) -> i32 {
    let Some(bus) = index(bus, I2C_BUS_COUNT) else { return rc::INVALID };
    sim(rc::BUSY, |s| {
        for dev in &mut s.i2c {
            dev.registers = [0; 256];
            if dev.addr == I2C_ADDR_EPS_CONTROLLER {
                dev.registers[0x00] = 0x43; // STATUS: charging, sunlit
                dev.registers[0x01] = 0x01; // MODE: IDLE
                dev.registers[0x16] = 75; // SOC: 75 %
            }
        }
        if let Some(init) = s.i2c_initialized.get_mut(bus) {
            *init = true;
        }
        rc::OK
    })
}

fn device(s: &mut Sim, addr: u8) -> Option<&mut I2cDevice> {
    s.i2c.iter_mut().find(|d| d.addr == addr)
}

fn bus_ready(s: &Sim, bus: u32) -> bool {
    index(bus, I2C_BUS_COUNT).and_then(|b| s.i2c_initialized.get(b).copied()).unwrap_or(false)
}

/// Write registers: the first byte is the register address, the rest the
/// values (the address wraps at 0xFF)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_i2c_write(bus: u32, addr: u8, data: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len bytes
    let data = unsafe { ptr::bytes(data, len) };
    sim(rc::BUSY, |s| {
        if !bus_ready(s, bus) {
            return rc::ERROR;
        }
        let Some([reg, values @ ..]) = data else { return rc::NULL_PTR };
        let Some(dev) = device(s, addr) else { return rc::ERROR };
        let mut reg = *reg;
        for &v in values {
            if let Some(r) = dev.registers.get_mut(usize::from(reg)) {
                *r = v;
            }
            reg = reg.wrapping_add(1);
        }
        rc::OK
    })
}

/// Read registers from register 0
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_i2c_read(bus: u32, addr: u8, data: *mut u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len writable bytes
    let data = unsafe { ptr::bytes_mut(data, len) };
    sim(rc::BUSY, |s| {
        if !bus_ready(s, bus) {
            return rc::ERROR;
        }
        let Some(data) = data.filter(|d| !d.is_empty()) else { return rc::NULL_PTR };
        let Some(dev) = device(s, addr) else { return rc::ERROR };
        match dev.registers.get(..data.len()) {
            Some(regs) => {
                data.copy_from_slice(regs);
                rc::OK
            }
            None => rc::INVALID,
        }
    })
}

/// Write the register address (first byte), then read from it
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_i2c_write_read(
    _bus: u32,
    addr: u8,
    write_data: *const u8,
    write_len: usize,
    read_data: *mut u8,
    read_len: usize,
) -> i32 {
    // SAFETY: the C contract: write_data points to write_len bytes and
    // read_data to read_len writable bytes
    let (write, read) = unsafe { (ptr::bytes(write_data, write_len), ptr::bytes_mut(read_data, read_len)) };
    sim(rc::BUSY, |s| {
        let Some([reg, ..]) = write else { return rc::NULL_PTR };
        let Some(dev) = device(s, addr) else { return rc::ERROR };
        let Some(read) = read else { return rc::NULL_PTR };
        let start = usize::from(*reg);
        match dev.registers.get(start..start + read.len()) {
            Some(regs) => {
                read.copy_from_slice(regs);
                rc::OK
            }
            None => rc::INVALID,
        }
    })
}

/// Whether a simulated device answers at `addr`
#[unsafe(no_mangle)]
pub extern "C" fn hal_i2c_device_present(_bus: u32, addr: u8) -> bool {
    sim(false, |s| device(s, addr).is_some())
}

/// Bus recovery (nothing to recover in simulation)
#[unsafe(no_mangle)]
pub extern "C" fn hal_i2c_recover(_bus: u32) -> i32 {
    rc::OK
}

// ---------------------------------------------------------------------------
// hal_uart.h

/// Open a port (the configuration is not simulated)
#[unsafe(no_mangle)]
pub extern "C" fn hal_uart_init(port: u32, _config: *const c_void) -> i32 {
    let Some(port) = index(port, UART_PORT_COUNT) else { return rc::INVALID };
    sim(rc::BUSY, |s| {
        if let Some(init) = s.uart_initialized.get_mut(port) {
            *init = true;
        }
        rc::OK
    })
}

/// Transmit; the debug port's output goes to stdout
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_uart_write(port: u32, data: *const u8, len: usize) -> i32 {
    let ready =
        sim(false, |s| index(port, UART_PORT_COUNT).and_then(|p| s.uart_initialized.get(p).copied()) == Some(true));
    if !ready {
        return rc::ERROR;
    }
    // SAFETY: the C contract: data points to len bytes
    let Some(data) = (unsafe { ptr::bytes(data, len) }) else { return rc::NULL_PTR };
    if port == UART_PORT_DEBUG {
        os::print_bytes(data);
    }
    rc::OK
}

/// Return injected bytes, if any; otherwise time out
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_uart_read(
    port: u32,
    data: *mut u8,
    len: usize,
    _timeout_ms: u32,
    bytes_read: *mut usize,
) -> i32 {
    // SAFETY: the C contract: data points to len writable bytes, bytes_read to a size_t
    let (data, out) = unsafe { (ptr::bytes_mut(data, len), ptr::out(bytes_read)) };
    sim(rc::BUSY, |s| {
        let Some(p) = index(port, UART_PORT_COUNT).filter(|&p| s.uart_initialized.get(p) == Some(&true)) else {
            return rc::ERROR;
        };
        let (Some(data), Some(out)) = (data, out) else { return rc::NULL_PTR };
        let Some(fifo) = s.uart_rx.get_mut(p) else { return rc::ERROR };
        let mut n = 0;
        for slot in data.iter_mut() {
            let Some(byte) = fifo.pop() else { break };
            *slot = byte;
            n += 1;
        }
        *out = n;
        if n > 0 { rc::OK } else { rc::TIMEOUT }
    })
}

/// Line reads are not simulated: always times out
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_uart_read_line(
    _port: u32,
    _buffer: *mut c_char,
    _buffer_len: usize,
    _timeout_ms: u32,
    line_len: *mut usize,
) -> i32 {
    // SAFETY: the C contract: NULL or a size_t
    if let Some(len) = unsafe { ptr::out(line_len) } {
        *len = 0;
    }
    rc::TIMEOUT
}

/// Bytes waiting in the receive FIFO
#[unsafe(no_mangle)]
pub extern "C" fn hal_uart_available(port: u32) -> usize {
    sim(0, |s| index(port, UART_PORT_COUNT).and_then(|p| s.uart_rx.get(p)).map_or(0, |f| f.count))
}

/// Discard the receive FIFO
#[unsafe(no_mangle)]
pub extern "C" fn hal_uart_flush(port: u32) -> i32 {
    let Some(p) = index(port, UART_PORT_COUNT) else { return rc::INVALID };
    sim(rc::BUSY, |s| {
        if let Some(fifo) = s.uart_rx.get_mut(p) {
            *fifo = Fifo::EMPTY;
        }
        rc::OK
    })
}

// ---------------------------------------------------------------------------
// hal_spi.h

/// Open a bus (the configuration is not simulated)
#[unsafe(no_mangle)]
pub extern "C" fn hal_spi_init(bus: u32, _config: *const c_void) -> i32 {
    let Some(bus) = index(bus, SPI_BUS_COUNT) else { return rc::INVALID };
    sim(rc::BUSY, |s| {
        if let Some(init) = s.spi_initialized.get_mut(bus) {
            *init = true;
        }
        rc::OK
    })
}

/// Transfer: transmitted bytes are dropped, received bytes are zero
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_transfer(bus: u32, _tx_data: *const u8, rx_data: *mut u8, len: usize) -> i32 {
    let ready = sim(false, |s| index(bus, SPI_BUS_COUNT).and_then(|b| s.spi_initialized.get(b).copied()) == Some(true));
    if !ready {
        return rc::ERROR;
    }
    // SAFETY: the C contract: NULL or len writable bytes
    if let Some(rx) = unsafe { ptr::bytes_mut(rx_data, len) } {
        rx.fill(0);
    }
    rc::OK
}

/// Transmit only
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_write(bus: u32, data: *const u8, len: usize) -> i32 {
    // SAFETY: forwarded contract; no receive buffer
    unsafe { hal_spi_transfer(bus, data, core::ptr::null_mut(), len) }
}

/// Receive only
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_spi_read(bus: u32, data: *mut u8, len: usize) -> i32 {
    // SAFETY: forwarded contract
    unsafe { hal_spi_transfer(bus, core::ptr::null(), data, len) }
}

/// Chip select (not simulated)
#[unsafe(no_mangle)]
pub extern "C" fn hal_spi_cs_assert(_bus: u32) -> i32 {
    rc::OK
}

/// Chip deselect (not simulated)
#[unsafe(no_mangle)]
pub extern "C" fn hal_spi_cs_deassert(_bus: u32) -> i32 {
    rc::OK
}

// ---------------------------------------------------------------------------
// hal_timer.h

struct Timer {
    initialized: bool,
    start_us: u64,
}

static TIMER: Global<Timer> = Global::new(Timer { initialized: false, start_us: 0 });

/// Wall-clock time plus the test offset, so the HAL timer and
/// smart_qso_now_ms() agree
fn time_us() -> u64 {
    os::realtime_us().wrapping_add(time::smart_qso_sim_offset_ms().wrapping_mul(1000))
}

/// Start the time base
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_init() -> i32 {
    let now = time_us();
    let _ = TIMER.with(|t| {
        t.start_us = now;
        t.initialized = true;
    });
    rc::OK
}

/// Microseconds since hal_timer_init (0 before it)
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_get_us() -> u64 {
    let now = time_us();
    TIMER.with(|t| if t.initialized { now.wrapping_sub(t.start_us) } else { 0 }).unwrap_or(0)
}

/// Milliseconds since hal_timer_init (0 before it)
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_get_ms() -> u64 {
    hal_timer_get_us() / 1000
}

/// Sleep
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_delay_ms(ms: u32) {
    os::sleep_us(u64::from(ms) * 1000);
}

/// Sleep
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_delay_us(us: u32) {
    os::sleep_us(u64::from(us));
}

/// Whether `timeout_ms` has passed since `start_ms`
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_timeout(start_ms: u64, timeout_ms: u32) -> bool {
    hal_timer_get_ms().wrapping_sub(start_ms) >= u64::from(timeout_ms)
}

/// Milliseconds since `start_ms`
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_elapsed_ms(start_ms: u64) -> u64 {
    hal_timer_get_ms().wrapping_sub(start_ms)
}

/// Periodic timers are not simulated: accepted, never fire
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_timer_start_periodic(
    _period_ms: u32,
    _callback: Option<unsafe extern "C" fn(*mut c_void)>,
    _context: *mut c_void,
    timer_id: *mut u8,
) -> i32 {
    // SAFETY: the C contract: NULL or a HalTimerId_t
    if let Some(id) = unsafe { ptr::out(timer_id) } {
        *id = 0;
    }
    rc::OK
}

/// Stop a periodic timer (not simulated)
#[unsafe(no_mangle)]
pub extern "C" fn hal_timer_stop(_timer_id: u8) -> i32 {
    rc::OK
}

// ---------------------------------------------------------------------------
// hal_adc.h

/// Start the converter (resolution and reference are not simulated)
#[unsafe(no_mangle)]
pub extern "C" fn hal_adc_init(_resolution: u32, _reference: u32) -> i32 {
    sim(rc::BUSY, |s| {
        s.adc_initialized = true;
        s.adc_values = ADC_VALUES;
        rc::OK
    })
}

fn adc_value(channel: u32) -> Result<f64, i32> {
    sim(Err(rc::BUSY), |s| {
        if !s.adc_initialized {
            return Err(rc::ERROR);
        }
        index(channel, ADC_CHANNEL_COUNT).and_then(|c| s.adc_values.get(c).copied()).ok_or(rc::INVALID)
    })
}

/// 12-bit reading of the simulated input against a 3.3 V reference
/// (negative inputs read 0)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_adc_read_raw(channel: u32, value: *mut u16) -> i32 {
    if !sim(false, |s| s.adc_initialized) {
        return rc::ERROR;
    }
    // SAFETY: the C contract: value points to a uint16_t
    let Some(out) = (unsafe { ptr::out(value) }) else { return rc::NULL_PTR };
    match adc_value(channel) {
        Ok(v) => {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // saturating, in range
            {
                *out = ((v / 3.3) * 4095.0) as u16;
            }
            rc::OK
        }
        Err(e) => e,
    }
}

/// The simulated input in its own unit
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_adc_read_voltage(channel: u32, voltage: *mut f64) -> i32 {
    if !sim(false, |s| s.adc_initialized) {
        return rc::ERROR;
    }
    // SAFETY: the C contract: voltage points to a double
    let Some(out) = (unsafe { ptr::out(voltage) }) else { return rc::NULL_PTR };
    match adc_value(channel) {
        Ok(v) => {
            *out = v;
            rc::OK
        }
        Err(e) => e,
    }
}

/// Raw readings of several channels; stops at the first failure
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_adc_read_multiple(channels: *const u32, values: *mut u16, count: usize) -> i32 {
    if channels.is_null() || values.is_null() {
        return rc::NULL_PTR;
    }
    for i in 0..count {
        // SAFETY: the C contract: both arrays hold count elements
        let result = unsafe { hal_adc_read_raw(channels.add(i).read(), values.add(i)) };
        if result != rc::OK {
            return result;
        }
    }
    rc::OK
}

/// Nothing to calibrate in simulation
#[unsafe(no_mangle)]
pub extern "C" fn hal_adc_calibrate() -> i32 {
    rc::OK
}

// ---------------------------------------------------------------------------
// hal_flash.h

/// Make the regions available; after hal_deinit they start zeroed again
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_init() -> i32 {
    sim(rc::BUSY, |s| {
        if !s.flash.allocated {
            s.flash.data.fill(0);
            s.flash.allocated = true;
        }
        s.flash.initialized = true;
        rc::OK
    })
}

/// The range `offset..offset+len` of `region`, if it is inside
fn span(flash: &mut Flash, region: u32, offset: u32, len: usize) -> Result<&mut [u8], i32> {
    let bytes = flash.region(region).ok_or(rc::INVALID)?;
    let start = usize::try_from(offset).map_err(|_| rc::INVALID)?;
    let end = start.checked_add(len).ok_or(rc::INVALID)?;
    bytes.get_mut(start..end).ok_or(rc::INVALID)
}

/// Read stored bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_flash_read(region: u32, offset: u32, data: *mut u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len writable bytes
    let data = unsafe { ptr::bytes_mut(data, len) };
    sim(rc::BUSY, |s| {
        if !s.flash.initialized {
            return rc::ERROR;
        }
        let Some(data) = data else { return rc::NULL_PTR };
        match span(&mut s.flash, region, offset, data.len()) {
            Ok(stored) => {
                data.copy_from_slice(stored);
                rc::OK
            }
            Err(e) => e,
        }
    })
}

/// Store bytes, unless a test has injected a write failure
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_flash_write(region: u32, offset: u32, data: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: data points to len bytes
    let data = unsafe { ptr::bytes(data, len) };
    sim(rc::BUSY, |s| {
        let flash = &mut s.flash;
        if !flash.initialized {
            return rc::ERROR;
        }
        if flash.fail_writes || flash.writes_before_fail == 0 {
            return rc::IO;
        }
        if flash.writes_before_fail > 0 {
            flash.writes_before_fail -= 1;
        }
        let Some(data) = data else { return rc::NULL_PTR };
        match span(flash, region, offset, data.len()) {
            Ok(stored) => {
                stored.copy_from_slice(data);
                rc::OK
            }
            Err(e) => e,
        }
    })
}

/// Erase a region to 0xFF
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_erase(region: u32) -> i32 {
    sim(rc::BUSY, |s| {
        if !s.flash.initialized {
            return rc::ERROR;
        }
        match s.flash.region(region) {
            Some(bytes) => {
                bytes.fill(0xFF);
                rc::OK
            }
            None => rc::INVALID,
        }
    })
}

/// A region's size in bytes (0 for an unknown region)
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_region_size(region: u32) -> usize {
    index(region, FLASH_REGION_COUNT).and_then(|r| FLASH_SIZES.get(r).copied()).unwrap_or(0)
}

/// Physical address (not applicable in simulation)
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_region_base(_region: u32) -> u32 {
    0
}

/// Never busy in simulation
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_busy() -> bool {
    false
}

/// No wear in simulation
#[unsafe(no_mangle)]
pub extern "C" fn hal_flash_wear_level(_region: u32) -> u8 {
    0
}

// ---------------------------------------------------------------------------
// hal_radio.h and hal_reset.h

/// The TNC (simulation: received frames injected by tests; transmitted
/// frames kept for them, and written as KISS to `$TNC_DEVICE` if set)
#[unsafe(no_mangle)]
pub extern "C" fn hal_radio_init() -> i32 {
    let device = os::env(c"TNC_DEVICE");
    sim(rc::BUSY, |s| {
        if s.tnc_fd < 0
            && let Some(path) = device
        {
            // SAFETY: path is NUL-terminated
            s.tnc_fd = unsafe { libc::open(path.as_ptr(), libc::O_WRONLY | libc::O_NOCTTY | libc::O_NONBLOCK) };
        }
        s.radio_ready = true;
        rc::OK
    })
}

/// The TNC's parameters and `frame` as KISS, into `out`
fn kiss_stream(frame: &[u8], out: &mut [u8]) -> Option<usize> {
    use sqso_protocol::kiss;
    let mut n = 0;
    for (command, value) in kiss::DOWNLINK_PARAMS {
        n += kiss::encode_command(command, value, 0, out.get_mut(n..)?).ok()?;
    }
    n += sqso_protocol::ax25::kiss_encode(frame, 0, out.get_mut(n..)?).ok()?;
    Some(n)
}

/// Keep the frame for `hal_sim_radio_take_tx()` (the newest four), and
/// write it to the TNC device if one is open
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_radio_transmit(frame: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: frame points to len bytes
    let Some(frame) = (unsafe { ptr::bytes(frame, len) }) else { return rc::NULL_PTR };
    if frame.is_empty() || frame.len() > RADIO_MAX_FRAME - 2 {
        return rc::INVALID;
    }
    sim(rc::BUSY, |s| {
        if !s.radio_ready {
            return rc::ERROR;
        }
        if s.radio_tx_result != rc::OK {
            return s.radio_tx_result;
        }
        let _ = s.radio_tx.push(frame, true);
        let mut wire = [0u8; 5 * 5 + 2 * RADIO_MAX_FRAME + 3];
        if s.tnc_fd >= 0
            && let Some(n) = kiss_stream(frame, &mut wire)
        {
            // SAFETY: wire holds n bytes; tnc_fd is an open descriptor
            let _ = unsafe { libc::write(s.tnc_fd, wire.as_ptr().cast(), n) };
        }
        rc::OK
    })
}

/// The next injected frame, or a timeout
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_radio_receive(frame: *mut u8, max_len: usize, len: *mut usize) -> i32 {
    // SAFETY: the C contract: frame points to max_len writable bytes, len to a size_t
    let (buf, out) = unsafe { (ptr::bytes_mut(frame, max_len), ptr::out(len)) };
    let (Some(buf), Some(out)) = (buf, out) else { return rc::NULL_PTR };
    sim(rc::BUSY, |s| match s.radio.pop(buf) {
        Ok(n) => {
            *out = n;
            rc::OK
        }
        Err(e) => e,
    })
}

/// What caused the last reset
#[unsafe(no_mangle)]
pub extern "C" fn hal_reset_cause() -> u32 {
    sim(RESET_POWER_ON, |s| s.reset_cause)
}

// ---------------------------------------------------------------------------
// hal_wdt.h

/// Set the timeout (1–60 s); the mode is not simulated
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_init(timeout_ms: u32, _mode: u32) -> i32 {
    if !(WDT_MIN_TIMEOUT_MS..=WDT_MAX_TIMEOUT_MS).contains(&timeout_ms) {
        return rc::INVALID;
    }
    sim(rc::BUSY, |s| {
        s.wdt.timeout_ms = timeout_ms;
        s.wdt.initialized = true;
        rc::OK
    })
}

/// Start counting down
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_start() -> i32 {
    let now = hal_timer_get_ms();
    sim(rc::BUSY, |s| {
        if !s.wdt.initialized {
            return rc::ERROR;
        }
        s.wdt.running = true;
        s.wdt.last_kick_ms = now;
        rc::OK
    })
}

/// Stop counting down
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_stop() -> i32 {
    sim(rc::BUSY, |s| {
        s.wdt.running = false;
        rc::OK
    })
}

/// Restart the countdown
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_kick() {
    let now = hal_timer_get_ms();
    sim((), |s| {
        if s.wdt.running {
            s.wdt.last_kick_ms = now;
        }
    });
}

/// Time left before the watchdog would reset (the full timeout while stopped)
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_remaining_ms() -> u32 {
    let now = hal_timer_get_ms();
    sim(0, |s| {
        if !s.wdt.running {
            return s.wdt.timeout_ms;
        }
        let elapsed = now.wrapping_sub(s.wdt.last_kick_ms);
        u32::try_from(u64::from(s.wdt.timeout_ms).saturating_sub(elapsed)).unwrap_or(0)
    })
}

/// Whether the last reset was the watchdog's
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_caused_reset() -> bool {
    sim(false, |s| s.wdt.reset_flag)
}

/// Clear the watchdog reset flag
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_clear_reset_flag() {
    sim((), |s| s.wdt.reset_flag = false);
}

/// Early-warning callbacks are not simulated
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_set_warning_callback(
    _callback: Option<unsafe extern "C" fn(*mut c_void)>,
    _context: *mut c_void,
) -> i32 {
    rc::OK
}

// ---------------------------------------------------------------------------
// hal_sim.h test hooks

/// Every flash write fails (or stops failing)
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_flash_fail_writes(fail: bool) {
    sim((), |s| s.flash.fail_writes = fail);
}

/// Flash writes fail after `writes` more succeed (-1: never)
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_flash_fail_writes_after(writes: i32) {
    sim((), |s| s.flash.writes_before_fail = writes);
}

/// Set an input pin's level, as the outside world would (it stays at that
/// level across `hal_gpio_init()`)
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_gpio_drive_input(pin: u32, state: bool) {
    sim((), |s| {
        if let Some(i) = index(pin, GPIO_PIN_COUNT) {
            if let Some(level) = s.gpio_state.get_mut(i) {
                *level = state;
            }
            if let Some(driven) = s.gpio_driven.get_mut(i) {
                *driven = Some(state);
            }
        }
    });
}

/// Queue received bytes on a UART port
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_sim_uart_inject(port: u32, data: *const u8, len: usize) -> i32 {
    let Some(p) = index(port, UART_PORT_COUNT) else { return rc::INVALID };
    // SAFETY: the C contract: data points to len bytes
    let Some(data) = (unsafe { ptr::bytes(data, len) }) else { return rc::INVALID };
    sim(rc::BUSY, |s| {
        let Some(fifo) = s.uart_rx.get_mut(p) else { return rc::INVALID };
        if data.len() > UART_RX_SIZE - fifo.count {
            return rc::NO_MEM;
        }
        for &b in data {
            fifo.push(b);
        }
        rc::OK
    })
}

/// Queue a received telecommand frame
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_sim_radio_inject(frame: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: frame points to len bytes
    let Some(frame) = (unsafe { ptr::bytes(frame, len) }).filter(|f| f.len() <= RADIO_MAX_FRAME) else {
        return rc::INVALID;
    };
    sim(rc::BUSY, |s| if s.radio.push(frame, false) { rc::OK } else { rc::NO_MEM })
}

/// Drop every queued frame
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_radio_clear() {
    sim((), |s| {
        s.radio.head = 0;
        s.radio.count = 0;
    });
}

/// The oldest frame given to `hal_radio_transmit()` and not yet taken
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_sim_radio_take_tx(frame: *mut u8, max_len: usize, len: *mut usize) -> i32 {
    // SAFETY: the C contract: frame points to max_len writable bytes, len to a size_t
    let (buf, out) = unsafe { (ptr::bytes_mut(frame, max_len), ptr::out(len)) };
    let (Some(buf), Some(out)) = (buf, out) else { return rc::NULL_PTR };
    sim(rc::BUSY, |s| match s.radio_tx.pop(buf) {
        Ok(n) => {
            *out = n;
            rc::OK
        }
        Err(e) => e,
    })
}

/// What `hal_radio_transmit()` returns from now on (`SMART_QSO_OK`: it
/// takes frames again); kept frames are dropped
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_radio_tx_result(result: i32) {
    sim((), |s| {
        s.radio_tx_result = result;
        s.radio_tx.head = 0;
        s.radio_tx.count = 0;
    });
}

/// Set one simulated ADC input, in its own unit, until `hal_adc_init()`
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_adc_set(channel: u32, value: f64) -> i32 {
    sim(rc::BUSY, |s| match index(channel, ADC_CHANNEL_COUNT).and_then(|c| s.adc_values.get_mut(c)) {
        Some(v) => {
            *v = value;
            rc::OK
        }
        None => rc::INVALID,
    })
}

/// The reset cause the next boot will see
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_set_reset_cause(cause: u32) {
    sim((), |s| s.reset_cause = cause);
}
