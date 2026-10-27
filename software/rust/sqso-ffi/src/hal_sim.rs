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

const GPIO_PIN_COUNT: usize = crate::pins::GPIO_PIN_COUNT as usize;
const GPIO_DIR_INPUT: u32 = 0;
const GPIO_PULL_NONE: u32 = 0;
const GPIO_PULL_UP: u32 = 1;
const GPIO_PULL_DOWN: u32 = 2;
const GPIO_DIR_OUTPUT: u32 = 1;
/// `GPIO_PIN_XPDR_EN`, the heartbeat pin
const GPIO_PIN_XPDR_EN: u32 = 16;
/// `HAL_GPIO_HEARTBEAT_HZ`: the wave's half period, µs
const HEARTBEAT_HALF_PERIOD_US: u64 = 1_000_000 / (2 * 100);
const I2C_BUS_COUNT: usize = 3;
const UART_PORT_COUNT: usize = 3;
const UART_PORT_DEBUG: u32 = 1;
const SPI_BUS_COUNT: usize = 3;
const ADC_CHANNEL_COUNT: usize = crate::pins::ADC_CHANNEL_COUNT as usize;
const FLASH_REGION_COUNT: usize = 17;
const RADIO_MAX_FRAME: usize = 330;
const WDT_DEFAULT_TIMEOUT_MS: u32 = 30_000;
const WDT_MIN_TIMEOUT_MS: u32 = 1000;
const WDT_MAX_TIMEOUT_MS: u32 = 60_000;
const RESET_POWER_ON: u32 = 0;

const I2C_ADDR_EPS_CONTROLLER: u8 = 0x48;
const I2C_ADDR_BATTERY_MONITOR: u8 = sqso_fsw::ina228::ADDRESS_PACK;
const I2C_ADDR_SOLAR_MONITOR: u8 = sqso_fsw::ina228::ADDRESS_ARRAY;
/// The ISIS AntS controller (`sqso_fsw::ants`): one-byte commands latch
/// their register to 1; the status (0xC3, 2 bytes) reads stowed until set
const I2C_ADDR_ANTS: u8 = sqso_fsw::ants::ADDRESS;

const UART_RX_SIZE: usize = 512;
const RADIO_QUEUE: usize = 4;

/// Simulated ADC inputs, volts (or the sensor's unit): VBATT, IBATT
/// (discharging), VSOLAR, ISOLAR, VBUS, TEMP_MCU, TEMP_BOARD, the downlink
/// transmitter's TX_TEMP (°C), TX_FWD and TX_REV (W, matched at 0.5 W), the
/// Jetson switch readback VSW1_RB (V), the spreader thermistor PT_NTC (pin
/// V: 1.65 V is 25 °C), the three coil currents (A) and the eFuse readback
/// VJET_RB (V). `hal_adc_init()` restores these; `hal_sim_adc_set()`
/// changes one. The two readbacks follow their switches unless a test or
/// the mission simulation has set them ([`readback`])
const ADC_VALUES: [f64; ADC_CHANNEL_COUNT] =
    [7.8, -0.5, 9.2, 0.45, 5.0, 35.0, 28.0, 30.0, 0.5, 0.01, 0.0, 1.65, 0.0, 0.0, 0.0, 0.0];

