//! `hal_capture.h`: timer input captures (SQSO-ICX-001 B §4.3).
//!
//! - TIM3 counts at 10 kHz (16 bits, 6.55 s around); CH1 captures the GNSS
//!   1PPS (PC6, AF2) and CH3 the mission clock's 1 Hz CLKOUT (PC8, AF2) on
//!   their rising edges. The period between two edges is the MCU's count of
//!   the outside second: 100 µs resolution (100 ppm), enough to see a clock
//!   failure (`CLOCK_LIMIT_PPM`, the executive).
//! - TIM8 counts at 10 kHz; CH2 captures both edges of PTT_EXCITER (PC7,
//!   AF3). Each edge's time is the capture's count back from the counter
//!   now, so the keyed time is measured to 0.1 ms, not estimated.
//!
//! Polled after every main-loop pass ([`poll`], 20 ms apart: no edge is more
//! than 6.55 s old when it is read). No interrupt. The pins belong to this
//! module (their `Peri` tokens are consumed); `GPIO_PIN_PTT_EXC`'s level is
//! read from GPIOC's input register ([`ptt_level`]).

use embassy_stm32::pac::gpio::vals::{Moder, Pupdr};
use embassy_stm32::pac::timer::vals::CcmrInputCcs;
use embassy_stm32::{pac, peripherals, rcc};

use crate::cell::Shared;
use crate::rc;

/// The timers' counter clock, Hz
const COUNT_HZ: u32 = 10_000;
/// µs a count
const US_PER_COUNT: u64 = 1_000_000 / COUNT_HZ as u64;
/// A period older than this is not reported, µs
const STALE_US: u64 = 3_000_000;

/// `HalCaptureInput_t`
const CAPTURE_GNSS_PPS: u32 = 0;
const CAPTURE_RTC_1HZ: u32 = 1;
const CAPTURE_PTT_EXC: u32 = 2;

struct State {
    /// Per periodic input (PPS on CH1, RTC on CH3): the last capture, the
    /// latest period in counts, and when it was measured, µs
    last: [Option<u16>; 2],
    period: [Option<(u32, u64)>; 2],
    /// PTT: high since (µs), and the accumulated high time, µs
    high_since: Option<u64>,
    high_us: u64,
}

static STATE: Shared<State> = Shared::new();

fn input_pin(port: pac::gpio::Gpio, n: usize, func: u8) {
    port.afr(n / 8).modify(|w| w.set_afr(n % 8, func));
    port.pupdr().modify(|w| w.set_pupdr(n, Pupdr::FLOATING));
    port.moder().modify(|w| w.set_moder(n, Moder::ALTERNATE));
}

fn now_us() -> u64 {
    embassy_time::Instant::now().as_micros()
}

/// Start the two timers and their captures; `_pins` are the three pins'
/// tokens, consumed so nothing else can take them
pub(crate) fn install<P>(
    _tim3: embassy_stm32::Peri<'static, peripherals::TIM3>,
    _tim8: embassy_stm32::Peri<'static, peripherals::TIM8>,
    _pins: P,
) {
    rcc::enable_and_reset::<peripherals::TIM3>();
    rcc::enable_and_reset::<peripherals::TIM8>();
    let t3 = pac::TIM3;
    let psc3 = u16::try_from((rcc::frequency::<peripherals::TIM3>().0 / COUNT_HZ).saturating_sub(1)).unwrap_or(0);
    t3.psc().write_value(psc3);
    t3.arr().write(|w| w.set_arr(0xFFFF));
    // CH1 and CH3 inputs on TI1 and TI3, a 4-sample filter, rising edges
    for ch in [0usize, 2] {
        t3.ccmr_input(ch / 2).modify(|w| {
            w.set_ccs(ch % 2, CcmrInputCcs::TI4);
            w.set_icf(ch % 2, pac::timer::vals::FilterValue::FCK_INT_N4);
        });
        t3.ccer().modify(|w| {
            w.set_ccp(ch, false);
            w.set_ccnp(ch, false);
            w.set_cce(ch, true);
        });
    }
    t3.egr().write(|w| w.set_ug(true));
    t3.cr1().modify(|w| w.set_cen(true));

    let t8 = pac::TIM8;
    let psc8 = u16::try_from((rcc::frequency::<peripherals::TIM8>().0 / COUNT_HZ).saturating_sub(1)).unwrap_or(0);
    t8.psc().write_value(psc8);
    t8.arr().write(|w| w.set_arr(0xFFFF));
    t8.ccmr_input(0).modify(|w| {
        w.set_ccs(1, CcmrInputCcs::TI4);
        w.set_icf(1, pac::timer::vals::FilterValue::FCK_INT_N4);
    });
    t8.ccer().modify(|w| {
        // Both edges: CC2P and CC2NP set
        w.set_ccp(1, true);
        w.set_ccnp(1, true);
        w.set_cce(1, true);
    });
    t8.egr().write(|w| w.set_ug(true));
    t8.cr1().modify(|w| w.set_cen(true));

    input_pin(pac::GPIOC, 6, 2);
    input_pin(pac::GPIOC, 8, 2);
    input_pin(pac::GPIOC, 7, 3);
    STATE.install(State { last: [None; 2], period: [None; 2], high_since: None, high_us: 0 });
}

