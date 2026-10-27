//! The downlinked housekeeping frame (`HK1`, LV-27): the state the ground
//! needs to prove commissioning, which the beacon does not carry.
//!
//! An AX.25 UI frame from the station's call sign to `CQ`, like the beacon,
//! whose information field starts `HK1 ` so beacon decoders can tell it
//! apart; the beacon's own format is unchanged (`docs/BEACON_SPEC.md`). The
//! field is plain ASCII `KEY=value` pairs separated by spaces, integers
//! only, `-` for a value not measured. Every value comes from a measurement
//! or the flight software's state, never from the AI payload.
//!
//! | Key | Value | Unit |
//! |-----|-------|------|
//! | `N` | frame sequence (wraps at 65536) | — |
//! | `UP` | seconds since boot | s |
//! | `BOOT` | resets since the mission data began | — |
//! | `RST` | last reset cause (`HalResetCause_t`) | — |
//! | `MODE` | operating state (`state_machine.h`) | — |
//! | `HOLD` | commissioning hold 1/0 | — |
//! | `XP` | transponder enabled now 1/0 | — |
//! | `TX` | transmission permitted 1/0 | — |
//! | `SOC` | state of charge | ‰ |
//! | `VB` | battery voltage | mV |
//! | `IB` | battery current, positive charging | mA |
//! | `PA` | array power | mW |
//! | `TB`, `TO`, `TP`, `TJ` | battery, OBC, PA, Jetson heat-spreader temperature | 0.1 °C |
//! | `DEP` | deployment state (`deployment.h`)/antenna sense/wing sense | —/1/1 |
//! | `RFS` | RF-silence state (`rf_silence.h`)/seconds counted | —/s |
//! | `CT` | loss-of-contact timeout | days |
//! | `CS` | time since the last authenticated command | h |
//! | `FC` | fault-log entries | — |
//! | `FL` | newest fault: type/severity (`fault_mgmt.h`) | — |
//! | `W` | body rate x/y/z | 0.01 °/s |
//! | `AM` | ADCS mode (`adcs_control.h`) | — |
//! | `MR` | magnetometer samples rejected (coils on) | — |
//! | `GF` | GNSS: age of the latest fix/satellites used | s/— |
//! | `GS` | GNSS sessions run/with a fix | — |
//!
//! At most [`MAX_LEN`] bytes, about 1.3 s on the air at 1200 bps.
//!
//! @requirement SRS-F085 The software shall downlink a housekeeping frame every fifth beacon and on command, carrying temperatures, battery current and voltage, array power, deployment, RF-silence, boot and reset, fault-log head, body rates, contact timeout and GNSS fix age

use core::fmt::Write;

/// The information field's first bytes
pub const PREFIX: &str = "HK1";
/// Longest information field, bytes
pub const MAX_LEN: usize = 200;
/// One frame every this many beacons (and on GET_HOUSEKEEPING)
pub const EVERY_BEACONS: u32 = 5;
/// Gap after the beacon before the frame, ms (the downlink sends one frame
/// at a time)
pub const AFTER_BEACON_MS: u64 = 5_000;

/// The values (`None`: not measured)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Housekeeping {
    /// Frame sequence
    pub seq: u16,
    /// Seconds since boot
    pub uptime_s: u64,
    /// Resets
    pub boots: u32,
    /// Last reset cause
    pub reset_cause: u32,
    /// Operating state
    pub mode: u32,
    /// Commissioning hold
    pub hold: bool,
    /// Transponder enabled now
    pub xpdr: bool,
    /// Transmission permitted
    pub tx_permitted: bool,
    /// State of charge, 0–1
    pub soc: Option<f64>,
    /// Battery voltage, V
    pub vbatt_v: Option<f64>,
    /// Battery current, A (positive charging)
    pub ibatt_a: Option<f64>,
    /// Array power, W
    pub array_w: Option<f64>,
    /// Battery temperature, °C
    pub tbatt_c: Option<f64>,
    /// OBC temperature, °C
    pub tobc_c: Option<f64>,
    /// PA temperature, °C
    pub tpa_c: Option<f64>,
    /// Jetson heat-spreader temperature, °C
    pub tjet_c: Option<f64>,
    /// Deployment state, antenna sense, wing sense
    pub deploy: (u32, bool, bool),
    /// RF-silence state and seconds counted
    pub rf_silence: (u32, u32),
    /// Loss-of-contact timeout, days
    pub contact_timeout_d: u16,
    /// Seconds since the last authenticated command
    pub since_contact_s: u32,
    /// Fault-log entries
    pub fault_count: u32,
    /// Newest fault: type and severity (`None`: empty log)
    pub last_fault: Option<(u8, u8)>,
    /// Body rate, 0.01 °/s
    pub rate_cdps: [i16; 3],
    /// ADCS mode
    pub adcs_mode: u8,
    /// Magnetometer samples rejected
    pub mag_rejected: u32,
    /// GNSS: age of the latest fix, s, and satellites (`None`: no fix)
    pub gnss_fix: Option<(u64, u8)>,
    /// GNSS sessions run and with a fix
    pub gnss_sessions: (u32, u32),
}

