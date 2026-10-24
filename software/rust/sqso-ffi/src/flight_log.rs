//! `flight_log.h`: the flight log (`sqso_fsw::flight_log`).
//!
//! `log_write()` and `log_vwrite()` take a printf format and C variadic
//! arguments, which stable Rust can neither define nor read, so those two
//! stay in C (`src/flight_log_fmt.c`): they ask [`flight_log_admit`]
//! whether the level passes, format the message with `vsnprintf`, and hand
//! the text to [`flight_log_record`]. Everything else is here.
//!
//! Output (the console line on the host, the registered callback) runs
//! with no log state borrowed, so a callback may log in turn.
//!
//! @note MISRA C:2012 Rule 21.6 (SQSO-CS-001): no stdio.h in production code

use core::ffi::{c_char, c_int};

use sqso_fsw::flight_log::{Entry, FlightLog, Formatted, MAX_MESSAGE_LEN, MAX_MODULE_LEN, Stats, output};

use crate::c::{self, rc};
use crate::cell::Global;
use crate::{ptr, safe_string};

/// `LogOutputCallback_t`
pub type LogOutputCallback = unsafe extern "C" fn(entry: *const Entry);

struct Log {
    log: FlightLog,
    callback: Option<LogOutputCallback>,
}

static LOG: Global<Log> = Global::new(Log { log: FlightLog::new(), callback: None });

/// The outputs `log_init()` selects: the buffer, and the console in the
/// host simulation
#[cfg(not(target_os = "none"))]
const DEFAULT_OUTPUTS: u8 = output::BUFFER | output::UART;
#[cfg(target_os = "none")]
const DEFAULT_OUTPUTS: u8 = output::BUFFER;

/// Write one formatted line to stderr (host only; cut at 255 bytes)
#[cfg(not(target_os = "none"))]
pub(crate) fn stderr_line(write: impl FnOnce(&mut sqso_fsw::flight_log::LineWriter<'_>)) {
    let mut buf = [0u8; 256];
    let mut line = sqso_fsw::flight_log::LineWriter::new(&mut buf);
    write(&mut line);
    crate::os::eprint_line(line.as_bytes());
}

/// Send an entry to the outputs (no log state borrowed)
fn output_entry(entry: &Entry, outputs: u8, callback: Option<LogOutputCallback>) {
    // The console line is the simulation's; the flight build has no
    // console output here (the debug UART is not driven by the log)
    #[cfg(not(target_os = "none"))]
    if outputs & output::UART != 0 {
        stderr_line(|line| entry.write_line(line));
    }
    #[cfg(target_os = "none")]
    let _ = outputs;
    if let Some(cb) = callback {
        // SAFETY: a C callback given a pointer to a live entry
        unsafe { cb(entry) };
    }
}

/// Initialize the log: empty, level DEBUG, default outputs, no callback
#[unsafe(no_mangle)]
pub extern "C" fn log_init() -> i32 {
    LOG.with(|l| {
        l.log.init(DEFAULT_OUTPUTS);
        l.callback = None;
        rc::OK
    })
    .unwrap_or(rc::BUSY)
}

/// Set the runtime level filter
#[unsafe(no_mangle)]
pub extern "C" fn log_set_level(level: u32) -> i32 {
    match LOG.with(|l| l.log.set_level(level)) {
        Some(true) => rc::OK,
        Some(false) => rc::PARAM,
        None => rc::BUSY,
    }
}

/// The runtime level filter
#[unsafe(no_mangle)]
pub extern "C" fn log_get_level() -> u32 {
    LOG.with(|l| l.log.level()).unwrap_or(sqso_fsw::flight_log::level::DEBUG)
}

/// Set the output destinations (`LogOutput_t` bits)
#[unsafe(no_mangle)]
pub extern "C" fn log_set_outputs(outputs: u8) -> i32 {
    LOG.with(|l| l.log.set_outputs(outputs)).map_or(rc::BUSY, |()| rc::OK)
}

/// Register the output callback (NULL removes it)
#[unsafe(no_mangle)]
pub extern "C" fn log_register_callback(callback: Option<LogOutputCallback>) -> i32 {
    LOG.with(|l| l.callback = callback).map_or(rc::BUSY, |()| rc::OK)
}

/// For `src/flight_log_fmt.c`: count a log call (initializing the log if
/// needed); whether its level passes the filters, so it is to be formatted
/// and recorded
#[unsafe(no_mangle)]
pub extern "C" fn flight_log_admit(level: u32) -> bool {
    LOG.with(|l| {
        if !l.log.is_initialized() {
            l.log.init(DEFAULT_OUTPUTS);
            l.callback = None;
        }
        l.log.admit(level)
    })
    .unwrap_or(false)
}

/// For `src/flight_log_fmt.c`: record an admitted entry and output it.
/// `message` is the formatter's buffer of `LOG_MAX_MESSAGE_LEN` bytes (NULL
/// without a format) and `written` its return value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn flight_log_record(
    level: u32,
    module: *const c_char,
    message: *const c_char,
    written: c_int,
) -> i32 {
    // SAFETY: the C contract: NULL or a C string (read to 15 bytes at most)
    let module = unsafe { safe_string::bounded(module, MAX_MODULE_LEN - 1) };
    let text = if message.is_null() {
        None
    } else {
        // SAFETY: flight_log_fmt.c passes its message buffer of
        // LOG_MAX_MESSAGE_LEN initialized bytes
        unsafe { message.cast::<[u8; MAX_MESSAGE_LEN]>().as_ref() }
    };
    record(level, module, text.map(|t| Formatted { text: t, written }))
}

