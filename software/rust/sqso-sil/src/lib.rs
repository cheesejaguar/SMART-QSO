//! SMART-QSO software-in-the-loop (SIL) mission simulation.
//!
//! The flight software (`sqso_ffi`: the executive and every flight module,
//! on the simulated HAL) runs against models of the spacecraft and its
//! environment, on a virtual clock, from integration to re-entry.

pub mod checks;
pub mod config;
pub mod env;
pub mod fsw;
pub mod hw;
pub mod jetson;
pub mod metrics;
pub mod mission;
pub mod ops;
pub mod qso;
pub mod recorder;
