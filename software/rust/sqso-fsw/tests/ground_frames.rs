//! The frames the ground decoder is tested with
//! (`software/ground/tests/fixtures/flight_frames.txt`), made by the flight
//! formatters: the beacon (`beacon::build_frame`, on the air with flags and
//! FCS) and the housekeeping texts (`housekeeping::format`, `format2`, cut
//! into frames by `housekeeping::Parts` as the executive sends them, each in
//! a UI frame to CQ with its FCS, as the TNC puts it on the air). Each
//! frame is one line, `<label> <hex bytes>`, the bytes separated by spaces
//! (a form the decoder's file input reads, and one the repository's secret
//! scan does not take for a 256-bit key).
//!
//! The test fails when the committed fixture differs from what the
//! formatters make now, so a change to a flight format reaches the ground
//! decoder's tests. Regenerate with
//! `SQSO_UPDATE_FIXTURES=1 cargo test -p sqso-fsw --test ground_frames`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use std::fmt::Write as _;
use std::path::PathBuf;

use sqso_fsw::beacon::{self, Content, Telemetry};
use sqso_fsw::housekeeping::{self, Housekeeping, Housekeeping2, Parts};
use sqso_fsw::selector;
use sqso_protocol::{ax25, kiss, station};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ground/tests/fixtures/flight_frames.txt")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().enumerate().fold(String::new(), |mut s, (i, b)| {
        let sep = if i == 0 { "" } else { " " };
        let _ = write!(s, "{sep}{b:02X}");
        s
    })
}

fn beacon_content(text: &str, telemetry: Telemetry) -> Content {
    let mut c = Content::ZERO;
    let n = text.len().min(beacon::MAX_TEXT_LEN - 1);
    c.text[..n].copy_from_slice(&text.as_bytes()[..n]);
    c.text_len = n;
    c.telemetry = telemetry;
    c
}

/// The beacon as it goes on the air: flag, frame, FCS, flag
fn beacon_frame(c: &Content) -> Vec<u8> {
    let (frame, ok) = beacon::build_frame(c);
    assert!(ok);
    let mut out = [0u8; ax25::MAX_FRAME_LEN];
    let n = beacon::serialize(&frame, &mut out);
    assert!(n > 0);
    out[..n].to_vec()
}

/// A housekeeping text's frames, each a UI frame CQ <- KK6GDB-11 with FCS
fn hk_frames(format: impl FnOnce(&mut [u8]) -> usize) -> Vec<Vec<u8>> {
    let mut parts = Parts::EMPTY;
    let n = format(parts.buffer());
    assert!(n > 0);
    parts.load(n);
    let mut frames = Vec::new();
    while parts.pending() {
        let mut info = [0u8; housekeeping::MAX_LEN];
        let (len, next) = parts.peek(&mut info);
        parts.commit(next);
        let mut out = [0u8; ax25::MAX_FRAME_LEN];
        let m = ax25::encode_ui(
            ax25::Station { call: "CQ", ssid: 0 },
            ax25::Station { call: station::CALLSIGN, ssid: station::SSID },
            &info[..len],
            true,
            &mut out,
        )
        .unwrap();
        frames.push(out[..m].to_vec());
    }
    frames
}

fn hk1_typical() -> Housekeeping {
    Housekeeping {
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
        deploy: (8, Some(15), true),
        rf_silence: (3, 2700),
        contact_timeout_d: 30,
        since_contact_s: 7300,
        fault_count: 4,
        last_fault: Some((17, 2)),
        rate_cdps: [12, -5, 600],
        adcs_mode: 2,
        gnss_fix: Some((1200, 7)),
        gnss_sessions: (5, 4),
        passivation: 1,
        eol: (Some(4), 0),
        eol_wait: 3,
        rx: (2, false),
        contacts: 3,
        stored: [12, 60],
        ants_gate: (2, Some(-18.6)),
        ..Housekeeping::default()
    }
}

