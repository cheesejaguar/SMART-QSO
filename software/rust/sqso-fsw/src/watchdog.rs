//! Watchdog manager (`watchdog_mgr.h`): the task table behind the
//! supervised hardware watchdog. The hardware watchdog is refreshed only
//! while every registered task checks in; a task that misses its period
//! [`MISS_THRESHOLD`] checks in a row is stalled, which stops the refresh
//! so the hardware resets the OBC (SW-15).
//!
//! The HAL, the fault log and the pre-reset callback are the caller's:
//! [`Watchdog::check_task`] reports a stall and the caller logs it and saves
//! state.
//!
//! @requirement SRS-F050 Kick hardware watchdog at least every 30 seconds
//! @requirement SRS-F051 Save state before watchdog reset

/// Task slots (`WDT_MAX_TASKS`)
pub const MAX_TASKS: usize = 8;
/// Task name bytes including the NUL (`WDT_TASK_NAME_LEN`)
pub const NAME_LEN: usize = 16;
/// Hardware watchdog timeout, ms
pub const HW_TIMEOUT_MS: u32 = 30_000;
/// Task check interval, ms
pub const TASK_CHECK_INTERVAL_MS: u64 = 1000;
/// Missed periods in a row that make a task stalled
pub const MISS_THRESHOLD: u32 = 3;

/// `WdtTaskEntry_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskEntry {
    /// Name, NUL-terminated
    pub name: [u8; NAME_LEN],
    /// Expected check-in period, ms
    pub expected_period_ms: u32,
    /// Last check-in, ms
    pub last_checkin_ms: u64,
    /// Missed checks in a row
    pub miss_count: u32,
    /// Check-ins since registration
    pub total_checkins: u32,
    /// Registered
    pub active: bool,
    /// Stalled
    pub stalled: bool,
}

impl TaskEntry {
    const ZERO: TaskEntry = TaskEntry {
        name: [0; NAME_LEN],
        expected_period_ms: 0,
        last_checkin_ms: 0,
        miss_count: 0,
        total_checkins: 0,
        active: false,
        stalled: false,
    };

    /// The name up to its NUL
    pub fn name(&self) -> &[u8] {
        let len = self.name.iter().position(|b| *b == 0).unwrap_or(NAME_LEN);
        self.name.get(..len).unwrap_or(&[])
    }
}

/// `WdtStats_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Hardware watchdog refreshes
    pub hw_kicks: u32,
    /// Task check runs
    pub task_check_runs: u32,
    /// Stalls detected
    pub task_stalls_detected: u32,
    /// Resets caused by the watchdog (kept across `init`)
    pub resets_caused: u32,
    /// Mission time of the last stall, s
    pub last_reset_time_s: u32,
    /// Task that stalled last
    pub last_stalled_task: u8,
    /// Hardware watchdog running
    pub hw_wdt_running: bool,
}

impl Stats {
    const ZERO: Stats = Stats {
        hw_kicks: 0,
        task_check_runs: 0,
        task_stalls_detected: 0,
        resets_caused: 0,
        last_reset_time_s: 0,
        last_stalled_task: 0,
        hw_wdt_running: false,
    };
}

/// Why a request was refused
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WdtError {
    /// Task id or period out of range
    Param,
    /// The task is not registered
    NotRegistered,
}

/// The watchdog manager's task table and statistics
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Watchdog {
    /// Task table
    pub tasks: [TaskEntry; MAX_TASKS],
    /// Statistics
    pub stats: Stats,
    /// Last task check, ms
    pub last_task_check_ms: u64,
    /// Initialized
    pub initialized: bool,
}

impl Watchdog {
    /// Before `wdt_mgr_init()`
    pub const fn new() -> Watchdog {
        Watchdog { tasks: [TaskEntry::ZERO; MAX_TASKS], stats: Stats::ZERO, last_task_check_ms: 0, initialized: false }
    }

