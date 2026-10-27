//! `hal_mtq.h`: the magnetorquer bridges (SQSO-OBC-BRD-001 B §10; SQSO-ICX-001
//! B §4.3).
//!
//! - TIM1 CH1–CH3 on PE9, PE11, PE13 (AF1): the bridges' EN lines, PWM at
//!   `HAL_MTQ_PWM_HZ` (40 kHz), the duty in ‰ of the period;
//! - TIM15 CH1 on PF9 (AF14): MTQ_ARM, one pulse of the burst length. The
//!   timer counts at 10 kHz in one-pulse mode, PWM mode 2 with the compare at
//!   1: the line rises 0.1 ms after the start and falls when the counter
//!   reaches the end of the burst and stops, whatever the software does
//!   (it cannot leave the line high: the counter stops at 0, where the
//!   output is inactive). The board's off-timer (0.92–0.98 s) backs it up.
//!
//! The direction lines (PE8, PE10, PE12) are GPIO (`gpio.rs`), set by the
//! flight software before each burst. Both timers are register-programmed
//! here; their pins are configured as alternate functions and owned by
//! this module (the `Peri` tokens are consumed at bring-up). No interrupt.

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_stm32::pac::gpio::vals::{Moder, Ospeedr, Ot, Pupdr};
use embassy_stm32::pac::timer::vals::Ocm;
use embassy_stm32::{pac, peripherals, rcc};

use crate::rc;

/// `HAL_MTQ_PWM_HZ`
const PWM_HZ: u32 = 40_000;
/// `HAL_MTQ_MAX_BURST_MS`
const MAX_BURST_MS: u32 = 900;
/// `HAL_MTQ_DUTY_FULL`
const DUTY_FULL: u32 = 1000;
/// TIM15's counter clock, Hz
const ARM_COUNT_HZ: u32 = 10_000;

static READY: AtomicBool = AtomicBool::new(false);

/// One pin to an alternate function, push-pull, no pull
fn af(port: pac::gpio::Gpio, n: usize, func: u8) {
    port.afr(n / 8).modify(|w| w.set_afr(n % 8, func));
    port.otyper().modify(|w| w.set_ot(n, Ot::PUSH_PULL));
    port.pupdr().modify(|w| w.set_pupdr(n, Pupdr::FLOATING));
    port.ospeedr().modify(|w| w.set_ospeedr(n, Ospeedr::LOW_SPEED));
    port.moder().modify(|w| w.set_moder(n, Moder::ALTERNATE));
}

/// Start TIM1 (40 kHz, every duty 0) and arm TIM15 for one-pulse bursts;
/// then hand the pins to the timers. `_pins` are the four pins' tokens,
/// consumed so nothing else can take them
pub(crate) fn install<P>(
    _tim1: embassy_stm32::Peri<'static, peripherals::TIM1>,
    _tim15: embassy_stm32::Peri<'static, peripherals::TIM15>,
    _pins: P,
) {
    rcc::enable_and_reset::<peripherals::TIM1>();
    rcc::enable_and_reset::<peripherals::TIM15>();
    let t1 = pac::TIM1;
    let clock = rcc::frequency::<peripherals::TIM1>().0;
    let arr = u16::try_from((clock / PWM_HZ).saturating_sub(1)).unwrap_or(u16::MAX);
    t1.psc().write_value(0);
    t1.arr().write(|w| w.set_arr(arr));
    for ch in 0..3usize {
        t1.ccmr_output(ch / 2).modify(|w| {
            w.set_ocm(ch % 2, Ocm::PWM_MODE1);
            w.set_ocpe(ch % 2, true);
        });
        t1.ccr(ch).write(|w| w.set_ccr(0));
        t1.ccer().modify(|w| w.set_cce(ch, true));
    }
    t1.egr().write(|w| w.set_ug(true));
    t1.bdtr().modify(|w| w.set_moe(true));
    t1.cr1().modify(|w| {
        w.set_arpe(true);
        w.set_cen(true);
    });

    let t15 = pac::TIM15;
    let clock15 = rcc::frequency::<peripherals::TIM15>().0;
    let psc = u16::try_from((clock15 / ARM_COUNT_HZ).saturating_sub(1)).unwrap_or(u16::MAX);
    t15.psc().write_value(psc);
    t15.ccmr_output().modify(|w| w.set_ocm(0, Ocm::PWM_MODE2));
    t15.ccr(0).write(|w| w.set_ccr(1));
    t15.arr().write(|w| w.set_arr(2));
    t15.egr().write(|w| w.set_ug(true));
    t15.sr().write(|w| w.set_uif(false));
    t15.cr1().modify(|w| w.set_opm(true));
    t15.ccer().modify(|w| w.set_cce(0, true));
    t15.bdtr().modify(|w| w.set_moe(true));

    // The pins to the timers, now that their outputs are low
    af(pac::GPIOE, 9, 1);
    af(pac::GPIOE, 11, 1);
    af(pac::GPIOE, 13, 1);
    af(pac::GPIOF, 9, 14);
    READY.store(true, Ordering::Relaxed);
}

/// Start one burst: the duties, then MTQ_ARM for `on_ms`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_mtq_burst(duty_permille: *const u16, on_ms: u32) -> i32 {
    if duty_permille.is_null() {
        return rc::NULL_PTR;
    }
    // SAFETY: the C contract: three uint16_t
    let duty = unsafe { [duty_permille.read(), duty_permille.add(1).read(), duty_permille.add(2).read()] };
    if duty.iter().any(|&d| u32::from(d) > DUTY_FULL) || on_ms == 0 || on_ms > MAX_BURST_MS {
        return rc::INVALID;
    }
    if !READY.load(Ordering::Relaxed) {
        return rc::ERROR;
    }
    let t1 = pac::TIM1;
    let period = u32::from(t1.arr().read().arr()) + 1;
    for (ch, d) in duty.iter().enumerate() {
        let ccr = u16::try_from(u32::from(*d) * period / DUTY_FULL).unwrap_or(u16::MAX);
        t1.ccr(ch).write(|w| w.set_ccr(ccr));
    }
    let t15 = pac::TIM15;
    t15.cr1().modify(|w| w.set_cen(false));
    let end = u16::try_from(on_ms * (ARM_COUNT_HZ / 1000) + 1).unwrap_or(u16::MAX);
    t15.arr().write(|w| w.set_arr(end));
    t15.cnt().write(|w| w.set_cnt(0));
    t15.egr().write(|w| w.set_ug(true));
    t15.sr().write(|w| w.set_uif(false));
    t15.cr1().modify(|w| w.set_cen(true));
    rc::OK
}

/// End any burst now: MTQ_ARM low, every duty 0
#[unsafe(no_mangle)]
pub extern "C" fn hal_mtq_off() -> i32 {
    if !READY.load(Ordering::Relaxed) {
        return rc::OK;
    }
    let t15 = pac::TIM15;
    t15.cr1().modify(|w| w.set_cen(false));
    t15.cnt().write(|w| w.set_cnt(0));
    let t1 = pac::TIM1;
    for ch in 0..3usize {
        t1.ccr(ch).write(|w| w.set_ccr(0));
    }
    rc::OK
}

/// Whether a burst is running (TIM15 counting)
#[unsafe(no_mangle)]
pub extern "C" fn hal_mtq_active() -> bool {
    READY.load(Ordering::Relaxed) && pac::TIM15.cr1().read().cen()
}
