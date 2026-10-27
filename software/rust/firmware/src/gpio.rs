//! `hal_gpio.h` on the STM32L4R5: the pin allocation of SQSO-ICX-001 D §4.1
//! (`hardware/electrical/INTERCONNECT.md`, the master allocation; its §6
//! lists the changes from the earlier, provisional map).
//!
//! Every output to the EPS, the RF board and the sensor rails is driven low
//! before its output driver is enabled ([`Boot::OutputLow`]), so no enable,
//! burn or passivation line glitches high at reset; until then the pins are
//! in their reset state (analog, high impedance) and the boards' pull-downs
//! hold the lines low. In particular PAYLOAD_EN is PB2 (no pull at reset;
//! PB4, the earlier pin, is NJTRST with a pull-up that switched SW1 on at
//! every reset: RID OBCB-01), DEPLOY_EN is PF3, JETSON_PWR is PA8.
//!
//! Logical pins without a GPIO here: `GPIO_PIN_ANTENNA_SENSE` (13: the AntS
//! reports over I2C; PB11 is now LPUART1_TX: reads low, cannot be driven) and
//! `GPIO_PIN_PTT_EXC` (42; PC7 belongs to the TIM8 capture, `capture.rs`,
//! and its level is read from the port's input register there).

use embassy_stm32::Peri;
use embassy_stm32::gpio::{Flex, Level, Pin, Pull, Speed};

use crate::cell::Shared;
use crate::rc;

/// Pins, in `HalGpioPin_t` order (`GPIO_PIN_COUNT`)
pub(crate) const COUNT: usize = 48;
/// `GPIO_PIN_PTT_EXC`: read from the capture's pin
const PIN_PTT_EXC: u32 = 42;

const DIR_OUTPUT: u32 = 1;
const PULL_UP: u32 = 1;
const PULL_DOWN: u32 = 2;

/// Each pin's reset configuration: outputs low, inputs with their pull
#[derive(Clone, Copy)]
pub(crate) enum Boot {
    /// Output, driven low before the output is enabled (no glitch on an
    /// enable or burn line)
    OutputLow,
    /// Input
    Input(Pull),
}

