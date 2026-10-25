//! The beacon (`beacon.h`, docs/BEACON_SPEC.md): text selection (filtered
//! Jetson text once, else the next template), the telemetry fields with
//! their validity (SW-13), the information field that goes on the air, the
//! AX.25 UI frame, and the schedule per power mode.
//!
//! The record types have the C layouts (`#[repr(C)]`), so the C side
//! passes them in and out unchanged.

use core::ffi::CStr;
use core::fmt::Write as _;

use sqso_protocol::{ax25, filter, station};

/// Text bytes, including the NUL (`BEACON_MAX_TEXT_LEN`)
pub const MAX_TEXT_LEN: usize = 200;
/// Information field bytes (`BEACON_MAX_PAYLOAD_LEN`)
pub const MAX_PAYLOAD_LEN: usize = 256;
/// The call sign as text (`BEACON_CALLSIGN`, the text form of `station`)
pub const CALLSIGN: &str = "KK6GDB-11";
/// Interval in ACTIVE, ms (the baseline: one beacon per 60 s)
pub const INTERVAL_ACTIVE_MS: u32 = 60_000;
/// Interval in IDLE, ms
pub const INTERVAL_IDLE_MS: u32 = 120_000;
/// Interval in SAFE, ms
pub const INTERVAL_SAFE_MS: u32 = 180_000;
/// Measurements older than this are not beaconed, ms
pub const MEAS_MAX_AGE_MS: u64 = 60_000;

/// Validity flag: state of charge
pub const TLM_SOC: u8 = 0x01;
/// Validity flag: battery voltage
pub const TLM_BV: u8 = 0x02;
/// Validity flag: sunlit
pub const TLM_SUN: u8 = 0x04;
/// Validity flag: radio power switch
pub const TLM_RF: u8 = 0x08;
/// Validity flag: QSO count
pub const TLM_QSO: u8 = 0x10;
/// The flags a measurement source may set
pub const MEASURED: u8 = TLM_SOC | TLM_BV | TLM_SUN | TLM_QSO;

/// `BeaconSource_t`: template text
pub const SOURCE_TEMPLATE: u32 = 0;
/// `BeaconSource_t`: Jetson text
pub const SOURCE_AI: u32 = 1;

/// `FaultSeverity_t` from which a fault sets `fault_flags` bit 0
pub const FAULT_SEVERITY_ERROR: u8 = 3;

/// Longest the fixed fields can be: "de " CALL ": " then " | T=" (u32),
/// " M=" (ACTIVE), " SOC=" (u8), " BV=" (65.5), " SUN=" (u8), " RF=" (u8),
/// " QSO=" (u16). The text gets whatever space is left, so the information
/// field always fits and the telemetry is never cut
pub const FIXED_MAX_LEN: usize = 3 + CALLSIGN.len() + 2 + 5 + 10 + 3 + 6 + 5 + 3 + 4 + 4 + 5 + 3 + 4 + 3 + 5 + 5;

/// Fallback texts, sent in turn while no Jetson text is available
pub const TEMPLATES: [&CStr; 8] = [
    c"SMART-QSO beacon - autonomous AI transponder demo",
    c"Hello from space! SMART-QSO CubeSat",
    c"Amateur radio in orbit - powered by AI",
    c"SMART-QSO 1U - technology demonstration mission",
    c"Greetings from LEO! 73 to all",
    c"SMART-QSO CubeSat beacon - reports welcome",
    c"SMART-QSO mission - thanks for listening!",
    c"Amateur satellite beacon - 73 de KK6GDB-11",
];

/// `BeaconTelemetry_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Telemetry {
    /// Mission time, s
    pub timestamp: u32,
    /// `PowerMode_t`
    pub power_mode: u8,
    /// State of charge, %
    pub soc: u8,
    /// Battery voltage, mV
    pub battery_mv: u16,
    /// OBC temperature, C
    pub temp_obc: i8,
    /// Payload temperature, C
    pub temp_payload: i8,
    /// Sunlit
    pub sunlit: u8,
    /// Radio power switch
    pub rf_power: u8,
    /// QSO count
    pub qso_count: u16,
    /// Bit 0: the last fault was ERROR or worse
    pub fault_flags: u8,
    /// `TLM_*` flags of the fields measured
    pub valid: u8,
}

/// `BeaconMeasurements_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Measurements {
    /// State of charge, % (0-100)
    pub soc: u8,
    /// Battery voltage, mV
    pub battery_mv: u16,
    /// 1 sunlit, 0 eclipse
    pub sunlit: u8,
    /// QSOs counted
    pub qso_count: u16,
    /// `TLM_SOC | TLM_BV | TLM_SUN | TLM_QSO`
    pub valid: u8,
}

