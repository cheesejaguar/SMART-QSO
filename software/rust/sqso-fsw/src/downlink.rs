//! The downlink transmitter (`downlink.h`): the last gate before an AX.25
//! frame goes to the 145.8 MHz transmitter's TNC.
//!
//! The OBC hands each frame to a KISS TNC on the radio UART
//! (`hal_radio_transmit()`); the TNC adds the flags and FCS, modulates
//! (AFSK 1200, Bell 202) and keys the transmitter. This module decides
//! whether a frame may go now and at what power, and watches the
//! transmitter while it is keyed:
//!
//! - **Permission**, read at the moment of hand-over: TX-OFF, the RF
//!   silence and the loss-of-contact inhibit (`rf_control_tx_permitted()`).
//!   A frame refused is not queued.
//! - **Key-down time**: the longest frame's worst-case air time (keying
//!   delay, every bit stuffed, tail) is within [`MAX_KEYDOWN_MS`] (SRS-P002;
//!   a compile-time check), and the keyed time is held to [`DUTY_PERCENT`] of the time by a credit
//!   that refills at that rate up to [`DUTY_WINDOW_MS`]'s worth. One frame at
//!   a time: a frame while the last may still be on the air is refused.
//! - **Power**: the commanded level (0.5 W or 1 W, BASELINE §7), capped by
//!   the VSWR protection below.
//! - **Monitors**, sampled while keyed: transmitter temperature, forward and
//!   reflected power. A reflected-to-forward ratio above [`VSWR_LIMIT_RATIO`]
//!   (VSWR 3:1) on [`VSWR_STRIKES`] samples in a row with output present caps
//!   the power at 0.5 W; at 0.5 W it stops the downlink for [`BACKOFF_MS`]. A
//!   keyed window with samples and no forward power is reported. Above
//!   [`TEMP_LIMIT_C`] nothing is sent until the temperature is below
//!   [`TEMP_RESUME_C`]. The cap lasts until the next reset.
//!
//! Times are the OBC's: the window a frame is on the air is estimated from
//! its length, the UART rate and the TNC's parameters (TBR with the TNC).
//! The thresholds are TBR with the transmitter (SQSO-RF-TX-001 §8 to §10).
//!
//! @requirement SRS-I023 Downlink frames to the TNC in KISS framing, with the TNC parameters
//! @requirement SRS-I024 No downlink frame unless transmission is permitted when it is handed over
//! @requirement SRS-I025 Key-down time per frame and duty limited
//! @requirement SRS-I026 Downlink power at the commanded level, never above 1 W
//! @requirement SRS-I027 Transmitter temperature and VSWR protection
//! @requirement SRS-P002 Beacon transmission shall complete within 5 seconds

use sqso_protocol::ax25;
use sqso_protocol::kiss;

use crate::rf_control::{POWER_0W5, POWER_1W};

/// Air bit rate (AFSK 1200)
pub const BIT_RATE: u32 = 1200;
/// OBC–TNC UART rate, 10 bits per byte
pub const UART_BAUD: u32 = 9600;
/// Largest frame handed over: addresses through information, no FCS
pub const MAX_FRAME_LEN: usize = ax25::MAX_FRAME_LEN - 2;
/// Longest a frame may keep the transmitter on (SRS-P002)
pub const MAX_KEYDOWN_MS: u32 = 5000;
/// Keyed time as a share of all time, percent (TBR; the transponder's duty
/// bound, BASELINE §5)
pub const DUTY_PERCENT: u64 = 25;
/// The keyed-time credit holds this window's worth at most
pub const DUTY_WINDOW_MS: u64 = 60_000;
/// Reflected over forward power above which the antenna match is bad:
/// VSWR 3:1 is |Γ| = 0.5, a power ratio of 0.25 (SQSO-RF-TX-001 §8.2)
pub const VSWR_LIMIT_RATIO: f64 = 0.25;
/// Forward power that counts as output present, W
pub const RF_PRESENT_W: f64 = 0.1;
/// Bad-match samples in a row before acting
pub const VSWR_STRIKES: u8 = 3;
/// Downlink stopped after a bad match at the lowest power, ms
pub const BACKOFF_MS: u64 = 600_000;
/// Transmitter temperature above which nothing is sent, °C (TBR; below the
/// PA's 85 °C junction shutdown, SQSO-RF-TX-001 §6.3)
pub const TEMP_LIMIT_C: f64 = 70.0;
/// Temperature below which sending resumes, °C
pub const TEMP_RESUME_C: f64 = 60.0;