/// PTT_EXCITER's level (GPIOC IDR bit 7; the pin is in alternate mode)
pub(crate) fn ptt_level() -> bool {
    pac::GPIOC.idr().read().idr(7) == pac::gpio::vals::Idr::HIGH
}

/// After each pass: take the captures that arrived
pub(crate) fn poll() {
    let now = now_us();
    let _ = STATE.with(|s| {
        let t3 = pac::TIM3;
        let sr = t3.sr().read();
        for (i, ch) in [(0usize, 0usize), (1, 2)] {
            if sr.ccif(ch) {
                let overrun = sr.ccof(ch);
                let ccr = t3.ccr(ch).read().ccr();
                t3.sr().write(|w| {
                    w.0 = !0;
                    w.set_ccof(ch, false);
                });
                if let (Some(slot), Some(period)) = (s.last.get_mut(i), s.period.get_mut(i)) {
                    if let (Some(prev), false) = (*slot, overrun) {
                        *period = Some((u32::from(ccr.wrapping_sub(prev)), now));
                    }
                    *slot = Some(ccr);
                }
            }
        }
        let t8 = pac::TIM8;
        let sr8 = t8.sr().read();
        let level = ptt_level();
        if sr8.ccif(1) {
            let ccr = t8.ccr(1).read().ccr();
            let cnt = t8.cnt().read().cnt();
            t8.sr().write(|w| {
                w.0 = !0;
                w.set_ccof(1, false);
            });
            let edge = now.saturating_sub(u64::from(cnt.wrapping_sub(ccr)) * US_PER_COUNT);
            match (level, s.high_since) {
                (true, None) => s.high_since = Some(edge),
                (false, Some(t)) => {
                    s.high_us = s.high_us.saturating_add(edge.saturating_sub(t));
                    s.high_since = None;
                }
                _ => {}
            }
        }
        // Edges missed (two in one pass): resynchronize on the level
        match (level, s.high_since) {
            (true, None) => s.high_since = Some(now),
            (false, Some(t)) => {
                s.high_us = s.high_us.saturating_add(now.saturating_sub(t));
                s.high_since = None;
            }
            _ => {}
        }
    });
}

/// The latest period of the 1PPS or the RTC's 1 Hz, µs
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_capture_period_us(input: u32, period_us: *mut u32) -> i32 {
    // SAFETY: the C contract: NULL or a uint32_t
    let Some(out) = (unsafe { rc::out(period_us) }) else { return rc::NULL_PTR };
    let i = match input {
        CAPTURE_GNSS_PPS => 0,
        CAPTURE_RTC_1HZ => 1,
        _ => return rc::INVALID,
    };
    let now = now_us();
    match STATE.with(|s| s.period.get(i).copied().flatten()) {
        Some(Some((counts, at))) if now.saturating_sub(at) <= STALE_US => {
            *out = u32::try_from(u64::from(counts) * US_PER_COUNT).unwrap_or(u32::MAX);
            rc::OK
        }
        Some(_) => rc::TIMEOUT,
        None => rc::BUSY,
    }
}

/// PTT_EXCITER's high time since boot, ms
#[unsafe(no_mangle)]
pub extern "C" fn hal_capture_high_ms(input: u32) -> u32 {
    if input != CAPTURE_PTT_EXC {
        return 0;
    }
    let now = now_us();
    let us = STATE.with(|s| s.high_us + s.high_since.map_or(0, |t| now.saturating_sub(t))).unwrap_or(0);
    u32::try_from(us / 1000).unwrap_or(u32::MAX)
}
