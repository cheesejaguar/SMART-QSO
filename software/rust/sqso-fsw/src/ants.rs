//! The ISIS AntS deployable antenna's I2C protocol (bought antenna,
//! `docs/hardware/BUILD_VS_BUY.md` §2.6; launch review LV-07, STR-16).
//!
//! The AntS carries its own deployment controller: the OBC arms it and asks
//! for an automatic sequential deployment of its four tape elements, then
//! reads the per-element deployment switches back. The three deployment
//! inhibits of CDS 2.3.8 sit on the AntS supply (hardware), not here.
//!
//! Command codes and the status word follow ISIS's published AntS
//! interface (arm 0xAD, disarm 0xAC, automatic deployment 0xA5 with a
//! per-element time limit in seconds, cancel 0xA9, status 0xC3 returning
//! two bytes, low byte first). **They must be confirmed against the ICD
//! delivered with the unit** (verification item AV-V7 in
//! `docs/interfaces/ICD_OBC_ADCS.md`); this module is the only place they
//! live.
//!
//! @requirement SRS-F089 The software shall deploy the bought antenna through its I2C controller, wings first, and read each element's deployment switch

/// The controller's 7-bit address (side A)
pub const ADDRESS: u8 = 0x31;
/// The AntS's second, redundant controller (side B): the retry path when
/// side A does not acknowledge or the elements stay stowed (SQSO-ICX-001 D
/// §6 row 30, RID SMA-B07)
pub const ADDRESS_B: u8 = 0x32;
/// Arm the deployment system
pub const CMD_ARM: u8 = 0xAD;
/// Disarm it
pub const CMD_DISARM: u8 = 0xAC;
/// Deploy every element in turn, each for at most the given seconds
pub const CMD_AUTO_DEPLOY: u8 = 0xA5;
/// Stop any deployment in progress
pub const CMD_CANCEL: u8 = 0xA9;
/// Report the deployment status (two bytes)
pub const CMD_STATUS: u8 = 0xC3;
/// Report the controller's temperature (two bytes, low byte first: a
/// 10-bit reading). **A documented assumption** until the delivered ISIS
/// ICD confirms the command and its conversion (SQSO-MFG-003 §12 V-1, with
/// AV-V7): the code of ISIS's published "measure system temperature"
/// command, and an analog sensor of 10 mV/°C with 500 mV at 0 °C read by a
/// 10-bit converter at 3.3 V full scale ([`temperature_c`])
pub const CMD_TEMPERATURE: u8 = 0xC0;
/// The converter's full scale, V, and its counts
const TEMP_FULL_SCALE_V: f64 = 3.3;
const TEMP_COUNTS: f64 = 1024.0;
/// The assumed sensor: V = 0.5 V + 10 mV/°C · T
const TEMP_OFFSET_V: f64 = 0.5;
const TEMP_SLOPE_V_PER_C: f64 = 0.010;
/// A conversion outside this range is not a reading, °C
pub const TEMP_VALID_C: (f64, f64) = (-60.0, 125.0);

/// The temperature, °C, from the two bytes [`CMD_TEMPERATURE`] returns
/// (`None`: more than 10 bits, either end of the converter's range (an
/// open or shorted sensor), or outside [`TEMP_VALID_C`])
pub fn temperature_c(bytes: [u8; 2]) -> Option<f64> {
    let raw = u16::from_le_bytes(bytes);
    if raw == 0 || raw >= 1023 {
        return None;
    }
    let volts = f64::from(raw) * TEMP_FULL_SCALE_V / TEMP_COUNTS;
    let t = (volts - TEMP_OFFSET_V) / TEMP_SLOPE_V_PER_C;
    (TEMP_VALID_C.0..=TEMP_VALID_C.1).contains(&t).then_some(t)
}

/// The two bytes for a temperature (the inverse of [`temperature_c`]: the
/// simulation's AntS model)
pub fn temperature_bytes(t_c: f64) -> [u8; 2] {
    let raw = crate::fmath::round((TEMP_OFFSET_V + TEMP_SLOPE_V_PER_C * t_c) * TEMP_COUNTS / TEMP_FULL_SCALE_V)
        .clamp(0.0, 1023.0);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped to 0..=1023
    (raw as u16).to_le_bytes()
}

/// Time limit per element, s (the AntS burns one element at a time)
pub const ELEMENT_LIMIT_S: u8 = 10;
/// Tape elements
pub const ELEMENTS: usize = 4;

