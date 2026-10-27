//! KISS framing from a TNC (the 435 MHz telecommand receiver delivers
//! received AX.25 frames this way, without the FCS, which the TNC checks).
//!
//! A frame is FEND, a command byte (high nibble port, low nibble command;
//! 0 is a data frame), the escaped frame, FEND. [`Decoder`] takes bytes one
//! at a time, so it can be fed straight from a UART; it never allocates,
//! and a frame longer than its buffer is dropped whole.
//!
//! @requirement SYS-204 Cease emissions on telecommand; stay ceased across resets (97.207(b))

/// Frame end
pub const FEND: u8 = 0xC0;
/// Frame escape
pub const FESC: u8 = 0xDB;
/// Escaped FEND
pub const TFEND: u8 = 0xDC;
/// Escaped FESC
pub const TFESC: u8 = 0xDD;

/// Command nibble: a data frame
pub const CMD_DATA: u8 = 0x00;
/// Command nibble: keying delay before data, in 10 ms units
pub const CMD_TXDELAY: u8 = 0x01;
/// Command nibble: CSMA persistence (255: always transmit at once)
pub const CMD_PERSIST: u8 = 0x02;
/// Command nibble: CSMA slot time, in 10 ms units
pub const CMD_SLOTTIME: u8 = 0x03;
/// Command nibble: hold after the data, in 10 ms units
pub const CMD_TXTAIL: u8 = 0x04;
/// Command nibble: full duplex (nonzero: do not wait for a clear channel)
pub const CMD_FULLDUPLEX: u8 = 0x05;

/// The downlink TNC's keying delay, 10 ms units (300 ms, TBR with the
/// transmitter: its lock and power-settle times, SQSO-RF-TX-001 §10.2)
pub const TXDELAY_10MS: u8 = 30;
/// The downlink TNC's tail, 10 ms units
pub const TXTAIL_10MS: u8 = 1;

/// The downlink TNC's parameters, sent before every frame because the TNC
/// may have been powered off since the last one (the radio is unpowered
/// during the RF silence). Full duplex: the downlink does not share the
/// uplink channel, so the TNC must not hold off for its carrier detect; and
/// persistence 255, slot time 0, for the same reason
pub const DOWNLINK_PARAMS: [(u8, u8); 5] = [
    (CMD_TXDELAY, TXDELAY_10MS),
    (CMD_PERSIST, 255),
    (CMD_SLOTTIME, 0),
    (CMD_TXTAIL, TXTAIL_10MS),
    (CMD_FULLDUPLEX, 1),
];

/// Most bytes a parameter command takes on the wire (the value escaped)
pub const COMMAND_MAX_LEN: usize = 5;

/// The bytes on the wire for a frame of `len` bytes, at most: every byte
/// escaped, plus FEND, the command byte and FEND
pub const fn frame_max_len(len: usize) -> usize {
    2 * len + 3
}

/// A parameter command: FEND, `command` for `port`, `value` (escaped), FEND
pub fn encode_command(command: u8, value: u8, port: u8, out: &mut [u8]) -> Result<usize, crate::ax25::EncodeError> {
    let escaped: &[u8] = match value {
        FEND => &[FESC, TFEND],
        FESC => &[FESC, TFESC],
        _ => core::slice::from_ref(&value),
    };
    let len = 3 + escaped.len();
    let dst = out.get_mut(..len).ok_or(crate::ax25::EncodeError)?;
    let [head @ .., end] = dst else { return Err(crate::ax25::EncodeError) };
    let (fend_cmd, body) = head.split_at_mut(2);
    fend_cmd.copy_from_slice(&[FEND, ((port & 0x0F) << 4) | (command & 0x0F)]);
    body.copy_from_slice(escaped);
    *end = FEND;
    Ok(len)
}

/// What the decoder is doing
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// Waiting for a FEND
    Hunt,
    /// After a FEND: the command byte comes next (or another FEND)
    Command,
    /// Inside a data frame
    Data,
    /// Inside a data frame, after FESC
    Escape,
    /// Inside a frame to skip (not data, not our port, too long or badly
    /// escaped) until the next FEND
    Skip,
}

/// Byte-at-a-time KISS decoder for data frames on one port, `N` bytes at most
#[derive(Clone, Debug)]
pub struct Decoder<const N: usize> {
    port: u8,
    state: State,
    buf: [u8; N],
    len: usize,
    /// Frames dropped (too long, badly escaped)
    pub dropped: u32,
}

impl<const N: usize> Decoder<N> {
    /// A decoder for data frames on `port` (0–15)
    pub const fn new(port: u8) -> Self {
        Decoder { port: port & 0x0F, state: State::Hunt, buf: [0; N], len: 0, dropped: 0 }
    }

