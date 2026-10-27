//! `hal_radio.h`: the KISS TNC on USART2 (PD5 TX / PD6 RX, 9600 baud;
//! `hardware/obc/OBC_DESIGN.md` §8.1, TBR) that serves the 435 MHz
//! telecommand receiver and the 145.8 MHz downlink transmitter
//! (SQSO-CRL-001 CR-2026-005). The KISS framing is `sqso_protocol::kiss`.
//!
//! Receive: the TNC hands over received AX.25 frames with the FCS checked
//! and left off; the FCS of the frame is appended here, because
//! `hal_radio.h` delivers frames through their FCS and the uplink checks it
//! again (`ax25::parse_ui`).
//!
//! Transmit: the TNC's parameters, then the frame as a KISS data frame. The
//! write cannot block: the downlink (`downlink.h`) hands over one frame at a
//! time and only after the last one's worst-case UART time, so the transmit
//! buffer is empty, and it holds the longest stream ([`STREAM_MAX`],
//! checked in `main.rs`).

use embassy_stm32::usart::BufferedUart;
use embedded_io::{Read, ReadReady, Write};
use sqso_protocol::ax25;
use sqso_protocol::kiss::{self, Decoder, HardwareDecoder};

use crate::cell::Shared;
use crate::rc;

/// `HAL_RADIO_MAX_FRAME`
const MAX_FRAME: usize = 330;
/// FCS length: received frames are delivered with it, transmitted without
const FCS_LEN: usize = 2;
/// The longest transmit stream on the UART: every parameter command and
/// every frame byte escaped
pub(crate) const STREAM_MAX: usize =
    kiss::DOWNLINK_PARAMS.len() * kiss::COMMAND_MAX_LEN + kiss::frame_max_len(MAX_FRAME - FCS_LEN);
/// Escaped bytes written to the UART at a time (stack, SRS-C003)
const CHUNK: usize = 32;

pub(crate) struct Radio {
    uart: BufferedUart<'static>,
    /// Frames as the TNC gives them (no FCS)
    kiss: Decoder<{ MAX_FRAME - FCS_LEN }>,
    /// The TNC's SetHardware frames (its STATUS answers, RF board E §8.6)
    hw: HardwareDecoder,
    /// The latest STATUS answer not yet taken, and its length
    status: Option<([u8; kiss::HW_MAX], usize)>,
    ready: bool,
}

static RADIO: Shared<Radio> = Shared::new();

pub(crate) fn install(uart: BufferedUart<'static>) {
    RADIO.install(Radio { uart, kiss: Decoder::new(0), hw: HardwareDecoder::new(), status: None, ready: false });
}

/// Start the TNC link (receive and transmit)
#[unsafe(no_mangle)]
pub extern "C" fn hal_radio_init() -> i32 {
    RADIO
        .with(|r| {
            r.ready = true;
            rc::OK
        })
        .unwrap_or(rc::ERROR)
}

/// The next received frame (through its FCS), or a timeout if none is complete
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_radio_receive(frame: *mut u8, max_len: usize, len: *mut usize) -> i32 {
    // SAFETY: the C contract: frame points to max_len writable bytes, len to a size_t
    let (buf, out) = unsafe { (rc::bytes_mut(frame, max_len), rc::out(len)) };
    let (Some(buf), Some(out)) = (buf, out) else { return rc::NULL_PTR };
    *out = 0;
    RADIO
        .with(|r| {
            if !r.ready {
                return rc::ERROR;
            }
            let mut byte = [0u8];
            while r.uart.read_ready().unwrap_or(false) {
                if !matches!(r.uart.read(&mut byte), Ok(1)) {
                    break;
                }
                let [b] = byte;
                if let Some(answer) = r.hw.feed(b) {
                    let mut kept = [0u8; kiss::HW_MAX];
                    if let Some(dst) = kept.get_mut(..answer.len()) {
                        dst.copy_from_slice(answer);
                        r.status = Some((kept, answer.len()));
                    }
                }
                if let Some(received) = r.kiss.feed(b) {
                    let Some(n) = ax25::append_fcs(received, buf) else { return rc::INVALID };
                    *out = n;
                    return rc::OK;
                }
            }
            rc::TIMEOUT
        })
        .unwrap_or(rc::BUSY)
}

