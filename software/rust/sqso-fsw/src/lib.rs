//! SMART-QSO flight logic, independent of the C runtime.
//!
//! Each module is the decision logic of one flight module, with its I/O
//! behind a trait (flash) or returned to the caller (log events), so it can
//! be tested and fuzzed on the host. `sqso-ffi` connects it to the C flight
//! software through the existing C headers.
//!
//! - [`nvm`]: power-fail-safe two-slot records (`nvm_record.h`)
//! - [`keys`]: telecommand key table and replay counters (`cmd_auth.h`)
//! - [`telecommand`]: frame to authenticated command (`cmd_handler.h`)
//! - [`mode`]: the guarded mode manager (`state_machine.h`, SW-14)
//! - [`scheduler`]: time-triggered task table (`scheduler.h`)
//! - [`fault_log`]: fault log ring with per-entry CRC (`fault_mgmt.h`)
//! - [`system_state`]: the system state record and its image (`system_state.h`)
//! - [`telemetry`]: binary telemetry frames and schedule (`telemetry.h`)
//! - [`beacon`]: beacon text, telemetry fields and AX.25 frame (`beacon.h`)
//! - [`adcs`]: B-dot detumble, sun pointing and ADCS modes (`adcs_control.h`)
//! - [`watchdog`]: the supervised watchdog's task table (`watchdog_mgr.h`)
//! - [`fmath`]: C `sqrt` and `round` for `no_std`

#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic))]

pub mod adcs;
pub mod beacon;
pub mod fault_log;
pub mod fmath;
pub mod keys;
pub mod mode;
pub mod nvm;
pub mod scheduler;
pub mod system_state;
pub mod telecommand;
pub mod telemetry;
pub mod watchdog;
