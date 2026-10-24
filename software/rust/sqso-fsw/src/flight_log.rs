//! Flight log ring buffer (`flight_log.h`).
//!
//! The newest [`BUFFER_SIZE`] entries, each with a time stamp, level,
//! module name, message and sequence number; when full, a new entry
//! replaces the oldest (counted as dropped). A compile-time minimum level
//! ([`MIN_LEVEL`]) and a runtime level filter what is recorded. The caller
//! formats the message (C `vsnprintf`) and does the output (console line,
//! callback).
//!
//! @note MISRA C:2012 Rule 21.6 (SQSO-CS-001): no stdio.h in production code
//! @note NPR 7150.2 (standard, not a requirement ID): logging and telemetry

use core::fmt;

use crate::safe_string;

/// Message bytes including the NUL (`LOG_MAX_MESSAGE_LEN`)
pub const MAX_MESSAGE_LEN: usize = 128;
/// Entries kept (`LOG_BUFFER_SIZE`)
pub const BUFFER_SIZE: usize = 64;
/// Module name bytes including the NUL (`LOG_MAX_MODULE_LEN`)
pub const MAX_MODULE_LEN: usize = 16;

/// `LogLevel_t`
pub mod level {
    /// Very detailed tracing
    pub const TRACE: u32 = 0;
    /// Debug information
    pub const DEBUG: u32 = 1;
    /// Informational messages
    pub const INFO: u32 = 2;
    /// Warning conditions
    pub const WARNING: u32 = 3;
    /// Error conditions
    pub const ERROR: u32 = 4;
    /// Critical errors
    pub const CRITICAL: u32 = 5;
    /// Logging disabled
    pub const OFF: u32 = 6;
}

/// The compile-time minimum level (`LOG_MIN_LEVEL`, DEBUG)
pub const MIN_LEVEL: u32 = level::DEBUG;

/// `LogOutput_t` bits
pub mod output {
    /// Store in the ring buffer
    pub const BUFFER: u8 = 0x01;
    /// Send to the debug console
    pub const UART: u8 = 0x02;
    /// Include in telemetry
    pub const TELEMETRY: u8 = 0x04;
}

/// One log entry (the C `LogEntry_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Mission time, ms (low 32 bits)
    pub timestamp_ms: u32,
    /// `LogLevel_t`
    pub level: u32,
    /// Module name, NUL-terminated
    pub module: [u8; MAX_MODULE_LEN],
    /// Message, NUL-terminated
    pub message: [u8; MAX_MESSAGE_LEN],
    /// Sequence number
    pub sequence: u16,
}

impl Entry {
    /// An all-zero entry
    pub const ZERO: Entry =
        Entry { timestamp_ms: 0, level: 0, module: [0; MAX_MODULE_LEN], message: [0; MAX_MESSAGE_LEN], sequence: 0 };

    /// The module name up to its NUL
    pub fn module(&self) -> &[u8] {
        safe_string::until_nul(&self.module)
    }

    /// The message up to its NUL
    pub fn message(&self) -> &[u8] {
        safe_string::until_nul(&self.message)
    }

    /// The console line: `[timestamp][LEVEL][module] message`
    pub fn write_line(&self, out: &mut LineWriter<'_>) {
        let _ = fmt::write(out, format_args!("[{:010}][{}][", self.timestamp_ms, level_name(self.level)));
        out.push(self.module());
        out.push(b"] ");
        out.push(self.message());
    }
}

/// Logging statistics (the C `LogStats_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Log calls
    pub total_logs: u32,
    /// Calls filtered out by level
    pub filtered_logs: u32,
    /// Entries overwritten while the buffer was full
    pub dropped_logs: u32,
    /// TRACE entries
    pub trace_count: u32,
    /// DEBUG entries
    pub debug_count: u32,
    /// INFO entries
    pub info_count: u32,
    /// WARNING entries
    pub warning_count: u32,
    /// ERROR entries
    pub error_count: u32,
    /// CRITICAL entries
    pub critical_count: u32,
    /// Entries held
    pub buffer_entries: u16,
    /// Most entries held
    pub buffer_high_water: u16,
}

