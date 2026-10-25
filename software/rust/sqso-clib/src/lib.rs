//! The flight software (`sqso-ffi`) as the static library `libsqso_flight.a`,
//! which CMake links into the C test suites (`software/flight/tests`): the
//! C headers in `software/flight/include` are its interface.
//!
//! The library is one LTO object, so every `#[no_mangle]` function of the
//! flight software is in it. This crate adds only the panic handler.

#![cfg_attr(not(test), no_std)]

// Linked for its exported C functions, which nothing here calls
extern crate sqso_ffi;

/// A Rust panic is a fatal assertion: reported to the assertion handler,
/// then the process stops. On the host the test aborts, so it fails; on
/// the target (the C library is cross-built for it too) the processor
/// waits for the independent watchdog.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    sqso_ffi::report_panic(info);
    stop()
}

#[cfg(all(not(test), not(target_os = "none")))]
fn stop() -> ! {
    sqso_ffi::abort()
}

#[cfg(all(not(test), target_os = "none"))]
fn stop() -> ! {
    #[allow(clippy::empty_loop)] // no kick: the watchdog resets the processor
    loop {}
}
