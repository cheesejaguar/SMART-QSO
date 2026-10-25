//! Sensor framework (`sensors.h`): the sensor configuration file, the
//! sensor kinds and their simulated readings, the status bitmap, and the
//! telemetry text.
//!
//! The configuration parser reads exactly the YAML subset the C loader
//! read, line by line as `fgets` delivered it (at most 255 bytes a call):
//! a `sensors:` line, then list items (`- key: value`) with `key: value`
//! fields. It is bounded and allocation-free; the file is untrusted input
//! (fuzz target `sensors_yaml`).
//!
//! @requirement SRS-F070 Collect telemetry from all sensors
//! @requirement SRS-F071 Sample sensors at configurable rates
//! @requirement SRS-F072 Validate sensor readings against limits

use core::ffi::c_ulong;
use core::fmt::{self, Write};

/// Sensors kept (`SMART_QSO_MAX_SENSORS`)
pub const MAX_SENSORS: usize = 32;
/// Identifier bytes including the NUL (`SMART_QSO_SENSOR_ID_LEN`)
pub const ID_LEN: usize = 8;
/// Name bytes including the NUL (`SMART_QSO_SENSOR_NAME_LEN`)
pub const NAME_LEN: usize = 64;
/// Units bytes including the NUL (`SMART_QSO_SENSOR_UNITS_LEN`)
pub const UNITS_LEN: usize = 16;
/// Type bytes including the NUL (`SMART_QSO_SENSOR_TYPE_LEN`)
pub const TYPE_LEN: usize = 32;
/// Channel bytes including the NUL (`SMART_QSO_SENSOR_CHANNEL_LEN`)
pub const CHANNEL_LEN: usize = 32;
/// Text reading bytes including the NUL (`Sensor_t.last_text`)
pub const TEXT_LEN: usize = 8;
/// Poll period when none is configured, ms
pub const DEFAULT_PERIOD_MS: u32 = 1000;

/// The bytes of a NUL-terminated field up to its NUL
pub fn until_nul(field: &[u8]) -> &[u8] {
    let len = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    field.get(..len).unwrap_or(&[])
}

/// `snprintf(dst, sizeof dst, "%s", src)`: up to `dst.len() - 1` bytes of
/// `src` (up to its NUL), then a NUL; bytes after the NUL are left as they were
pub fn copy_cstr(dst: &mut [u8], src: &[u8]) {
    let src = until_nul(src);
    let n = src.len().min(dst.len().saturating_sub(1));
    if let (Some(d), Some(s)) = (dst.get_mut(..n), src.get(..n)) {
        d.copy_from_slice(s);
    }
    if let Some(nul) = dst.get_mut(n) {
        *nul = 0;
    }
}

/// A sensor as configured (the configuration fields of `Sensor_t`)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SensorDef {
    /// Short identifier, NUL-terminated
    pub id: [u8; ID_LEN],
    /// Full name, NUL-terminated
    pub name: [u8; NAME_LEN],
    /// Units, NUL-terminated
    pub units: [u8; UNITS_LEN],
    /// Sensor type (selects the [`Kind`]), NUL-terminated
    pub type_: [u8; TYPE_LEN],
    /// Channel, NUL-terminated
    pub channel: [u8; CHANNEL_LEN],
    /// Poll period, ms (0: [`DEFAULT_PERIOD_MS`])
    pub period_ms: u32,
}

impl SensorDef {
    /// All fields empty
    pub const EMPTY: SensorDef = SensorDef {
        id: [0; ID_LEN],
        name: [0; NAME_LEN],
        units: [0; UNITS_LEN],
        type_: [0; TYPE_LEN],
        channel: [0; CHANNEL_LEN],
        period_ms: 0,
    };

