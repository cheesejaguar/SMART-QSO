//! Time-triggered scheduler (`scheduler.h`): a table of periodic tasks run
//! highest priority first when due, with deadline monitoring.
//!
//! The task function type is a parameter, so this module never calls a
//! task: [`Scheduler::begin_run`] hands the function to the caller, which
//! runs it with no borrow of the scheduler held (a task may call back into
//! the scheduler), then reports with [`Scheduler::finish_run`].

/// Task slots
pub const MAX_TASKS: usize = 16;
/// Task name bytes (NUL-terminated if shorter)
pub const NAME_LEN: usize = 16;
/// Shortest period, ms
pub const MIN_PERIOD_MS: u32 = 10;
/// Longest period, ms
pub const MAX_PERIOD_MS: u32 = 60_000;
/// Consecutive deadline misses that put a task in fault
pub const DEADLINE_MISS_LIMIT: u32 = 3;
/// CPU utilization window, ticks
pub const CPU_UTIL_WINDOW: u32 = 1000;
const AVG_FACTOR: u32 = 8;

/// Task state (values are the C `task_state_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum TaskState {
    /// Not scheduled
    Inactive = 0,
    /// Ready to run
    Ready = 1,
    /// Executing
    Running = 2,
    /// Temporarily suspended
    Suspended = 3,
    /// Missed too many deadlines
    Fault = 4,
}

/// Why a request failed (values are the C `sched_status_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SchedError {
    /// Bad parameter
    InvalidParam = 1,
    /// No free slot
    TableFull = 2,
    /// No task with that handle
    NotFound = 3,
    /// A task of that name exists
    AlreadyExists = 4,
    /// The scheduler is not initialized
    NotRunning = 5,
}

/// A task's configuration (the C `task_config_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskConfig<F> {
    /// Name (NUL-terminated if shorter than [`NAME_LEN`])
    pub name: [u8; NAME_LEN],
    /// Task function
    pub func: F,
    /// Period, ms
    pub period_ms: u32,
    /// Phase offset from registration, ms
    pub offset_ms: u32,
    /// Longest allowed run time, ms (0: none)
    pub deadline_ms: u32,
    /// Priority, lower is more urgent
    pub priority: u32,
    /// Ready at registration
    pub enabled: bool,
}

/// Run-time statistics (the C `task_stats_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskStats {
    /// Executions
    pub run_count: u32,
    /// Last run time, us
    pub last_run_time_us: u32,
    /// Longest run time, us
    pub max_run_time_us: u32,
    /// Shortest run time, us
    pub min_run_time_us: u32,
    /// Moving average run time, us
    pub avg_run_time_us: u32,
    /// Deadline misses
    pub deadline_misses: u32,
    /// Skipped executions
    pub skip_count: u32,
}

impl TaskStats {
    /// No runs yet
    pub const NEW: TaskStats = TaskStats {
        run_count: 0,
        last_run_time_us: 0,
        max_run_time_us: 0,
        min_run_time_us: u32::MAX,
        avg_run_time_us: 0,
        deadline_misses: 0,
        skip_count: 0,
    };
}

/// Scheduler-wide statistics (the C `sched_stats_t`)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SchedStats {
    /// Ticks since start
    pub tick_count: u32,
    /// CPU use over the last window, %
    pub cpu_utilization: u32,
    /// Idle time in the current window, us
    pub idle_time_us: u32,
    /// Tasks ready to run
    pub active_tasks: u32,
    /// Deadline misses over all tasks
    pub total_deadline_misses: u32,
}

#[derive(Clone, Copy, Debug)]
struct Task<F> {
    config: TaskConfig<F>,
    state: TaskState,
    stats: TaskStats,
    next_run_tick: u32,
    consecutive_misses: u32,
}

/// The task table
#[derive(Clone, Copy, Debug)]
pub struct Scheduler<F> {
    tasks: [Option<Task<F>>; MAX_TASKS],
    tick_count: u32,
    active_time_us: u32,
    idle_time_us: u32,
    cpu_utilization: u32,
    running_task: Option<u8>,
    running: bool,
    initialized: bool,
}

/// Compare a stored name with a C string, as `strncmp(.., NAME_LEN) == 0`
fn same_name(stored: &[u8; NAME_LEN], other: &[u8]) -> bool {
    for (i, a) in stored.iter().enumerate() {
        let b = other.get(i).copied().unwrap_or(0);
        if *a != b {
            return false;
        }
        if *a == 0 {
            return true;
        }
    }
    true
}

