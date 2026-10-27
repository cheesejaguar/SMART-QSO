//! The ISIS AntS deployable antenna on its own I2C bus, I2C3 (bus 3:
//! PG7 SCL, PB4 SDA, 100 kHz; SQSO-ICX-001 D §6 row 30, RID SMA-B07), through
//! its two redundant controllers (`sqso_fsw::ants`): side A (0x31) first,
//! side B (0x32) when side A does not acknowledge, and the sides taken in
//! turn from one START to the next, so a retry of an antenna that stayed
//! stowed goes through the other controller. When neither side answers,
//! the EPS controller's ANTS_PWR raises the AntS supply for the retry.

use core::sync::atomic::{AtomicU8, Ordering};

use sqso_fsw::ants::{self, Status};

use crate::c::{self, rc};

/// I2C bus 3 (I2C3): the AntS alone
pub(crate) const ANTS_I2C_BUS: u32 = 3;

/// The two controllers
const SIDES: [u8; 2] = [ants::ADDRESS, ants::ADDRESS_B];

/// The side the next START tries first, and the side last commanded
static NEXT: AtomicU8 = AtomicU8::new(0);
static LAST: AtomicU8 = AtomicU8::new(0);

fn send(addr: u8, writes: &[&[u8]]) -> bool {
    writes.iter().fold(true, |ok, w| {
        // SAFETY: w is a live slice (the HAL)
        let sent = unsafe { c::hal_i2c_write(ANTS_I2C_BUS, addr, w.as_ptr(), w.len()) } == rc::OK;
        ok && sent
    })
}

fn side(i: u8) -> u8 {
    SIDES.get(usize::from(i & 1)).copied().unwrap_or(ants::ADDRESS)
}

/// Start (arm, then deploy every element in turn) or stop (cancel, then
/// disarm) the AntS's own deployment; whether one side acknowledged every
/// write. A START tries this attempt's side first, then the other; a STOP
/// goes to both
pub(crate) fn command(start: bool) -> bool {
    if !start {
        let a = send(side(0), &ants::STOP);
        let b = send(side(1), &ants::STOP);
        let _ = crate::eps_supervision::ants_power(false);
        return a || b;
    }
    let first = NEXT.fetch_xor(1, Ordering::Relaxed) & 1;
    for i in [first, first ^ 1] {
        if send(side(i), &ants::START) {
            LAST.store(i, Ordering::Relaxed);
            return true;
        }
    }
    // Neither side answers with DEPLOY_EN high: its supply may be what
    // failed (SW8's enable); the EPS controller raises it too (ANTS_PWR,
    // RID SMA-B06), and the sequence's retry finds the AntS powered
    let _ = crate::eps_supervision::ants_power(true);
    false
}

/// The AntS's status, if either side answers (the side last commanded first)
pub fn status() -> Option<Status> {
    let last = LAST.load(Ordering::Relaxed) & 1;
    [last, last ^ 1].into_iter().find_map(|i| {
        let mut bytes = [0u8; 2];
        let reg = [ants::CMD_STATUS];
        // SAFETY: reg and bytes are live buffers of their lengths (the HAL)
        let ok = unsafe {
            c::hal_i2c_write_read(ANTS_I2C_BUS, side(i), reg.as_ptr(), reg.len(), bytes.as_mut_ptr(), bytes.len())
        } == rc::OK;
        ok.then(|| Status::decode(bytes))
    })
}

/// Boot: side A first
pub(crate) fn reset() {
    NEXT.store(0, Ordering::Relaxed);
    LAST.store(0, Ordering::Relaxed);
}
