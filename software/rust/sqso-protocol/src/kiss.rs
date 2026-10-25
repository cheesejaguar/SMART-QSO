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

#[cfg(test)]
mod tests {
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
