//! `hal_gpio.h` on the STM32L4R5.
//!
//! PROVISIONAL pin assignment (TBR): `hardware/obc/OBC_DESIGN.md` and
//! `STORAGE_DESIGN.md` disagree (PA4–PA7, PC0–PC1), so pins follow
//! OBC_DESIGN where that does not collide with the FRAM SPI bus, and the
//! rest are placeholders until the OBC schematic baselines a pinout. The
//! heater and status LED moved from PC2/PC3 (the C port's table) to PD2/PD3
//! so that PC2/PC3 can be ADC inputs (`adc.rs`). See `docs/BASELINE_PARAMETERS.md`.

use embassy_stm32::Peri;
use embassy_stm32::gpio::{Flex, Level, Pin, Pull, Speed};

use crate::cell::Shared;
use crate::rc;

/// Pins, in `HalGpioPin_t` order
pub(crate) const COUNT: usize = 16;

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
    pins: [Flex<'static>; COUNT],
}

static GPIO: Shared<Gpio> = Shared::new();

fn pin(p: Peri<'static, impl Pin>, boot: Boot) -> Flex<'static> {
    let mut f = Flex::new(p);
    match boot {
        Boot::OutputLow => {
            f.set_level(Level::Low);
            f.set_as_output(Speed::Low);
        }
        Boot::Input(pull) => f.set_as_input(pull),
    }
    f
}

/// Configure every pin to its boot state (at reset, before the flight
/// software runs: enable and burn lines are low from the first instant)
#[allow(clippy::too_many_arguments)]
pub(crate) fn install(
    payload_en: Peri<'static, impl Pin>,      // PB4
    radio_en: Peri<'static, impl Pin>,        // PB5
    adcs_en: Peri<'static, impl Pin>,         // PB6
    heater_en: Peri<'static, impl Pin>,       // PD2
    deploy_en: Peri<'static, impl Pin>,       // PA1
    led_status: Peri<'static, impl Pin>,      // PD3
    jetson_pwr: Peri<'static, impl Pin>,      // PA2
    rf_tx_en: Peri<'static, impl Pin>,        // PA3
    alert: Peri<'static, impl Pin>,           // PB7
    deploy_sense: Peri<'static, impl Pin>,    // PE2 (PA0 is the TX_TEMP ADC input)
    umbilical_sense: Peri<'static, impl Pin>, // PB8
    antenna_burn: Peri<'static, impl Pin>,    // PB9
    solar_burn: Peri<'static, impl Pin>,      // PB10
    antenna_sense: Peri<'static, impl Pin>,   // PB11
    solar_sense: Peri<'static, impl Pin>,     // PB12
    tx_pwr_sel: Peri<'static, impl Pin>,      // PE3
) {
    GPIO.install(Gpio {
        pins: [
            pin(payload_en, Boot::OutputLow),
            pin(radio_en, Boot::OutputLow),
            pin(adcs_en, Boot::OutputLow),
            pin(heater_en, Boot::OutputLow),
            pin(deploy_en, Boot::OutputLow),
            pin(led_status, Boot::OutputLow),
            pin(jetson_pwr, Boot::OutputLow),
            pin(rf_tx_en, Boot::OutputLow),
            pin(alert, Boot::Input(Pull::Up)),
            pin(deploy_sense, Boot::Input(Pull::None)),
            pin(umbilical_sense, Boot::Input(Pull::Down)),
            pin(antenna_burn, Boot::OutputLow),
            pin(solar_burn, Boot::OutputLow),
            pin(antenna_sense, Boot::Input(Pull::None)),
            pin(solar_sense, Boot::Input(Pull::None)),
            pin(tx_pwr_sel, Boot::OutputLow),
        ],
    });
}

fn with_pin<R>(pin: u32, invalid: R, busy: R, f: impl FnOnce(&mut Flex<'static>) -> R) -> R {
    let Ok(i) = usize::try_from(pin) else { return invalid };
    match GPIO.with(|g| g.pins.get_mut(i).map(f)) {
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
