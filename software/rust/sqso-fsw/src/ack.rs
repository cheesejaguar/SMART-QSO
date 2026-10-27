//! The command acknowledgement (RID GND-03; `docs/interfaces/ICD_SPACE_GROUND.md`
//! §5.5): after each authenticated command, one plain-ASCII UI frame
//!
//! ```text
//! ack KK6GDB-11: C=<counter> R=<result> N=<days since contact>
//! ```
//!
//! `C` is the command's counter, `R` its result (`CmdResult_t`: 0 success,
//! 1 invalid command, 2 invalid parameter, 4 not allowed, 5 execution
//! failed), `N` the whole days the loss-of-contact monitor counts after the
//! command: 0 confirms that the command counted as contact. Forged and
//! replayed frames get no acknowledgement (a forger must not be able to key
//! the transmitter). The frame goes through the downlink and its inhibits and
//! duty limit; one that cannot go within [`TTL_MS`] is dropped.
//!
//! @requirement SRS-F125 Acknowledgement frame after each authenticated command

use core::fmt::Write;

/// The information field's start
pub const PREFIX: &str = "ack ";
/// Longest information field, bytes
pub const MAX_LEN: usize = 48;
/// Acknowledgements waiting at most
pub const QUEUE: usize = 4;
/// An acknowledgement not sent within this long is dropped, ms
pub const TTL_MS: u64 = 30_000;

/// One acknowledgement
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ack {
    /// The command's counter
    pub counter: u32,
    /// Its result (`CmdResult_t`)
    pub result: u32,
    /// Whole days since contact after the command
    pub days: u32,
    /// When it was queued, ms
    pub queued_ms: u64,
}

struct Out<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl Write for Out<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(core::fmt::Error)?;
        self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// The information field in `out`; its length (0 if it does not fit)
pub fn format(a: &Ack, out: &mut [u8]) -> usize {
    let mut w = Out { buf: out, len: 0 };
    let call = crate::beacon::CALLSIGN;
    match write!(w, "{PREFIX}{call}: C={} R={} N={}", a.counter, a.result, a.days) {
        Ok(()) => w.len,
        Err(_) => 0,
    }
}

/// Acknowledgements waiting to be sent, oldest first
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Queue {
    items: [Option<Ack>; QUEUE],
    /// Dropped (queue full or too old)
    pub dropped: u32,
}

impl Queue {
    /// Empty
    pub const fn new() -> Queue {
        Queue { items: [None; QUEUE], dropped: 0 }
    }

    /// Add one; a full queue drops its oldest
    pub fn push(&mut self, a: Ack) {
        if self.items.iter().all(Option::is_some) {
            self.pop();
            self.dropped = self.dropped.saturating_add(1);
        }
        if let Some(slot) = self.items.iter_mut().find(|s| s.is_none()) {
            *slot = Some(a);
        }
    }

    /// The oldest still worth sending at `now_ms` (older ones are dropped)
    pub fn front(&mut self, now_ms: u64) -> Option<Ack> {
        while let Some(Some(a)) = self.items.first().copied() {
            if now_ms.saturating_sub(a.queued_ms) <= TTL_MS {
                return Some(a);
            }
            self.pop();
            self.dropped = self.dropped.saturating_add(1);
        }
        None
    }

    /// Remove the oldest
    pub fn pop(&mut self) {
        self.items.copy_within(1.., 0);
        if let Some(last) = self.items.last_mut() {
            *last = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    #[test]
    fn the_frame_text() {
        let mut b = [0u8; MAX_LEN];
        let n = format(&Ack { counter: 4_294_967_294, result: 5, days: 89, queued_ms: 0 }, &mut b);
        assert_eq!(core::str::from_utf8(&b[..n]).unwrap(), "ack KK6GDB-11: C=4294967294 R=5 N=89");
        assert!(n <= MAX_LEN);
        let mut small = [0u8; 10];
        assert_eq!(format(&Ack::default(), &mut small), 0);
    }

    #[test]
    fn queue_order_overflow_and_age() {
        let mut q = Queue::new();
        assert_eq!(q.front(0), None);
        for c in 1..=5 {
            q.push(Ack { counter: c, queued_ms: u64::from(c), ..Ack::default() });
        }
        assert_eq!(q.dropped, 1, "the oldest made room");
        assert_eq!(q.front(10).map(|a| a.counter), Some(2));
        q.pop();
        assert_eq!(q.front(10).map(|a| a.counter), Some(3));
        assert_eq!(q.front(4 + TTL_MS).map(|a| a.counter), Some(4), "3 too old");
        assert_eq!(q.front(u64::MAX), None);
        assert_eq!(q.dropped, 4);
        assert_eq!(Queue::default(), Queue { dropped: 0, ..Queue::new() });
    }
}