    /// Take one byte. Returns the frame when this byte completes one.
    pub fn feed(&mut self, byte: u8) -> Option<&[u8]> {
        match (self.state, byte) {
            (State::Data | State::Escape, FEND) => {
                let complete = self.state == State::Data && self.len > 0;
                if self.state == State::Escape {
                    self.dropped = self.dropped.saturating_add(1);
                }
                self.state = State::Command;
                let len = core::mem::replace(&mut self.len, 0);
                return if complete { self.buf.get(..len) } else { None };
            }
            (_, FEND) => self.state = State::Command,
            (State::Hunt | State::Skip, _) => {}
            (State::Command, command) => {
                self.len = 0;
                self.state = if command == self.port << 4 { State::Data } else { State::Skip };
            }
            (State::Data, FESC) => self.state = State::Escape,
            (State::Data, b) => self.push(b),
            (State::Escape, TFEND) => self.push(FEND),
            (State::Escape, TFESC) => self.push(FESC),
            (State::Escape, _) => self.drop_frame(),
        }
        None
    }

    fn push(&mut self, byte: u8) {
        match self.buf.get_mut(self.len) {
            Some(slot) => {
                *slot = byte;
                self.len += 1;
                self.state = State::Data;
            }
            None => self.drop_frame(),
        }
    }

    fn drop_frame(&mut self) {
        self.dropped = self.dropped.saturating_add(1);
        self.len = 0;
        self.state = State::Skip;
    }
}

/// Command nibble: SetHardware (the TNC's own commands)
pub const CMD_SETHARDWARE: u8 = 0x06;
/// SetHardware sub-command: the OBC's STATUS query (RF board E §8.6)
pub const HW_STATUS: u8 = 0x01;
/// SetHardware sub-command of the TNC's STATUS answer
pub const HW_STATUS_ANSWER: u8 = 0x81;
/// The STATUS query on the wire (port 0)
pub const STATUS_QUERY: [u8; 4] = [FEND, CMD_SETHARDWARE, HW_STATUS, FEND];
/// Longest SetHardware frame kept, bytes (after the command byte)
pub const HW_MAX: usize = 24;

/// The TNC's STATUS answer (TNC-FW-STAT; the layout the OBC reads, after the
/// sub-command 0x81, big-endian): firmware version; the six health criteria
/// (bit n set: criterion n+1 met); the in-channel noise floor and its
/// calibrated value, dBm (signed); the carrier-sense count; the frames
/// decoded; the uptime, s
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TncStatus {
    /// Firmware version
    pub version: u8,
    /// Health criteria met (bits 0-5)
    pub criteria: u8,
    /// Noise floor now, dBm
    pub floor_dbm: i8,
    /// Its calibrated value, dBm
    pub floor_cal_dbm: i8,
    /// Carrier-sense count (wraps)
    pub carrier_count: u16,
    /// Frames decoded (wraps)
    pub frames: u16,
    /// Uptime, s
    pub uptime_s: u32,
}

/// Bytes of the answer after the sub-command
pub const STATUS_LEN: usize = 12;

impl TncStatus {
    /// Decode a SetHardware frame's bytes (sub-command first); `None` if it
    /// is not a STATUS answer
    pub fn decode(frame: &[u8]) -> Option<TncStatus> {
        let &[HW_STATUS_ANSWER, version, criteria, floor, cal, c0, c1, f0, f1, u0, u1, u2, u3, ..] = frame else {
            return None;
        };
        Some(TncStatus {
            version,
            criteria: criteria & 0x3F,
            floor_dbm: i8::from_ne_bytes([floor]),
            floor_cal_dbm: i8::from_ne_bytes([cal]),
            carrier_count: u16::from_be_bytes([c0, c1]),
            frames: u16::from_be_bytes([f0, f1]),
            uptime_s: u32::from_be_bytes([u0, u1, u2, u3]),
        })
    }

    /// The answer's bytes after the command byte (for the simulated TNC)
    pub fn encode(&self) -> [u8; 1 + STATUS_LEN] {
        let [c0, c1] = self.carrier_count.to_be_bytes();
        let [f0, f1] = self.frames.to_be_bytes();
        let [u0, u1, u2, u3] = self.uptime_s.to_be_bytes();
        let [floor] = self.floor_dbm.to_ne_bytes();
        let [cal] = self.floor_cal_dbm.to_ne_bytes();
        [HW_STATUS_ANSWER, self.version, self.criteria, floor, cal, c0, c1, f0, f1, u0, u1, u2, u3]
    }
}

