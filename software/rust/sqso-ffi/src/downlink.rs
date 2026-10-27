//! `downlink.h`: the downlink transmitter's gate, limits and monitors
//! (`sqso_fsw::downlink`), on the radio, GPIO and ADC HAL and rf_control.
//!
//! Coverage: only a nested call (an interrupt) finds `DOWNLINK` in use (busy:
//! `Global::with` gives `None`); no C callback runs while it is held. The
//! unit test `tests::busy_is_refused` covers those paths
//! (`software/flight/scripts/safety_coverage.py` counts the unit tests'
//! export too).
//!
//! @requirement SRS-I023 Downlink frames to the TNC in KISS framing, with the TNC parameters
//! @requirement SRS-I024 No downlink frame unless transmission is permitted when it is handed over
//! @requirement SRS-I025 Key-down time per frame and duty limited
//! @requirement SRS-I026 Downlink power at the commanded level, never above 1 W
//! @requirement SRS-I027 Transmitter temperature and VSWR protection
//! @requirement SRS-F161 TX_TEMP at or below -50 °C or above +125 °C, rising faster than 5 °C/s (over at least 15 s) or 60 °C from the board temperature is a sensor fault: the downlink then uses the board temperature plus 15 °C

use sqso_fsw::downlink::{Downlink, Event, Refusal, Sample};
use sqso_fsw::pa_temp::PaTemp;
use sqso_fsw::rf_control::POWER_1W;

use crate::c::{self, rc, severity};
use crate::cell::Global;
use crate::ptr;
use crate::rf_control::{self, GPIO_DIR_OUTPUT};

/// `GPIO_PIN_TX_PWR_SEL`
pub(crate) const GPIO_PIN_TX_PWR_SEL: u32 = 15;
/// `ADC_CHANNEL_TX_TEMP`, `_TX_FWD`, `_TX_REV`
pub(crate) const ADC_CHANNEL_TX_TEMP: u32 = 7;
pub(crate) const ADC_CHANNEL_TX_FWD: u32 = 8;
pub(crate) const ADC_CHANNEL_TX_REV: u32 = 9;
/// `FAULT_TYPE_RF_TX`
pub(crate) const FAULT_TYPE_RF_TX: u32 = 23;

unsafe extern "C" {
    fn hal_radio_transmit(frame: *const u8, len: usize) -> i32;
    fn hal_adc_read_voltage(channel: u32, voltage: *mut f64) -> i32;
}

static DOWNLINK: Global<Downlink> = Global::new(Downlink::new());
/// The transmitter temperature's plausibility (RID SMA-B19)
static PA: Global<PaTemp> = Global::new(PaTemp::new());
/// `ADC_CHANNEL_TEMP_BOARD`: the OBC board's TMP235
const ADC_CHANNEL_TEMP_BOARD: u32 = crate::pins::ADC_TEMP_BOARD;

/// The transmitter temperature the downlink uses: TX_TEMP, or with the
/// sensor faulted the board's plus the margin; a new fault is logged
fn pa_temp() -> f64 {
    let (pa, board) = (adc(ADC_CHANNEL_TX_TEMP), adc(ADC_CHANNEL_TEMP_BOARD));
    let now = c::now_ms();
    let Some((t, new_fault)) = PA.with(|p| {
        let before = p.faults;
        let t = p.select(now, pa, board);
        (t, p.faults != before)
    }) else {
        return pa;
    };
    if new_fault {
        c::log(FAULT_TYPE_RF_TX, severity::WARNING, c"TX_TEMP implausible: board temperature used", 0.0);
    }
    t
}

/// The sensor faulted now, and its faults since boot (housekeeping)
pub(crate) fn pa_sensor_status() -> (bool, u32) {
    PA.with(|p| (p.faulted, p.faults)).unwrap_or((false, 0))
}

/// `DownlinkStatus_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DownlinkStatus {
    /// The counters
    pub counters: sqso_fsw::downlink::Counters,
    /// Highest power level allowed
    pub level_cap: u32,
    /// Held for temperature
    pub over_temp: bool,
    /// Backed off now
    pub backoff: bool,
    /// A frame may be on the air now
    pub keyed: bool,
}

/// One monitor input; not a number if the ADC fails
fn adc(channel: u32) -> f64 {
    let mut value = 0.0;
    // SAFETY: value is a live f64
    if unsafe { hal_adc_read_voltage(channel, &mut value) } == rc::OK { value } else { f64::NAN }
}

fn monitors() -> Sample {
    Sample { temp_c: pa_temp(), fwd_w: adc(ADC_CHANNEL_TX_FWD), rev_w: adc(ADC_CHANNEL_TX_REV) }
}

/// Start: nothing on the air, full credit, power select low (0.5 W), the
/// TNC's UART ready (shared with the receiver; initializing it twice is
/// harmless)
#[unsafe(no_mangle)]
pub extern "C" fn downlink_init() -> i32 {
    // SAFETY: no arguments (the HAL)
    let _ = unsafe { c::hal_radio_init() };
    rf_control::gpio_config(GPIO_PIN_TX_PWR_SEL, GPIO_DIR_OUTPUT);
    rf_control::gpio_set(GPIO_PIN_TX_PWR_SEL, false);
    let now = c::now_ms();
    DOWNLINK.with(|d| d.init(now)).map_or(rc::BUSY, |()| rc::OK)
}

