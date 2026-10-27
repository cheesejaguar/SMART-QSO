//! RAM and flash upsets (RID AV-07; `docs/environment/RADIATION_ANALYSIS.md`
//! §5.2): the live state in SRAM2, the parity NMI and the flash ECC NMI.
//!
//! - `memory.x` links `.data`, `.bss` and `.uninit` (every static of the
//!   flight software, the executor's task included) at the start of SRAM2,
//!   the STM32L4R5's only RAM with parity (one bit per byte); the link fails
//!   if they outgrow its 64 KB with the stack. The 8 KB stack is at the top
//!   of SRAM2 too (RID AV-13), so every live byte has parity.
//! - [`__pre_init`](self) erases SRAM2 by hardware (SYSCFG_SCSR.SRAM2ER)
//!   before cortex-m-rt initializes RAM, so no byte is ever read with the
//!   random parity of power-on.
//! - Parity is on when the option bit `SRAM2_PE` is 0. It is set when the
//!   board is programmed (`software/flight/docs/MEMORY_POLICY.md` §10 and
//!   `docs/development/RELEASE_PROCEDURE.md` §5.3.1: the flight software never
//!   writes option bytes); [`report`] gives the state to the boot telemetry
//!   (the console and HK2 `PE`) and logs every boot that finds it off.
//! - The NMI (an SRAM2 parity error, a flash double-bit ECC error or a
//!   clock-security failure) records its cause in SRAM1, outside the linked
//!   RAM (not erased, not zeroed), and resets the processor at once. The next
//!   boot logs the cause ([`report`]); the state is reloaded from FRAM under
//!   CRC, and the flight software checks its persisted state against FRAM
//!   every minute (SRS-F133).
//!
//! @requirement SRS-F132 SRAM2 parity and flash ECC errors: an NMI that resets at once, the cause logged at the next boot
//! @requirement SRS-F154 Boot telemetry reports the SRAM2 parity state

use embassy_stm32::pac;

/// The cause record in SRAM1: [tag | cause, !word 0, address]
const MARK: *mut [u32; 3] = 0x2002_FFF0 as *mut [u32; 3];
const TAG: u32 = 0x4E4D_4900; // "NMI\0"
const TAG_MASK: u32 = 0xFFFF_FF00;

/// The NMI's causes
const CAUSE_PARITY: u32 = 1;
const CAUSE_FLASH_ECC: u32 = 2;
const CAUSE_CLOCK: u32 = 3;
const CAUSE_OTHER: u32 = 4;

/// `FaultType_t`: FAULT_TYPE_SW_INTERNAL; `FaultSeverity_t`
const FAULT_SW_INTERNAL: u32 = 22;
const SEVERITY_WARNING: u32 = 2;
const SEVERITY_CRITICAL: u32 = 4;

// SRAM2 erase before RAM is initialized: registers only (no stack, no
// statics), called by cortex-m-rt's Reset with `bl`. RCC_APB2ENR.SYSCFGEN
// on, the SYSCFG_SKR key (0xCA, 0x53), SYSCFG_SCSR.SRAM2ER, then wait while
// SRAM2BSY.
core::arch::global_asm!(
    ".section .text.__pre_init,\"ax\"",
    ".global __pre_init",
    ".type __pre_init,%function",
    ".thumb_func",
    "__pre_init:",
    "    ldr r0, =0x40021060",
    "    ldr r1, [r0]",
    "    orr r1, r1, #1",
    "    str r1, [r0]",
    "    ldr r1, [r0]",
    "    ldr r0, =0x40010000",
    "    movs r1, #0xCA",
    "    str r1, [r0, #0x24]",
    "    movs r1, #0x53",
    "    str r1, [r0, #0x24]",
    "    movs r1, #1",
    "    str r1, [r0, #0x18]",
    "1:  ldr r1, [r0, #0x18]",
    "    tst r1, #2",
    "    bne 1b",
    "    bx lr",
    ".size __pre_init, . - __pre_init",
);

/// Whether SRAM2 parity is on (option bit `SRAM2_PE` = 0)
fn parity_on() -> bool {
    !pac::FLASH.optr().read().sram2_pe()
}

/// Read and clear the cause record left by an NMI before the reset
fn take_mark() -> Option<(u32, u32)> {
    // SAFETY: three aligned words of SRAM1, which nothing else uses (the
    // linked RAM starts at SRAM2); any value is valid
    let [word, check, address] = unsafe { core::ptr::read_volatile(MARK) };
    // SAFETY: as above
    unsafe { core::ptr::write_volatile(MARK, [0; 3]) };
    (check == !word && word & TAG_MASK == TAG).then_some((word & !TAG_MASK, address))
}

/// At boot, once the fault log is loaded: log the cause of an NMI reset,
/// and log if SRAM2 parity is off; clear the latched error flags
pub(crate) fn report() {
    pac::SYSCFG.cfgr2().modify(|w| w.set_spf(true));
    pac::FLASH.eccr().modify(|w| w.set_eccd(true));
    let text: Option<&core::ffi::CStr> = match take_mark() {
        Some((CAUSE_PARITY, _)) => Some(c"Reset by NMI: SRAM2 parity error"),
        Some((CAUSE_FLASH_ECC, _)) => Some(c"Reset by NMI: flash ECC double error"),
        Some((CAUSE_CLOCK, _)) => Some(c"Reset by NMI: clock security"),
        Some(_) => Some(c"Reset by NMI: unknown cause"),
        None => None,
    };
    if let Some(text) = text {
        log(SEVERITY_CRITICAL, text);
    }
    let on = parity_on();
    // Boot telemetry (RID AV-12): the console line and HK2 `PE`
    sqso_ffi::executive::set_parity_state(on);
    if !on {
        log(SEVERITY_WARNING, c"SRAM2 parity off (option bit SRAM2_PE)");
    }
}

fn log(severity: u32, text: &core::ffi::CStr) {
    // SAFETY: a NUL-terminated string; plain values
    let _ = unsafe { sqso_ffi::fault_mgmt::fault_log_add(FAULT_SW_INTERNAL, severity, text.as_ptr(), 0.0) };
}

/// The NMI: record the cause and reset at once (the state in RAM can no
/// longer be trusted; FRAM holds what must survive)
#[cortex_m_rt::exception]
unsafe fn NonMaskableInt() -> ! {
    let eccr = pac::FLASH.eccr().read();
    let (cause, address) = if pac::SYSCFG.cfgr2().read().spf() {
        (CAUSE_PARITY, 0)
    } else if eccr.eccd() {
        (CAUSE_FLASH_ECC, eccr.addr_ecc())
    } else if pac::RCC.cifr().read().cssf() {
        (CAUSE_CLOCK, 0)
    } else {
        (CAUSE_OTHER, 0)
    };
    let word = TAG | cause;
    // SAFETY: three aligned words of SRAM1 (see take_mark)
    unsafe { core::ptr::write_volatile(MARK, [word, !word, address]) };
    cortex_m::peripheral::SCB::sys_reset()
}
