//! C ABI for the SMART-QSO Rust modules.
//!
//! This static library implements C headers of the flight software
//! (`software/flight/include`) on top of `sqso-protocol` and `sqso-fsw`, so
//! the existing C callers and the C test suite use it unchanged. It is the
//! only crate in the workspace with `unsafe` code, and that code does only
//! three things: turn C pointers into slices and references (after a NULL
//! check), call C functions, and hold module state in [`cell::Global`].
//!
//! | C header | module |
//! |----------|--------|
//! | `smart_qso.h` (CRC-32) | [`crc`] |
//! | `sha256.h` | [`sha256`] |
//! | `ax25.h` | [`ax25`] |
//! | `jetson_link.h` | [`jetson_link`] |
//! | `input_validation.h` | [`input_validation`] |
//! | `cmd_dictionary.h` | [`cmd_dictionary`] |
//! | `nvm_record.h` | [`nvm_record`] |
//! | `cmd_auth.h` | [`cmd_auth`] |
//! | `beacon_filter.h` | [`beacon_filter`] |
//! | `gse_provision.h` | [`gse_provision`] |
//! | `uplink.h` | [`uplink`] |
//! | `cmd_handler.h` | [`cmd_handler`] |
//! | `state_machine.h` | [`state_machine`] |
//! | `scheduler.h` | [`scheduler`] |
//! | `fault_mgmt.h` | [`fault_mgmt`] |
//! | `safe_string.h` | [`safe_string`] |
//! | `assert_handler.h` | [`assert_handler`] |
//! | `flight_log.h` (with `flight_log_fmt.c`) | [`flight_log`] |
//! | `uart_comm.h` | [`uart_comm`] |
//!
//! Struct layouts mirror the C headers (`#[repr(C)]`); the C contract test
//! `tests/test_rust_contract.c` checks sizes and constants from the C side.

#![cfg_attr(not(test), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)] // every export documents its pointer contract in its C header

pub mod assert_handler;
pub mod ax25;
pub mod beacon_filter;
pub mod cmd_auth;
pub mod cmd_dictionary;
pub mod cmd_handler;
pub mod contract;
pub mod crc;
pub mod fault_mgmt;
pub mod flight_log;
pub mod gse_provision;
pub mod input_validation;
pub mod jetson_link;
pub mod nvm_record;
pub mod safe_string;
pub mod scheduler;
pub mod separation_switch;
pub mod sha256;
pub mod state_machine;
pub mod time;
pub mod uart_comm;
pub mod uplink;

mod c;
mod cell;
#[cfg(not(target_os = "none"))]
mod os;
mod ptr;
mod store;

/// A Rust panic (an overflow or a broken invariant; the parsers have no
/// panic paths) is a fatal assertion: reported to the C assertion handler,
/// then the system stops for the watchdog to reset it.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    let mut file = [0u8; 64];
    let mut line = 0;
    if let Some(location) = info.location() {
        for (dst, src) in file.iter_mut().zip(location.file().bytes().take(63)) {
            *dst = src;
        }
        line = location.line();
    }
    // SAFETY: file is NUL-terminated (at most 63 bytes copied into 64)
    unsafe { c::smart_qso_rust_panic(file.as_ptr().cast(), line) }
}