/// Region sizes, bytes, in `HalFlashRegion_t` order
const FLASH_SIZES: [usize; FLASH_REGION_COUNT] = [
    512,                               // MISSION_DATA
    256,                               // EPS_CONFIG
    512,                               // SENSOR_CONFIG
    4096,                              // FAULT_LOG
    1024,                              // BACKUP
    512,                               // STATE (SystemState_t)
    768,                               // AUTH (key table: CMD_AUTH_NVM_SIZE, six two-slot records)
    256,                               // RF_CONTROL (TX latch and power-level records)
    256,                               // CONTACT (loss-of-contact counter and timeout records)
    128,                               // RF_SILENCE (post-deployment silence record)
    128,                               // DEPLOYMENT (deployment sequence record)
    128,                               // OPS (commissioning hold and transponder enable record)
    sqso_fsw::hk_store::SUMMARY_BYTES, // HK_SUMMARY (five-minute housekeeping records)
    65_536,                            // HK_DETAIL (sixty-second records: ring and event windows)
    256,                               // HK_INDEX (the event-window table)
    128,                               // EOL (passivation record)
    256,                               // ADCS_CAL (gyro alignment and magnetometer bias records)
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

/// A register file. The INA228s' registers are wider than a byte, so for
/// them register r is the bytes from 5r ([`word_device`]); the others are
/// byte registers
struct I2cDevice {
    addr: u8,
    registers: [u8; I2C_REG_BYTES],
}

/// Bytes of a simulated device's register file (64 five-byte INA228 registers)
const I2C_REG_BYTES: usize = 320;

/// The INA228s: five bytes a register
fn word_device(addr: u8) -> bool {
    addr == I2C_ADDR_BATTERY_MONITOR || addr == I2C_ADDR_SOLAR_MONITOR
}

/// Where register `reg` of the device at `addr` starts
fn reg_offset(addr: u8, reg: u8) -> usize {
    if word_device(addr) { usize::from(reg) * 5 } else { usize::from(reg) }
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
    /// The internal pull each pin was configured with (`GPIO_PULL_*`)
    gpio_pull: [u32; GPIO_PIN_COUNT],
    /// Levels the outside world drives (`hal_sim_gpio_drive_input()`): they
    /// survive `hal_gpio_init()`, as a switch's level survives a reset
    gpio_driven: [Option<bool>; GPIO_PIN_COUNT],
    /// The `XPDR_EN` heartbeat (`hal_gpio_heartbeat()`): when the wave
    /// started and when it stops or stopped, on the HAL time base, µs
    /// (`None`: it has not run since `hal_gpio_init()`)
    heartbeat: Option<(u64, u64)>,
    i2c_initialized: [bool; I2C_BUS_COUNT],
    i2c: [I2cDevice; 4],
    uart_initialized: [bool; UART_PORT_COUNT],
    uart_rx: [Fifo<UART_RX_SIZE>; UART_PORT_COUNT],
    spi_initialized: [bool; SPI_BUS_COUNT],
    adc_initialized: bool,
    adc_values: [f64; ADC_CHANNEL_COUNT],
    /// Channels set by `hal_sim_adc_set()` since `hal_adc_init()`
    adc_set: [bool; ADC_CHANNEL_COUNT],
    /// The last coil burst (`hal_mtq.h`): duties and when it ends, µs
    mtq: ([u16; 3], u64),
    /// Captures (`hal_capture.h`): periods, µs (`None`: no edges), and the
    /// exciter's keyed time: accumulated ms, and since when it is high, µs
    capture_period_us: [Option<u32>; 2],
    ptt_high_ms: u64,
    ptt_since_us: Option<u64>,
    flash: Flash,
    radio: Radio,
    radio_ready: bool,
    radio_tx: Radio,
    radio_tx_result: i32,
    tnc_fd: i32,
    wdt: Wdt,
    reset_cause: u32,
    /// The EPS controller's end-of-life logic (SQSO-EPS-BRD-001 D §7)
    eol: EolSim,
}

/// The simulated EPS controller's end-of-life logic: the keyed EOL_KEY
/// (register 0x7E, 0x3C then 0xC3 within 1 s, accepted only with the latch
/// clear and F1 intact, high for 10 s: `EOL_STATE` bit 3), the arm latch
/// (`EOL_STATE` bit 0) clocked by an `EOL_ARM` rising edge while the key is
/// high, and the fire: with the latch set and `EOL_FIRE` high for
/// `open_ms`, the charge path opens (`EOL_STATE` bit 1 set, bit 2 F1 intact
/// clear). `open_ms` 0: never (the mission simulation models the heater
/// itself, `hal_sim_eps_eol_open_ms()`)
#[derive(Clone, Copy)]
struct EolSim {
    key_stage_ms: Option<u64>,
    key_until_ms: Option<u64>,
    fire_since_ms: Option<u64>,
    open_ms: u64,
}

impl EolSim {
    const RESET: EolSim = EolSim { key_stage_ms: None, key_until_ms: None, fire_since_ms: None, open_ms: 30_000 };
}

/// `EOL_STATE` and `EOL_KEY` of the EPS controller
const EOL_STATE_REG: usize = 0x7C;
const EOL_KEY_REG: u8 = 0x7E;

static SIM: Global<Sim> = Global::new(Sim {
    gpio_initialized: false,
    gpio_state: [false; GPIO_PIN_COUNT],
    gpio_dir: [GPIO_DIR_INPUT; GPIO_PIN_COUNT],
    gpio_pull: [GPIO_PULL_NONE; GPIO_PIN_COUNT],
    gpio_driven: [None; GPIO_PIN_COUNT],
    heartbeat: None,
    i2c_initialized: [false; I2C_BUS_COUNT],
    i2c: [
        I2cDevice { addr: I2C_ADDR_EPS_CONTROLLER, registers: [0; I2C_REG_BYTES] },
        I2cDevice { addr: I2C_ADDR_BATTERY_MONITOR, registers: [0; I2C_REG_BYTES] },
        I2cDevice { addr: I2C_ADDR_SOLAR_MONITOR, registers: [0; I2C_REG_BYTES] },
        I2cDevice { addr: I2C_ADDR_ANTS, registers: [0; I2C_REG_BYTES] },
    ],
    uart_initialized: [false; UART_PORT_COUNT],
    uart_rx: [Fifo::EMPTY, Fifo::EMPTY, Fifo::EMPTY],
    spi_initialized: [false; SPI_BUS_COUNT],
    adc_initialized: false,
    adc_values: ADC_VALUES,
    adc_set: [false; ADC_CHANNEL_COUNT],
    mtq: ([0; 3], 0),
    capture_period_us: [None; 2],
    ptt_high_ms: 0,
    ptt_since_us: None,
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
    eol: EolSim::RESET,
});

/// Run `f` on the simulation state; `busy` if it is in use (a HAL call
/// from an interrupt-like context in the middle of another)
/// Run `f` while the simulator is in use, so every HAL call it makes is
/// refused as busy (unit tests of a caller's failure path); `None` if the
/// simulator was already in use
#[cfg(test)]
pub(crate) fn holding_the_sim<R>(f: impl FnOnce() -> R) -> Option<R> {
    SIM.with(|_| f())
}

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
        s.gpio_pull = [GPIO_PULL_NONE; GPIO_PIN_COUNT];
        s.heartbeat = None;
        s.gpio_initialized = true;
        rc::OK
    })
}

