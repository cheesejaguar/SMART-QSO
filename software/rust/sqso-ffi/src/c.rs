//! The C side: functions Rust calls, result codes, and C-owned structs.
//!
//! C enums are passed as `u32` (or `i32` where they have negative values).
//! The STM32L4 build compiles C with `-fno-short-enums`, so C enums are
//! 32 bits on every target, as on the host where the tests run.

use core::ffi::c_char;

/// `SmartQsoResult_t`
pub(crate) mod rc {
    pub(crate) const OK: i32 = 0;
    pub(crate) const ERROR: i32 = -1;
    pub(crate) const NULL_PTR: i32 = -2;
    pub(crate) const INVALID: i32 = -3;
    pub(crate) const BUSY: i32 = -5;
    pub(crate) const NO_MEM: i32 = -6;
    pub(crate) const IO: i32 = -7;
    pub(crate) const PARAM: i32 = -9;
}

/// `FaultType_t` values used here
pub(crate) mod fault_type {
    pub(crate) const POWER: u32 = 1;
    pub(crate) const MODE_CHANGE: u32 = 2;
    pub(crate) const THERMAL: u32 = 3;
    pub(crate) const POWER_CRITICAL: u32 = 4;
    pub(crate) const WATCHDOG: u32 = 5;
    pub(crate) const UART: u32 = 6;
    pub(crate) const RECOVERY: u32 = 12;
    pub(crate) const INIT: u32 = 16;
    pub(crate) const COMMAND: u32 = 19;
}

/// `FaultSeverity_t`
pub(crate) mod severity {
    pub(crate) const INFO: u32 = 1;
    pub(crate) const WARNING: u32 = 2;
    pub(crate) const ERROR: u32 = 3;
    pub(crate) const CRITICAL: u32 = 4;
}

/// `HalFlashRegion_t` values used here
pub(crate) const FLASH_REGION_AUTH: u32 = 6;

/// `PowerState_t` (system_state.h)
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct PowerState {
    pub(crate) battery_voltage: f64,
    pub(crate) battery_current: f64,
    pub(crate) state_of_charge: f64,
    pub(crate) solar_power: f64,
    pub(crate) total_energy_wh: f64,
    pub(crate) power_mode: u32,
    pub(crate) payload_enabled: bool,
    pub(crate) mode_entry_time_ms: u32,
}

/// `AdcsState_t` (system_state.h)
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct AdcsState {
    pub(crate) mag_ut: [f32; 3],
    pub(crate) gyro_dps: [f32; 3],
    pub(crate) sun_vector: [f32; 3],
    pub(crate) detumbled: bool,
    pub(crate) sun_acquired: bool,
    pub(crate) last_update_ms: u32,
}

/// `SmContext_t` (state_machine.h), owned by system_state.c
#[repr(C)]
pub struct SmContext {
    pub(crate) current_state: u32,
    pub(crate) previous_state: u32,
    pub(crate) transition_count: u32,
    pub(crate) last_transition_ms: u64,
    pub(crate) state_entry_ms: u64,
    pub(crate) is_initialized: bool,
}

unsafe extern "C" {
    // Rust panics end here (src/rust_panic.c)
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) fn smart_qso_rust_panic(file: *const c_char, line: u32) -> !;

    // Assertions
    pub(crate) fn smart_qso_assert_failed(file: *const c_char, line: u32, message: *const c_char);

    // HAL
    #[cfg_attr(not(target_os = "none"), allow(dead_code))]
    pub(crate) fn hal_timer_get_ms() -> u64;
    #[cfg_attr(not(target_os = "none"), allow(dead_code))]
    pub(crate) fn hal_flash_region_size(region: u32) -> usize;
    pub(crate) fn hal_flash_read(region: u32, offset: u32, data: *mut u8, len: usize) -> i32;
    pub(crate) fn hal_flash_write(region: u32, offset: u32, data: *const u8, len: usize) -> i32;
    pub(crate) fn hal_flash_erase(region: u32) -> i32;
    pub(crate) fn hal_radio_init() -> i32;
    pub(crate) fn hal_radio_receive(frame: *mut u8, max_len: usize, len: *mut usize) -> i32;
    pub(crate) fn hal_gpio_config(pin: u32, direction: u32, pull: u32) -> i32;
    pub(crate) fn hal_gpio_get(pin: u32) -> bool;
    pub(crate) fn hal_uart_init(port: u32, config: *const core::ffi::c_void) -> i32;
    pub(crate) fn hal_uart_available(port: u32) -> usize;
    pub(crate) fn hal_uart_flush(port: u32) -> i32;
    pub(crate) fn hal_uart_read(port: u32, data: *mut u8, len: usize, timeout_ms: u32, got: *mut usize) -> i32;
    pub(crate) fn hal_uart_write(port: u32, data: *const u8, len: usize) -> i32;

    // System state
    pub(crate) fn sys_get_sm_context() -> *mut SmContext;
    pub(crate) fn sys_get_operational_state() -> u32;
    pub(crate) fn sys_get_power_mode() -> u32;
    pub(crate) fn sys_is_payload_enabled() -> bool;
    pub(crate) fn sys_has_thermal_fault() -> bool;
    pub(crate) fn sys_get_power_state(state: *mut PowerState) -> i32;
    pub(crate) fn sys_get_adcs_state(state: *mut AdcsState) -> i32;
    pub(crate) fn sys_increment_command_count() -> i32;
    pub(crate) fn sys_set_beacon_interval(interval_s: u16) -> i32;
    pub(crate) fn sys_set_payload_enabled(enabled: bool) -> i32;

    // EPS and payload guard
    pub(crate) fn eps_set_power_mode(mode: u32, soc: f64) -> i32;
    pub(crate) fn eps_get_soc() -> f64;
    pub(crate) fn eps_control_beacon(enable: bool) -> i32;
    pub(crate) fn eps_control_payload(enable: bool, soc: f64) -> i32;
    pub(crate) fn eps_is_payload_enabled() -> bool;
    pub(crate) fn payload_guard_may_enable(now_ms: u64) -> bool;

    // RF and contact
    pub(crate) fn contact_monitor_contact();
    pub(crate) fn contact_monitor_set_timeout_days(days: u16) -> i32;
    pub(crate) fn rf_control_tx_off() -> i32;
    pub(crate) fn rf_control_tx_on() -> i32;
    pub(crate) fn rf_control_set_power_level(level: u32) -> i32;
    pub(crate) fn rf_silence_arm() -> i32;
    pub(crate) fn deployment_arm() -> i32;
}

