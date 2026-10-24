//! OBC–Jetson serial line configuration (`uart_comm.h`).
//!
//! The settings and the rules for the environment overrides of the host
//! build (`UART_DEVICE`, `UART_BAUDRATE`, `TELEMETRY_INTERVAL_MS`). The
//! port itself (termios on the host, the HAL on the target) is in
//! `sqso-ffi`.
//!
//! @requirement SRS-F030 Send health telemetry to Jetson via UART
//! @requirement SRS-F031 Receive beacon text from Jetson via UART
//! @requirement SYS-520 Detect AI payload failure or timeout (UART health check)
//! @requirement SRS-I001 UART at 115200 baud, 8N1 (the host build may override device and baud)

/// Device path bytes including the NUL (`UART_DEVICE_MAX_LEN`)
pub const DEVICE_MAX_LEN: usize = 64;
/// Default device (`UART_DEVICE_DEFAULT`)
pub const DEVICE_DEFAULT: &[u8] = b"/dev/ttyUSB0";
/// Default baud rate (`UART_BAUDRATE_DEFAULT`)
pub const BAUDRATE_DEFAULT: i32 = 115_200;
/// Default read timeout, ms
pub const TIMEOUT_DEFAULT_MS: u32 = 1000;
/// Default telemetry interval (`SMART_QSO_TELEMETRY_INTERVAL_MS`)
pub const TELEMETRY_INTERVAL_DEFAULT_MS: u32 = 60_000;
/// Longest telemetry interval accepted, ms (one hour)
pub const TELEMETRY_INTERVAL_MAX_MS: u32 = 3_600_000;
/// The baud rates the line supports
pub const BAUD_RATES: [i32; 6] = [9600, 19_200, 38_400, 57_600, 115_200, 230_400];

/// Whether `baud` is a supported rate
pub fn is_supported_baud(baud: i32) -> bool {
    BAUD_RATES.contains(&baud)
}

/// The rate the line is set to: `baud` if supported, else the default
pub fn line_baud(baud: i32) -> i32 {
    if is_supported_baud(baud) { baud } else { BAUDRATE_DEFAULT }
}

/// C `atoi` as glibc does it on a 64-bit host: leading white space, an
/// optional sign and decimal digits, read as a `long` (saturating), then
/// converted to `int` (keeping the low 32 bits)
pub fn atoi(s: &[u8]) -> i32 {
    let mut rest = s;
    while let [b' ' | b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r', tail @ ..] = rest {
        rest = tail;
    }
    let negative = matches!(rest, [b'-', ..]);
    if let [b'-' | b'+', tail @ ..] = rest {
        rest = tail;
    }
    let mut value: i64 = 0;
    for b in rest.iter().take_while(|b| b.is_ascii_digit()) {
        let digit = i64::from(b - b'0');
        value = if negative {
            value.checked_mul(10).and_then(|v| v.checked_sub(digit)).unwrap_or(i64::MIN)
        } else {
            value.checked_mul(10).and_then(|v| v.checked_add(digit)).unwrap_or(i64::MAX)
        };
    }
    let [b0, b1, b2, b3, ..] = value.to_le_bytes();
    i32::from_le_bytes([b0, b1, b2, b3])
}

/// `UART_BAUDRATE`: the rate if supported
pub fn parse_baud(text: &[u8]) -> Option<i32> {
    Some(atoi(text)).filter(|b| is_supported_baud(*b))
}

/// `TELEMETRY_INTERVAL_MS`: the interval if 1 ms to one hour
pub fn parse_interval(text: &[u8]) -> Option<u32> {
    u32::try_from(atoi(text)).ok().filter(|ms| (1..=TELEMETRY_INTERVAL_MAX_MS).contains(ms))
}

/// `UART_DEVICE`: used if it fits the field with its NUL
pub fn device_fits(path: &[u8]) -> bool {
    path.len() < DEVICE_MAX_LEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atoi_like_c() {
        assert_eq!(atoi(b"38400"), 38400);
        assert_eq!(atoi(b"  -12abc"), -12);
        assert_eq!(atoi(b"+7"), 7);
        assert_eq!(atoi(b""), 0);
        assert_eq!(atoi(b"x1"), 0);
        assert_eq!(atoi(b"\t\n 5"), 5);
        assert_eq!(atoi(b"- 5"), 0);
        // long, then int: the low 32 bits
        assert_eq!(atoi(b"4294967296000"), 0);
        assert_eq!(atoi(b"2147483648"), i32::MIN);
        assert_eq!(atoi(b"99999999999999999999999"), -1);
        assert_eq!(atoi(b"-99999999999999999999999"), 0);
    }

    #[test]
    fn baud_rates() {
        assert_eq!(parse_baud(b"38400"), Some(38400));
        assert_eq!(parse_baud(b"1234"), None);
        assert_eq!(parse_baud(b"abc"), None);
        assert_eq!(line_baud(57600), 57600);
        assert_eq!(line_baud(1234), BAUDRATE_DEFAULT);
    }

    #[test]
    fn intervals() {
        assert_eq!(parse_interval(b"5000"), Some(5000));
        assert_eq!(parse_interval(b"0"), None);
        assert_eq!(parse_interval(b"-5"), None);
        assert_eq!(parse_interval(b"3600000"), Some(3_600_000));
        assert_eq!(parse_interval(b"3600001"), None);
    }

    #[test]
    fn device_length() {
        assert!(device_fits(&[b'a'; 63]));
        assert!(!device_fits(&[b'a'; 64]));
        assert!(device_fits(DEVICE_DEFAULT));
    }
}