/// Set a pin's direction and internal pull. The pull is recorded
/// (`hal_sim_gpio_pull()`), and on the two inputs the board drives through
/// a weak source it decides the level, as on the board: DEPLOY_SENSE (1 MΩ)
/// and SOLAR_SENSE (11 kΩ pull-up behind 10 kΩ) read the internal pull's
/// level whatever the outside world drives (RID SMA-B02)
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_config(pin: u32, direction: u32, pull: u32) -> i32 {
    sim(rc::BUSY, |s| {
        if !s.gpio_initialized {
            return rc::ERROR;
        }
        let i = index(pin, GPIO_PIN_COUNT).unwrap_or(usize::MAX);
        match (s.gpio_dir.get_mut(i), s.gpio_pull.get_mut(i)) {
            (Some(dir), Some(p)) => {
                *dir = direction;
                *p = pull;
                rc::OK
            }
            _ => rc::INVALID,
        }
    })
}

/// The internal pull a pin was last configured with (`GPIO_PULL_*`; an
/// unknown pin reads `GPIO_PULL_NONE`)
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_gpio_pull(pin: u32) -> u32 {
    sim(GPIO_PULL_NONE, |s| {
        index(pin, GPIO_PIN_COUNT).and_then(|i| s.gpio_pull.get(i).copied()).unwrap_or(GPIO_PULL_NONE)
    })
}

/// An input the board drives through a weak source: the internal pull's
/// level if one is configured
fn weak_input(s: &Sim, pin: u32, i: usize) -> Option<bool> {
    let weak = matches!(pin, crate::separation_switch::GPIO_PIN_DEPLOY_SENSE | crate::deployment::GPIO_PIN_SOLAR_SENSE);
    match s.gpio_pull.get(i) {
        Some(&GPIO_PULL_UP) if weak => Some(true),
        Some(&GPIO_PULL_DOWN) if weak => Some(false),
        _ => None,
    }
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
        let was = s.gpio_state.get(i).copied().unwrap_or(false);
        if let Some(level) = s.gpio_state.get_mut(i) {
            *level = state;
        }
        eol_line(s, pin, was, state, time_us() / 1000);
        rc::OK
    })
}

