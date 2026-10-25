//! The stack's 8 KB budget (SRS-C003) at run time.
//!
//! The image's worst case is computed in CI from its machine code
//! (`scripts/stack_usage.py`). Here, at boot:
//!
//! - an MPU region with no access just below the 8 KB under the initial SP,
//!   so a stack that outgrows its budget faults (MemManage escalates to
//!   HardFault, whose handler spins until the independent watchdog resets
//!   the processor) instead of running on, unseen, toward the static data
//!   about 600 KB below;
//! - the budget painted with [`PAINT`], so a debug probe can read how much
//!   of it was ever used: the stack high-water check of the hardware tests
//!   (`docs/verification/TEST_PROCEDURES.md`).

use cortex_m::peripheral::MPU;

/// SRS-C003
pub const BUDGET: u32 = 8 * 1024;
/// Guard region size: the MPU's smallest (base aligned to it)
const GUARD: u32 = 32;
/// The paint pattern
pub const PAINT: u32 = 0x57AC_C0DE;

unsafe extern "C" {
    /// The initial SP (cortex-m-rt's link.x)
    static _stack_start: u32;
}

/// Lowest address of the budget, just above the guard
fn floor() -> u32 {
    let top = core::ptr::addr_of!(_stack_start) as u32;
    top - BUDGET
}

/// Paint the unused part of the budget and arm the guard below it. Call
/// first, before any interrupt is enabled.
pub fn protect() {
    paint(floor(), cortex_m::register::msp::read() & !3);
    arm_guard(floor() - GUARD);
}

/// Fill [from, to) with [`PAINT`] (both word-aligned, below the live stack)
fn paint(from: u32, to: u32) {
    let mut addr = from;
    while addr < to {
        // SAFETY: an aligned word of RAM inside the stack budget and below
        // the current SP, which nothing else uses
        unsafe { core::ptr::write_volatile(addr as *mut u32, PAINT) };
        addr += 4;
    }
}

/// MPU region 0: `base` (GUARD-aligned), GUARD bytes, no access, never
/// executed; the default memory map elsewhere (PRIVDEFENA)
fn arm_guard(base: u32) {
    const RASR_XN: u32 = 1 << 28;
    const RASR_ENABLE: u32 = 1;
    const SIZE_32: u32 = 4 << 1; // 2^(4 + 1) bytes; AP = 0: no access
    const CTRL_ENABLE: u32 = 1;
    const CTRL_PRIVDEFENA: u32 = 1 << 2;
    // SAFETY: the MPU's registers; one region, set before anything relies
    // on the memory map changing
    unsafe {
        let mpu = &*MPU::PTR;
        mpu.rnr.write(0);
        mpu.rbar.write(base);
        mpu.rasr.write(RASR_XN | SIZE_32 | RASR_ENABLE);
        mpu.ctrl.write(CTRL_ENABLE | CTRL_PRIVDEFENA);
    }
    cortex_m::asm::dsb();
    cortex_m::asm::isb();
}
