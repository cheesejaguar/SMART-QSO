//! The mission: integration and key load on the bench, storage in the
//! dispenser, separation, launch and early operations, the design life,
//! end-of-life passivation, and the orbit's decay to re-entry.
//!
//! Each step of operations: the environment (orbit, Sun, Earth's shadow,
//! magnetic field, atmosphere), the attitude under the flight software's
//! magnetorquer commands and the disturbance torques, the array, the EPS
//! and the pack, the thermal network, the ground segment (passes, beacon
//! reception, telecommands), then one pass of the flight software's main
//! loop on the OBC's clock, then the hardware it drives (radio, Jetson,
//! deployment mechanisms), the radiation environment's single-event
//! effects, and the records.

use std::path::Path;

use sqso_fsw::eps_telemetry::Telemetry;

use crate::checks::{self, Totals, Verdict};
use crate::config::Config;
use crate::env::attitude::{self, Attitude, Disturbance};
use crate::env::battery::Pack;
use crate::env::math::{OMEGA_EARTH, Rng, V3};
use crate::env::orbit::Orbit;
use crate::env::solar::{Array, ArrayInputs};
use crate::env::thermal::{self, Thermal, ThermalInputs};
use crate::env::{atmosphere, earth, eclipse, ground, link, magfield, radiation, sun, time};
use crate::fsw::{Fsw, ResetCause, Snapshot, adc, pin};
use crate::hw::deploy::{self, Deployables};
use crate::hw::eps::{Eps, Loads};
use crate::hw::radio::{self, Radio};
use crate::jetson::Jetson;
use crate::metrics::{Orbits, Sample};
use crate::ops::{self, Uplink, cmd};
use crate::qso;
use crate::recorder::{Logs, Table};

/// OBC, FRAM and sensors on 3V3_AO, W (BASELINE §5 housekeeping)
const OBC_W: f64 = 0.165;
/// ADCS sensors with the ADCS rail on, W
const ADCS_SENSORS_W: f64 = 0.12;
/// The rods and their drive (SQSO-OBC-BRD-001 B §10): the CR0002's gain,
/// A·m² per A, its resistance, Ω, the bridges' efficiency and their awake
/// current with MTQ_ARM high (three DRV8876, 3 mA each), A. The coils run
/// from VBAT (SW11): the supply power is I² R / efficiency per rod
const ROD_GAIN: f64 = 2.3;
const ROD_OHMS: f64 = 51.0;
const BRIDGE_EFF: f64 = 0.97;
const BRIDGES_AWAKE_A: f64 = 0.009;
/// Battery heater, Ω (BASELINE §8: about 207 Ω, 0.70 W at 12.0 V to 1.30 W across the
/// bus; THM-15): its power is V²/R at the pack voltage
const HEATER_OHMS: f64 = 207.0;
/// The end-of-life disconnect's heater, Ω: F1 ITV4030L1412NR, 11.2–20.0 Ω
/// (Littelfuse; SQSO-EPS-BRD-001 D §7), 15.6 Ω nominal (`eol_heater_ohms`)
pub const EOL_HEATER_OHMS: f64 = 15.6;
/// The fuse's heater firing window, W (the data sheet's 6–35 W)
pub const EOL_FIRE_WINDOW_W: (f64, f64) = (6.0, 35.0);
/// The thermal cut-off opens after this long on its heater inside the
/// window, s (the data sheet's cut within 1 min; inside the flight
/// software's 60 s minimum hold)
const EOL_OPEN_S: f64 = 30.0;
/// The hardware load shed (SQSO-EPS-BRD-001 E §8.4, SQSO-ICD-001 1.6):
/// SHED_N low after VBAT below 13.6 V for 5 s, high again above 14.4 V; it
/// gates SW1 and the Jetson eFuse U19 (not the PA: RID SMA-B05),
/// independent of the flight software
pub const SHED_V: (f64, f64) = (13.6, 14.4);
const SHED_DELAY_S: f64 = 5.0;

/// The fuse's heater power at `vbat` through `ohms`, W, and whether it is
/// inside the firing window
pub fn eol_heater(vbat: f64, ohms: f64) -> (f64, bool) {
    let w = vbat * vbat / ohms;
    (w, (EOL_FIRE_WINDOW_W.0..=EOL_FIRE_WINDOW_W.1).contains(&w))
}
/// Status LED, W
const LED_W: f64 = 0.005;
/// Mass, kg (BASELINE §4: lifetime analyses use 1.6 kg)
const MASS_KG: f64 = 1.6;
/// Extra input power of a Jetson single-event latch-up, W (a parasitic
/// path of 1–2 A on the 5 V rail; the OBC's guard trips above 8 W). The
/// latched module hangs, still drawing its idle supply current
/// ([`crate::jetson::IDLE_W`]) as well
const LATCHUP_EXTRA_W: f64 = 6.0;
/// A latch-up the rail cut within this long is cleared, s (the guard's
/// response: two passes of its 1 s power sample, with margin)
const LATCHUP_CLEAR_S: f64 = 10.0;
/// The operational state ACTIVE (`state_machine.h`)
const STATE_ACTIVE: u32 = 4;
/// Mean length of one over on the transponder, s
const OVER_S: f64 = 20.0;
/// The repeat path's key-down timer, s (SQSO-RF-BRD-001 §8.2)
const KEY_DOWN_TIMER_S: f64 = 120.0;

/// Rate across +Y that counts as detumbled (captured), °/s, held for a minute
const DETUMBLED_DPS: f64 = 0.5;

/// The time-series columns
const TS_HEADER: &[&str] = &[
    "t_s",
    "alt_km",
    "lat_deg",
    "lon_deg",
    "beta_deg",
    "illum",
    "rate_dps",
    "sun_angle_deg",
    "array_mpp_w",
    "array_w",
    "load_w",
    "batt_a",
    "vbat",
    "soc",
    "soc_gauge",
    "soc_fsw",
    "capacity_ah",
    "t_batt",
    "t_obc",
    "t_pa",
    "t_jetson",
    "t_panel_min",
    "t_panel_max",
    "obc_on",
    "op_state",
    "power_mode",
    "payload_on",
    "jetson_w",
    "radio_on",
    "pa_on",
    "radiating",
    "heater_on",
    "adcs_on",
    "dipole",
    "antennas",
    "wings",
    "best_elev_deg",
    "rf_silence",
    "contact_inhibit",
    "tx_permitted",
    "xpdr_on",
    "xpdr_keyed",
    "stations",
    "carrier",
];

fn sample_row(s: &Sample) -> Vec<f64> {
    let b = |x: bool| if x { 1.0 } else { 0.0 };
    vec![
        s.t_s,
        s.alt_km,
        s.lat_deg,
        s.lon_deg,
        s.beta_deg,
        s.illum,
        s.rate_dps,
        s.sun_angle_deg,
        s.array_mpp_w,
        s.array_w,
        s.load_w,
        s.batt_a,
        s.vbat,
        s.soc,
        s.soc_gauge,
        s.soc_fsw,
        s.capacity_ah,
        s.t_batt,
        s.t_obc,
        s.t_pa,
        s.t_jetson,
        s.t_panel_min,
        s.t_panel_max,
        b(s.obc_on),
        f64::from(s.op_state),
        f64::from(s.power_mode),
        b(s.payload_on),
        s.jetson_w,
        b(s.radio_on),
        b(s.pa_on),
        b(s.radiating),
        b(s.heater_on),
        b(s.adcs_on),
        s.dipole,
        b(s.antennas),
        b(s.wings),
        s.best_elev_deg,
        b(s.rf_silence),
        b(s.contact_inhibit),
        b(s.tx_permitted),
        b(s.xpdr_on),
        b(s.xpdr_keyed),
        s.stations,
        b(s.carrier),
    ]
}

/// A pass over one station in progress
#[derive(Clone, Copy, Debug, Default)]
struct Pass {
    aos_s: f64,
    /// Faraday rotation at AOS, rad, and its drift, rad/s (SQSO-LBA-001 v3.0
    /// §2.3: a random start, 0.5–3 rad over a pass)
    faraday0: f64,
    faraday_rate: f64,
    max_elev_deg: f64,
    frames: u32,
    heard: u32,
    commands: u32,
}

/// The mission and everything in it
pub struct Mission {
    cfg: Config,
    /// Now, s since J2000
    t: f64,
    /// Deployment, s since J2000
    t0: f64,
    orbit: Orbit,
    att: Attitude,
    dist: Disturbance,
    array: Array,
    thermal: Thermal,
    eps: Eps,
    radio: Radio,
    deploy: Deployables,
    jetson: Jetson,
    fsw: Fsw,
    stations: Vec<ground::Station>,
    passes: Vec<Option<Pass>>,
    uplink: Uplink,
    rad: radiation::Radiation,
    rng: Rng,
    /// The transponder users' draws (kept apart, so the rest of the run
    /// draws the same numbers with or without them)
    qso_rng: Rng,
    /// A CTCSS-valid carrier is on the transponder's uplink
    carrier: bool,
    /// The ADCS sensors' errors
    sensors: crate::adcs_mc::Sensors,
    /// Their draws (kept apart, as the transponder users')
    sensor_rng: Rng,
    /// Expected stations calling under the footprint
    calling: f64,
    ts: Table,
    orbit_table: Table,
    pass_table: Table,
    logs: Logs,
    orbits: Orbits,
    totals: Totals,
    prev: Snapshot,
    last_record_s: f64,
    /// Keyed seconds in each of the last 60 s (duty check)
    keyed_hist: std::collections::VecDeque<(f64, f64)>,
    /// The GNSS receiver's records: GPS time of week, ms, and the true ECEF
    /// position, m, and velocity, m/s (the GNSS state frame's check)
    gnss_truth: std::collections::VecDeque<(i64, V3, V3)>,
    last_frame_s: Option<f64>,
    /// The last beacon while the receiver was known failed, s
    rx_failed_last_beacon_s: Option<f64>,
    /// When the body rate first stayed below the detumble threshold
    below_rate_since: Option<f64>,
    /// Commands sent today, and the day
    commands_today: (i64, u32),
    contact_timeout_set: bool,
    /// The Jetson's rail is latched up (drawing the latch-up current)
    latched_up: bool,
    /// How long the present latch-up has lasted, s
    latched_s: f64,
    /// Where the end-of-life plan is
    eol: ops::Eol,
    /// The EPS's end-of-life arm latch is set (an `EOL_ARM` rising edge)
    eol_latch: bool,
    /// The hardware load shed (SHED_N low) and how long VBAT has been low, s
    shed: bool,
    shed_below_s: f64,
    /// The four wing lines driven at the last step, and each wing deployed
    wing_lines_prev: [[bool; 2]; 2],
    /// The AntS was told to deploy (powered, armed, a limit) last step
    ants_cmd_prev: bool,
    /// The uplink carrier (with its tone) present without a break, s
    carrier_s: f64,
    /// Continuous sunlight (illumination at least 0.5), s
    sun_run_s: f64,
    /// The downlink's VSWR count seen this boot
    vswr_seen: u32,
    wing_deployed_prev: [bool; 2],
    /// The disconnect's heater has been on this long, s
    eol_heat_s: f64,
    /// The command receiver is hung (a single-event functional interrupt)
    /// since this time, s after deployment
    rx_hung_since: Option<f64>,
    /// The day of the last stored-housekeeping dump the ground asked for
    last_dump_day: i64,
    /// Where the commissioning plan is
    commissioning: ops::Commissioning,
    /// How long the GNSS receiver has been powered, s
    gnss_on_s: f64,
    /// The receiver's own arc while it is on: ECI position and velocity,
    /// propagated with J2 from the orbit model's state at power-on (the
    /// mean-element orbit has no short-period J2 terms; the receiver's
    /// fixes are then consistent with the dynamics a real orbit has)
    gnss_arc: Option<(f64, V3, V3)>,
    /// Last illumination, for the eclipse records
    last_sample: Sample,
}

