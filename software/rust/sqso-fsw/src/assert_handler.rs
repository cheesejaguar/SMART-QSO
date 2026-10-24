//! Assertion failure log and response policy (`assert_handler.h`).
//!
//! Every failed assertion is counted and logged (file, line, type,
//! severity, message); a repeat of the same file and line raises the
//! entry's occurrence count instead of adding an entry. The severity
//! decides the response: a warning continues, an error returns an error,
//! a critical failure enters safe mode and a fatal one resets. The caller
//! carries out safe mode and reset (fault log, callbacks): see [`Action`].
//!
//! Assertions are never compiled out, flight builds included (SW-08).
//!
//! @note NPR 7150.2 (standard, not a requirement ID): software safety
//! @note JPL Power of Ten rule 5 (SQSO-CS-001 §3): at least two assertions per function on average

use crate::safe_string;

/// Message bytes including the NUL (`ASSERT_MAX_MSG_LEN`)
pub const MAX_MSG_LEN: usize = 64;
/// Entries in the failure log (`ASSERT_MAX_FAILURE_LOG`)
pub const MAX_FAILURE_LOG: usize = 16;
/// File name bytes including the NUL (`ASSERT_MAX_FILENAME_LEN`)
pub const MAX_FILENAME_LEN: usize = 32;

/// `AssertSeverity_t`
pub mod severity {
    /// Log and continue
    pub const WARNING: u32 = 0;
    /// Log and return an error
    pub const ERROR: u32 = 1;
    /// Log and enter safe mode
    pub const CRITICAL: u32 = 2;
    /// Log and reset
    pub const FATAL: u32 = 3;
}

/// `AssertType_t` values used here
pub mod kind {
    /// Function precondition
    pub const PRECONDITION: u32 = 0;
    /// Loop or state invariant
    pub const INVARIANT: u32 = 2;
}

/// `AssertAction_t`: the response to a failure
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Action {
    /// Continue execution
    Continue = 0,
    /// Return from the function with an error
    ReturnError = 1,
    /// Enter safe mode (the caller fault-logs it and calls the safe-mode callback)
    SafeMode = 2,
    /// Reset (the caller fault-logs it and calls the reset callback)
    Reset = 3,
}

/// One logged failure (the C `AssertRecord_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    /// Source file name (no directory), NUL-terminated
    pub filename: [u8; MAX_FILENAME_LEN],
    /// Line number
    pub line: u16,
    /// `AssertType_t`
    pub kind: u32,
    /// `AssertSeverity_t`
    pub severity: u32,
    /// Uptime of the latest failure, s
    pub timestamp_s: u32,
    /// Times this assertion failed
    pub occurrence_count: u32,
    /// Message, NUL-terminated
    pub message: [u8; MAX_MSG_LEN],
}

impl Record {
    /// An all-zero record
    pub const ZERO: Record = Record {
        filename: [0; MAX_FILENAME_LEN],
        line: 0,
        kind: 0,
        severity: 0,
        timestamp_s: 0,
        occurrence_count: 0,
        message: [0; MAX_MSG_LEN],
    };
}

/// Failure statistics (the C `AssertStats_t`)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Assertions evaluated (counted on failure only)
    pub total_checks: u32,
    /// Failures
    pub total_failures: u32,
    /// Warning-level failures
    pub warnings: u32,
    /// Error-level failures (and unknown severities)
    pub errors: u32,
    /// Critical-level failures
    pub criticals: u32,
    /// Fatal-level failures
    pub fatals: u32,
    /// Times safe mode was triggered
    pub safe_mode_triggers: u32,
    /// Times reset was triggered
    pub reset_triggers: u32,
    /// Entries in the log
    pub log_count: u16,
    /// Times the log wrapped
    pub log_overflows: u16,
}

impl Stats {
    const ZERO: Stats = Stats {
        total_checks: 0,
        total_failures: 0,
        warnings: 0,
        errors: 0,
        criticals: 0,
        fatals: 0,
        safe_mode_triggers: 0,
        reset_triggers: 0,
        log_count: 0,
        log_overflows: 0,
    };
}

/// The response to a failure of `severity` (an unknown severity is
/// treated as an error)
pub fn response(severity: u32) -> Action {
    match severity {
        severity::WARNING => Action::Continue,
        severity::CRITICAL => Action::SafeMode,
        severity::FATAL => Action::Reset,
        _ => Action::ReturnError,
    }
}

/// The file name after the last `/` or `\`; "unknown" without a path
pub fn file_name(path: Option<&[u8]>) -> &[u8] {
    match path {
        None => b"unknown",
        Some(p) => match p.iter().rposition(|b| *b == b'/' || *b == b'\\') {
            Some(i) => p.get(i.saturating_add(1)..).unwrap_or(&[]),
            None => p,
        },
    }
}

