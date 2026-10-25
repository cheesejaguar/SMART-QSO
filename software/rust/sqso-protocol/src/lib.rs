//! SMART-QSO wire formats.
//!
//! Everything the spacecraft receives from outside passes through this crate
//! first: AX.25 UI frames from the 70 cm receiver, the uplink command frame
//! and its authentication block, lines from the Jetson payload, frames on
//! the GSE umbilical, and beacon text from the payload's language model. It
//! also encodes the same formats, so the flight software, the payload and
//! the ground tools share one implementation of each (SW-04, SW-24).
//!
//! Rules for this crate:
//! - `#![no_std]`, no allocation: it runs on the STM32L4 as well as the host.
//! - No `unsafe` (`forbid`).
//! - No panics on any input: no slice indexing that can go out of bounds,
//!   no `unwrap` (clippy lints, denied in CI), and the parsers are fuzzed
//!   (`software/rust/fuzz`).
//!
//! The interface documents are the authority for each format:
//! SQSO-ICD-013 (uplink and telecommand), `docs/interfaces/ICD_OBC_JETSON.md`
//! (Jetson link), `docs/interfaces/TELECOMMAND_AUTHENTICATION.md`, and
//! `docs/BEACON_SPEC.md` section 6 (content filter).

#![no_std]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic))]

pub mod auth;
pub mod ax25;
pub mod checksum;
pub mod command;
#[rustfmt::skip] // generated (scripts/gen_command_dictionary.py)
pub mod dictionary;
pub mod filter;
pub mod frame;
pub mod gse;
pub mod jetson;
pub mod kiss;
pub mod station;

mod write;

pub use command::{CommandSpec, KeyClass, MAX_ARGS};
pub use dictionary::Command;