/// Record an admitted entry and output it (no log state borrowed while
/// it is output)
fn record(level: u32, module: Option<&[u8]>, message: Option<Formatted<'_>>) -> i32 {
    let now = c::now_ms();
    let recorded = LOG.with(|l| {
        let entry = l.log.record(level, now, module, message);
        (entry, l.log.outputs(), l.callback)
    });
    match recorded {
        Some((entry, outputs, callback)) => {
            output_entry(&entry, outputs, callback);
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Log a message from Rust: the same as `log_write(level, module, "%s",
/// message)` in C. `module` and `message` end at their first NUL, if any;
/// the module name is cut at 15 bytes and the message at 127 (then it ends
/// with "..."). Returns `SMART_QSO_OK` (also when the level is filtered
/// out), or `SMART_QSO_ERROR_BUSY` if the log is in use (re-entered).
pub(crate) fn write_str(level: u32, module: &[u8], message: &[u8]) -> i32 {
    if !flight_log_admit(level) {
        return rc::OK;
    }
    let mut text = [0u8; MAX_MESSAGE_LEN];
    let message = sqso_fsw::safe_string::until_nul(message);
    let _ = sqso_fsw::safe_string::copy(&mut text, message, usize::MAX);
    // As vsnprintf reports it: the full length
    let written = i32::try_from(message.len()).unwrap_or(i32::MAX);
    record(level, Some(sqso_fsw::safe_string::until_nul(module)), Some(Formatted { text: &text, written }))
}

/// Entry at `index` (0 is the oldest)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn log_get_entry(index: u16, entry: *mut Entry) -> i32 {
    // SAFETY: the C contract: NULL or a LogEntry_t (same layout)
    let Some(out) = (unsafe { ptr::out(entry) }) else { return rc::NULL_PTR };
    match LOG.with(|l| l.log.get(usize::from(index)).copied()) {
        Some(Some(e)) => {
            *out = e;
            rc::OK
        }
        Some(None) => rc::PARAM,
        None => rc::BUSY,
    }
}

/// Entries in the buffer
#[unsafe(no_mangle)]
pub extern "C" fn log_get_count() -> u16 {
    LOG.with(|l| u16::try_from(l.log.len()).unwrap_or(u16::MAX)).unwrap_or(0)
}

/// Empty the buffer
#[unsafe(no_mangle)]
pub extern "C" fn log_clear() -> i32 {
    LOG.with(|l| l.log.clear()).map_or(rc::BUSY, |()| rc::OK)
}

/// The statistics
#[unsafe(no_mangle)]
pub unsafe extern "C" fn log_get_stats(stats: *mut Stats) -> i32 {
    // SAFETY: the C contract: NULL or a LogStats_t (same layout)
    let Some(out) = (unsafe { ptr::out(stats) }) else { return rc::NULL_PTR };
    match LOG.with(|l| l.log.stats()) {
        Some(s) => {
            *out = s;
            rc::OK
        }
        None => rc::BUSY,
    }
}

/// Output every buffered entry, oldest first (if buffering is selected)
#[unsafe(no_mangle)]
pub extern "C" fn log_flush() -> i32 {
    match LOG.with(|l| l.log.outputs() & output::BUFFER != 0) {
        Some(true) => {}
        Some(false) => return rc::OK,
        None => return rc::BUSY,
    }
    // The count is read again for each entry: a callback may log
    let mut i = 0;
    loop {
        let next = LOG.with(|l| l.log.get(i).copied().map(|e| (e, l.log.outputs(), l.callback)));
        match next {
            Some(Some((entry, outputs, callback))) => output_entry(&entry, outputs, callback),
            Some(None) => return rc::OK,
            None => return rc::BUSY,
        }
        i += 1;
    }
}