/// Credit is kept in hundredths of a millisecond of keyed time, so that
/// refilling at a percentage loses nothing to rounding
const CREDIT_SCALE: u64 = 100;
const CREDIT_MAX: u64 = DUTY_WINDOW_MS * DUTY_PERCENT;

/// Why a frame was not sent
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Not initialized, or transmission not permitted (TX-OFF, RF silence,
    /// loss of contact)
    Inhibited,
    /// Empty, or longer than [`MAX_FRAME_LEN`]
    Invalid,
    /// The transmitter is above [`TEMP_LIMIT_C`] (until below [`TEMP_RESUME_C`])
    OverTemperature,
    /// Stopped after a bad antenna match at the lowest power
    BackOff,
    /// The last frame may still be on the air
    Busy,
    /// Not enough keyed-time credit
    Duty,
}

/// What [`Downlink::request`] decided for a frame
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plan {
    /// Power level to select (`RfPowerLevel_t`)
    pub level: u32,
    /// Worst-case time on the air, ms
    pub airtime_ms: u32,
}

/// The transmitter monitors (`hal_adc.h` TX channels)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    /// Transmitter temperature, °C
    pub temp_c: f64,
    /// Forward power, W
    pub fwd_w: f64,
    /// Reflected power, W
    pub rev_w: f64,
}

/// Something [`Downlink::poll`] found, to report
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// Bad match: power capped at 0.5 W
    PowerReduced,
    /// Bad match at 0.5 W: downlink stopped for [`BACKOFF_MS`]
    BackOff,
    /// A keyed window with no forward power
    NoOutput,
    /// Above [`TEMP_LIMIT_C`]: downlink held
    OverTemperature,
    /// Below [`TEMP_RESUME_C`] again
    Cooled,
}

/// `DownlinkStatus_t` counters
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    /// Frames handed to the TNC
    pub sent: u32,
    /// Refused: not permitted
    pub inhibited: u32,
    /// Refused: invalid length
    pub invalid: u32,
    /// Refused: over temperature
    pub over_temperature: u32,
    /// Refused: backed off after a bad match
    pub backed_off: u32,
    /// Refused: the last frame still on the air
    pub busy: u32,
    /// Refused: no keyed-time credit
    pub duty: u32,
    /// The HAL did not take a frame
    pub hal_errors: u32,
    /// Bad-match detections
    pub vswr_events: u32,
    /// Keyed windows without output
    pub no_output: u32,
}

/// Worst-case air time of a frame of `len` bytes (addresses through
/// information): keying delay, opening flag, the frame and its FCS with a
/// stuffed bit after every five, closing flag, tail
pub fn airtime_ms(len: usize) -> u32 {
    u32::try_from(airtime_ms_wide(len)).unwrap_or(u32::MAX)
}

const fn airtime_ms_wide(len: usize) -> u64 {
    let bits = (len as u64).saturating_add(4).saturating_mul(8);
    let stuffed = bits.saturating_mul(6).div_ceil(5);
    let data_ms = stuffed.saturating_mul(1000).div_ceil(BIT_RATE as u64);
    (kiss::TXDELAY_10MS as u64 + kiss::TXTAIL_10MS as u64) * 10 + data_ms
}

// Every frame that fits is on the air for less than the key-down limit
// (SRS-P002): the longest, with every bit stuffed, takes about 3 s
const _: () = assert!(airtime_ms_wide(MAX_FRAME_LEN) <= MAX_KEYDOWN_MS as u64);

/// Worst-case time to send a frame of `len` bytes to the TNC, with its
/// parameters, ms
pub fn uart_ms(len: usize) -> u32 {
    let params = (kiss::DOWNLINK_PARAMS.len() * kiss::COMMAND_MAX_LEN) as u64;
    let bytes = params.saturating_add((len as u64).saturating_mul(2)).saturating_add(3);
    let ms = bytes.saturating_mul(10 * 1000).div_ceil(u64::from(UART_BAUD));
    u32::try_from(ms).unwrap_or(u32::MAX)
}