/// Every field at its widest (341 bytes: two frames)
fn hk1_widest() -> Housekeeping {
    Housekeeping {
        seq: u16::MAX,
        uptime_s: 99_999_999,
        boots: 99_999,
        reset_cause: 5,
        mode: 5,
        hold: true,
        xpdr: true,
        tx_permitted: true,
        soc: Some(1.0),
        vbatt_v: Some(16.8),
        ibatt_a: Some(-3.5),
        array_w: Some(9.999),
        tbatt_c: Some(-40.0),
        tobc_c: Some(-40.0),
        tpa_c: Some(-40.0),
        tjet_c: Some(-40.0),
        deploy: (10, Some(15), true),
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
        passivation: 4,
        eol: (Some(255), 3),
        eol_wait: 15,
        rx: (99_999, true),
        contacts: 99_999,
        stored: [u32::MAX, u32::MAX],
        fram: (2, u32::MAX),
        ants_gate: (u8::MAX, Some(-59.6)),
    }
}

fn hk2_typical() -> Housekeeping2 {
    let mut sel = selector::Stats { enabled: true, ..selector::Stats::new() };
    sel.record(selector::Head::Template, true);
    sel.record(selector::Head::PowerMode, false);
    Housekeeping2 {
        seq: 7,
        commands: (12, 3, 1),
        wdt_resets: 2,
        xpdr: (4, 3600, 4100),
        array_a: Some(0.412),
        bursts: (6, 1210.4, 10, 0),
        jetson: (5, 1, 0, 0),
        eol: (Some(0x04), 0),
        rtc: (Some(1_900_000_000), None),
        clock_ppm: Some(-12.4),
        trips: [1, 0, 0, 0],
        gyro: (0, 37),
        parity: Some(true),
        separation: (true, 1),
        pa_sensor: (false, 2),
        supv: (Some(0x21), 0),
        chg_inhibit: Some(0x40),
        tnc: Some(kiss::TncStatus {
            version: 3,
            criteria: 5,
            floor_dbm: -121,
            floor_cal_dbm: -119,
            carrier_count: 42,
            frames: 17,
            uptime_s: 86_400,
        }),
        rx_lines: (true, false),
        selector: sel,
        geometry: 1,
        sun: Some([-0.1383, 0.9894, 0.0443]),
        ..Housekeeping2::default()
    }
}

/// Every field at its widest (573 bytes: three frames)
fn hk2_widest() -> Housekeeping2 {
    Housekeeping2 {
        seq: u16::MAX,
        commands: (u32::MAX, u32::MAX, u32::MAX),
        wdt_resets: u32::MAX,
        xpdr: (u32::MAX, u32::MAX, u32::MAX),
        array_a: Some(-2.6),
        hard_limit: u32::MAX,
        bursts: (u32::MAX, 4.0e9, usize::MAX, u32::MAX),
        jetson: (u32::MAX, u32::MAX, u32::MAX, u32::MAX),
        eol: (Some(255), 255),
        rtc: (Some(u32::MAX), Some(u32::MAX)),
        clock_ppm: Some(-1.0e6),
        trips: [u32::MAX; 4],
        gyro: (2, u32::MAX),
        mtq_faults: (u32::MAX, u32::MAX),
        parity: Some(false),
        separation: (true, u32::MAX),
        pa_sensor: (true, u32::MAX),
        supv: (Some(255), u32::MAX),
        chg_inhibit: Some(255),
        tnc: Some(kiss::TncStatus {
            version: 255,
            criteria: 63,
            floor_dbm: -128,
            floor_cal_dbm: -128,
            carrier_count: u16::MAX,
            frames: u16::MAX,
            uptime_s: u32::MAX,
        }),
        rx_lines: (true, true),
        selector: selector::Stats {
            enabled: true,
            decisions: [u32::MAX; selector::HEADS],
            abstentions: [u32::MAX; selector::HEADS],
        },
        geometry: u8::MAX,
        sun: Some([-1.0; 3]),
    }
}