    /// A definition from strings (cut to the field sizes)
    pub fn new(id: &[u8], name: &[u8], type_: &[u8], units: &[u8], channel: &[u8], period_ms: u32) -> SensorDef {
        let mut d = SensorDef::EMPTY;
        copy_cstr(&mut d.id, id);
        copy_cstr(&mut d.name, name);
        copy_cstr(&mut d.type_, type_);
        copy_cstr(&mut d.units, units);
        copy_cstr(&mut d.channel, channel);
        d.period_ms = period_ms;
        d
    }

    /// The kind its type names, if known
    pub fn kind(&self) -> Option<Kind> {
        Kind::from_type(until_nul(&self.type_))
    }

    /// Store one `key: value` field; unknown keys are ignored
    fn set_field(&mut self, key: &[u8], val: &[u8]) {
        match key {
            b"id" => copy_cstr(&mut self.id, val),
            b"name" => copy_cstr(&mut self.name, val),
            b"type" => copy_cstr(&mut self.type_, val),
            b"units" => copy_cstr(&mut self.units, val),
            b"channel" => copy_cstr(&mut self.channel, val),
            b"period_ms" => self.period_ms = strtoul_u32(val),
            _ => {}
        }
    }
}

/// The built-in sensor table (used without a configuration file)
pub fn defaults() -> [SensorDef; 10] {
    [
        SensorDef::new(b"SET", b"Spacecraft Elapsed Timer", b"software_timer", b"s", b"", 1000),
        SensorDef::new(b"BV", b"Battery Voltage", b"eps_voltage", b"V", b"battery", 1000),
        SensorDef::new(b"BT", b"Battery Temperature", b"eps_temperature", b"C", b"battery", 2000),
        SensorDef::new(b"BUSV", b"Bus Voltage", b"eps_voltage", b"V", b"bus", 1000),
        SensorDef::new(b"BDI", b"Battery Discharge Current", b"eps_current", b"A", b"battery_discharge", 1000),
        SensorDef::new(b"JPC", b"Jetson Payload Current", b"eps_current", b"A", b"jetson", 1000),
        SensorDef::new(b"PT", b"Payload Heat Spreader Temperature", b"payload_temperature", b"C", b"payload", 1000),
        SensorDef::new(b"SPV", b"Solar Panel Voltage", b"eps_voltage", b"V", b"solar", 1000),
        SensorDef::new(b"SPC", b"Solar Panel Current", b"eps_current", b"A", b"solar", 1000),
        SensorDef::new(b"ST", b"Status Hex", b"status_hex2", b"hex", b"", 1000),
    ]
}

/// What a sensor's reading is
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ValueType {
    /// A number (`SENSOR_VALUE_NUMERIC`)
    Numeric = 0,
    /// Two hex characters (`SENSOR_VALUE_HEX2`)
    Hex2 = 1,
}

/// Sensor behaviour, bound from the configured type
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// `software_timer`: seconds since the first reading
    SoftwareTimer,
    /// `eps_voltage`: battery, bus or solar voltage
    EpsVoltage,
    /// `eps_current`: battery discharge, Jetson or solar current
    EpsCurrent,
    /// `eps_temperature`: EPS board temperature
    EpsTemperature,
    /// `payload_temperature`: payload heat spreader (OBC thermistor)
    PayloadTemperature,
    /// `status_hex2`: the status bitmap as two hex digits
    StatusHex2,
}

impl Kind {
    /// The kind a type string names
    pub fn from_type(ty: &[u8]) -> Option<Kind> {
        match ty {
            b"software_timer" => Some(Kind::SoftwareTimer),
            b"eps_voltage" => Some(Kind::EpsVoltage),
            b"eps_current" => Some(Kind::EpsCurrent),
            b"eps_temperature" => Some(Kind::EpsTemperature),
            b"payload_temperature" => Some(Kind::PayloadTemperature),
            b"status_hex2" => Some(Kind::StatusHex2),
            _ => None,
        }
    }

    /// Its reading's type
    pub fn value_type(self) -> ValueType {
        match self {
            Kind::StatusHex2 => ValueType::Hex2,
            _ => ValueType::Numeric,
        }
    }
}