/// Hand one AX.25 frame (no FCS) to the TNC if every gate allows it now
#[unsafe(no_mangle)]
pub unsafe extern "C" fn downlink_transmit(frame: *const u8, len: usize) -> i32 {
    if frame.is_null() {
        return rc::NULL_PTR;
    }
    let now = c::now_ms();
    let permitted = rf_control::rf_control_tx_permitted();
    let level = rf_control::rf_control_get_power_level();
    let temp_c = pa_temp();
    let Some(decision) = DOWNLINK.with(|d| d.request(now, len, permitted, level, temp_c)) else { return rc::BUSY };
    let plan = match decision {
        Ok(plan) => plan,
        Err(Refusal::Inhibited) => return rc::ERROR,
        Err(Refusal::Invalid) => return rc::INVALID,
        Err(Refusal::OverTemperature | Refusal::BackOff | Refusal::Busy | Refusal::Duty) => return rc::BUSY,
    };
    rf_control::gpio_set(GPIO_PIN_TX_PWR_SEL, plan.level == POWER_1W);
    // SAFETY: the C contract: frame points to len readable bytes
    let result = unsafe { hal_radio_transmit(frame, len) };
    if result != rc::OK {
        let _ = DOWNLINK.with(|d| d.cancel(plan));
    }
    result
}

/// Each main-loop pass: watch the transmitter while it may be keyed
#[unsafe(no_mangle)]
pub extern "C" fn downlink_poll() {
    let now = c::now_ms();
    let permitted = rf_control::rf_control_tx_permitted();
    let Some(Some(event)) = DOWNLINK.with(|d| d.poll(now, permitted, monitors)) else { return };
    let (level, text) = match event {
        Event::PowerReduced => (severity::WARNING, c"Downlink VSWR above 3:1: power capped at 0.5 W"),
        Event::BackOff => (severity::ERROR, c"Downlink VSWR above 3:1 at 0.5 W: downlink stopped for 10 min"),
        Event::NoOutput => (severity::WARNING, c"Downlink keyed with no forward power"),
        Event::OverTemperature => (severity::WARNING, c"Downlink transmitter over temperature: held"),
        Event::Cooled => (severity::INFO, c"Downlink transmitter cooled: resumed"),
    };
    c::log(FAULT_TYPE_RF_TX, level, text, 0.0);
}

/// Counters and protection state
#[unsafe(no_mangle)]
pub unsafe extern "C" fn downlink_get_status(status: *mut DownlinkStatus) -> i32 {
    // SAFETY: the C contract: NULL or a DownlinkStatus_t
    let Some(out) = (unsafe { ptr::out(status) }) else { return rc::NULL_PTR };
    let now = c::now_ms();
    let Some(s) = DOWNLINK.with(|d| DownlinkStatus {
        counters: d.counters,
        level_cap: d.level_cap(),
        over_temp: d.over_temperature(),
        backoff: d.backed_off(now),
        keyed: d.keyed(now),
    }) else {
        return rc::BUSY;
    };
    *out = s;
    rc::OK
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    /// The transmitter temperature's plausibility: a nested call reads the
    /// sensor as it is; the status for housekeeping
    #[test]
    fn pa_temperature_busy_and_status() {
        let _serial = serial();
        let raw = adc(ADC_CHANNEL_TX_TEMP);
        let nested = PA.with(|_| (pa_temp(), pa_sensor_status()));
        let (t, st) = nested.unwrap_or((f64::NAN, (true, 1)));
        assert!(t == raw || (t.is_nan() && raw.is_nan()), "busy: the raw reading");
        assert_eq!(st, (false, 0), "busy: nothing known");
        let _ = pa_sensor_status();
    }

    #[test]
    fn busy_is_refused() {
        let _serial = serial();
        let frame = [0x41u8; 20];
        let mut status = DownlinkStatus::default();
        let nested = DOWNLINK.with(|_| {
            downlink_poll();
            [
                downlink_init(),
                // SAFETY: frame is 20 readable bytes
                unsafe { downlink_transmit(frame.as_ptr(), frame.len()) },
                // SAFETY: status is a live DownlinkStatus
                unsafe { downlink_get_status(&mut status) },
            ]
        });
        assert_eq!(nested, Some([rc::BUSY; 3]));
        assert_eq!(status, DownlinkStatus::default());
    }

    #[test]
    fn a_failed_conversion_reads_as_not_a_number() {
        let _serial = serial();
        // SAFETY: plain values (the simulated HAL)
        assert_eq!(unsafe { crate::c::hal_adc_init(12, 2) }, rc::OK);
        assert!(adc(ADC_CHANNEL_TX_TEMP).is_finite());
        assert!(adc(99).is_nan());
    }
}