impl Measurements {
    /// Whether every flagged value is in range (SOC at most 100, sunlit 0 or 1)
    pub fn in_range(&self) -> bool {
        !(((self.valid & TLM_SOC) != 0 && self.soc > 100) || ((self.valid & TLM_SUN) != 0 && self.sunlit > 1))
    }
}

/// `BeaconContent_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Content {
    /// Text, NUL-terminated
    pub text: [u8; MAX_TEXT_LEN],
    /// Text length
    pub text_len: usize,
    /// `BeaconSource_t`
    pub source: u32,
    /// Telemetry
    pub telemetry: Telemetry,
    /// Beacon sequence number
    pub sequence: u32,
}

impl Content {
    /// All zero
    pub const ZERO: Content = Content {
        text: [0; MAX_TEXT_LEN],
        text_len: 0,
        source: SOURCE_TEMPLATE,
        telemetry: TELEMETRY_ZERO,
        sequence: 0,
    };

    /// The text up to its NUL (at most the whole array)
    pub fn text(&self) -> &[u8] {
        let len = self.text.iter().position(|b| *b == 0).unwrap_or(MAX_TEXT_LEN);
        self.text.get(..len).unwrap_or(&[])
    }

    fn set_text(&mut self, text: &[u8], source: u32) {
        let len = text.len().min(MAX_TEXT_LEN - 1);
        for (dst, src) in self.text.iter_mut().zip(text.iter().take(len)) {
            *dst = *src;
        }
        if let Some(nul) = self.text.get_mut(len) {
            *nul = 0;
        }
        self.text_len = len;
        self.source = source;
    }
}

const TELEMETRY_ZERO: Telemetry = Telemetry {
    timestamp: 0,
    power_mode: 0,
    soc: 0,
    battery_mv: 0,
    temp_obc: 0,
    temp_payload: 0,
    sunlit: 0,
    rf_power: 0,
    qso_count: 0,
    fault_flags: 0,
    valid: 0,
};

/// `Ax25Frame_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Destination address
    pub dest_addr: [u8; ax25::ADDR_LEN],
    /// Source address
    pub src_addr: [u8; ax25::ADDR_LEN],
    /// Control field
    pub ctrl: u8,
    /// Protocol ID
    pub pid: u8,
    /// Information field
    pub info: [u8; MAX_PAYLOAD_LEN],
    /// Information length
    pub info_len: usize,
    /// Frame check sequence
    pub fcs: u16,
}

impl Frame {
    /// All zero
    pub const ZERO: Frame = Frame {
        dest_addr: [0; ax25::ADDR_LEN],
        src_addr: [0; ax25::ADDR_LEN],
        ctrl: 0,
        pid: 0,
        info: [0; MAX_PAYLOAD_LEN],
        info_len: 0,
        fcs: 0,
    };
}

/// `BeaconState_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    /// Last beacon, ms
    pub last_beacon_ms: u64,
    /// Current interval, ms
    pub interval_ms: u32,
    /// Beacon sequence counter
    pub sequence: u32,
    /// Beacons sent with Jetson text
    pub ai_beacon_count: u32,
    /// Beacons sent with a template
    pub template_count: u32,
    /// Jetson texts the content filter refused
    pub ai_rejected_count: u32,
    /// Bytes transmitted
    pub total_bytes_tx: u32,
    /// Jetson text generation available
    pub ai_available: bool,
    /// Stored Jetson text (empty once used)
    pub last_ai_text: [u8; MAX_TEXT_LEN],
}

impl State {
    /// All zero
    pub const ZERO: State = State {
        last_beacon_ms: 0,
        interval_ms: 0,
        sequence: 0,
        ai_beacon_count: 0,
        template_count: 0,
        ai_rejected_count: 0,
        total_bytes_tx: 0,
        ai_available: false,
        last_ai_text: [0; MAX_TEXT_LEN],
    };
}

/// What the telemetry fields are read from at the time of a beacon
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Sources {
    /// Mission time, ms
    pub now_ms: u64,
    /// EPS power mode (`PowerMode_t`)
    pub power_mode: u32,
    /// Radio power switch, if the EPS reports it
    pub radio_enabled: Option<bool>,
    /// Severity of the newest logged fault, if any
    pub last_fault_severity: Option<u8>,
}