/// Bytes for the UART, written `CHUNK` at a time
struct Chunked<'a> {
    uart: &'a mut BufferedUart<'static>,
    buf: [u8; CHUNK],
    n: usize,
}

impl Chunked<'_> {
    fn push(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if self.n + bytes.len() > CHUNK {
            self.flush()?;
        }
        self.buf.get_mut(self.n..self.n + bytes.len()).ok_or(())?.copy_from_slice(bytes);
        self.n += bytes.len();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), ()> {
        self.uart.write_all(self.buf.get(..self.n).ok_or(())?).map_err(drop)?;
        self.n = 0;
        Ok(())
    }
}

/// The TNC's parameters, then `frame` KISS-escaped as a data frame on port 0
fn write_stream(uart: &mut BufferedUart<'static>, frame: &[u8]) -> Result<(), ()> {
    let mut out = Chunked { uart, buf: [0; CHUNK], n: 0 };
    let mut command = [0u8; kiss::COMMAND_MAX_LEN];
    for (cmd, value) in kiss::DOWNLINK_PARAMS {
        let k = kiss::encode_command(cmd, value, 0, &mut command).map_err(drop)?;
        out.push(command.get(..k).ok_or(())?)?;
    }
    out.push(&[kiss::FEND, kiss::CMD_DATA])?;
    for &b in frame {
        match b {
            kiss::FEND => out.push(&[kiss::FESC, kiss::TFEND])?,
            kiss::FESC => out.push(&[kiss::FESC, kiss::TFESC])?,
            _ => out.push(&[b])?,
        }
    }
    out.push(&[kiss::FEND])?;
    out.flush()
}

/// Send the TNC its STATUS query (KISS SetHardware 0x06/0x01)
#[unsafe(no_mangle)]
pub extern "C" fn hal_radio_status_query() -> i32 {
    RADIO
        .with(|r| {
            if !r.ready {
                return rc::ERROR;
            }
            match r.uart.write_all(&kiss::STATUS_QUERY) {
                Ok(()) => rc::OK,
                Err(_) => rc::IO,
            }
        })
        .unwrap_or(rc::BUSY)
}

/// The TNC's latest STATUS answer since the last call (the answer is
/// gathered while `hal_radio_receive` reads the UART)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_radio_take_status(data: *mut u8, max_len: usize, len: *mut usize) -> i32 {
    // SAFETY: the C contract: data points to max_len writable bytes, len to a size_t
    let (buf, out) = unsafe { (rc::bytes_mut(data, max_len), rc::out(len)) };
    let (Some(buf), Some(out)) = (buf, out) else { return rc::NULL_PTR };
    *out = 0;
    RADIO
        .with(|r| {
            let Some((kept, n)) = r.status.take() else { return rc::TIMEOUT };
            let (Some(dst), Some(src)) = (buf.get_mut(..n), kept.get(..n)) else { return rc::INVALID };
            dst.copy_from_slice(src);
            *out = n;
            rc::OK
        })
        .unwrap_or(rc::BUSY)
}

/// Hand one frame (no FCS) to the TNC: its parameters, then the frame
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hal_radio_transmit(frame: *const u8, len: usize) -> i32 {
    // SAFETY: the C contract: frame points to len bytes
    let Some(frame) = (unsafe { rc::bytes(frame, len) }) else { return rc::NULL_PTR };
    if frame.is_empty() || frame.len() > MAX_FRAME - FCS_LEN {
        return rc::INVALID;
    }
    RADIO
        .with(|r| {
            if !r.ready {
                return rc::ERROR;
            }
            match write_stream(&mut r.uart, frame) {
                Ok(()) => rc::OK,
                Err(()) => rc::IO,
            }
        })
        .unwrap_or(rc::BUSY)
}
