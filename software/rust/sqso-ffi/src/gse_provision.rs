//! `gse_provision.h`: key provisioning over the GSE umbilical (integration only)
//!
//! @requirement SYS-325 Execute only authenticated, fresh telecommands (47 CFR 97.211(b))

use sqso_protocol::gse::{self, FRAME_SIZE, GseError, GseOp};

use crate::c::{self, fault_type, rc, severity};
use crate::{cmd_auth, ptr};

/// `GseStatus_t`
const STATUS_OK: u32 = 0;
const STATUS_LOCKED: u32 = 1;
const STATUS_BAD_FRAME: u32 = 2;
const STATUS_BAD_OP: u32 = 3;
const STATUS_REJECTED: u32 = 4;

/// HAL values (hal_gpio.h, hal_uart.h)
const GPIO_PIN_UMBILICAL_SENSE: u32 = 10;
const GPIO_DIR_INPUT: u32 = 0;
const GPIO_PULL_DOWN: u32 = 2;
const UART_PORT_DEBUG: u32 = 1;

/// Configure the umbilical sense line and port
#[unsafe(no_mangle)]
pub extern "C" fn gse_init() -> i32 {
    // SAFETY: plain values; NULL selects the default UART configuration
    unsafe {
        match c::hal_gpio_config(GPIO_PIN_UMBILICAL_SENSE, GPIO_DIR_INPUT, GPIO_PULL_DOWN) {
            rc::OK => c::hal_uart_init(UART_PORT_DEBUG, core::ptr::null()),
            other => other,
        }
    }
}

/// Whether the GSE umbilical is connected
#[unsafe(no_mangle)]
pub extern "C" fn gse_umbilical_present() -> bool {
    // SAFETY: plain value
    unsafe { c::hal_gpio_get(GPIO_PIN_UMBILICAL_SENSE) }
}

/// Check and execute one provisioning frame
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gse_process_frame(frame: *const u8, len: usize) -> u32 {
    if !gse_umbilical_present() {
        return STATUS_LOCKED;
    }
    // SAFETY: the C contract: frame has len bytes
    let Some(frame) = (unsafe { ptr::bytes(frame, len) }) else { return STATUS_BAD_FRAME };
    let (opcode, result) = match gse::parse(frame) {
        Err(GseError::BadFrame) => return STATUS_BAD_FRAME,
        Err(GseError::BadOp) => return STATUS_BAD_OP,
        Ok(GseOp::Provision { slot, key_id, key }) => {
            // SAFETY: key.0 is a live KEY_SIZE array
            (gse::op::PROVISION, unsafe { cmd_auth::cmd_auth_provision_slot(slot, &key.0, key_id) })
        }
        Ok(GseOp::Activate { slot }) => (gse::op::ACTIVATE, cmd_auth::rotate(slot)),
        Ok(GseOp::Zeroize) => (gse::op::ZEROIZE, cmd_auth::zeroize()),
        Ok(GseOp::ArmFlight) => {
            // One action arms both: two steps would be two chances to ship a
            // unit that completed its silence or deployment on the bench
            // SAFETY: no arguments
            let silence = unsafe { c::rf_silence_arm() };
            // SAFETY: no arguments
            let deploy = unsafe { c::deployment_arm() };
            // And the commissioning hold: no ACTIVE, payload or transponder
            // until the ground releases it (LV-01)
            let hold = crate::ops_state::ops_set_commissioning_hold(true);
            (gse::op::ARM_FLIGHT, if deploy == rc::OK && hold == rc::OK { silence } else { rc::IO })
        }
    };
    let ok = result == rc::OK;
    let text = if ok { c"GSE key operation done" } else { c"GSE key operation refused" };
    c::log(fault_type::COMMAND, severity::INFO, text, f64::from(opcode));
    if ok { STATUS_OK } else { STATUS_REJECTED }
}

/// Read and answer one frame from the umbilical port
#[unsafe(no_mangle)]
pub extern "C" fn gse_poll() -> bool {
    // SAFETY (this function): HAL calls with local buffers of the stated sizes
    unsafe {
        if !gse_umbilical_present() {
            // Nothing on the umbilical port is honoured without the umbilical
            if c::hal_uart_available(UART_PORT_DEBUG) > 0 {
                let _ = c::hal_uart_flush(UART_PORT_DEBUG);
            }
            return false;
        }
        if c::hal_uart_available(UART_PORT_DEBUG) < FRAME_SIZE {
            return false;
        }
        let mut frame = [0u8; FRAME_SIZE];
        let mut got = 0usize;
        if c::hal_uart_read(UART_PORT_DEBUG, frame.as_mut_ptr(), frame.len(), 0, &mut got) != rc::OK
            || got != FRAME_SIZE
        {
            return false;
        }
        let status = gse_process_frame(frame.as_ptr(), got);
        zeroize::Zeroize::zeroize(&mut frame);
        let reply = gse::reply(status.to_le_bytes()[0]);
        let _ = c::hal_uart_write(UART_PORT_DEBUG, reply.as_ptr(), reply.len());
        true
    }
}