fn fixture() -> String {
    let mut out = String::from(
        "# Frames made by the flight formatters (software/rust/sqso-fsw/tests/ground_frames.rs); do not edit.\n\
         # One frame per line: <label> <frame bytes in hex>. Beacons on the air (flags and FCS); housekeeping frames\n\
         # from the address field through the FCS.\n",
    );
    let mut line = |label: &str, frame: &[u8]| {
        let _ = writeln!(out, "{label} {}", hex(frame));
    };

    let typical = Telemetry {
        timestamp: 1_792_115_511,
        power_mode: 2,
        soc: 83,
        battery_mv: 14_400,
        sunlit: 1,
        rf_power: 1,
        valid: beacon::TLM_SOC | beacon::TLM_BV | beacon::TLM_SUN | beacon::TLM_RF,
        ..Telemetry::default()
    };
    line("beacon_typical", &beacon_frame(&beacon_content("Amateur satellite beacon - 73 de KK6GDB-11", typical)));
    let unmeasured = Telemetry { timestamp: 42, power_mode: 0, valid: 0, ..Telemetry::default() };
    line("beacon_unmeasured", &beacon_frame(&beacon_content("Hello from space! SMART-QSO CubeSat", unmeasured)));
    let idle =
        Telemetry { timestamp: 0, power_mode: 1, battery_mv: 14_449, valid: beacon::TLM_BV, ..Telemetry::default() };
    line("beacon_idle", &beacon_frame(&beacon_content("Greetings from LEO! 73 to all", idle)));
    // The longest text, cut to the room the widest telemetry leaves
    let widest = Telemetry {
        timestamp: u32::MAX,
        power_mode: 2,
        soc: u8::MAX,
        battery_mv: u16::MAX,
        sunlit: u8::MAX,
        rf_power: u8::MAX,
        qso_count: u16::MAX,
        valid: beacon::TLM_SOC | beacon::TLM_BV | beacon::TLM_SUN | beacon::TLM_RF | beacon::TLM_QSO,
        ..Telemetry::default()
    };
    let long = "Hello from orbit: a long beacon text. ".repeat(6);
    line("beacon_widest", &beacon_frame(&beacon_content(&long, widest)));

    for (label, frames) in [
        ("hk1_typical", hk_frames(|b| housekeeping::format(&hk1_typical(), b))),
        ("hk1_widest", hk_frames(|b| housekeeping::format(&hk1_widest(), b))),
        ("hk1_unmeasured", hk_frames(|b| housekeeping::format(&Housekeeping::default(), b))),
        ("hk2_typical", hk_frames(|b| housekeeping::format2(&hk2_typical(), b))),
        ("hk2_widest", hk_frames(|b| housekeeping::format2(&hk2_widest(), b))),
        ("hk2_unmeasured", hk_frames(|b| housekeeping::format2(&Housekeeping2::default(), b))),
    ] {
        for (k, f) in frames.iter().enumerate() {
            line(&format!("{label}.{}/{}", k + 1, frames.len()), f);
        }
    }
    out
}

#[test]
fn ground_decoder_fixture_is_current() {
    let made = fixture();
    let path = fixture_path();
    if std::env::var_os("SQSO_UPDATE_FIXTURES").is_some() {
        std::fs::write(&path, &made).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        committed == made,
        "{} is not what the flight formatters make now; regenerate it with \
         SQSO_UPDATE_FIXTURES=1 cargo test -p sqso-fsw --test ground_frames",
        path.display()
    );
}

/// The fixture covers the widest texts, cut into several frames
#[test]
fn widest_texts_take_several_frames() {
    let made = fixture();
    assert!(made.contains("hk1_widest.2/2 "));
    assert!(made.contains("hk2_widest.3/3 "));
}
