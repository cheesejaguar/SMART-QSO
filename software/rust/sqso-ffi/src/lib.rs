//! The SMART-QSO flight software behind its C headers.
//!
//! Each module implements one header of `software/flight/include` on top of
//! `sqso-protocol` and `sqso-fsw`, so the C test suites (the specification)
//! use it unchanged; the executive ([`executive`]) runs the main loop and
//! the simulated HAL (`hal_sim`) stands in for the hardware on the host.
//! `sqso-clib` builds it as the static library for the C suites, `sqso-sim`
//! as the host simulation and the firmware links it for the STM32L4R5.
//!
//! With the firmware, this is the only crate with `unsafe` code, and here
//! that code does only three things: turn C pointers into slices and
//! references (after a NULL check), call functions through the C ABI (the
//! HAL, other modules, libc on the host), and hold module state in
//! [`cell::Global`].
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
//! | `smart_qso.h` (mission clock) | [`time`] |
//! | `separation_switch.h` | [`separation_switch`] |
//! | `hal/*.h`, `hal/hal_sim.h` (host simulation) | `hal_sim` |
//! | `rf_control.h` | [`rf_control`] |
//! | `rf_silence.h` | [`rf_silence`] |
//! | `contact_monitor.h` | [`contact_monitor`] |
//! | `deployment.h` | [`deployment`] |
//! | `payload_guard.h` | [`payload_guard`] |
//! | `safe_string.h` | [`safe_string`] |
//! | `assert_handler.h` | [`assert_handler`] |
//! | `flight_log.h` (with `flight_log_fmt.c`) | [`flight_log`] |
//! | `uart_comm.h` | [`uart_comm`] |
//! | (`main.c`) | [`executive`] |
//! | `eps_control.h` | [`eps_control`] |
//! | `sensors.h` | [`sensors`] |
//! | `mission_data.h` | [`mission_data`] |
//! | `gnss.h` | [`gnss`] |
//! | `system_state.h` | [`system_state`] |
//! | `telemetry.h` | [`telemetry`] |
//! | `beacon.h` | [`beacon`] |
//! | `adcs_control.h` | [`adcs_control`] |
//! | `watchdog_mgr.h` | [`watchdog_mgr`] |
//! | `passivation.h` | [`passivation`] |
//! | `receiver.h` | [`receiver`] |
//! | `hk_store.h` | [`hk_store`] |
//! | `eps_lockout.h` | [`eps_lockout`] |
//!
//! Struct layouts mirror the C headers (`#[repr(C)]`); the C contract test
//! `tests/test_rust_contract.c` checks sizes and constants from the C side.

#![cfg_attr(not(test), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)] // every export documents its pointer contract in its C header

pub mod adcs_control;
pub mod ants;
pub mod assert_handler;
pub mod ax25;
pub mod beacon;
pub mod beacon_filter;
pub mod cmd_auth;
pub mod cmd_dictionary;
pub mod cmd_handler;
pub mod contact_monitor;
pub mod contract;
pub mod crc;
pub mod deployment;
pub mod downlink;
pub mod eps_control;
pub mod eps_lockout;
pub mod eps_supervision;
pub mod executive;
pub mod fault_mgmt;
pub mod flight_log;
pub mod fram_health;
pub mod gnss;
pub mod gse_provision;
#[cfg(not(target_os = "none"))]
pub mod hal_sim;
pub mod hk_store;
pub mod input_validation;
pub mod jetson_link;
pub mod mission_data;
pub mod nvm_record;
pub mod ops_state;
pub mod passivation;
pub mod payload_guard;
pub mod receiver;
pub mod rf_control;
pub mod rf_silence;
pub mod safe_string;
pub mod scheduler;
pub mod sensors;
pub mod separation_switch;
pub mod sha256;
#[cfg(not(target_os = "none"))]
pub mod sil;
pub mod state_machine;
pub mod system_state;
pub mod telemetry;
pub mod time;
pub mod uart_comm;
pub mod uplink;
pub mod watchdog_mgr;

mod c;
mod cell;
mod hw;
#[cfg(not(target_os = "none"))]
mod os;
mod pins;
mod ptr;
mod store;

/// Report a Rust panic (an overflow or a broken invariant; the parsers have
/// no panic paths) as a fatal assertion. The crate that links the program
/// owns the panic handler and calls this, then stops: `sqso-clib` (the C
/// test suites), `sqso-sim` (the host simulation) and the firmware.
pub fn report_panic(info: &core::panic::PanicInfo<'_>) {
    report_panic_at(info.location());
}

/// [`report_panic`] for a panic at `location` (what a `std` panic hook has)
pub fn report_panic_at(location: Option<&core::panic::Location<'_>>) {
    let mut file = [0u8; 64];
    let mut line = 0u16;
    if let Some(location) = location {
        for (dst, src) in file.iter_mut().zip(location.file().bytes().take(63)) {
            *dst = src;
        }
        line = u16::try_from(location.line()).unwrap_or(u16::MAX);
    }
    // SAFETY: both strings are NUL-terminated (at most 63 bytes copied into 64)
    let _ = unsafe {
        assert_handler::assert_handle_failure(
            file.as_ptr().cast(),
            line,
            assert_handler::ASSERT_TYPE_INVARIANT,
            assert_handler::ASSERT_SEVERITY_FATAL,
            c"Rust panic".as_ptr(),
        )
    };
}

/// Stop the host program (abort, so a test that panics fails)
#[cfg(not(target_os = "none"))]
pub fn abort() -> ! {
    os::abort()
}