/// The dispenser cold soak's length with `separation_soak_c`, s (THERMAL_ANALYSIS
/// 2.4 §4.4: 2 h)
const SOAK_S: f64 = 7200.0;

fn utc(t: f64) -> String {
    time::iso(t)
}

impl Mission {
    fn new(cfg: &Config, out: &Path) -> std::io::Result<Mission> {
        let mut rng = Rng::new(cfg.seed);
        let (y, m, d, h) = cfg.deploy_utc;
        let mut t0 = time::from_utc(y, m, d, h.floor() as u32, ((h.fract()) * 60.0) as u32, 0.0);
        if cfg.separate_at_eclipse {
            t0 = next_umbra_entry(cfg.altitude_km * 1000.0, cfg.ltan_h, t0);
        }
        let (iy, im, id) = cfg.integration_date;
        let t_int = time::from_utc(iy, im, id, 16, 0, 0.0);
        let orbit = Orbit::sso(cfg.altitude_km * 1000.0, cfg.ltan_h, t0);
        let att = Attitude::tumbling(&mut rng, attitude::INITIAL_TUMBLE_RAD_S, attitude::inertia_stowed());
        let pack = Pack::with_spread(cfg.integration_soc, &mut rng).scaled(cfg.battery_scale);
        let mut eps = Eps::new(pack, cfg.gauge_offset_a);
        eps.cv_cell_v = cfg.charge_v_cell;
        let limit = (cfg.burn_limit_a > 0.0).then_some(cfg.burn_limit_a);
        let mut deploy = Deployables::new(&mut rng, limit);
        if cfg.wings_failed {
            for e in &mut deploy.wings.elements {
                e.release_j = [f64::INFINITY; 2]; // the hold-down never lets go
            }
        }
        if cfg.wing_a_failed {
            for e in &mut deploy.wings.elements {
                e.release_j[0] = f64::INFINITY; // resistor A heats but does not part the line
            }
        }
        let mut jetson = Jetson::open(rng.next_u64())?;
        jetson.reject_fraction = cfg.ai_reject_fraction;
        // The fuse opens by the heater model here (`eol_hardware`), not by
        // the simulated controller's timer
        sqso_ffi::hal_sim::hal_sim_eps_eol_open_ms(0);
        // The flight software's host files and its serial port
        for (k, f) in [
            ("SMART_QSO_FAULT_LOG", "fsw_fault_log.dat"),
            ("SMART_QSO_EPS_CONFIG", "fsw_eps_config.dat"),
            ("SMART_QSO_MISSION_DATA", "fsw_mission_data.dat"),
            ("SMART_QSO_STATE_FILE", "fsw_state.dat"),
        ] {
            let path = out.join(f);
            let _ = std::fs::remove_file(&path);
            // SAFETY: single-threaded, before the flight software reads them
            unsafe { std::env::set_var(k, path) };
        }
        let yaml = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flight/sensors.yaml");
        // SAFETY: as above
        unsafe {
            std::env::set_var("UART_DEVICE", jetson.device());
            std::env::set_var("SENSORS_YAML", yaml);
        }
        let stations = ground::DEFAULT_NETWORK.to_vec();
        let n = stations.len();
        let rad = radiation::Radiation::new(rng.fork());
        let qso_rng = rng.fork();
        let mut sensor_rng = rng.fork();
        let mut sensor_model = crate::adcs_mc::default_params().sensors;
        sensor_model.gyro = crate::adcs_mc::gyro_preset(&cfg.gyro).flatten();
        let sensors = crate::adcs_mc::Sensors::new(sensor_model, &mut sensor_rng);
        Ok(Mission {
            cfg: cfg.clone(),
            t: t_int,
            t0,
            orbit,
            att,
            dist: Disturbance::nominal(),
            array: Array::new(),
            thermal: Thermal::new(cfg.storage_temp_c),
            eps,
            radio: Radio::default(),
            deploy,
            jetson,
            fsw: Fsw::new(),
            stations,
            passes: vec![None; n],
            uplink: Uplink::new(cfg.seed),
            rad,
            rng,
            qso_rng,
            carrier: false,
            sensors,
            sensor_rng,
            calling: 0.0,
            ts: Table::create(out, "timeseries.csv", TS_HEADER)?,
            orbit_table: Table::create(out, "orbits.csv", crate::metrics::Orbit::HEADER)?,
            pass_table: Table::create(
                out,
                "passes.csv",
                &["station", "aos_s", "los_s", "max_elev_deg", "frames", "heard", "commands"],
            )?,
            logs: Logs::create(out)?,
            orbits: Orbits::default(),
            totals: Totals {
                soc_min: 1.0,
                soc_min_ops: 1.0,
                tbatt_min: f64::INFINITY,
                tbatt_max: f64::NEG_INFINITY,
                tobc_min: f64::INFINITY,
                tobc_max: f64::NEG_INFINITY,
                tpa_max: f64::NEG_INFINITY,
                tjetson_max: f64::NEG_INFINITY,
                rx_failure_modelled: cfg.rx_failed_day > 0.0,
                ..Totals::default()
            },
            prev: Snapshot::default(),
            last_record_s: f64::NEG_INFINITY,
            keyed_hist: Default::default(),
            gnss_truth: Default::default(),
            last_frame_s: None,
            rx_failed_last_beacon_s: None,
            below_rate_since: None,
            commands_today: (-1, 0),
            contact_timeout_set: false,
            latched_up: false,
            latched_s: 0.0,
            eol: ops::Eol::Arm,
            eol_latch: false,
            shed: false,
            shed_below_s: 0.0,
            wing_lines_prev: [[false; 2]; 2],
            ants_cmd_prev: false,
            carrier_s: 0.0,
            sun_run_s: 0.0,
            vswr_seen: 0,
            wing_deployed_prev: [false; 2],
            eol_heat_s: 0.0,
            rx_hung_since: None,
            last_dump_day: -1,
            commissioning: ops::Commissioning::Housekeeping,
            gnss_on_s: 0.0,
            gnss_arc: None,
            last_sample: Sample::default(),
        })
    }

    /// Seconds since deployment
    fn rel(&self) -> f64 {
        self.t - self.t0
    }

    fn event(&mut self, kind: &str, detail: &str) {
        let (r, u) = (self.rel(), utc(self.t));
        self.logs.event(r, &u, kind, detail);
    }

    // ---------------------------------------------------------------------
    // Integration, storage, separation

    /// On the bench: the OBC on the umbilical (the bench supply, not the
    /// pack), stowed; the GSE loads the keys and arms the flight sequence
    fn bench(&mut self) {
        self.event("phase", "integration: OBC on the umbilical, key load and ARM_FLIGHT");
        // DEPLOY_SENSE low: DS2 stowed (the EPS drives it high once released)
        self.fsw.drive(pin::DEPLOY_SENSE, false);
        self.fsw.drive(pin::UMBILICAL_SENSE, true);
        if !self.fsw.boot(ResetCause::PowerOn) {
            self.event("anomaly", "bench boot failed");
        }
        self.consume_console();
        for frame in ops::gse_preflight(self.cfg.seed) {
            let _ = self.fsw.gse_bytes(&frame);
            for _ in 0..10 {
                self.fsw.step(100_000);
            }
        }
        let mut done = 0;
        for l in self.fsw.drain_console() {
            if l.contains("GSE key operation done") {
                done += 1;
            }
            if l.contains("GSE key operation refused") {
                self.event("anomaly", "GSE refused a pre-flight frame");
            }
            if !(l.starts_with("[READ]") || l.starts_with("SQGR") || l.starts_with("[UART] Sent")) {
                self.logs.console(self.rel(), &l);
            }
        }
        self.event("phase", &format!("GSE: {done} of 7 operations done; OBC off, umbilical removed"));
        self.fsw.power_off();
        self.fsw.drive(pin::UMBILICAL_SENSE, false);
    }

    /// From integration to deployment: unpowered, the pack self-discharging
    fn storage(&mut self) {
        let days = ((self.t0 - self.t) / 86_400.0).max(0.0);
        let soc0 = self.eps.pack.soc();
        let mut left = self.t0 - self.t;
        while left > 0.0 {
            let dt = left.min(86_400.0);
            self.eps.pack.step(0.0, dt, self.cfg.storage_temp_c);
            left -= dt;
        }
        self.t = self.t0;
        let soc1 = self.eps.pack.soc();
        self.eps.gauge.charge_ah = soc1 * self.eps.gauge.capacity_ah;
        self.totals.capacity_start_ah = self.eps.pack.capacity_ah();
        self.event("phase", &format!("storage {days:.0} days: SOC {:.1} % to {:.1} %", soc0 * 100.0, soc1 * 100.0));
    }

