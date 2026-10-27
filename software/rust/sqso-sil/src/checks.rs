//! The run's verdict: whole-mission totals, judged against the baseline
//! (`docs/BASELINE_PARAMETERS.md`) and the CubeSat Design Specification.
//!
//! Each check is an anomaly (the mission or the software is not nominal),
//! a warning (inside the limits but with little margin, or not measured),
//! or a pass. The run's exit status is non-zero if any anomaly remains.

use std::fmt::Write as _;

use crate::metrics::Orbit;
use crate::recorder::json_str;

/// Whole-mission totals the mission loop keeps
#[derive(Clone, Debug, Default)]
pub struct Totals {
    /// Seconds simulated after deployment
    pub mission_s: f64,
    /// Seconds with the OBC up after deployment
    pub obc_up_s: f64,
    /// First radiated frame, s after deployment
    pub first_rf_s: Option<f64>,
    /// First burn, s after deployment
    pub first_burn_s: Option<f64>,
    /// Antennas confirmed deployed, s after deployment
    pub antennas_s: Option<f64>,
    /// Wings confirmed deployed, s after deployment
    pub wings_s: Option<f64>,
    /// Body rate below 0.5 °/s (and staying there 10 min), s after deployment
    pub detumbled_s: Option<f64>,
    /// Lowest true SOC after the first day
    pub soc_min_ops: f64,
    /// Lowest true SOC at all after deployment
    pub soc_min: f64,
    /// Battery temperature extremes after deployment, °C
    pub tbatt_min: f64,
    /// Hottest battery, °C
    pub tbatt_max: f64,
    /// Hottest PA, °C
    pub tpa_max: f64,
    /// Hottest Jetson, °C
    pub tjetson_max: f64,
    /// OBC board extremes, °C
    pub tobc_min: f64,
    /// Hottest OBC board, °C
    pub tobc_max: f64,
    /// Seconds charging was blocked by temperature
    pub charge_blocked_s: f64,
    /// Under-voltage cut-offs (bus dark)
    pub uv_trips: u32,
    /// Over-current cut-offs
    pub oc_trips: u32,
    /// Boots by cause
    pub boots_power_on: u32,
    /// Boots after a brownout
    pub boots_brownout: u32,
    /// Watchdog resets
    pub boots_watchdog: u32,
    /// Resets from single-event effects
    pub boots_seu: u32,
    /// Jetson latch-ups
    pub latchups: u32,
    /// Latch-ups the OBC's guard cleared (rail cut within its limits)
    pub latchups_cleared: u32,
    /// Longest a latch-up lasted before the rail was cut, s
    pub latchup_max_s: f64,
    /// Frames radiated
    pub frames: u64,
    /// Frames with Jetson text (passed the filter)
    pub ai_frames: u64,
    /// Longest key-down, s
    pub longest_key_s: f64,
    /// Highest keyed fraction over any 60 s window
    pub max_duty_60s: f64,
    /// Longest gap between frames while transmission was permitted, s
    pub max_beacon_gap_s: f64,
    /// Frames radiated while transmission was not permitted (TX-OFF, silence,
    /// inhibits) or after passivation
    pub frames_forbidden: u64,
    /// Seconds the loss-of-contact inhibit was active
    pub contact_inhibit_s: f64,
    /// Transponder enabled, s
    pub xpdr_on_s: f64,
    /// Transponder radiating, s
    pub xpdr_keyed_s: f64,
    /// Transponder radiating while transmission was not permitted or after
    /// passivation, s
    pub xpdr_forbidden_s: f64,
    /// Transponder radiating outside its power gates (deep eclipse, or the
    /// flight software's SOC below 54 %), s
    pub xpdr_out_of_gate_s: f64,
    /// Uplink carrier time while every gate held (the demand the
    /// transponder could serve), s
    pub xpdr_demand_s: f64,
    /// Of that, time the transponder repeated it, s
    pub xpdr_served_s: f64,
    /// Beacons heard on the ground
    pub beacons_heard: u64,
    /// Passes and passes with at least one beacon heard
    pub passes: u64,
    /// Passes with a beacon heard
    pub passes_heard: u64,
    /// Commands sent and executed onboard
    pub commands_sent: u32,
    /// Commands the flight software executed
    pub commands_executed: u32,
    /// Authentication failures
    pub auth_failures: u32,
    /// Flight-software fault-log entries by text
    pub faults: std::collections::BTreeMap<String, u64>,
    /// Jetson energy, Wh, and the beacons it produced that flew
    pub jetson_wh: f64,
    /// Jetson power-ons
    pub jetson_power_ons: u32,
    /// Energy generated and consumed after deployment, Wh
    pub gen_wh: f64,
    /// Energy to the loads, Wh
    pub load_wh: f64,
    /// Battery capacity at the start and the end of operations, Ah
    pub capacity_start_ah: f64,
    /// Capacity at the end
    pub capacity_end_ah: f64,
    /// Largest gauge error (|gauge − true| SOC)
    pub gauge_err_max: f64,
    /// Passivation fired, s after deployment
    pub passivated_s: Option<f64>,
    /// OBC permanently dark after passivation, s after deployment
    pub dark_s: Option<f64>,
    /// Charging after passivation, Ah
    pub charge_after_passivation_ah: f64,
    /// Re-entry, s after deployment
    pub reentry_s: Option<f64>,
    /// Altitude at the end of the design life, km
    pub eol_alt_km: f64,
    /// Radiation: total ionizing dose, krad(Si)
    pub tid_krad: f64,
    /// Mode transitions after deployment
    pub mode_changes: u64,
    /// Entries into SAFE and EMERGENCY
    pub safe_entries: u32,
    /// Entries into EMERGENCY
    pub emergency_entries: u32,
}

