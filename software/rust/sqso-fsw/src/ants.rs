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
    fn a_stowed_unit_reports_every_element_not_deployed() {
        let s = Status::decode([0x00, 0x00]);
        assert!(s.all_deployed(), "all bits clear: every element deployed");
        let stowed = Status::decode([0x88, 0x88]);
        assert!(!stowed.armed && !stowed.all_deployed());
        assert_eq!(stowed.deployed_mask(), 0);
        // Element 1 out, armed, element 2 burning
        let mut st = stowed;
        st.armed = true;
        st.elements[0].deployed = true;
        st.elements[1].burning = true;
        let back = Status::decode(st.encode());
        assert_eq!(back, st);
        assert_eq!(back.deployed_mask(), 0b0001);
        assert!(back.elements[1].burning && !back.elements[1].deployed);
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