/// Byte-at-a-time decoder for the TNC's SetHardware frames on port 0 (fed
/// the same bytes as the data [`Decoder`])
#[derive(Clone, Debug, Default)]
pub struct HardwareDecoder {
    state: Option<bool>,
    escape: bool,
    buf: [u8; HW_MAX],
    len: usize,
    after_fend: bool,
}

impl HardwareDecoder {
    /// Waiting for a frame
    pub const fn new() -> Self {
        HardwareDecoder { state: None, escape: false, buf: [0; HW_MAX], len: 0, after_fend: false }
    }

    /// Take one byte; the frame's bytes (after the command byte) when this
    /// byte completes a SetHardware frame
    pub fn feed(&mut self, byte: u8) -> Option<&[u8]> {
        if byte == FEND {
            let done = self.state == Some(true) && !self.escape && self.len > 0;
            self.state = None;
            self.escape = false;
            self.after_fend = true;
            let len = core::mem::replace(&mut self.len, 0);
            return if done { self.buf.get(..len) } else { None };
        }
        if self.after_fend {
            self.after_fend = false;
            self.state = Some(byte == CMD_SETHARDWARE);
            return None;
        }
        if self.state != Some(true) {
            return None;
        }
        let b = match (self.escape, byte) {
            (false, FESC) => {
                self.escape = true;
                return None;
            }
            (true, TFEND) => FEND,
            (true, TFESC) => FESC,
            (true, _) => {
                self.state = Some(false);
                return None;
            }
            (false, b) => b,
        };
        self.escape = false;
        match self.buf.get_mut(self.len) {
            Some(slot) => {
                *slot = b;
                self.len += 1;
            }
            None => self.state = Some(false),
        }
        None
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    #[test]
    fn status_answer_round_trip_and_the_hardware_decoder() {
        let st = TncStatus {
            version: 3,
            criteria: 0x3F,
            floor_dbm: -121,
            floor_cal_dbm: -122,
            carrier_count: 0xC0DB,
            frames: 7,
            uptime_s: 86_400,
        };
        let body = st.encode();
        assert_eq!(TncStatus::decode(&body), Some(st));
        assert_eq!(TncStatus::decode(&body[..5]), None);
        assert_eq!(TncStatus::decode(&[HW_STATUS, 0]), None);
        // On the wire: FEND, 0x06, the escaped bytes, FEND, between data frames
        let mut wire = std::vec![FEND, CMD_DATA, 1, 2, FEND, FEND, CMD_SETHARDWARE];
        for b in body {
            match b {
                FEND => wire.extend([FESC, TFEND]),
                FESC => wire.extend([FESC, TFESC]),
                b => wire.push(b),
            }
        }
        wire.push(FEND);
        let mut d = HardwareDecoder::new();
        let mut got = None;
        for b in wire {
            if let Some(f) = d.feed(b) {
                got = TncStatus::decode(f);
            }
        }
        assert_eq!(got, Some(st));
        // A bad escape or an overlong frame is dropped
        let mut d = HardwareDecoder::default();
        let mut out = 0;
        for b in [FEND, CMD_SETHARDWARE, FESC, 0x00, 1, FEND] {
            out += usize::from(d.feed(b).is_some());
        }
        for b in core::iter::once(CMD_SETHARDWARE).chain(core::iter::repeat_n(1u8, HW_MAX + 1)).chain([FEND]) {
            out += usize::from(d.feed(b).is_some());
        }
        assert_eq!(out, 0);
        assert_eq!(STATUS_QUERY, [FEND, CMD_SETHARDWARE, HW_STATUS, FEND]);
    }

    use super::*;
    use crate::ax25::kiss_encode;

    fn decode_all<const N: usize>(d: &mut Decoder<N>, bytes: &[u8]) -> Option<([u8; N], usize)> {
        let mut found = None;
        for &b in bytes {
            if let Some(frame) = d.feed(b) {
                let mut out = [0; N];
                out[..frame.len()].copy_from_slice(frame);
                found = Some((out, frame.len()));
            }
        }
        found
    }

    #[test]
    fn commands_are_framed_and_escaped() {
        let mut out = [0u8; COMMAND_MAX_LEN];
        assert_eq!(encode_command(CMD_TXDELAY, 30, 0, &mut out), Ok(4));
        assert_eq!(&out[..4], &[FEND, 0x01, 30, FEND]);
        assert_eq!(encode_command(CMD_FULLDUPLEX, 1, 2, &mut out), Ok(4));
        assert_eq!(&out[..4], &[FEND, 0x25, 1, FEND]);
        assert_eq!(encode_command(CMD_PERSIST, FEND, 0, &mut out), Ok(5));
        assert_eq!(out, [FEND, 0x02, FESC, TFEND, FEND]);
        assert_eq!(encode_command(CMD_PERSIST, FESC, 0, &mut out), Ok(5));
        assert_eq!(out, [FEND, 0x02, FESC, TFESC, FEND]);
        // A command frame is never taken for data
        let mut d = Decoder::<16>::new(0);
        assert!(decode_all(&mut d, &out).is_none());
    }

    #[test]
    fn a_command_needs_room() {
        let mut out = [0u8; 4];
        assert!(encode_command(CMD_TXDELAY, 30, 0, &mut out).is_ok());
        assert!(encode_command(CMD_PERSIST, FEND, 0, &mut out).is_err());
        assert!(encode_command(CMD_TXDELAY, 30, 0, &mut out[..3]).is_err());
    }

    #[test]
    fn frame_max_len_bounds_the_encoder() {
        let frame = [FEND; 40];
        let mut wire = [0u8; 128];
        assert_eq!(kiss_encode(&frame, 0, &mut wire), Ok(frame_max_len(frame.len())));
        assert!(kiss_encode(&frame, 0, &mut wire[..frame_max_len(frame.len()) - 1]).is_err());
    }

    #[test]
    fn round_trip_with_escapes() {
        let frame = [0x01, FEND, 0x02, FESC, 0x03];
        let mut wire = [0u8; 32];
        let n = kiss_encode(&frame, 0, &mut wire).unwrap();
        let mut d = Decoder::<16>::new(0);
        let (out, len) = decode_all(&mut d, &wire[..n]).unwrap();
        assert_eq!(&out[..len], &frame);
        assert_eq!(d.dropped, 0);
    }

    #[test]
    fn noise_before_the_first_fend_is_ignored() {
        let mut d = Decoder::<16>::new(0);
        let (out, len) = decode_all(&mut d, &[0x55, 0xAA, FEND, 0x00, 0x41, FEND]).unwrap();
        assert_eq!(&out[..len], b"A");
    }

    #[test]
    fn other_ports_and_commands_are_skipped() {
        let mut d = Decoder::<16>::new(0);
        assert!(decode_all(&mut d, &[FEND, 0x10, 0x41, FEND]).is_none()); // port 1
        assert!(decode_all(&mut d, &[FEND, 0x01, 0x41, FEND]).is_none()); // TXDELAY
        let (out, len) = decode_all(&mut d, &[FEND, 0x00, 0x42, FEND]).unwrap();
        assert_eq!(&out[..len], b"B");
    }

    #[test]
    fn back_to_back_fends_make_no_empty_frames() {
        let mut d = Decoder::<16>::new(0);
        assert!(decode_all(&mut d, &[FEND, FEND, FEND, 0x00, FEND]).is_none());
    }

    #[test]
    fn too_long_or_bad_escape_drops_the_frame_only() {
        let mut d = Decoder::<4>::new(0);
        assert!(decode_all(&mut d, &[FEND, 0x00, 1, 2, 3, 4, 5, FEND]).is_none());
        assert!(decode_all(&mut d, &[0x00, FESC, 0x01, 0x41, FEND]).is_none());
        assert_eq!(d.dropped, 2);
        let (out, len) = decode_all(&mut d, &[0x00, 1, 2, 3, 4, FEND]).unwrap();
        assert_eq!(&out[..len], &[1, 2, 3, 4]);
    }

    #[test]
    fn fend_after_fesc_drops_the_frame() {
        let mut d = Decoder::<8>::new(0);
        assert!(decode_all(&mut d, &[FEND, 0x00, 0x41, FESC, FEND]).is_none());
        assert_eq!(d.dropped, 1);
        let (out, len) = decode_all(&mut d, &[0x00, 0x43, FEND]).unwrap();
        assert_eq!(&out[..len], b"C");
    }

    #[test]
    fn port_is_the_high_nibble_of_the_command_byte() {
        let mut d = Decoder::<16>::new(1);
        assert!(decode_all(&mut d, &[FEND, 0x00, 0x41, FEND]).is_none()); // port 0
        let (out, len) = decode_all(&mut d, &[FEND, 0x10, 0x42, FEND]).unwrap();
        assert_eq!(&out[..len], b"B");
        let mut d = Decoder::<16>::new(15);
        let (out, len) = decode_all(&mut d, &[FEND, 0xF0, 0x43, FEND]).unwrap();
        assert_eq!(&out[..len], b"C");
    }
}
