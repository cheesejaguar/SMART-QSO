//! The ground segment's side of the flight software's interfaces:
//! pre-launch key loading over the umbilical (GSE), authenticated
//! telecommands over the 435 MHz uplink, and decoding of the beacons heard.
//!
//! Keys here are simulation keys derived from the run's seed at run time;
//! no key material is stored in the repository (BASELINE §7).

use sqso_protocol::ax25::{self, Station};
use sqso_protocol::{auth, gse, station};

/// The ground station's call sign and SSID on the uplink (the licensee's
/// station; telecommand from a designated station, 47 CFR 97.211)
pub const GROUND_CALL: &str = "KK6GDB";
/// The ground station's SSID
pub const GROUND_SSID: u8 = 1;

/// Telecommand ids (`sqso_protocol::dictionary`)
pub mod cmd {
    /// No operation (a contact)
    pub const NOP: u8 = 0x00;
    /// Status request (a contact)
    pub const GET_STATUS: u8 = 0x03;
    /// Clear the fault log
    pub const CLEAR_FAULTS: u8 = 0x05;
    /// Loss-of-contact timeout, days (u16 big-endian)
    pub const SET_CONTACT_TIMEOUT: u8 = 0x07;
    /// Operational mode
    pub const SET_MODE: u8 = 0x02;
    /// Transmitter on (clears the TX-OFF latch)
    pub const TX_ENABLE: u8 = 0x31;
    /// Transmitter off (latched TX-OFF)
    pub const TX_DISABLE: u8 = 0x32;
    /// Downlink power level
    pub const SET_POWER: u8 = 0x33;
}

/// A simulation key: 32 bytes from SplitMix64 of the seed and the slot
pub fn sim_key(seed: u64, slot: u8) -> [u8; auth::KEY_SIZE] {
    let mut x = seed ^ (u64::from(slot) + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut key = [0u8; auth::KEY_SIZE];
    for chunk in key.chunks_mut(8) {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        chunk.copy_from_slice(&z.to_le_bytes()[..chunk.len()]);
    }
    key
}

/// The GSE frames of the pre-launch key load: four operational keys and
/// the recovery key, activate slot 0, then arm the RF silence and the
/// deployment sequence for flight
pub fn gse_preflight(seed: u64) -> Vec<[u8; gse::FRAME_SIZE]> {
    let mut frames = Vec::new();
    for slot in 0..=sqso_fsw::keys::RECOVERY_SLOT {
        frames.push(gse::encode(gse::op::PROVISION, slot, slot + 1, &sim_key(seed, slot)));
    }
    let zero = [0u8; auth::KEY_SIZE];
    frames.push(gse::encode(gse::op::ACTIVATE, 0, 0, &zero));
    frames.push(gse::encode(gse::op::ARM_FLIGHT, 0, 0, &zero));
    frames
}

/// The ground's telecommand sender: the active key and its counter
#[derive(Clone, Debug)]
pub struct Uplink {
    key: [u8; auth::KEY_SIZE],
    counter: u32,
    /// Commands built
    pub sent: u32,
}

impl Uplink {
    /// Sending with operational key 0
    pub fn new(seed: u64) -> Uplink {
        Uplink { key: sim_key(seed, 0), counter: 0, sent: 0 }
    }

    /// The AX.25 UI frame (with FCS, as the TNC delivers it to the OBC)
    /// carrying the authenticated command `cmd_id` with `args`
    pub fn frame(&mut self, cmd_id: u8, args: &[u8]) -> Option<Vec<u8>> {
        self.counter += 1;
        self.sent += 1;
        let key = auth::Key::from_slice(&self.key)?;
        let mut payload = [0u8; 300];
        let n = auth::build_frame(&key, cmd_id, self.counter, args, &mut payload).ok()?;
        let mut out = [0u8; ax25::MAX_FRAME_LEN];
        let dest = Station { call: station::CALLSIGN, ssid: station::SSID };
        let src = Station { call: GROUND_CALL, ssid: GROUND_SSID };
        let len = ax25::encode_ui(dest, src, &payload[..n], true, &mut out).ok()?;
        Some(out[..len].to_vec())
    }
}

/// What a beacon told the ground
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Beacon {
    /// The information field as text
    pub text: String,
    /// `SOC=` field, %
    pub soc: Option<f64>,
    /// `SUN=` field
    pub sunlit: Option<bool>,
    /// Whether it carried Jetson-generated text (not a template)
    pub ai: bool,
}

/// Decode a frame the OBC handed to the TNC (addresses, control, PID,
/// information; the TNC adds the FCS)
pub fn decode_beacon(frame: &[u8]) -> Option<Beacon> {
    // Addresses end at the byte with the extension bit set
    let end = frame.iter().position(|b| b & 1 == 1)? + 1;
    let info = frame.get(end + 2..)?;
    let text = String::from_utf8_lossy(info).into_owned();
    let field = |key: &str| {
        text.split([' ', ',', '|'])
            .find_map(|w| w.strip_prefix(key))
            .and_then(|v| v.trim_end_matches(|c: char| !c.is_ascii_digit()).parse::<f64>().ok())
    };
    Some(Beacon { soc: field("SOC="), sunlit: field("SUN=").map(|v| v > 0.5), ai: !is_template(&text), text })
}

/// Whether a beacon's text is one of the flight software's templates
/// (otherwise it is a Jetson line the content filter passed)
fn is_template(text: &str) -> bool {
    sqso_fsw::beacon::TEMPLATES.iter().any(|t| t.to_str().is_ok_and(|t| text.contains(t)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_differ_by_slot_and_seed() {
        assert_ne!(sim_key(1, 0), sim_key(1, 1));
        assert_ne!(sim_key(1, 0), sim_key(2, 0));
        assert_eq!(sim_key(7, 3), sim_key(7, 3));
    }

    #[test]
    fn preflight_frames_parse() {
        let frames = gse_preflight(42);
        assert_eq!(frames.len(), 7);
        for f in &frames {
            assert!(gse::parse(f).is_ok());
        }
    }

    #[test]
    fn uplink_frame_is_addressed_to_the_spacecraft_and_counts_up() {
        let mut up = Uplink::new(9);
        let a = up.frame(cmd::NOP, &[]).unwrap();
        let b = up.frame(cmd::NOP, &[]).unwrap();
        assert_ne!(a, b);
        let ui = ax25::parse_ui(&a).unwrap();
        assert!(ax25::address_matches(ui.dest, station::CALLSIGN.as_bytes(), station::SSID));
    }
}