/// The failure log and statistics
#[derive(Clone, Debug)]
pub struct AssertLog {
    stats: Stats,
    log: [Record; MAX_FAILURE_LOG],
    /// Next slot to write
    index: u16,
    has_critical: bool,
    initialized: bool,
}

impl AssertLog {
    /// Empty and not initialized
    pub const fn new() -> Self {
        AssertLog {
            stats: Stats::ZERO,
            log: [Record::ZERO; MAX_FAILURE_LOG],
            index: 0,
            has_critical: false,
            initialized: false,
        }
    }

    /// Whether [`Self::init`] has run
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Clear everything (statistics, log, critical flag)
    pub fn init(&mut self) {
        *self = AssertLog { initialized: true, ..AssertLog::new() };
    }

    /// Record a failure; the response the severity calls for. `file` and
    /// `message` are the C strings (`None` for NULL), `uptime_s` the time.
    pub fn handle(
        &mut self,
        file: Option<&[u8]>,
        line: u16,
        kind: u32,
        severity: u32,
        message: Option<&[u8]>,
        uptime_s: u32,
    ) -> Action {
        let s = &mut self.stats;
        s.total_checks = s.total_checks.wrapping_add(1);
        s.total_failures = s.total_failures.wrapping_add(1);
        self.log_failure(file_name(file), line, kind, severity, message, uptime_s);
        let s = &mut self.stats;
        let action = response(severity);
        match action {
            Action::Continue => s.warnings = s.warnings.wrapping_add(1),
            Action::ReturnError => s.errors = s.errors.wrapping_add(1),
            Action::SafeMode => {
                s.criticals = s.criticals.wrapping_add(1);
                s.safe_mode_triggers = s.safe_mode_triggers.wrapping_add(1);
                self.has_critical = true;
            }
            Action::Reset => {
                s.fatals = s.fatals.wrapping_add(1);
                s.reset_triggers = s.reset_triggers.wrapping_add(1);
                self.has_critical = true;
            }
        }
        action
    }

    fn log_failure(&mut self, filename: &[u8], line: u16, kind: u32, severity: u32, message: Option<&[u8]>, now: u32) {
        let held = usize::from(self.stats.log_count);
        // The same file and line: count it (names compare over the full
        // field width, as the C did)
        for r in self.log.iter_mut().take(held) {
            if r.line == line && safe_string::compare(&r.filename, filename, MAX_FILENAME_LEN).0 == 0 {
                r.occurrence_count = r.occurrence_count.wrapping_add(1);
                r.timestamp_s = now;
                return;
            }
        }

        let mut record = Record { line, kind, severity, timestamp_s: now, occurrence_count: 1, ..Record::ZERO };
        let _ = safe_string::copy(&mut record.filename, filename, MAX_FILENAME_LEN);
        if let Some(m) = message {
            let _ = safe_string::copy(&mut record.message, m, MAX_MSG_LEN);
        }
        if let Some(slot) = self.log.get_mut(usize::from(self.index)) {
            *slot = record;
        }

        self.index = self.index.saturating_add(1);
        if usize::from(self.index) >= MAX_FAILURE_LOG {
            self.index = 0;
            self.stats.log_overflows = self.stats.log_overflows.wrapping_add(1);
        }
        if held < MAX_FAILURE_LOG {
            self.stats.log_count = self.stats.log_count.saturating_add(1);
        }
    }

    /// The statistics
    pub fn stats(&self) -> Stats {
        self.stats
    }

    /// The `i`-th record held, oldest first
    pub fn record(&self, i: usize) -> Option<&Record> {
        let held = usize::from(self.stats.log_count);
        if i >= held {
            return None;
        }
        // Once the log has wrapped, the oldest is at the write index
        let start = if held >= MAX_FAILURE_LOG { usize::from(self.index) } else { 0 };
        self.log.get(start.checked_add(i)? % MAX_FAILURE_LOG)
    }

    /// Records held
    pub fn len(&self) -> usize {
        usize::from(self.stats.log_count)
    }

    /// No records held
    pub fn is_empty(&self) -> bool {
        self.stats.log_count == 0
    }

    /// Clear the counts and the log; the safe-mode and reset trigger
    /// counts, the overflow count and the critical flag are kept for
    /// mission analysis
    pub fn clear_stats(&mut self) {
        let s = &mut self.stats;
        *s = Stats {
            safe_mode_triggers: s.safe_mode_triggers,
            reset_triggers: s.reset_triggers,
            log_overflows: s.log_overflows,
            ..Stats::ZERO
        };
        self.log = [Record::ZERO; MAX_FAILURE_LOG];
        self.index = 0;
    }

    /// Whether a critical or fatal failure has occurred since [`Self::init`]
    pub fn has_critical_failures(&self) -> bool {
        self.has_critical
    }
}

