//! `mission_data.h`: mission statistics across resets
//! (`sqso_fsw::mission_data`), stored through [`crate::store`]
//! (`MISSION_DATA_FILE`, or `$SMART_QSO_MISSION_DATA`, on the host; its
//! flash region on the target). The stored bytes are the C `MissionData_t`.
//!
//! @requirement SRS-F060 Persist mission data to non-volatile memory
//! @requirement SRS-F061 Protect persisted data with CRC32
//! @requirement SRS-F063 Maintain mission statistics across resets

use core::ffi::c_char;

use sqso_fsw::mission_data::{MissionData, Phase, RECORD_SIZE};

use crate::c::{self, rc};
use crate::cell::Global;
use crate::ptr;
use crate::store::{self, Store};

struct Mission {
    data: MissionData,
    initialized: bool,
}

static MISSION: Global<Mission> = Global::new(Mission { data: MissionData::ZERO, initialized: false });

fn with<R>(busy: R, f: impl FnOnce(&mut Mission) -> R) -> R {
    MISSION.with(f).unwrap_or(busy)
}

/// Change the data and its CRC
fn update(f: impl FnOnce(&mut MissionData)) -> i32 {
    with(rc::BUSY, |m| {
        f(&mut m.data);
        m.data.update_crc();
        rc::OK
    })
}

/// Initialize: load the stored data and count this reset, or start a new
/// mission
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_init() -> i32 {
    let loaded = mission_data_load() == rc::OK;
    let now = c::now_ms();
    let resets = with(None, |m| {
        if loaded {
            m.data.count_reset(now);
        } else {
            m.data.start_new(now);
        }
        m.initialized = true;
        Some(m.data.reset_count)
    });
    match (loaded, resets) {
        (_, None) => return rc::BUSY,
        (true, Some(n)) => c::console(format_args!("[MISSION] Loaded mission data (reset #{n})")),
        (false, Some(_)) => c::console(format_args!("[MISSION] Initialized new mission data")),
    }
    rc::OK
}

/// Copy of the mission data
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mission_data_get(data: *mut MissionData) -> i32 {
    // SAFETY: the C contract: data points to a MissionData_t (same layout)
    let Some(out) = (unsafe { ptr::out(data) }) else {
        c::assert_failed(c"Null pointer: data", line!());
        return rc::NULL_PTR;
    };
    match MISSION.with(|m| m.initialized.then_some(m.data)) {
        Some(Some(d)) => {
            *out = d;
            rc::OK
        }
        Some(None) => rc::ERROR,
        None => rc::BUSY,
    }
}

/// Set the mission start time
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_set_start(start_ms: u64) -> i32 {
    update(|d| d.mission_start_ms = start_ms)
}

/// Set the total uptime
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_update_uptime(uptime_ms: u64) -> i32 {
    update(|d| d.total_uptime_ms = uptime_ms)
}

/// Count a reset now
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_increment_reset() -> i32 {
    let now = c::now_ms();
    with(rc::BUSY, |m| {
        m.data.count_reset(now);
        rc::OK
    })
}

/// Add consumed energy, Wh
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_add_energy(energy_wh: f64) -> i32 {
    update(|d| d.total_energy_wh += energy_wh)
}

/// Set the mission phase
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_set_phase(phase: u32) -> i32 {
    let Some(phase) = Phase::from_raw(phase) else {
        c::assert_failed(c"Invalid mission phase", line!());
        return rc::INVALID;
    };
    update(|d| d.mission_phase = phase as u8)
}

/// Current mission phase (the stored byte)
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_get_phase() -> u32 {
    with(0, |m| u32::from(m.data.mission_phase))
}

/// Count a fault and keep its description (cut at 63 bytes)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mission_data_record_fault(description: *const c_char) -> i32 {
    // SAFETY: the C contract: a C string
    let Some(description) = (unsafe { ptr::cstr(description) }) else {
        c::assert_failed(c"Null pointer: description", line!());
        return rc::NULL_PTR;
    };
    with(rc::BUSY, |m| {
        m.data.record_fault(description);
        rc::OK
    })
}

/// Persist the mission data
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_save() -> i32 {
    let Some(bytes) = MISSION.with(|m| {
        m.data.update_crc();
        m.data.to_bytes()
    }) else {
        return rc::BUSY;
    };
    store::save(Store::MissionData, &bytes)
}

/// Load the stored mission data; a CRC mismatch keeps the current data
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_load() -> i32 {
    let mut bytes = [0u8; RECORD_SIZE];
    match store::load(Store::MissionData, &mut bytes) {
        Ok(RECORD_SIZE) => {}
        Ok(_) => return rc::IO,
        Err(e) => return e,
    }
    let Ok(loaded) = MissionData::from_bytes(&bytes) else {
        c::console(format_args!("[MISSION] CRC mismatch in mission data, using defaults"));
        return rc::ERROR;
    };
    with(rc::BUSY, |m| {
        m.data = loaded;
        rc::OK
    })
}

/// Number of resets
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_get_reset_count() -> u32 {
    with(0, |m| m.data.reset_count)
}

/// Total uptime, ms
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_get_uptime() -> u64 {
    with(0, |m| m.data.total_uptime_ms)
}

/// Total energy consumed, Wh
#[unsafe(no_mangle)]
pub extern "C" fn mission_data_get_energy() -> f64 {
    with(0.0, |m| m.data.total_energy_wh)
}
