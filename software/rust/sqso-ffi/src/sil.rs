//! Hooks for the software-in-the-loop mission simulation (`sqso-sil`):
//! the virtual clock and the console sink. Host only.
//!
//! The SIL runs the flight software for simulated years. On the virtual
//! clock, time is what the simulation says it is: the HAL timer and
//! `smart_qso_now_ms()` read it, and the HAL's delays advance it instead
//! of sleeping. The simulation starts the clock at each simulated boot near
//! zero, as the target's time base starts at reset.
//!
//! @note Host simulation only (not a flight requirement)

/// Run on the virtual clock from now on, reading `start_us`
pub fn boot_clock(start_us: u64) {
    crate::os::virtual_clock(start_us);
}

/// Advance the virtual clock by `us`
pub fn advance_us(us: u64) {
    crate::os::virtual_advance_us(us);
}

/// The clock now, us (the virtual clock's reading when it is on)
pub fn now_us() -> u64 {
    crate::os::realtime_us()
}

/// Send console output (and error lines) to `sink`, one line per call;
/// `None` restores stdout and stderr
pub fn console_sink(sink: Option<fn(&[u8])>) {
    crate::os::console_sink(sink);
}