    /// Separation: the switches release, the pack connects, the OBC boots
    fn separate(&mut self) {
        if let Some(wall_c) = self.cfg.separation_soak_c {
            // The whole network, the internal nodes too (Version F; Version E
            // set only the structure and the AntS, which then warmed from the
            // internal nodes at the storage temperature)
            self.thermal.dispenser_soak(wall_c, SOAK_S);
            self.event(
                "phase",
                &format!(
                    "dispenser cold soak, {:.0} h to a wall at {wall_c:.1} °C: the AntS at {:.1} °C, the battery at {:.1} °C at separation",
                    SOAK_S / 3600.0,
                    self.thermal.temp_c(thermal::ANTS),
                    self.thermal.battery_c()
                ),
            );
        }
        self.deploy.separated = true;
        self.fsw.drive(pin::DEPLOY_SENSE, true);
        self.event("phase", "separation from the dispenser: switches released, OBC powers up");
        self.boot(ResetCause::PowerOn);
    }

    fn boot(&mut self, cause: ResetCause) {
        match cause {
            ResetCause::PowerOn => self.totals.boots_power_on += 1,
            ResetCause::Brownout => self.totals.boots_brownout += 1,
            ResetCause::Watchdog => self.totals.boots_watchdog += 1,
            ResetCause::Software => self.totals.boots_seu += 1,
        }
        if !self.fsw.boot(cause) {
            self.event("anomaly", &format!("boot after {cause:?} failed"));
        }
        self.vswr_seen = 0; // the downlink's counters restart
        self.event("reset", &format!("OBC boot #{} ({cause:?})", self.fsw.boots()));
    }

    /// Decision C-19: when the flight software's AntS transmitter hold
    /// ended, and each stop of the downlink by its VSWR protection
    fn track_ants_hold(&mut self, snap: &crate::fsw::Snapshot) {
        if !self.fsw.running() || self.rel() < 0.0 {
            return;
        }
        if self.totals.ants_hold_end_s.is_none() && !snap.ants_hold {
            self.totals.ants_hold_end_s = Some(self.rel());
            let out = if self.deploy.antennas_deployed() { "deployed" } else { "stowed" };
            self.event("deploy", &format!("AntS transmitter hold over (decision C-19), the AntS {out}"));
        }
        let n = snap.downlink.vswr_events;
        if n > self.vswr_seen {
            let new = n - self.vswr_seen;
            self.totals.vswr_stops += new;
            if !self.deploy.antennas_deployed() {
                self.totals.vswr_stops_stowed += new;
            }
            self.event("downlink", "VSWR protection stopped the downlink");
        }
        self.vswr_seen = n;
    }

    // ---------------------------------------------------------------------
    // Operations

    fn loads(&self) -> Loads {
        let on = self.fsw.running();
        let p = |n| on && self.fsw.pin(n);
        let mut l = Loads::default();
        if on {
            l.w_3v3 += OBC_W;
        }
        if p(pin::LED_STATUS) {
            l.w_3v3 += LED_W;
        }
        if p(pin::ADCS_EN) {
            l.w_3v3 += ADCS_SENSORS_W;
        }
        l.w_3v3 += self.radio.dc_w.min(crate::hw::radio::RX_W);
        l.w_5v += (self.radio.dc_w - crate::hw::radio::RX_W).max(0.0);
        let vbat = self.eps.telemetry.vbatt_v.max(self.eps.pack.ocv());
        if p(pin::HEATER_EN) {
            l.w_vbat += vbat * vbat / HEATER_OHMS;
        }
        if p(pin::ADCS_EN) {
            l.w_mtq = self.mtq_w(vbat);
            l.w_vbat += l.w_mtq;
        }
        if self.eol_heater_on() {
            l.w_vbat += eol_heater(vbat, self.cfg.eol_heater_ohms).0;
        }
        l.w_payload = if self.latched_up { crate::jetson::IDLE_W + LATCHUP_EXTRA_W } else { self.jetson.power_w() };
        l.burn_a = self.deploy.current_a;
        l
    }

    /// The coils' supply power, W, averaged over the control period: each
    /// rod's I² R through the bridge for the burst's 900 ms, and the bridges
    /// awake while MTQ_ARM is high
    fn mtq_w(&self, vbat: f64) -> f64 {
        let (d, on) = self.fsw.adcs_burst();
        if !on {
            return 0.0;
        }
        let rod = |m: f64| (m.abs() / ROD_GAIN).powi(2) * ROD_OHMS / BRIDGE_EFF;
        let frac = sqso_fsw::adcs::MTQ_ON_MS as f64 / sqso_fsw::adcs::CONTROL_PERIOD_MS as f64;
        (rod(d.x) + rod(d.y) + rod(d.z) + BRIDGES_AWAKE_A * vbat) * frac
    }

    fn fine_needed(&self) -> bool {
        self.radio.busy()
            || self.deploy.current_a > 0.0
            || (self.fsw.running() && pin::WING_LINES.iter().flatten().any(|&n| self.fsw.pin(n)))
    }

    /// The disconnect's heater conducts: the arm latch set and the fire
    /// switch (Q20, `EOL_FIRE`) closed, the charge path not yet open
    fn eol_heater_on(&self) -> bool {
        self.eol_latch && !self.eps.passivated && self.fsw.pin(pin::EOL_FIRE)
    }

    /// The EPS's end-of-life charge disconnect (SQSO-EPS-BRD-001 D §7): the
    /// simulated EPS controller (`hal_sim`) takes the key at `EOL_KEY` and
    /// sets the arm latch on an `EOL_ARM` rising edge while its key is up;
    /// with the latch set and `EOL_FIRE` high, F1's heater warms, and after
    /// [`EOL_OPEN_S`] inside the firing window the charge path opens for
    /// good. This is the only way the simulation passivates: through the
    /// flight software's key and two lines
    fn eol_hardware(&mut self, dt: f64) {
        use sqso_fsw::passivation::eol_state::{ARMED, FIRED, FUSE_INTACT};
        if !self.fsw.running() {
            return;
        }
        let state = self.fsw.eps_eol_state();
        self.eol_latch |= state & ARMED != 0;
        if self.eol_heater_on() && eol_heater(self.eps.telemetry.vbatt_v, self.cfg.eol_heater_ohms).1 {
            self.eol_heat_s += dt;
            if self.eol_heat_s >= EOL_OPEN_S {
                self.passivate();
            }
        }
        // The controller's state survives an OBC reset (the simulated bus
        // is re-initialized at every boot)
        let mut want = if self.eol_latch { state | ARMED } else { state };
        if self.eps.passivated {
            want = (want | FIRED) & !FUSE_INTACT;
        }
        if want != state {
            self.fsw.set_eps_eol_state(want);
        }
    }

    /// The hardware load shed: SHED_N low after [`SHED_DELAY_S`] below
    /// [`SHED_V`].0, high again above [`SHED_V`].1
    fn load_shed(&mut self, dt: f64, vbat: f64) {
        if vbat < SHED_V.0 {
            self.shed_below_s += dt;
            if !self.shed && self.shed_below_s >= SHED_DELAY_S {
                self.shed = true;
                if self.totals.passivated_s.is_none() {
                    self.totals.sheds += 1;
                }
                self.event("eps", &format!("hardware load shed: VBAT {vbat:.2} V (SW1 and U19 off)"));
            }
        } else {
            self.shed_below_s = 0.0;
            if self.shed && vbat > SHED_V.1 {
                self.shed = false;
                self.event("eps", &format!("load shed released: VBAT {vbat:.2} V"));
            }
        }
    }

    /// The command receiver: carrier detect from the users' carriers and the
    /// ground's frames, its health line, a hang (single-event functional
    /// interrupt) that only a power cycle clears, and a failure for good
    fn receiver(&mut self, radio_on: bool) {
        let failed = self.cfg.rx_failed_day > 0.0 && self.rel() >= self.cfg.rx_failed_day * 86_400.0;
        if !radio_on && let Some(since) = self.rx_hung_since.take() {
            self.totals.rx_recovered += 1;
            self.totals.rx_hang_max_s = self.totals.rx_hang_max_s.max(self.rel() - since);
            self.event(
                "seu",
                &format!("receiver power-cycled: recovered after {:.1} h", (self.rel() - since) / 3600.0),
            );
        }
        let working = radio_on && self.rx_hung_since.is_none() && !failed;
        self.fsw.drive(pin::RX_CD, working && self.carrier);
        self.fsw.drive(pin::RX_FAULT, failed);
    }