/// The beacon interval for a power mode (unknown modes: the ACTIVE rate,
/// never faster)
pub fn interval_for(power_mode: u32) -> u32 {
    match power_mode {
        0 => INTERVAL_SAFE_MS,
        1 => INTERVAL_IDLE_MS,
        _ => INTERVAL_ACTIVE_MS,
    }
}

/// Whether a text passes the content filter (BEACON_SPEC.md section 6)
pub fn text_ok(text: &[u8]) -> bool {
    filter::check(text) == filter::Verdict::Ok
}

/// `snprintf` into a byte buffer: writes what fits with a NUL after it,
/// counts what would have been written
struct Out<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl Out<'_> {
    fn bytes(&mut self, b: &[u8]) {
        for &c in b {
            if self.len + 1 < self.buf.len()
                && let Some(slot) = self.buf.get_mut(self.len)
            {
                *slot = c;
            }
            self.len += 1;
        }
    }

    /// NUL-terminate; the length snprintf returns
    fn finish(self) -> usize {
        let end = self.len.min(self.buf.len().saturating_sub(1));
        if let Some(slot) = self.buf.get_mut(end) {
            *slot = 0;
        }
        self.len
    }
}

impl core::fmt::Write for Out<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.bytes(s.as_bytes());
        Ok(())
    }
}

/// A telemetry field: its value, or "-" when not flagged valid
fn field(out: &mut Out<'_>, valid: u8, flag: u8, value: u32) {
    if valid & flag != 0 {
        let _ = write!(out, "{value}");
    } else {
        out.bytes(b"-");
    }
}

/// The information field: `de CALL: text | T=.. M=.. SOC=.. BV=.. SUN=..
/// RF=.. QSO=..`, NUL-terminated in `buf`. The text is cut to fit, the
/// telemetry never; 0 if `buf` cannot hold the fixed fields
pub fn format_payload(content: &Content, buf: &mut [u8]) -> usize {
    let Some(room) = buf.len().checked_sub(FIXED_MAX_LEN + 1) else {
        return 0;
    };
    let text = content.text();
    let text = text.get(..text.len().min(room)).unwrap_or(&[]);
    let t = &content.telemetry;
    let buf_len = buf.len();
    let mut out = Out { buf, len: 0 };
    out.bytes(b"de ");
    out.bytes(CALLSIGN.as_bytes());
    out.bytes(b": ");
    out.bytes(text);
    let mode = match t.power_mode {
        0 => "SAFE",
        1 => "IDLE",
        _ => "ACTIVE",
    };
    let _ = write!(out, " | T={} M={mode} SOC=", t.timestamp);
    field(&mut out, t.valid, TLM_SOC, u32::from(t.soc));
    out.bytes(b" BV=");
    if t.valid & TLM_BV != 0 {
        // Battery volts to one decimal, rounded, in integer arithmetic
        let tenths = (u32::from(t.battery_mv) + 50) / 100;
        let _ = write!(out, "{}.{}", tenths / 10, tenths % 10);
    } else {
        out.bytes(b"-");
    }
    out.bytes(b" SUN=");
    field(&mut out, t.valid, TLM_SUN, u32::from(t.sunlit));
    out.bytes(b" RF=");
    field(&mut out, t.valid, TLM_RF, u32::from(t.rf_power));
    out.bytes(b" QSO=");
    field(&mut out, t.valid, TLM_QSO, u32::from(t.qso_count));
    let written = out.finish();
    if written >= buf_len { 0 } else { written }
}

/// The UI frame CQ <- KK6GDB-11 carrying the information field, with its
/// FCS; `false` (with the frame as far as it was built) if the information
/// field could not be formatted
pub fn build_frame(content: &Content) -> (Frame, bool) {
    let mut frame = Frame {
        dest_addr: ax25::encode_address(b"CQ", 0, false),
        src_addr: ax25::encode_address(station::CALLSIGN.as_bytes(), station::SSID, true),
        ctrl: ax25::CTRL_UI,
        pid: ax25::PID_NO_L3,
        ..Frame::ZERO
    };
    frame.info_len = format_payload(content, &mut frame.info);
    if frame.info_len == 0 {
        return (frame, false);
    }
    frame.fcs = ax25::fcs_parts(&[
        frame.dest_addr.as_slice(),
        frame.src_addr.as_slice(),
        &[frame.ctrl, frame.pid],
        frame.info.get(..frame.info_len).unwrap_or(&[]),
    ]);
    (frame, true)
}

