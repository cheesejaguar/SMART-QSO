//! The OBC selector's decision and abstention counters (RID SMA-15;
//! `docs/payload/OBC_SELECTOR_DESIGN.md`, SQSO-PAY-SEL-001 §6.4).
//!
//! The selector itself (its three heads: H1 beacon template, H2
//! transponder bucket, H3 power-mode preference) is a fabrication-phase
//! task (SQSO-PAY-SEL-001 §11). Its telemetry is defined now, so the
//! mission success criteria's "selector decisions, abstentions" (SQSO-MSC-001
//! §7) have their housekeeping field from the first flight build: HK2 `SL`
//! carries whether a selector table is in use and, per head, the decisions
//! it made and the times it abstained (the fixed rules then decided) since
//! boot. Until the selector exists the table is absent and every count is 0.
//!
//! @requirement SRS-F170 The selector shall count, per head (H1 template, H2 transponder bucket, H3 power mode), its decisions and its abstentions since boot, and report them with its table state in HK2 field SL

/// The selector's heads (SQSO-PAY-SEL-001 §3)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Head {
    /// H1: the beacon template
    Template = 0,
    /// H2: the transponder bucket
    Transponder = 1,
    /// H3: the power-mode preference
    PowerMode = 2,
}

/// Heads
pub const HEADS: usize = 3;

/// The counters HK2 `SL` reports
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// A selector table passed its checks at boot and is in use
    pub enabled: bool,
    /// Decisions made, per head
    pub decisions: [u32; HEADS],
    /// Abstentions (the fixed rules decided), per head
    pub abstentions: [u32; HEADS],
}

impl Stats {
    /// No table, nothing counted
    pub const fn new() -> Stats {
        Stats { enabled: false, decisions: [0; HEADS], abstentions: [0; HEADS] }
    }

    /// One evaluation of `head`: a decision, or an abstention
    pub fn record(&mut self, head: Head, decided: bool) {
        let counts = if decided { &mut self.decisions } else { &mut self.abstentions };
        if let Some(c) = counts.get_mut(head as usize) {
            *c = c.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_per_head() {
        let mut s = Stats::new();
        assert_eq!(s, Stats::default());
        s.record(Head::Template, true);
        s.record(Head::Template, false);
        s.record(Head::Transponder, false);
        s.record(Head::PowerMode, true);
        s.record(Head::PowerMode, true);
        assert_eq!((s.decisions, s.abstentions), ([1, 0, 2], [1, 1, 0]));
        s.decisions[0] = u32::MAX;
        s.record(Head::Template, true);
        assert_eq!(s.decisions[0], 0, "wraps");
    }
}
