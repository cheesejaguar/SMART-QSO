//! Driver state shared by the HAL functions.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// One driver's state. The HAL is called from the main task only, never
/// from an interrupt, so a busy flag is enough: a nested call (which would
/// be a bug) gets `None` instead of a second, aliasing `&mut`. Unlike a
/// critical section, holding it does not mask the interrupts the UART and
/// time drivers run on.
pub(crate) struct Shared<T> {
    busy: AtomicBool,
    value: UnsafeCell<Option<T>>,
}

// SAFETY: access is serialized by `busy`
unsafe impl<T: Send> Sync for Shared<T> {}

impl<T> Shared<T> {
    pub(crate) const fn new() -> Self {
        Shared { busy: AtomicBool::new(false), value: UnsafeCell::new(None) }
    }

    /// Install the driver (once, at boot, before the flight software runs)
    pub(crate) fn install(&self, value: T) {
        let _ = self.with_slot(|slot| *slot = Some(value));
    }

    /// Run `f` on the driver; `None` if it is not installed or in use
    pub(crate) fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        self.with_slot(|slot| slot.as_mut().map(f)).flatten()
    }

    fn with_slot<R>(&self, f: impl FnOnce(&mut Option<T>) -> R) -> Option<R> {
        if self.busy.swap(true, Ordering::Acquire) {
            return None;
        }
        // SAFETY: `busy` was false and is now true, so no other reference
        // to the value exists until it is cleared below
        let result = f(unsafe { &mut *self.value.get() });
        self.busy.store(false, Ordering::Release);
        Some(result)
    }
}
