//! Binary telemetry frames (`telemetry.h`): a packed 14-byte header, a
//! payload (housekeeping, EPS, ADCS or a 4-byte status beacon) and a CRC-32
//! over header and payload, plus the downlink schedule and statistics.
//!
//! Frames are built as bytes in the C `TlmFrame_t` layout (packed, fields
//! in the machine's byte order as the C struct holds them); serialized, the
//! CRC trailer is big-endian.

use sqso_protocol::checksum::Crc32Digest;

use crate::fmath::lround;

/// Frame size, bytes (`TLM_MAX_FRAME_SIZE`)
pub const MAX_FRAME_SIZE: usize = 256;
/// Header bytes (`sizeof(TlmHeader_t)`)
pub const HEADER_SIZE: usize = 14;
/// Payload bytes a frame holds
pub const PAYLOAD_MAX: usize = MAX_FRAME_SIZE - HEADER_SIZE - 4;
/// Offset of the CRC field in `TlmFrame_t`
pub const CRC_OFFSET: usize = HEADER_SIZE + PAYLOAD_MAX;
/// Sync pattern (`TLM_SYNC_WORD`)
pub const SYNC_WORD: u32 = 0x1ACF_FC1D;
/// Frame version
pub const VERSION: u8 = 1;
/// Default rate, ms
pub const DEFAULT_RATE_MS: u32 = 60_000;
/// Fastest rate, ms
pub const MIN_RATE_MS: u32 = 10_000;
/// Slowest rate, ms
pub const MAX_RATE_MS: u32 = 300_000;

/// Housekeeping payload bytes (`sizeof(TlmHousekeeping_t)`)
pub const HOUSEKEEPING_SIZE: usize = 26;
/// EPS payload bytes (`sizeof(TlmEps_t)`)
pub const EPS_SIZE: usize = 14;
/// ADCS payload bytes (`sizeof(TlmAdcs_t)`)
pub const ADCS_SIZE: usize = 20;
/// Status beacon payload bytes
pub const BEACON_SIZE: usize = 4;

/// `TlmType_t`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    /// Housekeeping
    Housekeeping = 0x01,
    /// Beacon
    Beacon = 0x04,
    /// ADCS
    Adcs = 0x05,
    /// EPS
    Eps = 0x06,
}

/// `TlmStats_t`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Frames generated
    pub frames_generated: u32,
    /// Frames transmitted
    pub frames_transmitted: u32,
    /// Transmission failures
    pub frames_failed: u32,
    /// Last transmission, ms (low 32 bits)
    pub last_tx_time_ms: u32,
    /// Sequence number of the next frame
    pub sequence_number: u16,
}

/// The system state a frame reports (read through `system_state.h`; power
/// mode and SOC are the EPS's)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Inputs {
    /// Battery voltage, V
    pub battery_voltage: f64,
    /// Battery current, A
    pub battery_current: f64,
    /// State of charge, 0-1
    pub state_of_charge: f64,
    /// Solar power, W
    pub solar_power: f64,
    /// `PowerMode_t`
    pub power_mode: u32,
    /// Payload enabled
    pub payload_enabled: bool,
    /// OBC, EPS, battery, payload temperatures, C
    pub temps_c: [f32; 4],
    /// Heater on
    pub heater_enabled: bool,
    /// `SmState_t`
    pub op_state: u32,
    /// Thermal fault
    pub thermal_fault: bool,
    /// Boot count
    pub boot_count: u32,
    /// Uptime, s
    pub uptime_s: u32,
    /// Packets sent
    pub packets_sent: u32,
    /// Packets received
    pub packets_received: u32,
    /// Beacons sent
    pub beacon_count: u32,
    /// Magnetometer, uT
    pub mag_ut: [f32; 3],
    /// Gyroscope, deg/s
    pub gyro_dps: [f32; 3],
    /// Sun vector
    pub sun_vector: [f32; 3],
    /// Detumbled
    pub detumbled: bool,
    /// Sun acquired
    pub sun_acquired: bool,
}

/// C `(uint16_t)lround(v)`: rounded, then the low 16 bits
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // modular, as the C conversion
fn round_u16(v: f64) -> u16 {
    lround(v) as u16
}

/// C `(int16_t)lround(v)`
#[allow(clippy::cast_possible_truncation)] // modular, as the C conversion
fn round_i16(v: f64) -> i16 {
    lround(v) as i16
}

/// C `(uint8_t)lround(v)`
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // modular, as the C conversion
fn round_u8(v: f64) -> u8 {
    lround(v) as u8
}

/// The low byte (C `(uint8_t)` of an unsigned value)
#[allow(clippy::cast_possible_truncation)] // the low byte, as the C conversion
fn low8(v: u32) -> u8 {
    v as u8
}

