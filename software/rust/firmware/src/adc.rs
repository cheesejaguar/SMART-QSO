//! `hal_adc.h`: ADC1, 12-bit. External inputs (pins TBR, gpio.rs):
//! VBATT PC2 (IN3), IBATT PC3 (IN4), VSOLAR PC4 (IN13), ISOLAR PC5 (IN14),
//! VBUS PB0 (IN15), TEMP_BOARD PB1 (IN16); TEMP_MCU is the internal sensor.
//!
//! `hal_adc_read_voltage()` gives each input in its own unit (volts, amps or
//! °C) as the simulation does: the pin voltage, with VDDA measured against
//! VREFINT, through the front-end scaling in [`FRONT_END`]. Those scale
//! factors are TBR until the EPS sense circuits are designed.

use embassy_stm32::adc::{Adc, AdcChannel, AnyAdcChannel, SampleTime, Temperature, VrefInt};
use embassy_stm32::peripherals::ADC1;

use crate::cell::Shared;
use crate::rc;

/// `HalAdcChannel_t` count
const CHANNELS: usize = 7;
const TEMP_MCU: usize = 5;
const RESOLUTION_12BIT: u32 = 12;
const FULL_SCALE: f64 = 4095.0;
/// VREFINT_CAL and TS_CAL were taken at VDDA = 3.0 V
const CAL_VDDA: f64 = 3.0;
const SAMPLE: SampleTime = SampleTime::CYCLES247_5;

/// Input = pin volts × gain + offset, per channel (TBR)
const FRONT_END: [(f64, f64); CHANNELS] = [
    (3.0, 0.0),      // VBATT: 1:3 divider (2S pack, 8.4 V max)
    (2.0, -3.3),     // IBATT: bidirectional sense amplifier, 1.65 V at 0 A, 0.5 V/A
    (4.0, 0.0),      // VSOLAR: 1:4 divider
    (1.0, 0.0),      // ISOLAR: 1 V/A sense amplifier
    (2.0, 0.0),      // VBUS: 1:2 divider (5 V bus)
    (1.0, 0.0),      // TEMP_MCU: computed from TS_CAL (see temperature())
    (100.0, -50.0),  // TEMP_BOARD: TMP36-style, 10 mV/°C, 0.5 V at 0 °C
];

pub(crate) struct Converter {
    adc: Adc<'static, ADC1>,
    inputs: [AnyAdcChannel<'static, ADC1>; 6],
    temp: Temperature,
    vref: VrefInt,
    ready: bool,
}

static ADC: Shared<Converter> = Shared::new();

pub(crate) fn install(adc: Adc<'static, ADC1>, inputs: [AnyAdcChannel<'static, ADC1>; 6]) {
    let temp = adc.enable_temperature();
    let vref = adc.enable_vrefint();
    ADC.install(Converter { adc, inputs, temp, vref, ready: false });
}

impl Converter {
    fn raw(&mut self, channel: usize) -> Option<u16> {
        if channel == TEMP_MCU {
            return Some(self.adc.blocking_read(&mut self.temp, SAMPLE));
        }
        let index = if channel > TEMP_MCU { channel - 1 } else { channel };
        let input = self.inputs.get_mut(index)?;
        Some(self.adc.blocking_read(input, SAMPLE))
    }

    /// VDDA, from VREFINT and its factory calibration
    fn vdda(&mut self) -> f64 {
        let cal = f64::from(self.vref.calibrated_value());
        let raw = f64::from(self.adc.blocking_read(&mut self.vref, SAMPLE));
        if raw > 0.0 { CAL_VDDA * cal / raw } else { 3.3 }
    }
}

/// TS_CAL1 (30 °C) and TS_CAL2 (130 °C), RM0432 / DS12023
fn ts_cal() -> (f64, f64) {
    const TS_CAL1: *const u16 = 0x1FFF_75A8 as *const u16;
    const TS_CAL2: *const u16 = 0x1FFF_75CA as *const u16;
    // SAFETY: factory calibration words in system memory, always readable
    // and aligned on this part
    unsafe { (f64::from(TS_CAL1.read_volatile()), f64::from(TS_CAL2.read_volatile())) }
}

/// Junction temperature from a raw sample taken at `vdda`
fn temperature(raw: u16, vdda: f64) -> f64 {
    let (cal1, cal2) = ts_cal();
    let at_cal = f64::from(raw) * vdda / CAL_VDDA;
    if cal2 > cal1 { (130.0 - 30.0) / (cal2 - cal1) * (at_cal - cal1) + 30.0 } else { f64::NAN }
}

fn channel_index(channel: u32) -> Option<usize> {
    usize::try_from(channel).ok().filter(|&c| c < CHANNELS)
}

/// Start the converter: 12 bits, VDDA reference
#[unsafe(no_mangle)]
pub extern "C" fn hal_adc_init(resolution: u32, _reference: u32) -> i32 {
    if resolution != RESOLUTION_12BIT {
        return rc::INVALID;
    }
    ADC.with(|a| {
        a.ready = true;
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// A 12-bit sample
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_adc_read_raw(channel: u32, value: *mut u16) -> i32 {
    // SAFETY: the C contract: value points to a uint16_t
    let Some(out) = (unsafe { rc::out(value) }) else { return rc::NULL_PTR };
    let Some(ch) = channel_index(channel) else { return rc::INVALID };
    ADC.with(|a| {
        if !a.ready {
            return rc::ERROR;
        }
        match a.raw(ch) {
            Some(v) => {
                *out = v;
                rc::OK
            }
            None => rc::INVALID,
        }
    })
    .unwrap_or(rc::BUSY)
}

/// The input in its own unit (see the module comment)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_adc_read_voltage(channel: u32, voltage: *mut f64) -> i32 {
    // SAFETY: the C contract: voltage points to a double
    let Some(out) = (unsafe { rc::out(voltage) }) else { return rc::NULL_PTR };
    let Some(ch) = channel_index(channel) else { return rc::INVALID };
    ADC.with(|a| {
        if !a.ready {
            return rc::ERROR;
        }
        let vdda = a.vdda();
        let Some(raw) = a.raw(ch) else { return rc::INVALID };
        *out = if ch == TEMP_MCU {
            temperature(raw, vdda)
        } else {
            let (gain, offset) = FRONT_END.get(ch).copied().unwrap_or((1.0, 0.0));
            f64::from(raw) / FULL_SCALE * vdda * gain + offset
        };
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Samples of several channels; stops at the first failure
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

/// The ADC calibrates itself when the driver starts it
#[unsafe(no_mangle)]
pub extern "C" fn hal_adc_calibrate() -> i32 {
    rc::OK
}

/// Make an input channel from a pin
pub(crate) fn input(pin: impl AdcChannel<ADC1> + 'static) -> AnyAdcChannel<'static, ADC1> {
    pin.degrade_adc()
}
