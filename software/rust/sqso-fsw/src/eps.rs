//! EPS power policy (`eps_control.h`): load switches per power mode, the
//! power budget of each mode, the payload enable rule, the bus regulation
//! check, and the persisted EPS state record.
//!
//! The power modes are [`crate::mode::PowerMode`]; the payload threshold is
//! [`crate::mode::SOC_PAYLOAD`], the one the mode manager's guards use.
//!
//! @requirement SRS-F011 Control power to subsystems via load switches
//! @requirement SRS-F012 Implement power modes SAFE, IDLE, ACTIVE
//! @requirement SRS-F016 Enforce power budget limits per mode

use sqso_protocol::checksum::crc32;

use crate::mode::{PowerMode, SOC_PAYLOAD};

/// Power limit for SAFE mode, W (`EPS_POWER_LIMIT_SAFE_W`)
pub const POWER_LIMIT_SAFE_W: f64 = 0.5;
/// Power limit for IDLE mode, W (`EPS_POWER_LIMIT_IDLE_W`)
pub const POWER_LIMIT_IDLE_W: f64 = 1.5;
/// Power limit for ACTIVE mode, W (`EPS_POWER_LIMIT_ACTIVE_W`)
pub const POWER_LIMIT_ACTIVE_W: f64 = 3.0;
/// 5 V bus regulation, lower limit (`EPS_BUS_MIN_V`)
pub const BUS_MIN_V: f64 = 4.5;
/// 5 V bus regulation, upper limit (`EPS_BUS_MAX_V`)
pub const BUS_MAX_V: f64 = 5.5;

/// Bytes of the stored record (`sizeof(EpsControlState_t)`)
pub const RECORD_SIZE: usize = 24;
/// Offset of the CRC: it covers every byte before it
const CRC_OFFSET: usize = 20;

/// A power mode from its C value
pub fn power_mode(raw: u32) -> Option<PowerMode> {
    match raw {
        0 => Some(PowerMode::Safe),
        1 => Some(PowerMode::Idle),
        2 => Some(PowerMode::Active),
        _ => None,
    }
}

/// Power budget of a mode, W
pub fn power_limit(mode: PowerMode) -> f64 {
    match mode {
        PowerMode::Safe => POWER_LIMIT_SAFE_W,
        PowerMode::Idle => POWER_LIMIT_IDLE_W,
        PowerMode::Active => POWER_LIMIT_ACTIVE_W,
    }
}

/// The load switches a mode sets
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loads {
    /// Jetson payload
    pub payload: bool,
    /// Radio / transponder
    pub radio: bool,
    /// ADCS
    pub adcs: bool,
    /// Beacon
    pub beacon: bool,
}

/// Load switches of a mode. SAFE keeps the beacon for emergencies; only
/// ACTIVE asks for the payload (which its own rule may still refuse).
pub fn loads(mode: PowerMode) -> Loads {
    match mode {
        PowerMode::Safe => Loads { payload: false, radio: false, adcs: false, beacon: true },
        PowerMode::Idle => Loads { payload: false, radio: true, adcs: true, beacon: true },
        PowerMode::Active => Loads { payload: true, radio: true, adcs: true, beacon: true },
    }
}

/// A state of charge the EPS accepts as an argument: 0..1 (NaN is not)
pub fn soc_in_range(soc: f64) -> bool {
    (0.0..=1.0).contains(&soc)
}

/// Why the payload may not be switched on
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PayloadBlock {
    /// The payload guard's cool-down or a hot heat spreader (SW-16)
    Guard,
    /// State of charge below [`SOC_PAYLOAD`]
    LowSoc,
}

/// The payload enable rule: the OBC's own power and temperature limits
/// first (the guard), then the state of charge. Switching off is always
/// allowed.
pub fn payload_rule(enable: bool, guard_allows: bool, soc: f64) -> Result<(), PayloadBlock> {
    if enable && !guard_allows {
        return Err(PayloadBlock::Guard);
    }
    if enable && soc < SOC_PAYLOAD {
        return Err(PayloadBlock::LowSoc);
    }
    Ok(())
}

/// A measured bus voltage within regulation (FDIR, SW-16); NaN is not
pub fn bus_voltage_ok(volts: f64) -> bool {
    (BUS_MIN_V..=BUS_MAX_V).contains(&volts)
}