/// Length of the frame on the air ([`serialize`]'s result for a buffer of
/// [`ax25::MAX_FRAME_LEN`]); 0 if the information length is beyond its field
pub fn serialized_len(frame: &Frame) -> usize {
    let Some(info) = frame.info.get(..frame.info_len) else { return 0 };
    let size = 1 + 2 * ax25::ADDR_LEN + 2 + info.len() + 2 + 1;
    if size <= ax25::MAX_FRAME_LEN { size } else { 0 }
}

/// The frame on the air: flag, addresses, control, PID, information, FCS
/// (little-endian), flag. 0 if `out` is too small (or the frame's
/// information length is beyond its field)
pub fn serialize(frame: &Frame, out: &mut [u8]) -> usize {
    let Some(info) = frame.info.get(..frame.info_len) else { return 0 };
    let size = 1 + 2 * ax25::ADDR_LEN + 2 + info.len() + 2 + 1;
    let Some(dst) = out.get_mut(..size) else { return 0 };
    let fcs = frame.fcs.to_le_bytes();
    let mut n = 0;
    for part in [
        [ax25::FLAG].as_slice(),
        frame.dest_addr.as_slice(),
        frame.src_addr.as_slice(),
        &[frame.ctrl, frame.pid],
        info,
        &fcs,
        &[ax25::FLAG],
    ] {
        if let Some(d) = dst.get_mut(n..n + part.len()) {
            d.copy_from_slice(part);
        }
        n += part.len();
    }
    n
}

/// The beacon module's state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Beacon {
    /// What `beacon_get_state()` reports
    pub state: State,
    /// Initialized
    pub initialized: bool,
    template_index: usize,
    meas: Measurements,
    meas_ms: u64,
    meas_set: bool,
}

impl Beacon {
    /// Before `beacon_init()`
    pub const fn new() -> Beacon {
        Beacon {
            state: State::ZERO,
            initialized: false,
            template_index: 0,
            meas: Measurements { soc: 0, battery_mv: 0, sunlit: 0, qso_count: 0, valid: 0 },
            meas_ms: 0,
            meas_set: false,
        }
    }

    /// `beacon_init()`: ACTIVE rate, templates from the first, no history,
    /// no measurements
    pub fn init(&mut self) {
        *self = Beacon { initialized: true, ..Beacon::new() };
        self.state.interval_ms = INTERVAL_ACTIVE_MS;
    }

    /// Whether the interval has passed since the last beacon
    pub fn interval_elapsed(&self, now_ms: u64) -> bool {
        now_ms.wrapping_sub(self.state.last_beacon_ms) >= u64::from(self.state.interval_ms)
    }

    /// The interval for the power mode (nothing before init)
    pub fn update_interval(&mut self, power_mode: u32) {
        if self.initialized {
            self.state.interval_ms = interval_for(power_mode);
        }
    }

    /// The index in [`TEMPLATES`] of the next template, in turn
    pub fn next_template(&mut self) -> usize {
        let index = self.template_index;
        self.template_index += 1;
        if self.template_index >= TEMPLATES.len() {
            self.template_index = 0;
        }
        index
    }

    /// The text: the given Jetson text if it passes the filter, else the
    /// stored text (once) while AI is available, else the next template
    fn select_text(&mut self, ai_text: Option<&[u8]>, content: &mut Content) {
        let given = ai_text.filter(|t| !t.is_empty());
        let ok = given.is_some_and(text_ok);
        if given.is_some() && !ok {
            self.state.ai_rejected_count = self.state.ai_rejected_count.wrapping_add(1);
        }
        match given {
            Some(text) if ok => content.set_text(text, SOURCE_AI),
            _ if self.state.ai_available && self.state.last_ai_text[0] != 0 => {
                let stored = self.state.last_ai_text;
                let len = stored.iter().position(|b| *b == 0).unwrap_or(MAX_TEXT_LEN);
                content.set_text(stored.get(..len).unwrap_or(&[]), SOURCE_AI);
                // Stored Jetson text is used for one beacon only (BEACON_SPEC section 5)
                self.state.last_ai_text[0] = 0;
            }
            _ => {
                let t = TEMPLATES.get(self.next_template()).copied().unwrap_or(c"");
                content.set_text(t.to_bytes(), SOURCE_TEMPLATE);
            }
        }
    }