/// What the simulated readings depend on
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Environment {
    /// Spacecraft sunlit
    pub sunlit: bool,
    /// State of charge (0..1)
    pub soc: f64,
    /// Jetson payload powered
    pub payload_enabled: bool,
}

/// A condition a reading reports to the fault log
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alarm {
    /// Battery voltage below 7 V
    LowBatteryVoltage,
    /// Battery discharge current above 1 A
    HighDischargeCurrent,
}

/// A numeric reading: the value, and an alarm to log
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Numeric {
    /// The value
    pub value: f64,
    /// A condition to log
    pub alarm: Option<Alarm>,
}

impl Numeric {
    fn plain(value: f64) -> Numeric {
        Numeric { value, alarm: None }
    }
}

/// A voltage reading on `channel`; `noise` gives a uniform value in [0, 1]
pub fn eps_voltage(channel: &[u8], env: &Environment, noise: &mut impl FnMut() -> f64) -> Numeric {
    let mut alarm = None;
    let base = match channel {
        b"battery" => {
            let base = 8.1 + 0.15 * (env.soc - 0.5);
            if base < 7.0 {
                alarm = Some(Alarm::LowBatteryVoltage);
            }
            base
        }
        // The range check is FDIR on the measured value, not here
        b"bus" => 5.0,
        b"solar" => {
            if env.sunlit {
                7.5
            } else {
                0.2
            }
        }
        _ => 0.0,
    };
    Numeric { value: base + 0.02 * (noise() - 0.5), alarm }
}

/// A current reading on `channel`
pub fn eps_current(channel: &[u8], env: &Environment, noise: &mut impl FnMut() -> f64) -> Numeric {
    match channel {
        b"battery_discharge" => {
            let val = if env.sunlit {
                0.05
            } else if env.payload_enabled {
                0.8
            } else {
                0.25
            };
            Numeric { value: val, alarm: (val > 1.0).then_some(Alarm::HighDischargeCurrent) }
        }
        b"jetson" => Numeric::plain(if env.payload_enabled { 0.7 + 0.05 * (noise() - 0.5) } else { 0.0 }),
        b"solar" => Numeric::plain(if env.sunlit { 0.6 + 0.1 * (noise() - 0.5) } else { 0.0 }),
        _ => Numeric::plain(0.0),
    }
}

/// EPS board temperature, C (limits are checked on the measured value)
pub fn eps_temperature(env: &Environment, noise: &mut impl FnMut() -> f64) -> f64 {
    let mut base = if env.sunlit { 22.0 } else { 14.0 };
    if env.payload_enabled {
        base += 3.0;
    }
    base + 0.5 * (noise() - 0.5)
}

/// Payload heat spreader temperature, C: warms with the Jetson on
pub fn payload_temperature(env: &Environment, noise: &mut impl FnMut() -> f64) -> f64 {
    let mut base = if env.sunlit { 22.0 } else { 14.0 };
    if env.payload_enabled {
        base += 20.0;
    }
    base + 0.5 * (noise() - 0.5)
}

/// Seconds elapsed since `start_ms` (C unsigned arithmetic)
pub fn elapsed_s(start_ms: u64, now_ms: u64) -> f64 {
    (now_ms.wrapping_sub(start_ms) / 1000) as f64
}

/// What the status bitmap reports
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatusInputs {
    /// Beacon powered
    pub beacon_enabled: bool,
    /// Jetson payload powered
    pub payload_enabled: bool,
    /// Current time, ms
    pub now_ms: u64,
    /// Spacecraft sunlit
    pub sunlit: bool,
    /// State of charge
    pub soc: f64,
    /// Power mode is SAFE
    pub safe_mode: bool,
    /// The fault log holds entries
    pub faults: bool,
    /// The watchdog fired
    pub watchdog: bool,
}