/// A check's outcome
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Verdict {
    /// Within the limits with margin
    Pass,
    /// Within the limits, little margin or not measurable
    Warning,
    /// Outside the limits: not nominal
    Anomaly,
}

/// One check
#[derive(Clone, Debug)]
pub struct Check {
    /// Short id
    pub id: &'static str,
    /// What it checks and against what
    pub title: &'static str,
    /// The measured value, as text
    pub value: String,
    /// The limit, as text
    pub limit: String,
    /// Outcome
    pub verdict: Verdict,
}

fn check(id: &'static str, title: &'static str, value: String, limit: &str, verdict: Verdict) -> Check {
    Check { id, title, value, limit: limit.into(), verdict }
}

fn days(s: f64) -> f64 {
    s / 86_400.0
}

/// Judge the run
pub fn evaluate(t: &Totals, orbits: &[Orbit], design_life_s: f64, passivation_planned: bool) -> Vec<Check> {
    use Verdict::*;
    let mut c = Vec::new();
    // Launch and early operations (CDS 2.4.4, 2.4.5)
    c.push(match t.first_burn_s {
        Some(s) => check(
            "deploy-timer",
            "First burn at least 30 min after separation (CDS 2.4.4)",
            format!("{:.1} min", s / 60.0),
            ">= 30 min",
            if s >= 1800.0 { Pass } else { Anomaly },
        ),
        None => {
            check("deploy-timer", "First burn at least 30 min after separation", "no burn".into(), ">= 30 min", Anomaly)
        }
    });
    c.push(match t.first_rf_s {
        Some(s) => check(
            "rf-silence",
            "No RF before 45 min after separation (CDS 2.4.5)",
            format!("{:.1} min", s / 60.0),
            ">= 45 min",
            if s >= 2700.0 { Pass } else { Anomaly },
        ),
        None => check("rf-silence", "First transmission", "never transmitted".into(), ">= 45 min", Anomaly),
    });
    for (id, title, at) in [
        ("antennas", "Both antennas deployed and sensed", t.antennas_s),
        ("wings", "Both solar wings deployed and sensed", t.wings_s),
    ] {
        c.push(match at {
            Some(s) => {
                check(id, title, format!("{:.1} min", s / 60.0), "within 2 h", if s < 7200.0 { Pass } else { Warning })
            }
            None => check(id, title, "not deployed".into(), "within 2 h", Anomaly),
        });
    }
    c.push(match t.detumbled_s {
        Some(s) => check(
            "detumble",
            "Detumbled (across +Y below 0.5 °/s, spin below twice its target) within 3 orbits (BASELINE §8)",
            format!("{:.1} h", s / 3600.0),
            "<= 4.7 h",
            if s <= 3.0 * 5670.0 { Pass } else { Anomaly },
        ),
        None => {
            check("detumble", "Detumbled (across +Y below 0.5 °/s) (BASELINE §8)", "never".into(), "<= 4.7 h", Anomaly)
        }
    });

    // Power (BASELINE §5)
    let max_dod = orbits.iter().filter(|o| o.start_s > 86_400.0).map(|o| o.dod).fold(0.0, f64::max);
    c.push(check(
        "dod",
        "Depth of discharge per eclipse (BASELINE §5: <= 10 % nominal, <= 30 % contingency)",
        format!("{:.1} %", max_dod * 100.0),
        "<= 10 %",
        if max_dod <= 0.10 {
            Pass
        } else if max_dod <= 0.30 {
            Warning
        } else {
            Anomaly
        },
    ));
    c.push(check(
        "soc-min",
        "Lowest state of charge in operations (SAFE below 25 %)",
        format!("{:.1} %", t.soc_min_ops * 100.0),
        ">= 40 %",
        if t.soc_min_ops >= 0.40 {
            Pass
        } else if t.soc_min_ops >= 0.25 {
            Warning
        } else {
            Anomaly
        },
    ));
    let margin = if t.load_wh > 0.0 { t.gen_wh / t.load_wh - 1.0 } else { 0.0 };
    c.push(check(
        "energy-margin",
        "Generation over consumption, whole mission (requirement >= 20 % margin on the orbit average)",
        format!("generated {:.0} Wh, used {:.0} Wh", t.gen_wh, t.load_wh),
        "balanced",
        if margin >= -0.02 { Pass } else { Anomaly },
    ));
    c.push(check(
        "uv-trips",
        "Battery under-voltage cut-offs (the bus going dark)",
        t.uv_trips.to_string(),
        "0 before passivation",
        if t.uv_trips == 0 { Pass } else { Anomaly },
    ));
    c.push(check(
        "oc-trips",
        "BMS over-current cut-offs",
        t.oc_trips.to_string(),
        "0",
        if t.oc_trips == 0 { Pass } else { Anomaly },
    ));
    c.push(check(
        "safe-entries",
        "Entries into SAFE or EMERGENCY after commissioning, before passivation",
        format!("{} (EMERGENCY {})", t.safe_entries, t.emergency_entries),
        "0",
        if t.emergency_entries > 0 {
            Anomaly
        } else if t.safe_entries > 0 {
            Warning
        } else {
            Pass
        },
    ));
    let worst_chatter = orbits.iter().map(|o| o.mode_changes).max().unwrap_or(0);
    c.push(check(
        "mode-chatter",
        "Mode changes in any one orbit (hysteresis)",
        worst_chatter.to_string(),
        "<= 4",
        if worst_chatter <= 4 { Pass } else { Anomaly },
    ));
    let fade = if t.capacity_start_ah > 0.0 { 1.0 - t.capacity_end_ah / t.capacity_start_ah } else { 0.0 };
    c.push(check(
        "capacity-fade",
        "Battery capacity fade over the design life",
        format!("{:.1} %", fade * 100.0),
        "<= 20 %",
        if fade <= 0.20 { Pass } else { Anomaly },
    ));
    c.push(check(
        "gauge",
        "Fuel-gauge SOC error",
        format!("{:.1} %", t.gauge_err_max * 100.0),
        "<= 5 %",
        if t.gauge_err_max <= 0.05 { Pass } else { Warning },
    ));

    // Thermal (BASELINE §5, §8)
    c.push(check(
        "tbatt",
        "Battery temperature",
        format!("{:.1} to {:.1} °C", t.tbatt_min, t.tbatt_max),
        "-10 to +45 °C",
        if t.tbatt_min >= 0.0 && t.tbatt_max <= 45.0 {
            Pass
        } else if t.tbatt_min >= -10.0 && t.tbatt_max <= 50.0 {
            Warning
        } else {
            Anomaly
        },
    ));
    let blocked_frac = if t.mission_s > 0.0 { t.charge_blocked_s / t.mission_s } else { 0.0 };
    c.push(check(
        "charge-window",
        "Time charging was wanted but refused by the 0–45 °C window (SYS-602)",
        format!("{:.2} % ({:.1} h)", blocked_frac * 100.0, t.charge_blocked_s / 3600.0),
        "< 1 %",
        if blocked_frac < 0.01 { Pass } else { Anomaly },
    ));
    c.push(check(
        "tpa",
        "PA temperature (downlink holds above 70 °C)",
        format!("{:.1} °C", t.tpa_max),
        "<= 70 °C",
        if t.tpa_max <= 60.0 {
            Pass
        } else if t.tpa_max <= 70.0 {
            Warning
        } else {
            Anomaly
        },
    ));
    c.push(check(
        "tjetson",
        "Jetson module temperature (guard limit 95 °C junction, 70 °C spreader)",
        format!("{:.1} °C", t.tjetson_max),
        "<= 70 °C",
        if t.tjetson_max <= 70.0 { Pass } else { Anomaly },
    ));
    c.push(check(
        "tobc",
        "OBC board temperature (operating -40 to +85 °C)",
        format!("{:.1} to {:.1} °C", t.tobc_min, t.tobc_max),
        "-20 to +60 °C",
        if t.tobc_min >= -20.0 && t.tobc_max <= 60.0 { Pass } else { Warning },
    ));

    // Radio (SRS-P002, §7)
    c.push(check(
        "keydown",
        "Longest key-down (SRS-P002: 5 s)",
        format!("{:.2} s", t.longest_key_s),
        "<= 5 s",
        if t.longest_key_s <= 5.0 { Pass } else { Anomaly },
    ));
    c.push(check(
        "duty",
        "Downlink duty over 60 s (25 %)",
        format!("{:.1} %", t.max_duty_60s * 100.0),
        "<= 25 %",
        if t.max_duty_60s <= 0.25 + 1e-6 { Pass } else { Anomaly },
    ));
    c.push(check(
        "forbidden-tx",
        "Frames radiated while not permitted (TX-OFF, silence, inhibits, after passivation)",
        t.frames_forbidden.to_string(),
        "0",
        if t.frames_forbidden == 0 { Pass } else { Anomaly },
    ));
    // The FM transponder (BASELINE §2, §5)
    c.push(check(
        "xpdr-forbidden",
        "Transponder radiating while not permitted (TX-OFF, silence, inhibits, after passivation)",
        format!("{:.0} s", t.xpdr_forbidden_s),
        "0",
        if t.xpdr_forbidden_s == 0.0 { Pass } else { Anomaly },
    ));
    let out_share = if t.xpdr_keyed_s > 0.0 { t.xpdr_out_of_gate_s / t.xpdr_keyed_s } else { 0.0 };
    c.push(check(
        "xpdr-gates",
        "Transponder radiating in eclipse or below 55 % SOC (share of its keyed time)",
        format!("{:.2} % ({:.0} s)", out_share * 100.0, t.xpdr_out_of_gate_s),
        "<= 0.5 %",
        if out_share <= 0.005 { Pass } else { Anomaly },
    ));
    let worst_xpdr = orbits.iter().filter(|o| o.len_s > 3000.0).map(|o| o.xpdr_keyed_s / o.len_s).fold(0.0, f64::max);
    c.push(check(
        "xpdr-duty",
        "Transponder keyed share of an orbit (25 % of 100 min)",
        format!("{:.1} %", worst_xpdr * 100.0),
        "<= 26.5 %",
        if worst_xpdr <= 0.265 { Pass } else { Anomaly },
    ));
    let served = if t.xpdr_demand_s > 0.0 { t.xpdr_served_s / t.xpdr_demand_s } else { 0.0 };
    let mission_days = (t.mission_s / 86_400.0).max(1.0 / 24.0);
    c.push(check(
        "xpdr-service",
        "Uplink carrier time repeated while the gates held; transponder time a day",
        format!(
            "{:.1} %; {:.0} min/day on, {:.0} min/day keyed",
            served * 100.0,
            t.xpdr_on_s / mission_days / 60.0,
            t.xpdr_keyed_s / mission_days / 60.0
        ),
        ">= 70 %",
        if t.xpdr_demand_s == 0.0 {
            Warning
        } else if served >= 0.7 {
            Pass
        } else {
            Warning
        },
    ));
    c.push(check(
        "beacon-gap",
        "Longest gap between beacons while permitted",
        format!("{:.0} s", t.max_beacon_gap_s),
        "<= 600 s",
        if t.max_beacon_gap_s <= 600.0 { Pass } else { Anomaly },
    ));
    let heard = if t.passes > 0 { t.passes_heard as f64 / t.passes as f64 } else { 0.0 };
    c.push(check(
        "passes-heard",
        "Passes (above 5°) with at least one beacon decoded",
        format!("{:.1} % of {}", heard * 100.0, t.passes),
        ">= 80 %",
        if heard >= 0.8 { Pass } else { Warning },
    ));
    c.push(check(
        "contact",
        "Loss-of-contact inhibit (should never trigger with the contact plan)",
        format!("{:.1} h", t.contact_inhibit_s / 3600.0),
        "0",
        if t.contact_inhibit_s == 0.0 { Pass } else { Anomaly },
    ));
    c.push(check(
        "commands",
        "Telecommands executed of those sent; authentication failures",
        format!("{} of {}, {} auth failures", t.commands_executed, t.commands_sent, t.auth_failures),
        ">= 90 % executed, 0 failures",
        if t.auth_failures > 0 {
            Anomaly
        } else if t.commands_sent > 0 && f64::from(t.commands_executed) >= 0.9 * f64::from(t.commands_sent) {
            Pass
        } else {
            Warning
        },
    ));

    // Resets and payload
    c.push(check(
        "watchdog",
        "Watchdog resets",
        t.boots_watchdog.to_string(),
        "0",
        if t.boots_watchdog == 0 { Pass } else { Anomaly },
    ));
    c.push(check(
        "brownouts",
        "Brownout reboots before passivation",
        t.boots_brownout.to_string(),
        "0",
        if t.boots_brownout == 0 { Pass } else { Anomaly },
    ));
    c.push(check(
        "seu-recovery",
        "Single-event resets recovered; Jetson latch-ups cleared by the rail cut within 10 s",
        format!(
            "{} resets, {} of {} latch-ups cleared (longest {:.1} s)",
            t.boots_seu, t.latchups_cleared, t.latchups, t.latchup_max_s
        ),
        "all latch-ups cleared",
        if t.latchups_cleared == t.latchups { Pass } else { Anomaly },
    ));
    let per_beacon = if t.ai_frames > 0 { t.jetson_wh / t.ai_frames as f64 } else { f64::INFINITY };
    c.push(check(
        "ai-energy",
        "Jetson energy per accepted beacon (BASELINE §6 gate: <= 0.1 Wh)",
        if t.ai_frames > 0 {
            format!("{:.3} Wh ({:.0} Wh, {} beacons)", per_beacon, t.jetson_wh, t.ai_frames)
        } else {
            format!("no AI beacons ({:.1} Wh used)", t.jetson_wh)
        },
        "<= 0.1 Wh",
        if per_beacon <= 0.1 { Pass } else { Anomaly },
    ));
    c.push(check(
        "uptime",
        "OBC availability after deployment",
        format!("{:.3} %", if t.mission_s > 0.0 { 100.0 * t.obc_up_s / t.mission_s } else { 0.0 }),
        ">= 99 %",
        if t.mission_s > 0.0 && t.obc_up_s / t.mission_s >= 0.99 { Pass } else { Anomaly },
    ));

    // End of life (47 CFR 97.207(g), FCC 5-year rule)
    if passivation_planned && t.mission_s >= design_life_s {
        c.push(match t.passivated_s {
            Some(_) => check(
                "passivation",
                "EOL passivation: charge path open, transmitters silent",
                format!("charge after passivation {:.3} Ah", t.charge_after_passivation_ah),
                "0 Ah",
                if t.charge_after_passivation_ah < 1e-3 { Pass } else { Anomaly },
            ),
            None => check("passivation", "EOL passivation", "not fired".into(), "at end of life", Anomaly),
        });
    }
    if let Some(s) = t.reentry_s {
        let after = days(s - design_life_s) / 365.25;
        c.push(check(
            "reentry",
            "Re-entry no later than 5 years after the end of the mission (FCC)",
            format!("{:.2} years after the design life ({:.2} after deployment)", after, days(s) / 365.25),
            "<= 5 years",
            if after <= 5.0 { Pass } else { Anomaly },
        ));
    }
    c
}

/// The checks and the totals as `summary.json`
pub fn summary_json(checks: &[Check], t: &Totals, extra: &[(&str, String)]) -> String {
    let mut s = String::from("{\n  \"checks\": [\n");
    for (i, k) in checks.iter().enumerate() {
        let _ = writeln!(
            s,
            "    {{\"id\": {}, \"title\": {}, \"value\": {}, \"limit\": {}, \"verdict\": \"{:?}\"}}{}",
            json_str(k.id),
            json_str(k.title),
            json_str(&k.value),
            json_str(&k.limit),
            k.verdict,
            if i + 1 < checks.len() { "," } else { "" }
        );
    }
    s.push_str("  ],\n  \"faults\": {");
    for (i, (k, v)) in t.faults.iter().enumerate() {
        let _ = write!(s, "{}\n    {}: {}", if i > 0 { "," } else { "" }, json_str(k), v);
    }
    s.push_str("\n  },\n  \"totals\": {\n");
    let _ = write!(s, "    \"debug\": {}", json_str(&format!("{t:?}")));
    for (k, v) in extra {
        let _ = write!(s, ",\n    {}: {}", json_str(k), v);
    }
    s.push_str("\n  }\n}\n");
    s
}
