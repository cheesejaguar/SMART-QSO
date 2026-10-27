//! The EPS controller's supervision registers (`sqso_fsw::eps_supervision`,
//! SQSO-ICD-001 1.6 §4.6): the OBC heartbeat, the AntS supply, the tracker
//! disable and EPS_SUPV, on I2C bus 0 (I2C2) at 0x48.
//!
//! @requirement SRS-F162 OBC heartbeat to the EPS controller (0x8C, 0x5A) every 30 s; EPS_SUPV in HK2; ANTS_PWR as the AntS supply retry path; EPS_DISABLE_TRACKERS writes TRK_DISABLE with its key

use sqso_fsw::eps_supervision::{self as supv, Heartbeat};

use crate::c::{self, fault_type, rc, severity};
use crate::cell::Global;

/// I2C bus 0 (I2C2) and the controller
const EPS_BUS: u32 = 0;
const EPS_ADDRESS: u8 = sqso_fsw::eps_telemetry::ADDRESS;

static HEARTBEAT: Global<Heartbeat> = Global::new(Heartbeat::new());

fn write(bytes: &[u8]) -> bool {
    // SAFETY: bytes is a live slice (the HAL)
    unsafe { c::hal_i2c_write(EPS_BUS, EPS_ADDRESS, bytes.as_ptr(), bytes.len()) == rc::OK }
}

/// Boot: the first heartbeat at the next pass
pub(crate) fn init() {
    let _ = HEARTBEAT.with(|h| *h = Heartbeat::new());
}

/// Each pass: the heartbeat when it is due (RID SMA-B09)
pub(crate) fn step(now_ms: u64) {
    if !HEARTBEAT.with(|h| h.due(now_ms)).unwrap_or(false) {
        return;
    }
    let acked = write(&[supv::REG_HEARTBEAT, supv::HEARTBEAT_VALUE]);
    let _ = HEARTBEAT.with(|h| h.written(now_ms, acked));
}

/// The AntS supply through the controller (ANTS_PWR, RID SMA-B06): on for
/// at most 30 min, or off; whether the controller took it
pub(crate) fn ants_power(on: bool) -> bool {
    write(&[supv::REG_ANTS_PWR, if on { supv::ANTS_PWR_ON } else { supv::ANTS_PWR_OFF }])
}

/// EPS_DISABLE_TRACKERS: the trackers in `mask` (bits 0-3, T1-T4) off, the
/// others on; `INVALID` for other bits, `IO` if the controller refuses
pub(crate) fn disable_trackers(mask: u8) -> i32 {
    let Some(w) = supv::trk_disable(mask) else { return rc::INVALID };
    if !write(&w) {
        return rc::IO;
    }
    c::log(fault_type::COMMAND, severity::WARNING, c"EPS trackers disabled by command (TRK_DISABLE)", 0.0);
    rc::OK
}

/// EPS_SUPV, if the controller answers
pub(crate) fn supv_register() -> Option<u8> {
    let reg = [supv::REG_SUPV];
    let mut v = [0u8; 1];
    // SAFETY: reg and v are live buffers (the HAL)
    let ok = unsafe { c::hal_i2c_write_read(EPS_BUS, EPS_ADDRESS, reg.as_ptr(), 1, v.as_mut_ptr(), 1) } == rc::OK;
    ok.then_some(v[0])
}

/// HK2 `SV`: EPS_SUPV (`None`: no answer) and the refused heartbeats
pub(crate) fn status() -> (Option<u8>, u32) {
    (supv_register(), HEARTBEAT.with(|h| h.failures).unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fault_mgmt::tests::serial;

    fn reg(r: u8) -> Option<u8> {
        let mut v = [0u8; 1];
        // SAFETY: live buffers (the HAL)
        let ok = unsafe { c::hal_i2c_write_read(EPS_BUS, EPS_ADDRESS, [r].as_ptr(), 1, v.as_mut_ptr(), 1) } == rc::OK;
        ok.then_some(v[0])
    }

    #[test]
    fn heartbeat_ants_power_and_trackers() {
        let _serial = serial();
        assert_eq!(crate::hal_sim::hal_i2c_init(EPS_BUS, 100_000), rc::OK);
        init();
        step(1_000);
        assert_eq!(reg(supv::REG_HEARTBEAT), Some(supv::HEARTBEAT_VALUE));
        assert!(write(&[supv::REG_HEARTBEAT, 0]));
        step(1_000 + supv::HEARTBEAT_PERIOD_MS - 1);
        assert_eq!(reg(supv::REG_HEARTBEAT), Some(0), "not yet due");
        step(1_000 + supv::HEARTBEAT_PERIOD_MS);
        assert_eq!(reg(supv::REG_HEARTBEAT), Some(supv::HEARTBEAT_VALUE));
        assert!(ants_power(true));
        assert_eq!(reg(supv::REG_ANTS_PWR), Some(supv::ANTS_PWR_ON));
        assert!(ants_power(false));
        assert_eq!(reg(supv::REG_ANTS_PWR), Some(supv::ANTS_PWR_OFF));
        assert_eq!(disable_trackers(0x10), rc::INVALID);
        assert_eq!(disable_trackers(0b0011), rc::OK);
        assert_eq!(reg(supv::REG_TRK_DISABLE), Some(supv::TRK_KEY));
        assert_eq!(status(), (reg(supv::REG_SUPV), 0));
        // A controller that refuses: counted, retried at the next pass
        crate::hal_sim::I2C_FAIL_REG
            .store(u32::from(EPS_ADDRESS) << 8 | u32::from(supv::REG_HEARTBEAT), core::sync::atomic::Ordering::Relaxed);
        step(10 * supv::HEARTBEAT_PERIOD_MS);
        crate::hal_sim::I2C_FAIL_REG.store(
            u32::from(EPS_ADDRESS) << 8 | u32::from(supv::REG_TRK_DISABLE),
            core::sync::atomic::Ordering::Relaxed,
        );
        assert_eq!(disable_trackers(1), rc::IO);
        crate::hal_sim::I2C_FAIL_REG.store(0, core::sync::atomic::Ordering::Relaxed);
        assert_eq!(status().1, 1);
        step(10 * supv::HEARTBEAT_PERIOD_MS + 20);
        assert_eq!(status().1, 1);
    }
}