impl Stats {
    const ZERO: Stats = Stats {
        total_logs: 0,
        filtered_logs: 0,
        dropped_logs: 0,
        trace_count: 0,
        debug_count: 0,
        info_count: 0,
        warning_count: 0,
        error_count: 0,
        critical_count: 0,
        buffer_entries: 0,
        buffer_high_water: 0,
    };
}

/// The five-character console name of a level
pub fn level_name(level: u32) -> &'static str {
    match level {
        level::TRACE => "TRACE",
        level::DEBUG => "DEBUG",
        level::INFO => "INFO ",
        level::WARNING => "WARN ",
        level::ERROR => "ERROR",
        level::CRITICAL => "CRIT ",
        level::OFF => "OFF  ",
        _ => "?????",
    }
}

/// A formatted message as the C formatter left it
#[derive(Clone, Copy, Debug)]
pub struct Formatted<'a> {
    /// The formatter's output buffer (NUL-terminated by it)
    pub text: &'a [u8; MAX_MESSAGE_LEN],
    /// Its return value: the full length, or negative on an error
    pub written: i32,
}

/// The log
#[derive(Clone, Debug)]
pub struct FlightLog {
    buffer: [Entry; BUFFER_SIZE],
    write: usize,
    read: usize,
    count: usize,
    sequence: u16,
    level: u32,
    outputs: u8,
    stats: Stats,
    initialized: bool,
}

impl FlightLog {
    /// Empty, not initialized: level DEBUG, output to the buffer
    pub const fn new() -> Self {
        FlightLog {
            buffer: [Entry::ZERO; BUFFER_SIZE],
            write: 0,
            read: 0,
            count: 0,
            sequence: 0,
            level: level::DEBUG,
            outputs: output::BUFFER,
            stats: Stats::ZERO,
            initialized: false,
        }
    }

    /// Clear everything; level DEBUG and the given outputs
    pub fn init(&mut self, outputs: u8) {
        *self = FlightLog { outputs, initialized: true, ..FlightLog::new() };
    }

    /// Whether [`Self::init`] has run
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Set the runtime level; false (unchanged) above OFF
    pub fn set_level(&mut self, level: u32) -> bool {
        if level > level::OFF {
            return false;
        }
        self.level = level;
        true
    }

    /// The runtime level
    pub fn level(&self) -> u32 {
        self.level
    }

    /// Set the output bits
    pub fn set_outputs(&mut self, outputs: u8) {
        self.outputs = outputs;
    }

    /// The output bits
    pub fn outputs(&self) -> u8 {
        self.outputs
    }

    /// Count a log call; whether it passes the level filters (then it is
    /// counted by level and the caller records it with [`Self::record`])
    pub fn admit(&mut self, level: u32) -> bool {
        let s = &mut self.stats;
        s.total_logs = s.total_logs.wrapping_add(1);
        if level < MIN_LEVEL || level < self.level {
            s.filtered_logs = s.filtered_logs.wrapping_add(1);
            return false;
        }
        let counter = match level {
            level::TRACE => &mut s.trace_count,
            level::DEBUG => &mut s.debug_count,
            level::INFO => &mut s.info_count,
            level::WARNING => &mut s.warning_count,
            level::ERROR => &mut s.error_count,
            level::CRITICAL => &mut s.critical_count,
            _ => return true,
        };
        *counter = counter.wrapping_add(1);
        true
    }

