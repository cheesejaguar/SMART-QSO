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
        // Both calls are made every time (the direction is set again before
        // each level, against an upset); the line is driven if both succeed
        // SAFETY: plain values (the HAL)
        let done = unsafe { [c::hal_gpio_config(pin, 1, 2), c::hal_gpio_set(pin, on)] };
        done == [rc::OK; 2]
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
    let now = c::now_ms();
    let _ = EOL.with(|p| p.verify(now, &mut HalFlash));
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

    /// The HAL failing under the sequence: the record not written, the lines
    /// not driven (the fire ends, the report follows), a corrupt record;
    /// each logged. The sequence is left before its TX-OFF (the inhibit
    /// would hold for the rest of this test process)
    #[test]
    fn hal_failures_are_reported() {
        use sqso_fsw::fault_log::Entry;
        let _serial = serial();
        let logged = |text: &[u8]| {
            (0..crate::fault_mgmt::fault_log_get_count()).any(|i| {
                let mut e = Entry::new(0, 0, 0, b"", 0.0);
                // SAFETY: a live entry
                let got = unsafe { crate::fault_mgmt::fault_log_get_entry(i, &mut e) };
                (got, e.description()) == (rc::OK, text)
            })
        };
        let reset = || {
            assert_eq!(crate::hal_sim::hal_flash_erase(FLASH_REGION_EOL), rc::OK);
            let _ = EOL.with(|p| *p = Passivation::new(FLASH_REGION_EOL));
        };
        assert_eq!(crate::hal_sim::hal_flash_init(), rc::OK);
        assert_eq!(crate::hal_sim::hal_gpio_init(), rc::OK);
        reset();
        assert_eq!(passivation_init(), rc::OK);
        // SAFETY: a live 4-byte buffer
        assert_eq!(unsafe { passivation_arm(b"EOL!".as_ptr(), 4) }, rc::OK);
        crate::hal_sim::hal_sim_flash_fail_writes(true);
        assert_eq!(passivation_execute(), rc::OK);
        crate::hal_sim::hal_sim_flash_fail_writes(false);
        assert!(logged(b"Passivation record not persisted"));
        // The lines cannot be driven (the HAL busy): the fire ends at once
        assert_eq!(crate::hal_sim::holding_the_sim(|| passivation_tick(true)), Some(()));
        assert!(logged(b"Passivation line cannot be driven"));
        assert_eq!(passivation_phase(), 4, "straight to the final report");
        assert_eq!(passivation_take_report(), 1);
        assert_eq!(passivation_attempts(), 1);

        // A record that is neither blank nor valid: its magic, a bad CRC
        reset();
        let mut junk = [0x5Au8; 32];
        junk[..4].copy_from_slice(&sqso_fsw::passivation::RECORD_MAGIC.to_be_bytes());
        // SAFETY: a live buffer of its length
        assert_eq!(unsafe { crate::hal_sim::hal_flash_write(FLASH_REGION_EOL, 0, junk.as_ptr(), junk.len()) }, rc::OK);
        assert_eq!(passivation_init(), rc::OK);
        assert!(logged(b"Passivation record unreadable"));
        assert!(!passivation_passivated());
        reset();
        assert_eq!(passivation_init(), rc::OK);
    }
}