/// EPS control state (the C `EpsControlState_t`, padding as a named field)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EpsState {
    /// Jetson payload power
    pub payload_enabled: bool,
    /// Radio / transponder power
    pub radio_enabled: bool,
    /// ADCS power
    pub adcs_enabled: bool,
    /// Beacon power
    pub beacon_enabled: bool,
    /// Current power mode (`PowerMode_t` value)
    pub power_mode: u8,
    pad: [u8; 3],
    /// Current power limit, W
    pub power_limit_w: f64,
    /// Last control command time, ms (low 32 bits)
    pub last_control_ms: u32,
    /// CRC-32 of every byte before this field
    pub crc32: u32,
}

const _: () = assert!(core::mem::size_of::<EpsState>() == RECORD_SIZE);

/// Why a stored record was not adopted
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
    /// CRC mismatch, or a switch flag that is neither 0 nor 1
    Corrupt,
}

impl EpsState {
    /// The power-on state: IDLE, payload off, everything else on
    pub const DEFAULT: EpsState = EpsState {
        payload_enabled: false,
        radio_enabled: true,
        adcs_enabled: true,
        beacon_enabled: true,
        power_mode: PowerMode::Idle as u8,
        pad: [0; 3],
        power_limit_w: POWER_LIMIT_IDLE_W,
        last_control_ms: 0,
        crc32: 0,
    };

    /// Back to the power-on switches, mode and limit, controlled at
    /// `now_ms` (the padding is kept, as the C module kept it)
    pub fn reset_defaults(&mut self, now_ms: u32) {
        *self = EpsState { pad: self.pad, last_control_ms: now_ms, ..EpsState::DEFAULT };
        self.update_crc();
    }