/// The downlink's state
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Downlink {
    initialized: bool,
    /// The last frame's estimated time on the air: [from, until); until 0
    /// when nothing is keyed
    keyed_from_ms: u64,
    keyed_until_ms: u64,
    /// Keyed-time credit, [`CREDIT_SCALE`] units, as of `credit_at_ms`
    credit: u64,
    credit_at_ms: u64,
    /// Highest power level allowed
    level_cap: u32,
    strikes: u8,
    samples: u32,
    saw_output: bool,
    backoff_until_ms: u64,
    hot: bool,
    /// What `downlink_get_status()` reports
    pub counters: Counters,
}

impl Downlink {
    /// Uninitialized: everything refused
    pub const fn new() -> Self {
        Downlink {
            initialized: false,
            keyed_from_ms: 0,
            keyed_until_ms: 0,
            credit: 0,
            credit_at_ms: 0,
            level_cap: POWER_1W,
            strikes: 0,
            samples: 0,
            saw_output: false,
            backoff_until_ms: 0,
            hot: false,
            counters: Counters {
                sent: 0,
                inhibited: 0,
                invalid: 0,
                over_temperature: 0,
                backed_off: 0,
                busy: 0,
                duty: 0,
                hal_errors: 0,
                vswr_events: 0,
                no_output: 0,
            },
        }
    }

    /// Start at `now_ms` with a full credit and nothing keyed
    pub fn init(&mut self, now_ms: u64) {
        *self = Downlink::new();
        self.initialized = true;
        self.credit = CREDIT_MAX;
        self.credit_at_ms = now_ms;
    }

    /// Decide on a frame of `len` bytes at `now_ms`: `permitted` is
    /// `rf_control_tx_permitted()` now, `level` the commanded power level,
    /// `temp_c` the transmitter temperature now. On success the frame is
    /// counted as on the air; [`Downlink::cancel`] undoes that if the HAL
    /// does not take it.
    pub fn request(
        &mut self,
        now_ms: u64,
        len: usize,
        permitted: bool,
        level: u32,
        temp_c: f64,
    ) -> Result<Plan, Refusal> {
        let result = self.decide(now_ms, len, permitted, level, temp_c);
        let c = &mut self.counters;
        let counter = match result {
            Ok(_) => &mut c.sent,
            Err(Refusal::Inhibited) => &mut c.inhibited,
            Err(Refusal::Invalid) => &mut c.invalid,
            Err(Refusal::OverTemperature) => &mut c.over_temperature,
            Err(Refusal::BackOff) => &mut c.backed_off,
            Err(Refusal::Busy) => &mut c.busy,
            Err(Refusal::Duty) => &mut c.duty,
        };
        *counter = counter.saturating_add(1);
        result
    }

    fn decide(&mut self, now_ms: u64, len: usize, permitted: bool, level: u32, temp_c: f64) -> Result<Plan, Refusal> {
        if !self.initialized || !permitted {
            return Err(Refusal::Inhibited);
        }
        if len == 0 || len > MAX_FRAME_LEN {
            return Err(Refusal::Invalid);
        }
        let airtime = airtime_ms(len);
        let _ = self.thermal(temp_c);
        if self.hot {
            return Err(Refusal::OverTemperature);
        }
        if now_ms < self.backoff_until_ms {
            return Err(Refusal::BackOff);
        }
        if self.keyed(now_ms) {
            return Err(Refusal::Busy);
        }
        self.refill(now_ms);
        let cost = u64::from(airtime) * CREDIT_SCALE;
        if cost > self.credit {
            return Err(Refusal::Duty);
        }
        self.credit -= cost;
        self.keyed_from_ms = now_ms;
        self.keyed_until_ms = now_ms.saturating_add(u64::from(uart_ms(len))).saturating_add(u64::from(airtime));
        self.strikes = 0;
        self.samples = 0;
        self.saw_output = false;
        Ok(Plan { level: if level >= POWER_1W { self.level_cap } else { POWER_0W5 }, airtime_ms: airtime })
    }

