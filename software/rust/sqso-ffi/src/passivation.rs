//! `passivation.h`: end-of-life passivation by telecommand
//! (`sqso_fsw::passivation`) on the GPIO, flash and I2C HAL and
//! `rf_control`.
//!
//! The two lines of the EPS's charge disconnect are `GPIO_PIN_EOL_ARM` (PF4)
//! and `GPIO_PIN_EOL_FIRE` (PG1). The EPS controller's `EOL_STATE` register
//! (0x7C, SMART-QSO-ICD-001 §4.6: bit 0 armed, bit 1 fired (charge path
//! open), bit 2 fuse intact) confirms the disconnect and goes into the final
//! report (`HK2`), which the executive sends when [`passivation_take_report`]
//! says so; TX-OFF follows it (RID REG-12).
//!
//! Coverage: only a nested call finds `EOL` in use (busy: `Global::with`
//! gives `None`): the answers are then the safe ones (passivated reads as
//! passivated only if it was; commands refused as busy).
//!
//! @requirement SRS-F110 Two-step authenticated end-of-life passivation
//! @requirement SRS-F111 Passivation order: record, charge disconnect through two series lines while transmitting, final report, then TX-OFF
//! @requirement SRS-F155 The final passivation report: three frames over three minutes carrying EOL_STATE, the fuse state and the attempts, before TX-OFF
//! @requirement SRS-F112 Transmitters off for good after passivation

use sqso_fsw::passivation::{Line, Passivation, PassivationIo, Refusal, Report};
use sqso_fsw::rf_control::INHIBIT_PASSIVATED;

use crate::c::{self, HalFlash, fault_type, rc, severity};
use crate::cell::Global;
use crate::ptr;

/// `FLASH_REGION_EOL`
pub(crate) const FLASH_REGION_EOL: u32 = 15;
/// `GPIO_PIN_EOL_ARM`
pub(crate) const GPIO_PIN_EOL_ARM: u32 = 19;
/// `GPIO_PIN_EOL_FIRE`
pub(crate) const GPIO_PIN_EOL_FIRE: u32 = 20;
/// The EPS controller (SMART-QSO-ICD-001)
const EPS_BUS: u32 = 0;
const EPS_ADDRESS: u8 = 0x48;
/// `EOL_STATE`: bit 0 armed, bit 1 fired (charge path open)
const REG_EOL_STATE: u8 = 0x7C;
const EOL_FIRED: u8 = 0x02;

static EOL: Global<Passivation> = Global::new(Passivation::new(FLASH_REGION_EOL));

impl PassivationIo for HalFlash {
    fn tx_off(&mut self) {
        let _ = crate::rf_control::rf_control_tx_off();
    }

    fn inhibit(&mut self) {
        crate::rf_control::rf_control_set_inhibit(INHIBIT_PASSIVATED, true);
    }

    fn drive(&mut self, line: Line, on: bool) -> bool {
        let pin = match line {
            Line::Arm => GPIO_PIN_EOL_ARM,
            Line::Fire => GPIO_PIN_EOL_FIRE,
        };
        // SAFETY: plain values (the HAL)
        unsafe { c::hal_gpio_config(pin, 1, 2) == rc::OK && c::hal_gpio_set(pin, on) == rc::OK }
    }

    fn charge_path_open(&mut self) -> Option<bool> {
        eol_state().map(|v| v & EOL_FIRED != 0)
    }

    fn report(&mut self, report: Report) {
        let (sev, text) = match report {
            Report::Armed => (severity::WARNING, c"Passivation armed (600 s)"),
            Report::Lapsed => (severity::INFO, c"Passivation arm lapsed"),
            Report::Executed => (severity::CRITICAL, c"Passivation executed: charge disconnect firing"),
            Report::NotPersisted => (severity::ERROR, c"Passivation record not persisted"),
            Report::LineUnavailable => (severity::ERROR, c"Passivation line cannot be driven"),
            Report::ChargePathOpen => (severity::CRITICAL, c"Passivation: EPS reports charge path open"),
            Report::NotConfirmed => (severity::ERROR, c"Passivation fire ended, not confirmed by EPS"),
            Report::Unreadable => (severity::ERROR, c"Passivation record unreadable"),
            Report::Passivated => (severity::INFO, c"Passivated: transmitters inhibited"),
            Report::Silenced => (severity::CRITICAL, c"Passivation reported: TX-OFF, transmitters off for good"),
        };
        c::log(fault_type::COMMAND, sev, text, 0.0);
    }
}