/// The status bitmap, LSB first: OBC on, beacon on, Jetson enabled, thermal
/// hot (simulated, late in each sunlit 300 s), battery low (SOC < 0.30),
/// SAFE mode, fault logged, recovery (watchdog)
pub fn status_bits(i: &StatusInputs) -> u8 {
    let mut bits = 1u8;
    if i.beacon_enabled {
        bits |= 1 << 1;
    }
    if i.payload_enabled {
        bits |= 1 << 2;
    }
    if i.sunlit && (i.now_ms / 1000) % 300 > 240 {
        bits |= 1 << 3;
    }
    if i.soc < 0.30 {
        bits |= 1 << 4;
    }
    if i.safe_mode {
        bits |= 1 << 5;
    }
    if i.faults {
        bits |= 1 << 6;
    }
    if i.watchdog {
        bits |= 1 << 7;
    }
    bits
}

/// The bitmap as `"%02X"`, NUL-terminated
pub fn hex2(bits: u8) -> [u8; TEXT_LEN] {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let digit = |n: u8| HEX.get(usize::from(n & 0xF)).copied().unwrap_or(b'0');
    [digit(bits >> 4), digit(bits), 0, 0, 0, 0, 0, 0]
}

/// `strtoul(s, NULL, 10)` with the platform's `unsigned long`, cast to
/// `uint32_t`: leading white space, an optional sign, decimal digits;
/// out of range gives `ULONG_MAX`, a minus sign negates
fn strtoul_u32(s: &[u8]) -> u32 {
    let mut rest = s;
    while let [b' ' | b'\t' | b'\n' | 0x0B | 0x0C | b'\r', tail @ ..] = rest {
        rest = tail;
    }
    let negative = match rest {
        [b'-', tail @ ..] => {
            rest = tail;
            true
        }
        [b'+', tail @ ..] => {
            rest = tail;
            false
        }
        _ => false,
    };
    let mut value: c_ulong = 0;
    let mut overflow = false;
    while let [c @ b'0'..=b'9', tail @ ..] = rest {
        match value.checked_mul(10).and_then(|v| v.checked_add(c_ulong::from(c - b'0'))) {
            Some(v) if !overflow => value = v,
            _ => overflow = true,
        }
        rest = tail;
    }
    let result = if overflow {
        c_ulong::MAX
    } else if negative {
        value.wrapping_neg()
    } else {
        value
    };
    // The C cast to uint32_t keeps the low 32 bits
    #[allow(clippy::cast_possible_truncation)]
    let low = result as u32;
    low
}

/// Bytes `fgets` returns at most per call (`char line[256]`)
pub const LINE_CHUNK: usize = 255;
/// Key bytes kept (`char key[64]`)
const KEY_MAX: usize = 63;
/// Value bytes kept (`char val[128]`)
const VAL_MAX: usize = 127;

/// The configuration file parser. Feed it the file's bytes, then
/// [`YamlParser::finish`]; each completed list item is passed to the sink.
pub struct YamlParser {
    line: [u8; LINE_CHUNK],
    len: usize,
    in_list: bool,
    have_item: bool,
    cur: SensorDef,
}

impl Default for YamlParser {
    fn default() -> Self {
        YamlParser::new()
    }
}

impl YamlParser {
    /// Nothing read yet
    pub const fn new() -> YamlParser {
        YamlParser { line: [0; LINE_CHUNK], len: 0, in_list: false, have_item: false, cur: SensorDef::EMPTY }
    }

    /// Feed bytes of the file
    pub fn feed(&mut self, bytes: &[u8], sink: &mut impl FnMut(&SensorDef)) {
        for &b in bytes {
            if let Some(slot) = self.line.get_mut(self.len) {
                *slot = b;
                self.len += 1;
            }
            // fgets returns at a newline or when its buffer is full
            if b == b'\n' || self.len >= LINE_CHUNK {
                self.end_chunk(sink);
            }
        }
    }

    /// End of file: the last partial line and the last item
    pub fn finish(mut self, sink: &mut impl FnMut(&SensorDef)) {
        if self.len > 0 {
            self.end_chunk(sink);
        }
        if self.have_item {
            sink(&self.cur);
        }
    }