/// A pin's level (false before init or for an unknown pin); `XPDR_EN`
/// while its heartbeat runs is the square wave's level now
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_get(pin: u32) -> bool {
    let now = hal_timer_get_us();
    sim(false, |s| {
        if let (GPIO_PIN_XPDR_EN, Some((start, end))) = (pin, s.heartbeat)
            && s.gpio_initialized
            && now < end
        {
            return (now.saturating_sub(start) / HEARTBEAT_HALF_PERIOD_US).is_multiple_of(2);
        }
        let Some(i) = index(pin, GPIO_PIN_COUNT).filter(|_| s.gpio_initialized) else { return false };
        if let Some(level) = weak_input(s, pin, i) {
            return level;
        }
        if s.gpio_driven.get(i) == Some(&None)
            && let Some(level) = derived_input(s, pin)
        {
            return level;
        }
        s.gpio_state.get(i).copied().unwrap_or(false)
    })
}

/// Run the `XPDR_EN` heartbeat for `lease_ms` from now (a running wave's
/// lease is restarted), or stop it now (0)
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_heartbeat(pin: u32, lease_ms: u32) -> i32 {
    let now = hal_timer_get_us();
    sim(rc::BUSY, |s| {
        if !s.gpio_initialized {
            return rc::ERROR;
        }
        if pin != GPIO_PIN_XPDR_EN {
            return rc::INVALID;
        }
        let running = s.heartbeat.filter(|&(_, end)| now < end);
        s.heartbeat = match (running, lease_ms) {
            (Some((start, _)), 0) => Some((start, now)),
            (None, 0) => s.heartbeat,
            (Some((start, _)), lease) => Some((start, now + u64::from(lease) * 1000)),
            (None, lease) => Some((now, now + u64::from(lease) * 1000)),
        };
        rc::OK
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
        s.eol = EolSim { open_ms: s.eol.open_ms, ..EolSim::RESET };
        for dev in &mut s.i2c {
            dev.registers = [0; I2C_REG_BYTES];
            if word_device(dev.addr) {
                use sqso_fsw::ina228::{DEVICE_ID, MANUFACTURER_ID, reg};
                let mut put = |r: u8, v: u16| {
                    let at = reg_offset(dev.addr, r);
                    if let Some(w) = dev.registers.get_mut(at..at + 2) {
                        w.copy_from_slice(&v.to_be_bytes());
                    }
                };
                put(reg::MANUFACTURER_ID, MANUFACTURER_ID);
                put(reg::DEVICE_ID, (DEVICE_ID << 4) | 1);
            }
            if dev.addr == I2C_ADDR_EPS_CONTROLLER {
                dev.registers[EOL_STATE_REG] = sqso_fsw::passivation::eol_state::FUSE_INTACT;
                dev.registers[0x00] = 0x43; // STATUS: charging, sunlit
                dev.registers[0x01] = 0x01; // MODE: IDLE
                dev.registers[0x16] = 75; // SOC: 75 %
            }
            if dev.addr == I2C_ADDR_ANTS {
                // Every element stowed, not armed
                let stowed = sqso_fsw::ants::Status::decode([0x88, 0x88]).encode();
                let at = usize::from(sqso_fsw::ants::CMD_STATUS);
                if let Some(r) = dev.registers.get_mut(at..at + 2) {
                    r.copy_from_slice(&stowed);
                }
            }
        }
        if let Some(init) = s.i2c_initialized.get_mut(bus) {
            *init = true;
        }
        rc::OK
    })
}

