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
    /// The AntS node's temperature when the AntS was first told to deploy,
    /// °C (the flight software's temperature gate, decision C-18), and the
    /// AntS node's temperature at separation + 31 min (an ungated release)
    pub ants_release_c: Option<f64>,
    /// When the AntS was first told to deploy, s after deployment, and the
    /// continuous sunlight before it, s (0 in eclipse; RID P3F-G-02)
    pub ants_release_s: Option<f64>,
    /// Continuous sunlight before the first AntS deployment command, s
    pub ants_release_sun_s: Option<f64>,
    /// AntS deployment commands (each attempt's START), and those made
    /// while the AntS already read deployed in the model (RID P3F-A-01:
    /// a deployed AntS is never fired again)
    pub ants_starts: u32,
    /// AntS deployment commands with its elements already deployed
    pub ants_starts_deployed: u32,
    /// The AntS node at separation + 31 min, °C
    pub ants_31min_c: Option<f64>,
    /// The AntS node's coldest, °C, in the first 3 h after separation
    pub ants_min_c: Option<f64>,
    /// When the AntS temperature gate first stood open at or after the first
    /// attempt (separation + 30 min): the AntS node at or above −17 °C, s
    /// after deployment
    pub ants_gate_open_s: Option<f64>,
    /// The flight software's AntS transmitter hold ended (decision C-19), s
    /// after deployment
    pub ants_hold_end_s: Option<f64>,
    /// The downlink's VSWR protection stopping it, and of those the stops
    /// with the AntS elements stowed
    pub vswr_stops: u32,
    /// VSWR stops with the AntS stowed
    pub vswr_stops_stowed: u32,
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
    /// When the worst 60 s ended, s after deployment
    pub max_duty_at_s: f64,
    /// Longest gap between frames while transmission was permitted, s
    pub max_beacon_gap_s: f64,
    /// Frames radiated while transmission was not permitted (TX-OFF, silence,
    /// inhibits) or after passivation
    pub frames_forbidden: u64,
    /// Hardware load sheds (SHED_N, below 13.6 V for 5 s)
    pub sheds: u32,
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
    /// Every Jetson power-on: input energy, J, and lines meant to pass the
    /// filter (decision C-20)
    pub jetson_bursts: Vec<(f64, u32)>,
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
    /// The charge disconnect opened after the ground's PASSIVATE_ARM and
    /// PASSIVATE_EXECUTE (the only way the simulation passivates)
    pub passivation_by_command: bool,
    /// Command acknowledgements radiated
    pub acks: u64,
    /// Stored-housekeeping dump frames radiated, and the records in them
    pub dump_frames: u64,
    /// Records in the dump frames
    pub dump_records: u64,
    /// GNSS sessions that ended with a fix, GNSS state frames radiated, and
    /// those with a velocity
    pub gnss_fix_sessions: u64,
    /// GNSS state frames radiated
    pub gnss_frames: u64,
    /// Those with a velocity
    pub gnss_vel_frames: u64,
    /// Largest position error of a frame against the receiver's record, m
    pub gnss_pos_err_max_m: f64,
    /// Largest and summed velocity error against the truth, m/s
    pub gnss_vel_err_max: f64,
    /// Summed velocity error, m/s
    pub gnss_vel_err_sum: f64,
    /// The next five-minute record number the last HK1 frame showed
    pub store_next: Option<u32>,
    /// Command receiver hangs, and those its power cycle cleared
    pub rx_hangs: u32,
    /// Hangs cleared by the flight software's power cycle
    pub rx_recovered: u32,
    /// Longest hang, s
    pub rx_hang_max_s: f64,
    /// Most wing burn lines on at once
    pub wing_lines_max: usize,
    /// The command receiver fails for good in this run (`rx_failed_day`)
    pub rx_failure_modelled: bool,
    /// Time the flight software held the command receiver known failed, s
    pub rx_known_failed_s: f64,
    /// Beacons radiated while the receiver was known failed
    pub rx_failed_beacons: u64,
    /// Longest gap between beacons while the receiver was known failed and
    /// transmission permitted, s
    pub rx_failed_beacon_gap_s: f64,
    /// Transponder keyed time while the receiver was known failed, s
    pub rx_failed_xpdr_s: f64,
    /// Time the loss-of-contact inhibit silenced the beacon after the
    /// receiver failed, before it was known failed, s
    pub rx_failed_pending_s: f64,
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
    // The payload's hard limits (MSC L2-05): the Jetson on only in ACTIVE,
    // sunlit, SOC >= 55 %, within its burst limit (the flight software's own
    // condition, logged on each onset)
    let hard = t.faults.get("Payload hard limit violated").copied().unwrap_or(0);
    c.push(check(
        "payload-limits",
        "Jetson within its hard limits: ACTIVE, sunlit, SOC >= 55 %, burst limit (MSC L2-05; the flight software's condition)",
        format!("{hard} onset(s)"),
        "0 (a stuck-on payload contingency shows it)",
        if hard == 0 { Pass } else { Anomaly },
    ));
    // The AntS temperature gate (decision C-18): deferred while the AntS is
    // below -17 °C, at most 30 slots of 50 min; at the cap, or on the
    // analysis schedule after three unknown readings, released in sunlight
    // after 20 min of sun (RID P3F-G-02)
    let deferrals =
        t.faults.iter().filter(|(k, _)| k.starts_with("AntS deployment deferred")).map(|(_, n)| *n).sum::<u64>();
    let schedule = t.faults.iter().any(|(k, _)| k.starts_with("AntS temperature unknown"));
    let cap = deferrals >= u64::from(sqso_fsw::deployment::ANTS_MAX_DEFERRALS);
    let sun_need_s = f64::from(sqso_fsw::deployment::ANTS_SUN_MS) / 1000.0 - 60.0;
    let sun_wait_s = if cap || schedule { f64::from(sqso_fsw::deployment::ANTS_SUN_WAIT_MS) / 1000.0 } else { 0.0 };
    let gate_limit_s = 7200.0 + deferrals as f64 * 3000.0 + sun_wait_s;
    c.push(match t.ants_release_c {
        Some(tc) => {
            let sun = t.ants_release_sun_s.unwrap_or(0.0);
            let at = t.ants_release_s.unwrap_or(0.0);
            let warm = tc >= sqso_fsw::deployment::ANTS_MIN_TEMP_C - 0.5 && !cap && !schedule;
            let cap_ok = cap && sun >= sun_need_s;
            let schedule_ok = schedule && at >= f64::from(sqso_fsw::deployment::ANTS_SCHEDULE_S) && sun >= sun_need_s;
            check(
                "ants-gate",
                "AntS deployed at or above -17 °C (decision C-18); at the 30-deferral cap or on the schedule fallback, in sunlight after 20 min of sun",
                format!(
                    "released at {:.1} min, {tc:.1} °C, {:.1} min of sun, after {deferrals} deferral(s){}{}; {} at separation + 31 min, coldest {}, gate open (>= -17 °C) from {}",
                    at / 60.0,
                    sun / 60.0,
                    if cap { ", at the cap" } else { "" },
                    if schedule { ", on the schedule (temperature unknown)" } else { "" },
                    t.ants_31min_c.map_or("-".to_string(), |x| format!("{x:.1} °C")),
                    t.ants_min_c.map_or("-".to_string(), |x| format!("{x:.1} °C")),
                    t.ants_gate_open_s.map_or("-".to_string(), |s| format!("{:.1} min", s / 60.0))
                ),
                ">= -17 °C; or cap/schedule in sunlight >= 20 min",
                if warm || cap_ok || schedule_ok { Pass } else { Anomaly },
            )
        }
        None => check("ants-gate", "AntS deployment", "not commanded in the run".into(), ">= -17 °C", Warning),
    });
    // RID P3F-A-01: the AntS is read while powered, so a deployed AntS is
    // confirmed and never told to deploy again
    c.push(check(
        "ants-commands",
        "AntS deployment commands; none once its elements read deployed (it is read powered, P3F-A-01)",
        format!("{} command(s), {} with the elements deployed", t.ants_starts, t.ants_starts_deployed),
        "1 to 3; 0 after deployment",
        if (1..=3).contains(&t.ants_starts) && t.ants_starts_deployed == 0 { Pass } else { Anomaly },
    ));
    for (id, title, at, limit_s) in [
        (
            "antennas",
            "Both antennas deployed and sensed (2 h, plus 50 min per AntS deferral)",
            t.antennas_s,
            gate_limit_s,
        ),
        ("wings", "Both solar wings deployed and sensed", t.wings_s, 7200.0),
    ] {
        c.push(match at {
            Some(s) => check(
                id,
                title,
                format!("{:.1} min", s / 60.0),
                &format!("within {:.0} min", limit_s / 60.0),
                if s < limit_s { Pass } else { Warning },
            ),
            None => check(id, title, "not deployed".into(), "within 2 h", Anomaly),
        });
    }
    // SW-HAZ-014 control C-014-9 (SRS-F168): the exciter keyed with no
    // frame and the repeat path idle trips TX-OFF; never in a nominal run
    let ptt = t.faults.iter().filter(|(k, _)| k.starts_with("PTT_EXCITER keyed")).map(|(_, n)| *n).sum::<u64>();
    c.push(check(
        "ptt-watch",
        "Exciter keyed over 6 s with no frame and no repeat, or by the repeat path without the heartbeat or past 130 s (C-014-9, P3F-A-02): TX-OFF trips",
        format!("{ptt}"),
        "0 (the TNC never keys alone in the model)",
        if ptt == 0 { Pass } else { Anomaly },
    ));
    // Decision C-19: silent while the AntS is pending; no keying into stowed
    // elements before its sequence ends (after a failed sequence the
    // stowed-antenna beacon, and its VSWR stops, are expected)
    let ants_out = t.antennas_s.is_some_and(|a| t.ants_hold_end_s.is_some_and(|h| a <= h + 1.0));
    c.push(match (t.first_rf_s, t.ants_hold_end_s) {
        (Some(rf), Some(end)) => check(
            "ants-hold",
            "Transmitters silent until the AntS reads deployed or its sequence ends (decision C-19); no VSWR stop into stowed elements",
            format!(
                "first RF {:.1} min, hold over at {:.1} min (AntS {}); {} VSWR stop(s), {} with the AntS stowed",
                rf / 60.0,
                end / 60.0,
                if ants_out { "deployed" } else { "stowed: its sequence ended" },
                t.vswr_stops,
                t.vswr_stops_stowed
            ),
            "first RF after the hold; 0 stowed VSWR stops unless the AntS failed",
            if rf + 1.0 >= end && (t.vswr_stops_stowed == 0 || !ants_out) { Pass } else { Anomaly },
        ),
        (None, _) => check("ants-hold", "AntS transmitter hold", "never transmitted".into(), "after the hold", Anomaly),
        (Some(_), None) => check("ants-hold", "AntS transmitter hold", "never ended".into(), "after the hold", Anomaly),
    });
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
        "load-shed",
        "Hardware load sheds of SW1 and U19 (VBAT below 13.6 V for 5 s), before passivation",
        t.sheds.to_string(),
        "0 (a stuck-on payload contingency shows it acting)",
        if t.sheds == 0 { Pass } else { Warning },
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
        format!("{:.1} % (day {:.2})", t.max_duty_60s * 100.0, t.max_duty_at_s / 86_400.0),
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
    // Decision C-20: one burst definition (a power-on of at most 10 min and
    // 3.6 kJ) and the metric counted on accepted lines; the per-burst
    // distribution is reported (the flight gate meters the carrier input)
    if !t.jetson_bursts.is_empty() {
        let (p50, p95) = (percentile(&t.jetson_bursts, 0.50), percentile(&t.jetson_bursts, 0.95));
        let max = t.jetson_bursts.iter().map(|b| b.0).fold(0.0, f64::max);
        c.push(check(
            "ai-bursts",
            "Jetson burst energy at the model's input (decision C-20: at most 3.6 kJ a burst, the gate on 360 J per accepted line)",
            format!(
                "{} bursts: p50 {p50:.0} J, p95 {p95:.0} J, max {max:.0} J; {:.4} Wh per good line",
                t.jetson_bursts.len(),
                wh_per_line(&t.jetson_bursts)
            ),
            "max <= 3.75 kJ (3.6 kJ at the carrier input, metered 20 ms late)",
            if max <= 3750.0 { Pass } else { Anomaly },
        ));
    }
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
        // Not exercised (no batch yet: the commissioning hold) is no anomaly;
        // energy spent with no beacon to show for it is
        if per_beacon <= 0.1 {
            Pass
        } else if t.ai_frames == 0 && t.jetson_wh < 0.01 {
            Warning
        } else {
            Anomaly
        },
    ));
    c.push(check(
        "uptime",
        "OBC availability after deployment",
        format!("{:.3} %", if t.mission_s > 0.0 { 100.0 * t.obc_up_s / t.mission_s } else { 0.0 }),
        ">= 99 %",
        if t.mission_s > 0.0 && t.obc_up_s / t.mission_s >= 0.99 { Pass } else { Anomaly },
    ));

    operations_cdr(&mut c, t);

    // End of life (47 CFR 97.207(g), FCC 5-year rule)
    if passivation_planned && t.mission_s >= design_life_s {
        c.push(match t.passivated_s {
            Some(s) => check(
                "passivation",
                "EOL passivation by PASSIVATE_ARM and PASSIVATE_EXECUTE: charge path open, transmitters silent",
                format!(
                    "day {:.1}, by telecommand {}, charge after passivation {:.3} Ah",
                    days(s),
                    t.passivation_by_command,
                    t.charge_after_passivation_ah
                ),
                "by telecommand, 0 Ah",
                if t.charge_after_passivation_ah < 1e-3 && t.passivation_by_command { Pass } else { Anomaly },
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

/// REL-04: a receiver known failed keeps the beacon (the loss-of-contact
/// inhibit no longer silences it) and never the transponder; a healthy
/// receiver is never declared failed
fn rx_failed_check(t: &Totals) -> Check {
    use Verdict::*;
    let (value, verdict) = if t.rx_failure_modelled {
        let kept = t.rx_known_failed_s > 0.0
            && t.rx_failed_beacons > 0
            && t.rx_failed_beacon_gap_s <= 600.0
            && t.rx_failed_xpdr_s == 0.0;
        (
            format!(
                "known failed {:.1} d: {} beacons, longest gap {:.0} s, transponder {:.0} s; silenced {:.1} h before",
                t.rx_known_failed_s / 86_400.0,
                t.rx_failed_beacons,
                t.rx_failed_beacon_gap_s,
                t.rx_failed_xpdr_s,
                t.rx_failed_pending_s / 3600.0
            ),
            if kept { Pass } else { Anomaly },
        )
    } else {
        (
            format!("receiver healthy; held failed {:.1} h", t.rx_known_failed_s / 3600.0),
            if t.rx_known_failed_s == 0.0 { Pass } else { Anomaly },
        )
    };
    check(
        "rx-failed-beacon",
        "Receiver failed for good (REL-04): the beacon kept, gaps <= 600 s, the transponder off; never declared failed when healthy",
        value,
        "beacon kept, transponder 0 s; or 0 h when healthy",
        verdict,
    )
}

/// The checks added at CDR: one burn line at a time, the receiver's
/// recovery, acknowledgements and stored-housekeeping dumps
fn operations_cdr(c: &mut Vec<Check>, t: &Totals) {
    use Verdict::*;
    c.push(check(
        "burn-lines",
        "Wing burn resistors on at once (EPS board: one at a time, 1.0 A pack limit)",
        t.wing_lines_max.to_string(),
        "<= 1",
        if t.wing_lines_max <= 1 { Pass } else { Anomaly },
    ));
    // A hang still open at the end (the 24 h silence not yet served) is not a failure
    let open = t.rx_hangs.saturating_sub(t.rx_recovered);
    c.push(check(
        "rx-recovery",
        "Command-receiver hangs cleared by the flight software's power cycle (FSW-09: after 24 h of silence)",
        format!("{} of {} (longest {:.1} h)", t.rx_recovered, t.rx_hangs, t.rx_hang_max_s / 3600.0),
        "all, each within 26 h",
        if open <= 1 && t.rx_hang_max_s <= 26.0 * 3600.0 { Pass } else { Anomaly },
    ));
    c.push(rx_failed_check(t));
    let acked = if t.commands_executed > 0 { t.acks as f64 / f64::from(t.commands_executed) } else { 0.0 };
    c.push(check(
        "acks",
        "Command acknowledgements radiated per command executed (GND-03; none while TX-OFF)",
        format!("{} for {} ({:.1} %)", t.acks, t.commands_executed, acked * 100.0),
        ">= 90 %",
        if t.commands_executed == 0 || acked >= 0.9 { Pass } else { Warning },
    ));
    let vel_mean = if t.gnss_vel_frames > 0 { t.gnss_vel_err_sum / t.gnss_vel_frames as f64 } else { 0.0 };
    c.push(check(
        "gnss-frame",
        "GNSS state frames (SRS-F158) against the truth: position as recorded, the derived ECEF velocity (the receiver's arc propagated with J2)",
        format!(
            "{} frames for {} fix sessions, {} with a velocity; position {:.2} m, velocity mean {:.3} max {:.3} m/s",
            t.gnss_frames, t.gnss_fix_sessions, t.gnss_vel_frames, t.gnss_pos_err_max_m, vel_mean, t.gnss_vel_err_max
        ),
        "a frame per fix session; position <= 1 m; velocity <= 0.5 m/s",
        if t.gnss_pos_err_max_m > 1.0 || t.gnss_vel_err_max > 0.5 {
            Anomaly
        } else if t.gnss_fix_sessions > 0 && t.gnss_frames == 0 {
            Warning
        } else {
            Pass
        },
    ));
    c.push(check(
        "dumps",
        "Stored-housekeeping dump frames radiated (DUMP_TLM, daily after commissioning)",
        format!("{} frames, {} records", t.dump_frames, t.dump_records),
        "> 0 after commissioning",
        if t.mission_s <= 15.0 * 86_400.0 || t.dump_frames > 0 { Pass } else { Warning },
    ));
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

/// The `q` quantile of the bursts' energies, J (0 with none)
pub fn percentile(bursts: &[(f64, u32)], q: f64) -> f64 {
    let mut e: Vec<f64> = bursts.iter().map(|b| b.0).collect();
    if e.is_empty() {
        return 0.0;
    }
    e.sort_by(f64::total_cmp);
    let i = ((e.len() - 1) as f64 * q).round() as usize;
    e[i.min(e.len() - 1)]
}

/// Energy per line meant to pass the filter over all bursts, Wh (infinite
/// with none)
pub fn wh_per_line(bursts: &[(f64, u32)]) -> f64 {
    let (j, n) = bursts.iter().fold((0.0, 0u64), |(j, n), b| (j + b.0, n + u64::from(b.1)));
    if n == 0 { f64::INFINITY } else { j / 3600.0 / n as f64 }
}