    /// Telemetry: measured values only, each with its validity flag (SW-13);
    /// fields not measured, or measured too long ago, are sent "-"
    fn telemetry(&self, src: &Sources) -> Telemetry {
        let now = src.now_ms;
        #[allow(clippy::cast_possible_truncation)] // low bits, as the C conversions
        let mut t = Telemetry { timestamp: (now / 1000) as u32, power_mode: src.power_mode as u8, ..TELEMETRY_ZERO };
        if self.meas_set && now >= self.meas_ms && now - self.meas_ms <= MEAS_MAX_AGE_MS {
            let m = &self.meas;
            if m.valid & TLM_SOC != 0 {
                t.soc = m.soc;
            }
            if m.valid & TLM_BV != 0 {
                t.battery_mv = m.battery_mv;
            }
            if m.valid & TLM_SUN != 0 {
                t.sunlit = m.sunlit;
            }
            if m.valid & TLM_QSO != 0 {
                t.qso_count = m.qso_count;
            }
            t.valid = m.valid & MEASURED;
        }
        if let Some(on) = src.radio_enabled {
            t.rf_power = u8::from(on);
            t.valid |= TLM_RF;
        }
        if let Some(severity) = src.last_fault_severity {
            t.fault_flags = u8::from(severity >= FAULT_SEVERITY_ERROR);
        }
        t
    }

    /// The next beacon's content (`beacon_generate_content()`)
    pub fn generate(&mut self, ai_text: Option<&[u8]>, src: &Sources) -> Content {
        let mut content = Content::ZERO;
        self.select_text(ai_text, &mut content);
        content.telemetry = self.telemetry(src);
        self.state.sequence = self.state.sequence.wrapping_add(1);
        content.sequence = self.state.sequence;
        content
    }

    /// Count a transmitted frame
    pub fn count_transmit(&mut self, tx_len: usize, source: u32) {
        #[allow(clippy::cast_possible_truncation)] // a frame is at most 330 bytes
        let bytes = tx_len as u32;
        self.state.total_bytes_tx = self.state.total_bytes_tx.wrapping_add(bytes);
        if source == SOURCE_AI {
            self.state.ai_beacon_count = self.state.ai_beacon_count.wrapping_add(1);
        } else {
            self.state.template_count = self.state.template_count.wrapping_add(1);
        }
    }

    /// Store Jetson text for the next beacon; false (counted) if the filter
    /// refuses it
    pub fn store_ai_text(&mut self, text: &[u8]) -> bool {
        if !text_ok(text) {
            self.state.ai_rejected_count = self.state.ai_rejected_count.wrapping_add(1);
            return false;
        }
        let len = text.len().min(MAX_TEXT_LEN - 1);
        for (dst, src) in self.state.last_ai_text.iter_mut().zip(text.iter().take(len)) {
            *dst = *src;
        }
        if let Some(nul) = self.state.last_ai_text.get_mut(len) {
            *nul = 0;
        }
        true
    }

    /// Record measurements; false (nothing recorded) if a flagged value is
    /// out of range
    pub fn update_measurements(&mut self, m: &Measurements, sample_ms: u64) -> bool {
        if !m.in_range() {
            return false;
        }
        self.meas = *m;
        self.meas_ms = sample_ms;
        self.meas_set = true;
        true
    }

    /// Beacons transmitted (0 before init)
    pub fn total_count(&self) -> u32 {
        if self.initialized { self.state.ai_beacon_count.wrapping_add(self.state.template_count) } else { 0 }
    }

    /// Percentage of beacons with Jetson text (0 before init or before any)
    pub fn ai_percentage(&self) -> u8 {
        let total = self.total_count();
        if total == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)] // at most 100
        let pct = (self.state.ai_beacon_count.wrapping_mul(100) / total) as u8;
        pct
    }

    /// Clear the transmission statistics and the sequence (nothing before init)
    pub fn reset_stats(&mut self) {
        if self.initialized {
            self.state.ai_beacon_count = 0;
            self.state.template_count = 0;
            self.state.total_bytes_tx = 0;
            self.state.sequence = 0;
        }
    }
}

