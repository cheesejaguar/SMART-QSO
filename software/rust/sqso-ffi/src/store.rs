//! Persistent blobs: the fault log, EPS configuration, mission data and
//! system state.
//!
//! On the host each is a file (an environment variable can override the
//! path, so parallel tests do not share one). On the target each is a flash
//! region holding a 4-byte length and the data.
//!
//! @requirement SRS-F041 Maintain the fault log in non-volatile memory

#[cfg(not(target_os = "none"))]
use core::ffi::CStr;

#[cfg(not(target_os = "none"))]
use crate::c::rc;

/// What is stored
#[allow(dead_code)] // callers arrive as the remaining C modules are ported
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Store {
    /// The fault log (`fault_mgmt.h`)
    FaultLog,
    /// EPS configuration (`eps_control.h`)
    EpsConfig,
    /// Mission data (`mission_data.h`)
    MissionData,
    /// System state snapshot (`system_state.h`)
    SystemState,
}

#[cfg(not(target_os = "none"))]
impl Store {
    /// The file: `$<env>` if set, else the default path
    fn path(self) -> &'static CStr {
        let (env, default) = match self {
            Store::FaultLog => (Some(c"SMART_QSO_FAULT_LOG"), c"/tmp/fault_log.dat"),
            Store::EpsConfig => (Some(c"SMART_QSO_EPS_CONFIG"), c"/tmp/eps_config.dat"),
            Store::MissionData => (Some(c"SMART_QSO_MISSION_DATA"), c"/tmp/mission_data.dat"),
            Store::SystemState => (None, c"/tmp/smart_qso_state.dat"),
        };
        env.and_then(crate::os::env).unwrap_or(default)
    }
}

/// The largest blob the store holds (no limit for a file)
#[cfg(not(target_os = "none"))]
pub(crate) fn capacity(_store: Store) -> usize {
    usize::MAX
}

/// Replace the stored blob
#[cfg(not(target_os = "none"))]
pub(crate) fn save(store: Store, data: &[u8]) -> i32 {
    if crate::os::file_write(store.path(), data) { rc::OK } else { rc::IO }
}

/// Read the stored blob into `buf`; the bytes read
#[cfg(not(target_os = "none"))]
pub(crate) fn load(store: Store, buf: &mut [u8]) -> Result<usize, i32> {
    crate::os::file_read(store.path(), buf).ok_or(rc::IO)
}

/// Remove the stored blob
#[cfg(not(target_os = "none"))]
pub(crate) fn erase(store: Store) {
    crate::os::file_remove(store.path());
}

#[cfg(target_os = "none")]
mod flash {
    use super::Store;
    use crate::c::{self, rc};

    /// `HalFlashRegion_t`
    fn region(store: Store) -> u32 {
        match store {
            Store::MissionData => 0,
            Store::EpsConfig => 1,
            Store::FaultLog => 3,
            Store::SystemState => 5,
        }
    }

    pub(crate) fn capacity(store: Store) -> usize {
        // SAFETY: plain value
        unsafe { c::hal_flash_region_size(region(store)) }.saturating_sub(4)
    }

    pub(crate) fn save(store: Store, data: &[u8]) -> i32 {
        let region = region(store);
        if data.len() > capacity(store) {
            return rc::INVALID;
        }
        let header = u32::try_from(data.len()).unwrap_or(0).to_le_bytes();
        let body = data;
        // SAFETY: header and body are live slices
        unsafe {
            match c::hal_flash_write(region, 0, header.as_ptr(), header.len()) {
                rc::OK => c::hal_flash_write(region, 4, body.as_ptr(), body.len()),
                other => other,
            }
        }
    }

    pub(crate) fn load(store: Store, buf: &mut [u8]) -> Result<usize, i32> {
        let region = region(store);
        let mut header = [0u8; 4];
        // SAFETY: header is a live 4-byte buffer
        match unsafe { c::hal_flash_read(region, 0, header.as_mut_ptr(), 4) } {
            rc::OK => {}
            other => return Err(other),
        }
        let stored = usize::try_from(u32::from_le_bytes(header)).unwrap_or(usize::MAX);
        if stored > capacity(store) {
            return Err(rc::IO); // erased (0xFFFFFFFF) or corrupt
        }
        let len = stored.min(buf.len());
        // SAFETY: buf is live and at least len bytes
        match unsafe { c::hal_flash_read(region, 4, buf.as_mut_ptr(), len) } {
            rc::OK => Ok(len),
            other => Err(other),
        }
    }

    pub(crate) fn erase(store: Store) {
        // SAFETY: plain value
        let _ = unsafe { c::hal_flash_erase(region(store)) };
    }
}

#[cfg(target_os = "none")]
pub(crate) use flash::{capacity, erase, load, save};