    fn end_chunk(&mut self, sink: &mut impl FnMut(&SensorDef)) {
        let chunk = self.line;
        let len = self.len;
        self.len = 0;
        // A C string: up to the first NUL
        self.line_of(until_nul(chunk.get(..len).unwrap_or(&[])), sink);
    }

    /// One line as the C loader handled it
    fn line_of(&mut self, raw: &[u8], sink: &mut impl FnMut(&SensorDef)) {
        let line = trim(raw);
        if line.is_empty() || line.first() == Some(&b'#') {
            return;
        }
        if !self.in_list {
            if line == b"sensors:" {
                self.in_list = true;
            }
            return;
        }
        let line = match line {
            b"-" => Some(&b""[..]),
            [b'-', b' ', rest @ ..] => Some(trim(rest)),
            _ => None,
        }
        .map_or(line, |field| {
            // A new list item: the previous one is complete
            if self.have_item {
                sink(&self.cur);
                self.cur = SensorDef::EMPTY;
            }
            self.have_item = true;
            field
        });
        if let Some((key, val)) = key_value(line).filter(|(key, _)| !key.is_empty()) {
            self.cur.set_field(key, val);
        }
    }
}

/// Strip trailing CR, LF, space and tab, then leading space and tab
fn trim(s: &[u8]) -> &[u8] {
    let mut s = s;
    while let [rest @ .., b'\n' | b'\r' | b' ' | b'\t'] = s {
        s = rest;
    }
    while let [b' ' | b'\t', rest @ ..] = s {
        s = rest;
    }
    s
}

/// `key: value` at the first colon (key cut to 63 bytes, value to 127,
/// both trimmed, one pair of surrounding quotes removed); `None` without a colon
fn key_value(line: &[u8]) -> Option<(&[u8], &[u8])> {
    let colon = line.iter().position(|b| *b == b':')?;
    let key = line.get(..colon)?;
    let key = trim(key.get(..key.len().min(KEY_MAX))?);
    let mut val = line.get(colon + 1..)?;
    while let [b' ' | b'\t', rest @ ..] = val {
        val = rest;
    }
    let val = trim(val.get(..val.len().min(VAL_MAX))?);
    let val = match val {
        [b'"' | b'\'', inner @ .., b'"' | b'\''] => inner,
        _ => val,
    };
    Some((key, val))
}

/// A writer with `snprintf` semantics: writes what fits before the final
/// NUL, counts everything
pub struct Snprintf<'a> {
    buf: &'a mut [u8],
    written: usize,
    total: usize,
}

impl<'a> Snprintf<'a> {
    /// Write into `buf`
    pub fn new(buf: &'a mut [u8]) -> Snprintf<'a> {
        Snprintf { buf, written: 0, total: 0 }
    }

    /// Append bytes
    pub fn bytes(&mut self, data: &[u8]) {
        for &b in data {
            let room = self.written + 1 < self.buf.len();
            if let Some(slot) = self.buf.get_mut(self.written).filter(|_| room) {
                *slot = b;
                self.written += 1;
            }
            self.total = self.total.saturating_add(1);
        }
    }

    /// Append `%.3f`
    pub fn fixed3(&mut self, v: f64) {
        if v.is_nan() {
            self.bytes(if v.is_sign_negative() { b"-nan" } else { b"nan" });
        } else {
            let _ = write!(self, "{}", crate::fixed::Fixed::new(v, 3));
        }
    }

    /// Write the NUL (if there is room for anything); the length the whole
    /// output had, as `snprintf` returns it
    pub fn finish(self) -> usize {
        if let Some(nul) = self.buf.get_mut(self.written) {
            *nul = 0;
        }
        self.total
    }
}

impl Write for Snprintf<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.bytes(s.as_bytes());
        Ok(())
    }
}