impl Default for AssertLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(r: &Record) -> &[u8] {
        safe_string::until_nul(&r.filename)
    }

    #[test]
    fn file_names() {
        assert_eq!(file_name(None), b"unknown");
        assert_eq!(file_name(Some(b"a/b/c.c")), b"c.c");
        assert_eq!(file_name(Some(b"a\\b.c")), b"b.c");
        assert_eq!(file_name(Some(b"x.c")), b"x.c");
        assert_eq!(file_name(Some(b"dir/")), b"");
    }

    #[test]
    fn severities_and_counts() {
        let mut a = AssertLog::new();
        a.init();
        assert_eq!(a.handle(Some(b"a.c"), 1, 7, severity::WARNING, Some(b"w"), 5), Action::Continue);
        assert_eq!(a.handle(Some(b"b.c"), 2, 7, severity::ERROR, Some(b"e"), 5), Action::ReturnError);
        assert_eq!(a.handle(Some(b"c.c"), 3, 7, 99, None, 5), Action::ReturnError);
        assert!(!a.has_critical_failures());
        assert_eq!(a.handle(Some(b"d.c"), 4, 6, severity::CRITICAL, Some(b"c"), 5), Action::SafeMode);
        assert!(a.has_critical_failures());
        assert_eq!(a.handle(Some(b"e.c"), 5, 2, severity::FATAL, Some(b"f"), 5), Action::Reset);
        let s = a.stats();
        assert_eq!((s.total_failures, s.total_checks, s.warnings, s.errors), (5, 5, 1, 2));
        assert_eq!((s.criticals, s.fatals, s.safe_mode_triggers, s.reset_triggers), (1, 1, 1, 1));
        assert_eq!(s.log_count, 5);
        assert_eq!(a.record(2).unwrap().message[0], 0);
    }

    #[test]
    fn repeat_counts_once() {
        let mut a = AssertLog::new();
        for t in 0..3 {
            let _ = a.handle(Some(b"src/same.c"), 50, 7, severity::WARNING, Some(b"msg"), t);
        }
        assert_eq!(a.len(), 1);
        let r = a.record(0).unwrap();
        assert_eq!((r.occurrence_count, r.timestamp_s), (3, 2));
        assert_eq!(name(r), b"same.c");
        // Another line of the same file is a new entry
        let _ = a.handle(Some(b"same.c"), 51, 7, severity::WARNING, None, 0);
        assert_eq!(a.len(), 2);
    }

    #[test]
    fn wraps_oldest_first() {
        let mut a = AssertLog::new();
        a.init();
        for line in 0..20u16 {
            let _ = a.handle(Some(b"w.c"), line, 7, severity::WARNING, None, 0);
        }
        let s = a.stats();
        assert_eq!((s.log_count, s.log_overflows), (16, 1));
        assert_eq!(a.record(0).unwrap().line, 4);
        assert_eq!(a.record(15).unwrap().line, 19);
        assert!(a.record(16).is_none());
    }

    #[test]
    fn long_names_and_messages_are_cut() {
        let mut a = AssertLog::new();
        let long = [b'x'; 80];
        let _ = a.handle(Some(&long), 1, 0, severity::ERROR, Some(&long), 0);
        let r = a.record(0).unwrap();
        assert_eq!(name(r).len(), MAX_FILENAME_LEN - 1);
        assert_eq!(safe_string::until_nul(&r.message).len(), MAX_MSG_LEN - 1);
    }

    #[test]
    fn clear_keeps_trigger_counts() {
        let mut a = AssertLog::new();
        a.init();
        let _ = a.handle(Some(b"t.c"), 1, 0, severity::CRITICAL, None, 0);
        a.clear_stats();
        let s = a.stats();
        assert_eq!((s.total_failures, s.criticals, s.log_count, s.safe_mode_triggers), (0, 0, 0, 1));
        assert!(a.is_empty());
        assert!(a.has_critical_failures());
        a.init();
        assert!(!a.has_critical_failures());
        assert_eq!(a.stats(), Stats::default());
    }

    /// Mutation check: a record carries its kind, severity and time; the
    /// overflow and reset counts survive clear_stats; the init flag
    #[test]
    fn record_fields_and_kept_counts() {
        let mut a = AssertLog::new();
        assert!(!a.is_initialized());
        assert!(a.is_empty());
        a.init();
        assert!(a.is_initialized());
        let _ = a.handle(Some(b"k.c"), 9, 6, severity::CRITICAL, Some(b"m"), 1234);
        assert!(!a.is_empty());
        let r = a.record(0).unwrap();
        assert_eq!(
            (r.line, r.kind, r.severity, r.timestamp_s, r.occurrence_count),
            (9, 6, severity::CRITICAL, 1234, 1)
        );
        for line in 0..20u16 {
            let _ = a.handle(Some(b"w.c"), line, 7, severity::FATAL, None, 0);
        }
        a.clear_stats();
        let s = a.stats();
        assert_eq!((s.log_overflows, s.reset_triggers, s.safe_mode_triggers, s.log_count), (1, 20, 1, 0));
        assert!(a.is_initialized());
    }
}
