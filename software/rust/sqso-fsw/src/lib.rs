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
//! - [`rf_control`]: transmitter permission and latched TX-OFF (`rf_control.h`)
//! - [`rf_silence`]: 45-minute post-deployment RF silence (`rf_silence.h`)
//! - [`contact_monitor`]: loss-of-contact transmit inhibit (`contact_monitor.h`)
//! - [`deployment`]: deployment wait and burn sequence (`deployment.h`)
//! - [`payload_guard`]: OBC-enforced Jetson power and temperature limits (`payload_guard.h`)
//! - [`safe_string`]: bounded string and memory operations (`safe_string.h`)
//! - [`assert_handler`]: assertion failure log and response (`assert_handler.h`)
//! - [`flight_log`]: flight log ring buffer (`flight_log.h`)
//! - [`uart_comm`]: OBC–Jetson serial line settings (`uart_comm.h`)

#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic))]

pub mod assert_handler;
pub mod contact_monitor;
pub mod deployment;
pub mod fault_log;
pub mod flight_log;
pub mod keys;
pub mod mode;
pub mod nvm;
pub mod payload_guard;
pub mod rf_control;
pub mod rf_silence;
pub mod safe_string;
pub mod scheduler;
pub mod telecommand;
pub mod uart_comm;