    /// The HAL did not take the frame [`Downlink::request`] accepted with
    /// `plan`: nothing is on the air, the credit is returned
    pub fn cancel(&mut self, plan: Plan) {
        self.credit = self.credit.saturating_add(u64::from(plan.airtime_ms) * CREDIT_SCALE).min(CREDIT_MAX);
        self.keyed_until_ms = 0;
        self.counters.sent = self.counters.sent.saturating_sub(1);
        self.counters.hal_errors = self.counters.hal_errors.saturating_add(1);
    }

    fn refill(&mut self, now_ms: u64) {
        let elapsed = now_ms.saturating_sub(self.credit_at_ms);
        self.credit = self.credit.saturating_add(elapsed.saturating_mul(DUTY_PERCENT)).min(CREDIT_MAX);
        self.credit_at_ms = now_ms;
    }

    /// Hysteresis on the transmitter temperature; the edge, if any. A
    /// reading that is not a number (a failed sensor) changes nothing
    fn thermal(&mut self, temp_c: f64) -> Option<Event> {
        if !self.hot && temp_c > TEMP_LIMIT_C {
            self.hot = true;
            return Some(Event::OverTemperature);
        }
        if self.hot && temp_c < TEMP_RESUME_C {
            self.hot = false;
            return Some(Event::Cooled);
        }
        None
    }

    /// Whether a frame may be on the air at `now_ms`
    pub fn keyed(&self, now_ms: u64) -> bool {
        self.keyed_until_ms != 0 && now_ms < self.keyed_until_ms
    }

    /// Each main-loop pass: while a frame may be on the air, `read` the
    /// monitors and apply the protections; `permitted` false (TX-OFF or an
    /// inhibit cut the transmitter) ends the window without a report
    pub fn poll(&mut self, now_ms: u64, permitted: bool, read: impl FnOnce() -> Sample) -> Option<Event> {
        if self.keyed_until_ms == 0 {
            return None;
        }
        if !permitted {
            self.keyed_until_ms = 0;
            return None;
        }
        if now_ms >= self.keyed_until_ms {
            self.keyed_until_ms = 0;
            if self.samples > 0 && !self.saw_output {
                self.counters.no_output = self.counters.no_output.saturating_add(1);
                return Some(Event::NoOutput);
            }
            return None;
        }
        if now_ms < self.keyed_from_ms.saturating_add(u64::from(kiss::TXDELAY_10MS) * 10) {
            return None;
        }
        let s = read();
        if let Some(event) = self.thermal(s.temp_c) {
            return Some(event);
        }
        // A power reading that failed (not a number) is no sample: it must
        // not look like a bad match or a dead transmitter
        if !(s.fwd_w.is_finite() && s.rev_w.is_finite()) {
            return None;
        }
        self.samples = self.samples.saturating_add(1);
        if s.fwd_w < RF_PRESENT_W {
            self.strikes = 0;
            return None;
        }
        self.saw_output = true;
        if s.rev_w <= s.fwd_w * VSWR_LIMIT_RATIO {
            self.strikes = 0;
            return None;
        }
        self.strikes += 1;
        if self.strikes < VSWR_STRIKES {
            return None;
        }
        self.strikes = 0;
        self.counters.vswr_events = self.counters.vswr_events.saturating_add(1);
        if self.level_cap > POWER_0W5 {
            self.level_cap = POWER_0W5;
            return Some(Event::PowerReduced);
        }
        self.backoff_until_ms = now_ms.saturating_add(BACKOFF_MS);
        Some(Event::BackOff)
    }

    /// Highest power level the protections allow
    pub const fn level_cap(&self) -> u32 {
        self.level_cap
    }

    /// Whether the transmitter is held for temperature
    pub const fn over_temperature(&self) -> bool {
        self.hot
    }

    /// Whether the downlink is backed off at `now_ms`
    pub fn backed_off(&self, now_ms: u64) -> bool {
        now_ms < self.backoff_until_ms
    }
}

impl Default for Downlink {
    fn default() -> Self {
        Downlink::new()
    }
}

#[cfg(test)]
#[path = "downlink_tests.rs"]
mod tests;