/// One element's status bits
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ElementState {
    /// The deployment switch reads deployed
    pub deployed: bool,
    /// The last activation stopped at its time limit
    pub timed_out: bool,
    /// Burning now
    pub burning: bool,
}

/// The status word
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Status {
    /// The deployment system is armed
    pub armed: bool,
    /// Elements 1 to 4
    pub elements: [ElementState; ELEMENTS],
}

impl Status {
    /// From the two status bytes (low byte first). Element n (1 to 4) has
    /// "not deployed" at bit 15 − 4(n − 1), "time limit reached" one below
    /// it and "burning" two below; bit 0 is "armed"
    pub fn decode(bytes: [u8; 2]) -> Status {
        let word = u16::from_le_bytes(bytes);
        let bit = |b: u32| word & (1u16 << b) != 0;
        let mut s = Status { armed: bit(0), elements: [ElementState::default(); ELEMENTS] };
        for (k, e) in (0u32..).zip(s.elements.iter_mut()) {
            let top = 15 - 4 * k;
            *e = ElementState { deployed: !bit(top), timed_out: bit(top - 1), burning: bit(top - 2) };
        }
        s
    }

    /// Every element's switch reads deployed
    pub fn all_deployed(&self) -> bool {
        self.elements.iter().all(|e| e.deployed)
    }

    /// The deployed elements as bits 0 to 3 (telemetry)
    pub fn deployed_mask(&self) -> u8 {
        (0u8..).zip(self.elements.iter()).fold(0, |m, (k, e)| if e.deployed { m | (1 << k) } else { m })
    }

    /// The status word for these states (the inverse of [`Status::decode`]:
    /// the simulation's antenna model)
    pub fn encode(&self) -> [u8; 2] {
        let mut word = u16::from(self.armed);
        for (k, e) in (0u32..).zip(self.elements.iter()) {
            let top = 15 - 4 * k;
            if !e.deployed {
                word |= 1 << top;
            }
            if e.timed_out {
                word |= 1 << (top - 1);
            }
            if e.burning {
                word |= 1 << (top - 2);
            }
        }
        word.to_le_bytes()
    }
}

/// The I2C writes that start a deployment: arm, then deploy all
pub const START: [&[u8]; 2] = [&[CMD_ARM], &[CMD_AUTO_DEPLOY, ELEMENT_LIMIT_S]];
/// The I2C writes that stop one: cancel, then disarm
pub const STOP: [&[u8]; 2] = [&[CMD_CANCEL], &[CMD_DISARM]];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_temperature_reading() {
        for t in [-40.0, -17.0, 0.0, 25.0, 60.0] {
            let back = temperature_c(temperature_bytes(t)).unwrap();
            assert!((back - t).abs() < 0.2, "{t} {back}");
        }
        assert_eq!(temperature_c([0x00, 0x04]), None, "more than 10 bits");
        assert_eq!(temperature_c([0x00, 0x00]), None, "the converter's floor: open or shorted");
        assert_eq!(temperature_c([0xFF, 0x03]), None, "its ceiling");
        assert_eq!(temperature_c(temperature_bytes(130.0)), None, "above +125 C");
    }

    #[test]
    fn a_stowed_unit_reports_every_element_not_deployed() {
        let s = Status::decode([0x00, 0x00]);
        assert!(s.all_deployed(), "all bits clear: every element deployed");
        let stowed = Status::decode([0x88, 0x88]);
        assert!(!stowed.armed);
        assert!(!stowed.all_deployed());
        assert_eq!(stowed.deployed_mask(), 0);
        // Element 1 out, armed, element 2 burning
        let mut st = stowed;
        st.armed = true;
        st.elements[0].deployed = true;
        st.elements[1].burning = true;
        let back = Status::decode(st.encode());
        assert_eq!(back, st);
        assert_eq!(back.deployed_mask(), 0b0001);
        assert!(back.elements[1].burning);
        assert!(!back.elements[1].deployed);
    }

    #[test]
    fn every_bit_round_trips() {
        for word in [0u16, 0xFFFF, 0x8889, 0x4444, 0x2222, 0x1111] {
            let s = Status::decode(word.to_le_bytes());
            // Bits 4, 8 and 12 (unused here) do not survive; all others do
            assert_eq!(u16::from_le_bytes(s.encode()) & !0x1110, word & !0x1110, "{word:#x}");
        }
        assert_eq!(START[1], &[CMD_AUTO_DEPLOY, ELEMENT_LIMIT_S]);
        assert_eq!(STOP[0], &[CMD_CANCEL]);
    }
}
