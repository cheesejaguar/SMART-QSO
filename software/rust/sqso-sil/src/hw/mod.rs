//! Spacecraft hardware the flight software drives and reads, between it
//! and the physical environment (`crate::env`): the EPS electronics (MPPT,
//! charge regulator, BMS, regulators, load switches, the controller's
//! telemetry and fuel gauge), the loads, the downlink radio and its
//! monitors, and the deployment mechanisms.
//!
//! Parts not yet selected are modelled from the design documents' values
//! and the SQSO-HW-DIAG-001 schematics; every assumption is a named
//! constant.

pub mod deploy;
pub mod eps;
pub mod radio;
