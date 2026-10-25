//! SMART-QSO OBC firmware for the STM32L4R5ZI.
//!
//! Embassy brings the chip up (clocks, the time driver, interrupt-driven
//! UARTs) and this crate implements the flight software's HAL (`hal/*.h`)
//! on it, so the flight software (`sqso-ffi`) runs unchanged on the host
//! simulation and here. The main loop is the same as the simulation's: one
//! pass every 20 ms, with the core asleep in the executor in between, and
//! the hardware watchdog refreshed only when every stage of a pass checked
//! in. Each pass is timed for the probe (timing.rs, SQSO_LOOP_TIMING).
//!
//! No hardware has run this yet: CI builds and checks the image. The pin
//! assignment is provisional (gpio.rs).
//!
//! @requirement SRS-F001 Initialize all subsystems within 30 seconds of power-on
//! @requirement SRS-P001 Main control loop executes within 20 ms

#![no_std]
#![no_main]

mod adc;
mod cell;
mod fram;
mod gpio;
mod i2c;
mod radio;
mod rc;
mod spi;
mod stack;
mod timer;
mod timing;
mod uart;
mod wdt;

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::Spawner;
use embassy_stm32::adc::Adc;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::spi::Spi;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{self, BufferedUart};
use embassy_stm32::{Config, bind_interrupts, peripherals, rcc};
use embassy_time::Timer;
use sqso_ffi::executive::Executive;
use static_cell::ConstStaticCell;

bind_interrupts!(struct Irqs {
    USART1 => usart::BufferedInterruptHandler<peripherals::USART1>;
    USART2 => usart::BufferedInterruptHandler<peripherals::USART2>;
    USART3 => usart::BufferedInterruptHandler<peripherals::USART3>;
});

/// Main-loop period (the simulation's MAIN_LOOP_SLEEP_NS)
const TICK_MS: u64 = 20;

/// 80 MHz, the top of voltage range 1 normal mode (declocked from 120 MHz
/// for power, as in hardware/obc/OBC_DESIGN.md): the 4 MHz MSI through the
/// PLL, M = 1, N = 40, R = 2; AHB and APB at the core clock
fn clocks() -> Config {
    let mut config = Config::default();
    config.rcc.msi = Some(rcc::MSIRange::RANGE4M);
    config.rcc.pll = Some(rcc::Pll {
        source: rcc::PllSource::MSI,
        prediv: rcc::PllPreDiv::DIV1,
        mul: rcc::PllMul::MUL40,
        divp: None,
        divq: None,
        divr: Some(rcc::PllRDiv::DIV2),
    });
    config.rcc.sys = rcc::Sysclk::PLL1_R;
    config
}

fn uart(baud: u32) -> usart::Config {
    let mut c = usart::Config::default();
    c.baudrate = baud;
    c
}

/// Receive and transmit buffers for the three UARTs
static JETSON_RX: ConstStaticCell<[u8; 512]> = ConstStaticCell::new([0; 512]);
static JETSON_TX: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0; 256]);
static DEBUG_RX: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0; 64]);
static DEBUG_TX: ConstStaticCell<[u8; 512]> = ConstStaticCell::new([0; 512]);
static RADIO_RX: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0; 1024]);
static RADIO_TX: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);

/// Stop and let the hardware watchdog reset the OBC (it was started by the
/// flight software, or the boot failed before it could be: then the
/// system reset below restarts it)
fn halt() -> ! {
    cortex_m::peripheral::SCB::sys_reset()
}

/// Set by the first panic
static PANICKING: AtomicBool = AtomicBool::new(false);

/// A Rust panic is a fatal assertion: reported to the assertion handler
/// (which logs it and saves state), then the processor waits for the
/// independent watchdog to reset it. If the watchdog was never started
/// (a panic during bring-up), it resets itself instead.
///
/// A panic while one is being reported (in the fault log or a callback) is
/// not reported again, so the handler nests at most once; the stack
/// analysis (`scripts/stack_usage.py`) counts that one nesting.
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    if !PANICKING.swap(true, Ordering::Relaxed) {
        sqso_ffi::report_panic(info);
    }
    if wdt::running() {
        loop {
            cortex_m::asm::wfi();
        }
    }
    halt()
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    stack::protect();
    wdt::start_boot_watchdog();
    timing::start();
    bring_up();
    let flight = FLIGHT.take();
    if !flight.start() {
        halt()
    }
    loop {
        let pass = timing::now();
        flight.step();
        timing::record(pass);
        Timer::after_millis(TICK_MS).await;
    }
}

/// The flight software's state, for the whole mission. Started in place:
/// the async main task keeps only a reference, not the state's 800 bytes
/// in its frame (SRS-C003)
static FLIGHT: ConstStaticCell<Executive> = ConstStaticCell::new(Executive::BOOT);

/// Clocks, pins and the HAL's peripherals. Not inlined: the async main
/// task's frame would carry this one's temporaries for the whole mission
/// (SRS-C003, `scripts/stack_usage.py`)
#[inline(never)]
fn bring_up() {
    let p = embassy_stm32::init(clocks());

    // Enable and burn lines low first
    gpio::install(
        p.PB4, p.PB5, p.PB6, p.PD2, p.PA1, p.PD3, p.PA2, p.PA3, p.PB7, p.PA0, p.PB8, p.PB9, p.PB10, p.PB11, p.PB12,
    );

    // FRAM: SPI mode 0 at 20 MHz (STORAGE_DESIGN §3.4); WP# and HOLD# high
    let mut fram_cfg = embassy_stm32::spi::Config::default();
    fram_cfg.frequency = Hertz(20_000_000);
    fram::install(
        Spi::new_blocking(p.SPI1, p.PA5, p.PA7, p.PA6, fram_cfg),
        Output::new(p.PA4, Level::High, Speed::VeryHigh),
        Output::new(p.PC0, Level::High, Speed::Low),
        Output::new(p.PC1, Level::High, Speed::Low),
    );

    let spi_cfg = embassy_stm32::spi::Config::default();
    spi::install(
        (Spi::new_blocking(p.SPI2, p.PB13, p.PB15, p.PB14, spi_cfg), Output::new(p.PD0, Level::High, Speed::High)),
        (Spi::new_blocking(p.SPI3, p.PC10, p.PC12, p.PC11, spi_cfg), Output::new(p.PD1, Level::High, Speed::High)),
    );

    let i2c_cfg = embassy_stm32::i2c::Config::default();
    i2c::install(I2c::new_blocking(p.I2C2, p.PF1, p.PF0, i2c_cfg), I2c::new_blocking(p.I2C4, p.PD12, p.PD13, i2c_cfg));

    let (Ok(jetson), Ok(debug), Ok(tnc)) = (
        BufferedUart::new(p.USART1, p.PA10, p.PA9, JETSON_TX.take(), JETSON_RX.take(), Irqs, uart(115_200)),
        BufferedUart::new(p.USART3, p.PD9, p.PD8, DEBUG_TX.take(), DEBUG_RX.take(), Irqs, uart(115_200)),
        BufferedUart::new(p.USART2, p.PD6, p.PD5, RADIO_TX.take(), RADIO_RX.take(), Irqs, uart(9600)),
    ) else {
        halt()
    };
    uart::install(jetson, debug);
    radio::install(tnc);

    adc::install(
        Adc::new(p.ADC1),
        [
            adc::input(p.PC2),
            adc::input(p.PC3),
            adc::input(p.PC4),
            adc::input(p.PC5),
            adc::input(p.PB0),
            adc::input(p.PB1),
        ],
    );
}