/// Log a fault with a static description
pub(crate) fn log(fault_type: u32, severity: u32, description: &core::ffi::CStr, soc: f64) {
    let _ = crate::fault_mgmt::add(fault_type, severity, description.to_bytes(), soc);
}

/// Report a failed check to the C assertion handler (outside any module
/// state: the handler writes the fault log)
pub(crate) fn assert_failed(message: &core::ffi::CStr, line: u32) {
    // SAFETY: both strings are NUL-terminated statics
    unsafe { smart_qso_assert_failed(c"sqso-ffi".as_ptr(), line, message.as_ptr()) };
}

/// Write one formatted line to the console (cut at 159 bytes)
pub(crate) fn console(args: core::fmt::Arguments<'_>) {
    struct Line {
        buf: [u8; 160],
        len: usize,
    }
    impl core::fmt::Write for Line {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for b in s.bytes() {
                if let Some(slot) = self.buf.get_mut(self.len).filter(|_| self.len < 159) {
                    *slot = if b == 0 { b'?' } else { b };
                    self.len += 1;
                }
            }
            Ok(())
        }
    }
    let mut line = Line { buf: [0; 160], len: 0 };
    let _ = core::fmt::write(&mut line, args);
    console_line(line.buf.get(..line.len).unwrap_or(&[]));
}

/// Current time, ms
pub(crate) fn now_ms() -> u64 {
    crate::time::now_ms()
}

/// The flash HAL as an `sqso_fsw::nvm::Flash`
pub(crate) struct HalFlash;

impl sqso_fsw::nvm::Flash for HalFlash {
    fn read(&mut self, region: u32, offset: u32, buf: &mut [u8]) -> Result<(), sqso_fsw::nvm::FlashError> {
        // SAFETY: buf is a valid writable slice of buf.len() bytes
        match unsafe { hal_flash_read(region, offset, buf.as_mut_ptr(), buf.len()) } {
            rc::OK => Ok(()),
            code => Err(sqso_fsw::nvm::FlashError(code)),
        }
    }

    fn write(&mut self, region: u32, offset: u32, data: &[u8]) -> Result<(), sqso_fsw::nvm::FlashError> {
        // SAFETY: data is a valid slice of data.len() bytes
        match unsafe { hal_flash_write(region, offset, data.as_ptr(), data.len()) } {
            rc::OK => Ok(()),
            code => Err(sqso_fsw::nvm::FlashError(code)),
        }
    }

    fn erase(&mut self, region: u32) -> Result<(), sqso_fsw::nvm::FlashError> {
        // SAFETY: no pointers
        match unsafe { hal_flash_erase(region) } {
            rc::OK => Ok(()),
            code => Err(sqso_fsw::nvm::FlashError(code)),
        }
    }
}

/// Write one line to the console: stdout on the host, the debug UART on
/// the target
pub(crate) fn console_line(line: &[u8]) {
    #[cfg(not(target_os = "none"))]
    crate::os::print_line(line);
    #[cfg(target_os = "none")]
    {
        const UART_PORT_DEBUG: u32 = 1;
        // SAFETY: both are live slices. Console output is best effort
        unsafe {
            let _ = hal_uart_write(UART_PORT_DEBUG, line.as_ptr(), line.len());
            let _ = hal_uart_write(UART_PORT_DEBUG, b"\r\n".as_ptr(), 2);
        }
    }
}