/// The low 16 bits (C `(uint16_t)` of an unsigned value)
#[allow(clippy::cast_possible_truncation)] // the low half, as the C conversion
fn low16(v: u32) -> u16 {
    v as u16
}

/// C float to integer conversion: toward zero (out of range saturates,
/// where C is undefined)
#[allow(clippy::cast_possible_truncation)] // saturating by definition of `as`
fn temp_i8(v: f32) -> u8 {
    (v as i8).to_ne_bytes()[0]
}

/// Housekeeping payload (`TlmHousekeeping_t`)
pub fn housekeeping(i: &Inputs) -> [u8; HOUSEKEEPING_SIZE] {
    let mut out = [0u8; HOUSEKEEPING_SIZE];
    let mut w = Put(&mut out, 0);
    // Power (rounded: truncation sent values exact in the unit one unit low)
    w.bytes(&round_u16(i.battery_voltage * 1000.0).to_ne_bytes());
    w.bytes(&round_i16(i.battery_current * 1000.0).to_ne_bytes());
    w.bytes(&[round_u8(i.state_of_charge * 100.0), low8(i.power_mode)]);
    // Thermal
    for t in i.temps_c {
        w.bytes(&[temp_i8(t)]);
    }
    // Status
    w.bytes(&[low8(i.op_state), u8::from(i.thermal_fault)]);
    w.bytes(&low16(i.boot_count).to_ne_bytes());
    w.bytes(&i.uptime_s.to_ne_bytes());
    // Communications
    w.bytes(&low16(i.packets_sent).to_ne_bytes());
    w.bytes(&low16(i.packets_received).to_ne_bytes());
    w.bytes(&low16(i.beacon_count).to_ne_bytes());
    // ADCS: no ADCS mode source yet
    w.bytes(&[0, u8::from(i.detumbled)]);
    out
}

/// EPS payload (`TlmEps_t`)
pub fn eps(i: &Inputs) -> [u8; EPS_SIZE] {
    let mut out = [0u8; EPS_SIZE];
    let mut w = Put(&mut out, 0);
    w.bytes(&round_u16(i.battery_voltage * 1000.0).to_ne_bytes());
    w.bytes(&round_i16(i.battery_current * 1000.0).to_ne_bytes());
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // C (uint16_t) of a double
    let solar = (i.solar_power * 100.0) as u16; // simplified
    w.bytes(&solar.to_ne_bytes());
    w.bytes(&0i16.to_ne_bytes()); // solar current: not measured
    let [_, eps_pcb, battery, _] = i.temps_c;
    w.bytes(&[
        round_u8(i.state_of_charge * 100.0),
        low8(i.power_mode),
        u8::from(i.heater_enabled),
        u8::from(i.payload_enabled),
        temp_i8(battery),
        temp_i8(eps_pcb),
    ]);
    out
}

/// ADCS payload (`TlmAdcs_t`): field 0.1 uT, rate 0.1 deg/s, sun 0.01
pub fn adcs(i: &Inputs) -> [u8; ADCS_SIZE] {
    #[allow(clippy::cast_possible_truncation)] // C (int16_t) of a float
    fn scaled(v: f32, k: f32) -> [u8; 2] {
        ((v * k) as i16).to_ne_bytes()
    }
    let mut out = [0u8; ADCS_SIZE];
    let mut w = Put(&mut out, 0);
    for v in i.mag_ut.into_iter().chain(i.gyro_dps) {
        w.bytes(&scaled(v, 10.0));
    }
    for v in i.sun_vector {
        w.bytes(&scaled(v, 100.0));
    }
    let status = u8::from(i.detumbled) | (u8::from(i.sun_acquired) << 1);
    w.bytes(&[0, status]);
    out
}

/// Status beacon payload: operational state, SOC (%), power mode, thermal fault
pub fn beacon(i: &Inputs) -> [u8; BEACON_SIZE] {
    [low8(i.op_state), round_u8(i.state_of_charge * 100.0), low8(i.power_mode), u8::from(i.thermal_fault)]
}

/// Sequential writer into a byte array
struct Put<'a, const N: usize>(&'a mut [u8; N], usize);

impl<const N: usize> Put<'_, N> {
    fn bytes(&mut self, b: &[u8]) {
        if let Some(dst) = self.0.get_mut(self.1..self.1 + b.len()) {
            dst.copy_from_slice(b);
        }
        self.1 += b.len();
    }
}

/// `TlmHeader_t` bytes
pub fn header(frame_type: FrameType, sequence: u16, timestamp_s: u32, data_len: u16) -> [u8; HEADER_SIZE] {
    let mut out = [0u8; HEADER_SIZE];
    let mut w = Put(&mut out, 0);
    w.bytes(&SYNC_WORD.to_ne_bytes());
    w.bytes(&[VERSION, frame_type as u8]);
    w.bytes(&sequence.to_ne_bytes());
    w.bytes(&timestamp_s.to_ne_bytes());
    w.bytes(&data_len.to_ne_bytes());
    out
}