struct Out<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl Write for Out<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(core::fmt::Error)?;
        let dst = self.buf.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        dst.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// An optional scaled value: the rounded integer, or `-`
struct Opt(Option<f64>, f64);

impl core::fmt::Display for Opt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0.filter(|v| v.is_finite()) {
            #[allow(clippy::cast_possible_truncation)] // clamped to i32
            Some(v) => write!(f, "{}", crate::fmath::round(v * self.1).clamp(-2.0e9, 2.0e9) as i32),
            None => f.write_str("-"),
        }
    }
}

fn flag(b: bool) -> u8 {
    u8::from(b)
}

/// Write the information field into `out`; its length (0 if it does not fit)
pub fn format(hk: &Housekeeping, out: &mut [u8]) -> usize {
    let room = out.len().min(MAX_LEN);
    let mut w = Out { buf: out.get_mut(..room).unwrap_or(&mut []), len: 0 };
    let ok = write!(
        w,
        "{PREFIX} N={} UP={} BOOT={} RST={} MODE={} HOLD={} XP={} TX={} SOC={} VB={} IB={} PA={} TB={} TO={} TP={} TJ={} \
         DEP={}/{}/{} RFS={}/{} CT={} CS={} FC={} ",
        hk.seq,
        hk.uptime_s,
        hk.boots,
        hk.reset_cause,
        hk.mode,
        flag(hk.hold),
        flag(hk.xpdr),
        flag(hk.tx_permitted),
        Opt(hk.soc, 1000.0),
        Opt(hk.vbatt_v, 1000.0),
        Opt(hk.ibatt_a, 1000.0),
        Opt(hk.array_w, 1000.0),
        Opt(hk.tbatt_c, 10.0),
        Opt(hk.tobc_c, 10.0),
        Opt(hk.tpa_c, 10.0),
        Opt(hk.tjet_c, 10.0),
        hk.deploy.0,
        flag(hk.deploy.1),
        flag(hk.deploy.2),
        hk.rf_silence.0,
        hk.rf_silence.1,
        hk.contact_timeout_d,
        hk.since_contact_s / 3600,
        hk.fault_count,
    )
    .and_then(|()| match hk.last_fault {
        Some((t, s)) => write!(w, "FL={t}/{s} "),
        None => w.write_str("FL=- "),
    })
    .and_then(|()| {
        write!(
            w,
            "W={}/{}/{} AM={} MR={} ",
            hk.rate_cdps[0], hk.rate_cdps[1], hk.rate_cdps[2], hk.adcs_mode, hk.mag_rejected
        )
    })
    .and_then(|()| match hk.gnss_fix {
        Some((age, sats)) => write!(w, "GF={age}/{sats} "),
        None => w.write_str("GF=- "),
    })
    .and_then(|()| write!(w, "GS={}/{}", hk.gnss_sessions.0, hk.gnss_sessions.1))
    .is_ok();
    if ok { w.len } else { 0 }
}

/// When the next frame is due: every [`EVERY_BEACONS`] beacons, or at once
/// on request
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cadence {
    beacons: u32,
    requested: bool,
    due_ms: Option<u64>,
    /// Sequence of the next frame
    pub seq: u16,
}

impl Cadence {
    /// Nothing sent yet
    pub const fn new() -> Cadence {
        Cadence { beacons: 0, requested: false, due_ms: None, seq: 0 }
    }

    /// GET_HOUSEKEEPING: a frame after the next beacon gap
    pub fn request(&mut self, now_ms: u64) {
        self.requested = true;
        self.due_ms = Some(now_ms.saturating_add(AFTER_BEACON_MS));
    }

    /// A beacon was sent at `now_ms`
    pub fn beacon_sent(&mut self, now_ms: u64) {
        self.beacons = self.beacons.saturating_add(1);
        if self.beacons >= EVERY_BEACONS && self.due_ms.is_none() {
            self.due_ms = Some(now_ms.saturating_add(AFTER_BEACON_MS));
        }
    }

