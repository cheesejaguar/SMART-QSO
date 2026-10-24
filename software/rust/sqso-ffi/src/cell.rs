//! Module state shared with C callers.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

/// A module's state. [`Global::with`] hands out the one `&mut` at a time;
/// a nested call (a C callback that re-enters the module, or an interrupt)
/// gets `None` instead of a second, aliasing reference.
pub(crate) struct Global<T> {
    busy: AtomicBool,
    value: UnsafeCell<T>,
}

// SAFETY: access is serialized by `busy`; T is only reached through `with`
unsafe impl<T: Send> Sync for Global<T> {}

impl<T> Global<T> {
    pub(crate) const fn new(value: T) -> Self {
        Global { busy: AtomicBool::new(false), value: UnsafeCell::new(value) }
    }

    /// Run `f` on the state, or return `None` if it is already in use
    pub(crate) fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
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
