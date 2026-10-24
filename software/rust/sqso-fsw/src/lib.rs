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
//! - [`eps`]: EPS power policy and its stored state (`eps_control.h`)
//! - [`sensors`]: sensor configuration file, readings, telemetry text (`sensors.h`)
//! - [`mission_data`]: mission statistics record (`mission_data.h`)
//! - [`gnss`]: GNSS NMEA receiver and fix acceptance (`gnss.h`)

#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic))]

pub mod eps;
pub mod fault_log;
pub mod gnss;
pub mod keys;
pub mod mission_data;
pub mod mode;
pub mod nvm;
pub mod scheduler;
pub mod sensors;
pub mod telecommand;