/// The frame CRC: CRC-32 over header and payload
pub fn frame_crc(header: &[u8; HEADER_SIZE], payload: &[u8]) -> u32 {
    let mut d = Crc32Digest::new();
    d.update(header);
    d.update(payload);
    d.finalize()
}

/// Why serialization failed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoRoom;

/// Serialize a frame (`TlmFrame_t` bytes): header, `payload_len` payload
/// bytes, then the CRC big-endian. The payload is at most the frame's
/// [`PAYLOAD_MAX`] bytes and `out` must hold the whole frame
pub fn serialize(frame: &[u8; MAX_FRAME_SIZE], payload_len: usize, out: &mut [u8]) -> Result<usize, NoRoom> {
    if payload_len > PAYLOAD_MAX {
        return Err(NoRoom);
    }
    let total = HEADER_SIZE + payload_len + 4;
    let (Some(dst), Some(src)) = (out.get_mut(..total), frame.get(..HEADER_SIZE + payload_len)) else {
        return Err(NoRoom);
    };
    let (body, trailer) = dst.split_at_mut(HEADER_SIZE + payload_len);
    body.copy_from_slice(src);
    let mut crc = [0u8; 4];
    if let Some(field) = frame.get(CRC_OFFSET..CRC_OFFSET + 4) {
        crc.copy_from_slice(field);
    }
    trailer.copy_from_slice(&u32::from_ne_bytes(crc).to_be_bytes());
    Ok(total)
}

/// The downlink schedule and statistics
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Telemetry {
    /// Statistics
    pub stats: Stats,
    /// Rate, ms
    pub rate_ms: u32,
    /// Last transmission, ms
    pub last_ms: u64,
    /// Initialized
    pub initialized: bool,
}

impl Telemetry {
    /// Before `tlm_init()`
    pub const fn new() -> Telemetry {
        Telemetry {
            stats: Stats {
                frames_generated: 0,
                frames_transmitted: 0,
                frames_failed: 0,
                last_tx_time_ms: 0,
                sequence_number: 0,
            },
            rate_ms: DEFAULT_RATE_MS,
            last_ms: 0,
            initialized: false,
        }
    }

    /// `tlm_init()`: statistics zero, default rate, a frame due at once
    pub fn init(&mut self) {
        *self = Telemetry { initialized: true, ..Telemetry::new() };
    }

    /// Set the rate, held to [`MIN_RATE_MS`]..[`MAX_RATE_MS`]
    pub fn set_rate(&mut self, rate_ms: u32) {
        self.rate_ms = rate_ms.clamp(MIN_RATE_MS, MAX_RATE_MS);
    }

    /// Whether a frame is due (never before `init`)
    pub fn is_due(&self, now_ms: u64) -> bool {
        self.initialized && now_ms.wrapping_sub(self.last_ms) >= u64::from(self.rate_ms)
    }

    /// Record a transmission attempt; true if it succeeded (the caller
    /// counts the packet)
    pub fn mark_transmitted(&mut self, now_ms: u64, success: bool) -> bool {
        self.last_ms = now_ms;
        #[allow(clippy::cast_possible_truncation)] // the low 32 bits, as the C field
        {
            self.stats.last_tx_time_ms = now_ms as u32;
        }
        if success {
            self.stats.frames_transmitted = self.stats.frames_transmitted.wrapping_add(1);
        } else {
            self.stats.frames_failed = self.stats.frames_failed.wrapping_add(1);
        }
        success
    }

    /// Take the next sequence number (16-bit, wrapping)
    pub fn next_sequence(&mut self) -> u16 {
        let seq = self.stats.sequence_number;
        self.stats.sequence_number = seq.wrapping_add(1);
        seq
    }