    /// Clear the tasks and statistics, keeping the reset count (the caller
    /// then initializes the hardware and sets `initialized`)
    pub fn reset(&mut self) {
        let resets = self.stats.resets_caused;
        self.tasks = [TaskEntry::ZERO; MAX_TASKS];
        self.stats = Stats { resets_caused: resets, ..Stats::ZERO };
        self.last_task_check_ms = 0;
    }

    /// Register a task (the name is cut to 15 bytes; bytes after its NUL
    /// keep what an earlier name left there)
    pub fn register(&mut self, task_id: u32, name: &[u8], period_ms: u32, now_ms: u64) -> Result<(), WdtError> {
        let entry = self.entry_mut(task_id)?;
        if period_ms == 0 {
            return Err(WdtError::Param);
        }
        let len = name.iter().take(NAME_LEN - 1).take_while(|b| **b != 0).count();
        // The name and its NUL (len < NAME_LEN, so the NUL fits)
        for (dst, src) in entry.name.iter_mut().zip(name.iter().take(len).chain(core::iter::once(&0))) {
            *dst = *src;
        }
        entry.expected_period_ms = period_ms;
        entry.last_checkin_ms = now_ms;
        entry.miss_count = 0;
        entry.total_checkins = 0;
        entry.active = true;
        entry.stalled = false;
        Ok(())
    }

    /// Whether a task id is in range
    pub fn valid_id(task_id: u32) -> bool {
        usize::try_from(task_id).is_ok_and(|i| i < MAX_TASKS)
    }

    fn entry_mut(&mut self, task_id: u32) -> Result<&mut TaskEntry, WdtError> {
        usize::try_from(task_id).ok().and_then(|i| self.tasks.get_mut(i)).ok_or(WdtError::Param)
    }

    /// A task's entry
    pub fn entry(&self, task_id: u32) -> Option<&TaskEntry> {
        usize::try_from(task_id).ok().and_then(|i| self.tasks.get(i))
    }

    /// Stop monitoring a task
    pub fn unregister(&mut self, task_id: u32) -> Result<(), WdtError> {
        self.entry_mut(task_id)?.active = false;
        Ok(())
    }

    /// A task's heartbeat
    pub fn checkin(&mut self, task_id: u32, now_ms: u64) -> Result<(), WdtError> {
        let entry = self.entry_mut(task_id)?;
        if !entry.active {
            return Err(WdtError::NotRegistered);
        }
        entry.last_checkin_ms = now_ms;
        entry.miss_count = 0;
        entry.stalled = false;
        entry.total_checkins = entry.total_checkins.wrapping_add(1);
        Ok(())
    }

    /// Whether a task is stalled (false for an id out of range)
    pub fn is_stalled(&self, task_id: u32) -> bool {
        self.entry(task_id).is_some_and(|e| e.stalled)
    }

    /// Whether any registered task is stalled
    pub fn any_stalled(&self) -> bool {
        self.tasks.iter().any(|t| t.active && t.stalled)
    }

    /// Whether the task check is due (once per [`TASK_CHECK_INTERVAL_MS`])
    pub fn check_due(&self, now_ms: u64) -> bool {
        now_ms.wrapping_sub(self.last_task_check_ms) >= TASK_CHECK_INTERVAL_MS
    }

    /// Check one task slot; `true` if the task has just become stalled (the
    /// caller logs it and saves state)
    pub fn check_task(&mut self, index: usize, now_ms: u64) -> bool {
        let Some(task) = self.tasks.get_mut(index) else { return false };
        if !task.active {
            return false;
        }
        if now_ms.wrapping_sub(task.last_checkin_ms) <= u64::from(task.expected_period_ms) {
            // Healthy
            task.miss_count = 0;
            task.stalled = false;
            return false;
        }
        task.miss_count = task.miss_count.wrapping_add(1);
        if task.miss_count < MISS_THRESHOLD || task.stalled {
            return false;
        }
        task.stalled = true;
        self.stats.task_stalls_detected = self.stats.task_stalls_detected.wrapping_add(1);
        self.stats.last_stalled_task = u8::try_from(index).unwrap_or(u8::MAX);
        #[allow(clippy::cast_possible_truncation)] // as the C (uint32_t) conversion
        {
            self.stats.last_reset_time_s = (now_ms / 1000) as u32;
        }
        true
    }