impl<F: Copy> Scheduler<F> {
    /// An uninitialized scheduler
    pub const fn new() -> Self {
        Scheduler {
            tasks: [None; MAX_TASKS],
            tick_count: 0,
            active_time_us: 0,
            idle_time_us: 0,
            cpu_utilization: 0,
            running_task: None,
            running: false,
            initialized: false,
        }
    }

    /// Clear the table and mark it initialized
    pub fn init(&mut self) {
        *self = Scheduler::new();
        self.initialized = true;
    }

    fn task(&self, handle: u8) -> Result<&Task<F>, SchedError> {
        match self.tasks.get(usize::from(handle)) {
            None => Err(SchedError::InvalidParam),
            Some(None) => Err(SchedError::NotFound),
            Some(Some(t)) => Ok(t),
        }
    }

    fn task_mut(&mut self, handle: u8) -> Result<&mut Task<F>, SchedError> {
        match self.tasks.get_mut(usize::from(handle)) {
            None => Err(SchedError::InvalidParam),
            Some(None) => Err(SchedError::NotFound),
            Some(Some(t)) => Ok(t),
        }
    }

    /// Add a task; returns its handle
    pub fn register(&mut self, config: TaskConfig<F>) -> Result<u8, SchedError> {
        if !(MIN_PERIOD_MS..=MAX_PERIOD_MS).contains(&config.period_ms) {
            return Err(SchedError::InvalidParam);
        }
        let slot = self.tasks.iter().position(Option::is_none).ok_or(SchedError::TableFull)?;
        if self.tasks.iter().flatten().any(|t| same_name(&t.config.name, &config.name)) {
            return Err(SchedError::AlreadyExists);
        }
        let task = Task {
            config,
            state: if config.enabled { TaskState::Ready } else { TaskState::Inactive },
            stats: TaskStats::NEW,
            next_run_tick: self.tick_count.wrapping_add(config.offset_ms),
            consecutive_misses: 0,
        };
        *self.tasks.get_mut(slot).ok_or(SchedError::TableFull)? = Some(task);
        u8::try_from(slot).map_err(|_| SchedError::TableFull)
    }

    /// Remove a task (not the one running)
    pub fn unregister(&mut self, handle: u8) -> Result<(), SchedError> {
        self.task(handle)?;
        if self.running_task == Some(handle) {
            return Err(SchedError::InvalidParam);
        }
        if let Some(slot) = self.tasks.get_mut(usize::from(handle)) {
            *slot = None;
        }
        Ok(())
    }

    /// Make a task ready now
    pub fn enable(&mut self, handle: u8) -> Result<(), SchedError> {
        let tick = self.tick_count;
        let t = self.task_mut(handle)?;
        t.state = TaskState::Ready;
        t.config.enabled = true;
        t.next_run_tick = tick;
        Ok(())
    }

    /// Stop scheduling a task
    pub fn disable(&mut self, handle: u8) -> Result<(), SchedError> {
        let t = self.task_mut(handle)?;
        t.state = TaskState::Inactive;
        t.config.enabled = false;
        Ok(())
    }

    /// Suspend a task
    pub fn suspend(&mut self, handle: u8) -> Result<(), SchedError> {
        self.task_mut(handle)?.state = TaskState::Suspended;
        Ok(())
    }

    /// Resume a suspended task
    pub fn resume(&mut self, handle: u8) -> Result<(), SchedError> {
        let t = self.task_mut(handle)?;
        if t.state == TaskState::Suspended {
            t.state = TaskState::Ready;
        }
        Ok(())
    }

    /// A task's state (`Inactive` for an unknown handle)
    pub fn state(&self, handle: u8) -> TaskState {
        self.task(handle).map_or(TaskState::Inactive, |t| t.state)
    }

    /// A task's statistics
    pub fn stats(&self, handle: u8) -> Result<TaskStats, SchedError> {
        self.task(handle).map(|t| t.stats)
    }

    /// Reset a task's statistics
    pub fn reset_stats(&mut self, handle: u8) -> Result<(), SchedError> {
        self.task_mut(handle)?.stats = TaskStats::NEW;
        Ok(())
    }