    /// Count a generated frame
    pub fn count_frame(&mut self) {
        self.stats.frames_generated = self.stats.frames_generated.wrapping_add(1);
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Telemetry::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs() -> Inputs {
        Inputs {
            battery_voltage: 8.1,
            battery_current: -0.25,
            state_of_charge: 0.83,
            solar_power: 1.234,
            power_mode: 1,
            payload_enabled: true,
            temps_c: [21.0, 22.0, -10.0, 45.0],
            op_state: 3,
            boot_count: 3,
            uptime_s: 0x0102_0304,
            packets_sent: 2,
            packets_received: 1,
            beacon_count: 1,
            mag_ut: [12.3, -45.6, 0.0],
            gyro_dps: [1.5, -0.2, 300.0],
            detumbled: true,
            ..Inputs::default()
        }
    }

    #[test]
    fn housekeeping_fields() {
        let hk = housekeeping(&inputs());
        assert_eq!(u16::from_ne_bytes([hk[0], hk[1]]), 8100);
        assert_eq!(i16::from_ne_bytes([hk[2], hk[3]]), -250);
        assert_eq!(&hk[4..12], &[83, 1, 21, 22, (-10i8) as u8, 45, 3, 0]);
        assert_eq!(u16::from_ne_bytes([hk[12], hk[13]]), 3);
        assert_eq!(u32::from_ne_bytes([hk[14], hk[15], hk[16], hk[17]]), 0x0102_0304);
        assert_eq!(&hk[18..26], &[2, 0, 1, 0, 1, 0, 0, 1]);
    }

    #[test]
    fn eps_and_adcs_fields() {
        let e = eps(&inputs());
        assert_eq!(u16::from_ne_bytes([e[4], e[5]]), 123);
        assert_eq!(&e[6..14], &[0, 0, 83, 1, 0, 1, (-10i8) as u8, 22]);
        let a = adcs(&inputs());
        let field = |k: usize| i16::from_ne_bytes([a[2 * k], a[2 * k + 1]]);
        assert_eq!([field(0), field(1), field(2)], [123, -456, 0]);
        assert_eq!([field(3), field(4), field(5)], [15, -2, 3000]);
        assert_eq!(&a[18..20], &[0, 1]);
        assert_eq!(beacon(&inputs()), [3, 83, 1, 0]);
    }

    #[test]
    fn scaled_values_round_to_the_unit() {
        for pct in 0..=100u32 {
            let i = Inputs { state_of_charge: f64::from(pct) / 100.0, ..Inputs::default() };
            assert_eq!(beacon(&i)[1], u8::try_from(pct).unwrap());
        }
        // Out-of-range values wrap as the C conversion does
        let i = Inputs { battery_voltage: 70.0, ..Inputs::default() };
        assert_eq!(u16::from_ne_bytes([housekeeping(&i)[0], housekeeping(&i)[1]]), (70_000u32 & 0xFFFF) as u16);
    }

    #[test]
    fn header_and_crc() {
        let h = header(FrameType::Eps, 0xABCD, 7, 14);
        assert_eq!(&h[0..4], &SYNC_WORD.to_ne_bytes());
        assert_eq!(&h[4..6], &[1, 6]);
        assert_eq!(&h[6..8], &0xABCDu16.to_ne_bytes());
        let p = eps(&inputs());
        let mut all = [0u8; HEADER_SIZE + EPS_SIZE];
        all[..HEADER_SIZE].copy_from_slice(&h);
        all[HEADER_SIZE..].copy_from_slice(&p);
        assert_eq!(frame_crc(&h, &p), sqso_protocol::checksum::crc32(&all));
    }

    #[test]
    fn serialize_bounds_and_layout() {
        let mut frame = [0x55u8; MAX_FRAME_SIZE];
        frame[CRC_OFFSET..].copy_from_slice(&0x1122_3344u32.to_ne_bytes());
        let mut out = [0u8; 2 * MAX_FRAME_SIZE];
        assert_eq!(serialize(&frame, 4, &mut out), Ok(22));
        assert_eq!(&out[18..22], &[0x11, 0x22, 0x33, 0x44]);
        assert_eq!(serialize(&frame, PAYLOAD_MAX, &mut out), Ok(MAX_FRAME_SIZE));
        assert_eq!(serialize(&frame, PAYLOAD_MAX + 1, &mut out), Err(NoRoom));
        assert_eq!(serialize(&frame, usize::MAX, &mut out), Err(NoRoom));
        assert_eq!(serialize(&frame, 4, &mut out[..21]), Err(NoRoom));
        assert_eq!(serialize(&frame, 0, &mut out[..18]), Ok(18));
    }

    #[test]
    fn schedule() {
        let mut t = Telemetry::new();
        assert!(!t.is_due(1_000_000));
        t.init();
        assert!(t.is_due(1_000_000));
        t.set_rate(0);
        assert_eq!(t.rate_ms, MIN_RATE_MS);
        t.set_rate(u32::MAX);
        assert_eq!(t.rate_ms, MAX_RATE_MS);
        t.set_rate(30_000);
        assert!(t.mark_transmitted(1_000_000, true));
        assert!(!t.is_due(1_029_999));
        assert!(t.is_due(1_030_000));
        assert!(!t.mark_transmitted(1_000_000, false));
        assert_eq!((t.stats.frames_transmitted, t.stats.frames_failed), (1, 1));
        t.stats.sequence_number = u16::MAX;
        assert_eq!(t.next_sequence(), u16::MAX);
        assert_eq!(t.next_sequence(), 0);
    }
}
