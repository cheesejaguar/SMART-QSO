//! `smart_qso_flight`: the flight software on the simulated HAL, as a host
//! program. It runs the executive (`sqso_ffi::executive`) for about 20 s of
//! simulated operation (1000 passes, 20 ms apart), or until stopped in the
//! flight configuration; SIGINT and SIGTERM end it early with state saved.
//!
//! @note Host simulation only (not a flight requirement)

use std::process::ExitCode;

fn main() -> ExitCode {
    // A Rust panic is a fatal assertion, as on the target: reported to the
    // assertion handler; the release profile then aborts (panic = "abort")
    std::panic::set_hook(Box::new(|info| sqso_ffi::report_panic_at(info.location())));
    let ticks = if cfg!(feature = "flight") { None } else { Some(sqso_fsw_ticks()) };
    match sqso_ffi::executive::run_host(ticks) {
        0 => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

/// Passes in a simulation run
fn sqso_fsw_ticks() -> u64 {
    sqso_ffi::executive::SIMULATION_TICKS
}