/// The simulated AntS's command latches: a command without arguments sets
/// its register to 1; arm and disarm clear each other, and so do a
/// deployment (its time limit in 0xA5) and cancel
fn ants_command(registers: &mut [u8; I2C_REG_BYTES], reg: u8, no_args: bool) {
    use sqso_fsw::ants::{CMD_ARM, CMD_AUTO_DEPLOY, CMD_CANCEL, CMD_DISARM};
    let mut set = |r: u8, v: u8| {
        if let Some(slot) = registers.get_mut(usize::from(r)) {
            *slot = v;
        }
    };
    if no_args {
        set(reg, 1);
    }
    match reg {
        CMD_ARM => set(CMD_DISARM, 0),
        CMD_DISARM => set(CMD_ARM, 0),
        CMD_CANCEL => set(CMD_AUTO_DEPLOY, 0),
        CMD_AUTO_DEPLOY => set(CMD_CANCEL, 0),
        _ => {}
    }
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
        if I2C_FAIL_REG.load(core::sync::atomic::Ordering::Relaxed) == (u32::from(addr) << 8 | u32::from(*reg)) {
            return rc::ERROR;
        }
        if addr == I2C_ADDR_EPS_CONTROLLER && *reg == EOL_KEY_REG {
            let now = time_us() / 1000;
            for &v in values {
                eol_key_write(s, v, now);
            }
        }
        let Some(dev) = device(s, addr) else { return rc::ERROR };
        if dev.addr == I2C_ADDR_ANTS {
            ants_command(&mut dev.registers, *reg, values.is_empty());
        }
        let start = reg_offset(dev.addr, *reg);
        let mut at = start;
        for &v in values {
            if let Some(r) = dev.registers.get_mut(at) {
                *r = v;
            }
            // Byte registers wrap at 0xFF, as before
            at = if word_device(dev.addr) { at + 1 } else { (at + 1) & 0xFF };
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

/// One register that does not answer (`addr << 8 | reg`; 0: none): the
/// unit tests' failed access to a single register
pub(crate) static I2C_FAIL_REG: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

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
        if I2C_FAIL_REG.load(core::sync::atomic::Ordering::Relaxed) == (u32::from(addr) << 8 | u32::from(*reg)) {
            return rc::ERROR;
        }
        if addr == I2C_ADDR_EPS_CONTROLLER {
            eol_update(s, time_us() / 1000);
        }
        let Some(dev) = device(s, addr) else { return rc::ERROR };
        let Some(read) = read else { return rc::NULL_PTR };
        let start = reg_offset(dev.addr, *reg);
        match dev.registers.get(start..start + read.len()) {
            Some(regs) => {
                read.copy_from_slice(regs);
                rc::OK
            }
            None => rc::INVALID,
        }
    })
}

/// The EPS controller's `EOL_STATE` byte
fn eol_state_mut(s: &mut Sim) -> Option<&mut u8> {
    device(s, I2C_ADDR_EPS_CONTROLLER).and_then(|d| d.registers.get_mut(EOL_STATE_REG))
}

/// One byte written to `EOL_KEY`: the first of the pair, the second (the
/// key raised if the latch is clear and F1 intact), or anything else (the
/// key dropped)
fn eol_key_write(s: &mut Sim, v: u8, now_ms: u64) {
    use sqso_fsw::passivation::{KEY, eol_state};
    let state = eol_state_mut(s).map_or(0, |b| *b);
    let ready = state & eol_state::ARMED == 0 && state & eol_state::FUSE_INTACT != 0;
    let staged = s.eol.key_stage_ms.is_some_and(|t| now_ms.saturating_sub(t) <= 1000);
    s.eol.key_stage_ms = None;
    s.eol.key_until_ms = None;
    if v == KEY[0] {
        s.eol.key_stage_ms = Some(now_ms);
    } else if v == KEY[1] && staged && ready {
        s.eol.key_until_ms = Some(now_ms + 10_000);
    }
    eol_update(s, now_ms);
}

/// `EOL_STATE` now: the key's bit, and the charge path opened by a fire
/// held long enough with the latch set
fn eol_update(s: &mut Sim, now_ms: u64) {
    use sqso_fsw::passivation::eol_state;
    let key = s.eol.key_until_ms.is_some_and(|t| now_ms < t);
    let opened = s.eol.open_ms > 0 && s.eol.fire_since_ms.is_some_and(|t| now_ms.saturating_sub(t) >= s.eol.open_ms);
    if let Some(b) = eol_state_mut(s) {
        *b = if key { *b | eol_state::KEY } else { *b & !eol_state::KEY };
        if opened && *b & eol_state::ARMED != 0 {
            *b = (*b | eol_state::FIRED) & !eol_state::FUSE_INTACT;
        }
    }
}