    /// One step of `dt` s in orbit
    #[allow(clippy::too_many_lines)]
    fn step(&mut self, dt: f64) {
        // Environment
        self.orbit.propagate(self.t);
        let (r, v) = self.orbit.state();
        let (sun_u, sun_d) = sun::sun_eci(self.t);
        let illum = eclipse::illumination(r, sun_u, sun_d);
        let flux = sun::solar_flux(sun_d);
        let b_eci = magfield::field_eci(r, self.t);
        let alt_m = Orbit::altitude_m(r);
        let density = atmosphere::density_nominal(alt_m, self.t);
        let wings = [self.deploy.wings.elements[0].deployed, self.deploy.wings.elements[1].deployed];
        let v_rel = v - V3::new(0.0, 0.0, OMEGA_EARTH).cross(r);
        let v_body = self.att.to_body(v_rel);
        let area = attitude::projected_area(v_body, wings);
        self.orbit.apply_drag(dt, density, area, MASS_KG, self.dist.cd);
        let sun_b = self.att.to_body(sun_u);
        let nadir_b = self.att.to_body(-r.unit());
        let b_b = self.att.to_body(b_eci);
        let cos_zeta = r.unit().dot(sun_u);

        // Attitude: the flight software's coils (when the ADCS rail is on),
        // the residual dipole, gravity gradient and drag
        let dipole = if self.fsw.running() && self.fsw.pin(pin::ADCS_EN) && !self.cfg.adcs_failed {
            self.fsw.adcs_dipole()
        } else {
            V3::ZERO
        };
        let torque = attitude::magnetic_torque(dipole, b_b)
            + self.dist.residual_torque(b_b)
            + attitude::gravity_gradient(r, self.att.q, self.att.inertia)
            + self.dist.aero_torque(v_body, density, wings);
        self.att.step(dt, torque);
        let out = self.deploy.wings.elements.map(|w| w.deployed);
        let inertia = match out {
            [true, true] => Some(attitude::inertia_deployed()),
            [true, false] => Some(attitude::INERTIA_ONE_WING[0]),
            [false, true] => Some(attitude::INERTIA_ONE_WING[1]),
            [false, false] => None,
        };
        if let Some(i) = inertia
            && i != self.att.inertia
        {
            // A wing swings out: the body's angular momentum is kept
            let h = self.att.w.hadamard(self.att.inertia);
            self.att.inertia = i;
            self.att.w = V3::new(h.x / i.x, h.y / i.y, h.z / i.z);
        }

        // Array and EPS
        let years = self.rel() / (365.25 * 86_400.0);
        let ainp = ArrayInputs {
            sun_body: sun_b,
            illum,
            flux,
            nadir_body: nadir_b,
            albedo_w_m2: earth::albedo_nadir_w_m2(r.norm(), cos_zeta, flux),
            r_m: r.norm(),
            temp_c: self.thermal.string_temps_c(),
            years_in_orbit: years.max(0.0) + self.cfg.array_years,
            wings_deployed: wings,
        };
        let strings = self.array.strings(&ainp);
        let mpp: f64 = strings.iter().map(|s| s.pmp).sum();
        let array_v = strings.iter().map(|s| s.vmp).fold(0.0, f64::max);
        let loads = self.loads();
        let tb = self.thermal.battery_c();
        let e = self.eps.step(dt, mpp, array_v, &loads, tb);

        // Thermal
        let mut inp = ThermalInputs::new(sun_b, illum, flux, nadir_b, r.norm(), cos_zeta);
        inp.wings_deployed = wings;
        inp.elec_w = thermal::elec_from_strings(&strings);
        inp.internal_w[thermal::BAT] = e.pack_heat_w + loads.w_vbat - loads.w_mtq;
        inp.internal_w[thermal::OBC] =
            loads.w_3v3 - self.radio.dc_w.min(crate::hw::radio::RX_W) + e.loss_w + loads.w_mtq;
        inp.internal_w[thermal::RF] = self.radio.pa_heat_w + self.radio.dc_w.min(crate::hw::radio::RX_W);
        inp.internal_w[thermal::JET] = loads.w_payload;
        self.thermal.step(dt, &inp);

        // Bus up or dark
        if !e.bus_up && self.fsw.running() {
            self.fsw.power_off();
            self.event("anomaly", "battery under-voltage cut-off: bus dark, OBC off");
        } else if e.bus_up && !self.fsw.running() && self.eps.passivated {
            // After passivation the pack only drains; no reboot is counted
        } else if e.bus_up && !self.fsw.running() {
            self.boot(ResetCause::Brownout);
        }

        // The OBC's inputs: EPS controller, ADC, sense lines
        // The EPS board is in the stack: its INA228s at the stack's temperature
        self.fsw.set_eps(&self.eps.telemetry, self.thermal.temp_c(thermal::OBC));
        let tel: Telemetry = self.eps.telemetry;
        self.fsw.set_adc(adc::VBATT, tel.vbatt_v);
        self.fsw.set_adc(adc::IBATT, -tel.ibatt_a);
        self.fsw.set_adc(adc::VSOLAR, tel.vsolar_v);
        self.fsw.set_adc(adc::ISOLAR, tel.isolar_a);
        self.fsw.set_adc(adc::VBUS, tel.vbus_v);
        self.fsw.set_adc(adc::TEMP_MCU, self.thermal.obc_c() + 4.0);
        // The OBC's TMP235, and the Jetson spreader's thermistor (biased by
        // the OBC: readable with the carrier off)
        self.fsw.set_adc(adc::TEMP_BOARD, self.thermal.obc_c());
        self.fsw.set_adc(adc::PT_NTC, crate::fsw::ntc_volts(self.thermal.jetson_c()));
        self.fsw.set_adc(adc::TX_TEMP, self.thermal.rf_c());
        self.fsw.set_adc(adc::TX_FWD, self.radio.fwd_w);
        self.fsw.set_adc(adc::TX_REV, self.radio.rev_w);
        self.fsw.drive(pin::ANTENNA_SENSE, self.deploy.antennas_deployed());
        // Low = both wings deployed (the hinge switches in series close)
        self.fsw.drive(pin::SOLAR_SENSE, !self.deploy.wings_deployed());
        let mut mag_meas = self.sensors.magnetometer(&mut self.sensor_rng, b_b);
        if self.jetson.power_good() {
            // The module's input current at the magnetometer (JETB-09); the
            // flight software does not use these samples (JET_PG gate)
            mag_meas += V3::new(crate::jetson::MAG_FIELD_T, 0.0, 0.0);
        }
        let sun_meas = self.sensors.sun_sensors(&mut self.sensor_rng, sun_b, illum);
        let gyro_meas = self.sensors.gyro(&mut self.sensor_rng, self.att.w, dt);
        self.fsw.set_adcs_sensors(mag_meas, sun_meas, gyro_meas);

        // Ground: passes, and telecommands
        let best_elev = self.ground(r);

        // The flight software
        self.fsw.step((dt * 1e6).round() as u64);
        if self.fsw.watchdog_expired() {
            self.event("anomaly", "hardware watchdog expired");
            self.boot(ResetCause::Watchdog);
        }
        let frames = self.fsw.take_tx();
        let snap = self.fsw.snapshot();
        self.track_ants_hold(&snap);
        for f in &frames {
            self.on_frame(f, &snap, r);
        }

        // Hardware the flight software drives
        let p = |n| self.fsw.pin(n);
        let (radio_on, pa_on, hi, armed) = (p(pin::RADIO_EN), p(pin::RF_TX_EN), p(pin::TX_PWR_SEL), p(pin::DEPLOY_EN));
        let wing = pin::WING_LINES.map(|w| w.map(p));
        // SW1 and U19 in series, each gated by SHED_N in hardware; the
        // stuck-on contingency fails the OBC's commands of both (SHED_N still
        // gates them, SQSO-EPS-BRD-001 D §8.4)
        let stuck = self.cfg.payload_stuck_on && self.rel() > 0.0;
        let jet_rail = (p(pin::PAYLOAD_EN) || stuck) && (p(pin::JETSON_PWR) || stuck) && !self.shed;
        // The transponder enable is the heartbeat detector's, not the pin's level
        self.gnss_receiver(dt, r, v_rel);
        let xpdr_en = radio::heartbeat_enable(self.fsw.xpdr_heartbeat());
        self.update_carrier(dt, r);
        // The radio board's key-down timer (HC4060, 120 s, reset when the
        // tone drops): an over longer than that unkeys the repeat path
        self.carrier_s = if self.carrier { self.carrier_s + dt } else { 0.0 };
        self.radio.xpdr = xpdr_en && self.carrier && self.carrier_s < KEY_DOWN_TIMER_S;
        self.radio.step(dt, radio_on, pa_on, hi, self.deploy.antennas_deployed());
        let was_burning = self.deploy.current_a > 0.0;
        // The wings' four lines and the AntS on I2C
        let (ants_armed, ants_limit_s) = self.fsw.ants_command();
        let cmd = deploy::Commands { armed, wing, ants_armed, ants_limit_s };
        let wing_c = [self.thermal.temp_c(thermal::W1), self.thermal.temp_c(thermal::W2)];
        let ants_c = self.thermal.temp_c(thermal::ANTS);
        self.sun_run_s = if illum >= 0.5 { self.sun_run_s + dt } else { 0.0 };
        let ants_cmd = ants_armed && ants_limit_s > 0;
        if ants_cmd && !self.ants_cmd_prev {
            self.totals.ants_starts += 1;
            if self.deploy.antennas_deployed() {
                self.totals.ants_starts_deployed += 1;
            }
            if self.totals.ants_release_c.is_none() {
                self.totals.ants_release_c = Some(ants_c);
                self.totals.ants_release_s = Some(self.rel());
                self.totals.ants_release_sun_s = Some(if illum >= 0.5 { self.sun_run_s } else { 0.0 });
            }
            self.event(
                "deploy",
                &format!(
                    "AntS told to deploy (command {}) at {ants_c:.1} °C, {:.1} min of sun",
                    self.totals.ants_starts,
                    if illum >= 0.5 { self.sun_run_s / 60.0 } else { 0.0 }
                ),
            );
        }
        self.ants_cmd_prev = ants_cmd;
        if (0.0..3.0 * 3600.0).contains(&self.rel()) {
            self.totals.ants_min_c = Some(self.totals.ants_min_c.map_or(ants_c, |m: f64| m.min(ants_c)));
        }
        if self.rel() >= f64::from(sqso_fsw::deployment::WAIT_TIME_S)
            && self.totals.ants_gate_open_s.is_none()
            && ants_c >= sqso_fsw::deployment::ANTS_MIN_TEMP_C
        {
            self.totals.ants_gate_open_s = Some(self.rel());
        }
        if self.rel() >= 31.0 * 60.0 && self.totals.ants_31min_c.is_none() {
            self.totals.ants_31min_c = Some(ants_c);
        }
        let _ = self.deploy.step(dt, cmd, e.vbat, wing_c, ants_c);
        if self.fsw.running() {
            self.fsw.set_ants_status(self.deploy.ants.status(ants_armed));
            // Its controller's own sensor: the AntS node (decision C-18's
            // gate), with the reading's error or a failed path (P3F-G-02)
            let reading = (!self.cfg.ants_temp_failed)
                .then_some(self.cfg.ants_temp_fixed_c.unwrap_or(ants_c + self.cfg.ants_temp_offset_c));
            self.fsw.set_ants_temperature(reading);
        }
        if self.deploy.current_a > 0.0 && !was_burning {
            if self.totals.first_burn_s.is_none() {
                self.totals.first_burn_s = Some(self.rel());
            }
            self.event(
                "deploy",
                &format!(
                    "burn on (wing lines W1A {} W1B {} W2A {} W2B {}, AntS {}), {:.2} A",
                    wing[0][0],
                    wing[0][1],
                    wing[1][0],
                    wing[1][1],
                    ants_limit_s > 0,
                    self.deploy.current_a
                ),
            );
        }
        if self.totals.antennas_s.is_none() && self.deploy.antennas_deployed() {
            self.totals.antennas_s = Some(self.rel());
            self.event("deploy", "every AntS antenna element deployed");
        }
        // Each wing line's burn and each wing's release, in order (the four
        // lines, one resistor at a time; the hinge switches are in series,
        // so SOLAR_SENSE reads deployed only with both wings out)
        let names = [["W1A", "W1B"], ["W2A", "W2B"]];
        for (w, wl) in wing.iter().enumerate() {
            for (r, &on) in wl.iter().enumerate() {
                if on && armed && !self.wing_lines_prev[w][r] {
                    self.event("deploy", &format!("wing line {} on", names[w][r]));
                }
                self.wing_lines_prev[w][r] = on && armed;
            }
            let out = self.deploy.wings.elements[w].deployed;
            if out && !self.wing_deployed_prev[w] {
                let burned = self.deploy.wings.elements[w].burn_s;
                self.event(
                    "deploy",
                    &format!("{} released after {burned:.1} s of burn", self.deploy.wings.elements[w].name),
                );
            }
            self.wing_deployed_prev[w] = out;
        }
        if self.totals.wings_s.is_none() && self.deploy.wings_deployed() {
            self.totals.wings_s = Some(self.rel());
            self.event("deploy", "both solar wings deployed");
        }
        if self.latched_up {
            if jet_rail {
                self.latched_s += dt;
            } else {
                // The rail is cut: the latch-up is cleared
                self.latched_up = false;
                self.totals.latchup_max_s = self.totals.latchup_max_s.max(self.latched_s);
                if self.latched_s <= LATCHUP_CLEAR_S {
                    self.totals.latchups_cleared += 1;
                }
                self.event("seu", &format!("Jetson latch-up cleared: rail off after {:.1} s", self.latched_s));
            }
        }
        self.jetson.step(dt, jet_rail && !self.latched_up, self.thermal.jetson_c());
        self.fsw.drive(pin::JET_PG, self.jetson.power_good());
        self.fsw.drive(pin::PTT_EXC, self.radio.keyed());
        self.fsw.drive(pin::XPDR_KEY, self.radio.xpdr_keyed);
        self.eol_hardware(dt);
        self.load_shed(dt, self.eps.telemetry.vbatt_v);
        self.receiver(radio_on);

        // Single-event effects
        if self.cfg.radiation {
            let (lat, lon, _) = earth::subpoint(r, self.t);
            let ev = self.rad.step(dt, lat, lon, alt_m);
            if ev.obc_reset && self.fsw.running() {
                self.event("seu", "single-event functional interrupt: OBC reset");
                self.boot(ResetCause::Software);
            }
            if ev.rx_sefi && self.cfg.rx_sefi && radio_on && self.rx_hung_since.is_none() {
                self.rx_hung_since = Some(self.rel());
                self.totals.rx_hangs += 1;
                self.event("seu", "command receiver hung (single-event functional interrupt)");
            }
            if ev.jetson_latchup && jet_rail && !self.latched_up {
                self.latched_up = true;
                self.latched_s = 0.0;
                self.totals.latchups += 1;
                self.event("seu", "Jetson single-event latch-up");
            }
        }

        self.consume_console();
        self.observe(&snap, dt, illum, &e, sun_b, best_elev, alt_m, r, dipole, radio_on, pa_on);
    }

