//! `receiver.h`: supervision of the 435 MHz command receiver
//! (`sqso_fsw::receiver`) on the GPIO HAL, the uplink statistics and
//! `rf_control`.
//!
//! Inputs: `GPIO_PIN_RX_CD` (PG0, carrier detect, high on a carrier) and
//! `GPIO_PIN_RX_FAULT` (PF13, the receiver's health line, high on a fault),
//! from the radio board (SQSO-RF-BRD-001 B §4.1), both qualified with
//! `GPIO_PIN_RADIO_EN` and the fault line ignored for 1 s after it switches
//! on (`sqso_fsw::receiver::Receiver::qualify`, RID RFB-07); valid frames
//! from `uplink_get_stats()`. A recovery cycle is `RF_INHIBIT_RX_CYCLE`.
//!
//! @requirement SRS-F152 RX_CD and RX_FAULT qualified with RADIO_EN; RX_FAULT ignored for 1.5 s after RADIO_EN switches on
//!
//! @requirement SRS-F115 Hung-receiver recovery by a brief power cycle, never during a pass
//! @requirement SRS-F116 Evidence that the command receiver has failed

use sqso_fsw::receiver::{Action, Inputs, QUERY_PERIOD_MS, Receiver};
use sqso_fsw::rf_control::INHIBIT_RX_CYCLE;
use sqso_protocol::kiss::TncStatus;

use crate::c::{self, fault_type, severity};
use crate::cell::Global;

/// `GPIO_PIN_RX_CD`
pub(crate) const GPIO_PIN_RX_CD: u32 = 21;
/// `GPIO_PIN_RX_FAULT`
pub(crate) const GPIO_PIN_RX_FAULT: u32 = 22;
const GPIO_DIR_INPUT: u32 = 0;
/// No internal pull: the OBC board's 100 kΩ pull-downs hold the lines; an
/// internal pull-down too would load the RF board's drive through its 10 kΩ
const GPIO_PULL_NONE: u32 = 0;

struct State {
    rx: Receiver,
    /// Valid frames the uplink had counted at the last pass
    valid_frames: u32,
    /// The last STATUS query, ms
    last_query_ms: Option<u64>,
    /// The TNC's last STATUS answer (HK2 `TS`)
    status: Option<TncStatus>,
}

static RX: Global<State> =
    Global::new(State { rx: Receiver::new(0), valid_frames: 0, last_query_ms: None, status: None });

/// The STATUS loop query (RF board E §8.6): sent every
/// `QUERY_PERIOD_MS` with the radio on; whether an answer came
fn loop_query(now: u64, radio_en: bool) -> bool {
    let mut buf = [0u8; sqso_protocol::kiss::HW_MAX];
    let mut n = 0usize;
    // SAFETY: buf and n are live (the HAL)
    let answer = (unsafe { c::hal_radio_take_status(buf.as_mut_ptr(), buf.len(), &mut n) } == crate::c::rc::OK)
        .then(|| TncStatus::decode(buf.get(..n).unwrap_or(&[])))
        .flatten();
    let due = RX
        .with(|s| {
            if answer.is_some() {
                s.status = answer;
            }
            let due = radio_en && s.last_query_ms.is_none_or(|t| now.saturating_sub(t) >= QUERY_PERIOD_MS);
            if due {
                s.last_query_ms = Some(now);
            }
            if !radio_en {
                s.last_query_ms = None;
            }
            due
        })
        .unwrap_or(false);
    if due {
        // SAFETY: no arguments (the HAL)
        let _ = unsafe { c::hal_radio_status_query() };
    }
    answer.is_some()
}

fn valid_frames() -> u32 {
    let mut stats = crate::uplink::UplinkStats::default();
    // SAFETY: stats is a live UplinkStats_t
    unsafe { crate::uplink::uplink_get_stats(&mut stats) };
    stats.frames.wrapping_sub(stats.rejected)
}

/// Start counting from now (boot)
#[unsafe(no_mangle)]
pub extern "C" fn receiver_init() {
    for pin in [GPIO_PIN_RX_CD, GPIO_PIN_RX_FAULT] {
        // SAFETY: plain values (the HAL)
        let _ = unsafe { c::hal_gpio_config(pin, GPIO_DIR_INPUT, GPIO_PULL_NONE) };
    }
    let now = c::now_ms();
    let frames = valid_frames();
    let _ = RX.with(|s| {
        s.rx = Receiver::new(now);
        s.valid_frames = frames;
        s.last_query_ms = None;
        s.status = None;
    });
}

/// One pass (`keyed`: the downlink or the transponder is keyed)
#[unsafe(no_mangle)]
pub extern "C" fn receiver_step(keyed: bool) {
    let now = c::now_ms();
    let frames = valid_frames();
    // SAFETY: plain values (the HAL)
    let (carrier, fault, radio_en) = unsafe {
        (c::hal_gpio_get(GPIO_PIN_RX_CD), c::hal_gpio_get(GPIO_PIN_RX_FAULT), c::hal_gpio_get(crate::pins::RADIO_EN))
    };
    let powered = !crate::rf_silence::rf_silence_active();
    let contact_lost = crate::contact_monitor::contact_monitor_inhibited();
    let answered = loop_query(now, radio_en);
    let action = RX.with(|s| {
        let frame = frames != s.valid_frames;
        s.valid_frames = frames;
        let raw = Inputs {
            powered: powered && !s.rx.cycling(),
            frame,
            carrier,
            fault,
            keyed,
            contact_lost,
            radio_en,
            answered,
        };
        let inputs = s.rx.qualify(now, &raw);
        let was_failed = s.rx.failed();
        let action = s.rx.step(now, &inputs);
        (action, !was_failed && s.rx.failed())
    });
    let Some((action, newly_failed)) = action else { return };
    match action {
        Action::None => {}
        Action::PowerOff => {
            crate::rf_control::rf_control_set_inhibit(INHIBIT_RX_CYCLE, true);
            c::log(fault_type::RECOVERY, severity::WARNING, c"Receiver silent 24 h: power cycled", 0.0);
        }
        Action::PowerOn => crate::rf_control::rf_control_set_inhibit(INHIBIT_RX_CYCLE, false),
    }
    if newly_failed {
        c::log(fault_type::RECOVERY, severity::CRITICAL, c"Receiver known failed: beacon kept (REL-04)", 0.0);
    }
}

/// Whether the receiver is known failed (REL-04 evidence)
#[unsafe(no_mangle)]
pub extern "C" fn receiver_failed() -> bool {
    RX.with(|s| s.rx.failed()).unwrap_or(false)
}

/// HK2 `TS` and `RL`: the TNC's last STATUS answer, whether RX_FAULT passed
/// its power-on self-test, whether the loop query has gone unanswered for a
/// day
pub(crate) fn tnc_status() -> (Option<TncStatus>, bool, bool) {
    let now = c::now_ms();
    RX.with(|s| (s.status, s.rx.line_usable(), s.rx.path_fault(now))).unwrap_or((None, true, false))
}

/// Recovery power cycles since boot
#[unsafe(no_mangle)]
pub extern "C" fn receiver_cycles() -> u32 {
    RX.with(|s| s.rx.cycles).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_reads_healthy() {
        // The other tests use the HAL and these globals: one at a time
        let _serial = crate::fault_mgmt::tests::serial();
        let nested = RX.with(|_| {
            receiver_init();
            receiver_step(false);
            (receiver_failed(), receiver_cycles())
        });
        assert_eq!(nested, Some((false, 0)));
    }
}