    /// Record an admitted entry, overwriting the oldest if full; a copy of it.
    /// `module` is cut at 15 bytes; `message` is `None` without a format.
    pub fn record(&mut self, level: u32, now_ms: u64, module: Option<&[u8]>, message: Option<Formatted<'_>>) -> Entry {
        let mut entry = Entry {
            timestamp_ms: u32::try_from(now_ms & 0xFFFF_FFFF).unwrap_or(0),
            level,
            sequence: self.sequence,
            ..Entry::ZERO
        };
        self.sequence = self.sequence.wrapping_add(1);
        if let Some(m) = module {
            let _ = safe_string::copy(&mut entry.module, m, MAX_MODULE_LEN - 1);
        }
        // A formatter error leaves the message empty
        if let Some(f) = message.filter(|f| f.written >= 0) {
            entry.message = *f.text;
            if let Some(end) = entry.message.last_mut() {
                *end = 0;
            }
            if usize::try_from(f.written).unwrap_or(usize::MAX) >= MAX_MESSAGE_LEN {
                // Truncated: end with an ellipsis
                if let Some(tail) = entry.message.get_mut(MAX_MESSAGE_LEN - 4..) {
                    tail.copy_from_slice(b"...\0");
                }
            }
        }
        if let Some(slot) = self.buffer.get_mut(self.write) {
            *slot = entry;
        }

        self.write = (self.write + 1) % BUFFER_SIZE;
        if self.count < BUFFER_SIZE {
            self.count += 1;
        } else {
            self.read = (self.read + 1) % BUFFER_SIZE;
            self.stats.dropped_logs = self.stats.dropped_logs.wrapping_add(1);
        }
        let held = u16::try_from(self.count).unwrap_or(u16::MAX);
        self.stats.buffer_entries = held;
        self.stats.buffer_high_water = self.stats.buffer_high_water.max(held);
        entry
    }

    /// The entry at `index` (0 is the oldest)
    pub fn get(&self, index: usize) -> Option<&Entry> {
        if index >= self.count {
            return None;
        }
        self.buffer.get((self.read + index) % BUFFER_SIZE)
    }

    /// Entries held
    pub fn len(&self) -> usize {
        self.count
    }

    /// No entries held
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Empty the buffer (statistics other than the entry count are kept)
    pub fn clear(&mut self) {
        self.write = 0;
        self.read = 0;
        self.count = 0;
        self.stats.buffer_entries = 0;
    }

    /// The statistics
    pub fn stats(&self) -> Stats {
        self.stats
    }
}

impl Default for FlightLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Formats text into a byte buffer, cutting what does not fit and keeping
/// room for a NUL
pub struct LineWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> LineWriter<'a> {
    /// Write into `buf`
    pub fn new(buf: &'a mut [u8]) -> Self {
        LineWriter { buf, len: 0 }
    }

    /// Append raw bytes
    pub fn push(&mut self, bytes: &[u8]) {
        let room = self.buf.len().saturating_sub(1).saturating_sub(self.len);
        let n = bytes.len().min(room);
        if let (Some(dst), Some(src)) = (self.buf.get_mut(self.len..self.len + n), bytes.get(..n)) {
            dst.copy_from_slice(src);
            self.len += n;
        }
    }

    /// The text so far
    pub fn as_bytes(&self) -> &[u8] {
        self.buf.get(..self.len).unwrap_or(&[])
    }
}