    /// Whether a frame is due at `now_ms`
    pub fn due(&self, now_ms: u64) -> bool {
        self.due_ms.is_some_and(|t| now_ms >= t)
    }

    /// The frame went out (or could not be sent and is dropped)
    pub fn sent(&mut self) {
        self.beacons = 0;
        self.requested = false;
        self.due_ms = None;
        self.seq = self.seq.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate std;

    fn text(hk: &Housekeeping) -> std::string::String {
        let mut b = [0u8; 256];
        let n = format(hk, &mut b);
        std::string::String::from_utf8(b[..n].to_vec()).unwrap()
    }

    #[test]
    fn every_field_measured_or_dashed() {
        let hk = Housekeeping {
            seq: 7,
            uptime_s: 3600,
            boots: 3,
            reset_cause: 2,
            mode: 3,
            hold: true,
            tx_permitted: true,
            soc: Some(0.8734),
            vbatt_v: Some(15.812),
            ibatt_a: Some(-0.12),
            array_w: Some(2.41),
            tbatt_c: Some(12.3),
            tobc_c: Some(21.5),
            tpa_c: Some(30.1),
            deploy: (8, true, true),
            rf_silence: (3, 2700),
            contact_timeout_d: 30,
            since_contact_s: 7300,
            fault_count: 4,
            last_fault: Some((17, 2)),
            rate_cdps: [12, -5, 600],
            adcs_mode: 2,
            gnss_fix: Some((1200, 7)),
            gnss_sessions: (5, 4),
            ..Housekeeping::default()
        };
        let t = text(&hk);
        assert_eq!(
            t,
            "HK1 N=7 UP=3600 BOOT=3 RST=2 MODE=3 HOLD=1 XP=0 TX=1 SOC=873 VB=15812 IB=-120 PA=2410 TB=123 TO=215 \
             TP=301 TJ=- DEP=8/1/1 RFS=3/2700 CT=30 CS=2 FC=4 FL=17/2 W=12/-5/600 AM=2 MR=0 GF=1200/7 GS=5/4"
        );
        assert!(t.len() <= MAX_LEN && t.is_ascii());
        let empty = text(&Housekeeping { soc: Some(f64::NAN), ..Housekeeping::default() });
        assert!(empty.contains("SOC=- ") && empty.contains("FL=- ") && empty.contains("GF=- "), "{empty}");
    }

    #[test]
    fn worst_case_fits_and_a_small_buffer_gives_nothing() {
        let hk = Housekeeping {
            seq: u16::MAX,
            uptime_s: 99_999_999,
            boots: 99_999,
            reset_cause: 5,
            mode: 5,
            soc: Some(1.0),
            vbatt_v: Some(16.8),
            ibatt_a: Some(-3.5),
            array_w: Some(9.999),
            tbatt_c: Some(-40.0),
            tobc_c: Some(-40.0),
            tpa_c: Some(-40.0),
            tjet_c: Some(-40.0),
            deploy: (10, true, true),
            rf_silence: (3, 2700),
            contact_timeout_d: 90,
            since_contact_s: 90 * 86_400,
            fault_count: 100,
            last_fault: Some((22, 4)),
            rate_cdps: [-32768, -32768, -32768],
            adcs_mode: 4,
            mag_rejected: 4_000_000_000,
            gnss_fix: Some((99_999_999, 24)),
            gnss_sessions: (99_999, 99_999),
            ..Housekeeping::default()
        };
        assert!(text(&hk).len() <= MAX_LEN, "{}", text(&hk).len());
        let mut small = [0u8; 20];
        assert_eq!(format(&hk, &mut small), 0);
    }

    #[test]
    fn cadence_every_fifth_beacon_or_on_request() {
        let mut c = Cadence::new();
        for k in 1..EVERY_BEACONS {
            c.beacon_sent(u64::from(k) * 60_000);
            assert!(!c.due(u64::from(k) * 60_000 + 10_000));
        }
        c.beacon_sent(300_000);
        assert!(!c.due(300_000 + AFTER_BEACON_MS - 1));
        assert!(c.due(300_000 + AFTER_BEACON_MS));
        c.sent();
        assert_eq!(c.seq, 1);
        assert!(!c.due(400_000));
        c.request(500_000);
        assert!(c.due(500_000 + AFTER_BEACON_MS));
        c.sent();
        assert!(!c.due(u64::MAX));
        assert_eq!(Cadence::default(), Cadence { seq: 0, ..Cadence::new() });
    }
}