    /// The stored bytes (C memory layout, native byte order)
    pub fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut b = [0u8; RECORD_SIZE];
        let [p0, p1, p2] = self.pad;
        let head = [
            u8::from(self.payload_enabled),
            u8::from(self.radio_enabled),
            u8::from(self.adcs_enabled),
            u8::from(self.beacon_enabled),
            self.power_mode,
            p0,
            p1,
            p2,
        ];
        let parts: [&[u8]; 4] =
            [&head, &self.power_limit_w.to_ne_bytes(), &self.last_control_ms.to_ne_bytes(), &self.crc32.to_ne_bytes()];
        let mut at = 0;
        for part in parts {
            if let Some(dst) = b.get_mut(at..at + part.len()) {
                dst.copy_from_slice(part);
            }
            at += part.len();
        }
        b
    }

    /// CRC-32 over the bytes before the CRC field, in memory order
    pub fn compute_crc(&self) -> u32 {
        crc32(self.to_bytes().get(..CRC_OFFSET).unwrap_or(&[]))
    }

    /// Set the CRC from the other fields
    pub fn update_crc(&mut self) {
        self.crc32 = self.compute_crc();
    }

    /// A stored record, if its CRC matches and its flags are booleans
    pub fn from_bytes(b: &[u8; RECORD_SIZE]) -> Result<EpsState, RecordError> {
        let flag = |v: u8| match v {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(RecordError::Corrupt),
        };
        let [p, r, a, bc, mode, p0, p1, p2, l0, l1, l2, l3, l4, l5, l6, l7, t0, t1, t2, t3, c0, c1, c2, c3] = *b;
        let crc = u32::from_ne_bytes([c0, c1, c2, c3]);
        if crc32(b.get(..CRC_OFFSET).unwrap_or(&[])) != crc {
            return Err(RecordError::Corrupt);
        }
        Ok(EpsState {
            payload_enabled: flag(p)?,
            radio_enabled: flag(r)?,
            adcs_enabled: flag(a)?,
            beacon_enabled: flag(bc)?,
            power_mode: mode,
            pad: [p0, p1, p2],
            power_limit_w: f64::from_ne_bytes([l0, l1, l2, l3, l4, l5, l6, l7]),
            last_control_ms: u32::from_ne_bytes([t0, t1, t2, t3]),
            crc32: crc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_tables() {
        assert_eq!(power_mode(3), None);
        assert_eq!(power_mode(2), Some(PowerMode::Active));
        assert_eq!(power_limit(PowerMode::Safe), 0.5);
        assert_eq!(power_limit(PowerMode::Idle), 1.5);
        assert_eq!(power_limit(PowerMode::Active), 3.0);
        assert!(loads(PowerMode::Safe).beacon && !loads(PowerMode::Safe).radio);
        assert!(!loads(PowerMode::Idle).payload && loads(PowerMode::Idle).adcs);
        assert!(loads(PowerMode::Active).payload);
    }

    #[test]
    fn payload_rule_order() {
        assert_eq!(payload_rule(true, false, 0.9), Err(PayloadBlock::Guard));
        assert_eq!(payload_rule(true, false, 0.1), Err(PayloadBlock::Guard));
        assert_eq!(payload_rule(true, true, SOC_PAYLOAD - 0.01), Err(PayloadBlock::LowSoc));
        assert_eq!(payload_rule(true, true, SOC_PAYLOAD), Ok(()));
        assert_eq!(payload_rule(false, false, 0.0), Ok(()));
    }

    #[test]
    fn soc_and_bus_limits() {
        assert!(soc_in_range(0.0) && soc_in_range(1.0));
        assert!(!soc_in_range(-0.01) && !soc_in_range(1.01) && !soc_in_range(f64::NAN));
        assert!(!bus_voltage_ok(BUS_MIN_V - 0.01));
        assert!(bus_voltage_ok(BUS_MIN_V) && bus_voltage_ok(BUS_MAX_V));
        assert!(!bus_voltage_ok(BUS_MAX_V + 0.01) && !bus_voltage_ok(f64::NAN));
    }

    #[test]
    fn record_round_trip_and_corruption() {
        let mut s = EpsState { payload_enabled: true, power_mode: 2, last_control_ms: 1234, ..EpsState::DEFAULT };
        s.update_crc();
        let b = s.to_bytes();
        assert_eq!(&b[..5], &[1, 1, 1, 1, 2]);
        assert_eq!(EpsState::from_bytes(&b), Ok(s));
        let mut bad = b;
        bad[16] ^= 1;
        assert_eq!(EpsState::from_bytes(&bad), Err(RecordError::Corrupt));
        // A valid CRC over a flag that is not a boolean
        let mut flag = b;
        flag[0] = 2;
        let crc = crc32(&flag[..CRC_OFFSET]).to_ne_bytes();
        flag[20..].copy_from_slice(&crc);
        assert_eq!(EpsState::from_bytes(&flag), Err(RecordError::Corrupt));
    }

    #[test]
    fn every_power_mode_value() {
        assert_eq!(power_mode(0), Some(PowerMode::Safe));
        assert_eq!(power_mode(1), Some(PowerMode::Idle));
        assert_eq!(power_mode(2), Some(PowerMode::Active));
        for mode in [PowerMode::Safe, PowerMode::Idle, PowerMode::Active] {
            assert_eq!(power_mode(mode as u32), Some(mode));
        }
    }

    /// Mutation check: reset_defaults restores the power-on switches, mode
    /// and limit, stamps the control time, keeps the padding (as the C
    /// module did) and leaves a CRC that matches
    #[test]
    fn reset_defaults_restores_the_power_on_state() {
        let mut s = EpsState {
            payload_enabled: true,
            radio_enabled: false,
            adcs_enabled: false,
            beacon_enabled: false,
            power_mode: PowerMode::Active as u8,
            pad: [7, 8, 9],
            power_limit_w: POWER_LIMIT_ACTIVE_W,
            last_control_ms: 5,
            crc32: 0,
        };
        s.reset_defaults(4242);
        assert_eq!(s, EpsState { pad: [7, 8, 9], last_control_ms: 4242, crc32: s.compute_crc(), ..EpsState::DEFAULT });
        assert_ne!(s.crc32, 0);
        assert_eq!(EpsState::from_bytes(&s.to_bytes()), Ok(s));
    }

    #[test]
    fn record_with_switches_off_round_trips() {
        let mut s = EpsState::DEFAULT; // payload off
        s.radio_enabled = false;
        s.update_crc();
        let b = s.to_bytes();
        assert_eq!(&b[..4], &[0, 0, 1, 1]);
        assert_eq!(EpsState::from_bytes(&b), Ok(s));
    }
}
