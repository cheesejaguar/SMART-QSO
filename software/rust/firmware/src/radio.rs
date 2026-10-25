//! `hal_radio.h`: the 435 MHz telecommand receiver. The receiver's TNC
//! hands over received AX.25 frames in KISS framing on USART2 (PD5 TX /
//! PD6 RX, 9600 baud; `hardware/obc/OBC_DESIGN.md` §8.1, TBR), FCS already
//! checked. The KISS decoder is `sqso_protocol::kiss`.

use embassy_stm32::usart::BufferedUart;
use embedded_io::{Read, ReadReady};
use sqso_protocol::kiss::Decoder;

use crate::cell::Shared;
use crate::rc;

/// `HAL_RADIO_MAX_FRAME`
const MAX_FRAME: usize = 330;

pub(crate) struct Radio {
    uart: BufferedUart<'static>,
    kiss: Decoder<MAX_FRAME>,
    ready: bool,
}

static RADIO: Shared<Radio> = Shared::new();

pub(crate) fn install(uart: BufferedUart<'static>) {
    RADIO.install(Radio { uart, kiss: Decoder::new(0), ready: false });
}

/// Start receiving
#[unsafe(no_mangle)]
pub extern "C" fn hal_radio_init() -> i32 {
    RADIO
        .with(|r| {
            r.ready = true;
            rc::OK
        })
        .unwrap_or(rc::ERROR)
}

/// The next received frame, or a timeout if none is complete
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
                if let Some(received) = r.kiss.feed(b) {
                    let Some(dst) = buf.get_mut(..received.len()) else { return rc::INVALID };
                    dst.copy_from_slice(received);
                    *out = received.len();
                    return rc::OK;
                }
            }
            rc::TIMEOUT
        })
        .unwrap_or(rc::BUSY)
}
