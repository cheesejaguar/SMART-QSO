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
//! No hardware has run this yet: CI builds and checks the image. The pins
//! are the master allocation SQSO-ICX-001 B §4.1 (gpio.rs).
//!
//! @requirement SRS-F001 Initialize all subsystems within 30 seconds of power-on
//! @requirement SRS-P001 Main control loop executes within 20 ms

#![no_std]
#![no_main]

mod adc;
mod capture;
mod cell;
mod fram;
mod gpio;
mod heartbeat;
mod i2c;
mod mtq;
mod parity;
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
    LPUART1 => usart::BufferedInterruptHandler<peripherals::LPUART1>;
});

/// Main-loop period (the simulation's MAIN_LOOP_SLEEP_NS)
const TICK_MS: u64 = 20;

/// 80 MHz, the top of voltage range 1 normal mode (declocked from 120 MHz
/// for power, as in hardware/obc/OBC_DESIGN.md): the 4 MHz MSI through the
/// PLL, M = 1, N = 40, R = 2; AHB and APB at the core clock
fn clocks() -> Config {
    let mut config = Config::default();
    // VDDIO2 (PWR_CR2.IOSV) for port G (PG2-PG15: the FRAM, LPUART1, JET_PG,
    // the RTC's I2C1, WDI; SQSO-ICX-001 B §4.2)
    config.enable_independent_io_supply = true;
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

/// The Jetson's line: 8N2 (the module's receiver needs two stop bits,
/// SMART-QSO-ICD-002 1.3 §10.2; RID JETB-03)
fn uart_jetson() -> usart::Config {
    let mut c = uart(115_200);
    c.stop_bits = usart::StopBits::STOP2;
    c
}

/// Receive and transmit buffers for the three UARTs
static JETSON_RX: ConstStaticCell<[u8; 512]> = ConstStaticCell::new([0; 512]);
static JETSON_TX: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0; 256]);
static DEBUG_RX: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0; 64]);
static DEBUG_TX: ConstStaticCell<[u8; 512]> = ConstStaticCell::new([0; 512]);
static RADIO_RX: ConstStaticCell<[u8; 1024]> = ConstStaticCell::new([0; 1024]);
static GNSS_RX: ConstStaticCell<[u8; 256]> = ConstStaticCell::new([0; 256]);
static GNSS_TX: ConstStaticCell<[u8; 64]> = ConstStaticCell::new([0; 64]);
/// Holds the longest downlink stream, so `hal_radio_transmit()` never waits
const RADIO_TX_LEN: usize = 1024;
const _: () = assert!(RADIO_TX_LEN >= radio::STREAM_MAX);
static RADIO_TX: ConstStaticCell<[u8; RADIO_TX_LEN]> = ConstStaticCell::new([0; RADIO_TX_LEN]);

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
    parity::report();
    loop {
        let pass = timing::now();
        flight.step();
        capture::poll();
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

    // Every pin of SQSO-ICX-001 B §4.1, outputs low first (gpio.rs)
    gpio::install_board(
        p.PB2, p.PB5, p.PB6, p.PD2, p.PF3, p.PD3, p.PA8, p.PA3, p.PB7, p.PE2, p.PB8, p.PB9, p.PB10, p.PB12, p.PE3,
        p.PE4, p.PE5, p.PE6, p.PF4, p.PG1, p.PG0, p.PF13, p.PE7, p.PF5, p.PF6, p.PF7, p.PF8, p.PF10, p.PF11, p.PF12,
        p.PD14, p.PD10, p.PD15, p.PD11, p.PE8, p.PE10, p.PE12, p.PE14, p.PE15, p.PC9, p.PG12, p.PG6, p.PG15, p.PG11,
    );
    // The XPDR_EN heartbeat's timer (stopped, pin low, until the first lease)
    heartbeat::install();
    // The magnetorquers (TIM1 PWM, TIM15 one-pulse; ARM low) and the
    // captures (TIM3: 1PPS and the RTC's 1 Hz; TIM8: PTT_EXCITER)
    mtq::install(p.TIM1, p.TIM15, (p.PE9, p.PE11, p.PE13, p.PF9));
    capture::install(p.TIM3, p.TIM8, (p.PC6, p.PC7, p.PC8));

    // FRAM: SPI1 mode 0 at 20 MHz on PG2 SCK / PG3 MISO / PG4 MOSI, CS# PG5
    // (SQSO-ICX-001 B §6 row 3) and the mirror U36 on CS# PD7 (D §6 row 32);
    // WP# and HOLD# high on PE0/PE1
    let mut fram_cfg = embassy_stm32::spi::Config::default();
    fram_cfg.frequency = Hertz(20_000_000);
    fram::install(
        Spi::new_blocking(p.SPI1, p.PG2, p.PG4, p.PG3, fram_cfg),
        [Output::new(p.PG5, Level::High, Speed::VeryHigh), Output::new(p.PD7, Level::High, Speed::VeryHigh)],
        Output::new(p.PE0, Level::High, Speed::Low),
        Output::new(p.PE1, Level::High, Speed::Low),
    );

    // The gyros: SPI2 for the BMI088 (gyro CS PD0, accelerometer CS PD4),
    // SPI3 for the M-G370 (CS PD1); configured by hal_spi_init()
    let spi_cfg = embassy_stm32::spi::Config::default();
    spi::install(
        Spi::new_blocking(p.SPI2, p.PB13, p.PB15, p.PB14, spi_cfg),
        [Output::new(p.PD0, Level::High, Speed::High), Output::new(p.PD4, Level::High, Speed::High)],
        Spi::new_blocking(p.SPI3, p.PC10, p.PC12, p.PC11, spi_cfg),
        Output::new(p.PD1, Level::High, Speed::High),
    );

    let i2c_cfg = embassy_stm32::i2c::Config::default();
    i2c::install(
        I2c::new_blocking(p.I2C2, p.PF1, p.PF0, i2c_cfg),
        I2c::new_blocking(p.I2C4, p.PD12, p.PD13, i2c_cfg),
        I2c::new_blocking(p.I2C1, p.PG14, p.PG13, i2c_cfg),
        I2c::new_blocking(p.I2C3, p.PG7, p.PB4, i2c_cfg),
    );

    // The Jetson (USART1, 8N2), the console (USART3), the TNC (USART2) and
    // the GNSS receiver's NMEA (LPUART1 PG8 RX / PB11 TX, AF8, 9600 baud:
    // PG7 is I2C3's SCL, SQSO-ICX-001 D §6 row 29)
    let (Ok(jetson), Ok(debug), Ok(tnc), Ok(gnss)) = (
        BufferedUart::new(p.USART1, p.PA10, p.PA9, JETSON_TX.take(), JETSON_RX.take(), Irqs, uart_jetson()),
        BufferedUart::new(p.USART3, p.PD9, p.PD8, DEBUG_TX.take(), DEBUG_RX.take(), Irqs, uart(115_200)),
        BufferedUart::new(p.USART2, p.PD6, p.PD5, RADIO_TX.take(), RADIO_RX.take(), Irqs, uart(9600)),
        BufferedUart::new(p.LPUART1, p.PG8, p.PB11, GNSS_TX.take(), GNSS_RX.take(), Irqs, uart(9600)),
    ) else {
        halt()
    };
    uart::install(jetson, debug, gnss);
    radio::install(tnc);

    // ADC1 in HalAdcChannel_t order (TEMP_MCU, index 5, is internal)
    adc::install(
        Adc::new(p.ADC1),
        [
            adc::input(p.PC2), // VBATT
            adc::input(p.PC3), // IBATT
            adc::input(p.PC4), // VSOLAR
            adc::input(p.PC5), // ISOLAR
            adc::input(p.PB0), // VBUS
            adc::input(p.PB1), // TEMP_BOARD
            adc::input(p.PA0), // TX_TEMP
            adc::input(p.PC0), // TX_FWD
            adc::input(p.PC1), // TX_REV
            adc::input(p.PA1), // VSW1_RB
            adc::input(p.PA2), // PT_NTC
            adc::input(p.PA4), // MTQ_X_I
            adc::input(p.PA5), // MTQ_Y_I
            adc::input(p.PA6), // MTQ_Z_I
            adc::input(p.PA7), // VJET_RB
        ],
    );
}