impl Default for Beacon {
    fn default() -> Self {
        Beacon::new()
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    fn text_of(buf: &[u8], len: usize) -> &str {
        core::str::from_utf8(&buf[..len]).unwrap()
    }

    fn filled(text_len: usize) -> Content {
        let mut c = Content::ZERO;
        c.text[..text_len].fill(b'A');
        c.text_len = text_len;
        c.telemetry.valid = TLM_SOC | TLM_BV | TLM_SUN | TLM_RF | TLM_QSO;
        c
    }

    #[test]
    fn callsign_is_the_station() {
        assert_eq!(CALLSIGN, std::format!("{}-{}", station::CALLSIGN, station::SSID));
        assert_eq!(FIXED_MAX_LEN, 79);
    }

    /// The golden lines of test_beacon_format.c
    #[test]
    fn golden_payloads() {
        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        let mut c = filled(0);
        c.set_text(b"Hello from orbit", SOURCE_TEMPLATE);
        c.telemetry.battery_mv = 7400;
        c.telemetry.power_mode = 1;
        let n = format_payload(&c, &mut buf);
        assert_eq!(text_of(&buf, n), "de KK6GDB-11: Hello from orbit | T=0 M=IDLE SOC=0 BV=7.4 SUN=0 RF=0 QSO=0");
        assert_eq!(buf[n], 0);

        let mut c = filled(MAX_TEXT_LEN - 1);
        c.telemetry = Telemetry {
            timestamp: u32::MAX,
            power_mode: 2,
            soc: u8::MAX,
            battery_mv: u16::MAX,
            sunlit: u8::MAX,
            rf_power: u8::MAX,
            qso_count: u16::MAX,
            ..c.telemetry
        };
        let n = format_payload(&c, &mut buf);
        assert!(n > 0 && n < buf.len());
        let s = text_of(&buf, n);
        assert!(s.starts_with("de KK6GDB-11: AAAA"));
        assert!(s.ends_with(" BV=65.5 SUN=255 RF=255 QSO=65535"));
        assert!(s.contains(" | T=4294967295 M=ACTIVE SOC=255"));
    }

    #[test]
    fn unmeasured_fields_are_dashes() {
        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        let mut c = filled(1);
        c.telemetry.valid = 0;
        let n = format_payload(&c, &mut buf);
        assert_eq!(text_of(&buf, n), "de KK6GDB-11: A | T=0 M=SAFE SOC=- BV=- SUN=- RF=- QSO=-");
    }

    #[test]
    fn battery_voltage_rounds_to_tenths() {
        let mut buf = [0u8; MAX_PAYLOAD_LEN];
        for (mv, text) in [
            (0, "BV=0.0 "),
            (7400, "BV=7.4 "),
            (7449, "BV=7.4 "),
            (7450, "BV=7.5 "),
            (8399, "BV=8.4 "),
            (65535, "BV=65.5 "),
        ] {
            let mut c = filled(1);
            c.telemetry.battery_mv = mv;
            let n = format_payload(&c, &mut buf);
            assert!(text_of(&buf, n).contains(text), "{mv}");
        }
    }

    #[test]
    fn small_buffers_cut_the_text_only() {
        let c = filled(150);
        let mut buf = [0u8; 100];
        let n = format_payload(&c, &mut buf);
        assert!(n > 0 && n < 100);
        assert!(text_of(&buf, n).ends_with(" QSO=0"));
        assert_eq!(format_payload(&c, &mut buf[..40]), 0);
        assert_eq!(format_payload(&c, &mut buf[..FIXED_MAX_LEN]), 0);
        // The fixed fields alone, no text
        let mut exact = [0u8; FIXED_MAX_LEN + 1];
        let n = format_payload(&c, &mut exact);
        assert!(text_of(&exact, n).starts_with("de KK6GDB-11:  | T="));
    }

    #[test]
    fn frame_and_serialization() {
        let mut c = filled(0);
        c.set_text(b"Hello from orbit", SOURCE_AI);
        let (frame, ok) = build_frame(&c);
        assert!(ok);
        assert_eq!(frame.ctrl, 0x03);
        assert_eq!(frame.pid, 0xF0);
        assert!(ax25::address_matches(&frame.dest_addr, b"CQ", 0));
        assert!(ax25::address_matches(&frame.src_addr, station::CALLSIGN.as_bytes(), station::SSID));
        let mut out = [0u8; ax25::MAX_FRAME_LEN];
        let n = serialize(&frame, &mut out);
        assert_eq!(n, frame.info_len + 1 + 14 + 2 + 2 + 1);
        assert_eq!((out[0], out[n - 1]), (0x7E, 0x7E));
        assert_eq!(u16::from_le_bytes([out[n - 3], out[n - 2]]), ax25::fcs(&out[1..n - 3]));
        let ui = ax25::parse_ui(&out[1..n - 1]).unwrap();
        assert_eq!(ui.info, &frame.info[..frame.info_len]);
        assert_eq!(serialize(&frame, &mut out[..n - 1]), 0);
        assert_eq!(serialized_len(&frame), n);
        let bad = Frame { info_len: MAX_PAYLOAD_LEN + 1, ..frame };
        assert_eq!(serialize(&bad, &mut out), 0);
        assert_eq!(serialized_len(&bad), 0);
    }

    #[test]
    fn templates_in_turn_then_stored_text_once() {
        let mut b = Beacon::new();
        b.init();
        let src = Sources::default();
        for round in 0..2u32 {
            for (i, t) in TEMPLATES.iter().enumerate() {
                let c = b.generate(None, &src);
                assert_eq!(c.source, SOURCE_TEMPLATE);
                assert_eq!(c.text(), t.to_bytes());
                assert_eq!(c.text_len, t.to_bytes().len());
                assert_eq!(c.sequence, round * 8 + u32::try_from(i).unwrap() + 1);
            }
        }
        b.state.ai_available = true;
        assert!(b.store_ai_text(b"Hello from orbit"));
        let c = b.generate(None, &src);
        assert_eq!((c.source, c.text()), (SOURCE_AI, &b"Hello from orbit"[..]));
        assert_eq!(b.generate(None, &src).source, SOURCE_TEMPLATE);
    }

    #[test]
    fn given_text_must_pass_the_filter() {
        let mut b = Beacon::new();
        b.init();
        let src = Sources::default();
        let c = b.generate(Some(b"Hello, world from orbit"), &src);
        assert_eq!(c.source, SOURCE_AI);
        let c = b.generate(Some(b"buy now at www.example.com"), &src);
        assert_eq!(c.source, SOURCE_TEMPLATE);
        assert_eq!(b.state.ai_rejected_count, 1);
        assert_eq!(b.generate(Some(b""), &src).source, SOURCE_TEMPLATE);
        assert_eq!(b.state.ai_rejected_count, 1);
        assert!(!b.store_ai_text(b"\x01"));
        assert_eq!(b.state.ai_rejected_count, 2);
    }

    #[test]
    fn telemetry_from_fresh_measurements_only() {
        let mut b = Beacon::new();
        b.init();
        let m = Measurements { soc: 42, battery_mv: 8123, sunlit: 0, qso_count: 7, valid: MEASURED };
        assert!(b.update_measurements(&m, 100_000));
        let src = Sources { now_ms: 160_000, power_mode: 2, radio_enabled: Some(true), last_fault_severity: Some(3) };
        let t = b.generate(None, &src).telemetry;
        assert_eq!((t.soc, t.battery_mv, t.qso_count, t.valid), (42, 8123, 7, MEASURED | TLM_RF));
        assert_eq!((t.timestamp, t.power_mode, t.rf_power, t.fault_flags), (160, 2, 1, 1));
        let stale = Sources { now_ms: 160_001, ..src };
        assert_eq!(b.generate(None, &stale).telemetry.valid, TLM_RF);
        let early = Sources { now_ms: 99_999, last_fault_severity: Some(2), radio_enabled: None, ..src };
        let t = b.generate(None, &early).telemetry;
        assert_eq!((t.valid, t.fault_flags), (0, 0));

        assert!(!b.update_measurements(&Measurements { soc: 101, valid: TLM_SOC, ..m }, 0));
        assert!(!b.update_measurements(&Measurements { sunlit: 2, valid: TLM_SUN, ..m }, 0));
        assert!(b.update_measurements(&Measurements { soc: 101, valid: TLM_BV, ..m }, 0));
    }

    #[test]
    fn schedule_and_statistics() {
        let mut b = Beacon::new();
        b.update_interval(0);
        assert_eq!(b.state.interval_ms, 0);
        assert_eq!(b.total_count(), 0);
        b.init();
        for (mode, ms) in
            [(0, INTERVAL_SAFE_MS), (1, INTERVAL_IDLE_MS), (2, INTERVAL_ACTIVE_MS), (3, INTERVAL_ACTIVE_MS)]
        {
            b.update_interval(mode);
            assert_eq!(b.state.interval_ms, ms);
        }
        b.state.last_beacon_ms = 5_000_000_000;
        assert!(!b.interval_elapsed(5_000_059_999));
        assert!(b.interval_elapsed(5_000_060_000));
        b.count_transmit(100, SOURCE_TEMPLATE);
        b.count_transmit(90, SOURCE_AI);
        b.count_transmit(90, SOURCE_AI);
        assert_eq!((b.total_count(), b.ai_percentage(), b.state.total_bytes_tx), (3, 66, 280));
        b.reset_stats();
        assert_eq!((b.total_count(), b.ai_percentage(), b.state.sequence), (0, 0, 0));
    }

    /// Mutation check: the range check looks only at flagged fields, and
    /// 100 % is in range
    #[test]
    fn range_check_edges() {
        let m = Measurements { soc: 100, sunlit: 1, valid: TLM_SOC | TLM_SUN, ..Measurements::default() };
        assert!(m.in_range());
        assert!(Measurements { sunlit: 5, valid: TLM_SOC, ..m }.in_range(), "sunlit not flagged");
        assert!(!Measurements { sunlit: 5, valid: TLM_SUN, ..m }.in_range());
    }

    /// Mutation check: each field is copied only when flagged, and the
    /// validity carries only the measured flags (never RF from a
    /// measurement)
    #[test]
    fn telemetry_copies_flagged_fields_only() {
        let all = Measurements { soc: 42, battery_mv: 8000, sunlit: 1, qso_count: 7, valid: 0 };
        let src = Sources { now_ms: 1000, ..Sources::default() };
        for flag in [TLM_SOC, TLM_BV, TLM_SUN, TLM_QSO] {
            let mut b = Beacon::new();
            b.init();
            assert!(b.update_measurements(&Measurements { valid: flag | TLM_RF, ..all }, 1000));
            let t = b.generate(None, &src).telemetry;
            assert_eq!(t.valid, flag, "{flag:#x}");
            assert_eq!(t.soc, if flag == TLM_SOC { 42 } else { 0 });
            assert_eq!(t.battery_mv, if flag == TLM_BV { 8000 } else { 0 });
            assert_eq!(t.sunlit, if flag == TLM_SUN { 1 } else { 0 });
            assert_eq!(t.qso_count, if flag == TLM_QSO { 7 } else { 0 });
            assert_eq!(t.rf_power, 0);
        }
    }

    #[test]
    fn accepted_texts_are_not_counted_as_rejected() {
        let mut b = Beacon::new();
        b.init();
        let src = Sources::default();
        assert_eq!(b.generate(Some(b"Hello from orbit"), &src).source, SOURCE_AI);
        assert_eq!(b.generate(Some(b"Greetings from LEO"), &src).source, SOURCE_AI);
        assert_eq!(b.state.ai_rejected_count, 0);
    }

    /// The text helper keeps at most 199 bytes and a NUL
    #[test]
    fn set_text_is_bounded() {
        let mut c = Content::ZERO;
        c.set_text(&[b'x'; 250], SOURCE_AI);
        assert_eq!(c.text_len, MAX_TEXT_LEN - 1);
        assert_eq!(c.text().len(), MAX_TEXT_LEN - 1);
        assert_eq!(c.text[MAX_TEXT_LEN - 1], 0);
    }

    /// The snprintf helper writes what fits, a NUL after it, and counts
    /// everything
    #[test]
    fn out_writes_what_fits() {
        let mut buf = [0xFFu8; 4];
        let mut out = Out { buf: &mut buf, len: 0 };
        out.bytes(b"abcdef");
        assert_eq!(out.finish(), 6);
        assert_eq!(&buf, b"abc\0");
    }

    /// Mutation check: the longest information field gets the right FCS,
    /// and serialization needs exactly the frame's size
    #[test]
    fn longest_frame_fcs_and_exact_buffer() {
        let mut c = filled(MAX_TEXT_LEN - 1);
        c.telemetry = Telemetry {
            timestamp: u32::MAX,
            power_mode: 2,
            soc: u8::MAX,
            battery_mv: u16::MAX,
            sunlit: u8::MAX,
            rf_power: u8::MAX,
            qso_count: u16::MAX,
            ..c.telemetry
        };
        let (frame, ok) = build_frame(&c);
        assert!(ok);
        assert!(frame.info_len >= MAX_PAYLOAD_LEN - 3, "{}", frame.info_len);
        let mut out = [0u8; ax25::MAX_FRAME_LEN];
        let n = serialize(&frame, &mut out);
        assert_eq!(n, frame.info_len + 20);
        assert_eq!(u16::from_le_bytes([out[n - 3], out[n - 2]]), ax25::fcs(&out[1..n - 3]));
        let mut exact = [0u8; ax25::MAX_FRAME_LEN];
        assert_eq!(serialize(&frame, &mut exact[..n]), n);
        assert_eq!(&exact[..n], &out[..n]);
        // A short frame fits its exact size too
        let (short, _) = build_frame(&filled(1));
        let m = short.info_len + 20;
        assert_eq!(serialize(&short, &mut exact[..m]), m);
    }
}