    /// Change a task's period
    pub fn set_period(&mut self, handle: u8, period_ms: u32) -> Result<(), SchedError> {
        if usize::from(handle) >= MAX_TASKS || !(MIN_PERIOD_MS..=MAX_PERIOD_MS).contains(&period_ms) {
            return Err(SchedError::InvalidParam);
        }
        self.task_mut(handle)?.config.period_ms = period_ms;
        Ok(())
    }

    /// A task's period (0 for an unknown handle)
    pub fn period(&self, handle: u8) -> u32 {
        self.task(handle).map_or(0, |t| t.config.period_ms)
    }

    /// A task's name
    pub fn name(&self, handle: u8) -> Option<&[u8; NAME_LEN]> {
        self.task(handle).ok().map(|t| &t.config.name)
    }

    /// The handle of the task with this name
    pub fn handle_by_name(&self, name: &[u8]) -> Option<u8> {
        let i = self.tasks.iter().position(|t| t.as_ref().is_some_and(|t| same_name(&t.config.name, name)))?;
        u8::try_from(i).ok()
    }

    /// Scheduler-wide statistics
    pub fn sched_stats(&self) -> SchedStats {
        let tasks = self.tasks.iter().flatten();
        SchedStats {
            tick_count: self.tick_count,
            cpu_utilization: self.cpu_utilization,
            idle_time_us: self.idle_time_us,
            active_tasks: u32::try_from(tasks.clone().filter(|t| t.state == TaskState::Ready).count()).unwrap_or(u32::MAX),
            total_deadline_misses: tasks.fold(0u32, |sum, t| sum.wrapping_add(t.stats.deadline_misses)),
        }
    }

    /// CPU use over the last window, %
    pub fn cpu_utilization(&self) -> u32 {
        self.cpu_utilization
    }

    /// Ticks since start
    pub fn tick_count(&self) -> u32 {
        self.tick_count
    }

    /// Initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Between start and stop
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Start: reset the tick count (the caller then ticks until stopped)
    pub fn start(&mut self) -> Result<(), SchedError> {
        if !self.initialized {
            return Err(SchedError::NotRunning);
        }
        self.running = true;
        self.tick_count = 0;
        Ok(())
    }

    /// Ask the tick loop to stop
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Advance one tick; the most urgent due task, if any
    pub fn tick(&mut self) -> Option<u8> {
        self.tick_count = self.tick_count.wrapping_add(1);
        let tick = self.tick_count;
        let mut best: Option<(usize, u32)> = None;
        for (i, t) in self.tasks.iter().enumerate() {
            let Some(t) = t else { continue };
            if t.state != TaskState::Ready || tick < t.next_run_tick {
                continue;
            }
            if best.is_none_or(|(_, p)| t.config.priority < p) {
                best = Some((i, t.config.priority));
            }
        }
        best.and_then(|(i, _)| u8::try_from(i).ok())
    }

    /// Account idle time
    pub fn idle(&mut self, idle_us: u32) {
        self.idle_time_us = self.idle_time_us.wrapping_add(idle_us);
    }

    /// Mark a task running; its function, for the caller to run
    pub fn begin_run(&mut self, handle: u8) -> Result<F, SchedError> {
        let t = self.task_mut(handle)?;
        t.state = TaskState::Running;
        let func = t.config.func;
        self.running_task = Some(handle);
        Ok(func)
    }