    /// The ground segment this step: each station's pass, and the
    /// commanding plan on the primary station; the best elevation
    fn ground(&mut self, r: V3) -> f64 {
        let mut best = -90.0f64;
        let rel = self.rel();
        for i in 0..self.stations.len() {
            let (el, _, range) = ground::look(&self.stations[i], r, self.t);
            let el_deg = el.to_degrees();
            best = best.max(el_deg);
            let visible = el >= self.stations[i].min_el_rad;
            match (&mut self.passes[i], visible) {
                (None, true) => {
                    let faraday0 = self.rng.uniform() * core::f64::consts::PI;
                    let faraday_rate = (0.5 + 2.5 * self.rng.uniform()) / 600.0;
                    self.passes[i] =
                        Some(Pass { aos_s: rel, faraday0, faraday_rate, max_elev_deg: el_deg, ..Pass::default() })
                }
                (Some(p), true) => p.max_elev_deg = p.max_elev_deg.max(el_deg),
                (Some(p), false) => {
                    let p = *p;
                    self.passes[i] = None;
                    self.totals.passes += 1;
                    if p.heard > 0 {
                        self.totals.passes_heard += 1;
                    }
                    self.pass_table.row(&[
                        i as f64,
                        p.aos_s,
                        rel,
                        p.max_elev_deg,
                        f64::from(p.frames),
                        f64::from(p.heard),
                        f64::from(p.commands),
                    ]);
                }
                (None, false) => {}
            }
            if self.stations[i].telecommand && visible && el_deg >= self.cfg.contact_min_elev_deg {
                self.command_opportunity(i, el, range, r);
            }
        }
        best
    }

    /// The primary station's commanding plan: one telecommand a pass, sent
    /// once the station has heard the spacecraft in that pass (operators
    /// command a spacecraft they hear, on an orbit they have confirmed)
    fn command_opportunity(&mut self, i: usize, el: f64, range: f64, r: V3) {
        let Some(p) = self.passes[i] else { return };
        // The end of the TX-OFF test may follow in the same pass, and is sent
        // even though nothing is heard (the transmitter is off)
        if let ops::Commissioning::TxOnAfterTest(from) = self.commissioning {
            if self.rel() >= from
                && !self.send_command(i, el, range, r, cmd::TX_ENABLE, &[], "TX_ENABLE (end of the TX-OFF test)")
            {
                self.commissioning = ops::Commissioning::TxOnAfterTest(self.rel() + 30.0);
            }
            return;
        }
        // PASSIVATE_EXECUTE follows PASSIVATE_ARM in the same pass, with the
        // recovery key, until the arm's window closes
        if let ops::Eol::Execute(from, until) = self.eol {
            let rel = self.rel();
            if rel >= until {
                self.eol = ops::Eol::Arm;
            } else if rel >= from {
                let what = "PASSIVATE_EXECUTE (end of life, recovery key)";
                self.eol = if self.send_command(i, el, range, r, cmd::PASSIVATE_EXECUTE, &[], what) {
                    self.event("phase", "end-of-life passivation commanded");
                    ops::Eol::Done
                } else {
                    ops::Eol::Execute(rel + 30.0, until)
                };
            }
            return;
        }
        if p.commands > 0 || p.heard == 0 || self.eol == ops::Eol::Done {
            return;
        }
        let day = (self.rel() / 86_400.0).floor() as i64;
        if self.commands_today.0 != day {
            self.commands_today = (day, 0);
        }
        let outage = self.cfg.contact_outage_days > 0.0
            && (self.cfg.contact_outage_start_day..self.cfg.contact_outage_start_day + self.cfg.contact_outage_days)
                .contains(&(self.rel() / 86_400.0));
        let eol = self.rel() >= self.cfg.mission_days * 86_400.0;
        if outage || (self.commands_today.1 >= self.cfg.contacts_per_day && !eol) {
            return;
        }
        let dump_due = self.commissioning == ops::Commissioning::Done
            && self.cfg.dump_every_days > 0.0
            && day != self.last_dump_day
            && (day as f64 / self.cfg.dump_every_days).fract() == 0.0;
        let (id, args, what): (u8, Vec<u8>, &str) = if eol && self.cfg.passivate_at_eol {
            (cmd::PASSIVATE_ARM, ops::EOL_CODE.to_vec(), "PASSIVATE_ARM (end of life)")
        } else if dump_due && self.totals.store_next.is_some() {
            // The newest 4 h of five-minute records the last HK1 showed
            self.last_dump_day = day;
            let first = self.totals.store_next.unwrap_or(0).saturating_sub(48);
            let mut a = vec![0u8];
            a.extend_from_slice(&first.to_be_bytes());
            a.extend_from_slice(&48u16.to_be_bytes());
            (cmd::DUMP_TLM, a, "DUMP_TLM (the newest 48 five-minute records)")
        } else if self.commissioning == ops::Commissioning::Housekeeping {
            (cmd::GET_HOUSEKEEPING, vec![], "GET_HOUSEKEEPING (commissioning)")
        } else if self.commissioning == ops::Commissioning::TxOffTest {
            (cmd::TX_DISABLE, vec![], "TX_DISABLE (TX-OFF test)")
        } else if self.commissioning == ops::Commissioning::Release {
            (cmd::COMMISSIONING_RELEASE, vec![], "COMMISSIONING_RELEASE")
        } else if !self.contact_timeout_set && self.rel() >= self.cfg.commissioning_days * 86_400.0 {
            self.contact_timeout_set = true;
            (cmd::SET_CONTACT_TIMEOUT, 14u16.to_be_bytes().to_vec(), "SET_CONTACT_TIMEOUT 14 (end of commissioning)")
        } else {
            (cmd::GET_STATUS, vec![], "GET_STATUS")
        };
        let delivered = self.send_command(i, el, range, r, id, &args, what);
        if id == cmd::PASSIVATE_ARM && delivered {
            let rel = self.rel();
            // EXECUTE 10 s later, within the 600 s window
            self.eol = ops::Eol::Execute(rel + 10.0, rel + 590.0);
        }
    }

    /// Send one telecommand to the spacecraft in station `i`'s pass; whether
    /// it was received. Advances the commissioning plan on a received one
    #[allow(clippy::too_many_arguments)]
    fn send_command(&mut self, i: usize, el: f64, range: f64, r: V3, id: u8, args: &[u8], what: &str) -> bool {
        let Some(frame) = self.uplink.frame(id, args) else { return false };
        self.totals.commands_sent += 1;
        self.commands_today.1 += 1;
        if let Some(p) = self.passes[i].as_mut() {
            p.commands += 1;
        }
        let failed = self.cfg.rx_failed_day > 0.0 && self.rel() >= self.cfg.rx_failed_day * 86_400.0;
        let rx_on = self.fsw.running() && self.fsw.pin(pin::RADIO_EN) && self.rx_hung_since.is_none() && !failed;
        let (gain, pol) = self.sc_link_terms(link::UHF_AXIS_BODY, i, r);
        let cn0 = link::uplink(&self.stations[i].radio, gain, range, el, pol).cn0_dbhz;
        let bits = (frame.len() * 8) as u32;
        let ok = rx_on && self.rng.uniform() < link::frame_success(cn0, link::Demod::Telecommand, bits);
        let delivered = ok && self.fsw.uplink(&frame);
        self.event("command", &format!("{what}: {}", if delivered { "received" } else { "lost" }));
        if delivered {
            self.commissioning = match (self.commissioning, id) {
                (ops::Commissioning::Housekeeping, cmd::GET_HOUSEKEEPING) => ops::Commissioning::TxOffTest,
                (ops::Commissioning::TxOffTest, cmd::TX_DISABLE) => {
                    // The beacon gap the test makes is commanded, not a fault
                    self.last_frame_s = None;
                    ops::Commissioning::TxOnAfterTest(self.rel() + ops::TX_OFF_TEST_S)
                }
                (ops::Commissioning::TxOnAfterTest(_), cmd::TX_ENABLE) => {
                    self.last_frame_s = None;
                    ops::Commissioning::Release
                }
                (ops::Commissioning::Release, cmd::COMMISSIONING_RELEASE) => {
                    self.event("phase", "commissioning hold released by telecommand");
                    ops::Commissioning::Done
                }
                (c, _) => c,
            };
        }
        delivered
    }