impl fmt::Write for LineWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push(s.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &[u8]) -> [u8; MAX_MESSAGE_LEN] {
        let mut t = [0u8; MAX_MESSAGE_LEN];
        let _ = safe_string::copy(&mut t, s, usize::MAX);
        t
    }

    fn log(l: &mut FlightLog, level: u32, msg: &[u8]) -> bool {
        if !l.admit(level) {
            return false;
        }
        let t = text(msg);
        let _ =
            l.record(level, 0, Some(b"TEST"), Some(Formatted { text: &t, written: i32::try_from(msg.len()).unwrap() }));
        true
    }

    #[test]
    fn filters_by_level() {
        let mut l = FlightLog::new();
        l.init(output::BUFFER);
        assert!(l.set_level(level::WARNING));
        assert!(!l.set_level(100));
        assert_eq!(l.level(), level::WARNING);
        for lv in [level::TRACE, level::DEBUG, level::INFO] {
            assert!(!log(&mut l, lv, b"x"));
        }
        assert!(log(&mut l, level::WARNING, b"w"));
        assert!(log(&mut l, level::ERROR, b"e"));
        assert_eq!(l.len(), 2);
        let s = l.stats();
        assert_eq!((s.total_logs, s.filtered_logs, s.warning_count, s.error_count), (5, 3, 1, 1));

        // TRACE is below the compile-time minimum at any runtime level
        assert!(l.set_level(level::TRACE));
        assert!(!log(&mut l, level::TRACE, b"t"));
        assert_eq!(l.stats().trace_count, 0);
    }

    #[test]
    fn entries_in_order_and_wrap() {
        let mut l = FlightLog::new();
        for i in 0..(BUFFER_SIZE + 10) {
            let mut t = [0u8; MAX_MESSAGE_LEN];
            let mut w = LineWriter::new(&mut t);
            let _ = fmt::write(&mut w, format_args!("Message {i}"));
            let n = i32::try_from(w.as_bytes().len()).unwrap();
            assert!(l.admit(level::INFO));
            let e = l.record(level::INFO, 0x1_0000_0005, Some(b"MOD"), Some(Formatted { text: &t, written: n }));
            assert_eq!(e.timestamp_ms, 5);
        }
        assert_eq!(l.len(), BUFFER_SIZE);
        let s = l.stats();
        assert_eq!((s.dropped_logs, s.buffer_entries, s.buffer_high_water), (10, 64, 64));
        assert_eq!(l.get(0).unwrap().message(), b"Message 10");
        assert_eq!(l.get(0).unwrap().sequence, 10);
        assert_eq!(l.get(BUFFER_SIZE - 1).unwrap().message(), b"Message 73");
        assert!(l.get(BUFFER_SIZE).is_none());
        l.clear();
        assert!(l.is_empty());
        assert_eq!(l.stats().dropped_logs, 10);
    }

    #[test]
    fn message_forms() {
        let mut l = FlightLog::new();
        // No format: empty message; no module: empty name
        let e = l.record(level::INFO, 0, None, None);
        assert_eq!((e.message(), e.module()), (&b""[..], &b""[..]));
        // Formatter error: empty
        let t = text(b"partial");
        let e = l.record(level::INFO, 0, Some(b"M"), Some(Formatted { text: &t, written: -1 }));
        assert_eq!(e.message(), b"");
        // Truncated: 124 characters and an ellipsis
        let t = text(&[b'a'; 200]);
        let e = l.record(level::INFO, 0, Some(b"M"), Some(Formatted { text: &t, written: 200 }));
        assert_eq!(e.message().len(), 127);
        assert_eq!(&e.message()[124..], b"...");
        assert_eq!(e.message[123], b'a');
        // Module names are cut at 15 bytes
        let e = l.record(level::INFO, 0, Some(b"ABCDEFGHIJKLMNOPQRS"), None);
        assert_eq!(e.module(), b"ABCDEFGHIJKLMNO");
    }

    #[test]
    fn console_line() {
        let mut l = FlightLog::new();
        let t = text(b"Value: 42");
        let e = l.record(level::WARNING, 1234, Some(b"EPS"), Some(Formatted { text: &t, written: 9 }));
        let mut buf = [0u8; 200];
        let mut w = LineWriter::new(&mut buf);
        e.write_line(&mut w);
        assert_eq!(w.as_bytes(), b"[0000001234][WARN ][EPS] Value: 42");
        assert_eq!(level_name(77), "?????");

        let mut small = [0u8; 4];
        let mut w = LineWriter::new(&mut small);
        w.push(b"abcdef");
        assert_eq!(w.as_bytes(), b"abc");
    }

    #[test]
    fn unknown_level_is_kept_uncounted() {
        let mut l = FlightLog::new();
        assert!(l.admit(level::OFF));
        assert!(l.admit(42));
        let s = l.stats();
        assert_eq!(s.total_logs, 2);
        assert_eq!(s.critical_count + s.error_count + s.info_count, 0);
    }
}