    /// Record a finished run. Returns the deadline overrun in us, if the
    /// deadline was missed. A task that changed its own state while running
    /// (disabled or suspended itself) keeps that state.
    pub fn finish_run(&mut self, handle: u8, run_time_us: u32) -> Option<u32> {
        let tick = self.tick_count;
        let mut overrun = None;
        if let Ok(t) = self.task_mut(handle) {
            let s = &mut t.stats;
            s.run_count = s.run_count.wrapping_add(1);
            s.last_run_time_us = run_time_us;
            s.max_run_time_us = s.max_run_time_us.max(run_time_us);
            s.min_run_time_us = s.min_run_time_us.min(run_time_us);
            let avg = (u64::from(s.avg_run_time_us) * u64::from(AVG_FACTOR - 1) + u64::from(run_time_us)) / u64::from(AVG_FACTOR);
            s.avg_run_time_us = u32::try_from(avg).unwrap_or(u32::MAX);

            let deadline_us = t.config.deadline_ms.saturating_mul(1000);
            if deadline_us > 0 && run_time_us > deadline_us {
                s.deadline_misses = s.deadline_misses.wrapping_add(1);
                t.consecutive_misses = t.consecutive_misses.saturating_add(1);
                overrun = Some(run_time_us - deadline_us);
                if t.consecutive_misses >= DEADLINE_MISS_LIMIT {
                    t.state = TaskState::Fault;
                }
            } else {
                t.consecutive_misses = 0;
            }

            t.next_run_tick = tick.wrapping_add(t.config.period_ms);
            if t.state == TaskState::Running {
                t.state = TaskState::Ready;
            }
        }
        self.running_task = None;

        self.active_time_us = self.active_time_us.wrapping_add(run_time_us);
        if tick % CPU_UTIL_WINDOW == 0 {
            let total = u64::from(self.active_time_us) + u64::from(self.idle_time_us);
            if let Some(percent) = (u64::from(self.active_time_us) * 100).checked_div(total) {
                self.cpu_utilization = u32::try_from(percent).unwrap_or(100);
            }
            self.active_time_us = 0;
            self.idle_time_us = 0;
        }
        overrun
    }
}

impl<F: Copy> Default for Scheduler<F> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(name: &[u8], period_ms: u32, priority: u32) -> TaskConfig<u8> {
        let mut n = [0u8; NAME_LEN];
        n[..name.len()].copy_from_slice(name);
        TaskConfig { name: n, func: 0, period_ms, offset_ms: 0, deadline_ms: 1, priority, enabled: true }
    }

    fn run(s: &mut Scheduler<u8>, h: u8, us: u32) -> Option<u32> {
        s.begin_run(h).unwrap();
        s.finish_run(h, us)
    }

    #[test]
    fn priority_and_period() {
        let mut s = Scheduler::<u8>::new();
        s.init();
        let low = s.register(cfg(b"low", 10, 3)).unwrap();
        let high = s.register(cfg(b"high", 20, 0)).unwrap();
        assert_eq!(s.tick(), Some(high));
        run(&mut s, high, 0);
        assert_eq!(s.tick(), Some(low));
        run(&mut s, low, 0); // ran at tick 2, next due at tick 12
        for _ in 3..12 {
            assert_eq!(s.tick(), None);
        }
        assert_eq!(s.tick(), Some(low));
    }

    #[test]
    fn registration_rules() {
        let mut s = Scheduler::<u8>::new();
        s.init();
        assert_eq!(s.register(cfg(b"x", 9, 0)), Err(SchedError::InvalidParam));
        assert_eq!(s.register(cfg(b"x", 60_001, 0)), Err(SchedError::InvalidParam));
        s.register(cfg(b"x", 10, 0)).unwrap();
        assert_eq!(s.register(cfg(b"x", 10, 0)), Err(SchedError::AlreadyExists));
        for i in 1..MAX_TASKS {
            s.register(cfg(&[b'a' + u8::try_from(i).unwrap()], 10, 0)).unwrap();
        }
        assert_eq!(s.register(cfg(b"z", 10, 0)), Err(SchedError::TableFull));
        assert_eq!(s.handle_by_name(b"x\0"), Some(0));
        assert_eq!(s.unregister(16), Err(SchedError::InvalidParam));
    }

    #[test]
    fn deadline_misses_fault_the_task() {
        let mut s = Scheduler::<u8>::new();
        s.init();
        let h = s.register(cfg(b"t", 10, 0)).unwrap();
        assert_eq!(run(&mut s, h, 1500), Some(500));
        assert_eq!(run(&mut s, h, 1500), Some(500));
        assert_eq!(s.state(h), TaskState::Ready);
        assert_eq!(run(&mut s, h, 1500), Some(500));
        assert_eq!(s.state(h), TaskState::Fault);
        assert_eq!(s.sched_stats().total_deadline_misses, 3);
    }

    #[test]
    fn self_suspension_survives_the_run() {
        let mut s = Scheduler::<u8>::new();
        s.init();
        let h = s.register(cfg(b"t", 10, 0)).unwrap();
        s.begin_run(h).unwrap();
        s.suspend(h).unwrap();
        s.finish_run(h, 0);
        assert_eq!(s.state(h), TaskState::Suspended);
        assert_eq!(s.unregister(h), Ok(()));
    }
}