    /// The spacecraft antenna's gain toward station `i` (an element along
    /// `axis` in the body frame), dBi, 20 dB down while it is stowed; and the
    /// polarization loss, dB: 3 dB to a circular station, the geometric
    /// mismatch with the Faraday rotation to a linear one (its Yagi's
    /// elements horizontal, across the line of sight)
    fn sc_link_terms(&self, axis: V3, i: usize, r: V3) -> (f64, f64) {
        let st = ground::station_eci(&self.stations[i], self.t);
        let los = (st - r).unit();
        let g = link::sc_gain_dbi(axis, self.att.to_body(los));
        let g = if self.deploy.antennas_deployed() { g } else { g - 20.0 };
        let radio = &self.stations[i].radio;
        let pol = if radio.circular {
            link::pol_loss_db(true, 0.0)
        } else {
            let element = self.att.q.rotate(axis);
            let h = st.unit().cross(-los).unit();
            let far = self.passes[i].map_or(0.0, |p| p.faraday0 + p.faraday_rate * (self.rel() - p.aos_s));
            link::pol_loss_db(false, link::linear_pol_factor(element, -los, h, far))
        };
        (g, pol)
    }

    /// EOL passivation after the TX-OFF (47 CFR 97.207(g)): the latch opens
    /// the charge path for good
    /// The amateurs under the footprint (`crate::qso`): each step an over
    /// ends with probability dt / [`OVER_S`], and a new one starts at the
    /// rate that keeps the carrier on for the demand model's share of time
    fn update_carrier(&mut self, dt: f64, r: V3) {
        let (lat, lon, _) = earth::subpoint(r, self.t);
        let days = (self.t + 43_200.0) / 86_400.0; // days since 2000-01-01 00:00 (a Saturday)
        let weekend = (days.floor() as i64 + 5).rem_euclid(7) >= 5;
        let alt_km = Orbit::altitude_m(r) / 1000.0;
        self.calling = qso::stations_calling(lat.to_degrees(), lon.to_degrees(), alt_km, days.fract() * 24.0, weekend);
        let share = qso::carrier_fraction(self.calling);
        let u = self.qso_rng.uniform();
        self.carrier = if self.carrier {
            u >= dt / OVER_S
        } else {
            share > 0.0 && u < dt * share / (OVER_S * (1.0 - share).max(1e-6))
        };
    }

    /// The GNSS receiver (piNAV-NG, decision D6): powered by the flight
    /// software's load switch; after an 80 s cold start it sends a GGA with a
    /// GPS fix and a `$PSLSP` state record of the true position every 10 s
    fn gnss_receiver(&mut self, dt: f64, r: V3, v_rel: V3) {
        if !self.fsw.pin(pin::GNSS_EN) {
            self.gnss_on_s = 0.0;
            self.gnss_arc = None;
            return;
        }
        let omega = V3::new(0.0, 0.0, OMEGA_EARTH);
        // The arc is advanced to this step's time (the steps are 1 s or 0.1 s)
        let (r, v) = match self.gnss_arc {
            None => (r, v_rel + omega.cross(r)),
            Some((t0, mut r0, mut v0)) => {
                let span = self.t - t0;
                let n = (span / 1.0).ceil().max(1.0);
                for _ in 0..n as u32 {
                    (r0, v0) = j2_step(r0, v0, span / n);
                }
                (r0, v0)
            }
        };
        self.gnss_arc = Some((self.t, r, v));
        let v_rel = v - omega.cross(r);
        let before = self.gnss_on_s;
        self.gnss_on_s += dt;
        const COLD_START_S: f64 = 80.0;
        if self.gnss_on_s < COLD_START_S || (before / 10.0).floor() == (self.gnss_on_s / 10.0).floor() {
            return;
        }
        let x = earth::eci_to_ecef(r, self.t);
        let tow = (self.t + 43_200.0 - 1_198_800.0 + 13.0).rem_euclid(604_800.0); // GPS time of week, near enough
        let gga = format!("GPGGA,{:06.0},0000.000,N,00000.000,E,1,08,1.1,500000.0,M,0.0,M,,", tow % 86_400.0);
        let pslsp = format!("PSLSP,{tow:.3},2400,{:.3},{:.3},{:.3},8,1.6", x.x, x.y, x.z);
        self.gnss_truth.push_back(((tow * 1000.0).round() as i64, x, earth::eci_to_ecef(v_rel, self.t)));
        if self.gnss_truth.len() > 64 {
            self.gnss_truth.pop_front();
        }
        for body in [gga, pslsp] {
            let sum = body.bytes().fold(0u8, |a, b| a ^ b);
            let _ = self.fsw.gnss_bytes(format!("${body}*{sum:02X}\r\n").as_bytes());
        }
    }

    fn passivate(&mut self) {
        self.eps.passivated = true;
        self.totals.passivated_s = Some(self.rel());
        self.totals.passivation_by_command = self.eol == ops::Eol::Done;
        self.event(
            "passivation",
            "EOL charge disconnect opened (the EPS's key and EOL_ARM set the latch, EOL_FIRE the heater): array disconnected; the final report, then TX-OFF",
        );
    }

    fn on_frame(&mut self, f: &[u8], snap: &Snapshot, r: V3) {
        self.radio.send(f.len());
        let b = ops::decode_beacon(f);
        let rel = self.rel();
        // After the charge path opens the final report still goes out
        // (RID REG-12): forbidden only past its window
        let silenced = self
            .totals
            .passivated_s
            .is_some_and(|t| rel > t + (sqso_fsw::passivation::REPORT_WINDOW_MS / 1000) as f64 + 60.0);
        if !snap.tx_permitted || silenced {
            self.totals.frames_forbidden += 1;
            self.event("anomaly", "frame handed to the TNC while transmission was not permitted");
        }
        if self.totals.first_rf_s.is_none() && self.fsw.pin(pin::RF_TX_EN) {
            self.totals.first_rf_s = Some(rel);
            self.event("phase", "first transmission");
        }
        self.totals.frames += 1;
        if b.as_ref().is_some_and(|b| b.ai) {
            self.totals.ai_frames += 1;
        }
        let kind = b.as_ref().map(|b| b.kind).unwrap_or_default();
        match kind {
            ops::FrameKind::Ack => self.totals.acks += 1,
            ops::FrameKind::Dump => {
                self.totals.dump_frames += 1;
                self.totals.dump_records +=
                    u64::from(f.get(f.iter().position(|b| b & 1 == 1).unwrap_or(0) + 7).copied().unwrap_or(0));
            }
            ops::FrameKind::Gnss => self.gnss_frame(b.as_ref().map_or("", |b| b.text.as_str())),
            ops::FrameKind::Housekeeping | ops::FrameKind::Beacon => {}
        }
        if kind == ops::FrameKind::Housekeeping
            && let Some(st) = b.as_ref().and_then(|b| b.text.split(' ').find_map(|w| w.strip_prefix("ST=")))
            && let Some(next) = st.split('/').next().and_then(|v| v.parse::<u32>().ok())
        {
            self.totals.store_next = Some(next);
        }
        // The beacon gap counts beacons only
        if kind == ops::FrameKind::Beacon
            && let Some(last) = self.last_frame_s
            && snap.tx_permitted
            && !snap.rf_silence
            && !snap.contact_inhibit
        {
            self.totals.max_beacon_gap_s = self.totals.max_beacon_gap_s.max(rel - last);
        }
        // REL-04: the beacons go on with the receiver known failed, the
        // loss-of-contact inhibit notwithstanding
        if kind == ops::FrameKind::Beacon && snap.rx_failed {
            self.totals.rx_failed_beacons += 1;
            if let Some(last) = self.rx_failed_last_beacon_s
                && snap.tx_permitted
                && !snap.rf_silence
            {
                self.totals.rx_failed_beacon_gap_s = self.totals.rx_failed_beacon_gap_s.max(rel - last);
            }
            self.rx_failed_last_beacon_s = Some(rel);
        }
        if kind == ops::FrameKind::Beacon {
            self.last_frame_s = Some(rel);
        }
        // Who heard it
        let tx_w = if self.fsw.pin(pin::TX_PWR_SEL) { link::TX_MAX_W } else { link::TX_NOMINAL_W };
        let radiating = self.fsw.pin(pin::RF_TX_EN) && self.fsw.pin(pin::RADIO_EN);
        for i in 0..self.stations.len() {
            if self.passes[i].is_none() {
                continue;
            }
            let (el, _, range) = ground::look(&self.stations[i], r, self.t);
            let (gain, pol) = self.sc_link_terms(link::VHF_AXIS_BODY, i, r);
            let radio = self.stations[i].radio;
            let cn0 = link::downlink(tx_w, gain, range, el, &radio, pol).cn0_dbhz;
            let bits = (f.len() * 8) as u32;
            let heard = radiating && self.rng.uniform() < link::frame_success(cn0, radio.downlink_demod(), bits);
            if let Some(p) = self.passes[i].as_mut() {
                p.frames += 1;
                if heard {
                    p.heard += 1;
                    self.totals.beacons_heard += 1;
                }
            }
        }
    }

