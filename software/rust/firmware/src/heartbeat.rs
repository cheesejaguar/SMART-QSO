//! `hal_gpio_heartbeat()`: the `XPDR_EN` heartbeat on PE4
//! (docs/hardware/BUILD_VS_BUY.md §3.2; `sqso_fsw::transponder`).
//!
//! TIM7, a basic timer with no output, interrupts at twice
//! `HAL_GPIO_HEARTBEAT_HZ`; its handler toggles PE4 through the port's
//! set/reset register while the lease lasts, and drives it low when the
//! lease runs out. The main loop renews the lease on every pass, so:
//!
//! - a main loop that stalls (a stage that never returns, the watchdog
//!   about to reset) stops renewing it, and the wave stops within the lease;
//! - a hard fault, a lock-up or interrupts left masked stop the handler,
//!   and with it the edges, at once;
//! - a reset puts PE4 in its reset state (analog input, high impedance),
//!   which the radio board pulls down.
//!
//! The radio board's retriggerable monostable then drops the transponder
//! enable within about 50 ms of the last edge. Nothing here can hold the
//! transponder on without the main loop: the handler only counts the
//! lease down. PE4 is configured as a push-pull output, low, by `gpio.rs`
//! at boot; this module only writes its set/reset bits.
//!
//! The timer is the only interrupt the flight software adds; its handler
//! is a few instructions and uses no stack beyond its frame. Not run on
//! hardware yet (HIL: the heartbeat stops within the lease after a stalled
//! loop and at once after a halt).

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use embassy_stm32::interrupt::InterruptExt;
use embassy_stm32::{interrupt, pac, peripherals, rcc};

use crate::rc;

/// `GPIO_PIN_XPDR_EN`
const PIN_XPDR_EN: u32 = 16;
/// PE4: its bit in GPIOE's set/reset register
const XPDR_BIT: usize = 4;
/// `HAL_GPIO_HEARTBEAT_HZ`
const HEARTBEAT_HZ: u32 = 100;
/// Timer interrupts a second (one edge each)
const EDGE_HZ: u32 = 2 * HEARTBEAT_HZ;
/// The timer's counter clock, Hz (prescaled from the APB1 timer clock)
const COUNT_HZ: u32 = 10_000;
/// Interrupt period, ms (5 ms at 100 Hz)
const EDGE_MS: u32 = 1000 / EDGE_HZ;

/// Edges left in the lease (0: stopped, pin low)
static EDGES_LEFT: AtomicU32 = AtomicU32::new(0);
/// The pin's level now
static LEVEL: AtomicBool = AtomicBool::new(false);
/// The timer has been started
static STARTED: AtomicBool = AtomicBool::new(false);

fn drive(high: bool) {
    LEVEL.store(high, Ordering::Relaxed);
    // One write to the set/reset register: no read-modify-write of the port,
    // so the main loop's writes to the other PE pins are never disturbed
    pac::GPIOE.bsrr().write(|w| {
        if high {
            w.set_bs(XPDR_BIT, true);
        } else {
            w.set_br(XPDR_BIT, true);
        }
    });
}

/// Start TIM7 at `EDGE_HZ` with its interrupt (the wave itself stays off
/// until the first lease). Called once at bring-up, after the GPIO
pub(crate) fn install() {
    rcc::enable_and_reset::<peripherals::TIM7>();
    let clock = rcc::frequency::<peripherals::TIM7>().0;
    let psc = u16::try_from((clock / COUNT_HZ).saturating_sub(1)).unwrap_or(u16::MAX);
    let arr = u16::try_from((COUNT_HZ / EDGE_HZ).saturating_sub(1)).unwrap_or(u16::MAX);
    let tim = pac::TIM7;
    tim.psc().write_value(psc);
    tim.arr().write(|w| w.set_arr(arr));
    tim.egr().write(|w| w.set_ug(true)); // load the prescaler
    tim.sr().write(|w| w.set_uif(false));
    tim.dier().write(|w| w.set_uie(true));
    tim.cr1().write(|w| w.set_cen(true));
    // At the reset priority, as every other interrupt (scripts/stack_usage.py)
    interrupt::TIM7.unpend();
    // SAFETY: the handler below touches only this module's atomics and
    // PE4's set/reset bits
    unsafe { interrupt::TIM7.enable() };
    STARTED.store(true, Ordering::Relaxed);
}

/// TIM7 update: one edge while the lease lasts, then the pin low
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
unsafe extern "C" fn TIM7() {
    pac::TIM7.sr().write(|w| w.set_uif(false));
    // The main loop only stores to EDGES_LEFT, and this handler preempts it,
    // so the load and the store below cannot be split by a renewal
    let left = EDGES_LEFT.load(Ordering::Relaxed);
    if left > 0 {
        EDGES_LEFT.store(left - 1, Ordering::Relaxed);
        drive(!LEVEL.load(Ordering::Relaxed));
    } else if LEVEL.load(Ordering::Relaxed) {
        drive(false);
    } else {
        // Stopped and low
    }
}

/// Run the heartbeat for `lease_ms` from now (0: stop it now, pin low)
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_heartbeat(pin: u32, lease_ms: u32) -> i32 {
    if pin != PIN_XPDR_EN {
        return rc::INVALID;
    }
    if !STARTED.load(Ordering::Relaxed) {
        return rc::ERROR;
    }
    let edges = lease_ms.div_ceil(EDGE_MS);
    EDGES_LEFT.store(edges, Ordering::Relaxed);
    if edges == 0 {
        critical_section::with(|_| drive(false));
    }
    rc::OK
}