    /// Count a completed task check
    pub fn finish_check(&mut self, now_ms: u64) {
        self.stats.task_check_runs = self.stats.task_check_runs.wrapping_add(1);
        self.last_task_check_ms = now_ms;
    }

    /// Count a hardware refresh
    pub fn count_kick(&mut self) {
        self.stats.hw_kicks = self.stats.hw_kicks.wrapping_add(1);
    }
}

impl Default for Watchdog {
    fn default() -> Self {
        Watchdog::new()
    }
}

/// The fault description of a stall: "Task NAME stalled" (NUL-terminated,
/// at most `N - 1` bytes; "Task stalled" if it would not fit)
pub fn stall_message<const N: usize>(name: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    let parts: [&[u8]; 3] = [b"Task ", name, b" stalled"];
    let len: usize = parts.iter().map(|p| p.len()).sum();
    let parts: [&[u8]; 3] = if len < N { parts } else { [b"Task stalled", b"", b""] };
    // At most N - 1 bytes: the rest stay NUL
    for (slot, b) in out.iter_mut().take(N.saturating_sub(1)).zip(parts.iter().flat_map(|p| p.iter())) {
        *slot = *b;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration() {
        let mut w = Watchdog::new();
        assert_eq!(w.register(8, b"x", 1000, 0), Err(WdtError::Param));
        assert_eq!(w.register(0, b"x", 0, 0), Err(WdtError::Param));
        assert_eq!(w.checkin(5, 0), Err(WdtError::NotRegistered));
        assert_eq!(w.checkin(100, 0), Err(WdtError::Param));
        assert!(w.register(2, b"ADCS_Task", 2000, 7).is_ok());
        assert_eq!(w.tasks[2].name(), b"ADCS_Task");
        assert!(w.register(2, b"ADCS", 2000, 7).is_ok());
        assert_eq!(&w.tasks[2].name[..10], b"ADCS\0Task\0");
        assert!(w.register(3, b"a-very-long-task-name", 1, 0).is_ok());
        assert_eq!(w.tasks[3].name(), b"a-very-long-tas");
        assert!(w.unregister(3).is_ok());
        assert_eq!(w.unregister(8), Err(WdtError::Param));
        assert!(!w.tasks[3].active);
    }

    #[test]
    fn stall_after_three_misses_reported_once() {
        let mut w = Watchdog::new();
        assert!(w.register(0, b"main", 1000, 0).is_ok());
        let mut stalls = 0;
        for s in 1..=40u64 {
            if w.check_task(0, s * 1000) {
                stalls += 1;
            }
        }
        // 1 s is within the period; misses from 2 s, stalled at 4 s
        assert_eq!(stalls, 1);
        assert!(w.any_stalled());
        assert_eq!(w.stats.task_stalls_detected, 1);
        assert_eq!(w.stats.last_reset_time_s, 4);
        assert!(w.checkin(0, 41_000).is_ok());
        assert!(!w.is_stalled(0));
        assert!(!w.check_task(0, 41_500));
        assert!(!w.any_stalled());
    }

    #[test]
    fn reset_keeps_the_reset_count() {
        let mut w = Watchdog::new();
        w.stats.resets_caused = 2;
        w.stats.hw_kicks = 9;
        assert!(w.register(1, b"eps", 10, 0).is_ok());
        w.reset();
        assert_eq!(w.stats, Stats { resets_caused: 2, ..Stats::default() });
        assert!(!w.tasks[1].active);
    }

    #[test]
    fn messages() {
        assert_eq!(&stall_message::<64>(b"main")[..18], b"Task main stalled\0");
        assert_eq!(&stall_message::<16>(b"main-loop")[..13], b"Task stalled\0");
    }

    #[test]
    fn ids_entries_and_stall_state() {
        assert!(Watchdog::valid_id(0) && Watchdog::valid_id(7));
        assert!(!Watchdog::valid_id(8) && !Watchdog::valid_id(u32::MAX));
        let mut w = Watchdog::new();
        assert!(w.register(2, b"comm", 1000, 0).is_ok());
        assert_eq!(w.entry(2).map(TaskEntry::name), Some(&b"comm"[..]));
        assert_eq!(w.entry(2).map(|e| e.expected_period_ms), Some(1000));
        assert!(w.entry(8).is_none());
        for s in 1..=4u64 {
            let _ = w.check_task(2, s * 1000);
        }
        assert!(w.is_stalled(2));
        assert!(!w.is_stalled(3) && !w.is_stalled(99));
    }

    /// Mutation check: the task check runs once per interval and is
    /// counted; hardware refreshes are counted
    #[test]
    fn check_interval_and_counters() {
        let mut w = Watchdog::new();
        assert!(!w.check_due(TASK_CHECK_INTERVAL_MS - 1));
        assert!(w.check_due(TASK_CHECK_INTERVAL_MS));
        w.finish_check(5000);
        assert_eq!((w.stats.task_check_runs, w.last_task_check_ms), (1, 5000));
        assert!(!w.check_due(5000 + TASK_CHECK_INTERVAL_MS - 1));
        assert!(w.check_due(5000 + TASK_CHECK_INTERVAL_MS));
        w.count_kick();
        w.count_kick();
        assert_eq!(w.stats.hw_kicks, 2);
    }

    /// Mutation check: a message that would exactly fill the buffer (no
    /// room for the NUL) falls back, and the fallback is cut to N - 1
    /// bytes plus a NUL
    #[test]
    fn stall_message_bounds() {
        // "Task abc stalled" is 16 bytes: it needs 17
        assert_eq!(&stall_message::<16>(b"abc")[..13], b"Task stalled\0");
        assert_eq!(&stall_message::<17>(b"abc"), b"Task abc stalled\0");
        assert_eq!(&stall_message::<8>(b"abc"), b"Task st\0");
        assert_eq!(&stall_message::<1>(b"abc"), b"\0");
        assert_eq!(&stall_message::<0>(b"abc"), b"");
    }

    /// A slot index past the table is never stalled and changes nothing
    #[test]
    fn check_task_out_of_range_is_not_a_stall() {
        let mut w = Watchdog::new();
        assert!(w.register(0, b"main", 1, 0).is_ok());
        let before = w;
        for now in [10_000, 20_000, 30_000, 40_000] {
            assert!(!w.check_task(MAX_TASKS, now));
            assert!(!w.check_task(usize::MAX, now));
        }
        assert_eq!(w, before);
    }

    /// The default watchdog is the one before `wdt_mgr_init()`
    #[test]
    fn default_is_uninitialized_and_empty() {
        let w = Watchdog::default();
        assert_eq!(w, Watchdog::new());
        assert!(!w.initialized && !w.any_stalled());
        assert!(w.tasks.iter().all(|t| !t.active));
        assert_eq!(w.stats, Stats::default());
    }

    /// A name of exactly 15 bytes fills the field and ends in its NUL
    #[test]
    fn longest_name_keeps_its_nul() {
        let mut w = Watchdog::new();
        w.tasks[1].name = [b'#'; NAME_LEN];
        assert!(w.register(1, b"fifteen-bytes!!", 10, 0).is_ok());
        assert_eq!(&w.tasks[1].name, b"fifteen-bytes!!\0");
        assert!(w.register(1, b"", 10, 0).is_ok());
        assert_eq!(&w.tasks[1].name[..2], b"\0i");
    }
}
