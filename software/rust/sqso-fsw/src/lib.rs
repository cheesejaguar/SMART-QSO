//! SMART-QSO flight logic, independent of any runtime.
//!
//! Each module is the decision logic of one flight module, with its I/O
//! behind a trait (flash) or returned to the caller (log events), so it can
//! be tested and fuzzed on the host. `sqso-ffi` puts it behind the C headers
//! and connects it to the HAL.
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
//! - [`payload_schedule`]: the Jetson's batch duty cycle
//! - [`heater`]: battery heater thermostat
//! - [`transponder`]: the FM transponder's power, thermal and duty gates
//! - [`safe_string`]: bounded string and memory operations (`safe_string.h`)
//! - [`assert_handler`]: assertion failure log and response (`assert_handler.h`)
//! - [`flight_log`]: flight log ring buffer (`flight_log.h`)
//! - [`uart_comm`]: OBC–Jetson serial line settings (`uart_comm.h`)
//! - [`eps`]: EPS power policy and its stored state (`eps_control.h`)
//! - [`eps_telemetry`]: EPS controller telemetry registers (SMART-QSO-ICD-001)
//! - [`executive`]: the main loop's power model and field rules (was `main.c`)
//! - [`sensors`]: sensor configuration file, readings, telemetry text (`sensors.h`)
//! - [`mission_data`]: mission statistics record (`mission_data.h`)
//! - [`gnss`]: GNSS NMEA receiver and fix acceptance (`gnss.h`)
//! - [`system_state`]: the system state record and its image (`system_state.h`)
//! - [`telemetry`]: binary telemetry frames and schedule (`telemetry.h`)
//! - [`beacon`]: beacon text, telemetry fields and AX.25 frame (`beacon.h`)
//! - [`downlink`]: the downlink transmitter's gate, limits and monitors (`downlink.h`)
//! - [`adcs`]: B-dot detumble, sun pointing and ADCS modes (`adcs_control.h`)
//! - [`watchdog`]: the supervised watchdog's task table (`watchdog_mgr.h`)
//! - [`fmath`]: C `sqrt` and `round` for `no_std`
//! - [`fixed`]: `{:.N}` text for an `f64` from a small stack frame (SRS-C003)
//! - [`passivation`]: end-of-life passivation by two telecommands (`passivation.h`)
//! - [`receiver`]: command-receiver supervision, recovery and failure evidence (`receiver.h`)
//! - [`hk_store`]: stored housekeeping records and their dump (`hk_store.h`)
//! - [`ack`]: the command acknowledgement frame (`cmd_handler.h`)
//! - [`bus`]: the SPI and I2C traits the device drivers use
//! - [`mg370`], [`bmi088`], [`rm3100`], [`ads7138`]: the ADCS sensors' drivers
//! - [`adcs_sensors`]: the sensors' rails, bring-up and cold-spare gyro
//! - [`mtq`]: the magnetorquer drive (duty, direction, polarity state)
//! - [`rv3028`]: the mission clock (reset time stamps)
//! - [`condition`]: a repeated condition logged at onset and clearance
//! - [`jetson_power`]: the Jetson's switch-off, power-good and switch readbacks
//! - [`ina228`]: the EPS's pack and array power monitors

#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic))]

pub mod ack;
pub mod adcs;
pub mod adcs_sensors;
pub mod ads7138;
pub mod ants;
pub mod assert_handler;
pub mod beacon;
pub mod bmi088;
pub mod bus;
pub mod condition;
pub mod contact_monitor;
pub mod deployment;
pub mod downlink;
pub mod eps;
pub mod eps_telemetry;
pub mod executive;
pub mod fault_log;
pub mod fixed;
pub mod flight_log;
pub mod fmath;
pub mod gnss;
pub mod gnss_frame;
pub mod gnss_schedule;
pub mod heater;
pub mod hk_store;
pub mod housekeeping;
pub mod ina228;
pub mod jetson_power;
pub mod keys;
pub mod mg370;
pub mod mission_data;
pub mod mode;
pub mod mtq;
pub mod nvm;
pub mod ops_state;
pub mod passivation;
pub mod payload_guard;
pub mod payload_schedule;
pub mod receiver;
pub mod rf_control;
pub mod rf_silence;
pub mod rm3100;
pub mod rv3028;
pub mod safe_string;
pub mod scheduler;
pub mod sensors;
pub mod system_state;
pub mod telecommand;
pub mod telemetry;
pub mod transponder;
pub mod uart_comm;
pub mod watchdog;
