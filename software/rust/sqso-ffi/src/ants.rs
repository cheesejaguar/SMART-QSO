//! The ISIS AntS deployable antenna on I2C bus 0 (`sqso_fsw::ants`): the
//! deployment sequence's antenna "burn" and its per-element sense.

use sqso_fsw::ants::{self, Status};

use crate::c::{self, rc};

/// I2C bus 0: the EPS controller, the power monitors and the AntS
pub(crate) const ANTS_I2C_BUS: u32 = 0;

/// Start (arm, then deploy every element in turn) or stop (cancel, then
/// disarm) the AntS's own deployment; whether every write was acknowledged
pub(crate) fn command(start: bool) -> bool {
    let writes = if start { ants::START } else { ants::STOP };
    writes.iter().fold(true, |ok, w| {
        // SAFETY: w is a live slice (the HAL)
        let sent = unsafe { c::hal_i2c_write(ANTS_I2C_BUS, ants::ADDRESS, w.as_ptr(), w.len()) } == rc::OK;
        ok && sent
    })
}

/// The AntS's status, if it answers
pub fn status() -> Option<Status> {
    let mut bytes = [0u8; 2];
    let reg = [ants::CMD_STATUS];
    // SAFETY: reg and bytes are live buffers of their lengths (the HAL)
    let ok = unsafe {
        c::hal_i2c_write_read(ANTS_I2C_BUS, ants::ADDRESS, reg.as_ptr(), reg.len(), bytes.as_mut_ptr(), bytes.len())
    } == rc::OK;
    ok.then(|| Status::decode(bytes))
}