    /// A GNSS state frame against the receiver's records (SRS-F158): the
    /// position as reported, the velocity the flight software derived
    fn gnss_frame(&mut self, text: &str) {
        let field = |k: &str| {
            text.split(' ')
                .find_map(|w| w.strip_prefix(k).and_then(|v| v.strip_prefix('=')))
                .and_then(|v| v.parse::<f64>().ok())
        };
        self.totals.gnss_frames += 1;
        let (Some(tow), Some(x), Some(y), Some(z)) = (field("TOW"), field("X"), field("Y"), field("Z")) else {
            self.event("anomaly", &format!("GNSS frame without time or position: {text}"));
            return;
        };
        let Some(&(_, rt, vt)) = self.gnss_truth.iter().find(|(t, _, _)| *t == tow as i64) else { return };
        let pos_err = (V3::new(x, y, z) - rt).norm();
        self.totals.gnss_pos_err_max_m = self.totals.gnss_pos_err_max_m.max(pos_err);
        if let (Some(vx), Some(vy), Some(vz)) = (field("VX"), field("VY"), field("VZ")) {
            let vel_err = (V3::new(vx, vy, vz) * 1e-3 - vt).norm();
            self.totals.gnss_vel_frames += 1;
            self.totals.gnss_vel_err_max = self.totals.gnss_vel_err_max.max(vel_err);
            self.totals.gnss_vel_err_sum += vel_err;
        }
    }

