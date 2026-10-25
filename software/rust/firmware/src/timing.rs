//! Main-loop timing (SRS-P001: a pass within 20 ms), for a debug probe.
//!
//! Each `Executive::step()` pass is timed with the Cortex-M DWT cycle
//! counter (80 MHz core clock, `clocks()` in main.rs) and the result kept
//! in [`SQSO_LOOP_TIMING`]. Nothing in the flight software reads it: the
//! hardware test HIL-040 reads it by symbol name through the probe
//! (`tools/loop_timing.py`, `software/flight/tests/hil/HIL_TEST_PLAN.md`).
//! It writes at most four words of its own per pass and has no other
//! effect on the flight software.
//!
//! The counter belongs to the debug logic, which a debugger may switch off
//! when it detaches (DEMCR = 0, as probe-rs's generic Cortex-M sequence
//! does). So each pass starts the counter again if it is off, and a pass
//! during which it went off is not recorded.
//!
//! The counter is 32 bits: a pass longer than about 53.7 s (2^32 cycles)
//! would be under-reported, but the 30 s watchdog resets the OBC first.
//! The counter stops while the core is halted by a debugger.
//!
//! Instrumentation for the verification of SRS-P001 (main.rs carries the
//! requirement tag), not part of the flight logic.

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::peripheral::{DCB, DWT};

/// The core clock (`clocks()` in main.rs: 80 MHz)
pub const CORE_HZ: u32 = 80_000_000;
/// SRS-P001: 20 ms in core cycles
pub const LIMIT_CYCLES: u32 = CORE_HZ / 1000 * 20;

/// DEMCR.TRCENA: the DWT's enable
const DEMCR_TRCENA: u32 = 1 << 24;
/// DWT_CTRL.CYCCNTENA
const CTRL_CYCCNTENA: u32 = 1;

/// The main loop's pass times, in core cycles. Layout fixed for the probe
/// (`tools/loop_timing.py`): six little-endian 32-bit words.
#[repr(C)]
pub struct LoopTiming {
    /// [`CORE_HZ`], so the probe converts cycles without assuming the clock
    pub core_hz: u32,
    /// [`LIMIT_CYCLES`]
    pub limit_cycles: u32,
    /// The last pass
    pub last_cycles: AtomicU32,
    /// The longest pass since boot
    pub max_cycles: AtomicU32,
    /// Passes longer than [`LIMIT_CYCLES`] (saturates)
    pub overruns: AtomicU32,
    /// Passes recorded (saturates)
    pub passes: AtomicU32,
}

/// Read by the probe by name (HIL-040)
#[unsafe(no_mangle)]
pub static SQSO_LOOP_TIMING: LoopTiming = LoopTiming {
    core_hz: CORE_HZ,
    limit_cycles: LIMIT_CYCLES,
    last_cycles: AtomicU32::new(0),
    max_cycles: AtomicU32::new(0),
    overruns: AtomicU32::new(0),
    passes: AtomicU32::new(0),
};

/// Whether the trace clock (DEMCR.TRCENA) is on
fn trace_enabled() -> bool {
    // SAFETY: a read of DEMCR, a core debug register with no read side effects
    unsafe { (*DCB::PTR).demcr.read() & DEMCR_TRCENA != 0 }
}

/// Start the cycle counter if it is off: DEMCR.TRCENA (the DWT's clock),
/// then DWT_CTRL.CYCCNTENA. Called at boot and before each pass.
pub fn start() {
    if trace_enabled() && DWT::cycle_counter_enabled() {
        return;
    }
    // SAFETY: two read-modify-writes of core debug registers, from the main
    // task only; they turn on the trace clock and the cycle counter and
    // change nothing the flight software or the HAL uses
    unsafe {
        (*DCB::PTR).demcr.modify(|w| w | DEMCR_TRCENA);
        (*DWT::PTR).ctrl.modify(|w| w | CTRL_CYCCNTENA);
    }
}

/// The cycle counter at the start of a pass (started if it was off)
pub fn now() -> u32 {
    start();
    DWT::cycle_count()
}

/// Record a pass that started at `start` (from [`now`]); not recorded if
/// the counter was switched off meanwhile
pub fn record(start: u32) {
    if !(trace_enabled() && DWT::cycle_counter_enabled()) {
        return;
    }
    let cycles = DWT::cycle_count().wrapping_sub(start);
    let t = &SQSO_LOOP_TIMING;
    // One writer (the main task), so plain loads and stores suffice
    t.last_cycles.store(cycles, Ordering::Relaxed);
    if cycles > t.max_cycles.load(Ordering::Relaxed) {
        t.max_cycles.store(cycles, Ordering::Relaxed);
    }
    if cycles > LIMIT_CYCLES {
        t.overruns.store(t.overruns.load(Ordering::Relaxed).saturating_add(1), Ordering::Relaxed);
    }
    t.passes.store(t.passes.load(Ordering::Relaxed).saturating_add(1), Ordering::Relaxed);
}