/// Load the record and apply it before anything can transmit
#[unsafe(no_mangle)]
pub extern "C" fn passivation_init() -> i32 {
    let now = c::now_ms();
    EOL.with(|p| {
        p.init(now, &mut HalFlash);
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

fn code(r: Result<(), Refusal>) -> i32 {
    match r {
        Ok(()) => rc::OK,
        Err(Refusal::BadCode) => rc::INVALID,
        Err(Refusal::NotArmed | Refusal::Passivated) => rc::ERROR,
    }
}

/// PASSIVATE_ARM with its argument (`len` bytes at `arm_code`)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn passivation_arm(arm_code: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: arm_code has len bytes
    let Some(bytes) = (unsafe { ptr::bytes(arm_code, len) }) else { return rc::NULL_PTR };
    let now = c::now_ms();
    EOL.with(|p| code(p.arm(bytes, now, &mut HalFlash))).unwrap_or(rc::BUSY)
}

/// PASSIVATE_EXECUTE
#[unsafe(no_mangle)]
pub extern "C" fn passivation_execute() -> i32 {
    let now = c::now_ms();
    EOL.with(|p| code(p.execute(now, &mut HalFlash))).unwrap_or(rc::BUSY)
}

/// Every pass: the arm's window and the fire sequence
#[unsafe(no_mangle)]
pub extern "C" fn passivation_tick(sunlit: bool) {
    let now = c::now_ms();
    let _ = EOL.with(|p| p.tick(now, sunlit, &mut HalFlash));
}

/// Check the RAM copy against the record (AV-07)
#[unsafe(no_mangle)]
pub extern "C" fn passivation_verify() {
    let _ = EOL.with(|p| p.verify(&mut HalFlash));
}

/// The next final-report frame, if one is due now (1 to 3; 0: none)
#[unsafe(no_mangle)]
pub extern "C" fn passivation_take_report() -> u8 {
    let now = c::now_ms();
    EOL.with(|p| p.take_report(now)).flatten().unwrap_or(0)
}

/// The fire attempts so far
#[unsafe(no_mangle)]
pub extern "C" fn passivation_attempts() -> u8 {
    EOL.with(|p| p.attempts()).unwrap_or(0)
}

/// The EPS controller's `EOL_STATE` (`None`: no answer)
pub fn eol_state() -> Option<u8> {
    let reg = [REG_EOL_STATE];
    let mut v = [0u8];
    // SAFETY: both are live one-byte buffers
    let read = unsafe { c::hal_i2c_write_read(EPS_BUS, EPS_ADDRESS, reg.as_ptr(), 1, v.as_mut_ptr(), 1) };
    (read == rc::OK).then_some(v[0])
}

/// `PassivationPhase_t`: 0 idle, 1 armed, 2 firing, 3 passivated, 4 reporting
#[unsafe(no_mangle)]
pub extern "C" fn passivation_phase() -> u32 {
    let now = c::now_ms();
    EOL.with(|p| p.phase(now) as u32).unwrap_or(0)
}

/// Whether the spacecraft is passivated (silenced: transmitters off for good)
#[unsafe(no_mangle)]
pub extern "C" fn passivation_passivated() -> bool {
    EOL.with(|p| p.passivated()).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    #[test]
    fn busy_refuses() {
        let _serial = serial();
        let nested = EOL.with(|_| {
            (
                passivation_init(),
                // SAFETY: a live 4-byte buffer
                unsafe { passivation_arm(b"EOL!".as_ptr(), 4) },
                passivation_execute(),
                passivation_phase(),
                passivation_passivated(),
            )
        });
        assert_eq!(nested, Some((rc::BUSY, rc::BUSY, rc::BUSY, 0, false)));
        assert_eq!(EOL.with(|_| (passivation_take_report(), passivation_attempts())), Some((0, 0)));
        let _ = EOL.with(|_| {
            passivation_tick(true);
            passivation_verify();
        });
        // SAFETY: NULL is refused
        assert_eq!(unsafe { passivation_arm(core::ptr::null(), 4) }, rc::NULL_PTR);
        assert_eq!(code(Err(Refusal::NotArmed)), rc::ERROR);
    }
}