/// A sensor's latest reading, for the telemetry text
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value<'a> {
    /// `id=<value>.3f<units>`
    Numeric(f64),
    /// `id=<text><units>`
    Text(&'a [u8]),
}

/// One `id=value units,` entry per sensor, as many as fit in `buf` with a
/// NUL; the length written (without the NUL). An entry that does not fit
/// ends the text; like `snprintf`, its start is written up to the end of
/// the buffer.
pub fn format_telemetry<'a>(entries: impl Iterator<Item = (&'a [u8], Value<'a>, &'a [u8])>, buf: &mut [u8]) -> usize {
    let mut offset = 0;
    for (id, value, units) in entries {
        let Some(rest) = buf.get_mut(offset..) else { break };
        let room = rest.len();
        let mut w = Snprintf::new(rest);
        w.bytes(until_nul(id));
        w.bytes(b"=");
        match value {
            Value::Numeric(v) => w.fixed3(v),
            Value::Text(t) => w.bytes(until_nul(t)),
        }
        w.bytes(until_nul(units));
        w.bytes(b",");
        let n = w.finish();
        if n >= room {
            break;
        }
        offset += n;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &[u8]) -> ([SensorDef; 40], usize) {
        let mut out = [SensorDef::EMPTY; 40];
        let mut n = 0;
        let mut p = YamlParser::new();
        let mut sink = |d: &SensorDef| {
            out[n] = *d;
            n += 1;
        };
        p.feed(text, &mut sink);
        p.finish(&mut sink);
        (out, n)
    }

    #[test]
    fn shipped_layout_keeps_ids() {
        let (defs, n) = parse(
            b"# comment\nsensors:\n  - id: BV\n    name: Battery Voltage\n    type: eps_voltage\n    channel: battery\n    \
units: V\n    period_ms: 1000\n  - id: BT\n    name: \"Battery Temperature\"\n    type: eps_temperature\n    period_ms: 2000",
        );
        assert_eq!(n, 2);
        assert_eq!(until_nul(&defs[0].id), b"BV");
        assert_eq!(until_nul(&defs[0].channel), b"battery");
        assert_eq!(defs[0].kind(), Some(Kind::EpsVoltage));
        assert_eq!(until_nul(&defs[1].name), b"Battery Temperature");
        assert_eq!(defs[1].period_ms, 2000);
    }

    #[test]
    fn lines_before_the_list_and_bare_items() {
        let (defs, n) = parse(b"id: X\nsensors:\n-\n  id: A\n- id: B\n-\tid: C\n");
        // "-\tid: C" is not an item marker, and its key is "-\tid": ignored
        assert_eq!(n, 2);
        assert_eq!(until_nul(&defs[0].id), b"A");
        assert_eq!(until_nul(&defs[1].id), b"B");
    }

    #[test]
    fn fields_are_cut_and_keep_old_bytes_after_the_nul() {
        let (defs, n) = parse(b"sensors:\n - id: ABCDEFGHIJ\n   id: X\n   units: '12345678901234567890'\n");
        assert_eq!(n, 1);
        assert_eq!(&defs[0].id, b"X\0CDEFG\0");
        assert_eq!(until_nul(&defs[0].units), b"123456789012345");
    }

    #[test]
    fn long_lines_split_like_fgets() {
        let mut text = [b' '; 300];
        text[..9].copy_from_slice(b"sensors:\n");
        text[9..15].copy_from_slice(b"- id:A");
        // 255 bytes after the item line start: "name: N" begins a new chunk
        let at = 9 + LINE_CHUNK;
        text[at..at + 7].copy_from_slice(b"name: N");
        text[299] = b'\n';
        let (defs, n) = parse(&text);
        assert_eq!(n, 1);
        assert_eq!(until_nul(&defs[0].id), b"A");
        assert_eq!(until_nul(&defs[0].name), b"N");
    }

    #[test]
    fn nul_ends_a_line() {
        let (defs, n) = parse(b"sensors:\n- id: A\0B: c\n  type: x\0y\n");
        assert_eq!(n, 1);
        assert_eq!(until_nul(&defs[0].id), b"A");
        assert_eq!(until_nul(&defs[0].type_), b"x");
    }

    #[test]
    fn quotes_and_colons() {
        assert_eq!(key_value(b"name: \"a: b\""), Some((&b"name"[..], &b"a: b"[..])));
        assert_eq!(key_value(b"name: '\""), Some((&b"name"[..], &b""[..])));
        assert_eq!(key_value(b"name: \""), Some((&b"name"[..], &b"\""[..])));
        assert_eq!(key_value(b"no colon"), None);
        assert_eq!(key_value(b"  k  :   v  "), Some((&b"k"[..], &b"v"[..])));
    }

    #[test]
    fn strtoul_like_c() {
        assert_eq!(strtoul_u32(b"1000"), 1000);
        assert_eq!(strtoul_u32(b"12ab"), 12);
        assert_eq!(strtoul_u32(b"x"), 0);
        assert_eq!(strtoul_u32(b"+7"), 7);
        assert_eq!(strtoul_u32(b"\x0b 5"), 5);
        assert_eq!(strtoul_u32(b"-1"), u32::MAX);
        assert_eq!(strtoul_u32(b"-5"), 5u32.wrapping_neg());
        assert_eq!(strtoul_u32(b"99999999999999999999999"), u32::MAX);
        if c_ulong::BITS == 64 {
            assert_eq!(strtoul_u32(b"4294967296"), 0);
            assert_eq!(strtoul_u32(b"4294967297"), 1);
        }
    }

    #[test]
    fn kinds_and_defaults() {
        let d = defaults();
        assert!(d.iter().all(|s| s.kind().is_some()));
        assert_eq!(d[9].kind().map(Kind::value_type), Some(ValueType::Hex2));
        assert_eq!(Kind::from_type(b"thermistor"), None);
        assert_eq!(until_nul(&d[6].name), b"Payload Heat Spreader Temperature");
    }

    #[test]
    fn readings() {
        let env = Environment { sunlit: true, soc: 0.5, payload_enabled: false };
        let mut half = || 0.5;
        assert_eq!(eps_voltage(b"battery", &env, &mut half), Numeric::plain(8.1));
        assert_eq!(eps_voltage(b"bus", &env, &mut half).value, 5.0);
        assert_eq!(eps_voltage(b"solar", &env, &mut half).value, 7.5);
        assert_eq!(eps_voltage(b"other", &env, &mut half).value, 0.0);
        let low = Environment { soc: -10.0, ..env };
        assert_eq!(eps_voltage(b"battery", &low, &mut half).alarm, Some(Alarm::LowBatteryVoltage));
        let dark = Environment { sunlit: false, payload_enabled: true, ..env };
        assert_eq!(eps_current(b"battery_discharge", &dark, &mut half).value, 0.8);
        assert_eq!(eps_current(b"jetson", &dark, &mut half).value, 0.7);
        assert_eq!(eps_current(b"solar", &dark, &mut half).value, 0.0);
        assert_eq!(payload_temperature(&dark, &mut half), 34.0);
        assert_eq!(eps_temperature(&dark, &mut half), 17.0);
        assert_eq!(elapsed_s(1000, 4999), 3.0);
    }

    #[test]
    fn status_bitmap() {
        let mut i = StatusInputs {
            beacon_enabled: true,
            payload_enabled: false,
            now_ms: 0,
            sunlit: true,
            soc: 0.75,
            safe_mode: false,
            faults: false,
            watchdog: false,
        };
        assert_eq!(status_bits(&i), 0x03);
        i.now_ms = 241_000;
        assert_eq!(status_bits(&i), 0x0B);
        i = StatusInputs { soc: 0.2, safe_mode: true, faults: true, watchdog: true, payload_enabled: true, ..i };
        assert_eq!(status_bits(&i), 0xFF);
        assert_eq!(&hex2(0xA5)[..3], b"A5\0");
    }

    #[test]
    fn telemetry_text_like_snprintf() {
        let entries = [
            (&b"SET\0"[..], Value::Numeric(0.0), &b"s\0"[..]),
            (&b"BV"[..], Value::Numeric(8.1004), &b"V"[..]),
            (&b"ST"[..], Value::Text(b"0B\0\0"), &b"hex"[..]),
        ];
        let mut buf = [b'Z'; 64];
        let n = format_telemetry(entries.iter().copied(), &mut buf);
        assert_eq!(&buf[..n], b"SET=0.000s,BV=8.100V,ST=0Bhex,");
        assert_eq!(buf[n], 0);

        // Truncated: the entry that does not fit is cut at the end of the buffer
        let mut small = [b'Z'; 24];
        let n = format_telemetry(entries.iter().copied(), &mut small);
        assert_eq!(n, 21);
        assert_eq!(&small[..24], b"SET=0.000s,BV=8.100V,ST\0");

        let mut w = Snprintf::new(&mut buf);
        w.fixed3(f64::NAN);
        w.fixed3(-0.0001);
        w.fixed3(0.0625);
        assert_eq!(w.finish(), 14);
        assert_eq!(&buf[..15], b"nan-0.0000.062\0");
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    /// Mutation check: the noise terms' sign and scale (noise 1.0 is the
    /// top of each band), the solar current, the temperature offsets and
    /// the alarm thresholds
    #[test]
    fn reading_models() {
        let env = Environment { sunlit: true, soc: 0.5, payload_enabled: true };
        let mut top = || 1.0;
        assert!(close(eps_voltage(b"battery", &env, &mut top).value, 8.11));
        assert!(close(eps_voltage(b"bus", &env, &mut top).value, 5.01));
        assert!(close(eps_current(b"jetson", &env, &mut top).value, 0.725));
        assert!(close(eps_current(b"solar", &env, &mut top).value, 0.65));
        let dark = Environment { sunlit: false, payload_enabled: false, ..env };
        assert_eq!(eps_current(b"solar", &dark, &mut top).value, 0.0);
        assert_eq!(eps_current(b"jetson", &dark, &mut top).value, 0.0);
        assert!(close(eps_temperature(&env, &mut top), 25.25));
        assert!(close(payload_temperature(&env, &mut top), 42.25));
        assert!(close(eps_temperature(&dark, &mut top), 14.25));
        assert!(close(payload_temperature(&dark, &mut top), 14.25));
        // No discharge current reaches the 1 A alarm
        for e in [env, dark, Environment { sunlit: false, ..env }] {
            assert_eq!(eps_current(b"battery_discharge", &e, &mut top).alarm, None);
        }
        assert_eq!(eps_current(b"battery_discharge", &dark, &mut top).value, 0.25);
        // Exactly 7.0 V is not below the alarm level
        let edge = Environment { soc: -6.833_333_333_333_334, ..env };
        let mut half = || 0.5;
        let v = eps_voltage(b"battery", &edge, &mut half);
        assert_eq!((v.value, v.alarm), (7.0, None));
    }

    #[test]
    fn status_thresholds_are_strict() {
        let i = StatusInputs {
            beacon_enabled: false,
            payload_enabled: false,
            now_ms: 240_999,
            sunlit: true,
            soc: 0.30,
            safe_mode: false,
            faults: false,
            watchdog: false,
        };
        assert_eq!(status_bits(&i), 0x01, "240 s is not late; SOC 0.30 is not low");
        assert_eq!(status_bits(&StatusInputs { soc: 0.299, now_ms: 299_999, ..i }), 0x19);
    }

    #[test]
    fn items_before_the_list_are_ignored() {
        let (defs, n) = parse(b"- id: X\n  name: early\nsensors:\n- id: A\n");
        assert_eq!(n, 1);
        assert_eq!(until_nul(&defs[0].id), b"A");
        assert_eq!(until_nul(&defs[0].name), b"");
    }
}