/// An end-of-life line driven: `EOL_ARM`'s rising edge sets the latch while
/// the key is high; `EOL_FIRE` high with the latch set starts the fire
fn eol_line(s: &mut Sim, pin: u32, was: bool, level: bool, now_ms: u64) {
    use sqso_fsw::passivation::eol_state;
    eol_update(s, now_ms);
    let state = eol_state_mut(s).map_or(0, |b| *b);
    if pin == crate::passivation::GPIO_PIN_EOL_ARM
        && level
        && !was
        && state & eol_state::KEY != 0
        && let Some(b) = eol_state_mut(s)
    {
        *b |= eol_state::ARMED;
    }
    if pin == crate::passivation::GPIO_PIN_EOL_FIRE {
        s.eol.fire_since_ms =
            if level && state & eol_state::ARMED != 0 { Some(s.eol.fire_since_ms.unwrap_or(now_ms)) } else { None };
    }
}

/// How long `EOL_FIRE` must be held, with the latch set, before the
/// simulated EPS reports the charge path open, ms (0: never; the mission
/// simulation then sets `EOL_STATE` from its own heater model)
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_eps_eol_open_ms(ms: u32) {
    let _ = SIM.with(|s| s.eol.open_ms = u64::from(ms));
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
    time_us() / 1000
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
        s.adc_set = [false; ADC_CHANNEL_COUNT];
        rc::OK
    })
}

