//! `hal_wdt.h` (the independent watchdog, IWDG, on the 32 kHz LSI) and
//! `hal_reset.h` (the reset cause from the RCC CSR flags).

use core::cell::Cell;
use core::ffi::c_void;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use critical_section::Mutex;

use embassy_stm32::pac;
use embassy_stm32::pac::iwdg::vals::{Key, Pr};

use crate::rc;
use crate::timer::hal_timer_get_ms;

const MODE_RESET: u32 = 0;
const LSI_HZ: u64 = 32_000;
const RELOAD_MAX: u64 = 4095;

const RESET_POWER_ON: u32 = 0;
const RESET_WATCHDOG: u32 = 2;
const RESET_SOFTWARE: u32 = 3;
const RESET_PIN: u32 = 4;
const RESET_UNKNOWN: u32 = 5;

static TIMEOUT_MS: AtomicU32 = AtomicU32::new(0);
/// No 64-bit atomics on the Cortex-M4: a (two-instruction) critical section
static LAST_KICK_MS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

fn set_last_kick(ms: u64) {
    critical_section::with(|cs| LAST_KICK_MS.borrow(cs).set(ms));
}

fn last_kick() -> u64 {
    critical_section::with(|cs| LAST_KICK_MS.borrow(cs).get())
}
static RUNNING: AtomicBool = AtomicBool::new(false);
static RESET_FLAG_CLEARED: AtomicBool = AtomicBool::new(false);
/// RESET_UNKNOWN + 1 until read
static RESET_CAUSE: AtomicU32 = AtomicU32::new(u32::MAX);

/// The smallest prescaler (4 << pr) whose reload fits in 12 bits:
/// `(pr, reload)`, or `None` above about 32.8 s
fn divider_for(timeout_ms: u32) -> Option<(u8, u16)> {
    (0u8..=6).find_map(|pr| {
        let divider = 4u64 << pr;
        let ticks = u64::from(timeout_ms) * LSI_HZ / (1000 * divider);
        (1..=RELOAD_MAX + 1).contains(&ticks).then(|| (pr, u16::try_from(ticks - 1).unwrap_or(0)))
    })
}

/// Start the IWDG at the operating timeout (25.6 s: reload 3200 at the LSI /
/// 256, 24.1 to 27.8 s, inside the external supervisor's 29.3 s minimum,
/// SQSO-ICX-001 D §4.3 and §6 row 37) before anything else runs: the clock set-up
/// waits for the PLL to lock, and a part that never locks must reset rather
/// than hang. It starts at the reset, as the supervisor's watchdog does, so
/// the two share a time base; the flight software refreshes it only with a
/// WDI falling edge (its initialisation checkpoints, then the main loop: RID
/// P3F-A-03), and its hal_wdt_init/hal_wdt_start leave the count running.
pub(crate) fn start_boot_watchdog() {
    const BOOT_TIMEOUT_MS: u32 = 25_600;
    if hal_wdt_init(BOOT_TIMEOUT_MS, MODE_RESET) == rc::OK {
        let _ = hal_wdt_start();
    }
}

/// Whether the IWDG has been started (it then cannot be stopped)
pub(crate) fn running() -> bool {
    RUNNING.load(Ordering::Relaxed)
}

/// Set the timeout; the IWDG only resets (no interrupt mode)
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_init(timeout_ms: u32, mode: u32) -> i32 {
    if mode != MODE_RESET || timeout_ms == 0 {
        return rc::INVALID;
    }
    let Some((pr, reload)) = divider_for(timeout_ms) else { return rc::INVALID };
    let iwdg = pac::IWDG;
    iwdg.kr().write(|w| w.set_key(Key::ENABLE)); // unlock PR and RLR
    iwdg.pr().write(|w| w.set_pr(Pr::from_bits(pr)));
    iwdg.rlr().write(|w| w.set_rl(reload));
    TIMEOUT_MS.store(timeout_ms, Ordering::Relaxed);
    rc::OK
}

/// Start the IWDG (it cannot be stopped again). Already running (the boot
/// watchdog): nothing changes, its count and its last reload stand, so the
/// flight software times the supervisor's WDI from that reload (RID
/// P3F-A-03; the START key does not reload a running IWDG)
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_start() -> i32 {
    if TIMEOUT_MS.load(Ordering::Relaxed) == 0 {
        return rc::ERROR;
    }
    if RUNNING.load(Ordering::Relaxed) {
        return rc::OK;
    }
    pac::IWDG.kr().write(|w| w.set_key(Key::START));
    RUNNING.store(true, Ordering::Relaxed);
    set_last_kick(hal_timer_get_ms());
    rc::OK
}

/// The IWDG cannot be stopped once started
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_stop() -> i32 {
    rc::ERROR
}

/// Reload the IWDG
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_kick() {
    if RUNNING.load(Ordering::Relaxed) {
        pac::IWDG.kr().write(|w| w.set_key(Key::RESET));
        set_last_kick(hal_timer_get_ms());
    }
}

/// Time left before the IWDG resets (the full timeout while stopped)
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_remaining_ms() -> u32 {
    let timeout = TIMEOUT_MS.load(Ordering::Relaxed);
    if !RUNNING.load(Ordering::Relaxed) {
        return timeout;
    }
    let elapsed = hal_timer_get_ms().wrapping_sub(last_kick());
    u32::try_from(u64::from(timeout).saturating_sub(elapsed)).unwrap_or(0)
}

/// Whether the last reset was a watchdog's (until cleared)
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_caused_reset() -> bool {
    !RESET_FLAG_CLEARED.load(Ordering::Relaxed) && hal_reset_cause() == RESET_WATCHDOG
}

/// Forget the watchdog reset
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_clear_reset_flag() {
    RESET_FLAG_CLEARED.store(true, Ordering::Relaxed);
}

/// The IWDG has no early warning
#[unsafe(no_mangle)]
pub extern "C" fn hal_wdt_set_warning_callback(
    _callback: Option<unsafe extern "C" fn(*mut c_void)>,
    _context: *mut c_void,
) -> i32 {
    rc::ERROR
}

/// What caused the last reset (read once from RCC CSR, then the flags are
/// cleared so the next reset reports its own cause)
#[unsafe(no_mangle)]
pub extern "C" fn hal_reset_cause() -> u32 {
    let cached = RESET_CAUSE.load(Ordering::Relaxed);
    if cached != u32::MAX {
        return cached;
    }
    let csr = pac::RCC.csr().read();
    // NRST is pulled low by every internal reset, so the pin flag is checked
    // last. The L4 has no separate power-on flag: a power-on sets BORRSTF.
    let cause = if csr.iwdgrstf() || csr.wwdgrstf() {
        RESET_WATCHDOG
    } else if csr.sftrstf() {
        RESET_SOFTWARE
    } else if csr.borrstf() {
        RESET_POWER_ON
    } else if csr.pinrstf() {
        RESET_PIN
    } else {
        RESET_UNKNOWN
    };
    pac::RCC.csr().modify(|w| w.set_rmvf(true));
    RESET_CAUSE.store(cause, Ordering::Relaxed);
    cause
}