pub(crate) struct Gpio {
    pins: [Option<Flex<'static>>; COUNT],
}

static GPIO: Shared<Gpio> = Shared::new();

/// A pin in its boot state
pub(crate) fn pin(p: Peri<'static, impl Pin>, boot: Boot) -> Option<Flex<'static>> {
    let mut f = Flex::new(p);
    match boot {
        Boot::OutputLow => {
            f.set_level(Level::Low);
            f.set_as_output(Speed::Low);
        }
        Boot::Input(pull) => f.set_as_input(pull),
    }
    Some(f)
}

/// Output low at boot
pub(crate) fn out(p: Peri<'static, impl Pin>) -> Option<Flex<'static>> {
    pin(p, Boot::OutputLow)
}

/// Input with the board's own pull resistors (no internal pull)
pub(crate) fn input(p: Peri<'static, impl Pin>) -> Option<Flex<'static>> {
    pin(p, Boot::Input(Pull::None))
}

/// Install the pins (at reset, before the flight software runs), in
/// `HalGpioPin_t` order; `main.rs` builds the table from SQSO-ICX-001 D
pub(crate) fn install(pins: [Option<Flex<'static>>; COUNT]) {
    GPIO.install(Gpio { pins });
}

/// Install every pin of SQSO-ICX-001 D §4.1 in `HalGpioPin_t` order, at
/// reset, before the flight software runs: outputs low first
#[allow(clippy::too_many_arguments)]
pub(crate) fn install_board(
    pb2: Peri<'static, impl Pin>,
    pb5: Peri<'static, impl Pin>,
    pb6: Peri<'static, impl Pin>,
    pd2: Peri<'static, impl Pin>,
    pf3: Peri<'static, impl Pin>,
    pd3: Peri<'static, impl Pin>,
    pa8: Peri<'static, impl Pin>,
    pa3: Peri<'static, impl Pin>,
    pb7: Peri<'static, impl Pin>,
    pe2: Peri<'static, impl Pin>,
    pb8: Peri<'static, impl Pin>,
    pb9: Peri<'static, impl Pin>,
    pb10: Peri<'static, impl Pin>,
    pb12: Peri<'static, impl Pin>,
    pe3: Peri<'static, impl Pin>,
    pe4: Peri<'static, impl Pin>,
    pe5: Peri<'static, impl Pin>,
    pe6: Peri<'static, impl Pin>,
    pf4: Peri<'static, impl Pin>,
    pg1: Peri<'static, impl Pin>,
    pg0: Peri<'static, impl Pin>,
    pf13: Peri<'static, impl Pin>,
    pe7: Peri<'static, impl Pin>,
    pf5: Peri<'static, impl Pin>,
    pf6: Peri<'static, impl Pin>,
    pf7: Peri<'static, impl Pin>,
    pf8: Peri<'static, impl Pin>,
    pf10: Peri<'static, impl Pin>,
    pf11: Peri<'static, impl Pin>,
    pf12: Peri<'static, impl Pin>,
    pd14: Peri<'static, impl Pin>,
    pd10: Peri<'static, impl Pin>,
    pd15: Peri<'static, impl Pin>,
    pd11: Peri<'static, impl Pin>,
    pe8: Peri<'static, impl Pin>,
    pe10: Peri<'static, impl Pin>,
    pe12: Peri<'static, impl Pin>,
    pe14: Peri<'static, impl Pin>,
    pe15: Peri<'static, impl Pin>,
    pc9: Peri<'static, impl Pin>,
    pg12: Peri<'static, impl Pin>,
    pg6: Peri<'static, impl Pin>,
    pg15: Peri<'static, impl Pin>,
    pg11: Peri<'static, impl Pin>,
    pf14: Peri<'static, impl Pin>,
    pf15: Peri<'static, impl Pin>,
) {
    install([
        out(pb2),    // 0 PAYLOAD_EN (SW1)
        out(pb5),    // 1 RADIO_EN
        out(pb6),    // 2 ADCS_EN
        out(pd2),    // 3 HEATER_EN
        out(pf3),    // 4 DEPLOY_EN
        out(pd3),    // 5 LED_STATUS
        out(pa8),    // 6 JETSON_PWR (eFuse U19)
        out(pa3),    // 7 RF_TX_EN
        input(pb7),  // 8 EPS_ALERT_N (open drain; the EPS 4.7 kΩ pull-up)
        input(pe2),  // 9 DEPLOY_SENSE
        input(pb8),  // 10 UMBILICAL_SENSE (OBC 100 kΩ pull-down)
        out(pb9),    // 11 BURN_W2A
        out(pb10),   // 12 BURN_W1A
        None,        // 13 ANTENNA_SENSE: not wired (PB11 is LPUART1_TX)
        input(pb12), // 14 SOLAR_SENSE
        out(pe3),    // 15 TX_PWR_SEL
        out(pe4),    // 16 XPDR_EN (heartbeat.rs)
        out(pe5),    // 17 GNSS_EN
        out(pe6),    // 18 BURN_W1B
        out(pf4),    // 19 EOL_ARM
        out(pg1),    // 20 EOL_FIRE
        input(pg0),  // 21 RX_CD
        input(pf13), // 22 RX_FAULT
        out(pe7),    // 23 BURN_W2B
        out(pf5),    // 24 LOCK_CLR
        out(pf6),    // 25 GYRO_EN
        out(pf7),    // 26 IMU2_EN
        out(pf8),    // 27 SNS_EN
        input(pf10), // 28 GYRO_RB
        input(pf11), // 29 IMU2_RB
        input(pf12), // 30 SNS_RB
        out(pd14),   // 31 GYRO_RST_N (low: reset)
        input(pd10), // 32 GYRO_DRDY
        input(pd15), // 33 IMU2_DRDY
        input(pd11), // 34 MAG_DRDY
        out(pe8),    // 35 MTQ_X_DIR
        out(pe10),   // 36 MTQ_Y_DIR
        out(pe12),   // 37 MTQ_Z_DIR
        input(pe14), // 38 MTQ_FAULT_N
        input(pe15), // 39 MTQ_EXPIRED
        input(pc9),  // 40 TONE_OK
        input(pg12), // 41 XPDR_KEY
        None,        // 42 PTT_EXC: PC7 is TIM8's (capture.rs)
        input(pg6),  // 43 JET_PG
        input(pg15), // 44 GNSS_PG
        out(pg11),   // 45 WDI
        out(pf14),   // 46 MAG_EN (U35, 3V3_MAG)
        input(pf15), // 47 MAG_RB
    ]);
}

fn with_pin<R>(pin: u32, invalid: R, busy: R, f: impl FnOnce(&mut Flex<'static>) -> R) -> R {
    let Ok(i) = usize::try_from(pin) else { return invalid };
    match GPIO.with(|g| g.pins.get_mut(i).and_then(Option::as_mut).map(f)) {
        Some(Some(r)) => r,
        Some(None) => invalid,
        None => busy,
    }
}

/// The pins were configured at reset; nothing more to do
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_init() -> i32 {
    rc::OK
}

/// Set a pin's direction and pull
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_config(pin: u32, direction: u32, pull: u32) -> i32 {
    let pull = match pull {
        PULL_UP => Pull::Up,
        PULL_DOWN => Pull::Down,
        _ => Pull::None,
    };
    with_pin(pin, rc::INVALID, rc::BUSY, |f| {
        if direction == DIR_OUTPUT {
            f.set_level(Level::Low);
            f.set_as_output(Speed::Low);
        } else {
            f.set_as_input(pull);
        }
        rc::OK
    })
}

/// Drive an output pin
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_set(pin: u32, state: bool) -> i32 {
    with_pin(pin, rc::INVALID, rc::BUSY, |f| {
        f.set_level(if state { Level::High } else { Level::Low });
        rc::OK
    })
}

/// A pin's input level
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_get(pin: u32) -> bool {
    if pin == PIN_PTT_EXC {
        return crate::capture::ptt_level();
    }
    with_pin(pin, false, false, |f| f.is_high())
}

/// Invert an output pin
#[unsafe(no_mangle)]
pub extern "C" fn hal_gpio_toggle(pin: u32) -> i32 {
    with_pin(pin, rc::INVALID, rc::BUSY, |f| {
        f.toggle();
        rc::OK
    })
}