fn adc_value(channel: u32) -> Result<f64, i32> {
    sim(Err(rc::BUSY), |s| {
        if !s.adc_initialized {
            return Err(rc::ERROR);
        }
        let c = index(channel, ADC_CHANNEL_COUNT).ok_or(rc::INVALID)?;
        if s.adc_set.get(c) == Some(&false)
            && let Some(v) = readback(s, channel)
        {
            return Ok(v);
        }
        s.adc_values.get(c).copied().ok_or(rc::INVALID)
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

/// A readback ADC channel follows its switch, as the hardware's divider
/// would: the switch output at the battery voltage when its enables are
/// on, 0 V off (`None`: not a readback channel)
fn readback(s: &Sim, channel: u32) -> Option<f64> {
    use crate::pins::{ADC_VBATT, ADC_VJET_RB, ADC_VSW1_RB, JETSON_PWR, PAYLOAD_EN};
    let out = |pin: u32| index(pin, GPIO_PIN_COUNT).and_then(|i| s.gpio_state.get(i).copied()).unwrap_or(false);
    let vbat = index(ADC_VBATT, ADC_CHANNEL_COUNT).and_then(|c| s.adc_values.get(c).copied()).unwrap_or(0.0);
    match channel {
        ADC_VSW1_RB => Some(if out(PAYLOAD_EN) { vbat } else { 0.0 }),
        ADC_VJET_RB => Some(if out(PAYLOAD_EN) && out(JETSON_PWR) { vbat } else { 0.0 }),
        _ => None,
    }
}

/// An input pin nobody drives that follows the outputs, as the board's
/// readbacks do with nothing failed (`None`: an ordinary pin): the Jetson
/// carrier's power-good with both of its switches on, each latch-up
/// switch's EN node and the GNSS rail's power-good with their enables, the
/// bridges' fault line high (no fault); DEPLOY_SENSE high (the switch
/// released: the flight state) and SOLAR_SENSE high (the wings stowed)
fn derived_input(s: &Sim, pin: u32) -> Option<bool> {
    use crate::pins::{
        GNSS_EN, GNSS_PG, GYRO_EN, GYRO_RB, IMU2_EN, IMU2_RB, JET_PG, JETSON_PWR, MTQ_FAULT_N, PAYLOAD_EN, SNS_EN,
        SNS_RB,
    };
    let out = |p: u32| index(p, GPIO_PIN_COUNT).and_then(|i| s.gpio_state.get(i).copied()).unwrap_or(false);
    match pin {
        JET_PG => Some(out(PAYLOAD_EN) && out(JETSON_PWR)),
        GYRO_RB => Some(out(GYRO_EN)),
        IMU2_RB => Some(out(IMU2_EN)),
        SNS_RB => Some(out(SNS_EN)),
        GNSS_PG => Some(out(GNSS_EN)),
        MTQ_FAULT_N => Some(true),
        crate::separation_switch::GPIO_PIN_DEPLOY_SENSE | crate::deployment::GPIO_PIN_SOLAR_SENSE => Some(true),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// hal_mtq.h

/// Start a coil burst (kept for tests: `hal_sim_mtq_last()`)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_mtq_burst(duty_permille: *const u16, on_ms: u32) -> i32 {
    if duty_permille.is_null() {
        return rc::NULL_PTR;
    }
    // SAFETY: the C contract: three uint16_t
    let duty = unsafe { [duty_permille.read(), duty_permille.add(1).read(), duty_permille.add(2).read()] };
    if duty.iter().any(|&d| d > 1000) || on_ms == 0 || on_ms > 900 {
        return rc::INVALID;
    }
    let now = hal_timer_get_us();
    sim(rc::BUSY, |s| {
        s.mtq = (duty, now + u64::from(on_ms) * 1000);
        rc::OK
    })
}

/// End any burst now
#[unsafe(no_mangle)]
pub extern "C" fn hal_mtq_off() -> i32 {
    let now = hal_timer_get_us();
    sim(rc::BUSY, |s| {
        s.mtq.1 = s.mtq.1.min(now);
        rc::OK
    })
}

/// Whether a burst runs now
#[unsafe(no_mangle)]
pub extern "C" fn hal_mtq_active() -> bool {
    let now = hal_timer_get_us();
    sim(false, |s| now < s.mtq.1)
}

/// The latest burst's duties, ‰ (tests)
pub fn hal_sim_mtq_last() -> [u16; 3] {
    sim([0; 3], |s| s.mtq.0)
}

// ---------------------------------------------------------------------------
// hal_capture.h

/// The latest period of the 1PPS or the RTC's 1 Hz, as set by the
/// simulation (`hal_sim_capture_set_period()`)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_capture_period_us(input: u32, period_us: *mut u32) -> i32 {
    // SAFETY: the C contract: NULL or a uint32_t
    let Some(out) = (unsafe { ptr::out(period_us) }) else { return rc::NULL_PTR };
    let Some(i) = index(input, 2) else { return rc::INVALID };
    match sim(None, |s| s.capture_period_us.get(i).copied().flatten()) {
        Some(p) => {
            *out = p;
            rc::OK
        }
        None => rc::TIMEOUT,
    }
}

/// The exciter's keyed time since boot, ms: the time `GPIO_PIN_PTT_EXC`
/// was driven high (`hal_sim_gpio_drive_input()`)
#[unsafe(no_mangle)]
pub extern "C" fn hal_capture_high_ms(input: u32) -> u32 {
    if input != crate::pins::CAPTURE_PTT_EXC {
        return 0;
    }
    let now = hal_timer_get_us();
    let ms = sim(0, |s| s.ptt_high_ms + s.ptt_since_us.map_or(0, |t| now.saturating_sub(t) / 1000));
    u32::try_from(ms).unwrap_or(u32::MAX)
}

/// Set the period a captured input shows (`None`: it stopped)
pub fn hal_sim_capture_set_period(input: u32, period_us: Option<u32>) {
    sim((), |s| {
        if let Some(p) = index(input, 2).and_then(|i| s.capture_period_us.get_mut(i)) {
            *p = period_us;
        }
    });
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

/// When the `XPDR_EN` heartbeat stops, or stopped, on the HAL time base
/// (`hal_timer_get_us()`), µs; 0 if it has not run since `hal_gpio_init()`
/// or for another pin. The radio board's monostable holds the transponder
/// enabled until 50 ms after this
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_gpio_heartbeat_end_us(pin: u32) -> u64 {
    sim(0, |s| match (pin, s.heartbeat) {
        (GPIO_PIN_XPDR_EN, Some((_, end))) => end,
        _ => 0,
    })
}

/// Set an input pin's level, as the outside world would (it stays at that
/// level across `hal_gpio_init()`)
#[unsafe(no_mangle)]
pub extern "C" fn hal_sim_gpio_drive_input(pin: u32, state: bool) {
    let now = hal_timer_get_us();
    sim((), |s| {
        if pin == crate::pins::PTT_EXC {
            match (state, s.ptt_since_us) {
                (true, None) => s.ptt_since_us = Some(now),
                (false, Some(t)) => {
                    s.ptt_high_ms += now.saturating_sub(t) / 1000;
                    s.ptt_since_us = None;
                }
                _ => {}
            }
        }
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
            if let Some(set) = s.adc_set.get_mut(usize::try_from(channel).unwrap_or(usize::MAX)) {
                *set = true;
            }
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