    fn consume_console(&mut self) {
        let rel = self.rel();
        for l in self.fsw.drain_console() {
            if l.starts_with("[READ]") || l.starts_with("[UART] Sent") || l.starts_with("SQGR") {
                continue;
            }
            if l.contains("[GNSS] Session off: fix") {
                self.totals.gnss_fix_sessions += 1;
            }
            if let Some(rest) = l.strip_prefix("[FAULT] Type=")
                && let Some((_, text)) = rest.split_once(": ")
            {
                *self.totals.faults.entry(text.to_string()).or_default() += 1;
            }
            self.logs.console(rel, &l);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn observe(
        &mut self,
        snap: &Snapshot,
        dt: f64,
        illum: f64,
        e: &crate::hw::eps::EpsOut,
        sun_b: V3,
        best_elev: f64,
        alt_m: f64,
        r: V3,
        dipole: V3,
        radio_on: bool,
        pa_on: bool,
    ) {
        let rel = self.rel();
        let (lat, lon, _) = earth::subpoint(r, self.t);
        let (sun_u, _) = sun::sun_eci(self.t);
        let rate = self.att.w.norm().to_degrees();
        let panels: Vec<f64> = (0..8).map(|n| self.thermal.temp_c(n)).collect();
        let s = Sample {
            t_s: rel,
            dt,
            alt_km: alt_m / 1000.0,
            illum,
            beta_deg: self.orbit.beta(sun_u).to_degrees(),
            lat_deg: lat.to_degrees(),
            lon_deg: lon.to_degrees(),
            rate_dps: rate,
            sun_angle_deg: sun_b.angle(V3::new(0.0, 1.0, 0.0)).to_degrees(),
            array_mpp_w: e.array_mpp_w,
            array_w: e.array_w,
            load_w: e.load_w,
            batt_a: e.batt_a,
            vbat: e.vbat,
            soc: self.eps.pack.soc(),
            soc_gauge: self.eps.gauge.soc(),
            soc_fsw: snap.soc,
            capacity_ah: self.eps.pack.capacity_ah(),
            t_batt: self.thermal.battery_c(),
            t_obc: self.thermal.obc_c(),
            t_pa: self.thermal.rf_c(),
            t_jetson: self.thermal.jetson_c(),
            t_panel_min: panels.iter().copied().fold(f64::INFINITY, f64::min),
            t_panel_max: panels.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            charge_blocked: e.charge_blocked_cold || e.charge_blocked_hot,
            obc_on: self.fsw.running(),
            op_state: snap.op_state,
            power_mode: snap.power_mode,
            payload_on: snap.payload_on,
            jetson_w: self.jetson.power_w(),
            radio_on,
            pa_on,
            keyed: self.radio.keyed(),
            radiating: self.radio.fwd_w > 0.0,
            heater_on: self.fsw.pin(pin::HEATER_EN),
            adcs_on: self.fsw.pin(pin::ADCS_EN),
            dipole: dipole.norm(),
            antennas: self.deploy.antennas_deployed(),
            wings: self.deploy.wings_deployed(),
            best_elev_deg: best_elev,
            rf_silence: snap.rf_silence,
            contact_inhibit: snap.contact_inhibit,
            tx_permitted: snap.tx_permitted,
            xpdr_on: radio::heartbeat_enable(self.fsw.xpdr_heartbeat()),
            xpdr_keyed: self.radio.xpdr_keyed && self.radio.fwd_w > 0.0,
            stations: self.calling,
            carrier: self.carrier,
        };
        self.totals_add(&s, snap, dt);
        if let Some(o) = self.orbits.add(&s) {
            self.orbit_table.row(&o.row());
        }
        let interval = if rel < self.cfg.detail_days * 86_400.0 { self.cfg.detail_record_s } else { self.cfg.record_s };
        if rel - self.last_record_s >= interval {
            self.last_record_s = rel;
            self.ts.row(&sample_row(&s));
        }
        self.mode_events(snap);
        self.prev = *snap;
        self.last_sample = s;
    }

    fn totals_add(&mut self, s: &Sample, snap: &Snapshot, dt: f64) {
        let rx_failure_on = self.cfg.rx_failed_day > 0.0 && self.rel() >= self.cfg.rx_failed_day * 86_400.0;
        let t = &mut self.totals;
        t.mission_s += dt;
        if s.obc_on {
            t.obc_up_s += dt;
        }
        t.soc_min = t.soc_min.min(s.soc);
        if s.t_s > 86_400.0 && t.passivated_s.is_none() {
            t.soc_min_ops = t.soc_min_ops.min(s.soc);
        }
        if t.passivated_s.is_none() {
            t.tbatt_min = t.tbatt_min.min(s.t_batt);
            t.tbatt_max = t.tbatt_max.max(s.t_batt);
            t.tobc_min = t.tobc_min.min(s.t_obc);
            t.tobc_max = t.tobc_max.max(s.t_obc);
            t.tpa_max = t.tpa_max.max(s.t_pa);
            t.tjetson_max = t.tjetson_max.max(s.t_jetson);
            if s.charge_blocked {
                t.charge_blocked_s += dt;
            }
            t.gen_wh += s.array_w * dt / 3600.0;
            t.load_wh += s.load_w * dt / 3600.0;
            t.gauge_err_max = t.gauge_err_max.max((s.soc_gauge - s.soc).abs());
        } else if s.batt_a > 0.0 {
            t.charge_after_passivation_ah += s.batt_a * dt / 3600.0;
        }
        if snap.contact_inhibit {
            t.contact_inhibit_s += dt;
        }
        if snap.rx_failed {
            t.rx_known_failed_s += dt;
            if s.xpdr_keyed {
                t.rx_failed_xpdr_s += dt;
            }
        } else {
            self.rx_failed_last_beacon_s = None;
            if snap.contact_inhibit && rx_failure_on {
                t.rx_failed_pending_s += dt;
            }
        }
        // The transponder: its gates, and the demand it served
        if s.xpdr_on {
            t.xpdr_on_s += dt;
        }
        if s.xpdr_keyed {
            t.xpdr_keyed_s += dt;
            if !snap.tx_permitted || snap.rf_silence || snap.contact_inhibit || self.eps.passivated {
                t.xpdr_forbidden_s += dt;
            }
            if s.illum < 0.05 || snap.soc < sqso_fsw::mode::SOC_PAYLOAD - 0.01 {
                t.xpdr_out_of_gate_s += dt;
            }
        }
        let gates = s.illum > 0.5
            && snap.soc >= sqso_fsw::mode::SOC_PAYLOAD
            && snap.tx_permitted
            && snap.op_state == STATE_ACTIVE
            && s.antennas;
        if s.carrier && gates {
            t.xpdr_demand_s += dt;
            if s.xpdr_keyed {
                t.xpdr_served_s += dt;
            }
        }
        // Downlink (TNC) duty over the last 60 s: the steps starting in
        // (t - 60 s, t], each covering its own dt, so exactly 60 s (Version D
        // kept the step 60 s back as well, 61 s of steps)
        let keyed = if s.radiating && s.keyed { dt } else { 0.0 };
        self.keyed_hist.push_back((s.t_s, keyed));
        while self.keyed_hist.front().is_some_and(|(t0, _)| s.t_s - t0 >= 60.0 - 1e-6) {
            self.keyed_hist.pop_front();
        }
        if keyed > 0.0 {
            let sum: f64 = self.keyed_hist.iter().map(|(_, k)| k).sum();
            if sum / 60.0 > t.max_duty_60s {
                t.max_duty_60s = sum / 60.0;
                t.max_duty_at_s = s.t_s;
            }
        }
        t.longest_key_s = self.radio.stats.longest_key_s;
        // Detumble
        if crate::adcs_mc::captured(self.att.w, sqso_fsw::adcs::SPIN_RATE) {
            let since = *self.below_rate_since.get_or_insert(s.t_s);
            if t.detumbled_s.is_none() && s.t_s - since >= 60.0 {
                t.detumbled_s = Some(since);
                let (r, u) = (s.t_s, utc(self.t));
                self.logs.event(r, &u, "phase", &format!("detumbled: rate across +Y below {DETUMBLED_DPS} °/s"));
            }
        } else {
            self.below_rate_since = None;
        }
        if !self.eps.passivated {
            // After passivation the pack is meant to drain to its cut-off
            t.uv_trips = self.eps.uv_trips;
        }
        t.oc_trips = self.eps.oc_trips;
        // The flight software's counters restart at each boot
        let grew = |now: u32, before: u32| if now >= before { now - before } else { now };
        t.commands_executed += grew(snap.commands_executed, self.prev.commands_executed);
        t.auth_failures += grew(snap.auth_failures, self.prev.auth_failures);
        t.jetson_wh = self.jetson.stats.energy_wh;
        t.jetson_bursts.clone_from(&self.jetson.bursts);
        t.jetson_power_ons = self.jetson.stats.power_ons;
    }

    fn mode_events(&mut self, snap: &Snapshot) {
        if !self.fsw.running() {
            return;
        }
        // sqso_fsw::mode::State
        const NAMES: [&str; 6] = ["BOOT", "SAFE", "DETUMBLE", "IDLE", "ACTIVE", "EMERGENCY"];
        const SAFE: u32 = 1;
        const EMERGENCY: u32 = 5;
        if snap.op_state != self.prev.op_state {
            self.totals.mode_changes += 1;
            let in_mission = self.rel() > self.cfg.commissioning_days * 86_400.0 && !self.eps.passivated;
            if in_mission && snap.op_state == SAFE {
                self.totals.safe_entries += 1;
            }
            if snap.op_state == EMERGENCY && !self.eps.passivated {
                self.totals.safe_entries += 1;
                self.totals.emergency_entries += 1;
            }
            let name = |s: u32| NAMES.get(s as usize).copied().unwrap_or("?");
            let watched = |s: u32| s == SAFE || s == EMERGENCY || s == 2;
            if watched(snap.op_state) || watched(self.prev.op_state) {
                let d =
                    format!("{} -> {} (SOC {:.0} %)", name(self.prev.op_state), name(snap.op_state), snap.soc * 100.0);
                self.event("mode", &d);
            }
        }
        if snap.contact_inhibit != self.prev.contact_inhibit {
            self.event(
                "anomaly",
                &format!("loss-of-contact inhibit {}", if snap.contact_inhibit { "on" } else { "off" }),
            );
        }
        if self.prev.deploy_state != snap.deploy_state {
            self.event("deploy", &format!("deployment state {}", snap.deploy_state));
        }
    }

    /// Operations from separation to the end of the design life (and after
    /// it, until the OBC goes dark after passivation)
    fn operate(&mut self) {
        let end = self.cfg.mission_days * 86_400.0;
        let report_every = 30.0 * 86_400.0;
        let mut next_report = report_every;
        let wall = std::time::Instant::now();
        loop {
            let rel = self.rel();
            if rel >= end && (!self.cfg.passivate_at_eol || (self.eps.passivated && !self.fsw.running())) {
                break;
            }
            if rel >= end + 120.0 * 86_400.0 {
                self.event("anomaly", "OBC still running 120 days after the end of the design life");
                break;
            }
            let dt = if self.fine_needed() { self.cfg.fine_step_s } else { self.cfg.step_s };
            self.step(dt);
            self.t += dt;
            if self.eps.passivated && !self.fsw.running() && self.totals.dark_s.is_none() {
                self.totals.dark_s = Some(self.rel());
                self.event("passivation", "OBC dark for good: the pack is drained and cannot be charged");
            }
            if rel >= next_report {
                next_report += report_every;
                eprintln!(
                    "[SIL] day {:.0}: SOC {:.0} %, alt {:.1} km, {:.0} s wall",
                    rel / 86_400.0,
                    self.eps.pack.soc() * 100.0,
                    self.last_sample.alt_km,
                    wall.elapsed().as_secs_f64()
                );
                self.ts.flush();
                self.logs.flush();
            }
        }
        self.totals.eol_alt_km = self.orbit.mean_altitude_m() / 1000.0;
        self.totals.capacity_end_ah = self.eps.pack.capacity_ah();
        self.totals.tid_krad = self.rad.dose_rad / 1000.0;
        self.totals.wing_lines_max = self.deploy.max_wing_lines;
    }

    /// The orbit alone, without the spacecraft working, to re-entry
    fn decay(&mut self, out: &Path) -> std::io::Result<()> {
        let mut table = Table::create(out, "decay.csv", &["t_s", "alt_km"])?;
        let area = 0.0247; // wings deployed, NASA-STD-8719.14C average (BASELINE §3)
        let dt = 86_400.0;
        while !self.orbit.reentered() {
            let alt = self.orbit.mean_altitude_m();
            let rho = atmosphere::density_nominal(alt, self.t);
            self.orbit.apply_drag(dt, rho, area, MASS_KG, self.dist.cd);
            self.t += dt;
            table.row(&[self.rel(), alt / 1000.0]);
            if self.rel() > 40.0 * 365.25 * 86_400.0 {
                break;
            }
        }
        self.totals.reentry_s = Some(self.rel());
        self.event("reentry", &format!("re-entry {:.2} years after deployment", self.rel() / (365.25 * 86_400.0)));
        table.flush();
        Ok(())
    }
}

/// Run the configured mission, writing the records to `out`; the number of
/// anomalies found
pub fn run(cfg: &Config, out: &Path) -> std::io::Result<usize> {
    std::fs::create_dir_all(out)?;
    std::fs::write(out.join("config.json"), cfg.to_json())?;
    let mut m = Mission::new(cfg, out)?;
    m.bench();
    m.storage();
    m.separate();
    m.operate();
    if cfg.to_reentry {
        m.decay(out)?;
    }
    m.ts.flush();
    m.orbit_table.flush();
    m.pass_table.flush();
    let checks = checks::evaluate(&m.totals, &m.orbits.done, cfg.mission_days * 86_400.0, cfg.passivate_at_eol);
    let anomalies = checks.iter().filter(|c| c.verdict == Verdict::Anomaly).count();
    for c in &checks {
        if c.verdict != Verdict::Pass {
            let d = format!("{}: {} (limit {})", c.title, c.value, c.limit);
            m.event(if c.verdict == Verdict::Anomaly { "check-anomaly" } else { "check-warning" }, &d);
        }
    }
    let extra = vec![
        ("days_simulated", format!("{:.2}", m.totals.mission_s / 86_400.0)),
        ("orbits", m.orbits.done.len().to_string()),
        ("frames_radiated", m.totals.frames.to_string()),
        ("ai_beacons", m.totals.ai_frames.to_string()),
        ("beacons_heard", m.totals.beacons_heard.to_string()),
        ("passes", m.totals.passes.to_string()),
        ("commands_sent", m.totals.commands_sent.to_string()),
        ("obc_boots", m.fsw.boots().to_string()),
        ("seu_resets", m.totals.boots_seu.to_string()),
        ("jetson_energy_wh", format!("{:.1}", m.totals.jetson_wh)),
        ("jetson_power_ons", m.totals.jetson_power_ons.to_string()),
        ("jetson_burst_j_p50", format!("{:.0}", crate::checks::percentile(&m.totals.jetson_bursts, 0.50))),
        ("jetson_burst_j_p95", format!("{:.0}", crate::checks::percentile(&m.totals.jetson_bursts, 0.95))),
        ("jetson_wh_per_good_line", format!("{:.4}", crate::checks::wh_per_line(&m.totals.jetson_bursts))),
        ("energy_generated_wh", format!("{:.0}", m.totals.gen_wh)),
        ("energy_consumed_wh", format!("{:.0}", m.totals.load_wh)),
        ("capacity_start_ah", format!("{:.3}", m.totals.capacity_start_ah)),
        ("capacity_end_ah", format!("{:.3}", m.totals.capacity_end_ah)),
        ("eol_altitude_km", format!("{:.1}", m.totals.eol_alt_km)),
        ("tid_krad", format!("{:.2}", m.totals.tid_krad)),
        ("downlink_keyed_s", format!("{:.0}", m.radio.stats.keyed_s)),
        ("acks_radiated", m.totals.acks.to_string()),
        ("dump_frames", m.totals.dump_frames.to_string()),
        ("rx_hangs", m.totals.rx_hangs.to_string()),
        ("rx_known_failed_s", format!("{:.0}", m.totals.rx_known_failed_s)),
        ("rx_failed_beacons", m.totals.rx_failed_beacons.to_string()),
    ];
    std::fs::write(out.join("summary.json"), checks::summary_json(&checks, &m.totals, &extra))?;
    m.logs.flush();
    for c in &checks {
        eprintln!("[SIL] {:?}: {} = {} (limit {})", c.verdict, c.id, c.value, c.limit);
    }
    Ok(anomalies)
}

/// Point-mass and J2 gravity in an Earth-centred inertial frame (z the
/// pole), m/s²
fn gravity_j2(r: V3) -> V3 {
    use crate::env::math::{J2, MU_EARTH, R_EARTH};
    let r2 = r.dot(r);
    let rn = r2.sqrt();
    let k = 1.5 * J2 * (R_EARTH * R_EARTH) / r2;
    let z2 = r.z * r.z / r2;
    let s = -MU_EARTH / (r2 * rn);
    V3::new(
        s * r.x * (1.0 + k * (1.0 - 5.0 * z2)),
        s * r.y * (1.0 + k * (1.0 - 5.0 * z2)),
        s * r.z * (1.0 + k * (3.0 - 5.0 * z2)),
    )
}

/// One RK4 step of `dt` seconds under [`gravity_j2`]
fn j2_step(r: V3, v: V3, dt: f64) -> (V3, V3) {
    let k1v = gravity_j2(r);
    let k1r = v;
    let k2v = gravity_j2(r + k1r * (dt / 2.0));
    let k2r = v + k1v * (dt / 2.0);
    let k3v = gravity_j2(r + k2r * (dt / 2.0));
    let k3r = v + k2v * (dt / 2.0);
    let k4v = gravity_j2(r + k3r * dt);
    let k4r = v + k3v * dt;
    (r + (k1r + k2r * 2.0 + k3r * 2.0 + k4r) * (dt / 6.0), v + (k1v + k2v * 2.0 + k3v * 2.0 + k4v) * (dt / 6.0))
}

/// The first umbra entry (illumination falling below 0.5) at or after `t`,
/// searched in 10 s steps over two orbits (`t` if none is found)
fn next_umbra_entry(alt_m: f64, ltan_h: f64, t: f64) -> f64 {
    let orbit = Orbit::sso(alt_m, ltan_h, t);
    let lit = |tt: f64| {
        let (r, _) = orbit.state_at(tt);
        let (u, d) = sun::sun_eci(tt);
        eclipse::illumination(r, u, d) >= 0.5
    };
    let mut prev = lit(t);
    let mut tt = t;
    while tt < t + 2.0 * orbit.period() {
        tt += 10.0;
        let now = lit(tt);
        if prev && !now {
            return tt;
        }
        prev = now;
    }
    t
}

#[cfg(test)]
mod eol_tests {
    use super::*;

    /// F1's heater (ITV4030L1412NR, 11.2–20.0 Ω): both ends of its
    /// resistance fire inside the 6–35 W window at 14.0 V (9.8 and 17.5 W)
    /// and across the pack's 13.6–16.4 V; 35 Ω (the earlier model) would not
    #[test]
    fn the_fuse_heater_fires_at_both_ends_of_its_resistance() {
        for ohms in [11.2, EOL_HEATER_OHMS, 20.0] {
            for v in [13.6, 14.0, 16.4] {
                let (w, inside) = eol_heater(v, ohms);
                assert!(inside, "{ohms} ohm at {v} V: {w:.1} W");
            }
        }
        assert!((eol_heater(14.0, 20.0).0 - 9.8).abs() < 0.05);
        assert!((eol_heater(14.0, 11.2).0 - 17.5).abs() < 0.05);
        assert!(!eol_heater(14.0, 35.0).1, "5.6 W: below the window");
    }
}
